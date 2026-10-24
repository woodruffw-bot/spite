//! Unicode atom quantifiers and proved boundary assertions (22.2.2.3, 22.2.2.4).
use crate::{
    JsString, RegExpUnicodeCharacterMatcher, RegExpUnicodeClassMatcher,
    regexp_literal::unicode_start,
    regexp_unicode_character::{
        UnicodeWordAssertions, unicode_input_character, unicode_word_assertions_with_work,
        unicode_word_boundary,
    },
};
use std::ops::Range;

#[path = "regexp_unicode_nullable_class_captures.rs"]
mod nullable_class_captures;
pub use nullable_class_captures::{
    RegExpUnicodeAssertedCountedNullableScalarStringClassCaptureMatcher,
    RegExpUnicodeAssertedCountedNullableStringClassCaptureMatcher,
    RegExpUnicodeAssertedNullableStringClassCaptureMatcher,
    RegExpUnicodeAssertedRepeatedStringClassCaptureMatcher,
    RegExpUnicodeAssertedStringClassAlternativeMatcher,
    RegExpUnicodeAssertedTwoRequiredNullableStringClassCaptureMatcher,
    RegExpUnicodeBoundedStringClassCaptureMatcher,
    RegExpUnicodeConjunctivePredicateChoiceSingleStringClassCaptureMatcher,
    RegExpUnicodeConjunctivePredicateChoiceStringClassAlternativeMatcher,
    RegExpUnicodeConjunctivePredicateChoiceStringClassCaptureMatcher,
    RegExpUnicodeCountedNullableStringClassCaptureMatcher,
    RegExpUnicodeEmptyStringClassCaptureMatcher, RegExpUnicodeMatchError,
    RegExpUnicodeNullableClassCaptureMatcher, RegExpUnicodeNullableStringClassCaptureMatcher,
    RegExpUnicodePredicateChoiceSingleStringClassCaptureMatcher,
    RegExpUnicodePredicateChoiceStringClassAlternativeMatcher,
    RegExpUnicodePredicateChoiceStringClassCaptureMatcher,
    RegExpUnicodePredicateChoiceStringClassMatch, RegExpUnicodePrefixFreeStringClassCaptureMatcher,
    RegExpUnicodeRepeatedClassCaptureMatcher, RegExpUnicodeRepeatedStringClassCaptureMatcher,
    RegExpUnicodeRepeatedStringClassMatch, RegExpUnicodeRequiredStringClassCaptureMatcher,
    RegExpUnicodeSingleStringClassCaptureMatcher, RegExpUnicodeStringClassBranchMatch,
    RegExpUnicodeSurroundingAssertedStringClassAlternativeMatcher,
    RegExpUnicodeTruthTableCountedNullableStringClassCaptureMatcher,
    RegExpUnicodeTruthTableExactOneGroupedStringClassAlternativeMatcher,
    RegExpUnicodeTruthTableGroupedStringClassAlternativeMatcher,
    RegExpUnicodeTruthTableOptionalStringClassAlternativeMatcher,
    RegExpUnicodeTruthTableRootStringClassAlternativeMatcher,
    RegExpUnicodeTruthTableScopedStringClassAlternativeMatcher,
    RegExpUnicodeTruthTableSingleStringClassCaptureMatcher,
    RegExpUnicodeTruthTableStringClassAlternativeMatcher,
    RegExpUnicodeTruthTableStringClassCaptureMatcher,
    RegExpUnicodeTruthTableSurroundingStringClassAlternativeMatcher,
    RegExpUnicodeTwoRequiredStringClassCaptureMatcher,
    RegExpUnicodeUnassertedZeroCountSingleStringClassCaptureMatcher,
    RegExpUnicodeUnassertedZeroCountStringClassAlternativeMatcher,
    RegExpUnicodeUnassertedZeroCountStringClassCaptureMatcher,
    RegExpUnicodeUnassertedZeroMinimumSingleStringClassCaptureMatcher,
    RegExpUnicodeUnassertedZeroMinimumStringClassAlternativeMatcher,
    RegExpUnicodeUnassertedZeroMinimumStringClassCaptureMatcher,
    RegExpUnicodeZeroCountAssertedSingleStringClassCaptureMatcher,
    RegExpUnicodeZeroCountAssertedStringClassAlternativeMatcher,
    RegExpUnicodeZeroCountAssertedStringClassCaptureMatcher,
    RegExpUnicodeZeroMinimumAssertedSingleStringClassCaptureMatcher,
    RegExpUnicodeZeroMinimumAssertedStringClassAlternativeMatcher,
    RegExpUnicodeZeroMinimumAssertedStringClassCaptureMatcher,
};

/// A selected Unicode repetition and the last captured atom iteration.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RegExpUnicodeRepeatedAtomMatch {
    /// Original UTF-16 range of the complete repetition.
    pub range: Range<usize>,
    /// Last selected iteration, or None without inner captures or iterations.
    pub atom_capture: Option<Range<usize>>,
}

/// A complete single consuming Unicode atom with one greedy or lazy quantifier.
///
/// The caller must validate the whole u/v Pattern. Dedicated entry points admit
/// outside line and word assertions, and complete wrappers inside the repeated
/// atom. Scoped entry points preserve word and line predicates at each consuming
/// iteration. A dedicated entry point proves alternatives whose branches each
/// consume one Unicode character. A separate proof admits one raw empty
/// alternative alongside a character under *, + or ?. Concatenations remain excluded.
/// Complete enclosing groups may be proved separately by the caller.
/// Construction and search add no native recursion or input-sized storage;
/// work quotas remain opt-in.
#[derive(Clone, Debug)]
pub struct RegExpUnicodeRepeatedAtomMatcher {
    atom: Atom,
    minimum: Option<usize>,
    maximum: Option<usize>,
    greedy: bool,
    start_anchor: bool,
    multiline: bool,
    end_anchor: bool,
    atom_captures: usize,
    start_word: Option<bool>,
    end_word: Option<bool>,
    word_ignore_case: bool,
    nullable: Option<EmptyBranchOrder>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum EmptyBranchOrder {
    First,
    Last,
    FirstFiniteOptional(usize),
}

#[derive(Clone, Copy)]
enum ClassProof {
    Checked,
    SingleCharacterStrings,
    ScalarLeftStrings,
    CharacterResultStrings,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum NullableProof {
    Raw,
    Counted,
    TransparentEmpty,
    ZeroMinimum,
    OneMinimumOpen,
    OneIteration,
    PositiveOpen,
    PositiveExact,
    PositiveFinite,
    UnrepresentableExact,
    UnrepresentableFinite,
    OverflowingMaximum,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum EmptyBranchProof {
    Raw,
    Nested,
    Sequences,
    QuantifiedSequences,
    Choices,
}

#[derive(Clone, Copy)]
struct NullableOptions {
    bounds: NullableProof,
    empty: EmptyBranchProof,
}

impl From<NullableProof> for NullableOptions {
    fn from(bounds: NullableProof) -> Self {
        Self {
            bounds,
            empty: if bounds == NullableProof::Raw {
                EmptyBranchProof::Raw
            } else {
                EmptyBranchProof::Nested
            },
        }
    }
}

#[derive(Clone, Copy)]
struct QuantifierBounds {
    minimum: Option<usize>,
    maximum: Option<usize>,
    greedy: bool,
}

#[cfg(test)]
mod v_set_checked_repetition_tests {
    use super::*;

    fn plan(source: &str, ignore_case: bool, multiline: bool) -> RegExpUnicodeRepeatedAtomMatcher {
        RegExpUnicodeRepeatedAtomMatcher::compile_v_set_with_line_anchors_and_work(
            &JsString::from(source),
            ignore_case,
            multiline,
            |_| Ok::<(), ()>(()),
        )
        .unwrap()
        .unwrap()
    }

    fn find(
        plan: &RegExpUnicodeRepeatedAtomMatcher,
        input: &JsString,
        start: usize,
        sticky: bool,
    ) -> Option<Range<usize>> {
        plan.find_with_work(input, start, sticky, |_| Ok::<(), ()>(()))
            .unwrap()
    }

    #[test]
    fn checked_v_set_repetition_empty_and_finite_bounds() {
        for (source, input, expected) in [
            ("[a--a]+", "", None),
            ("[a--a]+", "a", None),
            ("^[a--a]*$", "", Some(0..0)),
            ("^[a--a]*$", "a", None),
            ("^[a--a]{0,0}$", "", Some(0..0)),
            ("^[a--a]{1,3}$", "", None),
            ("^[a--b]{2,3}$", "a", None),
            ("^[a--b]{2,3}$", "aa", Some(0..2)),
            ("^[a--b]{2,3}$", "aaaa", None),
            ("^[a--b]{0002,0003}$", "aaa", Some(0..3)),
        ] {
            assert_eq!(
                find(&plan(source, false, false), &JsString::from(input), 0, true),
                expected,
                "{source} {input:?}"
            );
        }
    }

    #[test]
    fn checked_v_set_repetition_greediness_and_unrepresentable_bounds() {
        for (quantifier, end) in [
            ("+", 3),
            ("+?", 1),
            ("*", 3),
            ("*?", 0),
            ("?", 1),
            ("??", 0),
            ("{1,2}", 2),
            ("{1,2}?", 1),
        ] {
            let matcher = plan(&format!("[a--b]{quantifier}"), false, false);
            assert_eq!(
                find(&matcher, &JsString::from("aaa"), 0, true),
                Some(0..end)
            );
        }
        assert_eq!(
            find(
                &plan("^[a--b]+?$", false, false),
                &JsString::from("aaa"),
                0,
                true
            ),
            Some(0..3)
        );
        let huge = "9".repeat(100);
        let impossible = plan(&format!("[a--b]{{{huge}}}"), false, false);
        assert_eq!(impossible.minimum, None);
        assert_eq!(find(&impossible, &JsString::from("aa"), 0, false), None);
        let unbounded = plan(&format!("^[a--b]{{1,{huge}}}$"), false, false);
        assert_eq!(unbounded.minimum, Some(1));
        assert_eq!(unbounded.maximum, None);
        assert_eq!(
            find(&unbounded, &JsString::from("aaa"), 0, true),
            Some(0..3)
        );
        let native_bound = plan(&format!("[a--b]{{{}}}", usize::MAX), false, false);
        assert_eq!(native_bound.minimum, Some(usize::MAX));
        assert_eq!(find(&native_bound, &JsString::from("aa"), 0, false), None);
    }

    #[test]
    fn checked_v_set_repetition_line_contexts_and_unicode_widths() {
        let input = JsString::from("x\naaa\n");
        assert_eq!(
            find(&plan("^[a--b]+$", false, false), &input, 0, false),
            None
        );
        assert_eq!(
            find(&plan("^[a--b]+$", false, true), &input, 0, false),
            Some(2..5)
        );
        assert_eq!(
            find(
                &plan("^[a--b]+$", false, true),
                &JsString::from("\u{2028}aa\u{2029}"),
                0,
                false,
            ),
            Some(1..3)
        );
        assert_eq!(
            find(
                &plan("^[A&&a]+$", false, false),
                &JsString::from("Aa"),
                0,
                true
            ),
            None
        );
        assert_eq!(
            find(
                &plan("^[A&&a]+$", true, false),
                &JsString::from("Aa"),
                0,
                true
            ),
            Some(0..2)
        );
        let source = r"^[[\u{10400}-\u{10427}]&&[\u{10400}]]+$";
        let matcher = plan(source, false, false);
        assert_eq!(find(&matcher, &JsString::from("𐐀𐐀"), 0, true), Some(0..4));
        // Retain the approved whole-match leading-boundary behavior inside a pair.
        assert_eq!(find(&matcher, &JsString::from("𐐀𐐀"), 1, true), Some(0..4));
        assert_eq!(
            find(&plan(source, true, false), &JsString::from("𐐨𐐨"), 0, true),
            Some(0..4)
        );
        assert_eq!(
            find(
                &plan(r"^[[\uD800]--[a]]+$", false, false),
                &JsString::from_code_units(vec![0xd800]),
                0,
                true,
            ),
            Some(0..1)
        );
    }

    #[test]
    fn checked_v_set_repetition_complete_body_and_legacy_contracts() {
        for source in ["[[a]]+", "[[a--b]&&a]+"] {
            let matcher =
                RegExpUnicodeRepeatedAtomMatcher::compile_v_set_with_line_anchors_and_work(
                    &JsString::from(source),
                    false,
                    false,
                    |_| Ok::<(), ()>(()),
                )
                .unwrap()
                .unwrap();
            assert_eq!(find(&matcher, &JsString::from("aab"), 0, true), Some(0..2));
            assert_eq!(find(&matcher, &JsString::from("b"), 0, true), None);
        }
        for source in [
            "[a--b]",
            "([a--b])+",
            "a[a--b]+",
            "[a--b]+b",
            "^[a--b]+$x",
            r"\b[a--b]+\b",
            "[a--b]+|c",
            r"[a--b]\+",
            r"[a--a--\q{a}]+",
            r"[a&&\p{RGI_Emoji}]+",
        ] {
            assert!(
                RegExpUnicodeRepeatedAtomMatcher::compile_v_set_with_line_anchors_and_work(
                    &JsString::from(source),
                    false,
                    false,
                    |_| Ok::<(), ()>(()),
                )
                .unwrap()
                .is_none(),
                "{source}"
            );
        }
        assert!(
            RegExpUnicodeRepeatedAtomMatcher::compile_with_line_anchors_and_work(
                &JsString::from("^[a--b]+$"),
                true,
                false,
                false,
                false,
                |_| Ok::<(), ()>(()),
            )
            .unwrap()
            .is_none()
        );
    }

    #[test]
    fn checked_v_set_repetition_work_errors_preserve_prefix() {
        let source = JsString::from(r"^[[A-Z]--[Q]]{2,4}?$");
        let mut charges = Vec::new();
        RegExpUnicodeRepeatedAtomMatcher::compile_v_set_with_line_anchors_and_work(
            &source,
            true,
            true,
            |work| {
                charges.push(work);
                Ok::<(), usize>(())
            },
        )
        .unwrap()
        .unwrap();
        for failure in 1..=charges.len() {
            let mut seen = Vec::new();
            let result = RegExpUnicodeRepeatedAtomMatcher::compile_v_set_with_line_anchors_and_work(
                &source,
                true,
                true,
                |work| {
                    seen.push(work);
                    if seen.len() == failure {
                        Err(failure)
                    } else {
                        Ok(())
                    }
                },
            );
            assert!(
                matches!(result, Err(crate::RegExpUnicodeSetCompileError::Work(at)) if at == failure)
            );
            assert_eq!(seen, charges[..failure]);
        }
    }
    #[test]
    fn checked_v_set_repetition_word_endpoints_match_sparse_reference() {
        #[derive(Clone, Copy)]
        struct Gates {
            before: &'static str,
            after: &'static str,
            start_word: Option<bool>,
            end_word: Option<bool>,
            start_anchor: bool,
            end_anchor: bool,
            multiline: bool,
        }
        struct Input {
            text: JsString,
            points: Vec<u32>,
            boundaries: Vec<usize>,
        }
        struct Reference {
            class: u8,
            minimum: usize,
            maximum: Option<usize>,
            greedy: bool,
            gates: Gates,
            ignore_case: bool,
        }
        fn boundary(units: &[u16], at: usize, ignore_case: bool) -> bool {
            let word = |unit: Option<&u16>| {
                unit.is_some_and(|&unit| {
                    matches!(unit, 0x30..=0x39 | 0x41..=0x5a | 0x5f | 0x61..=0x7a)
                        || (ignore_case && matches!(unit, 0x17f | 0x212a))
                })
            };
            word(at.checked_sub(1).and_then(|at| units.get(at))) != word(units.get(at))
        }
        fn member(class: u8, point: u32, ignore_case: bool) -> bool {
            match class {
                0 => point == 0x21 || point == 0x61 || (ignore_case && point == 0x41),
                // Independent C/S preimages: K/k/Kelvin and long-s/s/S.
                1 => {
                    matches!(point, 0x4b | 0x17f | 0x1f600 | 0xd800 | 0xdc00)
                        || (ignore_case && matches!(point, 0x6b | 0x212a | 0x73 | 0x53))
                }
                2 => false,
                _ => unreachable!(),
            }
        }
        fn expected(
            reference: &Reference,
            input: &Input,
            start: usize,
            sticky: bool,
        ) -> Option<Range<usize>> {
            let units = input.text.code_units();
            if start > units.len() {
                return None;
            }
            let first = input.boundaries.partition_point(|&at| at <= start) - 1;
            let gates = reference.gates;
            for candidate in first..input.boundaries.len() {
                if sticky && candidate != first {
                    break;
                }
                let begin = input.boundaries[candidate];
                if (gates.start_anchor
                    && begin != 0
                    && !(gates.multiline
                        && matches!(units[begin - 1], 0x0a | 0x0d | 0x2028 | 0x2029)))
                    || gates
                        .start_word
                        .is_some_and(|word| boundary(units, begin, reference.ignore_case) != word)
                {
                    continue;
                }
                let mut best = None;
                for count in 0..=input.points.len() - candidate {
                    if reference.maximum.is_some_and(|maximum| count > maximum) {
                        break;
                    }
                    let end = input.boundaries[candidate + count];
                    let line_end = !gates.end_anchor
                        || end == units.len()
                        || (gates.multiline && matches!(units[end], 0x0a | 0x0d | 0x2028 | 0x2029));
                    let word_end = gates
                        .end_word
                        .is_none_or(|word| boundary(units, end, reference.ignore_case) == word);
                    if count >= reference.minimum && line_end && word_end {
                        best = Some(begin..end);
                        if !reference.greedy {
                            return best;
                        }
                    }
                    if candidate + count == input.points.len()
                        || !member(
                            reference.class,
                            input.points[candidate + count],
                            reference.ignore_case,
                        )
                    {
                        break;
                    }
                }
                if best.is_some() {
                    return best;
                }
            }
            None
        }
        let inputs: Vec<_> = [
            JsString::from(""),
            JsString::from("aa!"),
            JsString::from("aa_!"),
            JsString::from("KkKſsS!"),
            JsString::from("a😀B"),
            JsString::from_code_units(vec![0x61, 0xd800, 0x21, 0xdc00, 0x61]),
            JsString::from("x\r\nAA\u{2028}"),
            JsString::from("Kſ"),
        ]
        .into_iter()
        .map(|text| {
            let units = text.code_units();
            let mut points = Vec::new();
            let mut boundaries = vec![0];
            let mut at = 0;
            while at < units.len() {
                let first = units[at];
                at += 1;
                let point = if (0xd800..=0xdbff).contains(&first)
                    && units
                        .get(at)
                        .is_some_and(|last| (0xdc00..=0xdfff).contains(last))
                {
                    let point =
                        0x10000 + 0x400 * u32::from(first - 0xd800) + u32::from(units[at] - 0xdc00);
                    at += 1;
                    point
                } else {
                    u32::from(first)
                };
                points.push(point);
                boundaries.push(at);
            }
            Input {
                text,
                points,
                boundaries,
            }
        })
        .collect();
        let both = Gates {
            before: r"\b",
            after: r"\b",
            start_word: Some(true),
            end_word: Some(true),
            start_anchor: false,
            end_anchor: false,
            multiline: false,
        };
        let gates = [
            both,
            Gates {
                after: r"\B",
                end_word: Some(false),
                ..both
            },
            Gates {
                before: "",
                start_word: None,
                ..both
            },
            Gates {
                after: "",
                end_word: None,
                ..both
            },
            Gates {
                before: r"\B",
                after: r"\B",
                start_word: Some(false),
                end_word: Some(false),
                ..both
            },
            Gates {
                before: r"^\b",
                after: r"\b$",
                start_anchor: true,
                end_anchor: true,
                ..both
            },
            Gates {
                before: r"^\b",
                after: r"\b$",
                start_anchor: true,
                end_anchor: true,
                multiline: true,
                ..both
            },
        ];
        for (class, body) in [
            (0, "[[a!]]"),
            (1, r"[[Kſ\u{1F600}][\uD800][\uDC00]]"),
            (2, "[a--a]"),
        ] {
            for (quantifier, minimum, maximum, greedy) in [
                ("+", 1, None, true),
                ("+?", 1, None, false),
                ("*", 0, None, true),
                ("*?", 0, None, false),
                ("{1,2}", 1, Some(2), true),
                ("{0}", 0, Some(0), true),
            ] {
                for gates in gates {
                    let source = format!("{}{body}{quantifier}{}", gates.before, gates.after);
                    for ignore_case in [false, true] {
                        let matcher = RegExpUnicodeRepeatedAtomMatcher::compile_v_set_with_word_boundaries_and_work(
                            &JsString::from(source.as_str()), ignore_case, gates.multiline,
                            |_| Ok::<(), ()>(()),
                        ).unwrap().unwrap();
                        let reference = Reference {
                            class,
                            minimum,
                            maximum,
                            greedy,
                            gates,
                            ignore_case,
                        };
                        for input in &inputs {
                            for start in 0..=input.text.len() + 1 {
                                for sticky in [false, true] {
                                    assert_eq!(
                                        find(&matcher, &input.text, start, sticky),
                                        expected(&reference, input, start, sticky),
                                        "{source} i={ignore_case} m={} {:?} start={start} sticky={sticky}",
                                        gates.multiline,
                                        input.text,
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn checked_v_set_repetition_word_contracts_and_work_prefixes() {
        fn checked(
            source: &str,
            ignore_case: bool,
            multiline: bool,
        ) -> RegExpUnicodeRepeatedAtomMatcher {
            RegExpUnicodeRepeatedAtomMatcher::compile_v_set_with_word_boundaries_and_work(
                &JsString::from(source),
                ignore_case,
                multiline,
                |_| Ok::<(), ()>(()),
            )
            .unwrap()
            .unwrap()
        }
        for (source, input, start, sticky, ignore_case, multiline, expected) in [
            (r"\b[[a]]?\b", "aa", 0, true, false, false, Some(0..0)),
            (r"\b[[a!]]+\b", "a!", 0, true, false, false, Some(0..1)),
            (r"\b[[a]]+?\b", "aa!", 0, true, false, false, Some(0..2)),
            (r"[[a]]{1,2}\b", "aaa", 0, false, false, false, Some(1..3)),
            (r"[[a]]{1,2}\b", "aaa", 0, true, false, false, None),
            (r"\B[a--a]{0}\B", "", 0, true, false, false, Some(0..0)),
            (
                r"\b[[\p{Any}]]+\B",
                "a😀",
                2,
                true,
                false,
                false,
                Some(1..3),
            ),
            (
                r"\b[[\p{Any}]]{0}\b",
                "a😀",
                2,
                true,
                false,
                false,
                Some(1..1),
            ),
            (
                r"^\b[[a]]{1,2}?\b$",
                "x\naa\n",
                0,
                false,
                false,
                true,
                Some(2..4),
            ),
            (r"\b[[ſ]]+\b", "ſ", 0, true, false, false, None),
            (r"\b[[ſ]]+\b", "ſ", 0, true, true, false, Some(0..1)),
        ] {
            assert_eq!(
                find(
                    &checked(source, ignore_case, multiline),
                    &JsString::from(input),
                    start,
                    sticky
                ),
                expected,
                "{source} {input:?} start={start} sticky={sticky} i={ignore_case} m={multiline}",
            );
        }
        let huge = "9".repeat(100);
        for (quantifier, expected) in [
            (format!("{{{huge}}}"), None),
            (format!("{{0,{huge}}}?"), Some(0..0)),
            (format!("{{1,{huge}}}"), Some(0..2)),
            (format!("{{{}}}", usize::MAX), None),
        ] {
            let source = format!(r"\b[[a]]{quantifier}\b");
            assert_eq!(
                find(
                    &checked(&source, false, false),
                    &JsString::from("aa"),
                    0,
                    true
                ),
                expected
            );
        }
        for source in [
            "[[a]]+",
            r"\b[[a]]\b",
            r"\b[a]+\b",
            r"\b^[[a]]+\b",
            r"\b[[a]]+$\b",
            r"\b\B[[a]]+",
            r"[[a]]+\b\B",
            r"a\b[[a]]+",
            r"\b[[a]]+a",
            r"(\b[[a]]+\b)",
            r"\b([[a]])+\b",
            r"\b[a--a--\q{a}]{0}\b",
            r"\B[\p{RGI_Emoji}--\p{RGI_Emoji}]{0}\B",
            r"\b[[a]]+\\b",
            r"\b[[a]]+\$",
            r"\b[[a]]\+\b",
            r"^\b[[a]]+\b$$",
        ] {
            assert!(
                RegExpUnicodeRepeatedAtomMatcher::compile_v_set_with_word_boundaries_and_work(
                    &JsString::from(source),
                    true,
                    true,
                    |_| Ok::<(), ()>(()),
                )
                .unwrap()
                .is_none(),
                "{source}"
            );
        }
        for (source, ignore_case) in [
            (r"\b[[\p{ASCII}\dA-Z]--[Q]]{1,2}?\b", false),
            (r"^\b[[K]&&[k]]+\b$", true),
        ] {
            let source = JsString::from(source);
            let mut charges = Vec::new();
            RegExpUnicodeRepeatedAtomMatcher::compile_v_set_with_word_boundaries_and_work(
                &source,
                ignore_case,
                true,
                |work| {
                    charges.push(work);
                    Ok::<(), usize>(())
                },
            )
            .unwrap()
            .unwrap();
            for failure in 1..=charges.len() {
                let mut seen = Vec::new();
                let result =
                    RegExpUnicodeRepeatedAtomMatcher::compile_v_set_with_word_boundaries_and_work(
                        &source,
                        ignore_case,
                        true,
                        |work| {
                            seen.push(work);
                            if seen.len() == failure {
                                Err(failure)
                            } else {
                                Ok(())
                            }
                        },
                    );
                assert!(
                    matches!(result, Err(crate::RegExpUnicodeSetCompileError::Work(at)) if at == failure)
                );
                assert_eq!(seen, charges[..failure]);
            }
        }
        let matcher = checked(r"\b[[a_]]+\B", false, false);
        let input = JsString::from(" aa_!");
        let mut charges = Vec::new();
        assert_eq!(
            matcher
                .find_with_work(&input, 0, false, |work| {
                    charges.push(work);
                    Ok::<(), usize>(())
                })
                .unwrap(),
            Some(1..3)
        );
        for failure in 1..=charges.len() {
            let mut seen = Vec::new();
            let result = matcher.find_with_work(&input, 0, false, |work| {
                seen.push(work);
                if seen.len() == failure {
                    Err(failure)
                } else {
                    Ok(())
                }
            });
            assert!(matches!(result, Err(at) if at == failure));
            assert_eq!(seen, charges[..failure]);
        }
    }
}

#[cfg(test)]
mod start_assertion_tests {
    use super::*;
    use std::fmt::Write;

    fn plan(source: &str, v: bool, i: bool, m: bool, s: bool) -> RegExpUnicodeRepeatedAtomMatcher {
        RegExpUnicodeRepeatedAtomMatcher::compile_with_start_anchor_and_work(
            &JsString::from(source),
            v,
            i,
            m,
            s,
            |_| Ok::<(), ()>(()),
        )
        .unwrap()
        .unwrap()
    }

    const CASES: [(&str, u8, usize, Option<usize>, bool); 10] = [
        ("^a+", 0, 1, None, true),
        ("^a*?", 0, 0, None, false),
        ("^a??", 0, 0, Some(1), false),
        ("^a{2,3}", 0, 2, Some(3), true),
        ("^.+", 1, 1, None, true),
        (r"^\D{1,2}", 2, 1, Some(2), true),
        (r"^\W+", 3, 1, None, true),
        (r"^\p{Join_C}+", 4, 1, None, true),
        (r"^\p{NChar}{1,2}", 5, 1, Some(2), true),
        ("^[aſK]+", 6, 1, None, true),
    ];

    #[test]
    fn unicode_repeated_atom_start_ranges_snapshot() {
        let mut rows = String::new();
        for (source, ..) in CASES {
            let pattern = JsString::from(source);
            for v in [false, true] {
                for i in [false, true] {
                    for m in [false, true] {
                        for s in [false, true] {
                            let matcher = plan(source, v, i, m, s);
                            for input in [
                                JsString::from("B\raaa\nAAſK\u{2028}"),
                                JsString::from("\r\n\u{200c}\u{200d}\u{ffff}\u{10ffff}"),
                                JsString::from("😀\naaa"),
                                JsString::from_code_units(vec![0x61, 0xd800, 0x0a, 0x61, 0xdc00]),
                                JsString::from(""),
                            ] {
                                for start in [0, 1, 2, input.len(), input.len() + 1] {
                                    for sticky in [false, true] {
                                        writeln!(rows,"{pattern:?} v={v} i={i} m={m} s={s} input={input:?} start={start} sticky={sticky} {:?}", matcher.find_with_work(&input,start,sticky,|_|Ok::<(),()>(())).unwrap()).unwrap();
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        insta::assert_snapshot!(rows);
    }

    #[test]
    fn unicode_repeated_atom_start_matches_independent_candidate_oracle() {
        let alphabet: [u16; 10] = [
            0x41, 0x61, 0x0a, 0x0d, 0x2028, 0x17f, 0x200c, 0xffff, 0xd83d, 0xde00,
        ];
        let lt = |c| matches!(c, 0x0a | 0x0d | 0x2028 | 0x2029);
        for i in [false, true] {
            for m in [false, true] {
                for s in [false, true] {
                    let fold = |c| {
                        if i {
                            match c {
                                0x41 => 0x61,
                                0x17f => 0x73,
                                0x212a => 0x6b,
                                _ => c,
                            }
                        } else {
                            c
                        }
                    };
                    let member = |kind, c| match kind {
                        0 => fold(c) == 0x61,
                        1 => s || !lt(c),
                        2 => !(0x30..=0x39).contains(&c),
                        3 => !char::from_u32(fold(c))
                            .is_some_and(|c| c.is_ascii_alphanumeric() || c == '_'),
                        4 => (0x200c..=0x200d).contains(&c),
                        5 => matches!(c, 0xfdd0..=0xfdef) || (c & 0xffff) >= 0xfffe,
                        6 => [0x61, fold(0x17f), fold(0x212a)].contains(&fold(c)),
                        _ => unreachable!(),
                    };
                    let plans: Vec<_> = CASES.iter().map(|c| plan(c.0, false, i, m, s)).collect();
                    for len in 0..=4 {
                        for ordinal in 0..alphabet.len().pow(len) {
                            let units: Vec<_> = (0..len)
                                .map(|j| alphabet[ordinal / alphabet.len().pow(j) % alphabet.len()])
                                .collect();
                            let mut points = Vec::new();
                            let mut bounds = vec![0];
                            let mut j = 0;
                            while j < units.len() {
                                let mut c = u32::from(units[j]);
                                j += 1;
                                if (0xd800..=0xdbff).contains(&c)
                                    && j < units.len()
                                    && (0xdc00..=0xdfff).contains(&units[j])
                                {
                                    c = 65536 + (c - 0xd800) * 1024 + u32::from(units[j]) - 0xdc00;
                                    j += 1;
                                }
                                points.push(c);
                                bounds.push(j);
                            }
                            let input = JsString::from_code_units(units);
                            for start in 0..=input.len() + 1 {
                                for sticky in [false, true] {
                                    for (case, matcher) in CASES.iter().zip(&plans) {
                                        let expected = if start > input.len() {
                                            None
                                        } else {
                                            let first = bounds.partition_point(|&b| b <= start) - 1;
                                            (first..=points.len()).find_map(|at| {
                                                if (sticky && at != first)
                                                    || (at != 0 && !(m && lt(points[at - 1])))
                                                {
                                                    return None;
                                                }
                                                let available = points[at..]
                                                    .iter()
                                                    .take_while(|&&c| member(case.1, c))
                                                    .count();
                                                let count =
                                                    available.min(case.3.unwrap_or(available));
                                                if count < case.2 {
                                                    return None;
                                                }
                                                Some(
                                                    bounds[at]
                                                        ..bounds[at
                                                            + if case.4 { count } else { case.2 }],
                                                )
                                            })
                                        };
                                        assert_eq!(
                                            matcher
                                                .find_with_work(&input, start, sticky, |_| Ok::<
                                                    (),
                                                    (),
                                                >(
                                                    ()
                                                ))
                                                .unwrap(),
                                            expected,
                                            "{} i={i} m={m} s={s} {input:?} start={start} sticky={sticky}",
                                            case.0
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

    #[test]
    fn unicode_repeated_atom_start_linear_work_clone_and_contracts() {
        let input = JsString::from("aaB\n".repeat(50000).as_str());
        let mut work = 0;
        assert_eq!(
            plan("^a{3}", true, false, true, false)
                .clone()
                .find_with_work(&input, 0, false, |n| {
                    work += n;
                    Ok::<(), ()>(())
                })
                .unwrap(),
            None
        );
        assert!(work < 6 * input.len(), "{work}");
        assert_eq!(
            plan("^a*?", false, false, true, false)
                .find_with_work(&JsString::from("X\r\n"), 2, true, |_| Ok::<(), ()>(()))
                .unwrap(),
            Some(2..2)
        );
        assert_eq!(
            plan("^.+", true, false, false, false)
                .find_with_work(&JsString::from("😀"), 1, true, |_| Ok::<(), ()>(()))
                .unwrap(),
            Some(0..2)
        );
        assert!(
            plan("^.+", true, false, true, true)
                .find_with_work(&input, 0, false, |_| Err(()))
                .is_err()
        );
        let source = JsString::from("^a{2,3}");
        let mut calls = 0;
        RegExpUnicodeRepeatedAtomMatcher::compile_with_start_anchor_and_work(
            &source,
            true,
            true,
            true,
            false,
            |_| {
                calls += 1;
                Ok::<(), ()>(())
            },
        )
        .unwrap()
        .unwrap();
        for fail in 1..=calls {
            let mut n = 0;
            assert!(
                RegExpUnicodeRepeatedAtomMatcher::compile_with_start_anchor_and_work(
                    &source,
                    true,
                    true,
                    true,
                    false,
                    |_| {
                        n += 1;
                        if n == fail { Err(()) } else { Ok(()) }
                    }
                )
                .is_err()
            );
        }
        {
            let source = r"^\p{Script_Extensions=Latin}+";
            let plan = RegExpUnicodeRepeatedAtomMatcher::compile_with_start_anchor_and_work(
                &JsString::from(source),
                true,
                true,
                true,
                false,
                |_| Ok::<(), ()>(()),
            )
            .unwrap()
            .unwrap();
            assert_eq!(
                plan.find_with_work(&JsString::from("AA"), 0, true, |_| Ok::<(), ()>(()))
                    .unwrap(),
                Some(0..2)
            );
            assert_eq!(
                plan.find_with_work(&JsString::from("א"), 0, true, |_| Ok::<(), ()>(()))
                    .unwrap(),
                None
            );
        }
        for source in ["^^a+", "^a+$", r"^\ba+", "^(a)+", "^a+b", r"^[a--b]+"] {
            assert!(
                RegExpUnicodeRepeatedAtomMatcher::compile_with_start_anchor_and_work(
                    &JsString::from(source),
                    true,
                    true,
                    true,
                    false,
                    |_| Ok::<(), ()>(())
                )
                .unwrap()
                .is_none(),
                "{source}"
            );
        }
    }
}

#[derive(Clone, Debug)]
enum Atom {
    Character(RegExpUnicodeCharacterMatcher),
    Class(RegExpUnicodeClassMatcher),
    Alternatives {
        branches: Vec<AlternativeAtom>,
        work: usize,
        guard: Option<RegExpUnicodeCharacterMatcher>,
    },
}

// A separate leaf type keeps construction, matching and destruction flat even
// when the source has many alternatives. Branches contain no nested captures.
#[derive(Clone, Debug)]
struct AlternativeAtom {
    leaf: AlternativeLeaf,
    guard: Option<RegExpUnicodeCharacterMatcher>,
    impossible: bool,
}

#[derive(Clone, Debug)]
enum AlternativeLeaf {
    Character(RegExpUnicodeCharacterMatcher),
    Class(RegExpUnicodeClassMatcher),
}

impl AlternativeAtom {
    fn character(m: RegExpUnicodeCharacterMatcher) -> Self {
        Self {
            leaf: AlternativeLeaf::Character(m),
            guard: None,
            impossible: false,
        }
    }
    fn class(m: RegExpUnicodeClassMatcher) -> Self {
        Self {
            leaf: AlternativeLeaf::Class(m),
            guard: None,
            impossible: false,
        }
    }
    fn find(&self, input: &JsString, at: usize) -> Option<Range<usize>> {
        if self.impossible {
            return None;
        }
        if let Some(guard) = &self.guard {
            guard.find(input, at, true)?;
        }
        match &self.leaf {
            AlternativeLeaf::Character(m) => m.find(input, at, true),
            AlternativeLeaf::Class(m) => m.find(input, at, true),
        }
    }
    fn work(&self) -> usize {
        let leaf = match &self.leaf {
            AlternativeLeaf::Character(m) => m.search_passes(),
            AlternativeLeaf::Class(m) => m.search_passes(),
        };
        leaf.saturating_add(self.guard.as_ref().map_or(0, |m| m.search_passes()))
    }
}

#[cfg(test)]
mod end_assertion_tests {
    use super::*;
    use std::fmt::Write;
    const CASES: [(&str, u8, usize, Option<usize>); 14] = [
        ("a+$", 0, 1, None),
        ("a*?$", 0, 0, None),
        ("a??$", 0, 0, Some(1)),
        ("a{2,3}$", 0, 2, Some(3)),
        ("a{002,03}?$", 0, 2, Some(3)),
        (".+$", 1, 1, None),
        (r"\D{1,2}$", 2, 1, Some(2)),
        (r"\W+$", 3, 1, None),
        (r"\p{Join_C}+$", 4, 1, None),
        (r"\p{NChar}{1,2}$", 5, 1, Some(2)),
        (r"\u{61}?$", 0, 0, Some(1)),
        (r"\??$", 6, 0, Some(1)),
        ("a{0,0}$", 0, 0, Some(0)),
        ("[aſK]+$", 7, 1, None),
    ];
    fn plan(source: &str, v: bool, i: bool, s: bool) -> RegExpUnicodeRepeatedAtomMatcher {
        RegExpUnicodeRepeatedAtomMatcher::compile_with_line_anchors_and_work(
            &JsString::from(source),
            v,
            i,
            false,
            s,
            |_| Ok::<(), ()>(()),
        )
        .unwrap()
        .unwrap()
    }
    #[test]
    fn unicode_repeated_atom_end_ranges_snapshot() {
        let mut rows = String::new();
        for (source, ..) in CASES {
            for begin in [false, true] {
                let source = format!("{}{source}", if begin { "^" } else { "" });
                let pattern = JsString::from(source.as_str());
                for v in [false, true] {
                    for i in [false, true] {
                        for s in [false, true] {
                            let matcher = plan(&source, v, i, s);
                            for input in [
                                JsString::from("Aaaa"),
                                JsString::from("A\r\n\u{200c}\u{200d}\u{ffff}\u{10ffff}"),
                                JsString::from("😀??aaaſK"),
                                JsString::from_code_units(vec![0x61, 0xd800, 0x61, 0xdc00]),
                                JsString::from(""),
                            ] {
                                for start in [0, 1, 2, input.len(), input.len() + 1] {
                                    for sticky in [false, true] {
                                        writeln!(rows,"{pattern:?} v={v} i={i} s={s} input={input:?} start={start} sticky={sticky} {:?}",matcher.find_with_work(&input,start,sticky,|_|Ok::<(),()>(())).unwrap()).unwrap();
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        insta::assert_snapshot!(rows);
    }
    #[test]
    fn unicode_repeated_atom_end_agrees_with_independent_start_and_count_oracle() {
        let alphabet: [u16; 10] = [
            0x41, 0x61, 0x0a, 0x17f, 0x212a, 0x200c, 0xffff, 0x3f, 0xd83d, 0xde00,
        ];
        for i in [false, true] {
            for s in [false, true] {
                let fold = |c| {
                    if i {
                        match c {
                            0x41 => 0x61,
                            0x17f => 0x73,
                            0x212a => 0x6b,
                            _ => c,
                        }
                    } else {
                        c
                    }
                };
                let member = |kind, c| match kind {
                    0 => fold(c) == 0x61,
                    1 => s || ![10, 13, 0x2028, 0x2029].contains(&c),
                    2 => !(0x30..=0x39).contains(&c),
                    3 => !char::from_u32(fold(c))
                        .is_some_and(|c| c.is_ascii_alphanumeric() || c == '_'),
                    4 => (0x200c..=0x200d).contains(&c),
                    5 => matches!(c, 0xfdd0..=0xfdef) || (c & 0xffff) >= 0xfffe,
                    6 => c == 0x3f,
                    7 => [0x61, fold(0x17f), fold(0x212a)].contains(&fold(c)),
                    _ => unreachable!(),
                };
                let plans: Vec<_> = CASES
                    .iter()
                    .flat_map(|c| {
                        [
                            plan(c.0, false, i, s),
                            plan(&format!("^{}", c.0), false, i, s),
                        ]
                    })
                    .collect();
                for len in 0..=4 {
                    for ordinal in 0..alphabet.len().pow(len) {
                        let units: Vec<_> = (0..len)
                            .map(|j| alphabet[ordinal / alphabet.len().pow(j) % alphabet.len()])
                            .collect();
                        let mut points = Vec::new();
                        let mut bounds = vec![0];
                        let mut j = 0;
                        while j < units.len() {
                            let mut c = u32::from(units[j]);
                            j += 1;
                            if (0xd800..=0xdbff).contains(&c)
                                && j < units.len()
                                && (0xdc00..=0xdfff).contains(&units[j])
                            {
                                c = 65536 + (c - 0xd800) * 1024 + u32::from(units[j]) - 0xdc00;
                                j += 1;
                            }
                            points.push(c);
                            bounds.push(j);
                        }
                        let input = JsString::from_code_units(units);
                        for start in 0..=input.len() + 1 {
                            for sticky in [false, true] {
                                for (ordinal, matcher) in plans.iter().enumerate() {
                                    let case = &CASES[ordinal / 2];
                                    let begin = ordinal % 2 != 0;
                                    let expected = if start > input.len() {
                                        None
                                    } else {
                                        let first = bounds.partition_point(|&b| b <= start) - 1;
                                        (first..=points.len()).find_map(|at| {
                                            if (sticky && at != first) || (begin && at != 0) {
                                                return None;
                                            }
                                            let count = points.len() - at;
                                            if count < case.2
                                                || case.3.is_some_and(|upper| count > upper)
                                                || !points[at..].iter().all(|&c| member(case.1, c))
                                            {
                                                return None;
                                            }
                                            Some(bounds[at]..input.len())
                                        })
                                    };
                                    assert_eq!(
                                        matcher
                                            .find_with_work(
                                                &input,
                                                start,
                                                sticky,
                                                |_| Ok::<(), ()>(())
                                            )
                                            .unwrap(),
                                        expected,
                                        "{} begin={begin} i={i} s={s} {input:?} start={start} sticky={sticky}",
                                        case.0
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    #[test]
    fn unicode_repeated_atom_end_work_bounds_clone_and_complete_contracts() {
        let input = JsString::from("a".repeat(200000).as_str());
        let mut work = 0;
        assert_eq!(
            plan("a{50000,100000}?$", true, false, false)
                .clone()
                .find_with_work(&input, 0, false, |n| {
                    work += n;
                    Ok::<(), ()>(())
                })
                .unwrap(),
            Some(100000..200000)
        );
        assert!(work < 4 * input.len(), "{work}");
        assert_eq!(
            plan("^a{50000,100000}$", true, false, false)
                .find_with_work(&input, 0, false, |_| Ok::<(), ()>(()))
                .unwrap(),
            None
        );
        assert_eq!(
            plan("a{0}$", false, false, false)
                .find_with_work(&input, 1, false, |_| Ok::<(), ()>(()))
                .unwrap(),
            Some(input.len()..input.len())
        );
        assert_eq!(
            plan("a{0}$", false, false, false)
                .find_with_work(&input, 1, true, |_| Ok::<(), ()>(()))
                .unwrap(),
            None
        );
        assert!(
            plan(".+$", true, false, true)
                .find_with_work(&input, 0, false, |_| Err(()))
                .is_err()
        );
        let source = JsString::from("^a{2,3}$");
        let mut calls = 0;
        RegExpUnicodeRepeatedAtomMatcher::compile_with_line_anchors_and_work(
            &source,
            true,
            true,
            false,
            false,
            |_| {
                calls += 1;
                Ok::<(), ()>(())
            },
        )
        .unwrap()
        .unwrap();
        for fail in 1..=calls {
            let mut n = 0;
            assert!(
                RegExpUnicodeRepeatedAtomMatcher::compile_with_line_anchors_and_work(
                    &source,
                    true,
                    true,
                    false,
                    false,
                    |_| {
                        n += 1;
                        if n == fail { Err(()) } else { Ok(()) }
                    }
                )
                .is_err()
            );
        }
        for (source, m) in [
            ("a+b$", true),
            ("^a+b$", true),
            ("a+$$", false),
            ("(a)+$", false),
            ("a+b$", false),
            (r"\ba+$", false),
            (r"[a--b]+$", false),
        ] {
            assert!(
                RegExpUnicodeRepeatedAtomMatcher::compile_with_line_anchors_and_work(
                    &JsString::from(source),
                    true,
                    true,
                    m,
                    false,
                    |_| Ok::<(), ()>(())
                )
                .unwrap()
                .is_none(),
                "{source}"
            );
        }
    }
}

impl Atom {
    fn find(&self, input: &JsString, at: usize) -> Option<Range<usize>> {
        match self {
            Self::Character(m) => m.find(input, at, true),
            Self::Class(m) => m.find(input, at, true),
            Self::Alternatives {
                branches, guard, ..
            } => {
                if guard
                    .as_ref()
                    .is_some_and(|m| m.find(input, at, true).is_none())
                {
                    return None;
                }
                branches.iter().find_map(|m| m.find(input, at))
            }
        }
    }
    fn work(&self) -> usize {
        match self {
            Self::Character(m) => m.search_passes(),
            Self::Class(m) => m.search_passes(),
            Self::Alternatives { work, .. } => *work,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum AtomAssertions {
    None,
    Line,
    Word,
    Alternative,
    WrappedAlternative,
    LineWrappedAlternative,
    AssertedWrappedAlternative,
    SequenceWrappedAlternative,
    NestedSequenceWrappedAlternative,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum AlternativeProof {
    Bare,
    Wrapped,
    LineWrapped,
    AssertedWrapped,
    SequenceWrapped,
    NestedSequenceWrapped,
}

impl RegExpUnicodeRepeatedAtomMatcher {
    /// Compiles one checked nested v-mode set expression with a quantifier.
    ///
    /// Requires complete v-mode Pattern validation. Optional outside ^/$ retain
    /// their line contexts (22.2.2.4); the quantifier keeps its exact greedy or
    /// lazy bounds (22.2.2.3). Inner groups, word assertions and concatenations
    /// remain separate proofs. The retained class is borrowed without copying.
    pub fn compile_v_set_with_line_anchors_and_work<E>(
        source: &JsString,
        ignore_case: bool,
        multiline: bool,
        charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, crate::RegExpUnicodeSetCompileError<E>> {
        Self::compile_v_set_units_with_line_anchors_and_work(
            source.code_units(),
            ignore_case,
            multiline,
            charge,
        )
    }

    /// Compiles outside word assertions on one checked v-mode class repetition.
    ///
    /// Requires complete validated v/vi Pattern input. The admitted order is
    /// optional ^, optional leading \b/\B, one nested/operator character class
    /// with one greedy or lazy quantifier, optional trailing \b/\B, optional $.
    /// At least one word assertion is required. Inner groups, reordered gates,
    /// strings and consuming continuations remain separate caller proofs.
    ///
    /// Class units are borrowed; even zero or impossible bounds require complete
    /// class compilation. Word predicates apply to the whole repetition in its
    /// original input context. Work and capacity errors retain the checked class
    /// contract and final shared-storage allocation boundary. Legacy constructors
    /// retain their existing domains and callback sequences.
    pub fn compile_v_set_with_word_boundaries_and_work<E>(
        source: &JsString,
        ignore_case: bool,
        multiline: bool,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, crate::RegExpUnicodeSetCompileError<E>> {
        let units = source.code_units();
        let start_anchor = units.first() == Some(&0x5e);
        let end_anchor = units.last() == Some(&0x24);
        let mut begin = usize::from(start_anchor);
        let Some(mut end) = units.len().checked_sub(usize::from(end_anchor)) else {
            return Ok(None);
        };
        let start_word = match units.get(begin..end).and_then(|body| body.get(..2)) {
            Some([0x5c, kind @ (0x62 | 0x42)]) => {
                begin += 2;
                Some(*kind == 0x62)
            }
            _ => None,
        };
        let end_word = if end >= begin + 2 {
            match &units[end - 2..end] {
                [0x5c, kind @ (0x62 | 0x42)] => {
                    end -= 2;
                    Some(*kind == 0x62)
                }
                _ => None,
            }
        } else {
            None
        };
        if start_word.is_none() && end_word.is_none() {
            return Ok(None);
        }
        let Some(body) = units.get(begin..end) else {
            return Ok(None);
        };
        // Retained line gates cannot be overwritten with the outside flags.
        // Complete class/quantifier compilation also rejects escaped edge tokens.
        if body.first() != Some(&0x5b) || body.last() == Some(&0x24) {
            return Ok(None);
        }
        charge(units.len()).map_err(crate::RegExpUnicodeSetCompileError::Work)?;
        let Some(mut plan) = Self::compile_v_set_units_with_line_anchors_and_work(
            body,
            ignore_case,
            multiline,
            &mut charge,
        )?
        else {
            return Ok(None);
        };
        if plan.start_anchor || plan.end_anchor {
            return Ok(None);
        }
        plan.start_anchor = start_anchor;
        plan.end_anchor = end_anchor;
        plan.multiline = multiline;
        plan.start_word = start_word;
        plan.end_word = end_word;
        plan.word_ignore_case = ignore_case;
        Ok(Some(plan))
    }

    fn compile_v_set_units_with_line_anchors_and_work<E>(
        units: &[u16],
        ignore_case: bool,
        multiline: bool,
        charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, crate::RegExpUnicodeSetCompileError<E>> {
        Self::compile_v_set_repetition_units_with_work(
            units,
            ignore_case,
            multiline,
            ClassProof::Checked,
            charge,
        )
    }

    fn compile_v_set_repetition_units_with_work<E>(
        units: &[u16],
        ignore_case: bool,
        multiline: bool,
        proof: ClassProof,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, crate::RegExpUnicodeSetCompileError<E>> {
        // Bound both quantifier attempts, decimal comparisons and anchor scans
        // before inspecting source. No quantifier count expands into storage.
        charge(units.len().saturating_mul(8)).map_err(crate::RegExpUnicodeSetCompileError::Work)?;
        let start_anchor = units.first() == Some(&0x5e);
        let end_anchor = units.last() == Some(&0x24) && !escaped(units, units.len() - 1);
        let units = &units[usize::from(start_anchor)..units.len() - usize::from(end_anchor)];
        // As in the legacy repetition proof, try a final lazy marker first.
        // Only a complete retained class can select either interpretation.
        for (end, greedy) in [
            (
                units
                    .len()
                    .checked_sub(1)
                    .filter(|_| units.last() == Some(&0x3f)),
                false,
            ),
            (Some(units.len()), true),
        ] {
            let Some(end) = end else { continue };
            let Some((body_end, minimum, maximum)) = quantifier(units, end) else {
                continue;
            };
            let body = &units[..body_end];
            if body.first() != Some(&0x5b) || body.last() != Some(&0x5d) {
                continue;
            }
            let atom = match proof {
                ClassProof::SingleCharacterStrings => RegExpUnicodeClassMatcher::compile_v_set_units_with_single_character_strings_and_work(
                    body,
                    ignore_case,
                    false,
                    &mut charge,
                ),
                ClassProof::Checked => RegExpUnicodeClassMatcher::compile_v_set_units_with_work(
                    body,
                    ignore_case,
                    false,
                    &mut charge,
                ),
                ClassProof::ScalarLeftStrings => RegExpUnicodeClassMatcher::compile_scalar_left_string_operations_units_with_work(
                    body, ignore_case, false, &mut charge,
                ),
                ClassProof::CharacterResultStrings => RegExpUnicodeClassMatcher::compile_character_result_v_set_units_with_work(
                    body, ignore_case, false, &mut charge,
                ),
            };
            let Some(atom) = atom? else {
                continue;
            };
            return Ok(Some(Self {
                atom: Atom::Class(atom),
                minimum,
                maximum,
                greedy,
                start_anchor,
                multiline,
                end_anchor,
                atom_captures: 0,
                start_word: None,
                end_word: None,
                word_ignore_case: false,
                nullable: None,
            }));
        }
        Ok(None)
    }

    /// Compiles one complete character-only q-class with one quantifier.
    ///
    /// The complete Pattern must already be validated in v/vi mode. Its class
    /// must contain a ClassStringDisjunction, and every alternative of every q
    /// operand must decode to one character. Supported nested character-only
    /// unions, complements, intersections and subtractions retain fold closure.
    ///
    /// The admitted order is optional ^, optional leading \b/\B, one complete
    /// class with an existing greedy/lazy quantifier, optional trailing \b/\B
    /// and optional $. Predicates apply to the whole repetition in original
    /// input context. Complete captures remain a separate caller proof.
    /// Inner groups, consuming continuations and string-valued leaves decline,
    /// even under zero or impossible bounds. Exact counts are not expanded.
    ///
    /// Existing class storage, opt-in work/capacity errors and flat incremental
    /// repetition matching are reused (22.2.2.3, 22.2.2.4, 22.2.2.9). Legacy
    /// entry points retain their accepted domains and callback sequences.
    pub fn compile_v_set_with_single_character_strings_and_work<E>(
        source: &JsString,
        ignore_case: bool,
        multiline: bool,
        charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, crate::RegExpUnicodeSetCompileError<E>> {
        Self::compile_string_class_repetition_with_work(
            source,
            ignore_case,
            multiline,
            ClassProof::SingleCharacterStrings,
            charge,
        )
    }

    /// Compiles a quantified scalar-left string operation as a character atom.
    ///
    /// Requires complete v/vi validation. The positive class starts with a q-free
    /// scalar operand followed by intersection or subtraction and contains an
    /// empty or longer q member. Character projection proves every iteration
    /// consumes one complete input character, even with empty q alternatives.
    /// Existing greedy/lazy and exact decimal bounds apply without expansion.
    /// Canonical optional outside line/word gates retain whole-repetition scope.
    /// Inner groups, root string unions, further atoms, and string properties
    /// remain separate proofs, including under zero or impossible bounds.
    /// https://262.ecma-international.org/17.0/#sec-repeatmatcher
    pub fn compile_scalar_left_string_operations_with_work<E>(
        source: &JsString,
        ignore_case: bool,
        multiline: bool,
        charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, crate::RegExpUnicodeSetCompileError<E>> {
        Self::compile_string_class_repetition_with_work(
            source,
            ignore_case,
            multiline,
            ClassProof::ScalarLeftStrings,
            charge,
        )
    }

    /// Compiles one quantified v/vi class with a static character-only result.
    ///
    /// Requires complete validation, MayContainStrings false, and at least one
    /// empty or longer q member. The projected class is a consuming character
    /// atom, including when its result is empty. Greedy/lazy exact bounds and
    /// canonical outside word/line gates reuse the existing repetition proof.
    /// Complete inner wrappers, string properties, further atoms and conservative
    /// string results remain separate proofs, even under zero bounds.
    /// https://262.ecma-international.org/17.0/#sec-repeatmatcher
    pub fn compile_character_result_v_set_with_work<E>(
        source: &JsString,
        ignore_case: bool,
        multiline: bool,
        charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, crate::RegExpUnicodeSetCompileError<E>> {
        Self::compile_string_class_repetition_with_work(
            source,
            ignore_case,
            multiline,
            ClassProof::CharacterResultStrings,
            charge,
        )
    }

    fn compile_string_class_repetition_with_work<E>(
        source: &JsString,
        ignore_case: bool,
        multiline: bool,
        proof: ClassProof,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, crate::RegExpUnicodeSetCompileError<E>> {
        let units = source.code_units();
        charge(units.len()).map_err(crate::RegExpUnicodeSetCompileError::Work)?;
        let start_anchor = units.first() == Some(&0x5e);
        let end_anchor = units.last() == Some(&0x24) && !escaped(units, units.len() - 1);
        let mut begin = usize::from(start_anchor);
        let mut end = units.len() - usize::from(end_anchor);
        let start_word = match units.get(begin..end).and_then(|body| body.get(..2)) {
            Some([0x5c, kind @ (0x62 | 0x42)]) => {
                begin += 2;
                Some(*kind == 0x62)
            }
            _ => None,
        };
        let end_word = if end >= begin + 2 {
            match &units[end - 2..end] {
                [0x5c, kind @ (0x62 | 0x42)] => {
                    end -= 2;
                    Some(*kind == 0x62)
                }
                _ => None,
            }
        } else {
            None
        };
        let Some(body) = units.get(begin..end) else {
            return Ok(None);
        };
        // The retained class/quantifier proof rejects escaped edge tokens.
        // Do not overwrite a second, retained line assertion with outside flags.
        if body.first() != Some(&0x5b) || body.last() == Some(&0x24) {
            return Ok(None);
        }
        let Some(mut plan) = Self::compile_v_set_repetition_units_with_work(
            body,
            ignore_case,
            multiline,
            proof,
            &mut charge,
        )?
        else {
            return Ok(None);
        };
        if plan.start_anchor || plan.end_anchor {
            return Ok(None);
        }
        plan.start_anchor = start_anchor;
        plan.end_anchor = end_anchor;
        plan.start_word = start_word;
        plan.end_word = end_word;
        plan.word_ignore_case = ignore_case;
        Ok(Some(plan))
    }

    /// Proves complete capturing/noncapturing wrappers around one repeated q-class.
    ///
    /// The complete Pattern must already be validated in v/vi mode and named
    /// captures normalized. One or more complete ordinary wrappers contain one
    /// bare checked class with at least one singleton-character q operand, then
    /// one greedy/lazy quantifier. Optional outside ^, leading \b/\B, trailing
    /// \b/\B and $ apply to the whole repetition in that order.
    ///
    /// Every inner capture selects the final original character, or remains
    /// undefined after zero iterations. Complete outer captures remain a caller
    /// proof. Inner assertions, alternatives, extra atoms, partial captures and
    /// string-valued leaves decline, including under zero or impossible bounds.
    /// Source slices are borrowed; counts and wrappers never expand into storage.
    /// Earlier constructors retain their domains and callback sequences.
    /// https://262.ecma-international.org/17.0/#sec-repeatmatcher
    pub fn compile_wrapped_v_set_with_single_character_strings_and_work<E>(
        source: &JsString,
        ignore_case: bool,
        multiline: bool,
        charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, crate::RegExpUnicodeSetCompileError<E>> {
        Self::compile_wrapped_string_class_repetition_with_work(
            source,
            ignore_case,
            multiline,
            ClassProof::SingleCharacterStrings,
            charge,
        )
    }

    /// Compiles complete wrappers around one repeated character-result v class.
    ///
    /// Requires validated v/vi input and normalized capture names. At least one
    /// ordinary/noncapturing wrapper surrounds a bare class with a false static
    /// MayContainStrings result and an empty or longer q member, followed by one
    /// existing greedy/lazy quantifier. Each inner capture selects the final
    /// complete original character, or is undefined after zero iterations.
    /// Canonical outside line/word gates apply to the entire repetition; outer
    /// captures remain a caller proof. Partial groups, internal assertions, extra
    /// atoms, string properties and conservative string results remain separate
    /// proofs, including under zero or impossible bounds. Source stays borrowed
    /// and counts/wrappers never expand into recursive matching storage.
    /// https://262.ecma-international.org/17.0/#sec-repeatmatcher
    pub fn compile_wrapped_character_result_v_set_with_work<E>(
        source: &JsString,
        ignore_case: bool,
        multiline: bool,
        charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, crate::RegExpUnicodeSetCompileError<E>> {
        Self::compile_wrapped_string_class_repetition_with_work(
            source,
            ignore_case,
            multiline,
            ClassProof::CharacterResultStrings,
            charge,
        )
    }

    fn compile_wrapped_string_class_repetition_with_work<E>(
        source: &JsString,
        ignore_case: bool,
        multiline: bool,
        proof: ClassProof,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, crate::RegExpUnicodeSetCompileError<E>> {
        let units = source.code_units();
        // Bound outside framing, both lazy/greedy quantifier interpretations,
        // decimal comparisons and both borrowed wrapper walks before any scan.
        charge(units.len().saturating_mul(16))
            .map_err(crate::RegExpUnicodeSetCompileError::Work)?;
        let start_anchor = units.first() == Some(&0x5e);
        let end_anchor = units.last() == Some(&0x24) && !escaped(units, units.len() - 1);
        let mut begin = usize::from(start_anchor);
        let mut end = units.len() - usize::from(end_anchor);
        let start_word = match units.get(begin..end).and_then(|body| body.get(..2)) {
            Some([0x5c, kind @ (0x62 | 0x42)]) => {
                begin += 2;
                Some(*kind == 0x62)
            }
            _ => None,
        };
        let end_word = if end >= begin + 2
            && units[end - 2] == 0x5c
            && matches!(units[end - 1], 0x62 | 0x42)
            && !escaped(units, end - 2)
        {
            end -= 2;
            Some(units[end + 1] == 0x62)
        } else {
            None
        };
        let Some(body) = units.get(begin..end) else {
            return Ok(None);
        };
        for (end, greedy) in [
            (
                body.len()
                    .checked_sub(1)
                    .filter(|_| body.last() == Some(&0x3f)),
                false,
            ),
            (Some(body.len()), true),
        ] {
            let Some(end) = end else { continue };
            let Some((atom_end, minimum, maximum)) = quantifier(body, end) else {
                continue;
            };
            let (mut at, mut wrappers, mut captures) = (0, 0, 0);
            while body.get(at) == Some(&0x28) {
                if body.get(at + 1) == Some(&0x3f) {
                    if body.get(at + 2) != Some(&0x3a) {
                        break;
                    }
                    at += 3;
                } else {
                    at += 1;
                    captures += 1;
                }
                wrappers += 1;
            }
            let Some(leaf_end) = atom_end.checked_sub(wrappers) else {
                continue;
            };
            let Some(leaf) = body.get(at..leaf_end) else {
                continue;
            };
            if wrappers == 0
                || leaf.first() != Some(&0x5b)
                || leaf.last() != Some(&0x5d)
                || !body[leaf_end..atom_end].iter().all(|&unit| unit == 0x29)
            {
                continue;
            }
            // A bare complete leaf prevents the class helper's optional ^/$
            // from accepting any assertion inside the repeated wrappers.
            let atom = match proof {
                ClassProof::Checked => RegExpUnicodeClassMatcher::compile_v_set_units_with_work(leaf, ignore_case, false, &mut charge),
                ClassProof::SingleCharacterStrings => RegExpUnicodeClassMatcher::compile_v_set_units_with_single_character_strings_and_work(leaf, ignore_case, false, &mut charge),
                ClassProof::ScalarLeftStrings => RegExpUnicodeClassMatcher::compile_scalar_left_string_operations_units_with_work(leaf, ignore_case, false, &mut charge),
                ClassProof::CharacterResultStrings => RegExpUnicodeClassMatcher::compile_character_result_v_set_units_with_work(leaf, ignore_case, false, &mut charge),
            };
            let Some(atom) = atom? else { continue };
            return Ok(Some(Self {
                atom: Atom::Class(atom),
                minimum,
                maximum,
                greedy,
                start_anchor,
                multiline,
                end_anchor,
                atom_captures: captures,
                start_word,
                end_word,
                word_ignore_case: ignore_case,
                nullable: None,
            }));
        }
        Ok(None)
    }

    /// Proves one raw empty branch and one complete singleton-q class branch.
    ///
    /// The complete Pattern must already be validated in v/vi mode and named
    /// captures normalized. Complete nested ordinary/noncapturing wrappers
    /// surround either the bare class then |, or | then the bare class. Only
    /// *, + and ? are admitted, including their lazy forms. Canonical optional
    /// outside ^, leading \b/\B, trailing \b/\B and $ keep whole-match scope.
    ///
    /// RepeatMatcher rejects optional empty iterations; a required empty plus
    /// iteration instead retains defined empty inner captures. All q operands
    /// must prove one-character alternatives before either path can succeed.
    /// Branch-local captures/assertions, grouped empty branches, extra branches,
    /// braced counts and string-valued leaves decline. Existing constructors,
    /// nullable search, captures, work callbacks and storage remain unchanged.
    /// https://262.ecma-international.org/17.0/#sec-repeatmatcher
    pub fn compile_nullable_v_set_with_single_character_strings_and_work<E>(
        source: &JsString,
        ignore_case: bool,
        multiline: bool,
        charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, crate::RegExpUnicodeSetCompileError<E>> {
        Self::compile_nullable_string_class_repetition_with_work(
            source,
            ignore_case,
            multiline,
            ClassProof::SingleCharacterStrings,
            NullableProof::Raw,
            EmptyBranchProof::Raw,
            charge,
        )
    }

    /// Compiles one raw empty alternative beside one character-result v class.
    ///
    /// Requires validated v/vi input and normalized capture names. Complete
    /// ordinary/noncapturing wrappers contain either the bare class then |, or
    /// | then the class, followed by *, + or ?, including lazy forms. The class
    /// must have a false static MayContainStrings result and an empty/long q
    /// member. Canonical outside word/line gates apply to the whole repetition.
    /// Required empty iterations retain defined empty captures; zero iterations
    /// leave them undefined. Branch-local captures/assertions, extra branches,
    /// grouped empty alternatives, braced counts, string properties, conservative
    /// string results and further atoms remain separate proofs. All leaves are
    /// compiled before matching, including behind the empty-first alternative.
    /// https://262.ecma-international.org/17.0/#sec-repeatmatcher
    pub fn compile_nullable_character_result_v_set_with_work<E>(
        source: &JsString,
        ignore_case: bool,
        multiline: bool,
        charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, crate::RegExpUnicodeSetCompileError<E>> {
        Self::compile_nullable_string_class_repetition_with_work(
            source,
            ignore_case,
            multiline,
            ClassProof::CharacterResultStrings,
            NullableProof::Raw,
            EmptyBranchProof::Raw,
            charge,
        )
    }

    /// Compiles zero-minimum braced repetition of a singleton-q class or empty branch.
    ///
    /// Requires validated v/vi input and normalized capture names. Complete
    /// wrappers contain either the bare class then |, or | then the class.
    /// Only braced counts with a zero lower bound are admitted, including exact
    /// zero, padded/overflowing upper bounds and lazy forms. Every q member must
    /// be a single character. All leaves are compiled even under exact zero.
    /// Optional empty iterations are rejected before exporting captures.
    /// https://262.ecma-international.org/17.0/#sec-repeatmatcher
    pub fn compile_zero_minimum_nullable_v_set_with_single_character_strings_and_work<E>(
        source: &JsString,
        ignore_case: bool,
        multiline: bool,
        charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, crate::RegExpUnicodeSetCompileError<E>> {
        Self::compile_nullable_string_class_repetition_with_work(
            source,
            ignore_case,
            multiline,
            ClassProof::SingleCharacterStrings,
            NullableProof::ZeroMinimum,
            EmptyBranchProof::Raw,
            charge,
        )
    }

    /// Compiles zero-minimum braced repetition of a character-result class or empty branch.
    ///
    /// Requires validated v/vi input and normalized capture names. Complete
    /// wrappers surround one raw empty alternative beside a bare class whose
    /// static MayContainStrings result is false and which contains an empty/long
    /// q member. Canonical outside word/line gates retain whole-repetition scope.
    /// Exact zero, padded/overflowing upper bounds and lazy forms preserve
    /// progress checks, undefined zero-iteration captures and final original
    /// character ranges. Every class operand is proved even under exact zero.
    /// Positive lower bounds, simple quantifiers, branch-local captures,
    /// grouped/extra empty branches and further atoms remain separate proofs.
    /// https://262.ecma-international.org/17.0/#sec-repeatmatcher
    pub fn compile_zero_minimum_nullable_character_result_v_set_with_work<E>(
        source: &JsString,
        ignore_case: bool,
        multiline: bool,
        charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, crate::RegExpUnicodeSetCompileError<E>> {
        Self::compile_nullable_string_class_repetition_with_work(
            source,
            ignore_case,
            multiline,
            ClassProof::CharacterResultStrings,
            NullableProof::ZeroMinimum,
            EmptyBranchProof::Raw,
            charge,
        )
    }

    /// Compiles {1,} repetition of a singleton-q class or raw empty branch.
    ///
    /// Requires validated v/vi input and normalized capture names. Complete
    /// wrappers contain either the bare class then |, or | then the class.
    /// Padded lower counts and lazy forms retain source branch priority and
    /// canonical outside word/line gates. Every q member must be one character.
    /// Required empty iterations define empty captures; zero optional iterations
    /// cannot replace those captures. All class leaves are proved before search.
    /// https://262.ecma-international.org/17.0/#sec-repeatmatcher
    pub fn compile_one_minimum_open_nullable_v_set_with_single_character_strings_and_work<E>(
        source: &JsString,
        ignore_case: bool,
        multiline: bool,
        charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, crate::RegExpUnicodeSetCompileError<E>> {
        Self::compile_nullable_string_class_repetition_with_work(
            source,
            ignore_case,
            multiline,
            ClassProof::SingleCharacterStrings,
            NullableProof::OneMinimumOpen,
            EmptyBranchProof::Raw,
            charge,
        )
    }

    /// Compiles {1,} repetition of a character-result class or raw empty branch.
    ///
    /// Requires validated v/vi input and normalized capture names. Complete
    /// wrappers contain one raw empty alternative and one bare class with a
    /// false static MayContainStrings result and an empty/long q member. Padded
    /// lower counts and lazy forms preserve branch order, required empty
    /// captures, optional-empty progress checks, final original ranges and
    /// canonical outside word/line gates. Every class leaf is compiled even
    /// behind an empty-first branch. Other lower bounds, finite upper bounds,
    /// simple quantifiers, branch-local captures/assertions, grouped/extra empty
    /// alternatives and additional atoms remain separate proofs.
    /// https://262.ecma-international.org/17.0/#sec-repeatmatcher
    pub fn compile_one_minimum_open_nullable_character_result_v_set_with_work<E>(
        source: &JsString,
        ignore_case: bool,
        multiline: bool,
        charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, crate::RegExpUnicodeSetCompileError<E>> {
        Self::compile_nullable_string_class_repetition_with_work(
            source,
            ignore_case,
            multiline,
            ClassProof::CharacterResultStrings,
            NullableProof::OneMinimumOpen,
            EmptyBranchProof::Raw,
            charge,
        )
    }

    /// Compiles exactly one iteration of a singleton-q class or raw empty branch.
    ///
    /// Requires validated v/vi input and normalized capture names. Complete
    /// wrappers contain either the bare class then |, or | then the class.
    /// Braced {1} and {1,1}, including padded/lazy forms, retain source branch
    /// priority: greediness cannot reorder the single mandatory iteration.
    /// Every q member must be one character. Required empty captures are defined.
    /// Canonical outside word/line gates retain whole-repetition scope.
    /// https://262.ecma-international.org/17.0/#sec-repeatmatcher
    pub fn compile_one_iteration_nullable_v_set_with_single_character_strings_and_work<E>(
        source: &JsString,
        ignore_case: bool,
        multiline: bool,
        charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, crate::RegExpUnicodeSetCompileError<E>> {
        Self::compile_nullable_string_class_repetition_with_work(
            source,
            ignore_case,
            multiline,
            ClassProof::SingleCharacterStrings,
            NullableProof::OneIteration,
            EmptyBranchProof::Raw,
            charge,
        )
    }

    /// Compiles exactly one iteration of a character-result class or raw empty branch.
    ///
    /// Requires validated v/vi input and normalized capture names. Complete
    /// wrappers surround one raw empty alternative and a bare class whose
    /// static MayContainStrings result is false and which contains an empty/long
    /// q member. Braced {1} and {1,1}, including padded/lazy forms, preserve
    /// source branch priority, defined empty captures, original character ranges
    /// and canonical outside word/line gates. Every leaf is compiled even behind
    /// an empty-first branch. Other counts, simple quantifiers, branch-local
    /// captures/assertions, grouped/extra empty branches and additional atoms
    /// remain separate proofs.
    /// https://262.ecma-international.org/17.0/#sec-repeatmatcher
    pub fn compile_one_iteration_nullable_character_result_v_set_with_work<E>(
        source: &JsString,
        ignore_case: bool,
        multiline: bool,
        charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, crate::RegExpUnicodeSetCompileError<E>> {
        Self::compile_nullable_string_class_repetition_with_work(
            source,
            ignore_case,
            multiline,
            ClassProof::CharacterResultStrings,
            NullableProof::OneIteration,
            EmptyBranchProof::Raw,
            charge,
        )
    }

    /// Compiles positive open-count repetition of a singleton-q class or empty branch.
    ///
    /// Requires validated v/vi input and normalized capture names. Complete
    /// wrappers contain either the bare class then |, or | then the class.
    /// Braced {m,} requires a positive lower bound, including padded and
    /// unrepresentable counts and lazy forms. Source branch priority, mandatory
    /// empty captures, optional progress checks and canonical outside word/line
    /// gates retain their existing semantics. Every q member must be a single
    /// character, and every class leaf is proved before search.
    /// https://262.ecma-international.org/17.0/#sec-repeatmatcher
    pub fn compile_positive_open_nullable_v_set_with_single_character_strings_and_work<E>(
        source: &JsString,
        ignore_case: bool,
        multiline: bool,
        charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, crate::RegExpUnicodeSetCompileError<E>> {
        Self::compile_nullable_string_class_repetition_with_work(
            source,
            ignore_case,
            multiline,
            ClassProof::SingleCharacterStrings,
            NullableProof::PositiveOpen,
            EmptyBranchProof::Raw,
            charge,
        )
    }

    /// Compiles positive open-count repetition of a character-result class or empty branch.
    ///
    /// Requires validated v/vi input and normalized capture names. Complete
    /// wrappers contain a raw empty alternative and a bare class whose static
    /// MayContainStrings result is false and which contains an empty/long q
    /// member. Positive braced {m,}, including padded/unrepresentable bounds and
    /// lazy forms, reuses mandatory branch priority, optional progress checks,
    /// final original capture ranges and canonical outside word/line gates.
    /// Required empty captures stay defined, even after consuming characters.
    /// Every leaf is compiled behind empty-first choices. Finite upper bounds,
    /// zero lower bounds, simple quantifiers, branch-local captures/assertions,
    /// grouped/extra empty alternatives and further atoms remain separate proofs.
    /// https://262.ecma-international.org/17.0/#sec-repeatmatcher
    pub fn compile_positive_open_nullable_character_result_v_set_with_work<E>(
        source: &JsString,
        ignore_case: bool,
        multiline: bool,
        charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, crate::RegExpUnicodeSetCompileError<E>> {
        Self::compile_nullable_string_class_repetition_with_work(
            source,
            ignore_case,
            multiline,
            ClassProof::CharacterResultStrings,
            NullableProof::PositiveOpen,
            EmptyBranchProof::Raw,
            charge,
        )
    }

    /// Compiles representable exact counts above one of a singleton-q class or empty branch.
    ///
    /// Requires validated v/vi input and normalized capture names. Complete
    /// wrappers contain either the bare class then |, or | then the class.
    /// Braced {m} and {m,m} require a represented bound greater than one,
    /// including padded/lazy forms. Every iteration is mandatory, so source
    /// branch priority determines empty/character captures independently of
    /// greediness. Every q member must be a single character. Canonical outside
    /// word/line gates retain whole-repetition scope and all leaves are compiled.
    /// https://262.ecma-international.org/17.0/#sec-repeatmatcher
    pub fn compile_positive_exact_nullable_v_set_with_single_character_strings_and_work<E>(
        source: &JsString,
        ignore_case: bool,
        multiline: bool,
        charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, crate::RegExpUnicodeSetCompileError<E>> {
        Self::compile_nullable_string_class_repetition_with_work(
            source,
            ignore_case,
            multiline,
            ClassProof::SingleCharacterStrings,
            NullableProof::PositiveExact,
            EmptyBranchProof::Raw,
            charge,
        )
    }

    /// Compiles representable exact counts above one of a character-result class or empty branch.
    ///
    /// Requires validated v/vi input and normalized capture names. Complete
    /// wrappers contain a raw empty alternative and a bare class with a false
    /// static MayContainStrings result and an empty/long q member. Braced {m}
    /// and {m,m}, including padded/lazy forms, require a represented bound above
    /// one. Mandatory source priority retains final empty or original character
    /// captures independently of greediness. Canonical outside word/line gates
    /// retain their scope and every leaf is compiled behind empty-first choices.
    /// Other bounds, simple quantifiers, branch-local captures/assertions,
    /// grouped/extra empty alternatives and additional atoms remain separate.
    /// https://262.ecma-international.org/17.0/#sec-repeatmatcher
    pub fn compile_positive_exact_nullable_character_result_v_set_with_work<E>(
        source: &JsString,
        ignore_case: bool,
        multiline: bool,
        charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, crate::RegExpUnicodeSetCompileError<E>> {
        Self::compile_nullable_string_class_repetition_with_work(
            source,
            ignore_case,
            multiline,
            ClassProof::CharacterResultStrings,
            NullableProof::PositiveExact,
            EmptyBranchProof::Raw,
            charge,
        )
    }

    /// Compiles an unrepresentable exact count of a singleton-q class or empty branch.
    ///
    /// Requires validated v/vi input and normalized capture names. Complete
    /// wrappers contain either the bare class then |, or | then the class.
    /// Braced {m} or {m,m} must exceed native count storage, with numerically
    /// equal range endpoints after trimming leading zeros. Padded/lazy forms
    /// preserve mandatory source priority, defined empty captures and canonical
    /// outside word/line gates. Every q member is one character and every class
    /// operand is proved. Counts and captures never expand into matching state.
    /// https://262.ecma-international.org/17.0/#sec-repeatmatcher
    pub fn compile_unrepresentable_exact_nullable_v_set_with_single_character_strings_and_work<
        E,
    >(
        source: &JsString,
        ignore_case: bool,
        multiline: bool,
        charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, crate::RegExpUnicodeSetCompileError<E>> {
        Self::compile_nullable_string_class_repetition_with_work(
            source,
            ignore_case,
            multiline,
            ClassProof::SingleCharacterStrings,
            NullableProof::UnrepresentableExact,
            EmptyBranchProof::Raw,
            charge,
        )
    }

    /// Compiles an unrepresentable exact count of a character-result class or empty branch.
    ///
    /// Requires validated v/vi input and normalized capture names. Complete
    /// wrappers contain a raw empty alternative and a bare class with a false
    /// static MayContainStrings result and an empty/long q member. Braced {m}
    /// and numerically equal {m,m} require overflowing bounds; padded/lazy forms
    /// retain mandatory source priority independently of quantifier greediness.
    /// Every leaf is compiled behind empty-first choices. Original ranges and
    /// required empty captures reuse existing flat search and capture storage.
    /// Represented/unequal/open bounds, simple quantifiers, partial captures,
    /// grouped/extra empty alternatives and further atoms remain separate proofs.
    /// https://262.ecma-international.org/17.0/#sec-repeatmatcher
    pub fn compile_unrepresentable_exact_nullable_character_result_v_set_with_work<E>(
        source: &JsString,
        ignore_case: bool,
        multiline: bool,
        charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, crate::RegExpUnicodeSetCompileError<E>> {
        Self::compile_nullable_string_class_repetition_with_work(
            source,
            ignore_case,
            multiline,
            ClassProof::CharacterResultStrings,
            NullableProof::UnrepresentableExact,
            EmptyBranchProof::Raw,
            charge,
        )
    }

    /// Compiles represented positive finite ranges of a singleton-q class or empty branch.
    ///
    /// Requires validated v/vi input and normalized capture names. Complete
    /// wrappers contain either the bare class then |, or | then the class.
    /// Braced {m,n} requires represented positive m and greater represented n.
    /// Padded/lazy forms retain mandatory source priority, exact optional budgets,
    /// required empty captures and canonical outside word/line gates. Every q
    /// member is one character and every class operand is proved before search.
    /// https://262.ecma-international.org/17.0/#sec-repeatmatcher
    pub fn compile_positive_finite_nullable_v_set_with_single_character_strings_and_work<E>(
        source: &JsString,
        ignore_case: bool,
        multiline: bool,
        charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, crate::RegExpUnicodeSetCompileError<E>> {
        Self::compile_nullable_string_class_repetition_with_work(
            source,
            ignore_case,
            multiline,
            ClassProof::SingleCharacterStrings,
            NullableProof::PositiveFinite,
            EmptyBranchProof::Raw,
            charge,
        )
    }

    /// Compiles represented positive finite ranges of a character-result class or empty branch.
    ///
    /// Requires validated v/vi input and normalized capture names. Complete
    /// wrappers contain a raw empty alternative and a bare class with a false
    /// static MayContainStrings result and an empty/long q member. Braced {m,n}
    /// requires represented positive m and greater represented n, including
    /// padded/lazy forms. Mandatory source priority, exact optional budgets,
    /// required empty/original character captures and canonical outside gates
    /// reuse existing linear search. Every leaf is compiled behind empty-first
    /// choices. Other counts, partial captures, grouped/extra empty alternatives
    /// and further atoms remain separate proofs.
    /// https://262.ecma-international.org/17.0/#sec-repeatmatcher
    pub fn compile_positive_finite_nullable_character_result_v_set_with_work<E>(
        source: &JsString,
        ignore_case: bool,
        multiline: bool,
        charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, crate::RegExpUnicodeSetCompileError<E>> {
        Self::compile_nullable_string_class_repetition_with_work(
            source,
            ignore_case,
            multiline,
            ClassProof::CharacterResultStrings,
            NullableProof::PositiveFinite,
            EmptyBranchProof::Raw,
            charge,
        )
    }

    /// Compiles a positive nullable singleton-q class with an overflowing finite upper bound.
    ///
    /// Requires validated v/vi input and normalized capture names. Complete
    /// wrappers contain a bare class and a raw empty alternative. Braced {m,n}
    /// requires represented positive m and an unrepresentable finite n. Borrowed
    /// decimal subtraction preserves the exact optional budget even when n-m
    /// fits native storage. Padded/lazy forms, mandatory branch priority, final
    /// empty/original character captures and canonical outside gates reuse the
    /// existing linear matcher. Every class leaf is proved before search.
    /// Other counts, partial captures and further atoms remain separate proofs.
    /// https://262.ecma-international.org/17.0/#sec-repeatmatcher
    pub fn compile_overflowing_maximum_nullable_v_set_with_single_character_strings_and_work<E>(
        source: &JsString,
        ignore_case: bool,
        multiline: bool,
        charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, crate::RegExpUnicodeSetCompileError<E>> {
        Self::compile_nullable_string_class_repetition_with_work(
            source,
            ignore_case,
            multiline,
            ClassProof::SingleCharacterStrings,
            NullableProof::OverflowingMaximum,
            EmptyBranchProof::Raw,
            charge,
        )
    }

    /// Compiles a positive nullable character-result class with an overflowing finite upper bound.
    ///
    /// Requires validated v/vi input and normalized capture names. Complete
    /// wrappers contain a bare class and a raw empty alternative. Braced {m,n}
    /// requires represented positive m and an unrepresentable finite n. Borrowed
    /// decimal subtraction preserves the exact optional budget even when n-m
    /// fits native storage. Padded/lazy forms, mandatory branch priority, final
    /// empty/original character captures and canonical outside gates reuse the
    /// existing linear matcher. Every class leaf is proved before search.
    /// Other counts, partial captures and further atoms remain separate proofs.
    /// https://262.ecma-international.org/17.0/#sec-repeatmatcher
    pub fn compile_overflowing_maximum_nullable_character_result_v_set_with_work<E>(
        source: &JsString,
        ignore_case: bool,
        multiline: bool,
        charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, crate::RegExpUnicodeSetCompileError<E>> {
        Self::compile_nullable_string_class_repetition_with_work(
            source,
            ignore_case,
            multiline,
            ClassProof::CharacterResultStrings,
            NullableProof::OverflowingMaximum,
            EmptyBranchProof::Raw,
            charge,
        )
    }

    /// Compiles finite nullable singleton-q ranges with two overflowing unequal bounds.
    ///
    /// Requires validated v/vi input and normalized capture names. Complete
    /// wrappers contain a bare class and a raw empty alternative under {m,n}.
    /// Both finite bounds exceed native storage and are numerically unequal.
    /// Borrowed decimal subtraction retains exact optional budgets, including
    /// padded endpoints and differences that fit native storage. All consumed
    /// characters can be mandatory: character-first priority is independent of
    /// laziness, while empty-first choices retain optional progress and outside
    /// word/line gates. Final captures and linear search reuse existing state.
    /// Every class operand is proved before matching. Other counts, partial
    /// captures and further atoms remain separate proofs.
    /// https://262.ecma-international.org/17.0/#sec-repeatmatcher
    pub fn compile_unrepresentable_finite_nullable_v_set_with_single_character_strings_and_work<
        E,
    >(
        source: &JsString,
        ignore_case: bool,
        multiline: bool,
        charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, crate::RegExpUnicodeSetCompileError<E>> {
        Self::compile_nullable_string_class_repetition_with_work(
            source,
            ignore_case,
            multiline,
            ClassProof::SingleCharacterStrings,
            NullableProof::UnrepresentableFinite,
            EmptyBranchProof::Raw,
            charge,
        )
    }

    /// Compiles finite nullable character-result ranges with two overflowing unequal bounds.
    ///
    /// Requires validated v/vi input and normalized capture names. Complete
    /// wrappers contain a bare class and a raw empty alternative under {m,n}.
    /// Both finite bounds exceed native storage and are numerically unequal.
    /// Borrowed decimal subtraction retains exact optional budgets, including
    /// padded endpoints and differences that fit native storage. All consumed
    /// characters can be mandatory: character-first priority is independent of
    /// laziness, while empty-first choices retain optional progress and outside
    /// word/line gates. Final captures and linear search reuse existing state.
    /// Every class operand is proved before matching. Other counts, partial
    /// captures and further atoms remain separate proofs.
    /// https://262.ecma-international.org/17.0/#sec-repeatmatcher
    pub fn compile_unrepresentable_finite_nullable_character_result_v_set_with_work<E>(
        source: &JsString,
        ignore_case: bool,
        multiline: bool,
        charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, crate::RegExpUnicodeSetCompileError<E>> {
        Self::compile_nullable_string_class_repetition_with_work(
            source,
            ignore_case,
            multiline,
            ClassProof::CharacterResultStrings,
            NullableProof::UnrepresentableFinite,
            EmptyBranchProof::Raw,
            charge,
        )
    }

    /// Compiles a nullable singleton-q class beside nested noncapturing empty groups.
    ///
    /// Requires validated v/vi input and normalized capture names. Complete
    /// ordinary/noncapturing wrappers contain one bare class and an empty branch
    /// consisting of one or more nested (?:) groups, followed by *, + or ? and
    /// an optional lazy marker. The empty groups have no captures, assertions,
    /// quantifiers or sibling sequences. Mandatory branch priority, optional
    /// progress, final empty/undefined/original character captures and canonical
    /// outside word/line gates reuse existing linear search. Every class operand
    /// is proved before matching. Other counts and partial capture shapes remain
    /// separate proofs; construction and clone/drop add no native recursion.
    /// https://262.ecma-international.org/17.0/#sec-repeatmatcher
    pub fn compile_noncapturing_empty_nullable_v_set_with_single_character_strings_and_work<E>(
        source: &JsString,
        ignore_case: bool,
        multiline: bool,
        charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, crate::RegExpUnicodeSetCompileError<E>> {
        Self::compile_nullable_string_class_repetition_with_work(
            source,
            ignore_case,
            multiline,
            ClassProof::SingleCharacterStrings,
            NullableProof::Raw,
            EmptyBranchProof::Nested,
            charge,
        )
    }

    /// Compiles a nullable character-result class beside nested noncapturing empty groups.
    ///
    /// Requires validated v/vi input and normalized capture names. Complete
    /// ordinary/noncapturing wrappers contain one bare class and an empty branch
    /// consisting of one or more nested (?:) groups, followed by *, + or ? and
    /// an optional lazy marker. The empty groups have no captures, assertions,
    /// quantifiers or sibling sequences. Mandatory branch priority, optional
    /// progress, final empty/undefined/original character captures and canonical
    /// outside word/line gates reuse existing linear search. Every class operand
    /// is proved before matching. Other counts and partial capture shapes remain
    /// separate proofs; construction and clone/drop add no native recursion.
    /// https://262.ecma-international.org/17.0/#sec-repeatmatcher
    pub fn compile_noncapturing_empty_nullable_character_result_v_set_with_work<E>(
        source: &JsString,
        ignore_case: bool,
        multiline: bool,
        charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, crate::RegExpUnicodeSetCompileError<E>> {
        Self::compile_nullable_string_class_repetition_with_work(
            source,
            ignore_case,
            multiline,
            ClassProof::CharacterResultStrings,
            NullableProof::Raw,
            EmptyBranchProof::Nested,
            charge,
        )
    }

    /// Compiles zero-minimum braced nullable singleton-q counts beside nested empty groups.
    ///
    /// Requires validated v/vi input and normalized capture names. Complete
    /// wrappers contain one bare class and an effect-free nested noncapturing
    /// empty group branch. Braced {0}, {0,n} and {0,} include padded/lazy
    /// forms and overflowing upper bounds. Optional empty iterations cannot
    /// export captures; zero iterations leave all inner captures undefined.
    /// Canonical outside word/line gates and original character ranges reuse
    /// existing linear search. Every class operand is proved even under zero
    /// counts. Other counts, empty sequences/choices and partial capture shapes
    /// remain separate proofs; construction and clone/drop are iterative.
    /// https://262.ecma-international.org/17.0/#sec-repeatmatcher
    pub fn compile_zero_minimum_noncapturing_empty_nullable_v_set_with_single_character_strings_and_work<
        E,
    >(
        source: &JsString,
        ignore_case: bool,
        multiline: bool,
        charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, crate::RegExpUnicodeSetCompileError<E>> {
        Self::compile_nullable_string_class_repetition_with_work(
            source,
            ignore_case,
            multiline,
            ClassProof::SingleCharacterStrings,
            NullableProof::ZeroMinimum,
            EmptyBranchProof::Nested,
            charge,
        )
    }

    /// Compiles zero-minimum braced nullable character-result counts beside nested empty groups.
    ///
    /// Requires validated v/vi input and normalized capture names. Complete
    /// wrappers contain one bare class and an effect-free nested noncapturing
    /// empty group branch. Braced {0}, {0,n} and {0,} include padded/lazy
    /// forms and overflowing upper bounds. Optional empty iterations cannot
    /// export captures; zero iterations leave all inner captures undefined.
    /// Canonical outside word/line gates and original character ranges reuse
    /// existing linear search. Every class operand is proved even under zero
    /// counts. Other counts, empty sequences/choices and partial capture shapes
    /// remain separate proofs; construction and clone/drop are iterative.
    /// https://262.ecma-international.org/17.0/#sec-repeatmatcher
    pub fn compile_zero_minimum_noncapturing_empty_nullable_character_result_v_set_with_work<E>(
        source: &JsString,
        ignore_case: bool,
        multiline: bool,
        charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, crate::RegExpUnicodeSetCompileError<E>> {
        Self::compile_nullable_string_class_repetition_with_work(
            source,
            ignore_case,
            multiline,
            ClassProof::CharacterResultStrings,
            NullableProof::ZeroMinimum,
            EmptyBranchProof::Nested,
            charge,
        )
    }

    /// Compiles {1,} nullable singleton-q counts beside nested noncapturing empty groups.
    ///
    /// Requires validated v/vi input and normalized capture names. Complete
    /// wrappers contain one bare class and an effect-free nested empty group
    /// alternative under {1,}, including padded/lazy forms. A mandatory empty
    /// iteration exports defined empty inner captures; optional empty iterations
    /// cannot overwrite the last selected original character. Source branch
    /// priority and canonical outside word/line gates reuse linear search.
    /// Every class operand is proved before matching. Other counts, empty
    /// sequences/choices/quantifiers and partial captures remain separate proofs.
    /// Source storage and clone/drop are flat and counts are never expanded.
    /// https://262.ecma-international.org/17.0/#sec-repeatmatcher
    pub fn compile_one_minimum_open_noncapturing_empty_nullable_v_set_with_single_character_strings_and_work<
        E,
    >(
        source: &JsString,
        ignore_case: bool,
        multiline: bool,
        charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, crate::RegExpUnicodeSetCompileError<E>> {
        Self::compile_nullable_string_class_repetition_with_work(
            source,
            ignore_case,
            multiline,
            ClassProof::SingleCharacterStrings,
            NullableProof::OneMinimumOpen,
            EmptyBranchProof::Nested,
            charge,
        )
    }

    /// Compiles {1,} nullable character-result counts beside nested noncapturing empty groups.
    ///
    /// Requires validated v/vi input and normalized capture names. Complete
    /// wrappers contain one bare class and an effect-free nested empty group
    /// alternative under {1,}, including padded/lazy forms. A mandatory empty
    /// iteration exports defined empty inner captures; optional empty iterations
    /// cannot overwrite the last selected original character. Source branch
    /// priority and canonical outside word/line gates reuse linear search.
    /// Every class operand is proved before matching. Other counts, empty
    /// sequences/choices/quantifiers and partial captures remain separate proofs.
    /// Source storage and clone/drop are flat and counts are never expanded.
    /// https://262.ecma-international.org/17.0/#sec-repeatmatcher
    pub fn compile_one_minimum_open_noncapturing_empty_nullable_character_result_v_set_with_work<
        E,
    >(
        source: &JsString,
        ignore_case: bool,
        multiline: bool,
        charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, crate::RegExpUnicodeSetCompileError<E>> {
        Self::compile_nullable_string_class_repetition_with_work(
            source,
            ignore_case,
            multiline,
            ClassProof::CharacterResultStrings,
            NullableProof::OneMinimumOpen,
            EmptyBranchProof::Nested,
            charge,
        )
    }

    /// Compiles exactly-one nullable singleton-q counts beside nested empty groups.
    ///
    /// Requires validated v/vi input and normalized capture names. Complete
    /// wrappers contain one bare class and a nested effect-free noncapturing
    /// empty alternative under {1} or {1,1}, including padded/lazy forms.
    /// The one mandatory iteration follows source branch priority independently
    /// of greediness, retaining defined empty or original character captures.
    /// Canonical outside word/line gates reuse existing linear search. Every
    /// class operand is proved before matching. Other counts and partial/other
    /// empty-branch shapes remain separate proofs; storage and clone/drop stay flat.
    /// https://262.ecma-international.org/17.0/#sec-repeatmatcher
    pub fn compile_one_iteration_noncapturing_empty_nullable_v_set_with_single_character_strings_and_work<
        E,
    >(
        source: &JsString,
        ignore_case: bool,
        multiline: bool,
        charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, crate::RegExpUnicodeSetCompileError<E>> {
        Self::compile_nullable_string_class_repetition_with_work(
            source,
            ignore_case,
            multiline,
            ClassProof::SingleCharacterStrings,
            NullableProof::OneIteration,
            EmptyBranchProof::Nested,
            charge,
        )
    }

    /// Compiles exactly-one nullable character-result counts beside nested empty groups.
    ///
    /// Requires validated v/vi input and normalized capture names. Complete
    /// wrappers contain one bare class and a nested effect-free noncapturing
    /// empty alternative under {1} or {1,1}, including padded/lazy forms.
    /// The one mandatory iteration follows source branch priority independently
    /// of greediness, retaining defined empty or original character captures.
    /// Canonical outside word/line gates reuse existing linear search. Every
    /// class operand is proved before matching. Other counts and partial/other
    /// empty-branch shapes remain separate proofs; storage and clone/drop stay flat.
    /// https://262.ecma-international.org/17.0/#sec-repeatmatcher
    pub fn compile_one_iteration_noncapturing_empty_nullable_character_result_v_set_with_work<E>(
        source: &JsString,
        ignore_case: bool,
        multiline: bool,
        charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, crate::RegExpUnicodeSetCompileError<E>> {
        Self::compile_nullable_string_class_repetition_with_work(
            source,
            ignore_case,
            multiline,
            ClassProof::CharacterResultStrings,
            NullableProof::OneIteration,
            EmptyBranchProof::Nested,
            charge,
        )
    }

    /// Compiles positive open nullable singleton-q counts beside nested empty groups.
    ///
    /// Requires validated v/vi input and normalized capture names. Complete
    /// wrappers contain one bare class and a nested effect-free noncapturing
    /// empty alternative under {m,}, with positive represented or overflowing m.
    /// Padded/lazy forms preserve mandatory source priority and defined final
    /// empty captures when too few characters satisfy the required iterations.
    /// Optional progress, original character ranges and canonical outside gates
    /// reuse existing linear search. Every class operand is proved before search.
    /// Other counts and partial/other-empty shapes remain separate proofs.
    /// Counts never expand state; source storage and clone/drop remain flat.
    /// https://262.ecma-international.org/17.0/#sec-repeatmatcher
    pub fn compile_positive_open_noncapturing_empty_nullable_v_set_with_single_character_strings_and_work<
        E,
    >(
        source: &JsString,
        ignore_case: bool,
        multiline: bool,
        charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, crate::RegExpUnicodeSetCompileError<E>> {
        Self::compile_nullable_string_class_repetition_with_work(
            source,
            ignore_case,
            multiline,
            ClassProof::SingleCharacterStrings,
            NullableProof::PositiveOpen,
            EmptyBranchProof::Nested,
            charge,
        )
    }

    /// Compiles positive open nullable character-result counts beside nested empty groups.
    ///
    /// Requires validated v/vi input and normalized capture names. Complete
    /// wrappers contain one bare class and a nested effect-free noncapturing
    /// empty alternative under {m,}, with positive represented or overflowing m.
    /// Padded/lazy forms preserve mandatory source priority and defined final
    /// empty captures when too few characters satisfy the required iterations.
    /// Optional progress, original character ranges and canonical outside gates
    /// reuse existing linear search. Every class operand is proved before search.
    /// Other counts and partial/other-empty shapes remain separate proofs.
    /// Counts never expand state; source storage and clone/drop remain flat.
    /// https://262.ecma-international.org/17.0/#sec-repeatmatcher
    pub fn compile_positive_open_noncapturing_empty_nullable_character_result_v_set_with_work<E>(
        source: &JsString,
        ignore_case: bool,
        multiline: bool,
        charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, crate::RegExpUnicodeSetCompileError<E>> {
        Self::compile_nullable_string_class_repetition_with_work(
            source,
            ignore_case,
            multiline,
            ClassProof::CharacterResultStrings,
            NullableProof::PositiveOpen,
            EmptyBranchProof::Nested,
            charge,
        )
    }

    /// Compiles represented positive exact nullable singleton-q counts beside nested empty groups.
    ///
    /// Requires validated v/vi input and normalized capture names. Complete
    /// wrappers contain one bare class and a nested effect-free noncapturing
    /// empty alternative under {m} or {m,m}, with represented m greater than one.
    /// Padded/lazy forms preserve mandatory branch priority independently of
    /// greediness and retain final required empty or original character captures.
    /// Canonical outside word/line gates reuse existing linear search. Every
    /// class operand is proved before matching; counts never expand state.
    /// Other bounds, partial captures and other empty-branch shapes remain
    /// separate proofs. Source storage and clone/drop stay flat.
    /// https://262.ecma-international.org/17.0/#sec-repeatmatcher
    pub fn compile_positive_exact_noncapturing_empty_nullable_v_set_with_single_character_strings_and_work<
        E,
    >(
        source: &JsString,
        ignore_case: bool,
        multiline: bool,
        charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, crate::RegExpUnicodeSetCompileError<E>> {
        Self::compile_nullable_string_class_repetition_with_work(
            source,
            ignore_case,
            multiline,
            ClassProof::SingleCharacterStrings,
            NullableProof::PositiveExact,
            EmptyBranchProof::Nested,
            charge,
        )
    }

    /// Compiles represented positive exact nullable character-result counts beside nested empty groups.
    ///
    /// Requires validated v/vi input and normalized capture names. Complete
    /// wrappers contain one bare class and a nested effect-free noncapturing
    /// empty alternative under {m} or {m,m}, with represented m greater than one.
    /// Padded/lazy forms preserve mandatory branch priority independently of
    /// greediness and retain final required empty or original character captures.
    /// Canonical outside word/line gates reuse existing linear search. Every
    /// class operand is proved before matching; counts never expand state.
    /// Other bounds, partial captures and other empty-branch shapes remain
    /// separate proofs. Source storage and clone/drop stay flat.
    /// https://262.ecma-international.org/17.0/#sec-repeatmatcher
    pub fn compile_positive_exact_noncapturing_empty_nullable_character_result_v_set_with_work<
        E,
    >(
        source: &JsString,
        ignore_case: bool,
        multiline: bool,
        charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, crate::RegExpUnicodeSetCompileError<E>> {
        Self::compile_nullable_string_class_repetition_with_work(
            source,
            ignore_case,
            multiline,
            ClassProof::CharacterResultStrings,
            NullableProof::PositiveExact,
            EmptyBranchProof::Nested,
            charge,
        )
    }

    /// Compiles overflowing exact nullable singleton-q counts beside nested empty groups.
    ///
    /// Requires validated v/vi input and normalized capture names. Complete
    /// wrappers contain one bare class and a nested effect-free noncapturing
    /// empty alternative under {m} or {m,m}, with unrepresentable m. Borrowed
    /// decimal comparison proves padded endpoint equality without expanding
    /// counts. Mandatory source priority is independent of greediness and retains
    /// final required empty or original character captures. Canonical outside
    /// word/line gates reuse existing linear search. Every class leaf is proved
    /// behind empty-first choices. Other counts and partial/other-empty shapes
    /// remain separate proofs; storage and clone/drop are flat.
    /// https://262.ecma-international.org/17.0/#sec-repeatmatcher
    pub fn compile_unrepresentable_exact_noncapturing_empty_nullable_v_set_with_single_character_strings_and_work<
        E,
    >(
        source: &JsString,
        ignore_case: bool,
        multiline: bool,
        charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, crate::RegExpUnicodeSetCompileError<E>> {
        Self::compile_nullable_string_class_repetition_with_work(
            source,
            ignore_case,
            multiline,
            ClassProof::SingleCharacterStrings,
            NullableProof::UnrepresentableExact,
            EmptyBranchProof::Nested,
            charge,
        )
    }

    /// Compiles overflowing exact nullable character-result counts beside nested empty groups.
    ///
    /// Requires validated v/vi input and normalized capture names. Complete
    /// wrappers contain one bare class and a nested effect-free noncapturing
    /// empty alternative under {m} or {m,m}, with unrepresentable m. Borrowed
    /// decimal comparison proves padded endpoint equality without expanding
    /// counts. Mandatory source priority is independent of greediness and retains
    /// final required empty or original character captures. Canonical outside
    /// word/line gates reuse existing linear search. Every class leaf is proved
    /// behind empty-first choices. Other counts and partial/other-empty shapes
    /// remain separate proofs; storage and clone/drop are flat.
    /// https://262.ecma-international.org/17.0/#sec-repeatmatcher
    pub fn compile_unrepresentable_exact_noncapturing_empty_nullable_character_result_v_set_with_work<
        E,
    >(
        source: &JsString,
        ignore_case: bool,
        multiline: bool,
        charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, crate::RegExpUnicodeSetCompileError<E>> {
        Self::compile_nullable_string_class_repetition_with_work(
            source,
            ignore_case,
            multiline,
            ClassProof::CharacterResultStrings,
            NullableProof::UnrepresentableExact,
            EmptyBranchProof::Nested,
            charge,
        )
    }

    /// Compiles represented finite nullable singleton-q ranges beside nested empty groups.
    ///
    /// Requires validated v/vi input and normalized capture names. Complete
    /// wrappers contain one bare class and a nested effect-free noncapturing
    /// empty alternative under {m,n}, with represented positive m and greater n.
    /// Padded/lazy forms preserve mandatory branch priority, exact optional
    /// budgets, earliest cross-phase starts and final required empty/original
    /// character captures. Canonical outside word/line gates reuse linear search.
    /// Every class leaf is proved behind empty-first choices. Other bounds and
    /// partial/other-empty shapes remain separate proofs; counts never expand
    /// matching state and source storage and clone/drop remain flat.
    /// https://262.ecma-international.org/17.0/#sec-repeatmatcher
    pub fn compile_positive_finite_noncapturing_empty_nullable_v_set_with_single_character_strings_and_work<
        E,
    >(
        source: &JsString,
        ignore_case: bool,
        multiline: bool,
        charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, crate::RegExpUnicodeSetCompileError<E>> {
        Self::compile_nullable_string_class_repetition_with_work(
            source,
            ignore_case,
            multiline,
            ClassProof::SingleCharacterStrings,
            NullableProof::PositiveFinite,
            EmptyBranchProof::Nested,
            charge,
        )
    }

    /// Compiles represented finite nullable character-result ranges beside nested empty groups.
    ///
    /// Requires validated v/vi input and normalized capture names. Complete
    /// wrappers contain one bare class and a nested effect-free noncapturing
    /// empty alternative under {m,n}, with represented positive m and greater n.
    /// Padded/lazy forms preserve mandatory branch priority, exact optional
    /// budgets, earliest cross-phase starts and final required empty/original
    /// character captures. Canonical outside word/line gates reuse linear search.
    /// Every class leaf is proved behind empty-first choices. Other bounds and
    /// partial/other-empty shapes remain separate proofs; counts never expand
    /// matching state and source storage and clone/drop remain flat.
    /// https://262.ecma-international.org/17.0/#sec-repeatmatcher
    pub fn compile_positive_finite_noncapturing_empty_nullable_character_result_v_set_with_work<
        E,
    >(
        source: &JsString,
        ignore_case: bool,
        multiline: bool,
        charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, crate::RegExpUnicodeSetCompileError<E>> {
        Self::compile_nullable_string_class_repetition_with_work(
            source,
            ignore_case,
            multiline,
            ClassProof::CharacterResultStrings,
            NullableProof::PositiveFinite,
            EmptyBranchProof::Nested,
            charge,
        )
    }

    /// Compiles nullable singleton-q ranges beside nested empty groups with an overflowing maximum.
    ///
    /// Requires validated v/vi input and normalized capture names. Complete
    /// wrappers contain one bare class and a nested effect-free noncapturing
    /// empty alternative under {m,n}, with represented positive m and overflowing n.
    /// Borrowed decimal subtraction preserves exact optional budgets even when
    /// n-m fits native storage. Padded/lazy forms retain mandatory source priority,
    /// earliest cross-phase starts and required empty/original character captures.
    /// Canonical outside word/line gates reuse linear search. Every class leaf is
    /// proved behind empty-first choices. Other counts and partial/other-empty
    /// shapes remain separate proofs; counts and groups never expand state.
    /// https://262.ecma-international.org/17.0/#sec-repeatmatcher
    pub fn compile_overflowing_maximum_noncapturing_empty_nullable_v_set_with_single_character_strings_and_work<
        E,
    >(
        source: &JsString,
        ignore_case: bool,
        multiline: bool,
        charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, crate::RegExpUnicodeSetCompileError<E>> {
        Self::compile_nullable_string_class_repetition_with_work(
            source,
            ignore_case,
            multiline,
            ClassProof::SingleCharacterStrings,
            NullableProof::OverflowingMaximum,
            EmptyBranchProof::Nested,
            charge,
        )
    }

    /// Compiles nullable character-result ranges beside nested empty groups with an overflowing maximum.
    ///
    /// Requires validated v/vi input and normalized capture names. Complete
    /// wrappers contain one bare class and a nested effect-free noncapturing
    /// empty alternative under {m,n}, with represented positive m and overflowing n.
    /// Borrowed decimal subtraction preserves exact optional budgets even when
    /// n-m fits native storage. Padded/lazy forms retain mandatory source priority,
    /// earliest cross-phase starts and required empty/original character captures.
    /// Canonical outside word/line gates reuse linear search. Every class leaf is
    /// proved behind empty-first choices. Other counts and partial/other-empty
    /// shapes remain separate proofs; counts and groups never expand state.
    /// https://262.ecma-international.org/17.0/#sec-repeatmatcher
    pub fn compile_overflowing_maximum_noncapturing_empty_nullable_character_result_v_set_with_work<
        E,
    >(
        source: &JsString,
        ignore_case: bool,
        multiline: bool,
        charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, crate::RegExpUnicodeSetCompileError<E>> {
        Self::compile_nullable_string_class_repetition_with_work(
            source,
            ignore_case,
            multiline,
            ClassProof::CharacterResultStrings,
            NullableProof::OverflowingMaximum,
            EmptyBranchProof::Nested,
            charge,
        )
    }

    /// Compiles grouped-empty singleton-q ranges with two overflowing unequal bounds.
    ///
    /// Requires validated v/vi input and normalized capture names. Complete
    /// wrappers contain a bare class and a nested effect-free noncapturing
    /// empty alternative under {m,n}.
    /// Both finite bounds exceed native storage and are numerically unequal.
    /// Borrowed decimal subtraction retains exact optional budgets, including
    /// padded endpoints and differences that fit native storage. All consumed
    /// characters can be mandatory: character-first priority is independent of
    /// laziness, while empty-first choices retain optional progress and outside
    /// word/line gates. Final captures and linear search reuse existing state.
    /// Every class operand is proved before matching. Other counts, partial
    /// captures and other empty shapes remain separate proofs.
    /// https://262.ecma-international.org/17.0/#sec-repeatmatcher
    pub fn compile_unrepresentable_finite_noncapturing_empty_nullable_v_set_with_single_character_strings_and_work<
        E,
    >(
        source: &JsString,
        ignore_case: bool,
        multiline: bool,
        charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, crate::RegExpUnicodeSetCompileError<E>> {
        Self::compile_nullable_string_class_repetition_with_work(
            source,
            ignore_case,
            multiline,
            ClassProof::SingleCharacterStrings,
            NullableProof::UnrepresentableFinite,
            EmptyBranchProof::Nested,
            charge,
        )
    }

    /// Compiles grouped-empty character-result ranges with two overflowing unequal bounds.
    ///
    /// Requires validated v/vi input and normalized capture names. Complete
    /// wrappers contain a bare class and a nested effect-free noncapturing
    /// empty alternative under {m,n}.
    /// Both finite bounds exceed native storage and are numerically unequal.
    /// Borrowed decimal subtraction retains exact optional budgets, including
    /// padded endpoints and differences that fit native storage. All consumed
    /// characters can be mandatory: character-first priority is independent of
    /// laziness, while empty-first choices retain optional progress and outside
    /// word/line gates. Final captures and linear search reuse existing state.
    /// Every class operand is proved before matching. Other counts, partial
    /// captures and other empty shapes remain separate proofs.
    /// https://262.ecma-international.org/17.0/#sec-repeatmatcher
    pub fn compile_unrepresentable_finite_noncapturing_empty_nullable_character_result_v_set_with_work<
        E,
    >(
        source: &JsString,
        ignore_case: bool,
        multiline: bool,
        charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, crate::RegExpUnicodeSetCompileError<E>> {
        Self::compile_nullable_string_class_repetition_with_work(
            source,
            ignore_case,
            multiline,
            ClassProof::CharacterResultStrings,
            NullableProof::UnrepresentableFinite,
            EmptyBranchProof::Nested,
            charge,
        )
    }

    /// Compiles a nullable singleton-q class beside noncapturing empty sequences.
    ///
    /// Requires validated v/vi input and normalized capture names. Complete
    /// wrappers contain one bare class and a nonempty sequence of effect-free
    /// noncapturing empty groups, with arbitrary nesting, under *, + or ? and an
    /// optional lazy marker. Every class leaf is proved behind empty-first
    /// choices. Mandatory priority, optional progress, captures and canonical
    /// outside word/line gates reuse linear search. Quantified empty groups,
    /// choices, capturing empty groups and partial captures remain separate
    /// proofs. Construction and clone/drop add no native recursion.
    /// https://262.ecma-international.org/17.0/#sec-repeatmatcher
    pub fn compile_noncapturing_empty_sequence_nullable_v_set_with_single_character_strings_and_work<
        E,
    >(
        source: &JsString,
        ignore_case: bool,
        multiline: bool,
        charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, crate::RegExpUnicodeSetCompileError<E>> {
        Self::compile_nullable_string_class_repetition_with_work(
            source,
            ignore_case,
            multiline,
            ClassProof::SingleCharacterStrings,
            NullableProof::Raw,
            EmptyBranchProof::Sequences,
            charge,
        )
    }

    /// Compiles a nullable character-result class beside noncapturing empty sequences.
    ///
    /// Requires validated v/vi input and normalized capture names. Complete
    /// wrappers contain one bare class and a nonempty sequence of effect-free
    /// noncapturing empty groups, with arbitrary nesting, under *, + or ? and an
    /// optional lazy marker. Every class leaf is proved behind empty-first
    /// choices. Mandatory priority, optional progress, captures and canonical
    /// outside word/line gates reuse linear search. Quantified empty groups,
    /// choices, capturing empty groups and partial captures remain separate
    /// proofs. Construction and clone/drop add no native recursion.
    /// https://262.ecma-international.org/17.0/#sec-repeatmatcher
    pub fn compile_noncapturing_empty_sequence_nullable_character_result_v_set_with_work<E>(
        source: &JsString,
        ignore_case: bool,
        multiline: bool,
        charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, crate::RegExpUnicodeSetCompileError<E>> {
        Self::compile_nullable_string_class_repetition_with_work(
            source,
            ignore_case,
            multiline,
            ClassProof::CharacterResultStrings,
            NullableProof::Raw,
            EmptyBranchProof::Sequences,
            charge,
        )
    }

    /// Compiles counted nullable singleton-q classes beside noncapturing empty sequences.
    ///
    /// Requires validated v/vi input and normalized capture names. Complete
    /// wrappers contain one bare class and a nonempty sequence of effect-free
    /// noncapturing empty groups under a braced count, with an optional lazy
    /// marker. Classifying zero, exact, open and finite bounds reuses the existing
    /// count proofs, including overflowing endpoints and exact decimal optional
    /// budgets. Mandatory priority, earliest starts, captures and outside gates
    /// retain linear search. Every class leaf is proved behind empty-first
    /// choices. Quantified/choice empty groups and partial captures remain
    /// separate proofs; source counts and groups never expand matching state.
    /// https://262.ecma-international.org/17.0/#sec-repeatmatcher
    pub fn compile_counted_noncapturing_empty_sequence_nullable_v_set_with_single_character_strings_and_work<
        E,
    >(
        source: &JsString,
        ignore_case: bool,
        multiline: bool,
        charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, crate::RegExpUnicodeSetCompileError<E>> {
        Self::compile_nullable_string_class_repetition_with_work(
            source,
            ignore_case,
            multiline,
            ClassProof::SingleCharacterStrings,
            NullableProof::Counted,
            EmptyBranchProof::Sequences,
            charge,
        )
    }

    /// Compiles counted nullable character-result classes beside noncapturing empty sequences.
    ///
    /// Requires validated v/vi input and normalized capture names. Complete
    /// wrappers contain one bare class and a nonempty sequence of effect-free
    /// noncapturing empty groups under a braced count, with an optional lazy
    /// marker. Classifying zero, exact, open and finite bounds reuses the existing
    /// count proofs, including overflowing endpoints and exact decimal optional
    /// budgets. Mandatory priority, earliest starts, captures and outside gates
    /// retain linear search. Every class leaf is proved behind empty-first
    /// choices. Quantified/choice empty groups and partial captures remain
    /// separate proofs; source counts and groups never expand matching state.
    /// https://262.ecma-international.org/17.0/#sec-repeatmatcher
    pub fn compile_counted_noncapturing_empty_sequence_nullable_character_result_v_set_with_work<
        E,
    >(
        source: &JsString,
        ignore_case: bool,
        multiline: bool,
        charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, crate::RegExpUnicodeSetCompileError<E>> {
        Self::compile_nullable_string_class_repetition_with_work(
            source,
            ignore_case,
            multiline,
            ClassProof::CharacterResultStrings,
            NullableProof::Counted,
            EmptyBranchProof::Sequences,
            charge,
        )
    }

    /// Compiles nullable singleton-q classes beside quantified empty sequences.
    ///
    /// Requires validated v/vi input and normalized capture names. Complete
    /// wrappers contain one bare class and a nonempty sequence of effect-free
    /// noncapturing empty groups under *, + or ?, with an optional lazy marker.
    /// Empty groups may nest, have siblings and own validated greedy/lazy counts,
    /// including overflowing counts. Their complete iterative proof contributes
    /// no captures, assertions or input consumption. Every class leaf is proved
    /// behind empty-first choices. Existing repetition proofs preserve mandatory
    /// priority, progress, earliest starts, captures and outside word/line gates.
    /// Choices and partial/capturing empty shapes remain separate proofs.
    /// Neither empty-group counts nor outer counts expand matching state.
    /// https://262.ecma-international.org/17.0/#sec-repeatmatcher
    pub fn compile_quantified_empty_sequence_nullable_v_set_with_single_character_strings_and_work<
        E,
    >(
        source: &JsString,
        ignore_case: bool,
        multiline: bool,
        charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, crate::RegExpUnicodeSetCompileError<E>> {
        Self::compile_nullable_string_class_repetition_with_work(
            source,
            ignore_case,
            multiline,
            ClassProof::SingleCharacterStrings,
            NullableProof::Raw,
            EmptyBranchProof::QuantifiedSequences,
            charge,
        )
    }

    /// Compiles nullable character-result classes beside quantified empty sequences.
    ///
    /// Requires validated v/vi input and normalized capture names. Complete
    /// wrappers contain one bare class and a nonempty sequence of effect-free
    /// noncapturing empty groups under *, + or ?, with an optional lazy marker.
    /// Empty groups may nest, have siblings and own validated greedy/lazy counts,
    /// including overflowing counts. Their complete iterative proof contributes
    /// no captures, assertions or input consumption. Every class leaf is proved
    /// behind empty-first choices. Existing repetition proofs preserve mandatory
    /// priority, progress, earliest starts, captures and outside word/line gates.
    /// Choices and partial/capturing empty shapes remain separate proofs.
    /// Neither empty-group counts nor outer counts expand matching state.
    /// https://262.ecma-international.org/17.0/#sec-repeatmatcher
    pub fn compile_quantified_empty_sequence_nullable_character_result_v_set_with_work<E>(
        source: &JsString,
        ignore_case: bool,
        multiline: bool,
        charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, crate::RegExpUnicodeSetCompileError<E>> {
        Self::compile_nullable_string_class_repetition_with_work(
            source,
            ignore_case,
            multiline,
            ClassProof::CharacterResultStrings,
            NullableProof::Raw,
            EmptyBranchProof::QuantifiedSequences,
            charge,
        )
    }

    /// Compiles nullable singleton-q classes beside quantified empty sequences.
    ///
    /// Requires validated v/vi input and normalized capture names. Complete
    /// wrappers contain one bare class and a nonempty sequence of effect-free
    /// noncapturing empty groups under a braced count, with an optional lazy marker.
    /// Empty groups may nest, have siblings and own validated greedy/lazy counts,
    /// including overflowing counts. Their complete iterative proof contributes
    /// no captures, assertions or input consumption. Every class leaf is proved
    /// behind empty-first choices. Existing repetition proofs preserve mandatory
    /// priority, progress, earliest starts, captures and outside word/line gates.
    /// Choices and partial/capturing empty shapes remain separate proofs.
    /// Neither empty-group counts nor outer counts expand matching state.
    /// https://262.ecma-international.org/17.0/#sec-repeatmatcher
    pub fn compile_counted_quantified_empty_sequence_nullable_v_set_with_single_character_strings_and_work<
        E,
    >(
        source: &JsString,
        ignore_case: bool,
        multiline: bool,
        charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, crate::RegExpUnicodeSetCompileError<E>> {
        Self::compile_nullable_string_class_repetition_with_work(
            source,
            ignore_case,
            multiline,
            ClassProof::SingleCharacterStrings,
            NullableProof::Counted,
            EmptyBranchProof::QuantifiedSequences,
            charge,
        )
    }

    /// Compiles nullable character-result classes beside quantified empty sequences.
    ///
    /// Requires validated v/vi input and normalized capture names. Complete
    /// wrappers contain one bare class and a nonempty sequence of effect-free
    /// noncapturing empty groups under a braced count, with an optional lazy marker.
    /// Empty groups may nest, have siblings and own validated greedy/lazy counts,
    /// including overflowing counts. Their complete iterative proof contributes
    /// no captures, assertions or input consumption. Every class leaf is proved
    /// behind empty-first choices. Existing repetition proofs preserve mandatory
    /// priority, progress, earliest starts, captures and outside word/line gates.
    /// Choices and partial/capturing empty shapes remain separate proofs.
    /// Neither empty-group counts nor outer counts expand matching state.
    /// https://262.ecma-international.org/17.0/#sec-repeatmatcher
    pub fn compile_counted_quantified_empty_sequence_nullable_character_result_v_set_with_work<
        E,
    >(
        source: &JsString,
        ignore_case: bool,
        multiline: bool,
        charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, crate::RegExpUnicodeSetCompileError<E>> {
        Self::compile_nullable_string_class_repetition_with_work(
            source,
            ignore_case,
            multiline,
            ClassProof::CharacterResultStrings,
            NullableProof::Counted,
            EmptyBranchProof::QuantifiedSequences,
            charge,
        )
    }

    /// Compiles nullable singleton-q classes beside effect-free empty choices.
    ///
    /// Requires validated v/vi input and normalized capture names. Complete
    /// wrappers contain one bare class and a nonempty sequence of noncapturing
    /// empty groups under *, + or ?, with an optional lazy marker. Empty groups
    /// may nest, have siblings, choices and validated greedy/lazy or huge counts.
    /// Every alternative must be empty and contribute no captures or assertions.
    /// Balanced edge scans locate the complete empty branch independently of
    /// bars and escaped parentheses in q payloads. Every class leaf is proved
    /// behind empty-first choices. Existing repetition proofs preserve mandatory
    /// priority, progress, budgets, earliest starts, captures and outside gates.
    /// Consuming, asserted or capturing empty branches remain separate proofs.
    /// Construction and clone/drop add no native recursion or expanded counts.
    /// https://262.ecma-international.org/17.0/#sec-repeatmatcher
    pub fn compile_empty_choice_nullable_v_set_with_single_character_strings_and_work<E>(
        source: &JsString,
        ignore_case: bool,
        multiline: bool,
        charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, crate::RegExpUnicodeSetCompileError<E>> {
        Self::compile_nullable_string_class_repetition_with_work(
            source,
            ignore_case,
            multiline,
            ClassProof::SingleCharacterStrings,
            NullableProof::Raw,
            EmptyBranchProof::Choices,
            charge,
        )
    }

    /// Compiles nullable character-result classes beside effect-free empty choices.
    ///
    /// Requires validated v/vi input and normalized capture names. Complete
    /// wrappers contain one bare class and a nonempty sequence of noncapturing
    /// empty groups under *, + or ?, with an optional lazy marker. Empty groups
    /// may nest, have siblings, choices and validated greedy/lazy or huge counts.
    /// Every alternative must be empty and contribute no captures or assertions.
    /// Balanced edge scans locate the complete empty branch independently of
    /// bars and escaped parentheses in q payloads. Every class leaf is proved
    /// behind empty-first choices. Existing repetition proofs preserve mandatory
    /// priority, progress, budgets, earliest starts, captures and outside gates.
    /// Consuming, asserted or capturing empty branches remain separate proofs.
    /// Construction and clone/drop add no native recursion or expanded counts.
    /// https://262.ecma-international.org/17.0/#sec-repeatmatcher
    pub fn compile_empty_choice_nullable_character_result_v_set_with_work<E>(
        source: &JsString,
        ignore_case: bool,
        multiline: bool,
        charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, crate::RegExpUnicodeSetCompileError<E>> {
        Self::compile_nullable_string_class_repetition_with_work(
            source,
            ignore_case,
            multiline,
            ClassProof::CharacterResultStrings,
            NullableProof::Raw,
            EmptyBranchProof::Choices,
            charge,
        )
    }

    /// Compiles nullable singleton-q classes beside effect-free empty choices.
    ///
    /// Requires validated v/vi input and normalized capture names. Complete
    /// wrappers contain one bare class and a nonempty sequence of noncapturing
    /// empty groups under a braced count, with an optional lazy marker. Empty groups
    /// may nest, have siblings, choices and validated greedy/lazy or huge counts.
    /// Every alternative must be empty and contribute no captures or assertions.
    /// Balanced edge scans locate the complete empty branch independently of
    /// bars and escaped parentheses in q payloads. Every class leaf is proved
    /// behind empty-first choices. Existing repetition proofs preserve mandatory
    /// priority, progress, budgets, earliest starts, captures and outside gates.
    /// Consuming, asserted or capturing empty branches remain separate proofs.
    /// Construction and clone/drop add no native recursion or expanded counts.
    /// https://262.ecma-international.org/17.0/#sec-repeatmatcher
    pub fn compile_counted_empty_choice_nullable_v_set_with_single_character_strings_and_work<E>(
        source: &JsString,
        ignore_case: bool,
        multiline: bool,
        charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, crate::RegExpUnicodeSetCompileError<E>> {
        Self::compile_nullable_string_class_repetition_with_work(
            source,
            ignore_case,
            multiline,
            ClassProof::SingleCharacterStrings,
            NullableProof::Counted,
            EmptyBranchProof::Choices,
            charge,
        )
    }

    /// Compiles nullable character-result classes beside effect-free empty choices.
    ///
    /// Requires validated v/vi input and normalized capture names. Complete
    /// wrappers contain one bare class and a nonempty sequence of noncapturing
    /// empty groups under a braced count, with an optional lazy marker. Empty groups
    /// may nest, have siblings, choices and validated greedy/lazy or huge counts.
    /// Every alternative must be empty and contribute no captures or assertions.
    /// Balanced edge scans locate the complete empty branch independently of
    /// bars and escaped parentheses in q payloads. Every class leaf is proved
    /// behind empty-first choices. Existing repetition proofs preserve mandatory
    /// priority, progress, budgets, earliest starts, captures and outside gates.
    /// Consuming, asserted or capturing empty branches remain separate proofs.
    /// Construction and clone/drop add no native recursion or expanded counts.
    /// https://262.ecma-international.org/17.0/#sec-repeatmatcher
    pub fn compile_counted_empty_choice_nullable_character_result_v_set_with_work<E>(
        source: &JsString,
        ignore_case: bool,
        multiline: bool,
        charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, crate::RegExpUnicodeSetCompileError<E>> {
        Self::compile_nullable_string_class_repetition_with_work(
            source,
            ignore_case,
            multiline,
            ClassProof::CharacterResultStrings,
            NullableProof::Counted,
            EmptyBranchProof::Choices,
            charge,
        )
    }

    fn compile_nullable_string_class_repetition_with_work<E>(
        source: &JsString,
        ignore_case: bool,
        multiline: bool,
        proof: ClassProof,
        bounds: NullableProof,
        empty: EmptyBranchProof,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, crate::RegExpUnicodeSetCompileError<E>> {
        let units = source.code_units();
        // Edge parity scans <=2N, both simple-token parity scans <=2N and
        // both opener/closing walks <=4N; reserve 2N for boundary recognition.
        // Simple proofs decline braced tokens before the quantifier helper.
        charge(units.len().saturating_mul(10))
            .map_err(crate::RegExpUnicodeSetCompileError::Work)?;
        if empty == EmptyBranchProof::Nested {
            // Both branch scans, empty-group proofs and recovery of initial
            // empty-first openers are linear across both token interpretations.
            charge(units.len().saturating_mul(12))
                .map_err(crate::RegExpUnicodeSetCompileError::Work)?;
        }
        if empty == EmptyBranchProof::Sequences {
            // Across both token interpretations, branch/empty scans take <=12N;
            // trailing-close recovery and discarded-opener proof take <=4N.
            charge(units.len().saturating_mul(16))
                .map_err(crate::RegExpUnicodeSetCompileError::Work)?;
        }
        if empty == EmptyBranchProof::QuantifiedSequences {
            // Each token interpretation reserves <=16N for branch/empty walks,
            // decimal quantifier recognition/parsing and wrapper recovery.
            // Each group/token is scanned a bounded number of times; neither
            // inner nor outer decimal counts are expanded.
            charge(units.len().saturating_mul(32))
                .map_err(crate::RegExpUnicodeSetCompileError::Work)?;
        }
        if empty == EmptyBranchProof::Choices {
            // Quantified-empty recognition plus both balanced edge scans and
            // choice-depth tracking stay linear across both token interpretations.
            charge(units.len().saturating_mul(40))
                .map_err(crate::RegExpUnicodeSetCompileError::Work)?;
        }
        if bounds != NullableProof::Raw {
            // One full decimal parse and one simple-token retry take <=8N.
            // Source counts are compared and retained without expansion.
            charge(units.len().saturating_mul(8))
                .map_err(crate::RegExpUnicodeSetCompileError::Work)?;
        }
        let start_anchor = units.first() == Some(&0x5e);
        let end_anchor = units.last() == Some(&0x24) && !escaped(units, units.len() - 1);
        let mut begin = usize::from(start_anchor);
        let mut end = units.len() - usize::from(end_anchor);
        let start_word = match units.get(begin..end).and_then(|body| body.get(..2)) {
            Some([0x5c, kind @ (0x62 | 0x42)]) => {
                begin += 2;
                Some(*kind == 0x62)
            }
            _ => None,
        };
        let end_word = if end >= begin + 2
            && units[end - 2] == 0x5c
            && matches!(units[end - 1], 0x62 | 0x42)
            && !escaped(units, end - 2)
        {
            end -= 2;
            Some(units[end + 1] == 0x62)
        } else {
            None
        };
        let Some(body) = units.get(begin..end) else {
            return Ok(None);
        };
        for (end, greedy) in [
            (
                body.len()
                    .checked_sub(1)
                    .filter(|_| body.last() == Some(&0x3f)),
                false,
            ),
            (Some(body.len()), true),
        ] {
            let Some(end) = end else { continue };
            if bounds == NullableProof::Raw
                && !end
                    .checked_sub(1)
                    .and_then(|at| body.get(at))
                    .is_some_and(|unit| matches!(*unit, 0x2a | 0x2b | 0x3f))
            {
                continue;
            }
            let Some((atom_end, minimum, maximum)) = quantifier(body, end) else {
                continue;
            };
            let bounds = if bounds == NullableProof::Counted {
                if body.get(atom_end) != Some(&0x7b) {
                    continue;
                }
                match (minimum, maximum) {
                    (Some(0), _) => NullableProof::ZeroMinimum,
                    (Some(1), Some(1)) => NullableProof::OneIteration,
                    (Some(low), Some(high)) if low == high => NullableProof::PositiveExact,
                    (Some(low), Some(high)) if low < high => NullableProof::PositiveFinite,
                    (_, None) if end.checked_sub(2).and_then(|at| body.get(at)) == Some(&0x2c) => {
                        NullableProof::PositiveOpen
                    }
                    (Some(_), None) => NullableProof::OverflowingMaximum,
                    (None, None) => {
                        // Comma/zero-prefix scans and equality classify finite
                        // overflowing endpoints without expanding decimal counts.
                        charge((end - atom_end).saturating_mul(3))
                            .map_err(crate::RegExpUnicodeSetCompileError::Work)?;
                        let digits = &body[atom_end + 1..end - 1];
                        if let Some(comma) = digits.iter().position(|&unit| unit == 0x2c) {
                            if significant(&digits[..comma]) == significant(&digits[comma + 1..]) {
                                NullableProof::UnrepresentableExact
                            } else {
                                NullableProof::UnrepresentableFinite
                            }
                        } else {
                            NullableProof::UnrepresentableExact
                        }
                    }
                    _ => continue,
                }
            } else {
                bounds
            };
            if bounds == NullableProof::ZeroMinimum
                && (minimum != Some(0) || body.get(atom_end) != Some(&0x7b))
            {
                continue;
            }
            if bounds == NullableProof::OneMinimumOpen
                && (minimum != Some(1)
                    || body.get(atom_end) != Some(&0x7b)
                    || end.checked_sub(2).and_then(|at| body.get(at)) != Some(&0x2c)
                    || body.get(end - 1) != Some(&0x7d))
            {
                continue;
            }
            if bounds == NullableProof::OneIteration
                && (minimum != Some(1) || maximum != Some(1) || body.get(atom_end) != Some(&0x7b))
            {
                continue;
            }
            if bounds == NullableProof::PositiveOpen
                && (minimum == Some(0)
                    || body.get(atom_end) != Some(&0x7b)
                    || end.checked_sub(2).and_then(|at| body.get(at)) != Some(&0x2c)
                    || body.get(end - 1) != Some(&0x7d))
            {
                continue;
            }
            if bounds == NullableProof::PositiveExact
                && (!minimum.is_some_and(|minimum| minimum > 1)
                    || minimum != maximum
                    || body.get(atom_end) != Some(&0x7b))
            {
                continue;
            }
            if bounds == NullableProof::PositiveFinite
                && (body.get(atom_end) != Some(&0x7b)
                    || !matches!((minimum, maximum), (Some(minimum), Some(maximum)) if minimum > 0 && maximum > minimum))
            {
                continue;
            }
            let unrepresentable_exact = if bounds == NullableProof::UnrepresentableExact
                && minimum.is_none()
                && maximum.is_none()
                && body.get(atom_end) == Some(&0x7b)
            {
                // A comma scan, both zero-prefix scans and equality take <=3N.
                charge((end - atom_end).saturating_mul(3))
                    .map_err(crate::RegExpUnicodeSetCompileError::Work)?;
                let digits = &body[atom_end + 1..end - 1];
                if let Some(comma) = digits.iter().position(|&unit| unit == 0x2c) {
                    let first = &digits[..comma];
                    let last = &digits[comma + 1..];
                    !last.is_empty() && significant(first) == significant(last)
                } else {
                    true
                }
            } else {
                false
            };
            if bounds == NullableProof::UnrepresentableExact && !unrepresentable_exact {
                continue;
            }
            let unrepresentable_finite = if bounds == NullableProof::UnrepresentableFinite
                && minimum.is_none()
                && maximum.is_none()
                && body.get(atom_end) == Some(&0x7b)
            {
                // Comma/zero-prefix scans, equality and borrowed subtraction
                // take <=5N, including the native difference conversion.
                charge((end - atom_end).saturating_mul(5))
                    .map_err(crate::RegExpUnicodeSetCompileError::Work)?;
                let digits = &body[atom_end + 1..end - 1];
                digits
                    .iter()
                    .position(|&unit| unit == 0x2c)
                    .and_then(|comma| {
                        let first = significant(&digits[..comma]);
                        let last = significant(&digits[comma + 1..]);
                        (!last.is_empty() && first != last).then(|| decimal_difference(first, last))
                    })
            } else {
                None
            };
            if bounds == NullableProof::UnrepresentableFinite && unrepresentable_finite.is_none() {
                continue;
            }
            let overflowing_maximum = if bounds == NullableProof::OverflowingMaximum
                && minimum.is_some_and(|minimum| minimum > 0)
                && maximum.is_none()
                && body.get(atom_end) == Some(&0x7b)
            {
                // Comma/zero-prefix scans and borrowed decimal subtraction
                // take <=5N, including the native difference conversion.
                charge((end - atom_end).saturating_mul(5))
                    .map_err(crate::RegExpUnicodeSetCompileError::Work)?;
                let digits = &body[atom_end + 1..end - 1];
                digits
                    .iter()
                    .position(|&unit| unit == 0x2c)
                    .and_then(|comma| {
                        let first = significant(&digits[..comma]);
                        let last = significant(&digits[comma + 1..]);
                        (!last.is_empty()).then(|| decimal_difference(first, last))
                    })
            } else {
                None
            };
            if bounds == NullableProof::OverflowingMaximum && overflowing_maximum.is_none() {
                continue;
            }
            let (mut at, mut wrappers, mut captures) = (0, 0, 0);
            while body.get(at) == Some(&0x28) {
                if body.get(at + 1) == Some(&0x3f) {
                    if body.get(at + 2) != Some(&0x3a) {
                        break;
                    }
                    at += 3;
                } else {
                    at += 1;
                    captures += 1;
                }
                wrappers += 1;
            }
            if empty == EmptyBranchProof::Nested {
                // Initial noncapturing openers can belong to the empty-first
                // branch rather than complete wrappers. Recover them once;
                // probing every opener separately would be quadratic in depth.
                let mut after = at;
                while body.get(after) == Some(&0x29) {
                    after += 1;
                }
                let depth = after - at;
                if depth != 0 && depth < wrappers && body.get(after) == Some(&0x7c) {
                    if let Some(empty_start) = at.checked_sub(depth.saturating_mul(3)) {
                        if body[empty_start..at]
                            .chunks_exact(3)
                            .all(|chunk| chunk == [0x28, 0x3f, 0x3a])
                        {
                            wrappers -= depth;
                            at = empty_start;
                        }
                    }
                }
            }
            if matches!(
                empty,
                EmptyBranchProof::Sequences
                    | EmptyBranchProof::QuantifiedSequences
                    | EmptyBranchProof::Choices
            ) {
                // A bare class ends in ], so an empty-first branch leaves only
                // complete wrapper closes at the body's end. Recover its initial
                // noncapturing openers once, including sibling empty groups.
                let trailing = body[..atom_end]
                    .iter()
                    .rev()
                    .take_while(|&&unit| unit == 0x29)
                    .count();
                if trailing < wrappers {
                    let Some(width) = (wrappers - trailing).checked_mul(3) else {
                        continue;
                    };
                    let Some(empty_start) = at.checked_sub(width) else {
                        continue;
                    };
                    if !body[empty_start..at]
                        .chunks_exact(3)
                        .all(|chunk| chunk == [0x28, 0x3f, 0x3a])
                    {
                        continue;
                    }
                    wrappers = trailing;
                    at = empty_start;
                }
            }
            let Some(inner_end) = atom_end.checked_sub(wrappers) else {
                continue;
            };
            let Some(inner) = body.get(at..inner_end) else {
                continue;
            };
            if wrappers == 0 || !body[inner_end..atom_end].iter().all(|&unit| unit == 0x29) {
                continue;
            }
            let (leaf, order) = if matches!(
                empty,
                EmptyBranchProof::Nested
                    | EmptyBranchProof::Sequences
                    | EmptyBranchProof::QuantifiedSequences
                    | EmptyBranchProof::Choices
            ) {
                let nested_empty = |branch: &[u16]| {
                    if matches!(
                        empty,
                        EmptyBranchProof::Sequences
                            | EmptyBranchProof::QuantifiedSequences
                            | EmptyBranchProof::Choices
                    ) {
                        return !branch.is_empty()
                            && pure_empty_groups(
                                branch,
                                matches!(
                                    empty,
                                    EmptyBranchProof::QuantifiedSequences
                                        | EmptyBranchProof::Choices
                                ),
                                empty == EmptyBranchProof::Choices,
                            );
                    }
                    let mut at = 0;
                    let mut depth = 0;
                    while branch.get(at..at + 3) == Some(&[0x28, 0x3f, 0x3a]) {
                        at += 3;
                        depth += 1;
                    }
                    depth != 0
                        && branch.len() - at == depth
                        && branch[at..].iter().all(|&unit| unit == 0x29)
                };
                // Only a completely proved empty edge supplies the separator.
                // Its parentheses are syntax; punctuation in the class payload
                // on the opposite edge cannot become an accepted delimiter.
                let edge_separator = |reverse: bool| {
                    let (open, close) = if reverse { (0x29, 0x28) } else { (0x28, 0x29) };
                    let mut depth = 0usize;
                    for offset in 0..inner.len() {
                        let at = if reverse {
                            inner.len() - offset - 1
                        } else {
                            offset
                        };
                        let unit = inner[at];
                        if unit == open {
                            depth += 1;
                        } else if unit == close {
                            depth = depth.checked_sub(1)?;
                        } else if unit == 0x7c && depth == 0 {
                            return Some(at);
                        }
                    }
                    None
                };
                let first = if empty == EmptyBranchProof::Choices {
                    edge_separator(false)
                } else {
                    inner.iter().position(|&unit| unit == 0x7c)
                };
                if let Some(separator) =
                    first.filter(|&separator| nested_empty(&inner[..separator]))
                {
                    (&inner[separator + 1..], EmptyBranchOrder::First)
                } else if let Some(separator) = (if empty == EmptyBranchProof::Choices {
                    edge_separator(true)
                } else {
                    inner.iter().rposition(|&unit| unit == 0x7c)
                })
                .filter(|&separator| nested_empty(&inner[separator + 1..]))
                {
                    (&inner[..separator], EmptyBranchOrder::Last)
                } else {
                    continue;
                }
            } else if let Some(leaf) = inner.strip_prefix(&[0x7c]) {
                (leaf, EmptyBranchOrder::First)
            } else if let Some(leaf) = inner.strip_suffix(&[0x7c]) {
                (leaf, EmptyBranchOrder::Last)
            } else {
                continue;
            };
            let order = if order == EmptyBranchOrder::First {
                if let Some(Some(optional)) = overflowing_maximum.or(unrepresentable_finite) {
                    EmptyBranchOrder::FirstFiniteOptional(optional)
                } else {
                    order
                }
            } else {
                order
            };
            // The whole remaining bare class proves q payload bars and every
            // later leaf, independently of the selected empty-branch shape.
            if leaf.first() != Some(&0x5b) || leaf.last() != Some(&0x5d) {
                continue;
            }
            let atom = match proof {
                ClassProof::Checked => RegExpUnicodeClassMatcher::compile_v_set_units_with_work(leaf, ignore_case, false, &mut charge),
                ClassProof::SingleCharacterStrings => RegExpUnicodeClassMatcher::compile_v_set_units_with_single_character_strings_and_work(leaf, ignore_case, false, &mut charge),
                ClassProof::ScalarLeftStrings => RegExpUnicodeClassMatcher::compile_scalar_left_string_operations_units_with_work(leaf, ignore_case, false, &mut charge),
                ClassProof::CharacterResultStrings => RegExpUnicodeClassMatcher::compile_character_result_v_set_units_with_work(leaf, ignore_case, false, &mut charge),
            };
            let Some(atom) = atom? else { continue };
            return Ok(Some(Self {
                atom: Atom::Class(atom),
                minimum,
                maximum,
                // An overflowing minimum leaves every consumed character
                // potentially mandatory. Preserve character-first choices
                // independently of optional-phase source greediness.
                greedy: if unrepresentable_exact
                    || (unrepresentable_finite.is_some() && order == EmptyBranchOrder::Last)
                {
                    order == EmptyBranchOrder::Last
                } else {
                    greedy
                },
                start_anchor,
                multiline,
                end_anchor,
                atom_captures: captures,
                start_word,
                end_word,
                word_ignore_case: ignore_case,
                nullable: Some(order),
            }));
        }
        Ok(None)
    }

    /// Compiles one unanchored consuming atom and quantifier after full validation.
    ///
    /// Exact decimal bounds beyond native storage are retained semantically:
    /// an impossible minimum never matches, and an unreachable maximum cannot
    /// restrict a representable input. Large counts do not allocate repetitions.
    pub fn compile_with_flags_and_work<E>(
        source: &JsString,
        unicode_sets: bool,
        ignore_case: bool,
        dot_all: bool,
        charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, E> {
        Self::compile_atom_repetition_with_work(
            source,
            unicode_sets,
            ignore_case,
            false,
            dot_all,
            AtomAssertions::None,
            charge,
        )
    }

    // Line assertions in this atom are checked at every selected iteration.
    // Only a complete repeated-group proof may request this scoped mode.
    fn compile_atom_repetition_with_work<E>(
        source: &JsString,
        unicode_sets: bool,
        ignore_case: bool,
        multiline: bool,
        dot_all: bool,
        assertions: AtomAssertions,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, E> {
        charge(source.len().saturating_mul(4))?;
        let units = source.code_units();
        // A braced Unicode escape can look like a numeric quantifier before ?.
        // Prove its retained atom completely before choosing either parse.
        for (end, greedy) in [
            (
                units
                    .len()
                    .checked_sub(1)
                    .filter(|_| units.last() == Some(&0x3f)),
                false,
            ),
            (Some(units.len()), true),
        ] {
            let Some(end) = end else { continue };
            let Some((body_end, minimum, maximum)) = quantifier(units, end) else {
                continue;
            };
            let body = &units[..body_end];
            if assertions == AtomAssertions::None
                && (body.first() == Some(&0x5e)
                    || (body.last() == Some(&0x24) && !escaped(body, body.len() - 1)))
            {
                continue;
            }
            charge(body.len())?;
            let body = JsString::from_code_units(body.to_vec());
            let atom = if let Some(m) = RegExpUnicodeCharacterMatcher::compile_with_flags_and_work(
                &body,
                ignore_case,
                multiline,
                dot_all,
                &mut charge,
            )? {
                Some(Atom::Character(m))
            } else if let Some(m) =
                RegExpUnicodeCharacterMatcher::compile_literal_with_flags_and_work(
                    &body,
                    ignore_case,
                    multiline,
                    &mut charge,
                )?
            {
                Some(Atom::Character(m))
            } else if let Some(m) = RegExpUnicodeClassMatcher::compile_with_flags_and_work(
                &body,
                unicode_sets,
                ignore_case,
                multiline,
                &mut charge,
            )? {
                Some(Atom::Class(m))
            } else {
                let atom = RegExpUnicodeClassMatcher::compile_binary_property_with_flags_and_work(
                    &body,
                    unicode_sets,
                    ignore_case,
                    multiline,
                    &mut charge,
                )?
                .map(Atom::Class);
                if atom.is_some() || assertions != AtomAssertions::Word {
                    atom
                } else if let Some(matcher) =
                    RegExpUnicodeCharacterMatcher::compile_with_word_boundaries_and_work(
                        &body,
                        ignore_case,
                        multiline,
                        dot_all,
                        &mut charge,
                    )?
                {
                    Some(Atom::Character(matcher))
                } else {
                    RegExpUnicodeClassMatcher::compile_with_word_boundaries_and_work(
                        &body,
                        unicode_sets,
                        ignore_case,
                        multiline,
                        &mut charge,
                    )?
                    .map(Atom::Class)
                }
            };
            let atom = if atom.is_none()
                && matches!(
                    assertions,
                    AtomAssertions::Alternative
                        | AtomAssertions::WrappedAlternative
                        | AtomAssertions::LineWrappedAlternative
                        | AtomAssertions::AssertedWrappedAlternative
                        | AtomAssertions::SequenceWrappedAlternative
                        | AtomAssertions::NestedSequenceWrappedAlternative
                ) {
                Self::compile_alternatives_with_work(
                    &body,
                    unicode_sets,
                    ignore_case,
                    multiline,
                    dot_all,
                    match assertions {
                        AtomAssertions::WrappedAlternative => AlternativeProof::Wrapped,
                        AtomAssertions::LineWrappedAlternative => AlternativeProof::LineWrapped,
                        AtomAssertions::AssertedWrappedAlternative => {
                            AlternativeProof::AssertedWrapped
                        }
                        AtomAssertions::SequenceWrappedAlternative => {
                            AlternativeProof::SequenceWrapped
                        }
                        AtomAssertions::NestedSequenceWrappedAlternative => {
                            AlternativeProof::NestedSequenceWrapped
                        }
                        _ => AlternativeProof::Bare,
                    },
                    &mut charge,
                )?
                .map(|branches| {
                    let work = branches
                        .iter()
                        .fold(0usize, |n, m| n.saturating_add(m.work()));
                    Atom::Alternatives {
                        branches,
                        work,
                        guard: None,
                    }
                })
            } else {
                atom
            };
            if let Some(atom) = atom {
                return Ok(Some(Self {
                    atom,
                    minimum,
                    maximum,
                    greedy,
                    start_anchor: false,
                    multiline: false,
                    end_anchor: false,
                    atom_captures: 0,
                    start_word: None,
                    end_word: None,
                    word_ignore_case: false,
                    nullable: None,
                }));
            }
        }
        Ok(None)
    }

    /// Compiles one consuming atom and quantifier with an optional leading ^.
    ///
    /// The caller validates the whole u/v Pattern with the supplied flags.
    /// Only the actual beginning or a position after a LineTerminator may pass
    /// the assertion (22.2.2.4). Trailing assertions remain separate proofs.
    pub fn compile_with_start_anchor_and_work<E>(
        source: &JsString,
        unicode_sets: bool,
        ignore_case: bool,
        multiline: bool,
        dot_all: bool,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, E> {
        if source.code_units().first() != Some(&0x5e) {
            return Self::compile_with_flags_and_work(
                source,
                unicode_sets,
                ignore_case,
                dot_all,
                charge,
            );
        }
        charge(source.len())?;
        let body = JsString::from_code_units(source.code_units()[1..].to_vec());
        let Some(mut plan) = Self::compile_with_flags_and_work(
            &body,
            unicode_sets,
            ignore_case,
            dot_all,
            &mut charge,
        )?
        else {
            return Ok(None);
        };
        plan.start_anchor = true;
        plan.multiline = multiline;
        Ok(Some(plan))
    }

    /// Compiles one consuming atom and quantifier with proved line assertions.
    ///
    /// A leading ^ retains its multiline contexts. A trailing $ requires the
    /// actual input end or, with m, a position before a LineTerminator (22.2.2.4).
    /// The caller validates the complete u/v Pattern with these flags first.
    pub fn compile_with_line_anchors_and_work<E>(
        source: &JsString,
        unicode_sets: bool,
        ignore_case: bool,
        multiline: bool,
        dot_all: bool,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, E> {
        let units = source.code_units();
        let end_anchor = units.last() == Some(&0x24) && !escaped(units, units.len() - 1);
        if !end_anchor {
            return Self::compile_with_start_anchor_and_work(
                source,
                unicode_sets,
                ignore_case,
                multiline,
                dot_all,
                charge,
            );
        }
        charge(units.len())?;
        let body = JsString::from_code_units(units[..units.len() - 1].to_vec());
        let Some(mut plan) = Self::compile_with_start_anchor_and_work(
            &body,
            unicode_sets,
            ignore_case,
            multiline,
            dot_all,
            &mut charge,
        )?
        else {
            return Ok(None);
        };
        plan.end_anchor = true;
        plan.multiline = multiline;
        Ok(Some(plan))
    }

    /// Proves complete ordinary/noncapturing wrappers inside the repeated atom.
    ///
    /// The caller validates the whole u/v Pattern and normalizes named groups.
    /// Every retained inner body must be one consuming atom. Assertions inside
    /// these wrappers, concatenations, alternatives and empty bodies are excluded.
    /// Optional outer line assertions retain their original scopes (22.2.2.7).
    pub fn compile_capturing_atom_with_work<E>(
        source: &JsString,
        unicode_sets: bool,
        ignore_case: bool,
        multiline: bool,
        dot_all: bool,
        charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, E> {
        Self::compile_wrapped_atom_with_work(
            source,
            unicode_sets,
            ignore_case,
            multiline,
            dot_all,
            AtomAssertions::None,
            charge,
        )
    }

    /// Proves optional line assertions inside complete repeated atom groups.
    ///
    /// Requires complete u/v validation and normalized named groups. Each
    /// selected consuming iteration checks its own ^/$ against original input
    /// context. Complete wrappers capture the final iteration; optional outside
    /// line assertions retain their distinct scopes. Empty units, word assertions,
    /// alternatives and concatenations remain excluded.
    pub fn compile_capturing_atom_with_line_assertions_and_work<E>(
        source: &JsString,
        unicode_sets: bool,
        ignore_case: bool,
        multiline: bool,
        dot_all: bool,
        charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, E> {
        Self::compile_wrapped_atom_with_work(
            source,
            unicode_sets,
            ignore_case,
            multiline,
            dot_all,
            AtomAssertions::Line,
            charge,
        )
    }

    /// Proves word and line assertions inside complete repeated atom groups.
    ///
    /// Requires complete u/v validation and normalized named groups. Each
    /// consuming iteration checks its retained assertions against the original
    /// input context. Optional outside line assertions keep their own scopes.
    /// Empty units, alternatives, concatenations and outside word assertions
    /// remain excluded. Complete inner captures retain the final iteration.
    pub fn compile_capturing_atom_with_assertions_and_work<E>(
        source: &JsString,
        unicode_sets: bool,
        ignore_case: bool,
        multiline: bool,
        dot_all: bool,
        charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, E> {
        Self::compile_wrapped_atom_with_work(
            source,
            unicode_sets,
            ignore_case,
            multiline,
            dot_all,
            AtomAssertions::Word,
            charge,
        )
    }

    /// Proves complete repeated wrappers around one-character alternatives.
    ///
    /// Requires full u/v validation and normalized named groups. Every branch
    /// must consume exactly one Unicode character, with optional branch-scoped
    /// line and word assertions. Empty branches, concatenations and captures
    /// inside branches are excluded. All wrappers capture the final iteration.
    /// Storage is bounded by source length; work includes every branch predicate.
    pub fn compile_capturing_alternatives_with_work<E>(
        source: &JsString,
        unicode_sets: bool,
        ignore_case: bool,
        multiline: bool,
        dot_all: bool,
        charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, E> {
        Self::compile_wrapped_atom_with_work(
            source,
            unicode_sets,
            ignore_case,
            multiline,
            dot_all,
            AtomAssertions::Alternative,
            charge,
        )
    }

    /// Composes outside word assertions with complete one-character alternatives.
    ///
    /// Requires full u/v validation and normalized named groups. The retained
    /// repetition must pass the complete alternative proof before attaching any
    /// outside predicate. Branch word/line assertions keep their original scopes.
    pub fn compile_alternatives_with_word_boundaries_and_work<E>(
        source: &JsString,
        unicode_sets: bool,
        ignore_case: bool,
        multiline: bool,
        dot_all: bool,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, E> {
        let Some(assertions) = unicode_word_assertions_with_work(source, &mut charge)? else {
            return Ok(None);
        };
        let Some(mut plan) = Self::compile_capturing_alternatives_with_work(
            &assertions.source,
            unicode_sets,
            ignore_case,
            multiline,
            dot_all,
            charge,
        )?
        else {
            return Ok(None);
        };
        plan.start_word = assertions.start;
        plan.end_word = assertions.end;
        plan.word_ignore_case = ignore_case;
        Ok(Some(plan))
    }

    /// Proves single-character alternatives with complete noncapturing branches.
    ///
    /// Requires full u/v validation and normalized named groups. Every branch
    /// consumes one unquantified character; assertions inside its noncapturing
    /// wrappers retain their scope. Branch captures, nullable branches, nested
    /// alternatives and concatenations remain excluded. Enclosing captures keep
    /// the final selected character. Construction uses source-bounded storage
    /// and adds no native recursion or input-sized search storage.
    pub fn compile_noncapturing_branches_with_work<E>(
        source: &JsString,
        unicode_sets: bool,
        ignore_case: bool,
        multiline: bool,
        dot_all: bool,
        charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, E> {
        Self::compile_wrapped_atom_with_work(
            source,
            unicode_sets,
            ignore_case,
            multiline,
            dot_all,
            AtomAssertions::WrappedAlternative,
            charge,
        )
    }

    /// Composes outside word assertions with complete noncapturing branches.
    ///
    /// Requires full u/v validation and normalized named groups. The complete
    /// retained body must pass the noncapturing one-character branch proof
    /// before attaching any whole-range word gate. Branch predicates and capture
    /// effects retain their original scopes.
    pub fn compile_noncapturing_branch_words_with_work<E>(
        source: &JsString,
        unicode_sets: bool,
        ignore_case: bool,
        multiline: bool,
        dot_all: bool,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, E> {
        let Some(assertions) = unicode_word_assertions_with_work(source, &mut charge)? else {
            return Ok(None);
        };
        let Some(mut plan) = Self::compile_noncapturing_branches_with_work(
            &assertions.source,
            unicode_sets,
            ignore_case,
            multiline,
            dot_all,
            charge,
        )?
        else {
            return Ok(None);
        };
        plan.start_word = assertions.start;
        plan.end_word = assertions.end;
        plan.word_ignore_case = ignore_case;
        Ok(Some(plan))
    }

    /// Proves line assertions outside complete noncapturing character branches.
    ///
    /// Requires full u/v validation and normalized named groups. Each branch
    /// must consume exactly one unquantified character and contain no captures.
    /// Its proved leading/trailing line gates become per-branch predicates on
    /// that character, separate from whole-repetition assertions. Branch word
    /// gates outside wrappers and nested alternatives remain excluded.
    pub fn compile_noncapturing_branch_lines_with_work<E>(
        source: &JsString,
        unicode_sets: bool,
        ignore_case: bool,
        multiline: bool,
        dot_all: bool,
        charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, E> {
        Self::compile_wrapped_atom_with_work(
            source,
            unicode_sets,
            ignore_case,
            multiline,
            dot_all,
            AtomAssertions::LineWrappedAlternative,
            charge,
        )
    }

    /// Composes outside words with proved noncapturing branch line assertions.
    ///
    /// Requires full u/v validation and normalized named groups. The complete
    /// retained repetition must pass the branch line proof before whole-range
    /// word predicates are attached. Branch and repetition assertions retain
    /// distinct scopes and enclosing groups retain their final captures.
    pub fn compile_noncapturing_branch_line_words_with_work<E>(
        source: &JsString,
        unicode_sets: bool,
        ignore_case: bool,
        multiline: bool,
        dot_all: bool,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, E> {
        let Some(assertions) = unicode_word_assertions_with_work(source, &mut charge)? else {
            return Ok(None);
        };
        let Some(mut plan) = Self::compile_noncapturing_branch_lines_with_work(
            &assertions.source,
            unicode_sets,
            ignore_case,
            multiline,
            dot_all,
            charge,
        )?
        else {
            return Ok(None);
        };
        plan.start_word = assertions.start;
        plan.end_word = assertions.end;
        plan.word_ignore_case = ignore_case;
        Ok(Some(plan))
    }

    /// Proves line and word gates around complete noncapturing character branches.
    ///
    /// Requires full u/v validation and normalized named groups. Every branch
    /// must completely prove one unquantified character with no captures. Its
    /// outside predicates constrain that character's original boundaries, while
    /// whole-repetition gates remain separate. No recursive branch representation
    /// or input-sized search storage is added.
    pub fn compile_noncapturing_branch_assertions_with_work<E>(
        source: &JsString,
        unicode_sets: bool,
        ignore_case: bool,
        multiline: bool,
        dot_all: bool,
        charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, E> {
        Self::compile_wrapped_atom_with_work(
            source,
            unicode_sets,
            ignore_case,
            multiline,
            dot_all,
            AtomAssertions::AssertedWrappedAlternative,
            charge,
        )
    }

    /// Composes whole-repetition word gates with complete asserted branches.
    ///
    /// Requires full u/v validation and normalized named groups. The retained
    /// body must pass the complete noncapturing-branch assertion proof before
    /// whole-match predicates are attached. Each branch keeps its own scope.
    pub fn compile_noncapturing_branch_scoped_assertions_with_work<E>(
        source: &JsString,
        unicode_sets: bool,
        ignore_case: bool,
        multiline: bool,
        dot_all: bool,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, E> {
        let Some(assertions) = unicode_word_assertions_with_work(source, &mut charge)? else {
            return Ok(None);
        };
        let Some(mut plan) = Self::compile_noncapturing_branch_assertions_with_work(
            &assertions.source,
            unicode_sets,
            ignore_case,
            multiline,
            dot_all,
            charge,
        )?
        else {
            return Ok(None);
        };
        plan.start_word = assertions.start;
        plan.end_word = assertions.end;
        plan.word_ignore_case = ignore_case;
        Ok(Some(plan))
    }

    /// Composes whole-repetition word gates with complete branch assertion sequences.
    ///
    /// Requires full u/v validation and normalized named groups. The retained
    /// body must pass the complete noncapturing-branch assertion-sequence proof before
    /// whole-match predicates are attached. Each branch keeps its own scope.
    pub fn compile_noncapturing_branch_sequence_words_with_work<E>(
        source: &JsString,
        unicode_sets: bool,
        ignore_case: bool,
        multiline: bool,
        dot_all: bool,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, E> {
        let Some(assertions) = unicode_word_assertions_with_work(source, &mut charge)? else {
            return Ok(None);
        };
        let Some(mut plan) = Self::compile_noncapturing_branch_assertion_sequences_with_work(
            &assertions.source,
            unicode_sets,
            ignore_case,
            multiline,
            dot_all,
            charge,
        )?
        else {
            return Ok(None);
        };
        plan.start_word = assertions.start;
        plan.end_word = assertions.end;
        plan.word_ignore_case = ignore_case;
        Ok(Some(plan))
    }

    /// Proves exterior assertion sequences on complete Unicode character branches.
    ///
    /// Requires full u/v validation and normalized named groups. Identical word
    /// and line predicates at the same branch boundary are combined. Conflicting
    /// word predicates make that branch impossible, after its complete retained
    /// body proves one unquantified character without captures. Whole-repetition
    /// assertions and predicates inside the branch retain their original scopes.
    pub fn compile_noncapturing_branch_assertion_sequences_with_work<E>(
        source: &JsString,
        unicode_sets: bool,
        ignore_case: bool,
        multiline: bool,
        dot_all: bool,
        charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, E> {
        Self::compile_wrapped_atom_with_work(
            source,
            unicode_sets,
            ignore_case,
            multiline,
            dot_all,
            AtomAssertions::SequenceWrappedAlternative,
            charge,
        )
    }

    /// Composes whole-repetition word gates with complete nested branch assertions.
    ///
    /// Requires full u/v validation and normalized named groups. The retained
    /// body must pass the complete nested noncapturing-branch assertion proof before
    /// whole-match predicates are attached. Each branch keeps its own scope.
    pub fn compile_noncapturing_branch_nested_assertion_words_with_work<E>(
        source: &JsString,
        unicode_sets: bool,
        ignore_case: bool,
        multiline: bool,
        dot_all: bool,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, E> {
        let Some(assertions) = unicode_word_assertions_with_work(source, &mut charge)? else {
            return Ok(None);
        };
        let Some(mut plan) = Self::compile_noncapturing_branch_nested_assertions_with_work(
            &assertions.source,
            unicode_sets,
            ignore_case,
            multiline,
            dot_all,
            charge,
        )?
        else {
            return Ok(None);
        };
        plan.start_word = assertions.start;
        plan.end_word = assertions.end;
        plan.word_ignore_case = ignore_case;
        Ok(Some(plan))
    }

    /// Proves complete noncapturing branch wrappers interleaved with assertions.
    ///
    /// Requires full u/v validation and normalized named groups. Pure word/line
    /// predicates at each original boundary can be combined only after peeling
    /// balanced noncapturing wrappers and proving the entire remaining body to
    /// be one unquantified character without captures. Conflicts make the branch
    /// impossible; whole-repetition scopes and earlier contracts are unchanged.
    pub fn compile_noncapturing_branch_nested_assertions_with_work<E>(
        source: &JsString,
        unicode_sets: bool,
        ignore_case: bool,
        multiline: bool,
        dot_all: bool,
        charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, E> {
        Self::compile_wrapped_atom_with_work(
            source,
            unicode_sets,
            ignore_case,
            multiline,
            dot_all,
            AtomAssertions::NestedSequenceWrappedAlternative,
            charge,
        )
    }

    /// Proves exactly one raw empty branch and one complete character branch.
    ///
    /// Requires full u/v validation and normalized named groups. Only *, + and
    /// ? are admitted, including their lazy forms. Branch captures, guarded
    /// empty branches, extra alternatives and braced quantifiers remain outside
    /// this proof. RepeatMatcher rejects empty optional iterations (22.2.2.3),
    /// while a mandatory empty iteration preserves defined empty captures.
    pub fn compile_nullable_alternatives_with_work<E>(
        source: &JsString,
        unicode_sets: bool,
        ignore_case: bool,
        multiline: bool,
        dot_all: bool,
        charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, E> {
        Self::compile_nullable_alternatives_impl_with_work(
            source,
            unicode_sets,
            ignore_case,
            multiline,
            dot_all,
            NullableProof::Raw.into(),
            charge,
        )
    }

    /// Proves empty alternatives enclosed by complete noncapturing wrappers.
    ///
    /// Requires full u/v validation and normalized named groups. Empty branches
    /// may be raw or contain only balanced noncapturing wrappers; captures,
    /// predicates, choices and quantifiers inside them remain excluded. The
    /// character branch and repetition retain the complete nullable proof.
    pub fn compile_noncapturing_empty_alternatives_with_work<E>(
        source: &JsString,
        unicode_sets: bool,
        ignore_case: bool,
        multiline: bool,
        dot_all: bool,
        charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, E> {
        Self::compile_nullable_alternatives_impl_with_work(
            source,
            unicode_sets,
            ignore_case,
            multiline,
            dot_all,
            NullableProof::TransparentEmpty.into(),
            charge,
        )
    }

    /// Extends the complete nullable proof to braced counts with a zero minimum.
    ///
    /// Requires full u/v validation and normalized named groups. Every iteration
    /// is optional, so RepeatMatcher rejects each zero-progress alternative
    /// (22.2.2.3). The proved character branch retains the exact finite/unbounded
    /// upper limit and lazy/greedy order. Positive-minimum braces remain excluded.
    pub fn compile_zero_minimum_nullable_alternatives_with_work<E>(
        source: &JsString,
        unicode_sets: bool,
        ignore_case: bool,
        multiline: bool,
        dot_all: bool,
        charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, E> {
        Self::compile_nullable_alternatives_impl_with_work(
            source,
            unicode_sets,
            ignore_case,
            multiline,
            dot_all,
            NullableProof::ZeroMinimum.into(),
            charge,
        )
    }

    /// Proves open-ended nullable counts with exactly one mandatory iteration.
    ///
    /// Requires full u/v validation and normalized named groups. The complete
    /// two-branch proof admits {1,}, including padded spellings and lazy forms.
    /// This has exactly the RepeatMatcher bounds of + (22.2.2.3), including
    /// defined mandatory empty captures and lazy source-order preference.
    /// Other positive lower bounds and finite positive-minimum ranges remain
    /// excluded. Earlier braced-count entry points keep their narrow contracts.
    pub fn compile_one_minimum_open_nullable_alternatives_with_work<E>(
        source: &JsString,
        unicode_sets: bool,
        ignore_case: bool,
        multiline: bool,
        dot_all: bool,
        charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, E> {
        Self::compile_nullable_alternatives_impl_with_work(
            source,
            unicode_sets,
            ignore_case,
            multiline,
            dot_all,
            NullableProof::OneMinimumOpen.into(),
            charge,
        )
    }

    /// Proves exactly one mandatory nullable iteration, retaining branch order.
    ///
    /// Requires full u/v validation and normalized named groups. {1}, {1,1},
    /// padded and lazy spellings retain exactly one selected branch (22.2.2.3).
    /// Source order chooses empty or character paths at the same candidate;
    /// outside predicates can reject either path before the next is tried.
    pub fn compile_one_iteration_nullable_alternatives_with_work<E>(
        source: &JsString,
        unicode_sets: bool,
        ignore_case: bool,
        multiline: bool,
        dot_all: bool,
        charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, E> {
        Self::compile_nullable_alternatives_impl_with_work(
            source,
            unicode_sets,
            ignore_case,
            multiline,
            dot_all,
            NullableProof::OneIteration.into(),
            charge,
        )
    }

    /// Proves an explicitly open nullable count with a positive lower bound.
    ///
    /// Requires validated u/v syntax and normalized names. Mandatory empty
    /// iterations may satisfy arbitrarily large source counts without expanding
    /// them. Branch order preserves RepeatMatcher selection and final captures
    /// (22.2.2.3); finite upper bounds remain outside this proof.
    pub fn compile_positive_open_nullable_alternatives_with_work<E>(
        source: &JsString,
        unicode_sets: bool,
        ignore_case: bool,
        multiline: bool,
        dot_all: bool,
        charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, E> {
        Self::compile_nullable_alternatives_impl_with_work(
            source,
            unicode_sets,
            ignore_case,
            multiline,
            dot_all,
            NullableProof::PositiveOpen.into(),
            charge,
        )
    }

    /// Proves a representable exact nullable count greater than one.
    ///
    /// Requires validated u/v syntax and normalized names. With only mandatory
    /// iterations, source branch order selects endpoints independently of the
    /// quantifier's greedy/lazy spelling (22.2.2.3). Counts are not expanded.
    /// Unequal finite bounds and unrepresentable exact counts remain excluded.
    pub fn compile_positive_exact_nullable_alternatives_with_work<E>(
        source: &JsString,
        unicode_sets: bool,
        ignore_case: bool,
        multiline: bool,
        dot_all: bool,
        charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, E> {
        Self::compile_nullable_alternatives_impl_with_work(
            source,
            unicode_sets,
            ignore_case,
            multiline,
            dot_all,
            NullableProof::PositiveExact.into(),
            charge,
        )
    }

    /// Proves unequal finite nullable bounds with a positive minimum.
    ///
    /// Requires validated u/v syntax and normalized names. Both bounds must fit
    /// usize. Mandatory empty iterations and the optional iteration budget
    /// preserve RepeatMatcher source order without expanding counts (22.2.2.3).
    pub fn compile_positive_finite_nullable_alternatives_with_work<E>(
        source: &JsString,
        unicode_sets: bool,
        ignore_case: bool,
        multiline: bool,
        dot_all: bool,
        charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, E> {
        Self::compile_nullable_alternatives_impl_with_work(
            source,
            unicode_sets,
            ignore_case,
            multiline,
            dot_all,
            NullableProof::PositiveFinite.into(),
            charge,
        )
    }

    /// Proves an exact nullable count too large for usize.
    ///
    /// Requires validated u/v syntax and normalized names. Decimal source
    /// equality proves single/equal-range spellings without a big integer.
    /// Every finite input has fewer characters than the mandatory count, so
    /// effective branch priority reuses the existing search/capture kernels
    /// without expanding mandatory empty iterations (22.2.2.3).
    pub fn compile_unrepresentable_exact_nullable_alternatives_with_work<E>(
        source: &JsString,
        unicode_sets: bool,
        ignore_case: bool,
        multiline: bool,
        dot_all: bool,
        charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, E> {
        Self::compile_nullable_alternatives_impl_with_work(
            source,
            unicode_sets,
            ignore_case,
            multiline,
            dot_all,
            NullableProof::UnrepresentableExact.into(),
            charge,
        )
    }

    /// Proves unequal finite nullable bounds both larger than usize.
    ///
    /// Requires validated u/v syntax and normalized names. Decimal subtraction
    /// retains a representable optional count without expanding mandatory empty
    /// iterations. An overflowing difference exceeds any finite input; source
    /// branch order still governs mandatory choices (22.2.2.3).
    pub fn compile_unrepresentable_finite_nullable_alternatives_with_work<E>(
        source: &JsString,
        unicode_sets: bool,
        ignore_case: bool,
        multiline: bool,
        dot_all: bool,
        charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, E> {
        Self::compile_nullable_alternatives_impl_with_work(
            source,
            unicode_sets,
            ignore_case,
            multiline,
            dot_all,
            NullableProof::UnrepresentableFinite.into(),
            charge,
        )
    }

    /// Proves a represented positive minimum with an overflowing finite maximum.
    ///
    /// Requires validated u/v syntax and normalized names. The finite upper
    /// bound cannot restrict native input lengths, but its exact optional count
    /// can still constrain empty-first greedy choices when the lower bound is
    /// close to usize::MAX. Existing mandatory capture priorities are retained
    /// without expanding the source counts (22.2.2.3).
    pub fn compile_overflowing_maximum_nullable_alternatives_with_work<E>(
        source: &JsString,
        unicode_sets: bool,
        ignore_case: bool,
        multiline: bool,
        dot_all: bool,
        charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, E> {
        Self::compile_nullable_alternatives_impl_with_work(
            source,
            unicode_sets,
            ignore_case,
            multiline,
            dot_all,
            NullableProof::OverflowingMaximum.into(),
            charge,
        )
    }

    /// Proves nullable alternatives with complete noncapturing empty sequences.
    ///
    /// Requires validated u/v syntax and normalized names. Every empty-branch
    /// token must belong to a balanced, unquantified noncapturing group. These
    /// groups contain no assertions, captures or consuming/effectful operations.
    /// Reuses the existing count, branch-priority and capture proofs (22.2.2.3).
    pub fn compile_noncapturing_empty_sequence_nullable_alternatives_with_work<E>(
        source: &JsString,
        unicode_sets: bool,
        ignore_case: bool,
        multiline: bool,
        dot_all: bool,
        charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, E> {
        Self::compile_nullable_empty_branches_with_work(
            source,
            unicode_sets,
            ignore_case,
            multiline,
            dot_all,
            EmptyBranchProof::Sequences,
            charge,
        )
    }

    /// Proves nullable alternatives with quantified noncapturing empty groups.
    ///
    /// Requires validated u/v syntax and normalized names. Each empty-branch
    /// group contains only other effect-free empty groups. Its quantifier may
    /// select any validated finite/open count, including unrepresentable bounds.
    /// All paths succeed at the same boundary without captures or assertions
    /// (22.2.2.3); established outer count and capture priority remain unchanged.
    pub fn compile_quantified_empty_sequence_nullable_alternatives_with_work<E>(
        source: &JsString,
        unicode_sets: bool,
        ignore_case: bool,
        multiline: bool,
        dot_all: bool,
        charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, E> {
        Self::compile_nullable_empty_branches_with_work(
            source,
            unicode_sets,
            ignore_case,
            multiline,
            dot_all,
            EmptyBranchProof::QuantifiedSequences,
            charge,
        )
    }

    /// Proves nullable alternatives with complete effect-free empty choices.
    ///
    /// Requires validated u/v syntax and normalized names. Every branch of each
    /// noncapturing choice contains only other pure empty groups and validated
    /// quantifiers. All paths succeed at the same boundary without captures,
    /// assertions or consuming operations (22.2.2.3); their redundant choices
    /// preserve the established outer repetition and capture priority.
    pub fn compile_empty_choice_nullable_alternatives_with_work<E>(
        source: &JsString,
        unicode_sets: bool,
        ignore_case: bool,
        multiline: bool,
        dot_all: bool,
        charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, E> {
        Self::compile_nullable_empty_branches_with_work(
            source,
            unicode_sets,
            ignore_case,
            multiline,
            dot_all,
            EmptyBranchProof::Choices,
            charge,
        )
    }

    fn compile_nullable_empty_branches_with_work<E>(
        source: &JsString,
        unicode_sets: bool,
        ignore_case: bool,
        multiline: bool,
        dot_all: bool,
        empty: EmptyBranchProof,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, E> {
        for bounds in [
            NullableProof::Raw,
            NullableProof::TransparentEmpty,
            NullableProof::ZeroMinimum,
            NullableProof::OneMinimumOpen,
            NullableProof::OneIteration,
            NullableProof::PositiveOpen,
            NullableProof::PositiveExact,
            NullableProof::PositiveFinite,
            NullableProof::UnrepresentableExact,
            NullableProof::UnrepresentableFinite,
            NullableProof::OverflowingMaximum,
        ] {
            if let Some(plan) = Self::compile_nullable_alternatives_impl_with_work(
                source,
                unicode_sets,
                ignore_case,
                multiline,
                dot_all,
                NullableOptions { bounds, empty },
                &mut charge,
            )? {
                return Ok(Some(plan));
            }
        }
        Ok(None)
    }

    fn compile_nullable_alternatives_impl_with_work<E>(
        source: &JsString,
        unicode_sets: bool,
        ignore_case: bool,
        multiline: bool,
        dot_all: bool,
        options: NullableOptions,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, E> {
        let proof = options.bounds;
        let assertions = unicode_word_assertions_with_work(source, &mut charge)?;
        let retained = assertions.as_ref().map_or(source, |a| &a.source);
        let units = retained.code_units();
        charge(units.len().saturating_mul(4))?;
        let start_anchor = units.first() == Some(&0x5e);
        let end_anchor = units.last() == Some(&0x24) && !escaped(units, units.len() - 1);
        let start = usize::from(start_anchor);
        let end = units.len().saturating_sub(usize::from(end_anchor));
        let Some(body) = units.get(start..end) else {
            return Ok(None);
        };
        let mut ends = vec![usize::MAX; body.len()];
        let mut stack = Vec::new();
        let mut classes = 0usize;
        let mut at = 0;
        while at < body.len() {
            match body[at] {
                0x5c => {
                    at += 2;
                    continue;
                }
                0x5b if classes == 0 || unicode_sets => {
                    classes += 1;
                }
                0x5d if classes != 0 => {
                    classes -= 1;
                }
                0x28 if classes == 0 => {
                    stack.push(at);
                }
                0x29 if classes == 0 => {
                    let Some(open) = stack.pop() else {
                        return Ok(None);
                    };
                    ends[open] = at;
                }
                _ => {}
            }
            at += 1;
        }
        if !stack.is_empty() || classes != 0 {
            return Ok(None);
        }
        for end in [
            body.len()
                .checked_sub(1)
                .filter(|_| body.last() == Some(&0x3f)),
            Some(body.len()),
        ] {
            let Some(end) = end else {
                continue;
            };
            let Some((atom_end, minimum, maximum)) = quantifier(body, end) else {
                continue;
            };
            let simple = atom_end.checked_add(1) == Some(end)
                && matches!(body[atom_end], 0x2a | 0x2b | 0x3f);
            let zero = proof == NullableProof::ZeroMinimum && minimum == Some(0);
            let one_open = proof == NullableProof::OneMinimumOpen
                && minimum == Some(1)
                && end.checked_sub(2).and_then(|at| body.get(at)) == Some(&0x2c)
                && body.get(end - 1) == Some(&0x7d);
            let one =
                proof == NullableProof::OneIteration && minimum == Some(1) && maximum == Some(1);
            let positive_open = proof == NullableProof::PositiveOpen
                && minimum != Some(0)
                && end.checked_sub(2).and_then(|at| body.get(at)) == Some(&0x2c)
                && body.get(end - 1) == Some(&0x7d);
            let positive_exact = proof == NullableProof::PositiveExact
                && minimum.is_some_and(|minimum| minimum > 1)
                && minimum == maximum;
            let positive_finite = proof == NullableProof::PositiveFinite
                && matches!((minimum, maximum), (Some(minimum), Some(maximum)) if minimum > 0 && maximum > minimum);
            let unrepresentable_exact = if proof == NullableProof::UnrepresentableExact
                && minimum.is_none()
                && maximum.is_none()
            {
                charge((end - atom_end).saturating_mul(3))?;
                let digits = &body[atom_end + 1..end - 1];
                if let Some(comma) = digits.iter().position(|&unit| unit == 0x2c) {
                    let first = &digits[..comma];
                    let last = &digits[comma + 1..];
                    !last.is_empty() && significant(first) == significant(last)
                } else {
                    true
                }
            } else {
                false
            };
            let unrepresentable_finite = if proof == NullableProof::UnrepresentableFinite
                && minimum.is_none()
                && maximum.is_none()
            {
                charge((end - atom_end).saturating_mul(5))?;
                let digits = &body[atom_end + 1..end - 1];
                digits
                    .iter()
                    .position(|&unit| unit == 0x2c)
                    .and_then(|comma| {
                        let first = significant(&digits[..comma]);
                        let last = significant(&digits[comma + 1..]);
                        (!last.is_empty() && first != last).then(|| decimal_difference(first, last))
                    })
            } else {
                None
            };
            let overflowing_maximum = if proof == NullableProof::OverflowingMaximum
                && minimum.is_some_and(|minimum| minimum > 0)
                && maximum.is_none()
                && body.get(atom_end) == Some(&0x7b)
            {
                charge((end - atom_end).saturating_mul(5))?;
                let digits = &body[atom_end + 1..end - 1];
                digits
                    .iter()
                    .position(|&unit| unit == 0x2c)
                    .and_then(|comma| {
                        let first = significant(&digits[..comma]);
                        let last = significant(&digits[comma + 1..]);
                        (!last.is_empty()).then(|| decimal_difference(first, last))
                    })
            } else {
                None
            };
            if !simple
                && !zero
                && !one_open
                && !one
                && !positive_open
                && !positive_exact
                && !positive_finite
                && !unrepresentable_exact
                && unrepresentable_finite.is_none()
                && overflowing_maximum.is_none()
            {
                continue;
            }
            let mut at = 0;
            let mut wrappers = 0;
            let mut captures = 0;
            while body.get(at) == Some(&0x28)
                && atom_end.checked_sub(wrappers + 1) == Some(ends[at])
            {
                if body.get(at + 1) == Some(&0x3f) {
                    if body.get(at + 2) != Some(&0x3a) {
                        break;
                    }
                    at += 3;
                } else {
                    at += 1;
                    captures += 1;
                }
                wrappers += 1;
            }
            let Some(inner_end) = atom_end.checked_sub(wrappers) else {
                continue;
            };
            if wrappers == 0
                || at >= inner_end
                || !body[inner_end..atom_end].iter().all(|&c| c == 0x29)
            {
                continue;
            }
            let inner = &body[at..inner_end];
            let mut separator = None;
            let mut depth = 0usize;
            let mut classes = 0usize;
            let mut cursor = 0;
            let mut extra = false;
            while cursor < inner.len() {
                match inner[cursor] {
                    0x5c => {
                        cursor += 2;
                        continue;
                    }
                    0x5b if classes == 0 || unicode_sets => {
                        classes += 1;
                    }
                    0x5d if classes != 0 => {
                        classes -= 1;
                    }
                    0x28 if classes == 0 => {
                        depth += 1;
                    }
                    0x29 if classes == 0 => {
                        let Some(previous) = depth.checked_sub(1) else {
                            return Ok(None);
                        };
                        depth = previous;
                    }
                    0x7c if classes == 0 && depth == 0 => {
                        extra |= separator.replace(cursor).is_some();
                    }
                    _ => {}
                }
                cursor += 1;
            }
            let Some(separator) = separator else {
                continue;
            };
            if extra || depth != 0 || classes != 0 {
                continue;
            }
            let first = &inner[..separator];
            let last = &inner[separator + 1..];
            if options.empty != EmptyBranchProof::Raw {
                let weight = if matches!(
                    options.empty,
                    EmptyBranchProof::QuantifiedSequences | EmptyBranchProof::Choices
                ) {
                    4
                } else {
                    2
                };
                charge(inner.len().saturating_mul(weight))?;
            }
            let empty_branch = |branch: &[u16]| {
                if branch.is_empty() {
                    return true;
                }
                if options.empty == EmptyBranchProof::Raw {
                    return false;
                }
                if matches!(
                    options.empty,
                    EmptyBranchProof::Sequences
                        | EmptyBranchProof::QuantifiedSequences
                        | EmptyBranchProof::Choices
                ) {
                    return pure_empty_groups(
                        branch,
                        matches!(
                            options.empty,
                            EmptyBranchProof::QuantifiedSequences | EmptyBranchProof::Choices
                        ),
                        options.empty == EmptyBranchProof::Choices,
                    );
                }
                let mut at = 0;
                let mut wrappers = 0;
                while branch.get(at..at + 3) == Some(&[0x28, 0x3f, 0x3a]) {
                    at += 3;
                    wrappers += 1;
                }
                wrappers != 0
                    && branch.len().checked_sub(at) == Some(wrappers)
                    && branch[at..].iter().all(|&c| c == 0x29)
            };
            let (rune, order) = if empty_branch(first) && !empty_branch(last) {
                (last, EmptyBranchOrder::First)
            } else if empty_branch(last) && !empty_branch(first) {
                (first, EmptyBranchOrder::Last)
            } else {
                continue;
            };
            // Duplicate the pure consuming branch to reuse the complete branch
            // proof, including assertions and transparent noncapturing wrappers.
            // Neither duplicate can contribute captures or effects. This is a
            // source-sized construction; quantified input paths are not expanded.
            charge(rune.len().saturating_mul(2).saturating_add(5))?;
            let mut proved_source =
                Vec::with_capacity(rune.len().saturating_mul(2).saturating_add(5));
            proved_source.push(0x28);
            proved_source.extend_from_slice(rune);
            proved_source.push(0x7c);
            proved_source.extend_from_slice(rune);
            proved_source.push(0x29);
            proved_source.extend_from_slice(&body[atom_end..]);
            let Some(mut plan) = Self::compile_noncapturing_branch_nested_assertions_with_work(
                &JsString::from_code_units(proved_source),
                unicode_sets,
                ignore_case,
                multiline,
                dot_all,
                &mut charge,
            )?
            else {
                continue;
            };
            plan.atom_captures = captures;
            plan.nullable = Some(order);
            if unrepresentable_exact {
                // The exact mandatory count exceeds every finite character
                // count. Character-first chooses descending endpoints and
                // empty-first ascending endpoints, independently of laziness.
                // Original source/flags remain in the constructor metadata.
                plan.greedy = order == EmptyBranchOrder::Last;
            }
            if let Some(optional) = unrepresentable_finite {
                if order == EmptyBranchOrder::Last {
                    // All finite character counts remain mandatory, so lazy
                    // spelling cannot reorder character-first choices.
                    plan.greedy = true;
                } else if let Some(optional) = optional {
                    plan.nullable = Some(EmptyBranchOrder::FirstFiniteOptional(optional));
                }
            }
            if order == EmptyBranchOrder::First {
                if let Some(Some(optional)) = overflowing_maximum {
                    plan.nullable = Some(EmptyBranchOrder::FirstFiniteOptional(optional));
                }
            }
            plan.start_anchor = start_anchor;
            plan.end_anchor = end_anchor;
            plan.multiline = multiline;
            if let Some(assertions) = &assertions {
                plan.start_word = assertions.start;
                plan.end_word = assertions.end;
                plan.word_ignore_case = ignore_case;
            }
            return Ok(Some(plan));
        }
        Ok(None)
    }

    fn compile_alternatives_with_work<E>(
        source: &JsString,
        unicode_sets: bool,
        ignore_case: bool,
        multiline: bool,
        dot_all: bool,
        proof: AlternativeProof,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Vec<AlternativeAtom>>, E> {
        let units = source.code_units();
        let allow_wrapped_branches = proof != AlternativeProof::Bare;
        charge(units.len().saturating_mul(4))?;
        let mut ranges = Vec::new();
        let mut start = 0;
        let mut depth = 0usize;
        let mut group_depth = 0usize;
        let mut at = 0;
        while at < units.len() {
            match units[at] {
                0x5c => {
                    at += 2;
                    continue;
                }
                0x5b if depth == 0 || unicode_sets => {
                    depth += 1;
                }
                0x5d if depth != 0 => {
                    depth -= 1;
                }
                0x28 if depth == 0 && allow_wrapped_branches => {
                    group_depth += 1;
                }
                0x29 if depth == 0 && allow_wrapped_branches => {
                    let Some(previous) = group_depth.checked_sub(1) else {
                        return Ok(None);
                    };
                    group_depth = previous;
                }
                0x7c if depth == 0 && group_depth == 0 => {
                    ranges.push(start..at);
                    start = at + 1;
                }
                _ => {}
            }
            at += 1;
        }
        if ranges.is_empty() || group_depth != 0 {
            return Ok(None);
        }
        ranges.push(start..units.len());
        let mut branches = Vec::with_capacity(ranges.len());
        for range in ranges {
            if range.is_empty() {
                return Ok(None);
            }
            charge(range.len())?;
            let branch = JsString::from_code_units(units[range].to_vec());
            let proved = if let Some(m) =
                RegExpUnicodeCharacterMatcher::compile_with_flags_and_work(
                    &branch,
                    ignore_case,
                    multiline,
                    dot_all,
                    &mut charge,
                )? {
                Some(AlternativeAtom::character(m))
            } else if let Some(m) =
                RegExpUnicodeCharacterMatcher::compile_literal_with_flags_and_work(
                    &branch,
                    ignore_case,
                    multiline,
                    &mut charge,
                )?
            {
                Some(AlternativeAtom::character(m))
            } else if let Some(m) = RegExpUnicodeClassMatcher::compile_with_flags_and_work(
                &branch,
                unicode_sets,
                ignore_case,
                multiline,
                &mut charge,
            )? {
                Some(AlternativeAtom::class(m))
            } else if let Some(m) =
                RegExpUnicodeClassMatcher::compile_binary_property_with_flags_and_work(
                    &branch,
                    unicode_sets,
                    ignore_case,
                    multiline,
                    &mut charge,
                )?
            {
                Some(AlternativeAtom::class(m))
            } else if let Some(m) =
                RegExpUnicodeCharacterMatcher::compile_with_word_boundaries_and_work(
                    &branch,
                    ignore_case,
                    multiline,
                    dot_all,
                    &mut charge,
                )?
            {
                Some(AlternativeAtom::character(m))
            } else {
                RegExpUnicodeClassMatcher::compile_with_word_boundaries_and_work(
                    &branch,
                    unicode_sets,
                    ignore_case,
                    multiline,
                    &mut charge,
                )?
                .map(AlternativeAtom::class)
            };
            let proved = if proved.is_none() && allow_wrapped_branches {
                // This exact-one repetition reuses the complete consuming-group
                // proof. The branch has no quantifier; every retained wrapper
                // must be noncapturing and all predicates stay on its character.
                let mut start_word = None;
                let mut end_word = None;
                let mut impossible = false;
                let branch_source = if matches!(
                    proof,
                    AlternativeProof::SequenceWrapped | AlternativeProof::NestedSequenceWrapped
                ) {
                    if let Some((assertions, conflict)) =
                        unicode_branch_assertion_sequences_with_work(
                            &branch,
                            proof == AlternativeProof::NestedSequenceWrapped,
                            &mut charge,
                        )?
                    {
                        start_word = assertions.start;
                        end_word = assertions.end;
                        impossible = conflict;
                        assertions.source
                    } else {
                        branch.clone()
                    }
                } else if proof == AlternativeProof::AssertedWrapped {
                    if let Some(assertions) =
                        unicode_word_assertions_with_work(&branch, &mut charge)?
                    {
                        start_word = assertions.start;
                        end_word = assertions.end;
                        assertions.source
                    } else {
                        branch.clone()
                    }
                } else {
                    branch.clone()
                };
                charge(branch_source.len().saturating_add(3))?;
                let mut units = branch_source.code_units().to_vec();
                let end_line = matches!(
                    proof,
                    AlternativeProof::LineWrapped
                        | AlternativeProof::AssertedWrapped
                        | AlternativeProof::SequenceWrapped
                        | AlternativeProof::NestedSequenceWrapped
                ) && units.last() == Some(&0x24)
                    && !escaped(&units, units.len() - 1);
                if end_line {
                    units.pop();
                }
                units.extend_from_slice(&[0x7b, 0x31, 0x7d]);
                if end_line {
                    units.push(0x24);
                }
                let branch = JsString::from_code_units(units);
                // Keep the non-alternative leaf proof's callback type fixed:
                // its call graph shares these helpers, but this mode cannot
                // admit alternatives or recurse while matching a branch.
                let branch_charge: &mut dyn FnMut(usize) -> Result<(), E> = &mut charge;
                match Self::compile_wrapped_atom_with_work(
                    &branch,
                    unicode_sets,
                    ignore_case,
                    multiline,
                    dot_all,
                    AtomAssertions::Word,
                    branch_charge,
                )? {
                    Some(plan)
                        if plan.atom_captures == 0
                            && plan.minimum == Some(1)
                            && plan.maximum == Some(1)
                            && (matches!(
                                proof,
                                AlternativeProof::LineWrapped
                                    | AlternativeProof::AssertedWrapped
                                    | AlternativeProof::SequenceWrapped
                                    | AlternativeProof::NestedSequenceWrapped
                            ) || (!plan.start_anchor && !plan.end_anchor))
                            && plan.start_word.is_none()
                            && plan.end_word.is_none() =>
                    {
                        let mut branch = match plan.atom {
                            Atom::Character(m) => AlternativeAtom::character(m),
                            Atom::Class(m) => AlternativeAtom::class(m),
                            Atom::Alternatives { .. } => return Ok(None),
                        };
                        branch.impossible = impossible;
                        if plan.start_anchor
                            || plan.end_anchor
                            || start_word.is_some()
                            || end_word.is_some()
                        {
                            // This complete branch consumes one character, so
                            // its exterior word and line predicates constrain
                            // that character's original boundaries.
                            let words = start_word.is_some() || end_word.is_some();
                            charge(if words { 8 } else { 3 })?;
                            let mut guard = Vec::with_capacity(if words { 7 } else { 3 });
                            if plan.start_anchor {
                                guard.push(0x5e);
                            }
                            if let Some(positive) = start_word {
                                guard
                                    .extend_from_slice(&[0x5c, if positive { 0x62 } else { 0x42 }]);
                            }
                            guard.push(0x2e);
                            if let Some(positive) = end_word {
                                guard
                                    .extend_from_slice(&[0x5c, if positive { 0x62 } else { 0x42 }]);
                            }
                            if plan.end_anchor {
                                guard.push(0x24);
                            }
                            let guard = JsString::from_code_units(guard);
                            branch.guard = if words {
                                RegExpUnicodeCharacterMatcher::compile_with_word_boundaries_and_work(
                                    &guard, ignore_case, multiline, true, &mut charge,
                                )?
                            } else {
                                RegExpUnicodeCharacterMatcher::compile_with_flags_and_work(
                                    &guard,
                                    ignore_case,
                                    multiline,
                                    true,
                                    &mut charge,
                                )?
                            };
                            if branch.guard.is_none() {
                                return Ok(None);
                            }
                        }
                        Some(branch)
                    }
                    _ => None,
                }
            } else {
                proved
            };
            let Some(proved) = proved else {
                return Ok(None);
            };
            branches.push(proved);
        }
        Ok(Some(branches))
    }

    fn compile_wrapped_atom_with_work<E>(
        source: &JsString,
        unicode_sets: bool,
        ignore_case: bool,
        multiline: bool,
        dot_all: bool,
        assertions: AtomAssertions,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, E> {
        let units = source.code_units();
        charge(units.len())?;
        let start_anchor = units.first() == Some(&0x5e);
        let end_anchor = units.last() == Some(&0x24) && !escaped(units, units.len() - 1);
        let start = usize::from(start_anchor);
        let end = units.len().saturating_sub(usize::from(end_anchor));
        let Some(body) = units.get(start..end) else {
            return Ok(None);
        };
        // Branch wrappers are not whole-unit wrappers. Record balanced ends
        // once so a leading branch group cannot be stripped from an alternative.
        // Previous entry points need no map and keep their original charges.
        let group_ends = if matches!(
            assertions,
            AtomAssertions::WrappedAlternative
                | AtomAssertions::LineWrappedAlternative
                | AtomAssertions::AssertedWrappedAlternative
                | AtomAssertions::SequenceWrappedAlternative
                | AtomAssertions::NestedSequenceWrappedAlternative
        ) {
            charge(body.len().saturating_mul(3))?;
            let mut ends = vec![usize::MAX; body.len()];
            let mut stack = Vec::new();
            let mut classes = 0usize;
            let mut at = 0;
            while at < body.len() {
                match body[at] {
                    0x5c => {
                        at += 2;
                        continue;
                    }
                    0x5b if classes == 0 || unicode_sets => {
                        classes += 1;
                    }
                    0x5d if classes != 0 => {
                        classes -= 1;
                    }
                    0x28 if classes == 0 => {
                        stack.push(at);
                    }
                    0x29 if classes == 0 => {
                        let Some(open) = stack.pop() else {
                            return Ok(None);
                        };
                        ends[open] = at;
                    }
                    _ => {}
                }
                at += 1;
            }
            if !stack.is_empty() || classes != 0 {
                return Ok(None);
            }
            Some(ends)
        } else {
            None
        };
        for end in [
            body.len()
                .checked_sub(1)
                .filter(|_| body.last() == Some(&0x3f)),
            Some(body.len()),
        ] {
            let Some(end) = end else { continue };
            let Some((atom_end, _, _)) = quantifier(body, end) else {
                continue;
            };
            let mut at = 0;
            let mut wrappers = 0;
            let mut captures = 0;
            while body.get(at) == Some(&0x28) {
                if let Some(ends) = &group_ends {
                    if atom_end.checked_sub(wrappers + 1) != Some(ends[at]) {
                        break;
                    }
                }
                if body.get(at + 1) == Some(&0x3f) {
                    if body.get(at + 2) != Some(&0x3a) {
                        break;
                    }
                    at += 3;
                } else {
                    at += 1;
                    captures += 1;
                }
                wrappers += 1;
            }
            let Some(inner_end) = atom_end.checked_sub(wrappers) else {
                continue;
            };
            if wrappers == 0
                || at >= inner_end
                || !body[inner_end..atom_end].iter().all(|&c| c == 0x29)
            {
                continue;
            }
            // Retained assertions belong to the consuming atom when explicitly
            // requested. They must never be moved onto the whole repetition.
            charge(body.len())?;
            let mut repeated = Vec::with_capacity(body.len());
            repeated.extend_from_slice(&body[at..inner_end]);
            repeated.extend_from_slice(&body[atom_end..]);
            let repeated = JsString::from_code_units(repeated);
            let Some(mut plan) = Self::compile_atom_repetition_with_work(
                &repeated,
                unicode_sets,
                ignore_case,
                if assertions != AtomAssertions::None {
                    multiline
                } else {
                    false
                },
                dot_all,
                assertions,
                &mut charge,
            )?
            else {
                continue;
            };
            plan.start_anchor = start_anchor;
            plan.end_anchor = end_anchor;
            plan.multiline = multiline;
            plan.atom_captures = captures;
            return Ok(Some(plan));
        }
        Ok(None)
    }

    /// Compiles a leading word assertion outside one complete atom repetition.
    ///
    /// Requires complete u/v validation and normalized named groups. An optional
    /// ^ precedes the word assertion; an optional $ follows the repetition.
    /// Complete inner wrappers retain their final-iteration captures. Trailing
    /// word assertions and assertions inside the repeated atom remain excluded.
    pub fn compile_with_start_word_boundary_and_work<E>(
        source: &JsString,
        unicode_sets: bool,
        ignore_case: bool,
        multiline: bool,
        dot_all: bool,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, E> {
        let Some(assertions) = unicode_word_assertions_with_work(source, &mut charge)? else {
            return Ok(None);
        };
        if assertions.start.is_none() || assertions.end.is_some() {
            return Ok(None);
        }
        Self::compile_word_boundary_body_with_work(
            assertions,
            unicode_sets,
            ignore_case,
            multiline,
            dot_all,
            charge,
        )
    }

    /// Compiles optional leading and trailing word assertions on a repetition.
    ///
    /// Requires complete u/v validation and normalized named groups. At least
    /// one outside word assertion is required. Optional ^/$ keep their line
    /// scopes, and complete inner groups retain final selected captures.
    pub fn compile_with_word_boundaries_and_work<E>(
        source: &JsString,
        unicode_sets: bool,
        ignore_case: bool,
        multiline: bool,
        dot_all: bool,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, E> {
        let Some(assertions) = unicode_word_assertions_with_work(source, &mut charge)? else {
            return Ok(None);
        };
        Self::compile_word_boundary_body_with_work(
            assertions,
            unicode_sets,
            ignore_case,
            multiline,
            dot_all,
            charge,
        )
    }

    /// Compiles outside word assertions on complete repeated atom groups.
    ///
    /// Requires complete u/v validation and normalized named groups. The retained
    /// body must pass the consuming-group proof, including any per-iteration line
    /// assertions. Word predicates remain outside the repetition; optional
    /// outside ^/$ keep their separate original scopes. Inner word assertions,
    /// empty units and other compositions remain excluded.
    pub fn compile_with_scoped_word_boundaries_and_work<E>(
        source: &JsString,
        unicode_sets: bool,
        ignore_case: bool,
        multiline: bool,
        dot_all: bool,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, E> {
        let Some(assertions) = unicode_word_assertions_with_work(source, &mut charge)? else {
            return Ok(None);
        };
        let Some(mut plan) = Self::compile_capturing_atom_with_line_assertions_and_work(
            &assertions.source,
            unicode_sets,
            ignore_case,
            multiline,
            dot_all,
            charge,
        )?
        else {
            return Ok(None);
        };
        plan.start_word = assertions.start;
        plan.end_word = assertions.end;
        plan.word_ignore_case = ignore_case;
        Ok(Some(plan))
    }

    /// Compiles outside word assertions on complete repeated atom groups.
    ///
    /// Requires complete u/v validation and normalized named groups. The retained
    /// body must pass the consuming-group proof, including any per-iteration word or line
    /// assertions. Word predicates remain outside the repetition; optional
    /// outside ^/$ keep their separate original scopes. Empty units and other compositions remain excluded.
    pub fn compile_with_composed_word_boundaries_and_work<E>(
        source: &JsString,
        unicode_sets: bool,
        ignore_case: bool,
        multiline: bool,
        dot_all: bool,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, E> {
        let Some(assertions) = unicode_word_assertions_with_work(source, &mut charge)? else {
            return Ok(None);
        };
        let Some(mut plan) = Self::compile_capturing_atom_with_assertions_and_work(
            &assertions.source,
            unicode_sets,
            ignore_case,
            multiline,
            dot_all,
            charge,
        )?
        else {
            return Ok(None);
        };
        plan.start_word = assertions.start;
        plan.end_word = assertions.end;
        plan.word_ignore_case = ignore_case;
        Ok(Some(plan))
    }

    /// Proves complete nested groups interleaved with atom assertions.
    ///
    /// Requires full u/v validation and normalized named groups. Every group
    /// must contain the same single consuming character. Retained word/line
    /// predicates are checked on that character at each selected iteration;
    /// optional outside line assertions keep their whole-repetition scopes.
    /// Empty groups, alternatives, concatenations, duplicate predicates and
    /// outside word assertions remain excluded. This scan adds no recursion.
    pub fn compile_nested_assertion_groups_with_work<E>(
        source: &JsString,
        unicode_sets: bool,
        ignore_case: bool,
        multiline: bool,
        dot_all: bool,
        charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, E> {
        Self::compile_nested_assertion_unit_with_work(
            source,
            unicode_sets,
            ignore_case,
            multiline,
            dot_all,
            false,
            charge,
        )
    }

    /// Proves shared assertions outside a complete repeated alternative group.
    ///
    /// Requires full u/v validation and normalized named groups. Every wrapper
    /// contains one consuming alternative unit. Shared predicates must surround
    /// the innermost complete group, so they constrain every branch rather than
    /// only the first or last branch. Branches retain their own full leaf proofs.
    pub fn compile_nested_alternative_assertions_with_work<E>(
        source: &JsString,
        unicode_sets: bool,
        ignore_case: bool,
        multiline: bool,
        dot_all: bool,
        charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, E> {
        Self::compile_nested_assertion_unit_with_work(
            source,
            unicode_sets,
            ignore_case,
            multiline,
            dot_all,
            true,
            charge,
        )
    }

    /// Composes outside word assertions with complete shared alternative groups.
    ///
    /// Requires full u/v validation and normalized named groups. The entire
    /// retained body must pass the shared group proof before outside predicates
    /// are attached. Shared, branch and whole-repetition scopes remain distinct.
    pub fn compile_scoped_alternative_words_with_work<E>(
        source: &JsString,
        unicode_sets: bool,
        ignore_case: bool,
        multiline: bool,
        dot_all: bool,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, E> {
        let Some(assertions) = unicode_word_assertions_with_work(source, &mut charge)? else {
            return Ok(None);
        };
        let Some(mut plan) = Self::compile_nested_alternative_assertions_with_work(
            &assertions.source,
            unicode_sets,
            ignore_case,
            multiline,
            dot_all,
            charge,
        )?
        else {
            return Ok(None);
        };
        plan.start_word = assertions.start;
        plan.end_word = assertions.end;
        plan.word_ignore_case = ignore_case;
        Ok(Some(plan))
    }

    fn compile_nested_assertion_unit_with_work<E>(
        source: &JsString,
        unicode_sets: bool,
        ignore_case: bool,
        multiline: bool,
        dot_all: bool,
        alternatives: bool,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, E> {
        let units = source.code_units();
        charge(units.len().saturating_mul(8))?;
        let outer_start = units.first() == Some(&0x5e);
        let outer_end = units.last() == Some(&0x24) && !escaped(units, units.len() - 1);
        let Some(body) =
            units.get(usize::from(outer_start)..units.len().saturating_sub(usize::from(outer_end)))
        else {
            return Ok(None);
        };
        if body.first() != Some(&0x28) {
            return Ok(None);
        }
        for end in [
            body.len()
                .checked_sub(1)
                .filter(|_| body.last() == Some(&0x3f)),
            Some(body.len()),
        ] {
            let Some(end) = end else { continue };
            let Some((atom_end, minimum, maximum)) = quantifier(body, end) else {
                continue;
            };
            let greedy = end == body.len();
            let atom = &body[..atom_end];
            let (mut begin, mut end, mut opened, mut closed, mut captures) =
                (0, atom.len(), 0, 0, 0);
            let (mut start_line, mut end_line, mut start_word, mut end_word) =
                (false, false, None, None);
            let (mut prefix_guard_depth, mut suffix_guard_depth) = (None, None);
            loop {
                match atom.get(begin) {
                    Some(0x28) => {
                        if atom.get(begin + 1) == Some(&0x3f) {
                            if atom.get(begin + 2) != Some(&0x3a) {
                                return Ok(None);
                            }
                            begin += 3;
                        } else {
                            begin += 1;
                            captures += 1;
                        }
                        opened += 1;
                    }
                    Some(0x5e) if !start_line => {
                        start_line = true;
                        prefix_guard_depth = Some(opened);
                        begin += 1;
                    }
                    Some(0x5c) if start_word.is_none() => match atom.get(begin + 1) {
                        Some(kind @ (0x62 | 0x42)) => {
                            start_word = Some(*kind == 0x62);
                            prefix_guard_depth = Some(opened);
                            begin += 2;
                        }
                        _ => break,
                    },
                    _ => break,
                }
            }
            while end > begin {
                if atom[end - 1] == 0x29 && !escaped(atom, end - 1) {
                    closed += 1;
                    end -= 1;
                } else if atom[end - 1] == 0x24 && !end_line && !escaped(atom, end - 1) {
                    end_line = true;
                    suffix_guard_depth = Some(closed);
                    end -= 1;
                } else if end >= begin + 2
                    && atom[end - 2] == 0x5c
                    && matches!(atom[end - 1], 0x62 | 0x42)
                    && end_word.is_none()
                    && !escaped(atom, end - 2)
                {
                    end_word = Some(atom[end - 1] == 0x62);
                    suffix_guard_depth = Some(closed);
                    end -= 2;
                } else {
                    break;
                }
            }
            if opened == 0 || closed != opened || begin >= end {
                continue;
            }
            if alternatives {
                // In (\ba|b), the word predicate belongs only to the first
                // branch. In (\b(a|b)), another group opens after it, proving
                // that it surrounds the complete disjunction. Apply the same
                // requirement in reverse for suffix predicates.
                if prefix_guard_depth.is_some_and(|depth| depth == opened)
                    || suffix_guard_depth.is_some_and(|depth| depth == closed)
                {
                    continue;
                }
                charge(body.len())?;
                let leaf = JsString::from_code_units(atom[begin..end].to_vec());
                let Some(branches) = Self::compile_alternatives_with_work(
                    &leaf,
                    unicode_sets,
                    ignore_case,
                    multiline,
                    dot_all,
                    AlternativeProof::Bare,
                    &mut charge,
                )?
                else {
                    continue;
                };
                charge(8)?;
                let mut guard_source = Vec::with_capacity(7);
                if start_line {
                    guard_source.push(0x5e);
                }
                if let Some(positive) = start_word {
                    guard_source.extend_from_slice(&[0x5c, if positive { 0x62 } else { 0x42 }]);
                }
                // DotAll consumes exactly the same decoded character as every
                // branch, leaving only the shared pure assertions as a gate.
                guard_source.push(0x2e);
                if let Some(positive) = end_word {
                    guard_source.extend_from_slice(&[0x5c, if positive { 0x62 } else { 0x42 }]);
                }
                if end_line {
                    guard_source.push(0x24);
                }
                let guard_source = JsString::from_code_units(guard_source);
                let guard = if start_word.is_some() || end_word.is_some() {
                    RegExpUnicodeCharacterMatcher::compile_with_word_boundaries_and_work(
                        &guard_source,
                        ignore_case,
                        multiline,
                        true,
                        &mut charge,
                    )?
                } else {
                    RegExpUnicodeCharacterMatcher::compile_with_flags_and_work(
                        &guard_source,
                        ignore_case,
                        multiline,
                        true,
                        &mut charge,
                    )?
                };
                let Some(guard) = guard else {
                    continue;
                };
                let work = branches
                    .iter()
                    .fold(guard.search_passes(), |n, m| n.saturating_add(m.work()));
                return Ok(Some(Self {
                    atom: Atom::Alternatives {
                        branches,
                        work,
                        guard: Some(guard),
                    },
                    minimum,
                    maximum,
                    greedy,
                    start_anchor: outer_start,
                    end_anchor: outer_end,
                    multiline,
                    atom_captures: captures,
                    start_word: None,
                    end_word: None,
                    word_ignore_case: false,
                    nullable: None,
                }));
            }
            // All groups contain the same consuming leaf. Assertions have no
            // capture effects, so their conjunction can be represented in the
            // established leaf order without changing positions or scopes.
            charge(body.len())?;
            let mut repeated = Vec::with_capacity(body.len());
            if start_line {
                repeated.push(0x5e);
            }
            if let Some(positive) = start_word {
                repeated.extend_from_slice(&[0x5c, if positive { 0x62 } else { 0x42 }]);
            }
            repeated.extend_from_slice(&atom[begin..end]);
            if let Some(positive) = end_word {
                repeated.extend_from_slice(&[0x5c, if positive { 0x62 } else { 0x42 }]);
            }
            if end_line {
                repeated.push(0x24);
            }
            repeated.extend_from_slice(&body[atom_end..]);
            let Some(mut plan) = Self::compile_atom_repetition_with_work(
                &JsString::from_code_units(repeated),
                unicode_sets,
                ignore_case,
                multiline,
                dot_all,
                AtomAssertions::Word,
                &mut charge,
            )?
            else {
                continue;
            };
            plan.start_anchor = outer_start;
            plan.end_anchor = outer_end;
            plan.multiline = multiline;
            plan.atom_captures = captures;
            return Ok(Some(plan));
        }
        Ok(None)
    }

    /// Compiles outside word assertions on complete repeated atom groups.
    ///
    /// Requires complete u/v validation and normalized named groups. The retained
    /// body must pass the complete interleaved-group proof, including any per-iteration word or line
    /// assertions. Word predicates remain outside the repetition; optional
    /// outside ^/$ keep their separate original scopes. Empty units and other compositions remain excluded.
    pub fn compile_with_nested_assertion_word_boundaries_and_work<E>(
        source: &JsString,
        unicode_sets: bool,
        ignore_case: bool,
        multiline: bool,
        dot_all: bool,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, E> {
        let Some(assertions) = unicode_word_assertions_with_work(source, &mut charge)? else {
            return Ok(None);
        };
        let Some(mut plan) = Self::compile_nested_assertion_groups_with_work(
            &assertions.source,
            unicode_sets,
            ignore_case,
            multiline,
            dot_all,
            charge,
        )?
        else {
            return Ok(None);
        };
        plan.start_word = assertions.start;
        plan.end_word = assertions.end;
        plan.word_ignore_case = ignore_case;
        Ok(Some(plan))
    }

    fn compile_word_boundary_body_with_work<E>(
        assertions: UnicodeWordAssertions,
        unicode_sets: bool,
        ignore_case: bool,
        multiline: bool,
        dot_all: bool,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, E> {
        let matcher = Self::compile_with_line_anchors_and_work(
            &assertions.source,
            unicode_sets,
            ignore_case,
            multiline,
            dot_all,
            &mut charge,
        )?;
        let matcher = match matcher {
            Some(matcher) => Some(matcher),
            None => Self::compile_capturing_atom_with_work(
                &assertions.source,
                unicode_sets,
                ignore_case,
                multiline,
                dot_all,
                charge,
            )?,
        };
        Ok(matcher.map(|mut matcher| {
            matcher.start_word = assertions.start;
            matcher.end_word = assertions.end;
            matcher.word_ignore_case = ignore_case;
            matcher
        }))
    }

    /// Returns the number of complete ordinary wrappers inside the repeated atom.
    pub fn capture_count(&self) -> usize {
        self.atom_captures
    }

    /// Finds the repetition and its final selected complete-character capture.
    ///
    /// Complete inner captures all share the final iteration. They are undefined
    /// for zero iterations, but empty for a mandatory nullable iteration. This
    /// adds no per-capture allocation.
    pub fn find_with_captures_and_work<E>(
        &self,
        input: &JsString,
        start: usize,
        sticky: bool,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<RegExpUnicodeRepeatedAtomMatch>, E> {
        let Some(range) = self.find_with_work(input, start, sticky, &mut charge)? else {
            return Ok(None);
        };
        // With the character branch first, fewer than the mandatory count
        // selected characters leave a final mandatory empty iteration. Count
        // only the selected UTF-16 characters, stopping at the source bound.
        // An unrepresentable bound exceeds every finite input character count.
        let mandatory_empty = if self.atom_captures != 0
            && self.nullable.is_some()
            && self.minimum != Some(0)
            && self.minimum != Some(1)
        {
            if range.is_empty()
                || (self.nullable == Some(EmptyBranchOrder::Last) && self.minimum.is_none())
            {
                true
            } else if self.nullable == Some(EmptyBranchOrder::Last) {
                let units = input.code_units();
                let mut at = range.start;
                let mut count = 0;
                while at < range.end && Some(count) != self.minimum {
                    charge(2)?;
                    let (_, width) = unicode_input_character(units, at, units[at]);
                    at += width;
                    count += 1;
                }
                Some(count) != self.minimum
            } else {
                false
            }
        } else {
            false
        };
        let atom_capture = if mandatory_empty {
            charge(1)?;
            Some(range.end..range.end)
        } else if self.atom_captures != 0
            && range.is_empty()
            && self.nullable.is_some()
            && self.minimum == Some(1)
        {
            charge(1)?;
            Some(range.clone())
        } else if self.atom_captures != 0 && !range.is_empty() {
            charge(2)?;
            let units = input.code_units();
            let mut at = range.end - 1;
            if at > range.start
                && (0xdc00..=0xdfff).contains(&units[at])
                && (0xd800..=0xdbff).contains(&units[at - 1])
            {
                at -= 1;
            }
            Some(at..range.end)
        } else {
            None
        };
        Ok(Some(RegExpUnicodeRepeatedAtomMatch {
            range,
            atom_capture,
        }))
    }

    /// Finds the earliest complete-character candidate with source-order choices.
    ///
    /// Nullable alternatives preserve mandatory empty iterations. For lazy +
    /// with the character branch first, nonempty and empty candidates are found
    /// in separate linear passes: earliest start wins, with the character path
    /// preferred at the same start (22.2.2.3). No input-sized storage is added.
    pub fn find_with_work<E>(
        &self,
        input: &JsString,
        start: usize,
        sticky: bool,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Range<usize>>, E> {
        let mut bounds = QuantifierBounds {
            minimum: self.minimum,
            maximum: self.maximum,
            greedy: self.greedy,
        };
        if self.nullable.is_some() {
            bounds.minimum = Some(0);
        }
        let optional = match (self.nullable, self.greedy, self.minimum, self.maximum) {
            (Some(EmptyBranchOrder::First), true, Some(minimum), Some(maximum))
                if minimum != 0 && maximum > minimum =>
            {
                Some(maximum - minimum)
            }
            (Some(EmptyBranchOrder::FirstFiniteOptional(optional)), true, _, None) => {
                Some(optional)
            }
            _ => None,
        };
        if let Some(optional) = optional {
            // Prefer greedy optional iterations after all mandatory empty
            // choices, then backtrack mandatory choices to longer lazy runs.
            // Representable positive bounds prove optional < maximum.
            // Overflowed source bounds retain a checked optional count.
            let first = self.find_with_bounds_and_work(
                input,
                start,
                sticky,
                QuantifierBounds {
                    maximum: Some(optional),
                    ..bounds
                },
                &mut charge,
            )?;
            // An optional count of usize::MAX exceeds every input range.
            // Its higher phase is unreachable and must not overflow +1.
            let Some(higher_minimum) = optional.checked_add(1) else {
                return Ok(first);
            };
            let higher = self.find_with_bounds_and_work(
                input,
                start,
                sticky,
                QuantifierBounds {
                    minimum: Some(higher_minimum),
                    greedy: false,
                    ..bounds
                },
                charge,
            )?;
            return Ok(match (first, higher) {
                (Some(a), Some(b)) => Some(if a.start <= b.start { a } else { b }),
                (a, b) => a.or(b),
            });
        }
        if self.nullable.is_some()
            && self.minimum.is_some_and(|minimum| minimum > 1)
            && self.maximum == self.minimum
        {
            // Every iteration is mandatory. Character-first choices prefer
            // longer accepted runs; empty-first choices prefer shorter runs.
            // The quantifier's greedy flag cannot reorder those choices.
            bounds.greedy = self.nullable == Some(EmptyBranchOrder::Last);
            return self.find_with_bounds_and_work(input, start, sticky, bounds, charge);
        }
        if self.nullable.is_some() && self.minimum == Some(1) && self.maximum == Some(1) {
            // Exactly one mandatory iteration has no optional repetition.
            // Earliest start wins; the original branch order breaks a tie.
            let consuming = self.find_with_bounds_and_work(
                input,
                start,
                sticky,
                QuantifierBounds {
                    minimum: Some(1),
                    ..bounds
                },
                &mut charge,
            )?;
            let empty = self.find_with_bounds_and_work(
                input,
                start,
                sticky,
                QuantifierBounds {
                    maximum: Some(0),
                    ..bounds
                },
                charge,
            )?;
            return Ok(match (consuming, empty) {
                (Some(a), Some(b)) => Some(
                    if a.start < b.start
                        || (a.start == b.start && self.nullable == Some(EmptyBranchOrder::Last))
                    {
                        a
                    } else {
                        b
                    },
                ),
                (a, b) => a.or(b),
            });
        }
        if self.nullable == Some(EmptyBranchOrder::Last)
            && self.minimum != Some(0)
            && self.minimum != Some(1)
            && !self.greedy
        {
            // Mandatory character choices precede mandatory empty choices.
            // First try lazily ending at or above the lower bound, then greedily
            // below it. Each pass remains monotone, including outside guards.
            let enough = self.find_with_bounds_and_work(
                input,
                start,
                sticky,
                QuantifierBounds {
                    minimum: self.minimum,
                    ..bounds
                },
                &mut charge,
            )?;
            let shorter = self.find_with_bounds_and_work(
                input,
                start,
                sticky,
                QuantifierBounds {
                    maximum: self.minimum.map(|minimum| minimum - 1),
                    greedy: true,
                    ..bounds
                },
                charge,
            )?;
            return Ok(match (enough, shorter) {
                (Some(a), Some(b)) => Some(if a.start <= b.start { a } else { b }),
                (a, b) => a.or(b),
            });
        }
        if self.nullable == Some(EmptyBranchOrder::Last) && self.minimum == Some(1) && !self.greedy
        {
            let consuming = self.find_with_bounds_and_work(
                input,
                start,
                sticky,
                QuantifierBounds {
                    minimum: Some(1),
                    ..bounds
                },
                &mut charge,
            )?;
            let empty = self.find_with_bounds_and_work(
                input,
                start,
                sticky,
                QuantifierBounds {
                    maximum: Some(0),
                    ..bounds
                },
                charge,
            )?;
            return Ok(match (consuming, empty) {
                (Some(a), Some(b)) => Some(if a.start <= b.start { a } else { b }),
                (a, b) => a.or(b),
            });
        }
        self.find_with_bounds_and_work(input, start, sticky, bounds, charge)
    }

    // With no multiline $, only the actual input end can satisfy the assertion.
    // Enumerating starts in order is equivalent to choosing the earliest start
    // of the final matching run that leaves a count within the exact bounds.
    // A second monotone pointer trims a finite maximum by complete characters.
    fn find_at_actual_end_with_work<E>(
        &self,
        input: &JsString,
        start: usize,
        sticky: bool,
        bounds: QuantifierBounds,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Range<usize>>, E> {
        charge(1)?;
        let Some(minimum) = bounds.minimum else {
            return Ok(None);
        };
        let Some(mut candidate) = unicode_start(input, start) else {
            return Ok(None);
        };
        if self.start_anchor && candidate != 0 {
            return Ok(None);
        };
        let units = input.code_units();
        if bounds.maximum == Some(0) {
            return Ok((!(sticky || self.start_anchor) || candidate == units.len())
                .then_some(units.len()..units.len()));
        }
        let mut cursor = candidate;
        let mut count = 0;
        while cursor < units.len() {
            charge(1usize.saturating_add(self.atom.work()))?;
            if let Some(found) = self.atom.find(input, cursor) {
                cursor = found.end;
                count += 1;
                if bounds.maximum.is_some_and(|upper| count > upper) {
                    if sticky || self.start_anchor {
                        return Ok(None);
                    };
                    charge(2)?;
                    let (_, width) = unicode_input_character(units, candidate, units[candidate]);
                    candidate += width;
                    count -= 1;
                }
            } else {
                if sticky || self.start_anchor {
                    return Ok(None);
                };
                charge(2)?;
                let (_, width) = unicode_input_character(units, cursor, units[cursor]);
                cursor += width;
                candidate = cursor;
                count = 0;
            }
        }
        Ok((count >= minimum).then_some(candidate..cursor))
    }

    // Complete-character endpoints are visited in order. The earliest start
    // that can reach an eligible endpoint never moves backwards: a finite
    // maximum or a leading assertion can only trim the matching run. Once that
    // start has succeeded, greedy search extends only its endpoint; lazy search
    // returns immediately. Both pointers advance at most once over each input
    // character. Ordinary membership and assertions invoke no user code.
    fn find_with_boundary_assertions_and_work<E>(
        &self,
        input: &JsString,
        start: usize,
        sticky: bool,
        bounds: QuantifierBounds,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Range<usize>>, E> {
        charge(1)?;
        let Some(minimum) = bounds.minimum else {
            return Ok(None);
        };
        let Some(mut candidate) = unicode_start(input, start) else {
            return Ok(None);
        };
        let units = input.code_units();
        let mut cursor = candidate;
        let mut count = 0;
        let mut best = None;
        loop {
            if self.start_anchor || self.start_word.is_some() {
                while {
                    charge(1 + 8 * usize::from(self.start_word.is_some()))?;
                    (self.start_anchor
                        && candidate != 0
                        && !(self.multiline
                            && matches!(units[candidate - 1], 0x0a | 0x0d | 0x2028 | 0x2029)))
                        || self.start_word.is_some_and(|expected| {
                            unicode_word_boundary(units, candidate, self.word_ignore_case)
                                != expected
                        })
                } {
                    if sticky || (self.start_anchor && !self.multiline) {
                        return Ok(None);
                    }
                    let Some(&first) = units.get(candidate) else {
                        return Ok(None);
                    };
                    charge(3)?;
                    let (_, width) = unicode_input_character(units, candidate, first);
                    candidate += width;
                    if candidate > cursor {
                        cursor = candidate;
                        count = 0;
                    } else {
                        count -= 1;
                    }
                }
            }
            charge(1 + 8 * usize::from(self.end_word.is_some()))?;
            let at_end = !self.end_anchor
                || cursor == units.len()
                || (self.multiline && matches!(units[cursor], 0x0a | 0x0d | 0x2028 | 0x2029));
            let word_end = self.end_word.is_none_or(|expected| {
                unicode_word_boundary(units, cursor, self.word_ignore_case) == expected
            });
            if at_end && word_end && count >= minimum {
                best = Some(candidate..cursor);
                if !bounds.greedy {
                    return Ok(best);
                }
            }
            if cursor == units.len() || (best.is_some() && bounds.maximum == Some(count)) {
                return Ok(best);
            }
            charge(1usize.saturating_add(self.atom.work()))?;
            if let Some(found) = self.atom.find(input, cursor) {
                cursor = found.end;
                count += 1;
                if bounds.maximum.is_some_and(|upper| count > upper) {
                    if sticky {
                        return Ok(None);
                    }
                    charge(2)?;
                    let (_, width) = unicode_input_character(units, candidate, units[candidate]);
                    candidate += width;
                    count -= 1;
                }
            } else {
                if best.is_some() || sticky {
                    return Ok(best);
                }
                charge(2)?;
                let (_, width) = unicode_input_character(units, cursor, units[cursor]);
                cursor += width;
                candidate = cursor;
                count = 0;
            }
        }
    }

    /// Finds the earliest complete-character candidate and its required run.
    ///
    /// A failed short run cannot satisfy the same minimum at a later position
    /// inside that run. Skip it and its failing character once, keeping search
    /// linear and constant in storage. Initial pair offsets use the approved
    /// leading boundary; returned empty ranges use their actual selected position.
    fn find_with_bounds_and_work<E>(
        &self,
        input: &JsString,
        start: usize,
        sticky: bool,
        bounds: QuantifierBounds,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Range<usize>>, E> {
        if self.start_word.is_some() || self.end_word.is_some() {
            return self
                .find_with_boundary_assertions_and_work(input, start, sticky, bounds, charge);
        }
        if self.end_anchor {
            return if self.multiline {
                self.find_with_boundary_assertions_and_work(input, start, sticky, bounds, charge)
            } else {
                self.find_at_actual_end_with_work(input, start, sticky, bounds, charge)
            };
        }
        charge(1)?;
        let Some(minimum) = bounds.minimum else {
            return Ok(None);
        };
        let Some(mut candidate) = unicode_start(input, start) else {
            return Ok(None);
        };
        let units = input.code_units();
        loop {
            if self.start_anchor {
                charge(1)?;
                if candidate != 0
                    && !(self.multiline
                        && matches!(units[candidate - 1], 0x0a | 0x0d | 0x2028 | 0x2029))
                {
                    if sticky || !self.multiline {
                        return Ok(None);
                    }
                    let Some(&first) = units.get(candidate) else {
                        return Ok(None);
                    };
                    charge(2)?;
                    let (_, width) = unicode_input_character(units, candidate, first);
                    candidate += width;
                    continue;
                }
            }
            let mut cursor = candidate;
            let mut count = 0;
            while bounds.maximum.is_none_or(|upper| count < upper) {
                if !bounds.greedy && count == minimum {
                    break;
                }
                charge(1usize.saturating_add(self.atom.work()))?;
                let Some(found) = self.atom.find(input, cursor) else {
                    break;
                };
                cursor = found.end;
                count += 1;
            }
            if count >= minimum {
                return Ok(Some(candidate..cursor));
            }
            if sticky {
                return Ok(None);
            }
            let Some(&first) = units.get(cursor) else {
                return Ok(None);
            };
            charge(2)?;
            let (_, width) = unicode_input_character(units, cursor, first);
            candidate = cursor + width;
        }
    }
}

// All extracted predicates are pure and inspect the same original boundary
// (22.2.2.4). The caller still proves the entire consuming branch before use.
fn unicode_branch_assertion_sequences_with_work<E>(
    source: &JsString,
    allow_noncapturing_wrappers: bool,
    mut charge: impl FnMut(usize) -> Result<(), E>,
) -> Result<Option<(UnicodeWordAssertions, bool)>, E> {
    let units = source.code_units();
    charge(units.len().saturating_mul(2))?;
    let (mut begin, mut end) = (0, units.len());
    let (mut start_line, mut end_line) = (false, false);
    let (mut start_word, mut end_word) = (None, None);
    let mut impossible = false;
    let (mut opened, mut closed) = (0, 0);
    while begin < end {
        if units[begin] == 0x5e {
            start_line = true;
            begin += 1;
        } else if let Some([0x5c, kind @ (0x62 | 0x42)]) = units[begin..end].get(..2) {
            let positive = *kind == 0x62;
            impossible |= start_word.is_some_and(|old| old != positive);
            start_word = Some(positive);
            begin += 2;
        } else if allow_noncapturing_wrappers
            && units[begin..end].get(..3) == Some(&[0x28, 0x3f, 0x3a])
        {
            opened += 1;
            begin += 3;
        } else {
            break;
        }
    }
    while end > begin {
        if units[end - 1] == 0x24 && !escaped(units, end - 1) {
            end_line = true;
            end -= 1;
        } else if end - begin >= 2
            && matches!(&units[end - 2..end], [0x5c, 0x62 | 0x42])
            && !escaped(units, end - 2)
        {
            let positive = units[end - 1] == 0x62;
            impossible |= end_word.is_some_and(|old| old != positive);
            end_word = Some(positive);
            end -= 2;
        } else if closed < opened && units[end - 1] == 0x29 && !escaped(units, end - 1) {
            closed += 1;
            end -= 1;
        } else {
            break;
        }
    }
    if opened != closed {
        return Ok(None);
    }
    if begin == 0 && end == units.len() {
        return Ok(None);
    }
    let length = end - begin
        + usize::from(start_line)
        + usize::from(end_line)
        + if allow_noncapturing_wrappers { 4 } else { 0 };
    charge(length)?;
    let mut body = Vec::with_capacity(length);
    if start_line {
        body.push(0x5e);
    }
    if allow_noncapturing_wrappers {
        body.extend_from_slice(&[0x28, 0x3f, 0x3a]);
    }
    body.extend_from_slice(&units[begin..end]);
    if allow_noncapturing_wrappers {
        body.push(0x29);
    }
    if end_line {
        body.push(0x24);
    }
    Ok(Some((
        UnicodeWordAssertions {
            source: JsString::from_code_units(body),
            start: start_word,
            end: end_word,
        },
        impossible,
    )))
}

fn escaped(units: &[u16], at: usize) -> bool {
    units[..at].iter().rev().take_while(|&&u| u == 0x5c).count() % 2 != 0
}

#[cfg(test)]
mod captured_atom_tests {
    use super::*;
    use std::fmt::Write;

    type Case = (
        &'static str,
        u8,
        usize,
        Option<usize>,
        bool,
        usize,
        bool,
        bool,
    );
    const CASES: [Case; 16] = [
        ("(a)+", 0, 1, None, true, 1, false, false),
        ("((a)){0,3}?", 0, 0, Some(3), false, 2, false, false),
        ("(?:(a)){2,4}", 0, 2, Some(4), true, 1, false, false),
        ("(?:a)+", 0, 1, None, true, 0, false, false),
        ("((?:a)){0}", 0, 0, Some(0), true, 1, false, false),
        ("([aſK])+", 1, 1, None, true, 1, false, false),
        ("((.)){1,3}$", 2, 1, Some(3), true, 2, false, true),
        (r"^((\D)){0,2}?$", 3, 0, Some(2), false, 2, true, true),
        (r"^(\p{Join_C})+$", 4, 1, None, true, 1, true, true),
        (r"(\p{NChar}){1,2}$", 5, 1, Some(2), true, 1, false, true),
        ("(😀)+", 6, 1, None, true, 1, false, false),
        (r"((\uD800)){0,1}", 7, 0, Some(1), true, 2, false, false),
        (r"(\?)??", 8, 0, Some(1), false, 1, false, false),
        (r"(?:(\W))+", 9, 1, None, true, 1, false, false),
        (r"(\u{61})?", 0, 0, Some(1), true, 1, false, false),
        ("((a)){1,3}$", 0, 1, Some(3), true, 2, false, true),
    ];
    fn plan(source: &str, v: bool, i: bool, m: bool, s: bool) -> RegExpUnicodeRepeatedAtomMatcher {
        RegExpUnicodeRepeatedAtomMatcher::compile_capturing_atom_with_work(
            &JsString::from(source),
            v,
            i,
            m,
            s,
            |_| Ok::<(), ()>(()),
        )
        .unwrap()
        .unwrap()
    }

    #[test]
    fn unicode_captured_atom_ranges_snapshot() {
        let mut rows = String::new();
        for (source, ..) in CASES {
            let pattern = JsString::from(source);
            for v in [false, true] {
                for i in [false, true] {
                    for m in [false, true] {
                        for s in [false, true] {
                            let matcher = plan(source, v, i, m, s);
                            for input in [
                                JsString::from("Aaaa\nAA"),
                                JsString::from("A\r\n\u{200c}\u{200d}\u{ffff}\u{10ffff}"),
                                JsString::from("😀??aaſK"),
                                JsString::from_code_units(vec![0x61, 0xd800, 0x61, 0xdc00]),
                                JsString::from(""),
                            ] {
                                for start in [0, 1, input.len(), input.len() + 1] {
                                    for sticky in [false, true] {
                                        let found = matcher
                                            .find_with_captures_and_work(
                                                &input,
                                                start,
                                                sticky,
                                                |_| Ok::<(), ()>(()),
                                            )
                                            .unwrap();
                                        writeln!(rows,"{pattern:?} v={v} i={i} m={m} s={s} input={input:?} start={start} sticky={sticky} count={} range={:?} capture={:?}",matcher.capture_count(),found.as_ref().map(|f|f.range.clone()),found.and_then(|f|f.atom_capture)).unwrap();
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        insta::assert_snapshot!(rows);
    }

    #[test]
    fn unicode_captured_atoms_agree_with_independent_counts_and_final_character() {
        let alphabet: [u16; 10] = [
            0x41, 0x61, 10, 13, 0x17f, 0x212a, 0xd83d, 0xde00, 0xd800, 0xdc00,
        ];
        let lt = |c| [10, 13, 0x2028, 0x2029].contains(&c);
        for i in [false, true] {
            for m in [false, true] {
                for s in [false, true] {
                    let plans: Vec<_> = CASES.iter().map(|c| plan(c.0, true, i, m, s)).collect();
                    let fold = |c| {
                        if i {
                            match c {
                                0x41 => 0x61,
                                0x17f => 0x73,
                                0x212a => 0x6b,
                                _ => c,
                            }
                        } else {
                            c
                        }
                    };
                    let member = |kind, c| match kind {
                        0 => fold(c) == 0x61,
                        1 => [0x61, fold(0x17f), fold(0x212a)].contains(&fold(c)),
                        2 => s || !lt(c),
                        3 => !(0x30..=0x39).contains(&c),
                        4 => (0x200c..=0x200d).contains(&c),
                        5 => matches!(c, 0xfdd0..=0xfdef) || (c & 0xffff) >= 0xfffe,
                        6 => c == 0x1f600,
                        7 => c == 0xd800,
                        8 => c == 0x3f,
                        9 => !char::from_u32(fold(c))
                            .is_some_and(|c| c.is_ascii_alphanumeric() || c == '_'),
                        _ => unreachable!(),
                    };
                    for len in 0..=4 {
                        for ordinal in 0..alphabet.len().pow(len) {
                            let units: Vec<_> = (0..len)
                                .map(|j| alphabet[ordinal / alphabet.len().pow(j) % alphabet.len()])
                                .collect();
                            let mut points = Vec::new();
                            let mut bounds = vec![0];
                            let mut j = 0;
                            while j < units.len() {
                                let mut c = u32::from(units[j]);
                                j += 1;
                                if (0xd800..=0xdbff).contains(&c)
                                    && j < units.len()
                                    && (0xdc00..=0xdfff).contains(&units[j])
                                {
                                    c = 65536 + (c - 0xd800) * 1024 + u32::from(units[j]) - 0xdc00;
                                    j += 1;
                                }
                                points.push(c);
                                bounds.push(j);
                            }
                            let input = JsString::from_code_units(units);
                            for start in 0..=input.len() + 1 {
                                for sticky in [false, true] {
                                    for (case, matcher) in CASES.iter().zip(&plans) {
                                        let expected = if start > input.len() {
                                            None
                                        } else {
                                            let first = bounds.partition_point(|&b| b <= start) - 1;
                                            (first..=points.len()).find_map(|at| {
                                                if (sticky && at != first)
                                                    || (case.6
                                                        && at != 0
                                                        && !(m && lt(points[at - 1])))
                                                {
                                                    return None;
                                                }
                                                let available = points[at..]
                                                    .iter()
                                                    .take_while(|&&c| member(case.1, c))
                                                    .count();
                                                let upper =
                                                    available.min(case.3.unwrap_or(available));
                                                if upper < case.2 {
                                                    return None;
                                                }
                                                let endpoint = |n: usize| {
                                                    let end = at + n;
                                                    if case.7
                                                        && end != points.len()
                                                        && !(m && lt(points[end]))
                                                    {
                                                        return None;
                                                    }
                                                    Some(RegExpUnicodeRepeatedAtomMatch {
                                                        range: bounds[at]..bounds[end],
                                                        atom_capture: (case.5 != 0 && n != 0)
                                                            .then(|| bounds[end - 1]..bounds[end]),
                                                    })
                                                };
                                                let mut counts = case.2..=upper;
                                                if case.4 {
                                                    counts.rev().find_map(endpoint)
                                                } else {
                                                    counts.find_map(endpoint)
                                                }
                                            })
                                        };
                                        assert_eq!(matcher.capture_count(), case.5, "{}", case.0);
                                        assert_eq!(
                                            matcher
                                                .find_with_captures_and_work(
                                                    &input,
                                                    start,
                                                    sticky,
                                                    |_| Ok::<(), ()>(())
                                                )
                                                .unwrap(),
                                            expected,
                                            "{} i={i} m={m} s={s} {input:?} start={start} sticky={sticky}",
                                            case.0
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

    #[test]
    fn unicode_captured_atoms_large_wrappers_charges_clones_and_scope_contracts() {
        let source = format!("{}😀{}+", "(".repeat(3000), ")".repeat(3000));
        let matcher = plan(&source, true, false, false, false).clone();
        let input = JsString::from("😀".repeat(100000).as_str());
        assert_eq!(matcher.capture_count(), 3000);
        let mut work = 0;
        assert_eq!(
            matcher
                .find_with_captures_and_work(&input, 0, false, |n| {
                    work += n;
                    Ok::<(), ()>(())
                })
                .unwrap(),
            Some(RegExpUnicodeRepeatedAtomMatch {
                range: 0..200000,
                atom_capture: Some(199998..200000)
            })
        );
        assert!(work < 6 * input.len(), "{work}");
        let matcher = plan("((?:a)){2,3}$", false, false, true, false);
        let mut calls = 0;
        matcher
            .find_with_captures_and_work(&JsString::from("aaa\n"), 0, false, |_| {
                calls += 1;
                Ok::<(), ()>(())
            })
            .unwrap();
        for fail in 1..=calls {
            let mut n = 0;
            assert!(
                matcher
                    .find_with_captures_and_work(&JsString::from("aaa\n"), 0, false, |_| {
                        n += 1;
                        if n == fail { Err(()) } else { Ok(()) }
                    })
                    .is_err()
            );
        }
        for source in [
            "(^a)+",
            "(a$)+",
            "(a?)+",
            "(a|b)+",
            "()+",
            "(ab)+",
            "(?:a)(a)+",
            "(a)+b",
            "(?i:a)+",
            r"(\b.)+",
            r"(\b)+",
            r"(\B)+",
        ] {
            assert!(
                RegExpUnicodeRepeatedAtomMatcher::compile_capturing_atom_with_work(
                    &JsString::from(source),
                    true,
                    false,
                    true,
                    false,
                    |_| Ok::<(), ()>(())
                )
                .unwrap()
                .is_none(),
                "{source}"
            );
        }
        assert!(
            RegExpUnicodeRepeatedAtomMatcher::compile_with_flags_and_work(
                &JsString::from("(a)+"),
                true,
                false,
                false,
                |_| Ok::<(), ()>(())
            )
            .unwrap()
            .is_none()
        );
    }
}

// The outer Option distinguishes malformed digits from a storage-sized overflow.
fn decimal(units: &[u16]) -> Option<Option<usize>> {
    if units.is_empty() || !units.iter().all(|u| (0x30..=0x39).contains(u)) {
        return None;
    }
    let mut n = Some(0usize);
    for &unit in units {
        n = n.and_then(|n| n.checked_mul(10)?.checked_add(usize::from(unit - 0x30)));
    }
    Some(n)
}

// Both ASCII decimal slices have validated ordering. Retain their exact
// difference when it fits usize; overflowing positive terms cannot cancel.
// Borrow and positional weight use constant storage, including long zero runs.
fn decimal_difference(first: &[u16], last: &[u16]) -> Option<usize> {
    let mut lower = first.iter().rev();
    let mut borrow = 0;
    let mut weight = Some(1usize);
    let mut value = 0usize;
    for &unit in last.iter().rev() {
        let high = usize::from(unit - 0x30);
        let low = lower.next().map_or(0, |&unit| usize::from(unit - 0x30)) + borrow;
        let digit = if high < low {
            borrow = 1;
            high + 10 - low
        } else {
            borrow = 0;
            high - low
        };
        if digit != 0 {
            value = value.checked_add(weight?.checked_mul(digit)?)?;
        }
        weight = weight.and_then(|weight| weight.checked_mul(10));
    }
    debug_assert!(borrow == 0 && lower.next().is_none());
    Some(value)
}

fn significant(units: &[u16]) -> &[u16] {
    &units[units.iter().position(|&u| u != 0x30).unwrap_or(units.len())..]
}

fn quantifier(units: &[u16], end: usize) -> Option<(usize, Option<usize>, Option<usize>)> {
    let last = end.checked_sub(1)?;
    if escaped(units, last) {
        return None;
    }
    match units[last] {
        0x2a => Some((last, Some(0), None)),
        0x2b => Some((last, Some(1), None)),
        0x3f => Some((last, Some(0), Some(1))),
        0x7d => {
            let open = units[..last].iter().rposition(|&u| u == 0x7b)?;
            if escaped(units, open) {
                return None;
            }
            let contents = &units[open + 1..last];
            if let Some(comma) = contents.iter().position(|&u| u == 0x2c) {
                let lower = &contents[..comma];
                let upper = &contents[comma + 1..];
                let minimum = decimal(lower)?;
                let maximum = if upper.is_empty() {
                    None
                } else {
                    let maximum = decimal(upper)?;
                    let (lower, upper) = (significant(lower), significant(upper));
                    if lower.len() > upper.len() || (lower.len() == upper.len() && lower > upper) {
                        return None;
                    }
                    maximum
                };
                Some((open, minimum, maximum))
            } else {
                let exact = decimal(contents)?;
                Some((open, exact, exact))
            }
        }
        _ => None,
    }
}

// A complete effect-free noncapturing group proof. The caller charges the
// source scan before entering; validated decimal counts are never expanded.
pub(crate) fn pure_empty_groups(branch: &[u16], quantified: bool, choices: bool) -> bool {
    let mut at = 0;
    let mut depth = 0usize;
    while at < branch.len() {
        if branch.get(at..at + 3) == Some(&[0x28, 0x3f, 0x3a]) {
            depth += 1;
            at += 3;
        } else if branch[at] == 0x29 {
            let Some(previous) = depth.checked_sub(1) else {
                return false;
            };
            depth = previous;
            at += 1;
            if quantified {
                // Only a completed, effect-free empty group can
                // own a quantifier. No count is expanded.
                let end = match branch.get(at) {
                    Some(0x2a | 0x2b | 0x3f) => at + 1,
                    Some(0x7b) => {
                        let Some(close) = branch[at + 1..].iter().position(|&c| c == 0x7d) else {
                            return false;
                        };
                        at + 2 + close
                    }
                    _ => continue,
                };
                let Some((start, _, _)) = quantifier(&branch[at..end], end - at) else {
                    return false;
                };
                if start != 0 {
                    return false;
                }
                at = end;
                if branch.get(at) == Some(&0x3f) {
                    at += 1;
                }
            }
        } else if choices && branch[at] == 0x7c && depth != 0 {
            // Every complete alternative has the same empty,
            // effect-free result; no path is expanded.
            at += 1;
        } else {
            return false;
        }
    }
    depth == 0
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fmt::Write;

    fn plan(source: &str, v: bool, i: bool, s: bool) -> RegExpUnicodeRepeatedAtomMatcher {
        RegExpUnicodeRepeatedAtomMatcher::compile_with_flags_and_work(
            &JsString::from(source),
            v,
            i,
            s,
            |_| Ok::<(), ()>(()),
        )
        .unwrap()
        .unwrap()
    }

    #[test]
    fn unicode_repeated_atom_original_ranges_snapshot() {
        let mut rows = String::new();
        for source in [
            r"a+",
            r"a*?",
            r"a??",
            r"a{2,3}",
            r"a{002,03}?",
            r".+",
            r"\D{1,2}",
            r"\W+",
            r"\p{Join_C}+",
            r"\p{NChar}{1,2}",
            r"\u{61}?",
            r"\??",
            r"a{0,0}",
            r"[aſK]+",
        ] {
            for v in [false, true] {
                for i in [false, true] {
                    for s in [false, true] {
                        let matcher = plan(source, v, i, s);
                        for input in [
                            JsString::from("aaa AA1 ſK\n"),
                            JsString::from("A\r\n\u{200c}\u{200d}\u{ffff}\u{10ffff}"),
                            JsString::from("😀??a"),
                            JsString::from_code_units(vec![0x61, 0xd800, 0x61, 0xdc00]),
                            JsString::from(""),
                        ] {
                            for start in [0, 1, 2, input.len(), input.len() + 1] {
                                for sticky in [false, true] {
                                    let found = matcher
                                        .find_with_work(&input, start, sticky, |_| Ok::<(), ()>(()))
                                        .unwrap();
                                    writeln!(rows,"{:?} v={v} i={i} s={s} input={input:?} start={start} sticky={sticky} {found:?}",JsString::from(source)).unwrap();
                                }
                            }
                        }
                    }
                }
            }
        }
        insta::assert_snapshot!(rows);
    }

    #[test]
    fn unicode_repeated_atoms_agree_with_independent_candidate_and_count_oracle() {
        type Case = (&'static str, u8, usize, Option<usize>, bool);
        let cases: &[Case] = &[
            ("a+", 0, 1, None, true),
            ("a*?", 0, 0, None, false),
            ("a??", 0, 0, Some(1), false),
            ("a{2,3}", 0, 2, Some(3), true),
            ("a{002,03}?", 0, 2, Some(3), false),
            (".+", 1, 1, None, true),
            (r"\D{1,2}", 2, 1, Some(2), true),
            (r"\W+", 3, 1, None, true),
            (r"\p{Join_C}+", 4, 1, None, true),
            (r"\p{NChar}{1,2}", 5, 1, Some(2), true),
            (r"\u{61}?", 0, 0, Some(1), true),
            (r"\??", 6, 0, Some(1), true),
            ("a{0,0}", 0, 0, Some(0), true),
            ("[aſK]+", 7, 1, None, true),
        ];
        let alphabet = [
            0x41, 0x61, 0x31, 10, 0x17f, 0x212a, 0x200c, 0xffff, 0xd83d, 0xde00,
        ];
        for i in [false, true] {
            for s in [false, true] {
                let fold = |c| {
                    if i {
                        match c {
                            0x41 => 0x61,
                            0x17f => 0x73,
                            0x212a => 0x6b,
                            _ => c,
                        }
                    } else {
                        c
                    }
                };
                let wanted = |kind, cp| match kind {
                    0 => fold(cp) == 0x61,
                    1 => s || ![10, 13, 0x2028, 0x2029].contains(&cp),
                    2 => !(0x30..=0x39).contains(&cp),
                    3 => !char::from_u32(fold(cp))
                        .is_some_and(|c| c.is_ascii_alphanumeric() || c == '_'),
                    4 => (0x200c..=0x200d).contains(&cp),
                    5 => matches!(cp, 0xfdd0..=0xfdef) || (cp & 0xffff) >= 0xfffe,
                    6 => cp == 0x3f,
                    7 => [0x61, fold(0x17f), fold(0x212a)].contains(&fold(cp)),
                    _ => unreachable!(),
                };
                let plans: Vec<_> = cases.iter().map(|c| plan(c.0, false, i, s)).collect();
                for len in 0..=4 {
                    for ordinal in 0..alphabet.len().pow(len) {
                        let units: Vec<_> = (0..len)
                            .map(|j| alphabet[ordinal / alphabet.len().pow(j) % alphabet.len()])
                            .collect();
                        let mut points = Vec::new();
                        let mut offsets = vec![0];
                        let mut j = 0;
                        while j < units.len() {
                            let mut cp = u32::from(units[j]);
                            j += 1;
                            if (0xd800..=0xdbff).contains(&cp)
                                && j < units.len()
                                && (0xdc00..=0xdfff).contains(&units[j])
                            {
                                cp = 65536 + (cp - 0xd800) * 1024 + u32::from(units[j]) - 0xdc00;
                                j += 1;
                            }
                            points.push(cp);
                            offsets.push(j);
                        }
                        let input = JsString::from_code_units(units);
                        for start in 0..=input.len() + 1 {
                            for sticky in [false, true] {
                                for (case, matcher) in cases.iter().zip(&plans) {
                                    let expected = if start > input.len() {
                                        None
                                    } else {
                                        let first = offsets.partition_point(|&n| n <= start) - 1;
                                        (first..=points.len()).find_map(|at| {
                                            if sticky && at != first {
                                                return None;
                                            }
                                            let available = points[at..]
                                                .iter()
                                                .take_while(|&&cp| wanted(case.1, cp))
                                                .count();
                                            let maximum =
                                                available.min(case.3.unwrap_or(available));
                                            if maximum < case.2 {
                                                return None;
                                            }
                                            let count = if case.4 { maximum } else { case.2 };
                                            Some(offsets[at]..offsets[at + count])
                                        })
                                    };
                                    assert_eq!(
                                        matcher
                                            .find_with_work(
                                                &input,
                                                start,
                                                sticky,
                                                |_| Ok::<(), ()>(())
                                            )
                                            .unwrap(),
                                        expected,
                                        "{} i={i} s={s} {input:?} start={start} sticky={sticky}",
                                        case.0
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn unicode_repeated_atoms_huge_counts_linear_work_clones_and_complete_contracts() {
        let huge = "9".repeat(200);
        let minimum = plan(&format!("a{{{huge}}}"), true, true, false);
        assert_eq!(
            minimum
                .find_with_work(&JsString::from("aaa"), 0, false, |_| Ok::<(), ()>(()))
                .unwrap(),
            None
        );
        assert_eq!(
            plan(&format!("a{{0,{huge}}}"), false, false, false)
                .find_with_work(&JsString::from("aaa"), 0, false, |_| Ok::<(), ()>(()))
                .unwrap(),
            Some(0..3)
        );
        let source = format!("a{{{}2}}", "0".repeat(100000));
        assert_eq!(
            plan(&source, false, false, false)
                .find_with_work(&JsString::from("aaa"), 0, false, |_| Ok::<(), ()>(()))
                .unwrap(),
            Some(0..2)
        );
        let input = JsString::from("aaB".repeat(50000).as_str());
        let mut work = 0;
        assert_eq!(
            plan("a{3}", false, false, false)
                .clone()
                .find_with_work(&input, 0, false, |n| {
                    work += n;
                    Ok::<(), ()>(())
                })
                .unwrap(),
            None
        );
        assert!(work < 3 * input.len(), "{work}");
        assert!(
            plan(".+", true, false, true)
                .find_with_work(&input, 0, false, |_| Err(()))
                .is_err()
        );
        assert!(
            RegExpUnicodeRepeatedAtomMatcher::compile_with_flags_and_work(
                &JsString::from("a+"),
                true,
                true,
                false,
                |_| Err(())
            )
            .is_err()
        );
        {
            let source = r"\p{Script_Extensions=Latin}+";
            let plan = RegExpUnicodeRepeatedAtomMatcher::compile_with_flags_and_work(
                &JsString::from(source),
                true,
                true,
                false,
                |_| Ok::<(), ()>(()),
            )
            .unwrap()
            .unwrap();
            assert_eq!(
                plan.find_with_work(&JsString::from("AA"), 0, true, |_| Ok::<(), ()>(()))
                    .unwrap(),
                Some(0..2)
            );
            assert_eq!(
                plan.find_with_work(&JsString::from("א"), 0, true, |_| Ok::<(), ()>(()))
                    .unwrap(),
                None
            );
        }
        for source in [
            "a",
            "ab+",
            "(a)+",
            "^a+",
            "a+$",
            r"\ba+",
            r"[a--b]+",
            r"[\q{ab}]+",
            "a{2,1}",
        ] {
            assert!(
                RegExpUnicodeRepeatedAtomMatcher::compile_with_flags_and_work(
                    &JsString::from(source),
                    true,
                    true,
                    false,
                    |_| Ok::<(), ()>(())
                )
                .unwrap()
                .is_none(),
                "{source}"
            );
        }
    }
}

#[cfg(test)]
mod multiline_end_assertion_tests {
    use super::*;
    use std::fmt::Write;
    const CASES: [(&str, u8, usize, Option<usize>, bool); 18] = [
        ("a+$", 0, 1, None, true),
        ("a*?$", 0, 0, None, false),
        ("a??$", 0, 0, Some(1), false),
        ("a{2,3}$", 0, 2, Some(3), true),
        ("a{002,03}?$", 0, 2, Some(3), false),
        (".+$", 1, 1, None, true),
        (r"\D{1,2}$", 2, 1, Some(2), true),
        (r"\W+$", 3, 1, None, true),
        (r"\p{Join_C}+$", 4, 1, None, true),
        (r"\p{NChar}{1,2}$", 5, 1, Some(2), true),
        (r"\u{61}?$", 0, 0, Some(1), true),
        (r"\??$", 6, 0, Some(1), true),
        ("a{0,0}$", 0, 0, Some(0), true),
        ("[aſK]+$", 7, 1, None, true),
        (".+?$", 1, 1, None, false),
        (r"\D+?$", 2, 1, None, false),
        (r"\D{1,3}?$", 2, 1, Some(3), false),
        ("a{1,3}?$", 0, 1, Some(3), false),
    ];
    fn plan(source: &str, v: bool, i: bool, s: bool) -> RegExpUnicodeRepeatedAtomMatcher {
        RegExpUnicodeRepeatedAtomMatcher::compile_with_line_anchors_and_work(
            &JsString::from(source),
            v,
            i,
            true,
            s,
            |_| Ok::<(), ()>(()),
        )
        .unwrap()
        .unwrap()
    }
    #[test]
    fn unicode_repeated_atom_multiline_end_ranges_snapshot() {
        let mut rows = String::new();
        for (source, ..) in CASES {
            for begin in [false, true] {
                let source = format!("{}{source}", if begin { "^" } else { "" });
                let pattern = JsString::from(source.as_str());
                for v in [false, true] {
                    for i in [false, true] {
                        for s in [false, true] {
                            let matcher = plan(&source, v, i, s);
                            for input in [
                                JsString::from("Aaaa\nAA"),
                                JsString::from("A\r\n\u{200c}\u{200d}\u{ffff}\u{10ffff}"),
                                JsString::from("😀??aaaſK"),
                                JsString::from_code_units(vec![0x61, 0xd800, 0x61, 0xdc00]),
                                JsString::from(""),
                            ] {
                                for start in [0, 1, 2, input.len(), input.len() + 1] {
                                    for sticky in [false, true] {
                                        writeln!(rows,"{pattern:?} v={v} i={i} s={s} input={input:?} start={start} sticky={sticky} {:?}",matcher.find_with_work(&input,start,sticky,|_|Ok::<(),()>(())).unwrap()).unwrap();
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        insta::assert_snapshot!(rows);
    }
    #[test]
    fn unicode_repeated_atom_multiline_end_agrees_with_independent_start_and_count_oracle() {
        let alphabet: [u16; 10] = [
            0x41, 0x61, 0x0a, 0x17f, 0x212a, 0x200c, 0xffff, 0x3f, 0xd83d, 0xde00,
        ];
        for i in [false, true] {
            for s in [false, true] {
                let fold = |c| {
                    if i {
                        match c {
                            0x41 => 0x61,
                            0x17f => 0x73,
                            0x212a => 0x6b,
                            _ => c,
                        }
                    } else {
                        c
                    }
                };
                let member = |kind, c| match kind {
                    0 => fold(c) == 0x61,
                    1 => s || ![10, 13, 0x2028, 0x2029].contains(&c),
                    2 => !(0x30..=0x39).contains(&c),
                    3 => !char::from_u32(fold(c))
                        .is_some_and(|c| c.is_ascii_alphanumeric() || c == '_'),
                    4 => (0x200c..=0x200d).contains(&c),
                    5 => matches!(c, 0xfdd0..=0xfdef) || (c & 0xffff) >= 0xfffe,
                    6 => c == 0x3f,
                    7 => [0x61, fold(0x17f), fold(0x212a)].contains(&fold(c)),
                    _ => unreachable!(),
                };
                let plans: Vec<_> = CASES
                    .iter()
                    .flat_map(|c| {
                        [
                            plan(c.0, false, i, s),
                            plan(&format!("^{}", c.0), false, i, s),
                        ]
                    })
                    .collect();
                for len in 0..=4 {
                    for ordinal in 0..alphabet.len().pow(len) {
                        let units: Vec<_> = (0..len)
                            .map(|j| alphabet[ordinal / alphabet.len().pow(j) % alphabet.len()])
                            .collect();
                        let mut points = Vec::new();
                        let mut bounds = vec![0];
                        let mut j = 0;
                        while j < units.len() {
                            let mut c = u32::from(units[j]);
                            j += 1;
                            if (0xd800..=0xdbff).contains(&c)
                                && j < units.len()
                                && (0xdc00..=0xdfff).contains(&units[j])
                            {
                                c = 65536 + (c - 0xd800) * 1024 + u32::from(units[j]) - 0xdc00;
                                j += 1;
                            }
                            points.push(c);
                            bounds.push(j);
                        }
                        let input = JsString::from_code_units(units);
                        for start in 0..=input.len() + 1 {
                            for sticky in [false, true] {
                                for (ordinal, matcher) in plans.iter().enumerate() {
                                    let case = &CASES[ordinal / 2];
                                    let begin = ordinal % 2 != 0;
                                    let expected = if start > input.len() {
                                        None
                                    } else {
                                        let first = bounds.partition_point(|&b| b <= start) - 1;
                                        (first..=points.len()).find_map(|at| {
                                            if (sticky && at != first)
                                                || (begin
                                                    && at != 0
                                                    && ![10, 13, 0x2028, 0x2029]
                                                        .contains(&points[at - 1]))
                                            {
                                                return None;
                                            }
                                            let upper = (points.len() - at)
                                                .min(case.3.unwrap_or(usize::MAX));
                                            if upper < case.2 {
                                                return None;
                                            }
                                            let endpoint = |count: usize| {
                                                let end = at + count;
                                                (points[at..end].iter().all(|&c| member(case.1, c))
                                                    && (end == points.len()
                                                        || [10, 13, 0x2028, 0x2029]
                                                            .contains(&points[end])))
                                                .then_some(bounds[at]..bounds[end])
                                            };
                                            let counts = case.2..=upper;
                                            if case.4 {
                                                counts.rev().find_map(endpoint)
                                            } else {
                                                counts.into_iter().find_map(endpoint)
                                            }
                                        })
                                    };
                                    assert_eq!(
                                        matcher
                                            .find_with_work(
                                                &input,
                                                start,
                                                sticky,
                                                |_| Ok::<(), ()>(())
                                            )
                                            .unwrap(),
                                        expected,
                                        "{} begin={begin} i={i} s={s} {input:?} start={start} sticky={sticky}",
                                        case.0
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    #[test]
    fn unicode_repeated_atom_multiline_end_work_bounds_clone_and_complete_contracts() {
        let input = JsString::from("aaB\n".repeat(50000).as_str());
        let mut work = 0;
        assert_eq!(
            plan("^a{3}$", true, false, false)
                .clone()
                .find_with_work(&input, 0, false, |n| {
                    work += n;
                    Ok::<(), ()>(())
                })
                .unwrap(),
            None
        );
        assert!(work < 12 * input.len(), "{work}");
        let input = JsString::from("aaaa\n".repeat(40000).as_str());
        for (pattern, expected) in [
            (r"\D+$", Some(0..input.len())),
            (r"\D+?$", Some(0..4)),
            (r"\D{1,2}$", Some(2..4)),
        ] {
            let mut work = 0;
            assert_eq!(
                plan(pattern, false, false, false)
                    .find_with_work(&input, 0, false, |n| {
                        work += n;
                        Ok::<(), ()>(())
                    })
                    .unwrap(),
                expected
            );
            assert!(work < 12 * input.len(), "{work}");
        }
        assert_eq!(
            plan("^a{0}$", false, false, false)
                .find_with_work(&JsString::from("\r\n"), 1, true, |_| Ok::<(), ()>(()))
                .unwrap(),
            Some(1..1)
        );
        assert_eq!(
            plan("a{0}$", false, false, false)
                .find_with_work(&JsString::from("😀\n"), 1, true, |_| Ok::<(), ()>(()))
                .unwrap(),
            None
        );
        assert_eq!(
            plan(".+$", true, false, false)
                .find_with_work(&JsString::from("😀\n"), 1, true, |_| Ok::<(), ()>(()))
                .unwrap(),
            Some(0..2)
        );
        let matcher = plan(r"\D+$", false, false, false);
        let mut calls = 0;
        matcher
            .find_with_work(&JsString::from("a\na"), 0, false, |_| {
                calls += 1;
                Ok::<(), ()>(())
            })
            .unwrap();
        for fail in 1..=calls {
            let mut n = 0;
            assert!(
                matcher
                    .find_with_work(&JsString::from("a\na"), 0, false, |_| {
                        n += 1;
                        if n == fail { Err(()) } else { Ok(()) }
                    })
                    .is_err()
            );
        }
        for source in ["a+b$", "(a)+$", "a+$$", r"\ba+$", r"[a--b]+$"] {
            assert!(
                RegExpUnicodeRepeatedAtomMatcher::compile_with_line_anchors_and_work(
                    &JsString::from(source),
                    true,
                    false,
                    true,
                    false,
                    |_| Ok::<(), ()>(())
                )
                .unwrap()
                .is_none(),
                "{source}"
            );
        }
        assert!(
            RegExpUnicodeRepeatedAtomMatcher::compile_with_flags_and_work(
                &JsString::from("a+$"),
                true,
                false,
                false,
                |_| Ok::<(), ()>(())
            )
            .unwrap()
            .is_none()
        );
    }
}
#[cfg(test)]
mod start_word_tests {
    use super::*;
    use std::fmt::Write;

    type Case = (
        &'static str,
        u8,
        usize,
        Option<usize>,
        bool,
        usize,
        bool,
        bool,
    );
    const CASES: [Case; 16] = [
        (r"\b(a)+", 0, 1, None, true, 1, false, false),
        (r"\B((a)){0,3}?", 0, 0, Some(3), false, 2, false, false),
        (r"\b(?:(a)){2,4}", 0, 2, Some(4), true, 1, false, false),
        (r"\B(?:a)+", 0, 1, None, true, 0, false, false),
        (r"\B((?:a)){0}", 0, 0, Some(0), true, 1, false, false),
        (r"\b([aſK])+", 1, 1, None, true, 1, false, false),
        (r"\B((.)){1,3}$", 2, 1, Some(3), true, 2, false, true),
        (r"^\b((\D)){0,2}?$", 3, 0, Some(2), false, 2, true, true),
        (r"^\b(\p{Join_C})+$", 4, 1, None, true, 1, true, true),
        (r"\B(\p{NChar}){1,2}$", 5, 1, Some(2), true, 1, false, true),
        (r"\b(😀)+", 6, 1, None, true, 1, false, false),
        (r"\B((\uD800)){0,1}", 7, 0, Some(1), true, 2, false, false),
        (r"\b(\?)??", 8, 0, Some(1), false, 1, false, false),
        (r"\B(?:(\W))+", 9, 1, None, true, 1, false, false),
        (r"\b(\u{61})?", 0, 0, Some(1), true, 1, false, false),
        (r"\B((a)){1,3}$", 0, 1, Some(3), true, 2, false, true),
    ];
    fn plan(source: &str, v: bool, i: bool, m: bool, s: bool) -> RegExpUnicodeRepeatedAtomMatcher {
        RegExpUnicodeRepeatedAtomMatcher::compile_with_start_word_boundary_and_work(
            &JsString::from(source),
            v,
            i,
            m,
            s,
            |_| Ok::<(), ()>(()),
        )
        .unwrap()
        .unwrap()
    }

    #[test]
    fn unicode_repeated_atom_start_word_ranges_snapshot() {
        let mut rows = String::new();
        for (source, ..) in CASES {
            let pattern = JsString::from(source);
            for v in [false, true] {
                for i in [false, true] {
                    for m in [false, true] {
                        for s in [false, true] {
                            let matcher = plan(source, v, i, m, s);
                            for input in [
                                JsString::from("Aaaa\nAA"),
                                JsString::from("A\r\n\u{200c}\u{200d}\u{ffff}\u{10ffff}"),
                                JsString::from("😀??aaſK"),
                                JsString::from_code_units(vec![0x61, 0xd800, 0x61, 0xdc00]),
                                JsString::from(""),
                            ] {
                                for start in [0, 1, input.len(), input.len() + 1] {
                                    for sticky in [false, true] {
                                        let found = matcher
                                            .find_with_captures_and_work(
                                                &input,
                                                start,
                                                sticky,
                                                |_| Ok::<(), ()>(()),
                                            )
                                            .unwrap();
                                        writeln!(rows,"{pattern:?} v={v} i={i} m={m} s={s} input={input:?} start={start} sticky={sticky} count={} range={:?} capture={:?}",matcher.capture_count(),found.as_ref().map(|f|f.range.clone()),found.and_then(|f|f.atom_capture)).unwrap();
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        insta::assert_snapshot!(rows);
    }

    #[test]
    fn unicode_repeated_atom_start_words_agree_with_independent_counts_and_final_character() {
        let alphabet: [u16; 10] = [
            0x41, 0x61, 10, 13, 0x17f, 0x212a, 0xd83d, 0xde00, 0xd800, 0xdc00,
        ];
        let lt = |c| [10, 13, 0x2028, 0x2029].contains(&c);
        for i in [false, true] {
            for m in [false, true] {
                for s in [false, true] {
                    let plans: Vec<_> = CASES
                        .iter()
                        .map(|c| (plan(c.0, true, i, m, s), c.0.contains(r"\b")))
                        .collect();
                    let fold = |c| {
                        if i {
                            match c {
                                0x41 => 0x61,
                                0x17f => 0x73,
                                0x212a => 0x6b,
                                _ => c,
                            }
                        } else {
                            c
                        }
                    };
                    let member = |kind, c| match kind {
                        0 => fold(c) == 0x61,
                        1 => [0x61, fold(0x17f), fold(0x212a)].contains(&fold(c)),
                        2 => s || !lt(c),
                        3 => !(0x30..=0x39).contains(&c),
                        4 => (0x200c..=0x200d).contains(&c),
                        5 => matches!(c, 0xfdd0..=0xfdef) || (c & 0xffff) >= 0xfffe,
                        6 => c == 0x1f600,
                        7 => c == 0xd800,
                        8 => c == 0x3f,
                        9 => !char::from_u32(fold(c))
                            .is_some_and(|c| c.is_ascii_alphanumeric() || c == '_'),
                        _ => unreachable!(),
                    };
                    for len in 0..=4 {
                        for ordinal in 0..alphabet.len().pow(len) {
                            let units: Vec<_> = (0..len)
                                .map(|j| alphabet[ordinal / alphabet.len().pow(j) % alphabet.len()])
                                .collect();
                            let mut points = Vec::new();
                            let mut bounds = vec![0];
                            let mut j = 0;
                            while j < units.len() {
                                let mut c = u32::from(units[j]);
                                j += 1;
                                if (0xd800..=0xdbff).contains(&c)
                                    && j < units.len()
                                    && (0xdc00..=0xdfff).contains(&units[j])
                                {
                                    c = 65536 + (c - 0xd800) * 1024 + u32::from(units[j]) - 0xdc00;
                                    j += 1;
                                }
                                points.push(c);
                                bounds.push(j);
                            }
                            let input = JsString::from_code_units(units);
                            for start in 0..=input.len() + 1 {
                                for sticky in [false, true] {
                                    for (case, (matcher, positive)) in CASES.iter().zip(&plans) {
                                        let expected = if start > input.len() {
                                            None
                                        } else {
                                            let first = bounds.partition_point(|&b| b <= start) - 1;
                                            (first..=points.len()).find_map(|at| {
                                                let word = |c: Option<&u32>| c.is_some_and(|&c| {
                                                    matches!(c, 0x30..=0x39 | 0x41..=0x5a | 0x5f | 0x61..=0x7a)
                                                        || (i && matches!(c, 0x17f | 0x212a))
                                                });
                                                let boundary = word(at.checked_sub(1).and_then(|p| points.get(p))) != word(points.get(at));
                                                if boundary != *positive || (sticky && at != first)
                                                    || (case.6
                                                        && at != 0
                                                        && !(m && lt(points[at - 1])))
                                                {
                                                    return None;
                                                }
                                                let available = points[at..]
                                                    .iter()
                                                    .take_while(|&&c| member(case.1, c))
                                                    .count();
                                                let upper =
                                                    available.min(case.3.unwrap_or(available));
                                                if upper < case.2 {
                                                    return None;
                                                }
                                                let endpoint = |n: usize| {
                                                    let end = at + n;
                                                    if case.7
                                                        && end != points.len()
                                                        && !(m && lt(points[end]))
                                                    {
                                                        return None;
                                                    }
                                                    Some(RegExpUnicodeRepeatedAtomMatch {
                                                        range: bounds[at]..bounds[end],
                                                        atom_capture: (case.5 != 0 && n != 0)
                                                            .then(|| bounds[end - 1]..bounds[end]),
                                                    })
                                                };
                                                let mut counts = case.2..=upper;
                                                if case.4 {
                                                    counts.rev().find_map(endpoint)
                                                } else {
                                                    counts.find_map(endpoint)
                                                }
                                            })
                                        };
                                        assert_eq!(matcher.capture_count(), case.5, "{}", case.0);
                                        assert_eq!(
                                            matcher
                                                .find_with_captures_and_work(
                                                    &input,
                                                    start,
                                                    sticky,
                                                    |_| Ok::<(), ()>(())
                                                )
                                                .unwrap(),
                                            expected,
                                            "{} i={i} m={m} s={s} {input:?} start={start} sticky={sticky}",
                                            case.0
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

    #[test]
    fn unicode_repeated_atom_start_word_linear_work_and_contracts() {
        let source = format!(r"\B{}😀{}+", "(".repeat(3000), ")".repeat(3000));
        let matcher = plan(&source, true, false, false, false).clone();
        let input = JsString::from("😀".repeat(100000).as_str());
        let mut work = 0;
        assert_eq!(matcher.capture_count(), 3000);
        assert_eq!(
            matcher
                .find_with_captures_and_work(&input, 0, false, |n| {
                    work += n;
                    Ok::<(), ()>(())
                })
                .unwrap(),
            Some(RegExpUnicodeRepeatedAtomMatch {
                range: 0..200000,
                atom_capture: Some(199998..200000)
            })
        );
        assert!(work < 40 * input.len(), "{work}");
        let input = JsString::from(format!("{}X", "a".repeat(200000)).as_str());
        let matcher = plan(r"\Ba{3}$", false, false, false, false);
        work = 0;
        assert_eq!(
            matcher
                .find_with_work(&input, 0, false, |n| {
                    work += n;
                    Ok::<(), ()>(())
                })
                .unwrap(),
            None
        );
        assert!(work < 40 * input.len(), "{work}");
        let matcher = plan(r"\b((a)){2,3}$", false, false, true, false);
        let input = JsString::from("aaa\n");
        let mut calls = 0;
        matcher
            .find_with_captures_and_work(&input, 0, false, |_| {
                calls += 1;
                Ok::<(), ()>(())
            })
            .unwrap();
        for fail in 1..=calls {
            let mut n = 0;
            assert!(
                matcher
                    .find_with_captures_and_work(&input, 0, false, |_| {
                        n += 1;
                        if n == fail { Err(()) } else { Ok(()) }
                    })
                    .is_err()
            );
        }
        for source in [
            r"a+",
            r"\ba+\b",
            r"a+\b",
            r"\b(^a)+",
            r"\b(a$)+",
            r"\b(a?)+",
            r"\b(a|b)+",
            r"\b()+",
            r"\b(ab)+",
            r"\b(a)+b",
            r"\b(?i:a)+",
            r"\b(\b.)+",
            r"\b(\B)+",
        ] {
            assert!(
                RegExpUnicodeRepeatedAtomMatcher::compile_with_start_word_boundary_and_work(
                    &JsString::from(source),
                    true,
                    false,
                    true,
                    false,
                    |_| Ok::<(), ()>(())
                )
                .unwrap()
                .is_none(),
                "{source}"
            );
        }
        for source in [r"\ba+", r"\B(a)+"] {
            assert!(
                RegExpUnicodeRepeatedAtomMatcher::compile_with_line_anchors_and_work(
                    &JsString::from(source),
                    true,
                    false,
                    true,
                    false,
                    |_| Ok::<(), ()>(())
                )
                .unwrap()
                .is_none()
            );
            assert!(
                RegExpUnicodeRepeatedAtomMatcher::compile_capturing_atom_with_work(
                    &JsString::from(source),
                    true,
                    false,
                    true,
                    false,
                    |_| Ok::<(), ()>(())
                )
                .unwrap()
                .is_none()
            );
        }
    }
}

#[cfg(test)]
mod word_boundary_tests {
    use super::*;
    use std::fmt::Write;

    type Case = (
        &'static str,
        u8,
        usize,
        Option<usize>,
        bool,
        usize,
        bool,
        bool,
        Option<bool>,
        bool,
    );
    const CASES: [Case; 16] = [
        (r"(a)+\b", 0, 1, None, true, 1, false, false, None, true),
        (
            r"\B((a)){0,3}?\B",
            0,
            0,
            Some(3),
            false,
            2,
            false,
            false,
            Some(false),
            false,
        ),
        (
            r"\b(?:(a)){2,4}\B",
            0,
            2,
            Some(4),
            true,
            1,
            false,
            false,
            Some(true),
            false,
        ),
        (r"(?:a)+\B", 0, 1, None, true, 0, false, false, None, false),
        (
            r"\B((?:a)){0}\b",
            0,
            0,
            Some(0),
            true,
            1,
            false,
            false,
            Some(false),
            true,
        ),
        (
            r"\b([aſK])+\b",
            1,
            1,
            None,
            true,
            1,
            false,
            false,
            Some(true),
            true,
        ),
        (
            r"((.)){1,3}\b$",
            2,
            1,
            Some(3),
            true,
            2,
            false,
            true,
            None,
            true,
        ),
        (
            r"^\b((\D)){0,2}?\B$",
            3,
            0,
            Some(2),
            false,
            2,
            true,
            true,
            Some(true),
            false,
        ),
        (
            r"^\b(\p{Join_C})+\B$",
            4,
            1,
            None,
            true,
            1,
            true,
            true,
            Some(true),
            false,
        ),
        (
            r"(\p{NChar}){1,2}\B$",
            5,
            1,
            Some(2),
            true,
            1,
            false,
            true,
            None,
            false,
        ),
        (r"(😀)+\B", 6, 1, None, true, 1, false, false, None, false),
        (
            r"((\uD800)){0,1}\b",
            7,
            0,
            Some(1),
            true,
            2,
            false,
            false,
            None,
            true,
        ),
        (
            r"(\?)??\B",
            8,
            0,
            Some(1),
            false,
            1,
            false,
            false,
            None,
            false,
        ),
        (
            r"(?:(\W))+\b",
            9,
            1,
            None,
            true,
            1,
            false,
            false,
            None,
            true,
        ),
        (
            r"\b(\u{61})?\b",
            0,
            0,
            Some(1),
            true,
            1,
            false,
            false,
            Some(true),
            true,
        ),
        (
            r"\B((a)){1,3}\b$",
            0,
            1,
            Some(3),
            true,
            2,
            false,
            true,
            Some(false),
            true,
        ),
    ];
    fn plan(source: &str, v: bool, i: bool, m: bool, s: bool) -> RegExpUnicodeRepeatedAtomMatcher {
        RegExpUnicodeRepeatedAtomMatcher::compile_with_word_boundaries_and_work(
            &JsString::from(source),
            v,
            i,
            m,
            s,
            |_| Ok::<(), ()>(()),
        )
        .unwrap()
        .unwrap()
    }

    #[test]
    fn unicode_repeated_atom_word_boundary_ranges_snapshot() {
        let mut rows = String::new();
        for (source, ..) in CASES {
            let pattern = JsString::from(source);
            for v in [false, true] {
                for i in [false, true] {
                    for m in [false, true] {
                        for s in [false, true] {
                            let matcher = plan(source, v, i, m, s);
                            for input in [
                                JsString::from("Aaaa\nAA"),
                                JsString::from("A\r\n\u{200c}\u{200d}\u{ffff}\u{10ffff}"),
                                JsString::from("😀??aaſK"),
                                JsString::from_code_units(vec![0x61, 0xd800, 0x61, 0xdc00]),
                                JsString::from(""),
                            ] {
                                for start in [0, 1, input.len(), input.len() + 1] {
                                    for sticky in [false, true] {
                                        let found = matcher
                                            .find_with_captures_and_work(
                                                &input,
                                                start,
                                                sticky,
                                                |_| Ok::<(), ()>(()),
                                            )
                                            .unwrap();
                                        writeln!(rows,"{pattern:?} v={v} i={i} m={m} s={s} input={input:?} start={start} sticky={sticky} count={} range={:?} capture={:?}",matcher.capture_count(),found.as_ref().map(|f|f.range.clone()),found.and_then(|f|f.atom_capture)).unwrap();
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        insta::assert_snapshot!(rows);
    }

    #[test]
    fn unicode_repeated_atom_word_boundaries_agree_with_independent_counts_and_final_character() {
        let alphabet: [u16; 10] = [
            0x41, 0x61, 10, 13, 0x17f, 0x212a, 0xd83d, 0xde00, 0xd800, 0xdc00,
        ];
        let lt = |c| [10, 13, 0x2028, 0x2029].contains(&c);
        for i in [false, true] {
            for m in [false, true] {
                for s in [false, true] {
                    let plans: Vec<_> = CASES.iter().map(|c| plan(c.0, true, i, m, s)).collect();
                    let fold = |c| {
                        if i {
                            match c {
                                0x41 => 0x61,
                                0x17f => 0x73,
                                0x212a => 0x6b,
                                _ => c,
                            }
                        } else {
                            c
                        }
                    };
                    let member = |kind, c| match kind {
                        0 => fold(c) == 0x61,
                        1 => [0x61, fold(0x17f), fold(0x212a)].contains(&fold(c)),
                        2 => s || !lt(c),
                        3 => !(0x30..=0x39).contains(&c),
                        4 => (0x200c..=0x200d).contains(&c),
                        5 => matches!(c, 0xfdd0..=0xfdef) || (c & 0xffff) >= 0xfffe,
                        6 => c == 0x1f600,
                        7 => c == 0xd800,
                        8 => c == 0x3f,
                        9 => !char::from_u32(fold(c))
                            .is_some_and(|c| c.is_ascii_alphanumeric() || c == '_'),
                        _ => unreachable!(),
                    };
                    for len in 0..=4 {
                        for ordinal in 0..alphabet.len().pow(len) {
                            let units: Vec<_> = (0..len)
                                .map(|j| alphabet[ordinal / alphabet.len().pow(j) % alphabet.len()])
                                .collect();
                            let mut points = Vec::new();
                            let mut bounds = vec![0];
                            let mut j = 0;
                            while j < units.len() {
                                let mut c = u32::from(units[j]);
                                j += 1;
                                if (0xd800..=0xdbff).contains(&c)
                                    && j < units.len()
                                    && (0xdc00..=0xdfff).contains(&units[j])
                                {
                                    c = 65536 + (c - 0xd800) * 1024 + u32::from(units[j]) - 0xdc00;
                                    j += 1;
                                }
                                points.push(c);
                                bounds.push(j);
                            }
                            let input = JsString::from_code_units(units);
                            for start in 0..=input.len() + 1 {
                                for sticky in [false, true] {
                                    for (case, matcher) in CASES.iter().zip(&plans) {
                                        let expected = if start > input.len() {
                                            None
                                        } else {
                                            let first = bounds.partition_point(|&b| b <= start) - 1;
                                            (first..=points.len()).find_map(|at| {
                                                let word = |c: Option<&u32>| c.is_some_and(|&c| {
                                                    matches!(c, 0x30..=0x39 | 0x41..=0x5a | 0x5f | 0x61..=0x7a)
                                                        || (i && matches!(c, 0x17f | 0x212a))
                                                });
                                                let boundary = word(at.checked_sub(1).and_then(|p| points.get(p))) != word(points.get(at));
                                                if case.8.is_some_and(|positive| boundary != positive) || (sticky && at != first)
                                                    || (case.6
                                                        && at != 0
                                                        && !(m && lt(points[at - 1])))
                                                {
                                                    return None;
                                                }
                                                let available = points[at..]
                                                    .iter()
                                                    .take_while(|&&c| member(case.1, c))
                                                    .count();
                                                let upper =
                                                    available.min(case.3.unwrap_or(available));
                                                if upper < case.2 {
                                                    return None;
                                                }
                                                let endpoint = |n: usize| {
                                                    let end = at + n;
                                                    if case.7
                                                        && end != points.len()
                                                        && !(m && lt(points[end]))
                                                    {
                                                        return None;
                                                    }
                                                    let boundary = word(end.checked_sub(1).and_then(|p| points.get(p))) != word(points.get(end));
                                                    if boundary != case.9 {return None;}
                                                    Some(RegExpUnicodeRepeatedAtomMatch {
                                                        range: bounds[at]..bounds[end],
                                                        atom_capture: (case.5 != 0 && n != 0)
                                                            .then(|| bounds[end - 1]..bounds[end]),
                                                    })
                                                };
                                                let mut counts = case.2..=upper;
                                                if case.4 {
                                                    counts.rev().find_map(endpoint)
                                                } else {
                                                    counts.find_map(endpoint)
                                                }
                                            })
                                        };
                                        assert_eq!(matcher.capture_count(), case.5, "{}", case.0);
                                        assert_eq!(
                                            matcher
                                                .find_with_captures_and_work(
                                                    &input,
                                                    start,
                                                    sticky,
                                                    |_| Ok::<(), ()>(())
                                                )
                                                .unwrap(),
                                            expected,
                                            "{} i={i} m={m} s={s} {input:?} start={start} sticky={sticky}",
                                            case.0
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

    #[test]
    fn unicode_repeated_atom_word_boundary_linear_work_and_contracts() {
        let source = format!(r"\B{}😀{}+\B", "(".repeat(3000), ")".repeat(3000));
        let matcher = plan(&source, true, false, false, false).clone();
        let input = JsString::from("😀".repeat(100000).as_str());
        let mut work = 0;
        assert_eq!(matcher.capture_count(), 3000);
        assert_eq!(
            matcher
                .find_with_captures_and_work(&input, 0, false, |n| {
                    work += n;
                    Ok::<(), ()>(())
                })
                .unwrap(),
            Some(RegExpUnicodeRepeatedAtomMatch {
                range: 0..200000,
                atom_capture: Some(199998..200000)
            })
        );
        assert!(work < 40 * input.len(), "{work}");
        let input = JsString::from(format!("{}X", "a".repeat(200000)).as_str());
        let matcher = plan(r"\Ba{3}\b$", false, false, false, false);
        work = 0;
        assert_eq!(
            matcher
                .find_with_work(&input, 0, false, |n| {
                    work += n;
                    Ok::<(), ()>(())
                })
                .unwrap(),
            None
        );
        assert!(work < 40 * input.len(), "{work}");
        let matcher = plan(r"\b((a)){2,3}\b$", false, false, true, false);
        let input = JsString::from("aaa\n");
        let mut calls = 0;
        matcher
            .find_with_captures_and_work(&input, 0, false, |_| {
                calls += 1;
                Ok::<(), ()>(())
            })
            .unwrap();
        for fail in 1..=calls {
            let mut n = 0;
            assert!(
                matcher
                    .find_with_captures_and_work(&input, 0, false, |_| {
                        n += 1;
                        if n == fail { Err(()) } else { Ok(()) }
                    })
                    .is_err()
            );
        }
        for source in [
            r"a+",
            r"\b(^a)+",
            r"\b(a$)+",
            r"\b(a?)+",
            r"\b(a|b)+",
            r"\b()+",
            r"\b(ab)+",
            r"\b(a)+b",
            r"\b(?i:a)+",
            r"\b(\b.)+",
            r"\b(\B)+",
        ] {
            assert!(
                RegExpUnicodeRepeatedAtomMatcher::compile_with_word_boundaries_and_work(
                    &JsString::from(source),
                    true,
                    false,
                    true,
                    false,
                    |_| Ok::<(), ()>(())
                )
                .unwrap()
                .is_none(),
                "{source}"
            );
        }
        for source in [r"\ba+", r"\B(a)+"] {
            assert!(
                RegExpUnicodeRepeatedAtomMatcher::compile_with_line_anchors_and_work(
                    &JsString::from(source),
                    true,
                    false,
                    true,
                    false,
                    |_| Ok::<(), ()>(())
                )
                .unwrap()
                .is_none()
            );
            assert!(
                RegExpUnicodeRepeatedAtomMatcher::compile_capturing_atom_with_work(
                    &JsString::from(source),
                    true,
                    false,
                    true,
                    false,
                    |_| Ok::<(), ()>(())
                )
                .unwrap()
                .is_none()
            );
        }
    }
}
#[cfg(test)]
mod inner_line_tests {
    use super::*;
    use std::fmt::Write;

    type Case = (
        &'static str,
        u8,
        usize,
        Option<usize>,
        bool,
        usize,
        bool,
        bool,
        bool,
        bool,
    );
    const CASES: [Case; 16] = [
        (r"(^a)+", 0, 1, None, true, 1, false, false, true, false),
        (
            r"((a$)){0,3}?",
            0,
            0,
            Some(3),
            false,
            2,
            false,
            false,
            false,
            true,
        ),
        (
            r"(?:(^a$)){2,4}",
            0,
            2,
            Some(4),
            true,
            1,
            false,
            false,
            true,
            true,
        ),
        (r"(?:^a)+", 0, 1, None, true, 0, false, false, true, false),
        (
            r"((?:a$)){0}",
            0,
            0,
            Some(0),
            true,
            1,
            false,
            false,
            false,
            true,
        ),
        (r"(^[aſK])+", 1, 1, None, true, 1, false, false, true, false),
        (
            r"((.$)){1,3}$",
            2,
            1,
            Some(3),
            true,
            2,
            false,
            true,
            false,
            true,
        ),
        (
            r"^((^\D$)){0,2}?$",
            3,
            0,
            Some(2),
            false,
            2,
            true,
            true,
            true,
            true,
        ),
        (
            r"^(^\p{Join_C})+$",
            4,
            1,
            None,
            true,
            1,
            true,
            true,
            true,
            false,
        ),
        (
            r"(\p{NChar}$){1,2}$",
            5,
            1,
            Some(2),
            true,
            1,
            false,
            true,
            false,
            true,
        ),
        (r"(^😀)+", 6, 1, None, true, 1, false, false, true, false),
        (
            r"((\uD800$)){0,1}",
            7,
            0,
            Some(1),
            true,
            2,
            false,
            false,
            false,
            true,
        ),
        (
            r"(^\?)??",
            8,
            0,
            Some(1),
            false,
            1,
            false,
            false,
            true,
            false,
        ),
        (
            r"(?:(\W$))+",
            9,
            1,
            None,
            true,
            1,
            false,
            false,
            false,
            true,
        ),
        (
            r"(^\u{61})?",
            0,
            0,
            Some(1),
            true,
            1,
            false,
            false,
            true,
            false,
        ),
        (
            r"((^a)){1,3}$",
            0,
            1,
            Some(3),
            true,
            2,
            false,
            true,
            true,
            false,
        ),
    ];
    fn plan(source: &str, v: bool, i: bool, m: bool, s: bool) -> RegExpUnicodeRepeatedAtomMatcher {
        RegExpUnicodeRepeatedAtomMatcher::compile_capturing_atom_with_line_assertions_and_work(
            &JsString::from(source),
            v,
            i,
            m,
            s,
            |_| Ok::<(), ()>(()),
        )
        .unwrap()
        .unwrap()
    }

    #[test]
    fn unicode_repeated_atom_inner_line_ranges_snapshot() {
        let mut rows = String::new();
        for (source, ..) in CASES {
            let pattern = JsString::from(source);
            for v in [false, true] {
                for i in [false, true] {
                    for m in [false, true] {
                        for s in [false, true] {
                            let matcher = plan(source, v, i, m, s);
                            for input in [
                                JsString::from("Aaaa\nAA"),
                                JsString::from("A\r\n\u{200c}\u{200d}\u{ffff}\u{10ffff}"),
                                JsString::from("😀??aaſK"),
                                JsString::from_code_units(vec![0x61, 0xd800, 0x61, 0xdc00]),
                                JsString::from(""),
                            ] {
                                for start in [0, 1, input.len(), input.len() + 1] {
                                    for sticky in [false, true] {
                                        let found = matcher
                                            .find_with_captures_and_work(
                                                &input,
                                                start,
                                                sticky,
                                                |_| Ok::<(), ()>(()),
                                            )
                                            .unwrap();
                                        writeln!(rows,"{pattern:?} v={v} i={i} m={m} s={s} input={input:?} start={start} sticky={sticky} count={} range={:?} capture={:?}",matcher.capture_count(),found.as_ref().map(|f|f.range.clone()),found.and_then(|f|f.atom_capture)).unwrap();
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        insta::assert_snapshot!(rows);
    }

    #[test]
    fn unicode_repeated_atom_inner_lines_agree_with_independent_counts_and_final_character() {
        let alphabet: [u16; 10] = [
            0x41, 0x61, 10, 13, 0x17f, 0x212a, 0xd83d, 0xde00, 0xd800, 0xdc00,
        ];
        let lt = |c| [10, 13, 0x2028, 0x2029].contains(&c);
        for i in [false, true] {
            for m in [false, true] {
                for s in [false, true] {
                    let plans: Vec<_> = CASES.iter().map(|c| plan(c.0, true, i, m, s)).collect();
                    let fold = |c| {
                        if i {
                            match c {
                                0x41 => 0x61,
                                0x17f => 0x73,
                                0x212a => 0x6b,
                                _ => c,
                            }
                        } else {
                            c
                        }
                    };
                    let member = |kind, c| match kind {
                        0 => fold(c) == 0x61,
                        1 => [0x61, fold(0x17f), fold(0x212a)].contains(&fold(c)),
                        2 => s || !lt(c),
                        3 => !(0x30..=0x39).contains(&c),
                        4 => (0x200c..=0x200d).contains(&c),
                        5 => matches!(c, 0xfdd0..=0xfdef) || (c & 0xffff) >= 0xfffe,
                        6 => c == 0x1f600,
                        7 => c == 0xd800,
                        8 => c == 0x3f,
                        9 => !char::from_u32(fold(c))
                            .is_some_and(|c| c.is_ascii_alphanumeric() || c == '_'),
                        _ => unreachable!(),
                    };
                    for len in 0..=4 {
                        for ordinal in 0..alphabet.len().pow(len) {
                            let units: Vec<_> = (0..len)
                                .map(|j| alphabet[ordinal / alphabet.len().pow(j) % alphabet.len()])
                                .collect();
                            let mut points = Vec::new();
                            let mut bounds = vec![0];
                            let mut j = 0;
                            while j < units.len() {
                                let mut c = u32::from(units[j]);
                                j += 1;
                                if (0xd800..=0xdbff).contains(&c)
                                    && j < units.len()
                                    && (0xdc00..=0xdfff).contains(&units[j])
                                {
                                    c = 65536 + (c - 0xd800) * 1024 + u32::from(units[j]) - 0xdc00;
                                    j += 1;
                                }
                                points.push(c);
                                bounds.push(j);
                            }
                            let input = JsString::from_code_units(units);
                            for start in 0..=input.len() + 1 {
                                for sticky in [false, true] {
                                    for (case, matcher) in CASES.iter().zip(&plans) {
                                        let expected = if start > input.len() {
                                            None
                                        } else {
                                            let first = bounds.partition_point(|&b| b <= start) - 1;
                                            (first..=points.len()).find_map(|at| {
                                                if (sticky && at != first)
                                                    || (case.6
                                                        && at != 0
                                                        && !(m && lt(points[at - 1])))
                                                {
                                                    return None;
                                                }
                                                let available = (at..points.len())
                                                    .take_while(|&p| {
                                                        member(case.1, points[p])
                                                            && (!case.8
                                                                || p == 0
                                                                || (m && lt(points[p - 1])))
                                                            && (!case.9
                                                                || p + 1 == points.len()
                                                                || (m && lt(points[p + 1])))
                                                    })
                                                    .count();
                                                let upper =
                                                    available.min(case.3.unwrap_or(available));
                                                if upper < case.2 {
                                                    return None;
                                                }
                                                let endpoint = |n: usize| {
                                                    let end = at + n;
                                                    if case.7
                                                        && end != points.len()
                                                        && !(m && lt(points[end]))
                                                    {
                                                        return None;
                                                    }
                                                    Some(RegExpUnicodeRepeatedAtomMatch {
                                                        range: bounds[at]..bounds[end],
                                                        atom_capture: (case.5 != 0 && n != 0)
                                                            .then(|| bounds[end - 1]..bounds[end]),
                                                    })
                                                };
                                                let mut counts = case.2..=upper;
                                                if case.4 {
                                                    counts.rev().find_map(endpoint)
                                                } else {
                                                    counts.find_map(endpoint)
                                                }
                                            })
                                        };
                                        assert_eq!(matcher.capture_count(), case.5, "{}", case.0);
                                        assert_eq!(
                                            matcher
                                                .find_with_captures_and_work(
                                                    &input,
                                                    start,
                                                    sticky,
                                                    |_| Ok::<(), ()>(())
                                                )
                                                .unwrap(),
                                            expected,
                                            "{} i={i} m={m} s={s} {input:?} start={start} sticky={sticky}",
                                            case.0
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

    #[test]
    fn unicode_repeated_atom_inner_line_work_clones_and_scope_contracts() {
        let source = format!("{}^.${}+", "(".repeat(3000), ")".repeat(3000));
        let matcher = plan(&source, true, false, true, true).clone();
        let input = JsString::from("\n".repeat(200000).as_str());
        let mut work = 0;
        assert_eq!(matcher.capture_count(), 3000);
        assert_eq!(
            matcher
                .find_with_captures_and_work(&input, 0, false, |n| {
                    work += n;
                    Ok::<(), ()>(())
                })
                .unwrap(),
            Some(RegExpUnicodeRepeatedAtomMatch {
                range: 0..200000,
                atom_capture: Some(199999..200000)
            })
        );
        assert!(work < 40 * input.len(), "{work}");
        let matcher = plan("(a$){2}", false, false, false, false);
        let input = JsString::from("a".repeat(200000).as_str());
        work = 0;
        assert_eq!(
            matcher
                .find_with_work(&input, 0, false, |n| {
                    work += n;
                    Ok::<(), ()>(())
                })
                .unwrap(),
            None
        );
        assert!(work < 40 * input.len(), "{work}");
        let matcher = plan("((^.$)){2,3}$", true, false, true, true);
        let input = JsString::from("\r\n");
        let mut calls = 0;
        matcher
            .find_with_captures_and_work(&input, 0, false, |_| {
                calls += 1;
                Ok::<(), ()>(())
            })
            .unwrap();
        for fail in 1..=calls {
            let mut n = 0;
            assert!(
                matcher
                    .find_with_captures_and_work(&input, 0, false, |_| {
                        n += 1;
                        if n == fail { Err(()) } else { Ok(()) }
                    })
                    .is_err()
            );
        }
        for source in [
            r"(^$)+",
            r"(a?)+",
            r"(a|b)+",
            r"()+",
            r"(ab)+",
            r"(^a)+b",
            r"(?i:a)+",
            r"(\b.)+",
            r"(\B)+",
            r"(^a(b))+",
            r"(a^)+",
            r"($a)+",
        ] {
            assert!(RegExpUnicodeRepeatedAtomMatcher::compile_capturing_atom_with_line_assertions_and_work(&JsString::from(source),true,false,true,true,|_|Ok::<(),()>(())).unwrap().is_none(),"{source}");
        }
        for source in ["(^a)+", "(a$)+", "((^.$))+", r"(^(?:a))+", r"(^\D$)+"] {
            assert!(
                RegExpUnicodeRepeatedAtomMatcher::compile_capturing_atom_with_work(
                    &JsString::from(source),
                    true,
                    false,
                    true,
                    true,
                    |_| Ok::<(), ()>(())
                )
                .unwrap()
                .is_none()
            );
        }
    }
}

#[cfg(test)]
mod scoped_word_tests {
    use super::*;
    use std::fmt::Write;

    type Case = (
        &'static str,
        u8,
        usize,
        Option<usize>,
        bool,
        usize,
        bool,
        bool,
        bool,
        bool,
        Option<bool>,
        Option<bool>,
    );
    const CASES: [Case; 16] = [
        (
            r"\b(^a)+\B",
            0,
            1,
            None,
            true,
            1,
            false,
            false,
            true,
            false,
            Some(true),
            Some(false),
        ),
        (
            r"\B((a$)){0,3}?",
            0,
            0,
            Some(3),
            false,
            2,
            false,
            false,
            false,
            true,
            Some(false),
            None,
        ),
        (
            r"\b(?:(^a$)){2,4}\b",
            0,
            2,
            Some(4),
            true,
            1,
            false,
            false,
            true,
            true,
            Some(true),
            Some(true),
        ),
        (
            r"(?:^a)+\b",
            0,
            1,
            None,
            true,
            0,
            false,
            false,
            true,
            false,
            None,
            Some(true),
        ),
        (
            r"\B((?:a$)){0}\B",
            0,
            0,
            Some(0),
            true,
            1,
            false,
            false,
            false,
            true,
            Some(false),
            Some(false),
        ),
        (
            r"\b(^[aſK])+\b",
            1,
            1,
            None,
            true,
            1,
            false,
            false,
            true,
            false,
            Some(true),
            Some(true),
        ),
        (
            r"((.$)){1,3}\b$",
            2,
            1,
            Some(3),
            true,
            2,
            false,
            true,
            false,
            true,
            None,
            Some(true),
        ),
        (
            r"^\b((^\D$)){0,2}?\B$",
            3,
            0,
            Some(2),
            false,
            2,
            true,
            true,
            true,
            true,
            Some(true),
            Some(false),
        ),
        (
            r"^\B(^\p{Join_C})+\B$",
            4,
            1,
            None,
            true,
            1,
            true,
            true,
            true,
            false,
            Some(false),
            Some(false),
        ),
        (
            r"(\p{NChar}$){1,2}\B$",
            5,
            1,
            Some(2),
            true,
            1,
            false,
            true,
            false,
            true,
            None,
            Some(false),
        ),
        (
            r"\B(^😀)+\B",
            6,
            1,
            None,
            true,
            1,
            false,
            false,
            true,
            false,
            Some(false),
            Some(false),
        ),
        (
            r"((\uD800$)){0,1}\b",
            7,
            0,
            Some(1),
            true,
            2,
            false,
            false,
            false,
            true,
            None,
            Some(true),
        ),
        (
            r"\b(^\?)??",
            8,
            0,
            Some(1),
            false,
            1,
            false,
            false,
            true,
            false,
            Some(true),
            None,
        ),
        (
            r"\B(?:(\W$))+\B",
            9,
            1,
            None,
            true,
            1,
            false,
            false,
            false,
            true,
            Some(false),
            Some(false),
        ),
        (
            r"(^\u{61})?\b",
            0,
            0,
            Some(1),
            true,
            1,
            false,
            false,
            true,
            false,
            None,
            Some(true),
        ),
        (
            r"\b((^a)){1,3}\b$",
            0,
            1,
            Some(3),
            true,
            2,
            false,
            true,
            true,
            false,
            Some(true),
            Some(true),
        ),
    ];
    fn plan(source: &str, v: bool, i: bool, m: bool, s: bool) -> RegExpUnicodeRepeatedAtomMatcher {
        RegExpUnicodeRepeatedAtomMatcher::compile_with_scoped_word_boundaries_and_work(
            &JsString::from(source),
            v,
            i,
            m,
            s,
            |_| Ok::<(), ()>(()),
        )
        .unwrap()
        .unwrap()
    }

    #[test]
    fn unicode_repeated_atom_scoped_word_ranges_snapshot() {
        let mut rows = String::new();
        for (source, ..) in CASES {
            let pattern = JsString::from(source);
            for v in [false, true] {
                for i in [false, true] {
                    for m in [false, true] {
                        for s in [false, true] {
                            let matcher = plan(source, v, i, m, s);
                            for input in [
                                JsString::from("Aaaa\nAA"),
                                JsString::from("A\r\n\u{200c}\u{200d}\u{ffff}\u{10ffff}"),
                                JsString::from("😀??aaſK"),
                                JsString::from_code_units(vec![0x61, 0xd800, 0x61, 0xdc00]),
                                JsString::from(""),
                            ] {
                                for start in [0, 1, input.len(), input.len() + 1] {
                                    for sticky in [false, true] {
                                        let found = matcher
                                            .find_with_captures_and_work(
                                                &input,
                                                start,
                                                sticky,
                                                |_| Ok::<(), ()>(()),
                                            )
                                            .unwrap();
                                        writeln!(rows,"{pattern:?} v={v} i={i} m={m} s={s} input={input:?} start={start} sticky={sticky} count={} range={:?} capture={:?}",matcher.capture_count(),found.as_ref().map(|f|f.range.clone()),found.and_then(|f|f.atom_capture)).unwrap();
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        insta::assert_snapshot!(rows);
    }

    #[test]
    fn unicode_repeated_atom_scoped_words_agree_with_independent_counts_and_final_character() {
        let alphabet: [u16; 10] = [
            0x41, 0x61, 10, 13, 0x17f, 0x212a, 0xd83d, 0xde00, 0xd800, 0xdc00,
        ];
        let lt = |c| [10, 13, 0x2028, 0x2029].contains(&c);
        for i in [false, true] {
            for m in [false, true] {
                for s in [false, true] {
                    let plans: Vec<_> = CASES.iter().map(|c| plan(c.0, true, i, m, s)).collect();
                    let fold = |c| {
                        if i {
                            match c {
                                0x41 => 0x61,
                                0x17f => 0x73,
                                0x212a => 0x6b,
                                _ => c,
                            }
                        } else {
                            c
                        }
                    };
                    let member = |kind, c| match kind {
                        0 => fold(c) == 0x61,
                        1 => [0x61, fold(0x17f), fold(0x212a)].contains(&fold(c)),
                        2 => s || !lt(c),
                        3 => !(0x30..=0x39).contains(&c),
                        4 => (0x200c..=0x200d).contains(&c),
                        5 => matches!(c, 0xfdd0..=0xfdef) || (c & 0xffff) >= 0xfffe,
                        6 => c == 0x1f600,
                        7 => c == 0xd800,
                        8 => c == 0x3f,
                        9 => !char::from_u32(fold(c))
                            .is_some_and(|c| c.is_ascii_alphanumeric() || c == '_'),
                        _ => unreachable!(),
                    };
                    for len in 0..=4 {
                        for ordinal in 0..alphabet.len().pow(len) {
                            let units: Vec<_> = (0..len)
                                .map(|j| alphabet[ordinal / alphabet.len().pow(j) % alphabet.len()])
                                .collect();
                            let mut points = Vec::new();
                            let mut bounds = vec![0];
                            let mut j = 0;
                            while j < units.len() {
                                let mut c = u32::from(units[j]);
                                j += 1;
                                if (0xd800..=0xdbff).contains(&c)
                                    && j < units.len()
                                    && (0xdc00..=0xdfff).contains(&units[j])
                                {
                                    c = 65536 + (c - 0xd800) * 1024 + u32::from(units[j]) - 0xdc00;
                                    j += 1;
                                }
                                points.push(c);
                                bounds.push(j);
                            }
                            let input = JsString::from_code_units(units);
                            for start in 0..=input.len() + 1 {
                                for sticky in [false, true] {
                                    for (case, matcher) in CASES.iter().zip(&plans) {
                                        let expected = if start > input.len() {
                                            None
                                        } else {
                                            let first = bounds.partition_point(|&b| b <= start) - 1;
                                            (first..=points.len()).find_map(|at| {
                                                let word=|c:Option<&u32>|c.is_some_and(|&c|matches!(c,0x30..=0x39|0x41..=0x5a|0x5f|0x61..=0x7a)||(i&&matches!(c,0x17f|0x212a)));
                                                let boundary=word(at.checked_sub(1).and_then(|p|points.get(p)))!=word(points.get(at));
                                                if case.10.is_some_and(|positive|boundary!=positive) || (sticky && at != first)
                                                    || (case.6
                                                        && at != 0
                                                        && !(m && lt(points[at - 1])))
                                                {
                                                    return None;
                                                }
                                                let available = (at..points.len())
                                                    .take_while(|&p| {
                                                        member(case.1, points[p])
                                                            && (!case.8
                                                                || p == 0
                                                                || (m && lt(points[p - 1])))
                                                            && (!case.9
                                                                || p + 1 == points.len()
                                                                || (m && lt(points[p + 1])))
                                                    })
                                                    .count();
                                                let upper =
                                                    available.min(case.3.unwrap_or(available));
                                                if upper < case.2 {
                                                    return None;
                                                }
                                                let endpoint = |n: usize| {
                                                    let end = at + n;
                                                    if case.7
                                                        && end != points.len()
                                                        && !(m && lt(points[end]))
                                                    {
                                                        return None;
                                                    }
                                                    let boundary=word(end.checked_sub(1).and_then(|p|points.get(p)))!=word(points.get(end));
                                                    if case.11.is_some_and(|positive|boundary!=positive){return None;}
                                                    Some(RegExpUnicodeRepeatedAtomMatch {
                                                        range: bounds[at]..bounds[end],
                                                        atom_capture: (case.5 != 0 && n != 0)
                                                            .then(|| bounds[end - 1]..bounds[end]),
                                                    })
                                                };
                                                let mut counts = case.2..=upper;
                                                if case.4 {
                                                    counts.rev().find_map(endpoint)
                                                } else {
                                                    counts.find_map(endpoint)
                                                }
                                            })
                                        };
                                        assert_eq!(matcher.capture_count(), case.5, "{}", case.0);
                                        assert_eq!(
                                            matcher
                                                .find_with_captures_and_work(
                                                    &input,
                                                    start,
                                                    sticky,
                                                    |_| Ok::<(), ()>(())
                                                )
                                                .unwrap(),
                                            expected,
                                            "{} i={i} m={m} s={s} {input:?} start={start} sticky={sticky}",
                                            case.0
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

    #[test]
    fn unicode_repeated_atom_scoped_word_work_clones_and_scope_contracts() {
        let source = format!(r"\B{}^.${}+\B", "(".repeat(3000), ")".repeat(3000));
        let matcher = plan(&source, true, false, true, true).clone();
        let input = JsString::from("\n".repeat(200000).as_str());
        let mut work = 0;
        assert_eq!(matcher.capture_count(), 3000);
        assert_eq!(
            matcher
                .find_with_captures_and_work(&input, 0, false, |n| {
                    work += n;
                    Ok::<(), ()>(())
                })
                .unwrap(),
            Some(RegExpUnicodeRepeatedAtomMatch {
                range: 0..200000,
                atom_capture: Some(199999..200000)
            })
        );
        assert!(work < 40 * input.len(), "{work}");
        let matcher = plan(r"\B(a$){2}\b", false, false, false, false);
        let input = JsString::from("a".repeat(200000).as_str());
        work = 0;
        assert_eq!(
            matcher
                .find_with_work(&input, 0, false, |n| {
                    work += n;
                    Ok::<(), ()>(())
                })
                .unwrap(),
            None
        );
        assert!(work < 40 * input.len(), "{work}");
        let matcher = plan(r"\B((^.$)){2,3}\B$", true, false, true, true);
        let input = JsString::from("\r\n");
        let mut calls = 0;
        matcher
            .find_with_captures_and_work(&input, 0, false, |_| {
                calls += 1;
                Ok::<(), ()>(())
            })
            .unwrap();
        for fail in 1..=calls {
            let mut n = 0;
            assert!(
                matcher
                    .find_with_captures_and_work(&input, 0, false, |_| {
                        n += 1;
                        if n == fail { Err(()) } else { Ok(()) }
                    })
                    .is_err()
            );
        }
        for source in [
            r"(^a)+",
            r"\b(^$)+",
            r"\b(a?)+",
            r"\b(a|b)+",
            r"\b()+",
            r"\b(ab)+",
            r"\b(^a)+b",
            r"\b(?i:a)+",
            r"\b(\b.)+",
            r"\b(\B)+",
            r"\b(^a(b))+",
            r"\b(a^)+",
        ] {
            assert!(
                RegExpUnicodeRepeatedAtomMatcher::compile_with_scoped_word_boundaries_and_work(
                    &JsString::from(source),
                    true,
                    false,
                    true,
                    true,
                    |_| Ok::<(), ()>(())
                )
                .unwrap()
                .is_none(),
                "{source}"
            );
        }
        for source in [r"\b(^a)+", r"(a$)+\b", r"\B((^.$))+\B"] {
            assert!(
                RegExpUnicodeRepeatedAtomMatcher::compile_with_word_boundaries_and_work(
                    &JsString::from(source),
                    true,
                    false,
                    true,
                    true,
                    |_| Ok::<(), ()>(())
                )
                .unwrap()
                .is_none()
            );
        }
    }
}

#[cfg(test)]
mod inner_word_tests {
    use super::*;
    use std::fmt::Write;

    type Case = (
        &'static str,
        u8,
        usize,
        Option<usize>,
        bool,
        usize,
        bool,
        bool,
        bool,
        bool,
        Option<bool>,
        Option<bool>,
    );
    const CASES: [Case; 16] = [
        (
            r"(\ba)+",
            0,
            1,
            None,
            true,
            1,
            false,
            false,
            false,
            false,
            Some(true),
            None,
        ),
        (
            r"((a\b)){0,3}?",
            0,
            0,
            Some(3),
            false,
            2,
            false,
            false,
            false,
            false,
            None,
            Some(true),
        ),
        (
            r"(?:(\ba\b)){2,4}",
            0,
            2,
            Some(4),
            true,
            1,
            false,
            false,
            false,
            false,
            Some(true),
            Some(true),
        ),
        (
            r"(?:\Ba)+",
            0,
            1,
            None,
            true,
            0,
            false,
            false,
            false,
            false,
            Some(false),
            None,
        ),
        (
            r"((?:a\B)){0}",
            0,
            0,
            Some(0),
            true,
            1,
            false,
            false,
            false,
            false,
            None,
            Some(false),
        ),
        (
            r"(^\b[aſK])+",
            1,
            1,
            None,
            true,
            1,
            false,
            false,
            true,
            false,
            Some(true),
            None,
        ),
        (
            r"((.\B$)){1,3}$",
            2,
            1,
            Some(3),
            true,
            2,
            false,
            true,
            false,
            true,
            None,
            Some(false),
        ),
        (
            r"^((^\b\D\B$)){0,2}?$",
            3,
            0,
            Some(2),
            false,
            2,
            true,
            true,
            true,
            true,
            Some(true),
            Some(false),
        ),
        (
            r"^(\B\p{Join_C}\B)+$",
            4,
            1,
            None,
            true,
            1,
            true,
            true,
            false,
            false,
            Some(false),
            Some(false),
        ),
        (
            r"(\p{NChar}\B$){1,2}$",
            5,
            1,
            Some(2),
            true,
            1,
            false,
            true,
            false,
            true,
            None,
            Some(false),
        ),
        (
            r"(\B😀\B)+",
            6,
            1,
            None,
            true,
            1,
            false,
            false,
            false,
            false,
            Some(false),
            Some(false),
        ),
        (
            r"((\uD800\b)){0,1}",
            7,
            0,
            Some(1),
            true,
            2,
            false,
            false,
            false,
            false,
            None,
            Some(true),
        ),
        (
            r"(\b\?)??",
            8,
            0,
            Some(1),
            false,
            1,
            false,
            false,
            false,
            false,
            Some(true),
            None,
        ),
        (
            r"(?:(\B\W\B$))+",
            9,
            1,
            None,
            true,
            1,
            false,
            false,
            false,
            true,
            Some(false),
            Some(false),
        ),
        (
            r"(\u{61}\b)?",
            0,
            0,
            Some(1),
            true,
            1,
            false,
            false,
            false,
            false,
            None,
            Some(true),
        ),
        (
            r"((^\ba\b)){1,3}$",
            0,
            1,
            Some(3),
            true,
            2,
            false,
            true,
            true,
            false,
            Some(true),
            Some(true),
        ),
    ];
    fn plan(source: &str, v: bool, i: bool, m: bool, s: bool) -> RegExpUnicodeRepeatedAtomMatcher {
        RegExpUnicodeRepeatedAtomMatcher::compile_capturing_atom_with_assertions_and_work(
            &JsString::from(source),
            v,
            i,
            m,
            s,
            |_| Ok::<(), ()>(()),
        )
        .unwrap()
        .unwrap()
    }

    #[test]
    fn unicode_repeated_atom_inner_word_ranges_snapshot() {
        let mut rows = String::new();
        for (source, ..) in CASES {
            let pattern = JsString::from(source);
            for v in [false, true] {
                for i in [false, true] {
                    for m in [false, true] {
                        for s in [false, true] {
                            let matcher = plan(source, v, i, m, s);
                            for input in [
                                JsString::from("Aaaa\nAA"),
                                JsString::from("A\r\n\u{200c}\u{200d}\u{ffff}\u{10ffff}"),
                                JsString::from("😀??aaſK"),
                                JsString::from_code_units(vec![0x61, 0xd800, 0x61, 0xdc00]),
                                JsString::from(""),
                            ] {
                                for start in [0, 1, input.len(), input.len() + 1] {
                                    for sticky in [false, true] {
                                        let found = matcher
                                            .find_with_captures_and_work(
                                                &input,
                                                start,
                                                sticky,
                                                |_| Ok::<(), ()>(()),
                                            )
                                            .unwrap();
                                        writeln!(rows,"{pattern:?} v={v} i={i} m={m} s={s} input={input:?} start={start} sticky={sticky} count={} range={:?} capture={:?}",matcher.capture_count(),found.as_ref().map(|f|f.range.clone()),found.and_then(|f|f.atom_capture)).unwrap();
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        insta::assert_snapshot!(rows);
    }

    #[test]
    fn unicode_repeated_atom_inner_words_agree_with_independent_counts_and_final_character() {
        let alphabet: [u16; 10] = [
            0x41, 0x61, 10, 13, 0x17f, 0x212a, 0xd83d, 0xde00, 0xd800, 0xdc00,
        ];
        let lt = |c| [10, 13, 0x2028, 0x2029].contains(&c);
        for i in [false, true] {
            for m in [false, true] {
                for s in [false, true] {
                    let plans: Vec<_> = CASES.iter().map(|c| plan(c.0, true, i, m, s)).collect();
                    let fold = |c| {
                        if i {
                            match c {
                                0x41 => 0x61,
                                0x17f => 0x73,
                                0x212a => 0x6b,
                                _ => c,
                            }
                        } else {
                            c
                        }
                    };
                    let member = |kind, c| match kind {
                        0 => fold(c) == 0x61,
                        1 => [0x61, fold(0x17f), fold(0x212a)].contains(&fold(c)),
                        2 => s || !lt(c),
                        3 => !(0x30..=0x39).contains(&c),
                        4 => (0x200c..=0x200d).contains(&c),
                        5 => matches!(c, 0xfdd0..=0xfdef) || (c & 0xffff) >= 0xfffe,
                        6 => c == 0x1f600,
                        7 => c == 0xd800,
                        8 => c == 0x3f,
                        9 => !char::from_u32(fold(c))
                            .is_some_and(|c| c.is_ascii_alphanumeric() || c == '_'),
                        _ => unreachable!(),
                    };
                    for len in 0..=4 {
                        for ordinal in 0..alphabet.len().pow(len) {
                            let units: Vec<_> = (0..len)
                                .map(|j| alphabet[ordinal / alphabet.len().pow(j) % alphabet.len()])
                                .collect();
                            let mut points = Vec::new();
                            let mut bounds = vec![0];
                            let mut j = 0;
                            while j < units.len() {
                                let mut c = u32::from(units[j]);
                                j += 1;
                                if (0xd800..=0xdbff).contains(&c)
                                    && j < units.len()
                                    && (0xdc00..=0xdfff).contains(&units[j])
                                {
                                    c = 65536 + (c - 0xd800) * 1024 + u32::from(units[j]) - 0xdc00;
                                    j += 1;
                                }
                                points.push(c);
                                bounds.push(j);
                            }
                            let input = JsString::from_code_units(units);
                            for start in 0..=input.len() + 1 {
                                for sticky in [false, true] {
                                    for (case, matcher) in CASES.iter().zip(&plans) {
                                        let expected = if start > input.len() {
                                            None
                                        } else {
                                            let first = bounds.partition_point(|&b| b <= start) - 1;
                                            (first..=points.len()).find_map(|at| {
                                                let word=|c:Option<&u32>|c.is_some_and(|&c|matches!(c,0x30..=0x39|0x41..=0x5a|0x5f|0x61..=0x7a)||(i&&matches!(c,0x17f|0x212a)));
                                                if (sticky && at != first)
                                                    || (case.6
                                                        && at != 0
                                                        && !(m && lt(points[at - 1])))
                                                {
                                                    return None;
                                                }
                                                let available = (at..points.len())
                                                    .take_while(|&p| {
                                                        member(case.1, points[p])
                                                            && !case.10.is_some_and(|positive| (word(p.checked_sub(1).and_then(|n|points.get(n))) != word(points.get(p))) != positive)
                                                            && !case.11.is_some_and(|positive| (word(points.get(p)) != word(points.get(p+1))) != positive)
                                                            && (!case.8
                                                                || p == 0
                                                                || (m && lt(points[p - 1])))
                                                            && (!case.9
                                                                || p + 1 == points.len()
                                                                || (m && lt(points[p + 1])))
                                                    })
                                                    .count();
                                                let upper =
                                                    available.min(case.3.unwrap_or(available));
                                                if upper < case.2 {
                                                    return None;
                                                }
                                                let endpoint = |n: usize| {
                                                    let end = at + n;
                                                    if case.7
                                                        && end != points.len()
                                                        && !(m && lt(points[end]))
                                                    {
                                                        return None;
                                                    }
                                                    Some(RegExpUnicodeRepeatedAtomMatch {
                                                        range: bounds[at]..bounds[end],
                                                        atom_capture: (case.5 != 0 && n != 0)
                                                            .then(|| bounds[end - 1]..bounds[end]),
                                                    })
                                                };
                                                let mut counts = case.2..=upper;
                                                if case.4 {
                                                    counts.rev().find_map(endpoint)
                                                } else {
                                                    counts.find_map(endpoint)
                                                }
                                            })
                                        };
                                        assert_eq!(matcher.capture_count(), case.5, "{}", case.0);
                                        assert_eq!(
                                            matcher
                                                .find_with_captures_and_work(
                                                    &input,
                                                    start,
                                                    sticky,
                                                    |_| Ok::<(), ()>(())
                                                )
                                                .unwrap(),
                                            expected,
                                            "{} i={i} m={m} s={s} {input:?} start={start} sticky={sticky}",
                                            case.0
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

    #[test]
    fn unicode_repeated_atom_inner_word_work_clones_and_scope_contracts() {
        let source = format!(r"{}^\B.\B${}+", "(".repeat(3000), ")".repeat(3000));
        let matcher = plan(&source, true, false, true, true).clone();
        let input = JsString::from("\n".repeat(200000).as_str());
        let mut work = 0;
        assert_eq!(matcher.capture_count(), 3000);
        assert_eq!(
            matcher
                .find_with_captures_and_work(&input, 0, false, |n| {
                    work += n;
                    Ok::<(), ()>(())
                })
                .unwrap(),
            Some(RegExpUnicodeRepeatedAtomMatch {
                range: 0..200000,
                atom_capture: Some(199999..200000)
            })
        );
        assert!(work < 50 * input.len(), "{work}");
        let matcher = plan(r"(\ba){2}", false, false, false, false);
        let input = JsString::from("a".repeat(200000).as_str());
        work = 0;
        assert_eq!(
            matcher
                .find_with_work(&input, 0, false, |n| {
                    work += n;
                    Ok::<(), ()>(())
                })
                .unwrap(),
            None
        );
        assert!(work < 50 * input.len(), "{work}");
        let matcher = plan(r"(^\B.\B$){2,3}$", true, false, true, true);
        let input = JsString::from("\r\n");
        let mut calls = 0;
        matcher
            .find_with_captures_and_work(&input, 0, false, |_| {
                calls += 1;
                Ok::<(), ()>(())
            })
            .unwrap();
        for fail in 1..=calls {
            let mut n = 0;
            assert!(
                matcher
                    .find_with_captures_and_work(&input, 0, false, |_| {
                        n += 1;
                        if n == fail { Err(()) } else { Ok(()) }
                    })
                    .is_err()
            );
        }
        for source in [
            r"(\b)+",
            r"(^\B$)+",
            r"(\ba?)+",
            r"(\ba|b)+",
            r"(\bab)+",
            r"(\ba)+b",
            r"\b(\ba)+",
            r"(\ba)+\b",
            r"(a^\b)+",
            r"(\ba(b))+",
            r"(?i:\ba)+",
        ] {
            assert!(
                RegExpUnicodeRepeatedAtomMatcher::compile_capturing_atom_with_assertions_and_work(
                    &JsString::from(source),
                    true,
                    false,
                    true,
                    true,
                    |_| Ok::<(), ()>(())
                )
                .unwrap()
                .is_none(),
                "{source}"
            );
        }
        for source in [r"(\ba)+", r"(a\b)+", r"(^\B.\B$)+"] {
            assert!(RegExpUnicodeRepeatedAtomMatcher::compile_capturing_atom_with_line_assertions_and_work(&JsString::from(source),true,false,true,true,|_|Ok::<(),()>(())).unwrap().is_none(),"{source}");
            assert!(
                RegExpUnicodeRepeatedAtomMatcher::compile_capturing_atom_with_work(
                    &JsString::from(source),
                    true,
                    false,
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
}

#[cfg(test)]
mod composed_word_tests {
    use super::*;
    use std::fmt::Write;

    type Case = (
        &'static str,
        u8,
        usize,
        Option<usize>,
        bool,
        usize,
        bool,
        bool,
        bool,
        bool,
        Option<bool>,
        Option<bool>,
        Option<bool>,
        Option<bool>,
    );
    const CASES: [Case; 16] = [
        (
            r"\b(\ba)+\B",
            0,
            1,
            None,
            true,
            1,
            false,
            false,
            false,
            false,
            Some(true),
            None,
            Some(true),
            Some(false),
        ),
        (
            r"\B((a\b)){0,3}?\b",
            0,
            0,
            Some(3),
            false,
            2,
            false,
            false,
            false,
            false,
            None,
            Some(true),
            Some(false),
            Some(true),
        ),
        (
            r"\b(?:(\ba\b)){2,4}\b",
            0,
            2,
            Some(4),
            true,
            1,
            false,
            false,
            false,
            false,
            Some(true),
            Some(true),
            Some(true),
            Some(true),
        ),
        (
            r"\B(?:\Ba)+",
            0,
            1,
            None,
            true,
            0,
            false,
            false,
            false,
            false,
            Some(false),
            None,
            Some(false),
            None,
        ),
        (
            r"((?:a\B)){0}\B",
            0,
            0,
            Some(0),
            true,
            1,
            false,
            false,
            false,
            false,
            None,
            Some(false),
            None,
            Some(false),
        ),
        (
            r"\b(^\b[aſK])+\b",
            1,
            1,
            None,
            true,
            1,
            false,
            false,
            true,
            false,
            Some(true),
            None,
            Some(true),
            Some(true),
        ),
        (
            r"((.\B$)){1,3}\B$",
            2,
            1,
            Some(3),
            true,
            2,
            false,
            true,
            false,
            true,
            None,
            Some(false),
            None,
            Some(false),
        ),
        (
            r"^\b((^\b\D\B$)){0,2}?\B$",
            3,
            0,
            Some(2),
            false,
            2,
            true,
            true,
            true,
            true,
            Some(true),
            Some(false),
            Some(true),
            Some(false),
        ),
        (
            r"^\B(\B\p{Join_C}\B)+\B$",
            4,
            1,
            None,
            true,
            1,
            true,
            true,
            false,
            false,
            Some(false),
            Some(false),
            Some(false),
            Some(false),
        ),
        (
            r"(\p{NChar}\B$){1,2}\B$",
            5,
            1,
            Some(2),
            true,
            1,
            false,
            true,
            false,
            true,
            None,
            Some(false),
            None,
            Some(false),
        ),
        (
            r"\B(\B😀\B)+\B",
            6,
            1,
            None,
            true,
            1,
            false,
            false,
            false,
            false,
            Some(false),
            Some(false),
            Some(false),
            Some(false),
        ),
        (
            r"((\uD800\b)){0,1}\b",
            7,
            0,
            Some(1),
            true,
            2,
            false,
            false,
            false,
            false,
            None,
            Some(true),
            None,
            Some(true),
        ),
        (
            r"\b(\b\?)??",
            8,
            0,
            Some(1),
            false,
            1,
            false,
            false,
            false,
            false,
            Some(true),
            None,
            Some(true),
            None,
        ),
        (
            r"\B(?:(\B\W\B$))+\B",
            9,
            1,
            None,
            true,
            1,
            false,
            false,
            false,
            true,
            Some(false),
            Some(false),
            Some(false),
            Some(false),
        ),
        (
            r"(\u{61}\b)?\b",
            0,
            0,
            Some(1),
            true,
            1,
            false,
            false,
            false,
            false,
            None,
            Some(true),
            None,
            Some(true),
        ),
        (
            r"\b((^\ba\b)){1,3}\b$",
            0,
            1,
            Some(3),
            true,
            2,
            false,
            true,
            true,
            false,
            Some(true),
            Some(true),
            Some(true),
            Some(true),
        ),
    ];
    fn plan(source: &str, v: bool, i: bool, m: bool, s: bool) -> RegExpUnicodeRepeatedAtomMatcher {
        RegExpUnicodeRepeatedAtomMatcher::compile_with_composed_word_boundaries_and_work(
            &JsString::from(source),
            v,
            i,
            m,
            s,
            |_| Ok::<(), ()>(()),
        )
        .unwrap()
        .unwrap()
    }

    #[test]
    fn unicode_repeated_atom_composed_word_ranges_snapshot() {
        let mut rows = String::new();
        for (source, ..) in CASES {
            let pattern = JsString::from(source);
            for v in [false, true] {
                for i in [false, true] {
                    for m in [false, true] {
                        for s in [false, true] {
                            let matcher = plan(source, v, i, m, s);
                            for input in [
                                JsString::from("Aaaa\nAA"),
                                JsString::from("A\r\n\u{200c}\u{200d}\u{ffff}\u{10ffff}"),
                                JsString::from("😀??aaſK"),
                                JsString::from_code_units(vec![0x61, 0xd800, 0x61, 0xdc00]),
                                JsString::from(""),
                            ] {
                                for start in [0, 1, input.len(), input.len() + 1] {
                                    for sticky in [false, true] {
                                        let found = matcher
                                            .find_with_captures_and_work(
                                                &input,
                                                start,
                                                sticky,
                                                |_| Ok::<(), ()>(()),
                                            )
                                            .unwrap();
                                        writeln!(rows,"{pattern:?} v={v} i={i} m={m} s={s} input={input:?} start={start} sticky={sticky} count={} range={:?} capture={:?}",matcher.capture_count(),found.as_ref().map(|f|f.range.clone()),found.and_then(|f|f.atom_capture)).unwrap();
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        insta::assert_snapshot!(rows);
    }

    #[test]
    fn unicode_repeated_atom_composed_words_agree_with_independent_counts_and_final_character() {
        let alphabet: [u16; 10] = [
            0x41, 0x61, 10, 13, 0x17f, 0x212a, 0xd83d, 0xde00, 0xd800, 0xdc00,
        ];
        let lt = |c| [10, 13, 0x2028, 0x2029].contains(&c);
        for i in [false, true] {
            for m in [false, true] {
                for s in [false, true] {
                    let plans: Vec<_> = CASES.iter().map(|c| plan(c.0, true, i, m, s)).collect();
                    let fold = |c| {
                        if i {
                            match c {
                                0x41 => 0x61,
                                0x17f => 0x73,
                                0x212a => 0x6b,
                                _ => c,
                            }
                        } else {
                            c
                        }
                    };
                    let member = |kind, c| match kind {
                        0 => fold(c) == 0x61,
                        1 => [0x61, fold(0x17f), fold(0x212a)].contains(&fold(c)),
                        2 => s || !lt(c),
                        3 => !(0x30..=0x39).contains(&c),
                        4 => (0x200c..=0x200d).contains(&c),
                        5 => matches!(c, 0xfdd0..=0xfdef) || (c & 0xffff) >= 0xfffe,
                        6 => c == 0x1f600,
                        7 => c == 0xd800,
                        8 => c == 0x3f,
                        9 => !char::from_u32(fold(c))
                            .is_some_and(|c| c.is_ascii_alphanumeric() || c == '_'),
                        _ => unreachable!(),
                    };
                    for len in 0..=4 {
                        for ordinal in 0..alphabet.len().pow(len) {
                            let units: Vec<_> = (0..len)
                                .map(|j| alphabet[ordinal / alphabet.len().pow(j) % alphabet.len()])
                                .collect();
                            let mut points = Vec::new();
                            let mut bounds = vec![0];
                            let mut j = 0;
                            while j < units.len() {
                                let mut c = u32::from(units[j]);
                                j += 1;
                                if (0xd800..=0xdbff).contains(&c)
                                    && j < units.len()
                                    && (0xdc00..=0xdfff).contains(&units[j])
                                {
                                    c = 65536 + (c - 0xd800) * 1024 + u32::from(units[j]) - 0xdc00;
                                    j += 1;
                                }
                                points.push(c);
                                bounds.push(j);
                            }
                            let input = JsString::from_code_units(units);
                            for start in 0..=input.len() + 1 {
                                for sticky in [false, true] {
                                    for (case, matcher) in CASES.iter().zip(&plans) {
                                        let expected = if start > input.len() {
                                            None
                                        } else {
                                            let first = bounds.partition_point(|&b| b <= start) - 1;
                                            (first..=points.len()).find_map(|at| {
                                                let word=|c:Option<&u32>|c.is_some_and(|&c|matches!(c,0x30..=0x39|0x41..=0x5a|0x5f|0x61..=0x7a)||(i&&matches!(c,0x17f|0x212a)));
                                                let start_boundary=word(at.checked_sub(1).and_then(|n|points.get(n)))!=word(points.get(at));
                                                if case.12.is_some_and(|expected|start_boundary!=expected) || (sticky && at != first)
                                                    || (case.6
                                                        && at != 0
                                                        && !(m && lt(points[at - 1])))
                                                {
                                                    return None;
                                                }
                                                let available = (at..points.len())
                                                    .take_while(|&p| {
                                                        member(case.1, points[p])
                                                            && !case.10.is_some_and(|positive| (word(p.checked_sub(1).and_then(|n|points.get(n))) != word(points.get(p))) != positive)
                                                            && !case.11.is_some_and(|positive| (word(points.get(p)) != word(points.get(p+1))) != positive)
                                                            && (!case.8
                                                                || p == 0
                                                                || (m && lt(points[p - 1])))
                                                            && (!case.9
                                                                || p + 1 == points.len()
                                                                || (m && lt(points[p + 1])))
                                                    })
                                                    .count();
                                                let upper =
                                                    available.min(case.3.unwrap_or(available));
                                                if upper < case.2 {
                                                    return None;
                                                }
                                                let endpoint = |n: usize| {
                                                    let end = at + n;
                                                    if case.7
                                                        && end != points.len()
                                                        && !(m && lt(points[end]))
                                                    {
                                                        return None;
                                                    }
                                                    let end_boundary=word(end.checked_sub(1).and_then(|n|points.get(n)))!=word(points.get(end));
                                                    if case.13.is_some_and(|expected|end_boundary!=expected){return None;}
                                                    Some(RegExpUnicodeRepeatedAtomMatch {
                                                        range: bounds[at]..bounds[end],
                                                        atom_capture: (case.5 != 0 && n != 0)
                                                            .then(|| bounds[end - 1]..bounds[end]),
                                                    })
                                                };
                                                let mut counts = case.2..=upper;
                                                if case.4 {
                                                    counts.rev().find_map(endpoint)
                                                } else {
                                                    counts.find_map(endpoint)
                                                }
                                            })
                                        };
                                        assert_eq!(matcher.capture_count(), case.5, "{}", case.0);
                                        assert_eq!(
                                            matcher
                                                .find_with_captures_and_work(
                                                    &input,
                                                    start,
                                                    sticky,
                                                    |_| Ok::<(), ()>(())
                                                )
                                                .unwrap(),
                                            expected,
                                            "{} i={i} m={m} s={s} {input:?} start={start} sticky={sticky}",
                                            case.0
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

    #[test]
    fn unicode_repeated_atom_composed_word_work_clones_and_scope_contracts() {
        let source = format!(r"\B{}^\B.\B${}+\B", "(".repeat(3000), ")".repeat(3000));
        let matcher = plan(&source, true, false, true, true).clone();
        let input = JsString::from("\n".repeat(200000).as_str());
        let mut work = 0;
        assert_eq!(matcher.capture_count(), 3000);
        assert_eq!(
            matcher
                .find_with_captures_and_work(&input, 0, false, |n| {
                    work += n;
                    Ok::<(), ()>(())
                })
                .unwrap(),
            Some(RegExpUnicodeRepeatedAtomMatch {
                range: 0..200000,
                atom_capture: Some(199999..200000)
            })
        );
        assert!(work < 50 * input.len(), "{work}");
        let matcher = plan(r"\b(\ba){2}\b", false, false, false, false);
        let input = JsString::from("a".repeat(200000).as_str());
        work = 0;
        assert_eq!(
            matcher
                .find_with_work(&input, 0, false, |n| {
                    work += n;
                    Ok::<(), ()>(())
                })
                .unwrap(),
            None
        );
        assert!(work < 50 * input.len(), "{work}");
        let matcher = plan(r"\B(^\B.\B$){2,3}\B$", true, false, true, true);
        let input = JsString::from("\r\n");
        let mut calls = 0;
        matcher
            .find_with_captures_and_work(&input, 0, false, |_| {
                calls += 1;
                Ok::<(), ()>(())
            })
            .unwrap();
        for fail in 1..=calls {
            let mut n = 0;
            assert!(
                matcher
                    .find_with_captures_and_work(&input, 0, false, |_| {
                        n += 1;
                        if n == fail { Err(()) } else { Ok(()) }
                    })
                    .is_err()
            );
        }
        for source in [
            r"(\ba)+",
            r"\b(\b)+",
            r"\B(^\B$)+",
            r"\b(\ba?)+",
            r"\b(\ba|b)+",
            r"\b(\bab)+",
            r"\b(\ba)+b",
            r"\b(a^\b)+",
            r"\b(\ba(b))+",
            r"\b(?i:\ba)+",
        ] {
            assert!(
                RegExpUnicodeRepeatedAtomMatcher::compile_with_composed_word_boundaries_and_work(
                    &JsString::from(source),
                    true,
                    false,
                    true,
                    true,
                    |_| Ok::<(), ()>(())
                )
                .unwrap()
                .is_none(),
                "{source}"
            );
        }
        for source in [r"\b(\ba)+", r"(a\b)+\b", r"\B(^\B.\B$)+\B"] {
            assert!(
                RegExpUnicodeRepeatedAtomMatcher::compile_with_scoped_word_boundaries_and_work(
                    &JsString::from(source),
                    true,
                    false,
                    true,
                    true,
                    |_| Ok::<(), ()>(())
                )
                .unwrap()
                .is_none(),
                "{source}"
            );
            assert!(
                RegExpUnicodeRepeatedAtomMatcher::compile_with_word_boundaries_and_work(
                    &JsString::from(source),
                    true,
                    false,
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
}

#[cfg(test)]
mod nested_assertion_group_tests {
    use super::*;
    use std::fmt::Write;

    type Case = (
        &'static str,
        u8,
        usize,
        Option<usize>,
        bool,
        usize,
        bool,
        bool,
        bool,
        bool,
        Option<bool>,
        Option<bool>,
    );
    const CASES: [Case; 16] = [
        (
            r"(\b(a))+",
            0,
            1,
            None,
            true,
            2,
            false,
            false,
            false,
            false,
            Some(true),
            None,
        ),
        (
            r"((a)\b){0,3}?",
            0,
            0,
            Some(3),
            false,
            2,
            false,
            false,
            false,
            false,
            None,
            Some(true),
        ),
        (
            r"(?:\b(a)\b){2,4}",
            0,
            2,
            Some(4),
            true,
            1,
            false,
            false,
            false,
            false,
            Some(true),
            Some(true),
        ),
        (
            r"(?:\B(?:a))+",
            0,
            1,
            None,
            true,
            0,
            false,
            false,
            false,
            false,
            Some(false),
            None,
        ),
        (
            r"((?:a)\B){0}",
            0,
            0,
            Some(0),
            true,
            1,
            false,
            false,
            false,
            false,
            None,
            Some(false),
        ),
        (
            r"(\b^([aſK]))+",
            1,
            1,
            None,
            true,
            2,
            false,
            false,
            true,
            false,
            Some(true),
            None,
        ),
        (
            r"((.)$\B){1,3}$",
            2,
            1,
            Some(3),
            true,
            2,
            false,
            true,
            false,
            true,
            None,
            Some(false),
        ),
        (
            r"^((^\b(\D)\B$)){0,2}?$",
            3,
            0,
            Some(2),
            false,
            3,
            true,
            true,
            true,
            true,
            Some(true),
            Some(false),
        ),
        (
            r"^(\B(\p{Join_C})\B)+$",
            4,
            1,
            None,
            true,
            2,
            true,
            true,
            false,
            false,
            Some(false),
            Some(false),
        ),
        (
            r"((\p{NChar})\B$){1,2}$",
            5,
            1,
            Some(2),
            true,
            2,
            false,
            true,
            false,
            true,
            None,
            Some(false),
        ),
        (
            r"(\B(😀)\B)+",
            6,
            1,
            None,
            true,
            2,
            false,
            false,
            false,
            false,
            Some(false),
            Some(false),
        ),
        (
            r"((\uD800)\b){0,1}",
            7,
            0,
            Some(1),
            true,
            2,
            false,
            false,
            false,
            false,
            None,
            Some(true),
        ),
        (
            r"(\b(\?))??",
            8,
            0,
            Some(1),
            false,
            2,
            false,
            false,
            false,
            false,
            Some(true),
            None,
        ),
        (
            r"(?:(\B(\W)\B$))+",
            9,
            1,
            None,
            true,
            2,
            false,
            false,
            false,
            true,
            Some(false),
            Some(false),
        ),
        (
            r"((\u{61})\b)?",
            0,
            0,
            Some(1),
            true,
            2,
            false,
            false,
            false,
            false,
            None,
            Some(true),
        ),
        (
            r"((^\b(a)\b)){1,3}$",
            0,
            1,
            Some(3),
            true,
            3,
            false,
            true,
            true,
            false,
            Some(true),
            Some(true),
        ),
    ];
    fn plan(source: &str, v: bool, i: bool, m: bool, s: bool) -> RegExpUnicodeRepeatedAtomMatcher {
        RegExpUnicodeRepeatedAtomMatcher::compile_nested_assertion_groups_with_work(
            &JsString::from(source),
            v,
            i,
            m,
            s,
            |_| Ok::<(), ()>(()),
        )
        .unwrap()
        .unwrap()
    }

    #[test]
    fn unicode_repeated_atom_nested_assertion_group_ranges_snapshot() {
        let mut rows = String::new();
        for (source, ..) in CASES {
            let pattern = JsString::from(source);
            for v in [false, true] {
                for i in [false, true] {
                    for m in [false, true] {
                        for s in [false, true] {
                            let matcher = plan(source, v, i, m, s);
                            for input in [
                                JsString::from("Aaaa\nAA"),
                                JsString::from("A\r\n\u{200c}\u{200d}\u{ffff}\u{10ffff}"),
                                JsString::from("😀??aaſK"),
                                JsString::from_code_units(vec![0x61, 0xd800, 0x61, 0xdc00]),
                                JsString::from(""),
                            ] {
                                for start in [0, 1, input.len(), input.len() + 1] {
                                    for sticky in [false, true] {
                                        let found = matcher
                                            .find_with_captures_and_work(
                                                &input,
                                                start,
                                                sticky,
                                                |_| Ok::<(), ()>(()),
                                            )
                                            .unwrap();
                                        writeln!(rows,"{pattern:?} v={v} i={i} m={m} s={s} input={input:?} start={start} sticky={sticky} count={} range={:?} capture={:?}",matcher.capture_count(),found.as_ref().map(|f|f.range.clone()),found.and_then(|f|f.atom_capture)).unwrap();
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        insta::assert_snapshot!(rows);
    }

    #[test]
    fn unicode_repeated_atom_nested_assertion_groups_agree_with_independent_counts_and_final_character()
     {
        let alphabet: [u16; 10] = [
            0x41, 0x61, 10, 13, 0x17f, 0x212a, 0xd83d, 0xde00, 0xd800, 0xdc00,
        ];
        let lt = |c| [10, 13, 0x2028, 0x2029].contains(&c);
        for i in [false, true] {
            for m in [false, true] {
                for s in [false, true] {
                    let plans: Vec<_> = CASES.iter().map(|c| plan(c.0, true, i, m, s)).collect();
                    let fold = |c| {
                        if i {
                            match c {
                                0x41 => 0x61,
                                0x17f => 0x73,
                                0x212a => 0x6b,
                                _ => c,
                            }
                        } else {
                            c
                        }
                    };
                    let member = |kind, c| match kind {
                        0 => fold(c) == 0x61,
                        1 => [0x61, fold(0x17f), fold(0x212a)].contains(&fold(c)),
                        2 => s || !lt(c),
                        3 => !(0x30..=0x39).contains(&c),
                        4 => (0x200c..=0x200d).contains(&c),
                        5 => matches!(c, 0xfdd0..=0xfdef) || (c & 0xffff) >= 0xfffe,
                        6 => c == 0x1f600,
                        7 => c == 0xd800,
                        8 => c == 0x3f,
                        9 => !char::from_u32(fold(c))
                            .is_some_and(|c| c.is_ascii_alphanumeric() || c == '_'),
                        _ => unreachable!(),
                    };
                    for len in 0..=4 {
                        for ordinal in 0..alphabet.len().pow(len) {
                            let units: Vec<_> = (0..len)
                                .map(|j| alphabet[ordinal / alphabet.len().pow(j) % alphabet.len()])
                                .collect();
                            let mut points = Vec::new();
                            let mut bounds = vec![0];
                            let mut j = 0;
                            while j < units.len() {
                                let mut c = u32::from(units[j]);
                                j += 1;
                                if (0xd800..=0xdbff).contains(&c)
                                    && j < units.len()
                                    && (0xdc00..=0xdfff).contains(&units[j])
                                {
                                    c = 65536 + (c - 0xd800) * 1024 + u32::from(units[j]) - 0xdc00;
                                    j += 1;
                                }
                                points.push(c);
                                bounds.push(j);
                            }
                            let input = JsString::from_code_units(units);
                            for start in 0..=input.len() + 1 {
                                for sticky in [false, true] {
                                    for (case, matcher) in CASES.iter().zip(&plans) {
                                        let expected = if start > input.len() {
                                            None
                                        } else {
                                            let first = bounds.partition_point(|&b| b <= start) - 1;
                                            (first..=points.len()).find_map(|at| {
                                                let word=|c:Option<&u32>|c.is_some_and(|&c|matches!(c,0x30..=0x39|0x41..=0x5a|0x5f|0x61..=0x7a)||(i&&matches!(c,0x17f|0x212a)));
                                                if (sticky && at != first)
                                                    || (case.6
                                                        && at != 0
                                                        && !(m && lt(points[at - 1])))
                                                {
                                                    return None;
                                                }
                                                let available = (at..points.len())
                                                    .take_while(|&p| {
                                                        member(case.1, points[p])
                                                            && !case.10.is_some_and(|positive| (word(p.checked_sub(1).and_then(|n|points.get(n))) != word(points.get(p))) != positive)
                                                            && !case.11.is_some_and(|positive| (word(points.get(p)) != word(points.get(p+1))) != positive)
                                                            && (!case.8
                                                                || p == 0
                                                                || (m && lt(points[p - 1])))
                                                            && (!case.9
                                                                || p + 1 == points.len()
                                                                || (m && lt(points[p + 1])))
                                                    })
                                                    .count();
                                                let upper =
                                                    available.min(case.3.unwrap_or(available));
                                                if upper < case.2 {
                                                    return None;
                                                }
                                                let endpoint = |n: usize| {
                                                    let end = at + n;
                                                    if case.7
                                                        && end != points.len()
                                                        && !(m && lt(points[end]))
                                                    {
                                                        return None;
                                                    }
                                                    Some(RegExpUnicodeRepeatedAtomMatch {
                                                        range: bounds[at]..bounds[end],
                                                        atom_capture: (case.5 != 0 && n != 0)
                                                            .then(|| bounds[end - 1]..bounds[end]),
                                                    })
                                                };
                                                let mut counts = case.2..=upper;
                                                if case.4 {
                                                    counts.rev().find_map(endpoint)
                                                } else {
                                                    counts.find_map(endpoint)
                                                }
                                            })
                                        };
                                        assert_eq!(matcher.capture_count(), case.5, "{}", case.0);
                                        assert_eq!(
                                            matcher
                                                .find_with_captures_and_work(
                                                    &input,
                                                    start,
                                                    sticky,
                                                    |_| Ok::<(), ()>(())
                                                )
                                                .unwrap(),
                                            expected,
                                            "{} i={i} m={m} s={s} {input:?} start={start} sticky={sticky}",
                                            case.0
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

    #[test]
    fn unicode_repeated_atom_nested_assertion_group_work_clones_and_scope_contracts() {
        let source = format!(
            r"{}^\B{}.{}\B${}+",
            "(".repeat(1500),
            "(".repeat(1500),
            ")".repeat(1500),
            ")".repeat(1500)
        );
        let matcher = plan(&source, true, false, true, true).clone();
        let input = JsString::from("\n".repeat(200000).as_str());
        let mut work = 0;
        assert_eq!(matcher.capture_count(), 3000);
        assert_eq!(
            matcher
                .find_with_captures_and_work(&input, 0, false, |n| {
                    work += n;
                    Ok::<(), ()>(())
                })
                .unwrap(),
            Some(RegExpUnicodeRepeatedAtomMatch {
                range: 0..200000,
                atom_capture: Some(199999..200000)
            })
        );
        assert!(work < 50 * input.len(), "{work}");
        let matcher = plan(r"(\b(a)){2}", false, false, false, false);
        let input = JsString::from("a".repeat(200000).as_str());
        work = 0;
        assert_eq!(
            matcher
                .find_with_work(&input, 0, false, |n| {
                    work += n;
                    Ok::<(), ()>(())
                })
                .unwrap(),
            None
        );
        assert!(work < 50 * input.len(), "{work}");
        let matcher = plan(r"(^\B(.)\B$){2,3}$", true, false, true, true);
        let input = JsString::from("\r\n");
        let mut calls = 0;
        matcher
            .find_with_captures_and_work(&input, 0, false, |_| {
                calls += 1;
                Ok::<(), ()>(())
            })
            .unwrap();
        for fail in 1..=calls {
            let mut n = 0;
            assert!(
                matcher
                    .find_with_captures_and_work(&input, 0, false, |_| {
                        n += 1;
                        if n == fail { Err(()) } else { Ok(()) }
                    })
                    .is_err()
            );
        }
        for source in [
            r"(\b)+",
            r"(^\B$)+",
            r"(\b(a?))+",
            r"(\b(a|b))+",
            r"(\b(ab))+",
            r"(\b(a))+b",
            r"\b(\b(a))+",
            r"(\b(a))+\b",
            r"(a^\b)+",
            r"(\ba(b))+",
            r"(?i:\ba)+",
            r"((\b)a)+",
            r"(a(\b))+",
            r"(\b\b(a))+",
            r"(\b(a)\b\b)+",
            r"((a)(b))+",
            r"(()a)+",
        ] {
            assert!(
                RegExpUnicodeRepeatedAtomMatcher::compile_nested_assertion_groups_with_work(
                    &JsString::from(source),
                    true,
                    false,
                    true,
                    true,
                    |_| Ok::<(), ()>(())
                )
                .unwrap()
                .is_none(),
                "{source}"
            );
        }
        for source in [r"(\b(a))+", r"((a)\b)+", r"(^\B(.)\B$)+"] {
            assert!(
                RegExpUnicodeRepeatedAtomMatcher::compile_capturing_atom_with_assertions_and_work(
                    &JsString::from(source),
                    true,
                    false,
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
}

#[cfg(test)]
mod nested_assertion_word_tests {
    use super::*;
    use std::fmt::Write;

    type Case = (
        &'static str,
        u8,
        usize,
        Option<usize>,
        bool,
        usize,
        bool,
        bool,
        bool,
        bool,
        Option<bool>,
        Option<bool>,
        Option<bool>,
        Option<bool>,
    );
    const CASES: [Case; 16] = [
        (
            r"\b(\b(a))+\B",
            0,
            1,
            None,
            true,
            2,
            false,
            false,
            false,
            false,
            Some(true),
            None,
            Some(true),
            Some(false),
        ),
        (
            r"\B((a)\b){0,3}?\b",
            0,
            0,
            Some(3),
            false,
            2,
            false,
            false,
            false,
            false,
            None,
            Some(true),
            Some(false),
            Some(true),
        ),
        (
            r"\b(?:\b(a)\b){2,4}\b",
            0,
            2,
            Some(4),
            true,
            1,
            false,
            false,
            false,
            false,
            Some(true),
            Some(true),
            Some(true),
            Some(true),
        ),
        (
            r"\B(?:\B(?:a))+",
            0,
            1,
            None,
            true,
            0,
            false,
            false,
            false,
            false,
            Some(false),
            None,
            Some(false),
            None,
        ),
        (
            r"((?:a)\B){0}\B",
            0,
            0,
            Some(0),
            true,
            1,
            false,
            false,
            false,
            false,
            None,
            Some(false),
            None,
            Some(false),
        ),
        (
            r"\b(\b^([aſK]))+\b",
            1,
            1,
            None,
            true,
            2,
            false,
            false,
            true,
            false,
            Some(true),
            None,
            Some(true),
            Some(true),
        ),
        (
            r"((.)$\B){1,3}\B$",
            2,
            1,
            Some(3),
            true,
            2,
            false,
            true,
            false,
            true,
            None,
            Some(false),
            None,
            Some(false),
        ),
        (
            r"^\b((^\b(\D)\B$)){0,2}?\B$",
            3,
            0,
            Some(2),
            false,
            3,
            true,
            true,
            true,
            true,
            Some(true),
            Some(false),
            Some(true),
            Some(false),
        ),
        (
            r"^\B(\B(\p{Join_C})\B)+\B$",
            4,
            1,
            None,
            true,
            2,
            true,
            true,
            false,
            false,
            Some(false),
            Some(false),
            Some(false),
            Some(false),
        ),
        (
            r"((\p{NChar})\B$){1,2}\B$",
            5,
            1,
            Some(2),
            true,
            2,
            false,
            true,
            false,
            true,
            None,
            Some(false),
            None,
            Some(false),
        ),
        (
            r"\B(\B(😀)\B)+\B",
            6,
            1,
            None,
            true,
            2,
            false,
            false,
            false,
            false,
            Some(false),
            Some(false),
            Some(false),
            Some(false),
        ),
        (
            r"((\uD800)\b){0,1}\b",
            7,
            0,
            Some(1),
            true,
            2,
            false,
            false,
            false,
            false,
            None,
            Some(true),
            None,
            Some(true),
        ),
        (
            r"\b(\b(\?))??",
            8,
            0,
            Some(1),
            false,
            2,
            false,
            false,
            false,
            false,
            Some(true),
            None,
            Some(true),
            None,
        ),
        (
            r"\B(?:(\B(\W)\B$))+\B",
            9,
            1,
            None,
            true,
            2,
            false,
            false,
            false,
            true,
            Some(false),
            Some(false),
            Some(false),
            Some(false),
        ),
        (
            r"((\u{61})\b)?\b",
            0,
            0,
            Some(1),
            true,
            2,
            false,
            false,
            false,
            false,
            None,
            Some(true),
            None,
            Some(true),
        ),
        (
            r"\b((^\b(a)\b)){1,3}\b$",
            0,
            1,
            Some(3),
            true,
            3,
            false,
            true,
            true,
            false,
            Some(true),
            Some(true),
            Some(true),
            Some(true),
        ),
    ];
    fn plan(source: &str, v: bool, i: bool, m: bool, s: bool) -> RegExpUnicodeRepeatedAtomMatcher {
        RegExpUnicodeRepeatedAtomMatcher::compile_with_nested_assertion_word_boundaries_and_work(
            &JsString::from(source),
            v,
            i,
            m,
            s,
            |_| Ok::<(), ()>(()),
        )
        .unwrap()
        .unwrap()
    }

    #[test]
    fn unicode_repeated_atom_nested_assertion_word_ranges_snapshot() {
        let mut rows = String::new();
        for (source, ..) in CASES {
            let pattern = JsString::from(source);
            for v in [false, true] {
                for i in [false, true] {
                    for m in [false, true] {
                        for s in [false, true] {
                            let matcher = plan(source, v, i, m, s);
                            for input in [
                                JsString::from("Aaaa\nAA"),
                                JsString::from("A\r\n\u{200c}\u{200d}\u{ffff}\u{10ffff}"),
                                JsString::from("😀??aaſK"),
                                JsString::from_code_units(vec![0x61, 0xd800, 0x61, 0xdc00]),
                                JsString::from(""),
                            ] {
                                for start in [0, 1, input.len(), input.len() + 1] {
                                    for sticky in [false, true] {
                                        let found = matcher
                                            .find_with_captures_and_work(
                                                &input,
                                                start,
                                                sticky,
                                                |_| Ok::<(), ()>(()),
                                            )
                                            .unwrap();
                                        writeln!(rows,"{pattern:?} v={v} i={i} m={m} s={s} input={input:?} start={start} sticky={sticky} count={} range={:?} capture={:?}",matcher.capture_count(),found.as_ref().map(|f|f.range.clone()),found.and_then(|f|f.atom_capture)).unwrap();
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        insta::assert_snapshot!(rows);
    }

    #[test]
    fn unicode_repeated_atom_nested_assertion_words_agree_with_independent_counts_and_final_character()
     {
        let alphabet: [u16; 10] = [
            0x41, 0x61, 10, 13, 0x17f, 0x212a, 0xd83d, 0xde00, 0xd800, 0xdc00,
        ];
        let lt = |c| [10, 13, 0x2028, 0x2029].contains(&c);
        for i in [false, true] {
            for m in [false, true] {
                for s in [false, true] {
                    let plans: Vec<_> = CASES.iter().map(|c| plan(c.0, true, i, m, s)).collect();
                    let fold = |c| {
                        if i {
                            match c {
                                0x41 => 0x61,
                                0x17f => 0x73,
                                0x212a => 0x6b,
                                _ => c,
                            }
                        } else {
                            c
                        }
                    };
                    let member = |kind, c| match kind {
                        0 => fold(c) == 0x61,
                        1 => [0x61, fold(0x17f), fold(0x212a)].contains(&fold(c)),
                        2 => s || !lt(c),
                        3 => !(0x30..=0x39).contains(&c),
                        4 => (0x200c..=0x200d).contains(&c),
                        5 => matches!(c, 0xfdd0..=0xfdef) || (c & 0xffff) >= 0xfffe,
                        6 => c == 0x1f600,
                        7 => c == 0xd800,
                        8 => c == 0x3f,
                        9 => !char::from_u32(fold(c))
                            .is_some_and(|c| c.is_ascii_alphanumeric() || c == '_'),
                        _ => unreachable!(),
                    };
                    for len in 0..=4 {
                        for ordinal in 0..alphabet.len().pow(len) {
                            let units: Vec<_> = (0..len)
                                .map(|j| alphabet[ordinal / alphabet.len().pow(j) % alphabet.len()])
                                .collect();
                            let mut points = Vec::new();
                            let mut bounds = vec![0];
                            let mut j = 0;
                            while j < units.len() {
                                let mut c = u32::from(units[j]);
                                j += 1;
                                if (0xd800..=0xdbff).contains(&c)
                                    && j < units.len()
                                    && (0xdc00..=0xdfff).contains(&units[j])
                                {
                                    c = 65536 + (c - 0xd800) * 1024 + u32::from(units[j]) - 0xdc00;
                                    j += 1;
                                }
                                points.push(c);
                                bounds.push(j);
                            }
                            let input = JsString::from_code_units(units);
                            for start in 0..=input.len() + 1 {
                                for sticky in [false, true] {
                                    for (case, matcher) in CASES.iter().zip(&plans) {
                                        let expected = if start > input.len() {
                                            None
                                        } else {
                                            let first = bounds.partition_point(|&b| b <= start) - 1;
                                            (first..=points.len()).find_map(|at| {
                                                let word=|c:Option<&u32>|c.is_some_and(|&c|matches!(c,0x30..=0x39|0x41..=0x5a|0x5f|0x61..=0x7a)||(i&&matches!(c,0x17f|0x212a)));
                                                let start_boundary=word(at.checked_sub(1).and_then(|n|points.get(n)))!=word(points.get(at));
                                                if case.12.is_some_and(|expected|start_boundary!=expected) || (sticky && at != first)
                                                    || (case.6
                                                        && at != 0
                                                        && !(m && lt(points[at - 1])))
                                                {
                                                    return None;
                                                }
                                                let available = (at..points.len())
                                                    .take_while(|&p| {
                                                        member(case.1, points[p])
                                                            && !case.10.is_some_and(|positive| (word(p.checked_sub(1).and_then(|n|points.get(n))) != word(points.get(p))) != positive)
                                                            && !case.11.is_some_and(|positive| (word(points.get(p)) != word(points.get(p+1))) != positive)
                                                            && (!case.8
                                                                || p == 0
                                                                || (m && lt(points[p - 1])))
                                                            && (!case.9
                                                                || p + 1 == points.len()
                                                                || (m && lt(points[p + 1])))
                                                    })
                                                    .count();
                                                let upper =
                                                    available.min(case.3.unwrap_or(available));
                                                if upper < case.2 {
                                                    return None;
                                                }
                                                let endpoint = |n: usize| {
                                                    let end = at + n;
                                                    if case.7
                                                        && end != points.len()
                                                        && !(m && lt(points[end]))
                                                    {
                                                        return None;
                                                    }
                                                    let end_boundary=word(end.checked_sub(1).and_then(|n|points.get(n)))!=word(points.get(end));
                                                    if case.13.is_some_and(|expected|end_boundary!=expected){return None;}
                                                    Some(RegExpUnicodeRepeatedAtomMatch {
                                                        range: bounds[at]..bounds[end],
                                                        atom_capture: (case.5 != 0 && n != 0)
                                                            .then(|| bounds[end - 1]..bounds[end]),
                                                    })
                                                };
                                                let mut counts = case.2..=upper;
                                                if case.4 {
                                                    counts.rev().find_map(endpoint)
                                                } else {
                                                    counts.find_map(endpoint)
                                                }
                                            })
                                        };
                                        assert_eq!(matcher.capture_count(), case.5, "{}", case.0);
                                        assert_eq!(
                                            matcher
                                                .find_with_captures_and_work(
                                                    &input,
                                                    start,
                                                    sticky,
                                                    |_| Ok::<(), ()>(())
                                                )
                                                .unwrap(),
                                            expected,
                                            "{} i={i} m={m} s={s} {input:?} start={start} sticky={sticky}",
                                            case.0
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

    #[test]
    fn unicode_repeated_atom_nested_assertion_word_work_clones_and_scope_contracts() {
        let source = format!(
            r"\B{}^\B{}.{}\B${}+\B",
            "(".repeat(1500),
            "(".repeat(1500),
            ")".repeat(1500),
            ")".repeat(1500)
        );
        let matcher = plan(&source, true, false, true, true).clone();
        let input = JsString::from("\n".repeat(200000).as_str());
        let mut work = 0;
        assert_eq!(matcher.capture_count(), 3000);
        assert_eq!(
            matcher
                .find_with_captures_and_work(&input, 0, false, |n| {
                    work += n;
                    Ok::<(), ()>(())
                })
                .unwrap(),
            Some(RegExpUnicodeRepeatedAtomMatch {
                range: 0..200000,
                atom_capture: Some(199999..200000)
            })
        );
        assert!(work < 50 * input.len(), "{work}");
        let matcher = plan(r"\b(\b(a)){2}\b", false, false, false, false);
        let input = JsString::from("a".repeat(200000).as_str());
        work = 0;
        assert_eq!(
            matcher
                .find_with_work(&input, 0, false, |n| {
                    work += n;
                    Ok::<(), ()>(())
                })
                .unwrap(),
            None
        );
        assert!(work < 50 * input.len(), "{work}");
        let matcher = plan(r"\B(^\B(.)\B$){2,3}\B$", true, false, true, true);
        let input = JsString::from("\r\n");
        let mut calls = 0;
        matcher
            .find_with_captures_and_work(&input, 0, false, |_| {
                calls += 1;
                Ok::<(), ()>(())
            })
            .unwrap();
        for fail in 1..=calls {
            let mut n = 0;
            assert!(
                matcher
                    .find_with_captures_and_work(&input, 0, false, |_| {
                        n += 1;
                        if n == fail { Err(()) } else { Ok(()) }
                    })
                    .is_err()
            );
        }
        for source in [
            r"(\b(a))+",
            r"\b(\b)+",
            r"\B(^\B$)+",
            r"\b(\b(a?))+",
            r"\b(\b(a|b))+",
            r"\b(\b(ab))+",
            r"\b(\b(a))+b",
            r"\b(a^\b)+",
            r"\b(\ba(b))+",
            r"\b(?i:\ba)+",
            r"\b((\b)a)+",
            r"\b(\b\b(a))+",
        ] {
            assert!(RegExpUnicodeRepeatedAtomMatcher::compile_with_nested_assertion_word_boundaries_and_work(&JsString::from(source),true,false,true,true,|_|Ok::<(),()>(())).unwrap().is_none(),"{source}");
        }
        for source in [r"\b(\b(a))+", r"((a)\b)+\b", r"\B(^\B(.)\B$)+\B"] {
            assert!(
                RegExpUnicodeRepeatedAtomMatcher::compile_with_composed_word_boundaries_and_work(
                    &JsString::from(source),
                    true,
                    false,
                    true,
                    true,
                    |_| Ok::<(), ()>(())
                )
                .unwrap()
                .is_none(),
                "{source}"
            );
            assert!(
                RegExpUnicodeRepeatedAtomMatcher::compile_nested_assertion_groups_with_work(
                    &JsString::from(source),
                    true,
                    false,
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
}

#[cfg(test)]
mod alternative_tests {
    use super::*;
    use std::fmt::Write;

    type Case = (
        &'static str,
        u8,
        usize,
        Option<usize>,
        bool,
        usize,
        bool,
        bool,
    );
    const CASES: [Case; 16] = [
        (r"(a|b)+", 0, 1, None, true, 1, false, false),
        (r"((a|[ſK])){0,3}?", 1, 0, Some(3), false, 2, false, false),
        (r"(?:.|\n){2,4}", 2, 2, Some(4), true, 0, false, false),
        (r"(\d|\D)*", 3, 0, None, true, 1, false, false),
        (
            r"^(\p{Join_C}|\p{NChar})+$",
            4,
            1,
            None,
            true,
            1,
            true,
            true,
        ),
        (r"(😀|a){1,3}$", 5, 1, Some(3), true, 1, false, true),
        (
            r"(\uD800|\uDC00){0,2}?",
            6,
            0,
            Some(2),
            false,
            1,
            false,
            false,
        ),
        (r"(\||a)+", 7, 1, None, true, 1, false, false),
        (r"(\ba|b\b)+", 8, 1, None, true, 1, false, false),
        (r"(^a|b$){1,2}$", 9, 1, Some(2), true, 1, false, true),
        (r"([a\|]|b)??", 10, 0, Some(1), false, 1, false, false),
        (r"(\B.|\b[aſK]){1,3}", 11, 1, Some(3), true, 1, false, false),
        (r"(a$|^b)+", 12, 1, None, true, 1, false, false),
        (
            r"^([a-z]|\p{Join_C}){0,2}$",
            13,
            0,
            Some(2),
            true,
            1,
            true,
            true,
        ),
        (
            r"(\u{61}|\u{1f600})?",
            14,
            0,
            Some(1),
            true,
            1,
            false,
            false,
        ),
        (r"(^\B.$|\ba\b){1,3}$", 15, 1, Some(3), true, 1, false, true),
    ];
    fn plan(source: &str, v: bool, i: bool, m: bool, s: bool) -> RegExpUnicodeRepeatedAtomMatcher {
        RegExpUnicodeRepeatedAtomMatcher::compile_capturing_alternatives_with_work(
            &JsString::from(source),
            v,
            i,
            m,
            s,
            |_| Ok::<(), ()>(()),
        )
        .unwrap()
        .unwrap()
    }

    #[test]
    fn unicode_repeated_atom_alternative_ranges_snapshot() {
        let mut rows = String::new();
        for (source, ..) in CASES {
            let pattern = JsString::from(source);
            for v in [false, true] {
                for i in [false, true] {
                    for m in [false, true] {
                        for s in [false, true] {
                            let matcher = plan(source, v, i, m, s);
                            for input in [
                                JsString::from("Aaaa\nAA"),
                                JsString::from("A\r\n\u{200c}\u{200d}\u{ffff}\u{10ffff}"),
                                JsString::from("😀??aaſK"),
                                JsString::from_code_units(vec![0x61, 0xd800, 0x61, 0xdc00]),
                                JsString::from(""),
                            ] {
                                for start in [0, 1, input.len(), input.len() + 1] {
                                    for sticky in [false, true] {
                                        let found = matcher
                                            .find_with_captures_and_work(
                                                &input,
                                                start,
                                                sticky,
                                                |_| Ok::<(), ()>(()),
                                            )
                                            .unwrap();
                                        writeln!(rows,"{pattern:?} v={v} i={i} m={m} s={s} input={input:?} start={start} sticky={sticky} count={} range={:?} capture={:?}",matcher.capture_count(),found.as_ref().map(|f|f.range.clone()),found.and_then(|f|f.atom_capture)).unwrap();
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        insta::assert_snapshot!(rows);
    }

    #[test]
    fn unicode_repeated_atom_alternatives_agree_with_independent_counts_and_final_character() {
        let alphabet: [u16; 10] = [
            0x41, 0x61, 10, 13, 0x17f, 0x212a, 0xd83d, 0xde00, 0xd800, 0xdc00,
        ];
        let lt = |c| [10, 13, 0x2028, 0x2029].contains(&c);
        for i in [false, true] {
            for m in [false, true] {
                for s in [false, true] {
                    let plans: Vec<_> = CASES.iter().map(|c| plan(c.0, true, i, m, s)).collect();
                    let fold = |c| {
                        if i {
                            match c {
                                0x41 => 0x61,
                                0x17f => 0x73,
                                0x212a => 0x6b,
                                _ => c,
                            }
                        } else {
                            c
                        }
                    };
                    for len in 0..=4 {
                        for ordinal in 0..alphabet.len().pow(len) {
                            let units: Vec<_> = (0..len)
                                .map(|j| alphabet[ordinal / alphabet.len().pow(j) % alphabet.len()])
                                .collect();
                            let mut points = Vec::new();
                            let mut bounds = vec![0];
                            let mut j = 0;
                            while j < units.len() {
                                let mut c = u32::from(units[j]);
                                j += 1;
                                if (0xd800..=0xdbff).contains(&c)
                                    && j < units.len()
                                    && (0xdc00..=0xdfff).contains(&units[j])
                                {
                                    c = 65536 + (c - 0xd800) * 1024 + u32::from(units[j]) - 0xdc00;
                                    j += 1;
                                }
                                points.push(c);
                                bounds.push(j);
                            }
                            let input = JsString::from_code_units(units);
                            for start in 0..=input.len() + 1 {
                                for sticky in [false, true] {
                                    for (case, matcher) in CASES.iter().zip(&plans) {
                                        let expected = if start > input.len() {
                                            None
                                        } else {
                                            let first = bounds.partition_point(|&b| b <= start) - 1;
                                            (first..=points.len()).find_map(|at| {
                                                let word=|c:Option<&u32>|c.is_some_and(|&c|matches!(c,0x30..=0x39|0x41..=0x5a|0x5f|0x61..=0x7a)||(i&&matches!(c,0x17f|0x212a)));
                                                if (sticky && at != first)
                                                    || (case.6
                                                        && at != 0
                                                        && !(m && lt(points[at - 1])))
                                                {
                                                    return None;
                                                }
                                                let available = (at..points.len()).take_while(|&p| {
                                                    let c = points[p];
                                                    let prev = p.checked_sub(1).and_then(|n| points.get(n));
                                                    let next = points.get(p+1);
                                                    let wb = word(prev) != word(Some(&c));
                                                    let we = word(Some(&c)) != word(next);
                                                    let begin = p == 0 || (m && prev.is_some_and(|&c| lt(c)));
                                                    let end = next.is_none() || (m && next.is_some_and(|&c| lt(c)));
                                                    let dot = s || !lt(c);
                                                    let class = [0x61, fold(0x17f), fold(0x212a)].contains(&fold(c));
                                                    match case.1 {
                                                        0 => [0x61,0x62].contains(&fold(c)),
                                                        1 => class,
                                                        2 => dot || c == 10,
                                                        3 => true,
                                                        4 => (0x200c..=0x200d).contains(&c) || matches!(c,0xfdd0..=0xfdef) || c&0xffff >= 0xfffe,
                                                        5 | 14 => c == 0x1f600 || fold(c) == 0x61,
                                                        6 => [0xd800,0xdc00].contains(&c),
                                                        7 => c == 0x7c || fold(c) == 0x61,
                                                        8 => (fold(c) == 0x61 && wb) || (fold(c) == 0x62 && we),
                                                        9 => (fold(c) == 0x61 && begin) || (fold(c) == 0x62 && end),
                                                        10 => [0x61,0x62,0x7c].contains(&fold(c)),
                                                        11 => (dot && !wb) || (class && wb),
                                                        12 => (fold(c) == 0x61 && end) || (fold(c) == 0x62 && begin),
                                                        13 => (0x61..=0x7a).contains(&fold(c)) || (0x200c..=0x200d).contains(&c),
                                                        15 => (dot && begin && end && !wb) || (fold(c) == 0x61 && wb && we),
                                                        _ => unreachable!(),
                                                    }
                                                }).count();
                                                let upper =
                                                    available.min(case.3.unwrap_or(available));
                                                if upper < case.2 {
                                                    return None;
                                                }
                                                let endpoint = |n: usize| {
                                                    let end = at + n;
                                                    if case.7
                                                        && end != points.len()
                                                        && !(m && lt(points[end]))
                                                    {
                                                        return None;
                                                    }
                                                    Some(RegExpUnicodeRepeatedAtomMatch {
                                                        range: bounds[at]..bounds[end],
                                                        atom_capture: (case.5 != 0 && n != 0)
                                                            .then(|| bounds[end - 1]..bounds[end]),
                                                    })
                                                };
                                                let mut counts = case.2..=upper;
                                                if case.4 {
                                                    counts.rev().find_map(endpoint)
                                                } else {
                                                    counts.find_map(endpoint)
                                                }
                                            })
                                        };
                                        assert_eq!(matcher.capture_count(), case.5, "{}", case.0);
                                        assert_eq!(
                                            matcher
                                                .find_with_captures_and_work(
                                                    &input,
                                                    start,
                                                    sticky,
                                                    |_| Ok::<(), ()>(())
                                                )
                                                .unwrap(),
                                            expected,
                                            "{} i={i} m={m} s={s} {input:?} start={start} sticky={sticky}",
                                            case.0
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

    #[test]
    fn unicode_repeated_atom_alternative_work_clones_and_contracts() {
        let source = format!("{}.|\\n{}+", "(".repeat(3000), ")".repeat(3000));
        let matcher = plan(&source, true, false, true, false).clone();
        let input = JsString::from("\n".repeat(200000).as_str());
        let mut work = 0;
        assert_eq!(matcher.capture_count(), 3000);
        assert_eq!(
            matcher
                .find_with_captures_and_work(&input, 0, false, |n| {
                    work += n;
                    Ok::<(), ()>(())
                })
                .unwrap(),
            Some(RegExpUnicodeRepeatedAtomMatch {
                range: 0..200000,
                atom_capture: Some(199999..200000)
            })
        );
        assert!(work < 50 * input.len(), "{work}");
        let many = format!("({}|b)+", "a|".repeat(20000).trim_end_matches('|'));
        assert_eq!(
            plan(&many, false, false, false, false)
                .find_with_work(&JsString::from("b"), 0, true, |_| Ok::<(), ()>(()))
                .unwrap(),
            Some(0..1)
        );
        let source = JsString::from(r"(^\B.$|\ba\b){1,3}$");
        let mut calls = 0;
        RegExpUnicodeRepeatedAtomMatcher::compile_capturing_alternatives_with_work(
            &source,
            true,
            false,
            true,
            true,
            |_| {
                calls += 1;
                Ok::<(), ()>(())
            },
        )
        .unwrap()
        .unwrap();
        for fail in 1..=calls {
            let mut n = 0;
            assert!(
                RegExpUnicodeRepeatedAtomMatcher::compile_capturing_alternatives_with_work(
                    &source,
                    true,
                    false,
                    true,
                    true,
                    |_| {
                        n += 1;
                        if n == fail { Err(()) } else { Ok(()) }
                    }
                )
                .is_err()
            );
        }
        let matcher = plan(r"(^\B.$|\ba\b){1,3}$", true, false, true, true);
        let input = JsString::from("\r\n");
        calls = 0;
        matcher
            .find_with_captures_and_work(&input, 0, false, |_| {
                calls += 1;
                Ok::<(), ()>(())
            })
            .unwrap();
        for fail in 1..=calls {
            let mut n = 0;
            assert!(
                matcher
                    .find_with_captures_and_work(&input, 0, false, |_| {
                        n += 1;
                        if n == fail { Err(()) } else { Ok(()) }
                    })
                    .is_err()
            );
        }
        for source in [
            r"(a|)+",
            r"(|b)+",
            r"(a|ab)+",
            r"(a|b?)+",
            r"(a|(b))+",
            r"(a|\b)+",
            r"(a|(?i:b))+",
            r"(a|b)+c",
            r"\b(a|b)+",
            r"(a|b)+\b",
            r"(\b(a|b))+",
            r"((a)|(b))+",
            r"(a|b){2}c",
        ] {
            assert!(
                RegExpUnicodeRepeatedAtomMatcher::compile_capturing_alternatives_with_work(
                    &JsString::from(source),
                    true,
                    false,
                    true,
                    true,
                    |_| Ok::<(), ()>(())
                )
                .unwrap()
                .is_none(),
                "{source}"
            );
        }
        for source in [r"(a|b)+", r"(^a|b$)+", r"(\ba|b\b)+"] {
            assert!(
                RegExpUnicodeRepeatedAtomMatcher::compile_capturing_atom_with_assertions_and_work(
                    &JsString::from(source),
                    true,
                    false,
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
}

#[cfg(test)]
mod word_alternative_tests {
    use super::*;
    use std::fmt::Write;

    type Case = (
        &'static str,
        u8,
        usize,
        Option<usize>,
        bool,
        usize,
        bool,
        bool,
        Option<bool>,
        Option<bool>,
    );
    const CASES: [Case; 16] = [
        (
            r"\b(a|b)+\B",
            0,
            1,
            None,
            true,
            1,
            false,
            false,
            Some(true),
            Some(false),
        ),
        (
            r"\B((a|[ſK])){0,3}?\b",
            1,
            0,
            Some(3),
            false,
            2,
            false,
            false,
            Some(false),
            Some(true),
        ),
        (
            r"\b(?:.|\n){2,4}\b",
            2,
            2,
            Some(4),
            true,
            0,
            false,
            false,
            Some(true),
            Some(true),
        ),
        (
            r"\B(\d|\D)*",
            3,
            0,
            None,
            true,
            1,
            false,
            false,
            Some(false),
            None,
        ),
        (
            r"^(\p{Join_C}|\p{NChar})+\B$",
            4,
            1,
            None,
            true,
            1,
            true,
            true,
            None,
            Some(false),
        ),
        (
            r"\b(😀|a){1,3}\b$",
            5,
            1,
            Some(3),
            true,
            1,
            false,
            true,
            Some(true),
            Some(true),
        ),
        (
            r"(\uD800|\uDC00){0,2}?\b",
            6,
            0,
            Some(2),
            false,
            1,
            false,
            false,
            None,
            Some(true),
        ),
        (
            r"\b(\||a)+",
            7,
            1,
            None,
            true,
            1,
            false,
            false,
            Some(true),
            None,
        ),
        (
            r"\b(\ba|b\b)+\B",
            8,
            1,
            None,
            true,
            1,
            false,
            false,
            Some(true),
            Some(false),
        ),
        (
            r"(^a|b$){1,2}\b$",
            9,
            1,
            Some(2),
            true,
            1,
            false,
            true,
            None,
            Some(true),
        ),
        (
            r"\B([a\|]|b)??\B",
            10,
            0,
            Some(1),
            false,
            1,
            false,
            false,
            Some(false),
            Some(false),
        ),
        (
            r"\B(\B.|\b[aſK]){1,3}\b",
            11,
            1,
            Some(3),
            true,
            1,
            false,
            false,
            Some(false),
            Some(true),
        ),
        (
            r"(a$|^b)+\b",
            12,
            1,
            None,
            true,
            1,
            false,
            false,
            None,
            Some(true),
        ),
        (
            r"^\b([a-z]|\p{Join_C}){0,2}\B$",
            13,
            0,
            Some(2),
            true,
            1,
            true,
            true,
            Some(true),
            Some(false),
        ),
        (
            r"(\u{61}|\u{1f600})?\b",
            14,
            0,
            Some(1),
            true,
            1,
            false,
            false,
            None,
            Some(true),
        ),
        (
            r"\B(^\B.$|\ba\b){1,3}\B$",
            15,
            1,
            Some(3),
            true,
            1,
            false,
            true,
            Some(false),
            Some(false),
        ),
    ];
    fn plan(source: &str, v: bool, i: bool, m: bool, s: bool) -> RegExpUnicodeRepeatedAtomMatcher {
        RegExpUnicodeRepeatedAtomMatcher::compile_alternatives_with_word_boundaries_and_work(
            &JsString::from(source),
            v,
            i,
            m,
            s,
            |_| Ok::<(), ()>(()),
        )
        .unwrap()
        .unwrap()
    }

    #[test]
    fn unicode_repeated_atom_word_alternative_ranges_snapshot() {
        let mut rows = String::new();
        for (source, ..) in CASES {
            let pattern = JsString::from(source);
            for v in [false, true] {
                for i in [false, true] {
                    for m in [false, true] {
                        for s in [false, true] {
                            let matcher = plan(source, v, i, m, s);
                            for input in [
                                JsString::from("Aaaa\nAA"),
                                JsString::from("A\r\n\u{200c}\u{200d}\u{ffff}\u{10ffff}"),
                                JsString::from("😀??aaſK"),
                                JsString::from_code_units(vec![0x61, 0xd800, 0x61, 0xdc00]),
                                JsString::from(""),
                            ] {
                                for start in [0, 1, input.len(), input.len() + 1] {
                                    for sticky in [false, true] {
                                        let found = matcher
                                            .find_with_captures_and_work(
                                                &input,
                                                start,
                                                sticky,
                                                |_| Ok::<(), ()>(()),
                                            )
                                            .unwrap();
                                        writeln!(rows,"{pattern:?} v={v} i={i} m={m} s={s} input={input:?} start={start} sticky={sticky} count={} range={:?} capture={:?}",matcher.capture_count(),found.as_ref().map(|f|f.range.clone()),found.and_then(|f|f.atom_capture)).unwrap();
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        insta::assert_snapshot!(rows);
    }

    #[test]
    fn unicode_repeated_atom_word_alternatives_agree_with_independent_counts_and_final_character() {
        let alphabet: [u16; 10] = [
            0x41, 0x61, 10, 13, 0x17f, 0x212a, 0xd83d, 0xde00, 0xd800, 0xdc00,
        ];
        let lt = |c| [10, 13, 0x2028, 0x2029].contains(&c);
        for i in [false, true] {
            for m in [false, true] {
                for s in [false, true] {
                    let plans: Vec<_> = CASES.iter().map(|c| plan(c.0, true, i, m, s)).collect();
                    let fold = |c| {
                        if i {
                            match c {
                                0x41 => 0x61,
                                0x17f => 0x73,
                                0x212a => 0x6b,
                                _ => c,
                            }
                        } else {
                            c
                        }
                    };
                    for len in 0..=4 {
                        for ordinal in 0..alphabet.len().pow(len) {
                            let units: Vec<_> = (0..len)
                                .map(|j| alphabet[ordinal / alphabet.len().pow(j) % alphabet.len()])
                                .collect();
                            let mut points = Vec::new();
                            let mut bounds = vec![0];
                            let mut j = 0;
                            while j < units.len() {
                                let mut c = u32::from(units[j]);
                                j += 1;
                                if (0xd800..=0xdbff).contains(&c)
                                    && j < units.len()
                                    && (0xdc00..=0xdfff).contains(&units[j])
                                {
                                    c = 65536 + (c - 0xd800) * 1024 + u32::from(units[j]) - 0xdc00;
                                    j += 1;
                                }
                                points.push(c);
                                bounds.push(j);
                            }
                            let input = JsString::from_code_units(units);
                            for start in 0..=input.len() + 1 {
                                for sticky in [false, true] {
                                    for (case, matcher) in CASES.iter().zip(&plans) {
                                        let expected = if start > input.len() {
                                            None
                                        } else {
                                            let first = bounds.partition_point(|&b| b <= start) - 1;
                                            (first..=points.len()).find_map(|at| {
                                                let word=|c:Option<&u32>|c.is_some_and(|&c|matches!(c,0x30..=0x39|0x41..=0x5a|0x5f|0x61..=0x7a)||(i&&matches!(c,0x17f|0x212a)));
                                                if (sticky && at != first)
                                                    || (case.6
                                                        && at != 0
                                                        && !(m && lt(points[at - 1])))
                                                {
                                                    return None;
                                                }
                                                if case.8.is_some_and(|positive| (word(at.checked_sub(1).and_then(|n|points.get(n))) != word(points.get(at))) != positive) { return None; }
                                                let available = (at..points.len()).take_while(|&p| {
                                                    let c = points[p];
                                                    let prev = p.checked_sub(1).and_then(|n| points.get(n));
                                                    let next = points.get(p+1);
                                                    let wb = word(prev) != word(Some(&c));
                                                    let we = word(Some(&c)) != word(next);
                                                    let begin = p == 0 || (m && prev.is_some_and(|&c| lt(c)));
                                                    let end = next.is_none() || (m && next.is_some_and(|&c| lt(c)));
                                                    let dot = s || !lt(c);
                                                    let class = [0x61, fold(0x17f), fold(0x212a)].contains(&fold(c));
                                                    match case.1 {
                                                        0 => [0x61,0x62].contains(&fold(c)),
                                                        1 => class,
                                                        2 => dot || c == 10,
                                                        3 => true,
                                                        4 => (0x200c..=0x200d).contains(&c) || matches!(c,0xfdd0..=0xfdef) || c&0xffff >= 0xfffe,
                                                        5 | 14 => c == 0x1f600 || fold(c) == 0x61,
                                                        6 => [0xd800,0xdc00].contains(&c),
                                                        7 => c == 0x7c || fold(c) == 0x61,
                                                        8 => (fold(c) == 0x61 && wb) || (fold(c) == 0x62 && we),
                                                        9 => (fold(c) == 0x61 && begin) || (fold(c) == 0x62 && end),
                                                        10 => [0x61,0x62,0x7c].contains(&fold(c)),
                                                        11 => (dot && !wb) || (class && wb),
                                                        12 => (fold(c) == 0x61 && end) || (fold(c) == 0x62 && begin),
                                                        13 => (0x61..=0x7a).contains(&fold(c)) || (0x200c..=0x200d).contains(&c),
                                                        15 => (dot && begin && end && !wb) || (fold(c) == 0x61 && wb && we),
                                                        _ => unreachable!(),
                                                    }
                                                }).count();
                                                let upper =
                                                    available.min(case.3.unwrap_or(available));
                                                if upper < case.2 {
                                                    return None;
                                                }
                                                let endpoint = |n: usize| {
                                                    let end = at + n;
                                                    if case.9.is_some_and(|positive| (word(end.checked_sub(1).and_then(|n|points.get(n))) != word(points.get(end))) != positive) { return None; }

                                                    if case.7
                                                        && end != points.len()
                                                        && !(m && lt(points[end]))
                                                    {
                                                        return None;
                                                    }
                                                    Some(RegExpUnicodeRepeatedAtomMatch {
                                                        range: bounds[at]..bounds[end],
                                                        atom_capture: (case.5 != 0 && n != 0)
                                                            .then(|| bounds[end - 1]..bounds[end]),
                                                    })
                                                };
                                                let mut counts = case.2..=upper;
                                                if case.4 {
                                                    counts.rev().find_map(endpoint)
                                                } else {
                                                    counts.find_map(endpoint)
                                                }
                                            })
                                        };
                                        assert_eq!(matcher.capture_count(), case.5, "{}", case.0);
                                        assert_eq!(
                                            matcher
                                                .find_with_captures_and_work(
                                                    &input,
                                                    start,
                                                    sticky,
                                                    |_| Ok::<(), ()>(())
                                                )
                                                .unwrap(),
                                            expected,
                                            "{} i={i} m={m} s={s} {input:?} start={start} sticky={sticky}",
                                            case.0
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

    #[test]
    fn unicode_repeated_atom_word_alternative_work_and_contracts() {
        let source = format!(r"\B{}.|\n{}+\B$", "(".repeat(3000), ")".repeat(3000));
        let matcher = plan(&source, true, false, true, false).clone();
        let input = JsString::from("\n".repeat(200000).as_str());
        let mut work = 0;
        assert_eq!(matcher.capture_count(), 3000);
        assert_eq!(
            matcher
                .find_with_captures_and_work(&input, 0, false, |n| {
                    work += n;
                    Ok::<(), ()>(())
                })
                .unwrap(),
            Some(RegExpUnicodeRepeatedAtomMatch {
                range: 0..200000,
                atom_capture: Some(199999..200000)
            })
        );
        assert!(work < 60 * input.len(), "{work}");
        let source = JsString::from(r"\B(^\B.$|\ba\b){1,3}\B$");
        let mut calls = 0;
        RegExpUnicodeRepeatedAtomMatcher::compile_alternatives_with_word_boundaries_and_work(
            &source,
            true,
            false,
            true,
            true,
            |_| {
                calls += 1;
                Ok::<(), ()>(())
            },
        )
        .unwrap()
        .unwrap();
        for fail in 1..=calls {
            let mut n = 0;
            assert!(RegExpUnicodeRepeatedAtomMatcher::compile_alternatives_with_word_boundaries_and_work(&source,true,false,true,true,|_|{n+=1;if n==fail {Err(())} else {Ok(())}}).is_err());
        }
        let input = JsString::from("\r\n");
        let matcher = plan(r"\B(^\B.$|\ba\b){1,3}\B$", true, false, true, true);
        calls = 0;
        matcher
            .find_with_captures_and_work(&input, 0, false, |_| {
                calls += 1;
                Ok::<(), ()>(())
            })
            .unwrap();
        for fail in 1..=calls {
            let mut n = 0;
            assert!(
                matcher
                    .find_with_captures_and_work(&input, 0, false, |_| {
                        n += 1;
                        if n == fail { Err(()) } else { Ok(()) }
                    })
                    .is_err()
            );
        }
        for source in [
            r"\b(a|)+",
            r"\b(|b)+",
            r"\b(a|ab)+",
            r"\b(a|b?)+",
            r"\b(a|(b))+",
            r"\B(a|\b)+",
            r"\b(a|(?i:b))+",
            r"\b(a|b)+c",
            r"\b(\b(a|b))+",
            r"\b((a)|(b))+",
            r"\b(a|b){2}c",
        ] {
            assert!(RegExpUnicodeRepeatedAtomMatcher::compile_alternatives_with_word_boundaries_and_work(&JsString::from(source),true,false,true,true,|_|Ok::<(),()>(())).unwrap().is_none(),"{source}");
        }
        for source in [r"\b(a|b)+", r"(^a|b$)+\b", r"\b(\ba|b\b)+\B"] {
            assert!(
                RegExpUnicodeRepeatedAtomMatcher::compile_capturing_alternatives_with_work(
                    &JsString::from(source),
                    true,
                    false,
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
}

#[cfg(test)]
mod shared_alternative_assertion_tests {
    use super::*;
    use std::fmt::Write;

    type Case = (
        &'static str,
        u8,
        usize,
        Option<usize>,
        bool,
        usize,
        bool,
        bool,
        bool,
        bool,
        Option<bool>,
        Option<bool>,
    );
    const CASES: [Case; 16] = [
        (
            r"(\b(a|b)\B)+",
            0,
            1,
            None,
            true,
            2,
            false,
            false,
            false,
            false,
            Some(true),
            Some(false),
        ),
        (
            r"(\B((a|[ſK]))\b){0,3}?",
            1,
            0,
            Some(3),
            false,
            3,
            false,
            false,
            false,
            false,
            Some(false),
            Some(true),
        ),
        (
            r"(\B(?:.|\n)\B){2,4}",
            2,
            2,
            Some(4),
            true,
            1,
            false,
            false,
            false,
            false,
            Some(false),
            Some(false),
        ),
        (
            r"(\b(\d|\D)\b)*",
            3,
            0,
            None,
            true,
            2,
            false,
            false,
            false,
            false,
            Some(true),
            Some(true),
        ),
        (
            r"^(\B(\p{Join_C}|\p{NChar})\B)+$",
            4,
            1,
            None,
            true,
            2,
            true,
            true,
            false,
            false,
            Some(false),
            Some(false),
        ),
        (
            r"(\B(😀|a)\B){1,3}$",
            5,
            1,
            Some(3),
            true,
            2,
            false,
            true,
            false,
            false,
            Some(false),
            Some(false),
        ),
        (
            r"(\B(\uD800|\uDC00)\B){0,2}?",
            6,
            0,
            Some(2),
            false,
            2,
            false,
            false,
            false,
            false,
            Some(false),
            Some(false),
        ),
        (
            r"(\b(\||a)\B)+",
            7,
            1,
            None,
            true,
            2,
            false,
            false,
            false,
            false,
            Some(true),
            Some(false),
        ),
        (
            r"(\b(\ba|b\b)\B)+",
            8,
            1,
            None,
            true,
            2,
            false,
            false,
            false,
            false,
            Some(true),
            Some(false),
        ),
        (
            r"(\b^(^a|b$)$\b){1,2}$",
            9,
            1,
            Some(2),
            true,
            2,
            false,
            true,
            true,
            true,
            Some(true),
            Some(true),
        ),
        (
            r"(\B([a\|]|b)\B)??",
            10,
            0,
            Some(1),
            false,
            2,
            false,
            false,
            false,
            false,
            Some(false),
            Some(false),
        ),
        (
            r"(\B(\B.|\b[aſK])\b){1,3}",
            11,
            1,
            Some(3),
            true,
            2,
            false,
            false,
            false,
            false,
            Some(false),
            Some(true),
        ),
        (
            r"(\B(a$|^b)\b)+",
            12,
            1,
            None,
            true,
            2,
            false,
            false,
            false,
            false,
            Some(false),
            Some(true),
        ),
        (
            r"^(\b([a-z]|\p{Join_C})\B){0,2}$",
            13,
            0,
            Some(2),
            true,
            2,
            true,
            true,
            false,
            false,
            Some(true),
            Some(false),
        ),
        (
            r"(\b(\u{61}|\u{1f600})\B)?",
            14,
            0,
            Some(1),
            true,
            2,
            false,
            false,
            false,
            false,
            Some(true),
            Some(false),
        ),
        (
            r"(\B^(^\B.$|\ba\b)\B){1,3}$",
            15,
            1,
            Some(3),
            true,
            2,
            false,
            true,
            true,
            false,
            Some(false),
            Some(false),
        ),
    ];
    fn plan(source: &str, v: bool, i: bool, m: bool, s: bool) -> RegExpUnicodeRepeatedAtomMatcher {
        RegExpUnicodeRepeatedAtomMatcher::compile_nested_alternative_assertions_with_work(
            &JsString::from(source),
            v,
            i,
            m,
            s,
            |_| Ok::<(), ()>(()),
        )
        .unwrap()
        .unwrap()
    }

    #[test]
    fn unicode_repeated_atom_shared_alternative_assertion_ranges_snapshot() {
        let mut rows = String::new();
        for (source, ..) in CASES {
            let pattern = JsString::from(source);
            for v in [false, true] {
                for i in [false, true] {
                    for m in [false, true] {
                        for s in [false, true] {
                            let matcher = plan(source, v, i, m, s);
                            for input in [
                                JsString::from("Aaaa\nAA"),
                                JsString::from("A\r\n\u{200c}\u{200d}\u{ffff}\u{10ffff}"),
                                JsString::from("😀??aaſK"),
                                JsString::from_code_units(vec![0x61, 0xd800, 0x61, 0xdc00]),
                                JsString::from(""),
                            ] {
                                for start in [0, 1, input.len(), input.len() + 1] {
                                    for sticky in [false, true] {
                                        let found = matcher
                                            .find_with_captures_and_work(
                                                &input,
                                                start,
                                                sticky,
                                                |_| Ok::<(), ()>(()),
                                            )
                                            .unwrap();
                                        writeln!(rows,"{pattern:?} v={v} i={i} m={m} s={s} input={input:?} start={start} sticky={sticky} count={} range={:?} capture={:?}",matcher.capture_count(),found.as_ref().map(|f|f.range.clone()),found.and_then(|f|f.atom_capture)).unwrap();
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        insta::assert_snapshot!(rows);
    }

    #[test]
    fn unicode_repeated_atom_shared_alternative_assertions_agree_with_independent_counts_and_final_character()
     {
        let alphabet: [u16; 10] = [
            0x41, 0x61, 10, 13, 0x17f, 0x212a, 0xd83d, 0xde00, 0xd800, 0xdc00,
        ];
        let lt = |c| [10, 13, 0x2028, 0x2029].contains(&c);
        for i in [false, true] {
            for m in [false, true] {
                for s in [false, true] {
                    let plans: Vec<_> = CASES.iter().map(|c| plan(c.0, true, i, m, s)).collect();
                    let fold = |c| {
                        if i {
                            match c {
                                0x41 => 0x61,
                                0x17f => 0x73,
                                0x212a => 0x6b,
                                _ => c,
                            }
                        } else {
                            c
                        }
                    };
                    for len in 0..=4 {
                        for ordinal in 0..alphabet.len().pow(len) {
                            let units: Vec<_> = (0..len)
                                .map(|j| alphabet[ordinal / alphabet.len().pow(j) % alphabet.len()])
                                .collect();
                            let mut points = Vec::new();
                            let mut bounds = vec![0];
                            let mut j = 0;
                            while j < units.len() {
                                let mut c = u32::from(units[j]);
                                j += 1;
                                if (0xd800..=0xdbff).contains(&c)
                                    && j < units.len()
                                    && (0xdc00..=0xdfff).contains(&units[j])
                                {
                                    c = 65536 + (c - 0xd800) * 1024 + u32::from(units[j]) - 0xdc00;
                                    j += 1;
                                }
                                points.push(c);
                                bounds.push(j);
                            }
                            let input = JsString::from_code_units(units);
                            for start in 0..=input.len() + 1 {
                                for sticky in [false, true] {
                                    for (case, matcher) in CASES.iter().zip(&plans) {
                                        let expected = if start > input.len() {
                                            None
                                        } else {
                                            let first = bounds.partition_point(|&b| b <= start) - 1;
                                            (first..=points.len()).find_map(|at| {
                                                let word=|c:Option<&u32>|c.is_some_and(|&c|matches!(c,0x30..=0x39|0x41..=0x5a|0x5f|0x61..=0x7a)||(i&&matches!(c,0x17f|0x212a)));
                                                if (sticky && at != first)
                                                    || (case.6
                                                        && at != 0
                                                        && !(m && lt(points[at - 1])))
                                                {
                                                    return None;
                                                }
                                                let available = (at..points.len()).take_while(|&p| {
                                                    let c = points[p];
                                                    let prev = p.checked_sub(1).and_then(|n| points.get(n));
                                                    let next = points.get(p+1);
                                                    let wb = word(prev) != word(Some(&c));
                                                    let we = word(Some(&c)) != word(next);
                                                    let begin = p == 0 || (m && prev.is_some_and(|&c| lt(c)));
                                                    let end = next.is_none() || (m && next.is_some_and(|&c| lt(c)));
                                                    let dot = s || !lt(c);
                                                    let class = [0x61, fold(0x17f), fold(0x212a)].contains(&fold(c));
                                                    let branch = match case.1 {
                                                        0 => [0x61,0x62].contains(&fold(c)),
                                                        1 => class,
                                                        2 => dot || c == 10,
                                                        3 => true,
                                                        4 => (0x200c..=0x200d).contains(&c) || matches!(c,0xfdd0..=0xfdef) || c&0xffff >= 0xfffe,
                                                        5 | 14 => c == 0x1f600 || fold(c) == 0x61,
                                                        6 => [0xd800,0xdc00].contains(&c),
                                                        7 => c == 0x7c || fold(c) == 0x61,
                                                        8 => (fold(c) == 0x61 && wb) || (fold(c) == 0x62 && we),
                                                        9 => (fold(c) == 0x61 && begin) || (fold(c) == 0x62 && end),
                                                        10 => [0x61,0x62,0x7c].contains(&fold(c)),
                                                        11 => (dot && !wb) || (class && wb),
                                                        12 => (fold(c) == 0x61 && end) || (fold(c) == 0x62 && begin),
                                                        13 => (0x61..=0x7a).contains(&fold(c)) || (0x200c..=0x200d).contains(&c),
                                                        15 => (dot && begin && end && !wb) || (fold(c) == 0x61 && wb && we),
                                                        _ => unreachable!(),
                                                    };
                                                    branch
                                                        && !case.10.is_some_and(|positive| wb != positive)
                                                        && !case.11.is_some_and(|positive| we != positive)
                                                        && (!case.8 || begin)
                                                        && (!case.9 || end)
                                                }).count();
                                                let upper =
                                                    available.min(case.3.unwrap_or(available));
                                                if upper < case.2 {
                                                    return None;
                                                }
                                                let endpoint = |n: usize| {
                                                    let end = at + n;
                                                    if case.7
                                                        && end != points.len()
                                                        && !(m && lt(points[end]))
                                                    {
                                                        return None;
                                                    }
                                                    Some(RegExpUnicodeRepeatedAtomMatch {
                                                        range: bounds[at]..bounds[end],
                                                        atom_capture: (case.5 != 0 && n != 0)
                                                            .then(|| bounds[end - 1]..bounds[end]),
                                                    })
                                                };
                                                let mut counts = case.2..=upper;
                                                if case.4 {
                                                    counts.rev().find_map(endpoint)
                                                } else {
                                                    counts.find_map(endpoint)
                                                }
                                            })
                                        };
                                        assert_eq!(matcher.capture_count(), case.5, "{}", case.0);
                                        assert_eq!(
                                            matcher
                                                .find_with_captures_and_work(
                                                    &input,
                                                    start,
                                                    sticky,
                                                    |_| Ok::<(), ()>(())
                                                )
                                                .unwrap(),
                                            expected,
                                            "{} i={i} m={m} s={s} {input:?} start={start} sticky={sticky}",
                                            case.0
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

    #[test]
    fn unicode_repeated_atom_shared_alternative_assertion_work_and_scope_contracts() {
        let source = format!(
            r"{}\B{}.|\n{}\B{}+",
            "(".repeat(1500),
            "(".repeat(1500),
            ")".repeat(1500),
            ")".repeat(1500)
        );
        let matcher = plan(&source, true, false, true, false).clone();
        let input = JsString::from("\n".repeat(200000).as_str());
        let mut work = 0;
        assert_eq!(matcher.capture_count(), 3000);
        assert_eq!(
            matcher
                .find_with_captures_and_work(&input, 0, false, |n| {
                    work += n;
                    Ok::<(), ()>(())
                })
                .unwrap(),
            Some(RegExpUnicodeRepeatedAtomMatch {
                range: 0..200000,
                atom_capture: Some(199999..200000)
            })
        );
        assert!(work < 70 * input.len(), "{work}");
        let matcher = plan(r"(\b(a|b)\B){2}", true, false, false, false);
        let input = JsString::from("a".repeat(200000).as_str());
        work = 0;
        assert_eq!(
            matcher
                .find_with_work(&input, 0, false, |n| {
                    work += n;
                    Ok::<(), ()>(())
                })
                .unwrap(),
            None
        );
        assert!(work < 70 * input.len(), "{work}");
        let source = JsString::from(r"(\B^(^\B.$|\ba\b)\B){1,3}$");
        let mut calls = 0;
        RegExpUnicodeRepeatedAtomMatcher::compile_nested_alternative_assertions_with_work(
            &source,
            true,
            false,
            true,
            true,
            |_| {
                calls += 1;
                Ok::<(), ()>(())
            },
        )
        .unwrap()
        .unwrap();
        for fail in 1..=calls {
            let mut n = 0;
            assert!(
                RegExpUnicodeRepeatedAtomMatcher::compile_nested_alternative_assertions_with_work(
                    &source,
                    true,
                    false,
                    true,
                    true,
                    |_| {
                        n += 1;
                        if n == fail { Err(()) } else { Ok(()) }
                    }
                )
                .is_err()
            );
        }
        let matcher = plan(r"(\B^(^\B.$|\ba\b)\B){1,3}$", true, false, true, true);
        let input = JsString::from("\r\n");
        calls = 0;
        matcher
            .find_with_captures_and_work(&input, 0, false, |_| {
                calls += 1;
                Ok::<(), ()>(())
            })
            .unwrap();
        for fail in 1..=calls {
            let mut n = 0;
            assert!(
                matcher
                    .find_with_captures_and_work(&input, 0, false, |_| {
                        n += 1;
                        if n == fail { Err(()) } else { Ok(()) }
                    })
                    .is_err()
            );
        }
        for source in [
            r"(\ba|b)+",
            r"(a|b\b)+",
            r"(^a|b$)+",
            r"(\b(a|)\B)+",
            r"(\b(a|ab)\B)+",
            r"(\b(a|b?)\B)+",
            r"(\b(a|(b))\B)+",
            r"(\b((a)|(b))\B)+",
            r"\b(\b(a|b)\B)+",
            r"(\b(a|b)\B)+b",
            r"((\b)(a|b)\B)+",
        ] {
            assert!(
                RegExpUnicodeRepeatedAtomMatcher::compile_nested_alternative_assertions_with_work(
                    &JsString::from(source),
                    true,
                    false,
                    true,
                    true,
                    |_| Ok::<(), ()>(())
                )
                .unwrap()
                .is_none(),
                "{source}"
            );
        }
        for source in [r"(\b(a|b))+", r"((a|b)\b)+", r"(\B^(^a|b$)$\B)+"] {
            assert!(
                RegExpUnicodeRepeatedAtomMatcher::compile_nested_assertion_groups_with_work(
                    &JsString::from(source),
                    true,
                    false,
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
}

#[cfg(test)]
mod scoped_alternative_word_tests {
    use super::*;
    use std::fmt::Write;

    type Case = (
        &'static str,
        u8,
        usize,
        Option<usize>,
        bool,
        usize,
        bool,
        bool,
        bool,
        bool,
        Option<bool>,
        Option<bool>,
        Option<bool>,
        Option<bool>,
    );
    const CASES: [Case; 16] = [
        (
            r"\b(\b(a|b)\B)+\B",
            0,
            1,
            None,
            true,
            2,
            false,
            false,
            false,
            false,
            Some(true),
            Some(false),
            Some(true),
            Some(false),
        ),
        (
            r"\B(\B((a|[ſK]))\b){0,3}?\b",
            1,
            0,
            Some(3),
            false,
            3,
            false,
            false,
            false,
            false,
            Some(false),
            Some(true),
            Some(false),
            Some(true),
        ),
        (
            r"\b(\B(?:.|\n)\B){2,4}\b",
            2,
            2,
            Some(4),
            true,
            1,
            false,
            false,
            false,
            false,
            Some(false),
            Some(false),
            Some(true),
            Some(true),
        ),
        (
            r"\B(\b(\d|\D)\b)*",
            3,
            0,
            None,
            true,
            2,
            false,
            false,
            false,
            false,
            Some(true),
            Some(true),
            Some(false),
            None,
        ),
        (
            r"^(\B(\p{Join_C}|\p{NChar})\B)+\B$",
            4,
            1,
            None,
            true,
            2,
            true,
            true,
            false,
            false,
            Some(false),
            Some(false),
            None,
            Some(false),
        ),
        (
            r"\b(\B(😀|a)\B){1,3}\b$",
            5,
            1,
            Some(3),
            true,
            2,
            false,
            true,
            false,
            false,
            Some(false),
            Some(false),
            Some(true),
            Some(true),
        ),
        (
            r"(\B(\uD800|\uDC00)\B){0,2}?\b",
            6,
            0,
            Some(2),
            false,
            2,
            false,
            false,
            false,
            false,
            Some(false),
            Some(false),
            None,
            Some(true),
        ),
        (
            r"\b(\b(\||a)\B)+",
            7,
            1,
            None,
            true,
            2,
            false,
            false,
            false,
            false,
            Some(true),
            Some(false),
            Some(true),
            None,
        ),
        (
            r"\b(\b(\ba|b\b)\B)+\B",
            8,
            1,
            None,
            true,
            2,
            false,
            false,
            false,
            false,
            Some(true),
            Some(false),
            Some(true),
            Some(false),
        ),
        (
            r"(\b^(^a|b$)$\b){1,2}\b$",
            9,
            1,
            Some(2),
            true,
            2,
            false,
            true,
            true,
            true,
            Some(true),
            Some(true),
            None,
            Some(true),
        ),
        (
            r"\B(\B([a\|]|b)\B)??\B",
            10,
            0,
            Some(1),
            false,
            2,
            false,
            false,
            false,
            false,
            Some(false),
            Some(false),
            Some(false),
            Some(false),
        ),
        (
            r"\B(\B(\B.|\b[aſK])\b){1,3}\b",
            11,
            1,
            Some(3),
            true,
            2,
            false,
            false,
            false,
            false,
            Some(false),
            Some(true),
            Some(false),
            Some(true),
        ),
        (
            r"(\B(a$|^b)\b)+\b",
            12,
            1,
            None,
            true,
            2,
            false,
            false,
            false,
            false,
            Some(false),
            Some(true),
            None,
            Some(true),
        ),
        (
            r"^\b(\b([a-z]|\p{Join_C})\B){0,2}\B$",
            13,
            0,
            Some(2),
            true,
            2,
            true,
            true,
            false,
            false,
            Some(true),
            Some(false),
            Some(true),
            Some(false),
        ),
        (
            r"(\b(\u{61}|\u{1f600})\B)?\b",
            14,
            0,
            Some(1),
            true,
            2,
            false,
            false,
            false,
            false,
            Some(true),
            Some(false),
            None,
            Some(true),
        ),
        (
            r"\B(\B^(^\B.$|\ba\b)\B){1,3}\B$",
            15,
            1,
            Some(3),
            true,
            2,
            false,
            true,
            true,
            false,
            Some(false),
            Some(false),
            Some(false),
            Some(false),
        ),
    ];
    fn plan(source: &str, v: bool, i: bool, m: bool, s: bool) -> RegExpUnicodeRepeatedAtomMatcher {
        RegExpUnicodeRepeatedAtomMatcher::compile_scoped_alternative_words_with_work(
            &JsString::from(source),
            v,
            i,
            m,
            s,
            |_| Ok::<(), ()>(()),
        )
        .unwrap()
        .unwrap()
    }

    #[test]
    fn unicode_repeated_atom_scoped_alternative_word_ranges_snapshot() {
        let mut rows = String::new();
        for (source, ..) in CASES {
            let pattern = JsString::from(source);
            for v in [false, true] {
                for i in [false, true] {
                    for m in [false, true] {
                        for s in [false, true] {
                            let matcher = plan(source, v, i, m, s);
                            for input in [
                                JsString::from("Aaaa\nAA"),
                                JsString::from("A\r\n\u{200c}\u{200d}\u{ffff}\u{10ffff}"),
                                JsString::from("😀??aaſK"),
                                JsString::from_code_units(vec![0x61, 0xd800, 0x61, 0xdc00]),
                                JsString::from(""),
                            ] {
                                for start in [0, 1, input.len(), input.len() + 1] {
                                    for sticky in [false, true] {
                                        let found = matcher
                                            .find_with_captures_and_work(
                                                &input,
                                                start,
                                                sticky,
                                                |_| Ok::<(), ()>(()),
                                            )
                                            .unwrap();
                                        writeln!(rows,"{pattern:?} v={v} i={i} m={m} s={s} input={input:?} start={start} sticky={sticky} count={} range={:?} capture={:?}",matcher.capture_count(),found.as_ref().map(|f|f.range.clone()),found.and_then(|f|f.atom_capture)).unwrap();
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        insta::assert_snapshot!(rows);
    }

    #[test]
    fn unicode_repeated_atom_scoped_alternative_words_agree_with_independent_counts_and_final_character()
     {
        let alphabet: [u16; 10] = [
            0x41, 0x61, 10, 13, 0x17f, 0x212a, 0xd83d, 0xde00, 0xd800, 0xdc00,
        ];
        let lt = |c| [10, 13, 0x2028, 0x2029].contains(&c);
        for i in [false, true] {
            for m in [false, true] {
                for s in [false, true] {
                    let plans: Vec<_> = CASES.iter().map(|c| plan(c.0, true, i, m, s)).collect();
                    let fold = |c| {
                        if i {
                            match c {
                                0x41 => 0x61,
                                0x17f => 0x73,
                                0x212a => 0x6b,
                                _ => c,
                            }
                        } else {
                            c
                        }
                    };
                    for len in 0..=4 {
                        for ordinal in 0..alphabet.len().pow(len) {
                            let units: Vec<_> = (0..len)
                                .map(|j| alphabet[ordinal / alphabet.len().pow(j) % alphabet.len()])
                                .collect();
                            let mut points = Vec::new();
                            let mut bounds = vec![0];
                            let mut j = 0;
                            while j < units.len() {
                                let mut c = u32::from(units[j]);
                                j += 1;
                                if (0xd800..=0xdbff).contains(&c)
                                    && j < units.len()
                                    && (0xdc00..=0xdfff).contains(&units[j])
                                {
                                    c = 65536 + (c - 0xd800) * 1024 + u32::from(units[j]) - 0xdc00;
                                    j += 1;
                                }
                                points.push(c);
                                bounds.push(j);
                            }
                            let input = JsString::from_code_units(units);
                            for start in 0..=input.len() + 1 {
                                for sticky in [false, true] {
                                    for (case, matcher) in CASES.iter().zip(&plans) {
                                        let expected = if start > input.len() {
                                            None
                                        } else {
                                            let first = bounds.partition_point(|&b| b <= start) - 1;
                                            (first..=points.len()).find_map(|at| {
                                                let word=|c:Option<&u32>|c.is_some_and(|&c|matches!(c,0x30..=0x39|0x41..=0x5a|0x5f|0x61..=0x7a)||(i&&matches!(c,0x17f|0x212a)));
                                                if (sticky && at != first)
                                                    || (case.6
                                                        && at != 0
                                                        && !(m && lt(points[at - 1])))
                                                {
                                                    return None;
                                                }
                                                if case.12.is_some_and(|positive| (word(at.checked_sub(1).and_then(|n|points.get(n))) != word(points.get(at))) != positive) { return None; }
                                                let available = (at..points.len()).take_while(|&p| {
                                                    let c = points[p];
                                                    let prev = p.checked_sub(1).and_then(|n| points.get(n));
                                                    let next = points.get(p+1);
                                                    let wb = word(prev) != word(Some(&c));
                                                    let we = word(Some(&c)) != word(next);
                                                    let begin = p == 0 || (m && prev.is_some_and(|&c| lt(c)));
                                                    let end = next.is_none() || (m && next.is_some_and(|&c| lt(c)));
                                                    let dot = s || !lt(c);
                                                    let class = [0x61, fold(0x17f), fold(0x212a)].contains(&fold(c));
                                                    let branch = match case.1 {
                                                        0 => [0x61,0x62].contains(&fold(c)),
                                                        1 => class,
                                                        2 => dot || c == 10,
                                                        3 => true,
                                                        4 => (0x200c..=0x200d).contains(&c) || matches!(c,0xfdd0..=0xfdef) || c&0xffff >= 0xfffe,
                                                        5 | 14 => c == 0x1f600 || fold(c) == 0x61,
                                                        6 => [0xd800,0xdc00].contains(&c),
                                                        7 => c == 0x7c || fold(c) == 0x61,
                                                        8 => (fold(c) == 0x61 && wb) || (fold(c) == 0x62 && we),
                                                        9 => (fold(c) == 0x61 && begin) || (fold(c) == 0x62 && end),
                                                        10 => [0x61,0x62,0x7c].contains(&fold(c)),
                                                        11 => (dot && !wb) || (class && wb),
                                                        12 => (fold(c) == 0x61 && end) || (fold(c) == 0x62 && begin),
                                                        13 => (0x61..=0x7a).contains(&fold(c)) || (0x200c..=0x200d).contains(&c),
                                                        15 => (dot && begin && end && !wb) || (fold(c) == 0x61 && wb && we),
                                                        _ => unreachable!(),
                                                    };
                                                    branch
                                                        && !case.10.is_some_and(|positive| wb != positive)
                                                        && !case.11.is_some_and(|positive| we != positive)
                                                        && (!case.8 || begin)
                                                        && (!case.9 || end)
                                                }).count();
                                                let upper =
                                                    available.min(case.3.unwrap_or(available));
                                                if upper < case.2 {
                                                    return None;
                                                }
                                                let endpoint = |n: usize| {
                                                    let end = at + n;
                                                    if case.13.is_some_and(|positive| (word(end.checked_sub(1).and_then(|n|points.get(n))) != word(points.get(end))) != positive) { return None; }

                                                    if case.7
                                                        && end != points.len()
                                                        && !(m && lt(points[end]))
                                                    {
                                                        return None;
                                                    }
                                                    Some(RegExpUnicodeRepeatedAtomMatch {
                                                        range: bounds[at]..bounds[end],
                                                        atom_capture: (case.5 != 0 && n != 0)
                                                            .then(|| bounds[end - 1]..bounds[end]),
                                                    })
                                                };
                                                let mut counts = case.2..=upper;
                                                if case.4 {
                                                    counts.rev().find_map(endpoint)
                                                } else {
                                                    counts.find_map(endpoint)
                                                }
                                            })
                                        };
                                        assert_eq!(matcher.capture_count(), case.5, "{}", case.0);
                                        assert_eq!(
                                            matcher
                                                .find_with_captures_and_work(
                                                    &input,
                                                    start,
                                                    sticky,
                                                    |_| Ok::<(), ()>(())
                                                )
                                                .unwrap(),
                                            expected,
                                            "{} i={i} m={m} s={s} {input:?} start={start} sticky={sticky}",
                                            case.0
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

    #[test]
    fn unicode_repeated_atom_scoped_alternative_word_work_and_contracts() {
        let source = format!(
            r"\B{}\B{}.|\n{}\B{}+\B$",
            "(".repeat(1500),
            "(".repeat(1500),
            ")".repeat(1500),
            ")".repeat(1500)
        );
        let matcher = plan(&source, true, false, true, false).clone();
        let input = JsString::from("\n".repeat(200000).as_str());
        let mut work = 0;
        assert_eq!(matcher.capture_count(), 3000);
        assert_eq!(
            matcher
                .find_with_captures_and_work(&input, 0, false, |n| {
                    work += n;
                    Ok::<(), ()>(())
                })
                .unwrap(),
            Some(RegExpUnicodeRepeatedAtomMatch {
                range: 0..200000,
                atom_capture: Some(199999..200000)
            })
        );
        assert!(work < 80 * input.len(), "{work}");
        let matcher = plan(r"\b(\b(a|b)\B){2}\B", true, false, false, false);
        let input = JsString::from("a".repeat(200000).as_str());
        work = 0;
        assert_eq!(
            matcher
                .find_with_work(&input, 0, false, |n| {
                    work += n;
                    Ok::<(), ()>(())
                })
                .unwrap(),
            None
        );
        assert!(work < 80 * input.len(), "{work}");
        let source = JsString::from(r"\B(\B^(^\B.$|\ba\b)\B){1,3}\B$");
        let mut calls = 0;
        RegExpUnicodeRepeatedAtomMatcher::compile_scoped_alternative_words_with_work(
            &source,
            true,
            false,
            true,
            true,
            |_| {
                calls += 1;
                Ok::<(), ()>(())
            },
        )
        .unwrap()
        .unwrap();
        for fail in 1..=calls {
            let mut n = 0;
            assert!(
                RegExpUnicodeRepeatedAtomMatcher::compile_scoped_alternative_words_with_work(
                    &source,
                    true,
                    false,
                    true,
                    true,
                    |_| {
                        n += 1;
                        if n == fail { Err(()) } else { Ok(()) }
                    }
                )
                .is_err()
            );
        }
        let matcher = plan(r"\B(\B^(^\B.$|\ba\b)\B){1,3}\B$", true, false, true, true);
        let input = JsString::from("\r\n");
        calls = 0;
        matcher
            .find_with_captures_and_work(&input, 0, false, |_| {
                calls += 1;
                Ok::<(), ()>(())
            })
            .unwrap();
        for fail in 1..=calls {
            let mut n = 0;
            assert!(
                matcher
                    .find_with_captures_and_work(&input, 0, false, |_| {
                        n += 1;
                        if n == fail { Err(()) } else { Ok(()) }
                    })
                    .is_err()
            );
        }
        for source in [
            r"\b(\ba|b)+",
            r"(a|b\b)+\b",
            r"\b(^a|b$)+",
            r"\b(\b(a|)\B)+",
            r"\b(\b(a|ab)\B)+",
            r"\b(\b(a|b?)\B)+",
            r"\b(\b(a|(b))\B)+",
            r"\b(\b((a)|(b))\B)+",
            r"\b(\b(a|b)\B)+b",
            r"\b((\b)(a|b)\B)+",
        ] {
            assert!(
                RegExpUnicodeRepeatedAtomMatcher::compile_scoped_alternative_words_with_work(
                    &JsString::from(source),
                    true,
                    false,
                    true,
                    true,
                    |_| Ok::<(), ()>(())
                )
                .unwrap()
                .is_none(),
                "{source}"
            );
        }
        for source in [r"\b(\b(a|b))+", r"((a|b)\b)+\b", r"\B(\B^(^a|b$)$\B)+\B"] {
            assert!(
                RegExpUnicodeRepeatedAtomMatcher::compile_nested_alternative_assertions_with_work(
                    &JsString::from(source),
                    true,
                    false,
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
}

#[cfg(test)]
mod noncapturing_branch_tests {
    use super::*;
    use std::fmt::Write;

    type Case = (
        &'static str,
        u8,
        usize,
        Option<usize>,
        bool,
        usize,
        bool,
        bool,
    );
    const CASES: [Case; 16] = [
        (r"((?:a)|(?:b))+", 0, 1, None, true, 1, false, false),
        (
            r"(((?:a)|(?:[ſK]))){0,3}?",
            1,
            0,
            Some(3),
            false,
            2,
            false,
            false,
        ),
        (
            r"(?:(?:.)|(?:\n)){2,4}",
            2,
            2,
            Some(4),
            true,
            0,
            false,
            false,
        ),
        (r"((?:\d)|(?:\D))*", 3, 0, None, true, 1, false, false),
        (
            r"^((?:\p{Join_C})|(?:\p{NChar}))+$",
            4,
            1,
            None,
            true,
            1,
            true,
            true,
        ),
        (r"((?:😀)|(?:a)){1,3}$", 5, 1, Some(3), true, 1, false, true),
        (
            r"((?:\uD800)|(?:\uDC00)){0,2}?",
            6,
            0,
            Some(2),
            false,
            1,
            false,
            false,
        ),
        (r"((?:\|)|(?:a))+", 7, 1, None, true, 1, false, false),
        (r"((?:\ba)|(?:b\b))+", 8, 1, None, true, 1, false, false),
        (
            r"((?:^a)|(?:b$)){1,2}$",
            9,
            1,
            Some(2),
            true,
            1,
            false,
            true,
        ),
        (
            r"((?:[a\|])|(?:b))??",
            10,
            0,
            Some(1),
            false,
            1,
            false,
            false,
        ),
        (
            r"((?:\B.)|(?:\b[aſK])){1,3}",
            11,
            1,
            Some(3),
            true,
            1,
            false,
            false,
        ),
        (r"((?:a$)|(?:^b))+", 12, 1, None, true, 1, false, false),
        (
            r"^((?:[a-z])|(?:\p{Join_C})){0,2}$",
            13,
            0,
            Some(2),
            true,
            1,
            true,
            true,
        ),
        (
            r"((?:\u{61})|(?:\u{1f600}))?",
            14,
            0,
            Some(1),
            true,
            1,
            false,
            false,
        ),
        (
            r"((?:^\B.$)|(?:\ba\b)){1,3}$",
            15,
            1,
            Some(3),
            true,
            1,
            false,
            true,
        ),
    ];
    fn plan(source: &str, v: bool, i: bool, m: bool, s: bool) -> RegExpUnicodeRepeatedAtomMatcher {
        RegExpUnicodeRepeatedAtomMatcher::compile_noncapturing_branches_with_work(
            &JsString::from(source),
            v,
            i,
            m,
            s,
            |_| Ok::<(), ()>(()),
        )
        .unwrap()
        .unwrap()
    }

    #[test]
    fn unicode_repeated_atom_noncapturing_branch_ranges_snapshot() {
        let mut rows = String::new();
        for (source, ..) in CASES {
            let pattern = JsString::from(source);
            for v in [false, true] {
                for i in [false, true] {
                    for m in [false, true] {
                        for s in [false, true] {
                            let matcher = plan(source, v, i, m, s);
                            for input in [
                                JsString::from("Aaaa\nAA"),
                                JsString::from("A\r\n\u{200c}\u{200d}\u{ffff}\u{10ffff}"),
                                JsString::from("😀??aaſK"),
                                JsString::from_code_units(vec![0x61, 0xd800, 0x61, 0xdc00]),
                                JsString::from(""),
                            ] {
                                for start in [0, 1, input.len(), input.len() + 1] {
                                    for sticky in [false, true] {
                                        let found = matcher
                                            .find_with_captures_and_work(
                                                &input,
                                                start,
                                                sticky,
                                                |_| Ok::<(), ()>(()),
                                            )
                                            .unwrap();
                                        writeln!(rows,"{pattern:?} v={v} i={i} m={m} s={s} input={input:?} start={start} sticky={sticky} count={} range={:?} capture={:?}",matcher.capture_count(),found.as_ref().map(|f|f.range.clone()),found.and_then(|f|f.atom_capture)).unwrap();
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        insta::assert_snapshot!(rows);
    }

    #[test]
    fn unicode_repeated_atom_alternatives_agree_with_independent_counts_and_final_character() {
        let alphabet: [u16; 10] = [
            0x41, 0x61, 10, 13, 0x17f, 0x212a, 0xd83d, 0xde00, 0xd800, 0xdc00,
        ];
        let lt = |c| [10, 13, 0x2028, 0x2029].contains(&c);
        for i in [false, true] {
            for m in [false, true] {
                for s in [false, true] {
                    let plans: Vec<_> = CASES.iter().map(|c| plan(c.0, true, i, m, s)).collect();
                    let fold = |c| {
                        if i {
                            match c {
                                0x41 => 0x61,
                                0x17f => 0x73,
                                0x212a => 0x6b,
                                _ => c,
                            }
                        } else {
                            c
                        }
                    };
                    for len in 0..=4 {
                        for ordinal in 0..alphabet.len().pow(len) {
                            let units: Vec<_> = (0..len)
                                .map(|j| alphabet[ordinal / alphabet.len().pow(j) % alphabet.len()])
                                .collect();
                            let mut points = Vec::new();
                            let mut bounds = vec![0];
                            let mut j = 0;
                            while j < units.len() {
                                let mut c = u32::from(units[j]);
                                j += 1;
                                if (0xd800..=0xdbff).contains(&c)
                                    && j < units.len()
                                    && (0xdc00..=0xdfff).contains(&units[j])
                                {
                                    c = 65536 + (c - 0xd800) * 1024 + u32::from(units[j]) - 0xdc00;
                                    j += 1;
                                }
                                points.push(c);
                                bounds.push(j);
                            }
                            let input = JsString::from_code_units(units);
                            for start in 0..=input.len() + 1 {
                                for sticky in [false, true] {
                                    for (case, matcher) in CASES.iter().zip(&plans) {
                                        let expected = if start > input.len() {
                                            None
                                        } else {
                                            let first = bounds.partition_point(|&b| b <= start) - 1;
                                            (first..=points.len()).find_map(|at| {
                                                let word=|c:Option<&u32>|c.is_some_and(|&c|matches!(c,0x30..=0x39|0x41..=0x5a|0x5f|0x61..=0x7a)||(i&&matches!(c,0x17f|0x212a)));
                                                if (sticky && at != first)
                                                    || (case.6
                                                        && at != 0
                                                        && !(m && lt(points[at - 1])))
                                                {
                                                    return None;
                                                }
                                                let available = (at..points.len()).take_while(|&p| {
                                                    let c = points[p];
                                                    let prev = p.checked_sub(1).and_then(|n| points.get(n));
                                                    let next = points.get(p+1);
                                                    let wb = word(prev) != word(Some(&c));
                                                    let we = word(Some(&c)) != word(next);
                                                    let begin = p == 0 || (m && prev.is_some_and(|&c| lt(c)));
                                                    let end = next.is_none() || (m && next.is_some_and(|&c| lt(c)));
                                                    let dot = s || !lt(c);
                                                    let class = [0x61, fold(0x17f), fold(0x212a)].contains(&fold(c));
                                                    match case.1 {
                                                        0 => [0x61,0x62].contains(&fold(c)),
                                                        1 => class,
                                                        2 => dot || c == 10,
                                                        3 => true,
                                                        4 => (0x200c..=0x200d).contains(&c) || matches!(c,0xfdd0..=0xfdef) || c&0xffff >= 0xfffe,
                                                        5 | 14 => c == 0x1f600 || fold(c) == 0x61,
                                                        6 => [0xd800,0xdc00].contains(&c),
                                                        7 => c == 0x7c || fold(c) == 0x61,
                                                        8 => (fold(c) == 0x61 && wb) || (fold(c) == 0x62 && we),
                                                        9 => (fold(c) == 0x61 && begin) || (fold(c) == 0x62 && end),
                                                        10 => [0x61,0x62,0x7c].contains(&fold(c)),
                                                        11 => (dot && !wb) || (class && wb),
                                                        12 => (fold(c) == 0x61 && end) || (fold(c) == 0x62 && begin),
                                                        13 => (0x61..=0x7a).contains(&fold(c)) || (0x200c..=0x200d).contains(&c),
                                                        15 => (dot && begin && end && !wb) || (fold(c) == 0x61 && wb && we),
                                                        _ => unreachable!(),
                                                    }
                                                }).count();
                                                let upper =
                                                    available.min(case.3.unwrap_or(available));
                                                if upper < case.2 {
                                                    return None;
                                                }
                                                let endpoint = |n: usize| {
                                                    let end = at + n;
                                                    if case.7
                                                        && end != points.len()
                                                        && !(m && lt(points[end]))
                                                    {
                                                        return None;
                                                    }
                                                    Some(RegExpUnicodeRepeatedAtomMatch {
                                                        range: bounds[at]..bounds[end],
                                                        atom_capture: (case.5 != 0 && n != 0)
                                                            .then(|| bounds[end - 1]..bounds[end]),
                                                    })
                                                };
                                                let mut counts = case.2..=upper;
                                                if case.4 {
                                                    counts.rev().find_map(endpoint)
                                                } else {
                                                    counts.find_map(endpoint)
                                                }
                                            })
                                        };
                                        assert_eq!(matcher.capture_count(), case.5, "{}", case.0);
                                        assert_eq!(
                                            matcher
                                                .find_with_captures_and_work(
                                                    &input,
                                                    start,
                                                    sticky,
                                                    |_| Ok::<(), ()>(())
                                                )
                                                .unwrap(),
                                            expected,
                                            "{} i={i} m={m} s={s} {input:?} start={start} sticky={sticky}",
                                            case.0
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

    #[test]
    fn unicode_repeated_atom_noncapturing_branch_work_clones_and_contracts() {
        let source = format!("{}.|\\n{}+", "(".repeat(3000), ")".repeat(3000));
        let matcher = plan(&source, true, false, true, false).clone();
        let input = JsString::from("\n".repeat(200000).as_str());
        let mut work = 0;
        assert_eq!(matcher.capture_count(), 3000);
        assert_eq!(
            matcher
                .find_with_captures_and_work(&input, 0, false, |n| {
                    work += n;
                    Ok::<(), ()>(())
                })
                .unwrap(),
            Some(RegExpUnicodeRepeatedAtomMatch {
                range: 0..200000,
                atom_capture: Some(199999..200000)
            })
        );
        assert!(work < 50 * input.len(), "{work}");
        let many = format!("({}|b)+", "a|".repeat(20000).trim_end_matches('|'));
        assert_eq!(
            plan(&many, false, false, false, false)
                .find_with_work(&JsString::from("b"), 0, true, |_| Ok::<(), ()>(()))
                .unwrap(),
            Some(0..1)
        );
        let source = JsString::from(r"((?:^\B.$)|(?:\ba\b)){1,3}$");
        let mut calls = 0;
        RegExpUnicodeRepeatedAtomMatcher::compile_noncapturing_branches_with_work(
            &source,
            true,
            false,
            true,
            true,
            |_| {
                calls += 1;
                Ok::<(), ()>(())
            },
        )
        .unwrap()
        .unwrap();
        for fail in 1..=calls {
            let mut n = 0;
            assert!(
                RegExpUnicodeRepeatedAtomMatcher::compile_noncapturing_branches_with_work(
                    &source,
                    true,
                    false,
                    true,
                    true,
                    |_| {
                        n += 1;
                        if n == fail { Err(()) } else { Ok(()) }
                    }
                )
                .is_err()
            );
        }
        let matcher = plan(r"((?:^\B.$)|(?:\ba\b)){1,3}$", true, false, true, true);
        let input = JsString::from("\r\n");
        calls = 0;
        matcher
            .find_with_captures_and_work(&input, 0, false, |_| {
                calls += 1;
                Ok::<(), ()>(())
            })
            .unwrap();
        for fail in 1..=calls {
            let mut n = 0;
            assert!(
                matcher
                    .find_with_captures_and_work(&input, 0, false, |_| {
                        n += 1;
                        if n == fail { Err(()) } else { Ok(()) }
                    })
                    .is_err()
            );
        }

        let deep = format!("({}a{}|b)+", "(?:".repeat(6000), ")".repeat(6000));
        assert_eq!(plan(&deep, true, false, false, false).capture_count(), 1);
        assert_eq!(
            plan(&deep, true, false, false, false)
                .find_with_work(&JsString::from("aa"), 0, false, |_| Ok::<(), ()>(()))
                .unwrap(),
            Some(0..2)
        );
        for source in [
            r"((?:a)|)+",
            r"(|(?:b))+",
            r"((?:a)|(?:ab))+",
            r"((?:a)|(?:b?))+",
            r"((?:a)|(b))+",
            r"((?:a)|(?:\b))+",
            r"((?:a)|(?i:b))+",
            r"((?:a)|(?:b))+c",
            r"\b((?:a)|(?:b))+",
            r"((?:a)|(?:b))+\b",
            r"(\b((?:a)|(?:b)))+",
            r"((a)|(?:b))+",
            r"((?:a)|(?:b)){2}c",
            r"((?:a|b)|(?:c))+",
            r"((?:a){1}|(?:b))+",
            r"((?:a{1})|(?:b))+",
            r"(^(?:a)|(?:b))+",
            r"((?:a)$|(?:b))+",
            r"(\b(?:a)|(?:b))+",
        ] {
            assert!(
                RegExpUnicodeRepeatedAtomMatcher::compile_noncapturing_branches_with_work(
                    &JsString::from(source),
                    true,
                    false,
                    true,
                    true,
                    |_| Ok::<(), ()>(())
                )
                .unwrap()
                .is_none(),
                "{source}"
            );
        }
        for source in [
            r"((?:a)|(?:b))+",
            r"((?:^a)|(?:b$))+",
            r"((?:\ba)|(?:b\b))+",
        ] {
            assert!(
                RegExpUnicodeRepeatedAtomMatcher::compile_capturing_alternatives_with_work(
                    &JsString::from(source),
                    true,
                    false,
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
}

#[cfg(test)]
mod noncapturing_branch_word_tests {
    use super::*;
    use std::fmt::Write;

    type Case = (
        &'static str,
        u8,
        usize,
        Option<usize>,
        bool,
        usize,
        bool,
        bool,
        Option<bool>,
        Option<bool>,
    );
    const CASES: [Case; 16] = [
        (
            r"\b((?:a)|(?:b))+\B",
            0,
            1,
            None,
            true,
            1,
            false,
            false,
            Some(true),
            Some(false),
        ),
        (
            r"\B(((?:a)|(?:[ſK]))){0,3}?\b",
            1,
            0,
            Some(3),
            false,
            2,
            false,
            false,
            Some(false),
            Some(true),
        ),
        (
            r"\b(?:(?:.)|(?:\n)){2,4}\b",
            2,
            2,
            Some(4),
            true,
            0,
            false,
            false,
            Some(true),
            Some(true),
        ),
        (
            r"\B((?:\d)|(?:\D))*",
            3,
            0,
            None,
            true,
            1,
            false,
            false,
            Some(false),
            None,
        ),
        (
            r"^((?:\p{Join_C})|(?:\p{NChar}))+\B$",
            4,
            1,
            None,
            true,
            1,
            true,
            true,
            None,
            Some(false),
        ),
        (
            r"\b((?:😀)|(?:a)){1,3}\b$",
            5,
            1,
            Some(3),
            true,
            1,
            false,
            true,
            Some(true),
            Some(true),
        ),
        (
            r"((?:\uD800)|(?:\uDC00)){0,2}?\b",
            6,
            0,
            Some(2),
            false,
            1,
            false,
            false,
            None,
            Some(true),
        ),
        (
            r"\b((?:\|)|(?:a))+",
            7,
            1,
            None,
            true,
            1,
            false,
            false,
            Some(true),
            None,
        ),
        (
            r"\b((?:\ba)|(?:b\b))+\B",
            8,
            1,
            None,
            true,
            1,
            false,
            false,
            Some(true),
            Some(false),
        ),
        (
            r"((?:^a)|(?:b$)){1,2}\b$",
            9,
            1,
            Some(2),
            true,
            1,
            false,
            true,
            None,
            Some(true),
        ),
        (
            r"\B((?:[a\|])|(?:b))??\B",
            10,
            0,
            Some(1),
            false,
            1,
            false,
            false,
            Some(false),
            Some(false),
        ),
        (
            r"\B((?:\B.)|(?:\b[aſK])){1,3}\b",
            11,
            1,
            Some(3),
            true,
            1,
            false,
            false,
            Some(false),
            Some(true),
        ),
        (
            r"((?:a$)|(?:^b))+\b",
            12,
            1,
            None,
            true,
            1,
            false,
            false,
            None,
            Some(true),
        ),
        (
            r"^\b((?:[a-z])|(?:\p{Join_C})){0,2}\B$",
            13,
            0,
            Some(2),
            true,
            1,
            true,
            true,
            Some(true),
            Some(false),
        ),
        (
            r"((?:\u{61})|(?:\u{1f600}))?\b",
            14,
            0,
            Some(1),
            true,
            1,
            false,
            false,
            None,
            Some(true),
        ),
        (
            r"\B((?:^\B.$)|(?:\ba\b)){1,3}\B$",
            15,
            1,
            Some(3),
            true,
            1,
            false,
            true,
            Some(false),
            Some(false),
        ),
    ];
    fn plan(source: &str, v: bool, i: bool, m: bool, s: bool) -> RegExpUnicodeRepeatedAtomMatcher {
        RegExpUnicodeRepeatedAtomMatcher::compile_noncapturing_branch_words_with_work(
            &JsString::from(source),
            v,
            i,
            m,
            s,
            |_| Ok::<(), ()>(()),
        )
        .unwrap()
        .unwrap()
    }

    #[test]
    fn unicode_repeated_atom_noncapturing_branch_word_ranges_snapshot() {
        let mut rows = String::new();
        for (source, ..) in CASES {
            let pattern = JsString::from(source);
            for v in [false, true] {
                for i in [false, true] {
                    for m in [false, true] {
                        for s in [false, true] {
                            let matcher = plan(source, v, i, m, s);
                            for input in [
                                JsString::from("Aaaa\nAA"),
                                JsString::from("A\r\n\u{200c}\u{200d}\u{ffff}\u{10ffff}"),
                                JsString::from("😀??aaſK"),
                                JsString::from_code_units(vec![0x61, 0xd800, 0x61, 0xdc00]),
                                JsString::from(""),
                            ] {
                                for start in [0, 1, input.len(), input.len() + 1] {
                                    for sticky in [false, true] {
                                        let found = matcher
                                            .find_with_captures_and_work(
                                                &input,
                                                start,
                                                sticky,
                                                |_| Ok::<(), ()>(()),
                                            )
                                            .unwrap();
                                        writeln!(rows,"{pattern:?} v={v} i={i} m={m} s={s} input={input:?} start={start} sticky={sticky} count={} range={:?} capture={:?}",matcher.capture_count(),found.as_ref().map(|f|f.range.clone()),found.and_then(|f|f.atom_capture)).unwrap();
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        insta::assert_snapshot!(rows);
    }

    #[test]
    fn unicode_repeated_atom_word_alternatives_agree_with_independent_counts_and_final_character() {
        let alphabet: [u16; 10] = [
            0x41, 0x61, 10, 13, 0x17f, 0x212a, 0xd83d, 0xde00, 0xd800, 0xdc00,
        ];
        let lt = |c| [10, 13, 0x2028, 0x2029].contains(&c);
        for i in [false, true] {
            for m in [false, true] {
                for s in [false, true] {
                    let plans: Vec<_> = CASES.iter().map(|c| plan(c.0, true, i, m, s)).collect();
                    let fold = |c| {
                        if i {
                            match c {
                                0x41 => 0x61,
                                0x17f => 0x73,
                                0x212a => 0x6b,
                                _ => c,
                            }
                        } else {
                            c
                        }
                    };
                    for len in 0..=4 {
                        for ordinal in 0..alphabet.len().pow(len) {
                            let units: Vec<_> = (0..len)
                                .map(|j| alphabet[ordinal / alphabet.len().pow(j) % alphabet.len()])
                                .collect();
                            let mut points = Vec::new();
                            let mut bounds = vec![0];
                            let mut j = 0;
                            while j < units.len() {
                                let mut c = u32::from(units[j]);
                                j += 1;
                                if (0xd800..=0xdbff).contains(&c)
                                    && j < units.len()
                                    && (0xdc00..=0xdfff).contains(&units[j])
                                {
                                    c = 65536 + (c - 0xd800) * 1024 + u32::from(units[j]) - 0xdc00;
                                    j += 1;
                                }
                                points.push(c);
                                bounds.push(j);
                            }
                            let input = JsString::from_code_units(units);
                            for start in 0..=input.len() + 1 {
                                for sticky in [false, true] {
                                    for (case, matcher) in CASES.iter().zip(&plans) {
                                        let expected = if start > input.len() {
                                            None
                                        } else {
                                            let first = bounds.partition_point(|&b| b <= start) - 1;
                                            (first..=points.len()).find_map(|at| {
                                                let word=|c:Option<&u32>|c.is_some_and(|&c|matches!(c,0x30..=0x39|0x41..=0x5a|0x5f|0x61..=0x7a)||(i&&matches!(c,0x17f|0x212a)));
                                                if (sticky && at != first)
                                                    || (case.6
                                                        && at != 0
                                                        && !(m && lt(points[at - 1])))
                                                {
                                                    return None;
                                                }
                                                if case.8.is_some_and(|positive| (word(at.checked_sub(1).and_then(|n|points.get(n))) != word(points.get(at))) != positive) { return None; }
                                                let available = (at..points.len()).take_while(|&p| {
                                                    let c = points[p];
                                                    let prev = p.checked_sub(1).and_then(|n| points.get(n));
                                                    let next = points.get(p+1);
                                                    let wb = word(prev) != word(Some(&c));
                                                    let we = word(Some(&c)) != word(next);
                                                    let begin = p == 0 || (m && prev.is_some_and(|&c| lt(c)));
                                                    let end = next.is_none() || (m && next.is_some_and(|&c| lt(c)));
                                                    let dot = s || !lt(c);
                                                    let class = [0x61, fold(0x17f), fold(0x212a)].contains(&fold(c));
                                                    match case.1 {
                                                        0 => [0x61,0x62].contains(&fold(c)),
                                                        1 => class,
                                                        2 => dot || c == 10,
                                                        3 => true,
                                                        4 => (0x200c..=0x200d).contains(&c) || matches!(c,0xfdd0..=0xfdef) || c&0xffff >= 0xfffe,
                                                        5 | 14 => c == 0x1f600 || fold(c) == 0x61,
                                                        6 => [0xd800,0xdc00].contains(&c),
                                                        7 => c == 0x7c || fold(c) == 0x61,
                                                        8 => (fold(c) == 0x61 && wb) || (fold(c) == 0x62 && we),
                                                        9 => (fold(c) == 0x61 && begin) || (fold(c) == 0x62 && end),
                                                        10 => [0x61,0x62,0x7c].contains(&fold(c)),
                                                        11 => (dot && !wb) || (class && wb),
                                                        12 => (fold(c) == 0x61 && end) || (fold(c) == 0x62 && begin),
                                                        13 => (0x61..=0x7a).contains(&fold(c)) || (0x200c..=0x200d).contains(&c),
                                                        15 => (dot && begin && end && !wb) || (fold(c) == 0x61 && wb && we),
                                                        _ => unreachable!(),
                                                    }
                                                }).count();
                                                let upper =
                                                    available.min(case.3.unwrap_or(available));
                                                if upper < case.2 {
                                                    return None;
                                                }
                                                let endpoint = |n: usize| {
                                                    let end = at + n;
                                                    if case.9.is_some_and(|positive| (word(end.checked_sub(1).and_then(|n|points.get(n))) != word(points.get(end))) != positive) { return None; }

                                                    if case.7
                                                        && end != points.len()
                                                        && !(m && lt(points[end]))
                                                    {
                                                        return None;
                                                    }
                                                    Some(RegExpUnicodeRepeatedAtomMatch {
                                                        range: bounds[at]..bounds[end],
                                                        atom_capture: (case.5 != 0 && n != 0)
                                                            .then(|| bounds[end - 1]..bounds[end]),
                                                    })
                                                };
                                                let mut counts = case.2..=upper;
                                                if case.4 {
                                                    counts.rev().find_map(endpoint)
                                                } else {
                                                    counts.find_map(endpoint)
                                                }
                                            })
                                        };
                                        assert_eq!(matcher.capture_count(), case.5, "{}", case.0);
                                        assert_eq!(
                                            matcher
                                                .find_with_captures_and_work(
                                                    &input,
                                                    start,
                                                    sticky,
                                                    |_| Ok::<(), ()>(())
                                                )
                                                .unwrap(),
                                            expected,
                                            "{} i={i} m={m} s={s} {input:?} start={start} sticky={sticky}",
                                            case.0
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

    #[test]
    fn unicode_repeated_atom_noncapturing_branch_word_work_and_contracts() {
        let source = format!(
            r"\B{}(?:.)|(?:\n){}+\B$",
            "(".repeat(3000),
            ")".repeat(3000)
        );
        let matcher = plan(&source, true, false, true, false).clone();
        let input = JsString::from("\n".repeat(200000).as_str());
        let mut work = 0;
        assert_eq!(matcher.capture_count(), 3000);
        assert_eq!(
            matcher
                .find_with_captures_and_work(&input, 0, false, |n| {
                    work += n;
                    Ok::<(), ()>(())
                })
                .unwrap(),
            Some(RegExpUnicodeRepeatedAtomMatch {
                range: 0..200000,
                atom_capture: Some(199999..200000)
            })
        );
        assert!(work < 60 * input.len(), "{work}");
        let source = JsString::from(r"\B((?:^\B.$)|(?:\ba\b)){1,3}\B$");
        let mut calls = 0;
        RegExpUnicodeRepeatedAtomMatcher::compile_noncapturing_branch_words_with_work(
            &source,
            true,
            false,
            true,
            true,
            |_| {
                calls += 1;
                Ok::<(), ()>(())
            },
        )
        .unwrap()
        .unwrap();
        for fail in 1..=calls {
            let mut n = 0;
            assert!(
                RegExpUnicodeRepeatedAtomMatcher::compile_noncapturing_branch_words_with_work(
                    &source,
                    true,
                    false,
                    true,
                    true,
                    |_| {
                        n += 1;
                        if n == fail { Err(()) } else { Ok(()) }
                    }
                )
                .is_err()
            );
        }
        let input = JsString::from("\r\n");
        let matcher = plan(r"\B((?:^\B.$)|(?:\ba\b)){1,3}\B$", true, false, true, true);
        calls = 0;
        matcher
            .find_with_captures_and_work(&input, 0, false, |_| {
                calls += 1;
                Ok::<(), ()>(())
            })
            .unwrap();
        for fail in 1..=calls {
            let mut n = 0;
            assert!(
                matcher
                    .find_with_captures_and_work(&input, 0, false, |_| {
                        n += 1;
                        if n == fail { Err(()) } else { Ok(()) }
                    })
                    .is_err()
            );
        }
        for source in [
            r"\b(a|)+",
            r"\b(|b)+",
            r"\b(a|ab)+",
            r"\b((?:a)|(?:b)?)+",
            r"\b(a|(b))+",
            r"\B(a|\b)+",
            r"\b(a|(?i:b))+",
            r"\b((?:a)|(?:b))+c",
            r"\b(\b((?:a)|(?:b)))+",
            r"\b((a)|(b))+",
            r"\b((?:a)|(?:b)){2}c",
        ] {
            assert!(
                RegExpUnicodeRepeatedAtomMatcher::compile_noncapturing_branch_words_with_work(
                    &JsString::from(source),
                    true,
                    false,
                    true,
                    true,
                    |_| Ok::<(), ()>(())
                )
                .unwrap()
                .is_none(),
                "{source}"
            );
        }
        for source in [
            r"\b((?:a)|(?:b))+",
            r"((?:^a)|(?:b$))+\b",
            r"\b((?:\ba)|(?:b\b))+\B",
        ] {
            assert!(
                RegExpUnicodeRepeatedAtomMatcher::compile_noncapturing_branches_with_work(
                    &JsString::from(source),
                    true,
                    false,
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
}

#[cfg(test)]
mod noncapturing_branch_line_tests {
    use super::*;
    use std::fmt::Write;

    type Case = (
        &'static str,
        u8,
        usize,
        Option<usize>,
        bool,
        usize,
        bool,
        bool,
    );
    const CASES: [Case; 16] = [
        (r"((?:a)|(?:b))+", 0, 1, None, true, 1, false, false),
        (
            r"(((?:a)|(?:[ſK]))){0,3}?",
            1,
            0,
            Some(3),
            false,
            2,
            false,
            false,
        ),
        (
            r"(?:(?:.)|(?:\n)){2,4}",
            2,
            2,
            Some(4),
            true,
            0,
            false,
            false,
        ),
        (r"((?:\d)|(?:\D))*", 3, 0, None, true, 1, false, false),
        (
            r"^((?:\p{Join_C})|(?:\p{NChar}))+$",
            4,
            1,
            None,
            true,
            1,
            true,
            true,
        ),
        (r"((?:😀)|(?:a)){1,3}$", 5, 1, Some(3), true, 1, false, true),
        (
            r"((?:\uD800)|(?:\uDC00)){0,2}?",
            6,
            0,
            Some(2),
            false,
            1,
            false,
            false,
        ),
        (r"((?:\|)|(?:a))+", 7, 1, None, true, 1, false, false),
        (r"((?:\ba)|(?:b\b))+", 8, 1, None, true, 1, false, false),
        (
            r"(^(?:a)|(?:b)$){1,2}$",
            9,
            1,
            Some(2),
            true,
            1,
            false,
            true,
        ),
        (
            r"((?:[a\|])|(?:b))??",
            10,
            0,
            Some(1),
            false,
            1,
            false,
            false,
        ),
        (
            r"((?:\B.)|(?:\b[aſK])){1,3}",
            11,
            1,
            Some(3),
            true,
            1,
            false,
            false,
        ),
        (r"((?:a)$|^(?:b))+", 12, 1, None, true, 1, false, false),
        (
            r"^((?:[a-z])|(?:\p{Join_C})){0,2}$",
            13,
            0,
            Some(2),
            true,
            1,
            true,
            true,
        ),
        (
            r"((?:\u{61})|(?:\u{1f600}))?",
            14,
            0,
            Some(1),
            true,
            1,
            false,
            false,
        ),
        (
            r"(^(?:\B.)$|(?:\ba\b)){1,3}$",
            15,
            1,
            Some(3),
            true,
            1,
            false,
            true,
        ),
    ];
    fn plan(source: &str, v: bool, i: bool, m: bool, s: bool) -> RegExpUnicodeRepeatedAtomMatcher {
        RegExpUnicodeRepeatedAtomMatcher::compile_noncapturing_branch_lines_with_work(
            &JsString::from(source),
            v,
            i,
            m,
            s,
            |_| Ok::<(), ()>(()),
        )
        .unwrap()
        .unwrap_or_else(|| panic!("Unproved line-branch source: {source}"))
    }

    #[test]
    fn unicode_repeated_atom_noncapturing_branch_line_ranges_snapshot() {
        let mut rows = String::new();
        for (source, ..) in CASES {
            let pattern = JsString::from(source);
            for v in [false, true] {
                for i in [false, true] {
                    for m in [false, true] {
                        for s in [false, true] {
                            let matcher = plan(source, v, i, m, s);
                            for input in [
                                JsString::from("Aaaa\nAA"),
                                JsString::from("A\r\n\u{200c}\u{200d}\u{ffff}\u{10ffff}"),
                                JsString::from("😀??aaſK"),
                                JsString::from_code_units(vec![0x61, 0xd800, 0x61, 0xdc00]),
                                JsString::from(""),
                            ] {
                                for start in [0, 1, input.len(), input.len() + 1] {
                                    for sticky in [false, true] {
                                        let found = matcher
                                            .find_with_captures_and_work(
                                                &input,
                                                start,
                                                sticky,
                                                |_| Ok::<(), ()>(()),
                                            )
                                            .unwrap();
                                        writeln!(rows,"{pattern:?} v={v} i={i} m={m} s={s} input={input:?} start={start} sticky={sticky} count={} range={:?} capture={:?}",matcher.capture_count(),found.as_ref().map(|f|f.range.clone()),found.and_then(|f|f.atom_capture)).unwrap();
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        insta::assert_snapshot!(rows);
    }

    #[test]
    fn unicode_repeated_atom_alternatives_agree_with_independent_counts_and_final_character() {
        let alphabet: [u16; 10] = [
            0x41, 0x61, 10, 13, 0x17f, 0x212a, 0xd83d, 0xde00, 0xd800, 0xdc00,
        ];
        let lt = |c| [10, 13, 0x2028, 0x2029].contains(&c);
        for i in [false, true] {
            for m in [false, true] {
                for s in [false, true] {
                    let plans: Vec<_> = CASES.iter().map(|c| plan(c.0, true, i, m, s)).collect();
                    let fold = |c| {
                        if i {
                            match c {
                                0x41 => 0x61,
                                0x17f => 0x73,
                                0x212a => 0x6b,
                                _ => c,
                            }
                        } else {
                            c
                        }
                    };
                    for len in 0..=4 {
                        for ordinal in 0..alphabet.len().pow(len) {
                            let units: Vec<_> = (0..len)
                                .map(|j| alphabet[ordinal / alphabet.len().pow(j) % alphabet.len()])
                                .collect();
                            let mut points = Vec::new();
                            let mut bounds = vec![0];
                            let mut j = 0;
                            while j < units.len() {
                                let mut c = u32::from(units[j]);
                                j += 1;
                                if (0xd800..=0xdbff).contains(&c)
                                    && j < units.len()
                                    && (0xdc00..=0xdfff).contains(&units[j])
                                {
                                    c = 65536 + (c - 0xd800) * 1024 + u32::from(units[j]) - 0xdc00;
                                    j += 1;
                                }
                                points.push(c);
                                bounds.push(j);
                            }
                            let input = JsString::from_code_units(units);
                            for start in 0..=input.len() + 1 {
                                for sticky in [false, true] {
                                    for (case, matcher) in CASES.iter().zip(&plans) {
                                        let expected = if start > input.len() {
                                            None
                                        } else {
                                            let first = bounds.partition_point(|&b| b <= start) - 1;
                                            (first..=points.len()).find_map(|at| {
                                                let word=|c:Option<&u32>|c.is_some_and(|&c|matches!(c,0x30..=0x39|0x41..=0x5a|0x5f|0x61..=0x7a)||(i&&matches!(c,0x17f|0x212a)));
                                                if (sticky && at != first)
                                                    || (case.6
                                                        && at != 0
                                                        && !(m && lt(points[at - 1])))
                                                {
                                                    return None;
                                                }
                                                let available = (at..points.len()).take_while(|&p| {
                                                    let c = points[p];
                                                    let prev = p.checked_sub(1).and_then(|n| points.get(n));
                                                    let next = points.get(p+1);
                                                    let wb = word(prev) != word(Some(&c));
                                                    let we = word(Some(&c)) != word(next);
                                                    let begin = p == 0 || (m && prev.is_some_and(|&c| lt(c)));
                                                    let end = next.is_none() || (m && next.is_some_and(|&c| lt(c)));
                                                    let dot = s || !lt(c);
                                                    let class = [0x61, fold(0x17f), fold(0x212a)].contains(&fold(c));
                                                    match case.1 {
                                                        0 => [0x61,0x62].contains(&fold(c)),
                                                        1 => class,
                                                        2 => dot || c == 10,
                                                        3 => true,
                                                        4 => (0x200c..=0x200d).contains(&c) || matches!(c,0xfdd0..=0xfdef) || c&0xffff >= 0xfffe,
                                                        5 | 14 => c == 0x1f600 || fold(c) == 0x61,
                                                        6 => [0xd800,0xdc00].contains(&c),
                                                        7 => c == 0x7c || fold(c) == 0x61,
                                                        8 => (fold(c) == 0x61 && wb) || (fold(c) == 0x62 && we),
                                                        9 => (fold(c) == 0x61 && begin) || (fold(c) == 0x62 && end),
                                                        10 => [0x61,0x62,0x7c].contains(&fold(c)),
                                                        11 => (dot && !wb) || (class && wb),
                                                        12 => (fold(c) == 0x61 && end) || (fold(c) == 0x62 && begin),
                                                        13 => (0x61..=0x7a).contains(&fold(c)) || (0x200c..=0x200d).contains(&c),
                                                        15 => (dot && begin && end && !wb) || (fold(c) == 0x61 && wb && we),
                                                        _ => unreachable!(),
                                                    }
                                                }).count();
                                                let upper =
                                                    available.min(case.3.unwrap_or(available));
                                                if upper < case.2 {
                                                    return None;
                                                }
                                                let endpoint = |n: usize| {
                                                    let end = at + n;
                                                    if case.7
                                                        && end != points.len()
                                                        && !(m && lt(points[end]))
                                                    {
                                                        return None;
                                                    }
                                                    Some(RegExpUnicodeRepeatedAtomMatch {
                                                        range: bounds[at]..bounds[end],
                                                        atom_capture: (case.5 != 0 && n != 0)
                                                            .then(|| bounds[end - 1]..bounds[end]),
                                                    })
                                                };
                                                let mut counts = case.2..=upper;
                                                if case.4 {
                                                    counts.rev().find_map(endpoint)
                                                } else {
                                                    counts.find_map(endpoint)
                                                }
                                            })
                                        };
                                        assert_eq!(matcher.capture_count(), case.5, "{}", case.0);
                                        assert_eq!(
                                            matcher
                                                .find_with_captures_and_work(
                                                    &input,
                                                    start,
                                                    sticky,
                                                    |_| Ok::<(), ()>(())
                                                )
                                                .unwrap(),
                                            expected,
                                            "{} i={i} m={m} s={s} {input:?} start={start} sticky={sticky}",
                                            case.0
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

    #[test]
    fn unicode_repeated_atom_noncapturing_branch_line_work_clones_and_contracts() {
        let source = format!("{}.|\\n{}+", "(".repeat(3000), ")".repeat(3000));
        let matcher = plan(&source, true, false, true, false).clone();
        let input = JsString::from("\n".repeat(200000).as_str());
        let mut work = 0;
        assert_eq!(matcher.capture_count(), 3000);
        assert_eq!(
            matcher
                .find_with_captures_and_work(&input, 0, false, |n| {
                    work += n;
                    Ok::<(), ()>(())
                })
                .unwrap(),
            Some(RegExpUnicodeRepeatedAtomMatch {
                range: 0..200000,
                atom_capture: Some(199999..200000)
            })
        );
        assert!(work < 50 * input.len(), "{work}");
        let many = format!("({}|b)+", "a|".repeat(20000).trim_end_matches('|'));
        assert_eq!(
            plan(&many, false, false, false, false)
                .find_with_work(&JsString::from("b"), 0, true, |_| Ok::<(), ()>(()))
                .unwrap(),
            Some(0..1)
        );
        let source = JsString::from(r"(^(?:\B.)$|(?:\ba\b)){1,3}$");
        let mut calls = 0;
        RegExpUnicodeRepeatedAtomMatcher::compile_noncapturing_branch_lines_with_work(
            &source,
            true,
            false,
            true,
            true,
            |_| {
                calls += 1;
                Ok::<(), ()>(())
            },
        )
        .unwrap()
        .unwrap();
        for fail in 1..=calls {
            let mut n = 0;
            assert!(
                RegExpUnicodeRepeatedAtomMatcher::compile_noncapturing_branch_lines_with_work(
                    &source,
                    true,
                    false,
                    true,
                    true,
                    |_| {
                        n += 1;
                        if n == fail { Err(()) } else { Ok(()) }
                    }
                )
                .is_err()
            );
        }
        let matcher = plan(r"(^(?:\B.)$|(?:\ba\b)){1,3}$", true, false, true, true);
        let input = JsString::from("\r\n");
        calls = 0;
        matcher
            .find_with_captures_and_work(&input, 0, false, |_| {
                calls += 1;
                Ok::<(), ()>(())
            })
            .unwrap();
        for fail in 1..=calls {
            let mut n = 0;
            assert!(
                matcher
                    .find_with_captures_and_work(&input, 0, false, |_| {
                        n += 1;
                        if n == fail { Err(()) } else { Ok(()) }
                    })
                    .is_err()
            );
        }

        let deep = format!("({}a{}|b)+", "(?:".repeat(6000), ")".repeat(6000));
        assert_eq!(plan(&deep, true, false, false, false).capture_count(), 1);
        assert_eq!(
            plan(&deep, true, false, false, false)
                .find_with_work(&JsString::from("aa"), 0, false, |_| Ok::<(), ()>(()))
                .unwrap(),
            Some(0..2)
        );

        let guarded = format!("(^{}a{}|b)+", "(?:".repeat(6000), ")".repeat(6000));
        assert_eq!(
            plan(&guarded, true, false, false, false)
                .find_with_work(&JsString::from("aa"), 0, false, |_| Ok::<(), ()>(()))
                .unwrap(),
            Some(0..1)
        );
        for source in [
            r"((?:a)|)+",
            r"(|(?:b))+",
            r"((?:a)|(?:ab))+",
            r"((?:a)|(?:b?))+",
            r"((?:a)|(b))+",
            r"((?:a)|(?:\b))+",
            r"((?:a)|(?i:b))+",
            r"((?:a)|(?:b))+c",
            r"\b((?:a)|(?:b))+",
            r"((?:a)|(?:b))+\b",
            r"(\b((?:a)|(?:b)))+",
            r"((a)|(?:b))+",
            r"((?:a)|(?:b)){2}c",
            r"((?:a|b)|(?:c))+",
            r"((?:a){1}|(?:b))+",
            r"((?:a{1})|(?:b))+",
            r"(\b(?:a)|(?:b))+",
        ] {
            assert!(
                RegExpUnicodeRepeatedAtomMatcher::compile_noncapturing_branch_lines_with_work(
                    &JsString::from(source),
                    true,
                    false,
                    true,
                    true,
                    |_| Ok::<(), ()>(())
                )
                .unwrap()
                .is_none(),
                "{source}"
            );
        }
        for source in [
            r"(^(?:a)|(?:b)$)+",
            r"((?:a)$|^(?:b))+",
            r"(^(?:\B.)$|(?:\ba\b))+",
        ] {
            assert!(
                RegExpUnicodeRepeatedAtomMatcher::compile_noncapturing_branches_with_work(
                    &JsString::from(source),
                    true,
                    false,
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
}

#[cfg(test)]
mod noncapturing_branch_line_word_tests {
    use super::*;
    use std::fmt::Write;

    type Case = (
        &'static str,
        u8,
        usize,
        Option<usize>,
        bool,
        usize,
        bool,
        bool,
        Option<bool>,
        Option<bool>,
    );
    const CASES: [Case; 16] = [
        (
            r"\b((?:a)|(?:b))+\B",
            0,
            1,
            None,
            true,
            1,
            false,
            false,
            Some(true),
            Some(false),
        ),
        (
            r"\B(((?:a)|(?:[ſK]))){0,3}?\b",
            1,
            0,
            Some(3),
            false,
            2,
            false,
            false,
            Some(false),
            Some(true),
        ),
        (
            r"\b(?:(?:.)|(?:\n)){2,4}\b",
            2,
            2,
            Some(4),
            true,
            0,
            false,
            false,
            Some(true),
            Some(true),
        ),
        (
            r"\B((?:\d)|(?:\D))*",
            3,
            0,
            None,
            true,
            1,
            false,
            false,
            Some(false),
            None,
        ),
        (
            r"^((?:\p{Join_C})|(?:\p{NChar}))+\B$",
            4,
            1,
            None,
            true,
            1,
            true,
            true,
            None,
            Some(false),
        ),
        (
            r"\b((?:😀)|(?:a)){1,3}\b$",
            5,
            1,
            Some(3),
            true,
            1,
            false,
            true,
            Some(true),
            Some(true),
        ),
        (
            r"((?:\uD800)|(?:\uDC00)){0,2}?\b",
            6,
            0,
            Some(2),
            false,
            1,
            false,
            false,
            None,
            Some(true),
        ),
        (
            r"\b((?:\|)|(?:a))+",
            7,
            1,
            None,
            true,
            1,
            false,
            false,
            Some(true),
            None,
        ),
        (
            r"\b((?:\ba)|(?:b\b))+\B",
            8,
            1,
            None,
            true,
            1,
            false,
            false,
            Some(true),
            Some(false),
        ),
        (
            r"(^(?:a)|(?:b)$){1,2}\b$",
            9,
            1,
            Some(2),
            true,
            1,
            false,
            true,
            None,
            Some(true),
        ),
        (
            r"\B((?:[a\|])|(?:b))??\B",
            10,
            0,
            Some(1),
            false,
            1,
            false,
            false,
            Some(false),
            Some(false),
        ),
        (
            r"\B((?:\B.)|(?:\b[aſK])){1,3}\b",
            11,
            1,
            Some(3),
            true,
            1,
            false,
            false,
            Some(false),
            Some(true),
        ),
        (
            r"((?:a)$|^(?:b))+\b",
            12,
            1,
            None,
            true,
            1,
            false,
            false,
            None,
            Some(true),
        ),
        (
            r"^\b((?:[a-z])|(?:\p{Join_C})){0,2}\B$",
            13,
            0,
            Some(2),
            true,
            1,
            true,
            true,
            Some(true),
            Some(false),
        ),
        (
            r"((?:\u{61})|(?:\u{1f600}))?\b",
            14,
            0,
            Some(1),
            true,
            1,
            false,
            false,
            None,
            Some(true),
        ),
        (
            r"\B(^(?:\B.)$|(?:\ba\b)){1,3}\B$",
            15,
            1,
            Some(3),
            true,
            1,
            false,
            true,
            Some(false),
            Some(false),
        ),
    ];
    fn plan(source: &str, v: bool, i: bool, m: bool, s: bool) -> RegExpUnicodeRepeatedAtomMatcher {
        RegExpUnicodeRepeatedAtomMatcher::compile_noncapturing_branch_line_words_with_work(
            &JsString::from(source),
            v,
            i,
            m,
            s,
            |_| Ok::<(), ()>(()),
        )
        .unwrap()
        .unwrap()
    }

    #[test]
    fn unicode_repeated_atom_noncapturing_branch_line_word_ranges_snapshot() {
        let mut rows = String::new();
        for (source, ..) in CASES {
            let pattern = JsString::from(source);
            for v in [false, true] {
                for i in [false, true] {
                    for m in [false, true] {
                        for s in [false, true] {
                            let matcher = plan(source, v, i, m, s);
                            for input in [
                                JsString::from("Aaaa\nAA"),
                                JsString::from("A\r\n\u{200c}\u{200d}\u{ffff}\u{10ffff}"),
                                JsString::from("😀??aaſK"),
                                JsString::from_code_units(vec![0x61, 0xd800, 0x61, 0xdc00]),
                                JsString::from(""),
                            ] {
                                for start in [0, 1, input.len(), input.len() + 1] {
                                    for sticky in [false, true] {
                                        let found = matcher
                                            .find_with_captures_and_work(
                                                &input,
                                                start,
                                                sticky,
                                                |_| Ok::<(), ()>(()),
                                            )
                                            .unwrap();
                                        writeln!(rows,"{pattern:?} v={v} i={i} m={m} s={s} input={input:?} start={start} sticky={sticky} count={} range={:?} capture={:?}",matcher.capture_count(),found.as_ref().map(|f|f.range.clone()),found.and_then(|f|f.atom_capture)).unwrap();
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        insta::assert_snapshot!(rows);
    }

    #[test]
    fn unicode_repeated_atom_word_alternatives_agree_with_independent_counts_and_final_character() {
        let alphabet: [u16; 10] = [
            0x41, 0x61, 10, 13, 0x17f, 0x212a, 0xd83d, 0xde00, 0xd800, 0xdc00,
        ];
        let lt = |c| [10, 13, 0x2028, 0x2029].contains(&c);
        for i in [false, true] {
            for m in [false, true] {
                for s in [false, true] {
                    let plans: Vec<_> = CASES.iter().map(|c| plan(c.0, true, i, m, s)).collect();
                    let fold = |c| {
                        if i {
                            match c {
                                0x41 => 0x61,
                                0x17f => 0x73,
                                0x212a => 0x6b,
                                _ => c,
                            }
                        } else {
                            c
                        }
                    };
                    for len in 0..=4 {
                        for ordinal in 0..alphabet.len().pow(len) {
                            let units: Vec<_> = (0..len)
                                .map(|j| alphabet[ordinal / alphabet.len().pow(j) % alphabet.len()])
                                .collect();
                            let mut points = Vec::new();
                            let mut bounds = vec![0];
                            let mut j = 0;
                            while j < units.len() {
                                let mut c = u32::from(units[j]);
                                j += 1;
                                if (0xd800..=0xdbff).contains(&c)
                                    && j < units.len()
                                    && (0xdc00..=0xdfff).contains(&units[j])
                                {
                                    c = 65536 + (c - 0xd800) * 1024 + u32::from(units[j]) - 0xdc00;
                                    j += 1;
                                }
                                points.push(c);
                                bounds.push(j);
                            }
                            let input = JsString::from_code_units(units);
                            for start in 0..=input.len() + 1 {
                                for sticky in [false, true] {
                                    for (case, matcher) in CASES.iter().zip(&plans) {
                                        let expected = if start > input.len() {
                                            None
                                        } else {
                                            let first = bounds.partition_point(|&b| b <= start) - 1;
                                            (first..=points.len()).find_map(|at| {
                                                let word=|c:Option<&u32>|c.is_some_and(|&c|matches!(c,0x30..=0x39|0x41..=0x5a|0x5f|0x61..=0x7a)||(i&&matches!(c,0x17f|0x212a)));
                                                if (sticky && at != first)
                                                    || (case.6
                                                        && at != 0
                                                        && !(m && lt(points[at - 1])))
                                                {
                                                    return None;
                                                }
                                                if case.8.is_some_and(|positive| (word(at.checked_sub(1).and_then(|n|points.get(n))) != word(points.get(at))) != positive) { return None; }
                                                let available = (at..points.len()).take_while(|&p| {
                                                    let c = points[p];
                                                    let prev = p.checked_sub(1).and_then(|n| points.get(n));
                                                    let next = points.get(p+1);
                                                    let wb = word(prev) != word(Some(&c));
                                                    let we = word(Some(&c)) != word(next);
                                                    let begin = p == 0 || (m && prev.is_some_and(|&c| lt(c)));
                                                    let end = next.is_none() || (m && next.is_some_and(|&c| lt(c)));
                                                    let dot = s || !lt(c);
                                                    let class = [0x61, fold(0x17f), fold(0x212a)].contains(&fold(c));
                                                    match case.1 {
                                                        0 => [0x61,0x62].contains(&fold(c)),
                                                        1 => class,
                                                        2 => dot || c == 10,
                                                        3 => true,
                                                        4 => (0x200c..=0x200d).contains(&c) || matches!(c,0xfdd0..=0xfdef) || c&0xffff >= 0xfffe,
                                                        5 | 14 => c == 0x1f600 || fold(c) == 0x61,
                                                        6 => [0xd800,0xdc00].contains(&c),
                                                        7 => c == 0x7c || fold(c) == 0x61,
                                                        8 => (fold(c) == 0x61 && wb) || (fold(c) == 0x62 && we),
                                                        9 => (fold(c) == 0x61 && begin) || (fold(c) == 0x62 && end),
                                                        10 => [0x61,0x62,0x7c].contains(&fold(c)),
                                                        11 => (dot && !wb) || (class && wb),
                                                        12 => (fold(c) == 0x61 && end) || (fold(c) == 0x62 && begin),
                                                        13 => (0x61..=0x7a).contains(&fold(c)) || (0x200c..=0x200d).contains(&c),
                                                        15 => (dot && begin && end && !wb) || (fold(c) == 0x61 && wb && we),
                                                        _ => unreachable!(),
                                                    }
                                                }).count();
                                                let upper =
                                                    available.min(case.3.unwrap_or(available));
                                                if upper < case.2 {
                                                    return None;
                                                }
                                                let endpoint = |n: usize| {
                                                    let end = at + n;
                                                    if case.9.is_some_and(|positive| (word(end.checked_sub(1).and_then(|n|points.get(n))) != word(points.get(end))) != positive) { return None; }

                                                    if case.7
                                                        && end != points.len()
                                                        && !(m && lt(points[end]))
                                                    {
                                                        return None;
                                                    }
                                                    Some(RegExpUnicodeRepeatedAtomMatch {
                                                        range: bounds[at]..bounds[end],
                                                        atom_capture: (case.5 != 0 && n != 0)
                                                            .then(|| bounds[end - 1]..bounds[end]),
                                                    })
                                                };
                                                let mut counts = case.2..=upper;
                                                if case.4 {
                                                    counts.rev().find_map(endpoint)
                                                } else {
                                                    counts.find_map(endpoint)
                                                }
                                            })
                                        };
                                        assert_eq!(matcher.capture_count(), case.5, "{}", case.0);
                                        assert_eq!(
                                            matcher
                                                .find_with_captures_and_work(
                                                    &input,
                                                    start,
                                                    sticky,
                                                    |_| Ok::<(), ()>(())
                                                )
                                                .unwrap(),
                                            expected,
                                            "{} i={i} m={m} s={s} {input:?} start={start} sticky={sticky}",
                                            case.0
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

    #[test]
    fn unicode_repeated_atom_noncapturing_branch_line_word_work_and_contracts() {
        let source = format!(
            r"\B{}(?:.)|(?:\n){}+\B$",
            "(".repeat(3000),
            ")".repeat(3000)
        );
        let matcher = plan(&source, true, false, true, false).clone();
        let input = JsString::from("\n".repeat(200000).as_str());
        let mut work = 0;
        assert_eq!(matcher.capture_count(), 3000);
        assert_eq!(
            matcher
                .find_with_captures_and_work(&input, 0, false, |n| {
                    work += n;
                    Ok::<(), ()>(())
                })
                .unwrap(),
            Some(RegExpUnicodeRepeatedAtomMatch {
                range: 0..200000,
                atom_capture: Some(199999..200000)
            })
        );
        assert!(work < 60 * input.len(), "{work}");
        let source = JsString::from(r"\B(^(?:\B.)$|(?:\ba\b)){1,3}\B$");
        let mut calls = 0;
        RegExpUnicodeRepeatedAtomMatcher::compile_noncapturing_branch_line_words_with_work(
            &source,
            true,
            false,
            true,
            true,
            |_| {
                calls += 1;
                Ok::<(), ()>(())
            },
        )
        .unwrap()
        .unwrap();
        for fail in 1..=calls {
            let mut n = 0;
            assert!(
                RegExpUnicodeRepeatedAtomMatcher::compile_noncapturing_branch_line_words_with_work(
                    &source,
                    true,
                    false,
                    true,
                    true,
                    |_| {
                        n += 1;
                        if n == fail { Err(()) } else { Ok(()) }
                    }
                )
                .is_err()
            );
        }
        let input = JsString::from("\r\n");
        let matcher = plan(r"\B(^(?:\B.)$|(?:\ba\b)){1,3}\B$", true, false, true, true);
        calls = 0;
        matcher
            .find_with_captures_and_work(&input, 0, false, |_| {
                calls += 1;
                Ok::<(), ()>(())
            })
            .unwrap();
        for fail in 1..=calls {
            let mut n = 0;
            assert!(
                matcher
                    .find_with_captures_and_work(&input, 0, false, |_| {
                        n += 1;
                        if n == fail { Err(()) } else { Ok(()) }
                    })
                    .is_err()
            );
        }
        for source in [
            r"\b(a|)+",
            r"\b(|b)+",
            r"\b(a|ab)+",
            r"\b((?:a)|(?:b)?)+",
            r"\b(a|(b))+",
            r"\B(a|\b)+",
            r"\b(a|(?i:b))+",
            r"\b((?:a)|(?:b))+c",
            r"\b(\b((?:a)|(?:b)))+",
            r"\b((a)|(b))+",
            r"\b((?:a)|(?:b)){2}c",
        ] {
            assert!(
                RegExpUnicodeRepeatedAtomMatcher::compile_noncapturing_branch_line_words_with_work(
                    &JsString::from(source),
                    true,
                    false,
                    true,
                    true,
                    |_| Ok::<(), ()>(())
                )
                .unwrap()
                .is_none(),
                "{source}"
            );
        }
        for source in [
            r"\b((?:a)|(?:b))+",
            r"(^(?:a)|(?:b)$)+\b",
            r"\b((?:\ba)|(?:b\b))+\B",
        ] {
            assert!(
                RegExpUnicodeRepeatedAtomMatcher::compile_noncapturing_branch_lines_with_work(
                    &JsString::from(source),
                    true,
                    false,
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
}

#[cfg(test)]
mod noncapturing_branch_assertion_tests {
    use super::*;
    use std::fmt::Write;

    type Case = (
        &'static str,
        u8,
        usize,
        Option<usize>,
        bool,
        usize,
        bool,
        bool,
    );
    const CASES: [Case; 16] = [
        (r"((?:a)|(?:b))+", 0, 1, None, true, 1, false, false),
        (
            r"(((?:a)|(?:[ſK]))){0,3}?",
            1,
            0,
            Some(3),
            false,
            2,
            false,
            false,
        ),
        (
            r"(?:(?:.)|(?:\n)){2,4}",
            2,
            2,
            Some(4),
            true,
            0,
            false,
            false,
        ),
        (r"((?:\d)|(?:\D))*", 3, 0, None, true, 1, false, false),
        (
            r"^((?:\p{Join_C})|(?:\p{NChar}))+$",
            4,
            1,
            None,
            true,
            1,
            true,
            true,
        ),
        (r"((?:😀)|(?:a)){1,3}$", 5, 1, Some(3), true, 1, false, true),
        (
            r"((?:\uD800)|(?:\uDC00)){0,2}?",
            6,
            0,
            Some(2),
            false,
            1,
            false,
            false,
        ),
        (r"((?:\|)|(?:a))+", 7, 1, None, true, 1, false, false),
        (r"(\b(?:a)|(?:b)\b)+", 8, 1, None, true, 1, false, false),
        (
            r"(^(?:a)|(?:b)$){1,2}$",
            9,
            1,
            Some(2),
            true,
            1,
            false,
            true,
        ),
        (
            r"((?:[a\|])|(?:b))??",
            10,
            0,
            Some(1),
            false,
            1,
            false,
            false,
        ),
        (
            r"(\B(?:.)|\b(?:[aſK])){1,3}",
            11,
            1,
            Some(3),
            true,
            1,
            false,
            false,
        ),
        (r"((?:a)$|^(?:b))+", 12, 1, None, true, 1, false, false),
        (
            r"^((?:[a-z])|(?:\p{Join_C})){0,2}$",
            13,
            0,
            Some(2),
            true,
            1,
            true,
            true,
        ),
        (
            r"((?:\u{61})|(?:\u{1f600}))?",
            14,
            0,
            Some(1),
            true,
            1,
            false,
            false,
        ),
        (
            r"(^\B(?:.)$|\b(?:a)\b){1,3}$",
            15,
            1,
            Some(3),
            true,
            1,
            false,
            true,
        ),
    ];
    fn plan(source: &str, v: bool, i: bool, m: bool, s: bool) -> RegExpUnicodeRepeatedAtomMatcher {
        RegExpUnicodeRepeatedAtomMatcher::compile_noncapturing_branch_assertions_with_work(
            &JsString::from(source),
            v,
            i,
            m,
            s,
            |_| Ok::<(), ()>(()),
        )
        .unwrap()
        .unwrap_or_else(|| panic!("Unproved line-branch source: {source}"))
    }

    #[test]
    fn unicode_noncapturing_branch_assertion_ranges_snapshot() {
        let mut rows = String::new();
        for (source, ..) in CASES {
            let pattern = JsString::from(source);
            for v in [false, true] {
                for i in [false, true] {
                    for m in [false, true] {
                        for s in [false, true] {
                            let matcher = plan(source, v, i, m, s);
                            for input in [
                                JsString::from("Aaaa\nAA"),
                                JsString::from("A\r\n\u{200c}\u{200d}\u{ffff}\u{10ffff}"),
                                JsString::from("😀??aaſK"),
                                JsString::from_code_units(vec![0x61, 0xd800, 0x61, 0xdc00]),
                                JsString::from(""),
                            ] {
                                for start in [0, 1, input.len(), input.len() + 1] {
                                    for sticky in [false, true] {
                                        let found = matcher
                                            .find_with_captures_and_work(
                                                &input,
                                                start,
                                                sticky,
                                                |_| Ok::<(), ()>(()),
                                            )
                                            .unwrap();
                                        writeln!(rows,"{pattern:?} v={v} i={i} m={m} s={s} input={input:?} start={start} sticky={sticky} count={} range={:?} capture={:?}",matcher.capture_count(),found.as_ref().map(|f|f.range.clone()),found.and_then(|f|f.atom_capture)).unwrap();
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        insta::assert_snapshot!(rows);
    }

    #[test]
    fn unicode_repeated_atom_alternatives_agree_with_independent_counts_and_final_character() {
        let alphabet: [u16; 10] = [
            0x41, 0x61, 10, 13, 0x17f, 0x212a, 0xd83d, 0xde00, 0xd800, 0xdc00,
        ];
        let lt = |c| [10, 13, 0x2028, 0x2029].contains(&c);
        for i in [false, true] {
            for m in [false, true] {
                for s in [false, true] {
                    let plans: Vec<_> = CASES.iter().map(|c| plan(c.0, true, i, m, s)).collect();
                    let fold = |c| {
                        if i {
                            match c {
                                0x41 => 0x61,
                                0x17f => 0x73,
                                0x212a => 0x6b,
                                _ => c,
                            }
                        } else {
                            c
                        }
                    };
                    for len in 0..=4 {
                        for ordinal in 0..alphabet.len().pow(len) {
                            let units: Vec<_> = (0..len)
                                .map(|j| alphabet[ordinal / alphabet.len().pow(j) % alphabet.len()])
                                .collect();
                            let mut points = Vec::new();
                            let mut bounds = vec![0];
                            let mut j = 0;
                            while j < units.len() {
                                let mut c = u32::from(units[j]);
                                j += 1;
                                if (0xd800..=0xdbff).contains(&c)
                                    && j < units.len()
                                    && (0xdc00..=0xdfff).contains(&units[j])
                                {
                                    c = 65536 + (c - 0xd800) * 1024 + u32::from(units[j]) - 0xdc00;
                                    j += 1;
                                }
                                points.push(c);
                                bounds.push(j);
                            }
                            let input = JsString::from_code_units(units);
                            for start in 0..=input.len() + 1 {
                                for sticky in [false, true] {
                                    for (case, matcher) in CASES.iter().zip(&plans) {
                                        let expected = if start > input.len() {
                                            None
                                        } else {
                                            let first = bounds.partition_point(|&b| b <= start) - 1;
                                            (first..=points.len()).find_map(|at| {
                                                let word=|c:Option<&u32>|c.is_some_and(|&c|matches!(c,0x30..=0x39|0x41..=0x5a|0x5f|0x61..=0x7a)||(i&&matches!(c,0x17f|0x212a)));
                                                if (sticky && at != first)
                                                    || (case.6
                                                        && at != 0
                                                        && !(m && lt(points[at - 1])))
                                                {
                                                    return None;
                                                }
                                                let available = (at..points.len()).take_while(|&p| {
                                                    let c = points[p];
                                                    let prev = p.checked_sub(1).and_then(|n| points.get(n));
                                                    let next = points.get(p+1);
                                                    let wb = word(prev) != word(Some(&c));
                                                    let we = word(Some(&c)) != word(next);
                                                    let begin = p == 0 || (m && prev.is_some_and(|&c| lt(c)));
                                                    let end = next.is_none() || (m && next.is_some_and(|&c| lt(c)));
                                                    let dot = s || !lt(c);
                                                    let class = [0x61, fold(0x17f), fold(0x212a)].contains(&fold(c));
                                                    match case.1 {
                                                        0 => [0x61,0x62].contains(&fold(c)),
                                                        1 => class,
                                                        2 => dot || c == 10,
                                                        3 => true,
                                                        4 => (0x200c..=0x200d).contains(&c) || matches!(c,0xfdd0..=0xfdef) || c&0xffff >= 0xfffe,
                                                        5 | 14 => c == 0x1f600 || fold(c) == 0x61,
                                                        6 => [0xd800,0xdc00].contains(&c),
                                                        7 => c == 0x7c || fold(c) == 0x61,
                                                        8 => (fold(c) == 0x61 && wb) || (fold(c) == 0x62 && we),
                                                        9 => (fold(c) == 0x61 && begin) || (fold(c) == 0x62 && end),
                                                        10 => [0x61,0x62,0x7c].contains(&fold(c)),
                                                        11 => (dot && !wb) || (class && wb),
                                                        12 => (fold(c) == 0x61 && end) || (fold(c) == 0x62 && begin),
                                                        13 => (0x61..=0x7a).contains(&fold(c)) || (0x200c..=0x200d).contains(&c),
                                                        15 => (dot && begin && end && !wb) || (fold(c) == 0x61 && wb && we),
                                                        _ => unreachable!(),
                                                    }
                                                }).count();
                                                let upper =
                                                    available.min(case.3.unwrap_or(available));
                                                if upper < case.2 {
                                                    return None;
                                                }
                                                let endpoint = |n: usize| {
                                                    let end = at + n;
                                                    if case.7
                                                        && end != points.len()
                                                        && !(m && lt(points[end]))
                                                    {
                                                        return None;
                                                    }
                                                    Some(RegExpUnicodeRepeatedAtomMatch {
                                                        range: bounds[at]..bounds[end],
                                                        atom_capture: (case.5 != 0 && n != 0)
                                                            .then(|| bounds[end - 1]..bounds[end]),
                                                    })
                                                };
                                                let mut counts = case.2..=upper;
                                                if case.4 {
                                                    counts.rev().find_map(endpoint)
                                                } else {
                                                    counts.find_map(endpoint)
                                                }
                                            })
                                        };
                                        assert_eq!(matcher.capture_count(), case.5, "{}", case.0);
                                        assert_eq!(
                                            matcher
                                                .find_with_captures_and_work(
                                                    &input,
                                                    start,
                                                    sticky,
                                                    |_| Ok::<(), ()>(())
                                                )
                                                .unwrap(),
                                            expected,
                                            "{} i={i} m={m} s={s} {input:?} start={start} sticky={sticky}",
                                            case.0
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

    #[test]
    fn unicode_noncapturing_branch_assertion_work_clones_and_contracts() {
        let source = format!("{}.|\\n{}+", "(".repeat(3000), ")".repeat(3000));
        let matcher = plan(&source, true, false, true, false).clone();
        let input = JsString::from("\n".repeat(200000).as_str());
        let mut work = 0;
        assert_eq!(matcher.capture_count(), 3000);
        assert_eq!(
            matcher
                .find_with_captures_and_work(&input, 0, false, |n| {
                    work += n;
                    Ok::<(), ()>(())
                })
                .unwrap(),
            Some(RegExpUnicodeRepeatedAtomMatch {
                range: 0..200000,
                atom_capture: Some(199999..200000)
            })
        );
        assert!(work < 50 * input.len(), "{work}");

        for source in [
            r"(\b(?:a)|(?:b))+",
            r"(\b(?:a)|b)+",
            r"(a|(?:b)\b)+",
            r"(^\B(?:.)$|\b(?:a)\b)+",
        ] {
            let source = JsString::from(source);
            assert!(
                RegExpUnicodeRepeatedAtomMatcher::compile_noncapturing_branch_lines_with_work(
                    &source,
                    true,
                    false,
                    true,
                    true,
                    |_| Ok::<(), ()>(())
                )
                .unwrap()
                .is_none()
            );
            assert!(
                RegExpUnicodeRepeatedAtomMatcher::compile_noncapturing_branch_assertions_with_work(
                    &source,
                    true,
                    false,
                    true,
                    true,
                    |_| Ok::<(), ()>(())
                )
                .unwrap()
                .is_some()
            );
        }
        let many = format!("({}|b)+", "a|".repeat(20000).trim_end_matches('|'));
        assert_eq!(
            plan(&many, false, false, false, false)
                .find_with_work(&JsString::from("b"), 0, true, |_| Ok::<(), ()>(()))
                .unwrap(),
            Some(0..1)
        );
        let source = JsString::from(r"(^\B(?:.)$|\b(?:a)\b){1,3}$");
        let mut calls = 0;
        RegExpUnicodeRepeatedAtomMatcher::compile_noncapturing_branch_assertions_with_work(
            &source,
            true,
            false,
            true,
            true,
            |_| {
                calls += 1;
                Ok::<(), ()>(())
            },
        )
        .unwrap()
        .unwrap();
        for fail in 1..=calls {
            let mut n = 0;
            assert!(
                RegExpUnicodeRepeatedAtomMatcher::compile_noncapturing_branch_assertions_with_work(
                    &source,
                    true,
                    false,
                    true,
                    true,
                    |_| {
                        n += 1;
                        if n == fail { Err(()) } else { Ok(()) }
                    }
                )
                .is_err()
            );
        }
        let matcher = plan(r"(^\B(?:.)$|\b(?:a)\b){1,3}$", true, false, true, true);
        let input = JsString::from("\r\n");
        calls = 0;
        matcher
            .find_with_captures_and_work(&input, 0, false, |_| {
                calls += 1;
                Ok::<(), ()>(())
            })
            .unwrap();
        for fail in 1..=calls {
            let mut n = 0;
            assert!(
                matcher
                    .find_with_captures_and_work(&input, 0, false, |_| {
                        n += 1;
                        if n == fail { Err(()) } else { Ok(()) }
                    })
                    .is_err()
            );
        }

        let deep = format!("({}a{}|b)+", "(?:".repeat(6000), ")".repeat(6000));
        assert_eq!(plan(&deep, true, false, false, false).capture_count(), 1);
        assert_eq!(
            plan(&deep, true, false, false, false)
                .find_with_work(&JsString::from("aa"), 0, false, |_| Ok::<(), ()>(()))
                .unwrap(),
            Some(0..2)
        );

        let guarded = format!("(^{}a{}|b)+", "(?:".repeat(6000), ")".repeat(6000));
        assert_eq!(
            plan(&guarded, true, false, false, false)
                .find_with_work(&JsString::from("aa"), 0, false, |_| Ok::<(), ()>(()))
                .unwrap(),
            Some(0..1)
        );
        for source in [
            r"((?:a)|)+",
            r"(|(?:b))+",
            r"((?:a)|(?:ab))+",
            r"((?:a)|(?:b?))+",
            r"((?:a)|(b))+",
            r"((?:a)|(?:\b))+",
            r"((?:a)|(?i:b))+",
            r"((?:a)|(?:b))+c",
            r"\b((?:a)|(?:b))+",
            r"((?:a)|(?:b))+\b",
            r"(\b((?:a)|(?:b)))+",
            r"((a)|(?:b))+",
            r"((?:a)|(?:b)){2}c",
            r"((?:a|b)|(?:c))+",
            r"((?:a){1}|(?:b))+",
            r"((?:a{1})|(?:b))+",
            r"(\b(?:a)|(?:b))+c",
        ] {
            assert!(
                RegExpUnicodeRepeatedAtomMatcher::compile_noncapturing_branch_assertions_with_work(
                    &JsString::from(source),
                    true,
                    false,
                    true,
                    true,
                    |_| Ok::<(), ()>(())
                )
                .unwrap()
                .is_none(),
                "{source}"
            );
        }
        for source in [
            r"(^(?:a)|(?:b)$)+",
            r"((?:a)$|^(?:b))+",
            r"(^\B(?:.)$|\b(?:a)\b)+",
        ] {
            assert!(
                RegExpUnicodeRepeatedAtomMatcher::compile_noncapturing_branches_with_work(
                    &JsString::from(source),
                    true,
                    false,
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
}

#[cfg(test)]
mod noncapturing_branch_scoped_assertion_tests {
    use super::*;
    use std::fmt::Write;

    type Case = (
        &'static str,
        u8,
        usize,
        Option<usize>,
        bool,
        usize,
        bool,
        bool,
        Option<bool>,
        Option<bool>,
    );
    const CASES: [Case; 16] = [
        (
            r"\b((?:a)|(?:b))+\B",
            0,
            1,
            None,
            true,
            1,
            false,
            false,
            Some(true),
            Some(false),
        ),
        (
            r"\B(((?:a)|(?:[ſK]))){0,3}?\b",
            1,
            0,
            Some(3),
            false,
            2,
            false,
            false,
            Some(false),
            Some(true),
        ),
        (
            r"\b(?:(?:.)|(?:\n)){2,4}\b",
            2,
            2,
            Some(4),
            true,
            0,
            false,
            false,
            Some(true),
            Some(true),
        ),
        (
            r"\B((?:\d)|(?:\D))*",
            3,
            0,
            None,
            true,
            1,
            false,
            false,
            Some(false),
            None,
        ),
        (
            r"^((?:\p{Join_C})|(?:\p{NChar}))+\B$",
            4,
            1,
            None,
            true,
            1,
            true,
            true,
            None,
            Some(false),
        ),
        (
            r"\b((?:😀)|(?:a)){1,3}\b$",
            5,
            1,
            Some(3),
            true,
            1,
            false,
            true,
            Some(true),
            Some(true),
        ),
        (
            r"((?:\uD800)|(?:\uDC00)){0,2}?\b",
            6,
            0,
            Some(2),
            false,
            1,
            false,
            false,
            None,
            Some(true),
        ),
        (
            r"\b((?:\|)|(?:a))+",
            7,
            1,
            None,
            true,
            1,
            false,
            false,
            Some(true),
            None,
        ),
        (
            r"\b(\b(?:a)|(?:b)\b)+\B",
            8,
            1,
            None,
            true,
            1,
            false,
            false,
            Some(true),
            Some(false),
        ),
        (
            r"(^(?:a)|(?:b)$){1,2}\b$",
            9,
            1,
            Some(2),
            true,
            1,
            false,
            true,
            None,
            Some(true),
        ),
        (
            r"\B((?:[a\|])|(?:b))??\B",
            10,
            0,
            Some(1),
            false,
            1,
            false,
            false,
            Some(false),
            Some(false),
        ),
        (
            r"\B(\B(?:.)|\b(?:[aſK])){1,3}\b",
            11,
            1,
            Some(3),
            true,
            1,
            false,
            false,
            Some(false),
            Some(true),
        ),
        (
            r"((?:a)$|^(?:b))+\b",
            12,
            1,
            None,
            true,
            1,
            false,
            false,
            None,
            Some(true),
        ),
        (
            r"^\b((?:[a-z])|(?:\p{Join_C})){0,2}\B$",
            13,
            0,
            Some(2),
            true,
            1,
            true,
            true,
            Some(true),
            Some(false),
        ),
        (
            r"((?:\u{61})|(?:\u{1f600}))?\b",
            14,
            0,
            Some(1),
            true,
            1,
            false,
            false,
            None,
            Some(true),
        ),
        (
            r"\B(^\B(?:.)$|\b(?:a)\b){1,3}\B$",
            15,
            1,
            Some(3),
            true,
            1,
            false,
            true,
            Some(false),
            Some(false),
        ),
    ];
    fn plan(source: &str, v: bool, i: bool, m: bool, s: bool) -> RegExpUnicodeRepeatedAtomMatcher {
        RegExpUnicodeRepeatedAtomMatcher::compile_noncapturing_branch_scoped_assertions_with_work(
            &JsString::from(source),
            v,
            i,
            m,
            s,
            |_| Ok::<(), ()>(()),
        )
        .unwrap()
        .unwrap()
    }

    #[test]
    fn unicode_noncapturing_branch_scoped_assertion_ranges_snapshot() {
        let mut rows = String::new();
        for (source, ..) in CASES {
            let pattern = JsString::from(source);
            for v in [false, true] {
                for i in [false, true] {
                    for m in [false, true] {
                        for s in [false, true] {
                            let matcher = plan(source, v, i, m, s);
                            for input in [
                                JsString::from("Aaaa\nAA"),
                                JsString::from("A\r\n\u{200c}\u{200d}\u{ffff}\u{10ffff}"),
                                JsString::from("😀??aaſK"),
                                JsString::from_code_units(vec![0x61, 0xd800, 0x61, 0xdc00]),
                                JsString::from(""),
                            ] {
                                for start in [0, 1, input.len(), input.len() + 1] {
                                    for sticky in [false, true] {
                                        let found = matcher
                                            .find_with_captures_and_work(
                                                &input,
                                                start,
                                                sticky,
                                                |_| Ok::<(), ()>(()),
                                            )
                                            .unwrap();
                                        writeln!(rows,"{pattern:?} v={v} i={i} m={m} s={s} input={input:?} start={start} sticky={sticky} count={} range={:?} capture={:?}",matcher.capture_count(),found.as_ref().map(|f|f.range.clone()),found.and_then(|f|f.atom_capture)).unwrap();
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        insta::assert_snapshot!(rows);
    }

    #[test]
    fn unicode_repeated_atom_word_alternatives_agree_with_independent_counts_and_final_character() {
        let alphabet: [u16; 10] = [
            0x41, 0x61, 10, 13, 0x17f, 0x212a, 0xd83d, 0xde00, 0xd800, 0xdc00,
        ];
        let lt = |c| [10, 13, 0x2028, 0x2029].contains(&c);
        for i in [false, true] {
            for m in [false, true] {
                for s in [false, true] {
                    let plans: Vec<_> = CASES.iter().map(|c| plan(c.0, true, i, m, s)).collect();
                    let fold = |c| {
                        if i {
                            match c {
                                0x41 => 0x61,
                                0x17f => 0x73,
                                0x212a => 0x6b,
                                _ => c,
                            }
                        } else {
                            c
                        }
                    };
                    for len in 0..=4 {
                        for ordinal in 0..alphabet.len().pow(len) {
                            let units: Vec<_> = (0..len)
                                .map(|j| alphabet[ordinal / alphabet.len().pow(j) % alphabet.len()])
                                .collect();
                            let mut points = Vec::new();
                            let mut bounds = vec![0];
                            let mut j = 0;
                            while j < units.len() {
                                let mut c = u32::from(units[j]);
                                j += 1;
                                if (0xd800..=0xdbff).contains(&c)
                                    && j < units.len()
                                    && (0xdc00..=0xdfff).contains(&units[j])
                                {
                                    c = 65536 + (c - 0xd800) * 1024 + u32::from(units[j]) - 0xdc00;
                                    j += 1;
                                }
                                points.push(c);
                                bounds.push(j);
                            }
                            let input = JsString::from_code_units(units);
                            for start in 0..=input.len() + 1 {
                                for sticky in [false, true] {
                                    for (case, matcher) in CASES.iter().zip(&plans) {
                                        let expected = if start > input.len() {
                                            None
                                        } else {
                                            let first = bounds.partition_point(|&b| b <= start) - 1;
                                            (first..=points.len()).find_map(|at| {
                                                let word=|c:Option<&u32>|c.is_some_and(|&c|matches!(c,0x30..=0x39|0x41..=0x5a|0x5f|0x61..=0x7a)||(i&&matches!(c,0x17f|0x212a)));
                                                if (sticky && at != first)
                                                    || (case.6
                                                        && at != 0
                                                        && !(m && lt(points[at - 1])))
                                                {
                                                    return None;
                                                }
                                                if case.8.is_some_and(|positive| (word(at.checked_sub(1).and_then(|n|points.get(n))) != word(points.get(at))) != positive) { return None; }
                                                let available = (at..points.len()).take_while(|&p| {
                                                    let c = points[p];
                                                    let prev = p.checked_sub(1).and_then(|n| points.get(n));
                                                    let next = points.get(p+1);
                                                    let wb = word(prev) != word(Some(&c));
                                                    let we = word(Some(&c)) != word(next);
                                                    let begin = p == 0 || (m && prev.is_some_and(|&c| lt(c)));
                                                    let end = next.is_none() || (m && next.is_some_and(|&c| lt(c)));
                                                    let dot = s || !lt(c);
                                                    let class = [0x61, fold(0x17f), fold(0x212a)].contains(&fold(c));
                                                    match case.1 {
                                                        0 => [0x61,0x62].contains(&fold(c)),
                                                        1 => class,
                                                        2 => dot || c == 10,
                                                        3 => true,
                                                        4 => (0x200c..=0x200d).contains(&c) || matches!(c,0xfdd0..=0xfdef) || c&0xffff >= 0xfffe,
                                                        5 | 14 => c == 0x1f600 || fold(c) == 0x61,
                                                        6 => [0xd800,0xdc00].contains(&c),
                                                        7 => c == 0x7c || fold(c) == 0x61,
                                                        8 => (fold(c) == 0x61 && wb) || (fold(c) == 0x62 && we),
                                                        9 => (fold(c) == 0x61 && begin) || (fold(c) == 0x62 && end),
                                                        10 => [0x61,0x62,0x7c].contains(&fold(c)),
                                                        11 => (dot && !wb) || (class && wb),
                                                        12 => (fold(c) == 0x61 && end) || (fold(c) == 0x62 && begin),
                                                        13 => (0x61..=0x7a).contains(&fold(c)) || (0x200c..=0x200d).contains(&c),
                                                        15 => (dot && begin && end && !wb) || (fold(c) == 0x61 && wb && we),
                                                        _ => unreachable!(),
                                                    }
                                                }).count();
                                                let upper =
                                                    available.min(case.3.unwrap_or(available));
                                                if upper < case.2 {
                                                    return None;
                                                }
                                                let endpoint = |n: usize| {
                                                    let end = at + n;
                                                    if case.9.is_some_and(|positive| (word(end.checked_sub(1).and_then(|n|points.get(n))) != word(points.get(end))) != positive) { return None; }

                                                    if case.7
                                                        && end != points.len()
                                                        && !(m && lt(points[end]))
                                                    {
                                                        return None;
                                                    }
                                                    Some(RegExpUnicodeRepeatedAtomMatch {
                                                        range: bounds[at]..bounds[end],
                                                        atom_capture: (case.5 != 0 && n != 0)
                                                            .then(|| bounds[end - 1]..bounds[end]),
                                                    })
                                                };
                                                let mut counts = case.2..=upper;
                                                if case.4 {
                                                    counts.rev().find_map(endpoint)
                                                } else {
                                                    counts.find_map(endpoint)
                                                }
                                            })
                                        };
                                        assert_eq!(matcher.capture_count(), case.5, "{}", case.0);
                                        assert_eq!(
                                            matcher
                                                .find_with_captures_and_work(
                                                    &input,
                                                    start,
                                                    sticky,
                                                    |_| Ok::<(), ()>(())
                                                )
                                                .unwrap(),
                                            expected,
                                            "{} i={i} m={m} s={s} {input:?} start={start} sticky={sticky}",
                                            case.0
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

    #[test]
    fn unicode_noncapturing_branch_scoped_assertion_work_and_contracts() {
        let source = format!(
            r"\B{}(?:.)|(?:\n){}+\B$",
            "(".repeat(3000),
            ")".repeat(3000)
        );
        let matcher = plan(&source, true, false, true, false).clone();
        let input = JsString::from("\n".repeat(200000).as_str());
        let mut work = 0;
        assert_eq!(matcher.capture_count(), 3000);
        assert_eq!(
            matcher
                .find_with_captures_and_work(&input, 0, false, |n| {
                    work += n;
                    Ok::<(), ()>(())
                })
                .unwrap(),
            Some(RegExpUnicodeRepeatedAtomMatch {
                range: 0..200000,
                atom_capture: Some(199999..200000)
            })
        );
        assert!(work < 60 * input.len(), "{work}");
        let source = JsString::from(r"\B(^\B(?:.)$|\b(?:a)\b){1,3}\B$");

        for source in [
            r"\b(\b(?:a)|b)+\b",
            r"\b(a|(?:b)\b)+",
            r"\B(^\B(?:.)$|\b(?:a)\b)+\B",
        ] {
            let source = JsString::from(source);
            assert!(
                RegExpUnicodeRepeatedAtomMatcher::compile_noncapturing_branch_line_words_with_work(
                    &source,
                    true,
                    false,
                    true,
                    true,
                    |_| Ok::<(), ()>(())
                )
                .unwrap()
                .is_none()
            );
            assert!(RegExpUnicodeRepeatedAtomMatcher::compile_noncapturing_branch_scoped_assertions_with_work(
                &source, true, false, true, true, |_| Ok::<(), ()>(())
            ).unwrap().is_some());
        }
        let mut calls = 0;
        RegExpUnicodeRepeatedAtomMatcher::compile_noncapturing_branch_scoped_assertions_with_work(
            &source,
            true,
            false,
            true,
            true,
            |_| {
                calls += 1;
                Ok::<(), ()>(())
            },
        )
        .unwrap()
        .unwrap();
        for fail in 1..=calls {
            let mut n = 0;
            assert!(
                RegExpUnicodeRepeatedAtomMatcher::compile_noncapturing_branch_scoped_assertions_with_work(
                    &source,
                    true,
                    false,
                    true,
                    true,
                    |_| {
                        n += 1;
                        if n == fail { Err(()) } else { Ok(()) }
                    }
                )
                .is_err()
            );
        }
        let input = JsString::from("\r\n");
        let matcher = plan(r"\B(^\B(?:.)$|\b(?:a)\b){1,3}\B$", true, false, true, true);
        calls = 0;
        matcher
            .find_with_captures_and_work(&input, 0, false, |_| {
                calls += 1;
                Ok::<(), ()>(())
            })
            .unwrap();
        for fail in 1..=calls {
            let mut n = 0;
            assert!(
                matcher
                    .find_with_captures_and_work(&input, 0, false, |_| {
                        n += 1;
                        if n == fail { Err(()) } else { Ok(()) }
                    })
                    .is_err()
            );
        }
        for source in [
            r"\b(a|)+",
            r"\b(|b)+",
            r"\b(a|ab)+",
            r"\b((?:a)|(?:b)?)+",
            r"\b(a|(b))+",
            r"\B(a|\b)+",
            r"\b(a|(?i:b))+",
            r"\b((?:a)|(?:b))+c",
            r"\b(\b((?:a)|(?:b)))+",
            r"\b((a)|(b))+",
            r"\b((?:a)|(?:b)){2}c",
        ] {
            assert!(
                RegExpUnicodeRepeatedAtomMatcher::compile_noncapturing_branch_scoped_assertions_with_work(
                    &JsString::from(source),
                    true,
                    false,
                    true,
                    true,
                    |_| Ok::<(), ()>(())
                )
                .unwrap()
                .is_none(),
                "{source}"
            );
        }
        for source in [
            r"\b((?:a)|(?:b))+",
            r"(^(?:a)|(?:b)$)+\b",
            r"\b(\b(?:a)|(?:b)\b)+\B",
        ] {
            assert!(
                RegExpUnicodeRepeatedAtomMatcher::compile_noncapturing_branch_lines_with_work(
                    &JsString::from(source),
                    true,
                    false,
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
}

#[cfg(test)]
mod noncapturing_branch_assertion_sequence_tests {
    use super::*;
    use std::fmt::Write;

    type Case = (
        &'static str,
        u8,
        usize,
        Option<usize>,
        bool,
        usize,
        bool,
        bool,
    );
    const CASES: [Case; 16] = [
        (r"((?:a)|(?:b))+", 0, 1, None, true, 1, false, false),
        (
            r"(((?:a)|(?:[ſK]))){0,3}?",
            1,
            0,
            Some(3),
            false,
            2,
            false,
            false,
        ),
        (
            r"(?:(?:.)|(?:\n)){2,4}",
            2,
            2,
            Some(4),
            true,
            0,
            false,
            false,
        ),
        (r"((?:\d)|(?:\D))*", 3, 0, None, true, 1, false, false),
        (
            r"^((?:\p{Join_C})|(?:\p{NChar}))+$",
            4,
            1,
            None,
            true,
            1,
            true,
            true,
        ),
        (r"((?:😀)|(?:a)){1,3}$", 5, 1, Some(3), true, 1, false, true),
        (
            r"((?:\uD800)|(?:\uDC00)){0,2}?",
            6,
            0,
            Some(2),
            false,
            1,
            false,
            false,
        ),
        (r"((?:\|)|(?:a))+", 7, 1, None, true, 1, false, false),
        (r"(\b\b(?:a)|(?:b)\b\b)+", 8, 1, None, true, 1, false, false),
        (
            r"(^^(?:a)|(?:b)$$){1,2}$",
            9,
            1,
            Some(2),
            true,
            1,
            false,
            true,
        ),
        (
            r"((?:[a\|])|(?:b))??",
            10,
            0,
            Some(1),
            false,
            1,
            false,
            false,
        ),
        (
            r"(\B\B(?:.)|\b\b(?:[aſK])){1,3}",
            11,
            1,
            Some(3),
            true,
            1,
            false,
            false,
        ),
        (r"((?:a)$$|^^(?:b))+", 12, 1, None, true, 1, false, false),
        (
            r"^((?:[a-z])|(?:\p{Join_C})){0,2}$",
            13,
            0,
            Some(2),
            true,
            1,
            true,
            true,
        ),
        (
            r"((?:\u{61})|(?:\u{1f600}))?",
            14,
            0,
            Some(1),
            true,
            1,
            false,
            false,
        ),
        (
            r"(\B^\B(?:.)$$|\b\b(?:a)\b\b){1,3}$",
            15,
            1,
            Some(3),
            true,
            1,
            false,
            true,
        ),
    ];
    fn plan(source: &str, v: bool, i: bool, m: bool, s: bool) -> RegExpUnicodeRepeatedAtomMatcher {
        RegExpUnicodeRepeatedAtomMatcher::compile_noncapturing_branch_assertion_sequences_with_work(
            &JsString::from(source),
            v,
            i,
            m,
            s,
            |_| Ok::<(), ()>(()),
        )
        .unwrap()
        .unwrap_or_else(|| panic!("Unproved line-branch source: {source}"))
    }

    #[test]
    fn unicode_noncapturing_branch_assertion_sequence_ranges_snapshot() {
        let mut rows = String::new();
        for (source, ..) in CASES {
            let pattern = JsString::from(source);
            for v in [false, true] {
                for i in [false, true] {
                    for m in [false, true] {
                        for s in [false, true] {
                            let matcher = plan(source, v, i, m, s);
                            for input in [
                                JsString::from("Aaaa\nAA"),
                                JsString::from("A\r\n\u{200c}\u{200d}\u{ffff}\u{10ffff}"),
                                JsString::from("😀??aaſK"),
                                JsString::from_code_units(vec![0x61, 0xd800, 0x61, 0xdc00]),
                                JsString::from(""),
                            ] {
                                for start in [0, 1, input.len(), input.len() + 1] {
                                    for sticky in [false, true] {
                                        let found = matcher
                                            .find_with_captures_and_work(
                                                &input,
                                                start,
                                                sticky,
                                                |_| Ok::<(), ()>(()),
                                            )
                                            .unwrap();
                                        writeln!(rows,"{pattern:?} v={v} i={i} m={m} s={s} input={input:?} start={start} sticky={sticky} count={} range={:?} capture={:?}",matcher.capture_count(),found.as_ref().map(|f|f.range.clone()),found.and_then(|f|f.atom_capture)).unwrap();
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        insta::assert_snapshot!(rows);
    }

    #[test]
    fn unicode_repeated_atom_alternatives_agree_with_independent_counts_and_final_character() {
        let alphabet: [u16; 10] = [
            0x41, 0x61, 10, 13, 0x17f, 0x212a, 0xd83d, 0xde00, 0xd800, 0xdc00,
        ];
        let lt = |c| [10, 13, 0x2028, 0x2029].contains(&c);
        for i in [false, true] {
            for m in [false, true] {
                for s in [false, true] {
                    let plans: Vec<_> = CASES.iter().map(|c| plan(c.0, true, i, m, s)).collect();
                    let fold = |c| {
                        if i {
                            match c {
                                0x41 => 0x61,
                                0x17f => 0x73,
                                0x212a => 0x6b,
                                _ => c,
                            }
                        } else {
                            c
                        }
                    };
                    for len in 0..=4 {
                        for ordinal in 0..alphabet.len().pow(len) {
                            let units: Vec<_> = (0..len)
                                .map(|j| alphabet[ordinal / alphabet.len().pow(j) % alphabet.len()])
                                .collect();
                            let mut points = Vec::new();
                            let mut bounds = vec![0];
                            let mut j = 0;
                            while j < units.len() {
                                let mut c = u32::from(units[j]);
                                j += 1;
                                if (0xd800..=0xdbff).contains(&c)
                                    && j < units.len()
                                    && (0xdc00..=0xdfff).contains(&units[j])
                                {
                                    c = 65536 + (c - 0xd800) * 1024 + u32::from(units[j]) - 0xdc00;
                                    j += 1;
                                }
                                points.push(c);
                                bounds.push(j);
                            }
                            let input = JsString::from_code_units(units);
                            for start in 0..=input.len() + 1 {
                                for sticky in [false, true] {
                                    for (case, matcher) in CASES.iter().zip(&plans) {
                                        let expected = if start > input.len() {
                                            None
                                        } else {
                                            let first = bounds.partition_point(|&b| b <= start) - 1;
                                            (first..=points.len()).find_map(|at| {
                                                let word=|c:Option<&u32>|c.is_some_and(|&c|matches!(c,0x30..=0x39|0x41..=0x5a|0x5f|0x61..=0x7a)||(i&&matches!(c,0x17f|0x212a)));
                                                if (sticky && at != first)
                                                    || (case.6
                                                        && at != 0
                                                        && !(m && lt(points[at - 1])))
                                                {
                                                    return None;
                                                }
                                                let available = (at..points.len()).take_while(|&p| {
                                                    let c = points[p];
                                                    let prev = p.checked_sub(1).and_then(|n| points.get(n));
                                                    let next = points.get(p+1);
                                                    let wb = word(prev) != word(Some(&c));
                                                    let we = word(Some(&c)) != word(next);
                                                    let begin = p == 0 || (m && prev.is_some_and(|&c| lt(c)));
                                                    let end = next.is_none() || (m && next.is_some_and(|&c| lt(c)));
                                                    let dot = s || !lt(c);
                                                    let class = [0x61, fold(0x17f), fold(0x212a)].contains(&fold(c));
                                                    match case.1 {
                                                        0 => [0x61,0x62].contains(&fold(c)),
                                                        1 => class,
                                                        2 => dot || c == 10,
                                                        3 => true,
                                                        4 => (0x200c..=0x200d).contains(&c) || matches!(c,0xfdd0..=0xfdef) || c&0xffff >= 0xfffe,
                                                        5 | 14 => c == 0x1f600 || fold(c) == 0x61,
                                                        6 => [0xd800,0xdc00].contains(&c),
                                                        7 => c == 0x7c || fold(c) == 0x61,
                                                        8 => (fold(c) == 0x61 && wb) || (fold(c) == 0x62 && we),
                                                        9 => (fold(c) == 0x61 && begin) || (fold(c) == 0x62 && end),
                                                        10 => [0x61,0x62,0x7c].contains(&fold(c)),
                                                        11 => (dot && !wb) || (class && wb),
                                                        12 => (fold(c) == 0x61 && end) || (fold(c) == 0x62 && begin),
                                                        13 => (0x61..=0x7a).contains(&fold(c)) || (0x200c..=0x200d).contains(&c),
                                                        15 => (dot && begin && end && !wb) || (fold(c) == 0x61 && wb && we),
                                                        _ => unreachable!(),
                                                    }
                                                }).count();
                                                let upper =
                                                    available.min(case.3.unwrap_or(available));
                                                if upper < case.2 {
                                                    return None;
                                                }
                                                let endpoint = |n: usize| {
                                                    let end = at + n;
                                                    if case.7
                                                        && end != points.len()
                                                        && !(m && lt(points[end]))
                                                    {
                                                        return None;
                                                    }
                                                    Some(RegExpUnicodeRepeatedAtomMatch {
                                                        range: bounds[at]..bounds[end],
                                                        atom_capture: (case.5 != 0 && n != 0)
                                                            .then(|| bounds[end - 1]..bounds[end]),
                                                    })
                                                };
                                                let mut counts = case.2..=upper;
                                                if case.4 {
                                                    counts.rev().find_map(endpoint)
                                                } else {
                                                    counts.find_map(endpoint)
                                                }
                                            })
                                        };
                                        assert_eq!(matcher.capture_count(), case.5, "{}", case.0);
                                        assert_eq!(
                                            matcher
                                                .find_with_captures_and_work(
                                                    &input,
                                                    start,
                                                    sticky,
                                                    |_| Ok::<(), ()>(())
                                                )
                                                .unwrap(),
                                            expected,
                                            "{} i={i} m={m} s={s} {input:?} start={start} sticky={sticky}",
                                            case.0
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

    #[test]
    fn unicode_noncapturing_branch_assertion_sequence_work_clones_and_contracts() {
        let source = format!("{}.|\\n{}+", "(".repeat(3000), ")".repeat(3000));
        let matcher = plan(&source, true, false, true, false).clone();
        let input = JsString::from("\n".repeat(200000).as_str());
        let mut work = 0;
        assert_eq!(matcher.capture_count(), 3000);
        assert_eq!(
            matcher
                .find_with_captures_and_work(&input, 0, false, |n| {
                    work += n;
                    Ok::<(), ()>(())
                })
                .unwrap(),
            Some(RegExpUnicodeRepeatedAtomMatch {
                range: 0..200000,
                atom_capture: Some(199999..200000)
            })
        );
        assert!(work < 50 * input.len(), "{work}");

        for source in [
            r"(\b\b(?:a)|(?:b))+",
            r"(\b\b(?:a)|b)+",
            r"(a|(?:b)\b\b)+",
            r"(\B^\B(?:.)$$|\b\b(?:a)\b\b)+",
        ] {
            let source = JsString::from(source);
            assert!(
                RegExpUnicodeRepeatedAtomMatcher::compile_noncapturing_branch_assertions_with_work(
                    &source,
                    true,
                    false,
                    true,
                    true,
                    |_| Ok::<(), ()>(())
                )
                .unwrap()
                .is_none()
            );
            assert!(
                RegExpUnicodeRepeatedAtomMatcher::compile_noncapturing_branch_assertion_sequences_with_work(
                    &source,
                    true,
                    false,
                    true,
                    true,
                    |_| Ok::<(), ()>(())
                )
                .unwrap()
                .is_some()
            );
        }

        for source in [r"(\b\B(?:a)|b)+", r"((?:a)\b\B|b)+"] {
            let matcher = plan(source, true, false, false, false);
            assert_eq!(
                matcher
                    .find_with_work(&JsString::from("a"), 0, false, |_| Ok::<(), ()>(()))
                    .unwrap(),
                None
            );
            assert_eq!(
                matcher
                    .find_with_work(&JsString::from("b"), 0, false, |_| Ok::<(), ()>(()))
                    .unwrap(),
                Some(0..1)
            );
        }
        let source = r"(\b\B(?:a)|\B\b(?:b))*";
        let matcher = plan(source, true, false, false, false).clone();
        assert_eq!(
            matcher
                .find_with_captures_and_work(&JsString::from("ab"), 0, false, |_| Ok::<(), ()>(()))
                .unwrap(),
            Some(RegExpUnicodeRepeatedAtomMatch {
                range: 0..0,
                atom_capture: None
            })
        );
        assert_eq!(
            plan(r"(\b\B(?:a)|\B\b(?:b))+", true, false, false, false)
                .find_with_work(&JsString::from("ab"), 0, false, |_| Ok::<(), ()>(()))
                .unwrap(),
            None
        );
        assert!(
            matcher
                .find_with_work(&JsString::from("ab"), 0, false, |_| Err(()))
                .is_err()
        );
        let many = format!("({}|b)+", "a|".repeat(20000).trim_end_matches('|'));
        assert_eq!(
            plan(&many, false, false, false, false)
                .find_with_work(&JsString::from("b"), 0, true, |_| Ok::<(), ()>(()))
                .unwrap(),
            Some(0..1)
        );
        let source = JsString::from(r"(\B^\B(?:.)$$|\b\b(?:a)\b\b){1,3}$");
        let mut calls = 0;
        RegExpUnicodeRepeatedAtomMatcher::compile_noncapturing_branch_assertion_sequences_with_work(
            &source,
            true,
            false,
            true,
            true,
            |_| {
                calls += 1;
                Ok::<(), ()>(())
            },
        )
        .unwrap()
        .unwrap();
        for fail in 1..=calls {
            let mut n = 0;
            assert!(
                RegExpUnicodeRepeatedAtomMatcher::compile_noncapturing_branch_assertion_sequences_with_work(
                    &source,
                    true,
                    false,
                    true,
                    true,
                    |_| {
                        n += 1;
                        if n == fail { Err(()) } else { Ok(()) }
                    }
                )
                .is_err()
            );
        }
        let matcher = plan(
            r"(\B^\B(?:.)$$|\b\b(?:a)\b\b){1,3}$",
            true,
            false,
            true,
            true,
        );
        let input = JsString::from("\r\n");
        calls = 0;
        matcher
            .find_with_captures_and_work(&input, 0, false, |_| {
                calls += 1;
                Ok::<(), ()>(())
            })
            .unwrap();
        for fail in 1..=calls {
            let mut n = 0;
            assert!(
                matcher
                    .find_with_captures_and_work(&input, 0, false, |_| {
                        n += 1;
                        if n == fail { Err(()) } else { Ok(()) }
                    })
                    .is_err()
            );
        }

        let deep = format!("({}a{}|b)+", "(?:".repeat(6000), ")".repeat(6000));
        assert_eq!(plan(&deep, true, false, false, false).capture_count(), 1);
        assert_eq!(
            plan(&deep, true, false, false, false)
                .find_with_work(&JsString::from("aa"), 0, false, |_| Ok::<(), ()>(()))
                .unwrap(),
            Some(0..2)
        );

        let guarded = format!("(^{}a{}|b)+", "(?:".repeat(6000), ")".repeat(6000));
        assert_eq!(
            plan(&guarded, true, false, false, false)
                .find_with_work(&JsString::from("aa"), 0, false, |_| Ok::<(), ()>(()))
                .unwrap(),
            Some(0..1)
        );
        for source in [
            r"((?:a)|)+",
            r"(|(?:b))+",
            r"((?:a)|(?:ab))+",
            r"((?:a)|(?:b?))+",
            r"((?:a)|(b))+",
            r"((?:a)|(?:\b))+",
            r"((?:a)|(?i:b))+",
            r"((?:a)|(?:b))+c",
            r"\b((?:a)|(?:b))+",
            r"((?:a)|(?:b))+\b",
            r"(\b((?:a)|(?:b)))+",
            r"((a)|(?:b))+",
            r"((?:a)|(?:b)){2}c",
            r"((?:a|b)|(?:c))+",
            r"((?:a){1}|(?:b))+",
            r"((?:a{1})|(?:b))+",
            r"(\b(?:a)|(?:b))+c",
        ] {
            assert!(
                RegExpUnicodeRepeatedAtomMatcher::compile_noncapturing_branch_assertion_sequences_with_work(
                    &JsString::from(source),
                    true,
                    false,
                    true,
                    true,
                    |_| Ok::<(), ()>(())
                )
                .unwrap()
                .is_none(),
                "{source}"
            );
        }
        for source in [
            r"(^^(?:a)|(?:b)$$)+",
            r"((?:a)$$|^^(?:b))+",
            r"(\B^\B(?:.)$$|\b\b(?:a)\b\b)+",
        ] {
            assert!(
                RegExpUnicodeRepeatedAtomMatcher::compile_noncapturing_branches_with_work(
                    &JsString::from(source),
                    true,
                    false,
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
}

#[cfg(test)]
mod noncapturing_branch_sequence_word_tests {
    use super::*;
    use std::fmt::Write;

    type Case = (
        &'static str,
        u8,
        usize,
        Option<usize>,
        bool,
        usize,
        bool,
        bool,
        Option<bool>,
        Option<bool>,
    );
    const CASES: [Case; 16] = [
        (
            r"\b((?:a)|(?:b))+\B",
            0,
            1,
            None,
            true,
            1,
            false,
            false,
            Some(true),
            Some(false),
        ),
        (
            r"\B(((?:a)|(?:[ſK]))){0,3}?\b",
            1,
            0,
            Some(3),
            false,
            2,
            false,
            false,
            Some(false),
            Some(true),
        ),
        (
            r"\b(?:(?:.)|(?:\n)){2,4}\b",
            2,
            2,
            Some(4),
            true,
            0,
            false,
            false,
            Some(true),
            Some(true),
        ),
        (
            r"\B((?:\d)|(?:\D))*",
            3,
            0,
            None,
            true,
            1,
            false,
            false,
            Some(false),
            None,
        ),
        (
            r"^((?:\p{Join_C})|(?:\p{NChar}))+\B$",
            4,
            1,
            None,
            true,
            1,
            true,
            true,
            None,
            Some(false),
        ),
        (
            r"\b((?:😀)|(?:a)){1,3}\b$",
            5,
            1,
            Some(3),
            true,
            1,
            false,
            true,
            Some(true),
            Some(true),
        ),
        (
            r"((?:\uD800)|(?:\uDC00)){0,2}?\b",
            6,
            0,
            Some(2),
            false,
            1,
            false,
            false,
            None,
            Some(true),
        ),
        (
            r"\b((?:\|)|(?:a))+",
            7,
            1,
            None,
            true,
            1,
            false,
            false,
            Some(true),
            None,
        ),
        (
            r"\b(\b\b(?:a)|(?:b)\b\b)+\B",
            8,
            1,
            None,
            true,
            1,
            false,
            false,
            Some(true),
            Some(false),
        ),
        (
            r"(^^(?:a)|(?:b)$$){1,2}\b$",
            9,
            1,
            Some(2),
            true,
            1,
            false,
            true,
            None,
            Some(true),
        ),
        (
            r"\B((?:[a\|])|(?:b))??\B",
            10,
            0,
            Some(1),
            false,
            1,
            false,
            false,
            Some(false),
            Some(false),
        ),
        (
            r"\B(\B\B(?:.)|\b\b(?:[aſK])){1,3}\b",
            11,
            1,
            Some(3),
            true,
            1,
            false,
            false,
            Some(false),
            Some(true),
        ),
        (
            r"((?:a)$$|^^(?:b))+\b",
            12,
            1,
            None,
            true,
            1,
            false,
            false,
            None,
            Some(true),
        ),
        (
            r"^\b((?:[a-z])|(?:\p{Join_C})){0,2}\B$",
            13,
            0,
            Some(2),
            true,
            1,
            true,
            true,
            Some(true),
            Some(false),
        ),
        (
            r"((?:\u{61})|(?:\u{1f600}))?\b",
            14,
            0,
            Some(1),
            true,
            1,
            false,
            false,
            None,
            Some(true),
        ),
        (
            r"\B(\B^\B(?:.)$$|\b\b(?:a)\b\b){1,3}\B$",
            15,
            1,
            Some(3),
            true,
            1,
            false,
            true,
            Some(false),
            Some(false),
        ),
    ];
    fn plan(source: &str, v: bool, i: bool, m: bool, s: bool) -> RegExpUnicodeRepeatedAtomMatcher {
        RegExpUnicodeRepeatedAtomMatcher::compile_noncapturing_branch_sequence_words_with_work(
            &JsString::from(source),
            v,
            i,
            m,
            s,
            |_| Ok::<(), ()>(()),
        )
        .unwrap()
        .unwrap()
    }

    #[test]
    fn unicode_noncapturing_branch_sequence_word_ranges_snapshot() {
        let mut rows = String::new();
        for (source, ..) in CASES {
            let pattern = JsString::from(source);
            for v in [false, true] {
                for i in [false, true] {
                    for m in [false, true] {
                        for s in [false, true] {
                            let matcher = plan(source, v, i, m, s);
                            for input in [
                                JsString::from("Aaaa\nAA"),
                                JsString::from("A\r\n\u{200c}\u{200d}\u{ffff}\u{10ffff}"),
                                JsString::from("😀??aaſK"),
                                JsString::from_code_units(vec![0x61, 0xd800, 0x61, 0xdc00]),
                                JsString::from(""),
                            ] {
                                for start in [0, 1, input.len(), input.len() + 1] {
                                    for sticky in [false, true] {
                                        let found = matcher
                                            .find_with_captures_and_work(
                                                &input,
                                                start,
                                                sticky,
                                                |_| Ok::<(), ()>(()),
                                            )
                                            .unwrap();
                                        writeln!(rows,"{pattern:?} v={v} i={i} m={m} s={s} input={input:?} start={start} sticky={sticky} count={} range={:?} capture={:?}",matcher.capture_count(),found.as_ref().map(|f|f.range.clone()),found.and_then(|f|f.atom_capture)).unwrap();
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        insta::assert_snapshot!(rows);
    }

    #[test]
    fn unicode_repeated_atom_word_alternatives_agree_with_independent_counts_and_final_character() {
        let alphabet: [u16; 10] = [
            0x41, 0x61, 10, 13, 0x17f, 0x212a, 0xd83d, 0xde00, 0xd800, 0xdc00,
        ];
        let lt = |c| [10, 13, 0x2028, 0x2029].contains(&c);
        for i in [false, true] {
            for m in [false, true] {
                for s in [false, true] {
                    let plans: Vec<_> = CASES.iter().map(|c| plan(c.0, true, i, m, s)).collect();
                    let fold = |c| {
                        if i {
                            match c {
                                0x41 => 0x61,
                                0x17f => 0x73,
                                0x212a => 0x6b,
                                _ => c,
                            }
                        } else {
                            c
                        }
                    };
                    for len in 0..=4 {
                        for ordinal in 0..alphabet.len().pow(len) {
                            let units: Vec<_> = (0..len)
                                .map(|j| alphabet[ordinal / alphabet.len().pow(j) % alphabet.len()])
                                .collect();
                            let mut points = Vec::new();
                            let mut bounds = vec![0];
                            let mut j = 0;
                            while j < units.len() {
                                let mut c = u32::from(units[j]);
                                j += 1;
                                if (0xd800..=0xdbff).contains(&c)
                                    && j < units.len()
                                    && (0xdc00..=0xdfff).contains(&units[j])
                                {
                                    c = 65536 + (c - 0xd800) * 1024 + u32::from(units[j]) - 0xdc00;
                                    j += 1;
                                }
                                points.push(c);
                                bounds.push(j);
                            }
                            let input = JsString::from_code_units(units);
                            for start in 0..=input.len() + 1 {
                                for sticky in [false, true] {
                                    for (case, matcher) in CASES.iter().zip(&plans) {
                                        let expected = if start > input.len() {
                                            None
                                        } else {
                                            let first = bounds.partition_point(|&b| b <= start) - 1;
                                            (first..=points.len()).find_map(|at| {
                                                let word=|c:Option<&u32>|c.is_some_and(|&c|matches!(c,0x30..=0x39|0x41..=0x5a|0x5f|0x61..=0x7a)||(i&&matches!(c,0x17f|0x212a)));
                                                if (sticky && at != first)
                                                    || (case.6
                                                        && at != 0
                                                        && !(m && lt(points[at - 1])))
                                                {
                                                    return None;
                                                }
                                                if case.8.is_some_and(|positive| (word(at.checked_sub(1).and_then(|n|points.get(n))) != word(points.get(at))) != positive) { return None; }
                                                let available = (at..points.len()).take_while(|&p| {
                                                    let c = points[p];
                                                    let prev = p.checked_sub(1).and_then(|n| points.get(n));
                                                    let next = points.get(p+1);
                                                    let wb = word(prev) != word(Some(&c));
                                                    let we = word(Some(&c)) != word(next);
                                                    let begin = p == 0 || (m && prev.is_some_and(|&c| lt(c)));
                                                    let end = next.is_none() || (m && next.is_some_and(|&c| lt(c)));
                                                    let dot = s || !lt(c);
                                                    let class = [0x61, fold(0x17f), fold(0x212a)].contains(&fold(c));
                                                    match case.1 {
                                                        0 => [0x61,0x62].contains(&fold(c)),
                                                        1 => class,
                                                        2 => dot || c == 10,
                                                        3 => true,
                                                        4 => (0x200c..=0x200d).contains(&c) || matches!(c,0xfdd0..=0xfdef) || c&0xffff >= 0xfffe,
                                                        5 | 14 => c == 0x1f600 || fold(c) == 0x61,
                                                        6 => [0xd800,0xdc00].contains(&c),
                                                        7 => c == 0x7c || fold(c) == 0x61,
                                                        8 => (fold(c) == 0x61 && wb) || (fold(c) == 0x62 && we),
                                                        9 => (fold(c) == 0x61 && begin) || (fold(c) == 0x62 && end),
                                                        10 => [0x61,0x62,0x7c].contains(&fold(c)),
                                                        11 => (dot && !wb) || (class && wb),
                                                        12 => (fold(c) == 0x61 && end) || (fold(c) == 0x62 && begin),
                                                        13 => (0x61..=0x7a).contains(&fold(c)) || (0x200c..=0x200d).contains(&c),
                                                        15 => (dot && begin && end && !wb) || (fold(c) == 0x61 && wb && we),
                                                        _ => unreachable!(),
                                                    }
                                                }).count();
                                                let upper =
                                                    available.min(case.3.unwrap_or(available));
                                                if upper < case.2 {
                                                    return None;
                                                }
                                                let endpoint = |n: usize| {
                                                    let end = at + n;
                                                    if case.9.is_some_and(|positive| (word(end.checked_sub(1).and_then(|n|points.get(n))) != word(points.get(end))) != positive) { return None; }

                                                    if case.7
                                                        && end != points.len()
                                                        && !(m && lt(points[end]))
                                                    {
                                                        return None;
                                                    }
                                                    Some(RegExpUnicodeRepeatedAtomMatch {
                                                        range: bounds[at]..bounds[end],
                                                        atom_capture: (case.5 != 0 && n != 0)
                                                            .then(|| bounds[end - 1]..bounds[end]),
                                                    })
                                                };
                                                let mut counts = case.2..=upper;
                                                if case.4 {
                                                    counts.rev().find_map(endpoint)
                                                } else {
                                                    counts.find_map(endpoint)
                                                }
                                            })
                                        };
                                        assert_eq!(matcher.capture_count(), case.5, "{}", case.0);
                                        assert_eq!(
                                            matcher
                                                .find_with_captures_and_work(
                                                    &input,
                                                    start,
                                                    sticky,
                                                    |_| Ok::<(), ()>(())
                                                )
                                                .unwrap(),
                                            expected,
                                            "{} i={i} m={m} s={s} {input:?} start={start} sticky={sticky}",
                                            case.0
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

    #[test]
    fn unicode_noncapturing_branch_sequence_word_work_and_contracts() {
        let source = format!(
            r"\B{}(?:.)|(?:\n){}+\B$",
            "(".repeat(3000),
            ")".repeat(3000)
        );
        let matcher = plan(&source, true, false, true, false).clone();
        let input = JsString::from("\n".repeat(200000).as_str());
        let mut work = 0;
        assert_eq!(matcher.capture_count(), 3000);
        assert_eq!(
            matcher
                .find_with_captures_and_work(&input, 0, false, |n| {
                    work += n;
                    Ok::<(), ()>(())
                })
                .unwrap(),
            Some(RegExpUnicodeRepeatedAtomMatch {
                range: 0..200000,
                atom_capture: Some(199999..200000)
            })
        );
        assert!(work < 60 * input.len(), "{work}");
        let source = JsString::from(r"\B(\B^\B(?:.)$$|\b\b(?:a)\b\b){1,3}\B$");

        for source in [
            r"\b(\b\b(?:a)|b)+\b",
            r"\b(a|(?:b)\b\b)+",
            r"\B(\B^\B(?:.)$$|\b\b(?:a)\b\b)+\B",
        ] {
            let source = JsString::from(source);
            assert!(
                RegExpUnicodeRepeatedAtomMatcher::compile_noncapturing_branch_scoped_assertions_with_work(
                    &source,
                    true,
                    false,
                    true,
                    true,
                    |_| Ok::<(), ()>(())
                )
                .unwrap()
                .is_none()
            );
            assert!(RegExpUnicodeRepeatedAtomMatcher::compile_noncapturing_branch_sequence_words_with_work(
                &source, true, false, true, true, |_| Ok::<(), ()>(())
            ).unwrap().is_some());
        }
        let mut calls = 0;
        RegExpUnicodeRepeatedAtomMatcher::compile_noncapturing_branch_sequence_words_with_work(
            &source,
            true,
            false,
            true,
            true,
            |_| {
                calls += 1;
                Ok::<(), ()>(())
            },
        )
        .unwrap()
        .unwrap();
        for fail in 1..=calls {
            let mut n = 0;
            assert!(
                RegExpUnicodeRepeatedAtomMatcher::compile_noncapturing_branch_sequence_words_with_work(
                    &source,
                    true,
                    false,
                    true,
                    true,
                    |_| {
                        n += 1;
                        if n == fail { Err(()) } else { Ok(()) }
                    }
                )
                .is_err()
            );
        }
        let input = JsString::from("\r\n");
        let matcher = plan(
            r"\B(\B^\B(?:.)$$|\b\b(?:a)\b\b){1,3}\B$",
            true,
            false,
            true,
            true,
        );
        calls = 0;
        matcher
            .find_with_captures_and_work(&input, 0, false, |_| {
                calls += 1;
                Ok::<(), ()>(())
            })
            .unwrap();
        for fail in 1..=calls {
            let mut n = 0;
            assert!(
                matcher
                    .find_with_captures_and_work(&input, 0, false, |_| {
                        n += 1;
                        if n == fail { Err(()) } else { Ok(()) }
                    })
                    .is_err()
            );
        }
        for source in [
            r"\b(a|)+",
            r"\b(|b)+",
            r"\b(a|ab)+",
            r"\b((?:a)|(?:b)?)+",
            r"\b(a|(b))+",
            r"\B(a|\b)+",
            r"\b(a|(?i:b))+",
            r"\b((?:a)|(?:b))+c",
            r"\b(\b((?:a)|(?:b)))+",
            r"\b((a)|(b))+",
            r"\b((?:a)|(?:b)){2}c",
        ] {
            assert!(
                RegExpUnicodeRepeatedAtomMatcher::compile_noncapturing_branch_sequence_words_with_work(
                    &JsString::from(source),
                    true,
                    false,
                    true,
                    true,
                    |_| Ok::<(), ()>(())
                )
                .unwrap()
                .is_none(),
                "{source}"
            );
        }
        for source in [
            r"\b((?:a)|(?:b))+",
            r"(^^(?:a)|(?:b)$$)+\b",
            r"\b(\b\b(?:a)|(?:b)\b\b)+\B",
        ] {
            assert!(
                RegExpUnicodeRepeatedAtomMatcher::compile_noncapturing_branch_lines_with_work(
                    &JsString::from(source),
                    true,
                    false,
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
}

#[cfg(test)]
mod noncapturing_branch_nested_assertion_tests {
    use super::*;
    use std::fmt::Write;

    type Case = (
        &'static str,
        u8,
        usize,
        Option<usize>,
        bool,
        usize,
        bool,
        bool,
    );
    const CASES: [Case; 16] = [
        (r"((?:a)|(?:b))+", 0, 1, None, true, 1, false, false),
        (
            r"(((?:a)|(?:[ſK]))){0,3}?",
            1,
            0,
            Some(3),
            false,
            2,
            false,
            false,
        ),
        (
            r"(?:(?:.)|(?:\n)){2,4}",
            2,
            2,
            Some(4),
            true,
            0,
            false,
            false,
        ),
        (r"((?:\d)|(?:\D))*", 3, 0, None, true, 1, false, false),
        (
            r"^((?:\p{Join_C})|(?:\p{NChar}))+$",
            4,
            1,
            None,
            true,
            1,
            true,
            true,
        ),
        (r"((?:😀)|(?:a)){1,3}$", 5, 1, Some(3), true, 1, false, true),
        (
            r"((?:\uD800)|(?:\uDC00)){0,2}?",
            6,
            0,
            Some(2),
            false,
            1,
            false,
            false,
        ),
        (r"((?:\|)|(?:a))+", 7, 1, None, true, 1, false, false),
        (
            r"((?:\b(?:\ba))|(?:(?:b\b)\b))+",
            8,
            1,
            None,
            true,
            1,
            false,
            false,
        ),
        (
            r"((?:^(?:^a))|(?:(?:b$)$)){1,2}$",
            9,
            1,
            Some(2),
            true,
            1,
            false,
            true,
        ),
        (
            r"((?:[a\|])|(?:b))??",
            10,
            0,
            Some(1),
            false,
            1,
            false,
            false,
        ),
        (
            r"((?:\B(?:\B.))|(?:\b(?:\b[aſK]))){1,3}",
            11,
            1,
            Some(3),
            true,
            1,
            false,
            false,
        ),
        (
            r"((?:(?:a$)$)|(?:^(?:^b)))+",
            12,
            1,
            None,
            true,
            1,
            false,
            false,
        ),
        (
            r"^((?:[a-z])|(?:\p{Join_C})){0,2}$",
            13,
            0,
            Some(2),
            true,
            1,
            true,
            true,
        ),
        (
            r"((?:\u{61})|(?:\u{1f600}))?",
            14,
            0,
            Some(1),
            true,
            1,
            false,
            false,
        ),
        (
            r"((?:\B^(?:\B.$)$)|(?:\b(?:\ba\b)\b)){1,3}$",
            15,
            1,
            Some(3),
            true,
            1,
            false,
            true,
        ),
    ];
    fn plan(source: &str, v: bool, i: bool, m: bool, s: bool) -> RegExpUnicodeRepeatedAtomMatcher {
        RegExpUnicodeRepeatedAtomMatcher::compile_noncapturing_branch_nested_assertions_with_work(
            &JsString::from(source),
            v,
            i,
            m,
            s,
            |_| Ok::<(), ()>(()),
        )
        .unwrap()
        .unwrap_or_else(|| panic!("Unproved line-branch source: {source}"))
    }

    #[test]
    fn unicode_noncapturing_branch_nested_assertion_ranges_snapshot() {
        let mut rows = String::new();
        for (source, ..) in CASES {
            let pattern = JsString::from(source);
            for v in [false, true] {
                for i in [false, true] {
                    for m in [false, true] {
                        for s in [false, true] {
                            let matcher = plan(source, v, i, m, s);
                            for input in [
                                JsString::from("Aaaa\nAA"),
                                JsString::from("A\r\n\u{200c}\u{200d}\u{ffff}\u{10ffff}"),
                                JsString::from("😀??aaſK"),
                                JsString::from_code_units(vec![0x61, 0xd800, 0x61, 0xdc00]),
                                JsString::from(""),
                            ] {
                                for start in [0, 1, input.len(), input.len() + 1] {
                                    for sticky in [false, true] {
                                        let found = matcher
                                            .find_with_captures_and_work(
                                                &input,
                                                start,
                                                sticky,
                                                |_| Ok::<(), ()>(()),
                                            )
                                            .unwrap();
                                        writeln!(rows,"{pattern:?} v={v} i={i} m={m} s={s} input={input:?} start={start} sticky={sticky} count={} range={:?} capture={:?}",matcher.capture_count(),found.as_ref().map(|f|f.range.clone()),found.and_then(|f|f.atom_capture)).unwrap();
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        insta::assert_snapshot!(rows);
    }

    #[test]
    fn unicode_repeated_atom_alternatives_agree_with_independent_counts_and_final_character() {
        let alphabet: [u16; 10] = [
            0x41, 0x61, 10, 13, 0x17f, 0x212a, 0xd83d, 0xde00, 0xd800, 0xdc00,
        ];
        let lt = |c| [10, 13, 0x2028, 0x2029].contains(&c);
        for i in [false, true] {
            for m in [false, true] {
                for s in [false, true] {
                    let plans: Vec<_> = CASES.iter().map(|c| plan(c.0, true, i, m, s)).collect();
                    let fold = |c| {
                        if i {
                            match c {
                                0x41 => 0x61,
                                0x17f => 0x73,
                                0x212a => 0x6b,
                                _ => c,
                            }
                        } else {
                            c
                        }
                    };
                    for len in 0..=4 {
                        for ordinal in 0..alphabet.len().pow(len) {
                            let units: Vec<_> = (0..len)
                                .map(|j| alphabet[ordinal / alphabet.len().pow(j) % alphabet.len()])
                                .collect();
                            let mut points = Vec::new();
                            let mut bounds = vec![0];
                            let mut j = 0;
                            while j < units.len() {
                                let mut c = u32::from(units[j]);
                                j += 1;
                                if (0xd800..=0xdbff).contains(&c)
                                    && j < units.len()
                                    && (0xdc00..=0xdfff).contains(&units[j])
                                {
                                    c = 65536 + (c - 0xd800) * 1024 + u32::from(units[j]) - 0xdc00;
                                    j += 1;
                                }
                                points.push(c);
                                bounds.push(j);
                            }
                            let input = JsString::from_code_units(units);
                            for start in 0..=input.len() + 1 {
                                for sticky in [false, true] {
                                    for (case, matcher) in CASES.iter().zip(&plans) {
                                        let expected = if start > input.len() {
                                            None
                                        } else {
                                            let first = bounds.partition_point(|&b| b <= start) - 1;
                                            (first..=points.len()).find_map(|at| {
                                                let word=|c:Option<&u32>|c.is_some_and(|&c|matches!(c,0x30..=0x39|0x41..=0x5a|0x5f|0x61..=0x7a)||(i&&matches!(c,0x17f|0x212a)));
                                                if (sticky && at != first)
                                                    || (case.6
                                                        && at != 0
                                                        && !(m && lt(points[at - 1])))
                                                {
                                                    return None;
                                                }
                                                let available = (at..points.len()).take_while(|&p| {
                                                    let c = points[p];
                                                    let prev = p.checked_sub(1).and_then(|n| points.get(n));
                                                    let next = points.get(p+1);
                                                    let wb = word(prev) != word(Some(&c));
                                                    let we = word(Some(&c)) != word(next);
                                                    let begin = p == 0 || (m && prev.is_some_and(|&c| lt(c)));
                                                    let end = next.is_none() || (m && next.is_some_and(|&c| lt(c)));
                                                    let dot = s || !lt(c);
                                                    let class = [0x61, fold(0x17f), fold(0x212a)].contains(&fold(c));
                                                    match case.1 {
                                                        0 => [0x61,0x62].contains(&fold(c)),
                                                        1 => class,
                                                        2 => dot || c == 10,
                                                        3 => true,
                                                        4 => (0x200c..=0x200d).contains(&c) || matches!(c,0xfdd0..=0xfdef) || c&0xffff >= 0xfffe,
                                                        5 | 14 => c == 0x1f600 || fold(c) == 0x61,
                                                        6 => [0xd800,0xdc00].contains(&c),
                                                        7 => c == 0x7c || fold(c) == 0x61,
                                                        8 => (fold(c) == 0x61 && wb) || (fold(c) == 0x62 && we),
                                                        9 => (fold(c) == 0x61 && begin) || (fold(c) == 0x62 && end),
                                                        10 => [0x61,0x62,0x7c].contains(&fold(c)),
                                                        11 => (dot && !wb) || (class && wb),
                                                        12 => (fold(c) == 0x61 && end) || (fold(c) == 0x62 && begin),
                                                        13 => (0x61..=0x7a).contains(&fold(c)) || (0x200c..=0x200d).contains(&c),
                                                        15 => (dot && begin && end && !wb) || (fold(c) == 0x61 && wb && we),
                                                        _ => unreachable!(),
                                                    }
                                                }).count();
                                                let upper =
                                                    available.min(case.3.unwrap_or(available));
                                                if upper < case.2 {
                                                    return None;
                                                }
                                                let endpoint = |n: usize| {
                                                    let end = at + n;
                                                    if case.7
                                                        && end != points.len()
                                                        && !(m && lt(points[end]))
                                                    {
                                                        return None;
                                                    }
                                                    Some(RegExpUnicodeRepeatedAtomMatch {
                                                        range: bounds[at]..bounds[end],
                                                        atom_capture: (case.5 != 0 && n != 0)
                                                            .then(|| bounds[end - 1]..bounds[end]),
                                                    })
                                                };
                                                let mut counts = case.2..=upper;
                                                if case.4 {
                                                    counts.rev().find_map(endpoint)
                                                } else {
                                                    counts.find_map(endpoint)
                                                }
                                            })
                                        };
                                        assert_eq!(matcher.capture_count(), case.5, "{}", case.0);
                                        assert_eq!(
                                            matcher
                                                .find_with_captures_and_work(
                                                    &input,
                                                    start,
                                                    sticky,
                                                    |_| Ok::<(), ()>(())
                                                )
                                                .unwrap(),
                                            expected,
                                            "{} i={i} m={m} s={s} {input:?} start={start} sticky={sticky}",
                                            case.0
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

    #[test]
    fn unicode_noncapturing_branch_nested_assertion_work_clones_and_contracts() {
        let source = format!("{}.|\\n{}+", "(".repeat(3000), ")".repeat(3000));
        let matcher = plan(&source, true, false, true, false).clone();
        let input = JsString::from("\n".repeat(200000).as_str());
        let mut work = 0;
        assert_eq!(matcher.capture_count(), 3000);
        assert_eq!(
            matcher
                .find_with_captures_and_work(&input, 0, false, |n| {
                    work += n;
                    Ok::<(), ()>(())
                })
                .unwrap(),
            Some(RegExpUnicodeRepeatedAtomMatch {
                range: 0..200000,
                atom_capture: Some(199999..200000)
            })
        );
        assert!(work < 50 * input.len(), "{work}");

        for source in [
            r"((?:\b(?:\ba))|(?:b))+",
            r"((?:\b(?:\ba))|b)+",
            r"(a|(?:(?:b\b)\b))+",
            r"((?:\B^(?:\B.$)$)|(?:\b(?:\ba\b)\b))+",
        ] {
            let source = JsString::from(source);
            assert!(
                RegExpUnicodeRepeatedAtomMatcher::compile_noncapturing_branch_assertion_sequences_with_work(
                    &source,
                    true,
                    false,
                    true,
                    true,
                    |_| Ok::<(), ()>(())
                )
                .unwrap()
                .is_none()
            );
            assert!(
                RegExpUnicodeRepeatedAtomMatcher::compile_noncapturing_branch_nested_assertions_with_work(
                    &source,
                    true,
                    false,
                    true,
                    true,
                    |_| Ok::<(), ()>(())
                )
                .unwrap()
                .is_some()
            );
        }

        for source in [r"((?:\b(?:\Ba))|b)+", r"((?:(?:a\b)\B)|b)+"] {
            let matcher = plan(source, true, false, false, false);
            assert_eq!(
                matcher
                    .find_with_work(&JsString::from("a"), 0, false, |_| Ok::<(), ()>(()))
                    .unwrap(),
                None
            );
            assert_eq!(
                matcher
                    .find_with_work(&JsString::from("b"), 0, false, |_| Ok::<(), ()>(()))
                    .unwrap(),
                Some(0..1)
            );
        }
        let source = r"((?:\b(?:\Ba))|(?:\B(?:\bb)))*";
        let matcher = plan(source, true, false, false, false).clone();
        assert_eq!(
            matcher
                .find_with_captures_and_work(&JsString::from("ab"), 0, false, |_| Ok::<(), ()>(()))
                .unwrap(),
            Some(RegExpUnicodeRepeatedAtomMatch {
                range: 0..0,
                atom_capture: None
            })
        );
        assert_eq!(
            plan(r"((?:\b(?:\Ba))|(?:\B(?:\bb)))+", true, false, false, false)
                .find_with_work(&JsString::from("ab"), 0, false, |_| Ok::<(), ()>(()))
                .unwrap(),
            None
        );
        assert!(
            matcher
                .find_with_work(&JsString::from("ab"), 0, false, |_| Err(()))
                .is_err()
        );

        for source in [
            r"((?:\b(?:a)c)|b)+",
            r"((?:\b(a))|b)+",
            r"((?:\b(?:a|b))|c)+",
            r"((?:\b(?:a)? )|b)+",
        ] {
            assert!(RegExpUnicodeRepeatedAtomMatcher::compile_noncapturing_branch_nested_assertions_with_work(
                &JsString::from(source), true, false, false, false, |_| Ok::<(), ()>(())
            ).unwrap().is_none(), "{source}");
        }
        let many = format!("({}|b)+", "a|".repeat(20000).trim_end_matches('|'));
        assert_eq!(
            plan(&many, false, false, false, false)
                .find_with_work(&JsString::from("b"), 0, true, |_| Ok::<(), ()>(()))
                .unwrap(),
            Some(0..1)
        );
        let source = JsString::from(r"((?:\B^(?:\B.$)$)|(?:\b(?:\ba\b)\b)){1,3}$");
        let mut calls = 0;
        RegExpUnicodeRepeatedAtomMatcher::compile_noncapturing_branch_nested_assertions_with_work(
            &source,
            true,
            false,
            true,
            true,
            |_| {
                calls += 1;
                Ok::<(), ()>(())
            },
        )
        .unwrap()
        .unwrap();
        for fail in 1..=calls {
            let mut n = 0;
            assert!(
                RegExpUnicodeRepeatedAtomMatcher::compile_noncapturing_branch_nested_assertions_with_work(
                    &source,
                    true,
                    false,
                    true,
                    true,
                    |_| {
                        n += 1;
                        if n == fail { Err(()) } else { Ok(()) }
                    }
                )
                .is_err()
            );
        }
        let matcher = plan(
            r"((?:\B^(?:\B.$)$)|(?:\b(?:\ba\b)\b)){1,3}$",
            true,
            false,
            true,
            true,
        );
        let input = JsString::from("\r\n");
        calls = 0;
        matcher
            .find_with_captures_and_work(&input, 0, false, |_| {
                calls += 1;
                Ok::<(), ()>(())
            })
            .unwrap();
        for fail in 1..=calls {
            let mut n = 0;
            assert!(
                matcher
                    .find_with_captures_and_work(&input, 0, false, |_| {
                        n += 1;
                        if n == fail { Err(()) } else { Ok(()) }
                    })
                    .is_err()
            );
        }

        let deep = format!("({}a{}|b)+", "(?:".repeat(6000), ")".repeat(6000));
        assert_eq!(plan(&deep, true, false, false, false).capture_count(), 1);
        assert_eq!(
            plan(&deep, true, false, false, false)
                .find_with_work(&JsString::from("aa"), 0, false, |_| Ok::<(), ()>(()))
                .unwrap(),
            Some(0..2)
        );

        let guarded = format!("(^{}a{}|b)+", "(?:".repeat(6000), ")".repeat(6000));
        assert_eq!(
            plan(&guarded, true, false, false, false)
                .find_with_work(&JsString::from("aa"), 0, false, |_| Ok::<(), ()>(()))
                .unwrap(),
            Some(0..1)
        );
        for source in [
            r"((?:a)|)+",
            r"(|(?:b))+",
            r"((?:a)|(?:ab))+",
            r"((?:a)|(?:b?))+",
            r"((?:a)|(b))+",
            r"((?:a)|(?:\b))+",
            r"((?:a)|(?i:b))+",
            r"((?:a)|(?:b))+c",
            r"\b((?:a)|(?:b))+",
            r"((?:a)|(?:b))+\b",
            r"(\b((?:a)|(?:b)))+",
            r"((a)|(?:b))+",
            r"((?:a)|(?:b)){2}c",
            r"((?:a|b)|(?:c))+",
            r"((?:a){1}|(?:b))+",
            r"((?:a{1})|(?:b))+",
            r"(\b(?:a)|(?:b))+c",
        ] {
            assert!(
                RegExpUnicodeRepeatedAtomMatcher::compile_noncapturing_branch_nested_assertions_with_work(
                    &JsString::from(source),
                    true,
                    false,
                    true,
                    true,
                    |_| Ok::<(), ()>(())
                )
                .unwrap()
                .is_none(),
                "{source}"
            );
        }
        for source in [
            r"((?:^(?:^a))|(?:(?:b$)$))+",
            r"((?:(?:a$)$)|(?:^(?:^b)))+",
            r"((?:\B^(?:\B.$)$)|(?:\b(?:\ba\b)\b))+",
        ] {
            assert!(
                RegExpUnicodeRepeatedAtomMatcher::compile_noncapturing_branches_with_work(
                    &JsString::from(source),
                    true,
                    false,
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
}

#[cfg(test)]
mod noncapturing_branch_nested_assertion_word_tests {
    use super::*;
    use std::fmt::Write;

    type Case = (
        &'static str,
        u8,
        usize,
        Option<usize>,
        bool,
        usize,
        bool,
        bool,
        Option<bool>,
        Option<bool>,
    );
    const CASES: [Case; 16] = [
        (
            r"\b((?:a)|(?:b))+\B",
            0,
            1,
            None,
            true,
            1,
            false,
            false,
            Some(true),
            Some(false),
        ),
        (
            r"\B(((?:a)|(?:[ſK]))){0,3}?\b",
            1,
            0,
            Some(3),
            false,
            2,
            false,
            false,
            Some(false),
            Some(true),
        ),
        (
            r"\b(?:(?:.)|(?:\n)){2,4}\b",
            2,
            2,
            Some(4),
            true,
            0,
            false,
            false,
            Some(true),
            Some(true),
        ),
        (
            r"\B((?:\d)|(?:\D))*",
            3,
            0,
            None,
            true,
            1,
            false,
            false,
            Some(false),
            None,
        ),
        (
            r"^((?:\p{Join_C})|(?:\p{NChar}))+\B$",
            4,
            1,
            None,
            true,
            1,
            true,
            true,
            None,
            Some(false),
        ),
        (
            r"\b((?:😀)|(?:a)){1,3}\b$",
            5,
            1,
            Some(3),
            true,
            1,
            false,
            true,
            Some(true),
            Some(true),
        ),
        (
            r"((?:\uD800)|(?:\uDC00)){0,2}?\b",
            6,
            0,
            Some(2),
            false,
            1,
            false,
            false,
            None,
            Some(true),
        ),
        (
            r"\b((?:\|)|(?:a))+",
            7,
            1,
            None,
            true,
            1,
            false,
            false,
            Some(true),
            None,
        ),
        (
            r"\b((?:\b(?:\ba))|(?:(?:b\b)\b))+\B",
            8,
            1,
            None,
            true,
            1,
            false,
            false,
            Some(true),
            Some(false),
        ),
        (
            r"((?:^(?:^a))|(?:(?:b$)$)){1,2}\b$",
            9,
            1,
            Some(2),
            true,
            1,
            false,
            true,
            None,
            Some(true),
        ),
        (
            r"\B((?:[a\|])|(?:b))??\B",
            10,
            0,
            Some(1),
            false,
            1,
            false,
            false,
            Some(false),
            Some(false),
        ),
        (
            r"\B((?:\B(?:\B.))|(?:\b(?:\b[aſK]))){1,3}\b",
            11,
            1,
            Some(3),
            true,
            1,
            false,
            false,
            Some(false),
            Some(true),
        ),
        (
            r"((?:(?:a$)$)|(?:^(?:^b)))+\b",
            12,
            1,
            None,
            true,
            1,
            false,
            false,
            None,
            Some(true),
        ),
        (
            r"^\b((?:[a-z])|(?:\p{Join_C})){0,2}\B$",
            13,
            0,
            Some(2),
            true,
            1,
            true,
            true,
            Some(true),
            Some(false),
        ),
        (
            r"((?:\u{61})|(?:\u{1f600}))?\b",
            14,
            0,
            Some(1),
            true,
            1,
            false,
            false,
            None,
            Some(true),
        ),
        (
            r"\B((?:\B^(?:\B.$)$)|(?:\b(?:\ba\b)\b)){1,3}\B$",
            15,
            1,
            Some(3),
            true,
            1,
            false,
            true,
            Some(false),
            Some(false),
        ),
    ];
    fn plan(source: &str, v: bool, i: bool, m: bool, s: bool) -> RegExpUnicodeRepeatedAtomMatcher {
        RegExpUnicodeRepeatedAtomMatcher::compile_noncapturing_branch_nested_assertion_words_with_work(
            &JsString::from(source),
            v,
            i,
            m,
            s,
            |_| Ok::<(), ()>(()),
        )
        .unwrap()
        .unwrap()
    }

    #[test]
    fn unicode_noncapturing_branch_nested_assertion_word_ranges_snapshot() {
        let mut rows = String::new();
        for (source, ..) in CASES {
            let pattern = JsString::from(source);
            for v in [false, true] {
                for i in [false, true] {
                    for m in [false, true] {
                        for s in [false, true] {
                            let matcher = plan(source, v, i, m, s);
                            for input in [
                                JsString::from("Aaaa\nAA"),
                                JsString::from("A\r\n\u{200c}\u{200d}\u{ffff}\u{10ffff}"),
                                JsString::from("😀??aaſK"),
                                JsString::from_code_units(vec![0x61, 0xd800, 0x61, 0xdc00]),
                                JsString::from(""),
                            ] {
                                for start in [0, 1, input.len(), input.len() + 1] {
                                    for sticky in [false, true] {
                                        let found = matcher
                                            .find_with_captures_and_work(
                                                &input,
                                                start,
                                                sticky,
                                                |_| Ok::<(), ()>(()),
                                            )
                                            .unwrap();
                                        writeln!(rows,"{pattern:?} v={v} i={i} m={m} s={s} input={input:?} start={start} sticky={sticky} count={} range={:?} capture={:?}",matcher.capture_count(),found.as_ref().map(|f|f.range.clone()),found.and_then(|f|f.atom_capture)).unwrap();
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        insta::assert_snapshot!(rows);
    }

    #[test]
    fn unicode_repeated_atom_word_alternatives_agree_with_independent_counts_and_final_character() {
        let alphabet: [u16; 10] = [
            0x41, 0x61, 10, 13, 0x17f, 0x212a, 0xd83d, 0xde00, 0xd800, 0xdc00,
        ];
        let lt = |c| [10, 13, 0x2028, 0x2029].contains(&c);
        for i in [false, true] {
            for m in [false, true] {
                for s in [false, true] {
                    let plans: Vec<_> = CASES.iter().map(|c| plan(c.0, true, i, m, s)).collect();
                    let fold = |c| {
                        if i {
                            match c {
                                0x41 => 0x61,
                                0x17f => 0x73,
                                0x212a => 0x6b,
                                _ => c,
                            }
                        } else {
                            c
                        }
                    };
                    for len in 0..=4 {
                        for ordinal in 0..alphabet.len().pow(len) {
                            let units: Vec<_> = (0..len)
                                .map(|j| alphabet[ordinal / alphabet.len().pow(j) % alphabet.len()])
                                .collect();
                            let mut points = Vec::new();
                            let mut bounds = vec![0];
                            let mut j = 0;
                            while j < units.len() {
                                let mut c = u32::from(units[j]);
                                j += 1;
                                if (0xd800..=0xdbff).contains(&c)
                                    && j < units.len()
                                    && (0xdc00..=0xdfff).contains(&units[j])
                                {
                                    c = 65536 + (c - 0xd800) * 1024 + u32::from(units[j]) - 0xdc00;
                                    j += 1;
                                }
                                points.push(c);
                                bounds.push(j);
                            }
                            let input = JsString::from_code_units(units);
                            for start in 0..=input.len() + 1 {
                                for sticky in [false, true] {
                                    for (case, matcher) in CASES.iter().zip(&plans) {
                                        let expected = if start > input.len() {
                                            None
                                        } else {
                                            let first = bounds.partition_point(|&b| b <= start) - 1;
                                            (first..=points.len()).find_map(|at| {
                                                let word=|c:Option<&u32>|c.is_some_and(|&c|matches!(c,0x30..=0x39|0x41..=0x5a|0x5f|0x61..=0x7a)||(i&&matches!(c,0x17f|0x212a)));
                                                if (sticky && at != first)
                                                    || (case.6
                                                        && at != 0
                                                        && !(m && lt(points[at - 1])))
                                                {
                                                    return None;
                                                }
                                                if case.8.is_some_and(|positive| (word(at.checked_sub(1).and_then(|n|points.get(n))) != word(points.get(at))) != positive) { return None; }
                                                let available = (at..points.len()).take_while(|&p| {
                                                    let c = points[p];
                                                    let prev = p.checked_sub(1).and_then(|n| points.get(n));
                                                    let next = points.get(p+1);
                                                    let wb = word(prev) != word(Some(&c));
                                                    let we = word(Some(&c)) != word(next);
                                                    let begin = p == 0 || (m && prev.is_some_and(|&c| lt(c)));
                                                    let end = next.is_none() || (m && next.is_some_and(|&c| lt(c)));
                                                    let dot = s || !lt(c);
                                                    let class = [0x61, fold(0x17f), fold(0x212a)].contains(&fold(c));
                                                    match case.1 {
                                                        0 => [0x61,0x62].contains(&fold(c)),
                                                        1 => class,
                                                        2 => dot || c == 10,
                                                        3 => true,
                                                        4 => (0x200c..=0x200d).contains(&c) || matches!(c,0xfdd0..=0xfdef) || c&0xffff >= 0xfffe,
                                                        5 | 14 => c == 0x1f600 || fold(c) == 0x61,
                                                        6 => [0xd800,0xdc00].contains(&c),
                                                        7 => c == 0x7c || fold(c) == 0x61,
                                                        8 => (fold(c) == 0x61 && wb) || (fold(c) == 0x62 && we),
                                                        9 => (fold(c) == 0x61 && begin) || (fold(c) == 0x62 && end),
                                                        10 => [0x61,0x62,0x7c].contains(&fold(c)),
                                                        11 => (dot && !wb) || (class && wb),
                                                        12 => (fold(c) == 0x61 && end) || (fold(c) == 0x62 && begin),
                                                        13 => (0x61..=0x7a).contains(&fold(c)) || (0x200c..=0x200d).contains(&c),
                                                        15 => (dot && begin && end && !wb) || (fold(c) == 0x61 && wb && we),
                                                        _ => unreachable!(),
                                                    }
                                                }).count();
                                                let upper =
                                                    available.min(case.3.unwrap_or(available));
                                                if upper < case.2 {
                                                    return None;
                                                }
                                                let endpoint = |n: usize| {
                                                    let end = at + n;
                                                    if case.9.is_some_and(|positive| (word(end.checked_sub(1).and_then(|n|points.get(n))) != word(points.get(end))) != positive) { return None; }

                                                    if case.7
                                                        && end != points.len()
                                                        && !(m && lt(points[end]))
                                                    {
                                                        return None;
                                                    }
                                                    Some(RegExpUnicodeRepeatedAtomMatch {
                                                        range: bounds[at]..bounds[end],
                                                        atom_capture: (case.5 != 0 && n != 0)
                                                            .then(|| bounds[end - 1]..bounds[end]),
                                                    })
                                                };
                                                let mut counts = case.2..=upper;
                                                if case.4 {
                                                    counts.rev().find_map(endpoint)
                                                } else {
                                                    counts.find_map(endpoint)
                                                }
                                            })
                                        };
                                        assert_eq!(matcher.capture_count(), case.5, "{}", case.0);
                                        assert_eq!(
                                            matcher
                                                .find_with_captures_and_work(
                                                    &input,
                                                    start,
                                                    sticky,
                                                    |_| Ok::<(), ()>(())
                                                )
                                                .unwrap(),
                                            expected,
                                            "{} i={i} m={m} s={s} {input:?} start={start} sticky={sticky}",
                                            case.0
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

    #[test]
    fn unicode_noncapturing_branch_nested_assertion_word_work_and_contracts() {
        let source = format!(
            r"\B{}(?:.)|(?:\n){}+\B$",
            "(".repeat(3000),
            ")".repeat(3000)
        );
        let matcher = plan(&source, true, false, true, false).clone();
        let input = JsString::from("\n".repeat(200000).as_str());
        let mut work = 0;
        assert_eq!(matcher.capture_count(), 3000);
        assert_eq!(
            matcher
                .find_with_captures_and_work(&input, 0, false, |n| {
                    work += n;
                    Ok::<(), ()>(())
                })
                .unwrap(),
            Some(RegExpUnicodeRepeatedAtomMatch {
                range: 0..200000,
                atom_capture: Some(199999..200000)
            })
        );
        assert!(work < 60 * input.len(), "{work}");
        let source = JsString::from(r"\B((?:\B^(?:\B.$)$)|(?:\b(?:\ba\b)\b)){1,3}\B$");

        for source in [
            r"\b((?:\b(?:\ba))|b)+\b",
            r"\b(a|(?:(?:b\b)\b))+",
            r"\B((?:\B^(?:\B.$)$)|(?:\b(?:\ba\b)\b))+\B",
        ] {
            let source = JsString::from(source);
            assert!(
                RegExpUnicodeRepeatedAtomMatcher::compile_noncapturing_branch_sequence_words_with_work(
                    &source,
                    true,
                    false,
                    true,
                    true,
                    |_| Ok::<(), ()>(())
                )
                .unwrap()
                .is_none()
            );
            assert!(RegExpUnicodeRepeatedAtomMatcher::compile_noncapturing_branch_nested_assertion_words_with_work(
                &source, true, false, true, true, |_| Ok::<(), ()>(())
            ).unwrap().is_some());
        }
        let mut calls = 0;
        RegExpUnicodeRepeatedAtomMatcher::compile_noncapturing_branch_nested_assertion_words_with_work(
            &source,
            true,
            false,
            true,
            true,
            |_| {
                calls += 1;
                Ok::<(), ()>(())
            },
        )
        .unwrap()
        .unwrap();
        for fail in 1..=calls {
            let mut n = 0;
            assert!(
                RegExpUnicodeRepeatedAtomMatcher::compile_noncapturing_branch_nested_assertion_words_with_work(
                    &source,
                    true,
                    false,
                    true,
                    true,
                    |_| {
                        n += 1;
                        if n == fail { Err(()) } else { Ok(()) }
                    }
                )
                .is_err()
            );
        }
        let input = JsString::from("\r\n");
        let matcher = plan(
            r"\B((?:\B^(?:\B.$)$)|(?:\b(?:\ba\b)\b)){1,3}\B$",
            true,
            false,
            true,
            true,
        );
        calls = 0;
        matcher
            .find_with_captures_and_work(&input, 0, false, |_| {
                calls += 1;
                Ok::<(), ()>(())
            })
            .unwrap();
        for fail in 1..=calls {
            let mut n = 0;
            assert!(
                matcher
                    .find_with_captures_and_work(&input, 0, false, |_| {
                        n += 1;
                        if n == fail { Err(()) } else { Ok(()) }
                    })
                    .is_err()
            );
        }
        for source in [
            r"\b(a|)+",
            r"\b(|b)+",
            r"\b(a|ab)+",
            r"\b((?:a)|(?:b)?)+",
            r"\b(a|(b))+",
            r"\B(a|\b)+",
            r"\b(a|(?i:b))+",
            r"\b((?:a)|(?:b))+c",
            r"\b(\b((?:a)|(?:b)))+",
            r"\b((a)|(b))+",
            r"\b((?:a)|(?:b)){2}c",
        ] {
            assert!(
                RegExpUnicodeRepeatedAtomMatcher::compile_noncapturing_branch_nested_assertion_words_with_work(
                    &JsString::from(source),
                    true,
                    false,
                    true,
                    true,
                    |_| Ok::<(), ()>(())
                )
                .unwrap()
                .is_none(),
                "{source}"
            );
        }
        for source in [
            r"\b((?:a)|(?:b))+",
            r"((?:^(?:^a))|(?:(?:b$)$))+\b",
            r"\b((?:\b(?:\ba))|(?:(?:b\b)\b))+\B",
        ] {
            assert!(
                RegExpUnicodeRepeatedAtomMatcher::compile_noncapturing_branch_lines_with_work(
                    &JsString::from(source),
                    true,
                    false,
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
}

#[cfg(test)]
mod nullable_alternative_tests {
    use super::*;
    use std::fmt::Write;

    fn plan(source: &str, v: bool, i: bool, m: bool, s: bool) -> RegExpUnicodeRepeatedAtomMatcher {
        RegExpUnicodeRepeatedAtomMatcher::compile_nullable_alternatives_with_work(
            &JsString::from(source),
            v,
            i,
            m,
            s,
            |_| Ok::<(), ()>(()),
        )
        .unwrap()
        .unwrap()
    }

    // A small, independent RepeatMatcher interpreter. Its optional empty
    // iterations are rejected before continuation; mandatory empties are kept.
    #[allow(clippy::too_many_arguments)]
    fn repeat(
        input: &[u8],
        at: usize,
        minimum: usize,
        maximum: usize,
        greedy: bool,
        empty_first: bool,
        end: bool,
        capture: Option<Range<usize>>,
    ) -> Option<(usize, Option<Range<usize>>)> {
        let continuation = || {
            if minimum == 0 && (!end || at == input.len()) {
                Some((at, capture.clone()))
            } else {
                None
            }
        };
        if !greedy {
            if let Some(found) = continuation() {
                return Some(found);
            }
        }
        if maximum != 0 {
            for empty in [empty_first, !empty_first] {
                let next = if empty {
                    at
                } else if input.get(at) == Some(&b'a') {
                    at + 1
                } else {
                    continue;
                };
                if minimum == 0 && next == at {
                    continue;
                }
                if let Some(found) = repeat(
                    input,
                    next,
                    minimum.saturating_sub(1),
                    maximum - 1,
                    greedy,
                    empty_first,
                    end,
                    Some(at..next),
                ) {
                    return Some(found);
                }
            }
        }
        if greedy { continuation() } else { None }
    }

    #[test]
    fn unicode_nullable_alternative_repeat_matcher_snapshot() {
        let mut rows = String::new();
        let mut audited = 0;
        for empty_first in [false, true] {
            for (quantifier, minimum, maximum, greedy) in [
                ("*", 0, None, true),
                ("*?", 0, None, false),
                ("+", 1, None, true),
                ("+?", 1, None, false),
                ("?", 0, Some(1), true),
                ("??", 0, Some(1), false),
            ] {
                for start_anchor in [false, true] {
                    for end_anchor in [false, true] {
                        let source = format!(
                            "{}({}){}{}",
                            if start_anchor { "^" } else { "" },
                            if empty_first { "|a" } else { "a|" },
                            quantifier,
                            if end_anchor { "$" } else { "" }
                        );
                        let p = plan(&source, false, false, false, false);
                        for input in [
                            "", "a", "b", "aa", "ab", "ba", "bb", "aaa", "aba", "baa", "aab",
                            "bbb", "aaaa", "aaab", "baaa", "abab", "baba",
                        ] {
                            for start in 0..=input.len() + 1 {
                                for sticky in [false, true] {
                                    let expected = (start..=input.len()).find_map(|at| {
                                        if (sticky && at != start) || (start_anchor && at != 0) {
                                            return None;
                                        }
                                        repeat(
                                            input.as_bytes(),
                                            at,
                                            minimum,
                                            maximum.unwrap_or(input.len() + 1),
                                            greedy,
                                            empty_first,
                                            end_anchor,
                                            None,
                                        )
                                        .map(|(end, capture)| (at..end, capture))
                                    });
                                    let actual = p
                                        .find_with_captures_and_work(
                                            &JsString::from(input),
                                            start,
                                            sticky,
                                            |_| Ok::<(), ()>(()),
                                        )
                                        .unwrap();
                                    let actual = actual.map(|m| (m.range, m.atom_capture));
                                    assert_eq!(
                                        actual, expected,
                                        "{source:?} {input:?} {start} {sticky}"
                                    );
                                    audited += 1;
                                    if start == 0 && !sticky && ["", "aaa", "baaa"].contains(&input)
                                    {
                                        writeln!(rows, "{source:?} {input:?} {actual:?}").unwrap();
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        writeln!(rows, "{audited} independent RepeatMatcher comparisons").unwrap();
        insta::assert_snapshot!(rows);
    }

    #[test]
    fn unicode_nullable_alternative_proof_stays_complete_and_narrow() {
        for source in [
            r"(a|b|)+",
            r"(||a)+",
            r"(|)+",
            r"(a|(?:))+",
            r"(a|\b)+",
            r"((a)|)+",
            r"(a|){1}",
            r"(a|){0,3}?",
            r"(ab|)+",
            r"(a+|)+",
            r"((?:a|b)|)+",
            r"(?:(?i:a)|)+",
            r"(?:a|)+b",
            r"(?:a|)(?:a|)+",
        ] {
            assert!(
                RegExpUnicodeRepeatedAtomMatcher::compile_nullable_alternatives_with_work(
                    &JsString::from(source),
                    true,
                    false,
                    false,
                    false,
                    |_| Ok::<(), ()>(()),
                )
                .unwrap()
                .is_none(),
                "{source}"
            );
        }
        for source in [
            r"((?:\b(?:\ba))|)+",
            r"(|(?:\B^(?:.$)$))+?",
            r"\B(((?:\p{Join_C})|))+\B$",
            r"((?:[a\|])|)??",
            r"(|\$)+?",
        ] {
            let p = plan(source, true, true, true, true);
            assert!(p.nullable.is_some());
            assert!(RegExpUnicodeRepeatedAtomMatcher::compile_noncapturing_branch_nested_assertion_words_with_work(
                &JsString::from(source), true, true, true, true, |_| Ok::<(), ()>(()),
            ).unwrap().is_none(), "old proof {source}");
        }
    }

    #[test]
    fn unicode_nullable_alternative_all_work_callbacks_can_abort() {
        let source =
            JsString::from(format!("\\B{}a|{}+?\\B", "(".repeat(24), ")".repeat(24)).as_str());
        let mut calls = 0;
        RegExpUnicodeRepeatedAtomMatcher::compile_nullable_alternatives_with_work(
            &source,
            true,
            false,
            false,
            false,
            |_| {
                calls += 1;
                Ok::<(), ()>(())
            },
        )
        .unwrap()
        .unwrap();
        for fail in 0..calls {
            let mut at = 0;
            assert!(matches!(
                RegExpUnicodeRepeatedAtomMatcher::compile_nullable_alternatives_with_work(
                    &source,
                    true,
                    false,
                    false,
                    false,
                    |_| {
                        let selected = at == fail;
                        at += 1;
                        if selected { Err(()) } else { Ok(()) }
                    },
                ),
                Err(())
            ));
        }
        let p = plan(r"\B(a|)+?\B", true, false, false, false);
        let input = JsString::from("baaa");
        let mut calls = 0;
        p.find_with_captures_and_work(&input, 0, false, |_| {
            calls += 1;
            Ok::<(), ()>(())
        })
        .unwrap();
        for fail in 0..calls {
            let mut at = 0;
            assert_eq!(
                p.find_with_captures_and_work(&input, 0, false, |_| {
                    let selected = at == fail;
                    at += 1;
                    if selected { Err(()) } else { Ok(()) }
                }),
                Err(())
            );
        }
    }
}

#[cfg(test)]
mod noncapturing_empty_alternative_tests {
    use super::*;
    use std::fmt::Write;

    fn plan(source: &str, v: bool, i: bool, m: bool, s: bool) -> RegExpUnicodeRepeatedAtomMatcher {
        RegExpUnicodeRepeatedAtomMatcher::compile_noncapturing_empty_alternatives_with_work(
            &JsString::from(source),
            v,
            i,
            m,
            s,
            |_| Ok::<(), ()>(()),
        )
        .unwrap()
        .unwrap()
    }

    // A small, independent RepeatMatcher interpreter. Its optional empty
    // iterations are rejected before continuation; mandatory empties are kept.
    #[allow(clippy::too_many_arguments)]
    fn repeat(
        input: &[u8],
        at: usize,
        minimum: usize,
        maximum: usize,
        greedy: bool,
        empty_first: bool,
        end: bool,
        capture: Option<Range<usize>>,
    ) -> Option<(usize, Option<Range<usize>>)> {
        let continuation = || {
            if minimum == 0 && (!end || at == input.len()) {
                Some((at, capture.clone()))
            } else {
                None
            }
        };
        if !greedy {
            if let Some(found) = continuation() {
                return Some(found);
            }
        }
        if maximum != 0 {
            for empty in [empty_first, !empty_first] {
                let next = if empty {
                    at
                } else if input.get(at) == Some(&b'a') {
                    at + 1
                } else {
                    continue;
                };
                if minimum == 0 && next == at {
                    continue;
                }
                if let Some(found) = repeat(
                    input,
                    next,
                    minimum.saturating_sub(1),
                    maximum - 1,
                    greedy,
                    empty_first,
                    end,
                    Some(at..next),
                ) {
                    return Some(found);
                }
            }
        }
        if greedy { continuation() } else { None }
    }

    #[test]
    fn unicode_noncapturing_empty_alternative_repeat_matcher_snapshot() {
        let mut rows = String::new();
        let mut audited = 0;
        for empty in ["(?:)", "(?:(?:(?:)))"] {
            for empty_first in [false, true] {
                for (quantifier, minimum, maximum, greedy) in [
                    ("*", 0, None, true),
                    ("*?", 0, None, false),
                    ("+", 1, None, true),
                    ("+?", 1, None, false),
                    ("?", 0, Some(1), true),
                    ("??", 0, Some(1), false),
                ] {
                    for start_anchor in [false, true] {
                        for end_anchor in [false, true] {
                            let source = format!(
                                "{}({}){}{}",
                                if start_anchor { "^" } else { "" },
                                if empty_first {
                                    format!("{empty}|a")
                                } else {
                                    format!("a|{empty}")
                                },
                                quantifier,
                                if end_anchor { "$" } else { "" }
                            );
                            let p = plan(&source, false, false, false, false);
                            for input in [
                                "", "a", "b", "aa", "ab", "ba", "bb", "aaa", "aba", "baa", "aab",
                                "bbb", "aaaa", "aaab", "baaa", "abab", "baba",
                            ] {
                                for start in 0..=input.len() + 1 {
                                    for sticky in [false, true] {
                                        let expected = (start..=input.len()).find_map(|at| {
                                            if (sticky && at != start) || (start_anchor && at != 0)
                                            {
                                                return None;
                                            }
                                            repeat(
                                                input.as_bytes(),
                                                at,
                                                minimum,
                                                maximum.unwrap_or(input.len() + 1),
                                                greedy,
                                                empty_first,
                                                end_anchor,
                                                None,
                                            )
                                            .map(|(end, capture)| (at..end, capture))
                                        });
                                        let actual = p
                                            .find_with_captures_and_work(
                                                &JsString::from(input),
                                                start,
                                                sticky,
                                                |_| Ok::<(), ()>(()),
                                            )
                                            .unwrap();
                                        let actual = actual.map(|m| (m.range, m.atom_capture));
                                        assert_eq!(
                                            actual, expected,
                                            "{source:?} {input:?} {start} {sticky}"
                                        );
                                        audited += 1;
                                        if start == 0
                                            && !sticky
                                            && ["", "aaa", "baaa"].contains(&input)
                                        {
                                            writeln!(rows, "{source:?} {input:?} {actual:?}")
                                                .unwrap();
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        writeln!(rows, "{audited} independent RepeatMatcher comparisons").unwrap();
        insta::assert_snapshot!(rows);
    }

    #[test]
    fn unicode_noncapturing_empty_alternative_proof_stays_complete_and_narrow() {
        for source in [
            r"(a|b|)+",
            r"(||a)+",
            r"(|)+",
            r"(a|(?:\b))+",
            r"(a|\b)+",
            r"((a)|)+",
            r"(a|){1}",
            r"(a|){0,3}?",
            r"(ab|)+",
            r"(a+|)+",
            r"((?:a|b)|)+",
            r"(?:(?i:a)|)+",
            r"(?:a|)+b",
            r"(?:a|)(?:a|)+",
        ] {
            assert!(
                RegExpUnicodeRepeatedAtomMatcher::compile_noncapturing_empty_alternatives_with_work(
                    &JsString::from(source),
                    true,
                    false,
                    false,
                    false,
                    |_| Ok::<(), ()>(()),
                )
                .unwrap()
                .is_none(),
                "{source}"
            );
        }
        for source in [
            r"((?:\b(?:\ba))|(?:))+",
            r"((?:(?:))|(?:\B^(?:.$)$))+?",
            r"\B(((?:\p{Join_C})|(?:(?:))))+\B$",
            r"((?:[a\|])|(?:))??",
            r"((?:)|\$)+?",
        ] {
            let p = plan(source, true, true, true, true);
            assert!(p.nullable.is_some());
            assert!(
                RegExpUnicodeRepeatedAtomMatcher::compile_nullable_alternatives_with_work(
                    &JsString::from(source),
                    true,
                    true,
                    true,
                    true,
                    |_| Ok::<(), ()>(()),
                )
                .unwrap()
                .is_none(),
                "raw-only proof {source}"
            );
            assert!(RegExpUnicodeRepeatedAtomMatcher::compile_noncapturing_branch_nested_assertion_words_with_work(
                &JsString::from(source), true, true, true, true, |_| Ok::<(), ()>(()),
            ).unwrap().is_none(), "old proof {source}");
        }
    }

    #[test]
    fn unicode_noncapturing_empty_alternative_all_work_callbacks_can_abort() {
        let source =
            JsString::from(format!("\\B{}a|{}+?\\B", "(".repeat(24), ")".repeat(24)).as_str());
        let mut calls = 0;
        RegExpUnicodeRepeatedAtomMatcher::compile_noncapturing_empty_alternatives_with_work(
            &source,
            true,
            false,
            false,
            false,
            |_| {
                calls += 1;
                Ok::<(), ()>(())
            },
        )
        .unwrap()
        .unwrap();
        for fail in 0..calls {
            let mut at = 0;
            assert!(matches!(
                RegExpUnicodeRepeatedAtomMatcher::compile_noncapturing_empty_alternatives_with_work(
                    &source,
                    true,
                    false,
                    false,
                    false,
                    |_| {
                        let selected = at == fail;
                        at += 1;
                        if selected { Err(()) } else { Ok(()) }
                    },
                ),
                Err(())
            ));
        }
        let p = plan(r"\B(a|)+?\B", true, false, false, false);
        let input = JsString::from("baaa");
        let mut calls = 0;
        p.find_with_captures_and_work(&input, 0, false, |_| {
            calls += 1;
            Ok::<(), ()>(())
        })
        .unwrap();
        for fail in 0..calls {
            let mut at = 0;
            assert_eq!(
                p.find_with_captures_and_work(&input, 0, false, |_| {
                    let selected = at == fail;
                    at += 1;
                    if selected { Err(()) } else { Ok(()) }
                }),
                Err(())
            );
        }
    }
}

#[cfg(test)]
mod zero_minimum_nullable_alternative_tests {
    use super::*;
    use std::fmt::Write;

    fn plan(source: &str, v: bool, i: bool, m: bool, s: bool) -> RegExpUnicodeRepeatedAtomMatcher {
        RegExpUnicodeRepeatedAtomMatcher::compile_zero_minimum_nullable_alternatives_with_work(
            &JsString::from(source),
            v,
            i,
            m,
            s,
            |_| Ok::<(), ()>(()),
        )
        .unwrap()
        .unwrap()
    }

    // A small, independent RepeatMatcher interpreter. Its optional empty
    // iterations are rejected before continuation; mandatory empties are kept.
    #[allow(clippy::too_many_arguments)]
    fn repeat(
        input: &[u8],
        at: usize,
        minimum: usize,
        maximum: usize,
        greedy: bool,
        empty_first: bool,
        end: bool,
        capture: Option<Range<usize>>,
    ) -> Option<(usize, Option<Range<usize>>)> {
        let continuation = || {
            if minimum == 0 && (!end || at == input.len()) {
                Some((at, capture.clone()))
            } else {
                None
            }
        };
        if !greedy {
            if let Some(found) = continuation() {
                return Some(found);
            }
        }
        if maximum != 0 {
            for empty in [empty_first, !empty_first] {
                let next = if empty {
                    at
                } else if input.get(at) == Some(&b'a') {
                    at + 1
                } else {
                    continue;
                };
                if minimum == 0 && next == at {
                    continue;
                }
                if let Some(found) = repeat(
                    input,
                    next,
                    minimum.saturating_sub(1),
                    maximum - 1,
                    greedy,
                    empty_first,
                    end,
                    Some(at..next),
                ) {
                    return Some(found);
                }
            }
        }
        if greedy { continuation() } else { None }
    }

    #[test]
    fn unicode_zero_minimum_nullable_alternative_repeat_matcher_snapshot() {
        let mut rows = String::new();
        let mut audited = 0;
        for empty in ["(?:)", "(?:(?:(?:)))"] {
            for empty_first in [false, true] {
                for (quantifier, minimum, maximum, greedy) in [
                    ("{0}", 0, Some(0), true),
                    ("{0,0}?", 0, Some(0), false),
                    ("{000,0003}", 0, Some(3), true),
                    ("{0,3}?", 0, Some(3), false),
                    ("{0,}", 0, None, true),
                    ("{0,999999999999999999999999}?", 0, None, false),
                ] {
                    for start_anchor in [false, true] {
                        for end_anchor in [false, true] {
                            let source = format!(
                                "{}({}){}{}",
                                if start_anchor { "^" } else { "" },
                                if empty_first {
                                    format!("{empty}|a")
                                } else {
                                    format!("a|{empty}")
                                },
                                quantifier,
                                if end_anchor { "$" } else { "" }
                            );
                            let p = plan(&source, false, false, false, false);
                            for input in [
                                "", "a", "b", "aa", "ab", "ba", "bb", "aaa", "aba", "baa", "aab",
                                "bbb", "aaaa", "aaab", "baaa", "abab", "baba",
                            ] {
                                for start in 0..=input.len() + 1 {
                                    for sticky in [false, true] {
                                        let expected = (start..=input.len()).find_map(|at| {
                                            if (sticky && at != start) || (start_anchor && at != 0)
                                            {
                                                return None;
                                            }
                                            repeat(
                                                input.as_bytes(),
                                                at,
                                                minimum,
                                                maximum.unwrap_or(input.len() + 1),
                                                greedy,
                                                empty_first,
                                                end_anchor,
                                                None,
                                            )
                                            .map(|(end, capture)| (at..end, capture))
                                        });
                                        let actual = p
                                            .find_with_captures_and_work(
                                                &JsString::from(input),
                                                start,
                                                sticky,
                                                |_| Ok::<(), ()>(()),
                                            )
                                            .unwrap();
                                        let actual = actual.map(|m| (m.range, m.atom_capture));
                                        assert_eq!(
                                            actual, expected,
                                            "{source:?} {input:?} {start} {sticky}"
                                        );
                                        audited += 1;
                                        if start == 0
                                            && !sticky
                                            && ["", "aaa", "baaa"].contains(&input)
                                        {
                                            writeln!(rows, "{source:?} {input:?} {actual:?}")
                                                .unwrap();
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        writeln!(rows, "{audited} independent RepeatMatcher comparisons").unwrap();
        insta::assert_snapshot!(rows);
    }

    #[test]
    fn unicode_zero_minimum_nullable_alternative_proof_stays_complete_and_narrow() {
        for source in [
            r"(a|b|)+",
            r"(||a)+",
            r"(|)+",
            r"(a|(?:\b))+",
            r"(a|\b)+",
            r"((a)|)+",
            r"(a|){1}",
            r"(a|){1,3}?",
            r"(ab|)+",
            r"(a+|)+",
            r"((?:a|b)|)+",
            r"(?:(?i:a)|)+",
            r"(?:a|)+b",
            r"(?:a|)(?:a|)+",
        ] {
            assert!(
                RegExpUnicodeRepeatedAtomMatcher::compile_zero_minimum_nullable_alternatives_with_work(
                    &JsString::from(source),
                    true,
                    false,
                    false,
                    false,
                    |_| Ok::<(), ()>(()),
                )
                .unwrap()
                .is_none(),
                "{source}"
            );
        }
        for source in [
            r"((?:\b(?:\ba))|(?:)){0,3}",
            r"((?:(?:))|(?:\B^(?:.$)$)){0,3}?",
            r"\B(((?:\p{Join_C})|(?:(?:)))){0,}\B$",
            r"((?:[a\|])|(?:)){0,0}?",
            r"((?:)|\$){0,999999999999999999999999}?",
        ] {
            let p = plan(source, true, true, true, true);
            assert!(p.nullable.is_some());
            assert!(RegExpUnicodeRepeatedAtomMatcher::compile_noncapturing_empty_alternatives_with_work(
                &JsString::from(source), true, true, true, true, |_| Ok::<(), ()>(()),
            ).unwrap().is_none(), "simple-quantifier proof {source}");
            assert!(
                RegExpUnicodeRepeatedAtomMatcher::compile_nullable_alternatives_with_work(
                    &JsString::from(source),
                    true,
                    true,
                    true,
                    true,
                    |_| Ok::<(), ()>(()),
                )
                .unwrap()
                .is_none(),
                "raw-only proof {source}"
            );
            assert!(RegExpUnicodeRepeatedAtomMatcher::compile_noncapturing_branch_nested_assertion_words_with_work(
                &JsString::from(source), true, true, true, true, |_| Ok::<(), ()>(()),
            ).unwrap().is_none(), "old proof {source}");
        }
    }

    #[test]
    fn unicode_zero_minimum_nullable_alternative_all_work_callbacks_can_abort() {
        let source = JsString::from(
            format!("\\B{}a|{}{{0,3}}?\\B", "(".repeat(24), ")".repeat(24)).as_str(),
        );
        let mut calls = 0;
        RegExpUnicodeRepeatedAtomMatcher::compile_zero_minimum_nullable_alternatives_with_work(
            &source,
            true,
            false,
            false,
            false,
            |_| {
                calls += 1;
                Ok::<(), ()>(())
            },
        )
        .unwrap()
        .unwrap();
        for fail in 0..calls {
            let mut at = 0;
            assert!(matches!(
                RegExpUnicodeRepeatedAtomMatcher::compile_zero_minimum_nullable_alternatives_with_work(
                    &source,
                    true,
                    false,
                    false,
                    false,
                    |_| {
                        let selected = at == fail;
                        at += 1;
                        if selected { Err(()) } else { Ok(()) }
                    },
                ),
                Err(())
            ));
        }
        let p = plan(r"\B(a|){0,3}?\B", true, false, false, false);
        let input = JsString::from("baaa");
        let mut calls = 0;
        p.find_with_captures_and_work(&input, 0, false, |_| {
            calls += 1;
            Ok::<(), ()>(())
        })
        .unwrap();
        for fail in 0..calls {
            let mut at = 0;
            assert_eq!(
                p.find_with_captures_and_work(&input, 0, false, |_| {
                    let selected = at == fail;
                    at += 1;
                    if selected { Err(()) } else { Ok(()) }
                }),
                Err(())
            );
        }
    }
}

#[cfg(test)]
mod one_minimum_open_nullable_alternative_tests {
    use super::*;
    use std::fmt::Write;

    fn plan(source: &str, v: bool, i: bool, m: bool, s: bool) -> RegExpUnicodeRepeatedAtomMatcher {
        RegExpUnicodeRepeatedAtomMatcher::compile_one_minimum_open_nullable_alternatives_with_work(
            &JsString::from(source),
            v,
            i,
            m,
            s,
            |_| Ok::<(), ()>(()),
        )
        .unwrap()
        .unwrap()
    }

    // A small, independent RepeatMatcher interpreter. Its optional empty
    // iterations are rejected before continuation; mandatory empties are kept.
    #[allow(clippy::too_many_arguments)]
    fn repeat(
        input: &[u8],
        at: usize,
        minimum: usize,
        maximum: usize,
        greedy: bool,
        empty_first: bool,
        end: bool,
        capture: Option<Range<usize>>,
    ) -> Option<(usize, Option<Range<usize>>)> {
        let continuation = || {
            if minimum == 0 && (!end || at == input.len()) {
                Some((at, capture.clone()))
            } else {
                None
            }
        };
        if !greedy {
            if let Some(found) = continuation() {
                return Some(found);
            }
        }
        if maximum != 0 {
            for empty in [empty_first, !empty_first] {
                let next = if empty {
                    at
                } else if input.get(at) == Some(&b'a') {
                    at + 1
                } else {
                    continue;
                };
                if minimum == 0 && next == at {
                    continue;
                }
                if let Some(found) = repeat(
                    input,
                    next,
                    minimum.saturating_sub(1),
                    maximum - 1,
                    greedy,
                    empty_first,
                    end,
                    Some(at..next),
                ) {
                    return Some(found);
                }
            }
        }
        if greedy { continuation() } else { None }
    }

    #[test]
    fn unicode_one_minimum_open_nullable_alternative_repeat_matcher_snapshot() {
        let mut rows = String::new();
        let mut audited = 0;
        for empty_first in [false, true] {
            for (quantifier, minimum, maximum, greedy) in [
                ("{1,}", 1, None, true),
                ("{1,}?", 1, None, false),
                ("{01,}", 1, None, true),
                ("{01,}?", 1, None, false),
                ("{000001,}", 1, None, true),
                ("{000001,}?", 1, None, false),
            ] {
                for start_anchor in [false, true] {
                    for end_anchor in [false, true] {
                        let source = format!(
                            "{}({}){}{}",
                            if start_anchor { "^" } else { "" },
                            if empty_first { "|a" } else { "a|" },
                            quantifier,
                            if end_anchor { "$" } else { "" }
                        );
                        let p = plan(&source, false, false, false, false);
                        for input in [
                            "", "a", "b", "aa", "ab", "ba", "bb", "aaa", "aba", "baa", "aab",
                            "bbb", "aaaa", "aaab", "baaa", "abab", "baba",
                        ] {
                            for start in 0..=input.len() + 1 {
                                for sticky in [false, true] {
                                    let expected = (start..=input.len()).find_map(|at| {
                                        if (sticky && at != start) || (start_anchor && at != 0) {
                                            return None;
                                        }
                                        repeat(
                                            input.as_bytes(),
                                            at,
                                            minimum,
                                            maximum.unwrap_or(input.len() + 1),
                                            greedy,
                                            empty_first,
                                            end_anchor,
                                            None,
                                        )
                                        .map(|(end, capture)| (at..end, capture))
                                    });
                                    let actual = p
                                        .find_with_captures_and_work(
                                            &JsString::from(input),
                                            start,
                                            sticky,
                                            |_| Ok::<(), ()>(()),
                                        )
                                        .unwrap();
                                    let actual = actual.map(|m| (m.range, m.atom_capture));
                                    assert_eq!(
                                        actual, expected,
                                        "{source:?} {input:?} {start} {sticky}"
                                    );
                                    audited += 1;
                                    if start == 0 && !sticky && ["", "aaa", "baaa"].contains(&input)
                                    {
                                        writeln!(rows, "{source:?} {input:?} {actual:?}").unwrap();
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        writeln!(rows, "{audited} independent RepeatMatcher comparisons").unwrap();
        insta::assert_snapshot!(rows);
    }

    #[test]
    fn unicode_one_minimum_open_nullable_alternative_proof_stays_complete_and_narrow() {
        for source in [
            r"(a|b|)+",
            r"(||a)+",
            r"(|)+",
            r"(a|(?:\b))+",
            r"(a|\b)+",
            r"((a)|)+",
            r"(a|){1}",
            r"(a|){2,}",
            r"(a|){0,}",
            r"(a|){9007199254740991,}",
            r"(a|){0,3}?",
            r"(ab|)+",
            r"(a+|)+",
            r"((?:a|b)|)+",
            r"(?:(?i:a)|)+",
            r"(?:a|)+b",
            r"(?:a|)(?:a|)+",
        ] {
            assert!(
                RegExpUnicodeRepeatedAtomMatcher::compile_one_minimum_open_nullable_alternatives_with_work(
                    &JsString::from(source),
                    true,
                    false,
                    false,
                    false,
                    |_| Ok::<(), ()>(()),
                )
                .unwrap()
                .is_none(),
                "{source}"
            );
        }
        for source in [
            r"((?:\b(?:\ba))|){1,}",
            r"((?:)|(?:\B^(?:.$)$)){01,}?",
            r"\B(((?:\p{Join_C})|)){0001,}\B$",
            r"((?:[a\|])|(?:)){1,}?",
            r"(|\$){1,}?",
        ] {
            let p = plan(source, true, true, true, true);
            assert!(p.nullable.is_some());
            assert!(RegExpUnicodeRepeatedAtomMatcher::compile_zero_minimum_nullable_alternatives_with_work(
                &JsString::from(source), true, true, true, true, |_| Ok::<(), ()>(()),
            ).unwrap().is_none(), "zero-lower-bound proof {source}");
            assert!(RegExpUnicodeRepeatedAtomMatcher::compile_noncapturing_branch_nested_assertion_words_with_work(
                &JsString::from(source), true, true, true, true, |_| Ok::<(), ()>(()),
            ).unwrap().is_none(), "old proof {source}");
        }
    }

    #[test]
    fn unicode_one_minimum_open_nullable_alternative_all_work_callbacks_can_abort() {
        let source =
            JsString::from(format!("\\B{}a|{}{{1,}}?\\B", "(".repeat(24), ")".repeat(24)).as_str());
        let mut calls = 0;
        RegExpUnicodeRepeatedAtomMatcher::compile_one_minimum_open_nullable_alternatives_with_work(
            &source,
            true,
            false,
            false,
            false,
            |_| {
                calls += 1;
                Ok::<(), ()>(())
            },
        )
        .unwrap()
        .unwrap();
        for fail in 0..calls {
            let mut at = 0;
            assert!(matches!(
                RegExpUnicodeRepeatedAtomMatcher::compile_one_minimum_open_nullable_alternatives_with_work(
                    &source,
                    true,
                    false,
                    false,
                    false,
                    |_| {
                        let selected = at == fail;
                        at += 1;
                        if selected { Err(()) } else { Ok(()) }
                    },
                ),
                Err(())
            ));
        }
        let p = plan(r"\B(a|){1,}?\B", true, false, false, false);
        let input = JsString::from("baaa");
        let mut calls = 0;
        p.find_with_captures_and_work(&input, 0, false, |_| {
            calls += 1;
            Ok::<(), ()>(())
        })
        .unwrap();
        for fail in 0..calls {
            let mut at = 0;
            assert_eq!(
                p.find_with_captures_and_work(&input, 0, false, |_| {
                    let selected = at == fail;
                    at += 1;
                    if selected { Err(()) } else { Ok(()) }
                }),
                Err(())
            );
        }
    }
}

#[cfg(test)]
mod one_iteration_nullable_alternative_tests {
    use super::*;
    use std::fmt::Write;

    fn plan(source: &str, v: bool, i: bool, m: bool, s: bool) -> RegExpUnicodeRepeatedAtomMatcher {
        RegExpUnicodeRepeatedAtomMatcher::compile_one_iteration_nullable_alternatives_with_work(
            &JsString::from(source),
            v,
            i,
            m,
            s,
            |_| Ok::<(), ()>(()),
        )
        .unwrap()
        .unwrap()
    }

    // A small, independent RepeatMatcher interpreter. Its optional empty
    // iterations are rejected before continuation; mandatory empties are kept.
    #[allow(clippy::too_many_arguments)]
    fn repeat(
        input: &[u8],
        at: usize,
        minimum: usize,
        maximum: usize,
        greedy: bool,
        empty_first: bool,
        end: bool,
        capture: Option<Range<usize>>,
    ) -> Option<(usize, Option<Range<usize>>)> {
        let continuation = || {
            if minimum == 0 && (!end || at == input.len()) {
                Some((at, capture.clone()))
            } else {
                None
            }
        };
        if !greedy {
            if let Some(found) = continuation() {
                return Some(found);
            }
        }
        if maximum != 0 {
            for empty in [empty_first, !empty_first] {
                let next = if empty {
                    at
                } else if input.get(at) == Some(&b'a') {
                    at + 1
                } else {
                    continue;
                };
                if minimum == 0 && next == at {
                    continue;
                }
                if let Some(found) = repeat(
                    input,
                    next,
                    minimum.saturating_sub(1),
                    maximum - 1,
                    greedy,
                    empty_first,
                    end,
                    Some(at..next),
                ) {
                    return Some(found);
                }
            }
        }
        if greedy { continuation() } else { None }
    }

    #[test]
    fn unicode_one_iteration_nullable_alternative_repeat_matcher_snapshot() {
        let mut rows = String::new();
        let mut audited = 0;
        for empty_first in [false, true] {
            for (quantifier, minimum, maximum, greedy) in [
                ("{1}", 1, Some(1), true),
                ("{1}?", 1, Some(1), false),
                ("{01,0001}", 1, Some(1), true),
                ("{01,0001}?", 1, Some(1), false),
                ("{000001}", 1, Some(1), true),
                ("{000001}?", 1, Some(1), false),
            ] {
                for start_anchor in [false, true] {
                    for end_anchor in [false, true] {
                        let source = format!(
                            "{}({}){}{}",
                            if start_anchor { "^" } else { "" },
                            if empty_first { "|a" } else { "a|" },
                            quantifier,
                            if end_anchor { "$" } else { "" }
                        );
                        let p = plan(&source, false, false, false, false);
                        for input in [
                            "", "a", "b", "aa", "ab", "ba", "bb", "aaa", "aba", "baa", "aab",
                            "bbb", "aaaa", "aaab", "baaa", "abab", "baba",
                        ] {
                            for start in 0..=input.len() + 1 {
                                for sticky in [false, true] {
                                    let expected = (start..=input.len()).find_map(|at| {
                                        if (sticky && at != start) || (start_anchor && at != 0) {
                                            return None;
                                        }
                                        repeat(
                                            input.as_bytes(),
                                            at,
                                            minimum,
                                            maximum.unwrap_or(input.len() + 1),
                                            greedy,
                                            empty_first,
                                            end_anchor,
                                            None,
                                        )
                                        .map(|(end, capture)| (at..end, capture))
                                    });
                                    let actual = p
                                        .find_with_captures_and_work(
                                            &JsString::from(input),
                                            start,
                                            sticky,
                                            |_| Ok::<(), ()>(()),
                                        )
                                        .unwrap();
                                    let actual = actual.map(|m| (m.range, m.atom_capture));
                                    assert_eq!(
                                        actual, expected,
                                        "{source:?} {input:?} {start} {sticky}"
                                    );
                                    audited += 1;
                                    if start == 0 && !sticky && ["", "aaa", "baaa"].contains(&input)
                                    {
                                        writeln!(rows, "{source:?} {input:?} {actual:?}").unwrap();
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        writeln!(rows, "{audited} independent RepeatMatcher comparisons").unwrap();
        insta::assert_snapshot!(rows);
    }

    #[test]
    fn unicode_one_iteration_nullable_alternative_proof_stays_complete_and_narrow() {
        for source in [
            r"(a|b|)+",
            r"(||a)+",
            r"(|)+",
            r"(a|(?:\b))+",
            r"(a|\b)+",
            r"((a)|)+",
            r"(a|){1,2}",
            r"(a|){2,}",
            r"(a|){0,}",
            r"(a|){9007199254740991,}",
            r"(a|){0,3}?",
            r"(ab|)+",
            r"(a+|)+",
            r"((?:a|b)|)+",
            r"(?:(?i:a)|)+",
            r"(?:a|)+b",
            r"(?:a|)(?:a|)+",
        ] {
            assert!(
                RegExpUnicodeRepeatedAtomMatcher::compile_one_iteration_nullable_alternatives_with_work(
                    &JsString::from(source),
                    true,
                    false,
                    false,
                    false,
                    |_| Ok::<(), ()>(()),
                )
                .unwrap()
                .is_none(),
                "{source}"
            );
        }
        for source in [
            r"((?:\b(?:\ba))|){1}",
            r"((?:)|(?:\B^(?:.$)$)){01,0001}?",
            r"\B(((?:\p{Join_C})|)){0001,0001}\B$",
            r"((?:[a\|])|(?:)){1}?",
            r"(|\$){1}?",
        ] {
            let p = plan(source, true, true, true, true);
            assert!(p.nullable.is_some());
            assert!(RegExpUnicodeRepeatedAtomMatcher::compile_one_minimum_open_nullable_alternatives_with_work(
                &JsString::from(source), true, true, true, true, |_| Ok::<(), ()>(()),
            ).unwrap().is_none(), "open-count proof {source}");
            assert!(RegExpUnicodeRepeatedAtomMatcher::compile_zero_minimum_nullable_alternatives_with_work(
                &JsString::from(source), true, true, true, true, |_| Ok::<(), ()>(()),
            ).unwrap().is_none(), "zero-lower-bound proof {source}");
            assert!(RegExpUnicodeRepeatedAtomMatcher::compile_noncapturing_branch_nested_assertion_words_with_work(
                &JsString::from(source), true, true, true, true, |_| Ok::<(), ()>(()),
            ).unwrap().is_none(), "old proof {source}");
        }
    }

    #[test]
    fn unicode_one_iteration_nullable_alternative_all_work_callbacks_can_abort() {
        let source =
            JsString::from(format!("\\B{}a|{}{{1}}?\\B", "(".repeat(24), ")".repeat(24)).as_str());
        let mut calls = 0;
        RegExpUnicodeRepeatedAtomMatcher::compile_one_iteration_nullable_alternatives_with_work(
            &source,
            true,
            false,
            false,
            false,
            |_| {
                calls += 1;
                Ok::<(), ()>(())
            },
        )
        .unwrap()
        .unwrap();
        for fail in 0..calls {
            let mut at = 0;
            assert!(matches!(
                RegExpUnicodeRepeatedAtomMatcher::compile_one_iteration_nullable_alternatives_with_work(
                    &source,
                    true,
                    false,
                    false,
                    false,
                    |_| {
                        let selected = at == fail;
                        at += 1;
                        if selected { Err(()) } else { Ok(()) }
                    },
                ),
                Err(())
            ));
        }
        let p = plan(r"\B(a|){1}?\B", true, false, false, false);
        let input = JsString::from("baaa");
        let mut calls = 0;
        p.find_with_captures_and_work(&input, 0, false, |_| {
            calls += 1;
            Ok::<(), ()>(())
        })
        .unwrap();
        for fail in 0..calls {
            let mut at = 0;
            assert_eq!(
                p.find_with_captures_and_work(&input, 0, false, |_| {
                    let selected = at == fail;
                    at += 1;
                    if selected { Err(()) } else { Ok(()) }
                }),
                Err(())
            );
        }
    }
}

#[cfg(test)]
mod positive_open_nullable_alternative_tests {
    use super::*;
    use std::fmt::Write;

    fn plan(source: &str, v: bool, i: bool, m: bool, s: bool) -> RegExpUnicodeRepeatedAtomMatcher {
        RegExpUnicodeRepeatedAtomMatcher::compile_positive_open_nullable_alternatives_with_work(
            &JsString::from(source),
            v,
            i,
            m,
            s,
            |_| Ok::<(), ()>(()),
        )
        .unwrap()
        .unwrap()
    }

    // A small, independent RepeatMatcher interpreter. Its optional empty
    // iterations are rejected before continuation; mandatory empties are kept.
    #[allow(clippy::too_many_arguments)]
    fn repeat(
        input: &[u8],
        at: usize,
        minimum: usize,
        maximum: usize,
        greedy: bool,
        empty_first: bool,
        end: bool,
        capture: Option<Range<usize>>,
    ) -> Option<(usize, Option<Range<usize>>)> {
        let continuation = || {
            if minimum == 0 && (!end || at == input.len()) {
                Some((at, capture.clone()))
            } else {
                None
            }
        };
        if !greedy {
            if let Some(found) = continuation() {
                return Some(found);
            }
        }
        if maximum != 0 {
            for empty in [empty_first, !empty_first] {
                let next = if empty {
                    at
                } else if input.get(at) == Some(&b'a') {
                    at + 1
                } else {
                    continue;
                };
                if minimum == 0 && next == at {
                    continue;
                }
                if let Some(found) = repeat(
                    input,
                    next,
                    minimum.saturating_sub(1),
                    maximum - 1,
                    greedy,
                    empty_first,
                    end,
                    Some(at..next),
                ) {
                    return Some(found);
                }
            }
        }
        if greedy { continuation() } else { None }
    }

    #[test]
    fn unicode_positive_open_nullable_alternative_repeat_matcher_snapshot() {
        let mut rows = String::new();
        let mut audited = 0;
        for empty_first in [false, true] {
            for (quantifier, minimum, maximum, greedy) in [
                ("{2,}", 2, None, true),
                ("{2,}?", 2, None, false),
                ("{3,}", 3, None, true),
                ("{3,}?", 3, None, false),
                ("{0003,}", 3, None, true),
                ("{0003,}?", 3, None, false),
                ("{7,}", 7, None, true),
                ("{7,}?", 7, None, false),
            ] {
                for start_anchor in [false, true] {
                    for end_anchor in [false, true] {
                        let source = format!(
                            "{}({}){}{}",
                            if start_anchor { "^" } else { "" },
                            if empty_first { "|a" } else { "a|" },
                            quantifier,
                            if end_anchor { "$" } else { "" }
                        );
                        let p = plan(&source, false, false, false, false);
                        for input in [
                            "", "a", "b", "aa", "ab", "ba", "bb", "aaa", "aba", "baa", "aab",
                            "bbb", "aaaa", "aaab", "baaa", "abab", "baba",
                        ] {
                            for start in 0..=input.len() + 1 {
                                for sticky in [false, true] {
                                    let expected = (start..=input.len()).find_map(|at| {
                                        if (sticky && at != start) || (start_anchor && at != 0) {
                                            return None;
                                        }
                                        repeat(
                                            input.as_bytes(),
                                            at,
                                            minimum,
                                            maximum.unwrap_or(input.len() + minimum),
                                            greedy,
                                            empty_first,
                                            end_anchor,
                                            None,
                                        )
                                        .map(|(end, capture)| (at..end, capture))
                                    });
                                    let actual = p
                                        .find_with_captures_and_work(
                                            &JsString::from(input),
                                            start,
                                            sticky,
                                            |_| Ok::<(), ()>(()),
                                        )
                                        .unwrap();
                                    let actual = actual.map(|m| (m.range, m.atom_capture));
                                    assert_eq!(
                                        actual, expected,
                                        "{source:?} {input:?} {start} {sticky}"
                                    );
                                    audited += 1;
                                    if start == 0 && !sticky && ["", "aaa", "baaa"].contains(&input)
                                    {
                                        writeln!(rows, "{source:?} {input:?} {actual:?}").unwrap();
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        writeln!(rows, "{audited} independent RepeatMatcher comparisons").unwrap();
        insta::assert_snapshot!(rows);
    }

    #[test]
    fn unicode_positive_open_nullable_alternative_proof_stays_complete_and_narrow() {
        for source in [
            r"(a|b|)+",
            r"(||a)+",
            r"(|)+",
            r"(a|(?:\b))+",
            r"(a|\b)+",
            r"((a)|)+",
            r"(a|){1,2}",
            r"(a|){2,3}",
            r"(a|){2}",
            r"(a|){0,}",
            r"(a|){0,3}?",
            r"(ab|)+",
            r"(a+|)+",
            r"((?:a|b)|)+",
            r"(?:(?i:a)|)+",
            r"(?:a|)+b",
            r"(?:a|)(?:a|)+",
        ] {
            assert!(
                RegExpUnicodeRepeatedAtomMatcher::compile_positive_open_nullable_alternatives_with_work(
                    &JsString::from(source),
                    true,
                    false,
                    false,
                    false,
                    |_| Ok::<(), ()>(()),
                )
                .unwrap()
                .is_none(),
                "{source}"
            );
        }
        for source in [
            r"((?:\b(?:\ba))|){2,}",
            r"((?:)|(?:\B^(?:.$)$)){0003,}?",
            r"\B(((?:\p{Join_C})|)){7,}\B$",
            r"((?:[a\|])|(?:)){2,}?",
            r"(|\$){3,}?",
        ] {
            let p = plan(source, true, true, true, true);
            assert!(p.nullable.is_some());
            assert!(RegExpUnicodeRepeatedAtomMatcher::compile_one_minimum_open_nullable_alternatives_with_work(
                &JsString::from(source), true, true, true, true, |_| Ok::<(), ()>(()),
            ).unwrap().is_none(), "open-count proof {source}");
            assert!(RegExpUnicodeRepeatedAtomMatcher::compile_zero_minimum_nullable_alternatives_with_work(
                &JsString::from(source), true, true, true, true, |_| Ok::<(), ()>(()),
            ).unwrap().is_none(), "zero-lower-bound proof {source}");
            assert!(RegExpUnicodeRepeatedAtomMatcher::compile_noncapturing_branch_nested_assertion_words_with_work(
                &JsString::from(source), true, true, true, true, |_| Ok::<(), ()>(()),
            ).unwrap().is_none(), "old proof {source}");
        }
    }

    #[test]
    fn unicode_positive_open_nullable_alternative_all_work_callbacks_can_abort() {
        let source =
            JsString::from(format!("\\B{}a|{}{{2,}}?\\B", "(".repeat(24), ")".repeat(24)).as_str());
        let mut calls = 0;
        RegExpUnicodeRepeatedAtomMatcher::compile_positive_open_nullable_alternatives_with_work(
            &source,
            true,
            false,
            false,
            false,
            |_| {
                calls += 1;
                Ok::<(), ()>(())
            },
        )
        .unwrap()
        .unwrap();
        for fail in 0..calls {
            let mut at = 0;
            assert!(matches!(
                RegExpUnicodeRepeatedAtomMatcher::compile_positive_open_nullable_alternatives_with_work(
                    &source,
                    true,
                    false,
                    false,
                    false,
                    |_| {
                        let selected = at == fail;
                        at += 1;
                        if selected { Err(()) } else { Ok(()) }
                    },
                ),
                Err(())
            ));
        }
        let p = plan(r"\B(a|){3,}?\B", true, false, false, false);
        let input = JsString::from("baaa");
        let mut calls = 0;
        p.find_with_captures_and_work(&input, 0, false, |_| {
            calls += 1;
            Ok::<(), ()>(())
        })
        .unwrap();
        for fail in 0..calls {
            let mut at = 0;
            assert_eq!(
                p.find_with_captures_and_work(&input, 0, false, |_| {
                    let selected = at == fail;
                    at += 1;
                    if selected { Err(()) } else { Ok(()) }
                }),
                Err(())
            );
        }
    }
}

#[cfg(test)]
mod positive_exact_nullable_alternative_tests {
    use super::*;
    use std::fmt::Write;

    fn plan(source: &str, v: bool, i: bool, m: bool, s: bool) -> RegExpUnicodeRepeatedAtomMatcher {
        RegExpUnicodeRepeatedAtomMatcher::compile_positive_exact_nullable_alternatives_with_work(
            &JsString::from(source),
            v,
            i,
            m,
            s,
            |_| Ok::<(), ()>(()),
        )
        .unwrap()
        .unwrap()
    }

    // A small, independent RepeatMatcher interpreter. Its optional empty
    // iterations are rejected before continuation; mandatory empties are kept.
    #[allow(clippy::too_many_arguments)]
    fn repeat(
        input: &[u8],
        at: usize,
        minimum: usize,
        maximum: usize,
        greedy: bool,
        empty_first: bool,
        end: bool,
        capture: Option<Range<usize>>,
    ) -> Option<(usize, Option<Range<usize>>)> {
        let continuation = || {
            if minimum == 0 && (!end || at == input.len()) {
                Some((at, capture.clone()))
            } else {
                None
            }
        };
        if !greedy {
            if let Some(found) = continuation() {
                return Some(found);
            }
        }
        if maximum != 0 {
            for empty in [empty_first, !empty_first] {
                let next = if empty {
                    at
                } else if input.get(at) == Some(&b'a') {
                    at + 1
                } else {
                    continue;
                };
                if minimum == 0 && next == at {
                    continue;
                }
                if let Some(found) = repeat(
                    input,
                    next,
                    minimum.saturating_sub(1),
                    maximum - 1,
                    greedy,
                    empty_first,
                    end,
                    Some(at..next),
                ) {
                    return Some(found);
                }
            }
        }
        if greedy { continuation() } else { None }
    }

    #[test]
    fn unicode_positive_exact_nullable_alternative_repeat_matcher_snapshot() {
        let mut rows = String::new();
        let mut audited = 0;
        for empty_first in [false, true] {
            for (quantifier, minimum, maximum, greedy) in [
                ("{2}", 2, Some(2), true),
                ("{2}?", 2, Some(2), false),
                ("{3}", 3, Some(3), true),
                ("{3}?", 3, Some(3), false),
                ("{0003,003}", 3, Some(3), true),
                ("{0003,003}?", 3, Some(3), false),
                ("{7}", 7, Some(7), true),
                ("{7}?", 7, Some(7), false),
            ] {
                for start_anchor in [false, true] {
                    for end_anchor in [false, true] {
                        let source = format!(
                            "{}({}){}{}",
                            if start_anchor { "^" } else { "" },
                            if empty_first { "|a" } else { "a|" },
                            quantifier,
                            if end_anchor { "$" } else { "" }
                        );
                        let p = plan(&source, false, false, false, false);
                        for input in [
                            "", "a", "b", "aa", "ab", "ba", "bb", "aaa", "aba", "baa", "aab",
                            "bbb", "aaaa", "aaab", "baaa", "abab", "baba",
                        ] {
                            for start in 0..=input.len() + 1 {
                                for sticky in [false, true] {
                                    let expected = (start..=input.len()).find_map(|at| {
                                        if (sticky && at != start) || (start_anchor && at != 0) {
                                            return None;
                                        }
                                        repeat(
                                            input.as_bytes(),
                                            at,
                                            minimum,
                                            maximum.unwrap_or(input.len() + minimum),
                                            greedy,
                                            empty_first,
                                            end_anchor,
                                            None,
                                        )
                                        .map(|(end, capture)| (at..end, capture))
                                    });
                                    let actual = p
                                        .find_with_captures_and_work(
                                            &JsString::from(input),
                                            start,
                                            sticky,
                                            |_| Ok::<(), ()>(()),
                                        )
                                        .unwrap();
                                    let actual = actual.map(|m| (m.range, m.atom_capture));
                                    assert_eq!(
                                        actual, expected,
                                        "{source:?} {input:?} {start} {sticky}"
                                    );
                                    audited += 1;
                                    if start == 0 && !sticky && ["", "aaa", "baaa"].contains(&input)
                                    {
                                        writeln!(rows, "{source:?} {input:?} {actual:?}").unwrap();
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        writeln!(rows, "{audited} independent RepeatMatcher comparisons").unwrap();
        insta::assert_snapshot!(rows);
    }

    #[test]
    fn unicode_positive_exact_nullable_alternative_proof_stays_complete_and_narrow() {
        for source in [
            r"(a|b|)+",
            r"(||a)+",
            r"(|)+",
            r"(a|(?:\b))+",
            r"(a|\b)+",
            r"((a)|)+",
            r"(a|){1,2}",
            r"(a|){2,3}",
            r"(a|){2,}",
            r"(a|){999999999999999999999999}",
            r"(a|){0,}",
            r"(a|){0,3}?",
            r"(ab|)+",
            r"(a+|)+",
            r"((?:a|b)|)+",
            r"(?:(?i:a)|)+",
            r"(?:a|)+b",
            r"(?:a|)(?:a|)+",
        ] {
            assert!(
                RegExpUnicodeRepeatedAtomMatcher::compile_positive_exact_nullable_alternatives_with_work(
                    &JsString::from(source),
                    true,
                    false,
                    false,
                    false,
                    |_| Ok::<(), ()>(()),
                )
                .unwrap()
                .is_none(),
                "{source}"
            );
        }
        for source in [
            r"((?:\b(?:\ba))|){2}",
            r"((?:)|(?:\B^(?:.$)$)){0003,003}?",
            r"\B(((?:\p{Join_C})|)){7}\B$",
            r"((?:[a\|])|(?:)){2}?",
            r"(|\$){3}?",
        ] {
            let p = plan(source, true, true, true, true);
            assert!(p.nullable.is_some());
            assert!(RegExpUnicodeRepeatedAtomMatcher::compile_positive_open_nullable_alternatives_with_work(
                &JsString::from(source), true, true, true, true, |_| Ok::<(), ()>(()),
            ).unwrap().is_none(), "open proof {source}");
            assert!(RegExpUnicodeRepeatedAtomMatcher::compile_one_minimum_open_nullable_alternatives_with_work(
                &JsString::from(source), true, true, true, true, |_| Ok::<(), ()>(()),
            ).unwrap().is_none(), "open-count proof {source}");
            assert!(RegExpUnicodeRepeatedAtomMatcher::compile_zero_minimum_nullable_alternatives_with_work(
                &JsString::from(source), true, true, true, true, |_| Ok::<(), ()>(()),
            ).unwrap().is_none(), "zero-lower-bound proof {source}");
            assert!(RegExpUnicodeRepeatedAtomMatcher::compile_noncapturing_branch_nested_assertion_words_with_work(
                &JsString::from(source), true, true, true, true, |_| Ok::<(), ()>(()),
            ).unwrap().is_none(), "old proof {source}");
        }
    }

    #[test]
    fn unicode_positive_exact_nullable_alternative_all_work_callbacks_can_abort() {
        let source =
            JsString::from(format!("\\B{}a|{}{{2}}?\\B", "(".repeat(24), ")".repeat(24)).as_str());
        let mut calls = 0;
        RegExpUnicodeRepeatedAtomMatcher::compile_positive_exact_nullable_alternatives_with_work(
            &source,
            true,
            false,
            false,
            false,
            |_| {
                calls += 1;
                Ok::<(), ()>(())
            },
        )
        .unwrap()
        .unwrap();
        for fail in 0..calls {
            let mut at = 0;
            assert!(matches!(
                RegExpUnicodeRepeatedAtomMatcher::compile_positive_exact_nullable_alternatives_with_work(
                    &source,
                    true,
                    false,
                    false,
                    false,
                    |_| {
                        let selected = at == fail;
                        at += 1;
                        if selected { Err(()) } else { Ok(()) }
                    },
                ),
                Err(())
            ));
        }
        let p = plan(r"\B(a|){3}?\B", true, false, false, false);
        let input = JsString::from("baaa");
        let mut calls = 0;
        p.find_with_captures_and_work(&input, 0, false, |_| {
            calls += 1;
            Ok::<(), ()>(())
        })
        .unwrap();
        for fail in 0..calls {
            let mut at = 0;
            assert_eq!(
                p.find_with_captures_and_work(&input, 0, false, |_| {
                    let selected = at == fail;
                    at += 1;
                    if selected { Err(()) } else { Ok(()) }
                }),
                Err(())
            );
        }
    }
}

#[cfg(test)]
mod positive_finite_nullable_alternative_tests {
    use super::*;
    use std::fmt::Write;

    fn plan(source: &str, v: bool, i: bool, m: bool, s: bool) -> RegExpUnicodeRepeatedAtomMatcher {
        RegExpUnicodeRepeatedAtomMatcher::compile_positive_finite_nullable_alternatives_with_work(
            &JsString::from(source),
            v,
            i,
            m,
            s,
            |_| Ok::<(), ()>(()),
        )
        .unwrap()
        .unwrap()
    }

    // A small, independent RepeatMatcher interpreter. Its optional empty
    // iterations are rejected before continuation; mandatory empties are kept.
    #[allow(clippy::too_many_arguments)]
    fn repeat(
        input: &[u8],
        at: usize,
        minimum: usize,
        maximum: usize,
        greedy: bool,
        empty_first: bool,
        end: bool,
        capture: Option<Range<usize>>,
    ) -> Option<(usize, Option<Range<usize>>)> {
        let continuation = || {
            if minimum == 0 && (!end || at == input.len()) {
                Some((at, capture.clone()))
            } else {
                None
            }
        };
        if !greedy {
            if let Some(found) = continuation() {
                return Some(found);
            }
        }
        if maximum != 0 {
            for empty in [empty_first, !empty_first] {
                let next = if empty {
                    at
                } else if input.get(at) == Some(&b'a') {
                    at + 1
                } else {
                    continue;
                };
                if minimum == 0 && next == at {
                    continue;
                }
                if let Some(found) = repeat(
                    input,
                    next,
                    minimum.saturating_sub(1),
                    maximum - 1,
                    greedy,
                    empty_first,
                    end,
                    Some(at..next),
                ) {
                    return Some(found);
                }
            }
        }
        if greedy { continuation() } else { None }
    }

    #[test]
    fn unicode_positive_finite_nullable_alternative_repeat_matcher_snapshot() {
        let mut rows = String::new();
        let mut audited = 0;
        for empty_first in [false, true] {
            for (quantifier, minimum, maximum, greedy) in [
                ("{1,2}", 1, Some(2), true),
                ("{1,2}?", 1, Some(2), false),
                ("{2,3}", 2, Some(3), true),
                ("{2,3}?", 2, Some(3), false),
                ("{3,7}", 3, Some(7), true),
                ("{3,7}?", 3, Some(7), false),
                ("{0002,005}", 2, Some(5), true),
                ("{0002,005}?", 2, Some(5), false),
            ] {
                for start_anchor in [false, true] {
                    for end_anchor in [false, true] {
                        let source = format!(
                            "{}({}){}{}",
                            if start_anchor { "^" } else { "" },
                            if empty_first { "|a" } else { "a|" },
                            quantifier,
                            if end_anchor { "$" } else { "" }
                        );
                        let p = plan(&source, false, false, false, false);
                        for input in [
                            "", "a", "b", "aa", "ab", "ba", "bb", "aaa", "aba", "baa", "aab",
                            "bbb", "aaaa", "aaab", "baaa", "abab", "baba",
                        ] {
                            for start in 0..=input.len() + 1 {
                                for sticky in [false, true] {
                                    let expected = (start..=input.len()).find_map(|at| {
                                        if (sticky && at != start) || (start_anchor && at != 0) {
                                            return None;
                                        }
                                        repeat(
                                            input.as_bytes(),
                                            at,
                                            minimum,
                                            maximum.unwrap_or(input.len() + minimum),
                                            greedy,
                                            empty_first,
                                            end_anchor,
                                            None,
                                        )
                                        .map(|(end, capture)| (at..end, capture))
                                    });
                                    let actual = p
                                        .find_with_captures_and_work(
                                            &JsString::from(input),
                                            start,
                                            sticky,
                                            |_| Ok::<(), ()>(()),
                                        )
                                        .unwrap();
                                    let actual = actual.map(|m| (m.range, m.atom_capture));
                                    assert_eq!(
                                        actual, expected,
                                        "{source:?} {input:?} {start} {sticky}"
                                    );
                                    audited += 1;
                                    if start == 0 && !sticky && ["", "aaa", "baaa"].contains(&input)
                                    {
                                        writeln!(rows, "{source:?} {input:?} {actual:?}").unwrap();
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        writeln!(rows, "{audited} independent RepeatMatcher comparisons").unwrap();
        insta::assert_snapshot!(rows);
    }

    #[test]
    fn unicode_positive_finite_nullable_alternative_proof_stays_complete_and_narrow() {
        for source in [
            r"(a|b|)+",
            r"(||a)+",
            r"(|)+",
            r"(a|(?:\b))+",
            r"(a|\b)+",
            r"((a)|)+",
            r"(a|){2,}",
            r"(a|){1,999999999999999999999999}",
            r"(a|){2}",
            r"(a|){2,2}",
            r"(a|){0,}",
            r"(a|){0,3}?",
            r"(ab|)+",
            r"(a+|)+",
            r"((?:a|b)|)+",
            r"(?:(?i:a)|)+",
            r"(?:a|)+b",
            r"(?:a|)(?:a|)+",
        ] {
            assert!(
                RegExpUnicodeRepeatedAtomMatcher::compile_positive_finite_nullable_alternatives_with_work(
                    &JsString::from(source),
                    true,
                    false,
                    false,
                    false,
                    |_| Ok::<(), ()>(()),
                )
                .unwrap()
                .is_none(),
                "{source}"
            );
        }
        for source in [
            r"((?:\b(?:\ba))|){2,3}",
            r"((?:)|(?:\B^(?:.$)$)){0003,007}?",
            r"\B(((?:\p{Join_C})|)){1,2}\B$",
            r"((?:[a\|])|(?:)){2,5}?",
            r"(|\$){3,7}?",
        ] {
            let p = plan(source, true, true, true, true);
            assert!(p.nullable.is_some());
            assert!(RegExpUnicodeRepeatedAtomMatcher::compile_positive_exact_nullable_alternatives_with_work(
                &JsString::from(source), true, true, true, true, |_| Ok::<(), ()>(()),
            ).unwrap().is_none(), "exact proof {source}");
            assert!(RegExpUnicodeRepeatedAtomMatcher::compile_positive_open_nullable_alternatives_with_work(
                &JsString::from(source), true, true, true, true, |_| Ok::<(), ()>(()),
            ).unwrap().is_none(), "open proof {source}");
            assert!(RegExpUnicodeRepeatedAtomMatcher::compile_one_minimum_open_nullable_alternatives_with_work(
                &JsString::from(source), true, true, true, true, |_| Ok::<(), ()>(()),
            ).unwrap().is_none(), "open-count proof {source}");
            assert!(RegExpUnicodeRepeatedAtomMatcher::compile_zero_minimum_nullable_alternatives_with_work(
                &JsString::from(source), true, true, true, true, |_| Ok::<(), ()>(()),
            ).unwrap().is_none(), "zero-lower-bound proof {source}");
            assert!(RegExpUnicodeRepeatedAtomMatcher::compile_noncapturing_branch_nested_assertion_words_with_work(
                &JsString::from(source), true, true, true, true, |_| Ok::<(), ()>(()),
            ).unwrap().is_none(), "old proof {source}");
        }
    }

    #[test]
    fn unicode_positive_finite_nullable_alternative_all_work_callbacks_can_abort() {
        let source = JsString::from(
            format!("\\B{}a|{}{{2,3}}?\\B", "(".repeat(24), ")".repeat(24)).as_str(),
        );
        let mut calls = 0;
        RegExpUnicodeRepeatedAtomMatcher::compile_positive_finite_nullable_alternatives_with_work(
            &source,
            true,
            false,
            false,
            false,
            |_| {
                calls += 1;
                Ok::<(), ()>(())
            },
        )
        .unwrap()
        .unwrap();
        for fail in 0..calls {
            let mut at = 0;
            assert!(matches!(
                RegExpUnicodeRepeatedAtomMatcher::compile_positive_finite_nullable_alternatives_with_work(
                    &source,
                    true,
                    false,
                    false,
                    false,
                    |_| {
                        let selected = at == fail;
                        at += 1;
                        if selected { Err(()) } else { Ok(()) }
                    },
                ),
                Err(())
            ));
        }
        for source in [r"\B(a|){3,7}?\B", r"\B(|a){2,3}\B"] {
            let p = plan(source, true, false, false, false);
            let input = JsString::from("baaa");
            let mut calls = 0;
            p.find_with_captures_and_work(&input, 0, false, |_| {
                calls += 1;
                Ok::<(), ()>(())
            })
            .unwrap();
            for fail in 0..calls {
                let mut at = 0;
                assert_eq!(
                    p.find_with_captures_and_work(&input, 0, false, |_| {
                        let selected = at == fail;
                        at += 1;
                        if selected { Err(()) } else { Ok(()) }
                    }),
                    Err(())
                );
            }
        }
    }
}

#[cfg(test)]
mod unrepresentable_exact_nullable_alternative_tests {
    use super::*;
    use std::fmt::Write;

    fn plan(source: &str, v: bool, i: bool, m: bool, s: bool) -> RegExpUnicodeRepeatedAtomMatcher {
        RegExpUnicodeRepeatedAtomMatcher::compile_unrepresentable_exact_nullable_alternatives_with_work(
            &JsString::from(source),
            v,
            i,
            m,
            s,
            |_| Ok::<(), ()>(()),
        )
        .unwrap()
        .unwrap()
    }

    // A small, independent RepeatMatcher interpreter. Its optional empty
    // iterations are rejected before continuation; mandatory empties are kept.
    #[allow(clippy::too_many_arguments)]
    fn repeat(
        input: &[u8],
        at: usize,
        minimum: usize,
        maximum: usize,
        greedy: bool,
        empty_first: bool,
        end: bool,
        capture: Option<Range<usize>>,
    ) -> Option<(usize, Option<Range<usize>>)> {
        let continuation = || {
            if minimum == 0 && (!end || at == input.len()) {
                Some((at, capture.clone()))
            } else {
                None
            }
        };
        if !greedy {
            if let Some(found) = continuation() {
                return Some(found);
            }
        }
        if maximum != 0 {
            for empty in [empty_first, !empty_first] {
                let next = if empty {
                    at
                } else if input.get(at) == Some(&b'a') {
                    at + 1
                } else {
                    continue;
                };
                if minimum == 0 && next == at {
                    continue;
                }
                if let Some(found) = repeat(
                    input,
                    next,
                    minimum.saturating_sub(1),
                    maximum - 1,
                    greedy,
                    empty_first,
                    end,
                    Some(at..next),
                ) {
                    return Some(found);
                }
            }
        }
        if greedy { continuation() } else { None }
    }

    #[test]
    fn unicode_unrepresentable_exact_nullable_alternative_repeat_matcher_snapshot() {
        let mut rows = String::new();
        let mut audited = 0;
        for empty_first in [false, true] {
            // Every source bound exceeds usize::MAX. These inputs have at
            // most four characters, so seven mandatory iterations are an
            // independent finite RepeatMatcher reference with the same choices.
            for (quantifier, minimum, maximum, greedy) in [
                ("{999999999999999999999999}", 7, Some(7), true),
                ("{999999999999999999999999}?", 7, Some(7), false),
                (
                    "{000999999999999999999999999,999999999999999999999999}",
                    7,
                    Some(7),
                    true,
                ),
                (
                    "{999999999999999999999999,000999999999999999999999999}?",
                    7,
                    Some(7),
                    false,
                ),
                ("{18446744073709551616}", 7, Some(7), true),
                ("{18446744073709551616}?", 7, Some(7), false),
                (
                    "{00018446744073709551616,18446744073709551616}",
                    7,
                    Some(7),
                    true,
                ),
                (
                    "{18446744073709551616,00018446744073709551616}?",
                    7,
                    Some(7),
                    false,
                ),
            ] {
                for start_anchor in [false, true] {
                    for end_anchor in [false, true] {
                        let source = format!(
                            "{}({}){}{}",
                            if start_anchor { "^" } else { "" },
                            if empty_first { "|a" } else { "a|" },
                            quantifier,
                            if end_anchor { "$" } else { "" }
                        );
                        let p = plan(&source, false, false, false, false);
                        assert!(p.minimum.is_none() && p.maximum.is_none());
                        assert_eq!(p.greedy, !empty_first);
                        for input in [
                            "", "a", "b", "aa", "ab", "ba", "bb", "aaa", "aba", "baa", "aab",
                            "bbb", "aaaa", "aaab", "baaa", "abab", "baba",
                        ] {
                            for start in 0..=input.len() + 1 {
                                for sticky in [false, true] {
                                    let expected = (start..=input.len()).find_map(|at| {
                                        if (sticky && at != start) || (start_anchor && at != 0) {
                                            return None;
                                        }
                                        repeat(
                                            input.as_bytes(),
                                            at,
                                            minimum,
                                            maximum.unwrap_or(input.len() + minimum),
                                            greedy,
                                            empty_first,
                                            end_anchor,
                                            None,
                                        )
                                        .map(|(end, capture)| (at..end, capture))
                                    });
                                    let actual = p
                                        .find_with_captures_and_work(
                                            &JsString::from(input),
                                            start,
                                            sticky,
                                            |_| Ok::<(), ()>(()),
                                        )
                                        .unwrap();
                                    let actual = actual.map(|m| (m.range, m.atom_capture));
                                    assert_eq!(
                                        actual, expected,
                                        "{source:?} {input:?} {start} {sticky}"
                                    );
                                    audited += 1;
                                    if start == 0 && !sticky && ["", "aaa", "baaa"].contains(&input)
                                    {
                                        writeln!(rows, "{source:?} {input:?} {actual:?}").unwrap();
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        writeln!(rows, "{audited} independent RepeatMatcher comparisons").unwrap();
        insta::assert_snapshot!(rows);
    }

    #[test]
    fn unicode_unrepresentable_exact_nullable_alternative_proof_stays_complete_and_narrow() {
        for source in [
            r"(a|b|)+",
            r"(||a)+",
            r"(|)+",
            r"(a|(?:\b))+",
            r"((a)|)+",
            r"(a|){1}",
            r"(a|){2,2}",
            r"(a|){2,3}",
            r"(a|){0,}",
            r"(a|){999999999999999999999999,}",
            r"(a|){999999999999999999999999,1000000000000000000000000}",
            r"(ab|)+",
            r"(a+|)+",
            r"((?:a|b)|)+",
            r"(?:(?i:a)|)+",
            r"(?:a|)+b",
        ] {
            assert!(
                RegExpUnicodeRepeatedAtomMatcher::compile_unrepresentable_exact_nullable_alternatives_with_work(
                    &JsString::from(source),
                    true,
                    false,
                    false,
                    false,
                    |_| Ok::<(), ()>(()),
                )
                .unwrap()
                .is_none(),
                "{source}"
            );
        }
        for source in [
            r"((?:\b(?:\ba))|){999999999999999999999999}",
            r"((?:)|(?:\B^(?:.$)$)){000999999999999999999999999,999999999999999999999999}?",
            r"\B(((?:\p{Join_C})|)){18446744073709551616}\B$",
            r"((?:[a\|])|(?:)){999999999999999999999999}?",
            r"(|\$){18446744073709551616,00018446744073709551616}?",
        ] {
            let p = plan(source, true, true, true, true);
            assert!(p.nullable.is_some());
            assert!(RegExpUnicodeRepeatedAtomMatcher::compile_positive_exact_nullable_alternatives_with_work(
                &JsString::from(source), true, true, true, true, |_| Ok::<(), ()>(()),
            ).unwrap().is_none(), "represented exact proof {source}");
            assert!(RegExpUnicodeRepeatedAtomMatcher::compile_positive_open_nullable_alternatives_with_work(
                &JsString::from(source), true, true, true, true, |_| Ok::<(), ()>(()),
            ).unwrap().is_none(), "open proof {source}");
            assert!(RegExpUnicodeRepeatedAtomMatcher::compile_one_minimum_open_nullable_alternatives_with_work(
                &JsString::from(source), true, true, true, true, |_| Ok::<(), ()>(()),
            ).unwrap().is_none(), "open-count proof {source}");
            assert!(RegExpUnicodeRepeatedAtomMatcher::compile_zero_minimum_nullable_alternatives_with_work(
                &JsString::from(source), true, true, true, true, |_| Ok::<(), ()>(()),
            ).unwrap().is_none(), "zero-lower-bound proof {source}");
            assert!(RegExpUnicodeRepeatedAtomMatcher::compile_noncapturing_branch_nested_assertion_words_with_work(
                &JsString::from(source), true, true, true, true, |_| Ok::<(), ()>(()),
            ).unwrap().is_none(), "old proof {source}");
        }
    }

    #[test]
    fn unicode_unrepresentable_exact_nullable_alternative_all_work_callbacks_can_abort() {
        let source = JsString::from(
            format!(
                "\\B{}a|{}{{999999999999999999999999}}?\\B",
                "(".repeat(24),
                ")".repeat(24)
            )
            .as_str(),
        );
        let mut calls = 0;
        RegExpUnicodeRepeatedAtomMatcher::compile_unrepresentable_exact_nullable_alternatives_with_work(
            &source,
            true,
            false,
            false,
            false,
            |_| {
                calls += 1;
                Ok::<(), ()>(())
            },
        )
        .unwrap()
        .unwrap();
        for fail in 0..calls {
            let mut at = 0;
            assert!(matches!(
                RegExpUnicodeRepeatedAtomMatcher::compile_unrepresentable_exact_nullable_alternatives_with_work(
                    &source,
                    true,
                    false,
                    false,
                    false,
                    |_| {
                        let selected = at == fail;
                        at += 1;
                        if selected { Err(()) } else { Ok(()) }
                    },
                ),
                Err(())
            ));
        }
        for source in [
            r"\B(a|){999999999999999999999999}?\B",
            r"\B(|a){18446744073709551616}\B",
            r"\B(|a){999999999999999999999999}$",
        ] {
            let p = plan(source, true, false, false, false);
            let input = JsString::from("baaa");
            let mut calls = 0;
            p.find_with_captures_and_work(&input, 0, false, |_| {
                calls += 1;
                Ok::<(), ()>(())
            })
            .unwrap();
            for fail in 0..calls {
                let mut at = 0;
                assert_eq!(
                    p.find_with_captures_and_work(&input, 0, false, |_| {
                        let selected = at == fail;
                        at += 1;
                        if selected { Err(()) } else { Ok(()) }
                    }),
                    Err(())
                );
            }
        }
    }
}

#[cfg(test)]
mod unrepresentable_finite_nullable_alternative_tests {
    use super::*;
    use std::fmt::Write;

    fn plan(source: &str, v: bool, i: bool, m: bool, s: bool) -> RegExpUnicodeRepeatedAtomMatcher {
        RegExpUnicodeRepeatedAtomMatcher::compile_unrepresentable_finite_nullable_alternatives_with_work(
            &JsString::from(source),
            v,
            i,
            m,
            s,
            |_| Ok::<(), ()>(()),
        )
        .unwrap()
        .unwrap()
    }

    // A small, independent RepeatMatcher interpreter. Its optional empty
    // iterations are rejected before continuation; mandatory empties are kept.
    #[allow(clippy::too_many_arguments)]
    fn repeat(
        input: &[u8],
        at: usize,
        minimum: usize,
        maximum: usize,
        greedy: bool,
        empty_first: bool,
        end: bool,
        capture: Option<Range<usize>>,
    ) -> Option<(usize, Option<Range<usize>>)> {
        let continuation = || {
            if minimum == 0 && (!end || at == input.len()) {
                Some((at, capture.clone()))
            } else {
                None
            }
        };
        if !greedy {
            if let Some(found) = continuation() {
                return Some(found);
            }
        }
        if maximum != 0 {
            for empty in [empty_first, !empty_first] {
                let next = if empty {
                    at
                } else if input.get(at) == Some(&b'a') {
                    at + 1
                } else {
                    continue;
                };
                if minimum == 0 && next == at {
                    continue;
                }
                if let Some(found) = repeat(
                    input,
                    next,
                    minimum.saturating_sub(1),
                    maximum - 1,
                    greedy,
                    empty_first,
                    end,
                    Some(at..next),
                ) {
                    return Some(found);
                }
            }
        }
        if greedy { continuation() } else { None }
    }

    #[test]
    fn unicode_unrepresentable_finite_nullable_alternative_repeat_matcher_snapshot() {
        let mut rows = String::new();
        let mut audited = 0;
        for empty_first in [false, true] {
            // Every source bound exceeds usize::MAX. These inputs have at
            // most four characters, so seven mandatory iterations are an
            // independent finite RepeatMatcher reference retaining the exact optional budget.
            // Differences larger than any tested input use seven optional copies.
            for (quantifier, minimum, maximum, greedy) in [
                (
                    "{18446744073709551616,18446744073709551617}",
                    7,
                    Some(8),
                    true,
                ),
                (
                    "{18446744073709551616,18446744073709551617}?",
                    7,
                    Some(8),
                    false,
                ),
                (
                    "{999999999999999999999999,1000000000000000000000000}",
                    7,
                    Some(8),
                    true,
                ),
                (
                    "{000999999999999999999999999,1000000000000000000000000}?",
                    7,
                    Some(8),
                    false,
                ),
                (
                    "{999999999999999999999999,1000000000000000000000002}",
                    7,
                    Some(10),
                    true,
                ),
                (
                    "{999999999999999999999999,0001000000000000000000000002}?",
                    7,
                    Some(10),
                    false,
                ),
                (
                    "{18446744073709551616,36893488147419103231}",
                    7,
                    Some(14),
                    true,
                ),
                (
                    "{18446744073709551616,36893488147419103232}?",
                    7,
                    Some(14),
                    false,
                ),
            ] {
                for start_anchor in [false, true] {
                    for end_anchor in [false, true] {
                        let source = format!(
                            "{}({}){}{}",
                            if start_anchor { "^" } else { "" },
                            if empty_first { "|a" } else { "a|" },
                            quantifier,
                            if end_anchor { "$" } else { "" }
                        );
                        let p = plan(&source, false, false, false, false);
                        assert!(p.minimum.is_none() && p.maximum.is_none());
                        assert_eq!(p.greedy, !empty_first || greedy);
                        for input in [
                            "", "a", "b", "aa", "ab", "ba", "bb", "aaa", "aba", "baa", "aab",
                            "bbb", "aaaa", "aaab", "baaa", "abab", "baba",
                        ] {
                            for start in 0..=input.len() + 1 {
                                for sticky in [false, true] {
                                    let expected = (start..=input.len()).find_map(|at| {
                                        if (sticky && at != start) || (start_anchor && at != 0) {
                                            return None;
                                        }
                                        repeat(
                                            input.as_bytes(),
                                            at,
                                            minimum,
                                            maximum.unwrap_or(input.len() + minimum),
                                            greedy,
                                            empty_first,
                                            end_anchor,
                                            None,
                                        )
                                        .map(|(end, capture)| (at..end, capture))
                                    });
                                    let actual = p
                                        .find_with_captures_and_work(
                                            &JsString::from(input),
                                            start,
                                            sticky,
                                            |_| Ok::<(), ()>(()),
                                        )
                                        .unwrap();
                                    let actual = actual.map(|m| (m.range, m.atom_capture));
                                    assert_eq!(
                                        actual, expected,
                                        "{source:?} {input:?} {start} {sticky}"
                                    );
                                    audited += 1;
                                    if start == 0 && !sticky && ["", "aaa", "baaa"].contains(&input)
                                    {
                                        writeln!(rows, "{source:?} {input:?} {actual:?}").unwrap();
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        writeln!(rows, "{audited} independent RepeatMatcher comparisons").unwrap();
        insta::assert_snapshot!(rows);
    }

    #[test]
    fn unicode_unrepresentable_finite_nullable_alternative_proof_stays_complete_and_narrow() {
        for source in [
            r"(a|b|)+",
            r"(||a)+",
            r"(|)+",
            r"(a|(?:\b))+",
            r"((a)|)+",
            r"(a|){1}",
            r"(a|){2,2}",
            r"(a|){2,3}",
            r"(a|){0,}",
            r"(a|){999999999999999999999999,}",
            r"(a|){999999999999999999999999,999999999999999999999999}",
            r"(ab|)+",
            r"(a+|)+",
            r"((?:a|b)|)+",
            r"(?:(?i:a)|)+",
            r"(?:a|)+b",
        ] {
            assert!(
                RegExpUnicodeRepeatedAtomMatcher::compile_unrepresentable_finite_nullable_alternatives_with_work(
                    &JsString::from(source),
                    true,
                    false,
                    false,
                    false,
                    |_| Ok::<(), ()>(()),
                )
                .unwrap()
                .is_none(),
                "{source}"
            );
        }
        for source in [
            r"((?:\b(?:\ba))|){999999999999999999999999,1000000000000000000000000}",
            r"((?:)|(?:\B^(?:.$)$)){000999999999999999999999999,1000000000000000000000000}?",
            r"\B(((?:\p{Join_C})|)){18446744073709551616,18446744073709551617}\B$",
            r"((?:[a\|])|(?:)){999999999999999999999999,1000000000000000000000002}?",
            r"(|\$){18446744073709551616,00036893488147419103231}?",
        ] {
            let p = plan(source, true, true, true, true);
            assert!(p.nullable.is_some());
            assert!(RegExpUnicodeRepeatedAtomMatcher::compile_positive_exact_nullable_alternatives_with_work(
                &JsString::from(source), true, true, true, true, |_| Ok::<(), ()>(()),
            ).unwrap().is_none(), "represented exact proof {source}");
            assert!(RegExpUnicodeRepeatedAtomMatcher::compile_positive_open_nullable_alternatives_with_work(
                &JsString::from(source), true, true, true, true, |_| Ok::<(), ()>(()),
            ).unwrap().is_none(), "open proof {source}");
            assert!(RegExpUnicodeRepeatedAtomMatcher::compile_one_minimum_open_nullable_alternatives_with_work(
                &JsString::from(source), true, true, true, true, |_| Ok::<(), ()>(()),
            ).unwrap().is_none(), "open-count proof {source}");
            assert!(RegExpUnicodeRepeatedAtomMatcher::compile_zero_minimum_nullable_alternatives_with_work(
                &JsString::from(source), true, true, true, true, |_| Ok::<(), ()>(()),
            ).unwrap().is_none(), "zero-lower-bound proof {source}");
            assert!(RegExpUnicodeRepeatedAtomMatcher::compile_noncapturing_branch_nested_assertion_words_with_work(
                &JsString::from(source), true, true, true, true, |_| Ok::<(), ()>(()),
            ).unwrap().is_none(), "old proof {source}");
        }
    }

    #[test]
    fn unicode_unrepresentable_finite_nullable_alternative_all_work_callbacks_can_abort() {
        let source = JsString::from(
            format!(
                "\\B{}a|{}{{999999999999999999999999,1000000000000000000000000}}?\\B",
                "(".repeat(24),
                ")".repeat(24)
            )
            .as_str(),
        );
        let mut calls = 0;
        RegExpUnicodeRepeatedAtomMatcher::compile_unrepresentable_finite_nullable_alternatives_with_work(
            &source,
            true,
            false,
            false,
            false,
            |_| {
                calls += 1;
                Ok::<(), ()>(())
            },
        )
        .unwrap()
        .unwrap();
        for fail in 0..calls {
            let mut at = 0;
            assert!(matches!(
                RegExpUnicodeRepeatedAtomMatcher::compile_unrepresentable_finite_nullable_alternatives_with_work(
                    &source,
                    true,
                    false,
                    false,
                    false,
                    |_| {
                        let selected = at == fail;
                        at += 1;
                        if selected { Err(()) } else { Ok(()) }
                    },
                ),
                Err(())
            ));
        }
        for source in [
            r"\B(a|){999999999999999999999999,1000000000000000000000000}?\B",
            r"\B(|a){18446744073709551616,18446744073709551618}\B",
            r"\B(|a){999999999999999999999999,1000000000000000000000000}$",
        ] {
            let p = plan(source, true, false, false, false);
            let input = JsString::from("baaa");
            let mut calls = 0;
            p.find_with_captures_and_work(&input, 0, false, |_| {
                calls += 1;
                Ok::<(), ()>(())
            })
            .unwrap();
            for fail in 0..calls {
                let mut at = 0;
                assert_eq!(
                    p.find_with_captures_and_work(&input, 0, false, |_| {
                        let selected = at == fail;
                        at += 1;
                        if selected { Err(()) } else { Ok(()) }
                    }),
                    Err(())
                );
            }
        }
    }

    #[test]
    fn decimal_optional_budget_subtraction_preserves_borrow_and_overflow() {
        for first in 0usize..128 {
            for last in first..256 {
                for zeros in [0, 4] {
                    let a = JsString::from(format!("{}{first}", "0".repeat(zeros)).as_str());
                    let b = JsString::from(format!("{last}").as_str());
                    assert_eq!(
                        decimal_difference(
                            significant(a.code_units()),
                            significant(b.code_units())
                        ),
                        Some(last - first)
                    );
                }
            }
        }
        for (first, last, expected) in [
            (
                "999999999999999999999999",
                "1000000000000000000000000",
                Some(1),
            ),
            (
                "999999999999999999999999",
                "1000000000000000000000002",
                Some(3),
            ),
            (
                "18446744073709551616",
                "36893488147419103231",
                usize::try_from(u64::MAX).ok(),
            ),
            ("18446744073709551616", "36893488147419103232", None),
            (
                "999999999999999999999999",
                "1999999999999999999999999",
                None,
            ),
        ] {
            assert_eq!(
                decimal_difference(
                    JsString::from(first).code_units(),
                    JsString::from(last).code_units()
                ),
                expected
            );
        }
        let lower = JsString::from(format!("{}999", "1".repeat(6000)).as_str());
        let upper = JsString::from(format!("{}2000", "1".repeat(5999)).as_str());
        assert_eq!(
            decimal_difference(lower.code_units(), upper.code_units()),
            Some(1)
        );
    }
}

#[cfg(test)]
mod overflowing_maximum_nullable_alternative_tests {
    use super::*;
    use std::fmt::Write;

    fn plan(source: &str, v: bool, i: bool, m: bool, s: bool) -> RegExpUnicodeRepeatedAtomMatcher {
        RegExpUnicodeRepeatedAtomMatcher::compile_overflowing_maximum_nullable_alternatives_with_work(
            &JsString::from(source),
            v,
            i,
            m,
            s,
            |_| Ok::<(), ()>(()),
        )
        .unwrap()
        .unwrap()
    }

    // A small, independent RepeatMatcher interpreter. Its optional empty
    // iterations are rejected before continuation; mandatory empties are kept.
    #[allow(clippy::too_many_arguments)]
    fn repeat(
        input: &[u8],
        at: usize,
        minimum: usize,
        maximum: usize,
        greedy: bool,
        empty_first: bool,
        end: bool,
        capture: Option<Range<usize>>,
    ) -> Option<(usize, Option<Range<usize>>)> {
        let continuation = || {
            if minimum == 0 && (!end || at == input.len()) {
                Some((at, capture.clone()))
            } else {
                None
            }
        };
        if !greedy {
            if let Some(found) = continuation() {
                return Some(found);
            }
        }
        if maximum != 0 {
            for empty in [empty_first, !empty_first] {
                let next = if empty {
                    at
                } else if input.get(at) == Some(&b'a') {
                    at + 1
                } else {
                    continue;
                };
                if minimum == 0 && next == at {
                    continue;
                }
                if let Some(found) = repeat(
                    input,
                    next,
                    minimum.saturating_sub(1),
                    maximum - 1,
                    greedy,
                    empty_first,
                    end,
                    Some(at..next),
                ) {
                    return Some(found);
                }
            }
        }
        if greedy { continuation() } else { None }
    }

    #[test]
    fn unicode_overflowing_maximum_nullable_alternative_repeat_matcher_snapshot() {
        let mut rows = String::new();
        let mut audited = 0;
        for empty_first in [false, true] {
            // Only the finite upper source bound exceeds usize. The independent
            // reference retains each positive minimum and enough optional copies
            // for every physical input in this exhaustive domain.
            for (quantifier, minimum, maximum, greedy) in [
                ("{1,999999999999999999999999}", 1, None, true),
                ("{1,999999999999999999999999}?", 1, None, false),
                ("{3,999999999999999999999999}", 3, None, true),
                ("{3,999999999999999999999999}?", 3, None, false),
                ("{0003,000999999999999999999999999}", 3, None, true),
                ("{0003,000999999999999999999999999}?", 3, None, false),
                ("{7,18446744073709551616}", 7, None, true),
                ("{7,18446744073709551616}?", 7, None, false),
            ] {
                for start_anchor in [false, true] {
                    for end_anchor in [false, true] {
                        let source = format!(
                            "{}({}){}{}",
                            if start_anchor { "^" } else { "" },
                            if empty_first { "|a" } else { "a|" },
                            quantifier,
                            if end_anchor { "$" } else { "" }
                        );
                        let p = plan(&source, false, false, false, false);
                        assert_eq!(p.minimum, Some(minimum));
                        assert!(p.maximum.is_none());
                        assert_eq!(p.greedy, greedy);
                        for input in [
                            "", "a", "b", "aa", "ab", "ba", "bb", "aaa", "aba", "baa", "aab",
                            "bbb", "aaaa", "aaab", "baaa", "abab", "baba",
                        ] {
                            for start in 0..=input.len() + 1 {
                                for sticky in [false, true] {
                                    let expected = (start..=input.len()).find_map(|at| {
                                        if (sticky && at != start) || (start_anchor && at != 0) {
                                            return None;
                                        }
                                        repeat(
                                            input.as_bytes(),
                                            at,
                                            minimum,
                                            maximum.unwrap_or(input.len() + minimum),
                                            greedy,
                                            empty_first,
                                            end_anchor,
                                            None,
                                        )
                                        .map(|(end, capture)| (at..end, capture))
                                    });
                                    let actual = p
                                        .find_with_captures_and_work(
                                            &JsString::from(input),
                                            start,
                                            sticky,
                                            |_| Ok::<(), ()>(()),
                                        )
                                        .unwrap();
                                    let actual = actual.map(|m| (m.range, m.atom_capture));
                                    assert_eq!(
                                        actual, expected,
                                        "{source:?} {input:?} {start} {sticky}"
                                    );
                                    audited += 1;
                                    if start == 0 && !sticky && ["", "aaa", "baaa"].contains(&input)
                                    {
                                        writeln!(rows, "{source:?} {input:?} {actual:?}").unwrap();
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        writeln!(rows, "{audited} independent RepeatMatcher comparisons").unwrap();
        insta::assert_snapshot!(rows);
    }

    #[test]
    fn unicode_overflowing_maximum_nullable_alternative_proof_stays_complete_and_narrow() {
        for source in [r"(a|)+", r"(|a)+?", r"(a|)*", r"(?:(?:)|a)?"] {
            let p = plan(source, false, false, false, false);
            assert!(
                p.find_with_captures_and_work(&JsString::from("aaa"), 0, false, |_| Ok::<(), ()>(
                    ()
                ))
                .unwrap()
                .is_some(),
                "{source}"
            );
        }
        let lower = usize::MAX.to_string();
        let upper = ((usize::MAX as u128) + 1).to_string();
        for empty_first in [false, true] {
            let source = format!(
                "({}){{{lower},{upper}}}",
                if empty_first { "|a" } else { "a|" }
            );
            let p = plan(&source, false, false, false, false);
            let m = p
                .find_with_captures_and_work(&JsString::from("aaa"), 0, false, |_| Ok::<(), ()>(()))
                .unwrap()
                .unwrap();
            assert_eq!(
                (m.range, m.atom_capture),
                if empty_first {
                    (0..1, Some(0..1))
                } else {
                    (0..3, Some(3..3))
                }
            );
        }
        for source in [
            r"(a|b|)+",
            r"(||a)+",
            r"(|)+",
            r"(a|(?:\b))+",
            r"((a)|)+",
            r"(a|){1}",
            r"(a|){2,3}",
            r"(a|){0,999999999999999999999999}",
            r"(a|){3,}",
            r"(a|){999999999999999999999999,1000000000000000000000000}",
            r"(ab|)+",
            r"(a+|)+",
            r"((?:a|b)|)+",
            r"(?:(?i:a)|)+",
            r"(?:a|)+b",
        ] {
            assert!(
                RegExpUnicodeRepeatedAtomMatcher::compile_overflowing_maximum_nullable_alternatives_with_work(
                    &JsString::from(source),
                    true,
                    false,
                    false,
                    false,
                    |_| Ok::<(), ()>(()),
                )
                .unwrap()
                .is_none(),
                "{source}"
            );
        }
        for source in [
            r"((?:\b(?:\ba))|){3,999999999999999999999999}",
            r"((?:)|(?:\B^(?:.$)$)){0003,999999999999999999999999}?",
            r"\B(((?:\p{Join_C})|)){1,18446744073709551616}\B$",
            r"((?:[a\|])|(?:)){7,999999999999999999999999}?",
            r"(|\$){3,000999999999999999999999999}?",
        ] {
            let p = plan(source, true, true, true, true);
            assert!(p.nullable.is_some());
            assert!(RegExpUnicodeRepeatedAtomMatcher::compile_positive_exact_nullable_alternatives_with_work(
                &JsString::from(source), true, true, true, true, |_| Ok::<(), ()>(()),
            ).unwrap().is_none(), "represented exact proof {source}");
            assert!(RegExpUnicodeRepeatedAtomMatcher::compile_positive_open_nullable_alternatives_with_work(
                &JsString::from(source), true, true, true, true, |_| Ok::<(), ()>(()),
            ).unwrap().is_none(), "open proof {source}");
            assert!(RegExpUnicodeRepeatedAtomMatcher::compile_one_minimum_open_nullable_alternatives_with_work(
                &JsString::from(source), true, true, true, true, |_| Ok::<(), ()>(()),
            ).unwrap().is_none(), "open-count proof {source}");
            assert!(RegExpUnicodeRepeatedAtomMatcher::compile_zero_minimum_nullable_alternatives_with_work(
                &JsString::from(source), true, true, true, true, |_| Ok::<(), ()>(()),
            ).unwrap().is_none(), "zero-lower-bound proof {source}");
            assert!(RegExpUnicodeRepeatedAtomMatcher::compile_noncapturing_branch_nested_assertion_words_with_work(
                &JsString::from(source), true, true, true, true, |_| Ok::<(), ()>(()),
            ).unwrap().is_none(), "old proof {source}");
        }
    }

    #[test]
    fn unicode_overflowing_maximum_nullable_alternative_all_work_callbacks_can_abort() {
        let source = JsString::from(
            format!(
                "\\B{}a|{}{{3,999999999999999999999999}}?\\B",
                "(".repeat(24),
                ")".repeat(24)
            )
            .as_str(),
        );
        let mut calls = 0;
        RegExpUnicodeRepeatedAtomMatcher::compile_overflowing_maximum_nullable_alternatives_with_work(
            &source,
            true,
            false,
            false,
            false,
            |_| {
                calls += 1;
                Ok::<(), ()>(())
            },
        )
        .unwrap()
        .unwrap();
        for fail in 0..calls {
            let mut at = 0;
            assert!(matches!(
                RegExpUnicodeRepeatedAtomMatcher::compile_overflowing_maximum_nullable_alternatives_with_work(
                    &source,
                    true,
                    false,
                    false,
                    false,
                    |_| {
                        let selected = at == fail;
                        at += 1;
                        if selected { Err(()) } else { Ok(()) }
                    },
                ),
                Err(())
            ));
        }
        for source in [
            r"\B(a|){3,999999999999999999999999}?\B",
            r"\B(|a){3,18446744073709551616}\B",
            r"\B(|a){3,999999999999999999999999}$",
        ] {
            let p = plan(source, true, false, false, false);
            let input = JsString::from("baaa");
            let mut calls = 0;
            p.find_with_captures_and_work(&input, 0, false, |_| {
                calls += 1;
                Ok::<(), ()>(())
            })
            .unwrap();
            for fail in 0..calls {
                let mut at = 0;
                assert_eq!(
                    p.find_with_captures_and_work(&input, 0, false, |_| {
                        let selected = at == fail;
                        at += 1;
                        if selected { Err(()) } else { Ok(()) }
                    }),
                    Err(())
                );
            }
        }
    }
}

#[cfg(test)]
mod noncapturing_empty_sequence_nullable_alternative_tests {
    use super::*;
    use std::fmt::Write;

    fn plan(source: &str, v: bool, i: bool, m: bool, s: bool) -> RegExpUnicodeRepeatedAtomMatcher {
        RegExpUnicodeRepeatedAtomMatcher::compile_noncapturing_empty_sequence_nullable_alternatives_with_work(
            &JsString::from(source),
            v,
            i,
            m,
            s,
            |_| Ok::<(), ()>(()),
        )
        .unwrap()
        .unwrap()
    }

    // A small, independent RepeatMatcher interpreter. Its optional empty
    // iterations are rejected before continuation; mandatory empties are kept.
    #[allow(clippy::too_many_arguments)]
    fn repeat(
        input: &[u8],
        at: usize,
        minimum: usize,
        maximum: usize,
        greedy: bool,
        empty_first: bool,
        end: bool,
        capture: Option<Range<usize>>,
    ) -> Option<(usize, Option<Range<usize>>)> {
        let continuation = || {
            if minimum == 0 && (!end || at == input.len()) {
                Some((at, capture.clone()))
            } else {
                None
            }
        };
        if !greedy {
            if let Some(found) = continuation() {
                return Some(found);
            }
        }
        if maximum != 0 {
            for empty in [empty_first, !empty_first] {
                let next = if empty {
                    at
                } else if input.get(at) == Some(&b'a') {
                    at + 1
                } else {
                    continue;
                };
                if minimum == 0 && next == at {
                    continue;
                }
                if let Some(found) = repeat(
                    input,
                    next,
                    minimum.saturating_sub(1),
                    maximum - 1,
                    greedy,
                    empty_first,
                    end,
                    Some(at..next),
                ) {
                    return Some(found);
                }
            }
        }
        if greedy { continuation() } else { None }
    }

    #[test]
    fn unicode_noncapturing_empty_sequence_nullable_alternative_repeat_matcher_snapshot() {
        let mut rows = String::new();
        let mut audited = 0;
        for empty_first in [false, true] {
            for empty in ["(?:)(?:)", "(?:(?:)(?:))", "(?:)(?:(?:)(?:))"] {
                for (quantifier, minimum, maximum, greedy) in [
                    ("*", 0, None, true),
                    ("*?", 0, None, false),
                    ("+", 1, None, true),
                    ("+?", 1, None, false),
                    ("?", 0, Some(1), true),
                    ("??", 0, Some(1), false),
                    ("{0,3}", 0, Some(3), true),
                    ("{0,3}?", 0, Some(3), false),
                    ("{1,}", 1, None, true),
                    ("{1,}?", 1, None, false),
                    ("{1}", 1, Some(1), true),
                    ("{1}?", 1, Some(1), false),
                    ("{3,}", 3, None, true),
                    ("{3,}?", 3, None, false),
                    ("{3}", 3, Some(3), true),
                    ("{3}?", 3, Some(3), false),
                    ("{3,6}", 3, Some(6), true),
                    ("{3,6}?", 3, Some(6), false),
                    ("{999999999999999999999999}", 7, Some(7), true),
                    ("{999999999999999999999999}?", 7, Some(7), false),
                    (
                        "{999999999999999999999999,1000000000000000000000002}",
                        7,
                        Some(10),
                        true,
                    ),
                    (
                        "{999999999999999999999999,1000000000000000000000002}?",
                        7,
                        Some(10),
                        false,
                    ),
                    ("{3,999999999999999999999999}", 3, None, true),
                    ("{3,999999999999999999999999}?", 3, None, false),
                    ("{999999999999999999999999,}", 7, None, true),
                    ("{999999999999999999999999,}?", 7, None, false),
                ] {
                    for start_anchor in [false, true] {
                        for end_anchor in [false, true] {
                            let source = format!(
                                "{}({}){}{}",
                                if start_anchor { "^" } else { "" },
                                if empty_first {
                                    format!("{empty}|a")
                                } else {
                                    format!("a|{empty}")
                                },
                                quantifier,
                                if end_anchor { "$" } else { "" }
                            );
                            let p = plan(&source, false, false, false, false);
                            for input in [
                                "", "a", "b", "aa", "ab", "ba", "bb", "aaa", "aba", "baa", "aab",
                                "bbb", "aaaa", "aaab", "baaa", "abab", "baba",
                            ] {
                                for start in 0..=input.len() + 1 {
                                    for sticky in [false, true] {
                                        let expected = (start..=input.len()).find_map(|at| {
                                            if (sticky && at != start) || (start_anchor && at != 0)
                                            {
                                                return None;
                                            }
                                            repeat(
                                                input.as_bytes(),
                                                at,
                                                minimum,
                                                maximum.unwrap_or(input.len() + minimum),
                                                greedy,
                                                empty_first,
                                                end_anchor,
                                                None,
                                            )
                                            .map(|(end, capture)| (at..end, capture))
                                        });
                                        let actual = p
                                            .find_with_captures_and_work(
                                                &JsString::from(input),
                                                start,
                                                sticky,
                                                |_| Ok::<(), ()>(()),
                                            )
                                            .unwrap();
                                        let actual = actual.map(|m| (m.range, m.atom_capture));
                                        assert_eq!(
                                            actual, expected,
                                            "{source:?} {input:?} {start} {sticky}"
                                        );
                                        audited += 1;
                                        if start == 0
                                            && !sticky
                                            && ["", "aaa", "baaa"].contains(&input)
                                        {
                                            writeln!(rows, "{source:?} {input:?} {actual:?}")
                                                .unwrap();
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        writeln!(rows, "{audited} independent RepeatMatcher comparisons").unwrap();
        insta::assert_snapshot!(rows);
    }

    #[test]
    fn unicode_noncapturing_empty_sequence_nullable_alternative_contracts_remain_complete() {
        for source in [
            r"(a|(?:)(?:))*",
            r"((?:(?:)(?:))|a){1}",
            r"\B((?:[a\|])|(?:)(?:)){3,6}?\B",
            r"((?:\b(?:\ba))|(?:(?:)(?:))){3,999999999999999999999999}",
        ] {
            let p = plan(source, true, true, true, true);
            assert!(p.nullable.is_some());
            assert!(RegExpUnicodeRepeatedAtomMatcher::compile_noncapturing_empty_alternatives_with_work(
                &JsString::from(source), true, true, true, true, |_| Ok::<(), ()>(()),
            ).unwrap().is_none(), "nested-only proof {source}");
            assert!(RegExpUnicodeRepeatedAtomMatcher::compile_overflowing_maximum_nullable_alternatives_with_work(
                &JsString::from(source), true, true, true, true, |_| Ok::<(), ()>(()),
            ).unwrap().is_none(), "prior empty-branch proof {source}");
        }
        for source in [
            r"(a|(?:)(()))+",
            r"(a|(?:)(?:\b))+",
            r"(a|(?:)(?:b))+",
            r"(a|(?:)(?:)*)+",
            r"(a|(?:)(?:)|b)+",
            r"(ab|(?:)(?:))+",
            r"((a)|(?:)(?:))+",
            r"(?:a|(?:)(?:))+b",
            r"(?:(?i:a)|(?:)(?:))+",
            r"(a|(?=)(?:))+",
        ] {
            assert!(RegExpUnicodeRepeatedAtomMatcher::compile_noncapturing_empty_sequence_nullable_alternatives_with_work(
                &JsString::from(source), true, false, false, false, |_| Ok::<(), ()>(()),
            ).unwrap().is_none(), "{source}");
        }
    }

    #[test]
    fn unicode_noncapturing_empty_sequence_nullable_alternative_all_work_callbacks_can_abort() {
        let source = JsString::from(
            format!(
                "\\B{}a|(?:)(?:(?:)(?:)){}{{3,999999999999999999999999}}?\\B",
                "(".repeat(24),
                ")".repeat(24)
            )
            .as_str(),
        );
        let mut calls = 0;
        RegExpUnicodeRepeatedAtomMatcher::compile_noncapturing_empty_sequence_nullable_alternatives_with_work(
            &source,
            true,
            false,
            false,
            false,
            |_| {
                calls += 1;
                Ok::<(), ()>(())
            },
        )
        .unwrap()
        .unwrap();
        for fail in 0..calls {
            let mut at = 0;
            assert!(matches!(
                RegExpUnicodeRepeatedAtomMatcher::compile_noncapturing_empty_sequence_nullable_alternatives_with_work(
                    &source,
                    true,
                    false,
                    false,
                    false,
                    |_| {
                        let selected = at == fail;
                        at += 1;
                        if selected { Err(()) } else { Ok(()) }
                    },
                ),
                Err(())
            ));
        }
        for source in [
            r"\B(a|(?:)(?:)){3,999999999999999999999999}?\B",
            r"\B((?:(?:)(?:))|a){3,18446744073709551616}\B",
            r"\B((?:(?:)(?:))|a){3,999999999999999999999999}$",
        ] {
            let p = plan(source, true, false, false, false);
            let input = JsString::from("baaa");
            let mut calls = 0;
            p.find_with_captures_and_work(&input, 0, false, |_| {
                calls += 1;
                Ok::<(), ()>(())
            })
            .unwrap();
            for fail in 0..calls {
                let mut at = 0;
                assert_eq!(
                    p.find_with_captures_and_work(&input, 0, false, |_| {
                        let selected = at == fail;
                        at += 1;
                        if selected { Err(()) } else { Ok(()) }
                    }),
                    Err(())
                );
            }
        }
    }
}

#[cfg(test)]
mod quantified_empty_sequence_nullable_alternative_tests {
    use super::*;
    use std::fmt::Write;

    fn plan(source: &str, v: bool, i: bool, m: bool, s: bool) -> RegExpUnicodeRepeatedAtomMatcher {
        RegExpUnicodeRepeatedAtomMatcher::compile_quantified_empty_sequence_nullable_alternatives_with_work(
            &JsString::from(source),
            v,
            i,
            m,
            s,
            |_| Ok::<(), ()>(()),
        )
        .unwrap()
        .unwrap()
    }

    // A small, independent RepeatMatcher interpreter. Its optional empty
    // iterations are rejected before continuation; mandatory empties are kept.
    #[allow(clippy::too_many_arguments)]
    fn repeat(
        input: &[u8],
        at: usize,
        minimum: usize,
        maximum: usize,
        greedy: bool,
        empty_first: bool,
        end: bool,
        capture: Option<Range<usize>>,
    ) -> Option<(usize, Option<Range<usize>>)> {
        let continuation = || {
            if minimum == 0 && (!end || at == input.len()) {
                Some((at, capture.clone()))
            } else {
                None
            }
        };
        if !greedy {
            if let Some(found) = continuation() {
                return Some(found);
            }
        }
        if maximum != 0 {
            for empty in [empty_first, !empty_first] {
                let next = if empty {
                    at
                } else if input.get(at) == Some(&b'a') {
                    at + 1
                } else {
                    continue;
                };
                if minimum == 0 && next == at {
                    continue;
                }
                if let Some(found) = repeat(
                    input,
                    next,
                    minimum.saturating_sub(1),
                    maximum - 1,
                    greedy,
                    empty_first,
                    end,
                    Some(at..next),
                ) {
                    return Some(found);
                }
            }
        }
        if greedy { continuation() } else { None }
    }

    #[test]
    fn unicode_quantified_empty_sequence_nullable_alternative_repeat_matcher_snapshot() {
        let mut rows = String::new();
        let mut audited = 0;
        for empty_first in [false, true] {
            for empty in [
                "(?:)*(?:)+?",
                "(?:(?:)?(?:){0,3}?){3}",
                "(?:){999999999999999999999999}",
                "(?:(?:){999999999999999999999999,1000000000000000000000002})+?",
                "(?:){000,003}?(?:){999999999999999999999999,}",
            ] {
                for (quantifier, minimum, maximum, greedy) in [
                    ("*", 0, None, true),
                    ("*?", 0, None, false),
                    ("+", 1, None, true),
                    ("+?", 1, None, false),
                    ("?", 0, Some(1), true),
                    ("??", 0, Some(1), false),
                    ("{0,3}", 0, Some(3), true),
                    ("{0,3}?", 0, Some(3), false),
                    ("{1,}", 1, None, true),
                    ("{1,}?", 1, None, false),
                    ("{1}", 1, Some(1), true),
                    ("{1}?", 1, Some(1), false),
                    ("{3,}", 3, None, true),
                    ("{3,}?", 3, None, false),
                    ("{3}", 3, Some(3), true),
                    ("{3}?", 3, Some(3), false),
                    ("{3,6}", 3, Some(6), true),
                    ("{3,6}?", 3, Some(6), false),
                    ("{999999999999999999999999}", 7, Some(7), true),
                    ("{999999999999999999999999}?", 7, Some(7), false),
                    (
                        "{999999999999999999999999,1000000000000000000000002}",
                        7,
                        Some(10),
                        true,
                    ),
                    (
                        "{999999999999999999999999,1000000000000000000000002}?",
                        7,
                        Some(10),
                        false,
                    ),
                    ("{3,999999999999999999999999}", 3, None, true),
                    ("{3,999999999999999999999999}?", 3, None, false),
                    ("{999999999999999999999999,}", 7, None, true),
                    ("{999999999999999999999999,}?", 7, None, false),
                ] {
                    for start_anchor in [false, true] {
                        for end_anchor in [false, true] {
                            let source = format!(
                                "{}({}){}{}",
                                if start_anchor { "^" } else { "" },
                                if empty_first {
                                    format!("{empty}|a")
                                } else {
                                    format!("a|{empty}")
                                },
                                quantifier,
                                if end_anchor { "$" } else { "" }
                            );
                            let p = plan(&source, false, false, false, false);
                            for input in [
                                "", "a", "b", "aa", "ab", "ba", "bb", "aaa", "aba", "baa", "aab",
                                "bbb", "aaaa", "aaab", "baaa", "abab", "baba",
                            ] {
                                for start in 0..=input.len() + 1 {
                                    for sticky in [false, true] {
                                        let expected = (start..=input.len()).find_map(|at| {
                                            if (sticky && at != start) || (start_anchor && at != 0)
                                            {
                                                return None;
                                            }
                                            repeat(
                                                input.as_bytes(),
                                                at,
                                                minimum,
                                                maximum.unwrap_or(input.len() + minimum),
                                                greedy,
                                                empty_first,
                                                end_anchor,
                                                None,
                                            )
                                            .map(|(end, capture)| (at..end, capture))
                                        });
                                        let actual = p
                                            .find_with_captures_and_work(
                                                &JsString::from(input),
                                                start,
                                                sticky,
                                                |_| Ok::<(), ()>(()),
                                            )
                                            .unwrap();
                                        let actual = actual.map(|m| (m.range, m.atom_capture));
                                        assert_eq!(
                                            actual, expected,
                                            "{source:?} {input:?} {start} {sticky}"
                                        );
                                        audited += 1;
                                        if start == 0
                                            && !sticky
                                            && ["", "aaa", "baaa"].contains(&input)
                                        {
                                            writeln!(rows, "{source:?} {input:?} {actual:?}")
                                                .unwrap();
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        writeln!(rows, "{audited} independent RepeatMatcher comparisons").unwrap();
        insta::assert_snapshot!(rows);
    }

    #[test]
    fn unicode_quantified_empty_sequence_nullable_alternative_contracts_remain_complete() {
        for source in [
            r"(a|(?:)*(?:)+?)*",
            r"((?:(?:){0,3}?){3}|a){1}",
            r"\B((?:[a\|])|(?:){000,003}?(?:){999999999999999999999999,}){3,6}?\B",
            r"((?:\b(?:\ba))|(?:){999999999999999999999999}){3,999999999999999999999999}",
        ] {
            let p = plan(source, true, true, true, true);
            assert!(p.nullable.is_some());
            assert!(RegExpUnicodeRepeatedAtomMatcher::compile_noncapturing_empty_sequence_nullable_alternatives_with_work(
                &JsString::from(source), true, true, true, true, |_| Ok::<(), ()>(()),
            ).unwrap().is_none(), "prior unquantified proof {source}");
        }
        for source in [
            r"(a|(?:)()*)+",
            r"(a|(?:)(?:\b)*)+",
            r"(a|(?:)(?:b){0})+",
            r"(a|(?:)*|b)+",
            r"(ab|(?:)*)+",
            r"((a)|(?:)*)+",
            r"(?:a|(?:)*)+b",
            r"(?:(?i:a)|(?:)*)+",
            r"(a|(?=)*)+",
            r"(a|(?:){2,1})+",
            r"(a|(?:){999999999999999999999999,3})+",
            r"(a|(?:)*{3})+",
            r"(a|(?:)*??)+",
            r"(a|(?:){})+",
            r"(a|(?:){,3})+",
            r"(a|(?:){3,,4})+",
            r"(a|(?:)*\b)+",
        ] {
            assert!(RegExpUnicodeRepeatedAtomMatcher::compile_quantified_empty_sequence_nullable_alternatives_with_work(
                &JsString::from(source), true, false, false, false, |_| Ok::<(), ()>(()),
            ).unwrap().is_none(), "{source}");
        }
    }

    #[test]
    fn unicode_quantified_empty_sequence_nullable_alternative_all_work_callbacks_can_abort() {
        let source = JsString::from(
            format!(
                "\\B{}a|(?:)*(?:(?:)*(?:)+?){}{{3,999999999999999999999999}}?\\B",
                "(".repeat(24),
                ")".repeat(24)
            )
            .as_str(),
        );
        let mut calls = 0;
        RegExpUnicodeRepeatedAtomMatcher::compile_quantified_empty_sequence_nullable_alternatives_with_work(
            &source,
            true,
            false,
            false,
            false,
            |_| {
                calls += 1;
                Ok::<(), ()>(())
            },
        )
        .unwrap()
        .unwrap();
        for fail in 0..calls {
            let mut at = 0;
            assert!(matches!(
                RegExpUnicodeRepeatedAtomMatcher::compile_quantified_empty_sequence_nullable_alternatives_with_work(
                    &source,
                    true,
                    false,
                    false,
                    false,
                    |_| {
                        let selected = at == fail;
                        at += 1;
                        if selected { Err(()) } else { Ok(()) }
                    },
                ),
                Err(())
            ));
        }
        for source in [
            r"\B(a|(?:)*(?:)+?){3,999999999999999999999999}?\B",
            r"\B((?:(?:)*(?:)+?)|a){3,18446744073709551616}\B",
            r"\B((?:(?:)*(?:)+?)|a){3,999999999999999999999999}$",
        ] {
            let p = plan(source, true, false, false, false);
            let input = JsString::from("baaa");
            let mut calls = 0;
            p.find_with_captures_and_work(&input, 0, false, |_| {
                calls += 1;
                Ok::<(), ()>(())
            })
            .unwrap();
            for fail in 0..calls {
                let mut at = 0;
                assert_eq!(
                    p.find_with_captures_and_work(&input, 0, false, |_| {
                        let selected = at == fail;
                        at += 1;
                        if selected { Err(()) } else { Ok(()) }
                    }),
                    Err(())
                );
            }
        }
    }
}

#[cfg(test)]
mod empty_choice_nullable_alternative_tests {
    use super::*;
    use std::fmt::Write;

    fn plan(source: &str, v: bool, i: bool, m: bool, s: bool) -> RegExpUnicodeRepeatedAtomMatcher {
        RegExpUnicodeRepeatedAtomMatcher::compile_empty_choice_nullable_alternatives_with_work(
            &JsString::from(source),
            v,
            i,
            m,
            s,
            |_| Ok::<(), ()>(()),
        )
        .unwrap()
        .unwrap()
    }

    // A small, independent RepeatMatcher interpreter. Its optional empty
    // iterations are rejected before continuation; mandatory empties are kept.
    #[allow(clippy::too_many_arguments)]
    fn repeat(
        input: &[u8],
        at: usize,
        minimum: usize,
        maximum: usize,
        greedy: bool,
        empty_first: bool,
        end: bool,
        capture: Option<Range<usize>>,
    ) -> Option<(usize, Option<Range<usize>>)> {
        let continuation = || {
            if minimum == 0 && (!end || at == input.len()) {
                Some((at, capture.clone()))
            } else {
                None
            }
        };
        if !greedy {
            if let Some(found) = continuation() {
                return Some(found);
            }
        }
        if maximum != 0 {
            for empty in [empty_first, !empty_first] {
                let next = if empty {
                    at
                } else if input.get(at) == Some(&b'a') {
                    at + 1
                } else {
                    continue;
                };
                if minimum == 0 && next == at {
                    continue;
                }
                if let Some(found) = repeat(
                    input,
                    next,
                    minimum.saturating_sub(1),
                    maximum - 1,
                    greedy,
                    empty_first,
                    end,
                    Some(at..next),
                ) {
                    return Some(found);
                }
            }
        }
        if greedy { continuation() } else { None }
    }

    #[test]
    fn unicode_empty_choice_nullable_alternative_repeat_matcher_snapshot() {
        let mut rows = String::new();
        let mut audited = 0;
        for empty_first in [false, true] {
            for empty in [
                "(?:|)",
                "(?:||(?:))",
                "(?:(?:)*|(?:)+?)",
                "(?:(?:|(?:)){999999999999999999999999})",
                "(?:(?:|)|(?:(?:)*|(?:){0,3}?)){999999999999999999999999,}",
            ] {
                for (quantifier, minimum, maximum, greedy) in [
                    ("*", 0, None, true),
                    ("*?", 0, None, false),
                    ("+", 1, None, true),
                    ("+?", 1, None, false),
                    ("?", 0, Some(1), true),
                    ("??", 0, Some(1), false),
                    ("{0,3}", 0, Some(3), true),
                    ("{0,3}?", 0, Some(3), false),
                    ("{1,}", 1, None, true),
                    ("{1,}?", 1, None, false),
                    ("{1}", 1, Some(1), true),
                    ("{1}?", 1, Some(1), false),
                    ("{3,}", 3, None, true),
                    ("{3,}?", 3, None, false),
                    ("{3}", 3, Some(3), true),
                    ("{3}?", 3, Some(3), false),
                    ("{3,6}", 3, Some(6), true),
                    ("{3,6}?", 3, Some(6), false),
                    ("{999999999999999999999999}", 7, Some(7), true),
                    ("{999999999999999999999999}?", 7, Some(7), false),
                    (
                        "{999999999999999999999999,1000000000000000000000002}",
                        7,
                        Some(10),
                        true,
                    ),
                    (
                        "{999999999999999999999999,1000000000000000000000002}?",
                        7,
                        Some(10),
                        false,
                    ),
                    ("{3,999999999999999999999999}", 3, None, true),
                    ("{3,999999999999999999999999}?", 3, None, false),
                    ("{999999999999999999999999,}", 7, None, true),
                    ("{999999999999999999999999,}?", 7, None, false),
                ] {
                    for start_anchor in [false, true] {
                        for end_anchor in [false, true] {
                            let source = format!(
                                "{}({}){}{}",
                                if start_anchor { "^" } else { "" },
                                if empty_first {
                                    format!("{empty}|a")
                                } else {
                                    format!("a|{empty}")
                                },
                                quantifier,
                                if end_anchor { "$" } else { "" }
                            );
                            let p = plan(&source, false, false, false, false);
                            for input in [
                                "", "a", "b", "aa", "ab", "ba", "bb", "aaa", "aba", "baa", "aab",
                                "bbb", "aaaa", "aaab", "baaa", "abab", "baba",
                            ] {
                                for start in 0..=input.len() + 1 {
                                    for sticky in [false, true] {
                                        let expected = (start..=input.len()).find_map(|at| {
                                            if (sticky && at != start) || (start_anchor && at != 0)
                                            {
                                                return None;
                                            }
                                            repeat(
                                                input.as_bytes(),
                                                at,
                                                minimum,
                                                maximum.unwrap_or(input.len() + minimum),
                                                greedy,
                                                empty_first,
                                                end_anchor,
                                                None,
                                            )
                                            .map(|(end, capture)| (at..end, capture))
                                        });
                                        let actual = p
                                            .find_with_captures_and_work(
                                                &JsString::from(input),
                                                start,
                                                sticky,
                                                |_| Ok::<(), ()>(()),
                                            )
                                            .unwrap();
                                        let actual = actual.map(|m| (m.range, m.atom_capture));
                                        assert_eq!(
                                            actual, expected,
                                            "{source:?} {input:?} {start} {sticky}"
                                        );
                                        audited += 1;
                                        if start == 0
                                            && !sticky
                                            && ["", "aaa", "baaa"].contains(&input)
                                        {
                                            writeln!(rows, "{source:?} {input:?} {actual:?}")
                                                .unwrap();
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        writeln!(rows, "{audited} independent RepeatMatcher comparisons").unwrap();
        insta::assert_snapshot!(rows);
    }

    #[test]
    fn unicode_empty_choice_nullable_alternative_contracts_remain_complete() {
        for source in [
            r"(a|(?:|))*",
            r"((?:||(?:))|a){1}",
            r"\B((?:[a\|])|(?:(?:)*|(?:){999999999999999999999999})){3,6}?\B",
            r"((?:\b(?:\ba))|(?:(?:|(?:)){999999999999999999999999,})){3,999999999999999999999999}",
        ] {
            let p = plan(source, true, true, true, true);
            assert!(p.nullable.is_some());
            assert!(RegExpUnicodeRepeatedAtomMatcher::compile_quantified_empty_sequence_nullable_alternatives_with_work(
                &JsString::from(source), true, true, true, true, |_| Ok::<(), ()>(()),
            ).unwrap().is_none(), "prior no-choice proof {source}");
        }
        for source in [
            r"(a|(?:|()))+",
            r"(a|(?:|\b))+",
            r"(a|(?:|b){0})+",
            r"(a|(?:|)|b)+",
            r"(ab|(?:|))+",
            r"((a)|(?:|))+",
            r"(?:a|(?:|))+b",
            r"(?:(?i:a)|(?:|))+",
            r"(a|(?i:|))+",
            r"(a|(?:|){2,1})+",
            r"(a|(?:|){999999999999999999999999,3})+",
            r"(a|(?:|)*{3})+",
            r"(a|(?:|)*??)+",
            r"(a|(?:|){})+",
            r"(a|(?:|){,3})+",
            r"(a|(?:|){3,,4})+",
            r"(a|(?:|)*\b)+",
        ] {
            assert!(RegExpUnicodeRepeatedAtomMatcher::compile_empty_choice_nullable_alternatives_with_work(
                &JsString::from(source), true, false, false, false, |_| Ok::<(), ()>(()),
            ).unwrap().is_none(), "{source}");
        }
    }

    #[test]
    fn unicode_empty_choice_nullable_alternative_all_work_callbacks_can_abort() {
        let source = JsString::from(
            format!(
                "\\B{}a|(?:)*(?:(?:(?:)*|(?:)+?)){}{{3,999999999999999999999999}}?\\B",
                "(".repeat(24),
                ")".repeat(24)
            )
            .as_str(),
        );
        let mut calls = 0;
        RegExpUnicodeRepeatedAtomMatcher::compile_empty_choice_nullable_alternatives_with_work(
            &source,
            true,
            false,
            false,
            false,
            |_| {
                calls += 1;
                Ok::<(), ()>(())
            },
        )
        .unwrap()
        .unwrap();
        for fail in 0..calls {
            let mut at = 0;
            assert!(matches!(
                RegExpUnicodeRepeatedAtomMatcher::compile_empty_choice_nullable_alternatives_with_work(
                    &source,
                    true,
                    false,
                    false,
                    false,
                    |_| {
                        let selected = at == fail;
                        at += 1;
                        if selected { Err(()) } else { Ok(()) }
                    },
                ),
                Err(())
            ));
        }
        for source in [
            r"\B(a|(?:(?:)*|(?:)+?)){3,999999999999999999999999}?\B",
            r"\B((?:(?:(?:)*|(?:)+?))|a){3,18446744073709551616}\B",
            r"\B((?:(?:(?:)*|(?:)+?))|a){3,999999999999999999999999}$",
        ] {
            let p = plan(source, true, false, false, false);
            let input = JsString::from("baaa");
            let mut calls = 0;
            p.find_with_captures_and_work(&input, 0, false, |_| {
                calls += 1;
                Ok::<(), ()>(())
            })
            .unwrap();
            for fail in 0..calls {
                let mut at = 0;
                assert_eq!(
                    p.find_with_captures_and_work(&input, 0, false, |_| {
                        let selected = at == fail;
                        at += 1;
                        if selected { Err(()) } else { Ok(()) }
                    }),
                    Err(())
                );
            }
        }
    }
}

#[cfg(test)]
#[path = "regexp_unicode_repeated_single_character_string_operands_tests.rs"]
mod single_character_string_operands_repetition_tests;

#[cfg(test)]
#[path = "regexp_unicode_wrapped_single_character_string_repetition_tests.rs"]
mod wrapped_single_character_string_repetition_tests;

#[cfg(test)]
#[path = "regexp_unicode_raw_empty_single_character_string_repetition_tests.rs"]
mod raw_empty_single_character_string_repetition_tests;

#[cfg(test)]
#[path = "regexp_unicode_scalar_left_string_repetition_tests.rs"]
mod scalar_left_string_repetition_tests;
