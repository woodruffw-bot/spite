//! Native RegExp original Pattern and flag slots (22.2).

use super::{Error, Objects};
use spite_core::{JsString, RegExpDisjunctionMatcher, RegExpLiteralMatcher};
use spite_heap::Handle;
use std::ops::Range;

#[derive(Clone, Debug)]
pub(crate) enum RegExpMatcher {
    Literal(RegExpLiteralMatcher),
    Disjunction(RegExpDisjunctionMatcher),
}

impl RegExpMatcher {
    pub fn find(&self, input: &JsString, start: usize, sticky: bool) -> Option<Range<usize>> {
        match self {
            Self::Literal(matcher) => matcher.find(input, start, sticky),
            Self::Disjunction(matcher) => matcher.find(input, start, sticky),
        }
    }
    pub fn capture_ranges(&self) -> &[Range<usize>] {
        match self {
            Self::Literal(matcher) => matcher.capture_ranges(),
            Self::Disjunction(_) => &[],
        }
    }
    pub fn search_passes(&self) -> usize {
        match self {
            Self::Literal(_) => 1,
            Self::Disjunction(matcher) => matcher.alternative_count(),
        }
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
            data.matcher.as_ref().unwrap().find(&source, 0, true),
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
