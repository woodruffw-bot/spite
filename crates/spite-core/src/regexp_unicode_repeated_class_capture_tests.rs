// Remove the nullable choice from the independent executable tree. Sequence
// entry/exit still records ranges directly, independently of production roles.
fn consuming_tree(tree: BoundaryTree) -> BoundaryTree {
    use BoundaryTree::{Capture, First, Main, Repeat, Sequence};
    match tree {
        Main(_, character, _) => consuming_tree(*character),
        Capture(index, child) => Capture(index, Box::new(consuming_tree(*child))),
        Sequence(nodes) => Sequence(nodes.into_iter().map(consuming_tree).collect()),
        First(nodes) => First(nodes.into_iter().map(consuming_tree).collect()),
        Repeat(child, min, source) => Repeat(Box::new(consuming_tree(*child)), min, source),
        leaf => leaf,
    }
}

#[test]
fn repeated_class_captures_follow_independent_executable_sequence_capture_oracle() {
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
    let huge = 10u128.pow(30);
    for (class, contains) in classes {
        for (open, close, roots) in [("(", ")", 1), ("(?:", ")", 0)] {
            for layout in 0..6 {
                let empty_first = false;
                let mut tree = consuming_tree(boundary_tree(layout, class, empty_first));
                let mut captures = roots;
                let body = tree.source(&mut captures);
                for (count, minimum, optional, greedy) in [
                    ("*".to_owned(), Some(0), None, true),
                    ("*?".to_owned(), Some(0), None, false),
                    ("+".to_owned(), Some(1), None, true),
                    ("+?".to_owned(), Some(1), None, false),
                    ("?".to_owned(), Some(0), Some(1), true),
                    ("??".to_owned(), Some(0), Some(1), false),
                    ("{0}".to_owned(), Some(0), Some(0), true),
                    ("{1}".to_owned(), Some(1), Some(0), true),
                    ("{2,3}".to_owned(), Some(2), Some(1), true),
                    ("{2,3}?".to_owned(), Some(2), Some(1), false),
                    (format!("{{{huge}}}"), None, Some(0), true),
                    (format!("{{{huge},}}?"), None, None, false),
                    (format!("{{{huge},{}}}", huge + 1), None, Some(1), true),
                    (format!("{{1,{huge}}}?"), Some(1), None, false),
                ] {
                    for (prefix, suffix, anchors, words) in [
                        ("", "", false, [None, None]),
                        ("^", "$", true, [None, None]),
                        (r"^\b", r"\b$", true, [Some(true), Some(true)]),
                    ] {
                        for i in [false, true] {
                            for m in [false, true] {
                                let source = format!("{prefix}{open}{body}{close}{count}{suffix}");
                                let plan =
                                    super::RegExpUnicodeRepeatedClassCaptureMatcher::compile_with_work(
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
                                    let minimum = minimum.unwrap_or(input.len() + 1);
                                    let oracle = BoundaryOracle {
                                        tree: &tree,
                                        base: Oracle {
                                            input: &input,
                                            contains,
                                            minimum,
                                            maximum: optional.map(|n| minimum + n),
                                            greedy,
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
}

#[test]
fn repeated_class_point_capture_work_cancels_and_large_layouts_stay_flat() {
    use super::RegExpUnicodeRepeatedClassCaptureMatcher as Repeated;
    for pattern in [
        r"^\b(?:()([\q{a|b}])()){2,3}?\b$",
        r"(?:()?[\q{ab}&&a]()){0}",
        r"(?:(())?([\q{😀|😀x}&&\p{Any}])()){999999999999999999999999999999,1000000000000000000000000000000}",
    ] {
        let source = JsString::from(pattern);
        let mut complete = Vec::new();
        Repeated::compile_with_work(&source, true, true, |n| {
            complete.push(n);
            Ok::<(), usize>(())
        })
        .unwrap()
        .unwrap();
        assert_eq!(complete[0], source.len() * 80);
        for fail in 1..=complete.len() {
            let mut seen = Vec::new();
            let result = Repeated::compile_with_work(&source, true, true, |n| {
                seen.push(n);
                if seen.len() == fail {
                    Err(fail)
                } else {
                    Ok(())
                }
            });
            assert!(
                matches!(result, Err(crate::RegExpUnicodeSetCompileError::Work(at)) if at == fail)
            );
            assert_eq!(seen, complete[..fail]);
        }
        let mut recovered = Vec::new();
        Repeated::compile_with_work(&source, true, true, |n| {
            recovered.push(n);
            Ok::<(), usize>(())
        })
        .unwrap()
        .unwrap();
        assert_eq!(recovered, complete);
    }
    for before in [false, true] {
        let groups = if before {
            format!("{}{}", "(".repeat(20_000), ")".repeat(20_000))
        } else {
            "()".repeat(20_000)
        };
        for count in [
            "+",
            "{0}",
            "{999999999999999999999999999999,1000000000000000000000000000000}",
        ] {
            let source = if before {
                format!("(?:{groups}[\\q{{a|b}}]){count}")
            } else {
                format!("(?:[\\q{{a|b}}]{groups}){count}")
            };
            let source = JsString::from(source.as_str());
            let mut work = 0;
            let original = Repeated::compile_with_work(&source, false, false, |n| {
                work += n;
                Ok::<(), ()>(())
            })
            .unwrap()
            .unwrap();
            assert!(work < source.len() * 256 + 1024, "{work}");
            let plan = original.clone();
            drop(original);
            assert_eq!(plan.capture_count(), 20_000);
            let found = plan
                .find_with_captures_and_work(&JsString::from("aaa"), 0, true, |_| Ok::<(), ()>(()))
                .unwrap();
            if count.starts_with("{999") {
                assert!(found.is_none());
                continue;
            }
            let found = found.unwrap();
            assert_eq!(found.range, 0..if count == "+" { 3 } else { 0 });
            for slot in 0..20_000 {
                assert_eq!(
                    plan.capture_range(slot, found.atom_capture.as_ref()),
                    if count == "+" {
                        let point = if before { 2 } else { 3 };
                        Some(point..point)
                    } else {
                        None
                    }
                );
            }
            assert_eq!(
                plan.capture_range(20_000, found.atom_capture.as_ref()),
                None
            );
        }
    }
    let digits = "9".repeat(20_000);
    for (inner, defined) in [
        (format!("{{0,{digits}}}?"), false),
        (format!("{{{digits}}}"), true),
    ] {
        let source = JsString::from(format!("(?:(){inner}[\\q{{😀}}]())+").as_str());
        let plan = Repeated::compile_with_work(&source, false, false, |_| Ok::<(), ()>(()))
            .unwrap()
            .unwrap();
        let found = plan
            .find_with_captures_and_work(&JsString::from("😀😀"), 1, true, |_| Ok::<(), ()>(()))
            .unwrap()
            .unwrap();
        assert_eq!(found.range, 0..4);
        assert_eq!(
            plan.capture_range(0, found.atom_capture.as_ref()),
            defined.then_some(2..2)
        );
        assert_eq!(
            plan.capture_range(1, found.atom_capture.as_ref()),
            Some(4..4)
        );
    }
}

#[test]
fn repeated_class_point_captures_keep_nullable_consuming_asserted_unknown_and_no_point_shapes_closed()
 {
    for source in [
        r"(?:[\q{a}])+",
        r"([\q{a}])+",
        r"(?:()([\q{a}])|)+",
        r"(?:()(?:[\q{a}]|))?",
        r"(?:()([\q{a}])?)+",
        r"(?:()[\q{a}][\q{b}])+",
        r"(?:()[\q{a}]x)+",
        r"(?:()\b[\q{a}])+",
        r"(?:()[\q{a}]\1)+",
        r"(?:()[\q{a}])+[b]",
        r"(?:()[\q{a}])",
        r"(?:()[\q{ab}&&\q{cd}]){0}",
        r"(?:()[\q{ab}&&\p{Any}&&\p{RGI_Emoji}]){0}",
        r"(?:()[\q{a|ab}]){0}",
    ] {
        assert!(
            super::RegExpUnicodeRepeatedClassCaptureMatcher::compile_with_work(
                &JsString::from(source),
                true,
                true,
                |_| Ok::<(), ()>(())
            )
            .unwrap()
            .is_none(),
            "{source}"
        );
    }
}
