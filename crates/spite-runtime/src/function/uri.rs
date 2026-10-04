//! URI encoding and checked output accumulation (19.2.6.3–5).

use crate::{Error, ExceptionKind, Realm, Value};
use spite_core::{JsString, Span};

const RESERVED: &str = ";/?:@&=+$,#";
const HEX: &[u8; 16] = b"0123456789ABCDEF";

impl Realm {
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

fn uri_limit(span: Span) -> Error {
    Error::Limit {
        span,
        message: "URI output exceeds host or platform capacity".into(),
    }
}
