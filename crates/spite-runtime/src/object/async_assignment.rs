//! Destructuring assignment state shares binding iterator/property cursors.

use super::{
    async_binding::BindingCursor,
    async_expression::{ExprPath, ExprRoot},
};
use crate::Value;

#[derive(Clone, Copy, Debug)]
pub(crate) enum AssignmentPosition {
    Element(usize),
    Rest,
}

#[derive(Debug)]
pub(crate) struct AsyncAssignmentPattern {
    pub root: ExprRoot,
    pub source: ExprPath,
    pub original: Value,
    pub cursors: Vec<BindingCursor>,
    pub pending: AssignmentPosition,
    /// Class expressions entered after this pattern must unwind before closing it.
    pub class_depth: usize,
}
