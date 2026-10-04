//! InstanceofOperator and OrdinaryHasInstance (13.10.2, 7.3.21).

use super::Callable;
use crate::{Error, ExceptionKind, ObjectHandle, Realm, Value};
use spite_core::{JsString, Span};

impl Realm {
    pub(crate) fn instance_of(
        &mut self,
        value: Value,
        mut target: Value,
        span: Span,
    ) -> Result<bool, Error> {
        loop {
            let Value::Object(object) = target else {
                return Err(Self::exception(
                    ExceptionKind::TypeError,
                    span,
                    "right operand of instanceof must be an object",
                ));
            };
            // Symbol properties cannot be installed or reflected yet. The only
            // reachable @@hasInstance method is Function.prototype's fixed
            // intrinsic (20.2.3.6); recognize its exact inheritance path. Symbol
            // support must replace this lookup before custom hooks are exposed.
            let intrinsic = self.has_instance_intrinsic(&object, span)?;
            if intrinsic {
                self.check_argument_count(1, span)?;
            }
            let (callable, bound_target) = self.object_work(span, |objects, _| {
                let record = objects.inspect(&object)?;
                Ok((
                    record.is_callable(),
                    match record.callable() {
                        Some(Callable::Bound(bound)) => Some(bound.target.clone()),
                        _ => None,
                    },
                ))
            })?;
            if !callable {
                return if intrinsic {
                    Ok(false)
                } else {
                    Err(Self::exception(
                        ExceptionKind::TypeError,
                        span,
                        "right operand of instanceof is not callable",
                    ))
                };
            }
            // The intrinsic calls OrdinaryHasInstance; an absent hook falls
            // back to it after IsCallable. Bound targets re-enter InstanceofOperator.
            if let Some(bound) = bound_target {
                target = Value::Object(bound);
                continue;
            }
            let Value::Object(value) = &value else {
                return Ok(false);
            };
            let prototype = self.get_property(&object, &JsString::from("prototype"), span)?;
            let Value::Object(prototype) = prototype else {
                return Err(Self::exception(
                    ExceptionKind::TypeError,
                    span,
                    "instanceof prototype must be an object",
                ));
            };
            let mut next = Some(value.clone());
            while let Some(handle) = next {
                next = self.object_work(span, |objects, _| {
                    Ok(objects.inspect(&handle)?.prototype().cloned())
                })?;
                if next.as_ref() == Some(&prototype) {
                    return Ok(true);
                }
            }
            return Ok(false);
        }
    }

    fn has_instance_intrinsic(&mut self, object: &ObjectHandle, span: Span) -> Result<bool, Error> {
        let intrinsic = self
            .intrinsics
            .as_ref()
            .expect("initialized realm")
            .function_prototype
            .clone();
        let mut next = Some(object.clone());
        while let Some(handle) = next {
            self.tick(span)?;
            if handle == intrinsic {
                return Ok(true);
            }
            next = self.object_work(span, |objects, _| {
                Ok(objects.inspect(&handle)?.prototype().cloned())
            })?;
        }
        Ok(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        Limits,
        function::BoundFunction,
        object::{Budget, DescriptorKind, PropertyDescriptor},
    };

    #[test]
    fn prototype_getters_run_only_for_object_left_operands_and_receive_the_target() {
        let mut realm = Realm::default();
        let Value::Object(target) = realm
            .eval("let calls=0,receiver;let p={};let target=()=>1;target")
            .unwrap()
        else {
            panic!()
        };
        let Value::Object(getter) = realm
            .eval("(function(){'use strict';calls++;receiver=this;return p;})")
            .unwrap()
        else {
            panic!()
        };
        realm
            .objects
            .define(
                &target,
                "prototype",
                PropertyDescriptor {
                    kind: DescriptorKind::Accessor {
                        get: Some(Some(getter)),
                        set: Some(None),
                    },
                    configurable: Some(true),
                    ..Default::default()
                },
                &mut Budget::new(1000),
            )
            .unwrap();
        assert_eq!(
            realm.eval("!(1 instanceof target) && calls===0"),
            Ok(Value::Boolean(true))
        );
        assert_eq!(
            realm.eval("({__proto__:p}) instanceof target && calls===1 && receiver===target"),
            Ok(Value::Boolean(true))
        );
        let Value::Object(bound) = realm.eval("let bound=target.bind(null);bound").unwrap() else {
            panic!()
        };
        let thrower = realm.intrinsics.as_ref().unwrap().throw_type_error.clone();
        realm
            .objects
            .define(
                &bound,
                "prototype",
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
            realm.eval("({__proto__:p}) instanceof bound && calls===2 && receiver===target"),
            Ok(Value::Boolean(true))
        );
        assert_eq!(
            realm.eval("p=null;1 instanceof target"),
            Ok(Value::Boolean(false))
        );
        assert!(matches!(
            realm.eval("({}) instanceof target"),
            Err(Error::Exception {
                kind: ExceptionKind::TypeError,
                ..
            })
        ));
    }

    #[test]
    fn absent_intrinsic_hook_falls_back_to_callability_and_ordinary_has_instance() {
        let mut realm = Realm::default();
        let Value::Object(target) = realm.eval("function F(){}F").unwrap() else {
            panic!()
        };
        realm
            .objects
            .set_prototype(&target, None, &mut Budget::new(1000))
            .unwrap();
        assert_eq!(realm.eval("new F instanceof F"), Ok(Value::Boolean(true)));
        assert_eq!(realm.eval("1 instanceof F"), Ok(Value::Boolean(false)));
    }

    #[test]
    fn deep_bound_instance_checks_are_iterative_and_budgeted() {
        let mut realm = Realm::new(Limits {
            max_heap_entries: 20_000,
            ..Limits::default()
        });
        let Value::Object(mut target) = realm.eval("function F(){}let instance=new F;F").unwrap()
        else {
            panic!()
        };
        for _ in 0..10_000 {
            target = realm
                .objects
                .create_bound(
                    BoundFunction {
                        target,
                        this: Value::Null,
                        arguments: Vec::new(),
                    },
                    &mut Budget::new(10),
                )
                .unwrap();
        }
        let instance = realm.eval("instance").unwrap();
        assert_eq!(
            realm.instance_of(
                instance.clone(),
                Value::Object(target.clone()),
                Span::new(0, 0)
            ),
            Ok(true)
        );
        realm.remaining_steps = 5;
        assert!(matches!(
            realm.instance_of(instance, Value::Object(target), Span::new(0, 0)),
            Err(Error::Limit { .. })
        ));
    }
}
