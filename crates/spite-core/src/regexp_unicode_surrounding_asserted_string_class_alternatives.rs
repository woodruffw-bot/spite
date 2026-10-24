//! Required empty assertion siblings around one class/empty disjunction.
use super::super::super::{CaptureRole, projected_selected_capture_range};
use super::{
    JsString, RegExpUnicodeAssertedStringClassAlternativeMatcher, RegExpUnicodeSetCompileError,
    RegExpUnicodeSingleStringClassCaptureMatcher, RegExpUnicodeStringClassBranchMatch,
    has_root_alternative, root_bar,
};
use std::ops::Range;

/// A proved class/empty choice with flat outer whole-range and point captures.
///
/// Requires validated v/vi input and normalized capture names. One ordinary
/// group contains the two-way choice; all outside terms pass the existing
/// required single-class spine proof with a synthetic class at the choice.
/// Predicates are conjoined on each original branch at its actual endpoints.
/// Both full proofs precede selection, including false conjunctions. Lexical
/// inner slots are inserted between the original outside capture slots
/// (22.2.2.3–4, 22.2.2.7).
/// https://262.ecma-international.org/17.0/#sec-compileassertion
#[derive(Clone, Debug)]
pub struct RegExpUnicodeSurroundingAssertedStringClassAlternativeMatcher {
    inner: RegExpUnicodeAssertedStringClassAlternativeMatcher,
    outside: Box<[CaptureRole]>,
    before_inner: usize,
    captures: usize,
}
impl RegExpUnicodeSurroundingAssertedStringClassAlternativeMatcher {
    /// Proves all original branches and the complete outside empty-term spine.
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
        let Some(hole) = choice_hole(units) else {
            return Ok(None);
        };
        let payload = &units[hole.body.clone()];
        let Some(bar) = root_bar(payload) else {
            return Ok(None);
        };
        let mut shadow = Vec::with_capacity(units.len().saturating_add(6));
        shadow.extend_from_slice(&units[..hole.body.start]);
        shadow.extend_from_slice(&[0x5b, 0x5c, 0x71, 0x7b, 0x7d, 0x5d]);
        shadow.extend_from_slice(&units[hole.body.end..]);
        let Some(gates) = outside_gates(&shadow) else {
            return Ok(None);
        };
        let mut decorated = Vec::with_capacity(payload.len().saturating_add(24));
        for (index, branch) in [&payload[..bar], &payload[bar + 1..]]
            .into_iter()
            .enumerate()
        {
            if index != 0 {
                decorated.push(0x7c);
            }
            if gates.start_line {
                decorated.push(0x5e);
            }
            if gates.start_end_line {
                decorated.push(0x24);
            }
            if gates.start_words & 1 != 0 {
                decorated.extend_from_slice(&[0x5c, 0x62]);
            }
            if gates.start_words & 2 != 0 {
                decorated.extend_from_slice(&[0x5c, 0x42]);
            }
            decorated.extend_from_slice(branch);
            if gates.end_words & 1 != 0 {
                decorated.extend_from_slice(&[0x5c, 0x62]);
            }
            if gates.end_words & 2 != 0 {
                decorated.extend_from_slice(&[0x5c, 0x42]);
            }
            if gates.end_start_line {
                decorated.push(0x5e);
            }
            if gates.end_line {
                decorated.push(0x24);
            }
        }
        let Some(inner) = RegExpUnicodeAssertedStringClassAlternativeMatcher::compile_with_work(
            &JsString::from_code_units(decorated),
            ignore_case,
            multiline,
            &mut charge,
        )?
        else {
            return Ok(None);
        };
        // The placeholder proves structure/participation only. Its empty width
        // must never decide eligibility or collapse opposite endpoint words.
        let shadow = JsString::from_code_units(shadow);
        let outside=if let Some(plan)=RegExpUnicodeSingleStringClassCaptureMatcher::compile_with_work(&shadow,ignore_case,multiline,&mut charge)? {
            Some(plan)
        }else if let Some(plan)=RegExpUnicodeSingleStringClassCaptureMatcher::compile_unconditional_assertions_with_work(&shadow,ignore_case,multiline,&mut charge)? {
            Some(plan)
        }else{
            RegExpUnicodeSingleStringClassCaptureMatcher::compile_reversed_line_assertions_with_work(&shadow,ignore_case,multiline,&mut charge)?
        };
        let Some(outside) = outside else {
            return Ok(None);
        };
        if hole.before > outside.capture_count() {
            return Ok(None);
        }
        let captures = outside
            .capture_count()
            .checked_add(inner.capture_count())
            .ok_or(RegExpUnicodeSetCompileError::Capacity)?;
        Ok(Some(Self {
            inner,
            outside: outside.captures,
            before_inner: hole.before,
            captures,
        }))
    }
    /// Counts original outside and branch captures in lexical source order.
    pub fn capture_count(&self) -> usize {
        self.captures
    }
    /// Projects outside ranges/points and conditional original branch slots.
    pub fn capture_range(
        &self,
        index: usize,
        found: &RegExpUnicodeStringClassBranchMatch,
    ) -> Option<Range<usize>> {
        if index >= self.captures {
            return None;
        }
        if index < self.before_inner {
            return projected_selected_capture_range(&self.outside, index, &found.range, true);
        }
        if index < self.before_inner + self.inner.capture_count() {
            return self.inner.capture_range(index - self.before_inner, found);
        }
        projected_selected_capture_range(
            &self.outside,
            index - self.inner.capture_count(),
            &found.range,
            true,
        )
    }
    /// Delegates ordered Unicode searches to the complete original branch proof.
    pub fn find_with_captures_and_work<E>(
        &self,
        input: &JsString,
        start: usize,
        sticky: bool,
        charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<RegExpUnicodeStringClassBranchMatch>, E> {
        self.inner
            .find_with_captures_and_work(input, start, sticky, charge)
    }
}
struct Hole {
    body: Range<usize>,
    before: usize,
}
struct HoleFrame {
    body: usize,
    before: usize,
    class: bool,
    bars: usize,
}
fn class_end(units: &[u16], start: usize) -> Option<usize> {
    let mut depth = 1usize;
    let mut at = start + 1;
    while let Some(&unit) = units.get(at) {
        if unit == 0x5c {
            units.get(at + 1)?;
            at += 2;
            continue;
        }
        if unit == 0x5b {
            depth = depth.checked_add(1)?;
        } else if unit == 0x5d {
            depth -= 1;
        }
        at += 1;
        if depth == 0 {
            return Some(at);
        }
    }
    None
}
// The closest class ancestor owning a bar is the only possible two-way hole.
// Class data stays opaque. All remaining outside terms are proved separately.
fn choice_hole(units: &[u16]) -> Option<Hole> {
    let (mut at, mut captures, mut class_seen) = (0usize, 0usize, false);
    let mut frames = Vec::<HoleFrame>::new();
    let mut hole = None;
    while let Some(&unit) = units.get(at) {
        match unit {
            0x5c => {
                units.get(at + 1)?;
                at += 2;
            }
            0x5b => {
                if class_seen {
                    return None;
                }
                at = class_end(units, at)?;
                class_seen = true;
                for frame in &mut frames {
                    frame.class = true;
                }
            }
            0x28 => {
                let width = if units.get(at + 1) == Some(&0x3f) {
                    if units.get(at + 2) != Some(&0x3a) {
                        return None;
                    }
                    3
                } else {
                    captures = captures.checked_add(1)?;
                    1
                };
                at += width;
                frames.push(HoleFrame {
                    body: at,
                    before: captures,
                    class: false,
                    bars: 0,
                });
            }
            0x7c => {
                let frame = frames.last_mut()?;
                frame.bars = frame.bars.checked_add(1)?;
                at += 1;
            }
            0x29 => {
                let frame = frames.pop()?;
                if frame.class && frame.bars != 0 && hole.is_none() {
                    if frame.bars != 1 {
                        return None;
                    }
                    hole = Some(Hole {
                        body: frame.body..at,
                        before: frame.before,
                    });
                }
                at += 1;
            }
            _ => at += 1,
        }
    }
    (class_seen && frames.is_empty()).then_some(hole).flatten()
}
#[derive(Default)]
struct Gates {
    start_line: bool,
    end_line: bool,
    start_end_line: bool,
    end_start_line: bool,
    start_words: u8,
    end_words: u8,
}
fn outside_gates(units: &[u16]) -> Option<Gates> {
    let mut gates = Gates::default();
    let (mut at, mut class_seen) = (0usize, false);
    while let Some(&unit) = units.get(at) {
        match unit {
            0x5b => {
                if class_seen {
                    return None;
                }
                at = class_end(units, at)?;
                class_seen = true;
            }
            0x5e => {
                if class_seen {
                    gates.end_start_line = true;
                } else {
                    gates.start_line = true;
                }
                at += 1;
            }
            0x24 => {
                if class_seen {
                    gates.end_line = true;
                } else {
                    gates.start_end_line = true;
                }
                at += 1;
            }
            0x5c => {
                let word = *units.get(at + 1)?;
                if matches!(word, 0x62 | 0x42) {
                    let bits = if word == 0x62 { 1 } else { 2 };
                    if class_seen {
                        gates.end_words |= bits;
                    } else {
                        gates.start_words |= bits;
                    }
                }
                at += 2;
            }
            _ => at += 1,
        }
    }
    class_seen.then_some(gates)
}
