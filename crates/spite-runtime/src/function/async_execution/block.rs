//! Nested source BlockStatement cursors and lexical disposal (14.2.3).

use super::{AsyncExecutionData, Step, active_statement};
use crate::{
    Completion, Error, Realm,
    object::async_execution::{AsyncBlockCursor, StatementChild, StatementLocation},
};
use spite_core::Span;
use spite_parser::ast::{ArrowBody, StatementKind};
use std::collections::BTreeMap;

fn child_location(
    parent: &StatementLocation,
    index: usize,
    span: Span,
) -> Result<StatementLocation, Error> {
    let mut children = Vec::new();
    children
        .try_reserve(parent.children.len() + 1)
        .map_err(|_| allocation_error(span))?;
    children.extend_from_slice(&parent.children);
    children.push(StatementChild::Block(index));
    Ok(StatementLocation {
        root: parent.root,
        children,
    })
}

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
        } else if let Some(block) = data.blocks.last() {
            let StatementKind::Block(body) =
                &active_statement(&data.body, Some(&block.location)).kind
            else {
                unreachable!("block cursor path");
            };
            if block.index == body.len() {
                let block = data.blocks.pop().expect("active block");
                self.finish_async_block(block, Ok(Completion::normal(None)))?;
                return Ok(Step::Continue);
            }
            let location = child_location(&block.location, block.index, block.span)?;
            data.blocks.last_mut().expect("active block").index += 1;
            location
        } else {
            let ArrowBody::Block(body) = &data.body else {
                unreachable!("block body");
            };
            if data.index == body.statements().len() {
                return Ok(Step::Complete(Completion::normal(None)));
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
        data.blocks
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
        data.blocks.push(AsyncBlockCursor {
            location,
            index: 0,
            environment,
            span,
        });
        self.instantiate(body.iter(), false, true)?;
        Ok(Step::Continue)
    }

    pub(super) fn finish_async_block(
        &mut self,
        block: AsyncBlockCursor,
        completion: Result<Completion, Error>,
    ) -> Result<Completion, Error> {
        let result = self.dispose_environment_resources(&block.environment, completion, block.span);
        let environment = self.scopes.pop().expect("active block environment");
        debug_assert_eq!(environment, block.environment);
        result
    }
}
