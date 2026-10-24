//! Capture projection for UnicodeSets character-class repetition.

use super::{
    RegExpUnicodeRepeatedAtomMatch, RegExpUnicodeRepeatedAtomMatcher, escaped, quantifier,
};
use crate::{JsString, RegExpUnicodeSetCompileError};
use std::ops::Range;

#[path = "regexp_unicode_nullable_string_class_captures.rs"]
mod string_class_captures;
pub use string_class_captures::{
    RegExpUnicodeAssertedCountedNullableScalarStringClassCaptureMatcher,
    RegExpUnicodeAssertedCountedNullableStringClassCaptureMatcher,
    RegExpUnicodeAssertedNullableStringClassCaptureMatcher,
    RegExpUnicodeAssertedRepeatedStringClassCaptureMatcher,
    RegExpUnicodeAssertedStringClassAlternativeMatcher,
    RegExpUnicodeAssertedTwoRequiredNullableStringClassCaptureMatcher,
    RegExpUnicodeBoundedStringClassCaptureMatcher,
    RegExpUnicodeCountedNullableStringClassCaptureMatcher,
    RegExpUnicodeEmptyStringClassCaptureMatcher, RegExpUnicodeMatchError,
    RegExpUnicodeNullableStringClassCaptureMatcher,
    RegExpUnicodePrefixFreeStringClassCaptureMatcher,
    RegExpUnicodeRepeatedStringClassCaptureMatcher, RegExpUnicodeRepeatedStringClassMatch,
    RegExpUnicodeRequiredStringClassCaptureMatcher, RegExpUnicodeSingleStringClassCaptureMatcher,
    RegExpUnicodeStringClassBranchMatch,
    RegExpUnicodeSurroundingAssertedStringClassAlternativeMatcher,
    RegExpUnicodeTwoRequiredStringClassCaptureMatcher,
    RegExpUnicodeUnassertedZeroCountStringClassCaptureMatcher,
    RegExpUnicodeZeroCountAssertedStringClassCaptureMatcher,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CaptureRole {
    Iteration,
    Character,
    Empty,
    Never,
    StartPoint,
    EndPoint,
    CharacterStartPoint,
    CharacterEndPoint,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum CaptureProof {
    Raw,
    EmptyCaptures,
    NoncapturingSequence,
    QuantifiedSequence,
    EmptyChoices,
    EmptyCaptureEffects,
    BoundaryCaptures,
    RepeatedClass,
    BranchCaptures,
    CompleteCaptures,
    RootAlternative,
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
    quantified_empty: bool,
    choice_empty: bool,
    boundary_captures: bool,
    captures: Vec<CaptureRole>,
}

/// A nullable UnicodeSets class repetition with flat branch and boundary captures.
///
/// Complete wrappers capture the last iteration, including required empty
/// iterations. Groups confined to the character branch are undefined when the
/// last iteration selects the empty branch. Empty siblings retain their
/// conditional or unconditional start/end points. Construction, clone and drop
/// use flat source-sized storage; matching reuses the existing linear matcher.
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

    /// Proves character captures beside quantified noncapturing empty groups.
    ///
    /// At least one group on the empty side has a validated greedy/lazy simple
    /// or braced count; groups may nest or form sequences. The empty side has no
    /// captures, choices, assertions or consuming terms, so every selected empty
    /// branch has the same result regardless of inner count spelling or size.
    /// Character-side groups remain unquantified transparent wrappers. All outer
    /// counts, gates and capture requirements match
    /// `compile_with_noncapturing_empty_branch_and_work` (22.2.2.3).
    /// https://262.ecma-international.org/17.0/#sec-repeatmatcher
    pub fn compile_with_quantified_noncapturing_empty_branch_and_work<E>(
        source: &JsString,
        ignore_case: bool,
        multiline: bool,
        charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, RegExpUnicodeSetCompileError<E>> {
        Self::compile_capture_proof(
            source,
            ignore_case,
            multiline,
            CaptureProof::QuantifiedSequence,
            charge,
        )
    }

    /// Proves character captures beside effect-free noncapturing empty choices.
    ///
    /// The empty side contains at least one choice in a noncapturing group, with
    /// optional nesting, siblings and validated simple/braced greedy/lazy counts.
    /// Every choice is empty and contributes no captures or assertions. There is
    /// exactly one class-versus-empty separator; payload bars inside the whole
    /// validated class do not participate in this proof. Character-side groups
    /// remain unquantified transparent wrappers. Other requirements match
    /// `compile_with_noncapturing_empty_branch_and_work` (22.2.2.3).
    /// https://262.ecma-international.org/17.0/#sec-repeatmatcher
    pub fn compile_with_noncapturing_empty_choices_and_work<E>(
        source: &JsString,
        ignore_case: bool,
        multiline: bool,
        charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, RegExpUnicodeSetCompileError<E>> {
        Self::compile_capture_proof(
            source,
            ignore_case,
            multiline,
            CaptureProof::EmptyChoices,
            charge,
        )
    }

    /// Proves captures in an empty branch containing quantifiers or choices.
    ///
    /// At least one empty-only capture and one empty-side quantifier or choice
    /// are required. Every empty alternative contains only groups and validated
    /// counts. Assertions, references, consuming terms and character-side empty
    /// groups/quantifiers retain separate proofs. Character-only captures are
    /// optional. Other outer-count and gate requirements match `compile_with_work`.
    ///
    /// Optional empty iterations fail the progress check, so their captures never
    /// participate. Required iterations select the first empty alternative and
    /// clear all atom captures before each iteration (RepeatMatcher, 22.2.2.3).
    /// Source-sized interval masks resolve these roles without expanding counts.
    /// https://262.ecma-international.org/17.0/#sec-repeatmatcher
    pub fn compile_with_empty_capture_effects_and_work<E>(
        source: &JsString,
        ignore_case: bool,
        multiline: bool,
        charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, RegExpUnicodeSetCompileError<E>> {
        Self::compile_capture_proof(
            source,
            ignore_case,
            multiline,
            CaptureProof::EmptyCaptureEffects,
            charge,
        )
    }

    /// Proves empty captures before or after the class or its nullable alternative.
    ///
    /// At least one such boundary capture is required. Empty sibling groups may
    /// nest, repeat or contain choices, but have no assertions, references or
    /// consuming terms. Character wrappers remain unquantified, and exactly one
    /// alternative separates the class from an empty branch. Other outer-count
    /// and gate requirements match `compile_with_work`.
    ///
    /// Siblings outside the alternative run on either branch; empty siblings on
    /// the character branch participate only when it consumes. Their ranges are
    /// the selected iteration's start/end points. Zero-minimum groups and later
    /// empty alternatives remain undefined (RepeatMatcher, 22.2.2.3).
    /// https://262.ecma-international.org/17.0/#sec-repeatmatcher
    pub fn compile_with_boundary_empty_captures_and_work<E>(
        source: &JsString,
        ignore_case: bool,
        multiline: bool,
        charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, RegExpUnicodeSetCompileError<E>> {
        Self::compile_capture_proof(
            source,
            ignore_case,
            multiline,
            CaptureProof::BoundaryCaptures,
            charge,
        )
    }

    /// Proves empty point captures beside one unquantified nullable class group.
    ///
    /// Requires validated v/vi input and normalized capture names. The group
    /// contains exactly one character-class/empty alternative; empty siblings may
    /// precede/follow it without a complete enclosing wrapper. At least one
    /// conditional or unconditional boundary capture is required. Empty groups
    /// may contain validated counts/choices; assertions, references, additional
    /// consuming terms and class-containing inner counts retain separate proofs.
    /// Root alternatives decline so outside line/word gates keep their original
    /// whole-pattern scope. The body delegates exactly one required iteration;
    /// original lexical captures and first-branch priority are preserved.
    /// https://262.ecma-international.org/17.0/#sec-repeatmatcher
    pub fn compile_single_with_boundary_empty_captures_and_work<E>(
        source: &JsString,
        ignore_case: bool,
        multiline: bool,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, RegExpUnicodeSetCompileError<E>> {
        charge(source.len().saturating_mul(4).saturating_add(32))
            .map_err(RegExpUnicodeSetCompileError::Work)?;
        if has_root_alternative(source.code_units()) {
            return Ok(None);
        }
        let Some(canonical) = single_capture_source(source, &mut charge)? else {
            return Ok(None);
        };
        Self::compile_with_boundary_empty_captures_and_work(
            &canonical,
            ignore_case,
            multiline,
            charge,
        )
    }

    /// Proves captures confined to branches of one unquantified nullable group.
    ///
    /// Requires validated v/vi input and normalized capture names. At least one
    /// character-only or empty-only capture is required; optional/later empty
    /// captures may always be undefined. Empty groups may nest, repeat or contain
    /// choices. Captured empty siblings outside the empty branch retain the
    /// separate single boundary-capture proof. Original complete wrappers are
    /// optional, root alternatives decline, and outside word/line gates preserve
    /// whole-pattern scope. References, internal assertions, further consuming
    /// terms and class-containing inner counts retain separate proofs.
    ///
    /// Exactly one required iteration selects the first successful branch and
    /// clears its captures (RepeatMatcher, 22.2.2.3).
    /// https://262.ecma-international.org/17.0/#sec-repeatmatcher
    pub fn compile_single_branch_captures_with_work<E>(
        source: &JsString,
        ignore_case: bool,
        multiline: bool,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, RegExpUnicodeSetCompileError<E>> {
        charge(source.len().saturating_mul(4).saturating_add(32))
            .map_err(RegExpUnicodeSetCompileError::Work)?;
        if has_root_alternative(source.code_units()) {
            return Ok(None);
        }
        let Some(canonical) = single_capture_source(source, &mut charge)? else {
            return Ok(None);
        };
        Self::compile_capture_proof(
            &canonical,
            ignore_case,
            multiline,
            CaptureProof::BranchCaptures,
            charge,
        )
    }

    /// Proves one unquantified nullable class group without branch-local captures.
    ///
    /// Requires validated v/vi input and normalized capture names. There may be
    /// no captures, or complete enclosing captures only. Capture-free empty terms
    /// may nest, repeat or contain choices beside the class or empty branch.
    /// Branch-local/boundary captures retain separate proofs. Root alternatives,
    /// internal assertions, references, further consuming terms and class-bearing
    /// inner counts decline; outside word/line gates retain whole-pattern scope.
    /// Strict character-class checks apply even when the empty branch comes first.
    /// Exactly one required iteration preserves original first-branch selection
    /// and whole-range captures, including empty matches (22.2.2.3).
    /// https://262.ecma-international.org/17.0/#sec-repeatmatcher
    pub fn compile_single_complete_with_work<E>(
        source: &JsString,
        ignore_case: bool,
        multiline: bool,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, RegExpUnicodeSetCompileError<E>> {
        charge(source.len().saturating_mul(4).saturating_add(32))
            .map_err(RegExpUnicodeSetCompileError::Work)?;
        if has_root_alternative(source.code_units()) {
            return Ok(None);
        }
        let Some(canonical) = single_capture_source(source, &mut charge)? else {
            return Ok(None);
        };
        Self::compile_capture_proof(
            &canonical,
            ignore_case,
            multiline,
            CaptureProof::CompleteCaptures,
            charge,
        )
    }

    /// Proves one ungrouped class-versus-empty alternative with optional captures.
    ///
    /// Requires validated v/vi input and normalized capture names, and at least
    /// one root alternative. Exactly one branch contains one checked character
    /// class; the other branch contains only empty groups, counts or choices.
    /// Original character, empty, conditional boundary and undefined captures
    /// retain their lexical slots. Zero original captures are supported.
    ///
    /// The entire original source is wrapped before compilation. No leading or
    /// trailing assertion is extracted or moved across the alternative. Internal
    /// and branch-local assertions remain separate proofs, as do references,
    /// additional consuming terms, multiple class alternatives and class-bearing
    /// inner counts. One required iteration preserves first-branch priority and
    /// UTF-16 ranges (22.2.2 and 22.2.2.3).
    /// https://262.ecma-international.org/17.0/#sec-patterns
    pub fn compile_single_root_alternative_with_work<E>(
        source: &JsString,
        ignore_case: bool,
        multiline: bool,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, RegExpUnicodeSetCompileError<E>> {
        let units = source.code_units();
        charge(units.len().saturating_mul(8).saturating_add(32))
            .map_err(RegExpUnicodeSetCompileError::Work)?;
        if !has_root_alternative(units) {
            return Ok(None);
        }
        let mut canonical = Vec::with_capacity(units.len().saturating_add(7));
        canonical.extend_from_slice(&[0x28, 0x3f, 0x3a]);
        canonical.extend_from_slice(units);
        canonical.extend_from_slice(&[0x29, 0x7b, 0x31, 0x7d]);
        Self::compile_capture_proof(
            &JsString::from_code_units(canonical),
            ignore_case,
            multiline,
            CaptureProof::RootAlternative,
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
        let factor = match proof {
            CaptureProof::BoundaryCaptures
            | CaptureProof::RepeatedClass
            | CaptureProof::BranchCaptures
            | CaptureProof::CompleteCaptures
            | CaptureProof::RootAlternative => 80,
            CaptureProof::EmptyCaptureEffects => 64,
            CaptureProof::EmptyChoices => 40,
            CaptureProof::QuantifiedSequence => 32,
            _ => 16,
        };
        charge(units.len().saturating_mul(factor)).map_err(RegExpUnicodeSetCompileError::Work)?;
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
            let Some(parsed) = parse_body(
                &body[..atom_end],
                proof != CaptureProof::Raw,
                matches!(
                    proof,
                    CaptureProof::QuantifiedSequence
                        | CaptureProof::EmptyChoices
                        | CaptureProof::EmptyCaptureEffects
                        | CaptureProof::BoundaryCaptures
                        | CaptureProof::RepeatedClass
                        | CaptureProof::BranchCaptures
                        | CaptureProof::CompleteCaptures
                        | CaptureProof::RootAlternative
                ),
                matches!(
                    proof,
                    CaptureProof::EmptyChoices
                        | CaptureProof::EmptyCaptureEffects
                        | CaptureProof::BoundaryCaptures
                        | CaptureProof::RepeatedClass
                        | CaptureProof::BranchCaptures
                        | CaptureProof::CompleteCaptures
                        | CaptureProof::RootAlternative
                ),
                matches!(
                    proof,
                    CaptureProof::EmptyCaptureEffects
                        | CaptureProof::BoundaryCaptures
                        | CaptureProof::RepeatedClass
                        | CaptureProof::BranchCaptures
                        | CaptureProof::CompleteCaptures
                        | CaptureProof::RootAlternative
                ),
                matches!(
                    proof,
                    CaptureProof::BoundaryCaptures
                        | CaptureProof::RepeatedClass
                        | CaptureProof::BranchCaptures
                        | CaptureProof::CompleteCaptures
                        | CaptureProof::RootAlternative
                ),
                proof == CaptureProof::RepeatedClass,
            ) else {
                continue;
            };
            let required = if proof == CaptureProof::EmptyCaptures {
                CaptureRole::Empty
            } else {
                CaptureRole::Character
            };
            let present = if proof == CaptureProof::RootAlternative {
                true
            } else if proof == CaptureProof::CompleteCaptures {
                !parsed.boundary_captures
                    && parsed
                        .captures
                        .iter()
                        .all(|role| *role == CaptureRole::Iteration)
            } else if proof == CaptureProof::BranchCaptures {
                !parsed.boundary_captures
                    && parsed.captures.iter().any(|role| {
                        matches!(
                            role,
                            CaptureRole::Character | CaptureRole::Empty | CaptureRole::Never
                        )
                    })
            } else if matches!(
                proof,
                CaptureProof::BoundaryCaptures | CaptureProof::RepeatedClass
            ) {
                parsed.boundary_captures
            } else if proof == CaptureProof::EmptyCaptureEffects {
                parsed
                    .captures
                    .iter()
                    .any(|role| matches!(role, CaptureRole::Empty | CaptureRole::Never))
            } else {
                parsed.captures.contains(&required)
            };
            if !present
                || (matches!(
                    proof,
                    CaptureProof::NoncapturingSequence
                        | CaptureProof::QuantifiedSequence
                        | CaptureProof::EmptyChoices
                ) && (!parsed.grouped_empty || parsed.captures.contains(&CaptureRole::Empty)))
                || (proof == CaptureProof::QuantifiedSequence && !parsed.quantified_empty)
                || (proof == CaptureProof::EmptyChoices && !parsed.choice_empty)
                || (proof == CaptureProof::EmptyCaptureEffects
                    && !parsed.choice_empty
                    && !parsed.quantified_empty)
            {
                continue;
            }
            let class = &body[parsed.class];
            let consuming = proof == CaptureProof::RepeatedClass;
            let length = begin
                + class.len()
                + (body.len() - atom_end)
                + (units.len() - end)
                + if consuming { 2 } else { 7 };
            // One copy plus JsString's code-unit storage conversion. Counts and
            // source spellings are retained without expansion or normalization.
            charge(length.saturating_mul(2)).map_err(RegExpUnicodeSetCompileError::Work)?;
            let mut canonical = Vec::with_capacity(length);
            canonical.extend_from_slice(&units[..begin]);
            canonical.push(0x28);
            if !consuming && parsed.empty_first {
                canonical.extend_from_slice(&[0x28, 0x3f, 0x3a, 0x29, 0x7c]);
            }
            canonical.extend_from_slice(class);
            if !consuming && !parsed.empty_first {
                canonical.extend_from_slice(&[0x7c, 0x28, 0x3f, 0x3a, 0x29]);
            }
            canonical.push(0x29);
            canonical.extend_from_slice(&body[atom_end..]);
            canonical.extend_from_slice(&units[end..]);
            debug_assert_eq!(canonical.len(), length);
            let canonical = JsString::from_code_units(canonical);
            let counted = body.get(atom_end) == Some(&0x7b);
            let singleton = if consuming {
                RegExpUnicodeRepeatedAtomMatcher::compile_wrapped_v_set_with_single_character_strings_and_work(&canonical,ignore_case,multiline,&mut charge)?
            } else if counted {
                RegExpUnicodeRepeatedAtomMatcher::compile_counted_noncapturing_empty_sequence_nullable_v_set_with_single_character_strings_and_work(&canonical,ignore_case,multiline,&mut charge)?
            } else {
                RegExpUnicodeRepeatedAtomMatcher::compile_noncapturing_empty_sequence_nullable_v_set_with_single_character_strings_and_work(&canonical,ignore_case,multiline,&mut charge)?
            };
            let matcher = if singleton.is_some() {
                singleton
            } else if consuming {
                RegExpUnicodeRepeatedAtomMatcher::compile_wrapped_character_result_v_set_with_work(
                    &canonical,
                    ignore_case,
                    multiline,
                    &mut charge,
                )?
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
        projected_capture_range(&self.captures, index, iteration)
    }
}

/// A consuming UnicodeSets class repetition with empty point captures.
///
/// Empty sequences may occur before or after the sole class, including nested
/// captures, choices and validated greedy/lazy counts. At least one empty
/// capture is required. Every selected outer iteration consumes one character;
/// captures use its UTF-16 start/end, or remain undefined after zero iterations.
/// Storage, clone and drop remain flat and proportional to the source.
#[derive(Clone, Debug)]
pub struct RegExpUnicodeRepeatedClassCaptureMatcher {
    matcher: RegExpUnicodeNullableClassCaptureMatcher,
}

impl RegExpUnicodeRepeatedClassCaptureMatcher {
    /// Proves one consuming class with empty captures and no outer quantifier.
    ///
    /// Requires validated v/vi input and normalized capture names. Empty groups
    /// may precede/follow the class without complete enclosing wrappers. Outside
    /// line/word gates keep whole-pattern scope. This proof wraps only the body
    /// and delegates an exact single iteration; source counts are never expanded.
    /// Class-containing choices or counts, assertions, references, further
    /// consuming terms and unknown/string-valued leaves retain separate proofs.
    /// https://262.ecma-international.org/17.0/#sec-patterns
    pub fn compile_single_with_work<E>(
        source: &JsString,
        ignore_case: bool,
        multiline: bool,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, RegExpUnicodeSetCompileError<E>> {
        let Some(canonical) = single_capture_source(source, &mut charge)? else {
            return Ok(None);
        };
        Self::compile_with_work(&canonical, ignore_case, multiline, charge)
    }

    /// Proves a character class with empty sibling captures and one outer count.
    ///
    /// Requires validated v/vi input and normalized capture names. The class
    /// must have singleton q operands or a statically character-only result;
    /// unknown/string-valued leaves decline even under zero counts. Canonical
    /// outside word/line gates and enclosing captures retain their existing
    /// caller proofs. Class-containing choices/inner counts, assertions,
    /// references and additional consuming terms remain separate proofs.
    ///
    /// RepeatMatcher clears captures before each iteration. Optional empty
    /// subgroups fail its progress check; required empty iterations select the
    /// first successful alternative (22.2.2.3).
    /// https://262.ecma-international.org/17.0/#sec-repeatmatcher
    pub fn compile_with_work<E>(
        source: &JsString,
        ignore_case: bool,
        multiline: bool,
        charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, RegExpUnicodeSetCompileError<E>> {
        RegExpUnicodeNullableClassCaptureMatcher::compile_capture_proof(
            source,
            ignore_case,
            multiline,
            CaptureProof::RepeatedClass,
            charge,
        )
        .map(|plan| plan.map(|matcher| Self { matcher }))
    }

    /// Number of original captures inside the repeated atom.
    pub fn capture_count(&self) -> usize {
        self.matcher.capture_count()
    }

    /// Finds the repetition and its last consuming iteration.
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

    /// Projects a zero-based original capture from the final iteration.
    pub fn capture_range(
        &self,
        index: usize,
        iteration: Option<&Range<usize>>,
    ) -> Option<Range<usize>> {
        self.matcher.capture_range(index, iteration)
    }
}

fn projected_capture_range(
    captures: &[CaptureRole],
    index: usize,
    iteration: Option<&Range<usize>>,
) -> Option<Range<usize>> {
    let role = captures.get(index)?;
    let range = iteration?;
    match role {
        CaptureRole::Iteration => Some(range.clone()),
        CaptureRole::Character => (!range.is_empty()).then(|| range.clone()),
        CaptureRole::Empty => range.is_empty().then(|| range.clone()),
        CaptureRole::Never => None,
        CaptureRole::StartPoint => Some(range.start..range.start),
        CaptureRole::EndPoint => Some(range.end..range.end),
        CaptureRole::CharacterStartPoint => (!range.is_empty()).then_some(range.start..range.start),
        CaptureRole::CharacterEndPoint => (!range.is_empty()).then_some(range.end..range.end),
    }
}

fn projected_selected_capture_range(
    captures: &[CaptureRole],
    index: usize,
    range: &Range<usize>,
    class_selected: bool,
) -> Option<Range<usize>> {
    match captures.get(index)? {
        CaptureRole::Iteration => Some(range.clone()),
        CaptureRole::Character => class_selected.then(|| range.clone()),
        CaptureRole::Empty => (!class_selected).then(|| range.clone()),
        CaptureRole::Never => None,
        CaptureRole::StartPoint => Some(range.start..range.start),
        CaptureRole::EndPoint => Some(range.end..range.end),
        CaptureRole::CharacterStartPoint => class_selected.then_some(range.start..range.start),
        CaptureRole::CharacterEndPoint => class_selected.then_some(range.end..range.end),
    }
}

// Preserve outside framing while adding one exact, noncapturing iteration.
fn single_capture_source<E>(
    source: &JsString,
    charge: &mut impl FnMut(usize) -> Result<(), E>,
) -> Result<Option<JsString>, RegExpUnicodeSetCompileError<E>> {
    let units = source.code_units();
    // Precharge outside edge scans and both source copies before allocation.
    charge(units.len().saturating_mul(8).saturating_add(32))
        .map_err(RegExpUnicodeSetCompileError::Work)?;
    let mut begin = usize::from(units.first() == Some(&0x5e));
    let mut end =
        units.len() - usize::from(units.last() == Some(&0x24) && !escaped(units, units.len() - 1));
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
    let mut canonical = Vec::with_capacity(units.len().saturating_add(7));
    canonical.extend_from_slice(&units[..begin]);
    canonical.extend_from_slice(&[0x28, 0x3f, 0x3a]);
    canonical.extend_from_slice(body);
    canonical.extend_from_slice(&[0x29, 0x7b, 0x31, 0x7d]);
    canonical.extend_from_slice(&units[end..]);
    Ok(Some(JsString::from_code_units(canonical)))
}

// A leading/trailing assertion in an unwrapped disjunction belongs to only one
// branch. Keep root alternatives closed before moving canonical outside gates.
fn has_root_alternative(units: &[u16]) -> bool {
    let (mut group, mut class, mut at) = (0usize, 0usize, 0usize);
    while at < units.len() {
        match units[at] {
            0x5c => {
                at += 2;
                continue;
            }
            0x5b => class += 1,
            0x5d if class != 0 => class -= 1,
            0x28 if class == 0 => group += 1,
            0x29 if class == 0 => group = group.saturating_sub(1),
            0x7c if class == 0 && group == 0 => return true,
            _ => {}
        }
        at += 1;
    }
    false
}

// Groups containing the class capture either the whole iteration or only the
// character. The separate empty-capture proof also accepts wholly empty groups
// on the opposite side of the separator. Additional consuming alternatives and
// other terms retain separate proofs.
fn parse_body(
    units: &[u16],
    empty_captures: bool,
    quantifiers: bool,
    choices: bool,
    effects: bool,
    boundary: bool,
    consuming: bool,
) -> Option<Parsed> {
    if units.first() != Some(&0x28) || units.last() != Some(&0x29) {
        return None;
    }
    let mut frames = Vec::<Frame>::new();
    let mut captures = Vec::new();
    let mut completed = Vec::new();
    let mut capture_starts = Vec::new();
    let mut group_ends = if effects {
        vec![0; units.len()]
    } else {
        Vec::new()
    };
    let mut disabled_ends = if effects {
        vec![0; units.len()]
    } else {
        Vec::new()
    };
    let mut class = None::<Range<usize>>;
    let mut separator = None;
    let mut separator_owner = None;
    let mut bars = Vec::new();
    // Active ancestors at the sole class identify separator-owning groups.
    // Earlier constructors allocate no marker storage and keep their probes.
    let mut class_owners = if choices {
        vec![false; units.len()]
    } else {
        Vec::new()
    };
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
                    if effects {
                        capture_starts.push(start);
                    }
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
                if choices {
                    for frame in &frames {
                        class_owners[frame.start] = true;
                    }
                }
                class = Some(start..at);
            }
            0x7c => {
                if choices {
                    bars.push((at, frames.last()?.start));
                } else {
                    let owner = frames.last()?.start;
                    if separator.replace(at).is_some() {
                        return None;
                    }
                    separator_owner = Some(owner);
                }
                at += 1;
            }
            0x29 => {
                let frame = frames.pop()?;
                if effects {
                    group_ends[frame.start] = at;
                }
                if empty_captures {
                    // An empty-first group can close before the class is seen.
                    // Retain flat spans until both branches are known.
                    completed.push((frame, at, false, false));
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
                if quantifiers {
                    let (end, zero_minimum) = empty_group_quantifier(units, at)?;
                    if end != at {
                        let group = completed.last_mut()?;
                        group.2 = true;
                        group.3 = zero_minimum;
                        at = end;
                    }
                }
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
    let mut choice_empty = false;
    if choices {
        // A bar owned by a class ancestor separates the consuming and empty
        // branches. Every other bar must lie wholly on the empty side.
        for &(at, owner) in &bars {
            if class_owners[owner] {
                if consuming || separator.replace(at).is_some() {
                    return None;
                }
                separator_owner = Some(owner);
            }
        }
        let split = if consuming { None } else { Some(separator?) };
        for (at, owner) in bars {
            if !class_owners[owner] {
                if !boundary {
                    let split = split?;
                    if (split < class.start) != (at < split) {
                        return None;
                    }
                }
                choice_empty = true;
                if effects {
                    // Later empty alternatives cannot improve an input-only
                    // continuation. Their groups never enter the first success.
                    disabled_ends[at + 1] = disabled_ends[at + 1].max(group_ends[owner] + 1);
                }
            }
        }
    }
    let split = if consuming { None } else { Some(separator?) };
    let owner_bounds = separator_owner.and_then(|start| {
        completed
            .iter()
            .find_map(|(frame, end, _, _)| (frame.start == start).then_some((start, *end)))
    });
    let mut grouped_empty = false;
    let mut quantified_empty = false;
    let mut boundary_captures = false;
    for (frame, end, quantified, zero_minimum) in completed {
        let contains_class = frame.start < class.start && class.end <= end;
        let contains_split = split.is_some_and(|split| frame.start < split && split < end);
        let role = if contains_class {
            if quantified {
                return None;
            }
            if contains_split {
                CaptureRole::Iteration
            } else {
                CaptureRole::Character
            }
        } else {
            if contains_split {
                return None;
            }
            let role = if consuming {
                if end < class.start {
                    CaptureRole::StartPoint
                } else if frame.start >= class.end {
                    CaptureRole::EndPoint
                } else {
                    return None;
                }
            } else {
                let split = split?;
                let (owner_start, owner_end) = owner_bounds?;
                let inside_owner = frame.start > owner_start && end < owner_end;
                let on_empty_side = if split < class.start {
                    end < split
                } else {
                    frame.start > split
                };
                if inside_owner && on_empty_side {
                    CaptureRole::Empty
                } else if boundary {
                    if end < owner_start {
                        CaptureRole::StartPoint
                    } else if frame.start > owner_end {
                        CaptureRole::EndPoint
                    } else if end < class.start {
                        CaptureRole::CharacterStartPoint
                    } else if frame.start >= class.end {
                        CaptureRole::CharacterEndPoint
                    } else {
                        return None;
                    }
                } else {
                    if !on_empty_side || (frame.capture.is_some() && !inside_owner) {
                        return None;
                    }
                    // Capture-free epsilon siblings remain transparent.
                    CaptureRole::Empty
                }
            };
            if frame.capture.is_some()
                && matches!(
                    role,
                    CaptureRole::StartPoint
                        | CaptureRole::EndPoint
                        | CaptureRole::CharacterStartPoint
                        | CaptureRole::CharacterEndPoint
                )
            {
                boundary_captures = true;
            }
            grouped_empty = true;
            quantified_empty |= quantified;
            if effects && zero_minimum {
                disabled_ends[frame.start] = disabled_ends[frame.start].max(end + 1);
            }
            role
        };
        if let Some(index) = frame.capture {
            captures[index] = role;
        }
    }
    if effects {
        let mut cursor = 0;
        let mut disabled_end = 0;
        for (index, start) in capture_starts.into_iter().enumerate() {
            while cursor <= start {
                disabled_end = disabled_end.max(disabled_ends[cursor]);
                cursor += 1;
            }
            if start < disabled_end {
                debug_assert!(
                    captures[index] == CaptureRole::Empty
                        || (boundary
                            && matches!(
                                captures[index],
                                CaptureRole::StartPoint
                                    | CaptureRole::EndPoint
                                    | CaptureRole::CharacterStartPoint
                                    | CaptureRole::CharacterEndPoint
                            ))
                );
                captures[index] = CaptureRole::Never;
            }
        }
    }
    Some(Parsed {
        empty_first: split.is_some_and(|split| split < class.start),
        grouped_empty,
        quantified_empty,
        choice_empty,
        boundary_captures,
        class,
        captures,
    })
}

// Validated counts are scanned once forward and checked by the existing borrowed
// quantifier parser. Their numeric magnitudes never expand source or state.
fn empty_group_quantifier(units: &[u16], start: usize) -> Option<(usize, bool)> {
    let mut end = match units.get(start) {
        Some(0x2a | 0x2b | 0x3f) => start + 1,
        Some(0x7b) => {
            let mut at = start + 1;
            while matches!(units.get(at), Some(0x30..=0x39 | 0x2c)) {
                at += 1;
            }
            if units.get(at) != Some(&0x7d) {
                return None;
            }
            at + 1
        }
        _ => return Some((start, false)),
    };
    let (atom_end, minimum, _) = quantifier(units, end)?;
    if atom_end != start {
        return None;
    }
    if units.get(end) == Some(&0x3f) {
        end += 1;
    }
    Some((end, minimum == Some(0)))
}

#[cfg(test)]
mod tests {
    include!("regexp_unicode_nullable_class_capture_tests.rs");
    include!("regexp_unicode_nullable_boundary_capture_tests.rs");
    include!("regexp_unicode_repeated_class_capture_tests.rs");
    include!("regexp_unicode_single_class_capture_tests.rs");
    include!("regexp_unicode_single_nullable_boundary_capture_tests.rs");
    include!("regexp_unicode_single_nullable_branch_capture_tests.rs");
    include!("regexp_unicode_single_nullable_complete_capture_tests.rs");
    include!("regexp_unicode_single_root_nullable_capture_tests.rs");
    include!("regexp_unicode_nonempty_string_root_capture_tests.rs");
    include!("regexp_unicode_empty_string_root_capture_tests.rs");
    include!("regexp_unicode_grouped_string_capture_tests.rs");
    include!("regexp_unicode_single_string_class_capture_tests.rs");
    include!("regexp_unicode_asserted_single_string_class_capture_tests.rs");
    include!("regexp_unicode_enclosed_asserted_single_string_class_capture_tests.rs");
    include!("regexp_unicode_surrounding_line_string_class_capture_tests.rs");
    include!("regexp_unicode_surrounding_word_string_class_capture_tests.rs");
    include!("regexp_unicode_nested_assertion_string_class_capture_tests.rs");
    include!("regexp_unicode_unconditional_assertion_string_class_capture_tests.rs");
    include!("regexp_unicode_reversed_line_string_class_capture_tests.rs");
    include!("regexp_unicode_asserted_string_class_root_tests.rs");
    include!("regexp_unicode_grouped_asserted_string_class_tests.rs");
    include!("regexp_unicode_common_line_asserted_string_class_tests.rs");
    include!("regexp_unicode_common_word_asserted_string_class_tests.rs");
    include!("regexp_unicode_layered_asserted_string_class_tests.rs");
    include!("regexp_unicode_reversed_common_line_asserted_string_class_tests.rs");
    include!("regexp_unicode_layered_endpoint_asserted_string_class_tests.rs");
    include!("regexp_unicode_surrounding_asserted_string_class_tests.rs");
    include!("regexp_unicode_exact_single_surrounding_asserted_string_class_tests.rs");
    include!("regexp_unicode_exact_single_root_asserted_string_class_tests.rs");
    include!("regexp_unicode_exact_single_class_spine_tests.rs");
    include!("regexp_unicode_exact_single_unasserted_string_class_tests.rs");
    include!("regexp_unicode_simple_repeated_string_class_tests.rs");
    include!("regexp_unicode_bounded_string_class_tests.rs");
    include!("regexp_unicode_counted_nullable_string_class_tests.rs");
    include!("regexp_unicode_prefix_free_string_class_tests.rs");
    include!("regexp_unicode_two_required_string_class_tests.rs");
    include!("regexp_unicode_required_string_class_tests.rs");
    include!("regexp_unicode_asserted_repeated_string_class_tests.rs");
    include!("regexp_unicode_asserted_nullable_string_class_tests.rs");
    include!("regexp_unicode_asserted_two_required_nullable_string_class_tests.rs");
    include!("regexp_unicode_asserted_counted_nullable_string_class_tests.rs");
    include!("regexp_unicode_asserted_counted_nullable_scalar_string_class_tests.rs");
    include!("regexp_unicode_zero_count_asserted_string_class_tests.rs");
    include!("regexp_unicode_unasserted_zero_count_string_class_tests.rs");
}
