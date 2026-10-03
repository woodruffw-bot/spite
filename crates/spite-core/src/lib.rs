//! Shared source locations, diagnostics, and ECMAScript strings.

use std::fmt;

/// A half-open range of UTF-8 byte offsets in source text.
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
#[derive(Clone, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct JsString(Vec<u16>);

impl JsString {
    /// Creates a string without validating surrogate pairing.
    pub fn from_code_units(units: Vec<u16>) -> Self {
        Self(units)
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
        let mut units = self.0.clone();
        units.extend_from_slice(&other.0);
        Self(units)
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
        for unit in &self.0 {
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

#[cfg(test)]
mod tests {
    use super::*;

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
