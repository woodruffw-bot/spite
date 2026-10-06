//! RegExp.escape and EncodeForRegExpEscape (22.2.5.1, edition 17).

use crate::{JsString, is_line_terminator, is_whitespace};

/// Streams the UTF-16 result of RegExp.escape for a String argument.
///
/// The first ASCII letter or digit is hex-escaped to keep an adjacent preceding
/// escape from consuming it. Paired surrogates retain their original units;
/// unpaired surrogates use Unicode escapes. The iterator allocates no storage.
/// An embedding can clone it to check the exact output length before allocation.
pub fn regexp_escape_units(value: &JsString) -> impl Iterator<Item = u16> + Clone + '_ {
    char::decode_utf16(value.code_units().iter().copied())
        .enumerate()
        .flat_map(|(index, point)| {
            let cp = point
                .map(u32::from)
                .unwrap_or_else(|error| u32::from(error.unpaired_surrogate()));
            let (units, len) = encode(cp, index == 0);
            units.into_iter().take(len)
        })
}

fn encode(cp: u32, leading: bool) -> ([u16; 6], usize) {
    if leading && cp <= 0x7f && (cp as u8).is_ascii_alphanumeric() {
        return hex_escape(cp as u16, 2);
    }
    if cp <= 0x7f && b"^$\\.*+?()[]{}|/".contains(&(cp as u8)) {
        return ([u16::from(b'\\'), cp as u16, 0, 0, 0, 0], 2);
    }
    let control = match cp {
        0x09 => Some(b't'),
        0x0a => Some(b'n'),
        0x0b => Some(b'v'),
        0x0c => Some(b'f'),
        0x0d => Some(b'r'),
        _ => None,
    };
    if let Some(control) = control {
        return ([u16::from(b'\\'), u16::from(control), 0, 0, 0, 0], 2);
    }
    // Syntax characters and ControlEscape values take precedence over this
    // branch. In particular, NUL stays literal rather than becoming \\0.
    let punctuator = cp <= 0x7f && b",-=<>#&!%:;@~'`\"".contains(&(cp as u8));
    let space = char::from_u32(cp).is_some_and(|c| is_whitespace(c) || is_line_terminator(c));
    if punctuator || space || (0xd800..=0xdfff).contains(&cp) {
        return hex_escape(cp as u16, if cp <= 0xff { 2 } else { 4 });
    }
    if cp <= 0xffff {
        ([cp as u16, 0, 0, 0, 0, 0], 1)
    } else {
        let n = cp - 0x10000;
        (
            [
                0xd800 + (n >> 10) as u16,
                0xdc00 + (n & 0x3ff) as u16,
                0,
                0,
                0,
                0,
            ],
            2,
        )
    }
}

fn hex_escape(unit: u16, width: usize) -> ([u16; 6], usize) {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut units = [0; 6];
    units[0] = u16::from(b'\\');
    units[1] = u16::from(if width == 2 { b'x' } else { b'u' });
    for index in 0..width {
        units[index + 2] = u16::from(HEX[usize::from((unit >> (4 * (width - index - 1))) & 0xf)]);
    }
    (units, width + 2)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fmt::Write;

    fn escaped(value: &JsString) -> JsString {
        JsString::from_code_units(regexp_escape_units(value).collect())
    }

    #[test]
    fn ascii_encoding_distinguishes_leading_letters_digits_and_all_punctuators() {
        let mut table = String::new();
        for unit in 0..=0x7f {
            let single = JsString::from_code_units(vec![unit]);
            let later = JsString::from_code_units(vec![u16::from(b'_'), unit]);
            writeln!(
                table,
                "{unit:02x} leading={:?} later={:?}",
                escaped(&single),
                escaped(&later)
            )
            .unwrap();
        }
        insta::assert_snapshot!(table);
    }

    #[test]
    fn unicode_whitespace_and_unpaired_surrogates_escape_without_rewriting_pairs() {
        let cases = [
            vec![],
            vec![0, 0x85, 0x180e, 0x200b, 0x2060, 0xfffd],
            vec![
                0xa0, 0x1680, 0x2000, 0x2001, 0x2002, 0x2003, 0x2004, 0x2005, 0x2006, 0x2007,
                0x2008, 0x2009, 0x200a, 0x2028, 0x2029, 0x202f, 0x205f, 0x3000, 0xfeff,
            ],
            vec![0xe9, 0x301, 0x1c89],
            vec![0xd800],
            vec![0xdbff],
            vec![0xdc00],
            vec![0xdfff],
            vec![0xd800, 0xdc00],
            vec![0xdbff, 0xdfff],
            vec![0xd83d, 0xdca9],
            vec![0xd800, 0xd800, 0xdc00],
            vec![0xd800, 0xdc00, 0xdc00],
            vec![0xdc00, 0xd800, 0x41, 0xd800, 0xdc00, 0xdfff],
            vec![0x41, 0xd800, 0xdc00, 0x39],
        ];
        let mut table = String::new();
        for units in cases {
            let value = JsString::from_code_units(units);
            let stream = regexp_escape_units(&value);
            assert_eq!(
                stream.clone().collect::<Vec<_>>(),
                stream.collect::<Vec<_>>()
            );
            writeln!(table, "{value:?} -> {:?}", escaped(&value)).unwrap();
        }
        insta::assert_snapshot!(table);
    }
}
