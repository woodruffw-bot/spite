//! Bound function captures and Function.prototype.bind metadata semantics.

use super::Callable;
use crate::{Error, ExceptionKind, ObjectHandle, Realm, Value};
use spite_core::{JsString, Span};

#[derive(Clone, Debug)]
pub(crate) struct BoundFunction {
    pub target: ObjectHandle,
    pub this: Value,
    pub arguments: Vec<Value>,
}

impl Realm {
    pub(super) fn bind_function(
        &mut self,
        target: Value,
        mut arguments: std::vec::IntoIter<Value>,
        span: Span,
    ) -> Result<Value, Error> {
        if !self.is_callable(&target, span)? {
            return Err(Self::exception(
                ExceptionKind::TypeError,
                span,
                "bind requires a callable receiver",
            ));
        }
        let Value::Object(target) = target else {
            unreachable!("callable values are objects")
        };
        let bound = BoundFunction {
            target: target.clone(),
            this: arguments.next().unwrap_or(Value::Undefined),
            arguments: arguments.collect(),
        };
        let count = bound.arguments.len();
        // 20.2.3.2 allocates before observable length/name reads. The partially
        // initialized object is unreachable if either getter subsequently throws.
        let function =
            self.object_work(span, |objects, budget| objects.create_bound(bound, budget))?;
        let has_length = self.object_work(span, |objects, budget| {
            objects.has_own(&target, &JsString::from("length"), budget)
        })?;
        let mut length = 0.0;
        if has_length {
            if let Value::Number(number) =
                self.get_property(&target, &JsString::from("length"), span)?
            {
                if !number.is_nan() && number.trunc() > count as f64 {
                    length = number.trunc() - count as f64;
                }
            }
        }
        self.define_builtin_property(&function, "length", Value::Number(length), false, span)?;
        let name = self.get_property(&target, &JsString::from("name"), span)?;
        let mut units: Vec<u16> = "bound ".encode_utf16().collect();
        if let Value::String(name) = name {
            self.append_string(&mut units, &name, span)?;
        }
        let name = Value::String(JsString::from_code_units(units));
        self.check_string(&name, span)?;
        self.define_builtin_property(&function, "name", name, false, span)?;
        Ok(Value::Object(function))
    }
}

impl Callable {
    pub(super) fn copy_with_budget(
        &self,
        budget: &mut crate::object::Budget,
    ) -> Result<Self, crate::object::Error> {
        if let Self::Bound(bound) = self {
            budget.charge(1)?;
            budget.value(&bound.this)?;
            for value in &bound.arguments {
                budget.value(value)?;
            }
        }
        Ok(self.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        Limits,
        object::{Budget, DataDescriptor, DescriptorKind, PropertyDescriptor},
    };

    fn target(realm: &mut Realm) -> ObjectHandle {
        let Value::Object(target) = realm.eval("let target = ({}).toString; target").unwrap()
        else {
            panic!("function")
        };
        target
    }

    fn replace(realm: &mut Realm, object: &ObjectHandle, name: &str, value: Value) {
        assert!(
            realm
                .objects
                .define(
                    object,
                    JsString::from(name),
                    DataDescriptor {
                        value: Some(value),
                        ..Default::default()
                    },
                    &mut Budget::new(1000)
                )
                .unwrap()
        );
    }

    #[test]
    fn bound_length_uses_only_own_number_values_and_normalizes_special_numbers() {
        let mut realm = Realm::default();
        let target = target(&mut realm);
        for (value, expected) in [
            (Value::Number(f64::INFINITY), f64::INFINITY),
            (Value::Number(f64::NEG_INFINITY), 0.0),
            (Value::Number(f64::NAN), 0.0),
            (Value::Number(-0.0), 0.0),
            (Value::Number(3.9), 2.0),
            (Value::Number(-3.9), 0.0),
            (Value::String(JsString::from("9")), 0.0),
            (Value::Object(target.clone()), 0.0),
        ] {
            replace(&mut realm, &target, "length", value);
            let Value::Number(length) = realm.eval("target.bind(null, 1).length").unwrap() else {
                panic!("length")
            };
            assert_eq!(length, expected);
            assert!(!length.is_sign_negative());
        }
        realm
            .objects
            .delete(&target, &JsString::from("length"), &mut Budget::new(1000))
            .unwrap();
        let intrinsics = realm.intrinsics.as_ref().unwrap();
        let prototype = intrinsics.function_prototype.clone();
        let thrower = intrinsics.throw_type_error.clone();
        realm
            .objects
            .define(
                &prototype,
                JsString::from("length"),
                PropertyDescriptor {
                    kind: DescriptorKind::Accessor {
                        get: Some(Some(thrower)),
                        set: Some(None),
                    },
                    ..Default::default()
                },
                &mut Budget::new(1000),
            )
            .unwrap();
        assert_eq!(
            realm.eval("target.bind(null).length"),
            Ok(Value::Number(0.0))
        );
    }

    #[test]
    fn bound_names_preserve_code_units_and_do_not_coerce_nonstrings() {
        let mut realm = Realm::default();
        let target = target(&mut realm);
        replace(&mut realm, &target, "name", Value::Object(target.clone()));
        assert_eq!(
            realm.eval("target.bind(null).name"),
            Ok(Value::String(JsString::from("bound ")))
        );
        replace(
            &mut realm,
            &target,
            "name",
            Value::String(JsString::from_code_units(vec![0xd800, 0x61])),
        );
        let mut units: Vec<_> = "bound ".encode_utf16().collect();
        units.extend([0xd800, 0x61]);
        assert_eq!(
            realm.eval("target.bind(null).name"),
            Ok(Value::String(JsString::from_code_units(units)))
        );
    }

    #[test]
    fn bound_prototype_copies_target_prototype_and_allocation_precedes_metadata_reads() {
        let mut realm = Realm::default();
        let target = target(&mut realm);
        realm.eval("let bind = target.bind").unwrap();
        realm
            .objects
            .set_prototype(&target, None, &mut Budget::new(1000))
            .unwrap();
        let Value::Object(bound) = realm
            .eval("let bound = bind.call(target, null); bound")
            .unwrap()
        else {
            panic!("function")
        };
        assert!(realm.objects.inspect(&bound).unwrap().prototype().is_none());
        assert_eq!(
            realm.eval("bound()"),
            Ok(Value::String(JsString::from("[object Null]")))
        );
        assert_eq!(realm.eval("bound.caller"), Ok(Value::Undefined));
        realm.collect(1000).unwrap();
        let thrower = realm.intrinsics.as_ref().unwrap().throw_type_error.clone();
        realm
            .objects
            .define(
                &target,
                JsString::from("length"),
                PropertyDescriptor {
                    kind: DescriptorKind::Accessor {
                        get: Some(Some(thrower)),
                        set: Some(None),
                    },
                    ..Default::default()
                },
                &mut Budget::new(1000),
            )
            .unwrap();
        assert!(matches!(
            realm.eval("bind.call(target, null)"),
            Err(Error::Exception {
                kind: ExceptionKind::TypeError,
                ..
            })
        ));
        assert_eq!(realm.collect(1000).unwrap().reclaimed, 1);
    }

    #[test]
    fn recursive_metadata_getters_hit_host_nesting_limits_and_restore_call_state() {
        let mut realm = Realm::default();
        let target = target(&mut realm);
        realm.eval("let flag = 0").unwrap();
        realm.collect(1000).unwrap();
        let bind = realm.intrinsics.as_ref().unwrap().function_bind.clone();
        realm
            .objects
            .define(
                &target,
                JsString::from("length"),
                PropertyDescriptor {
                    kind: DescriptorKind::Accessor {
                        get: Some(Some(bind)),
                        set: Some(None),
                    },
                    ..Default::default()
                },
                &mut Budget::new(1000),
            )
            .unwrap();
        let Err(Error::Limit { message, .. }) =
            realm.eval("try { target.bind(null); } catch { flag = 1; } finally { flag = 2; }")
        else {
            panic!("host nesting limit")
        };
        assert_eq!(message, "call nesting limit exceeded");
        assert_eq!(realm.call_depth, 0);
        assert_eq!(realm.eval("flag"), Ok(Value::Number(0.0)));
        let result = realm.collect(20000).unwrap();
        assert!(result.reclaimed > 0);
        assert_eq!(result.live, 11);
        replace(&mut realm, &target, "length", Value::Number(0.0));
        assert_eq!(
            realm.eval("target.bind(null)()"),
            Ok(Value::String(JsString::from("[object Null]")))
        );
    }

    #[test]
    fn deep_bound_chains_dispatch_and_trace_iteratively() {
        let mut realm = Realm::new(Limits {
            max_heap_entries: 11_000,
            ..Limits::default()
        });
        let mut function = target(&mut realm);
        for _ in 0..10_000 {
            function = realm
                .objects
                .create_bound(
                    BoundFunction {
                        target: function,
                        this: Value::Null,
                        arguments: Vec::new(),
                    },
                    &mut Budget::new(10),
                )
                .unwrap();
        }
        let value = Value::Object(function);
        let root = realm.root_value(value.clone(), 100).unwrap();
        assert_eq!(realm.collect(200_000).unwrap().live, 10011);
        assert_eq!(
            realm.call(value, Value::Undefined, Vec::new(), Span::new(0, 0)),
            Ok(Value::String(JsString::from("[object Null]")))
        );
        assert_eq!(realm.call_depth, 0);
        drop(root);
        assert_eq!(realm.collect(200_000).unwrap().reclaimed, 10_000);
    }
}
