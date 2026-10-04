//! Expression-bodied arrow closures and function name inference.

use crate::{BindingState, Error, Realm, Value, environment::EnvironmentHandle};
use spite_core::{JsString, Span};
use spite_parser::ast::{Binding, Expr, ExprKind, FunctionSource};
use std::{collections::BTreeMap, rc::Rc};

#[derive(Clone, Debug)]
pub(crate) struct ArrowFunction {
    pub environment: EnvironmentHandle,
    pub parameters: Rc<[Binding]>,
    pub body: Rc<Expr>,
    pub source: FunctionSource,
    pub strict: bool,
}

impl Realm {
    pub(crate) fn arrow_function(
        &mut self,
        parameters: &Rc<[Binding]>,
        body: &Rc<Expr>,
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
        let arrow = ArrowFunction {
            environment: self.scopes.last().expect("active environment").clone(),
            parameters: parameters.clone(),
            body: body.clone(),
            source: source.clone(),
            strict: self.strict,
        };
        let function =
            self.object_work(span, |objects, _| objects.create_arrow(&prototype, arrow))?;
        self.define_builtin_property(
            &function,
            "length",
            Value::Number(parameters.len() as f64),
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
        arrow: ArrowFunction,
        mut arguments: std::vec::IntoIter<Value>,
        span: Span,
    ) -> Result<Value, Error> {
        let mut bindings = BTreeMap::new();
        for parameter in arrow.parameters.iter() {
            // Binding name copies are charged before allocation.
            self.object_work(parameter.span, |_, budget| {
                budget.charge(parameter.name.len() + 1)
            })?;
            bindings.insert(
                parameter.name.clone(),
                BindingState {
                    value: Some(arguments.next().unwrap_or(Value::Undefined)),
                    mutable: true,
                },
            );
        }
        let environment = self.object_work(span, |objects, budget| {
            objects.create_environment(Some(arrow.environment), bindings, budget)
        })?;
        let caller_strict = self.strict;
        let caller_depth = self.scopes.len();
        self.scopes.push(environment);
        self.strict = arrow.strict;
        let result = self.expression(&arrow.body);
        self.strict = caller_strict;
        self.scopes.truncate(caller_depth);
        result
    }

    pub(crate) fn named_expression(
        &mut self,
        expression: &Expr,
        name: JsString,
    ) -> Result<Value, Error> {
        let value = self.expression(expression)?;
        if anonymous_definition(expression) {
            let Value::Object(function) = &value else {
                unreachable!("anonymous function value")
            };
            let name = Value::String(name);
            self.check_string(&name, expression.span)?;
            self.define_builtin_property(function, "name", name, false, expression.span)?;
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
        assert_eq!(realm.expression_depth, 0);
        assert_eq!(realm.scopes.len(), 1);
    }
}
