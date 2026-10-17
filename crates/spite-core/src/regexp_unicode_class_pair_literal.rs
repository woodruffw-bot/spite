//! Two UnicodeSets classes followed by one literal (22.2.2.3).

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

/// Canonical outside assertions around two classes and one literal.
///
/// Requires complete validated v/vi Pattern input. The checked entry requires
/// two checked classes and at least one outside \b/\B; the separate flat entry
/// requires two flat classes and permits absent word gates. Optional ^/$ inspect
/// original input and multiline context. Whole-body captures remain a separate
/// caller proof. Earlier typed domains and callbacks remain unchanged.
#[derive(Clone, Debug)]
pub struct RegExpUnicodeClassPairLiteralWordMatcher {
    matcher: RegExpUnicodeClassPairLiteralMatcher,
    assertions: ClassPairLiteralWordAssertions,
}

#[derive(Clone, Debug)]
struct ClassPairLiteralWordAssertions {
    start_word: Option<bool>,
    end_word: Option<bool>,
    start_anchor: bool,
    end_anchor: bool,
    ignore_case: bool,
    multiline: bool,
}

impl ClassPairLiteralWordAssertions {
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

impl RegExpUnicodeClassPairLiteralWordMatcher {
    /// Proves canonical framing, both complete checked classes and one literal.
    ///
    /// Four source walks cover borrowed framing, two class boundaries and literal
    /// decoding before any scans. All component compilers then charge their
    /// proofs in source order, even behind either empty class. No source or
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
        if body.first() != Some(&0x5b) {
            return Ok(None);
        }
        let Some(split) = class_end(body, 0) else {
            return Ok(None);
        };
        if body.get(split) != Some(&0x5b) {
            return Ok(None);
        }
        let Some(literal_begin) = class_end(body, split) else {
            return Ok(None);
        };
        let Some(&first) = body.get(literal_begin) else {
            return Ok(None);
        };
        // The bare suffix contains exactly one literal, never an inner ^/$.
        let mut literal_end = literal_begin + 1;
        if unicode_literal_atom(body, &mut literal_end, first).is_none()
            || literal_end != body.len()
        {
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
            &body[split..literal_begin],
            ignore_case,
            multiline,
            &mut charge,
        )?
        else {
            return Ok(None);
        };
        let Some(literal) =
            RegExpUnicodeCharacterMatcher::compile_literal_units_with_flags_and_work(
                &body[literal_begin..],
                ignore_case,
                multiline,
                &mut charge,
            )
            .map_err(RegExpUnicodeSetCompileError::Work)?
        else {
            return Ok(None);
        };
        Ok(Some(Self {
            matcher: RegExpUnicodeClassPairLiteralMatcher {
                classes: [first, second],
                literal,
            },
            assertions: ClassPairLiteralWordAssertions {
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

    /// Retries the first class's complete-character end after later or gate failure.
    pub fn find(&self, input: &JsString, start: usize, sticky: bool) -> Option<Range<usize>> {
        let mut cursor = start;
        loop {
            let first = self.matcher.classes[0].find(input, cursor, sticky)?;
            if let Some(second) = self.matcher.classes[1].find(input, first.end, true) {
                if let Some(literal) = self.matcher.literal.find(input, second.end, true) {
                    if self.assertions.matches(input, first.start, literal.end) {
                        return Some(first.start..literal.end);
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

impl RegExpUnicodeClassPairLiteralWordMatcher {
    /// Proves two independently flat v/vi classes followed by one literal.
    ///
    /// Requires complete validated v/vi Pattern input. Optional canonical
    /// outside word gates and line anchors retain original
    /// i/m context, including the bare form with no word gates. Four source walks
    /// precede borrowed framing and literal decoding. Complete flat-class,
    /// flat-class and literal proofs then run in source order, including behind
    /// empty classes. Checked/mixed/string classes and other shapes stay closed.
    /// Earlier APIs, callbacks and search bodies remain unchanged.
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
        if body.first() != Some(&0x5b) {
            return Ok(None);
        }
        let Some(split) = class_end(body, 0) else {
            return Ok(None);
        };
        if body.get(split) != Some(&0x5b) {
            return Ok(None);
        }
        let Some(literal_begin) = class_end(body, split) else {
            return Ok(None);
        };
        let Some(&first) = body.get(literal_begin) else {
            return Ok(None);
        };
        // The bare suffix contains exactly one literal, never an inner ^/$.
        let mut literal_end = literal_begin + 1;
        if unicode_literal_atom(body, &mut literal_end, first).is_none()
            || literal_end != body.len()
        {
            return Ok(None);
        }
        let Some(first) = RegExpUnicodeClassMatcher::compile_flat_v_set_units_with_work(
            &body[..split],
            ignore_case,
            multiline,
            &mut charge,
        )
        .map_err(RegExpUnicodeSetCompileError::Work)?
        else {
            return Ok(None);
        };
        let Some(second) = RegExpUnicodeClassMatcher::compile_flat_v_set_units_with_work(
            &body[split..literal_begin],
            ignore_case,
            multiline,
            &mut charge,
        )
        .map_err(RegExpUnicodeSetCompileError::Work)?
        else {
            return Ok(None);
        };
        let Some(literal) =
            RegExpUnicodeCharacterMatcher::compile_literal_units_with_flags_and_work(
                &body[literal_begin..],
                ignore_case,
                multiline,
                &mut charge,
            )
            .map_err(RegExpUnicodeSetCompileError::Work)?
        else {
            return Ok(None);
        };
        Ok(Some(Self {
            matcher: RegExpUnicodeClassPairLiteralMatcher {
                classes: [first, second],
                literal,
            },
            assertions: ClassPairLiteralWordAssertions {
                start_word,
                end_word,
                start_anchor,
                end_anchor,
                ignore_case,
                multiline,
            },
        }))
    }
}

impl RegExpUnicodeClassPairLiteralWordMatcher {
    /// Proves one checked class, one flat class and one literal in v/vi mode.
    ///
    /// Requires complete validated v/vi Pattern input. Optional canonical
    /// outside word gates and line anchors retain original
    /// i/m context, including the bare form with no word gates. Four source walks
    /// precede borrowed framing and literal decoding. Complete checked-class,
    /// flat-class and literal proofs then run in source order, including behind
    /// empty classes. Other proof orders and string classes stay closed.
    /// Earlier APIs, callbacks and search bodies remain unchanged.
    pub fn compile_checked_then_flat_v_set_with_work<E>(
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
        if body.first() != Some(&0x5b) {
            return Ok(None);
        }
        let Some(split) = class_end(body, 0) else {
            return Ok(None);
        };
        if body.get(split) != Some(&0x5b) {
            return Ok(None);
        }
        let Some(literal_begin) = class_end(body, split) else {
            return Ok(None);
        };
        let Some(&first) = body.get(literal_begin) else {
            return Ok(None);
        };
        // The bare suffix contains exactly one literal, never an inner ^/$.
        let mut literal_end = literal_begin + 1;
        if unicode_literal_atom(body, &mut literal_end, first).is_none()
            || literal_end != body.len()
        {
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
            &body[split..literal_begin],
            ignore_case,
            multiline,
            &mut charge,
        )
        .map_err(RegExpUnicodeSetCompileError::Work)?
        else {
            return Ok(None);
        };
        let Some(literal) =
            RegExpUnicodeCharacterMatcher::compile_literal_units_with_flags_and_work(
                &body[literal_begin..],
                ignore_case,
                multiline,
                &mut charge,
            )
            .map_err(RegExpUnicodeSetCompileError::Work)?
        else {
            return Ok(None);
        };
        Ok(Some(Self {
            matcher: RegExpUnicodeClassPairLiteralMatcher {
                classes: [first, second],
                literal,
            },
            assertions: ClassPairLiteralWordAssertions {
                start_word,
                end_word,
                start_anchor,
                end_anchor,
                ignore_case,
                multiline,
            },
        }))
    }
}

impl RegExpUnicodeClassPairLiteralWordMatcher {
    /// Proves one flat class, one checked class and one literal in v/vi mode.
    ///
    /// Requires complete validated v/vi Pattern input. Optional canonical
    /// outside word gates and line anchors retain original
    /// i/m context, including the bare form with no word gates. Four source walks
    /// precede borrowed framing and literal decoding. Complete flat-class,
    /// checked-class and literal proofs then run in source order, including behind
    /// empty classes. Other proof orders and string classes stay closed.
    /// Earlier APIs, callbacks and search bodies remain unchanged.
    pub fn compile_flat_then_checked_v_set_with_work<E>(
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
        if body.first() != Some(&0x5b) {
            return Ok(None);
        }
        let Some(split) = class_end(body, 0) else {
            return Ok(None);
        };
        if body.get(split) != Some(&0x5b) {
            return Ok(None);
        }
        let Some(literal_begin) = class_end(body, split) else {
            return Ok(None);
        };
        let Some(&first) = body.get(literal_begin) else {
            return Ok(None);
        };
        // The bare suffix contains exactly one literal, never an inner ^/$.
        let mut literal_end = literal_begin + 1;
        if unicode_literal_atom(body, &mut literal_end, first).is_none()
            || literal_end != body.len()
        {
            return Ok(None);
        }
        let Some(first) = RegExpUnicodeClassMatcher::compile_flat_v_set_units_with_work(
            &body[..split],
            ignore_case,
            multiline,
            &mut charge,
        )
        .map_err(RegExpUnicodeSetCompileError::Work)?
        else {
            return Ok(None);
        };
        let Some(second) = RegExpUnicodeClassMatcher::compile_v_set_units_with_work(
            &body[split..literal_begin],
            ignore_case,
            multiline,
            &mut charge,
        )?
        else {
            return Ok(None);
        };
        let Some(literal) =
            RegExpUnicodeCharacterMatcher::compile_literal_units_with_flags_and_work(
                &body[literal_begin..],
                ignore_case,
                multiline,
                &mut charge,
            )
            .map_err(RegExpUnicodeSetCompileError::Work)?
        else {
            return Ok(None);
        };
        Ok(Some(Self {
            matcher: RegExpUnicodeClassPairLiteralMatcher {
                classes: [first, second],
                literal,
            },
            assertions: ClassPairLiteralWordAssertions {
                start_word,
                end_word,
                start_anchor,
                end_anchor,
                ignore_case,
                multiline,
            },
        }))
    }
}

#[cfg(test)]
#[path = "regexp_unicode_class_pair_literal_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "regexp_unicode_class_pair_literal_word_tests.rs"]
mod word_tests;

#[cfg(test)]
#[path = "regexp_unicode_flat_class_pair_literal_tests.rs"]
mod flat_class_tests;

#[cfg(test)]
#[path = "regexp_unicode_checked_flat_class_pair_literal_tests.rs"]
mod checked_flat_class_tests;

#[cfg(test)]
#[path = "regexp_unicode_flat_checked_class_pair_literal_tests.rs"]
mod flat_checked_class_tests;
