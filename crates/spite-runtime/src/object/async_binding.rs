//! Owned BindingInitialization state between computed-key and default Await jobs.

use crate::{
    ObjectHandle, Reference, Value, environment::EnvironmentHandle, function::IteratorRecord,
};
use spite_core::{JsSymbol, PropertyKey, Span};
use std::collections::HashSet;

#[derive(Clone, Copy, Debug)]
pub(crate) enum BindingChild {
    Element(usize),
    Property(usize),
    Rest,
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum ObjectBindingPhase {
    Key,
    Bind,
    Finish,
}

#[derive(Debug)]
pub(crate) struct ObjectBindingCursor {
    pub value: Value,
    pub key: Option<PropertyKey>,
    pub excluded: HashSet<PropertyKey>,
    pub phase: ObjectBindingPhase,
}

#[derive(Debug)]
pub(crate) enum BindingCursorKind {
    Array(IteratorRecord),
    Object(ObjectBindingCursor),
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum PatternLayout {
    Array { rest: bool },
    Object { properties: usize, rest: bool },
}

#[derive(Debug)]
pub(crate) struct BindingCursor {
    pub path: Vec<BindingChild>,
    pub kind: BindingCursorKind,
    pub index: usize,
    pub rest_pending: bool,
    pub span: Span,
    pub reference: Option<Reference<'static>>,
}

impl BindingCursor {
    pub(super) fn values(&self) -> impl Iterator<Item = &Value> {
        let value = match &self.kind {
            BindingCursorKind::Array(iterator) => iterator.next_value(),
            BindingCursorKind::Object(cursor) => &cursor.value,
        };
        std::iter::once(value).chain(self.reference.iter().flat_map(Reference::values))
    }
    pub(super) fn objects(&self) -> impl Iterator<Item = &ObjectHandle> {
        let iterator = match &self.kind {
            BindingCursorKind::Array(iterator) => Some(iterator.iterator_handle()),
            BindingCursorKind::Object(_) => None,
        };
        iterator
            .into_iter()
            .chain(self.reference.iter().filter_map(Reference::object))
    }
    pub(super) fn symbols(&self) -> impl Iterator<Item = &JsSymbol> {
        let object = match &self.kind {
            BindingCursorKind::Array(_) => None,
            BindingCursorKind::Object(cursor) => Some(cursor),
        };
        self.reference
            .iter()
            .flat_map(Reference::symbols)
            .chain(object.into_iter().flat_map(|cursor| {
                cursor
                    .key
                    .iter()
                    .chain(cursor.excluded.iter())
                    .filter_map(|key| match key {
                        PropertyKey::Symbol(symbol) => Some(symbol),
                        PropertyKey::String(_) => None,
                    })
            }))
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum BindingRoot {
    Declaration(usize),
    Iteration,
    Catch,
}

#[derive(Debug)]
pub(crate) struct AsyncPatternBinding {
    pub root: BindingRoot,
    pub environment: Option<EnvironmentHandle>,
    pub cursors: Vec<BindingCursor>,
    pub pending_index: Option<usize>,
}
