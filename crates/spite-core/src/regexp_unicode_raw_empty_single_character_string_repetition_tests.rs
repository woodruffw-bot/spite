use super::{JsString, RegExpUnicodeRepeatedAtomMatch, RegExpUnicodeRepeatedAtomMatcher};
use crate::RegExpUnicodeSetCompileError;
use std::ops::Range;

#[derive(Clone, Copy)]
enum Set {
    Only(&'static [u32]),
    Except(&'static [u32]),
}
impl Set {
    fn contains(self, p: u32) -> bool {
        match self {
            Self::Only(a) => a.contains(&p),
            Self::Except(a) => !a.contains(&p),
        }
    }
}
#[derive(Clone, Copy, Default)]
struct Gates {
    anchors: [bool; 2],
    words: [Option<bool>; 2],
    ignore_case: bool,
    multiline: bool,
}
#[derive(Clone, Copy)]
struct Repeat {
    minimum: usize,
    maximum: Option<usize>,
    greedy: bool,
    empty_first: bool,
    captures: usize,
}
struct Case<'a> {
    source: &'a str,
    set: Set,
    repeat: Repeat,
    gates: Gates,
}
struct Input {
    text: JsString,
    points: Vec<u32>,
    bounds: Vec<usize>,
}
impl Input {
    fn new(text: JsString) -> Self {
        let mut points = Vec::new();
        let mut bounds = vec![0];
        for d in char::decode_utf16(text.code_units().iter().copied()) {
            let (p, n) = match d {
                Ok(c) => (u32::from(c), c.len_utf16()),
                Err(e) => (u32::from(e.unpaired_surrogate()), 1),
            };
            points.push(p);
            bounds.push(bounds.last().unwrap() + n);
        }
        Self {
            text,
            points,
            bounds,
        }
    }
    fn boundary(&self, at: usize, i: bool) -> bool {
        let word = |p| {
            matches!(p,0x30..=0x39|0x41..=0x5a|0x5f|0x61..=0x7a)
                || (i && matches!(p, 0x17f | 0x212a))
        };
        self.points.get(at).is_some_and(|&p| word(p))
            != at
                .checked_sub(1)
                .and_then(|n| self.points.get(n))
                .is_some_and(|&p| word(p))
    }
}
fn line(p: u32) -> bool {
    matches!(p, 10 | 13 | 0x2028 | 0x2029)
}

// A tiny independent RepeatMatcher interpreter. No artificial iteration budget:
// every call consumes a character or reduces the sole mandatory minimum to 0.
// Optional empty iterations are rejected before their continuation.
fn repeat(
    case: &Case<'_>,
    input: &Input,
    begin: usize,
    at: usize,
    minimum: usize,
    maximum: Option<usize>,
    capture: Option<Range<usize>>,
) -> Option<RegExpUnicodeRepeatedAtomMatch> {
    let continuation = || {
        (minimum == 0
            && (!case.gates.anchors[1]
                || at == input.points.len()
                || (case.gates.multiline && line(input.points[at])))
            && case.gates.words[1]
                .is_none_or(|wanted| input.boundary(at, case.gates.ignore_case) == wanted))
        .then(|| RegExpUnicodeRepeatedAtomMatch {
            range: input.bounds[begin]..input.bounds[at],
            atom_capture: capture.clone(),
        })
    };
    if !case.repeat.greedy {
        if let Some(found) = continuation() {
            return Some(found);
        }
    }
    if maximum != Some(0) {
        for empty in [case.repeat.empty_first, !case.repeat.empty_first] {
            let next = if empty {
                at
            } else if input.points.get(at).is_some_and(|&p| case.set.contains(p)) {
                at + 1
            } else {
                continue;
            };
            if minimum == 0 && next == at {
                continue;
            }
            let captured =
                (case.repeat.captures != 0).then(|| input.bounds[at]..input.bounds[next]);
            if let Some(found) = repeat(
                case,
                input,
                begin,
                next,
                minimum.saturating_sub(1),
                maximum.map(|n| n - 1),
                captured,
            ) {
                return Some(found);
            }
        }
    }
    if case.repeat.greedy {
        continuation()
    } else {
        None
    }
}
fn expected(
    case: &Case<'_>,
    input: &Input,
    start: usize,
    sticky: bool,
) -> Option<RegExpUnicodeRepeatedAtomMatch> {
    if start > input.text.len() {
        return None;
    }
    let first = input.bounds.partition_point(|&n| n <= start) - 1;
    for at in first..=input.points.len() {
        if sticky && at != first {
            break;
        }
        if (case.gates.anchors[0]
            && at != 0
            && !(case.gates.multiline && line(input.points[at - 1])))
            || !case.gates.words[0]
                .is_none_or(|wanted| input.boundary(at, case.gates.ignore_case) == wanted)
        {
            continue;
        }
        if let Some(found) = repeat(
            case,
            input,
            at,
            at,
            case.repeat.minimum,
            case.repeat.maximum,
            None,
        ) {
            return Some(found);
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
    RegExpUnicodeRepeatedAtomMatcher::compile_nullable_v_set_with_single_character_strings_and_work(
        source,
        i,
        m,
        |n| {
            work.push(n);
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
fn raw_empty_single_character_string_repetition_matches_independent_repeat_interpreter() {
    use Set::{Except, Only};
    let inputs = [
        JsString::from(""),
        JsString::from("aaa!"),
        JsString::from("ba"),
        JsString::from("A😀😀!"),
        JsString::from_code_units(vec![65, 0xd800, 0xd800, 33]),
        JsString::from("x\naa\nz"),
        JsString::from("KKkſs!"),
    ]
    .map(Input::new);
    let base = Repeat {
        minimum: 0,
        maximum: None,
        greedy: true,
        empty_first: false,
        captures: 1,
    };
    let plain = Gates::default();
    let rows = [
        (r"((?:[\q{a}]|))*", Only(&[97]), base, plain),
        (
            r"(?:(|[\q{a}]))*",
            Only(&[97]),
            Repeat {
                empty_first: true,
                ..base
            },
            plain,
        ),
        (
            r"([\q{a}]|)*?",
            Only(&[97]),
            Repeat {
                greedy: false,
                ..base
            },
            plain,
        ),
        (
            r"(|[\q{a}])*?",
            Only(&[97]),
            Repeat {
                greedy: false,
                empty_first: true,
                ..base
            },
            plain,
        ),
        (
            r"(([\q{a}]|))+",
            Only(&[97]),
            Repeat {
                minimum: 1,
                captures: 2,
                ..base
            },
            plain,
        ),
        (
            r"(|[\q{a}])+",
            Only(&[97]),
            Repeat {
                minimum: 1,
                empty_first: true,
                ..base
            },
            plain,
        ),
        (
            r"([\q{a}]|)+?",
            Only(&[97]),
            Repeat {
                minimum: 1,
                greedy: false,
                ..base
            },
            plain,
        ),
        (
            r"(|[\q{a}])+?",
            Only(&[97]),
            Repeat {
                minimum: 1,
                greedy: false,
                empty_first: true,
                ..base
            },
            plain,
        ),
        (
            r"([\q{a}]|)?",
            Only(&[97]),
            Repeat {
                maximum: Some(1),
                ..base
            },
            plain,
        ),
        (
            r"(|[\q{a}])?",
            Only(&[97]),
            Repeat {
                maximum: Some(1),
                empty_first: true,
                ..base
            },
            plain,
        ),
        (
            r"([\q{a}]|)??",
            Only(&[97]),
            Repeat {
                maximum: Some(1),
                greedy: false,
                ..base
            },
            plain,
        ),
        (
            r"(?:|[\q{a}])??",
            Only(&[97]),
            Repeat {
                maximum: Some(1),
                greedy: false,
                empty_first: true,
                captures: 0,
                ..base
            },
            plain,
        ),
        (
            r"([\q{a}--a]|)+",
            Only(&[]),
            Repeat { minimum: 1, ..base },
            plain,
        ),
        (
            r"([^\q{K|s}]|)+?\B",
            Except(&[75, 107, 8490, 83, 115, 383]),
            Repeat {
                minimum: 1,
                greedy: false,
                ..base
            },
            Gates {
                ignore_case: true,
                words: [None, Some(false)],
                ..plain
            },
        ),
        (
            r"([\q{\u{1f600}}]|)+",
            Only(&[0x1f600]),
            Repeat { minimum: 1, ..base },
            plain,
        ),
        (
            r"\b([\q{\uD800|x}--x]|)+\B",
            Only(&[0xd800]),
            Repeat { minimum: 1, ..base },
            Gates {
                words: [Some(true), Some(false)],
                ..plain
            },
        ),
    ];
    assert_eq!(rows.len(), 16);
    let mut queries = 0;
    for (source, set, repeat, gates) in rows {
        let case = Case {
            source,
            set,
            repeat,
            gates,
        };
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
                        expected(&case, input, start, sticky),
                        "{} input={:?} start={start} sticky={sticky}",
                        case.source,
                        input.text
                    );
                    queries += 1;
                }
            }
        }
    }
    // Seven inputs contain 28 original units: 2*(28+2*7)=84 queries per plan.
    assert_eq!(queries, 1344);
    for (source, input, start, sticky, m, range, capture) in [
        (r"([\q{a}]|)+?\b", "aaa!", 0, false, false, 0..3, Some(2..3)),
        (r"(|[\q{a}])+?\b", "aaa!", 0, false, false, 0..0, Some(0..0)),
        (r"([\q{a}]|)+?", "ba", 0, false, false, 0..0, Some(0..0)),
        (r"([\q{a}]|)*?", "b", 0, false, false, 0..0, None),
        (
            r"^([\q{a}]|)+$",
            "x\naa\nz",
            0,
            false,
            true,
            2..4,
            Some(3..4),
        ),
        (
            r"\b([\q{\u{1f600}}]|)+\B",
            "A😀😀!",
            2,
            true,
            false,
            1..5,
            Some(3..5),
        ),
    ] {
        assert_eq!(
            compile(source, false, m)
                .unwrap()
                .find_with_captures_and_work(
                    &JsString::from(input),
                    start,
                    sticky,
                    |_| Ok::<(), ()>(())
                )
                .unwrap(),
            Some(RegExpUnicodeRepeatedAtomMatch {
                range,
                atom_capture: capture
            }),
            "{source}"
        );
    }
}

#[test]
fn raw_empty_single_character_string_repetition_full_proofs_and_cancellation() {
    for source in [
        r"([a]|)+",
        r"([\q{a}])+",
        r"[\q{a}]|",
        r"(|[\q{a}]|)+",
        r"([\q{a}]||)+",
        r"(||[\q{a}])+",
        r"(([\q{a}])|)+",
        r"(|([\q{a}]))+",
        r"([\q{a}]|(?:))+",
        r"([\q{a}]|\b)+",
        r"(\b[\q{a}]|)+",
        r"([^\q{a}]$|)+",
        r"([\q{a}]|b)+",
        r"([\q{a}]a|)+",
        r"(?i:[\q{a}]|)+",
        r"(?<x>[\q{a}]|)+",
        r"([\q{a}]|){0}",
        r"([\q{a}]|){1,}",
        r"([\q{a}]|){1}",
        r"([\q{a}]|){999999999999999999999999}",
        r"([\q{a|bc}]|)*",
        r"(|[\q{}])+?",
        r"(|[\q{a}&&[]&&\q{bc}])+?",
        r"([\q{a}--a--\p{RGI_Emoji}]|)*",
        r"([\q{a}]|)+a",
        r"\b^([\q{a}]|)+",
        r"([\q{a}]|)+$$",
        r"\b\b([\q{a}]|)+",
        r"([\q{a}]|)+\b\b",
    ] {
        assert!(compile(source, true, true).is_none(), "{source}");
    }
    for (source, i, accepted) in [
        (r"((?:[\q{a|b}]|))*", false, true),
        (r"^\b(|[\q{K}&&k])+?\B$", true, true),
        (r"([\q{a}--a]|)+", false, true),
        (r"([\q{\|}]|)*", false, true),
        (r"(|[\q{}])+?", false, false),
        (r"([\q{a}--a--\p{RGI_Emoji}]|)*", false, false),
    ] {
        let source = JsString::from(source);
        let mut complete = Vec::new();
        assert_eq!(
            recording(&source, i, false, &mut complete, None)
                .unwrap()
                .is_some(),
            accepted
        );
        assert_eq!(complete.first(), Some(&source.len().saturating_mul(10)));
        assert!(complete.len() > 1);
        for fail in 1..=complete.len() {
            let mut seen = Vec::new();
            let result = recording(&source, i, false, &mut seen, Some(fail));
            assert!(matches!(result,Err(RegExpUnicodeSetCompileError::Work(at))if at==fail));
            assert_eq!(seen, complete[..fail]);
        }
    }
    for (source, input, range, capture) in [
        (r"([\q{a}]|)+?\b", "aaa!", 0..3, Some(2..3)),
        (r"(|[\q{a}])+?\b", "aaa!", 0..0, Some(0..0)),
        (r"([\q{a}]|)*?", "a", 0..0, None),
    ] {
        let matcher = compile(source, false, false).unwrap();
        let input = JsString::from(input);
        let mut complete = Vec::new();
        assert_eq!(
            matcher
                .find_with_captures_and_work(&input, 0, false, |n| {
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
            let result = matcher.find_with_captures_and_work(&input, 0, false, |n| {
                seen.push(n);
                if seen.len() == fail {
                    Err(fail)
                } else {
                    Ok(())
                }
            });
            assert!(matches!(result,Err(at)if at==fail));
            assert_eq!(seen, complete[..fail]);
        }
    }
}
