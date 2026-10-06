//! EscapeRegExpPattern (22.2.6.11.1, edition 17).

use crate::JsString;

/// Streams a validated Pattern's UTF-16 source for use between literal delimiters.
///
/// Empty input becomes `(?:)`. Solidus and raw line terminators are escaped while
/// preserving existing escapes and all other code units, including lone surrogates.
/// The same encoding works in every Pattern mode. The iterator allocates no storage
/// and can be cloned to check the exact output length before allocation.
pub fn regexp_pattern_source_units(value: &JsString) -> impl Iterator<Item = u16> + Clone + '_ {
    let mut escaped = false;
    "(?:)"
        .encode_utf16()
        .take(if value.is_empty() { 4 } else { 0 })
        .chain(value.code_units().iter().copied().flat_map(move |unit| {
            let preceding_escape = escaped;
            escaped = !escaped && unit == u16::from(b'\\');
            let (units, len) = match unit {
                0x2f if !preceding_escape => ([0x5c, 0x2f, 0, 0, 0, 0], 2),
                0x0a | 0x0d | 0x2028 | 0x2029 => {
                    let (units, len) = match unit {
                        0x0a => ([0x5c, u16::from(b'n'), 0, 0, 0, 0], 2),
                        0x0d => ([0x5c, u16::from(b'r'), 0, 0, 0, 0], 2),
                        0x2028 => ([0x5c, 0x75, 0x32, 0x30, 0x32, 0x38], 6),
                        _ => ([0x5c, 0x75, 0x32, 0x30, 0x32, 0x39], 6),
                    };
                    if preceding_escape {
                        // A non-Unicode IdentityEscape of a raw line terminator
                        // already emitted its reverse solidus. Reuse it rather
                        // than turn the escaped terminator into a literal backslash.
                        (
                            [units[1], units[2], units[3], units[4], units[5], 0],
                            len - 1,
                        )
                    } else {
                        (units, len)
                    }
                }
                _ => ([unit, 0, 0, 0, 0, 0], 1),
            };
            units.into_iter().take(len)
        }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fmt::Write;

    fn source(value: &JsString) -> JsString {
        JsString::from_code_units(regexp_pattern_source_units(value).collect())
    }

    #[test]
    fn source_preserves_pattern_syntax_and_utf16_units() {
        let cases = [
            vec![],
            "(?:)".encode_utf16().collect(),
            "^a(b|c)+[d-f]$".encode_utf16().collect(),
            r"(\d)\w\s\u{1f4a9}\1".encode_utf16().collect(),
            "[/]".encode_utf16().collect(),
            r"[\q{a\/b}]".encode_utf16().collect(),
            vec![0, 0x09, 0x0b, 0x0c, 0x85, 0xa0, 0x180e, 0x200b, 0xfeff],
            vec![0xd800],
            vec![0xdc00],
            vec![0xd800, 0xdc00],
            vec![0xdc00, 0xd800, 0x41, 0xd800, 0xdc00, 0xdfff],
        ];
        let mut table = String::new();
        for units in cases {
            let value = JsString::from_code_units(units);
            let stream = regexp_pattern_source_units(&value);
            assert_eq!(
                stream.clone().collect::<Vec<_>>(),
                stream.collect::<Vec<_>>()
            );
            writeln!(table, "{value:?} -> {:?}", source(&value)).unwrap();
        }
        insta::assert_snapshot!(table);
    }

    #[test]
    fn delimiters_and_line_terminators_preserve_existing_escape_parity() {
        let mut table = String::new();
        for terminator in [0x2f, 0x0a, 0x0d, 0x2028, 0x2029] {
            for slashes in 0..=6 {
                let mut units = vec![0x5c; slashes];
                units.push(terminator);
                let value = JsString::from_code_units(units);
                let encoded = source(&value);
                assert!(
                    !encoded
                        .code_units()
                        .iter()
                        .any(|unit| matches!(unit, 0x0a | 0x0d | 0x2028 | 0x2029))
                );
                writeln!(table, "{value:?} -> {encoded:?}").unwrap();
            }
        }
        insta::assert_snapshot!(table);
    }
}
