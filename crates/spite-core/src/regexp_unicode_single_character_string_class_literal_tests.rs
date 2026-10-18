use super::*;

#[derive(Clone, Copy)]
enum Set {
    Only(&'static [u32]),
    Except(&'static [u32]),
}

impl Set {
    fn contains(self, point: u32) -> bool {
        match self {
            Self::Only(points) => points.contains(&point),
            Self::Except(points) => !points.contains(&point),
        }
    }
}

#[derive(Clone, Copy, Default)]
struct Gates {
    ignore_case: bool,
    multiline: bool,
    anchors: [bool; 2],
    words: [Option<bool>; 2],
}

struct Input {
    text: JsString,
    points: Vec<u32>,
    boundaries: Vec<usize>,
}

impl Input {
    fn new(text: JsString) -> Self {
        let mut points = Vec::new();
        let mut boundaries = vec![0];
        // Standard decoding and handwritten membership avoid matcher internals.
        for point in char::decode_utf16(text.code_units().iter().copied()) {
            let (point, width) = match point {
                Ok(character) => (u32::from(character), character.len_utf16()),
                Err(error) => (u32::from(error.unpaired_surrogate()), 1),
            };
            points.push(point);
            boundaries.push(boundaries.last().unwrap() + width);
        }
        Self {
            text,
            points,
            boundaries,
        }
    }
}

fn fold(point: u32) -> u32 {
    match point {
        0x41..=0x5a => point + 0x20,
        0x17f => 0x73,
        0x212a => 0x6b,
        _ => point,
    }
}

fn expected(
    set: Set,
    literal: u32,
    gates: Gates,
    input: &Input,
    start: usize,
    sticky: bool,
) -> Option<Range<usize>> {
    if start > input.text.len() {
        return None;
    }
    // The approved low-surrogate start selects the containing leading boundary.
    let first = input.boundaries.partition_point(|&at| at <= start) - 1;
    let canonical = |point| {
        if gates.ignore_case {
            fold(point)
        } else {
            point
        }
    };
    let word = |point| {
        matches!(point, 0x30..=0x39 | 0x41..=0x5a | 0x5f | 0x61..=0x7a)
            || (gates.ignore_case && matches!(point, 0x17f | 0x212a))
    };
    let boundary = |at: usize| {
        input.points.get(at).is_some_and(|&p| word(p))
            != at
                .checked_sub(1)
                .and_then(|at| input.points.get(at))
                .is_some_and(|&p| word(p))
    };
    let line = |point| matches!(point, 0xa | 0xd | 0x2028 | 0x2029);
    // Independent adjacent-character enumeration, rather than production retries.
    for at in first..input.points.len().saturating_sub(1) {
        if sticky && at != first {
            break;
        }
        if set.contains(input.points[at])
            && canonical(input.points[at + 1]) == canonical(literal)
            && (!gates.anchors[0] || at == 0 || (gates.multiline && line(input.points[at - 1])))
            && (!gates.anchors[1]
                || at + 2 == input.points.len()
                || (gates.multiline && line(input.points[at + 2])))
            && gates.words[0].is_none_or(|wanted| boundary(at) == wanted)
            && gates.words[1].is_none_or(|wanted| boundary(at + 2) == wanted)
        {
            return Some(input.boundaries[at]..input.boundaries[at + 2]);
        }
    }
    None
}

fn recording(
    source: &JsString,
    i: bool,
    m: bool,
    work: &mut Vec<usize>,
    fail: Option<usize>,
) -> Result<Option<RegExpUnicodeClassLiteralMatcher>, RegExpUnicodeSetCompileError<usize>> {
    RegExpUnicodeClassLiteralMatcher::compile_v_set_then_literal_with_single_character_strings_and_work(
        source, i, m, |amount| {
            work.push(amount);
            if fail == Some(work.len()) { Err(work.len()) } else { Ok(()) }
        },
    )
}

fn compile(source: &str, i: bool, m: bool) -> Option<RegExpUnicodeClassLiteralMatcher> {
    recording(&JsString::from(source), i, m, &mut Vec::new(), None).unwrap()
}

#[test]
fn single_character_string_class_literal_matches_independent_utf16_pairs() {
    use Set::{Except, Only};
    let inputs = [
        JsString::from(""),
        JsString::from("aabaaa!abx"),
        JsString::from("ſKsK!ks"),
        JsString::from("A😀🚀!"),
        JsString::from_code_units(vec![0xd800, 92, 0xdc00, 0xdc00, 33]),
        JsString::from("x\r\nab\u{2028}]$tail"),
        JsString::from("$]$[\\a!"),
        JsString::from("K$"),
    ]
    .map(Input::new);
    let plain = Gates::default();
    let folded = Gates {
        ignore_case: true,
        ..plain
    };
    let anchored = Gates {
        anchors: [true, true],
        ..plain
    };
    let cases = [
        (r"[\q{a|b}]a", Only(&[0x61, 0x62]), 0x61, plain),
        (r"[\q{a|b}&&[b-z]]a", Only(&[0x62]), 0x61, plain),
        (r"[\q{a|b}--a]a", Only(&[0x62]), 0x61, plain),
        (r"[[\q{a|b}]--[\q{a}]]a", Only(&[0x62]), 0x61, plain),
        (r"[[^\q{a}]&&[\q{a|b}]]a", Only(&[0x62]), 0x61, plain),
        (r"[^\q{a|b}]a", Except(&[0x61, 0x62]), 0x61, plain),
        (r"[\q{a}--a]a", Only(&[]), 0x61, plain),
        (r"[\q{K}&&k]s", Only(&[0x4b, 0x6b, 0x212a]), 0x73, folded),
        (r"[\q{K}&&k]s", Only(&[]), 0x73, plain),
        (
            r"[^\q{K|s}]s",
            Except(&[0x4b, 0x6b, 0x212a, 0x53, 0x73, 0x17f]),
            0x73,
            folded,
        ),
        (r"[^\q{K|s}]s", Except(&[0x4b, 0x73]), 0x73, plain),
        (
            r"[\q{\u{1f600}|x}--x]\u{1f680}",
            Only(&[0x1f600]),
            0x1f680,
            plain,
        ),
        (
            r"[\q{\uD83D\uDE00}]\uD83D\uDE80",
            Only(&[0x1f600]),
            0x1f680,
            plain,
        ),
        ("[\\q{😀}]🚀", Only(&[0x1f600]), 0x1f680, plain),
        (r"[\q{\uD800|x}--x]\\", Only(&[0xd800]), 92, plain),
        (r"[\q{\]|\}}]\$", Only(&[0x5d, 0x7d]), 0x24, plain),
        (r"[\q{\[|\\}]a", Only(&[0x5b, 92]), 0x61, plain),
        (r"[\q{a}]\u{00000061}", Only(&[0x61]), 0x61, plain),
        (r"^[\q{a}]b$", Only(&[0x61]), 0x62, anchored),
        (
            r"^[\q{a}]b$",
            Only(&[0x61]),
            0x62,
            Gates {
                multiline: true,
                ..anchored
            },
        ),
        (
            r"[\q{a}]a\b",
            Only(&[0x61]),
            0x61,
            Gates {
                words: [None, Some(true)],
                ..plain
            },
        ),
        (
            r"\b[\q{a}]a\B",
            Only(&[0x61]),
            0x61,
            Gates {
                words: [Some(true), Some(false)],
                ..plain
            },
        ),
        (
            r"^[\q{a}]b",
            Only(&[0x61]),
            0x62,
            Gates {
                anchors: [true, false],
                multiline: true,
                ..plain
            },
        ),
        (
            r"^\b[\q{\!}\p{Any}]\$\B$",
            Except(&[]),
            0x24,
            Gates {
                ignore_case: true,
                words: [Some(true), Some(false)],
                ..anchored
            },
        ),
        (
            r"^\b[\q{\!}\p{Any}]\$\B$",
            Except(&[]),
            0x24,
            Gates {
                words: [Some(true), Some(false)],
                ..anchored
            },
        ),
        (
            r"^\b[\q{a}]b\b$",
            Only(&[0x61]),
            0x62,
            Gates {
                multiline: true,
                words: [Some(true), Some(true)],
                ..anchored
            },
        ),
    ];
    assert_eq!(cases.len(), 26);
    let mut queries = 0;
    for (source, set, literal, gates) in cases {
        let original = compile(source, gates.ignore_case, gates.multiline).unwrap();
        let matcher = original.clone();
        drop(original);
        for input in &inputs {
            for start in 0..=input.text.len() + 1 {
                for sticky in [false, true] {
                    assert_eq!(
                        matcher.find(&input.text, start, sticky),
                        expected(set, literal, gates, input, start, sticky),
                        "{source} input={:?} start={start} sticky={sticky}",
                        input.text
                    );
                    queries += 1;
                }
            }
        }
    }
    // Eight tiny inputs contain49 units:2*(49+2*8)=130 queries per plan.
    assert_eq!(queries, 3380);
    for (source, input, start, sticky, i, m, wanted) in [
        (r"[\q{a}]b", "aab!", 0, false, false, false, Some(1..3)),
        (r"[\q{a}]a\b", "aaa!", 0, false, false, false, Some(1..3)),
        (r"[\q{a}]a\b", "aaa!", 0, true, false, false, None),
        (r"[\q{a}]a\b", "aaa!", 1, true, false, false, Some(1..3)),
        (r"^[\q{a}]b", "abx", 0, false, false, false, Some(0..2)),
        (
            r"^[\q{\]|\\|\[}]\$",
            "zxy\r\n]$tail\u{2028}",
            0,
            false,
            false,
            true,
            Some(5..7),
        ),
        (
            r"^\b[\q{\!}\p{Any}]\$\B$",
            "K$",
            0,
            true,
            true,
            false,
            Some(0..2),
        ),
        (
            r"^\b[\q{\!}\p{Any}]\$\B$",
            "K$",
            0,
            true,
            false,
            false,
            None,
        ),
        (
            r"[\q{\uD83D\uDE00}]\uD83D\uDE80",
            "A😀🚀!",
            2,
            true,
            false,
            false,
            Some(1..5),
        ),
    ] {
        assert_eq!(
            compile(source, i, m)
                .unwrap()
                .find(&JsString::from(input), start, sticky),
            wanted,
            "{source}"
        );
    }
}

#[test]
fn single_character_string_class_literal_domains_and_work_prefixes() {
    for source in [
        "",
        "ab",
        "[a]b",
        "[[a]]b",
        r"[\p{Any}]b",
        r"a[\q{b}]",
        r"[\q{a}]",
        r"[\q{a}]bc",
        r"[\q{a}][b]",
        r"([\q{a}])b",
        r"[\q{a}](b)",
        r"([\q{a}]b)",
        r"[\q{a}]+b",
        r"[\q{a}]b+",
        r"[\q{a}]b?",
        r"[\q{a}]b|c",
        r"\b^[\q{a}]b",
        r"[\q{a}]\bb",
        r"[\q{a}]b$\b",
        r"\b\b[\q{a}]b",
        r"[\q{a}]b\b\b",
        r"[\q{a}]b$$",
        r"^^[\q{a}]b",
        r"[\q{a}]^b",
        r"[\q{a}]\u{D800}\u{DC00}",
        r"[\q{\u{D800}\u{DC00}}]b",
        r"[\q{}]b",
        r"[\q{|a}]b",
        r"[\q{a|bc}]b",
        r"[a--a--\q{a|bc}]b",
        r"[\q{a}&&[]&&\p{RGI_Emoji}]b",
        r"[\p{RGI_Emoji}]b",
    ] {
        assert!(compile(source, true, true).is_none(), "{source}");
    }
    // Component API transcripts independently specify callback order and amount;
    // no empty class or impossible outside gate may skip the literal proof.
    for (source, class, literal, i, m, accepted) in [
        (
            r"^\b[\q{a|b}&&[b]]a\b$",
            r"[\q{a|b}&&[b]]",
            "a",
            false,
            false,
            true,
        ),
        (
            r"[a--a--\q{a}]\u{00000061}",
            r"[a--a--\q{a}]",
            r"\u{00000061}",
            false,
            false,
            true,
        ),
        (r"^\b[\q{K}&&k]a\b$", r"[\q{K}&&k]", "a", true, false, true),
        (
            r"[\q{\]|\\|\[}]\$",
            r"[\q{\]|\\|\[}]",
            r"\$",
            false,
            true,
            true,
        ),
        (
            r"[\q{a}--a--\q{b|cd}]z",
            r"[\q{a}--a--\q{b|cd}]",
            "z",
            false,
            false,
            false,
        ),
        (
            r"[\q{a}&&[]&&\p{RGI_Emoji}]z",
            r"[\q{a}&&[]&&\p{RGI_Emoji}]",
            "z",
            false,
            false,
            false,
        ),
        (r"\b[\q{a|}]\$\B", r"[\q{a|}]", r"\$", false, false, false),
    ] {
        let source = JsString::from(source);
        let class = JsString::from(class);
        let literal = JsString::from(literal);
        let mut expected = vec![source.len().saturating_mul(4)];
        let component =
            RegExpUnicodeClassMatcher::compile_v_set_units_with_single_character_strings_and_work(
                class.code_units(),
                i,
                m,
                |amount| {
                    expected.push(amount);
                    Ok::<(), usize>(())
                },
            )
            .unwrap();
        if component.is_some() {
            assert!(
                RegExpUnicodeCharacterMatcher::compile_literal_units_with_flags_and_work(
                    literal.code_units(),
                    i,
                    m,
                    |amount| {
                        expected.push(amount);
                        Ok::<(), usize>(())
                    },
                )
                .unwrap()
                .is_some()
            );
        }
        let mut complete = Vec::new();
        assert_eq!(
            recording(&source, i, m, &mut complete, None)
                .unwrap()
                .is_some(),
            accepted
        );
        assert_eq!(complete, expected);
        for fail in 1..=complete.len() {
            let mut seen = Vec::new();
            assert!(
                matches!(recording(&source, i, m, &mut seen, Some(fail)), Err(RegExpUnicodeSetCompileError::Work(at)) if at == fail)
            );
            assert_eq!(seen, complete[..fail]);
        }
    }
    // All eight older adapters keep declining q-first pairs with their original
    // callbacks; the legacy boundary helper continues to reject the q token.
    let source = JsString::from(r"\b[\q{a|b}]a\b");
    assert_eq!(
        class_end(JsString::from(r"[\q{a|b}]a").code_units(), 0),
        None
    );
    macro_rules! old_none {
        ($method:ident) => {
            assert!(
                RegExpUnicodeClassLiteralMatcher::$method(&source, false, false, |_| Ok::<(), ()>(
                    ()
                ))
                .unwrap()
                .is_none()
            );
        };
    }
    old_none!(compile_v_set_with_work);
    old_none!(compile_literal_then_v_set_with_work);
    old_none!(compile_v_set_with_word_boundaries_and_work);
    old_none!(compile_flat_v_set_then_literal_with_work);
    old_none!(compile_flat_v_set_then_literal_with_word_boundaries_and_work);
    old_none!(compile_literal_then_flat_v_set_with_work);
    old_none!(compile_literal_then_flat_v_set_with_word_boundaries_and_work);
    old_none!(compile_literal_then_v_set_with_single_character_strings_and_work);
    for (source, input, start, sticky, wanted) in [
        (r"[\q{a}]b", "aab!", 0, false, Some(1..3)),
        (r"[\q{a}]a\b", "aaa!", 0, true, None),
        (
            r"[\q{\uD83D\uDE00}]\uD83D\uDE80",
            "A😀🚀!",
            2,
            true,
            Some(1..5),
        ),
    ] {
        let matcher = compile(source, false, false).unwrap();
        let input = JsString::from(input);
        let mut work = Vec::new();
        assert_eq!(
            matcher
                .find_with_work(&input, start, sticky, |amount| {
                    work.push(amount);
                    Ok::<(), usize>(())
                })
                .unwrap(),
            wanted
        );
        assert_eq!(work, [1]);
        assert!(matches!(
            matcher.find_with_work(&input, start, sticky, |_| Err(7)),
            Err(7)
        ));
    }
}
