//! Suspended EnumerateObjectProperties with live descriptors and prototypes (14.7.5.9–10).

use super::{AsyncExecutionData, FrameStep, Step, active_statement, child_location};
use crate::{
    Completion, Error, Realm, Value,
    object::async_execution::{AsyncStatementFrame, StatementChild},
};
use spite_parser::ast::StatementKind;

impl Realm {
    pub(super) fn async_for_in_step(
        &mut self,
        data: &mut AsyncExecutionData,
    ) -> Result<Step, Error> {
        let (next, body, span) = {
            let Some(AsyncStatementFrame::ForIn(cursor)) = data.frames.last_mut() else {
                unreachable!("for-in enumeration frame")
            };
            let span = active_statement(&data.body, Some(&cursor.location)).span;
            let next = self.for_in_next(&mut cursor.iterator, span)?;
            let body = if next.is_some() {
                Some(child_location(
                    &cursor.location,
                    StatementChild::ForInBody,
                    span,
                )?)
            } else {
                None
            };
            (next, body, span)
        };
        let Some(key) = next else {
            let Some(AsyncStatementFrame::ForIn(cursor)) = data.frames.pop() else {
                unreachable!("exhausted for-in frame")
            };
            return Ok(Step::Complete(Completion::normal(Some(cursor.value))));
        };
        let Some(AsyncStatementFrame::ForIn(cursor)) = data.frames.last() else {
            unreachable!("for-in binding frame")
        };
        let StatementKind::ForIn { binding, .. } =
            &active_statement(&data.body, Some(&cursor.location)).kind
        else {
            unreachable!("for-in binding source")
        };
        self.initialize_async_iteration_binding(
            &mut data.frames,
            binding,
            Value::String(key),
            span,
        )?;
        data.next = body;
        Ok(Step::Continue)
    }

    pub(super) fn advance_async_for_in(
        &mut self,
        data: &mut AsyncExecutionData,
        result: Result<Completion, Error>,
    ) -> FrameStep {
        let Some(AsyncStatementFrame::ForIn(mut cursor)) = data.frames.pop() else {
            unreachable!("for-in completion frame")
        };
        let mut completion = match result {
            Err(error) => return FrameStep::Complete(Err(error)),
            Ok(completion) => completion
                .update_empty(Some(std::mem::replace(&mut cursor.value, Value::Undefined))),
        };
        if super::labelled::loop_continues(data, &completion) {
            cursor.value = completion.value.take().expect("nonempty for-in value");
            data.frames.push(AsyncStatementFrame::ForIn(cursor));
            FrameStep::Continue
        } else {
            // This internal iterator has no observable return hook.
            FrameStep::Complete(Ok(completion.consume_unlabelled_break()))
        }
    }
}
