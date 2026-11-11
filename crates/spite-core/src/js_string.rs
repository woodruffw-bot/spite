//! Immutable UTF-16 strings preserving lone surrogates.

use std::fmt;
use std::sync::Arc;

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

#[cfg(test)]
mod tests {
    use super::JsString;

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
