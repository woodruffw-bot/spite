//! Negative lookahead over variable-length UnicodeSets q classes (22.2.2.4).

use crate::{
    JsString, RegExpUnicodePositiveStringClassMatcher, RegExpUnicodeSetCompileError,
    regexp_literal::unicode_start, regexp_zero_count_group::enclosing_group_body,
};
use std::ops::Range;

/// Complete negative lookahead over a variable-length UnicodeSets q class.
///
/// Full original v/vi validation and normalized capture heads are required.
/// Complete wrappers preserve the lexical inventory. Outside captures select
/// the empty assertion range; all operand captures remain undefined on success.
#[derive(Clone, Debug)]
pub struct RegExpUnicodeNegativeStringClassMatcher {
    positive: RegExpUnicodePositiveStringClassMatcher,
    outside_captures: usize,
}

impl RegExpUnicodeNegativeStringClassMatcher {
    /// Compiles complete negative lookahead with optional operand ^/$ gates.
    ///
    /// Only the proved assertion sign is projected; the complete positive proof
    /// checks the class, wrappers and binding roles without separate validation
    /// or rebinding. Work and fallible projection precede source copying.
    pub fn compile_single_q_lookahead_validated_with_work<E>(
        source: &JsString,
        ignore_case: bool,
        multiline: bool,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, RegExpUnicodeSetCompileError<E>> {
        Self::compile_with_work_impl::<false, 0, E>(source, ignore_case, multiline, &mut charge)
    }

    /// Compiles negative single-q lookahead with operand word predicates and line gates.
    ///
    /// At least one predicate is required. The checked positive proof tests the
    /// complete original-context conjunction, including shorter-member retries.
    /// Successful negation leaves every operand capture undefined (22.2.2.4).
    /// Full original v/vi validation and normalized capture heads are required.
    pub fn compile_single_q_lookahead_with_word_boundaries_validated_with_work<E>(
        source: &JsString,
        ignore_case: bool,
        multiline: bool,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, RegExpUnicodeSetCompileError<E>> {
        Self::compile_with_work_impl::<true, 0, E>(source, ignore_case, multiline, &mut charge)
    }

    /// Compiles negative lookahead over at least two adjacent q operands.
    ///
    /// An empty or longer member is required overall. Sign-only projection
    /// delegates checked union/scope validation without rebinding; optional ^/$
    /// gates retain original context and all operand slots stay undefined on
    /// success. Full original v/vi validation and normalized heads are required.
    pub fn compile_q_union_lookahead_validated_with_work<E>(
        source: &JsString,
        ignore_case: bool,
        multiline: bool,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, RegExpUnicodeSetCompileError<E>> {
        Self::compile_with_work_impl::<false, 1, E>(source, ignore_case, multiline, &mut charge)
    }

    /// Compiles negative q-union lookahead with word predicates and optional ^/$.
    ///
    /// At least one predicate and two q operands are required, with an empty or
    /// longer member overall. Negation tests the complete original-context
    /// conjunction, leaving all operand captures undefined on success. Full
    /// original v/vi validation and normalized capture heads are required.
    pub fn compile_q_union_lookahead_with_word_boundaries_validated_with_work<E>(
        source: &JsString,
        ignore_case: bool,
        multiline: bool,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, RegExpUnicodeSetCompileError<E>> {
        Self::compile_with_work_impl::<true, 1, E>(source, ignore_case, multiline, &mut charge)
    }

    #[inline(never)]
    fn compile_with_work_impl<const WORD: bool, const CLASS_KIND: u8, E>(
        source: &JsString,
        ignore_case: bool,
        multiline: bool,
        charge: &mut dyn FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, RegExpUnicodeSetCompileError<E>> {
        charge(source.len().saturating_mul(16).saturating_add(16))
            .map_err(RegExpUnicodeSetCompileError::Work)?;
        let units = source.code_units();
        let (assertion, outside_captures) = if let Some(scope) = enclosing_group_body(units, true) {
            (scope.body, scope.captures)
        } else {
            (0..units.len(), 0)
        };
        if !units[assertion.clone()].starts_with(&[0x28, 0x3f, 0x21]) {
            return Ok(None);
        }
        let mut projected = Vec::new();
        projected
            .try_reserve_exact(units.len())
            .map_err(|_| RegExpUnicodeSetCompileError::Capacity)?;
        projected.extend_from_slice(units);
        projected[assertion.start + 2] = 0x3d;
        let source = JsString::from_code_units(projected);
        let positive = if CLASS_KIND == 1 {
            if WORD {
                RegExpUnicodePositiveStringClassMatcher::compile_q_union_lookahead_with_word_boundaries_validated_with_work(
                    &source, ignore_case, multiline, charge,
                )?
            } else {
                RegExpUnicodePositiveStringClassMatcher::compile_q_union_lookahead_validated_with_work(
                    &source, ignore_case, multiline, charge,
                )?
            }
        } else if WORD {
            RegExpUnicodePositiveStringClassMatcher::compile_single_q_lookahead_with_word_boundaries_validated_with_work(
                &source, ignore_case, multiline, charge,
            )?
        } else {
            RegExpUnicodePositiveStringClassMatcher::compile_single_q_lookahead_validated_with_work(
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

    /// Returns the complete lexical count, including undefined operand slots.
    pub fn capture_count(&self) -> usize {
        self.positive.capture_count()
    }

    /// Returns outside empty captures; every operand capture remains undefined.
    pub fn capture_range(&self, index: usize, matched: &Range<usize>) -> Option<Range<usize>> {
        (index < self.outside_captures).then(|| matched.clone())
    }

    /// Searches legal Unicode candidates with cancellable sticky operand probes.
    ///
    /// Each candidate tests the original-context conjunction independently;
    /// negation never searches the remaining suffix from within an operand.
    /// An accepted empty member makes negation fail at that candidate.
    pub fn find_with_work<E>(
        &self,
        input: &JsString,
        start: usize,
        sticky: bool,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Range<usize>>, E> {
        self.find_with_work_impl(input, start, sticky, &mut charge)
    }

    #[inline(never)]
    fn find_with_work_impl<E>(
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
}

#[cfg(test)]
#[path = "regexp_unicode_negative_string_class_tests.rs"]
mod tests;
