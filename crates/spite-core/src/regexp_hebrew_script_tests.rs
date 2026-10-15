use super::{JsString, RegExpUnicodeClassMatcher};

fn compile(pattern: &str, sets: bool, ic: bool) -> RegExpUnicodeClassMatcher {
    let source = JsString::from(pattern);
    if pattern.starts_with('[') {
        RegExpUnicodeClassMatcher::compile_with_flags(&source, sets, ic, false).unwrap()
    } else {
        RegExpUnicodeClassMatcher::compile_binary_property_with_flags(&source, sets, ic, false)
            .unwrap()
    }
}

#[test]
fn hebrew_script_complete_membership_aliases_and_complements() {
    let mut membership = vec![false; 0x110000];
    let mut records = 0;
    let mut previous = None;
    for line in include_str!("testdata/Scripts-18.0.0.txt").lines() {
        let fields: Vec<_> = line
            .split('#')
            .next()
            .unwrap()
            .split(';')
            .map(str::trim)
            .collect();
        if fields.len() < 2 || fields[1] != "Hebrew" {
            continue;
        }
        let span: Vec<_> = fields[0]
            .split("..")
            .map(|s| usize::from_str_radix(s, 16).unwrap())
            .collect();
        let a = span[0];
        let b = *span.last().unwrap();
        assert!(a <= b && b < 0x110000 && previous.is_none_or(|p| p < a));
        previous = Some(b);
        membership[a..=b].fill(true);
        records += 1;
    }
    assert_eq!(records, 22);
    assert_eq!(membership.iter().filter(|&&m| m).count(), 136);
    let mut folds = 0;
    for line in include_str!("testdata/CaseFolding-18.0.0.txt").lines() {
        let fields: Vec<_> = line
            .split('#')
            .next()
            .unwrap()
            .split(';')
            .map(str::trim)
            .collect();
        if fields.len() >= 3 && matches!(fields[1], "C" | "S") {
            let a = usize::from_str_radix(fields[0], 16).unwrap();
            let targets: Vec<_> = fields[2].split_whitespace().collect();
            assert_eq!(targets.len(), 1);
            let b = usize::from_str_radix(targets[0], 16).unwrap();
            assert_eq!(membership[a], membership[b]);
            folds += 1;
        }
    }
    assert_eq!(folds, 1533);
    let mut checks = 0;
    for name in ["Script=Hebrew", "Script=Hebr", "sc=Hebrew", "sc=Hebr"] {
        for sets in [false, true] {
            for ic in [false, true] {
                for (pattern, negated) in [
                    (format!(r"\p{{{name}}}"), false),
                    (format!(r"\P{{{name}}}"), true),
                    (format!(r"[\p{{{name}}}]"), false),
                    (format!(r"[\P{{{name}}}]"), true),
                    (format!(r"[^\p{{{name}}}]"), true),
                    (format!(r"[^\P{{{name}}}]"), false),
                ] {
                    let plan = compile(&pattern, sets, ic);
                    for (point, &member) in membership.iter().enumerate() {
                        assert_eq!(
                            plan.matches(point as u32),
                            member ^ negated,
                            "{pattern} U+{point:X} sets={sets} i={ic}"
                        );
                        checks += 1;
                    }
                }
            }
        }
    }
    assert_eq!(checks, 106_954_752);
}

#[test]
fn hebrew_script_high_mask_bits_coexist_with_general_categories_and_older_properties() {
    for sets in [false, true] {
        for ic in [false, true] {
            for (pattern, points) in [
                (
                    r"[\p{ASCII}\p{Cs}\p{sc=Hebr}]",
                    [
                        (0x41, true),
                        (0xd800, true),
                        (0x5d0, true),
                        (0xfb1d, true),
                        (0x3a3, false),
                        (0x1df95, false),
                    ],
                ),
                (
                    r"[^\P{Script=Hebrew}\p{sc=Arab}]",
                    [
                        (0x41, false),
                        (0x5d0, true),
                        (0x5d1, true),
                        (0xb5, false),
                        (0xfb1d, true),
                        (0xd800, false),
                    ],
                ),
                (
                    r"[\P{sc=Hebr}\p{sc=Latn}]",
                    [
                        (0x41, true),
                        (0x5d0, false),
                        (0x5d1, false),
                        (0xd800, true),
                        (0xfb1d, false),
                        (0x3b9, true),
                    ],
                ),
            ] {
                let plan = compile(pattern, sets, ic);
                for (point, wanted) in points {
                    assert_eq!(
                        plan.matches(point),
                        wanted,
                        "{pattern} U+{point:X} sets={sets} i={ic}"
                    );
                }
            }
            let source = JsString::from(
                format!(
                    "[{}]",
                    r"\p{sc=Hebrew}\P{Script=Hebr}\p{gc=Cs}\P{ASCII}".repeat(10000)
                )
                .as_str(),
            );
            let plan =
                RegExpUnicodeClassMatcher::compile_with_flags(&source, sets, ic, false).unwrap();
            for point in [0, 0x41, 0x378, 0xd800, 0xe000, 0xfb1d, 0x10ffff] {
                assert!(plan.matches(point));
            }
        }
    }
}

#[test]
fn hebrew_script_work_utf16_and_exact_namespace_contracts() {
    for pattern in [r"\P{Script=Hebrew}", r"[\P{sc=Hebr}]"] {
        let source = JsString::from(pattern);
        let class = pattern.starts_with('[');
        let mut calls = 0;
        let charge = |_| {
            calls += 1;
            Ok::<(), ()>(())
        };
        let plan = if class {
            RegExpUnicodeClassMatcher::compile_with_flags_and_work(
                &source, true, true, false, charge,
            )
        } else {
            RegExpUnicodeClassMatcher::compile_binary_property_with_flags_and_work(
                &source, true, true, false, charge,
            )
        }
        .unwrap()
        .unwrap();
        assert!(!plan.matches(0x5d0));
        assert!(!plan.matches(0xfb1d));
        assert!(plan.matches(0xd800));
        for fail in 1..=calls {
            let mut count = 0;
            let charge = |_| {
                count += 1;
                if count == fail { Err(()) } else { Ok(()) }
            };
            let result = if class {
                RegExpUnicodeClassMatcher::compile_with_flags_and_work(
                    &source, true, true, false, charge,
                )
            } else {
                RegExpUnicodeClassMatcher::compile_binary_property_with_flags_and_work(
                    &source, true, true, false, charge,
                )
            };
            assert!(result.is_err());
        }
    }
    for pattern in [
        r"\p{Hebrew}",
        r"\p{Hebr}",
        r"\p{gc=Hebrew}",
        r"\p{SCRIPT=Hebrew}",
        r"\p{Script=hebrew}",
        r"\p{sc=Script=Hebrew}",
        r"\p{Script_Extensions=Hebrew}",
    ] {
        assert!(
            RegExpUnicodeClassMatcher::compile_binary_property_with_flags(
                &JsString::from(pattern),
                false,
                false,
                false
            )
            .is_none(),
            "{pattern}"
        );
    }
    let input = JsString::from_code_units(vec![0xd83d, 0xde00, 0xd800, 0x5d0, 0xfb1d]);
    for sets in [false, true] {
        for ic in [false, true] {
            let positive = compile(r"\p{sc=Hebrew}", sets, ic);
            let negative = compile(r"\P{sc=Hebrew}", sets, ic);
            assert_eq!(positive.find(&input, 1, false), Some(3..4));
            assert_eq!(positive.find(&input, 1, true), None);
            assert_eq!(positive.find(&input, 4, true), Some(4..5));
            assert_eq!(negative.find(&input, 1, true), Some(0..2));
            assert_eq!(negative.find(&input, 2, true), Some(2..3));
        }
    }
}

#[test]
fn hebrew_script_find_ranges_snapshot() {
    use std::fmt::Write;
    let mut rows = String::new();
    for name in ["Script=Hebrew", "Script=Hebr", "sc=Hebrew", "sc=Hebr"] {
        for pattern in [
            format!(r"\p{{{name}}}"),
            format!(r"\P{{{name}}}"),
            format!(r"[\p{{{name}}}]"),
            format!(r"[\P{{{name}}}]"),
            format!(r"[^\p{{{name}}}]"),
            format!(r"[^\P{{{name}}}]"),
        ] {
            for sets in [false, true] {
                for ic in [false, true] {
                    let plan = compile(&pattern, sets, ic);
                    for units in [
                        vec![0x5d0, 0x5d1, 0xfb1d, 0x3a3, 0x41, 10],
                        vec![0xd800, 0x5d0, 0xd83d, 0xde00, 0xdc00, 0x3a3],
                        vec![],
                    ] {
                        let input = JsString::from_code_units(units);
                        for start in [0, 1, 3] {
                            for sticky in [false, true] {
                                let found = plan.find(&input, start, sticky);
                                writeln!(rows, "{pattern:?} sets={sets} i={ic} input={input:?} start={start} sticky={sticky} {found:?}").unwrap();
                            }
                        }
                    }
                }
            }
        }
    }
    insta::assert_snapshot!(rows);
}
