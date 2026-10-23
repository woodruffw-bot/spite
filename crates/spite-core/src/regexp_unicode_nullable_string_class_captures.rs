//! Root string-class alternatives with separate class/empty selection proofs.
use super::{
    CaptureRole, has_root_alternative, parse_body, projected_capture_range,
    projected_selected_capture_range,
};
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
        charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, RegExpUnicodeSetCompileError<E>> {
        compile_string_root(source, ignore_case, multiline, false, charge).map(|plan| {
            plan.map(|plan| Self {
                class: plan.class,
                captures: plan.captures,
                empty_first: plan.empty_first,
            })
        })
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

struct StringRootPlan {
    class: RegExpUnicodeStringSetMatcher,
    captures: Box<[CaptureRole]>,
    empty_first: bool,
}
fn compile_string_root<E>(
    source: &JsString,
    ignore_case: bool,
    multiline: bool,
    want_empty: bool,
    mut charge: impl FnMut(usize) -> Result<(), E>,
) -> Result<Option<StringRootPlan>, RegExpUnicodeSetCompileError<E>> {
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
    if class.has_empty_member() != want_empty {
        return Ok(None);
    }
    Ok(Some(StringRootPlan {
        class,
        captures: parsed.captures.into_boxed_slice(),
        empty_first: parsed.empty_first,
    }))
}

/// A root string-class match with explicit selected-alternative identity.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RegExpUnicodeStringClassBranchMatch {
    /// Complete original UTF-16 match range.
    pub range: Range<usize>,
    /// True for a selected class member, including a zero-width member.
    pub class_selected: bool,
}

/// A root string-class/empty alternative whose class contains the empty string.
///
/// The explicit selected-branch result distinguishes a class capture of an empty
/// member from the root empty alternative. Storage, clone and drop remain flat
/// and source-sized; matching retains no capture history.
#[derive(Clone, Debug)]
pub struct RegExpUnicodeEmptyStringClassCaptureMatcher {
    class: RegExpUnicodeStringSetMatcher,
    captures: Box<[CaptureRole]>,
    empty_first: bool,
}
impl RegExpUnicodeEmptyStringClassCaptureMatcher {
    /// Proves root alternatives with an empty member in the compiled class set.
    ///
    /// Requires validated v/vi input and normalized capture names. Class proofs,
    /// root ownership, flat empty effects and closed assertion/reference/count
    /// domains are shared with `RegExpUnicodeNullableStringClassCaptureMatcher`.
    /// Complete class validation precedes selection. Longer class words precede
    /// its empty member, while original root-alternative order is preserved.
    /// Captures follow explicit branch selection even for zero-width class
    /// members (22.2.2.3, 22.2.2.7).
    /// https://262.ecma-international.org/17.0/#sec-compileatom
    pub fn compile_with_work<E>(
        source: &JsString,
        ignore_case: bool,
        multiline: bool,
        charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, RegExpUnicodeSetCompileError<E>> {
        compile_string_root(source, ignore_case, multiline, true, charge).map(|plan| {
            plan.map(|plan| Self {
                class: plan.class,
                captures: plan.captures,
                empty_first: plan.empty_first,
            })
        })
    }

    /// Number of original captures in source order.
    pub fn capture_count(&self) -> usize {
        self.captures.len()
    }

    /// Selects one root branch at the first valid Unicode boundary.
    ///
    /// Empty-first roots skip the class query. Otherwise the longest class word,
    /// including its empty member, retains class identity. Both sticky/searching
    /// modes stop at that boundary. Work precedes boundary and prefix inspection.
    pub fn find_with_captures_and_work<E>(
        &self,
        input: &JsString,
        start: usize,
        _sticky: bool,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<RegExpUnicodeStringClassBranchMatch>, E> {
        charge(4)?;
        let Some(start) = unicode_start(input, start) else {
            return Ok(None);
        };
        let (end, class_selected) = if self.empty_first {
            (start, false)
        } else {
            let inspected = (input.len() - start)
                .min(self.class.max_match_code_units())
                .saturating_add(1);
            charge(inspected.saturating_mul(self.class.search_passes(true)))?;
            let found = self.class.find(input, start, true);
            debug_assert!(found.is_some());
            found.map_or((start, false), |range| {
                debug_assert_eq!(range.start, start);
                (range.end, true)
            })
        };
        Ok(Some(RegExpUnicodeStringClassBranchMatch {
            range: start..end,
            class_selected,
        }))
    }

    /// Projects an original capture using explicit class/empty branch identity.
    pub fn capture_range(
        &self,
        index: usize,
        range: &Range<usize>,
        class_selected: bool,
    ) -> Option<Range<usize>> {
        projected_selected_capture_range(&self.captures, index, range, class_selected)
    }
}
