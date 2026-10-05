//! Public ClassFieldDefinitionEvaluation, DefineField, and instance initialization.

use crate::{Error, ObjectHandle, Realm, Value, environment::EnvironmentHandle};
use spite_core::{PropertyKey, Span};
use spite_parser::ast::Expr;
use std::rc::Rc;

#[derive(Clone, Debug)]
pub(crate) struct FieldInitializer {
    pub expression: Rc<Expr>,
    pub environment: EnvironmentHandle,
    pub home_object: ObjectHandle,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::object::{Budget, Error as ObjectError};

    fn object(value: Value) -> ObjectHandle {
        let Value::Object(handle) = value else {
            panic!("object")
        };
        handle
    }

    #[test]
    fn checked_field_edges_fail_before_replacing_the_constructor_fields() {
        let mut realm = Realm::default();
        let constructor = object(realm.eval("class C{x=1;}C").unwrap());
        let fields = realm
            .instance_fields(&constructor, Span::new(0, 0))
            .unwrap();
        let stale = object(realm.eval("({})").unwrap());
        realm.collect(usize::MAX).unwrap();
        let mut other = Realm::default();
        let foreign = object(other.eval("({})").unwrap());
        let original = fields[0].initializer.as_ref().unwrap();
        for (environment, home_object, expected) in [
            (
                other.scopes[0].clone(),
                original.home_object.clone(),
                ObjectError::Heap(spite_heap::Error::ForeignHandle),
            ),
            (
                original.environment.clone(),
                foreign,
                ObjectError::Heap(spite_heap::Error::ForeignHandle),
            ),
            (
                original.environment.clone(),
                stale,
                ObjectError::Heap(spite_heap::Error::StaleHandle),
            ),
        ] {
            let fields: Rc<[ClassField]> = vec![ClassField {
                name: PropertyKey::from("wrong"),
                initializer: Some(FieldInitializer {
                    expression: original.expression.clone(),
                    environment,
                    home_object,
                }),
                span: fields[0].span,
            }]
            .into();
            assert_eq!(
                realm.objects.set_class_fields(
                    &constructor,
                    fields,
                    &mut Budget::with_work_limit(None)
                ),
                Err(expected)
            );
            assert_eq!(realm.eval("new C().x"), Ok(Value::Number(1.0)));
        }
        let plain = object(realm.eval("({})").unwrap());
        assert_eq!(
            realm.objects.set_class_fields(
                &plain,
                fields.clone(),
                &mut Budget::with_work_limit(None)
            ),
            Err(ObjectError::WrongKind)
        );
        assert_eq!(
            realm
                .objects
                .set_class_fields(&constructor, fields, &mut Budget::new(0)),
            Err(ObjectError::WorkLimit)
        );
        assert_eq!(realm.eval("new C().x"), Ok(Value::Number(1.0)));
    }
}

#[derive(Clone, Debug)]
pub(crate) struct ClassField {
    pub name: PropertyKey,
    pub initializer: Option<FieldInitializer>,
    pub span: Span,
}

impl Realm {
    pub(super) fn instance_fields(
        &mut self,
        constructor: &ObjectHandle,
        span: Span,
    ) -> Result<Rc<[ClassField]>, Error> {
        self.object_work(span, |objects, _| {
            let Some(super::Callable::ClassConstructor(class)) =
                objects.inspect(constructor)?.callable()
            else {
                return Ok(Rc::from([]));
            };
            Ok(class.fields.clone())
        })
    }

    pub(crate) fn initialize_instance_fields(
        &mut self,
        receiver: &Value,
        constructor: &ObjectHandle,
        span: Span,
    ) -> Result<(), Error> {
        let fields = self.instance_fields(constructor, span)?;
        self.initialize_fields(receiver, &fields)
    }

    pub(super) fn initialize_fields(
        &mut self,
        receiver: &Value,
        fields: &[ClassField],
    ) -> Result<(), Error> {
        let Value::Object(receiver) = receiver else {
            unreachable!("constructed object")
        };
        for field in fields {
            self.tick(field.span)?;
            let value = if let Some(initializer) = &field.initializer {
                self.field_initializer(initializer, receiver, field.name.clone(), field.span)?
            } else {
                Value::Undefined
            };
            // DefineField uses CreateDataPropertyOrThrow, without inherited
            // setters, and observes descriptor failures after the initializer.
            self.define_property_or_throw(
                receiver,
                field.name.clone(),
                crate::object::DataDescriptor {
                    value: Some(value),
                    writable: Some(true),
                    enumerable: Some(true),
                    configurable: Some(true),
                }
                .into(),
                field.span,
            )?;
        }
        Ok(())
    }

    fn field_initializer(
        &mut self,
        initializer: &FieldInitializer,
        receiver: &ObjectHandle,
        name: PropertyKey,
        span: Span,
    ) -> Result<Value, Error> {
        self.enter_call(span)?;
        let result = self.field_initializer_inner(initializer, receiver, name, span);
        self.call_depth -= 1;
        result
    }

    fn field_initializer_inner(
        &mut self,
        initializer: &FieldInitializer,
        receiver: &ObjectHandle,
        name: PropertyKey,
        span: Span,
    ) -> Result<Value, Error> {
        let environment = self.object_work(span, |objects, budget| {
            objects.create_function_environment(
                initializer.environment.clone(),
                Default::default(),
                Value::Object(receiver.clone()),
                crate::environment::FunctionContext {
                    home_object: Some(initializer.home_object.clone()),
                    class_field_initializer: true,
                    ..Default::default()
                },
                budget,
            )
        })?;
        let caller_depth = self.scopes.len();
        let caller_strict = std::mem::replace(&mut self.strict, true);
        let caller_variable = self.variable_environment.replace(environment.clone());
        self.scopes.push(environment);
        // EvaluateBody for Initializer omits FunctionDeclarationInstantiation
        // and performs NamedEvaluation of anonymous functions/classes (15.2.3).
        let result = self.named_expression(&initializer.expression, name);
        self.scopes.truncate(caller_depth);
        self.strict = caller_strict;
        self.variable_environment = caller_variable;
        result
    }
}
