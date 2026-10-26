//! YieldExpression delegation and abrupt iterator forwarding (15.5).

use super::{AsyncAwaitAction, AsyncExecutionData, SourceResume, Step};
use crate::object::async_expression::DelegationOperation;
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
        let (method, arguments, operation) = match received {
            SourceResume::Value(value) => (
                delegation.iterator.next_value().clone(),
                vec![value],
                DelegationOperation::Next,
            ),
            SourceResume::Throw(Error::Thrown(value)) => {
                let Some(method) = self.get_method(&receiver, &JsString::from("throw"), span)?
                else {
                    if data.function.is_async_generator() {
                        let Some(method) =
                            self.get_method(&receiver, &JsString::from("return"), span)?
                        else {
                            return Err(Self::missing_delegated_throw(span));
                        };
                        let result = self.call(method, receiver, Vec::new(), span)?;
                        data.expression
                            .as_mut()
                            .expect("delegating expression")
                            .delegation = Some(delegation);
                        return Ok(Step::Await(
                            result,
                            AsyncAwaitAction::DelegationResult(DelegationOperation::Close),
                            span,
                        ));
                    }
                    // A missing throw method closes with a normal completion;
                    // cleanup failure precedes the protocol TypeError.
                    self.iterator_close(&delegation.iterator, span)?;
                    return Err(Self::missing_delegated_throw(span));
                };
                (method, vec![value], DelegationOperation::Next)
            }
            SourceResume::Throw(error) => return Err(error),
            SourceResume::Return(value) => {
                let Some(method) = self.get_method(&receiver, &JsString::from("return"), span)?
                else {
                    if data.function.is_async_generator() {
                        // Yield* awaits a missing-return completion separately
                        // from AsyncGeneratorUnwrapYieldResumption (15.5).
                        return Ok(Step::Await(
                            value,
                            AsyncAwaitAction::AsyncGeneratorReturn,
                            span,
                        ));
                    }
                    return Ok(Step::Complete(Completion {
                        kind: CompletionKind::Return,
                        value: Some(value),
                        target: None,
                    }));
                };
                (method, vec![value], DelegationOperation::Return)
            }
        };
        let result = self.call(method, receiver, arguments, span)?;
        data.expression
            .as_mut()
            .expect("delegating expression")
            .delegation = Some(delegation);
        if data.function.is_async_generator() {
            return Ok(Step::Await(
                result,
                AsyncAwaitAction::DelegationResult(operation),
                span,
            ));
        }
        self.finish_yield_delegation(data, result, operation)
    }

    pub(super) fn finish_yield_delegation(
        &mut self,
        data: &mut AsyncExecutionData,
        result: Value,
        operation: DelegationOperation,
    ) -> Result<Step, Error> {
        let delegation = data
            .expression
            .as_mut()
            .expect("delegating expression")
            .delegation
            .take()
            .expect("active yield delegation");
        let span = delegation.span;
        let Value::Object(object) = &result else {
            return Err(Self::exception(
                ExceptionKind::TypeError,
                span,
                "delegated iterator result is not an object",
            ));
        };
        if matches!(operation, DelegationOperation::Close) {
            // AsyncIteratorClose requires an object without reading done/value.
            return Err(Self::missing_delegated_throw(span));
        }
        let done = self.get_property(object, &JsString::from("done"), span)?;
        if done.to_boolean() {
            let value = self.get_property(object, &JsString::from("value"), span)?;
            if matches!(operation, DelegationOperation::Return) {
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
        let value = if data.function.is_async_generator() {
            // True async iterators yield raw values. The sync adapter performs
            // its own unwrapping; AsyncGeneratorYield does not await here.
            self.get_property(object, &JsString::from("value"), span)?
        } else {
            // GeneratorYield forwards this exact object without reading value.
            result
        };
        Ok(Step::Yield(value, AsyncAwaitAction::Expression, span))
    }

    fn missing_delegated_throw(span: spite_core::Span) -> Error {
        Self::exception(
            ExceptionKind::TypeError,
            span,
            "delegated iterator has no throw method",
        )
    }
}
