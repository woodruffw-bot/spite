//! Restore a function's Realm before forwarding to its next target (9.3, 10.3).

use super::Intrinsics;
use crate::{
    Error, ObjectHandle, Realm,
    environment::EnvironmentHandle,
    object::{Error as ObjectError, source_context::SourceExecutionContext},
};
use spite_core::Span;

#[derive(Debug)]
#[must_use = "restore this transition with leave_function_realm"]
pub(super) struct RealmTransition {
    previous: EnvironmentHandle,
    target: EnvironmentHandle,
    context: SourceExecutionContext,
    tail_scope: Option<usize>,
}

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
        let transition = self.enter_function_realm(target, span)?;
        let result = execute(self);
        self.leave_function_realm(transition, result)
    }

    pub(super) fn enter_function_realm(
        &mut self,
        target: &EnvironmentHandle,
        span: Span,
    ) -> Result<Option<RealmTransition>, Error> {
        if self.record.global_environment.as_ref() == Some(target) {
            return Ok(None);
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
            scopes: std::mem::replace(&mut self.scopes, scopes),
            variable: self.variable_environment.replace(target.clone()),
            strict: std::mem::replace(&mut self.strict, false),
            // Builtin contexts have null ScriptOrModule; GetActiveScriptOrModule
            // still sees the suspended caller. Source drivers install their own.
            module: self.active_module.clone(),
        };
        Ok(Some(RealmTransition {
            previous: current,
            target: target.clone(),
            context,
            tail_scope,
        }))
    }

    pub(super) fn leave_function_realm<T>(
        &mut self,
        transition: Option<RealmTransition>,
        result: Result<T, Error>,
    ) -> Result<T, Error> {
        let Some(transition) = transition else {
            return result;
        };
        debug_assert_eq!(
            self.record.global_environment.as_ref(),
            Some(&transition.target)
        );
        let result = match result {
            Err(error @ Error::Exception { .. }) => {
                // Create the language Error object before leaving its Realm.
                match self.language_exception_value(error) {
                    Ok(value) => Err(Error::Thrown(value)),
                    Err(error) => Err(error),
                }
            }
            result => result,
        };
        let mut inactive = self
            .objects
            .environment_mut(&transition.previous)
            .expect("active caller environment remains live")
            .inactive_realm
            .take()
            .expect("caller Realm was saved");
        std::mem::swap(&mut self.record, inactive.as_mut());
        self.objects
            .environment_mut(&transition.target)
            .expect("target environment remains live")
            .inactive_realm = Some(inactive);
        self.objects
            .set_active_function_realm(&transition.previous)
            .expect("caller environment remains live");
        self.scopes = transition.context.scopes;
        self.variable_environment = transition.context.variable;
        self.strict = transition.context.strict;
        self.active_module = transition.context.module;
        self.tail_scope = transition.tail_scope;
        result
    }
}

#[cfg(test)]
mod tests;
