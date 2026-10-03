//! Builtin function objects and the initial Object/Function prototype graph.

use crate::{
    Error, ExceptionKind, ObjectHandle, Realm, Value,
    object::{DataDescriptor, DescriptorKind, PropertyDescriptor},
};
use spite_core::{JsString, Span};

#[derive(Clone, Copy, Debug)]
pub(crate) enum Builtin {
    FunctionPrototype,
    ThrowTypeError,
    ObjectToString,
    ObjectValueOf,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::object::Budget;

    #[test]
    fn object_prototype_is_immutable_and_function_prototype_is_callable() {
        let mut realm = Realm::default();
        realm.eval("({})").unwrap();
        let intrinsics = realm.intrinsics.as_ref().unwrap();
        let object = intrinsics.object_prototype.clone();
        let function = intrinsics.function_prototype.clone();
        assert!(realm.objects.inspect(&object).unwrap().is_extensible());
        assert!(
            realm
                .objects
                .set_prototype(&object, None, &mut Budget::new(100))
                .unwrap()
        );
        assert!(
            !realm
                .objects
                .set_prototype(&object, Some(&function), &mut Budget::new(100))
                .unwrap()
        );
        let independent = realm.objects.create(None).unwrap();
        assert!(
            !realm
                .objects
                .set_prototype(&object, Some(&independent), &mut Budget::new(100))
                .unwrap()
        );
        assert_eq!(
            realm.call(
                Value::Object(function),
                Value::Null,
                vec![Value::Number(7.0)],
                Span::new(0, 0)
            ),
            Ok(Value::Undefined)
        );
    }

    #[test]
    fn restricted_descriptors_share_a_frozen_nonextensible_thrower() {
        let mut realm = Realm::default();
        realm.eval("let f = ({}).toString").unwrap();
        let intrinsics = realm.intrinsics.as_ref().unwrap();
        let function = intrinsics.function_prototype.clone();
        let thrower = intrinsics.throw_type_error.clone();
        let object = realm.objects.inspect(&thrower).unwrap();
        assert!(object.is_callable());
        assert!(!object.is_extensible());
        assert_eq!(object.prototype(), Some(&function));
        for (name, expected) in [
            ("name", Value::String(JsString::from(""))),
            ("length", Value::Number(0.0)),
        ] {
            let property = object
                .own_property(&JsString::from(name))
                .unwrap()
                .as_data()
                .unwrap();
            assert_eq!(property.value, expected);
            assert!(!property.writable && !property.enumerable && !property.configurable);
        }
        for name in ["caller", "arguments"] {
            let crate::object::Property::Accessor(property) = realm
                .objects
                .inspect(&function)
                .unwrap()
                .own_property(&JsString::from(name))
                .unwrap()
            else {
                panic!("accessor")
            };
            assert_eq!(property.get.as_ref(), Some(&thrower));
            assert_eq!(property.set.as_ref(), Some(&thrower));
            assert!(!property.enumerable && property.configurable);
            assert!(
                realm
                    .objects
                    .delete(&function, &JsString::from(name), &mut Budget::new(1000))
                    .unwrap()
            );
            assert_eq!(realm.eval(&format!("f.{name}")), Ok(Value::Undefined));
            assert_eq!(
                realm.eval(&format!("'{name}' in f")),
                Ok(Value::Boolean(false))
            );
        }
        // The intrinsic identity is retained even after deleting all accessor edges.
        assert_eq!(realm.collect(1000).unwrap().live, 5);
        assert!(realm.objects.inspect(&thrower).is_ok());
    }
}

#[derive(Clone, Debug)]
pub(crate) enum Callable {
    Builtin(Builtin),
}

#[derive(Debug)]
pub(super) struct Intrinsics {
    pub object_prototype: ObjectHandle,
    pub function_prototype: ObjectHandle,
    pub object_to_string: ObjectHandle,
    pub object_value_of: ObjectHandle,
    pub throw_type_error: ObjectHandle,
}

impl Intrinsics {
    pub fn roots(&self) -> impl Iterator<Item = &ObjectHandle> {
        [
            &self.object_prototype,
            &self.function_prototype,
            &self.object_to_string,
            &self.object_value_of,
            &self.throw_type_error,
        ]
        .into_iter()
    }
}

impl Realm {
    pub(super) fn ensure_object_intrinsics(&mut self, span: Span) -> Result<ObjectHandle, Error> {
        if let Some(intrinsics) = &self.intrinsics {
            return Ok(intrinsics.object_prototype.clone());
        }
        let object_prototype =
            self.object_work(span, |objects, _| objects.create_object_prototype())?;
        let function_prototype =
            self.new_builtin(&object_prototype, Builtin::FunctionPrototype, "", span)?;
        let object_to_string = self.new_builtin(
            &function_prototype,
            Builtin::ObjectToString,
            "toString",
            span,
        )?;
        let object_value_of =
            self.new_builtin(&function_prototype, Builtin::ObjectValueOf, "valueOf", span)?;
        for (name, handle) in [
            ("toString", &object_to_string),
            ("valueOf", &object_value_of),
        ] {
            self.define_builtin_property(
                &object_prototype,
                name,
                Value::Object(handle.clone()),
                true,
                span,
            )?;
        }
        // 9.3.2 / 10.2.4: Function.prototype owns the shared restricted accessors.
        let throw_type_error =
            self.new_builtin(&function_prototype, Builtin::ThrowTypeError, "", span)?;
        for name in ["name", "length"] {
            self.object_work(span, |objects, budget| {
                objects.define(
                    &throw_type_error,
                    JsString::from(name),
                    DataDescriptor {
                        configurable: Some(false),
                        ..Default::default()
                    },
                    budget,
                )
            })?;
        }
        self.object_work(span, |objects, _| {
            objects.prevent_extensions(&throw_type_error)
        })?;
        for name in ["caller", "arguments"] {
            self.object_work(span, |objects, budget| {
                objects.define(
                    &function_prototype,
                    JsString::from(name),
                    PropertyDescriptor {
                        kind: DescriptorKind::Accessor {
                            get: Some(Some(throw_type_error.clone())),
                            set: Some(Some(throw_type_error.clone())),
                        },
                        enumerable: Some(false),
                        configurable: Some(true),
                    },
                    budget,
                )
            })?;
        }
        // Publish only after the graph is fully initialized. A failed attempt
        // leaves unreachable allocations that explicit collection can reclaim.
        self.intrinsics = Some(Intrinsics {
            object_prototype: object_prototype.clone(),
            function_prototype,
            object_to_string,
            object_value_of,
            throw_type_error,
        });
        Ok(object_prototype)
    }

    fn new_builtin(
        &mut self,
        prototype: &ObjectHandle,
        builtin: Builtin,
        name: &str,
        span: Span,
    ) -> Result<ObjectHandle, Error> {
        let object = self.object_work(span, |objects, _| {
            objects.create_builtin(prototype, builtin)
        })?;
        self.define_builtin_property(&object, "length", Value::Number(0.0), false, span)?;
        self.define_builtin_property(
            &object,
            "name",
            Value::String(JsString::from(name)),
            false,
            span,
        )?;
        Ok(object)
    }

    fn define_builtin_property(
        &mut self,
        object: &ObjectHandle,
        name: &str,
        value: Value,
        writable: bool,
        span: Span,
    ) -> Result<(), Error> {
        let defined = self.object_work(span, |objects, budget| {
            objects.define(
                object,
                JsString::from(name),
                DataDescriptor {
                    value: Some(value),
                    writable: Some(writable),
                    enumerable: Some(false),
                    configurable: Some(true),
                },
                budget,
            )
        })?;
        debug_assert!(defined, "fresh intrinsic property");
        Ok(())
    }

    pub(super) fn is_callable(&mut self, value: &Value, span: Span) -> Result<bool, Error> {
        let Value::Object(object) = value else {
            return Ok(false);
        };
        self.object_work(span, |objects, _| {
            Ok(objects.inspect(object)?.is_callable())
        })
    }

    pub(super) fn call(
        &mut self,
        function: Value,
        this: Value,
        _arguments: Vec<Value>,
        span: Span,
    ) -> Result<Value, Error> {
        // 13.3.6.2: callers evaluate arguments before entering this callable check.
        let callable = if let Value::Object(object) = function {
            self.object_work(span, |objects, _| {
                Ok(objects.inspect(&object)?.callable().cloned())
            })?
        } else {
            None
        };
        let builtin = match callable {
            Some(Callable::Builtin(builtin)) => builtin,
            None => {
                return Err(Self::exception(
                    ExceptionKind::TypeError,
                    span,
                    "value is not callable",
                ));
            }
        };
        match builtin {
            Builtin::FunctionPrototype => Ok(Value::Undefined),
            Builtin::ThrowTypeError => Err(Self::exception(
                ExceptionKind::TypeError,
                span,
                "restricted function property",
            )),
            Builtin::ObjectValueOf => {
                Self::require_object_coercible(&this, span)?;
                if matches!(this, Value::Object(_)) {
                    Ok(this)
                } else {
                    Err(Self::unsupported(
                        span,
                        "returning primitive wrapper objects is not implemented",
                    ))
                }
            }
            Builtin::ObjectToString => {
                // 20.1.3.6. No Symbol keys or additional exotic object kinds are
                // exposed yet; their tags and hooks must join this dispatch later.
                let tag = match &this {
                    Value::Undefined => "Undefined",
                    Value::Null => "Null",
                    Value::Boolean(_) => "Boolean",
                    Value::Number(_) => "Number",
                    Value::BigInt(_) => "BigInt",
                    Value::String(_) => "String",
                    Value::Object(_) if self.is_callable(&this, span)? => "Function",
                    Value::Object(_) => "Object",
                };
                Ok(Value::String(JsString::from(
                    format!("[object {tag}]").as_str(),
                )))
            }
        }
    }
}
