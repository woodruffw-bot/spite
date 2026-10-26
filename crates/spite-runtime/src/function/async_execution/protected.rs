//! Try/Catch/Finally completion transitions across source suspension (14.15.3).

use super::{AsyncExecutionData, FrameStep, Step, active_statement, child_location};
use crate::{
    Completion, CompletionKind, Error, Realm, Value,
    object::async_execution::{AsyncStatementFrame, AsyncTryCursor, AsyncTryPhase, StatementChild},
};
use spite_core::Span;
use spite_parser::ast::StatementKind;

fn allocation_error(span: Span) -> Error {
    Error::Limit {
        span,
        message: "async try continuation allocation failed".into(),
    }
}

impl Realm {
    pub(super) fn enter_async_try(
        &mut self,
        data: &mut AsyncExecutionData,
        span: Span,
    ) -> Result<Step, Error> {
        data.frames
            .try_reserve(1)
            .map_err(|_| allocation_error(span))?;
        let location = data.statement.take().expect("try source location");
        let body = child_location(&location, StatementChild::TryBody, span)?;
        data.frames.push(AsyncStatementFrame::Try(AsyncTryCursor {
            location,
            phase: AsyncTryPhase::Body,
            pending: None,
        }));
        debug_assert!(data.next.is_none());
        data.next = Some(body);
        Ok(Step::Continue)
    }

    pub(super) fn advance_async_try(
        &mut self,
        data: &mut AsyncExecutionData,
        result: Result<Completion, Error>,
    ) -> FrameStep {
        let Some(AsyncStatementFrame::Try(mut cursor)) = data.frames.pop() else {
            unreachable!("try completion frame");
        };
        let statement = active_statement(&data.body, Some(&cursor.location));
        let StatementKind::Try {
            handler, finalizer, ..
        } = &statement.kind
        else {
            unreachable!("try source path");
        };
        let span = statement.span;
        if matches!(cursor.phase, AsyncTryPhase::Body)
            && result.as_ref().is_err_and(Error::is_language_exception)
            && let Some(handler) = handler
        {
            cursor.phase = AsyncTryPhase::Catch;
            let body = match child_location(&cursor.location, StatementChild::CatchBody, span) {
                Ok(body) => body,
                Err(error) => return FrameStep::Complete(Err(error)),
            };
            data.frames.push(AsyncStatementFrame::Try(cursor));
            let suspended_binding = handler
                .parameter
                .as_ref()
                .is_some_and(super::binding_array::has_array_binding_await);
            if handler.parameter.is_some()
                && data
                    .frames
                    .try_reserve(if suspended_binding { 2 } else { 1 })
                    .is_err()
            {
                return FrameStep::Complete(Err(allocation_error(handler.span)));
            }
            let Err(error) = result else {
                unreachable!("caught throw");
            };
            match self.prepare_catch_clause(handler, error) {
                Ok(Some((environment, value))) => {
                    data.frames.push(AsyncStatementFrame::Scope {
                        environment: environment.clone(),
                        span: handler.span,
                        disposable: false,
                    });
                    if suspended_binding {
                        // Initialize on the next execution step so acquisition errors
                        // enter the ordinary binding cleanup and completion path.
                        data.frames
                            .push(AsyncStatementFrame::CatchBinding { environment, value });
                    } else if let Err(error) = self.initialize_pattern(
                        handler.parameter.as_ref().expect("catch parameter"),
                        value,
                        &environment,
                    ) {
                        return FrameStep::Complete(Err(error));
                    }
                }
                Ok(None) => (),
                Err(error) => return FrameStep::Complete(Err(error)),
            }
            data.next = Some(body);
            return FrameStep::Continue;
        }
        if matches!(cursor.phase, AsyncTryPhase::Finally) {
            let protected = cursor.pending.take().expect("protected completion");
            let result = match result {
                Ok(completion) if completion.kind == CompletionKind::Normal => protected,
                result => result,
            };
            return FrameStep::Complete(
                result.map(|completion| completion.update_empty(Some(Value::Undefined))),
            );
        }
        if finalizer.is_some() {
            let body = match child_location(&cursor.location, StatementChild::FinallyBody, span) {
                Ok(body) => body,
                Err(error) => return FrameStep::Complete(Err(error)),
            };
            cursor.phase = AsyncTryPhase::Finally;
            cursor.pending = Some(result);
            data.frames.push(AsyncStatementFrame::Try(cursor));
            data.next = Some(body);
            FrameStep::Continue
        } else {
            FrameStep::Complete(
                result.map(|completion| completion.update_empty(Some(Value::Undefined))),
            )
        }
    }
}
