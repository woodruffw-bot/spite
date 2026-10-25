//! Shared UTF-16 adapter for the regress ECMAScript matching engine.

use crate::JsString;
use std::{ops::Range, sync::Arc};

/// An immutable compiled expression with inexpensive shared Rust clones.
///
/// Callers validate the original Pattern and flags against the language baseline
/// before compilation. Backend rejection is an implementation failure, separate
/// from the original Pattern's syntax errors.
#[derive(Clone, Debug)]
pub struct RegExpProgram {
    compiled: Arc<regress::Regex>,
    unicode: bool,
}

/// A successful match with UTF-16 offsets into the complete original input.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RegExpProgramMatch {
    /// The complete match, including a defined empty match.
    pub range: Range<usize>,
    /// Source-order captures; undefined captures remain distinct from empty ones.
    pub captures: Vec<Option<Range<usize>>>,
}

/// The outcome of execution, including an explicit host work interruption.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RegExpProgramResult {
    /// A successful match.
    Match(RegExpProgramMatch),
    /// The search completed without a match.
    NoMatch,
    /// The optional host work allowance was exhausted; the answer is unknown.
    WorkLimit,
}

impl RegExpProgram {
    /// Compiles a fully validated original Pattern without rewriting its bindings.
    pub fn compile_validated(source: &JsString, flags: &JsString) -> Result<Self, String> {
        let mut flags = regress::Flags::new(flags.code_units().iter().copied().map(u32::from));
        let unicode = flags.unicode || flags.unicode_sets;
        flags.unicode = unicode;
        let points = char::decode_utf16(source.code_units().iter().copied()).map(|point| {
            point.map_or_else(|error| u32::from(error.unpaired_surrogate()), u32::from)
        });
        let compiled = if unicode {
            regress::Regex::from_unicode(points, flags)
        } else {
            regress::Regex::from_unicode(source.code_units().iter().copied().map(u32::from), flags)
        }
        .map_err(|error| error.text)?;
        Ok(Self {
            compiled: Arc::new(compiled),
            unicode,
        })
    }

    /// Returns the complete lexical capture count.
    pub fn capture_count(&self) -> usize {
        self.compiled.capture_count()
    }

    /// Executes against original input with optional work accounting.
    ///
    /// Sticky searches probe exactly the supplied Unicode boundary. Unicode
    /// starts inside a valid pair rewind to its leading boundary, following the
    /// approved edition-17 RegExpBuiltinExec resolution. No limit is imposed
    /// when `work_limit` is `None`. The second result is consumed host work.
    pub fn find(
        &self,
        input: &JsString,
        mut start: usize,
        sticky: bool,
        work_limit: Option<usize>,
    ) -> (RegExpProgramResult, usize) {
        let units = input.code_units();
        if start > units.len() {
            return (RegExpProgramResult::NoMatch, 0);
        }
        if self.unicode
            && start > 0
            && units
                .get(start)
                .is_some_and(|unit| (0xdc00..=0xdfff).contains(unit))
            && (0xd800..=0xdbff).contains(&units[start - 1])
        {
            start -= 1;
        }
        let (result, used) = self.compiled.find_utf16_with_work_limit(
            units,
            start,
            self.unicode,
            sticky,
            work_limit.map(|limit| limit as u64),
        );
        let result = match result {
            regress::BudgetedMatch::Match(found) => {
                RegExpProgramResult::Match(RegExpProgramMatch {
                    range: found.range,
                    captures: found.captures,
                })
            }
            regress::BudgetedMatch::NoMatch => RegExpProgramResult::NoMatch,
            regress::BudgetedMatch::BudgetExhausted => RegExpProgramResult::WorkLimit,
        };
        (result, used as usize)
    }
}

#[cfg(test)]
#[path = "regexp_program_tests.rs"]
mod tests;
