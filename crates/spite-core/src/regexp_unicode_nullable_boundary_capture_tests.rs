// An executable small capture tree provides sequence entry/exit ranges instead
// of deriving the production matcher's source-position capture roles.
#[derive(Clone)]
enum BoundaryTree {
    Empty,
    Class(&'static str),
    Capture(usize, Box<Self>),
    Sequence(Vec<Self>),
    Main(bool, Box<Self>, Box<Self>),
    First(Vec<Self>),
    Repeat(Box<Self>, usize, String),
}
impl BoundaryTree {
    fn capture(child: Self) -> Self {
        Self::Capture(usize::MAX, Box::new(child))
    }
    fn empty_capture() -> Self {
        Self::capture(Self::Empty)
    }
    fn source(&mut self, next: &mut usize) -> String {
        match self {
            Self::Empty => String::new(),
            Self::Class(source) => (*source).to_owned(),
            Self::Capture(index, child) => {
                *index = *next;
                *next += 1;
                format!("({})", child.source(next))
            }
            Self::Sequence(nodes) => nodes.iter_mut().map(|node| node.source(next)).collect(),
            Self::Main(empty_first, character, empty) => {
                if *empty_first {
                    format!("(?:{}|{})", empty.source(next), character.source(next))
                } else {
                    format!("(?:{}|{})", character.source(next), empty.source(next))
                }
            }
            Self::First(nodes) => format!(
                "(?:{})",
                nodes
                    .iter_mut()
                    .map(|node| node.source(next))
                    .collect::<Vec<_>>()
                    .join("|")
            ),
            Self::Repeat(child, _, count) => format!("(?:{}){count}", child.source(next)),
        }
    }
    fn clear(&self, captures: &mut [Option<Range<usize>>]) {
        match self {
            Self::Capture(index, child) => {
                captures[*index] = None;
                child.clear(captures);
            }
            Self::Sequence(nodes) | Self::First(nodes) => {
                for child in nodes {
                    child.clear(captures);
                }
            }
            Self::Main(_, character, empty) => {
                character.clear(captures);
                empty.clear(captures);
            }
            Self::Repeat(child, _, _) => child.clear(captures),
            Self::Empty | Self::Class(_) => {}
        }
    }
    fn eval(
        &self,
        consuming: bool,
        at: usize,
        captures: &mut [Option<Range<usize>>],
        oracle: &Oracle<'_>,
    ) -> Option<usize> {
        match self {
            Self::Empty => Some(at),
            Self::Class(_) => {
                if !consuming {
                    return None;
                }
                let (mut point, next) = oracle.point(at)?;
                if oracle.i {
                    point = match point {
                        0x41..=0x5a => point + 32,
                        0x17f => 0x73,
                        0x212a => 0x6b,
                        0x10400..=0x10427 => point + 0x28,
                        _ => point,
                    };
                }
                (oracle.contains)(point).then_some(next)
            }
            Self::Capture(index, child) => {
                let end = child.eval(consuming, at, captures, oracle)?;
                captures[*index] = Some(at..end);
                Some(end)
            }
            Self::Sequence(nodes) => {
                let mut end = at;
                for child in nodes {
                    end = child.eval(consuming, end, captures, oracle)?;
                }
                Some(end)
            }
            Self::Main(_, character, empty) => {
                if consuming {
                    character.eval(consuming, at, captures, oracle)
                } else {
                    empty.eval(consuming, at, captures, oracle)
                }
            }
            // These choices have only epsilon children and an input-only
            // continuation. Their first success supplies the capture profile.
            Self::First(nodes) => nodes.first().unwrap().eval(consuming, at, captures, oracle),
            Self::Repeat(child, minimum, _) => {
                // All repeated subtrees here are epsilon. Optional iterations
                // fail the progress check; required ones clear and enter groups.
                for _ in 0..*minimum {
                    child.clear(captures);
                    assert_eq!(child.eval(consuming, at, captures, oracle), Some(at));
                }
                Some(at)
            }
        }
    }
}

struct BoundaryOracle<'a> {
    base: Oracle<'a>,
    tree: &'a BoundaryTree,
}
impl BoundaryOracle<'_> {
    fn repeat(
        &self,
        at: usize,
        min: usize,
        max: Option<usize>,
        captures: &[Option<Range<usize>>],
    ) -> Option<(usize, Vec<Option<Range<usize>>>)> {
        if max == Some(0) {
            return (min == 0 && self.base.end(at)).then(|| (at, captures.to_vec()));
        }
        if min == 0 && !self.base.greedy && self.base.end(at) {
            return Some((at, captures.to_vec()));
        }
        for consuming in if self.base.empty_first {
            [false, true]
        } else {
            [true, false]
        } {
            let mut fresh = vec![None; self.base.captures];
            let Some(next) = self.tree.eval(consuming, at, &mut fresh, &self.base) else {
                continue;
            };
            if min == 0 && next == at {
                continue;
            }
            for capture in fresh.iter_mut().take(self.base.roots) {
                *capture = Some(at..next);
            }
            if let Some(result) =
                self.repeat(next, min.saturating_sub(1), max.map(|n| n - 1), &fresh)
            {
                return Some(result);
            }
        }
        (min == 0 && self.base.greedy && self.base.end(at)).then(|| (at, captures.to_vec()))
    }
    fn find(&self, requested: usize, sticky: bool) -> Option<Expected> {
        let units = self.base.input.code_units();
        if requested > units.len() {
            return None;
        }
        let mut start = requested;
        if start > 0
            && start < units.len()
            && (0xdc00..=0xdfff).contains(&units[start])
            && (0xd800..=0xdbff).contains(&units[start - 1])
        {
            start -= 1;
        }
        loop {
            let head = (!self.base.anchors
                || start == 0
                || (self.base.m && units.get(start - 1).is_some_and(|&p| Oracle::line(p))))
                && self.base.words[0].is_none_or(|word| self.base.boundary(start) == word);
            if head {
                if let Some((end, captures)) = self.repeat(
                    start,
                    self.base.minimum,
                    self.base.maximum,
                    &vec![None; self.base.captures],
                ) {
                    return Some(Expected {
                        range: start..end,
                        captures,
                    });
                }
            }
            if sticky || start == units.len() {
                return None;
            }
            start = self.base.point(start).unwrap().1;
        }
    }
}

fn boundary_tree(layout: usize, class: &'static str, empty_first: bool) -> BoundaryTree {
    use BoundaryTree::{Capture, Class, Empty, First, Main, Repeat, Sequence};
    let capture = BoundaryTree::empty_capture;
    let main = |character, empty| Main(empty_first, Box::new(character), Box::new(empty));
    match layout {
        0 => Sequence(vec![
            main(BoundaryTree::capture(Class(class)), Empty),
            capture(),
        ]),
        1 => Sequence(vec![
            capture(),
            main(BoundaryTree::capture(Class(class)), Empty),
        ]),
        2 => main(
            Sequence(vec![capture(), BoundaryTree::capture(Class(class))]),
            Empty,
        ),
        3 => main(
            Sequence(vec![BoundaryTree::capture(Class(class)), capture()]),
            Empty,
        ),
        4 => Sequence(vec![
            capture(),
            main(
                Sequence(vec![
                    capture(),
                    BoundaryTree::capture(Class(class)),
                    capture(),
                ]),
                Sequence(vec![
                    BoundaryTree::capture(First(vec![capture(), capture()])),
                    Repeat(Box::new(capture()), 0, "?".to_owned()),
                ]),
            ),
            capture(),
        ]),
        _ => Sequence(vec![
            Repeat(
                Box::new(capture()),
                0,
                "{0,999999999999999999999999999999}?".to_owned(),
            ),
            main(
                Sequence(vec![
                    Capture(
                        usize::MAX,
                        Box::new(Repeat(Box::new(capture()), 0, "?".to_owned())),
                    ),
                    BoundaryTree::capture(Class(class)),
                    BoundaryTree::capture(First(vec![capture(), capture()])),
                ]),
                BoundaryTree::capture(Repeat(Box::new(capture()), 0, "?".to_owned())),
            ),
            Repeat(
                Box::new(capture()),
                1,
                "{999999999999999999999999999999}".to_owned(),
            ),
        ]),
    }
}

#[test]
fn nullable_boundary_captures_follow_independent_executable_sequence_capture_oracle() {
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
                for empty_first in [false, true] {
                    let mut tree = boundary_tree(layout, class, empty_first);
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
                                    let source =
                                        format!("{prefix}{open}{body}{close}{count}{suffix}");
                                    let plan =
                                        Matcher::compile_with_boundary_empty_captures_and_work(
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
}

#[test]
fn nullable_boundary_capture_construction_cancels_callbacks_and_keeps_large_point_tables_flat() {
    for pattern in [
        r"^\b(?:(?:([\q{a|b}])|)()){2,3}?\b$",
        r"(?:()(?:|[\q{ab}&&a])){0}",
        r"(?:(())?(?:|([\q{😀|😀x}&&\p{Any}]))()){999999999999999999999999999999,1000000000000000000000000000000}",
    ] {
        let source = JsString::from(pattern);
        let mut complete = Vec::new();
        Matcher::compile_with_boundary_empty_captures_and_work(&source, true, true, |n| {
            complete.push(n);
            Ok::<(), usize>(())
        })
        .unwrap()
        .unwrap();
        assert_eq!(complete[0], source.len() * 80);
        for fail in 1..=complete.len() {
            let mut seen = Vec::new();
            let result =
                Matcher::compile_with_boundary_empty_captures_and_work(&source, true, true, |n| {
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
        Matcher::compile_with_boundary_empty_captures_and_work(&source, true, true, |n| {
            recovered.push(n);
            Ok::<(), usize>(())
        })
        .unwrap()
        .unwrap();
        assert_eq!(recovered, complete);
    }
    for empty_first in [false, true] {
        for before in [false, true] {
            let captures = if before {
                format!("{}{}", "(".repeat(20_000), ")".repeat(20_000))
            } else {
                "()".repeat(20_000)
            };
            let branch = if empty_first {
                r"|[\q{a|b}]"
            } else {
                r"[\q{a|b}]|"
            };
            let source = if before {
                format!(
                    "(?:{captures}(?:{branch})){{999999999999999999999999999999,1000000000000000000000000000000}}"
                )
            } else {
                format!(
                    "(?:(?:{branch}){captures}){{999999999999999999999999999999,1000000000000000000000000000000}}"
                )
            };
            let source = JsString::from(source.as_str());
            let mut work = 0;
            let original = Matcher::compile_with_boundary_empty_captures_and_work(
                &source,
                false,
                false,
                |n| {
                    work += n;
                    Ok::<(), ()>(())
                },
            )
            .unwrap()
            .unwrap();
            assert!(work < source.len() * 256 + 1024, "{work}");
            let plan = original.clone();
            drop(original);
            assert_eq!(plan.capture_count(), 20_000);
            let found = plan
                .find_with_captures_and_work(&JsString::from("aaa"), 0, true, |_| Ok::<(), ()>(()))
                .unwrap()
                .unwrap();
            assert_eq!(found.range, 0..if empty_first { 1 } else { 3 });
            let point = if empty_first { usize::from(!before) } else { 3 };
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
    }
}

#[test]
fn nullable_boundary_capture_proofs_keep_nonnullable_consuming_asserted_and_unknown_shapes_closed()
{
    for source in [
        r"(?:()([\q{a}])())+",
        r"(?:(?:([\q{a}])|()))+",
        r"(([\q{a}])|()?)+",
        r"(?:x(?:[\q{a}]|)())+",
        r"(?:(?:[\q{a}]|)\b())+",
        r"(?:(?:[\q{a}]|)\1())+",
        r"(?:(?:[\q{a}]|)())",
        r"(?:(?:[\q{a}]|)())+[b]",
        r"(?:(?:[\q{a}][\q{b}]|)())+",
        r"(?:(?:([\q{a}])?|)())+",
        r"(?:(?:[\q{ab}&&\q{cd}]|)()){0}",
        r"(?:()(?:|[\q{ab}&&\p{Any}&&\p{RGI_Emoji}])){0}",
    ] {
        assert!(
            Matcher::compile_with_boundary_empty_captures_and_work(
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
