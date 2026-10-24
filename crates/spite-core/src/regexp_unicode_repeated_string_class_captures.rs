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
            loop {
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
                if let Some((end, minimum_one, maximum_one, greedy)) = simple_count(units, at) {
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
                    if let Some((end, minimum_one, maximum_one, greedy)) = simple_count(units, at) {
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
