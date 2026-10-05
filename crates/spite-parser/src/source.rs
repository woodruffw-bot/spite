//! Lossless source code points (11.1), with byte offsets compatible with UTF-8.

use spite_core::{Diagnostic, DiagnosticKind, JsString, Span};
use std::ops::Range;

#[derive(PartialEq)]
pub(crate) struct SourceText {
    // Scalar text is unchanged. Each unpaired surrogate occupies the three
    // bytes of U+FFFD here, with its original code point recorded separately.
    // U+FFFD and surrogates have identical lexical classifications outside
    // literal contents/comments: neither is an identifier or punctuator.
    // Never expose this scanner representation as the original source text.
    text: String,
    surrogates: Vec<(usize, u16)>,
}

fn capacity_error() -> Diagnostic {
    Diagnostic::new(
        DiagnosticKind::Limit,
        Span::default(),
        "source capacity exceeded",
    )
}

impl SourceText {
    pub(crate) fn from_str(text: &str) -> Self {
        Self {
            text: text.to_owned(),
            surrogates: Vec::new(),
        }
    }

    pub(crate) fn from_utf16(source: &JsString) -> Result<Self, Diagnostic> {
        let mut bytes = 0usize;
        let mut count = 0usize;
        for point in char::decode_utf16(source.code_units().iter().copied()) {
            bytes = bytes
                .checked_add(point.as_ref().map_or(3, |c| c.len_utf8()))
                .ok_or_else(capacity_error)?;
            count += usize::from(point.is_err());
        }
        let mut text = String::new();
        text.try_reserve_exact(bytes)
            .map_err(|_| capacity_error())?;
        let mut surrogates = Vec::new();
        surrogates
            .try_reserve_exact(count)
            .map_err(|_| capacity_error())?;
        for point in char::decode_utf16(source.code_units().iter().copied()) {
            match point {
                Ok(c) => text.push(c),
                Err(error) => {
                    surrogates.push((text.len(), error.unpaired_surrogate()));
                    text.push('\u{fffd}');
                }
            }
        }
        Ok(Self { text, surrogates })
    }

    pub(crate) fn join(parts: &[&Self]) -> Result<Self, Diagnostic> {
        let mut bytes = 0usize;
        let mut count = 0usize;
        for part in parts {
            bytes = bytes
                .checked_add(part.text.len())
                .ok_or_else(capacity_error)?;
            count = count
                .checked_add(part.surrogates.len())
                .ok_or_else(capacity_error)?;
        }
        let mut text = String::new();
        text.try_reserve_exact(bytes)
            .map_err(|_| capacity_error())?;
        let mut surrogates = Vec::new();
        surrogates
            .try_reserve_exact(count)
            .map_err(|_| capacity_error())?;
        for part in parts {
            let offset = text.len();
            surrogates.extend(
                part.surrogates
                    .iter()
                    .map(|&(pos, unit)| (offset + pos, unit)),
            );
            text.push_str(&part.text);
        }
        Ok(Self { text, surrogates })
    }

    pub(crate) fn lexical_text(&self) -> &str {
        &self.text
    }

    pub(crate) fn original_code_point(&self, offset: usize, scalar: char) -> u32 {
        match self
            .surrogates
            .binary_search_by_key(&offset, |&(pos, _)| pos)
        {
            Ok(index) => u32::from(self.surrogates[index].1),
            Err(_) => scalar as u32,
        }
    }

    pub(crate) fn as_utf8(&self, range: Range<usize>) -> Option<&str> {
        let index = self
            .surrogates
            .partition_point(|&(pos, _)| pos < range.start);
        if self
            .surrogates
            .get(index)
            .is_some_and(|&(pos, _)| pos < range.end)
        {
            None
        } else {
            Some(&self.text[range])
        }
    }

    pub(crate) fn to_js_string(&self, range: Range<usize>) -> JsString {
        let mut units = Vec::new();
        for (offset, scalar) in self.text[range.clone()].char_indices() {
            let point = self.original_code_point(range.start + offset, scalar);
            if point <= 0xffff {
                units.push(point as u16);
            } else {
                units.extend_from_slice(scalar.encode_utf16(&mut [0; 2]));
            }
        }
        JsString::from_code_units(units)
    }
}
