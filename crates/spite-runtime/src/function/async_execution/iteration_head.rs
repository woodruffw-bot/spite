//! Shared suspended ForIn/Of head evaluation and iteration binding setup (14.7.5.6–7).

use super::{AsyncAwaitAction, AsyncExecutionData, Step, active_statement, expression};
use crate::{
    Completion, Error, ExceptionKind, Realm, Value,
    object::{
        async_execution::{
            AsyncForInCursor, AsyncForOfCursor, AsyncIterationHead, AsyncStatementFrame,
        },
        async_expression::ExprRoot,
    },
};
use spite_core::{Span, WellKnownSymbol};
use spite_parser::ast::{ForBinding, StatementKind};

impl Realm {
    pub(super) fn enter_async_iteration(
        &mut self,
        data: &mut AsyncExecutionData,
        span: Span,
    ) -> Result<Step, Error> {
        data.frames.try_reserve(1).map_err(|_| Error::Limit {
            span,
            message: "async iteration continuation allocation failed".into(),
        })?;
        let location = data.statement.take().expect("iteration source location");
        let (binding, source) = match &active_statement(&data.body, Some(&location)).kind {
            StatementKind::ForOf {
                binding, iterable, ..
            } => (binding, iterable),
            StatementKind::ForIn {
                binding, object, ..
            } => (binding, object),
            _ => unreachable!("iteration source path"),
        };
        let environment = self.prepare_iteration_head_scope(binding, span)?;
        data.frames
            .push(AsyncStatementFrame::IterationHead(AsyncIterationHead {
                location,
                environment,
            }));
        if expression::has_supported_await(source) {
            self.start_async_expression(
                data,
                ExprRoot::IterationSource,
                AsyncAwaitAction::IterationSource,
            )
        } else {
            let value = self.expression(source)?;
            self.finish_async_iteration_source(data, value)
        }
    }

    pub(super) fn finish_async_iteration_source(
        &mut self,
        data: &mut AsyncExecutionData,
        value: Value,
    ) -> Result<Step, Error> {
        let Some(AsyncStatementFrame::IterationHead(head)) = data.frames.pop() else {
            unreachable!("iteration head completion")
        };
        let statement = active_statement(&data.body, Some(&head.location));
        let span = statement.span;
        if let Some(environment) = head.environment {
            let scope = self.scopes.pop().expect("iteration RHS environment");
            debug_assert_eq!(scope, environment);
        }
        let frame = if matches!(statement.kind, StatementKind::ForIn { .. }) {
            let Some(iterator) = self.get_for_in_iterator(value, span)? else {
                return Ok(Step::Complete(Completion::normal(Some(Value::Undefined))));
            };
            AsyncStatementFrame::ForIn(AsyncForInCursor {
                location: head.location,
                iterator,
                value: Value::Undefined,
            })
        } else {
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
            AsyncStatementFrame::ForOf(AsyncForOfCursor {
                location: head.location,
                iterator,
                value: Value::Undefined,
            })
        };
        // Popping the head reserved this slot before acquiring either iterator.
        data.frames.push(frame);
        Ok(Step::Continue)
    }

    pub(super) fn initialize_async_iteration_binding(
        &mut self,
        frames: &mut Vec<AsyncStatementFrame>,
        binding: &ForBinding,
        value: Value,
        span: Span,
    ) -> Result<(), Error> {
        if matches!(binding, ForBinding::Lexical { .. } | ForBinding::Using(_)) {
            frames.try_reserve(1).map_err(|_| Error::Limit {
                span,
                message: "async iteration binding allocation failed".into(),
            })?;
        }
        let scope = self.prepare_iteration_scope(binding, span)?;
        if let Some((environment, disposable)) = &scope {
            frames.push(AsyncStatementFrame::Scope {
                environment: environment.clone(),
                span,
                disposable: *disposable,
            });
        }
        self.initialize_iteration_binding(
            binding,
            value,
            scope.as_ref().map(|(environment, _)| environment),
        )
    }
}
