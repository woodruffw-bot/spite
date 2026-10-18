use super::{JsString, RegExpUnicodeClassMatcher, RegExpUnicodeSetCompileError};
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

// Finite common/simple-fold facts for the witnesses below, not a production
// Unicode table, decoder, membership predicate or assertion implementation.
fn fold(point: u32) -> u32 {
    match point {
        0x41..=0x5a => point + 0x20,
        0x17f => 0x73,
        0x212a => 0x6b,
        0x345 | 0x399 => 0x3b9,
        0x3a3 | 0x3c2 => 0x3c3,
        0xb5 | 0x39c => 0x3bc,
        0x10400 => 0x10428,
        _ => point,
    }
}

fn word(point: u32, ignore_case: bool) -> bool {
    matches!(point, 0x30..=0x39 | 0x41..=0x5a | 0x5f | 0x61..=0x7a)
        || (ignore_case && matches!(point, 0x17f | 0x212a))
}

#[derive(Clone, Copy)]
struct Gates {
    anchors: [bool; 2],
    words: [Option<bool>; 2],
    ignore_case: bool,
    multiline: bool,
}

fn expected(
    members: &[u32],
    inverted: bool,
    gates: Gates,
    input: &Input,
    start: usize,
    sticky: bool,
) -> Option<Range<usize>> {
    if start > input.text.len() {
        return None;
    }
    // Locate the complete decoded character containing the requested UTF-16
    // offset. Thus a low-half start is compared at its leading boundary.
    let first = input.boundaries.partition_point(|&at| at <= start) - 1;
    let canonical = |point| {
        if gates.ignore_case {
            fold(point)
        } else {
            point
        }
    };
    let boundary = |at: usize| {
        let left = at.checked_sub(1).and_then(|at| input.points.get(at));
        let right = input.points.get(at);
        left.is_some_and(|&point| word(point, gates.ignore_case))
            != right.is_some_and(|&point| word(point, gates.ignore_case))
    };
    let line = |point| matches!(point, 0xa | 0xd | 0x2028 | 0x2029);
    for at in first..input.points.len() {
        if sticky && at != first {
            break;
        }
        let begin = input.boundaries[at];
        let end = input.boundaries[at + 1];
        let member = members
            .iter()
            .any(|&point| canonical(point) == canonical(input.points[at]));
        if member != inverted
            && (!gates.anchors[0] || at == 0 || (gates.multiline && line(input.points[at - 1])))
            && (!gates.anchors[1]
                || at + 1 == input.points.len()
                || (gates.multiline && line(input.points[at + 1])))
            && gates.words[0].is_none_or(|wanted| boundary(at) == wanted)
            && gates.words[1].is_none_or(|wanted| boundary(at + 1) == wanted)
        {
            return Some(begin..end);
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
) -> Result<Option<RegExpUnicodeClassMatcher>, RegExpUnicodeSetCompileError<usize>> {
    RegExpUnicodeClassMatcher::compile_single_character_strings_v_set_with_work(
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

fn compile(source: &str, ignore_case: bool, multiline: bool) -> RegExpUnicodeClassMatcher {
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
fn single_character_strings_match_independent_tiny_utf16_oracle() {
    let inputs = [
        JsString::from(""),
        JsString::from("aAKkKsſ!z"),
        JsString::from("𐐀𐐨😀😁"),
        JsString::from("\0\u{1}\u{8}\t\n\u{b}\u{c}\rabc"),
        JsString::from("! -|{}&# /[]?"),
        JsString::from("\r\nA\nK\u{2028}s\u{2029}!"),
        JsString::from("0a_A a!aa"),
        JsString::from("ΣςσΙι\u{345}µΜμ"),
        JsString::from("İıIıß"),
        JsString::from("😀a😀A😀"),
        JsString::from_code_units(vec![
            0xd800, 0x61, 0xdc00, 0xd83d, 0xde00, 0xd800, 0xd800, 0xdc00,
        ]),
    ]
    .map(Input::new);
    let cases: &[(&str, &[u32], bool)] = &[
        (r"[\q{A|K|s|K}]", &[0x41, 0x4b, 0x73, 0x4b], false),
        (r"[^\q{A|K|s}]", &[0x41, 0x4b, 0x73], true),
        (r"[\q{😀|𐐀}]", &[0x1f600, 0x10400], false),
        (
            r"[\q{\uD83D\uDE00|\uD801\uDC00}]",
            &[0x1f600, 0x10400],
            false,
        ),
        (r"[\q{\u{d800}|\u{dc00}}]", &[0xd800, 0xdc00], false),
        (
            r"[\q{\b|\!|\-|\||\{|\}|\&|\#}]",
            &[8, 0x21, 0x2d, 0x7c, 0x7b, 0x7d, 0x26, 0x23],
            false,
        ),
        (
            r"[\q{\0|\cA|\f|\n|\r|\t|\v|\x61|\u0062|\u{63}}]",
            &[0, 1, 0xc, 0xa, 0xd, 9, 0xb, 0x61, 0x62, 0x63],
            false,
        ),
        (
            r"[\q{\u{345}|\u{3a3}|\u{b5}|\u{10400}}]",
            &[0x345, 0x3a3, 0xb5, 0x10400],
            false,
        ),
    ];
    for &(body, members, inverted) in cases {
        for anchors in [[false, false], [true, false], [false, true], [true, true]] {
            for leading in [None, Some(false), Some(true)] {
                for trailing in [None, Some(false), Some(true)] {
                    let word_gate = |gate| match gate {
                        None => "",
                        Some(false) => r"\B",
                        Some(true) => r"\b",
                    };
                    let source = format!(
                        "{}{}{}{}{}",
                        if anchors[0] { "^" } else { "" },
                        word_gate(leading),
                        body,
                        word_gate(trailing),
                        if anchors[1] { "$" } else { "" }
                    );
                    for ignore_case in [false, true] {
                        for multiline in [false, true] {
                            let gates = Gates {
                                anchors,
                                words: [leading, trailing],
                                ignore_case,
                                multiline,
                            };
                            let original = compile(&source, ignore_case, multiline);
                            let matcher = original.clone();
                            drop(original);
                            for input in &inputs {
                                for start in 0..=input.text.len() + 1 {
                                    for sticky in [false, true] {
                                        assert_eq!(
                                            matcher.find(&input.text, start, sticky),
                                            expected(
                                                members, inverted, gates, input, start, sticky
                                            ),
                                            "{source} i={ignore_case} m={multiline} input={:?} start={start} sticky={sticky}",
                                            input.text
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
    // Both actual flag consumers and fold-before-inversion must change results.
    let kelvin_line = JsString::from("\nK\n");
    let gated = r"^\b[\q{K}]\b$";
    assert_eq!(
        compile(gated, true, true).find(&kelvin_line, 0, false),
        Some(1..2)
    );
    assert!(
        compile(gated, false, true)
            .find(&kelvin_line, 0, false)
            .is_none()
    );
    assert!(
        compile(gated, true, false)
            .find(&kelvin_line, 0, false)
            .is_none()
    );
    let folded_aliases = JsString::from("Kſ");
    assert!(
        compile(r"[^\q{K|s}]", true, false)
            .find(&folded_aliases, 0, false)
            .is_none()
    );
    assert_eq!(
        compile(r"[^\q{K|s}]", false, false).find(&folded_aliases, 0, false),
        Some(0..1)
    );
}

#[test]
fn single_character_strings_domains_escapes_and_work_prefixes() {
    // Independent small ledgers: one accepted interval writes once, merges once
    // and publishes once. A late invalid branch never starts normalization.
    for (text, supported, tail) in [
        (r"[\q{a}]", true, &[1, 1, 1][..]),
        (r"[\q{\u{0000000000000061}}]", true, &[1, 1, 1][..]),
        (r"[\q{}]", false, &[][..]),
        (r"[\q{ab}]", false, &[][..]),
        (r"[\q{|a}]", false, &[][..]),
        (r"[\q{a|}]", false, &[1][..]),
        (r"[\q{a|bc}]", false, &[1][..]),
        (r"[\q{a||b}]", false, &[1][..]),
    ] {
        let source = JsString::from(text);
        let mut seen = Vec::new();
        assert_eq!(
            recording(&source, false, false, &mut seen, None)
                .unwrap()
                .is_some(),
            supported
        );
        let mut ledger = vec![source.len() * 3];
        ledger.extend_from_slice(tail);
        assert_eq!(seen, ledger, "{text}");
    }
    for (text, supported, ignore_case, multiline) in [
        (r"[\q{a}]", true, false, false),
        (r"[\q{a|a|b}]", true, false, false),
        (r"^\b[^\q{K|s}]\B$", true, true, true),
        (r"[\q{\uD83D\uDE00|\u{d800}|\b}]", true, true, false),
        (r"[\q{a|bc}]", false, true, false),
        (r"[\q{a|}]", false, false, true),
        (r"[\q{\u{D800}\u{DC00}}]", false, false, false),
        (r"[\q{\uD800\u{DC00}}]", false, true, false),
        (r"[\q{\u{D800}\uDC00}]", false, false, true),
    ] {
        let source = JsString::from(text);
        let mut complete = Vec::new();
        assert_eq!(
            recording(&source, ignore_case, multiline, &mut complete, None)
                .unwrap()
                .is_some(),
            supported,
            "{text}"
        );
        assert_eq!(complete[0], source.len() * 3);
        for fail in 1..=complete.len() {
            let mut aborted = Vec::new();
            assert!(
                matches!(recording(&source, ignore_case, multiline, &mut aborted, Some(fail)),
                Err(RegExpUnicodeSetCompileError::Work(at)) if at == fail)
            );
            assert_eq!(aborted, complete[..fail], "{text} abort={fail}");
            let mut fresh = Vec::new();
            assert_eq!(
                recording(&source, ignore_case, multiline, &mut fresh, None)
                    .unwrap()
                    .is_some(),
                supported
            );
            assert_eq!(fresh, complete, "{text} after abort={fail}");
        }
    }
    for text in [
        "",
        "^",
        "$",
        "^$",
        "a",
        "[a]",
        r"\q{a}",
        r"[\q{a}b]",
        r"[a\q{b}]",
        r"[\q{a}\q{b}]",
        r"[[\q{a}]]",
        r"[\q{a}&&[a]]",
        r"[\q{a}--[a]]",
        r"[\q{a}]b",
        r"a[\q{b}]",
        r"[\q{a}][b]",
        r"[\q{a}]+",
        r"([\q{a}])",
        r"(?<whole>[\q{a}])",
        r"[\p{Basic_Emoji}]",
        r"\b^[\q{a}]",
        r"[\q{a}]$\b",
        r"\b\b[\q{a}]",
        r"[\q{a}]\b\B",
        r"[\q{\d}]",
        r"[\q{\w}]",
        r"[\q{\s}]",
        r"[\q{\B}]",
        r"[\q{\p{ASCII}}]",
        r"[\q{\P{ASCII}}]",
        r"[\q{\q}]",
        r"[\q{\1}]",
        r"[\q{\01}]",
        r"[\q{\a}]",
        r"[\q{\u{110000}}]",
        r"[\q{(}]",
        r"[\q{-}]",
        r"[\q{[}]",
        r"[\q{/}]",
    ] {
        let source = JsString::from(text);
        let mut seen = Vec::new();
        assert!(
            recording(&source, true, true, &mut seen, None)
                .unwrap()
                .is_none(),
            "{text}"
        );
        assert_eq!(seen, [source.len() * 3], "{text}");
        let mut aborted = Vec::new();
        assert!(matches!(
            recording(&source, true, true, &mut aborted, Some(1)),
            Err(RegExpUnicodeSetCompileError::Work(1))
        ));
        assert_eq!(aborted, seen);
    }
    for (escape, unit) in [
        (r"\&", b'&'),
        (r"\-", b'-'),
        (r"\!", b'!'),
        (r"\#", b'#'),
        (r"\%", b'%'),
        (r"\,", b','),
        (r"\:", b':'),
        (r"\;", b';'),
        (r"\<", b'<'),
        (r"\=", b'='),
        (r"\>", b'>'),
        (r"\@", b'@'),
        (r"\`", b'`'),
        (r"\~", b'~'),
        (r"\^", b'^'),
        (r"\$", b'$'),
        (r"\\", b'\\'),
        (r"\.", b'.'),
        (r"\*", b'*'),
        (r"\+", b'+'),
        (r"\?", b'?'),
        (r"\(", b'('),
        (r"\)", b')'),
        (r"\[", b'['),
        (r"\]", b']'),
        (r"\{", b'{'),
        (r"\}", b'}'),
        (r"\|", b'|'),
        (r"\/", b'/'),
    ] {
        let source = format!(r"[\q{{{escape}}}]");
        let input = JsString::from_code_units(vec![u16::from(unit)]);
        assert_eq!(
            compile(&source, false, false).find(&input, 0, true),
            Some(0..1)
        );
        assert!(
            compile(&source, false, false)
                .find(&JsString::from("z"), 0, true)
                .is_none()
        );
    }
    // Raw lone source units are admitted, but may not consume half an input pair.
    let mut units = r"[\q{".encode_utf16().collect::<Vec<_>>();
    units.extend([0xd800, 0x7c, 0xdc00]);
    units.extend("}]".encode_utf16());
    let source = JsString::from_code_units(units);
    let matcher = recording(&source, false, false, &mut Vec::new(), None)
        .unwrap()
        .unwrap();
    assert_eq!(
        matcher.find(&JsString::from_code_units(vec![0xd800]), 0, true),
        Some(0..1)
    );
    assert_eq!(
        matcher.find(&JsString::from_code_units(vec![0xdc00]), 0, true),
        Some(0..1)
    );
    let pair = JsString::from_code_units(vec![0xd800, 0xdc00]);
    for start in 0..=pair.len() + 1 {
        assert!(matcher.find(&pair, start, false).is_none());
        assert!(matcher.find(&pair, start, true).is_none());
    }
    // New q support does not widen old public flat/checked algebra entries.
    for text in [
        r"[\q{a}]",
        r"[^\q{K|s}]",
        r"^\b[\q{a}]\b$",
        r"[[\q{a}]]",
        r"[\q{a}&&[a]]",
        r"[\q{a}--[a]]",
    ] {
        let source = JsString::from(text);
        let accept: fn(usize) -> Result<(), usize> = |_| Ok(());
        assert!(
            RegExpUnicodeClassMatcher::compile_with_flags_and_work(
                &source, true, true, true, accept
            )
            .unwrap()
            .is_none(),
            "{text}"
        );
        assert!(
            RegExpUnicodeClassMatcher::compile_with_word_boundaries_and_work(
                &source, true, true, true, accept
            )
            .unwrap()
            .is_none(),
            "{text}"
        );
        assert!(
            RegExpUnicodeClassMatcher::compile_v_set_with_work(&source, true, true, accept)
                .unwrap()
                .is_none(),
            "{text}"
        );
        assert!(
            RegExpUnicodeClassMatcher::compile_v_set_with_word_boundaries_and_work(
                &source, true, true, accept
            )
            .unwrap()
            .is_none(),
            "{text}"
        );
    }
}
