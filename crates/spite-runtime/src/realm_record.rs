//! Owned Realm data, separate from the Agent's heap, jobs and execution context (9.3).

use crate::{
    Error, ObjectHandle, Realm, TimeZone, environment::EnvironmentHandle, function::Intrinsics,
    object::source_context::SourceExecutionContext,
};
use spite_core::{JsString, Span};
use spite_heap::Trace;
use spite_parser::ast::TemplateElement;
use std::{collections::BTreeSet, rc::Rc};

#[derive(Debug, Default)]
pub(crate) struct RealmRecord {
    pub global_environment: Option<EnvironmentHandle>,
    pub global_object: Option<ObjectHandle>,
    pub intrinsics: Option<Intrinsics>,
    pub template_map: Vec<(Rc<[TemplateElement]>, ObjectHandle)>,
    pub unsupported_host_globals: BTreeSet<String>,
    pub inspectable_host_absences: BTreeSet<String>,
    pub supported_import_attributes: BTreeSet<JsString>,
    pub time_zone: Option<TimeZone>,
}

impl Trace for RealmRecord {
    fn trace(&self) -> impl Iterator<Item = Option<&ObjectHandle>> {
        self.global_environment
            .iter()
            .map(|environment| Some(&environment.0))
            .chain(self.global_object.iter().map(Some))
            .chain(
                self.intrinsics
                    .iter()
                    .flat_map(|intrinsics| intrinsics.roots())
                    .map(Some),
            )
            .chain(self.template_map.iter().map(|(_, template)| Some(template)))
    }
}

impl Realm {
    // Bootstrap a new Realm on this Agent's heap. Fixed intrinsic and host setup
    // runs outside Script work limits; initialize must restore an allowance
    // before executing any source. Neither initialization nor evaluation
    // collects, so the temporarily owned caller record stays live throughout.
    pub(super) fn with_new_realm<T>(
        &mut self,
        span: Span,
        initialize: impl FnOnce(&mut Self) -> Result<T, Error>,
    ) -> Result<(EnvironmentHandle, T), Error> {
        let parent_environment =
            self.record
                .global_environment
                .clone()
                .ok_or_else(|| Error::Host {
                    span,
                    message: "caller Realm is not initialized".into(),
                })?;
        let child = RealmRecord {
            unsupported_host_globals: self.record.unsupported_host_globals.clone(),
            inspectable_host_absences: self.record.inspectable_host_absences.clone(),
            supported_import_attributes: self.record.supported_import_attributes.clone(),
            time_zone: self.record.time_zone.clone(),
            ..RealmRecord::default()
        };
        let parent = std::mem::replace(&mut self.record, child);
        let caller = SourceExecutionContext {
            scopes: std::mem::take(&mut self.scopes),
            variable: self.variable_environment.take(),
            strict: std::mem::replace(&mut self.strict, false),
            module: self.active_module.take(),
        };
        let tail_scope = self.tail_scope.take();
        let work = self.remaining_steps;
        let job = std::mem::replace(&mut self.job_active, true);
        self.objects.clear_active_function_realm();
        let result = self.initialize_realm().and_then(|()| initialize(self));
        let result = match result {
            Err(error @ Error::Exception { .. }) if self.record.intrinsics.is_some() => {
                match self.language_exception_value(error) {
                    Ok(value) => Err(Error::Thrown(value)),
                    Err(error) => Err(error),
                }
            }
            result => result,
        };
        let child = std::mem::replace(&mut self.record, parent);
        let environment = child.global_environment.clone();
        if let Some(environment) = &environment {
            // Keep even a partially initialized record coherent until idle GC.
            // A thrown language value may also retain this Realm on failure.
            let target = self
                .objects
                .environment_mut(environment)
                .expect("bootstrap environment remains live without collection");
            debug_assert!(target.inactive_realm.is_none());
            target.inactive_realm = Some(Box::new(child));
        }
        self.scopes = caller.scopes;
        self.variable_environment = caller.variable;
        self.strict = caller.strict;
        self.active_module = caller.module;
        self.tail_scope = tail_scope;
        self.remaining_steps = work;
        self.job_active = job;
        self.objects
            .set_active_function_realm(&parent_environment)
            .expect("caller remains live without collection");
        result.map(|value| (environment.expect("initialized child Realm"), value))
    }
}

#[cfg(test)]
pub(crate) fn child_realm_fixture(
    realm: &mut Realm,
    source: &str,
) -> (EnvironmentHandle, crate::Value) {
    realm.initialize_realm().unwrap();
    assert_eq!(realm.call_depth, 0);
    assert_eq!(realm.evaluation_depth, 0);
    realm
        .with_new_realm(Span::new(0, 0), |realm| {
            realm.remaining_steps = realm.limits.max_steps;
            realm.eval(source)
        })
        .unwrap()
}

#[cfg(test)]
mod tests {
    use super::child_realm_fixture;
    use crate::{Realm, Value};

    #[test]
    fn retained_function_keeps_inactive_realm_intrinsics_and_templates_until_collection() {
        let mut realm = Realm::default();
        // A same-agent fixture: only Realm data changes, while the heap stays put.
        let (child_environment, function) = child_realm_fixture(
            &mut realm,
            "function tag(template){return template;}\
                 function keep(){return tag`held`;}keep();\
                 let held=Object.assign;Object.setPrototypeOf(held,null);delete Object.assign;\
                 delete Object.prototype.constructor;delete globalThis.Object;held",
        );
        let child = realm
            .objects
            .environment(&child_environment)
            .unwrap()
            .inactive_realm
            .as_ref()
            .unwrap();
        let child_global = child.global_object.clone().unwrap();
        let child_object = child
            .intrinsics
            .as_ref()
            .unwrap()
            .object
            .constructor
            .clone();
        let child_template = child.template_map[0].1.clone();
        assert!(matches!(&function, Value::Object(_)));
        let root = realm.root_value(function, 1000).unwrap();
        realm.collect(usize::MAX).unwrap();
        assert!(realm.objects.environment(&child_environment).is_ok());
        assert!(realm.objects.inspect(&child_global).is_ok());
        assert!(realm.objects.inspect(&child_object).is_ok());
        assert!(realm.objects.inspect(&child_template).is_ok());

        drop(root);
        assert_eq!(
            realm.collect(usize::MAX).unwrap().live,
            crate::test_support::REALM_ENTRIES
        );
        assert!(realm.objects.environment(&child_environment).is_err());
        assert!(realm.objects.inspect(&child_global).is_err());
        assert!(realm.objects.inspect(&child_object).is_err());
        assert!(realm.objects.inspect(&child_template).is_err());
    }
}
