//! A JavaScript parser and interpreter targeting ECMAScript 2026.
//!
//! This facade re-exports the parser, runtime, and their supporting types.
//! A [`Realm`] retains JavaScript state between evaluations. Resource quotas
//! are optional; see [`Limits`]. Language exceptions and host failures have
//! distinct [`Error`] variants.
//!
//! Embedders must provide a native thread stack of at least 2 MiB. The
//! [embedding guide](https://github.com/woodruffw-bot/spite/blob/main/docs/embedding.md)
//! covers job scheduling, rooting, collection, and module loading.
//!
//! # Evaluate JavaScript
//!
//! ```
//! use spite::{Realm, Value};
//!
//! std::thread::Builder::new()
//!     .stack_size(2 * 1024 * 1024)
//!     .spawn(|| {
//!         let mut realm = Realm::default();
//!         realm.eval("let answer = 40").expect("initialize answer");
//!         let value = realm.eval("answer += 2").expect("evaluate expression");
//!         assert_eq!(value, Value::Number(42.0));
//!     })
//!     .expect("create interpreter thread")
//!     .join()
//!     .expect("interpreter thread panicked");
//! ```
//!
//! # Preserve UTF-16 source
//!
//! JavaScript strings can contain unpaired surrogates, which Rust strings cannot
//! represent. Parse a [`JsString`] directly to preserve those code units:
//!
//! ```
//! use spite::{JsString, Realm, Value, parse_script_utf16};
//!
//! # std::thread::Builder::new().stack_size(2 * 1024 * 1024).spawn(|| {
//! // A string literal containing one unpaired surrogate.
//! let source = JsString::from_code_units(vec![0x0022, 0xd800, 0x0022]);
//! let script = parse_script_utf16(&source).expect("parse UTF-16 source");
//! let value = Realm::default().evaluate(&script).expect("evaluate script");
//! assert_eq!(value, Value::String(JsString::from([0xd800])));
//! # }).unwrap().join().unwrap();
//! ```
//!
//! # Run finalization work
//!
//! Collection and finalization are explicit host operations. End the synchronous
//! host sequence, collect unreachable objects, then run a queued cleanup job:
//!
//! ```
//! use spite::{FinalizationCleanupOutcome, Realm, Value};
//!
//! # std::thread::Builder::new().stack_size(2 * 1024 * 1024).spawn(|| {
//! let mut realm = Realm::default();
//! realm
//!     .eval(
//!         r#"
//!         let finalized = 0;
//!         let registry = new FinalizationRegistry(value => { finalized = value; });
//!         registry.register({}, 7);
//!         "#,
//!     )
//!     .expect("register finalization callback");
//!
//! realm.clear_kept_objects().expect("finish host sequence");
//! realm.collect(usize::MAX).expect("collect garbage");
//!
//! let outcome = realm.run_finalization_cleanup_job().expect("run cleanup job");
//! assert!(matches!(outcome, FinalizationCleanupOutcome::Completed));
//! assert_eq!(realm.eval("finalized").unwrap(), Value::Number(7.0));
//! # }).unwrap().join().unwrap();
//! ```
//!
//! Before collecting, retain any object-valued results with [`Realm::root_value`].
//! Cloning a [`Value`] alone does not register a root.

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
