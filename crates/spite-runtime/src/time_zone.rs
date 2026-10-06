//! Immutable host time-zone histories for full-range Date operations.

use crate::Realm;
use spite_core::date::{
    LocalTimeZoneError, RecurringTimeZone, RecurringTimeZoneError, TzifTimeZone, TzifTimeZoneError,
    parse_posix_time_zone,
};
use std::{borrow::Cow, env, ffi::OsStr, fmt, fs, io, path::Path, path::PathBuf, sync::Arc};

#[cfg(test)]
mod tests;

/// A loaded, immutable time-zone history that can be shared between realms.
///
/// Historical offsets retain their exact TZif instants. Recurring rules apply
/// only where the history specifies them, without a backend calendar-year cap.
/// Loading may access host configuration; queries perform no I/O or allocation.
#[derive(Clone, Debug)]
pub struct TimeZone(Arc<Zone>);

#[derive(Debug)]
struct Zone {
    name: Option<Cow<'static, str>>,
    history: History,
}

#[derive(Debug)]
enum History {
    Fixed(i32),
    Tzif(TzifTimeZone),
    Recurring(RecurringTimeZone),
}

/// A host time-zone configuration or data-loading failure.
///
/// These native errors are neither JavaScript exceptions nor embedding quotas.
#[derive(Debug)]
pub enum TimeZoneError {
    /// A named identifier contains invalid characters or path components.
    InvalidName,
    /// The host and bundled databases do not contain the named identifier.
    UnknownName,
    /// A complete explicit POSIX rule could not be parsed.
    InvalidPosixRule,
    /// TZif data could not be loaded as a checked offset history.
    Tzif(TzifTimeZoneError),
    /// Parsed recurring rules could not be compiled consistently.
    Recurring(RecurringTimeZoneError),
    /// A configured file could not be read.
    Io(io::Error),
    /// The platform could not identify a usable system time zone.
    System(String),
}

impl fmt::Display for TimeZoneError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidName => f.write_str("invalid named time-zone identifier"),
            Self::UnknownName => f.write_str("unknown named time-zone identifier"),
            Self::InvalidPosixRule => f.write_str("invalid explicit POSIX time-zone rule"),
            Self::Tzif(error) => write!(f, "{error}"),
            Self::Recurring(error) => write!(f, "{error}"),
            Self::Io(error) => write!(f, "time-zone file: {error}"),
            Self::System(message) => write!(f, "system time zone: {message}"),
        }
    }
}

impl std::error::Error for TimeZoneError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Tzif(error) => Some(error),
            Self::Recurring(error) => Some(error),
            Self::Io(error) => Some(error),
            _ => None,
        }
    }
}

impl TimeZone {
    /// Creates the UTC time zone without consulting host configuration.
    pub fn utc() -> Self {
        Self::new(Some(Cow::Borrowed("UTC")), History::Fixed(0))
    }

    /// Creates a constant offset, in whole seconds east of UTC.
    ///
    /// Every native i32 offset is accepted. No Date clipping or host quota is
    /// applied to an offset or to a query's intermediate calendar value.
    pub fn fixed(seconds_east: i32) -> Self {
        Self::new(None, History::Fixed(seconds_east))
    }

    /// Loads TZif bytes without assuming a named identifier.
    pub fn from_tzif(bytes: &[u8]) -> Result<Self, TimeZoneError> {
        Self::load_tzif(bytes, None)
    }

    /// Loads a complete POSIX rule, with explicit daylight transition dates.
    ///
    /// Missing daylight rules never acquire platform-specific default dates.
    pub fn from_posix(rule: &str) -> Result<Self, TimeZoneError> {
        let parsed = parse_posix_time_zone(rule).ok_or(TimeZoneError::InvalidPosixRule)?;
        let zone = RecurringTimeZone::from_posix(parsed).map_err(TimeZoneError::Recurring)?;
        Ok(Self::new(None, History::Recurring(zone)))
    }

    /// Loads a case-insensitive identifier from the pinned IANA 2026c bundle.
    ///
    /// This entry point performs no host discovery or filesystem access and is
    /// suitable for reproducible embeddings. Aliases retain their IANA spelling.
    pub fn bundled(name: &str) -> Result<Self, TimeZoneError> {
        validate_name(name)?;
        let (name, bytes) = jiff_tzdb::get(name).ok_or(TimeZoneError::UnknownName)?;
        Self::load_tzif(bytes, Some(Cow::Borrowed(name)))
    }

    /// Loads a named identifier from host TZif files, falling back to the bundle.
    ///
    /// Searches TZDIR, then the conventional Unix zoneinfo directories. Missing
    /// files permit fallback; unreadable or malformed existing files report an
    /// error instead of silently substituting a different database version.
    /// Named inputs cannot escape a zoneinfo directory through path components.
    pub fn named(name: &str) -> Result<Self, TimeZoneError> {
        let roots = env::var_os("TZDIR").map(PathBuf::from).into_iter().chain([
            PathBuf::from("/usr/share/zoneinfo"),
            PathBuf::from("/usr/share/lib/zoneinfo"),
            PathBuf::from("/usr/lib/zoneinfo"),
        ]);
        Self::named_from_roots(name, roots)
    }

    /// Loads the host's configured system zone, including its TZ override.
    ///
    /// TZ may be an IANA identifier, a complete POSIX rule, or an explicit TZif
    /// file path. An empty TZ selects UTC. Unix localtime files retain their raw
    /// historical data even when unnamed; Jiff handles platform discovery on
    /// Windows and other supported systems. Failure never silently selects UTC.
    pub fn system() -> Result<Self, TimeZoneError> {
        if let Some(tz) = env::var_os("TZ") {
            return Self::from_tz_setting(&tz);
        }
        #[cfg(unix)]
        if let Some(bytes) = read_optional(Path::new("/etc/localtime"))? {
            let name = fs::read_link("/etc/localtime").ok().and_then(|path| {
                let text = path.to_str()?;
                let name = text.rsplit_once("zoneinfo/")?.1;
                validate_name(name).ok()?;
                Some(canonical_name(name))
            });
            return Self::load_tzif(&bytes, name);
        }
        let detected = jiff::tz::TimeZone::try_system()
            .map_err(|error| TimeZoneError::System(error.to_string()))?;
        if let Some(name) = detected.iana_name() {
            return Self::named(name);
        }
        if let Ok(offset) = detected.to_fixed_offset() {
            return Ok(Self::fixed(offset.seconds()));
        }
        Err(TimeZoneError::System(
            "anonymous platform history is unavailable as TZif".into(),
        ))
    }

    /// Returns the IANA spelling when this history has a named identifier.
    pub fn name(&self) -> Option<&str> {
        self.0.name.as_deref()
    }

    /// Returns whole seconds east of UTC at an exact native epoch millisecond.
    pub fn offset_at(&self, epoch_milliseconds: i128) -> i32 {
        match &self.0.history {
            History::Fixed(offset) => *offset,
            History::Tzif(zone) => zone.offset_at(epoch_milliseconds),
            History::Recurring(zone) => zone.offset_at(epoch_milliseconds),
        }
    }

    /// Resolves nominal local milliseconds with ECMAScript UTC's gap/fold rules.
    ///
    /// Repeated times choose the earliest epoch; skipped times use the offset
    /// at the latest epoch corresponding to the last preceding valid local ms.
    /// Date clipping remains with the caller; native i128 overflow is separate.
    pub fn resolve_local(&self, local_milliseconds: i128) -> Result<i128, LocalTimeZoneError> {
        match &self.0.history {
            History::Fixed(offset) => local_milliseconds
                .checked_sub(i128::from(*offset) * 1000)
                .ok_or(LocalTimeZoneError::OutOfRange),
            History::Tzif(zone) => zone.resolve_local(local_milliseconds),
            History::Recurring(zone) => zone.resolve_local(local_milliseconds),
        }
    }

    fn new(name: Option<Cow<'static, str>>, history: History) -> Self {
        Self(Arc::new(Zone { name, history }))
    }

    fn load_tzif(bytes: &[u8], name: Option<Cow<'static, str>>) -> Result<Self, TimeZoneError> {
        let history = TzifTimeZone::parse(bytes).map_err(TimeZoneError::Tzif)?;
        Ok(Self::new(name, History::Tzif(history)))
    }

    fn named_from_roots(
        name: &str,
        roots: impl IntoIterator<Item = PathBuf>,
    ) -> Result<Self, TimeZoneError> {
        validate_name(name)?;
        let name = canonical_name(name);
        for root in roots {
            if let Some(bytes) = read_optional(&root.join(name.as_ref()))? {
                return Self::load_tzif(&bytes, Some(name));
            }
        }
        let (name, bytes) = jiff_tzdb::get(&name).ok_or(TimeZoneError::UnknownName)?;
        Self::load_tzif(bytes, Some(Cow::Borrowed(name)))
    }

    fn from_tz_setting(setting: &OsStr) -> Result<Self, TimeZoneError> {
        if setting.is_empty() {
            return Ok(Self::utc());
        }
        if let Some(text) = setting.to_str() {
            let (text, implementation) = match text.strip_prefix(':') {
                Some(text) => (text, true),
                None => (text, false),
            };
            if !implementation && parse_posix_time_zone(text).is_some() {
                return Self::from_posix(text);
            }
            match Self::named(text) {
                Ok(zone) => return Ok(zone),
                Err(TimeZoneError::InvalidName | TimeZoneError::UnknownName) => {}
                Err(error) => return Err(error),
            }
            return fs::read(text)
                .map_err(TimeZoneError::Io)
                .and_then(|bytes| Self::from_tzif(&bytes));
        }
        fs::read(Path::new(setting))
            .map_err(TimeZoneError::Io)
            .and_then(|bytes| Self::from_tzif(&bytes))
    }
}

impl Realm {
    /// Selects a loaded time zone for this realm, independently of other realms.
    ///
    /// The immutable history can be cloned cheaply and reused by many realms.
    pub fn set_time_zone(&mut self, time_zone: TimeZone) {
        self.time_zone = Some(time_zone);
    }

    /// Discards the selected zone so the next lookup reloads host configuration.
    pub fn use_system_time_zone(&mut self) {
        self.time_zone = None;
    }

    /// Returns this realm's zone, loading the system zone lazily if necessary.
    ///
    /// A failed load leaves the realm unconfigured so the host may retry or
    /// install an explicit override. No default resource allowance is imposed.
    pub fn time_zone(&mut self) -> Result<&TimeZone, TimeZoneError> {
        if self.time_zone.is_none() {
            self.time_zone = Some(TimeZone::system()?);
        }
        Ok(self.time_zone.as_ref().expect("loaded time zone"))
    }
}

fn validate_name(name: &str) -> Result<(), TimeZoneError> {
    if name.is_empty()
        || name
            .split('/')
            .any(|part| part.is_empty() || part == "." || part == "..")
        || !name
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || b"/_+-.".contains(&byte))
    {
        return Err(TimeZoneError::InvalidName);
    }
    Ok(())
}

fn canonical_name(name: &str) -> Cow<'static, str> {
    if let Some((name, _)) = jiff_tzdb::get(name) {
        return Cow::Borrowed(name);
    }
    jiff::tz::db()
        .available()
        .find(|entry| entry.as_str().eq_ignore_ascii_case(name))
        .map_or_else(
            || Cow::Owned(name.to_owned()),
            |name| Cow::Owned(name.to_string()),
        )
}

fn read_optional(path: &Path) -> Result<Option<Vec<u8>>, TimeZoneError> {
    match fs::read(path) {
        Ok(bytes) => Ok(Some(bytes)),
        Err(error)
            if matches!(
                error.kind(),
                io::ErrorKind::NotFound | io::ErrorKind::NotADirectory
            ) =>
        {
            Ok(None)
        }
        Err(error) => Err(TimeZoneError::Io(error)),
    }
}
