//! IfStatement selection after ordinary or suspended condition evaluation (14.6.2).

use super::{AsyncExecutionData, Step, active_statement};
use crate::{Error, Realm, Value, object::async_execution::StatementChild};
use spite_parser::ast::StatementKind;

impl Realm {
    pub(super) fn finish_async_if(
        &mut self,
        data: &mut AsyncExecutionData,
        value: Value,
    ) -> Result<Step, Error> {
        let statement = active_statement(&data.body, data.statement.as_ref());
        let StatementKind::If { alternate, .. } = &statement.kind else {
            unreachable!("conditional statement continuation");
        };
        let span = statement.span;
        let child = if value.to_boolean() {
            Some(StatementChild::IfConsequent)
        } else {
            alternate.as_ref().map(|_| StatementChild::IfAlternate)
        };
        let mut location = data.statement.take().expect("conditional source location");
        if let Some(child) = child {
            location.children.try_reserve(1).map_err(|_| Error::Limit {
                span,
                message: "async conditional continuation allocation failed".into(),
            })?;
            location.children.push(child);
            debug_assert!(data.next.is_none());
            data.next = Some(location);
        }
        // Normal statement values do not become async function return values.
        // Explicit returns and throws in the selected branch keep their own
        // completion, including the ordinary evaluator's UpdateEmpty behavior.
        Ok(Step::Continue)
    }
}
