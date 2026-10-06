use super::*;
use std::fmt::Write;

#[test]
fn posix_zone_syntax_and_normalized_rules_snapshot() {
    let mut report = String::new();
    for text in [
        "UTC0",
        "GMT+0",
        "EST5EDT,M3.2.0,M11.1.0",
        "CET-1CEST,M3.5.0,M10.5.0/3",
        "<+1030>-10:30<+11>-11,M10.1.0,M4.1.0",
        "IST-1GMT0,M10.5.0,M3.5.0/1",
        "XXX3EDT4,0/0,J365/23",
        "AAA-24:59:59BBB,J60/-167:59:59,365/+167:59:59",
        "AAA24:59:59BBB-24:59:59,J1/24:00:01,J365/0",
        "AAA-00:00:01BBB+00:00:01,M01.1.0/000,M12.5.6/001",
    ] {
        writeln!(
            report,
            "{text}\n{:#?}",
            parse_posix_time_zone(text).unwrap()
        )
        .unwrap();
    }
    insta::assert_snapshot!(report);
}

#[test]
fn offset_signs_defaults_and_extended_times_are_exact() {
    for (text, standard, daylight) in [
        ("AAA5BBB,M3.2.0,M11.1.0", -18_000, -14_400),
        ("AAA-24:59:59BBB,0,365", 89_999, 93_599),
        ("AAA24:59:59BBB,0,365", -89_999, -86_399),
        ("AAA+00:00:01BBB-00:00:01,0,365", -1, 1),
        ("AAA-00:00:01BBB+00:00:01,0,365", 1, -1),
    ] {
        let zone = parse_posix_time_zone(text).unwrap();
        assert_eq!(zone.standard_offset, standard);
        let adjustment = zone.daylight.unwrap();
        assert_eq!(adjustment.offset, daylight);
        assert_eq!(adjustment.start.seconds, 7200);
        assert_eq!(adjustment.end.seconds, 7200);
        assert_eq!(adjustment.start.clock, TransitionClock::Wall);
    }
    let zone = parse_posix_time_zone("AAA0BBB,J1/-167:59:59,J365/167:59:59").unwrap();
    let adjustment = zone.daylight.unwrap();
    assert_eq!(adjustment.start.seconds, -604_799);
    assert_eq!(adjustment.end.seconds, 604_799);
    assert_eq!(
        adjustment.start.epoch_milliseconds(2024, 0, 0),
        Some(1_703_462_401_000)
    );
    assert_eq!(
        adjustment.end.epoch_milliseconds(2024, 3600, 0),
        Some(1_736_204_399_000)
    );
}

#[test]
fn missing_rules_malformed_fields_and_trailing_input_are_rejected() {
    for text in [
        "",
        "UTC",
        "UT0",
        "<>0",
        "<AB>0",
        "<ABC0",
        "ABC>0",
        "<A_B>0",
        "<A:B>0",
        "<A B>0",
        "<A/B>0",
        "<A.B>0",
        "UTC25",
        "UTC-25",
        "UTC+25",
        "UTC1:60",
        "UTC1:00:60",
        "UTC1:",
        "UTC1:0",
        "UTC1:00:",
        "UTC1:00:0",
        "UTC000",
        "UTC--1",
        "UTC+-1",
        "UTC1:000",
        "UTC0DST",
        "UTC0DST1",
        "UTC0DST,",
        "UTC0DST,0",
        "UTC0DST,0,",
        "UTC0DST,0,365,0",
        "UTC0DST;0,365",
        "UTC0DST,J0,J365",
        "UTC0DST,J366,J365",
        "UTC0DST,366,365",
        "UTC0DST,-1,365",
        "UTC0DST,0000,365",
        "UTC0DST,M0.1.0,M1.1.0",
        "UTC0DST,M13.1.0,M1.1.0",
        "UTC0DST,M01.0.0,M1.1.0",
        "UTC0DST,M1.6.0,M1.1.0",
        "UTC0DST,M1.1.7,M1.1.0",
        "UTC0DST,M1.01.0,M1.1.0",
        "UTC0DST,M1.1.00,M1.1.0",
        "UTC0DST,M001.1.0,M1.1.0",
        "UTC0DST,M1,365",
        "UTC0DST,J1/,J365",
        "UTC0DST,J1/168,J365",
        "UTC0DST,J1/-168,J365",
        "UTC0DST,J1/167:60,J365",
        "UTC0DST,J1/167:00:60,J365",
        "UTC0DST,J1/0000,J365",
        "UTC0DST,J1/2s,J365",
        "UTC0DST,J1/2u,J365",
        "UTC0DST25,J1,J365",
        "UTC0DST-25,J1,J365",
        "UTC0DST1:0,J1,J365",
        " UTC0",
        "UTC0 ",
        "UTC0\n",
        "UTC0\0",
        "UTC0DST,J1,J365extra",
    ] {
        assert_eq!(parse_posix_time_zone(text), None, "{text:?}");
    }
}

#[test]
fn truncated_or_non_ascii_inputs_do_not_panic_or_parse_as_complete_rules() {
    let text = "<+1030>-10:30<+11>-11,M10.1.0/2:30:45,M4.1.0/3:00:01";
    let complete_prefixes = [
        "<+1030>-1",
        "<+1030>-10",
        "<+1030>-10:30",
        "<+1030>-10:30<+11>-11,M10.1.0/2:30:45,M4.1.0",
        "<+1030>-10:30<+11>-11,M10.1.0/2:30:45,M4.1.0/3",
        "<+1030>-10:30<+11>-11,M10.1.0/2:30:45,M4.1.0/3:00",
    ];
    for length in 0..text.len() {
        // A truncation ending after the standard offset is a complete fixed zone.
        let prefix = &text[..length];
        assert_eq!(
            parse_posix_time_zone(prefix).is_some(),
            complete_prefixes.contains(&prefix),
            "{prefix:?}"
        );
    }
    for index in 0..text.len() {
        for replacement in ["\0", "é", "😀", "\u{feff}", "\u{ff10}", "\n"] {
            let altered = format!("{}{replacement}{}", &text[..index], &text[index + 1..]);
            assert_eq!(parse_posix_time_zone(&altered), None, "{altered:?}");
        }
    }
}

#[test]
fn designations_are_borrowed_without_an_arbitrary_name_length_limit() {
    let name = "Long".repeat(4096);
    let text = format!("<{name}>0{name},J1,J365");
    let zone = parse_posix_time_zone(&text).unwrap();
    assert_eq!(zone.standard_name, name);
    assert_eq!(zone.daylight.unwrap().name, name);
    assert_eq!(zone.standard_name.as_ptr(), text[1..].as_ptr());
    assert_eq!(
        zone.daylight.unwrap().name.as_ptr(),
        text[name.len() + 3..].as_ptr()
    );
}
