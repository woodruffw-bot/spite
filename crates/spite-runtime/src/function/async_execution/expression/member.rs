//! Awaited member Reference evaluation shared by GetValue, calls and deletion.

use super::{AsyncExecutionData, child_path, expression_at, push};
use crate::{
    Error, Realm, Reference, Value,
    object::async_expression::{
        AsyncExpressionData, ExprChild, ExprPath, ExprTask, InvocationTarget, MemberAction,
    },
};
use spite_core::Span;
use spite_parser::ast::{ExprKind, PropertyName};

impl Realm {
    pub(super) fn begin_async_member(
        &mut self,
        data: &AsyncExecutionData,
        state: &mut AsyncExpressionData,
        mut path: ExprPath,
        action: MemberAction,
    ) -> Result<(), Error> {
        let span = expression_at(data, state.root, &path).span;
        while matches!(
            expression_at(data, state.root, &path).kind,
            ExprKind::Parenthesized(_)
        ) {
            path = child_path(&path, ExprChild::Inner, span)?;
        }
        if matches!(
            expression_at(data, state.root, &path).kind,
            ExprKind::OptionalChain { .. }
        ) {
            return self.begin_async_optional(data, state, path, action);
        }
        if let ExprKind::SuperProperty(PropertyName::Computed(_)) =
            &expression_at(data, state.root, &path).kind
        {
            let this_value = self.this_value(span)?;
            let child = child_path(&path, ExprChild::SuperKey, span)?;
            push(
                state,
                ExprTask::SuperKey {
                    this_value,
                    action,
                    span,
                },
                span,
            )?;
            return push(state, ExprTask::Evaluate(child), span);
        }
        debug_assert!(matches!(
            expression_at(data, state.root, &path).kind,
            ExprKind::Member(..)
        ));
        let base = child_path(&path, ExprChild::MemberBase, span)?;
        push(state, ExprTask::MemberBase { path, action, span }, span)?;
        push(state, ExprTask::Evaluate(base), span)
    }

    pub(super) fn async_member_base(
        &mut self,
        data: &AsyncExecutionData,
        state: &mut AsyncExpressionData,
        path: ExprPath,
        action: MemberAction,
        span: Span,
    ) -> Result<(), Error> {
        let base = state.current.take().expect("member base");
        let ExprKind::Member(_, name) = &expression_at(data, state.root, &path).kind else {
            unreachable!("member reference path");
        };
        match name {
            PropertyName::Computed(_) => {
                let key = child_path(&path, ExprChild::MemberKey, span)?;
                push(state, ExprTask::MemberKey { base, action, span }, span)?;
                push(state, ExprTask::Evaluate(key), span)
            }
            PropertyName::Literal(literal) => {
                let key = self.literal_value(literal, span)?;
                self.finish_async_member(
                    data,
                    state,
                    Reference::Property { base, key },
                    action,
                    span,
                )
            }
            PropertyName::Private(name) => {
                let name = self.resolve_private_name(&name.name, name.span)?;
                self.finish_async_member(
                    data,
                    state,
                    Reference::Private { base, name },
                    action,
                    span,
                )
            }
        }
    }

    pub(super) fn finish_async_member(
        &mut self,
        data: &AsyncExecutionData,
        state: &mut AsyncExpressionData,
        mut reference: Reference<'_>,
        action: MemberAction,
        span: Span,
    ) -> Result<(), Error> {
        // Edition 17 leaves computed keys unconverted in the Reference. Core
        // GetValue/Delete performs coercion after both base and key evaluation.
        match action {
            MemberAction::IterationAssignment => {
                let value = data
                    .iteration_value
                    .as_ref()
                    .expect("pending iteration value")
                    .clone();
                self.put(reference, value, span)?;
                state.current = Some(Value::Undefined);
            }
            MemberAction::Value => state.current = Some(self.get(&mut reference, span)?),
            MemberAction::OptionalBase { path, action } => {
                self.async_optional_step(data, state, path, 0, reference, *action)?
            }
            MemberAction::Update {
                op,
                prefix,
                span,
                argument_span,
            } => {
                state.current =
                    Some(self.update_reference(reference, op, prefix, span, argument_span)?)
            }
            MemberAction::Assignment { path, op, span } => {
                self.finish_async_assignment_target(data, state, reference, path, op, span)?
            }
            MemberAction::Delete { span } => {
                state.current = Some(Value::Boolean(self.delete_reference(reference, span)?));
            }
            MemberAction::Call {
                path,
                span: call_span,
            } => {
                let target = InvocationTarget::Call(self.prepare_reference_call(reference, span)?);
                self.begin_async_invocation_arguments(data, state, path, target, call_span)?;
            }
        }
        Ok(())
    }
}
