//! Complete negative Unicode lookahead over one character (22.2.2.4).

use crate::{
    JsString, RegExpUnicodePositiveAtomMatcher, regexp_literal::unicode_start,
    regexp_zero_count_group::enclosing_group_body,
};
use std::ops::Range;

/// A complete negative Unicode lookahead over one unquantified character atom.
///
/// Full original u/v Pattern validation and normalized capture heads are
/// required. Complete ordinary/noncapturing wrappers may surround the assertion
/// and operand. Optional operand line gates retain original input context.
/// Outside captures select the empty match; operand captures remain undefined
/// on success (22.2.2.4). Other counts and references need other proofs.
#[derive(Clone, Debug)]
pub struct RegExpUnicodeNegativeAtomMatcher {
    positive: RegExpUnicodePositiveAtomMatcher,
    outside_captures: usize,
}

impl RegExpUnicodeNegativeAtomMatcher {
    /// Compiles complete validated negative lookahead over one character atom.
    ///
    /// Changing only the proved assertion sign preserves original lexical
    /// bindings. The positive single-atom proof rejects reference syntax before
    /// projection; projected source is never separately validated or rebound.
    pub fn compile_single_atom_validated_with_work<E>(
        source: &JsString,
        unicode_sets: bool,
        ignore_case: bool,
        multiline: bool,
        dot_all: bool,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, E> {
        Self::compile_with_work::<false, E>(
            source,
            unicode_sets,
            ignore_case,
            multiline,
            dot_all,
            &mut charge,
        )
    }

    /// Compiles negative lookahead with operand word predicates and line gates.
    ///
    /// Full original u/v validation and normalized capture heads are required.
    /// At least one leading/trailing word predicate surrounds the proved
    /// character atom. Both predicates inspect original input context, while
    /// successful negation leaves every operand capture undefined (22.2.2.4).
    pub fn compile_single_atom_with_word_boundaries_validated_with_work<E>(
        source: &JsString,
        unicode_sets: bool,
        ignore_case: bool,
        multiline: bool,
        dot_all: bool,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, E> {
        Self::compile_with_work::<true, E>(
            source,
            unicode_sets,
            ignore_case,
            multiline,
            dot_all,
            &mut charge,
        )
    }

    #[inline(never)]
    fn compile_with_work<const WORD: bool, E>(
        source: &JsString,
        unicode_sets: bool,
        ignore_case: bool,
        multiline: bool,
        dot_all: bool,
        charge: &mut dyn FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, E> {
        charge(source.len().saturating_mul(16).saturating_add(16))?;
        let units = source.code_units();
        let (assertion, outside_captures) =
            if let Some(scope) = enclosing_group_body(units, unicode_sets) {
                (scope.body, scope.captures)
            } else {
                (0..units.len(), 0)
            };
        if !units[assertion.clone()].starts_with(&[0x28, 0x3f, 0x21]) {
            return Ok(None);
        }
        let mut positive = units.to_vec();
        positive[assertion.start + 2] = 0x3d;
        let source = JsString::from_code_units(positive);
        let positive = if WORD {
            RegExpUnicodePositiveAtomMatcher::compile_single_atom_with_word_boundaries_validated_with_work(
                &source,
                unicode_sets,
                ignore_case,
                multiline,
                dot_all,
                charge,
            )?
        } else {
            RegExpUnicodePositiveAtomMatcher::compile_single_atom_validated_with_work(
                &source,
                unicode_sets,
                ignore_case,
                multiline,
                dot_all,
                charge,
            )?
        };
        Ok(positive.map(|positive| Self {
            positive,
            outside_captures,
        }))
    }

    /// Full original lexical count, including undefined operand captures.
    pub fn capture_count(&self) -> usize {
        self.positive.capture_count()
    }

    /// Searches complete Unicode candidates, succeeding when the operand fails.
    ///
    /// One fixed-count sticky probe bounds each candidate's actual work; a
    /// failed negative assertion never searches the remaining suffix on its own.
    /// Initial trailing-pair offsets use the approved leading-boundary policy.
    pub fn find_with_work<E>(
        &self,
        input: &JsString,
        start: usize,
        sticky: bool,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Range<usize>>, E> {
        self.find_impl_with_work(input, start, sticky, &mut charge)
    }

    #[inline(never)]
    fn find_impl_with_work<E>(
        &self,
        input: &JsString,
        start: usize,
        sticky: bool,
        mut charge: &mut dyn FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Range<usize>>, E> {
        charge(1)?;
        let Some(mut candidate) = unicode_start(input, start) else {
            return Ok(None);
        };
        let units = input.code_units();
        loop {
            if self
                .positive
                .find_with_work(input, candidate, true, &mut charge)?
                .is_none()
            {
                return Ok(Some(candidate..candidate));
            }
            if sticky || candidate == units.len() {
                return Ok(None);
            }
            charge(3)?;
            candidate += if (0xd800..=0xdbff).contains(&units[candidate])
                && units
                    .get(candidate + 1)
                    .is_some_and(|tail| (0xdc00..=0xdfff).contains(tail))
            {
                2
            } else {
                1
            };
        }
    }

    /// Outside captures select the empty match; all operand slots are undefined.
    pub fn capture_range(&self, index: usize, matched: &Range<usize>) -> Option<Range<usize>> {
        (index < self.outside_captures).then(|| matched.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn plan(
        source: &str,
        sets: bool,
        i: bool,
        m: bool,
        s: bool,
    ) -> RegExpUnicodeNegativeAtomMatcher {
        RegExpUnicodeNegativeAtomMatcher::compile_single_atom_validated_with_work(
            &JsString::from(source),
            sets,
            i,
            m,
            s,
            |_| Ok::<(), ()>(()),
        )
        .unwrap()
        .unwrap()
    }

    fn find(
        p: &RegExpUnicodeNegativeAtomMatcher,
        text: &str,
        start: usize,
        sticky: bool,
    ) -> Option<Range<usize>> {
        p.find_with_work(&JsString::from(text), start, sticky, |_| Ok::<(), ()>(()))
            .unwrap()
    }

    #[test]
    fn negative_lookahead_keeps_outside_empty_and_operand_captures_undefined() {
        for sets in [false, true] {
            let p = plan("((?!((a))))", sets, false, false, false);
            assert_eq!(p.capture_count(), 3);
            let m = find(&p, "aab", 0, false).unwrap();
            assert_eq!(m, 2..2);
            assert_eq!(p.capture_range(0, &m), Some(2..2));
            for index in 1..=4 {
                assert_eq!(p.capture_range(index, &m), None);
            }
            assert_eq!(find(&p, "aab", 0, true), None);
            assert_eq!(find(&p, "aab", 3, true), Some(3..3));
            assert_eq!(find(&p, "aab", 4, false), None);
            assert_eq!(find(&p, "", 0, true), Some(0..0));
        }
    }

    #[test]
    fn negative_lookahead_retains_original_line_dot_fold_and_class_context() {
        for sets in [false, true] {
            let p = plan(r"(?!(^a$))", sets, true, true, false);
            assert_eq!(find(&p, "\nA\n", 1, true), None);
            assert_eq!(find(&p, "!A!", 1, true), Some(1..1));
            let p = plan(r"(?!(.))", sets, false, false, false);
            assert_eq!(find(&p, "a\nb", 0, false), Some(1..1));
            let p = plan(r"(?!(.))", sets, false, false, true);
            assert_eq!(find(&p, "a\nb", 0, false), Some(3..3));
            let p = plan(r"(?!(\p{ASCII}))", sets, true, false, false);
            assert_eq!(find(&p, "ſK!", 0, false), Some(3..3));
            let p = plan(r"(?!(\p{ASCII}))", sets, false, false, false);
            assert_eq!(find(&p, "ſK!", 0, false), Some(0..0));
            let p = plan(r"(?!([\b]))", sets, false, false, false);
            assert_eq!(find(&p, "\u{8}a", 0, false), Some(1..1));
        }
    }

    #[test]
    fn negative_lookahead_uses_complete_characters_and_initial_pair_rewind() {
        for sets in [false, true] {
            let p = plan(r"(?!(😀))", sets, false, false, false);
            assert_eq!(find(&p, "😀😀a", 0, false), Some(4..4));
            assert_eq!(find(&p, "😀a", 1, true), None);
            assert_eq!(find(&p, "😀a", 2, true), Some(2..2));
            let p = plan(r"(?!(\uDE00))", sets, false, false, false);
            assert_eq!(find(&p, "😀", 1, true), Some(0..0));
            let p = plan(r"(?!(\uD800))", sets, false, false, false);
            let input = JsString::from_code_units(vec![0xd800, 0x61]);
            assert_eq!(
                p.find_with_work(&input, 0, false, |_| Ok::<(), ()>(()))
                    .unwrap(),
                Some(1..1)
            );
        }
    }

    #[test]
    fn negative_lookahead_declines_counts_references_continuations_and_other_assertions() {
        for source in [
            "(?=(a))",
            "(?<!(a))",
            "(?!(a*))",
            "(?!(a{1}))",
            "(?!(a|b))",
            "(?!(ab))",
            r"(?!((a)\1))",
            r"((?!(\1)))",
            r"(?!((a)))\1",
            "(?!((a)))*",
            "(?i:(?!(a)))",
            r"(?!(\ba))",
            "(?!((a)))|b",
            r"(?!(\u{D83D}\u{DE00}))",
        ] {
            assert!(
                RegExpUnicodeNegativeAtomMatcher::compile_single_atom_validated_with_work(
                    &JsString::from(source),
                    false,
                    false,
                    false,
                    false,
                    |_| Ok::<(), ()>(())
                )
                .unwrap()
                .is_none(),
                "{source}"
            );
        }
    }

    #[test]
    fn negative_lookahead_deep_wrappers_and_failing_candidate_work_stay_flat() {
        let source = format!(
            "{}(?!{}a{}){}",
            "(".repeat(100_000),
            "(".repeat(100_000),
            ")".repeat(100_000),
            ")".repeat(100_000)
        );
        let p = plan(&source, false, false, false, false);
        assert_eq!(p.capture_count(), 200_000);
        let m = find(&p.clone(), "b", 0, true).unwrap();
        assert_eq!(p.capture_range(99_999, &m), Some(0..0));
        assert_eq!(p.capture_range(100_000, &m), None);
        let p = plan("(?!(a))", false, false, false, false);
        let input = JsString::from("a".repeat(20_000).as_str());
        let mut work = 0usize;
        assert_eq!(
            p.find_with_work(&input, 0, false, |n| {
                work += n;
                Ok::<(), ()>(())
            })
            .unwrap(),
            Some(20_000..20_000)
        );
        assert!(work < 120 * input.len() + 64, "{work}");
    }

    #[test]
    fn negative_lookahead_charges_before_work_and_recovers_from_each_cancellation() {
        let source = JsString::from("((?!((a))))");
        let mut work = Vec::new();
        let p = RegExpUnicodeNegativeAtomMatcher::compile_single_atom_validated_with_work(
            &source,
            false,
            false,
            false,
            false,
            |n| {
                work.push(n);
                Ok::<(), usize>(())
            },
        )
        .unwrap()
        .unwrap();
        assert_eq!(work[0], source.len() * 16 + 16);
        for stop in 0..work.len() {
            let mut seen = Vec::new();
            assert!(
                matches!(RegExpUnicodeNegativeAtomMatcher::compile_single_atom_validated_with_work(&source, false, false, false, false, |n| {seen.push(n); if seen.len() == stop + 1 {Err(stop)} else {Ok(())}}), Err(n) if n == stop)
            );
            assert_eq!(seen, work[..=stop]);
        }
        work.clear();
        let input = JsString::from("aaa");
        assert_eq!(
            p.find_with_work(&input, 0, false, |n| {
                work.push(n);
                Ok::<(), usize>(())
            })
            .unwrap(),
            Some(3..3)
        );
        for stop in 0..work.len() {
            let mut seen = Vec::new();
            assert_eq!(
                p.find_with_work(&input, 0, false, |n| {
                    seen.push(n);
                    if seen.len() == stop + 1 {
                        Err(stop)
                    } else {
                        Ok(())
                    }
                }),
                Err(stop)
            );
            assert_eq!(seen, work[..=stop]);
            assert_eq!(find(&p, "aaa", 0, false), Some(3..3));
        }
        assert_eq!(
            p.find_with_work(&input, usize::MAX, false, |_| Err(59)),
            Err(59)
        );
    }

    fn word_plan(source: &str, sets: bool, i: bool, m: bool) -> RegExpUnicodeNegativeAtomMatcher {
        RegExpUnicodeNegativeAtomMatcher::compile_single_atom_with_word_boundaries_validated_with_work(
            &JsString::from(source), sets, i, m, false, |_| Ok::<(), ()>(()),
        ).unwrap().unwrap()
    }

    #[test]
    fn negative_word_assertion_negates_both_boundaries_and_discards_operand_captures() {
        for sets in [false, true] {
            for (source, text, point) in [
                (r"((?!((\ba\b))))", "a!", 1),
                (r"((?!((\ba\B))))", "aa!", 1),
                (r"((?!((\Ba\B))))", "aaa!", 0),
                (r"((?!((\Ba\b))))", "aa!", 0),
            ] {
                let p = word_plan(source, sets, false, false);
                assert_eq!(p.capture_count(), 3);
                let m = find(&p, text, 0, false).unwrap();
                assert_eq!(m, point..point, "{source}");
                assert_eq!(p.capture_range(0, &m), Some(m.clone()));
                assert_eq!(p.capture_range(1, &m), None);
                assert_eq!(p.capture_range(2, &m), None);
            }
            let p = word_plan(r"(?!(\ba\b))", sets, false, false);
            assert_eq!(find(&p, "aa", 0, true), Some(0..0));
            assert_eq!(find(&p, "!a!", 1, true), None);
        }
    }

    #[test]
    fn negative_word_assertion_uses_original_line_fold_backspace_and_surrogate_context() {
        for sets in [false, true] {
            let p = word_plan(r"(?!(^\ba\b$))", sets, true, true);
            assert_eq!(find(&p, "\nA\n", 1, true), None);
            assert_eq!(find(&p, "!A!", 1, true), Some(1..1));
            let p = word_plan(r"(?!(\b\p{ASCII}\b))", sets, true, false);
            assert_eq!(find(&p, "!ſ!", 1, true), None);
            let p = word_plan(r"(?!(\B[\b]\B))", sets, false, false);
            assert_eq!(find(&p, "\u{8}", 0, true), None);
            assert_eq!(find(&p, "a\u{8}b", 1, true), Some(1..1));
            let p = word_plan(r"(?!(\B.\b))", sets, false, false);
            assert_eq!(find(&p, "a😀ab!", 4, true), None);
            assert_eq!(find(&p, "a😀ab!", 2, false), Some(1..1));
            let p = word_plan(r"(?!(\B😀\B))", sets, false, false);
            assert_eq!(find(&p, "😀😀", 0, false), Some(4..4));
        }
    }

    #[test]
    fn negative_word_assertion_declines_other_counts_and_unproved_scopes() {
        for source in [
            r"(?!(a))",
            r"(?!(\ba*\b))",
            r"(?!(\ba{1}\b))",
            r"(?!(\ba\bb))",
            r"(?!(\ba\b))b",
            r"(?!(\ba\b))|b",
            r"((?!(\ba\b)))*",
            r"(?!(\b(a)\b))",
            r"((?!(\b\1\b)))",
            r"(?<!(\ba\b))",
            r"(?=(\ba\b))",
            r"(?!(\b\ba\b))",
        ] {
            assert!(RegExpUnicodeNegativeAtomMatcher::compile_single_atom_with_word_boundaries_validated_with_work(&JsString::from(source), false, false, false, false, |_| Ok::<(), ()>(())).unwrap().is_none(), "{source}");
        }
    }

    #[test]
    fn negative_word_assertion_deep_scopes_and_every_work_cancellation_remain_flat() {
        let source = format!(
            "{}(?!{}\\ba\\b{}){}",
            "(".repeat(100_000),
            "(".repeat(100_000),
            ")".repeat(100_000),
            ")".repeat(100_000)
        );
        let p = word_plan(&source, false, false, false);
        assert_eq!(p.capture_count(), 200_000);
        let matched = find(&p.clone(), "!", 0, true).unwrap();
        assert_eq!(p.capture_range(99_999, &matched), Some(0..0));
        assert_eq!(p.capture_range(100_000, &matched), None);
        let source = JsString::from(r"((?!((\ba\b))))");
        let mut work = Vec::new();
        let p = RegExpUnicodeNegativeAtomMatcher::compile_single_atom_with_word_boundaries_validated_with_work(&source, false, false, false, false, |n| {work.push(n); Ok::<(), usize>(())}).unwrap().unwrap();
        assert_eq!(work[0], source.len() * 16 + 16);
        for stop in 0..work.len() {
            let mut seen = Vec::new();
            assert!(
                matches!(RegExpUnicodeNegativeAtomMatcher::compile_single_atom_with_word_boundaries_validated_with_work(&source, false, false, false, false, |n| {seen.push(n); if seen.len() == stop + 1 {Err(stop)} else {Ok(())}}), Err(n) if n == stop)
            );
            assert_eq!(seen, work[..=stop]);
        }
        let input = JsString::from("a!a");
        work.clear();
        assert_eq!(
            p.find_with_work(&input, 0, false, |n| {
                work.push(n);
                Ok::<(), usize>(())
            })
            .unwrap(),
            Some(1..1)
        );
        for stop in 0..work.len() {
            let mut seen = Vec::new();
            assert_eq!(
                p.find_with_work(&input, 0, false, |n| {
                    seen.push(n);
                    if seen.len() == stop + 1 {
                        Err(stop)
                    } else {
                        Ok(())
                    }
                }),
                Err(stop)
            );
            assert_eq!(seen, work[..=stop]);
            assert_eq!(find(&p, "a!a", 0, false), Some(1..1));
        }
    }
}
