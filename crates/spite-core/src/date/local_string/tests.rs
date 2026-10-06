use super::*;

#[test]
fn local_strings_preserve_calendar_widths_and_exact_offsets() {
    let mut report = String::new();
    for time in [-MAX_TIME_VALUE, -1, 0, MAX_TIME_VALUE] {
        for offset in [i32::MIN, -17762, -3600, 0, 3600, 86400, i32::MAX] {
            writeln!(
                report,
                "{time} {offset} => {}",
                format_date_string(time, offset).unwrap().to_utf8().unwrap()
            )
            .unwrap();
        }
    }
    for text in [
        "0000-01-01",
        "0001-01-01",
        "0099-01-01",
        "9999-01-01",
        "+010000-01-01",
        "-000001-01-01",
    ] {
        let time = super::super::parse_date_time_string(&JsString::from(text))
            .unwrap()
            .utc_time_value()
            .unwrap() as i64;
        writeln!(
            report,
            "{text} => {}",
            format_date_string(time, 0).unwrap().to_utf8().unwrap()
        )
        .unwrap();
    }
    for local in [i64::MIN, i64::MAX] {
        writeln!(
            report,
            "native {local} => {} | {}",
            format_local_date_string(local).to_utf8().unwrap(),
            format_local_time_string(local, 0).to_utf8().unwrap()
        )
        .unwrap();
    }
    insta::assert_snapshot!(report);
}

#[test]
fn local_output_round_trips_whole_seconds_for_every_native_offset_extreme() {
    let check = |time, offset| {
        let text = format_date_string(time, offset).unwrap();
        assert_eq!(
            parse_local_date_string(&text),
            Some(time.div_euclid(1000) * 1000),
            "{time} {offset}: {text:?}"
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
        for offset in [
            i32::MIN,
            -90000,
            -86400,
            -86399,
            -3600,
            -60,
            -1,
            0,
            1,
            60,
            3600,
            86399,
            86400,
            90000,
            i32::MAX,
        ] {
            check(time, offset);
        }
    }
    let mut state = 2026_u64;
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
        for offset in [-17762, 0, 7200] {
            check(time, offset);
        }
    }
    for _ in 0..4096 {
        state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
        let time = (state % (2 * MAX_TIME_VALUE as u64 + 1)) as i64 - MAX_TIME_VALUE;
        let offset = (state >> 32) as i32;
        check(time, offset);
    }
    for time in [i64::MIN, -MAX_TIME_VALUE - 1, MAX_TIME_VALUE + 1, i64::MAX] {
        assert_eq!(format_date_string(time, 0), None);
    }
}

#[test]
fn local_parser_checks_calendar_weekday_exact_offset_and_format_consistency() {
    for text in [
        "Invalid Date",
        "Thu Jan 01 1970 00:00:00 GMT-0000",
        "Wed Jan 01 1970 00:00:00 GMT+0000",
        "Fri Feb 29 1900 00:00:00 GMT+0000",
        "Wed Apr 31 2024 00:00:00 GMT+0000",
        "Thu Jan 01 1970 24:00:00 GMT+0000",
        "Thu Jan 01 1970 00:60:00 GMT+0000",
        "Thu Jan 01 1970 00:00:60 GMT+0000",
        "Thu Jan 01 1970 00:00:00.000 GMT+0000",
        "Thu Jan 01 +1970 00:00:00 GMT+0000",
        "Thu Jan 01 01970 00:00:00 GMT+0000",
        "Sat Jan 01 -0000 00:00:00 GMT+0000",
        "Thu Jan 01 1970 00:00:00 GMT+2400",
        "Thu Jan 01 1970 00:00:00 GMT+0060",
        "Thu Jan 01 1970 00:00:00 GMT+0000 (UTC+00:00:00)",
        "Thu Jan 01 1970 00:00:00 GMT+0000 (UTC-00:00:01)",
        "Thu Jan 01 1970 00:00:00 GMT+0000 (UTC+000:00:01)",
        "Thu Jan 01 1970 00:00:00 GMT+0000 (UTC+00:60:01)",
        "Thu Jan 01 1970 00:00:00 GMT+0000 (UTC+00:00:60)",
        "Thu Jan 01 1970 00:00:00 GMT+0000 (UTC+999999:00:00)",
        "Thu Jan 01 1970 00:00:00 GMT+0000 (UTC+00:00:01) trailing",
        "Mon Apr 19 -271821 23:59:59 GMT+0000",
        "Sat Sep 13 275760 00:00:01 GMT+0000",
    ] {
        assert_eq!(
            parse_local_date_string(&JsString::from(text)),
            None,
            "{text}"
        );
    }
    // The suffix preserves both a whole day and a single historical second.
    assert_eq!(
        parse_local_date_string(&JsString::from(
            "Thu Jan 01 1970 00:00:00 GMT+0000 (UTC+24:00:00)"
        )),
        Some(-86400000)
    );
    assert_eq!(
        parse_local_date_string(&JsString::from(
            "Thu Jan 01 1970 00:00:00 GMT+0000 (UTC+00:00:01)"
        )),
        Some(-1000)
    );
}

#[test]
fn local_parser_rejects_non_ascii_code_units_at_every_output_position() {
    for text in [
        format_date_string(0, 0).unwrap(),
        format_date_string(0, i32::MIN).unwrap(),
    ] {
        for index in 0..text.len() {
            for unit in [0, 0xd800, 0xdc00, 0xffff, 0xff10] {
                let mut units = text.code_units().to_vec();
                units[index] = unit;
                assert_eq!(
                    parse_local_date_string(&JsString::from_code_units(units)),
                    None,
                    "{index} {unit}"
                );
            }
        }
    }
}
