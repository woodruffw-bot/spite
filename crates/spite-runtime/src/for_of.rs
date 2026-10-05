//! Synchronous ForIn/OfHeadEvaluation and ForIn/OfBodyEvaluation (14.7.5.6–7).

use crate::{Completion, Error, ExceptionKind, Realm, Value};
use spite_core::{Span, WellKnownSymbol};
use spite_parser::ast::{Expr, ForBinding, Statement};

impl Realm {
    pub(super) fn for_of(
        &mut self,
        binding: &ForBinding,
        iterable: &Expr,
        body: &Statement,
        labels: &[&str],
        span: Span,
    ) -> Result<Completion, Error> {
        let source = self.iteration_source(binding, iterable, span)?;
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
            let result = self.iteration_body(binding, next, body, span);
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

    pub(super) fn iteration_source(
        &mut self,
        binding: &ForBinding,
        iterable: &Expr,
        span: Span,
    ) -> Result<Value, Error> {
        // Lexical names shadow outer bindings in the RHS but remain uninitialized.
        // Restore the outer environment before GetIterator, including on failure.
        if let ForBinding::Lexical { mutable, binding } = binding {
            let bindings = self.pattern_bindings([binding], *mutable)?;
            self.push_scope(bindings, span)?;
            let result = self.expression(iterable);
            self.scopes.pop();
            result
        } else {
            self.expression(iterable)
        }
    }

    pub(super) fn iteration_body(
        &mut self,
        binding: &ForBinding,
        value: Value,
        body: &Statement,
        span: Span,
    ) -> Result<Completion, Error> {
        match binding {
            ForBinding::Lexical { mutable, binding } => {
                // Both let and const receive a new environment each iteration.
                let bindings = self.pattern_bindings([binding], *mutable)?;
                self.push_scope(bindings, span)?;
                let environment = self.scopes.last().expect("iteration environment").clone();
                let result = self
                    .initialize_pattern(binding, value, &environment)
                    .and_then(|()| self.statement(body));
                self.scopes.pop();
                result
            }
            ForBinding::Var(binding) => self
                .resolve(&binding.name, binding.span)
                .and_then(|reference| self.put(reference, value, binding.span))
                .and_then(|()| self.statement(body)),
            ForBinding::Assignment(target) => self
                .reference(target)
                .and_then(|reference| self.put(reference, value, target.span))
                .and_then(|()| self.statement(body)),
        }
    }
}
