//! Root nullable alternatives whose class result contains no empty string.
use super::{CaptureRole, has_root_alternative, parse_body, projected_capture_range};
use crate::{
    JsString, RegExpUnicodeRepeatedAtomMatch, RegExpUnicodeSetCompileError,
    RegExpUnicodeStringSetMatcher, regexp_literal::unicode_start,
};
use std::ops::Range;

/// One root string-class/empty alternative with flat original capture projection.
///
/// The class result must exclude the empty string. Its nonzero selected range
/// therefore distinguishes the consuming branch from the empty branch, including
/// variable-width words. Captures retain original UTF-16 ranges; clone/drop use
/// flat source-sized storage and matching retains no capture history.
#[derive(Clone, Debug)]
pub struct RegExpUnicodeNullableStringClassCaptureMatcher {
    class: RegExpUnicodeStringSetMatcher,
    captures: Box<[CaptureRole]>,
    empty_first: bool,
}
impl RegExpUnicodeNullableStringClassCaptureMatcher {
    /// Proves one ungrouped finite string-class versus empty alternative.
    ///
    /// Requires validated v/vi input and normalized capture names. Existing
    /// standalone, union, nested-union and finite/scalar-filtered string class
    /// proofs must compile the complete class, with an empty or longer q member
    /// somewhere in its source. The resulting set must not contain the empty
    /// string, even when the root empty branch comes first. Root capture roles,
    /// empty counts/choices and first-branch selection reuse the flat parser.
    /// Assertions, references, further consuming terms, class-bearing counts,
    /// multiple class alternatives and unknown leaves retain separate proofs.
    /// No original edge assertion is extracted or moved (22.2.2).
    /// https://262.ecma-international.org/17.0/#sec-compileatom
    pub fn compile_with_work<E>(
        source: &JsString,
        ignore_case: bool,
        multiline: bool,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, RegExpUnicodeSetCompileError<E>> {
        let units = source.code_units();
        charge(units.len().saturating_mul(96).saturating_add(256))
            .map_err(RegExpUnicodeSetCompileError::Work)?;
        if !has_root_alternative(units) {
            return Ok(None);
        }
        let mut body = Vec::with_capacity(units.len().saturating_add(4));
        body.extend_from_slice(&[0x28, 0x3f, 0x3a]);
        body.extend_from_slice(units);
        body.push(0x29);
        let Some(parsed) = parse_body(&body, true, true, true, true, true, false) else {
            return Ok(None);
        };
        let source = JsString::from_code_units(body[parsed.class.clone()].to_vec());
        let mut class = RegExpUnicodeStringSetMatcher::compile_with_work(
            &source,
            ignore_case,
            multiline,
            &mut charge,
        )?;
        if class.is_none() {
            class = RegExpUnicodeStringSetMatcher::compile_string_union_with_work(
                &source,
                ignore_case,
                multiline,
                &mut charge,
            )?;
        }
        if class.is_none() {
            class = RegExpUnicodeStringSetMatcher::compile_character_union_with_work(
                &source,
                ignore_case,
                multiline,
                &mut charge,
            )?;
        }
        if class.is_none() {
            class = RegExpUnicodeStringSetMatcher::compile_scalar_union_with_work(
                &source,
                ignore_case,
                multiline,
                &mut charge,
            )?;
        }
        if class.is_none() {
            class = RegExpUnicodeStringSetMatcher::compile_positive_nested_union_with_work(
                &source,
                ignore_case,
                multiline,
                &mut charge,
            )?;
        }
        if class.is_none() {
            class = RegExpUnicodeStringSetMatcher::compile_string_operations_with_work(
                &source,
                ignore_case,
                multiline,
                &mut charge,
            )?;
        }
        if class.is_none() {
            class = RegExpUnicodeStringSetMatcher::compile_finite_string_operations_with_work(
                &source,
                ignore_case,
                multiline,
                &mut charge,
            )?;
        }
        if class.is_none() {
            class = RegExpUnicodeStringSetMatcher::compile_scalar_rhs_string_operations_with_work(
                &source,
                ignore_case,
                multiline,
                &mut charge,
            )?;
        }
        let Some(class) = class else { return Ok(None) };
        if class.has_empty_member() {
            return Ok(None);
        }
        Ok(Some(Self {
            class,
            captures: parsed.captures.into_boxed_slice(),
            empty_first: parsed.empty_first,
        }))
    }

    /// Number of original captures in source order.
    pub fn capture_count(&self) -> usize {
        self.captures.len()
    }

    /// Finds the first valid Unicode boundary and the selected longest class word.
    ///
    /// Both sticky and searching modes stop at that boundary because the empty
    /// branch can always succeed. A root empty-first alternative skips the class
    /// query. Work is charged before inspecting the bounded class prefix; initial
    /// offsets inside a surrogate pair use the approved leading boundary.
    pub fn find_with_captures_and_work<E>(
        &self,
        input: &JsString,
        start: usize,
        _sticky: bool,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<RegExpUnicodeRepeatedAtomMatch>, E> {
        charge(4)?;
        let Some(start) = unicode_start(input, start) else {
            return Ok(None);
        };
        let end = if self.empty_first {
            start
        } else {
            let inspected = (input.len() - start)
                .min(self.class.max_match_code_units())
                .saturating_add(1);
            charge(inspected.saturating_mul(self.class.search_passes(true)))?;
            self.class.find(input, start, true).map_or(start, |range| {
                debug_assert_eq!(range.start, start);
                debug_assert!(range.end > start);
                range.end
            })
        };
        let range = start..end;
        Ok(Some(RegExpUnicodeRepeatedAtomMatch {
            range: range.clone(),
            atom_capture: Some(range),
        }))
    }

    /// Projects a zero-based original capture from the selected root branch.
    pub fn capture_range(
        &self,
        index: usize,
        iteration: Option<&Range<usize>>,
    ) -> Option<Range<usize>> {
        projected_capture_range(&self.captures, index, iteration)
    }
}
