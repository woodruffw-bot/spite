//! Exact conversion of radix integer literals to binary64.

/// Converts an unsigned base-2, base-8, or base-16 integer to binary64.
///
/// Accepts ASCII digits only; rejects empty or invalid input. Rounds once,
/// with ties to even.
pub fn parse_radix_integer(digits: &str, radix: u32) -> Option<f64> {
    if !matches!(radix, 2 | 8 | 16)
        || digits.is_empty()
        || !digits.bytes().all(|byte| char::from(byte).is_digit(radix))
    {
        return None;
    }
    let width = radix.trailing_zeros();
    let mut count = 0usize;
    let mut significand = 0u64;
    let mut guard = false;
    let mut sticky = false;
    for digit in digits
        .bytes()
        .filter_map(|byte| char::from(byte).to_digit(radix))
    {
        for shift in (0..width).rev() {
            let bit = digit & (1 << shift) != 0;
            if count == 0 && !bit {
                continue;
            }
            count += 1;
            // Every digit was validated above. At 1,025 significant bits the
            // value already exceeds binary64's finite range.
            if count > 1024 {
                return Some(f64::INFINITY);
            }
            if count <= 53 {
                significand = (significand << 1) | u64::from(bit);
            } else if count == 54 {
                guard = bit;
            } else {
                sticky |= bit;
            }
        }
    }
    if count <= 53 {
        return Some(significand as f64);
    }
    let mut exponent = count - 1;
    if guard && (sticky || significand & 1 != 0) {
        significand += 1;
    }
    if significand == 1 << 53 {
        significand >>= 1;
        exponent += 1;
    }
    if exponent > 1023 {
        return Some(f64::INFINITY);
    }
    Some(f64::from_bits(
        ((exponent as u64 + 1023) << 52) | (significand & ((1 << 52) - 1)),
    ))
}

#[cfg(test)]
mod tests {
    use super::parse_radix_integer;

    #[test]
    fn radix_integer_overflow_rounds_at_the_midpoint_and_validates_the_entire_input() {
        // The overflow midpoint is 2^1024 - 2^970; the integer immediately
        // below it rounds to MAX, and the tie rounds to Infinity.
        for (radix, below, midpoint, overflowing) in [
            (
                2,
                "1".repeat(53) + "0" + &"1".repeat(970),
                "1".repeat(54) + &"0".repeat(970),
                "1".to_owned() + &"0".repeat(1024),
            ),
            (
                8,
                "1".to_owned() + &"7".repeat(17) + "5" + &"7".repeat(323),
                "1".to_owned() + &"7".repeat(17) + "6" + &"0".repeat(323),
                "4".to_owned() + &"0".repeat(341),
            ),
            (
                16,
                "f".repeat(13) + "b" + &"f".repeat(242),
                "f".repeat(13) + "c" + &"0".repeat(242),
                "1".to_owned() + &"0".repeat(256),
            ),
        ] {
            assert_eq!(parse_radix_integer(&below, radix), Some(f64::MAX));
            assert_eq!(parse_radix_integer(&midpoint, radix), Some(f64::INFINITY));
            assert_eq!(
                parse_radix_integer(&overflowing, radix),
                Some(f64::INFINITY)
            );
            assert_eq!(parse_radix_integer(&(overflowing + "g"), radix), None);
            assert_eq!(
                parse_radix_integer(&("0".repeat(2048) + &below), radix),
                Some(f64::MAX)
            );
        }
    }
}
