//! A safe Rust ECMAScript parser and interpreter under development.
//!
//! Embedders must provide at least a 2 MiB native thread stack. The cleanup
//! example below shows how to create an explicit host thread with that stack.
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
//! # Lossless UTF-16 source
//!
//! A host can parse source containing an unpaired surrogate and evaluate the
//! validated Script through the same Realm entry point.
//!
//! ```
//! use spite::{JsString, Realm, Value, parse_module_utf16_with_source_limit,
//!             parse_script_utf16, parse_script_utf16_with_source_limit};
//!
//! let source = JsString::from_code_units(vec![0x22, 0xd800, 0x22]);
//! let script = parse_script_utf16_with_source_limit(&source, 5).unwrap();
//! assert_eq!(script, parse_script_utf16(&source).unwrap());
//! assert!(parse_module_utf16_with_source_limit(&source, 5).unwrap().body().is_strict());
//! assert_eq!(Realm::default().evaluate(&script).unwrap(),
//!            Value::String(JsString::from([0xd800])));
//! ```
//!
//! # Explicit cleanup turns
//!
//! The host ends its synchronous sequence, collects, and chooses when to run a
//! queued FinalizationRegistry cleanup job.
//!
//! ```
//! use spite::{FinalizationCleanupOutcome, Realm, Value};
//!
//! std::thread::Builder::new()
//!     .stack_size(2 * 1024 * 1024)
//!     .spawn(|| {
//!         let mut realm = Realm::default();
//!         realm.eval("let finalized=0, registry=new FinalizationRegistry(v=>finalized=v); registry.register({},7)").unwrap();
//!         realm.clear_kept_objects().unwrap();
//!         realm.collect(usize::MAX).unwrap();
//!         assert!(matches!(realm.run_finalization_cleanup_job().unwrap(),
//!                          FinalizationCleanupOutcome::Completed));
//!         assert_eq!(realm.eval("finalized").unwrap(), Value::Number(7.0));
//!         assert!(matches!(realm.run_finalization_cleanup_job().unwrap(),
//!                          FinalizationCleanupOutcome::NoJob));
//!     })
//!     .unwrap()
//!     .join()
//!     .unwrap();
//! ```
//!
//! Implementation status, conformance evidence and constraints are documented in the
//! repository. Unsupported features are not treated as JavaScript exceptions.

pub use spite_core::{Diagnostic, DiagnosticKind, JsString, Span};
pub use spite_parser::{
    ast, parse_dynamic_async_generator_function, parse_dynamic_async_generator_function_utf16,
    parse_module, parse_module_utf16, parse_module_utf16_with_source_limit,
    parse_module_with_source_limit, parse_script, parse_script_utf16,
    parse_script_utf16_with_source_limit, parse_script_with_source_limit,
};
pub use spite_runtime::{
    AgentControl, Collection, ConversionError, Error, ExceptionKind, FinalizationCleanupOutcome,
    Limits, ModuleHandle, ModuleImportRequest, ObjectHandle, PromiseJobOutcome, PromiseState,
    Realm, RootedValue, SharedBuffer, TimeZone, TimeZoneError, Value,
};
