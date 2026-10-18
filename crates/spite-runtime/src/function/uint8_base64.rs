//! Uint8Array Base64 conversion (23.3.1.1, 23.3.2.1/.3, 23.3.3.4-.7).

use super::uint8_hex::Uint8Layout;
use crate::{Error, ExceptionKind, ObjectHandle, Realm, Value, binary_data::ElementType, object};
use spite_core::{JsString, Span};

const MAX_DECODED_LENGTH: u64 = (1u64 << 53) - 1;
const STANDARD: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
const URL: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";

#[derive(Clone, Copy)]
pub(super) enum Method {
    From,
    SetFrom,
    To,
}

impl Method {
    pub(super) fn from_name(name: &str) -> Option<Self> {
        match name {
            "fromBase64" => Some(Self::From),
            "setFromBase64" => Some(Self::SetFrom),
            "toBase64" => Some(Self::To),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Alphabet {
    Standard,
    Url,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum LastChunk {
    Loose,
    Strict,
    StopBeforePartial,
}

#[derive(Debug, Eq, PartialEq)]
struct Decoded {
    read: usize,
    bytes: Vec<u8>,
    // Internal diagnostic position only, never a public result property.
    error: Option<usize>,
}

fn skip_ascii_whitespace(
    units: &[u16],
    mut index: usize,
    budget: &mut object::Budget,
) -> Result<usize, object::Error> {
    while let Some(unit) = units.get(index) {
        budget.charge(1)?;
        if !matches!(*unit, 0x09 | 0x0a | 0x0c | 0x0d | 0x20) {
            break;
        }
        index += 1;
    }
    Ok(index)
}

fn sextet(unit: u16, alphabet: Alphabet) -> Option<u8> {
    match unit {
        0x41..=0x5a => Some((unit - 0x41) as u8),
        0x61..=0x7a => Some((unit - 0x61 + 26) as u8),
        0x30..=0x39 => Some((unit - 0x30 + 52) as u8),
        0x2b if alphabet == Alphabet::Standard => Some(62),
        0x2f if alphabet == Alphabet::Standard => Some(63),
        0x2d if alphabet == Alphabet::Url => Some(62),
        0x5f if alphabet == Alphabet::Url => Some(63),
        _ => None,
    }
}

fn decoded_chunk(chunk: [u8; 4]) -> [u8; 3] {
    [
        (chunk[0] << 2) | (chunk[1] >> 4),
        ((chunk[1] & 15) << 4) | (chunk[2] >> 2),
        ((chunk[2] & 3) << 6) | chunk[3],
    ]
}

fn append_bytes(
    output: &mut Vec<u8>,
    bytes: &[u8],
    budget: &mut object::Budget,
) -> Result<(), object::Error> {
    if output.capacity() - output.len() < bytes.len() {
        budget.charge(output.len())?;
    }
    budget.charge(bytes.len())?;
    output
        .try_reserve(bytes.len())
        .map_err(|_| object::Error::Heap(spite_heap::Error::Capacity))?;
    output.extend_from_slice(bytes);
    Ok(())
}

/// FromBase64 (23.3.3.7). Hold a final quantum until all its padding, suffix,
/// and strict overflow-bit checks pass. Syntax failure retains prior quanta.
fn from_base64(
    units: &[u16],
    alphabet: Alphabet,
    last: LastChunk,
    maximum: u64,
    budget: &mut object::Budget,
) -> Result<Decoded, object::Error> {
    let mut result = Decoded {
        read: 0,
        bytes: Vec::new(),
        error: None,
    };
    budget.charge(1)?;
    if maximum == 0 {
        return Ok(result);
    }
    let mut chunk = [0u8; 4];
    let mut chunk_length = 0usize;
    let mut written = 0u64;
    let mut index = 0usize;
    loop {
        index = skip_ascii_whitespace(units, index, budget)?;
        if index == units.len() {
            if chunk_length > 0 {
                if last == LastChunk::StopBeforePartial {
                    return Ok(result);
                }
                if last == LastChunk::Strict || chunk_length == 1 {
                    result.error = Some(index);
                    return Ok(result);
                }
                let bytes = decoded_chunk(chunk);
                append_bytes(&mut result.bytes, &bytes[..chunk_length - 1], budget)?;
            }
            result.read = units.len();
            return Ok(result);
        }
        budget.charge(1)?;
        let position = index;
        let unit = units[index];
        index += 1;
        if unit == 0x3d {
            if chunk_length < 2 {
                result.error = Some(position);
                return Ok(result);
            }
            index = skip_ascii_whitespace(units, index, budget)?;
            if chunk_length == 2 {
                if index == units.len() {
                    if last == LastChunk::StopBeforePartial {
                        return Ok(result);
                    }
                    result.error = Some(index);
                    return Ok(result);
                }
                budget.charge(1)?;
                if units[index] == 0x3d {
                    index = skip_ascii_whitespace(units, index + 1, budget)?;
                }
            }
            if index < units.len() {
                result.error = Some(index);
                return Ok(result);
            }
            if last == LastChunk::Strict
                && (chunk_length == 2 && chunk[1] & 15 != 0
                    || chunk_length == 3 && chunk[2] & 3 != 0)
            {
                result.error = Some(position);
                return Ok(result);
            }
            let bytes = decoded_chunk(chunk);
            append_bytes(&mut result.bytes, &bytes[..chunk_length - 1], budget)?;
            result.read = units.len();
            return Ok(result);
        }
        let Some(value) = sextet(unit, alphabet) else {
            result.error = Some(position);
            return Ok(result);
        };
        // Validate the next alphabet character BEFORE deciding that this
        // incomplete full quantum cannot fit. Padding follows its own path.
        let remaining = maximum - written;
        if remaining == 1 && chunk_length == 2 || remaining == 2 && chunk_length == 3 {
            return Ok(result);
        }
        chunk[chunk_length] = value;
        chunk_length += 1;
        if chunk_length == 4 {
            append_bytes(&mut result.bytes, &decoded_chunk(chunk), budget)?;
            written += 3;
            chunk = [0; 4];
            chunk_length = 0;
            result.read = index;
            if written == maximum {
                return Ok(result);
            }
        }
    }
}

fn keyword(
    string: &JsString,
    expected: &[u8],
    budget: &mut object::Budget,
) -> Result<bool, object::Error> {
    budget.charge(1)?;
    if string.len() != expected.len() {
        return Ok(false);
    }
    budget.charge(string.len())?;
    Ok(string
        .code_units()
        .iter()
        .copied()
        .eq(expected.iter().copied().map(u16::from)))
}

impl Realm {
    pub(super) fn uint8_base64_method(
        &mut self,
        method: Method,
        receiver: Value,
        mut arguments: std::vec::IntoIter<Value>,
        span: Span,
    ) -> Result<Value, Error> {
        if matches!(method, Method::To) {
            let object = self.uint8_brand(receiver, span)?;
            let options =
                self.uint8_base64_options(arguments.next().unwrap_or(Value::Undefined), span)?;
            let alphabet = self.uint8_base64_alphabet(&options, span)?;
            let omit_padding = self
                .get_property(&options, &JsString::from("omitPadding"), span)?
                .to_boolean();
            let layout = self.uint8_layout(&object, span)?;
            return self.uint8_base64_encode(&layout, alphabet, omit_padding, span);
        }
        // Static this is ignored. Setters check exact Uint8 brand before the
        // String type check, options getters, and eventual live witness.
        let object = if matches!(method, Method::SetFrom) {
            Some(self.uint8_brand(receiver, span)?)
        } else {
            None
        };
        let string = match arguments.next().unwrap_or(Value::Undefined) {
            Value::String(string) => string,
            _ => {
                return Err(Self::exception(
                    ExceptionKind::TypeError,
                    span,
                    "base64 input must be a String",
                ));
            }
        };
        let options =
            self.uint8_base64_options(arguments.next().unwrap_or(Value::Undefined), span)?;
        let alphabet = self.uint8_base64_alphabet(&options, span)?;
        let last = self.uint8_base64_last_chunk(&options, span)?;
        let layout = object
            .as_ref()
            .map(|object| self.uint8_layout(object, span))
            .transpose()?;
        let maximum = layout
            .as_ref()
            .map_or(MAX_DECODED_LENGTH, |layout| layout.length);
        let decoded = self.object_work(span, |_, budget| {
            from_base64(string.code_units(), alphabet, last, maximum, budget)
        })?;
        if let Some(layout) = layout {
            self.uint8_write_bytes(&layout, &decoded.bytes, span)?;
            if let Some(position) = decoded.error {
                return Err(Self::uint8_base64_syntax(position, span));
            }
            return self.uint8_decoding_result(decoded.read, decoded.bytes.len(), span);
        }
        if let Some(position) = decoded.error {
            return Err(Self::uint8_base64_syntax(position, span));
        }
        let length =
            u64::try_from(decoded.bytes.len()).map_err(|_| Self::uint8_base64_capacity(span))?;
        let prototype = self
            .intrinsics
            .as_ref()
            .expect("initialized realm")
            .typed_array
            .prototype(ElementType::Uint8)
            .clone();
        let result = self.allocate_typed_array(ElementType::Uint8, &prototype, length, span)?;
        let layout = self.uint8_layout(&result, span)?;
        self.uint8_write_bytes(&layout, &decoded.bytes, span)?;
        Ok(Value::Object(result))
    }

    /// GetOptionsObject (7.3.36): undefined allocates a null-prototype object;
    /// every other non-Object is rejected without boxing or coercion hooks.
    fn uint8_base64_options(&mut self, options: Value, span: Span) -> Result<ObjectHandle, Error> {
        match options {
            Value::Undefined => self.object_work(span, |objects, _| objects.create(None)),
            Value::Object(object) => Ok(object),
            _ => Err(Self::exception(
                ExceptionKind::TypeError,
                span,
                "base64 options must be an Object or undefined",
            )),
        }
    }

    fn uint8_base64_alphabet(
        &mut self,
        options: &ObjectHandle,
        span: Span,
    ) -> Result<Alphabet, Error> {
        let value = self.get_property(options, &JsString::from("alphabet"), span)?;
        let alphabet = self.object_work(span, |_, budget| {
            Ok(match &value {
                Value::Undefined => Some(Alphabet::Standard),
                Value::String(string) if keyword(string, b"base64", budget)? => {
                    Some(Alphabet::Standard)
                }
                Value::String(string) if keyword(string, b"base64url", budget)? => {
                    Some(Alphabet::Url)
                }
                _ => None,
            })
        })?;
        alphabet.ok_or_else(|| {
            Self::exception(ExceptionKind::TypeError, span, "invalid base64 alphabet")
        })
    }

    fn uint8_base64_last_chunk(
        &mut self,
        options: &ObjectHandle,
        span: Span,
    ) -> Result<LastChunk, Error> {
        let value = self.get_property(options, &JsString::from("lastChunkHandling"), span)?;
        let last = self.object_work(span, |_, budget| {
            Ok(match &value {
                Value::Undefined => Some(LastChunk::Loose),
                Value::String(string) if keyword(string, b"loose", budget)? => {
                    Some(LastChunk::Loose)
                }
                Value::String(string) if keyword(string, b"strict", budget)? => {
                    Some(LastChunk::Strict)
                }
                Value::String(string) if keyword(string, b"stop-before-partial", budget)? => {
                    Some(LastChunk::StopBeforePartial)
                }
                _ => None,
            })
        })?;
        last.ok_or_else(|| {
            Self::exception(
                ExceptionKind::TypeError,
                span,
                "invalid base64 lastChunkHandling",
            )
        })
    }

    fn uint8_base64_encode(
        &mut self,
        layout: &Uint8Layout,
        alphabet: Alphabet,
        omit_padding: bool,
        span: Span,
    ) -> Result<Value, Error> {
        let remainder = layout.length % 3;
        let extra = if remainder == 0 {
            0
        } else if omit_padding {
            remainder + 1
        } else {
            4
        };
        let length = (layout.length / 3)
            .checked_mul(4)
            .and_then(|length| length.checked_add(extra))
            .and_then(|length| usize::try_from(length).ok())
            .ok_or_else(|| Self::uint8_base64_capacity(span))?;
        if self
            .limits
            .max_string_units
            .is_some_and(|limit| length > limit)
        {
            return Err(Error::Limit {
                span,
                message: "string length limit exceeded".into(),
            });
        }
        let alphabet = if alphabet == Alphabet::Standard {
            STANDARD
        } else {
            URL
        };
        let mut units = Vec::new();
        let mut index = 0u64;
        while index < layout.length {
            let count = (layout.length - index).min(3) as usize;
            let offset = layout
                .byte_offset
                .checked_add(index)
                .ok_or_else(|| Self::uint8_base64_capacity(span))?;
            let bytes = self.uint8_buffer_work(span, |objects, budget| {
                objects.read_array_buffer_bytes(&layout.buffer, offset, count, budget)
            })?;
            let symbols = [
                bytes[0] >> 2,
                ((bytes[0] & 3) << 4) | (bytes[1] >> 4),
                ((bytes[1] & 15) << 2) | (bytes[2] >> 6),
                bytes[2] & 63,
            ];
            let mut part = [u16::from(b'='); 4];
            for (unit, symbol) in part.iter_mut().zip(symbols).take(count + 1) {
                *unit = u16::from(alphabet[usize::from(symbol)]);
            }
            let output_count = if omit_padding { count + 1 } else { 4 };
            self.object_work(span, |_, budget| {
                if units.capacity() - units.len() < output_count {
                    budget.charge(units.len())?;
                }
                budget.charge(output_count)
            })?;
            units
                .try_reserve(output_count)
                .map_err(|_| Self::uint8_base64_capacity(span))?;
            units.extend_from_slice(&part[..output_count]);
            index += count as u64;
        }
        // Same existing final Vec-to-Arc boundary as hex/string traversal.
        Ok(Value::String(JsString::from_code_units(units)))
    }

    fn uint8_base64_syntax(position: usize, span: Span) -> Error {
        Self::exception(
            ExceptionKind::SyntaxError,
            span,
            format!("invalid base64 input at UTF-16 offset {position}"),
        )
    }

    fn uint8_base64_capacity(span: Span) -> Error {
        Error::Limit {
            span,
            message: "Uint8Array base64 storage exceeds platform capacity".into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Limits;

    fn decode(text: &str, last: LastChunk, maximum: u64) -> Decoded {
        from_base64(
            &text.encode_utf16().collect::<Vec<_>>(),
            Alphabet::Standard,
            last,
            maximum,
            &mut object::Budget::with_work_limit(None),
        )
        .unwrap()
    }

    #[test]
    fn base64_boundary_matrix_keeps_quantum_read_and_pending_error_prefixes() {
        let mut rows = Vec::new();
        for (text, last, maximum) in [
            ("Zg", LastChunk::Loose, 9),
            ("Zg", LastChunk::Strict, 9),
            ("Zg", LastChunk::StopBeforePartial, 9),
            ("Z", LastChunk::Loose, 9),
            ("Z", LastChunk::StopBeforePartial, 9),
            ("Zg==", LastChunk::Strict, 9),
            ("Zh==", LastChunk::Strict, 9),
            ("Zh==", LastChunk::StopBeforePartial, 9),
            ("Zg=", LastChunk::StopBeforePartial, 9),
            ("Zg=", LastChunk::Loose, 9),
            ("Zm9v!", LastChunk::Loose, 3),
            ("Zm9v!", LastChunk::Loose, 4),
            ("TWFu", LastChunk::Loose, 1),
            ("TWFu", LastChunk::Loose, 2),
            ("TQ==", LastChunk::Strict, 1),
            ("TW!", LastChunk::Loose, 1),
            ("!", LastChunk::Strict, 0),
            (" Zm9v ", LastChunk::Loose, 9),
            (" Zm9v ", LastChunk::Loose, 3),
            (" Zm9v Zg", LastChunk::StopBeforePartial, 9),
            ("Zm9vZg==x", LastChunk::Loose, 9),
        ] {
            let result = decode(text, last, maximum);
            rows.push(format!(
                "{text:?} {last:?} cap{maximum}: read={} bytes={:02x?} syntax={}",
                result.read,
                result.bytes,
                result.error.is_some()
            ));
        }
        insta::assert_snapshot!(rows.join("\n"), @r#"
        "Zg" Loose cap9: read=2 bytes=[66] syntax=false
        "Zg" Strict cap9: read=0 bytes=[] syntax=true
        "Zg" StopBeforePartial cap9: read=0 bytes=[] syntax=false
        "Z" Loose cap9: read=0 bytes=[] syntax=true
        "Z" StopBeforePartial cap9: read=0 bytes=[] syntax=false
        "Zg==" Strict cap9: read=4 bytes=[66] syntax=false
        "Zh==" Strict cap9: read=0 bytes=[] syntax=true
        "Zh==" StopBeforePartial cap9: read=4 bytes=[66] syntax=false
        "Zg=" StopBeforePartial cap9: read=0 bytes=[] syntax=false
        "Zg=" Loose cap9: read=0 bytes=[] syntax=true
        "Zm9v!" Loose cap3: read=4 bytes=[66, 6f, 6f] syntax=false
        "Zm9v!" Loose cap4: read=4 bytes=[66, 6f, 6f] syntax=true
        "TWFu" Loose cap1: read=0 bytes=[] syntax=false
        "TWFu" Loose cap2: read=0 bytes=[] syntax=false
        "TQ==" Strict cap1: read=4 bytes=[4d] syntax=false
        "TW!" Loose cap1: read=0 bytes=[] syntax=true
        "!" Strict cap0: read=0 bytes=[] syntax=false
        " Zm9v " Loose cap9: read=6 bytes=[66, 6f, 6f] syntax=false
        " Zm9v " Loose cap3: read=5 bytes=[66, 6f, 6f] syntax=false
        " Zm9v Zg" StopBeforePartial cap9: read=5 bytes=[66, 6f, 6f] syntax=false
        "Zm9vZg==x" Loose cap9: read=4 bytes=[66, 6f, 6f] syntax=true
        "#);
    }

    #[test]
    fn every_padded_tail_sextet_obeys_strict_overflow_bits() {
        for value in 0u8..64 {
            let symbol = char::from(STANDARD[usize::from(value)]);
            for (text, expected, valid) in [
                (format!("A{symbol}=="), vec![value >> 4], value & 15 == 0),
                (format!("AA{symbol}="), vec![0, value >> 2], value & 3 == 0),
            ] {
                let strict = decode(&text, LastChunk::Strict, 9);
                assert_eq!(strict.error.is_none(), valid, "{text}");
                if valid {
                    assert_eq!(strict.bytes, expected, "{text}");
                } else {
                    assert!(strict.bytes.is_empty(), "{text}");
                }
                for mode in [LastChunk::Loose, LastChunk::StopBeforePartial] {
                    let result = decode(&text, mode, 9);
                    assert_eq!(result.read, text.len(), "{text}");
                    assert_eq!(result.bytes, expected, "{text}");
                    assert_eq!(result.error, None, "{text}");
                }
            }
        }
    }

    #[test]
    fn full_capacity_stops_before_suffix_work_and_zero_capacity_skips_syntax() {
        let prefix = "Zm9v".encode_utf16().collect::<Vec<_>>();
        let mut long = prefix.clone();
        long.extend(std::iter::repeat_n(0xffff, 8192));
        let mut a = object::Budget::new(100);
        let mut b = object::Budget::new(100);
        let expected =
            from_base64(&prefix, Alphabet::Standard, LastChunk::Loose, 3, &mut a).unwrap();
        assert_eq!(
            from_base64(&long, Alphabet::Standard, LastChunk::Loose, 3, &mut b),
            Ok(expected)
        );
        assert_eq!(a.remaining_work(), b.remaining_work());
        assert_eq!(
            from_base64(
                &long,
                Alphabet::Url,
                LastChunk::Strict,
                0,
                &mut object::Budget::new(1)
            ),
            Ok(Decoded {
                read: 0,
                bytes: vec![],
                error: None
            })
        );
        assert_eq!(
            decode("Zg==", LastChunk::Strict, MAX_DECODED_LENGTH).bytes,
            [102]
        );
    }

    #[test]
    fn explicit_options_preserve_decode_and_write_priority_with_a_full_heap() {
        const SETUP: &str = "let target=Uint8Array.of(9,9,9,9),options={};";
        let mut probe = Realm::default();
        probe.eval(SETUP).unwrap();
        probe.eval("target").unwrap();
        probe.eval("options").unwrap();
        let collection = probe.collect(usize::MAX).unwrap();
        let mut realm = Realm::new(Limits {
            max_heap_entries: Some(collection.live.checked_add(collection.reclaimed).unwrap()),
            ..Limits::default()
        });
        realm.eval(SETUP).unwrap();
        let target = realm.eval("target").unwrap();
        let options = realm.eval("options").unwrap();
        loop {
            match realm.objects.create(None) {
                Ok(_) => {}
                Err(object::Error::Heap(spite_heap::Error::Capacity)) => break,
                other => panic!("unexpected heap-fill result: {other:?}"),
            }
        }
        let span = Span::new(0, 0);
        let input =
            |text: &str| vec![Value::String(JsString::from(text)), options.clone()].into_iter();
        let exhausted = Err(Error::Limit {
            span,
            message: "heap storage capacity exceeded".into(),
        });
        // No collection/eval boundary follows the heap fill. The explicit
        // existing options object avoids the earlier default-options allocation.
        assert_eq!(
            realm.uint8_base64_method(
                Method::From,
                Value::Null,
                vec![Value::String(JsString::from("!"))].into_iter(),
                span
            ),
            exhausted
        );
        assert!(matches!(
            realm.uint8_base64_method(Method::From, Value::Null, input("Zg==x"), span),
            Err(Error::Exception {
                kind: ExceptionKind::SyntaxError,
                ..
            })
        ));
        assert_eq!(
            realm.uint8_base64_method(Method::From, Value::Null, input("AQID"), span),
            exhausted
        );
        assert_eq!(
            realm.uint8_base64_method(Method::SetFrom, target.clone(), input("AQID"), span),
            exhausted
        );
        assert!(matches!(
            realm.uint8_base64_method(Method::SetFrom, target.clone(), input("BAUG!"), span),
            Err(Error::Exception {
                kind: ExceptionKind::SyntaxError,
                ..
            })
        ));
        assert_eq!(
            realm.uint8_base64_method(Method::To, target, vec![options.clone()].into_iter(), span),
            Ok(Value::String(JsString::from("BAUGCQ==")))
        );
    }

    #[test]
    fn explicit_string_limit_preflights_output_without_changing_source_or_catching_host_errors() {
        let mut realm = Realm::default();
        assert_eq!(realm.eval("let a=new Uint8Array(7),flag=0;a[0]=7;a[6]=9;Uint8Array.fromBase64('AAEC').toBase64()==='AAEC'"),Ok(Value::Boolean(true)));
        // Enable the output-focused quota after constructing the fixture.
        realm.limits.max_string_units = Some(8);
        assert!(matches!(
            realm.eval("try{a.toBase64();}catch{flag=1;}finally{flag=2;}"),
            Err(Error::Limit { .. })
        ));
        assert_eq!(
            realm.eval("flag===0 && a[0]===7 && a[6]===9 && a.length===7"),
            Ok(Value::Boolean(true))
        );
        realm
            .eval("let huge=new Uint8Array(9007199254740991)")
            .unwrap();
        assert!(matches!(
            realm.eval("try{huge.toBase64();}catch{flag=1;}finally{flag=2;}"),
            Err(Error::Limit { .. })
        ));
        assert_eq!(realm.eval("flag"), Ok(Value::Number(0.0)));
    }
}
