//! Destructuring assignment continuations share binding cursors and closing (13.15.5).

use super::{
    AsyncExecutionData, allocation_error, class, expression_at, has_supported_await, push,
};
use crate::{
    Error, Realm, Value,
    object::{
        async_assignment::{AssignmentPosition, AsyncAssignmentPattern},
        async_binding::{BindingChild, BindingCursorKind, ObjectBindingPhase, PatternLayout},
        async_expression::{AsyncExpressionData, ExprChild, ExprPath, ExprTask, MemberAction},
    },
};
use spite_core::{PropertyKey, Span};
use spite_parser::ast::{
    AssignmentElement, AssignmentPattern, AssignmentPatternKind, AssignmentTarget, Expr, ExprKind,
    PropertyName,
};

fn element(pattern: &AssignmentPattern, index: usize) -> &AssignmentElement {
    match &pattern.kind {
        AssignmentPatternKind::Array { elements, .. } => {
            elements[index].as_ref().expect("assignment element")
        }
        AssignmentPatternKind::Object { properties, .. } => &properties[index].element,
    }
}

#[derive(Clone, Copy)]
enum Target<'a> {
    Reference(&'a Expr),
    Pattern(&'a AssignmentPattern),
}

fn target(pattern: &AssignmentPattern, position: AssignmentPosition) -> Target<'_> {
    fn view(target: &AssignmentTarget) -> Target<'_> {
        match target {
            AssignmentTarget::Reference(value) => Target::Reference(value),
            AssignmentTarget::Pattern(pattern) => Target::Pattern(pattern),
        }
    }
    match position {
        AssignmentPosition::Element(index) => view(&element(pattern, index).target),
        AssignmentPosition::Rest => match &pattern.kind {
            AssignmentPatternKind::Array { rest, .. } => view(rest.as_ref().expect("array rest")),
            AssignmentPatternKind::Object { rest, .. } => {
                Target::Reference(rest.as_ref().expect("object rest"))
            }
        },
    }
}

pub(super) fn has_pattern_await(pattern: &AssignmentPattern) -> bool {
    fn target_await(target: &AssignmentTarget) -> bool {
        match target {
            AssignmentTarget::Reference(value) => has_supported_await(value),
            AssignmentTarget::Pattern(pattern) => has_pattern_await(pattern),
        }
    }
    let element_await = |element: &AssignmentElement| {
        target_await(&element.target)
            || element
                .initializer
                .as_ref()
                .is_some_and(has_supported_await)
    };
    match &pattern.kind {
        AssignmentPatternKind::Array { elements, rest } => {
            elements.iter().flatten().any(element_await) || rest.as_ref().is_some_and(target_await)
        }
        AssignmentPatternKind::Object { properties, rest } => {
            properties.iter().any(|property| {
                matches!(&property.key, PropertyName::Computed(key) if has_supported_await(key))
                    || element_await(&property.element)
            }) || rest.as_ref().is_some_and(|rest| has_supported_await(rest))
        }
    }
}

fn source_pattern<'a>(
    data: &'a AsyncExecutionData,
    record: &AsyncAssignmentPattern,
    path: &[BindingChild],
) -> &'a AssignmentPattern {
    let mut pattern = if direct_iteration(record) {
        super::super::iteration_head::iteration_pattern(data)
    } else {
        let ExprKind::DestructuringAssign { pattern, .. } =
            &expression_at(data, record.root, &record.source).kind
        else {
            unreachable!("destructuring source")
        };
        pattern.as_ref()
    };
    for child in path {
        let position = match child {
            BindingChild::Element(index) | BindingChild::Property(index) => {
                AssignmentPosition::Element(*index)
            }
            BindingChild::Rest => AssignmentPosition::Rest,
        };
        let Target::Pattern(nested) = target(pattern, position) else {
            unreachable!("nested assignment pattern")
        };
        pattern = nested;
    }
    pattern
}

fn direct_iteration(record: &AsyncAssignmentPattern) -> bool {
    matches!(record.root, super::ExprRoot::IterationPattern) && record.source.is_empty()
}

fn expression_path(
    record: &AsyncAssignmentPattern,
    child: ExprChild,
    span: Span,
) -> Result<ExprPath, Error> {
    let cursor = record.cursors.last().expect("assignment cursor");
    let length = record
        .source
        .len()
        .checked_add(cursor.path.len())
        .and_then(|n| n.checked_add(if direct_iteration(record) { 1 } else { 2 }))
        .ok_or_else(|| allocation_error(span))?;
    let mut path = Vec::new();
    path.try_reserve_exact(length)
        .map_err(|_| allocation_error(span))?;
    path.extend_from_slice(&record.source);
    if !direct_iteration(record) {
        path.push(ExprChild::AssignmentPattern);
    }
    path.extend(cursor.path.iter().map(|child| match child {
        BindingChild::Element(index) | BindingChild::Property(index) => {
            ExprChild::AssignmentTarget(*index)
        }
        BindingChild::Rest => ExprChild::AssignmentRest,
    }));
    path.push(child);
    Ok(path)
}

enum Next {
    Complete,
    Read,
    Write(Value),
    Reference(ExprPath),
    Evaluate {
        path: ExprPath,
        name: Option<PropertyKey>,
        key: bool,
        span: Span,
    },
    Continue,
}

impl Realm {
    pub(super) fn start_async_assignment_pattern(
        &mut self,
        data: &mut AsyncExecutionData,
        state: &mut AsyncExpressionData,
        path: ExprPath,
        value: Value,
    ) -> Result<(), Error> {
        let span = if matches!(state.root, super::ExprRoot::IterationPattern) && path.is_empty() {
            super::super::iteration_head::iteration_pattern(data).span
        } else {
            expression_at(data, state.root, &path).span
        };
        data.assignment_patterns
            .try_reserve(1)
            .map_err(|_| allocation_error(span))?;
        let record = AsyncAssignmentPattern {
            root: state.root,
            source: path,
            original: value.clone(),
            cursors: Vec::new(),
            pending: AssignmentPosition::Element(0),
            class_depth: data.expression_classes.len(),
        };
        data.assignment_patterns.push(record);
        let mut record = data.assignment_patterns.pop().expect("assignment record");
        let result = self.push_assignment_cursor(data, &mut record, Vec::new(), value);
        data.assignment_patterns.push(record);
        result?;
        push(state, ExprTask::PatternNext, span)
    }

    fn push_assignment_cursor(
        &mut self,
        data: &AsyncExecutionData,
        record: &mut AsyncAssignmentPattern,
        path: Vec<BindingChild>,
        value: Value,
    ) -> Result<(), Error> {
        let pattern = source_pattern(data, record, &path);
        let span = pattern.span;
        let layout = match &pattern.kind {
            AssignmentPatternKind::Array { rest, .. } => PatternLayout::Array {
                rest: rest.is_some(),
            },
            AssignmentPatternKind::Object { properties, rest } => PatternLayout::Object {
                properties: properties.len(),
                rest: rest.is_some(),
            },
        };
        record
            .cursors
            .try_reserve(1)
            .map_err(|_| allocation_error(span))?;
        record
            .cursors
            .push(self.new_pattern_cursor(path, value, layout, span)?);
        Ok(())
    }

    fn assignment_reference_step(
        &mut self,
        data: &AsyncExecutionData,
        record: &mut AsyncAssignmentPattern,
    ) -> Result<Next, Error> {
        let cursor = record.cursors.last().expect("assignment cursor");
        let pattern = source_pattern(data, record, &cursor.path);
        let Target::Reference(reference) = target(pattern, record.pending) else {
            return Ok(Next::Read);
        };
        if has_supported_await(reference) {
            let child = match record.pending {
                AssignmentPosition::Element(index) => ExprChild::AssignmentTarget(index),
                AssignmentPosition::Rest => ExprChild::AssignmentRest,
            };
            return Ok(Next::Reference(expression_path(
                record,
                child,
                reference.span,
            )?));
        }
        let reference = self.reference(reference)?.into_owned(reference.span)?;
        record
            .cursors
            .last_mut()
            .expect("assignment cursor")
            .reference = Some(reference);
        Ok(Next::Read)
    }

    fn next_assignment_step(
        &mut self,
        data: &AsyncExecutionData,
        record: &mut AsyncAssignmentPattern,
    ) -> Result<Next, Error> {
        loop {
            let Some(cursor) = record.cursors.last() else {
                return Ok(Next::Complete);
            };
            let pattern = source_pattern(data, record, &cursor.path);
            let span = pattern.span;
            match &pattern.kind {
                AssignmentPatternKind::Array { elements, .. } => {
                    let cursor = record.cursors.last_mut().expect("array cursor");
                    if cursor.index < elements.len() {
                        self.tick(span)?;
                        let index = cursor.index;
                        cursor.index += 1;
                        if elements[index].is_none() {
                            let BindingCursorKind::Array(iterator) = &mut cursor.kind else {
                                unreachable!("array cursor")
                            };
                            if !iterator.is_done() {
                                self.iterator_skip_value(iterator, span)?;
                            }
                            continue;
                        }
                        record.pending = AssignmentPosition::Element(index);
                        return self.assignment_reference_step(data, record);
                    }
                    if cursor.rest_pending {
                        record.pending = AssignmentPosition::Rest;
                        return self.assignment_reference_step(data, record);
                    }
                    let cursor = record.cursors.pop().expect("completed cursor");
                    let BindingCursorKind::Array(iterator) = cursor.kind else {
                        unreachable!("array cursor")
                    };
                    if !iterator.is_done() {
                        self.iterator_close(&iterator, span)?;
                    }
                }
                AssignmentPatternKind::Object { properties, .. } => {
                    let cursor = record.cursors.last_mut().expect("object cursor");
                    let BindingCursorKind::Object(object) = &mut cursor.kind else {
                        unreachable!("object cursor")
                    };
                    if matches!(object.phase, ObjectBindingPhase::Finish) {
                        let key = object.key.take().expect("completed property");
                        if cursor.rest_pending {
                            self.exclude_rest_key(&mut object.excluded, key, span)?;
                        }
                        cursor.index += 1;
                        object.phase = ObjectBindingPhase::Key;
                    }
                    if cursor.index < properties.len() {
                        record.pending = AssignmentPosition::Element(cursor.index);
                        let index = cursor.index;
                        let property = &properties[index];
                        if let PropertyName::Computed(key) = &property.key
                            && has_supported_await(key)
                        {
                            let path =
                                expression_path(record, ExprChild::AssignmentKey(index), key.span)?;
                            return Ok(Next::Evaluate {
                                path,
                                name: None,
                                key: true,
                                span: key.span,
                            });
                        }
                        let value = match &property.key {
                            PropertyName::Literal(literal) => self.literal_value(literal, span)?,
                            PropertyName::Computed(key) => self.expression(key)?,
                            PropertyName::Private(_) => unreachable!("public assignment key"),
                        };
                        object.key = Some(self.property_key(value, span)?);
                        object.phase = ObjectBindingPhase::Bind;
                        return self.assignment_reference_step(data, record);
                    }
                    if cursor.rest_pending {
                        record.pending = AssignmentPosition::Rest;
                        return self.assignment_reference_step(data, record);
                    }
                    record.cursors.pop();
                }
            }
        }
    }

    fn apply_assignment_step(
        &mut self,
        data: &mut AsyncExecutionData,
        state: &mut AsyncExpressionData,
        record: AsyncAssignmentPattern,
        next: Result<Next, Error>,
    ) -> Result<(), Error> {
        let span = record
            .cursors
            .last()
            .map_or(data.span, |cursor| cursor.span);
        if matches!(next, Ok(Next::Complete)) {
            state.current = Some(record.original);
            return Ok(());
        }
        data.assignment_patterns.push(record);
        match next? {
            Next::Read => push(state, ExprTask::PatternRead, span),
            Next::Write(value) => {
                state.current = Some(value);
                push(state, ExprTask::PatternWrite, span)
            }
            Next::Reference(path) => {
                self.begin_async_member(data, state, path, MemberAction::PatternReference)
            }
            Next::Evaluate {
                path,
                name,
                key,
                span,
            } => {
                let task = match name {
                    Some(name) => {
                        class::named_task(expression_at(data, state.root, &path), path, name)
                    }
                    None => ExprTask::Evaluate(path),
                };
                push(
                    state,
                    if key {
                        ExprTask::PatternKey
                    } else {
                        ExprTask::PatternWrite
                    },
                    span,
                )?;
                push(state, task, span)
            }
            Next::Continue => push(state, ExprTask::PatternNext, span),
            Next::Complete => unreachable!("completed record returned above"),
        }
    }

    pub(super) fn advance_async_assignment_pattern(
        &mut self,
        data: &mut AsyncExecutionData,
        state: &mut AsyncExpressionData,
    ) -> Result<(), Error> {
        let mut record = data.assignment_patterns.pop().expect("assignment record");
        let next = self.next_assignment_step(data, &mut record);
        self.apply_assignment_step(data, state, record, next)
    }

    pub(super) fn finish_async_assignment_key(
        &mut self,
        data: &mut AsyncExecutionData,
        state: &mut AsyncExpressionData,
        value: Value,
    ) -> Result<(), Error> {
        let mut record = data.assignment_patterns.pop().expect("assignment record");
        let span = record.cursors.last().expect("object cursor").span;
        let next = (|| {
            let key = self.property_key(value, span)?;
            let BindingCursorKind::Object(object) =
                &mut record.cursors.last_mut().expect("object cursor").kind
            else {
                unreachable!("object cursor")
            };
            object.key = Some(key);
            object.phase = ObjectBindingPhase::Bind;
            self.assignment_reference_step(data, &mut record)
        })();
        self.apply_assignment_step(data, state, record, next)
    }

    pub(super) fn read_async_assignment_pattern(
        &mut self,
        data: &mut AsyncExecutionData,
        state: &mut AsyncExpressionData,
    ) -> Result<(), Error> {
        let mut record = data.assignment_patterns.pop().expect("assignment record");
        let next = (|| {
            let cursor = record.cursors.last_mut().expect("assignment cursor");
            let span = cursor.span;
            let value = match record.pending {
                AssignmentPosition::Element(_) => match &mut cursor.kind {
                    BindingCursorKind::Array(iterator) => {
                        if iterator.is_done() {
                            Value::Undefined
                        } else {
                            self.iterator_step_value(iterator, span)?
                                .unwrap_or(Value::Undefined)
                        }
                    }
                    BindingCursorKind::Object(object) => {
                        object.phase = ObjectBindingPhase::Finish;
                        self.get_property_value(
                            &object.value,
                            object.key.as_ref().expect("property key"),
                            span,
                        )?
                    }
                },
                AssignmentPosition::Rest => {
                    cursor.rest_pending = false;
                    match &mut cursor.kind {
                        BindingCursorKind::Array(iterator) => self.iterator_rest_array(
                            iterator,
                            span,
                            "assignment array capacity exceeded",
                        )?,
                        BindingCursorKind::Object(object) => {
                            self.copy_rest_object(object.value.clone(), &object.excluded, span)?
                        }
                    }
                }
            };
            if matches!(value, Value::Undefined)
                && let AssignmentPosition::Element(index) = record.pending
            {
                let pattern = source_pattern(
                    data,
                    &record,
                    &record.cursors.last().expect("assignment cursor").path,
                );
                if let Some(initializer) = &element(pattern, index).initializer {
                    if has_supported_await(initializer) {
                        let path = expression_path(
                            &record,
                            ExprChild::AssignmentDefault(index),
                            initializer.span,
                        )?;
                        let name = match target(pattern, record.pending) {
                            Target::Reference(reference) => match &reference.kind {
                                ExprKind::Identifier(name) => {
                                    Some(PropertyKey::from(name.as_str()))
                                }
                                _ => None,
                            },
                            Target::Pattern(_) => None,
                        };
                        return Ok(Next::Evaluate {
                            path,
                            name,
                            key: false,
                            span: initializer.span,
                        });
                    }
                    let value = match target(pattern, record.pending) {
                        Target::Reference(reference) => {
                            self.assignment_expression(reference, initializer)?
                        }
                        Target::Pattern(_) => self.expression(initializer)?,
                    };
                    return Ok(Next::Write(value));
                }
            }
            Ok(Next::Write(value))
        })();
        self.apply_assignment_step(data, state, record, next)
    }

    pub(super) fn write_async_assignment_pattern(
        &mut self,
        data: &mut AsyncExecutionData,
        state: &mut AsyncExpressionData,
        value: Value,
    ) -> Result<(), Error> {
        let mut record = data.assignment_patterns.pop().expect("assignment record");
        let next = (|| {
            let cursor = record.cursors.last().expect("assignment cursor");
            let span = cursor.span;
            let pattern = source_pattern(data, &record, &cursor.path);
            match target(pattern, record.pending) {
                Target::Reference(reference) => {
                    let captured = record
                        .cursors
                        .last_mut()
                        .expect("assignment cursor")
                        .reference
                        .take()
                        .expect("captured assignment reference");
                    self.put(captured, value, reference.span)?;
                }
                Target::Pattern(_) => {
                    let child = match record.pending {
                        AssignmentPosition::Element(index) => match cursor.kind {
                            BindingCursorKind::Array(_) => BindingChild::Element(index),
                            BindingCursorKind::Object(_) => BindingChild::Property(index),
                        },
                        AssignmentPosition::Rest => BindingChild::Rest,
                    };
                    let path = crate::function::async_execution::binding_pattern::child_path(
                        &cursor.path,
                        child,
                        span,
                    )?;
                    self.push_assignment_cursor(data, &mut record, path, value)?;
                }
            }
            Ok(Next::Continue)
        })();
        self.apply_assignment_step(data, state, record, next)
    }
}
