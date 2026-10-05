//! Ordered DestructuringAssignmentEvaluation (13.15.5.2–6).

use crate::{Error, ExceptionKind, Realm, Reference, Value, object::DataDescriptor};
use spite_core::{JsString, PropertyKey, WellKnownSymbol};
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
                        self.object_work(span, |_, budget| {
                            budget.charge(match &key {
                                PropertyKey::String(key) => key.len() + 1,
                                PropertyKey::Symbol(_) => 1,
                            })
                        })?;
                        excluded.insert(key);
                    }
                }
                if let Some(rest) = rest {
                    let reference = self.reference(rest)?;
                    let prototype = self
                        .intrinsics
                        .as_ref()
                        .expect("initialized")
                        .object_prototype
                        .clone();
                    let object =
                        self.object_work(span, |objects, _| objects.create(Some(&prototype)))?;
                    self.copy_data_properties(&object, value, &excluded, span)?;
                    self.put(reference, Value::Object(object), rest.span)?;
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
                        let Value::Object(array) = self.create_array_from_list([], span)? else {
                            unreachable!("intrinsic Array creation")
                        };
                        let mut index = 0usize;
                        while !iterator.is_done() {
                            let Some(next) = self.iterator_step_value(&mut iterator, span)? else {
                                break;
                            };
                            self.define_property_or_throw(
                                &array,
                                JsString::from(index.to_string().as_str()),
                                DataDescriptor {
                                    value: Some(next),
                                    writable: Some(true),
                                    enumerable: Some(true),
                                    configurable: Some(true),
                                }
                                .into(),
                                span,
                            )?;
                            index = index.checked_add(1).ok_or_else(|| Error::Limit {
                                span,
                                message: "assignment array capacity exceeded".into(),
                            })?;
                        }
                        self.assign_destructured_target(rest, Value::Object(array), reference)?;
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
        if matches!(value, Value::Undefined) {
            if let Some(initializer) = &element.initializer {
                value = if let AssignmentTarget::Reference(target) = &element.target {
                    self.assignment_expression(target, initializer)?
                } else {
                    self.expression(initializer)?
                };
            }
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
