use super::*;

fn day(year: f64, month: f64, date: f64) -> f64 {
    make_day(year, month, date, &mut Budget::with_limits(None, None)).unwrap()
}

#[test]
fn components_truncate_normalize_and_keep_literal_short_years() {
    for (year, month, date, expected) in [
        (1970.0, 0.0, 1.0, 0.0),
        (1970.9, -0.9, 1.9, 0.0),
        (1970.0, 0.0, -0.9, -1.0),
        (1970.0, -1.9, 1.0, -31.0),
        (1969.0, 12.9, 1.0, 0.0),
        (1970.0, 1.0, 0.0, 30.0),
        (0.0, 0.0, 1.0, -719528.0),
        (-0.9, 0.9, 1.9, -719528.0),
        (0.0, -1.0, 1.0, -719559.0),
        (2000.0, 1.0, 30.0, 11017.0),
        (1900.0, 1.0, 29.0, -25508.0),
        (-271821.0, 3.0, 1.0, -100000019.0),
        (-271821.0, 3.0, 20.0, -100000000.0),
        (275760.0, 8.0, 13.0, 100000000.0),
    ] {
        assert_eq!(day(year, month, date), expected, "{year} {month} {date}");
    }
}

#[test]
fn nonfinite_components_and_normalized_year_overflow_produce_nan() {
    for invalid in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        for index in 0..3 {
            let mut args = [1970.0, 0.0, 1.0];
            args[index] = invalid;
            assert!(day(args[0], args[1], args[2]).is_nan());
        }
    }
    assert!(day(f64::MAX, f64::MAX, 1.0).is_nan());
    assert!(day(-f64::MAX, -f64::MAX, 1.0).is_nan());
    // MakeDay does not reject a nonfinite *result* after finite day addition.
    assert_eq!(day(1e295, 0.0, f64::MAX), f64::INFINITY);
}

#[test]
fn mathematical_month_floor_precedes_number_rounding_and_year_cancellation() {
    // floor(month / 12) in the mathematical domain is 2^54+2, which rounds
    // to 2^54. Number division first rounds up to 2^54+4 instead.
    assert_eq!(
        day(-((1_u64 << 54) as f64), 216172782113783840.0, 1.0),
        -719284.0
    );
    assert_eq!(
        day((1_u64 << 54) as f64 + 4.0, -216172782113783840.0, 1.0),
        -719407.0
    );
    assert_eq!(day(-f64::MAX / 12.0, f64::MAX, 1.0), -719284.0);
    assert_eq!(day(f64::MAX / 12.0, -f64::MAX, 1.0), -719407.0);
    let mut state = 2026_u64;
    for index in 0..512 {
        state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
        // Above 2^85, division by 12 rounds the same as its floored integer
        // quotient: its repeating thirds stay far from Number halfway cases.
        let exponent = 1108 + index % (2047 - 1108);
        let bits =
            (state & ((1_u64 << 52) - 1)) | ((exponent as u64) << 52) | (state & (1_u64 << 63));
        let month = f64::from_bits(bits);
        let normalized = month.rem_euclid(12.0) as u8;
        let expected = super::super::DateTimeString {
            year: 0,
            month: normalized,
            day: 1,
            hour: 0,
            minute: 0,
            second: 0,
            millisecond: 0,
            zone: super::super::DateTimeZone::Utc,
        }
        .utc_time_value()
        .unwrap()
            / MS;
        assert_eq!(day(-month / 12.0, month, 1.0), expected, "{month}");
    }
}

#[test]
fn large_finite_calendar_witnesses_allow_day_cancellation_before_timeclip() {
    for (year, first_day) in [
        (5e9, 1826211780472.0),
        (1e12, 365242499280472.0),
        (1e15, 3.6524249999928045e17),
        ((1_u64 << 53) as f64, 3.2898119737990175e18),
        (1e200, 3.652425e202),
    ] {
        assert_eq!(day(year, 0.0, -first_day), -1.0, "{year}");
    }
    assert_eq!(day(5e9, 0.0, 1.0 - 1826211780472.0), 0.0);
    // Rounded year-boundary plateaus cannot identify the requested year.
    assert!(day(1e297, 0.0, -3.6524250000000004e299).is_nan());
    // Day rounding skips this January boundary's day number inside its year.
    assert!(day(1e100, 0.0, -3.652425e102).is_nan());
    // DayFromYear + a small month prefix cannot represent March's first day.
    assert!(day(1e200, 2.0, -3.652425e202).is_nan());
}

#[test]
fn opt_in_integer_work_failure_is_an_error_instead_of_a_calendar_nan() {
    assert_eq!(
        make_day(1970.0, 0.0, 1.0, &mut Budget::with_limits(None, Some(0))),
        Err(Error::Limit)
    );
    assert_eq!(
        make_day(
            -f64::MAX / 12.0,
            f64::MAX,
            1.0,
            &mut Budget::with_limits(None, Some(1))
        ),
        Err(Error::Limit)
    );
    assert_eq!(day(-f64::MAX / 12.0, f64::MAX, 1.0), -719284.0);
}

#[test]
fn gregorian_integer_arithmetic_rounds_once_at_number_conversion() {
    // Exact integer Gregorian values independently converted by Python.
    for (year, expected_bits) in [
        (5000000000.0, 0x427a932b45778000),
        (-5000000000.0, 0xc27a932ca4cc8000),
        (9007199254740992.0, 0x43c6d3e147ae0efe),
        (-9007199254740992.0, 0xc3c6d3e147ae19f8),
        (9007199254740994.0, 0x43c6d3e147ae0eff),
        (-9007199254740994.0, 0xc3c6d3e147ae19fa),
        (1.152921504606847e+18, 0x4436d3e147ae1470),
        (-1.152921504606847e+18, 0xc436d3e147ae1486),
        (1e+100, 0x553a177b2d0cf014),
        (-1e+100, 0xd53a177b2d0cf014),
        (1e+200, 0x69fdd2900fd8d50f),
        (-1e+200, 0xe9fdd2900fd8d50f),
        (1e+297, 0x7e2173d1a8e4693f),
        (-1e+297, 0xfe2173d1a8e4693f),
        (1.7976931348623157e+308, 0x7ff0000000000000),
        (-1.7976931348623157e+308, 0xfff0000000000000),
    ] {
        assert_eq!(
            day_from_year(year, &mut Budget::with_limits(None, None))
                .unwrap()
                .to_bits(),
            expected_bits,
            "{year}"
        );
    }
}
