//! PerformEval and EvalDeclarationInstantiation (19.2.1.1–3).

use crate::{
    BindingState, CompletionKind, Error, ExceptionKind, Realm, Reference, Value,
    environment::EnvironmentHandle, standard_global,
};
use spite_core::{DiagnosticKind, Span};
use spite_parser::{EvalContext, ast::Script, parse_eval_utf16};
use std::collections::BTreeSet;

impl Realm {
    pub(super) fn perform_eval(
        &mut self,
        value: Value,
        direct: bool,
        span: Span,
    ) -> Result<Value, Error> {
        // Non-String values precede compilation, quotas, and context creation.
        let Value::String(source) = value else {
            return Ok(value);
        };
        // Indirect calls already pass through call(); direct eval bypasses it.
        // Both forms must share the native-stack guard and caller's work budget.
        if direct {
            self.enter_call(span)?;
        }
        let result = self.eval_string(&source, direct, span);
        if direct {
            self.call_depth -= 1;
        }
        result
    }

    fn eval_string(
        &mut self,
        source: &spite_core::JsString,
        direct: bool,
        span: Span,
    ) -> Result<Value, Error> {
        let bytes = char::decode_utf16(source.code_units().iter().copied())
            .try_fold(0usize, |n, point| {
                n.checked_add(point.map_or(3, char::len_utf8))
            })
            .ok_or_else(|| Error::Limit {
                span,
                message: "eval source capacity exceeded".into(),
            })?;
        if self
            .limits
            .max_source_bytes
            .is_some_and(|limit| bytes > limit)
        {
            return Err(Error::Limit {
                span,
                message: "eval source size limit exceeded".into(),
            });
        }
        self.object_work(span, |_, budget| budget.charge(bytes))?;
        let mut context = EvalContext::default();
        if direct {
            context.strict = self.strict;
            // GetThisEnvironment skips declarative/with/arrow environments.
            let mut next = self.scopes.last().cloned();
            while let Some(environment) = next {
                let (function, method, outer) = self.object_work(span, |objects, budget| {
                    budget.charge(1)?;
                    let record = objects.environment(&environment)?;
                    Ok((
                        record.this.is_some(),
                        record.home_object.is_some(),
                        record.outer.clone(),
                    ))
                })?;
                if function {
                    context.in_function = true;
                    context.in_method = method;
                    break;
                }
                next = outer;
            }
        }
        let script =
            parse_eval_utf16(source, context).map_err(|diagnostic| match diagnostic.kind {
                DiagnosticKind::Syntax => {
                    Self::exception(ExceptionKind::SyntaxError, span, diagnostic.message)
                }
                DiagnosticKind::Unsupported => Self::unsupported(span, diagnostic.message),
                DiagnosticKind::Limit => Error::Limit {
                    span,
                    message: diagnostic.message,
                },
            })?;
        if script.statements().is_empty() {
            return Ok(Value::Undefined);
        }
        let global = self.scopes.first().expect("global environment").clone();
        let outer = if direct {
            self.scopes
                .last()
                .expect("caller lexical environment")
                .clone()
        } else {
            global.clone()
        };
        let lexical = self.object_work(span, |objects, budget| {
            objects.create_environment(Some(outer), Default::default(), budget)
        })?;
        let variable = if script.is_strict() {
            lexical.clone()
        } else if direct {
            self.variable_environment
                .as_ref()
                .expect("caller variable environment")
                .clone()
        } else {
            global.clone()
        };
        // The lexical chain retains the direct caller; indirect eval starts from
        // GlobalEnv. Restore all caller state after language and host failures.
        let caller = std::mem::replace(&mut self.scopes, vec![global, lexical]);
        let caller_strict = std::mem::replace(&mut self.strict, script.is_strict());
        let caller_variable = self.variable_environment.replace(variable.clone());
        let result = (|| {
            self.instantiate_eval(&script, &variable)?;
            let completion = self.statements(script.statements())?;
            debug_assert_eq!(completion.kind, CompletionKind::Normal);
            Ok(completion.value.unwrap_or(Value::Undefined))
        })();
        self.scopes = caller;
        self.strict = caller_strict;
        self.variable_environment = caller_variable;
        result
    }

    fn instantiate_eval(
        &mut self,
        script: &Script,
        variable: &EnvironmentHandle,
    ) -> Result<(), Error> {
        // Last declarations win; instantiate only their function objects.
        let mut seen = BTreeSet::new();
        let mut functions: Vec<_> = script
            .function_declarations()
            .into_iter()
            .rev()
            .filter(|function| {
                seen.insert(
                    function
                        .name
                        .as_ref()
                        .expect("named declaration")
                        .name
                        .as_str(),
                )
            })
            .collect();
        functions.reverse();
        let variables = script.var_declarations();
        let global = self.scopes.first().expect("global environment") == variable;
        let names: Vec<_> = variables
            .iter()
            .map(|binding| (binding.name, binding.span))
            .chain(functions.iter().map(|function| {
                let name = function.name.as_ref().expect("named declaration");
                (name.name.as_str(), name.span)
            }))
            .collect();
        if !self.strict {
            if global {
                self.check_eval_conflicts(variable, &names)?;
            }
            // No Annex B catch exception. Object Environment Records are skipped
            // without invoking getters, HasProperty, or Symbol.unscopables.
            let mut environment = self
                .scopes
                .last()
                .expect("eval lexical environment")
                .clone();
            while &environment != variable {
                let (object, outer) =
                    self.object_work(script.statements()[0].span, |objects, budget| {
                        budget.charge(1)?;
                        let record = objects.environment(&environment)?;
                        Ok((record.binding_object.is_some(), record.outer.clone()))
                    })?;
                if !object {
                    self.check_eval_conflicts(&environment, &names)?;
                }
                environment = outer.expect("variable environment is an ancestor");
            }
        }
        if global {
            // Complete every global check before creating any binding.
            for &(name, range) in &names {
                if (standard_global(name) || self.unsupported_host_globals.contains(name))
                    && self.global_own(name, range)?.is_none()
                    && !seen.contains(name)
                {
                    return Err(Self::unsupported(
                        range,
                        format!("{name} is not implemented"),
                    ));
                }
            }
            for function in &functions {
                let name = function.name.as_ref().expect("named declaration");
                if !self.can_declare_global_function(&name.name, name.span)? {
                    return Err(Self::exception(
                        ExceptionKind::TypeError,
                        name.span,
                        "cannot declare eval function over restricted global",
                    ));
                }
            }
            for binding in &variables {
                if !seen.contains(binding.name)
                    && !self.can_declare_global_var(binding.name, binding.span)?
                {
                    return Err(Self::exception(
                        ExceptionKind::TypeError,
                        binding.span,
                        "cannot declare eval var on non-extensible global object",
                    ));
                }
            }
        }
        self.instantiate(script.statements().iter(), false, false)?;
        for function in functions {
            let name = function.name.as_ref().expect("named declaration");
            let value = self.ordinary_function(function, false, name.span)?;
            if global {
                self.create_global_function_binding(&name.name, value, true, name.span)?;
            } else {
                let exists = self
                    .objects
                    .environment(variable)
                    .expect("eval variable environment")
                    .bindings
                    .contains_key(&name.name);
                if exists {
                    self.put(
                        Reference::Lexical(variable.clone(), &name.name),
                        value,
                        name.span,
                    )?;
                } else {
                    self.objects
                        .environment_mut(variable)
                        .expect("eval variable environment")
                        .bindings
                        .insert(
                            name.name.clone(),
                            BindingState {
                                value: Some(value),
                                mutable: true,
                                deletable: true,
                                strict: false,
                            },
                        );
                }
            }
        }
        for binding in variables {
            if seen.contains(binding.name) {
                continue;
            }
            if global {
                self.create_global_var_binding(binding.name, true, binding.span)?;
            } else {
                self.objects
                    .environment_mut(variable)
                    .expect("eval variable environment")
                    .bindings
                    .entry(binding.name.to_owned())
                    .or_insert(BindingState {
                        value: Some(Value::Undefined),
                        mutable: true,
                        deletable: true,
                        strict: false,
                    });
            }
        }
        Ok(())
    }

    fn check_eval_conflicts(
        &mut self,
        environment: &EnvironmentHandle,
        names: &[(&str, Span)],
    ) -> Result<(), Error> {
        for &(name, span) in names {
            self.tick(span)?;
            if self
                .objects
                .environment(environment)
                .expect("eval environment")
                .bindings
                .contains_key(name)
            {
                return Err(Self::exception(
                    ExceptionKind::SyntaxError,
                    span,
                    "eval var declaration conflicts with lexical binding",
                ));
            }
        }
        Ok(())
    }
}
