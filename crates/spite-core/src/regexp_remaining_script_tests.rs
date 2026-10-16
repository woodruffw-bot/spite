use super::{JsString, RegExpUnicodeClassMatcher, fixed_property_kind, general_category_kind};
use crate::regexp_script_data;
use std::{collections::BTreeMap, fmt::Write, sync::OnceLock};

const LIMIT: usize = 0x110000;

struct ScriptReference {
    name: String,
    aliases: Vec<String>,
    kind: u16,
    positive: Vec<(u32, u32)>,
    negative: Vec<(u32, u32)>,
}

struct Reference {
    scripts: Vec<ScriptReference>,
    owners: Vec<usize>,
    folds: Vec<(u32, u32)>,
}

fn legacy_kind(name: &str) -> Option<u16> {
    Some(match name {
        "Han" => 26,
        "Latin" => 184,
        "Greek" => 186,
        "Cyrillic" => 188,
        "Arabic" => 190,
        "Hebrew" => 192,
        "Common" => 194,
        "Inherited" => 196,
        "Devanagari" => 198,
        "Bengali" => 200,
        "Gurmukhi" => 202,
        "Gujarati" => 204,
        _ => return None,
    })
}

fn reference() -> &'static Reference {
    static REFERENCE: OnceLock<Reference> = OnceLock::new();
    REFERENCE.get_or_init(|| {
        // Read original aliases independently of both generated registries.
        let mut names = BTreeMap::new();
        for line in include_str!("testdata/PropertyValueAliases-18.0.0.txt").lines() {
            let mut fields = line.split('#').next().unwrap().split(';').map(str::trim);
            if fields.next() != Some("sc") {
                continue;
            }
            let short = fields.next().unwrap();
            let canonical = fields.next().unwrap();
            let mut aliases: Vec<_> = [short, canonical]
                .into_iter()
                .chain(fields)
                .map(str::to_owned)
                .collect();
            aliases.sort();
            aliases.dedup();
            assert!(aliases.iter().all(|alias| !alias.is_empty()));
            assert!(names.insert(canonical.to_owned(), aliases).is_none());
        }
        assert_eq!(names.len(), 179);
        let mut next_kind = 206;
        let mut scripts: Vec<_> = names
            .into_iter()
            .map(|(name, aliases)| {
                let kind = legacy_kind(&name).unwrap_or_else(|| {
                    let kind = next_kind;
                    next_kind += 2;
                    kind
                });
                ScriptReference {
                    name,
                    aliases,
                    kind,
                    positive: Vec::new(),
                    negative: Vec::new(),
                }
            })
            .collect();
        assert_eq!(next_kind, 540);
        let indices: BTreeMap<_, _> = scripts
            .iter()
            .enumerate()
            .map(|(index, script)| (script.name.as_str(), index))
            .collect();
        let missing = indices["Unknown"];
        let source = include_str!("testdata/Scripts-18.0.0.txt");
        assert!(
            source
                .lines()
                .any(|line| line == "# @missing: 0000..10FFFF; Unknown")
        );
        let mut owners = vec![missing; LIMIT];
        let mut explicit = vec![false; LIMIT];
        for line in source.lines() {
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
            assert!(explicit[first..=last].iter().all(|&seen| !seen));
            explicit[first..=last].fill(true);
            owners[first..=last].fill(indices[name.trim()]);
        }
        // Derive all normalized lists in a single independent partition scan.
        let mut first = 0;
        for next in 1..=LIMIT {
            if next == LIMIT || owners[next] != owners[first] {
                scripts[owners[first]]
                    .positive
                    .push((first as u32, (next - 1) as u32));
                first = next;
            }
        }
        for script in &mut scripts {
            let mut cursor = 0;
            for &(first, last) in &script.positive {
                if cursor < first {
                    script.negative.push((cursor, first - 1));
                }
                cursor = last + 1;
            }
            if cursor < LIMIT as u32 {
                script.negative.push((cursor, LIMIT as u32 - 1));
            }
        }
        let mut folds = Vec::new();
        for line in include_str!("testdata/CaseFolding-18.0.0.txt").lines() {
            let mut fields = line.split('#').next().unwrap().split(';').map(str::trim);
            let source = fields.next().unwrap();
            if !matches!(fields.next(), Some("C" | "S")) {
                continue;
            }
            let mut targets = fields.next().unwrap().split_whitespace();
            let target = targets.next().unwrap();
            assert!(targets.next().is_none());
            folds.push((
                u32::from_str_radix(source, 16).unwrap(),
                u32::from_str_radix(target, 16).unwrap(),
            ));
        }
        assert_eq!(folds.len(), 1533);
        Reference {
            scripts,
            owners,
            folds,
        }
    })
}

fn compile(pattern: &str, sets: bool, ic: bool) -> RegExpUnicodeClassMatcher {
    compile_with_work(pattern, sets, ic, |_| Ok::<(), ()>(()))
        .unwrap()
        .unwrap()
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

fn witnesses(script: &ScriptReference) -> Vec<u32> {
    let mut points = vec![0, 0x10ffff, 0xd800, 0xdfff, 0xe000, 0x10fffd];
    for &(first, last) in &script.positive {
        points.extend([first, last, first + (last - first) / 2]);
        if first != 0 {
            points.push(first - 1);
        }
        if last != 0x10ffff {
            points.push(last + 1);
        }
    }
    points.sort_unstable();
    points.dedup();
    points
}

#[test]
fn remaining_scripts_independent_partition_aliases_and_fold_invariance() {
    let reference = reference();
    assert_eq!(reference.scripts.len(), 179);
    let mut aliases: BTreeMap<&str, &str> = BTreeMap::new();
    let mut new_sets = 0;
    let mut new_pairs = 0;
    let mut all_members = 0;
    let mut empty = Vec::new();
    for script in &reference.scripts {
        let members: u32 = script.positive.iter().map(|&(a, b)| b - a + 1).sum();
        let nonmembers: u32 = script.negative.iter().map(|&(a, b)| b - a + 1).sum();
        assert_eq!(members + nonmembers, LIMIT as u32, "{}", script.name);
        all_members += members;
        if members == 0 {
            empty.push(script.name.as_str());
        }
        for alias in &script.aliases {
            assert!(
                aliases
                    .insert(alias.as_str(), script.name.as_str())
                    .is_none()
            );
            for prefix in ["Script=", "sc="] {
                let key: Vec<_> = format!("{prefix}{alias}").encode_utf16().collect();
                assert_eq!(fixed_property_kind(&key), Some(script.kind));
                assert_eq!(
                    regexp_script_data::kind(&key),
                    legacy_kind(&script.name).is_none().then_some(script.kind)
                );
            }
        }
        if script.kind >= 206 {
            new_sets += 1;
            new_pairs += script.positive.len() + script.negative.len();
            assert_eq!(
                regexp_script_data::ranges(script.kind),
                Some(script.positive.as_slice())
            );
            assert_eq!(
                regexp_script_data::ranges(script.kind + 1),
                Some(script.negative.as_slice())
            );
            let bound = script.positive.len().max(script.negative.len());
            assert_eq!(
                regexp_script_data::construction_intervals(script.kind),
                Some(bound)
            );
            assert_eq!(
                regexp_script_data::construction_intervals(script.kind + 1),
                Some(bound)
            );
        }
    }
    assert_eq!(aliases.len(), 351);
    assert_eq!(new_sets, 167);
    assert_eq!(new_pairs, 2764);
    assert_eq!(all_members, LIMIT as u32);
    assert_eq!(empty, ["Katakana_Or_Hiragana"]);
    assert_eq!(aliases["Qaai"], "Inherited");
    assert_eq!(aliases["Qaac"], "Coptic");
    let unknown = reference
        .scripts
        .iter()
        .position(|script| script.name == "Unknown")
        .unwrap();
    assert_eq!(reference.scripts[unknown].positive.len(), 733);
    assert_eq!(
        reference
            .owners
            .iter()
            .filter(|&&owner| owner == unknown)
            .count(),
        941_239
    );
    for point in [0x378, 0xd800, 0xdfff, 0xe000, 0xf0000, 0x10fffd, 0x10ffff] {
        assert_eq!(reference.owners[point], unknown);
    }
    let mut crossing = Vec::new();
    for &(source, target) in &reference.folds {
        let source_script = &reference.scripts[reference.owners[source as usize]];
        let target_script = &reference.scripts[reference.owners[target as usize]];
        if source_script.name != target_script.name {
            crossing.push((
                source,
                target,
                source_script.name.as_str(),
                target_script.name.as_str(),
            ));
            assert!(legacy_kind(&source_script.name).is_some());
            assert!(legacy_kind(&target_script.name).is_some());
        }
    }
    assert_eq!(
        crossing,
        [
            (0xb5, 0x3bc, "Common", "Greek"),
            (0x345, 0x3b9, "Inherited", "Greek")
        ]
    );
}

#[test]
fn remaining_scripts_every_alias_form_and_flag_matches_exact_intervals() {
    let reference = reference();
    let mut compilations = 0;
    for (owner, script) in reference
        .scripts
        .iter()
        .enumerate()
        .filter(|(_, script)| script.kind >= 206)
    {
        let points = witnesses(script);
        for alias in &script.aliases {
            for prefix in ["Script=", "sc="] {
                let name = format!("{prefix}{alias}");
                for sets in [false, true] {
                    for ic in [false, true] {
                        for (pattern, complement, outer, class) in [
                            (format!(r"\p{{{name}}}"), false, false, false),
                            (format!(r"\P{{{name}}}"), true, false, false),
                            (format!(r"[\p{{{name}}}]"), false, false, true),
                            (format!(r"[\P{{{name}}}]"), true, false, true),
                            (format!(r"[^\p{{{name}}}]"), false, true, true),
                            (format!(r"[^\P{{{name}}}]"), true, true, true),
                        ] {
                            let plan = compile(&pattern, sets, ic);
                            // Standalone P usually inverts the positive table;
                            // u+i complements it before the shared fold pass.
                            let inverted = outer || (!class && complement && !(ic && !sets));
                            let negative_table = complement && (class || (ic && !sets));
                            let expected = if negative_table {
                                &script.negative
                            } else {
                                &script.positive
                            };
                            assert_eq!(
                                &*plan.ranges,
                                expected.as_slice(),
                                "{pattern} sets={sets} i={ic}"
                            );
                            assert_eq!(plan.inverted, inverted, "{pattern} sets={sets} i={ic}");
                            for &point in &points {
                                let member = reference.owners[point as usize] == owner;
                                assert_eq!(
                                    plan.matches(point),
                                    member ^ complement ^ outer,
                                    "{pattern} U+{point:X} sets={sets} i={ic}"
                                );
                            }
                            for &(source, target) in &reference.folds {
                                if reference.owners[source as usize] == owner {
                                    assert_eq!(plan.matches(source), !(complement ^ outer));
                                    assert_eq!(plan.matches(target), !(complement ^ outer));
                                }
                            }
                            compilations += 1;
                        }
                    }
                }
            }
        }
    }
    assert_eq!(compilations, 15_648);
}

#[test]
fn remaining_scripts_wide_mask_preserves_each_set_and_legacy_preimages() {
    let reference = reference();
    let names = [
        ("Adlam", 206),
        ("Coptic", 252),
        ("Cuneiform", 254),
        ("Cypriot", 256),
        ("Kawi", 318),
        ("Kayah_Li", 320),
        ("Multani", 382),
        ("Myanmar", 384),
        ("Rejang", 446),
        ("Runic", 448),
        ("Tifinagh", 510),
        ("Tirhuta", 512),
        ("Zanabazar_Square", 538),
    ];
    let mut union = String::from(r"[\p{ASCII}\p{Cs}\p{sc=Hani}");
    let mut points = Vec::new();
    for (name, kind) in names {
        let script = reference
            .scripts
            .iter()
            .find(|script| script.name == name)
            .unwrap();
        assert_eq!(script.kind, kind);
        points.push(script.positive[0].0);
        write!(union, r"\p{{sc={name}}}").unwrap();
    }
    union.push(']');
    for sets in [false, true] {
        for ic in [false, true] {
            let plan = compile(&union, sets, ic);
            for &point in &points {
                assert!(
                    plan.matches(point),
                    "{union} U+{point:X} sets={sets} i={ic}"
                );
            }
            for point in [0x41, 0x4e00, 0xd800] {
                assert!(plan.matches(point));
            }
            for point in [0x531, 0x378, 0xe000, 0x1f600] {
                assert!(!plan.matches(point));
            }
            for ((name, _), &point) in names.into_iter().zip(&points) {
                let negative = compile(&format!(r"[\P{{sc={name}}}\p{{sc=Latn}}]"), sets, ic);
                assert!(!negative.matches(point));
                assert!(negative.matches(0));
                assert!(negative.matches(0x41));
                let inverted = compile(&format!(r"[^\P{{sc={name}}}\p{{sc=Latn}}]"), sets, ic);
                assert!(inverted.matches(point));
                assert!(!inverted.matches(0));
                assert!(!inverted.matches(0x41));
            }
            for (name, source, target) in [("Common", 0xb5, 0x3bc), ("Inherited", 0x345, 0x3b9)] {
                let positive = compile(&format!(r"[\p{{sc={name}}}\p{{sc=Cprt}}]"), sets, ic);
                assert!(positive.matches(source));
                assert_eq!(positive.matches(target), ic);
                let negative = compile(&format!(r"[\P{{sc={name}}}\p{{sc=Cprt}}]"), sets, ic);
                assert_eq!(negative.matches(source), ic && !sets);
                assert_eq!(negative.matches(target), !ic || !sets);
            }
        }
    }
}

#[test]
fn remaining_scripts_work_deduplication_bounds_and_failure_propagation() {
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
            for (pattern, bound) in [
                (r"\p{sc=Zzzz}", 733),
                (r"\P{sc=Zzzz}", 733),
                (r"[\p{sc=Zzzz}]", 2932),
                (r"[\P{sc=Zzzz}]", 2932),
                (r"\p{sc=Hrkt}", 1),
                (r"\P{sc=Hrkt}", 1),
                (r"[\p{sc=Hrkt}]", 4),
                (r"[\P{sc=Hrkt}]", 4),
            ] {
                assert_eq!(charges(pattern, sets, ic)[1], bound);
            }
            for (once, aliases) in [
                (
                    r"[\p{sc=Cprt}\P{sc=Cprt}]",
                    r"\p{Script=Cypriot}\p{sc=Cprt}\P{Script=Cypriot}\P{sc=Cprt}",
                ),
                (
                    r"[\p{sc=Copt}\P{sc=Copt}]",
                    r"\p{Script=Coptic}\p{sc=Qaac}\P{Script=Copt}\P{sc=Qaac}",
                ),
                (
                    r"[\p{sc=Zzzz}\P{sc=Zzzz}]",
                    r"\p{Script=Unknown}\p{sc=Zzzz}\P{Script=Unknown}\P{sc=Zzzz}",
                ),
                (
                    r"[\p{sc=Hrkt}\P{sc=Hrkt}]",
                    r"\p{Script=Katakana_Or_Hiragana}\P{sc=Hrkt}",
                ),
                (
                    r"[\p{sc=Zanb}\P{sc=Zanb}]",
                    r"\p{Script=Zanabazar_Square}\P{sc=Zanb}",
                ),
            ] {
                let repeated = format!("[{}]", aliases.repeat(1000));
                assert_eq!(
                    &charges(once, sets, ic)[1..],
                    &charges(&repeated, sets, ic)[1..]
                );
            }
            for pattern in [
                r"\p{sc=Zanb}",
                r"\P{sc=Zanb}",
                r"\p{sc=Zzzz}",
                r"\P{sc=Zzzz}",
                r"\p{sc=Hrkt}",
                r"\P{sc=Hrkt}",
                r"[\p{sc=Latn}\p{Cs}\p{sc=Cprt}\P{sc=Zanb}\p{sc=Zzzz}]",
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
fn remaining_scripts_exact_names_bounds_and_utf16_find() {
    for (kind, expected) in [
        (107, None),
        (108, Some(108)),
        (183, Some(183)),
        (184, None),
        (256, None),
        (539, None),
    ] {
        assert_eq!(general_category_kind(kind), expected);
    }
    for kind in [0, 183, 205, 540, u16::MAX] {
        assert!(regexp_script_data::ranges(kind).is_none());
        assert!(regexp_script_data::construction_intervals(kind).is_none());
    }
    assert_eq!(regexp_script_data::FIRST_KIND, 206);
    assert_eq!(regexp_script_data::KIND_COUNT, 540);
    assert_eq!(usize::from(regexp_script_data::KIND_COUNT).div_ceil(64), 9);
    for name in [
        "Adlam",
        "Adlm",
        "gc=Adlam",
        "SCRIPT=Adlam",
        "Sc=Adlam",
        "sc=adlam",
        "Script=ADLAM",
        "sc=Ad_lam",
        "sc=Script=Adlam",
        "scx=Adlam",
        "Script_Extensions=Adlam",
        "sc=Hrkt ",
        "sc=QAAc",
        "sc=Qaai ",
        "sc=Zzzz=",
    ] {
        let units: Vec<_> = name.encode_utf16().collect();
        assert!(fixed_property_kind(&units).is_none(), "{name}");
        for pattern in [format!(r"\p{{{name}}}"), format!(r"[\p{{{name}}}]")] {
            assert!(
                compile_with_work(&pattern, false, false, |_| Ok::<(), ()>(()))
                    .unwrap()
                    .is_none()
            );
        }
    }
    for tail in [0xd800, 0xdc00, 0x100, 0xffff] {
        let mut name: Vec<_> = "sc=Adlam".encode_utf16().collect();
        name.push(tail);
        assert!(fixed_property_kind(&name).is_none());
    }
    // Adlam is supplementary, Unknown includes lone surrogates, and Hrkt is empty.
    let input = JsString::from_code_units(vec![0x41, 0xd83a, 0xdd00, 0xd800, 0xe000]);
    for sets in [false, true] {
        for ic in [false, true] {
            let adlam = compile(r"\p{sc=Adlm}", sets, ic);
            assert_eq!(adlam.find(&input, 0, false), Some(1..3));
            assert_eq!(adlam.find(&input, 2, true), Some(1..3));
            assert_eq!(adlam.find(&input, 3, true), None);
            let unknown = compile(r"\p{sc=Zzzz}", sets, ic);
            assert_eq!(unknown.find(&input, 0, false), Some(3..4));
            assert_eq!(unknown.find(&input, 2, true), None);
            assert_eq!(unknown.find(&input, 4, true), Some(4..5));
            let empty = compile(r"\p{sc=Hrkt}", sets, ic);
            assert_eq!(empty.find(&input, 0, false), None);
            let universal = compile(r"\P{sc=Hrkt}", sets, ic);
            assert_eq!(universal.find(&input, 2, true), Some(1..3));
            assert_eq!(universal.find(&input, 3, true), Some(3..4));
        }
    }
}

#[test]
fn remaining_scripts_selected_find_ranges_snapshot() {
    let mut rows = String::new();
    for name in [
        "sc=Adlm", "sc=Cprt", "sc=Qaac", "sc=Zanb", "sc=Zzzz", "sc=Hrkt", "sc=Qaai", "sc=Zyyy",
    ] {
        for sets in [false, true] {
            for ic in [false, true] {
                for pattern in [format!(r"\p{{{name}}}"), format!(r"[^\p{{{name}}}]")] {
                    let plan = compile(&pattern, sets, ic);
                    for units in [
                        vec![0xb5, 0x3bc, 0x345, 0x3b9],
                        vec![0xd83a, 0xdd00, 0xd800, 0xe000],
                    ] {
                        let input = JsString::from_code_units(units);
                        for (start, sticky) in [(0, false), (1, true), (2, true)] {
                            writeln!(rows, "{pattern:?} sets={sets} i={ic} input={input:?} start={start} sticky={sticky} {:?}", plan.find(&input, start, sticky)).unwrap();
                        }
                    }
                }
            }
        }
    }
    insta::assert_snapshot!(rows);
}
