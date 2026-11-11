//! Shared ECMAScript source locations, diagnostics, strings, symbols and numeric utilities.

pub mod date;
mod diagnostic;
mod js_string;
mod number;
mod span;
mod symbol;
mod well_known;

pub use diagnostic::{Diagnostic, DiagnosticKind};
pub use js_string::JsString;
pub use number::parse_radix_integer;
pub use span::Span;
pub use spite_unicode::{
    UNICODE_VERSION, canonical_combining_class, canonical_composition, hangul_decomposition,
    is_identifier_part, is_identifier_start, is_line_terminator, is_unicode_case_ignorable,
    is_unicode_cased, is_whitespace, regexp_canonicalize_character, unicode_case_mapping,
    unicode_decomposition,
};
pub use symbol::{JsSymbol, PropertyKey, PropertyKeyRef, WeakJsSymbol};
#[doc(hidden)]
pub use symbol::{SymbolCollectionGuard, SymbolCollectionPin};
pub use well_known::WellKnownSymbol;
