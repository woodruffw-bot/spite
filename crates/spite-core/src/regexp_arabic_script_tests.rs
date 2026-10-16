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
fn arabic_script_complete_membership_aliases_and_complements() {
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
        if fields.len() < 2 || fields[1] != "Arabic" {
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
    assert_eq!(records, 98);
    assert_eq!(membership.iter().filter(|&&m| m).count(), 1452);
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
    for name in ["Script=Arabic", "Script=Arab", "sc=Arabic", "sc=Arab"] {
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
fn arabic_script_high_mask_bits_coexist_with_general_categories_and_older_properties() {
    for sets in [false, true] {
        for ic in [false, true] {
            for (pattern, points) in [
                (
                    r"[\p{ASCII}\p{Cs}\p{sc=Arab}]",
                    [
                        (0x41, true),
                        (0xd800, true),
                        (0x621, true),
                        (0x1ee00, true),
                        (0x3a3, false),
                        (0x1df95, false),
                    ],
                ),
                (
                    r"[^\P{Script=Arabic}\p{sc=Cyrl}]",
                    [
                        (0x41, false),
                        (0x621, true),
                        (0x627, true),
                        (0xb5, false),
                        (0x1ee00, true),
                        (0xd800, false),
                    ],
                ),
                (
                    r"[\P{sc=Arab}\p{sc=Latn}]",
                    [
                        (0x41, true),
                        (0x621, false),
                        (0x627, false),
                        (0xd800, true),
                        (0x1ee00, false),
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
                    r"\p{sc=Arabic}\P{Script=Arab}\p{gc=Cs}\P{ASCII}".repeat(10000)
                )
                .as_str(),
            );
            let plan =
                RegExpUnicodeClassMatcher::compile_with_flags(&source, sets, ic, false).unwrap();
            for point in [0, 0x41, 0x378, 0xd800, 0xe000, 0x1ee00, 0x10ffff] {
                assert!(plan.matches(point));
            }
        }
    }
}

#[test]
fn arabic_script_work_utf16_and_exact_namespace_contracts() {
    for pattern in [r"\P{Script=Arabic}", r"[\P{sc=Arab}]"] {
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
        assert!(!plan.matches(0x621));
        assert!(!plan.matches(0x1ee00));
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
    {
        let pattern = r"\p{Script_Extensions=Arabic}";
        let plan = RegExpUnicodeClassMatcher::compile_binary_property_with_flags(
            &JsString::from(pattern),
            false,
            false,
            false,
        )
        .unwrap();
        assert!(plan.matches(0x627), "{pattern}");
        assert!(!plan.matches(0x41), "{pattern}");
        // This extension membership differs from the Script fallback.
        assert!(plan.matches(0x60c));
        assert!(!compile(r"\p{sc=Arabic}", false, false).matches(0x60c));
    }
    for pattern in [
        r"\p{Arabic}",
        r"\p{Arab}",
        r"\p{gc=Arabic}",
        r"\p{SCRIPT=Arabic}",
        r"\p{Script=arabic}",
        r"\p{sc=Script=Arabic}",
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
    let input = JsString::from_code_units(vec![0xd83d, 0xde00, 0xd800, 0x621, 0xd83b, 0xde00]);
    for sets in [false, true] {
        for ic in [false, true] {
            let plan = compile(r"\p{sc=Arabic}", sets, ic);
            assert_eq!(plan.find(&input, 1, false), Some(3..4));
            assert_eq!(plan.find(&input, 1, true), None);
            assert_eq!(plan.find(&input, 5, true), Some(4..6));
        }
    }
}

#[test]
fn arabic_script_find_ranges_snapshot() {
    use std::fmt::Write;
    let mut rows = String::new();
    for name in ["Script=Arabic", "Script=Arab", "sc=Arabic", "sc=Arab"] {
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
                        vec![0x621, 0x627, 0x3a3, 0x41, 10],
                        vec![0xd800, 0x621, 0xd83b, 0xde00, 0xdc00, 0x3a3],
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
