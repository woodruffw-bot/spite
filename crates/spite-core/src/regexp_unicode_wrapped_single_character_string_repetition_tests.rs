use super::{JsString, RegExpUnicodeRepeatedAtomMatch, RegExpUnicodeRepeatedAtomMatcher};
use crate::RegExpUnicodeSetCompileError;

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
#[derive(Clone, Copy)]
struct Repeat {
    minimum: Option<usize>,
    maximum: Option<usize>,
    greedy: bool,
    captures: usize,
}
#[derive(Clone, Copy, Default)]
struct Gates {
    anchors: [bool; 2],
    words: [Option<bool>; 2],
    ignore_case: bool,
    multiline: bool,
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
        for decoded in char::decode_utf16(text.code_units().iter().copied()) {
            let (point, width) = match decoded {
                Ok(c) => (u32::from(c), c.len_utf16()),
                Err(e) => (u32::from(e.unpaired_surrogate()), 1),
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

// Handwritten final memberships, bounds, capture counts and gates are separate
// from Pattern parsing, production fold tables and incremental-run matching.
fn expected(
    set: Set,
    repeat: Repeat,
    gates: Gates,
    input: &Input,
    start: usize,
    sticky: bool,
) -> Option<RegExpUnicodeRepeatedAtomMatch> {
    let minimum = repeat.minimum?;
    if start > input.text.len() {
        return None;
    }
    let first = input.boundaries.partition_point(|&at| at <= start) - 1;
    let word = |p| {
        matches!(p, 0x30..=0x39 | 0x41..=0x5a | 0x5f | 0x61..=0x7a)
            || (gates.ignore_case && matches!(p, 0x17f | 0x212a))
    };
    let boundary = |at: usize| {
        input.points.get(at).is_some_and(|&p| word(p))
            != at
                .checked_sub(1)
                .and_then(|at| input.points.get(at))
                .is_some_and(|&p| word(p))
    };
    let line = |p| matches!(p, 0xa | 0xd | 0x2028 | 0x2029);
    for begin in first..=input.points.len() {
        if sticky && begin != first {
            break;
        }
        if (gates.anchors[0] && begin != 0 && !(gates.multiline && line(input.points[begin - 1])))
            || !gates.words[0].is_none_or(|wanted| boundary(begin) == wanted)
        {
            continue;
        }
        // Enumerate each finite candidate count and prove its consumed slice.
        // This deliberately does not compute the production sliding-run state.
        let upper = repeat
            .maximum
            .unwrap_or(input.points.len() - begin)
            .min(input.points.len() - begin);
        if minimum > upper {
            continue;
        }
        for choice in minimum..=upper {
            let count = if repeat.greedy {
                upper - (choice - minimum)
            } else {
                choice
            };
            let end = begin + count;
            if input.points[begin..end].iter().all(|&p| set.contains(p))
                && (!gates.anchors[1]
                    || end == input.points.len()
                    || (gates.multiline && line(input.points[end])))
                && gates.words[1].is_none_or(|wanted| boundary(end) == wanted)
            {
                return Some(RegExpUnicodeRepeatedAtomMatch {
                    range: input.boundaries[begin]..input.boundaries[end],
                    atom_capture: (repeat.captures != 0 && count != 0)
                        .then(|| input.boundaries[end - 1]..input.boundaries[end]),
                });
            }
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
) -> Result<Option<RegExpUnicodeRepeatedAtomMatcher>, RegExpUnicodeSetCompileError<usize>> {
    RegExpUnicodeRepeatedAtomMatcher::compile_wrapped_v_set_with_single_character_strings_and_work(
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
fn compile(source: &str, i: bool, m: bool) -> Option<RegExpUnicodeRepeatedAtomMatcher> {
    recording(&JsString::from(source), i, m, &mut Vec::new(), None).unwrap()
}

#[test]
fn wrapped_single_character_string_repetition_matches_independent_capture_oracle() {
    use Set::{Except, Only};
    let inputs = [
        JsString::from(""),
        JsString::from("aaab!"),
        JsString::from("KKkſs!"),
        JsString::from("A😀😀!"),
        JsString::from_code_units(vec![65, 0xd800, 0xd800, 33]),
        JsString::from("x\nab\nz()}"),
    ]
    .map(Input::new);
    let one = Repeat {
        minimum: Some(1),
        maximum: None,
        greedy: true,
        captures: 1,
    };
    let plain = Gates::default();
    let folded = Gates {
        ignore_case: true,
        ..plain
    };
    let words = Gates {
        words: [Some(false), Some(true)],
        ..plain
    };
    let cases = [
        (r"([\q{a|b}])+", Only(&[97, 98]), one, plain),
        (
            r"(?:[\q{a|b}])+?",
            Only(&[97, 98]),
            Repeat {
                greedy: false,
                captures: 0,
                ..one
            },
            plain,
        ),
        (
            r"((?:([\q{a|b}]))){1,2}",
            Only(&[97, 98]),
            Repeat {
                maximum: Some(2),
                captures: 2,
                ..one
            },
            plain,
        ),
        (
            r"((([\q{a}]))){0,2}?",
            Only(&[97]),
            Repeat {
                minimum: Some(0),
                maximum: Some(2),
                greedy: false,
                captures: 3,
            },
            plain,
        ),
        (
            r"([\q{a}--a]){0}",
            Only(&[]),
            Repeat {
                minimum: Some(0),
                maximum: Some(0),
                ..one
            },
            plain,
        ),
        (r"([\q{a}&&[]])+", Only(&[]), one, plain),
        (
            r"(([\q{K}&&k]))+",
            Only(&[75, 107, 8490]),
            Repeat { captures: 2, ..one },
            folded,
        ),
        (
            r"([^\q{K|s}]){1,2}",
            Except(&[75, 107, 8490, 83, 115, 383]),
            Repeat {
                maximum: Some(2),
                ..one
            },
            folded,
        ),
        (
            r"(?:([\q{\u{1f600}}])){1,2}",
            Only(&[0x1f600]),
            Repeat {
                maximum: Some(2),
                ..one
            },
            plain,
        ),
        (
            r"(([\q{\uD800|x}--x]))+",
            Only(&[0xd800]),
            Repeat { captures: 2, ..one },
            plain,
        ),
        (
            r"^([\q{a|b}])+$",
            Only(&[97, 98]),
            one,
            Gates {
                anchors: [true, true],
                multiline: true,
                ..plain
            },
        ),
        (
            r"\B([\q{a|b}]){1,2}\b",
            Only(&[97, 98]),
            Repeat {
                maximum: Some(2),
                ..one
            },
            words,
        ),
        (
            r"\B([\q{a|b}]){1,2}?\b",
            Only(&[97, 98]),
            Repeat {
                maximum: Some(2),
                greedy: false,
                ..one
            },
            words,
        ),
        (
            r"([\q{a}]){999999999999999999999999}",
            Only(&[97]),
            Repeat {
                minimum: None,
                ..one
            },
            plain,
        ),
        (
            r"([\q{a}]){0,999999999999999999999999}?",
            Only(&[97]),
            Repeat {
                minimum: Some(0),
                greedy: false,
                ..one
            },
            plain,
        ),
        (r"([\q{\(|\)|\}}])+", Only(&[40, 41, 125]), one, plain),
    ];
    assert_eq!(cases.len(), 16);
    let mut queries = 0;
    for (source, set, repeat, gates) in cases {
        let original = compile(source, gates.ignore_case, gates.multiline).unwrap();
        let matcher = original.clone();
        drop(original);
        assert_eq!(matcher.capture_count(), repeat.captures, "{source}");
        for input in &inputs {
            for start in 0..=input.text.len() + 1 {
                for sticky in [false, true] {
                    assert_eq!(
                        matcher
                            .find_with_captures_and_work(
                                &input.text,
                                start,
                                sticky,
                                |_| Ok::<(), ()>(())
                            )
                            .unwrap(),
                        expected(set, repeat, gates, input, start, sticky),
                        "{source} input={:?} start={start} sticky={sticky}",
                        input.text
                    );
                    queries += 1;
                }
            }
        }
    }
    // Six inputs contain30 original units: 2*(30+2*6)=84 queries per plan.
    assert_eq!(queries, 1344);
}

#[test]
fn wrapped_single_character_string_repetition_full_proofs_and_cancellation() {
    for source in [
        r"[\q{a}]+",
        r"([a])+",
        r"([\q{a}])",
        r"(?<x>[\q{a}])+",
        r"(?i:[\q{a}])+",
        r"(^[\q{a}])+",
        r"([\q{a}]$)+",
        r"(\b[\q{a}])+",
        r"([\q{a}]\b)+",
        r"([\q{a}])([\q{b}])+",
        r"(([\q{a}])([\q{b}]))+",
        r"([\q{a}]|[b])+",
        r"((?:[\q{a}])?)+",
        r"(([\q{a}])*)+",
        r"([\q{a}]a)+",
        r"([\q{a}]|)+",
        r"([\q{}]){0}",
        r"([\q{a|bc}]){0}",
        r"([\q{a}&&[]&&\q{bc}]){0}",
        r"([\q{a}--a--\p{RGI_Emoji}]){999999999999999999999999}",
        r"\b^([\q{a}])+",
        r"([\q{a}])+$\b",
        r"\b\b([\q{a}])+",
        r"([\q{a}])+\b\b",
        r"([\q{a}])+$$",
        r"([\q{a}])+a",
        r"(\u{1f600}[\q{a}])+",
    ] {
        assert!(compile(source, true, true).is_none(), "{source}");
    }
    for (source, i, accepted) in [
        (r"((?:([\q{a|b}]))){1,2}", false, true),
        (r"^\B(?:([\q{K}&&k])){1,2}?\b$", true, true),
        (r"([\q{a}--a]){0}", false, true),
        (r"([\q{a}&&[]&&\q{bc}]){0}", false, false),
        (
            r"([\q{a}--a--\p{RGI_Emoji}]){999999999999999999999999}",
            false,
            false,
        ),
        (r"([\q{}]){0}", false, false),
    ] {
        let source = JsString::from(source);
        let mut complete = Vec::new();
        assert_eq!(
            recording(&source, i, false, &mut complete, None)
                .unwrap()
                .is_some(),
            accepted
        );
        assert_eq!(complete.first(), Some(&source.len().saturating_mul(16)));
        assert!(complete.len() > 1);
        for fail in 1..=complete.len() {
            let mut seen = Vec::new();
            let result = recording(&source, i, false, &mut seen, Some(fail));
            assert!(matches!(result,Err(RegExpUnicodeSetCompileError::Work(at)) if at==fail));
            assert_eq!(seen, complete[..fail]);
        }
    }
    for (source, input, start, sticky, range, capture) in [
        (r"([\q{a|b}])+", "aaab!", 0, false, 0..4, Some(3..4)),
        (r"([\q{\u{1f600}}])+", "A😀😀!", 2, true, 1..5, Some(3..5)),
        (r"([\q{a}--a]){0}", "a", 0, true, 0..0, None),
    ] {
        let matcher = compile(source, false, false).unwrap();
        let input = JsString::from(input);
        let mut complete = Vec::new();
        assert_eq!(
            matcher
                .find_with_captures_and_work(&input, start, sticky, |n| {
                    complete.push(n);
                    Ok::<(), usize>(())
                })
                .unwrap(),
            Some(RegExpUnicodeRepeatedAtomMatch {
                range,
                atom_capture: capture
            })
        );
        for fail in 1..=complete.len() {
            let mut seen = Vec::new();
            let result = matcher.find_with_captures_and_work(&input, start, sticky, |n| {
                seen.push(n);
                if seen.len() == fail {
                    Err(fail)
                } else {
                    Ok(())
                }
            });
            assert!(matches!(result,Err(at) if at==fail));
            assert_eq!(seen, complete[..fail]);
        }
    }
}
