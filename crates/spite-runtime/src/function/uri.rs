//! URI encoding/decoding and checked output accumulation (19.2.6).

use crate::{Error, ExceptionKind, Realm, Value};
use spite_core::{JsString, Span};

const RESERVED: &str = ";/?:@&=+$,#";
const HEX: &[u8; 16] = b"0123456789ABCDEF";

impl Realm {
    pub(crate) fn decode_uri(
        &mut self,
        value: Value,
        component: bool,
        span: Span,
    ) -> Result<Value, Error> {
        let string = self.string(value, span)?;
        let units = string.code_units();
        let mut result = Vec::new();
        let mut index = 0;
        while index < units.len() {
            self.tick(span)?;
            if units[index] != u16::from(b'%') {
                // Decode preserves literal code units, including unpaired
                // surrogates. Unicode validation applies only to escaped UTF-8.
                self.append_uri_units(&mut result, &units[index..index + 1], span)?;
                index += 1;
                continue;
            }
            let first = parse_octet(&units[index..], span)?;
            let count = match first.leading_ones() {
                0 => 1,
                n @ 2..=4 => n as usize,
                _ => return Err(malformed_uri(span)),
            };
            let escapes = units[index..]
                .get(..count * 3)
                .ok_or_else(|| malformed_uri(span))?;
            let mut octets = [0; 4];
            octets[0] = first;
            for (offset, escape) in escapes.chunks_exact(3).enumerate().skip(1) {
                octets[offset] = parse_octet(escape, span)?;
            }
            // RFC 3629 validation rejects overlong encodings, bad continuation
            // bytes, surrogate code points, and values above U+10FFFF.
            let point = std::str::from_utf8(&octets[..count])
                .map_err(|_| malformed_uri(span))?
                .chars()
                .next()
                .expect("one validated UTF-8 code point");
            if !component && point.is_ascii() && RESERVED.contains(point) {
                // Keep the original hex spelling, including lower-case letters.
                self.append_uri_units(&mut result, escapes, span)?;
            } else {
                self.append_uri_units(&mut result, point.encode_utf16(&mut [0; 2]), span)?;
            }
            // A decoded percent sign is not scanned again.
            index += escapes.len();
        }
        Ok(Value::String(JsString::from_code_units(result)))
    }

    pub(crate) fn encode_uri(
        &mut self,
        value: Value,
        component: bool,
        span: Span,
    ) -> Result<Value, Error> {
        // ToString precedes Unicode validation and uses the string hint.
        let string = self.string(value, span)?;
        let mut result = Vec::new();
        for point in char::decode_utf16(string.code_units().iter().copied()) {
            self.tick(span)?;
            let point = point.map_err(|_| {
                Self::exception(ExceptionKind::URIError, span, "unpaired URI surrogate")
            })?;
            if point.is_ascii_alphanumeric()
                || "_-.!~*'()".contains(point)
                || (!component && RESERVED.contains(point))
            {
                self.append_uri_units(&mut result, &[point as u16], span)?;
            } else {
                let mut utf8 = [0; 4];
                let octets = point.encode_utf8(&mut utf8).as_bytes();
                let mut escaped = [0; 12];
                for (index, &octet) in octets.iter().enumerate() {
                    escaped[index * 3] = u16::from(b'%');
                    escaped[index * 3 + 1] = u16::from(HEX[usize::from(octet >> 4)]);
                    escaped[index * 3 + 2] = u16::from(HEX[usize::from(octet & 0xf)]);
                }
                self.append_uri_units(&mut result, &escaped[..octets.len() * 3], span)?;
            }
        }
        Ok(Value::String(JsString::from_code_units(result)))
    }

    fn append_uri_units(
        &mut self,
        result: &mut Vec<u16>,
        part: &[u16],
        span: Span,
    ) -> Result<(), Error> {
        let length = result
            .len()
            .checked_add(part.len())
            .ok_or_else(|| uri_limit(span))?;
        if self
            .limits
            .max_string_units
            .is_some_and(|limit| length > limit)
        {
            return Err(uri_limit(span));
        }
        self.object_work(span, |_, budget| budget.charge(part.len()))?;
        result
            .try_reserve(part.len())
            .map_err(|_| uri_limit(span))?;
        result.extend_from_slice(part);
        Ok(())
    }
}

fn parse_octet(escape: &[u16], span: Span) -> Result<u8, Error> {
    let [0x25, high, low, ..] = escape else {
        return Err(malformed_uri(span));
    };
    let high = hex_digit(*high).ok_or_else(|| malformed_uri(span))?;
    let low = hex_digit(*low).ok_or_else(|| malformed_uri(span))?;
    Ok((high << 4) | low)
}

fn hex_digit(unit: u16) -> Option<u8> {
    match unit {
        0x30..=0x39 => Some((unit - 0x30) as u8),
        0x41..=0x46 => Some((unit - 0x41 + 10) as u8),
        0x61..=0x66 => Some((unit - 0x61 + 10) as u8),
        _ => None,
    }
}

fn malformed_uri(span: Span) -> Error {
    Realm::exception(
        ExceptionKind::URIError,
        span,
        "malformed URI escape or UTF-8 sequence",
    )
}

fn uri_limit(span: Span) -> Error {
    Error::Limit {
        span,
        message: "URI output exceeds host or platform capacity".into(),
    }
}
