//! Ordinary construction and iterative bound/default forwarding (10.2.2, 10.4.1.2).

use super::{Builtin, Callable};
use crate::{Error, ExceptionKind, ObjectHandle, Realm, Value};
use spite_core::{JsString, Span};

impl Realm {
    pub(crate) fn construct(
        &mut self,
        function: Value,
        arguments: Vec<Value>,
        span: Span,
    ) -> Result<Value, Error> {
        self.construct_with_new_target(function, arguments, None, span)
    }

    pub(crate) fn construct_with_new_target(
        &mut self,
        function: Value,
        arguments: Vec<Value>,
        new_target: Option<ObjectHandle>,
        span: Span,
    ) -> Result<Value, Error> {
        self.enter_call(span)?;
        let mut elements = Vec::new();
        let result = self
            .construct_inner(function, arguments, new_target, &mut elements, span)
            .and_then(|value| {
                for pending in elements.iter().rev() {
                    self.initialize_elements(&value, pending, span)?;
                }
                Ok(value)
            });
        self.call_depth -= 1;
        result
    }

    fn construct_inner(
        &mut self,
        function: Value,
        mut arguments: Vec<Value>,
        new_target: Option<ObjectHandle>,
        elements: &mut Vec<super::class_field::InstanceElements>,
        span: Span,
    ) -> Result<Value, Error> {
        self.check_argument_count(arguments.len(), span)?;
        // EvaluateNew (13.3.5.1.1) evaluates every argument before IsConstructor.
        if !self.is_constructor(&function, span)? {
            return Err(Self::exception(
                ExceptionKind::TypeError,
                span,
                "value is not a constructor",
            ));
        }
        let Value::Object(mut function) = function else {
            unreachable!("constructor object")
        };
        let mut new_target = new_target.unwrap_or_else(|| function.clone());
        if !self.is_constructor(&Value::Object(new_target.clone()), span)? {
            return Err(Self::exception(
                ExceptionKind::TypeError,
                span,
                "newTarget is not a constructor",
            ));
        }
        loop {
            let callable = self.object_work(span, |objects, budget| {
                objects
                    .inspect(&function)?
                    .callable()
                    .expect("constructor has call metadata")
                    .copy_with_budget(budget)
            })?;
            match callable {
                Callable::Builtin(Builtin::Function) => {
                    return self.dynamic_function(Some(new_target), arguments.into_iter(), span);
                }
                Callable::Bound(bound) => {
                    let count = bound
                        .arguments
                        .len()
                        .checked_add(arguments.len())
                        .ok_or_else(|| Error::Limit {
                            span,
                            message: "call argument limit exceeded".into(),
                        })?;
                    self.check_argument_count(count, span)?;
                    let mut values = bound.arguments;
                    values.extend(arguments);
                    arguments = values;
                    if new_target == function {
                        new_target = bound.target.clone();
                    }
                    function = bound.target;
                    // [[BoundThis]] is ignored during construction.
                }
                Callable::Ordinary(code) => {
                    return self.construct_base_function(
                        code, function, new_target, None, arguments, span,
                    );
                }
                Callable::ClassConstructor(class) if class.derived && class.default => {
                    // The default derived closure forwards the argument List;
                    // no rest binding or Array iterator is observable (15.7.14).
                    let superclass = self.object_work(span, |objects, budget| {
                        budget.charge(1)?;
                        Ok(objects
                            .inspect(&function)?
                            .prototype()
                            .cloned()
                            .map_or(Value::Null, Value::Object))
                    })?;
                    if !self.is_constructor(&superclass, span)? {
                        return Err(Self::exception(
                            ExceptionKind::TypeError,
                            span,
                            "class superclass is not a constructor",
                        ));
                    }
                    let Value::Object(superclass) = superclass else {
                        unreachable!("constructor")
                    };
                    if !class.elements.fields.is_empty()
                        || !class.elements.private_methods.is_empty()
                    {
                        elements.try_reserve(1).map_err(|_| Error::Limit {
                            span,
                            message: "derived element forwarding capacity exceeded".into(),
                        })?;
                        elements.push(class.elements);
                    }
                    function = superclass;
                    // Initialize each default class's own elements, from the
                    // superclass outward, after this forwarded construction.
                }
                Callable::ClassConstructor(class) if class.derived => {
                    return self.call_ordinary(
                        class.method.code,
                        function.clone(),
                        Value::Undefined,
                        crate::environment::FunctionContext {
                            new_target: Some(new_target),
                            home_object: Some(class.method.home_object),
                            derived_constructor: Some(function),
                            class_field_initializer: false,
                        },
                        arguments.into_iter(),
                        span,
                    );
                }
                Callable::ClassConstructor(class) => {
                    return self.construct_base_function(
                        class.method.code,
                        function,
                        new_target,
                        Some(class.method.home_object),
                        arguments,
                        span,
                    );
                }
                Callable::Builtin(Builtin::Object) => {
                    return self.object_constructor(
                        Some(new_target),
                        arguments.into_iter().next().unwrap_or(Value::Undefined),
                        span,
                    );
                }
                Callable::Builtin(Builtin::Set) => {
                    return self.set_constructor(
                        new_target,
                        arguments.into_iter().next().unwrap_or(Value::Undefined),
                        span,
                    );
                }
                Callable::Builtin(Builtin::WeakSet) => {
                    return self.weak_set_constructor(
                        new_target,
                        arguments.into_iter().next().unwrap_or(Value::Undefined),
                        span,
                    );
                }
                Callable::Builtin(Builtin::Map) => {
                    return self.map_constructor(
                        new_target,
                        arguments.into_iter().next().unwrap_or(Value::Undefined),
                        span,
                    );
                }
                Callable::Builtin(Builtin::Array) => {
                    return self.array_constructor(Some(new_target), arguments.into_iter(), span);
                }
                Callable::Builtin(Builtin::Iterator) => {
                    // Iterator (27.1.3.1.1) is abstract but has [[Construct]].
                    // Reject the active function before reading newTarget.prototype.
                    if new_target == function {
                        return Err(Self::exception(
                            ExceptionKind::TypeError,
                            span,
                            "Iterator requires a distinct newTarget",
                        ));
                    }
                    let prototype =
                        self.get_property(&new_target, &JsString::from("prototype"), span)?;
                    let prototype = if let Value::Object(prototype) = prototype {
                        prototype
                    } else {
                        self.intrinsics
                            .as_ref()
                            .expect("initialized realm")
                            .iterator
                            .prototype
                            .clone()
                    };
                    return self
                        .object_work(span, |objects, _| objects.create(Some(&prototype)))
                        .map(Value::Object);
                }
                Callable::Builtin(Builtin::String) => {
                    return self.string_constructor(
                        Some(new_target),
                        arguments.into_iter().next(),
                        span,
                    );
                }
                Callable::Builtin(Builtin::Error(kind)) => {
                    return self.error_constructor(
                        kind,
                        Some(new_target),
                        arguments.into_iter(),
                        span,
                    );
                }
                Callable::Builtin(Builtin::Boolean) => {
                    let value = arguments.first().unwrap_or(&Value::Undefined).to_boolean();
                    let prototype =
                        self.get_property(&new_target, &JsString::from("prototype"), span)?;
                    let prototype = if let Value::Object(prototype) = prototype {
                        prototype
                    } else {
                        self.intrinsics
                            .as_ref()
                            .expect("initialized realm")
                            .boolean
                            .prototype
                            .clone()
                    };
                    return self
                        .object_work(span, |objects, _| objects.create_boolean(&prototype, value))
                        .map(Value::Object);
                }
                Callable::Builtin(Builtin::Number) => {
                    let value =
                        self.number_constructor_value(arguments.into_iter().next(), span)?;
                    let prototype =
                        self.get_property(&new_target, &JsString::from("prototype"), span)?;
                    let prototype = if let Value::Object(prototype) = prototype {
                        prototype
                    } else {
                        self.intrinsics
                            .as_ref()
                            .expect("initialized realm")
                            .number
                            .prototype
                            .clone()
                    };
                    return self
                        .object_work(span, |objects, _| objects.create_number(&prototype, value))
                        .map(Value::Object);
                }
                _ => unreachable!("constructibility is enabled only for known constructors"),
            }
        }
    }

    fn construct_base_function(
        &mut self,
        code: super::ScriptFunction,
        function: ObjectHandle,
        new_target: ObjectHandle,
        home_object: Option<ObjectHandle>,
        arguments: Vec<Value>,
        span: Span,
    ) -> Result<Value, Error> {
        // 10.2.2, 10.1.13–14: allocate before binding parameters or running the
        // body; base constructors ignore primitive returns, including null.
        let prototype = self.get_property(&new_target, &JsString::from("prototype"), span)?;
        let prototype = if let Value::Object(prototype) = prototype {
            prototype
        } else {
            self.intrinsics
                .as_ref()
                .expect("initialized realm")
                .object_prototype
                .clone()
        };
        let instance = self.object_work(span, |objects, _| objects.create(Some(&prototype)))?;
        let this = Value::Object(instance);
        // Base instance elements precede FunctionDeclarationInstantiation,
        // including parameter defaults, and use their own initializer context.
        self.initialize_instance_elements(&this, &function, span)?;
        let result = self.call_ordinary(
            code,
            function,
            this.clone(),
            crate::environment::FunctionContext {
                new_target: Some(new_target),
                home_object,
                derived_constructor: None,
                class_field_initializer: false,
            },
            arguments.into_iter(),
            span,
        )?;
        Ok(if matches!(result, Value::Object(_)) {
            result
        } else {
            this
        })
    }

    pub(crate) fn is_constructor(&mut self, value: &Value, span: Span) -> Result<bool, Error> {
        if let Value::Object(object) = value {
            self.object_work(span, |objects, _| {
                Ok(objects.inspect(object)?.is_constructor())
            })
        } else {
            Ok(false)
        }
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
    fn bound_prototype_getters_are_ignored_and_constructor_flags_follow_the_target() {
        let mut realm = Realm::default();
        let Value::Object(bound) = realm
            .eval("function F(){this.x=7;}let B=F.bind(null);B")
            .unwrap()
        else {
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
                    enumerable: Some(false),
                    configurable: Some(true),
                },
                &mut Budget::new(1000),
            )
            .unwrap();
        assert!(realm.inspect_object(&bound).unwrap().is_constructor());
        assert_eq!(realm.eval("new B().x"), Ok(Value::Number(7.0)));
        for source in [
            "()=>1",
            "({}).toString",
            "(()=>1).bind(null)",
            "({}).toString.bind(null)",
        ] {
            let Value::Object(object) = realm.eval(source).unwrap() else {
                panic!()
            };
            assert!(realm.inspect_object(&object).unwrap().is_callable());
            assert!(!realm.inspect_object(&object).unwrap().is_constructor());
        }
    }

    #[test]
    fn deep_bound_constructors_forward_and_trace_without_rust_recursion() {
        let mut realm = Realm::new(Limits {
            max_heap_entries: Some(20_000),
            ..Limits::default()
        });
        let Value::Object(mut target) = realm.eval("function F(){this.x=7;}F").unwrap() else {
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
        let value = Value::Object(target);
        let root = realm.root_value(value.clone(), 100).unwrap();
        assert_eq!(
            realm.collect(usize::MAX).unwrap().live,
            crate::test_support::REALM_ENTRIES + 10_002
        );
        let instance = realm.construct(value, Vec::new(), Span::new(0, 0)).unwrap();
        assert_eq!(
            realm.get_property_value(&instance, &JsString::from("x"), Span::new(0, 0)),
            Ok(Value::Number(7.0))
        );
        assert_eq!(realm.call_depth, 0);
        drop(root);
        assert_eq!(
            realm.collect(usize::MAX).unwrap().live,
            crate::test_support::REALM_ENTRIES + 2
        );
    }

    #[test]
    fn constructor_abrupt_completions_restore_scope_strictness_and_depth() {
        let mut realm = Realm::default();
        realm
            .eval("let effect=0;function F(){'use strict';throw 7;}")
            .unwrap();
        assert_eq!(realm.eval("new F"), Err(Error::Thrown(Value::Number(7.0))));
        assert_eq!(realm.call_depth, 0);
        assert_eq!(realm.evaluation_depth, 0);
        assert_eq!(realm.scopes.len(), 1);
        assert!(!realm.strict);
        assert_eq!(realm.eval("sloppy=7;sloppy"), Ok(Value::Number(7.0)));
        realm.eval("function G(){new G;}").unwrap();
        assert!(matches!(realm.eval("new G"), Err(Error::Limit { .. })));
        assert_eq!(realm.call_depth, 0);
        assert_eq!(realm.evaluation_depth, 0);
        assert_eq!(realm.scopes.len(), 1);
        assert_eq!(realm.eval("effect"), Ok(Value::Number(0.0)));
    }
}
