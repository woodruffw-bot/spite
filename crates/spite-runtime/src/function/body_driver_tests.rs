//! Body drivers observe initialized calls and restore callers on every exit.

use super::{Callable, ordinary::OrdinaryInvocation};
use crate::{Error, Realm, Value, environment::EnvironmentHandle};
use spite_core::Span;
use spite_parser::ast::{ArrowBody, FunctionBody};

fn invoke<T>(
    realm: &mut Realm,
    source: &str,
    arguments: Vec<Value>,
    execute: impl FnOnce(&mut Realm, &FunctionBody) -> Result<T, Error>,
) -> Result<T, Error> {
    let Value::Object(callee) = realm.eval(source)? else {
        panic!("function");
    };
    let callable = realm
        .inspect_object(&callee)
        .unwrap()
        .callable()
        .unwrap()
        .clone();
    let span = Span::new(0, 0);
    match callable {
        Callable::Ordinary(code) => {
            let invocation = OrdinaryInvocation {
                code,
                callee,
                this: Value::Undefined,
                context: Default::default(),
                arguments: arguments.into_iter(),
            };
            realm
                .with_ordinary_call(invocation, span, execute)
                .map(|(value, constructor)| {
                    assert!(constructor.is_none());
                    value
                })
        }
        Callable::Arrow(code) => {
            realm.with_arrow_call(code, arguments.into_iter(), span, |realm, body| {
                let ArrowBody::Block(body) = body else {
                    panic!("block");
                };
                execute(realm, &body)
            })
        }
        _ => panic!("ordinary or arrow"),
    }
}

fn evaluate_and_capture(
    realm: &mut Realm,
    body: &FunctionBody,
) -> Result<(Value, EnvironmentHandle), Error> {
    assert!(realm.strict);
    let value = realm.function_body(body, Span::new(0, 0))?;
    Ok((value, realm.scopes.last().unwrap().clone()))
}

#[test]
fn drivers_return_owned_state_from_initialized_ordinary_and_arrow_contexts() {
    for source in [
        "(function(a){'use strict';let b=3;return a+b})",
        "a=>{'use strict';let b=3;return a+b}",
    ] {
        let mut realm = Realm::default();
        realm.eval("0").unwrap();
        let scopes = realm.scopes.clone();
        let variable = realm.variable_environment.clone();
        let strict = realm.strict;
        let (value, environment) = invoke(
            &mut realm,
            source,
            vec![Value::Number(4.0)],
            evaluate_and_capture,
        )
        .unwrap();
        assert_eq!(value, Value::Number(7.0));
        let retained = realm.objects.environment(&environment).unwrap();
        assert_eq!(retained.bindings["a"].value, Some(Value::Number(4.0)));
        assert_eq!(retained.bindings["b"].value, Some(Value::Number(3.0)));
        assert_eq!(realm.scopes, scopes);
        assert_eq!(realm.variable_environment, variable);
        assert_eq!(realm.strict, strict);
        assert_eq!(
            realm.eval("typeof a==='undefined'&&typeof b==='undefined'"),
            Ok(Value::Boolean(true))
        );
    }
}

#[test]
fn driver_and_parameter_failures_restore_caller_scopes_variables_and_strictness() {
    for source in [
        "(function(a){'use strict';return a})",
        "a=>{'use strict';return a}",
    ] {
        for host in [false, true] {
            let mut realm = Realm::default();
            realm.eval("0").unwrap();
            let caller = (
                realm.scopes.clone(),
                realm.variable_environment.clone(),
                realm.strict,
            );
            let result: Result<(), Error> = invoke(
                &mut realm,
                source,
                vec![Value::Number(4.0)],
                |realm, body| {
                    assert!(realm.strict);
                    realm.prepare_function_body(body, Span::new(0, 0))?;
                    Err(if host {
                        Realm::unsupported(Span::new(0, 0), "body driver test")
                    } else {
                        Error::Thrown(Value::Number(11.0))
                    })
                },
            );
            assert!(matches!(result, Err(Error::Unsupported { .. })) == host);
            if !host {
                assert_eq!(result, Err(Error::Thrown(Value::Number(11.0))));
            }
            assert_eq!(
                (realm.scopes, realm.variable_environment, realm.strict),
                caller
            );
        }
    }
    for source in [
        "(function(a=(()=>{throw 23})()){return a})",
        "(a=(()=>{throw 23})())=>{return a}",
    ] {
        let mut realm = Realm::default();
        realm.eval("0").unwrap();
        let caller = (
            realm.scopes.clone(),
            realm.variable_environment.clone(),
            realm.strict,
        );
        let result: Result<(), Error> = invoke(&mut realm, source, Vec::new(), |_, _| {
            panic!("parameter error precedes the body driver")
        });
        assert_eq!(result, Err(Error::Thrown(Value::Number(23.0))));
        assert_eq!(
            (realm.scopes, realm.variable_environment, realm.strict),
            caller
        );
    }
}
