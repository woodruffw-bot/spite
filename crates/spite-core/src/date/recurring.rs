//! Gregorian offset cycles for recurring TZif political time-zone rules.

use super::{MS_PER_DAY, PosixTimeZone, day_from_year};

// Gregorian dates and weekdays repeat exactly after 400 years (146097 days).
const CYCLE_MILLISECONDS: i64 = 146_097 * MS_PER_DAY;

/// An offset history compiled from recurring POSIX time-zone rules.
///
/// This supports political rules for
/// [GetNamedTimeZoneOffsetNanoseconds](https://262.ecma-international.org/17.0/#sec-getnamedtimezoneoffsetnanoseconds)
/// without restricting queries to a time-zone backend's civil calendar range.
/// TZif's explicit historical transitions must be handled separately.
#[derive(Debug, Eq, PartialEq)]
pub struct RecurringTimeZone {
    offsets: [i32; 2],
    constant_offset: i32,
    transitions: Vec<Transition>,
}

#[derive(Debug, Eq, PartialEq)]
struct Transition {
    time: i64,
    offset: i32,
}

/// Failure to compile a recurring time-zone offset history.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RecurringTimeZoneError {
    /// A native rule record contains an invalid date or time field.
    InvalidRule,
    /// Distinct rules require different offsets at the same instant.
    ConflictingTransitions,
    /// Native allocation of the calendar cycle failed.
    Allocation,
}

impl std::fmt::Display for RecurringTimeZoneError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::InvalidRule => "invalid recurring time-zone rule",
            Self::ConflictingTransitions => "conflicting recurring time-zone transitions",
            Self::Allocation => "recurring time-zone allocation failed",
        })
    }
}

impl std::error::Error for RecurringTimeZoneError {}

impl RecurringTimeZone {
    /// Compiles a complete recurring Gregorian cycle from parsed TZif rules.
    ///
    /// The cycle contains at most two transitions for each of 400 years. This
    /// bound follows from the calendar and rule format, not a resource quota.
    /// Native allocation failure remains distinct from invalid/conflicting data.
    /// A daylight interval spanning a whole year has no standard-time interval,
    /// implementing TZif's year-round daylight-time extension
    /// ([RFC 9636](https://www.rfc-editor.org/rfc/rfc9636.html#section-3.3.1)).
    pub fn from_posix(zone: PosixTimeZone<'_>) -> Result<Self, RecurringTimeZoneError> {
        let Some(daylight) = zone.daylight else {
            return Ok(Self {
                offsets: [zone.standard_offset; 2],
                constant_offset: zone.standard_offset,
                transitions: Vec::new(),
            });
        };
        let mut transitions = Vec::new();
        transitions
            .try_reserve_exact(800)
            .map_err(|_| RecurringTimeZoneError::Allocation)?;
        for year in 1970..2370 {
            let start = daylight
                .start
                .epoch_milliseconds(year, zone.standard_offset, zone.standard_offset)
                .ok_or(RecurringTimeZoneError::InvalidRule)?;
            let end = daylight
                .end
                .epoch_milliseconds(year, daylight.offset, zone.standard_offset)
                .ok_or(RecurringTimeZoneError::InvalidRule)?;
            let year_length =
                i128::from(day_from_year(year + 1) - day_from_year(year)) * i128::from(MS_PER_DAY);
            // Reversed dates describe southern seasons. For ordered dates,
            // omit zero-length or whole-year daylight intervals as in tzcode's
            // recurring-rule expansion. If the whole cycle has no transitions,
            // daylight time is perpetual. Other years retain neighboring rules.
            if start > end || (start < end && end - start < year_length) {
                for (time, offset) in [(start, daylight.offset), (end, zone.standard_offset)] {
                    transitions.push(Transition {
                        time: time.rem_euclid(i128::from(CYCLE_MILLISECONDS)) as i64,
                        offset,
                    });
                }
            }
        }
        transitions.sort_unstable_by_key(|transition| transition.time);
        if transitions
            .windows(2)
            .any(|pair| pair[0].time == pair[1].time && pair[0].offset != pair[1].offset)
        {
            return Err(RecurringTimeZoneError::ConflictingTransitions);
        }
        transitions.dedup_by_key(|transition| transition.time);
        Ok(Self {
            offsets: [zone.standard_offset, daylight.offset],
            constant_offset: daylight.offset,
            transitions,
        })
    }

    /// Returns the rule's candidate offsets, in seconds east of UTC.
    ///
    /// A fixed zone repeats its single offset. A recurring zone can expose an
    /// offset with no occurrence, such as standard time under perpetual daylight
    /// rules. Local-time candidates must be checked against `offset_at`.
    pub fn possible_offsets(&self) -> [i32; 2] {
        self.offsets
    }

    /// Returns the offset in seconds east of UTC at exact epoch milliseconds.
    ///
    /// A transition applies at its exact instant. Integer Euclidean remainder
    /// maps negative and positive times into the mathematically identical
    /// Gregorian cycle, without clipping, rounding or a calendar-year bound.
    /// Lookup allocates nothing and handles every native `i128` input.
    pub fn offset_at(&self, epoch_milliseconds: i128) -> i32 {
        if self.transitions.is_empty() {
            return self.constant_offset;
        }
        let time = epoch_milliseconds.rem_euclid(i128::from(CYCLE_MILLISECONDS)) as i64;
        let position = self
            .transitions
            .partition_point(|transition| transition.time <= time);
        let index = if position == 0 {
            self.transitions.len() - 1
        } else {
            position - 1
        };
        self.transitions[index].offset
    }
}

#[cfg(test)]
mod tests;
