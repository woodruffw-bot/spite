//! Local Date strings and exact parsing of our own whole-second output.

use super::utc_string::{MONTHS, WEEKDAYS, ascii, name};
use super::{DateTimeString, DateTimeZone, MAX_TIME_VALUE, UtcDateTime};
use crate::JsString;
use std::fmt::Write;

#[cfg(test)]
mod tests;

/// Applies DateString to an unclipped local calendar millisecond value.
///
/// Years have at least four digits, with a minus sign only for negative years.
/// See [DateString](https://262.ecma-international.org/17.0/#sec-datestring).
pub fn format_local_date_string(local: i64) -> JsString {
    let mut text = String::with_capacity(22);
    write_date(&mut text, UtcDateTime::from_epoch_milliseconds(local));
    JsString::from(text.as_str())
}

/// Applies TimeString and TimeZoneString for a local value and exact UTC offset.
///
/// The GMT hours/minutes follow the specification, including their 24-hour
/// wrap. Historical seconds or offsets of a day or more are preserved in an
/// implementation-defined timezone name, `(UTC+HH:MM:SS)` or its negative form.
/// Other offsets use the empty timezone name. See
/// [TimeZoneString](https://262.ecma-international.org/17.0/#sec-timezoneestring).
pub fn format_local_time_string(local: i64, offset: i32) -> JsString {
    let mut text = String::with_capacity(36);
    write_time(
        &mut text,
        UtcDateTime::from_epoch_milliseconds(local),
        offset,
    );
    JsString::from(text.as_str())
}

/// Formats a clipped Date time value with its exact UTC offset.
///
/// Implements the finite branch of
/// [ToDateString](https://262.ecma-international.org/17.0/#sec-todatestring).
/// Local calendar fields are never clipped; milliseconds are omitted.
pub fn format_date_string(time: i64, offset: i32) -> Option<JsString> {
    if !(-MAX_TIME_VALUE..=MAX_TIME_VALUE).contains(&time) {
        return None;
    }
    let local = time + i64::from(offset) * 1000;
    let date = UtcDateTime::from_epoch_milliseconds(local);
    let mut text = String::with_capacity(55);
    write_date(&mut text, date);
    text.push(' ');
    write_time(&mut text, date, offset);
    Some(JsString::from(text.as_str()))
}

fn write_date(text: &mut String, date: UtcDateTime) {
    let sign = if date.year < 0 { "-" } else { "" };
    write!(
        text,
        "{} {} {:02} {sign}{:04}",
        WEEKDAYS[usize::from(date.weekday)],
        MONTHS[usize::from(date.month)],
        date.day,
        date.year.unsigned_abs()
    )
    .expect("writing date fields to String is infallible");
}

fn write_time(text: &mut String, date: UtcDateTime, offset: i32) {
    let sign = if offset < 0 { '-' } else { '+' };
    let magnitude = offset.unsigned_abs();
    write!(
        text,
        "{:02}:{:02}:{:02} GMT{sign}{:02}{:02}",
        date.hour,
        date.minute,
        date.second,
        (magnitude / 3600) % 24,
        (magnitude / 60) % 60
    )
    .expect("writing time fields to String is infallible");
    if needs_exact_name(offset) {
        write!(
            text,
            " (UTC{sign}{:02}:{:02}:{:02})",
            magnitude / 3600,
            (magnitude / 60) % 60,
            magnitude % 60
        )
        .expect("writing an offset to String is infallible");
    }
}

fn needs_exact_name(offset: i32) -> bool {
    offset % 60 != 0 || offset.unsigned_abs() >= 86_400
}

/// Parses the canonical whole-second output of [`format_date_string`].
///
/// Borrows UTF-16, validates calendar fields/weekday and treats short years
/// literally. The exact offset in our optional timezone name preserves the
/// [Date.parse](https://262.ecma-international.org/17.0/#sec-date.parse) round trip
/// for historical seconds and every native offset, independently of host data.
/// Other legacy formats and timezone names are not recognized.
pub fn parse_local_date_string(source: &JsString) -> Option<i64> {
    let units = source.code_units();
    // These lengths bound our clipped Date output for every native i32 offset,
    // not input quotas. Local years still have at most six magnitude digits.
    if !(33..=55).contains(&units.len())
        || units[3] != u16::from(b' ')
        || units[7] != u16::from(b' ')
        || units[10] != u16::from(b' ')
    {
        return None;
    }
    let weekday = name(&units[..3], &WEEKDAYS)?;
    let month = name(&units[4..7], &MONTHS)?;
    let day = decimal(&units[8..10])? as u8;
    let end = 11
        + units[11..]
            .iter()
            .position(|&unit| unit == u16::from(b' '))?;
    let year = &units[11..end];
    let negative = year.first() == Some(&u16::from(b'-'));
    let magnitude = if negative { &year[1..] } else { year };
    if !(4..=6).contains(&magnitude.len())
        || (magnitude.len() > 4 && magnitude[0] == u16::from(b'0'))
    {
        return None;
    }
    let year = decimal(magnitude)? as i32;
    if negative && year == 0 {
        return None;
    }
    let time = &units[end..];
    if time.len() < 18
        || time[0] != u16::from(b' ')
        || time[3] != u16::from(b':')
        || time[6] != u16::from(b':')
        || !ascii(&time[9..13], " GMT")
    {
        return None;
    }
    let offset_negative = negative_sign(time[13])?;
    let hours = decimal(&time[14..16])?;
    let minutes = decimal(&time[16..18])?;
    if hours >= 24 || minutes >= 60 {
        return None;
    }
    let offset = if time.len() == 18 {
        signed_offset(hours * 3600 + minutes * 60, offset_negative)?
    } else {
        let suffix = &time[18..];
        if suffix.len() < 15
            || !ascii(&suffix[..5], " (UTC")
            || suffix.last() != Some(&u16::from(b')'))
        {
            return None;
        }
        let negative = negative_sign(suffix[5])?;
        let end = 6 + suffix[6..]
            .iter()
            .position(|&unit| unit == u16::from(b':'))?;
        let hours = &suffix[6..end];
        if !(2..=6).contains(&hours.len())
            || (hours.len() > 2 && hours[0] == u16::from(b'0'))
            || suffix.len() != end + 7
            || suffix[end + 3] != u16::from(b':')
        {
            return None;
        }
        let minutes = decimal(&suffix[end + 1..end + 3])?;
        let seconds = decimal(&suffix[end + 4..end + 6])?;
        if minutes >= 60 || seconds >= 60 {
            return None;
        }
        let magnitude = decimal(hours)?
            .checked_mul(3600)?
            .checked_add(minutes * 60 + seconds)?;
        let offset = signed_offset(magnitude, negative)?;
        if !needs_exact_name(offset) {
            return None;
        }
        offset
    };
    let magnitude = offset.unsigned_abs();
    if offset_negative != (offset < 0)
        || hours != (magnitude / 3600) % 24
        || minutes != (magnitude / 60) % 60
    {
        return None;
    }
    let fields = DateTimeString {
        year: if negative { -year } else { year },
        month,
        day,
        hour: decimal(&time[1..3])? as u8,
        minute: decimal(&time[4..6])? as u8,
        second: decimal(&time[7..9])? as u8,
        millisecond: 0,
        zone: DateTimeZone::Utc,
    };
    let local = i64::try_from(fields.nominal_epoch_milliseconds()?).ok()?;
    let actual = UtcDateTime::from_epoch_milliseconds(local);
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
    if actual != expected {
        return None;
    }
    let utc = i128::from(local) - i128::from(offset) * 1000;
    if utc.abs() > i128::from(MAX_TIME_VALUE) {
        return None;
    }
    i64::try_from(utc).ok()
}

fn negative_sign(unit: u16) -> Option<bool> {
    match u8::try_from(unit).ok()? {
        b'+' => Some(false),
        b'-' => Some(true),
        _ => None,
    }
}

fn signed_offset(magnitude: u32, negative: bool) -> Option<i32> {
    let magnitude = i64::from(magnitude);
    i32::try_from(if negative { -magnitude } else { magnitude }).ok()
}

fn decimal(units: &[u16]) -> Option<u32> {
    units.iter().try_fold(0_u32, |value, &unit| {
        (u16::from(b'0')..=u16::from(b'9'))
            .contains(&unit)
            .then_some(())?;
        value
            .checked_mul(10)?
            .checked_add(u32::from(unit - u16::from(b'0')))
    })
}
