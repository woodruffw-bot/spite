//! Standard UTC Date strings and their required Date.parse round trip.

use super::{DateTimeString, DateTimeZone, UtcDateTime};
use crate::JsString;
use std::fmt::Write;

pub(super) const WEEKDAYS: [&str; 7] = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
pub(super) const MONTHS: [&str; 12] = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];

/// Formats a clipped integral time value using Date.prototype.toUTCString.
///
/// Years use a minimum of four digits, with a minus sign only for negative
/// years. Milliseconds are omitted. Returns `None` outside the TimeClip domain.
/// See [toUTCString](https://262.ecma-international.org/17.0/#sec-date.prototype.toutcstring).
pub fn format_utc_date_string(time: i64) -> Option<JsString> {
    let date = UtcDateTime::from_time_value(time)?;
    let mut text = String::with_capacity(32);
    let sign = if date.year < 0 { "-" } else { "" };
    write!(
        text,
        "{}, {:02} {} {sign}{:04} {:02}:{:02}:{:02} GMT",
        WEEKDAYS[usize::from(date.weekday)],
        date.day,
        MONTHS[usize::from(date.month)],
        date.year.unsigned_abs(),
        date.hour,
        date.minute,
        date.second,
    )
    .expect("writing numeric date fields to String is infallible");
    Some(JsString::from(text.as_str()))
}

/// Parses the standard UTC string produced by [`format_utc_date_string`].
///
/// This provides the whole-second round trip required by
/// [Date.parse](https://262.ecma-international.org/17.0/#sec-date.parse).
/// It borrows UTF-16 without allocating, validates the calendar and weekday,
/// and treats years 0–99 literally. Other legacy formats are not recognized.
pub fn parse_utc_date_string(source: &JsString) -> Option<i64> {
    let units = source.code_units();
    // Every clipped year has at most six digits. These bounds describe the
    // standard output (29–32 ASCII code units), not a host resource quota.
    if !(29..=32).contains(&units.len())
        || !ascii(&units[3..5], ", ")
        || units[7] != u16::from(b' ')
        || units[11] != u16::from(b' ')
    {
        return None;
    }
    let weekday = name(&units[..3], &WEEKDAYS)?;
    let day = digits(&units[5..7])? as u8;
    let month = name(&units[8..11], &MONTHS)?;
    let (year, time) = units[12..].split_at(units.len() - 25);
    let negative = year.first() == Some(&u16::from(b'-'));
    let magnitude = if negative { &year[1..] } else { year };
    if !(4..=6).contains(&magnitude.len())
        || (magnitude.len() > 4 && magnitude[0] == u16::from(b'0'))
    {
        return None;
    }
    let year = digits(magnitude)?;
    if negative && year == 0 {
        return None;
    }
    if time[0] != u16::from(b' ')
        || time[3] != u16::from(b':')
        || time[6] != u16::from(b':')
        || !ascii(&time[9..], " GMT")
    {
        return None;
    }
    let fields = DateTimeString {
        year: if negative { -year } else { year },
        month,
        day,
        hour: digits(&time[1..3])? as u8,
        minute: digits(&time[4..6])? as u8,
        second: digits(&time[7..9])? as u8,
        millisecond: 0,
        zone: DateTimeZone::Utc,
    };
    let time = fields.utc_time_value()?;
    if !time.is_finite() {
        return None;
    }
    let time = time as i64;
    // UTC strings name an actual calendar day, unlike interchange strings
    // whose DD element permits normalization through the following month.
    let expected = UtcDateTime {
        year: fields.year,
        month,
        day,
        weekday,
        hour: fields.hour,
        minute: fields.minute,
        second: fields.second,
        millisecond: 0,
    };
    (UtcDateTime::from_time_value(time)? == expected).then_some(time)
}

pub(super) fn ascii(units: &[u16], expected: &str) -> bool {
    units.len() == expected.len()
        && units
            .iter()
            .zip(expected.bytes())
            .all(|(&unit, byte)| unit == u16::from(byte))
}

pub(super) fn name(units: &[u16], names: &[&str]) -> Option<u8> {
    names
        .iter()
        .position(|name| ascii(units, name))
        .map(|n| n as u8)
}

fn digits(units: &[u16]) -> Option<i32> {
    units.iter().try_fold(0, |value, &unit| {
        (u16::from(b'0')..=u16::from(b'9'))
            .contains(&unit)
            .then(|| value * 10 + i32::from(unit - u16::from(b'0')))
    })
}
