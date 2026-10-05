//! BindingInitialization for declarative identifiers and patterns (8.6.3, 14.3.3).

use crate::{
    BindingState, Error, ExceptionKind, Realm, Value, environment::EnvironmentHandle,
    object::DataDescriptor,
};
use spite_core::{JsString, PropertyKey, WellKnownSymbol};
use spite_parser::ast::{BindingElement, BindingPattern, BindingPatternKind, PropertyName};
use std::collections::{BTreeMap, HashSet};

impl Realm {
    pub(super) fn pattern_bindings<'a>(
        &mut self,
        patterns: impl IntoIterator<Item = &'a BindingPattern>,
        mutable: bool,
    ) -> Result<BTreeMap<String, BindingState>, Error> {
        let mut bindings = BTreeMap::new();
        for pattern in patterns {
            for (name, span) in pattern.bound_names() {
                self.object_work(span, |_, budget| budget.charge(name.len() + 1))?;
                bindings.insert(
                    name.to_owned(),
                    BindingState {
                        value: None,
                        mutable,
                        strict: true,
                    },
                );
            }
        }
        Ok(bindings)
    }

    pub(super) fn initialize_pattern(
        &mut self,
        pattern: &BindingPattern,
        value: Value,
        environment: &EnvironmentHandle,
    ) -> Result<(), Error> {
        // Share the evaluator's native-stack guard with reentrant defaults and
        // property/iterator hooks; this introduces no new host quota.
        self.enter_evaluation(pattern.span)?;
        let result = self.initialize_pattern_inner(pattern, value, environment);
        self.evaluation_depth -= 1;
        result
    }

    fn initialize_pattern_inner(
        &mut self,
        pattern: &BindingPattern,
        value: Value,
        environment: &EnvironmentHandle,
    ) -> Result<(), Error> {
        let span = pattern.span;
        self.tick(span)?;
        match &pattern.kind {
            BindingPatternKind::Identifier(name) => {
                self.objects
                    .environment_mut(environment)
                    .expect("binding environment")
                    .bindings
                    .get_mut(name)
                    .expect("instantiated binding")
                    .value = Some(value);
            }
            BindingPatternKind::Object { properties, rest } => {
                // GetV preserves the original primitive receiver. Empty object
                // patterns still reject null/undefined without coercing objects.
                Self::require_object_coercible(&value, span)?;
                let mut excluded = HashSet::new();
                if rest.is_some() {
                    excluded
                        .try_reserve(properties.len())
                        .map_err(|_| Error::Limit {
                            span,
                            message: "binding property capacity exceeded".into(),
                        })?;
                }
                for property in properties {
                    let key_value = match &property.key {
                        PropertyName::Literal(literal) => {
                            self.literal_value(literal, property.element.pattern.span)?
                        }
                        PropertyName::Computed(expression) => self.expression(expression)?,
                    };
                    let key = self.property_key(key_value, span)?;
                    let next = self.get_property_value(&value, &key, span)?;
                    self.initialize_binding_element(&property.element, next, environment)?;
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
                    let prototype = self
                        .intrinsics
                        .as_ref()
                        .expect("initialized")
                        .object_prototype
                        .clone();
                    let object =
                        self.object_work(span, |objects, _| objects.create(Some(&prototype)))?;
                    self.copy_data_properties(&object, value, &excluded, span)?;
                    self.initialize_pattern(rest, Value::Object(object), environment)?;
                }
            }
            BindingPatternKind::Array { elements, rest } => {
                let method = self
                    .get_method(&value, &WellKnownSymbol::Iterator.symbol(), span)?
                    .ok_or_else(|| {
                        Self::exception(
                            ExceptionKind::TypeError,
                            span,
                            "binding value is not iterable",
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
                        let next = if iterator.is_done() {
                            Value::Undefined
                        } else {
                            self.iterator_step_value(&mut iterator, span)?
                                .unwrap_or(Value::Undefined)
                        };
                        self.initialize_binding_element(element, next, environment)?;
                    }
                    if let Some(rest) = rest {
                        let Value::Object(array) = self.create_array_from_list([], span)? else {
                            unreachable!("intrinsic Array creation");
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
                                message: "binding array capacity exceeded".into(),
                            })?;
                        }
                        self.initialize_pattern(rest, Value::Object(array), environment)?;
                    }
                    Ok(())
                })();
                // IteratorStep/Value failures set Done; default or nested-binding
                // throws close a still-active iterator and preserve throw identity.
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

    pub(super) fn initialize_binding_element(
        &mut self,
        element: &BindingElement,
        mut value: Value,
        environment: &EnvironmentHandle,
    ) -> Result<(), Error> {
        if matches!(value, Value::Undefined) {
            if let Some(initializer) = &element.initializer {
                value = if let BindingPatternKind::Identifier(name) = &element.pattern.kind {
                    self.named_expression(initializer, JsString::from(name.as_str()))?
                } else {
                    self.expression(initializer)?
                };
            }
        }
        self.initialize_pattern(&element.pattern, value, environment)
    }
}
