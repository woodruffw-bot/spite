//! Indirect PerformEval and EvalDeclarationInstantiation (19.2.1.1–3).

use crate::{BindingState, CompletionKind, Error, ExceptionKind, Realm, Value, standard_global};
use spite_core::{DiagnosticKind, Span};
use spite_parser::{ast::Script, parse_script_utf16};
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
        if direct {
            return Err(Self::unsupported(
                span,
                "direct String eval is not implemented",
            ));
        }
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
        let script = parse_script_utf16(&source).map_err(|diagnostic| match diagnostic.kind {
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
        let lexical = self.object_work(span, |objects, budget| {
            objects.create_environment(Some(global.clone()), Default::default(), budget)
        })?;
        // Eval starts from GlobalEnv, never the caller's function/with chain.
        // Keep the caller's allowance and native-depth guards, restoring its
        // environment and strictness even after JavaScript or host failures.
        let caller = std::mem::replace(&mut self.scopes, vec![global, lexical]);
        let caller_strict = std::mem::replace(&mut self.strict, script.is_strict());
        let result = (|| {
            self.instantiate_indirect_eval(&script)?;
            let completion = self.statements(script.statements())?;
            debug_assert_eq!(completion.kind, CompletionKind::Normal);
            Ok(completion.value.unwrap_or(Value::Undefined))
        })();
        self.scopes = caller;
        self.strict = caller_strict;
        result
    }

    fn instantiate_indirect_eval(&mut self, script: &Script) -> Result<(), Error> {
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
        if !self.strict {
            let global = self.scopes.first().expect("global environment").clone();
            for (name, range) in variables
                .iter()
                .map(|binding| (binding.name, binding.span))
                .chain(functions.iter().map(|function| {
                    let name = function.name.as_ref().expect("named declaration");
                    (name.name.as_str(), name.span)
                }))
            {
                self.tick(range)?;
                if self
                    .objects
                    .environment(&global)
                    .expect("global environment")
                    .bindings
                    .contains_key(name)
                {
                    return Err(Self::exception(
                        ExceptionKind::SyntaxError,
                        range,
                        "eval var declaration conflicts with global lexical binding",
                    ));
                }
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
            // Complete every declaration check before creating global bindings.
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
        let lexical = self
            .scopes
            .last()
            .expect("eval lexical environment")
            .clone();
        for function in functions {
            let name = function.name.as_ref().expect("named declaration");
            let value = self.ordinary_function(function, false, name.span)?;
            if self.strict {
                self.objects
                    .environment_mut(&lexical)
                    .expect("eval lexical environment")
                    .bindings
                    .insert(
                        name.name.clone(),
                        BindingState {
                            value: Some(value),
                            mutable: true,
                            strict: false,
                        },
                    );
            } else {
                self.create_global_function_binding(&name.name, value, true, name.span)?;
            }
        }
        for binding in variables {
            if seen.contains(binding.name) {
                continue;
            }
            if self.strict {
                self.objects
                    .environment_mut(&lexical)
                    .expect("eval lexical environment")
                    .bindings
                    .entry(binding.name.to_owned())
                    .or_insert(BindingState {
                        value: Some(Value::Undefined),
                        mutable: true,
                        strict: false,
                    });
            } else {
                self.create_global_var_binding(binding.name, true, binding.span)?;
            }
        }
        Ok(())
    }
}
