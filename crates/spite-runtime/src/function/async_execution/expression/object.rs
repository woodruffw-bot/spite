//! Suspended PropertyDefinitionEvaluation with shared literal operations (13.2.5.5).

use super::{AsyncExecutionData, child_path, expression_at, has_supported_await, push};
use crate::{
    Error, ObjectHandle, Realm, Value,
    object::async_expression::{AsyncExpressionData, ExprChild, ExprPath, ExprTask},
};
use spite_core::PropertyKey;
use spite_parser::ast::{ExprKind, ObjectElement, ObjectProperty, PropertyKind, PropertyName};

pub(super) fn handle(value: &Value) -> &ObjectHandle {
    let Value::Object(object) = value else {
        unreachable!("partial object literal")
    };
    object
}

pub(super) fn property<'a>(
    data: &'a AsyncExecutionData,
    state: &AsyncExpressionData,
    path: &ExprPath,
    element: usize,
) -> &'a ObjectProperty {
    let ExprKind::Object(elements) = &expression_at(data, state.root, path).kind else {
        unreachable!("object source path")
    };
    let ObjectElement::Property(property) = &elements[element] else {
        unreachable!("property source path")
    };
    property
}

fn key_value(key: PropertyKey) -> Value {
    match key {
        PropertyKey::String(key) => Value::String(key),
        PropertyKey::Symbol(key) => Value::Symbol(key),
    }
}

pub(super) fn value_key(key: Value) -> PropertyKey {
    match key {
        Value::String(key) => PropertyKey::String(key),
        Value::Symbol(key) => PropertyKey::Symbol(key),
        _ => unreachable!("converted property key"),
    }
}

impl Realm {
    pub(super) fn async_object_next(
        &mut self,
        data: &AsyncExecutionData,
        state: &mut AsyncExpressionData,
        path: ExprPath,
        element: usize,
        object: Value,
    ) -> Result<(), Error> {
        let expression = expression_at(data, state.root, &path);
        let ExprKind::Object(elements) = &expression.kind else {
            unreachable!("object source")
        };
        let span = expression.span;
        let Some(next) = elements.get(element) else {
            state.current = Some(object);
            return Ok(());
        };
        match next {
            ObjectElement::Spread(_) => {
                let child = child_path(&path, ExprChild::ObjectSpread(element), span)?;
                push(
                    state,
                    ExprTask::ObjectSpread {
                        path,
                        element,
                        object,
                    },
                    span,
                )?;
                push(state, ExprTask::Evaluate(child), span)
            }
            ObjectElement::Property(property) => {
                self.tick(property.span)?;
                if matches!(&property.name, PropertyName::Computed(key) if has_supported_await(key))
                {
                    let child = child_path(&path, ExprChild::ObjectKey(element), property.span)?;
                    push(
                        state,
                        ExprTask::ObjectKey {
                            path,
                            element,
                            object,
                        },
                        span,
                    )?;
                    push(state, ExprTask::Evaluate(child), span)
                } else {
                    let key = self.literal_property_key(property)?;
                    self.async_object_property(data, state, path, element, object, key)
                }
            }
        }
    }

    pub(super) fn async_object_property(
        &mut self,
        data: &AsyncExecutionData,
        state: &mut AsyncExpressionData,
        path: ExprPath,
        element: usize,
        object: Value,
        key: PropertyKey,
    ) -> Result<(), Error> {
        let property = property(data, state, &path, element);
        if matches!(
            property.kind,
            PropertyKind::Method | PropertyKind::Getter | PropertyKind::Setter
        ) {
            self.define_method_property(handle(&object), key, property, true)?;
            return push(
                state,
                ExprTask::ObjectNext {
                    path,
                    element: element + 1,
                    object,
                },
                property.span,
            );
        }
        if has_supported_await(&property.value) {
            let child = child_path(&path, ExprChild::ObjectValue(element), property.span)?;
            let task = if property.kind == PropertyKind::Prototype {
                ExprTask::Evaluate(child)
            } else {
                super::class::named_task(&property.value, child, key.clone())
            };
            push(
                state,
                ExprTask::ObjectValue {
                    path,
                    element,
                    object,
                    key: key_value(key),
                },
                property.span,
            )?;
            push(state, task, property.span)
        } else {
            let value = if property.kind == PropertyKind::Prototype {
                self.expression(&property.value)?
            } else {
                self.named_expression(&property.value, key.clone())?
            };
            self.async_object_value(data, state, path, element, object, (key, value))
        }
    }

    pub(super) fn async_object_value(
        &mut self,
        data: &AsyncExecutionData,
        state: &mut AsyncExpressionData,
        path: ExprPath,
        element: usize,
        object: Value,
        entry: (PropertyKey, Value),
    ) -> Result<(), Error> {
        let (key, value) = entry;
        let property = property(data, state, &path, element);
        self.define_literal_data_property(handle(&object), key, property, value)?;
        push(
            state,
            ExprTask::ObjectNext {
                path,
                element: element + 1,
                object,
            },
            property.span,
        )
    }
}
