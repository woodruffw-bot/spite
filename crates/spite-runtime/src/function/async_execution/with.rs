//! Suspended WithStatement object evaluation and scope restoration (14.11.2).

use super::{AsyncExecutionData, Step, active_statement, child_location};
use crate::{
    Error, Realm, Value,
    object::async_execution::{AsyncStatementFrame, StatementChild},
};
use spite_parser::ast::StatementKind;

impl Realm {
    pub(super) fn enter_async_with(
        &mut self,
        data: &mut AsyncExecutionData,
        value: Value,
    ) -> Result<Step, Error> {
        let statement = active_statement(&data.body, data.statement.as_ref());
        let StatementKind::With { object, .. } = &statement.kind else {
            unreachable!("with statement entry")
        };
        data.frames.try_reserve(1).map_err(|_| Error::Limit {
            span: statement.span,
            message: "async with continuation allocation failed".into(),
        })?;
        let location = data.statement.take().expect("with source location");
        let body = child_location(&location, StatementChild::WithBody, statement.span)?;
        let environment = self.enter_with_environment(value, object.span, statement.span)?;
        data.frames.push(AsyncStatementFrame::With(environment));
        data.next = Some(body);
        Ok(Step::Continue)
    }
}
