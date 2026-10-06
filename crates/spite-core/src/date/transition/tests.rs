use super::*;
use crate::date::{DateTimeString, DateTimeZone, MAX_TIME_VALUE, UtcDateTime};

fn nominal(year: i32, month: u8, day: u8, hour: u8, minute: u8, second: u8) -> i128 {
    DateTimeString {
        year,
        month: month - 1,
        day,
        hour,
        minute,
        second,
        millisecond: 0,
        zone: DateTimeZone::Utc,
    }
    .nominal_epoch_milliseconds()
    .unwrap()
}

fn rule(day: TransitionDay) -> TransitionRule {
    TransitionRule {
        day,
        seconds: 7200,
        clock: TransitionClock::Wall,
    }
}

#[test]
fn transition_clocks_use_the_offset_before_the_change() {
    let start = rule(TransitionDay::MonthWeekday {
        month: 3,
        week: 2,
        weekday: 0,
    });
    let end = rule(TransitionDay::MonthWeekday {
        month: 11,
        week: 1,
        weekday: 0,
    });
    // New York's 2024 transitions: wall 02:00, with different prior offsets.
    assert_eq!(
        start.epoch_milliseconds(2024, -18_000, -18_000),
        Some(1_710_054_000_000)
    );
    assert_eq!(
        end.epoch_milliseconds(2024, -14_400, -18_000),
        Some(1_730_613_600_000)
    );
    assert_eq!(
        TransitionRule {
            clock: TransitionClock::Standard,
            ..end
        }
        .epoch_milliseconds(2024, -14_400, -18_000),
        Some(nominal(2024, 11, 3, 7, 0, 0))
    );
    assert_eq!(
        TransitionRule {
            clock: TransitionClock::Utc,
            ..end
        }
        .epoch_milliseconds(2024, -14_400, -18_000),
        Some(nominal(2024, 11, 3, 2, 0, 0))
    );
    // Lord Howe's half-hour adjustment crosses the southern-hemisphere season.
    let southern_start = rule(TransitionDay::MonthWeekday {
        month: 10,
        week: 1,
        weekday: 0,
    });
    let southern_end = rule(TransitionDay::MonthWeekday {
        month: 4,
        week: 1,
        weekday: 0,
    });
    assert_eq!(
        southern_start.epoch_milliseconds(2024, 37_800, 37_800),
        Some(nominal(2024, 10, 5, 15, 30, 0))
    );
    assert_eq!(
        southern_end.epoch_milliseconds(2024, 39_600, 37_800),
        Some(nominal(2024, 4, 6, 15, 0, 0))
    );
    // The calendar conversion does not assume that daylight offsets increase.
    assert_eq!(
        end.epoch_milliseconds(2024, 0, 3600),
        Some(nominal(2024, 11, 3, 2, 0, 0))
    );
}

#[test]
fn julian_rules_distinguish_leap_days_and_roll_common_year_day_365() {
    for year in [
        i32::MIN,
        -400,
        -100,
        -4,
        -1,
        0,
        1,
        4,
        100,
        400,
        1900,
        2000,
        2023,
        2024,
        i32::MAX,
    ] {
        let leap = year % 400 == 0 || (year % 4 == 0 && year % 100 != 0);
        let noon = |day| TransitionRule {
            seconds: 43_200,
            clock: TransitionClock::Utc,
            ..rule(day)
        };
        assert_eq!(
            noon(TransitionDay::JulianWithoutLeap(59)).epoch_milliseconds(year, 0, 0),
            Some(nominal(year, 2, 28, 12, 0, 0))
        );
        assert_eq!(
            noon(TransitionDay::JulianWithoutLeap(60)).epoch_milliseconds(year, 0, 0),
            Some(nominal(year, 3, 1, 12, 0, 0))
        );
        assert_eq!(
            noon(TransitionDay::JulianWithLeap(59)).epoch_milliseconds(year, 0, 0),
            Some(nominal(
                year,
                if leap { 2 } else { 3 },
                if leap { 29 } else { 1 },
                12,
                0,
                0
            ))
        );
        assert_eq!(
            noon(TransitionDay::JulianWithoutLeap(365)).epoch_milliseconds(year, 0, 0),
            Some(nominal(year, 12, 31, 12, 0, 0))
        );
        assert_eq!(
            noon(TransitionDay::JulianWithLeap(365)).epoch_milliseconds(year, 0, 0),
            Some(nominal(year, 12, 31, 12, 0, 0) + if leap { 0 } else { i128::from(MS_PER_DAY) })
        );
    }
}

#[test]
fn fifth_week_selects_the_last_weekday_in_four_and_five_week_months() {
    for (year, month, weekday, expected) in [
        (2024, 2, 0, 25),
        (2024, 3, 0, 31),
        (2023, 2, 2, 28),
        (1900, 2, 3, 28),
        (2000, 2, 2, 29),
        (0, 2, 2, 29),
        (-1, 2, 0, 28),
    ] {
        let transition = rule(TransitionDay::MonthWeekday {
            month,
            week: 5,
            weekday,
        });
        assert_eq!(
            transition.epoch_milliseconds(year, 0, 0),
            Some(nominal(year, month, expected, 2, 0, 0))
        );
    }
}

#[test]
fn signed_transition_times_cross_years_without_clamping_or_clipping() {
    let january = TransitionRule {
        seconds: -604_799,
        clock: TransitionClock::Wall,
        ..rule(TransitionDay::JulianWithLeap(0))
    };
    let december = TransitionRule {
        seconds: 604_799,
        ..january
    };
    assert_eq!(
        january.epoch_milliseconds(2024, 3600, 0),
        Some(nominal(2023, 12, 24, 23, 0, 1))
    );
    assert_eq!(
        TransitionRule {
            day: TransitionDay::JulianWithoutLeap(365),
            ..december
        }
        .epoch_milliseconds(2023, -3600, 0),
        Some(nominal(2024, 1, 7, 0, 59, 59))
    );
    assert_eq!(
        TransitionRule {
            seconds: 86_400,
            ..december
        }
        .epoch_milliseconds(0, 0, 0),
        Some(nominal(0, 1, 2, 0, 0, 0))
    );
    let start = nominal(i32::MIN, 1, 1, 0, 0, 0);
    assert_eq!(
        january.epoch_milliseconds(i32::MIN, i32::MAX, 0),
        Some(start + (-604_799_i128 - i128::from(i32::MAX)) * 1000)
    );
    assert_eq!(
        december.epoch_milliseconds(i32::MAX, i32::MIN, 0),
        Some(nominal(i32::MAX, 1, 1, 0, 0, 0) + (604_799_i128 - i128::from(i32::MIN)) * 1000)
    );
    for time in [i64::MIN, -MAX_TIME_VALUE, MAX_TIME_VALUE, i64::MAX] {
        let year = UtcDateTime::from_epoch_milliseconds(time).year;
        let transition = january.epoch_milliseconds(year, 0, 0).unwrap();
        assert_eq!(transition, nominal(year, 1, 1, 0, 0, 0) - 604_799_000);
    }
}

#[test]
fn malformed_transition_fields_are_rejected_without_indexing_panics() {
    for day in [
        TransitionDay::JulianWithoutLeap(0),
        TransitionDay::JulianWithoutLeap(366),
        TransitionDay::JulianWithoutLeap(u16::MAX),
        TransitionDay::JulianWithLeap(366),
        TransitionDay::JulianWithLeap(u16::MAX),
        TransitionDay::MonthWeekday {
            month: 0,
            week: 1,
            weekday: 0,
        },
        TransitionDay::MonthWeekday {
            month: 13,
            week: 1,
            weekday: 0,
        },
        TransitionDay::MonthWeekday {
            month: 255,
            week: 1,
            weekday: 0,
        },
        TransitionDay::MonthWeekday {
            month: 1,
            week: 0,
            weekday: 0,
        },
        TransitionDay::MonthWeekday {
            month: 1,
            week: 6,
            weekday: 0,
        },
        TransitionDay::MonthWeekday {
            month: 1,
            week: 255,
            weekday: 0,
        },
        TransitionDay::MonthWeekday {
            month: 1,
            week: 1,
            weekday: 7,
        },
        TransitionDay::MonthWeekday {
            month: 1,
            week: 1,
            weekday: 255,
        },
    ] {
        assert_eq!(rule(day).epoch_milliseconds(2024, 0, 0), None);
    }
    for seconds in [i32::MIN, -604_800, 604_800, i32::MAX] {
        assert_eq!(
            TransitionRule {
                seconds,
                ..rule(TransitionDay::JulianWithLeap(0))
            }
            .epoch_milliseconds(2024, 0, 0),
            None
        );
    }
}
