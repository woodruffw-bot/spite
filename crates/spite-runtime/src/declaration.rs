//! Lexical and global declaration instantiation.

use crate::environment::BindingState;
use crate::{Error, ExceptionKind, Realm, Value};
use spite_core::{JsString, Span};
use spite_parser::ast::{BindingElement, BindingPatternKind, Script, Statement, StatementKind};
use std::collections::{BTreeMap, BTreeSet};

impl Realm {
    pub(crate) fn push_scope(
        &mut self,
        bindings: BTreeMap<String, BindingState>,
        span: Span,
    ) -> Result<(), Error> {
        let outer = self.scopes.last().cloned();
        let scope = self.object_work(span, |objects, budget| {
            objects.create_environment(outer, bindings, budget)
        })?;
        self.scopes.push(scope);
        Ok(())
    }

    pub(crate) fn check_lexical_conflicts<'a>(
        &mut self,
        statements: impl Iterator<Item = &'a Statement>,
        global: bool,
        functions_lexical: bool,
    ) -> Result<(), Error> {
        let handle = self.scopes.last().expect("active environment").clone();
        for statement in statements {
            for (name, span, _) in lexical_declarations(statement, functions_lexical) {
                if self
                    .objects
                    .environment(&handle)
                    .expect("active environment")
                    .bindings
                    .contains_key(name)
                    || (global && self.restricted_global_property(name, span)?)
                {
                    return Err(Self::exception(
                        ExceptionKind::SyntaxError,
                        span,
                        "conflicting lexical declaration",
                    ));
                }
            }
        }
        Ok(())
    }

    pub(crate) fn instantiate<'a>(
        &mut self,
        statements: impl DoubleEndedIterator<Item = &'a Statement> + Clone,
        global: bool,
        functions_lexical: bool,
    ) -> Result<(), Error> {
        if !global {
            self.check_lexical_conflicts(statements.clone(), false, functions_lexical)?;
        }
        let handle = self.scopes.last().expect("active environment").clone();
        let scope = &mut self
            .objects
            .environment_mut(&handle)
            .expect("active environment")
            .bindings;
        for statement in statements.clone() {
            for (name, _, mutable) in lexical_declarations(statement, functions_lexical) {
                scope.insert(
                    name.to_owned(),
                    BindingState {
                        value: None,
                        mutable,
                        deletable: false,
                        strict: true,
                    },
                );
            }
        }
        if functions_lexical {
            self.initialize_functions(
                statements.filter_map(|statement| match &statement.kind {
                    StatementKind::Function(function) if function.name.is_some() => {
                        Some(function.as_ref())
                    }
                    _ => None,
                }),
                Some(&handle),
            )?;
        }
        Ok(())
    }

    pub(crate) fn initialize_bindings(&mut self, bindings: &[BindingElement]) -> Result<(), Error> {
        let environment = self.scopes.last().expect("active environment").clone();
        for binding in bindings {
            self.tick(binding.pattern.span)?;
            self.initialize_binding_element(binding, Value::Undefined, &environment)?;
        }
        Ok(())
    }

    pub(crate) fn instantiate_global(&mut self, script: &Script) -> Result<(), Error> {
        // ECMA-262 16.1.7: check conflicts before creating any new bindings.
        // All Script vars are instantiated, including vars in unreachable code.
        self.check_lexical_conflicts(script.statements().iter(), true, false)?;
        let functions = script.function_declarations();
        let function_names: BTreeSet<_> = functions
            .iter()
            .map(|f| f.name.as_ref().expect("named declaration").name.as_str())
            .collect();
        let declarations = script.var_declarations();
        for binding in &declarations {
            self.tick(binding.span)?;
            if self
                .objects
                .environment(&self.scopes[0])
                .expect("global environment")
                .bindings
                .contains_key(binding.name)
            {
                return Err(Self::exception(
                    ExceptionKind::SyntaxError,
                    binding.span,
                    "var declaration conflicts with global lexical binding",
                ));
            }
            if self.record.unsupported_host_globals.contains(binding.name)
                && self.global_own(binding.name, binding.span)?.is_none()
                && !function_names.contains(binding.name)
            {
                return Err(Self::unsupported(
                    binding.span,
                    format!("{} is not implemented", binding.name),
                ));
            }
        }
        for function in &functions {
            let name = function.name.as_ref().expect("named declaration");
            self.tick(name.span)?;
            if self
                .objects
                .environment(&self.scopes[0])
                .expect("global environment")
                .bindings
                .contains_key(&name.name)
            {
                return Err(Self::exception(
                    ExceptionKind::SyntaxError,
                    name.span,
                    "function declaration conflicts with global lexical binding",
                ));
            }
        }
        for function in &functions {
            let name = function.name.as_ref().expect("named declaration");
            if !self.can_declare_global_function(&name.name, name.span)? {
                return Err(Self::exception(
                    ExceptionKind::TypeError,
                    name.span,
                    "cannot declare function over restricted global",
                ));
            }
        }
        for binding in &declarations {
            if !function_names.contains(binding.name)
                && !self.can_declare_global_var(binding.name, binding.span)?
            {
                return Err(Self::exception(
                    ExceptionKind::TypeError,
                    binding.span,
                    "cannot declare var on non-extensible global object",
                ));
            }
        }
        self.instantiate(script.statements().iter(), true, false)?;
        self.initialize_functions(functions.iter().copied(), None)?;
        for binding in declarations {
            if !function_names.contains(binding.name) {
                self.create_global_var(binding.name, binding.span)?;
            }
        }
        Ok(())
    }

    pub(crate) fn evaluate_var_bindings(
        &mut self,
        bindings: &[BindingElement],
    ) -> Result<(), Error> {
        for binding in bindings {
            self.tick(binding.pattern.span)?;
            // ECMA-262 14.3.2.1: a declaration without an initializer does not
            // assign. Identifier initializers resolve before the RHS; patterns
            // perform BindingInitialization only after evaluating their RHS.
            if let Some(expr) = &binding.initializer {
                if let BindingPatternKind::Identifier(name) = &binding.pattern.kind {
                    let reference = self.resolve(name, binding.pattern.span)?;
                    let value = self.named_expression(expr, JsString::from(name.as_str()))?;
                    self.put(reference, value, binding.pattern.span)?;
                } else {
                    let value = self.expression(expr)?;
                    self.assign_pattern(&binding.pattern, value)?;
                }
            }
        }
        Ok(())
    }
}

fn lexical_declarations(
    statement: &Statement,
    functions: bool,
) -> impl Iterator<Item = (&str, Span, bool)> {
    let (bindings, mutable) = match &statement.kind {
        StatementKind::Lexical { bindings, mutable } => (bindings.as_slice(), *mutable),
        StatementKind::Using(bindings) => (bindings.as_slice(), false),
        _ => (&[][..], true),
    };
    let function = match &statement.kind {
        StatementKind::Function(function) if functions => function.name.as_ref(),
        StatementKind::Class(class) => class.name.as_ref(),
        _ => None,
    };
    bindings
        .iter()
        .flat_map(|binding| binding.pattern.bound_names())
        .map(move |(name, span)| (name, span, mutable))
        .chain(function.map(|name| (name.name.as_str(), name.span, true)))
}
