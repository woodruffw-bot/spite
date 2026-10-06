//! Borrowed parsing of TZif's recurring POSIX time-zone string.

use super::{TransitionClock, TransitionDay, TransitionRule};

/// A parsed TZif POSIX time-zone string, borrowing its ASCII designations.
///
/// Offsets are seconds east of UTC, reversing POSIX's written sign convention.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PosixTimeZone<'a> {
    /// Standard-time designation without angle brackets.
    pub standard_name: &'a str,
    /// Standard-time offset, in seconds east of UTC.
    pub standard_offset: i32,
    /// Daylight time with explicit recurring rules, when present.
    pub daylight: Option<PosixDaylightTime<'a>>,
}

/// A daylight-time designation, offset and paired recurring transition rules.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PosixDaylightTime<'a> {
    /// Daylight-time designation without angle brackets.
    pub name: &'a str,
    /// Daylight-time offset, in seconds east of UTC.
    pub offset: i32,
    /// Transition from standard time into daylight time.
    pub start: TransitionRule,
    /// Transition from daylight time into standard time.
    pub end: TransitionRule,
}

/// Parses a complete recurring time-zone string in TZif version 3/4 notation.
///
/// See [RFC 9636](https://www.rfc-editor.org/rfc/rfc9636.html#section-3.3).
/// Daylight rules must be explicit; no platform default rules are inferred.
/// Extended transition hours can be signed and range from −167 through 167.
/// Empty strings and malformed names, offsets, rules or trailing input return
/// `None`. Names are borrowed without allocation or an arbitrary length limit.
/// Parsing does not load a zone or resolve an offset at an instant.
pub fn parse_posix_time_zone(text: &str) -> Option<PosixTimeZone<'_>> {
    let mut parser = Parser { text, position: 0 };
    let standard_name = parser.name()?;
    let standard_offset = -parser.time(24, 2)?;
    let daylight = if parser.position == text.len() {
        None
    } else {
        let name = parser.name()?;
        let offset = if parser.peek() == Some(b',') {
            standard_offset + 3600
        } else {
            -parser.time(24, 2)?
        };
        parser.require(b',')?;
        let start = parser.rule()?;
        parser.require(b',')?;
        let end = parser.rule()?;
        Some(PosixDaylightTime {
            name,
            offset,
            start,
            end,
        })
    };
    if parser.position != text.len() {
        return None;
    }
    Some(PosixTimeZone {
        standard_name,
        standard_offset,
        daylight,
    })
}

struct Parser<'a> {
    text: &'a str,
    position: usize,
}

impl<'a> Parser<'a> {
    fn peek(&self) -> Option<u8> {
        self.text.as_bytes().get(self.position).copied()
    }

    fn consume(&mut self, byte: u8) -> bool {
        if self.peek() == Some(byte) {
            self.position += 1;
            true
        } else {
            false
        }
    }

    fn require(&mut self, byte: u8) -> Option<()> {
        self.consume(byte).then_some(())
    }

    fn name(&mut self) -> Option<&'a str> {
        let quoted = self.consume(b'<');
        let start = self.position;
        while self.peek().is_some_and(|byte| {
            byte.is_ascii_alphabetic()
                || (quoted && (byte.is_ascii_digit() || matches!(byte, b'+' | b'-')))
        }) {
            self.position += 1;
        }
        let end = self.position;
        if end - start < 3 || (quoted && !self.consume(b'>')) {
            return None;
        }
        // Only ASCII bytes have advanced these boundaries, even on non-ASCII input.
        Some(&self.text[start..end])
    }

    fn number(&mut self, digits: usize, maximum: u16, exact: bool) -> Option<u16> {
        let start = self.position;
        let mut value = 0;
        while self.position - start < digits {
            let Some(byte @ b'0'..=b'9') = self.peek() else {
                break;
            };
            value = value * 10 + u16::from(byte - b'0');
            self.position += 1;
        }
        let count = self.position - start;
        if count == 0 || (exact && count != digits) || value > maximum {
            return None;
        }
        Some(value)
    }

    fn time(&mut self, maximum_hour: u16, hour_digits: usize) -> Option<i32> {
        let negative = self.consume(b'-');
        if !negative {
            self.consume(b'+');
        }
        let hour = self.number(hour_digits, maximum_hour, false)?;
        let minute = if self.consume(b':') {
            self.number(2, 59, true)?
        } else {
            0
        };
        let second = if self.consume(b':') {
            self.number(2, 59, true)?
        } else {
            0
        };
        let value = i32::from(hour) * 3600 + i32::from(minute) * 60 + i32::from(second);
        Some(if negative { -value } else { value })
    }

    fn rule(&mut self) -> Option<TransitionRule> {
        let day = if self.consume(b'J') {
            let value = self.number(3, 365, false)?;
            if value == 0 {
                return None;
            }
            TransitionDay::JulianWithoutLeap(value)
        } else if self.consume(b'M') {
            let month = self.number(2, 12, false)? as u8;
            self.require(b'.')?;
            let week = self.number(1, 5, true)? as u8;
            self.require(b'.')?;
            let weekday = self.number(1, 6, true)? as u8;
            if month == 0 || week == 0 {
                return None;
            }
            TransitionDay::MonthWeekday {
                month,
                week,
                weekday,
            }
        } else {
            TransitionDay::JulianWithLeap(self.number(3, 365, false)?)
        };
        let seconds = if self.consume(b'/') {
            self.time(167, 3)?
        } else {
            7200
        };
        Some(TransitionRule {
            day,
            seconds,
            clock: TransitionClock::Wall,
        })
    }
}

#[cfg(test)]
mod tests;
