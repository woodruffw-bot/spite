//! Budgeted value conversion, comparison and string operations.

use crate::realm_object::Hint;
use crate::{ConversionError, Error, ExceptionKind, Realm, Value};
use spite_bigint::{Budget, Error as IntegerError};
use spite_core::{JsString, Span};

impl Realm {
    pub(crate) fn exception(kind: ExceptionKind, span: Span, message: impl Into<String>) -> Error {
        Error::Exception {
            kind,
            span,
            message: message.into(),
        }
    }
    pub(crate) fn unsupported(span: Span, message: impl Into<String>) -> Error {
        Error::Unsupported {
            span,
            message: message.into(),
        }
    }
    pub(crate) fn tick(&mut self, span: Span) -> Result<(), Error> {
        if self
            .atomic_waits
            .as_ref()
            .is_some_and(|waits| waits.owner().is_terminated())
        {
            return Err(self.agent_termination(span));
        }
        if let Some(remaining) = &mut self.remaining_steps {
            *remaining = remaining.checked_sub(1).ok_or_else(|| Error::Limit {
                span,
                message: "evaluation step limit exceeded".into(),
            })?;
        }
        Ok(())
    }

    pub(crate) fn integer_work<T>(
        &mut self,
        span: Span,
        work: impl FnOnce(&mut Budget) -> Result<T, IntegerError>,
    ) -> Result<T, Error> {
        let mut budget = Budget::with_limits(self.limits.max_bigint_bits, self.remaining_steps);
        let result = work(&mut budget);
        self.remaining_steps = budget.remaining_work();
        result.map_err(|error| Self::integer_error(error, span))
    }

    pub(crate) fn integer_error(error: IntegerError, span: Span) -> Error {
        match error {
            IntegerError::Limit => Error::Limit {
                span,
                message: "integer resource limit exceeded".into(),
            },
            IntegerError::DivisionByZero | IntegerError::NegativeExponent => {
                Self::exception(ExceptionKind::RangeError, span, error.to_string())
            }
            IntegerError::InvalidDigit | IntegerError::InvalidRadix => {
                unreachable!("integer input was validated before conversion")
            }
        }
    }

    pub(crate) fn conversion_work<T>(
        &mut self,
        span: Span,
        work: impl FnOnce(&mut Budget) -> Result<T, ConversionError>,
    ) -> Result<T, Error> {
        let mut budget = Budget::with_limits(self.limits.max_bigint_bits, self.remaining_steps);
        let result = work(&mut budget);
        self.remaining_steps = budget.remaining_work();
        result.map_err(|error| Self::conversion_error(error, span))
    }

    pub(crate) fn conversion_error(error: ConversionError, span: Span) -> Error {
        match error {
            ConversionError::BigIntToNumber => Self::exception(
                ExceptionKind::TypeError,
                span,
                "cannot convert BigInt to Number",
            ),
            ConversionError::SymbolToNumber => Self::exception(
                ExceptionKind::TypeError,
                span,
                "cannot convert Symbol to Number",
            ),
            ConversionError::SymbolToString => Self::exception(
                ExceptionKind::TypeError,
                span,
                "cannot convert Symbol to String",
            ),
            ConversionError::ObjectNeedsContext => Self::unsupported(
                span,
                "object conversion requires ToPrimitive and callable hooks",
            ),
            ConversionError::Integer(error) => Self::integer_error(error, span),
        }
    }

    pub(crate) fn number(&mut self, value: Value, span: Span) -> Result<f64, Error> {
        let value = self.primitive(value, Hint::Number, span)?;
        self.conversion_work(span, |budget| value.to_number_with_budget(budget))
    }

    pub(crate) fn numeric(&mut self, value: Value, span: Span) -> Result<Value, Error> {
        match self.primitive(value, Hint::Number, span)? {
            value @ (Value::BigInt(_) | Value::Number(_)) => Ok(value),
            other => Ok(Value::Number(self.number(other, span)?)),
        }
    }

    pub(crate) fn comparison_work(
        &mut self,
        left: &Value,
        right: &Value,
        span: Span,
    ) -> Result<(), Error> {
        let words = [left, right]
            .into_iter()
            .map(|value| match value {
                Value::BigInt(value) => value.bit_length().div_ceil(32),
                _ => 0,
            })
            .max()
            .unwrap_or(0);
        self.integer_work(span, |budget| budget.charge(words))
    }

    pub(crate) fn strictly_equal(
        &mut self,
        left: &Value,
        right: &Value,
        span: Span,
    ) -> Result<bool, Error> {
        self.comparison_work(left, right, span)?;
        self.integer_work(span, |budget| {
            left.strictly_equal_with_budget(right, budget)
        })
    }
    pub(crate) fn check_string(&self, value: &Value, span: Span) -> Result<(), Error> {
        if let Value::String(s) = value
            && self
                .limits
                .max_string_units
                .is_some_and(|limit| s.len() > limit)
        {
            return Err(Error::Limit {
                span,
                message: "string length limit exceeded".into(),
            });
        }
        Ok(())
    }

    pub(crate) fn append_budgeted_string(
        &mut self,
        units: &mut Vec<u16>,
        part: &JsString,
        span: Span,
    ) -> Result<(), Error> {
        let length = units
            .len()
            .checked_add(part.len())
            .ok_or_else(|| Error::Limit {
                span,
                message: "string length exceeds platform capacity".into(),
            })?;
        if self
            .limits
            .max_string_units
            .is_some_and(|limit| length > limit)
        {
            return Err(Error::Limit {
                span,
                message: "string length limit exceeded".into(),
            });
        }
        let movement = if length > units.capacity() {
            units.len()
        } else {
            0
        };
        self.object_work(span, |_, budget| {
            budget.charge(movement)?;
            budget.charge(part.len())
        })?;
        units.try_reserve(part.len()).map_err(|_| Error::Limit {
            span,
            message: "string storage exceeds platform capacity".into(),
        })?;
        self.append_string(units, part, span)
    }

    pub(crate) fn append_string(
        &self,
        units: &mut Vec<u16>,
        part: &JsString,
        span: Span,
    ) -> Result<(), Error> {
        if units.len().checked_add(part.len()).is_none_or(|len| {
            self.limits
                .max_string_units
                .is_some_and(|limit| len > limit)
        }) {
            return Err(Error::Limit {
                span,
                message: "string length limit exceeded".into(),
            });
        }
        units.try_reserve(part.len()).map_err(|_| Error::Limit {
            span,
            message: "string allocation failed".into(),
        })?;
        units.extend_from_slice(part.code_units());
        Ok(())
    }
}
