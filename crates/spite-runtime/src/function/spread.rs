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
            match argument {
                Argument::Expression(_) => {
                    self.append_argument(&mut values, value, expression.span)?
                }
                Argument::Spread(_) => {
                    let method = self
                        .get_method(&value, &WellKnownSymbol::Iterator.symbol(), expression.span)?
                        .ok_or_else(|| {
                            Self::exception(
                                ExceptionKind::TypeError,
                                expression.span,
                                "spread value is not iterable",
                            )
                        })?;
                    let mut iterator =
                        self.get_iterator_from_method(value, method, expression.span)?;
                    loop {
                        self.tick(expression.span)?;
                        let Some(value) =
                            self.iterator_step_value(&mut iterator, expression.span)?
                        else {
                            break;
                        };
                        // ArgumentListEvaluation propagates iterator failures;
                        // an opted-in host/capacity failure does not run return.
                        self.append_argument(&mut values, value, expression.span)?;
                    }
                }
            }
        }
        Ok(values)
    }

    fn append_argument(
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
