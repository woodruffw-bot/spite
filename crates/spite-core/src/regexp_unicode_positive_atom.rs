//! Complete positive Unicode assertions over one character atom (22.2.2.3–4).

use crate::{
    JsString, RegExpUnicodeRepeatedAtomMatcher, regexp_literal::unicode_start,
    regexp_zero_count_group::enclosing_group_body, regexp_zero_count_sequence::group_end,
    regexp_zero_count_tree::surrogate_atom_end,
};
use std::ops::Range;

/// A zero-width assertion match and its independently retained operand extent.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RegExpUnicodePositiveAtomMatch {
    /// Whole assertion match at the original UTF-16 candidate boundary.
    pub range: Range<usize>,
    /// Selected operand range, which may precede or follow the empty whole match.
    pub operand: Range<usize>,
}

/// A complete positive Unicode assertion over one character atom.
///
/// The original u/v Pattern and flags must be validated and capture heads
/// normalized first. Complete unquantified ordinary/noncapturing wrappers may
/// surround the assertion and its operand. Outside captures stay empty; operand
/// wrappers capture the selected run, including a defined empty zero-iteration
/// run (22.2.2.3.1, 22.2.2.4). Internal captures, modifiers, alternatives,
/// continuations and repeated lookbehind operands require other compilers.
#[derive(Clone, Debug)]
pub struct RegExpUnicodePositiveAtomMatcher {
    operand: RegExpUnicodeRepeatedAtomMatcher,
    outside_captures: usize,
    captures: usize,
    backward: bool,
}

impl RegExpUnicodePositiveAtomMatcher {
    /// Compiles a complete validated Unicode lookahead without rebinding references.
    ///
    /// Original full validation and normalized capture heads are required.
    /// Operand projections retain their original binding context; unsupported
    /// reference syntax is rejected by the existing atom compiler. Source work
    /// precedes scans/copies; wrapper depth and symbolic bounds never recurse or
    /// expand. Existing atom compilation retains its work and flag semantics.
    pub fn compile_validated_with_work<E>(
        source: &JsString,
        unicode_sets: bool,
        ignore_case: bool,
        multiline: bool,
        dot_all: bool,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, E> {
        Self::compile_mode_with_work::<0, E>(
            source,
            unicode_sets,
            ignore_case,
            multiline,
            dot_all,
            &mut charge,
        )
    }

    /// Compiles complete lookahead with operand word boundaries and line gates.
    ///
    /// Full original u/v validation and normalized capture heads are required.
    /// Leading/trailing word predicates belong to the selected operand extent;
    /// they retain original input context while the assertion consumes nothing
    /// (22.2.2.4). Complete operand wrappers capture that selected extent. Other
    /// wrapper, atom and capture requirements match the line-only constructor.
    /// Its proof domain and work events remain separate.
    pub fn compile_with_word_boundaries_validated_with_work<E>(
        source: &JsString,
        unicode_sets: bool,
        ignore_case: bool,
        multiline: bool,
        dot_all: bool,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, E> {
        Self::compile_mode_with_work::<1, E>(
            source,
            unicode_sets,
            ignore_case,
            multiline,
            dot_all,
            &mut charge,
        )
    }

    /// Compiles complete lookahead over one unquantified Unicode character atom.
    ///
    /// Requires original full u/v validation and normalized capture heads.
    /// A flat lexical proof isolates exactly one non-reference atom between
    /// optional line assertions. Its fixed single iteration reuses the existing
    /// consuming atom compiler, preserving original input context and capture
    /// extents (22.2.2.3–4). Unknown escapes and additional terms remain unproved;
    /// projected source is never independently validated or rebound.
    pub fn compile_single_atom_validated_with_work<E>(
        source: &JsString,
        unicode_sets: bool,
        ignore_case: bool,
        multiline: bool,
        dot_all: bool,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, E> {
        Self::compile_mode_with_work::<2, E>(
            source,
            unicode_sets,
            ignore_case,
            multiline,
            dot_all,
            &mut charge,
        )
    }

    /// Compiles one unquantified operand atom with outside word and line gates.
    ///
    /// Requires full original u/v validation and normalized capture heads.
    /// Exactly one non-reference atom occurs between optional leading/trailing
    /// word predicates, with ^ before and $ after them. At least one word
    /// predicate is required. The fixed single-iteration projection retains
    /// predicate scope and complete-character candidate boundaries (22.2.2.4).
    pub fn compile_single_atom_with_word_boundaries_validated_with_work<E>(
        source: &JsString,
        unicode_sets: bool,
        ignore_case: bool,
        multiline: bool,
        dot_all: bool,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, E> {
        Self::compile_mode_with_work::<3, E>(
            source,
            unicode_sets,
            ignore_case,
            multiline,
            dot_all,
            &mut charge,
        )
    }

    /// Compiles complete positive lookbehind over one unquantified Unicode atom.
    ///
    /// Full original u/v validation and normalized capture heads are required.
    /// A non-reference character atom has one complete predecessor character;
    /// optional operand line assertions retain their original input context.
    /// Outside wrappers capture the empty assertion position, while complete
    /// operand wrappers capture that predecessor (22.2.2.4). Other counts and
    /// word predicates remain separate proofs.
    pub fn compile_single_lookbehind_validated_with_work<E>(
        source: &JsString,
        unicode_sets: bool,
        ignore_case: bool,
        multiline: bool,
        dot_all: bool,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, E> {
        Self::compile_mode_with_work::<4, E>(
            source,
            unicode_sets,
            ignore_case,
            multiline,
            dot_all,
            &mut charge,
        )
    }

    /// Compiles one predecessor atom with original-context word and line gates.
    ///
    /// Requires full original u/v validation and normalized capture heads.
    /// At least one leading/trailing operand word predicate surrounds a proved
    /// non-reference character atom. A single predecessor makes the operand's
    /// predicates independent of matching direction, while captures retain the
    /// original predecessor extent (22.2.2.4). Other counts remain unproved.
    pub fn compile_single_lookbehind_with_word_boundaries_validated_with_work<E>(
        source: &JsString,
        unicode_sets: bool,
        ignore_case: bool,
        multiline: bool,
        dot_all: bool,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, E> {
        Self::compile_mode_with_work::<5, E>(
            source,
            unicode_sets,
            ignore_case,
            multiline,
            dot_all,
            &mut charge,
        )
    }

    /// Compiles lookahead over one v-mode class with single-character q members.
    ///
    /// Requires full original v/vi Pattern validation and normalized capture
    /// heads. Every ClassStringDisjunction member must decode to one character;
    /// supported nested set operations retain their checked scalar domain.
    /// Complete outside and operand wrappers keep independent empty/consuming
    /// spans. Optional operand line gates inspect original input context
    /// (22.2.2.4, 22.2.2.9). Other counts and word gates remain separate proofs.
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

    /// Compiles one single-character q class with operand word predicates.
    ///
    /// Full original v/vi validation and normalized capture heads are required.
    /// At least one leading/trailing word predicate surrounds the checked class;
    /// optional ^/$ assertions retain original input context. Outside captures
    /// remain empty and operand wrappers retain the selected complete character
    /// (22.2.2.4, 22.2.2.9). Other counts and scopes remain separate proofs.
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

    /// Compiles lookbehind over one v-mode class with single-character q members.
    ///
    /// Requires full original v/vi validation and normalized capture heads.
    /// The checked class must prove every q member is one character. Optional
    /// line gates retain original input context, and complete operand wrappers
    /// capture the one complete predecessor while outside wrappers stay empty
    /// (22.2.2.4, 22.2.2.9). Other counts and word gates need separate proofs.
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

    /// Compiles one single-character q predecessor with operand word predicates.
    ///
    /// Full original v/vi validation and normalized capture heads are required.
    /// At least one leading/trailing word predicate surrounds the checked class;
    /// optional ^/$ gates retain original input context. Operand wrappers capture
    /// the complete predecessor and outside wrappers remain empty at its end
    /// (22.2.2.4, 22.2.2.9). Other counts and scopes remain separate proofs.
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

    /// Compiles lookahead over one unquantified checked scalar v-mode set class.
    ///
    /// Full original v/vi validation and normalized capture heads are required.
    /// At least one nested class or set operator is required; every leaf must be
    /// character-valued. Complete wrappers retain independent empty outside and
    /// consuming operand spans. Optional ^/$ gates keep original input context
    /// (22.2.2.4, 22.2.2.9). Strings, counts and word gates need other proofs.
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

    /// Compiles one scalar v-set lookahead operand with word predicates and line gates.
    ///
    /// Full original v/vi validation and normalized capture heads are required.
    /// A nested/operator class with character-valued leaves occurs between
    /// canonical leading/trailing word predicates; at least one is required.
    /// Predicates inspect original input context and operand wrappers retain
    /// the selected complete character (22.2.2.4, 22.2.2.9). Other scopes and
    /// string leaves remain separate proofs.
    pub fn compile_single_v_set_lookahead_with_word_boundaries_validated_with_work<E>(
        source: &JsString,
        ignore_case: bool,
        multiline: bool,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, crate::RegExpUnicodeSetCompileError<E>> {
        Self::compile_single_character_class_assertion_with_work::<true, false, true, E>(
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
        mut charge: &mut dyn FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, crate::RegExpUnicodeSetCompileError<E>> {
        charge(source.len().saturating_mul(16).saturating_add(16))
            .map_err(crate::RegExpUnicodeSetCompileError::Work)?;
        let units = source.code_units();
        let (assertion, outside_captures) = if let Some(scope) = enclosing_group_body(units, true) {
            (&units[scope.body], scope.captures)
        } else {
            (units, 0)
        };
        let prefix: &[u16] = if BACKWARD {
            &[0x28, 0x3f, 0x3c, 0x3d]
        } else {
            &[0x28, 0x3f, 0x3d]
        };
        if !assertion.starts_with(prefix) || group_end(assertion, 0, true) != Some(assertion.len())
        {
            return Ok(None);
        }
        let body = &assertion[prefix.len()..assertion.len() - 1];
        let (body, captures) = if let Some(scope) = enclosing_group_body(body, true) {
            (&body[scope.body], scope.captures)
        } else {
            (body, 0)
        };
        let mut start = usize::from(body.first() == Some(&0x5e));
        let leading_word = WORD && word_at(body, start);
        start += 2 * usize::from(leading_word);
        if body.get(start) != Some(&0x5b) {
            return Ok(None);
        }
        let Some(end) = surrogate_atom_end(body, start, true, true) else {
            return Ok(None);
        };
        let trailing_word = WORD && word_at(body, end);
        let tail = end + 2 * usize::from(trailing_word);
        if (WORD && !leading_word && !trailing_word)
            || (tail != body.len() && !(body.get(tail) == Some(&0x24) && tail + 1 == body.len()))
        {
            return Ok(None);
        }
        let mut projected = Vec::with_capacity(body.len().saturating_add(3));
        projected.extend_from_slice(&body[..end]);
        projected.extend_from_slice(&[0x7b, 0x31, 0x7d]);
        projected.extend_from_slice(&body[end..]);
        let source = JsString::from_code_units(projected);
        let operand = if SCALAR_SET {
            if WORD {
                RegExpUnicodeRepeatedAtomMatcher::compile_v_set_with_word_boundaries_and_work(
                    &source,
                    ignore_case,
                    multiline,
                    &mut charge,
                )?
            } else {
                RegExpUnicodeRepeatedAtomMatcher::compile_v_set_with_line_anchors_and_work(
                    &source,
                    ignore_case,
                    multiline,
                    &mut charge,
                )?
            }
        } else {
            RegExpUnicodeRepeatedAtomMatcher::compile_v_set_with_single_character_strings_and_work(
                &source,
                ignore_case,
                multiline,
                &mut charge,
            )?
        };
        Ok(operand
            .filter(|operand| operand.capture_count() == 0)
            .map(|operand| Self {
                operand,
                outside_captures,
                captures: outside_captures + captures,
                backward: BACKWARD,
            }))
    }

    // Share the wrapper parser and delegated compiler across callback types.
    #[inline(never)]
    fn compile_mode_with_work<const MODE: u8, E>(
        source: &JsString,
        unicode_sets: bool,
        ignore_case: bool,
        multiline: bool,
        dot_all: bool,
        mut charge: &mut dyn FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, E> {
        charge(source.len().saturating_mul(16).saturating_add(16))?;
        let units = source.code_units();
        let (assertion, outside_captures) =
            if let Some(scope) = enclosing_group_body(units, unicode_sets) {
                (&units[scope.body], scope.captures)
            } else {
                (units, 0)
            };
        let (prefix, width): (&[u16], usize) = if MODE >= 4 {
            (&[0x28, 0x3f, 0x3c, 0x3d], 4)
        } else {
            (&[0x28, 0x3f, 0x3d], 3)
        };
        if !assertion.starts_with(prefix)
            || group_end(assertion, 0, unicode_sets) != Some(assertion.len())
        {
            return Ok(None);
        }
        let body = &assertion[width..assertion.len() - 1];
        let (body, captures) = if let Some(scope) = enclosing_group_body(body, unicode_sets) {
            (&body[scope.body], scope.captures)
        } else {
            (body, 0)
        };
        let body = if MODE >= 2 {
            let mut start = usize::from(body.first() == Some(&0x5e));
            let word = MODE == 3 || MODE == 5;
            let leading_word = word && word_at(body, start);
            start += 2 * usize::from(leading_word);
            let Some(end) = surrogate_atom_end(body, start, true, unicode_sets) else {
                return Ok(None);
            };
            let trailing_word = word && word_at(body, end);
            let tail = end + 2 * usize::from(trailing_word);
            if (word && !leading_word && !trailing_word)
                || (tail != body.len()
                    && !(body.get(tail) == Some(&0x24) && tail + 1 == body.len()))
            {
                return Ok(None);
            }
            // A proved non-reference Atom has exactly one selected iteration.
            // Insert the fixed bound before its outside end assertions.
            let mut units = Vec::with_capacity(body.len().saturating_add(3));
            units.extend_from_slice(&body[..end]);
            units.extend_from_slice(&[0x7b, 0x31, 0x7d]);
            units.extend_from_slice(&body[end..]);
            JsString::from_code_units(units)
        } else {
            JsString::from_code_units(body.to_vec())
        };
        let operand = if MODE == 1 || MODE == 3 || MODE == 5 {
            RegExpUnicodeRepeatedAtomMatcher::compile_with_word_boundaries_and_work(
                &body,
                unicode_sets,
                ignore_case,
                multiline,
                dot_all,
                &mut charge,
            )?
        } else {
            RegExpUnicodeRepeatedAtomMatcher::compile_with_line_anchors_and_work(
                &body,
                unicode_sets,
                ignore_case,
                multiline,
                dot_all,
                &mut charge,
            )?
        };
        let Some(operand) = operand else {
            return Ok(None);
        };
        if operand.capture_count() != 0 {
            return Ok(None);
        }
        Ok(Some(Self {
            operand,
            outside_captures,
            captures: outside_captures + captures,
            backward: MODE >= 4,
        }))
    }

    /// Full original lexical capture count, including outside assertion wrappers.
    pub fn capture_count(&self) -> usize {
        self.captures
    }

    /// Searches using the operand's original context while retaining its capture extent.
    ///
    /// The underlying Unicode repetition keeps greedy/lazy counts, line anchors,
    /// sticky candidates and the approved leading-surrogate boundary policy.
    /// Its callback charges actual work before input access; no capture-sized
    /// allocation or rescanning is needed (22.2.2.4, 22.2.7.2).
    /// Single-atom lookbehind tries one complete predecessor per candidate;
    /// the fixed count bounds each sticky operand probe without suffix rescans.
    pub fn find_with_work<E>(
        &self,
        input: &JsString,
        start: usize,
        sticky: bool,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<RegExpUnicodePositiveAtomMatch>, E> {
        if self.backward {
            charge(1)?;
            let Some(mut candidate) = unicode_start(input, start) else {
                return Ok(None);
            };
            let units = input.code_units();
            loop {
                // A candidate inspects at most two preceding units. The proved
                // fixed count bounds each sticky operand attempt to two atoms.
                charge(4)?;
                if candidate > 0 {
                    let previous = candidate
                        - if candidate >= 2
                            && (0xdc00..=0xdfff).contains(&units[candidate - 1])
                            && (0xd800..=0xdbff).contains(&units[candidate - 2])
                        {
                            2
                        } else {
                            1
                        };
                    if let Some(operand) =
                        self.operand
                            .find_with_work(input, previous, true, &mut charge)?
                    {
                        if operand.end == candidate {
                            return Ok(Some(RegExpUnicodePositiveAtomMatch {
                                range: candidate..candidate,
                                operand,
                            }));
                        }
                    }
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
        Ok(self
            .operand
            .find_with_work(input, start, sticky, charge)?
            .map(|operand| RegExpUnicodePositiveAtomMatch {
                range: operand.start..operand.start,
                operand,
            }))
    }

    /// Exact empty outside span or full operand capture span, in constant work.
    pub fn capture_range(
        &self,
        index: usize,
        matched: &RegExpUnicodePositiveAtomMatch,
    ) -> Option<Range<usize>> {
        if index >= self.captures {
            None
        } else if index < self.outside_captures {
            Some(matched.range.clone())
        } else {
            Some(matched.operand.clone())
        }
    }
}

fn word_at(units: &[u16], at: usize) -> bool {
    units.get(at) == Some(&0x5c) && matches!(units.get(at + 1), Some(0x62 | 0x42))
}

#[cfg(test)]
mod tests {
    use super::*;
    fn plan(source: &str) -> RegExpUnicodePositiveAtomMatcher {
        RegExpUnicodePositiveAtomMatcher::compile_validated_with_work(
            &JsString::from(source),
            false,
            false,
            false,
            false,
            |_| Ok::<(), ()>(()),
        )
        .unwrap()
        .unwrap()
    }
    #[test]
    fn unicode_lookahead_retains_consumed_capture_ranges_beyond_empty_whole_match() {
        let p = plan("((?=((a*))))");
        let m = p
            .find_with_work(&JsString::from("aaa"), 0, false, |_| Ok::<(), ()>(()))
            .unwrap()
            .unwrap();
        assert_eq!(m.range, 0..0);
        assert_eq!(m.operand, 0..3);
        assert_eq!(p.capture_count(), 3);
        assert_eq!(p.capture_range(0, &m), Some(0..0));
        assert_eq!(p.capture_range(1, &m), Some(0..3));
        assert_eq!(p.capture_range(2, &m), Some(0..3));
        assert_eq!(p.capture_range(3, &m), None);
        let p = plan("(?=(a*?))");
        let m = p
            .find_with_work(&JsString::from("aaa"), 0, false, |_| Ok::<(), ()>(()))
            .unwrap()
            .unwrap();
        assert_eq!(m.operand, 0..0);
        assert_eq!(p.capture_range(0, &m), Some(0..0));
    }
    #[test]
    fn unicode_lookahead_search_sticky_flags_and_surrogate_candidates_keep_original_positions() {
        let p = plan("(?=(😀+))");
        let input = JsString::from("x😀😀y");
        let m = p
            .find_with_work(&input, 0, false, |_| Ok::<(), ()>(()))
            .unwrap()
            .unwrap();
        assert_eq!(m.range, 1..1);
        assert_eq!(m.operand, 1..5);
        assert!(
            p.find_with_work(&input, 0, true, |_| Ok::<(), ()>(()))
                .unwrap()
                .is_none()
        );
        let m = p
            .find_with_work(&input, 2, true, |_| Ok::<(), ()>(()))
            .unwrap()
            .unwrap();
        assert_eq!(m.range, 1..1);
        assert_eq!(m.operand, 1..5);
    }

    #[test]
    fn unicode_lookahead_ascii_property_uses_the_specification_folded_set() {
        // CompileToCharSet and CharacterSetMatcher fold both the positive
        // property and the input. See the documented Node iv discrepancy.
        for sets in [false, true] {
            for ignore_case in [false, true] {
                let p = RegExpUnicodePositiveAtomMatcher::compile_validated_with_work(
                    &JsString::from(r"(?=(\p{ASCII}+))"),
                    sets,
                    ignore_case,
                    false,
                    false,
                    |_| Ok::<(), ()>(()),
                )
                .unwrap()
                .unwrap();
                let input = JsString::from("ſKa");
                let m = p
                    .find_with_work(&input, 0, false, |_| Ok::<(), ()>(()))
                    .unwrap()
                    .unwrap();
                let start = if ignore_case { 0 } else { 2 };
                assert_eq!(m.range, start..start);
                assert_eq!(m.operand, start..3);
                assert_eq!(p.capture_range(0, &m), Some(start..3));
            }
        }
    }

    #[test]
    fn unicode_lookahead_rejects_continuations_modifiers_quantified_wrappers_and_internal_captures()
    {
        for source in [
            "(?<=(a*))",
            "(?!(a*))",
            "(?=(a*))a",
            "a(?=(a*))",
            "(?=(a*))|b",
            "((?=(a*)))*",
            "(?i:(?=(a*)))",
            "(?=(a)* )",
            "(?=((a)*))",
            "(?=(a*b*))",
        ] {
            assert!(
                RegExpUnicodePositiveAtomMatcher::compile_validated_with_work(
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
    fn unicode_lookahead_deep_wrappers_symbolic_bounds_and_work_interruptions_are_flat() {
        let source = format!(
            "{}(?={}a*{}){}",
            "(".repeat(100_000),
            "(".repeat(100_000),
            ")".repeat(100_000),
            ")".repeat(100_000)
        );
        let p = plan(&source);
        assert_eq!(p.capture_count(), 200_000);
        let source = format!("(?=(a{{{}}}))", "9".repeat(20_000));
        let p = plan(&source);
        assert!(
            p.find_with_work(&JsString::from("aaaa"), 0, false, |_| Ok::<(), ()>(()))
                .unwrap()
                .is_none()
        );
        let source = JsString::from("((?=(a*)))");
        let mut work = Vec::new();
        RegExpUnicodePositiveAtomMatcher::compile_validated_with_work(
            &source,
            false,
            false,
            false,
            false,
            |n| {
                work.push(n);
                Ok::<(), ()>(())
            },
        )
        .unwrap()
        .unwrap();
        assert_eq!(work[0], source.len() * 16 + 16);
        for stop in 0..work.len() {
            let mut seen = Vec::new();
            assert!(
                matches!(RegExpUnicodePositiveAtomMatcher::compile_validated_with_work(&source,false,false,false,false,|n|{seen.push(n);if seen.len()==stop+1{Err(stop)}else{Ok(())}}),Err(n)if n==stop)
            );
            assert_eq!(seen, work[..=stop]);
        }
        assert_eq!(
            plan("(?=(a*))").find_with_work(&JsString::from(""), usize::MAX, false, |_| Err(59)),
            Err(59)
        );
    }

    fn word_plan(source: &str, i: bool, m: bool) -> RegExpUnicodePositiveAtomMatcher {
        RegExpUnicodePositiveAtomMatcher::compile_with_word_boundaries_validated_with_work(
            &JsString::from(source),
            false,
            i,
            m,
            false,
            |_| Ok::<(), ()>(()),
        )
        .unwrap()
        .unwrap()
    }

    #[test]
    fn lookahead_word_endpoints_constrain_lazy_operands_without_consuming_the_match() {
        let p = word_plan(r"((?=((\ba+?\b))))", false, false);
        let m = p
            .find_with_work(&JsString::from("!aaa!"), 0, false, |_| Ok::<(), ()>(()))
            .unwrap()
            .unwrap();
        assert_eq!(m.range, 1..1);
        assert_eq!(m.operand, 1..4);
        assert_eq!(p.capture_count(), 3);
        assert_eq!(p.capture_range(0, &m), Some(1..1));
        assert_eq!(p.capture_range(1, &m), Some(1..4));
        assert_eq!(p.capture_range(2, &m), Some(1..4));
        assert!(
            RegExpUnicodePositiveAtomMatcher::compile_validated_with_work(
                &JsString::from(r"(?=(\ba+\b))"),
                false,
                false,
                false,
                false,
                |_| Ok::<(), ()>(())
            )
            .unwrap()
            .is_none()
        );
    }

    #[test]
    fn lookahead_word_folds_line_context_and_empty_nonboundaries_keep_operand_positions() {
        for i in [false, true] {
            let p = word_plan(r"(?=(\bſ+\b))", i, false);
            let m = p
                .find_with_work(&JsString::from("!ſſ!"), 0, false, |_| Ok::<(), ()>(()))
                .unwrap();
            if i {
                assert_eq!(m.unwrap().operand, 1..3);
            } else {
                assert!(m.is_none());
            }
        }
        let p = word_plan(r"(?=(^\b.+\b$))", false, true);
        let m = p
            .find_with_work(&JsString::from("\nAA\n"), 0, false, |_| Ok::<(), ()>(()))
            .unwrap()
            .unwrap();
        assert_eq!(m.range, 1..1);
        assert_eq!(m.operand, 1..3);
        let p = word_plan(r"(?=(\B.*?\B))", false, false);
        let m = p
            .find_with_work(&JsString::from("!a"), 0, true, |_| Ok::<(), ()>(()))
            .unwrap()
            .unwrap();
        assert_eq!(m.range, 0..0);
        assert_eq!(p.capture_range(0, &m), Some(0..0));
    }

    #[test]
    fn lookahead_word_candidates_respect_original_surrogate_context_and_proof_boundaries() {
        let p = word_plan(r"(?=(\ba+\b))", false, false);
        let input = JsString::from("😀a!");
        assert!(
            p.find_with_work(&input, 1, true, |_| Ok::<(), ()>(()))
                .unwrap()
                .is_none()
        );
        assert_eq!(
            p.find_with_work(&input, 1, false, |_| Ok::<(), ()>(()))
                .unwrap()
                .unwrap()
                .operand,
            2..3
        );
        // RegExpBuiltinExec advances by complete Unicode characters (22.2.7.2).
        // The empty nonboundary occurs between the final a and b, never between
        // the surrogate halves of the intervening emoji.
        let p = word_plan(r"(?=(\Bſ*?\B))", false, false);
        let m = p
            .find_with_work(&JsString::from("a😀ab!"), 0, false, |_| Ok::<(), ()>(()))
            .unwrap()
            .unwrap();
        assert_eq!(m.range, 4..4);
        assert_eq!(m.operand, 4..4);
        for source in [
            r"(?<=(\ba+\b))",
            r"(?=(\ba+\b))a",
            r"(?=(\b(a)+\b))",
            r"(?i:(?=(\ba+\b)))",
            r"((?=(\ba+\b)))*",
        ] {
            assert!(
                RegExpUnicodePositiveAtomMatcher::compile_with_word_boundaries_validated_with_work(
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
    fn lookahead_word_deep_capture_storage_and_every_work_abort_remain_flat() {
        let source = format!(
            "{}(?={}\\ba+?\\b{}){}",
            "(".repeat(100_000),
            "(".repeat(100_000),
            ")".repeat(100_000),
            ")".repeat(100_000)
        );
        assert_eq!(word_plan(&source, false, false).capture_count(), 200_000);
        let source = JsString::from(r"((?=(\ba+?\b)))");
        let mut work = Vec::new();
        RegExpUnicodePositiveAtomMatcher::compile_with_word_boundaries_validated_with_work(
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
        for stop in 0..work.len() {
            let mut seen = Vec::new();
            assert!(
                matches!(RegExpUnicodePositiveAtomMatcher::compile_with_word_boundaries_validated_with_work(&source, false, false, false, false, |n| { seen.push(n); if seen.len()==stop+1 { Err(stop) } else { Ok(()) } }), Err(n) if n==stop)
            );
            assert_eq!(seen, work[..=stop]);
        }
        let p = word_plan(r"(?=(\ba+?\b))", false, false);
        let input = JsString::from("!aaa!");
        let mut work = Vec::new();
        let expected = p
            .find_with_work(&input, 0, false, |n| {
                work.push(n);
                Ok::<(), usize>(())
            })
            .unwrap();
        for stop in 0..work.len() {
            let mut seen = Vec::new();
            assert!(
                matches!(p.find_with_work(&input,0,false, |n| { seen.push(n); if seen.len()==stop+1 { Err(stop) } else { Ok(()) } }),Err(n) if n==stop)
            );
            assert_eq!(seen, work[..=stop]);
        }
        assert_eq!(
            p.find_with_work(&input, 0, false, |_| Ok::<(), usize>(()))
                .unwrap(),
            expected
        );
    }

    fn single_plan(
        source: &str,
        sets: bool,
        i: bool,
        m: bool,
        s: bool,
    ) -> RegExpUnicodePositiveAtomMatcher {
        RegExpUnicodePositiveAtomMatcher::compile_single_atom_validated_with_work(
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
    fn single_atom_lookahead_captures_one_complete_original_unicode_character() {
        for (atom, input, expected) in [
            ("a", "ba!", 1..2),
            ("[a-z]", "!a!", 1..2),
            (r"\d", "a2!", 1..2),
            (".", "😀", 0..2),
            (r"\u{1F600}", "!😀!", 1..3),
            (r"\uD83D\uDE00", "!😀!", 1..3),
            (r"\x24", "!$!", 1..2),
            (r"\$", "!$!", 1..2),
            ("[$]", "!$!", 1..2),
        ] {
            let source = format!("((?=(({atom}))))");
            let p = single_plan(&source, false, false, false, false);
            let m = p
                .find_with_work(&JsString::from(input), 0, false, |_| Ok::<(), ()>(()))
                .unwrap()
                .unwrap();
            assert_eq!(m.range, expected.start..expected.start, "{source}");
            assert_eq!(m.operand, expected, "{source}");
            assert_eq!(p.capture_count(), 3);
            assert_eq!(p.capture_range(0, &m), Some(m.range.clone()));
            assert_eq!(p.capture_range(1, &m), Some(expected.clone()));
            assert_eq!(p.capture_range(2, &m), Some(expected));
        }
    }

    #[test]
    fn single_atom_lookahead_preserves_line_dot_and_specification_property_folds() {
        for sets in [false, true] {
            let p = single_plan(r"(?=(\uD800))", sets, false, false, false);
            let input = JsString::from_code_units(vec![0xd800, 0x78, 0xdc00]);
            assert_eq!(
                p.find_with_work(&input, 0, true, |_| Ok::<(), ()>(()))
                    .unwrap()
                    .unwrap()
                    .operand,
                0..1
            );
            let pair = JsString::from_code_units(vec![0xd800, 0xdc00]);
            assert!(
                p.find_with_work(&pair, 1, false, |_| Ok::<(), ()>(()))
                    .unwrap()
                    .is_none()
            );
            let p = single_plan(r"(?=(^a$))", sets, true, true, false);
            let m = p
                .find_with_work(&JsString::from("x\nA\n"), 0, false, |_| Ok::<(), ()>(()))
                .unwrap()
                .unwrap();
            assert_eq!(m.operand, 2..3);
            let p = single_plan(r"(?=(^\$$))", sets, false, false, false);
            assert_eq!(
                p.find_with_work(&JsString::from("$"), 0, true, |_| Ok::<(), ()>(()))
                    .unwrap()
                    .unwrap()
                    .operand,
                0..1
            );
            for i in [false, true] {
                let p = single_plan(r"(?=(\p{ASCII}))", sets, i, false, false);
                let m = p
                    .find_with_work(&JsString::from("ſa"), 0, false, |_| Ok::<(), ()>(()))
                    .unwrap()
                    .unwrap();
                assert_eq!(m.operand, if i { 0..1 } else { 1..2 });
            }
            for s in [false, true] {
                let p = single_plan("(?=(.))", sets, false, false, s);
                let m = p
                    .find_with_work(&JsString::from("\na"), 0, false, |_| Ok::<(), ()>(()))
                    .unwrap()
                    .unwrap();
                assert_eq!(m.operand, if s { 0..1 } else { 1..2 });
            }
        }
    }

    #[test]
    fn single_atom_lookahead_proof_excludes_other_terms_counts_and_reference_rebinding() {
        for source in [
            r"(?=(ab))",
            r"(?=(a*))",
            r"(?=(a{1}))",
            r"(?=(\ba\b))",
            r"(?=(\1))",
            r"((?=(\1)))",
            r"(?=(a|b))",
            r"(?=(a(b)))",
            r"(?<=(a))",
            r"((?=(a)))*",
            r"(?=(^^a$))",
        ] {
            assert!(
                RegExpUnicodePositiveAtomMatcher::compile_single_atom_validated_with_work(
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
        assert!(
            RegExpUnicodePositiveAtomMatcher::compile_validated_with_work(
                &JsString::from("(?=(a))"),
                false,
                false,
                false,
                false,
                |_| Ok::<(), ()>(())
            )
            .unwrap()
            .is_none()
        );
    }

    #[test]
    fn single_atom_lookahead_large_sources_capture_counts_and_work_cancellation_are_flat() {
        let source = format!(
            "{}(?={}a{}){}",
            "(".repeat(100_000),
            "(".repeat(100_000),
            ")".repeat(100_000),
            ")".repeat(100_000)
        );
        let p = single_plan(&source, false, false, false, false);
        assert_eq!(p.capture_count(), 200_000);
        let source = format!("(?=([{}]))", "a".repeat(20_000));
        let p = single_plan(&source, false, false, false, false);
        let input = JsString::from("😀a");
        assert_eq!(
            p.find_with_work(&input, 1, false, |_| Ok::<(), ()>(()))
                .unwrap()
                .unwrap()
                .operand,
            2..3
        );
        let source = JsString::from(r"((?=(\u{1F600})))");
        let mut work = Vec::new();
        RegExpUnicodePositiveAtomMatcher::compile_single_atom_validated_with_work(
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
                matches!(RegExpUnicodePositiveAtomMatcher::compile_single_atom_validated_with_work(&source,false,false,false,false, |n| {seen.push(n);if seen.len()==stop+1 {Err(stop)}else{Ok(())}}), Err(n) if n==stop)
            );
            assert_eq!(seen, work[..=stop]);
        }
    }

    fn single_word_plan(source: &str, i: bool, m: bool) -> RegExpUnicodePositiveAtomMatcher {
        RegExpUnicodePositiveAtomMatcher::compile_single_atom_with_word_boundaries_validated_with_work(
            &JsString::from(source), false, i, m, false, |_| Ok::<(), ()>(()),
        ).unwrap().unwrap()
    }

    #[test]
    fn single_word_lookahead_selects_one_character_with_independent_capture_ranges() {
        for (source, input, range) in [
            (r"((?=((\ba\B))))", "aaa", 0..1),
            (r"(?=(\Ba\B))", "aaaa", 1..2),
            (r"(?=(\b[a-z]\b))", "!a!", 1..2),
            (r"(?=(\B[\b]\B))", "\x08", 0..1),
        ] {
            let p = single_word_plan(source, false, false);
            let m = p
                .find_with_work(&JsString::from(input), 0, false, |_| Ok::<(), ()>(()))
                .unwrap()
                .unwrap();
            assert_eq!(m.range, range.start..range.start);
            assert_eq!(m.operand, range);
            assert_eq!(p.capture_range(p.capture_count() - 1, &m), Some(range));
            if p.capture_count() == 3 {
                assert_eq!(p.capture_range(0, &m), Some(0..0));
            }
        }
    }

    #[test]
    fn single_word_lookahead_uses_original_line_fold_and_surrogate_contexts() {
        let p = single_word_plan(r"(?=(^\ba\b$))", true, true);
        let m = p
            .find_with_work(&JsString::from("\nA\n"), 0, false, |_| Ok::<(), ()>(()))
            .unwrap()
            .unwrap();
        assert_eq!(m.operand, 1..2);
        let p = single_word_plan(r"(?=(\bſ\b))", true, false);
        assert_eq!(
            p.find_with_work(&JsString::from("!ſ!"), 0, false, |_| Ok::<(), ()>(()))
                .unwrap()
                .unwrap()
                .operand,
            1..2
        );
        let p = single_word_plan(r"(?=(\bſ\b))", false, false);
        assert!(
            p.find_with_work(&JsString::from("!ſ!"), 0, false, |_| Ok::<(), ()>(()))
                .unwrap()
                .is_none()
        );
        let p = single_word_plan(r"(?=(\B.\b))", false, false);
        let input = JsString::from("a😀ab!");
        let m = p
            .find_with_work(&input, 0, false, |_| Ok::<(), ()>(()))
            .unwrap()
            .unwrap();
        assert_eq!(m.range, 4..4);
        assert_eq!(m.operand, 4..5);
        assert!(
            p.find_with_work(&input, 2, true, |_| Ok::<(), ()>(()))
                .unwrap()
                .is_none()
        );
    }

    #[test]
    fn single_word_lookahead_keeps_separate_term_count_and_predicate_domains() {
        for source in [
            r"(?=(a))",
            r"(?=(\b))",
            r"(?=(\ba+\b))",
            r"(?=(\ba{1}\b))",
            r"(?=(a\bb))",
            r"(?=(\ba\b))a",
            r"(?<=(\ba\b))",
            r"(?i:(?=(\ba\b)))",
            r"(?=(\b\1\b))",
        ] {
            assert!(RegExpUnicodePositiveAtomMatcher::compile_single_atom_with_word_boundaries_validated_with_work(&JsString::from(source),false,false,false,false, |_| Ok::<(),()>(())).unwrap().is_none(),"{source}");
        }
        assert!(
            RegExpUnicodePositiveAtomMatcher::compile_single_atom_validated_with_work(
                &JsString::from(r"(?=(\ba\b))"),
                false,
                false,
                false,
                false,
                |_| Ok::<(), ()>(())
            )
            .unwrap()
            .is_none()
        );
    }

    #[test]
    fn single_word_lookahead_deep_wrappers_and_all_work_cancellations_are_flat() {
        let source = format!(
            "{}(?={}\\ba\\b{}){}",
            "(".repeat(100_000),
            "(".repeat(100_000),
            ")".repeat(100_000),
            ")".repeat(100_000)
        );
        assert_eq!(
            single_word_plan(&source, false, false).capture_count(),
            200_000
        );
        let source = JsString::from(r"((?=(\ba\b)))");
        let mut work = Vec::new();
        RegExpUnicodePositiveAtomMatcher::compile_single_atom_with_word_boundaries_validated_with_work(&source,false,false,false,false, |n| {work.push(n);Ok::<(),usize>(())}).unwrap().unwrap();
        for stop in 0..work.len() {
            let mut seen = Vec::new();
            assert!(
                matches!(RegExpUnicodePositiveAtomMatcher::compile_single_atom_with_word_boundaries_validated_with_work(&source,false,false,false,false, |n| {seen.push(n);if seen.len()==stop+1 {Err(stop)}else{Ok(())}}),Err(n) if n==stop)
            );
            assert_eq!(seen, work[..=stop]);
        }
        let p = single_word_plan(r"(?=(\ba\b))", false, false);
        let input = JsString::from("!!a!");
        let mut work = Vec::new();
        let expected = p
            .find_with_work(&input, 0, false, |n| {
                work.push(n);
                Ok::<(), usize>(())
            })
            .unwrap();
        for stop in 0..work.len() {
            let mut seen = Vec::new();
            assert!(
                matches!(p.find_with_work(&input,0,false, |n| {seen.push(n);if seen.len()==stop+1 {Err(stop)}else{Ok(())}}),Err(n) if n==stop)
            );
            assert_eq!(seen, work[..=stop]);
        }
        assert_eq!(
            p.find_with_work(&input, 0, false, |_| Ok::<(), usize>(()))
                .unwrap(),
            expected
        );
    }

    fn behind_plan(
        source: &str,
        sets: bool,
        i: bool,
        m: bool,
        s: bool,
    ) -> RegExpUnicodePositiveAtomMatcher {
        RegExpUnicodePositiveAtomMatcher::compile_single_lookbehind_validated_with_work(
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
    fn single_lookbehind_captures_predecessor_ranges_before_empty_outside_captures() {
        for (atom, input, range) in [
            ("a", "ba!", 1..2),
            ("[a-z]", "!a!", 1..2),
            (r"\d", "a2!", 1..2),
            (".", "😀a", 0..2),
            (r"\u{1F600}", "x😀!", 1..3),
            (r"\uD83D\uDE00", "x😀!", 1..3),
            (r"\$", "!$!", 1..2),
        ] {
            let p = behind_plan(&format!("((?<=(({atom}))))"), false, false, false, false);
            let m = p
                .find_with_work(&JsString::from(input), 0, false, |_| Ok::<(), ()>(()))
                .unwrap()
                .unwrap();
            assert_eq!(m.range, range.end..range.end);
            assert_eq!(m.operand, range);
            assert_eq!(p.capture_count(), 3);
            assert_eq!(p.capture_range(0, &m), Some(m.range.clone()));
            assert_eq!(p.capture_range(1, &m), Some(range.clone()));
            assert_eq!(p.capture_range(2, &m), Some(range));
        }
    }

    #[test]
    fn single_lookbehind_line_dot_and_property_folds_use_original_input_context() {
        for sets in [false, true] {
            let p = behind_plan(r"(?<=(^a$))", sets, true, true, false);
            assert_eq!(
                p.find_with_work(&JsString::from("\nA\n"), 0, false, |_| Ok::<(), ()>(()))
                    .unwrap()
                    .unwrap()
                    .operand,
                1..2
            );
            let p = behind_plan(r"(?<=(^\$$))", sets, false, false, false);
            assert_eq!(
                p.find_with_work(&JsString::from("$"), 0, false, |_| Ok::<(), ()>(()))
                    .unwrap()
                    .unwrap()
                    .operand,
                0..1
            );
            for i in [false, true] {
                let p = behind_plan(r"(?<=(\p{ASCII}))", sets, i, false, false);
                let m = p
                    .find_with_work(&JsString::from("ſa"), 0, false, |_| Ok::<(), ()>(()))
                    .unwrap()
                    .unwrap();
                assert_eq!(m.operand, if i { 0..1 } else { 1..2 });
            }
            for s in [false, true] {
                let p = behind_plan("(?<=(.))", sets, false, false, s);
                let m = p
                    .find_with_work(&JsString::from("\na"), 0, false, |_| Ok::<(), ()>(()))
                    .unwrap()
                    .unwrap();
                assert_eq!(m.operand, if s { 0..1 } else { 1..2 });
            }
        }
    }

    #[test]
    fn single_lookbehind_sticky_surrogate_and_end_candidates_remain_complete() {
        let p = behind_plan("(?<=(.))", false, false, false, false);
        let input = JsString::from("😀a");
        assert!(
            p.find_with_work(&input, 1, true, |_| Ok::<(), ()>(()))
                .unwrap()
                .is_none()
        );
        let m = p
            .find_with_work(&input, 1, false, |_| Ok::<(), ()>(()))
            .unwrap()
            .unwrap();
        assert_eq!(m.range, 2..2);
        assert_eq!(m.operand, 0..2);
        assert_eq!(
            p.find_with_work(&input, 3, true, |_| Ok::<(), ()>(()))
                .unwrap()
                .unwrap()
                .operand,
            2..3
        );
        assert!(
            p.find_with_work(&input, 4, false, |_| Ok::<(), ()>(()))
                .unwrap()
                .is_none()
        );
        let p = behind_plan(r"(?<=(\uD800))", false, false, false, false);
        assert!(
            p.find_with_work(
                &JsString::from_code_units(vec![0xd800, 0xdc00]),
                0,
                false,
                |_| Ok::<(), ()>(())
            )
            .unwrap()
            .is_none()
        );
        assert_eq!(
            p.find_with_work(
                &JsString::from_code_units(vec![0xd800]),
                0,
                false,
                |_| Ok::<(), ()>(())
            )
            .unwrap()
            .unwrap()
            .operand,
            0..1
        );
        assert_eq!(
            p.find_with_work(&input, usize::MAX, false, |_| Err(59)),
            Err(59)
        );
    }

    #[test]
    fn single_lookbehind_excludes_counts_other_directions_and_unproved_bindings() {
        for source in [
            r"(?<=(a*))",
            r"(?<=(a{1}))",
            r"(?<=(ab))",
            r"(?<=(a|b))",
            r"(?<=(\ba\b))",
            r"(?<=(\1))",
            r"(?<!(a))",
            r"(?=(a))",
            r"(?<=(a))a",
            r"(?i:(?<=(a)))",
            r"((?<=(a)))*",
        ] {
            assert!(
                RegExpUnicodePositiveAtomMatcher::compile_single_lookbehind_validated_with_work(
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
        assert!(
            RegExpUnicodePositiveAtomMatcher::compile_single_atom_validated_with_work(
                &JsString::from("(?<=(a))"),
                false,
                false,
                false,
                false,
                |_| Ok::<(), ()>(())
            )
            .unwrap()
            .is_none()
        );
    }

    #[test]
    fn single_lookbehind_storage_work_cancellations_and_anchored_search_are_flat() {
        let source = format!(
            "{}(?<={}a{}){}",
            "(".repeat(100_000),
            "(".repeat(100_000),
            ")".repeat(100_000),
            ")".repeat(100_000)
        );
        assert_eq!(
            behind_plan(&source, false, false, false, false).capture_count(),
            200_000
        );
        let p = behind_plan("(?<=(a$))", false, false, false, false);
        let input = JsString::from("a".repeat(20_000).as_str());
        let mut work = 0;
        let m = p
            .find_with_work(&input, 0, false, |n| {
                work += n;
                assert!(work <= 120 * input.len() + 64);
                Ok::<(), ()>(())
            })
            .unwrap()
            .unwrap();
        assert_eq!(m.range, 20_000..20_000);
        assert_eq!(m.operand, 19_999..20_000);
        let source = JsString::from("((?<=(a)))");
        let mut work = Vec::new();
        RegExpUnicodePositiveAtomMatcher::compile_single_lookbehind_validated_with_work(
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
        for stop in 0..work.len() {
            let mut seen = Vec::new();
            assert!(
                matches!(RegExpUnicodePositiveAtomMatcher::compile_single_lookbehind_validated_with_work(&source,false,false,false,false, |n| {seen.push(n);if seen.len()==stop+1 {Err(stop)}else{Ok(())}}),Err(n) if n==stop)
            );
            assert_eq!(seen, work[..=stop]);
        }
        let p = behind_plan("(?<=(a))", false, false, false, false);
        let input = JsString::from("!aaa!");
        let mut work = Vec::new();
        let expected = p
            .find_with_work(&input, 0, false, |n| {
                work.push(n);
                Ok::<(), usize>(())
            })
            .unwrap();
        for stop in 0..work.len() {
            let mut seen = Vec::new();
            assert!(
                matches!(p.find_with_work(&input,0,false, |n| {seen.push(n);if seen.len()==stop+1 {Err(stop)}else{Ok(())}}),Err(n) if n==stop)
            );
            assert_eq!(seen, work[..=stop]);
        }
        assert_eq!(
            p.clone()
                .find_with_work(&input, 0, false, |_| Ok::<(), usize>(()))
                .unwrap(),
            expected
        );
    }

    fn behind_word_plan(source: &str, i: bool, m: bool) -> RegExpUnicodePositiveAtomMatcher {
        RegExpUnicodePositiveAtomMatcher::compile_single_lookbehind_with_word_boundaries_validated_with_work(
            &JsString::from(source), false, i, m, false, |_| Ok::<(), ()>(()),
        ).unwrap().unwrap()
    }

    #[test]
    fn single_word_lookbehind_retains_predecessor_and_empty_outside_ranges() {
        for (source, input, range) in [
            (r"((?<=((\ba\B))))", "aaa", 0..1),
            (r"(?<=(\Ba\B))", "aaaa", 1..2),
            (r"(?<=(\b[a-z]\b))", "!a!", 1..2),
            (r"(?<=(\B[\b]\B))", "\x08", 0..1),
        ] {
            let p = behind_word_plan(source, false, false);
            let m = p
                .find_with_work(&JsString::from(input), 0, false, |_| Ok::<(), ()>(()))
                .unwrap()
                .unwrap();
            assert_eq!(m.range, range.end..range.end);
            assert_eq!(m.operand, range);
            assert_eq!(p.capture_range(p.capture_count() - 1, &m), Some(range));
            if p.capture_count() == 3 {
                assert_eq!(p.capture_range(0, &m), Some(m.range.clone()));
            }
        }
    }

    #[test]
    fn single_word_lookbehind_keeps_line_fold_and_complete_surrogate_contexts() {
        let p = behind_word_plan(r"(?<=(^\ba\b$))", true, true);
        assert_eq!(
            p.find_with_work(&JsString::from("\nA\n"), 0, false, |_| Ok::<(), ()>(()))
                .unwrap()
                .unwrap()
                .operand,
            1..2
        );
        for i in [false, true] {
            let p = behind_word_plan(r"(?<=(\bſ\b))", i, false);
            let m = p
                .find_with_work(&JsString::from("!ſ!"), 0, false, |_| Ok::<(), ()>(()))
                .unwrap();
            if i {
                assert_eq!(m.unwrap().operand, 1..2);
            } else {
                assert!(m.is_none());
            }
        }
        let p = behind_word_plan(r"(?<=(\B.\b))", false, false);
        let input = JsString::from("a😀ab!");
        let m = p
            .find_with_work(&input, 0, false, |_| Ok::<(), ()>(()))
            .unwrap()
            .unwrap();
        assert_eq!(m.range, 5..5);
        assert_eq!(m.operand, 4..5);
        assert!(
            p.find_with_work(&input, 2, true, |_| Ok::<(), ()>(()))
                .unwrap()
                .is_none()
        );
    }

    #[test]
    fn single_word_lookbehind_excludes_other_counts_directions_and_reference_roles() {
        for source in [
            r"(?<=(a))",
            r"(?<=(\b))",
            r"(?<=(\ba+\b))",
            r"(?<=(\ba{1}\b))",
            r"(?<=(a\bb))",
            r"(?<=(\ba\b))a",
            r"(?=(\ba\b))",
            r"(?<!(\ba\b))",
            r"(?i:(?<=(\ba\b)))",
            r"(?<=(\b\1\b))",
        ] {
            assert!(RegExpUnicodePositiveAtomMatcher::compile_single_lookbehind_with_word_boundaries_validated_with_work(&JsString::from(source),false,false,false,false, |_| Ok::<(),()>(())).unwrap().is_none(),"{source}");
        }
        assert!(
            RegExpUnicodePositiveAtomMatcher::compile_single_lookbehind_validated_with_work(
                &JsString::from(r"(?<=(\ba\b))"),
                false,
                false,
                false,
                false,
                |_| Ok::<(), ()>(())
            )
            .unwrap()
            .is_none()
        );
    }

    #[test]
    fn single_word_lookbehind_deep_storage_and_every_work_abort_remain_flat() {
        let source = format!(
            "{}(?<={}\\ba\\b{}){}",
            "(".repeat(100_000),
            "(".repeat(100_000),
            ")".repeat(100_000),
            ")".repeat(100_000)
        );
        assert_eq!(
            behind_word_plan(&source, false, false).capture_count(),
            200_000
        );
        let source = JsString::from(r"((?<=(\ba\b)))");
        let mut work = Vec::new();
        RegExpUnicodePositiveAtomMatcher::compile_single_lookbehind_with_word_boundaries_validated_with_work(&source,false,false,false,false, |n| {work.push(n);Ok::<(),usize>(())}).unwrap().unwrap();
        for stop in 0..work.len() {
            let mut seen = Vec::new();
            assert!(
                matches!(RegExpUnicodePositiveAtomMatcher::compile_single_lookbehind_with_word_boundaries_validated_with_work(&source,false,false,false,false, |n| {seen.push(n);if seen.len()==stop+1 {Err(stop)}else{Ok(())}}),Err(n) if n==stop)
            );
            assert_eq!(seen, work[..=stop]);
        }
        let p = behind_word_plan(r"(?<=(\ba\b))", false, false);
        let input = JsString::from("!!a!");
        let mut work = Vec::new();
        let expected = p
            .find_with_work(&input, 0, false, |n| {
                work.push(n);
                Ok::<(), usize>(())
            })
            .unwrap();
        for stop in 0..work.len() {
            let mut seen = Vec::new();
            assert!(
                matches!(p.find_with_work(&input,0,false, |n| {seen.push(n);if seen.len()==stop+1 {Err(stop)}else{Ok(())}}),Err(n) if n==stop)
            );
            assert_eq!(seen, work[..=stop]);
        }
        assert_eq!(
            p.find_with_work(&input, 0, false, |_| Ok::<(), usize>(()))
                .unwrap(),
            expected
        );
    }

    fn string_class_plan(source: &str, i: bool, m: bool) -> RegExpUnicodePositiveAtomMatcher {
        RegExpUnicodePositiveAtomMatcher::compile_single_character_string_class_lookahead_validated_with_work(
            &JsString::from(source), i, m, |_| Ok::<(), ()>(()),
        ).unwrap().unwrap()
    }

    #[test]
    fn single_character_q_lookahead_retains_independent_empty_and_consuming_ranges() {
        for class in [
            r"[\q{a|😀}]",
            r"[[\q{a|😀}]--[b]]",
            r"[[\q{a|😀}]&&[a😀]]",
            r"[x\q{a|😀}]",
            r"[\q{a|\u{1F600}}]",
        ] {
            let p = string_class_plan(&format!("((?=(({class}))))"), false, false);
            assert_eq!(p.capture_count(), 3);
            let input = JsString::from("!😀a");
            let m = p
                .find_with_work(&input, 0, false, |_| Ok::<(), ()>(()))
                .unwrap()
                .unwrap();
            assert_eq!(m.range, 1..1);
            assert_eq!(m.operand, 1..3);
            assert_eq!(p.capture_range(0, &m), Some(1..1));
            assert_eq!(p.capture_range(1, &m), Some(1..3));
            assert_eq!(p.capture_range(2, &m), Some(1..3));
            assert_eq!(p.capture_range(3, &m), None);
            assert!(
                p.find_with_work(&input, 0, true, |_| Ok::<(), ()>(()))
                    .unwrap()
                    .is_none()
            );
            assert_eq!(
                p.find_with_work(&input, 2, true, |_| Ok::<(), ()>(()))
                    .unwrap()
                    .unwrap()
                    .range,
                1..1
            );
        }
    }

    #[test]
    fn single_character_q_lookahead_keeps_original_lines_folds_and_lone_surrogates() {
        let p = string_class_plan(r"(?=(^[\q{a|😀}]$))", true, true);
        let m = p
            .find_with_work(&JsString::from("\nA\n"), 0, false, |_| Ok::<(), ()>(()))
            .unwrap()
            .unwrap();
        assert_eq!(m.range, 1..1);
        assert_eq!(m.operand, 1..2);
        let p = string_class_plan(r"(?=([\q{ſ|k}]))", true, false);
        let m = p
            .find_with_work(&JsString::from("!K"), 0, false, |_| Ok::<(), ()>(()))
            .unwrap()
            .unwrap();
        assert_eq!(m.operand, 1..2);
        let p = string_class_plan(r"(?=([\q{\uD800|x}]))", false, false);
        assert!(
            p.find_with_work(&JsString::from("😀"), 0, false, |_| Ok::<(), ()>(()))
                .unwrap()
                .is_none()
        );
        let input = JsString::from_code_units(vec![0xd800, 0x78]);
        assert_eq!(
            p.find_with_work(&input, 0, false, |_| Ok::<(), ()>(()))
                .unwrap()
                .unwrap()
                .operand,
            0..1
        );
    }

    #[test]
    fn single_character_q_lookahead_declines_other_domains_and_string_widths() {
        for source in [
            r"(?=([a]))",
            r"(?=([\q{ab|a}]))",
            r"(?=([\q{|a}]))",
            r"(?=([\q{a|b}]+))",
            r"(?=([\q{a|b}]{1}))",
            r"(?=([\q{a|b}]a))",
            r"(?=([\q{a|b}]))a",
            r"(?=([\q{a|b}]))|b",
            r"((?=([\q{a|b}])))*",
            r"(?<=([\q{a|b}]))",
            r"(?!([\q{a|b}]))",
            r"(?=(\b[\q{a|b}]\b))",
            r"(?i:(?=([\q{a|b}])))",
            r"((?=(\1)))",
        ] {
            assert!(RegExpUnicodePositiveAtomMatcher::compile_single_character_string_class_lookahead_validated_with_work(&JsString::from(source), false, false, |_| Ok::<(), ()>(())).unwrap().is_none(), "{source}");
        }
    }

    #[test]
    fn single_character_q_lookahead_deep_scopes_and_checked_work_abort_recovery_stay_flat() {
        let source = format!(
            "{}(?={}[\\q{{a|😀}}]{}){}",
            "(".repeat(100_000),
            "(".repeat(100_000),
            ")".repeat(100_000),
            ")".repeat(100_000)
        );
        let p = string_class_plan(&source, false, false);
        assert_eq!(p.capture_count(), 200_000);
        let m = p
            .clone()
            .find_with_work(&JsString::from("😀"), 0, true, |_| Ok::<(), ()>(()))
            .unwrap()
            .unwrap();
        assert_eq!(p.capture_range(99_999, &m), Some(0..0));
        assert_eq!(p.capture_range(100_000, &m), Some(0..2));
        let source = JsString::from(r"((?=([\q{a|😀}])))");
        let mut work = Vec::new();
        let p=RegExpUnicodePositiveAtomMatcher::compile_single_character_string_class_lookahead_validated_with_work(&source,false,false,|n|{work.push(n);Ok::<(),usize>(())}).unwrap().unwrap();
        assert_eq!(work[0], source.len() * 16 + 16);
        for stop in 0..work.len() {
            let mut seen = Vec::new();
            assert!(
                matches!(RegExpUnicodePositiveAtomMatcher::compile_single_character_string_class_lookahead_validated_with_work(&source,false,false,|n|{seen.push(n);if seen.len()==stop+1{Err(stop)}else{Ok(())}}),Err(crate::RegExpUnicodeSetCompileError::Work(n))if n==stop)
            );
            assert_eq!(seen, work[..=stop]);
        }
        let input = JsString::from("!!😀");
        work.clear();
        let expected = p
            .find_with_work(&input, 0, false, |n| {
                work.push(n);
                Ok::<(), usize>(())
            })
            .unwrap();
        for stop in 0..work.len() {
            let mut seen = Vec::new();
            assert!(
                matches!(p.find_with_work(&input,0,false,|n|{seen.push(n);if seen.len()==stop+1{Err(stop)}else{Ok(())}}),Err(n)if n==stop)
            );
            assert_eq!(seen, work[..=stop]);
        }
        assert_eq!(
            p.find_with_work(&input, 0, false, |_| Ok::<(), usize>(()))
                .unwrap(),
            expected
        );
    }

    fn word_string_class_plan(source: &str, i: bool, m: bool) -> RegExpUnicodePositiveAtomMatcher {
        RegExpUnicodePositiveAtomMatcher::compile_single_character_string_class_lookahead_with_word_boundaries_validated_with_work(
            &JsString::from(source), i, m, |_| Ok::<(), ()>(()),
        ).unwrap().unwrap()
    }

    #[test]
    fn single_character_q_word_lookahead_retains_independent_ranges_and_boundary_polarity() {
        let p = word_string_class_plan(r"((?=((\b[\q{a|b}]\b))))", false, false);
        let m = p
            .find_with_work(&JsString::from("ab a!"), 0, false, |_| Ok::<(), ()>(()))
            .unwrap()
            .unwrap();
        assert_eq!(m.range, 3..3);
        assert_eq!(m.operand, 3..4);
        assert_eq!(p.capture_count(), 3);
        assert_eq!(p.capture_range(0, &m), Some(3..3));
        assert_eq!(p.capture_range(1, &m), Some(3..4));
        assert_eq!(p.capture_range(2, &m), Some(3..4));
        assert_eq!(p.capture_range(3, &m), None);
        let p = word_string_class_plan(r"(?=(\b[\q{a|b}]\B))", false, false);
        assert_eq!(
            p.find_with_work(&JsString::from("ab!"), 0, true, |_| Ok::<(), ()>(()))
                .unwrap()
                .unwrap()
                .operand,
            0..1
        );
        assert!(
            p.find_with_work(&JsString::from("a!"), 0, true, |_| Ok::<(), ()>(()))
                .unwrap()
                .is_none()
        );
    }

    #[test]
    fn single_character_q_word_lookahead_keeps_original_line_fold_backspace_and_pair_context() {
        let p = word_string_class_plan(r"(?=(^\b[\q{a|😀}]\b$))", true, true);
        assert_eq!(
            p.find_with_work(&JsString::from("\nA\n"), 0, false, |_| Ok::<(), ()>(()))
                .unwrap()
                .unwrap()
                .operand,
            1..2
        );
        let p = word_string_class_plan(r"(?=(\b[\q{ſ|k}]\b))", true, false);
        for text in ["!S!", "!ſ!", "!K!", "!K!"] {
            assert_eq!(
                p.find_with_work(&JsString::from(text), 0, false, |_| Ok::<(), ()>(()))
                    .unwrap()
                    .unwrap()
                    .operand,
                1..2
            );
        }
        let p = word_string_class_plan(r"(?=(\B[\q{\u0008|a}]\B))", false, false);
        assert_eq!(
            p.find_with_work(&JsString::from("\u{8}"), 0, true, |_| Ok::<(), ()>(()))
                .unwrap()
                .unwrap()
                .operand,
            0..1
        );
        let p = word_string_class_plan(r"(?=(\B[\q{😀|a}]\B))", false, false);
        let m = p
            .find_with_work(&JsString::from("😀😀"), 1, true, |_| Ok::<(), ()>(()))
            .unwrap()
            .unwrap();
        assert_eq!(m.range, 0..0);
        assert_eq!(m.operand, 0..2);
    }

    #[test]
    fn single_character_q_word_lookahead_declines_other_members_counts_and_scopes() {
        for source in [
            r"(?=([\q{a|b}]))",
            r"(?=(\b[ab]\b))",
            r"(?=(\b[\q{ab|a}]\b))",
            r"(?=(\b[\q{|a}]\b))",
            r"(?=(\b[\q{a|b}]+\b))",
            r"(?=(\b[\q{a|b}]{1}\b))",
            r"(?=(\b[\q{a|b}]\b))a",
            r"(?=(\b[\q{a|b}]\b))|b",
            r"((?=(\b[\q{a|b}]\b)))*",
            r"(?<=(\b[\q{a|b}]\b))",
            r"(?!(\b[\q{a|b}]\b))",
            r"(?=(\b([\q{a|b}])\b))",
            r"((?=(\b\1\b)))",
        ] {
            assert!(RegExpUnicodePositiveAtomMatcher::compile_single_character_string_class_lookahead_with_word_boundaries_validated_with_work(&JsString::from(source),false,false, |_| Ok::<(), ()>(())).unwrap().is_none(),"{source}");
        }
    }

    #[test]
    fn single_character_q_word_lookahead_deep_storage_and_every_work_abort_remain_flat() {
        let source = format!(
            "{}(?={}\\b[\\q{{a|b}}]\\b{}){}",
            "(".repeat(100_000),
            "(".repeat(100_000),
            ")".repeat(100_000),
            ")".repeat(100_000)
        );
        let p = word_string_class_plan(&source, false, false);
        assert_eq!(p.capture_count(), 200_000);
        let m = p
            .clone()
            .find_with_work(&JsString::from("a!"), 0, true, |_| Ok::<(), ()>(()))
            .unwrap()
            .unwrap();
        assert_eq!(p.capture_range(99_999, &m), Some(0..0));
        assert_eq!(p.capture_range(100_000, &m), Some(0..1));
        let source = JsString::from(r"((?=(\b[\q{a|b}]\b)))");
        let mut work = Vec::new();
        let p=RegExpUnicodePositiveAtomMatcher::compile_single_character_string_class_lookahead_with_word_boundaries_validated_with_work(&source,false,false,|n|{work.push(n);Ok::<(),usize>(())}).unwrap().unwrap();
        assert_eq!(work[0], source.len() * 16 + 16);
        for stop in 0..work.len() {
            let mut seen = Vec::new();
            assert!(
                matches!(RegExpUnicodePositiveAtomMatcher::compile_single_character_string_class_lookahead_with_word_boundaries_validated_with_work(&source,false,false,|n|{seen.push(n);if seen.len()==stop+1{Err(stop)}else{Ok(())}}),Err(crate::RegExpUnicodeSetCompileError::Work(n))if n==stop)
            );
            assert_eq!(seen, work[..=stop]);
        }
        let input = JsString::from("!!a!");
        work.clear();
        let expected = p
            .find_with_work(&input, 0, false, |n| {
                work.push(n);
                Ok::<(), usize>(())
            })
            .unwrap();
        for stop in 0..work.len() {
            let mut seen = Vec::new();
            assert!(
                matches!(p.find_with_work(&input,0,false,|n|{seen.push(n);if seen.len()==stop+1{Err(stop)}else{Ok(())}}),Err(n)if n==stop)
            );
            assert_eq!(seen, work[..=stop]);
        }
        assert_eq!(
            p.find_with_work(&input, 0, false, |_| Ok::<(), usize>(()))
                .unwrap(),
            expected
        );
    }

    fn behind_string_class_plan(
        source: &str,
        i: bool,
        m: bool,
    ) -> RegExpUnicodePositiveAtomMatcher {
        RegExpUnicodePositiveAtomMatcher::compile_single_character_string_class_lookbehind_validated_with_work(
            &JsString::from(source),i,m, |_| Ok::<(), ()>(()),
        ).unwrap().unwrap()
    }

    #[test]
    fn single_character_q_lookbehind_captures_the_predecessor_before_empty_outside_spans() {
        for class in [
            r"[\q{a|😀}]",
            r"[[\q{a|😀}]--[b]]",
            r"[[\q{a|😀}]&&[a😀]]",
            r"[x\q{a|😀}]",
        ] {
            let p = behind_string_class_plan(
                &format!("((?=(({class}))))").replacen("(?=", "(?<=", 1),
                false,
                false,
            );
            let input = JsString::from("!😀a");
            let m = p
                .find_with_work(&input, 0, false, |_| Ok::<(), ()>(()))
                .unwrap()
                .unwrap();
            assert_eq!(p.capture_count(), 3);
            assert_eq!(m.range, 3..3);
            assert_eq!(m.operand, 1..3);
            assert_eq!(p.capture_range(0, &m), Some(3..3));
            assert_eq!(p.capture_range(1, &m), Some(1..3));
            assert_eq!(p.capture_range(2, &m), Some(1..3));
            assert_eq!(p.capture_range(3, &m), None);
            assert!(
                p.find_with_work(&input, 2, true, |_| Ok::<(), ()>(()))
                    .unwrap()
                    .is_none()
            );
            assert_eq!(
                p.find_with_work(&input, 4, true, |_| Ok::<(), ()>(()))
                    .unwrap()
                    .unwrap()
                    .operand,
                3..4
            );
            assert!(
                p.find_with_work(&JsString::from(""), 0, false, |_| Ok::<(), ()>(()))
                    .unwrap()
                    .is_none()
            );
        }
    }

    #[test]
    fn single_character_q_lookbehind_retains_original_line_fold_and_surrogate_context() {
        let p = behind_string_class_plan(r"(?<=(^[\q{a|😀}]$))", true, true);
        let m = p
            .find_with_work(&JsString::from("\nA\n"), 0, false, |_| Ok::<(), ()>(()))
            .unwrap()
            .unwrap();
        assert_eq!(m.range, 2..2);
        assert_eq!(m.operand, 1..2);
        let p = behind_string_class_plan(r"(?<=([\q{ſ|k}]))", true, false);
        for text in ["!S", "!ſ", "!K", "!K"] {
            assert_eq!(
                p.find_with_work(&JsString::from(text), 0, false, |_| Ok::<(), ()>(()))
                    .unwrap()
                    .unwrap()
                    .operand,
                1..2
            );
        }
        let p = behind_string_class_plan(r"(?<=([\q{\uD800|x}]))", false, false);
        assert!(
            p.find_with_work(&JsString::from("😀"), 0, false, |_| Ok::<(), ()>(()))
                .unwrap()
                .is_none()
        );
        let input = JsString::from_code_units(vec![0xd800, 0x78]);
        assert_eq!(
            p.find_with_work(&input, 0, false, |_| Ok::<(), ()>(()))
                .unwrap()
                .unwrap()
                .operand,
            0..1
        );
    }

    #[test]
    fn single_character_q_lookbehind_declines_other_domains_widths_counts_and_scopes() {
        for source in [
            r"(?<=([a]))",
            r"(?<=([\q{ab|a}]))",
            r"(?<=([\q{|a}]))",
            r"(?<=([\q{a|b}]+))",
            r"(?<=([\q{a|b}]{1}))",
            r"(?<=([\q{a|b}]a))",
            r"(?<=([\q{a|b}]))a",
            r"(?<=([\q{a|b}]))|b",
            r"((?<=([\q{a|b}])))*",
            r"(?=([\q{a|b}]))",
            r"(?<!([\q{a|b}]))",
            r"(?<=(\b[\q{a|b}]\b))",
            r"(?i:(?<=([\q{a|b}])))",
            r"((?<=(\1)))",
        ] {
            assert!(RegExpUnicodePositiveAtomMatcher::compile_single_character_string_class_lookbehind_validated_with_work(&JsString::from(source),false,false, |_| Ok::<(), ()>(())).unwrap().is_none(),"{source}");
        }
    }

    #[test]
    fn single_character_q_lookbehind_deep_scopes_linear_retries_and_work_recovery_stay_flat() {
        let source = format!(
            "{}(?<={}[\\q{{a|😀}}]{}){}",
            "(".repeat(100_000),
            "(".repeat(100_000),
            ")".repeat(100_000),
            ")".repeat(100_000)
        );
        let p = behind_string_class_plan(&source, false, false);
        assert_eq!(p.capture_count(), 200_000);
        let m = p
            .clone()
            .find_with_work(&JsString::from("😀"), 2, true, |_| Ok::<(), ()>(()))
            .unwrap()
            .unwrap();
        assert_eq!(p.capture_range(99_999, &m), Some(2..2));
        assert_eq!(p.capture_range(100_000, &m), Some(0..2));
        let p = behind_string_class_plan(r"(?<=([\q{a|b}]$))", false, false);
        let input = JsString::from(("a".repeat(20_000) + "!").as_str());
        let mut total = 0usize;
        assert!(
            p.find_with_work(&input, 0, false, |n| {
                total += n;
                Ok::<(), ()>(())
            })
            .unwrap()
            .is_none()
        );
        assert!(total < 250 * input.len() + 128, "{total}");
        let source = JsString::from(r"((?<=([\q{a|😀}])))");
        let mut work = Vec::new();
        let p=RegExpUnicodePositiveAtomMatcher::compile_single_character_string_class_lookbehind_validated_with_work(&source,false,false,|n|{work.push(n);Ok::<(),usize>(())}).unwrap().unwrap();
        assert_eq!(work[0], source.len() * 16 + 16);
        for stop in 0..work.len() {
            let mut seen = Vec::new();
            assert!(
                matches!(RegExpUnicodePositiveAtomMatcher::compile_single_character_string_class_lookbehind_validated_with_work(&source,false,false,|n|{seen.push(n);if seen.len()==stop+1{Err(stop)}else{Ok(())}}),Err(crate::RegExpUnicodeSetCompileError::Work(n))if n==stop)
            );
            assert_eq!(seen, work[..=stop]);
        }
        let input = JsString::from("!!😀");
        work.clear();
        let expected = p
            .find_with_work(&input, 0, false, |n| {
                work.push(n);
                Ok::<(), usize>(())
            })
            .unwrap();
        for stop in 0..work.len() {
            let mut seen = Vec::new();
            assert!(
                matches!(p.find_with_work(&input,0,false,|n|{seen.push(n);if seen.len()==stop+1{Err(stop)}else{Ok(())}}),Err(n)if n==stop)
            );
            assert_eq!(seen, work[..=stop]);
        }
        assert_eq!(
            p.find_with_work(&input, 0, false, |_| Ok::<(), usize>(()))
                .unwrap(),
            expected
        );
    }

    fn behind_word_string_class_plan(
        source: &str,
        i: bool,
        m: bool,
    ) -> RegExpUnicodePositiveAtomMatcher {
        RegExpUnicodePositiveAtomMatcher::compile_single_character_string_class_lookbehind_with_word_boundaries_validated_with_work(
            &JsString::from(source),i,m, |_| Ok::<(), ()>(()),
        ).unwrap().unwrap()
    }

    #[test]
    fn single_character_q_word_lookbehind_keeps_predecessor_ranges_and_predicate_polarity() {
        let p = behind_word_string_class_plan(r"((?<=((\b[\q{a|b}]\b))))", false, false);
        let m = p
            .find_with_work(&JsString::from("ab a!"), 0, false, |_| Ok::<(), ()>(()))
            .unwrap()
            .unwrap();
        assert_eq!(m.range, 4..4);
        assert_eq!(m.operand, 3..4);
        assert_eq!(p.capture_count(), 3);
        assert_eq!(p.capture_range(0, &m), Some(4..4));
        assert_eq!(p.capture_range(1, &m), Some(3..4));
        assert_eq!(p.capture_range(2, &m), Some(3..4));
        assert_eq!(p.capture_range(3, &m), None);
        let p = behind_word_string_class_plan(r"(?<=(\b[\q{a|b}]\B))", false, false);
        assert_eq!(
            p.find_with_work(&JsString::from("ab!"), 1, true, |_| Ok::<(), ()>(()))
                .unwrap()
                .unwrap()
                .operand,
            0..1
        );
        assert!(
            p.find_with_work(&JsString::from("a!"), 1, true, |_| Ok::<(), ()>(()))
                .unwrap()
                .is_none()
        );
    }

    #[test]
    fn single_character_q_word_lookbehind_keeps_original_line_fold_backspace_and_pair_context() {
        let p = behind_word_string_class_plan(r"(?<=(^\b[\q{a|😀}]\b$))", true, true);
        let m = p
            .find_with_work(&JsString::from("\nA\n"), 0, false, |_| Ok::<(), ()>(()))
            .unwrap()
            .unwrap();
        assert_eq!(m.range, 2..2);
        assert_eq!(m.operand, 1..2);
        let p = behind_word_string_class_plan(r"(?<=(\b[\q{ſ|k}]\b))", true, false);
        for text in ["!S!", "!ſ!", "!K!", "!K!"] {
            assert_eq!(
                p.find_with_work(&JsString::from(text), 0, false, |_| Ok::<(), ()>(()))
                    .unwrap()
                    .unwrap()
                    .operand,
                1..2
            );
        }
        let p = behind_word_string_class_plan(r"(?<=(\B[\q{\u0008|a}]\B))", false, false);
        assert_eq!(
            p.find_with_work(&JsString::from("\u{8}"), 1, true, |_| Ok::<(), ()>(()))
                .unwrap()
                .unwrap()
                .operand,
            0..1
        );
        let p = behind_word_string_class_plan(r"(?<=(\B[\q{😀|a}]\B))", false, false);
        let m = p
            .find_with_work(&JsString::from("😀😀"), 3, true, |_| Ok::<(), ()>(()))
            .unwrap()
            .unwrap();
        assert_eq!(m.range, 2..2);
        assert_eq!(m.operand, 0..2);
        assert!(
            p.find_with_work(&JsString::from("x😀"), 0, false, |_| Ok::<(), ()>(()))
                .unwrap()
                .is_none()
        );
    }

    #[test]
    fn single_character_q_word_lookbehind_declines_other_members_counts_and_scopes() {
        for source in [
            r"(?<=([\q{a|b}]))",
            r"(?<=(\b[ab]\b))",
            r"(?<=(\b[\q{ab|a}]\b))",
            r"(?<=(\b[\q{|a}]\b))",
            r"(?<=(\b[\q{a|b}]+\b))",
            r"(?<=(\b[\q{a|b}]{1}\b))",
            r"(?<=(\b[\q{a|b}]\b))a",
            r"(?<=(\b[\q{a|b}]\b))|b",
            r"((?<=(\b[\q{a|b}]\b)))*",
            r"(?=(\b[\q{a|b}]\b))",
            r"(?<!(\b[\q{a|b}]\b))",
            r"(?<=(\b([\q{a|b}])\b))",
            r"((?<=(\b\1\b)))",
        ] {
            assert!(RegExpUnicodePositiveAtomMatcher::compile_single_character_string_class_lookbehind_with_word_boundaries_validated_with_work(&JsString::from(source),false,false, |_| Ok::<(), ()>(())).unwrap().is_none(),"{source}");
        }
    }

    #[test]
    fn single_character_q_word_lookbehind_deep_storage_and_every_work_abort_remain_flat() {
        let source = format!(
            "{}(?<={}\\b[\\q{{a|b}}]\\b{}){}",
            "(".repeat(100_000),
            "(".repeat(100_000),
            ")".repeat(100_000),
            ")".repeat(100_000)
        );
        let p = behind_word_string_class_plan(&source, false, false);
        assert_eq!(p.capture_count(), 200_000);
        let m = p
            .clone()
            .find_with_work(&JsString::from("a!"), 1, true, |_| Ok::<(), ()>(()))
            .unwrap()
            .unwrap();
        assert_eq!(p.capture_range(99_999, &m), Some(1..1));
        assert_eq!(p.capture_range(100_000, &m), Some(0..1));
        let source = JsString::from(r"((?<=(\b[\q{a|b}]\b)))");
        let mut work = Vec::new();
        let p=RegExpUnicodePositiveAtomMatcher::compile_single_character_string_class_lookbehind_with_word_boundaries_validated_with_work(&source,false,false,|n|{work.push(n);Ok::<(),usize>(())}).unwrap().unwrap();
        assert_eq!(work[0], source.len() * 16 + 16);
        for stop in 0..work.len() {
            let mut seen = Vec::new();
            assert!(
                matches!(RegExpUnicodePositiveAtomMatcher::compile_single_character_string_class_lookbehind_with_word_boundaries_validated_with_work(&source,false,false,|n|{seen.push(n);if seen.len()==stop+1{Err(stop)}else{Ok(())}}),Err(crate::RegExpUnicodeSetCompileError::Work(n))if n==stop)
            );
            assert_eq!(seen, work[..=stop]);
        }
        let input = JsString::from("!!a!");
        work.clear();
        let expected = p
            .find_with_work(&input, 0, false, |n| {
                work.push(n);
                Ok::<(), usize>(())
            })
            .unwrap();
        for stop in 0..work.len() {
            let mut seen = Vec::new();
            assert!(
                matches!(p.find_with_work(&input,0,false,|n|{seen.push(n);if seen.len()==stop+1{Err(stop)}else{Ok(())}}),Err(n)if n==stop)
            );
            assert_eq!(seen, work[..=stop]);
        }
        assert_eq!(
            p.find_with_work(&input, 0, false, |_| Ok::<(), usize>(()))
                .unwrap(),
            expected
        );
    }
    fn scalar_set_plan(source: &str, i: bool, m: bool) -> RegExpUnicodePositiveAtomMatcher {
        RegExpUnicodePositiveAtomMatcher::compile_single_v_set_lookahead_validated_with_work(
            &JsString::from(source),
            i,
            m,
            |_| Ok::<(), ()>(()),
        )
        .unwrap()
        .unwrap()
    }

    #[test]
    fn scalar_v_set_lookahead_keeps_original_character_and_independent_capture_ranges() {
        for (class, text, start, end) in [
            (r"[[a-z]--[aeiou]]", "!ab", 2, 3),
            (r"[[a-z]&&[ab]]", "!ab", 1, 2),
            (r"[[a][b]]", "!ab", 1, 2),
            (r"[[😀a]--[a]]", "!😀a", 1, 3),
            (r"[^[a-z]]", "😀abc", 0, 2),
        ] {
            let p = scalar_set_plan(&format!("((?=(({class}))))"), false, false);
            let input = JsString::from(text);
            let m = p
                .find_with_work(&input, 0, false, |_| Ok::<(), ()>(()))
                .unwrap()
                .unwrap();
            assert_eq!(m.range, start..start);
            assert_eq!(m.operand, start..end);
            assert_eq!(p.capture_count(), 3);
            assert_eq!(p.capture_range(0, &m), Some(start..start));
            assert_eq!(p.capture_range(1, &m), Some(start..end));
            assert_eq!(p.capture_range(2, &m), Some(start..end));
            assert_eq!(p.capture_range(3, &m), None);
        }
    }

    #[test]
    fn scalar_v_set_lookahead_keeps_original_lines_folds_empty_sets_and_surrogate_boundaries() {
        let p = scalar_set_plan(r"(?=(^[[a-z]--[aeiou]]$))", true, true);
        assert_eq!(
            p.find_with_work(&JsString::from("\nB\n"), 0, false, |_| Ok::<(), ()>(()))
                .unwrap()
                .unwrap()
                .operand,
            1..2
        );
        let p = scalar_set_plan(r"(?=([[\p{ASCII}]--[x]]))", true, false);
        assert_eq!(
            p.find_with_work(&JsString::from("ſK"), 0, false, |_| Ok::<(), ()>(()))
                .unwrap()
                .unwrap()
                .operand,
            0..1
        );
        let p = scalar_set_plan(r"(?=([^[\p{ASCII}]]))", true, false);
        assert_eq!(
            p.find_with_work(&JsString::from("ſK😀"), 0, false, |_| Ok::<(), ()>(()))
                .unwrap()
                .unwrap()
                .operand,
            2..4
        );
        let p = scalar_set_plan(r"(?=([\p{Script=Greek}&&[Α-Ωα-ω]]))", true, false);
        assert_eq!(
            p.find_with_work(&JsString::from("!Σ"), 0, false, |_| Ok::<(), ()>(()))
                .unwrap()
                .unwrap()
                .operand,
            1..2
        );
        let p = scalar_set_plan(r"(?=([[a-z]--[a-z]]))", false, false);
        assert!(
            p.find_with_work(&JsString::from("abc"), 0, false, |_| Ok::<(), ()>(()))
                .unwrap()
                .is_none()
        );
        let p = scalar_set_plan(r"(?=([[\uD800]--[x]]))", false, false);
        assert!(
            p.find_with_work(&JsString::from("😀"), 0, false, |_| Ok::<(), ()>(()))
                .unwrap()
                .is_none()
        );
        let input = JsString::from_code_units(vec![0xd800, 0x78]);
        assert_eq!(
            p.find_with_work(&input, 0, true, |_| Ok::<(), ()>(()))
                .unwrap()
                .unwrap()
                .operand,
            0..1
        );
        let p = scalar_set_plan(r"(?=([[😀a]--[a]]))", false, false);
        let m = p
            .find_with_work(&JsString::from("!😀a"), 2, true, |_| Ok::<(), ()>(()))
            .unwrap()
            .unwrap();
        assert_eq!(m.range, 1..1);
        assert_eq!(m.operand, 1..3);
    }

    #[test]
    fn scalar_v_set_lookahead_declines_flat_classes_strings_counts_and_other_scopes() {
        for source in [
            r"(?=([a]))",
            r"(?=([\q{a|b}]))",
            r"(?=([[\q{a|b}]--[x]]))",
            r"(?=([[\q{ab|a}]--[x]]))",
            r"(?=([[a-z]--[x]]+))",
            r"(?=([[a-z]--[x]]{1}))",
            r"(?=([[a-z]--[x]]a))",
            r"(?=([[a-z]--[x]]))a",
            r"(?=([[a-z]--[x]]))|b",
            r"((?=([[a-z]--[x]])))*",
            r"(?<=([[a-z]--[x]]))",
            r"(?!([[a-z]--[x]]))",
            r"(?=(\b[[a-z]--[x]]\b))",
            r"(?i:(?=([[a-z]--[x]])))",
            r"((?=(\1)))",
        ] {
            assert!(RegExpUnicodePositiveAtomMatcher::compile_single_v_set_lookahead_validated_with_work(&JsString::from(source),false,false, |_| Ok::<(), ()>(())).unwrap().is_none(),"{source}");
        }
    }

    #[test]
    fn scalar_v_set_lookahead_deep_scopes_classes_and_every_work_abort_remain_flat() {
        let source = format!(
            "{}(?={}[[a-z]--[aeiou]]{}){}",
            "(".repeat(100_000),
            "(".repeat(100_000),
            ")".repeat(100_000),
            ")".repeat(100_000)
        );
        let p = scalar_set_plan(&source, false, false);
        assert_eq!(p.capture_count(), 200_000);
        let m = p
            .clone()
            .find_with_work(&JsString::from("b!"), 0, true, |_| Ok::<(), ()>(()))
            .unwrap()
            .unwrap();
        assert_eq!(p.capture_range(99_999, &m), Some(0..0));
        assert_eq!(p.capture_range(100_000, &m), Some(0..1));
        let source = format!("(?=({}a{}))", "[".repeat(20_000), "]".repeat(20_000));
        let p = scalar_set_plan(&source, false, false);
        assert_eq!(
            p.find_with_work(&JsString::from("a"), 0, true, |_| Ok::<(), ()>(()))
                .unwrap()
                .unwrap()
                .operand,
            0..1
        );
        let source = JsString::from(r"((?=([[a-z]--[aeiou]])))");
        let mut work = Vec::new();
        let p =
            RegExpUnicodePositiveAtomMatcher::compile_single_v_set_lookahead_validated_with_work(
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
                matches!(RegExpUnicodePositiveAtomMatcher::compile_single_v_set_lookahead_validated_with_work(&source,false,false,|n|{seen.push(n);if seen.len()==stop+1{Err(stop)}else{Ok(())}}),Err(crate::RegExpUnicodeSetCompileError::Work(n))if n==stop)
            );
            assert_eq!(seen, work[..=stop]);
        }
        let input = JsString::from("!ab");
        work.clear();
        let expected = p
            .find_with_work(&input, 0, false, |n| {
                work.push(n);
                Ok::<(), usize>(())
            })
            .unwrap();
        for stop in 0..work.len() {
            let mut seen = Vec::new();
            assert!(
                matches!(p.find_with_work(&input,0,false,|n|{seen.push(n);if seen.len()==stop+1{Err(stop)}else{Ok(())}}),Err(n)if n==stop)
            );
            assert_eq!(seen, work[..=stop]);
        }
        assert_eq!(
            p.find_with_work(&input, 0, false, |_| Ok::<(), usize>(()))
                .unwrap(),
            expected
        );
    }
    fn word_scalar_set_plan(source: &str, i: bool, m: bool) -> RegExpUnicodePositiveAtomMatcher {
        RegExpUnicodePositiveAtomMatcher::compile_single_v_set_lookahead_with_word_boundaries_validated_with_work(
            &JsString::from(source),i,m, |_| Ok::<(), ()>(()),
        ).unwrap().unwrap()
    }

    #[test]
    fn scalar_v_set_word_lookahead_keeps_original_polarity_and_independent_capture_ranges() {
        let p = word_scalar_set_plan(r"((?=((\b[[a-z]--[aeiou]]\b))))", false, false);
        let m = p
            .find_with_work(&JsString::from("ab b!"), 0, false, |_| Ok::<(), ()>(()))
            .unwrap()
            .unwrap();
        assert_eq!(m.range, 3..3);
        assert_eq!(m.operand, 3..4);
        assert_eq!(p.capture_count(), 3);
        assert_eq!(p.capture_range(0, &m), Some(3..3));
        assert_eq!(p.capture_range(1, &m), Some(3..4));
        assert_eq!(p.capture_range(2, &m), Some(3..4));
        assert_eq!(p.capture_range(3, &m), None);
        let p = word_scalar_set_plan(r"(?=(\B[[a-z]--[aeiou]]\b))", false, false);
        assert_eq!(
            p.find_with_work(&JsString::from("ab!"), 0, false, |_| Ok::<(), ()>(()))
                .unwrap()
                .unwrap()
                .operand,
            1..2
        );
        let p = word_scalar_set_plan(r"(?=(\b[[a-z]--[aeiou]]\B))", false, false);
        assert_eq!(
            p.find_with_work(&JsString::from("bb!"), 0, true, |_| Ok::<(), ()>(()))
                .unwrap()
                .unwrap()
                .operand,
            0..1
        );
    }

    #[test]
    fn scalar_v_set_word_lookahead_keeps_original_line_fold_backspace_and_pair_context() {
        let p = word_scalar_set_plan(r"(?=(^\b[[a-z]--[aeiou]]\b$))", true, true);
        assert_eq!(
            p.find_with_work(&JsString::from("\nB\n"), 0, false, |_| Ok::<(), ()>(()))
                .unwrap()
                .unwrap()
                .operand,
            1..2
        );
        let p = word_scalar_set_plan(r"(?=(\b[[\p{ASCII}]--[x]]\b))", true, false);
        for text in ["!S!", "!ſ!", "!K!", "!K!"] {
            assert_eq!(
                p.find_with_work(&JsString::from(text), 0, false, |_| Ok::<(), ()>(()))
                    .unwrap()
                    .unwrap()
                    .operand,
                1..2
            );
        }
        let p = word_scalar_set_plan(r"(?=(\B[[\b]--[a]]\B))", false, false);
        assert_eq!(
            p.find_with_work(&JsString::from("\u{8}"), 0, true, |_| Ok::<(), ()>(()))
                .unwrap()
                .unwrap()
                .operand,
            0..1
        );
        let p = word_scalar_set_plan(r"(?=(\B[[😀a]--[a]]\b))", false, false);
        let m = p
            .find_with_work(&JsString::from("!😀a"), 2, true, |_| Ok::<(), ()>(()))
            .unwrap()
            .unwrap();
        assert_eq!(m.range, 1..1);
        assert_eq!(m.operand, 1..3);
    }

    #[test]
    fn scalar_v_set_word_lookahead_declines_other_domains_counts_and_scopes() {
        for source in [
            r"(?=([[a-z]--[x]]))",
            r"(?=(\b[ab]\b))",
            r"(?=(\b[\q{a|b}]\b))",
            r"(?=(\b[[\q{a|b}]--[x]]\b))",
            r"(?=(\b[[a-z]--[x]]+\b))",
            r"(?=(\b[[a-z]--[x]]{1}\b))",
            r"(?=(\b[[a-z]--[x]]\b))a",
            r"(?=(\b[[a-z]--[x]]\b))|b",
            r"((?=(\b[[a-z]--[x]]\b)))*",
            r"(?<=(\b[[a-z]--[x]]\b))",
            r"(?!(\b[[a-z]--[x]]\b))",
            r"(?=(\b([[a-z]--[x]])\b))",
            r"((?=(\b\1\b)))",
        ] {
            assert!(RegExpUnicodePositiveAtomMatcher::compile_single_v_set_lookahead_with_word_boundaries_validated_with_work(&JsString::from(source),false,false, |_| Ok::<(), ()>(())).unwrap().is_none(),"{source}");
        }
    }

    #[test]
    fn scalar_v_set_word_lookahead_deep_scopes_and_every_work_abort_remain_flat() {
        let source = format!(
            "{}(?={}\\b[[a-z]--[aeiou]]\\b{}){}",
            "(".repeat(100_000),
            "(".repeat(100_000),
            ")".repeat(100_000),
            ")".repeat(100_000)
        );
        let p = word_scalar_set_plan(&source, false, false);
        assert_eq!(p.capture_count(), 200_000);
        let m = p
            .clone()
            .find_with_work(&JsString::from("b!"), 0, true, |_| Ok::<(), ()>(()))
            .unwrap()
            .unwrap();
        assert_eq!(p.capture_range(99_999, &m), Some(0..0));
        assert_eq!(p.capture_range(100_000, &m), Some(0..1));
        let source = JsString::from(r"((?=(\b[[a-z]--[aeiou]]\b)))");
        let mut work = Vec::new();
        let p=RegExpUnicodePositiveAtomMatcher::compile_single_v_set_lookahead_with_word_boundaries_validated_with_work(&source,false,false,|n|{work.push(n);Ok::<(),usize>(())}).unwrap().unwrap();
        assert_eq!(work[0], source.len() * 16 + 16);
        for stop in 0..work.len() {
            let mut seen = Vec::new();
            assert!(
                matches!(RegExpUnicodePositiveAtomMatcher::compile_single_v_set_lookahead_with_word_boundaries_validated_with_work(&source,false,false,|n|{seen.push(n);if seen.len()==stop+1{Err(stop)}else{Ok(())}}),Err(crate::RegExpUnicodeSetCompileError::Work(n))if n==stop)
            );
            assert_eq!(seen, work[..=stop]);
        }
        let input = JsString::from("!ab b!");
        work.clear();
        let expected = p
            .find_with_work(&input, 0, false, |n| {
                work.push(n);
                Ok::<(), usize>(())
            })
            .unwrap();
        for stop in 0..work.len() {
            let mut seen = Vec::new();
            assert!(
                matches!(p.find_with_work(&input,0,false,|n|{seen.push(n);if seen.len()==stop+1{Err(stop)}else{Ok(())}}),Err(n)if n==stop)
            );
            assert_eq!(seen, work[..=stop]);
        }
        assert_eq!(
            p.find_with_work(&input, 0, false, |_| Ok::<(), usize>(()))
                .unwrap(),
            expected
        );
    }
}
