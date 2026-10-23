//! Unquantified string-class alternatives with explicit class/empty proofs.
use super::{
    CaptureRole, has_root_alternative, parse_body, projected_capture_range,
    projected_selected_capture_range,
};
use crate::{
    JsString, RegExpUnicodeRepeatedAtomMatch, RegExpUnicodeSetCompileError,
    RegExpUnicodeStringSetMatcher, regexp_literal::unicode_start,
};
use std::ops::Range;

/// An unquantified string-class/empty alternative with flat capture projection.
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
        compile_string_alternative(
            source,
            ignore_case,
            multiline,
            AlternativeScope::Root,
            false,
            charge,
        )
        .map(|plan| {
            plan.map(|plan| Self {
                class: plan.class,
                captures: plan.captures,
                empty_first: plan.empty_first,
            })
        })
    }

    /// Proves one grouped class/empty alternative without a class-bearing count.
    ///
    /// Requires validated v/vi input and normalized capture names. The full
    /// source retains its alternative owner, complete enclosing captures and
    /// empty sibling points. Class compilation and empty effects are shared
    /// with `compile_with_work`; root alternatives, assertions, references and
    /// additional consuming terms retain separate proofs. No source wrapper is
    /// peeled, so enclosing ranges cover either selected branch (22.2.2.3).
    /// https://262.ecma-international.org/17.0/#sec-disjunction
    pub fn compile_grouped_with_work<E>(
        source: &JsString,
        ignore_case: bool,
        multiline: bool,
        charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, RegExpUnicodeSetCompileError<E>> {
        compile_string_alternative(
            source,
            ignore_case,
            multiline,
            AlternativeScope::Grouped,
            false,
            charge,
        )
        .map(|plan| {
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

struct StringAlternativePlan {
    class: RegExpUnicodeStringSetMatcher,
    captures: Box<[CaptureRole]>,
    empty_first: bool,
}
#[derive(Clone, Copy)]
enum AlternativeScope {
    Root,
    Grouped,
}

fn compile_string_alternative<E>(
    source: &JsString,
    ignore_case: bool,
    multiline: bool,
    scope: AlternativeScope,
    want_empty: bool,
    mut charge: impl FnMut(usize) -> Result<(), E>,
) -> Result<Option<StringAlternativePlan>, RegExpUnicodeSetCompileError<E>> {
    let units = source.code_units();
    charge(units.len().saturating_mul(96).saturating_add(256))
        .map_err(RegExpUnicodeSetCompileError::Work)?;
    if has_root_alternative(units) != matches!(scope, AlternativeScope::Root) {
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
    let Some(class) = compile_string_class(&source, ignore_case, multiline, &mut charge)? else {
        return Ok(None);
    };
    if class.has_empty_member() != want_empty {
        return Ok(None);
    }
    Ok(Some(StringAlternativePlan {
        class,
        captures: parsed.captures.into_boxed_slice(),
        empty_first: parsed.empty_first,
    }))
}

/// A string-class match with explicit selected-alternative identity.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RegExpUnicodeStringClassBranchMatch {
    /// Complete original UTF-16 match range.
    pub range: Range<usize>,
    /// True for a selected class member, including a zero-width member.
    pub class_selected: bool,
}

/// A string-class/empty alternative whose class contains the empty string.
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
        compile_string_alternative(
            source,
            ignore_case,
            multiline,
            AlternativeScope::Root,
            true,
            charge,
        )
        .map(|plan| {
            plan.map(|plan| Self {
                class: plan.class,
                captures: plan.captures,
                empty_first: plan.empty_first,
            })
        })
    }

    /// Proves one grouped class/empty alternative without a class-bearing count.
    ///
    /// Requires validated v/vi input and normalized capture names. The full
    /// source retains its alternative owner, complete enclosing captures and
    /// empty sibling points. Class compilation and empty effects are shared
    /// with `compile_with_work`; root alternatives, assertions, references and
    /// additional consuming terms retain separate proofs. No source wrapper is
    /// peeled, so enclosing ranges cover either selected branch (22.2.2.3).
    /// https://262.ecma-international.org/17.0/#sec-disjunction
    pub fn compile_grouped_with_work<E>(
        source: &JsString,
        ignore_case: bool,
        multiline: bool,
        charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, RegExpUnicodeSetCompileError<E>> {
        compile_string_alternative(
            source,
            ignore_case,
            multiline,
            AlternativeScope::Grouped,
            true,
            charge,
        )
        .map(|plan| {
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

    /// Selects a class/empty branch at the first valid Unicode boundary.
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

fn compile_string_class<E>(
    source: &JsString,
    ignore_case: bool,
    multiline: bool,
    mut charge: impl FnMut(usize) -> Result<(), E>,
) -> Result<Option<RegExpUnicodeStringSetMatcher>, RegExpUnicodeSetCompileError<E>> {
    let mut class = RegExpUnicodeStringSetMatcher::compile_with_work(
        source,
        ignore_case,
        multiline,
        &mut charge,
    )?;
    if class.is_none() {
        class = RegExpUnicodeStringSetMatcher::compile_string_union_with_work(
            source,
            ignore_case,
            multiline,
            &mut charge,
        )?;
    }
    if class.is_none() {
        class = RegExpUnicodeStringSetMatcher::compile_character_union_with_work(
            source,
            ignore_case,
            multiline,
            &mut charge,
        )?;
    }
    if class.is_none() {
        class = RegExpUnicodeStringSetMatcher::compile_scalar_union_with_work(
            source,
            ignore_case,
            multiline,
            &mut charge,
        )?;
    }
    if class.is_none() {
        class = RegExpUnicodeStringSetMatcher::compile_positive_nested_union_with_work(
            source,
            ignore_case,
            multiline,
            &mut charge,
        )?;
    }
    if class.is_none() {
        class = RegExpUnicodeStringSetMatcher::compile_string_operations_with_work(
            source,
            ignore_case,
            multiline,
            &mut charge,
        )?;
    }
    if class.is_none() {
        class = RegExpUnicodeStringSetMatcher::compile_finite_string_operations_with_work(
            source,
            ignore_case,
            multiline,
            &mut charge,
        )?;
    }
    if class.is_none() {
        class = RegExpUnicodeStringSetMatcher::compile_scalar_rhs_string_operations_with_work(
            source,
            ignore_case,
            multiline,
            &mut charge,
        )?;
    }
    Ok(class)
}

/// One unquantified string class with complete and empty point captures.
///
/// Captures retain lexical slots and original UTF-16 ranges, including defined
/// empty members. Optional/later empty groups remain undefined. Storage, clone
/// and drop are flat and source-sized; matching allocates no capture history.
#[derive(Clone, Debug)]
pub struct RegExpUnicodeSingleStringClassCaptureMatcher {
    class: RegExpUnicodeStringSetMatcher,
    captures: Box<[CaptureRole]>,
}
impl RegExpUnicodeSingleStringClassCaptureMatcher {
    /// Proves one string class with flat capturing/noncapturing empty siblings.
    ///
    /// Requires validated v/vi input and normalized capture names. Complete
    /// enclosing captures and empty sibling points preserve group entry/exit
    /// ranges (22.2.2.3, 22.2.2.7). Class members prefer the longest word, including
    /// empty members. Complete class validation precedes matching. Assertions,
    /// references, class-bearing counts, additional consuming terms and nullable
    /// alternatives retain separate proofs.
    /// https://262.ecma-international.org/17.0/#sec-compileatom
    pub fn compile_with_work<E>(
        source: &JsString,
        ignore_case: bool,
        multiline: bool,
        charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, RegExpUnicodeSetCompileError<E>> {
        Self::compile_single(source, ignore_case, multiline, false, charge)
    }

    /// Proves a single string class with canonical outside word/line assertions.
    ///
    /// Requires validated v/vi input and normalized capture names. Leading ^ and
    /// word gates, plus trailing word gates and $, retain whole-pattern scope.
    /// Original root disjunctions decline before extracting any assertion. Empty
    /// siblings and complete captures retain their ranges. A rejected longer
    /// class member leaves shorter successful continuations eligible (22.2.2.4,
    /// 22.2.2.7). Inner assertions, references and class counts stay separate.
    /// https://262.ecma-international.org/17.0/#sec-compileassertion
    pub fn compile_with_outer_assertions_and_work<E>(
        source: &JsString,
        ignore_case: bool,
        multiline: bool,
        charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, RegExpUnicodeSetCompileError<E>> {
        Self::compile_single(source, ignore_case, multiline, true, charge)
    }

    /// Proves complete ordinary wrappers enclosing canonical outside assertions.
    ///
    /// Requires validated v/vi input and normalized capture names. Complete
    /// unquantified capturing/noncapturing wrappers retain whole-match ranges
    /// and original lexical slots (22.2.2.7). Three flat scans keep nested v
    /// classes and escaped string data opaque; the complete retained body must
    /// pass `compile_with_outer_assertions_and_work`. Branch-local assertions,
    /// class-bearing counts, references and unknown leaves stay separate.
    /// https://262.ecma-international.org/17.0/#sec-compileatom
    pub fn compile_enclosed_outer_assertions_with_work<E>(
        source: &JsString,
        ignore_case: bool,
        multiline: bool,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, RegExpUnicodeSetCompileError<E>> {
        charge(source.len().saturating_mul(8).saturating_add(64))
            .map_err(RegExpUnicodeSetCompileError::Work)?;
        let Some((body, captures)) = enclosed_v_group_body(source.code_units()) else {
            return Ok(None);
        };
        let body = JsString::from_code_units(source.code_units()[body].to_vec());
        let Some(mut plan) =
            Self::compile_with_outer_assertions_and_work(&body, ignore_case, multiline, charge)?
        else {
            return Ok(None);
        };
        if captures != 0 {
            let mut roles = vec![CaptureRole::Iteration; captures];
            roles.extend_from_slice(&plan.captures);
            plan.captures = roles.into_boxed_slice();
        }
        Ok(Some(plan))
    }

    /// Proves line assertions surrounding complete asserted-class wrappers.
    ///
    /// Requires validated v/vi input and normalized capture names. Canonical
    /// outer ^/$ gates commute through complete unquantified wrappers; any
    /// equal inner line gates are idempotent at the same input position. Inner
    /// word gates retain their order and every original capture slot remains
    /// unchanged (22.2.2.4, 22.2.2.7). Root disjunctions decline before movement,
    /// and the entire retained body must pass the existing assertion proof.
    /// https://262.ecma-international.org/17.0/#sec-compileassertion
    pub fn compile_surrounding_line_assertions_with_work<E>(
        source: &JsString,
        ignore_case: bool,
        multiline: bool,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, RegExpUnicodeSetCompileError<E>> {
        let units = source.code_units();
        charge(units.len().saturating_mul(8).saturating_add(64))
            .map_err(RegExpUnicodeSetCompileError::Work)?;
        if has_root_alternative(units) {
            return Ok(None);
        }
        let start = units.first() == Some(&0x5e);
        let end = units.last() == Some(&0x24) && !super::escaped(units, units.len() - 1);
        if !start && !end {
            return Ok(None);
        }
        let Some(wrapped) = units.get(usize::from(start)..units.len() - usize::from(end)) else {
            return Ok(None);
        };
        let Some((body, captures)) = enclosed_v_group_body(wrapped) else {
            return Ok(None);
        };
        let body = &wrapped[body];
        let mut canonical = Vec::with_capacity(units.len());
        if start && body.first() != Some(&0x5e) {
            canonical.push(0x5e);
        }
        canonical.extend_from_slice(body);
        let inner_end = body.last() == Some(&0x24) && !super::escaped(body, body.len() - 1);
        if end && !inner_end {
            canonical.push(0x24);
        }
        let canonical = JsString::from_code_units(canonical);
        let Some(mut plan) = Self::compile_with_outer_assertions_and_work(
            &canonical,
            ignore_case,
            multiline,
            charge,
        )?
        else {
            return Ok(None);
        };
        if captures != 0 {
            let mut roles = vec![CaptureRole::Iteration; captures];
            roles.extend_from_slice(&plan.captures);
            plan.captures = roles.into_boxed_slice();
        }
        Ok(Some(plan))
    }

    fn compile_single<E>(
        source: &JsString,
        ignore_case: bool,
        multiline: bool,
        assertions: bool,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, RegExpUnicodeSetCompileError<E>> {
        let units = source.code_units();
        charge(units.len().saturating_mul(96).saturating_add(256))
            .map_err(RegExpUnicodeSetCompileError::Work)?;
        if has_root_alternative(units) {
            return Ok(None);
        }
        let (begin, end) = if assertions {
            let mut begin = usize::from(units.first() == Some(&0x5e));
            let mut end = units.len()
                - usize::from(
                    units.last() == Some(&0x24) && !super::escaped(units, units.len() - 1),
                );
            if matches!(
                units.get(begin..end).and_then(|body| body.get(..2)),
                Some([0x5c, 0x62 | 0x42])
            ) {
                begin += 2;
            }
            if end >= begin + 2
                && units[end - 2] == 0x5c
                && matches!(units[end - 1], 0x62 | 0x42)
                && !super::escaped(units, end - 2)
            {
                end -= 2;
            }
            if begin == 0 && end == units.len() {
                return Ok(None);
            }
            (begin, end)
        } else {
            (0, units.len())
        };
        let Some(inner) = units.get(begin..end) else {
            return Ok(None);
        };
        let mut body = Vec::with_capacity(inner.len().saturating_add(4));
        body.extend_from_slice(&[0x28, 0x3f, 0x3a]);
        body.extend_from_slice(inner);
        body.push(0x29);
        let Some(parsed) = parse_body(&body, true, true, true, true, true, true) else {
            return Ok(None);
        };
        let mut class_source = Vec::with_capacity(units.len());
        class_source.extend_from_slice(&units[..begin]);
        class_source.extend_from_slice(&body[parsed.class]);
        class_source.extend_from_slice(&units[end..]);
        let class_source = JsString::from_code_units(class_source);
        let Some(class) = compile_string_class(&class_source, ignore_case, multiline, charge)?
        else {
            return Ok(None);
        };
        Ok(Some(Self {
            class,
            captures: parsed.captures.into_boxed_slice(),
        }))
    }

    /// Number of original captures in lexical source order.
    pub fn capture_count(&self) -> usize {
        self.captures.len()
    }

    /// Finds the earliest start and longest class member with original ranges.
    ///
    /// Work is charged before Unicode boundary and bounded prefix/search scans.
    /// Defined empty members retain class identity without width-based inference.
    pub fn find_with_captures_and_work<E>(
        &self,
        input: &JsString,
        start: usize,
        sticky: bool,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Range<usize>>, E> {
        charge(4)?;
        let Some(start) = unicode_start(input, start) else {
            return Ok(None);
        };
        let remaining = input.len() - start;
        let first_only =
            sticky || (self.class.has_empty_member() && !self.class.has_outside_assertions());
        let inspected = if first_only {
            remaining.min(self.class.max_match_code_units())
        } else {
            remaining
        }
        .saturating_add(1);
        charge(inspected.saturating_mul(self.class.search_passes(first_only)))?;
        Ok(self.class.find(input, start, sticky))
    }

    /// Projects an original capture from the selected class member's range.
    pub fn capture_range(&self, index: usize, range: &Range<usize>) -> Option<Range<usize>> {
        projected_selected_capture_range(&self.captures, index, range, true)
    }
}

// Recover only complete ordinary wrappers in validated v source. Unlike the
// legacy outer-group scanner, every nested class bracket remains opaque here.
fn enclosed_v_group_body(units: &[u16]) -> Option<(Range<usize>, usize)> {
    fn opener(units: &[u16], at: usize) -> Option<usize> {
        let tail = units.get(at..)?;
        if tail.starts_with(&[0x28, 0x3f, 0x3a]) {
            Some(3)
        } else if tail.first() == Some(&0x28) && tail.get(1) != Some(&0x3f) {
            Some(1)
        } else {
            None
        }
    }
    let mut opening_end = 0;
    let mut leading = 0;
    while let Some(width) = opener(units, opening_end) {
        opening_end += width;
        leading += 1;
    }
    if leading == 0 {
        return None;
    }
    let (mut remaining, mut depth, mut class, mut complete, mut at) =
        (leading, 0usize, 0usize, 0usize, 0usize);
    while let Some(&unit) = units.get(at) {
        at += 1;
        if unit == 0x5c {
            units.get(at)?;
            at += 1;
            continue;
        }
        if unit == 0x5b {
            class += 1;
            continue;
        }
        if class != 0 {
            if unit == 0x5d {
                class -= 1;
            }
            continue;
        }
        match unit {
            0x28 => depth += 1,
            0x29 => {
                depth = depth.checked_sub(1)?;
                if depth < remaining {
                    remaining = depth;
                    if at == units.len().checked_sub(depth)? {
                        complete = complete.max(depth + 1);
                    }
                }
            }
            _ => {}
        }
    }
    if depth != 0 || class != 0 || complete == 0 {
        return None;
    }
    let (mut begin, mut captures) = (0, 0);
    for _ in 0..complete {
        let width = opener(units, begin)?;
        captures += usize::from(width == 1);
        begin += width;
    }
    Some((begin..units.len() - complete, captures))
}
