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
