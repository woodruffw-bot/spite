//! Shared source locations, diagnostics, strings, symbols, and property keys.

mod case;
mod case_data;
pub mod date;
mod normalization;
mod normalization_data;
mod regexp_canonicalize;
mod regexp_escape;
mod regexp_pattern_source;
mod regexp_program;
mod replacement;
mod symbol;
mod unicode;
mod unicode_data;
mod well_known;

pub use case::{is_unicode_case_ignorable, is_unicode_cased, unicode_case_mapping};
pub use normalization::{
    canonical_combining_class, canonical_composition, hangul_decomposition, unicode_decomposition,
};
pub use regexp_canonicalize::regexp_canonicalize_character;
pub use regexp_escape::regexp_escape_units;
pub use regexp_pattern_source::regexp_pattern_source_units;
pub use regexp_program::{RegExpProgram, RegExpProgramMatch, RegExpProgramResult};
pub use replacement::{ReplacementPart, replacement_parts};
pub use symbol::{JsSymbol, PropertyKey, PropertyKeyRef, WeakJsSymbol};
#[doc(hidden)]
pub use symbol::{SymbolCollectionGuard, SymbolCollectionPin};
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

    /// Creates the exact decimal representation of an unsigned property index.
    ///
    /// Formats the integer directly, without converting it to an ECMAScript Number.
    pub fn from_index(mut index: u64) -> Self {
        let mut units = [0u16; 20];
        let mut start = units.len();
        loop {
            start -= 1;
            units[start] = u16::from(b'0') + (index % 10) as u16;
            index /= 10;
            if index == 0 {
                return Self::from(&units[start..]);
            }
        }
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
        if self.is_empty() {
            return other.clone();
        }
        if other.is_empty() {
            return self.clone();
        }
        let length = self
            .len()
            .checked_add(other.len())
            .expect("string length exceeds addressable capacity");
        let mut units = Vec::with_capacity(length);
        units.extend_from_slice(self.code_units());
        units.extend_from_slice(other.code_units());
        Self(units.into())
    }
}

impl From<&str> for JsString {
    fn from(value: &str) -> Self {
        // Exact-size ASCII iteration lets Arc allocate its final slice directly.
        if value.is_ascii() {
            return Self(value.bytes().map(u16::from).collect());
        }
        Self(value.encode_utf16().collect())
    }
}

impl<const N: usize> From<[u16; N]> for JsString {
    fn from(value: [u16; N]) -> Self {
        Self(Arc::from(value))
    }
}

impl From<&[u16]> for JsString {
    fn from(value: &[u16]) -> Self {
        Self(Arc::from(value))
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
/// Accepts ASCII digits only; rejects empty or invalid input. Rounds once,
/// with ties to even.
pub fn parse_radix_integer(digits: &str, radix: u32) -> Option<f64> {
    if !matches!(radix, 2 | 8 | 16)
        || digits.is_empty()
        || !digits.bytes().all(|byte| char::from(byte).is_digit(radix))
    {
        return None;
    }
    let width = radix.trailing_zeros();
    let mut count = 0usize;
    let mut significand = 0u64;
    let mut guard = false;
    let mut sticky = false;
    for digit in digits
        .bytes()
        .filter_map(|byte| char::from(byte).to_digit(radix))
    {
        for shift in (0..width).rev() {
            let bit = digit & (1 << shift) != 0;
            if count == 0 && !bit {
                continue;
            }
            count += 1;
            // Every digit was validated above. At 1,025 significant bits the
            // value already exceeds binary64's finite range.
            if count > 1024 {
                return Some(f64::INFINITY);
            }
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
    fn radix_integer_overflow_rounds_at_the_midpoint_and_validates_the_entire_input() {
        // The overflow midpoint is 2^1024 - 2^970; the integer immediately
        // below it rounds to MAX, and the tie rounds to Infinity.
        for (radix, below, midpoint, overflowing) in [
            (
                2,
                "1".repeat(53) + "0" + &"1".repeat(970),
                "1".repeat(54) + &"0".repeat(970),
                "1".to_owned() + &"0".repeat(1024),
            ),
            (
                8,
                "1".to_owned() + &"7".repeat(17) + "5" + &"7".repeat(323),
                "1".to_owned() + &"7".repeat(17) + "6" + &"0".repeat(323),
                "4".to_owned() + &"0".repeat(341),
            ),
            (
                16,
                "f".repeat(13) + "b" + &"f".repeat(242),
                "f".repeat(13) + "c" + &"0".repeat(242),
                "1".to_owned() + &"0".repeat(256),
            ),
        ] {
            assert_eq!(parse_radix_integer(&below, radix), Some(f64::MAX));
            assert_eq!(parse_radix_integer(&midpoint, radix), Some(f64::INFINITY));
            assert_eq!(
                parse_radix_integer(&overflowing, radix),
                Some(f64::INFINITY)
            );
            assert_eq!(parse_radix_integer(&(overflowing + "g"), radix), None);
            assert_eq!(
                parse_radix_integer(&("0".repeat(2048) + &below), radix),
                Some(f64::MAX)
            );
        }
    }

    #[test]
    fn decimal_indices_match_integer_formatting_at_full_width_boundaries() {
        let mut indices: Vec<u64> = (0..10_000).collect();
        for power in (0..=19).map(|exponent| 10u64.pow(exponent)) {
            indices.extend([power - 1, power, power + 1]);
        }
        for bit in 0..64 {
            let power = 1u64 << bit;
            indices.extend([power - 1, power, power + 1]);
        }
        indices.extend([u64::MAX - 1, u64::MAX]);
        let mut sample = 0x1234_5678_9abc_def0u64;
        for _ in 0..4096 {
            sample = sample.wrapping_mul(6364136223846793005).wrapping_add(1);
            indices.push(sample);
        }
        for index in indices {
            assert_eq!(
                JsString::from_index(index).code_units(),
                index.to_string().encode_utf16().collect::<Vec<_>>(),
                "{index}"
            );
        }
    }

    #[test]
    fn string_values_keep_content_hashing_order_and_thread_safe_ownership() {
        use std::collections::{BTreeSet, HashSet};
        let original = JsString::from([0xd800, 0, 0xdc00]);
        let independent = JsString::from_code_units(vec![0xd800, 0, 0xdc00]);
        let values = HashSet::from([
            original.clone(),
            independent,
            JsString::from(original.code_units()),
        ]);
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
        for length in [0, 1, 15, 16, 17, 32] {
            for byte in 0..=127u8 {
                let source = char::from(byte).to_string().repeat(length);
                let string = JsString::from(source.as_str());
                assert_eq!(string.code_units(), vec![u16::from(byte); length]);
                assert_eq!(string.to_utf8().unwrap(), source);
            }
            for text in ["é", "Ā", "𐀀", "💩", "a\0é💩"] {
                let source = text.repeat(length);
                let string = JsString::from(source.as_str());
                assert_eq!(
                    string.code_units(),
                    source.encode_utf16().collect::<Vec<_>>()
                );
                assert_eq!(string.to_utf8().unwrap(), source);
            }
        }
        assert_eq!(JsString::from("💩").len(), 2);
        let lone = JsString::from([0xd800]);
        let empty = JsString::default();
        assert_eq!(empty.concat(&lone), lone);
        assert_eq!(lone.concat(&empty), lone);
        assert!(lone.to_utf8().is_err());
        assert_eq!(format!("{lone:?}"), "\"\\ud800\"");
        let pair = lone.concat(&JsString::from_code_units(vec![0xdc00]));
        assert_eq!(pair.to_utf8().unwrap(), "𐀀");
    }
}
