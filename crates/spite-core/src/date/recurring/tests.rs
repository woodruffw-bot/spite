use super::*;
use crate::date::{
    DateTimeString, DateTimeZone, MAX_TIME_VALUE, TransitionDay, parse_posix_time_zone,
};

fn zone(text: &str) -> RecurringTimeZone {
    RecurringTimeZone::from_posix(parse_posix_time_zone(text).unwrap()).unwrap()
}

fn nominal(year: i32, month: u8, day: u8, hour: u8, minute: u8) -> i128 {
    DateTimeString {
        year,
        month: month - 1,
        day,
        hour,
        minute,
        second: 0,
        millisecond: 0,
        zone: DateTimeZone::Utc,
    }
    .nominal_epoch_milliseconds()
    .unwrap()
}

#[test]
fn northern_transitions_take_effect_at_the_exact_millisecond() {
    let zone = zone("EST5EDT,M3.2.0,M11.1.0");
    assert_eq!(zone.possible_offsets(), [-18_000, -14_400]);
    assert_eq!(zone.offset_at(0), -18_000);
    for (time, before, after) in [
        (1_710_054_000_000, -18_000, -14_400),
        (1_730_613_600_000, -14_400, -18_000),
    ] {
        assert_eq!(zone.offset_at(time - 1), before);
        assert_eq!(zone.offset_at(time), after);
        assert_eq!(zone.offset_at(time + 1), after);
    }
    assert_eq!(zone.offset_at(nominal(2024, 1, 1, 0, 0)), -18_000);
    assert_eq!(zone.offset_at(nominal(2024, 7, 1, 0, 0)), -14_400);
}

#[test]
fn southern_half_hour_and_negative_daylight_seasons_cross_new_year() {
    let southern = zone("<+1030>-10:30<+11>-11,M10.1.0,M4.1.0");
    assert_eq!(southern.possible_offsets(), [37_800, 39_600]);
    assert_eq!(southern.offset_at(nominal(2024, 1, 1, 0, 0)), 39_600);
    assert_eq!(southern.offset_at(nominal(2024, 7, 1, 0, 0)), 37_800);
    for (time, before, after) in [
        (nominal(2024, 4, 6, 15, 0), 39_600, 37_800),
        (nominal(2024, 10, 5, 15, 30), 37_800, 39_600),
    ] {
        assert_eq!(southern.offset_at(time - 1), before);
        assert_eq!(southern.offset_at(time), after);
    }
    let negative = zone("IST-1GMT0,M10.5.0,M3.5.0/1");
    assert_eq!(negative.possible_offsets(), [3600, 0]);
    assert_eq!(negative.offset_at(nominal(2024, 1, 1, 0, 0)), 0);
    assert_eq!(negative.offset_at(nominal(2024, 7, 1, 0, 0)), 3600);
    assert_eq!(negative.offset_at(nominal(2024, 3, 31, 1, 0) - 1), 0);
    assert_eq!(negative.offset_at(nominal(2024, 3, 31, 1, 0)), 3600);
}

#[test]
fn fixed_and_year_round_daylight_offsets_cover_all_native_inputs() {
    for (text, expected) in [
        ("UTC0", 0),
        ("AAA24:59:59", -89_999),
        ("AAA-24:59:59", 89_999),
        ("XXX3EDT4,0/0,J365/23", -14_400),
        ("AAA-24:59:59BBB,0/0,J365/25", 93_599),
        ("AAA0BBB-1,0/-167,365/167", 3600),
    ] {
        let zone = zone(text);
        for time in [
            i128::MIN,
            i128::MAX,
            i128::from(i64::MIN),
            i128::from(i64::MAX),
            -i128::from(MAX_TIME_VALUE),
            i128::from(MAX_TIME_VALUE),
            -1,
            0,
            1,
            nominal(2024, 1, 1, 0, 0),
        ] {
            assert_eq!(zone.offset_at(time), expected, "{text}, {time}");
        }
    }
}

#[test]
fn cycles_preserve_extreme_years_leap_centuries_and_transition_boundaries() {
    let parsed = parse_posix_time_zone("EST5EDT,M3.2.0,M11.1.0").unwrap();
    let rules = parsed.daylight.unwrap();
    let zone = RecurringTimeZone::from_posix(parsed).unwrap();
    for year in [
        i32::MIN,
        -271_821,
        -400,
        -100,
        -1,
        0,
        1900,
        2000,
        2100,
        275_760,
        i32::MAX,
    ] {
        let start = rules
            .start
            .epoch_milliseconds(year, -18_000, -18_000)
            .unwrap();
        let end = rules
            .end
            .epoch_milliseconds(year, -14_400, -18_000)
            .unwrap();
        assert_eq!(zone.offset_at(start - 1), -18_000, "year {year}");
        assert_eq!(zone.offset_at(start), -14_400, "year {year}");
        assert_eq!(zone.offset_at(end - 1), -14_400, "year {year}");
        assert_eq!(zone.offset_at(end), -18_000, "year {year}");
    }
    assert_eq!(zone.offset_at(-i128::from(MAX_TIME_VALUE)), -14_400);
    assert_eq!(zone.offset_at(i128::from(MAX_TIME_VALUE)), -14_400);
}

#[test]
fn cycles_keep_neighboring_offsets_through_years_without_transitions() {
    // Four and five Sunday occurrences coincide in most Februaries. This rule
    // has a daylight week only when February has five Sundays: an event search
    // limited to nearby years would fail across the longer gaps between them.
    let zone = zone("AAA0BBB-1,M2.4.0/2,M2.5.0/3");
    for (year, expected) in [(2004, 3600), (2024, 0), (2026, 0), (2032, 3600), (2100, 0)] {
        assert_eq!(
            zone.offset_at(nominal(year, 2, 28, 12, 0)),
            expected,
            "{year}"
        );
        assert_eq!(zone.offset_at(nominal(year, 3, 1, 0, 0)), 0, "{year}");
    }
}

#[test]
fn julian_leap_conventions_and_cross_year_times_keep_exact_offsets() {
    let zone = zone("AAA0BBB-1,J60/-1,J365/25");
    for year in [-400, 0, 1900, 2000, 2023, 2024, 2100] {
        let transition = nominal(year, 2, 28, 23, 0)
            + if year % 400 == 0 || (year % 4 == 0 && year % 100 != 0) {
                i128::from(MS_PER_DAY)
            } else {
                0
            };
        assert_eq!(zone.offset_at(transition - 1), 0);
        assert_eq!(zone.offset_at(transition), 3600);
        assert_eq!(zone.offset_at(nominal(year, 12, 31, 23, 59)), 3600);
        assert_eq!(zone.offset_at(nominal(year + 1, 1, 1, 0, 0)), 0);
    }
}

#[test]
fn invalid_native_records_and_conflicting_cycles_are_distinct_errors() {
    let mut parsed = parse_posix_time_zone("AAA0BBB-1,M3.2.0,M11.1.0").unwrap();
    parsed.daylight.as_mut().unwrap().start.day = TransitionDay::JulianWithoutLeap(0);
    assert_eq!(
        RecurringTimeZone::from_posix(parsed),
        Err(RecurringTimeZoneError::InvalidRule)
    );
    let mut parsed = parse_posix_time_zone("AAA0BBB-1,M3.2.0,M11.1.0").unwrap();
    parsed.daylight.as_mut().unwrap().end.seconds = i32::MAX;
    assert_eq!(
        RecurringTimeZone::from_posix(parsed),
        Err(RecurringTimeZoneError::InvalidRule)
    );
    let conflicting = parse_posix_time_zone("AAA0BBB-1,365/0,0/1").unwrap();
    assert_eq!(
        RecurringTimeZone::from_posix(conflicting),
        Err(RecurringTimeZoneError::ConflictingTransitions)
    );
}
