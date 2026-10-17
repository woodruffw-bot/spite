//! Exactly three checked UnicodeSets classes (22.2.2.3).

use crate::{
    JsString, RegExpUnicodeClassMatcher, RegExpUnicodeSetCompileError,
    regexp_unicode_class_literal::class_end,
};
use std::ops::Range;

/// Exactly three unquantified, independently checked character-only v/vi classes.
///
/// Requires complete validated v/vi Pattern input. Each class independently
/// requires the existing nested-class/set-operator proof. Optional outside ^/$
/// inspect original input line context. Complete whole-body captures remain a
/// separate caller proof. Flat classes, extra atoms, inner assertions/captures,
/// quantifiers, word gates and string-set leaves remain excluded. Source slices
/// are borrowed and matching retains original UTF-16 ranges.
#[derive(Clone, Debug)]
pub struct RegExpUnicodeClassTripleMatcher {
    classes: [RegExpUnicodeClassMatcher; 3],
}

impl RegExpUnicodeClassTripleMatcher {
    /// Proves all three complete classes in source order, even after empty sets.
    ///
    /// Three source walks are charged before any class boundary scan. Complete
    /// class compilers then charge their own proofs in first/second/third order.
    /// This adapter creates no source buffer or new shared allocation. Earlier
    /// typed APIs retain their exact domains and callback bodies.
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
        let Some(first_end) = class_end(units, begin) else {
            return Ok(None);
        };
        if units.get(first_end) != Some(&0x5b) {
            return Ok(None);
        }
        let Some(second_end) = class_end(units, first_end) else {
            return Ok(None);
        };
        if units.get(second_end) != Some(&0x5b) {
            return Ok(None);
        }
        let Some(end) = class_end(units, second_end) else {
            return Ok(None);
        };
        if end.saturating_add(usize::from(units.get(end) == Some(&0x24))) != units.len() {
            return Ok(None);
        }
        let Some(first) = RegExpUnicodeClassMatcher::compile_v_set_units_with_work(
            &units[..first_end],
            ignore_case,
            multiline,
            &mut charge,
        )?
        else {
            return Ok(None);
        };
        let Some(second) = RegExpUnicodeClassMatcher::compile_v_set_units_with_work(
            &units[first_end..second_end],
            ignore_case,
            multiline,
            &mut charge,
        )?
        else {
            return Ok(None);
        };
        let Some(third) = RegExpUnicodeClassMatcher::compile_v_set_units_with_work(
            &units[second_end..],
            ignore_case,
            multiline,
            &mut charge,
        )?
        else {
            return Ok(None);
        };
        Ok(Some(Self {
            classes: [first, second, third],
        }))
    }

    /// Bounds the three existing set predicates and all three consuming terms.
    pub fn search_passes(&self) -> usize {
        self.classes[0]
            .search_passes()
            .saturating_add(self.classes[1].search_passes())
            .saturating_add(self.classes[2].search_passes())
            .saturating_add(3)
    }

    /// Searches a precharged suffix with one additional fallible entry charge.
    /// Sticky calls can bound three complete characters to six original UTF-16
    /// units, including any approved initial pair rewind.
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
                if let Some(third) = self.classes[2].find(input, second.end, true) {
                    return Some(first.start..third.end);
                }
            }
            if sticky {
                return None;
            }
            // Retry the first class end after either later class fails, so
            // overlapping viable starts are never skipped.
            cursor = first.end;
        }
    }
}

#[cfg(test)]
#[path = "regexp_unicode_class_triple_tests.rs"]
mod tests;
