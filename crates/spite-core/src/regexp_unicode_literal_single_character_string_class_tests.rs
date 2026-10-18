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
        // Standard-library decoding is independent of matcher boundary helpers.
        for decoded in char::decode_utf16(text.code_units().iter().copied()) {
            let (point, width) = match decoded {
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

// Only the selected simple/common-fold facts needed by the literal oracle.
// Each class's final finite membership is handwritten separately below.
fn fold(point: u32) -> u32 {
    match point {
        0x41..=0x5a => point + 0x20,
        0x17f => 0x73,
        0x212a => 0x6b,
        _ => point,
    }
}

fn expected(
    literal: u32,
    set: Set,
    gates: Gates,
    input: &Input,
    start: usize,
    sticky: bool,
) -> Option<Range<usize>> {
    if start > input.text.len() {
        return None;
    }
    // The approved initial-low-pair behavior chooses its leading boundary.
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
        input.points.get(at).is_some_and(|&point| word(point))
            != at
                .checked_sub(1)
                .and_then(|at| input.points.get(at))
                .is_some_and(|&point| word(point))
    };
    let line = |point| matches!(point, 0xa | 0xd | 0x2028 | 0x2029);
    // Enumerate adjacent decoded pairs, rather than imitating production retries.
    for at in first..input.points.len().saturating_sub(1) {
        if sticky && at != first {
            break;
        }
        if canonical(input.points[at]) == canonical(literal)
            && set.contains(input.points[at + 1])
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
    RegExpUnicodeClassLiteralMatcher::compile_literal_then_v_set_with_single_character_strings_and_work(
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
fn literal_single_character_string_class_matches_independent_utf16_pairs() {
    use Set::{Except, Only};
    let inputs = [
        JsString::from(""),
        JsString::from("aabaa!aaa!"),
        JsString::from("ſKsK!ks"),
        JsString::from("A😀😁!"),
        JsString::from_code_units(vec![65, 0xd800, 0xd800, 33, 0xdc00, 0xdc00]),
        JsString::from("x\r\nab\u{2028}aa"),
        JsString::from("$]$}[[[]!"),
        JsString::from("$K"),
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
        (r"a[\q{a|b}]", 0x61, Only(&[0x61, 0x62]), plain),
        (r"a[\q{b|b}&&[b-z]]", 0x61, Only(&[0x62]), plain),
        (r"a[\q{a|b}--a]", 0x61, Only(&[0x62]), plain),
        (r"a[[\q{a|b}]--[\q{a}]]", 0x61, Only(&[0x62]), plain),
        (r"a[[^\q{a}]&&[\q{a|b}]]", 0x61, Only(&[0x62]), plain),
        (r"a[^\q{a|b}]", 0x61, Except(&[0x61, 0x62]), plain),
        (r"a[\q{b}--b]", 0x61, Only(&[]), plain),
        (r"s[\q{K}&&k]", 0x73, Only(&[0x4b, 0x6b, 0x212a]), folded),
        (r"s[\q{K}&&k]", 0x73, Only(&[]), plain),
        (
            r"s[^\q{K|s}]",
            0x73,
            Except(&[0x4b, 0x6b, 0x212a, 0x53, 0x73, 0x17f]),
            folded,
        ),
        (r"s[^\q{K|s}]", 0x73, Except(&[0x4b, 0x73]), plain),
        (
            r"\u{1f600}[\q{\u{1f601}|x}--x]",
            0x1f600,
            Only(&[0x1f601]),
            plain,
        ),
        (
            r"\uD83D\uDE00[\q{\uD83D\uDE01}]",
            0x1f600,
            Only(&[0x1f601]),
            plain,
        ),
        (r"😀[\q{😁}]", 0x1f600, Only(&[0x1f601]), plain),
        (r"\uD800[\q{\uD800|x}--x]", 0xd800, Only(&[0xd800]), plain),
        (r"\$[\q{\]|\}}]", 0x24, Only(&[0x5d, 0x7d]), plain),
        (r"\[[\q{\[|\]}]", 0x5b, Only(&[0x5b, 0x5d]), plain),
        (r"^a[\q{b}]$", 0x61, Only(&[0x62]), anchored),
        (
            r"^a[\q{b}]$",
            0x61,
            Only(&[0x62]),
            Gates {
                multiline: true,
                ..anchored
            },
        ),
        (
            r"\Ba[\q{a}]\b",
            0x61,
            Only(&[0x61]),
            Gates {
                words: [Some(false), Some(true)],
                ..plain
            },
        ),
        (
            r"\ba[\q{a}]\B",
            0x61,
            Only(&[0x61]),
            Gates {
                words: [Some(true), Some(false)],
                ..plain
            },
        ),
        (
            r"^\B\$[\q{\!}\p{Any}]\b$",
            0x24,
            Except(&[]),
            Gates {
                ignore_case: true,
                words: [Some(false), Some(true)],
                ..anchored
            },
        ),
        (
            r"^\B\$[\q{\!}\p{Any}]\b$",
            0x24,
            Except(&[]),
            Gates {
                words: [Some(false), Some(true)],
                ..anchored
            },
        ),
        (
            r"^\ba[\q{a}]\b$",
            0x61,
            Only(&[0x61]),
            Gates {
                multiline: true,
                words: [Some(true), Some(true)],
                ..anchored
            },
        ),
    ];
    assert_eq!(cases.len(), 24);
    let mut queries = 0;
    for (source, literal, set, gates) in cases {
        let original = compile(source, gates.ignore_case, gates.multiline).unwrap();
        let matcher = original.clone();
        drop(original);
        for input in &inputs {
            for start in 0..=input.text.len() + 1 {
                for sticky in [false, true] {
                    assert_eq!(
                        matcher.find(&input.text, start, sticky),
                        expected(literal, set, gates, input, start, sticky),
                        "{source} input={:?} start={start} sticky={sticky}",
                        input.text,
                    );
                    queries += 1;
                }
            }
        }
    }
    // Eight tiny inputs contain48 units: 2*(48+2*8)=128 queries per plan.
    assert_eq!(queries, 3072);
    for (source, input, start, sticky, i, expected) in [
        (r"a[\q{b}]", "aab!", 0, false, false, Some(1..3)),
        (r"\Ba[\q{a}]\b", "aaa!", 0, false, false, Some(1..3)),
        (r"^\B\$[\q{\!}\p{Any}]\b$", "$K", 0, true, true, Some(0..2)),
        (r"^\B\$[\q{\!}\p{Any}]\b$", "$K", 0, true, false, None),
        (
            r"\u{1f600}[\q{\u{1f601}}]",
            "A😀😁!",
            2,
            true,
            false,
            Some(1..5),
        ),
    ] {
        assert_eq!(
            compile(source, i, false)
                .unwrap()
                .find(&JsString::from(input), start, sticky),
            expected,
            "{source}"
        );
    }
}

#[test]
fn literal_single_character_string_class_domains_and_work_prefixes() {
    for source in [
        "",
        "ab",
        "a[b]",
        "a[[b]]",
        r"a[\p{Any}]",
        r"[\q{a}]b",
        r"aa[\q{b}]",
        r"a[\q{b}]c",
        r"a[\q{b}][c]",
        r"[a][\q{b}]",
        r"(a)[\q{b}]",
        r"a([\q{b}])",
        r"(a[\q{b}])",
        r"a+[\q{b}]",
        r"a[\q{b}]+",
        r"a[\q{b}]?",
        r"a[\q{b}]{0}",
        r"a[\q{b}]|c",
        r"a\b[\q{b}]",
        r"\b^a[\q{b}]",
        r"a[\q{b}]$\b",
        r"\b\ba[\q{b}]",
        r"a[\q{b}]\b\b",
        r"a[\q{b}]$$",
        r"^^a[\q{b}]",
        r"a^[\q{b}]",
        r"a[\q{b}]\$",
        r"a[\q{b}]\\b",
        r"a[\q{b}]]",
        r"\u{D800}\u{DC00}[\q{a}]",
        r"a[\q{\u{D800}\u{DC00}}]",
        r"a[\q{}]",
        r"a[\q{|b}]",
        r"a[\q{b|cd}]",
        r"a[\q{a}--a--\q{b|cd}]",
        r"a[\q{a}&&[]&&\p{RGI_Emoji}]",
        r"a[\p{RGI_Emoji}]",
    ] {
        assert!(compile(source, true, true).is_none(), "{source}");
    }
    for (source, i, m, accepted) in [
        (r"^\ba[\q{a|b}&&[b]]\b$", false, false, true),
        (r"^\B\$[\q{\!}\p{Any}]\b$", true, false, true),
        (r"\u{1f600}[\q{\uD83D\uDE01|x}--x]", false, false, true),
        (r"a[\q{a}--a--\q{b|cd}]", false, false, false),
        (r"a[\q{a}&&[]&&\p{RGI_Emoji}]", false, false, false),
        (r"a[\q{a|}]", false, false, false),
    ] {
        let source = JsString::from(source);
        let mut complete = Vec::new();
        assert_eq!(
            recording(&source, i, m, &mut complete, None)
                .unwrap()
                .is_some(),
            accepted
        );
        assert_eq!(complete.first(), Some(&source.len().saturating_mul(3)));
        assert!(complete.len() > 2);
        for fail in 1..=complete.len() {
            let mut seen = Vec::new();
            let result = recording(&source, i, m, &mut seen, Some(fail));
            assert!(matches!(result, Err(RegExpUnicodeSetCompileError::Work(at)) if at == fail));
            assert_eq!(seen, complete[..fail]);
        }
    }
    for (source, input, start, sticky, wanted) in [
        (r"a[\q{b}]", "aab!", 0, false, Some(1..3)),
        (r"\Ba[\q{a}]\b", "aaa!", 0, true, None),
        (r"\u{1f600}[\q{\u{1f601}}]", "A😀😁!", 2, true, Some(1..5)),
    ] {
        let matcher = compile(source, false, false).unwrap();
        let input = JsString::from(input);
        let mut complete = Vec::new();
        assert_eq!(
            matcher
                .find_with_work(&input, start, sticky, |work| {
                    complete.push(work);
                    Ok::<(), usize>(())
                })
                .unwrap(),
            wanted
        );
        assert_eq!(complete, [1]);
        assert!(matches!(
            matcher.find_with_work(&input, start, sticky, |_| Err(7)),
            Err(7)
        ));
    }
}
