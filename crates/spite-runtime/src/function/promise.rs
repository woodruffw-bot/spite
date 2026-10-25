//! Shared NewPromiseCapability and generic Promise static methods (27.2).

use crate::{Error, ExceptionKind, ObjectHandle, Realm, Value};
use spite_core::{JsString, Span};

pub(super) struct PromiseCapability {
    pub promise: Value,
    pub resolve: Value,
    pub reject: Value,
}

impl Realm {
    // NewPromiseCapability (27.2.1.5) uses the supplied constructor without
    // branding its result. Its executor captures a private traced Record.
    pub(super) fn new_promise_capability(
        &mut self,
        constructor: Value,
        span: Span,
    ) -> Result<PromiseCapability, Error> {
        if !self.is_constructor(&constructor, span)? {
            return Err(Self::exception(
                ExceptionKind::TypeError,
                span,
                "Promise capability requires a constructor",
            ));
        }
        let capability = self.object_work(span, |objects, _| objects.create(None))?;
        for key in ["resolve", "reject"] {
            self.define_builtin_property(&capability, key, Value::Undefined, true, span)?;
        }
        let prototype = self
            .intrinsics
            .as_ref()
            .expect("initialized")
            .function_prototype
            .clone();
        let executor = self.object_work(span, |objects, _| {
            objects.create_promise_capability_executor(&prototype, &capability)
        })?;
        self.define_builtin_property(&executor, "length", Value::Number(2.0), false, span)?;
        self.define_builtin_property(
            &executor,
            "name",
            Value::String(JsString::from("")),
            false,
            span,
        )?;
        let promise = self.construct(constructor, vec![Value::Object(executor)], span)?;
        let resolve = self.get_property(&capability, &JsString::from("resolve"), span)?;
        let reject = self.get_property(&capability, &JsString::from("reject"), span)?;
        if !self.is_callable(&resolve, span)? || !self.is_callable(&reject, span)? {
            return Err(Self::exception(
                ExceptionKind::TypeError,
                span,
                "Promise capability functions must be callable",
            ));
        }
        Ok(PromiseCapability {
            promise,
            resolve,
            reject,
        })
    }

    pub(super) fn promise_capability_executor(
        &mut self,
        capability: ObjectHandle,
        mut arguments: std::vec::IntoIter<Value>,
        span: Span,
    ) -> Result<Value, Error> {
        // Check both existing fields before changing either. In particular a
        // prior call with undefined/undefined does not prevent a later call.
        let resolve = self.get_property(&capability, &JsString::from("resolve"), span)?;
        let reject = self.get_property(&capability, &JsString::from("reject"), span)?;
        if !matches!(resolve, Value::Undefined) || !matches!(reject, Value::Undefined) {
            return Err(Self::exception(
                ExceptionKind::TypeError,
                span,
                "Promise capability executor was already initialized",
            ));
        }
        self.define_builtin_property(
            &capability,
            "resolve",
            arguments.next().unwrap_or(Value::Undefined),
            true,
            span,
        )?;
        self.define_builtin_property(
            &capability,
            "reject",
            arguments.next().unwrap_or(Value::Undefined),
            true,
            span,
        )?;
        Ok(Value::Undefined)
    }

    pub(super) fn promise_with_resolvers(
        &mut self,
        constructor: Value,
        span: Span,
    ) -> Result<Value, Error> {
        let capability = self.new_promise_capability(constructor, span)?;
        let prototype = self
            .intrinsics
            .as_ref()
            .expect("initialized")
            .object_prototype
            .clone();
        let object = self.object_work(span, |objects, _| objects.create(Some(&prototype)))?;
        // CreateDataPropertyOrThrow gives all three fields writable/enumerable/
        // configurable attributes and bypasses inherited setters.
        for (key, value) in [
            ("promise", capability.promise),
            ("resolve", capability.resolve),
            ("reject", capability.reject),
        ] {
            self.define_property_or_throw(
                &object,
                JsString::from(key),
                crate::object::DataDescriptor {
                    value: Some(value),
                    writable: Some(true),
                    enumerable: Some(true),
                    configurable: Some(true),
                }
                .into(),
                span,
            )?;
        }
        Ok(Value::Object(object))
    }

    pub(super) fn promise_reject(
        &mut self,
        constructor: Value,
        reason: Value,
        span: Span,
    ) -> Result<Value, Error> {
        let capability = self.new_promise_capability(constructor, span)?;
        self.call(capability.reject, Value::Undefined, vec![reason], span)?;
        Ok(capability.promise)
    }

    pub(super) fn promise_try(
        &mut self,
        constructor: Value,
        mut arguments: std::vec::IntoIter<Value>,
        span: Span,
    ) -> Result<Value, Error> {
        // Edition 17 sec-promise.try checks only the receiver's type before
        // invoking the callback. Capability construction follows that Call.
        if !matches!(constructor, Value::Object(_)) {
            return Err(Self::exception(
                ExceptionKind::TypeError,
                span,
                "Promise.try requires an object receiver",
            ));
        }
        let callback = arguments.next().unwrap_or(Value::Undefined);
        match self.call(callback, Value::Undefined, arguments.collect(), span) {
            Ok(value) => {
                // PromiseResolve's branded identity case becomes reachable when
                // intrinsic Promise construction is implemented. Existing values
                // require a fresh capability, including user-created lookalikes.
                let capability = self.new_promise_capability(constructor, span)?;
                self.call(capability.resolve, Value::Undefined, vec![value], span)?;
                Ok(capability.promise)
            }
            Err(error) => {
                let value = self.language_exception_value(error)?;
                let capability = self.new_promise_capability(constructor, span)?;
                self.call(capability.reject, Value::Undefined, vec![value], span)?;
                Ok(capability.promise)
            }
        }
    }
}
