use super::{DateTimeString, DateTimeZone, JsString, parse_date_time_string};
use std::fmt::Write;
use test_case::test_case;

#[test]
fn date_time_forms_snapshot_defaults_expanded_years_midnight_and_zones() {
    let mut report = String::new();
    for source in [
        "1970",
        "1970-02",
        "1970-02-03",
        "0000",
        "+000000-01-01",
        "-000001-01-01",
        "+001970-01-01",
        "-271821-04-20T00:00:00Z",
        "+275760-09-13T00:00:00.000Z",
        "+999999-12-31T23:59:59.999Z",
        "1970T00:00",
        "1970-02T12:34",
        "1970-02-03T12:34",
        "1970-02-03T12:34:56",
        "1970-02-03T12:34:56.007",
        "1970-02-03T12:34Z",
        "1970-02-03T12:34:56Z",
        "1970-02-03T12:34:56.999Z",
        "1995-02-04T24:00",
        "1995-02-04T24:00:00.000Z",
        "1970-01-01T00:00+00:00",
        "1970-01-01T00:00-00:00",
        "1970-01-01T00:00+05:30",
        "1970-01-01T00:00-23:59",
        "2000-02-30",
    ] {
        let parsed = parse_date_time_string(&JsString::from(source)).unwrap();
        writeln!(report, "{source:?} => {parsed:?}").unwrap();
    }
    insta::assert_snapshot!(report, @r#"
        "1970" => DateTimeString { year: 1970, month: 0, day: 1, hour: 0, minute: 0, second: 0, millisecond: 0, zone: Utc }
        "1970-02" => DateTimeString { year: 1970, month: 1, day: 1, hour: 0, minute: 0, second: 0, millisecond: 0, zone: Utc }
        "1970-02-03" => DateTimeString { year: 1970, month: 1, day: 3, hour: 0, minute: 0, second: 0, millisecond: 0, zone: Utc }
        "0000" => DateTimeString { year: 0, month: 0, day: 1, hour: 0, minute: 0, second: 0, millisecond: 0, zone: Utc }
        "+000000-01-01" => DateTimeString { year: 0, month: 0, day: 1, hour: 0, minute: 0, second: 0, millisecond: 0, zone: Utc }
        "-000001-01-01" => DateTimeString { year: -1, month: 0, day: 1, hour: 0, minute: 0, second: 0, millisecond: 0, zone: Utc }
        "+001970-01-01" => DateTimeString { year: 1970, month: 0, day: 1, hour: 0, minute: 0, second: 0, millisecond: 0, zone: Utc }
        "-271821-04-20T00:00:00Z" => DateTimeString { year: -271821, month: 3, day: 20, hour: 0, minute: 0, second: 0, millisecond: 0, zone: Utc }
        "+275760-09-13T00:00:00.000Z" => DateTimeString { year: 275760, month: 8, day: 13, hour: 0, minute: 0, second: 0, millisecond: 0, zone: Utc }
        "+999999-12-31T23:59:59.999Z" => DateTimeString { year: 999999, month: 11, day: 31, hour: 23, minute: 59, second: 59, millisecond: 999, zone: Utc }
        "1970T00:00" => DateTimeString { year: 1970, month: 0, day: 1, hour: 0, minute: 0, second: 0, millisecond: 0, zone: Local }
        "1970-02T12:34" => DateTimeString { year: 1970, month: 1, day: 1, hour: 12, minute: 34, second: 0, millisecond: 0, zone: Local }
        "1970-02-03T12:34" => DateTimeString { year: 1970, month: 1, day: 3, hour: 12, minute: 34, second: 0, millisecond: 0, zone: Local }
        "1970-02-03T12:34:56" => DateTimeString { year: 1970, month: 1, day: 3, hour: 12, minute: 34, second: 56, millisecond: 0, zone: Local }
        "1970-02-03T12:34:56.007" => DateTimeString { year: 1970, month: 1, day: 3, hour: 12, minute: 34, second: 56, millisecond: 7, zone: Local }
        "1970-02-03T12:34Z" => DateTimeString { year: 1970, month: 1, day: 3, hour: 12, minute: 34, second: 0, millisecond: 0, zone: Utc }
        "1970-02-03T12:34:56Z" => DateTimeString { year: 1970, month: 1, day: 3, hour: 12, minute: 34, second: 56, millisecond: 0, zone: Utc }
        "1970-02-03T12:34:56.999Z" => DateTimeString { year: 1970, month: 1, day: 3, hour: 12, minute: 34, second: 56, millisecond: 999, zone: Utc }
        "1995-02-04T24:00" => DateTimeString { year: 1995, month: 1, day: 4, hour: 24, minute: 0, second: 0, millisecond: 0, zone: Local }
        "1995-02-04T24:00:00.000Z" => DateTimeString { year: 1995, month: 1, day: 4, hour: 24, minute: 0, second: 0, millisecond: 0, zone: Utc }
        "1970-01-01T00:00+00:00" => DateTimeString { year: 1970, month: 0, day: 1, hour: 0, minute: 0, second: 0, millisecond: 0, zone: OffsetMinutes(0) }
        "1970-01-01T00:00-00:00" => DateTimeString { year: 1970, month: 0, day: 1, hour: 0, minute: 0, second: 0, millisecond: 0, zone: OffsetMinutes(0) }
        "1970-01-01T00:00+05:30" => DateTimeString { year: 1970, month: 0, day: 1, hour: 0, minute: 0, second: 0, millisecond: 0, zone: OffsetMinutes(330) }
        "1970-01-01T00:00-23:59" => DateTimeString { year: 1970, month: 0, day: 1, hour: 0, minute: 0, second: 0, millisecond: 0, zone: OffsetMinutes(-1439) }
        "2000-02-30" => DateTimeString { year: 2000, month: 1, day: 30, hour: 0, minute: 0, second: 0, millisecond: 0, zone: Utc }
        "#);
}

#[test]
fn rejected_date_time_syntax_snapshot() {
    let mut report = String::new();
    for source in [
        "",
        "197",
        "19700",
        "1970 ",
        " 1970",
        "1970\n",
        "+1970",
        "-1970",
        "+00000",
        "+0000000",
        "-000000",
        "-000000-01-01",
        "1970-1",
        "1970-001",
        "1970-00",
        "1970-13",
        "1970-01-1",
        "1970-01-001",
        "1970-01-00",
        "1970-01-32",
        "1970--01",
        "1970-01-01-01",
        "1970Z",
        "1970-01-01+00:00",
        "1970-01-01t00:00Z",
        "1970-01-01T0:00",
        "1970-01-01T00:0",
        "1970-01-01T00:00:0",
        "1970-01-01T25:00",
        "1970-01-01T23:60",
        "1970-01-01T23:59:60",
        "1970-01-01T24:01",
        "1970-01-01T24:00:01",
        "1970-01-01T24:00:00.001",
        "1970-01-01T00:00.000",
        "1970-01-01T00:00:00.",
        "1970-01-01T00:00:00.1",
        "1970-01-01T00:00:00.01",
        "1970-01-01T00:00:00.0000",
        "1970-01-01T00:00z",
        "1970-01-01T00:00ZZ",
        "1970-01-01T00:00Z+00:00",
        "1970-01-01T00:00+0:00",
        "1970-01-01T00:00+00:0",
        "1970-01-01T00:00+0000",
        "1970-01-01T00:00+24:00",
        "1970-01-01T00:00-24:00",
        "1970-01-01T00:00+00:60",
        "1970-01-01T00:00+00:00:00",
        "1970-01-01T00:00+00:00x",
        "１９７０",
        "1970-０１-01",
        "1970-01-01T00:00\0",
    ] {
        let parsed = parse_date_time_string(&JsString::from(source));
        assert_eq!(parsed, None, "{source:?}");
        writeln!(report, "{source:?} => {parsed:?}").unwrap();
    }
    insta::assert_snapshot!(report, @r#"
        "" => None
        "197" => None
        "19700" => None
        "1970 " => None
        " 1970" => None
        "1970\n" => None
        "+1970" => None
        "-1970" => None
        "+00000" => None
        "+0000000" => None
        "-000000" => None
        "-000000-01-01" => None
        "1970-1" => None
        "1970-001" => None
        "1970-00" => None
        "1970-13" => None
        "1970-01-1" => None
        "1970-01-001" => None
        "1970-01-00" => None
        "1970-01-32" => None
        "1970--01" => None
        "1970-01-01-01" => None
        "1970Z" => None
        "1970-01-01+00:00" => None
        "1970-01-01t00:00Z" => None
        "1970-01-01T0:00" => None
        "1970-01-01T00:0" => None
        "1970-01-01T00:00:0" => None
        "1970-01-01T25:00" => None
        "1970-01-01T23:60" => None
        "1970-01-01T23:59:60" => None
        "1970-01-01T24:01" => None
        "1970-01-01T24:00:01" => None
        "1970-01-01T24:00:00.001" => None
        "1970-01-01T00:00.000" => None
        "1970-01-01T00:00:00." => None
        "1970-01-01T00:00:00.1" => None
        "1970-01-01T00:00:00.01" => None
        "1970-01-01T00:00:00.0000" => None
        "1970-01-01T00:00z" => None
        "1970-01-01T00:00ZZ" => None
        "1970-01-01T00:00Z+00:00" => None
        "1970-01-01T00:00+0:00" => None
        "1970-01-01T00:00+00:0" => None
        "1970-01-01T00:00+0000" => None
        "1970-01-01T00:00+24:00" => None
        "1970-01-01T00:00-24:00" => None
        "1970-01-01T00:00+00:60" => None
        "1970-01-01T00:00+00:00:00" => None
        "1970-01-01T00:00+00:00x" => None
        "１９７０" => None
        "1970-０１-01" => None
        "1970-01-01T00:00\0" => None
        "#);
}

#[test]
fn date_only_forms_use_utc_and_absent_datetime_offsets_require_local_resolution() {
    for date in ["1970", "1970-01", "1970-01-01"] {
        assert_eq!(
            parse_date_time_string(&JsString::from(date)).unwrap().zone,
            DateTimeZone::Utc
        );
        for time in ["T00:00", "T00:00:00", "T00:00:00.000"] {
            let local = format!("{date}{time}");
            assert_eq!(
                parse_date_time_string(&JsString::from(local.as_str()))
                    .unwrap()
                    .zone,
                DateTimeZone::Local
            );
            let utc = format!("{local}Z");
            assert_eq!(
                parse_date_time_string(&JsString::from(utc.as_str()))
                    .unwrap()
                    .zone,
                DateTimeZone::Utc
            );
        }
    }
}

#[test]
fn utc_offset_hour_and_minute_boundaries_keep_their_sign() {
    for (text, expected) in [
        ("+00:01", 1),
        ("-00:01", -1),
        ("+23:59", 1439),
        ("-23:59", -1439),
    ] {
        let source = format!("1970-01-01T00:00{text}");
        assert_eq!(
            parse_date_time_string(&JsString::from(source.as_str()))
                .unwrap()
                .zone,
            DateTimeZone::OffsetMinutes(expected)
        );
    }
}

#[test]
fn unpaired_surrogates_cannot_become_format_elements_or_disappear_in_conversion() {
    for source in [
        JsString::from_code_units(vec![0xd800]),
        JsString::from_code_units(vec![b'1' as u16, b'9' as u16, 0xdc00, b'0' as u16]),
        JsString::from_code_units("1970".encode_utf16().chain([0xd800, 0xdc00]).collect()),
    ] {
        assert_eq!(parse_date_time_string(&source), None);
    }
}

fn parsed(source: &str) -> DateTimeString {
    parse_date_time_string(&JsString::from(source)).unwrap()
}

#[test]
fn nominal_calendar_milliseconds_normalize_before_zone_conversion_and_clipping() {
    for (source, expected) in [
        ("1970-01-01T00:00", 0),
        ("1970-01-01T00:00Z", 0),
        ("1970-01-01T00:00+05:30", 0),
        ("1970-01-01T00:00-05:30", 0),
        ("0000-02-29T00:00:00.001", -62_162_121_599_999),
        ("2000-02-31T24:00", 952_041_600_000),
        ("-271821-04-19T23:00-01:00", -8_640_000_003_600_000),
        ("+275760-09-13T01:00+01:00", 8_640_000_003_600_000),
    ] {
        assert_eq!(
            parsed(source).nominal_epoch_milliseconds(),
            Some(expected),
            "{source}"
        );
    }
    assert_eq!(parsed("1970-01-01T00:00").utc_time_value(), None);
    assert_eq!(
        parsed("-271821-04-19T23:00-01:00").utc_time_value(),
        Some(-8_640_000_000_000_000.0)
    );
    assert_eq!(
        parsed("+275760-09-13T01:00+01:00").utc_time_value(),
        Some(8_640_000_000_000_000.0)
    );
    let minimum = DateTimeString {
        year: i32::MIN,
        ..parsed("1970")
    };
    let maximum = DateTimeString {
        year: i32::MAX,
        month: 11,
        day: 31,
        hour: 24,
        ..parsed("1970")
    };
    assert_eq!(
        minimum.nominal_epoch_milliseconds(),
        Some(-67_768_100_567_971_200_000)
    );
    assert_eq!(
        maximum.nominal_epoch_milliseconds(),
        Some(67_767_976_233_532_800_000)
    );
    assert!(minimum.utc_time_value().unwrap().is_nan());
    assert!(maximum.utc_time_value().unwrap().is_nan());
}

#[test]
fn nominal_calendar_validation_keeps_invalid_fields_distinct_from_zone_selection() {
    let base = parsed("1970-01-01T00:00");
    for invalid in [
        DateTimeString { month: 12, ..base },
        DateTimeString { day: 0, ..base },
        DateTimeString { day: 32, ..base },
        DateTimeString { hour: 25, ..base },
        DateTimeString { minute: 60, ..base },
        DateTimeString { second: 60, ..base },
        DateTimeString {
            millisecond: 1000,
            ..base
        },
        DateTimeString {
            hour: 24,
            minute: 1,
            ..base
        },
        DateTimeString {
            hour: 24,
            second: 1,
            ..base
        },
        DateTimeString {
            hour: 24,
            millisecond: 1,
            ..base
        },
    ] {
        assert_eq!(invalid.nominal_epoch_milliseconds(), None);
        assert!(invalid.utc_time_value().unwrap().is_nan());
    }
    let invalid_zone = DateTimeString {
        zone: DateTimeZone::OffsetMinutes(i16::MAX),
        ..base
    };
    assert_eq!(invalid_zone.nominal_epoch_milliseconds(), Some(0));
    assert!(invalid_zone.utc_time_value().unwrap().is_nan());
}

#[test_case("1970" => Some(0.0); "year_defaults_to_january_first")]
#[test_case("1970T00:00Z" => Some(0.0); "time_with_default_month_and_day")]
#[test_case("1970-02" => Some(2_678_400_000.0); "month_defaults_to_first_day")]
#[test_case("1969-12-31T23:59:59.999Z" => Some(-1.0); "last_millisecond_before_epoch")]
#[test_case("1970-01-01T00:00+05:30" => Some(-19_800_000.0); "positive_offset")]
#[test_case("1970-01-01T00:00-05:30" => Some(19_800_000.0); "negative_offset")]
#[test_case("0000" => Some(-62_167_219_200_000.0); "year_zero")]
#[test_case("+000000" => Some(-62_167_219_200_000.0); "expanded_year_zero")]
#[test_case("-000001" => Some(-62_198_755_200_000.0); "negative_year")]
fn utc_forms_convert_to_expected_milliseconds(source: &str) -> Option<f64> {
    parsed(source).utc_time_value()
}

#[test]
fn utc_conversion_preserves_positive_zero_and_literal_years() {
    assert_eq!(parsed("1970").utc_time_value().unwrap().to_bits(), 0);
    for source in ["0001", "0099", "+000001", "-000001"] {
        let value = parsed(source);
        let time = value.utc_time_value().unwrap();
        assert_eq!(
            super::super::UtcDateTime::from_time_value(time as i64)
                .unwrap()
                .year,
            value.year,
            "year must not receive Date constructor's 1900 adjustment: {source}",
        );
    }
}

#[test]
fn end_of_day_and_calendar_day_elements_normalize_before_conversion() {
    assert_eq!(
        parsed("1995-02-04T24:00Z").utc_time_value(),
        Some(791_942_400_000.0)
    );
    assert_eq!(
        parsed("1995-02-04T24:00Z").utc_time_value(),
        parsed("1995-02-05T00:00Z").utc_time_value()
    );
    assert_eq!(
        parsed("2000-02-30").utc_time_value(),
        Some(951_868_800_000.0)
    );
    assert_eq!(
        parsed("1900-02-29").utc_time_value(),
        parsed("1900-03-01").utc_time_value()
    );
    assert_eq!(
        parsed("2000-02-29").utc_time_value(),
        Some(951_782_400_000.0)
    );
}

#[test]
fn utc_range_checks_follow_offset_adjustment_at_both_endpoints() {
    for (source, expected) in [
        ("-271821-04-20T00:00:00Z", -8_640_000_000_000_000.0),
        ("+275760-09-13T00:00:00Z", 8_640_000_000_000_000.0),
        ("+275760-09-12T24:00Z", 8_640_000_000_000_000.0),
        ("-271821-04-19T23:59:59.999-00:01", -8_639_999_999_940_001.0),
        ("+275760-09-13T00:00:00.001+00:01", 8_639_999_999_940_001.0),
    ] {
        assert_eq!(parsed(source).utc_time_value(), Some(expected), "{source}");
    }
    for source in [
        "-271821-04-19T23:59:59.999Z",
        "+275760-09-13T00:00:00.001Z",
        "-271821-04-20T00:00+00:01",
        "+275760-09-13T00:00-00:01",
        "+999999",
        "-999999",
    ] {
        assert!(
            parsed(source).utc_time_value().unwrap().is_nan(),
            "{source}"
        );
    }
}

#[test_case("1970T00:00"; "epoch_time_without_zone")]
#[test_case("2000-02-29T12:34:56.789"; "leap_day_with_fractional_seconds")]
#[test_case("+275760-09-13T00:00"; "expanded_year_without_zone")]
fn local_forms_require_time_zone_resolution(source: &str) {
    assert_eq!(parsed(source).utc_time_value(), None);
}

#[test]
fn externally_constructed_invalid_records_and_extreme_years_do_not_overflow() {
    let base = parsed("1970-01-01T00:00Z");
    let invalid = [
        DateTimeString { month: 12, ..base },
        DateTimeString { day: 0, ..base },
        DateTimeString { day: 32, ..base },
        DateTimeString { hour: 25, ..base },
        DateTimeString { minute: 60, ..base },
        DateTimeString { second: 60, ..base },
        DateTimeString {
            millisecond: 1000,
            ..base
        },
        DateTimeString {
            hour: 24,
            minute: 1,
            ..base
        },
        DateTimeString {
            hour: 24,
            second: 1,
            ..base
        },
        DateTimeString {
            hour: 24,
            millisecond: 1,
            ..base
        },
        DateTimeString {
            zone: DateTimeZone::OffsetMinutes(1440),
            ..base
        },
        DateTimeString {
            zone: DateTimeZone::OffsetMinutes(i16::MIN),
            ..base
        },
        DateTimeString {
            year: i32::MIN,
            ..base
        },
        DateTimeString {
            year: i32::MAX,
            ..base
        },
    ];
    for value in invalid {
        assert!(value.utc_time_value().unwrap().is_nan(), "{value:?}");
    }
}
