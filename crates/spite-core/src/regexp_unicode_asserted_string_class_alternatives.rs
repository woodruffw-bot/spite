//! A root choice between an asserted single string class and pure empty terms.
use super::{
    JsString, RegExpUnicodeSetCompileError, RegExpUnicodeSingleStringClassCaptureMatcher,
    RegExpUnicodeStringClassBranchMatch, enclosed_v_group_body, has_root_alternative,
    single_v_group_closes, single_word_gate_frame, unicode_start,
};
use std::ops::Range;

#[path = "regexp_unicode_surrounding_asserted_string_class_alternatives.rs"]
mod surrounding;
pub use surrounding::RegExpUnicodeSurroundingAssertedStringClassAlternativeMatcher;

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
    /// Proves exact-one counts in the class branch of a root alternative.
    ///
    /// Requires validated v/vi input and normalized capture names. The sole
    /// class atom and its ordinary ancestors may use {1} or {1,1}, with leading
    /// zeros and lazy forms. RepeatMatcher executes their body once even when
    /// empty; removing only those count tokens preserves lexical slots and
    /// branch-local predicates in this reference-free proof. The entire original
    /// class and both branches still require proof, including impossible gates.
    /// Original root scope is required; enclosing choices use separate proofs
    /// (22.2.2.3, RepeatMatcher).
    /// https://262.ecma-international.org/17.0/#sec-repeatmatcher
    pub fn compile_exact_single_counts_with_work<E>(
        source: &JsString,
        ignore_case: bool,
        multiline: bool,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, RegExpUnicodeSetCompileError<E>> {
        let units = source.code_units();
        charge(units.len().saturating_mul(24).saturating_add(128))
            .map_err(RegExpUnicodeSetCompileError::Work)?;
        if root_bar(units).is_none() {
            return Ok(None);
        }
        let Some(normalized) = surrounding::exact_single_count_source(units) else {
            return Ok(None);
        };
        Self::compile_with_work(
            &JsString::from_code_units(normalized),
            ignore_case,
            multiline,
            charge,
        )
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

    /// Proves common word predicates outside complete alternative wrappers.
    ///
    /// Requires validated v/vi input and normalized capture names. At least one
    /// common word predicate is required; canonical outside ^/$ may accompany it.
    /// Every original term/capture keeps its branch. Common start/end words may
    /// differ for a consuming class while sharing one point for the empty arm.
    /// Contradictory local conjunctions still require full class/body proof.
    /// Root choices decline before extracting predicates (22.2.2.3–4).
    /// https://262.ecma-international.org/17.0/#sec-compileassertion
    pub fn compile_surrounding_word_assertions_with_work<E>(
        source: &JsString,
        ignore_case: bool,
        multiline: bool,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, RegExpUnicodeSetCompileError<E>> {
        let units = source.code_units();
        charge(units.len().saturating_mul(24).saturating_add(192))
            .map_err(RegExpUnicodeSetCompileError::Work)?;
        if has_root_alternative(units) {
            return Ok(None);
        }
        let Some(frame) = single_word_gate_frame(units) else {
            return Ok(None);
        };
        if frame.start_word.is_none() && frame.end_word.is_none() {
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
        let mut decorated = Vec::with_capacity(units.len().saturating_add(8));
        for (index, branch) in [&body[..bar], &body[bar + 1..]].into_iter().enumerate() {
            if index != 0 {
                decorated.push(0x7c);
            }
            if frame.start_line {
                decorated.push(0x5e);
            }
            if let Some(word) = frame.start_word {
                decorated.extend_from_slice(&[0x5c, if word { 0x62 } else { 0x42 }]);
            }
            decorated.extend_from_slice(branch);
            if let Some(word) = frame.end_word {
                decorated.extend_from_slice(&[0x5c, if word { 0x62 } else { 0x42 }]);
            }
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

    /// Proves canonical assertion layers around complete alternative wrappers.
    ///
    /// Requires validated v/vi input and normalized capture names. One flat
    /// delimiter pass bounds all subsequent frame recovery. Original root
    /// alternatives end recovery before their local predicates are inspected.
    /// Common predicates are conjoined on each original branch at its actual
    /// endpoints; even false conjunctions require full leaf/body proof.
    /// Complete wrappers preserve lexical whole-range slots (22.2.2.3–4, .7).
    /// https://262.ecma-international.org/17.0/#sec-compileassertion
    pub fn compile_nested_assertion_wrappers_with_work<E>(
        source: &JsString,
        ignore_case: bool,
        multiline: bool,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, RegExpUnicodeSetCompileError<E>> {
        let units = source.code_units();
        charge(units.len().saturating_mul(40).saturating_add(192))
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
            // Root predicates belong to their own alternatives. The delimiter
            // map identifies that payload without repeatedly scanning it.
            let tail = &units[begin..end];
            let Some(frame) = single_word_gate_frame(tail) else {
                return Ok(None);
            };
            let candidate_begin = begin + frame.body.start;
            let candidate_end = begin + frame.body.end;
            let candidate = &units[candidate_begin..candidate_end];
            let width = if candidate.starts_with(&[0x28, 0x3f, 0x3a]) {
                3
            } else if candidate.first() == Some(&0x28) && candidate.get(1) != Some(&0x3f) {
                1
            } else {
                break;
            };
            if closes.get(candidate_begin) != Some(&(candidate_end - 1)) {
                break;
            }
            start_line |= frame.start_line;
            end_line |= frame.end_line;
            if let Some(word) = frame.start_word {
                start_words |= if word { 1 } else { 2 };
            }
            if let Some(word) = frame.end_word {
                end_words |= if word { 1 } else { 2 };
            }
            wrappers += 1;
            captures += usize::from(width == 1);
            begin = candidate_begin + width;
            end = candidate_end - 1;
        }
        if wrappers == 0 || (!start_line && !end_line && start_words == 0 && end_words == 0) {
            return Ok(None);
        }
        let payload = &units[begin..end];
        let Some(bar) = root_bar(payload) else {
            return Ok(None);
        };
        let mut decorated = Vec::with_capacity(units.len().saturating_add(12));
        for (index, branch) in [&payload[..bar], &payload[bar + 1..]]
            .into_iter()
            .enumerate()
        {
            if index != 0 {
                decorated.push(0x7c);
            }
            if start_line {
                decorated.push(0x5e);
            }
            if start_words & 1 != 0 {
                decorated.extend_from_slice(&[0x5c, 0x62]);
            }
            if start_words & 2 != 0 {
                decorated.extend_from_slice(&[0x5c, 0x42]);
            }
            decorated.extend_from_slice(branch);
            if end_words & 1 != 0 {
                decorated.extend_from_slice(&[0x5c, 0x62]);
            }
            if end_words & 2 != 0 {
                decorated.extend_from_slice(&[0x5c, 0x42]);
            }
            if end_line {
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
            .checked_add(captures)
            .ok_or(RegExpUnicodeSetCompileError::Capacity)?;
        plan.enclosing_captures = captures;
        Ok(Some(plan))
    }

    /// Proves common $ before and/or ^ after complete alternative wrappers.
    ///
    /// Requires validated v/vi input and normalized capture names. Optional
    /// common word predicates share each actual boundary. Every original branch
    /// and class leaf passes the existing full proof, including false predicates.
    /// Multiline $...^ may consume a newline between different endpoints; shorter
    /// eligible members remain available (22.2.2.3–4, 22.2.2.7).
    /// https://262.ecma-international.org/17.0/#sec-compileassertion
    pub fn compile_reversed_surrounding_line_assertions_with_work<E>(
        source: &JsString,
        ignore_case: bool,
        multiline: bool,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, RegExpUnicodeSetCompileError<E>> {
        let units = source.code_units();
        charge(units.len().saturating_mul(32).saturating_add(160))
            .map_err(RegExpUnicodeSetCompileError::Work)?;
        if has_root_alternative(units) {
            return Ok(None);
        }
        let start_end_line = units.first() == Some(&0x24);
        let end_start_line =
            units.last() == Some(&0x5e) && !super::super::escaped(units, units.len() - 1);
        if !start_end_line && !end_start_line {
            return Ok(None);
        }
        let mut begin = usize::from(start_end_line);
        let mut end = units.len() - usize::from(end_start_line);
        let start_word = match units.get(begin..end).and_then(|body| body.get(..2)) {
            Some([0x5c, word @ (0x62 | 0x42)]) => {
                begin += 2;
                Some(*word)
            }
            _ => None,
        };
        let end_word = if end >= begin + 2
            && units[end - 2] == 0x5c
            && matches!(units[end - 1], 0x62 | 0x42)
            && !super::super::escaped(units, end - 2)
        {
            let word = units[end - 1];
            end -= 2;
            Some(word)
        } else {
            None
        };
        let Some(inner) = units.get(begin..end) else {
            return Ok(None);
        };
        let Some((body, enclosing_captures)) = enclosed_v_group_body(inner) else {
            return Ok(None);
        };
        let body = &inner[body];
        let Some(bar) = root_bar(body) else {
            return Ok(None);
        };
        let mut decorated = Vec::with_capacity(units.len().saturating_add(8));
        for (index, branch) in [&body[..bar], &body[bar + 1..]].into_iter().enumerate() {
            if index != 0 {
                decorated.push(0x7c);
            }
            if start_end_line {
                decorated.push(0x24);
            }
            if let Some(word) = start_word {
                decorated.extend_from_slice(&[0x5c, word]);
            }
            decorated.extend_from_slice(branch);
            if let Some(word) = end_word {
                decorated.extend_from_slice(&[0x5c, word]);
            }
            if end_start_line {
                decorated.push(0x5e);
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

    /// Proves canonical and reversed line/word layers around full wrappers.
    ///
    /// Requires validated v/vi input and normalized capture names. At least one
    /// common $ before or ^ after a wrapper is required. A flat delimiter map
    /// bounds inward recovery; local root predicates keep their branch scope.
    /// All original leaves/terms prove before false same-point conjunctions.
    /// Selected consuming endpoints may differ (22.2.2.3–4, 22.2.2.7).
    /// https://262.ecma-international.org/17.0/#sec-compileassertion
    pub fn compile_nested_endpoint_assertion_wrappers_with_work<E>(
        source: &JsString,
        ignore_case: bool,
        multiline: bool,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, RegExpUnicodeSetCompileError<E>> {
        let units = source.code_units();
        charge(units.len().saturating_mul(48).saturating_add(192))
            .map_err(RegExpUnicodeSetCompileError::Work)?;
        if has_root_alternative(units) {
            return Ok(None);
        }
        let Some(closes) = single_v_group_closes(units) else {
            return Ok(None);
        };
        let (mut begin, mut end, mut captures, mut wrappers) =
            (0usize, units.len(), 0usize, 0usize);
        let (mut start_line, mut end_line, mut start_end_line, mut end_start_line) =
            (false, false, false, false);
        let (mut start_words, mut end_words) = (0u8, 0u8);
        loop {
            let Some(frame) = endpoint_gate_frame(&units[begin..end]) else {
                return Ok(None);
            };
            let candidate_begin = begin + frame.body.start;
            let candidate_end = begin + frame.body.end;
            let candidate = &units[candidate_begin..candidate_end];
            let width = if candidate.starts_with(&[0x28, 0x3f, 0x3a]) {
                3
            } else if candidate.first() == Some(&0x28) && candidate.get(1) != Some(&0x3f) {
                1
            } else {
                break;
            };
            if closes.get(candidate_begin) != Some(&(candidate_end - 1)) {
                break;
            }
            start_line |= frame.start_line == Some(0x5e);
            start_end_line |= frame.start_line == Some(0x24);
            end_line |= frame.end_line == Some(0x24);
            end_start_line |= frame.end_line == Some(0x5e);
            if let Some(word) = frame.start_word {
                start_words |= if word == 0x62 { 1 } else { 2 };
            }
            if let Some(word) = frame.end_word {
                end_words |= if word == 0x62 { 1 } else { 2 };
            }
            wrappers += 1;
            captures += usize::from(width == 1);
            begin = candidate_begin + width;
            end = candidate_end - 1;
        }
        if wrappers == 0 || (!start_end_line && !end_start_line) {
            return Ok(None);
        }
        let payload = &units[begin..end];
        let Some(bar) = root_bar(payload) else {
            return Ok(None);
        };
        let mut decorated = Vec::with_capacity(units.len().saturating_add(16));
        for (index, branch) in [&payload[..bar], &payload[bar + 1..]]
            .into_iter()
            .enumerate()
        {
            if index != 0 {
                decorated.push(0x7c);
            }
            if start_line {
                decorated.push(0x5e);
            }
            if start_end_line {
                decorated.push(0x24);
            }
            if start_words & 1 != 0 {
                decorated.extend_from_slice(&[0x5c, 0x62]);
            }
            if start_words & 2 != 0 {
                decorated.extend_from_slice(&[0x5c, 0x42]);
            }
            decorated.extend_from_slice(branch);
            if end_words & 1 != 0 {
                decorated.extend_from_slice(&[0x5c, 0x62]);
            }
            if end_words & 2 != 0 {
                decorated.extend_from_slice(&[0x5c, 0x42]);
            }
            if end_start_line {
                decorated.push(0x5e);
            }
            if end_line {
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
            .checked_add(captures)
            .ok_or(RegExpUnicodeSetCompileError::Capacity)?;
        plan.enclosing_captures = captures;
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

struct EndpointGateFrame {
    body: Range<usize>,
    start_line: Option<u16>,
    end_line: Option<u16>,
    start_word: Option<u16>,
    end_word: Option<u16>,
}
fn endpoint_gate_frame(units: &[u16]) -> Option<EndpointGateFrame> {
    let mut begin = 0;
    let mut end = units.len();
    let start_line = units.first().copied().filter(|u| matches!(u, 0x5e | 0x24));
    begin += usize::from(start_line.is_some());
    let end_line = if end > begin
        && matches!(units[end - 1], 0x5e | 0x24)
        && !super::super::escaped(units, end - 1)
    {
        end -= 1;
        Some(units[end])
    } else {
        None
    };
    let start_word = match units.get(begin..end).and_then(|body| body.get(..2)) {
        Some([0x5c, word @ (0x62 | 0x42)]) => {
            begin += 2;
            Some(*word)
        }
        _ => None,
    };
    let end_word = if end >= begin + 2
        && units[end - 2] == 0x5c
        && matches!(units[end - 1], 0x62 | 0x42)
        && !super::super::escaped(units, end - 2)
    {
        let word = units[end - 1];
        end -= 2;
        Some(word)
    } else {
        None
    };
    units.get(begin..end)?;
    Some(EndpointGateFrame {
        body: begin..end,
        start_line,
        end_line,
        start_word,
        end_word,
    })
}
