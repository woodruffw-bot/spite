//! Owned lexical Array BindingInitialization state between default Await jobs.

use crate::{ObjectHandle, Value, environment::EnvironmentHandle, function::IteratorRecord};
use spite_core::Span;

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
}

#[derive(Debug)]
pub(crate) struct AsyncArrayBinding {
    pub binding: usize,
    pub environment: EnvironmentHandle,
    pub cursors: Vec<ArrayBindingCursor>,
    pub pending_index: Option<usize>,
}

impl AsyncArrayBinding {
    pub(super) fn values(&self) -> impl Iterator<Item = &Value> {
        self.cursors
            .iter()
            .map(|cursor| cursor.iterator.next_value())
    }
    pub(super) fn objects(&self) -> impl Iterator<Item = &ObjectHandle> {
        self.cursors
            .iter()
            .map(|cursor| cursor.iterator.iterator_handle())
    }
}
