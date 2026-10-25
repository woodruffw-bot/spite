//! Synchronous ForOf iteration with suspended source expressions and bodies.

use super::{
    AsyncAwaitAction, AsyncExecutionData, FrameStep, Step, active_statement, child_location,
    expression,
};
use crate::{
    Completion, CompletionKind, Error, ExceptionKind, Realm, Value,
    object::{
        async_execution::{AsyncForOfCursor, AsyncForOfHead, AsyncStatementFrame, StatementChild},
        async_expression::ExprRoot,
    },
};
use spite_core::{Span, WellKnownSymbol};
use spite_parser::ast::{ForBinding, StatementKind};

fn allocation_error(span: Span) -> Error {
    Error::Limit {
        span,
        message: "async for-of continuation allocation failed".into(),
    }
}

impl Realm {
    pub(super) fn enter_async_for_of(
        &mut self,
        data: &mut AsyncExecutionData,
        span: Span,
    ) -> Result<Step, Error> {
        data.frames
            .try_reserve(1)
            .map_err(|_| allocation_error(span))?;
        let location = data.statement.take().expect("for-of source location");
        let StatementKind::ForOf {
            binding, iterable, ..
        } = &active_statement(&data.body, Some(&location)).kind
        else {
            unreachable!("for-of source path");
        };
        let environment = self.prepare_iteration_head_scope(binding, span)?;
        data.frames
            .push(AsyncStatementFrame::ForOfHead(AsyncForOfHead {
                location,
                environment,
            }));
        if expression::has_supported_await(iterable) {
            self.start_async_expression(data, ExprRoot::ForOfSource, AsyncAwaitAction::ForOfSource)
        } else {
            let value = self.expression(iterable)?;
            self.finish_async_for_of_source(data, value)
        }
    }

    pub(super) fn finish_async_for_of_source(
        &mut self,
        data: &mut AsyncExecutionData,
        value: Value,
    ) -> Result<Step, Error> {
        let Some(AsyncStatementFrame::ForOfHead(head)) = data.frames.pop() else {
            unreachable!("for-of head completion");
        };
        let span = active_statement(&data.body, Some(&head.location)).span;
        if let Some(environment) = head.environment {
            let scope = self.scopes.pop().expect("iteration RHS environment");
            debug_assert_eq!(scope, environment);
        }
        let method = self
            .get_method(&value, &WellKnownSymbol::Iterator.symbol(), span)?
            .ok_or_else(|| {
                Self::exception(
                    ExceptionKind::TypeError,
                    span,
                    "for-of value is not iterable",
                )
            })?;
        let iterator = self.get_iterator_from_method(value, method, span)?;
        // Popping the head reserved this slot before iterator acquisition.
        data.frames
            .push(AsyncStatementFrame::ForOf(AsyncForOfCursor {
                location: head.location,
                iterator,
                value: Value::Undefined,
            }));
        Ok(Step::Continue)
    }

    pub(super) fn async_for_of_step(
        &mut self,
        data: &mut AsyncExecutionData,
    ) -> Result<Step, Error> {
        let (next, body, span) = {
            let Some(AsyncStatementFrame::ForOf(cursor)) = data.frames.last_mut() else {
                unreachable!("for-of iterator frame");
            };
            let span = active_statement(&data.body, Some(&cursor.location)).span;
            self.tick(span)?;
            // IteratorStepValue marks the record done on every step failure,
            // preventing IteratorClose for next/done/value errors.
            let next = self.iterator_step_value(&mut cursor.iterator, span)?;
            let body = if next.is_some() {
                Some(child_location(
                    &cursor.location,
                    StatementChild::ForOfBody,
                    span,
                )?)
            } else {
                None
            };
            (next, body, span)
        };
        let Some(value) = next else {
            let Some(AsyncStatementFrame::ForOf(cursor)) = data.frames.pop() else {
                unreachable!("exhausted for-of frame");
            };
            return Ok(Step::Complete(Completion::normal(Some(cursor.value))));
        };
        let Some(AsyncStatementFrame::ForOf(cursor)) = data.frames.last() else {
            unreachable!("for-of binding frame");
        };
        let StatementKind::ForOf { binding, .. } =
            &active_statement(&data.body, Some(&cursor.location)).kind
        else {
            unreachable!("for-of binding source");
        };
        if matches!(binding, ForBinding::Lexical { .. } | ForBinding::Using(_)) {
            data.frames
                .try_reserve(1)
                .map_err(|_| allocation_error(span))?;
        }
        let scope = self.prepare_iteration_scope(binding, span)?;
        if let Some((environment, disposable)) = &scope {
            data.frames.push(AsyncStatementFrame::Scope {
                environment: environment.clone(),
                span,
                disposable: *disposable,
            });
        }
        self.initialize_iteration_binding(
            binding,
            value,
            scope.as_ref().map(|(environment, _)| environment),
        )?;
        data.next = body;
        Ok(Step::Continue)
    }

    pub(super) fn advance_async_for_of(
        &mut self,
        data: &mut AsyncExecutionData,
        result: Result<Completion, Error>,
    ) -> FrameStep {
        let Some(AsyncStatementFrame::ForOf(mut cursor)) = data.frames.pop() else {
            unreachable!("for-of completion frame");
        };
        let span = active_statement(&data.body, Some(&cursor.location)).span;
        let mut completion = match result {
            Err(error) => {
                return FrameStep::Complete(Err(if cursor.iterator.is_done() {
                    error
                } else {
                    self.iterator_close_error(&cursor.iterator, error, span)
                }));
            }
            Ok(completion) => completion
                .update_empty(Some(std::mem::replace(&mut cursor.value, Value::Undefined))),
        };
        if completion.kind == CompletionKind::Normal
            || (completion.kind == CompletionKind::Continue && completion.target.is_none())
        {
            cursor.value = completion.value.take().expect("nonempty for-of value");
            data.frames.push(AsyncStatementFrame::ForOf(cursor));
            FrameStep::Continue
        } else {
            if let Err(error) = self.iterator_close(&cursor.iterator, span) {
                return FrameStep::Complete(Err(error));
            }
            if completion.kind == CompletionKind::Break && completion.target.is_none() {
                completion.kind = CompletionKind::Normal;
            }
            FrameStep::Complete(Ok(completion))
        }
    }
}
