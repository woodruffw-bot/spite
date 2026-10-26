//! CaseBlock selection, shared lexical scope and fall-through (14.12.2–4).

use super::{
    AsyncAwaitAction, AsyncExecutionData, FrameStep, Step, active_statement, child_location,
    expression,
};
use crate::{
    Completion, CompletionKind, Error, Realm, Value,
    object::{
        async_execution::{AsyncStatementFrame, AsyncSwitchCursor, StatementChild},
        async_expression::ExprRoot,
    },
};
use spite_parser::ast::StatementKind;
use std::collections::BTreeMap;

impl Realm {
    pub(super) fn enter_async_switch(
        &mut self,
        data: &mut AsyncExecutionData,
        input: Value,
    ) -> Result<Step, Error> {
        let location = data.statement.take().expect("switch source location");
        let statement = active_statement(&data.body, Some(&location));
        let StatementKind::Switch { clauses, .. } = &statement.kind else {
            unreachable!("switch entry")
        };
        data.frames.try_reserve(1).map_err(|_| Error::Limit {
            span: statement.span,
            message: "async switch continuation allocation failed".into(),
        })?;
        self.push_scope(BTreeMap::new(), statement.span)?;
        let environment = self.scopes.last().expect("switch environment").clone();
        data.frames
            .push(AsyncStatementFrame::Switch(AsyncSwitchCursor {
                location,
                environment,
                clause: 0,
                statement: 0,
                default: None,
                input: Some(input),
                value: Value::Undefined,
            }));
        self.instantiate(
            clauses.iter().flat_map(|clause| clause.statements.iter()),
            false,
            true,
        )?;
        Ok(Step::Continue)
    }

    pub(super) fn async_switch_step(
        &mut self,
        data: &mut AsyncExecutionData,
    ) -> Result<Step, Error> {
        loop {
            let Some(AsyncStatementFrame::Switch(cursor)) = data.frames.last() else {
                unreachable!("switch cursor")
            };
            let statement = active_statement(&data.body, Some(&cursor.location));
            let StatementKind::Switch { clauses, .. } = &statement.kind else {
                unreachable!("switch source")
            };
            if cursor.input.is_some() {
                if cursor.clause == clauses.len() {
                    if let Some(default) = cursor.default {
                        let Some(AsyncStatementFrame::Switch(cursor)) = data.frames.last_mut()
                        else {
                            unreachable!("switch cursor")
                        };
                        cursor.clause = default;
                        cursor.input = None;
                        continue;
                    }
                    return self
                        .finish_async_switch(data, Ok(Completion::normal(None)))
                        .map(Step::Complete);
                }
                let clause = &clauses[cursor.clause];
                self.tick(clause.span)?;
                if let Some(test) = &clause.test {
                    if expression::has_supported_await(test) {
                        return self.start_async_expression(
                            data,
                            ExprRoot::SwitchSelector,
                            AsyncAwaitAction::SwitchSelector,
                        );
                    }
                    let value = self.expression(test)?;
                    return self.finish_async_switch_selector(data, value);
                }
                let Some(AsyncStatementFrame::Switch(cursor)) = data.frames.last_mut() else {
                    unreachable!("switch cursor")
                };
                cursor.default = Some(cursor.clause);
                cursor.clause += 1;
            } else {
                if cursor.clause == clauses.len() {
                    return self
                        .finish_async_switch(data, Ok(Completion::normal(None)))
                        .map(Step::Complete);
                }
                let clause = &clauses[cursor.clause];
                if cursor.statement == 0 {
                    self.tick(clause.span)?;
                }
                if cursor.statement == clause.statements.len() {
                    let Some(AsyncStatementFrame::Switch(cursor)) = data.frames.last_mut() else {
                        unreachable!("switch cursor")
                    };
                    cursor.clause += 1;
                    cursor.statement = 0;
                    continue;
                }
                let location = child_location(
                    &cursor.location,
                    StatementChild::SwitchBody {
                        clause: cursor.clause,
                        statement: cursor.statement,
                    },
                    statement.span,
                )?;
                let Some(AsyncStatementFrame::Switch(cursor)) = data.frames.last_mut() else {
                    unreachable!("switch cursor")
                };
                cursor.statement += 1;
                data.statement = Some(location);
                return self.async_source_statement(data);
            }
        }
    }

    pub(super) fn finish_async_switch_selector(
        &mut self,
        data: &mut AsyncExecutionData,
        value: Value,
    ) -> Result<Step, Error> {
        let Some(AsyncStatementFrame::Switch(cursor)) = data.frames.last_mut() else {
            unreachable!("switch selector cursor")
        };
        let StatementKind::Switch { clauses, .. } =
            &active_statement(&data.body, Some(&cursor.location)).kind
        else {
            unreachable!("switch selector source")
        };
        let test = clauses[cursor.clause].test.as_ref().expect("case selector");
        if self.strictly_equal(
            cursor.input.as_ref().expect("switch input"),
            &value,
            test.span,
        )? {
            cursor.input = None;
        } else {
            cursor.clause += 1;
        }
        Ok(Step::Continue)
    }

    pub(super) fn advance_async_switch(
        &mut self,
        data: &mut AsyncExecutionData,
        mut result: Result<Completion, Error>,
    ) -> FrameStep {
        if let Ok(completion) = &mut result
            && completion.kind == CompletionKind::Normal
        {
            let Some(AsyncStatementFrame::Switch(cursor)) = data.frames.last_mut() else {
                unreachable!("switch completion cursor")
            };
            if let Some(value) = completion.value.take() {
                cursor.value = value;
            }
            return FrameStep::Continue;
        }
        FrameStep::Complete(self.finish_async_switch(data, result))
    }

    fn finish_async_switch(
        &mut self,
        data: &mut AsyncExecutionData,
        result: Result<Completion, Error>,
    ) -> Result<Completion, Error> {
        let Some(AsyncStatementFrame::Switch(cursor)) = data.frames.pop() else {
            unreachable!("switch completion cursor")
        };
        let result = result.map(|completion| completion.update_empty(Some(cursor.value)));
        let environment = self.scopes.pop().expect("active switch environment");
        debug_assert_eq!(environment, cursor.environment);
        result.map(Completion::consume_unlabelled_break)
    }
}
