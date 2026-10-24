//! Simple string-class repetition with a proved pure-empty continuation.
use super::super::{CaptureRole, projected_selected_capture_range};
use super::{
    AlternativeScope, JsString, RegExpUnicodeSetCompileError,
    RegExpUnicodeSingleStringClassCaptureMatcher, RegExpUnicodeStringSetMatcher,
    asserted_alternatives::exact_single_count_source, compile_string_alternative,
    has_root_alternative, unicode_start,
};
use std::ops::Range;

/// One proved string-class atom or class/empty group under *, + or ?.
///
/// Requires validated v/vi input and normalized capture names. Every body and
/// outside term is proved before matching; the continuation is pure empty and
/// cannot fail. Source-priority longest members therefore need no backtracking
/// history. Optional empty iterations are rejected, including an empty-first
/// branch that must retry the consuming class. Mandatory + retains its required
/// empty captures. All capture state is the final accepted iteration plus the
/// whole range (22.2.2.3, RepeatMatcher).
/// https://262.ecma-international.org/17.0/#sec-repeatmatcher
#[derive(Clone, Debug)]
pub struct RegExpUnicodeRepeatedStringClassCaptureMatcher {
    class: RegExpUnicodeStringSetMatcher,
    inner: Box<[CaptureRole]>,
    outside: Box<[CaptureRole]>,
    before: usize,
    captures: usize,
    empty_alternative: bool,
    empty_first: bool,
    minimum_one: bool,
    maximum_one: bool,
    greedy: bool,
}

/// Whole repetition and final accepted iteration in original UTF-16 positions.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RegExpUnicodeRepeatedStringClassMatch {
    /// Complete match, including a zero-iteration empty result.
    pub range: Range<usize>,
    /// Final accepted body; absent when no iteration executes.
    pub last_iteration: Option<Range<usize>>,
    /// True when the final iteration selected the class, including its empty member.
    pub class_selected: bool,
}
impl RegExpUnicodeRepeatedStringClassCaptureMatcher {
    /// Proves one simple counted class atom/group and all surrounding empty terms.
    pub fn compile_with_work<E>(
        source: &JsString,
        ignore_case: bool,
        multiline: bool,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, RegExpUnicodeSetCompileError<E>> {
        let units = source.code_units();
        charge(units.len().saturating_mul(96).saturating_add(256))
            .map_err(RegExpUnicodeSetCompileError::Work)?;
        if has_root_alternative(units) {
            return Ok(None);
        }
        let Some(hole) = repeated_hole(units) else {
            return Ok(None);
        };
        let inner_source = normalize_ones(&units[hole.atom.clone()]);
        let (class, inner, empty_alternative, empty_first) = if let Some(plan) =
            RegExpUnicodeSingleStringClassCaptureMatcher::compile_with_work(
                &inner_source,
                ignore_case,
                multiline,
                &mut charge,
            )? {
            (plan.class, plan.captures, false, false)
        } else {
            let mut plan = compile_string_alternative(
                &inner_source,
                ignore_case,
                multiline,
                AlternativeScope::Grouped,
                false,
                &mut charge,
            )?;
            if plan.is_none() {
                plan = compile_string_alternative(
                    &inner_source,
                    ignore_case,
                    multiline,
                    AlternativeScope::Grouped,
                    true,
                    &mut charge,
                )?;
            }
            let Some(plan) = plan else {
                return Ok(None);
            };
            (plan.class, plan.captures, true, plan.empty_first)
        };
        let mut shadow = Vec::with_capacity(units.len().saturating_add(6));
        shadow.extend_from_slice(&units[..hole.atom.start]);
        shadow.extend_from_slice(&[0x5b, 0x5c, 0x71, 0x7b, 0x7d, 0x5d]);
        shadow.extend_from_slice(&units[hole.count_end..]);
        let shadow = normalize_ones(&shadow);
        let Some(outside) = RegExpUnicodeSingleStringClassCaptureMatcher::compile_with_work(
            &shadow,
            ignore_case,
            multiline,
            &mut charge,
        )?
        else {
            return Ok(None);
        };
        if hole.before > outside.capture_count() {
            return Ok(None);
        }
        let captures = outside
            .capture_count()
            .checked_add(inner.len())
            .ok_or(RegExpUnicodeSetCompileError::Capacity)?;
        Ok(Some(Self {
            class,
            inner,
            outside: outside.captures,
            before: hole.before,
            captures,
            empty_alternative,
            empty_first,
            minimum_one: hole.minimum_one,
            maximum_one: hole.maximum_one,
            greedy: hole.greedy,
        }))
    }
    /// Counts every original lexical capture exactly once.
    pub fn capture_count(&self) -> usize {
        self.captures
    }
    /// Projects enclosing/point slots on the whole match and body slots on its last iteration.
    pub fn capture_range(
        &self,
        index: usize,
        found: &RegExpUnicodeRepeatedStringClassMatch,
    ) -> Option<Range<usize>> {
        if index >= self.captures {
            return None;
        }
        if index < self.before {
            return projected_selected_capture_range(&self.outside, index, &found.range, true);
        }
        if index < self.before + self.inner.len() {
            return projected_selected_capture_range(
                &self.inner,
                index - self.before,
                found.last_iteration.as_ref()?,
                found.class_selected,
            );
        }
        projected_selected_capture_range(
            &self.outside,
            index - self.inner.len(),
            &found.range,
            true,
        )
    }
    /// Executes required bodies and consuming optional bodies without capture history.
    pub fn find_with_captures_and_work<E>(
        &self,
        input: &JsString,
        requested: usize,
        sticky: bool,
        charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<RegExpUnicodeRepeatedStringClassMatch>, E> {
        self.find_with_maximum(input, requested, sticky, None, charge)
    }
    fn find_with_maximum<E>(
        &self,
        input: &JsString,
        requested: usize,
        sticky: bool,
        maximum: Option<usize>,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<RegExpUnicodeRepeatedStringClassMatch>, E> {
        charge(4)?;
        let Some(mut start) = unicode_start(input, requested) else {
            return Ok(None);
        };
        let mut at = start;
        let mut last_iteration = None;
        let mut class_selected = false;
        if self.minimum_one {
            if self.empty_alternative && self.empty_first {
                last_iteration = Some(at..at);
            } else {
                let mut found = self.class_prefix(input, at, &mut charge)?;
                if found.is_none() && !self.empty_alternative && !sticky {
                    charge(
                        (input.len() - at)
                            .saturating_add(1)
                            .saturating_mul(self.class.search_passes(false).saturating_add(4)),
                    )?;
                    found = self.class.find(input, at, false);
                }
                if let Some(range) = found {
                    start = range.start;
                    at = range.end;
                    last_iteration = Some(range);
                    class_selected = true;
                } else if self.empty_alternative {
                    last_iteration = Some(at..at);
                } else {
                    return Ok(None);
                }
            }
        }
        if self.greedy {
            let mut remaining = maximum.map(|limit| limit - usize::from(self.minimum_one));
            loop {
                if remaining == Some(0) {
                    break;
                }
                let Some(range) = self.class_prefix(input, at, &mut charge)? else {
                    break;
                };
                debug_assert_eq!(range.start, at);
                // RepeatMatcher's min=0 continuation rejects EPS before any
                // capture state commits. E-first retries C after that rejection.
                if range.end == at {
                    break;
                }
                at = range.end;
                last_iteration = Some(range);
                class_selected = true;
                remaining = remaining.map(|left| left - 1);
                if self.maximum_one {
                    break;
                }
            }
        }
        Ok(Some(RegExpUnicodeRepeatedStringClassMatch {
            range: start..at,
            last_iteration,
            class_selected,
        }))
    }
    fn class_prefix<E>(
        &self,
        input: &JsString,
        at: usize,
        charge: &mut impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Range<usize>>, E> {
        let inspected = (input.len() - at)
            .min(self.class.max_match_code_units())
            .saturating_add(1);
        charge(
            inspected
                .saturating_mul(self.class.search_passes(true))
                .saturating_add(4),
        )?;
        Ok(self.class.find(input, at, true))
    }
}
fn normalize_ones(units: &[u16]) -> JsString {
    JsString::from_code_units(exact_single_count_source(units).unwrap_or_else(|| units.to_vec()))
}
struct Hole {
    atom: Range<usize>,
    count_end: usize,
    before: usize,
    minimum_one: bool,
    maximum_one: bool,
    greedy: bool,
}
struct Frame {
    start: usize,
    before: usize,
    class: bool,
}
fn simple_count(units: &[u16], at: usize) -> Option<(usize, bool, bool, bool)> {
    let (minimum_one, maximum_one) = match units.get(at)? {
        0x2a => (false, false),
        0x2b => (true, false),
        0x3f => (false, true),
        _ => return None,
    };
    let lazy = units.get(at + 1) == Some(&0x3f);
    Some((at + 1 + usize::from(lazy), minimum_one, maximum_one, !lazy))
}
fn class_end(units: &[u16], start: usize) -> Option<usize> {
    let (mut at, mut depth) = (start + 1, 1usize);
    while let Some(&unit) = units.get(at) {
        if unit == 0x5c {
            units.get(at + 1)?;
            at += 2;
            continue;
        }
        if unit == 0x5b {
            depth = depth.checked_add(1)?;
        }
        if unit == 0x5d {
            depth -= 1;
        }
        at += 1;
        if depth == 0 {
            return Some(at);
        }
    }
    None
}
fn repeated_hole(units: &[u16]) -> Option<Hole> {
    counted_hole(units, simple_count)
}
fn counted_hole(
    units: &[u16],
    mut count: impl FnMut(&[u16], usize) -> Option<(usize, bool, bool, bool)>,
) -> Option<Hole> {
    let (mut at, mut captures, mut class_seen) = (0usize, 0usize, false);
    let mut frames = Vec::<Frame>::new();
    let mut hole = None;
    while let Some(&unit) = units.get(at) {
        match unit {
            0x5c => {
                units.get(at + 1)?;
                at += 2;
            }
            0x28 => {
                let before = captures;
                let header = if units.get(at + 1) == Some(&0x3f) {
                    if units.get(at + 2) != Some(&0x3a) {
                        return None;
                    }
                    3
                } else {
                    captures = captures.checked_add(1)?;
                    1
                };
                frames.push(Frame {
                    start: at,
                    before,
                    class: false,
                });
                at += header;
            }
            0x5b => {
                if class_seen {
                    return None;
                }
                class_seen = true;
                for frame in &mut frames {
                    frame.class = true;
                }
                let begin = at;
                at = class_end(units, at)?;
                if let Some((end, minimum_one, maximum_one, greedy)) = count(units, at) {
                    if hole.is_some() {
                        return None;
                    }
                    hole = Some(Hole {
                        atom: begin..at,
                        count_end: end,
                        before: captures,
                        minimum_one,
                        maximum_one,
                        greedy,
                    });
                    at = end;
                }
            }
            0x29 => {
                let frame = frames.pop()?;
                at += 1;
                if frame.class {
                    if let Some((end, minimum_one, maximum_one, greedy)) = count(units, at) {
                        if hole.is_some() {
                            return None;
                        }
                        hole = Some(Hole {
                            atom: frame.start..at,
                            count_end: end,
                            before: frame.before,
                            minimum_one,
                            maximum_one,
                            greedy,
                        });
                        at = end;
                    }
                }
            }
            _ => at += 1,
        }
    }
    if !frames.is_empty() || !class_seen {
        return None;
    }
    hole
}

/// A braced string-class repetition whose minimum is zero or one.
///
/// Requires validated v/vi input with normalized capture names.
/// The complete body and pure-empty outside continuation use the same proof as
/// simple repetition. A larger minimum needs backtracking between class members
/// and remains a separate proof. Exact-one counts use the existing single-class
/// and nullable-alternative constructors (22.2.2.3, RepeatMatcher).
/// https://262.ecma-international.org/17.0/#sec-repeatmatcher
#[derive(Clone, Debug)]
pub struct RegExpUnicodeBoundedStringClassCaptureMatcher {
    plan: RegExpUnicodeRepeatedStringClassCaptureMatcher,
    maximum: Option<usize>,
}
impl RegExpUnicodeBoundedStringClassCaptureMatcher {
    /// Proves one braced count with a zero/one minimum, except exact-one counts.
    pub fn compile_with_work<E>(
        source: &JsString,
        ignore_case: bool,
        multiline: bool,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, RegExpUnicodeSetCompileError<E>> {
        let units = source.code_units();
        charge(units.len().saturating_mul(24).saturating_add(128))
            .map_err(RegExpUnicodeSetCompileError::Work)?;
        let mut maximum = None;
        let Some(hole) = counted_hole(units, |units, at| {
            let (end, minimum_one, limit, greedy) = bounded_count(units, at)?;
            maximum = limit;
            Some((end, minimum_one, limit == Some(1), greedy))
        }) else {
            return Ok(None);
        };
        let mut simple = Vec::with_capacity(units.len());
        simple.extend_from_slice(&units[..hole.atom.end]);
        simple.push(if hole.minimum_one { 0x2b } else { 0x2a });
        if !hole.greedy {
            simple.push(0x3f);
        }
        simple.extend_from_slice(&units[hole.count_end..]);
        let Some(plan) = RegExpUnicodeRepeatedStringClassCaptureMatcher::compile_with_work(
            &JsString::from_code_units(simple),
            ignore_case,
            multiline,
            &mut charge,
        )?
        else {
            return Ok(None);
        };
        Ok(Some(Self { plan, maximum }))
    }
    /// Counts every original lexical capture, including skipped body slots.
    pub fn capture_count(&self) -> usize {
        self.plan.capture_count()
    }
    /// Projects body slots on the last accepted iteration and outside slots on the whole.
    pub fn capture_range(
        &self,
        index: usize,
        found: &RegExpUnicodeRepeatedStringClassMatch,
    ) -> Option<Range<usize>> {
        self.plan.capture_range(index, found)
    }
    /// Executes at most the original upper bound, rejecting optional empty iterations.
    pub fn find_with_captures_and_work<E>(
        &self,
        input: &JsString,
        requested: usize,
        sticky: bool,
        charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<RegExpUnicodeRepeatedStringClassMatch>, E> {
        self.plan
            .find_with_maximum(input, requested, sticky, self.maximum, charge)
    }
}
fn bounded_count(units: &[u16], at: usize) -> Option<(usize, bool, Option<usize>, bool)> {
    if units.get(at) != Some(&0x7b) {
        return None;
    }
    let mut end = at + 1;
    let digits = |end: &mut usize| {
        let begin = *end;
        let mut value = 0usize;
        while let Some(&digit) = units.get(*end) {
            if !(0x30..=0x39).contains(&digit) {
                break;
            }
            value = value
                .saturating_mul(10)
                .saturating_add(usize::from(digit - 0x30));
            *end += 1;
        }
        (*end != begin).then_some(value)
    };
    let minimum = digits(&mut end)?;
    if minimum > 1 {
        return None;
    }
    let maximum = if units.get(end) == Some(&0x2c) {
        end += 1;
        digits(&mut end)
    } else {
        Some(minimum)
    };
    if units.get(end) != Some(&0x7d)
        || maximum.is_some_and(|limit| limit < minimum)
        || minimum == 1 && maximum == Some(1)
    {
        return None;
    }
    end += 1;
    let lazy = units.get(end) == Some(&0x3f);
    Some((end + usize::from(lazy), minimum == 1, maximum, !lazy))
}

/// A braced repetition with a larger minimum and an always-nullable body.
///
/// Requires validated v/vi input and normalized capture names. The body is one
/// string class with an empty member, or a class/empty alternative. Required
/// empty bodies overwrite the final iteration; optional empty bodies fail the
/// progress check. Empty required tails therefore collapse without capture
/// history or storage proportional to a decimal count (22.2.2.3 RepeatMatcher).
/// https://262.ecma-international.org/17.0/#sec-repeatmatcher
#[derive(Clone, Debug)]
pub struct RegExpUnicodeCountedNullableStringClassCaptureMatcher {
    plan: RegExpUnicodeRepeatedStringClassCaptureMatcher,
    minimum: usize,
    optional: Option<usize>,
}
impl RegExpUnicodeCountedNullableStringClassCaptureMatcher {
    /// Proves a nullable body under one braced count whose minimum exceeds one.
    pub fn compile_with_work<E>(
        source: &JsString,
        ignore_case: bool,
        multiline: bool,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, RegExpUnicodeSetCompileError<E>> {
        let units = source.code_units();
        charge(units.len().saturating_mul(48).saturating_add(256))
            .map_err(RegExpUnicodeSetCompileError::Work)?;
        let (mut minimum, mut optional) = (0, None);
        let Some(hole) = counted_hole(units, |units, at| {
            let (end, lower, remaining, greedy) = nullable_count(units, at)?;
            minimum = lower;
            optional = remaining;
            Some((end, true, false, greedy))
        }) else {
            return Ok(None);
        };
        let mut simple = Vec::with_capacity(units.len());
        simple.extend_from_slice(&units[..hole.atom.end]);
        simple.push(0x2b);
        if !hole.greedy {
            simple.push(0x3f);
        }
        simple.extend_from_slice(&units[hole.count_end..]);
        let Some(plan) = RegExpUnicodeRepeatedStringClassCaptureMatcher::compile_with_work(
            &JsString::from_code_units(simple),
            ignore_case,
            multiline,
            &mut charge,
        )?
        else {
            return Ok(None);
        };
        if !plan.empty_alternative && !plan.class.has_empty_member() {
            return Ok(None);
        }
        Ok(Some(Self {
            plan,
            minimum,
            optional,
        }))
    }
    /// Counts every original lexical capture, including conditional branch slots.
    pub fn capture_count(&self) -> usize {
        self.plan.capture_count()
    }
    /// Projects the whole range and the final required or optional iteration.
    pub fn capture_range(
        &self,
        index: usize,
        found: &RegExpUnicodeRepeatedStringClassMatch,
    ) -> Option<Range<usize>> {
        self.plan.capture_range(index, found)
    }
    /// Executes required bodies and consuming optional bodies at the initial boundary.
    pub fn find_with_captures_and_work<E>(
        &self,
        input: &JsString,
        requested: usize,
        _sticky: bool,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<RegExpUnicodeRepeatedStringClassMatch>, E> {
        charge(4)?;
        let Some(start) = unicode_start(input, requested) else {
            return Ok(None);
        };
        let mut at = start;
        let mut last_iteration = Some(at..at);
        let mut class_selected = false;
        if !self.plan.empty_first {
            let mut required = self.minimum;
            while required != 0 {
                let Some(range) = self.plan.class_prefix(input, at, &mut charge)? else {
                    debug_assert!(self.plan.empty_alternative);
                    last_iteration = Some(at..at);
                    class_selected = false;
                    break;
                };
                debug_assert_eq!(range.start, at);
                let empty = range.end == at;
                at = range.end;
                last_iteration = Some(range);
                class_selected = true;
                if empty {
                    break;
                }
                required -= 1;
            }
        }
        if self.plan.greedy {
            let mut optional = self.optional;
            while optional != Some(0) {
                let Some(range) = self.plan.class_prefix(input, at, &mut charge)? else {
                    break;
                };
                debug_assert_eq!(range.start, at);
                if range.end == at {
                    break;
                }
                at = range.end;
                last_iteration = Some(range);
                class_selected = true;
                optional = optional.map(|left| left - 1);
            }
        }
        Ok(Some(RegExpUnicodeRepeatedStringClassMatch {
            range: start..at,
            last_iteration,
            class_selected,
        }))
    }
}
fn count_digits<'a>(units: &'a [u16], end: &mut usize) -> Option<&'a [u16]> {
    let begin = *end;
    while units
        .get(*end)
        .is_some_and(|unit| (0x30..=0x39).contains(unit))
    {
        *end += 1;
    }
    (*end != begin).then_some(&units[begin..*end])
}
fn significant_digits(digits: &[u16]) -> &[u16] {
    &digits[digits
        .iter()
        .position(|&digit| digit != 0x30)
        .unwrap_or(digits.len())..]
}
fn saturated_decimal(digits: &[u16]) -> usize {
    digits.iter().fold(0usize, |value, &digit| {
        value
            .saturating_mul(10)
            .saturating_add(usize::from(digit - 0x30))
    })
}
// Subtract before saturation: huge adjacent bounds can leave one optional body.
fn decimal_difference(upper: &[u16], lower: &[u16]) -> usize {
    let (mut result, mut place, mut borrow) = (0usize, 1usize, 0i32);
    let mut lower = lower.iter().rev().copied();
    for &upper in upper.iter().rev() {
        let lower = lower.next().unwrap_or(0x30);
        let mut digit = i32::from(upper) - i32::from(lower) - borrow;
        borrow = i32::from(digit < 0);
        if digit < 0 {
            digit += 10;
        }
        result =
            result
                .saturating_add(place.saturating_mul(
                    usize::try_from(digit).expect("borrow leaves a decimal digit"),
                ));
        place = place.saturating_mul(10);
    }
    debug_assert_eq!(borrow, 0);
    result
}
fn nullable_count(units: &[u16], at: usize) -> Option<(usize, usize, Option<usize>, bool)> {
    if units.get(at) != Some(&0x7b) {
        return None;
    }
    let mut end = at + 1;
    let lower = significant_digits(count_digits(units, &mut end)?);
    let minimum = saturated_decimal(lower);
    if minimum < 2 {
        return None;
    }
    let upper = if units.get(end) == Some(&0x2c) {
        end += 1;
        count_digits(units, &mut end).map(significant_digits)
    } else {
        Some(lower)
    };
    if units.get(end) != Some(&0x7d) {
        return None;
    }
    let optional = if let Some(upper) = upper {
        if upper.len() < lower.len() || upper.len() == lower.len() && upper < lower {
            return None;
        }
        Some(decimal_difference(upper, lower))
    } else {
        None
    };
    end += 1;
    let lazy = units.get(end) == Some(&0x3f);
    Some((end + usize::from(lazy), minimum, optional, !lazy))
}

/// A larger braced count of a nonempty prefix-free finite string class.
///
/// Requires validated v/vi input and normalized capture names. No class member
/// may be a strict prefix of another after canonicalization; scalar fallbacks,
/// empty members and empty alternatives remain separate proofs. Each input
/// position therefore has at most one accepted member endpoint. Failed required
/// runs may retry later starting boundaries without word/capture history
/// (22.2.2.3 RepeatMatcher).
/// https://262.ecma-international.org/17.0/#sec-repeatmatcher
#[derive(Clone, Debug)]
pub struct RegExpUnicodePrefixFreeStringClassCaptureMatcher {
    plan: RegExpUnicodeRepeatedStringClassCaptureMatcher,
    minimum: usize,
    optional: Option<usize>,
}
impl RegExpUnicodePrefixFreeStringClassCaptureMatcher {
    /// Proves one larger braced count and all original body/outside terms.
    pub fn compile_with_work<E>(
        source: &JsString,
        ignore_case: bool,
        multiline: bool,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, RegExpUnicodeSetCompileError<E>> {
        let units = source.code_units();
        charge(units.len().saturating_mul(48).saturating_add(256))
            .map_err(RegExpUnicodeSetCompileError::Work)?;
        let (mut minimum, mut optional) = (0, None);
        let Some(hole) = counted_hole(units, |units, at| {
            let (end, lower, remaining, greedy) = nullable_count(units, at)?;
            minimum = lower;
            optional = remaining;
            Some((end, true, false, greedy))
        }) else {
            return Ok(None);
        };
        let mut simple = Vec::with_capacity(units.len());
        simple.extend_from_slice(&units[..hole.atom.end]);
        simple.push(0x2b);
        if !hole.greedy {
            simple.push(0x3f);
        }
        simple.extend_from_slice(&units[hole.count_end..]);
        let Some(plan) = RegExpUnicodeRepeatedStringClassCaptureMatcher::compile_with_work(
            &JsString::from_code_units(simple),
            ignore_case,
            multiline,
            &mut charge,
        )?
        else {
            return Ok(None);
        };
        if plan.empty_alternative
            || plan.class.has_outside_assertions()
            || !plan
                .class
                .has_prefix_free_string_members_with_work(&mut charge)?
        {
            return Ok(None);
        }
        Ok(Some(Self {
            plan,
            minimum,
            optional,
        }))
    }
    /// Counts every original lexical capture.
    pub fn capture_count(&self) -> usize {
        self.plan.capture_count()
    }
    /// Projects outside scopes on the whole range and body scopes on the last member.
    pub fn capture_range(
        &self,
        index: usize,
        found: &RegExpUnicodeRepeatedStringClassMatch,
    ) -> Option<Range<usize>> {
        self.plan.capture_range(index, found)
    }
    /// Tries complete required runs, then consuming optional members up to the bound.
    pub fn find_with_captures_and_work<E>(
        &self,
        input: &JsString,
        requested: usize,
        sticky: bool,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<RegExpUnicodeRepeatedStringClassMatch>, E> {
        charge(4)?;
        let Some(start) = unicode_start(input, requested) else {
            return Ok(None);
        };
        if self.minimum > input.len() - start || self.plan.class.max_match_code_units() == 0 {
            return Ok(None);
        }
        let mut first = self.first_from(input, start, sticky, &mut charge)?;
        while let Some(range) = first {
            let start = range.start;
            if self.minimum > input.len() - start {
                return Ok(None);
            }
            let (mut at, mut last, mut required) = (range.end, range, self.minimum - 1);
            while required != 0 {
                let Some(range) = self.plan.class_prefix(input, at, &mut charge)? else {
                    break;
                };
                debug_assert_eq!(range.start, at);
                debug_assert!(range.end > at);
                if range.end == at {
                    return Ok(None);
                }
                at = range.end;
                last = range;
                required -= 1;
            }
            if required == 0 {
                if self.plan.greedy {
                    let mut optional = self.optional;
                    while optional != Some(0) {
                        let Some(range) = self.plan.class_prefix(input, at, &mut charge)? else {
                            break;
                        };
                        debug_assert_eq!(range.start, at);
                        debug_assert!(range.end > at);
                        if range.end == at {
                            break;
                        }
                        at = range.end;
                        last = range;
                        optional = optional.map(|left| left - 1);
                    }
                }
                return Ok(Some(RegExpUnicodeRepeatedStringClassMatch {
                    range: start..at,
                    last_iteration: Some(last),
                    class_selected: true,
                }));
            }
            if sticky {
                return Ok(None);
            }
            charge(2)?;
            let units = input.code_units();
            let next = start
                + if (0xd800..=0xdbff).contains(&units[start])
                    && units
                        .get(start + 1)
                        .is_some_and(|unit| (0xdc00..=0xdfff).contains(unit))
                {
                    2
                } else {
                    1
                };
            if self.minimum > input.len() - next {
                return Ok(None);
            }
            // Retrying inside a previously matched member is observable: with
            // {abcd,bc,da}{2}, abcda fails at zero and succeeds at one.
            first = self.first_from(input, next, false, &mut charge)?;
        }
        Ok(None)
    }
    fn first_from<E>(
        &self,
        input: &JsString,
        at: usize,
        sticky: bool,
        charge: &mut impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Range<usize>>, E> {
        let found = self.plan.class_prefix(input, at, charge)?;
        if found.is_some() || sticky {
            return Ok(found);
        }
        charge(
            (input.len() - at)
                .saturating_add(1)
                .saturating_mul(self.plan.class.search_passes(false).saturating_add(4)),
        )?;
        Ok(self.plan.class.find(input, at, false))
    }
}

/// A nonnullable string-class body with exactly two required iterations.
///
/// Requires validated v/vi input and normalized capture names. The first
/// member must admit a second; a rejected longer member leaves shorter
/// endpoints eligible. Once both requirements succeed, the proved pure-empty
/// continuation permits ordinary greedy/lazy optional matching without history
/// (22.2.2.3 RepeatMatcher).
/// https://262.ecma-international.org/17.0/#sec-repeatmatcher
#[derive(Clone, Debug)]
pub struct RegExpUnicodeTwoRequiredStringClassCaptureMatcher {
    plan: RegExpUnicodeRepeatedStringClassCaptureMatcher,
    optional: Option<usize>,
    candidates: usize,
}
impl RegExpUnicodeTwoRequiredStringClassCaptureMatcher {
    /// Proves a nonnullable plain body and one braced count with minimum two.
    pub fn compile_with_work<E>(
        source: &JsString,
        ignore_case: bool,
        multiline: bool,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, RegExpUnicodeSetCompileError<E>> {
        let units = source.code_units();
        charge(units.len().saturating_mul(48).saturating_add(256))
            .map_err(RegExpUnicodeSetCompileError::Work)?;
        let mut optional = None;
        let Some(hole) = counted_hole(units, |units, at| {
            let (end, minimum, remaining, greedy) = nullable_count(units, at)?;
            if minimum != 2 {
                return None;
            }
            optional = remaining;
            Some((end, true, false, greedy))
        }) else {
            return Ok(None);
        };
        let mut simple = Vec::with_capacity(units.len());
        simple.extend_from_slice(&units[..hole.atom.end]);
        simple.push(0x2b);
        if !hole.greedy {
            simple.push(0x3f);
        }
        simple.extend_from_slice(&units[hole.count_end..]);
        let Some(plan) = RegExpUnicodeRepeatedStringClassCaptureMatcher::compile_with_work(
            &JsString::from_code_units(simple),
            ignore_case,
            multiline,
            &mut charge,
        )?
        else {
            return Ok(None);
        };
        if plan.empty_alternative
            || plan.class.has_empty_member()
            || plan.class.has_outside_assertions()
        {
            return Ok(None);
        }
        let candidates = plan
            .class
            .prefix_candidate_bound_with_work(&mut charge)
            .map_err(RegExpUnicodeSetCompileError::Work)?;
        Ok(Some(Self {
            plan,
            optional,
            candidates,
        }))
    }
    /// Counts every original lexical capture.
    pub fn capture_count(&self) -> usize {
        self.plan.capture_count()
    }
    /// Projects outer slots on the whole match and body slots on the last member.
    pub fn capture_range(
        &self,
        index: usize,
        found: &RegExpUnicodeRepeatedStringClassMatch,
    ) -> Option<Range<usize>> {
        self.plan.capture_range(index, found)
    }
    /// Tries each first-member endpoint against its required second continuation.
    pub fn find_with_captures_and_work<E>(
        &self,
        input: &JsString,
        requested: usize,
        sticky: bool,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<RegExpUnicodeRepeatedStringClassMatch>, E> {
        charge(4)?;
        let Some(start) = unicode_start(input, requested) else {
            return Ok(None);
        };
        if input.len() - start < 2 || self.candidates == 0 {
            return Ok(None);
        }
        let mut first = self.first_from(input, start, sticky, &mut charge)?;
        while let Some(first_range) = first {
            let start = first_range.start;
            if input.len() - start < 2 {
                return Ok(None);
            }
            let inspected = (input.len() - start)
                .min(self.plan.class.max_match_code_units())
                .saturating_add(1);
            let one = inspected
                .saturating_mul(self.plan.class.search_passes(true))
                .saturating_add(4);
            let probes = self.candidates.min(inspected.saturating_add(1));
            charge(one.saturating_mul(probes.saturating_add(1)))?;
            if let Some((_, second)) = self.plan.class.prefix_pair(input, start) {
                let mut at = second.end;
                let mut last = second;
                if self.plan.greedy {
                    let mut optional = self.optional;
                    while optional != Some(0) {
                        let Some(range) = self.plan.class_prefix(input, at, &mut charge)? else {
                            break;
                        };
                        if range.end == at {
                            break;
                        }
                        at = range.end;
                        last = range;
                        optional = optional.map(|left| left - 1);
                    }
                }
                return Ok(Some(RegExpUnicodeRepeatedStringClassMatch {
                    range: start..at,
                    last_iteration: Some(last),
                    class_selected: true,
                }));
            }
            if sticky {
                return Ok(None);
            }
            charge(2)?;
            let units = input.code_units();
            let next = start
                + if (0xd800..=0xdbff).contains(&units[start])
                    && units
                        .get(start + 1)
                        .is_some_and(|unit| (0xdc00..=0xdfff).contains(unit))
                {
                    2
                } else {
                    1
                };
            if input.len() - next < 2 {
                return Ok(None);
            }
            first = self.first_from(input, next, false, &mut charge)?;
        }
        Ok(None)
    }
    fn first_from<E>(
        &self,
        input: &JsString,
        at: usize,
        sticky: bool,
        charge: &mut impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Range<usize>>, E> {
        let found = self.plan.class_prefix(input, at, charge)?;
        if found.is_some() || sticky {
            return Ok(found);
        }
        charge(
            (input.len() - at)
                .saturating_add(1)
                .saturating_mul(self.plan.class.search_passes(false).saturating_add(4)),
        )?;
        Ok(self.plan.class.find(input, at, false))
    }
}
/// A host failure while executing a Unicode RegExp matcher.
///
/// These failures remain separate from unsupported syntax and JavaScript exceptions.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RegExpUnicodeMatchError<E> {
    /// The embedding's opt-in matching work callback failed.
    Work(E),
    /// Matching continuation storage exceeds platform capacity.
    Capacity,
}
impl<E: std::fmt::Display> std::fmt::Display for RegExpUnicodeMatchError<E> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Work(error) => write!(formatter, "Unicode RegExp matching work failed: {error}"),
            Self::Capacity => {
                formatter.write_str("Unicode RegExp matching exceeds platform capacity")
            }
        }
    }
}
impl<E: std::error::Error + 'static> std::error::Error for RegExpUnicodeMatchError<E> {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Work(error) => Some(error),
            Self::Capacity => None,
        }
    }
}

/// A nonnullable string-class body with three or more required iterations.
///
/// Validated v/vi input and normalized capture names are required. Iterative
/// backtracking retains one endpoint per required iteration on a fallibly
/// allocated flat stack; optional matching begins only after all requirements
/// succeed (22.2.2.3 RepeatMatcher).
/// https://262.ecma-international.org/17.0/#sec-repeatmatcher
#[derive(Clone, Debug)]
pub struct RegExpUnicodeRequiredStringClassCaptureMatcher {
    plan: RegExpUnicodeRepeatedStringClassCaptureMatcher,
    optional: Option<usize>,
    minimum: usize,
}
impl RegExpUnicodeRequiredStringClassCaptureMatcher {
    /// Proves a nonnullable plain body and one braced count with minimum at least three.
    pub fn compile_with_work<E>(
        source: &JsString,
        ignore_case: bool,
        multiline: bool,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, RegExpUnicodeSetCompileError<E>> {
        let units = source.code_units();
        charge(units.len().saturating_mul(48).saturating_add(256))
            .map_err(RegExpUnicodeSetCompileError::Work)?;
        let mut optional = None;
        let mut required = 0;
        let Some(hole) = counted_hole(units, |units, at| {
            let (end, minimum, remaining, greedy) = nullable_count(units, at)?;
            if minimum < 3 {
                return None;
            }
            required = minimum;
            optional = remaining;
            Some((end, true, false, greedy))
        }) else {
            return Ok(None);
        };
        let mut simple = Vec::with_capacity(units.len());
        simple.extend_from_slice(&units[..hole.atom.end]);
        simple.push(0x2b);
        if !hole.greedy {
            simple.push(0x3f);
        }
        simple.extend_from_slice(&units[hole.count_end..]);
        let Some(plan) = RegExpUnicodeRepeatedStringClassCaptureMatcher::compile_with_work(
            &JsString::from_code_units(simple),
            ignore_case,
            multiline,
            &mut charge,
        )?
        else {
            return Ok(None);
        };
        if plan.empty_alternative
            || plan.class.has_empty_member()
            || plan.class.has_outside_assertions()
        {
            return Ok(None);
        }
        Ok(Some(Self {
            plan,
            optional,
            minimum: required,
        }))
    }
    /// Counts every original lexical capture.
    pub fn capture_count(&self) -> usize {
        self.plan.capture_count()
    }
    /// Projects outer slots on the whole match and body slots on the last member.
    pub fn capture_range(
        &self,
        index: usize,
        found: &RegExpUnicodeRepeatedStringClassMatch,
    ) -> Option<Range<usize>> {
        self.plan.capture_range(index, found)
    }
    /// Executes required alternatives with flat, input-bounded continuation storage.
    pub fn find_with_captures_and_work<E>(
        &self,
        input: &JsString,
        requested: usize,
        sticky: bool,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<RegExpUnicodeRepeatedStringClassMatch>, RegExpUnicodeMatchError<E>> {
        use RegExpUnicodeMatchError::{Capacity, Work};
        charge(4).map_err(Work)?;
        let Some(start) = unicode_start(input, requested) else {
            return Ok(None);
        };
        if self.minimum > input.len() - start || self.plan.class.max_match_code_units() == 0 {
            return Ok(None);
        }
        let mut first = self
            .first_from(input, start, sticky, &mut charge)
            .map_err(Work)?;
        if first.is_none() {
            return Ok(None);
        }
        charge(self.minimum.saturating_mul(2).saturating_add(4)).map_err(Work)?;
        let mut required = Vec::new();
        required
            .try_reserve_exact(self.minimum)
            .map_err(|_| Capacity)?;
        while let Some(first_range) = first {
            let start = first_range.start;
            if self.minimum > input.len() - start {
                return Ok(None);
            }
            required.clear();
            required.push(first_range);
            loop {
                if required.len() == self.minimum {
                    let mut last = required.last().expect("required endpoint").clone();
                    let mut at = last.end;
                    if self.plan.greedy {
                        let mut optional = self.optional;
                        while optional != Some(0) {
                            let Some(range) = self
                                .plan
                                .class_prefix(input, at, &mut charge)
                                .map_err(Work)?
                            else {
                                break;
                            };
                            if range.end == at {
                                break;
                            }
                            at = range.end;
                            last = range;
                            optional = optional.map(|left| left - 1);
                        }
                    }
                    return Ok(Some(RegExpUnicodeRepeatedStringClassMatch {
                        range: start..at,
                        last_iteration: Some(last),
                        class_selected: true,
                    }));
                }
                let at = required.last().expect("required endpoint").end;
                charge(2).map_err(Work)?;
                if self.minimum - required.len() <= input.len() - at {
                    if let Some(next) = self
                        .plan
                        .class_prefix(input, at, &mut charge)
                        .map_err(Work)?
                    {
                        required.push(next);
                        continue;
                    }
                }
                let mut retried = false;
                while let Some(previous) = required.pop() {
                    let inspected = (input.len() - previous.start)
                        .min(self.plan.class.max_match_code_units())
                        .saturating_add(1);
                    charge(
                        inspected
                            .saturating_mul(self.plan.class.search_passes(true))
                            .saturating_add(4),
                    )
                    .map_err(Work)?;
                    if let Some(shorter) =
                        self.plan
                            .class
                            .prefix_before(input, previous.start, previous.end)
                    {
                        required.push(shorter);
                        retried = true;
                        break;
                    }
                }
                if !retried {
                    break;
                }
            }
            if sticky {
                return Ok(None);
            }
            charge(2).map_err(Work)?;
            let units = input.code_units();
            let next = start
                + if (0xd800..=0xdbff).contains(&units[start])
                    && units
                        .get(start + 1)
                        .is_some_and(|unit| (0xdc00..=0xdfff).contains(unit))
                {
                    2
                } else {
                    1
                };
            if self.minimum > input.len() - next {
                return Ok(None);
            }
            first = self
                .first_from(input, next, false, &mut charge)
                .map_err(Work)?;
        }
        Ok(None)
    }
    fn first_from<E>(
        &self,
        input: &JsString,
        at: usize,
        sticky: bool,
        charge: &mut impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Range<usize>>, E> {
        let found = self.plan.class_prefix(input, at, charge)?;
        if found.is_some() || sticky {
            return Ok(found);
        }
        charge(
            (input.len() - at)
                .saturating_add(1)
                .saturating_mul(self.plan.class.search_passes(false).saturating_add(4)),
        )?;
        Ok(self.plan.class.find(input, at, false))
    }
}
#[derive(Clone, Copy, Debug, Default)]
struct RepeatedStringAssertions {
    start: bool,
    end: bool,
    word: bool,
    nonword: bool,
}
impl RepeatedStringAssertions {
    fn matches(self, input: &JsString, at: usize, ignore_case: bool, multiline: bool) -> bool {
        use crate::regexp_unicode_character::{unicode_assertions_match, unicode_word_boundary};
        let units = input.code_units();
        unicode_assertions_match(units, at, at, self.start, self.end, multiline)
            && (!self.word || unicode_word_boundary(units, at, ignore_case))
            && (!self.nonword || !unicode_word_boundary(units, at, ignore_case))
    }
}
#[derive(Debug)]
struct AssertedStringRepeatFrame {
    at: usize,
    before: Option<usize>,
    body_done: bool,
    stop_done: bool,
}
impl AssertedStringRepeatFrame {
    fn new(at: usize) -> Self {
        Self {
            at,
            before: None,
            body_done: false,
            stop_done: false,
        }
    }
}
fn asserted_repeat_count(units: &[u16], at: usize) -> Option<(usize, usize, Option<usize>, bool)> {
    if units.get(at) != Some(&0x7b) {
        return None;
    }
    let mut end = at + 1;
    let lower = significant_digits(count_digits(units, &mut end)?);
    let minimum = saturated_decimal(lower);
    let upper = if units.get(end) == Some(&0x2c) {
        end += 1;
        count_digits(units, &mut end).map(significant_digits)
    } else {
        Some(lower)
    };
    if units.get(end) != Some(&0x7d) {
        return None;
    }
    let optional = if let Some(upper) = upper {
        if upper.len() < lower.len() || upper.len() == lower.len() && upper < lower {
            return None;
        }
        Some(decimal_difference(upper, lower))
    } else {
        None
    };
    end += 1;
    let lazy = units.get(end) == Some(&0x3f);
    Some((end + usize::from(lazy), minimum, optional, !lazy))
}

/// A nonnullable string-class repetition with unconditional outside assertions.
///
/// Requires validated v/vi input and normalized capture names. Outside ^/$/word
/// predicates are unconditional; assertion-free empty groups retain their proved
/// alternatives and counts. The body has no assertion or empty branch. A continuation
/// stack retains greedy/lazy count and longest-member backtracking order
/// (22.2.2.3 RepeatMatcher).
/// https://262.ecma-international.org/17.0/#sec-repeatmatcher
#[derive(Clone, Debug)]
pub struct RegExpUnicodeAssertedRepeatedStringClassCaptureMatcher {
    plan: RegExpUnicodeRepeatedStringClassCaptureMatcher,
    minimum: usize,
    maximum: Option<usize>,
    assertions: [RepeatedStringAssertions; 2],
    ignore_case: bool,
    multiline: bool,
}
impl RegExpUnicodeAssertedRepeatedStringClassCaptureMatcher {
    /// Proves one consuming class body and unconditional prefix/suffix predicates.
    pub fn compile_with_work<E>(
        source: &JsString,
        ignore_case: bool,
        multiline: bool,
        charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, RegExpUnicodeSetCompileError<E>> {
        let Some(plan) = Self::compile_plan_with_work(source, ignore_case, multiline, charge)?
        else {
            return Ok(None);
        };
        if plan.plan.empty_alternative || plan.plan.class.has_empty_member() {
            return Ok(None);
        }
        Ok(Some(plan))
    }
    fn compile_plan_with_work<E>(
        source: &JsString,
        ignore_case: bool,
        multiline: bool,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, RegExpUnicodeSetCompileError<E>> {
        let units = source.code_units();
        charge(units.len().saturating_mul(64).saturating_add(256))
            .map_err(RegExpUnicodeSetCompileError::Work)?;
        let (mut minimum, mut maximum) = (0, None);
        let Some(hole) = counted_hole(units, |units, at| {
            if let Some((end, one, maximum_one, greedy)) = simple_count(units, at) {
                minimum = usize::from(one);
                maximum = maximum_one.then_some(1);
                return Some((end, one, maximum_one, greedy));
            }
            let (end, low, delta, greedy) = asserted_repeat_count(units, at)?;
            if low == 1 && delta == Some(0) {
                return None;
            }
            minimum = low;
            maximum = delta.map(|remaining| low.saturating_add(remaining));
            Some((end, low > 0, maximum == Some(1), greedy))
        }) else {
            return Ok(None);
        };
        let mut assertions = [RepeatedStringAssertions::default(); 2];
        let (mut at, mut seen) = (0, false);
        let capacity = units
            .len()
            .checked_mul(4)
            .ok_or(RegExpUnicodeSetCompileError::Capacity)?;
        let mut simple = Vec::new();
        simple
            .try_reserve_exact(capacity)
            .map_err(|_| RegExpUnicodeSetCompileError::Capacity)?;
        let mut scopes = Vec::new();
        scopes
            .try_reserve_exact(units.len().saturating_add(1))
            .map_err(|_| RegExpUnicodeSetCompileError::Capacity)?;
        // Each outside scope records assertion participation and alternation.
        scopes.push((false, false));
        while at < units.len() {
            if at == hole.atom.start {
                simple.extend_from_slice(&units[hole.atom.clone()]);
                simple.push(0x2b);
                if !hole.greedy {
                    simple.push(0x3f);
                }
                at = hole.count_end;
                continue;
            }
            let phase = usize::from(at >= hole.count_end);
            let mut assertion = false;
            match units[at] {
                0x5e => {
                    assertions[phase].start = true;
                    at += 1;
                    assertion = true;
                }
                0x24 => {
                    assertions[phase].end = true;
                    at += 1;
                    assertion = true;
                }
                0x5c => match units.get(at + 1) {
                    Some(0x62) => {
                        assertions[phase].word = true;
                        at += 2;
                        assertion = true;
                    }
                    Some(0x42) => {
                        assertions[phase].nonword = true;
                        at += 2;
                        assertion = true;
                    }
                    _ => return Ok(None),
                },
                0x28 => {
                    let length = if units.get(at + 1) == Some(&0x3f) {
                        if units.get(at + 2) != Some(&0x3a) {
                            return Ok(None);
                        }
                        3
                    } else {
                        1
                    };
                    scopes.push((false, false));
                    simple.extend_from_slice(&units[at..at + length]);
                    at += length;
                }
                0x29 => {
                    if scopes.len() == 1 {
                        return Ok(None);
                    }
                    let (asserted, alternate) = scopes.pop().expect("outside scope");
                    if asserted && alternate {
                        return Ok(None);
                    }
                    scopes.last_mut().expect("outside scope").0 |= asserted;
                    simple.push(0x29);
                    at += 1;
                    if let Some((end, _, _, _)) = simple_count(units, at) {
                        if asserted {
                            return Ok(None);
                        }
                        simple.extend_from_slice(&units[at..end]);
                        at = end;
                    } else if let Some((end, minimum, delta, _)) = asserted_repeat_count(units, at)
                    {
                        if asserted && !(minimum == 1 && delta == Some(0)) {
                            return Ok(None);
                        }
                        simple.extend_from_slice(&units[at..end]);
                        at = end;
                    }
                }
                0x7c => {
                    scopes.last_mut().expect("outside scope").1 = true;
                    simple.push(0x7c);
                    at += 1;
                }
                _ => return Ok(None),
            }
            if assertion {
                seen = true;
                scopes.last_mut().expect("outside scope").0 = true;
                simple.extend_from_slice(&[0x28, 0x3f, 0x3a, 0x29]);
            }
        }
        if scopes.len() != 1 || scopes[0].0 && scopes[0].1 {
            return Ok(None);
        }
        if !seen {
            return Ok(None);
        }
        let Some(plan) = RegExpUnicodeRepeatedStringClassCaptureMatcher::compile_with_work(
            &JsString::from_code_units(simple),
            ignore_case,
            multiline,
            &mut charge,
        )?
        else {
            return Ok(None);
        };
        if plan.class.has_outside_assertions() {
            return Ok(None);
        }
        Ok(Some(Self {
            plan,
            minimum,
            maximum,
            assertions,
            ignore_case,
            multiline,
        }))
    }
    /// Counts every original lexical capture.
    pub fn capture_count(&self) -> usize {
        self.plan.capture_count()
    }
    /// Projects outside scopes on the whole match and body scopes on its last member.
    pub fn capture_range(
        &self,
        index: usize,
        found: &RegExpUnicodeRepeatedStringClassMatch,
    ) -> Option<Range<usize>> {
        self.plan.capture_range(index, found)
    }
    /// Executes assertions, member alternatives and count continuations on a flat stack.
    pub fn find_with_captures_and_work<E>(
        &self,
        input: &JsString,
        requested: usize,
        sticky: bool,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<RegExpUnicodeRepeatedStringClassMatch>, RegExpUnicodeMatchError<E>> {
        use RegExpUnicodeMatchError::{Capacity, Work};
        charge(4).map_err(Work)?;
        let Some(mut start) = unicode_start(input, requested) else {
            return Ok(None);
        };
        if self.minimum > input.len() - start {
            return Ok(None);
        }
        let mut frames = Vec::<AssertedStringRepeatFrame>::new();
        loop {
            charge(8).map_err(Work)?;
            if self.assertions[0].matches(input, start, self.ignore_case, self.multiline) {
                if frames.capacity() == 0 {
                    charge(6).map_err(Work)?;
                    frames.try_reserve(1).map_err(|_| Capacity)?;
                }
                frames.clear();
                frames.push(AssertedStringRepeatFrame::new(start));
                while !frames.is_empty() {
                    charge(4).map_err(Work)?;
                    let depth = frames.len() - 1;
                    let frame = frames.last_mut().expect("continuation frame");
                    let at = frame.at;
                    let can_stop = depth >= self.minimum;
                    if can_stop && !self.plan.greedy && !frame.stop_done {
                        frame.stop_done = true;
                        charge(8).map_err(Work)?;
                        if self.assertions[1].matches(input, at, self.ignore_case, self.multiline) {
                            return Ok(Some(self.result(&frames, start)));
                        }
                    }
                    if !frame.body_done {
                        if self.maximum.is_some_and(|maximum| depth >= maximum)
                            || input.len() - at < self.minimum.saturating_sub(depth)
                        {
                            frame.body_done = true;
                        } else {
                            let next = if let Some(before) = frame.before {
                                let inspected = (input.len() - at)
                                    .min(self.plan.class.max_match_code_units())
                                    .saturating_add(1);
                                charge(
                                    inspected
                                        .saturating_mul(self.plan.class.search_passes(true))
                                        .saturating_add(4),
                                )
                                .map_err(Work)?;
                                self.plan.class.prefix_before(input, at, before)
                            } else {
                                self.plan
                                    .class_prefix(input, at, &mut charge)
                                    .map_err(Work)?
                            };
                            if let Some(next) = next {
                                frame.before = Some(next.end);
                                if frames.len() == frames.capacity() {
                                    charge(frames.capacity().saturating_mul(6).saturating_add(6))
                                        .map_err(Work)?;
                                    frames.try_reserve(1).map_err(|_| Capacity)?;
                                }
                                frames.push(AssertedStringRepeatFrame::new(next.end));
                                continue;
                            }
                            frame.body_done = true;
                        }
                    }
                    if can_stop && self.plan.greedy && !frame.stop_done {
                        frame.stop_done = true;
                        charge(8).map_err(Work)?;
                        if self.assertions[1].matches(input, at, self.ignore_case, self.multiline) {
                            return Ok(Some(self.result(&frames, start)));
                        }
                    }
                    frames.pop();
                }
            }
            if sticky || start == input.len() {
                return Ok(None);
            }
            charge(2).map_err(Work)?;
            let units = input.code_units();
            let first = units[start];
            start +=
                crate::regexp_unicode_character::unicode_input_character(units, start, first).1;
            if self.minimum > input.len() - start {
                return Ok(None);
            }
        }
    }
    fn result(
        &self,
        frames: &[AssertedStringRepeatFrame],
        start: usize,
    ) -> RegExpUnicodeRepeatedStringClassMatch {
        let at = frames.last().expect("successful continuation").at;
        RegExpUnicodeRepeatedStringClassMatch {
            range: start..at,
            last_iteration: frames
                .len()
                .checked_sub(2)
                .map(|previous| frames[previous].at..at),
            class_selected: frames.len() > 1,
        }
    }
}

#[derive(Clone, Copy)]
struct NullableStringRepeatConfig {
    minimum: usize,
    maximum: Option<usize>,
    greedy: bool,
    allow_required_empty: bool,
}
#[derive(Debug)]
struct NullableAssertedStringRepeatFrame {
    at: usize,
    before: Option<usize>,
    body_done: bool,
    stop_done: bool,
    epsilon_done: bool,
    class_selected: bool,
}
impl NullableAssertedStringRepeatFrame {
    fn new(at: usize, class_selected: bool) -> Self {
        Self {
            at,
            before: None,
            body_done: false,
            stop_done: false,
            epsilon_done: false,
            class_selected,
        }
    }
}
/// A nullable string-class body with unconditional assertions and minimum zero or one.
///
/// Requires validated v/vi input and normalized capture names. A required empty
/// class/body branch participates once; optional empty iterations are rejected.
/// Required branch priority and member/count retries follow RepeatMatcher,
/// retaining only the final iteration's class-versus-empty identity.
/// https://262.ecma-international.org/17.0/#sec-repeatmatcher
#[derive(Clone, Debug)]
pub struct RegExpUnicodeAssertedNullableStringClassCaptureMatcher {
    plan: RegExpUnicodeAssertedRepeatedStringClassCaptureMatcher,
}
impl RegExpUnicodeAssertedNullableStringClassCaptureMatcher {
    /// Proves a nullable body, unconditional predicates and a zero/one minimum.
    pub fn compile_with_work<E>(
        source: &JsString,
        ignore_case: bool,
        multiline: bool,
        charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, RegExpUnicodeSetCompileError<E>> {
        let Some(plan) =
            RegExpUnicodeAssertedRepeatedStringClassCaptureMatcher::compile_plan_with_work(
                source,
                ignore_case,
                multiline,
                charge,
            )?
        else {
            return Ok(None);
        };
        if plan.minimum > 1 || !(plan.plan.empty_alternative || plan.plan.class.has_empty_member())
        {
            return Ok(None);
        }
        Ok(Some(Self { plan }))
    }
    /// Counts every original lexical capture.
    pub fn capture_count(&self) -> usize {
        self.plan.capture_count()
    }
    /// Projects lexical scopes on the whole match or the final required/optional body.
    pub fn capture_range(
        &self,
        index: usize,
        found: &RegExpUnicodeRepeatedStringClassMatch,
    ) -> Option<Range<usize>> {
        self.plan.capture_range(index, found)
    }
    /// Executes assertions, member alternatives and count continuations on a flat stack.
    pub fn find_with_captures_and_work<E>(
        &self,
        input: &JsString,
        requested: usize,
        sticky: bool,
        charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<RegExpUnicodeRepeatedStringClassMatch>, RegExpUnicodeMatchError<E>> {
        Self::find_plan_with_work(&self.plan, input, requested, sticky, charge)
    }
    fn find_plan_with_work<E>(
        plan: &RegExpUnicodeAssertedRepeatedStringClassCaptureMatcher,
        input: &JsString,
        requested: usize,
        sticky: bool,
        charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<RegExpUnicodeRepeatedStringClassMatch>, RegExpUnicodeMatchError<E>> {
        Self::find_plan_with_config(
            plan,
            NullableStringRepeatConfig {
                minimum: plan.minimum,
                maximum: plan.maximum,
                greedy: plan.plan.greedy,
                allow_required_empty: true,
            },
            input,
            requested,
            sticky,
            charge,
        )
    }
    fn find_plan_with_config<E>(
        plan: &RegExpUnicodeAssertedRepeatedStringClassCaptureMatcher,
        config: NullableStringRepeatConfig,
        input: &JsString,
        requested: usize,
        sticky: bool,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<RegExpUnicodeRepeatedStringClassMatch>, RegExpUnicodeMatchError<E>> {
        use RegExpUnicodeMatchError::{Capacity, Work};
        charge(4).map_err(Work)?;
        let Some(mut start) = unicode_start(input, requested) else {
            return Ok(None);
        };
        let mut frames = Vec::<NullableAssertedStringRepeatFrame>::new();
        loop {
            charge(8).map_err(Work)?;
            if plan.assertions[0].matches(input, start, plan.ignore_case, plan.multiline) {
                if frames.capacity() == 0 {
                    charge(6).map_err(Work)?;
                    frames.try_reserve(1).map_err(|_| Capacity)?;
                }
                frames.clear();
                frames.push(NullableAssertedStringRepeatFrame::new(start, false));
                while !frames.is_empty() {
                    charge(4).map_err(Work)?;
                    let depth = frames.len() - 1;
                    let frame = frames.last_mut().expect("continuation frame");
                    let at = frame.at;
                    let can_stop = depth >= config.minimum;
                    if can_stop && !config.greedy && !frame.stop_done {
                        frame.stop_done = true;
                        charge(8).map_err(Work)?;
                        if plan.assertions[1].matches(input, at, plan.ignore_case, plan.multiline) {
                            return Ok(Some(Self::result(&frames, start)));
                        }
                    }
                    if config.allow_required_empty
                        && depth < config.minimum
                        && plan.plan.empty_first
                        && !frame.epsilon_done
                    {
                        frame.epsilon_done = true;
                        if frames.len() == frames.capacity() {
                            charge(frames.capacity().saturating_mul(8).saturating_add(8))
                                .map_err(Work)?;
                            frames.try_reserve(1).map_err(|_| Capacity)?;
                        }
                        frames.push(NullableAssertedStringRepeatFrame::new(at, false));
                        continue;
                    }
                    if !frame.body_done {
                        if config.maximum.is_some_and(|maximum| depth >= maximum)
                            || if config.allow_required_empty && config.minimum > 1 {
                                at == input.len()
                            } else {
                                input.len() - at < config.minimum.saturating_sub(depth)
                            }
                        {
                            frame.body_done = true;
                        } else {
                            let next = if let Some(before) = frame.before {
                                let inspected = (input.len() - at)
                                    .min(plan.plan.class.max_match_code_units())
                                    .saturating_add(1);
                                charge(
                                    inspected
                                        .saturating_mul(plan.plan.class.search_passes(true))
                                        .saturating_add(4),
                                )
                                .map_err(Work)?;
                                plan.plan.class.prefix_before(input, at, before)
                            } else {
                                plan.plan
                                    .class_prefix(input, at, &mut charge)
                                    .map_err(Work)?
                            };
                            if let Some(next) = next.filter(|range| range.end > at) {
                                frame.before = Some(next.end);
                                if frames.len() == frames.capacity() {
                                    charge(frames.capacity().saturating_mul(8).saturating_add(8))
                                        .map_err(Work)?;
                                    frames.try_reserve(1).map_err(|_| Capacity)?;
                                }
                                frames.push(NullableAssertedStringRepeatFrame::new(next.end, true));
                                continue;
                            }
                            frame.body_done = true;
                        }
                    }
                    if config.allow_required_empty && depth < config.minimum && !frame.epsilon_done
                    {
                        frame.epsilon_done = true;
                        let selected = plan.plan.class.has_empty_member();
                        if selected || plan.plan.empty_alternative {
                            if frames.len() == frames.capacity() {
                                charge(frames.capacity().saturating_mul(8).saturating_add(8))
                                    .map_err(Work)?;
                                frames.try_reserve(1).map_err(|_| Capacity)?;
                            }
                            frames.push(NullableAssertedStringRepeatFrame::new(at, selected));
                            continue;
                        }
                    }
                    if can_stop && config.greedy && !frame.stop_done {
                        frame.stop_done = true;
                        charge(8).map_err(Work)?;
                        if plan.assertions[1].matches(input, at, plan.ignore_case, plan.multiline) {
                            return Ok(Some(Self::result(&frames, start)));
                        }
                    }
                    frames.pop();
                }
            }
            if sticky || start == input.len() {
                return Ok(None);
            }
            charge(2).map_err(Work)?;
            let units = input.code_units();
            let first = units[start];
            start +=
                crate::regexp_unicode_character::unicode_input_character(units, start, first).1;
        }
    }
    fn result(
        frames: &[NullableAssertedStringRepeatFrame],
        start: usize,
    ) -> RegExpUnicodeRepeatedStringClassMatch {
        let at = frames.last().expect("successful continuation").at;
        RegExpUnicodeRepeatedStringClassMatch {
            range: start..at,
            last_iteration: frames
                .len()
                .checked_sub(2)
                .map(|previous| frames[previous].at..at),
            class_selected: frames.len() > 1
                && frames
                    .last()
                    .expect("successful continuation")
                    .class_selected,
        }
    }
}

/// A nullable asserted string-class body with exactly two required iterations.
///
/// Requires validated v/vi input and normalized capture names. Each required
/// empty body participates and resets captures; optional empty progress is
/// rejected. Word, branch and repetition continuations use a flat stack.
/// https://262.ecma-international.org/17.0/#sec-repeatmatcher
#[derive(Clone, Debug)]
pub struct RegExpUnicodeAssertedTwoRequiredNullableStringClassCaptureMatcher {
    plan: RegExpUnicodeAssertedRepeatedStringClassCaptureMatcher,
}
impl RegExpUnicodeAssertedTwoRequiredNullableStringClassCaptureMatcher {
    /// Proves a nullable body, unconditional assertions and minimum exactly two.
    pub fn compile_with_work<E>(
        source: &JsString,
        ignore_case: bool,
        multiline: bool,
        charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, RegExpUnicodeSetCompileError<E>> {
        let Some(plan) =
            RegExpUnicodeAssertedRepeatedStringClassCaptureMatcher::compile_plan_with_work(
                source,
                ignore_case,
                multiline,
                charge,
            )?
        else {
            return Ok(None);
        };
        if plan.minimum != 2 || !(plan.plan.empty_alternative || plan.plan.class.has_empty_member())
        {
            return Ok(None);
        }
        Ok(Some(Self { plan }))
    }
    /// Counts every original lexical capture.
    pub fn capture_count(&self) -> usize {
        self.plan.capture_count()
    }
    /// Projects outside scopes on the whole match and body slots on its last iteration.
    pub fn capture_range(
        &self,
        index: usize,
        found: &RegExpUnicodeRepeatedStringClassMatch,
    ) -> Option<Range<usize>> {
        self.plan.capture_range(index, found)
    }
    /// Executes two required bodies and consuming optional continuations.
    pub fn find_with_captures_and_work<E>(
        &self,
        input: &JsString,
        requested: usize,
        sticky: bool,
        charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<RegExpUnicodeRepeatedStringClassMatch>, RegExpUnicodeMatchError<E>> {
        RegExpUnicodeAssertedNullableStringClassCaptureMatcher::find_plan_with_work(
            &self.plan, input, requested, sticky, charge,
        )
    }
}

/// A larger nullable finite string-class count with unconditional assertions.
///
/// Requires validated v/vi input and normalized capture names. Finite words may
/// overlap and contain an empty member; a separate empty arm may occur in either
/// order. Scalar fallback and smaller counts remain separate proofs. Counts above
/// the remaining input bound reduce required-empty padding without allocating
/// storage proportional to their decimal value (22.2.2.3 RepeatMatcher).
/// https://262.ecma-international.org/17.0/#sec-repeatmatcher
#[derive(Clone, Debug)]
pub struct RegExpUnicodeAssertedCountedNullableStringClassCaptureMatcher {
    plan: RegExpUnicodeAssertedRepeatedStringClassCaptureMatcher,
    optional: Option<usize>,
    empty_only: bool,
}
impl RegExpUnicodeAssertedCountedNullableStringClassCaptureMatcher {
    /// Proves a finite nullable body, unconditional assertions and minimum at least three.
    pub fn compile_with_work<E>(
        source: &JsString,
        ignore_case: bool,
        multiline: bool,
        charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, RegExpUnicodeSetCompileError<E>> {
        Self::compile_body_with_work(source, ignore_case, multiline, false, charge)
    }
    fn compile_body_with_work<E>(
        source: &JsString,
        ignore_case: bool,
        multiline: bool,
        require_scalar: bool,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, RegExpUnicodeSetCompileError<E>> {
        let Some(plan) =
            RegExpUnicodeAssertedRepeatedStringClassCaptureMatcher::compile_plan_with_work(
                source,
                ignore_case,
                multiline,
                &mut charge,
            )?
        else {
            return Ok(None);
        };
        if plan.minimum < 3
            || !(plan.plan.empty_alternative || plan.plan.class.has_empty_member())
            || plan.plan.class.has_scalar_fallback() != require_scalar
        {
            return Ok(None);
        }
        charge(source.len().saturating_mul(8).saturating_add(64))
            .map_err(RegExpUnicodeSetCompileError::Work)?;
        let mut optional = None;
        let hole = counted_hole(source.code_units(), |units, at| {
            let (end, minimum, delta, greedy) = asserted_repeat_count(units, at)?;
            if minimum < 3 {
                return None;
            }
            optional = delta;
            Some((end, true, false, greedy))
        });
        if hole.is_none() {
            return Ok(None);
        }
        let members = plan
            .plan
            .class
            .prefix_candidate_bound_with_work(&mut charge)
            .map_err(RegExpUnicodeSetCompileError::Work)?;
        let empty_only = members == usize::from(plan.plan.class.has_empty_member());
        Ok(Some(Self {
            plan,
            optional,
            empty_only,
        }))
    }
    /// Counts every original lexical capture.
    pub fn capture_count(&self) -> usize {
        self.plan.capture_count()
    }
    /// Projects lexical slots from the whole range and final body identity.
    pub fn capture_range(
        &self,
        index: usize,
        found: &RegExpUnicodeRepeatedStringClassMatch,
    ) -> Option<Range<usize>> {
        self.plan.capture_range(index, found)
    }
    /// Executes bounded required state or reduces a required-empty padding run.
    pub fn find_with_captures_and_work<E>(
        &self,
        input: &JsString,
        requested: usize,
        sticky: bool,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<RegExpUnicodeRepeatedStringClassMatch>, RegExpUnicodeMatchError<E>> {
        use RegExpUnicodeAssertedNullableStringClassCaptureMatcher as Nullable;
        use RegExpUnicodeMatchError::Work;
        charge(4).map_err(Work)?;
        let Some(mut start) = unicode_start(input, requested) else {
            return Ok(None);
        };
        if self
            .plan
            .assertions
            .iter()
            .any(|assertions| assertions.word && assertions.nonword)
        {
            return Ok(None);
        }
        let preferred_empty_class =
            !self.plan.plan.empty_first && self.plan.plan.class.has_empty_member();
        loop {
            charge(8).map_err(Work)?;
            if self.plan.assertions[0].matches(
                input,
                start,
                self.plan.ignore_case,
                self.plan.multiline,
            ) {
                let remaining = input.len() - start;
                if self.empty_only {
                    charge(8).map_err(Work)?;
                    if self.plan.assertions[1].matches(
                        input,
                        start,
                        self.plan.ignore_case,
                        self.plan.multiline,
                    ) {
                        return Ok(Some(RegExpUnicodeRepeatedStringClassMatch {
                            range: start..start,
                            last_iteration: Some(start..start),
                            class_selected: preferred_empty_class,
                        }));
                    }
                } else if self.plan.minimum <= remaining {
                    let config = NullableStringRepeatConfig {
                        minimum: self.plan.minimum,
                        maximum: self
                            .optional
                            .map(|optional| self.plan.minimum.saturating_add(optional)),
                        greedy: self.plan.plan.greedy,
                        allow_required_empty: true,
                    };
                    if let Some(found) = Nullable::find_plan_with_config(
                        &self.plan,
                        config,
                        input,
                        start,
                        true,
                        &mut charge,
                    )? {
                        return Ok(Some(found));
                    }
                } else if !self.plan.plan.empty_first {
                    // More required copies than input units force a preferred empty tail.
                    let config = NullableStringRepeatConfig {
                        minimum: 0,
                        maximum: None,
                        greedy: true,
                        allow_required_empty: false,
                    };
                    if let Some(mut found) = Nullable::find_plan_with_config(
                        &self.plan,
                        config,
                        input,
                        start,
                        true,
                        &mut charge,
                    )? {
                        found.last_iteration = Some(found.range.end..found.range.end);
                        found.class_selected = preferred_empty_class;
                        return Ok(Some(found));
                    }
                } else {
                    // Empty-first required paths first exhaust zero consuming required
                    // copies, then one, then two, with endpoint order within each tier.
                    for consuming in 0..=remaining {
                        charge(2).map_err(Work)?;
                        let config = NullableStringRepeatConfig {
                            minimum: consuming,
                            maximum: self
                                .optional
                                .map(|optional| consuming.saturating_add(optional)),
                            greedy: self.plan.plan.greedy,
                            allow_required_empty: false,
                        };
                        if let Some(mut found) = Nullable::find_plan_with_config(
                            &self.plan,
                            config,
                            input,
                            start,
                            true,
                            &mut charge,
                        )? {
                            if found.last_iteration.is_none() {
                                found.last_iteration = Some(found.range.end..found.range.end);
                                found.class_selected = false;
                            }
                            return Ok(Some(found));
                        }
                        if consuming == 0
                            && self.optional.is_none_or(|optional| optional >= remaining)
                        {
                            break;
                        }
                    }
                }
            }
            if sticky || start == input.len() {
                return Ok(None);
            }
            charge(2).map_err(Work)?;
            let units = input.code_units();
            start += crate::regexp_unicode_character::unicode_input_character(
                units,
                start,
                units[start],
            )
            .1;
        }
    }
}

/// A larger asserted nullable string-class count with ordinary scalar fallback.
///
/// Requires validated v/vi input and normalized capture names. Scalar fallback
/// retains the same descending endpoint order as finite words. Every consuming
/// choice advances at least one input unit, so required-empty count reduction
/// and the exact optional allowance apply to both kinds of member.
/// https://262.ecma-international.org/17.0/#sec-repeatmatcher
#[derive(Clone, Debug)]
pub struct RegExpUnicodeAssertedCountedNullableScalarStringClassCaptureMatcher {
    plan: RegExpUnicodeAssertedCountedNullableStringClassCaptureMatcher,
}
impl RegExpUnicodeAssertedCountedNullableScalarStringClassCaptureMatcher {
    /// Proves a nullable scalar/string class, outside assertions and a larger count.
    pub fn compile_with_work<E>(
        source: &JsString,
        ignore_case: bool,
        multiline: bool,
        charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, RegExpUnicodeSetCompileError<E>> {
        RegExpUnicodeAssertedCountedNullableStringClassCaptureMatcher::compile_body_with_work(
            source,
            ignore_case,
            multiline,
            true,
            charge,
        )
        .map(|plan| plan.map(|plan| Self { plan }))
    }
    /// Counts every original lexical capture.
    pub fn capture_count(&self) -> usize {
        self.plan.capture_count()
    }
    /// Projects lexical slots from the whole range and final body identity.
    pub fn capture_range(
        &self,
        index: usize,
        found: &RegExpUnicodeRepeatedStringClassMatch,
    ) -> Option<Range<usize>> {
        self.plan.capture_range(index, found)
    }
    /// Executes scalar and finite-word continuations with required-empty reduction.
    pub fn find_with_captures_and_work<E>(
        &self,
        input: &JsString,
        requested: usize,
        sticky: bool,
        charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<RegExpUnicodeRepeatedStringClassMatch>, RegExpUnicodeMatchError<E>> {
        self.plan
            .find_with_captures_and_work(input, requested, sticky, charge)
    }
}

mod zero_count_asserted {
    include!("regexp_unicode_zero_count_asserted_string_class.rs");
}
pub use zero_count_asserted::{
    RegExpUnicodeUnassertedZeroCountSingleStringClassCaptureMatcher,
    RegExpUnicodeUnassertedZeroCountStringClassAlternativeMatcher,
    RegExpUnicodeUnassertedZeroCountStringClassCaptureMatcher,
    RegExpUnicodeZeroCountAssertedSingleStringClassCaptureMatcher,
    RegExpUnicodeZeroCountAssertedStringClassCaptureMatcher,
};
