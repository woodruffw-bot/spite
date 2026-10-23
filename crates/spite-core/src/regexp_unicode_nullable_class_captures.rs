//! Capture projection for a nullable UnicodeSets character-class repetition.

use super::{
    RegExpUnicodeRepeatedAtomMatch, RegExpUnicodeRepeatedAtomMatcher, escaped, quantifier,
};
use crate::{JsString, RegExpUnicodeSetCompileError};
use std::ops::Range;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CaptureRole {
    Iteration,
    Character,
    Empty,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum CaptureProof {
    Raw,
    EmptyCaptures,
    NoncapturingSequence,
}

#[derive(Clone, Copy)]
struct Frame {
    start: usize,
    capture: Option<usize>,
}

struct Parsed {
    class: Range<usize>,
    empty_first: bool,
    grouped_empty: bool,
    captures: Vec<CaptureRole>,
}

/// A nullable UnicodeSets class repetition with captures confined to either branch.
///
/// Complete wrappers capture the last iteration, including required empty
/// iterations. Groups confined to the character branch are undefined when the
/// last iteration selects the empty branch. Construction, clone and drop use
/// flat source-sized storage; matching reuses the existing linear matcher.
#[derive(Clone, Debug)]
pub struct RegExpUnicodeNullableClassCaptureMatcher {
    matcher: RegExpUnicodeRepeatedAtomMatcher,
    captures: Box<[CaptureRole]>,
}

impl RegExpUnicodeNullableClassCaptureMatcher {
    /// Proves partial character captures beside one raw empty alternative.
    ///
    /// Requires validated v/vi input and normalized capture names. The repeated
    /// atom is a complete capturing/noncapturing wrapper chain around one bare
    /// singleton-q or character-result class and a raw empty alternative. Groups
    /// on the character branch must be complete transparent wrappers. Simple and
    /// all braced outer counts, canonical outside word/line gates and lazy forms
    /// reuse existing proofs. At least one character-only capture is required.
    /// Empty-branch captures, additional atoms, inner assertions/quantifiers and
    /// additional alternatives retain separate proofs. Enclosing whole-match
    /// groups must be handled separately by the caller.
    ///
    /// RepeatMatcher clears the atom's captures before each iteration; projecting
    /// from the last selected branch therefore preserves required final empty
    /// iterations even after earlier consuming iterations (22.2.2.3).
    /// https://262.ecma-international.org/17.0/#sec-repeatmatcher
    pub fn compile_with_work<E>(
        source: &JsString,
        ignore_case: bool,
        multiline: bool,
        charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, RegExpUnicodeSetCompileError<E>> {
        Self::compile_capture_proof(source, ignore_case, multiline, CaptureProof::Raw, charge)
    }

    /// Proves captures on an unquantified empty branch beside a character class.
    ///
    /// The empty branch contains only balanced capturing/noncapturing groups,
    /// including nested or sibling empty groups. At least one empty-only capture
    /// is required. Complete-atom and character-only captures retain independent
    /// roles. Choices, inner quantifiers/assertions and empty groups on the
    /// character branch retain separate proofs. Other requirements and outside
    /// count/gate semantics are the same as `compile_with_work`.
    ///
    /// RepeatMatcher clears captures before each iteration: selecting an empty
    /// branch defines its groups at the current point and leaves character-only
    /// groups undefined, including after earlier consuming iterations (22.2.2.3).
    /// https://262.ecma-international.org/17.0/#sec-repeatmatcher
    pub fn compile_with_empty_branch_captures_and_work<E>(
        source: &JsString,
        ignore_case: bool,
        multiline: bool,
        charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, RegExpUnicodeSetCompileError<E>> {
        Self::compile_capture_proof(
            source,
            ignore_case,
            multiline,
            CaptureProof::EmptyCaptures,
            charge,
        )
    }

    /// Proves character-only captures beside an unquantified noncapturing empty sequence.
    ///
    /// The empty branch contains a nonempty sequence of balanced noncapturing
    /// groups, including nesting and siblings. At least one character-only
    /// capture is required. Captured empty groups, inner quantifiers/choices,
    /// assertions and empty groups on the character branch retain separate
    /// proofs. All outside count/gate and projection requirements match
    /// `compile_with_work` (RepeatMatcher, 22.2.2.3).
    /// https://262.ecma-international.org/17.0/#sec-repeatmatcher
    pub fn compile_with_noncapturing_empty_branch_and_work<E>(
        source: &JsString,
        ignore_case: bool,
        multiline: bool,
        charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, RegExpUnicodeSetCompileError<E>> {
        Self::compile_capture_proof(
            source,
            ignore_case,
            multiline,
            CaptureProof::NoncapturingSequence,
            charge,
        )
    }

    fn compile_capture_proof<E>(
        source: &JsString,
        ignore_case: bool,
        multiline: bool,
        proof: CaptureProof,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, RegExpUnicodeSetCompileError<E>> {
        let units = source.code_units();
        // Edge parity, both token interpretations, group/class walks and the
        // flat capture table are covered before structural inspection.
        charge(units.len().saturating_mul(16)).map_err(RegExpUnicodeSetCompileError::Work)?;
        let mut begin = usize::from(units.first() == Some(&0x5e));
        let mut end = units.len()
            - usize::from(units.last() == Some(&0x24) && !escaped(units, units.len() - 1));
        if matches!(
            units.get(begin..end).and_then(|body| body.get(..2)),
            Some([0x5c, 0x62 | 0x42])
        ) {
            begin += 2;
        }
        if end >= begin + 2
            && units[end - 2] == 0x5c
            && matches!(units[end - 1], 0x62 | 0x42)
            && !escaped(units, end - 2)
        {
            end -= 2;
        }
        let Some(body) = units.get(begin..end) else {
            return Ok(None);
        };
        for token_end in [
            body.len()
                .checked_sub(1)
                .filter(|_| body.last() == Some(&0x3f)),
            Some(body.len()),
        ] {
            let Some(token_end) = token_end else { continue };
            let Some((atom_end, _, _)) = quantifier(body, token_end) else {
                continue;
            };
            let Some(parsed) = parse_body(&body[..atom_end], proof != CaptureProof::Raw) else {
                continue;
            };
            let required = if proof == CaptureProof::EmptyCaptures {
                CaptureRole::Empty
            } else {
                CaptureRole::Character
            };
            if !parsed.captures.contains(&required)
                || (proof == CaptureProof::NoncapturingSequence
                    && (!parsed.grouped_empty || parsed.captures.contains(&CaptureRole::Empty)))
            {
                continue;
            }
            let class = &body[parsed.class];
            let length = begin + class.len() + (body.len() - atom_end) + (units.len() - end) + 7;
            // One copy plus JsString's code-unit storage conversion. Counts and
            // source spellings are retained without expansion or normalization.
            charge(length.saturating_mul(2)).map_err(RegExpUnicodeSetCompileError::Work)?;
            let mut canonical = Vec::with_capacity(length);
            canonical.extend_from_slice(&units[..begin]);
            canonical.push(0x28);
            if parsed.empty_first {
                canonical.extend_from_slice(&[0x28, 0x3f, 0x3a, 0x29, 0x7c]);
            }
            canonical.extend_from_slice(class);
            if !parsed.empty_first {
                canonical.extend_from_slice(&[0x7c, 0x28, 0x3f, 0x3a, 0x29]);
            }
            canonical.push(0x29);
            canonical.extend_from_slice(&body[atom_end..]);
            canonical.extend_from_slice(&units[end..]);
            debug_assert_eq!(canonical.len(), length);
            let canonical = JsString::from_code_units(canonical);
            let counted = body.get(atom_end) == Some(&0x7b);
            let singleton = if counted {
                RegExpUnicodeRepeatedAtomMatcher::compile_counted_noncapturing_empty_sequence_nullable_v_set_with_single_character_strings_and_work(&canonical,ignore_case,multiline,&mut charge)?
            } else {
                RegExpUnicodeRepeatedAtomMatcher::compile_noncapturing_empty_sequence_nullable_v_set_with_single_character_strings_and_work(&canonical,ignore_case,multiline,&mut charge)?
            };
            let matcher = if singleton.is_some() {
                singleton
            } else if counted {
                RegExpUnicodeRepeatedAtomMatcher::compile_counted_noncapturing_empty_sequence_nullable_character_result_v_set_with_work(&canonical,ignore_case,multiline,&mut charge)?
            } else {
                RegExpUnicodeRepeatedAtomMatcher::compile_noncapturing_empty_sequence_nullable_character_result_v_set_with_work(&canonical,ignore_case,multiline,&mut charge)?
            };
            if let Some(matcher) = matcher {
                // This synthetic complete capture records the final branch. Its
                // zero/nonzero width is unambiguous for a character-only class.
                debug_assert_eq!(matcher.capture_count(), 1);
                return Ok(Some(Self {
                    matcher,
                    captures: parsed.captures.into_boxed_slice(),
                }));
            }
        }
        Ok(None)
    }

    /// Number of original captures inside the repeated atom.
    pub fn capture_count(&self) -> usize {
        self.captures.len()
    }

    /// Finds the repetition and its final iteration for capture projection.
    ///
    /// Use `capture_range` to obtain an original capture slot; `atom_capture`
    /// records the selected iteration even when no character capture participated.
    pub fn find_with_captures_and_work<E>(
        &self,
        input: &JsString,
        start: usize,
        sticky: bool,
        charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<RegExpUnicodeRepeatedAtomMatch>, E> {
        self.matcher
            .find_with_captures_and_work(input, start, sticky, charge)
    }

    /// Projects a zero-based original capture from the selected iteration.
    pub fn capture_range(
        &self,
        index: usize,
        iteration: Option<&Range<usize>>,
    ) -> Option<Range<usize>> {
        let role = self.captures.get(index)?;
        let range = iteration?;
        if match role {
            CaptureRole::Iteration => true,
            CaptureRole::Character => !range.is_empty(),
            CaptureRole::Empty => range.is_empty(),
        } {
            Some(range.clone())
        } else {
            None
        }
    }
}

// Groups containing the class capture either the whole iteration or only the
// character. The separate empty-capture proof also accepts wholly empty groups
// on the opposite side of the separator. No other tokens or alternatives pass.
fn parse_body(units: &[u16], empty_captures: bool) -> Option<Parsed> {
    if units.first() != Some(&0x28) || units.last() != Some(&0x29) {
        return None;
    }
    let mut frames = Vec::<Frame>::new();
    let mut captures = Vec::new();
    let mut completed = Vec::new();
    let mut class = None::<Range<usize>>;
    let mut separator = None;
    let mut at = 0;
    while at < units.len() {
        match units[at] {
            0x28 => {
                let start = at;
                let capture = if units.get(at + 1) == Some(&0x3f) {
                    if units.get(at + 2) != Some(&0x3a) {
                        return None;
                    }
                    at += 3;
                    None
                } else {
                    let index = captures.len();
                    captures.push(CaptureRole::Iteration);
                    at += 1;
                    Some(index)
                };
                frames.push(Frame { start, capture });
            }
            0x5b => {
                if class.is_some() || frames.is_empty() {
                    return None;
                }
                let start = at;
                let mut depth = 1usize;
                at += 1;
                while at < units.len() && depth != 0 {
                    match units[at] {
                        0x5c => {
                            at += 2;
                            continue;
                        }
                        0x5b => depth += 1,
                        0x5d => depth -= 1,
                        _ => {}
                    }
                    at += 1;
                }
                if depth != 0 {
                    return None;
                }
                class = Some(start..at);
            }
            0x7c => {
                if frames.is_empty() || separator.replace(at).is_some() {
                    return None;
                }
                at += 1;
            }
            0x29 => {
                let frame = frames.pop()?;
                if empty_captures {
                    // An empty-first group can close before the class is seen.
                    // Retain flat spans until both branches are known.
                    completed.push((frame, at));
                } else {
                    let span = class.as_ref()?;
                    if frame.start >= span.start || span.end > at {
                        return None;
                    }
                    if let Some(index) = frame.capture {
                        captures[index] =
                            if separator.is_some_and(|split| frame.start < split && split < at) {
                                CaptureRole::Iteration
                            } else {
                                CaptureRole::Character
                            };
                    }
                }
                at += 1;
                if frames.is_empty() && at != units.len() {
                    return None;
                }
            }
            _ => return None,
        }
    }
    if !frames.is_empty() {
        return None;
    }
    let class = class?;
    let split = separator?;
    let mut grouped_empty = false;
    for (frame, end) in completed {
        let contains_class = frame.start < class.start && class.end <= end;
        let contains_split = frame.start < split && split < end;
        let role = if contains_class {
            if contains_split {
                CaptureRole::Iteration
            } else {
                CaptureRole::Character
            }
        } else if !contains_split
            && if split < class.start {
                end < split
            } else {
                frame.start > split
            }
        {
            grouped_empty = true;
            CaptureRole::Empty
        } else {
            return None;
        };
        if let Some(index) = frame.capture {
            captures[index] = role;
        }
    }
    Some(Parsed {
        empty_first: split < class.start,
        grouped_empty,
        class,
        captures,
    })
}

#[cfg(test)]
mod tests {
    include!("regexp_unicode_nullable_class_capture_tests.rs");
}
