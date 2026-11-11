//! JavaScript exceptions and distinct host failures.

use crate::Value;
use crate::object;
use spite_core::{Diagnostic, Span};
use std::fmt;

/// Built-in error categories produced by implemented runtime operations.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ExceptionKind {
    /// A runtime syntax failure, such as an invalid declaration or integer string.
    SyntaxError,
    /// An unresolvable or uninitialized binding.
    ReferenceError,
    /// An operation forbidden by a value or binding's type.
    TypeError,
    /// A numeric argument is outside an operation's domain.
    RangeError,
    /// URI encoding or decoding encountered malformed Unicode or escape syntax.
    URIError,
}

/// An evaluation failure. Host failures are distinct from JavaScript exceptions.
#[derive(Clone, Debug, PartialEq)]
pub enum Error {
    /// An embedding supplied a foreign, stale, or non-object handle.
    InvalidObject(object::Error),
    /// Parsing failed before execution began.
    Parse(Diagnostic),
    /// An implemented operation produced a built-in JavaScript error.
    Exception {
        /// Error category.
        kind: ExceptionKind,
        /// Associated source range.
        span: Span,
        /// Explanation.
        message: String,
    },
    /// A throw statement produced an uncaught value.
    Thrown(Value),
    /// Evaluation needs functionality that has not been implemented.
    Unsupported {
        /// Associated source range.
        span: Span,
        /// Explanation.
        message: String,
    },
    /// A host service or configuration failed independently of JavaScript.
    Host {
        /// Associated source range.
        span: Span,
        /// Explanation.
        message: String,
    },
    /// Evaluation exceeded a host resource limit.
    Limit {
        /// Associated source range.
        span: Span,
        /// Explanation.
        message: String,
    },
}

impl Error {
    pub(crate) fn is_language_exception(&self) -> bool {
        match self {
            Self::Thrown(_) | Self::Exception { .. } => true,
            Self::InvalidObject(_)
            | Self::Parse(_)
            | Self::Unsupported { .. }
            | Self::Host { .. }
            | Self::Limit { .. } => false,
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidObject(error) => write!(f, "invalid embedding object: {error}"),
            Self::Parse(diagnostic) => write!(f, "{diagnostic}"),
            Self::Exception {
                kind,
                span,
                message,
            } => write!(f, "{kind:?} at {}..{}: {message}", span.start, span.end),
            Self::Thrown(value) => write!(f, "uncaught {value}"),
            Self::Unsupported { span, message } => {
                write!(f, "Unsupported at {}..{}: {message}", span.start, span.end)
            }
            Self::Host { span, message } => {
                write!(f, "Host at {}..{}: {message}", span.start, span.end)
            }
            Self::Limit { span, message } => {
                write!(f, "Limit at {}..{}: {message}", span.start, span.end)
            }
        }
    }
}
impl std::error::Error for Error {}
