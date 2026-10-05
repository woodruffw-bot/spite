//! Base ClassDefinitionEvaluation and immutable internal names (15.7.14–16).

use crate::{Error, Realm, Value, environment::BindingState, object::DataDescriptor};
use spite_core::{JsString, PropertyKey};
use spite_parser::ast::{Class, Expr, ExprKind, PropertyName};
use std::collections::BTreeMap;

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
            let function_prototype = self
                .intrinsics
                .as_ref()
                .expect("initialized")
                .function_prototype
                .clone();
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
                objects.make_class_constructor(&function, &prototype)?;
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
            for element in &syntax.elements {
                let property = &element.property;
                self.tick(property.span)?;
                let key = match &property.name {
                    PropertyName::Literal(literal) => self.literal_value(literal, property.span)?,
                    PropertyName::Computed(expression) => self.expression(expression)?,
                };
                let key = self.property_key(key, property.span)?;
                let home = if element.is_static {
                    &function
                } else {
                    &prototype
                };
                self.define_method_property(home, key, property, false)?;
            }
            Ok(Value::Object(function))
        })();
        self.strict = previous_strict;
        self.scopes.pop();
        let value = result?;
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
                realm.objects.make_class_constructor(&function, &home),
                Err(expected)
            );
            assert!(matches!(
                realm.objects.inspect(&function).unwrap().callable(),
                Some(Callable::Ordinary(_))
            ));
        }
        assert_eq!(
            realm.objects.make_class_constructor(&method, &home),
            Err(ObjectError::WrongKind)
        );
        assert!(matches!(
            realm.objects.inspect(&method).unwrap().callable(),
            Some(Callable::Method(_))
        ));
        realm
            .objects
            .make_class_constructor(&function, &home)
            .unwrap();
        let Some(Callable::ClassConstructor(method)) =
            realm.objects.inspect(&function).unwrap().callable()
        else {
            panic!("class constructor")
        };
        assert_eq!(method.home_object, home);
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
