//! A safe Rust ECMAScript parser and interpreter under development.
//!
//! # Example
//!
//! ```
//! use spite::{Realm, Value};
//!
//! let mut realm = Realm::default();
//! assert_eq!(realm.eval("let x = 6; x * 7").unwrap(), Value::Number(42.0));
//! assert_eq!(realm.eval("x = x + 1; x").unwrap(), Value::Number(7.0));
//! ```
//!
//! The supported subset and remaining conformance work are documented in the
//! repository. Unsupported features are not treated as JavaScript exceptions.

pub use spite_core::{Diagnostic, DiagnosticKind, JsString, Span};
pub use spite_parser::{ast, parse_script};
pub use spite_runtime::{Error, ExceptionKind, Limits, Realm, Value};
