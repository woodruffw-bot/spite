//! Async ForIn/OfBodyEvaluation and AsyncIteratorClose (14.7.5, 7.4).

use super::{
    AsyncAwaitAction, AsyncExecutionData, FrameStep, Step, active_statement, child_location,
};
use crate::{
    Completion, Error, ExceptionKind, Realm, Value,
    function::IteratorRecord,
    object::async_execution::{AsyncForAwaitPhase, AsyncStatementFrame, StatementChild},
};
use spite_core::{JsString, Span, WellKnownSymbol};
use spite_parser::ast::StatementKind;

impl Realm {
    pub(super) fn get_for_await_iterator(
        &mut self,
        value: Value,
        span: Span,
    ) -> Result<IteratorRecord, Error> {
        if let Some(method) =
            self.get_method(&value, &WellKnownSymbol::AsyncIterator.symbol(), span)?
        {
            return self.get_iterator_from_method(value, method, span);
        }
        let method = self
            .get_method(&value, &WellKnownSymbol::Iterator.symbol(), span)?
            .ok_or_else(|| {
                Self::exception(
                    ExceptionKind::TypeError,
                    span,
                    "for-await-of value is not iterable",
                )
            })?;
        let record = self.get_iterator_from_method(value, method, span)?;
        self.create_async_from_sync_iterator(record, span)
    }

    pub(super) fn async_for_await_step(
        &mut self,
        data: &mut AsyncExecutionData,
    ) -> Result<Step, Error> {
        let Some(AsyncStatementFrame::ForAwait(cursor)) = data.frames.last() else {
            unreachable!("async iterator cursor")
        };
        let span = active_statement(&data.body, Some(&cursor.location)).span;
        self.tick(span)?;
        let iterator = Value::Object(cursor.iterator.iterator_handle().clone());
        match cursor.phase {
            AsyncForAwaitPhase::Next => {
                let next = cursor.iterator.next_value().clone();
                let result = self.call(next, iterator, Vec::new(), span)?;
                Ok(Step::Await(result, AsyncAwaitAction::ForAwaitNext, span))
            }
            AsyncForAwaitPhase::Close => {
                let Some(method) = self.get_method(&iterator, &JsString::from("return"), span)?
                else {
                    return self
                        .finish_async_for_await_close(data, None)
                        .map(Step::Complete);
                };
                let result = self.call(method, iterator, Vec::new(), span)?;
                Ok(Step::Await(result, AsyncAwaitAction::ForAwaitClose, span))
            }
            AsyncForAwaitPhase::Body => {
                unreachable!("active async iterator body completes before advancing")
            }
        }
    }

    pub(super) fn finish_async_for_await_next(
        &mut self,
        data: &mut AsyncExecutionData,
        value: Value,
    ) -> Result<Step, Error> {
        let Some(AsyncStatementFrame::ForAwait(cursor)) = data.frames.last() else {
            unreachable!("async next cursor")
        };
        let statement = active_statement(&data.body, Some(&cursor.location));
        let span = statement.span;
        let Value::Object(result) = value else {
            return Err(Self::exception(
                ExceptionKind::TypeError,
                span,
                "async iterator result is not an object",
            ));
        };
        if self
            .get_property(&result, &JsString::from("done"), span)?
            .to_boolean()
        {
            let Some(AsyncStatementFrame::ForAwait(cursor)) = data.frames.pop() else {
                unreachable!("exhausted async iterator cursor")
            };
            return Ok(Step::Complete(Completion::normal(Some(cursor.value))));
        }
        // Async iterator values are raw. Only the sync adapter unwraps values.
        let value = self.get_property(&result, &JsString::from("value"), span)?;
        let body = child_location(&cursor.location, StatementChild::ForOfBody, span)?;
        let StatementKind::ForOf { binding, .. } = &statement.kind else {
            unreachable!("async for-of source")
        };
        let Some(AsyncStatementFrame::ForAwait(cursor)) = data.frames.last_mut() else {
            unreachable!("async binding cursor")
        };
        // Binding failures close, unlike next/done/value failures.
        cursor.phase = AsyncForAwaitPhase::Body;
        self.initialize_async_iteration_binding(&mut data.frames, binding, value, span)?;
        data.next = Some(body);
        Ok(Step::Continue)
    }

    pub(super) fn advance_async_for_await(
        &mut self,
        data: &mut AsyncExecutionData,
        result: Result<Completion, Error>,
    ) -> FrameStep {
        let Some(AsyncStatementFrame::ForAwait(mut cursor)) = data.frames.pop() else {
            unreachable!("async iterator completion cursor")
        };
        match cursor.phase {
            AsyncForAwaitPhase::Next => FrameStep::Complete(result),
            AsyncForAwaitPhase::Close => {
                data.frames.push(AsyncStatementFrame::ForAwait(cursor));
                let Err(error) = result else {
                    unreachable!("close synchronous errors")
                };
                FrameStep::Complete(self.finish_async_for_await_close(data, Some(Err(error))))
            }
            AsyncForAwaitPhase::Body => {
                let result = result.map(|completion| {
                    completion
                        .update_empty(Some(std::mem::replace(&mut cursor.value, Value::Undefined)))
                });
                if let Ok(mut completion) = result {
                    if super::labelled::loop_continues(data, &completion) {
                        cursor.value = completion.value.take().expect("async loop value");
                        cursor.phase = AsyncForAwaitPhase::Next;
                    } else {
                        cursor.phase = AsyncForAwaitPhase::Close;
                        cursor.pending = Some(Ok(completion));
                    }
                } else {
                    cursor.value = Value::Undefined;
                    cursor.phase = AsyncForAwaitPhase::Close;
                    cursor.pending = Some(result);
                }
                data.frames.push(AsyncStatementFrame::ForAwait(cursor));
                FrameStep::Continue
            }
        }
    }

    pub(super) fn finish_async_for_await_close(
        &mut self,
        data: &mut AsyncExecutionData,
        result: Option<Result<Value, Error>>,
    ) -> Result<Completion, Error> {
        let Some(AsyncStatementFrame::ForAwait(mut cursor)) = data.frames.pop() else {
            unreachable!("async iterator close cursor")
        };
        let span = active_statement(&data.body, Some(&cursor.location)).span;
        let pending = cursor
            .pending
            .take()
            .expect("pending async iterator completion");
        let result = match result {
            Some(Err(error)) if !error.is_language_exception() => return Err(error),
            result => result,
        };
        // Original throws dominate closing language failures, including getters.
        let pending = pending?;
        if let Some(result) = result
            && !matches!(result?, Value::Object(_))
        {
            return Err(Self::exception(
                ExceptionKind::TypeError,
                span,
                "async iterator return result is not an object",
            ));
        }
        Ok(pending.consume_unlabelled_break())
    }
}
