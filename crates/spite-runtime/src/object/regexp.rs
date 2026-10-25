//! Native RegExp slots and one shared regress program.

use super::{Error, Objects, store::Budget};
use spite_core::{JsString, RegExpProgram, RegExpProgramResult};
use spite_heap::Handle;
use std::{ops::Range, sync::Arc};

#[derive(Clone, Debug)]
pub(crate) struct RegExpMatcher {
    program: RegExpProgram,
}

impl RegExpMatcher {
    pub fn new(program: RegExpProgram) -> Self {
        Self { program }
    }
    pub fn find_with_budget(
        &self,
        input: &JsString,
        start: usize,
        sticky: bool,
        budget: &mut Budget,
    ) -> Result<Option<RegExpMatch>, Error> {
        let (result, used) = self
            .program
            .find(input, start, sticky, budget.remaining_work());
        budget.charge(used)?;
        match result {
            RegExpProgramResult::Match(found) => Ok(Some(RegExpMatch {
                range: found.range,
                capture_count: found.captures.len(),
                captures: found.captures,
            })),
            RegExpProgramResult::NoMatch => Ok(None),
            RegExpProgramResult::WorkLimit => Err(Error::WorkLimit),
        }
    }
}

#[derive(Debug)]
pub(crate) struct RegExpMatch {
    pub range: Range<usize>,
    pub capture_count: usize,
    captures: Vec<Option<Range<usize>>>,
}
impl RegExpMatch {
    pub fn capture(&self, index: usize) -> Option<Range<usize>> {
        self.captures.get(index).cloned().flatten()
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
