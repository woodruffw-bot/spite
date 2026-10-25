//! Positive lookahead over variable-length UnicodeSets q classes (22.2.2.4, 22.2.2.9).

use crate::{
    JsString, RegExpUnicodePositiveAtomMatch, RegExpUnicodeSetCompileError,
    RegExpUnicodeStringSetMatcher, regexp_zero_count_group::enclosing_group_body,
    regexp_zero_count_sequence::group_end, regexp_zero_count_tree::surrogate_atom_end,
};
use std::ops::Range;

/// A complete positive lookahead over one single-q class with variable-length members.
///
/// Full original v/vi validation and normalized capture heads are prerequisites.
/// Complete unquantified ordinary/noncapturing wrappers may surround the
/// assertion and its whole operand. Outside captures remain empty; operand
/// wrappers capture the selected longest accepted member, including a defined
/// empty member. Original input context and UTF-16 extents are retained.
#[derive(Clone, Debug)]
pub struct RegExpUnicodePositiveStringClassMatcher {
    operand: RegExpUnicodeStringSetMatcher,
    outside_captures: usize,
    captures: usize,
}

#[cfg(test)]
#[path = "regexp_unicode_positive_string_class_tests.rs"]
mod tests;

impl RegExpUnicodePositiveStringClassMatcher {
    /// Compiles one unquantified single-q lookahead with optional operand ^/$ gates.
    ///
    /// The class must contain an empty or longer member. Every q alternative is
    /// decoded and folded by the checked string-set compiler. Full original
    /// v/vi validation and normalized capture heads are required; projected
    /// source is never independently validated or rebound. Other class operands,
    /// counts, word gates and binding roles remain separate proofs (22.2.2.4,
    /// 22.2.2.7, 22.2.2.9).
    pub fn compile_single_q_lookahead_validated_with_work<E>(
        source: &JsString,
        ignore_case: bool,
        multiline: bool,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, RegExpUnicodeSetCompileError<E>> {
        Self::compile_with_work_impl(source, ignore_case, multiline, &mut charge)
    }

    #[inline(never)]
    fn compile_with_work_impl<E>(
        source: &JsString,
        ignore_case: bool,
        multiline: bool,
        mut charge: &mut dyn FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, RegExpUnicodeSetCompileError<E>> {
        charge(source.len().saturating_mul(16).saturating_add(16))
            .map_err(RegExpUnicodeSetCompileError::Work)?;
        let units = source.code_units();
        let (assertion, outside_captures) = if let Some(scope) = enclosing_group_body(units, true) {
            (&units[scope.body], scope.captures)
        } else {
            (units, 0)
        };
        if !assertion.starts_with(&[0x28, 0x3f, 0x3d])
            || group_end(assertion, 0, true) != Some(assertion.len())
        {
            return Ok(None);
        }
        let body = &assertion[3..assertion.len() - 1];
        let (body, inside_captures) = if let Some(scope) = enclosing_group_body(body, true) {
            (&body[scope.body], scope.captures)
        } else {
            (body, 0)
        };
        let start = usize::from(body.first() == Some(&0x5e));
        if body.get(start) != Some(&0x5b) {
            return Ok(None);
        }
        let Some(end) = surrogate_atom_end(body, start, true, true) else {
            return Ok(None);
        };
        if end != body.len() && !(body.get(end) == Some(&0x24) && end + 1 == body.len()) {
            return Ok(None);
        }
        let mut projected = Vec::new();
        projected
            .try_reserve_exact(body.len())
            .map_err(|_| RegExpUnicodeSetCompileError::Capacity)?;
        projected.extend_from_slice(body);
        let operand = RegExpUnicodeStringSetMatcher::compile_with_work(
            &JsString::from_code_units(projected),
            ignore_case,
            multiline,
            &mut charge,
        )?;
        Ok(operand.map(|operand| Self {
            operand,
            outside_captures,
            captures: outside_captures + inside_captures,
        }))
    }

    /// Returns the complete original lexical capture inventory.
    pub fn capture_count(&self) -> usize {
        self.captures
    }

    /// Returns one outside or operand-wrapper capture without allocating state.
    pub fn capture_range(
        &self,
        index: usize,
        matched: &RegExpUnicodePositiveAtomMatch,
    ) -> Option<Range<usize>> {
        if index >= self.captures {
            None
        } else if index < self.outside_captures {
            Some(matched.range.clone())
        } else {
            Some(matched.operand.clone())
        }
    }

    /// Finds the first assertion position, retaining the selected whole member.
    ///
    /// An accepted empty member exports a defined empty operand range. Search
    /// meters visited string-trie steps and allocates no input-sized state.
    pub fn find_with_work<E>(
        &self,
        input: &JsString,
        start: usize,
        sticky: bool,
        charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<RegExpUnicodePositiveAtomMatch>, E> {
        self.operand
            .find_with_work(input, start, sticky, charge)
            .map(|found| {
                found.map(|operand| RegExpUnicodePositiveAtomMatch {
                    range: operand.start..operand.start,
                    operand,
                })
            })
    }
}
