//! Ordinary character concatenations, assertions and references (22.2.2.4, 22.2.2.9).

use crate::regexp_assertion::Assertions;
use crate::regexp_character::PreparedCharacter;
use crate::regexp_literal::{character_escape, is_syntax};
use crate::regexp_quantified::{Bounds, quantifier};
use crate::{JsString, RegExpCharacterMatcher, regexp_canonicalize_character};
use std::{
    collections::{HashMap, HashSet},
    ops::Range,
    sync::Arc,
};

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
/// Top-level and arbitrarily nested alternatives, quantified reference atoms and
/// groups repeating references, one-unit character terms and ordinary assertions
/// are accepted, with whole capturing enclosures, empty captures and partial ranges.
/// Character atoms can also be quantified. Repeated groups may supply captures
/// to references outside their body. Body references use
/// completed current-iteration captures; forward and open targets remain empty.
/// Capture-free repeated choices accept nested/sequential branches when every
/// complete body path consumes a unit, with flat source-order iteration retries.
/// Deterministic capture-free inner quantifiers compose with these body paths.
/// Capture-free branch loops also nest through flat parent-linked contexts.
/// Other quantifiers return `None`. At least one numbered or registered named
/// reference is required by the reference-specific constructors. The ordinary
/// fallback constructor also accepts bodies without references.
/// References are never expanded into source or compiled literal strings.
#[derive(Clone, Debug)]
pub struct RegExpBackreferenceMatcher(Arc<Program>);

#[derive(Debug)]
struct Program {
    instructions: Vec<Instruction>,
    capture_count: usize,
    ignore_case: bool,
    multiline: bool,
    frame_capacity: usize,
    capture_names: Vec<Option<usize>>,
    named_count: usize,
}

#[derive(Debug)]
enum Instruction {
    Nop,
    Choice(Box<[usize]>),
    Jump(usize),
    RepeatChoice {
        body: usize,
        head: usize,
        end: usize,
        bounds: Bounds,
    },
    RepeatChoiceEnd(usize),
    Character(u16),
    Set(RegExpCharacterMatcher),
    // A sequence repetition owns these terms; ordinary traversal skips them.
    RepeatedCharacter(u16),
    RepeatedSet(RegExpCharacterMatcher),
    RepeatedAssert(Assertions),
    Assert(Assertions),
    Open(usize),
    Close(usize),
    Reference(usize),
    NamedReference(usize),
    QuantifiedReferenceSequence {
        references: Box<[ReferenceTarget]>,
        bounds: Bounds,
        captures: Box<[(usize, ReferenceCaptureSpan)]>,
    },
    QuantifiedReference {
        index: usize,
        named: bool,
        bounds: Bounds,
        copies: usize,
        captures: Box<[(usize, ReferenceCaptureSpan)]>,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ReferenceCaptureSpan {
    start: usize,
    end: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ReferenceTarget {
    Input { index: usize, named: bool },
    Local(ReferenceCaptureSpan),
    Empty,
    Term(usize),
}

enum ReferenceRange {
    Input(Range<usize>),
    Local(Range<usize>),
    Term(usize),
}

enum PreparedInstruction {
    Ready(Instruction),
    Set {
        plan: Box<PreparedCharacter>,
        source: Range<usize>,
        repeated: bool,
    },
}

struct GroupFrame {
    capture: Option<usize>,
    entry: usize,
    alternatives: Vec<usize>,
    exits: Vec<usize>,
}

impl GroupFrame {
    fn open(capture: Option<usize>, instructions: &mut Vec<PreparedInstruction>) -> Self {
        let entry = instructions.len();
        instructions.push(PreparedInstruction::Ready(Instruction::Nop));
        Self {
            capture,
            entry,
            alternatives: Vec::new(),
            exits: Vec::new(),
        }
    }
    fn next_alternative(&mut self, instructions: &mut Vec<PreparedInstruction>) {
        if self.alternatives.is_empty() {
            self.alternatives.push(self.entry + 1);
        }
        self.exits.push(instructions.len());
        instructions.push(PreparedInstruction::Ready(Instruction::Jump(usize::MAX)));
        self.alternatives.push(instructions.len());
    }
    fn finish(self, instructions: &mut [PreparedInstruction]) -> Option<usize> {
        if !self.alternatives.is_empty() {
            instructions[self.entry] = PreparedInstruction::Ready(Instruction::Choice(
                self.alternatives.into_boxed_slice(),
            ));
            let end = instructions.len();
            for exit in self.exits {
                instructions[exit] = PreparedInstruction::Ready(Instruction::Jump(end));
            }
        }
        self.capture
    }
}

struct ChoiceFrame {
    instruction: usize,
    checkpoint: usize,
    iteration_checkpoint: usize,
    alternative: PendingAlternative,
    iteration: Option<usize>,
}

#[derive(Clone, Copy)]
struct ChoiceIteration {
    parent: Option<usize>,
    instruction: usize,
    count: usize,
    start: usize,
}

enum PendingAlternative {
    RepeatChoice {
        enter: Option<bool>,
    },
    Choice {
        next: usize,
        prefix_end: usize,
    },
    SequenceRepetition {
        next: Option<usize>,
        limit: usize,
        base: usize,
        captures: Box<[ReferenceRange]>,
        offsets: Box<[usize]>,
        width: usize,
        greedy: bool,
    },
    Repetition {
        next: Option<usize>,
        limit: usize,
        base: usize,
        capture: Range<usize>,
        copies: usize,
        greedy: bool,
    },
}

impl ChoiceFrame {
    fn retry<E>(
        &mut self,
        program: &Program,
        input: &[u16],
        state: &mut CaptureState,
        charge: &mut impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<(usize, usize)>, E> {
        match &mut self.alternative {
            PendingAlternative::RepeatChoice { enter } => {
                let Some(enter) = enter.take() else {
                    return Ok(None);
                };
                let Instruction::RepeatChoice { head, end, .. } =
                    program.instructions[self.instruction]
                else {
                    unreachable!("choice repetition frame retains its entry")
                };
                let context =
                    state.iterations[state.iteration.expect("choice repetition context restored")];
                if enter {
                    Ok(Some((head, context.start)))
                } else {
                    state.iteration = context.parent;
                    Ok(Some((end, context.start)))
                }
            }
            PendingAlternative::Choice { next, prefix_end } => {
                let Instruction::Choice(alternatives) = &program.instructions[self.instruction]
                else {
                    unreachable!("choice frame refers to a choice")
                };
                let Some(&pc) = alternatives.get(*next) else {
                    return Ok(None);
                };
                *next += 1;
                Ok(Some((pc, *prefix_end)))
            }
            PendingAlternative::Repetition {
                next,
                limit,
                base,
                capture,
                copies,
                greedy,
            } => {
                let Some(count) = *next else {
                    return Ok(None);
                };
                if *greedy {
                    // The constructor bounds counts by actual remaining input units.
                    let width = capture.len() * *copies;
                    let end = *base + count * width;
                    *next = (count > *limit).then(|| count - 1);
                    program.complete_reference_captures(
                        self.instruction,
                        state,
                        (count > 0).then(|| end - width..end),
                        None,
                        charge,
                    )?;
                    Ok(Some((self.instruction + 1, end)))
                } else {
                    let width = capture.len() * *copies;
                    let start = *base + (count - 1) * width;
                    let Some(end) =
                        program.compare_reference_copies(input, start, capture, *copies, charge)?
                    else {
                        *next = None;
                        return Ok(None);
                    };
                    *next = (count < *limit).then(|| count + 1);
                    program.complete_reference_captures(
                        self.instruction,
                        state,
                        (count > 0).then(|| end - width..end),
                        None,
                        charge,
                    )?;
                    Ok(Some((self.instruction + 1, end)))
                }
            }
            PendingAlternative::SequenceRepetition {
                next,
                limit,
                base,
                captures,
                offsets,
                width,
                greedy,
            } => {
                let Some(count) = *next else {
                    return Ok(None);
                };
                let end = if *greedy {
                    *base + count * *width
                } else {
                    let start = *base + (count - 1) * *width;
                    let Some(end) =
                        program.compare_reference_sequence(input, start, captures, charge)?
                    else {
                        *next = None;
                        return Ok(None);
                    };
                    end
                };
                *next = if *greedy {
                    (count > *limit).then(|| count - 1)
                } else {
                    (count < *limit).then(|| count + 1)
                };
                program.complete_reference_captures(
                    self.instruction,
                    state,
                    (count > 0).then(|| end - *width..end),
                    Some(offsets),
                    charge,
                )?;
                Ok(Some((self.instruction + 1, end)))
            }
        }
    }
}

enum ScopeStep {
    Capture(usize),
    Open,
    Close,
    Alternative,
}

#[derive(Default)]
struct NameScope {
    current: HashSet<usize>,
    alternatives: HashSet<usize>,
}

fn union_names(target: &mut HashSet<usize>, mut source: HashSet<usize>) -> usize {
    if target.len() < source.len() {
        std::mem::swap(target, &mut source);
    }
    let work = source.len();
    target.extend(source);
    work
}

// Apply MightBothParticipate without copying names through every enclosing group.
// Keep preflight invalid/unsupported outcomes independent of host charging, then
// charge the recorded set operations once the complete program is accepted.
fn exclusive_names(steps: &[ScopeStep], names: &[Option<usize>]) -> Option<usize> {
    if names.is_empty() {
        return Some(0);
    }
    let mut scopes = Vec::new();
    let mut current = NameScope::default();
    let mut work = 0usize;
    for step in steps {
        work = work.saturating_add(1);
        match step {
            ScopeStep::Capture(slot) => {
                if let Some(Some(name)) = names.get(*slot) {
                    work = work.saturating_add(1);
                    if !current.current.insert(*name) {
                        return None;
                    }
                }
            }
            ScopeStep::Open => scopes.push(std::mem::take(&mut current)),
            ScopeStep::Close => {
                work = work.saturating_add(union_names(&mut current.alternatives, current.current));
                let children = current.alternatives;
                current = scopes.pop().expect("prepared group is balanced");
                work = work.saturating_add(current.current.len().min(children.len()));
                if !current.current.is_disjoint(&children) {
                    return None;
                }
                work = work.saturating_add(union_names(&mut current.current, children));
            }
            ScopeStep::Alternative => {
                work = work.saturating_add(union_names(
                    &mut current.alternatives,
                    std::mem::take(&mut current.current),
                ));
            }
        }
    }
    Some(work)
}

struct PreparedProgram {
    instructions: Vec<PreparedInstruction>,
    capture_count: usize,
    scopes: Vec<ScopeStep>,
    references: usize,
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
        charge: impl FnMut(usize) -> Result<(), E>,
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
        if plan.references == 0 {
            return Ok(None);
        }
        Self::compile_prepared(source, ignore_case, multiline, bindings, plan, charge)
    }

    /// Compiles the supported ordinary Pattern subset, including no-reference bodies.
    ///
    /// This fallback retains the complete original capture layout and the same
    /// validated named bindings. Unsupported syntax is rejected before work is
    /// charged; specialized literal/sequence matchers can still run first.
    pub fn compile_ordinary_with_named_bindings_and_work<E>(
        source: &JsString,
        ignore_case: bool,
        multiline: bool,
        dot_all: bool,
        bindings: RegExpBackreferenceNamedBindings<'_>,
        charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, E> {
        let Some(plan) = prepare(source.code_units(), ignore_case, dot_all, bindings) else {
            return Ok(None);
        };
        Self::compile_prepared(source, ignore_case, multiline, bindings, plan, charge)
    }

    fn compile_prepared<E>(
        source: &JsString,
        ignore_case: bool,
        multiline: bool,
        bindings: RegExpBackreferenceNamedBindings<'_>,
        plan: PreparedProgram,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, E> {
        let PreparedProgram {
            instructions: prepared,
            capture_count,
            scopes,
            references: _,
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
        if bindings
            .groups
            .iter()
            .any(|slots| slots.windows(2).any(|p| p[0] >= p[1]))
        {
            return Ok(None);
        }
        let Some(binding_work) = exclusive_names(&scopes, &capture_names) else {
            return Ok(None);
        };
        charge(binding_work)?;
        charge(capture_names.len())?;
        charge(bindings.groups.len())?;
        let choice_count = prepared
            .iter()
            .filter(|i| matches!(i, PreparedInstruction::Ready(Instruction::Choice(_))))
            .count();
        let repetitions = prepared
            .iter()
            .filter(|i| {
                matches!(
                    i,
                    PreparedInstruction::Ready(
                        Instruction::QuantifiedReference { .. }
                            | Instruction::QuantifiedReferenceSequence { .. }
                            | Instruction::RepeatChoice { .. }
                    )
                )
            })
            .count();
        let frame_capacity = choice_count + repetitions;
        charge(frame_capacity)?;
        for instruction in &prepared {
            if let PreparedInstruction::Ready(Instruction::Choice(alternatives)) = instruction {
                charge(alternatives.len())?;
            }
        }
        for instruction in &prepared {
            if let PreparedInstruction::Ready(Instruction::QuantifiedReference {
                captures, ..
            }) = instruction
            {
                charge(captures.len())?;
            }
        }
        for instruction in &prepared {
            if let PreparedInstruction::Ready(Instruction::QuantifiedReferenceSequence {
                references,
                captures,
                ..
            }) = instruction
            {
                charge(references.len())?;
                charge(captures.len())?;
            }
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
                    repeated,
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
                    if repeated {
                        Instruction::RepeatedSet(matcher)
                    } else {
                        Instruction::Set(matcher)
                    }
                }
            });
        }
        Ok(Some(Self(Arc::new(Program {
            instructions,
            capture_count,
            ignore_case,
            multiline,
            frame_capacity,
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
    /// terms are retained for each reachable prefix; inactive captures stay undefined.
    /// An open or forward capture has no completed range and its reference matches
    /// the empty string, as required by BackreferenceMatcher (22.2.2.9.2). Ignore-case
    /// comparison canonicalizes both input units with the ordinary ASCII boundary.
    /// Pending alternatives use a flat frame buffer; no recursion or expanded
    /// reference string is used.
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
            iteration: None,
            iterations: Vec::new(),
        };
        charge(self.0.frame_capacity)?;
        let mut frames = Vec::<ChoiceFrame>::with_capacity(self.0.frame_capacity);
        let end = if sticky { start } else { input.len() };
        'candidate: for candidate in start..=end {
            charge(1)?;
            self.0.clear_captures(&mut state, 0, &mut charge)?;
            frames.clear();
            let mut cursor = candidate;
            let mut pc = 0;
            state.iteration = None;
            state.iterations.clear();
            loop {
                if pc == self.0.instructions.len() {
                    return Ok(Some(RegExpBackreferenceMatch {
                        range: candidate..cursor,
                        captures: state.captures.into_boxed_slice(),
                    }));
                }
                if let Some(next) =
                    self.0
                        .advance(input, &mut cursor, pc, &mut state, &mut frames, &mut charge)?
                {
                    pc = next;
                    continue;
                }
                loop {
                    let Some(frame) = frames.last_mut() else {
                        continue 'candidate;
                    };
                    charge(1)?;
                    self.0
                        .clear_captures(&mut state, frame.checkpoint, &mut charge)?;
                    state.iterations.truncate(frame.iteration_checkpoint);
                    state.iteration = frame.iteration;
                    if let Some((next, end)) =
                        frame.retry(&self.0, input, &mut state, &mut charge)?
                    {
                        cursor = end;
                        pc = next;
                        break;
                    }
                    frames.pop();
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
    iteration: Option<usize>,
    // Immutable parent-linked records; retry checkpoints reclaim abandoned paths.
    iterations: Vec<ChoiceIteration>,
}

impl Program {
    fn choose_repeat<E>(
        &self,
        input: &[u16],
        cursor: usize,
        state: &mut CaptureState,
        frames: &mut Vec<ChoiceFrame>,
        charge: &mut impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<usize>, E> {
        let index = state.iteration.expect("choice repetition initialized");
        let context = state.iterations[index];
        let Instruction::RepeatChoice {
            head,
            end,
            bounds: (min, max, greedy),
            ..
        } = self.instructions[context.instruction]
        else {
            unreachable!("choice repetition retains its entry")
        };
        let Some(min) = min else {
            return Ok(None);
        };
        if min.saturating_sub(context.count) > input.len() - cursor {
            return Ok(None);
        }
        let can_exit = context.count >= min;
        let can_enter = context.count < max.unwrap_or(usize::MAX);
        if !can_exit && !can_enter {
            return Ok(None);
        }
        let enter = can_enter && (greedy || !can_exit);
        if can_enter && can_exit {
            charge(1)?;
            frames.push(ChoiceFrame {
                instruction: context.instruction,
                checkpoint: state.touched.len(),
                iteration_checkpoint: state.iterations.len(),
                alternative: PendingAlternative::RepeatChoice {
                    enter: Some(!enter),
                },
                iteration: Some(index),
            });
        }
        if enter {
            Ok(Some(head))
        } else {
            state.iteration = context.parent;
            Ok(Some(end))
        }
    }

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

    fn complete_reference_captures<E>(
        &self,
        pc: usize,
        state: &mut CaptureState,
        range: Option<Range<usize>>,
        offsets: Option<&[usize]>,
        charge: &mut impl FnMut(usize) -> Result<(), E>,
    ) -> Result<(), E> {
        let captures = match &self.instructions[pc] {
            Instruction::QuantifiedReference { captures, .. }
            | Instruction::QuantifiedReferenceSequence { captures, .. } => captures,
            _ => unreachable!("reference repetition owns its capture inventory"),
        };
        if let Some(range) = range {
            for &(slot, span) in captures {
                let offset = |index| {
                    if let Some(offsets) = offsets {
                        offsets[index]
                    } else {
                        let Instruction::QuantifiedReference { copies, .. } =
                            &self.instructions[pc]
                        else {
                            unreachable!("mixed targets provide prefix offsets")
                        };
                        range.len() / *copies * index
                    }
                };
                let range = range.start + offset(span.start)..range.start + offset(span.end);
                charge(1)?;
                state.captures[slot] = Some(range.clone());
                state.touched.push(slot);
                if !self.capture_names.is_empty() {
                    charge(2)?;
                    if let Some(group) = self.capture_names[slot] {
                        state.named[group] = Some(range.clone());
                    }
                }
            }
        }
        Ok(())
    }

    fn advance<E>(
        &self,
        input: &[u16],
        cursor: &mut usize,
        pc: usize,
        state: &mut CaptureState,
        frames: &mut Vec<ChoiceFrame>,
        charge: &mut impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<usize>, E> {
        let instruction = &self.instructions[pc];
        charge(1)?;
        match instruction {
            Instruction::Nop
            | Instruction::RepeatedCharacter(_)
            | Instruction::RepeatedSet(_)
            | Instruction::RepeatedAssert(_) => {}
            Instruction::Jump(target) => return Ok(Some(*target)),
            Instruction::Choice(alternatives) => {
                charge(1)?;
                frames.push(ChoiceFrame {
                    instruction: pc,
                    alternative: PendingAlternative::Choice {
                        next: 1,
                        prefix_end: *cursor,
                    },
                    checkpoint: state.touched.len(),
                    iteration_checkpoint: state.iterations.len(),
                    iteration: state.iteration,
                });
                return Ok(Some(alternatives[0]));
            }
            Instruction::RepeatChoice { .. } => {
                charge(1)?;
                let index = state.iterations.len();
                state.iterations.push(ChoiceIteration {
                    parent: state.iteration,
                    instruction: pc,
                    count: 0,
                    start: *cursor,
                });
                state.iteration = Some(index);
                return self.choose_repeat(input, *cursor, state, frames, charge);
            }
            Instruction::RepeatChoiceEnd(entry) => {
                let mut context =
                    state.iterations[state.iteration.expect("choice body retains its repetition")];
                debug_assert_eq!(context.instruction, *entry);
                debug_assert!(*cursor > context.start);
                // Each accepted body consumes at least one unit. Its count
                // cannot exceed the input length or overflow usize.
                context.count += 1;
                context.start = *cursor;
                charge(1)?;
                let index = state.iterations.len();
                state.iterations.push(context);
                state.iteration = Some(index);
                return self.choose_repeat(input, *cursor, state, frames, charge);
            }
            Instruction::Character(expected) => {
                let Some(&unit) = input.get(*cursor) else {
                    return Ok(None);
                };
                charge(1)?;
                if canonicalize(unit, self.ignore_case) != *expected {
                    return Ok(None);
                }
                *cursor += 1;
            }
            Instruction::Set(matcher) => {
                let Some(&unit) = input.get(*cursor) else {
                    return Ok(None);
                };
                charge(1)?;
                if !matcher.matches(unit) {
                    return Ok(None);
                }
                *cursor += 1;
            }
            Instruction::Assert(assertion) => {
                charge(2)?;
                if !assertion.accepts(input, *cursor, self.multiline) {
                    return Ok(None);
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
                if let Some(range) = range {
                    let Some(next) = self.compare_reference(input, *cursor, range, charge)? else {
                        return Ok(None);
                    };
                    *cursor = next;
                }
            }
            Instruction::QuantifiedReferenceSequence {
                references,
                bounds: (min, max, greedy),
                ..
            } => {
                charge(references.len())?;
                charge(references.len() + 1)?;
                let mut offsets = Vec::with_capacity(references.len() + 1);
                offsets.push(0usize);
                let mut captures = Vec::with_capacity(references.len());
                for &target in references {
                    charge(1)?;
                    let range = match target {
                        ReferenceTarget::Input { index, named } => {
                            let range = if named {
                                &state.named[index]
                            } else {
                                &state.captures[index]
                            };
                            range
                                .as_ref()
                                .filter(|r| !r.is_empty())
                                .cloned()
                                .map(ReferenceRange::Input)
                        }
                        ReferenceTarget::Local(span) => {
                            charge(2)?;
                            let range = offsets[span.start]..offsets[span.end];
                            (!range.is_empty()).then_some(ReferenceRange::Local(range))
                        }
                        ReferenceTarget::Empty => None,
                        ReferenceTarget::Term(position) => Some(ReferenceRange::Term(position)),
                    };
                    let length = range.as_ref().map_or(0, |range| match range {
                        ReferenceRange::Input(range) | ReferenceRange::Local(range) => range.len(),
                        ReferenceRange::Term(position) => usize::from(!matches!(
                            self.instructions[*position],
                            Instruction::RepeatedAssert(_)
                        )),
                    });
                    let Some(width) = offsets.last().unwrap().checked_add(length) else {
                        return Ok((*min == Some(0)).then_some(pc + 1));
                    };
                    offsets.push(width);
                    if let Some(range) = range {
                        captures.push(range);
                    }
                }
                let width = *offsets.last().unwrap();
                if width == 0 {
                    // Required empty iterations share one assertion position;
                    // optional empty iterations stop without capture effects.
                    if *min != Some(0)
                        && self
                            .compare_reference_sequence(input, *cursor, &captures, charge)?
                            .is_none()
                    {
                        return Ok(None);
                    }
                    self.complete_reference_captures(
                        pc,
                        state,
                        (*min != Some(0)).then_some(*cursor..*cursor),
                        Some(&offsets),
                        charge,
                    )?;
                    return Ok(Some(pc + 1));
                }
                let Some(min) = *min else {
                    return Ok(None);
                };
                let limit = max
                    .unwrap_or(usize::MAX)
                    .min((input.len() - *cursor) / width);
                if min > limit {
                    return Ok(None);
                }
                let base = *cursor;
                let mut count = 0;
                let wanted = if *greedy { limit } else { min };
                while count < wanted {
                    charge(1)?;
                    let Some(end) =
                        self.compare_reference_sequence(input, *cursor, &captures, charge)?
                    else {
                        break;
                    };
                    *cursor = end;
                    count += 1;
                }
                if count < min {
                    return Ok(None);
                }
                let next = if *greedy {
                    (count > min).then(|| count - 1)
                } else {
                    (count < limit).then(|| count + 1)
                };
                let checkpoint = state.touched.len();
                self.complete_reference_captures(
                    pc,
                    state,
                    (count > 0).then(|| *cursor - width..*cursor),
                    Some(&offsets),
                    charge,
                )?;
                if next.is_some() {
                    charge(1)?;
                    frames.push(ChoiceFrame {
                        instruction: pc,
                        checkpoint,
                        iteration_checkpoint: state.iterations.len(),
                        iteration: state.iteration,
                        alternative: PendingAlternative::SequenceRepetition {
                            next,
                            limit: if *greedy { min } else { limit },
                            base,
                            captures: captures.into_boxed_slice(),
                            offsets: offsets.into_boxed_slice(),
                            width,
                            greedy: *greedy,
                        },
                    });
                }
            }
            Instruction::QuantifiedReference {
                index,
                named,
                bounds: (min, max, greedy),
                copies,
                ..
            } => {
                let range = if *named {
                    &state.named[*index]
                } else {
                    &state.captures[*index]
                };
                // A sole reference is deterministic. Required empty iterations
                // leave only the final enclosing ranges; optional empty iterations
                // fail RepeatMatcher's zero-progress check and leave them undefined.
                let Some(capture) = range.as_ref().filter(|r| !r.is_empty()).cloned() else {
                    self.complete_reference_captures(
                        pc,
                        state,
                        (*min != Some(0)).then_some(*cursor..*cursor),
                        None,
                        charge,
                    )?;
                    return Ok(Some(pc + 1));
                };
                let Some(min) = *min else {
                    return Ok(None);
                };
                let limit = max
                    .unwrap_or(usize::MAX)
                    .min((input.len() - *cursor) / capture.len() / *copies);
                if min > limit {
                    return Ok(None);
                }
                let base = *cursor;
                let mut count = 0;
                let wanted = if *greedy { limit } else { min };
                while count < wanted {
                    charge(1)?;
                    let Some(next) =
                        self.compare_reference_copies(input, *cursor, &capture, *copies, charge)?
                    else {
                        break;
                    };
                    *cursor = next;
                    count += 1;
                }
                if count < min {
                    return Ok(None);
                }
                let next = if *greedy {
                    (count > min).then(|| count - 1)
                } else {
                    (count < limit).then(|| count + 1)
                };
                let checkpoint = state.touched.len();
                self.complete_reference_captures(
                    pc,
                    state,
                    (count > 0).then(|| *cursor - capture.len() * *copies..*cursor),
                    None,
                    charge,
                )?;
                if next.is_some() {
                    charge(1)?;
                    frames.push(ChoiceFrame {
                        instruction: pc,
                        checkpoint,
                        iteration_checkpoint: state.iterations.len(),
                        iteration: state.iteration,
                        alternative: PendingAlternative::Repetition {
                            next,
                            limit: if *greedy { min } else { limit },
                            base,
                            capture: capture.clone(),
                            copies: *copies,
                            greedy: *greedy,
                        },
                    });
                }
            }
        }
        Ok(Some(pc + 1))
    }
    fn compare_reference_sequence<E>(
        &self,
        input: &[u16],
        mut start: usize,
        captures: &[ReferenceRange],
        charge: &mut impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<usize>, E> {
        let base = start;
        for capture in captures {
            charge(1)?;
            let range = match capture {
                ReferenceRange::Input(range) => range.clone(),
                // Count limits bound the complete body before any comparison.
                ReferenceRange::Local(range) => base + range.start..base + range.end,
                ReferenceRange::Term(position) => {
                    charge(1)?;
                    if let Instruction::RepeatedAssert(assertion) = &self.instructions[*position] {
                        charge(2)?;
                        if !assertion.accepts(input, start, self.multiline) {
                            return Ok(None);
                        }
                        continue;
                    }
                    let Some(&unit) = input.get(start) else {
                        return Ok(None);
                    };
                    let matches = match &self.instructions[*position] {
                        Instruction::RepeatedCharacter(expected) => {
                            canonicalize(unit, self.ignore_case) == *expected
                        }
                        Instruction::RepeatedSet(matcher) => matcher.matches(unit),
                        _ => unreachable!("repeated consuming terms retain their flat instruction"),
                    };
                    if !matches {
                        return Ok(None);
                    }
                    start += 1;
                    continue;
                }
            };
            let Some(end) = self.compare_reference(input, start, &range, charge)? else {
                return Ok(None);
            };
            start = end;
        }
        Ok(Some(start))
    }

    fn compare_reference_copies<E>(
        &self,
        input: &[u16],
        mut start: usize,
        capture: &Range<usize>,
        copies: usize,
        charge: &mut impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<usize>, E> {
        for _ in 0..copies {
            charge(1)?;
            let Some(end) = self.compare_reference(input, start, capture, charge)? else {
                return Ok(None);
            };
            start = end;
        }
        Ok(Some(start))
    }

    fn compare_reference<E>(
        &self,
        input: &[u16],
        start: usize,
        capture: &Range<usize>,
        charge: &mut impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<usize>, E> {
        let Some(next) = start.checked_add(capture.len()) else {
            return Ok(None);
        };
        let Some(units) = input.get(start..next) else {
            return Ok(None);
        };
        for (&actual, &captured) in units.iter().zip(&input[capture.clone()]) {
            charge(2)?;
            if canonicalize(actual, self.ignore_case) != canonicalize(captured, self.ignore_case) {
                return Ok(None);
            }
        }
        Ok(Some(next))
    }
}

fn canonicalize(unit: u16, ignore_case: bool) -> u16 {
    regexp_canonicalize_character(u32::from(unit), ignore_case, false) as u16
}

fn reference_instruction(
    source: &[u16],
    cursor: &mut usize,
    index: usize,
    named: bool,
) -> Option<Instruction> {
    if let Some((bounds, consumed)) = quantifier(&source[*cursor..]) {
        *cursor += consumed;
        Some(Instruction::QuantifiedReference {
            index,
            named,
            bounds,
            copies: 1,
            captures: Box::new([]),
        })
    } else if named {
        Some(Instruction::NamedReference(index))
    } else {
        Some(Instruction::Reference(index))
    }
}

fn quantify_reference_wrapper(
    instructions: &mut [PreparedInstruction],
    body: Range<usize>,
    bounds: Bounds,
    named_slots: &HashMap<usize, usize>,
) -> Option<()> {
    let mut reference = None;
    let mut copies = 0usize;
    let mut targets = Vec::new();
    let mut captures = Vec::new();
    let mut opened = Vec::new();
    for (position, instruction) in instructions
        .iter()
        .enumerate()
        .take(body.end)
        .skip(body.start)
    {
        let target = match instruction {
            PreparedInstruction::Ready(Instruction::Nop) => continue,
            PreparedInstruction::Ready(Instruction::Open(slot)) => {
                opened.push((*slot, copies));
                continue;
            }
            PreparedInstruction::Ready(Instruction::Close(slot)) => {
                let (open, start) = opened.pop()?;
                if open != *slot {
                    return None;
                }
                captures.push((*slot, start, copies));
                continue;
            }
            PreparedInstruction::Ready(Instruction::Reference(index)) => ReferenceTarget::Input {
                index: *index,
                named: false,
            },
            PreparedInstruction::Ready(Instruction::NamedReference(index)) => {
                ReferenceTarget::Input {
                    index: *index,
                    named: true,
                }
            }
            PreparedInstruction::Ready(Instruction::Character(_) | Instruction::Assert(_))
            | PreparedInstruction::Set {
                repeated: false, ..
            } => ReferenceTarget::Term(position),
            _ => return None,
        };
        if let ReferenceTarget::Input { index, named } = target {
            if reference.is_none() {
                reference = Some((position, index, named));
            }
        }
        targets.push(target);
        copies = copies.checked_add(1)?;
    }
    if !opened.is_empty() {
        return None;
    }
    let position = reference.map_or(body.start, |(position, _, _)| position);
    let mut local = HashMap::new();
    for &(slot, start, end) in &captures {
        let span = ReferenceCaptureSpan { start, end };
        local.insert((slot, false), span);
        if let Some(&group) = named_slots.get(&slot) {
            local.insert((group, true), span);
        }
    }
    let targets: Vec<_> = targets
        .into_iter()
        .enumerate()
        .map(|(ordinal, target)| {
            if let ReferenceTarget::Input { index, named } = target {
                if let Some(&span) = local.get(&(index, named)) {
                    if span.end <= ordinal {
                        ReferenceTarget::Local(span)
                    } else {
                        ReferenceTarget::Empty
                    }
                } else {
                    target
                }
            } else {
                target
            }
        })
        .collect();
    let captures: Vec<_> = captures
        .into_iter()
        .map(|(slot, start, end)| (slot, ReferenceCaptureSpan { start, end }))
        .collect();
    let single = reference.filter(|&(_, index, named)| {
        targets
            .iter()
            .all(|target| *target == (ReferenceTarget::Input { index, named }))
    });
    if single.is_none() {
        for instruction in &mut instructions[body] {
            match instruction {
                PreparedInstruction::Ready(Instruction::Character(unit)) => {
                    *instruction =
                        PreparedInstruction::Ready(Instruction::RepeatedCharacter(*unit));
                }
                PreparedInstruction::Ready(Instruction::Assert(assertion)) => {
                    *instruction =
                        PreparedInstruction::Ready(Instruction::RepeatedAssert(*assertion));
                }
                PreparedInstruction::Set { repeated, .. } => *repeated = true,
                _ => {}
            }
            if matches!(
                instruction,
                PreparedInstruction::Ready(
                    Instruction::Reference(_)
                        | Instruction::NamedReference(_)
                        | Instruction::Open(_)
                        | Instruction::Close(_)
                )
            ) {
                *instruction = PreparedInstruction::Ready(Instruction::Nop);
            }
        }
        instructions[position] =
            PreparedInstruction::Ready(Instruction::QuantifiedReferenceSequence {
                references: targets.into_boxed_slice(),
                bounds,
                captures: captures.into_boxed_slice(),
            });
        return Some(());
    }
    let (_, index, named) = single?;
    for instruction in &mut instructions[body] {
        if matches!(
            instruction,
            PreparedInstruction::Ready(
                Instruction::Open(_)
                    | Instruction::Close(_)
                    | Instruction::Reference(_)
                    | Instruction::NamedReference(_)
            )
        ) {
            *instruction = PreparedInstruction::Ready(Instruction::Nop);
        }
    }
    instructions[position] = PreparedInstruction::Ready(Instruction::QuantifiedReference {
        index,
        named,
        bounds,
        copies,
        captures: captures.into_boxed_slice(),
    });
    Some(())
}

fn quantify_progressing_choice(
    instructions: &mut Vec<PreparedInstruction>,
    body: Range<usize>,
    bounds: Bounds,
    progresses: &mut Vec<bool>,
) -> Option<()> {
    if !matches!(
        instructions[body.start],
        PreparedInstruction::Ready(Instruction::Nop | Instruction::Choice(_))
    ) {
        return None;
    }
    // Treat an already proven child loop as one capture-free operation. Its
    // required iterations consume input; optional iterations use the suffix.
    // Reuse preparation storage instead of rescanning or clearing child bodies.
    progresses.resize(instructions.len() + 1, false);
    progresses[body.end] = false;
    let mut next = body.end;
    while next > body.start {
        let pc = next - 1;
        if pc > body.start {
            if let PreparedInstruction::Ready(Instruction::RepeatChoice {
                body: child,
                head,
                end,
                bounds: (min, _, _),
            }) = &instructions[pc - 1]
            {
                if *head != pc || *end != pc + 1 || *child < body.start || *child >= pc - 1 {
                    return None;
                }
                progresses[*child] = *min != Some(0) || progresses[*end];
                next = *child;
                continue;
            }
        }
        next = pc;
        progresses[pc] =
            match &instructions[pc] {
                PreparedInstruction::Ready(Instruction::Character(_))
                | PreparedInstruction::Set {
                    repeated: false, ..
                } => true,
                PreparedInstruction::Ready(
                    Instruction::Nop
                    | Instruction::Assert(_)
                    | Instruction::Reference(_)
                    | Instruction::NamedReference(_)
                    | Instruction::RepeatedCharacter(_)
                    | Instruction::RepeatedSet(_)
                    | Instruction::RepeatedAssert(_),
                )
                | PreparedInstruction::Set { repeated: true, .. } => progresses[pc + 1],
                PreparedInstruction::Ready(Instruction::QuantifiedReferenceSequence {
                    references,
                    bounds: (min, _, _),
                    captures,
                }) if captures.is_empty() => {
                    // Only required consuming terms establish progress. Outside
                    // references and assertions can match without consuming input.
                    let consuming = references.iter().any(|target| {
                        let ReferenceTarget::Term(position) = target else {
                            return false;
                        };
                        matches!(
                            instructions.get(*position),
                            Some(PreparedInstruction::Ready(Instruction::RepeatedCharacter(
                                _
                            ))) | Some(PreparedInstruction::Set { repeated: true, .. })
                        )
                    });
                    (*min != Some(0) && consuming) || progresses[pc + 1]
                }
                PreparedInstruction::Ready(Instruction::QuantifiedReference {
                    captures, ..
                }) if captures.is_empty() => progresses[pc + 1],
                PreparedInstruction::Ready(Instruction::Jump(target)) => {
                    if *target <= pc || *target > body.end {
                        return None;
                    }
                    progresses[*target]
                }
                PreparedInstruction::Ready(Instruction::Choice(branches)) => {
                    if branches.is_empty()
                        || branches
                            .iter()
                            .any(|&target| target <= pc || target > body.end)
                    {
                        return None;
                    }
                    branches.iter().all(|&target| progresses[target])
                }
                _ => return None,
            };
    }
    if !progresses[body.start] {
        return None;
    }
    let original_is_choice = matches!(
        instructions[body.start],
        PreparedInstruction::Ready(Instruction::Choice(_))
    );
    // Move the entry choice to the tail without moving any original branch
    // instruction or invalidating enclosing groups' original indices.
    let entry = instructions.len() + 1;
    let original = std::mem::replace(
        &mut instructions[body.start],
        PreparedInstruction::Ready(Instruction::Jump(entry)),
    );
    instructions.push(PreparedInstruction::Ready(Instruction::RepeatChoiceEnd(
        entry,
    )));
    instructions.push(PreparedInstruction::Ready(Instruction::RepeatChoice {
        body: body.start,
        head: entry + 1,
        end: entry + 2,
        bounds,
    }));
    instructions.push(if original_is_choice {
        original
    } else {
        PreparedInstruction::Ready(Instruction::Jump(body.start + 1))
    });
    Some(())
}

fn prepare(
    source: &[u16],
    ignore_case: bool,
    dot_all: bool,
    bindings: RegExpBackreferenceNamedBindings<'_>,
) -> Option<PreparedProgram> {
    let mut named_slots = HashMap::new();
    for (group, slots) in bindings.groups.iter().enumerate() {
        if slots.is_empty() || slots.windows(2).any(|p| p[0] >= p[1]) {
            return None;
        }
        for &slot in *slots {
            if named_slots.insert(slot, group).is_some() {
                return None;
            }
        }
    }
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
    let mut progresses = Vec::new();
    let mut last_choice_repeat = None;
    let mut groups = Vec::<GroupFrame>::new();
    let mut current = GroupFrame::open(None, &mut instructions);
    let mut scopes = Vec::new();
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
            cursor = reference.escape.end;
            instructions.push(PreparedInstruction::Ready(reference_instruction(
                source,
                &mut cursor,
                reference.group,
                true,
            )?));
            references += 1;
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
                    scopes.push(ScopeStep::Capture(index));
                    Some(index)
                };
                scopes.push(ScopeStep::Open);
                groups.push(std::mem::replace(
                    &mut current,
                    GroupFrame::open(group, &mut instructions),
                ));
            }
            0x29 => {
                let parent = groups.pop()?;
                let group = std::mem::replace(&mut current, parent);
                let entry = group.entry;
                let capture = group.finish(&mut instructions);
                scopes.push(ScopeStep::Close);
                if let Some(index) = capture {
                    instructions.push(PreparedInstruction::Ready(Instruction::Close(index)));
                }
                if let Some((bounds, consumed)) = quantifier(&source[cursor..]) {
                    let end = instructions.len();
                    let start = if capture.is_some() { entry - 1 } else { entry };
                    if last_choice_repeat.is_some_and(|pc| pc >= start)
                        || quantify_reference_wrapper(
                            &mut instructions,
                            start..end,
                            bounds,
                            &named_slots,
                        )
                        .is_none()
                    {
                        quantify_progressing_choice(
                            &mut instructions,
                            start..end,
                            bounds,
                            &mut progresses,
                        )?;
                        last_choice_repeat = Some(instructions.len() - 2);
                    }
                    cursor += consumed;
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
                instructions.push(PreparedInstruction::Ready(reference_instruction(
                    source,
                    &mut cursor,
                    number.checked_sub(1)?,
                    false,
                )?));
                references += 1;
            }
            0x7c => {
                scopes.push(ScopeStep::Alternative);
                current.next_alternative(&mut instructions);
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
                    repeated: false,
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
                    repeated: false,
                });
            }
            0x5c if !bindings.groups.is_empty() && source.get(cursor) == Some(&0x6b) => {
                return None;
            }
            0x5c => instructions.push(PreparedInstruction::Ready(Instruction::Character(
                canonicalize(character_escape(source, &mut cursor)?, ignore_case),
            ))),
            _ if is_syntax(unit) || unit == 0x7c => return None,
            _ => instructions.push(PreparedInstruction::Ready(Instruction::Character(
                canonicalize(unit, ignore_case),
            ))),
        }
        if matches!(
            instructions.last(),
            Some(
                PreparedInstruction::Ready(Instruction::Character(_))
                    | PreparedInstruction::Set {
                        repeated: false,
                        ..
                    }
            )
        ) {
            if let Some((bounds, consumed)) = quantifier(&source[cursor..]) {
                // Keep the unit separate from its owning repetition entry.
                // Existing branch/group indices precede this last instruction.
                let unit = instructions.pop()?;
                let start = instructions.len();
                instructions.push(PreparedInstruction::Ready(Instruction::Nop));
                instructions.push(unit);
                let end = instructions.len();
                quantify_reference_wrapper(&mut instructions, start..end, bounds, &named_slots)?;
                cursor += consumed;
            }
        }
    }
    if !groups.is_empty() || named_references.peek().is_some() {
        return None;
    }
    if instructions.iter().any(|instruction| {
        match instruction {
            PreparedInstruction::Ready(Instruction::QuantifiedReferenceSequence {references,..}) => references.iter().any(|target| matches!(target, ReferenceTarget::Input {index,named:false} if *index>=capture_count)),
            _ => matches!(instruction, PreparedInstruction::Ready(Instruction::Reference(index) | Instruction::QuantifiedReference { index, named: false, .. }) if *index >= capture_count),
        }
    }) {
        return None;
    }
    current.finish(&mut instructions);
    Some(PreparedProgram {
        instructions,
        capture_count,
        scopes,
        references,
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

    fn ordinary(
        source: &str,
        ignore_case: bool,
        multiline: bool,
        dot_all: bool,
    ) -> RegExpBackreferenceMatcher {
        RegExpBackreferenceMatcher::compile_ordinary_with_named_bindings_and_work(
            &JsString::from(source),
            ignore_case,
            multiline,
            dot_all,
            RegExpBackreferenceNamedBindings::default(),
            |_| Ok::<_, ()>(()),
        )
        .unwrap()
        .unwrap()
    }

    #[test]
    fn nested_choice_loop_execution_snapshot() {
        let mut rows = String::new();
        for source in [
            r"(?:(?:ab|a)+b|c)+d",
            r"(?:(?:ab|a)+?b|c)+?d",
            r"(?:(?:a|b)*c|d)+e",
            r"(?:(?:a|b)*?c|d)+?e",
            r"(?:(?:a|b)+)+c",
            r"(?:(?:a|b)+?)+?c",
            r"(?:(?:a|b)?c|d)+e",
            r"(?:(?:a|b)??c|d)+?e",
            r"(?:(?:a|b){2,3}c|d){2,3}e",
            r"(?:(?:a|b){2,3}?c|d){2,3}?e",
            r"((?:(?:ab|a)+b|c)+)d",
            r"((?:(?:ab|a)+?b|c)+?)d",
            r"(a)(?:(?:\1a|b)+c|b)+\1",
            r"(a)(?:(?:\1a|b)+?c|b)+?\1",
            r"(?:(?:[ab]|c)+d|e)+f",
            r"(?:(?:.|a)+b|a)+c",
            r"(?:(?:^a|b)+c|d)+",
            r"(?:(?:\ba|b)+c|d)+",
            r"(?:(?:a|b){999999999999999999999999999999}|c)+d",
            r"(?:(?:a|b){0,999999999999999999999999999999}c|d)+e",
            r"(?:(?:a|b)+c|d)*e|a",
            r"(?:(?:a|b)+c|d)+(?:a|b)+e",
            r"(?:(?:a|b)+c|d)+?(?:a|b)+?e",
            r"(?:(?:(?:a|b)+c|d)+e|f)+g",
            r"(?:(?:a+b|c)+d|e)+f",
            r"(?:(?:(?:a|)b)+c|d)+e",
        ] {
            for (ignore_case, multiline, dot_all) in [
                (false, false, false),
                (true, true, false),
                (false, false, true),
                (true, false, true),
            ] {
                let matcher = ordinary(source, ignore_case, multiline, dot_all);
                for text in [
                    "", "a", "b", "ab", "abb", "abbc", "abab", "ababc", "bbc", "aaba", "abbaa",
                    "abcc", "\n\n", " b ", "µΜ",
                ] {
                    let text = JsString::from(text);
                    for (start, sticky) in [(0, false), (0, true), (1, true)] {
                        writeln!(rows,"{source:?} i={ignore_case} m={multiline} s={dot_all} input={text:?} start={start} sticky={sticky} {:?}",matcher.find(&text,start,sticky)).unwrap();
                    }
                }
            }
        }
        insta::assert_snapshot!(rows);
    }

    #[test]
    fn child_loop_exits_and_retries_restore_parent_contexts_and_capture_closes() {
        for (source, text, range, captures) in [
            (r"((?:(?:ab|a)+b|c)+)d", "aabccd", 0..6, vec![Some(0..5)]),
            (r"(?:(?:a|b)+)+c", "abbbc", 0..5, vec![]),
            (r"(a)(?:(?:\1a|b)+c|b)+\1", "aaabca", 0..6, vec![Some(0..1)]),
            (r"(?:(?:a|b)+c|d)*e|a", "a", 0..1, vec![]),
        ] {
            let found = ordinary(source, false, false, false)
                .find(&JsString::from(text), 0, true)
                .unwrap();
            assert_eq!(found.range, range, "{source}");
            assert_eq!(&*found.captures, &*captures, "{source}");
        }
    }

    #[test]
    fn deeply_nested_child_loop_summaries_and_contexts_remain_flat_and_fallible() {
        let source = "(?:".repeat(100000) + "(?:a|b)" + &")+".repeat(100000);
        let source = JsString::from(source.as_str());
        let mut work = 0;
        let matcher = RegExpBackreferenceMatcher::compile_ordinary_with_named_bindings_and_work(
            &source,
            false,
            false,
            false,
            RegExpBackreferenceNamedBindings::default(),
            |n| {
                work += n;
                Ok::<_, ()>(())
            },
        )
        .unwrap()
        .unwrap();
        assert!(work < source.len() * 18 + 131072, "work {work}");
        assert_eq!(
            matcher.find(&JsString::from("a"), 0, true).unwrap().range,
            0..1
        );
        assert!(
            matcher
                .find_with_work(&JsString::from("a"), 0, true, |_| Err::<(), _>("work"))
                .is_err()
        );
        for source in [
            r"(?:(?:a|b)*)+",
            r"(?:(?:(a)|b)+c|d)+",
            r"(?:(?:\1|b)+c|d)+(a)",
            r"(?:(?=a)a|b)+",
        ] {
            let mut work = 0;
            assert!(
                RegExpBackreferenceMatcher::compile_ordinary_with_named_bindings_and_work(
                    &JsString::from(source),
                    false,
                    false,
                    false,
                    RegExpBackreferenceNamedBindings::default(),
                    |n| {
                        work += n;
                        Ok::<_, ()>(())
                    }
                )
                .unwrap()
                .is_none(),
                "{source}"
            );
            assert_eq!(work, 0, "{source}");
        }
    }

    #[test]
    fn quantified_progressing_body_execution_snapshot() {
        let mut rows = String::new();
        for source in [
            r"(?:a+b|c)+d",
            r"(?:a+?b|c)+?d",
            r"(?:a*b|c)+d",
            r"(?:a*?b|c)+?d",
            r"(?:a{2,3}b|c){2,3}d",
            r"(?:a{2,3}?b|c){2,3}?d",
            r"(?:a?b|c)+d",
            r"(?:a??b|c)+d",
            r"(?:(?:ab)+c|d)+e",
            r"(?:(?:ab)+?c|d)+?e",
            r"(?:(?:ab)*c|d)+e",
            r"(?:(?:ab)*?c|d)+?e",
            r"((?:a+b|c)+)d",
            r"((?:a+?b|c)+?)d",
            r"(a)(?:\1*b|c)+\1",
            r"(a)(?:\1*?b|c)+?\1",
            r"(?:(?:a|)b+)+c",
            r"(?:(?:a|)b+?)+?c",
            r"(?:[ab]+c|d)+e",
            r"(?:.+b|a)+c",
            r"(?:(?:^a)+b|c)+",
            r"(?:(?:\ba)+b|c)+",
            r"(?:a{999999999999999999999999999999}|b)+c",
            r"(?:a+b|c)*d|a",
            r"(?:a+b+){2}c",
            r"(?:a+?b+?){2}?c",
        ] {
            for (ignore_case, multiline, dot_all) in [
                (false, false, false),
                (true, true, false),
                (false, false, true),
                (true, false, true),
            ] {
                let matcher = ordinary(source, ignore_case, multiline, dot_all);
                for text in [
                    "", "a", "b", "ab", "abb", "abbc", "abab", "ababc", "bbc", "aaba", "abbaa",
                    "abcc", "\n\n", " b ", "µΜ",
                ] {
                    let text = JsString::from(text);
                    for (start, sticky) in [(0, false), (0, true), (1, true)] {
                        writeln!(rows, "{source:?} i={ignore_case} m={multiline} s={dot_all} input={text:?} start={start} sticky={sticky} {:?}", matcher.find(&text, start, sticky)).unwrap();
                    }
                }
            }
        }
        insta::assert_snapshot!(rows);
    }

    #[test]
    fn inner_counts_retry_before_outer_iterations_and_parent_closes() {
        for (source, text, range, captures) in [
            (r"((?:a+b|c)+)d", "aabccd", 0..6, vec![Some(0..5)]),
            (r"(?:(?:a|)b+)+c", "abbbc", 0..5, vec![]),
            (r"(a)(?:\1*b|c)+\1", "aaabca", 0..6, vec![Some(0..1)]),
            (r"(?:a+b|c)*d|a", "a", 0..1, vec![]),
        ] {
            let found = ordinary(source, false, false, false)
                .find(&JsString::from(text), 0, true)
                .unwrap();
            assert_eq!(found.range, range, "{source}");
            assert_eq!(&*found.captures, &*captures, "{source}");
        }
    }

    #[test]
    fn deterministic_inner_quantifiers_keep_preparation_flat_and_reject_capture_effects() {
        let source = "(?:".repeat(100000) + "(?:a+b|c)" + &")".repeat(100000) + "+";
        assert_eq!(
            ordinary(&source, false, false, false)
                .find(&JsString::from("aabcc"), 0, true)
                .unwrap()
                .range,
            0..5
        );
        let source = "(?:".to_owned()
            + &std::iter::repeat_n("a+b", 10000)
                .collect::<Vec<_>>()
                .join("|")
            + ")+";
        let source = JsString::from(source.as_str());
        let mut work = 0;
        let matcher = RegExpBackreferenceMatcher::compile_ordinary_with_named_bindings_and_work(
            &source,
            false,
            false,
            false,
            RegExpBackreferenceNamedBindings::default(),
            |n| {
                work += n;
                Ok::<_, ()>(())
            },
        )
        .unwrap()
        .unwrap();
        assert!(work < source.len() * 18 + 131072, "work {work}");
        assert_eq!(
            matcher.find(&JsString::from("ab"), 0, true).unwrap().range,
            0..2
        );
        assert!(
            matcher
                .find_with_work(&JsString::from("ab"), 0, true, |_| Err::<(), _>("work"))
                .is_err()
        );
        for source in [
            r"(?:a*b*)+",
            r"(?:(a+)b|c)+",
            r"(?:(?:(a)|b)+c|d)+",
            r"(?:\1*|b)+(a)",
        ] {
            let mut work = 0;
            assert!(
                RegExpBackreferenceMatcher::compile_ordinary_with_named_bindings_and_work(
                    &JsString::from(source),
                    false,
                    false,
                    false,
                    RegExpBackreferenceNamedBindings::default(),
                    |n| {
                        work += n;
                        Ok::<_, ()>(())
                    }
                )
                .unwrap()
                .is_none(),
                "{source}"
            );
            assert_eq!(work, 0, "{source}");
        }
    }

    #[test]
    fn nested_progressing_choice_execution_snapshot() {
        let mut rows = String::new();
        for source in [
            r"(?:(?:a|ab)b)+c",
            r"(?:(?:ab|a)b)+c",
            r"(?:(?:a|ab)b)+?c",
            r"(?:(?:a|)b)+c",
            r"(?:(?:a|)b)*c",
            r"(?:(?:a|)b)*?c",
            r"(?:a(?:b|c)|d)+e",
            r"(?:a(?:b|c)|d)+?e",
            r"(?:(?:a|)(?:b|c))+d",
            r"(?:(?:a|)(?:b|c))+?d",
            r"((?:(?:a|)b)+)c",
            r"((?:(?:a|)b)+?)c",
            r"(a)(?:(?:\1|)b)+\1",
            r"(a)(?:(?:\1|)b)+?\1",
            r"(a)(?:b(?:\1|))*\1",
            r"(a)(?:b(?:\1|))*?\1",
            r"(?:(?:[ab]|)a|b)+c",
            r"(?:(?:.|)b)+c",
            r"(?:(?:^|a)b)+",
            r"(?:(?:\b|a)b)+",
            r"(?:(?:a|)b){2,3}c",
            r"(?:(?:a|)b){2,3}?c",
            r"(?:(?:a|)b){999999999999999999999999999999}",
            r"(?:(?:a|)b)*c|a",
        ] {
            for (ignore_case, multiline, dot_all) in [
                (false, false, false),
                (true, true, false),
                (false, false, true),
                (true, false, true),
            ] {
                let matcher = ordinary(source, ignore_case, multiline, dot_all);
                for text in [
                    "", "a", "b", "ab", "abb", "abbc", "abab", "ababc", "bbc", "aaba", "abbaa",
                    "abcc", "\n\n", " b ", "µΜ",
                ] {
                    let text = JsString::from(text);
                    for (start, sticky) in [(0, false), (0, true), (1, true)] {
                        writeln!(rows, "{source:?} i={ignore_case} m={multiline} s={dot_all} input={text:?} start={start} sticky={sticky} {:?}", matcher.find(&text, start, sticky)).unwrap();
                    }
                }
            }
        }
        insta::assert_snapshot!(rows);
    }

    #[test]
    fn nested_body_retries_restore_iteration_counts_and_common_prefix_ranges() {
        for (source, text, range, captures) in [
            (r"((?:(?:a|)b)+)c", "abbc", 0..4, vec![Some(0..3)]),
            (r"((?:(?:a|)b)+?)c", "abbc", 0..4, vec![Some(0..3)]),
            (r"(?:(?:a|ab)b)+c", "abbc", 0..4, vec![]),
            (r"(?:(?:a|)(?:b|c))+d", "abcd", 0..4, vec![]),
            (r"(a)(?:(?:\1|)b)+\1", "aaba", 0..4, vec![Some(0..1)]),
            (r"(?:(?:a|)b)*c|a", "a", 0..1, vec![]),
        ] {
            let found = ordinary(source, false, false, false)
                .find(&JsString::from(text), 0, true)
                .unwrap();
            assert_eq!(found.range, range, "{source}");
            assert_eq!(&*found.captures, &*captures, "{source}");
        }
    }

    #[test]
    fn whole_body_progress_checks_keep_deep_shared_paths_linear_and_fallible() {
        let source = "(?:".repeat(100000) + "(?:a|)b" + &")".repeat(100000) + "+";
        let matcher = ordinary(&source, false, false, false);
        assert_eq!(
            matcher.find(&JsString::from("abb"), 0, true).unwrap().range,
            0..3
        );
        let source = "(?:".to_owned()
            + &std::iter::repeat_n("(?:a|)b", 10000)
                .collect::<Vec<_>>()
                .join("|")
            + ")+";
        let mut work = 0;
        let source = JsString::from(source.as_str());
        let matcher = RegExpBackreferenceMatcher::compile_ordinary_with_named_bindings_and_work(
            &source,
            false,
            false,
            false,
            RegExpBackreferenceNamedBindings::default(),
            |n| {
                work += n;
                Ok::<_, ()>(())
            },
        )
        .unwrap()
        .unwrap();
        assert!(work < source.len() * 18 + 131072, "work {work}");
        assert_eq!(
            matcher.find(&JsString::from("b"), 0, true).unwrap().range,
            0..1
        );
        assert!(
            matcher
                .find_with_work(&JsString::from("b"), 0, true, |_| Err::<(), _>("work"))
                .is_err()
        );
        for source in [
            r"(?:(?:a|))*",
            r"(?:(?:a|)(?:b|))+",
            r"(?:(a|)b)+",
            r"(?:(?:a|)(b+))+",
        ] {
            let mut work = 0;
            assert!(
                RegExpBackreferenceMatcher::compile_ordinary_with_named_bindings_and_work(
                    &JsString::from(source),
                    false,
                    false,
                    false,
                    RegExpBackreferenceNamedBindings::default(),
                    |n| {
                        work += n;
                        Ok::<_, ()>(())
                    },
                )
                .unwrap()
                .is_none()
            );
            assert_eq!(work, 0, "{source}");
        }
    }

    #[test]
    fn progressing_choice_repetition_execution_snapshot() {
        let mut rows = String::new();
        for source in [
            r"(?:ab|a)+b",
            r"(?:a|ab)+b",
            r"(?:ab|a)+?b",
            r"(?:ab|a)*b",
            r"(?:ab|a)*?b",
            r"(?:ab|c){2,3}d",
            r"(?:ab|c){2,3}?d",
            r"(?:ab|c)?d",
            r"(?:ab|c)??d",
            r"((?:ab|c)+)d",
            r"((?:ab|c)+?)d",
            r"(a)(?:\1b|b)+\1",
            r"(a)(?:\1b|b)+?\1",
            r"(a)(?:b\1|b)*\1",
            r"(a)(?:b\1|b)*?\1",
            r"(?:(?:ab|c)+)",
            r"(?:ab|c)+(?:bc|a)+d",
            r"(?:ab|c)+?(?:bc|a)+?d",
            r"(?:[ab]a|b)+c",
            r"(?:.b|a)+c",
            r"(?:^a|b$)+",
            r"(?:\ba|b\B)+",
            r"(?:ab|c){999999999999999999999999999999}",
            r"(?:ab|c)*d|a",
        ] {
            for (ignore_case, multiline, dot_all) in [
                (false, false, false),
                (true, true, false),
                (false, false, true),
                (true, false, true),
            ] {
                let matcher = ordinary(source, ignore_case, multiline, dot_all);
                for text in [
                    "", "a", "b", "ab", "abb", "abab", "aaab", "ababd", "abccd", "ababc", "abbd",
                    "bccd", "\n\n", " a ", "µΜ",
                ] {
                    let text = JsString::from(text);
                    for (start, sticky) in [(0, false), (0, true), (1, true)] {
                        writeln!(rows, "{source:?} i={ignore_case} m={multiline} s={dot_all} input={text:?} start={start} sticky={sticky} {:?}", matcher.find(&text, start, sticky)).unwrap();
                    }
                }
            }
        }
        insta::assert_snapshot!(rows);
    }

    #[test]
    fn choice_iteration_retries_keep_source_order_counts_and_outer_captures() {
        for (source, text, range, captures) in [
            (r"(?:ab|a)+b", "ab", 0..2, vec![]),
            (r"(?:ab|a)+b", "abb", 0..3, vec![]),
            (r"(?:a|ab)+b", "abb", 0..2, vec![]),
            (r"(?:ab|a)+?b", "ababb", 0..5, vec![]),
            (r"((?:ab|c)+)d", "abccd", 0..5, vec![Some(0..4)]),
            (r"(a)(?:\1b|b)+\1", "aaba", 0..4, vec![Some(0..1)]),
            (r"(?:ab|c)*d|a", "a", 0..1, vec![]),
            (r"(?:ab|c)+(?:ab|c)+", "abc", 0..3, vec![]),
            (r"((?:\1a|b)+)\1", "abab", 0..4, vec![Some(0..2)]),
            (
                r"(a)(?:\2b|a)+(\1)",
                "abbaa",
                0..5,
                vec![Some(0..1), Some(4..5)],
            ),
        ] {
            let found = ordinary(source, false, false, false)
                .find(&JsString::from(text), 0, true)
                .unwrap();
            assert_eq!(found.range, range, "{source}");
            assert_eq!(&*found.captures, &*captures, "{source}");
        }
    }

    #[test]
    fn progressing_choice_counts_and_enclosures_remain_flat_and_opt_in() {
        let source = "(".repeat(100000) + "(?:ab|c)+" + &")".repeat(100000);
        let matcher = ordinary(&source, false, false, false);
        let found = matcher.find(&JsString::from("abc"), 0, true).unwrap();
        assert_eq!(found.range, 0..3);
        assert!(found.captures.iter().all(|r| *r == Some(0..3)));
        let matcher = ordinary(r"(?:ab|a)+b", false, false, false);
        let input = JsString::from(("a".repeat(50000) + "b").as_str());
        assert_eq!(matcher.find(&input, 0, true).unwrap().range, 0..50001);
        assert!(
            matcher
                .find_with_work(&input, 0, false, |_| Err::<(), _>("work"))
                .is_err()
        );
        let n = "9".repeat(10000);
        let source = format!("(?:ab|c){{{n}}}");
        assert!(
            ordinary(&source, false, false, false)
                .find(&JsString::from("abc"), 0, true)
                .is_none()
        );
        let source = format!("(?:ab|c){{0,{n}}}");
        assert_eq!(
            ordinary(&source, false, false, false)
                .find(&JsString::from("q"), 0, true)
                .unwrap()
                .range,
            0..0
        );
        for source in [
            r"(?:ab|)+",
            r"(?:\1|b)+(a)",
            r"((a)|(b))+",
            r"(?:(?:(ab)|c)+){2}",
        ] {
            let mut work = 0;
            assert!(
                RegExpBackreferenceMatcher::compile_ordinary_with_named_bindings_and_work(
                    &JsString::from(source),
                    false,
                    false,
                    false,
                    RegExpBackreferenceNamedBindings::default(),
                    |n| {
                        work += n;
                        Ok::<_, ()>(())
                    },
                )
                .unwrap()
                .is_none()
            );
            assert_eq!(work, 0, "{source}");
        }
    }

    #[test]
    fn ordinary_fallback_execution_snapshot() {
        let mut rows = String::new();
        for source in [
            r"((1)|(12))((3)|(23))",
            r"(a|ab)b",
            r"(ab|a)b",
            r"((a|ab)(b|))",
            r"(a|)(b|a)",
            r"(?:ab|cd)\d?",
            r"(?:(a)|(b))(c|)",
            r"(a+)(b+)",
            r"(a+?)(b+?)",
            r"([ab]*)c",
            r"([ab]*?)c",
            r"((a+)(b*))",
            r"(?:(a*)b(c*))",
            r"^(a|ab)b$",
            r"\b(a+)\b",
            r"(.+)(.)",
            r"(.+?)(.)",
            r"(a?)",
            r"([ab]?)",
            r"()?",
            r"()+",
            r"(){2,3}",
            r"(a){1,3}",
            r"([ab]{0,999999999999999999999999999999})",
        ] {
            for (ignore_case, multiline, dot_all) in [
                (false, false, false),
                (true, true, false),
                (false, false, true),
                (true, false, true),
            ] {
                let matcher = ordinary(source, ignore_case, multiline, dot_all);
                for text in [
                    "", "a", "b", "ab", "abb", "aabb", "aaaaa", "aaabbbb", "123", "cd2", "aacc",
                    "\n\n", " a ", "\u{D7FF}", "µΜ",
                ] {
                    let text = JsString::from(text);
                    for (start, sticky) in [(0, false), (0, true), (1, true)] {
                        writeln!(rows, "{source:?} i={ignore_case} m={multiline} s={dot_all} input={text:?} start={start} sticky={sticky} {:?}", matcher.find(&text, start, sticky)).unwrap();
                    }
                }
            }
        }
        insta::assert_snapshot!(rows);
    }

    #[test]
    fn ordinary_choices_preserve_source_order_inactive_slots_and_whole_prefixes() {
        for (source, text, range, captures) in [
            (
                r"((1)|(12))((3)|(23))",
                "123",
                0..3,
                vec![Some(0..1), Some(0..1), None, Some(1..3), None, Some(1..3)],
            ),
            (r"(a|ab)b", "abb", 0..2, vec![Some(0..1)]),
            (r"(ab|a)b", "abb", 0..3, vec![Some(0..2)]),
            (
                r"((a|ab)(b|))",
                "ab",
                0..2,
                vec![Some(0..2), Some(0..1), Some(1..2)],
            ),
            (r"(a+)(b+)", "aaabbbb", 0..7, vec![Some(0..3), Some(3..7)]),
            (r"(a+?)(b+?)", "aaabbbb", 0..4, vec![Some(0..3), Some(3..4)]),
            (r"(a?)", "q", 0..0, vec![Some(0..0)]),
            (r"()?", "q", 0..0, vec![None]),
        ] {
            let found = ordinary(source, false, false, false)
                .find(&JsString::from(text), 0, true)
                .unwrap();
            assert_eq!(found.range, range, "{source}");
            assert_eq!(&*found.captures, &*captures, "{source}");
        }
        // Existing constructors retain their reference-only contract, including
        // decimal-looking escapes within character classes.
        for source in ["a", "(a+)", r"[\1]"] {
            assert!(RegExpBackreferenceMatcher::compile(&JsString::from(source), false).is_none());
        }
    }

    #[test]
    fn ordinary_fallback_keeps_deep_captures_flat_and_rejects_gaps_before_work() {
        let source = "(".repeat(100000) + "a+" + &")".repeat(100000);
        let matcher = ordinary(&source, false, false, false);
        let found = matcher.find(&JsString::from("aaa"), 0, true).unwrap();
        assert_eq!(found.range, 0..3);
        assert!(found.captures.iter().all(|r| *r == Some(0..3)));
        let source = JsString::from(source.as_str());
        assert!(
            RegExpBackreferenceMatcher::compile_ordinary_with_named_bindings_and_work(
                &source,
                false,
                false,
                false,
                RegExpBackreferenceNamedBindings::default(),
                |_| Err::<(), _>("work"),
            )
            .is_err()
        );
        for source in [r"(a|b)+", r"(a+)+", r"(?=a)a", r"(?i:a)"] {
            let mut work = 0;
            assert!(
                RegExpBackreferenceMatcher::compile_ordinary_with_named_bindings_and_work(
                    &JsString::from(source),
                    false,
                    false,
                    false,
                    RegExpBackreferenceNamedBindings::default(),
                    |n| {
                        work += n;
                        Ok::<_, ()>(())
                    },
                )
                .unwrap()
                .is_none()
            );
            assert_eq!(work, 0, "{source}");
        }
        let source = JsString::from("(a)|(b)");
        let groups = [&[0usize, 1][..]];
        let matcher = RegExpBackreferenceMatcher::compile_ordinary_with_named_bindings_and_work(
            &source,
            false,
            false,
            false,
            RegExpBackreferenceNamedBindings {
                groups: &groups,
                references: &[],
            },
            |_| Ok::<_, ()>(()),
        )
        .unwrap()
        .unwrap();
        assert_eq!(
            &*matcher
                .find(&JsString::from("b"), 0, true)
                .unwrap()
                .captures,
            &[None, Some(0..1)]
        );
        let source = JsString::from("(a)(b)");
        let mut work = 0;
        assert!(
            RegExpBackreferenceMatcher::compile_ordinary_with_named_bindings_and_work(
                &source,
                false,
                false,
                false,
                RegExpBackreferenceNamedBindings {
                    groups: &groups,
                    references: &[]
                },
                |n| {
                    work += n;
                    Ok::<_, ()>(())
                },
            )
            .unwrap()
            .is_none()
        );
        assert_eq!(work, 0);
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
            (r"(?:(a)(?:\k<x>)+){2}", vec![0], 9..14, 0),
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
        assert!(matcher.0.instructions.len() <= source.len() * 2);
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
    fn quantified_reference_character_execution_snapshot() {
        let mut rows = String::new();
        for source in [
            r"(a+)\1",
            r"(a+?)\1",
            r"([ab]*)\1c",
            r"([ab]*?)\1c",
            r"(a{1,3})\1",
            r"([ab]{1,3}?)\1",
            r"(?:a+)(b+)\1",
            r"(?:a+?)(b+?)\1",
            r"()(a*)\1\2",
            r"()(a*?)\1\2",
            r"\1(a+)",
            r"\1(a+?)",
            r"((a+)(b+))\1",
            r"((a+?)(b+?))\1",
            r"(a)(b*)\1\2",
            r"(a)(b*?)\1\2",
            r"(a?)\1",
            r"([ab]?)\1",
            r"(?:(a*)b(c*))\1\2",
            r"([ab]{999999999999999999999999999999})\1",
            r"([ab]{0,999999999999999999999999999999}?)\1c",
            r"^(\w+)\1$",
            r"\b(\d+\w*)\1\b",
            r"(.+)\1|()?\2",
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
                    "ab",
                    "abab",
                    "ababab",
                    "abb",
                    "abbabba",
                    "abbaabb",
                    "abababc",
                    "abbbbbba",
                    "aabb",
                    "aAbB",
                    "µaΜAµa",
                    "\n\n",
                    " ababab ",
                    "\u{2028}ab\u{2029}",
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
    fn quantified_units_restore_outer_prefix_ranges_before_following_references() {
        for (source, input, range, captures) in [
            (r"(a+)\1", "aaaaa", 0..4, vec![Some(0..2)]),
            (r"(a+?)\1", "aaaaa", 0..2, vec![Some(0..1)]),
            (r"([ab]*)\1c", "aacc", 0..3, vec![Some(0..1)]),
            (r"([ab]*)\1c", "c", 0..1, vec![Some(0..0)]),
            (r"(?:a+)(b+)\1", "aaabbbb", 0..7, vec![Some(3..5)]),
            (r"(?:a+?)(b+?)\1", "aaabbbb", 0..5, vec![Some(3..4)]),
            (r"(a?)\1", "q", 0..0, vec![Some(0..0)]),
            (r"\1(a+)", "aaa", 0..3, vec![Some(0..3)]),
        ] {
            let matcher =
                RegExpBackreferenceMatcher::compile(&JsString::from(source), false).unwrap();
            let found = matcher.find(&JsString::from(input), 0, true).unwrap();
            assert_eq!(found.range, range, "{source}");
            assert_eq!(&*found.captures, &*captures, "{source}");
        }
        let matcher = RegExpBackreferenceMatcher::compile_with_flags_and_work(
            &JsString::from(r"(.+)\1"),
            false,
            true,
            |_| Ok::<_, ()>(()),
        )
        .unwrap()
        .unwrap();
        let found = matcher.find(&JsString::from("\n\n"), 0, true).unwrap();
        assert_eq!(found.range, 0..2);
        assert_eq!(found.captures[0], Some(0..1));
    }

    #[test]
    fn deep_unit_quantifiers_shared_predicates_and_huge_bounds_keep_safe_entries() {
        let source =
            JsString::from(format!("{}a+{}\\1", "(".repeat(100000), ")".repeat(100000)).as_str());
        let matcher = RegExpBackreferenceMatcher::compile(&source, false).unwrap();
        let found = matcher.find(&JsString::from("aaaaa"), 0, true).unwrap();
        assert_eq!(found.range, 0..4);
        assert!(found.captures.iter().all(|r| *r == Some(0..2)));
        let source = JsString::from(format!("{}\\1", r"([ab]?)".repeat(10000)).as_str());
        let mut work = 0;
        let matcher = RegExpBackreferenceMatcher::compile_with_work(&source, false, |n| {
            work += n;
            Ok::<_, ()>(())
        })
        .unwrap()
        .unwrap();
        assert!(
            work < source.len() * 18 + 131072,
            "shared optional set work {work}"
        );
        let found = matcher.find(&JsString::from(""), 0, true).unwrap();
        assert_eq!(found.captures.len(), 10000);
        assert!(found.captures.iter().all(|r| *r == Some(0..0)));
        for (prefix, expected) in [("0,", true), ("", false)] {
            let source =
                JsString::from(format!("(a{{{prefix}{}}})\\1", "9".repeat(10000)).as_str());
            let matcher = RegExpBackreferenceMatcher::compile(&source, false).unwrap();
            let found = matcher.find(&JsString::from(""), 0, true);
            assert_eq!(found.is_some(), expected);
            if let Some(found) = found {
                assert_eq!(found.captures[0], Some(0..0));
            }
        }
        assert!(matches!(
            matcher.find_with_work(&JsString::from(""), 0, true, |_| Err::<(), _>("work")),
            Err("work")
        ));
        for source in [r"((a+)+)\1", r"(a|b)+\1", r"((?=a)a+)\1", r"((?i:a)+)\1"] {
            let mut work = 0;
            assert!(
                RegExpBackreferenceMatcher::compile_with_work(
                    &JsString::from(source),
                    false,
                    |n| {
                        work += n;
                        Ok::<_, ()>(())
                    }
                )
                .unwrap()
                .is_none(),
                "{source}"
            );
            assert_eq!(work, 0, "{source}");
        }
    }

    #[test]
    fn reference_free_repetition_execution_snapshot() {
        let mut rows = String::new();
        for source in [
            r"(a)+\1",
            r"(a)+?\1",
            r"([ab])*\1c",
            r"([ab])*?\1c",
            r"(a){1,3}\1",
            r"([ab]){1,3}?\1",
            r"(?:(a)(b)){1,3}\1\2",
            r"(?:(a)(b)){1,3}?\1\2",
            r"()(a)*\1\2",
            r"()(a)*?\1\2",
            r"\1(a)+",
            r"\1(a)+?",
            r"((a)(b)+)\1",
            r"((a)(b)+?)\1",
            r"(a)(b)*\1\2",
            r"(a)(b)*?\1\2",
            r"()?\1",
            r"()+\1",
            r"(){2,3}\1",
            r"([ab]){999999999999999999999999999999}\1",
            r"([ab]){0,999999999999999999999999999999}?\1c",
            r"(\b)+\1",
            r"((^)){2}\2",
            r"(a)+\1|()?\2",
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
                    "ab",
                    "abab",
                    "ababab",
                    "abb",
                    "abbabba",
                    "abbaabb",
                    "abababc",
                    "abbbbbba",
                    "aabb",
                    "aAbB",
                    "µaΜAµa",
                    "\n\n",
                    " ababab ",
                    "\u{2028}ab\u{2029}",
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
    fn reference_free_repetition_retries_restore_final_partial_and_empty_captures() {
        for (source, input, range, captures) in [
            (r"(a)+\1", "aaaaa", 0..5, vec![Some(3..4)]),
            (r"(a)+?\1", "aaaaa", 0..2, vec![Some(0..1)]),
            (r"([ab])+\1", "abba", 0..3, vec![Some(1..2)]),
            (
                r"(?:(a)(b)){1,3}\1\2",
                "ababab",
                0..6,
                vec![Some(2..3), Some(3..4)],
            ),
            (
                r"((a)(b)+)\1",
                "abbabb",
                0..6,
                vec![Some(0..3), Some(0..1), Some(2..3)],
            ),
            (r"\1(a)+", "aaa", 0..3, vec![Some(2..3)]),
            (r"()?\1", "q", 0..0, vec![None]),
            (r"()+\1", "q", 0..0, vec![Some(0..0)]),
            (r"(){2,3}\1", "q", 0..0, vec![Some(0..0)]),
            (r"((^)){2}\2", "q", 0..0, vec![Some(0..0), Some(0..0)]),
        ] {
            let matcher =
                RegExpBackreferenceMatcher::compile(&JsString::from(source), false).unwrap();
            let found = matcher.find(&JsString::from(input), 0, true).unwrap();
            assert_eq!(found.range, range, "{source}");
            assert_eq!(&*found.captures, &*captures, "{source}");
        }
        let matcher =
            RegExpBackreferenceMatcher::compile(&JsString::from(r"(\b)+\1"), false).unwrap();
        assert_eq!(
            matcher
                .find(&JsString::from("x"), 1, true)
                .unwrap()
                .captures[0],
            Some(1..1)
        );
        assert!(matcher.find(&JsString::from(" "), 0, true).is_none());
    }

    #[test]
    fn reference_free_body_preparation_keeps_deep_captures_and_huge_empty_counts_flat() {
        let source =
            JsString::from(format!("{}a{}+\\1", "(".repeat(100000), ")".repeat(100000)).as_str());
        let matcher = RegExpBackreferenceMatcher::compile(&source, false).unwrap();
        let found = matcher.find(&JsString::from("aaa"), 0, true).unwrap();
        assert_eq!(found.range, 0..3);
        assert_eq!(found.captures.len(), 100000);
        assert!(found.captures.iter().all(|r| *r == Some(1..2)));
        let source = JsString::from(format!("(){{{}}}\\1", "9".repeat(10000)).as_str());
        let matcher = RegExpBackreferenceMatcher::compile(&source, false).unwrap();
        let mut work = 0;
        let found = matcher
            .find_with_work(&JsString::from("q"), 0, true, |n| {
                work += n;
                Ok::<_, ()>(())
            })
            .unwrap()
            .unwrap();
        assert_eq!(found.range, 0..0);
        assert_eq!(found.captures[0], Some(0..0));
        assert!(work < 200, "empty work {work}");
        assert!(matches!(
            matcher.find_with_work(&JsString::from("q"), 0, true, |_| Err::<(), _>("work")),
            Err("work")
        ));
        for source in [r"(a|b)+\1", r"((a)+)+\1", r"(?:(a+)\1){2}", r"((?=a)a)+\1"] {
            let mut work = 0;
            assert!(
                RegExpBackreferenceMatcher::compile_with_work(
                    &JsString::from(source),
                    false,
                    |n| {
                        work += n;
                        Ok::<_, ()>(())
                    }
                )
                .unwrap()
                .is_none(),
                "{source}"
            );
            assert_eq!(work, 0, "{source}");
        }
    }

    #[test]
    fn repeated_reference_assertion_execution_snapshot() {
        let mut rows = String::new();
        for source in [
            r"(a)(b)(\b\1\2)*",
            r"(a)(b)(\b\1\2)*?",
            r"(a)(b)(\B\1\2)+",
            r"(a)(b)(\1\B\2)+?",
            r"(a)(b)(\1\2$)?",
            r"(a)(b)(^\1\2)??",
            r"(a)(bb)((\B\2)\4\1){1,3}",
            r"(a)(bb)(\2(.)(\B)\4){1,3}?",
            r"(a)(b)(()(\Ba)\5\2())*ab",
            r"(a)(b)(()\1()([ab])(\B)\6())*?ab",
            r"((a)(b)((\Ba)\5\3)+)\1",
            r"((a)(b)(\2([ab])\B\5)+?)\1",
            r"(?:(a)|(b))((\Ba)\4\2)+c",
            r"(?:(a)|(b))((\w)\B\4\1)+c",
            r"(a)(b)((a)\B\4\2)*\4",
            r"(a)(b)(\1([ab])\B\4)*?\4",
            r"(a)(b)(\4(a)\B\4\2)+",
            r"(a)(b)((\4a)\B\4\2)+",
            r"()()((^)\1\2){2,3}",
            r"(a)(b)(a\B\1\2){999999999999999999999999999999}",
            r"(a)(b)(a\B\1\2){0,999999999999999999999999999999}?c",
            r"^([µ])(a)((.)\B\4\2)+$",
            r"\b(\w)(\w)(\1(\d\w)\B\4)+\b",
            r"(.)(a)((.)$\4\2){1,2}b|()()(^\5\6)+",
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
                    "ab",
                    "abab",
                    "ababab",
                    "abb",
                    "abbabba",
                    "abbaabb",
                    "abababc",
                    "abbbbbba",
                    "aabb",
                    "aAbB",
                    "µaΜAµa",
                    "\n\n",
                    " ababab ",
                    "\u{2028}ab\u{2029}",
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
    fn repeated_assertions_use_full_input_context_and_restore_final_ranges() {
        for (source, input, start, multiline, dot_all, captures) in [
            (
                r"(a)(b)((\Ba)\4\2)*aab$",
                "abaabaab",
                0,
                false,
                false,
                Some(vec![Some(0..1), Some(1..2), Some(2..5), Some(2..3)]),
            ),
            (
                r"(a)(b)(([ab])\B\4\2)+$",
                "abaabbbb",
                0,
                false,
                false,
                Some(vec![Some(0..1), Some(1..2), Some(5..8), Some(5..6)]),
            ),
            (r"(\1^)+", "q", 0, false, false, Some(vec![Some(0..0)])),
            (r"(\1^)+", "q", 1, false, false, None),
            (r"(\1^)*", "q", 1, false, false, Some(vec![None])),
            (
                r"()((^)\1){2}",
                "q",
                0,
                false,
                false,
                Some(vec![Some(0..0), Some(0..0), Some(0..0)]),
            ),
            (r"()((^)\1){2}", "q", 1, false, false, None),
            (
                r"()((^)\1){0,2}",
                "q",
                1,
                false,
                false,
                Some(vec![Some(1..1), None, None]),
            ),
            (
                r"()((^)\1){2}",
                "\nq",
                1,
                true,
                false,
                Some(vec![Some(1..1), Some(1..1), Some(1..1)]),
            ),
            (r"()((^)\1){2}", "\nq", 1, false, false, None),
            (
                r"()((\b)\1)+",
                "x",
                1,
                false,
                false,
                Some(vec![Some(1..1), Some(1..1), Some(1..1)]),
            ),
            (r"()((\b)\1)+", " ", 0, false, false, None),
            (
                r"()((\B)\1)+",
                " ",
                0,
                false,
                false,
                Some(vec![Some(0..0), Some(0..0), Some(0..0)]),
            ),
            (
                r"(.)(a)((.)$\4\2){1,2}b",
                "aa\n\nab",
                0,
                true,
                true,
                Some(vec![Some(0..1), Some(1..2), Some(2..5), Some(2..3)]),
            ),
            (r"(.)(a)((.)$\4\2){1,2}b", "aa\n\nab", 0, false, true, None),
        ] {
            let matcher = RegExpBackreferenceMatcher::compile_with_assertions_and_work(
                &JsString::from(source),
                false,
                multiline,
                dot_all,
                |_| Ok::<_, ()>(()),
            )
            .unwrap()
            .unwrap();
            let found = matcher.find(&JsString::from(input), start, true);
            assert_eq!(
                found.map(|m| m.captures.into_vec()),
                captures,
                "{source} input={input:?}"
            );
        }
    }

    #[test]
    fn many_empty_assertion_captures_and_huge_required_minimums_remain_finite() {
        let source = JsString::from(format!("()({}\\1)+", r"(\b)".repeat(100000)).as_str());
        let matcher = RegExpBackreferenceMatcher::compile(&source, false).unwrap();
        let found = matcher.find(&JsString::from("x"), 0, true).unwrap();
        assert_eq!(found.range, 0..0);
        assert_eq!(found.captures.len(), 100002);
        assert!(found.captures.iter().all(|r| *r == Some(0..0)));
        assert!(matcher.find(&JsString::from(" "), 0, true).is_none());
        let source = JsString::from(format!("()((^)\\1){{{}}}", "9".repeat(10000)).as_str());
        let matcher = RegExpBackreferenceMatcher::compile(&source, false).unwrap();
        let mut work = 0;
        let found = matcher
            .find_with_work(&JsString::from("q"), 0, true, |n| {
                work += n;
                Ok::<_, ()>(())
            })
            .unwrap()
            .unwrap();
        assert!(found.captures.iter().all(|r| *r == Some(0..0)));
        assert!(work < 200, "empty assertion work {work}");
        assert!(matcher.find(&JsString::from("q"), 1, true).is_none());
        assert!(matches!(
            matcher.find_with_work(&JsString::from("q"), 0, true, |_| Err::<(), _>("work")),
            Err("work")
        ));
        for source in [
            r"(a)(b)((?=a)\1\2)+",
            r"(a)(b)((?i:\1)\2)+",
            r"(a)(b)(^\1|\2)+",
            r"(a)(b)(^\1+\2)+",
        ] {
            let mut work = 0;
            assert!(
                RegExpBackreferenceMatcher::compile_with_work(
                    &JsString::from(source),
                    false,
                    |n| {
                        work += n;
                        Ok::<_, ()>(())
                    }
                )
                .unwrap()
                .is_none(),
                "{source}"
            );
            assert_eq!(work, 0, "{source}");
        }
    }

    #[test]
    fn repeated_reference_character_execution_snapshot() {
        let mut rows = String::new();
        for source in [
            r"(a)(a\1)*",
            r"(a)(a\1)*?",
            r"(a)(b)(a\1\2)+",
            r"(a)(b)(\1b\2)+?",
            r"(a)(b)([ab]\1\2)?",
            r"(a)(b)(\1[^c]\2)??",
            r"(a)(bb)(([ab])\4\2){1,3}",
            r"(a)(bb)(\2(.)\4){1,3}?",
            r"(a)(b)(()(a)\5\2())*ab",
            r"(a)(b)(()\1()([ab])\6())*?ab",
            r"((a)(b)((a)\5\3)+)\1",
            r"((a)(b)(\2([ab])\5)+?)\1",
            r"(?:(a)|(b))((a)\4\2)+c",
            r"(?:(a)|(b))((\w)\4\1)+c",
            r"(a)(b)((a)\4\2)*\4",
            r"(a)(b)(\1([ab])\4)*?\4",
            r"(a)(b)(\4(a)\4\2)+",
            r"(a)(b)((\4a)\4\2)+",
            r"()()((a)\4\1\2){2,3}",
            r"(a)(b)(a\1\2){999999999999999999999999999999}",
            r"(a)(b)(a\1\2){0,999999999999999999999999999999}?c",
            r"^([µ])(a)((.)\4\2)+$",
            r"\b(\w)(\w)(\1(\d\w)\4)+\b",
            r"(.)(a)((.)\4\2){1,2}b|()()([ab]\5\6)+",
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
                    "ab",
                    "abab",
                    "ababab",
                    "abb",
                    "abbabba",
                    "abbaabb",
                    "abababc",
                    "abbbbbba",
                    "aabb",
                    "aAbB",
                    "µaΜAµa",
                    "\n\n",
                    " ababab ",
                    "\u{2028}ab\u{2029}",
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
    fn repeated_character_terms_keep_partial_local_ranges_and_complete_retries() {
        for (source, input, captures) in [
            (
                r"(a)(b)((a)\4\2)*aab$",
                "abaabaab",
                vec![Some(0..1), Some(1..2), Some(2..5), Some(2..3)],
            ),
            (
                r"(a)(b)(\1([ab])\4)+$",
                "ababbabb",
                vec![Some(0..1), Some(1..2), Some(5..8), Some(6..7)],
            ),
            (
                r"(a)(b)(\4(a)\4\2)+",
                "abaabaab",
                vec![Some(0..1), Some(1..2), Some(5..8), Some(5..6)],
            ),
            (
                r"()()((a)\4\1\2){2,3}",
                "aaaaaa",
                vec![Some(0..0), Some(0..0), Some(4..6), Some(4..5)],
            ),
            (
                r"(?:(a)|(b))((a)\4\2)+c",
                "baabc",
                vec![None, Some(0..1), Some(1..4), Some(1..2)],
            ),
        ] {
            let matcher =
                RegExpBackreferenceMatcher::compile(&JsString::from(source), false).unwrap();
            let found = matcher.find(&JsString::from(input), 0, true).unwrap();
            assert_eq!(&*found.captures, &*captures, "{source}");
        }
        for (source, input, captures) in [
            (r"(\1a)+", "aaa", vec![Some(2..3)]),
            (r"((a)\1)+", "aaa", vec![Some(2..3), Some(2..3)]),
            (r"(?:(a)\1){2}", "aaaa", vec![Some(2..3)]),
        ] {
            let matcher =
                RegExpBackreferenceMatcher::compile(&JsString::from(source), false).unwrap();
            let found = matcher.find(&JsString::from(input), 0, true).unwrap();
            assert_eq!(&*found.captures, &*captures, "{source}");
        }
        let source = JsString::from(r"(a)(b)((.)\4\2)+$");
        for dot_all in [false, true] {
            let matcher = RegExpBackreferenceMatcher::compile_with_flags_and_work(
                &source,
                false,
                dot_all,
                |_| Ok::<_, ()>(()),
            )
            .unwrap()
            .unwrap();
            assert_eq!(
                matcher.find(&JsString::from("ab\n\nb"), 0, true).is_some(),
                dot_all
            );
            let input = JsString::from_code_units(vec![0x61, 0x62, 0xd800, 0xd800, 0x62]);
            let found = matcher.find(&input, 0, true).unwrap();
            assert_eq!(found.range, 0..5);
            assert_eq!(found.captures[3], Some(2..3));
        }
    }

    #[test]
    fn deep_character_body_captures_shared_sets_and_huge_bounds_are_iterative() {
        let source = JsString::from(
            format!(
                "(a)(b)({}a{}\\4\\2)+",
                "(".repeat(100000),
                ")".repeat(100000)
            )
            .as_str(),
        );
        let matcher = RegExpBackreferenceMatcher::compile(&source, false).unwrap();
        let found = matcher.find(&JsString::from("abaabaab"), 0, true).unwrap();
        assert_eq!(found.range, 0..8);
        assert_eq!(found.captures[2], Some(5..8));
        assert!(found.captures[3..].iter().all(|r| *r == Some(5..6)));
        let source = JsString::from(format!("(a)(?:{})+", r"[ab]\1".repeat(50000)).as_str());
        let mut work = 0;
        let matcher = RegExpBackreferenceMatcher::compile_with_work(&source, false, |n| {
            work += n;
            Ok::<_, ()>(())
        })
        .unwrap()
        .unwrap();
        assert!(
            work < source.len() * 12 + 131072,
            "shared set construction work {work}"
        );
        let input = JsString::from("a".repeat(100001).as_str());
        assert_eq!(matcher.find(&input, 0, true).unwrap().range, 0..100001);
        assert!(matches!(
            matcher.find_with_work(&input, 0, true, |_| Err::<(), _>("work")),
            Err("work")
        ));
        for (min, expected) in [("0,", true), ("", false)] {
            let source =
                JsString::from(format!("(a)(a\\1){{{min}{}}}", "9".repeat(10000)).as_str());
            let matcher = RegExpBackreferenceMatcher::compile(&source, false).unwrap();
            let found = matcher.find(&JsString::from("a"), 0, true);
            assert_eq!(found.is_some(), expected);
            if let Some(found) = found {
                assert_eq!(found.range, 0..1);
                assert_eq!(found.captures[1], None);
            }
        }
        for source in [
            r"(a)(b)((\1)|\2)+",
            r"(a)(b)(a\1+\2)+",
            r"(a)(b)((?=a)\1\2)+",
            r"(?:(a)(b)(\b\1\2)+){2}",
        ] {
            let mut work = 0;
            assert!(
                RegExpBackreferenceMatcher::compile_with_work(
                    &JsString::from(source),
                    false,
                    |n| {
                        work += n;
                        Ok::<_, ()>(())
                    }
                )
                .unwrap()
                .is_none(),
                "{source}"
            );
            assert_eq!(work, 0, "{source}");
        }
    }

    #[test]
    fn local_reference_target_execution_snapshot() {
        let mut rows = String::new();
        for source in [
            r"(a)((\1)\3)*",
            r"(a)((\1)\3)*?",
            r"(a)(b)((\1)\4\2)+",
            r"(a)(b)(\1(\2)\4)+?",
            r"(a)(b)((\1)\4\2)?",
            r"(a)(b)(\1(\2)\4)??",
            r"(a)(bb)((\2)\4\1){1,3}",
            r"(a)(bb)(\2(\1)\4){1,3}?",
            r"(a)(b)(()(\1)()\5\2())*ab",
            r"(a)(b)(()\1()(\2)\6())*?ab",
            r"((a)(b)((\2)\5\3)+)\1",
            r"((a)(b)(\2(\3)\5)+?)\1",
            r"(?:(a)|(b))((\1)\4\2)+c",
            r"(?:(a)|(b))((\2)\4\1)+c",
            r"(a)(b)((\1)\4\2)*\4",
            r"(a)(b)(\1(\2)\4)*?\4",
            r"(a)(b)(\4(\1)\4\2)+",
            r"(a)(b)((\4\1)\4\2)+",
            r"()()((\1)\4\2){2,3}",
            r"(a)(b)((\1)\4\2){999999999999999999999999999999}",
            r"(a)(b)((\1)\4\2){0,999999999999999999999999999999}?c",
            r"^([µ])(a)((\1)\4\2)+$",
            r"\b(\w)(\w)(\1(\2\1)\4)+\b",
            r"(a)(b)(((\1)\5)\4\2)+",
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
                    "ab",
                    "abab",
                    "ababab",
                    "abb",
                    "abbabba",
                    "abbaabb",
                    "abababc",
                    "abbbbbba",
                    "aabb",
                    "aAbB",
                    "µaΜAµa",
                    "\n\n",
                    " ababab ",
                    "\u{2028}ab\u{2029}",
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
    fn local_reference_ranges_use_the_current_iteration_and_retry_capture_positions() {
        for (source, input, captures) in [
            (
                r"(a)(b)((\1)\4\2)+$",
                "abaabaab",
                vec![Some(0..1), Some(1..2), Some(5..8), Some(5..6)],
            ),
            (
                r"(a)((\1)\3)+$",
                "aaaaa",
                vec![Some(0..1), Some(3..5), Some(3..4)],
            ),
            (
                r"(a)(b)(\1(\2\1)\4)+$",
                "abababa",
                vec![Some(0..1), Some(1..2), Some(2..7), Some(3..5)],
            ),
            (
                r"(a)(b)(((\1)\5)\4\2)+$",
                "abaaaab",
                vec![Some(0..1), Some(1..2), Some(2..7), Some(2..4), Some(2..3)],
            ),
            (
                r"(?:(a)|(b))((\1)\4\2)+c",
                "bbc",
                vec![None, Some(0..1), Some(1..2), Some(1..1)],
            ),
            (
                r"(a)(b)((\1)\4\2)*aab$",
                "abaabaab",
                vec![Some(0..1), Some(1..2), Some(2..5), Some(2..3)],
            ),
            (
                r"(a)(b)(\4(\1)\4\2)+",
                "abaabaab",
                vec![Some(0..1), Some(1..2), Some(5..8), Some(5..6)],
            ),
            (
                r"()()((\1)\4\2)*",
                "",
                vec![Some(0..0), Some(0..0), None, None],
            ),
            (
                r"()()((\1)\4\2)+",
                "",
                vec![Some(0..0), Some(0..0), Some(0..0), Some(0..0)],
            ),
        ] {
            let matcher =
                RegExpBackreferenceMatcher::compile(&JsString::from(source), false).unwrap();
            let found = matcher.find(&JsString::from(input), 0, true).unwrap();
            assert_eq!(&*found.captures, &*captures, "{source}");
        }
    }

    #[test]
    fn local_reference_buffers_huge_empty_counts_and_width_overflow_are_safe() {
        let source = JsString::from(format!("(a)(b)((\\1){}\\2)+", r"\4".repeat(100000)).as_str());
        let matcher = RegExpBackreferenceMatcher::compile(&source, false).unwrap();
        let input = JsString::from(format!("ab{}b", "a".repeat(100001)).as_str());
        let found = matcher.find(&input, 0, true).unwrap();
        assert_eq!(found.range, 0..100004);
        assert_eq!(found.captures[2], Some(2..100004));
        assert_eq!(found.captures[3], Some(2..3));
        assert!(matches!(
            matcher.find_with_work(&input, 0, true, |_| Err::<(), _>("work")),
            Err("work")
        ));
        let source = JsString::from(format!("()()((\\1)\\4\\2){{{}}}", "9".repeat(10000)).as_str());
        let matcher = RegExpBackreferenceMatcher::compile(&source, false).unwrap();
        let mut work = 0;
        let found = matcher
            .find_with_work(&JsString::from(""), 0, true, |n| {
                work += n;
                Ok::<_, ()>(())
            })
            .unwrap()
            .unwrap();
        assert!(found.captures.iter().all(|r| *r == Some(0..0)));
        assert!(work < 200, "empty work {work}");
        let mut body = String::from(r"(\1)");
        for slot in 4..4 + usize::BITS as usize + 2 {
            write!(body, r"(\{slot}\{slot})").unwrap();
        }
        for (quantifier, expected) in [("*", true), ("+", false)] {
            let source = JsString::from(format!("(a)(b)({body}){quantifier}").as_str());
            let matcher = RegExpBackreferenceMatcher::compile(&source, false).unwrap();
            let found = matcher.find(&JsString::from("ab"), 0, true);
            assert_eq!(found.is_some(), expected);
            if let Some(found) = found {
                assert_eq!(found.range, 0..2);
                assert!(found.captures[2..].iter().all(Option::is_none));
            }
        }
        for source in [
            r"(?:(a)(b)(a(\1)\4\2)+){2}",
            r"(a)(b)((\1)|\2)+",
            r"(a)(b)((\1)+\4\2)+",
        ] {
            let mut work = 0;
            assert!(
                RegExpBackreferenceMatcher::compile_with_work(
                    &JsString::from(source),
                    false,
                    |n| {
                        work += n;
                        Ok::<_, ()>(())
                    }
                )
                .unwrap()
                .is_none(),
                "{source}"
            );
            assert_eq!(work, 0, "{source}");
        }
    }

    #[test]
    fn partial_reference_capture_execution_snapshot() {
        let mut rows = String::new();
        for source in [
            r"(a)((\1)\1)*",
            r"(a)((\1)\1)*?",
            r"(a)(b)((\1)\2)+",
            r"(a)(b)(\1(\2))+?",
            r"(a)(b)((\1)\2)?",
            r"(a)(b)(\1(\2))??",
            r"(a)(bb)((\2)\1){1,3}",
            r"(a)(bb)(\2(\1)){1,3}?",
            r"(a)(b)(()(\1)()\2())*ab",
            r"(a)(b)(()\1()(\2)())*?ab",
            r"((a)(b)((\2)\3)+)\1",
            r"((a)(b)(\2(\3))+?)\1",
            r"(?:(a)|(b))((\1)\2)+c",
            r"(?:(a)|(b))((\2)\1)+c",
            r"(a)(b)((\1)\2)*\4",
            r"(a)(b)(\1(\2))*?\4",
            r"(a)(b)(\4(\1)\2)+",
            r"(a)(b)((\4\1)\2)+",
            r"()()((\1)\2){2,3}",
            r"(a)(b)((\1)\2){999999999999999999999999999999}",
            r"(a)(b)((\1)\2){0,999999999999999999999999999999}?c",
            r"^([µ])(a)((\1)\2)+$",
            r"\b(\w)(\w)(\1(\2\1)\2)+\b",
            r"(.)(a)((\1)\2){1,2}b|()()((\5)\6)+",
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
                    "ab",
                    "abab",
                    "ababab",
                    "abb",
                    "abbabba",
                    "abbaabb",
                    "abababc",
                    "abbbbbba",
                    "aabb",
                    "aAbB",
                    "µaΜAµa",
                    "\n\n",
                    " ababab ",
                    "\u{2028}ab\u{2029}",
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
    fn partial_ranges_follow_reference_ordinals_and_restore_after_retries() {
        for (source, input, range, captures) in [
            (
                r"(a)(b)((\1)\2)*ab$",
                "ababab",
                0..6,
                vec![Some(0..1), Some(1..2), Some(2..4), Some(2..3)],
            ),
            (
                r"(a)(bb)(\2(\1))+$",
                "abbbbabba",
                0..9,
                vec![Some(0..1), Some(1..3), Some(6..9), Some(8..9)],
            ),
            (
                r"(a)((\1)\1)+$",
                "aaaaa",
                0..5,
                vec![Some(0..1), Some(3..5), Some(3..4)],
            ),
            (
                r"(?:(a)|(b))((\1)\2)+c",
                "bbc",
                0..3,
                vec![None, Some(0..1), Some(1..2), Some(1..1)],
            ),
            (
                r"(a)(b)(\4(\1)\2)+",
                "ababab",
                0..6,
                vec![Some(0..1), Some(1..2), Some(4..6), Some(4..5)],
            ),
            (
                r"(a)(b)((\4\1)\2)+",
                "ababab",
                0..6,
                vec![Some(0..1), Some(1..2), Some(4..6), Some(4..5)],
            ),
            (
                r"(a)(b)((\1)\2)*\4$",
                "ababa",
                0..5,
                vec![Some(0..1), Some(1..2), Some(2..4), Some(2..3)],
            ),
            (
                r"()()((\1)\2)*",
                "",
                0..0,
                vec![Some(0..0), Some(0..0), None, None],
            ),
            (
                r"()()((\1)\2)+",
                "",
                0..0,
                vec![Some(0..0), Some(0..0), Some(0..0), Some(0..0)],
            ),
        ] {
            let matcher =
                RegExpBackreferenceMatcher::compile(&JsString::from(source), false).unwrap();
            let found = matcher.find(&JsString::from(input), 0, true).unwrap();
            assert_eq!(found.range, range, "{source}");
            assert_eq!(&*found.captures, &*captures, "{source}");
        }
    }

    #[test]
    fn partial_capture_preparation_is_linear_and_preserves_local_target_boundaries() {
        let source = JsString::from(
            format!(
                "(a)(b)({}\\1{}\\2)+",
                "(".repeat(100000),
                ")".repeat(100000)
            )
            .as_str(),
        );
        let matcher = RegExpBackreferenceMatcher::compile(&source, false).unwrap();
        let found = matcher.find(&JsString::from("ababab"), 0, true).unwrap();
        assert_eq!(found.range, 0..6);
        assert_eq!(found.captures[2], Some(4..6));
        assert!(found.captures[3..].iter().all(|r| *r == Some(4..5)));
        let source = JsString::from(format!("()()((\\1)\\2){{{}}}", "9".repeat(10000)).as_str());
        let matcher = RegExpBackreferenceMatcher::compile(&source, false).unwrap();
        let mut work = 0;
        let found = matcher
            .find_with_work(&JsString::from(""), 0, true, |n| {
                work += n;
                Ok::<_, ()>(())
            })
            .unwrap()
            .unwrap();
        assert!(found.captures.iter().all(|r| *r == Some(0..0)));
        assert!(work < 200, "empty work {work}");
        assert!(matches!(
            matcher.find_with_work(&JsString::from(""), 0, true, |_| Err::<(), _>("work")),
            Err("work")
        ));
        for source in [
            r"(?:(a)(b)((\1)\4\2)+){2}",
            r"(?:(a)((\1)\3)+){2}",
            r"(?:(a)(b)(\1(\2)\4)+){2}",
            r"(?:(a)(b)(a(\1)\2)+){2}",
            r"(a)(b)((\1)|\2)+",
            r"(a)(b)((\1)+\2)+",
        ] {
            let mut work = 0;
            assert!(
                RegExpBackreferenceMatcher::compile_with_work(
                    &JsString::from(source),
                    false,
                    |n| {
                        work += n;
                        Ok::<_, ()>(())
                    }
                )
                .unwrap()
                .is_none(),
                "{source}"
            );
            assert_eq!(work, 0, "{source}");
        }
        for (source, slots) in [
            (r"(?:(a)(b)((\1)\k<x>\2)+){2}", &[3][..]),
            (r"(?:(a)(b)(\1(\2)\k<x>)+){2}", &[3][..]),
        ] {
            let source = JsString::from(source);
            let refs = named_escapes(&source);
            let groups = [slots];
            let mut work = 0;
            assert!(
                RegExpBackreferenceMatcher::compile_with_named_bindings_and_work(
                    &source,
                    false,
                    false,
                    false,
                    RegExpBackreferenceNamedBindings {
                        groups: &groups,
                        references: &refs
                    },
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
    fn middle_empty_capture_execution_snapshot() {
        let mut rows = String::new();
        for source in [
            r"(a)(\1()\1)*",
            r"(a)(\1()\1)*?",
            r"(a)(b)(\1()\2)+",
            r"(a)(b)(\1()\2)+?",
            r"(a)(b)(\1()\2)?",
            r"(a)(b)(\1()\2)??",
            r"(a)(bb)(\2()\1){1,3}",
            r"(a)(bb)(\2()\1){1,3}?",
            r"(a)(b)(()\1()\2())*ab",
            r"(a)(b)(()\1()\2())*?ab",
            r"((a)(b)(\2()\3)+)\1",
            r"((a)(b)(\2()\3)+?)\1",
            r"(?:(a)|(b))(\1()\2)+c",
            r"(?:(a)|(b))(\2()\1)+c",
            r"(a)(b)(\1()\4\2)*(\2()\1)*",
            r"(a)(b)(\1()\4\2)*?(\2()\1)*?",
            r"(\3()\4)+(a)(b)",
            r"((\1()\4)+a)(b)",
            r"()()(\1()\2){2,3}",
            r"(a)(b)(\1()\2){999999999999999999999999999999}",
            r"(a)(b)(\1()\2){0,999999999999999999999999999999}?c",
            r"^([µ])(a)(()\1()\2())+$",
            r"\b(\w)(\w)(\1()\2)+\b",
            r"(.)(a)(\1()\2){1,2}b|()()(\5()\6)+",
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
                    "ab",
                    "abab",
                    "ababab",
                    "abb",
                    "abbabba",
                    "abbaabb",
                    "abababc",
                    "abbbbbba",
                    "aabb",
                    "aAbB",
                    "µaΜAµa",
                    "\n\n",
                    " ababab ",
                    "\u{2028}ab\u{2029}",
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
    fn middle_empty_positions_follow_original_reference_widths_on_retries() {
        for (source, input, captures) in [
            (
                r"(a)(b)(()\1()\2())*ab$",
                "ababab",
                vec![
                    Some(0..1),
                    Some(1..2),
                    Some(2..4),
                    Some(2..2),
                    Some(3..3),
                    Some(4..4),
                ],
            ),
            (
                r"(a)(bb)(\2()\1)+$",
                "abbbbabba",
                vec![Some(0..1), Some(1..3), Some(6..9), Some(8..8)],
            ),
            (
                r"(a)(\1()\1)+$",
                "aaaaa",
                vec![Some(0..1), Some(3..5), Some(4..4)],
            ),
            (
                r"(?:(a)|(b))(\1()\2)+c",
                "bbc",
                vec![None, Some(0..1), Some(1..2), Some(1..1)],
            ),
            (
                r"(?:(a)|(b))(\2()\1)+c",
                "bbc",
                vec![None, Some(0..1), Some(1..2), Some(2..2)],
            ),
            (
                r"(a)(b)(\1()\4\2)+",
                "ababab",
                vec![Some(0..1), Some(1..2), Some(4..6), Some(5..5)],
            ),
            (
                r"()()(\1()\2)*",
                "",
                vec![Some(0..0), Some(0..0), None, None],
            ),
            (
                r"()()(\1()\2)+",
                "",
                vec![Some(0..0), Some(0..0), Some(0..0), Some(0..0)],
            ),
        ] {
            let matcher =
                RegExpBackreferenceMatcher::compile(&JsString::from(source), false).unwrap();
            let found = matcher.find(&JsString::from(input), 0, true).unwrap();
            assert_eq!(&*found.captures, &*captures, "{source}");
        }
    }

    #[test]
    fn many_middle_empty_captures_and_finite_empty_minimums_remain_iterative() {
        let source = JsString::from(format!("(a)(b)(\\1{}\\2)+", "()".repeat(100000)).as_str());
        let matcher = RegExpBackreferenceMatcher::compile(&source, false).unwrap();
        let found = matcher.find(&JsString::from("ababab"), 0, true).unwrap();
        assert_eq!(found.range, 0..6);
        assert_eq!(found.captures.len(), 100003);
        assert_eq!(found.captures[2], Some(4..6));
        assert!(found.captures[3..].iter().all(|r| *r == Some(5..5)));
        let source = JsString::from(format!("()()(\\1()\\2){{{}}}", "9".repeat(10000)).as_str());
        let matcher = RegExpBackreferenceMatcher::compile(&source, false).unwrap();
        let mut work = 0;
        let found = matcher
            .find_with_work(&JsString::from(""), 0, true, |n| {
                work += n;
                Ok::<_, ()>(())
            })
            .unwrap()
            .unwrap();
        assert!(found.captures.iter().all(|r| *r == Some(0..0)));
        assert!(work < 200, "empty work {work}");
        assert!(matches!(
            matcher.find_with_work(&JsString::from(""), 0, true, |_| Err::<(), _>("work")),
            Err("work")
        ));
        for source in [
            r"(?:(a)(b)((\1)\2)+){2}",
            r"(?:(a)(b)(a\1()\2)+){2}",
            r"(a)(b)(\1|()\2)+",
            r"(a)(b)(\1+()\2)+",
        ] {
            let mut work = 0;
            assert!(
                RegExpBackreferenceMatcher::compile_with_work(
                    &JsString::from(source),
                    false,
                    |n| {
                        work += n;
                        Ok::<_, ()>(())
                    }
                )
                .unwrap()
                .is_none(),
                "{source}"
            );
            assert_eq!(work, 0, "{source}");
        }
    }

    #[test]
    fn captured_reference_sequence_execution_snapshot() {
        let mut rows = String::new();
        for source in [
            r"(a)(b)(\1\2)*",
            r"(a)(b)(\1\2)*?",
            r"(a)(b)(\1\2)+",
            r"(a)(b)(\1\2)+?",
            r"(a)(b)(\1\2)?",
            r"(a)(b)(\1\2)??",
            r"(a)(bb)(\2\1){1,3}",
            r"(a)(bb)(\2\1){1,3}?",
            r"(a)(b)(()\1\2())*ab",
            r"(a)(b)(()\1\2())*?ab",
            r"((a)(b)(\2\3)+)\1",
            r"((a)(b)(\2\3)+?)\1",
            r"(?:(a)|(b))(\1\2)+c",
            r"(?:(a)|(b))(\2\1)+c",
            r"(a)(b)(\1\2)*(\2\1)*",
            r"(a)(b)(\1\2)*?(\2\1)*?",
            r"(\2\3)+(a)(b)",
            r"((\1\3)+a)(b)",
            r"()()(\1\2){2,3}",
            r"(a)(b)(\1\2){999999999999999999999999999999}",
            r"(a)(b)(\1\2){0,999999999999999999999999999999}?c",
            r"^([µ])(a)(()\1\2())+$",
            r"\b(\w)(\w)(\1\2)+\b",
            r"(.)(a)(\1\2){1,2}b|()()(\4\5)+",
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
                    "ab",
                    "abab",
                    "ababab",
                    "abb",
                    "abbabba",
                    "abbaabb",
                    "abababc",
                    "abbbbbba",
                    "aabb",
                    "aAbB",
                    "µaΜAµa",
                    "\n\n",
                    " ababab ",
                    "\u{2028}ab\u{2029}",
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
    fn captured_sequence_retries_restore_final_ranges_and_empty_siblings() {
        for (source, input, range, captures) in [
            (
                r"(a)(b)(()\1\2())*ab$",
                "ababab",
                0..6,
                vec![Some(0..1), Some(1..2), Some(2..4), Some(2..2), Some(4..4)],
            ),
            (
                r"((a)(b)(\2\3)+?)\1",
                "abababab",
                0..8,
                vec![Some(0..4), Some(0..1), Some(1..2), Some(2..4)],
            ),
            (
                r"(?:(a)|(b))(\1\2)+c",
                "bbc",
                0..3,
                vec![None, Some(0..1), Some(1..2)],
            ),
            (r"()()(\1\2)*", "", 0..0, vec![Some(0..0), Some(0..0), None]),
            (
                r"()()(\1\2)+",
                "",
                0..0,
                vec![Some(0..0), Some(0..0), Some(0..0)],
            ),
            (
                r"(\2\3)+(a)(b)",
                "ab",
                0..2,
                vec![Some(0..0), Some(0..1), Some(1..2)],
            ),
        ] {
            let matcher =
                RegExpBackreferenceMatcher::compile(&JsString::from(source), false).unwrap();
            let found = matcher.find(&JsString::from(input), 0, true).unwrap();
            assert_eq!(found.range, range, "{source}");
            assert_eq!(&*found.captures, &*captures, "{source}");
        }
    }

    #[test]
    fn deep_captured_sequences_keep_iterative_ranges_and_huge_empty_minimums_finite() {
        let source = JsString::from(
            format!("(a)(b){}\\1\\2{}+", "(".repeat(100000), ")".repeat(100000)).as_str(),
        );
        let matcher = RegExpBackreferenceMatcher::compile(&source, false).unwrap();
        let found = matcher.find(&JsString::from("ababab"), 0, true).unwrap();
        assert_eq!(found.range, 0..6);
        assert_eq!(found.captures.len(), 100002);
        assert!(found.captures[2..].iter().all(|r| *r == Some(4..6)));
        let source = JsString::from(format!("()()(()\\1\\2()){{{}}}", "9".repeat(10000)).as_str());
        let matcher = RegExpBackreferenceMatcher::compile(&source, false).unwrap();
        let mut work = 0;
        let found = matcher
            .find_with_work(&JsString::from(""), 0, true, |n| {
                work += n;
                Ok::<_, ()>(())
            })
            .unwrap()
            .unwrap();
        assert!(found.captures.iter().all(|r| *r == Some(0..0)));
        assert!(work < 200, "empty work {work}");
        assert!(matches!(
            matcher.find_with_work(&JsString::from(""), 0, true, |_| Err::<(), _>("work")),
            Err("work")
        ));
        for source in [
            r"(?:(a)(b)((\1)\2)+){2}",
            r"(?:(a)(b)(a\1()\2)+){2}",
            r"(?:(a)(b)(a\1\2)+){2}",
            r"(a)(b)(\1|\2)+",
            r"(a)(b)(\1+\2)+",
        ] {
            let mut work = 0;
            assert!(
                RegExpBackreferenceMatcher::compile_with_work(
                    &JsString::from(source),
                    false,
                    |n| {
                        work += n;
                        Ok::<_, ()>(())
                    }
                )
                .unwrap()
                .is_none(),
                "{source}"
            );
            assert_eq!(work, 0, "{source}");
        }
    }
    #[test]
    fn reference_sequence_body_execution_snapshot() {
        let mut rows = String::new();
        for source in [
            r"(a)(b)(?:\1\2)*",
            r"(a)(b)(?:\1\2)*?",
            r"(a)(b)(?:\1\2)+",
            r"(a)(b)(?:\1\2)+?",
            r"(a)(b)(?:\1\2)?",
            r"(a)(b)(?:\1\2)??",
            r"(a)(bb)(?:\2\1){1,3}",
            r"(a)(bb)(?:\2\1){1,3}?",
            r"(a)(b)(?:\1\2)*ab",
            r"(a)(b)(?:\1\2)*?ab",
            r"((a)(b)(?:\2\3)+)\1",
            r"((a)(b)(?:\2\3)+?)\1",
            r"(?:(a)|(b))(?:\1\2)+c",
            r"(?:(a)|(b))(?:\2\1)+c",
            r"(a)(b)(?:\1\2)*(?:\2\1)*",
            r"(a)(b)(?:\1\2)*?(?:\2\1)*?",
            r"(?:\1\2)+(a)(b)",
            r"((?:\1\2)+a)(b)",
            r"()()(?:\1\2){2,3}",
            r"(a)(b)(?:\1\2){999999999999999999999999999999}",
            r"(a)(b)(?:\1\2){0,999999999999999999999999999999}?c",
            r"^([µ])(a)(?:\1\2)+$",
            r"\b(\w)(\w)(?:\1\2)+\b",
            r"(.)(a)(?:\1\2){1,2}b|()()(?:\3\4)+",
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
                    "ab",
                    "abab",
                    "ababab",
                    "abb",
                    "abbabba",
                    "abbaabb",
                    "abababc",
                    "abbbbbba",
                    "aabb",
                    "aAbB",
                    "µaΜAµa",
                    "\n\n",
                    " ababab ",
                    "\u{2028}ab\u{2029}",
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
    fn reference_sequence_retries_preserve_prefix_captures_and_enclosures() {
        for (source, input, range, captures) in [
            (
                r"(a)(b)(?:\1\2)*ab$",
                "ababab",
                0..6,
                vec![Some(0..1), Some(1..2)],
            ),
            (
                r"((a)(b)(?:\2\3)+?)\1",
                "abababab",
                0..8,
                vec![Some(0..4), Some(0..1), Some(1..2)],
            ),
            (
                r"(?:(a)|(b))(?:\1\2)+c",
                "bbc",
                0..3,
                vec![None, Some(0..1)],
            ),
            (r"(?:\1\2)+(a)(b)", "ab", 0..2, vec![Some(0..1), Some(1..2)]),
            (r"()()(?:\1\2)*", "", 0..0, vec![Some(0..0), Some(0..0)]),
        ] {
            let matcher =
                RegExpBackreferenceMatcher::compile(&JsString::from(source), false).unwrap();
            let found = matcher.find(&JsString::from(input), 0, true).unwrap();
            assert_eq!(found.range, range, "{source}");
            assert_eq!(&*found.captures, &*captures, "{source}");
        }
    }

    #[test]
    fn long_reference_sequences_freeze_ranges_compactly_and_skip_finite_empty_work() {
        let source = JsString::from(format!("(a)(b)(?:{})+", r"\1\2".repeat(50000)).as_str());
        let matcher = RegExpBackreferenceMatcher::compile(&source, false).unwrap();
        assert!(matcher.0.instructions.len() <= source.len() * 2);
        let input = JsString::from("ab".repeat(50001).as_str());
        let found = matcher.find(&input, 0, true).unwrap();
        assert_eq!(found.range, 0..100002);
        assert_eq!(&*found.captures, &[Some(0..1), Some(1..2)]);
        assert!(matches!(
            matcher.find_with_work(&input, 0, true, |_| Err::<(), _>("work")),
            Err("work")
        ));
        let source = JsString::from(format!("()()(?:\\1\\2){{{}}}", "9".repeat(10000)).as_str());
        let matcher = RegExpBackreferenceMatcher::compile(&source, false).unwrap();
        let mut work = 0;
        let found = matcher
            .find_with_work(&JsString::from(""), 0, true, |n| {
                work += n;
                Ok::<_, ()>(())
            })
            .unwrap()
            .unwrap();
        assert!(found.captures.iter().all(|r| *r == Some(0..0)));
        assert!(work < 150, "empty work {work}");
        for source in [
            r"(?:(a)(b)(\1\2)+){2}",
            r"(?:(a)(b)(?:()\1\2)+){2}",
            r"(?:(a)(b)(?:a\1\2)+){2}",
            r"(a)(b)(?:\1|\2)+",
            r"(a)(b)(?:\1+\2)+",
            r"(a)(?:\1\2)+",
        ] {
            let mut work = 0;
            assert!(
                RegExpBackreferenceMatcher::compile_with_work(
                    &JsString::from(source),
                    false,
                    |n| {
                        work += n;
                        Ok::<_, ()>(())
                    }
                )
                .unwrap()
                .is_none(),
                "{source}"
            );
            assert_eq!(work, 0, "{source}");
        }
    }
    #[test]
    fn repeated_reference_body_execution_snapshot() {
        let mut rows = String::new();
        for source in [
            r"(a)(?:\1\1)*",
            r"(a)(?:\1\1)*?",
            r"(a)(?:\1\1)+",
            r"(a)(?:\1\1)+?",
            r"(a)(?:\1\1)?",
            r"(a)(?:\1\1)??",
            r"(ab)(?:\1\1){2,3}",
            r"(ab)(?:\1\1){1,3}?",
            r"(a)(?:\1\1)*ab",
            r"(a)(?:\1\1)*?ab",
            r"((a)(\2\2)+)\1",
            r"((a)(\2\2)+?)\1",
            r"(a|ab)(?:\1\1)+b",
            r"(?:(a)|(ab))(?:\1\1)*(?:\2\2)+",
            r"(a)(b)(?:\1\1)*(?:\2\2)*",
            r"(a)(b)(?:\1\1)*?(?:\2\2)*?",
            r"(?:\1\1)+(a)",
            r"(\1\1)+a\1",
            r"()(?:\1\1){2,3}",
            r"(a)(?:\1\1){999999999999999999999999999999}",
            r"(a)(?:\1\1){0,999999999999999999999999999999}?b",
            r"^([µ])(?:\1\1\1)+$",
            r"(a)(()\1\1())+\2",
            r"(.)((?:\1)\1){1,2}a|()(?:\3\3)+",
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
                    "a",
                    "aa",
                    "aaaaaa",
                    "ababab",
                    "abababab",
                    "aab",
                    "aaaab",
                    "ababb",
                    "aaabbb",
                    "AaAa",
                    "µΜµ",
                    "\n\n",
                    " aaa ",
                    "\u{2028}aa\u{2029}",
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
    fn repeated_reference_counts_preserve_whole_body_steps_and_capture_retries() {
        for (source, input, range, captures) in [
            (
                r"(a)(\1\1)+b",
                "aaaaaaab",
                0..8,
                vec![Some(0..1), Some(5..7)],
            ),
            (
                r"(a)(()\1\1())+\2",
                "aaaaaa",
                0..5,
                vec![Some(0..1), Some(1..3), Some(1..1), Some(3..3)],
            ),
            (r"(a)(\1\1)*?a$", "aaaa", 0..4, vec![Some(0..1), Some(1..3)]),
            (r"(a)(\1\1)*?a", "aaaa", 0..2, vec![Some(0..1), None]),
            (r"()(\1\1)*", "", 0..0, vec![Some(0..0), None]),
            (r"()(\1\1)+", "", 0..0, vec![Some(0..0), Some(0..0)]),
        ] {
            let matcher =
                RegExpBackreferenceMatcher::compile(&JsString::from(source), false).unwrap();
            let found = matcher.find(&JsString::from(input), 0, true).unwrap();
            assert_eq!(found.range, range, "{source}");
            assert_eq!(&*found.captures, &*captures, "{source}");
        }
        assert!(
            RegExpBackreferenceMatcher::compile(&JsString::from(r"^(a)(?:\1\1)+$"), false)
                .unwrap()
                .find(&JsString::from("aaaa"), 0, true)
                .is_none()
        );
    }

    #[test]
    fn large_repeated_reference_bodies_keep_multiplicity_compact_and_bounds_exact() {
        let source = JsString::from(format!("(a)({})+", r"\1".repeat(100000)).as_str());
        let matcher = RegExpBackreferenceMatcher::compile(&source, false).unwrap();
        let found = matcher
            .find(&JsString::from("a".repeat(100001).as_str()), 0, true)
            .unwrap();
        assert_eq!(found.range, 0..100001);
        assert_eq!(&*found.captures, &[Some(0..1), Some(1..100001)]);
        assert!(matches!(
            matcher.find_with_work(
                &JsString::from("a".repeat(100001).as_str()),
                0,
                true,
                |_| Err::<(), _>("work")
            ),
            Err("work")
        ));
        let source = JsString::from(format!("(a)({}){{0}}", r"\1".repeat(100000)).as_str());
        let found = RegExpBackreferenceMatcher::compile(&source, false)
            .unwrap()
            .find(&JsString::from("a"), 0, true)
            .unwrap();
        assert_eq!(&*found.captures, &[Some(0..1), None]);
        let source = JsString::from(format!("()(()\\1\\1()){{{}}}", "9".repeat(10000)).as_str());
        let matcher = RegExpBackreferenceMatcher::compile(&source, false).unwrap();
        let mut work = 0;
        let found = matcher
            .find_with_work(&JsString::from(""), 0, true, |n| {
                work += n;
                Ok::<_, ()>(())
            })
            .unwrap()
            .unwrap();
        assert!(found.captures.iter().all(|r| *r == Some(0..0)));
        assert!(work < 150, "empty work {work}");
        for source in [
            r"(?:(a)(b)(?:\1\2)+){2}",
            r"(?:(a)(?:a\1\1)+){2}",
            r"(a)(?:\1|\1)+",
            r"(a)(?:\1+\1)+",
            r"(?:(a)(?:a\1()\1)+){2}",
            r"(?:(a)(?:(\1)\1)+){2}",
        ] {
            let mut work = 0;
            assert!(
                RegExpBackreferenceMatcher::compile_with_work(
                    &JsString::from(source),
                    false,
                    |n| {
                        work += n;
                        Ok::<_, ()>(())
                    }
                )
                .unwrap()
                .is_none(),
                "{source}"
            );
            assert_eq!(work, 0, "{source}");
        }
    }
    #[test]
    fn empty_capture_quantified_reference_execution_snapshot() {
        let mut rows = String::new();
        for source in [
            r"(a)(()\1)*",
            r"(a)(()\1)*?",
            r"(a)(\1())+",
            r"(a)(\1())+?",
            r"(a)(()\1())?",
            r"(a)(()\1())??",
            r"(ab)(()(\1)()){2,3}",
            r"(ab)(()(\1)()){1,3}?",
            r"(a)(()\1())*ab",
            r"(a)(()\1())*?ab",
            r"((a)(()\2())+)\1",
            r"((a)(()\2())+?)\1",
            r"(a|ab)(()\1())+b",
            r"(?:(a)|(ab))(()\1())*(\2())+",
            r"(a)(b)(()\1())*(\2())*",
            r"(a)(b)(()\1())*?(\2())*?",
            r"(()\4())+(a)",
            r"(()\2())+a\1",
            r"()(()\1()){2,3}",
            r"(a)(()\1()){999999999999999999999999999999}",
            r"(a)(\1()){0,999999999999999999999999999999}?b",
            r"^([µ])(()\1())+$",
            r"(a)(()\1())+\2",
            r"(.)(()\1()){1,2}a|()(()\5())+",
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
                    "a",
                    "aa",
                    "aaaaaa",
                    "ababab",
                    "abababab",
                    "aab",
                    "aaaab",
                    "ababb",
                    "aaabbb",
                    "AaAa",
                    "µΜµ",
                    "\n\n",
                    " aaa ",
                    "\u{2028}aa\u{2029}",
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
    fn empty_sibling_captures_track_the_final_iteration_and_zero_counts() {
        for (source, input, range, captures) in [
            (
                r"(a)(()\1())+\2",
                "aaaa",
                0..4,
                vec![Some(0..1), Some(2..3), Some(2..2), Some(3..3)],
            ),
            (r"()(()\1())*", "", 0..0, vec![Some(0..0), None, None, None]),
            (
                r"()(()\1())+",
                "",
                0..0,
                vec![Some(0..0), Some(0..0), Some(0..0), Some(0..0)],
            ),
            (
                r"(()\2())+a\1",
                "a",
                0..1,
                vec![Some(0..0), Some(0..0), Some(0..0)],
            ),
            (
                r"((\1)())+",
                "",
                0..0,
                vec![Some(0..0), Some(0..0), Some(0..0)],
            ),
        ] {
            let matcher =
                RegExpBackreferenceMatcher::compile(&JsString::from(source), false).unwrap();
            let found = matcher.find(&JsString::from(input), 0, true).unwrap();
            assert_eq!(found.range, range, "{source}");
            assert_eq!(&*found.captures, &*captures, "{source}");
        }
    }

    #[test]
    fn many_empty_siblings_and_huge_required_minimums_keep_compact_capture_spans() {
        let source = JsString::from(
            format!("(a)({}\\1{})+", "()".repeat(50000), "()".repeat(50000)).as_str(),
        );
        let matcher = RegExpBackreferenceMatcher::compile(&source, false).unwrap();
        let found = matcher.find(&JsString::from("aaaa"), 0, true).unwrap();
        assert_eq!(found.range, 0..4);
        assert_eq!(found.captures.len(), 100002);
        assert_eq!(found.captures[1], Some(3..4));
        assert!(found.captures[2..50002].iter().all(|r| *r == Some(3..3)));
        assert!(found.captures[50002..].iter().all(|r| *r == Some(4..4)));
        let source = JsString::from(format!("()(()\\1()){{{}}}", "9".repeat(10000)).as_str());
        let matcher = RegExpBackreferenceMatcher::compile(&source, false).unwrap();
        let mut work = 0;
        let found = matcher
            .find_with_work(&JsString::from(""), 0, true, |n| {
                work += n;
                Ok::<_, ()>(())
            })
            .unwrap()
            .unwrap();
        assert!(found.captures.iter().all(|r| *r == Some(0..0)));
        assert!(work < 150, "empty work {work}");
        assert!(matches!(
            matcher.find_with_work(&JsString::from(""), 0, true, |_| Err::<(), _>("work")),
            Err("work")
        ));
        for source in [
            r"(?:(a)(()a\1())+){2}",
            r"(?:(a)(()a\1\1())+){2}",
            r"(a)(()\1|b())+",
            r"(a)(()\1*())+",
            r"(a)(()\1(?=a))+",
        ] {
            let mut work = 0;
            assert!(
                RegExpBackreferenceMatcher::compile_with_work(
                    &JsString::from(source),
                    false,
                    |n| {
                        work += n;
                        Ok::<_, ()>(())
                    }
                )
                .unwrap()
                .is_none(),
                "{source}"
            );
            assert_eq!(work, 0, "{source}");
        }
    }
    #[test]
    fn captured_quantified_reference_execution_snapshot() {
        let mut rows = String::new();
        for source in [
            r"(a)(\1)*",
            r"(a)(\1)*?",
            r"(a)(\1)+",
            r"(a)(\1)+?",
            r"(a)(\1)?",
            r"(a)(\1)??",
            r"(ab)((?:\1)){2,3}",
            r"(ab)((?:\1)){1,3}?",
            r"(a)((?:)\1(?:))*ab",
            r"(a)((?:)\1(?:))*?ab",
            r"((a)(\2)+)\1",
            r"((a)((?:\2))+?)\1",
            r"(a|ab)(\1)+b",
            r"(?:(a)|(ab))(\1)*(\2)+",
            r"(a)(b)((\1))*((?:\2))*",
            r"(a)(b)((\1))*?((?:\2))*?",
            r"(\2)+(a)",
            r"(\1)+a\1",
            r"()(\1){2,3}",
            r"(a)(\1){999999999999999999999999999999}",
            r"(a)(\1){0,999999999999999999999999999999}?b",
            r"^([µ])(\1)+$",
            r"(a)(\1)+\2",
            r"(.)(\1){1,2}a|()(\3)+",
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
                    "a",
                    "aa",
                    "aaaaaa",
                    "ababab",
                    "abababab",
                    "aab",
                    "aaaab",
                    "ababb",
                    "aaabbb",
                    "AaAa",
                    "µΜµ",
                    "\n\n",
                    " aaa ",
                    "\u{2028}aa\u{2029}",
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
    fn captured_reference_retries_restore_final_iterations_and_enclosing_ranges() {
        for (source, input, range, captures) in [
            (r"(a)(\1)+\2", "aaaa", 0..4, vec![Some(0..1), Some(2..3)]),
            (
                r"((a)(\2)+?)\1",
                "aaaaaa",
                0..4,
                vec![Some(0..2), Some(0..1), Some(1..2)],
            ),
            (r"(a)(\1)*?b|a", "a", 0..1, vec![None, None]),
            (r"()(\1)*", "", 0..0, vec![Some(0..0), None]),
            (r"()(\1)+", "", 0..0, vec![Some(0..0), Some(0..0)]),
            (r"(\1)+", "", 0..0, vec![Some(0..0)]),
            (r"(\2)+(a)", "a", 0..1, vec![Some(0..0), Some(0..1)]),
        ] {
            let matcher =
                RegExpBackreferenceMatcher::compile(&JsString::from(source), false).unwrap();
            let found = matcher.find(&JsString::from(input), 0, true).unwrap();
            assert_eq!(found.range, range, "{source}");
            assert_eq!(&*found.captures, &*captures, "{source}");
        }
    }

    #[test]
    fn deep_captured_wrappers_and_huge_empty_minimums_use_flat_capture_storage() {
        let source =
            JsString::from(format!("(a){}\\1{}+", "(".repeat(100000), ")".repeat(100000)).as_str());
        let matcher = RegExpBackreferenceMatcher::compile(&source, false).unwrap();
        let found = matcher.find(&JsString::from("aaaa"), 0, true).unwrap();
        assert_eq!(found.range, 0..4);
        assert_eq!(found.captures[0], Some(0..1));
        assert!(found.captures[1..].iter().all(|r| *r == Some(3..4)));
        let source = JsString::from(format!("()((?:)\\1(?:)){{{}}}", "9".repeat(10000)).as_str());
        let matcher = RegExpBackreferenceMatcher::compile(&source, false).unwrap();
        let mut work = 0;
        let found = matcher
            .find_with_work(&JsString::from(""), 0, true, |n| {
                work += n;
                Ok::<_, ()>(())
            })
            .unwrap()
            .unwrap();
        assert_eq!(&*found.captures, &[Some(0..0), Some(0..0)]);
        assert!(work < 100, "empty iteration work {work}");
        assert!(matches!(
            matcher.find_with_work(&JsString::from(""), 0, true, |_| Err::<(), _>("work")),
            Err("work")
        ));
        for source in [
            r"(?:(a)(a\1\1)+){2}",
            r"(?:(a)(a\1)+){2}",
            r"(a)(\1|b)+",
            r"(a)(\1*)+",
            r"(?:(a)(()a\1)+){2}",
            r"(?:(a)(\1a())+){2}",
            r"(a)(\1(?=a))+",
        ] {
            let mut work = 0;
            assert!(
                RegExpBackreferenceMatcher::compile_with_work(
                    &JsString::from(source),
                    false,
                    |n| {
                        work += n;
                        Ok::<_, ()>(())
                    }
                )
                .unwrap()
                .is_none(),
                "{source}"
            );
            assert_eq!(work, 0, "{source}");
        }
    }
    #[test]
    fn wrapped_quantified_reference_execution_snapshot() {
        let mut rows = String::new();
        for source in [
            r"(a)(?:\1)*",
            r"(a)(?:\1)*?",
            r"(a)(?:\1)+",
            r"(a)(?:\1)+?",
            r"(a)(?:\1)?",
            r"(a)(?:\1)??",
            r"(ab)(?:(?:\1)){2,3}",
            r"(ab)(?:(?:\1)){1,3}?",
            r"(a)(?:(?:)\1(?:))*ab",
            r"(a)(?:(?:)\1(?:))*?ab",
            r"((a)(?:\2)+)\1",
            r"((a)(?:(?:\2))+?)\1",
            r"(a|ab)(?:\1)+b",
            r"(?:(a)|(ab))(?:\1)*(?:\2)+",
            r"(a)(b)(?:\1)*(?:\2)*",
            r"(a)(b)(?:\1)*?(?:\2)*?",
            r"(?:\1)+(a)",
            r"((?:(?:\1))+a)\1",
            r"()(?:\1){2,3}",
            r"(a)(?:\1){999999999999999999999999999999}",
            r"(a)(?:\1){0,999999999999999999999999999999}?b",
            r"^([µ])(?:\1)+$",
            r"\b(\w)(?:\1)+\b",
            r"(.)(?:\1){1,2}a|()(?:\2)+",
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
                    "a",
                    "aa",
                    "aaaaaa",
                    "ababab",
                    "abababab",
                    "aab",
                    "aaaab",
                    "ababb",
                    "aaabbb",
                    "AaAa",
                    "µΜµ",
                    "\n\n",
                    " aaa ",
                    "\u{2028}aa\u{2029}",
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
    fn wrapped_reference_retries_restore_enclosing_captures_and_keep_source_numbering() {
        let matcher =
            RegExpBackreferenceMatcher::compile(&JsString::from(r"((a)(?:(?:\2))+?)\1"), false)
                .unwrap();
        let found = matcher.find(&JsString::from("aaaaaa"), 0, true).unwrap();
        assert_eq!(found.range, 0..4);
        assert_eq!(&*found.captures, &[Some(0..2), Some(0..1)]);
        let matcher = RegExpBackreferenceMatcher::compile(
            &JsString::from(r"(?:(a)|(ab))(?:(?:)\1)*?(?:\2)+$"),
            false,
        )
        .unwrap();
        let found = matcher.find(&JsString::from("abab"), 0, true).unwrap();
        assert_eq!(found.range, 0..4);
        assert_eq!(&*found.captures, &[None, Some(0..2)]);
    }

    #[test]
    fn deep_transparent_wrappers_keep_flat_storage_and_empty_targets_finite() {
        let source = JsString::from(
            format!("(a){}\\1{}+", "(?:".repeat(100000), ")".repeat(100000)).as_str(),
        );
        let matcher = RegExpBackreferenceMatcher::compile(&source, false).unwrap();
        assert!(matcher.0.instructions.len() <= source.len() * 2);
        assert_eq!(
            matcher
                .find(&JsString::from("aaaa"), 0, true)
                .unwrap()
                .range,
            0..4
        );
        let source = JsString::from(format!("()(?:(?:)\\1(?:)){{{}}}", "9".repeat(10000)).as_str());
        let matcher = RegExpBackreferenceMatcher::compile(&source, false).unwrap();
        let found = matcher.find(&JsString::from(""), 0, true).unwrap();
        assert_eq!(found.range, 0..0);
        assert_eq!(&*found.captures, &[Some(0..0)]);
        for source in [
            r"(?:(a)(a\1)+){2}",
            r"(?:(a)(?:a\1\1)+){2}",
            r"(?:(a)(?:a\1)+){2}",
            r"(a)(?:\1|b)+",
            r"(a)(?:\1*)+",
            r"(a)(?:\1(?=a))+",
            r"(?:(a)(?:a(\1))+){2}",
        ] {
            let mut work = 0;
            assert!(
                RegExpBackreferenceMatcher::compile_with_work(
                    &JsString::from(source),
                    false,
                    |n| {
                        work += n;
                        Ok::<_, ()>(())
                    }
                )
                .unwrap()
                .is_none(),
                "{source}"
            );
            assert_eq!(work, 0, "{source}");
        }
    }

    #[test]
    fn quantified_reference_execution_snapshot() {
        let mut rows = String::new();
        for source in [
            r"(a)\1*",
            r"(a)\1*?",
            r"(a)\1+",
            r"(a)\1+?",
            r"(a)\1?",
            r"(a)\1??",
            r"(ab)\1{2,3}",
            r"(ab)\1{1,3}?",
            r"(a)\1*ab",
            r"(a)\1*?ab",
            r"((a)\2+)\1",
            r"((a)\2+?)\1",
            r"(a|ab)\1+b",
            r"(?:(a)|(ab))\1*\2+",
            r"(a)(b)\1*\2*",
            r"(a)(b)\1*?\2*?",
            r"\1+(a)",
            r"(\1+a)\1",
            r"()\1{999999999999999999999999999999}",
            r"(a)\1{999999999999999999999999999999}",
            r"(a)\1{0,999999999999999999999999999999}?b",
            r"^([µ])\1+$",
            r"\b(\w)\1+\b",
            r"(.)\1{1,2}a|()\2+",
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
                    "a",
                    "aa",
                    "aaaaaa",
                    "ababab",
                    "abababab",
                    "aab",
                    "aaaab",
                    "ababb",
                    "aaabbb",
                    "AaAa",
                    "µΜµ",
                    "\n\n",
                    " aaa ",
                    "\u{2028}aa\u{2029}",
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
    fn greedy_and_lazy_reference_retries_restore_completed_enclosing_captures() {
        let matcher =
            RegExpBackreferenceMatcher::compile(&JsString::from(r"((a)\2+)\1"), false).unwrap();
        let found = matcher.find(&JsString::from("aaaaaa"), 0, true).unwrap();
        assert_eq!(found.range, 0..6);
        assert_eq!(&*found.captures, &[Some(0..3), Some(0..1)]);
        let matcher =
            RegExpBackreferenceMatcher::compile(&JsString::from(r"((a)\2+?)\1"), false).unwrap();
        let found = matcher.find(&JsString::from("aaaaaa"), 0, true).unwrap();
        assert_eq!(found.range, 0..4);
        assert_eq!(&*found.captures, &[Some(0..2), Some(0..1)]);
        let matcher =
            RegExpBackreferenceMatcher::compile(&JsString::from(r"(a)\1*?ab"), false).unwrap();
        let found = matcher.find(&JsString::from("qaaaab"), 0, false).unwrap();
        assert_eq!(found.range, 1..6);
        assert_eq!(&*found.captures, &[Some(1..2)]);
    }

    #[test]
    fn empty_targets_huge_bounds_many_quantified_references_and_optional_work_stay_finite() {
        let source = JsString::from(format!("()\\1{{{}}}", "9".repeat(10000)).as_str());
        let matcher = RegExpBackreferenceMatcher::compile(&source, false).unwrap();
        let mut work = 0;
        let found = matcher
            .find_with_work(&JsString::from(""), 0, true, |n| {
                work += n;
                Ok::<_, ()>(())
            })
            .unwrap()
            .unwrap();
        assert_eq!(found.range, 0..0);
        assert_eq!(&*found.captures, &[Some(0..0)]);
        assert!(work < 100, "actual empty-reference work {work}");
        let source = JsString::from(format!("(a){}b", r"\1*?".repeat(100000)).as_str());
        let matcher = RegExpBackreferenceMatcher::compile(&source, false).unwrap();
        assert!(matcher.0.instructions.len() <= source.len() * 2);
        let found = matcher.find(&JsString::from("ab"), 0, true).unwrap();
        assert_eq!(found.range, 0..2);
        assert_eq!(&*found.captures, &[Some(0..1)]);
        let mut work = 0;
        assert_eq!(
            matcher
                .find_with_work(&JsString::from("aa"), 0, true, |n| {
                    work += n;
                    if work > 350000 { Err("host") } else { Ok(()) }
                })
                .unwrap_err(),
            "host"
        );
    }

    #[test]
    fn missing_named_escape_metadata_and_quantified_bodies_remain_unsupported_before_charge() {
        let source = JsString::from(r"(a)\k<x>\k<x>");
        let references = named_escapes(&source);
        let mut work = 0;
        assert!(
            RegExpBackreferenceMatcher::compile_with_named_bindings_and_work(
                &source,
                false,
                false,
                false,
                RegExpBackreferenceNamedBindings {
                    groups: &[&[0]],
                    references: &references[..1]
                },
                |n| {
                    work += n;
                    Ok::<_, ()>(())
                }
            )
            .unwrap()
            .is_none()
        );
        assert_eq!(work, 0);
        for source in [
            r"(?:(?:(a)\1){2}){2}",
            r"(?:(a)(?:\1)+){2}",
            r"(?:(a)+\1){2}",
            r"(?=(a))\1+",
            r"(a)\2+",
        ] {
            let mut work = 0;
            assert!(
                RegExpBackreferenceMatcher::compile_with_work(
                    &JsString::from(source),
                    false,
                    |n| {
                        work += n;
                        Ok::<_, ()>(())
                    }
                )
                .unwrap()
                .is_none(),
                "{source}"
            );
            assert_eq!(work, 0, "{source}");
        }
    }

    #[test]
    fn nested_alternative_reference_execution_snapshot() {
        let mut rows = String::new();
        for source in [
            r"(?:(a|ab)|b)\1",
            r"(?:a|(b|bc))\1",
            r"((a|ab)(c|)|e)\1",
            r"((a)|(ab|abc))\1\2\3",
            r"(?:(?:(a)|(ab))|(b))\1\2\3",
            r"(q)(?:(a|ab)|b)(c|)\1\2\3",
            r"(?:(?:(a)|)|(?:(b)|))\1\2",
            r"((a|)|(b|))\1",
            r"(?:(a\1|b)|c)\1",
            r"((\1a|b)|c)\1",
            r"(?:(\w|\W)|(\d|\D))\1\2",
            r"([^µ]|(?:[ab]|µ))\1",
            r"\b((a|\w)|b)\1\b",
            r"((a|ab)|c)(b|(?:c|))\1\3",
            r"(?:(a|ab)|b)\1|(c|cd)\2",
            r"(?:(a|ab)|b)\1|",
            r"((?:(a)|b)|())\1\2\3",
            r"x(((a)|(?:b|c)))\2y",
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
                    "bb",
                    "abab",
                    "abcabc",
                    "bcc",
                    "qababqab",
                    "qabab",
                    "aabb",
                    "AaBa",
                    "µΜ",
                    "\n\n",
                    " bb ",
                    "\u{2028}abab\u{2029}",
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
    fn nested_branch_failures_restore_shared_and_enclosing_capture_ranges() {
        let matcher = RegExpBackreferenceMatcher::compile(
            &JsString::from(r"(?:(?:(a)|(ab))|(b))\1\2\3"),
            false,
        )
        .unwrap();
        let found = matcher.find(&JsString::from("qabab"), 0, false).unwrap();
        assert_eq!(found.range, 1..5);
        assert_eq!(&*found.captures, &[None, Some(1..3), None]);
        let matcher =
            RegExpBackreferenceMatcher::compile(&JsString::from(r"((a|ab)(c|)|e)\1"), false)
                .unwrap();
        let found = matcher.find(&JsString::from("abab"), 0, true).unwrap();
        assert_eq!(found.range, 0..4);
        assert_eq!(&*found.captures, &[Some(0..2), Some(0..2), Some(2..2)]);
        let found = matcher.find(&JsString::from("ee"), 0, true).unwrap();
        assert_eq!(found.range, 0..2);
        assert_eq!(&*found.captures, &[Some(0..1), None, None]);
    }

    #[test]
    fn nested_name_participation_accepts_exclusive_slots_and_rejects_coexisting_slots() {
        for (text, slots, expected) in [
            (r"(?:(?:(a)|(ab))|(b))\k<x>", &[0, 1, 2][..], true),
            (r"(?:(a)(b)|(c))\k<x>", &[0, 1][..], false),
            (r"((a)|(b|c))\k<x>", &[0, 1][..], false),
            (r"(?:(a|ab)|(b))\k<x>", &[0, 1][..], true),
            (r"(?:(?:(a)|(b))(c)|(d))\k<x>", &[0, 2, 3][..], false),
        ] {
            let source = JsString::from(text);
            let references = named_escapes(&source);
            let mut work = 0;
            let matcher = RegExpBackreferenceMatcher::compile_with_named_bindings_and_work(
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
            .unwrap();
            assert_eq!(matcher.is_some(), expected, "{text}");
            if !expected {
                assert_eq!(work, 0, "{text}");
            }
        }
    }

    #[test]
    fn deeply_nested_choices_keep_flat_storage_failure_unwinding_and_optional_work() {
        let source =
            JsString::from(format!("{}a{}\\1", "(".repeat(100000), "|b)".repeat(100000)).as_str());
        let matcher = RegExpBackreferenceMatcher::compile(&source, false).unwrap();
        assert!(matcher.0.instructions.len() <= source.len() * 2 * 2);
        let found = matcher.find(&JsString::from("aa"), 0, true).unwrap();
        assert_eq!(found.range, 0..2);
        assert!(found.captures.iter().all(|r| *r == Some(0..1)));
        assert!(matcher.find(&JsString::from("x"), 0, true).is_none());
        let mut work = 0;
        assert_eq!(
            matcher
                .find_with_work(&JsString::from("aa"), 0, true, |n| {
                    work += n;
                    if work > 350000 { Err("host") } else { Ok(()) }
                })
                .unwrap_err(),
            "host"
        );
    }

    #[test]
    fn sequential_alternative_reference_execution_snapshot() {
        let mut rows = String::new();
        for source in [
            r"(a|ab)(b|)\1\2",
            r"(ab|a)(a|b)\1\2",
            r"((a)|(ab))((b)|())\2\3\5\6",
            r"(q)(a|ab)(b|)\1\2\3",
            r"(?:(a)|(b))\1\2(?:(a)|(b))\3\4",
            r"((a|ab)(b|))\1",
            r"(?:(a)|)(?:|(b))\1\2",
            r"\2(a|b)\1(c|)\2",
            r"(a|b)((\1a)|b)\2",
            r"^(\w|\W)(\d|\D)\1\2$",
            r"([^µ]|[ab])(µ|.)\1\2",
            r"\b(a|\w)(b|\w)\1\2\b",
            r"()(|a)(|b)\1\2\3",
            r"x(((a)|b))(?:(c)|(d))\2\4\5y",
            r"(a|ab)(b|)\1\2|(b)\3",
            r"(a|ab)(b|)\1\2|",
            r"(|(a))((b)|)\2\4",
            r"(?:(a\1)|(b\2))(?:(a\3)|(b\4))\1\2\3\4",
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
                    "bb",
                    "abab",
                    "abba",
                    "ababb",
                    "qababqabab",
                    "qabab",
                    "aabb",
                    "AaBa",
                    "µΜµΜ",
                    "\n\n\n\n",
                    " ab ab ",
                    "\u{2028}abab\u{2029}",
                    "\r\n\r\n",
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
    fn later_choices_restore_prior_branches_common_captures_and_enclosing_ranges() {
        let matcher = RegExpBackreferenceMatcher::compile(
            &JsString::from(r"((a)|(ab))((c)|())\2\3\5\6"),
            false,
        )
        .unwrap();
        let found = matcher.find(&JsString::from("qabab"), 0, false).unwrap();
        assert_eq!(found.range, 1..5);
        assert_eq!(
            &*found.captures,
            &[Some(1..3), None, Some(1..3), Some(3..3), None, Some(3..3)]
        );
        let matcher =
            RegExpBackreferenceMatcher::compile(&JsString::from(r"((a|ab)(b|))\1"), false).unwrap();
        let found = matcher.find(&JsString::from("abab"), 0, true).unwrap();
        assert_eq!(found.range, 0..4);
        assert_eq!(&*found.captures, &[Some(0..2), Some(0..1), Some(1..2)]);
        let matcher =
            RegExpBackreferenceMatcher::compile(&JsString::from(r"(a|ab)(b|)\1\2"), false).unwrap();
        let found = matcher.find(&JsString::from("ababb"), 0, true).unwrap();
        assert_eq!(found.range, 0..4);
        assert_eq!(&*found.captures, &[Some(0..1), Some(1..2)]);
    }

    #[test]
    fn sequential_choice_name_owners_reject_coexisting_slots_before_charging() {
        for (text, slots) in [
            (r"(?:(a)|(b))(?:(a)|(b))\k<x>", &[0, 2][..]),
            (r"(q)(?:(a)|(b))(?:(a)|(b))\k<x>", &[0, 2][..]),
            (r"(?:(a)|(b))(?:(a)|(b))\k<x>", &[0, 1, 2, 3][..]),
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
            assert_eq!(work, 0);
        }
    }

    #[test]
    fn many_sequential_choices_keep_flat_capture_checkpoints_and_fallible_work() {
        let source = JsString::from(format!("{}\\1", "(a|b)".repeat(100000)).as_str());
        let matcher = RegExpBackreferenceMatcher::compile(&source, false).unwrap();
        assert!(matcher.0.instructions.len() <= source.len() * 2);
        let input = JsString::from("a".repeat(100001).as_str());
        let found = matcher.find(&input, 0, true).unwrap();
        assert_eq!(found.range, 0..100001);
        assert_eq!(found.captures[99999], Some(99999..100000));
        let mut work = 0;
        assert_eq!(
            matcher
                .find_with_work(&input, 0, true, |n| {
                    work += n;
                    if work > 350000 { Err("host") } else { Ok(()) }
                })
                .unwrap_err(),
            "host"
        );
        for source in [
            r"(?:(?:(a|b)|c)(?:\1)+){2}",
            r"(?:(?:a|(b|c))(?:\1)+){2}",
            r"(?:((a|b)(c|d)|e)(?:\1)+){2}",
            r"(?:(a|b)(c|d)(?:\1)+){2}",
        ] {
            let mut work = 0;
            assert!(
                RegExpBackreferenceMatcher::compile_with_work(
                    &JsString::from(source),
                    false,
                    |n| {
                        work += n;
                        Ok::<_, ()>(())
                    }
                )
                .unwrap()
                .is_none(),
                "{source}"
            );
            assert_eq!(work, 0);
        }
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
        assert!(matcher.0.instructions.len() <= source.len() * 2);
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
        for text in [
            r"(?:(\w)(?:\1)+){2}",
            r"(?:^([ab])(?:\1)+){2}",
            r"(?:([ab]|c)(?:\1)+){2}",
        ] {
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
            r"(?:(a)(?:\1)+){2}",
            r"(?:(a|b)(?:\1)+){2}",
            r"(?<x>a)\k<x>",
            r"(?:([ab])(?:\1)+){2}",
            r"(?:^(a)(?:\1)+){2}",
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
