//! Synchronous YieldExpression delegation and abrupt iterator forwarding (15.5).

use super::{AsyncAwaitAction, AsyncExecutionData, SourceResume, Step};
use crate::{Completion, CompletionKind, Error, ExceptionKind, Realm, Value};
use spite_core::JsString;

impl Realm {
    pub(super) fn resume_yield_delegation(
        &mut self,
        data: &mut AsyncExecutionData,
        received: SourceResume,
    ) -> Result<Step, Error> {
        let delegation = data
            .expression
            .as_mut()
            .expect("delegating expression")
            .delegation
            .take()
            .expect("active yield delegation");
        let span = delegation.span;
        let receiver = Value::Object(delegation.iterator.iterator_handle().clone());
        let (method, argument, returning) = match received {
            SourceResume::Value(value) => (delegation.iterator.next_value().clone(), value, false),
            SourceResume::Throw(Error::Thrown(value)) => {
                let Some(method) = self.get_method(&receiver, &JsString::from("throw"), span)?
                else {
                    // A missing throw method closes with a normal completion;
                    // cleanup failure precedes the protocol TypeError.
                    self.iterator_close(&delegation.iterator, span)?;
                    return Err(Self::exception(
                        ExceptionKind::TypeError,
                        span,
                        "delegated iterator has no throw method",
                    ));
                };
                (method, value, false)
            }
            SourceResume::Throw(error) => return Err(error),
            SourceResume::Return(value) => {
                let Some(method) = self.get_method(&receiver, &JsString::from("return"), span)?
                else {
                    return Ok(Step::Complete(Completion {
                        kind: CompletionKind::Return,
                        value: Some(value),
                        target: None,
                    }));
                };
                (method, value, true)
            }
        };
        let result = self.call(method, receiver, vec![argument], span)?;
        let Value::Object(object) = &result else {
            return Err(Self::exception(
                ExceptionKind::TypeError,
                span,
                "delegated iterator result is not an object",
            ));
        };
        let done = self.get_property(object, &JsString::from("done"), span)?;
        if done.to_boolean() {
            let value = self.get_property(object, &JsString::from("value"), span)?;
            if returning {
                return Ok(Step::Complete(Completion {
                    kind: CompletionKind::Return,
                    value: Some(value),
                    target: None,
                }));
            }
            return self.resume_async_expression(data, Some(value));
        }
        data.expression
            .as_mut()
            .expect("delegating expression")
            .delegation = Some(delegation);
        // GeneratorYield forwards this exact object without reading its value.
        Ok(Step::Yield(result, AsyncAwaitAction::Expression, span))
    }
}
