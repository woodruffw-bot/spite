use super::{JsString, RegExpUnicodeClassMatcher, fixed_property_kind, general_category_kind};
use crate::regexp_script_extensions_data as data;
use std::{collections::BTreeMap, fmt::Write, sync::OnceLock};

const LIMIT: usize = 0x110000;
type Mask = [u64; 3];

struct Override {
    first: u32,
    last: u32,
    members: Mask,
}

struct Script {
    name: String,
    aliases: Vec<String>,
    raw: Vec<(u32, u32)>,
    negative: Vec<(u32, u32)>,
    folded: Vec<(u32, u32)>,
    negative_folded: Vec<(u32, u32)>,
    v_negative: Vec<(u32, u32)>,
}

struct Reference {
    scripts: Vec<Script>,
    owners: Vec<u8>,
    overrides: Vec<Override>,
    folds: Vec<(u32, u32)>,
}

fn contains(ranges: &[(u32, u32)], point: u32) -> bool {
    ranges
        .get(ranges.partition_point(|&(_, last)| last < point))
        .is_some_and(|&(first, _)| first <= point)
}

fn normalized(mut ranges: Vec<(u32, u32)>) -> Vec<(u32, u32)> {
    ranges.sort_unstable();
    let mut result: Vec<(u32, u32)> = Vec::new();
    for (first, last) in ranges {
        assert!(first <= last && last < LIMIT as u32);
        if let Some(previous) = result.last_mut() {
            if first <= previous.1 + 1 {
                previous.1 = previous.1.max(last);
                continue;
            }
        }
        result.push((first, last));
    }
    result
}

fn complement(ranges: &[(u32, u32)]) -> Vec<(u32, u32)> {
    let mut result = Vec::new();
    let mut next = 0;
    for &(first, last) in ranges {
        if next < first {
            result.push((next, first - 1));
        }
        next = last + 1;
    }
    if next < LIMIT as u32 {
        result.push((next, LIMIT as u32 - 1));
    }
    result
}

fn folded(ranges: &[(u32, u32)], folds: &[(u32, u32)]) -> Vec<(u32, u32)> {
    // The interval constructor retains raw members, then adds their canonical
    // targets. Matching canonicalizes the input before consulting these ranges.
    normalized(
        ranges
            .iter()
            .copied()
            .chain(
                folds
                    .iter()
                    .filter(|&&(source, _)| contains(ranges, source))
                    .map(|&(_, target)| (target, target)),
            )
            .collect(),
    )
}

fn mask_has(mask: Mask, index: usize) -> bool {
    mask[index / 64] & (1 << (index % 64)) != 0
}

fn singleton(index: usize) -> Mask {
    let mut result = [0; 3];
    result[index / 64] = 1 << (index % 64);
    result
}

fn membership(owners: &[u8], overrides: &[Override], point: u32) -> Mask {
    let next = overrides.partition_point(|record| record.first <= point);
    if next != 0 && point <= overrides[next - 1].last {
        overrides[next - 1].members
    } else {
        singleton(usize::from(owners[point as usize]))
    }
}

fn reference() -> &'static Reference {
    static REFERENCE: OnceLock<Reference> = OnceLock::new();
    REFERENCE.get_or_init(|| {
        let mut aliases = BTreeMap::new();
        let mut values = BTreeMap::new();
        for line in include_str!("testdata/PropertyValueAliases-18.0.0.txt").lines() {
            let fields: Vec<_> = line
                .split('#')
                .next()
                .unwrap()
                .split(';')
                .map(str::trim)
                .collect();
            if fields.first() != Some(&"sc") {
                continue;
            }
            let canonical = fields[2].to_owned();
            let mut spellings: Vec<_> = fields[1..].iter().map(|s| (*s).to_owned()).collect();
            spellings.sort();
            spellings.dedup();
            for alias in &spellings {
                assert!(!alias.is_empty());
                assert!(aliases.insert(alias.clone(), canonical.clone()).is_none());
            }
            assert!(values.insert(canonical, spellings).is_none());
        }
        assert_eq!(values.len(), 179);
        assert_eq!(aliases.len(), 351);
        let names: Vec<_> = values.keys().cloned().collect();
        let indices: BTreeMap<_, _> = names
            .iter()
            .enumerate()
            .map(|(index, name)| (name.as_str(), index))
            .collect();
        let script_source = include_str!("testdata/Scripts-18.0.0.txt");
        assert!(
            script_source
                .lines()
                .any(|line| line == "# @missing: 0000..10FFFF; Unknown")
        );
        let mut owners = vec![u8::try_from(indices["Unknown"]).unwrap(); LIMIT];
        let mut assigned = vec![false; LIMIT];
        for line in script_source.lines() {
            let text = line.split('#').next().unwrap().trim();
            if text.is_empty() {
                continue;
            }
            let (span, name) = text.split_once(';').unwrap();
            let span = span.trim();
            let (first, last) = span.split_once("..").unwrap_or((span, span));
            let first = usize::from_str_radix(first, 16).unwrap();
            let last = usize::from_str_radix(last, 16).unwrap();
            assert!(first <= last && last < LIMIT);
            assert!(assigned[first..=last].iter().all(|&seen| !seen));
            assigned[first..=last].fill(true);
            owners[first..=last].fill(u8::try_from(indices[name.trim()]).unwrap());
        }
        let extension_source = include_str!("testdata/ScriptExtensions-18.0.0.txt");
        assert!(
            extension_source
                .lines()
                .any(|line| line == "# @missing: 0000..10FFFF; <script>")
        );
        let mut overrides = Vec::new();
        for line in extension_source.lines() {
            let text = line.split('#').next().unwrap().trim();
            if text.is_empty() {
                continue;
            }
            let (span, names) = text.split_once(';').unwrap();
            let span = span.trim();
            let (first, last) = span.split_once("..").unwrap_or((span, span));
            let first = u32::from_str_radix(first, 16).unwrap();
            let last = u32::from_str_radix(last, 16).unwrap();
            assert!(first <= last && last < LIMIT as u32);
            let mut members = [0; 3];
            for alias in names.split_whitespace() {
                let index = indices[aliases[alias].as_str()];
                assert!(!mask_has(members, index));
                members[index / 64] |= 1 << (index % 64);
            }
            assert_ne!(members, [0; 3]);
            overrides.push(Override {
                first,
                last,
                members,
            });
        }
        overrides.sort_unstable_by_key(|record| record.first);
        assert_eq!(overrides.len(), 210);
        assert!(
            overrides
                .windows(2)
                .all(|pair| pair[0].last < pair[1].first)
        );
        assert_eq!(
            overrides
                .iter()
                .map(|record| record.last - record.first + 1)
                .sum::<u32>(),
            741
        );
        // One transition scan derives overlapping sets independently of the
        // generated interval tables. An explicit set REPLACES its fallback.
        let mut raw = vec![Vec::new(); names.len()];
        let mut starts = [0u32; 179];
        let mut previous = [0; 3];
        let mut memberships = 0;
        let mut removed = BTreeMap::new();
        for record in &overrides {
            for point in record.first..=record.last {
                let owner = usize::from(owners[point as usize]);
                if !mask_has(record.members, owner) {
                    *removed.entry(names[owner].as_str()).or_insert(0) += 1;
                }
            }
        }
        assert_eq!(
            removed,
            BTreeMap::from([("Common", 468), ("Inherited", 98)])
        );
        let mut cursor = 0;
        for (point, owner) in owners.iter().copied().enumerate().chain([(LIMIT, 0)]) {
            while cursor < overrides.len() && overrides[cursor].last < point as u32 {
                cursor += 1;
            }
            let current = if point == LIMIT {
                [0; 3]
            } else if cursor < overrides.len() && overrides[cursor].first <= point as u32 {
                overrides[cursor].members
            } else {
                singleton(usize::from(owner))
            };
            if point != LIMIT {
                assert_ne!(current, [0; 3]);
                memberships += current.iter().map(|word| word.count_ones()).sum::<u32>();
            }
            for word in 0..3 {
                let mut changed = previous[word] ^ current[word];
                while changed != 0 {
                    let bit = changed.trailing_zeros() as usize;
                    let index = word * 64 + bit;
                    if current[word] & (1 << bit) != 0 {
                        starts[index] = point as u32;
                    } else {
                        raw[index].push((starts[index], point as u32 - 1));
                    }
                    changed &= changed - 1;
                }
            }
            previous = current;
        }
        assert_eq!(memberships, 1_115_175);
        let mut fallback = vec![Vec::new(); names.len()];
        let mut first = 0;
        for next in 1..=LIMIT {
            if next == LIMIT || owners[next] != owners[first] {
                fallback[usize::from(owners[first])].push((first as u32, (next - 1) as u32));
                first = next;
            }
        }
        assert_eq!(
            raw.iter()
                .zip(&fallback)
                .filter(|(extension, script)| extension == script)
                .count(),
            77
        );
        let mut folds = Vec::new();
        for line in include_str!("testdata/CaseFolding-18.0.0.txt").lines() {
            let fields: Vec<_> = line
                .split('#')
                .next()
                .unwrap()
                .split(';')
                .map(str::trim)
                .collect();
            if fields.len() < 3 || !matches!(fields[1], "C" | "S") {
                continue;
            }
            let target: Vec<_> = fields[2].split_whitespace().collect();
            assert_eq!(target.len(), 1);
            folds.push((
                u32::from_str_radix(fields[0], 16).unwrap(),
                u32::from_str_radix(target[0], 16).unwrap(),
            ));
        }
        assert_eq!(folds.len(), 1533);
        assert!(folds.windows(2).all(|pair| pair[0].0 < pair[1].0));
        assert!(
            folds
                .iter()
                .all(|&(_, target)| !folds.iter().any(|&(source, _)| source == target))
        );
        let mut crossing = Vec::new();
        for &(source, target) in &folds {
            if membership(&owners, &overrides, source) != membership(&owners, &overrides, target) {
                crossing.push((source, target));
            }
        }
        assert_eq!(crossing, [(0xb5, 0x3bc)]);
        let scripts = names
            .iter()
            .enumerate()
            .map(|(index, name)| {
                let raw = raw[index].clone();
                let negative = complement(&raw);
                let positive_folded = folded(&raw, &folds);
                let negative_folded = folded(&negative, &folds);
                let excluded = normalized(
                    positive_folded
                        .iter()
                        .copied()
                        .chain(
                            folds
                                .iter()
                                .filter(|&&(_, target)| contains(&positive_folded, target))
                                .map(|&(source, _)| (source, source)),
                        )
                        .collect(),
                );
                let v_negative = complement(&excluded);
                for &(source, target) in &folds {
                    assert_eq!(contains(&v_negative, source), contains(&v_negative, target));
                }
                Script {
                    name: name.clone(),
                    aliases: values[name].clone(),
                    raw,
                    negative,
                    folded: positive_folded,
                    negative_folded,
                    v_negative,
                }
            })
            .collect();
        Reference {
            scripts,
            owners,
            overrides,
            folds,
        }
    })
}

fn compile_with_work<E>(
    pattern: &str,
    sets: bool,
    ic: bool,
    charge: impl FnMut(usize) -> Result<(), E>,
) -> Result<Option<RegExpUnicodeClassMatcher>, E> {
    let source = JsString::from(pattern);
    if pattern.starts_with('[') {
        RegExpUnicodeClassMatcher::compile_with_flags_and_work(&source, sets, ic, false, charge)
    } else {
        RegExpUnicodeClassMatcher::compile_binary_property_with_flags_and_work(
            &source, sets, ic, false, charge,
        )
    }
}

fn compile(pattern: &str, sets: bool, ic: bool) -> RegExpUnicodeClassMatcher {
    compile_with_work(pattern, sets, ic, |_| Ok::<(), ()>(()))
        .unwrap()
        .unwrap()
}

fn witnesses(script: &Script) -> Vec<u32> {
    let mut points = vec![
        0, 0xb5, 0x300, 0x345, 0x39c, 0x3b9, 0x3bc, 0x30fc, 0xd800, 0xdfff, 0xe000, 0x10ffff,
    ];
    for &(first, last) in &script.raw {
        points.extend([first, last, first + (last - first) / 2]);
        if first != 0 {
            points.push(first - 1);
        }
        if last != LIMIT as u32 - 1 {
            points.push(last + 1);
        }
    }
    points.sort_unstable();
    points.dedup();
    points
}

#[test]
fn script_extensions_original_data_all_generated_sets_and_bounds() {
    let reference = reference();
    assert_eq!(data::FIRST_KIND, 540);
    assert_eq!(data::KIND_COUNT, 898);
    assert_eq!(usize::from(data::KIND_COUNT).div_ceil(64), 15);
    let mut pairs = 0;
    let mut distinct_negative = Vec::new();
    for (index, script) in reference.scripts.iter().enumerate() {
        let kind = 540 + 2 * u16::try_from(index).unwrap();
        assert_eq!(
            data::ranges(kind, false, false),
            Some(script.raw.as_slice()),
            "{}",
            script.name
        );
        for sets in [false, true] {
            for ic in [false, true] {
                let negative = if sets && ic {
                    &script.v_negative
                } else {
                    &script.negative
                };
                assert_eq!(
                    data::ranges(kind + 1, sets, ic),
                    Some(negative.as_slice()),
                    "{} sets={sets} i={ic}",
                    script.name
                );
            }
        }
        let bound = script
            .raw
            .len()
            .max(script.negative.len())
            .max(script.v_negative.len());
        assert_eq!(data::construction_intervals(kind), Some(bound));
        assert_eq!(data::construction_intervals(kind + 1), Some(bound));
        pairs += script.raw.len() + script.negative.len();
        if script.v_negative != script.negative {
            pairs += script.v_negative.len();
            distinct_negative.push(script.name.as_str());
        }
    }
    assert_eq!(pairs, 4604);
    assert_eq!(distinct_negative, ["Common", "Greek"]);
    for kind in [0, 183, 205, 539, 898, u16::MAX] {
        assert!(data::ranges(kind, true, true).is_none());
        assert!(data::construction_intervals(kind).is_none());
    }
    for kind in [540, 590, 630, 672, 882, 897, 898, u16::MAX] {
        assert!(general_category_kind(kind).is_none());
    }
    let unknown = reference
        .scripts
        .iter()
        .find(|script| script.name == "Unknown")
        .unwrap();
    assert_eq!(
        unknown.raw.iter().map(|&(a, b)| b - a + 1).sum::<u32>(),
        941_239
    );
    let empty = reference
        .scripts
        .iter()
        .find(|script| script.name == "Katakana_Or_Hiragana")
        .unwrap();
    assert!(empty.raw.is_empty());
    assert_eq!(empty.negative, [(0, 0x10ffff)]);
}

#[test]
fn script_extensions_every_alias_form_and_flag_has_independent_intervals() {
    let reference = reference();
    let mut keys = 0;
    let mut compilations = 0;
    for (index, script) in reference.scripts.iter().enumerate() {
        let points: Vec<_> = witnesses(script)
            .into_iter()
            .map(|point| {
                let canonical = reference
                    .folds
                    .binary_search_by_key(&point, |&(source, _)| source)
                    .map_or(point, |position| reference.folds[position].1);
                (point, canonical)
            })
            .collect();
        for alias in &script.aliases {
            for prefix in ["Script_Extensions=", "scx="] {
                let key = format!("{prefix}{alias}");
                assert_eq!(
                    fixed_property_kind(&key.encode_utf16().collect::<Vec<_>>()),
                    Some(540 + 2 * u16::try_from(index).unwrap())
                );
                keys += 1;
                for sets in [false, true] {
                    for ic in [false, true] {
                        for (pattern, negative, outer, class) in [
                            (format!(r"\p{{{key}}}"), false, false, false),
                            (format!(r"\P{{{key}}}"), true, false, false),
                            (format!(r"[\p{{{key}}}]"), false, false, true),
                            (format!(r"[\P{{{key}}}]"), true, false, true),
                            (format!(r"[^\p{{{key}}}]"), false, true, true),
                            (format!(r"[^\P{{{key}}}]"), true, true, true),
                        ] {
                            let plan = compile(&pattern, sets, ic);
                            let inverted = outer || (!class && negative && !(ic && !sets));
                            let negative_table = negative && (class || (ic && !sets));
                            let expected = match (negative_table, sets, ic) {
                                (false, _, false) => &script.raw,
                                (false, _, true) => &script.folded,
                                (true, _, false) => &script.negative,
                                (true, false, true) => &script.negative_folded,
                                (true, true, true) => &script.v_negative,
                            };
                            assert_eq!(
                                &*plan.ranges,
                                expected.as_slice(),
                                "{pattern} sets={sets} i={ic}"
                            );
                            assert_eq!(plan.inverted, inverted, "{pattern} sets={sets} i={ic}");
                            for &(point, canonical) in &points {
                                let point_to_check = if ic { canonical } else { point };
                                assert_eq!(
                                    plan.matches(point),
                                    contains(expected, point_to_check) ^ inverted,
                                    "{pattern} U+{point:X} sets={sets} i={ic}"
                                );
                            }
                            compilations += 1;
                        }
                    }
                }
            }
        }
    }
    assert_eq!(keys, 702);
    assert_eq!(compilations, 16_848);
}

#[test]
fn script_extensions_replacement_overlap_and_distinct_fold_preimages() {
    for sets in [false, true] {
        for ic in [false, true] {
            for (pattern, point, expected) in [
                (r"\p{scx=Common}", 0x30fc, false),
                (r"\p{scx=Hira}", 0x30fc, true),
                (r"\p{scx=Kana}", 0x30fc, true),
                (r"\p{sc=Common}", 0x30fc, true),
                (r"\p{scx=Inherited}", 0x300, false),
                (r"\p{scx=Inherited}", 0x345, false),
                (r"\p{scx=Qaai}", 0x345, false),
                (r"\p{scx=Greek}", 0x345, true),
                (r"\p{scx=Qaac}", 0x300, true),
                (r"\p{scx=Latn}", 0x300, true),
                (r"\P{scx=Greek}", 0x3b9, false),
                (r"\P{sc=Greek}", 0x3b9, ic && !sets),
                (r"[\P{scx=Greek}\p{sc=Inherited}]", 0x3b9, ic),
                (r"\P{scx=Common}", 0x3bc, !ic || !sets),
                (r"[^\P{scx=Common}]", 0x3bc, ic && sets),
            ] {
                assert_eq!(
                    compile(pattern, sets, ic).matches(point),
                    expected,
                    "{pattern} U+{point:X} sets={sets} i={ic}"
                );
            }
        }
    }
    let reference = reference();
    for point in [0x378, 0xd800, 0xdfff, 0xe000, 0x10ffff] {
        assert_eq!(
            membership(&reference.owners, &reference.overrides, point)
                .iter()
                .map(|word| word.count_ones())
                .sum::<u32>(),
            1
        );
        assert!(compile(r"\p{scx=Zzzz}", true, true).matches(point));
    }
}

#[test]
fn script_extensions_mask_boundaries_exact_names_and_utf16() {
    let reference = reference();
    let mut pattern = String::from(r"[\p{ASCII}\p{Cs}\p{sc=Grek}");
    let mut points = Vec::new();
    for (name, kind) in [
        ("Adlam", 540),
        ("Buhid", 574),
        ("Canadian_Aboriginal", 576),
        ("Gurung_Khema", 638),
        ("Han", 640),
        ("Lisu", 702),
        ("Lycian", 704),
        ("Old_Hungarian", 766),
        ("Old_Italic", 768),
        ("Soyombo", 830),
        ("Sundanese", 832),
        ("Yi", 894),
        ("Zanabazar_Square", 896),
    ] {
        let index = reference
            .scripts
            .iter()
            .position(|script| script.name == name)
            .unwrap();
        assert_eq!(540 + 2 * index as u16, kind);
        let point = reference
            .owners
            .iter()
            .enumerate()
            .find_map(|(point, &owner)| {
                (usize::from(owner) == index
                    && membership(&reference.owners, &reference.overrides, point as u32)
                        == singleton(index))
                .then_some(point as u32)
            })
            .unwrap();
        points.push(point);
        write!(pattern, r"\p{{scx={name}}}").unwrap();
    }
    pattern.push(']');
    for sets in [false, true] {
        for ic in [false, true] {
            let plan = compile(&pattern, sets, ic);
            for &point in &points {
                assert!(plan.matches(point), "U+{point:X}");
            }
            for point in [0x41, 0x3b1, 0x4e00, 0xd800] {
                assert!(plan.matches(point));
            }
            for point in [0x531, 0x378, 0xe000, 0x1f600] {
                assert!(!plan.matches(point));
            }
        }
    }
    for name in [
        "Greek",
        "Script_extensions=Greek",
        "SCX=Greek",
        "scx=greek",
        "scx=Greek ",
        "scx=Script=Greek",
        "scx=Hiragana_Katakana",
        "scx=Hrkt=",
        "scx=QAAi",
    ] {
        assert!(
            fixed_property_kind(&name.encode_utf16().collect::<Vec<_>>()).is_none(),
            "{name}"
        );
    }
    for tail in [0xd800, 0xdc00, 0x100, 0xffff] {
        let mut name: Vec<_> = "scx=Adlam".encode_utf16().collect();
        name.push(tail);
        assert!(fixed_property_kind(&name).is_none());
    }
    let text = JsString::from_code_units(vec![0x41, 0xd83a, 0xdd00, 0xd800, 0xe000]);
    for sets in [false, true] {
        let adlam = compile(r"\p{scx=Adlm}", sets, true);
        assert_eq!(adlam.find(&text, 2, true), Some(1..3));
        let unknown = compile(r"\p{scx=Zzzz}", sets, true);
        assert_eq!(unknown.find(&text, 2, false), Some(3..4));
        assert_eq!(unknown.find(&text, 2, true), None);
        assert_eq!(
            compile(r"\p{scx=Hrkt}", sets, true).find(&text, 0, false),
            None
        );
        assert_eq!(
            compile(r"\P{scx=Hrkt}", sets, true).find(&text, 2, true),
            Some(1..3)
        );
    }
}

#[test]
fn script_extensions_optional_work_deduplication_and_failures() {
    fn charges(pattern: &str, sets: bool, ic: bool) -> Vec<usize> {
        let mut values = Vec::new();
        compile_with_work(pattern, sets, ic, |value| {
            values.push(value);
            Ok::<(), ()>(())
        })
        .unwrap()
        .unwrap();
        assert_eq!(values[0], pattern.encode_utf16().count());
        values
    }
    for sets in [false, true] {
        for ic in [false, true] {
            for (name, bound) in [("Common", 166), ("Greek", 47), ("Zzzz", 733), ("Hrkt", 1)] {
                assert_eq!(charges(&format!(r"\p{{scx={name}}}"), sets, ic)[1], bound);
                assert_eq!(charges(&format!(r"\P{{scx={name}}}"), sets, ic)[1], bound);
                assert_eq!(
                    charges(&format!(r"[\p{{scx={name}}}]"), sets, ic)[1],
                    bound * 4
                );
                assert_eq!(
                    charges(&format!(r"[\P{{scx={name}}}]"), sets, ic)[1],
                    bound * 4
                );
            }
            for (once, aliases) in [
                (
                    r"[\p{scx=Greek}\P{scx=Greek}]",
                    r"\p{Script_Extensions=Greek}\p{scx=Grek}\P{Script_Extensions=Grek}\P{scx=Greek}",
                ),
                (
                    r"[\p{scx=Qaac}\P{scx=Qaac}]",
                    r"\p{scx=Coptic}\p{Script_Extensions=Qaac}\P{scx=Copt}",
                ),
                (
                    r"[\p{scx=Hrkt}\P{scx=Hrkt}]",
                    r"\p{scx=Katakana_Or_Hiragana}\P{Script_Extensions=Hrkt}",
                ),
                (
                    r"[\p{scx=Zanb}\P{scx=Zanb}]",
                    r"\p{Script_Extensions=Zanabazar_Square}\P{scx=Zanb}",
                ),
            ] {
                let repeated = format!("[{}]", aliases.repeat(1000));
                assert_eq!(
                    &charges(once, sets, ic)[1..],
                    &charges(&repeated, sets, ic)[1..]
                );
            }
            for pattern in [
                r"\p{scx=Common}",
                r"\P{scx=Greek}",
                r"[\P{scx=Common}\p{sc=Inherited}\P{scx=Greek}\p{Cs}\p{scx=Zanb}]",
                r"\p{scx=Hrkt}",
                r"\P{scx=Hrkt}",
            ] {
                let count = charges(pattern, sets, ic).len();
                for failure in 1..=count {
                    let mut calls = 0;
                    let result = compile_with_work(pattern, sets, ic, |_| {
                        calls += 1;
                        if calls == failure {
                            Err(failure)
                        } else {
                            Ok(())
                        }
                    });
                    assert!(matches!(result, Err(error) if error == failure));
                    assert_eq!(calls, failure);
                }
            }
        }
    }
}

#[test]
fn script_extensions_selected_find_ranges_snapshot() {
    let mut rows = String::new();
    for (pattern, units) in [
        (r"\P{scx=Common}", vec![0xb5, 0x39c, 0x3bc, 0x345, 0x3b9]),
        (r"[^\P{scx=Common}]", vec![0xb5, 0x39c, 0x3bc, 0x345, 0x3b9]),
        (r"\P{scx=Greek}", vec![0x345, 0x3b9, 0xb5, 0x3bc]),
        (r"\P{sc=Greek}", vec![0x345, 0x3b9, 0xb5, 0x3bc]),
        (
            r"[\p{scx=Hira}\p{scx=Kana}]",
            vec![0x30fc, 0x41, 0x3042, 0x30a2],
        ),
        (r"\p{scx=Inherited}", vec![0x300, 0x345, 0x34f, 0x41]),
        (r"\p{scx=Zzzz}", vec![0xd83a, 0xdd00, 0xd800, 0xe000]),
        (r"\P{scx=Hrkt}", vec![0xd83a, 0xdd00, 0xd800, 0xe000]),
    ] {
        let input = JsString::from_code_units(units);
        for (sets, ic) in [(false, false), (false, true), (true, true)] {
            let plan = compile(pattern, sets, ic);
            for (start, sticky) in [(0, false), (1, true), (2, true)] {
                writeln!(rows, "{pattern:?} sets={sets} i={ic} input={input:?} start={start} sticky={sticky} {:?}", plan.find(&input, start, sticky)).unwrap();
            }
        }
    }
    insta::assert_snapshot!(rows);
}
