//! Arguments objects and their intrinsic Array values iterator (10.4.4.6–7).

use crate::{
    Error, ObjectHandle, Realm, Value,
    environment::EnvironmentHandle,
    object::{DataDescriptor, DescriptorKind, PropertyDescriptor},
};
use spite_core::{JsString, Span, WellKnownSymbol};
use spite_parser::ast::{BindingPatternKind, Parameter};
use std::collections::{BTreeMap, BTreeSet};

impl Realm {
    pub(super) fn unmapped_arguments(
        &mut self,
        arguments: &[Value],
        span: Span,
    ) -> Result<Value, Error> {
        self.arguments_object(arguments, None, span)
    }

    pub(super) fn mapped_arguments(
        &mut self,
        parameters: &[Parameter],
        arguments: &[Value],
        callee: ObjectHandle,
        environment: EnvironmentHandle,
        span: Span,
    ) -> Result<Value, Error> {
        let object = self.arguments_object(arguments, Some(callee), span)?;
        let mut mapped = BTreeSet::new();
        let mut names = BTreeMap::new();
        for (index, parameter) in parameters.iter().enumerate().rev() {
            debug_assert!(
                parameter.is_simple(),
                "only simple lists have mapped arguments"
            );
            let parameter = parameter.binding();
            let BindingPatternKind::Identifier(name) = &parameter.pattern.kind else {
                unreachable!("mapped arguments require identifier parameters")
            };
            self.object_work(parameter.pattern.span, |_, budget| {
                budget.charge(name.len() + 1)
            })?;
            // A later duplicate suppresses every earlier occurrence, even when
            // the later parameter did not receive an argument (10.4.4.7).
            if mapped.insert(name.as_str()) && index < arguments.len() {
                let index = u32::try_from(index).map_err(|_| Error::Limit {
                    span,
                    message: "parameter index limit exceeded".into(),
                })?;
                names.insert(index, name.clone());
            }
        }
        let Value::Object(handle) = &object else {
            unreachable!("arguments object")
        };
        self.object_work(span, |objects, budget| {
            objects.map_arguments(handle, environment, names, budget)
        })?;
        Ok(object)
    }

    fn arguments_object(
        &mut self,
        arguments: &[Value],
        callee: Option<ObjectHandle>,
        span: Span,
    ) -> Result<Value, Error> {
        let intrinsics = self
            .intrinsics
            .as_ref()
            .expect("function intrinsics initialized");
        let prototype = intrinsics.object_prototype.clone();
        let thrower = intrinsics.throw_type_error.clone();
        let values = intrinsics.array.values.clone();
        let object = self.object_work(span, |objects, _| objects.create_arguments(&prototype))?;
        self.define_builtin_property(
            &object,
            "length",
            Value::Number(arguments.len() as f64),
            true,
            span,
        )?;
        for (index, value) in arguments.iter().enumerate() {
            self.object_work(span, |objects, budget| {
                budget.value(value)?;
                objects.define(
                    &object,
                    JsString::from(index.to_string().as_str()),
                    DataDescriptor {
                        value: Some(value.clone()),
                        writable: Some(true),
                        enumerable: Some(true),
                        configurable: Some(true),
                    },
                    budget,
                )
            })?;
        }
        self.object_work(span, |objects, budget| {
            objects.define(
                &object,
                WellKnownSymbol::Iterator.symbol(),
                DataDescriptor {
                    value: Some(Value::Object(values)),
                    writable: Some(true),
                    enumerable: Some(false),
                    configurable: Some(true),
                },
                budget,
            )
        })?;
        if let Some(callee) = callee {
            self.define_builtin_property(&object, "callee", Value::Object(callee), true, span)?;
        } else {
            self.object_work(span, |objects, budget| {
                objects.define(
                    &object,
                    JsString::from("callee"),
                    PropertyDescriptor {
                        kind: DescriptorKind::Accessor {
                            get: Some(Some(thrower.clone())),
                            set: Some(Some(thrower)),
                        },
                        enumerable: Some(false),
                        configurable: Some(false),
                    },
                    budget,
                )
            })?;
        }
        Ok(Value::Object(object))
    }
}
