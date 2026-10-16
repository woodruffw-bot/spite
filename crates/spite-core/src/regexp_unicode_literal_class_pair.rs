//! One literal followed by two checked UnicodeSets classes (22.2.2.3).

use crate::{
    JsString, RegExpUnicodeCharacterMatcher, RegExpUnicodeClassMatcher,
    RegExpUnicodeSetCompileError, regexp_literal::unicode_literal_atom,
    regexp_unicode_class_literal::class_end,
};
use std::ops::Range;

/// One literal Unicode character and exactly two unquantified checked v/vi classes.
///
/// Requires complete validated v/vi Pattern input. Each class independently
/// requires the existing nested-class/set-operator proof. Optional outside ^/$
/// inspect original input line context. Complete whole-body captures remain a
/// separate caller proof. Flat classes, additional atoms, inner assertions or
/// captures, quantifiers, word gates and string-set leaves remain excluded.
/// Source slices are borrowed and matching retains original UTF-16 ranges.
#[derive(Clone, Debug)]
pub struct RegExpUnicodeLiteralClassPairMatcher {
    literal: RegExpUnicodeCharacterMatcher,
    classes: [RegExpUnicodeClassMatcher; 2],
}

impl RegExpUnicodeLiteralClassPairMatcher {
    /// Proves the complete literal-then-checked-then-checked sequence.
    ///
    /// Construction charges three source walks before decoding the literal or
    /// walking either class boundary. The existing literal and class compilers
    /// then charge their own complete proofs in source order. Both classes are
    /// compiled fully even when the first set is empty. This adapter creates no
    /// source buffer or new shared allocation and leaves earlier APIs unchanged.
    pub fn compile_v_set_with_work<E>(
        source: &JsString,
        ignore_case: bool,
        multiline: bool,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, RegExpUnicodeSetCompileError<E>> {
        let units = source.code_units();
        charge(units.len().saturating_mul(3)).map_err(RegExpUnicodeSetCompileError::Work)?;
        let mut literal_end = usize::from(units.first() == Some(&0x5e));
        let Some(&first) = units.get(literal_end) else {
            return Ok(None);
        };
        literal_end += 1;
        if unicode_literal_atom(units, &mut literal_end, first).is_none()
            || units.get(literal_end) != Some(&0x5b)
        {
            return Ok(None);
        }
        let Some(split) = class_end(units, literal_end) else {
            return Ok(None);
        };
        if units.get(split) != Some(&0x5b) {
            return Ok(None);
        }
        let Some(end) = class_end(units, split) else {
            return Ok(None);
        };
        if end.saturating_add(usize::from(units.get(end) == Some(&0x24))) != units.len() {
            return Ok(None);
        }
        let Some(literal) =
            RegExpUnicodeCharacterMatcher::compile_literal_units_with_flags_and_work(
                &units[..literal_end],
                ignore_case,
                multiline,
                &mut charge,
            )
            .map_err(RegExpUnicodeSetCompileError::Work)?
        else {
            return Ok(None);
        };
        let Some(first) = RegExpUnicodeClassMatcher::compile_v_set_units_with_work(
            &units[literal_end..split],
            ignore_case,
            multiline,
            &mut charge,
        )?
        else {
            return Ok(None);
        };
        let Some(second) = RegExpUnicodeClassMatcher::compile_v_set_units_with_work(
            &units[split..],
            ignore_case,
            multiline,
            &mut charge,
        )?
        else {
            return Ok(None);
        };
        Ok(Some(Self {
            literal,
            classes: [first, second],
        }))
    }

    /// Bounds literal traversal, both existing set predicates and all three terms.
    pub fn search_passes(&self) -> usize {
        self.literal
            .search_passes()
            .saturating_add(self.classes[0].search_passes())
            .saturating_add(self.classes[1].search_passes())
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
            let literal = self.literal.find(input, cursor, sticky)?;
            if let Some(first) = self.classes[0].find(input, literal.end, true) {
                if let Some(second) = self.classes[1].find(input, first.end, true) {
                    return Some(literal.start..second.end);
                }
            }
            if sticky {
                return None;
            }
            // The literal consumes exactly one original character. Its end is
            // the next candidate, preserving overlap after either class fails.
            cursor = literal.end;
        }
    }
}

#[cfg(test)]
#[path = "regexp_unicode_literal_class_pair_tests.rs"]
mod tests;
