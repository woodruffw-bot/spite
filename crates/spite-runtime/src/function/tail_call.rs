//! Owned strict tail transfers (15.10), dispatched after caller restoration.

use super::CallTarget;
use crate::{
    Completion, CompletionKind, Error, ExceptionKind, Realm, Value, environment::EnvironmentHandle,
};
use spite_core::Span;
use spite_parser::ast::{BinaryOp, ChainStepKind, Expr, ExprKind};

#[derive(Debug)]
pub(crate) struct TailCall {
    // Retain only the caller's Realm identity after restoring its local scopes.
    // Proxy [[Call]] errors use this Realm (10.5.12), while ordinary and builtin
    // callees enter their own Realms (10.2.1, 10.3.1).
    pub realm: Option<EnvironmentHandle>,
    pub function: Value,
    pub this: Value,
    pub arguments: Vec<Value>,
    pub span: Span,
}

#[derive(Debug)]
pub(crate) enum CallOutcome {
    Value(Value),
    Tail(Box<TailCall>),
}

impl Realm {
    pub(crate) fn return_completion(
        &mut self,
        expression: Option<&Expr>,
    ) -> Result<Completion, Error> {
        let outcome = match expression {
            Some(expression) => self.return_expression(expression)?,
            None => CallOutcome::Value(Value::Undefined),
        };
        let (value, tail_call) = match outcome {
            CallOutcome::Value(value) => (Some(value), None),
            CallOutcome::Tail(call) => (None, Some(call)),
        };
        Ok(Completion {
            kind: CompletionKind::Return,
            value,
            target: None,
            tail_call,
        })
    }

    pub(super) fn with_tail_calls<T>(
        &mut self,
        body: impl FnOnce(&mut Self) -> Result<T, Error>,
    ) -> Result<T, Error> {
        let previous = self.tail_scope.take();
        if self.strict {
            self.tail_scope = Some(self.scopes.len() - 1);
        }
        let result = body(self);
        self.tail_scope = previous;
        result
    }

    pub(super) fn finish_call_outcome(&mut self, outcome: CallOutcome) -> Result<Value, Error> {
        match outcome {
            CallOutcome::Value(value) => Ok(value),
            CallOutcome::Tail(call) => self.call_inner(
                call.function,
                call.this,
                call.arguments,
                call.span,
                call.realm,
            ),
        }
    }

    pub(crate) fn return_expression(&mut self, expression: &Expr) -> Result<CallOutcome, Error> {
        let Some(start) = self.tail_scope else {
            return self.expression(expression).map(CallOutcome::Value);
        };
        // Lexical disposal must follow the actual call, including its failure.
        // Caller resources belong to earlier activations and do not prevent a
        // callee from transferring its own execution context.
        if self.scopes[start..].iter().any(|environment| {
            self.objects
                .environment(environment)
                .expect("active environment")
                .disposable_resources
                .is_some()
        }) {
            return self.expression(expression).map(CallOutcome::Value);
        }
        self.enter_evaluation(expression.span)?;
        let result = self.tail_expression(expression);
        self.evaluation_depth -= 1;
        result
    }

    fn tail_expression(&mut self, mut expression: &Expr) -> Result<CallOutcome, Error> {
        loop {
            match &expression.kind {
                ExprKind::Parenthesized(inner) => {
                    self.tick(expression.span)?;
                    expression = inner;
                }
                ExprKind::Conditional(test, yes, no) => {
                    self.tick(expression.span)?;
                    expression = if self.expression(test)?.to_boolean() {
                        yes
                    } else {
                        no
                    };
                }
                ExprKind::Binary(op, left, right) if tail_operator(*op) => {
                    self.tick(expression.span)?;
                    let value = self.expression(left)?;
                    if skips_right(*op, &value) {
                        return Ok(CallOutcome::Value(value));
                    }
                    expression = right;
                }
                ExprKind::BinaryChain { head, steps }
                    if steps.last().is_some_and(|step| tail_operator(step.op)) =>
                {
                    self.tick(expression.span)?;
                    let (last, prefix) = steps.split_last().expect("nonempty chain");
                    let value = if let Some(previous) = prefix.last() {
                        self.tick(previous.span)?;
                        let value = self.binary_chain(head, prefix)?;
                        self.check_string(&value, previous.span)?;
                        value
                    } else {
                        self.expression(head)?
                    };
                    if skips_right(last.op, &value) {
                        return Ok(CallOutcome::Value(value));
                    }
                    expression = &last.right;
                }
                ExprKind::Call { callee, arguments } => {
                    self.tick(expression.span)?;
                    let target = self.prepare_call(callee)?;
                    let arguments = self.argument_list(arguments)?;
                    return self.tail_call_target(target, arguments, expression.span);
                }
                ExprKind::OptionalChain { base, steps } => {
                    let (last, prefix) = steps.split_last().expect("nonempty optional chain");
                    let ChainStepKind::Call(arguments) = &last.kind else {
                        return self.expression(expression).map(CallOutcome::Value);
                    };
                    self.tick(expression.span)?;
                    let Some(mut reference) = self.optional_chain_prefix(base, prefix)? else {
                        return Ok(CallOutcome::Value(Value::Undefined));
                    };
                    self.tick(last.span)?;
                    let function = self.get(&mut reference, last.span)?;
                    if last.optional && matches!(function, Value::Undefined | Value::Null) {
                        return Ok(CallOutcome::Value(Value::Undefined));
                    }
                    let target = CallTarget {
                        function,
                        this: reference.call_receiver(),
                        direct_eval: false,
                    };
                    let arguments = self.argument_list(arguments)?;
                    return self.tail_call_target(target, arguments, last.span);
                }
                ExprKind::TaggedTemplate {
                    tag,
                    elements,
                    substitutions,
                } => {
                    self.tick(expression.span)?;
                    let (target, arguments) = self.prepare_tagged_template(
                        tag,
                        elements,
                        substitutions,
                        expression.span,
                    )?;
                    return self.tail_call_target(target, arguments, expression.span);
                }
                _ => return self.expression(expression).map(CallOutcome::Value),
            }
        }
    }

    fn tail_call_target(
        &mut self,
        target: CallTarget,
        arguments: Vec<Value>,
        span: Span,
    ) -> Result<CallOutcome, Error> {
        // Direct eval executes before its caller's context is retired.
        if target.direct_eval {
            return self
                .invoke_call(target, arguments, span)
                .map(CallOutcome::Value);
        }
        if !self.is_callable(&target.function, span)? {
            return Err(Self::exception(
                ExceptionKind::TypeError,
                span,
                "value is not callable",
            ));
        }
        let CallTarget { function, this, .. } = target;
        Ok(CallOutcome::Tail(Box::new(TailCall {
            realm: self.record.global_environment.clone(),
            function,
            this,
            arguments,
            span,
        })))
    }
}

fn tail_operator(op: BinaryOp) -> bool {
    matches!(
        op,
        BinaryOp::And | BinaryOp::Or | BinaryOp::Nullish | BinaryOp::Comma
    )
}

fn skips_right(op: BinaryOp, value: &Value) -> bool {
    match op {
        BinaryOp::And => !value.to_boolean(),
        BinaryOp::Or => value.to_boolean(),
        BinaryOp::Nullish => !matches!(value, Value::Null | Value::Undefined),
        _ => false,
    }
}
