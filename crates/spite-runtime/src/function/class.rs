//! ClassDefinitionEvaluation and immutable internal names (15.7.14–16).

use crate::{Error, Realm, Value, environment::BindingState, object::DataDescriptor};
use spite_core::{JsString, PropertyKey};
use spite_parser::ast::{Class, ClassElement, Expr, ExprKind, PropertyName};
use std::collections::BTreeMap;
use std::rc::Rc;

#[derive(Clone, Debug)]
pub(crate) struct ClassConstructor {
    pub method: super::MethodFunction,
    pub derived: bool,
    pub default: bool,
    pub fields: Rc<[super::ClassField]>,
}

impl Realm {
    pub(crate) fn class_definition(
        &mut self,
        syntax: &Class,
        inferred_name: Option<PropertyKey>,
    ) -> Result<Value, Error> {
        let span = syntax.source.span();
        self.ensure_object_intrinsics(span)?;
        let mut bindings = BTreeMap::new();
        if let Some(name) = &syntax.name {
            self.object_work(name.span, |_, budget| budget.charge(name.name.len() + 1))?;
            bindings.insert(
                name.name.clone(),
                BindingState {
                    value: None,
                    mutable: false,
                    deletable: false,
                    strict: true,
                },
            );
        }
        self.push_scope(bindings, span)?;
        let environment = self.scopes.last().expect("class environment").clone();
        let previous_strict = self.strict;
        self.strict = true;
        let result = (|| {
            let (function_prototype, prototype_parent) = if let Some(heritage) = &syntax.heritage {
                let superclass = self.expression(heritage)?;
                if matches!(superclass, Value::Null) {
                    (
                        self.intrinsics
                            .as_ref()
                            .expect("initialized")
                            .function_prototype
                            .clone(),
                        None,
                    )
                } else {
                    if !self.is_constructor(&superclass, heritage.span)? {
                        return Err(Self::exception(
                            crate::ExceptionKind::TypeError,
                            heritage.span,
                            "class superclass is not a constructor",
                        ));
                    }
                    let Value::Object(superclass) = superclass else {
                        unreachable!("constructor object")
                    };
                    let prototype = self.get_property(
                        &superclass,
                        &JsString::from("prototype"),
                        heritage.span,
                    )?;
                    let prototype = match prototype {
                        Value::Object(prototype) => Some(prototype),
                        Value::Null => None,
                        _ => {
                            return Err(Self::exception(
                                crate::ExceptionKind::TypeError,
                                heritage.span,
                                "class superclass prototype is not an object or null",
                            ));
                        }
                    };
                    (superclass, prototype)
                }
            } else {
                let intrinsics = self.intrinsics.as_ref().expect("initialized");
                (
                    intrinsics.function_prototype.clone(),
                    Some(intrinsics.object_prototype.clone()),
                )
            };
            let Value::Object(function) = self.allocate_ordinary_function(
                &syntax.constructor,
                environment.clone(),
                &function_prototype,
                true,
                span,
            )?
            else {
                unreachable!("ordinary function object")
            };
            let Value::Object(prototype) =
                self.get_property(&function, &JsString::from("prototype"), span)?
            else {
                unreachable!("new constructor prototype");
            };
            let changed = self.object_work(span, |objects, budget| {
                let changed =
                    objects.set_prototype(&prototype, prototype_parent.as_ref(), budget)?;
                debug_assert!(changed, "fresh class prototype");
                objects.make_class_constructor(
                    &function,
                    &prototype,
                    syntax.heritage.is_some(),
                    syntax.default_constructor,
                )?;
                objects.define(
                    &function,
                    JsString::from("prototype"),
                    DataDescriptor {
                        writable: Some(false),
                        ..Default::default()
                    },
                    budget,
                )
            })?;
            debug_assert!(changed, "new constructor prototype is writable");
            let name = syntax
                .name
                .as_ref()
                .map(|name| PropertyKey::from(name.name.as_str()))
                .or(inferred_name)
                .unwrap_or_else(|| PropertyKey::from(""));
            // SetFunctionName precedes every computed name and static method.
            self.set_function_name(&function, name, None, span)?;
            let mut instance_fields = Vec::new();
            let mut static_fields = Vec::new();
            for element in &syntax.elements {
                let span = element.span();
                self.tick(span)?;
                let key = match element.name() {
                    PropertyName::Literal(literal) => self.literal_value(literal, span)?,
                    PropertyName::Computed(expression) => self.expression(expression)?,
                };
                let key = self.property_key(key, span)?;
                let home = if element.is_static() {
                    &function
                } else {
                    &prototype
                };
                match element {
                    ClassElement::Method { property, .. } => {
                        self.define_method_property(home, key, property, false)?;
                    }
                    ClassElement::Field { initializer, .. } => {
                        let field = super::ClassField {
                            name: key,
                            initializer: initializer.as_ref().map(|expression| {
                                super::class_field::FieldInitializer {
                                    expression: expression.clone(),
                                    environment: environment.clone(),
                                    home_object: home.clone(),
                                }
                            }),
                            span,
                        };
                        let fields = if element.is_static() {
                            &mut static_fields
                        } else {
                            &mut instance_fields
                        };
                        self.object_work(span, |_, budget| budget.charge(1))?;
                        fields.try_reserve(1).map_err(|_| Error::Limit {
                            span,
                            message: "class field allocation capacity exceeded".into(),
                        })?;
                        fields.push(field);
                    }
                }
            }
            self.object_work(span, |objects, budget| {
                objects.set_class_fields(&function, instance_fields.into(), budget)
            })?;
            Ok((function, static_fields))
        })();
        self.strict = previous_strict;
        self.scopes.pop();
        let (function, static_fields) = result?;
        let value = Value::Object(function);
        // The name remains uninitialized during *all* computed names. Methods
        // capture this immutable binding, not the mutable declaration binding.
        if let Some(name) = &syntax.name {
            self.objects
                .environment_mut(&environment)
                .expect("class environment")
                .bindings
                .get_mut(&name.name)
                .expect("class name")
                .value = Some(value.clone());
        }
        // Initialize the internal class name before static initializers. All
        // computed names and method definitions have already completed (15.7.14).
        self.initialize_fields(&value, &static_fields)?;
        Ok(value)
    }

    pub(super) fn named_class_expression(
        &mut self,
        expression: &Expr,
        name: PropertyKey,
    ) -> Result<Value, Error> {
        // Keep the same guards and accounting as ordinary expression evaluation
        // while providing the name before ClassDefinitionEvaluation begins.
        self.enter_evaluation(expression.span)?;
        let result = self
            .tick(expression.span)
            .and_then(|()| match &expression.kind {
                ExprKind::Class(class) => self.class_definition(class, Some(name)),
                ExprKind::Parenthesized(inner) => self.named_class_expression(inner, name),
                _ => unreachable!("anonymous class definition"),
            });
        self.evaluation_depth -= 1;
        result
    }
}

pub(super) fn anonymous_class(expression: &Expr) -> bool {
    match &expression.kind {
        ExprKind::Class(class) => class.name.is_none(),
        ExprKind::Parenthesized(inner) => anonymous_class(inner),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ObjectHandle, function::Callable, object::Error as ObjectError};

    fn object(value: Value) -> ObjectHandle {
        let Value::Object(handle) = value else {
            panic!("object")
        };
        handle
    }

    #[test]
    fn marking_constructors_validates_home_and_function_before_mutating() {
        let mut realm = Realm::default();
        let function = object(realm.eval("let f=function(){};f").unwrap());
        let home = object(realm.eval("let home={};home").unwrap());
        let method = object(realm.eval("let m=({m(){}}).m;m").unwrap());
        let stale = object(realm.eval("({})").unwrap());
        realm.collect(usize::MAX).unwrap();
        let mut other = Realm::default();
        let foreign = object(other.eval("({})").unwrap());
        for (home, expected) in [
            (foreign, ObjectError::Heap(spite_heap::Error::ForeignHandle)),
            (stale, ObjectError::Heap(spite_heap::Error::StaleHandle)),
            (
                realm.scopes.last().unwrap().0.clone(),
                ObjectError::WrongKind,
            ),
        ] {
            assert_eq!(
                realm
                    .objects
                    .make_class_constructor(&function, &home, false, false),
                Err(expected)
            );
            assert!(matches!(
                realm.objects.inspect(&function).unwrap().callable(),
                Some(Callable::Ordinary(_))
            ));
        }
        assert_eq!(
            realm
                .objects
                .make_class_constructor(&method, &home, false, false),
            Err(ObjectError::WrongKind)
        );
        assert!(matches!(
            realm.objects.inspect(&method).unwrap().callable(),
            Some(Callable::Method(_))
        ));
        realm
            .objects
            .make_class_constructor(&function, &home, false, false)
            .unwrap();
        let Some(Callable::ClassConstructor(method)) =
            realm.objects.inspect(&function).unwrap().callable()
        else {
            panic!("class constructor")
        };
        assert_eq!(method.method.home_object, home);
    }

    #[test]
    fn derived_environment_checks_constructor_handles_before_allocation() {
        let mut realm = Realm::default();
        let constructor = object(realm.eval("class C extends Object{};C").unwrap());
        let non_constructor = object(realm.eval("let notConstructor={};notConstructor").unwrap());
        let stale = object(realm.eval("({})").unwrap());
        let before = realm.collect(usize::MAX).unwrap().live;
        let mut other = Realm::default();
        let foreign = object(other.eval("class C{};C").unwrap());
        for (derived, expected) in [
            (foreign, ObjectError::Heap(spite_heap::Error::ForeignHandle)),
            (stale, ObjectError::Heap(spite_heap::Error::StaleHandle)),
            (non_constructor, ObjectError::WrongKind),
        ] {
            let result = realm.objects.create_function_environment(
                realm.scopes.last().unwrap().clone(),
                BTreeMap::new(),
                Value::Undefined,
                crate::environment::FunctionContext {
                    new_target: Some(constructor.clone()),
                    derived_constructor: Some(derived),
                    ..Default::default()
                },
                &mut crate::object::Budget::new(100),
            );
            assert_eq!(result, Err(expected));
            assert_eq!(realm.collect(usize::MAX).unwrap().live, before);
        }
    }

    #[test]
    fn opt_in_class_name_limit_restores_outer_strictness_and_environment() {
        let mut realm = Realm::default();
        realm.eval("let flag=0;").unwrap();
        realm.limits.max_string_units = Some(8);
        assert!(matches!(
            realm.eval("try{class LongClassName{}}catch{flag=1;}finally{flag=2;}"),
            Err(Error::Limit { .. })
        ));
        realm.limits.max_string_units = None;
        assert_eq!(realm.eval("flag"), Ok(Value::Number(0.0)));
        assert_eq!(realm.eval("after=3"), Ok(Value::Number(3.0)));
        assert_eq!(
            realm.eval("try{LongClassName;}catch(e){e instanceof ReferenceError;}"),
            Ok(Value::Boolean(true))
        );
    }
}
