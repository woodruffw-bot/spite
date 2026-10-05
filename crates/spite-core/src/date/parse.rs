//! Date Time String Format syntax and absent-element defaults (21.4.1, 21.4.3.2).

use crate::JsString;

/// Zone interpretation of a parsed Date Time String Format instance.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DateTimeZone {
    /// An explicit Z or a date-only form with no offset representation.
    Utc,
    /// Signed minutes east of UTC, from an explicit ±HH:mm representation.
    OffsetMinutes(i16),
    /// A date-time form without an offset, requiring the host's local zone.
    Local,
}

/// Syntactically valid interchange date/time fields, with absent-element defaults.
///
/// Expanded years can lie outside TimeClip's domain. Converting fields into a
/// time value, including calendar normalization and zone resolution, is separate.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DateTimeString {
    /// Four-digit or signed six-digit year, including zero and negative years.
    pub year: i32,
    /// Zero-based month, from January (0) through December (11).
    pub month: u8,
    /// Day element from 1 through 31; absent days default to 1.
    pub day: u8,
    /// Hour element from 0 through 24; 24 requires all smaller elements to be zero.
    pub hour: u8,
    /// Minute element from 0 through 59, defaulting to 0.
    pub minute: u8,
    /// Second element from 0 through 59, defaulting to 0.
    pub second: u8,
    /// Three-digit millisecond element from 0 through 999, defaulting to 0.
    pub millisecond: u16,
    /// UTC, explicit offset, or unresolved local time.
    pub zone: DateTimeZone,
}

/// Parses the edition-17 Date Time String Format and its absent-element defaults.
///
/// Returns `None` for nonconforming syntax or out-of-bounds format elements.
/// Parsing borrows UTF-16 directly without allocating, interpreting local time,
/// clipping instants, or applying implementation-specific fallback formats.
/// See [Date Time String Format](https://262.ecma-international.org/17.0/#sec-date-time-string-format)
/// and [Date.parse](https://262.ecma-international.org/17.0/#sec-date.parse).
pub fn parse_date_time_string(source: &JsString) -> Option<DateTimeString> {
    let mut input = Input {
        units: source.code_units(),
        index: 0,
    };
    let expanded_sign = if input.take(b'+') {
        Some(1)
    } else if input.take(b'-') {
        Some(-1)
    } else {
        None
    };
    let year = input.digits(if expanded_sign.is_some() { 6 } else { 4 })?;
    if expanded_sign == Some(-1) && year == 0 {
        return None;
    }
    let mut result = DateTimeString {
        year: expanded_sign.unwrap_or(1) * year,
        month: 0,
        day: 1,
        hour: 0,
        minute: 0,
        second: 0,
        millisecond: 0,
        zone: DateTimeZone::Utc,
    };
    if input.take(b'-') {
        let month = input.digits(2)?;
        if !(1..=12).contains(&month) {
            return None;
        }
        result.month = (month - 1) as u8;
        if input.take(b'-') {
            let day = input.digits(2)?;
            if !(1..=31).contains(&day) {
                return None;
            }
            result.day = day as u8;
        }
    }
    if input.take(b'T') {
        result.zone = DateTimeZone::Local;
        let hour = input.digits(2)?;
        if hour > 24 || !input.take(b':') {
            return None;
        }
        let minute = input.digits(2)?;
        if minute > 59 {
            return None;
        }
        result.hour = hour as u8;
        result.minute = minute as u8;
        if input.take(b':') {
            let second = input.digits(2)?;
            if second > 59 {
                return None;
            }
            result.second = second as u8;
            if input.take(b'.') {
                result.millisecond = input.digits(3)? as u16;
            }
        }
        if hour == 24 && (minute != 0 || result.second != 0 || result.millisecond != 0) {
            return None;
        }
        if input.take(b'Z') {
            result.zone = DateTimeZone::Utc;
        } else {
            let sign = if input.take(b'+') {
                Some(1)
            } else if input.take(b'-') {
                Some(-1)
            } else {
                None
            };
            if let Some(sign) = sign {
                let hour = input.digits(2)?;
                if hour > 23 || !input.take(b':') {
                    return None;
                }
                let minute = input.digits(2)?;
                if minute > 59 {
                    return None;
                }
                result.zone = DateTimeZone::OffsetMinutes(sign * (hour * 60 + minute) as i16);
            }
        }
    }
    (input.index == input.units.len()).then_some(result)
}

struct Input<'a> {
    units: &'a [u16],
    index: usize,
}

impl Input<'_> {
    fn take(&mut self, expected: u8) -> bool {
        if self.units.get(self.index) == Some(&u16::from(expected)) {
            self.index += 1;
            true
        } else {
            false
        }
    }

    fn digits(&mut self, count: usize) -> Option<i32> {
        let mut value = 0;
        for _ in 0..count {
            let unit = *self.units.get(self.index)?;
            if !(u16::from(b'0')..=u16::from(b'9')).contains(&unit) {
                return None;
            }
            value = value * 10 + i32::from(unit - u16::from(b'0'));
            self.index += 1;
        }
        Some(value)
    }
}

#[cfg(test)]
mod tests;
