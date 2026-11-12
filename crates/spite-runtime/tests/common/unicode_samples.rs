//! Ordered Unicode samples for RegExp property tests.
//!
//! These fixture ranges remain independent of the engine's property tables.

use std::ops::RangeInclusive;

/// Expands inclusive scalar ranges without sorting or removing repeated points.
pub fn from_ranges(ranges: &[RangeInclusive<u32>]) -> String {
    let mut sample = String::new();
    for range in ranges {
        for point in range.clone() {
            let character = char::from_u32(point).expect("sample contains a non-scalar value");
            sample.push(character);
        }
    }
    sample
}

#[cfg(test)]
mod tests {
    use super::from_ranges;
    use std::ops::RangeInclusive;
    use std::slice;
    use test_case::test_case;

    #[test_case(&[], ""; "empty_sample")]
    #[test_case(slice::from_ref(&(0x41..=0x43)), "ABC"; "inclusive_range")]
    #[test_case(&[0x42..=0x42, 0x41..=0x42], "BAB"; "order_and_repeated_points")]
    #[test_case(slice::from_ref(&(0x10000..=0x10001)), "\u{10000}\u{10001}"; "supplementary_characters")]
    fn sample_preserves_characters(ranges: &[RangeInclusive<u32>], expected: &str) {
        assert_eq!(from_ranges(ranges), expected);
    }
}
