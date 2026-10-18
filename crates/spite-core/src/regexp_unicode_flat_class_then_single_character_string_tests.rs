use super::*;

#[derive(Clone, Copy)]
enum Set {
    Only(&'static [u32]),
    Except(&'static [u32]),
    Range(u32, u32),
    Outside(u32, u32),
    Word,
    FoldedAscii,
}

impl Set {
    fn contains(self, point: u32) -> bool {
        match self {
            Self::Only(points) => points.contains(&point),
            Self::Except(points) => !points.contains(&point),
            Self::Range(first, last) => (first..=last).contains(&point),
            Self::Outside(first, last) => !(first..=last).contains(&point),
            Self::FoldedAscii => matches!(point, 0..=0x7f | 0x17f | 0x212a),
            Self::Word => matches!(point, 0x30..=0x39 | 0x41..=0x5a | 0x5f | 0x61..=0x7a),
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
        // Standard decoding and two handwritten sets avoid matcher internals.
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

fn expected(
    first: Set,
    second: Set,
    gates: Gates,
    input: &Input,
    start: usize,
    sticky: bool,
) -> Option<Range<usize>> {
    if start > input.text.len() {
        return None;
    }
    // Approved Unicode low-surrogate starts select the containing boundary.
    let first_candidate = input.boundaries.partition_point(|&at| at <= start) - 1;
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
    // Enumerate adjacent decoded characters independently of production retries.
    for at in first_candidate..input.points.len().saturating_sub(1) {
        if sticky && at != first_candidate {
            break;
        }
        if first.contains(input.points[at])
            && second.contains(input.points[at + 1])
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
) -> Result<Option<RegExpUnicodeClassPairMatcher>, RegExpUnicodeSetCompileError<usize>> {
    RegExpUnicodeClassPairMatcher::compile_flat_then_v_set_with_single_character_strings_and_work(
        source,
        i,
        m,
        |amount| {
            work.push(amount);
            if fail == Some(work.len()) {
                Err(work.len())
            } else {
                Ok(())
            }
        },
    )
}

fn compile(source: &str, i: bool, m: bool) -> Option<RegExpUnicodeClassPairMatcher> {
    recording(&JsString::from(source), i, m, &mut Vec::new(), None).unwrap()
}

#[test]
fn flat_class_then_single_character_string_matches_independent_utf16_pairs() {
    use Set::{Except, FoldedAscii, Only, Outside, Range, Word};
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
    // Membership is handwritten independently for each operand before operators.
    // In vi ASCII closes over Kelvin and long s as well as U+0000..U+007F.
    let cases = [
        (r"[a][\q{a|b}]", Only(&[0x61]), Only(&[0x61, 0x62]), plain),
        (r"[a][\q{a|b}&&[b-z]]", Only(&[0x61]), Only(&[0x62]), plain),
        (
            r"[a-c][\q{a|b}--a]",
            Range(0x61, 0x63),
            Only(&[0x62]),
            plain,
        ),
        (
            r"[\D][[\q{a|b}]--[\q{a}]]",
            Outside(0x30, 0x39),
            Only(&[0x62]),
            plain,
        ),
        (r"[\w][[^\q{a}]&&[\q{a|b}]]", Word, Only(&[0x62]), plain),
        (
            r"[^a][^\q{a|b}]",
            Except(&[0x61]),
            Except(&[0x61, 0x62]),
            plain,
        ),
        (r"[][\q{a}--a]", Only(&[]), Only(&[]), plain),
        (
            r"[s][\q{K}&&k]",
            Only(&[0x53, 0x73, 0x17f]),
            Only(&[0x4b, 0x6b, 0x212a]),
            folded,
        ),
        (r"[s][\q{K}&&k]", Only(&[0x73]), Only(&[]), plain),
        (
            r"[\p{ASCII}][^\q{K|s}]",
            FoldedAscii,
            Except(&[0x4b, 0x6b, 0x212a, 0x53, 0x73, 0x17f]),
            folded,
        ),
        (
            r"[\p{ASCII}][^\q{K|s}]",
            Range(0, 0x7f),
            Except(&[0x4b, 0x73]),
            plain,
        ),
        (
            r"[\u{1f680}][\q{\u{1f600}|x}--x]",
            Only(&[0x1f680]),
            Only(&[0x1f600]),
            plain,
        ),
        (
            r"[\uD83D\uDE00][\q{\uD83D\uDE80}]",
            Only(&[0x1f600]),
            Only(&[0x1f680]),
            plain,
        ),
        ("[😀][\\q{🚀}]", Only(&[0x1f600]), Only(&[0x1f680]), plain),
        (
            r"[\\][\q{\uD800|\uDC00|x}--x]",
            Only(&[92]),
            Only(&[0xd800, 0xdc00]),
            plain,
        ),
        (
            r"[\$][\q{\]|\}}]",
            Only(&[0x24]),
            Only(&[0x5d, 0x7d]),
            plain,
        ),
        (r"[a][\q{\[|\\}]", Only(&[0x61]), Only(&[0x5b, 92]), plain),
        (
            r"[\u{00000061}][\q{a}]",
            Only(&[0x61]),
            Only(&[0x61]),
            plain,
        ),
        (r"^[a][\q{b}]$", Only(&[0x61]), Only(&[0x62]), anchored),
        (
            r"^[a][\q{b}]$",
            Only(&[0x61]),
            Only(&[0x62]),
            Gates {
                multiline: true,
                ..anchored
            },
        ),
        (
            r"[a][\q{a}]\b",
            Only(&[0x61]),
            Only(&[0x61]),
            Gates {
                words: [None, Some(true)],
                ..plain
            },
        ),
        (
            r"\b[a][\q{a}]\B",
            Only(&[0x61]),
            Only(&[0x61]),
            Gates {
                words: [Some(true), Some(false)],
                ..plain
            },
        ),
        (
            r"^[\$][\q{\]|\\|\[}]",
            Only(&[0x24]),
            Only(&[0x5d, 92, 0x5b]),
            Gates {
                anchors: [true, false],
                multiline: true,
                ..plain
            },
        ),
        (
            r"^\b[\p{ASCII}][\q{\!}\p{Any}]\B$",
            FoldedAscii,
            Except(&[]),
            Gates {
                ignore_case: true,
                words: [Some(true), Some(false)],
                ..anchored
            },
        ),
        (
            r"^\b[\p{ASCII}][\q{\!}\p{Any}]\B$",
            Range(0, 0x7f),
            Except(&[]),
            Gates {
                words: [Some(true), Some(false)],
                ..anchored
            },
        ),
        (
            r"^\b[a][\q{b}]\b$",
            Only(&[0x61]),
            Only(&[0x62]),
            Gates {
                multiline: true,
                words: [Some(true), Some(true)],
                ..anchored
            },
        ),
        (
            r"[\p{ASCII}][\q{a|b}]",
            Range(0, 0x7f),
            Only(&[0x61, 0x62]),
            plain,
        ),
        (
            r"[\P{ASCII}][\q{a|b}]",
            Outside(0, 0x7f),
            Only(&[0x61, 0x62]),
            plain,
        ),
    ];
    assert_eq!(cases.len(), 28);
    let mut queries = 0;
    for (source, first, second, gates) in cases {
        let original = compile(source, gates.ignore_case, gates.multiline).unwrap();
        let matcher = original.clone();
        drop(original);
        for input in &inputs {
            for start in 0..=input.text.len() + 1 {
                for sticky in [false, true] {
                    assert_eq!(
                        matcher.find(&input.text, start, sticky),
                        expected(first, second, gates, input, start, sticky),
                        "{source} input={:?} start={start} sticky={sticky}",
                        input.text
                    );
                    queries += 1;
                }
            }
        }
    }
    // Eight tiny inputs contain49 units:2*(49+2*8)=130 queries per plan.
    assert_eq!(queries, 3640);
    for (source, input, start, sticky, i, m, wanted) in [
        (r"[a][\q{b}]", "aab!", 0, false, false, false, Some(1..3)),
        (r"[a][\q{a}]\b", "aaa!", 0, false, false, false, Some(1..3)),
        (r"[a][\q{a}]\b", "aaa!", 0, true, false, false, None),
        (r"[a][\q{a}]\b", "aaa!", 1, true, false, false, Some(1..3)),
        (
            r"^[\$][\q{\]|\\|\[}]",
            "zxy\r\n$]tail\u{2028}",
            0,
            false,
            false,
            true,
            Some(5..7),
        ),
        (
            r"^\b[\p{ASCII}][\q{\!}\p{Any}]\B$",
            "K$",
            0,
            true,
            true,
            false,
            Some(0..2),
        ),
        (
            r"^\b[\p{ASCII}][\q{\!}\p{Any}]\B$",
            "K$",
            0,
            true,
            false,
            false,
            None,
        ),
        (
            r"\B[\p{ASCII}][^\q{K|s}]\B",
            "KſA$",
            0,
            false,
            true,
            false,
            Some(2..4),
        ),
        (
            r"[\uD83D\uDE00][\q{\uD83D\uDE80}]",
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
fn flat_class_then_single_character_string_domains_and_work_prefixes() {
    for source in [
        "",
        "ab",
        "[a][b]",
        "[[a]][b]",
        r"[a][[b]]",
        r"[a--a][\q{b}]",
        r"[[a]][\q{b}]",
        r"[\q{a}][b]",
        r"[\q{a}][\q{b}]",
        r"[a]",
        r"a[\q{b}]",
        r"[a][\q{b}][c]",
        r"([a])[\q{b}]",
        r"[a]([\q{b}])",
        r"([a][\q{b}])",
        r"[a]+[\q{b}]",
        r"[a][\q{b}]+",
        r"[a][\q{b}]?",
        r"[a][\q{b}]|c",
        r"\b^[a][\q{b}]",
        r"[a]\b[\q{b}]",
        r"[a][\q{b}]$\b",
        r"\b\b[a][\q{b}]",
        r"[a][\q{b}]\b\b",
        r"[a][\q{b}]$$",
        r"^^[a][\q{b}]",
        r"[a]^[\q{b}]",
        r"[a][\q{\u{D800}\u{DC00}}]",
        r"[][\q{}]",
        r"[][\q{|a}]",
        r"[][\q{a|bc}]",
        r"[][a--a--\q{a|bc}]",
        r"[][\q{a}&&[]&&\p{RGI_Emoji}]",
        r"[\p{RGI_Emoji}][\q{a}]",
    ] {
        assert!(compile(source, true, true).is_none(), "{source}");
    }
    let mut zero = Vec::new();
    assert!(
        recording(&JsString::from(""), false, false, &mut zero, None)
            .unwrap()
            .is_none()
    );
    assert_eq!(zero, [0]);
    assert!(matches!(
        recording(&JsString::from(""), false, false, &mut Vec::new(), Some(1)),
        Err(RegExpUnicodeSetCompileError::Work(1))
    ));
    // Independent child transcripts specify full flat-first then checked-second
    // work, including every late leaf behind an empty prefix and each abort.
    for (source, first, second, i, m, accepted) in [
        (
            r"^\b[a][\q{a|b}&&[b]]\b$",
            "[a]",
            r"[\q{a|b}&&[b]]",
            false,
            false,
            true,
        ),
        (
            r"[][a--a--\q{a}]",
            "[]",
            r"[a--a--\q{a}]",
            false,
            false,
            true,
        ),
        (
            r"^\b[\p{ASCII}][\q{K}&&k]\b$",
            r"[\p{ASCII}]",
            r"[\q{K}&&k]",
            true,
            false,
            true,
        ),
        (
            r"[\$][\q{\]|\\|\[}]",
            r"[\$]",
            r"[\q{\]|\\|\[}]",
            false,
            true,
            true,
        ),
        (
            r"[][\q{a}--a--\q{b|cd}]",
            "[]",
            r"[\q{a}--a--\q{b|cd}]",
            false,
            false,
            false,
        ),
        (
            r"[][\q{a}&&[]&&\p{RGI_Emoji}]",
            "[]",
            r"[\q{a}&&[]&&\p{RGI_Emoji}]",
            false,
            false,
            false,
        ),
        (r"\b[][\q{a|}]\B", "[]", r"[\q{a|}]", false, false, false),
        (
            r"[][a--a--\q{a|bc}]",
            "[]",
            r"[a--a--\q{a|bc}]",
            false,
            false,
            false,
        ),
    ] {
        let source = JsString::from(source);
        let mut expected = vec![source.len().saturating_mul(4)];
        let first = JsString::from(first);
        let second = JsString::from(second);
        let component = RegExpUnicodeClassMatcher::compile_flat_v_set_units_with_work(
            first.code_units(),
            i,
            m,
            |amount| {
                expected.push(amount);
                Ok::<(), usize>(())
            },
        )
        .unwrap();
        let proved = component.is_some() && RegExpUnicodeClassMatcher::compile_v_set_units_with_single_character_strings_and_work(
            second.code_units(), i, m, |amount| { expected.push(amount); Ok::<(), usize>(()) },
        ).unwrap().is_some();
        assert_eq!(proved, accepted);
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
    let source = JsString::from(r"\b[a][\q{a|b}]\b");
    assert_eq!(
        class_end(JsString::from(r"[\q{a|b}]").code_units(), 0),
        None
    );
    macro_rules! old_none {
        ($method:ident) => {
            assert!(
                RegExpUnicodeClassPairMatcher::$method(&source, false, false, |_| Ok::<(), ()>(()))
                    .unwrap()
                    .is_none()
            );
        };
    }
    old_none!(compile_v_set_with_work);
    old_none!(compile_v_set_with_word_boundaries_and_work);
    old_none!(compile_v_set_then_flat_with_work);
    old_none!(compile_flat_then_v_set_with_work);
    old_none!(compile_v_set_then_flat_with_word_boundaries_and_work);
    old_none!(compile_flat_then_v_set_with_word_boundaries_and_work);
    old_none!(compile_flat_pair_with_work);
    old_none!(compile_flat_pair_with_word_boundaries_and_work);
    old_none!(compile_v_set_then_flat_with_single_character_strings_and_work);
    for (source, input, start, sticky, wanted) in [
        (r"[a][\q{b}]", "aab!", 0, false, Some(1..3)),
        (r"[a][\q{a}]\b", "aaa!", 0, true, None),
        (
            r"[\uD83D\uDE00][\q{\uD83D\uDE80}]",
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
