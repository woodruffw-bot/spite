//! Exactly three checked UnicodeSets classes (22.2.2.3).

use crate::{
    JsString, RegExpUnicodeClassMatcher, RegExpUnicodeSetCompileError,
    regexp_unicode_character::{unicode_assertions_match, unicode_word_boundary},
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

/// Canonical outside word assertions around exactly three checked v/vi classes.
///
/// Requires complete validated v/vi Pattern input and at least one outside
/// \b/\B. Optional ^/$ inspect original input and multiline context. Complete
/// whole-body captures remain a separate caller proof. The existing plain
/// matcher and all earlier typed domains/callbacks remain unchanged.
#[derive(Clone, Debug)]
pub struct RegExpUnicodeClassTripleWordMatcher {
    matcher: RegExpUnicodeClassTripleMatcher,
    assertions: TripleWordAssertions,
}

#[derive(Clone, Debug)]
struct TripleWordAssertions {
    start_word: Option<bool>,
    end_word: Option<bool>,
    start_anchor: bool,
    end_anchor: bool,
    ignore_case: bool,
    multiline: bool,
}

impl TripleWordAssertions {
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

impl RegExpUnicodeClassTripleWordMatcher {
    /// Proves canonical framing and all three complete classes in source order.
    ///
    /// Four source walks cover borrowed framing and three class boundaries.
    /// Every component retains its independent nested/operator character-only
    /// proof, even behind empty sets. No source slice or input buffer is copied.
    pub fn compile_v_set_with_work<E>(
        source: &JsString,
        ignore_case: bool,
        multiline: bool,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, RegExpUnicodeSetCompileError<E>> {
        let units = source.code_units();
        // Bound framing and all three borrowed class-end walks before those scans.
        charge(units.len().saturating_mul(4)).map_err(RegExpUnicodeSetCompileError::Work)?;
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
        let Some(first_end) = class_end(body, 0) else {
            return Ok(None);
        };
        if body.get(first_end) != Some(&0x5b) {
            return Ok(None);
        }
        let Some(second_end) = class_end(body, first_end) else {
            return Ok(None);
        };
        if body.get(second_end) != Some(&0x5b) || class_end(body, second_end) != Some(body.len()) {
            return Ok(None);
        }
        let Some(first) = RegExpUnicodeClassMatcher::compile_v_set_units_with_work(
            &body[..first_end],
            ignore_case,
            multiline,
            &mut charge,
        )?
        else {
            return Ok(None);
        };
        let Some(second) = RegExpUnicodeClassMatcher::compile_v_set_units_with_work(
            &body[first_end..second_end],
            ignore_case,
            multiline,
            &mut charge,
        )?
        else {
            return Ok(None);
        };
        let Some(third) = RegExpUnicodeClassMatcher::compile_v_set_units_with_work(
            &body[second_end..],
            ignore_case,
            multiline,
            &mut charge,
        )?
        else {
            return Ok(None);
        };
        Ok(Some(Self {
            matcher: RegExpUnicodeClassTripleMatcher {
                classes: [first, second, third],
            },
            assertions: TripleWordAssertions {
                start_word,
                end_word,
                start_anchor,
                end_anchor,
                ignore_case,
                multiline,
            },
        }))
    }

    /// Bounds all three consuming predicates and original outside assertions.
    pub fn search_passes(&self) -> usize {
        self.matcher
            .search_passes()
            .saturating_add(self.assertions.search_passes())
    }

    /// Searches precharged original input after one fallible entry charge.
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

    /// Retries the first complete-character end after a later or gate failure.
    pub fn find(&self, input: &JsString, start: usize, sticky: bool) -> Option<Range<usize>> {
        let mut cursor = start;
        loop {
            let first = self.matcher.classes[0].find(input, cursor, sticky)?;
            if let Some(second) = self.matcher.classes[1].find(input, first.end, true) {
                if let Some(third) = self.matcher.classes[2].find(input, second.end, true) {
                    if self.assertions.matches(input, first.start, third.end) {
                        return Some(first.start..third.end);
                    }
                }
            }
            if sticky {
                return None;
            }
            cursor = first.end;
        }
    }
}

#[cfg(test)]
#[path = "regexp_unicode_class_triple_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "regexp_unicode_class_triple_word_tests.rs"]
mod word_tests;
