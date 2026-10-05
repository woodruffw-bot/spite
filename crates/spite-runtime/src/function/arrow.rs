//! Arrow closures, shared formal binding initialization, and function name inference.

use crate::{BindingState, CompletionKind, Error, Realm, Value, environment::EnvironmentHandle};
use spite_core::{JsString, Span};
use spite_parser::ast::{
    ArrowBody, Expr, ExprKind, FunctionBody, FunctionSource, Parameter, StatementKind,
};
use std::{
    collections::{BTreeMap, BTreeSet},
    rc::Rc,
};

#[derive(Clone, Debug)]
pub(crate) struct ScriptFunction {
    pub environment: EnvironmentHandle,
    pub parameters: Rc<[Parameter]>,
    pub body: ArrowBody,
    pub source: FunctionSource,
    pub strict: bool,
}

impl Realm {
    pub(crate) fn arrow_function(
        &mut self,
        parameters: &Rc<[Parameter]>,
        body: &ArrowBody,
        source: &FunctionSource,
        span: Span,
    ) -> Result<Value, Error> {
        self.ensure_object_intrinsics(span)?;
        let prototype = self
            .intrinsics
            .as_ref()
            .expect("initialized")
            .function_prototype
            .clone();
        let arrow = ScriptFunction {
            environment: self.scopes.last().expect("active environment").clone(),
            parameters: parameters.clone(),
            body: body.clone(),
            source: source.clone(),
            strict: self.strict || matches!(body, ArrowBody::Block(body) if body.is_strict()),
        };
        let function =
            self.object_work(span, |objects, _| objects.create_arrow(&prototype, arrow))?;
        self.define_builtin_property(
            &function,
            "length",
            Value::Number(
                parameters
                    .iter()
                    .take_while(|p| p.counts_toward_length())
                    .count() as f64,
            ),
            false,
            span,
        )?;
        self.define_builtin_property(
            &function,
            "name",
            Value::String(JsString::from("")),
            false,
            span,
        )?;
        Ok(Value::Object(function))
    }

    pub(super) fn call_arrow(
        &mut self,
        arrow: ScriptFunction,
        mut arguments: std::vec::IntoIter<Value>,
        span: Span,
    ) -> Result<Value, Error> {
        let bindings = self.pattern_bindings(
            arrow
                .parameters
                .iter()
                .map(|parameter| &parameter.binding().pattern),
            true,
        )?;
        let environment = self.object_work(span, |objects, budget| {
            objects.create_environment(Some(arrow.environment), bindings, budget)
        })?;
        let caller_strict = self.strict;
        let caller_depth = self.scopes.len();
        self.scopes.push(environment);
        self.strict = arrow.strict;
        let result = (|| {
            self.initialize_parameters(&arrow.parameters, &mut arguments)?;
            self.instantiate_function_vars(&arrow.parameters, &arrow.body, span)?;
            match &arrow.body {
                ArrowBody::Expression(body) => self.expression(body),
                ArrowBody::Block(body) => self.function_body(body, span),
            }
        })();
        self.strict = caller_strict;
        self.scopes.truncate(caller_depth);
        result
    }

    pub(super) fn initialize_parameters(
        &mut self,
        parameters: &[Parameter],
        arguments: &mut std::vec::IntoIter<Value>,
    ) -> Result<(), Error> {
        let environment = self.scopes.last().expect("parameter environment").clone();
        let mut names = BTreeSet::new();
        let mut duplicates = false;
        for (name, _) in parameters
            .iter()
            .flat_map(|parameter| parameter.binding().pattern.bound_names())
        {
            duplicates |= !names.insert(name);
        }
        // 10.2.11: duplicate names only occur in simple, sloppy lists. Their
        // bindings start at undefined and use assignment initialization.
        if duplicates {
            for name in names {
                self.objects
                    .environment_mut(&environment)
                    .expect("parameter environment")
                    .bindings
                    .get_mut(name)
                    .expect("parameter exists")
                    .value = Some(Value::Undefined);
            }
        }
        for parameter in parameters {
            let binding = parameter.binding();
            self.tick(binding.pattern.span)?;
            let value = if matches!(parameter, Parameter::Rest(_)) {
                self.create_array_from_list(arguments.by_ref(), binding.pattern.span)?
            } else {
                arguments.next().unwrap_or(Value::Undefined)
            };
            if duplicates {
                debug_assert!(parameter.is_simple());
                self.assign_pattern(&binding.pattern, value)?;
            } else {
                self.initialize_binding_element(binding, value, &environment)?;
            }
        }
        Ok(())
    }

    pub(super) fn instantiate_function_vars(
        &mut self,
        parameters: &[Parameter],
        body: &ArrowBody,
        span: Span,
    ) -> Result<(), Error> {
        let parameter_environment = self.scopes.last().expect("parameter environment").clone();
        let separate = parameters.iter().any(Parameter::contains_expression);
        if separate {
            // ECMA-262 10.2.11: pattern expressions cannot see body vars, even
            // through closures created by defaults or computed property keys.
            self.push_scope(BTreeMap::new(), span)?;
        }
        let environment = self.scopes.last().expect("var environment").clone();
        if let ArrowBody::Block(body) = body {
            let functions = body.function_declarations();
            let function_names: std::collections::BTreeSet<_> = functions
                .iter()
                .map(|f| f.name.as_ref().expect("named declaration").name.as_str())
                .collect();
            let variables = body.var_declarations();
            let declarations = variables
                .iter()
                .map(|binding| (binding.name, binding.span))
                .chain(functions.iter().map(|function| {
                    let name = function.name.as_ref().expect("named declaration");
                    (name.name.as_str(), name.span)
                }));
            for (name, span) in declarations {
                self.object_work(span, |_, budget| budget.charge(name.len() + 1))?;
                if self
                    .objects
                    .environment(&environment)
                    .expect("active environment")
                    .bindings
                    .contains_key(name)
                {
                    continue;
                }
                let value = if function_names.contains(name) {
                    Value::Undefined
                } else {
                    self.object_work(span, |objects, budget| {
                        let value = objects
                            .environment(&parameter_environment)?
                            .bindings
                            .get(name)
                            .and_then(|binding| binding.value.as_ref());
                        match value {
                            Some(value) => {
                                budget.value(value)?;
                                Ok(value.clone())
                            }
                            None => Ok(Value::Undefined),
                        }
                    })?
                };
                self.objects
                    .environment_mut(&environment)
                    .expect("active environment")
                    .bindings
                    .insert(
                        name.to_owned(),
                        BindingState {
                            value: Some(value),
                            mutable: true,
                            strict: true,
                        },
                    );
            }
        }
        Ok(())
    }

    pub(super) fn function_body(
        &mut self,
        body: &FunctionBody,
        span: Span,
    ) -> Result<Value, Error> {
        // ECMA-262 10.2.11: sloppy bodies have a separate lexical environment;
        // strict bodies reuse the parameter/var environment.
        let var_environment = self.scopes.last().expect("var environment").clone();
        if !self.strict {
            self.push_scope(BTreeMap::new(), span)?;
        }
        for statement in body.statements() {
            self.tick(statement.span)?;
            if let StatementKind::Lexical { bindings, .. } = &statement.kind {
                for (name, span) in bindings
                    .iter()
                    .flat_map(|binding| binding.pattern.bound_names())
                {
                    self.object_work(span, |_, budget| budget.charge(name.len() + 1))?;
                }
            }
        }
        self.instantiate(body.statements().iter(), false, false)?;
        self.initialize_functions(
            body.function_declarations().into_iter(),
            Some(&var_environment),
        )?;
        let completion = self.statements(body.statements())?;
        match completion.kind {
            CompletionKind::Return => Ok(completion.value.expect("return has a value")),
            CompletionKind::Normal => Ok(Value::Undefined),
            CompletionKind::Break | CompletionKind::Continue => {
                unreachable!("validated control target")
            }
        }
    }

    pub(crate) fn named_expression(
        &mut self,
        expression: &Expr,
        name: impl Into<spite_core::PropertyKey>,
    ) -> Result<Value, Error> {
        let name = name.into();
        let value = self.expression(expression)?;
        if anonymous_definition(expression) {
            let Value::Object(function) = &value else {
                unreachable!("anonymous function value")
            };
            self.set_function_name(function, name, None, expression.span)?;
        }
        Ok(value)
    }

    pub(crate) fn assignment_expression(
        &mut self,
        target: &Expr,
        expression: &Expr,
    ) -> Result<Value, Error> {
        // IsIdentifierRef is false for parenthesized assignment targets (8.4.4).
        if let ExprKind::Identifier(name) = &target.kind {
            self.named_expression(expression, JsString::from(name.as_str()))
        } else {
            self.expression(expression)
        }
    }
}

fn anonymous_definition(expression: &Expr) -> bool {
    match &expression.kind {
        ExprKind::Arrow { .. } => true,
        ExprKind::Function(function) => function.name.is_none(),
        ExprKind::Parenthesized(inner) => anonymous_definition(inner),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        ExceptionKind,
        object::{Budget, DescriptorKind, PropertyDescriptor},
    };

    #[test]
    fn user_getters_and_setters_execute_in_order_and_apply_snapshots_length_once() {
        let mut realm = Realm::default();
        realm.eval("let log=''; let list={length:2}; let get0=()=>(log+='0',list.length=0,1); let get1=()=>(log+='1',2); let setter=x=>(log+=x,999); let target=(a,b)=>(log+='t',a+b)").unwrap();
        let Value::Object(list) = realm.eval("list").unwrap() else {
            panic!("object")
        };
        for (name, get_name) in [("0", "get0"), ("1", "get1")] {
            let Value::Object(get) = realm.eval(get_name).unwrap() else {
                panic!("getter")
            };
            let Value::Object(set) = realm.eval("setter").unwrap() else {
                panic!("setter")
            };
            realm
                .objects
                .define(
                    &list,
                    JsString::from(name),
                    PropertyDescriptor {
                        kind: DescriptorKind::Accessor {
                            get: Some(Some(get)),
                            set: Some(Some(set)),
                        },
                        configurable: Some(true),
                        enumerable: Some(true),
                    },
                    &mut Budget::new(1000),
                )
                .unwrap();
        }
        assert_eq!(
            realm.eval("target.apply(null,list)"),
            Ok(Value::Number(3.0))
        );
        assert_eq!(realm.eval("log"), Ok(Value::String(JsString::from("01t"))));
        assert_eq!(
            realm.eval("list[0]='s'"),
            Ok(Value::String(JsString::from("s")))
        );
        assert_eq!(realm.eval("log"), Ok(Value::String(JsString::from("01ts"))));
        let Value::Object(thrower) = realm.eval("()=>missing").unwrap() else {
            panic!("getter")
        };
        realm
            .objects
            .define(
                &list,
                JsString::from("0"),
                PropertyDescriptor {
                    kind: DescriptorKind::Accessor {
                        get: Some(Some(thrower)),
                        set: None,
                    },
                    ..Default::default()
                },
                &mut Budget::new(1000),
            )
            .unwrap();
        assert!(matches!(
            realm.eval("log=''; list.length=2; target.apply(null,list)"),
            Err(Error::Exception {
                kind: ExceptionKind::ReferenceError,
                ..
            })
        ));
        assert_eq!(realm.eval("log"), Ok(Value::String(JsString::from(""))));
        assert_eq!(realm.call_depth, 0);
        assert_eq!(realm.evaluation_depth, 0);
        assert_eq!(realm.scopes.len(), 1);
    }
}
