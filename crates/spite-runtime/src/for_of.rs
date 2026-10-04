//! Synchronous ForIn/OfHeadEvaluation and ForIn/OfBodyEvaluation (14.7.5.6–7).

use crate::{BindingState, Completion, Error, ExceptionKind, Realm, Value};
use spite_core::{Span, WellKnownSymbol};
use spite_parser::ast::{Expr, ForOfBinding, Statement};
use std::collections::BTreeMap;

impl Realm {
    pub(super) fn for_of(
        &mut self,
        binding: &ForOfBinding,
        iterable: &Expr,
        body: &Statement,
        labels: &[&str],
        span: Span,
    ) -> Result<Completion, Error> {
        // Lexical names shadow outer bindings in the RHS but remain uninitialized.
        // Restore the outer environment before GetIterator, including on failure.
        let source = if let ForOfBinding::Lexical { mutable, binding } = binding {
            self.push_scope(
                BTreeMap::from([(
                    binding.name.clone(),
                    BindingState {
                        value: None,
                        mutable: *mutable,
                        strict: true,
                    },
                )]),
                span,
            )?;
            let result = self.expression(iterable);
            self.scopes.pop();
            result?
        } else {
            self.expression(iterable)?
        };
        let method = self
            .get_method(&source, &WellKnownSymbol::Iterator.symbol(), span)?
            .ok_or_else(|| {
                Self::exception(
                    ExceptionKind::TypeError,
                    span,
                    "for-of value is not iterable",
                )
            })?;
        let mut iterator = self.get_iterator_from_method(source, method, span)?;
        let mut value = Value::Undefined;
        loop {
            self.tick(span)?;
            // Step/done/value failures do not run return (14.7.5.7).
            let Some(next) = self.iterator_step_value(&mut iterator, span)? else {
                return Ok(Completion::normal(Some(value)));
            };
            let result = match binding {
                ForOfBinding::Lexical { mutable, binding } => {
                    // Both let and const receive a new environment each iteration.
                    self.push_scope(
                        BTreeMap::from([(
                            binding.name.clone(),
                            BindingState {
                                value: Some(next),
                                mutable: *mutable,
                                strict: true,
                            },
                        )]),
                        span,
                    )?;
                    let result = self.statement(body);
                    self.scopes.pop();
                    result
                }
                ForOfBinding::Var(binding) => self
                    .resolve(&binding.name, binding.span)
                    .and_then(|reference| self.put(reference, next, binding.span))
                    .and_then(|()| self.statement(body)),
                ForOfBinding::Assignment(target) => self
                    .reference(target)
                    .and_then(|reference| self.put(reference, next, target.span))
                    .and_then(|()| self.statement(body)),
            };
            let result = match result {
                Ok(result) => result.update_empty(Some(value)),
                Err(error) => return Err(self.iterator_close_error(&iterator, error, span)),
            };
            if !result.loop_continues(labels) {
                self.iterator_close(&iterator, span)?;
                return Ok(result.consume_unlabelled_break());
            }
            value = result.value.expect("loop UpdateEmpty supplies a value");
        }
    }
}
