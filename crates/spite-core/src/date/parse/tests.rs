use super::*;
use std::fmt::Write;

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
    insta::assert_snapshot!(report);
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
    insta::assert_snapshot!(report);
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
fn utc_forms_convert_defaults_extended_years_and_offsets_without_short_year_adjustment() {
    for (source, expected) in [
        ("1970", 0.0),
        ("1970T00:00Z", 0.0),
        ("1970-02", 2_678_400_000.0),
        ("1969-12-31T23:59:59.999Z", -1.0),
        ("1970-01-01T00:00+05:30", -19_800_000.0),
        ("1970-01-01T00:00-05:30", 19_800_000.0),
        ("0000", -62_167_219_200_000.0),
        ("+000000", -62_167_219_200_000.0),
        ("-000001", -62_198_755_200_000.0),
    ] {
        assert_eq!(parsed(source).utc_time_value(), Some(expected), "{source}");
    }
    assert_eq!(parsed("1970").utc_time_value().unwrap().to_bits(), 0);
    for source in ["0001", "0099", "+000001", "-000001"] {
        let value = parsed(source);
        let time = value.utc_time_value().unwrap();
        assert_eq!(
            super::super::UtcDateTime::from_time_value(time as i64)
                .unwrap()
                .year,
            value.year
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

#[test]
fn local_forms_require_resolution_instead_of_substituting_a_utc_instant() {
    for source in [
        "1970T00:00",
        "2000-02-29T12:34:56.789",
        "+275760-09-13T00:00",
    ] {
        assert_eq!(parsed(source).utc_time_value(), None);
    }
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
