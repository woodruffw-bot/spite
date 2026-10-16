//! Two checked UnicodeSets character classes in sequence (22.2.2.3).

use crate::{
    JsString, RegExpUnicodeClassMatcher, RegExpUnicodeSetCompileError,
    regexp_unicode_class_literal::class_end,
};
use std::ops::Range;

/// Exactly two unquantified checked character-only v/vi classes.
///
/// Requires complete validated v/vi Pattern input. Each class must independently
/// meet the existing nested-class/set-operator compiler gate. Optional outside
/// ^/$ use original input line context. No inner captures, assertions, extra
/// atoms, repetitions or string-set leaves are admitted. Callers may prove
/// complete enclosing captures separately. Both classes consume complete original
/// characters without copying source slices or allocating input-sized state.
#[derive(Clone, Debug)]
pub struct RegExpUnicodeClassPairMatcher {
    classes: [RegExpUnicodeClassMatcher; 2],
}

impl RegExpUnicodeClassPairMatcher {
    /// Compiles both complete checked classes with existing work/storage checks.
    ///
    /// Source units are borrowed. Empty first sets never bypass full second-set
    /// validation. The adapter adds no buffer/shared allocation and leaves the
    /// domains and callbacks of every previous compiler unchanged.
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
        // Bound both borrowed class-end walks before either begins.
        charge(units.len().saturating_mul(2)).map_err(RegExpUnicodeSetCompileError::Work)?;
        let Some(split) = class_end(units, begin) else {
            return Ok(None);
        };
        if units.get(split) != Some(&0x5b) {
            return Ok(None);
        }
        let Some(end) = class_end(units, split) else {
            return Ok(None);
        };
        let end_anchor = units.get(end) == Some(&0x24);
        if end.saturating_add(usize::from(end_anchor)) != units.len() {
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
            &units[split..],
            ignore_case,
            multiline,
            &mut charge,
        )?
        else {
            return Ok(None);
        };
        Ok(Some(Self {
            classes: [first, second],
        }))
    }

    /// Bounds pair traversal and both existing membership predicates.
    pub fn search_passes(&self) -> usize {
        self.classes[0]
            .search_passes()
            .saturating_add(self.classes[1].search_passes())
            .saturating_add(2)
    }

    /// Searches a precharged suffix with one additional fallible entry charge.
    ///
    /// Callers precharge search_passes() over the suffix. Sticky calls may bound
    /// that suffix to four UTF-16 units, including any initial pair rewind.
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

    /// Finds the earliest complete adjacent pair, or checks one sticky candidate.
    pub fn find(&self, input: &JsString, start: usize, sticky: bool) -> Option<Range<usize>> {
        let mut cursor = start;
        loop {
            let first = self.classes[0].find(input, cursor, sticky)?;
            if let Some(second) = self.classes[1].find(input, first.end, true) {
                return Some(first.start..second.end);
            }
            if sticky {
                return None;
            }
            // The first class always consumes one complete input character.
            // Retrying its end preserves overlap after a second-class failure.
            cursor = first.end;
        }
    }
}

#[cfg(test)]
#[path = "regexp_unicode_class_pair_tests.rs"]
mod tests;
