//! Owned lexical Array BindingInitialization state between default Await jobs.

use crate::{
    ObjectHandle, Reference, Value, environment::EnvironmentHandle, function::IteratorRecord,
};
use spite_core::{JsSymbol, Span};

#[derive(Clone, Copy, Debug)]
pub(crate) enum ArrayBindingChild {
    Element(usize),
    Rest,
}

#[derive(Debug)]
pub(crate) struct ArrayBindingCursor {
    pub path: Vec<ArrayBindingChild>,
    pub iterator: IteratorRecord,
    pub index: usize,
    pub rest_pending: bool,
    pub span: Span,
    pub reference: Option<Reference<'static>>,
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum ArrayBindingRoot {
    Declaration(usize),
    Iteration,
}

#[derive(Debug)]
pub(crate) struct AsyncArrayBinding {
    pub root: ArrayBindingRoot,
    pub environment: Option<EnvironmentHandle>,
    pub cursors: Vec<ArrayBindingCursor>,
    pub pending_index: Option<usize>,
}

impl AsyncArrayBinding {
    pub(super) fn values(&self) -> impl Iterator<Item = &Value> {
        self.cursors.iter().flat_map(|cursor| {
            std::iter::once(cursor.iterator.next_value())
                .chain(cursor.reference.iter().flat_map(Reference::values))
        })
    }
    pub(super) fn objects(&self) -> impl Iterator<Item = &ObjectHandle> {
        self.cursors.iter().flat_map(|cursor| {
            std::iter::once(cursor.iterator.iterator_handle())
                .chain(cursor.reference.iter().filter_map(Reference::object))
        })
    }
    pub(super) fn environments(&self) -> impl Iterator<Item = &EnvironmentHandle> {
        self.environment.iter().chain(
            self.cursors
                .iter()
                .filter_map(|cursor| cursor.reference.as_ref().and_then(Reference::environment)),
        )
    }
    pub(super) fn symbols(&self) -> impl Iterator<Item = &JsSymbol> {
        self.cursors
            .iter()
            .filter_map(|cursor| cursor.reference.as_ref())
            .flat_map(Reference::symbols)
    }
}
