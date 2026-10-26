//! Standalone Source Text Module environments and evaluation (16.2.1).

use crate::{BindingState, Error, Realm, Value, environment::ThisBinding};
use spite_core::Span;
use spite_parser::{
    ast::{ExportKind, Module},
    parse_module, parse_module_with_source_limit,
};
use std::collections::BTreeMap;

impl Realm {
    /// Parses and evaluates a fresh Module, returning its evaluation Promise.
    ///
    /// Drive pending awaits with [`Self::run_next_promise_job`]. Parsing/linking
    /// failures are returned directly; language evaluation failures reject the
    /// Promise. Returned values are unrooted; retain them with [`Self::root_value`]
    /// across collection. Evaluation never collects.
    ///
    /// Dependency loading and synthetic default exports remain unsupported.
    pub fn eval_module(&mut self, source: &str) -> Result<Value, Error> {
        let module = match self.limits.max_source_bytes {
            Some(limit) => parse_module_with_source_limit(source, limit),
            None => parse_module(source),
        }
        .map_err(Error::Parse)?;
        self.evaluate_module(&module)
    }

    /// Evaluates a fresh instance of a parsed Module in its own strict environment.
    ///
    /// See [`Self::eval_module`] for Promise handling and remaining boundaries.
    pub fn evaluate_module(&mut self, module: &Module) -> Result<Value, Error> {
        if !module.imports().is_empty()
            || module.exports().iter().any(|export| {
                matches!(
                    export.kind,
                    ExportKind::Star { .. }
                        | ExportKind::Named {
                            request: Some(_),
                            ..
                        }
                )
            })
        {
            return Err(Self::unsupported(
                Span::new(0, 0),
                "module dependency loading is not implemented",
            ));
        }
        if let Some(export) = module
            .exports()
            .iter()
            .find(|export| matches!(export.kind, ExportKind::Default { local: None, .. }))
        {
            return Err(Self::unsupported(
                export.span,
                "synthetic default export evaluation is not implemented",
            ));
        }
        self.initialize_realm()?;
        if !self.job_active {
            self.remaining_steps = self.limits.max_steps;
        }
        let body = module.body();
        let span = Span::new(
            0,
            body.statements()
                .last()
                .map_or(0, |statement| statement.span.end),
        );
        let global = self.scopes.first().expect("global environment").clone();
        let mut bindings = BTreeMap::new();
        for name in body.var_declarations() {
            self.tick(name.span)?;
            self.object_work(name.span, |_, budget| budget.charge(name.name.len() + 1))?;
            bindings
                .entry(name.name.to_owned())
                .or_insert(BindingState {
                    value: Some(Value::Undefined),
                    mutable: true,
                    deletable: false,
                    strict: true,
                });
        }
        let environment = self.object_work(span, |objects, budget| {
            let environment = objects.create_environment(Some(global.clone()), bindings, budget)?;
            objects.environment_mut(&environment)?.this = Some(ThisBinding::Module);
            Ok(environment)
        })?;
        self.evaluate_module_body(body, environment, span)
    }
}
