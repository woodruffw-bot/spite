//! Await for native asynchronous algorithms, without observable Promise species.

use crate::{Error, ObjectHandle, Realm, Value};
use spite_core::{JsString, Span};

#[derive(Clone, Copy, Debug)]
pub(crate) enum AsyncResumeKind {
    DisposeResources,
    ArrayFromAsync,
    Execution,
    AsyncGeneratorReturn,
    AsyncGeneratorResumeReturn,
    ModuleExecution,
    ModuleNamespace,
    ModuleImportLoad,
}

impl Realm {
    pub(super) fn await_native_value(
        &mut self,
        state: &ObjectHandle,
        kind: AsyncResumeKind,
        value: Value,
        span: Span,
    ) -> Result<(), Error> {
        let constructor = self
            .intrinsics
            .as_ref()
            .expect("initialized")
            .global_metadata
            .promise
            .clone();
        let Value::Object(promise) =
            self.promise_resolve(Value::Object(constructor), value, span)?
        else {
            unreachable!("intrinsic PromiseResolve returns a Promise");
        };
        self.react_to_native_promise(&promise, state, kind, span)
    }

    pub(crate) fn react_to_native_promise(
        &mut self,
        promise: &ObjectHandle,
        state: &ObjectHandle,
        kind: AsyncResumeKind,
        span: Span,
    ) -> Result<(), Error> {
        let fulfilled = self.native_promise_handler(state, kind, true, span)?;
        let rejected = self.native_promise_handler(state, kind, false, span)?;
        self.perform_promise_then(promise, fulfilled, rejected, None, span)?;
        Ok(())
    }

    pub(crate) fn native_promise_handler(
        &mut self,
        state: &ObjectHandle,
        kind: AsyncResumeKind,
        fulfilled: bool,
        span: Span,
    ) -> Result<Value, Error> {
        let prototype = self
            .intrinsics
            .as_ref()
            .expect("initialized")
            .function_prototype
            .clone();
        let handler = self.object_work(span, |objects, _| {
            objects.create_async_resume(&prototype, state, kind, fulfilled)
        })?;
        self.define_builtin_property(
            &handler,
            "name",
            Value::String(JsString::from("")),
            false,
            span,
        )?;
        let length = if matches!(
            kind,
            AsyncResumeKind::ModuleExecution | AsyncResumeKind::ModuleNamespace
        ) || (fulfilled && matches!(kind, AsyncResumeKind::ModuleImportLoad))
        {
            0.0
        } else {
            1.0
        };
        self.define_builtin_property(&handler, "length", Value::Number(length), false, span)?;
        Ok(Value::Object(handler))
    }
}
