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

#[path = "regexp_unicode_repeated_string_class_captures.rs"]
mod repeated_string_classes;
pub use repeated_string_classes::{
    RegExpUnicodeAssertedRepeatedStringClassCaptureMatcher,
    RegExpUnicodeBoundedStringClassCaptureMatcher,
    RegExpUnicodeCountedNullableStringClassCaptureMatcher, RegExpUnicodeMatchError,
    RegExpUnicodePrefixFreeStringClassCaptureMatcher,
    RegExpUnicodeRepeatedStringClassCaptureMatcher, RegExpUnicodeRepeatedStringClassMatch,
    RegExpUnicodeRequiredStringClassCaptureMatcher,
    RegExpUnicodeTwoRequiredStringClassCaptureMatcher,
};

#[path = "regexp_unicode_asserted_string_class_alternatives.rs"]
mod asserted_alternatives;
pub use asserted_alternatives::{
    RegExpUnicodeAssertedStringClassAlternativeMatcher,
    RegExpUnicodeSurroundingAssertedStringClassAlternativeMatcher,
};

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

    /// Proves exact-one counts in root or grouped class/empty alternatives.
    ///
    /// Requires validated v/vi input and normalized capture names. The sole
    /// class atom and its ordinary ancestors may use {1}/{1,1}, padded or lazy.
    /// RepeatMatcher executes the required body once, including empty selected
    /// alternatives. Removing only count tokens preserves lexical captures and
    /// empty sibling participation in this reference-free choice. The full
    /// existing root/grouped proof still validates every original branch and
    /// leaf before selection. Assertions, references and other class-path counts
    /// remain separate proofs (22.2.2.3, RepeatMatcher).
    /// The resulting class must exclude the empty string.
    /// https://262.ecma-international.org/17.0/#sec-repeatmatcher
    pub fn compile_exact_single_counts_with_work<E>(
        source: &JsString,
        ignore_case: bool,
        multiline: bool,
        charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, RegExpUnicodeSetCompileError<E>> {
        compile_exact_single_string_alternative(source, ignore_case, multiline, false, charge).map(
            |plan| {
                plan.map(|plan| Self {
                    class: plan.class,
                    captures: plan.captures,
                    empty_first: plan.empty_first,
                })
            },
        )
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

    /// Proves exact-one counts in root or grouped class/empty alternatives.
    ///
    /// Requires validated v/vi input and normalized capture names. The sole
    /// class atom and its ordinary ancestors may use {1}/{1,1}, padded or lazy.
    /// RepeatMatcher executes the required body once, including empty selected
    /// alternatives. Removing only count tokens preserves lexical captures and
    /// empty sibling participation in this reference-free choice. The full
    /// existing root/grouped proof still validates every original branch and
    /// leaf before selection. Assertions, references and other class-path counts
    /// remain separate proofs (22.2.2.3, RepeatMatcher).
    /// The resulting class must include the empty string; explicit branch identity retains its capture participation.
    /// https://262.ecma-international.org/17.0/#sec-repeatmatcher
    pub fn compile_exact_single_counts_with_work<E>(
        source: &JsString,
        ignore_case: bool,
        multiline: bool,
        charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, RegExpUnicodeSetCompileError<E>> {
        compile_exact_single_string_alternative(source, ignore_case, multiline, true, charge).map(
            |plan| {
                plan.map(|plan| Self {
                    class: plan.class,
                    captures: plan.captures,
                    empty_first: plan.empty_first,
                })
            },
        )
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

fn compile_exact_single_string_alternative<E>(
    source: &JsString,
    ignore_case: bool,
    multiline: bool,
    want_empty: bool,
    mut charge: impl FnMut(usize) -> Result<(), E>,
) -> Result<Option<StringAlternativePlan>, RegExpUnicodeSetCompileError<E>> {
    charge(source.len().saturating_mul(24).saturating_add(128))
        .map_err(RegExpUnicodeSetCompileError::Work)?;
    let Some(normalized) = asserted_alternatives::exact_single_count_source(source.code_units())
    else {
        return Ok(None);
    };
    let source = JsString::from_code_units(normalized);
    let scope = if has_root_alternative(source.code_units()) {
        AlternativeScope::Root
    } else {
        AlternativeScope::Grouped
    };
    compile_string_alternative(&source, ignore_case, multiline, scope, want_empty, charge)
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

    /// Proves exact-one counts around one class and its ordinary ancestors.
    ///
    /// Requires validated v/vi input and normalized capture names. Counts {1}
    /// and {1,1}, with leading zeros and lazy forms, execute the body once even
    /// when empty. Removing only class-path count tokens preserves lexical
    /// slots, point captures and mandatory predicates in this reference-free
    /// spine. Plain, canonical and reversed assertion proofs still validate
    /// every original term and class, including impossible conjunctions.
    /// Empty sibling counts retain their original capture participation; other
    /// class counts, references and class-bearing choices remain separate
    /// proofs (22.2.2.3, RepeatMatcher).
    /// https://262.ecma-international.org/17.0/#sec-repeatmatcher
    pub fn compile_exact_single_counts_with_work<E>(
        source: &JsString,
        ignore_case: bool,
        multiline: bool,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, RegExpUnicodeSetCompileError<E>> {
        charge(source.len().saturating_mul(24).saturating_add(128))
            .map_err(RegExpUnicodeSetCompileError::Work)?;
        let Some(normalized) =
            asserted_alternatives::exact_single_count_source(source.code_units())
        else {
            return Ok(None);
        };
        let source = JsString::from_code_units(normalized);
        if let Some(plan) = Self::compile_with_work(&source, ignore_case, multiline, &mut charge)? {
            return Ok(Some(plan));
        }
        if let Some(plan) = Self::compile_unconditional_assertions_with_work(
            &source,
            ignore_case,
            multiline,
            &mut charge,
        )? {
            return Ok(Some(plan));
        }
        Self::compile_reversed_line_assertions_with_work(&source, ignore_case, multiline, charge)
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

    /// Proves canonical word gates surrounding complete asserted-class wrappers.
    ///
    /// Requires validated v/vi input and normalized capture names. Equal word
    /// gates on the same side are idempotent; opposite word gates on that side
    /// form a false conjunction (22.2.2.4). The complete retained class/body is
    /// proved even for a contradiction before selecting a never-matching plan.
    /// Complete lexical wrapper captures and original ranges remain unchanged.
    /// Root choices, class counts, inner assertions and unknown leaves stay
    /// separate proofs. At least one original outside word gate is required.
    /// https://262.ecma-international.org/17.0/#sec-compileassertion
    pub fn compile_surrounding_word_assertions_with_work<E>(
        source: &JsString,
        ignore_case: bool,
        multiline: bool,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, RegExpUnicodeSetCompileError<E>> {
        let units = source.code_units();
        charge(units.len().saturating_mul(8).saturating_add(96))
            .map_err(RegExpUnicodeSetCompileError::Work)?;
        if has_root_alternative(units) {
            return Ok(None);
        }
        let Some(outer) = single_word_gate_frame(units) else {
            return Ok(None);
        };
        if outer.start_word.is_none() && outer.end_word.is_none() {
            return Ok(None);
        }
        let Some(wrapped) = units.get(outer.body) else {
            return Ok(None);
        };
        let Some((body, captures)) = enclosed_v_group_body(wrapped) else {
            return Ok(None);
        };
        let body = &wrapped[body];
        let Some(inner) = single_word_gate_frame(body) else {
            return Ok(None);
        };
        let Some(payload) = body.get(inner.body) else {
            return Ok(None);
        };
        let conflict = outer
            .start_word
            .zip(inner.start_word)
            .is_some_and(|(a, b)| a != b)
            || outer
                .end_word
                .zip(inner.end_word)
                .is_some_and(|(a, b)| a != b);
        let mut plan = if conflict {
            // Validate every original class leaf before lowering false gates.
            let body = JsString::from_code_units(payload.to_vec());
            let Some(mut plan) =
                Self::compile_with_work(&body, ignore_case, multiline, &mut charge)?
            else {
                return Ok(None);
            };
            let never = JsString::from(r"\b[\q{}]\B");
            let Some(never) = compile_string_class(&never, ignore_case, multiline, &mut charge)?
            else {
                return Ok(None);
            };
            plan.class = never;
            plan
        } else {
            let mut canonical = Vec::with_capacity(units.len());
            if outer.start_line || inner.start_line {
                canonical.push(0x5e);
            }
            if let Some(word) = outer.start_word.or(inner.start_word) {
                canonical.extend_from_slice(&[0x5c, if word { 0x62 } else { 0x42 }]);
            }
            canonical.extend_from_slice(payload);
            if let Some(word) = outer.end_word.or(inner.end_word) {
                canonical.extend_from_slice(&[0x5c, if word { 0x62 } else { 0x42 }]);
            }
            if outer.end_line || inner.end_line {
                canonical.push(0x24);
            }
            let canonical = JsString::from_code_units(canonical);
            let Some(plan) = Self::compile_with_outer_assertions_and_work(
                &canonical,
                ignore_case,
                multiline,
                charge,
            )?
            else {
                return Ok(None);
            };
            plan
        };
        if captures != 0 {
            let mut roles = vec![CaptureRole::Iteration; captures];
            roles.extend_from_slice(&plan.captures);
            plan.captures = roles.into_boxed_slice();
        }
        Ok(Some(plan))
    }

    /// Proves canonical assertion layers interleaved with complete wrappers.
    ///
    /// Requires validated v/vi input and normalized capture names. A single
    /// v-aware delimiter pass indexes group ends; framing then advances inward
    /// without rescanning nested bodies. Ordinary unquantified whole wrappers
    /// preserve lexical whole-range captures (22.2.2.7). Canonical predicates on
    /// each side form a conjunction (22.2.2.4), including false word conflicts.
    /// Every retained class/body is proved before lowering a false conjunction.
    /// Partial/quantified wrappers, root choices, inner assertions, references,
    /// counts and unknown leaves retain separate proofs.
    /// https://262.ecma-international.org/17.0/#sec-compileassertion
    pub fn compile_nested_assertion_wrappers_with_work<E>(
        source: &JsString,
        ignore_case: bool,
        multiline: bool,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, RegExpUnicodeSetCompileError<E>> {
        let units = source.code_units();
        charge(units.len().saturating_mul(32).saturating_add(128))
            .map_err(RegExpUnicodeSetCompileError::Work)?;
        if has_root_alternative(units) {
            return Ok(None);
        }
        let Some(closes) = single_v_group_closes(units) else {
            return Ok(None);
        };
        let (mut begin, mut end, mut captures, mut wrappers) =
            (0usize, units.len(), 0usize, 0usize);
        let (mut start_line, mut end_line) = (false, false);
        let (mut start_words, mut end_words) = (0u8, 0u8);
        loop {
            let Some(frame) = single_word_gate_frame(&units[begin..end]) else {
                return Ok(None);
            };
            start_line |= frame.start_line;
            end_line |= frame.end_line;
            if let Some(word) = frame.start_word {
                start_words |= if word { 1 } else { 2 };
            }
            if let Some(word) = frame.end_word {
                end_words |= if word { 1 } else { 2 };
            }
            let base = begin;
            begin = base + frame.body.start;
            end = base + frame.body.end;
            let tail = &units[begin..end];
            let width = if tail.starts_with(&[0x28, 0x3f, 0x3a]) {
                3
            } else if tail.first() == Some(&0x28) && tail.get(1) != Some(&0x3f) {
                1
            } else {
                break;
            };
            if closes.get(begin) != Some(&(end - 1)) {
                break;
            }
            wrappers += 1;
            captures += usize::from(width == 1);
            begin += width;
            end -= 1;
        }
        if wrappers == 0 || (!start_line && !end_line && start_words == 0 && end_words == 0) {
            return Ok(None);
        }
        let payload = &units[begin..end];
        let mut plan = if start_words == 3 || end_words == 3 {
            let body = JsString::from_code_units(payload.to_vec());
            let Some(mut plan) =
                Self::compile_with_work(&body, ignore_case, multiline, &mut charge)?
            else {
                return Ok(None);
            };
            let never = JsString::from(r"\b[\q{}]\B");
            let Some(never) = compile_string_class(&never, ignore_case, multiline, &mut charge)?
            else {
                return Ok(None);
            };
            plan.class = never;
            plan
        } else {
            let mut canonical = Vec::with_capacity(units.len());
            if start_line {
                canonical.push(0x5e);
            }
            if start_words != 0 {
                canonical.extend_from_slice(&[0x5c, if start_words == 1 { 0x62 } else { 0x42 }]);
            }
            canonical.extend_from_slice(payload);
            if end_words != 0 {
                canonical.extend_from_slice(&[0x5c, if end_words == 1 { 0x62 } else { 0x42 }]);
            }
            if end_line {
                canonical.push(0x24);
            }
            let canonical = JsString::from_code_units(canonical);
            let Some(plan) = Self::compile_with_outer_assertions_and_work(
                &canonical,
                ignore_case,
                multiline,
                charge,
            )?
            else {
                return Ok(None);
            };
            plan
        };
        if captures != 0 {
            let mut roles = vec![CaptureRole::Iteration; captures];
            roles.extend_from_slice(&plan.captures);
            plan.captures = roles.into_boxed_slice();
        }
        Ok(Some(plan))
    }

    /// Proves unconditional word/line predicates on a single-class spine.
    ///
    /// Requires validated v/vi input and normalized capture names. Assertions
    /// may occur in required empty siblings and complete wrappers. Positive
    /// empty-group counts repeat the same pure predicate at the same position;
    /// zero-minimum counts and assertion-bearing choices stay separate proofs.
    /// Only ^ before the class and $ after it are admitted. Every class/body is
    /// proved before contradictory word predicates select a never match.
    /// Captures retain original lexical slots and UTF-16 endpoints (22.2.2.3,
    /// 22.2.2.4, 22.2.2.7); storage and construction remain flat and source-sized.
    /// https://262.ecma-international.org/17.0/#sec-compileassertion
    pub fn compile_unconditional_assertions_with_work<E>(
        source: &JsString,
        ignore_case: bool,
        multiline: bool,
        charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, RegExpUnicodeSetCompileError<E>> {
        Self::compile_required_assertions(source, ignore_case, multiline, false, charge)
    }

    /// Proves mandatory assertion siblings with ^ after or $ before the class.
    ///
    /// Requires validated v/vi input and normalized capture names. At least one
    /// such line predicate must occur; all other mandatory word/line predicates
    /// and pure-empty capture terms follow the unconditional sibling proof.
    /// End predicates are checked for every class member before longest-member
    /// selection, so a rejected longer string can expose a shorter success.
    /// Optional assertions, assertion choices and class counts remain separate.
    /// https://262.ecma-international.org/17.0/#sec-compileassertion
    pub fn compile_reversed_line_assertions_with_work<E>(
        source: &JsString,
        ignore_case: bool,
        multiline: bool,
        charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, RegExpUnicodeSetCompileError<E>> {
        Self::compile_required_assertions(source, ignore_case, multiline, true, charge)
    }

    fn compile_required_assertions<E>(
        source: &JsString,
        ignore_case: bool,
        multiline: bool,
        reversed: bool,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, RegExpUnicodeSetCompileError<E>> {
        let units = source.code_units();
        charge(units.len().saturating_mul(160).saturating_add(512))
            .map_err(RegExpUnicodeSetCompileError::Work)?;
        if has_root_alternative(units) {
            return Ok(None);
        }
        // Stripping predicates is sound only after every assertion-bearing
        // ancestor proves mandatory participation. Preserve its group/capture
        // structure so the shared parser still proves the entire class spine.
        let mut shadow = Vec::with_capacity(units.len().saturating_add(4));
        shadow.extend_from_slice(&[0x28, 0x3f, 0x3a]);
        let mut frames = Vec::<UnconditionalAssertionFrame>::new();
        let (mut class_seen, mut asserted, mut start_line, mut end_line) =
            (false, false, false, false);
        let (mut start_end_line, mut end_start_line) = (false, false);
        let (mut start_words, mut end_words, mut at) = (0u8, 0u8, 0usize);
        while let Some(&unit) = units.get(at) {
            if unit == 0x5b {
                if class_seen {
                    return Ok(None);
                }
                let start = at;
                let mut depth = 1usize;
                at += 1;
                while let Some(&unit) = units.get(at) {
                    if unit == 0x5c {
                        if units.get(at + 1).is_none() {
                            return Ok(None);
                        }
                        at += 2;
                        continue;
                    }
                    if unit == 0x5b {
                        depth += 1;
                    } else if unit == 0x5d {
                        depth -= 1;
                    }
                    at += 1;
                    if depth == 0 {
                        break;
                    }
                }
                if depth != 0 {
                    return Ok(None);
                }
                shadow.extend_from_slice(&units[start..at]);
                class_seen = true;
                continue;
            }
            let word = if unit == 0x5c {
                match units.get(at + 1) {
                    Some(0x62) => Some(true),
                    Some(0x42) => Some(false),
                    _ => None,
                }
            } else {
                None
            };
            if unit == 0x5e || unit == 0x24 || word.is_some() {
                if unit == 0x5e {
                    if class_seen {
                        if !reversed {
                            return Ok(None);
                        }
                        end_start_line = true;
                    } else {
                        start_line = true;
                    }
                }
                if unit == 0x24 {
                    if !class_seen {
                        if !reversed {
                            return Ok(None);
                        }
                        start_end_line = true;
                    } else {
                        end_line = true;
                    }
                }
                if let Some(word) = word {
                    let bits = if word { 1 } else { 2 };
                    if class_seen {
                        end_words |= bits;
                    } else {
                        start_words |= bits;
                    }
                }
                asserted = true;
                if let Some(frame) = frames.last_mut() {
                    frame.asserted = true;
                }
                at += if word.is_some() { 2 } else { 1 };
                continue;
            }
            match unit {
                0x28 => {
                    frames.push(UnconditionalAssertionFrame {
                        asserted: false,
                        choice: false,
                    });
                    shadow.push(unit);
                    at += 1;
                }
                0x29 => {
                    let Some(frame) = frames.pop() else {
                        return Ok(None);
                    };
                    if frame.asserted && frame.choice {
                        return Ok(None);
                    }
                    shadow.push(unit);
                    at += 1;
                    if frame.asserted {
                        if matches!(units.get(at), Some(0x2a | 0x2b | 0x3f | 0x7b)) {
                            let Some((_, zero)) = super::empty_group_quantifier(units, at) else {
                                return Ok(None);
                            };
                            if zero {
                                return Ok(None);
                            }
                        }
                        if let Some(parent) = frames.last_mut() {
                            parent.asserted = true;
                        }
                    }
                }
                0x7c => {
                    let Some(frame) = frames.last_mut() else {
                        return Ok(None);
                    };
                    frame.choice = true;
                    shadow.push(unit);
                    at += 1;
                }
                0x5c => {
                    let Some(pair) = units.get(at..at + 2) else {
                        return Ok(None);
                    };
                    shadow.extend_from_slice(pair);
                    at += 2;
                }
                _ => {
                    shadow.push(unit);
                    at += 1;
                }
            }
        }
        if !asserted
            || !class_seen
            || !frames.is_empty()
            || (reversed && !start_end_line && !end_start_line)
        {
            return Ok(None);
        }
        shadow.push(0x29);
        let Some(parsed) = parse_body(&shadow, true, true, true, true, true, true) else {
            return Ok(None);
        };
        let class_units = &shadow[parsed.class];
        let class = if start_words == 3 || end_words == 3 {
            let original = JsString::from_code_units(class_units.to_vec());
            if compile_string_class(&original, ignore_case, multiline, &mut charge)?.is_none() {
                return Ok(None);
            }
            let never = JsString::from(r"\b[\q{}]\B");
            compile_string_class(&never, ignore_case, multiline, charge)?
        } else {
            let mut canonical = Vec::with_capacity(units.len());
            if start_line {
                canonical.push(0x5e);
            }
            if start_words != 0 {
                canonical.extend_from_slice(&[0x5c, if start_words == 1 { 0x62 } else { 0x42 }]);
            }
            canonical.extend_from_slice(class_units);
            if end_words != 0 {
                canonical.extend_from_slice(&[0x5c, if end_words == 1 { 0x62 } else { 0x42 }]);
            }
            if end_line {
                canonical.push(0x24);
            }
            compile_string_class(
                &JsString::from_code_units(canonical),
                ignore_case,
                multiline,
                charge,
            )?
        };
        let class = if reversed {
            class.and_then(|class| {
                class.with_endpoint_line_assertions(start_end_line, end_start_line)
            })
        } else {
            class
        };
        Ok(class.map(|class| Self {
            class,
            captures: parsed.captures.into_boxed_slice(),
        }))
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

struct SingleWordGateFrame {
    body: Range<usize>,
    start_line: bool,
    end_line: bool,
    start_word: Option<bool>,
    end_word: Option<bool>,
}
fn single_word_gate_frame(units: &[u16]) -> Option<SingleWordGateFrame> {
    let start_line = units.first() == Some(&0x5e);
    let end_line = units.last() == Some(&0x24) && !super::escaped(units, units.len() - 1);
    let mut begin = usize::from(start_line);
    let mut end = units.len() - usize::from(end_line);
    let start_word = match units.get(begin..end).and_then(|body| body.get(..2)) {
        Some([0x5c, word @ (0x62 | 0x42)]) => {
            begin += 2;
            Some(*word == 0x62)
        }
        _ => None,
    };
    let end_word = if end >= begin + 2
        && units[end - 2] == 0x5c
        && matches!(units[end - 1], 0x62 | 0x42)
        && !super::escaped(units, end - 2)
    {
        let word = units[end - 1] == 0x62;
        end -= 2;
        Some(word)
    } else {
        None
    };
    units.get(begin..end)?;
    Some(SingleWordGateFrame {
        body: begin..end,
        start_line,
        end_line,
        start_word,
        end_word,
    })
}

// Group delimiters in validated v source are indexed once. Nested classes and
// escape pairs remain opaque; ordinary-frame eligibility is checked separately.
fn single_v_group_closes(units: &[u16]) -> Option<Vec<usize>> {
    let mut closes = vec![0; units.len()];
    let mut stack = Vec::new();
    let (mut class, mut at) = (0usize, 0usize);
    while let Some(&unit) = units.get(at) {
        if unit == 0x5c {
            units.get(at + 1)?;
            at += 2;
            continue;
        }
        if unit == 0x5b {
            class += 1;
            at += 1;
            continue;
        }
        if class != 0 {
            if unit == 0x5d {
                class -= 1;
            }
            at += 1;
            continue;
        }
        match unit {
            0x28 => stack.push(at),
            0x29 => {
                let open = stack.pop()?;
                closes[open] = at;
            }
            _ => {}
        }
        at += 1;
    }
    (class == 0 && stack.is_empty()).then_some(closes)
}

struct UnconditionalAssertionFrame {
    asserted: bool,
    choice: bool,
}
