//! Suspended ChainEvaluation preserving references and nullish suffix skipping (13.3.10).

use super::{AsyncExecutionData, child_path, expression_at, has_supported_await, push};
use crate::{
    Error, Realm, Reference, Value,
    function::CallTarget,
    object::async_expression::{
        AsyncExpressionData, ExprChild, ExprPath, ExprTask, InvocationTarget, MemberAction,
        OptionalCallCursor,
    },
    reference_expression,
};
use spite_parser::ast::{Argument, ChainStepKind, ExprKind, PropertyName};

impl Realm {
    pub(super) fn begin_async_optional(
        &mut self,
        data: &AsyncExecutionData,
        state: &mut AsyncExpressionData,
        path: ExprPath,
        action: MemberAction,
    ) -> Result<(), Error> {
        let expression = expression_at(data, state.root, &path);
        let ExprKind::OptionalChain { base, .. } = &expression.kind else {
            unreachable!("optional source path")
        };
        let span = expression.span;
        if has_supported_await(base) {
            let child = child_path(&path, ExprChild::OptionalBase, span)?;
            if reference_expression(base) {
                self.begin_async_member(
                    data,
                    state,
                    child,
                    MemberAction::OptionalBase {
                        path,
                        action: Box::new(action),
                    },
                )
            } else {
                push(state, ExprTask::OptionalBase { path, action }, span)?;
                push(state, ExprTask::Evaluate(child), span)
            }
        } else {
            let reference = if reference_expression(base) {
                self.reference(base)?
            } else {
                Reference::Value(self.expression(base)?)
            };
            self.async_optional_step(data, state, path, 0, reference, action)
        }
    }

    pub(super) fn async_optional_step(
        &mut self,
        data: &AsyncExecutionData,
        state: &mut AsyncExpressionData,
        path: ExprPath,
        first: usize,
        mut reference: Reference<'_>,
        action: MemberAction,
    ) -> Result<(), Error> {
        let expression = expression_at(data, state.root, &path);
        let ExprKind::OptionalChain { steps, .. } = &expression.kind else {
            unreachable!("optional source path")
        };
        for (index, step) in steps.iter().enumerate().skip(first) {
            self.tick(step.span)?;
            let value = self.get(&mut reference, step.span)?;
            if step.optional && matches!(value, Value::Undefined | Value::Null) {
                return self.finish_async_member(
                    data,
                    state,
                    Reference::Value(Value::Undefined),
                    action,
                    expression.span,
                );
            }
            reference = match &step.kind {
                ChainStepKind::Property(name) => {
                    let key = match name {
                        PropertyName::Computed(key) if has_supported_await(key) => {
                            let child =
                                child_path(&path, ExprChild::OptionalKey(index), step.span)?;
                            push(
                                state,
                                ExprTask::OptionalKey {
                                    path,
                                    step: index,
                                    base: value,
                                    action,
                                },
                                step.span,
                            )?;
                            return push(state, ExprTask::Evaluate(child), step.span);
                        }
                        PropertyName::Computed(key) => self.expression(key)?,
                        PropertyName::Literal(literal) => self.literal_value(literal, step.span)?,
                        PropertyName::Private(name) => {
                            reference = Reference::Private {
                                base: value,
                                name: self.resolve_private_name(&name.name, name.span)?,
                            };
                            continue;
                        }
                    };
                    Reference::Property { base: value, key }
                }
                ChainStepKind::Call(_) => {
                    let target = InvocationTarget::Call(CallTarget {
                        function: value,
                        this: reference.call_receiver(),
                        direct_eval: false,
                    });
                    return push(
                        state,
                        ExprTask::OptionalArgumentNext(OptionalCallCursor {
                            path,
                            step: index,
                            index: 0,
                            target,
                            arguments: Vec::new(),
                            action,
                        }),
                        step.span,
                    );
                }
            };
        }
        self.finish_async_member(data, state, reference, action, expression.span)
    }

    pub(super) fn async_optional_argument_next(
        &mut self,
        data: &AsyncExecutionData,
        state: &mut AsyncExpressionData,
        cursor: OptionalCallCursor,
    ) -> Result<(), Error> {
        let expression = expression_at(data, state.root, &cursor.path);
        let ExprKind::OptionalChain { steps, .. } = &expression.kind else {
            unreachable!("optional source")
        };
        let step = &steps[cursor.step];
        let ChainStepKind::Call(arguments) = &step.kind else {
            unreachable!("optional call source")
        };
        if cursor.index == arguments.len() {
            let InvocationTarget::Call(target) = cursor.target else {
                unreachable!("optional call target")
            };
            let value = self.invoke_call(target, cursor.arguments, step.span)?;
            self.async_optional_step(
                data,
                state,
                cursor.path,
                cursor.step + 1,
                Reference::Value(value),
                cursor.action,
            )
        } else {
            let child = child_path(
                &cursor.path,
                ExprChild::OptionalArgument {
                    step: cursor.step,
                    index: cursor.index,
                },
                step.span,
            )?;
            push(state, ExprTask::OptionalArgumentAppend(cursor), step.span)?;
            push(state, ExprTask::Evaluate(child), step.span)
        }
    }

    pub(super) fn async_optional_argument_append(
        &mut self,
        data: &AsyncExecutionData,
        state: &mut AsyncExpressionData,
        mut cursor: OptionalCallCursor,
    ) -> Result<(), Error> {
        let expression = expression_at(data, state.root, &cursor.path);
        let ExprKind::OptionalChain { steps, .. } = &expression.kind else {
            unreachable!("optional source")
        };
        let step = &steps[cursor.step];
        let ChainStepKind::Call(arguments) = &step.kind else {
            unreachable!("optional call source")
        };
        let argument = &arguments[cursor.index];
        let value = state.current.take().expect("optional argument value");
        self.append_evaluated_argument(
            &mut cursor.arguments,
            value,
            matches!(argument, Argument::Spread(_)),
            argument.expression().span,
        )?;
        cursor.index += 1;
        push(state, ExprTask::OptionalArgumentNext(cursor), step.span)
    }
}
