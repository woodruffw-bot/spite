//! Shared source locations, diagnostics, strings, symbols, and property keys.

mod case;
mod case_data;
pub mod date;
mod normalization;
mod normalization_data;
mod regexp_escape;
mod replacement;
mod symbol;
mod unicode;
mod unicode_data;
mod well_known;

pub use case::{is_unicode_case_ignorable, is_unicode_cased, unicode_case_mapping};
pub use normalization::{
    canonical_combining_class, canonical_composition, hangul_decomposition, unicode_decomposition,
};
pub use regexp_escape::regexp_escape_units;
pub use replacement::{ReplacementPart, replacement_parts};
pub use symbol::{JsSymbol, PropertyKey, PropertyKeyRef, WeakJsSymbol};
pub use unicode::{is_identifier_part, is_identifier_start};
pub use unicode_data::UNICODE_VERSION;
pub use well_known::WellKnownSymbol;

use std::{fmt, sync::Arc};

/// A half-open range of encoded byte offsets in source text.
///
/// Scalar code points use UTF-8 lengths. In dynamically compiled UTF-16 source,
/// each unpaired surrogate occupies three bytes, preserving ordinary UTF-8 offsets.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Span {
    /// Inclusive start offset.
    pub start: usize,
    /// Exclusive end offset.
    pub end: usize,
}

impl Span {
    /// Creates a source range.
    pub const fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }
}

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

/// An ECMAScript string: a sequence of UTF-16 code units.
///
/// Lone surrogates are preserved. Conversion to a Rust string is fallible.
/// Immutable storage is shared across clones without copying code units.
#[derive(Clone, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct JsString(Arc<[u16]>);

impl JsString {
    /// Creates a string without validating surrogate pairing.
    pub fn from_code_units(units: Vec<u16>) -> Self {
        Self(units.into())
    }

    /// Returns the underlying UTF-16 code units.
    pub fn code_units(&self) -> &[u16] {
        &self.0
    }

    /// Returns the number of code units, as used by JavaScript string lengths.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns whether this string is empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Converts to UTF-8, rejecting unpaired surrogates.
    pub fn to_utf8(&self) -> Result<String, std::string::FromUtf16Error> {
        String::from_utf16(&self.0)
    }

    /// Concatenates two strings without interpreting surrogate pairs.
    pub fn concat(&self, other: &Self) -> Self {
        let mut units = self.code_units().to_vec();
        units.extend_from_slice(&other.0);
        Self(units.into())
    }
}

impl From<&str> for JsString {
    fn from(value: &str) -> Self {
        Self(value.encode_utf16().collect())
    }
}

impl fmt::Debug for JsString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("\"")?;
        for unit in self.code_units() {
            match *unit {
                0x20..=0x7e if *unit != 0x22 && *unit != 0x5c => {
                    write!(
                        f,
                        "{}",
                        char::from_u32(u32::from(*unit)).unwrap_or('\u{fffd}')
                    )?;
                }
                _ => write!(f, "\\u{unit:04x}")?,
            }
        }
        f.write_str("\"")
    }
}

/// Returns whether a character is an ECMAScript LineTerminator.
pub fn is_line_terminator(c: char) -> bool {
    matches!(c, '\n' | '\r' | '\u{2028}' | '\u{2029}')
}

/// Returns whether a character is ECMAScript WhiteSpace, excluding line terminators.
pub fn is_whitespace(c: char) -> bool {
    matches!(
        c,
        '\t' | '\u{b}' | '\u{c}' | ' ' | '\u{a0}' | '\u{feff}' | '\u{1680}' | '\u{2000}'
            ..='\u{200a}' | '\u{202f}' | '\u{205f}' | '\u{3000}'
    )
}

/// Converts an unsigned base-2, base-8, or base-16 integer to binary64.
///
/// Rejects empty or invalid digits. Rounds once, with ties to even.
pub fn parse_radix_integer(digits: &str, radix: u32) -> Option<f64> {
    if !matches!(radix, 2 | 8 | 16)
        || digits.is_empty()
        || !digits.chars().all(|c| c.is_digit(radix))
    {
        return None;
    }
    let width = radix.trailing_zeros();
    let mut count = 0usize;
    let mut significand = 0u64;
    let mut guard = false;
    let mut sticky = false;
    for digit in digits.chars().filter_map(|c| c.to_digit(radix)) {
        for shift in (0..width).rev() {
            let bit = digit & (1 << shift) != 0;
            if count == 0 && !bit {
                continue;
            }
            count += 1;
            if count <= 53 {
                significand = (significand << 1) | u64::from(bit);
            } else if count == 54 {
                guard = bit;
            } else {
                sticky |= bit;
            }
        }
    }
    if count <= 53 {
        return Some(significand as f64);
    }
    let mut exponent = count - 1;
    if guard && (sticky || significand & 1 != 0) {
        significand += 1;
    }
    if significand == 1 << 53 {
        significand >>= 1;
        exponent += 1;
    }
    if exponent > 1023 {
        return Some(f64::INFINITY);
    }
    Some(f64::from_bits(
        ((exponent as u64 + 1023) << 52) | (significand & ((1 << 52) - 1)),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn string_values_keep_content_hashing_order_and_thread_safe_ownership() {
        use std::collections::{BTreeSet, HashSet};
        let original = JsString::from_code_units(vec![0xd800, 0, 0xdc00]);
        let independent = JsString::from_code_units(vec![0xd800, 0, 0xdc00]);
        let values = HashSet::from([original.clone(), independent]);
        assert_eq!(values.len(), 1);
        let copy = original.clone();
        let extended =
            std::thread::spawn(move || copy.concat(&JsString::from_code_units(vec![0xd800])))
                .join()
                .unwrap();
        assert_eq!(original.code_units(), &[0xd800, 0, 0xdc00]);
        assert_eq!(extended.code_units(), &[0xd800, 0, 0xdc00, 0xd800]);
        let ordered = BTreeSet::from([extended.clone(), original.clone()]);
        assert_eq!(
            ordered.into_iter().collect::<Vec<_>>(),
            [original, extended]
        );
    }

    #[test]
    fn utf16_preserves_surrogates_and_counts_code_units() {
        assert_eq!(JsString::from("💩").len(), 2);
        let lone = JsString::from_code_units(vec![0xd800]);
        assert!(lone.to_utf8().is_err());
        assert_eq!(format!("{lone:?}"), "\"\\ud800\"");
        let pair = lone.concat(&JsString::from_code_units(vec![0xdc00]));
        assert_eq!(pair.to_utf8().unwrap(), "𐀀");
    }
}
