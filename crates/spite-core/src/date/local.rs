//! Shared local-time disambiguation for UTC (21.4.1.26).

/// Failure to resolve nominal local milliseconds into a native UTC instant.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LocalTimeZoneError {
    /// The offset history does not describe a valid preceding local instant.
    InvalidData,
    /// The mathematical result or a required intermediate exceeds native i128.
    ///
    /// This native representation boundary is distinct from Date's TimeClip.
    OutOfRange,
}

impl std::fmt::Display for LocalTimeZoneError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::InvalidData => "inconsistent local time-zone history",
            Self::OutOfRange => "local time-zone arithmetic exceeds native representation",
        })
    }
}

impl std::error::Error for LocalTimeZoneError {}

pub(super) fn resolve_local(
    local: i128,
    offsets: impl Iterator<Item = i32> + Clone,
    offset_at: impl Fn(i128) -> i32,
    local_before: impl Fn(i128) -> Result<Option<i128>, LocalTimeZoneError>,
) -> Result<i128, LocalTimeZoneError> {
    // UTC chooses the earliest epoch for a repeated wall time. For a gap, it
    // finds the last valid local time before the requested time, then the last
    // possible epoch for that local time, and uses that epoch's offset.
    // https://262.ecma-international.org/17.0/#sec-utc-t
    if let Some(utc) = possible_epoch(local, offsets.clone(), &offset_at, false)? {
        return Ok(utc);
    }
    let before = local_before(local)?.ok_or(LocalTimeZoneError::InvalidData)?;
    let utc_before = possible_epoch(before, offsets, &offset_at, true)?
        .ok_or(LocalTimeZoneError::InvalidData)?;
    local
        .checked_sub(i128::from(offset_at(utc_before)) * 1000)
        .ok_or(LocalTimeZoneError::OutOfRange)
}

fn possible_epoch(
    local: i128,
    offsets: impl Iterator<Item = i32>,
    offset_at: impl Fn(i128) -> i32,
    latest: bool,
) -> Result<Option<i128>, LocalTimeZoneError> {
    let mut selected: Option<i128> = None;
    for offset in offsets {
        let utc = local
            .checked_sub(i128::from(offset) * 1000)
            .ok_or(LocalTimeZoneError::OutOfRange)?;
        if offset_at(utc) == offset {
            selected = Some(match selected {
                None => utc,
                Some(previous) if latest => previous.max(utc),
                Some(previous) => previous.min(utc),
            });
        }
    }
    Ok(selected)
}
