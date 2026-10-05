//! BindingInitialization for declarations and catch clauses (8.6.3, 14.3.3).

use crate::{
    BindingState, Error, ExceptionKind, Realm, Reference, Value, environment::EnvironmentHandle,
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
                        deletable: false,
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
        self.binding_initialization(pattern, value, Some(environment), None)
    }

    pub(super) fn assign_pattern(
        &mut self,
        pattern: &BindingPattern,
        value: Value,
    ) -> Result<(), Error> {
        self.binding_initialization(pattern, value, None, None)
    }

    fn binding_reference<'a>(
        &mut self,
        pattern: &'a BindingPattern,
        environment: Option<&EnvironmentHandle>,
    ) -> Result<Option<Reference<'a>>, Error> {
        if environment.is_none() {
            if let BindingPatternKind::Identifier(name) = &pattern.kind {
                return self.resolve(name, pattern.span).map(Some);
            }
        }
        Ok(None)
    }

    fn binding_initialization<'a>(
        &mut self,
        pattern: &'a BindingPattern,
        value: Value,
        environment: Option<&EnvironmentHandle>,
        reference: Option<Reference<'a>>,
    ) -> Result<(), Error> {
        // Share the evaluator's native-stack guard with reentrant defaults and
        // property/iterator hooks; this introduces no new host quota.
        self.enter_evaluation(pattern.span)?;
        let result = self.initialize_pattern_inner(pattern, value, environment, reference);
        self.evaluation_depth -= 1;
        result
    }

    fn initialize_pattern_inner<'a>(
        &mut self,
        pattern: &'a BindingPattern,
        value: Value,
        environment: Option<&EnvironmentHandle>,
        reference: Option<Reference<'a>>,
    ) -> Result<(), Error> {
        let span = pattern.span;
        self.tick(span)?;
        match &pattern.kind {
            BindingPatternKind::Identifier(name) => {
                if let Some(environment) = environment {
                    self.objects
                        .environment_mut(environment)
                        .expect("binding environment")
                        .bindings
                        .get_mut(name)
                        .expect("instantiated binding")
                        .value = Some(value);
                } else {
                    let reference = match reference {
                        Some(reference) => reference,
                        None => self.resolve(name, span)?,
                    };
                    self.put(reference, value, span)?;
                }
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
                        PropertyName::Private(_) => {
                            return Err(Self::unsupported(
                                span,
                                "private elements are not implemented",
                            ));
                        }
                        PropertyName::Literal(literal) => {
                            self.literal_value(literal, property.element.pattern.span)?
                        }
                        PropertyName::Computed(expression) => self.expression(expression)?,
                    };
                    let key = self.property_key(key_value, span)?;
                    // KeyedBindingInitialization resolves SingleNameBinding before
                    // GetV; with/unscopables hooks can mutate the source or target.
                    let reference =
                        self.binding_reference(&property.element.pattern, environment)?;
                    let next = self.get_property_value(&value, &key, span)?;
                    self.binding_element(&property.element, next, environment, reference)?;
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
                    let reference = self.binding_reference(rest, environment)?;
                    let prototype = self
                        .intrinsics
                        .as_ref()
                        .expect("initialized")
                        .object_prototype
                        .clone();
                    let object =
                        self.object_work(span, |objects, _| objects.create(Some(&prototype)))?;
                    self.copy_data_properties(&object, value, &excluded, span)?;
                    self.binding_initialization(
                        rest,
                        Value::Object(object),
                        environment,
                        reference,
                    )?;
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
                        // Resolve before stepping even when the iterator is done.
                        let reference = self.binding_reference(&element.pattern, environment)?;
                        let next = if iterator.is_done() {
                            Value::Undefined
                        } else {
                            self.iterator_step_value(&mut iterator, span)?
                                .unwrap_or(Value::Undefined)
                        };
                        self.binding_element(element, next, environment, reference)?;
                    }
                    if let Some(rest) = rest {
                        let reference = self.binding_reference(rest, environment)?;
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
                        self.binding_initialization(
                            rest,
                            Value::Object(array),
                            environment,
                            reference,
                        )?;
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
        value: Value,
        environment: &EnvironmentHandle,
    ) -> Result<(), Error> {
        self.binding_element(element, value, Some(environment), None)
    }

    fn binding_element<'a>(
        &mut self,
        element: &'a BindingElement,
        mut value: Value,
        environment: Option<&EnvironmentHandle>,
        reference: Option<Reference<'a>>,
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
        self.binding_initialization(&element.pattern, value, environment, reference)
    }
}
