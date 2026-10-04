//! Ordinary function allocation, prototype properties, and declaration instantiation.

use super::ScriptFunction;
use crate::{
    BindingState, Error, GlobalBinding, Realm, Value, environment::EnvironmentHandle,
    object::DataDescriptor,
};
use spite_core::{JsString, Span};
use spite_parser::ast::{ArrowBody, Function};
use std::collections::{BTreeMap, BTreeSet};

impl Realm {
    pub(crate) fn ordinary_function(
        &mut self,
        syntax: &Function,
        expression: bool,
        span: Span,
    ) -> Result<Value, Error> {
        self.ensure_object_intrinsics(span)?;
        let intrinsics = self.intrinsics.as_ref().expect("initialized");
        let function_prototype = intrinsics.function_prototype.clone();
        let object_prototype = intrinsics.object_prototype.clone();
        let mut environment = self.scopes.last().expect("active environment").clone();
        // 15.2.5: a named expression captures a private immutable self binding.
        let local_name = syntax.name.as_ref().filter(|_| expression);
        if let Some(name) = local_name {
            self.object_work(name.span, |_, budget| budget.charge(name.name.len() + 1))?;
            let bindings = BTreeMap::from([(
                name.name.clone(),
                BindingState {
                    value: None,
                    mutable: false,
                    strict: false,
                },
            )]);
            environment = self.object_work(span, |objects, budget| {
                objects.create_environment(Some(environment), bindings, budget)
            })?;
        }
        let code = ScriptFunction {
            environment: environment.clone(),
            parameters: syntax.parameters.clone(),
            body: ArrowBody::Block(syntax.body.clone()),
            source: syntax.source.clone(),
            strict: self.strict || syntax.body.is_strict(),
        };
        let function = self.object_work(span, |objects, _| {
            objects.create_ordinary_function(&function_prototype, code)
        })?;
        let length = syntax
            .parameters
            .iter()
            .take_while(|parameter| parameter.initializer.is_none())
            .count();
        self.define_builtin_property(
            &function,
            "length",
            Value::Number(length as f64),
            false,
            span,
        )?;
        let name = syntax.name.as_ref().map_or("", |name| name.name.as_str());
        self.object_work(span, |_, budget| budget.charge(name.len() + 1))?;
        let name = Value::String(JsString::from(name));
        self.check_string(&name, span)?;
        self.define_builtin_property(&function, "name", name, false, span)?;
        // 10.2.5 MakeConstructor: every ordinary function owns its prototype;
        // the prototype's constructor points back to the same function object.
        let prototype =
            self.object_work(span, |objects, _| objects.create(Some(&object_prototype)))?;
        self.define_builtin_property(
            &prototype,
            "constructor",
            Value::Object(function.clone()),
            true,
            span,
        )?;
        self.object_work(span, |objects, budget| {
            objects.define(
                &function,
                JsString::from("prototype"),
                DataDescriptor {
                    value: Some(Value::Object(prototype)),
                    writable: Some(true),
                    enumerable: Some(false),
                    configurable: Some(false),
                },
                budget,
            )
        })?;
        let value = Value::Object(function);
        if let Some(name) = local_name {
            self.objects
                .environment_mut(&environment)
                .expect("function name environment")
                .bindings
                .get_mut(&name.name)
                .expect("local name exists")
                .value = Some(value.clone());
        }
        Ok(value)
    }

    pub(crate) fn initialize_functions<'a>(
        &mut self,
        functions: impl DoubleEndedIterator<Item = &'a Function>,
        target: Option<&EnvironmentHandle>,
    ) -> Result<(), Error> {
        // 10.2.11 / 16.1.7: use only the last declaration of each name, retaining
        // source order among the selected declarations.
        let mut names = BTreeSet::new();
        let mut selected = Vec::new();
        for function in functions.rev() {
            let name = function.name.as_ref().expect("named declaration");
            self.tick(name.span)?;
            if names.insert(name.name.as_str()) {
                selected.push(function);
            }
        }
        for function in selected.into_iter().rev() {
            let name = function.name.as_ref().expect("named declaration");
            let value = self.ordinary_function(function, false, name.span)?;
            if let Some(target) = target {
                self.objects
                    .environment_mut(target)
                    .expect("declaration environment")
                    .bindings
                    .get_mut(&name.name)
                    .expect("function binding exists")
                    .value = Some(value);
            } else {
                self.object_work(name.span, |_, budget| budget.charge(name.name.len() + 1))?;
                self.globals.insert(
                    name.name.clone(),
                    GlobalBinding {
                        value,
                        deletable: false,
                    },
                );
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ExceptionKind, Reference, function::Callable};

    #[test]
    fn named_expression_self_binding_ignores_sloppy_writes_but_rejects_strict_writes() {
        let mut realm = Realm::default();
        let value = realm.eval("(function local(){})").unwrap();
        let Value::Object(handle) = &value else {
            panic!()
        };
        let Some(Callable::Ordinary(code)) = realm.inspect_object(handle).unwrap().callable()
        else {
            panic!()
        };
        let environment = code.environment.clone();
        let span = Span::new(0, 0);
        realm
            .put(
                Reference::Lexical(environment.clone(), "local"),
                Value::Number(1.0),
                span,
            )
            .unwrap();
        assert_eq!(
            realm.get(&mut Reference::Lexical(environment.clone(), "local"), span),
            Ok(value.clone())
        );
        realm.strict = true;
        assert!(matches!(
            realm.put(
                Reference::Lexical(environment.clone(), "local"),
                Value::Number(1.0),
                span
            ),
            Err(Error::Exception {
                kind: ExceptionKind::TypeError,
                ..
            })
        ));
        assert_eq!(
            realm.get(&mut Reference::Lexical(environment, "local"), span),
            Ok(value)
        );
    }

    #[test]
    fn ordinary_function_metadata_captures_lexical_environment_and_strictness() {
        let mut realm = Realm::default();
        let Value::Object(handle)=realm.eval("(()=>{'use strict';let captured={value:7};function f(){return captured;}return f;})()").unwrap() else {panic!()};
        let Some(Callable::Ordinary(code)) = realm.inspect_object(&handle).unwrap().callable()
        else {
            panic!()
        };
        assert!(code.strict);
        let scope = realm.objects.environment(&code.environment).unwrap();
        assert!(matches!(
            scope.bindings["captured"].value,
            Some(Value::Object(_))
        ));
        assert_eq!(
            scope.bindings["f"].value,
            Some(Value::Object(handle.clone()))
        );
        assert_eq!(realm.scopes.len(), 1);
        assert_eq!(realm.call_depth, 0);
        assert_eq!(realm.evaluation_depth, 0);
    }
}
