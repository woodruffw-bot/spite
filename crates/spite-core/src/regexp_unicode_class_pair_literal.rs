//! Two checked UnicodeSets classes followed by one literal (22.2.2.3).

use crate::{
    JsString, RegExpUnicodeCharacterMatcher, RegExpUnicodeClassMatcher,
    RegExpUnicodeSetCompileError, regexp_literal::unicode_literal_atom,
    regexp_unicode_class_literal::class_end,
};
use std::ops::Range;

/// Exactly two unquantified checked v/vi classes and one literal Unicode character.
///
/// Requires complete validated v/vi Pattern input. Each class independently
/// requires the existing nested-class/set-operator proof. Optional outside ^/$
/// inspect original input line context. Complete whole-body captures remain a
/// separate caller proof. Flat classes, additional atoms, inner assertions or
/// captures, quantifiers, word gates and string-set leaves remain excluded.
/// Source slices are borrowed and matching retains original UTF-16 ranges.
#[derive(Clone, Debug)]
pub struct RegExpUnicodeClassPairLiteralMatcher {
    classes: [RegExpUnicodeClassMatcher; 2],
    literal: RegExpUnicodeCharacterMatcher,
}

impl RegExpUnicodeClassPairLiteralMatcher {
    /// Proves the complete checked-then-checked-then-literal sequence.
    ///
    /// Construction charges three source walks before walking either class
    /// boundary or decoding the literal. Both complete class proofs and then
    /// the literal proof run in source order, even when a class is empty. The
    /// adapter creates no source buffer or new shared allocation. Earlier APIs
    /// retain their domains and exact callback bodies.
    pub fn compile_v_set_with_work<E>(
        source: &JsString,
        ignore_case: bool,
        multiline: bool,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, RegExpUnicodeSetCompileError<E>> {
        let units = source.code_units();
        let begin = usize::from(units.first() == Some(&0x5e));
        if units.get(begin) != Some(&0x5b) {
            return Ok(None);
        }
        charge(units.len().saturating_mul(3)).map_err(RegExpUnicodeSetCompileError::Work)?;
        let Some(split) = class_end(units, begin) else {
            return Ok(None);
        };
        if units.get(split) != Some(&0x5b) {
            return Ok(None);
        }
        let Some(literal_begin) = class_end(units, split) else {
            return Ok(None);
        };
        let Some(&first) = units.get(literal_begin) else {
            return Ok(None);
        };
        // The suffix owns only an optional trailing $, never an inner ^.
        let mut end = literal_begin + 1;
        if unicode_literal_atom(units, &mut end, first).is_none()
            || end.saturating_add(usize::from(units.get(end) == Some(&0x24))) != units.len()
        {
            return Ok(None);
        }
        let Some(first) = RegExpUnicodeClassMatcher::compile_v_set_units_with_work(
            &units[..split],
            ignore_case,
            multiline,
            &mut charge,
        )?
        else {
            return Ok(None);
        };
        let Some(second) = RegExpUnicodeClassMatcher::compile_v_set_units_with_work(
            &units[split..literal_begin],
            ignore_case,
            multiline,
            &mut charge,
        )?
        else {
            return Ok(None);
        };
        let Some(literal) =
            RegExpUnicodeCharacterMatcher::compile_literal_units_with_flags_and_work(
                &units[literal_begin..],
                ignore_case,
                multiline,
                &mut charge,
            )
            .map_err(RegExpUnicodeSetCompileError::Work)?
        else {
            return Ok(None);
        };
        Ok(Some(Self {
            classes: [first, second],
            literal,
        }))
    }

    /// Bounds both set predicates, literal traversal and all three terms.
    pub fn search_passes(&self) -> usize {
        self.classes[0]
            .search_passes()
            .saturating_add(self.classes[1].search_passes())
            .saturating_add(self.literal.search_passes())
            .saturating_add(3)
    }

    /// Searches a precharged suffix with one additional fallible entry charge.
    ///
    /// Sticky calls can bound the suffix to six UTF-16 units, including any
    /// initial pair rewind. Existing two-term plans retain their four-unit bound.
    pub fn find_with_work<E>(
        &self,
        input: &JsString,
        start: usize,
        sticky: bool,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Range<usize>>, E> {
        charge(1)?;
        Ok(self.find(input, start, sticky))
    }

    /// Finds the earliest complete sequence, or checks one sticky candidate.
    pub fn find(&self, input: &JsString, start: usize, sticky: bool) -> Option<Range<usize>> {
        let mut cursor = start;
        loop {
            let first = self.classes[0].find(input, cursor, sticky)?;
            if let Some(second) = self.classes[1].find(input, first.end, true) {
                if let Some(literal) = self.literal.find(input, second.end, true) {
                    return Some(first.start..literal.end);
                }
            }
            if sticky {
                return None;
            }
            // The first class consumes exactly one original character. Retry
            // its end after either suffix fails, retaining overlapping starts.
            cursor = first.end;
        }
    }
}

#[cfg(test)]
#[path = "regexp_unicode_class_pair_literal_tests.rs"]
mod tests;
