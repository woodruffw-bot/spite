use super::{JsString, RegExpUnicodeRepeatedAtomMatch, RegExpUnicodeRepeatedAtomMatcher};
use crate::RegExpUnicodeSetCompileError;
use std::ops::Range;

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

// Resulting finite memberships are supplied by hand for each expression below.
// The oracle neither parses its Pattern nor reads a production fold/set table.
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
struct Bounds {
    minimum: Option<usize>,
    maximum: Option<usize>,
    greedy: bool,
}

#[derive(Clone, Copy, Default)]
struct Gates {
    anchors: [bool; 2],
    words: [Option<bool>; 2],
    ignore_case: bool,
    multiline: bool,
}

fn expected(
    set: Set,
    bounds: Bounds,
    gates: Gates,
    input: &Input,
    start: usize,
    sticky: bool,
) -> Option<Range<usize>> {
    let minimum = bounds.minimum?;
    if start > input.text.len() {
        return None;
    }
    // The approved inside-pair behavior uses the original leading boundary.
    let first = input.boundaries.partition_point(|&at| at <= start) - 1;
    let word = |point| {
        matches!(point, 0x30..=0x39 | 0x41..=0x5a | 0x5f | 0x61..=0x7a)
            || (gates.ignore_case && matches!(point, 0x17f | 0x212a))
    };
    let boundary = |at: usize| {
        let left = at.checked_sub(1).and_then(|at| input.points.get(at));
        let right = input.points.get(at);
        left.is_some_and(|&point| word(point)) != right.is_some_and(|&point| word(point))
    };
    let line = |point| matches!(point, 0xa | 0xd | 0x2028 | 0x2029);
    for begin in first..=input.points.len() {
        if sticky && begin != first {
            break;
        }
        if (gates.anchors[0] && begin != 0 && !(gates.multiline && line(input.points[begin - 1])))
            || !gates.words[0].is_none_or(|wanted| boundary(begin) == wanted)
        {
            continue;
        }
        // Enumerate endpoints independently at each candidate, rather than
        // copying the production sliding-run search or its charging behavior.
        let run = input.points[begin..]
            .iter()
            .take_while(|&&point| set.contains(point))
            .count();
        let maximum = bounds.maximum.unwrap_or(run).min(run);
        if minimum > maximum {
            continue;
        }
        for choice in minimum..=maximum {
            let count = if bounds.greedy {
                maximum - (choice - minimum)
            } else {
                choice
            };
            let end = begin + count;
            if (!gates.anchors[1]
                || end == input.points.len()
                || (gates.multiline && line(input.points[end])))
                && gates.words[1].is_none_or(|wanted| boundary(end) == wanted)
            {
                return Some(input.boundaries[begin]..input.boundaries[end]);
            }
        }
    }
    None
}

fn recording(
    source: &JsString,
    ignore_case: bool,
    multiline: bool,
    work: &mut Vec<usize>,
    fail: Option<usize>,
) -> Result<Option<RegExpUnicodeRepeatedAtomMatcher>, RegExpUnicodeSetCompileError<usize>> {
    RegExpUnicodeRepeatedAtomMatcher::compile_v_set_with_single_character_strings_and_work(
        source,
        ignore_case,
        multiline,
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

fn compile(source: &str, ignore_case: bool, multiline: bool) -> RegExpUnicodeRepeatedAtomMatcher {
    recording(
        &JsString::from(source),
        ignore_case,
        multiline,
        &mut Vec::new(),
        None,
    )
    .unwrap()
    .unwrap()
}

#[test]
fn repeated_single_character_string_operands_match_pairwise_count_oracle() {
    let plain = Gates::default();
    let full = Gates {
        anchors: [true, true],
        ..plain
    };
    let words = Gates {
        words: [Some(true), Some(true)],
        ..plain
    };
    let folded_line = Gates {
        anchors: full.anchors,
        ignore_case: true,
        multiline: true,
        ..words
    };
    let greedy = |minimum, maximum| Bounds {
        minimum: Some(minimum),
        maximum,
        greedy: true,
    };
    let lazy = |minimum, maximum| Bounds {
        minimum: Some(minimum),
        maximum,
        greedy: false,
    };
    let ab = Set::Only(&[0x61, 0x62]);
    let a = Set::Only(&[0x61]);
    let empty = Set::Only(&[]);
    let folded_k = Set::Only(&[0x4b, 0x6b, 0x212a]);
    let huge = "9".repeat(100);
    let overflow_minimum = format!("{}{{{huge}}}", r"[\q{a}]");
    let overflow_maximum = format!("{}{{1,{huge}}}", r"[\q{a}]");
    let native_maximum = format!("{}{{{}}}", r"[\q{a}]", usize::MAX);
    // Pairwise rows cover every greedy/lazy quantifier spelling, not a product
    // of sets, quantifiers, flags and gates. Folded memberships are explicit.
    let cases: &[(&str, Set, Bounds, Gates)] = &[
        (r"[a\q{b}]*", ab, greedy(0, None), plain),
        (
            r"\b[a\q{b}]*?",
            ab,
            lazy(0, None),
            Gates {
                words: [Some(true), None],
                ..plain
            },
        ),
        (
            r"[\q{a|b}]+\b",
            ab,
            greedy(1, None),
            Gates {
                words: [None, Some(true)],
                ..plain
            },
        ),
        (
            r"[\q{a|b}]+?\B",
            ab,
            lazy(1, None),
            Gates {
                words: [None, Some(false)],
                ..plain
            },
        ),
        (
            r"^[\q{a}&&[a]]?",
            a,
            greedy(0, Some(1)),
            Gates {
                anchors: [true, false],
                ..plain
            },
        ),
        (
            r"[\q{a}&&[a]]??$",
            a,
            lazy(0, Some(1)),
            Gates {
                anchors: [false, true],
                ..plain
            },
        ),
        (r"\b[\q{a|b}]{2}\b", ab, greedy(2, Some(2)), words),
        (
            r"\B[\q{a|b}]{2}?",
            ab,
            lazy(2, Some(2)),
            Gates {
                words: [Some(false), None],
                ..plain
            },
        ),
        (r"^[\q{a|b}]{1,}$", ab, greedy(1, None), full),
        (
            r"^[\q{a|b}]{1,}?$",
            ab,
            lazy(1, None),
            Gates {
                multiline: true,
                ..full
            },
        ),
        (
            r"[\q{a|b}]{1,2}\b",
            ab,
            greedy(1, Some(2)),
            Gates {
                words: [None, Some(true)],
                ..plain
            },
        ),
        (
            r"\b[\q{a|b}]{1,2}?\b$",
            ab,
            lazy(1, Some(2)),
            Gates {
                anchors: [false, true],
                ..words
            },
        ),
        (r"[\q{a}&&[]]*", empty, greedy(0, None), plain),
        (r"[\q{a}--[a]]+", empty, greedy(1, None), plain),
        (
            r"\B[[]&&\q{a}]{0}\B",
            empty,
            greedy(0, Some(0)),
            Gates {
                words: [Some(false), Some(false)],
                ..plain
            },
        ),
        (
            r"[^\q{K|s}]+",
            Set::Except(&[0x4b, 0x6b, 0x212a, 0x53, 0x73, 0x17f]),
            greedy(1, None),
            Gates {
                ignore_case: true,
                ..plain
            },
        ),
        (r"^\b[\q{K}&&k]+\b$", folded_k, greedy(1, None), folded_line),
        (
            r"^\b[\q{K}&&k]+\b$",
            empty,
            greedy(1, None),
            Gates {
                ignore_case: false,
                ..folded_line
            },
        ),
        (
            r"^\b[\q{K}&&k]+\b$",
            folded_k,
            greedy(1, None),
            Gates {
                multiline: false,
                ..folded_line
            },
        ),
        (
            r"\B[\q{\uD83D\uDE00|\u{D800}}[\u{DC00}]]{1,2}?\B",
            Set::Only(&[0x1f600, 0xd800, 0xdc00]),
            lazy(1, Some(2)),
            Gates {
                words: [Some(false), Some(false)],
                ..plain
            },
        ),
        (
            r"[\q{😀|𐐀}--\q{𐐨}]+",
            Set::Only(&[0x1f600]),
            greedy(1, None),
            Gates {
                ignore_case: true,
                ..plain
            },
        ),
        (
            r"[\d--\q{1}]{0001,0002}",
            Set::Only(&[0x30, 0x32, 0x33, 0x34, 0x35, 0x36, 0x37, 0x38, 0x39]),
            greedy(1, Some(2)),
            plain,
        ),
        (
            r"[\q{\{|\}|\?}]{1,2}?",
            Set::Only(&[0x7b, 0x7d, 0x3f]),
            lazy(1, Some(2)),
            plain,
        ),
        (
            &overflow_minimum,
            a,
            Bounds {
                minimum: None,
                maximum: None,
                greedy: true,
            },
            plain,
        ),
        (&overflow_maximum, a, greedy(1, None), plain),
        (
            &native_maximum,
            a,
            greedy(usize::MAX, Some(usize::MAX)),
            plain,
        ),
    ];
    let inputs = [
        JsString::from(""),
        JsString::from("aaabb!ab"),
        JsString::from("KKkſs!"),
        JsString::from("\r\nab\nKK\u{2028}!"),
        JsString::from("😀𐐀𐐨a"),
        JsString::from_code_units(vec![0xd800, 0xd800, 0x61, 0xdc00, 0xd83d, 0xde00]),
        JsString::from("{?}!012"),
    ]
    .map(Input::new);
    for &(source, set, bounds, gates) in cases {
        let original = compile(source, gates.ignore_case, gates.multiline);
        let matcher = original.clone();
        drop(original);
        for input in &inputs {
            for start in 0..=input.text.len() + 1 {
                for sticky in [false, true] {
                    let expected =
                        expected(set, bounds, gates, input, start, sticky).map(|range| {
                            RegExpUnicodeRepeatedAtomMatch {
                                range,
                                atom_capture: None,
                            }
                        });
                    assert_eq!(
                        matcher
                            .find_with_captures_and_work(
                                &input.text,
                                start,
                                sticky,
                                |_| Ok::<(), ()>(())
                            )
                            .unwrap(),
                        expected,
                        "{source} i={} m={} input={:?} start={start} sticky={sticky}",
                        gates.ignore_case,
                        gates.multiline,
                        input.text
                    );
                }
            }
        }
    }
    // Concrete endpoint/earliest-start witnesses keep those choices visible.
    for (source, input, sticky, wanted) in [
        (r"[\q{a|b}]+", "aaabb!ab", false, Some(0..5)),
        (r"[\q{a|b}]+?", "aaabb!ab", false, Some(0..1)),
        (r"[\q{a}]{1,2}\b", "aaa", false, Some(1..3)),
        (r"[\q{a}]{1,2}\b", "aaa", true, None),
        (r"[\q{a}&&[]]*", "a", true, Some(0..0)),
    ] {
        assert_eq!(
            compile(source, false, false)
                .find_with_work(&JsString::from(input), 0, sticky, |_| Ok::<(), ()>(()))
                .unwrap(),
            wanted
        );
    }
}

#[test]
fn repeated_single_character_string_operands_domains_and_cancellation() {
    let huge = "9".repeat(100);
    let bad_impossible = format!("{}{{{huge}}}", r"[\q{a}&&[]&&\q{b|cd}]");
    for (text, supported) in [
        (r"[a\q{b}]*?", true),
        (r"[[]&&\q{a}]{0}", true),
        (r"\B[\q{😀|\u{d800}}]{1,2}?\B", true),
        (r"[\q{a}&&[]&&\q{b|cd}]{0}", false),
        (bad_impossible.as_str(), false),
        (r"[\q{a}--a--\p{RGI_Emoji}]{0}", false),
    ] {
        let source = JsString::from(text);
        let mut complete = Vec::new();
        assert_eq!(
            recording(&source, false, false, &mut complete, None)
                .unwrap()
                .is_some(),
            supported,
            "{text}"
        );
        assert_eq!(complete[0], source.len());
        for fail in 1..=complete.len() {
            let mut seen = Vec::new();
            assert!(
                matches!(recording(&source, false, false, &mut seen, Some(fail)), Err(RegExpUnicodeSetCompileError::Work(at)) if at == fail)
            );
            assert_eq!(seen, complete[..fail], "{text} abort={fail}");
        }
        let mut fresh = Vec::new();
        assert_eq!(
            recording(&source, false, false, &mut fresh, None)
                .unwrap()
                .is_some(),
            supported
        );
        assert_eq!(fresh, complete);
    }
    // Search charges the suffix incrementally; aborting must not mutate the plan.
    for (source, input, sticky, wanted) in [
        (r"[\q{a}]+?", "baaa!", false, Some(1..2)),
        (r"[\q{a}]{1,2}\b", "aaa!", false, Some(1..3)),
        (r"[\q{a}&&[]]*", "a", true, Some(0..0)),
    ] {
        let matcher = compile(source, false, false);
        let input = JsString::from(input);
        let mut complete = Vec::new();
        assert_eq!(
            matcher
                .find_with_work(&input, 0, sticky, |amount| {
                    complete.push(amount);
                    Ok::<(), usize>(())
                })
                .unwrap(),
            wanted
        );
        for fail in 1..=complete.len() {
            let mut seen = Vec::new();
            let result = matcher.find_with_work(&input, 0, sticky, |amount| {
                seen.push(amount);
                if fail == seen.len() {
                    Err(fail)
                } else {
                    Ok(())
                }
            });
            assert_eq!(result, Err(fail));
            assert_eq!(seen, complete[..fail]);
        }
        let mut fresh = Vec::new();
        assert_eq!(
            matcher
                .find_with_work(&input, 0, sticky, |amount| {
                    fresh.push(amount);
                    Ok::<(), usize>(())
                })
                .unwrap(),
            wanted
        );
        assert_eq!(fresh, complete);
    }
    for text in [
        "",
        "^",
        "$",
        "^$",
        "a+",
        "[a]+",
        "[[a]]+",
        "[a&&b]+",
        r"[\q{a}]",
        r"[\q{}]{0}",
        r"[\q{ab}]*",
        r"[\q{a|}]{0}",
        r"[\q{a}\p{Basic_Emoji}]*",
        r"[\q{\d}]+",
        r"[\q{a|b]+",
        r"([\q{a}])+",
        r"(?:[\q{a}])+",
        r"(?<inner>[\q{a}])+",
        r"a[\q{b}]+",
        r"[\q{a}]+b",
        r"[\q{a}]+[b]+",
        r"[\q{a}]+|b",
        r"\b^[\q{a}]+",
        r"[\q{a}]+$\b",
        r"\b\b[\q{a}]+",
        r"[\q{a}]+\b\B",
        r"[\q{a}]{2,1}",
        r"[\q{a}]{,2}",
        r"[\q{a}]{1,2,3}",
    ] {
        let source = JsString::from(text);
        let mut seen = Vec::new();
        assert!(
            recording(&source, false, false, &mut seen, None)
                .unwrap()
                .is_none(),
            "{text}"
        );
        assert_eq!(seen[0], source.len());
    }
    for text in [
        r"[\q{a}]+",
        r"[a\q{b}]*?",
        r"[\q{a}&&[]]{0}",
        r"\b[\q{a}]+\b",
    ] {
        let source = JsString::from(text);
        let accept: fn(usize) -> Result<(), usize> = |_| Ok(());
        assert!(
            RegExpUnicodeRepeatedAtomMatcher::compile_with_flags_and_work(
                &source, true, false, false, accept
            )
            .unwrap()
            .is_none()
        );
        assert!(
            RegExpUnicodeRepeatedAtomMatcher::compile_with_line_anchors_and_work(
                &source, true, false, false, false, accept
            )
            .unwrap()
            .is_none()
        );
        assert!(
            RegExpUnicodeRepeatedAtomMatcher::compile_with_word_boundaries_and_work(
                &source, true, false, false, false, accept
            )
            .unwrap()
            .is_none()
        );
        assert!(
            RegExpUnicodeRepeatedAtomMatcher::compile_v_set_with_line_anchors_and_work(
                &source, false, false, accept
            )
            .unwrap()
            .is_none()
        );
        assert!(
            RegExpUnicodeRepeatedAtomMatcher::compile_v_set_with_word_boundaries_and_work(
                &source, false, false, accept
            )
            .unwrap()
            .is_none()
        );
    }
}
