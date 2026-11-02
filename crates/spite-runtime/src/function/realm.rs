//! Restore a function's Realm before forwarding to its next target (9.3, 10.3).

use super::Intrinsics;
use crate::{
    Error, ObjectHandle, Realm,
    environment::EnvironmentHandle,
    object::{Error as ObjectError, source_context::SourceExecutionContext},
};
use spite_core::Span;

impl Realm {
    // GetPrototypeFromConstructor uses newTarget's intrinsic graph, independently
    // of the constructor algorithm's current Realm (10.1.14, 7.3.24).
    pub(super) fn function_realm_intrinsic(
        &mut self,
        function: &ObjectHandle,
        span: Span,
        select: impl FnOnce(&Intrinsics) -> ObjectHandle,
    ) -> Result<ObjectHandle, Error> {
        let realm = self.function_realm_environment(function, span)?;
        self.realm_intrinsic(realm.as_ref(), span, select)
    }

    pub(super) fn realm_intrinsic(
        &mut self,
        realm: Option<&EnvironmentHandle>,
        span: Span,
        select: impl FnOnce(&Intrinsics) -> ObjectHandle,
    ) -> Result<ObjectHandle, Error> {
        if realm.is_none() || realm == self.record.global_environment.as_ref() {
            return Ok(select(
                self.record.intrinsics.as_ref().expect("initialized"),
            ));
        }
        let realm = realm.expect("foreign Realm");
        self.object_work(span, |objects, _| {
            let record = objects
                .environment(realm)?
                .inactive_realm
                .as_ref()
                .ok_or(ObjectError::WrongKind)?;
            if record.global_environment.as_ref() != Some(realm) {
                return Err(ObjectError::WrongKind);
            }
            let intrinsics = record.intrinsics.as_ref().ok_or(ObjectError::WrongKind)?;
            Ok(select(intrinsics))
        })
    }

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
mod tests;
