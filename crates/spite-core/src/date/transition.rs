//! Exact Gregorian transition calendars for recurring political time-zone rules.

use super::{MS_PER_DAY, day_from_year, month_lengths};

/// A recurring transition date in the POSIX rule notation used by TZif files.
///
/// See [TZif's TZ string](https://www.rfc-editor.org/rfc/rfc9636.html#section-3.3.1).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TransitionDay {
    /// `Jn`: one-based day 1–365, omitting February 29 from the count.
    JulianWithoutLeap(u16),
    /// `n`: zero-based day 0–365, including February 29 in the count.
    ///
    /// Day 365 in a common year reaches January 1 of the following year.
    JulianWithLeap(u16),
    /// `Mm.w.d`: the specified weekday in week 1–5 of month 1–12.
    ///
    /// Weekdays are Sunday (0) through Saturday (6). Week 5 selects the
    /// last occurrence in the month, even when there are only four.
    MonthWeekday {
        /// One-based month, from January (1) through December (12).
        month: u8,
        /// Occurrence 1–4, or 5 for the last occurrence in the month.
        week: u8,
        /// Sunday (0) through Saturday (6).
        weekday: u8,
    },
}

/// The clock used to interpret a time-zone transition's seconds from midnight.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TransitionClock {
    /// Local wall time, using the offset immediately before the transition.
    Wall,
    /// Local standard time, ignoring any daylight-saving adjustment.
    Standard,
    /// UTC, without a local offset adjustment.
    Utc,
}

/// A recurring Gregorian time-zone transition before selecting an offset.
///
/// These calendars support political rules for
/// [GetNamedTimeZoneOffsetNanoseconds](https://262.ecma-international.org/17.0/#sec-getnamedtimezoneoffsetnanoseconds).
/// Loading zone data and resolving complete offset histories are separate steps.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TransitionRule {
    /// The recurring calendar date.
    pub day: TransitionDay,
    /// Seconds from that date's midnight, in the TZif range −604799…604799.
    ///
    /// Negative times and times beyond one day can cross month/year boundaries.
    pub seconds: i32,
    /// The clock in which `seconds` is expressed.
    pub clock: TransitionClock,
}

impl TransitionRule {
    /// Computes exact transition epoch milliseconds for a native Gregorian year.
    ///
    /// Offsets are seconds east of UTC. `wall_offset` is the offset before this
    /// transition; `standard_offset` excludes daylight-saving adjustments. All
    /// native `i32` years and offsets are representable in the `i128` result.
    /// Invalid rule fields return `None`; transitions are never clamped to the
    /// given year or clipped to Date's range. No leap seconds are represented.
    pub fn epoch_milliseconds(
        self,
        year: i32,
        wall_offset: i32,
        standard_offset: i32,
    ) -> Option<i128> {
        if !(-604_799..=604_799).contains(&self.seconds) {
            return None;
        }
        let offset = match self.clock {
            TransitionClock::Wall => wall_offset,
            TransitionClock::Standard => standard_offset,
            TransitionClock::Utc => 0,
        };
        let day = i128::from(day_from_year(year)) + i128::from(self.day.day_of_year(year)?);
        Some(day * i128::from(MS_PER_DAY) + (i128::from(self.seconds) - i128::from(offset)) * 1000)
    }
}

impl TransitionDay {
    fn day_of_year(self, year: i32) -> Option<u16> {
        let lengths = month_lengths(year);
        match self {
            Self::JulianWithoutLeap(day) => {
                if !(1..=365).contains(&day) {
                    return None;
                }
                Some(day - 1 + u16::from(lengths[1] == 29 && day >= 60))
            }
            Self::JulianWithLeap(day) => (day <= 365).then_some(day),
            Self::MonthWeekday {
                month,
                week,
                weekday,
            } => {
                if !(1..=12).contains(&month) || !(1..=5).contains(&week) || weekday > 6 {
                    return None;
                }
                let month_index = usize::from(month - 1);
                let first_day: u16 = lengths[..month_index].iter().map(|&n| u16::from(n)).sum();
                let first_weekday = (day_from_year(year) + i64::from(first_day) + 4).rem_euclid(7);
                let first_match = (i64::from(weekday) - first_weekday).rem_euclid(7) as u16;
                let mut day = first_match + 7 * u16::from(week - 1);
                if day >= u16::from(lengths[month_index]) {
                    day -= 7;
                }
                Some(first_day + day)
            }
        }
    }
}

#[cfg(test)]
mod tests;
