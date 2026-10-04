//! Ordinary function allocation, prototype properties, and declaration instantiation.

use super::ScriptFunction;
use crate::{
    BindingState, Error, ObjectHandle, Realm, Value, environment::EnvironmentHandle,
    object::DataDescriptor,
};
use spite_core::{JsString, Span};
use spite_parser::ast::{ArrowBody, Function};
use std::collections::{BTreeMap, BTreeSet};

impl Realm {
    pub(super) fn call_ordinary(
        &mut self,
        code: ScriptFunction,
        callee: ObjectHandle,
        this: Value,
        new_target: Option<ObjectHandle>,
        mut arguments: std::vec::IntoIter<Value>,
        span: Span,
    ) -> Result<Value, Error> {
        let this = if code.strict {
            this
        } else {
            match this {
                Value::Undefined | Value::Null => Value::Object(self.global_object()),
                Value::Object(_) => this,
                _ => self.box_primitive(this, span)?,
            }
        };
        let ArrowBody::Block(body) = &code.body else {
            unreachable!("ordinary functions have block bodies")
        };
        let non_simple = code
            .parameters
            .iter()
            .any(|parameter| !parameter.is_simple());
        let has_parameter_expressions = code
            .parameters
            .iter()
            .any(|parameter| parameter.binding().initializer.is_some());
        let parameter_arguments = code
            .parameters
            .iter()
            .any(|parameter| parameter.binding().name == "arguments");
        let body_arguments = body
            .statements()
            .iter()
            .any(|statement| match &statement.kind {
                spite_parser::ast::StatementKind::Function(function) => function
                    .name
                    .as_ref()
                    .is_some_and(|name| name.name == "arguments"),
                spite_parser::ast::StatementKind::Lexical { bindings, .. } => {
                    bindings.iter().any(|binding| binding.name == "arguments")
                }
                _ => false,
            });
        // 10.2.11: body lexical/function names suppress arguments when parameters
        // have no expressions; a parameter named arguments always suppresses it.
        let arguments_needed =
            !parameter_arguments && (has_parameter_expressions || !body_arguments);
        let mut bindings = BTreeMap::new();
        for parameter in code.parameters.iter() {
            let parameter = parameter.binding();
            self.object_work(parameter.span, |_, budget| {
                budget.charge(parameter.name.len() + 1)
            })?;
            bindings.insert(
                parameter.name.clone(),
                BindingState {
                    value: None,
                    mutable: true,
                    strict: true,
                },
            );
        }
        if arguments_needed {
            bindings.insert(
                "arguments".into(),
                BindingState {
                    value: None,
                    mutable: !code.strict,
                    strict: code.strict,
                },
            );
        }
        // Sloppy parameter expressions get their own environment outside body vars.
        let separate_parameters = has_parameter_expressions && !code.strict;
        let parameters = if separate_parameters {
            std::mem::take(&mut bindings)
        } else {
            BTreeMap::new()
        };
        let environment = self.object_work(span, |objects, budget| {
            objects.create_function_environment(
                code.environment,
                bindings,
                this,
                new_target,
                budget,
            )
        })?;
        let caller_depth = self.scopes.len();
        let caller_strict = self.strict;
        self.scopes.push(environment);
        self.strict = code.strict;
        let result = (|| {
            if separate_parameters {
                self.push_scope(parameters, span)?;
            }
            let environment = self.scopes.last().expect("parameter environment").clone();
            if arguments_needed {
                let value = if code.strict || non_simple {
                    self.unmapped_arguments(arguments.as_slice(), span)?
                } else {
                    self.mapped_arguments(
                        &code.parameters,
                        arguments.as_slice(),
                        callee,
                        environment.clone(),
                        span,
                    )?
                };
                self.objects
                    .environment_mut(&environment)
                    .expect("parameter environment")
                    .bindings
                    .get_mut("arguments")
                    .expect("arguments binding")
                    .value = Some(value);
            }
            self.initialize_parameters(&code.parameters, &mut arguments)?;
            self.instantiate_function_vars(&code.parameters, &code.body, span)?;
            self.function_body(body, span)
        })();
        self.strict = caller_strict;
        self.scopes.truncate(caller_depth);
        result
    }

    pub(crate) fn this_value(&mut self, span: Span) -> Result<Value, Error> {
        let mut next = self.scopes.last().cloned();
        while let Some(environment) = next {
            let (value, outer) = self.object_work(span, |objects, budget| {
                let environment = objects.environment(&environment)?;
                let value = if let Some(value) = &environment.this {
                    budget.value(value)?;
                    Some(value.clone())
                } else {
                    None
                };
                Ok((value, environment.outer.clone()))
            })?;
            if let Some(value) = value {
                return Ok(value);
            }
            next = outer;
        }
        Ok(Value::Object(self.global_object()))
    }

    pub(crate) fn new_target_value(&mut self, span: Span) -> Result<Value, Error> {
        let mut next = self.scopes.last().cloned();
        while let Some(environment) = next {
            let (function, target, outer) = self.object_work(span, |objects, _| {
                let environment = objects.environment(&environment)?;
                Ok((
                    environment.this.is_some(),
                    environment.new_target.clone(),
                    environment.outer.clone(),
                ))
            })?;
            if function {
                return Ok(target.map_or(Value::Undefined, Value::Object));
            }
            next = outer;
        }
        unreachable!("new.target is validated inside a non-arrow function")
    }

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
            .take_while(|parameter| parameter.is_simple())
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
                self.create_global_function(&name.name, value, name.span)?;
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
