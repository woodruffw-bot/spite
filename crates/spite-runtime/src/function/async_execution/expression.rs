//! Value expression continuations; source nodes stay in the retained function AST.

mod member;

use super::{AsyncAwaitAction, AsyncExecutionData, Step};
use crate::function::CallTarget;
use crate::{
    Error, Realm, Value,
    object::async_expression::{
        AsyncExpressionData, ExprChild, ExprPath, ExprRoot, ExprTask, MemberAction,
    },
    reference_expression,
};
use spite_core::{JsString, Span};
use spite_parser::ast::{
    Argument, ArrayElement, ArrowBody, BinaryOp, Expr, ExprKind, PropertyName, StatementKind,
    UnaryOp,
};

pub(super) fn has_supported_await(expression: &Expr) -> bool {
    match &expression.kind {
        ExprKind::Await(_) => true,
        ExprKind::Parenthesized(inner) | ExprKind::Unary(_, inner) => has_supported_await(inner),
        ExprKind::Binary(_, left, right) => has_supported_await(left) || has_supported_await(right),
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

fn root_expression(body: &ArrowBody, root: ExprRoot) -> &Expr {
    match (body, root) {
        (ArrowBody::Expression(expression), ExprRoot::Arrow) => expression,
        (ArrowBody::Block(body), ExprRoot::Statement(index)) => {
            match &body.statements()[index].kind {
                StatementKind::Expression(expression) | StatementKind::Return(Some(expression)) => {
                    expression
                }
                _ => unreachable!("expression statement root"),
            }
        }
        (ArrowBody::Block(body), ExprRoot::Binding { statement, element }) => {
            let bindings = match &body.statements()[statement].kind {
                StatementKind::Lexical { bindings, .. }
                | StatementKind::Using(bindings)
                | StatementKind::Var(bindings) => bindings,
                _ => unreachable!("binding expression root"),
            };
            bindings[element]
                .initializer
                .as_ref()
                .expect("awaited initializer")
        }
        _ => unreachable!("expression root matches body"),
    }
}

fn expression_at<'a>(body: &'a ArrowBody, root: ExprRoot, path: &[ExprChild]) -> &'a Expr {
    let mut expression = root_expression(body, root);
    for child in path {
        expression = match (child, &expression.kind) {
            (
                ExprChild::Inner,
                ExprKind::Parenthesized(inner) | ExprKind::Await(inner) | ExprKind::Unary(_, inner),
            ) => inner,
            (ExprChild::Left, ExprKind::Binary(_, left, _)) => left,
            (ExprChild::Right, ExprKind::Binary(_, _, right)) => right,
            (ExprChild::ChainHead, ExprKind::BinaryChain { head, .. }) => head,
            (ExprChild::ChainRight(index), ExprKind::BinaryChain { steps, .. }) => {
                &steps[*index].right
            }
            (ExprChild::Test, ExprKind::Conditional(test, _, _)) => test,
            (ExprChild::Yes, ExprKind::Conditional(_, yes, _)) => yes,
            (ExprChild::No, ExprKind::Conditional(_, _, no)) => no,
            (ExprChild::CallCallee, ExprKind::Call { callee, .. }) => callee,
            (ExprChild::CallArgument(index), ExprKind::Call { arguments, .. }) => {
                arguments[*index].expression()
            }
            (ExprChild::ArrayElement(index), ExprKind::Array(elements)) => elements[*index]
                .expression()
                .expect("array element expression"),
            (ExprChild::MemberBase, ExprKind::Member(base, _)) => base,
            (ExprChild::MemberKey, ExprKind::Member(_, PropertyName::Computed(key))) => key,
            _ => unreachable!("typed expression path"),
        };
    }
    expression
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
    pub(super) fn start_async_expression(
        &mut self,
        data: &mut AsyncExecutionData,
        root: ExprRoot,
        action: AsyncAwaitAction,
    ) -> Result<Step, Error> {
        let span = root_expression(&data.body, root).span;
        let mut state = AsyncExpressionData {
            root,
            action,
            tasks: Vec::new(),
            current: None,
        };
        push(&mut state, ExprTask::Evaluate(Vec::new()), span)?;
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
                ExprTask::Evaluate(path) => {
                    let expression = expression_at(&data.body, state.root, &path);
                    let span = expression.span;
                    match &expression.kind {
                        ExprKind::Await(_)
                        | ExprKind::Parenthesized(_)
                        | ExprKind::Unary(_, _)
                        | ExprKind::Binary(..)
                        | ExprKind::BinaryChain { .. }
                        | ExprKind::Conditional(..)
                        | ExprKind::Call { .. }
                        | ExprKind::Array(_)
                        | ExprKind::Member(..)
                            if has_supported_await(expression) =>
                        {
                            self.tick(span)?;
                            push(&mut state, ExprTask::CheckString(span), span)?;
                            match &expression.kind {
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
                                        &data.body,
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
                                ExprKind::Call { callee, .. } => {
                                    if has_supported_await(callee) {
                                        let callee_path =
                                            child_path(&path, ExprChild::CallCallee, span)?;
                                        if reference_expression(callee) {
                                            self.begin_async_member(
                                                &data.body,
                                                &mut state,
                                                callee_path,
                                                MemberAction::Call { path, span },
                                            )?;
                                        } else {
                                            push(
                                                &mut state,
                                                ExprTask::CallCallee { path, span },
                                                span,
                                            )?;
                                            push(
                                                &mut state,
                                                ExprTask::Evaluate(callee_path),
                                                span,
                                            )?;
                                        }
                                    } else {
                                        let target = self.prepare_call(callee)?;
                                        push(
                                            &mut state,
                                            ExprTask::CallNext {
                                                path,
                                                index: 0,
                                                target,
                                                arguments: Vec::new(),
                                                span,
                                            },
                                            span,
                                        )?;
                                    }
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
                                        &data.body,
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
                    let expression = expression_at(&data.body, state.root, &path);
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
                ExprTask::CallCallee { path, span } => {
                    let function = state.current.take().expect("awaited callee");
                    let target = CallTarget {
                        function,
                        this: Value::Undefined,
                        direct_eval: false,
                    };
                    push(
                        &mut state,
                        ExprTask::CallNext {
                            path,
                            index: 0,
                            target,
                            arguments: Vec::new(),
                            span,
                        },
                        span,
                    )?;
                }
                ExprTask::CallNext {
                    path,
                    index,
                    target,
                    arguments,
                    span,
                } => {
                    let expression = expression_at(&data.body, state.root, &path);
                    let ExprKind::Call {
                        arguments: syntax, ..
                    } = &expression.kind
                    else {
                        unreachable!("call argument cursor");
                    };
                    if index == syntax.len() {
                        state.current = Some(self.invoke_call(target, arguments, span)?);
                    } else {
                        let argument = child_path(&path, ExprChild::CallArgument(index), span)?;
                        push(
                            &mut state,
                            ExprTask::CallAppend {
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
                ExprTask::CallAppend {
                    path,
                    index,
                    target,
                    mut arguments,
                    span,
                } => {
                    let expression = expression_at(&data.body, state.root, &path);
                    let ExprKind::Call {
                        arguments: syntax, ..
                    } = &expression.kind
                    else {
                        unreachable!("call argument append");
                    };
                    let argument = &syntax[index];
                    let value = state.current.take().expect("argument value");
                    self.append_evaluated_argument(
                        &mut arguments,
                        value,
                        matches!(argument, Argument::Spread(_)),
                        argument.expression().span,
                    )?;
                    push(
                        &mut state,
                        ExprTask::CallNext {
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
                    let expression = expression_at(&data.body, state.root, &path);
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
                    let expression = expression_at(&data.body, state.root, &path);
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
                    self.async_member_base(&data.body, &mut state, path, action, span)?;
                }
                ExprTask::MemberKey { base, action, span } => {
                    let key = state.current.take().expect("computed member key");
                    self.finish_async_member(
                        &mut state,
                        crate::Reference::Property { base, key },
                        action,
                        span,
                    )?;
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
