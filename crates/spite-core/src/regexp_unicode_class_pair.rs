//! Two character-only UnicodeSets classes in sequence (22.2.2.3).

use crate::{
    JsString, RegExpUnicodeClassMatcher, RegExpUnicodeSetCompileError,
    regexp_unicode_character::{unicode_assertions_match, unicode_word_boundary},
    regexp_unicode_class_literal::class_end,
};
use std::ops::Range;

/// Exactly two unquantified character-only v/vi classes with separate proofs.
///
/// Requires complete validated v/vi Pattern input. Checked-pair entry points
/// require both nested/operator gates; mixed-order entry points prove one flat
/// and one checked component separately. Optional outside
/// ^/$ use original input line context. Separate constructors admit canonical
/// outside word gates for two checked components or a checked-then-flat pair. Inner captures/assertions,
/// extra atoms, repetitions and
/// string-set leaves remain separate proofs. Callers may prove
/// complete enclosing captures separately. Both classes consume complete original
/// characters without copying source slices or allocating input-sized state.
#[derive(Clone, Debug)]
pub struct RegExpUnicodeClassPairMatcher {
    classes: [RegExpUnicodeClassMatcher; 2],
    assertions: Option<OutsideAssertions>,
}

#[derive(Clone, Debug)]
struct OutsideAssertions {
    start_word: Option<bool>,
    end_word: Option<bool>,
    start_anchor: bool,
    end_anchor: bool,
    ignore_case: bool,
    multiline: bool,
}

impl OutsideAssertions {
    fn matches(&self, input: &JsString, start: usize, end: usize) -> bool {
        let units = input.code_units();
        unicode_assertions_match(
            units,
            start,
            end,
            self.start_anchor,
            self.end_anchor,
            self.multiline,
        ) && self.start_word.is_none_or(|expected| {
            unicode_word_boundary(units, start, self.ignore_case) == expected
        }) && self
            .end_word
            .is_none_or(|expected| unicode_word_boundary(units, end, self.ignore_case) == expected)
    }

    fn search_passes(&self) -> usize {
        usize::from(self.start_anchor)
            + usize::from(self.end_anchor)
            + 8 * (usize::from(self.start_word.is_some()) + usize::from(self.end_word.is_some()))
    }
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
            assertions: None,
        }))
    }

    /// Compiles canonical outside word gates around two complete checked classes.
    ///
    /// Requires complete validated v/vi Pattern input and at least one outside
    /// \b/\B. Framing is optional ^, optional word gate, the bare complete pair,
    /// optional word gate, optional $. No inner/reordered assertion, capture,
    /// repetition, extra atom or string leaf is admitted. Both classes compile
    /// fully even when empty. Source slices are borrowed; old compilers and
    /// their callback sequences remain unchanged.
    pub fn compile_v_set_with_word_boundaries_and_work<E>(
        source: &JsString,
        ignore_case: bool,
        multiline: bool,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, RegExpUnicodeSetCompileError<E>> {
        let units = source.code_units();
        // Bound framing and both borrowed class-end walks before those scans.
        charge(units.len().saturating_mul(3)).map_err(RegExpUnicodeSetCompileError::Work)?;
        let start_anchor = units.first() == Some(&0x5e);
        let end_anchor = units.last() == Some(&0x24)
            && units[..units.len() - 1]
                .iter()
                .rev()
                .take_while(|&&unit| unit == 0x5c)
                .count()
                % 2
                == 0;
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
        let end_word = if end >= begin + 2
            && units[end - 2] == 0x5c
            && matches!(units[end - 1], 0x62 | 0x42)
            && units[..end - 1]
                .iter()
                .rev()
                .take_while(|&&unit| unit == 0x5c)
                .count()
                % 2
                == 1
        {
            end -= 2;
            Some(units[end + 1] == 0x62)
        } else {
            None
        };
        if start_word.is_none() && end_word.is_none() {
            return Ok(None);
        }
        let Some(body) = units.get(begin..end) else {
            return Ok(None);
        };
        if body.first() != Some(&0x5b) {
            return Ok(None);
        }
        let Some(split) = class_end(body, 0) else {
            return Ok(None);
        };
        if body.get(split) != Some(&0x5b) || class_end(body, split) != Some(body.len()) {
            return Ok(None);
        }
        let Some(first) = RegExpUnicodeClassMatcher::compile_v_set_units_with_work(
            &body[..split],
            ignore_case,
            multiline,
            &mut charge,
        )?
        else {
            return Ok(None);
        };
        let Some(second) = RegExpUnicodeClassMatcher::compile_v_set_units_with_work(
            &body[split..],
            ignore_case,
            multiline,
            &mut charge,
        )?
        else {
            return Ok(None);
        };
        Ok(Some(Self {
            classes: [first, second],
            assertions: Some(OutsideAssertions {
                start_word,
                end_word,
                start_anchor,
                end_anchor,
                ignore_case,
                multiline,
            }),
        }))
    }

    /// Compiles one checked v/vi class followed by one complete flat class.
    ///
    /// Requires complete validated v/vi Pattern input. The first class retains
    /// the old nested/operator gate; the second uses the separate borrowed flat
    /// proof. Optional outside ^/$ retain original line context. Reverse order,
    /// outside word gates, inner captures/assertions, repetitions, extra atoms
    /// and string leaves remain separate proofs. Empty first sets never bypass
    /// full second compilation. Existing constructors/callbacks are unchanged.
    pub fn compile_v_set_then_flat_with_work<E>(
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
        let Some(second) = RegExpUnicodeClassMatcher::compile_flat_v_set_units_with_work(
            &units[split..],
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
            assertions: None,
        }))
    }

    /// Compiles one complete flat v/vi class followed by one checked class.
    ///
    /// Requires complete validated v/vi Pattern input. The first component uses
    /// the existing borrowed flat proof; the second retains the nested/operator
    /// gate. Optional outside ^/$ retain original line context. Both components
    /// compile fully even when the first is empty. Outside word gates, inner
    /// captures/assertions, repetitions, extra atoms and string leaves remain
    /// separate proofs. Previous constructors and callbacks are unchanged.
    pub fn compile_flat_then_v_set_with_work<E>(
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
        let Some(first) = RegExpUnicodeClassMatcher::compile_flat_v_set_units_with_work(
            &units[..split],
            ignore_case,
            multiline,
            &mut charge,
        )
        .map_err(RegExpUnicodeSetCompileError::Work)?
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
            assertions: None,
        }))
    }

    /// Compiles canonical outside word gates around a checked-then-flat pair.
    ///
    /// Requires complete validated v/vi Pattern input and at least one outside
    /// \b/\B. The checked prefix and flat suffix retain their separate proofs.
    /// Optional outside ^/$ use original context. Both components compile fully,
    /// including empty first sets. Reverse order, flat-flat, inner/reordered
    /// assertions, captures, repetitions, extra atoms and string leaves remain
    /// separate proofs. Source slices are borrowed; old callbacks are unchanged.
    pub fn compile_v_set_then_flat_with_word_boundaries_and_work<E>(
        source: &JsString,
        ignore_case: bool,
        multiline: bool,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, RegExpUnicodeSetCompileError<E>> {
        let units = source.code_units();
        // Bound framing and both borrowed class-end walks before those scans.
        charge(units.len().saturating_mul(3)).map_err(RegExpUnicodeSetCompileError::Work)?;
        let start_anchor = units.first() == Some(&0x5e);
        let end_anchor = units.last() == Some(&0x24)
            && units[..units.len() - 1]
                .iter()
                .rev()
                .take_while(|&&unit| unit == 0x5c)
                .count()
                % 2
                == 0;
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
        let end_word = if end >= begin + 2
            && units[end - 2] == 0x5c
            && matches!(units[end - 1], 0x62 | 0x42)
            && units[..end - 1]
                .iter()
                .rev()
                .take_while(|&&unit| unit == 0x5c)
                .count()
                % 2
                == 1
        {
            end -= 2;
            Some(units[end + 1] == 0x62)
        } else {
            None
        };
        if start_word.is_none() && end_word.is_none() {
            return Ok(None);
        }
        let Some(body) = units.get(begin..end) else {
            return Ok(None);
        };
        if body.first() != Some(&0x5b) {
            return Ok(None);
        }
        let Some(split) = class_end(body, 0) else {
            return Ok(None);
        };
        if body.get(split) != Some(&0x5b) || class_end(body, split) != Some(body.len()) {
            return Ok(None);
        }
        let Some(first) = RegExpUnicodeClassMatcher::compile_v_set_units_with_work(
            &body[..split],
            ignore_case,
            multiline,
            &mut charge,
        )?
        else {
            return Ok(None);
        };
        let Some(second) = RegExpUnicodeClassMatcher::compile_flat_v_set_units_with_work(
            &body[split..],
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
            assertions: Some(OutsideAssertions {
                start_word,
                end_word,
                start_anchor,
                end_anchor,
                ignore_case,
                multiline,
            }),
        }))
    }

    /// Bounds pair traversal and both existing membership predicates.
    pub fn search_passes(&self) -> usize {
        self.classes[0]
            .search_passes()
            .saturating_add(self.classes[1].search_passes())
            .saturating_add(2)
            .saturating_add(
                self.assertions
                    .as_ref()
                    .map_or(0, OutsideAssertions::search_passes),
            )
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
                if self
                    .assertions
                    .as_ref()
                    .is_none_or(|gates| gates.matches(input, first.start, second.end))
                {
                    return Some(first.start..second.end);
                }
            }
            if sticky {
                return None;
            }
            // The first class always consumes one complete input character.
            // Retrying its end preserves overlap after a second-class or gate failure.
            cursor = first.end;
        }
    }
}

#[cfg(test)]
#[path = "regexp_unicode_class_pair_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "regexp_unicode_class_pair_word_tests.rs"]
mod outside_word_tests;

#[cfg(test)]
#[path = "regexp_unicode_class_flat_pair_tests.rs"]
mod checked_flat_tests;

#[cfg(test)]
#[path = "regexp_unicode_flat_class_pair_tests.rs"]
mod flat_checked_tests;

#[cfg(test)]
#[path = "regexp_unicode_class_flat_pair_word_tests.rs"]
mod checked_flat_word_tests;
