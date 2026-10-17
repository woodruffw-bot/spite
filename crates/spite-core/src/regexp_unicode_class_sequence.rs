//! Finite runs of independently checked UnicodeSets classes (22.2.2.3).

use crate::{
    JsString, RegExpUnicodeClassMatcher, RegExpUnicodeSetCompileError,
    regexp_unicode_character::{unicode_assertions_match, unicode_word_boundary},
    regexp_unicode_class_literal::class_end,
};
use std::{ops::Range, sync::Arc};

/// Finite unquantified character-only v/vi class runs with distinct proofs.
///
/// Requires complete validated v/vi Pattern input. The checked entry requires
/// at least four independently nested/operator classes; the separate flat entry
/// requires at least three independently flat classes. Optional canonical
/// outside ^/$ and \b/\B inspect original input with actual i/m flags. Complete
/// whole-body captures remain a separate caller proof. Mixed proof kinds,
/// literals, inner assertions/captures, quantifiers and string leaves stay
/// excluded. Existing shorter typed APIs and callback bodies are unchanged.
#[derive(Clone, Debug)]
pub struct RegExpUnicodeClassSequenceMatcher(Arc<Program>);

#[derive(Debug)]
struct Program {
    classes: Vec<RegExpUnicodeClassMatcher>,
    assertions: OutsideAssertions,
    search_passes: usize,
}

#[derive(Debug)]
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

impl RegExpUnicodeClassSequenceMatcher {
    /// Frames the complete source before reserving or proving any class.
    ///
    /// Four source walks cover two outside slash-parity walks and two disjoint
    /// class-boundary passes. Existing class compilers charge their additional
    /// proofs in source order, including every class behind an empty set. New
    /// variable-capacity storage reserves fallibly; final shared storage retains
    /// the existing infallible Arc allocation convention. No source is copied.
    pub fn compile_v_set_with_work<E>(
        source: &JsString,
        ignore_case: bool,
        multiline: bool,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, RegExpUnicodeSetCompileError<E>> {
        let units = source.code_units();
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
        let Some(body) = units.get(begin..end) else {
            return Ok(None);
        };

        let mut at = 0;
        let mut count = 0usize;
        while at < body.len() {
            if body.get(at) != Some(&0x5b) {
                return Ok(None);
            }
            let Some(next) = class_end(body, at) else {
                return Ok(None);
            };
            count = count
                .checked_add(1)
                .ok_or(RegExpUnicodeSetCompileError::Capacity)?;
            at = next;
        }
        if count < 4 {
            return Ok(None);
        }

        let assertions = OutsideAssertions {
            start_word,
            end_word,
            start_anchor,
            end_anchor,
            ignore_case,
            multiline,
        };
        let mut classes = Vec::new();
        classes
            .try_reserve_exact(count)
            .map_err(|_| RegExpUnicodeSetCompileError::Capacity)?;
        let mut search_passes = assertions.search_passes();
        at = 0;
        while at < body.len() {
            // The immutable first pass proved the complete same partition.
            let Some(next) = class_end(body, at) else {
                return Ok(None);
            };
            let Some(class) = RegExpUnicodeClassMatcher::compile_v_set_units_with_work(
                &body[at..next],
                ignore_case,
                multiline,
                &mut charge,
            )?
            else {
                return Ok(None);
            };
            search_passes = search_passes
                .saturating_add(class.search_passes())
                .saturating_add(1);
            // Exactly count successful proofs fit the pre-reserved capacity.
            classes.push(class);
            at = next;
        }
        debug_assert_eq!(classes.len(), count);
        Ok(Some(Self(Arc::new(Program {
            classes,
            assertions,
            search_passes,
        }))))
    }

    /// Bounds every set predicate, consuming term and outside assertion.
    pub fn search_passes(&self) -> usize {
        self.0.search_passes
    }

    /// Maximum original UTF-16 width, derived from the proved term count.
    ///
    /// Callers also allow the existing initial low-surrogate pair rewind when
    /// bounding their remaining original input. No source-length proxy is used.
    pub fn max_match_code_units(&self) -> usize {
        self.0.classes.len().saturating_mul(2)
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

    /// Retries the first complete-character end after any later or gate failure.
    pub fn find(&self, input: &JsString, start: usize, sticky: bool) -> Option<Range<usize>> {
        let mut cursor = start;
        loop {
            let first = self.0.classes[0].find(input, cursor, sticky)?;
            let mut end = first.end;
            let mut complete = true;
            for class in &self.0.classes[1..] {
                let Some(found) = class.find(input, end, true) else {
                    complete = false;
                    break;
                };
                end = found.end;
            }
            if complete && self.0.assertions.matches(input, first.start, end) {
                return Some(first.start..end);
            }
            if sticky {
                return None;
            }
            cursor = first.end;
        }
    }
}

impl RegExpUnicodeClassSequenceMatcher {
    /// Proves at least three complete flat character-only v/vi classes.
    ///
    /// This separate entry retains the checked entry's exact domain/callbacks.
    /// Four source walks frame/count the entire borrowed body before fallible
    /// exact reservation and ordered component proofs. Supported empty classes
    /// still require every later proof. Only the existing flat proof is used;
    /// nested/operator/mixed classes, extra atoms and string leaves stay excluded.
    pub fn compile_flat_v_set_with_work<E>(
        source: &JsString,
        ignore_case: bool,
        multiline: bool,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, RegExpUnicodeSetCompileError<E>> {
        let units = source.code_units();
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
        let Some(body) = units.get(begin..end) else {
            return Ok(None);
        };

        let mut at = 0;
        let mut count = 0usize;
        while at < body.len() {
            if body.get(at) != Some(&0x5b) {
                return Ok(None);
            }
            let Some(next) = class_end(body, at) else {
                return Ok(None);
            };
            count = count
                .checked_add(1)
                .ok_or(RegExpUnicodeSetCompileError::Capacity)?;
            at = next;
        }
        if count < 3 {
            return Ok(None);
        }

        let assertions = OutsideAssertions {
            start_word,
            end_word,
            start_anchor,
            end_anchor,
            ignore_case,
            multiline,
        };
        let mut classes = Vec::new();
        classes
            .try_reserve_exact(count)
            .map_err(|_| RegExpUnicodeSetCompileError::Capacity)?;
        let mut search_passes = assertions.search_passes();
        at = 0;
        while at < body.len() {
            // The immutable first pass proved the complete same partition.
            let Some(next) = class_end(body, at) else {
                return Ok(None);
            };
            let Some(class) = RegExpUnicodeClassMatcher::compile_flat_v_set_units_with_work(
                &body[at..next],
                ignore_case,
                multiline,
                &mut charge,
            )
            .map_err(RegExpUnicodeSetCompileError::Work)?
            else {
                return Ok(None);
            };
            search_passes = search_passes
                .saturating_add(class.search_passes())
                .saturating_add(1);
            // Exactly count successful proofs fit the pre-reserved capacity.
            classes.push(class);
            at = next;
        }
        debug_assert_eq!(classes.len(), count);
        Ok(Some(Self(Arc::new(Program {
            classes,
            assertions,
            search_passes,
        }))))
    }
}

#[cfg(test)]
#[path = "regexp_unicode_class_sequence_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "regexp_unicode_flat_class_sequence_tests.rs"]
mod flat_tests;
