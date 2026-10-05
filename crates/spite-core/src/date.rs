//! UTC calendar decomposition and Date time-value arithmetic (21.4.1).
//!
//! These operations use the proleptic Gregorian calendar, an epoch at the start
//! of 1970, and exactly 86,400 seconds per day. Interchange string syntax records
//! local or explicit zones; zone resolution, MakeDay and JavaScript intrinsics
//! are separate implementation steps.

mod parse;
pub use parse::{DateTimeString, DateTimeZone, parse_date_time_string};
mod utc_string;
pub use utc_string::{format_utc_date_string, parse_utc_date_string};

use crate::JsString;
use std::fmt::Write;

/// Milliseconds in an ECMAScript day; leap seconds are not represented.
pub const MS_PER_DAY: i64 = 86_400_000;
/// The inclusive absolute time-value boundary prescribed by TimeClip.
pub const MAX_TIME_VALUE: i64 = 8_640_000_000_000_000;

/// Applies TimeClip, returning NaN for non-finite or out-of-range inputs.
///
/// Finite values truncate toward zero and both zero signs become positive zero.
/// This is the Date domain prescribed by
/// [TimeClip](https://262.ecma-international.org/17.0/#sec-timeclip).
pub fn time_clip(time: f64) -> f64 {
    if !time.is_finite() || time.abs() > MAX_TIME_VALUE as f64 {
        return f64::NAN;
    }
    integer(time)
}

fn integer(value: f64) -> f64 {
    let value = value.trunc();
    if value == 0.0 { 0.0 } else { value }
}

/// Applies MakeFullYear, interpreting integer years 0 through 99 as 1900 through 1999.
///
/// NaN remains NaN; other inputs truncate toward zero, including signed zero.
/// Infinities remain infinite. Interchange string years do not use this operation.
/// See [MakeFullYear](https://262.ecma-international.org/17.0/#sec-makefullyear).
pub fn make_full_year(year: f64) -> f64 {
    let year = integer(year);
    if (0.0..=99.0).contains(&year) {
        1900.0 + year
    } else {
        year
    }
}

/// Applies MakeTime to numeric components, preserving ordered Number arithmetic.
///
/// Non-finite inputs yield NaN. Finite inputs are truncated individually before
/// computing hours, minutes, seconds, and milliseconds in that order
/// ([MakeTime](https://262.ecma-international.org/17.0/#sec-maketime)).
pub fn make_time(hour: f64, minute: f64, second: f64, millisecond: f64) -> f64 {
    if [hour, minute, second, millisecond]
        .into_iter()
        .any(|value| !value.is_finite())
    {
        return f64::NAN;
    }
    ((integer(hour) * 3_600_000.0 + integer(minute) * 60_000.0) + integer(second) * 1_000.0)
        + integer(millisecond)
}

/// Applies MakeDate, returning NaN when inputs or the ordered result are non-finite.
///
/// Finite results can exceed TimeClip's domain; clipping is a separate step
/// ([MakeDate](https://262.ecma-international.org/17.0/#sec-makedate)).
pub fn make_date(day: f64, time: f64) -> f64 {
    if !day.is_finite() || !time.is_finite() {
        return f64::NAN;
    }
    let time = day * MS_PER_DAY as f64 + time;
    if time.is_finite() { time } else { f64::NAN }
}

/// Formats a clipped integral time value as the canonical UTC interchange string.
///
/// Returns `None` outside the clipped domain. Years 0–9999 use four digits;
/// other years use a sign and six digits. Output always includes seconds, three
/// millisecond digits, and Z, matching the finite branch of
/// [Date.prototype.toISOString](https://262.ecma-international.org/17.0/#sec-date.prototype.toisostring).
pub fn format_iso_date_time(time: i64) -> Option<JsString> {
    let date = UtcDateTime::from_time_value(time)?;
    let mut text = String::with_capacity(27);
    if (0..=9999).contains(&date.year) {
        write!(text, "{:04}", date.year).expect("writing a numeric year to String is infallible");
    } else {
        let sign = if date.year < 0 { '-' } else { '+' };
        write!(text, "{sign}{:06}", date.year.unsigned_abs())
            .expect("writing a numeric year to String is infallible");
    }
    write!(
        text,
        "-{:02}-{:02}T{:02}:{:02}:{:02}.{:03}Z",
        date.month + 1,
        date.day,
        date.hour,
        date.minute,
        date.second,
        date.millisecond,
    )
    .expect("writing numeric date fields to String is infallible");
    Some(JsString::from(text.as_str()))
}

/// UTC calendar fields for one finite, integral, clipped time value.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UtcDateTime {
    /// Proleptic Gregorian year, including zero and negative years.
    pub year: i32,
    /// Zero-based month, from January (0) through December (11).
    pub month: u8,
    /// One-based day of the month, from 1 through 31.
    pub day: u8,
    /// Day of the week, from Sunday (0) through Saturday (6).
    pub weekday: u8,
    /// Hour of the UTC day, from 0 through 23.
    pub hour: u8,
    /// Minute of the hour, from 0 through 59.
    pub minute: u8,
    /// Second of the minute, from 0 through 59.
    pub second: u8,
    /// Millisecond of the second, from 0 through 999.
    pub millisecond: u16,
}

impl UtcDateTime {
    /// Decomposes a clipped integral time value; rejects values outside its domain.
    ///
    /// Euclidean division handles instants before the epoch. Year lookup follows
    /// YearFromTime using a fixed calendar interval containing the complete
    /// TimeClip domain, without scanning years or allocating
    /// ([YearFromTime](https://262.ecma-international.org/17.0/#sec-yearfromtime)).
    pub fn from_time_value(time: i64) -> Option<Self> {
        if !(-MAX_TIME_VALUE..=MAX_TIME_VALUE).contains(&time) {
            return None;
        }
        let day = time.div_euclid(MS_PER_DAY);
        // The clipped endpoints are -271821-04-20 and +275760-09-13.
        // Bracket their years by adjacent January boundaries.
        let mut lower = -271_822;
        let mut upper = 275_761;
        while lower + 1 < upper {
            let middle = lower + (upper - lower) / 2;
            if day_from_year(middle) <= day {
                lower = middle;
            } else {
                upper = middle;
            }
        }
        let mut within_year = day - day_from_year(lower);
        let mut month = 0;
        for length in month_lengths(lower) {
            if within_year < i64::from(length) {
                break;
            }
            within_year -= i64::from(length);
            month += 1;
        }
        let within_day = time.rem_euclid(MS_PER_DAY);
        Some(Self {
            year: lower,
            month,
            day: (within_year + 1) as u8,
            weekday: (day + 4).rem_euclid(7) as u8,
            hour: (within_day / 3_600_000) as u8,
            minute: (within_day / 60_000 % 60) as u8,
            second: (within_day / 1_000 % 60) as u8,
            millisecond: (within_day % 1_000) as u16,
        })
    }
}

fn day_from_year(year: i32) -> i64 {
    // Use floor, including for negative years (21.4.1.6).
    let year = i64::from(year);
    365 * (year - 1970) + (year - 1969).div_euclid(4) - (year - 1901).div_euclid(100)
        + (year - 1601).div_euclid(400)
}

fn month_lengths(year: i32) -> [u8; 12] {
    // DaysInYear: divisible by 400, or by 4 but not by 100 (21.4.1.5).
    let leap = year % 400 == 0 || (year % 4 == 0 && year % 100 != 0);
    [
        31,
        28 + u8::from(leap),
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
}

#[cfg(test)]
mod tests;
