use super::*;
use crate::Value;
use spite_core::date::{DateTimeString, DateTimeZone, MAX_TIME_VALUE};
use std::sync::atomic::{AtomicUsize, Ordering};

fn local(year: i32, month: u8, day: u8, hour: u8, minute: u8) -> i128 {
    DateTimeString {
        year,
        month: month - 1,
        day,
        hour,
        minute,
        second: 0,
        millisecond: 0,
        zone: DateTimeZone::Local,
    }
    .nominal_epoch_milliseconds()
    .unwrap()
}

struct Directory(PathBuf);

impl Directory {
    fn new() -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let path = env::temp_dir().join(format!(
            "spite-zone-{}-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
}

impl Drop for Directory {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn fixed_tzif(offset: i32) -> Vec<u8> {
    let mut bytes = b"TZif\0".to_vec();
    bytes.extend_from_slice(&[0; 15]);
    for count in [0_u32, 0, 0, 0, 1, 4] {
        bytes.extend_from_slice(&count.to_be_bytes());
    }
    bytes.extend_from_slice(&offset.to_be_bytes());
    bytes.extend_from_slice(b"\0\0ZZZ\0");
    bytes
}

#[test]
fn fixed_offsets_preserve_every_native_offset_and_check_only_integer_capacity() {
    for offset in [i32::MIN, -3600, 0, 1234, 3600, i32::MAX] {
        let zone = TimeZone::fixed(offset);
        for time in [
            i128::MIN,
            -i128::from(MAX_TIME_VALUE),
            0,
            i128::from(MAX_TIME_VALUE),
            i128::MAX,
        ] {
            assert_eq!(zone.offset_at(time), offset);
        }
        for time in [-i128::from(MAX_TIME_VALUE), 0, i128::from(MAX_TIME_VALUE)] {
            assert_eq!(
                zone.resolve_local(time),
                Ok(time - i128::from(offset) * 1000)
            );
        }
    }
    assert_eq!(
        TimeZone::fixed(1).resolve_local(i128::MIN),
        Err(LocalTimeZoneError::OutOfRange)
    );
    assert_eq!(
        TimeZone::fixed(-1).resolve_local(i128::MAX),
        Err(LocalTimeZoneError::OutOfRange)
    );
    assert_eq!(TimeZone::utc().resolve_local(i128::MIN), Ok(i128::MIN));
    assert_eq!(TimeZone::utc().name(), Some("UTC"));
}

#[test]
fn every_pinned_bundled_zone_loads_and_resolves_both_date_endpoints() {
    assert_eq!(jiff_tzdb::VERSION, Some("2026c"));
    let mut count = 0;
    for name in jiff_tzdb::available() {
        let zone = TimeZone::bundled(name).unwrap();
        assert_eq!(zone.name(), Some(name));
        for epoch in [
            -i128::from(MAX_TIME_VALUE),
            -1,
            0,
            i128::from(MAX_TIME_VALUE),
        ] {
            let wall = epoch + i128::from(zone.offset_at(epoch)) * 1000;
            let resolved = zone.resolve_local(wall).unwrap();
            assert!(resolved <= epoch, "{name} {epoch}");
            assert_eq!(
                resolved + i128::from(zone.offset_at(resolved)) * 1000,
                wall,
                "{name} {epoch}"
            );
        }
        let _ = zone.offset_at(i128::MIN);
        let _ = zone.offset_at(i128::MAX);
        count += 1;
    }
    assert_eq!(count, 598);
}

#[test]
fn bundled_aliases_and_historical_seconds_preserve_correct_gap_and_fold_choices() {
    let ny = TimeZone::bundled("america/NEW_YORK").unwrap();
    assert_eq!(ny.name(), Some("America/New_York"));
    assert_eq!(
        ny.resolve_local(local(2024, 3, 10, 2, 30)),
        Ok(local(2024, 3, 10, 7, 30))
    );
    assert_eq!(
        ny.resolve_local(local(2024, 11, 3, 1, 30)),
        Ok(local(2024, 11, 3, 5, 30))
    );
    let alias = TimeZone::bundled("us/eastern").unwrap();
    assert_eq!(alias.name(), Some("US/Eastern"));
    assert_eq!(alias.offset_at(local(1800, 1, 1, 0, 0)), -17762);
    let lord_howe = TimeZone::bundled("Australia/Lord_Howe").unwrap();
    assert_eq!(
        lord_howe.resolve_local(local(2024, 10, 6, 2, 15)),
        Ok(local(2024, 10, 5, 15, 45))
    );
    assert_eq!(
        lord_howe.resolve_local(local(2024, 4, 7, 1, 45)),
        Ok(local(2024, 4, 6, 14, 45))
    );
    let abidjan = TimeZone::bundled("Africa/Abidjan").unwrap();
    assert_eq!(abidjan.offset_at(-1_830_383_032_001), -968);
    assert_eq!(abidjan.offset_at(-1_830_383_032_000), 0);
}

#[test]
fn recurring_rules_reach_wide_years_without_replacing_explicit_history() {
    let recurring = TimeZone::from_posix("EST5EDT,M3.2.0,M11.1.0").unwrap();
    let ny = TimeZone::bundled("America/New_York").unwrap();
    for year in [2024, 12000, 200000] {
        for month in [1, 7] {
            let time = local(year, month, 1, 12, 0);
            assert_eq!(recurring.offset_at(time), ny.offset_at(time));
            assert_eq!(recurring.resolve_local(time), ny.resolve_local(time));
        }
    }
    assert_eq!(recurring.offset_at(local(1800, 1, 1, 0, 0)), -18000);
    assert_eq!(ny.offset_at(local(1800, 1, 1, 0, 0)), -17762);
    assert!(matches!(
        TimeZone::from_posix("EST5EDT"),
        Err(TimeZoneError::InvalidPosixRule)
    ));
}

#[test]
fn host_files_take_priority_and_existing_invalid_data_never_uses_fallback() {
    let first = Directory::new();
    let second = Directory::new();
    fs::write(first.0.join("UTC"), fixed_tzif(1234)).unwrap();
    fs::write(second.0.join("UTC"), fixed_tzif(5678)).unwrap();
    let zone = TimeZone::named_from_roots("utc", [first.0.clone(), second.0.clone()]).unwrap();
    assert_eq!(zone.name(), Some("UTC"));
    assert_eq!(zone.offset_at(0), 1234);
    fs::write(first.0.join("UTC"), b"invalid").unwrap();
    assert!(matches!(
        TimeZone::named_from_roots("UTC", [first.0.clone(), second.0.clone()]),
        Err(TimeZoneError::Tzif(TzifTimeZoneError::InvalidData))
    ));
    fs::remove_file(first.0.join("UTC")).unwrap();
    assert_eq!(
        TimeZone::named_from_roots("UTC", [first.0.clone(), second.0.clone()])
            .unwrap()
            .offset_at(0),
        5678
    );
    fs::remove_file(second.0.join("UTC")).unwrap();
    assert_eq!(
        TimeZone::named_from_roots("UTC", [first.0.clone()])
            .unwrap()
            .offset_at(0),
        0
    );
    fs::create_dir(first.0.join("UTC")).unwrap();
    assert!(matches!(
        TimeZone::named_from_roots("UTC", [first.0.clone()]),
        Err(TimeZoneError::Io(_))
    ));
    fs::create_dir(first.0.join("Spite")).unwrap();
    fs::write(first.0.join("Spite/New_Zone"), fixed_tzif(2345)).unwrap();
    let custom = TimeZone::named_from_roots("Spite/New_Zone", [first.0.clone()]).unwrap();
    assert_eq!(custom.name(), Some("Spite/New_Zone"));
    assert_eq!(custom.offset_at(0), 2345);
}

#[test]
fn named_identifiers_reject_path_traversal_and_report_unknown_names() {
    for name in [
        "",
        "/etc/passwd",
        "../UTC",
        "America/../New_York",
        "America//New_York",
        "America/./New_York",
        "UTC/",
        "C:/UTC",
        "America\\New_York",
        "Étc/UTC",
        "UTC\0",
    ] {
        assert!(
            matches!(TimeZone::bundled(name), Err(TimeZoneError::InvalidName)),
            "{name:?}"
        );
        assert!(
            matches!(
                TimeZone::named_from_roots(name, []),
                Err(TimeZoneError::InvalidName)
            ),
            "{name:?}"
        );
    }
    assert!(matches!(
        TimeZone::bundled("Spite/Unknown"),
        Err(TimeZoneError::UnknownName)
    ));
}

#[test]
fn explicit_tz_settings_support_rules_names_empty_utc_and_unnamed_files() {
    assert_eq!(
        TimeZone::from_tz_setting(OsStr::new("")).unwrap().name(),
        Some("UTC")
    );
    assert_eq!(
        TimeZone::from_tz_setting(OsStr::new("UTC0"))
            .unwrap()
            .offset_at(0),
        0
    );
    assert_eq!(
        TimeZone::from_tz_setting(OsStr::new("ABC-2:30"))
            .unwrap()
            .offset_at(0),
        9000
    );
    assert_eq!(
        TimeZone::from_tz_setting(OsStr::new(":america/new_york"))
            .unwrap()
            .name(),
        Some("America/New_York")
    );
    let root = Directory::new();
    let path = root.0.join("unnamed");
    fs::write(&path, fixed_tzif(-1234)).unwrap();
    for path in [
        path.clone().into_os_string(),
        format!(":{}", path.display()).into(),
    ] {
        let zone = TimeZone::from_tz_setting(&path).unwrap();
        assert_eq!(zone.name(), None);
        assert_eq!(zone.offset_at(0), -1234);
    }
    fs::write(&path, b"invalid").unwrap();
    assert!(matches!(
        TimeZone::from_tz_setting(path.as_os_str()),
        Err(TimeZoneError::Tzif(TzifTimeZoneError::InvalidData))
    ));
}

#[test]
fn realm_overrides_are_independent_and_utc_operations_do_not_load_host_zones() {
    let mut first = Realm::default();
    let mut second = Realm::default();
    assert_eq!(
        first.eval("new Date(0).getUTCFullYear()===1970 && Number.isNaN(new Date(NaN).getHours())"),
        Ok(Value::Boolean(true))
    );
    assert!(first.time_zone.is_none());
    first.set_time_zone(TimeZone::utc());
    let ny = TimeZone::bundled("America/New_York").unwrap();
    second.set_time_zone(ny.clone());
    assert_eq!(first.time_zone().unwrap().offset_at(0), 0);
    assert_eq!(second.time_zone().unwrap().offset_at(0), -18000);
    first.set_time_zone(ny);
    second.set_time_zone(TimeZone::fixed(1234));
    assert_eq!(first.time_zone().unwrap().offset_at(0), -18000);
    assert_eq!(second.time_zone().unwrap().offset_at(0), 1234);
    first.use_system_time_zone();
    assert!(first.time_zone.is_none());
    assert_eq!(second.time_zone().unwrap().offset_at(0), 1234);
}
