//! FinalizationRegistry construction and cell registration (26.2).

use crate::{Error, ExceptionKind, ObjectHandle, Realm, Value};
use spite_core::{JsString, Span};

impl Realm {
    pub(super) fn finalization_registry_constructor(
        &mut self,
        new_target: ObjectHandle,
        callback: Value,
        span: Span,
    ) -> Result<Value, Error> {
        // The ordinary builtin-call dispatcher rejects a missing NewTarget.
        // IsCallable precedes every observable prototype access, and a revoked
        // callable Proxy still passes this immutable capability check.
        if !self.is_callable(&callback, span)? {
            return Err(Self::exception(
                ExceptionKind::TypeError,
                span,
                "FinalizationRegistry cleanup callback must be callable",
            ));
        }
        let Value::Object(callback) = callback else {
            unreachable!("callable values are objects");
        };
        let prototype = self.get_property(&new_target, &JsString::from("prototype"), span)?;
        let prototype = if let Value::Object(prototype) = prototype {
            prototype
        } else {
            self.check_function_realm(&new_target, span)?;
            self.intrinsics
                .as_ref()
                .expect("initialized realm")
                .global_metadata
                .finalization_registry_prototype
                .clone()
        };
        // HostMakeJobCallback's required non-browser default stores this exact
        // function. The owning Objects store is the active constructor's Realm;
        // the callback's target chain is not a Realm lookup here.
        self.object_work(span, |objects, budget| {
            objects.create_finalization_registry(&prototype, &callback, budget)
        })
        .map(Value::Object)
    }

    pub(super) fn finalization_registry_register(
        &mut self,
        receiver: Value,
        target: Value,
        held: Value,
        token: Value,
        span: Span,
    ) -> Result<Value, Error> {
        let registry = self.finalization_registry_receiver(receiver, span)?;
        if !self.can_be_held_weakly(&target, span)? {
            return Err(Self::exception(
                ExceptionKind::TypeError,
                span,
                "FinalizationRegistry target cannot be held weakly",
            ));
        }
        // A valid target is only an Object or Symbol identity, so this check
        // cannot traverse or copy an arbitrary held BigInt or string payload.
        if target.same_value(&held) {
            return Err(Self::exception(
                ExceptionKind::TypeError,
                span,
                "FinalizationRegistry target and held value must differ",
            ));
        }
        let token = if self.can_be_held_weakly(&token, span)? {
            Some(&token)
        } else if matches!(token, Value::Undefined) {
            None
        } else {
            return Err(Self::exception(
                ExceptionKind::TypeError,
                span,
                "FinalizationRegistry unregister token cannot be held weakly",
            ));
        };
        self.object_work(span, |objects, budget| {
            objects.finalization_registry_register(&registry, &target, held, token, budget)
        })?;
        Ok(Value::Undefined)
    }

    pub(super) fn finalization_registry_unregister(
        &mut self,
        receiver: Value,
        token: Value,
        span: Span,
    ) -> Result<Value, Error> {
        let registry = self.finalization_registry_receiver(receiver, span)?;
        if !self.can_be_held_weakly(&token, span)? {
            return Err(Self::exception(
                ExceptionKind::TypeError,
                span,
                "FinalizationRegistry unregister token cannot be held weakly",
            ));
        }
        self.object_work(span, |objects, budget| {
            objects.finalization_registry_unregister(&registry, &token, budget)
        })
        .map(Value::Boolean)
    }

    fn finalization_registry_receiver(
        &mut self,
        receiver: Value,
        span: Span,
    ) -> Result<ObjectHandle, Error> {
        if let Value::Object(registry) = receiver
            && self.object_work(span, |objects, _| {
                objects.finalization_registry_has_slot(&registry)
            })?
        {
            return Ok(registry);
        }
        // RequireInternalSlot never unwraps Proxy receivers or reads public
        // properties, including when that Proxy has already been revoked.
        Err(Self::exception(
            ExceptionKind::TypeError,
            span,
            "receiver has no FinalizationRegistry Cells internal slot",
        ))
    }
}
