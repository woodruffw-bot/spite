//! Restore a function's Realm before forwarding to its next target (9.3, 10.3).

use crate::{
    Error, Realm,
    environment::EnvironmentHandle,
    object::{Error as ObjectError, source_context::SourceExecutionContext},
};
use spite_core::Span;

impl Realm {
    pub(super) fn with_function_realm<T>(
        &mut self,
        target: &EnvironmentHandle,
        span: Span,
        execute: impl FnOnce(&mut Self) -> Result<T, Error>,
    ) -> Result<T, Error> {
        if self.record.global_environment.as_ref() == Some(target) {
            return execute(self);
        }
        let current = self
            .record
            .global_environment
            .as_ref()
            .ok_or_else(|| Error::Host {
                span,
                message: "function Realm is not initialized".into(),
            })?
            .clone();
        let mut scopes = Vec::new();
        scopes.try_reserve_exact(1).map_err(|_| Error::Limit {
            span,
            message: "function Realm scope allocation failed".into(),
        })?;
        scopes.push(target.clone());
        // Pay and validate before moving any state. Evaluation and allocation
        // never collect, so these checked environments stay live until restore.
        let mut inactive = self.object_work(span, |objects, budget| {
            budget.charge(1)?;
            if objects.environment(&current)?.inactive_realm.is_some() {
                return Err(ObjectError::WrongKind);
            }
            let record = objects
                .environment(target)?
                .inactive_realm
                .as_ref()
                .ok_or(ObjectError::WrongKind)?;
            if record.global_environment.as_ref() != Some(target)
                || record.global_object.is_none()
                || record.intrinsics.is_none()
            {
                return Err(ObjectError::WrongKind);
            }
            Ok(objects
                .environment_mut(target)?
                .inactive_realm
                .take()
                .expect("validated inactive Realm"))
        })?;
        std::mem::swap(&mut self.record, inactive.as_mut());
        self.objects
            .environment_mut(&current)
            .expect("validated active global environment")
            .inactive_realm = Some(inactive);
        self.objects
            .set_active_function_realm(target)
            .expect("validated target global environment");
        let tail_scope = self.tail_scope.take();
        let context = SourceExecutionContext {
            scopes,
            variable: Some(target.clone()),
            strict: false,
            // Builtin contexts have null ScriptOrModule; GetActiveScriptOrModule
            // still sees the suspended caller. Source drivers install their own.
            module: self.active_module.clone(),
        };
        let result = self.with_source_execution_context(context, |realm| {
            match execute(realm) {
                Err(error @ Error::Exception { .. }) => {
                    // Create the language Error object before leaving its Realm.
                    match realm.language_exception_value(error) {
                        Ok(value) => Err(Error::Thrown(value)),
                        Err(error) => Err(error),
                    }
                }
                result => result,
            }
        });
        let mut inactive = self
            .objects
            .environment_mut(&current)
            .expect("active caller environment remains live")
            .inactive_realm
            .take()
            .expect("caller Realm was saved");
        std::mem::swap(&mut self.record, inactive.as_mut());
        self.objects
            .environment_mut(target)
            .expect("target environment remains live")
            .inactive_realm = Some(inactive);
        self.objects
            .set_active_function_realm(&current)
            .expect("caller environment remains live");
        self.tail_scope = tail_scope;
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Limits, ObjectHandle, Value, realm_record::child_realm_fixture};

    fn expose(realm: &mut Realm, name: &str, value: Value) {
        let global = realm.global_object();
        realm
            .define_builtin_property(&global, name, value, true, Span::new(0, 0))
            .unwrap();
    }

    fn expose_child(
        realm: &mut Realm,
        environment: &EnvironmentHandle,
        function: Value,
    ) -> ObjectHandle {
        let global = realm
            .objects
            .environment(environment)
            .unwrap()
            .inactive_realm
            .as_ref()
            .unwrap()
            .global_object
            .clone()
            .unwrap();
        expose(realm, "foreign", function);
        expose(realm, "child", Value::Object(global.clone()));
        global
    }

    #[test]
    fn foreign_calls_use_creation_intrinsics_and_restore_the_caller() {
        let mut realm = Realm::default();
        let (environment, function) = child_realm_fixture(
            &mut realm,
            "var mark=7;function receiver(){return this;}function f(){return {array:[],boxed:Object(7),global:globalThis};}f",
        );
        expose_child(&mut realm, &environment, function);
        assert_eq!(
            realm.eval(
                r#"var mark=42;let compiled=child.Function('return mark');let result=foreign();
                let inspector=child.Object.getOwnPropertyDescriptor;
                Object.setPrototypeOf(inspector,null);
                let descriptor=Reflect.apply(inspector,null,[{x:1},'x']);
                Object.getPrototypeOf(result)===child.Object.prototype &&
                Object.getPrototypeOf(result.array)===child.Array.prototype &&
                Object.getPrototypeOf(result.boxed)===child.Number.prototype &&
                result.global===child &&
                Object.getPrototypeOf(child.receiver.call('x'))===child.String.prototype &&
                Object.getPrototypeOf(descriptor)===child.Object.prototype &&
                compiled()===7 && Object.getPrototypeOf(compiled)===child.Function.prototype &&
                Object.getPrototypeOf([])===Array.prototype"#,
            ),
            Ok(Value::Boolean(true))
        );
        assert_ne!(realm.record.global_environment.as_ref(), Some(&environment));
        assert_eq!(realm.scopes.len(), 1);
    }

    #[test]
    fn foreign_errors_and_host_aborts_restore_the_owned_records() {
        let mut realm = Realm::new(Limits {
            max_steps: Some(10_000),
            ..Limits::default()
        });
        realm.reserve_unsupported_global("missingHost");
        let (environment, function) = child_realm_fixture(
            &mut realm,
            "var marker={};function f(mode){if(mode===0)return null.x;\
             if(mode===1)throw marker;globalThis.started=true;\
             if(mode===2)missingHost();while(true){}}f",
        );
        expose_child(&mut realm, &environment, function);
        let parent = realm.record.global_environment.clone().unwrap();
        for source in [
            "try{foreign(0)}catch(e){e instanceof child.TypeError && !(e instanceof TypeError)}",
            "try{child.Object.keys(null)}catch(e){e instanceof child.TypeError && !(e instanceof TypeError)}",
            "try{child.Function('return (')}catch(e){e instanceof child.SyntaxError && !(e instanceof SyntaxError)}",
            "try{foreign(1)}catch(e){e===child.marker}",
        ] {
            assert_eq!(realm.eval(source), Ok(Value::Boolean(true)), "{source}");
        }
        assert!(matches!(
            realm.eval("var effects=0;try{foreign(2)}catch(e){effects=1}finally{effects=2}"),
            Err(Error::Unsupported { .. })
        ));
        assert_eq!(
            realm.eval("effects===0 && child.started===true"),
            Ok(Value::Boolean(true))
        );
        assert!(matches!(
            realm.eval("child.started=false;try{foreign(3)}catch(e){effects=1}finally{effects=2}"),
            Err(Error::Limit { .. })
        ));
        assert_eq!(
            realm.eval("effects===0 && child.started===true"),
            Ok(Value::Boolean(true))
        );
        assert_eq!(realm.record.global_environment.as_ref(), Some(&parent));
        let Value::Object(local) = realm.eval("(function(){})").unwrap() else {
            panic!()
        };
        assert_eq!(
            realm.objects.inspect(&local).unwrap().function_realm(),
            Some(&parent)
        );
        assert_eq!(realm.call_depth, 0);
        assert_eq!(realm.scopes.len(), 1);
    }

    #[test]
    fn cross_realm_tail_calls_restore_each_record_before_forwarding() {
        let mut realm = Realm::default();
        realm
            .eval("const expected=globalThis;function parent(n){'use strict';return n===0?globalThis===expected:child.foreign(n-1);}")
            .unwrap();
        let (environment, function) = child_realm_fixture(
            &mut realm,
            "const expected=globalThis;function foreign(n){'use strict';return n===0?globalThis===expected:parent.parent(n-1);}foreign",
        );
        let global = expose_child(&mut realm, &environment, function);
        let parent_global = realm.global_object();
        realm
            .define_builtin_property(
                &global,
                "parent",
                Value::Object(parent_global),
                true,
                Span::new(0, 0),
            )
            .unwrap();
        assert_eq!(
            realm.eval("parent(4096) && child.foreign(4097)"),
            Ok(Value::Boolean(true))
        );
        assert_eq!(realm.call_depth, 0);
        assert_eq!(realm.evaluation_depth, 0);
        assert_eq!(realm.scopes.len(), 1);
        assert!(
            realm
                .objects
                .environment(realm.record.global_environment.as_ref().unwrap())
                .unwrap()
                .inactive_realm
                .is_none()
        );
        assert!(
            realm
                .objects
                .environment(&environment)
                .unwrap()
                .inactive_realm
                .is_some()
        );
    }

    #[test]
    fn proxy_bound_calls_and_template_caches_preserve_their_realms() {
        let mut realm = Realm::default();
        let (environment, function) = child_realm_fixture(
            &mut realm,
            "function tag(t){return t;}function f(){return tag`site`;}f",
        );
        expose_child(&mut realm, &environment, function);
        assert_eq!(
            realm.eval(
                r#"function tag(t){return t;}let own=tag`site`;
                let first=foreign(),second=foreign();
                let wrapped=new Proxy(foreign,{apply(target,receiver,args){
                    return Object.getPrototypeOf(args)===Array.prototype &&
                        Object.getPrototypeOf(target())===child.Array.prototype;
                }});
                let bound=foreign.bind(null);Object.setPrototypeOf(bound,null);
                wrapped(1,2) && bound()===first && first===second && first!==own &&
                Object.getPrototypeOf(first)===child.Array.prototype &&
                Object.getPrototypeOf(own)===Array.prototype"#,
            ),
            Ok(Value::Boolean(true))
        );
        realm.collect(usize::MAX).unwrap();
        assert_eq!(realm.eval("foreign()===first"), Ok(Value::Boolean(true)));
    }
}
