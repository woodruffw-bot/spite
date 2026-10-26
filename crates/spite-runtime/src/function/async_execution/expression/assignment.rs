//! Assignment Reference capture, short circuiting and PutValue (13.15.2).

use super::{AsyncExecutionData, child_path, expression_at, has_supported_await, push};
use crate::{
    Error, Realm, Reference, Value,
    object::async_expression::{AsyncExpressionData, ExprChild, ExprPath, ExprTask, MemberAction},
};
use spite_core::Span;
use spite_parser::ast::{BinaryOp, ExprKind};

fn operands<'a>(
    data: &'a AsyncExecutionData,
    state: &AsyncExpressionData,
    path: &ExprPath,
) -> (&'a spite_parser::ast::Expr, &'a spite_parser::ast::Expr) {
    match &expression_at(data, state.root, path).kind {
        ExprKind::Assign(target, right) | ExprKind::CompoundAssign(_, target, right) => {
            (target, right)
        }
        _ => unreachable!("assignment source path"),
    }
}

impl Realm {
    pub(super) fn begin_async_assignment(
        &mut self,
        data: &AsyncExecutionData,
        state: &mut AsyncExpressionData,
        path: ExprPath,
        op: Option<BinaryOp>,
        span: Span,
    ) -> Result<(), Error> {
        let (target, _) = operands(data, state, &path);
        if has_supported_await(target) {
            let target = child_path(&path, ExprChild::Left, span)?;
            // Only ordinary/private Member targets currently contain supported
            // awaits. Parentheses retain reference identity through this path.
            self.begin_async_member(
                data,
                state,
                target,
                MemberAction::Assignment { path, op, span },
            )
        } else {
            let reference = self.reference(target)?;
            self.finish_async_assignment_target(data, state, reference, path, op, span)
        }
    }

    pub(super) fn finish_async_assignment_target(
        &mut self,
        data: &AsyncExecutionData,
        state: &mut AsyncExpressionData,
        mut reference: Reference<'_>,
        path: ExprPath,
        op: Option<BinaryOp>,
        span: Span,
    ) -> Result<(), Error> {
        let left = if op.is_some() {
            Some(self.get(&mut reference, span)?)
        } else {
            None
        };
        if let (Some(op), Some(left)) = (op, left.as_ref()) {
            let short = match op {
                BinaryOp::And => !left.to_boolean(),
                BinaryOp::Or => left.to_boolean(),
                BinaryOp::Nullish => !matches!(left, Value::Null | Value::Undefined),
                _ => false,
            };
            if short {
                state.current = Some(left.clone());
                return Ok(());
            }
        }
        let (target, right) = operands(data, state, &path);
        if !has_supported_await(right) {
            // NamedEvaluation must run before a class's static elements. The
            // shared operation also preserves parenthesized-target semantics.
            let right = if op.is_none()
                || matches!(op, Some(BinaryOp::And | BinaryOp::Or | BinaryOp::Nullish))
            {
                self.assignment_expression(target, right)?
            } else {
                self.expression(right)?
            };
            return self.apply_async_assignment(state, reference, left, op, right, span);
        }
        // Simple assignment retains a raw computed key until PutValue. Compound
        // GetValue has already cached its converted key before RHS evaluation.
        let reference = reference.into_owned(span)?;
        let right = child_path(&path, ExprChild::Right, span)?;
        push(
            state,
            ExprTask::AssignmentApply {
                reference,
                left,
                op,
                span,
            },
            span,
        )?;
        push(state, ExprTask::Evaluate(right), span)
    }

    pub(super) fn apply_async_assignment(
        &mut self,
        state: &mut AsyncExpressionData,
        reference: Reference<'_>,
        left: Option<Value>,
        op: Option<BinaryOp>,
        right: Value,
        span: Span,
    ) -> Result<(), Error> {
        let value = if let Some(op) = op {
            self.binary(op, left.expect("compound assignment LHS"), right, span)?
        } else {
            right
        };
        self.put(reference, value.clone(), span)?;
        state.current = Some(value);
        Ok(())
    }
}
