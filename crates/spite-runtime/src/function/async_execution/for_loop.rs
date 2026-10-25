//! Suspended ForStatement and CreatePerIterationEnvironment (14.7.4.2–4).

use super::{
    AsyncAwaitAction, AsyncExecutionData, FrameStep, Step, active_statement, child_location,
    expression,
};
use crate::{
    Completion, Error, Realm, Value,
    object::{
        async_execution::{AsyncForCursor, AsyncForPhase, AsyncStatementFrame, StatementChild},
        async_expression::ExprRoot,
    },
};
use spite_core::Span;
use spite_parser::ast::{Expr, ForInitializer, StatementKind};

pub(super) fn header_expression(data: &AsyncExecutionData) -> &Expr {
    let Some(AsyncStatementFrame::For(cursor)) = data.frames.last() else {
        unreachable!("for header frame")
    };
    let StatementKind::For {
        initializer,
        test,
        update,
        ..
    } = &active_statement(&data.body, Some(&cursor.location)).kind
    else {
        unreachable!("for source")
    };
    match cursor.phase {
        AsyncForPhase::Initialize => {
            let Some(ForInitializer::Expression(expression)) = initializer else {
                unreachable!("for expression initializer")
            };
            expression
        }
        AsyncForPhase::Test => test.as_ref().expect("for condition"),
        AsyncForPhase::Update => update.as_ref().expect("for update"),
        AsyncForPhase::Body => unreachable!("for body is a statement"),
    }
}

impl Realm {
    pub(super) fn enter_async_for(
        &mut self,
        data: &mut AsyncExecutionData,
        span: Span,
    ) -> Result<Step, Error> {
        data.frames.try_reserve(1).map_err(|_| Error::Limit {
            span,
            message: "async for continuation allocation failed".into(),
        })?;
        let location = data.statement.take().expect("for source location");
        let StatementKind::For { initializer, .. } =
            &active_statement(&data.body, Some(&location)).kind
        else {
            unreachable!("for source")
        };
        let lexical = match initializer {
            Some(ForInitializer::Lexical { mutable, bindings }) => {
                Some((bindings, *mutable, false))
            }
            Some(ForInitializer::Using(bindings)) => Some((bindings, false, true)),
            _ => None,
        };
        let (environment, disposable) = if let Some((bindings, mutable, disposable)) = lexical {
            let bindings =
                self.pattern_bindings(bindings.iter().map(|binding| &binding.pattern), mutable)?;
            self.push_scope(bindings, span)?;
            (
                Some(self.scopes.last().expect("for header scope").clone()),
                disposable,
            )
        } else {
            (None, false)
        };
        // Record the scope before initializing any binding.
        data.frames.push(AsyncStatementFrame::For(AsyncForCursor {
            location,
            phase: AsyncForPhase::Initialize,
            environment,
            disposable,
            value: Value::Undefined,
        }));
        Ok(Step::Continue)
    }

    pub(super) fn async_for_step(&mut self, data: &mut AsyncExecutionData) -> Result<Step, Error> {
        let Some(AsyncStatementFrame::For(cursor)) = data.frames.last() else {
            unreachable!("for frame")
        };
        let StatementKind::For {
            initializer,
            test,
            update,
            ..
        } = &active_statement(&data.body, Some(&cursor.location)).kind
        else {
            unreachable!("for source")
        };
        match cursor.phase {
            AsyncForPhase::Initialize => match initializer {
                Some(
                    ForInitializer::Lexical { .. }
                    | ForInitializer::Using(_)
                    | ForInitializer::Var(_),
                ) => return self.async_binding_step(data, 0, None),
                None => return Ok(Step::Complete(Completion::normal(None))),
                Some(ForInitializer::Expression(_)) => (),
            },
            AsyncForPhase::Test if test.is_none() => {
                return self.finish_async_for_header(data, Value::Boolean(true));
            }
            AsyncForPhase::Update if update.is_none() => {
                return self.finish_async_for_header(data, Value::Undefined);
            }
            AsyncForPhase::Test | AsyncForPhase::Update => (),
            AsyncForPhase::Body => unreachable!("for body must complete first"),
        }
        let expression = header_expression(data);
        if expression::has_supported_await(expression) {
            self.start_async_expression(data, ExprRoot::ForHeader, AsyncAwaitAction::ForHeader)
        } else {
            let value = self.expression(expression)?;
            self.finish_async_for_header(data, value)
        }
    }

    pub(super) fn finish_async_for_header(
        &mut self,
        data: &mut AsyncExecutionData,
        value: Value,
    ) -> Result<Step, Error> {
        let Some(AsyncStatementFrame::For(cursor)) = data.frames.last_mut() else {
            unreachable!("for header frame")
        };
        match cursor.phase {
            AsyncForPhase::Initialize => Ok(Step::Complete(Completion::normal(None))),
            AsyncForPhase::Test => {
                if value.to_boolean() {
                    let span = active_statement(&data.body, Some(&cursor.location)).span;
                    data.next = Some(child_location(
                        &cursor.location,
                        StatementChild::ForBody,
                        span,
                    )?);
                    cursor.phase = AsyncForPhase::Body;
                    Ok(Step::Continue)
                } else {
                    let Some(AsyncStatementFrame::For(mut cursor)) = data.frames.pop() else {
                        unreachable!("for exit")
                    };
                    let value = std::mem::replace(&mut cursor.value, Value::Undefined);
                    self.finish_async_for_scope(cursor, Ok(Completion::normal(Some(value))), data)
                        .map(Step::Complete)
                }
            }
            AsyncForPhase::Update => {
                cursor.phase = AsyncForPhase::Test;
                Ok(Step::Continue)
            }
            AsyncForPhase::Body => unreachable!("for body is a statement"),
        }
    }

    fn copy_async_for_environment(
        &mut self,
        data: &AsyncExecutionData,
        cursor: &mut AsyncForCursor,
    ) -> Result<(), Error> {
        let StatementKind::For { initializer, .. } =
            &active_statement(&data.body, Some(&cursor.location)).kind
        else {
            unreachable!("for source")
        };
        if let Some(ForInitializer::Lexical {
            mutable: true,
            bindings,
        }) = initializer
        {
            self.create_per_iteration_environment(bindings)?;
            // The environment is replaced in place. Keep its traced owner and
            // exit check synchronized with the scope actually being executed.
            cursor.environment = Some(
                self.scopes
                    .last()
                    .expect("per-iteration for environment")
                    .clone(),
            );
        }
        Ok(())
    }

    fn finish_async_for_scope(
        &mut self,
        cursor: AsyncForCursor,
        mut result: Result<Completion, Error>,
        data: &AsyncExecutionData,
    ) -> Result<Completion, Error> {
        if let Some(environment) = cursor.environment {
            if cursor.disposable {
                let span = active_statement(&data.body, Some(&cursor.location)).span;
                result = self.dispose_environment_resources(&environment, result, span);
            }
            let scope = self.scopes.pop().expect("for header scope");
            debug_assert_eq!(scope, environment);
        }
        result
    }

    pub(super) fn advance_async_for(
        &mut self,
        data: &mut AsyncExecutionData,
        result: Result<Completion, Error>,
    ) -> FrameStep {
        let Some(AsyncStatementFrame::For(mut cursor)) = data.frames.pop() else {
            unreachable!("for completion frame")
        };
        let mut completion = match result {
            Err(error) => {
                return FrameStep::Complete(self.finish_async_for_scope(cursor, Err(error), data));
            }
            Ok(completion) => completion,
        };
        match cursor.phase {
            AsyncForPhase::Initialize => cursor.phase = AsyncForPhase::Test,
            AsyncForPhase::Body => {
                completion = completion
                    .update_empty(Some(std::mem::replace(&mut cursor.value, Value::Undefined)));
                if !super::labelled::loop_continues(data, &completion) {
                    return FrameStep::Complete(self.finish_async_for_scope(
                        cursor,
                        Ok(completion.consume_unlabelled_break()),
                        data,
                    ));
                }
                cursor.value = completion.value.take().expect("nonempty for value");
                cursor.phase = AsyncForPhase::Update;
            }
            AsyncForPhase::Test | AsyncForPhase::Update => {
                unreachable!("for header completion is direct")
            }
        }
        // The first copy precedes the first test. Later copies precede update,
        // after body finalizers and disposal have finished (14.7.4.3).
        if let Err(error) = self.copy_async_for_environment(data, &mut cursor) {
            return FrameStep::Complete(self.finish_async_for_scope(cursor, Err(error), data));
        }
        data.frames.push(AsyncStatementFrame::For(cursor));
        FrameStep::Continue
    }
}
