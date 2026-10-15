//! Independent general-category checks against the unchanged pinned UCD files.
use super::{JsString, RegExpUnicodeClassMatcher, regexp_canonicalize_character};

const LIMIT: usize = 0x110000;
const LEAVES: [&str; 30] = [
    "Cc", "Cf", "Cn", "Co", "Cs", "Ll", "Lm", "Lo", "Lt", "Lu", "Mc", "Me", "Mn", "Nd", "Nl", "No",
    "Pc", "Pd", "Pe", "Pf", "Pi", "Po", "Ps", "Sc", "Sk", "Sm", "So", "Zl", "Zp", "Zs",
];
const CATEGORIES: [&str; 38] = [
    "C", "Cc", "Cf", "Cn", "Co", "Cs", "L", "LC", "Ll", "Lm", "Lo", "Lt", "Lu", "M", "Mc", "Me",
    "Mn", "N", "Nd", "Nl", "No", "P", "Pc", "Pd", "Pe", "Pf", "Pi", "Po", "Ps", "S", "Sc", "Sk",
    "Sm", "So", "Z", "Zl", "Zp", "Zs",
];

fn raw_categories() -> Vec<usize> {
    let mut result = vec![2; LIMIT]; // All omitted code points are Cn.
    let mut pending = None;
    let mut previous = None;
    let mut assigned = 0;
    for line in include_str!("testdata/UnicodeData-18.0.0.txt").lines() {
        let fields: Vec<_> = line.split(';').collect();
        assert_eq!(fields.len(), 15);
        let point = usize::from_str_radix(fields[0], 16).unwrap();
        assert!(point < LIMIT && previous.is_none_or(|p| p < point));
        previous = Some(point);
        let category = LEAVES.iter().position(|&name| name == fields[2]).unwrap();
        assert_ne!(category, 2);
        if let Some(name) = fields[1].strip_suffix(", First>") {
            assert!(pending.replace((point, name, category)).is_none());
        } else if let Some(name) = fields[1].strip_suffix(", Last>") {
            let (start, first_name, first_category) = pending.take().unwrap();
            assert_eq!((name, category), (first_name, first_category));
            assert!(start < point);
            result[start..=point].fill(category);
            assigned += point - start + 1;
        } else {
            assert!(pending.is_none());
            result[point] = category;
            assigned += 1;
        }
    }
    assert!(pending.is_none());
    assert_eq!(assigned, 312389);
    assert!(result[0xd800..0xe000].iter().all(|&category| category == 4));
    assert_eq!(result[0xfdd0], 2);
    assert_eq!(result[0x10ffff], 2);
    result
}

fn raw_folds() -> Vec<(usize, usize)> {
    let mut result = Vec::new();
    for line in include_str!("testdata/CaseFolding-18.0.0.txt").lines() {
        let fields: Vec<_> = line
            .split('#')
            .next()
            .unwrap()
            .split(';')
            .map(str::trim)
            .collect();
        if fields.len() >= 3 && matches!(fields[1], "C" | "S") {
            let source = usize::from_str_radix(fields[0], 16).unwrap();
            let values: Vec<_> = fields[2].split_whitespace().collect();
            assert_eq!(values.len(), 1);
            result.push((source, usize::from_str_radix(values[0], 16).unwrap()));
        }
    }
    assert_eq!(result.len(), 1533);
    assert!(result.windows(2).all(|pair| pair[0].0 < pair[1].0));
    result
}

fn belongs(category: &str, leaf: &str) -> bool {
    match category {
        "LC" => matches!(leaf, "Ll" | "Lt" | "Lu"),
        name if name.len() == 1 => leaf.starts_with(name),
        name => leaf == name,
    }
}

fn compile(pattern: &str, sets: bool, ignore_case: bool) -> RegExpUnicodeClassMatcher {
    let source = JsString::from(pattern);
    if pattern.starts_with('[') {
        RegExpUnicodeClassMatcher::compile_with_flags(&source, sets, ignore_case, false).unwrap()
    } else {
        RegExpUnicodeClassMatcher::compile_binary_property_with_flags(
            &source,
            sets,
            ignore_case,
            false,
        )
        .unwrap()
    }
}

#[test]
fn every_general_category_and_complement_has_exact_reachable_membership() {
    let categories = raw_categories();
    let folds = raw_folds();
    let mut canonical: Vec<_> = (0..LIMIT).collect();
    for &(source, target) in &folds {
        canonical[source] = target;
    }
    for (point, &target) in canonical.iter().enumerate() {
        assert_eq!(canonical[target], target);
        assert_eq!(
            regexp_canonicalize_character(point as u32, true, true),
            target as u32
        );
    }
    let mut checks = 0usize;
    for category in CATEGORIES {
        let raw: Vec<_> = categories
            .iter()
            .map(|&kind| belongs(category, LEAVES[kind]))
            .collect();
        // Independently form the canonical images of the raw set and its
        // complement. These images can overlap in u+i, while v+i complements
        // the canonical positive set. No generated interval data is read here.
        let mut positive = vec![false; LIMIT];
        let mut negative = vec![false; LIMIT];
        for (point, &member) in raw.iter().enumerate() {
            if member {
                positive[canonical[point]] = true;
            } else {
                negative[canonical[point]] = true;
            }
        }
        for sets in [false, true] {
            for ic in [false, true] {
                for (pattern, complemented, inverted) in [
                    (format!(r"\p{{{category}}}"), false, false),
                    (format!(r"\P{{gc={category}}}"), true, false),
                    (
                        format!(r"[\p{{General_Category={category}}}]"),
                        false,
                        false,
                    ),
                    (format!(r"[\P{{{category}}}]"), true, false),
                    (format!(r"[^\p{{gc={category}}}]"), false, true),
                    (format!(r"[^\P{{General_Category={category}}}]"), true, true),
                ] {
                    let plan = compile(&pattern, sets, ic);
                    assert_eq!(plan.ignore_case, ic);
                    assert!(!plan.start_anchor && !plan.end_anchor && !plan.multiline);
                    assert!(plan.start_word.is_none() && plan.end_word.is_none());
                    assert!(
                        plan.ranges
                            .iter()
                            .all(|&(a, b)| a <= b && (b as usize) < LIMIT)
                    );
                    assert!(plan.ranges.windows(2).all(|pair| pair[0].1 + 1 < pair[1].0));
                    let mut actual = vec![plan.inverted; LIMIT];
                    for &(a, b) in plan.ranges.iter() {
                        actual[a as usize..=b as usize].fill(!plan.inverted);
                    }
                    let mut expected = if !ic {
                        raw.iter()
                            .map(|&member| member ^ complemented ^ inverted)
                            .collect::<Vec<_>>()
                    } else if complemented && !sets {
                        negative
                            .iter()
                            .map(|&member| member ^ inverted)
                            .collect::<Vec<_>>()
                    } else {
                        positive
                            .iter()
                            .map(|&member| member ^ complemented ^ inverted)
                            .collect::<Vec<_>>()
                    };
                    if ic {
                        // Mapped sources cannot be observed by matches(), whose
                        // input has already been canonicalized. Removing precisely
                        // those sources proves all reachable values by one complete
                        // set comparison, without repeated binary searches.
                        for &(source, _) in &folds {
                            actual[source] = false;
                            expected[source] = false;
                        }
                    }
                    assert!(actual == expected, "{pattern} sets={sets} i={ic}");
                    checks += LIMIT;
                    // Exercise matches() itself on every compiled interval boundary
                    // and every source/target of the independently parsed folds.
                    let mut points = vec![0, 0x378, 0xd800, 0xdfff, 0x10000, 0x10ffff];
                    for &(a, b) in plan.ranges.iter() {
                        points.extend([a.saturating_sub(1), a, b, (b + 1).min(0x10ffff)]);
                    }
                    for &(source, target) in &folds {
                        points.extend([source as u32, target as u32]);
                    }
                    for point in points {
                        let wanted = expected[if ic {
                            canonical[point as usize]
                        } else {
                            point as usize
                        }];
                        assert_eq!(
                            plan.matches(point),
                            wanted,
                            "{pattern} U+{point:X} sets={sets} i={ic}"
                        );
                    }
                }
            }
        }
    }
    assert_eq!(checks, 1_016_070_144);
}

#[test]
fn general_category_searches_preserve_utf16_boundaries_and_lone_surrogates() {
    let input =
        JsString::from_code_units(vec![0xd800, 0x41, 0xd83d, 0xde00, 0xdc00, 0xe000, 0x378]);
    for sets in [false, true] {
        for ic in [false, true] {
            for (pattern, start, sticky, expected) in [
                (r"\p{Cs}", 0, false, Some(0..1)),
                (r"\p{Cs}", 1, false, Some(4..5)),
                (r"\p{Cs}", 2, true, None),
                (r"\p{So}", 3, true, Some(2..4)),
                (r"\p{L}", 1, true, Some(1..2)),
                (r"\P{L}", 3, true, Some(2..4)),
                (r"\p{Co}", 4, false, Some(5..6)),
                (r"\p{Cn}", 5, false, Some(6..7)),
            ] {
                let plan = compile(pattern, sets, ic);
                assert_eq!(
                    plan.find(&input, start, sticky),
                    expected,
                    "{pattern} start={start} sets={sets} i={ic}"
                );
            }
        }
    }
}

#[test]
fn general_category_find_ranges_snapshot() {
    use std::fmt::Write;
    let mut rows = String::new();
    for category in ["Lu", "Ll", "L", "LC", "M", "Cs", "Co", "Cn"] {
        for pattern in [
            format!(r"\p{{{category}}}"),
            format!(r"\P{{gc={category}}}"),
            format!(r"[\p{{General_Category={category}}}]"),
            format!(r"[\P{{{category}}}]"),
            format!(r"[^\p{{gc={category}}}]"),
            format!(r"[^\P{{General_Category={category}}}]"),
        ] {
            for sets in [false, true] {
                for ic in [false, true] {
                    let plan = compile(&pattern, sets, ic);
                    for units in [
                        vec![0x212a, 0x61, 0xb5, 0x3a3, 0x3c2, 0x345, 0x378, 10],
                        vec![0xd800, 0x41, 0xd83d, 0xde00, 0xdc00, 0xe000, 0x378],
                        vec![],
                    ] {
                        let input = JsString::from_code_units(units);
                        for start in [0, 1, 3] {
                            for sticky in [false, true] {
                                let found = plan.find(&input, start, sticky);
                                writeln!(rows,"{pattern:?} sets={sets} i={ic} input={input:?} start={start} sticky={sticky} {found:?}").unwrap();
                            }
                        }
                    }
                }
            }
        }
    }
    insta::assert_snapshot!(rows);
}
