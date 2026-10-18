//! Shared RawBytesToNumeric and NumericToRawBytes element codecs (25.1.3).

use crate::{Value, value::to_uint32};
use spite_bigint::{BigInt, Budget, Error};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ElementType {
    Int8,
    Uint8,
    Uint8Clamped,
    Int16,
    Uint16,
    Int32,
    Uint32,
    BigInt64,
    BigUint64,
    Float16,
    Float32,
    Float64,
}

impl ElementType {
    pub(crate) fn size(self) -> usize {
        match self {
            Self::Int8 | Self::Uint8 | Self::Uint8Clamped => 1,
            Self::Int16 | Self::Uint16 | Self::Float16 => 2,
            Self::Int32 | Self::Uint32 | Self::Float32 => 4,
            Self::BigInt64 | Self::BigUint64 | Self::Float64 => 8,
        }
    }

    pub(crate) fn is_bigint(self) -> bool {
        matches!(self, Self::BigInt64 | Self::BigUint64)
    }

    // The caller completes observable coercions and validates the live view
    // before entering this fixed-width conversion. Unused output bytes are zero.
    pub(crate) fn encode(
        self,
        value: Value,
        little_endian: bool,
        budget: &mut Budget,
    ) -> Result<[u8; 8], Error> {
        budget.charge(self.size())?;
        let bits = match value {
            Value::BigInt(value) if self.is_bigint() => value.modulo_u64(budget)?,
            Value::Number(number) if !self.is_bigint() => match self {
                Self::Uint8Clamped => u64::from(uint8_clamp(number)),
                Self::Float16 => u64::from(binary16_bits(number)),
                Self::Float32 => u64::from(if number.is_nan() {
                    f32::NAN.to_bits()
                } else {
                    (number as f32).to_bits()
                }),
                Self::Float64 => {
                    if number.is_nan() {
                        f64::NAN.to_bits()
                    } else {
                        number.to_bits()
                    }
                }
                _ => u64::from(to_uint32(number)),
            },
            _ => unreachable!("value was coerced to the element's numeric type"),
        };
        let mut bytes = bits.to_le_bytes();
        bytes[self.size()..].fill(0);
        if !little_endian {
            bytes[..self.size()].reverse();
        }
        Ok(bytes)
    }

    pub(crate) fn decode(
        self,
        mut bytes: [u8; 8],
        little_endian: bool,
        budget: &mut Budget,
    ) -> Result<Value, Error> {
        budget.charge(self.size())?;
        if !little_endian {
            bytes[..self.size()].reverse();
        }
        bytes[self.size()..].fill(0);
        let bits = u64::from_le_bytes(bytes);
        let number = match self {
            Self::Int8 => f64::from(bits as i8),
            Self::Uint8 | Self::Uint8Clamped => f64::from(bits as u8),
            Self::Int16 => f64::from(bits as i16),
            Self::Uint16 => f64::from(bits as u16),
            Self::Int32 => f64::from(bits as i32),
            Self::Uint32 => f64::from(bits as u32),
            Self::BigInt64 => return BigInt::from_i64(bits as i64, budget).map(Value::BigInt),
            Self::BigUint64 => return BigInt::from_u64(bits, budget).map(Value::BigInt),
            Self::Float16 => binary16_number(bits as u16),
            Self::Float32 => f64::from(f32::from_bits(bits as u32)),
            Self::Float64 => f64::from_bits(bits),
        };
        Ok(Value::Number(if number.is_nan() {
            f64::NAN
        } else {
            number
        }))
    }
}

// ToUint8Clamp (7.1.12): clamp before nearest-even rounding, without a
// ToUint32 conversion or the upward ties used by Math.round.
fn uint8_clamp(number: f64) -> u8 {
    if number.is_nan() {
        0
    } else {
        number.clamp(0.0, 255.0).round_ties_even() as u8
    }
}

fn binary16_bits(number: f64) -> u16 {
    // Reuse Math.f16round's direct binary64 rounding. Going through binary32
    // first would double-round inputs adjacent to a binary16 midpoint.
    let rounded = crate::function::binary16_round(number);
    if rounded.is_nan() {
        return 0x7e00;
    }
    let bits = rounded.to_bits();
    let sign = ((bits >> 48) & 0x8000) as u16;
    if rounded.is_infinite() {
        return sign | 0x7c00;
    }
    if rounded.abs() < 1.0 / 16384.0 {
        // A rounded subnormal is an exact integer multiple of 2^-24.
        return sign | (rounded.abs() * 16777216.0) as u16;
    }
    let exponent = ((bits >> 52) & 0x7ff) as u16 - 1008;
    sign | (exponent << 10) | ((bits >> 42) & 0x3ff) as u16
}

fn binary16_number(bits: u16) -> f64 {
    let exponent = (bits >> 10) & 0x1f;
    let fraction = bits & 0x3ff;
    let magnitude = match exponent {
        0 => f64::from(fraction) / 16777216.0,
        0x1f if fraction == 0 => f64::INFINITY,
        0x1f => return f64::NAN,
        _ => f64::from_bits((u64::from(exponent + 1008) << 52) | (u64::from(fraction) << 42)),
    };
    if bits & 0x8000 == 0 {
        magnitude
    } else {
        -magnitude
    }
}

#[cfg(test)]
mod tests {
    use super::{ElementType, binary16_bits};
    use crate::Value;
    use spite_bigint::{BigInt, Budget, Error};

    fn unlimited() -> Budget {
        Budget::with_limits(None, None)
    }

    fn number(element: ElementType, bytes: [u8; 8], little_endian: bool) -> f64 {
        let Value::Number(number) = element
            .decode(bytes, little_endian, &mut unlimited())
            .unwrap()
        else {
            panic!("expected a Number element")
        };
        number
    }

    fn bytes(bits: u64, size: usize, little_endian: bool) -> [u8; 8] {
        let mut bytes = bits.to_le_bytes();
        bytes[size..].fill(0);
        if !little_endian {
            bytes[..size].reverse();
        }
        bytes
    }

    #[test]
    fn all_binary16_encodings_decode_exactly_and_round_trip_in_both_orders() {
        for bits in 0..=u16::MAX {
            let exponent = (bits >> 10) & 31;
            let fraction = bits & 1023;
            let magnitude = match exponent {
                0 => f64::from(fraction) * 2.0_f64.powi(-24),
                31 if fraction == 0 => f64::INFINITY,
                31 => f64::NAN,
                _ => (1024.0 + f64::from(fraction)) * 2.0_f64.powi(i32::from(exponent) - 25),
            };
            let expected = if bits & 0x8000 == 0 {
                magnitude
            } else {
                -magnitude
            };
            for little_endian in [false, true] {
                let encoded = bytes(u64::from(bits), 2, little_endian);
                let actual = number(ElementType::Float16, encoded, little_endian);
                if expected.is_nan() {
                    assert!(actual.is_nan(), "{bits:04x}");
                } else {
                    assert_eq!(actual.to_bits(), expected.to_bits(), "{bits:04x}");
                }
                let canonical = if expected.is_nan() { 0x7e00 } else { bits };
                assert_eq!(
                    ElementType::Float16
                        .encode(Value::Number(actual), little_endian, &mut unlimited())
                        .unwrap(),
                    bytes(u64::from(canonical), 2, little_endian),
                    "{bits:04x}"
                );
            }
        }
    }

    #[test]
    fn binary16_rounding_uses_binary64_midpoints_without_double_rounding() {
        for (midpoint, lower, even, upper) in [
            (2.0_f64.powi(-25), 0x0000, 0x0000, 0x0001),
            (3.0 * 2.0_f64.powi(-25), 0x0001, 0x0002, 0x0002),
            (1.0 + 2.0_f64.powi(-11), 0x3c00, 0x3c00, 0x3c01),
            (1.0 + 3.0 * 2.0_f64.powi(-11), 0x3c01, 0x3c02, 0x3c02),
            (65520.0, 0x7bff, 0x7c00, 0x7c00),
        ] {
            for (number, expected) in [
                (f64::from_bits(midpoint.to_bits() - 1), lower),
                (midpoint, even),
                (f64::from_bits(midpoint.to_bits() + 1), upper),
            ] {
                assert_eq!(binary16_bits(number), expected, "{number}");
                assert_eq!(binary16_bits(-number), expected | 0x8000, "{}", -number);
            }
        }
        assert_eq!(binary16_bits(f64::from_bits(1)), 0);
        assert_eq!(binary16_bits(-f64::from_bits(1)), 0x8000);
    }

    #[test]
    fn clamped_bytes_saturate_and_round_halfway_values_to_even() {
        for (input, expected) in [
            (f64::NAN, 0),
            (f64::NEG_INFINITY, 0),
            (-1.0, 0),
            (-0.0, 0),
            (0.0, 0),
            (f64::from_bits(1), 0),
            (255.0, 255),
            (256.0, 255),
            (f64::INFINITY, 255),
            (f64::MAX, 255),
        ] {
            for little_endian in [false, true] {
                let encoded = ElementType::Uint8Clamped
                    .encode(Value::Number(input), little_endian, &mut unlimited())
                    .unwrap();
                assert_eq!(encoded, [expected, 0, 0, 0, 0, 0, 0, 0]);
                assert_eq!(
                    number(ElementType::Uint8Clamped, encoded, little_endian),
                    f64::from(expected)
                );
            }
        }
        // Exercise both tie parities and each midpoint's binary64 neighbors.
        for lower in 0_u8..255 {
            let midpoint = f64::from(lower) + 0.5;
            for (input, expected) in [
                (f64::from_bits(midpoint.to_bits() - 1), lower),
                (midpoint, if lower % 2 == 0 { lower } else { lower + 1 }),
                (f64::from_bits(midpoint.to_bits() + 1), lower + 1),
            ] {
                assert_eq!(
                    ElementType::Uint8Clamped
                        .encode(Value::Number(input), true, &mut unlimited())
                        .unwrap(),
                    [expected, 0, 0, 0, 0, 0, 0, 0],
                    "{input}"
                );
            }
        }
        for byte in 0..=u8::MAX {
            assert_eq!(
                number(ElementType::Uint8Clamped, [byte; 8], true),
                f64::from(byte)
            );
        }
    }

    #[test]
    fn integer_elements_truncate_modulo_width_and_decode_signed_boundaries() {
        for (element, input, bits, expected) in [
            (ElementType::Int8, -129.9, 0x7f, 127.0),
            (ElementType::Int8, 128.9, 0x80, -128.0),
            (ElementType::Uint8, -1.9, 0xff, 255.0),
            (ElementType::Uint8, 256.9, 0, 0.0),
            (ElementType::Int16, -32769.9, 0x7fff, 32767.0),
            (ElementType::Int16, 32768.9, 0x8000, -32768.0),
            (ElementType::Uint16, -1.9, 0xffff, 65535.0),
            (ElementType::Uint16, 65536.9, 0, 0.0),
            (ElementType::Int32, -2147483649.0, 0x7fff_ffff, 2147483647.0),
            (ElementType::Int32, 2147483648.0, 0x8000_0000, -2147483648.0),
            (ElementType::Uint32, -1.9, 0xffff_ffff, 4294967295.0),
            (ElementType::Uint32, 4294967297.9, 1, 1.0),
        ] {
            for little_endian in [false, true] {
                let encoded = element
                    .encode(Value::Number(input), little_endian, &mut unlimited())
                    .unwrap();
                assert_eq!(encoded, bytes(bits, element.size(), little_endian));
                assert_eq!(number(element, encoded, little_endian), expected);
            }
        }
        for element in [
            ElementType::Int8,
            ElementType::Uint8,
            ElementType::Int16,
            ElementType::Uint16,
            ElementType::Int32,
            ElementType::Uint32,
        ] {
            for input in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY, -0.0, f64::MAX] {
                assert_eq!(
                    element
                        .encode(Value::Number(input), false, &mut unlimited())
                        .unwrap(),
                    [0; 8]
                );
            }
        }
    }

    #[test]
    fn wide_floats_preserve_zero_subnormals_and_special_values() {
        for (element, input, bits) in [
            (ElementType::Float32, -0.0, 0x8000_0000),
            (ElementType::Float32, 2.0_f64.powi(-149), 1),
            (ElementType::Float32, 2.0_f64.powi(-150), 0),
            (ElementType::Float32, -2.0_f64.powi(-150), 0x8000_0000),
            (ElementType::Float32, 2.0_f64.powi(-126), 0x0080_0000),
            (ElementType::Float32, f64::MAX, 0x7f80_0000),
            (ElementType::Float64, -0.0, 0x8000_0000_0000_0000),
            (ElementType::Float64, f64::from_bits(1), 1),
            (
                ElementType::Float64,
                -f64::from_bits(1),
                0x8000_0000_0000_0001,
            ),
            (ElementType::Float64, f64::INFINITY, 0x7ff0_0000_0000_0000),
        ] {
            for little_endian in [false, true] {
                assert_eq!(
                    element
                        .encode(Value::Number(input), little_endian, &mut unlimited())
                        .unwrap(),
                    bytes(bits, element.size(), little_endian)
                );
                let actual = number(
                    element,
                    bytes(bits, element.size(), little_endian),
                    little_endian,
                );
                let expected = if element == ElementType::Float32 {
                    f64::from(f32::from_bits(bits as u32))
                } else {
                    f64::from_bits(bits)
                };
                assert_eq!(actual.to_bits(), expected.to_bits());
            }
        }
        for (element, payload) in [
            (ElementType::Float16, 0xfd55),
            (ElementType::Float32, 0xffa0_0001),
            (ElementType::Float64, 0xfff0_0000_0000_0001),
        ] {
            assert!(number(element, bytes(payload, element.size(), true), true).is_nan());
            let canonical = element
                .encode(Value::Number(f64::NAN), true, &mut unlimited())
                .unwrap();
            assert_eq!(
                element
                    .encode(Value::Number(-f64::NAN), true, &mut unlimited())
                    .unwrap(),
                canonical
            );
        }
    }

    #[test]
    fn bigint_elements_preserve_every_bit_and_wrap_without_number_conversion() {
        for bits in [
            0,
            1,
            (1_u64 << 53) + 1,
            (1_u64 << 63) - 1,
            1_u64 << 63,
            u64::MAX,
        ] {
            for element in [ElementType::BigInt64, ElementType::BigUint64] {
                for little_endian in [false, true] {
                    let decoded = element
                        .decode(bits.to_le_bytes(), true, &mut unlimited())
                        .unwrap();
                    let Value::BigInt(value) = &decoded else {
                        panic!("expected BigInt element")
                    };
                    let expected = if element == ElementType::BigInt64 {
                        (bits as i64).to_string()
                    } else {
                        bits.to_string()
                    };
                    assert_eq!(value.to_radix(10, &mut unlimited()).unwrap(), expected);
                    let encoded = element
                        .encode(decoded.clone(), little_endian, &mut unlimited())
                        .unwrap();
                    assert_eq!(encoded, bytes(bits, 8, little_endian));
                    assert_eq!(
                        element
                            .decode(encoded, little_endian, &mut unlimited())
                            .unwrap(),
                        decoded
                    );
                }
            }
        }
        for (digits, expected) in [
            ("10000000000000001", 1_u64),
            ("123456789abcdef00123456789abcdef", 0x0123_4567_89ab_cdef),
            ("100000000000000000000000000000000", 0),
        ] {
            let value = BigInt::parse_digits(digits, 16, &mut unlimited()).unwrap();
            for (value, expected) in [
                (value.clone(), expected),
                (
                    value.neg(&mut unlimited()).unwrap(),
                    expected.wrapping_neg(),
                ),
            ] {
                for element in [ElementType::BigInt64, ElementType::BigUint64] {
                    assert_eq!(
                        element
                            .encode(Value::BigInt(value.clone()), true, &mut unlimited())
                            .unwrap(),
                        expected.to_le_bytes()
                    );
                }
            }
        }
    }

    #[test]
    fn codec_work_and_bigint_magnitude_limits_are_opt_in() {
        assert_eq!(
            ElementType::Float64.encode(Value::Number(1.0), true, &mut Budget::new(0, 7)),
            Err(Error::Limit)
        );
        assert_eq!(
            ElementType::Float64.decode([0; 8], true, &mut Budget::new(0, 7)),
            Err(Error::Limit)
        );
        assert_eq!(
            ElementType::BigInt64.decode([255; 8], true, &mut Budget::new(1, 100)),
            Ok(Value::BigInt(BigInt::from(-1)))
        );
        assert_eq!(
            ElementType::BigUint64.decode([255; 8], true, &mut Budget::new(63, 100)),
            Err(Error::Limit)
        );
        assert_eq!(
            ElementType::BigInt64.encode(
                Value::BigInt(BigInt::from(-1)),
                true,
                &mut Budget::new(0, 100)
            ),
            Ok([255; 8])
        );
    }
}
