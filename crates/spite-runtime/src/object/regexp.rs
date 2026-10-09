//! Native RegExp original Pattern and flag slots (22.2).

use super::{Error, Objects};
use spite_core::{
    JsString, RegExpAnchoredMatcher, RegExpBackreferenceMatcher, RegExpCharacterMatcher,
    RegExpDisjunctionMatcher, RegExpLiteralMatcher, RegExpPrefixedMatcher,
    RegExpQuantifiedContinuationMatcher, RegExpQuantifiedMatcher, RegExpRepeatedCaptureMatcher,
    RegExpRepeatedContinuationMatcher, RegExpRepeatedLiteralMatcher, RegExpRepeatedPrefixedMatcher,
    RegExpRepeatedSequenceMatcher, RegExpSequenceMatcher,
};
use spite_heap::Handle;
use std::{ops::Range, sync::Arc};

#[derive(Clone, Debug)]
pub(crate) enum RegExpMatcherBody {
    Backreferences(RegExpBackreferenceMatcher),
    Literal(RegExpLiteralMatcher),
    UnicodeEmpty(RegExpLiteralMatcher),
    Anchored(RegExpAnchoredMatcher),
    Character(RegExpCharacterMatcher),
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
            RegExpMatcherBody::Literal(m) | RegExpMatcherBody::UnicodeEmpty(m) => {
                m.capture_ranges().len()
            }
            RegExpMatcherBody::Sequence(m) => m.capture_ranges().len(),
            RegExpMatcherBody::Character(_) => 0,
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
            RegExpMatcherBody::Backreferences(_) => unreachable!("fallible reference dispatch"),
            RegExpMatcherBody::Literal(matcher) => (matcher.find(input, start, sticky)?, 0),
            RegExpMatcherBody::UnicodeEmpty(matcher) => {
                (matcher.find_unicode_empty(input, start)?, 0)
            }
            RegExpMatcherBody::Anchored(matcher) => (matcher.find(input, start, sticky)?, 0),
            RegExpMatcherBody::Character(matcher) => (matcher.find(input, start, sticky)?, 0),
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
        })
    }
    pub fn capture_count(&self) -> usize {
        self.capture_count
    }
    pub fn search_passes(&self, sticky: bool) -> usize {
        match &self.body {
            // References charge actual operations through find_with_work.
            RegExpMatcherBody::Backreferences(_) => 0,
            RegExpMatcherBody::Literal(_) | RegExpMatcherBody::UnicodeEmpty(_) => 1,
            RegExpMatcherBody::Anchored(matcher) => matcher.search_passes(sticky),
            RegExpMatcherBody::Character(_) => 1,
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
        let full_suffix = match &self.body {
            RegExpMatcherBody::Quantified(_)
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
            RegExpMatcherBody::Backreferences(_) => unreachable!("dynamic reference captures"),
            RegExpMatcherBody::Literal(matcher) | RegExpMatcherBody::UnicodeEmpty(matcher) => {
                matcher.capture_ranges()
            }
            RegExpMatcherBody::Sequence(matcher) => matcher.capture_ranges(),
            RegExpMatcherBody::Character(_) => return None,
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
