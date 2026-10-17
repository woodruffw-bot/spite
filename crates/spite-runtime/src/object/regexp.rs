//! Native RegExp original Pattern and flag slots (22.2).

use super::{Error, Objects};
use spite_core::{
    JsString, RegExpAnchoredMatcher, RegExpBackreferenceMatcher, RegExpCharacterMatcher,
    RegExpDisjunctionMatcher, RegExpLiteralMatcher, RegExpPrefixedMatcher,
    RegExpQuantifiedContinuationMatcher, RegExpQuantifiedMatcher, RegExpRepeatedCaptureMatcher,
    RegExpRepeatedContinuationMatcher, RegExpRepeatedLiteralMatcher, RegExpRepeatedPrefixedMatcher,
    RegExpRepeatedSequenceMatcher, RegExpSequenceMatcher, RegExpUnicodeCharacterMatcher,
    RegExpUnicodeClassLiteralClassMatcher, RegExpUnicodeClassLiteralMatcher,
    RegExpUnicodeClassMatcher, RegExpUnicodeClassPairLiteralMatcher, RegExpUnicodeClassPairMatcher,
    RegExpUnicodeClassTripleMatcher, RegExpUnicodeClassTripleWordMatcher,
    RegExpUnicodeFoldedLiteralMatcher, RegExpUnicodeLiteralClassPairMatcher,
    RegExpUnicodeLiteralClassPairWordMatcher, RegExpUnicodePropertyDotMatcher,
    RegExpUnicodeRepeatedAtomMatcher,
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
    UnicodeClassLiteral(RegExpUnicodeClassLiteralMatcher),
    UnicodeClassPair(RegExpUnicodeClassPairMatcher),
    UnicodeLiteralClassPair(RegExpUnicodeLiteralClassPairMatcher),
    UnicodeClassPairLiteral(RegExpUnicodeClassPairLiteralMatcher),
    UnicodeClassLiteralClass(RegExpUnicodeClassLiteralClassMatcher),
    UnicodeClassTriple(RegExpUnicodeClassTripleMatcher),
    UnicodeClassTripleWord(RegExpUnicodeClassTripleWordMatcher),
    UnicodeLiteralClassPairWord(RegExpUnicodeLiteralClassPairWordMatcher),
    UnicodeFoldedLiteral(RegExpUnicodeFoldedLiteralMatcher),
    UnicodeRepeatedAtom(RegExpUnicodeRepeatedAtomMatcher),
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
            | RegExpMatcherBody::UnicodeClassLiteral(_)
            | RegExpMatcherBody::UnicodeClassPair(_)
            | RegExpMatcherBody::UnicodeLiteralClassPair(_)
            | RegExpMatcherBody::UnicodeClassPairLiteral(_)
            | RegExpMatcherBody::UnicodeClassLiteralClass(_)
            | RegExpMatcherBody::UnicodeClassTriple(_)
            | RegExpMatcherBody::UnicodeClassTripleWord(_)
            | RegExpMatcherBody::UnicodeLiteralClassPairWord(_) => 0,
            RegExpMatcherBody::UnicodeRepeatedAtom(m) => m.capture_count(),
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
            | RegExpMatcherBody::UnicodePropertyDot(_)
            | RegExpMatcherBody::UnicodeClassLiteral(_)
            | RegExpMatcherBody::UnicodeClassPair(_)
            | RegExpMatcherBody::UnicodeLiteralClassPair(_)
            | RegExpMatcherBody::UnicodeClassPairLiteral(_)
            | RegExpMatcherBody::UnicodeClassLiteralClass(_)
            | RegExpMatcherBody::UnicodeClassTriple(_)
            | RegExpMatcherBody::UnicodeClassTripleWord(_)
            | RegExpMatcherBody::UnicodeLiteralClassPairWord(_) => {
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
            | RegExpMatcherBody::UnicodeRepeatedAtom(_) => 0,
            RegExpMatcherBody::Literal(_)
            | RegExpMatcherBody::UnicodeEmpty(_)
            | RegExpMatcherBody::UnicodeCodePoints(_) => 1,
            RegExpMatcherBody::Anchored(matcher) => matcher.search_passes(sticky),
            RegExpMatcherBody::Character(_) => 1,
            RegExpMatcherBody::UnicodeCharacter(matcher) => matcher.search_passes(),
            RegExpMatcherBody::UnicodeClass(matcher) => matcher.search_passes(),
            RegExpMatcherBody::UnicodeClassLiteral(matcher) => matcher.search_passes(),
            RegExpMatcherBody::UnicodeClassPair(matcher) => matcher.search_passes(),
            RegExpMatcherBody::UnicodeLiteralClassPair(matcher) => matcher.search_passes(),
            RegExpMatcherBody::UnicodeClassPairLiteral(matcher) => matcher.search_passes(),
            RegExpMatcherBody::UnicodeClassLiteralClass(matcher) => matcher.search_passes(),
            RegExpMatcherBody::UnicodeClassTriple(matcher) => matcher.search_passes(),
            RegExpMatcherBody::UnicodeClassTripleWord(matcher) => matcher.search_passes(),
            RegExpMatcherBody::UnicodeLiteralClassPairWord(matcher) => matcher.search_passes(),
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
        if matches!(
            self.body,
            RegExpMatcherBody::UnicodeLiteralClassPair(_)
                | RegExpMatcherBody::UnicodeClassPairLiteral(_)
                | RegExpMatcherBody::UnicodeClassLiteralClass(_)
                | RegExpMatcherBody::UnicodeClassTriple(_)
                | RegExpMatcherBody::UnicodeClassTripleWord(_)
                | RegExpMatcherBody::UnicodeLiteralClassPairWord(_)
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
            | RegExpMatcherBody::UnicodeClassLiteral(_)
            | RegExpMatcherBody::UnicodeClassPair(_)
            | RegExpMatcherBody::UnicodeLiteralClassPair(_)
            | RegExpMatcherBody::UnicodeClassPairLiteral(_)
            | RegExpMatcherBody::UnicodeClassLiteralClass(_)
            | RegExpMatcherBody::UnicodeClassTriple(_)
            | RegExpMatcherBody::UnicodeClassTripleWord(_)
            | RegExpMatcherBody::UnicodeLiteralClassPairWord(_) => {
                return None;
            }
            RegExpMatcherBody::UnicodeRepeatedAtom(_) => return self.atom_capture.clone(),
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
}
