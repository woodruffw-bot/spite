//! Parse diagnostics with distinct syntax, support and resource categories.

use crate::Span;
use std::fmt;

/// The reason parsing could not produce a program.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DiagnosticKind {
    /// The source violates an implemented grammar rule or early error.
    Syntax,
    /// The source requires a feature that is not implemented.
    Unsupported,
    /// Parsing exceeded a host resource limit.
    Limit,
}

/// A source diagnostic. Unsupported features are not syntax errors.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Diagnostic {
    /// Diagnostic category.
    pub kind: DiagnosticKind,
    /// Source range associated with the error.
    pub span: Span,
    /// Human-readable explanation.
    pub message: String,
}

impl Diagnostic {
    /// Creates a diagnostic.
    pub fn new(kind: DiagnosticKind, span: Span, message: impl Into<String>) -> Self {
        Self {
            kind,
            span,
            message: message.into(),
        }
    }
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:?} at {}..{}: {}",
            self.kind, self.span.start, self.span.end, self.message
        )
    }
}

impl std::error::Error for Diagnostic {}
