//! A root choice between an asserted single string class and pure empty terms.
use super::{
    JsString, RegExpUnicodeSetCompileError, RegExpUnicodeSingleStringClassCaptureMatcher,
    RegExpUnicodeStringClassBranchMatch, enclosed_v_group_body, has_root_alternative,
    single_word_gate_frame, unicode_start,
};
use std::ops::Range;

/// A proved two-way root choice with branch-specific mandatory assertions.
///
/// Requires validated v/vi input and normalized capture names. One alternative
/// contains one unquantified supported string class; the other contains only
/// pure empty terms and mandatory predicates. At least one predicate is required.
/// Source order resolves ties at the earliest Unicode candidate. Explicit branch
/// identity retains class captures even when the chosen class member is empty.
/// Complete wrappers use a separate constructor; choice counts remain separate
/// proofs (22.2.2.3–4).
/// https://262.ecma-international.org/17.0/#sec-compilepattern
#[derive(Clone, Debug)]
pub struct RegExpUnicodeAssertedStringClassAlternativeMatcher {
    class: RegExpUnicodeSingleStringClassCaptureMatcher,
    empty: RegExpUnicodeSingleStringClassCaptureMatcher,
    class_first: bool,
    captures: usize,
    enclosing_captures: usize,
}
impl RegExpUnicodeAssertedStringClassAlternativeMatcher {
    /// Proves both complete branches with source-sized construction work.
    pub fn compile_with_work<E>(
        source: &JsString,
        ignore_case: bool,
        multiline: bool,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, RegExpUnicodeSetCompileError<E>> {
        let units = source.code_units();
        charge(units.len().saturating_mul(192).saturating_add(512))
            .map_err(RegExpUnicodeSetCompileError::Work)?;
        let Some(bar) = root_bar(units) else {
            return Ok(None);
        };
        let left = JsString::from_code_units(units[..bar].to_vec());
        let right = JsString::from_code_units(units[bar + 1..].to_vec());
        let (class, empty_source, class_first) = if let Some(class) =
            single_branch(&left, ignore_case, multiline, &mut charge)?
        {
            (class, right, true)
        } else if let Some(class) = single_branch(&right, ignore_case, multiline, &mut charge)? {
            (class, left, false)
        } else {
            return Ok(None);
        };
        // A synthetic sole empty class proves that the entire other branch has
        // no consuming terms or actual class, without introducing capture slots.
        let mut empty_units = empty_source.code_units().to_vec();
        empty_units.extend_from_slice(&[0x5b, 0x5c, 0x71, 0x7b, 0x7d, 0x5d]);
        let Some(empty) = single_branch(
            &JsString::from_code_units(empty_units),
            ignore_case,
            multiline,
            &mut charge,
        )?
        else {
            return Ok(None);
        };
        if !class.class.has_outside_assertions() && !empty.class.has_outside_assertions() {
            return Ok(None);
        }
        let captures = class
            .capture_count()
            .checked_add(empty.capture_count())
            .ok_or(RegExpUnicodeSetCompileError::Capacity)?;
        Ok(Some(Self {
            class,
            empty,
            class_first,
            captures,
            enclosing_captures: 0,
        }))
    }
    /// Proves complete ordinary wrappers around an asserted root alternative.
    ///
    /// Requires validated v/vi input and normalized capture names. Complete
    /// capturing/noncapturing wrappers retain whole-match ranges in lexical
    /// prefix slots, including empty selected alternatives (22.2.2.7).
    /// Three flat scans keep nested v classes and escaped string data opaque.
    /// Outside predicates, partial wrappers and choice counts remain separate.
    /// https://262.ecma-international.org/17.0/#sec-compileatom
    pub fn compile_grouped_with_work<E>(
        source: &JsString,
        ignore_case: bool,
        multiline: bool,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, RegExpUnicodeSetCompileError<E>> {
        let units = source.code_units();
        charge(units.len().saturating_mul(8).saturating_add(64))
            .map_err(RegExpUnicodeSetCompileError::Work)?;
        let Some((body, enclosing_captures)) = enclosed_v_group_body(units) else {
            return Ok(None);
        };
        let Some(mut plan) = Self::compile_with_work(
            &JsString::from_code_units(units[body].to_vec()),
            ignore_case,
            multiline,
            charge,
        )?
        else {
            return Ok(None);
        };
        plan.captures = plan
            .captures
            .checked_add(enclosing_captures)
            .ok_or(RegExpUnicodeSetCompileError::Capacity)?;
        plan.enclosing_captures = enclosing_captures;
        Ok(Some(plan))
    }

    /// Proves common canonical ^/$ gates outside complete alternative wrappers.
    ///
    /// Requires validated v/vi input and normalized capture names. At least one
    /// common line predicate and one complete ordinary wrapper are required.
    /// The common predicates are conjoined with each original alternative at
    /// its own start/end; every class member remains eligible for continuation
    /// checks before longest-member selection (22.2.2.3–4, 22.2.2.7).
    /// Original root choices decline before extracting any common gate. Outside
    /// word predicates, partial wrappers and choice counts remain separate.
    /// https://262.ecma-international.org/17.0/#sec-compileassertion
    pub fn compile_surrounding_line_assertions_with_work<E>(
        source: &JsString,
        ignore_case: bool,
        multiline: bool,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, RegExpUnicodeSetCompileError<E>> {
        let units = source.code_units();
        charge(units.len().saturating_mul(16).saturating_add(128))
            .map_err(RegExpUnicodeSetCompileError::Work)?;
        if has_root_alternative(units) {
            return Ok(None);
        }
        let Some(frame) = single_word_gate_frame(units) else {
            return Ok(None);
        };
        if (!frame.start_line && !frame.end_line)
            || frame.start_word.is_some()
            || frame.end_word.is_some()
        {
            return Ok(None);
        }
        let inner = &units[frame.body];
        let Some((body, enclosing_captures)) = enclosed_v_group_body(inner) else {
            return Ok(None);
        };
        let body = &inner[body];
        let Some(bar) = root_bar(body) else {
            return Ok(None);
        };
        // Only predicates are duplicated; every original term and capture keeps
        // its branch and lexical order. Empty-branch predicates share one point.
        let mut decorated = Vec::with_capacity(units.len().saturating_add(4));
        for (index, branch) in [&body[..bar], &body[bar + 1..]].into_iter().enumerate() {
            if index != 0 {
                decorated.push(0x7c);
            }
            if frame.start_line {
                decorated.push(0x5e);
            }
            decorated.extend_from_slice(branch);
            if frame.end_line {
                decorated.push(0x24);
            }
        }
        let Some(mut plan) = Self::compile_with_work(
            &JsString::from_code_units(decorated),
            ignore_case,
            multiline,
            charge,
        )?
        else {
            return Ok(None);
        };
        plan.captures = plan
            .captures
            .checked_add(enclosing_captures)
            .ok_or(RegExpUnicodeSetCompileError::Capacity)?;
        plan.enclosing_captures = enclosing_captures;
        Ok(Some(plan))
    }

    /// Counts original lexical captures in wrappers and both alternatives.
    pub fn capture_count(&self) -> usize {
        self.captures
    }
    /// Projects whole wrappers and the selected branch; unselected slots are absent.
    pub fn capture_range(
        &self,
        index: usize,
        found: &RegExpUnicodeStringClassBranchMatch,
    ) -> Option<Range<usize>> {
        if index >= self.captures {
            return None;
        }
        if index < self.enclosing_captures {
            return Some(found.range.clone());
        }
        let index = index - self.enclosing_captures;
        let first = if self.class_first {
            self.class.capture_count()
        } else {
            self.empty.capture_count()
        };
        let class_selected = if self.class_first {
            index < first
        } else {
            index >= first
        };
        if class_selected != found.class_selected {
            return None;
        }
        let local = if index < first { index } else { index - first };
        if class_selected {
            self.class.capture_range(local, &found.range)
        } else {
            self.empty.capture_range(local, &found.range)
        }
    }
    /// Finds each branch's earliest success, then applies ordered-choice ties.
    ///
    /// Each immutable search charges before inspection and shares initial Unicode
    /// normalization. A first-branch success at that position needs no other
    /// search. An empty class member stays distinct from the empty alternative.
    pub fn find_with_captures_and_work<E>(
        &self,
        input: &JsString,
        start: usize,
        sticky: bool,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<RegExpUnicodeStringClassBranchMatch>, E> {
        charge(4)?;
        let Some(start) = unicode_start(input, start) else {
            return Ok(None);
        };
        let (class, empty) = if self.class_first {
            let class =
                self.class
                    .find_with_captures_and_work(input, start, sticky, &mut charge)?;
            if let Some(range) = class.as_ref().filter(|range| range.start == start) {
                return Ok(Some(RegExpUnicodeStringClassBranchMatch {
                    range: range.clone(),
                    class_selected: true,
                }));
            }
            let empty = self
                .empty
                .find_with_captures_and_work(input, start, sticky, charge)?;
            (class, empty)
        } else {
            let empty =
                self.empty
                    .find_with_captures_and_work(input, start, sticky, &mut charge)?;
            if let Some(range) = empty.as_ref().filter(|range| range.start == start) {
                return Ok(Some(RegExpUnicodeStringClassBranchMatch {
                    range: range.clone(),
                    class_selected: false,
                }));
            }
            let class = self
                .class
                .find_with_captures_and_work(input, start, sticky, charge)?;
            (class, empty)
        };
        let chosen = match (class, empty) {
            (Some(class), Some(empty)) => {
                if class.start < empty.start || (class.start == empty.start && self.class_first) {
                    Some((class, true))
                } else {
                    Some((empty, false))
                }
            }
            (Some(class), None) => Some((class, true)),
            (None, Some(empty)) => Some((empty, false)),
            (None, None) => None,
        };
        Ok(chosen.map(
            |(range, class_selected)| RegExpUnicodeStringClassBranchMatch {
                range,
                class_selected,
            },
        ))
    }
}
fn single_branch<E>(
    source: &JsString,
    ignore_case: bool,
    multiline: bool,
    mut charge: impl FnMut(usize) -> Result<(), E>,
) -> Result<Option<RegExpUnicodeSingleStringClassCaptureMatcher>, RegExpUnicodeSetCompileError<E>> {
    if let Some(plan) = RegExpUnicodeSingleStringClassCaptureMatcher::compile_with_work(
        source,
        ignore_case,
        multiline,
        &mut charge,
    )? {
        return Ok(Some(plan));
    }
    if let Some(plan) =
        RegExpUnicodeSingleStringClassCaptureMatcher::compile_unconditional_assertions_with_work(
            source,
            ignore_case,
            multiline,
            &mut charge,
        )?
    {
        return Ok(Some(plan));
    }
    RegExpUnicodeSingleStringClassCaptureMatcher::compile_reversed_line_assertions_with_work(
        source,
        ignore_case,
        multiline,
        charge,
    )
}
fn root_bar(units: &[u16]) -> Option<usize> {
    let (mut depth, mut class, mut at) = (0usize, 0usize, 0usize);
    let mut bar = None;
    while let Some(&unit) = units.get(at) {
        if unit == 0x5c {
            units.get(at + 1)?;
            at += 2;
            continue;
        }
        if class != 0 {
            if unit == 0x5b {
                class = class.checked_add(1)?;
            } else if unit == 0x5d {
                class -= 1;
            }
        } else {
            match unit {
                0x5b => class = 1,
                0x28 => depth = depth.checked_add(1)?,
                0x29 => depth = depth.checked_sub(1)?,
                0x7c if depth == 0 && bar.is_some() => return None,
                0x7c if depth == 0 => bar = Some(at),
                _ => {}
            }
        }
        at += 1;
    }
    (depth == 0 && class == 0).then_some(bar).flatten()
}
