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
        let prototype = self
            .intrinsics
            .as_ref()
            .expect("initialized")
            .function_prototype
            .clone();
        let mut handlers = [Value::Undefined, Value::Undefined];
        for (slot, fulfilled) in handlers.iter_mut().zip([true, false]) {
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
            self.define_builtin_property(&handler, "length", Value::Number(1.0), false, span)?;
            *slot = Value::Object(handler);
        }
        let [fulfilled, rejected] = handlers;
        self.perform_promise_then(&promise, fulfilled, rejected, None, span)?;
        Ok(())
    }
}
