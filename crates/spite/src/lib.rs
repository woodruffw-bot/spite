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
pub use spite_parser::{
    ast, parse_dynamic_async_generator_function, parse_dynamic_async_generator_function_utf16,
    parse_module, parse_module_utf16, parse_module_with_source_limit, parse_script,
    parse_script_with_source_limit,
};
pub use spite_runtime::{
    AgentControl, Collection, ConversionError, Error, ExceptionKind, Limits, ModuleHandle,
    ModuleImportRequest, ObjectHandle, PromiseState, Realm, RootedValue, SharedBuffer, TimeZone,
    TimeZoneError, Value,
};
