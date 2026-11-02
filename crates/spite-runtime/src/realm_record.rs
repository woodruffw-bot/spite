//! Owned Realm data, separate from the Agent's heap, jobs and execution context (9.3).

use crate::{ObjectHandle, TimeZone, environment::EnvironmentHandle, function::Intrinsics};
use spite_core::JsString;
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

#[cfg(test)]
pub(crate) fn child_realm_fixture(
    realm: &mut crate::Realm,
    source: &str,
) -> (EnvironmentHandle, crate::Value) {
    use crate::object::source_context::SourceExecutionContext;

    realm.initialize_realm().unwrap();
    assert_eq!(realm.call_depth, 0);
    assert_eq!(realm.evaluation_depth, 0);
    let child = RealmRecord {
        unsupported_host_globals: realm.record.unsupported_host_globals.clone(),
        supported_import_attributes: realm.record.supported_import_attributes.clone(),
        time_zone: realm.record.time_zone.clone(),
        ..RealmRecord::default()
    };
    let parent = std::mem::replace(&mut realm.record, child);
    let caller = SourceExecutionContext {
        scopes: std::mem::take(&mut realm.scopes),
        variable: realm.variable_environment.take(),
        strict: std::mem::replace(&mut realm.strict, false),
        module: realm.active_module.take(),
    };
    let work = realm.remaining_steps;
    let job = std::mem::replace(&mut realm.job_active, true);
    realm.initialize_realm().unwrap();
    realm.remaining_steps = realm.limits.max_steps;
    let value = realm.eval(source).unwrap();
    let child = std::mem::replace(&mut realm.record, parent);
    let environment = child.global_environment.clone().unwrap();
    realm
        .objects
        .environment_mut(&environment)
        .unwrap()
        .inactive_realm = Some(Box::new(child));
    realm.scopes = caller.scopes;
    realm.variable_environment = caller.variable;
    realm.strict = caller.strict;
    realm.active_module = caller.module;
    realm.remaining_steps = work;
    realm.job_active = job;
    realm
        .objects
        .set_active_function_realm(realm.record.global_environment.as_ref().unwrap())
        .unwrap();
    (environment, value)
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
