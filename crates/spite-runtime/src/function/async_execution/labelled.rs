//! LabelledEvaluation without a lexical environment (14.13).

use super::{AsyncExecutionData, Step, active_statement, child_location};
use crate::{
    Completion, CompletionKind, Error, Realm,
    object::async_execution::{AsyncStatementFrame, StatementChild},
};
use spite_core::Span;
use spite_parser::ast::StatementKind;

// Only a direct chain of labels belongs to an iteration statement. The caller
// has popped its loop cursor, leaving those label frames immediately below it.
pub(super) fn loop_continues(data: &AsyncExecutionData, completion: &Completion) -> bool {
    if completion.kind == CompletionKind::Normal {
        return true;
    }
    if completion.kind != CompletionKind::Continue {
        return false;
    }
    let Some(target) = completion.target.as_deref() else {
        return true;
    };
    for frame in data.frames.iter().rev() {
        let AsyncStatementFrame::Label(location) = frame else {
            break;
        };
        let StatementKind::Labelled { label, .. } =
            &active_statement(&data.body, Some(location)).kind
        else {
            unreachable!("loop label source");
        };
        if label.name == target {
            return true;
        }
    }
    false
}

impl Realm {
    pub(super) fn enter_async_label(
        &mut self,
        data: &mut AsyncExecutionData,
        span: Span,
    ) -> Result<Step, Error> {
        data.frames.try_reserve(1).map_err(|_| Error::Limit {
            span,
            message: "async label continuation allocation failed".into(),
        })?;
        let location = data.statement.take().expect("label source location");
        let body = child_location(&location, StatementChild::LabelBody, span)?;
        data.frames.push(AsyncStatementFrame::Label(location));
        data.next = Some(body);
        Ok(Step::Continue)
    }
}
