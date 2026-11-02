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
mod tests {
    use crate::{Realm, Value, object::source_context::SourceExecutionContext};

    #[test]
    fn retained_function_keeps_inactive_realm_intrinsics_and_templates_until_collection() {
        let mut realm = Realm::default();
        realm.eval("0").unwrap();
        let parent = std::mem::take(&mut realm.record);
        let caller = SourceExecutionContext {
            scopes: std::mem::take(&mut realm.scopes),
            variable: realm.variable_environment.take(),
            strict: std::mem::replace(&mut realm.strict, false),
            module: realm.active_module.take(),
        };
        // A same-agent fixture: only Realm data changes, while the heap stays put.
        let function = realm
            .eval(
                "function tag(template){return template;}\
                 function keep(){return tag`held`;}keep();\
                 let held=Object.assign;Object.setPrototypeOf(held,null);delete Object.assign;\
                 delete Object.prototype.constructor;delete globalThis.Object;held",
            )
            .unwrap();
        let child = std::mem::replace(&mut realm.record, parent);
        let child_environment = child.global_environment.clone().unwrap();
        let child_global = child.global_object.clone().unwrap();
        let child_object = child
            .intrinsics
            .as_ref()
            .unwrap()
            .object
            .constructor
            .clone();
        let child_template = child.template_map[0].1.clone();
        realm
            .objects
            .environment_mut(&child_environment)
            .unwrap()
            .inactive_realm = Some(Box::new(child));
        realm.scopes = caller.scopes;
        realm.variable_environment = caller.variable;
        realm.strict = caller.strict;
        realm.active_module = caller.module;
        realm
            .objects
            .set_active_function_realm(realm.record.global_environment.as_ref().unwrap())
            .unwrap();

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
