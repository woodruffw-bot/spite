//! Synchronous ForOf iteration with suspended source expressions and bodies.

use super::{AsyncExecutionData, FrameStep, Step, active_statement, child_location};
use crate::{
    Completion, CompletionKind, Error, Realm, Value,
    object::async_execution::{AsyncStatementFrame, StatementChild},
};
use spite_parser::ast::StatementKind;

impl Realm {
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
        self.initialize_async_iteration_binding(&mut data.frames, binding, value, span)?;
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
        if super::labelled::loop_continues(data, &completion) {
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
