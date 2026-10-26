//! Nested source BlockStatement cursors and lexical disposal (14.2.3).

use super::{AsyncExecutionData, Step, active_statement, child_location};
use crate::{
    Completion, Error, Realm,
    object::async_execution::{
        AsyncBlockCursor, AsyncStatementFrame, StatementChild, StatementLocation,
    },
};
use spite_core::Span;
use spite_parser::ast::{ArrowBody, StatementKind};
use std::collections::BTreeMap;

fn allocation_error(span: Span) -> Error {
    Error::Limit {
        span,
        message: "async block continuation allocation failed".into(),
    }
}

impl Realm {
    pub(super) fn async_block_step(
        &mut self,
        data: &mut AsyncExecutionData,
    ) -> Result<Step, Error> {
        let location = if let Some(next) = data.next.take() {
            next
        } else if matches!(data.frames.last(), Some(AsyncStatementFrame::Switch(_))) {
            return self.async_switch_step(data);
        } else if matches!(data.frames.last(), Some(AsyncStatementFrame::ForAwait(_))) {
            return self.async_for_await_step(data);
        } else if matches!(data.frames.last(), Some(AsyncStatementFrame::ForOf(_))) {
            return self.async_for_of_step(data);
        } else if matches!(data.frames.last(), Some(AsyncStatementFrame::ForIn(_))) {
            return self.async_for_in_step(data);
        } else if matches!(data.frames.last(), Some(AsyncStatementFrame::For(_))) {
            return self.async_for_step(data);
        } else if matches!(data.frames.last(), Some(AsyncStatementFrame::Loop(_))) {
            return self.async_loop_test(data);
        } else if let Some(frame) = data.frames.last() {
            let AsyncStatementFrame::Block(block) = frame else {
                unreachable!("a selected conditional must complete before advancing its parent");
            };
            let StatementKind::Block(body) =
                &active_statement(&data.body, Some(&block.location)).kind
            else {
                unreachable!("block cursor path");
            };
            if block.index == body.len() {
                let Some(AsyncStatementFrame::Block(block)) = data.frames.pop() else {
                    unreachable!("active block");
                };
                let completion = self.finish_async_block(block, Ok(Completion::normal(None)))?;
                return Ok(Step::Complete(completion));
            }
            let location = child_location(
                &block.location,
                StatementChild::Block(block.index),
                block.span,
            )?;
            let Some(AsyncStatementFrame::Block(block)) = data.frames.last_mut() else {
                unreachable!("active block");
            };
            block.index += 1;
            location
        } else {
            let ArrowBody::Block(body) = &data.body else {
                unreachable!("block body");
            };
            if data.index == body.statements().len() {
                return Ok(Step::Finish(Completion::normal(data.normal.take())));
            }
            let location = StatementLocation {
                root: data.index,
                children: Vec::new(),
            };
            data.index += 1;
            location
        };
        data.statement = Some(location);
        self.async_source_statement(data)
    }

    pub(super) fn enter_async_block(
        &mut self,
        data: &mut AsyncExecutionData,
        span: Span,
    ) -> Result<Step, Error> {
        data.frames
            .try_reserve(1)
            .map_err(|_| allocation_error(span))?;
        self.push_scope(BTreeMap::new(), span)?;
        let environment = self.scopes.last().expect("block environment").clone();
        let location = data.statement.take().expect("block source location");
        let StatementKind::Block(body) = &active_statement(&data.body, Some(&location)).kind else {
            unreachable!("block entry");
        };
        // Record the scope before instantiation so every abrupt completion can
        // unwind it through the same lexical disposal path.
        data.frames
            .push(AsyncStatementFrame::Block(AsyncBlockCursor {
                location,
                index: 0,
                environment,
                span,
                normal: None,
            }));
        self.instantiate(body.iter(), false, true)?;
        Ok(Step::Continue)
    }

    pub(super) fn finish_async_block(
        &mut self,
        mut block: AsyncBlockCursor,
        completion: Result<Completion, Error>,
    ) -> Result<Completion, Error> {
        let normal = block.normal.take();
        let completion = completion.map(|completion| completion.update_empty(normal));
        let result = self.dispose_environment_resources(&block.environment, completion, block.span);
        let environment = self.scopes.pop().expect("active block environment");
        debug_assert_eq!(environment, block.environment);
        result
    }
}
