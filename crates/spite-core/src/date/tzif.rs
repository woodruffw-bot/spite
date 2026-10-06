//! Checked TZif offset histories, preserving explicit and recurring transitions.

use super::{RecurringTimeZone, RecurringTimeZoneError, posix::parse_with_syntax};

/// An owned UTC offset history loaded from TZif bytes.
///
/// Supports TZif versions 1–4 without leap-second tables. Offset queries preserve
/// the authoritative historical transitions and recurring rules separately.
/// Designations and daylight flags are validated but not exposed as display
/// names; these records implement offsets for
/// [GetNamedTimeZoneOffsetNanoseconds](https://262.ecma-international.org/17.0/#sec-getnamedtimezoneoffsetnanoseconds).
#[derive(Debug, Eq, PartialEq)]
pub struct TzifTimeZone {
    offsets: Vec<i32>,
    transitions: Vec<Transition>,
    recurring: Option<RecurringTimeZone>,
}

#[derive(Debug, Eq, PartialEq)]
struct Transition {
    seconds: i64,
    type_index: u8,
}

/// Failure to load a TZif offset history.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TzifTimeZoneError {
    /// Required data, counts, fields or ordering are malformed or truncated.
    InvalidData,
    /// The file uses an unknown TZif version.
    UnsupportedVersion,
    /// Leap-second time scales have not been implemented.
    UnsupportedLeapSeconds,
    /// Recurring rules require contradictory simultaneous offset changes.
    ConflictingRules,
    /// Native allocation of the offset history failed.
    Allocation,
}

impl std::fmt::Display for TzifTimeZoneError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::InvalidData => "invalid TZif offset data",
            Self::UnsupportedVersion => "unsupported TZif version",
            Self::UnsupportedLeapSeconds => "unsupported TZif leap-second time scale",
            Self::ConflictingRules => "conflicting TZif recurring rules",
            Self::Allocation => "TZif offset allocation failed",
        })
    }
}

impl std::error::Error for TzifTimeZoneError {}

impl TzifTimeZone {
    /// Loads a checked offset history without a data-size or calendar-year quota.
    ///
    /// Declared block lengths are checked against the supplied bytes before any
    /// allocation. Versions 2–4 use the authoritative 64-bit block; their legacy
    /// 32-bit block is skipped, as recommended by
    /// [RFC 9636](https://www.rfc-editor.org/rfc/rfc9636.html#section-4).
    /// Trailing extension data is ignored after the applicable block/footer.
    /// Nonempty leap-second tables report a distinct unsupported time scale.
    pub fn parse(bytes: &[u8]) -> Result<Self, TzifTimeZoneError> {
        let mut cursor = Cursor { bytes, position: 0 };
        let mut header = Header::read(&mut cursor)?;
        let width = if header.version == 0 {
            4
        } else {
            cursor.take(header.block_length(4)?)?;
            let next = Header::read(&mut cursor)?;
            if next.version != header.version {
                return Err(TzifTimeZoneError::InvalidData);
            }
            header = next;
            8
        };
        let block = cursor.take(header.block_length(width)?)?;
        let mut fields = Cursor {
            bytes: block,
            position: 0,
        };
        let times = fields.take(header.times * width)?;
        let indices = fields.take(header.times)?;
        let types = fields.take(header.types * 6)?;
        let names = fields.take(header.names)?;
        fields.take(header.leaps * (width + 4))?;
        let standard = fields.take(header.standard)?;
        let universal = fields.take(header.universal)?;

        for (index, record) in types.chunks_exact(6).enumerate() {
            let offset = i32::from_be_bytes(record[..4].try_into().expect("four-byte type offset"));
            let name = names
                .get(usize::from(record[5])..)
                .ok_or(TzifTimeZoneError::InvalidData)?;
            if offset == i32::MIN || record[4] > 1 || !name.contains(&0) {
                return Err(TzifTimeZoneError::InvalidData);
            }
            let std = standard.get(index).copied().unwrap_or(0);
            let ut = universal.get(index).copied().unwrap_or(0);
            if std > 1 || ut > 1 || ut > std {
                return Err(TzifTimeZoneError::InvalidData);
            }
        }
        let mut previous = None;
        for (time, &index) in times.chunks_exact(width).zip(indices) {
            let time = seconds(time);
            if usize::from(index) >= header.types || previous.is_some_and(|last| last >= time) {
                return Err(TzifTimeZoneError::InvalidData);
            }
            previous = Some(time);
        }
        if header.leaps != 0 {
            return Err(TzifTimeZoneError::UnsupportedLeapSeconds);
        }
        let mut offsets = Vec::new();
        offsets
            .try_reserve_exact(header.types)
            .map_err(|_| TzifTimeZoneError::Allocation)?;
        for record in types.chunks_exact(6) {
            offsets.push(i32::from_be_bytes(
                record[..4].try_into().expect("four-byte type offset"),
            ));
        }
        let mut transitions = Vec::new();
        transitions
            .try_reserve_exact(header.times)
            .map_err(|_| TzifTimeZoneError::Allocation)?;
        for (time, &type_index) in times.chunks_exact(width).zip(indices) {
            transitions.push(Transition {
                seconds: seconds(time),
                type_index,
            });
        }
        let recurring = if header.version == 0 {
            None
        } else {
            cursor.take_byte(b'\n')?;
            let remaining = &cursor.bytes[cursor.position..];
            let end = remaining
                .iter()
                .position(|&byte| byte == b'\n')
                .ok_or(TzifTimeZoneError::InvalidData)?;
            let text = std::str::from_utf8(&remaining[..end])
                .map_err(|_| TzifTimeZoneError::InvalidData)?;
            if text.is_empty() {
                None
            } else {
                let rules = parse_with_syntax(text, header.version >= b'3')
                    .ok_or(TzifTimeZoneError::InvalidData)?;
                let recurring =
                    RecurringTimeZone::from_posix(rules).map_err(|error| match error {
                        RecurringTimeZoneError::InvalidRule => TzifTimeZoneError::InvalidData,
                        RecurringTimeZoneError::ConflictingTransitions => {
                            TzifTimeZoneError::ConflictingRules
                        }
                        RecurringTimeZoneError::Allocation => TzifTimeZoneError::Allocation,
                    })?;
                if let Some(last) = transitions.last() {
                    if recurring.offset_at(i128::from(last.seconds) * 1000)
                        != offsets[usize::from(last.type_index)]
                    {
                        return Err(TzifTimeZoneError::InvalidData);
                    }
                }
                Some(recurring)
            }
        };
        Ok(Self {
            offsets,
            transitions,
            recurring,
        })
    }

    /// Returns all historical and recurring candidate offsets, including duplicates.
    ///
    /// Offsets are seconds east of UTC. Candidates for local-time resolution
    /// must be checked against `offset_at`; past types need not occur again.
    pub fn possible_offsets(&self) -> impl Iterator<Item = i32> + '_ {
        self.offsets.iter().copied().chain(
            self.recurring
                .iter()
                .flat_map(|zone| zone.possible_offsets()),
        )
    }

    /// Returns seconds east of UTC at exact native epoch milliseconds.
    ///
    /// Historical changes apply at their integral second, with type 0 before the
    /// first change. A recurring footer applies after the last historical change
    /// or to all times in a history without transitions. Without a footer, the
    /// last historical offset persists. Lookup allocates nothing and never clips
    /// its input to a calendar or Date range.
    pub fn offset_at(&self, epoch_milliseconds: i128) -> i32 {
        let seconds = epoch_milliseconds.div_euclid(1000);
        let position = self
            .transitions
            .partition_point(|transition| i128::from(transition.seconds) <= seconds);
        if position == self.transitions.len() {
            if let Some(recurring) = &self.recurring {
                return recurring.offset_at(epoch_milliseconds);
            }
        }
        let index = if position == 0 {
            0
        } else {
            usize::from(self.transitions[position - 1].type_index)
        };
        self.offsets[index]
    }
}

struct Header {
    version: u8,
    universal: usize,
    standard: usize,
    leaps: usize,
    times: usize,
    types: usize,
    names: usize,
}

impl Header {
    fn read(cursor: &mut Cursor<'_>) -> Result<Self, TzifTimeZoneError> {
        let bytes = cursor.take(44)?;
        if &bytes[..4] != b"TZif" || bytes[5..20].iter().any(|&byte| byte != 0) {
            return Err(TzifTimeZoneError::InvalidData);
        }
        if !matches!(bytes[4], 0 | b'2' | b'3' | b'4') {
            return Err(TzifTimeZoneError::UnsupportedVersion);
        }
        let count = |index| {
            u32::from_be_bytes(
                bytes[index..index + 4]
                    .try_into()
                    .expect("four-byte header count"),
            ) as usize
        };
        let header = Self {
            version: bytes[4],
            universal: count(20),
            standard: count(24),
            leaps: count(28),
            times: count(32),
            types: count(36),
            names: count(40),
        };
        if !(1..=256).contains(&header.types)
            || header.names == 0
            || ![0, header.types].contains(&header.standard)
            || ![0, header.types].contains(&header.universal)
        {
            return Err(TzifTimeZoneError::InvalidData);
        }
        Ok(header)
    }

    fn block_length(&self, width: usize) -> Result<usize, TzifTimeZoneError> {
        let mut length = 0_usize;
        for (count, size) in [
            (self.times, width),
            (self.times, 1),
            (self.types, 6),
            (self.names, 1),
            (self.leaps, width + 4),
            (self.standard, 1),
            (self.universal, 1),
        ] {
            length = count
                .checked_mul(size)
                .and_then(|size| length.checked_add(size))
                .ok_or(TzifTimeZoneError::InvalidData)?;
        }
        Ok(length)
    }
}

struct Cursor<'a> {
    bytes: &'a [u8],
    position: usize,
}

impl<'a> Cursor<'a> {
    fn take(&mut self, length: usize) -> Result<&'a [u8], TzifTimeZoneError> {
        let end = self
            .position
            .checked_add(length)
            .ok_or(TzifTimeZoneError::InvalidData)?;
        let bytes = self
            .bytes
            .get(self.position..end)
            .ok_or(TzifTimeZoneError::InvalidData)?;
        self.position = end;
        Ok(bytes)
    }

    fn take_byte(&mut self, expected: u8) -> Result<(), TzifTimeZoneError> {
        if self.take(1)?[0] != expected {
            return Err(TzifTimeZoneError::InvalidData);
        }
        Ok(())
    }
}

fn seconds(bytes: &[u8]) -> i64 {
    if bytes.len() == 4 {
        i64::from(i32::from_be_bytes(
            bytes.try_into().expect("four-byte transition"),
        ))
    } else {
        i64::from_be_bytes(bytes.try_into().expect("eight-byte transition"))
    }
}

#[cfg(test)]
mod tests;
