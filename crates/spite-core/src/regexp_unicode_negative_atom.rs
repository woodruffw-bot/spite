//! Complete negative Unicode assertions over one character (22.2.2.4).

use crate::{
    JsString, RegExpUnicodePositiveAtomMatcher, regexp_literal::unicode_start,
    regexp_zero_count_group::enclosing_group_body,
};
use std::ops::Range;

/// A complete negative Unicode assertion over one unquantified character atom.
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
        Self::compile_with_work::<false, false, E>(
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
        Self::compile_with_work::<true, false, E>(
            source,
            unicode_sets,
            ignore_case,
            multiline,
            dot_all,
            &mut charge,
        )
    }

    /// Compiles complete negative lookbehind over one Unicode character atom.
    ///
    /// Full original u/v validation and normalized capture heads are required.
    /// A single non-reference character has one complete predecessor; optional
    /// line gates retain original input context. Successful negation preserves
    /// empty outside captures and leaves every operand slot undefined (22.2.2.4).
    pub fn compile_single_lookbehind_validated_with_work<E>(
        source: &JsString,
        unicode_sets: bool,
        ignore_case: bool,
        multiline: bool,
        dot_all: bool,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, E> {
        Self::compile_with_work::<false, true, E>(
            source,
            unicode_sets,
            ignore_case,
            multiline,
            dot_all,
            &mut charge,
        )
    }

    /// Compiles negative lookbehind with predecessor word predicates and line gates.
    ///
    /// Full original u/v validation and normalized capture heads are required.
    /// At least one leading/trailing word predicate surrounds one proved
    /// non-reference predecessor atom. Predicates retain original input context;
    /// all operand captures remain undefined on assertion success (22.2.2.4).
    pub fn compile_single_lookbehind_with_word_boundaries_validated_with_work<E>(
        source: &JsString,
        unicode_sets: bool,
        ignore_case: bool,
        multiline: bool,
        dot_all: bool,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, E> {
        Self::compile_with_work::<true, true, E>(
            source,
            unicode_sets,
            ignore_case,
            multiline,
            dot_all,
            &mut charge,
        )
    }

    /// Compiles negative lookahead over one v-mode class with one-character q members.
    ///
    /// Full original v/vi Pattern validation and normalized capture heads are
    /// required. Only the assertion sign is projected; the checked positive
    /// q-class proof rejects other widths, counts, gates and binding roles.
    /// Outside wrappers capture the empty assertion position and every operand
    /// slot stays undefined on success (22.2.2.4, 22.2.2.9).
    pub fn compile_single_character_string_class_lookahead_validated_with_work<E>(
        source: &JsString,
        ignore_case: bool,
        multiline: bool,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, crate::RegExpUnicodeSetCompileError<E>> {
        Self::compile_single_character_class_assertion_with_work::<false, false, false, E>(
            source,
            ignore_case,
            multiline,
            &mut charge,
        )
    }

    /// Compiles negative q-class lookahead with operand word predicates and line gates.
    ///
    /// Requires full original v/vi validation and normalized capture heads.
    /// At least one leading/trailing word predicate surrounds the checked
    /// single-character q class. Successful negation leaves all operand slots
    /// undefined while outside wrappers retain the empty assertion position
    /// (22.2.2.4, 22.2.2.9). Other widths and scopes remain separate proofs.
    pub fn compile_single_character_string_class_lookahead_with_word_boundaries_validated_with_work<
        E,
    >(
        source: &JsString,
        ignore_case: bool,
        multiline: bool,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, crate::RegExpUnicodeSetCompileError<E>> {
        Self::compile_single_character_class_assertion_with_work::<true, false, false, E>(
            source,
            ignore_case,
            multiline,
            &mut charge,
        )
    }

    /// Compiles negative lookbehind over one v-mode class with one-character q members.
    ///
    /// Requires full original v/vi validation and normalized capture heads.
    /// Optional operand line gates inspect original input context. Successful
    /// negation keeps outside wrappers empty and all operand slots undefined;
    /// the checked class proof establishes one complete predecessor character
    /// (22.2.2.4, 22.2.2.9). Other widths and word gates need separate proofs.
    pub fn compile_single_character_string_class_lookbehind_validated_with_work<E>(
        source: &JsString,
        ignore_case: bool,
        multiline: bool,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, crate::RegExpUnicodeSetCompileError<E>> {
        Self::compile_single_character_class_assertion_with_work::<false, true, false, E>(
            source,
            ignore_case,
            multiline,
            &mut charge,
        )
    }

    /// Compiles negative q-class lookbehind with operand word predicates and line gates.
    ///
    /// Full original v/vi validation and normalized capture heads are required.
    /// At least one leading/trailing word predicate surrounds the checked
    /// single-character predecessor. Successful negation keeps outside wrappers
    /// empty and all operand slots undefined (22.2.2.4, 22.2.2.9). Other widths,
    /// counts and scopes remain separate proofs.
    pub fn compile_single_character_string_class_lookbehind_with_word_boundaries_validated_with_work<
        E,
    >(
        source: &JsString,
        ignore_case: bool,
        multiline: bool,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, crate::RegExpUnicodeSetCompileError<E>> {
        Self::compile_single_character_class_assertion_with_work::<true, true, false, E>(
            source,
            ignore_case,
            multiline,
            &mut charge,
        )
    }

    /// Compiles negative lookahead over one checked scalar v-mode set class.
    ///
    /// Full original v/vi validation and normalized capture heads are required.
    /// A nested/operator class must prove character-valued leaves. Optional
    /// operand ^/$ gates retain original input context. Successful negation
    /// leaves outside wrappers empty and every operand slot undefined
    /// (22.2.2.4, 22.2.2.9). Other widths and word gates need separate proofs.
    pub fn compile_single_v_set_lookahead_validated_with_work<E>(
        source: &JsString,
        ignore_case: bool,
        multiline: bool,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, crate::RegExpUnicodeSetCompileError<E>> {
        Self::compile_single_character_class_assertion_with_work::<false, false, true, E>(
            source,
            ignore_case,
            multiline,
            &mut charge,
        )
    }

    #[inline(never)]
    fn compile_single_character_class_assertion_with_work<
        const WORD: bool,
        const BACKWARD: bool,
        const SCALAR_SET: bool,
        E,
    >(
        source: &JsString,
        ignore_case: bool,
        multiline: bool,
        charge: &mut dyn FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, crate::RegExpUnicodeSetCompileError<E>> {
        charge(source.len().saturating_mul(16).saturating_add(16))
            .map_err(crate::RegExpUnicodeSetCompileError::Work)?;
        let units = source.code_units();
        let (assertion, outside_captures) = if let Some(scope) = enclosing_group_body(units, true) {
            (scope.body, scope.captures)
        } else {
            (0..units.len(), 0)
        };
        let prefix: &[u16] = if BACKWARD {
            &[0x28, 0x3f, 0x3c, 0x21]
        } else {
            &[0x28, 0x3f, 0x21]
        };
        if !units[assertion.clone()].starts_with(prefix) {
            return Ok(None);
        }
        let mut positive = units.to_vec();
        positive[assertion.start + prefix.len() - 1] = 0x3d;
        let source = JsString::from_code_units(positive);
        let positive = if SCALAR_SET {
            if BACKWARD {
                if WORD {
                    RegExpUnicodePositiveAtomMatcher::compile_single_v_set_lookbehind_with_word_boundaries_validated_with_work(
                        &source, ignore_case, multiline, charge,
                    )?
                } else {
                    RegExpUnicodePositiveAtomMatcher::compile_single_v_set_lookbehind_validated_with_work(
                        &source, ignore_case, multiline, charge,
                    )?
                }
            } else if WORD {
                RegExpUnicodePositiveAtomMatcher::compile_single_v_set_lookahead_with_word_boundaries_validated_with_work(
                    &source, ignore_case, multiline, charge,
                )?
            } else {
                RegExpUnicodePositiveAtomMatcher::compile_single_v_set_lookahead_validated_with_work(
                    &source, ignore_case, multiline, charge,
                )?
            }
        } else if BACKWARD {
            if WORD {
                RegExpUnicodePositiveAtomMatcher::compile_single_character_string_class_lookbehind_with_word_boundaries_validated_with_work(
                    &source,
                    ignore_case,
                    multiline,
                    charge,
                )?
            } else {
                RegExpUnicodePositiveAtomMatcher::compile_single_character_string_class_lookbehind_validated_with_work(
                    &source,
                    ignore_case,
                    multiline,
                    charge,
                )?
            }
        } else if WORD {
            RegExpUnicodePositiveAtomMatcher::compile_single_character_string_class_lookahead_with_word_boundaries_validated_with_work(
                &source,
                ignore_case,
                multiline,
                charge,
            )?
        } else {
            RegExpUnicodePositiveAtomMatcher::compile_single_character_string_class_lookahead_validated_with_work(
                &source,
                ignore_case,
                multiline,
                charge,
            )?
        };
        Ok(positive.map(|positive| Self {
            positive,
            outside_captures,
        }))
    }

    #[inline(never)]
    fn compile_with_work<const WORD: bool, const BACKWARD: bool, E>(
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
        let prefix: &[u16] = if BACKWARD {
            &[0x28, 0x3f, 0x3c, 0x21]
        } else {
            &[0x28, 0x3f, 0x21]
        };
        if !units[assertion.clone()].starts_with(prefix) {
            return Ok(None);
        }
        let mut positive = units.to_vec();
        positive[assertion.start + prefix.len() - 1] = 0x3d;
        let source = JsString::from_code_units(positive);
        let positive = if BACKWARD {
            if WORD {
                RegExpUnicodePositiveAtomMatcher::compile_single_lookbehind_with_word_boundaries_validated_with_work(
                    &source,
                    unicode_sets,
                    ignore_case,
                    multiline,
                    dot_all,
                    charge,
                )?
            } else {
                RegExpUnicodePositiveAtomMatcher::compile_single_lookbehind_validated_with_work(
                    &source,
                    unicode_sets,
                    ignore_case,
                    multiline,
                    dot_all,
                    charge,
                )?
            }
        } else if WORD {
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

    fn behind_plan(
        source: &str,
        sets: bool,
        i: bool,
        m: bool,
        s: bool,
    ) -> RegExpUnicodeNegativeAtomMatcher {
        RegExpUnicodeNegativeAtomMatcher::compile_single_lookbehind_validated_with_work(
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

    #[test]
    fn negative_lookbehind_negates_predecessors_and_keeps_operand_slots_undefined() {
        for sets in [false, true] {
            let p = behind_plan("((?<!((a))))", sets, false, false, false);
            assert_eq!(p.capture_count(), 3);
            let matched = find(&p, "aab", 1, false).unwrap();
            assert_eq!(matched, 3..3);
            assert_eq!(p.capture_range(0, &matched), Some(3..3));
            for index in 1..=3 {
                assert_eq!(p.capture_range(index, &matched), None);
            }
            assert_eq!(find(&p, "aab", 0, true), Some(0..0));
            assert_eq!(find(&p, "aab", 1, true), None);
            assert_eq!(find(&p, "aaa", 1, false), None);
            assert_eq!(find(&p, "", 0, true), Some(0..0));
            assert_eq!(find(&p, "aab", 4, false), None);
        }
    }

    #[test]
    fn negative_lookbehind_retains_original_line_dot_and_folded_property_context() {
        for sets in [false, true] {
            let p = behind_plan(r"(?<!(^a$))", sets, true, true, false);
            assert_eq!(find(&p, "\nA\n", 2, true), None);
            assert_eq!(find(&p, "!A!", 2, true), Some(2..2));
            let p = behind_plan(r"(?<!(.))", sets, false, false, false);
            assert_eq!(find(&p, "a\n", 1, false), Some(2..2));
            let p = behind_plan(r"(?<!(.))", sets, false, false, true);
            assert_eq!(find(&p, "a\n", 2, true), None);
            for i in [false, true] {
                let p = behind_plan(r"(?<!(\p{ASCII}))", sets, i, false, false);
                assert_eq!(find(&p, "ſK!", 1, true), if i { None } else { Some(1..1) });
            }
        }
    }

    #[test]
    fn negative_lookbehind_uses_one_complete_predecessor_and_initial_pair_rewind() {
        for sets in [false, true] {
            let p = behind_plan(r"(?<!(😀))", sets, false, false, false);
            assert_eq!(find(&p, "😀😀a", 2, false), Some(5..5));
            assert_eq!(find(&p, "😀😀a", 3, true), None);
            assert_eq!(find(&p, "😀a", 1, true), Some(0..0));
            let p = behind_plan(r"(?<!(\uDE00))", sets, false, false, false);
            assert_eq!(find(&p, "😀", 2, true), Some(2..2));
            let input = JsString::from_code_units(vec![0xd800, 0x78, 0xde00]);
            assert!(
                p.find_with_work(&input, 3, true, |_| Ok::<(), ()>(()))
                    .unwrap()
                    .is_none()
            );
        }
    }

    #[test]
    fn negative_lookbehind_declines_other_counts_references_and_unproved_scopes() {
        for source in [
            r"(?<=(a))",
            r"(?!(a))",
            r"(?<!(a*))",
            r"(?<!(a{1}))",
            r"(?<!(ab))",
            r"(?<!(a|b))",
            r"(?<!(\ba))",
            r"(?<!(a))b",
            r"(?<!(a))|b",
            r"((?<!(a)))*",
            r"(?i:(?<!(a)))",
            r"((?<!(\1)))",
            r"(?<!(\u{D83D}\u{DE00}))",
        ] {
            assert!(
                RegExpUnicodeNegativeAtomMatcher::compile_single_lookbehind_validated_with_work(
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
    fn negative_lookbehind_deep_scopes_linear_retries_and_work_abort_recovery_stay_flat() {
        let source = format!(
            "{}(?<!{}a{}){}",
            "(".repeat(100_000),
            "(".repeat(100_000),
            ")".repeat(100_000),
            ")".repeat(100_000)
        );
        let p = behind_plan(&source, false, false, false, false);
        assert_eq!(p.capture_count(), 200_000);
        let matched = find(&p.clone(), "b", 1, true).unwrap();
        assert_eq!(p.capture_range(99_999, &matched), Some(1..1));
        assert_eq!(p.capture_range(100_000, &matched), None);
        let p = behind_plan("(?<!(a))", false, false, false, false);
        let input = JsString::from("a".repeat(20_000).as_str());
        let mut total = 0usize;
        assert!(
            p.find_with_work(&input, 1, false, |n| {
                total += n;
                Ok::<(), ()>(())
            })
            .unwrap()
            .is_none()
        );
        assert!(total < 150 * input.len() + 64, "{total}");
        let source = JsString::from("((?<!((a))))");
        let mut work = Vec::new();
        let p = RegExpUnicodeNegativeAtomMatcher::compile_single_lookbehind_validated_with_work(
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
                matches!(RegExpUnicodeNegativeAtomMatcher::compile_single_lookbehind_validated_with_work(&source, false, false, false, false, |n| {seen.push(n); if seen.len() == stop + 1 {Err(stop)} else {Ok(())}}), Err(n) if n == stop)
            );
            assert_eq!(seen, work[..=stop]);
        }
        let input = JsString::from("aab");
        work.clear();
        assert_eq!(
            p.find_with_work(&input, 1, false, |n| {
                work.push(n);
                Ok::<(), usize>(())
            })
            .unwrap(),
            Some(3..3)
        );
        for stop in 0..work.len() {
            let mut seen = Vec::new();
            assert_eq!(
                p.find_with_work(&input, 1, false, |n| {
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
            assert_eq!(find(&p, "aab", 1, false), Some(3..3));
        }
    }

    fn behind_word_plan(
        source: &str,
        sets: bool,
        i: bool,
        m: bool,
    ) -> RegExpUnicodeNegativeAtomMatcher {
        RegExpUnicodeNegativeAtomMatcher::compile_single_lookbehind_with_word_boundaries_validated_with_work(
            &JsString::from(source), sets, i, m, false, |_| Ok::<(), ()>(()),
        ).unwrap().unwrap()
    }

    #[test]
    fn negative_word_lookbehind_negates_the_predecessor_and_both_predicates() {
        for sets in [false, true] {
            let p = behind_word_plan(r"((?<!((\ba\b))))", sets, false, false);
            assert_eq!(p.capture_count(), 3);
            assert_eq!(find(&p, "a!", 1, true), None);
            let m = find(&p, "a!", 1, false).unwrap();
            assert_eq!(m, 2..2);
            assert_eq!(p.capture_range(0, &m), Some(2..2));
            for index in 1..=3 {
                assert_eq!(p.capture_range(index, &m), None);
            }
            assert_eq!(find(&p, "a!", 0, true), Some(0..0));
            assert_eq!(find(&p, "aa", 1, true), Some(1..1));
            let p = behind_word_plan(r"(?<!(\ba\B))", sets, false, false);
            assert_eq!(find(&p, "aa!", 1, true), None);
            assert_eq!(find(&p, "a!", 1, true), Some(1..1));
        }
    }

    #[test]
    fn negative_word_lookbehind_retains_original_context_and_complete_predecessors() {
        for sets in [false, true] {
            let p = behind_word_plan(r"(?<!(^\ba\b$))", sets, true, true);
            assert_eq!(find(&p, "\nA\n", 2, true), None);
            assert_eq!(find(&p, "!A!", 2, true), Some(2..2));
            let p = behind_word_plan(r"(?<!(\b\p{ASCII}\b))", sets, true, false);
            assert_eq!(find(&p, "!ſ!", 2, true), None);
            let p = behind_word_plan(r"(?<!(\B[\b]\B))", sets, false, false);
            assert_eq!(find(&p, "\u{8}", 1, true), None);
            assert_eq!(find(&p, "a\u{8}b", 2, true), Some(2..2));
            let p = behind_word_plan(r"(?<!(\B.\b))", sets, false, false);
            assert_eq!(find(&p, "a😀ab!", 5, true), None);
            assert_eq!(find(&p, "a😀ab!", 4, true), Some(4..4));
            assert_eq!(find(&p, "a😀ab!", 2, true), Some(1..1));
            let p = behind_word_plan(r"(?<!(\B😀\B))", sets, false, false);
            assert_eq!(find(&p, "😀😀", 2, false), None);
            assert_eq!(find(&p, "😀😀", 1, true), Some(0..0));
        }
    }

    #[test]
    fn negative_word_lookbehind_declines_other_counts_and_unproved_scopes() {
        for source in [
            r"(?<!(a))",
            r"(?<!(\ba*\b))",
            r"(?<!(\ba{1}\b))",
            r"(?<!(\ba\bb))",
            r"(?<!(\ba\b))b",
            r"(?<!(\ba\b))|b",
            r"((?<!(\ba\b)))*",
            r"(?<!(\b(a)\b))",
            r"((?<!(\b\1\b)))",
            r"(?!(\ba\b))",
            r"(?<=(\ba\b))",
            r"(?<!(\b\ba\b))",
        ] {
            assert!(RegExpUnicodeNegativeAtomMatcher::compile_single_lookbehind_with_word_boundaries_validated_with_work(&JsString::from(source), false, false, false, false, |_| Ok::<(), ()>(())).unwrap().is_none(), "{source}");
        }
    }

    #[test]
    fn negative_word_lookbehind_deep_scopes_and_every_work_cancellation_remain_flat() {
        let source = format!(
            "{}(?<!{}\\ba\\b{}){}",
            "(".repeat(100_000),
            "(".repeat(100_000),
            ")".repeat(100_000),
            ")".repeat(100_000)
        );
        let p = behind_word_plan(&source, false, false, false);
        assert_eq!(p.capture_count(), 200_000);
        let m = find(&p.clone(), "b", 1, true).unwrap();
        assert_eq!(p.capture_range(99_999, &m), Some(1..1));
        assert_eq!(p.capture_range(100_000, &m), None);
        let source = JsString::from(r"((?<!((\ba\b))))");
        let mut work = Vec::new();
        let p = RegExpUnicodeNegativeAtomMatcher::compile_single_lookbehind_with_word_boundaries_validated_with_work(&source, false, false, false, false, |n| {work.push(n); Ok::<(), usize>(())}).unwrap().unwrap();
        assert_eq!(work[0], source.len() * 16 + 16);
        for stop in 0..work.len() {
            let mut seen = Vec::new();
            assert!(
                matches!(RegExpUnicodeNegativeAtomMatcher::compile_single_lookbehind_with_word_boundaries_validated_with_work(&source, false, false, false, false, |n| {seen.push(n); if seen.len() == stop + 1 {Err(stop)} else {Ok(())}}), Err(n) if n == stop)
            );
            assert_eq!(seen, work[..=stop]);
        }
        let input = JsString::from("a!a");
        work.clear();
        assert_eq!(
            p.find_with_work(&input, 1, false, |n| {
                work.push(n);
                Ok::<(), usize>(())
            })
            .unwrap(),
            Some(2..2)
        );
        for stop in 0..work.len() {
            let mut seen = Vec::new();
            assert_eq!(
                p.find_with_work(&input, 1, false, |n| {
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
            assert_eq!(find(&p, "a!a", 1, false), Some(2..2));
        }
    }

    fn q_plan(source: &str, i: bool, m: bool) -> RegExpUnicodeNegativeAtomMatcher {
        RegExpUnicodeNegativeAtomMatcher::compile_single_character_string_class_lookahead_validated_with_work(
            &JsString::from(source),i,m, |_| Ok::<(), ()>(()),
        ).unwrap().unwrap()
    }

    #[test]
    fn negative_single_character_q_lookahead_keeps_empty_outside_and_undefined_operand_slots() {
        for class in [
            r"[\q{a|b}]",
            r"[[\q{a|b}]--[x]]",
            r"[[\q{a|b}]&&[ab]]",
            r"[x\q{a|b}]",
        ] {
            let p = q_plan(&format!("((?!(({class}))))"), false, false);
            assert_eq!(p.capture_count(), 3);
            let matched = find(&p, "ab!", 0, false).unwrap();
            assert_eq!(matched, 2..2);
            assert_eq!(p.capture_range(0, &matched), Some(2..2));
            for index in 1..=3 {
                assert_eq!(p.capture_range(index, &matched), None);
            }
            assert_eq!(find(&p, "ab!", 0, true), None);
            assert_eq!(find(&p, "ab!", 3, true), Some(3..3));
            assert_eq!(find(&p, "ab!", 4, false), None);
            assert_eq!(find(&p, "", 0, true), Some(0..0));
        }
    }

    #[test]
    fn negative_single_character_q_lookahead_keeps_original_line_fold_and_pair_context() {
        let p = q_plan(r"(?!(^[\q{a|😀}]$))", true, true);
        assert_eq!(find(&p, "\nA\n", 1, true), None);
        assert_eq!(find(&p, "!A!", 1, true), Some(1..1));
        let p = q_plan(r"(?!([\q{ſ|k}]))", true, false);
        for text in ["S", "ſ", "K", "K"] {
            assert_eq!(find(&p, text, 0, true), None);
        }
        let p = q_plan(r"(?!([\p{ASCII}\q{😀}]))", true, false);
        assert_eq!(find(&p, "ſK", 0, false), Some(2..2));
        let p = q_plan(r"(?!([\q{😀|a}]))", false, false);
        assert_eq!(find(&p, "😀😀b", 0, false), Some(4..4));
        assert_eq!(find(&p, "😀😀b", 1, true), None);
        let p = q_plan(r"(?!([\q{\uD800|x}]))", false, false);
        assert_eq!(find(&p, "😀", 0, true), Some(0..0));
        let input = JsString::from_code_units(vec![0xd800, 0x78]);
        assert_eq!(
            p.find_with_work(&input, 0, false, |_| Ok::<(), ()>(()))
                .unwrap(),
            Some(2..2)
        );
    }

    #[test]
    fn negative_single_character_q_lookahead_declines_other_domains_widths_and_scopes() {
        for source in [
            r"(?!([a]))",
            r"(?!([\q{ab|a}]))",
            r"(?!([\q{|a}]))",
            r"(?!([\q{a|b}]+))",
            r"(?!([\q{a|b}]{1}))",
            r"(?!([\q{a|b}]a))",
            r"(?!([\q{a|b}]))a",
            r"(?!([\q{a|b}]))|b",
            r"((?!([\q{a|b}])))*",
            r"(?=([\q{a|b}]))",
            r"(?<!([\q{a|b}]))",
            r"(?!(\b[\q{a|b}]\b))",
            r"(?i:(?!([\q{a|b}])))",
            r"((?!(\1)))",
        ] {
            assert!(RegExpUnicodeNegativeAtomMatcher::compile_single_character_string_class_lookahead_validated_with_work(&JsString::from(source),false,false, |_| Ok::<(), ()>(())).unwrap().is_none(),"{source}");
        }
    }

    fn word_q_plan(source: &str, i: bool, m: bool) -> RegExpUnicodeNegativeAtomMatcher {
        RegExpUnicodeNegativeAtomMatcher::compile_single_character_string_class_lookahead_with_word_boundaries_validated_with_work(
            &JsString::from(source),i,m, |_| Ok::<(), ()>(()),
        ).unwrap().unwrap()
    }

    #[test]
    fn negative_q_word_lookahead_negates_both_boundaries_and_discards_operand_slots() {
        let p = word_q_plan(r"((?!((\b[\q{a|b}]\b))))", false, false);
        assert_eq!(p.capture_count(), 3);
        assert_eq!(find(&p, "a!", 0, true), None);
        let matched = find(&p, "a!", 0, false).unwrap();
        assert_eq!(matched, 1..1);
        assert_eq!(p.capture_range(0, &matched), Some(1..1));
        for index in 1..=3 {
            assert_eq!(p.capture_range(index, &matched), None);
        }
        assert_eq!(find(&p, "aa", 0, true), Some(0..0));
        let p = word_q_plan(r"(?!(\b[\q{a|b}]\B))", false, false);
        assert_eq!(find(&p, "ab!", 0, true), None);
        assert_eq!(find(&p, "a!", 0, true), Some(0..0));
    }

    #[test]
    fn negative_q_word_lookahead_keeps_original_line_fold_backspace_and_pair_context() {
        let p = word_q_plan(r"(?!(^\b[\q{a|😀}]\b$))", true, true);
        assert_eq!(find(&p, "\nA\n", 1, true), None);
        assert_eq!(find(&p, "!A!", 1, true), Some(1..1));
        let p = word_q_plan(r"(?!(\b[\q{ſ|k}]\b))", true, false);
        for text in ["!S!", "!ſ!", "!K!", "!K!"] {
            assert_eq!(find(&p, text, 1, true), None);
        }
        let p = word_q_plan(r"(?!(\B[\q{\u0008|a}]\B))", false, false);
        assert_eq!(find(&p, "\u{8}", 0, true), None);
        assert_eq!(find(&p, "a\u{8}b", 1, true), Some(1..1));
        let p = word_q_plan(r"(?!(\B[\q{😀|a}]\B))", false, false);
        assert_eq!(find(&p, "😀😀", 0, false), Some(4..4));
        assert_eq!(find(&p, "😀😀", 1, true), None);
    }

    #[test]
    fn negative_q_word_lookahead_declines_other_domains_members_counts_and_scopes() {
        for source in [
            r"(?!([\q{a|b}]))",
            r"(?!(\b[ab]\b))",
            r"(?!(\b[\q{ab|a}]\b))",
            r"(?!(\b[\q{|a}]\b))",
            r"(?!(\b[\q{a|b}]+\b))",
            r"(?!(\b[\q{a|b}]{1}\b))",
            r"(?!(\b[\q{a|b}]\b))a",
            r"(?!(\b[\q{a|b}]\b))|b",
            r"((?!(\b[\q{a|b}]\b)))*",
            r"(?=(\b[\q{a|b}]\b))",
            r"(?<!(\b[\q{a|b}]\b))",
            r"(?!(\b([\q{a|b}])\b))",
            r"((?!(\b\1\b)))",
        ] {
            assert!(RegExpUnicodeNegativeAtomMatcher::compile_single_character_string_class_lookahead_with_word_boundaries_validated_with_work(&JsString::from(source),false,false, |_| Ok::<(), ()>(())).unwrap().is_none(),"{source}");
        }
    }

    #[test]
    fn negative_q_word_lookahead_deep_scopes_linear_retries_and_work_abort_recovery_stay_flat() {
        let source = format!(
            "{}(?!{}\\b[\\q{{a|b}}]\\b{}){}",
            "(".repeat(100_000),
            "(".repeat(100_000),
            ")".repeat(100_000),
            ")".repeat(100_000)
        );
        let p = word_q_plan(&source, false, false);
        assert_eq!(p.capture_count(), 200_000);
        let matched = find(&p.clone(), "!", 0, true).unwrap();
        assert_eq!(p.capture_range(99_999, &matched), Some(0..0));
        assert_eq!(p.capture_range(100_000, &matched), None);
        let p = word_q_plan(r"(?!(\B[\q{a|b}]\B))", false, false);
        let input = JsString::from("a".repeat(20_000).as_str());
        let mut total = 0usize;
        assert_eq!(
            p.find_with_work(&input, 1, false, |n| {
                total += n;
                Ok::<(), ()>(())
            })
            .unwrap(),
            Some(19_999..19_999)
        );
        assert!(total < 250 * input.len() + 128, "{total}");
        let source = JsString::from(r"((?!(\b[\q{a|b}]\b)))");
        let mut work = Vec::new();
        let p=RegExpUnicodeNegativeAtomMatcher::compile_single_character_string_class_lookahead_with_word_boundaries_validated_with_work(&source,false,false,|n|{work.push(n);Ok::<(),usize>(())}).unwrap().unwrap();
        assert_eq!(work[0], source.len() * 16 + 16);
        for stop in 0..work.len() {
            let mut seen = Vec::new();
            assert!(
                matches!(RegExpUnicodeNegativeAtomMatcher::compile_single_character_string_class_lookahead_with_word_boundaries_validated_with_work(&source,false,false,|n|{seen.push(n);if seen.len()==stop+1{Err(stop)}else{Ok(())}}),Err(crate::RegExpUnicodeSetCompileError::Work(n))if n==stop)
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

    fn behind_q_plan(source: &str, i: bool, m: bool) -> RegExpUnicodeNegativeAtomMatcher {
        RegExpUnicodeNegativeAtomMatcher::compile_single_character_string_class_lookbehind_validated_with_work(
            &JsString::from(source),i,m, |_| Ok::<(), ()>(()),
        ).unwrap().unwrap()
    }

    #[test]
    fn negative_q_lookbehind_negates_predecessors_and_keeps_operand_slots_undefined() {
        for class in [
            r"[\q{a|b}]",
            r"[[\q{a|b}]--[x]]",
            r"[[\q{a|b}]&&[ab]]",
            r"[x\q{a|b}]",
        ] {
            let p = behind_q_plan(&format!("((?<!(({class}))))"), false, false);
            assert_eq!(p.capture_count(), 3);
            let matched = find(&p, "ab!", 1, false).unwrap();
            assert_eq!(matched, 3..3);
            assert_eq!(p.capture_range(0, &matched), Some(3..3));
            for index in 1..=3 {
                assert_eq!(p.capture_range(index, &matched), None);
            }
            assert_eq!(find(&p, "ab!", 0, true), Some(0..0));
            assert_eq!(find(&p, "ab!", 1, true), None);
            assert_eq!(find(&p, "ab", 1, false), None);
            assert_eq!(find(&p, "", 0, true), Some(0..0));
            assert_eq!(find(&p, "ab!", 4, false), None);
        }
    }

    #[test]
    fn negative_q_lookbehind_keeps_original_line_fold_and_complete_predecessor_context() {
        let p = behind_q_plan(r"(?<!(^[\q{a|😀}]$))", true, true);
        assert_eq!(find(&p, "\nA\n", 2, true), None);
        assert_eq!(find(&p, "!A!", 2, true), Some(2..2));
        let p = behind_q_plan(r"(?<!([\q{ſ|k}]))", true, false);
        for text in ["S", "ſ", "K", "K"] {
            assert_eq!(find(&p, text, 1, true), None);
        }
        let p = behind_q_plan(r"(?<!([\p{ASCII}\q{😀}]))", true, false);
        assert_eq!(find(&p, "ſ", 1, true), None);
        let p = behind_q_plan(r"(?<!([\q{😀|a}]))", false, false);
        assert_eq!(find(&p, "😀😀b", 2, false), Some(5..5));
        assert_eq!(find(&p, "😀😀b", 3, true), None);
        assert_eq!(find(&p, "😀b", 1, true), Some(0..0));
        let p = behind_q_plan(r"(?<!([\q{\uD800|x}]))", false, false);
        assert_eq!(find(&p, "😀", 2, true), Some(2..2));
        let input = JsString::from_code_units(vec![0xd800, 0x78]);
        assert!(
            p.find_with_work(&input, 1, true, |_| Ok::<(), ()>(()))
                .unwrap()
                .is_none()
        );
    }

    #[test]
    fn negative_q_lookbehind_declines_other_domains_members_counts_and_scopes() {
        for source in [
            r"(?<!([a]))",
            r"(?<!([\q{ab|a}]))",
            r"(?<!([\q{|a}]))",
            r"(?<!([\q{a|b}]+))",
            r"(?<!([\q{a|b}]{1}))",
            r"(?<!([\q{a|b}]a))",
            r"(?<!([\q{a|b}]))a",
            r"(?<!([\q{a|b}]))|b",
            r"((?<!([\q{a|b}])))*",
            r"(?<=([\q{a|b}]))",
            r"(?!([\q{a|b}]))",
            r"(?<!(\b[\q{a|b}]\b))",
            r"(?i:(?<!([\q{a|b}])))",
            r"((?<!(\1)))",
        ] {
            assert!(RegExpUnicodeNegativeAtomMatcher::compile_single_character_string_class_lookbehind_validated_with_work(&JsString::from(source),false,false, |_| Ok::<(), ()>(())).unwrap().is_none(),"{source}");
        }
    }

    #[test]
    fn negative_q_lookbehind_deep_scopes_linear_retries_and_work_abort_recovery_stay_flat() {
        let source = format!(
            "{}(?<!{}[\\q{{a|b}}]{}){}",
            "(".repeat(100_000),
            "(".repeat(100_000),
            ")".repeat(100_000),
            ")".repeat(100_000)
        );
        let p = behind_q_plan(&source, false, false);
        assert_eq!(p.capture_count(), 200_000);
        let matched = find(&p.clone(), "!", 1, true).unwrap();
        assert_eq!(p.capture_range(99_999, &matched), Some(1..1));
        assert_eq!(p.capture_range(100_000, &matched), None);
        let p = behind_q_plan(r"(?<!([\q{a|b}]))", false, false);
        let input = JsString::from("a".repeat(20_000).as_str());
        let mut total = 0usize;
        assert!(
            p.find_with_work(&input, 1, false, |n| {
                total += n;
                Ok::<(), ()>(())
            })
            .unwrap()
            .is_none()
        );
        assert!(total < 250 * input.len() + 128, "{total}");
        let source = JsString::from(r"((?<!([\q{a|b}])))");
        let mut work = Vec::new();
        let p=RegExpUnicodeNegativeAtomMatcher::compile_single_character_string_class_lookbehind_validated_with_work(&source,false,false,|n|{work.push(n);Ok::<(),usize>(())}).unwrap().unwrap();
        assert_eq!(work[0], source.len() * 16 + 16);
        for stop in 0..work.len() {
            let mut seen = Vec::new();
            assert!(
                matches!(RegExpUnicodeNegativeAtomMatcher::compile_single_character_string_class_lookbehind_validated_with_work(&source,false,false,|n|{seen.push(n);if seen.len()==stop+1{Err(stop)}else{Ok(())}}),Err(crate::RegExpUnicodeSetCompileError::Work(n))if n==stop)
            );
            assert_eq!(seen, work[..=stop]);
        }
        let input = JsString::from("ab!");
        work.clear();
        assert_eq!(
            p.find_with_work(&input, 1, false, |n| {
                work.push(n);
                Ok::<(), usize>(())
            })
            .unwrap(),
            Some(3..3)
        );
        for stop in 0..work.len() {
            let mut seen = Vec::new();
            assert_eq!(
                p.find_with_work(&input, 1, false, |n| {
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
            assert_eq!(find(&p, "ab!", 1, false), Some(3..3));
        }
    }

    fn behind_word_q_plan(source: &str, i: bool, m: bool) -> RegExpUnicodeNegativeAtomMatcher {
        RegExpUnicodeNegativeAtomMatcher::compile_single_character_string_class_lookbehind_with_word_boundaries_validated_with_work(
            &JsString::from(source),i,m, |_| Ok::<(), ()>(()),
        ).unwrap().unwrap()
    }

    #[test]
    fn negative_q_word_lookbehind_negates_predecessors_and_keeps_operand_slots_undefined() {
        let p = behind_word_q_plan(r"((?<!((\b[\q{a|b}]\b))))", false, false);
        assert_eq!(p.capture_count(), 3);
        assert_eq!(find(&p, "a!", 1, true), None);
        let matched = find(&p, "a!", 1, false).unwrap();
        assert_eq!(matched, 2..2);
        assert_eq!(p.capture_range(0, &matched), Some(2..2));
        for index in 1..=3 {
            assert_eq!(p.capture_range(index, &matched), None);
        }
        assert_eq!(find(&p, "aa", 1, true), Some(1..1));
        assert_eq!(find(&p, "", 0, true), Some(0..0));
        let p = behind_word_q_plan(r"(?<!(\b[\q{a|b}]\B))", false, false);
        assert_eq!(find(&p, "ab!", 1, true), None);
        assert_eq!(find(&p, "a!", 1, true), Some(1..1));
    }

    #[test]
    fn negative_q_word_lookbehind_keeps_original_line_fold_backspace_and_pair_context() {
        let p = behind_word_q_plan(r"(?<!(^\b[\q{a|😀}]\b$))", true, true);
        assert_eq!(find(&p, "\nA\n", 2, true), None);
        assert_eq!(find(&p, "!A!", 2, true), Some(2..2));
        let p = behind_word_q_plan(r"(?<!(\b[\q{ſ|k}]\b))", true, false);
        for text in ["!S!", "!ſ!", "!K!", "!K!"] {
            assert_eq!(find(&p, text, 2, true), None);
        }
        let p = behind_word_q_plan(r"(?<!(\B[\q{\u0008|a}]\B))", false, false);
        assert_eq!(find(&p, "\u{8}", 1, true), None);
        assert_eq!(find(&p, "a\u{8}b", 2, true), Some(2..2));
        let p = behind_word_q_plan(r"(?<!(\B[\q{😀|a}]\B))", false, false);
        assert_eq!(find(&p, "😀😀", 2, false), None);
        assert_eq!(find(&p, "😀😀", 3, true), None);
        assert_eq!(find(&p, "😀😀", 1, true), Some(0..0));
    }

    #[test]
    fn negative_q_word_lookbehind_declines_other_domains_members_counts_and_scopes() {
        for source in [
            r"(?<!([\q{a|b}]))",
            r"(?<!(\b[ab]\b))",
            r"(?<!(\b[\q{ab|a}]\b))",
            r"(?<!(\b[\q{|a}]\b))",
            r"(?<!(\b[\q{a|b}]+\b))",
            r"(?<!(\b[\q{a|b}]{1}\b))",
            r"(?<!(\b[\q{a|b}]\b))a",
            r"(?<!(\b[\q{a|b}]\b))|b",
            r"((?<!(\b[\q{a|b}]\b)))*",
            r"(?<=(\b[\q{a|b}]\b))",
            r"(?!(\b[\q{a|b}]\b))",
            r"(?<!(\b([\q{a|b}])\b))",
            r"((?<!(\b\1\b)))",
        ] {
            assert!(RegExpUnicodeNegativeAtomMatcher::compile_single_character_string_class_lookbehind_with_word_boundaries_validated_with_work(&JsString::from(source),false,false, |_| Ok::<(), ()>(())).unwrap().is_none(),"{source}");
        }
    }

    #[test]
    fn negative_q_word_lookbehind_deep_scopes_linear_retries_and_work_abort_recovery_stay_flat() {
        let source = format!(
            "{}(?<!{}\\b[\\q{{a|b}}]\\b{}){}",
            "(".repeat(100_000),
            "(".repeat(100_000),
            ")".repeat(100_000),
            ")".repeat(100_000)
        );
        let p = behind_word_q_plan(&source, false, false);
        assert_eq!(p.capture_count(), 200_000);
        let matched = find(&p.clone(), "!", 1, true).unwrap();
        assert_eq!(p.capture_range(99_999, &matched), Some(1..1));
        assert_eq!(p.capture_range(100_000, &matched), None);
        let p = behind_word_q_plan(r"(?<!(\B[\q{a|b}]\B))", false, false);
        let input = JsString::from("a".repeat(20_000).as_str());
        let mut total = 0usize;
        assert_eq!(
            p.find_with_work(&input, 2, false, |n| {
                total += n;
                Ok::<(), ()>(())
            })
            .unwrap(),
            Some(20_000..20_000)
        );
        assert!(total < 250 * input.len() + 128, "{total}");
        let source = JsString::from(r"((?<!(\b[\q{a|b}]\b)))");
        let mut work = Vec::new();
        let p=RegExpUnicodeNegativeAtomMatcher::compile_single_character_string_class_lookbehind_with_word_boundaries_validated_with_work(&source,false,false,|n|{work.push(n);Ok::<(),usize>(())}).unwrap().unwrap();
        assert_eq!(work[0], source.len() * 16 + 16);
        for stop in 0..work.len() {
            let mut seen = Vec::new();
            assert!(
                matches!(RegExpUnicodeNegativeAtomMatcher::compile_single_character_string_class_lookbehind_with_word_boundaries_validated_with_work(&source,false,false,|n|{seen.push(n);if seen.len()==stop+1{Err(stop)}else{Ok(())}}),Err(crate::RegExpUnicodeSetCompileError::Work(n))if n==stop)
            );
            assert_eq!(seen, work[..=stop]);
        }
        let input = JsString::from("a!a");
        work.clear();
        assert_eq!(
            p.find_with_work(&input, 1, false, |n| {
                work.push(n);
                Ok::<(), usize>(())
            })
            .unwrap(),
            Some(2..2)
        );
        for stop in 0..work.len() {
            let mut seen = Vec::new();
            assert_eq!(
                p.find_with_work(&input, 1, false, |n| {
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
            assert_eq!(find(&p, "a!a", 1, false), Some(2..2));
        }
    }

    #[test]
    fn negative_single_character_q_lookahead_deep_scopes_linear_work_and_abort_recovery_stay_flat()
    {
        let source = format!(
            "{}(?!{}[\\q{{a|b}}]{}){}",
            "(".repeat(100_000),
            "(".repeat(100_000),
            ")".repeat(100_000),
            ")".repeat(100_000)
        );
        let p = q_plan(&source, false, false);
        assert_eq!(p.capture_count(), 200_000);
        let matched = find(&p.clone(), "!", 0, true).unwrap();
        assert_eq!(p.capture_range(99_999, &matched), Some(0..0));
        assert_eq!(p.capture_range(100_000, &matched), None);
        let p = q_plan(r"(?!([\q{a|b}]))", false, false);
        let input = JsString::from("a".repeat(20_000).as_str());
        let mut total = 0usize;
        assert_eq!(
            p.find_with_work(&input, 0, false, |n| {
                total += n;
                Ok::<(), ()>(())
            })
            .unwrap(),
            Some(20_000..20_000)
        );
        assert!(total < 250 * input.len() + 128, "{total}");
        let source = JsString::from(r"((?!([\q{a|b}])))");
        let mut work = Vec::new();
        let p=RegExpUnicodeNegativeAtomMatcher::compile_single_character_string_class_lookahead_validated_with_work(&source,false,false,|n|{work.push(n);Ok::<(),usize>(())}).unwrap().unwrap();
        assert_eq!(work[0], source.len() * 16 + 16);
        for stop in 0..work.len() {
            let mut seen = Vec::new();
            assert!(
                matches!(RegExpUnicodeNegativeAtomMatcher::compile_single_character_string_class_lookahead_validated_with_work(&source,false,false,|n|{seen.push(n);if seen.len()==stop+1{Err(stop)}else{Ok(())}}),Err(crate::RegExpUnicodeSetCompileError::Work(n))if n==stop)
            );
            assert_eq!(seen, work[..=stop]);
        }
        let input = JsString::from("aaa");
        work.clear();
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
    }
    fn scalar_set_plan(source: &str, i: bool, m: bool) -> RegExpUnicodeNegativeAtomMatcher {
        RegExpUnicodeNegativeAtomMatcher::compile_single_v_set_lookahead_validated_with_work(
            &JsString::from(source),
            i,
            m,
            |_| Ok::<(), ()>(()),
        )
        .unwrap()
        .unwrap()
    }

    #[test]
    fn negative_scalar_v_set_lookahead_keeps_empty_outside_and_undefined_operand_slots() {
        for class in [
            r"[[a-z]&&[ab]]",
            r"[[a-z]--[c-z]]",
            r"[[a][b]]",
            r"[[^x]&&[ab]]",
        ] {
            let p = scalar_set_plan(&format!("((?!(({class}))))"), false, false);
            assert_eq!(p.capture_count(), 3);
            let matched = find(&p, "ab!", 0, false).unwrap();
            assert_eq!(matched, 2..2);
            assert_eq!(p.capture_range(0, &matched), Some(2..2));
            for index in 1..=3 {
                assert_eq!(p.capture_range(index, &matched), None);
            }
            assert_eq!(find(&p, "ab!", 0, true), None);
            assert_eq!(find(&p, "ab!", 3, true), Some(3..3));
            assert_eq!(find(&p, "ab!", 4, false), None);
            assert_eq!(find(&p, "", 0, true), Some(0..0));
        }
    }

    #[test]
    fn negative_scalar_v_set_lookahead_keeps_original_line_fold_and_pair_context() {
        let p = scalar_set_plan(r"(?!(^[[a-z]--[aeiou]]$))", true, true);
        assert_eq!(find(&p, "\nB\n", 1, true), None);
        assert_eq!(find(&p, "!B!", 1, true), Some(1..1));
        let p = scalar_set_plan(r"(?!([[ſk]--[x]]))", true, false);
        for text in ["S", "ſ", "K", "K"] {
            assert_eq!(find(&p, text, 0, true), None);
        }
        let p = scalar_set_plan(r"(?!([[\p{ASCII}]--[x]]))", true, false);
        assert_eq!(find(&p, "ſK", 0, false), Some(2..2));
        let p = scalar_set_plan(r"(?!([^[\p{ASCII}]]))", true, false);
        assert_eq!(find(&p, "ſK😀", 0, true), Some(0..0));
        assert_eq!(find(&p, "ſK😀", 2, true), None);
        let p = scalar_set_plan(r"(?!([[a-z]--[a-z]]))", false, false);
        assert_eq!(find(&p, "a", 0, true), Some(0..0));
        let p = scalar_set_plan(r"(?!([[😀a]--[a]]))", false, false);
        assert_eq!(find(&p, "😀😀b", 0, false), Some(4..4));
        assert_eq!(find(&p, "😀😀b", 1, true), None);
        let p = scalar_set_plan(r"(?!([[\uD800x]--[q]]))", false, false);
        assert_eq!(find(&p, "😀", 0, true), Some(0..0));
        let input = JsString::from_code_units(vec![0xd800, 0x78]);
        assert_eq!(
            p.find_with_work(&input, 0, false, |_| Ok::<(), ()>(()))
                .unwrap(),
            Some(2..2)
        );
    }

    #[test]
    fn negative_scalar_v_set_lookahead_declines_other_domains_widths_and_scopes() {
        for source in [
            r"(?!([a]))",
            r"(?!([\q{a|b}]))",
            r"(?!([[\q{a|b}]--[x]]))",
            r"(?!([\q{ab|a}]))",
            r"(?!([\q{|a}]))",
            r"(?!([[a-z]&&[ab]]+))",
            r"(?!([[a-z]&&[ab]]{1}))",
            r"(?!([[a-z]&&[ab]]a))",
            r"(?!([[a-z]&&[ab]]))a",
            r"(?!([[a-z]&&[ab]]))|b",
            r"((?!([[a-z]&&[ab]])))*",
            r"(?=([[a-z]&&[ab]]))",
            r"(?<!([[a-z]&&[ab]]))",
            r"(?!(\b[[a-z]&&[ab]]\b))",
            r"(?i:(?!([[a-z]&&[ab]])))",
            r"((?!(\1)))",
        ] {
            assert!(RegExpUnicodeNegativeAtomMatcher::compile_single_v_set_lookahead_validated_with_work(&JsString::from(source),false,false, |_| Ok::<(), ()>(())).unwrap().is_none(),"{source}");
        }
    }

    #[test]
    fn negative_scalar_v_set_lookahead_deep_scopes_linear_work_and_abort_recovery_stay_flat() {
        let source = format!(
            "{}(?!{}[[a-z]&&[ab]]{}){}",
            "(".repeat(100_000),
            "(".repeat(100_000),
            ")".repeat(100_000),
            ")".repeat(100_000)
        );
        let p = scalar_set_plan(&source, false, false);
        assert_eq!(p.capture_count(), 200_000);
        let matched = find(&p.clone(), "!", 0, true).unwrap();
        assert_eq!(p.capture_range(99_999, &matched), Some(0..0));
        assert_eq!(p.capture_range(100_000, &matched), None);
        let p = scalar_set_plan(r"(?!([[a-z]&&[ab]]))", false, false);
        let input = JsString::from("a".repeat(20_000).as_str());
        let mut total = 0usize;
        assert_eq!(
            p.find_with_work(&input, 0, false, |n| {
                total += n;
                Ok::<(), ()>(())
            })
            .unwrap(),
            Some(20_000..20_000)
        );
        assert!(total < 250 * input.len() + 128, "{total}");
        let source = JsString::from(r"((?!([[a-z]&&[ab]])))");
        let mut work = Vec::new();
        let p =
            RegExpUnicodeNegativeAtomMatcher::compile_single_v_set_lookahead_validated_with_work(
                &source,
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
                matches!(RegExpUnicodeNegativeAtomMatcher::compile_single_v_set_lookahead_validated_with_work(&source,false,false,|n|{seen.push(n);if seen.len()==stop+1{Err(stop)}else{Ok(())}}),Err(crate::RegExpUnicodeSetCompileError::Work(n))if n==stop)
            );
            assert_eq!(seen, work[..=stop]);
        }
        let input = JsString::from("aaa");
        work.clear();
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
    }
}
