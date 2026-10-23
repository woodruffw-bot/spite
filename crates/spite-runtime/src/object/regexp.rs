//! Native RegExp original Pattern and flag slots (22.2).

use super::{Error, Objects};
use spite_core::{
    JsString, RegExpAnchoredMatcher, RegExpBackreferenceMatcher, RegExpCharacterMatcher,
    RegExpDisjunctionMatcher, RegExpLiteralMatcher, RegExpPrefixedMatcher,
    RegExpQuantifiedContinuationMatcher, RegExpQuantifiedMatcher, RegExpRepeatedCaptureMatcher,
    RegExpRepeatedContinuationMatcher, RegExpRepeatedLiteralMatcher, RegExpRepeatedPrefixedMatcher,
    RegExpRepeatedSequenceMatcher, RegExpSequenceMatcher, RegExpUnicodeCharacterMatcher,
    RegExpUnicodeClassLiteralClassMatcher, RegExpUnicodeClassLiteralClassWordMatcher,
    RegExpUnicodeClassLiteralMatcher, RegExpUnicodeClassMatcher,
    RegExpUnicodeClassPairLiteralMatcher, RegExpUnicodeClassPairLiteralWordMatcher,
    RegExpUnicodeClassPairMatcher, RegExpUnicodeClassSequenceMatcher,
    RegExpUnicodeClassTripleMatcher, RegExpUnicodeClassTripleWordMatcher,
    RegExpUnicodeFoldedLiteralMatcher, RegExpUnicodeLiteralClassPairMatcher,
    RegExpUnicodeLiteralClassPairWordMatcher, RegExpUnicodeNullableClassCaptureMatcher,
    RegExpUnicodePropertyDotMatcher, RegExpUnicodeRepeatedAtomMatcher,
    RegExpUnicodeStringSetMatcher,
};
use spite_heap::Handle;
use std::{ops::Range, sync::Arc};

#[derive(Clone, Debug)]
pub(crate) enum RegExpMatcherBody {
    Backreferences(RegExpBackreferenceMatcher),
    Literal(RegExpLiteralMatcher),
    UnicodeEmpty(RegExpLiteralMatcher),
    UnicodeCodePoints(RegExpLiteralMatcher),
    Anchored(RegExpAnchoredMatcher),
    Character(RegExpCharacterMatcher),
    UnicodeCharacter(RegExpUnicodeCharacterMatcher),
    UnicodeClass(RegExpUnicodeClassMatcher),
    UnicodeStringSet(RegExpUnicodeStringSetMatcher),
    UnicodeClassLiteral(RegExpUnicodeClassLiteralMatcher),
    UnicodeClassPair(RegExpUnicodeClassPairMatcher),
    UnicodeLiteralClassPair(RegExpUnicodeLiteralClassPairMatcher),
    UnicodeClassPairLiteral(RegExpUnicodeClassPairLiteralMatcher),
    UnicodeClassLiteralClass(RegExpUnicodeClassLiteralClassMatcher),
    UnicodeClassTriple(RegExpUnicodeClassTripleMatcher),
    UnicodeClassTripleWord(RegExpUnicodeClassTripleWordMatcher),
    UnicodeLiteralClassPairWord(RegExpUnicodeLiteralClassPairWordMatcher),
    UnicodeClassPairLiteralWord(RegExpUnicodeClassPairLiteralWordMatcher),
    UnicodeClassLiteralClassWord(RegExpUnicodeClassLiteralClassWordMatcher),
    UnicodeClassSequence(RegExpUnicodeClassSequenceMatcher),
    UnicodeFoldedLiteral(RegExpUnicodeFoldedLiteralMatcher),
    UnicodeRepeatedAtom(RegExpUnicodeRepeatedAtomMatcher),
    UnicodeNullableClassCaptures(RegExpUnicodeNullableClassCaptureMatcher),
    UnicodePropertyDot(RegExpUnicodePropertyDotMatcher),
    Sequence(RegExpSequenceMatcher),
    Disjunction(RegExpDisjunctionMatcher),
    Quantified(RegExpQuantifiedMatcher),
    QuantifiedContinuation(RegExpQuantifiedContinuationMatcher),
    Prefixed(RegExpPrefixedMatcher),
    RepeatedLiteral(RegExpRepeatedLiteralMatcher),
    RepeatedSequence(RegExpRepeatedSequenceMatcher),
    RepeatedContinuation(RegExpRepeatedContinuationMatcher),
    RepeatedPrefixed(RegExpRepeatedPrefixedMatcher),
    RepeatedCaptures(RegExpRepeatedCaptureMatcher),
}

/// A complete body's plan with a prefix of enclosing whole-match captures.
#[derive(Clone, Debug)]
pub(crate) struct RegExpMatcher {
    body: RegExpMatcherBody,
    enclosing_captures: usize,
    capture_count: usize,
}

impl RegExpMatcher {
    pub fn new(body: RegExpMatcherBody, enclosing_captures: usize) -> Self {
        let inner = match &body {
            RegExpMatcherBody::Backreferences(m) => m.capture_count(),
            RegExpMatcherBody::UnicodeFoldedLiteral(m) => m.capture_count(),
            RegExpMatcherBody::Literal(m)
            | RegExpMatcherBody::UnicodeEmpty(m)
            | RegExpMatcherBody::UnicodeCodePoints(m) => m.capture_ranges().len(),
            RegExpMatcherBody::Sequence(m) => m.capture_ranges().len(),
            RegExpMatcherBody::Character(_)
            | RegExpMatcherBody::UnicodeCharacter(_)
            | RegExpMatcherBody::UnicodeClass(_)
            | RegExpMatcherBody::UnicodeStringSet(_)
            | RegExpMatcherBody::UnicodeClassLiteral(_)
            | RegExpMatcherBody::UnicodeClassPair(_)
            | RegExpMatcherBody::UnicodeLiteralClassPair(_)
            | RegExpMatcherBody::UnicodeClassPairLiteral(_)
            | RegExpMatcherBody::UnicodeClassLiteralClass(_)
            | RegExpMatcherBody::UnicodeClassTriple(_)
            | RegExpMatcherBody::UnicodeClassTripleWord(_)
            | RegExpMatcherBody::UnicodeLiteralClassPairWord(_)
            | RegExpMatcherBody::UnicodeClassPairLiteralWord(_)
            | RegExpMatcherBody::UnicodeClassLiteralClassWord(_)
            | RegExpMatcherBody::UnicodeClassSequence(_) => 0,
            RegExpMatcherBody::UnicodeRepeatedAtom(m) => m.capture_count(),
            RegExpMatcherBody::UnicodeNullableClassCaptures(m) => m.capture_count(),
            RegExpMatcherBody::UnicodePropertyDot(_) => 2,
            RegExpMatcherBody::Prefixed(m) => m.capture_count(),
            RegExpMatcherBody::RepeatedLiteral(m) => m.capture_count(),
            RegExpMatcherBody::RepeatedSequence(m) => m.capture_count(),
            RegExpMatcherBody::RepeatedContinuation(m) => m.capture_count(),
            RegExpMatcherBody::RepeatedPrefixed(m) => m.capture_count(),
            RegExpMatcherBody::RepeatedCaptures(m) => m.capture_count(),
            RegExpMatcherBody::Anchored(m) => m.capture_count(),
            RegExpMatcherBody::Disjunction(m) => m.capture_count(),
            RegExpMatcherBody::Quantified(m) => m.capture_count(),
            RegExpMatcherBody::QuantifiedContinuation(m) => m.capture_count(),
        };
        // Both counts come from the same validated CapturingGroupsCount bound.
        let capture_count = enclosing_captures
            .checked_add(inner)
            .expect("validated capture count");
        Self {
            body,
            enclosing_captures,
            capture_count,
        }
    }

    pub fn find_with_work<'a, E>(
        &'a self,
        input: &JsString,
        start: usize,
        sticky: bool,
        charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<RegExpMatch<'a>>, E> {
        if let RegExpMatcherBody::Backreferences(matcher) = &self.body {
            return matcher
                .find_with_work(input, start, sticky, charge)
                .map(|found| {
                    found.map(|found| RegExpMatch {
                        range: found.range,
                        matcher: self,
                        branch: 0,
                        capture_count: self.capture_count,
                        dynamic_captures: Some(found.captures),
                        atom_capture: None,
                    })
                });
        }
        if let RegExpMatcherBody::UnicodeFoldedLiteral(matcher) = &self.body {
            return matcher
                .find_with_work(input, start, sticky, charge)
                .map(|found| {
                    found.map(|found| RegExpMatch {
                        range: found.range,
                        matcher: self,
                        branch: 0,
                        capture_count: self.capture_count,
                        dynamic_captures: Some(found.captures),
                        atom_capture: None,
                    })
                });
        }
        if let RegExpMatcherBody::UnicodeNullableClassCaptures(matcher) = &self.body {
            return matcher
                .find_with_captures_and_work(input, start, sticky, charge)
                .map(|found| {
                    found.map(|found| RegExpMatch {
                        range: found.range,
                        matcher: self,
                        branch: 0,
                        capture_count: self.capture_count,
                        dynamic_captures: None,
                        atom_capture: found.atom_capture,
                    })
                });
        }
        if let RegExpMatcherBody::UnicodeRepeatedAtom(matcher) = &self.body {
            return matcher
                .find_with_captures_and_work(input, start, sticky, charge)
                .map(|found| {
                    found.map(|found| RegExpMatch {
                        range: found.range,
                        matcher: self,
                        branch: 0,
                        capture_count: self.capture_count,
                        dynamic_captures: None,
                        atom_capture: found.atom_capture,
                    })
                });
        }
        if let RegExpMatcherBody::UnicodeClassLiteral(matcher) = &self.body {
            return matcher
                .find_with_work(input, start, sticky, charge)
                .map(|found| {
                    found.map(|range| RegExpMatch {
                        range,
                        matcher: self,
                        branch: 0,
                        capture_count: self.capture_count,
                        dynamic_captures: None,
                        atom_capture: None,
                    })
                });
        }
        if let RegExpMatcherBody::UnicodeClassPair(matcher) = &self.body {
            return matcher
                .find_with_work(input, start, sticky, charge)
                .map(|found| {
                    found.map(|range| RegExpMatch {
                        range,
                        matcher: self,
                        branch: 0,
                        capture_count: self.capture_count,
                        dynamic_captures: None,
                        atom_capture: None,
                    })
                });
        }
        if let RegExpMatcherBody::UnicodeLiteralClassPair(matcher) = &self.body {
            return matcher
                .find_with_work(input, start, sticky, charge)
                .map(|found| {
                    found.map(|range| RegExpMatch {
                        range,
                        matcher: self,
                        branch: 0,
                        capture_count: self.capture_count,
                        dynamic_captures: None,
                        atom_capture: None,
                    })
                });
        }
        if let RegExpMatcherBody::UnicodeClassPairLiteral(matcher) = &self.body {
            return matcher
                .find_with_work(input, start, sticky, charge)
                .map(|found| {
                    found.map(|range| RegExpMatch {
                        range,
                        matcher: self,
                        branch: 0,
                        capture_count: self.capture_count,
                        dynamic_captures: None,
                        atom_capture: None,
                    })
                });
        }
        if let RegExpMatcherBody::UnicodeClassLiteralClass(matcher) = &self.body {
            return matcher
                .find_with_work(input, start, sticky, charge)
                .map(|found| {
                    found.map(|range| RegExpMatch {
                        range,
                        matcher: self,
                        branch: 0,
                        capture_count: self.capture_count,
                        dynamic_captures: None,
                        atom_capture: None,
                    })
                });
        }
        if let RegExpMatcherBody::UnicodeClassTriple(matcher) = &self.body {
            return matcher
                .find_with_work(input, start, sticky, charge)
                .map(|found| {
                    found.map(|range| RegExpMatch {
                        range,
                        matcher: self,
                        branch: 0,
                        capture_count: self.capture_count,
                        dynamic_captures: None,
                        atom_capture: None,
                    })
                });
        }
        if let RegExpMatcherBody::UnicodeClassTripleWord(matcher) = &self.body {
            return matcher
                .find_with_work(input, start, sticky, charge)
                .map(|found| {
                    found.map(|range| RegExpMatch {
                        range,
                        matcher: self,
                        branch: 0,
                        capture_count: self.capture_count,
                        dynamic_captures: None,
                        atom_capture: None,
                    })
                });
        }
        if let RegExpMatcherBody::UnicodeLiteralClassPairWord(matcher) = &self.body {
            return matcher
                .find_with_work(input, start, sticky, charge)
                .map(|found| {
                    found.map(|range| RegExpMatch {
                        range,
                        matcher: self,
                        branch: 0,
                        capture_count: self.capture_count,
                        dynamic_captures: None,
                        atom_capture: None,
                    })
                });
        }
        if let RegExpMatcherBody::UnicodeClassPairLiteralWord(matcher) = &self.body {
            return matcher
                .find_with_work(input, start, sticky, charge)
                .map(|found| {
                    found.map(|range| RegExpMatch {
                        range,
                        matcher: self,
                        branch: 0,
                        capture_count: self.capture_count,
                        dynamic_captures: None,
                        atom_capture: None,
                    })
                });
        }
        if let RegExpMatcherBody::UnicodeClassLiteralClassWord(matcher) = &self.body {
            return matcher
                .find_with_work(input, start, sticky, charge)
                .map(|found| {
                    found.map(|range| RegExpMatch {
                        range,
                        matcher: self,
                        branch: 0,
                        capture_count: self.capture_count,
                        dynamic_captures: None,
                        atom_capture: None,
                    })
                });
        }
        if let RegExpMatcherBody::UnicodeClassSequence(matcher) = &self.body {
            return matcher
                .find_with_work(input, start, sticky, charge)
                .map(|found| {
                    found.map(|range| RegExpMatch {
                        range,
                        matcher: self,
                        branch: 0,
                        capture_count: self.capture_count,
                        dynamic_captures: None,
                        atom_capture: None,
                    })
                });
        }
        if let RegExpMatcherBody::UnicodePropertyDot(matcher) = &self.body {
            return matcher
                .find_with_work(input, start, sticky, charge)
                .map(|found| {
                    found.map(|found| RegExpMatch {
                        range: found.range,
                        matcher: self,
                        branch: 0,
                        capture_count: self.capture_count,
                        dynamic_captures: None,
                        atom_capture: Some(found.captures[0].clone()),
                    })
                });
        }
        Ok(self.find_precharged(input, start, sticky))
    }

    fn find_precharged<'a>(
        &'a self,
        input: &JsString,
        start: usize,
        sticky: bool,
    ) -> Option<RegExpMatch<'a>> {
        let (range, branch) = match &self.body {
            RegExpMatcherBody::Backreferences(_)
            | RegExpMatcherBody::UnicodeFoldedLiteral(_)
            | RegExpMatcherBody::UnicodeRepeatedAtom(_)
            | RegExpMatcherBody::UnicodeNullableClassCaptures(_)
            | RegExpMatcherBody::UnicodePropertyDot(_)
            | RegExpMatcherBody::UnicodeClassLiteral(_)
            | RegExpMatcherBody::UnicodeClassPair(_)
            | RegExpMatcherBody::UnicodeLiteralClassPair(_)
            | RegExpMatcherBody::UnicodeClassPairLiteral(_)
            | RegExpMatcherBody::UnicodeClassLiteralClass(_)
            | RegExpMatcherBody::UnicodeClassTriple(_)
            | RegExpMatcherBody::UnicodeClassTripleWord(_)
            | RegExpMatcherBody::UnicodeLiteralClassPairWord(_)
            | RegExpMatcherBody::UnicodeClassPairLiteralWord(_)
            | RegExpMatcherBody::UnicodeClassLiteralClassWord(_)
            | RegExpMatcherBody::UnicodeClassSequence(_) => {
                unreachable!("fallible matching dispatch")
            }
            RegExpMatcherBody::Literal(matcher) => (matcher.find(input, start, sticky)?, 0),
            RegExpMatcherBody::UnicodeCodePoints(matcher) => {
                (matcher.find_unicode_code_points(input, start, sticky)?, 0)
            }

            RegExpMatcherBody::UnicodeEmpty(matcher) => {
                (matcher.find_unicode_empty(input, start)?, 0)
            }
            RegExpMatcherBody::Anchored(matcher) => (matcher.find(input, start, sticky)?, 0),
            RegExpMatcherBody::Character(matcher) => (matcher.find(input, start, sticky)?, 0),
            RegExpMatcherBody::UnicodeCharacter(matcher) => {
                (matcher.find(input, start, sticky)?, 0)
            }
            RegExpMatcherBody::UnicodeClass(matcher) => (matcher.find(input, start, sticky)?, 0),
            RegExpMatcherBody::UnicodeStringSet(matcher) => {
                (matcher.find(input, start, sticky)?, 0)
            }
            RegExpMatcherBody::Prefixed(matcher) => (matcher.find(input, start, sticky)?, 0),
            RegExpMatcherBody::RepeatedLiteral(matcher) => (matcher.find(input, start, sticky)?, 0),
            RegExpMatcherBody::RepeatedSequence(matcher) => {
                (matcher.find(input, start, sticky)?, 0)
            }
            RegExpMatcherBody::RepeatedContinuation(matcher) => {
                (matcher.find(input, start, sticky)?, 0)
            }
            RegExpMatcherBody::RepeatedPrefixed(matcher) => {
                (matcher.find(input, start, sticky)?, 0)
            }
            RegExpMatcherBody::RepeatedCaptures(matcher) => {
                (matcher.find(input, start, sticky)?, 0)
            }
            RegExpMatcherBody::Quantified(matcher) => (matcher.find(input, start, sticky)?, 0),
            RegExpMatcherBody::QuantifiedContinuation(matcher) => {
                (matcher.find(input, start, sticky)?, 0)
            }
            RegExpMatcherBody::Sequence(matcher) => (matcher.find(input, start, sticky)?, 0),
            RegExpMatcherBody::Disjunction(matcher) => {
                let (branch, range) = matcher.find_branch(input, start, sticky)?;
                (range, branch)
            }
        };
        Some(RegExpMatch {
            range,
            matcher: self,
            branch,
            capture_count: self.capture_count(),
            dynamic_captures: None,
            atom_capture: None,
        })
    }
    pub fn capture_count(&self) -> usize {
        self.capture_count
    }
    pub fn search_passes(&self, sticky: bool) -> usize {
        match &self.body {
            // These plans charge actual operations through find_with_work.
            RegExpMatcherBody::Backreferences(_)
            | RegExpMatcherBody::UnicodeFoldedLiteral(_)
            | RegExpMatcherBody::UnicodeRepeatedAtom(_)
            | RegExpMatcherBody::UnicodeNullableClassCaptures(_) => 0,
            RegExpMatcherBody::Literal(_)
            | RegExpMatcherBody::UnicodeEmpty(_)
            | RegExpMatcherBody::UnicodeCodePoints(_) => 1,
            RegExpMatcherBody::Anchored(matcher) => matcher.search_passes(sticky),
            RegExpMatcherBody::Character(_) => 1,
            RegExpMatcherBody::UnicodeCharacter(matcher) => matcher.search_passes(),
            RegExpMatcherBody::UnicodeClass(matcher) => matcher.search_passes(),
            RegExpMatcherBody::UnicodeStringSet(matcher) => matcher.search_passes(sticky),
            RegExpMatcherBody::UnicodeClassLiteral(matcher) => matcher.search_passes(),
            RegExpMatcherBody::UnicodeClassPair(matcher) => matcher.search_passes(),
            RegExpMatcherBody::UnicodeLiteralClassPair(matcher) => matcher.search_passes(),
            RegExpMatcherBody::UnicodeClassPairLiteral(matcher) => matcher.search_passes(),
            RegExpMatcherBody::UnicodeClassLiteralClass(matcher) => matcher.search_passes(),
            RegExpMatcherBody::UnicodeClassTriple(matcher) => matcher.search_passes(),
            RegExpMatcherBody::UnicodeClassTripleWord(matcher) => matcher.search_passes(),
            RegExpMatcherBody::UnicodeLiteralClassPairWord(matcher) => matcher.search_passes(),
            RegExpMatcherBody::UnicodeClassPairLiteralWord(matcher) => matcher.search_passes(),
            RegExpMatcherBody::UnicodeClassLiteralClassWord(matcher) => matcher.search_passes(),
            RegExpMatcherBody::UnicodeClassSequence(matcher) => matcher.search_passes(),
            RegExpMatcherBody::UnicodePropertyDot(matcher) => matcher.search_passes(),
            RegExpMatcherBody::Quantified(_) => 1,
            RegExpMatcherBody::Prefixed(m) => m.search_passes(sticky),
            RegExpMatcherBody::RepeatedLiteral(m) => m.search_passes(),
            RegExpMatcherBody::RepeatedSequence(m) => m.search_passes(sticky),
            RegExpMatcherBody::RepeatedContinuation(m) => m.search_passes(sticky),
            RegExpMatcherBody::RepeatedPrefixed(m) => m.search_passes(sticky),
            RegExpMatcherBody::RepeatedCaptures(m) => m.search_passes(sticky),
            RegExpMatcherBody::QuantifiedContinuation(m) => m.search_passes(),
            RegExpMatcherBody::Sequence(matcher) => matcher.search_passes(sticky),
            RegExpMatcherBody::Disjunction(matcher) => matcher.search_passes(sticky),
        }
    }

    /// Sticky repeated atoms can consume more input than their source length.
    pub fn search_work(&self, sticky: bool, source_len: usize, remaining: usize) -> usize {
        if let RegExpMatcherBody::UnicodeStringSet(matcher) = &self.body {
            // Include a possible initial low-surrogate rewind and the final
            // candidate at input end, where an empty member can still match.
            let remaining = remaining.saturating_add(2);
            return if sticky {
                matcher
                    .max_match_code_units()
                    .saturating_add(1)
                    .min(remaining)
            } else {
                remaining
            };
        }
        if let RegExpMatcherBody::UnicodeClassSequence(matcher) = &self.body {
            // The proved term count bounds original UTF-16 width, including a
            // possible one-unit rewind at an initial low surrogate.
            let remaining = remaining.saturating_add(1);
            return if sticky {
                matcher.max_match_code_units().min(remaining)
            } else {
                remaining
            };
        }

        if matches!(
            self.body,
            RegExpMatcherBody::UnicodeLiteralClassPair(_)
                | RegExpMatcherBody::UnicodeClassPairLiteral(_)
                | RegExpMatcherBody::UnicodeClassLiteralClass(_)
                | RegExpMatcherBody::UnicodeClassTriple(_)
                | RegExpMatcherBody::UnicodeClassTripleWord(_)
                | RegExpMatcherBody::UnicodeLiteralClassPairWord(_)
                | RegExpMatcherBody::UnicodeClassPairLiteralWord(_)
                | RegExpMatcherBody::UnicodeClassLiteralClassWord(_)
        ) {
            // Three original characters consume at most six units. Their first
            // term retains the approved initial-low-surrogate rewind.
            let remaining = remaining.saturating_add(1);
            return if sticky { 6.min(remaining) } else { remaining };
        }

        if matches!(
            self.body,
            RegExpMatcherBody::UnicodePropertyDot(_)
                | RegExpMatcherBody::UnicodeClassLiteral(_)
                | RegExpMatcherBody::UnicodeClassPair(_)
        ) {
            let remaining = remaining.saturating_add(1);
            return if sticky { 4.min(remaining) } else { remaining };
        }

        if matches!(
            self.body,
            RegExpMatcherBody::UnicodeCharacter(_) | RegExpMatcherBody::UnicodeClass(_)
        ) {
            // A Unicode character plan consumes at most two units. Pair rewind
            // can expose one extra unit, including in the one-unit dot source.
            let remaining = remaining.saturating_add(1);
            return if sticky { 2.min(remaining) } else { remaining };
        }
        // A Unicode literal search can move its initial position back one unit.
        // Precharge that possible unit before executing an opted-in work quota.
        let remaining = if matches!(self.body, RegExpMatcherBody::UnicodeCodePoints(_)) {
            remaining.saturating_add(1)
        } else {
            remaining
        };
        let full_suffix = match &self.body {
            RegExpMatcherBody::Quantified(_)
            | RegExpMatcherBody::UnicodeRepeatedAtom(_)
            | RegExpMatcherBody::UnicodeNullableClassCaptures(_)
            | RegExpMatcherBody::QuantifiedContinuation(_)
            | RegExpMatcherBody::Prefixed(_)
            | RegExpMatcherBody::RepeatedLiteral(_)
            | RegExpMatcherBody::RepeatedSequence(_)
            | RegExpMatcherBody::RepeatedContinuation(_)
            | RegExpMatcherBody::RepeatedPrefixed(_)
            | RegExpMatcherBody::RepeatedCaptures(_) => true,
            RegExpMatcherBody::Anchored(matcher) => matcher.requires_full_suffix(),
            RegExpMatcherBody::Disjunction(matcher) => matcher.requires_full_suffix(),
            _ => false,
        };
        if sticky && !full_suffix {
            source_len.min(remaining)
        } else {
            remaining
        }
    }
}

/// A successful match borrows only immutable compiler storage, not the heap.
#[derive(Debug)]
pub(crate) struct RegExpMatch<'a> {
    pub range: Range<usize>,
    matcher: &'a RegExpMatcher,
    branch: usize,
    pub capture_count: usize,
    dynamic_captures: Option<Box<[Option<Range<usize>>]>>,
    atom_capture: Option<Range<usize>>,
}

impl RegExpMatch<'_> {
    /// Absolute UTF-16 range, or None for a group in an unselected branch.
    pub fn capture(&self, index: usize) -> Option<Range<usize>> {
        if index >= self.capture_count {
            return None;
        }
        if let Some(captures) = &self.dynamic_captures {
            return captures.get(index)?.clone();
        }
        if index < self.matcher.enclosing_captures {
            return Some(self.range.clone());
        }
        let index = index - self.matcher.enclosing_captures;
        let fixed = match &self.matcher.body {
            RegExpMatcherBody::Backreferences(_) | RegExpMatcherBody::UnicodeFoldedLiteral(_) => {
                unreachable!("dynamic captures")
            }
            RegExpMatcherBody::Literal(matcher)
            | RegExpMatcherBody::UnicodeEmpty(matcher)
            | RegExpMatcherBody::UnicodeCodePoints(matcher) => matcher.capture_ranges(),
            RegExpMatcherBody::Sequence(matcher) => matcher.capture_ranges(),
            RegExpMatcherBody::Character(_)
            | RegExpMatcherBody::UnicodeCharacter(_)
            | RegExpMatcherBody::UnicodeClass(_)
            | RegExpMatcherBody::UnicodeStringSet(_)
            | RegExpMatcherBody::UnicodeClassLiteral(_)
            | RegExpMatcherBody::UnicodeClassPair(_)
            | RegExpMatcherBody::UnicodeLiteralClassPair(_)
            | RegExpMatcherBody::UnicodeClassPairLiteral(_)
            | RegExpMatcherBody::UnicodeClassLiteralClass(_)
            | RegExpMatcherBody::UnicodeClassTriple(_)
            | RegExpMatcherBody::UnicodeClassTripleWord(_)
            | RegExpMatcherBody::UnicodeLiteralClassPairWord(_)
            | RegExpMatcherBody::UnicodeClassPairLiteralWord(_)
            | RegExpMatcherBody::UnicodeClassLiteralClassWord(_)
            | RegExpMatcherBody::UnicodeClassSequence(_) => {
                return None;
            }
            RegExpMatcherBody::UnicodeRepeatedAtom(_) => return self.atom_capture.clone(),
            RegExpMatcherBody::UnicodeNullableClassCaptures(matcher) => {
                return matcher.capture_range(index, self.atom_capture.as_ref());
            }
            RegExpMatcherBody::UnicodePropertyDot(_) => {
                return match index {
                    0 => self.atom_capture.clone(),
                    1 => self
                        .atom_capture
                        .as_ref()
                        .map(|first| first.end..self.range.end),
                    _ => None,
                };
            }

            RegExpMatcherBody::Prefixed(matcher) => {
                return matcher.capture_range(index, &self.range);
            }
            RegExpMatcherBody::RepeatedLiteral(matcher) => {
                return matcher.capture_range(index, &self.range);
            }
            RegExpMatcherBody::RepeatedSequence(matcher) => {
                return matcher.capture_range(index, &self.range);
            }
            RegExpMatcherBody::RepeatedContinuation(matcher) => {
                return matcher.capture_range(index, &self.range);
            }
            RegExpMatcherBody::RepeatedPrefixed(matcher) => {
                return matcher.capture_range(index, &self.range);
            }
            RegExpMatcherBody::RepeatedCaptures(matcher) => {
                return matcher.capture_range(index, &self.range);
            }
            RegExpMatcherBody::Anchored(matcher) => {
                return matcher.capture_range(index, &self.range);
            }
            RegExpMatcherBody::Quantified(matcher) => {
                return matcher.capture_range(index, &self.range);
            }
            RegExpMatcherBody::QuantifiedContinuation(matcher) => {
                return matcher.capture_range(index, &self.range);
            }
            RegExpMatcherBody::Disjunction(matcher) => {
                return matcher.capture_range(self.branch, index, &self.range);
            }
        };
        let relative = fixed.get(index)?;
        Some(
            self.range.start.checked_add(relative.start)?
                ..self.range.start.checked_add(relative.end)?,
        )
    }
}

#[derive(Clone, Debug)]
pub(crate) struct RegExpNamedGroup {
    pub name: JsString,
    pub slots: Vec<usize>,
}

#[derive(Clone, Debug)]
pub(crate) struct RegExpData {
    pub source: JsString,
    pub flags: JsString,
    pub matcher: Option<RegExpMatcher>,
    pub named_groups: Arc<[RegExpNamedGroup]>,
}

impl Objects {
    pub(crate) fn initialize_regexp(
        &mut self,
        object: &Handle,
        data: RegExpData,
    ) -> Result<(), Error> {
        let object = self.object_mut(object)?;
        if object.regexp.is_some() {
            return Err(Error::WrongKind);
        }
        object.regexp = Some(Box::new(data));
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn string_operation_empty_set_and_empty_member_share_bounds_but_not_results() {
        for (source, empty_member) in [(r"[\q{ab}&&\q{cd}]", false), (r"[\q{|ab}--\q{ab}]", true)] {
            let source = JsString::from(source);
            let body = RegExpUnicodeStringSetMatcher::compile_string_operations_with_work(
                &source,
                false,
                false,
                |_| Ok::<(), ()>(()),
            )
            .unwrap()
            .unwrap();
            let matcher = RegExpMatcher::new(RegExpMatcherBody::UnicodeStringSet(body), 2);
            assert_eq!(matcher.search_work(true, 0, usize::MAX), 1);
            assert_eq!(matcher.search_work(false, 0, usize::MAX), usize::MAX);
            for remaining in 0..5 {
                assert_eq!(matcher.search_work(true, source.len(), remaining), 1);
                assert_eq!(
                    matcher.search_work(false, source.len(), remaining),
                    remaining + 2
                );
            }
            let input = JsString::from("😀");
            let found = matcher
                .find_with_work(&input, 1, true, |_| Err::<(), ()>(()))
                .unwrap();
            if empty_member {
                let found = found.unwrap();
                assert_eq!(found.range, 0..0);
                assert_eq!(found.capture(0), Some(0..0));
                assert_eq!(found.capture(1), Some(0..0));
            } else {
                assert!(found.is_none());
            }
        }
    }

    #[test]
    fn variable_string_scalar_work_covers_consuming_members_beside_empty_strings() {
        let source = JsString::from(r"[\q{}\p{Any}]");
        let body = RegExpUnicodeStringSetMatcher::compile_scalar_union_with_work(
            &source,
            false,
            false,
            |_| Ok::<(), ()>(()),
        )
        .unwrap()
        .unwrap();
        let matcher = RegExpMatcher::new(RegExpMatcherBody::UnicodeStringSet(body), 2);
        for source_len in [0, source.len(), usize::MAX] {
            for remaining in 0..10 {
                assert_eq!(
                    matcher.search_work(true, source_len, remaining),
                    3.min(remaining + 2)
                );
                assert_eq!(
                    matcher.search_work(false, source_len, remaining),
                    remaining + 2
                );
            }
            assert_eq!(matcher.search_work(true, source_len, usize::MAX), 3);
            assert_eq!(
                matcher.search_work(false, source_len, usize::MAX),
                usize::MAX
            );
        }
        let input = JsString::from("😀");
        let found = matcher
            .find_with_work(&input, 1, true, |_| Err::<(), ()>(()))
            .unwrap()
            .unwrap();
        assert_eq!(found.range, 0..2);
        assert_eq!(found.capture(0), Some(0..2));
        assert_eq!(found.capture(1), Some(0..2));
        let found = matcher
            .find_with_work(&input, 2, true, |_| Err::<(), ()>(()))
            .unwrap()
            .unwrap();
        assert_eq!(found.range, 2..2);
        assert_eq!(found.capture(1), Some(2..2));
    }

    #[test]
    fn variable_string_work_bounds_include_empty_candidates_and_pair_rewind() {
        for (pattern, max_units) in [(r"[\q{}]", 0usize), (r"[\q{|😀x|a}]", 4)] {
            let source = JsString::from(pattern);
            let body =
                RegExpUnicodeStringSetMatcher::compile_with_work(&source, false, false, |_| {
                    Ok::<(), ()>(())
                })
                .unwrap()
                .unwrap();
            let passes = [body.search_passes(false), body.search_passes(true)];
            let matcher = RegExpMatcher::new(RegExpMatcherBody::UnicodeStringSet(body), 2);
            for source_len in [0, source.len(), usize::MAX] {
                for remaining in 0..10 {
                    assert_eq!(
                        matcher.search_work(true, source_len, remaining),
                        (max_units + 1).min(remaining + 2)
                    );
                    assert_eq!(
                        matcher.search_work(false, source_len, remaining),
                        remaining + 2
                    );
                }
                assert_eq!(
                    matcher.search_work(true, source_len, usize::MAX),
                    max_units + 1
                );
                assert_eq!(
                    matcher.search_work(false, source_len, usize::MAX),
                    usize::MAX
                );
            }
            assert_eq!(matcher.search_passes(false), passes[0]);
            assert_eq!(matcher.search_passes(true), passes[1]);
            let input = JsString::from("😀x");
            let found = matcher
                .find_with_work(&input, 1, true, |_| Err::<(), ()>(()))
                .unwrap()
                .unwrap();
            let wanted = if max_units == 0 { 0..0 } else { 0..3 };
            assert_eq!(found.range, wanted);
            assert_eq!(found.capture_count, 2);
            assert_eq!(found.capture(0), Some(wanted.clone()));
            assert_eq!(found.capture(1), Some(wanted));
            assert_eq!(found.capture(2), None);
            let found = matcher
                .find_with_work(&input, input.len(), true, |_| Err::<(), ()>(()))
                .unwrap()
                .unwrap();
            assert_eq!(found.range, input.len()..input.len());
        }
    }

    #[test]
    fn literal_checked_pair_work_bound_covers_three_original_characters() {
        let source = JsString::from(r"\u{1f600}[[\u{1f601}]][[\u{1f602}]]");
        let body = RegExpUnicodeLiteralClassPairMatcher::compile_v_set_with_work(
            &source,
            false,
            false,
            |_| Ok::<(), std::convert::Infallible>(()),
        )
        .unwrap()
        .unwrap();
        let matcher = RegExpMatcher::new(RegExpMatcherBody::UnicodeLiteralClassPair(body), 2);
        for source_len in [0, 1, source.len(), usize::MAX] {
            for remaining in 0usize..=7 {
                assert_eq!(
                    matcher.search_work(true, source_len, remaining),
                    [1, 2, 3, 4, 5, 6, 6, 6][remaining]
                );
                assert_eq!(
                    matcher.search_work(false, source_len, remaining),
                    remaining + 1
                );
            }
            assert_eq!(matcher.search_work(true, source_len, usize::MAX), 6);
            assert_eq!(
                matcher.search_work(false, source_len, usize::MAX),
                usize::MAX
            );
        }
        let input = JsString::from("😀😁😂");
        // Start1 is inside the leading pair. Its five remaining units need the
        // extra rewind unit; the complete three-character range is six units.
        assert_eq!(matcher.search_work(true, source.len(), input.len() - 1), 6);
        let mut charges = Vec::new();
        let found = matcher
            .find_with_work(&input, 1, true, |n| {
                charges.push(n);
                Ok::<(), std::convert::Infallible>(())
            })
            .unwrap()
            .unwrap();
        assert_eq!(charges, [1]);
        assert_eq!(found.range, 0..6);
        assert_eq!(found.capture_count, 2);
        assert_eq!(found.capture(0), Some(0..6));
        assert_eq!(found.capture(1), Some(0..6));
        assert_eq!(found.capture(2), None);
        let pair = RegExpUnicodeClassPairMatcher::compile_v_set_with_work(
            &JsString::from("[[a]][[b]]"),
            false,
            false,
            |_| Ok::<(), std::convert::Infallible>(()),
        )
        .unwrap()
        .unwrap();
        let old = RegExpMatcher::new(RegExpMatcherBody::UnicodeClassPair(pair), 0);
        for remaining in 0usize..=7 {
            assert_eq!(
                old.search_work(true, 1, remaining),
                [1, 2, 3, 4, 4, 4, 4, 4][remaining]
            );
            assert_eq!(old.search_work(false, 1, remaining), remaining + 1);
        }

        // The suffix-order variant shares the six-unit dispatcher; every old
        // prefix and four-unit assertion above remains unchanged.
        let suffix_source = JsString::from(r"[[\u{1f600}]][[\u{1f601}]]\u{1f602}");
        let suffix = RegExpUnicodeClassPairLiteralMatcher::compile_v_set_with_work(
            &suffix_source,
            false,
            false,
            |_| Ok::<(), std::convert::Infallible>(()),
        )
        .unwrap()
        .unwrap();
        let suffix = RegExpMatcher::new(RegExpMatcherBody::UnicodeClassPairLiteral(suffix), 2);
        for source_len in [0, 1, suffix_source.len(), usize::MAX] {
            for remaining in 0usize..=7 {
                assert_eq!(
                    suffix.search_work(true, source_len, remaining),
                    [1, 2, 3, 4, 5, 6, 6, 6][remaining]
                );
                assert_eq!(
                    suffix.search_work(false, source_len, remaining),
                    remaining + 1
                );
            }
            assert_eq!(suffix.search_work(true, source_len, usize::MAX), 6);
            assert_eq!(
                suffix.search_work(false, source_len, usize::MAX),
                usize::MAX
            );
        }
        assert_eq!(
            suffix.search_work(true, suffix_source.len(), input.len() - 1),
            6
        );
        let mut suffix_charges = Vec::new();
        let found = suffix
            .find_with_work(&input, 1, true, |n| {
                suffix_charges.push(n);
                Ok::<(), std::convert::Infallible>(())
            })
            .unwrap()
            .unwrap();
        assert_eq!(suffix_charges, [1]);
        assert_eq!(found.range, 0..6);
        assert_eq!(found.capture_count, 2);
        assert_eq!(found.capture(0), Some(0..6));
        assert_eq!(found.capture(1), Some(0..6));
        assert_eq!(found.capture(2), None);

        // The middle-literal variant shares the six-unit dispatcher; every
        // preceding prefix, suffix and four-unit assertion remains unchanged.
        let middle_source = JsString::from(r"[[\u{1f600}]]\u{1f601}[[\u{1f602}]]");
        let middle = RegExpUnicodeClassLiteralClassMatcher::compile_v_set_with_work(
            &middle_source,
            false,
            false,
            |_| Ok::<(), std::convert::Infallible>(()),
        )
        .unwrap()
        .unwrap();
        let middle = RegExpMatcher::new(RegExpMatcherBody::UnicodeClassLiteralClass(middle), 2);
        for source_len in [0, 1, middle_source.len(), usize::MAX] {
            for remaining in 0usize..=7 {
                assert_eq!(
                    middle.search_work(true, source_len, remaining),
                    [1, 2, 3, 4, 5, 6, 6, 6][remaining]
                );
                assert_eq!(
                    middle.search_work(false, source_len, remaining),
                    remaining + 1
                );
            }
            assert_eq!(middle.search_work(true, source_len, usize::MAX), 6);
            assert_eq!(
                middle.search_work(false, source_len, usize::MAX),
                usize::MAX
            );
        }
        assert_eq!(
            middle.search_work(true, middle_source.len(), input.len() - 1),
            6
        );
        let mut middle_charges = Vec::new();
        let found = middle
            .find_with_work(&input, 1, true, |n| {
                middle_charges.push(n);
                Ok::<(), std::convert::Infallible>(())
            })
            .unwrap()
            .unwrap();
        assert_eq!(middle_charges, [1]);
        assert_eq!(found.range, 0..6);
        assert_eq!(found.capture_count, 2);
        assert_eq!(found.capture(0), Some(0..6));
        assert_eq!(found.capture(1), Some(0..6));
        assert_eq!(found.capture(2), None);

        // Three checked classes share the same six-unit dispatcher. Every
        // preceding literal-order and four-unit assertion remains unchanged.
        let triple_source = JsString::from(r"[[\u{1f600}]][[\u{1f601}]][[\u{1f602}]]");
        let triple = RegExpUnicodeClassTripleMatcher::compile_v_set_with_work(
            &triple_source,
            false,
            false,
            |_| Ok::<(), std::convert::Infallible>(()),
        )
        .unwrap()
        .unwrap();
        let triple = RegExpMatcher::new(RegExpMatcherBody::UnicodeClassTriple(triple), 2);
        for source_len in [0, 1, triple_source.len(), usize::MAX] {
            for remaining in 0usize..=7 {
                assert_eq!(
                    triple.search_work(true, source_len, remaining),
                    [1, 2, 3, 4, 5, 6, 6, 6][remaining]
                );
                assert_eq!(
                    triple.search_work(false, source_len, remaining),
                    remaining + 1
                );
            }
            assert_eq!(triple.search_work(true, source_len, usize::MAX), 6);
            assert_eq!(
                triple.search_work(false, source_len, usize::MAX),
                usize::MAX
            );
        }
        assert_eq!(
            triple.search_work(true, triple_source.len(), input.len() - 1),
            6
        );
        let mut triple_charges = Vec::new();
        let found = triple
            .find_with_work(&input, 1, true, |n| {
                triple_charges.push(n);
                Ok::<(), std::convert::Infallible>(())
            })
            .unwrap()
            .unwrap();
        assert_eq!(triple_charges, [1]);
        assert_eq!(found.range, 0..6);
        assert_eq!(found.capture_count, 2);
        assert_eq!(found.capture(0), Some(0..6));
        assert_eq!(found.capture(1), Some(0..6));
        assert_eq!(found.capture(2), None);

        // Outside word gates share the unchanged six-unit branch and retain
        // original neighbors at the approved initial-low-pair boundary.
        let word_triple_source = JsString::from(r"\B[[\u{1f600}]][[\u{1f601}]][[\u{1f602}]]\B");
        let word_triple = RegExpUnicodeClassTripleWordMatcher::compile_v_set_with_work(
            &word_triple_source,
            false,
            false,
            |_| Ok::<(), std::convert::Infallible>(()),
        )
        .unwrap()
        .unwrap();
        let word_triple =
            RegExpMatcher::new(RegExpMatcherBody::UnicodeClassTripleWord(word_triple), 2);
        for source_len in [0, 1, word_triple_source.len(), usize::MAX] {
            for remaining in 0usize..=7 {
                assert_eq!(
                    word_triple.search_work(true, source_len, remaining),
                    [1, 2, 3, 4, 5, 6, 6, 6][remaining]
                );
                assert_eq!(
                    word_triple.search_work(false, source_len, remaining),
                    remaining + 1
                );
            }
            assert_eq!(word_triple.search_work(true, source_len, usize::MAX), 6);
            assert_eq!(
                word_triple.search_work(false, source_len, usize::MAX),
                usize::MAX
            );
        }
        assert_eq!(
            word_triple.search_work(true, word_triple_source.len(), input.len() - 1),
            6
        );
        let mut word_triple_charges = Vec::new();
        let found = word_triple
            .find_with_work(&input, 1, true, |n| {
                word_triple_charges.push(n);
                Ok::<(), std::convert::Infallible>(())
            })
            .unwrap()
            .unwrap();
        assert_eq!(word_triple_charges, [1]);
        assert_eq!(found.range, 0..6);
        assert_eq!(found.capture_count, 2);
        assert_eq!(found.capture(0), Some(0..6));
        assert_eq!(found.capture(1), Some(0..6));
        assert_eq!(found.capture(2), None);

        // Literal-prefix word gates share the unchanged six-unit branch and retain
        // original neighbors at the approved initial-low-pair boundary.
        let word_literal_pair_source = JsString::from(r"\B\u{1f600}[[\u{1f601}]][[\u{1f602}]]\B");
        let word_literal_pair = RegExpUnicodeLiteralClassPairWordMatcher::compile_v_set_with_work(
            &word_literal_pair_source,
            false,
            false,
            |_| Ok::<(), std::convert::Infallible>(()),
        )
        .unwrap()
        .unwrap();
        let word_literal_pair = RegExpMatcher::new(
            RegExpMatcherBody::UnicodeLiteralClassPairWord(word_literal_pair),
            2,
        );
        for source_len in [0, 1, word_literal_pair_source.len(), usize::MAX] {
            for remaining in 0usize..=7 {
                assert_eq!(
                    word_literal_pair.search_work(true, source_len, remaining),
                    [1, 2, 3, 4, 5, 6, 6, 6][remaining]
                );
                assert_eq!(
                    word_literal_pair.search_work(false, source_len, remaining),
                    remaining + 1
                );
            }
            assert_eq!(
                word_literal_pair.search_work(true, source_len, usize::MAX),
                6
            );
            assert_eq!(
                word_literal_pair.search_work(false, source_len, usize::MAX),
                usize::MAX
            );
        }
        assert_eq!(
            word_literal_pair.search_work(true, word_literal_pair_source.len(), input.len() - 1),
            6
        );
        let mut word_literal_pair_charges = Vec::new();
        let found = word_literal_pair
            .find_with_work(&input, 1, true, |n| {
                word_literal_pair_charges.push(n);
                Ok::<(), std::convert::Infallible>(())
            })
            .unwrap()
            .unwrap();
        assert_eq!(word_literal_pair_charges, [1]);
        assert_eq!(found.range, 0..6);
        assert_eq!(found.capture_count, 2);
        assert_eq!(found.capture(0), Some(0..6));
        assert_eq!(found.capture(1), Some(0..6));
        assert_eq!(found.capture(2), None);

        // Literal-suffix word gates share the unchanged six-unit branch and retain
        // original neighbors at the approved initial-low-pair boundary.
        let word_pair_literal_source = JsString::from(r"\B[[\u{1f600}]][[\u{1f601}]]\u{1f602}\B");
        let word_pair_literal = RegExpUnicodeClassPairLiteralWordMatcher::compile_v_set_with_work(
            &word_pair_literal_source,
            false,
            false,
            |_| Ok::<(), std::convert::Infallible>(()),
        )
        .unwrap()
        .unwrap();
        let word_pair_literal = RegExpMatcher::new(
            RegExpMatcherBody::UnicodeClassPairLiteralWord(word_pair_literal),
            2,
        );
        for source_len in [0, 1, word_pair_literal_source.len(), usize::MAX] {
            for remaining in 0usize..=7 {
                assert_eq!(
                    word_pair_literal.search_work(true, source_len, remaining),
                    [1, 2, 3, 4, 5, 6, 6, 6][remaining]
                );
                assert_eq!(
                    word_pair_literal.search_work(false, source_len, remaining),
                    remaining + 1
                );
            }
            assert_eq!(
                word_pair_literal.search_work(true, source_len, usize::MAX),
                6
            );
            assert_eq!(
                word_pair_literal.search_work(false, source_len, usize::MAX),
                usize::MAX
            );
        }
        assert_eq!(
            word_pair_literal.search_work(true, word_pair_literal_source.len(), input.len() - 1),
            6
        );
        let mut word_pair_literal_charges = Vec::new();
        let found = word_pair_literal
            .find_with_work(&input, 1, true, |n| {
                word_pair_literal_charges.push(n);
                Ok::<(), std::convert::Infallible>(())
            })
            .unwrap()
            .unwrap();
        assert_eq!(word_pair_literal_charges, [1]);
        assert_eq!(found.range, 0..6);
        assert_eq!(found.capture_count, 2);
        assert_eq!(found.capture(0), Some(0..6));
        assert_eq!(found.capture(1), Some(0..6));
        assert_eq!(found.capture(2), None);

        // Literal-middle word gates share the unchanged six-unit branch and retain
        // original neighbors at the approved initial-low-pair boundary.
        let word_class_literal_class_source =
            JsString::from(r"\B[[\u{1f600}]]\u{1f601}[[\u{1f602}]]\B");
        let word_class_literal_class =
            RegExpUnicodeClassLiteralClassWordMatcher::compile_v_set_with_work(
                &word_class_literal_class_source,
                false,
                false,
                |_| Ok::<(), std::convert::Infallible>(()),
            )
            .unwrap()
            .unwrap();
        let word_class_literal_class = RegExpMatcher::new(
            RegExpMatcherBody::UnicodeClassLiteralClassWord(word_class_literal_class),
            2,
        );
        for source_len in [0, 1, word_class_literal_class_source.len(), usize::MAX] {
            for remaining in 0usize..=7 {
                assert_eq!(
                    word_class_literal_class.search_work(true, source_len, remaining),
                    [1, 2, 3, 4, 5, 6, 6, 6][remaining]
                );
                assert_eq!(
                    word_class_literal_class.search_work(false, source_len, remaining),
                    remaining + 1
                );
            }
            assert_eq!(
                word_class_literal_class.search_work(true, source_len, usize::MAX),
                6
            );
            assert_eq!(
                word_class_literal_class.search_work(false, source_len, usize::MAX),
                usize::MAX
            );
        }
        assert_eq!(
            word_class_literal_class.search_work(
                true,
                word_class_literal_class_source.len(),
                input.len() - 1
            ),
            6
        );
        let mut word_class_literal_class_charges = Vec::new();
        let found = word_class_literal_class
            .find_with_work(&input, 1, true, |n| {
                word_class_literal_class_charges.push(n);
                Ok::<(), std::convert::Infallible>(())
            })
            .unwrap()
            .unwrap();
        assert_eq!(word_class_literal_class_charges, [1]);
        assert_eq!(found.range, 0..6);
        assert_eq!(found.capture_count, 2);
        assert_eq!(found.capture(0), Some(0..6));
        assert_eq!(found.capture(1), Some(0..6));
        assert_eq!(found.capture(2), None);
    }

    #[test]
    fn foreign_and_stale_handles_cannot_receive_original_slots() {
        let mut objects = Objects::new(1, 8);
        let stale = objects.create(None).unwrap();
        objects.collect([], 100).unwrap();
        let live = objects.create(None).unwrap();
        let mut other = Objects::new(1, 8);
        let foreign = other.create(None).unwrap();
        for (object, expected) in [
            (&foreign, spite_heap::Error::ForeignHandle),
            (&stale, spite_heap::Error::StaleHandle),
        ] {
            assert!(
                matches!(objects.initialize_regexp(object, RegExpData { source: JsString::from("a"), flags: JsString::from("g"),matcher:None, named_groups: Arc::from([]) }), Err(Error::Heap(error)) if error == expected)
            );
        }
        assert!(objects.inspect(&live).unwrap().regexp_data().is_none());
    }

    #[test]
    fn original_slots_survive_collection_and_are_owned_without_prototype_inheritance() {
        let mut objects = Objects::new(4, 8);
        let prototype = objects.create(None).unwrap();
        let object = objects.create(Some(&prototype)).unwrap();
        let source = JsString::from_code_units(vec![0xd800, 0x2f, 0x0a]);
        objects
            .initialize_regexp(
                &object,
                RegExpData {
                    source: source.clone(),
                    flags: JsString::from("yg"),
                    matcher: RegExpLiteralMatcher::compile(&source, false)
                        .map(|body| RegExpMatcher::new(RegExpMatcherBody::Literal(body), 0)),
                    named_groups: Arc::from([]),
                },
            )
            .unwrap();
        let lookalike = objects.create(Some(&object)).unwrap();
        assert!(objects.inspect(&lookalike).unwrap().regexp_data().is_none());
        assert_eq!(objects.collect([&object], 100).unwrap().live, 2);
        let data = objects.inspect(&object).unwrap().regexp_data().unwrap();
        assert_eq!(data.source, source);
        assert_eq!(data.flags, JsString::from("yg"));
        assert_eq!(
            data.matcher
                .as_ref()
                .unwrap()
                .find_with_work(&source, 0, true, |_| Ok::<_, std::convert::Infallible>(()))
                .unwrap()
                .map(|found| found.range),
            Some(0..source.len())
        );
        assert!(matches!(
            objects.initialize_regexp(
                &object,
                RegExpData {
                    source: JsString::default(),
                    flags: JsString::default(),
                    matcher: None,
                    named_groups: Arc::from([]),
                }
            ),
            Err(Error::WrongKind)
        ));
        assert_eq!(objects.collect([&prototype], 100).unwrap().live, 1);
        assert!(matches!(
            objects.inspect(&object),
            Err(Error::Heap(spite_heap::Error::StaleHandle))
        ));
    }

    #[test]
    fn unicode_class_sequence_work_bound_tracks_proved_terms() {
        for count in [4usize, 5, 17] {
            let pattern = format!(r"\B{}\B", r"[[\p{Any}]]".repeat(count));
            let source = JsString::from(pattern.as_str());
            let body = RegExpUnicodeClassSequenceMatcher::compile_v_set_with_work(
                &source,
                false,
                false,
                |_| Ok::<(), std::convert::Infallible>(()),
            )
            .unwrap()
            .unwrap();
            let width = count * 2;
            assert_eq!(body.max_match_code_units(), width);
            let matcher = RegExpMatcher::new(RegExpMatcherBody::UnicodeClassSequence(body), 2);
            assert_eq!(matcher.capture_count(), 2);
            assert!(matcher.search_passes(true) > 0);
            assert_eq!(matcher.search_passes(true), matcher.search_passes(false));
            for source_len in [0, 1, source.len(), usize::MAX] {
                for remaining in 0..=width + 1 {
                    assert_eq!(
                        matcher.search_work(true, source_len, remaining),
                        width.min(remaining + 1)
                    );
                    assert_eq!(
                        matcher.search_work(false, source_len, remaining),
                        remaining + 1
                    );
                }
                assert_eq!(matcher.search_work(true, source_len, usize::MAX), width);
                assert_eq!(
                    matcher.search_work(false, source_len, usize::MAX),
                    usize::MAX
                );
            }
            let input = JsString::from(format!("{}!", "😀".repeat(count)).as_str());
            let mut work = Vec::new();
            let found = matcher
                .find_with_work(&input, 1, true, |units| {
                    work.push(units);
                    Ok::<(), std::convert::Infallible>(())
                })
                .unwrap()
                .unwrap();
            assert_eq!(work, [1]);
            assert_eq!(found.range, 0..width);
            assert_eq!(found.capture_count, 2);
            assert_eq!(found.capture(0), Some(0..width));
            assert_eq!(found.capture(1), Some(0..width));
            assert_eq!(found.capture(2), None);
            assert_eq!(
                matcher.search_work(true, source.len(), input.len() - 1),
                width
            );
            assert!(matches!(
                matcher.find_with_work(&input, 1, true, |_| Err(7)),
                Err(7)
            ));
            assert!(
                matcher
                    .find_with_work(&input, input.len() + 1, false, |_| {
                        Ok::<(), std::convert::Infallible>(())
                    })
                    .unwrap()
                    .is_none()
            );
        }
    }

    #[test]
    fn unicode_flat_class_sequence_work_bound_tracks_proved_terms() {
        for count in [3usize, 4, 17] {
            let pattern = format!(r"\B{}\B", r"[\p{Any}]".repeat(count));
            let source = JsString::from(pattern.as_str());
            let body = RegExpUnicodeClassSequenceMatcher::compile_flat_v_set_with_work(
                &source,
                false,
                false,
                |_| Ok::<(), std::convert::Infallible>(()),
            )
            .unwrap()
            .unwrap();
            let width = count * 2;
            assert_eq!(body.max_match_code_units(), width);
            let matcher = RegExpMatcher::new(RegExpMatcherBody::UnicodeClassSequence(body), 2);
            assert_eq!(matcher.capture_count(), 2);
            assert!(matcher.search_passes(true) > 0);
            assert_eq!(matcher.search_passes(true), matcher.search_passes(false));
            for source_len in [0, 1, source.len(), usize::MAX] {
                for remaining in 0..=width + 1 {
                    assert_eq!(
                        matcher.search_work(true, source_len, remaining),
                        width.min(remaining + 1)
                    );
                    assert_eq!(
                        matcher.search_work(false, source_len, remaining),
                        remaining + 1
                    );
                }
                assert_eq!(matcher.search_work(true, source_len, usize::MAX), width);
                assert_eq!(
                    matcher.search_work(false, source_len, usize::MAX),
                    usize::MAX
                );
            }
            let input = JsString::from(format!("{}!", "😀".repeat(count)).as_str());
            let mut work = Vec::new();
            let found = matcher
                .find_with_work(&input, 1, true, |units| {
                    work.push(units);
                    Ok::<(), std::convert::Infallible>(())
                })
                .unwrap()
                .unwrap();
            assert_eq!(work, [1]);
            assert_eq!(found.range, 0..width);
            assert_eq!(found.capture_count, 2);
            assert_eq!(found.capture(0), Some(0..width));
            assert_eq!(found.capture(1), Some(0..width));
            assert_eq!(found.capture(2), None);
            assert_eq!(
                matcher.search_work(true, source.len(), input.len() - 1),
                width
            );
            assert!(matches!(
                matcher.find_with_work(&input, 1, true, |_| Err(7)),
                Err(7)
            ));
            assert!(
                matcher
                    .find_with_work(&input, input.len() + 1, false, |_| {
                        Ok::<(), std::convert::Infallible>(())
                    })
                    .unwrap()
                    .is_none()
            );
        }
    }

    #[test]
    fn unicode_mixed_class_sequence_work_bound_tracks_proved_terms() {
        for count in [3usize, 4, 17] {
            let pattern = format!(r"\B{}{}\B", r"[\p{Any}]", r"[[\p{Any}]]".repeat(count - 1));
            let source = JsString::from(pattern.as_str());
            let body = RegExpUnicodeClassSequenceMatcher::compile_mixed_v_set_with_work(
                &source,
                false,
                false,
                |_| Ok::<(), std::convert::Infallible>(()),
            )
            .unwrap()
            .unwrap();
            let width = count * 2;
            assert_eq!(body.max_match_code_units(), width);
            let matcher = RegExpMatcher::new(RegExpMatcherBody::UnicodeClassSequence(body), 2);
            assert_eq!(matcher.capture_count(), 2);
            assert!(matcher.search_passes(true) > 0);
            assert_eq!(matcher.search_passes(true), matcher.search_passes(false));
            for source_len in [0, 1, source.len(), usize::MAX] {
                for remaining in 0..=width + 1 {
                    assert_eq!(
                        matcher.search_work(true, source_len, remaining),
                        width.min(remaining + 1)
                    );
                    assert_eq!(
                        matcher.search_work(false, source_len, remaining),
                        remaining + 1
                    );
                }
                assert_eq!(matcher.search_work(true, source_len, usize::MAX), width);
                assert_eq!(
                    matcher.search_work(false, source_len, usize::MAX),
                    usize::MAX
                );
            }
            let input = JsString::from(format!("{}!", "😀".repeat(count)).as_str());
            let mut work = Vec::new();
            let found = matcher
                .find_with_work(&input, 1, true, |units| {
                    work.push(units);
                    Ok::<(), std::convert::Infallible>(())
                })
                .unwrap()
                .unwrap();
            assert_eq!(work, [1]);
            assert_eq!(found.range, 0..width);
            assert_eq!(found.capture_count, 2);
            assert_eq!(found.capture(0), Some(0..width));
            assert_eq!(found.capture(1), Some(0..width));
            assert_eq!(found.capture(2), None);
            assert_eq!(
                matcher.search_work(true, source.len(), input.len() - 1),
                width
            );
            assert!(matches!(
                matcher.find_with_work(&input, 1, true, |_| Err(7)),
                Err(7)
            ));
            assert!(
                matcher
                    .find_with_work(&input, input.len() + 1, false, |_| {
                        Ok::<(), std::convert::Infallible>(())
                    })
                    .unwrap()
                    .is_none()
            );
        }
    }

    #[test]
    fn unicode_flat_class_literal_work_bound_tracks_proved_terms() {
        let source = JsString::from(r"[\p{Any}]\u{1f601}");
        let body = RegExpUnicodeClassLiteralMatcher::compile_flat_v_set_then_literal_with_work(
            &source,
            false,
            false,
            |_| Ok::<(), std::convert::Infallible>(()),
        )
        .unwrap()
        .unwrap();
        let matcher = RegExpMatcher::new(RegExpMatcherBody::UnicodeClassLiteral(body), 2);
        assert_eq!(matcher.capture_count(), 2);
        assert!(matcher.search_passes(true) > 0);
        assert_eq!(matcher.search_passes(true), matcher.search_passes(false));
        for source_len in [0, 1, source.len(), usize::MAX] {
            for remaining in 0usize..=5 {
                assert_eq!(
                    matcher.search_work(true, source_len, remaining),
                    4.min(remaining + 1)
                );
                assert_eq!(
                    matcher.search_work(false, source_len, remaining),
                    remaining + 1
                );
            }
            assert_eq!(matcher.search_work(true, source_len, usize::MAX), 4);
            assert_eq!(
                matcher.search_work(false, source_len, usize::MAX),
                usize::MAX
            );
        }
        let input = JsString::from("😀😁!");
        let mut work = Vec::new();
        let found = matcher
            .find_with_work(&input, 1, true, |units| {
                work.push(units);
                Ok::<(), std::convert::Infallible>(())
            })
            .unwrap()
            .unwrap();
        assert_eq!(work, [1]);
        assert_eq!(found.range, 0..4);
        assert_eq!(found.capture_count, 2);
        assert_eq!(found.capture(0), Some(0..4));
        assert_eq!(found.capture(1), Some(0..4));
        assert_eq!(found.capture(2), None);
        assert_eq!(matcher.search_work(true, source.len(), input.len() - 1), 4);
        assert!(matches!(
            matcher.find_with_work(&input, 1, true, |_| Err(7)),
            Err(7)
        ));
        assert!(
            matcher
                .find_with_work(&input, input.len() + 1, false, |_| {
                    Ok::<(), std::convert::Infallible>(())
                })
                .unwrap()
                .is_none()
        );
    }

    #[test]
    fn unicode_flat_class_literal_word_work_bound_tracks_proved_terms() {
        let source = JsString::from(r"\b[\p{Any}]\u{1f601}\B");
        let body = RegExpUnicodeClassLiteralMatcher::compile_flat_v_set_then_literal_with_word_boundaries_and_work(
            &source,
            false,
            false,
            |_| Ok::<(), std::convert::Infallible>(()),
        )
        .unwrap()
        .unwrap();
        let matcher = RegExpMatcher::new(RegExpMatcherBody::UnicodeClassLiteral(body), 2);
        assert_eq!(matcher.capture_count(), 2);
        assert!(matcher.search_passes(true) > 0);
        assert_eq!(matcher.search_passes(true), matcher.search_passes(false));
        for source_len in [0, 1, source.len(), usize::MAX] {
            for remaining in 0usize..=5 {
                assert_eq!(
                    matcher.search_work(true, source_len, remaining),
                    4.min(remaining + 1)
                );
                assert_eq!(
                    matcher.search_work(false, source_len, remaining),
                    remaining + 1
                );
            }
            assert_eq!(matcher.search_work(true, source_len, usize::MAX), 4);
            assert_eq!(
                matcher.search_work(false, source_len, usize::MAX),
                usize::MAX
            );
        }
        let input = JsString::from("A😀😁");
        let mut work = Vec::new();
        let found = matcher
            .find_with_work(&input, 2, true, |units| {
                work.push(units);
                Ok::<(), std::convert::Infallible>(())
            })
            .unwrap()
            .unwrap();
        assert_eq!(work, [1]);
        assert_eq!(found.range, 1..5);
        assert_eq!(found.capture_count, 2);
        assert_eq!(found.capture(0), Some(1..5));
        assert_eq!(found.capture(1), Some(1..5));
        assert_eq!(found.capture(2), None);
        assert_eq!(matcher.search_work(true, source.len(), input.len() - 2), 4);
        assert!(matches!(
            matcher.find_with_work(&input, 2, true, |_| Err(7)),
            Err(7)
        ));
        assert!(
            matcher
                .find_with_work(&input, input.len() + 1, false, |_| {
                    Ok::<(), std::convert::Infallible>(())
                })
                .unwrap()
                .is_none()
        );
    }

    #[test]
    fn unicode_literal_flat_class_work_bound_tracks_proved_terms() {
        let source = JsString::from(r"\u{1f600}[\p{Any}]");
        let body = RegExpUnicodeClassLiteralMatcher::compile_literal_then_flat_v_set_with_work(
            &source,
            false,
            false,
            |_| Ok::<(), std::convert::Infallible>(()),
        )
        .unwrap()
        .unwrap();
        let matcher = RegExpMatcher::new(RegExpMatcherBody::UnicodeClassLiteral(body), 2);
        assert_eq!(matcher.capture_count(), 2);
        assert!(matcher.search_passes(true) > 0);
        assert_eq!(matcher.search_passes(true), matcher.search_passes(false));
        for source_len in [0, 1, source.len(), usize::MAX] {
            for remaining in 0usize..=5 {
                assert_eq!(
                    matcher.search_work(true, source_len, remaining),
                    4.min(remaining + 1)
                );
                assert_eq!(
                    matcher.search_work(false, source_len, remaining),
                    remaining + 1
                );
            }
            assert_eq!(matcher.search_work(true, source_len, usize::MAX), 4);
            assert_eq!(
                matcher.search_work(false, source_len, usize::MAX),
                usize::MAX
            );
        }
        let input = JsString::from("A😀😁!");
        let mut work = Vec::new();
        let found = matcher
            .find_with_work(&input, 2, true, |units| {
                work.push(units);
                Ok::<(), std::convert::Infallible>(())
            })
            .unwrap()
            .unwrap();
        assert_eq!(work, [1]);
        assert_eq!(found.range, 1..5);
        assert_eq!(found.capture_count, 2);
        assert_eq!(found.capture(0), Some(1..5));
        assert_eq!(found.capture(1), Some(1..5));
        assert_eq!(found.capture(2), None);
        assert_eq!(matcher.search_work(true, source.len(), input.len() - 2), 4);
        assert!(matches!(
            matcher.find_with_work(&input, 2, true, |_| Err(7)),
            Err(7)
        ));
        assert!(
            matcher
                .find_with_work(&input, input.len() + 1, false, |_| {
                    Ok::<(), std::convert::Infallible>(())
                })
                .unwrap()
                .is_none()
        );
    }

    #[test]
    fn unicode_literal_flat_class_word_work_bound_tracks_proved_terms() {
        let source = JsString::from(r"\b\u{1f600}[\p{Any}]\B");
        let body = RegExpUnicodeClassLiteralMatcher::compile_literal_then_flat_v_set_with_word_boundaries_and_work(
            &source,
            false,
            false,
            |_| Ok::<(), std::convert::Infallible>(()),
        )
        .unwrap()
        .unwrap();
        let matcher = RegExpMatcher::new(RegExpMatcherBody::UnicodeClassLiteral(body), 2);
        assert_eq!(matcher.capture_count(), 2);
        assert!(matcher.search_passes(true) > 0);
        assert_eq!(matcher.search_passes(true), matcher.search_passes(false));
        for source_len in [0, 1, source.len(), usize::MAX] {
            for remaining in 0usize..=5 {
                assert_eq!(
                    matcher.search_work(true, source_len, remaining),
                    4.min(remaining + 1)
                );
                assert_eq!(
                    matcher.search_work(false, source_len, remaining),
                    remaining + 1
                );
            }
            assert_eq!(matcher.search_work(true, source_len, usize::MAX), 4);
            assert_eq!(
                matcher.search_work(false, source_len, usize::MAX),
                usize::MAX
            );
        }
        let input = JsString::from("A😀😁!");
        let mut work = Vec::new();
        let found = matcher
            .find_with_work(&input, 2, true, |units| {
                work.push(units);
                Ok::<(), std::convert::Infallible>(())
            })
            .unwrap()
            .unwrap();
        assert_eq!(work, [1]);
        assert_eq!(found.range, 1..5);
        assert_eq!(found.capture_count, 2);
        assert_eq!(found.capture(0), Some(1..5));
        assert_eq!(found.capture(1), Some(1..5));
        assert_eq!(found.capture(2), None);
        assert_eq!(matcher.search_work(true, source.len(), input.len() - 2), 4);
        assert!(matches!(
            matcher.find_with_work(&input, 2, true, |_| Err(7)),
            Err(7)
        ));
        assert!(
            matcher
                .find_with_work(&input, input.len() + 1, false, |_| {
                    Ok::<(), std::convert::Infallible>(())
                })
                .unwrap()
                .is_none()
        );
    }

    #[test]
    fn unicode_flat_class_literal_flat_class_work_bound_tracks_proved_terms() {
        let source = JsString::from(r"\B[\p{Any}]\u{1f601}[\p{Any}]\B");
        let body = RegExpUnicodeClassLiteralClassWordMatcher::compile_flat_v_set_with_work(
            &source,
            false,
            false,
            |_| Ok::<(), std::convert::Infallible>(()),
        )
        .unwrap()
        .unwrap();
        let matcher = RegExpMatcher::new(RegExpMatcherBody::UnicodeClassLiteralClassWord(body), 2);
        assert_eq!(matcher.capture_count(), 2);
        assert!(matcher.search_passes(true) > 0);
        assert_eq!(matcher.search_passes(true), matcher.search_passes(false));
        for source_len in [0, 1, source.len(), usize::MAX] {
            for remaining in 0usize..=7 {
                assert_eq!(
                    matcher.search_work(true, source_len, remaining),
                    6.min(remaining + 1)
                );
                assert_eq!(
                    matcher.search_work(false, source_len, remaining),
                    remaining + 1
                );
            }
            assert_eq!(matcher.search_work(true, source_len, usize::MAX), 6);
            assert_eq!(
                matcher.search_work(false, source_len, usize::MAX),
                usize::MAX
            );
        }
        let input = JsString::from("😀😁😀");
        for sticky in [false, true] {
            let mut work = Vec::new();
            let found = matcher
                .find_with_work(&input, 1, sticky, |units| {
                    work.push(units);
                    Ok::<(), std::convert::Infallible>(())
                })
                .unwrap()
                .unwrap();
            assert_eq!(work, [1]);
            assert_eq!(found.range, 0..6);
            assert_eq!(found.capture_count, 2);
            assert_eq!(found.capture(0), Some(0..6));
            assert_eq!(found.capture(1), Some(0..6));
            assert_eq!(found.capture(2), None);
        }
        assert_eq!(matcher.search_work(true, source.len(), input.len() - 1), 6);
        assert!(matches!(
            matcher.find_with_work(&input, 1, true, |_| Err(7)),
            Err(7)
        ));
        assert!(
            matcher
                .find_with_work(&input, input.len() + 1, false, |_| Ok::<
                    (),
                    std::convert::Infallible,
                >(()))
                .unwrap()
                .is_none()
        );
        assert!(matches!(
            RegExpUnicodeClassLiteralClassWordMatcher::compile_flat_v_set_with_work(
                &source,
                false,
                false,
                |_| Err(9)
            ),
            Err(spite_core::RegExpUnicodeSetCompileError::Work(9))
        ));
    }

    #[test]
    fn unicode_literal_flat_class_pair_work_bound_tracks_proved_terms() {
        let source = JsString::from(r"\b\u{1f600}[\p{Any}][\p{Any}]\B");
        let body = RegExpUnicodeLiteralClassPairWordMatcher::compile_flat_v_set_with_work(
            &source,
            false,
            false,
            |_| Ok::<(), std::convert::Infallible>(()),
        )
        .unwrap()
        .unwrap();
        let matcher = RegExpMatcher::new(RegExpMatcherBody::UnicodeLiteralClassPairWord(body), 2);
        assert_eq!(matcher.capture_count(), 2);
        assert!(matcher.search_passes(true) > 0);
        assert_eq!(matcher.search_passes(true), matcher.search_passes(false));
        for source_len in [0, 1, source.len(), usize::MAX] {
            for remaining in 0usize..=7 {
                assert_eq!(
                    matcher.search_work(true, source_len, remaining),
                    6.min(remaining + 1)
                );
                assert_eq!(
                    matcher.search_work(false, source_len, remaining),
                    remaining + 1
                );
            }
            assert_eq!(matcher.search_work(true, source_len, usize::MAX), 6);
            assert_eq!(
                matcher.search_work(false, source_len, usize::MAX),
                usize::MAX
            );
        }
        let input = JsString::from("A😀😁😀");
        for sticky in [false, true] {
            let mut work = Vec::new();
            let found = matcher
                .find_with_work(&input, 2, sticky, |n| {
                    work.push(n);
                    Ok::<(), std::convert::Infallible>(())
                })
                .unwrap()
                .unwrap();
            assert_eq!(work, [1]);
            assert_eq!(found.range, 1..7);
            assert_eq!(found.capture_count, 2);
            assert_eq!(found.capture(0), Some(1..7));
            assert_eq!(found.capture(1), Some(1..7));
            assert_eq!(found.capture(2), None);
        }
        assert_eq!(matcher.search_work(true, source.len(), input.len() - 2), 6);
        assert!(matches!(
            matcher.find_with_work(&input, 2, true, |_| Err(7)),
            Err(7)
        ));
        assert!(
            matcher
                .find_with_work(&input, input.len() + 1, false, |_| Ok::<
                    (),
                    std::convert::Infallible,
                >(()))
                .unwrap()
                .is_none()
        );
        assert!(matches!(
            RegExpUnicodeLiteralClassPairWordMatcher::compile_flat_v_set_with_work(
                &source,
                false,
                false,
                |_| Err(9)
            ),
            Err(spite_core::RegExpUnicodeSetCompileError::Work(9))
        ));
    }

    #[test]
    fn unicode_flat_class_pair_literal_work_bound_tracks_proved_terms() {
        let source = JsString::from(r"\b[\p{Any}][\p{Any}]\u{1f600}\B");
        let body = RegExpUnicodeClassPairLiteralWordMatcher::compile_flat_v_set_with_work(
            &source,
            false,
            false,
            |_| Ok::<(), std::convert::Infallible>(()),
        )
        .unwrap()
        .unwrap();
        let matcher = RegExpMatcher::new(RegExpMatcherBody::UnicodeClassPairLiteralWord(body), 2);
        let old_source = JsString::from(r"[\p{Any}]\u{1f600}");
        let old_body = RegExpUnicodeClassLiteralMatcher::compile_flat_v_set_then_literal_with_work(
            &old_source,
            false,
            false,
            |_| Ok::<(), std::convert::Infallible>(()),
        )
        .unwrap()
        .unwrap();
        let old_four = RegExpMatcher::new(RegExpMatcherBody::UnicodeClassLiteral(old_body), 0);
        for source_len in [0, 1, source.len(), usize::MAX] {
            for remaining in 0usize..=7 {
                assert_eq!(
                    matcher.search_work(true, source_len, remaining),
                    6.min(remaining + 1)
                );
                assert_eq!(
                    matcher.search_work(false, source_len, remaining),
                    remaining + 1
                );
                assert_eq!(
                    old_four.search_work(true, source_len, remaining),
                    4.min(remaining + 1)
                );
                assert_eq!(
                    old_four.search_work(false, source_len, remaining),
                    remaining + 1
                );
            }
            assert_eq!(matcher.search_work(true, source_len, usize::MAX), 6);
            assert_eq!(
                matcher.search_work(false, source_len, usize::MAX),
                usize::MAX
            );
            assert_eq!(old_four.search_work(true, source_len, usize::MAX), 4);
            assert_eq!(
                old_four.search_work(false, source_len, usize::MAX),
                usize::MAX
            );
        }
        // No trailing unit: start2 leaves five units and really needs the sixth rewind unit.
        let input = JsString::from("A😀😁😀");
        for sticky in [false, true] {
            let mut work = Vec::new();
            let found = matcher
                .find_with_work(&input, 2, sticky, |n| {
                    work.push(n);
                    Ok::<(), std::convert::Infallible>(())
                })
                .unwrap()
                .unwrap();
            assert_eq!(work, [1]);
            assert_eq!(found.range, 1..7);
            assert_eq!(found.capture_count, 2);
            assert_eq!(found.capture(0), Some(1..7));
            assert_eq!(found.capture(1), Some(1..7));
            assert_eq!(found.capture(2), None);
        }
        assert_eq!(matcher.search_work(true, source.len(), input.len() - 2), 6);
        assert!(matches!(
            matcher.find_with_work(&input, 2, true, |_| Err(7)),
            Err(7)
        ));
        assert!(
            matcher
                .find_with_work(&input, input.len() + 1, false, |_| Ok::<
                    (),
                    std::convert::Infallible,
                >(()),)
                .unwrap()
                .is_none()
        );
        assert!(matches!(
            RegExpUnicodeClassPairLiteralWordMatcher::compile_flat_v_set_with_work(
                &source,
                false,
                false,
                |_| Err(9)
            ),
            Err(spite_core::RegExpUnicodeSetCompileError::Work(9))
        ));
    }

    #[test]
    fn unicode_literal_checked_flat_class_pair_work_bound_tracks_proved_terms() {
        let source = JsString::from(r"\b\u{1f600}[[\u{1f601}]][\u{1f600}]\B");
        let body =
            RegExpUnicodeLiteralClassPairWordMatcher::compile_checked_then_flat_v_set_with_work(
                &source,
                false,
                false,
                |_| Ok::<(), std::convert::Infallible>(()),
            )
            .unwrap()
            .unwrap();
        let matcher = RegExpMatcher::new(RegExpMatcherBody::UnicodeLiteralClassPairWord(body), 2);
        for source_len in [0, 1, source.len(), usize::MAX] {
            for remaining in 0usize..=7 {
                assert_eq!(
                    matcher.search_work(true, source_len, remaining),
                    6.min(remaining + 1)
                );
                assert_eq!(
                    matcher.search_work(false, source_len, remaining),
                    remaining + 1
                );
            }
            assert_eq!(matcher.search_work(true, source_len, usize::MAX), 6);
            assert_eq!(
                matcher.search_work(false, source_len, usize::MAX),
                usize::MAX
            );
        }
        // No trailing unit: initial low-pair start2 leaves five units and needs the sixth.
        let input = JsString::from("A😀😁😀");
        for sticky in [false, true] {
            let mut work = Vec::new();
            let found = matcher
                .find_with_work(&input, 2, sticky, |n| {
                    work.push(n);
                    Ok::<(), std::convert::Infallible>(())
                })
                .unwrap()
                .unwrap();
            assert_eq!(work, [1]);
            assert_eq!(found.range, 1..7);
            assert_eq!(found.capture_count, 2);
            assert_eq!(found.capture(0), Some(1..7));
            assert_eq!(found.capture(1), Some(1..7));
            assert_eq!(found.capture(2), None);
        }
        assert_eq!(matcher.search_work(true, source.len(), input.len() - 2), 6);
        assert!(matches!(
            matcher.find_with_work(&input, 2, true, |_| Err(7)),
            Err(7)
        ));
        assert!(
            matcher
                .find_with_work(&input, input.len() + 1, false, |_| Ok::<
                    (),
                    std::convert::Infallible,
                >(()))
                .unwrap()
                .is_none()
        );
        assert!(matches!(
            RegExpUnicodeLiteralClassPairWordMatcher::compile_checked_then_flat_v_set_with_work(
                &source,
                false,
                false,
                |_| Err(9)
            ),
            Err(spite_core::RegExpUnicodeSetCompileError::Work(9))
        ));
    }

    #[test]
    fn unicode_literal_flat_checked_class_pair_work_bound_tracks_proved_terms() {
        let source = JsString::from(r"\b\u{1f600}[\u{1f601}][[\u{1f600}]]\B");
        let body =
            RegExpUnicodeLiteralClassPairWordMatcher::compile_flat_then_checked_v_set_with_work(
                &source,
                false,
                false,
                |_| Ok::<(), std::convert::Infallible>(()),
            )
            .unwrap()
            .unwrap();
        let matcher = RegExpMatcher::new(RegExpMatcherBody::UnicodeLiteralClassPairWord(body), 2);
        for source_len in [0, 1, source.len(), usize::MAX] {
            for remaining in 0usize..=7 {
                assert_eq!(
                    matcher.search_work(true, source_len, remaining),
                    6.min(remaining + 1)
                );
                assert_eq!(
                    matcher.search_work(false, source_len, remaining),
                    remaining + 1
                );
            }
            assert_eq!(matcher.search_work(true, source_len, usize::MAX), 6);
            assert_eq!(
                matcher.search_work(false, source_len, usize::MAX),
                usize::MAX
            );
        }
        // No trailing unit: initial low-pair start2 leaves five units and needs the sixth.
        let input = JsString::from("A😀😁😀");
        for sticky in [false, true] {
            let mut work = Vec::new();
            let found = matcher
                .find_with_work(&input, 2, sticky, |n| {
                    work.push(n);
                    Ok::<(), std::convert::Infallible>(())
                })
                .unwrap()
                .unwrap();
            assert_eq!(work, [1]);
            assert_eq!(found.range, 1..7);
            assert_eq!(found.capture_count, 2);
            assert_eq!(found.capture(0), Some(1..7));
            assert_eq!(found.capture(1), Some(1..7));
            assert_eq!(found.capture(2), None);
        }
        assert_eq!(matcher.search_work(true, source.len(), input.len() - 2), 6);
        assert!(matches!(
            matcher.find_with_work(&input, 2, true, |_| Err(7)),
            Err(7)
        ));
        assert!(
            matcher
                .find_with_work(&input, input.len() + 1, false, |_| Ok::<
                    (),
                    std::convert::Infallible,
                >(()))
                .unwrap()
                .is_none()
        );
        assert!(matches!(
            RegExpUnicodeLiteralClassPairWordMatcher::compile_flat_then_checked_v_set_with_work(
                &source,
                false,
                false,
                |_| Err(9)
            ),
            Err(spite_core::RegExpUnicodeSetCompileError::Work(9))
        ));
    }
    #[test]
    fn unicode_checked_flat_class_pair_literal_work_bound_tracks_proved_terms() {
        let source = JsString::from(r"\b[[\u{1f600}]][\u{1f601}]\u{1f600}\B");
        let body =
            RegExpUnicodeClassPairLiteralWordMatcher::compile_checked_then_flat_v_set_with_work(
                &source,
                false,
                false,
                |_| Ok::<(), std::convert::Infallible>(()),
            )
            .unwrap()
            .unwrap();
        let matcher = RegExpMatcher::new(RegExpMatcherBody::UnicodeClassPairLiteralWord(body), 2);
        for source_len in [0, 1, source.len(), usize::MAX] {
            for remaining in 0usize..=7 {
                assert_eq!(
                    matcher.search_work(true, source_len, remaining),
                    6.min(remaining + 1)
                );
                assert_eq!(
                    matcher.search_work(false, source_len, remaining),
                    remaining + 1
                );
            }
            assert_eq!(matcher.search_work(true, source_len, usize::MAX), 6);
            assert_eq!(
                matcher.search_work(false, source_len, usize::MAX),
                usize::MAX
            );
        }
        // No trailing unit: initial low-pair start2 leaves five units and needs the sixth.
        let input = JsString::from("A😀😁😀");
        for sticky in [false, true] {
            let mut work = Vec::new();
            let found = matcher
                .find_with_work(&input, 2, sticky, |n| {
                    work.push(n);
                    Ok::<(), std::convert::Infallible>(())
                })
                .unwrap()
                .unwrap();
            assert_eq!(work, [1]);
            assert_eq!(found.range, 1..7);
            assert_eq!(found.capture_count, 2);
            assert_eq!(found.capture(0), Some(1..7));
            assert_eq!(found.capture(1), Some(1..7));
            assert_eq!(found.capture(2), None);
        }
        assert_eq!(matcher.search_work(true, source.len(), input.len() - 2), 6);
        assert!(matches!(
            matcher.find_with_work(&input, 2, true, |_| Err(7)),
            Err(7)
        ));
        assert!(
            matcher
                .find_with_work(&input, input.len() + 1, false, |_| Ok::<
                    (),
                    std::convert::Infallible,
                >(()))
                .unwrap()
                .is_none()
        );
        assert!(matches!(
            RegExpUnicodeClassPairLiteralWordMatcher::compile_checked_then_flat_v_set_with_work(
                &source,
                false,
                false,
                |_| Err(9)
            ),
            Err(spite_core::RegExpUnicodeSetCompileError::Work(9))
        ));
    }
    #[test]
    fn unicode_flat_checked_class_pair_literal_work_bound_tracks_proved_terms() {
        let source = JsString::from(r"\b[\u{1f600}][[\u{1f601}]]\u{1f600}\B");
        let body =
            RegExpUnicodeClassPairLiteralWordMatcher::compile_flat_then_checked_v_set_with_work(
                &source,
                false,
                false,
                |_| Ok::<(), std::convert::Infallible>(()),
            )
            .unwrap()
            .unwrap();
        let matcher = RegExpMatcher::new(RegExpMatcherBody::UnicodeClassPairLiteralWord(body), 2);
        for source_len in [0, 1, source.len(), usize::MAX] {
            for remaining in 0usize..=7 {
                assert_eq!(
                    matcher.search_work(true, source_len, remaining),
                    6.min(remaining + 1)
                );
                assert_eq!(
                    matcher.search_work(false, source_len, remaining),
                    remaining + 1
                );
            }
            assert_eq!(matcher.search_work(true, source_len, usize::MAX), 6);
            assert_eq!(
                matcher.search_work(false, source_len, usize::MAX),
                usize::MAX
            );
        }
        // No trailing unit: initial low-pair start2 leaves five units and needs the sixth.
        let input = JsString::from("A😀😁😀");
        for sticky in [false, true] {
            let mut work = Vec::new();
            let found = matcher
                .find_with_work(&input, 2, sticky, |n| {
                    work.push(n);
                    Ok::<(), std::convert::Infallible>(())
                })
                .unwrap()
                .unwrap();
            assert_eq!(work, [1]);
            assert_eq!(found.range, 1..7);
            assert_eq!(found.capture_count, 2);
            assert_eq!(found.capture(0), Some(1..7));
            assert_eq!(found.capture(1), Some(1..7));
            assert_eq!(found.capture(2), None);
        }
        assert_eq!(matcher.search_work(true, source.len(), input.len() - 2), 6);
        assert!(matches!(
            matcher.find_with_work(&input, 2, true, |_| Err(7)),
            Err(7)
        ));
        assert!(
            matcher
                .find_with_work(&input, input.len() + 1, false, |_| Ok::<
                    (),
                    std::convert::Infallible,
                >(()))
                .unwrap()
                .is_none()
        );
        assert!(matches!(
            RegExpUnicodeClassPairLiteralWordMatcher::compile_flat_then_checked_v_set_with_work(
                &source,
                false,
                false,
                |_| Err(9)
            ),
            Err(spite_core::RegExpUnicodeSetCompileError::Work(9))
        ));
    }
    #[test]
    fn unicode_checked_class_literal_flat_class_work_bound_tracks_proved_terms() {
        let source = JsString::from(r"\b[[\p{Any}]]\u{1f601}[\p{Any}]\B");
        let body =
            RegExpUnicodeClassLiteralClassWordMatcher::compile_checked_then_flat_v_set_with_work(
                &source,
                false,
                false,
                |_| Ok::<(), std::convert::Infallible>(()),
            )
            .unwrap()
            .unwrap();
        let matcher = RegExpMatcher::new(RegExpMatcherBody::UnicodeClassLiteralClassWord(body), 2);
        assert_eq!(matcher.capture_count(), 2);
        assert!(matcher.search_passes(true) > 0);
        assert_eq!(matcher.search_passes(true), matcher.search_passes(false));
        for source_len in [0, 1, source.len(), usize::MAX] {
            for remaining in 0usize..=7 {
                assert_eq!(
                    matcher.search_work(true, source_len, remaining),
                    6.min(remaining + 1)
                );
                assert_eq!(
                    matcher.search_work(false, source_len, remaining),
                    remaining + 1
                );
            }
            assert_eq!(matcher.search_work(true, source_len, usize::MAX), 6);
            assert_eq!(
                matcher.search_work(false, source_len, usize::MAX),
                usize::MAX
            );
        }
        let input = JsString::from("A😀😁😀!");
        for sticky in [false, true] {
            let mut work = Vec::new();
            let found = matcher
                .find_with_work(&input, 2, sticky, |units| {
                    work.push(units);
                    Ok::<(), std::convert::Infallible>(())
                })
                .unwrap()
                .unwrap();
            assert_eq!(work, [1]);
            assert_eq!(found.range, 1..7);
            assert_eq!(found.capture_count, 2);
            assert_eq!(found.capture(0), Some(1..7));
            assert_eq!(found.capture(1), Some(1..7));
            assert_eq!(found.capture(2), None);
        }
        assert_eq!(matcher.search_work(true, source.len(), input.len() - 2), 6);
        assert!(matches!(
            matcher.find_with_work(&input, 2, true, |_| Err(7)),
            Err(7)
        ));
        assert!(
            matcher
                .find_with_work(&input, input.len() + 1, false, |_| Ok::<
                    (),
                    std::convert::Infallible,
                >(()))
                .unwrap()
                .is_none()
        );
        assert!(matches!(
            RegExpUnicodeClassLiteralClassWordMatcher::compile_checked_then_flat_v_set_with_work(
                &source,
                false,
                false,
                |_| Err(9)
            ),
            Err(spite_core::RegExpUnicodeSetCompileError::Work(9))
        ));
    }
    #[test]
    fn unicode_flat_class_literal_checked_class_work_bound_tracks_proved_terms() {
        let source = JsString::from(r"\b[\p{Any}]\u{1f601}[[\p{Any}]]\B");
        let body =
            RegExpUnicodeClassLiteralClassWordMatcher::compile_flat_then_checked_v_set_with_work(
                &source,
                false,
                false,
                |_| Ok::<(), std::convert::Infallible>(()),
            )
            .unwrap()
            .unwrap();
        let matcher = RegExpMatcher::new(RegExpMatcherBody::UnicodeClassLiteralClassWord(body), 2);
        assert_eq!(matcher.capture_count(), 2);
        assert!(matcher.search_passes(true) > 0);
        assert_eq!(matcher.search_passes(true), matcher.search_passes(false));
        for source_len in [0, 1, source.len(), usize::MAX] {
            for remaining in 0usize..=7 {
                assert_eq!(
                    matcher.search_work(true, source_len, remaining),
                    6.min(remaining + 1)
                );
                assert_eq!(
                    matcher.search_work(false, source_len, remaining),
                    remaining + 1
                );
            }
            assert_eq!(matcher.search_work(true, source_len, usize::MAX), 6);
            assert_eq!(
                matcher.search_work(false, source_len, usize::MAX),
                usize::MAX
            );
        }
        let input = JsString::from("A😀😁😀!");
        for sticky in [false, true] {
            let mut work = Vec::new();
            let found = matcher
                .find_with_work(&input, 2, sticky, |units| {
                    work.push(units);
                    Ok::<(), std::convert::Infallible>(())
                })
                .unwrap()
                .unwrap();
            assert_eq!(work, [1]);
            assert_eq!(found.range, 1..7);
            assert_eq!(found.capture_count, 2);
            assert_eq!(found.capture(0), Some(1..7));
            assert_eq!(found.capture(1), Some(1..7));
            assert_eq!(found.capture(2), None);
        }
        assert_eq!(matcher.search_work(true, source.len(), input.len() - 2), 6);
        assert!(matches!(
            matcher.find_with_work(&input, 2, true, |_| Err(7)),
            Err(7)
        ));
        assert!(
            matcher
                .find_with_work(&input, input.len() + 1, false, |_| Ok::<
                    (),
                    std::convert::Infallible,
                >(()))
                .unwrap()
                .is_none()
        );
        assert!(matches!(
            RegExpUnicodeClassLiteralClassWordMatcher::compile_flat_then_checked_v_set_with_work(
                &source,
                false,
                false,
                |_| Err(9)
            ),
            Err(spite_core::RegExpUnicodeSetCompileError::Work(9))
        ));
    }

    #[test]
    fn single_character_v_strings_work_bound_tracks_original_characters() {
        let source = JsString::from(r"[\q{\uD83D\uDE00|x}]");
        let body = RegExpUnicodeClassMatcher::compile_single_character_strings_v_set_with_work(
            &source,
            false,
            false,
            |_| Ok::<(), std::convert::Infallible>(()),
        )
        .unwrap()
        .unwrap();
        let matcher = RegExpMatcher::new(RegExpMatcherBody::UnicodeClass(body), 2);
        for source_len in [0, 1, source.len(), usize::MAX] {
            for remaining in 0usize..=3 {
                assert_eq!(
                    matcher.search_work(true, source_len, remaining),
                    [1, 2, 2, 2][remaining]
                );
                assert_eq!(
                    matcher.search_work(false, source_len, remaining),
                    remaining + 1
                );
            }
            assert_eq!(matcher.search_work(true, source_len, usize::MAX), 2);
            assert_eq!(
                matcher.search_work(false, source_len, usize::MAX),
                usize::MAX
            );
        }
        let input = JsString::from("A😀!");
        // BuiltinExec precharges this fixed character plan via search_work;
        // unlike composite searches it needs no incremental work callback.
        let found = matcher
            .find_with_work(&input, 2, true, |_| -> Result<(), u8> {
                panic!("fixed character matching is precharged")
            })
            .unwrap()
            .unwrap();
        assert_eq!(found.range, 1..3);
        assert_eq!(found.capture_count, 2);
        assert_eq!(found.capture(0), Some(1..3));
        assert_eq!(found.capture(1), Some(1..3));
        assert_eq!(found.capture(2), None);
        assert!(
            matcher
                .find_with_work(&input, 0, true, |_| Ok::<(), u8>(()))
                .unwrap()
                .is_none()
        );
        assert!(matches!(
            RegExpUnicodeClassMatcher::compile_single_character_strings_v_set_with_work(
                &source,
                false,
                false,
                |_| Err(41u8),
            ),
            Err(spite_core::RegExpUnicodeSetCompileError::Work(41))
        ));
    }

    #[test]
    fn single_character_v_string_operands_work_bound_tracks_original_characters() {
        let source = JsString::from(r"[\q{\uD83D\uDE00|x}&&[^\q{x}]]");
        let body = RegExpUnicodeClassMatcher::compile_v_set_with_single_character_strings_and_work(
            &source,
            false,
            false,
            |_| Ok::<(), std::convert::Infallible>(()),
        )
        .unwrap()
        .unwrap();
        let matcher = RegExpMatcher::new(RegExpMatcherBody::UnicodeClass(body), 2);
        assert_eq!(matcher.search_work(true, source.len(), usize::MAX), 2);
        assert_eq!(matcher.search_work(false, source.len(), 3), 4);
        assert_eq!(
            matcher.search_work(false, source.len(), usize::MAX),
            usize::MAX
        );
        // This fixed character plan retains the existing BuiltinExec precharge.
        let found = matcher
            .find_with_work(&JsString::from("A😀!"), 2, true, |_| -> Result<(), u8> {
                panic!("fixed character matching is precharged")
            })
            .unwrap()
            .unwrap();
        assert_eq!(found.range, 1..3);
        assert_eq!(found.capture_count, 2);
        assert_eq!(found.capture(0), Some(1..3));
        assert_eq!(found.capture(1), Some(1..3));
        assert!(matches!(
            RegExpUnicodeClassMatcher::compile_v_set_with_single_character_strings_and_work(
                &source,
                false,
                false,
                |_| Err(73u8),
            ),
            Err(spite_core::RegExpUnicodeSetCompileError::Work(73))
        ));
    }

    #[test]
    fn single_character_v_string_repetition_work_remains_incremental_for_long_runs() {
        let source = JsString::from(r"[\q{a|b}&&[b]]+");
        let body =
            RegExpUnicodeRepeatedAtomMatcher::compile_v_set_with_single_character_strings_and_work(
                &source,
                false,
                false,
                |_| Ok::<(), std::convert::Infallible>(()),
            )
            .unwrap()
            .unwrap();
        let matcher = RegExpMatcher::new(RegExpMatcherBody::UnicodeRepeatedAtom(body), 2);
        let input = JsString::from("b".repeat(128).as_str());
        assert!(source.len() < input.len());
        assert_eq!(matcher.search_passes(true), 0);
        assert_eq!(matcher.search_passes(false), 0);
        assert_eq!(
            matcher.search_work(true, source.len(), input.len()),
            input.len()
        );
        assert_eq!(
            matcher.search_work(false, source.len(), input.len()),
            input.len()
        );
        let mut calls = 0;
        let found = matcher
            .find_with_work(&input, 0, true, |_| {
                calls += 1;
                Ok::<(), u8>(())
            })
            .unwrap()
            .unwrap();
        assert!(calls > 0);
        assert_eq!(found.range, 0..128);
        assert_eq!(found.capture_count, 2);
        assert_eq!(found.capture(0), Some(0..128));
        assert_eq!(found.capture(1), Some(0..128));
        assert_eq!(found.capture(2), None);
        assert!(matches!(
            matcher.find_with_work(&input, 0, true, |_| Err(19u8)),
            Err(19)
        ));
    }

    #[test]
    fn literal_single_character_v_string_class_work_bound_tracks_proved_terms() {
        let source = JsString::from(r"\b\u{1f600}[\q{\uD83D\uDE01|x}--x]\B");
        let body = RegExpUnicodeClassLiteralMatcher::compile_literal_then_v_set_with_single_character_strings_and_work(
            &source,
            false,
            false,
            |_| Ok::<(), std::convert::Infallible>(()),
        )
        .unwrap()
        .unwrap();
        let matcher = RegExpMatcher::new(RegExpMatcherBody::UnicodeClassLiteral(body), 2);
        assert_eq!(matcher.capture_count(), 2);
        assert!(matcher.search_passes(true) > 0);
        assert_eq!(matcher.search_passes(true), matcher.search_passes(false));
        for source_len in [0, 1, source.len(), usize::MAX] {
            for remaining in 0usize..=5 {
                assert_eq!(
                    matcher.search_work(true, source_len, remaining),
                    4.min(remaining + 1)
                );
                assert_eq!(
                    matcher.search_work(false, source_len, remaining),
                    remaining + 1
                );
            }
            assert_eq!(matcher.search_work(true, source_len, usize::MAX), 4);
            assert_eq!(
                matcher.search_work(false, source_len, usize::MAX),
                usize::MAX
            );
        }
        let input = JsString::from("A😀😁!");
        let mut work = Vec::new();
        let found = matcher
            .find_with_work(&input, 2, true, |units| {
                work.push(units);
                Ok::<(), std::convert::Infallible>(())
            })
            .unwrap()
            .unwrap();
        assert_eq!(work, [1]);
        assert_eq!(found.range, 1..5);
        assert_eq!(found.capture_count, 2);
        assert_eq!(found.capture(0), Some(1..5));
        assert_eq!(found.capture(1), Some(1..5));
        assert_eq!(found.capture(2), None);
        assert_eq!(matcher.search_work(true, source.len(), input.len() - 2), 4);
        assert!(matches!(
            matcher.find_with_work(&input, 2, true, |_| Err(29u8)),
            Err(29)
        ));
        assert!(
            matcher
                .find_with_work(&input, input.len() + 1, false, |_| {
                    Ok::<(), std::convert::Infallible>(())
                })
                .unwrap()
                .is_none()
        );
        assert!(matches!(
            RegExpUnicodeClassLiteralMatcher::compile_literal_then_v_set_with_single_character_strings_and_work(
                &source,
                false,
                false,
                |_| Err(43u8),
            ),
            Err(spite_core::RegExpUnicodeSetCompileError::Work(43))
        ));
    }

    #[test]
    fn single_character_v_string_class_literal_work_bound_tracks_proved_terms() {
        let source = JsString::from(r"\b[\q{\uD83D\uDE00|x}--x]\u{1f680}\B");
        let body = RegExpUnicodeClassLiteralMatcher::compile_v_set_then_literal_with_single_character_strings_and_work(
            &source,
            false,
            false,
            |_| Ok::<(), std::convert::Infallible>(()),
        )
        .unwrap()
        .unwrap();
        let matcher = RegExpMatcher::new(RegExpMatcherBody::UnicodeClassLiteral(body), 2);
        assert_eq!(matcher.capture_count(), 2);
        assert!(matcher.search_passes(true) > 0);
        assert_eq!(matcher.search_passes(true), matcher.search_passes(false));
        for source_len in [0, 1, source.len(), usize::MAX] {
            for remaining in 0usize..=5 {
                assert_eq!(
                    matcher.search_work(true, source_len, remaining),
                    4.min(remaining + 1)
                );
                assert_eq!(
                    matcher.search_work(false, source_len, remaining),
                    remaining + 1
                );
            }
            assert_eq!(matcher.search_work(true, source_len, usize::MAX), 4);
            assert_eq!(
                matcher.search_work(false, source_len, usize::MAX),
                usize::MAX
            );
        }
        let input = JsString::from("A😀🚀!");
        let mut work = Vec::new();
        let found = matcher
            .find_with_work(&input, 2, true, |units| {
                work.push(units);
                Ok::<(), std::convert::Infallible>(())
            })
            .unwrap()
            .unwrap();
        assert_eq!(work, [1]);
        assert_eq!(found.range, 1..5);
        assert_eq!(found.capture_count, 2);
        assert_eq!(found.capture(0), Some(1..5));
        assert_eq!(found.capture(1), Some(1..5));
        assert_eq!(found.capture(2), None);
        assert_eq!(matcher.search_work(true, source.len(), input.len() - 2), 4);
        assert!(matches!(
            matcher.find_with_work(&input, 2, true, |_| Err(29u8)),
            Err(29)
        ));
        assert!(
            matcher
                .find_with_work(&input, input.len() + 1, false, |_| {
                    Ok::<(), std::convert::Infallible>(())
                })
                .unwrap()
                .is_none()
        );
        assert!(matches!(
            RegExpUnicodeClassLiteralMatcher::compile_v_set_then_literal_with_single_character_strings_and_work(
                &source,
                false,
                false,
                |_| Err(43u8),
            ),
            Err(spite_core::RegExpUnicodeSetCompileError::Work(43))
        ));
    }

    #[test]
    fn single_character_v_string_then_flat_class_work_bound_tracks_proved_terms() {
        let source = JsString::from(r"\b[\q{\uD83D\uDE00|x}--x][\u{1f680}]\B");
        let body = RegExpUnicodeClassPairMatcher::compile_v_set_then_flat_with_single_character_strings_and_work(
            &source,
            false,
            false,
            |_| Ok::<(), std::convert::Infallible>(()),
        )
        .unwrap()
        .unwrap();
        let matcher = RegExpMatcher::new(RegExpMatcherBody::UnicodeClassPair(body), 2);
        assert_eq!(matcher.capture_count(), 2);
        assert!(matcher.search_passes(true) > 0);
        assert_eq!(matcher.search_passes(true), matcher.search_passes(false));
        for source_len in [0, 1, source.len(), usize::MAX] {
            for remaining in 0usize..=5 {
                assert_eq!(
                    matcher.search_work(true, source_len, remaining),
                    4.min(remaining + 1)
                );
                assert_eq!(
                    matcher.search_work(false, source_len, remaining),
                    remaining + 1
                );
            }
            assert_eq!(matcher.search_work(true, source_len, usize::MAX), 4);
            assert_eq!(
                matcher.search_work(false, source_len, usize::MAX),
                usize::MAX
            );
        }
        let input = JsString::from("A😀🚀!");
        let mut work = Vec::new();
        let found = matcher
            .find_with_work(&input, 2, true, |units| {
                work.push(units);
                Ok::<(), std::convert::Infallible>(())
            })
            .unwrap()
            .unwrap();
        assert_eq!(work, [1]);
        assert_eq!(found.range, 1..5);
        assert_eq!(found.capture_count, 2);
        assert_eq!(found.capture(0), Some(1..5));
        assert_eq!(found.capture(1), Some(1..5));
        assert_eq!(found.capture(2), None);
        assert_eq!(matcher.search_work(true, source.len(), input.len() - 2), 4);
        assert!(matches!(
            matcher.find_with_work(&input, 2, true, |_| Err(29u8)),
            Err(29)
        ));
        assert!(
            matcher
                .find_with_work(&input, input.len() + 1, false, |_| {
                    Ok::<(), std::convert::Infallible>(())
                })
                .unwrap()
                .is_none()
        );
        assert!(matches!(
            RegExpUnicodeClassPairMatcher::compile_v_set_then_flat_with_single_character_strings_and_work(
                &source,
                false,
                false,
                |_| Err(43u8),
            ),
            Err(spite_core::RegExpUnicodeSetCompileError::Work(43))
        ));
    }

    #[test]
    fn flat_class_then_single_character_v_string_work_bound_tracks_proved_terms() {
        let source = JsString::from(r"\b[\u{1f600}][\q{\uD83D\uDE80|x}--x]\B");
        let body = RegExpUnicodeClassPairMatcher::compile_flat_then_v_set_with_single_character_strings_and_work(
            &source,
            false,
            false,
            |_| Ok::<(), std::convert::Infallible>(()),
        )
        .unwrap()
        .unwrap();
        let matcher = RegExpMatcher::new(RegExpMatcherBody::UnicodeClassPair(body), 2);
        assert_eq!(matcher.capture_count(), 2);
        assert!(matcher.search_passes(true) > 0);
        assert_eq!(matcher.search_passes(true), matcher.search_passes(false));
        for source_len in [0, 1, source.len(), usize::MAX] {
            for remaining in 0usize..=5 {
                assert_eq!(
                    matcher.search_work(true, source_len, remaining),
                    4.min(remaining + 1)
                );
                assert_eq!(
                    matcher.search_work(false, source_len, remaining),
                    remaining + 1
                );
            }
            assert_eq!(matcher.search_work(true, source_len, usize::MAX), 4);
            assert_eq!(
                matcher.search_work(false, source_len, usize::MAX),
                usize::MAX
            );
        }
        let input = JsString::from("A😀🚀!");
        let mut work = Vec::new();
        let found = matcher
            .find_with_work(&input, 2, true, |units| {
                work.push(units);
                Ok::<(), std::convert::Infallible>(())
            })
            .unwrap()
            .unwrap();
        assert_eq!(work, [1]);
        assert_eq!(found.range, 1..5);
        assert_eq!(found.capture_count, 2);
        assert_eq!(found.capture(0), Some(1..5));
        assert_eq!(found.capture(1), Some(1..5));
        assert_eq!(found.capture(2), None);
        assert_eq!(matcher.search_work(true, source.len(), input.len() - 2), 4);
        assert!(matches!(
            matcher.find_with_work(&input, 2, true, |_| Err(29u8)),
            Err(29)
        ));
        assert!(
            matcher
                .find_with_work(&input, input.len() + 1, false, |_| {
                    Ok::<(), std::convert::Infallible>(())
                })
                .unwrap()
                .is_none()
        );
        assert!(matches!(
            RegExpUnicodeClassPairMatcher::compile_flat_then_v_set_with_single_character_strings_and_work(
                &source,
                false,
                false,
                |_| Err(43u8),
            ),
            Err(spite_core::RegExpUnicodeSetCompileError::Work(43))
        ));
    }

    #[test]
    fn single_character_v_string_class_pair_work_bound_tracks_proved_terms() {
        let source = JsString::from(r"\b[\q{\u{1f600}}][\q{\uD83D\uDE80|x}--x]\B");
        let body = RegExpUnicodeClassPairMatcher::compile_v_set_pair_with_single_character_strings_and_work(
            &source,
            false,
            false,
            |_| Ok::<(), std::convert::Infallible>(()),
        )
        .unwrap()
        .unwrap();
        let matcher = RegExpMatcher::new(RegExpMatcherBody::UnicodeClassPair(body), 2);
        assert_eq!(matcher.capture_count(), 2);
        assert!(matcher.search_passes(true) > 0);
        assert_eq!(matcher.search_passes(true), matcher.search_passes(false));
        for source_len in [0, 1, source.len(), usize::MAX] {
            for remaining in 0usize..=5 {
                assert_eq!(
                    matcher.search_work(true, source_len, remaining),
                    4.min(remaining + 1)
                );
                assert_eq!(
                    matcher.search_work(false, source_len, remaining),
                    remaining + 1
                );
            }
            assert_eq!(matcher.search_work(true, source_len, usize::MAX), 4);
            assert_eq!(
                matcher.search_work(false, source_len, usize::MAX),
                usize::MAX
            );
        }
        let input = JsString::from("A😀🚀!");
        let mut work = Vec::new();
        let found = matcher
            .find_with_work(&input, 2, true, |units| {
                work.push(units);
                Ok::<(), std::convert::Infallible>(())
            })
            .unwrap()
            .unwrap();
        assert_eq!(work, [1]);
        assert_eq!(found.range, 1..5);
        assert_eq!(found.capture_count, 2);
        assert_eq!(found.capture(0), Some(1..5));
        assert_eq!(found.capture(1), Some(1..5));
        assert_eq!(found.capture(2), None);
        assert_eq!(matcher.search_work(true, source.len(), input.len() - 2), 4);
        assert!(matches!(
            matcher.find_with_work(&input, 2, true, |_| Err(29u8)),
            Err(29)
        ));
        assert!(
            matcher
                .find_with_work(&input, input.len() + 1, false, |_| {
                    Ok::<(), std::convert::Infallible>(())
                })
                .unwrap()
                .is_none()
        );
        assert!(matches!(
            RegExpUnicodeClassPairMatcher::compile_v_set_pair_with_single_character_strings_and_work(
                &source,
                false,
                false,
                |_| Err(43u8),
            ),
            Err(spite_core::RegExpUnicodeSetCompileError::Work(43))
        ));
    }

    #[test]
    fn checked_class_then_single_character_v_string_work_bound_tracks_proved_terms() {
        let source = JsString::from(r"\b[[\u{1f600}]][\q{\uD83D\uDE80|x}--x]\B");
        let body = RegExpUnicodeClassPairMatcher::compile_checked_then_v_set_with_single_character_strings_and_work(
            &source,
            false,
            false,
            |_| Ok::<(), std::convert::Infallible>(()),
        )
        .unwrap()
        .unwrap();
        let matcher = RegExpMatcher::new(RegExpMatcherBody::UnicodeClassPair(body), 2);
        assert_eq!(matcher.capture_count(), 2);
        for source_len in [0, 1, source.len(), usize::MAX] {
            for remaining in 0usize..=5 {
                assert_eq!(
                    matcher.search_work(true, source_len, remaining),
                    4.min(remaining + 1)
                );
                assert_eq!(
                    matcher.search_work(false, source_len, remaining),
                    remaining + 1
                );
            }
            assert_eq!(matcher.search_work(true, source_len, usize::MAX), 4);
            assert_eq!(
                matcher.search_work(false, source_len, usize::MAX),
                usize::MAX
            );
        }
        let input = JsString::from("A😀🚀!");
        let mut work = Vec::new();
        let found = matcher
            .find_with_work(&input, 2, true, |units| {
                work.push(units);
                Ok::<(), std::convert::Infallible>(())
            })
            .unwrap()
            .unwrap();
        assert_eq!(work, [1]);
        assert_eq!(found.range, 1..5);
        assert_eq!(found.capture_count, 2);
        assert_eq!(found.capture(0), Some(1..5));
        assert_eq!(found.capture(1), Some(1..5));
        assert_eq!(found.capture(2), None);
        assert_eq!(matcher.search_work(true, source.len(), input.len() - 2), 4);
        let mut aborted = Vec::new();
        assert!(matches!(
            matcher.find_with_work(&input, 2, true, |units| {
                aborted.push(units);
                Err(29u8)
            }),
            Err(29)
        ));
        assert_eq!(aborted, [1]);
        let mut outside = Vec::new();
        assert!(
            matcher
                .find_with_work(&input, input.len() + 1, false, |units| {
                    outside.push(units);
                    Ok::<(), std::convert::Infallible>(())
                })
                .unwrap()
                .is_none()
        );
        assert_eq!(outside, [1]);
        let mut compile_aborted = Vec::new();
        assert!(matches!(
            RegExpUnicodeClassPairMatcher::compile_checked_then_v_set_with_single_character_strings_and_work(
                &source,
                false,
                false,
                |units| {
                    compile_aborted.push(units);
                    Err(43u8)
                },
            ),
            Err(spite_core::RegExpUnicodeSetCompileError::Work(43))
        ));
        assert_eq!(compile_aborted, [source.len().saturating_mul(4)]);
    }

    #[test]
    fn wrapped_single_character_v_string_repetition_capture_work_remains_incremental() {
        let source = JsString::from(r"((?:([\q{\uD83D\uDE00|x}--x])))+");
        let body = RegExpUnicodeRepeatedAtomMatcher::compile_wrapped_v_set_with_single_character_strings_and_work(
            &source,
            false,
            false,
            |_| Ok::<(), std::convert::Infallible>(()),
        )
        .unwrap()
        .unwrap();
        assert_eq!(body.capture_count(), 2);
        let matcher = RegExpMatcher::new(RegExpMatcherBody::UnicodeRepeatedAtom(body), 2);
        let input = JsString::from(format!("A{}!", "😀".repeat(128)).as_str());
        assert!(source.len() < input.len());
        assert_eq!(matcher.capture_count(), 4);
        assert_eq!(matcher.search_passes(true), 0);
        assert_eq!(matcher.search_passes(false), 0);
        assert_eq!(
            matcher.search_work(true, source.len(), input.len()),
            input.len()
        );
        assert_eq!(
            matcher.search_work(false, source.len(), input.len()),
            input.len()
        );
        let mut complete = Vec::new();
        let found = matcher
            .find_with_work(&input, 2, true, |amount| {
                complete.push(amount);
                Ok::<(), usize>(())
            })
            .unwrap()
            .unwrap();
        assert!(complete.len() > 2);
        assert_eq!(complete.last(), Some(&2));
        assert_eq!(found.range, 1..257);
        assert_eq!(found.capture_count, 4);
        assert_eq!(found.capture(0), Some(1..257));
        assert_eq!(found.capture(1), Some(1..257));
        assert_eq!(found.capture(2), Some(255..257));
        assert_eq!(found.capture(3), Some(255..257));
        assert_eq!(found.capture(4), None);
        // Fail at the final capture-extraction charge, after selecting the run
        // but before a result can expose captures or BuiltinExec writes effects.
        let mut seen = Vec::new();
        let cancelled = matcher.find_with_work(&input, 2, true, |amount| {
            seen.push(amount);
            if seen.len() == complete.len() {
                Err(seen.len())
            } else {
                Ok(())
            }
        });
        assert!(matches!(cancelled, Err(at) if at == complete.len()));
        assert_eq!(seen, complete);
        let source = JsString::from(r"((?:([\q{a}--a]))){0}");
        let body = RegExpUnicodeRepeatedAtomMatcher::compile_wrapped_v_set_with_single_character_strings_and_work(
            &source,
            false,
            false,
            |_| Ok::<(), std::convert::Infallible>(()),
        )
        .unwrap()
        .unwrap();
        let matcher = RegExpMatcher::new(RegExpMatcherBody::UnicodeRepeatedAtom(body), 2);
        let found = matcher
            .find_with_work(&JsString::from("a"), 0, true, |_| Ok::<(), usize>(()))
            .unwrap()
            .unwrap();
        assert_eq!(found.range, 0..0);
        assert_eq!(found.capture_count, 4);
        assert_eq!(found.capture(0), Some(0..0));
        assert_eq!(found.capture(1), Some(0..0));
        assert_eq!(found.capture(2), None);
        assert_eq!(found.capture(3), None);
    }

    #[test]
    fn raw_empty_single_character_v_string_repetition_capture_work_remains_incremental() {
        for (pattern, input, start, wanted, inner, capture_charge) in [
            (
                r"((?:([\q{\uD83D\uDE00}]|)))+",
                JsString::from("😀".repeat(128).as_str()),
                1usize,
                0..256,
                Some(254..256),
                Some(2),
            ),
            (
                r"((?:([\q{a}--a]|)))+",
                JsString::from("z"),
                0,
                0..0,
                Some(0..0),
                Some(1),
            ),
            (
                r"((?:([\q{a}]|)))*?",
                JsString::from("a"),
                0,
                0..0,
                None,
                None,
            ),
        ] {
            let source = JsString::from(pattern);
            let body = RegExpUnicodeRepeatedAtomMatcher::compile_nullable_v_set_with_single_character_strings_and_work(
                &source,
                false,
                false,
                |_| Ok::<(), std::convert::Infallible>(()),
            )
            .unwrap()
            .unwrap();
            assert_eq!(body.capture_count(), 2);
            let matcher = RegExpMatcher::new(RegExpMatcherBody::UnicodeRepeatedAtom(body), 2);
            assert_eq!(matcher.capture_count(), 4);
            assert_eq!(matcher.search_passes(true), 0);
            assert_eq!(matcher.search_passes(false), 0);
            for source_len in [0, source.len(), usize::MAX] {
                for sticky in [false, true] {
                    assert_eq!(
                        matcher.search_work(sticky, source_len, input.len()),
                        input.len()
                    );
                    assert_eq!(
                        matcher.search_work(sticky, source_len, usize::MAX),
                        usize::MAX
                    );
                }
            }
            let mut complete = Vec::new();
            let found = matcher
                .find_with_work(&input, start, true, |amount| {
                    complete.push(amount);
                    Ok::<(), usize>(())
                })
                .unwrap()
                .unwrap();
            assert!(!complete.is_empty());
            assert_eq!(found.range, wanted);
            assert_eq!(found.capture_count, 4);
            assert_eq!(found.capture(0), Some(wanted.clone()));
            assert_eq!(found.capture(1), Some(wanted));
            assert_eq!(found.capture(2), inner);
            assert_eq!(found.capture(3), inner);
            assert_eq!(found.capture(4), None);
            if let Some(charge) = capture_charge {
                // Mandatory empty capture extraction charges one unit; a
                // complete final supplementary character charges two units.
                assert_eq!(complete.last(), Some(&charge));
                let mut seen = Vec::new();
                let result = matcher.find_with_work(&input, start, true, |amount| {
                    seen.push(amount);
                    if seen.len() == complete.len() {
                        Err(seen.len())
                    } else {
                        Ok(())
                    }
                });
                assert!(matches!(result, Err(at) if at == complete.len()));
                assert_eq!(seen, complete);
            }
        }
    }

    #[test]
    fn unicode_nullable_class_captures_keep_enclosing_and_iteration_slots_separate() {
        let body = spite_core::RegExpUnicodeNullableClassCaptureMatcher::compile_with_work(
            &JsString::from(r"(([\q{a|b}])|){3}"),
            false,
            false,
            |_| Ok::<(), ()>(()),
        )
        .unwrap()
        .unwrap();
        let matcher = RegExpMatcher::new(RegExpMatcherBody::UnicodeNullableClassCaptures(body), 2);
        assert_eq!(matcher.capture_count(), 4);
        let input = JsString::from("ab");
        let found = matcher
            .find_with_work(&input, 0, true, |_| Ok::<(), ()>(()))
            .unwrap()
            .unwrap();
        assert_eq!(found.range, 0..2);
        assert_eq!(found.capture(0), Some(0..2));
        assert_eq!(found.capture(1), Some(0..2));
        assert_eq!(found.capture(2), Some(2..2));
        assert_eq!(found.capture(3), None);
        assert_eq!(found.capture(4), None);
    }

    #[test]
    fn unicode_nullable_class_captures_hide_the_synthetic_empty_projection_marker() {
        let body = spite_core::RegExpUnicodeNullableClassCaptureMatcher::compile_with_work(
            &JsString::from(r"(?:|(([\q{a|b}])))+?"),
            false,
            false,
            |_| Ok::<(), ()>(()),
        )
        .unwrap()
        .unwrap();
        let matcher = RegExpMatcher::new(RegExpMatcherBody::UnicodeNullableClassCaptures(body), 0);
        assert_eq!(matcher.capture_count(), 2);
        let input = JsString::from("a");
        let found = matcher
            .find_with_work(&input, 0, true, |_| Ok::<(), ()>(()))
            .unwrap()
            .unwrap();
        assert_eq!(found.range, 0..0);
        assert_eq!(found.capture(0), None);
        assert_eq!(found.capture(1), None);
    }

    #[test]
    fn unicode_nullable_empty_only_captures_project_required_optional_and_zero_iterations() {
        for (source, whole, empty) in [
            (r"(?:[\q{a|b}]|(())){3}", 0..2, Some(2..2)),
            (r"(?:(())|[\q{a|b}]){2,3}", 0..1, None),
            (r"(?:[\q{a|b}]|(())){0}", 0..0, None),
        ] {
            let body = RegExpUnicodeNullableClassCaptureMatcher::compile_with_empty_branch_captures_and_work(
                &JsString::from(source), false, false, |_| Ok::<(), ()>(()),
            ).unwrap().unwrap();
            let matcher =
                RegExpMatcher::new(RegExpMatcherBody::UnicodeNullableClassCaptures(body), 2);
            assert_eq!(matcher.capture_count(), 4);
            let input = JsString::from("ab");
            let found = matcher
                .find_with_work(&input, 0, true, |_| Ok::<(), ()>(()))
                .unwrap()
                .unwrap();
            assert_eq!(found.range, whole);
            assert_eq!(found.capture(0), Some(whole.clone()));
            assert_eq!(found.capture(1), Some(whole));
            assert_eq!(found.capture(2), empty);
            assert_eq!(found.capture(3), empty);
            assert_eq!(found.capture(4), None);
        }
    }

    #[test]
    fn unicode_nullable_empty_capture_effects_keep_required_parents_and_undefined_children() {
        for (source, whole, parent) in [
            (r"(?:((())?)|[\q{a|b}]){1}", 0..0, Some(0..0)),
            (r"(?:[\q{a|b}]|((())?)+?){3}", 0..2, Some(2..2)),
            (r"(?:[\q{a|b}]|((())?)+?){0}", 0..0, None),
        ] {
            let body = RegExpUnicodeNullableClassCaptureMatcher::compile_with_empty_capture_effects_and_work(
                &JsString::from(source), false, false, |_| Ok::<(), ()>(()),
            ).unwrap().unwrap();
            let matcher =
                RegExpMatcher::new(RegExpMatcherBody::UnicodeNullableClassCaptures(body), 2);
            assert_eq!(matcher.capture_count(), 5);
            let input = JsString::from("ab");
            let found = matcher
                .find_with_work(&input, 0, true, |_| Ok::<(), ()>(()))
                .unwrap()
                .unwrap();
            assert_eq!(found.range, whole);
            assert_eq!(found.capture(0), Some(whole.clone()));
            assert_eq!(found.capture(1), Some(whole));
            assert_eq!(found.capture(2), parent);
            assert_eq!(found.capture(3), None);
            assert_eq!(found.capture(4), None);
            assert_eq!(found.capture(5), None);
        }
    }

    #[test]
    fn unicode_nullable_boundary_captures_use_last_iteration_utf16_points() {
        for (count, expected) in [
            (
                "+",
                [
                    Some(2..2),
                    Some(2..2),
                    Some(2..4),
                    Some(4..4),
                    None,
                    Some(4..4),
                ],
            ),
            (
                "{3}",
                [Some(4..4), None, None, None, Some(4..4), Some(4..4)],
            ),
        ] {
            let source = format!(r"(?:()(?:()([\q{{😀}}])()|())()){count}");
            let body = RegExpUnicodeNullableClassCaptureMatcher::compile_with_boundary_empty_captures_and_work(
                &JsString::from(source.as_str()), false, false, |_| Ok::<(), ()>(()),
            ).unwrap().unwrap();
            let matcher =
                RegExpMatcher::new(RegExpMatcherBody::UnicodeNullableClassCaptures(body), 1);
            assert_eq!(matcher.capture_count(), 7);
            let input = JsString::from("😀😀");
            let found = matcher
                .find_with_work(&input, 0, true, |_| Ok::<(), ()>(()))
                .unwrap()
                .unwrap();
            assert_eq!(found.range, 0..4);
            assert_eq!(found.capture(0), Some(0..4));
            for (index, range) in expected.into_iter().enumerate() {
                assert_eq!(found.capture(index + 1), range);
            }
            assert_eq!(found.capture(7), None);
        }
    }
}
