//! Lexical array-binding defaults through the shared Await expression evaluator.

use super::{AsyncAwaitAction, AsyncExecutionData, Step, binding, expression};
use crate::{
    Error, Realm, Value,
    environment::EnvironmentHandle,
    object::{
        async_binding_array::{
            ArrayBindingChild, ArrayBindingCursor, ArrayBindingRoot, AsyncArrayBinding,
        },
        async_execution::{AsyncStatementFrame, StatementLocation},
        async_expression::ExprRoot,
    },
};
use spite_core::Span;
use spite_parser::ast::{ArrowBody, BindingPattern, BindingPatternKind, Expr, ForBinding};

pub(super) fn has_array_binding_await(pattern: &BindingPattern) -> bool {
    match &pattern.kind {
        BindingPatternKind::Array { elements, rest } => {
            elements.iter().flatten().any(|element| {
                element
                    .initializer
                    .as_ref()
                    .is_some_and(expression::has_supported_await)
                    || has_array_binding_await(&element.pattern)
            }) || rest
                .as_ref()
                .is_some_and(|rest| has_array_binding_await(rest))
        }
        _ => false,
    }
}

fn pattern_at<'a>(
    body: &'a ArrowBody,
    location: Option<&StatementLocation>,
    frames: &[AsyncStatementFrame],
    root: ArrayBindingRoot,
    path: &[ArrayBindingChild],
) -> &'a BindingPattern {
    let mut pattern = match root {
        ArrayBindingRoot::Declaration(index) => {
            &binding::binding_source(body, location).0[index].pattern
        }
        ArrayBindingRoot::Iteration => {
            let ForBinding::Lexical { binding, .. } =
                super::iteration_head::iteration_binding(body, frames)
            else {
                unreachable!("lexical iteration array binding")
            };
            binding
        }
    };
    for child in path {
        let BindingPatternKind::Array { elements, rest } = &pattern.kind else {
            unreachable!("array binding path")
        };
        pattern = match child {
            ArrayBindingChild::Element(index) => {
                &elements[*index]
                    .as_ref()
                    .expect("array binding element")
                    .pattern
            }
            ArrayBindingChild::Rest => rest.as_ref().expect("array rest binding"),
        };
    }
    pattern
}

pub(super) fn default_expression(data: &AsyncExecutionData) -> &Expr {
    let state = data.array_binding.as_ref().expect("array binding default");
    let cursor = state.cursors.last().expect("array binding cursor");
    let pattern = pattern_at(
        &data.body,
        binding::binding_location(data),
        &data.frames,
        state.root,
        &cursor.path,
    );
    let BindingPatternKind::Array { elements, .. } = &pattern.kind else {
        unreachable!("array default source")
    };
    elements[state.pending_index.expect("pending default index")]
        .as_ref()
        .expect("default element")
        .initializer
        .as_ref()
        .expect("default initializer")
}

fn child_path(
    path: &[ArrayBindingChild],
    child: ArrayBindingChild,
    span: Span,
) -> Result<Vec<ArrayBindingChild>, Error> {
    let mut result = Vec::new();
    result
        .try_reserve(path.len() + 1)
        .map_err(|_| Error::Limit {
            span,
            message: "array binding path allocation failed".into(),
        })?;
    result.extend_from_slice(path);
    result.push(child);
    Ok(result)
}

impl Realm {
    pub(super) fn start_async_array_binding(
        &mut self,
        data: &mut AsyncExecutionData,
        root: ArrayBindingRoot,
        value: Value,
        environment: EnvironmentHandle,
    ) -> Result<Step, Error> {
        debug_assert!(data.array_binding.is_none());
        data.array_binding = Some(AsyncArrayBinding {
            root,
            environment,
            cursors: Vec::new(),
            pending_index: None,
        });
        self.push_async_array_binding(data, Vec::new(), value)?;
        Ok(Step::Continue)
    }

    fn push_async_array_binding(
        &mut self,
        data: &mut AsyncExecutionData,
        path: Vec<ArrayBindingChild>,
        value: Value,
    ) -> Result<(), Error> {
        let state = data.array_binding.as_ref().expect("array binding state");
        let pattern = pattern_at(
            &data.body,
            binding::binding_location(data),
            &data.frames,
            state.root,
            &path,
        );
        let BindingPatternKind::Array { rest, .. } = &pattern.kind else {
            unreachable!("array binding entry")
        };
        let span = pattern.span;
        let rest_pending = rest.is_some();
        let state = data.array_binding.as_mut().expect("array binding state");
        state.cursors.try_reserve(1).map_err(|_| Error::Limit {
            span,
            message: "array binding cursor allocation failed".into(),
        })?;
        self.tick(span)?;
        let iterator = self.binding_iterator(value, span)?;
        state.cursors.push(ArrayBindingCursor {
            path,
            iterator,
            index: 0,
            rest_pending,
            span,
        });
        Ok(())
    }

    fn initialize_async_array_node(
        &mut self,
        data: &mut AsyncExecutionData,
        path: Vec<ArrayBindingChild>,
        value: Value,
    ) -> Result<(), Error> {
        let state = data.array_binding.as_ref().expect("array binding state");
        let pattern = pattern_at(
            &data.body,
            binding::binding_location(data),
            &data.frames,
            state.root,
            &path,
        );
        if has_array_binding_await(pattern) {
            self.push_async_array_binding(data, path, value)
        } else {
            self.initialize_pattern(pattern, value, &state.environment)
        }
    }

    pub(super) fn advance_async_array_binding(
        &mut self,
        data: &mut AsyncExecutionData,
    ) -> Result<Step, Error> {
        loop {
            let state = data.array_binding.as_ref().expect("array binding state");
            let Some(cursor) = state.cursors.last() else {
                let root = data
                    .array_binding
                    .take()
                    .expect("completed array binding")
                    .root;
                return match root {
                    ArrayBindingRoot::Declaration(binding) => {
                        self.async_binding_step(data, binding + 1, None)
                    }
                    ArrayBindingRoot::Iteration => Ok(Step::Continue),
                };
            };
            let pattern = pattern_at(
                &data.body,
                binding::binding_location(data),
                &data.frames,
                state.root,
                &cursor.path,
            );
            let BindingPatternKind::Array { elements, rest } = &pattern.kind else {
                unreachable!("array binding source")
            };
            let span = cursor.span;
            if cursor.index < elements.len() {
                let index = cursor.index;
                self.tick(span)?;
                let cursor = data
                    .array_binding
                    .as_mut()
                    .expect("array binding state")
                    .cursors
                    .last_mut()
                    .expect("array cursor");
                cursor.index += 1;
                let Some(element) = &elements[index] else {
                    if !cursor.iterator.is_done() {
                        self.iterator_skip_value(&mut cursor.iterator, span)?;
                    }
                    continue;
                };
                let value = if cursor.iterator.is_done() {
                    Value::Undefined
                } else {
                    self.iterator_step_value(&mut cursor.iterator, span)?
                        .unwrap_or(Value::Undefined)
                };
                if matches!(value, Value::Undefined)
                    && element
                        .initializer
                        .as_ref()
                        .is_some_and(expression::has_supported_await)
                {
                    data.array_binding
                        .as_mut()
                        .expect("array binding state")
                        .pending_index = Some(index);
                    return self.start_async_expression(
                        data,
                        ExprRoot::ArrayBindingDefault,
                        AsyncAwaitAction::ArrayBindingDefault,
                    );
                }
                let value = self.binding_element_value(element, value)?;
                let cursor = data
                    .array_binding
                    .as_ref()
                    .expect("array binding state")
                    .cursors
                    .last()
                    .expect("array cursor");
                let path = child_path(&cursor.path, ArrayBindingChild::Element(index), span)?;
                self.initialize_async_array_node(data, path, value)?;
            } else if cursor.rest_pending {
                debug_assert!(rest.is_some());
                let cursor = data
                    .array_binding
                    .as_mut()
                    .expect("array binding state")
                    .cursors
                    .last_mut()
                    .expect("array cursor");
                cursor.rest_pending = false;
                let value = self.binding_array_rest(&mut cursor.iterator, span)?;
                let path = child_path(&cursor.path, ArrayBindingChild::Rest, span)?;
                self.initialize_async_array_node(data, path, value)?;
            } else {
                let cursor = data
                    .array_binding
                    .as_mut()
                    .expect("array binding state")
                    .cursors
                    .pop()
                    .expect("completed array cursor");
                if !cursor.iterator.is_done() {
                    self.iterator_close(&cursor.iterator, span)?;
                }
            }
        }
    }

    pub(super) fn resume_async_array_default(
        &mut self,
        data: &mut AsyncExecutionData,
        value: Value,
    ) -> Result<Step, Error> {
        let state = data
            .array_binding
            .as_mut()
            .expect("array binding default state");
        let index = state
            .pending_index
            .take()
            .expect("pending array default index");
        let cursor = state.cursors.last().expect("pending array cursor");
        let path = child_path(&cursor.path, ArrayBindingChild::Element(index), cursor.span)?;
        self.initialize_async_array_node(data, path, value)?;
        // The outer driver advances the pattern after this expression unwinds.
        Ok(Step::Continue)
    }

    pub(super) fn close_async_array_binding_error(
        &mut self,
        data: &mut AsyncExecutionData,
        mut error: Error,
    ) -> Error {
        let mut state = data.array_binding.take().expect("abandoned array binding");
        while let Some(cursor) = state.cursors.pop() {
            if !error.is_language_exception() {
                break;
            }
            if !cursor.iterator.is_done() {
                error = self.iterator_close_error(&cursor.iterator, error, cursor.span);
            }
        }
        error
    }
}
