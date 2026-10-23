#[test]
fn single_class_captures_follow_independent_sequence_capture_oracle_without_outer_counts() {
    let classes: [(&str, Predicate); 8] = [
        (r"[\q{a|b}]", |p| p == 97 || p == 98),
        (r"[\q{a|b|ab}&&\p{Any}]", |p| p == 97 || p == 98),
        (r"[\q{ab}&&a]", |_| false),
        (r"[^\q{ab}&&a]", |_| true),
        (r"[\q{😀|😀x}&&\p{Any}]", |p| p == 0x1f600),
        (r"[\q{\u{10428}|ab}&&\p{Any}]", |p| p == 0x10428),
        (r"[\q{s|k}]", |p| p == 0x73 || p == 0x6b),
        (r"[\q{\(|\)|a}]", |p| p == 0x28 || p == 0x29 || p == 97),
    ];

    for (class, contains) in classes {
        for (open, close, roots) in [("(", ")", 1), ("(?:", ")", 0), ("", "", 0)] {
            for layout in 0..6 {
                let empty_first = false;
                let mut tree = consuming_tree(boundary_tree(layout, class, empty_first));
                let mut captures = roots;
                let body = tree.source(&mut captures);
                for (prefix, suffix, anchors, words) in [
                    ("", "", false, [None, None]),
                    ("^", "$", true, [None, None]),
                    (r"^\b", r"\b$", true, [Some(true), Some(true)]),
                ] {
                    for i in [false, true] {
                        for m in [false, true] {
                            let source = format!("{prefix}{open}{body}{close}{suffix}");
                            let plan =
                                super::RegExpUnicodeRepeatedClassCaptureMatcher::compile_single_with_work(
                                    &JsString::from(source.as_str()),
                                    i,
                                    m,
                                    |_| Ok::<(), ()>(()),
                                )
                                .unwrap()
                                .unwrap();
                            assert_eq!(plan.capture_count(), captures);
                            for input in [
                                JsString::from(""),
                                JsString::from("aaab!a"),
                                JsString::from("A😀😀a"),
                                JsString::from("!\r\na\u{2028}!"),
                                JsString::from("ſK"),
                                JsString::from("𐐀𐐨!"),
                                JsString::from("(a)!"),
                                JsString::from_code_units(vec![0xd800, 97, 0xdc00]),
                            ] {
                                let oracle = BoundaryOracle {
                                    tree: &tree,
                                    base: Oracle {
                                        input: &input,
                                        contains,
                                        minimum: 1,
                                        maximum: Some(1),
                                        greedy: true,
                                        empty_first,
                                        roots,
                                        captures,
                                        empty_slots: &[],
                                        never_slots: &[],
                                        i,
                                        m,
                                        anchors,
                                        words,
                                    },
                                };
                                for start in 0..=input.len() + 1 {
                                    for sticky in [false, true] {
                                        let found = plan
                                            .find_with_captures_and_work(
                                                &input,
                                                start,
                                                sticky,
                                                |_| Ok::<(), ()>(()),
                                            )
                                            .unwrap();
                                        let projected = found.map(|found| Expected {
                                            captures: (0..captures)
                                                .map(|slot| {
                                                    plan.capture_range(
                                                        slot,
                                                        found.atom_capture.as_ref(),
                                                    )
                                                })
                                                .collect(),
                                            range: found.range,
                                        });
                                        assert_eq!(
                                            projected,
                                            oracle.find(start, sticky),
                                            "{source} i={i} m={m} {input:?} start={start} sticky={sticky}"
                                        );
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn single_class_capture_construction_cancels_and_large_unwrapped_point_tables_stay_flat() {
    use super::RegExpUnicodeRepeatedClassCaptureMatcher as Single;
    for pattern in [
        r"^\b()([\q{a|b}])()\b$",
        r"(?:()?[\q{ab}&&a]())",
        r"(?:(())?([\q{😀|😀x}&&\p{Any}])()){999999999999999999999999999999}",
    ] {
        let source = JsString::from(pattern);
        let mut complete = Vec::new();
        let plan = Single::compile_single_with_work(&source, true, true, |n| {
            complete.push(n);
            Ok::<(), usize>(())
        })
        .unwrap();
        assert_eq!(complete[0], source.len() * 8 + 32);
        assert_eq!(
            plan.is_some(),
            !pattern.ends_with("999999999999999999999999999999}")
        );
        for fail in 1..=complete.len() {
            let mut seen = Vec::new();
            let result = Single::compile_single_with_work(&source, true, true, |n| {
                seen.push(n);
                if seen.len() == fail {
                    Err(fail)
                } else {
                    Ok(())
                }
            });
            assert!(
                matches!(result,Err(crate::RegExpUnicodeSetCompileError::Work(at)) if at==fail)
            );
            assert_eq!(seen, complete[..fail]);
        }
        let mut recovered = Vec::new();
        Single::compile_single_with_work(&source, true, true, |n| {
            recovered.push(n);
            Ok::<(), usize>(())
        })
        .unwrap();
        assert_eq!(recovered, complete);
    }
    for before in [false, true] {
        let captures = if before {
            format!("{}{}", "(".repeat(20_000), ")".repeat(20_000))
        } else {
            "()".repeat(20_000)
        };
        let source = JsString::from(
            if before {
                format!("{captures}[\\q{{😀}}]")
            } else {
                format!("[\\q{{😀}}]{captures}")
            }
            .as_str(),
        );
        let mut work = 0;
        let original = Single::compile_single_with_work(&source, false, false, |n| {
            work += n;
            Ok::<(), ()>(())
        })
        .unwrap()
        .unwrap();
        assert!(work < source.len() * 300 + 4096, "{work}");
        let plan = original.clone();
        drop(original);
        let found = plan
            .find_with_captures_and_work(&JsString::from("😀😀"), 1, true, |_| Ok::<(), ()>(()))
            .unwrap()
            .unwrap();
        assert_eq!(found.range, 0..2);
        assert_eq!(plan.capture_count(), 20_000);
        let point = if before { 0 } else { 2 };
        for slot in 0..20_000 {
            assert_eq!(
                plan.capture_range(slot, found.atom_capture.as_ref()),
                Some(point..point)
            );
        }
        assert_eq!(
            plan.capture_range(20_000, found.atom_capture.as_ref()),
            None
        );
    }
    let digits = "9".repeat(20_000);
    for (inner, defined) in [
        (format!("{{0,{digits}}}?"), false),
        (format!("{{{digits}}}"), true),
    ] {
        let source = JsString::from(format!("(){inner}[\\q{{a}}]()").as_str());
        let plan = Single::compile_single_with_work(&source, false, false, |_| Ok::<(), ()>(()))
            .unwrap()
            .unwrap();
        let found = plan
            .find_with_captures_and_work(&JsString::from("aaa"), 1, true, |_| Ok::<(), ()>(()))
            .unwrap()
            .unwrap();
        assert_eq!(found.range, 1..2);
        assert_eq!(
            plan.capture_range(0, found.atom_capture.as_ref()),
            defined.then_some(1..1)
        );
        assert_eq!(
            plan.capture_range(1, found.atom_capture.as_ref()),
            Some(2..2)
        );
    }
}

#[test]
fn single_class_capture_proof_keeps_outer_counts_nullable_choices_assertions_and_unknown_terms_closed()
 {
    for pattern in [
        r"[\q{a}]",
        r"([\q{a}])",
        r"(?:()[\q{a}])+",
        r"(?:()[\q{a}]){0}",
        r"()[\q{a}]?",
        r"()[\q{a}]|()",
        r"()(?:[\q{a}]|)",
        r"()\b[\q{a}]",
        r"()[\q{a}]\1",
        r"()[\q{a}]x",
        r"()[\q{a}][\q{b}]",
        r"()[\q{a|ab}]",
        r"()[\q{ab}&&\q{cd}]",
        r"()[\q{ab}&&\p{Any}&&\p{RGI_Emoji}]",
    ] {
        assert!(
            super::RegExpUnicodeRepeatedClassCaptureMatcher::compile_single_with_work(
                &JsString::from(pattern),
                true,
                true,
                |_| Ok::<(), ()>(())
            )
            .unwrap()
            .is_none(),
            "{pattern}"
        );
    }
}
