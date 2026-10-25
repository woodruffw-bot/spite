//! Synchronous ForIn/OfHeadEvaluation and ForIn/OfBodyEvaluation (14.7.5.6–7).

use crate::{Completion, Error, ExceptionKind, Realm, Value, environment::EnvironmentHandle};
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
        let environment = self.prepare_iteration_head_scope(binding, span)?;
        let result = self.expression(iterable);
        if environment.is_some() {
            self.scopes.pop();
        }
        result
    }

    pub(super) fn prepare_iteration_head_scope(
        &mut self,
        binding: &ForBinding,
        span: Span,
    ) -> Result<Option<EnvironmentHandle>, Error> {
        // The lexical RHS environment is restored before iterator acquisition.
        let lexical = match binding {
            ForBinding::Lexical { mutable, binding } => Some((binding, *mutable)),
            ForBinding::Using(binding) => Some((binding, false)),
            _ => None,
        };
        if let Some((binding, mutable)) = lexical {
            let bindings = self.pattern_bindings([binding], mutable)?;
            self.push_scope(bindings, span)?;
            Ok(Some(
                self.scopes
                    .last()
                    .expect("iteration head environment")
                    .clone(),
            ))
        } else {
            Ok(None)
        }
    }

    pub(super) fn iteration_body(
        &mut self,
        binding: &ForBinding,
        value: Value,
        body: &Statement,
        span: Span,
    ) -> Result<Completion, Error> {
        let scope = self.prepare_iteration_scope(binding, span)?;
        let result = self
            .initialize_iteration_binding(
                binding,
                value,
                scope.as_ref().map(|(environment, _)| environment),
            )
            .and_then(|()| self.statement(body));
        if let Some((environment, disposable)) = scope {
            let result = if disposable {
                self.dispose_environment_resources(&environment, result, span)
            } else {
                result
            };
            self.scopes.pop();
            result
        } else {
            result
        }
    }

    pub(super) fn prepare_iteration_scope(
        &mut self,
        binding: &ForBinding,
        span: Span,
    ) -> Result<Option<(EnvironmentHandle, bool)>, Error> {
        let lexical = match binding {
            ForBinding::Using(pattern) => Some((pattern, false, true)),
            ForBinding::Lexical { mutable, binding } => Some((binding, *mutable, false)),
            _ => None,
        };
        if let Some((pattern, mutable, disposable)) = lexical {
            let bindings = self.pattern_bindings([pattern], mutable)?;
            self.push_scope(bindings, span)?;
            Ok(Some((
                self.scopes.last().expect("iteration environment").clone(),
                disposable,
            )))
        } else {
            Ok(None)
        }
    }

    pub(super) fn initialize_iteration_binding(
        &mut self,
        binding: &ForBinding,
        value: Value,
        environment: Option<&EnvironmentHandle>,
    ) -> Result<(), Error> {
        match binding {
            ForBinding::Using(pattern) => self.initialize_using_value(
                pattern,
                value,
                environment.expect("using iteration environment"),
            ),
            ForBinding::Lexical { binding, .. } => self.initialize_pattern(
                binding,
                value,
                environment.expect("lexical iteration environment"),
            ),
            ForBinding::Var(binding) => self.assign_pattern(&binding.pattern, value),
            ForBinding::Assignment(target) => self
                .reference(target)
                .and_then(|reference| self.put(reference, value, target.span)),
            ForBinding::Pattern(pattern) => self.destructuring_assignment(pattern, value),
        }
    }
}
