//! Binding patterns through the shared Await expression evaluator.

use super::{AsyncAwaitAction, AsyncExecutionData, Step, active_statement, binding, expression};
use crate::{
    Error, Realm, Value,
    environment::EnvironmentHandle,
    object::{
        async_binding::{
            AsyncPatternBinding, BindingChild, BindingCursor, BindingCursorKind, BindingRoot,
            ObjectBindingCursor, ObjectBindingPhase,
        },
        async_execution::{AsyncStatementFrame, StatementLocation},
        async_expression::ExprRoot,
    },
};
use spite_core::Span;
use spite_parser::ast::{
    ArrowBody, BindingPattern, BindingPatternKind, Expr, ForBinding, PropertyName, StatementKind,
};

pub(super) fn has_binding_await(pattern: &BindingPattern) -> bool {
    match &pattern.kind {
        BindingPatternKind::Array { elements, rest } => {
            elements.iter().flatten().any(|element| {
                element
                    .initializer
                    .as_ref()
                    .is_some_and(expression::has_supported_await)
                    || has_binding_await(&element.pattern)
            }) || rest
                .as_ref()
                .is_some_and(|rest| has_binding_await(rest))
        }
        BindingPatternKind::Object { properties, rest } => {
            properties.iter().any(|property| {
                matches!(&property.key, PropertyName::Computed(key) if expression::has_supported_await(key))
                    || property.element.initializer.as_ref().is_some_and(expression::has_supported_await)
                    || has_binding_await(&property.element.pattern)
            }) || rest.as_ref().is_some_and(|rest| has_binding_await(rest))
        }
        BindingPatternKind::Identifier(_) => false,
    }
}

pub(super) fn pattern_at<'a>(
    body: &'a ArrowBody,
    location: Option<&StatementLocation>,
    frames: &[AsyncStatementFrame],
    root: BindingRoot,
    path: &[BindingChild],
) -> &'a BindingPattern {
    let mut pattern = match root {
        BindingRoot::Declaration(index) => {
            &binding::binding_source(body, location).0[index].pattern
        }
        BindingRoot::Catch => {
            let cursor = frames
                .iter()
                .rev()
                .find_map(|frame| match frame {
                    AsyncStatementFrame::Try(cursor) => Some(cursor),
                    _ => None,
                })
                .expect("catch binding source");
            let StatementKind::Try {
                handler: Some(handler),
                ..
            } = &active_statement(body, Some(&cursor.location)).kind
            else {
                unreachable!("catch parameter source")
            };
            handler.parameter.as_ref().expect("catch binding pattern")
        }
        BindingRoot::Iteration => match super::iteration_head::iteration_binding(body, frames) {
            ForBinding::Lexical { binding, .. } => binding,
            ForBinding::Var(binding) => &binding.pattern,
            _ => unreachable!("iteration array binding"),
        },
    };
    for child in path {
        pattern = match (&pattern.kind, child) {
            (BindingPatternKind::Array { elements, .. }, BindingChild::Element(index)) => {
                &elements[*index]
                    .as_ref()
                    .expect("array binding element")
                    .pattern
            }
            (BindingPatternKind::Object { properties, .. }, BindingChild::Property(index)) => {
                &properties[*index].element.pattern
            }
            (
                BindingPatternKind::Array { rest, .. } | BindingPatternKind::Object { rest, .. },
                BindingChild::Rest,
            ) => rest.as_ref().expect("rest binding"),
            _ => unreachable!("binding pattern path"),
        };
    }
    pattern
}

pub(super) fn default_expression(data: &AsyncExecutionData) -> &Expr {
    let state = data
        .pattern_binding
        .as_ref()
        .expect("array binding default");
    let cursor = state.cursors.last().expect("array binding cursor");
    let pattern = pattern_at(
        &data.body,
        binding::binding_location(data),
        &data.frames,
        state.root,
        &cursor.path,
    );
    let index = state.pending_index.expect("pending default index");
    let element = match &pattern.kind {
        BindingPatternKind::Array { elements, .. } => {
            elements[index].as_ref().expect("default element")
        }
        BindingPatternKind::Object { properties, .. } => &properties[index].element,
        BindingPatternKind::Identifier(_) => unreachable!("binding default source"),
    };
    element.initializer.as_ref().expect("default initializer")
}

pub(super) fn key_expression(data: &AsyncExecutionData) -> &Expr {
    let state = data.pattern_binding.as_ref().expect("binding key state");
    let cursor = state.cursors.last().expect("object binding cursor");
    let pattern = pattern_at(
        &data.body,
        binding::binding_location(data),
        &data.frames,
        state.root,
        &cursor.path,
    );
    let BindingPatternKind::Object { properties, .. } = &pattern.kind else {
        unreachable!("binding key source")
    };
    let PropertyName::Computed(key) = &properties[cursor.index].key else {
        unreachable!("computed binding key")
    };
    key
}

pub(super) fn child_path(
    path: &[BindingChild],
    child: BindingChild,
    span: Span,
) -> Result<Vec<BindingChild>, Error> {
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
    pub(super) fn start_async_pattern_binding(
        &mut self,
        data: &mut AsyncExecutionData,
        root: BindingRoot,
        value: Value,
        environment: Option<EnvironmentHandle>,
    ) -> Result<Step, Error> {
        debug_assert!(data.pattern_binding.is_none());
        data.pattern_binding = Some(AsyncPatternBinding {
            root,
            environment,
            cursors: Vec::new(),
            pending_index: None,
        });
        self.push_async_pattern_binding(data, Vec::new(), value)?;
        Ok(Step::Continue)
    }

    #[allow(clippy::mutable_key_type)]
    fn push_async_pattern_binding(
        &mut self,
        data: &mut AsyncExecutionData,
        path: Vec<BindingChild>,
        value: Value,
    ) -> Result<(), Error> {
        let state = data
            .pattern_binding
            .as_ref()
            .expect("pattern binding state");
        let pattern = pattern_at(
            &data.body,
            binding::binding_location(data),
            &data.frames,
            state.root,
            &path,
        );
        let rest_pending = match &pattern.kind {
            BindingPatternKind::Array { rest, .. } | BindingPatternKind::Object { rest, .. } => {
                rest.is_some()
            }
            BindingPatternKind::Identifier(_) => unreachable!("pattern binding entry"),
        };
        let span = pattern.span;
        let state = data
            .pattern_binding
            .as_mut()
            .expect("pattern binding state");
        state.cursors.try_reserve(1).map_err(|_| Error::Limit {
            span,
            message: "array binding cursor allocation failed".into(),
        })?;
        self.tick(span)?;
        let kind = match &pattern.kind {
            BindingPatternKind::Array { .. } => {
                BindingCursorKind::Array(self.binding_iterator(value, span)?)
            }
            BindingPatternKind::Object { properties, .. } => {
                Self::require_object_coercible(&value, span)?;
                let mut excluded = std::collections::HashSet::new();
                if rest_pending {
                    excluded
                        .try_reserve(properties.len())
                        .map_err(|_| Error::Limit {
                            span,
                            message: "binding property capacity exceeded".into(),
                        })?;
                }
                BindingCursorKind::Object(ObjectBindingCursor {
                    value,
                    key: None,
                    excluded,
                    phase: ObjectBindingPhase::Key,
                })
            }
            BindingPatternKind::Identifier(_) => unreachable!("pattern binding entry"),
        };
        state.cursors.push(BindingCursor {
            path,
            kind,
            index: 0,
            rest_pending,
            span,
            reference: None,
        });
        Ok(())
    }

    pub(super) fn initialize_async_pattern_node(
        &mut self,
        data: &mut AsyncExecutionData,
        path: Vec<BindingChild>,
        value: Value,
    ) -> Result<(), Error> {
        let state = data
            .pattern_binding
            .as_ref()
            .expect("pattern binding state");
        let pattern = pattern_at(
            &data.body,
            binding::binding_location(data),
            &data.frames,
            state.root,
            &path,
        );
        let environment = state.environment.clone();
        let reference = data
            .pattern_binding
            .as_mut()
            .expect("pattern binding state")
            .cursors
            .last_mut()
            .expect("binding reference cursor")
            .reference
            .take();
        if has_binding_await(pattern) {
            debug_assert!(reference.is_none());
            self.push_async_pattern_binding(data, path, value)
        } else {
            self.binding_initialization(pattern, value, environment.as_ref(), reference)
        }
    }

    pub(super) fn advance_async_pattern_binding(
        &mut self,
        data: &mut AsyncExecutionData,
    ) -> Result<Step, Error> {
        loop {
            let state = data
                .pattern_binding
                .as_ref()
                .expect("pattern binding state");
            let Some(cursor) = state.cursors.last() else {
                let root = data
                    .pattern_binding
                    .take()
                    .expect("completed array binding")
                    .root;
                return match root {
                    BindingRoot::Declaration(binding) => {
                        self.async_binding_step(data, binding + 1, None)
                    }
                    BindingRoot::Iteration | BindingRoot::Catch => Ok(Step::Continue),
                };
            };
            if matches!(&cursor.kind, BindingCursorKind::Object(_)) {
                if let Some(step) = self.advance_async_object_binding(data)? {
                    return Ok(step);
                }
                continue;
            }
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
                let environment = state.environment.clone();
                let cursor = data
                    .pattern_binding
                    .as_mut()
                    .expect("pattern binding state")
                    .cursors
                    .last_mut()
                    .expect("array cursor");
                cursor.index += 1;
                let BindingCursorKind::Array(iterator) = &mut cursor.kind else {
                    unreachable!("array binding cursor")
                };
                let Some(element) = &elements[index] else {
                    if !iterator.is_done() {
                        self.iterator_skip_value(iterator, span)?;
                    }
                    continue;
                };
                cursor.reference = self
                    .binding_reference(&element.pattern, environment.as_ref())?
                    .map(|reference| reference.into_owned(element.pattern.span))
                    .transpose()?;
                let value = if iterator.is_done() {
                    Value::Undefined
                } else {
                    self.iterator_step_value(iterator, span)?
                        .unwrap_or(Value::Undefined)
                };
                if matches!(value, Value::Undefined)
                    && element
                        .initializer
                        .as_ref()
                        .is_some_and(expression::has_supported_await)
                {
                    data.pattern_binding
                        .as_mut()
                        .expect("pattern binding state")
                        .pending_index = Some(index);
                    return self.start_async_expression(
                        data,
                        ExprRoot::BindingDefault,
                        AsyncAwaitAction::BindingDefault,
                    );
                }
                let value = self.binding_element_value(element, value)?;
                let cursor = data
                    .pattern_binding
                    .as_ref()
                    .expect("pattern binding state")
                    .cursors
                    .last()
                    .expect("array cursor");
                let path = child_path(&cursor.path, BindingChild::Element(index), span)?;
                self.initialize_async_pattern_node(data, path, value)?;
            } else if cursor.rest_pending {
                let rest = rest.as_ref().expect("array rest binding");
                let environment = state.environment.clone();
                let cursor = data
                    .pattern_binding
                    .as_mut()
                    .expect("pattern binding state")
                    .cursors
                    .last_mut()
                    .expect("array cursor");
                cursor.rest_pending = false;
                let BindingCursorKind::Array(iterator) = &mut cursor.kind else {
                    unreachable!("array rest cursor")
                };
                cursor.reference = self
                    .binding_reference(rest, environment.as_ref())?
                    .map(|reference| reference.into_owned(rest.span))
                    .transpose()?;
                let value = self.binding_array_rest(iterator, span)?;
                let path = child_path(&cursor.path, BindingChild::Rest, span)?;
                self.initialize_async_pattern_node(data, path, value)?;
            } else {
                let cursor = data
                    .pattern_binding
                    .as_mut()
                    .expect("pattern binding state")
                    .cursors
                    .pop()
                    .expect("completed array cursor");
                let BindingCursorKind::Array(iterator) = cursor.kind else {
                    unreachable!("completed array cursor")
                };
                if !iterator.is_done() {
                    self.iterator_close(&iterator, span)?;
                }
            }
        }
    }

    pub(super) fn resume_async_binding_default(
        &mut self,
        data: &mut AsyncExecutionData,
        value: Value,
    ) -> Result<Step, Error> {
        let state = data
            .pattern_binding
            .as_mut()
            .expect("array binding default state");
        let index = state
            .pending_index
            .take()
            .expect("pending array default index");
        let cursor = state.cursors.last().expect("pending array cursor");
        let child = match &cursor.kind {
            BindingCursorKind::Array(_) => BindingChild::Element(index),
            BindingCursorKind::Object(_) => BindingChild::Property(index),
        };
        let path = child_path(&cursor.path, child, cursor.span)?;
        self.initialize_async_pattern_node(data, path, value)?;
        // The outer driver advances the pattern after this expression unwinds.
        Ok(Step::Continue)
    }

    pub(super) fn close_async_pattern_binding_error(
        &mut self,
        data: &mut AsyncExecutionData,
        mut error: Error,
    ) -> Error {
        let mut state = data
            .pattern_binding
            .take()
            .expect("abandoned array binding");
        while let Some(mut cursor) = state.cursors.pop() {
            cursor.reference = None;
            if !error.is_language_exception() {
                break;
            }
            if let BindingCursorKind::Array(iterator) = cursor.kind
                && !iterator.is_done()
            {
                error = self.iterator_close_error(&iterator, error, cursor.span);
            }
        }
        error
    }
}
