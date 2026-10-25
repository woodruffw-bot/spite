//! ArgumentListEvaluation with iterable spread and checked storage (13.3.8.1).

use crate::{Error, ExceptionKind, Realm, Value};
use spite_core::{Span, WellKnownSymbol};
use spite_parser::ast::Argument;

impl Realm {
    pub(crate) fn argument_list(&mut self, arguments: &[Argument]) -> Result<Vec<Value>, Error> {
        let mut values = Vec::new();
        for argument in arguments {
            let expression = argument.expression();
            let value = self.expression(expression)?;
            self.append_evaluated_argument(
                &mut values,
                value,
                matches!(argument, Argument::Spread(_)),
                expression.span,
            )?;
        }
        Ok(values)
    }

    pub(crate) fn append_evaluated_argument(
        &mut self,
        values: &mut Vec<Value>,
        value: Value,
        spread: bool,
        span: Span,
    ) -> Result<(), Error> {
        if !spread {
            return self.append_argument(values, value, span);
        }
        let method = self
            .get_method(&value, &WellKnownSymbol::Iterator.symbol(), span)?
            .ok_or_else(|| {
                Self::exception(
                    ExceptionKind::TypeError,
                    span,
                    "spread value is not iterable",
                )
            })?;
        let mut iterator = self.get_iterator_from_method(value, method, span)?;
        loop {
            self.tick(span)?;
            let Some(value) = self.iterator_step_value(&mut iterator, span)? else {
                break;
            };
            // ArgumentListEvaluation never closes for iterator or host failures.
            self.append_argument(values, value, span)?;
        }
        Ok(())
    }

    pub(super) fn append_argument(
        &self,
        arguments: &mut Vec<Value>,
        value: Value,
        span: Span,
    ) -> Result<(), Error> {
        let capacity_error = || Error::Limit {
            span,
            message: "argument list exceeds platform capacity".into(),
        };
        let count = arguments.len().checked_add(1).ok_or_else(capacity_error)?;
        self.check_argument_count(count, span)?;
        arguments.try_reserve(1).map_err(|_| capacity_error())?;
        arguments.push(value);
        Ok(())
    }
}
