use super::{JsString, RegExpUnicodeClassPairMatcher, RegExpUnicodeSetCompileError};
use std::ops::Range;

fn compile(source: &str, i: bool, m: bool) -> Option<RegExpUnicodeClassPairMatcher> {
    RegExpUnicodeClassPairMatcher::compile_v_set_then_checked_with_single_character_strings_and_work(
        &JsString::from(source), i, m, |_| Ok::<(), ()>(()),
    ).unwrap()
}

#[derive(Clone, Copy)]
enum Set {
    Only(&'static [u32]),
    Except(&'static [u32]),
    Ascii,
}

impl Set {
    fn contains(self, point: u32) -> bool {
        match self {
            Self::Only(points) => points.contains(&point),
            Self::Except(points) => !points.contains(&point),
            Self::Ascii => point <= 0x7f,
        }
    }
}

#[derive(Clone, Copy, Default)]
struct Gates {
    i: bool,
    m: bool,
    anchors: [bool; 2],
    words: [Option<bool>; 2],
}

// Decode original characters independently of the matcher and its Unicode tables.
fn oracle(
    input: &JsString,
    start: usize,
    sticky: bool,
    sets: [Set; 2],
    gates: Gates,
) -> Option<Range<usize>> {
    if start > input.len() {
        return None;
    }
    let mut points = Vec::new();
    let mut boundaries = vec![0];
    for decoded in char::decode_utf16(input.code_units().iter().copied()) {
        let (point, width) = match decoded {
            Ok(c) => (u32::from(c), c.len_utf16()),
            Err(e) => (u32::from(e.unpaired_surrogate()), 1),
        };
        points.push(point);
        boundaries.push(boundaries.last().unwrap() + width);
    }
    let first = boundaries.partition_point(|&at| at <= start) - 1;
    let word = |point| {
        matches!(point, 0x30..=0x39 | 0x41..=0x5a | 0x5f | 0x61..=0x7a)
            || (gates.i && matches!(point, 0x17f | 0x212a))
    };
    let boundary = |at: usize| {
        points.get(at).is_some_and(|&p| word(p))
            != at
                .checked_sub(1)
                .and_then(|at| points.get(at))
                .is_some_and(|&p| word(p))
    };
    let line = |point| matches!(point, 10 | 13 | 0x2028 | 0x2029);
    for at in first..points.len().saturating_sub(1) {
        if sticky && at != first {
            break;
        }
        if sets[0].contains(points[at])
            && sets[1].contains(points[at + 1])
            && (!gates.anchors[0] || at == 0 || (gates.m && line(points[at - 1])))
            && (!gates.anchors[1] || at + 2 == points.len() || (gates.m && line(points[at + 2])))
            && gates.words[0].is_none_or(|wanted| boundary(at) == wanted)
            && gates.words[1].is_none_or(|wanted| boundary(at + 2) == wanted)
        {
            return Some(boundaries[at]..boundaries[at + 2]);
        }
    }
    None
}

#[test]
fn string_then_checked_pairs_match_independent_memberships_and_utf16_boundaries() {
    use Set::{Ascii, Except, Only};
    let plain = Gates::default();
    let folded = Gates { i: true, ..plain };
    let inputs = [
        JsString::from(""),
        JsString::from("aabaaa!abx"),
        JsString::from("ſKsK!ks"),
        JsString::from("A😀🚀!"),
        JsString::from("x\r\nab\u{2028}]$tail"),
        JsString::from("]$[\\a!"),
        JsString::from_code_units(vec![0xd800, 92, 0xdc00, 0xdc00, 33]),
    ];
    for (source, sets, gates) in [
        (r"[\q{a}][[b]]", [Only(&[97]), Only(&[98])], plain),
        (r"[\q{a|c}--c][b&&b]", [Only(&[97]), Only(&[98])], plain),
        (r"[^\q{a}][[b]]", [Except(&[97]), Only(&[98])], plain),
        (r"[\q{a}][^b&&b]", [Only(&[97]), Except(&[98])], plain),
        (r"[\q{a}--a][[b]]", [Only(&[]), Only(&[98])], plain),
        (r"[\q{a}][b--b]", [Only(&[97]), Only(&[])], plain),
        (
            r"[\q{K}][[s]]",
            [Only(&[75, 107, 0x212a]), Only(&[83, 115, 0x17f])],
            folded,
        ),
        (r"[\q{K}][[s]]", [Only(&[75]), Only(&[115])], plain),
        (r"[\q{a}\p{ASCII}][[b]]", [Ascii, Only(&[98])], plain),
        (
            r"[\q{\uD83D\uDE00}][[\u{1f680}]]",
            [Only(&[0x1f600]), Only(&[0x1f680])],
            plain,
        ),
        (r"[\q{\uD800}][[\\]]", [Only(&[0xd800]), Only(&[92])], plain),
        (
            r"[\q{\]|\\|\[}][[\$]]",
            [Only(&[93, 92, 91]), Only(&[36])],
            plain,
        ),
    ] {
        let original = compile(source, gates.i, gates.m).unwrap();
        let matcher = original.clone();
        drop(original);
        for input in &inputs {
            for start in 0..=input.len() + 1 {
                for sticky in [false, true] {
                    assert_eq!(
                        matcher.find(input, start, sticky),
                        oracle(input, start, sticky, sets, gates),
                        "{source} input={input:?} start={start} sticky={sticky}"
                    );
                }
            }
        }
    }
}

#[test]
fn outside_word_and_line_gates_use_original_input_and_keep_overlapping_candidates() {
    let inputs = [
        "aaa!",
        "aa",
        "xaa!",
        "!aax",
        "x\raa\ry",
        "x\naa\ny",
        "x\r\naa\r\ny",
        "x\u{2028}aa\u{2029}y",
    ];
    for m in [false, true] {
        for anchors in [[false, false], [true, false], [false, true], [true, true]] {
            for left in [None, Some(false), Some(true)] {
                for right in [None, Some(false), Some(true)] {
                    let word = |gate| match gate {
                        Some(true) => r"\b",
                        Some(false) => r"\B",
                        None => "",
                    };
                    let source = format!(
                        "{}{}[\\q{{a}}][[a]]{}{}",
                        if anchors[0] { "^" } else { "" },
                        word(left),
                        word(right),
                        if anchors[1] { "$" } else { "" }
                    );
                    let matcher = compile(&source, false, m).unwrap();
                    let gates = Gates {
                        m,
                        anchors,
                        words: [left, right],
                        ..Gates::default()
                    };
                    for text in inputs {
                        let input = JsString::from(text);
                        for start in 0..=input.len() + 1 {
                            for sticky in [false, true] {
                                assert_eq!(
                                    matcher.find(&input, start, sticky),
                                    oracle(
                                        &input,
                                        start,
                                        sticky,
                                        [Set::Only(&[97]), Set::Only(&[97])],
                                        gates
                                    ),
                                    "{source} m={m} {text:?} start={start} sticky={sticky}"
                                );
                            }
                        }
                    }
                }
            }
        }
    }
    let astral = compile(r"[\q{😀}][[🚀]]", false, false).unwrap();
    assert_eq!(astral.find(&JsString::from("A😀🚀!"), 2, true), Some(1..5));
    assert_eq!(
        compile(r"[\q{a}][[a]]\b", false, false)
            .unwrap()
            .find(&JsString::from("aaa!"), 0, false),
        Some(1..3)
    );
}

#[test]
fn proof_requires_q_prefix_and_checked_non_q_suffix_without_inner_syntax() {
    for source in [
        "",
        "ab",
        r"[\q{a}][b]",
        r"[\q{a}][\q{b}]",
        r"[[a]][[b]]",
        r"[[a]][\q{b}]",
        r"[\q{ab}][[b]]",
        r"[\q{}][[b]]",
        r"[\q{a|}][[b]]",
        r"[\q{\u{D800}\u{DC00}}][[b]]",
        r"[\q{a}\p{RGI_Emoji}][[b]]",
        r"[\q{a}--a][[\p{RGI_Emoji}]]",
        r"[\q{a}][[b]]+",
        r"[\q{a}]+[[b]]",
        r"([\q{a}])[[b]]",
        r"[\q{a}]([[b]])",
        r"([\q{a}][[b]])",
        r"[\q{a}][[b]][c]",
        r"[\q{a}]\b[[b]]",
        r"\b^[\q{a}][[b]]",
        r"[\q{a}][[b]]$\b",
        r"[\q{a}][[b]]$$",
        r"[\q{a}][[b]]\b\b",
        r"[\q{a}][[b]]|c",
    ] {
        assert!(compile(source, true, true).is_none(), "{source}");
    }
    let source = JsString::from(r"[\q{a}][[b]]");
    // No earlier public proof is widened to admit the new operand order.
    assert!(RegExpUnicodeClassPairMatcher::compile_checked_then_v_set_with_single_character_strings_and_work(&source,false,false, |_| Ok::<(),()>(())).unwrap().is_none());
    assert!(
        RegExpUnicodeClassPairMatcher::compile_v_set_pair_with_single_character_strings_and_work(
            &source,
            false,
            false,
            |_| Ok::<(), ()>(())
        )
        .unwrap()
        .is_none()
    );
    assert!(RegExpUnicodeClassPairMatcher::compile_v_set_then_flat_with_single_character_strings_and_work(&source,false,false, |_| Ok::<(),()>(())).unwrap().is_none());
}

#[test]
fn checked_suffix_compiles_after_an_empty_prefix_and_work_errors_stop_at_exact_prefixes() {
    for (source, accepted, suffix_work) in [
        (r"[\q{a}][[b]]", true, 20),
        (r"[\q{a}--a][[b]]", true, 20),
        (r"^\b[\q{a}--a][[\p{RGI_Emoji}]]\B$", false, 68),
    ] {
        let source = JsString::from(source);
        let mut complete = Vec::new();
        let plan = RegExpUnicodeClassPairMatcher::compile_v_set_then_checked_with_single_character_strings_and_work(
            &source,false,false,|work| {complete.push(work);Ok::<(),usize>(())}).unwrap();
        assert_eq!(plan.is_some(), accepted);
        assert_eq!(complete[0], source.len() * 4);
        // The second component's framing charge is distinguishable even after
        // an empty prefix: [[b]] => 20, [[\p{RGI_Emoji}]] => 68.
        assert!(complete[1..].contains(&suffix_work), "{complete:?}");
        for fail in 1..=complete.len() {
            let mut seen = Vec::new();
            let result = RegExpUnicodeClassPairMatcher::compile_v_set_then_checked_with_single_character_strings_and_work(
                &source,false,false,|work| {seen.push(work);if seen.len()==fail {Err(fail)} else {Ok(())}});
            assert!(matches!(result,Err(RegExpUnicodeSetCompileError::Work(at)) if at==fail));
            assert_eq!(seen, complete[..fail]);
        }
        let rebuilt = compile(&source.to_utf8().unwrap(), false, false);
        assert_eq!(rebuilt.is_some(), plan.is_some());
    }
}
