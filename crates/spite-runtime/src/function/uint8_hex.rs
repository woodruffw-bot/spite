//! Uint8Array hexadecimal conversion (23.3.1.2, 23.3.2.2/.4, 23.3.3).

use crate::{
    Error, ExceptionKind, ObjectHandle, Realm, Value,
    binary_data::ElementType,
    object::{self, ArrayBufferError, Budget, DataDescriptor, Objects},
};
use spite_core::{JsString, Span};

const MAX_DECODED_LENGTH: u64 = (1u64 << 53) - 1;
const HEX_DIGITS: &[u8; 16] = b"0123456789abcdef";

#[derive(Clone, Copy)]
pub(super) enum Method {
    From,
    SetFrom,
    To,
}

impl Method {
    pub(super) fn from_name(name: &str) -> Option<Self> {
        match name {
            "fromHex" => Some(Self::From),
            "setFromHex" => Some(Self::SetFrom),
            "toHex" => Some(Self::To),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum HexError {
    OddLength,
    InvalidPair(usize),
}

#[derive(Debug, Eq, PartialEq)]
struct DecodedHex {
    read: usize,
    bytes: Vec<u8>,
    error: Option<HexError>,
}

/// FromHex (23.3.3.8). Syntax errors retain the completed prefix in the
/// record; host work/allocation failures use the separate outer error channel.
fn from_hex(units: &[u16], maximum: u64, budget: &mut Budget) -> Result<DecodedHex, object::Error> {
    let mut result = DecodedHex {
        read: 0,
        bytes: Vec::new(),
        error: None,
    };
    budget.charge(1)?;
    // Whole-input parity precedes capacity, even when maximum is zero. This
    // reads the UTF-16 length, without scanning an intentionally ignored suffix.
    if units.len() % 2 != 0 {
        result.error = Some(HexError::OddLength);
        return Ok(result);
    }
    let mut written = 0u64;
    while result.read < units.len() && written < maximum {
        budget.charge(2)?;
        let pair = (
            hex_digit(units[result.read]),
            hex_digit(units[result.read + 1]),
        );
        let (Some(high), Some(low)) = pair else {
            result.error = Some(HexError::InvalidPair(result.read));
            return Ok(result);
        };
        if result.bytes.len() == result.bytes.capacity() {
            budget.charge(result.bytes.len())?;
        }
        budget.charge(1)?;
        result
            .bytes
            .try_reserve(1)
            .map_err(|_| object::Error::Heap(spite_heap::Error::Capacity))?;
        result.bytes.push((high << 4) | low);
        // read and the even input length are even, so this complete pair fits
        // in the existing usize input range. written remains <= maximum.
        result.read += 2;
        written += 1;
    }
    Ok(result)
}

fn hex_digit(unit: u16) -> Option<u8> {
    match unit {
        0x30..=0x39 => Some((unit - 0x30) as u8),
        0x41..=0x46 => Some((unit - 0x41 + 10) as u8),
        0x61..=0x66 => Some((unit - 0x61 + 10) as u8),
        _ => None,
    }
}

pub(super) struct Uint8Layout {
    pub(super) buffer: ObjectHandle,
    pub(super) byte_offset: u64,
    pub(super) length: u64,
}

impl Realm {
    pub(super) fn uint8_hex_method(
        &mut self,
        method: Method,
        receiver: Value,
        mut arguments: std::vec::IntoIter<Value>,
        span: Span,
    ) -> Result<Value, Error> {
        if matches!(method, Method::From) {
            // The static receiver is ignored, including subclasses and null.
            let string =
                Self::uint8_hex_string(arguments.next().unwrap_or(Value::Undefined), span)?;
            let decoded = self.object_work(span, |_, budget| {
                from_hex(string.code_units(), MAX_DECODED_LENGTH, budget)
            })?;
            if let Some(error) = decoded.error {
                return Err(Self::uint8_hex_syntax(error, span));
            }
            // Syntax rejection precedes normative intrinsic allocation. Use
            // AllocateTypedArray directly, without reading constructor/species.
            let length =
                u64::try_from(decoded.bytes.len()).map_err(|_| Self::uint8_hex_capacity(span))?;
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
            return Ok(Value::Object(result));
        }

        let object = self.uint8_brand(receiver, span)?;
        if matches!(method, Method::SetFrom) {
            // Brand -> String type -> current witness -> bounded decode.
            // Neither String conversion hooks nor public view properties run.
            let string =
                Self::uint8_hex_string(arguments.next().unwrap_or(Value::Undefined), span)?;
            let layout = self.uint8_layout(&object, span)?;
            let decoded = self.object_work(span, |_, budget| {
                from_hex(string.code_units(), layout.length, budget)
            })?;
            self.uint8_write_bytes(&layout, &decoded.bytes, span)?;
            // A language syntax failure is deliberately thrown only after all
            // completed pairs have been written. Result allocation follows it.
            if let Some(error) = decoded.error {
                return Err(Self::uint8_hex_syntax(error, span));
            }
            self.uint8_decoding_result(decoded.read, decoded.bytes.len(), span)
        } else {
            let layout = self.uint8_layout(&object, span)?;
            self.uint8_hex_encode(&layout, span)
        }
    }

    /// ValidateUint8Array (23.3.3.1) checks only exact private kind, without a
    /// current-bounds check. In particular, Uint8ClampedArray is not accepted.
    pub(super) fn uint8_brand(
        &mut self,
        receiver: Value,
        span: Span,
    ) -> Result<ObjectHandle, Error> {
        let invalid = || {
            Self::exception(
                ExceptionKind::TypeError,
                span,
                "receiver is not a Uint8Array",
            )
        };
        let Value::Object(object) = receiver else {
            return Err(invalid());
        };
        let valid = self.object_work(span, |objects, _| {
            Ok(objects
                .inspect(&object)?
                .typed_array_data()
                .is_some_and(|data| data.element_type() == ElementType::Uint8))
        })?;
        if !valid {
            return Err(invalid());
        }
        Ok(object)
    }

    fn uint8_hex_string(value: Value, span: Span) -> Result<JsString, Error> {
        match value {
            Value::String(string) => Ok(string),
            _ => Err(Self::exception(
                ExceptionKind::TypeError,
                span,
                "hexadecimal input must be a String",
            )),
        }
    }

    pub(super) fn uint8_layout(
        &mut self,
        object: &ObjectHandle,
        span: Span,
    ) -> Result<Uint8Layout, Error> {
        let layout = self.object_work(span, |objects, _| {
            let data = objects
                .inspect(object)?
                .typed_array_data()
                .expect("validated Uint8Array");
            let buffer = objects
                .inspect(data.buffer())?
                .array_buffer_data()
                .expect("TypedArray's buffer");
            Ok(data
                .in_bounds_array_length(buffer)
                .map(|length| Uint8Layout {
                    buffer: data.buffer().clone(),
                    byte_offset: data.byte_offset(),
                    length,
                }))
        })?;
        layout.ok_or_else(|| {
            Self::exception(
                ExceptionKind::TypeError,
                span,
                "Uint8Array is out of bounds or detached",
            )
        })
    }

    /// SetUint8ArrayBytes (23.3.3.3) has no callbacks or numeric conversions.
    /// Each bounded byte write retains its existing transactional guarantee;
    /// earlier bytes remain committed if a later host operation fails.
    pub(super) fn uint8_write_bytes(
        &mut self,
        layout: &Uint8Layout,
        bytes: &[u8],
        span: Span,
    ) -> Result<(), Error> {
        self.uint8_buffer_work(span, |objects, budget| {
            for (index, byte) in bytes.iter().enumerate() {
                let index = u64::try_from(index).map_err(|_| ArrayBufferError::OutOfBounds)?;
                let offset = layout
                    .byte_offset
                    .checked_add(index)
                    .ok_or(ArrayBufferError::OutOfBounds)?;
                objects.write_array_buffer_bytes(&layout.buffer, offset, &[*byte], budget)?;
            }
            Ok(())
        })
    }

    pub(super) fn uint8_decoding_result(
        &mut self,
        read: usize,
        written: usize,
        span: Span,
    ) -> Result<Value, Error> {
        let prototype = self
            .intrinsics
            .as_ref()
            .expect("initialized realm")
            .object_prototype
            .clone();
        self.object_work(span, |objects, budget| {
            let result = objects.create(Some(&prototype))?;
            for (key, number) in [("read", read), ("written", written)] {
                objects.define(
                    &result,
                    key,
                    DataDescriptor {
                        value: Some(Value::Number(number as f64)),
                        writable: Some(true),
                        enumerable: Some(true),
                        configurable: Some(true),
                    },
                    budget,
                )?;
            }
            Ok(Value::Object(result))
        })
    }

    fn uint8_hex_encode(&mut self, layout: &Uint8Layout, span: Span) -> Result<Value, Error> {
        // Fusing GetUint8ArrayBytes and encoding avoids a second full byte list.
        // No JavaScript hook or collection can run after the captured witness.
        let length = layout
            .length
            .checked_mul(2)
            .and_then(|length| usize::try_from(length).ok())
            .ok_or_else(|| Self::uint8_hex_capacity(span))?;
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
        let mut units = Vec::new();
        let mut index = 0u64;
        while index < layout.length {
            let count = (layout.length - index).min(8) as usize;
            let offset = layout
                .byte_offset
                .checked_add(index)
                .ok_or_else(|| Self::uint8_hex_capacity(span))?;
            let bytes = self.uint8_buffer_work(span, |objects, budget| {
                objects.read_array_buffer_bytes(&layout.buffer, offset, count, budget)
            })?;
            for byte in &bytes[..count] {
                self.object_work(span, |_, budget| {
                    if units.capacity() - units.len() < 2 {
                        budget.charge(units.len())?;
                    }
                    budget.charge(2)
                })?;
                units
                    .try_reserve(2)
                    .map_err(|_| Self::uint8_hex_capacity(span))?;
                units.push(u16::from(HEX_DIGITS[usize::from(byte >> 4)]));
                units.push(u16::from(HEX_DIGITS[usize::from(byte & 15)]));
            }
            index += count as u64;
        }
        // JsString's existing Vec-to-Arc API performs the final shared-storage
        // conversion; it does not expose a fallible allocation boundary.
        Ok(Value::String(JsString::from_code_units(units)))
    }

    pub(super) fn uint8_buffer_work<T>(
        &mut self,
        span: Span,
        work: impl FnOnce(&mut Objects, &mut Budget) -> Result<T, ArrayBufferError>,
    ) -> Result<T, Error> {
        let result = self.object_work(span, |objects, budget| match work(objects, budget) {
            Err(ArrayBufferError::Storage(error)) => Err(error),
            // Lazy sparse-page allocation is host storage, not the normative
            // fresh backing allocation used by AllocateTypedArray above.
            Err(ArrayBufferError::Capacity) => {
                Err(object::Error::Heap(spite_heap::Error::Capacity))
            }
            result => Ok(result),
        })?;
        result.map_err(|error| {
            Self::exception(
                match error {
                    ArrayBufferError::OutOfBounds => ExceptionKind::RangeError,
                    ArrayBufferError::Detached
                    | ArrayBufferError::NotResizable
                    | ArrayBufferError::SameBuffer => ExceptionKind::TypeError,
                    ArrayBufferError::Capacity | ArrayBufferError::Storage(_) => {
                        unreachable!("host errors returned above")
                    }
                },
                span,
                error.to_string(),
            )
        })
    }

    fn uint8_hex_syntax(error: HexError, span: Span) -> Error {
        let message = match error {
            HexError::OddLength => "hexadecimal input must have even UTF-16 length".into(),
            HexError::InvalidPair(index) => {
                format!("invalid hexadecimal pair at UTF-16 offset {index}")
            }
        };
        Self::exception(ExceptionKind::SyntaxError, span, message)
    }

    fn uint8_hex_capacity(span: Span) -> Error {
        Error::Limit {
            span,
            message: "Uint8Array hexadecimal storage exceeds platform capacity".into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Limits;

    fn decode(units: &[u16], maximum: u64) -> DecodedHex {
        from_hex(units, maximum, &mut Budget::with_work_limit(None)).unwrap()
    }

    #[test]
    fn hexadecimal_decode_matrix_preserves_utf16_read_prefix_and_pending_errors() {
        let mut rows = Vec::new();
        for (label, units, maximum) in [
            ("empty", vec![], 0),
            ("even zero-capacity", vec![0x7a, 0x7a], 0),
            ("odd zero-capacity", vec![0x7a], 0),
            ("ignored invalid suffix", vec![0x30, 0x30, 0x78, 0x7a], 1),
            ("invalid pair after prefix", vec![0x30, 0x30, 0x78, 0x7a], 2),
            ("whole odd suffix", vec![0x30, 0x30, 0x66], 1),
            ("mixed case", vec![0x41, 0x66, 0x31, 0x30], 9),
            ("surrogate pair", vec![0xd83d, 0xde00], 2),
        ] {
            let result = decode(&units, maximum);
            let error = match result.error {
                None => "none".into(),
                Some(HexError::OddLength) => "odd".into(),
                Some(HexError::InvalidPair(index)) => format!("pair@{index}"),
            };
            rows.push(format!(
                "{label}: read={}, bytes={:02x?}, error={error}",
                result.read, result.bytes
            ));
        }
        insta::assert_snapshot!(rows.join("\n"), @r"
        empty: read=0, bytes=[], error=none
        even zero-capacity: read=0, bytes=[], error=none
        odd zero-capacity: read=0, bytes=[], error=odd
        ignored invalid suffix: read=2, bytes=[00], error=none
        invalid pair after prefix: read=2, bytes=[00], error=pair@2
        whole odd suffix: read=0, bytes=[], error=odd
        mixed case: read=4, bytes=[af, 10], error=none
        surrogate pair: read=0, bytes=[], error=pair@0
        ");
    }

    #[test]
    fn all_byte_values_decode_from_upper_and_lower_ascii_pairs() {
        for byte in 0..=u8::MAX {
            for text in [format!("{byte:02x}"), format!("{byte:02X}")] {
                let units = text.encode_utf16().collect::<Vec<_>>();
                assert_eq!(
                    decode(&units, u64::MAX),
                    DecodedHex {
                        read: 2,
                        bytes: vec![byte],
                        error: None,
                    },
                    "{text}"
                );
            }
        }
    }

    #[test]
    fn every_non_hex_utf16_unit_rejects_in_either_pair_position() {
        let allowed = "0123456789abcdefABCDEF".encode_utf16().collect::<Vec<_>>();
        for unit in 0..=u16::MAX {
            if allowed.contains(&unit) {
                continue;
            }
            for pair in [[unit, 0x30], [0x30, unit]] {
                assert_eq!(
                    decode(&pair, 1),
                    DecodedHex {
                        read: 0,
                        bytes: vec![],
                        error: Some(HexError::InvalidPair(0)),
                    },
                    "{pair:04x?}"
                );
            }
        }
    }

    #[test]
    fn bounded_decode_neither_allocates_from_capacity_nor_charges_ignored_suffix() {
        let prefix = [0x30, 0x31];
        let mut long = prefix.to_vec();
        long.extend(std::iter::repeat_n(0xffff, 8192));
        let mut short_budget = Budget::new(100);
        let mut long_budget = Budget::new(100);
        let expected = from_hex(&prefix, 1, &mut short_budget).unwrap();
        assert_eq!(from_hex(&long, 1, &mut long_budget), Ok(expected));
        assert_eq!(short_budget.remaining_work(), long_budget.remaining_work());
        assert_eq!(decode(&prefix, MAX_DECODED_LENGTH).bytes, [1]);
        assert_eq!(
            from_hex(&prefix, 1, &mut Budget::new(0)),
            Err(object::Error::WorkLimit)
        );
    }

    #[test]
    fn syntax_precedes_static_allocation_and_result_allocation_follows_setter_writes() {
        const SETUP: &str = "let target=Uint8Array.of(9,9);";
        let mut probe = Realm::default();
        probe.eval(SETUP).unwrap();
        probe.eval("target").unwrap();
        let collection = probe.collect(usize::MAX).unwrap();
        let mut realm = Realm::new(Limits {
            max_heap_entries: Some(collection.live.checked_add(collection.reclaimed).unwrap()),
            ..Limits::default()
        });
        realm.eval(SETUP).unwrap();
        let target = realm.eval("target").unwrap();
        loop {
            match realm.objects.create(None) {
                Ok(_) => {}
                Err(object::Error::Heap(spite_heap::Error::Capacity)) => break,
                other => panic!("unexpected heap-fill result: {other:?}"),
            }
        }
        // No eval or collect follows the heap fill: allocation never invokes
        // collection, and all later calls enter the algorithm directly.
        let span = Span::new(0, 0);
        let input = |text: &str| vec![Value::String(JsString::from(text))].into_iter();
        assert!(matches!(
            realm.uint8_hex_method(Method::From, Value::Null, input("00zz"), span),
            Err(Error::Exception {
                kind: ExceptionKind::SyntaxError,
                ..
            })
        ));
        let exhausted = Err(Error::Limit {
            span,
            message: "heap storage capacity exceeded".into(),
        });
        assert_eq!(
            realm.uint8_hex_method(Method::From, Value::Null, input("0102"), span),
            exhausted
        );
        assert_eq!(
            realm.uint8_hex_method(Method::SetFrom, target.clone(), input("0102"), span),
            exhausted
        );
        assert!(matches!(
            realm.uint8_hex_method(Method::SetFrom, target.clone(), input("03zz"), span),
            Err(Error::Exception {
                kind: ExceptionKind::SyntaxError,
                ..
            })
        ));
        let Value::Object(target) = target else {
            unreachable!("fixture Uint8Array");
        };
        assert_eq!(
            realm.get_property(&target, &JsString::from("0"), span),
            Ok(Value::Number(3.0))
        );
        assert_eq!(
            realm.get_property(&target, &JsString::from("1"), span),
            Ok(Value::Number(2.0))
        );
    }
}
