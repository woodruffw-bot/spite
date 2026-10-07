//! Native RegExp original Pattern and flag slots (22.2).

use super::{Error, Objects};
use spite_core::{
    JsString, RegExpAnchoredMatcher, RegExpCharacterMatcher, RegExpDisjunctionMatcher,
    RegExpLiteralMatcher, RegExpQuantifiedContinuationMatcher, RegExpQuantifiedMatcher,
    RegExpSequenceMatcher,
};
use spite_heap::Handle;
use std::ops::Range;

#[derive(Clone, Debug)]
pub(crate) enum RegExpMatcher {
    Literal(RegExpLiteralMatcher),
    Anchored(RegExpAnchoredMatcher),
    Character(RegExpCharacterMatcher),
    Sequence(RegExpSequenceMatcher),
    Disjunction(RegExpDisjunctionMatcher),
    Quantified(RegExpQuantifiedMatcher),
    QuantifiedContinuation(RegExpQuantifiedContinuationMatcher),
}

impl RegExpMatcher {
    pub fn find<'a>(
        &'a self,
        input: &JsString,
        start: usize,
        sticky: bool,
    ) -> Option<RegExpMatch<'a>> {
        let (range, branch) = match self {
            Self::Literal(matcher) => (matcher.find(input, start, sticky)?, 0),
            Self::Anchored(matcher) => (matcher.find(input, start, sticky)?, 0),
            Self::Character(matcher) => (matcher.find(input, start, sticky)?, 0),
            Self::Quantified(matcher) => (matcher.find(input, start, sticky)?, 0),
            Self::QuantifiedContinuation(matcher) => (matcher.find(input, start, sticky)?, 0),
            Self::Sequence(matcher) => (matcher.find(input, start, sticky)?, 0),
            Self::Disjunction(matcher) => {
                let (branch, range) = matcher.find_branch(input, start, sticky)?;
                (range, branch)
            }
        };
        Some(RegExpMatch {
            range,
            matcher: self,
            branch,
            capture_count: self.capture_count(),
        })
    }
    pub fn capture_count(&self) -> usize {
        match self {
            Self::Literal(matcher) => matcher.capture_ranges().len(),
            Self::Anchored(matcher) => matcher.capture_count(),
            Self::Character(_) => 0,
            Self::Quantified(matcher) => matcher.capture_count(),
            Self::QuantifiedContinuation(matcher) => matcher.capture_count(),
            Self::Sequence(matcher) => matcher.capture_ranges().len(),
            Self::Disjunction(matcher) => matcher.capture_count(),
        }
    }
    pub fn search_passes(&self, sticky: bool) -> usize {
        match self {
            Self::Literal(_) => 1,
            Self::Anchored(matcher) => matcher.search_passes(sticky),
            Self::Character(_) => 1,
            Self::Quantified(_) => 1,
            Self::QuantifiedContinuation(_) => 2,
            Self::Sequence(matcher) => {
                if sticky {
                    1
                } else {
                    matcher.atom_count().max(1)
                }
            }
            Self::Disjunction(matcher) => matcher.search_passes(sticky),
        }
    }

    /// Sticky repeated atoms can consume more input than their source length.
    pub fn search_work(&self, sticky: bool, source_len: usize, remaining: usize) -> usize {
        let full_suffix = match self {
            Self::Quantified(_) | Self::QuantifiedContinuation(_) => true,
            Self::Anchored(matcher) => matcher.requires_full_suffix(),
            Self::Disjunction(matcher) => matcher.requires_full_suffix(),
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
}

impl RegExpMatch<'_> {
    /// Absolute UTF-16 range, or None for a group in an unselected branch.
    pub fn capture(&self, index: usize) -> Option<Range<usize>> {
        let fixed = match self.matcher {
            RegExpMatcher::Literal(matcher) => matcher.capture_ranges(),
            RegExpMatcher::Sequence(matcher) => matcher.capture_ranges(),
            RegExpMatcher::Character(_) => return None,
            RegExpMatcher::Anchored(matcher) => return matcher.capture_range(index, &self.range),
            RegExpMatcher::Quantified(matcher) => return matcher.capture_range(index, &self.range),
            RegExpMatcher::QuantifiedContinuation(matcher) => {
                return matcher.capture_range(index, &self.range);
            }
            RegExpMatcher::Disjunction(matcher) => {
                return matcher.capture_range(self.branch, index, &self.range);
            }
        };
        let relative = fixed.get(index)?;
        Some(self.range.start + relative.start..self.range.start + relative.end)
    }
}

#[derive(Clone, Debug)]
pub(crate) struct RegExpData {
    pub source: JsString,
    pub flags: JsString,
    pub matcher: Option<RegExpMatcher>,
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
                matches!(objects.initialize_regexp(object, RegExpData { source: JsString::from("a"), flags: JsString::from("g"),matcher:None }), Err(Error::Heap(error)) if error == expected)
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
                        .map(RegExpMatcher::Literal),
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
                .find(&source, 0, true)
                .map(|found| found.range),
            Some(0..source.len())
        );
        assert!(matches!(
            objects.initialize_regexp(
                &object,
                RegExpData {
                    source: JsString::default(),
                    flags: JsString::default(),
                    matcher: None
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
