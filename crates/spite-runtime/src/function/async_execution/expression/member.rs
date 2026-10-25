//! Awaited member Reference evaluation shared by GetValue, calls and deletion.

use super::{child_path, expression_at, push};
use crate::{
    Error, Realm, Reference, Value,
    object::async_expression::{AsyncExpressionData, ExprChild, ExprPath, ExprTask, MemberAction},
};
use spite_core::Span;
use spite_parser::ast::{ArrowBody, ExprKind, PropertyName};

impl Realm {
    pub(super) fn begin_async_member(
        &mut self,
        body: &ArrowBody,
        state: &mut AsyncExpressionData,
        mut path: ExprPath,
        action: MemberAction,
    ) -> Result<(), Error> {
        let span = expression_at(body, state.root, &path).span;
        while matches!(
            expression_at(body, state.root, &path).kind,
            ExprKind::Parenthesized(_)
        ) {
            path = child_path(&path, ExprChild::Inner, span)?;
        }
        debug_assert!(matches!(
            expression_at(body, state.root, &path).kind,
            ExprKind::Member(..)
        ));
        let base = child_path(&path, ExprChild::MemberBase, span)?;
        push(state, ExprTask::MemberBase { path, action, span }, span)?;
        push(state, ExprTask::Evaluate(base), span)
    }

    pub(super) fn async_member_base(
        &mut self,
        body: &ArrowBody,
        state: &mut AsyncExpressionData,
        path: ExprPath,
        action: MemberAction,
        span: Span,
    ) -> Result<(), Error> {
        let base = state.current.take().expect("member base");
        let ExprKind::Member(_, name) = &expression_at(body, state.root, &path).kind else {
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
                self.finish_async_member(state, Reference::Property { base, key }, action, span)
            }
            PropertyName::Private(name) => {
                let name = self.resolve_private_name(&name.name, name.span)?;
                self.finish_async_member(state, Reference::Private { base, name }, action, span)
            }
        }
    }

    pub(super) fn finish_async_member(
        &mut self,
        state: &mut AsyncExpressionData,
        mut reference: Reference<'_>,
        action: MemberAction,
        span: Span,
    ) -> Result<(), Error> {
        // Edition 17 leaves computed keys unconverted in the Reference. Core
        // GetValue/Delete performs coercion after both base and key evaluation.
        match action {
            MemberAction::Value => state.current = Some(self.get(&mut reference, span)?),
            MemberAction::Delete { span } => {
                state.current = Some(Value::Boolean(self.delete_reference(reference, span)?));
            }
            MemberAction::Call {
                path,
                span: call_span,
            } => {
                let target = self.prepare_reference_call(reference, span)?;
                push(
                    state,
                    ExprTask::CallNext {
                        path,
                        index: 0,
                        target,
                        arguments: Vec::new(),
                        span: call_span,
                    },
                    call_span,
                )?;
            }
        }
        Ok(())
    }
}
