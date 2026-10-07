//! Ordinary character concatenations, assertions and references (22.2.2.4, 22.2.2.9).

use crate::regexp_assertion::Assertions;
use crate::regexp_character::PreparedCharacter;
use crate::regexp_literal::{character_escape, is_syntax};
use crate::{JsString, RegExpCharacterMatcher, regexp_canonicalize_character};
use std::{collections::HashMap, ops::Range, sync::Arc};

/// One validated named-reference escape in a name-stripped private source.
#[derive(Clone, Debug)]
pub struct RegExpBackreferenceNamedReference {
    /// Complete absolute UTF-16 escape interval, including the closing `>`.
    pub escape: Range<usize>,
    /// Index into the shared name binding inventory.
    pub group: usize,
}

/// Shared original capture slots and ordered named-reference escape intervals.
///
/// Capture slots are zero-based. Each capture belongs to at most one name;
/// duplicate-name slots must be mutually exclusive in the validated Pattern.
#[derive(Clone, Copy, Debug, Default)]
pub struct RegExpBackreferenceNamedBindings<'a> {
    /// One shared slot slice per decoded name.
    pub groups: &'a [&'a [usize]],
    /// Ordered, disjoint escapes in the private source.
    pub references: &'a [RegExpBackreferenceNamedReference],
}

/// A flat, immutable ordinary character and numbered-backreference program.
///
/// The complete Pattern must already be validated without `u` or `v`. Capturing
/// and noncapturing groups are accepted, including empty, nested and forward
/// references, ordinary character sets, dot and word/input/line assertions.
/// Top-level alternatives and one inner choice per top-level branch are accepted.
/// Additional inner choices and quantifiers return `None`. At least one numbered
/// or registered named reference is required; plain literals
/// retain their existing linear-search matcher.
/// References are never expanded into source or compiled literal strings.
#[derive(Clone, Debug)]
pub struct RegExpBackreferenceMatcher(Arc<Program>);

#[derive(Debug)]
struct Program {
    instructions: Vec<Instruction>,
    capture_count: usize,
    ignore_case: bool,
    multiline: bool,
    branches: Vec<Branch>,
    capture_names: Vec<Option<usize>>,
    named_count: usize,
}

#[derive(Debug)]
enum Instruction {
    Character(u16),
    Set(RegExpCharacterMatcher),
    Assert(Assertions),
    Open(usize),
    Close(usize),
    Reference(usize),
    NamedReference(usize),
}

enum PreparedInstruction {
    Ready(Instruction),
    Set {
        plan: Box<PreparedCharacter>,
        source: Range<usize>,
    },
}

#[derive(Debug)]
struct Branch {
    leading: Range<usize>,
    alternatives: Vec<Range<usize>>,
    trailing: Range<usize>,
    captures: Range<usize>,
}

struct GroupFrame {
    capture: Option<usize>,
    body_start: usize,
    alternative_start: usize,
    alternatives: Vec<Range<usize>>,
}

struct Choice {
    start: usize,
    end: usize,
    alternatives: Vec<Range<usize>>,
}

fn branch_plan(start: usize, end: usize, captures: Range<usize>, choice: Option<Choice>) -> Branch {
    if let Some(choice) = choice {
        Branch {
            leading: start..choice.start,
            alternatives: choice.alternatives,
            trailing: choice.end..end,
            captures,
        }
    } else {
        Branch {
            leading: start..start,
            alternatives: std::iter::once(start..end).collect(),
            trailing: end..end,
            captures,
        }
    }
}

struct PreparedProgram {
    instructions: Vec<PreparedInstruction>,
    capture_count: usize,
    branches: Vec<Branch>,
}

/// Absolute UTF-16 endpoints from one successful execution.
#[derive(Debug)]
pub struct RegExpBackreferenceMatch {
    /// Whole-match endpoints.
    pub range: Range<usize>,
    /// Opening-parenthesis order, excluding the whole match.
    pub captures: Box<[Option<Range<usize>>]>,
}

impl RegExpBackreferenceMatcher {
    /// Compiles the complete supported subset without charging host work.
    pub fn compile(source: &JsString, ignore_case: bool) -> Option<Self> {
        Self::compile_with_work(source, ignore_case, |_| {
            Ok::<_, std::convert::Infallible>(())
        })
        .unwrap_or_else(|never| match never {})
    }

    /// Checks the entire syntax before charging optional construction work.
    pub fn compile_with_work<E>(
        source: &JsString,
        ignore_case: bool,
        charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, E> {
        Self::compile_with_flags_and_work(source, ignore_case, false, charge)
    }

    /// Compiles ordinary sets and dot with explicit DotAll and optional work.
    /// Identical set source shares one immutable predicate within this program.
    pub fn compile_with_flags_and_work<E>(
        source: &JsString,
        ignore_case: bool,
        dot_all: bool,
        charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, E> {
        Self::compile_with_assertions_and_work(source, ignore_case, false, dot_all, charge)
    }

    /// Compiles ordinary word/input/line assertions with explicit Multiline.
    /// Assertions use the complete input at the current consuming position.
    pub fn compile_with_assertions_and_work<E>(
        source: &JsString,
        ignore_case: bool,
        multiline: bool,
        dot_all: bool,
        charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, E> {
        Self::compile_with_named_bindings_and_work(
            source,
            ignore_case,
            multiline,
            dot_all,
            RegExpBackreferenceNamedBindings::default(),
            charge,
        )
    }

    /// Compiles validated named references using shared source-order bindings.
    /// The private source has capture-name specifiers removed and original named
    /// escapes retained. Invalid binding intervals or slot inventories return `None`
    /// before host work is charged. Named targets are never expanded into source.
    pub fn compile_with_named_bindings_and_work<E>(
        source: &JsString,
        ignore_case: bool,
        multiline: bool,
        dot_all: bool,
        bindings: RegExpBackreferenceNamedBindings<'_>,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, E> {
        if bindings.references.is_empty()
            && !source
                .code_units()
                .windows(2)
                .any(|units| units[0] == 0x5c && (0x31..=0x39).contains(&units[1]))
        {
            return Ok(None);
        }
        let Some(plan) = prepare(source.code_units(), ignore_case, dot_all, bindings) else {
            return Ok(None);
        };
        let PreparedProgram {
            instructions: prepared,
            capture_count,
            branches,
        } = plan;
        let mut capture_names = if bindings.groups.is_empty() {
            Vec::new()
        } else {
            vec![None; capture_count]
        };
        for (group, slots) in bindings.groups.iter().enumerate() {
            if slots.is_empty() {
                return Ok(None);
            }
            for &slot in *slots {
                let Some(name) = capture_names.get_mut(slot) else {
                    return Ok(None);
                };
                if name.replace(group).is_some() {
                    return Ok(None);
                }
            }
        }
        let mut owners = vec![(0, None); capture_count];
        for (root, branch) in branches.iter().enumerate() {
            if branch.captures.is_empty() {
                continue;
            }
            let begin = branch.leading.start;
            let end = branch.trailing.end;
            let mut alternative = 0;
            for (position, instruction) in prepared.iter().enumerate().take(end).skip(begin) {
                if let PreparedInstruction::Ready(Instruction::Open(slot)) = instruction {
                    let owner = if branch.alternatives.len() > 1
                        && position >= branch.leading.end
                        && position < branch.trailing.start
                    {
                        while position >= branch.alternatives[alternative].end {
                            alternative += 1;
                        }
                        Some(alternative)
                    } else {
                        None
                    };
                    owners[*slot] = (root, owner);
                }
            }
        }
        for slots in bindings.groups {
            let mut previous_slot = None;
            let mut previous_owner: Option<(usize, Option<usize>)> = None;
            for &slot in *slots {
                if previous_slot.is_some_and(|old| old >= slot) {
                    return Ok(None);
                }
                let owner = owners[slot];
                if let Some((root, alternative)) = previous_owner {
                    if root == owner.0
                        && (alternative.is_none() || owner.1.is_none() || alternative == owner.1)
                    {
                        return Ok(None);
                    }
                }
                previous_slot = Some(slot);
                previous_owner = Some(owner);
            }
        }
        charge(capture_names.len())?;
        charge(bindings.groups.len())?;
        charge(branches.len())?;
        for branch in &branches {
            charge(branch.alternatives.len())?;
        }
        charge(source.len())?;
        charge(prepared.len())?;
        charge(prepared.len())?;
        let set_count = prepared
            .iter()
            .filter(|instruction| matches!(instruction, PreparedInstruction::Set { .. }))
            .count();
        charge(set_count)?;
        charge(set_count)?;
        let mut sets = HashMap::<&[u16], RegExpCharacterMatcher>::with_capacity(set_count);
        let mut instructions = Vec::with_capacity(prepared.len());
        for instruction in prepared {
            instructions.push(match instruction {
                PreparedInstruction::Ready(instruction) => instruction,
                PreparedInstruction::Set {
                    plan,
                    source: range,
                } => {
                    let key = &source.code_units()[range];
                    charge(key.len())?;
                    charge(key.len())?;
                    let matcher = if let Some(matcher) = sets.get(key) {
                        matcher.clone()
                    } else {
                        charge(key.len())?;
                        let matcher = plan.compile_with_work(ignore_case, &mut charge)?;
                        sets.insert(key, matcher.clone());
                        matcher
                    };
                    Instruction::Set(matcher)
                }
            });
        }
        Ok(Some(Self(Arc::new(Program {
            instructions,
            capture_count,
            ignore_case,
            multiline,
            branches,
            capture_names,
            named_count: bindings.groups.len(),
        }))))
    }

    /// Number of source-order capturing groups.
    pub fn capture_count(&self) -> usize {
        self.0.capture_count
    }

    /// Finds the earliest match, or checks exactly `start` when sticky.
    pub fn find(
        &self,
        input: &JsString,
        start: usize,
        sticky: bool,
    ) -> Option<RegExpBackreferenceMatch> {
        self.find_with_work(input, start, sticky, |_| {
            Ok::<_, std::convert::Infallible>(())
        })
        .unwrap_or_else(|never| match never {})
    }

    /// Charges capture resets, executed instructions and actual unit comparisons.
    ///
    /// Capture buffers are reused across source-order branches at each ordered
    /// candidate start. Inner alternatives retain completed prefix captures and
    /// clear only captures completed by the failed body or continuation. Common
    /// prefixes execute once per branch; inactive captures remain undefined.
    /// An open or forward capture has no completed range and its reference matches
    /// the empty string, as required by BackreferenceMatcher (22.2.2.9.2). Ignore-case
    /// comparison canonicalizes both input units with the ordinary ASCII boundary.
    /// No recursion, backtracking stack or expanded reference string is used.
    pub fn find_with_work<E>(
        &self,
        input: &JsString,
        start: usize,
        sticky: bool,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<RegExpBackreferenceMatch>, E> {
        let input = input.code_units();
        if start > input.len() {
            return Ok(None);
        }
        charge(self.0.capture_count)?;
        charge(self.0.capture_count)?;
        charge(self.0.capture_count)?;
        charge(self.0.named_count)?;
        let mut state = CaptureState {
            captures: vec![None; self.0.capture_count],
            starts: vec![0; self.0.capture_count],
            named: vec![None; self.0.named_count],
            touched: Vec::with_capacity(self.0.capture_count),
        };
        let end = if sticky { start } else { input.len() };
        for candidate in start..=end {
            for branch in &self.0.branches {
                charge(1)?;
                self.0.clear_captures(&mut state, 0, &mut charge)?;
                let mut prefix_end = candidate;
                if !self.0.execute(
                    input,
                    &mut prefix_end,
                    branch.leading.clone(),
                    &mut state,
                    &mut charge,
                )? {
                    continue;
                }
                let checkpoint = state.touched.len();
                for alternative in &branch.alternatives {
                    charge(1)?;
                    self.0.clear_captures(&mut state, checkpoint, &mut charge)?;
                    let mut cursor = prefix_end;
                    if !self.0.execute(
                        input,
                        &mut cursor,
                        alternative.clone(),
                        &mut state,
                        &mut charge,
                    )? || !self.0.execute(
                        input,
                        &mut cursor,
                        branch.trailing.clone(),
                        &mut state,
                        &mut charge,
                    )? {
                        continue;
                    }
                    return Ok(Some(RegExpBackreferenceMatch {
                        range: candidate..cursor,
                        captures: state.captures.into_boxed_slice(),
                    }));
                }
            }
        }
        Ok(None)
    }
}

struct CaptureState {
    captures: Vec<Option<Range<usize>>>,
    starts: Vec<usize>,
    named: Vec<Option<Range<usize>>>,
    touched: Vec<usize>,
}

impl Program {
    fn clear_captures<E>(
        &self,
        state: &mut CaptureState,
        checkpoint: usize,
        charge: &mut impl FnMut(usize) -> Result<(), E>,
    ) -> Result<(), E> {
        charge(state.touched.len() - checkpoint)?;
        if !self.capture_names.is_empty() {
            charge(state.touched.len() - checkpoint)?;
        }
        for slot in state.touched.drain(checkpoint..) {
            state.captures[slot] = None;
            if let Some(Some(group)) = self.capture_names.get(slot) {
                state.named[*group] = None;
            }
        }
        Ok(())
    }

    fn execute<E>(
        &self,
        input: &[u16],
        cursor: &mut usize,
        range: Range<usize>,
        state: &mut CaptureState,
        charge: &mut impl FnMut(usize) -> Result<(), E>,
    ) -> Result<bool, E> {
        for instruction in &self.instructions[range] {
            charge(1)?;
            match instruction {
                Instruction::Character(expected) => {
                    let Some(&unit) = input.get(*cursor) else {
                        return Ok(false);
                    };
                    charge(1)?;
                    if canonicalize(unit, self.ignore_case) != *expected {
                        return Ok(false);
                    }
                    *cursor += 1;
                }
                Instruction::Set(matcher) => {
                    let Some(&unit) = input.get(*cursor) else {
                        return Ok(false);
                    };
                    charge(1)?;
                    if !matcher.matches(unit) {
                        return Ok(false);
                    }
                    *cursor += 1;
                }
                Instruction::Assert(assertion) => {
                    charge(2)?;
                    if !assertion.accepts(input, *cursor, self.multiline) {
                        return Ok(false);
                    }
                }
                Instruction::Open(index) => state.starts[*index] = *cursor,
                Instruction::Close(index) => {
                    charge(1)?;
                    let range = state.starts[*index]..*cursor;
                    state.captures[*index] = Some(range.clone());
                    state.touched.push(*index);
                    if !self.capture_names.is_empty() {
                        charge(2)?;
                        if let Some(group) = self.capture_names[*index] {
                            state.named[group] = Some(range);
                        }
                    }
                }
                Instruction::Reference(index) | Instruction::NamedReference(index) => {
                    let range = if matches!(instruction, Instruction::Reference(_)) {
                        &state.captures[*index]
                    } else {
                        &state.named[*index]
                    };
                    let Some(range) = range else {
                        continue;
                    };
                    let Some(next) = cursor.checked_add(range.len()) else {
                        return Ok(false);
                    };
                    let Some(units) = input.get(*cursor..next) else {
                        return Ok(false);
                    };
                    for (&actual, &captured) in units.iter().zip(&input[range.clone()]) {
                        charge(2)?;
                        if canonicalize(actual, self.ignore_case)
                            != canonicalize(captured, self.ignore_case)
                        {
                            return Ok(false);
                        }
                    }
                    *cursor = next;
                }
            }
        }
        Ok(true)
    }
}

fn canonicalize(unit: u16, ignore_case: bool) -> u16 {
    regexp_canonicalize_character(u32::from(unit), ignore_case, false) as u16
}

fn prepare(
    source: &[u16],
    ignore_case: bool,
    dot_all: bool,
    bindings: RegExpBackreferenceNamedBindings<'_>,
) -> Option<PreparedProgram> {
    let mut previous_end = 0;
    for reference in bindings.references {
        if reference.group >= bindings.groups.len() || reference.escape.start < previous_end {
            return None;
        }
        let units = source.get(reference.escape.clone())?;
        if units.len() < 5 || units.get(..3)? != [0x5c, 0x6b, 0x3c] || units.last() != Some(&0x3e) {
            return None;
        }
        previous_end = reference.escape.end;
    }
    let mut named_references = bindings.references.iter().peekable();
    let mut instructions = Vec::new();
    let mut groups = Vec::<GroupFrame>::new();
    let mut choice_started = false;
    let mut choice = None;
    let mut branches = Vec::new();
    let mut branch_start = 0;
    let mut branch_capture_start = 0;
    let mut capture_count = 0usize;
    let mut references = 0usize;
    let mut cursor = 0;
    while let Some(&unit) = source.get(cursor) {
        if named_references
            .peek()
            .is_some_and(|reference| reference.escape.start < cursor)
        {
            return None;
        }
        if named_references
            .peek()
            .is_some_and(|reference| reference.escape.start == cursor)
        {
            let reference = named_references.next().expect("reference selected");
            instructions.push(PreparedInstruction::Ready(Instruction::NamedReference(
                reference.group,
            )));
            references += 1;
            cursor = reference.escape.end;
            continue;
        }
        cursor += 1;
        match unit {
            0x28 => {
                let group = if source.get(cursor) == Some(&0x3f) {
                    if source.get(cursor..cursor + 2)? != [0x3f, 0x3a] {
                        return None;
                    }
                    cursor += 2;
                    None
                } else {
                    let index = capture_count;
                    capture_count = capture_count.checked_add(1)?;
                    instructions.push(PreparedInstruction::Ready(Instruction::Open(index)));
                    Some(index)
                };
                groups.push(GroupFrame {
                    capture: group,
                    body_start: instructions.len(),
                    alternative_start: instructions.len(),
                    alternatives: Vec::new(),
                });
            }
            0x29 => {
                let mut group = groups.pop()?;
                if !group.alternatives.is_empty() {
                    group
                        .alternatives
                        .push(group.alternative_start..instructions.len());
                    choice = Some(Choice {
                        start: group.body_start,
                        end: instructions.len(),
                        alternatives: group.alternatives,
                    });
                }
                if let Some(index) = group.capture {
                    instructions.push(PreparedInstruction::Ready(Instruction::Close(index)));
                }
            }
            0x5c if source
                .get(cursor)
                .is_some_and(|u| (0x31..=0x39).contains(u)) =>
            {
                let mut number = 0usize;
                while let Some(&digit @ 0x30..=0x39) = source.get(cursor) {
                    number = number
                        .checked_mul(10)?
                        .checked_add(usize::from(digit - 0x30))?;
                    cursor += 1;
                }
                instructions.push(PreparedInstruction::Ready(Instruction::Reference(
                    number.checked_sub(1)?,
                )));
                references += 1;
            }
            0x7c if groups.is_empty() => {
                branches.push(branch_plan(
                    branch_start,
                    instructions.len(),
                    branch_capture_start..capture_count,
                    choice.take(),
                ));
                branch_start = instructions.len();
                branch_capture_start = capture_count;
                choice_started = false;
            }
            0x7c => {
                let group = groups.last_mut()?;
                if group.alternatives.is_empty() {
                    if choice_started {
                        return None;
                    }
                    choice_started = true;
                }
                group
                    .alternatives
                    .push(group.alternative_start..instructions.len());
                group.alternative_start = instructions.len();
            }
            0x5e | 0x24 => {
                let mut assertion = Assertions::default();
                assertion.add_input_boundary(unit == 0x5e);
                instructions.push(PreparedInstruction::Ready(Instruction::Assert(assertion)));
            }
            0x5c if source
                .get(cursor)
                .is_some_and(|unit| matches!(unit, 0x62 | 0x42)) =>
            {
                let mut assertion = Assertions::default();
                assertion.add_word_boundary(source[cursor] == 0x62);
                cursor += 1;
                instructions.push(PreparedInstruction::Ready(Instruction::Assert(assertion)));
            }
            0x2e | 0x5b => {
                let start = cursor - 1;
                let (plan, length) = PreparedCharacter::parse(&source[start..], dot_all)?;
                cursor = start.checked_add(length)?;
                instructions.push(PreparedInstruction::Set {
                    plan: Box::new(plan),
                    source: start..cursor,
                });
            }
            0x5c if source
                .get(cursor)
                .is_some_and(|unit| matches!(unit, 0x64 | 0x44 | 0x73 | 0x53 | 0x77 | 0x57)) =>
            {
                let start = cursor - 1;
                let (plan, length) = PreparedCharacter::parse(&source[start..], dot_all)?;
                cursor = start.checked_add(length)?;
                instructions.push(PreparedInstruction::Set {
                    plan: Box::new(plan),
                    source: start..cursor,
                });
            }
            0x5c => instructions.push(PreparedInstruction::Ready(Instruction::Character(
                canonicalize(character_escape(source, &mut cursor)?, ignore_case),
            ))),
            _ if is_syntax(unit) || unit == 0x7c => return None,
            _ => instructions.push(PreparedInstruction::Ready(Instruction::Character(
                canonicalize(unit, ignore_case),
            ))),
        }
    }
    if !groups.is_empty() || references == 0 || named_references.peek().is_some() {
        return None;
    }
    if instructions.iter().any(|instruction| {
        matches!(instruction, PreparedInstruction::Ready(Instruction::Reference(index)) if *index >= capture_count)
    }) {
        return None;
    }
    branches.push(branch_plan(
        branch_start,
        instructions.len(),
        branch_capture_start..capture_count,
        choice,
    ));
    Some(PreparedProgram {
        instructions,
        capture_count,
        branches,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fmt::Write;

    fn named_escapes(source: &JsString) -> Vec<RegExpBackreferenceNamedReference> {
        source
            .code_units()
            .windows(5)
            .enumerate()
            .filter_map(|(start, units)| {
                if units[..3] == [0x5c, 0x6b, 0x3c] && units[4] == 0x3e {
                    Some(RegExpBackreferenceNamedReference {
                        escape: start..start + 5,
                        group: usize::from(units[3] - 0x78),
                    })
                } else {
                    None
                }
            })
            .collect()
    }

    #[test]
    fn named_binding_execution_snapshot() {
        let cases: &[(&str, &[&[usize]])] = &[
            (r"(a)\k<x>|(b)\k<x>", &[&[0, 1]]),
            (r"(a)|(\k<x>b)\k<x>", &[&[0, 1]]),
            (r"(\k<x>a)\k<x>|(b)\k<x>", &[&[0, 1]]),
            (r"(a)\k<x>0|(b)\k<x>0", &[&[0, 1]]),
            (r"((a)\k<x>)|(b)\k<x>", &[&[1, 2]]),
            (r"\k<x>(a)|(b)\k<x>", &[&[0, 1]]),
            (r"([µ])\k<x>|(\w)\k<x>", &[&[0, 1]]),
            (r"(a)\k<x>c|(b)\k<x>", &[&[0, 1]]),
            (r"^()\k<x>$|(\b)\k<x>\w", &[&[0, 1]]),
            (r"(a)(b)\k<x>\k<y>|(b)(a)\k<x>\k<y>", &[&[0, 2], &[1, 3]]),
            (r"^([\s\S])\k<x>$|^([ab])\k<x>$", &[&[0, 1]]),
            (r"(a)\k<x>|", &[&[0]]),
        ];
        let mut rows = String::new();
        for &(text, groups) in cases {
            let source = JsString::from(text);
            let references = named_escapes(&source);
            for (ignore_case, multiline, dot_all) in [
                (false, false, false),
                (true, true, false),
                (false, true, true),
            ] {
                let matcher = RegExpBackreferenceMatcher::compile_with_named_bindings_and_work(
                    &source,
                    ignore_case,
                    multiline,
                    dot_all,
                    RegExpBackreferenceNamedBindings {
                        groups,
                        references: &references,
                    },
                    |_| Ok::<_, ()>(()),
                )
                .unwrap()
                .unwrap();
                for text in [
                    "", "aa", "bb", "qbb", "aabb", "bb0", "Aa", "µΜ", " aa ", "\naa\n", "abab",
                    "baba",
                ] {
                    let input = JsString::from(text);
                    for (start, sticky) in [(0, false), (0, true), (1, true)] {
                        let found = matcher.find(&input, start, sticky);
                        writeln!(rows,"{source:?} groups={groups:?} i={ignore_case} m={multiline} s={dot_all} input={input:?} start={start} sticky={sticky} {found:?}").unwrap();
                    }
                }
            }
        }
        insta::assert_snapshot!(rows);
    }

    #[test]
    fn invalid_named_binding_metadata_returns_none_before_charging() {
        for (text, slots, escape, group) in [
            (r"(a)\k<x>", vec![0], 3..8, 1),
            (r"(a)\k<x>", vec![1], 3..8, 0),
            (r"(a)\k<x>", vec![], 3..8, 0),
            (r"(a)\k<x>", vec![0, 0], 3..8, 0),
            (r"(a)(b)\k<x>", vec![0, 1], 6..11, 0),
            (r"(a)\k<x>", vec![0], 3..9, 0),
            (r"(a)\k<x>", vec![0], 2..7, 0),
            (r"(a)[\k<x>]", vec![0], 4..9, 0),
            (r"(a)\k<x>+", vec![0], 3..8, 0),
        ] {
            let references = [RegExpBackreferenceNamedReference { escape, group }];
            let groups = [slots.as_slice()];
            let mut work = 0;
            assert!(
                RegExpBackreferenceMatcher::compile_with_named_bindings_and_work(
                    &JsString::from(text),
                    false,
                    false,
                    false,
                    RegExpBackreferenceNamedBindings {
                        groups: &groups,
                        references: &references
                    },
                    |n| {
                        work += n;
                        Ok::<_, ()>(())
                    }
                )
                .unwrap()
                .is_none(),
                "{text}"
            );
            assert_eq!(work, 0, "{text}");
        }
    }

    #[test]
    fn shared_named_bindings_keep_wide_choices_and_many_references_compact() {
        let source = JsString::from(
            format!("{}(a){}", "(b)|".repeat(10000), r"\k<x>".repeat(10000)).as_str(),
        );
        let references = named_escapes(&source);
        let slots: Vec<_> = (0..10001).collect();
        let groups = [slots.as_slice()];
        let matcher = RegExpBackreferenceMatcher::compile_with_named_bindings_and_work(
            &source,
            false,
            false,
            false,
            RegExpBackreferenceNamedBindings {
                groups: &groups,
                references: &references,
            },
            |_| Ok::<_, ()>(()),
        )
        .unwrap()
        .unwrap();
        assert!(matcher.0.instructions.len() <= source.len());
        assert_eq!(matcher.0.capture_names.len(), 10001);
        let mut work = 0;
        let found = matcher
            .find_with_work(&JsString::from("a".repeat(10001).as_str()), 0, true, |n| {
                work += n;
                Ok::<_, ()>(())
            })
            .unwrap()
            .unwrap();
        assert_eq!(found.range, 0..10001);
        assert!(found.captures[..10000].iter().all(Option::is_none));
        assert_eq!(found.captures[10000], Some(0..1));
        assert!(work < 150000, "actual named-reference work {work}");
    }

    #[test]
    fn inner_choice_binding_owners_reject_common_and_same_branch_duplicates() {
        for (text, slots) in [
            (r"(q)(?:(a)|(b))\k<x>", &[0, 1][..]),
            (r"(?:(a)|(b))(q)\k<x>", &[1, 2][..]),
            (r"(?:(a)(b)|(c))\k<x>", &[0, 1][..]),
            (r"((a)|(b))\k<x>", &[0, 2][..]),
        ] {
            let source = JsString::from(text);
            let references = named_escapes(&source);
            let mut work = 0;
            assert!(
                RegExpBackreferenceMatcher::compile_with_named_bindings_and_work(
                    &source,
                    false,
                    false,
                    false,
                    RegExpBackreferenceNamedBindings {
                        groups: &[slots],
                        references: &references,
                    },
                    |n| {
                        work += n;
                        Ok::<_, ()>(())
                    },
                )
                .unwrap()
                .is_none(),
                "{text}"
            );
            assert_eq!(work, 0, "{text}");
        }
        let source = JsString::from(r"(?:(a)|()|(b))\k<x>|(c)\k<x>");
        let references = named_escapes(&source);
        let matcher = RegExpBackreferenceMatcher::compile_with_named_bindings_and_work(
            &source,
            false,
            false,
            false,
            RegExpBackreferenceNamedBindings {
                groups: &[&[0, 1, 2, 3]],
                references: &references,
            },
            |_| Ok::<_, ()>(()),
        )
        .unwrap()
        .unwrap();
        let found = matcher.find(&JsString::from("b"), 0, true).unwrap();
        assert_eq!(found.range, 0..0);
        assert_eq!(&*found.captures, &[None, Some(0..0), None, None]);
    }

    #[test]
    fn inner_alternative_reference_execution_snapshot() {
        let mut rows = String::new();
        for source in [
            r"(a|b)\1",
            r"(a|ab)\1",
            r"(ab|a)\1",
            r"x(a|ab)\1y",
            r"(a|ab)\1b",
            r"((a)|(ab))\1",
            r"((a)|(ab))\2\3",
            r"(?:(a)|(b))\1\2",
            r"(?:(a)|)\1",
            r"(?:|(a))\1",
            r"(q)(?:(a)|(b))\1\2\3",
            r"(?:(a\1)|(b\2))\1\2",
            r"((\1a)|b)\1",
            r"^(\w|\W)\1$",
            r"([^µ]|[ab])\1",
            r"\b(a|\w)\1\b",
            r"((a)|())\1",
            r"x(((a)|b))\2y",
        ] {
            for (ignore_case, multiline, dot_all) in [
                (false, false, false),
                (true, true, false),
                (false, false, true),
                (false, true, true),
            ] {
                let body = JsString::from(source);
                let matcher = RegExpBackreferenceMatcher::compile_with_assertions_and_work(
                    &body,
                    ignore_case,
                    multiline,
                    dot_all,
                    |_| Ok::<_, ()>(()),
                )
                .unwrap()
                .unwrap();
                for text in [
                    "",
                    "aa",
                    "Aa",
                    "qaa",
                    "aa\n",
                    "\naa\n",
                    "\raa\r\n",
                    "aa aa",
                    " bb ",
                    "q\nbb\nq",
                    "µΜ",
                    "\n\n",
                    "a\n a",
                    "\u{2028}aa\u{2029}",
                    "\r\n",
                ] {
                    let input = JsString::from(text);
                    for (start, sticky) in [(0, false), (0, true), (1, true)] {
                        let found = matcher.find(&input, start, sticky);
                        writeln!(rows,"{body:?} i={ignore_case} m={multiline} s={dot_all} input={input:?} start={start} sticky={sticky} {found:?}").unwrap();
                    }
                }
            }
        }
        insta::assert_snapshot!(rows);
    }

    #[test]
    fn common_capture_checkpoints_survive_failed_choices_and_enclosing_closes() {
        let matcher =
            RegExpBackreferenceMatcher::compile(&JsString::from(r"((a)|(ab))\1"), false).unwrap();
        let found = matcher.find(&JsString::from("qabab"), 0, false).unwrap();
        assert_eq!(found.range, 1..5);
        assert_eq!(&*found.captures, &[Some(1..3), None, Some(1..3)]);
        let matcher =
            RegExpBackreferenceMatcher::compile(&JsString::from(r"(q)(?:(a)|(b))\1\2\3"), false)
                .unwrap();
        let found = matcher.find(&JsString::from("qbqb"), 0, true).unwrap();
        assert_eq!(&*found.captures, &[Some(0..1), None, Some(1..2)]);
        let matcher =
            RegExpBackreferenceMatcher::compile(&JsString::from(r"((a)|())\1"), false).unwrap();
        let found = matcher.find(&JsString::from("b"), 0, true).unwrap();
        assert_eq!(found.range, 0..0);
        assert_eq!(&*found.captures, &[Some(0..0), None, Some(0..0)]);
    }

    #[test]
    fn shared_common_regions_and_capture_checkpoints_keep_wide_choices_linear() {
        let source = JsString::from(
            format!("{}(?:(?:{})a)\\1", "(a)".repeat(10000), "b|".repeat(10000)).as_str(),
        );
        let matcher = RegExpBackreferenceMatcher::compile(&source, false).unwrap();
        assert!(matcher.0.instructions.len() <= source.len());
        let mut work = 0;
        let found = matcher
            .find_with_work(&JsString::from("a".repeat(10002).as_str()), 0, true, |n| {
                work += n;
                Ok::<_, ()>(())
            })
            .unwrap()
            .unwrap();
        assert_eq!(found.range, 0..10002);
        assert_eq!(found.captures[9999], Some(9999..10000));
        assert!(work < 200000, "actual prefix/alternative work {work}");
        let source = JsString::from(
            format!("{}(a|ab){}\\100001", "(".repeat(100000), ")".repeat(100000)).as_str(),
        );
        let matcher = RegExpBackreferenceMatcher::compile(&source, false).unwrap();
        let found = matcher.find(&JsString::from("abab"), 0, true).unwrap();
        assert_eq!(found.range, 0..4);
        assert_eq!(found.captures[100000], Some(0..2));
    }

    #[test]
    fn alternative_reference_execution_snapshot() {
        let mut rows = String::new();
        for source in [
            r"(a)\1|(b)\2",
            r"(a)\1|b",
            r"a|(ab)\1",
            r"(ab)\1|a",
            r"(a)\1|(a)\2",
            r"(a)\1|",
            r"|(a)\1",
            r"(a)\1|\1(b)",
            r"\2(a)\1|(b)\2",
            r"(a)\1|(\2b)\2",
            r"(\1a)\1|b",
            r"^()\1$|(\b)\2\w",
            r"([µ])\1|(\w)\2",
            r"([])\1|([^])\2",
            r"(a)\1|\B(.)\2\B",
            r"^^(a)\1$$|bb",
            r"(a)\1c|\1b",
            r"(a)\1c|(\1b)\2",
        ] {
            for (ignore_case, multiline, dot_all) in [
                (false, false, false),
                (true, true, false),
                (false, false, true),
                (false, true, true),
            ] {
                let body = JsString::from(source);
                let matcher = RegExpBackreferenceMatcher::compile_with_assertions_and_work(
                    &body,
                    ignore_case,
                    multiline,
                    dot_all,
                    |_| Ok::<_, ()>(()),
                )
                .unwrap()
                .unwrap();
                for text in [
                    "",
                    "aa",
                    "Aa",
                    "qaa",
                    "aa\n",
                    "\naa\n",
                    "\raa\r\n",
                    "aa aa",
                    " bb ",
                    "q\nbb\nq",
                    "µΜ",
                    "\n\n",
                    "a\n a",
                    "\u{2028}aa\u{2029}",
                    "\r\n",
                ] {
                    let input = JsString::from(text);
                    for (start, sticky) in [(0, false), (0, true), (1, true)] {
                        let found = matcher.find(&input, start, sticky);
                        writeln!(rows,"{body:?} i={ignore_case} m={multiline} s={dot_all} input={input:?} start={start} sticky={sticky} {found:?}").unwrap();
                    }
                }
            }
        }
        insta::assert_snapshot!(rows);
    }

    #[test]
    fn failed_branches_clear_only_their_original_capture_slots() {
        let matcher =
            RegExpBackreferenceMatcher::compile(&JsString::from(r"(a)\1c|(\1b)\2"), false).unwrap();
        let found = matcher.find(&JsString::from("aabb"), 0, false).unwrap();
        assert_eq!(found.range, 2..4);
        assert_eq!(&*found.captures, &[None, Some(2..3)]);
        let matcher =
            RegExpBackreferenceMatcher::compile(&JsString::from(r"(a)\1|b"), false).unwrap();
        let found = matcher.find(&JsString::from("baa"), 0, false).unwrap();
        assert_eq!(found.range, 0..1);
        assert_eq!(&*found.captures, &[None]);
        let matcher =
            RegExpBackreferenceMatcher::compile(&JsString::from(r"a|(ab)\1"), false).unwrap();
        assert_eq!(
            matcher
                .find(&JsString::from("abab"), 0, true)
                .unwrap()
                .range,
            0..1
        );
    }

    #[test]
    fn wide_reference_choices_have_linear_capture_resets_and_fallible_work() {
        let source = format!("{}(a)\\10001", "(b)|".repeat(10000));
        let matcher =
            RegExpBackreferenceMatcher::compile(&JsString::from(source.as_str()), false).unwrap();
        let mut work = 0;
        let found = matcher
            .find_with_work(&JsString::from("aa"), 0, true, |n| {
                work += n;
                Ok::<_, ()>(())
            })
            .unwrap()
            .unwrap();
        assert_eq!(found.range, 0..2);
        assert!(found.captures[..10000].iter().all(Option::is_none));
        assert_eq!(found.captures[10000], Some(0..1));
        assert!(work < 150000, "actual branch/reset work {work}");
        let mut work = 0;
        assert_eq!(
            matcher
                .find_with_work(&JsString::from("aa"), 0, true, |n| {
                    work += n;
                    if work > 1000 {
                        Err("explicit work")
                    } else {
                        Ok(())
                    }
                })
                .unwrap_err(),
            "explicit work"
        );
        let source = format!("{}b{}|(a)\\100001", "(".repeat(100000), ")".repeat(100000));
        let matcher =
            RegExpBackreferenceMatcher::compile(&JsString::from(source.as_str()), false).unwrap();
        let found = matcher.find(&JsString::from("aa"), 0, true).unwrap();
        assert_eq!(found.range, 0..2);
        assert!(found.captures[..100000].iter().all(Option::is_none));
        assert_eq!(found.captures[100000], Some(0..1));
    }

    #[test]
    fn asserted_reference_execution_snapshot() {
        let mut rows = String::new();
        for source in [
            r"^(a)\1$",
            r"\b(\w)\1\b",
            r"\B(.)\1\B",
            r"(\w)\b\1",
            r"(\w)\B\1",
            r"(a($))\1",
            r"(\1^a)\1",
            r"^()\1$",
            r"\b()\1\b",
            r"\B()\1\B",
            r"(\b)\1\w",
            r"(\B)\1.",
            r"(\1\b\w)\1",
            r"^([\s\S])\1$",
            r"\b\B(\w)\1",
            r"^^(\w)\1$$",
            r"\b(\w)\B\1\b",
            r"((a)\1)^",
        ] {
            for (ignore_case, multiline, dot_all) in [
                (false, false, false),
                (true, true, false),
                (false, false, true),
                (false, true, true),
            ] {
                let body = JsString::from(source);
                let matcher = RegExpBackreferenceMatcher::compile_with_assertions_and_work(
                    &body,
                    ignore_case,
                    multiline,
                    dot_all,
                    |_| Ok::<_, ()>(()),
                )
                .unwrap()
                .unwrap();
                for text in [
                    "",
                    "aa",
                    "Aa",
                    "qaa",
                    "aa\n",
                    "\naa\n",
                    "\raa\r\n",
                    "aa aa",
                    " bb ",
                    "q\nbb\nq",
                    "µΜ",
                    "\n\n",
                    "a\n a",
                    "\u{2028}aa\u{2029}",
                    "\r\n",
                ] {
                    let input = JsString::from(text);
                    for (start, sticky) in [(0, false), (0, true), (1, true)] {
                        let found = matcher.find(&input, start, sticky);
                        writeln!(rows,"{body:?} i={ignore_case} m={multiline} s={dot_all} input={input:?} start={start} sticky={sticky} {found:?}").unwrap();
                    }
                }
            }
        }
        insta::assert_snapshot!(rows);
    }

    #[test]
    fn absolute_assertion_positions_preserve_captures_and_ordinary_word_membership() {
        let matcher = RegExpBackreferenceMatcher::compile_with_assertions_and_work(
            &JsString::from(r"^(\w)\1$"),
            false,
            true,
            false,
            |_| Ok::<_, ()>(()),
        )
        .unwrap()
        .unwrap();
        let found = matcher.find(&JsString::from("q\naa\nz"), 0, false).unwrap();
        assert_eq!(found.range, 2..4);
        assert_eq!(&*found.captures, &[Some(2..3)]);
        let matcher =
            RegExpBackreferenceMatcher::compile(&JsString::from(r"\b()\1\b"), false).unwrap();
        assert_eq!(
            matcher.find(&JsString::from(" a"), 0, false).unwrap().range,
            1..1
        );
        let matcher =
            RegExpBackreferenceMatcher::compile(&JsString::from(r"\B(.)\1\B"), true).unwrap();
        assert_eq!(
            matcher.find(&JsString::from("µΜ"), 0, true).unwrap().range,
            0..2
        );
        assert!(matcher.find(&JsString::from("aa"), 0, true).is_none());
    }

    #[test]
    fn deep_assertion_streams_keep_flat_layouts_and_fallible_actual_search_work() {
        let text = format!("{}(a)\\1{}", "^".repeat(100000), "$".repeat(100000));
        let matcher =
            RegExpBackreferenceMatcher::compile(&JsString::from(text.as_str()), false).unwrap();
        assert_eq!(
            matcher.find(&JsString::from("aa"), 0, true).unwrap().range,
            0..2
        );
        let mut work = 0;
        assert_eq!(
            matcher
                .find_with_work(&JsString::from("aa"), 0, true, |n| {
                    work += n;
                    if work > 1000 {
                        Err("explicit work")
                    } else {
                        Ok(())
                    }
                })
                .unwrap_err(),
            "explicit work"
        );
        let text = format!(
            "{}(\\b)(a)\\100002{}",
            "(".repeat(100000),
            ")".repeat(100000)
        );
        let matcher =
            RegExpBackreferenceMatcher::compile(&JsString::from(text.as_str()), false).unwrap();
        let found = matcher.find(&JsString::from(" aa"), 0, false).unwrap();
        assert_eq!(found.range, 1..3);
        assert_eq!(found.captures[100000], Some(1..1));
        assert_eq!(found.captures[100001], Some(1..2));
    }

    #[test]
    fn character_reference_execution_snapshot() {
        let mut rows = String::new();
        for source in [
            r"([ab])\1",
            r"([^a])\1",
            r"([µ])\1",
            r"([ſ])\1",
            r"(\w)\1",
            r"(\W)\1",
            r"(\d)\1",
            r"(\D)\1",
            r"(\s)\1",
            r"(\S)\1",
            r"(.)\1",
            r"([^])\1",
            r"([])\1",
            r"\1(\w)",
            r"(\w\1)",
            r"([ab])(\1.)\2",
            r"(\w)(\w)\1\2",
            r"([\uD800])\1",
        ] {
            for ignore_case in [false, true] {
                for dot_all in [false, true] {
                    let body = JsString::from(source);
                    let matcher = RegExpBackreferenceMatcher::compile_with_flags_and_work(
                        &body,
                        ignore_case,
                        dot_all,
                        |_| Ok::<_, ()>(()),
                    )
                    .unwrap()
                    .unwrap();
                    for text in [
                        "", "aa", "aA", "bb", "11", "__", "µΜ", "ſS", "ſſ", "\n\n", "  ", "abab",
                        "abbb", "qabacac",
                    ] {
                        let input = JsString::from(text);
                        for (start, sticky) in [(0, false), (0, true), (1, true)] {
                            let found = matcher.find(&input, start, sticky);
                            writeln!(rows,"{body:?} i={ignore_case} s={dot_all} input={input:?} start={start} sticky={sticky} {found:?}").unwrap();
                        }
                    }
                }
            }
        }
        insta::assert_snapshot!(rows);
    }

    #[test]
    fn character_captures_and_ordinary_case_boundaries_use_original_input_units() {
        let matcher =
            RegExpBackreferenceMatcher::compile(&JsString::from(r"([ab])(\1.)\2"), false).unwrap();
        let found = matcher.find(&JsString::from("qaacac"), 0, false).unwrap();
        assert_eq!(found.range, 1..6);
        assert_eq!(&*found.captures, &[Some(1..2), Some(2..4)]);
        let matcher =
            RegExpBackreferenceMatcher::compile(&JsString::from(r"([µ])\1"), true).unwrap();
        assert_eq!(
            matcher.find(&JsString::from("Μµ"), 0, true).unwrap().range,
            0..2
        );
        let matcher =
            RegExpBackreferenceMatcher::compile(&JsString::from(r"(\w)\1"), true).unwrap();
        assert!(matcher.find(&JsString::from("ſſ"), 0, true).is_none());
        let matcher =
            RegExpBackreferenceMatcher::compile(&JsString::from(r"([\uD800])\1"), false).unwrap();
        assert_eq!(
            matcher
                .find(&JsString::from_code_units(vec![0xd800, 0xd800]), 0, true)
                .unwrap()
                .range,
            0..2
        );
    }

    #[test]
    fn identical_character_predicates_share_construction_in_deep_reference_programs() {
        let text = format!("{}\\1", r"(\w)".repeat(10000));
        let source = JsString::from(text.as_str());
        let mut work = 0;
        let matcher =
            RegExpBackreferenceMatcher::compile_with_flags_and_work(&source, true, false, |n| {
                work += n;
                Ok::<_, ()>(())
            })
            .unwrap()
            .unwrap();
        assert!(work < source.len() * 12 + 131072, "work={work}");
        let found = matcher
            .find(&JsString::from("a".repeat(10001).as_str()), 0, true)
            .unwrap();
        assert_eq!(found.range, 0..10001);
        assert_eq!(found.captures.len(), 10000);
        assert_eq!(found.captures[9999], Some(9999..10000));
        let text = format!("{}\\w{}\\100000", "(".repeat(100000), ")".repeat(100000));
        let matcher =
            RegExpBackreferenceMatcher::compile(&JsString::from(text.as_str()), false).unwrap();
        assert_eq!(
            matcher
                .find(&JsString::from("aa"), 0, true)
                .unwrap()
                .captures
                .len(),
            100000
        );
        for text in [r"(\w)\1+", r"^([ab])\1+", r"([ab]|c)\1+"] {
            let mut work = 0;
            assert!(
                RegExpBackreferenceMatcher::compile_with_flags_and_work(
                    &JsString::from(text),
                    true,
                    false,
                    |n| {
                        work += n;
                        Ok::<_, ()>(())
                    }
                )
                .unwrap()
                .is_none()
            );
            assert_eq!(work, 0);
        }
    }

    #[test]
    fn numbered_reference_execution_snapshot() {
        let mut rows = String::new();
        for (source, ignore_case) in [
            (r"(a)\1", false),
            (r"(a)\1", true),
            (r"\1(a)", false),
            (r"(a\1)", false),
            (r"(a(\1b)\2)\1", false),
            (r"()(a)\1\2", false),
            (r"(ab)(\1c)\2", false),
            (r"(?:q(a))\1", false),
            (r"(µ)\1", true),
            (r"(ſ)\1", true),
            (r"(\uD800)\1", false),
            (r"(\uD800\uDC00)\1", false),
            (r"(a)\x31\1", false),
            (r"(a)\\1\1", false),
            (r"()()()()()()()()()(a)\10", false),
        ] {
            let matcher =
                RegExpBackreferenceMatcher::compile(&JsString::from(source), ignore_case).unwrap();
            for text in [
                "",
                "a",
                "aa",
                "aA",
                "Aa",
                "qababcabc",
                "abbabb",
                "qaa",
                "µΜ",
                "ſS",
                "𐀀𐀀",
                "a1a",
                r"a\1a",
            ] {
                let input = JsString::from(text);
                for (start, sticky) in [(0, false), (0, true), (1, true)] {
                    let result = matcher.find(&input, start, sticky);
                    writeln!(rows, "{source:?} i={ignore_case} input={input:?} start={start} sticky={sticky} {result:?}").unwrap();
                }
            }
        }
        insta::assert_snapshot!(rows);
    }

    #[test]
    fn captures_references_and_canonicalization_follow_input_endpoints() {
        let matcher =
            RegExpBackreferenceMatcher::compile(&JsString::from(r"(a(\1b)\2)\1"), false).unwrap();
        let result = matcher.find(&JsString::from("qabbabb"), 0, false).unwrap();
        assert_eq!(result.range, 1..7);
        assert_eq!(&*result.captures, &[Some(1..4), Some(2..3)]);
        let matcher =
            RegExpBackreferenceMatcher::compile(&JsString::from(r"\2(a)(b)\1\2"), true).unwrap();
        let result = matcher.find(&JsString::from("qABab"), 0, false).unwrap();
        assert_eq!(result.range, 1..5);
        assert_eq!(&*result.captures, &[Some(1..2), Some(2..3)]);
        let matcher = RegExpBackreferenceMatcher::compile(&JsString::from(r"(ſ)\1"), true).unwrap();
        assert!(matcher.find(&JsString::from("ſS"), 0, false).is_none());
        assert!(matcher.find(&JsString::from("ſſ"), 0, false).is_some());
    }

    #[test]
    fn flat_programs_keep_deep_groups_and_references_compact_and_work_fallible() {
        let source = format!("{}a{}\\100000", "(".repeat(100000), ")".repeat(100000));
        let matcher =
            RegExpBackreferenceMatcher::compile(&JsString::from(source.as_str()), false).unwrap();
        let cloned = matcher.clone();
        drop(matcher);
        let result = cloned.find(&JsString::from("aa"), 0, true).unwrap();
        assert_eq!(result.range, 0..2);
        assert_eq!(result.captures.len(), 100000);
        assert!(result.captures.iter().all(|range| *range == Some(0..1)));
        let source = JsString::from(format!("(a){}", r"\1".repeat(100000)).as_str());
        let matcher = RegExpBackreferenceMatcher::compile(&source, false).unwrap();
        assert_eq!(
            matcher
                .find(&JsString::from("a".repeat(100001).as_str()), 0, true)
                .unwrap()
                .range,
            0..100001
        );
        let mut work = 0;
        assert_eq!(
            matcher
                .find_with_work(&JsString::from("a".repeat(100001).as_str()), 0, true, |n| {
                    work += n;
                    if work > 1000 {
                        Err("explicit work")
                    } else {
                        Ok(())
                    }
                })
                .unwrap_err(),
            "explicit work"
        );
        // Potential exponential expansion is retained as references and fails on
        // the short input without constructing an expanded string.
        let mut source = String::from("(a)");
        for index in 1..1000 {
            write!(source, "(\\{index}\\{index})").unwrap();
        }
        let source = JsString::from(source.as_str());
        let matcher = RegExpBackreferenceMatcher::compile(&source, false).unwrap();
        assert!(matcher.find(&JsString::from("aa"), 0, false).is_none());
        for source in [
            r"(a)\1+",
            r"(a|b)\1+",
            r"(?<x>a)\k<x>",
            r"([ab])\1+",
            r"^(a)\1+",
            r"(a)\2",
            "(a)",
        ] {
            let mut charged = 0;
            assert!(
                RegExpBackreferenceMatcher::compile_with_work(
                    &JsString::from(source),
                    false,
                    |n| {
                        charged += n;
                        Ok::<_, ()>(())
                    }
                )
                .unwrap()
                .is_none()
            );
            assert_eq!(charged, 0, "{source}");
        }
    }
}
