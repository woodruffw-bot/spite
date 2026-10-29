//! Value expression continuations; source nodes stay in the retained function AST.

mod assignment;
mod member;
mod object;
mod optional;
mod template;

use super::{AsyncAwaitAction, AsyncExecutionData, Step, active_statement};
use crate::function::CallTarget;
use crate::{
    Error, Realm, Value,
    object::async_expression::{
        AsyncExpressionData, ExprChild, ExprPath, ExprRoot, ExprTask, InvocationTarget,
        MemberAction,
    },
    reference_expression,
};
use spite_core::{JsString, Span};
use spite_parser::ast::{
    Argument, ArrayElement, ArrowBody, BinaryOp, ChainStepKind, Expr, ExprKind, ObjectElement,
    PropertyName, StatementKind, UnaryOp,
};

pub(super) fn has_supported_await(expression: &Expr) -> bool {
    match &expression.kind {
        ExprKind::Await(_) | ExprKind::Yield { .. } => true,
        ExprKind::ImportCall { specifier, options } => has_supported_await(specifier) || options.as_ref().is_some_and(|options| has_supported_await(options)),
        ExprKind::Parenthesized(inner) | ExprKind::Unary(_, inner) => has_supported_await(inner),
        ExprKind::Update { argument, .. } => has_supported_await(argument),
        ExprKind::DestructuringAssign { value, .. } | ExprKind::PrivateIn { value, .. } => {
            has_supported_await(value)
        }
        ExprKind::SuperCall(arguments) => arguments.iter().any(|argument| has_supported_await(argument.expression())),
        ExprKind::SuperProperty(PropertyName::Computed(key)) => has_supported_await(key),
        ExprKind::OptionalChain {base, steps} => has_supported_await(base) || steps.iter().any(|step| match &step.kind {
            ChainStepKind::Property(PropertyName::Computed(key)) => has_supported_await(key),
            ChainStepKind::Call(arguments) => arguments.iter().any(|argument| has_supported_await(argument.expression())),
            _ => false,
        }),
        ExprKind::Object(elements) => elements.iter().any(|element| match element {
            ObjectElement::Spread(value) => has_supported_await(value),
            ObjectElement::Property(property) => has_supported_await(&property.value) || matches!(&property.name, PropertyName::Computed(key) if has_supported_await(key)),
        }),
        ExprKind::Template { substitutions, .. } => substitutions.iter().any(has_supported_await),
        ExprKind::TaggedTemplate {
            tag, substitutions, ..
        } => has_supported_await(tag) || substitutions.iter().any(has_supported_await),
        ExprKind::Binary(_, left, right)
        | ExprKind::Assign(left, right)
        | ExprKind::CompoundAssign(_, left, right) => {
            has_supported_await(left) || has_supported_await(right)
        }
        ExprKind::BinaryChain { head, steps } => {
            has_supported_await(head) || steps.iter().any(|step| has_supported_await(&step.right))
        }
        ExprKind::Conditional(test, yes, no) => {
            has_supported_await(test) || has_supported_await(yes) || has_supported_await(no)
        }
        ExprKind::Call { callee, arguments } => {
            has_supported_await(callee)
                || arguments
                    .iter()
                    .any(|argument| has_supported_await(argument.expression()))
        }
        ExprKind::New { callee, arguments } => {
            has_supported_await(callee)
                || arguments
                    .as_deref()
                    .unwrap_or(&[])
                    .iter()
                    .any(|argument| has_supported_await(argument.expression()))
        }
        ExprKind::Array(elements) => elements
            .iter()
            .filter_map(ArrayElement::expression)
            .any(has_supported_await),
        ExprKind::Member(base, name) => {
            has_supported_await(base)
                || matches!(name, PropertyName::Computed(key) if has_supported_await(key))
        }
        // Unsupported parents retain the ordinary evaluator's explicit boundary.
        // Defining a nested async function does not execute its awaited body.
        _ => false,
    }
}

fn root_expression(data: &AsyncExecutionData, root: ExprRoot) -> &Expr {
    match (&data.body, root) {
        (ArrowBody::Expression(expression), ExprRoot::Arrow) => expression,
        (ArrowBody::Block(_), ExprRoot::ClassHeritage) => {
            let StatementKind::Class(class) =
                &active_statement(&data.body, data.statement.as_ref()).kind
            else {
                unreachable!("class declaration source")
            };
            class.heritage.as_ref().expect("suspended heritage")
        }
        (ArrowBody::Block(_), ExprRoot::Statement) => {
            match &active_statement(&data.body, data.statement.as_ref()).kind {
                StatementKind::Expression(expression)
                | StatementKind::Return(Some(expression))
                | StatementKind::Throw(expression) => expression,
                _ => unreachable!("expression statement root"),
            }
        }
        (ArrowBody::Block(_), ExprRoot::BindingKey) => super::binding_pattern::key_expression(data),
        (ArrowBody::Block(_), ExprRoot::BindingDefault) => {
            super::binding_pattern::default_expression(data)
        }
        (ArrowBody::Block(_), ExprRoot::IterationTarget) => {
            super::iteration_head::iteration_target(data)
        }
        (ArrowBody::Block(_), ExprRoot::IterationSource) => {
            let Some(crate::object::async_execution::AsyncStatementFrame::IterationHead(head)) =
                data.frames.last()
            else {
                unreachable!("for-of source frame");
            };
            match &active_statement(&data.body, Some(&head.location)).kind {
                StatementKind::ForOf { iterable, .. } => iterable,
                StatementKind::ForIn { object, .. } => object,
                _ => unreachable!("iteration source"),
            }
        }
        (ArrowBody::Block(_), ExprRoot::ForHeader) => super::for_loop::header_expression(data),
        (ArrowBody::Block(_), ExprRoot::LoopTest) => {
            let Some(crate::object::async_execution::AsyncStatementFrame::Loop(cursor)) =
                data.frames.last()
            else {
                unreachable!("loop condition frame");
            };
            match &active_statement(&data.body, Some(&cursor.location)).kind {
                StatementKind::While { test, .. } | StatementKind::DoWhile { test, .. } => test,
                _ => unreachable!("loop condition source"),
            }
        }
        (ArrowBody::Block(_), ExprRoot::WithObject) => {
            let StatementKind::With { object, .. } =
                &active_statement(&data.body, data.statement.as_ref()).kind
            else {
                unreachable!("with object source")
            };
            object
        }
        (ArrowBody::Block(_), ExprRoot::SwitchDiscriminant) => {
            let StatementKind::Switch { discriminant, .. } =
                &active_statement(&data.body, data.statement.as_ref()).kind
            else {
                unreachable!("switch discriminant")
            };
            discriminant
        }
        (ArrowBody::Block(_), ExprRoot::SwitchSelector) => {
            let Some(crate::object::async_execution::AsyncStatementFrame::Switch(cursor)) =
                data.frames.last()
            else {
                unreachable!("switch selector frame")
            };
            let StatementKind::Switch { clauses, .. } =
                &active_statement(&data.body, Some(&cursor.location)).kind
            else {
                unreachable!("switch selector source")
            };
            clauses[cursor.clause].test.as_ref().expect("case selector")
        }
        (ArrowBody::Block(_), ExprRoot::IfTest) => {
            let StatementKind::If { test, .. } =
                &active_statement(&data.body, data.statement.as_ref()).kind
            else {
                unreachable!("conditional expression root");
            };
            test
        }
        (ArrowBody::Block(_), ExprRoot::Binding { element }) => {
            let (bindings, _, _) =
                super::binding::binding_source(&data.body, super::binding::binding_location(data));
            bindings[element]
                .initializer
                .as_ref()
                .expect("awaited initializer")
        }
        _ => unreachable!("expression root matches body"),
    }
}

fn expression_at<'a>(data: &'a AsyncExecutionData, root: ExprRoot, path: &[ExprChild]) -> &'a Expr {
    let mut expression = root_expression(data, root);
    for child in path {
        expression = match (child, &expression.kind) {
            (
                ExprChild::Inner,
                ExprKind::Parenthesized(inner)
                | ExprKind::Await(inner)
                | ExprKind::Yield {
                    argument: Some(inner),
                    ..
                }
                | ExprKind::Unary(_, inner)
                | ExprKind::Update {
                    argument: inner, ..
                },
            ) => inner,
            (
                ExprChild::Left,
                ExprKind::Binary(_, left, _)
                | ExprKind::Assign(left, _)
                | ExprKind::CompoundAssign(_, left, _),
            ) => left,
            (
                ExprChild::Right,
                ExprKind::Binary(_, _, right)
                | ExprKind::Assign(_, right)
                | ExprKind::CompoundAssign(_, _, right),
            ) => right,
            (
                ExprChild::Right,
                ExprKind::DestructuringAssign { value, .. } | ExprKind::PrivateIn { value, .. },
            ) => value,
            (ExprChild::ImportSpecifier, ExprKind::ImportCall { specifier, .. }) => specifier,
            (
                ExprChild::ImportOptions,
                ExprKind::ImportCall {
                    options: Some(options),
                    ..
                },
            ) => options,
            (ExprChild::ChainHead, ExprKind::BinaryChain { head, .. }) => head,
            (ExprChild::ChainRight(index), ExprKind::BinaryChain { steps, .. }) => {
                &steps[*index].right
            }
            (ExprChild::Test, ExprKind::Conditional(test, _, _)) => test,
            (ExprChild::Yes, ExprKind::Conditional(_, yes, _)) => yes,
            (ExprChild::No, ExprKind::Conditional(_, _, no)) => no,
            (
                ExprChild::Callee,
                ExprKind::Call { callee, .. }
                | ExprKind::New { callee, .. }
                | ExprKind::TaggedTemplate { tag: callee, .. },
            ) => callee,
            (ExprChild::Argument(index), ExprKind::TaggedTemplate { substitutions, .. }) => {
                &substitutions[*index]
            }
            (ExprChild::Argument(index), ExprKind::SuperCall(arguments)) => {
                arguments[*index].expression()
            }
            (ExprChild::Argument(index), ExprKind::Call { arguments, .. }) => {
                arguments[*index].expression()
            }
            (ExprChild::Argument(index), ExprKind::New { arguments, .. }) => {
                arguments.as_ref().expect("construction arguments")[*index].expression()
            }
            (ExprChild::ArrayElement(index), ExprKind::Array(elements)) => elements[*index]
                .expression()
                .expect("array element expression"),
            (ExprChild::TemplateSubstitution(index), ExprKind::Template { substitutions, .. }) => {
                &substitutions[*index]
            }
            (ExprChild::ObjectSpread(index), ExprKind::Object(elements)) => {
                let ObjectElement::Spread(value) = &elements[*index] else {
                    unreachable!("object spread source")
                };
                value
            }
            (ExprChild::ObjectValue(index), ExprKind::Object(elements)) => {
                let ObjectElement::Property(property) = &elements[*index] else {
                    unreachable!("object property source")
                };
                &property.value
            }
            (ExprChild::ObjectKey(index), ExprKind::Object(elements)) => {
                let ObjectElement::Property(property) = &elements[*index] else {
                    unreachable!("object property source")
                };
                let PropertyName::Computed(value) = &property.name else {
                    unreachable!("computed object key")
                };
                value
            }
            (ExprChild::SuperKey, ExprKind::SuperProperty(PropertyName::Computed(key))) => key,
            (ExprChild::OptionalBase, ExprKind::OptionalChain { base, .. }) => base,
            (ExprChild::OptionalKey(step), ExprKind::OptionalChain { steps, .. }) => {
                let ChainStepKind::Property(PropertyName::Computed(key)) = &steps[*step].kind
                else {
                    unreachable!("optional key source")
                };
                key
            }
            (
                ExprChild::OptionalArgument { step, index },
                ExprKind::OptionalChain { steps, .. },
            ) => {
                let ChainStepKind::Call(arguments) = &steps[*step].kind else {
                    unreachable!("optional argument source")
                };
                arguments[*index].expression()
            }
            (ExprChild::MemberBase, ExprKind::Member(base, _)) => base,
            (ExprChild::MemberKey, ExprKind::Member(_, PropertyName::Computed(key))) => key,
            _ => unreachable!("typed expression path"),
        };
    }
    expression
}

fn invocation_arguments(expression: &Expr) -> &[Argument] {
    match &expression.kind {
        ExprKind::Call { arguments, .. } | ExprKind::SuperCall(arguments) => arguments,
        ExprKind::New { arguments, .. } => arguments.as_deref().unwrap_or(&[]),
        _ => unreachable!("invocation argument cursor"),
    }
}

fn invocation_argument_count(expression: &Expr) -> usize {
    if let ExprKind::TaggedTemplate { substitutions, .. } = &expression.kind {
        substitutions.len()
    } else {
        invocation_arguments(expression).len()
    }
}

fn invocation_argument(expression: &Expr, index: usize) -> (&Expr, bool) {
    if let ExprKind::TaggedTemplate { substitutions, .. } = &expression.kind {
        (&substitutions[index], false)
    } else {
        let argument = &invocation_arguments(expression)[index];
        (
            argument.expression(),
            matches!(argument, Argument::Spread(_)),
        )
    }
}

fn child_path(path: &[ExprChild], child: ExprChild, span: Span) -> Result<ExprPath, Error> {
    let mut result = Vec::new();
    result
        .try_reserve(path.len() + 1)
        .map_err(|_| allocation_error(span))?;
    result.extend_from_slice(path);
    result.push(child);
    Ok(result)
}

fn push(state: &mut AsyncExpressionData, task: ExprTask, span: Span) -> Result<(), Error> {
    state
        .tasks
        .try_reserve(1)
        .map_err(|_| allocation_error(span))?;
    state.tasks.push(task);
    Ok(())
}

fn allocation_error(span: Span) -> Error {
    Error::Limit {
        span,
        message: "async expression continuation allocation failed".into(),
    }
}

fn short_circuit(op: BinaryOp, value: &Value) -> bool {
    match op {
        BinaryOp::And => !value.to_boolean(),
        BinaryOp::Or => value.to_boolean(),
        BinaryOp::Nullish => !matches!(value, Value::Null | Value::Undefined),
        _ => false,
    }
}

impl Realm {
    pub(super) fn begin_async_invocation_arguments(
        &mut self,
        data: &AsyncExecutionData,
        state: &mut AsyncExpressionData,
        path: ExprPath,
        mut target: InvocationTarget,
        span: Span,
    ) -> Result<(), Error> {
        let mut arguments = Vec::new();
        if let ExprKind::TaggedTemplate { elements, .. } =
            &expression_at(data, state.root, &path).kind
        {
            let InvocationTarget::Call(call) = &mut target else {
                unreachable!("tagged call target")
            };
            // Tagged eval is an ordinary indirect call. GetTemplateObject runs
            // after callee/receiver capture and before any substitution.
            call.direct_eval = false;
            let template = self.get_template_object(elements, span)?;
            self.append_argument(&mut arguments, Value::Object(template), span)?;
        }
        push(
            state,
            ExprTask::ArgumentNext {
                path,
                index: 0,
                target,
                arguments,
                span,
            },
            span,
        )
    }

    pub(super) fn start_async_expression(
        &mut self,
        data: &mut AsyncExecutionData,
        root: ExprRoot,
        action: AsyncAwaitAction,
    ) -> Result<Step, Error> {
        let span = root_expression(data, root).span;
        let mut state = AsyncExpressionData {
            root,
            action,
            tasks: Vec::new(),
            current: None,
            delegation: None,
        };
        let task = if matches!(root, ExprRoot::IterationTarget) {
            ExprTask::IterationTarget(Vec::new())
        } else {
            ExprTask::Evaluate(Vec::new())
        };
        push(&mut state, task, span)?;
        data.expression = Some(state);
        self.resume_async_expression(data, None)
    }

    pub(super) fn resume_async_expression(
        &mut self,
        data: &mut AsyncExecutionData,
        value: Option<Value>,
    ) -> Result<Step, Error> {
        let mut state = data.expression.take().expect("active expression");
        if let Some(value) = value {
            state.current = Some(value);
        }
        while let Some(task) = state.tasks.pop() {
            match task {
                ExprTask::IterationTarget(path) => self.begin_async_member(
                    data,
                    &mut state,
                    path,
                    MemberAction::IterationAssignment,
                )?,
                ExprTask::Evaluate(path) => {
                    let expression = expression_at(data, state.root, &path);
                    let span = expression.span;
                    match &expression.kind {
                        ExprKind::Await(_)
                        | ExprKind::Yield { .. }
                        | ExprKind::Parenthesized(_)
                        | ExprKind::Unary(_, _)
                        | ExprKind::Template { .. }
                        | ExprKind::SuperCall(_)
                        | ExprKind::SuperProperty(_)
                        | ExprKind::OptionalChain { .. }
                        | ExprKind::Object(_)
                        | ExprKind::PrivateIn { .. }
                        | ExprKind::DestructuringAssign { .. }
                        | ExprKind::Update { .. }
                        | ExprKind::Assign(..)
                        | ExprKind::CompoundAssign(..)
                        | ExprKind::Binary(..)
                        | ExprKind::BinaryChain { .. }
                        | ExprKind::Conditional(..)
                        | ExprKind::TaggedTemplate { .. }
                        | ExprKind::Call { .. }
                        | ExprKind::ImportCall { .. }
                        | ExprKind::Array(_)
                        | ExprKind::Member(..)
                        | ExprKind::New { .. }
                            if has_supported_await(expression) =>
                        {
                            self.tick(span)?;
                            push(&mut state, ExprTask::CheckString(span), span)?;
                            match &expression.kind {
                                ExprKind::ImportCall { .. } => {
                                    push(
                                        &mut state,
                                        ExprTask::ImportSpecifier {
                                            path: path.clone(),
                                            span,
                                        },
                                        span,
                                    )?;
                                    push(
                                        &mut state,
                                        ExprTask::Evaluate(child_path(
                                            &path,
                                            ExprChild::ImportSpecifier,
                                            span,
                                        )?),
                                        span,
                                    )?;
                                }
                                ExprKind::Yield {
                                    delegated,
                                    argument,
                                } => {
                                    let task = if *delegated {
                                        ExprTask::DelegateYield(span)
                                    } else {
                                        ExprTask::Yield(span)
                                    };
                                    push(&mut state, task, span)?;
                                    if argument.is_some() {
                                        push(
                                            &mut state,
                                            ExprTask::Evaluate(child_path(
                                                &path,
                                                ExprChild::Inner,
                                                span,
                                            )?),
                                            span,
                                        )?;
                                    } else {
                                        state.current = Some(Value::Undefined);
                                    }
                                }
                                ExprKind::Await(_) => {
                                    push(&mut state, ExprTask::Await(span), span)?;
                                    push(
                                        &mut state,
                                        ExprTask::Evaluate(child_path(
                                            &path,
                                            ExprChild::Inner,
                                            span,
                                        )?),
                                        span,
                                    )?;
                                }
                                ExprKind::Parenthesized(_) => push(
                                    &mut state,
                                    ExprTask::Evaluate(child_path(&path, ExprChild::Inner, span)?),
                                    span,
                                )?,
                                ExprKind::Unary(op, inner)
                                    if *op == UnaryOp::Delete && reference_expression(inner) =>
                                {
                                    let inner_path = child_path(&path, ExprChild::Inner, span)?;
                                    self.begin_async_member(
                                        data,
                                        &mut state,
                                        inner_path,
                                        MemberAction::Delete { span: inner.span },
                                    )?;
                                }
                                ExprKind::Unary(op, _) => {
                                    push(&mut state, ExprTask::Unary { op: *op, span }, span)?;
                                    push(
                                        &mut state,
                                        ExprTask::Evaluate(child_path(
                                            &path,
                                            ExprChild::Inner,
                                            span,
                                        )?),
                                        span,
                                    )?;
                                }
                                ExprKind::Template { .. } => push(
                                    &mut state,
                                    ExprTask::TemplateNext {
                                        path,
                                        index: 0,
                                        units: Vec::new(),
                                    },
                                    span,
                                )?,
                                ExprKind::SuperCall(_) => {
                                    let target =
                                        InvocationTarget::Super(self.prepare_super_call(span)?);
                                    self.begin_async_invocation_arguments(
                                        data, &mut state, path, target, span,
                                    )?;
                                }
                                ExprKind::SuperProperty(_) => self.begin_async_member(
                                    data,
                                    &mut state,
                                    path,
                                    MemberAction::Value,
                                )?,
                                ExprKind::OptionalChain { .. } => self.begin_async_optional(
                                    data,
                                    &mut state,
                                    path,
                                    MemberAction::Value,
                                )?,
                                ExprKind::Object(_) => {
                                    let object = Value::Object(self.new_literal_object(span)?);
                                    push(
                                        &mut state,
                                        ExprTask::ObjectNext {
                                            path,
                                            element: 0,
                                            object,
                                        },
                                        span,
                                    )?;
                                }
                                ExprKind::PrivateIn { .. } => {
                                    let value = child_path(&path, ExprChild::Right, span)?;
                                    push(&mut state, ExprTask::PrivateIn(path), span)?;
                                    push(&mut state, ExprTask::Evaluate(value), span)?;
                                }
                                ExprKind::DestructuringAssign { .. } => {
                                    let value = child_path(&path, ExprChild::Right, span)?;
                                    push(&mut state, ExprTask::DestructureApply(path), span)?;
                                    push(&mut state, ExprTask::Evaluate(value), span)?;
                                }
                                ExprKind::Update {
                                    op,
                                    argument,
                                    prefix,
                                } => {
                                    let argument_path = child_path(&path, ExprChild::Inner, span)?;
                                    self.begin_async_member(
                                        data,
                                        &mut state,
                                        argument_path,
                                        MemberAction::Update {
                                            op: *op,
                                            prefix: *prefix,
                                            span,
                                            argument_span: argument.span,
                                        },
                                    )?;
                                }
                                ExprKind::Assign(..) => {
                                    self.begin_async_assignment(data, &mut state, path, None, span)?
                                }
                                ExprKind::CompoundAssign(op, ..) => self.begin_async_assignment(
                                    data,
                                    &mut state,
                                    path,
                                    Some(*op),
                                    span,
                                )?,
                                ExprKind::Binary(op, _, _) => {
                                    push(
                                        &mut state,
                                        ExprTask::BinaryLeft {
                                            op: *op,
                                            right: child_path(&path, ExprChild::Right, span)?,
                                            span,
                                        },
                                        span,
                                    )?;
                                    push(
                                        &mut state,
                                        ExprTask::Evaluate(child_path(
                                            &path,
                                            ExprChild::Left,
                                            span,
                                        )?),
                                        span,
                                    )?;
                                }
                                ExprKind::BinaryChain { .. } => {
                                    let head = child_path(&path, ExprChild::ChainHead, span)?;
                                    push(&mut state, ExprTask::ChainNext { path, index: 0 }, span)?;
                                    push(&mut state, ExprTask::Evaluate(head), span)?;
                                }
                                ExprKind::Conditional(..) => {
                                    push(
                                        &mut state,
                                        ExprTask::Conditional {
                                            yes: child_path(&path, ExprChild::Yes, span)?,
                                            no: child_path(&path, ExprChild::No, span)?,
                                            span,
                                        },
                                        span,
                                    )?;
                                    push(
                                        &mut state,
                                        ExprTask::Evaluate(child_path(
                                            &path,
                                            ExprChild::Test,
                                            span,
                                        )?),
                                        span,
                                    )?;
                                }
                                ExprKind::Call { callee, .. }
                                | ExprKind::TaggedTemplate { tag: callee, .. } => {
                                    if has_supported_await(callee) {
                                        let callee_path =
                                            child_path(&path, ExprChild::Callee, span)?;
                                        if reference_expression(callee) {
                                            self.begin_async_member(
                                                data,
                                                &mut state,
                                                callee_path,
                                                MemberAction::Call { path, span },
                                            )?;
                                        } else {
                                            push(
                                                &mut state,
                                                ExprTask::Callee { path, span },
                                                span,
                                            )?;
                                            push(
                                                &mut state,
                                                ExprTask::Evaluate(callee_path),
                                                span,
                                            )?;
                                        }
                                    } else {
                                        let target =
                                            InvocationTarget::Call(self.prepare_call(callee)?);
                                        self.begin_async_invocation_arguments(
                                            data, &mut state, path, target, span,
                                        )?;
                                    }
                                }
                                ExprKind::New { .. } => {
                                    let callee = child_path(&path, ExprChild::Callee, span)?;
                                    push(&mut state, ExprTask::Callee { path, span }, span)?;
                                    push(&mut state, ExprTask::Evaluate(callee), span)?;
                                }
                                ExprKind::Array(_) => {
                                    let array =
                                        Value::Object(self.create_intrinsic_array(0, span)?);
                                    push(
                                        &mut state,
                                        ExprTask::ArrayNext {
                                            path,
                                            element: 0,
                                            index: 0,
                                            array,
                                            span,
                                        },
                                        span,
                                    )?;
                                }
                                ExprKind::Member(..) => {
                                    self.begin_async_member(
                                        data,
                                        &mut state,
                                        path,
                                        MemberAction::Value,
                                    )?;
                                }
                                _ => unreachable!("supported value node"),
                            }
                        }
                        _ => state.current = Some(self.expression(expression)?),
                    }
                }
                ExprTask::Unary { op, span } => {
                    let value = state.current.take().expect("unary operand");
                    state.current = Some(self.unary_value(op, value, span)?);
                }
                ExprTask::ImportSpecifier { path, span } => {
                    let specifier = state.current.take().expect("import specifier");
                    let ExprKind::ImportCall { options, .. } =
                        &expression_at(data, state.root, &path).kind
                    else {
                        unreachable!("import source path")
                    };
                    if options.is_some() {
                        push(
                            &mut state,
                            ExprTask::ImportOptions { specifier, span },
                            span,
                        )?;
                        push(
                            &mut state,
                            ExprTask::Evaluate(child_path(&path, ExprChild::ImportOptions, span)?),
                            span,
                        )?;
                    } else {
                        state.current =
                            Some(self.import_call_values(specifier, Value::Undefined, span)?);
                    }
                }
                ExprTask::ImportOptions { specifier, span } => {
                    let options = state.current.take().expect("import options");
                    state.current = Some(self.import_call_values(specifier, options, span)?);
                }
                ExprTask::DelegateYield(span) => {
                    let value = state.current.take().expect("yield delegation operand");
                    let iterator = if data.function.is_async_generator() {
                        self.get_async_iterator(value, span)?
                    } else {
                        self.binding_iterator(value, span)?
                    };
                    state.delegation = Some(crate::object::async_expression::GeneratorDelegation {
                        iterator,
                        span,
                    });
                    data.expression = Some(state);
                    return self.resume_yield_delegation(
                        data,
                        super::SourceResume::Value(Value::Undefined),
                    );
                }
                ExprTask::Yield(span) => {
                    let value = state.current.take().expect("yield operand");
                    data.expression = Some(state);
                    if data.function.is_async_generator() {
                        return Ok(Step::Await(
                            value,
                            AsyncAwaitAction::AsyncGeneratorYield(span),
                            span,
                        ));
                    }
                    let result = self.iterator_result(value, false, span)?;
                    return Ok(Step::Yield(result, AsyncAwaitAction::Expression, span));
                }
                ExprTask::Await(span) => {
                    let value = state.current.take().expect("await operand");
                    data.expression = Some(state);
                    return Ok(Step::Await(value, AsyncAwaitAction::Expression, span));
                }
                ExprTask::BinaryLeft { op, right, span } => {
                    let left = state.current.take().expect("left operand");
                    if short_circuit(op, &left) {
                        state.current = Some(left);
                    } else {
                        push(&mut state, ExprTask::BinaryApply { op, left, span }, span)?;
                        push(&mut state, ExprTask::Evaluate(right), span)?;
                    }
                }
                ExprTask::BinaryApply { op, left, span } => {
                    let right = state.current.take().expect("right operand");
                    state.current = Some(self.binary(op, left, right, span)?);
                }
                ExprTask::ChainNext { path, index } => {
                    let expression = expression_at(data, state.root, &path);
                    let ExprKind::BinaryChain { steps, .. } = &expression.kind else {
                        unreachable!("chain");
                    };
                    if let Some(step) = steps.get(index) {
                        self.tick(step.span)?;
                        let left = state.current.take().expect("chain left operand");
                        if short_circuit(step.op, &left) {
                            state.current = Some(left);
                            push(
                                &mut state,
                                ExprTask::ChainNext {
                                    path,
                                    index: index + 1,
                                },
                                step.span,
                            )?;
                        } else {
                            let right = child_path(&path, ExprChild::ChainRight(index), step.span)?;
                            push(
                                &mut state,
                                ExprTask::ChainApply {
                                    op: step.op,
                                    left,
                                    span: step.span,
                                    path,
                                    next: index + 1,
                                },
                                step.span,
                            )?;
                            push(&mut state, ExprTask::Evaluate(right), step.span)?;
                        }
                    }
                }
                ExprTask::ChainApply {
                    op,
                    left,
                    span,
                    path,
                    next,
                } => {
                    let right = state.current.take().expect("chain right operand");
                    let value = self.binary(op, left, right, span)?;
                    self.check_string(&value, span)?;
                    state.current = Some(value);
                    push(&mut state, ExprTask::ChainNext { path, index: next }, span)?;
                }
                ExprTask::Conditional { yes, no, span } => {
                    let test = state.current.take().expect("conditional test");
                    push(
                        &mut state,
                        ExprTask::Evaluate(if test.to_boolean() { yes } else { no }),
                        span,
                    )?;
                }
                ExprTask::Callee { path, span } => {
                    let function = state.current.take().expect("callee value");
                    let target = if matches!(
                        expression_at(data, state.root, &path).kind,
                        ExprKind::New { .. }
                    ) {
                        InvocationTarget::Construct(function)
                    } else {
                        InvocationTarget::Call(CallTarget {
                            function,
                            this: Value::Undefined,
                            direct_eval: false,
                        })
                    };
                    self.begin_async_invocation_arguments(data, &mut state, path, target, span)?;
                }
                ExprTask::ArgumentNext {
                    path,
                    index,
                    target,
                    arguments,
                    span,
                } => {
                    let expression = expression_at(data, state.root, &path);
                    if index == invocation_argument_count(expression) {
                        state.current = Some(match target {
                            InvocationTarget::Call(target) => {
                                self.invoke_call(target, arguments, span)?
                            }
                            InvocationTarget::Super(target) => {
                                self.finish_super_call(target, arguments, span)?
                            }
                            InvocationTarget::Construct(constructor) => {
                                self.construct(constructor, arguments, span)?
                            }
                        });
                    } else {
                        let argument = child_path(&path, ExprChild::Argument(index), span)?;
                        push(
                            &mut state,
                            ExprTask::ArgumentAppend {
                                path,
                                index,
                                target,
                                arguments,
                                span,
                            },
                            span,
                        )?;
                        push(&mut state, ExprTask::Evaluate(argument), span)?;
                    }
                }
                ExprTask::ArgumentAppend {
                    path,
                    index,
                    target,
                    mut arguments,
                    span,
                } => {
                    let expression = expression_at(data, state.root, &path);
                    let (argument, spread) = invocation_argument(expression, index);
                    let value = state.current.take().expect("argument value");
                    self.append_evaluated_argument(&mut arguments, value, spread, argument.span)?;
                    push(
                        &mut state,
                        ExprTask::ArgumentNext {
                            path,
                            index: index + 1,
                            target,
                            arguments,
                            span,
                        },
                        span,
                    )?;
                }
                ExprTask::ArrayNext {
                    path,
                    mut element,
                    mut index,
                    array,
                    span,
                } => {
                    let expression = expression_at(data, state.root, &path);
                    let ExprKind::Array(elements) = &expression.kind else {
                        unreachable!("array accumulation cursor");
                    };
                    let Value::Object(handle) = &array else {
                        unreachable!("intrinsic array");
                    };
                    while matches!(elements.get(element), Some(ArrayElement::Elision)) {
                        index = Self::next_array_literal_index(index, span)?;
                        self.set_property_or_throw(
                            handle,
                            JsString::from("length"),
                            Value::Number(index as f64),
                            span,
                        )?;
                        element += 1;
                    }
                    if element == elements.len() {
                        self.set_property_or_throw(
                            handle,
                            JsString::from("length"),
                            Value::Number(index as f64),
                            span,
                        )?;
                        state.current = Some(array);
                    } else {
                        let child = child_path(&path, ExprChild::ArrayElement(element), span)?;
                        push(
                            &mut state,
                            ExprTask::ArrayAppend {
                                path,
                                element,
                                index,
                                array,
                                span,
                            },
                            span,
                        )?;
                        push(&mut state, ExprTask::Evaluate(child), span)?;
                    }
                }
                ExprTask::ArrayAppend {
                    path,
                    element,
                    index,
                    array,
                    span,
                } => {
                    let expression = expression_at(data, state.root, &path);
                    let ExprKind::Array(elements) = &expression.kind else {
                        unreachable!("array accumulation append");
                    };
                    let syntax = &elements[element];
                    let value = state.current.take().expect("array element value");
                    let Value::Object(handle) = &array else {
                        unreachable!("intrinsic array");
                    };
                    let index = self.append_array_literal_value(
                        handle,
                        index,
                        value,
                        matches!(syntax, ArrayElement::Spread(_)),
                        syntax.expression().expect("array element expression").span,
                    )?;
                    push(
                        &mut state,
                        ExprTask::ArrayNext {
                            path,
                            element: element + 1,
                            index,
                            array,
                            span,
                        },
                        span,
                    )?;
                }
                ExprTask::MemberBase { path, action, span } => {
                    self.async_member_base(data, &mut state, path, action, span)?;
                }
                ExprTask::MemberKey { base, action, span } => {
                    let key = state.current.take().expect("computed member key");
                    self.finish_async_member(
                        data,
                        &mut state,
                        crate::Reference::Property { base, key },
                        action,
                        span,
                    )?;
                }
                ExprTask::TemplateNext { path, index, units } => {
                    self.async_template_next(data, &mut state, path, index, units)?
                }
                ExprTask::TemplateAppend { path, index, units } => {
                    self.async_template_append(data, &mut state, path, index, units)?
                }
                ExprTask::ObjectNext {
                    path,
                    element,
                    object,
                } => self.async_object_next(data, &mut state, path, element, object)?,
                ExprTask::ObjectKey {
                    path,
                    element,
                    object,
                } => {
                    let value = state.current.take().expect("object computed key value");
                    let property = object::property(data, &state, &path, element);
                    let key = self.property_key(value, property.span)?;
                    self.async_object_property(data, &mut state, path, element, object, key)?;
                }
                ExprTask::ObjectValue {
                    path,
                    element,
                    object,
                    key,
                } => {
                    let value = state.current.take().expect("object property value");
                    self.async_object_value(
                        data,
                        &mut state,
                        path,
                        element,
                        object,
                        (object::value_key(key), value),
                    )?;
                }
                ExprTask::ObjectSpread {
                    path,
                    element,
                    object,
                } => {
                    let value = state.current.take().expect("object spread value");
                    let expression = expression_at(data, state.root, &path);
                    let ExprKind::Object(elements) = &expression.kind else {
                        unreachable!("object source")
                    };
                    let ObjectElement::Spread(source) = &elements[element] else {
                        unreachable!("object spread source")
                    };
                    self.copy_spread_properties(object::handle(&object), value, source.span)?;
                    push(
                        &mut state,
                        ExprTask::ObjectNext {
                            path,
                            element: element + 1,
                            object,
                        },
                        expression.span,
                    )?;
                }
                ExprTask::OptionalBase { path, action } => {
                    let value = state.current.take().expect("optional base value");
                    self.async_optional_step(
                        data,
                        &mut state,
                        path,
                        0,
                        crate::Reference::Value(value),
                        action,
                    )?;
                }
                ExprTask::OptionalKey {
                    path,
                    step,
                    base,
                    action,
                } => {
                    let key = state.current.take().expect("optional key value");
                    self.async_optional_step(
                        data,
                        &mut state,
                        path,
                        step + 1,
                        crate::Reference::Property { base, key },
                        action,
                    )?;
                }
                ExprTask::OptionalArgumentNext(cursor) => {
                    self.async_optional_argument_next(data, &mut state, cursor)?
                }
                ExprTask::OptionalArgumentAppend(cursor) => {
                    self.async_optional_argument_append(data, &mut state, cursor)?
                }
                ExprTask::SuperKey {
                    this_value,
                    action,
                    span,
                } => {
                    let key = state.current.take().expect("super computed key value");
                    let reference = self.make_super_property_reference(key, this_value, span)?;
                    self.finish_async_member(data, &mut state, reference, action, span)?;
                }
                ExprTask::PrivateIn(path) => {
                    let expression = expression_at(data, state.root, &path);
                    let ExprKind::PrivateIn { name, .. } = &expression.kind else {
                        unreachable!("private-in source path")
                    };
                    let value = state.current.take().expect("private-in RHS value");
                    state.current = Some(self.private_in(&name.name, value, expression.span)?);
                }
                ExprTask::DestructureApply(path) => {
                    let value = state.current.take().expect("destructuring RHS value");
                    let ExprKind::DestructuringAssign { pattern, .. } =
                        &expression_at(data, state.root, &path).kind
                    else {
                        unreachable!("destructuring source path")
                    };
                    self.destructuring_assignment(pattern, value.clone())?;
                    state.current = Some(value);
                }
                ExprTask::AssignmentApply {
                    reference,
                    left,
                    op,
                    span,
                } => {
                    let right = state.current.take().expect("assignment RHS value");
                    self.apply_async_assignment(&mut state, reference, left, op, right, span)?;
                }
                ExprTask::CheckString(span) => {
                    self.check_string(state.current.as_ref().expect("expression result"), span)?
                }
            }
        }
        let value = state.current.expect("completed expression");
        self.complete_async_value(data, state.action, value)
    }
}
