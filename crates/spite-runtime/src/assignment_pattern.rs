//! Ordered DestructuringAssignmentEvaluation (13.15.5.2–6).

use crate::{Error, ExceptionKind, Realm, Reference, Value};
use spite_core::WellKnownSymbol;
use spite_parser::ast::{
    AssignmentElement, AssignmentPattern, AssignmentPatternKind, AssignmentTarget, ExprKind,
    PropertyName,
};
use std::collections::HashSet;

impl Realm {
    pub(super) fn destructuring_assignment(
        &mut self,
        pattern: &AssignmentPattern,
        value: Value,
    ) -> Result<(), Error> {
        self.enter_evaluation(pattern.span)?;
        let result = self.destructuring_assignment_inner(pattern, value);
        self.evaluation_depth -= 1;
        result
    }

    fn assignment_target_reference<'a>(
        &mut self,
        target: &'a AssignmentTarget,
    ) -> Result<Option<Reference<'a>>, Error> {
        match target {
            AssignmentTarget::Reference(expression) => self.reference(expression).map(Some),
            AssignmentTarget::Pattern(_) => Ok(None),
        }
    }

    fn destructuring_assignment_inner(
        &mut self,
        pattern: &AssignmentPattern,
        value: Value,
    ) -> Result<(), Error> {
        let span = pattern.span;
        self.tick(span)?;
        match &pattern.kind {
            AssignmentPatternKind::Object { properties, rest } => {
                Self::require_object_coercible(&value, span)?;
                // PropertyKey hashes immutable strings or Symbol identity, never Symbol liveness.
                #[allow(clippy::mutable_key_type)]
                let mut excluded = HashSet::new();
                if rest.is_some() {
                    excluded
                        .try_reserve(properties.len())
                        .map_err(|_| Error::Limit {
                            span,
                            message: "assignment property capacity exceeded".into(),
                        })?;
                }
                for property in properties {
                    let key = match &property.key {
                        PropertyName::Private(_) => {
                            return Err(Self::unsupported(
                                span,
                                "private elements are not implemented",
                            ));
                        }
                        PropertyName::Literal(literal) => self.literal_value(literal, span)?,
                        PropertyName::Computed(expression) => self.expression(expression)?,
                    };
                    let key = self.property_key(key, span)?;
                    // Evaluate a simple target reference before GetV. Edition 17
                    // retains a computed target's raw key until PutValue.
                    let reference = self.assignment_target_reference(&property.element.target)?;
                    let next = self.get_property_value(&value, &key, span)?;
                    self.assignment_element_value(&property.element, next, reference)?;
                    if rest.is_some() {
                        self.exclude_rest_key(&mut excluded, key, span)?;
                    }
                }
                if let Some(rest) = rest {
                    let reference = self.reference(rest)?;
                    let value = self.copy_rest_object(value, &excluded, span)?;
                    self.put(reference, value, rest.span)?;
                }
            }
            AssignmentPatternKind::Array { elements, rest } => {
                let method = self
                    .get_method(&value, &WellKnownSymbol::Iterator.symbol(), span)?
                    .ok_or_else(|| {
                        Self::exception(
                            ExceptionKind::TypeError,
                            span,
                            "assignment value is not iterable",
                        )
                    })?;
                let mut iterator = self.get_iterator_from_method(value, method, span)?;
                let result = (|| {
                    for element in elements {
                        self.tick(span)?;
                        let Some(element) = element else {
                            if !iterator.is_done() {
                                self.iterator_skip_value(&mut iterator, span)?;
                            }
                            continue;
                        };
                        // Target evaluation precedes IteratorStepValue, including
                        // when an earlier step has already exhausted the iterator.
                        let reference = self.assignment_target_reference(&element.target)?;
                        let next = if iterator.is_done() {
                            Value::Undefined
                        } else {
                            self.iterator_step_value(&mut iterator, span)?
                                .unwrap_or(Value::Undefined)
                        };
                        self.assignment_element_value(element, next, reference)?;
                    }
                    if let Some(rest) = rest {
                        let reference = self.assignment_target_reference(rest)?;
                        let value = self.iterator_rest_array(
                            &mut iterator,
                            span,
                            "assignment array capacity exceeded",
                        )?;
                        self.assign_destructured_target(rest, value, reference)?;
                    }
                    Ok(())
                })();
                if !iterator.is_done() {
                    match result {
                        Ok(()) => self.iterator_close(&iterator, span)?,
                        Err(error) => return Err(self.iterator_close_error(&iterator, error, span)),
                    }
                } else {
                    result?;
                }
            }
        }
        Ok(())
    }

    fn assignment_element_value<'a>(
        &mut self,
        element: &'a AssignmentElement,
        mut value: Value,
        reference: Option<Reference<'a>>,
    ) -> Result<(), Error> {
        if matches!(value, Value::Undefined)
            && let Some(initializer) = &element.initializer
        {
            value = if let AssignmentTarget::Reference(target) = &element.target {
                self.assignment_expression(target, initializer)?
            } else {
                self.expression(initializer)?
            };
        }
        self.assign_destructured_target(&element.target, value, reference)
    }

    fn assign_destructured_target<'a>(
        &mut self,
        target: &'a AssignmentTarget,
        value: Value,
        reference: Option<Reference<'a>>,
    ) -> Result<(), Error> {
        match target {
            AssignmentTarget::Pattern(pattern) => self.destructuring_assignment(pattern, value),
            AssignmentTarget::Reference(expression) => {
                debug_assert!(!matches!(expression.kind, ExprKind::OptionalChain { .. }));
                self.put(
                    reference.expect("simple assignment reference evaluated before source access"),
                    value,
                    expression.span,
                )
            }
        }
    }
}
