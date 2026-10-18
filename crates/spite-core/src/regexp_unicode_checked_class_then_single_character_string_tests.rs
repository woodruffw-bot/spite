use super::{
    JsString, RegExpUnicodeClassMatcher, RegExpUnicodeClassPairMatcher,
    RegExpUnicodeSetCompileError,
};
use std::ops::Range;

#[derive(Clone, Copy)]
enum Set {
    Only(&'static [u32]),
    Except(&'static [u32]),
    Range(u32, u32),
    FoldedAscii,
}

impl Set {
    fn contains(self, point: u32) -> bool {
        match self {
            Self::Only(points) => points.contains(&point),
            Self::Except(points) => !points.contains(&point),
            Self::Range(first, last) => (first..=last).contains(&point),
            Self::FoldedAscii => matches!(point, 0..=0x7f | 0x17f | 0x212a),
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
        // Standard decoding, not the matcher decoder or a Unicode data table.
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
    // The approved Unicode start rule selects the containing pair boundary.
    let begin = input.boundaries.partition_point(|&at| at <= start) - 1;
    let word = |point| {
        matches!(point, 0x30..=0x39 | 0x41..=0x5a | 0x5f | 0x61..=0x7a)
            || (gates.ignore_case && matches!(point, 0x17f | 0x212a))
    };
    let boundary = |at: usize| {
        input.points.get(at).is_some_and(|&point| word(point))
            != at
                .checked_sub(1)
                .and_then(|before| input.points.get(before))
                .is_some_and(|&point| word(point))
    };
    let line = |point| matches!(point, 0xa | 0xd | 0x2028 | 0x2029);
    // Enumerating adjacent original characters independently preserves overlap.
    for at in begin..input.points.len().saturating_sub(1) {
        if sticky && at != begin {
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
    RegExpUnicodeClassPairMatcher::compile_checked_then_v_set_with_single_character_strings_and_work(
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
fn checked_class_then_single_character_string_matches_independent_utf16_pairs() {
    use Set::{Except, FoldedAscii, Only, Range};
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
    let anchored_lines = Gates {
        multiline: true,
        anchors: [true, true],
        ..plain
    };
    // These are full preimage memberships, handwritten after each set operation.
    // In vi, K/k/Kelvin and S/s/long-s form the only nontrivial finite groups here.
    let cases = [
        (r"[[a]][\q{b}]", Only(&[0x61]), Only(&[0x62]), plain),
        (r"[a&&a][\q{b|c}--c]", Only(&[0x61]), Only(&[0x62]), plain),
        (
            r"[[a-c]--b][\q{a|b}]",
            Only(&[0x61, 0x63]),
            Only(&[0x61, 0x62]),
            plain,
        ),
        (
            r"[^a&&a][^\q{a|b}]",
            Except(&[0x61]),
            Except(&[0x61, 0x62]),
            plain,
        ),
        (r"[a--a][\q{b}]", Only(&[]), Only(&[0x62]), plain),
        (r"[[a]][\q{b}--b]", Only(&[0x61]), Only(&[]), plain),
        (
            r"[[K]][\q{s}]",
            Only(&[0x4b, 0x6b, 0x212a]),
            Only(&[0x53, 0x73, 0x17f]),
            folded,
        ),
        (r"[[K]][\q{s}]", Only(&[0x4b]), Only(&[0x73]), plain),
        (
            r"[[\p{ASCII}]][^\q{K|s}]",
            FoldedAscii,
            Except(&[0x4b, 0x6b, 0x212a, 0x53, 0x73, 0x17f]),
            folded,
        ),
        (
            r"[[\p{ASCII}]][^\q{K|s}]",
            Range(0, 0x7f),
            Except(&[0x4b, 0x73]),
            plain,
        ),
        (
            r"[[\u{1f600}]][\q{\uD83D\uDE80}]",
            Only(&[0x1f600]),
            Only(&[0x1f680]),
            plain,
        ),
        (
            r"[[\uD800-\uDFFF]][\q{\\}]",
            Range(0xd800, 0xdfff),
            Only(&[92]),
            plain,
        ),
        (
            r"[[\$]][\q{\]|\}|\\|\[}]",
            Only(&[0x24]),
            Only(&[0x5d, 0x7d, 92, 0x5b]),
            plain,
        ),
        (r"[[\[]][\q{a}]", Only(&[0x5b]), Only(&[0x61]), plain),
        (
            r"^[[a]][\q{b}]$",
            Only(&[0x61]),
            Only(&[0x62]),
            anchored_lines,
        ),
        (
            r"^\b[[a]][\q{b}]\b$",
            Only(&[0x61]),
            Only(&[0x62]),
            Gates {
                words: [Some(true), Some(true)],
                ..anchored_lines
            },
        ),
        (
            r"[[a]][\q{a}]\b",
            Only(&[0x61]),
            Only(&[0x61]),
            Gates {
                words: [None, Some(true)],
                ..plain
            },
        ),
        (
            r"\B[[a]][\q{a}]\b",
            Only(&[0x61]),
            Only(&[0x61]),
            Gates {
                words: [Some(false), Some(true)],
                ..plain
            },
        ),
        (
            r"^\b[[\p{ASCII}]][\q{\!}\p{Any}]\B$",
            FoldedAscii,
            Except(&[]),
            Gates {
                anchors: [true, true],
                words: [Some(true), Some(false)],
                ..folded
            },
        ),
        (
            r"^\b[[\p{ASCII}]][\q{\!}\p{Any}]\B$",
            Range(0, 0x7f),
            Except(&[]),
            Gates {
                anchors: [true, true],
                words: [Some(true), Some(false)],
                ..plain
            },
        ),
    ];
    assert_eq!(cases.len(), 20);
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
    // Eight inputs contain49 UTF-16 units:20*2*(49+2*8)=2600 queries.
    assert_eq!(queries, 2600);

    // Direct witnesses state overlap, original line/word context and widths.
    let direct = [
        (
            r"[[a]][\q{b}]",
            JsString::from("aab!"),
            0,
            false,
            false,
            false,
            Some(1..3),
        ),
        (
            r"[[a]][\q{b}]",
            JsString::from("aab!"),
            0,
            true,
            false,
            false,
            None,
        ),
        (
            r"[[a]][\q{a}]\b",
            JsString::from("aaa!"),
            0,
            false,
            false,
            false,
            Some(1..3),
        ),
        (
            r"[[a]][\q{a}]\b",
            JsString::from("aaa!"),
            0,
            true,
            false,
            false,
            None,
        ),
        (
            r"[[a]][\q{a}]\b",
            JsString::from("aaa!"),
            1,
            true,
            false,
            false,
            Some(1..3),
        ),
        (
            r"[[\u{1f600}]][\q{\uD83D\uDE80}]",
            JsString::from("A😀🚀!"),
            2,
            true,
            false,
            false,
            Some(1..5),
        ),
        (
            r"^[[a]][\q{b}]$",
            JsString::from("x\r\nab\u{2028}"),
            0,
            false,
            false,
            true,
            Some(3..5),
        ),
        (
            r"^[[a]][\q{b}]$",
            JsString::from("ab"),
            0,
            true,
            false,
            false,
            Some(0..2),
        ),
        (
            r"^[[a]][\q{b}]$",
            JsString::from("ab\n"),
            0,
            false,
            false,
            false,
            None,
        ),
        (
            r"[[\[]][\q{a}]",
            JsString::from("[a"),
            0,
            true,
            false,
            false,
            Some(0..2),
        ),
        (
            r"[[\uD800-\uDFFF]][\q{\uDC00}]",
            JsString::from_code_units(vec![0xd800, 0xdc00]),
            0,
            false,
            false,
            false,
            None,
        ),
        (
            r"[[\uD800-\uDFFF]][\q{\\}]",
            JsString::from_code_units(vec![0xd800, 92]),
            0,
            true,
            false,
            false,
            Some(0..2),
        ),
        (
            r"^\b[[\p{ASCII}]][\q{\!}\p{Any}]\B$",
            JsString::from("K$"),
            0,
            true,
            true,
            false,
            Some(0..2),
        ),
        (
            r"^\b[[\p{ASCII}]][\q{\!}\p{Any}]\B$",
            JsString::from("K$"),
            0,
            true,
            false,
            false,
            None,
        ),
        (
            r"^\B[[\p{Any}]][\q{\!}\p{Any}]\b$",
            JsString::from("!K"),
            0,
            true,
            true,
            false,
            Some(0..2),
        ),
        (
            r"^\B[[\p{Any}]][\q{\!}\p{Any}]\b$",
            JsString::from("!K"),
            0,
            true,
            false,
            false,
            None,
        ),
        (
            r"\b[[a]][\q{b}]",
            JsString::from("abx"),
            0,
            true,
            false,
            false,
            Some(0..2),
        ),
    ];
    assert_eq!(direct.len(), 17);
    for (source, input, start, sticky, i, m, wanted) in direct {
        assert_eq!(
            compile(source, i, m).unwrap().find(&input, start, sticky),
            wanted,
            "{source}"
        );
    }
}

#[test]
fn checked_class_then_single_character_string_domains_and_work_prefixes() {
    // Framing or the legacy no-q boundary declines before either child proof.
    let framed_out = [
        "",
        "ab",
        r"[\q{a}][\q{b}]",
        r"[\q{a}][[b]]",
        r"([[a]])[\q{b}]",
        r"[[a]]([\q{b}])",
        r"([[a]][\q{b}])",
        r"(?:[[a]][\q{b}])",
        r"[[a]]+[\q{b}]",
        r"[[a]][\q{b}]+",
        r"[[a]][\q{b}][c]",
        r"a[[a]][\q{b}]",
        r"[[a]][\q{b}]|c",
        r"[[a]]\b[\q{b}]",
        r"[[a]]^[\q{b}]",
        r"\b^[[a]][\q{b}]",
        r"[[a]][\q{b}]$\b",
        r"\b\b[[a]][\q{b}]",
        r"[[a]][\q{b}]\b\b",
        r"^^[[a]][\q{b}]",
        r"[[a]][\q{b}]$$",
        r"[[a]][\q{b}",
        r"[[a]][\q{b}]\",
        r"[[a]][\q{b}]?",
    ];
    assert_eq!(framed_out.len(), 24);
    for source in framed_out {
        let source = JsString::from(source);
        let wanted = [source.len().saturating_mul(4)];
        let mut work = Vec::new();
        assert!(
            recording(&source, false, false, &mut work, None)
                .unwrap()
                .is_none()
        );
        assert_eq!(work, wanted);
        let mut aborted = Vec::new();
        assert!(matches!(
            recording(&source, false, false, &mut aborted, Some(1)),
            Err(RegExpUnicodeSetCompileError::Work(1))
        ));
        assert_eq!(aborted, wanted);
        let mut rebuilt = Vec::new();
        assert!(
            recording(&source, false, false, &mut rebuilt, None)
                .unwrap()
                .is_none()
        );
        assert_eq!(rebuilt, wanted);
    }
    for source in [
        r"[a][\q{b}]",
        r"[][\q{b}]",
        r"[\p{ASCII}][\q{b}]",
        r"[[a]][[b]]",
        r"[a&&a][b&&b]",
        r"[[a]][b]",
        r"[[a]][\q{}]",
        r"[[a]][\q{b|cd}]",
        r"[[a]][\q{\d}]",
        r"[[a]][\q{\u{D800}\u{DC00}}]",
        r"[a--a][\q{b}&&[]&&\p{RGI_Emoji}]",
        r"[[\p{RGI_Emoji}]][\q{b}]",
    ] {
        assert!(compile(source, true, true).is_none(), "{source}");
    }

    struct WorkCase {
        source: &'static str,
        first: &'static str,
        first_charges: &'static [usize],
        first_proved: bool,
        second: &'static str,
        second_charges: &'static [usize],
        accepted: bool,
        probe: Option<Range<usize>>,
    }
    // Handwritten full child sources and numeric transcripts are independent of
    // adapter framing. These tiny no-i proofs use at most one live interval per
    // operand, so their charges do not depend on allocator growth policy.
    let work_cases = [
        WorkCase {
            source: r"[[a]][\q{b}]",
            first: "[[a]]",
            first_charges: &[20, 1, 1, 1, 1, 1],
            first_proved: true,
            second: r"[\q{b}]",
            second_charges: &[28, 1, 1, 1, 1],
            accepted: true,
            probe: Some(1..3),
        },
        WorkCase {
            source: r"^\b[[a]][\q{b}]\b$",
            first: "[[a]]",
            first_charges: &[20, 1, 1, 1, 1, 1],
            first_proved: true,
            second: r"[\q{b}]",
            second_charges: &[28, 1, 1, 1, 1],
            accepted: true,
            probe: None,
        },
        WorkCase {
            source: r"[a--a][\q{b}]",
            first: "[a--a]",
            first_charges: &[24, 1, 1, 1, 1, 1, 6, 2, 0],
            first_proved: true,
            second: r"[\q{b}]",
            second_charges: &[28, 1, 1, 1, 1],
            accepted: true,
            probe: None,
        },
        WorkCase {
            source: r"\b[a--a][\q{b|cd}]\B",
            first: "[a--a]",
            first_charges: &[24, 1, 1, 1, 1, 1, 6, 2, 0],
            first_proved: true,
            second: r"[\q{b|cd}]",
            second_charges: &[40, 1, 1],
            accepted: false,
            probe: None,
        },
        WorkCase {
            source: r"^\b[a--a][\q{b}&&[]&&\p{RGI_Emoji}]\B$",
            first: "[a--a]",
            first_charges: &[24, 1, 1, 1, 1, 1, 6, 2, 0],
            first_proved: true,
            second: r"[\q{b}&&[]&&\p{RGI_Emoji}]",
            second_charges: &[104, 1, 1, 1, 1, 3, 1],
            accepted: false,
            probe: None,
        },
        WorkCase {
            source: r"[a][\q{b}]",
            first: "[a]",
            first_charges: &[12, 1, 1, 1],
            first_proved: false,
            second: r"[\q{b}]",
            second_charges: &[],
            accepted: false,
            probe: None,
        },
        WorkCase {
            source: "[[a]][b]",
            first: "[[a]]",
            first_charges: &[20, 1, 1, 1, 1, 1],
            first_proved: true,
            second: "[b]",
            second_charges: &[12, 1, 1, 1],
            accepted: false,
            probe: None,
        },
        WorkCase {
            source: r"^\b[[!]][\q{b|cd}]\B$",
            first: "[[!]]",
            first_charges: &[20, 1, 1, 1, 1, 1],
            first_proved: true,
            second: r"[\q{b|cd}]",
            second_charges: &[40, 1, 1],
            accepted: false,
            probe: None,
        },
        WorkCase {
            source: r"[a--a][\q{}]",
            first: "[a--a]",
            first_charges: &[24, 1, 1, 1, 1, 1, 6, 2, 0],
            first_proved: true,
            second: r"[\q{}]",
            second_charges: &[24, 1],
            accepted: false,
            probe: None,
        },
    ];
    assert_eq!(work_cases.len(), 9);
    let mut baseline_charges = 0;
    let mut abort_points = 0;
    let mut abort_charges = 0;
    let mut rebuild_charges = 0;
    let mut child_proofs = 0;
    let mut child_charges = 0;
    let probe = JsString::from("aab!");
    for case in work_cases {
        let first = JsString::from(case.first);
        let mut first_work = Vec::new();
        let first_plan = RegExpUnicodeClassMatcher::compile_v_set_units_with_work(
            first.code_units(),
            false,
            false,
            |amount| {
                first_work.push(amount);
                Ok::<(), usize>(())
            },
        )
        .unwrap();
        assert_eq!(first_plan.is_some(), case.first_proved, "{}", case.first);
        assert_eq!(first_work, case.first_charges, "{}", case.first);
        child_proofs += 1;
        child_charges += first_work.len();
        if case.first_proved {
            let second = JsString::from(case.second);
            let mut second_work = Vec::new();
            let second_plan = RegExpUnicodeClassMatcher::compile_v_set_units_with_single_character_strings_and_work(
                second.code_units(), false, false,
                |amount| { second_work.push(amount); Ok::<(), usize>(()) },
            ).unwrap();
            assert_eq!(second_plan.is_some(), case.accepted, "{}", case.second);
            assert_eq!(second_work, case.second_charges, "{}", case.second);
            child_proofs += 1;
            child_charges += second_work.len();
        }
        let source = JsString::from(case.source);
        let mut wanted = vec![source.len().saturating_mul(4)];
        wanted.extend_from_slice(case.first_charges);
        wanted.extend_from_slice(case.second_charges);
        let mut complete = Vec::new();
        let plan = recording(&source, false, false, &mut complete, None).unwrap();
        assert_eq!(plan.is_some(), case.accepted, "{}", case.source);
        assert_eq!(complete, wanted, "{}", case.source);
        if let Some(plan) = plan {
            assert_eq!(plan.find(&probe, 0, false), case.probe, "{}", case.source);
        }
        baseline_charges += wanted.len();
        for fail in 1..=wanted.len() {
            let mut aborted = Vec::new();
            assert!(matches!(
                recording(&source, false, false, &mut aborted, Some(fail)),
                Err(RegExpUnicodeSetCompileError::Work(at)) if at == fail
            ));
            assert_eq!(aborted, wanted[..fail]);
            abort_points += 1;
            abort_charges += aborted.len();
            // Every abort is followed by a separate fresh build of the same body.
            let mut rebuilt = Vec::new();
            let plan = recording(&source, false, false, &mut rebuilt, None).unwrap();
            assert_eq!(plan.is_some(), case.accepted);
            assert_eq!(rebuilt, wanted);
            if let Some(plan) = plan {
                assert_eq!(plan.find(&probe, 0, false), case.probe);
            }
            rebuild_charges += rebuilt.len();
        }
    }
    assert_eq!((child_proofs, child_charges), (17, 98));
    assert_eq!(
        (
            baseline_charges,
            abort_points,
            abort_charges,
            rebuild_charges
        ),
        (107, 107, 734, 1361)
    );

    // All eleven earlier entries retain None on canonical examples of this domain.
    let mut old_declines = 0;
    for (source, i, m) in [
        (r"[[a]][\q{b}]", false, false),
        (r"[a&&a][\q{b|c}]", false, false),
        (r"^\b[[a]][\q{b}]\b$", false, true),
        (r"[[K]][^\q{K|s}]", true, false),
    ] {
        let source = JsString::from(source);
        macro_rules! old_none {
            ($method:ident) => {
                assert!(
                    RegExpUnicodeClassPairMatcher::$method(&source, i, m, |_| Ok::<(), ()>(()),)
                        .unwrap()
                        .is_none()
                );
                old_declines += 1;
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
        old_none!(compile_flat_then_v_set_with_single_character_strings_and_work);
        old_none!(compile_v_set_pair_with_single_character_strings_and_work);
    }
    assert_eq!(old_declines, 44);
    for (source, input, start, sticky, wanted) in [
        (r"[[a]][\q{b}]", "aab!", 0, false, Some(1..3)),
        (r"[[a]][\q{a}]\b", "aaa!", 0, true, None),
        (
            r"[[\u{1f600}]][\q{\uD83D\uDE80}]",
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
        assert_eq!(matcher.find(&input, start, sticky), wanted);
    }
}
