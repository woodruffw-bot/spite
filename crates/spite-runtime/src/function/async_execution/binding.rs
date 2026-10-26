//! Shared source declaration and for-header initializer continuations.

use crate::object::async_binding_array::ArrayBindingRoot;

use super::{AsyncAwaitAction, AsyncExecutionData, Step, active_statement, expression};
use crate::{
    Completion, Error, Realm, Value,
    object::{
        async_execution::{AsyncForPhase, AsyncStatementFrame, StatementLocation},
        async_expression::ExprRoot,
    },
};
use spite_parser::ast::{
    ArrowBody, BindingElement, BindingPatternKind, ForInitializer, StatementKind,
};

pub(super) fn binding_location(data: &AsyncExecutionData) -> Option<&StatementLocation> {
    match data.frames.last() {
        Some(AsyncStatementFrame::For(cursor))
            if matches!(cursor.phase, AsyncForPhase::Initialize) =>
        {
            Some(&cursor.location)
        }
        _ => data.statement.as_ref(),
    }
}

pub(super) fn binding_source<'a>(
    body: &'a ArrowBody,
    location: Option<&StatementLocation>,
) -> (&'a [BindingElement], bool, bool) {
    match &active_statement(body, location).kind {
        StatementKind::Lexical { bindings, .. } => (bindings, false, false),
        StatementKind::Using(bindings) => (bindings, true, false),
        StatementKind::Var(bindings) => (bindings, false, true),
        StatementKind::For {
            initializer: Some(ForInitializer::Lexical { bindings, .. }),
            ..
        } => (bindings, false, false),
        StatementKind::For {
            initializer: Some(ForInitializer::Using(bindings)),
            ..
        } => (bindings, true, false),
        StatementKind::For {
            initializer: Some(ForInitializer::Var(bindings)),
            ..
        } => (bindings, false, true),
        _ => unreachable!("declaration continuation"),
    }
}

impl Realm {
    pub(super) fn async_binding_step(
        &mut self,
        data: &mut AsyncExecutionData,
        first: usize,
        mut resumed: Option<Value>,
    ) -> Result<Step, Error> {
        let (bindings, using, var) = binding_source(&data.body, binding_location(data));
        let environment = self.scopes.last().expect("body lexical scope").clone();
        for (element, binding) in bindings.iter().enumerate().skip(first) {
            if let Some(value) = resumed.take() {
                if var {
                    if let BindingPatternKind::Identifier(_) = &binding.pattern.kind {
                        let target = data.var_target.take().expect("resolved var target");
                        self.put(target, value, binding.pattern.span)?;
                    } else if super::binding_array::has_array_binding_await(&binding.pattern) {
                        return self.start_async_array_binding(
                            data,
                            ArrayBindingRoot::Declaration(element),
                            value,
                            None,
                        );
                    } else {
                        self.assign_pattern(&binding.pattern, value)?;
                    }
                } else if using {
                    // AddDisposableResource still precedes binding initialization.
                    self.initialize_using_value(&binding.pattern, value, &environment)?;
                } else {
                    if super::binding_array::has_array_binding_await(&binding.pattern) {
                        return self.start_async_array_binding(
                            data,
                            ArrayBindingRoot::Declaration(element),
                            value,
                            Some(environment),
                        );
                    }
                    self.initialize_pattern(&binding.pattern, value, &environment)?;
                }
            } else if binding
                .initializer
                .as_ref()
                .is_some_and(expression::has_supported_await)
            {
                self.tick(binding.pattern.span)?;
                if var && let BindingPatternKind::Identifier(name) = &binding.pattern.kind {
                    data.var_target = Some(
                        self.resolve(name, binding.pattern.span)?
                            .into_owned(binding.pattern.span)?,
                    );
                }
                return self.start_async_expression(
                    data,
                    ExprRoot::Binding { element },
                    AsyncAwaitAction::Binding { element },
                );
            } else if !using && super::binding_array::has_array_binding_await(&binding.pattern) {
                self.tick(binding.pattern.span)?;
                let value = self.binding_element_value(binding, Value::Undefined)?;
                return self.start_async_array_binding(
                    data,
                    ArrayBindingRoot::Declaration(element),
                    value,
                    if var { None } else { Some(environment) },
                );
            } else if var {
                self.evaluate_var_bindings(std::slice::from_ref(binding))?;
            } else if using {
                self.initialize_using_bindings(std::slice::from_ref(binding))?;
            } else {
                self.initialize_bindings(std::slice::from_ref(binding))?;
            }
        }
        Ok(Step::Complete(Completion::normal(None)))
    }
}
