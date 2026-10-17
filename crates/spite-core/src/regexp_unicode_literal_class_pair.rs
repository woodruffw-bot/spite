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

/// Canonical outside word assertions around one literal and two checked classes.
///
/// Requires complete validated v/vi Pattern input and at least one outside
/// \b/\B. Optional ^/$ inspect original input and multiline context. Complete
/// whole-body captures remain a separate caller proof. The existing plain
/// matcher and all earlier typed domains/callbacks remain unchanged.
#[derive(Clone, Debug)]
pub struct RegExpUnicodeLiteralClassPairWordMatcher {
    matcher: RegExpUnicodeLiteralClassPairMatcher,
    assertions: LiteralClassPairWordAssertions,
}

#[derive(Clone, Debug)]
struct LiteralClassPairWordAssertions {
    start_word: Option<bool>,
    end_word: Option<bool>,
    start_anchor: bool,
    end_anchor: bool,
    ignore_case: bool,
    multiline: bool,
}

impl LiteralClassPairWordAssertions {
    fn matches(&self, input: &JsString, start: usize, end: usize) -> bool {
        use crate::regexp_unicode_character::{unicode_assertions_match, unicode_word_boundary};
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

impl RegExpUnicodeLiteralClassPairWordMatcher {
    /// Proves canonical framing, one literal and both complete checked classes.
    ///
    /// Four source walks cover borrowed framing, literal decoding and two class
    /// boundaries before any scans. All component compilers then charge their
    /// proofs in source order, even behind an empty first class. No source or
    /// input buffer is copied and no shared allocation is introduced here.
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
        if start_word.is_none() && end_word.is_none() {
            return Ok(None);
        }
        let Some(body) = units.get(begin..end) else {
            return Ok(None);
        };
        let Some(&first) = body.first() else {
            return Ok(None);
        };
        let mut literal_end = 1;
        if unicode_literal_atom(body, &mut literal_end, first).is_none()
            || body.get(literal_end) != Some(&0x5b)
        {
            return Ok(None);
        }
        let Some(split) = class_end(body, literal_end) else {
            return Ok(None);
        };
        if body.get(split) != Some(&0x5b) || class_end(body, split) != Some(body.len()) {
            return Ok(None);
        }
        let Some(literal) =
            RegExpUnicodeCharacterMatcher::compile_literal_units_with_flags_and_work(
                &body[..literal_end],
                ignore_case,
                multiline,
                &mut charge,
            )
            .map_err(RegExpUnicodeSetCompileError::Work)?
        else {
            return Ok(None);
        };
        let Some(first) = RegExpUnicodeClassMatcher::compile_v_set_units_with_work(
            &body[literal_end..split],
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
            matcher: RegExpUnicodeLiteralClassPairMatcher {
                literal,
                classes: [first, second],
            },
            assertions: LiteralClassPairWordAssertions {
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

    /// Retries the literal's complete-character end after class or gate failure.
    pub fn find(&self, input: &JsString, start: usize, sticky: bool) -> Option<Range<usize>> {
        let mut cursor = start;
        loop {
            let literal = self.matcher.literal.find(input, cursor, sticky)?;
            if let Some(first) = self.matcher.classes[0].find(input, literal.end, true) {
                if let Some(second) = self.matcher.classes[1].find(input, first.end, true) {
                    if self.assertions.matches(input, literal.start, second.end) {
                        return Some(literal.start..second.end);
                    }
                }
            }
            if sticky {
                return None;
            }
            cursor = literal.end;
        }
    }
}

#[cfg(test)]
#[path = "regexp_unicode_literal_class_pair_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "regexp_unicode_literal_class_pair_word_tests.rs"]
mod word_tests;
