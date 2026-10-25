//! Awaited while/do-while conditions, body completions and unlabelled control.

use super::{
    AsyncAwaitAction, AsyncExecutionData, FrameStep, Step, active_statement, child_location,
    expression,
};
use crate::{
    Completion, CompletionKind, Error, Realm, Value,
    object::{
        async_execution::{AsyncLoopCursor, AsyncLoopPhase, AsyncStatementFrame, StatementChild},
        async_expression::ExprRoot,
    },
};
use spite_core::Span;
use spite_parser::ast::StatementKind;

fn allocation_error(span: Span) -> Error {
    Error::Limit {
        span,
        message: "async loop continuation allocation failed".into(),
    }
}

impl Realm {
    pub(super) fn enter_async_loop(
        &mut self,
        data: &mut AsyncExecutionData,
        span: Span,
    ) -> Result<Step, Error> {
        data.frames
            .try_reserve(1)
            .map_err(|_| allocation_error(span))?;
        let location = data.statement.take().expect("loop source location");
        let body_first = matches!(
            active_statement(&data.body, Some(&location)).kind,
            StatementKind::DoWhile { .. }
        );
        let phase = if body_first {
            AsyncLoopPhase::Body
        } else {
            AsyncLoopPhase::Test
        };
        if body_first {
            data.next = Some(child_location(&location, StatementChild::LoopBody, span)?);
        }
        data.frames.push(AsyncStatementFrame::Loop(AsyncLoopCursor {
            location,
            phase,
            value: Value::Undefined,
        }));
        Ok(Step::Continue)
    }

    pub(super) fn async_loop_test(&mut self, data: &mut AsyncExecutionData) -> Result<Step, Error> {
        let Some(AsyncStatementFrame::Loop(cursor)) = data.frames.last() else {
            unreachable!("loop condition frame");
        };
        debug_assert!(matches!(cursor.phase, AsyncLoopPhase::Test));
        let test = match &active_statement(&data.body, Some(&cursor.location)).kind {
            StatementKind::While { test, .. } | StatementKind::DoWhile { test, .. } => test,
            _ => unreachable!("loop condition source"),
        };
        if expression::has_supported_await(test) {
            self.start_async_expression(data, ExprRoot::LoopTest, AsyncAwaitAction::LoopTest)
        } else {
            let value = self.expression(test)?;
            self.finish_async_loop_test(data, value)
        }
    }

    pub(super) fn finish_async_loop_test(
        &mut self,
        data: &mut AsyncExecutionData,
        value: Value,
    ) -> Result<Step, Error> {
        let Some(AsyncStatementFrame::Loop(mut cursor)) = data.frames.pop() else {
            unreachable!("loop condition frame");
        };
        if !value.to_boolean() {
            return Ok(Step::Complete(Completion::normal(Some(cursor.value))));
        }
        let span = active_statement(&data.body, Some(&cursor.location)).span;
        let body = child_location(&cursor.location, StatementChild::LoopBody, span)?;
        cursor.phase = AsyncLoopPhase::Body;
        data.frames.push(AsyncStatementFrame::Loop(cursor));
        data.next = Some(body);
        Ok(Step::Continue)
    }

    pub(super) fn advance_async_loop(
        &mut self,
        data: &mut AsyncExecutionData,
        result: Result<Completion, Error>,
    ) -> FrameStep {
        let Some(AsyncStatementFrame::Loop(mut cursor)) = data.frames.pop() else {
            unreachable!("loop body frame");
        };
        let mut completion = match result {
            Ok(completion) => completion
                .update_empty(Some(std::mem::replace(&mut cursor.value, Value::Undefined))),
            Err(error) => return FrameStep::Complete(Err(error)),
        };
        debug_assert!(matches!(cursor.phase, AsyncLoopPhase::Body));
        if completion.kind == CompletionKind::Normal
            || (completion.kind == CompletionKind::Continue && completion.target.is_none())
        {
            cursor.value = completion.value.take().expect("nonempty loop value");
            cursor.phase = AsyncLoopPhase::Test;
            data.frames.push(AsyncStatementFrame::Loop(cursor));
            FrameStep::Continue
        } else {
            if completion.kind == CompletionKind::Break && completion.target.is_none() {
                completion.kind = CompletionKind::Normal;
            }
            FrameStep::Complete(Ok(completion))
        }
    }
}
