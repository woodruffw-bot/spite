use super::*;
use crate::date::{DateTimeString, DateTimeZone, MAX_TIME_VALUE};

struct Data {
    times: Vec<(i64, u8)>,
    types: Vec<(i32, u8, u8)>,
    names: Vec<u8>,
    leaps: Vec<(i64, i32)>,
    standard: Vec<u8>,
    universal: Vec<u8>,
}

impl Default for Data {
    fn default() -> Self {
        Self {
            times: Vec::new(),
            types: vec![(0, 0, 0)],
            names: b"ABC\0".to_vec(),
            leaps: Vec::new(),
            standard: Vec::new(),
            universal: Vec::new(),
        }
    }
}

fn block(version: u8, width: usize, data: &Data) -> Vec<u8> {
    let mut bytes = b"TZif".to_vec();
    bytes.push(version);
    bytes.extend([0; 15]);
    for count in [
        data.universal.len(),
        data.standard.len(),
        data.leaps.len(),
        data.times.len(),
        data.types.len(),
        data.names.len(),
    ] {
        bytes.extend(u32::try_from(count).unwrap().to_be_bytes());
    }
    let append_time = |bytes: &mut Vec<u8>, time: i64| {
        if width == 4 {
            bytes.extend(i32::try_from(time).unwrap().to_be_bytes());
        } else {
            bytes.extend(time.to_be_bytes());
        }
    };
    for &(time, _) in &data.times {
        append_time(&mut bytes, time);
    }
    for &(_, index) in &data.times {
        bytes.push(index);
    }
    for &(offset, daylight, name_index) in &data.types {
        bytes.extend(offset.to_be_bytes());
        bytes.extend([daylight, name_index]);
    }
    bytes.extend(&data.names);
    for &(time, correction) in &data.leaps {
        append_time(&mut bytes, time);
        bytes.extend(correction.to_be_bytes());
    }
    bytes.extend(&data.standard);
    bytes.extend(&data.universal);
    bytes
}

fn file(version: u8, data: &Data, footer: &str) -> Vec<u8> {
    if version == 0 {
        return block(0, 4, data);
    }
    let mut bytes = block(version, 4, &Data::default());
    bytes.extend(block(version, 8, data));
    bytes.push(b'\n');
    bytes.extend(footer.as_bytes());
    bytes.push(b'\n');
    bytes
}

fn nominal(year: i32, month: u8, day: u8) -> i128 {
    DateTimeString {
        year,
        month: month - 1,
        day,
        hour: 0,
        minute: 0,
        second: 0,
        millisecond: 0,
        zone: DateTimeZone::Utc,
    }
    .nominal_epoch_milliseconds()
    .unwrap()
}

#[test]
fn legacy_history_uses_type_zero_before_first_transition_and_keeps_the_last_offset() {
    let data = Data {
        times: vec![(i64::from(i32::MIN), 1), (0, 2), (i64::from(i32::MAX), 1)],
        types: vec![(1234, 0, 0), (-1800, 0, 0), (3600, 1, 0)],
        ..Data::default()
    };
    let mut bytes = file(0, &data, "");
    bytes.extend(b"future extension data");
    let zone = TzifTimeZone::parse(&bytes).unwrap();
    assert_eq!(
        zone.possible_offsets().collect::<Vec<_>>(),
        [1234, -1800, 3600]
    );
    for (time, before, after) in [
        (i128::from(i32::MIN) * 1000, 1234, -1800),
        (0, -1800, 3600),
        (i128::from(i32::MAX) * 1000, 3600, -1800),
    ] {
        assert_eq!(zone.offset_at(time - 1), before);
        assert_eq!(zone.offset_at(time), after);
        assert_eq!(zone.offset_at(time + 1), after);
    }
    assert_eq!(zone.offset_at(i128::MIN), 1234);
    assert_eq!(zone.offset_at(i128::MAX), -1800);
}

#[test]
fn authoritative_history_keeps_all_signed_64_bit_transition_seconds() {
    let data = Data {
        times: vec![(i64::MIN, 1), (0, 2), (i64::MAX, 1)],
        types: vec![(1234, 0, 0), (-1800, 0, 0), (3600, 1, 0)],
        ..Data::default()
    };
    for version in *b"234" {
        let zone = TzifTimeZone::parse(&file(version, &data, "")).unwrap();
        assert_eq!(zone.offset_at(i128::MIN), 1234);
        assert_eq!(zone.offset_at(i128::MAX), -1800);
        for (time, before, after) in [
            (i128::from(i64::MIN) * 1000, 1234, -1800),
            (0, -1800, 3600),
            (i128::from(i64::MAX) * 1000, 3600, -1800),
        ] {
            assert_eq!(zone.offset_at(time - 1), before);
            assert_eq!(zone.offset_at(time), after);
        }
    }
}

#[test]
fn negative_epoch_subseconds_keep_the_historical_offset_until_the_exact_transition() {
    // Africa/Abidjan switched from local mean time to GMT at this integral
    // second in 1912. Truncating negative milliseconds toward zero would apply
    // GMT one millisecond too early instead of preserving the -968-second offset.
    let data = Data {
        times: vec![(-1_830_383_032, 1)],
        types: vec![(-968, 0, 0), (0, 0, 0)],
        ..Data::default()
    };
    let zone = TzifTimeZone::parse(&file(b'3', &data, "GMT0")).unwrap();
    assert_eq!(zone.offset_at(-1_830_383_032_001), -968);
    assert_eq!(zone.offset_at(-1_830_383_032_000), 0);
    assert_eq!(zone.offset_at(-1_830_383_031_999), 0);
}

#[test]
fn recurring_footer_preserves_earlier_historical_offsets_and_covers_extreme_future_years() {
    let data = Data {
        times: vec![(0, 1)],
        types: vec![(-17762, 0, 0), (-18000, 0, 0)],
        ..Data::default()
    };
    let zone = TzifTimeZone::parse(&file(b'3', &data, "EST5EDT,M3.2.0,M11.1.0")).unwrap();
    assert_eq!(
        zone.possible_offsets().collect::<Vec<_>>(),
        [-17762, -18000, -18000, -14400]
    );
    assert_eq!(zone.offset_at(-1), -17762);
    assert_eq!(zone.offset_at(0), -18000);
    assert_eq!(zone.offset_at(nominal(2024, 7, 1)), -14400);
    assert_eq!(zone.offset_at(nominal(100000, 7, 1)), -14400);
    assert_eq!(zone.offset_at(nominal(i32::MAX, 7, 1)), -14400);
    assert_eq!(zone.offset_at(-i128::from(MAX_TIME_VALUE)), -17762);
    assert_eq!(zone.offset_at(i128::from(MAX_TIME_VALUE)), -14400);
}

#[test]
fn footer_without_historical_changes_applies_to_all_times_and_can_add_new_offset_types() {
    for version in *b"234" {
        let zone = TzifTimeZone::parse(&file(version, &Data::default(), "ABC-5:45")).unwrap();
        assert_eq!(
            zone.possible_offsets().collect::<Vec<_>>(),
            [0, 20700, 20700]
        );
        for time in [i128::MIN, i128::MAX, -1, 0, 1] {
            assert_eq!(zone.offset_at(time), 20700);
        }
    }
    let mut bytes = file(b'3', &Data::default(), "ABC0");
    bytes.extend(b"opaque future extension\xff");
    assert_eq!(TzifTimeZone::parse(&bytes).unwrap().offset_at(0), 0);
}

#[test]
fn modern_files_ignore_legacy_body_values_and_enforce_footer_version_syntax_and_consistency() {
    let mut bytes = file(b'3', &Data::default(), "ABC0");
    bytes[44..48].copy_from_slice(&i32::MIN.to_be_bytes());
    assert_eq!(TzifTimeZone::parse(&bytes).unwrap().offset_at(0), 0);
    for footer in [
        "ABC0DEF,J1/-1,J365",
        "ABC0DEF,J1/+1,J365",
        "ABC0DEF,J1/25,J365",
    ] {
        assert_eq!(
            TzifTimeZone::parse(&file(b'2', &Data::default(), footer)),
            Err(TzifTimeZoneError::InvalidData)
        );
        assert!(TzifTimeZone::parse(&file(b'3', &Data::default(), footer)).is_ok());
    }
    let mismatch = Data {
        times: vec![(0, 0)],
        types: vec![(3600, 0, 0)],
        ..Data::default()
    };
    assert_eq!(
        TzifTimeZone::parse(&file(b'3', &mismatch, "ABC0")),
        Err(TzifTimeZoneError::InvalidData)
    );
    for footer in [
        "ABC0DEF",
        "ABC0\r",
        "ééé0",
        "ABC0DEF,J0,J365",
        "ABC0DEF,365/0,0/1",
    ] {
        assert!(
            TzifTimeZone::parse(&file(b'3', &Data::default(), footer)).is_err(),
            "{footer}"
        );
    }
}

#[test]
fn type_indices_ordering_flags_designations_and_offsets_are_checked() {
    let invalid = [
        Data {
            types: vec![],
            ..Data::default()
        },
        Data {
            types: vec![(0, 0, 0); 257],
            ..Data::default()
        },
        Data {
            times: vec![(0, 1)],
            ..Data::default()
        },
        Data {
            times: vec![(0, 0), (0, 0)],
            ..Data::default()
        },
        Data {
            times: vec![(1, 0), (0, 0)],
            ..Data::default()
        },
        Data {
            types: vec![(i32::MIN, 0, 0)],
            ..Data::default()
        },
        Data {
            types: vec![(0, 2, 0)],
            ..Data::default()
        },
        Data {
            types: vec![(0, 0, 4)],
            ..Data::default()
        },
        Data {
            names: b"ABC".to_vec(),
            ..Data::default()
        },
        Data {
            names: vec![],
            ..Data::default()
        },
        Data {
            standard: vec![0, 0],
            ..Data::default()
        },
        Data {
            standard: vec![2],
            ..Data::default()
        },
        Data {
            universal: vec![2],
            standard: vec![1],
            ..Data::default()
        },
        Data {
            universal: vec![1],
            ..Data::default()
        },
    ];
    for data in invalid {
        assert_eq!(
            TzifTimeZone::parse(&file(b'3', &data, "")),
            Err(TzifTimeZoneError::InvalidData)
        );
    }
    let valid = Data {
        types: vec![(0, 0, 0); 256],
        times: vec![(0, 255)],
        ..Data::default()
    };
    assert_eq!(
        TzifTimeZone::parse(&file(b'3', &valid, ""))
            .unwrap()
            .offset_at(0),
        0
    );
    // TZif designation encodings are unspecified; offset arithmetic need not decode them.
    let encoded = Data {
        names: vec![255, 0],
        standard: vec![1],
        universal: vec![1],
        ..Data::default()
    };
    assert_eq!(
        TzifTimeZone::parse(&file(b'3', &encoded, ""))
            .unwrap()
            .offset_at(0),
        0
    );
}

#[test]
fn unknown_versions_and_leap_time_scales_remain_distinct_from_bad_data() {
    let mut bytes = file(0, &Data::default(), "");
    bytes[4] = b'5';
    assert_eq!(
        TzifTimeZone::parse(&bytes),
        Err(TzifTimeZoneError::UnsupportedVersion)
    );
    let leaps = Data {
        leaps: vec![(12345, 1)],
        ..Data::default()
    };
    for version in [0, b'2', b'3', b'4'] {
        assert_eq!(
            TzifTimeZone::parse(&file(version, &leaps, "")),
            Err(TzifTimeZoneError::UnsupportedLeapSeconds)
        );
    }
    let mut bytes = file(b'3', &Data::default(), "");
    bytes[54 + 4] = b'2';
    assert_eq!(
        TzifTimeZone::parse(&bytes),
        Err(TzifTimeZoneError::InvalidData)
    );
}

#[test]
fn truncated_and_adversarial_counts_are_rejected_before_allocation_or_indexing() {
    let bytes = file(
        b'3',
        &Data {
            times: vec![(0, 0)],
            ..Data::default()
        },
        "ABC0",
    );
    for length in 0..bytes.len() {
        assert!(
            TzifTimeZone::parse(&bytes[..length]).is_err(),
            "prefix {length}"
        );
    }
    for start in [
        20,
        24,
        28,
        32,
        36,
        40,
        54 + 20,
        54 + 24,
        54 + 28,
        54 + 32,
        54 + 36,
        54 + 40,
    ] {
        let mut altered = bytes.clone();
        altered[start..start + 4].copy_from_slice(&u32::MAX.to_be_bytes());
        assert_eq!(
            TzifTimeZone::parse(&altered),
            Err(TzifTimeZoneError::InvalidData)
        );
    }
    for index in 0..bytes.len() {
        for byte in [0, 1, 255] {
            let mut altered = bytes.clone();
            altered[index] = byte;
            let _ = TzifTimeZone::parse(&altered);
        }
    }
}

#[test]
fn historical_gaps_folds_and_subsecond_boundaries_resolve_before_clipping() {
    let data = Data {
        times: vec![(0, 1), (3600, 0)],
        types: vec![(0, 0, 0), (1800, 1, 0)],
        ..Data::default()
    };
    let zone = TzifTimeZone::parse(&file(b'3', &data, "")).unwrap();
    for (local, utc) in [
        (-1, -1),
        (0, 0),
        (1, 1),
        (1_799_999, 1_799_999),
        (1_800_000, 0),
        (3_600_000, 1_800_000),
        (5_399_999, 3_599_999),
        (5_400_000, 5_400_000),
    ] {
        assert_eq!(zone.resolve_local(local), Ok(utc));
    }
    let data = Data {
        times: vec![(-1_830_383_032, 1)],
        types: vec![(-968, 0, 0), (0, 0, 0)],
        ..Data::default()
    };
    let zone = TzifTimeZone::parse(&file(b'3', &data, "GMT0")).unwrap();
    let start = -1_830_383_032_000_i128 - 968_000;
    assert_eq!(zone.resolve_local(start - 1), Ok(start + 968_000 - 1));
    assert_eq!(zone.resolve_local(start), Ok(start + 968_000));
    assert_eq!(zone.resolve_local(start + 500_000), Ok(start + 1_468_000));
    assert_eq!(zone.resolve_local(start + 968_000), Ok(start + 968_000));
}

#[test]
fn overlapping_local_images_choose_the_latest_epoch_at_the_last_valid_local_time() {
    // A fold just before the gap leaves a later valid local endpoint than the
    // UTC transition that starts the gap. UTC must use that endpoint's offset.
    let data = Data {
        times: vec![(-1, 1), (50, 0)],
        types: vec![(100, 0, 0), (0, 0, 0)],
        ..Data::default()
    };
    let zone = TzifTimeZone::parse(&file(b'3', &data, "")).unwrap();
    assert_eq!(zone.resolve_local(100_000), Ok(0));
    assert_eq!(zone.resolve_local(98_999), Ok(-1001));
    assert_eq!(zone.resolve_local(99_000), Ok(-1000));
    // Two segments end at the same local millisecond. Gap disambiguation uses
    // the latest possible epoch of that endpoint, instead of the fold's first.
    let data = Data {
        times: vec![(0, 1), (10, 2)],
        types: vec![(0, 0, 0), (-10, 0, 0), (100, 0, 0)],
        ..Data::default()
    };
    let zone = TzifTimeZone::parse(&file(b'3', &data, "")).unwrap();
    assert_eq!(zone.resolve_local(-1), Ok(-1));
    assert_eq!(zone.resolve_local(0), Ok(10_000));
    assert_eq!(zone.resolve_local(50_000), Ok(60_000));
}

#[test]
fn recurring_local_endpoints_are_used_only_after_the_historical_cutoff() {
    let data = Data {
        times: vec![(0, 1)],
        types: vec![(-17762, 0, 0), (-18000, 0, 0)],
        ..Data::default()
    };
    let zone = TzifTimeZone::parse(&file(b'3', &data, "EST5EDT,M3.2.0,M11.1.0")).unwrap();
    let local = nominal(1800, 7, 1);
    assert_eq!(zone.resolve_local(local), Ok(local + 17_762_000));
    let local = nominal(2024, 3, 10) + 9_000_000;
    assert_eq!(zone.resolve_local(local), Ok(local + 18_000_000));
    let local = nominal(2024, 11, 3) + 5_400_000;
    assert_eq!(zone.resolve_local(local), Ok(local + 14_400_000));
}
