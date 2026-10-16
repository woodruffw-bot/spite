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
fn greek_script_complete_membership_aliases_and_complement_orders() {
    let mut raw = vec![false; 0x110000];
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
        if fields.len() < 2 || fields[1] != "Greek" {
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
        raw[a..=b].fill(true);
        records += 1;
    }
    assert_eq!(records, 56);
    assert_eq!(raw.iter().filter(|&&m| m).count(), 520);
    let mut fold: Vec<_> = (0..0x110000).collect();
    let mut folds = 0;
    let mut crossings = Vec::new();
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
            let targets: Vec<_> = fields[2].split_whitespace().collect();
            assert_eq!(targets.len(), 1);
            let target = usize::from_str_radix(targets[0], 16).unwrap();
            fold[source] = target;
            folds += 1;
            if raw[source] != raw[target] {
                crossings.push((source, target));
            }
        }
    }
    assert_eq!(folds, 1533);
    assert_eq!(crossings, [(0xb5, 0x3bc), (0x345, 0x3b9)]);
    let mut pos = vec![false; 0x110000];
    let mut neg = vec![false; 0x110000];
    for (point, &member) in raw.iter().enumerate() {
        let target = fold[point];
        assert_eq!(fold[target], target);
        if member {
            pos[target] = true;
        } else {
            neg[target] = true;
        }
    }
    let mut checks = 0;
    for name in ["Script=Greek", "Script=Grek", "sc=Greek", "sc=Grek"] {
        for sets in [false, true] {
            for ic in [false, true] {
                for (pattern, complemented, inverted) in [
                    (format!(r"\p{{{name}}}"), false, false),
                    (format!(r"\P{{{name}}}"), true, false),
                    (format!(r"[\p{{{name}}}]"), false, false),
                    (format!(r"[\P{{{name}}}]"), true, false),
                    (format!(r"[^\p{{{name}}}]"), false, true),
                    (format!(r"[^\P{{{name}}}]"), true, true),
                ] {
                    let plan = compile(&pattern, sets, ic);
                    for (point, &member) in raw.iter().enumerate() {
                        let wanted = if !ic {
                            member ^ complemented
                        } else if complemented && !sets {
                            neg[fold[point]]
                        } else {
                            pos[fold[point]] ^ complemented
                        };
                        assert_eq!(
                            plan.matches(point as u32),
                            wanted ^ inverted,
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
fn greek_script_high_mask_bits_and_mixed_property_preimages() {
    for sets in [false, true] {
        for ic in [false, true] {
            for (pattern, points) in [
                (
                    r"[\p{ASCII}\p{Cs}\p{sc=Grek}]",
                    [
                        (0x41, true),
                        (0xd800, true),
                        (0x1dff3, true),
                        (0x3a3, true),
                        (0xb5, ic),
                        (0x1df95, false),
                    ],
                ),
                (
                    r"[^\P{Script=Greek}\p{sc=Latn}]",
                    [
                        (0x41, false),
                        (0x3b9, !(ic && !sets)),
                        (0x3bc, !(ic && !sets)),
                        (0x345, ic && sets),
                        (0x1dff3, true),
                        (0x1df95, false),
                    ],
                ),
                (
                    r"[\P{sc=Greek}\p{Script=Latin}]",
                    [
                        (0x41, true),
                        (0x3b9, ic && !sets),
                        (0x3bc, ic && !sets),
                        (0x345, !ic || !sets),
                        (0x1dff3, false),
                        (0x1df95, true),
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
                    r"\p{sc=Greek}\P{Script=Grek}\p{Cs}\P{ASCII}".repeat(10000)
                )
                .as_str(),
            );
            let plan =
                RegExpUnicodeClassMatcher::compile_with_flags(&source, sets, ic, false).unwrap();
            for point in [0, 0x41, 0x378, 0xd800, 0xe000, 0x1dff3, 0x10ffff] {
                assert!(plan.matches(point));
            }
        }
    }
}

#[test]
fn greek_script_work_utf16_and_exact_namespace_contracts() {
    for pattern in [r"\P{Script=Greek}", r"[\P{sc=Grek}]"] {
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
        assert!(!plan.matches(0xb5));
        assert!(!plan.matches(0x345));
        assert!(!plan.matches(0x1dff3));
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
        let pattern = r"\p{Script_Extensions=Greek}";
        let plan = RegExpUnicodeClassMatcher::compile_binary_property_with_flags(
            &JsString::from(pattern),
            false,
            false,
            false,
        )
        .unwrap();
        assert!(plan.matches(0x3b1), "{pattern}");
        assert!(!plan.matches(0x41), "{pattern}");
        // This extension membership differs from the Script fallback.
        assert!(plan.matches(0xb7));
        assert!(!compile(r"\p{sc=Greek}", false, false).matches(0xb7));
    }
    for pattern in [
        r"\p{Greek}",
        r"\p{Grek}",
        r"\p{gc=Greek}",
        r"\p{SCRIPT=Greek}",
        r"\p{Script=greek}",
        r"\p{sc=Script=Greek}",
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
    let input = JsString::from_code_units(vec![0xd83d, 0xde00, 0xd800, 0x3a3, 0xd837, 0xdff3]);
    for sets in [false, true] {
        for ic in [false, true] {
            let plan = compile(r"\p{sc=Greek}", sets, ic);
            assert_eq!(plan.find(&input, 1, false), Some(3..4));
            assert_eq!(plan.find(&input, 1, true), None);
            assert_eq!(plan.find(&input, 5, true), Some(4..6));
        }
    }
}

#[test]
fn greek_script_find_ranges_snapshot() {
    use std::fmt::Write;
    let mut rows = String::new();
    for name in ["Script=Greek", "Script=Grek", "sc=Greek", "sc=Grek"] {
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
                        vec![0xb5, 0x345, 0x3b9, 0x3bc, 0x3a3, 0x3c2, 0x41, 10],
                        vec![0xd800, 0x3a3, 0xd800, 0xdd40, 0xdc00, 0x345],
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
