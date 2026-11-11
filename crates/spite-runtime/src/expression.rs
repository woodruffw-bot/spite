//! Expression evaluation and iterative binary chains.

use crate::realm_object::Hint;
use crate::reference::Reference;
use crate::value::{exponentiate, to_uint32};
use crate::{Error, ExceptionKind, Realm, Value};
use spite_bigint::BitwiseOp;
use spite_core::{JsString, Span};
use spite_parser::ast::{BinaryOp, Expr, ExprKind, UnaryOp};

impl Realm {
    pub(crate) fn check_evaluation_depth(&self, span: Span) -> Result<(), Error> {
        // Bound recursive statement/expression work across function calls.
        if self.evaluation_depth >= 64 {
            return Err(Error::Limit {
                span,
                message: "evaluation nesting limit exceeded".into(),
            });
        }
        Ok(())
    }

    pub(crate) fn enter_evaluation(&mut self, span: Span) -> Result<(), Error> {
        self.check_evaluation_depth(span)?;
        self.evaluation_depth += 1;
        Ok(())
    }

    pub(crate) fn expression(&mut self, expr: &Expr) -> Result<Value, Error> {
        self.enter_evaluation(expr.span)?;
        // Debug builds reserve space for every general expression branch. Do
        // not retain that frame through calls and recursive eval compilation.
        let result = match &expr.kind {
            ExprKind::Call { callee, arguments } => {
                self.call_expression(callee, arguments, expr.span)
            }
            ExprKind::SuperCall(arguments) => self.super_call(arguments, expr.span),
            _ => self.expression_inner(expr),
        };
        self.evaluation_depth -= 1;
        result
    }

    pub(crate) fn call_expression(
        &mut self,
        callee: &Expr,
        arguments: &[spite_parser::ast::Argument],
        span: Span,
    ) -> Result<Value, Error> {
        self.tick(span)?;
        let target = self.prepare_call(callee)?;
        let values = self.argument_list(arguments)?;
        let result = self.invoke_call(target, values, span)?;
        self.check_string(&result, span)?;
        Ok(result)
    }

    pub(crate) fn expression_inner(&mut self, expr: &Expr) -> Result<Value, Error> {
        self.tick(expr.span)?;
        let result = match &expr.kind {
            ExprKind::This => self.this_value(expr.span)?,
            ExprKind::NewTarget => self.new_target_value(expr.span)?,
            ExprKind::Function(function) => self.ordinary_function(function, true, expr.span)?,
            ExprKind::Class(class) => self.class_definition(class, None)?,
            ExprKind::Arrow {
                kind,
                parameters,
                body,
                source,
            } => self.arrow_function(*kind, parameters, body, source, expr.span)?,
            ExprKind::Object(properties) => self.object_literal(properties, expr.span)?,
            ExprKind::Array(elements) => self.array_literal(elements, expr.span)?,
            ExprKind::New { callee, arguments } => {
                let constructor = self.expression(callee)?;
                let values = self.argument_list(arguments.as_deref().unwrap_or(&[]))?;
                self.construct(constructor, values, expr.span)?
            }
            ExprKind::Call { .. } | ExprKind::SuperCall(_) => {
                unreachable!("call dispatched separately")
            }
            ExprKind::TaggedTemplate {
                tag,
                elements,
                substitutions,
            } => self.tagged_template(tag, elements, substitutions, expr.span)?,
            ExprKind::Template {
                elements,
                substitutions,
            } if substitutions.is_empty() => {
                let element = &elements[0];
                self.tick(element.span)?;
                self.template_string(
                    element
                        .cooked
                        .as_ref()
                        .expect("validated untagged template"),
                    element.span,
                )?
            }
            ExprKind::Template {
                elements,
                substitutions,
            } => {
                // ECMA-262 13.2.8.6: each substitution is evaluated and converted
                // with ToString before the next one. Raw text is not evaluated.
                let mut units = Vec::new();
                for (index, element) in elements.iter().enumerate() {
                    self.append_template_element(&mut units, element)?;
                    if let Some(substitution) = substitutions.get(index) {
                        let value = self.expression(substitution)?;
                        let value = self.string(value, substitution.span)?;
                        self.append_budgeted_string(&mut units, &value, substitution.span)?;
                    }
                }
                Value::String(JsString::from_code_units(units))
            }
            ExprKind::Literal(literal) => self.literal_value(literal, expr.span)?,
            ExprKind::PrivateIn { name, value } => {
                let value = self.expression(value)?;
                self.private_in(&name.name, value, expr.span)?
            }
            ExprKind::Identifier(name) => {
                let mut reference = self.resolve(name, expr.span)?;
                self.get(&mut reference, expr.span)?
            }
            ExprKind::Parenthesized(inner) => self.expression(inner)?,
            ExprKind::Member(..) | ExprKind::SuperProperty(_) | ExprKind::OptionalChain { .. } => {
                let mut reference = self.reference(expr)?;
                self.get(&mut reference, expr.span)?
            }
            ExprKind::Assign(target, right) => {
                let reference = self.reference(target)?;
                let value = self.assignment_expression(target, right)?;
                self.put(reference, value.clone(), expr.span)?;
                value
            }
            ExprKind::DestructuringAssign { pattern, value } => {
                let value = self.expression(value)?;
                self.destructuring_assignment(pattern, value.clone())?;
                value
            }
            ExprKind::CompoundAssign(op, target, right) => {
                // ECMA-262 13.15.2: read the reference before the RHS. Logical
                // assignments that short-circuit perform neither RHS nor PutValue.
                let mut reference = self.reference(target)?;
                let left = self.get(&mut reference, expr.span)?;
                match op {
                    BinaryOp::And if !left.to_boolean() => left,
                    BinaryOp::Or if left.to_boolean() => left,
                    BinaryOp::Nullish if !matches!(left, Value::Null | Value::Undefined) => left,
                    _ => {
                        let right =
                            if matches!(op, BinaryOp::And | BinaryOp::Or | BinaryOp::Nullish) {
                                self.assignment_expression(target, right)?
                            } else {
                                self.expression(right)?
                            };
                        let value = self.binary(*op, left, right, expr.span)?;
                        self.put(reference, value.clone(), expr.span)?;
                        value
                    }
                }
            }
            ExprKind::Update {
                op,
                argument,
                prefix,
            } => {
                let reference = self.reference(argument)?;
                self.update_reference(reference, *op, *prefix, expr.span, argument.span)?
            }
            ExprKind::Conditional(test, yes, no) => {
                if self.expression(test)?.to_boolean() {
                    self.expression(yes)?
                } else {
                    self.expression(no)?
                }
            }
            ExprKind::ImportMeta => self.import_meta_value(expr.span)?,
            ExprKind::ImportCall { specifier, options } => {
                let specifier = self.expression(specifier)?;
                let options = if let Some(options) = options {
                    self.expression(options)?
                } else {
                    Value::Undefined
                };
                self.import_call_values(specifier, options, expr.span)?
            }
            ExprKind::Yield { .. } => {
                unreachable!("yield is handled by the resumable execution driver")
            }
            ExprKind::Await(_) => {
                unreachable!("await is handled by the async execution driver")
            }
            ExprKind::Unary(op, inner) => {
                if *op == UnaryOp::Delete && reference_expression(inner) {
                    // ECMA-262 13.5.1.2: deleting an environment reference
                    // does not GetValue, even for an uninitialized binding.
                    let reference = self.reference(inner)?;
                    let deleted = self.delete_reference(reference, inner.span)?;
                    return Ok(Value::Boolean(deleted));
                }
                let value = if *op == UnaryOp::Typeof && identifier(inner).is_some() {
                    // Resolve once: object-environment lookup can execute getters.
                    let mut reference = self.reference(inner)?;
                    if matches!(reference, Reference::Unresolvable(_)) {
                        Value::Undefined
                    } else {
                        self.get(&mut reference, inner.span)?
                    }
                } else {
                    self.expression(inner)?
                };
                self.unary_value(*op, value, expr.span)?
            }
            ExprKind::Binary(op, left, right) => {
                let left = self.expression(left)?;
                match op {
                    BinaryOp::And if !left.to_boolean() => left,
                    BinaryOp::Or if left.to_boolean() => left,
                    BinaryOp::Nullish if !matches!(left, Value::Null | Value::Undefined) => left,
                    _ => {
                        let right = self.expression(right)?;
                        self.binary(*op, left, right, expr.span)?
                    }
                }
            }
            ExprKind::BinaryChain { head, steps } => self.binary_chain(head, steps)?,
        };
        self.check_string(&result, expr.span)?;
        Ok(result)
    }

    // Preserve the left spine's source order and per-operation coercions (13.6–13.13, 13.16).
    // Right operands retain their original precedence and parenthesized syntax.
    #[inline(never)]
    pub(crate) fn binary_chain(
        &mut self,
        head: &Expr,
        steps: &[spite_parser::ast::BinaryStep],
    ) -> Result<Value, Error> {
        // The outer expression has already consumed one tick. A recursive left
        // spine would enter its remaining binary nodes before evaluating `head`.
        for step in steps.iter().rev().skip(1) {
            self.tick(step.span)?;
        }
        let mut value = self.expression(head)?;
        for (index, step) in steps.iter().enumerate() {
            let skip = match step.op {
                BinaryOp::And => !value.to_boolean(),
                BinaryOp::Or => value.to_boolean(),
                BinaryOp::Nullish => !matches!(value, Value::Null | Value::Undefined),
                _ => false,
            };
            if !skip {
                let right = self.expression(&step.right)?;
                value = self.binary(step.op, value, right, step.span)?;
            }
            if index + 1 < steps.len() {
                self.check_string(&value, step.span)?;
            }
        }
        // expression_inner checks the complete final result at the outer span.
        Ok(value)
    }

    pub(crate) fn binary(
        &mut self,
        op: BinaryOp,
        mut left: Value,
        mut right: Value,
        span: Span,
    ) -> Result<Value, Error> {
        use BinaryOp::*;
        match op {
            And | Or | Nullish | Comma => return Ok(right),
            Instanceof => return Ok(Value::Boolean(self.instance_of(left, right, span)?)),
            In => {
                let Value::Object(object) = right else {
                    return Err(Self::exception(
                        ExceptionKind::TypeError,
                        span,
                        "right operand of in must be an object",
                    ));
                };
                let key = self.property_key(left, span)?;
                return Ok(Value::Boolean(self.has_property(&object, &key, span)?));
            }
            StrictEqual | StrictNotEqual => {
                let equal = self.strictly_equal(&left, &right, span)?;
                return Ok(Value::Boolean(if op == StrictEqual {
                    equal
                } else {
                    !equal
                }));
            }
            Equal | NotEqual => {
                if matches!(
                    (&left, &right),
                    (
                        Value::Object(_),
                        Value::String(_)
                            | Value::Number(_)
                            | Value::BigInt(_)
                            | Value::Boolean(_)
                            | Value::Symbol(_)
                    )
                ) {
                    left = self.primitive(left, Hint::Default, span)?;
                } else if matches!(
                    (&left, &right),
                    (
                        Value::String(_)
                            | Value::Number(_)
                            | Value::BigInt(_)
                            | Value::Boolean(_)
                            | Value::Symbol(_),
                        Value::Object(_)
                    )
                ) {
                    right = self.primitive(right, Hint::Default, span)?;
                }
                self.comparison_work(&left, &right, span)?;
                let equal =
                    self.conversion_work(span, |budget| left.loosely_equal(&right, budget))?;
                return Ok(Value::Boolean(if op == Equal { equal } else { !equal }));
            }
            Add => {
                left = self.primitive(left, Hint::Default, span)?;
                right = self.primitive(right, Hint::Default, span)?;
                if matches!(left, Value::String(_)) || matches!(right, Value::String(_)) {
                    let a = self.string(left, span)?;
                    let b = self.string(right, span)?;
                    if a.len().checked_add(b.len()).is_none_or(|length| {
                        self.limits
                            .max_string_units
                            .is_some_and(|limit| length > limit)
                    }) {
                        return Err(Error::Limit {
                            span,
                            message: "string length limit exceeded".into(),
                        });
                    }
                    if !a.is_empty() && !b.is_empty() {
                        self.integer_work(span, |budget| {
                            budget.charge(a.len())?;
                            budget.charge(b.len())
                        })?;
                    }
                    return Ok(Value::String(a.concat(&b)));
                }
            }
            Less | LessEqual | Greater | GreaterEqual => {
                left = self.primitive(left, Hint::Number, span)?;
                right = self.primitive(right, Hint::Number, span)?;
                self.comparison_work(&left, &right, span)?;
                let order = self.conversion_work(span, |budget| left.compare(&right, budget))?;
                return Ok(Value::Boolean(order.is_some_and(|order| match op {
                    Less => order.is_lt(),
                    LessEqual => order.is_le(),
                    Greater => order.is_gt(),
                    GreaterEqual => order.is_ge(),
                    _ => unreachable!(),
                })));
            }
            _ => {}
        }
        let left = self.numeric(left, span)?;
        let right = self.numeric(right, span)?;
        if let (Value::BigInt(a), Value::BigInt(b)) = (&left, &right) {
            if op == UnsignedRightShift {
                return Err(Self::exception(
                    ExceptionKind::TypeError,
                    span,
                    "BigInt has no unsigned right shift",
                ));
            }
            return self
                .integer_work(span, |budget| match op {
                    Add => a.add(b, budget),
                    Subtract => a.sub(b, budget),
                    Multiply => a.mul(b, budget),
                    Divide => a.div_rem(b, budget).map(|(quotient, _)| quotient),
                    Remainder => a.div_rem(b, budget).map(|(_, remainder)| remainder),
                    Exponentiate => a.pow(b, budget),
                    BitAnd => a.bitwise(b, BitwiseOp::And, budget),
                    BitOr => a.bitwise(b, BitwiseOp::Or, budget),
                    BitXor => a.bitwise(b, BitwiseOp::Xor, budget),
                    LeftShift => a.shl(b, budget),
                    RightShift => a.shr(b, budget),
                    _ => unreachable!("non-numeric operators returned above"),
                })
                .map(Value::BigInt);
        }
        // ToNumeric preserves BigInt. Arithmetic requires equal numeric types.
        if matches!(left, Value::BigInt(_)) || matches!(right, Value::BigInt(_)) {
            return Err(Self::exception(
                ExceptionKind::TypeError,
                span,
                "cannot mix BigInt and Number operands",
            ));
        }
        let Value::Number(a) = left else {
            unreachable!("numeric types checked above")
        };
        let Value::Number(b) = right else {
            unreachable!("numeric types checked above")
        };
        Ok(Value::Number(match op {
            Add => a + b,
            Subtract => a - b,
            Multiply => a * b,
            Divide => a / b,
            Remainder => a % b,
            Exponentiate => exponentiate(a, b),
            BitAnd => ((to_uint32(a) & to_uint32(b)) as i32) as f64,
            BitOr => ((to_uint32(a) | to_uint32(b)) as i32) as f64,
            BitXor => ((to_uint32(a) ^ to_uint32(b)) as i32) as f64,
            LeftShift => ((to_uint32(a) << (to_uint32(b) & 31)) as i32) as f64,
            RightShift => ((to_uint32(a) as i32) >> (to_uint32(b) & 31)) as f64,
            UnsignedRightShift => (to_uint32(a) >> (to_uint32(b) & 31)) as f64,
            _ => unreachable!("non-numeric operators returned above"),
        }))
    }
}

fn identifier(expr: &Expr) -> Option<&str> {
    match &expr.kind {
        ExprKind::Identifier(name) => Some(name),
        ExprKind::Parenthesized(e) => identifier(e),
        _ => None,
    }
}

pub(crate) fn reference_expression(expr: &Expr) -> bool {
    match &expr.kind {
        ExprKind::Identifier(_)
        | ExprKind::Member(..)
        | ExprKind::SuperProperty(_)
        | ExprKind::OptionalChain { .. } => true,
        ExprKind::Parenthesized(inner) => reference_expression(inner),
        _ => false,
    }
}
