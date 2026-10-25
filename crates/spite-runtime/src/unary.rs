//! Unary value operations shared by synchronous and resumed expression evaluation.

use crate::{Error, Realm, Value, value::to_uint32};
use spite_core::{JsString, Span};
use spite_parser::ast::UnaryOp;

impl Realm {
    pub(crate) fn unary_value(
        &mut self,
        op: UnaryOp,
        value: Value,
        span: Span,
    ) -> Result<Value, Error> {
        Ok(match op {
            UnaryOp::Plus => Value::Number(self.number(value, span)?),
            UnaryOp::Minus => match self.numeric(value, span)? {
                Value::BigInt(value) => {
                    Value::BigInt(self.integer_work(span, |budget| value.neg(budget))?)
                }
                Value::Number(value) => Value::Number(-value),
                _ => unreachable!("ToNumeric returns a numeric value"),
            },
            UnaryOp::Not => Value::Boolean(!value.to_boolean()),
            UnaryOp::BitNot => match self.numeric(value, span)? {
                Value::BigInt(value) => {
                    Value::BigInt(self.integer_work(span, |budget| value.bitnot(budget))?)
                }
                Value::Number(value) => Value::Number((!(to_uint32(value) as i32)) as f64),
                _ => unreachable!("ToNumeric returns a numeric value"),
            },
            UnaryOp::Void => Value::Undefined,
            UnaryOp::Typeof => Value::String(JsString::from(if self.is_callable(&value, span)? {
                "function"
            } else {
                value.type_name()
            })),
            UnaryOp::Delete => Value::Boolean(true),
        })
    }
}
