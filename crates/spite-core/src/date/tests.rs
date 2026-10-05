use super::*;

#[test]
fn standard_utc_strings_cover_names_year_widths_and_range_boundaries() {
    let mut report = String::new();
    for text in [
        "1970-01-01T00:00:00.000Z",
        "1969-12-31T23:59:59.999Z",
        "0000-01-01",
        "0001-01-01",
        "0099-01-01",
        "9999-01-01",
        "+010000-01-01",
        "+100000-01-01",
        "-000001-01-01",
        "-000012-01-01",
        "-000123-01-01",
        "-001234-01-01",
        "-012345-01-01",
        "-123456-01-01",
        "-271821-04-20",
        "+275760-09-13",
    ] {
        let time = parse_date_time_string(&JsString::from(text))
            .unwrap()
            .utc_time_value()
            .unwrap() as i64;
        writeln!(
            report,
            "{text} => {}",
            format_utc_date_string(time).unwrap().to_utf8().unwrap()
        )
        .unwrap();
    }
    for month in 0..12 {
        let time = DateTimeString {
            year: 2024,
            month,
            day: 1,
            hour: 12,
            minute: 34,
            second: 56,
            millisecond: 789,
            zone: DateTimeZone::Utc,
        }
        .utc_time_value()
        .unwrap() as i64;
        writeln!(
            report,
            "{}",
            format_utc_date_string(time).unwrap().to_utf8().unwrap()
        )
        .unwrap();
    }
    for day in 0..7 {
        writeln!(
            report,
            "{}",
            format_utc_date_string(day * MS_PER_DAY)
                .unwrap()
                .to_utf8()
                .unwrap()
        )
        .unwrap();
    }
    for time in [i64::MIN, -MAX_TIME_VALUE - 1, MAX_TIME_VALUE + 1, i64::MAX] {
        assert_eq!(format_utc_date_string(time), None);
    }
    insta::assert_snapshot!(report);
}

#[test]
fn standard_utc_strings_round_trip_whole_seconds_throughout_the_clipped_domain() {
    let check = |time: i64| {
        let text = format_utc_date_string(time).unwrap();
        // The containing second is floored, including before the epoch.
        assert_eq!(
            parse_utc_date_string(&text),
            Some(time.div_euclid(1000) * 1000),
            "{text:?}"
        );
    };
    for time in [
        -MAX_TIME_VALUE,
        -MAX_TIME_VALUE + 1,
        -1001,
        -1000,
        -999,
        -1,
        0,
        1,
        999,
        1000,
        MAX_TIME_VALUE - 1,
        MAX_TIME_VALUE,
    ] {
        check(time);
    }
    for year in 0..=99 {
        let time = DateTimeString {
            year,
            month: 1,
            day: 28,
            hour: 12,
            minute: 34,
            second: 56,
            millisecond: 0,
            zone: DateTimeZone::Utc,
        }
        .utc_time_value()
        .unwrap() as i64;
        check(time);
        let parsed = parse_utc_date_string(&format_utc_date_string(time).unwrap()).unwrap();
        assert_eq!(UtcDateTime::from_time_value(parsed).unwrap().year, year);
    }
    let mut state = 2026_u64;
    for _ in 0..4096 {
        state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
        check((state % (2 * MAX_TIME_VALUE as u64 + 1)) as i64 - MAX_TIME_VALUE);
    }
}

#[test]
fn utc_string_parser_rejects_noncanonical_and_invalid_calendar_forms() {
    for text in [
        "Invalid Date",
        "Thu, 01 Jan 1970 00:00:00 GMT ",
        " Thu, 01 Jan 1970 00:00:00 GMT",
        "thu, 01 Jan 1970 00:00:00 GMT",
        "Thu 01 Jan 1970 00:00:00 GMT",
        "Thu, 1 Jan 1970 00:00:00 GMT",
        "Thu, 01 JAN 1970 00:00:00 GMT",
        "Wed, 01 Jan 1970 00:00:00 GMT",
        "Thu, 00 Jan 1970 00:00:00 GMT",
        "Thu, 32 Jan 1970 00:00:00 GMT",
        "Wed, 31 Apr 2024 00:00:00 GMT",
        "Fri, 29 Feb 1900 00:00:00 GMT",
        "Thu, 01 Jan 1970 24:00:00 GMT",
        "Thu, 01 Jan 1970 00:60:00 GMT",
        "Thu, 01 Jan 1970 00:00:60 GMT",
        "Thu, 01 Jan 1970 00:00:00 UTC",
        "Thu, 01 Jan 1970 00:00:00.000 GMT",
        "Thu, 01 Jan +1970 00:00:00 GMT",
        "Thu, 01 Jan 01970 00:00:00 GMT",
        "Thu, 01 Jan 197 00:00:00 GMT",
        "Sat, 01 Jan -0000 00:00:00 GMT",
        "Sat, 01 Jan 1000000 00:00:00 GMT",
        "Mon, 19 Apr -271821 23:59:59 GMT",
        "Sat, 13 Sep 275760 00:00:01 GMT",
    ] {
        assert_eq!(parse_utc_date_string(&JsString::from(text)), None, "{text}");
    }
    let text = JsString::from("Thu, 01 Jan 1970 00:00:00 GMT");
    for index in 0..text.len() {
        for unit in [0, 0xd800, 0xdc00, 0xffff, 0xff10] {
            let mut units = text.code_units().to_vec();
            units[index] = unit;
            assert_eq!(
                parse_utc_date_string(&JsString::from_code_units(units)),
                None
            );
        }
    }
}

#[test]
fn full_year_adjustment_follows_truncation_and_preserves_non_short_years() {
    for year in [0.0, -0.0, 0.99, -0.99] {
        assert_eq!(make_full_year(year), 1900.0);
    }
    for (year, expected) in [
        (1.99, 1901.0),
        (99.99, 1999.0),
        (100.99, 100.0),
        (-1.99, -1.0),
        (2000.99, 2000.0),
        (1e100, 1e100),
    ] {
        assert_eq!(make_full_year(year), expected);
    }
    assert!(make_full_year(f64::NAN).is_nan());
    assert_eq!(make_full_year(f64::INFINITY), f64::INFINITY);
    assert_eq!(make_full_year(f64::NEG_INFINITY), f64::NEG_INFINITY);
}

#[test]
fn clipping_truncates_toward_zero_and_canonicalizes_both_zero_signs() {
    for value in [
        0.0,
        -0.0,
        0.99,
        -0.99,
        f64::from_bits(1),
        -f64::from_bits(1),
    ] {
        assert_eq!(time_clip(value).to_bits(), 0);
    }
    for (input, expected) in [(1.99, 1.0), (-1.99, -1.0), (123.5, 123.0), (-123.5, -123.0)] {
        assert_eq!(time_clip(input), expected);
    }
}

#[test]
fn clipping_includes_both_specification_boundaries_and_rejects_the_next_millisecond() {
    for sign in [-1.0, 1.0] {
        let boundary = sign * MAX_TIME_VALUE as f64;
        assert_eq!(time_clip(boundary), boundary);
        assert_eq!(time_clip(boundary - sign), boundary - sign);
        assert!(time_clip(boundary + sign).is_nan());
    }
    for value in [
        f64::NAN,
        f64::INFINITY,
        f64::NEG_INFINITY,
        f64::MAX,
        -f64::MAX,
    ] {
        assert!(time_clip(value).is_nan());
    }
}

#[test]
fn utc_fields_match_epoch_extended_year_century_and_boundary_examples() {
    // These fixed UTC vectors were also checked against Node's Date getters.
    for (time, year, month, day, weekday, hour, minute, second, millisecond) in [
        (0, 1970, 0, 1, 4, 0, 0, 0, 0),
        (-1, 1969, 11, 31, 3, 23, 59, 59, 999),
        (1, 1970, 0, 1, 4, 0, 0, 0, 1),
        (-8_640_000_000_000_000, -271821, 3, 20, 2, 0, 0, 0, 0),
        (8_640_000_000_000_000, 275760, 8, 13, 6, 0, 0, 0, 0),
        (-62_167_219_200_000, 0, 0, 1, 6, 0, 0, 0, 0),
        (-2_203_891_200_000, 1900, 2, 1, 4, 0, 0, 0, 0),
        (951_782_400_000, 2000, 1, 29, 2, 0, 0, 0, 0),
        (4_107_542_400_000, 2100, 2, 1, 1, 0, 0, 0, 0),
    ] {
        assert_eq!(
            UtcDateTime::from_time_value(time),
            Some(UtcDateTime {
                year,
                month,
                day,
                weekday,
                hour,
                minute,
                second,
                millisecond,
            })
        );
    }
}

#[test]
fn utc_decomposition_checks_the_domain_without_overflowing_extreme_integers() {
    for time in [i64::MIN, i64::MAX, -MAX_TIME_VALUE - 1, MAX_TIME_VALUE + 1] {
        assert_eq!(UtcDateTime::from_time_value(time), None);
    }
    let first = UtcDateTime::from_time_value(-MAX_TIME_VALUE + 1).unwrap();
    assert_eq!(
        (first.year, first.month, first.day, first.millisecond),
        (-271821, 3, 20, 1)
    );
    let last = UtcDateTime::from_time_value(MAX_TIME_VALUE - 1).unwrap();
    assert_eq!(
        (last.year, last.month, last.day, last.weekday),
        (275760, 8, 12, 5)
    );
    assert_eq!(
        (last.hour, last.minute, last.second, last.millisecond),
        (23, 59, 59, 999)
    );
}

#[test]
fn every_day_in_positive_and_negative_four_hundred_year_cycles_rolls_over_correctly() {
    for (start_year, start_time) in [(-400, -74_790_000_000_000), (2000, 946_684_800_000)] {
        let mut time = start_time;
        let mut weekday = 6;
        let mut days = 0;
        for year in start_year..start_year + 400 {
            let leap = (year % 4 == 0 && year % 100 != 0) || year % 400 == 0;
            for (month, length) in [
                31,
                if leap { 29 } else { 28 },
                31,
                30,
                31,
                30,
                31,
                31,
                30,
                31,
                30,
                31,
            ]
            .into_iter()
            .enumerate()
            {
                for day in 1..=length {
                    assert_eq!(
                        UtcDateTime::from_time_value(time),
                        Some(UtcDateTime {
                            year,
                            month: month as u8,
                            day,
                            weekday,
                            hour: 0,
                            minute: 0,
                            second: 0,
                            millisecond: 0,
                        })
                    );
                    time += MS_PER_DAY;
                    weekday = (weekday + 1) % 7;
                    days += 1;
                }
            }
        }
        assert_eq!(days, 146_097);
        assert_eq!(time - start_time, 12_622_780_800_000);
        assert_eq!(
            UtcDateTime::from_time_value(time).unwrap().year,
            start_year + 400
        );
    }
}

#[test]
fn make_time_truncates_each_component_and_allows_signed_rollover() {
    assert_eq!(make_time(1.9, 2.9, 3.9, 4.9), 3_723_004.0);
    assert_eq!(make_time(-1.9, -2.9, -3.9, -4.9), -3_723_004.0);
    assert_eq!(make_time(24.0, 0.0, 0.0, 0.0), MS_PER_DAY as f64);
    assert_eq!(make_time(0.0, 0.0, 0.0, -1.0), -1.0);
    assert_eq!(make_time(-0.0, -0.0, -0.0, -0.0).to_bits(), 0);
}

#[test]
fn make_time_preserves_number_operation_order_and_finite_input_overflow() {
    let hour = (1_u64 << 52) as f64;
    // The large hour and minute products cancel before adding small components.
    assert_eq!(make_time(hour, -60.0 * hour, 1.0, 1.0), 1001.0);
    assert_eq!(make_time(f64::MAX, 0.0, 0.0, 0.0), f64::INFINITY);
    for invalid in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        for index in 0..4 {
            let mut components = [0.0; 4];
            components[index] = invalid;
            assert!(make_time(components[0], components[1], components[2], components[3]).is_nan());
        }
    }
}

#[test]
fn make_date_preserves_signed_zero_and_defers_clipping_of_finite_results() {
    assert_eq!(make_date(-0.0, -0.0).to_bits(), (-0.0_f64).to_bits());
    assert_eq!(make_date(-1.0, MS_PER_DAY as f64 - 1.0), -1.0);
    let outside = make_date(100_000_001.0, 0.0);
    assert_eq!(outside, MAX_TIME_VALUE as f64 + MS_PER_DAY as f64);
    assert!(time_clip(outside).is_nan());
    assert_eq!(make_date(0.0, f64::MAX), f64::MAX);
    for invalid in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert!(make_date(invalid, 0.0).is_nan());
        assert!(make_date(0.0, invalid).is_nan());
    }
    assert!(make_date(f64::MAX, -f64::MAX).is_nan());
}

#[test]
fn canonical_iso_format_snapshot() {
    use std::fmt::Write;

    let mut report = String::new();
    for time in [
        -MAX_TIME_VALUE,
        -MAX_TIME_VALUE + 1,
        -62_198_755_200_000,
        -62_167_219_200_000,
        -1,
        0,
        1,
        951_782_400_000,
        253_402_300_799_999,
        253_402_300_800_000,
        MAX_TIME_VALUE - 1,
        MAX_TIME_VALUE,
        -MAX_TIME_VALUE - 1,
        MAX_TIME_VALUE + 1,
        i64::MIN,
        i64::MAX,
    ] {
        writeln!(report, "{time} => {:?}", format_iso_date_time(time)).unwrap();
    }
    insta::assert_snapshot!(report);
}

#[test]
fn canonical_iso_strings_round_trip_across_the_complete_clipped_domain() {
    let mut state = 2026_u64;
    let width = (2 * MAX_TIME_VALUE + 1) as u64;
    for _ in 0..4096 {
        state = state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1);
        let time = (state % width) as i64 - MAX_TIME_VALUE;
        let text = format_iso_date_time(time).unwrap();
        assert!(matches!(text.len(), 24 | 27));
        assert!(text.code_units().iter().all(|unit| *unit <= 0x7f));
        assert_eq!(
            parse_date_time_string(&text).unwrap().utc_time_value(),
            Some(time as f64)
        );
    }
}
