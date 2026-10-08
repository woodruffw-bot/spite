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
/// Repeated choices accept nested/sequential capturing branches when every
/// complete body path consumes a unit, with flat source-order iteration retries.
/// Deterministic inner quantifiers compose with these body paths.
/// Fixed lookbehind also accepts proven nested zero-width repetitions with
/// required empty capture effects and skipped optional slots.
/// Branch loops nest through flat parent-linked contexts. Body captures reset
/// per iteration, and retries restore previous starts, ranges and named aliases.
/// Positive and negative lookahead use flat atomic assertion contexts. Positive
/// success retains captures; negative assertions restore the input state.
/// Transparent zero-width assertion and alternative bodies execute required repetitions once
/// and skip optional zero-progress iterations with undefined capture slots.
/// Fixed positive and negative lookbehind preserve full input
/// context for character sequences, their exact counts, boundary assertions and
/// transparent groups, captures, equal-width alternatives and nested fixed lookbehind.
/// Counted predicate captures retain their leftmost backward iteration; required
/// pure boundary and empty counts accept variable bounds without expanded iterations.
/// Optional zero-progress iterations leave repeated captures undefined.
/// Zero-width sequence counts and proven empty local references nest through
/// the same flat wrapper plan.
/// Counted lookbehind accepts same-body references when its complete repeated
/// unit and every capture are proved empty in either direction.
/// Exact-zero native reference counts skip all reads and retain undefined owned
/// captures even when their unexecuted unit could consume input.
/// Exact-zero prepared progressing choices also compose with fixed lookbehind
/// and enclosing pure-zero repetitions without entering a branch.
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
    SkipGroup {
        captures: Range<usize>,
        body: usize,
        end: usize,
    },
    RepeatChoice {
        captures: Range<usize>,
        open: Option<usize>,
        body: usize,
        head: usize,
        end: usize,
        bounds: Bounds,
    },
    RepeatChoiceEnd(usize),
    RepeatZeroWidth {
        captures: Range<usize>,
        open: Option<usize>,
        body: usize,
        head: usize,
        end: usize,
        required: bool,
    },
    Lookahead {
        body: usize,
        head: usize,
        end: usize,
        negative: bool,
        fixed_width: Option<usize>,
    },
    LookaheadEnd(usize),
    Lookbehind {
        body: usize,
        width: LookbehindWidth,
        negative: bool,
        alternatives: Box<[usize]>,
    },
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
        // Only fixed lookbehind with zero-width terms needs stored unit offsets.
        lookbehind_offsets: Option<Box<[usize]>>,
    },
    QuantifiedReference {
        index: usize,
        named: bool,
        bounds: Bounds,
        copies: usize,
        captures: Box<[(usize, ReferenceCaptureSpan)]>,
    },
}

#[derive(Debug)]
enum LookbehindWidth {
    Fixed(usize),
    OutsideReferences {
        // None denotes a consuming width larger than any native input.
        fixed: Option<usize>,
        references: Box<[OutsideReferenceWidth]>,
    },
}

#[derive(Debug)]
struct OutsideReferenceWidth {
    index: usize,
    named: bool,
    copies: usize,
    // None denotes an unrepresentable required minimum, not an optional count.
    count: Option<usize>,
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
    // The target lies to the right of this read within the same unit.
    Future(ReferenceCaptureSpan),
    // The target encloses this read and is unclosed in either direction.
    Open,
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

struct ResolvedReferenceSequence {
    ranges: Vec<ReferenceRange>,
    offsets: Vec<usize>,
}

#[derive(Clone, Copy)]
enum GroupAssertion {
    Lookahead(bool),
    Lookbehind(bool),
}

struct GroupFrame {
    assertion: Option<GroupAssertion>,
    capture: Option<usize>,
    capture_start: usize,
    entry: usize,
    alternatives: Vec<usize>,
    exits: Vec<usize>,
}

impl GroupFrame {
    fn open(
        capture: Option<usize>,
        capture_start: usize,
        instructions: &mut Vec<PreparedInstruction>,
    ) -> Self {
        let entry = instructions.len();
        instructions.push(PreparedInstruction::Ready(Instruction::Nop));
        Self {
            assertion: None,
            capture,
            capture_start,
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

struct FixedLookbehindFrame {
    pc: usize,
    end: usize,
    cursor: Option<usize>,
    end_position: usize,
    checkpoint: usize,
    choice_base: usize,
    negative: bool,
    rightmost_iteration: bool,
}

#[derive(Clone, Copy)]
struct FixedLookbehindChoice {
    instruction: usize,
    next: usize,
    cursor: usize,
    checkpoint: usize,
}

struct ChoiceFrame {
    assertion_depth: usize,
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

#[derive(Clone, Copy)]
struct AssertionContext {
    instruction: usize,
    cursor: usize,
    checkpoint: usize,
    frames: usize,
    iteration: Option<usize>,
    iteration_checkpoint: usize,
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
                    program.enter_choice_body(state, self.instruction, context.start, charge)?;
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
            instructions: mut prepared,
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
        charge(prepared.len())?;
        if discard_skipped_regions(&mut prepared).is_none() {
            return Ok(None);
        }
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
    /// candidate start. Retries restore prior starts, completed capture ranges
    /// and named owners. Common terms are retained for each reachable prefix;
    /// inactive captures stay undefined.
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
            changes: Vec::with_capacity(self.0.capture_count),
            iteration: None,
            iterations: Vec::new(),
            assertions: Vec::new(),
        };
        charge(self.0.frame_capacity)?;
        let mut frames = Vec::<ChoiceFrame>::with_capacity(self.0.frame_capacity);
        let end = if sticky { start } else { input.len() };
        'candidate: for candidate in start..=end {
            charge(1)?;
            self.0.restore_captures(&mut state, 0, &mut charge)?;
            frames.clear();
            let mut cursor = candidate;
            let mut pc = 0;
            state.iteration = None;
            state.iterations.clear();
            state.assertions.clear();
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
                    // ECMA-262 22.2.2.8: exhausted assertion alternatives unwind
                    // before trying any choice made outside the assertion.
                    if let Some(context) = state.assertions.last().copied() {
                        if context.frames == frames.len() {
                            state.assertions.pop();
                            self.0
                                .restore_captures(&mut state, context.checkpoint, &mut charge)?;
                            state.iterations.truncate(context.iteration_checkpoint);
                            state.iteration = context.iteration;
                            cursor = context.cursor;
                            let Instruction::Lookahead { end, negative, .. } =
                                self.0.instructions[context.instruction]
                            else {
                                unreachable!("assertion retains its entry")
                            };
                            if negative {
                                pc = end;
                                break;
                            }
                            continue;
                        }
                    }
                    let Some(frame) = frames.last_mut() else {
                        continue 'candidate;
                    };
                    charge(1)?;
                    self.0
                        .restore_captures(&mut state, frame.checkpoint, &mut charge)?;
                    state.iterations.truncate(frame.iteration_checkpoint);
                    state.iteration = frame.iteration;
                    debug_assert!(state.assertions.len() >= frame.assertion_depth);
                    state.assertions.truncate(frame.assertion_depth);
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

enum CaptureUndo {
    Start {
        slot: usize,
        previous: usize,
    },
    Capture {
        slot: usize,
        previous: Option<Range<usize>>,
        named: Option<Range<usize>>,
    },
}

struct CaptureState {
    captures: Vec<Option<Range<usize>>>,
    starts: Vec<usize>,
    named: Vec<Option<Range<usize>>>,
    changes: Vec<CaptureUndo>,
    iteration: Option<usize>,
    // Immutable parent-linked records; retry checkpoints reclaim abandoned paths.
    iterations: Vec<ChoiceIteration>,
    assertions: Vec<AssertionContext>,
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
                assertion_depth: state.assertions.len(),
                instruction: context.instruction,
                checkpoint: state.changes.len(),
                iteration_checkpoint: state.iterations.len(),
                alternative: PendingAlternative::RepeatChoice {
                    enter: Some(!enter),
                },
                iteration: Some(index),
            });
        }
        if enter {
            self.enter_choice_body(state, context.instruction, cursor, charge)?;
            Ok(Some(head))
        } else {
            state.iteration = context.parent;
            Ok(Some(end))
        }
    }

    fn enter_choice_body<E>(
        &self,
        state: &mut CaptureState,
        pc: usize,
        cursor: usize,
        charge: &mut impl FnMut(usize) -> Result<(), E>,
    ) -> Result<(), E> {
        let (Instruction::RepeatChoice { captures, open, .. }
        | Instruction::RepeatZeroWidth { captures, open, .. }) = &self.instructions[pc]
        else {
            unreachable!("repetition owns its capture range")
        };
        for slot in captures.clone() {
            self.write_capture(state, slot, None, charge)?;
        }
        if let Some(slot) = *open {
            self.open_capture(state, slot, cursor, charge)?;
        }
        Ok(())
    }

    fn restore_captures<E>(
        &self,
        state: &mut CaptureState,
        checkpoint: usize,
        charge: &mut impl FnMut(usize) -> Result<(), E>,
    ) -> Result<(), E> {
        charge(state.changes.len() - checkpoint)?;
        if !self.capture_names.is_empty() {
            charge(state.changes.len() - checkpoint)?;
        }
        while state.changes.len() > checkpoint {
            match state.changes.pop().unwrap() {
                CaptureUndo::Start { slot, previous } => state.starts[slot] = previous,
                CaptureUndo::Capture {
                    slot,
                    previous,
                    named,
                } => {
                    state.captures[slot] = previous;
                    if let Some(Some(group)) = self.capture_names.get(slot) {
                        state.named[*group] = named;
                    }
                }
            }
        }
        Ok(())
    }

    fn open_capture<E>(
        &self,
        state: &mut CaptureState,
        slot: usize,
        cursor: usize,
        charge: &mut impl FnMut(usize) -> Result<(), E>,
    ) -> Result<(), E> {
        charge(1)?;
        if state.starts[slot] != cursor {
            state.changes.push(CaptureUndo::Start {
                slot,
                previous: state.starts[slot],
            });
            state.starts[slot] = cursor;
        }
        Ok(())
    }

    fn write_capture<E>(
        &self,
        state: &mut CaptureState,
        slot: usize,
        range: Option<Range<usize>>,
        charge: &mut impl FnMut(usize) -> Result<(), E>,
    ) -> Result<(), E> {
        charge(1)?;
        let group = self.capture_names.get(slot).copied().flatten();
        if !self.capture_names.is_empty() {
            charge(2)?;
        }
        if state.captures[slot] == range && group.is_none_or(|group| state.named[group] == range) {
            return Ok(());
        }
        let previous = std::mem::replace(&mut state.captures[slot], range.clone());
        let named = group.and_then(|group| std::mem::replace(&mut state.named[group], range));
        state.changes.push(CaptureUndo::Capture {
            slot,
            previous,
            named,
        });
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
                self.write_capture(state, slot, Some(range), charge)?;
            }
        } else {
            for &(slot, _) in captures {
                self.write_capture(state, slot, None, charge)?;
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
            Instruction::SkipGroup { captures, end, .. } => {
                for slot in captures.clone() {
                    self.write_capture(state, slot, None, charge)?;
                }
                return Ok(Some(*end));
            }
            Instruction::Choice(alternatives) => {
                charge(1)?;
                frames.push(ChoiceFrame {
                    assertion_depth: state.assertions.len(),
                    instruction: pc,
                    alternative: PendingAlternative::Choice {
                        next: 1,
                        prefix_end: *cursor,
                    },
                    checkpoint: state.changes.len(),
                    iteration_checkpoint: state.iterations.len(),
                    iteration: state.iteration,
                });
                return Ok(Some(alternatives[0]));
            }
            Instruction::Lookahead { head, .. } => {
                charge(1)?;
                state.assertions.push(AssertionContext {
                    instruction: pc,
                    cursor: *cursor,
                    checkpoint: state.changes.len(),
                    frames: frames.len(),
                    iteration: state.iteration,
                    iteration_checkpoint: state.iterations.len(),
                });
                return Ok(Some(*head));
            }
            Instruction::LookaheadEnd(entry) => {
                let context = state
                    .assertions
                    .pop()
                    .expect("assertion body retains its entry");
                debug_assert_eq!(context.instruction, *entry);
                let Instruction::Lookahead { end, negative, .. } = self.instructions[*entry] else {
                    unreachable!("assertion retains its entry")
                };
                // Lookahead commits its first successful path. Outer failure
                // cannot reconsider assertion captures (22.2.2.8).
                frames.truncate(context.frames);
                state.iterations.truncate(context.iteration_checkpoint);
                state.iteration = context.iteration;
                *cursor = context.cursor;
                if negative {
                    self.restore_captures(state, context.checkpoint, charge)?;
                    return Ok(None);
                }
                return Ok(Some(end));
            }
            Instruction::Lookbehind { .. } => {
                if !self.accepts_fixed_lookbehind(input, *cursor, pc, state, charge)? {
                    return Ok(None);
                }
            }
            Instruction::RepeatZeroWidth {
                captures,
                head,
                end,
                required,
                ..
            } => {
                // RepeatMatcher (22.2.2.3.1) rejects optional zero-progress
                // iterations. Required identical zero-width iterations collapse
                // to one body execution with the same final capture state.
                if *required {
                    self.enter_choice_body(state, pc, *cursor, charge)?;
                    return Ok(Some(*head));
                }
                for slot in captures.clone() {
                    self.write_capture(state, slot, None, charge)?;
                }
                return Ok(Some(*end));
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
            Instruction::Open(index) => self.open_capture(state, *index, *cursor, charge)?,
            Instruction::Close(index) => {
                let range = state.starts[*index]..*cursor;
                self.write_capture(state, *index, Some(range), charge)?;
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
                let Some(ResolvedReferenceSequence {
                    ranges: captures,
                    offsets,
                }) = self.resolve_reference_sequence(references, state, false, charge)?
                else {
                    return Ok((*min == Some(0)).then_some(pc + 1));
                };
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
                let checkpoint = state.changes.len();
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
                        assertion_depth: state.assertions.len(),
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
                let checkpoint = state.changes.len();
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
                        assertion_depth: state.assertions.len(),
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
    fn resolve_reference_sequence<E>(
        &self,
        references: &[ReferenceTarget],
        state: &CaptureState,
        backward: bool,
        charge: &mut impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<ResolvedReferenceSequence>, E> {
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
                    if backward {
                        // Its owned target lies to the left and has not matched
                        // in this cleared backward iteration (22.2.2.3.1).
                        None
                    } else {
                        let range = offsets[span.start]..offsets[span.end];
                        (!range.is_empty()).then_some(ReferenceRange::Local(range))
                    }
                }
                ReferenceTarget::Empty | ReferenceTarget::Open | ReferenceTarget::Future(_) => None,
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
                return Ok(None);
            };
            offsets.push(width);
            if let Some(range) = range {
                captures.push(range);
            }
        }
        Ok(Some(ResolvedReferenceSequence {
            ranges: captures,
            offsets,
        }))
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

    fn fixed_lookbehind_alternatives(&self, entry: usize) -> &[usize] {
        match &self.instructions[entry] {
            Instruction::Choice(alternatives) | Instruction::Lookbehind { alternatives, .. } => {
                alternatives
            }
            _ => unreachable!("fixed assertion choices retain their branch inventory"),
        }
    }

    fn choose_fixed_lookbehind<E>(
        &self,
        entry: usize,
        cursor: usize,
        state: &CaptureState,
        choices: &mut Vec<FixedLookbehindChoice>,
        charge: &mut impl FnMut(usize) -> Result<(), E>,
    ) -> Result<usize, E> {
        charge(1)?;
        let alternatives = self.fixed_lookbehind_alternatives(entry);
        if alternatives.len() > 1 {
            charge(1)?;
            choices.push(FixedLookbehindChoice {
                instruction: entry,
                next: 1,
                cursor,
                checkpoint: state.changes.len(),
            });
        }
        Ok(alternatives[0])
    }

    fn fixed_lookbehind_frame<E>(
        &self,
        cursor: usize,
        entry: usize,
        state: &CaptureState,
        choices: &mut Vec<FixedLookbehindChoice>,
        charge: &mut impl FnMut(usize) -> Result<(), E>,
    ) -> Result<FixedLookbehindFrame, E> {
        let Instruction::Lookbehind {
            body,
            ref width,
            negative,
            ref alternatives,
        } = self.instructions[entry]
        else {
            unreachable!("lookbehind retains its control instruction")
        };
        let width = match width {
            LookbehindWidth::Fixed(width) => Some(*width),
            LookbehindWidth::OutsideReferences { fixed, references } => {
                let mut width = *fixed;
                for reference in references {
                    charge(1)?;
                    let range = if reference.named {
                        &state.named[reference.index]
                    } else {
                        &state.captures[reference.index]
                    };
                    let term = range
                        .as_ref()
                        .map_or(0, Range::len)
                        .checked_mul(reference.copies)
                        .and_then(|width| {
                            if width == 0 {
                                Some(0)
                            } else {
                                reference.count.and_then(|count| width.checked_mul(count))
                            }
                        });
                    width = width.and_then(|width| width.checked_add(term?));
                }
                width
            }
        };
        let position = width.and_then(|width| cursor.checked_sub(width));
        let choice_base = choices.len();
        let mut pc = body + 1;
        if !alternatives.is_empty() {
            if let Some(position) = position {
                pc = self.choose_fixed_lookbehind(entry, position, state, choices, charge)?;
            }
        }
        Ok(FixedLookbehindFrame {
            pc,
            end: entry,
            cursor: position,
            end_position: cursor,
            checkpoint: state.changes.len(),
            choice_base,
            negative,
            rightmost_iteration: false,
        })
    }

    fn fixed_zero_width_frame<E>(
        &self,
        cursor: usize,
        entry: usize,
        state: &mut CaptureState,
        choice_base: usize,
        rightmost_iteration: bool,
        charge: &mut impl FnMut(usize) -> Result<(), E>,
    ) -> Result<FixedLookbehindFrame, E> {
        let Instruction::RepeatZeroWidth {
            head,
            end,
            required,
            ..
        } = self.instructions[entry]
        else {
            unreachable!("zero-width repetition retains its control instruction")
        };
        debug_assert!(required);
        let checkpoint = state.changes.len();
        self.enter_choice_body(state, entry, cursor, charge)?;
        Ok(FixedLookbehindFrame {
            pc: head,
            end,
            cursor: Some(cursor),
            end_position: cursor,
            checkpoint,
            choice_base,
            negative: false,
            rightmost_iteration,
        })
    }

    fn fixed_lookahead_frame(
        &self,
        input: &[u16],
        cursor: usize,
        entry: usize,
        state: &CaptureState,
        choice_base: usize,
    ) -> FixedLookbehindFrame {
        let Instruction::Lookahead {
            head,
            negative,
            fixed_width: Some(width),
            ..
        } = self.instructions[entry]
        else {
            unreachable!("fixed lookahead retains its proved width")
        };
        let end_position = cursor.checked_add(width).filter(|&end| end <= input.len());
        FixedLookbehindFrame {
            pc: head,
            end: entry - 1,
            cursor: end_position.map(|_| cursor),
            end_position: end_position.unwrap_or(cursor),
            checkpoint: state.changes.len(),
            choice_base,
            negative,
            rightmost_iteration: true,
        }
    }

    // Fixed predicates and unrepeated captures use the same absolute ranges
    // in either direction. Counted character captures use their leftmost iteration.
    // Equal-width choices inspect fixed positions without internal capture reads.
    fn accepts_fixed_lookbehind<E>(
        &self,
        input: &[u16],
        cursor: usize,
        entry: usize,
        state: &mut CaptureState,
        charge: &mut impl FnMut(usize) -> Result<(), E>,
    ) -> Result<bool, E> {
        let mut frames = Vec::<FixedLookbehindFrame>::new();
        let mut choices = Vec::<FixedLookbehindChoice>::new();
        let mut current =
            self.fixed_lookbehind_frame(cursor, entry, state, &mut choices, charge)?;
        loop {
            if current.cursor.is_none() && choices.len() > current.choice_base {
                charge(1)?;
                let mut choice = choices.pop().unwrap();
                self.restore_captures(state, choice.checkpoint, charge)?;
                let alternatives = self.fixed_lookbehind_alternatives(choice.instruction);
                current.pc = alternatives[choice.next];
                current.cursor = Some(choice.cursor);
                choice.next += 1;
                if choice.next < alternatives.len() {
                    charge(1)?;
                    choices.push(choice);
                }
                continue;
            }
            if current.cursor.is_none() || current.pc == current.end {
                charge(1)?;
                if let Some(position) = current.cursor {
                    debug_assert_eq!(position, current.end_position);
                }
                let accepted = current.cursor.is_some() != current.negative;
                // Assertions are atomic (22.2.2.8). A zero-width repetition's
                // first success also suffices here: preparation excludes all
                // capture-dependent continuations inside fixed lookbehind.
                charge(choices.len() - current.choice_base)?;
                choices.truncate(current.choice_base);
                // Positive success retains exact fixed ranges. All negative
                // outcomes and failed positive bodies restore their checkpoint
                // before resuming an outer assertion or choice (22.2.2.8).
                if current.negative || !accepted {
                    self.restore_captures(state, current.checkpoint, charge)?;
                }
                let Some(mut parent) = frames.pop() else {
                    return Ok(accepted);
                };
                if !accepted {
                    parent.cursor = None;
                }
                current = parent;
                continue;
            }
            charge(1)?;
            let position = current
                .cursor
                .as_mut()
                .expect("fixed body has an available prefix");
            let term = &self.instructions[current.pc];
            let accepted = match term {
                Instruction::Nop
                | Instruction::RepeatedCharacter(_)
                | Instruction::RepeatedSet(_)
                | Instruction::RepeatedAssert(_) => true,
                Instruction::Jump(target) => {
                    current.pc = *target;
                    continue;
                }
                Instruction::SkipGroup { captures, end, .. } => {
                    for slot in captures.clone() {
                        self.write_capture(state, slot, None, charge)?;
                    }
                    current.pc = *end;
                    continue;
                }
                Instruction::Choice(_) => {
                    current.pc = self.choose_fixed_lookbehind(
                        current.pc,
                        *position,
                        state,
                        &mut choices,
                        charge,
                    )?;
                    continue;
                }
                Instruction::Lookbehind { .. } => {
                    let child = self.fixed_lookbehind_frame(
                        *position,
                        current.pc,
                        state,
                        &mut choices,
                        charge,
                    )?;
                    current.pc += 1;
                    charge(1)?;
                    frames.push(current);
                    current = child;
                    continue;
                }
                Instruction::Lookahead { end, .. } => {
                    let child = self.fixed_lookahead_frame(
                        input,
                        *position,
                        current.pc,
                        state,
                        choices.len(),
                    );
                    current.pc = *end;
                    charge(1)?;
                    frames.push(current);
                    current = child;
                    continue;
                }
                Instruction::RepeatZeroWidth {
                    captures,
                    end,
                    required,
                    ..
                } => {
                    if *required {
                        let child = self.fixed_zero_width_frame(
                            *position,
                            current.pc,
                            state,
                            choices.len(),
                            current.rightmost_iteration,
                            charge,
                        )?;
                        current.pc = *end;
                        charge(1)?;
                        frames.push(current);
                        current = child;
                    } else {
                        // RepeatMatcher rejects optional empty iterations
                        // before exporting their captures (22.2.2.3.1).
                        for slot in captures.clone() {
                            self.write_capture(state, slot, None, charge)?;
                        }
                        current.pc = *end;
                    }
                    continue;
                }
                Instruction::Character(_) | Instruction::Set(_) => {
                    self.compare_fixed_lookbehind_term(input, position, term, charge)?
                }
                Instruction::Reference(index) | Instruction::NamedReference(index) => {
                    let range = if matches!(term, Instruction::Reference(_)) {
                        &state.captures[*index]
                    } else {
                        &state.named[*index]
                    };
                    if let Some(range) = range {
                        if let Some(end) =
                            self.compare_reference(input, *position, range, charge)?
                        {
                            *position = end;
                            true
                        } else {
                            false
                        }
                    } else {
                        true
                    }
                }
                Instruction::QuantifiedReference {
                    index,
                    named,
                    bounds: (min, max, _),
                    copies,
                    ..
                } => {
                    debug_assert!(min == max || min.is_none());
                    if *min == Some(0) {
                        // Exactly zero iterations never read the target.
                        self.complete_reference_captures(current.pc, state, None, None, charge)?;
                        true
                    } else {
                        let range = if *named {
                            &state.named[*index]
                        } else {
                            &state.captures[*index]
                        };
                        let width = range.as_ref().map_or(0, Range::len) * *copies;
                        let base = *position;
                        // Required empty iterations have identical effects;
                        // summarize them without expanding the source count.
                        let iterations = if width == 0 {
                            1
                        } else {
                            min.expect("available consuming outside counts are representable")
                        };
                        let mut accepted = true;
                        for _ in 0..iterations {
                            charge(1)?;
                            if let Some(range) = range {
                                if let Some(end) = self.compare_reference_copies(
                                    input, *position, range, *copies, charge,
                                )? {
                                    *position = end;
                                } else {
                                    accepted = false;
                                    break;
                                }
                            }
                        }
                        if accepted {
                            // RepeatMatcher retains the leftmost backward
                            // iteration (22.2.2.3.1). Targets are outside the
                            // complete assertion and stay unchanged.
                            let capture_base = if current.rightmost_iteration {
                                *position - width
                            } else {
                                base
                            };
                            self.complete_reference_captures(
                                current.pc,
                                state,
                                Some(capture_base..capture_base + width),
                                None,
                                charge,
                            )?;
                        }
                        accepted
                    }
                }
                Instruction::QuantifiedReferenceSequence {
                    references,
                    bounds: (min, _, _),
                    captures,
                    lookbehind_offsets,
                } => {
                    if *min != Some(0)
                        && references
                            .iter()
                            .any(|target| matches!(target, ReferenceTarget::Input { .. }))
                    {
                        // The complete owner's proof permits only immutable
                        // outside ranges and proved empty internal reads.
                        // Source-order comparison of each proved unit is valid
                        // backward too (22.2.2.3.1, 22.2.2.8, 22.2.2.9.2).
                        let Some(resolved) = self.resolve_reference_sequence(
                            references,
                            state,
                            !current.rightmost_iteration,
                            charge,
                        )?
                        else {
                            current.cursor = None;
                            continue;
                        };
                        let width = *resolved.offsets.last().unwrap();
                        let base = *position;
                        let iterations = if width == 0 {
                            1
                        } else {
                            min.expect("consuming outside counts are representable")
                        };
                        let mut accepted = true;
                        for _ in 0..iterations {
                            charge(1)?;
                            if let Some(end) = self.compare_reference_sequence(
                                input,
                                *position,
                                &resolved.ranges,
                                charge,
                            )? {
                                *position = end;
                            } else {
                                accepted = false;
                                break;
                            }
                        }
                        if accepted {
                            let capture_base = if current.rightmost_iteration {
                                *position - width
                            } else {
                                base
                            };
                            self.complete_reference_captures(
                                current.pc,
                                state,
                                Some(capture_base..capture_base + width),
                                Some(&resolved.offsets),
                                charge,
                            )?;
                        }
                        if !accepted {
                            current.cursor = None;
                        } else {
                            current.pc += 1;
                        }
                        continue;
                    }
                    let base = *position;
                    let width = lookbehind_offsets
                        .as_ref()
                        .map_or(references.len(), |offsets| *offsets.last().unwrap());
                    // Required zero-width iterations see identical predicates
                    // and only proved empty same-unit reads. Their final effects need one pass.
                    let required = *min != Some(0);
                    let iterations = if width == 0 {
                        usize::from(required)
                    } else {
                        min.expect("consuming fixed counts are representable")
                    };
                    let mut accepted = true;
                    'iterations: for _ in 0..iterations {
                        let iteration_start = *position;
                        for target in references {
                            let matched = match *target {
                                ReferenceTarget::Term(index) => self
                                    .compare_fixed_lookbehind_term(
                                        input,
                                        position,
                                        &self.instructions[index],
                                        charge,
                                    )?,
                                ReferenceTarget::Empty
                                | ReferenceTarget::Open
                                | ReferenceTarget::Local(_) => {
                                    // Empty units have empty reads in either direction.
                                    // In consuming backward units, proved closed
                                    // local targets lie to the left and have not
                                    // matched in this cleared iteration. Forward
                                    // consuming local reads retain their own plan.
                                    charge(1)?;
                                    true
                                }
                                ReferenceTarget::Future(span) => {
                                    charge(1)?;
                                    if current.rightmost_iteration {
                                        // The target is still unmatched in a forward assertion.
                                        true
                                    } else {
                                        let offsets = lookbehind_offsets
                                            .as_ref()
                                            .expect("future reads retain unit offsets");
                                        let range = iteration_start + offsets[span.start]
                                            ..iteration_start + offsets[span.end];
                                        if let Some(end) = self
                                            .compare_reference(input, *position, &range, charge)?
                                        {
                                            *position = end;
                                            true
                                        } else {
                                            false
                                        }
                                    }
                                }
                                ReferenceTarget::Input { .. } => {
                                    unreachable!("fixed lookbehind excludes outside capture reads")
                                }
                            };
                            if !matched {
                                accepted = false;
                                break 'iterations;
                            }
                        }
                    }
                    if accepted {
                        // Backward repetitions retain the leftmost iteration;
                        // lookahead uses forward, rightmost capture effects
                        // (22.2.2.3.1, 22.2.2.8). Same-unit reads retain fixed spans.
                        let capture_base = if current.rightmost_iteration && required {
                            *position - width
                        } else {
                            base
                        };
                        for &(slot, span) in captures {
                            let offset = |index| {
                                lookbehind_offsets
                                    .as_ref()
                                    .map_or(index, |offsets| offsets[index])
                            };
                            let range = required.then(|| {
                                capture_base + offset(span.start)..capture_base + offset(span.end)
                            });
                            self.write_capture(state, slot, range, charge)?;
                        }
                    }
                    accepted
                }
                Instruction::Assert(assertion) => {
                    charge(2)?;
                    assertion.accepts(input, *position, self.multiline)
                }
                Instruction::Open(slot) => {
                    self.open_capture(state, *slot, *position, charge)?;
                    true
                }
                Instruction::Close(slot) => {
                    self.write_capture(state, *slot, Some(state.starts[*slot]..*position), charge)?;
                    true
                }
                _ => unreachable!("lookbehind retains its fixed body"),
            };
            if !accepted {
                current.cursor = None;
            }
            current.pc += 1;
        }
    }

    // Character terms consume one unit; boundary predicates consume none.
    // Preparation proves every counted term position in the complete prefix.
    fn compare_fixed_lookbehind_term<E>(
        &self,
        input: &[u16],
        position: &mut usize,
        term: &Instruction,
        charge: &mut impl FnMut(usize) -> Result<(), E>,
    ) -> Result<bool, E> {
        charge(1)?;
        if let Instruction::RepeatedAssert(assertion) = term {
            charge(2)?;
            return Ok(assertion.accepts(input, *position, self.multiline));
        }
        let Some(&unit) = input.get(*position) else {
            return Ok(false);
        };
        *position += 1;
        Ok(match term {
            Instruction::Character(expected) | Instruction::RepeatedCharacter(expected) => {
                canonicalize(unit, self.ignore_case) == *expected
            }
            Instruction::Set(matcher) | Instruction::RepeatedSet(matcher) => matcher.matches(unit),
            _ => unreachable!("lookbehind term is a single character"),
        })
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
                    } else if span.start <= ordinal {
                        ReferenceTarget::Open
                    } else if span.start == span.end {
                        ReferenceTarget::Empty
                    } else {
                        ReferenceTarget::Future(span)
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
                lookbehind_offsets: None,
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

// Prove that every complete body path is zero-width. Prepared lookahead and
// zero-width child repetitions and fixed lookbehind summarize their bodies.
fn quantify_zero_width_assertion_wrapper(
    instructions: &mut Vec<PreparedInstruction>,
    body: Range<usize>,
    bounds: Bounds,
    captures: Range<usize>,
    zero_width: &mut Vec<bool>,
) -> Option<()> {
    let open = match &instructions[body.start] {
        PreparedInstruction::Ready(Instruction::Open(slot)) => Some(*slot),
        PreparedInstruction::Ready(Instruction::Nop | Instruction::Choice(_)) => None,
        _ => return None,
    };
    zero_width.resize(instructions.len() + 1, false);
    zero_width[body.end] = true;
    let mut next = body.end;
    let mut found = false;
    while next > body.start {
        let pc = next - 1;
        if let PreparedInstruction::Ready(
            Instruction::Lookbehind { body: child, .. }
            | Instruction::SkipGroup { body: child, .. },
        ) = &instructions[pc]
        {
            if *child < body.start || *child >= pc {
                return None;
            }
            zero_width[*child] = zero_width[pc + 1];
            found = true;
            next = *child;
            continue;
        }
        if pc > body.start {
            if let PreparedInstruction::Ready(
                Instruction::Lookahead {
                    body: child,
                    head,
                    end,
                    ..
                }
                | Instruction::RepeatZeroWidth {
                    body: child,
                    head,
                    end,
                    ..
                },
            ) = &instructions[pc - 1]
            {
                if *head != pc || *end != pc + 1 || *child < body.start || *child >= pc - 1 {
                    return None;
                }
                found = true;
                zero_width[*child] = zero_width[*end];
                next = *child;
                continue;
            }
        }
        next = pc;
        zero_width[pc] = match &instructions[pc] {
            PreparedInstruction::Ready(
                Instruction::Nop
                | Instruction::Open(_)
                | Instruction::Close(_)
                | Instruction::Assert(_)
                | Instruction::RepeatedCharacter(_)
                | Instruction::RepeatedSet(_)
                | Instruction::RepeatedAssert(_),
            )
            | PreparedInstruction::Set { repeated: true, .. } => zero_width[pc + 1],
            PreparedInstruction::Ready(Instruction::QuantifiedReferenceSequence {
                references,
                bounds: (min, max, _),
                ..
            }) => {
                if !(*min == Some(0) && *max == Some(0))
                    && references
                        .iter()
                        .enumerate()
                        .any(|(ordinal, target)| match target {
                            ReferenceTarget::Empty | ReferenceTarget::Open => false,
                            ReferenceTarget::Future(span) => {
                                span.start <= ordinal
                                    || span.start > span.end
                                    || span.end > references.len()
                            }
                            // Local spans contain only completed preceding terms.
                            ReferenceTarget::Local(span) => {
                                span.start > span.end || span.end > ordinal
                            }
                            ReferenceTarget::Term(index) => !matches!(
                                instructions.get(*index),
                                Some(PreparedInstruction::Ready(Instruction::RepeatedAssert(_)))
                            ),
                            ReferenceTarget::Input { .. } => true,
                        })
                {
                    return None;
                }
                found = true;
                zero_width[pc + 1]
            }
            PreparedInstruction::Ready(Instruction::Jump(target)) => {
                if *target <= pc || *target > body.end {
                    return None;
                }
                zero_width[*target]
            }
            PreparedInstruction::Ready(Instruction::Choice(branches)) => {
                if branches.is_empty()
                    || branches
                        .iter()
                        .any(|&target| target <= pc || target > body.end)
                {
                    return None;
                }
                found = true;
                branches.iter().all(|&target| zero_width[target])
            }
            _ => return None,
        };
    }
    if !found || !zero_width[body.start] {
        return None;
    }
    let entry = instructions.len() + 1;
    let original = std::mem::replace(
        &mut instructions[body.start],
        PreparedInstruction::Ready(Instruction::Jump(entry)),
    );
    instructions.push(PreparedInstruction::Ready(Instruction::Jump(entry + 2)));
    instructions.push(PreparedInstruction::Ready(Instruction::RepeatZeroWidth {
        captures,
        open,
        body: body.start,
        head: entry + 1,
        end: entry + 2,
        required: bounds.0 != Some(0),
    }));
    instructions.push(match original {
        PreparedInstruction::Ready(Instruction::Choice(_)) => original,
        _ => PreparedInstruction::Ready(Instruction::Jump(body.start + 1)),
    });
    Some(())
}

// The source regions are disjoint or nested. Visit an outer skipped owner first
// and bypass its descendants, so every discarded instruction is visited once.
fn discard_skipped_regions(instructions: &mut [PreparedInstruction]) -> Option<()> {
    let mut next = instructions.len();
    while next > 0 {
        let pc = next - 1;
        next = pc;
        let PreparedInstruction::Ready(Instruction::SkipGroup { body, end, .. }) =
            &instructions[pc]
        else {
            continue;
        };
        let body = *body;
        if body >= pc || *end != pc + 1 {
            return None;
        }
        if !matches!(instructions.get(body), Some(PreparedInstruction::Ready(Instruction::Jump(control))) if *control == pc)
        {
            return None;
        }
        for instruction in &mut instructions[body + 1..pc] {
            *instruction = PreparedInstruction::Ready(Instruction::Nop);
        }
        next = body;
    }
    Some(())
}

fn quantify_progressing_choice(
    instructions: &mut Vec<PreparedInstruction>,
    body: Range<usize>,
    bounds: Bounds,
    captures: Range<usize>,
    progresses: &mut Vec<bool>,
) -> Option<()> {
    let open = match &instructions[body.start] {
        PreparedInstruction::Ready(Instruction::Open(slot)) => Some(*slot),
        PreparedInstruction::Ready(Instruction::Nop | Instruction::Choice(_)) => None,
        _ => return None,
    };
    // Treat an already proven child loop as one operation. Its
    // required iterations consume input; optional iterations use the suffix.
    // Reuse preparation storage instead of rescanning or clearing child bodies.
    progresses.resize(instructions.len() + 1, false);
    progresses[body.end] = false;
    let mut next = body.end;
    while next > body.start {
        let pc = next - 1;
        if let PreparedInstruction::Ready(
            Instruction::Lookbehind { body: child, .. }
            | Instruction::SkipGroup { body: child, .. },
        ) = &instructions[pc]
        {
            if *child < body.start || *child >= pc {
                return None;
            }
            progresses[*child] = progresses[pc + 1];

            next = *child;
            continue;
        }
        if pc > body.start {
            if let PreparedInstruction::Ready(
                Instruction::Lookahead {
                    body: child,
                    head,
                    end,
                    ..
                }
                | Instruction::RepeatZeroWidth {
                    body: child,
                    head,
                    end,
                    ..
                },
            ) = &instructions[pc - 1]
            {
                if *head != pc || *end != pc + 1 || *child < body.start || *child >= pc - 1 {
                    return None;
                }
                // Examining input does not establish progress for a repetition.
                progresses[*child] = progresses[*end];
                next = *child;
                continue;
            }
            if let PreparedInstruction::Ready(Instruction::RepeatChoice {
                body: child,
                head,
                end,
                bounds: (min, _, _),
                ..
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
        progresses[pc] = match &instructions[pc] {
            PreparedInstruction::Ready(Instruction::Character(_))
            | PreparedInstruction::Set {
                repeated: false, ..
            } => true,
            PreparedInstruction::Ready(
                Instruction::Nop
                | Instruction::Assert(_)
                | Instruction::Reference(_)
                | Instruction::NamedReference(_)
                | Instruction::Open(_)
                | Instruction::Close(_)
                | Instruction::RepeatedCharacter(_)
                | Instruction::RepeatedSet(_)
                | Instruction::RepeatedAssert(_),
            )
            | PreparedInstruction::Set { repeated: true, .. } => progresses[pc + 1],
            PreparedInstruction::Ready(Instruction::QuantifiedReferenceSequence {
                references,
                bounds: (min, _, _),
                ..
            }) => {
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
            PreparedInstruction::Ready(Instruction::QuantifiedReference { .. }) => {
                progresses[pc + 1]
            }
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
        captures,
        open,
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

// Keep the same unit proof for fixed bodies and linear bodies importing outside
// ranges. Local targets to the left have not matched in cleared backward units;
// enclosing open targets are unclosed in either direction. Targets to the right
// have matched backward and retain exact same-unit source spans. Their widths
// depend only on later terms, so compute a flat suffix inventory before storing
// prefix offsets (22.2.2.3.1, 22.2.2.8, 22.2.2.9.2).
fn fixed_lookbehind_sequence_width(
    instructions: &[PreparedInstruction],
    references: &[ReferenceTarget],
    (min, max, _): Bounds,
    backward: bool,
) -> Option<(usize, Option<Box<[usize]>>)> {
    let mut offsets = vec![0usize; references.len() + 1];
    let mut needs_offsets = false;
    let mut has_local = false;
    for (ordinal, target) in references.iter().enumerate().rev() {
        let width = match *target {
            ReferenceTarget::Term(index) => match instructions.get(index) {
                Some(PreparedInstruction::Ready(Instruction::RepeatedAssert(_))) => {
                    needs_offsets = true;
                    0
                }
                Some(PreparedInstruction::Ready(Instruction::RepeatedCharacter(_)))
                | Some(PreparedInstruction::Set { repeated: true, .. }) => 1,
                _ => return None,
            },
            ReferenceTarget::Empty | ReferenceTarget::Open => {
                needs_offsets = true;
                0
            }
            ReferenceTarget::Local(span) if span.start <= span.end && span.end <= ordinal => {
                has_local = true;
                needs_offsets = true;
                0
            }
            ReferenceTarget::Future(span)
                if span.start > ordinal
                    && span.start <= span.end
                    && span.end <= references.len() =>
            {
                needs_offsets = true;
                if backward {
                    offsets[span.start].checked_sub(offsets[span.end])?
                } else {
                    0
                }
            }
            _ => return None,
        };
        offsets[ordinal] = offsets[ordinal + 1].checked_add(width)?;
    }
    let width = offsets[0];
    if has_local && !backward && width != 0 {
        return None;
    }
    let complete_width = if width == 0 {
        0
    } else {
        let count = min?;
        if max != Some(count) {
            return None;
        }
        count.checked_mul(width)?
    };
    let offsets = needs_offsets.then(|| {
        for offset in &mut offsets {
            *offset = width - *offset;
        }
        offsets.into_boxed_slice()
    });
    Some((complete_width, offsets))
}

fn fixed_lookbehind_width(
    instructions: &mut [PreparedInstruction],
    entry: usize,
    backward: bool,
) -> Option<usize> {
    let mut counted_offsets = Vec::<(usize, Box<[usize]>)>::new();
    let mut tails = HashMap::<usize, usize>::new();
    let mut zero_wrappers = HashMap::<usize, (usize, usize)>::new();
    tails.insert(instructions.len(), 0);
    let mut next = instructions.len();
    while next > entry + 1 {
        let pc = next - 1;
        next = pc;
        if let Some((head, end)) = zero_wrappers.remove(&pc) {
            let PreparedInstruction::Ready(Instruction::Jump(control)) = &instructions[pc] else {
                return None;
            };
            if control.checked_add(1) != Some(head) {
                return None;
            }
            let suffix = *tails.get(&end)?;
            let accepted = match &instructions[head] {
                PreparedInstruction::Ready(Instruction::Choice(alternatives)) => {
                    !alternatives.is_empty()
                        && alternatives
                            .iter()
                            .all(|target| tails.get(target) == Some(&suffix))
                }
                PreparedInstruction::Ready(Instruction::Jump(target)) => {
                    *target == pc + 1 && tails.get(target) == Some(&suffix)
                }
                _ => false,
            };
            if !accepted {
                return None;
            }
            tails.insert(pc, suffix);
            continue;
        }
        if pc > entry + 1 {
            if let PreparedInstruction::Ready(Instruction::Lookahead {
                body,
                head,
                end,
                fixed_width: Some(_),
                ..
            }) = &instructions[pc - 1]
            {
                if *body <= entry || *body >= pc - 1 || *head != pc || *end != pc + 1 {
                    return None;
                }
                if !matches!(instructions.get(*body), Some(PreparedInstruction::Ready(Instruction::Jump(control))) if *control == pc - 1)
                {
                    return None;
                }
                // This child's complete body has already proved
                // a fixed width. An assertion consumes no parent input.
                tails.insert(*body, *tails.get(end)?);
                next = *body;
                continue;
            }
            if let PreparedInstruction::Ready(Instruction::RepeatZeroWidth {
                body,
                head,
                end,
                ..
            }) = &instructions[pc - 1]
            {
                if *body <= entry || *body >= pc - 1 || *head != pc || *end != pc + 1 {
                    return None;
                }
                let suffix = *tails.get(end)?;
                if zero_wrappers.insert(*body, (*head, *end)).is_some() {
                    return None;
                }
                // Validate the actual body once before accepting its root.
                // Its control/head use zero width; unsupported body operations
                // and any nonzero complete branch still reject the owner.
                tails.insert(pc, suffix);
                tails.insert(pc - 1, suffix);
                next = pc - 1;
                continue;
            }
        }
        let width = match &instructions[pc] {
            PreparedInstruction::Ready(Instruction::Jump(target)) => *tails.get(target)?,
            PreparedInstruction::Ready(Instruction::Choice(alternatives)) => {
                let first = *tails.get(alternatives.first()?)?;
                if alternatives.iter().any(|pc| tails.get(pc) != Some(&first)) {
                    return None;
                }
                first
            }
            PreparedInstruction::Ready(
                Instruction::Lookbehind {
                    body: child,
                    width: LookbehindWidth::Fixed(_),
                    ..
                }
                | Instruction::SkipGroup { body: child, .. },
            ) => {
                if *child <= entry || *child >= pc {
                    return None;
                }
                let width = *tails.get(&(pc + 1))?;
                tails.insert(*child, width);
                next = *child;
                width
            }
            PreparedInstruction::Ready(Instruction::Character(_))
            | PreparedInstruction::Set {
                repeated: false, ..
            } => tails.get(&(pc + 1))?.checked_add(1)?,
            PreparedInstruction::Ready(
                Instruction::Nop
                | Instruction::Open(_)
                | Instruction::Close(_)
                | Instruction::Assert(_)
                | Instruction::RepeatedCharacter(_)
                | Instruction::RepeatedAssert(_),
            )
            | PreparedInstruction::Set { repeated: true, .. } => *tails.get(&(pc + 1))?,
            PreparedInstruction::Ready(
                Instruction::QuantifiedReference {
                    bounds: (Some(0), Some(0), _),
                    ..
                }
                | Instruction::QuantifiedReferenceSequence {
                    bounds: (Some(0), Some(0), _),
                    ..
                },
            ) => {
                // RepeatMatcher with maximum zero cannot observe its body or
                // any local/outside reference (22.2.2.3.1).
                *tails.get(&(pc + 1))?
            }
            PreparedInstruction::Ready(Instruction::QuantifiedReferenceSequence {
                references,
                bounds: (min, max, _),
                ..
            }) => {
                let (complete_width, offsets) = fixed_lookbehind_sequence_width(
                    instructions,
                    references,
                    (*min, *max, false),
                    backward,
                )?;
                if let Some(offsets) = offsets {
                    counted_offsets.push((pc, offsets));
                }
                tails.get(&(pc + 1))?.checked_add(complete_width)?
            }
            _ => return None,
        };
        tails.insert(pc, width);
    }
    if !zero_wrappers.is_empty() {
        return None;
    }
    let width = match &instructions[entry] {
        PreparedInstruction::Ready(Instruction::Nop) => tails.get(&(entry + 1)).copied(),
        PreparedInstruction::Ready(Instruction::Choice(alternatives)) => {
            let first = *tails.get(alternatives.first()?)?;
            alternatives
                .iter()
                .all(|pc| tails.get(pc) == Some(&first))
                .then_some(first)
        }
        _ => None,
    }?;
    // Only accepted owner bodies receive metadata. Summarized child regions
    // are skipped, so term-relative spans are translated exactly once.
    for (pc, offsets) in counted_offsets {
        let PreparedInstruction::Ready(Instruction::QuantifiedReferenceSequence {
            lookbehind_offsets,
            ..
        }) = &mut instructions[pc]
        else {
            unreachable!("counted fixed body retains its sequence instruction")
        };
        *lookbehind_offsets = Some(offsets);
    }
    Some(width)
}

// A linear body's outside targets cannot change while it executes. Resolve its
// width from those original ranges at entry, then compare the same prefix in
// source order. Internal reads and variable/dependent children need a different
// backward plan and remain unsupported (22.2.2.8, 22.2.2.9.2).
fn outside_reference_lookbehind_width(
    instructions: &mut [PreparedInstruction],
    entry: usize,
    captures: Range<usize>,
    named_groups: &[&[usize]],
) -> Option<LookbehindWidth> {
    if !matches!(
        instructions.get(entry),
        Some(PreparedInstruction::Ready(Instruction::Nop))
    ) {
        return None;
    }
    let mut fixed = Some(0usize);
    let mut width_references = Vec::new();
    let mut counted_offsets = Vec::new();
    let outside = |index: usize, named: bool| {
        if named {
            Some(
                !named_groups
                    .get(index)?
                    .iter()
                    .any(|slot| captures.contains(slot)),
            )
        } else {
            Some(!captures.contains(&index))
        }
    };
    let mut pc = entry + 1;
    while pc < instructions.len() {
        match &instructions[pc] {
            PreparedInstruction::Ready(Instruction::Reference(index)) => {
                if !outside(*index, false)? {
                    return None;
                }
                width_references.push(OutsideReferenceWidth {
                    index: *index,
                    named: false,
                    copies: 1,
                    count: Some(1),
                });
            }
            PreparedInstruction::Ready(Instruction::NamedReference(index)) => {
                if !outside(*index, true)? {
                    return None;
                }
                width_references.push(OutsideReferenceWidth {
                    index: *index,
                    named: true,
                    copies: 1,
                    count: Some(1),
                });
            }
            PreparedInstruction::Ready(Instruction::QuantifiedReference {
                index,
                named,
                bounds: (min, max, _),
                copies,
                ..
            }) if min == max || min.is_none() => {
                if *min != Some(0) {
                    if !outside(*index, *named)? {
                        return None;
                    }
                    width_references.push(OutsideReferenceWidth {
                        index: *index,
                        named: *named,
                        copies: *copies,
                        count: *min,
                    });
                }
            }
            PreparedInstruction::Ready(Instruction::QuantifiedReferenceSequence {
                bounds: (Some(0), Some(0), _),
                ..
            }) => {
                // Maximum zero never observes any body term or target.
            }
            PreparedInstruction::Ready(Instruction::QuantifiedReferenceSequence {
                references,
                bounds,
                ..
            }) => {
                if let Some((width, offsets)) =
                    fixed_lookbehind_sequence_width(instructions, references, *bounds, true)
                {
                    fixed = fixed.and_then(|fixed| fixed.checked_add(width));
                    if let Some(offsets) = offsets {
                        counted_offsets.push((pc, offsets));
                    }
                } else {
                    let (min, max, _) = *bounds;
                    if min.is_some() && min != max {
                        return None;
                    }
                    for (ordinal, target) in references.iter().enumerate() {
                        match *target {
                            ReferenceTarget::Input { index, named } => {
                                if !outside(index, named)? {
                                    return None;
                                }
                                // Width terms stay flat even when targets or
                                // unit lengths differ. Resolve them at entry.
                                width_references.push(OutsideReferenceWidth {
                                    index,
                                    named,
                                    copies: 1,
                                    count: min,
                                });
                            }
                            ReferenceTarget::Term(index) => {
                                let width = match instructions.get(index)? {
                                    PreparedInstruction::Ready(Instruction::RepeatedCharacter(
                                        _,
                                    ))
                                    | PreparedInstruction::Set { repeated: true, .. } => 1usize,
                                    PreparedInstruction::Ready(Instruction::RepeatedAssert(_)) => 0,
                                    _ => return None,
                                };
                                let term = if width == 0 {
                                    Some(0)
                                } else {
                                    min.and_then(|count| width.checked_mul(count))
                                };
                                fixed = fixed.and_then(|fixed| fixed.checked_add(term?));
                            }
                            ReferenceTarget::Empty | ReferenceTarget::Open => {}
                            ReferenceTarget::Local(span)
                                if span.start <= span.end && span.end <= ordinal => {}
                            // Dependent right-hand reads still need a mixed
                            // width proof; the fixed-unit proof handles those
                            // only when outside inputs are absent.
                            _ => return None,
                        }
                    }
                }
            }
            PreparedInstruction::Ready(Instruction::Character(_))
            | PreparedInstruction::Set {
                repeated: false, ..
            } => fixed = fixed.and_then(|fixed| fixed.checked_add(1)),
            PreparedInstruction::Ready(
                Instruction::Nop
                | Instruction::Open(_)
                | Instruction::Close(_)
                | Instruction::Assert(_)
                | Instruction::RepeatedCharacter(_)
                | Instruction::RepeatedAssert(_),
            )
            | PreparedInstruction::Set { repeated: true, .. } => {}
            PreparedInstruction::Ready(Instruction::Jump(owner)) => {
                let end =
                    match instructions.get(*owner)? {
                        PreparedInstruction::Ready(Instruction::Lookbehind {
                            body,
                            width: LookbehindWidth::Fixed(_),
                            ..
                        }) if *body == pc => owner.checked_add(1)?,
                        PreparedInstruction::Ready(Instruction::Lookahead {
                            body,
                            end,
                            fixed_width: Some(_),
                            ..
                        }) if *body == pc => *end,
                        PreparedInstruction::Ready(Instruction::SkipGroup {
                            body, end, ..
                        }) if *body == pc => *end,
                        _ => return None,
                    };
                if end <= pc || end > instructions.len() {
                    return None;
                }
                pc = end;
                continue;
            }
            _ => return None,
        }
        pc += 1;
    }
    if width_references.is_empty() {
        return None;
    }
    for (pc, offsets) in counted_offsets {
        let PreparedInstruction::Ready(Instruction::QuantifiedReferenceSequence {
            lookbehind_offsets,
            ..
        }) = &mut instructions[pc]
        else {
            unreachable!("fixed unit retains its prepared sequence");
        };
        *lookbehind_offsets = Some(offsets);
    }
    Some(LookbehindWidth::OutsideReferences {
        fixed,
        references: width_references.into_boxed_slice(),
    })
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
    let mut last_complex_group = None;
    let mut groups = Vec::<GroupFrame>::new();
    let mut current = GroupFrame::open(None, 0, &mut instructions);
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
                let mut assertion = None;
                let group = if source.get(cursor) == Some(&0x3f) {
                    assertion = match source.get(cursor + 1)? {
                        0x3a => None,
                        0x3d => Some(GroupAssertion::Lookahead(false)),
                        0x21 => Some(GroupAssertion::Lookahead(true)),
                        0x3c => {
                            let negative = match source.get(cursor + 2)? {
                                0x3d => false,
                                0x21 => true,
                                _ => return None,
                            };
                            cursor += 1;
                            Some(GroupAssertion::Lookbehind(negative))
                        }
                        _ => return None,
                    };
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
                let mut child =
                    GroupFrame::open(group, group.unwrap_or(capture_count), &mut instructions);
                child.assertion = assertion;
                groups.push(std::mem::replace(&mut current, child));
            }
            0x29 => {
                let parent = groups.pop()?;
                let group = std::mem::replace(&mut current, parent);
                let assertion = group.assertion;
                let entry = group.entry;
                let capture_start = group.capture_start;
                let capture = group.finish(&mut instructions);
                scopes.push(ScopeStep::Close);
                if let Some(index) = capture {
                    instructions.push(PreparedInstruction::Ready(Instruction::Close(index)));
                }
                if let Some(GroupAssertion::Lookbehind(negative)) = assertion {
                    if quantifier(&source[cursor..]).is_some() {
                        return None;
                    }
                    // Equal-width branches have fixed input positions and no
                    // internal capture reads. Counted captures retain their
                    // leftmost backward iteration (Assertion, 22.2.2.8).
                    let width = if let Some(width) =
                        fixed_lookbehind_width(&mut instructions, entry, true)
                    {
                        LookbehindWidth::Fixed(width)
                    } else {
                        outside_reference_lookbehind_width(
                            &mut instructions,
                            entry,
                            capture_start..capture_count,
                            bindings.groups,
                        )?
                    };
                    let original = std::mem::replace(
                        &mut instructions[entry],
                        PreparedInstruction::Ready(Instruction::Nop),
                    );
                    let alternatives = match original {
                        PreparedInstruction::Ready(Instruction::Choice(alternatives)) => {
                            alternatives
                        }
                        PreparedInstruction::Ready(Instruction::Nop) => Box::new([]),
                        _ => return None,
                    };
                    let control = instructions.len();
                    instructions[entry] = PreparedInstruction::Ready(Instruction::Jump(control));
                    instructions.push(PreparedInstruction::Ready(Instruction::Lookbehind {
                        body: entry,
                        width,
                        negative,
                        alternatives,
                    }));
                    last_complex_group = Some(control);
                } else if let Some(GroupAssertion::Lookahead(negative)) = assertion {
                    // Quantified lookahead is Annex B syntax, not core grammar.
                    if quantifier(&source[cursor..]).is_some() {
                        return None;
                    }
                    // The fixed proof excludes capture-dependent continuations.
                    // Lookahead frames retain forward capture effects (22.2.2.8).
                    let fixed_width = fixed_lookbehind_width(&mut instructions, entry, false);
                    let control = instructions.len() + 1;
                    let original = std::mem::replace(
                        &mut instructions[entry],
                        PreparedInstruction::Ready(Instruction::Jump(control)),
                    );
                    instructions.push(PreparedInstruction::Ready(Instruction::LookaheadEnd(
                        control,
                    )));
                    instructions.push(PreparedInstruction::Ready(Instruction::Lookahead {
                        body: entry,
                        head: control + 1,
                        end: control + 2,
                        negative,
                        fixed_width,
                    }));
                    instructions.push(match original {
                        PreparedInstruction::Ready(Instruction::Nop) => {
                            PreparedInstruction::Ready(Instruction::Jump(entry + 1))
                        }
                        choice => choice,
                    });
                    last_complex_group = Some(control);
                } else if let Some((bounds, consumed)) = quantifier(&source[cursor..]) {
                    let end = instructions.len();
                    let start = if capture.is_some() { entry - 1 } else { entry };
                    if bounds.0 == Some(0) && bounds.1 == Some(0) {
                        // RepeatMatcher's maximum-zero case cannot run any
                        // already lowered native body (22.2.2.3.1).
                        instructions[start] = PreparedInstruction::Ready(Instruction::Jump(end));
                        instructions.push(PreparedInstruction::Ready(Instruction::SkipGroup {
                            captures: capture_start..capture_count,
                            body: start,
                            end: end + 1,
                        }));
                        last_complex_group = Some(end);
                    } else if quantify_zero_width_assertion_wrapper(
                        &mut instructions,
                        start..end,
                        bounds,
                        capture_start..capture_count,
                        &mut progresses,
                    )
                    .is_some()
                    {
                        last_complex_group = Some(instructions.len() - 2);
                    } else if last_complex_group.is_some_and(|pc| pc >= start)
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
                            capture_start..capture_count,
                            &mut progresses,
                        )?;
                        last_complex_group = Some(instructions.len() - 2);
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
    fn mixed_empty_reference_lookbehind_execution_snapshot() {
        let mut rows = String::new();
        for source in [
            r"(a)(?<=((b)\3\1){2})c",
            r"(a)(?<=(b\1\2){2})c",
            r"(a)(?<=(\1(b)\3){2})c",
            r"(a)(?<=(\1(\2)){2})b",
            r"(ab)(?<=((b)\3\1){2})c",
            r"(a)(?<=(\3b()\1){2})c",
            r"(a)(?<=((b)\3\1){2}?)c",
            r"(a)(?<=((b)\3\1){0})c",
            r"(a)(?<!((b)\3\1){2}q)c",
            r"(a)(?<=((b)\3\1\B){2})c",
            r"()(?<=((())\3\1){2})b",
            r"(?<=((a)\2\3){2})(b)",
            r"(a(?<=((a)\3\1){2}))b",
        ] {
            for (ignore_case, multiline, dot_all) in [
                (false, false, false),
                (true, true, false),
                (false, false, true),
                (true, false, true),
            ] {
                let matcher = ordinary(source, ignore_case, multiline, dot_all);
                for text in [
                    "", "babac", "ababc", "babbabc", "aab", "baxac", "bAbAc", "ab", "b", "aaaab",
                ] {
                    let input = JsString::from(text);
                    for (start, sticky) in [(0, false), (0, true), (1, true)] {
                        writeln!(rows,"{source:?} i={ignore_case} m={multiline} s={dot_all} input={input:?} start={start} sticky={sticky} {:?}",matcher.find(&input,start,sticky)).unwrap();
                    }
                }
            }
        }
        insta::assert_snapshot!(rows);
    }

    #[test]
    fn mixed_empty_reference_lookbehind_preserves_imports_internal_offsets_and_owner_guards() {
        for (source, text, range, captures) in [
            (
                r"(a)(?<=((b)\3\1){2})c",
                "babac",
                3..5,
                vec![Some(3..4), Some(0..2), Some(0..1)],
            ),
            (
                r"(a)(?<=(b\1\2){2})c",
                "babac",
                3..5,
                vec![Some(3..4), Some(0..2)],
            ),
            (
                r"(a)(?<=(\1(a)\3){2})c",
                "aaaac",
                3..5,
                vec![Some(3..4), Some(0..2), Some(1..2)],
            ),
            (
                r"(a)(?<=(\1(\2)){2})b",
                "aab",
                1..3,
                vec![Some(1..2), Some(0..1), Some(1..1)],
            ),
            (
                r"(ab)(?<=((b)\3\1){2})c",
                "babbabc",
                4..7,
                vec![Some(4..6), Some(0..3), Some(0..1)],
            ),
            (
                r"(a)(?<=(\3b()\1){2})c",
                "babac",
                3..5,
                vec![Some(3..4), Some(0..2), Some(1..1)],
            ),
            (
                r"(a)(?<!((b)\3\1){2}q)c",
                "babaac",
                4..6,
                vec![Some(4..5), None, None],
            ),
            (
                r"()(?<=((())\3\1){2})b",
                "b",
                0..1,
                vec![Some(0..0), Some(0..0), Some(0..0), Some(0..0)],
            ),
            (
                r"(?<=((a)\2\3){2})(b)",
                "aab",
                2..3,
                vec![Some(0..1), Some(0..1), Some(2..3)],
            ),
            (
                r"(a(?<=((a)\3\1){2}))b",
                "aab",
                1..3,
                vec![Some(1..2), Some(0..1), Some(0..1)],
            ),
        ] {
            let found = ordinary(source, false, false, false)
                .find(&JsString::from(text), 0, false)
                .unwrap();
            assert_eq!(found.range, range, "{source}");
            assert_eq!(&*found.captures, &*captures, "{source}");
        }
        assert!(
            ordinary(r"(a)(?<=(\1(b)\3){2})c", false, false, false)
                .find(&JsString::from("ababc"), 0, false)
                .is_none()
        );
        let input = JsString::from_code_units(vec![0xdc00, 0xd800, 0xdc00, 0xd800, 0x63]);
        let found = ordinary(r"([\uD800])(?<=(([\uDC00])\3\1){2})c", false, false, false)
            .find(&input, 3, true)
            .unwrap();
        assert_eq!(found.range, 3..5);
        assert_eq!(&*found.captures, &[Some(3..4), Some(0..2), Some(0..1)]);
        for source in [
            r"(a)(?<=((b)\3\1){1,2})c",
            r"(a)(?<=(\1\3(b)){2})c",
            r"(a)(?<=((b)\3\1){2}|b)c",
        ] {
            assert!(
                RegExpBackreferenceMatcher::compile_ordinary_with_named_bindings_and_work(
                    &JsString::from(source),
                    false,
                    false,
                    false,
                    RegExpBackreferenceNamedBindings::default(),
                    |_| Ok::<_, ()>(())
                )
                .unwrap()
                .is_none(),
                "{source}"
            );
        }
    }

    #[test]
    fn mixed_empty_reference_lookbehind_deep_scopes_huge_empty_counts_negative_undo_and_work_are_flat()
     {
        let source =
            "(a)(?<=(".to_owned() + &"(".repeat(100000) + "b" + &")".repeat(100000) + r"\3\1){2})c";
        let matcher = ordinary(&source, false, false, false);
        let copy = matcher.clone();
        drop(matcher);
        let found = copy.find(&JsString::from("babac"), 3, true).unwrap();
        assert_eq!(found.range, 3..5);
        assert_eq!(found.captures.len(), 100002);
        assert_eq!(found.captures[0], Some(3..4));
        assert_eq!(found.captures[1], Some(0..2));
        assert!(found.captures[2..].iter().all(|r| *r == Some(0..1)));
        drop(copy);
        let negative = source.replace("(?<=", "(?<!").replace("{2})c", "{2}q)c");
        let found = ordinary(&negative, false, false, false)
            .find(&JsString::from("babaac"), 4, true)
            .unwrap();
        assert_eq!(found.captures[0], Some(4..5));
        assert!(found.captures[1..].iter().all(Option::is_none));
        let source = format!(r"()(?<=((())\3\1){{{}}})b", "9".repeat(100));
        let found = ordinary(&source, false, false, false)
            .find(&JsString::from("b"), 0, true)
            .unwrap();
        assert!(found.captures.iter().all(|r| *r == Some(0..0)));
        let matcher = ordinary(r"(a)(?<=((b)\3\1){50000})c", false, false, false);
        let input = JsString::from(("ba".repeat(50000) + "c").as_str());
        let found = matcher.find(&input, 99999, true).unwrap();
        assert_eq!(found.captures[1], Some(0..2));
        let mut work = 0;
        assert!(
            matcher
                .find_with_work(&input, 99999, true, |n| {
                    work += n;
                    if work > 1000 { Err(()) } else { Ok(()) }
                })
                .is_err()
        );
    }

    #[test]
    fn forward_target_lookbehind_execution_snapshot() {
        let mut rows = String::new();
        for source in [
            r"(?<=(\2(a)){2})b",
            r"(?<=(\2(a)){2}?)b",
            r"(?<=(\2([ab])){2})c",
            r"(?<=(\2(ab)){2})c",
            r"(?<=(\2(\3(a))){2})b",
            r"(?<=(\2(a)\2){2})b",
            r"(?<=(\2a()){2})b",
            r"(?<=(\2(a)){0})b",
            r"(?<!(\2(a)){2}q)b",
            r"(?<=((a)\1){2}(?=(\4(b)){2}))b",
            r"(?=(\2(a)){2})a",
            r"(b)(?<=(\3(a)){2}\1)c",
        ] {
            for (ignore_case, multiline, dot_all) in [
                (false, false, false),
                (true, true, false),
                (false, false, true),
                (true, false, true),
            ] {
                let matcher = ordinary(source, ignore_case, multiline, dot_all);
                for text in [
                    "",
                    "aab",
                    "aaaab",
                    "aabbc",
                    "ababababc",
                    "aaaaxb",
                    "aAaAb",
                    "aabbb",
                ] {
                    let input = JsString::from(text);
                    for (start, sticky) in [(0, false), (0, true), (1, true)] {
                        writeln!(rows,"{source:?} i={ignore_case} m={multiline} s={dot_all} input={input:?} start={start} sticky={sticky} {:?}",matcher.find(&input,start,sticky)).unwrap();
                    }
                }
            }
        }
        insta::assert_snapshot!(rows);
    }

    #[test]
    fn forward_target_lookbehind_preserves_nested_right_ranges_empty_future_slots_and_forward_direction()
     {
        for (source, text, range, captures) in [
            (
                r"(?<=(\2(a)){2})b",
                "aaaab",
                4..5,
                vec![Some(0..2), Some(1..2)],
            ),
            (
                r"(?<=(\2([ab])){2})c",
                "aabbc",
                4..5,
                vec![Some(0..2), Some(1..2)],
            ),
            (
                r"(?<=(\2(ab)){2})c",
                "ababababc",
                8..9,
                vec![Some(0..4), Some(2..4)],
            ),
            (
                r"(?<=(\2(\3(a))){2})b",
                "aaaaaaaab",
                8..9,
                vec![Some(0..4), Some(2..4), Some(3..4)],
            ),
            (
                r"(?<=(\2(a)\2){2})b",
                "aaaab",
                4..5,
                vec![Some(0..2), Some(1..2)],
            ),
            (
                r"(?<=(\2a()){2})b",
                "aab",
                2..3,
                vec![Some(0..1), Some(1..1)],
            ),
            (r"(?<!(\2(a)){2}q)b", "aaaaxb", 5..6, vec![None, None]),
            (
                r"(?<=((a)\1){2}(?=(\4(b)){2}))b",
                "aabbb",
                2..3,
                vec![Some(0..1), Some(0..1), Some(3..4), Some(3..4)],
            ),
            (r"(?=(\2(a)){2})a", "aa", 0..1, vec![Some(1..2), Some(1..2)]),
            (
                r"(b)(?<=(\3(a)){2}\1)c",
                "aaaabc",
                4..6,
                vec![Some(4..5), Some(0..2), Some(1..2)],
            ),
        ] {
            let found = ordinary(source, false, false, false)
                .find(&JsString::from(text), 0, false)
                .unwrap();
            assert_eq!(found.range, range, "{source}");
            assert_eq!(&*found.captures, &*captures, "{source}");
        }
        assert!(
            ordinary(r"(?<=((\3a)(b)){2})c", false, false, false)
                .find(&JsString::from("ababc"), 0, false)
                .is_none()
        );
        let input = JsString::from_code_units(vec![0xd800, 0xd800, 0xd800, 0xd800, 0xdc00]);
        let found = ordinary(r"(?<=(\2([\uD800])){2})[\uDC00]", false, false, false)
            .find(&input, 4, true)
            .unwrap();
        assert_eq!(found.range, 4..5);
        assert_eq!(&*found.captures, &[Some(0..2), Some(1..2)]);
        for source in [
            r"(?<=(\2(a)){1,2})b",
            r"(a)(?<=(\1\3(b)){2})c",
            r"(?<=(\2(a)){2}|a)b",
            r"(?<=(\2(a)){2}(?<=\2))b",
        ] {
            assert!(
                RegExpBackreferenceMatcher::compile_ordinary_with_named_bindings_and_work(
                    &JsString::from(source),
                    false,
                    false,
                    false,
                    RegExpBackreferenceNamedBindings::default(),
                    |_| Ok::<_, ()>(())
                )
                .unwrap()
                .is_none(),
                "{source}"
            );
        }
    }

    #[test]
    fn forward_target_lookbehind_deep_owned_scopes_clones_drop_negative_undo_and_counts_stay_flat()
    {
        let source =
            "(?<=(\\2".to_owned() + &"(".repeat(100000) + "a" + &")".repeat(100000) + "){2})b";
        let matcher = ordinary(&source, false, false, false);
        let copy = matcher.clone();
        drop(matcher);
        let found = copy.find(&JsString::from("aaaab"), 4, true).unwrap();
        assert_eq!(found.range, 4..5);
        assert_eq!(found.captures.len(), 100001);
        assert_eq!(found.captures[0], Some(0..2));
        assert!(found.captures[1..].iter().all(|r| *r == Some(1..2)));
        drop(copy);
        let source =
            "(?<!(\\2".to_owned() + &"(".repeat(100000) + "a" + &")".repeat(100000) + "){2}q)b";
        let found = ordinary(&source, false, false, false)
            .find(&JsString::from("aaaaxb"), 5, true)
            .unwrap();
        assert_eq!(found.range, 5..6);
        assert!(found.captures.iter().all(Option::is_none));
        let matcher = ordinary(r"(?<=(\2(a)){50000})b", false, false, false);
        let input = JsString::from(("a".repeat(100000) + "b").as_str());
        let found = matcher.find(&input, 100000, true).unwrap();
        assert_eq!(&*found.captures, &[Some(0..2), Some(1..2)]);
        let mut work = 0;
        assert!(
            matcher
                .find_with_work(&input, 100000, true, |n| {
                    work += n;
                    if work > 1000 { Err(()) } else { Ok(()) }
                })
                .is_err()
        );
    }

    #[test]
    fn counted_open_lookbehind_execution_snapshot() {
        let mut rows = String::new();
        for source in [
            r"(?<=(a\1){2})b",
            r"(?<=((a)\1){2})b",
            r"(?<=((a)\2\1){2})b",
            r"(?<=((\2)a){2})b",
            r"(?<=(a\1){2}?)b",
            r"(?<=([ab]\1){2})c",
            r"(?<=(a\1){0})b",
            r"(?<!(a\1){2}q)b",
            r"(?<=((a)\1){2}(?=(b\3){2}))b",
            r"(?=(a\1){2})a",
            r"(b)(?<=(a\2){2}\1)c",
            r"(?<=((a)\1\B){2})b",
        ] {
            for (ignore_case, multiline, dot_all) in [
                (false, false, false),
                (true, true, false),
                (false, false, true),
                (true, false, true),
            ] {
                let matcher = ordinary(source, ignore_case, multiline, dot_all);
                for text in ["", "ab", "aab", "aaaab", "abc", "aaxb", "aAb", "aabbb"] {
                    let input = JsString::from(text);
                    for (start, sticky) in [(0, false), (0, true), (1, true)] {
                        writeln!(rows,"{source:?} i={ignore_case} m={multiline} s={dot_all} input={input:?} start={start} sticky={sticky} {:?}",matcher.find(&input,start,sticky)).unwrap();
                    }
                }
            }
        }
        insta::assert_snapshot!(rows);
    }

    #[test]
    fn counted_open_lookbehind_keeps_unclosed_reads_empty_and_both_assertion_directions_exact() {
        for (source, text, range, captures) in [
            (r"(?<=(a\1){2})b", "aab", 2..3, vec![Some(0..1)]),
            (
                r"(?<=((a)\1){2})b",
                "aab",
                2..3,
                vec![Some(0..1), Some(0..1)],
            ),
            (
                r"(?<=((a)\2\1){2})b",
                "aab",
                2..3,
                vec![Some(0..1), Some(0..1)],
            ),
            (
                r"(?<=((\2)a){2})b",
                "aab",
                2..3,
                vec![Some(0..1), Some(0..0)],
            ),
            (r"(?<!(a\1){2}q)b", "aaxb", 3..4, vec![None]),
            (
                r"(?<=((a)\1){2}(?=(b\3){2}))b",
                "aabbb",
                2..3,
                vec![Some(0..1), Some(0..1), Some(3..4)],
            ),
            (r"(?=(a\1){2})a", "aa", 0..1, vec![Some(1..2)]),
            (
                r"(b)(?<=(a\2){2}\1)c",
                "aabc",
                2..4,
                vec![Some(2..3), Some(0..1)],
            ),
        ] {
            let found = ordinary(source, false, false, false)
                .find(&JsString::from(text), 0, false)
                .unwrap();
            assert_eq!(found.range, range, "{source}");
            assert_eq!(&*found.captures, &*captures, "{source}");
        }
        let input = JsString::from_code_units(vec![0xd800, 0xd800, 0xdc00]);
        let found = ordinary(r"(?<=([\uD800]\1){2})[\uDC00]", false, false, false)
            .find(&input, 2, true)
            .unwrap();
        assert_eq!(found.range, 2..3);
        assert_eq!(&*found.captures, &[Some(0..1)]);
        for source in [
            r"(?<=(\2(a)){1,2})b",
            r"(?<=((\3a)(b)){1,2})c",
            r"(?<=(a\1){1,2})b",
            r"(a)(?<=(b\1\2){1,2})c",
            r"(?<=(a\1){2}|a)b",
        ] {
            assert!(
                RegExpBackreferenceMatcher::compile_ordinary_with_named_bindings_and_work(
                    &JsString::from(source),
                    false,
                    false,
                    false,
                    RegExpBackreferenceNamedBindings::default(),
                    |_| Ok::<_, ()>(())
                )
                .unwrap()
                .is_none(),
                "{source}"
            );
        }
    }

    #[test]
    fn counted_open_lookbehind_deep_owned_scopes_copies_drop_negative_undo_and_large_counts_stay_flat()
     {
        let source =
            "(?<=(".to_owned() + &"(".repeat(100000) + "a" + &")".repeat(100000) + r"\1){2})b";
        let matcher = ordinary(&source, false, false, false);
        let copy = matcher.clone();
        drop(matcher);
        let found = copy.find(&JsString::from("aab"), 2, true).unwrap();
        assert_eq!(found.range, 2..3);
        assert_eq!(found.captures.len(), 100001);
        assert!(found.captures.iter().all(|r| *r == Some(0..1)));
        drop(copy);
        let source =
            "(?<!(".to_owned() + &"(".repeat(100000) + "a" + &")".repeat(100000) + r"\1){2}q)b";
        let found = ordinary(&source, false, false, false)
            .find(&JsString::from("aaxb"), 3, true)
            .unwrap();
        assert_eq!(found.range, 3..4);
        assert!(found.captures.iter().all(Option::is_none));
        let matcher = ordinary(r"(?<=(a\1){100000})b", false, false, false);
        let input = JsString::from(("a".repeat(100000) + "b").as_str());
        assert_eq!(
            &*matcher.find(&input, 100000, true).unwrap().captures,
            &[Some(0..1)]
        );
        let mut work = 0;
        assert!(
            matcher
                .find_with_work(&input, 100000, true, |n| {
                    work += n;
                    if work > 1000 { Err(()) } else { Ok(()) }
                })
                .is_err()
        );
    }

    #[test]
    fn lookbehind_local_future_reads_execution_snapshot() {
        let mut rows = String::new();
        for source in [
            r"(?<=((a)\2){2})b",
            r"(?<=((a)\2){2}?)b",
            r"(?<=((ab)\2){2})c",
            r"(?<=((a)(b)\2\3){2})c",
            r"(?<=((a)\2\2){2})b",
            r"(?<=(([ab])\2){2})c",
            r"(?<=((.)\2){2})b",
            r"(?<=((a)\2\B){2})b",
            r"(?<=((a)\2){0})b",
            r"(?<!((a)\2){2}q)b",
            r"(?<=((a)\2){2}(?=(b)))b",
            r"(b)(?<=((a)\3){2}\1)c",
        ] {
            for (ignore_case, multiline, dot_all) in [
                (false, false, false),
                (true, true, false),
                (false, false, true),
                (true, false, true),
            ] {
                let matcher = ordinary(source, ignore_case, multiline, dot_all);
                for text in ["", "ab", "aab", "aaaab", "ababc", "aaxb", "aAb", "a\nb"] {
                    let input = JsString::from(text);
                    for (start, sticky) in [(0, false), (0, true), (1, true)] {
                        writeln!(rows,"{source:?} i={ignore_case} m={multiline} s={dot_all} input={input:?} start={start} sticky={sticky} {:?}",matcher.find(&input,start,sticky)).unwrap();
                    }
                }
            }
        }
        insta::assert_snapshot!(rows);
    }

    #[test]
    fn lookbehind_local_future_reads_keep_backward_ranges_and_forward_assertion_direction() {
        for (source, text, range, captures) in [
            (
                r"(?<=((a)\2){2})b",
                "aab",
                2..3,
                vec![Some(0..1), Some(0..1)],
            ),
            (
                r"(?<=((ab)\2){2})c",
                "ababc",
                4..5,
                vec![Some(0..2), Some(0..2)],
            ),
            (
                r"(?<=((a)(b)\2\3){2})c",
                "ababc",
                4..5,
                vec![Some(0..2), Some(0..1), Some(1..2)],
            ),
            (r"(?<!((a)\2){2}q)b", "aaxb", 3..4, vec![None, None]),
            (
                r"(?<=((a)\2){2}(?=(b)))b",
                "aab",
                2..3,
                vec![Some(0..1), Some(0..1), Some(2..3)],
            ),
            (
                r"(?=(((a)\3){2}))a",
                "aaaab",
                0..1,
                vec![Some(0..4), Some(2..4), Some(2..3)],
            ),
            (
                r"(b)(?<=((a)\3){2}\1)c",
                "aabc",
                2..4,
                vec![Some(2..3), Some(0..1), Some(0..1)],
            ),
        ] {
            let found = ordinary(source, false, false, false)
                .find(&JsString::from(text), 0, false)
                .unwrap();
            assert_eq!(found.range, range, "{source}");
            assert_eq!(&*found.captures, &*captures, "{source}");
        }
        for source in [
            r"(?<=(\2(a)){1,2})b",
            r"(?<=((a)\1){1,2})b",
            r"(?<=((a)\2){1,2})b",
            r"(a)(?<=(\1(b)\3){1,2})c",
            r"(?<=((a)\2){2}(?=(((a)\5){2})))b",
        ] {
            assert!(
                RegExpBackreferenceMatcher::compile_ordinary_with_named_bindings_and_work(
                    &JsString::from(source),
                    false,
                    false,
                    false,
                    RegExpBackreferenceNamedBindings::default(),
                    |_| Ok::<_, ()>(())
                )
                .unwrap()
                .is_none(),
                "{source}"
            );
        }
        let input = JsString::from_code_units(vec![0xd800, 0xd800, 0xdc00]);
        let found = ordinary(r"(?<=(([\uD800])\2){2})[\uDC00]", false, false, false)
            .find(&input, 2, true)
            .unwrap();
        assert_eq!(found.range, 2..3);
        assert_eq!(&*found.captures, &[Some(0..1), Some(0..1)]);
    }

    #[test]
    fn lookbehind_local_future_reads_deep_owned_scopes_clone_drop_completed_negative_undo_and_work_stay_flat()
     {
        let source =
            "(?<=(".to_owned() + &"(".repeat(100000) + "a" + &")".repeat(100000) + r"\2){2})b";
        let matcher = ordinary(&source, false, false, false);
        let copy = matcher.clone();
        drop(matcher);
        let found = copy.find(&JsString::from("aab"), 2, true).unwrap();
        assert_eq!(found.range, 2..3);
        assert_eq!(found.captures.len(), 100001);
        assert!(found.captures.iter().all(|r| *r == Some(0..1)));
        drop(copy);
        let source =
            "(?<!(".to_owned() + &"(".repeat(100000) + "a" + &")".repeat(100000) + r"\2){2}q)b";
        let found = ordinary(&source, false, false, false)
            .find(&JsString::from("aaxb"), 3, true)
            .unwrap();
        assert_eq!(found.range, 3..4);
        assert!(found.captures.iter().all(Option::is_none));
        let matcher = ordinary(r"(?<=((a)\2){100000})b", false, false, false);
        let input = JsString::from(("a".repeat(100000) + "b").as_str());
        let found = matcher.find(&input, 100000, true).unwrap();
        assert_eq!(&*found.captures, &[Some(0..1), Some(0..1)]);
        let mut work = 0;
        assert!(
            matcher
                .find_with_work(&input, 100000, true, |n| {
                    work += n;
                    if work > 1000 { Err(()) } else { Ok(()) }
                })
                .is_err()
        );
    }

    #[test]
    fn unrepresentable_outside_lookbehind_counts_preserve_empty_effects_and_unavailable_prefixes() {
        let huge = "9".repeat(100);
        let upper = "1".to_owned() + &"0".repeat(101);
        for bounds in [
            format!("{{{huge}}}"),
            format!("{{{huge}}}?"),
            format!("{{{huge},}}"),
            format!("{{{huge},}}?"),
            format!("{{{huge},{upper}}}"),
        ] {
            for (pattern, input, range, captures) in [
                (
                    r"()(?<=(\1)COUNT)b",
                    "qb",
                    1..2,
                    vec![Some(1..1), Some(1..1)],
                ),
                (
                    r"()(?<=(\1)COUNTa)b",
                    "ab",
                    1..2,
                    vec![Some(1..1), Some(0..0)],
                ),
                (
                    r"()()(?<=(\1\2\b)COUNT)b",
                    " b",
                    1..2,
                    vec![Some(1..1), Some(1..1), Some(1..1)],
                ),
                (r"(a(?<=\1COUNT))b", "ab", 0..2, vec![Some(0..1)]),
                (r"(?<=\1COUNT)(a)", "qa", 1..2, vec![Some(1..2)]),
                (r"()(?<!(\1)COUNTq)b", "xb", 1..2, vec![Some(1..1), None]),
                (r"(a)(?<!(\1)COUNT)b", "ab", 0..2, vec![Some(0..1), None]),
                (r"(a)(?<!(\1a)COUNT)b", "ab", 0..2, vec![Some(0..1), None]),
                (r"(a)(?<!(ab)COUNT\1)b", "ab", 0..2, vec![Some(0..1), None]),
            ] {
                let source = pattern.replace("COUNT", &bounds);
                let found = ordinary(&source, false, false, false)
                    .find(&JsString::from(input), 0, false)
                    .unwrap();
                assert_eq!(found.range, range, "{source}");
                assert_eq!(&*found.captures, &*captures, "{source}");
            }
            for pattern in [
                r"(a)(?<=(\1)COUNT)b",
                r"(a)(?<=(\1a)COUNT)b",
                r"(a)(?<=(ab)COUNT\1)b",
            ] {
                let source = pattern.replace("COUNT", &bounds);
                assert!(
                    ordinary(&source, false, false, false)
                        .find(&JsString::from("ab"), 0, false)
                        .is_none(),
                    "{source}"
                );
            }
            let source = r"()(?<=(\1\B)COUNT)b".replace("COUNT", &bounds);
            assert!(
                ordinary(&source, false, false, false)
                    .find(&JsString::from(" b"), 0, false)
                    .is_none()
            );
        }
    }

    #[test]
    fn unrepresentable_outside_lookbehind_execution_snapshot() {
        let huge = "9".repeat(100);
        let mut rows = String::new();
        for pattern in [
            r"()(?<=(\1)COUNT)b",
            r"()()(?<=(\1\2\b)COUNT)b",
            r"(a)(?<=(\1a)COUNT)b",
            r"(a)(?<!(\1a)COUNT)b",
            r"()(?<!(\1)COUNTq)b",
        ] {
            let source = pattern.replace("COUNT", &format!("{{{huge}}}"));
            let matcher = ordinary(&source, false, false, false);
            for text in ["b", " b", "ab", "xb"] {
                writeln!(
                    rows,
                    "{pattern:?} input={text:?} {:?}",
                    matcher.find(&JsString::from(text), 0, false)
                )
                .unwrap();
            }
        }
        insta::assert_snapshot!(rows);
    }

    #[test]
    fn unrepresentable_outside_lookbehind_deep_capture_effects_clones_and_negative_undo_stay_flat()
    {
        let huge = "9".repeat(10000);
        let source = "()(?<=".to_owned()
            + &"(".repeat(100000)
            + r"\1"
            + &")".repeat(100000)
            + &format!("{{{huge}}})b");
        let matcher = ordinary(&source, false, false, false);
        let copy = matcher.clone();
        drop(matcher);
        let found = copy.find(&JsString::from("qb"), 1, true).unwrap();
        assert_eq!(found.range, 1..2);
        assert_eq!(found.captures.len(), 100001);
        assert!(found.captures.iter().all(|r| *r == Some(1..1)));
        drop(copy);
        let source = "()(?<!".to_owned()
            + &"(".repeat(100000)
            + r"\1"
            + &")".repeat(100000)
            + &format!("{{{huge}}}q)b");
        let found = ordinary(&source, false, false, false)
            .find(&JsString::from("xb"), 1, true)
            .unwrap();
        assert_eq!(found.range, 1..2);
        assert_eq!(found.captures[0], Some(1..1));
        assert!(found.captures[1..].iter().all(Option::is_none));
    }

    #[test]
    fn mixed_outside_lookbehind_execution_snapshot() {
        let mut rows = String::new();
        for source in [
            r"(a)(?<=(a\1){2})b",
            r"(a)(?<=(\1a){2})b",
            r"(a)(?<=(\1a){2}?)b",
            r"(a)(?<=(a\1){1})b",
            r"(a)(b)(?<=(\1\2){2})c",
            r"(a)(bb)(?<=(\1\2){2})c",
            r"(a)(b)(?<=(\2\1){2})c",
            r"(a)(b)(?<=(\1\2a){2})c",
            r"(?=(a))(?<=(\1b){2})a",
            r"(?=(ab))(?<=(\1b){2})a",
            r"(a)(?<=(\1\B){2})b",
            r"()(?<=(\1\b){2})b",
            r"()()(?<=(\1\2){2})b",
            r"()(?<=(\1\B){2})b",
            r"(a)(?<=(\1[ab]){2})b",
            r"(a)(?<=(.\1){2})b",
            r"(a)(?<=(\1.){2})b",
            r"(a)(?<=(^\1){2})b",
            r"(a)(?<=(\1$){2})b",
            r"(a)(?<!(\1a){2}q)b",
            r"(b)(?<!(\1a){2}q)c",
            r"(a)(?<!(a\1){2})b",
            r"(?<=(\2a){2})(a)",
            r"(a(?<=(\1a){2}))b",
            r"(?=(a))(?<=(\1[ab]\1){2})a",
            r"(a)(?<=(a\1){2}(?=b))b",
            r"(a)(?<=(?<=a)(a\1){2})b",
            r"(a)(?<=(a\1){2}(?=(b)))b",
            r"(?:(a|b)(?<=(.\1){2}))+c",
            r"(µ)(?<=(.\1){2})Μ",
            r"([\uD800])(?<=([\uD800]\1){2})[\uDC00]",
            r"()()(?<=(\1\2\b){30})b",
        ] {
            for (ignore_case, multiline, dot_all) in [
                (false, false, false),
                (true, true, false),
                (false, false, true),
                (true, false, true),
            ] {
                let matcher = ordinary(source, ignore_case, multiline, dot_all);
                for text in [
                    "", "a", "b", "ab", "aaab", "abb", "abc", "ababab", "aaaabb", " abc", "a b",
                    "a\nb", "a\r\n", "µµΜ", "abbbc",
                ] {
                    let input = JsString::from(text);
                    for (start, sticky) in [(0, false), (0, true), (1, true)] {
                        writeln!(rows,"{source:?} i={ignore_case} m={multiline} s={dot_all} input={input:?} start={start} sticky={sticky} {:?}",matcher.find(&input,start,sticky)).unwrap();
                    }
                }
            }
        }
        insta::assert_snapshot!(rows);
    }
    #[test]
    fn mixed_outside_lookbehind_different_widths_first_ranges_future_imports_and_empty_effects() {
        for (source, text, range, captures) in [
            (
                r"(a)(?<=(a\1){2})b",
                "aaaaab",
                4..6,
                vec![Some(4..5), Some(1..3)],
            ),
            (
                r"(a)(b)(?<=(\1\2){2})c",
                "abababc",
                4..7,
                vec![Some(4..5), Some(5..6), Some(2..4)],
            ),
            (
                r"(a)(bb)(?<=(\1\2){2})c",
                "abbabbabbc",
                6..10,
                vec![Some(6..7), Some(7..9), Some(3..6)],
            ),
            (
                r"(?=(a))(?<=(\1b){2})a",
                "ababa",
                4..5,
                vec![Some(4..5), Some(0..2)],
            ),
            (
                r"(b)(?<!(\1a){2}q)c",
                "bababc",
                4..6,
                vec![Some(4..5), None],
            ),
            (
                r"()()(?<=(\1\2\b){30})b",
                " b",
                1..2,
                vec![Some(1..1), Some(1..1), Some(1..1)],
            ),
            (
                r"(?<=(\2a){2})(a)",
                "aaa",
                2..3,
                vec![Some(0..1), Some(2..3)],
            ),
        ] {
            let found = ordinary(source, false, false, false)
                .find(&JsString::from(text), 0, false)
                .unwrap();
            assert_eq!(found.range, range, "{source}");
            assert_eq!(&*found.captures, &*captures, "{source}");
        }
        let found = ordinary(r"(?=(a))(?<=(?:\1b){2})a", false, false, false)
            .find(&JsString::from("ababa"), 0, false)
            .unwrap();
        assert_eq!(found.range, 4..5);
        assert_eq!(&*found.captures, &[Some(4..5)]);
        for source in [
            r"(a)(?<=(a\1){1,2})b",
            r"(?<=((a)\2){1,2})b",
            r"(a)(?<=(\1(\2)){1,2})b",
            r"(a)(?<=(a\1){2}|a)b",
            r"(a)(?<=(a\1){2}(?<=\1))b",
            r"(a)(?<=(a\1){2}(?=\1))b",
        ] {
            assert!(
                RegExpBackreferenceMatcher::compile_ordinary_with_named_bindings_and_work(
                    &JsString::from(source),
                    false,
                    false,
                    false,
                    RegExpBackreferenceNamedBindings::default(),
                    |_| Ok::<_, ()>(())
                )
                .unwrap()
                .is_none(),
                "{source}"
            );
        }
        let input = JsString::from_code_units(vec![0xd800, 0xd800, 0xd800, 0xd800, 0xd800, 0xdc00]);
        let found = ordinary(
            r"([\uD800])(?<=([\uD800]\1){2})[\uDC00]",
            false,
            false,
            false,
        )
        .find(&input, 4, true)
        .unwrap();
        assert_eq!(found.range, 4..6);
        assert_eq!(&*found.captures, &[Some(4..5), Some(1..3)]);
    }
    #[test]
    fn mixed_outside_lookbehind_deep_scopes_negative_restore_large_ranges_and_counts_are_flat() {
        let source =
            "(b)(?<=".to_owned() + &"(".repeat(100000) + r"a\1" + &")".repeat(100000) + "{2})c";
        let matcher = ordinary(&source, false, false, false);
        let copy = matcher.clone();
        drop(matcher);
        let found = copy.find(&JsString::from("qababc"), 4, true).unwrap();
        assert_eq!(found.range, 4..6);
        assert_eq!(found.captures.len(), 100001);
        assert_eq!(found.captures[0], Some(4..5));
        assert!(found.captures[1..].iter().all(|r| *r == Some(1..3)));
        drop(copy);
        let source =
            "(b)(?<!".to_owned() + &"(".repeat(100000) + r"\1a" + &")".repeat(100000) + "{2}q)c";
        let found = ordinary(&source, false, false, false)
            .find(&JsString::from("bababc"), 4, true)
            .unwrap();
        assert_eq!(found.range, 4..6);
        assert_eq!(found.captures[0], Some(4..5));
        assert!(found.captures[1..].iter().all(Option::is_none));
        let source = format!(r"()()(?<=(\1\2\b){{{}}})b", usize::MAX);
        let found = ordinary(&source, false, false, false)
            .find(&JsString::from("b"), 0, true)
            .unwrap();
        assert_eq!(&*found.captures, &[Some(0..0), Some(0..0), Some(0..0)]);
        let source = format!(r"(a)(?<=(\1a){{{}}})b", usize::MAX);
        assert!(
            ordinary(&source, false, false, false)
                .find(&JsString::from("ab"), 0, true)
                .is_none()
        );
        let source = format!(r"(a)(?<!(\1a){{{}}})b", usize::MAX);
        let found = ordinary(&source, false, false, false)
            .find(&JsString::from("ab"), 0, true)
            .unwrap();
        assert_eq!(found.range, 0..2);
        assert_eq!(&*found.captures, &[Some(0..1), None]);
        let matcher = ordinary(r"(?=(a+))(?<=(\1b){2})a", false, false, false);
        let a = "a".repeat(50000);
        let input = JsString::from((a.clone() + "b" + &a + "b" + &a).as_str());
        let found = matcher.find(&input, 100002, true).unwrap();
        assert_eq!(found.range, 100002..100003);
        assert_eq!(&*found.captures, &[Some(100002..150002), Some(0..50001)]);
        let mut work = 0;
        assert!(
            matcher
                .find_with_work(&input, 100002, true, |n| {
                    work += n;
                    if work > 1000 { Err(()) } else { Ok(()) }
                })
                .is_err()
        );
    }

    #[test]
    fn fixed_units_outside_lookbehind_execution_snapshot() {
        let mut rows = String::new();
        for source in [
            r"(a)(?<=a{2}\1)b",
            r"(a)(?<=[a]{2}\1)b",
            r"(b)(?<=(ab){2}\1)c",
            r"(b)(?<=\1(ab){2})c",
            r"(a)(?<=a{2}(\1){2})b",
            r"(a)(?<=(a){2}(\1){2})b",
            r"(a)(?<=(a\B){2}\1)b",
            r"(a)(?<=(^){2}\1)b",
            r"(a)(?<=\1(\B){2})b",
            r"(a)(?<=(\b){2}\1)b",
            r"(a)(?<=(()\3){2}\1)b",
            r"(a)(?<=(\1b){0}\1)b",
            r"(a)(?<!(a){2}\1q)b",
            r"(a)(?<!a{2}\1)b",
            r"(a)(?<!a{2}\1q)b",
            r"(?<=(a){2}\2)(a)",
            r"(a(?<=a{2}\1))b",
            r"()(?<=(a){2}\1)b",
            r"()(?<=(\b){2}\1)b",
            r"(a)(?<=(a){0}\1)b",
            r"(a)(?<=(ab){0}\1)b",
            r"(a)(?<=(a){2}?\1)b",
            r"(a)(?<=.{2}\1)b",
            r"(a)(?<=[^b]{2}\1)b",
            r"(a)(?<=a{2}\1(?=b))b",
            r"(a)(?<=(?<=a)a{2}\1)b",
            r"(a)(?<=a{2}\1(?=(b)))b",
            r"(?:(a|b)(?<=[ab]{2}\1))+c",
            r"(µ)(?<=(µ){2}\1)Μ",
            r"([\uD800])(?<=[\uD800]{2}\1)[\uDC00]",
            r"(a)(?<=(a){30}\1)b",
            r"(a)(?<!(a){30}\1)b",
        ] {
            for (ignore_case, multiline, dot_all) in [
                (false, false, false),
                (true, true, false),
                (false, false, true),
                (true, false, true),
            ] {
                let matcher = ordinary(source, ignore_case, multiline, dot_all);
                for text in [
                    "", "a", "b", "ab", "aaab", "abb", "abc", "ababab", "aaaabb", " abc", "a b",
                    "a\nb", "a\r\n", "µµΜ", "abbbc",
                ] {
                    let input = JsString::from(text);
                    for (start, sticky) in [(0, false), (0, true), (1, true)] {
                        writeln!(rows,"{source:?} i={ignore_case} m={multiline} s={dot_all} input={input:?} start={start} sticky={sticky} {:?}",matcher.find(&input,start,sticky)).unwrap();
                    }
                }
            }
        }
        insta::assert_snapshot!(rows);
    }
    #[test]
    fn fixed_units_outside_lookbehind_capture_offsets_empty_units_and_skipped_reads() {
        for (source, text, range, captures) in [
            (
                r"(a)(?<=(a){2}(\1){2})b",
                "aaaab",
                3..5,
                vec![Some(3..4), Some(0..1), Some(2..3)],
            ),
            (
                r"(b)(?<=(ab){2}\1)c",
                "ababbc",
                4..6,
                vec![Some(4..5), Some(0..2)],
            ),
            (
                r"(b)(?<=\1(ab){2})c",
                "bababc",
                4..6,
                vec![Some(4..5), Some(1..3)],
            ),
            (
                r"(a)(?<=(a\B){2}\1)b",
                "aaab",
                2..4,
                vec![Some(2..3), Some(0..1)],
            ),
            (
                r"(a)(?<=\1(\B){2})b",
                "qab",
                1..3,
                vec![Some(1..2), Some(2..2)],
            ),
            (
                r"(a)(?<=(()\3){2}\1)b",
                "qab",
                1..3,
                vec![Some(1..2), Some(1..1), Some(1..1)],
            ),
            (r"(a)(?<=(\1b){0}\1)b", "qab", 1..3, vec![Some(1..2), None]),
            (r"(a)(?<!(a){2}\1q)b", "aaab", 2..4, vec![Some(2..3), None]),
        ] {
            let found = ordinary(source, false, false, false)
                .find(&JsString::from(text), 0, false)
                .unwrap();
            assert_eq!(found.range, range, "{source}");
            assert_eq!(&*found.captures, &*captures, "{source}");
        }
        for source in [
            r"(a)(?<=(a){1,2}\1)b",
            r"(a)(?<=(a\1){1,2})b",
            r"(a)(?<=a{2}\1|a)b",
            r"(a)(?<=a{2}(?<=\1))b",
            r"(a)(?<=a{2}(?=\1)\1)b",
            r"(a)(?<=a{2}(\1){1,2})b",
        ] {
            assert!(
                RegExpBackreferenceMatcher::compile_ordinary_with_named_bindings_and_work(
                    &JsString::from(source),
                    false,
                    false,
                    false,
                    RegExpBackreferenceNamedBindings::default(),
                    |_| Ok::<_, ()>(())
                )
                .unwrap()
                .is_none(),
                "{source}"
            );
        }
        let input = JsString::from_code_units(vec![0xd800, 0xd800, 0xd800, 0xdc00]);
        let found = ordinary(r"([\uD800])(?<=[\uD800]{2}\1)[\uDC00]", false, false, false)
            .find(&input, 2, true)
            .unwrap();
        assert_eq!(found.range, 2..4);
        assert_eq!(&*found.captures, &[Some(2..3)]);
    }
    #[test]
    fn fixed_units_outside_lookbehind_deep_captures_negative_restore_clones_and_work_are_flat() {
        let source =
            "(a)(?<=".to_owned() + &"(".repeat(100000) + "a" + &")".repeat(100000) + r"{2}\1)b";
        let matcher = ordinary(&source, false, false, false);
        let copy = matcher.clone();
        drop(matcher);
        let found = copy.find(&JsString::from("aaaab"), 3, true).unwrap();
        assert_eq!(found.range, 3..5);
        assert_eq!(found.captures.len(), 100001);
        assert_eq!(found.captures[0], Some(3..4));
        assert!(found.captures[1..].iter().all(|r| *r == Some(1..2)));
        drop(copy);
        let source =
            "(a)(?<!".to_owned() + &"(".repeat(100000) + "a" + &")".repeat(100000) + r"{2}\1q)b";
        let found = ordinary(&source, false, false, false)
            .find(&JsString::from("aaaab"), 3, true)
            .unwrap();
        assert_eq!(found.range, 3..5);
        assert_eq!(found.captures[0], Some(3..4));
        assert!(found.captures[1..].iter().all(Option::is_none));
        let source = format!(r"(a)(?<=(){{{}}}\1)b", usize::MAX);
        let found = ordinary(&source, false, false, false)
            .find(&JsString::from("ab"), 0, true)
            .unwrap();
        assert_eq!(&*found.captures, &[Some(0..1), Some(0..0)]);
        let matcher = ordinary(r"(a+)(?<=(b){0}\1)b", false, false, false);
        let input = JsString::from(("a".repeat(50000) + "b").as_str());
        let found = matcher.find(&input, 0, true).unwrap();
        assert_eq!(found.range, 0..50001);
        assert_eq!(&*found.captures, &[Some(0..50000), None]);
        let mut work = 0;
        assert!(
            matcher
                .find_with_work(&input, 0, true, |n| {
                    work += n;
                    if work > 1000 { Err(()) } else { Ok(()) }
                })
                .is_err()
        );
    }

    #[test]
    fn counted_outside_reference_lookbehind_execution_snapshot() {
        let mut rows = String::new();
        for source in [
            r"(a)(?<=\1{2})b",
            r"(a)(?<=\1{2}?)b",
            r"(a)(?<=(\1){2})b",
            r"(a)(?<=(\1\1){2})b",
            r"(..)(?<=\1{3})",
            r"(a)(?<=a\1{2})b",
            r"(a)(?<=\1{2}a)b",
            r"(a)(?<!\1{2})b",
            r"(a)(?<!(\1){2}q)b",
            r"(?<=\1{2})(a)",
            r"(?<=(\2){2})a(a)",
            r"(a(?<=\1{2}))b",
            r"()(?<=(\1){2})b",
            r"()(?<=\1{30})b",
            r"(a)(?<=(\1){0}\1)b",
            r"(a)(?<=(\1){1}\1)b",
            r"(a)(?<=(\1\1){2}?)b",
            r"(a)(?<=\1{2}\b)b",
            r"(a)(?<=\1{2}\B)b",
            r"(a)(?<=^\1{2})b",
            r"(a)(?<=\1{2}(?=b))b",
            r"(a)(?<=\1{2}(?=(b)))b",
            r"(a)(?<=(?<=a)\1{2})b",
            r"(a)(?<=\1{2}(?<=a))b",
            r"(a)(?<=(a|b){0}\1{2})b",
            r"(?:(a|b)(?<=\1{2}))+c",
            r"(µ)(?<=\1{2})Μ",
            r"([\uD800])(?<=\1{2})[\uDC00]",
            r"(a)(?<=(\1\1){1})b",
            r"(?=(a))(?<=\1{2})a",
            r"(a)(?<=\1{30})b",
            r"(a)(?<!\1{30})b",
        ] {
            for (ignore_case, multiline, dot_all) in [
                (false, false, false),
                (true, true, false),
                (false, false, true),
                (true, false, true),
            ] {
                let matcher = ordinary(source, ignore_case, multiline, dot_all);
                for text in [
                    "", "a", "b", "ab", "aaab", "abb", "abc", "ababab", "aaaabb", " abc", "a b",
                    "a\nb", "a\r\n", "µµΜ", "abbbc",
                ] {
                    let input = JsString::from(text);
                    for (start, sticky) in [(0, false), (0, true), (1, true)] {
                        writeln!(rows,"{source:?} i={ignore_case} m={multiline} s={dot_all} input={input:?} start={start} sticky={sticky} {:?}",matcher.find(&input,start,sticky)).unwrap();
                    }
                }
            }
        }
        insta::assert_snapshot!(rows);
    }

    #[test]
    fn counted_outside_reference_lookbehind_preserves_owned_ranges_open_forward_and_negative_slots()
    {
        for (source, text, range, captures) in [
            (
                r"(a)(?<=(\1){2})b",
                "aaab",
                2..4,
                vec![Some(2..3), Some(1..2)],
            ),
            (
                r"(a)(?<=(\1\1){2})b",
                "aaaaab",
                4..6,
                vec![Some(4..5), Some(1..3)],
            ),
            (r"(?<=\1{2})(a)", "qa", 1..2, vec![Some(1..2)]),
            (r"(?<=(\2){2})a()", "qa", 1..2, vec![Some(1..1), Some(2..2)]),
            (r"(a(?<=\1{2}))b", "qab", 1..3, vec![Some(1..2)]),
            (r"()(?<=(\1){2})b", "qb", 1..2, vec![Some(1..1), Some(1..1)]),
            (r"(a)(?<!(\1){2}q)b", "aaab", 2..4, vec![Some(2..3), None]),
            (r"(a)(?<=(\1){0}\1)b", "qab", 1..3, vec![Some(1..2), None]),
            (
                r"(a)(?<=\1{2}(?=(b)))b",
                "aaab",
                2..4,
                vec![Some(2..3), Some(3..4)],
            ),
        ] {
            let found = ordinary(source, false, false, false)
                .find(&JsString::from(text), 0, false)
                .unwrap();
            assert_eq!(found.range, range, "{source}");
            assert_eq!(&*found.captures, &*captures, "{source}");
        }
        for source in [
            r"(?<=(\1))a",
            r"(a)(?<=(?<=\1))b",
            r"(a)(?<=(?=\1)\1)b",
            r"(a)(?<=\1|a)b",
            r"(a)(?<=\1{1,2})b",
            r"(a)(?<=\1+)b",
        ] {
            assert!(
                RegExpBackreferenceMatcher::compile_ordinary_with_named_bindings_and_work(
                    &JsString::from(source),
                    false,
                    false,
                    false,
                    RegExpBackreferenceNamedBindings::default(),
                    |_| Ok::<_, ()>(())
                )
                .unwrap()
                .is_none(),
                "{source}"
            );
        }
        let input = JsString::from_code_units(vec![0xd800, 0xdc00]);
        let found = ordinary(r"([\uD800])(?<=\1{1})[\uDC00]", false, false, false)
            .find(&input, 0, true)
            .unwrap();
        assert_eq!(found.range, 0..2);
        assert_eq!(&*found.captures, &[Some(0..1)]);
    }

    #[test]
    fn counted_outside_reference_lookbehind_deep_owned_ranges_clones_completed_negative_rollback_and_work()
     {
        let source =
            "(a)(?<=".to_owned() + &"(".repeat(100000) + r"\1\1" + &")".repeat(100000) + "{2})b";
        let matcher = ordinary(&source, false, false, false);
        let copy = matcher.clone();
        drop(matcher);
        let found = copy.find(&JsString::from("aaaaab"), 4, true).unwrap();
        assert_eq!(found.range, 4..6);
        assert_eq!(found.captures.len(), 100001);
        assert_eq!(found.captures[0], Some(4..5));
        assert!(found.captures[1..].iter().all(|r| *r == Some(1..3)));
        drop(copy);
        let source =
            "(a)(?<!".to_owned() + &"(".repeat(100000) + r"\1\1" + &")".repeat(100000) + "{2}q)b";
        let found = ordinary(&source, false, false, false)
            .find(&JsString::from("aaaaab"), 4, true)
            .unwrap();
        assert_eq!(found.range, 4..6);
        assert_eq!(found.captures[0], Some(4..5));
        assert!(found.captures[1..].iter().all(Option::is_none));
        let matcher = ordinary(r"(a+)(?<=\1{1})b", false, false, false);
        let input = JsString::from(("a".repeat(50000) + "b").as_str());
        let found = matcher.find(&input, 0, true).unwrap();
        assert_eq!(found.range, 0..50001);
        assert_eq!(&*found.captures, &[Some(0..50000)]);
        // Finite input cannot supply an overflowing nonempty prefix. Required
        // empty iterations retain one empty effect without count expansion.
        let count = usize::MAX;
        let source = format!(r"(ab)(?<=\1{{{count}}})c");
        assert!(
            ordinary(&source, false, false, false)
                .find(&JsString::from("abc"), 0, true)
                .is_none()
        );
        let source = format!(r"(ab)(?<!\1{{{count}}})c");
        let found = ordinary(&source, false, false, false)
            .find(&JsString::from("abc"), 0, true)
            .unwrap();
        assert_eq!(found.range, 0..3);
        assert_eq!(&*found.captures, &[Some(0..2)]);
        let source = format!(r"()(?<=(\1){{{count}}})b");
        let found = ordinary(&source, false, false, false)
            .find(&JsString::from("qb"), 1, true)
            .unwrap();
        assert_eq!(found.range, 1..2);
        assert_eq!(&*found.captures, &[Some(1..1), Some(1..1)]);
        let mut work = 0;
        assert!(
            matcher
                .find_with_work(&input, 0, true, |n| {
                    work += n;
                    if work > 1000 { Err(()) } else { Ok(()) }
                })
                .is_err()
        );
    }

    #[test]
    fn outside_reference_lookbehind_execution_snapshot() {
        let mut rows = String::new();
        for source in [
            r"(a)(?<=\1)b",
            r"(.)(?<=(\1\1))",
            r"(..)(?<=\1\1\1)",
            r"(a)(?<=a\1)b",
            r"(a)(?<=\1a)b",
            r"(a)(?<!\1)b",
            r"(a)(?<!b\1)b",
            r"(?<=\1)(a)",
            r"(?<=\1)a(a)",
            r"(a(?<=\1))b",
            r"()(?<=(\1))b",
            r"()(?<=\1)b",
            r"(?=(a))(?<=\1)a",
            r"(?=(a))(?<=(\1))a",
            r"(?=(a))(?<!\1)a",
            r"(a)(?<=\1\b)b",
            r"(a)(?<=\1\B)b",
            r"(a)(?<=^\1)b",
            r"(a)(?<=(?:\1))b",
            r"(a)(?<=(\1))b",
            r"(a)(?<=(\1\1))b",
            r"(a)(?<=\1(?=b))b",
            r"(a)(?<=\1(?=(b)))b",
            r"(a)(?<=(?<=a)\1)b",
            r"(a)(?<=\1(?<=a))b",
            r"(a)(?<=(a|b){0}\1)b",
            r"(a)(?<=(?:(?=q)\1){0}\1)b",
            r"(?:(a|b)(?<=\1))+c",
            r"(µ)(?<=\1)Μ",
            r"([\uD800])(?<=\1)[\uDC00]",
            r"(a)(?<!(\1)q)b",
            r"(?<=(?:\1){0})(a)",
        ] {
            for (ignore_case, multiline, dot_all) in [
                (false, false, false),
                (true, true, false),
                (false, false, true),
                (true, false, true),
            ] {
                let matcher = ordinary(source, ignore_case, multiline, dot_all);
                for text in [
                    "", "a", "b", "ab", "aaab", "abb", "abc", "ababab", "aaaabb", " abc", "a b",
                    "a\nb", "a\r\n", "µµΜ", "abbbc",
                ] {
                    let input = JsString::from(text);
                    for (start, sticky) in [(0, false), (0, true), (1, true)] {
                        writeln!(rows,"{source:?} i={ignore_case} m={multiline} s={dot_all} input={input:?} start={start} sticky={sticky} {:?}",matcher.find(&input,start,sticky)).unwrap();
                    }
                }
            }
        }
        insta::assert_snapshot!(rows);
    }

    #[test]
    fn outside_reference_lookbehind_preserves_owned_ranges_open_forward_and_negative_slots() {
        for (source, text, range, captures) in [
            (r"(a)(?<=\1)b", "qab", 1..3, vec![Some(1..2)]),
            (r"(.)(?<=(\1\1))", "abb", 2..3, vec![Some(2..3), Some(1..3)]),
            (
                r"(a)(?<=(\1\1))b",
                "aaab",
                2..4,
                vec![Some(2..3), Some(1..3)],
            ),
            (r"(?<=\1)(a)", "qa", 1..2, vec![Some(1..2)]),
            (r"(a(?<=\1))b", "qab", 1..3, vec![Some(1..2)]),
            (r"()(?<=(\1))b", "qb", 1..2, vec![Some(1..1), Some(1..1)]),
            (r"(a)(?<!(\1)q)b", "aaab", 2..4, vec![Some(2..3), None]),
            (
                r"(a)(?<=\1(?=(b)))b",
                "qab",
                1..3,
                vec![Some(1..2), Some(2..3)],
            ),
        ] {
            let found = ordinary(source, false, false, false)
                .find(&JsString::from(text), 0, false)
                .unwrap();
            assert_eq!(found.range, range, "{source}");
            assert_eq!(&*found.captures, &*captures, "{source}");
        }
        for source in [
            r"(?<=(\1))a",
            r"(a)(?<=(?<=\1))b",
            r"(a)(?<=(?=\1)\1)b",
            r"(a)(?<=\1|a)b",
            r"(a)(?<=\1{1,2})b",
            r"(a)(?<=\1+)b",
        ] {
            assert!(
                RegExpBackreferenceMatcher::compile_ordinary_with_named_bindings_and_work(
                    &JsString::from(source),
                    false,
                    false,
                    false,
                    RegExpBackreferenceNamedBindings::default(),
                    |_| Ok::<_, ()>(())
                )
                .unwrap()
                .is_none(),
                "{source}"
            );
        }
        let input = JsString::from_code_units(vec![0xd800, 0xdc00]);
        let found = ordinary(r"([\uD800])(?<=\1)[\uDC00]", false, false, false)
            .find(&input, 0, true)
            .unwrap();
        assert_eq!(found.range, 0..2);
        assert_eq!(&*found.captures, &[Some(0..1)]);
    }

    #[test]
    fn outside_reference_lookbehind_deep_owned_ranges_clones_completed_negative_rollback_and_work()
    {
        let source =
            "(a)(?<=".to_owned() + &"(".repeat(100000) + r"\1\1" + &")".repeat(100000) + ")b";
        let matcher = ordinary(&source, false, false, false);
        let copy = matcher.clone();
        drop(matcher);
        let found = copy.find(&JsString::from("aaab"), 2, true).unwrap();
        assert_eq!(found.range, 2..4);
        assert_eq!(found.captures.len(), 100001);
        assert_eq!(found.captures[0], Some(2..3));
        assert!(found.captures[1..].iter().all(|r| *r == Some(1..3)));
        drop(copy);
        let source =
            "(a)(?<!".to_owned() + &"(".repeat(100000) + r"\1\1" + &")".repeat(100000) + "q)b";
        let found = ordinary(&source, false, false, false)
            .find(&JsString::from("aaab"), 2, true)
            .unwrap();
        assert_eq!(found.range, 2..4);
        assert_eq!(found.captures[0], Some(2..3));
        assert!(found.captures[1..].iter().all(Option::is_none));
        let matcher = ordinary(r"(a+)(?<=\1)b", false, false, false);
        let input = JsString::from(("a".repeat(50000) + "b").as_str());
        let found = matcher.find(&input, 0, true).unwrap();
        assert_eq!(found.range, 0..50001);
        assert_eq!(&*found.captures, &[Some(0..50000)]);
        let mut work = 0;
        assert!(
            matcher
                .find_with_work(&input, 0, true, |n| {
                    work += n;
                    if work > 1000 { Err(()) } else { Ok(()) }
                })
                .is_err()
        );
    }

    #[test]
    fn fixed_lookahead_capture_execution_snapshot() {
        let mut rows = String::new();
        for source in [
            r"(?<=a(?=(b)))b",
            r"(?<=a(?=(b){2}))b",
            r"(?<=a(?=((b)){2}))b",
            r"(?<=a(?=(b){0}))b",
            r"(?<=a(?=((b){0}){2}))b",
            r"(?<=a(?=((b){0})*))b",
            r"(?<=a(?=(b|c)))b",
            r"(?<=a(?=(b)|(c)))b",
            r"(?<=a(?=(b.)|(bc)))b",
            r"(?<=a(?!((b){2})))b",
            r"(?<!(?=(b))q)c",
            r"(?<=(?=(a))(a))b",
            r"(?<=a(?=(?=(b))b))b",
            r"(?<=a(?=(?!(c))b))b",
            r"(?<=a(?=(?<=(a))b))b",
            r"(?<=a(?=(?<!((c)))b))b",
            r"(?<=a(?=(b{2})))b",
            r"(?<=a(?=((b)b){2}))b",
            r"(?<=a(?=(b()){2}))b",
            r"(?<=a(?=(()b){2}))b",
            r"(?<=a(?=(\b){2}b))b",
            r"(?<=a(?=((?=b)){2}b))b",
            r"(?<=((?=(b))){2})b",
            r"(?<=((?=(b)))*)b",
            r"(?<=a(?=(b)))b\1",
            r"(?<=a(?=(b){2}))b\1",
            r"(?<=a(?=(b))|a(?=(c)))b",
            r"(?<!a(?=(b)))b",
            r"(?<=µ(?=(Μ){2}))Μ",
            r"(?<=[\uD800](?=([\uDC00]){2}))[\uDC00]",
            r"(?:(?<=a(?=(b)))b|a)+c",
            r"(?=(?<=a(?=(b)))b)b",
        ] {
            for (ignore_case, multiline, dot_all) in [
                (false, false, false),
                (true, true, false),
                (false, false, true),
                (true, false, true),
            ] {
                let matcher = ordinary(source, ignore_case, multiline, dot_all);
                for text in [
                    "", "a", "b", "ab", "abb", "abbbb", "abc", "ad", "ac", " abc", "a b", "a\nb",
                    "a\r\n", "µµΜΜ", "abbbc",
                ] {
                    let input = JsString::from(text);
                    for (start, sticky) in [(0, false), (0, true), (1, true)] {
                        writeln!(rows,"{source:?} i={ignore_case} m={multiline} s={dot_all} input={input:?} start={start} sticky={sticky} {:?}",matcher.find(&input,start,sticky)).unwrap();
                    }
                }
            }
        }
        insta::assert_snapshot!(rows);
    }

    #[test]
    fn fixed_lookahead_captures_export_forward_rightmost_and_nested_backward_leftmost_ranges() {
        for (source, text, range, captures) in [
            (r"(?<=a(?=(b){2}))b", "qabb", 2..3, vec![Some(3..4)]),
            (
                r"(?<=a(?=((b)b){2}))b",
                "qabbbb",
                2..3,
                vec![Some(4..6), Some(4..5)],
            ),
            (r"(?<=a(?=(?<=(a){2})b))b", "qaab", 3..4, vec![Some(1..2)]),
            (
                r"(?<=((?=(b))){2})b",
                "qb",
                1..2,
                vec![Some(1..1), Some(1..2)],
            ),
            (r"(?<=((?=(b)))*)b", "qb", 1..2, vec![None, None]),
            (r"(?<!(?=(b))q)c", "qbc", 2..3, vec![None]),
            (r"(?<=(b){2})c", "qbbc", 3..4, vec![Some(1..2)]),
        ] {
            let found = ordinary(source, false, false, false)
                .find(&JsString::from(text), 0, false)
                .unwrap();
            assert_eq!(found.range, range, "{source}");
            assert_eq!(&*found.captures, &*captures, "{source}");
        }
        for source in [
            r"(?<=a(?=(b+)))b",
            r"(?<=a(?=(b)\1))b",
            r"(?<=a(?=(b|cc)))b",
            r"(?<=a(?=(b){1,2}))b",
        ] {
            assert!(
                RegExpBackreferenceMatcher::compile_ordinary_with_named_bindings_and_work(
                    &JsString::from(source),
                    false,
                    false,
                    false,
                    RegExpBackreferenceNamedBindings::default(),
                    |_| Ok::<_, ()>(())
                )
                .unwrap()
                .is_none(),
                "{source}"
            );
        }
    }

    #[test]
    fn fixed_lookahead_captures_complete_deep_slots_and_restore_negative_failures_with_flat_clones()
    {
        let source = "(?<=(?=".to_owned() + &"(".repeat(100000) + "b" + &")".repeat(100000) + "))b";
        let matcher = ordinary(&source, false, false, false);
        let copy = matcher.clone();
        drop(matcher);
        let found = copy.find(&JsString::from("qb"), 1, true).unwrap();
        assert_eq!(found.range, 1..2);
        assert_eq!(found.captures.len(), 100000);
        assert!(found.captures.iter().all(|r| *r == Some(1..2)));
        drop(copy);
        let source =
            "(?<!(?=".to_owned() + &"(".repeat(100000) + "b" + &")".repeat(100000) + ")q)c";
        let found = ordinary(&source, false, false, false)
            .find(&JsString::from("bc"), 1, true)
            .unwrap();
        assert_eq!(found.range, 1..2);
        assert_eq!(found.captures.len(), 100000);
        assert!(found.captures.iter().all(Option::is_none));
        let source = "(?<=".to_owned() + &"(?=(".repeat(10000) + "b" + &"))".repeat(10000) + ")b";
        let found = ordinary(&source, false, false, false)
            .find(&JsString::from("qb"), 1, true)
            .unwrap();
        assert_eq!(found.captures.len(), 10000);
        assert!(found.captures[..9999].iter().all(|r| *r == Some(1..1)));
        assert_eq!(found.captures[9999], Some(1..2));
        let matcher = ordinary(r"(?<=a(?=(b){2}))b", false, false, false);
        let mut work = 0;
        assert!(
            matcher
                .find_with_work(&JsString::from("a".repeat(10000).as_str()), 0, false, |n| {
                    work += n;
                    if work > 1000 { Err(()) } else { Ok(()) }
                })
                .is_err()
        );
    }

    #[test]
    fn fixed_lookahead_in_lookbehind_execution_snapshot() {
        let mut rows = String::new();
        for source in [
            r"(?<=a(?=b))b",
            r"(?<=a(?!c))b",
            r"(?<=(?=ab)a)b",
            r"(?<=(?!bb)a)b",
            r"(?<=a(?=b{2}))b",
            r"(?<=a(?=[bc]|d))b",
            r"(?<=a(?=b|c))b",
            r"(?<=a(?!b|c))d",
            r"(?<=a(?=(?=b)b))b",
            r"(?<=a(?=(?!c)b))b",
            r"(?<=a(?=(?<=a)b))b",
            r"(?<=a(?=(?<!c)b))b",
            r"(?<=a(?=(?:b){2}))bb",
            r"(?<=a(?=b{0}))b",
            r"(?<=a(?=(?:b|c){0}))b",
            r"(?<=a(?=$))",
            r"(?<=a(?=^))b",
            r"(?<=a(?=\b)) ",
            r"(?<=a(?=\B))b",
            r"(?<=a(?=.))b",
            r"(?<=a(?=b.))b",
            r"(?<=a(?!b.))b",
            r"(?<=(?=ab)(a))b",
            r"(?<=((?=b)){2})b",
            r"(?<=((?!a)){2})b",
            r"(?<=((?=b))*)b",
            r"(?<=a(?=(?:(?=b)){2}b))b",
            r"(?<=a(?=(?<=a(?=b))b))b",
            r"(?<=µ(?=Μ))Μ",
            r"(?<!a(?=b))b",
            r"(?<!a(?!b))b",
            r"(?<=a(?=b)|a(?!c))b",
            r"(?:(?<=a(?=b))b|a)+c",
            r"(?=(?<=a(?=b))b)b",
            r"(?<=[\uD800](?=[\uDC00]))[\uDC00]",
            r"(?<=((?=a)){1,2})a",
            r"(?<=((?=a)){2})a",
            r"(?<=((?=a)){2})c",
            r"(?<=(?:(?=a)){2})a",
            r"(?<=(?=a))a",
            r"(?<=(a(?=a)))b",
            r"(?<=a(?=a))b",
            r"(?<=a{2}(?=a))b",
            r"(?<=a|(?=a)b)c",
        ] {
            for (ignore_case, multiline, dot_all) in [
                (false, false, false),
                (true, true, false),
                (false, false, true),
                (true, false, true),
            ] {
                let matcher = ordinary(source, ignore_case, multiline, dot_all);
                for text in [
                    "", "a", "b", "ab", "aab", "abb", "abc", "ad", "ac", " abc", "a b", "a\nb",
                    "a\r\n", "µµΜ", "abbbc",
                ] {
                    let input = JsString::from(text);
                    for (start, sticky) in [(0, false), (0, true), (1, true)] {
                        writeln!(rows,"{source:?} i={ignore_case} m={multiline} s={dot_all} input={input:?} start={start} sticky={sticky} {:?}",matcher.find(&input,start,sticky)).unwrap();
                    }
                }
            }
        }
        insta::assert_snapshot!(rows);
    }

    #[test]
    fn fixed_lookahead_in_lookbehind_keeps_parent_ranges_full_input_and_negative_choices() {
        for (source, text, range, captures) in [
            (r"(?<=(?=ab)(a))b", "qab", 2..3, vec![Some(1..2)]),
            (r"(?<=((?=b)){2})b", "qb", 1..2, vec![Some(1..1)]),
            (r"(?<=((?=b))*)b", "qb", 1..2, vec![None]),
            (r"(?<!a(?=b))b", "qb", 1..2, vec![]),
            (r"(?<=a(?=b)|a(?!c))b", "qab", 2..3, vec![]),
        ] {
            let found = ordinary(source, false, false, false)
                .find(&JsString::from(text), 0, false)
                .unwrap();
            assert_eq!(found.range, range, "{source}");
            assert_eq!(&*found.captures, &*captures, "{source}");
        }
        assert!(
            ordinary(r"(?<=a(?=b.))b", false, false, false)
                .find(&JsString::from("ab"), 0, false)
                .is_none()
        );
        assert_eq!(
            ordinary(r"(?<=a(?!b.))b", false, false, false)
                .find(&JsString::from("ab"), 0, false)
                .unwrap()
                .range,
            1..2
        );
        let input = JsString::from_code_units(vec![0xd800, 0xdc00]);
        assert_eq!(
            ordinary(r"(?<=[\uD800](?=[\uDC00]))[\uDC00]", false, false, false)
                .find(&input, 0, false)
                .unwrap()
                .range,
            1..2
        );
        for source in [
            r"(?<=a(?=(b+)))b",
            r"(?<=a(?=b+))b",
            r"(?<=a(?=b|cc))b",
            r"(?<=a(?!b+))b",
            r"(?<=a+)b",
        ] {
            assert!(
                RegExpBackreferenceMatcher::compile_ordinary_with_named_bindings_and_work(
                    &JsString::from(source),
                    false,
                    false,
                    false,
                    RegExpBackreferenceNamedBindings::default(),
                    |_| Ok::<_, ()>(())
                )
                .unwrap()
                .is_none(),
                "{source}"
            );
        }
    }

    #[test]
    fn fixed_lookahead_in_lookbehind_nested_assertions_clone_and_drop_without_native_recursion() {
        let source = "(?<=".to_owned() + &"(?=".repeat(100000) + "b" + &")".repeat(100000) + ")b";
        let matcher = ordinary(&source, false, false, false);
        let copy = matcher.clone();
        drop(matcher);
        assert_eq!(
            copy.find(&JsString::from("qb"), 1, true).unwrap().range,
            1..2
        );
        let source = "(?<=".to_owned() + &"(?!".repeat(100000) + "b" + &")".repeat(100000) + ")b";
        assert_eq!(
            ordinary(&source, false, false, false)
                .find(&JsString::from("qb"), 1, true)
                .unwrap()
                .range,
            1..2
        );
        let matcher = ordinary(r"(?<=a(?=(?<=a(?=b))b))b", false, false, false);
        assert!(
            matcher
                .find_with_work(&JsString::from("a".repeat(10000).as_str()), 0, false, |n| {
                    if n > 0 { Err(()) } else { Ok(()) }
                })
                .is_err()
        );
    }

    #[test]
    fn zero_count_native_scope_execution_snapshot() {
        let mut rows = String::new();
        for source in [
            r"(?<=(|a){0})b",
            r"(?<=(a|){0}?)b",
            r"(?<=((?=a)\1){0})b",
            r"(?<=((?!(a))\2){0})b",
            r"(?<=((?=a)(\2)){0})b",
            r"(?<=((?=a)|b){0})c",
            r"(?<=((a|){0}){2})b",
            r"(?<=((a|){0})*)b",
            r"(?<=((a|){0}){2}q)b",
            r"(a)(?<=((?=a)\1){0})b\1",
            r"(?<=((?=a)|()){0})b",
            r"((?=a)\1){0}b",
            r"(?:(a|){0}){2}b",
            r"(?:(a|){0})*b",
            r"(?<=((?=a)\1){0}q)b",
            r"(?<=q((?=a)\1){0})b",
            r"(?<!((?=a)\1){0}q)b",
            r"(?<!((?=a)\1){0})b",
            r"(?<=((?=a)\1){0}|())b",
            r"(?<=([abc]|){0})b",
            r"(?<=((^|a)\1){0})b",
            r"(?<=((\b|a)\1){0})b",
            r"(?<=((a|){0}){2}µ)Μ",
            r"(?<=((?=a)\1){0}[\uD800])b",
            r"(?<=((?=a)\1){0})",
            r"(?:(?<=((a|){0}){2})b|a)+c",
            r"(?=(?<=((?=a)\1){0})b)b",
            r"((?<=((a|){0}){2})){2}b",
            r"((a|){0}){2}b",
            r"((a|){0})*b",
            r"(?<=(?:(a|){0}){0})b",
            r"(a)(?<=(\1|){0})b\1",
        ] {
            for (ignore_case, multiline, dot_all) in [
                (false, false, false),
                (true, true, false),
                (false, false, true),
                (true, false, true),
            ] {
                let matcher = ordinary(source, ignore_case, multiline, dot_all);
                for text in [
                    "", "a", "b", "ab", "aab", "ba", " abc", "aaarc", "aabcd", "\nab", "aa\nb",
                    "a\r\n", "µµΜ", " a", "qa",
                ] {
                    let input = JsString::from(text);
                    for (start, sticky) in [(0, false), (0, true), (1, true)] {
                        writeln!(rows,"{source:?} i={ignore_case} m={multiline} s={dot_all} input={input:?} start={start} sticky={sticky} {:?}",matcher.find(&input,start,sticky)).unwrap();
                    }
                }
            }
        }
        insta::assert_snapshot!(rows);
    }

    #[test]
    fn zero_count_native_scopes_clear_only_owned_slots_and_keep_enclosing_and_prefix_ranges() {
        for (source, text, captures) in [
            (r"(?<=(|a){0})b", "qb", vec![None]),
            (r"(?<=((?=a)\1){0})b", "qb", vec![None]),
            (r"(?<=((?!(a))\2){0})b", "qb", vec![None, None]),
            (r"(?<=((a|){0}){2})b", "qb", vec![Some(1..1), None]),
            (r"(?<=((a|){0})*)b", "qb", vec![None, None]),
            (r"(?<=((a|){0}){2}q)b", "qb", vec![Some(0..0), None]),
            (r"(?<!((?=a)\1){0}q)b", "xb", vec![None]),
            (r"((a|){0}){2}b", "qb", vec![Some(1..1), None]),
        ] {
            let found = ordinary(source, false, false, false)
                .find(&JsString::from(text), 1, true)
                .unwrap();
            assert_eq!(found.range, 1..2, "{source}");
            assert_eq!(&*found.captures, &*captures, "{source}");
        }
        let found = ordinary(r"(a)(?<=((?=a)\1){0})b\1", false, false, false)
            .find(&JsString::from("aba"), 0, true)
            .unwrap();
        assert_eq!(found.range, 0..3);
        assert_eq!(&*found.captures, &[Some(0..1), None]);
        for source in [
            r"(?<=(|a){1})b",
            r"(?<=((?=a)\1){1})b",
            r"(?<=((a|a*){1}){0})b",
            r"(?<=a+)b",
        ] {
            assert!(
                RegExpBackreferenceMatcher::compile_ordinary_with_named_bindings_and_work(
                    &JsString::from(source),
                    false,
                    false,
                    false,
                    RegExpBackreferenceNamedBindings::default(),
                    |_| Ok::<_, ()>(())
                )
                .unwrap()
                .is_none(),
                "{source}"
            );
        }
    }

    #[test]
    fn nested_zero_scopes_discard_unreachable_regions_once_and_keep_deep_captures_clones_flat() {
        let source =
            "(?<=".to_owned() + &"(".repeat(100000) + "(?=a)|b" + &"){0}".repeat(100000) + ")c";
        let mut work = 0;
        let matcher = RegExpBackreferenceMatcher::compile_ordinary_with_named_bindings_and_work(
            &JsString::from(source.as_str()),
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
        assert!(work < source.len() * 100);
        let copy = matcher.clone();
        drop(matcher);
        let found = copy.find(&JsString::from("qc"), 1, true).unwrap();
        assert_eq!(found.captures.len(), 100000);
        assert!(found.captures.iter().all(Option::is_none));
        let mut work = 0;
        assert_eq!(
            copy.find_with_work(&JsString::from("qc"), 1, true, |n| {
                work += n;
                if work > 1000 {
                    Err("explicit zero-scope work")
                } else {
                    Ok(())
                }
            })
            .unwrap_err(),
            "explicit zero-scope work"
        );
        let source = "(?:".repeat(10000) + "(?=a)|b" + &"){0}".repeat(10000) + "c";
        let matcher = ordinary(&source, false, false, false);
        let mut work = 0;
        assert_eq!(
            matcher
                .find_with_work(&JsString::from("qc"), 1, true, |n| {
                    work += n;
                    Ok::<_, ()>(())
                })
                .unwrap()
                .unwrap()
                .range,
            1..2
        );
        assert!(work < 100);
        let source = "(?<=".to_owned()
            + &"(?:".repeat(10000)
            + "((?=a)\\1){0}"
            + &"){2}".repeat(10000)
            + ")b";
        assert_eq!(
            &*ordinary(&source, false, false, false)
                .find(&JsString::from("b"), 0, true)
                .unwrap()
                .captures,
            &[None]
        );
    }

    #[test]
    fn zero_count_lookbehind_choice_execution_snapshot() {
        let mut rows = String::new();
        for source in [
            r"(?<=(a|aa){0})b",
            r"(?<=(a|aa){0}?)b",
            r"(?<=((a)|(bb)){0})c",
            r"(?<=(a|bb|ccc){0})b",
            r"(?<=(a|aa){0}q)b",
            r"(?<=q(a|aa){0})b",
            r"(?<!((a|aa){0}){2}q)b",
            r"(?<!(a|aa){0})b",
            r"(?<=((a|aa){0}))b",
            r"(?<=((a|aa){0}){2})b",
            r"(?<=((a|aa){0}){0,2})b",
            r"(?<=((a|aa){0})+)b",
            r"(?<=((a|aa){0})*)b",
            r"(?<=((a|aa){0}){2}q)b",
            r"(?<=(a|aa){0}q|r)b",
            r"(?<=(a|aa){0}q(?<=q))b",
            r"(?<=((a)\2|b){0})c",
            r"(?<=(a*?b|cc){0})b",
            r"(?<=((?=a)a|b){0})c",
            r"(?<=((?<=(a))a|b){0})c",
            r"(?<=(a|aa){0})b\1",
            r"(?<=((a|aa){0}){2})b\1\2",
            r"(?<=((a|aa){0}){2}µ)Μ",
            r"(?<=(a|aa){0}[\uD800])b",
            r"(?<=(a|aa){0})",
            r"(?:(?<=((a|aa){0}){2})b|a)+c",
            r"(?=(?<=(a|aa){0})b)b",
            r"((?<=((a|aa){0}){2})){2}b",
            r"((a|aa){0}){2}b",
            r"((a|aa){0})*b",
            r"(?:(a|aa){0}){2}b",
            r"(a)(?<=(a\1|bb){0})b\1",
        ] {
            for (ignore_case, multiline, dot_all) in [
                (false, false, false),
                (true, true, false),
                (false, false, true),
                (true, false, true),
            ] {
                let matcher = ordinary(source, ignore_case, multiline, dot_all);
                for text in [
                    "", "a", "b", "ab", "aab", "ba", " abc", "aaarc", "aabcd", "\nab", "aa\nb",
                    "a\r\n", "µµΜ", " a", "qa",
                ] {
                    let input = JsString::from(text);
                    for (start, sticky) in [(0, false), (0, true), (1, true)] {
                        writeln!(rows,"{source:?} i={ignore_case} m={multiline} s={dot_all} input={input:?} start={start} sticky={sticky} {:?}",matcher.find(&input,start,sticky)).unwrap();
                    }
                }
            }
        }
        insta::assert_snapshot!(rows);
    }

    #[test]
    fn zero_count_choices_keep_skipped_children_enclosing_empty_ranges_and_backward_prefixes() {
        for (source, text, captures) in [
            (r"(?<=(a|aa){0})b", "qb", vec![None]),
            (r"(?<=((a|aa){0}))b", "qb", vec![Some(1..1), None]),
            (r"(?<=((a|aa){0}){2})b", "qb", vec![Some(1..1), None]),
            (r"(?<=((a|aa){0})*)b", "qb", vec![None, None]),
            (r"(?<=((a|aa){0}){2}q)b", "qb", vec![Some(0..0), None]),
            (r"(?<!((a|aa){0}){2}q)b", "xb", vec![None, None]),
            (r"((a|aa){0}){2}b", "qb", vec![Some(1..1), None]),
            (r"((a|aa){0})*b", "qb", vec![None, None]),
        ] {
            let found = ordinary(source, false, false, false)
                .find(&JsString::from(text), 1, true)
                .unwrap();
            assert_eq!(found.range, 1..2, "{source}");
            assert_eq!(&*found.captures, &*captures, "{source}");
        }
        let found = ordinary(r"(a)(?<=(a\1|bb){0})b\1", false, false, false)
            .find(&JsString::from("aba"), 0, true)
            .unwrap();
        assert_eq!(found.range, 0..3);
        assert_eq!(&*found.captures, &[Some(0..1), None]);
        for source in [
            r"(?<=(a|aa){1})b",
            r"(?<=(a|aa){0,1})b",
            r"(?<=(|a){1})b",
            r"(?<=((?=a)\1){1})b",
        ] {
            assert!(
                RegExpBackreferenceMatcher::compile_ordinary_with_named_bindings_and_work(
                    &JsString::from(source),
                    false,
                    false,
                    false,
                    RegExpBackreferenceNamedBindings::default(),
                    |_| Ok::<_, ()>(())
                )
                .unwrap()
                .is_none(),
                "{source}"
            );
        }
    }

    #[test]
    fn zero_count_choices_deep_skipped_and_completed_captures_zero_bounds_clones_and_undo_are_flat()
    {
        let skipped = "(?:".to_owned() + &"(".repeat(100000) + "a" + &")".repeat(100000) + "|bb)";
        let matcher = ordinary(&format!("(?<={skipped}{{0}})c"), false, false, false);
        let copy = matcher.clone();
        drop(matcher);
        let found = copy.find(&JsString::from("qc"), 1, true).unwrap();
        assert_eq!(found.captures.len(), 100000);
        assert!(found.captures.iter().all(Option::is_none));
        let completed =
            "(?:".to_owned() + &"(".repeat(100000) + "(?:a|bb){0}" + &")".repeat(100000) + ")";
        let found = ordinary(&format!("(?<={completed}{{2}})b"), false, false, false)
            .find(&JsString::from("qb"), 1, true)
            .unwrap();
        assert_eq!(found.captures.len(), 100000);
        assert!(found.captures.iter().all(|r| r == &Some(1..1)));
        let found = ordinary(&format!("(?<!{completed}{{2}}q)b"), false, false, false)
            .find(&JsString::from("xb"), 1, true)
            .unwrap();
        assert_eq!(found.captures.len(), 100000);
        assert!(found.captures.iter().all(Option::is_none));
        let mut work = 0;
        assert_eq!(
            copy.find_with_work(&JsString::from("qc"), 1, true, |n| {
                work += n;
                if work > 1000 {
                    Err("explicit skipped-choice work")
                } else {
                    Ok(())
                }
            })
            .unwrap_err(),
            "explicit skipped-choice work"
        );
        let matcher = ordinary(
            &format!("(?<=(a|aa){{{}}})b", "0".repeat(10000)),
            false,
            false,
            false,
        );
        let mut work = 0;
        assert_eq!(
            &*matcher
                .find_with_work(&JsString::from("qb"), 1, true, |n| {
                    work += n;
                    Ok::<_, ()>(())
                })
                .unwrap()
                .unwrap()
                .captures,
            &[None]
        );
        assert!(work < 100);
        let source =
            "(?<=".to_owned() + &"(?:".repeat(10000) + "(a|aa){0}" + &"){2}".repeat(10000) + ")b";
        assert_eq!(
            &*ordinary(&source, false, false, false)
                .find(&JsString::from("b"), 0, true)
                .unwrap()
                .captures,
            &[None]
        );
    }

    #[test]
    fn zero_count_lookbehind_reference_execution_snapshot() {
        let mut rows = String::new();
        for source in [
            r"(?<=((a)\2){0})b",
            r"(?<=((\2)a){0})b",
            r"(?<=((ab)\2){0})b",
            r"(?<=((a)(\2)\3){0})b",
            r"(?<=((a)\2){0}q)b",
            r"(?<=q((a)\2){0})b",
            r"(?<!((a)\2){0})b",
            r"(?<!((a)\2){0}q)b",
            r"(a)(?<=(?:\1){0})b",
            r"(a)(?<=(\1){0})b\1",
            r"(?<=(\2){0})b()",
            r"(?<=\1{0})b()",
            r"(?<=((a)\2){0}µ)Μ",
            r"(?<=((.)\2){0})b",
            r"(?<=(([a-z])\2){0})b",
            r"(?<=((^a)\2){0})b",
            r"(?<=((\ba)\2){0})b",
            r"(?<=((()\3)a){0})b",
            r"(?<=((a)\2){0}?|())b",
            r"(?<=((a)\2){0}|())b\1\2\3",
            r"(?<=((a)\2){0})",
            r"(?<=((a)\2){0}[\uD800])b",
            r"(?<=(\2\2){0})b()",
            r"(a)(?<=((\1)){0})b\1",
            r"(?<=((a)\2){0})b\1\2",
            r"(?:(?<=((a)\2){0})b|a)+c",
            r"(?<=((()\3){0}){2})b",
            r"(?<=((a)\2){0}q|r)b",
            r"(?<=((a)\2){0}q(?<=q))b",
            r"(?=(?<=((a)\2){0})b)b",
            r"((?<=((a)\3){0})){2}b",
            r"(?:(?<=((a)\2){0})b|a)+c\1\2",
        ] {
            for (ignore_case, multiline, dot_all) in [
                (false, false, false),
                (true, true, false),
                (false, false, true),
                (true, false, true),
            ] {
                let matcher = ordinary(source, ignore_case, multiline, dot_all);
                for text in [
                    "", "a", "b", "ab", "aab", "ba", " abc", "aaarc", "aabcd", "\nab", "aa\nb",
                    "a\r\n", "µµΜ", " a", "qa",
                ] {
                    let input = JsString::from(text);
                    for (start, sticky) in [(0, false), (0, true), (1, true)] {
                        writeln!(rows,"{source:?} i={ignore_case} m={multiline} s={dot_all} input={input:?} start={start} sticky={sticky} {:?}",matcher.find(&input,start,sticky)).unwrap();
                    }
                }
            }
        }
        insta::assert_snapshot!(rows);
    }

    #[test]
    fn zero_count_lookbehind_references_skip_consuming_units_without_clearing_outside_captures() {
        for (source, text, captures) in [
            (r"(?<=((a)\2){0})b", "qb", vec![None, None]),
            (r"(?<=((a)(\2)\3){0})b", "qb", vec![None, None, None]),
            (r"(?<=((a)\2){0}q)b", "qb", vec![None, None]),
            (r"(?<=q((a)\2){0})b", "qb", vec![None, None]),
            (r"(?<!((a)\2){0}q)b", "xb", vec![None, None]),
            (r"(?<=((()\3){0}){2})b", "qb", vec![Some(1..1), None, None]),
        ] {
            let found = ordinary(source, false, false, false)
                .find(&JsString::from(text), 1, true)
                .unwrap();
            assert_eq!(found.range, 1..2, "{source}");
            assert_eq!(&*found.captures, &*captures, "{source}");
        }
        let found = ordinary(r"(a)(?<=((\1)){0})b\1", false, false, false)
            .find(&JsString::from("aba"), 0, true)
            .unwrap();
        assert_eq!(found.range, 0..3);
        assert_eq!(&*found.captures, &[Some(0..1), None, None]);
        assert!(
            ordinary(r"(?<!((a)\2){0})b", false, false, false)
                .find(&JsString::from("b"), 0, true)
                .is_none()
        );
        for source in [
            r"(?<=((a)\2){0,1})b",
            r"(?<=((\2)a){1,2})b",
            r"(?<=(\2){0,2})b()",
            r"(?<=(|a){1})b",
            r"(?<=((?=a)\1){1})b",
        ] {
            assert!(
                RegExpBackreferenceMatcher::compile_ordinary_with_named_bindings_and_work(
                    &JsString::from(source),
                    false,
                    false,
                    false,
                    RegExpBackreferenceNamedBindings::default(),
                    |_| Ok::<_, ()>(())
                )
                .unwrap()
                .is_none(),
                "{source}"
            );
        }
    }

    #[test]
    fn zero_count_lookbehind_references_deep_skipped_slots_long_zero_bounds_and_clones_are_flat() {
        let body = "(?:".to_owned() + &"(".repeat(100000) + "a" + &")".repeat(100000) + r"\100000)";
        let source = format!("(?<={body}{{0}})b");
        let matcher = ordinary(&source, false, false, false);
        let copy = matcher.clone();
        drop(matcher);
        let found = copy.find(&JsString::from("qb"), 1, true).unwrap();
        assert_eq!(found.captures.len(), 100000);
        assert!(found.captures.iter().all(Option::is_none));
        let mut work = 0;
        assert_eq!(
            copy.find_with_work(&JsString::from("qb"), 1, true, |n| {
                work += n;
                if work > 1000 {
                    Err("explicit skipped-reference work")
                } else {
                    Ok(())
                }
            })
            .unwrap_err(),
            "explicit skipped-reference work"
        );
        let source = format!(r"(?<=((a)\2){{{}}})b", "0".repeat(10000));
        let matcher = ordinary(&source, false, false, false);
        let mut work = 0;
        assert_eq!(
            &*matcher
                .find_with_work(&JsString::from("qb"), 1, true, |n| {
                    work += n;
                    Ok::<_, ()>(())
                })
                .unwrap()
                .unwrap()
                .captures,
            &[None, None]
        );
        assert!(work < 100);
        let source =
            "(?<=".to_owned() + &"(?:".repeat(10000) + r"((a)\2){0}" + &"){2}".repeat(10000) + ")b";
        assert_eq!(
            &*ordinary(&source, false, false, false)
                .find(&JsString::from("b"), 0, true)
                .unwrap()
                .captures,
            &[None, None]
        );
    }

    #[test]
    fn zero_reference_lookbehind_count_execution_snapshot() {
        let mut rows = String::new();
        for source in [
            r"(?<=(\1)+)a()",
            r"(?<=(\1){2})a()",
            r"(?<=(()\2){2})a",
            r"(?<=(()\2){0,2})a",
            r"(?<=((\2)){2})a",
            r"(?<=((\3)()){2})a",
            r"(?<=(()(\2)\3){2})a",
            r"(?<=(()(\2)(\3)\4){2})a",
            r"(?<=((()\3){2}){2})a",
            r"(?<=((()\3){0,2}){2})a",
            r"(?<=((()\3){2})*)a",
            r"(?<=((\2){2}){2})a",
            r"(?<=((\3()){2}){2})a",
            r"(?<=((()\3\3){2}){2})a",
            r"(?<=((()(\3)){2}){2})a",
            r"(?<=((\2){0,2}){2})a",
            r"(?<=((()\3)+)+)a",
            r"(?<=((()\3)*)+)a",
            r"(?<=((()\3)+)*)a",
            r"(?<=((()\3){1,3}?){2})a",
            r"(?<=((^()\3){2}){2})a",
            r"(?<=((\b()\3){2}){2})a",
            r"(?<=((\B()\3){2}){2})a",
            r"(?<=((()^\3){0,2}){2})a",
            r"(?<=(?:(?:(()\2){2}){2}|()){2})a",
            r"(?<=(?:(?:(()\2){0,2}){2}|()){2})a",
            r"(?<!(?:(()\2){2}){2}q)a",
            r"(?<=((()\3){2}){2}a)b",
            r"(?<=a((()\3){2}){2})b",
            r"(?:(?<=((()\3){2}){2})a|b)+c",
            r"(?<=((()\3){2}){2})a\1\2\3",
            r"(?<=((()\3){0,2}){2})a\1\2\3",
            r"(?<=((()\3){2}){2}µ)Μ",
            r"(?<=((()\3){2}){2}[\uD800])b",
        ] {
            for (ignore_case, multiline, dot_all) in [
                (false, false, false),
                (true, true, false),
                (false, false, true),
                (true, false, true),
            ] {
                let matcher = ordinary(source, ignore_case, multiline, dot_all);
                for text in [
                    "", "a", "b", "ab", "aab", "ba", " abc", "aaarc", "aabcd", "\nab", "aa\nb",
                    "a\r\n", "µµΜ", " a", "qa",
                ] {
                    let input = JsString::from(text);
                    for (start, sticky) in [(0, false), (0, true), (1, true)] {
                        writeln!(rows,"{source:?} i={ignore_case} m={multiline} s={dot_all} input={input:?} start={start} sticky={sticky} {:?}",matcher.find(&input,start,sticky)).unwrap();
                    }
                }
            }
        }
        insta::assert_snapshot!(rows);
    }

    #[test]
    fn zero_reference_lookbehind_counts_preserve_backward_empty_slots_and_enclosing_ranges() {
        for (source, text, captures) in [
            (r"(?<=(()\2){2})a", "qa", vec![Some(1..1), Some(1..1)]),
            (r"(?<=(()\2){0,2})a", "qa", vec![None, None]),
            (
                r"(?<=((\3)()){2})a",
                "qa",
                vec![Some(1..1), Some(1..1), Some(1..1)],
            ),
            (
                r"(?<=(()(\2)\3){2})a",
                "qa",
                vec![Some(1..1), Some(1..1), Some(1..1)],
            ),
            (
                r"(?<=((()\3){0,2}){2})a",
                "qa",
                vec![Some(1..1), None, None],
            ),
            (r"(?<!(?:(()\2){2}){2}q)a", "xa", vec![None, None]),
            (
                r"(?<=((^()\3){2}){2}a)b",
                "ab",
                vec![Some(0..0), Some(0..0), Some(0..0)],
            ),
            (
                r"(?<=a((()\3){2}){2})b",
                "ab",
                vec![Some(1..1), Some(1..1), Some(1..1)],
            ),
        ] {
            let found = ordinary(source, false, false, false)
                .find(&JsString::from(text), 1, true)
                .unwrap();
            assert_eq!(found.range, 1..2, "{source}");
            assert_eq!(&*found.captures, &*captures, "{source}");
        }
        for source in [
            r"(?<=((a)\2){1,2})b",
            r"(?<=((\2)a){1,2})b",
            r"(?<=(((a)\3){0,2}){2})a",
            r"(?<=(\2)+)a()",
            r"(?<=((?=a)\1){2})a",
        ] {
            assert!(
                RegExpBackreferenceMatcher::compile_ordinary_with_named_bindings_and_work(
                    &JsString::from(source),
                    false,
                    false,
                    false,
                    RegExpBackreferenceNamedBindings::default(),
                    |_| Ok::<_, ()>(())
                )
                .unwrap()
                .is_none(),
                "{source}"
            );
        }
    }

    #[test]
    fn zero_reference_lookbehind_counts_large_bounds_deep_captures_and_completed_negative_undo_are_flat()
     {
        let huge = "184467440737095516160000000000000000000";
        let body = "(?:".to_owned() + &"(".repeat(100000) + &")".repeat(100000) + r"\100000){2}";
        let source = format!("(?<=(?:{body}){{{huge},}})a");
        let matcher = ordinary(&source, false, false, false);
        let copy = matcher.clone();
        drop(matcher);
        let found = copy.find(&JsString::from("qa"), 1, true).unwrap();
        assert_eq!(found.captures.len(), 100000);
        assert!(found.captures.iter().all(|r| r == &Some(1..1)));
        let mut work = 0;
        assert_eq!(
            copy.find_with_work(&JsString::from("qa"), 1, true, |n| {
                work += n;
                if work > 1000 {
                    Err("explicit zero-reference lookbehind work")
                } else {
                    Ok(())
                }
            })
            .unwrap_err(),
            "explicit zero-reference lookbehind work"
        );
        let found = ordinary(&format!("(?<!(?:{body}){{2}}q)a"), false, false, false)
            .find(&JsString::from("xa"), 1, true)
            .unwrap();
        assert_eq!(found.captures.len(), 100000);
        assert!(found.captures.iter().all(Option::is_none));
        let source =
            "(?<=".to_owned() + &"(?:".repeat(10000) + r"(()\2){2}" + &"){2}".repeat(10000) + ")a";
        assert_eq!(
            &*ordinary(&source, false, false, false)
                .find(&JsString::from("a"), 0, true)
                .unwrap()
                .captures,
            &[Some(0..0), Some(0..0)]
        );
        let matcher = ordinary(&format!("(?<=(()\\2){{{huge},}})a"), false, false, false);
        let mut work = 0;
        assert!(
            matcher
                .find_with_work(&JsString::from("a"), 0, true, |n| {
                    work += n;
                    Ok::<_, ()>(())
                })
                .unwrap()
                .is_some()
        );
        assert!(work < 100);
    }

    #[test]
    fn fixed_lookbehind_zero_wrapper_execution_snapshot() {
        let mut rows = String::new();
        for source in [
            r"(?<=(()|()){2})a",
            r"(?<=(|)+)a",
            r"(?<=(|){2})a",
            r"(?<=(|){0,2})a",
            r"(?<=(|)+?)a",
            r"(?<=(|)*?)a",
            r"(?<=((){2}){2})a",
            r"(?<=((){0,2}){2})a",
            r"(?<=((){2})*)a",
            r"(?<=(()+)+)a",
            r"(?<=(()*)+)a",
            r"(?<=(()+)*)a",
            r"(?<=((^|$){2}){2})a",
            r"(?<=((\b|\B){2}){2})a",
            r"(?<=((^){0,2}){2})a",
            r"(?<=((^){1,2}){2})a",
            r"(?<=((a){0}){2})b",
            r"(?<=(([a-z]){0}){2})b",
            r"(?<=((.){0}){2})b",
            r"(?<=(?:(()^)|()){2})a",
            r"(?<=(?:(()$)|()){2})a",
            r"(?<=(?:((){2}){2}|()){2})a",
            r"(?<=(?:((){0}){2}|()){2})a",
            r"(?<=(((){2})?){2})a",
            r"(?<!((|){2}){2}q)a",
            r"(?<!((^|$){2}){2})a",
            r"(?<=((|){2}){2}a)b",
            r"(?<=a((|){2}){2})b",
            r"(?<=((^|$){2}){2}a)b",
            r"(?<=a((\b|\B){2}){2})b",
            r"(?<=((?<=a)){2})b",
            r"(?<=((?<=(a))){2})b",
            r"(?<=((?<=(a))){0,2})b",
            r"(?<=((|){2}){2})a\1\2",
            r"((?<=((|){2}){2})){2}a",
            r"(?:(?<=((|){2}){2})a|b)+c",
            r"(?<=((a){0}){2}µ)Μ",
            r"(?<=((|){2}){2}[\uD800])b",
        ] {
            for (ignore_case, multiline, dot_all) in [
                (false, false, false),
                (true, true, false),
                (false, false, true),
                (true, false, true),
            ] {
                let matcher = ordinary(source, ignore_case, multiline, dot_all);
                for text in [
                    "", "a", "b", "ab", "aab", "ba", " abc", "aaarc", "aabcd", "\nab", "aa\nb",
                    "a\r\n", "µµΜ", " a", "qa",
                ] {
                    let input = JsString::from(text);
                    for (start, sticky) in [(0, false), (0, true), (1, true)] {
                        writeln!(rows,"{source:?} i={ignore_case} m={multiline} s={dot_all} input={input:?} start={start} sticky={sticky} {:?}",matcher.find(&input,start,sticky)).unwrap();
                    }
                }
            }
        }
        insta::assert_snapshot!(rows);
    }

    #[test]
    fn fixed_lookbehind_zero_wrappers_keep_required_optional_and_failed_branch_ranges() {
        for (source, text, captures) in [
            (r"(?<=(|){2})a", "qa", vec![Some(1..1)]),
            (r"(?<=(|){0,2})a", "qa", vec![None]),
            (r"(?<=((){0,2}){2})a", "qa", vec![Some(1..1), None]),
            (r"(?<=((){2})*)a", "qa", vec![None, None]),
            (r"(?<=(?:(()^)|()){2})a", "qa", vec![None, None, Some(1..1)]),
            (r"(?<!((|){2}){2}q)a", "xa", vec![None, None]),
            (r"(?<=((^|$){2}){2}a)b", "ab", vec![Some(0..0), Some(0..0)]),
            (r"(?<=((?<=(a))){2})b", "ab", vec![Some(1..1), Some(0..1)]),
            (r"(?<=((?<=(a))){0,2})b", "ab", vec![None, None]),
        ] {
            let found = ordinary(source, false, false, false)
                .find(&JsString::from(text), 1, true)
                .unwrap();
            assert_eq!(found.range, 1..2, "{source}");
            assert_eq!(&*found.captures, &*captures, "{source}");
        }
        for source in [
            r"(?<=(|a){2})a",
            r"(?<=((a{0,2}){2}){2})a",
            r"(?<=((?=a+)){2})a",
            r"(?<=(((a)\3){0,2}){2})a",
        ] {
            assert!(
                RegExpBackreferenceMatcher::compile_ordinary_with_named_bindings_and_work(
                    &JsString::from(source),
                    false,
                    false,
                    false,
                    RegExpBackreferenceNamedBindings::default(),
                    |_| Ok::<_, ()>(())
                )
                .unwrap()
                .is_none(),
                "{source}"
            );
        }
    }

    #[test]
    fn fixed_lookbehind_zero_wrapper_large_counts_deep_captures_undo_and_frames_are_flat() {
        let huge = "184467440737095516160000000000000000000";
        let body = "(?:".to_owned() + &"(".repeat(100000) + &")".repeat(100000) + "|)";
        let source = format!("(?<={body}{{{huge},}})a");
        let matcher = ordinary(&source, false, false, false);
        let copy = matcher.clone();
        drop(matcher);
        let found = copy.find(&JsString::from("qa"), 1, true).unwrap();
        assert_eq!(found.captures.len(), 100000);
        assert!(found.captures.iter().all(|r| r == &Some(1..1)));
        let mut work = 0;
        assert_eq!(
            copy.find_with_work(&JsString::from("qa"), 1, true, |n| {
                work += n;
                if work > 1000 {
                    Err("explicit fixed-wrapper work")
                } else {
                    Ok(())
                }
            })
            .unwrap_err(),
            "explicit fixed-wrapper work"
        );
        let negative = ordinary(&format!("(?<!{body}{{2}}q)a"), false, false, false);
        let found = negative.find(&JsString::from("xa"), 1, true).unwrap();
        assert_eq!(found.captures.len(), 100000);
        assert!(found.captures.iter().all(Option::is_none));
        let source =
            "(?<=".to_owned() + &"(?:".repeat(10000) + "(|){2}" + &"){2}".repeat(10000) + ")a";
        assert_eq!(
            &*ordinary(&source, false, false, false)
                .find(&JsString::from("a"), 0, true)
                .unwrap()
                .captures,
            &[Some(0..0)]
        );
        let matcher = ordinary(&format!("(?<=(|){{{huge},}})a"), false, false, false);
        let mut work = 0;
        assert!(
            matcher
                .find_with_work(&JsString::from("a"), 0, true, |n| {
                    work += n;
                    Ok::<_, ()>(())
                })
                .unwrap()
                .is_some()
        );
        assert!(work < 100);
    }

    #[test]
    fn nested_zero_reference_execution_snapshot() {
        let mut rows = String::new();
        for source in [
            r"((()\3){2}){2}a",
            r"((()\3){0,2}){2}a",
            r"((()\3){2})*a",
            r"((\2){2}){2}a",
            r"((\3()){2}){2}a",
            r"((()\3\3){2}){2}a",
            r"((()(\3)){2}){2}a",
            r"((\2){0,2}){2}a",
            r"((()\3)+)+a",
            r"((()\3)*)+a",
            r"((()\3)+)*a",
            r"((()\3){1,3}?){2}a",
            r"((^()\3){2}){2}a",
            r"((\b()\3){2}){2}a",
            r"((\B()\3){2}){2}a",
            r"((()^\3){0,2}){2}a",
            r"(?:(?:(()\2){2}){2}|()){2}a",
            r"(?:(?:(()\2){0,2}){2}|()){2}a",
            r"(?!(?:(()\2){2}){2}q)a",
            r"(?=(?:(()\2){2}){2}a)a",
            r"(?<=(a))((()\4){2}){2}b",
            r"(?:(?:(()\2){2}){2}a|b)+c",
            r"(?:(?:(()\2){2})?a|b)+c",
            r"((()\3){2}){2}a\1\2\3",
            r"((()\3){0,2}){2}a\1\2\3",
            r"((()\3){2}){2}µΜ",
            r"((\2\2){2}){2}a",
            r"((\3()\3){2}){2}a",
            r"((()\3\4()){2}){2}a",
            r"(?:(()\2){0}){2}a",
            r"((()(\3)\4){2}){2}a",
            r"((()(\3)(\4)\5){2}){2}a",
        ] {
            for (ignore_case, multiline, dot_all) in [
                (false, false, false),
                (true, true, false),
                (false, false, true),
                (true, false, true),
            ] {
                let matcher = ordinary(source, ignore_case, multiline, dot_all);
                for text in [
                    "", "a", "b", "ab", "aab", "ba", " abc", "aaarc", "aabcd", "\nab", "aa\nb",
                    "a\r\n", "µµΜ", " a", "qa",
                ] {
                    let input = JsString::from(text);
                    for (start, sticky) in [(0, false), (0, true), (1, true)] {
                        writeln!(rows,"{source:?} i={ignore_case} m={multiline} s={dot_all} input={input:?} start={start} sticky={sticky} {:?}",matcher.find(&input,start,sticky)).unwrap();
                    }
                }
            }
        }
        insta::assert_snapshot!(rows);
    }

    #[test]
    fn nested_zero_references_preserve_open_forward_local_and_optional_capture_ranges() {
        for (source, text, captures) in [
            (
                r"((()\3){2}){2}a",
                "qa",
                vec![Some(1..1), Some(1..1), Some(1..1)],
            ),
            (r"((()\3){0,2}){2}a", "qa", vec![Some(1..1), None, None]),
            (r"((()\3){2})*a", "qa", vec![None, None, None]),
            (r"((\2){2}){2}a", "qa", vec![Some(1..1), Some(1..1)]),
            (
                r"((\3()){2}){2}a",
                "qa",
                vec![Some(1..1), Some(1..1), Some(1..1)],
            ),
            (
                r"((\3()\3){2}){2}a",
                "qa",
                vec![Some(1..1), Some(1..1), Some(1..1)],
            ),
            (r"(?!(?:(()\2){2}){2}q)a", "qa", vec![None, None]),
        ] {
            let found = ordinary(source, false, false, false)
                .find(&JsString::from(text), 0, false)
                .unwrap();
            assert_eq!(found.range, 1..2, "{source}");
            assert_eq!(&*found.captures, &*captures, "{source}");
        }
        for source in [
            r"((\1){1,2}){2}a()",
            r"(((a)(\3)\4){0,2}){2}a",
            r"(((a)\3){0,2}){2}a",
            r"(?<=(((a)\3){0,2}){2})a",
        ] {
            assert!(
                RegExpBackreferenceMatcher::compile_ordinary_with_named_bindings_and_work(
                    &JsString::from(source),
                    false,
                    false,
                    false,
                    RegExpBackreferenceNamedBindings::default(),
                    |_| Ok::<_, ()>(())
                )
                .unwrap()
                .is_none(),
                "{source}"
            );
        }
    }

    #[test]
    fn nested_zero_reference_large_bounds_deep_slots_clones_and_negative_completed_undo_are_flat() {
        let huge = "184467440737095516160000000000000000000";
        let body = "(?:".to_owned() + &"(".repeat(100000) + &")".repeat(100000) + r"\100000){2}";
        let source = format!("(?:{body}){{{huge},}}a");
        let matcher = ordinary(&source, false, false, false);
        let copy = matcher.clone();
        drop(matcher);
        let found = copy.find(&JsString::from("qa"), 1, true).unwrap();
        assert_eq!(found.captures.len(), 100000);
        assert!(found.captures.iter().all(|r| r == &Some(1..1)));
        let mut work = 0;
        assert_eq!(
            copy.find_with_work(&JsString::from("qa"), 1, true, |n| {
                work += n;
                if work > 1000 {
                    Err("explicit empty-reference work")
                } else {
                    Ok(())
                }
            })
            .unwrap_err(),
            "explicit empty-reference work"
        );
        let source = format!("(?!(?:{body}){{2}}q)a");
        let found = ordinary(&source, false, false, false)
            .find(&JsString::from("a"), 0, true)
            .unwrap();
        assert_eq!(found.captures.len(), 100000);
        assert!(found.captures.iter().all(Option::is_none));
        let mut work = 0;
        let matcher = ordinary(
            &format!("(?:(()\\2){{2}}){{{huge},}}a"),
            false,
            false,
            false,
        );
        assert_eq!(
            matcher
                .find_with_work(&JsString::from("a"), 0, true, |n| {
                    work += n;
                    Ok::<_, ()>(())
                })
                .unwrap()
                .unwrap()
                .range,
            0..1
        );
        assert!(work < 100);
        let source = "(?:".repeat(10000) + r"(()\2){2}" + &"){2}".repeat(10000) + "a";
        let found = ordinary(&source, false, false, false)
            .find(&JsString::from("a"), 0, true)
            .unwrap();
        assert_eq!(&*found.captures, &[Some(0..0), Some(0..0)]);
    }

    #[test]
    fn nested_zero_count_execution_snapshot() {
        let mut rows = String::new();
        for source in [
            r"((){2}){2}a",
            r"((){0}){2}a",
            r"((){0,2}){2}a",
            r"((){1,2}){2}a",
            r"(()+)+a",
            r"(()*)+a",
            r"(()+)*a",
            r"((){2}){2}?a",
            r"((^){2}){2}a",
            r"((^){1,2}){2}a",
            r"((^){0,2}){2}a",
            r"((\b){1,}){2}a",
            r"((\B){2})+a",
            r"((^\b){2}){2}a",
            r"((a){0}){2}b",
            r"((ab){0}){1,3}b",
            r"((a\2){0}){2}b",
            r"((()\3){0}){2}b",
            r"((()?){2}){2}a",
            r"(((){0}){2}){3}a",
            r"(?:((){2}){2}|()){2}a",
            r"(?:((){0}){2}|()){2}a",
            r"(((){2})?){2}a",
            r"(?!((){2}){2}q)a",
            r"(?=((){2}){2}a)a",
            r"(?<=(a))((){2}){2}b",
            r"(?:((){2}){2}a|b)+c",
            r"(?:((){2})?a|b)+c",
            r"((){2}){2}a\1\2",
            r"((a){0}){2}µΜ",
        ] {
            for (ignore_case, multiline, dot_all) in [
                (false, false, false),
                (true, true, false),
                (false, false, true),
                (true, false, true),
            ] {
                let matcher = ordinary(source, ignore_case, multiline, dot_all);
                for text in [
                    "", "a", "b", "ab", "aab", "ba", " abc", "aaarc", "aabcd", "\nab", "aa\nb",
                    "a\r\n", "µµΜ", " a", "qa",
                ] {
                    let input = JsString::from(text);
                    for (start, sticky) in [(0, false), (0, true), (1, true)] {
                        writeln!(rows,"{source:?} i={ignore_case} m={multiline} s={dot_all} input={input:?} start={start} sticky={sticky} {:?}",matcher.find(&input,start,sticky)).unwrap();
                    }
                }
            }
        }
        insta::assert_snapshot!(rows);
    }

    #[test]
    fn nested_zero_counts_keep_required_enclosures_optional_slots_and_negative_rollback() {
        for (source, text, range, captures) in [
            (r"((){2}){2}a", "qa", 1..2, vec![Some(1..1), Some(1..1)]),
            (r"((){0,2}){2}a", "qa", 1..2, vec![Some(1..1), None]),
            (r"((){2})*a", "qa", 1..2, vec![None, None]),
            (r"((^){0,2}){2}a", "qa", 1..2, vec![Some(1..1), None]),
            (r"((a){0}){2}b", "qb", 1..2, vec![Some(1..1), None]),
            (r"(([a-z]){0}){2}b", "qb", 1..2, vec![Some(1..1), None]),
            (r"((.){0}){2}b", "qb", 1..2, vec![Some(1..1), None]),
            (r"((a\2){0}){2}b", "qb", 1..2, vec![Some(1..1), None]),
            (
                r"(((){0}){2}){3}a",
                "qa",
                1..2,
                vec![Some(1..1), Some(1..1), None],
            ),
            (r"(?!((){2}){2}q)a", "a", 0..1, vec![None, None]),
        ] {
            let found = ordinary(source, false, false, false)
                .find(&JsString::from(text), 0, false)
                .unwrap();
            assert_eq!(found.range, range, "{source}");
            assert_eq!(&*found.captures, &*captures, "{source}");
        }
        assert!(
            ordinary(r"((^){1,2}){2}a", false, false, false)
                .find(&JsString::from("qa"), 0, false)
                .is_none()
        );
        for source in [
            r"((a){0,2}){2}b",
            r"(([a-z]){0,2}){2}b",
            r"((\1){1,2}){2}a()",
            r"(((a)\3){0,2}){2}a",
            r"(?<=((a{0,2}){2}){2})a",
        ] {
            assert!(
                RegExpBackreferenceMatcher::compile_ordinary_with_named_bindings_and_work(
                    &JsString::from(source),
                    false,
                    false,
                    false,
                    RegExpBackreferenceNamedBindings::default(),
                    |_| Ok::<_, ()>(())
                )
                .unwrap()
                .is_none(),
                "{source}"
            );
        }
    }

    #[test]
    fn nested_zero_count_large_bounds_deep_slots_copies_and_completed_negative_undo_are_flat() {
        let huge = "184467440737095516160000000000000000000";
        let source = "(?:".to_owned()
            + &"(".repeat(100000)
            + &")".repeat(100000)
            + &format!("{{2}}){{{huge},}}a");
        let matcher = ordinary(&source, false, false, false);
        let copy = matcher.clone();
        drop(matcher);
        let found = copy.find(&JsString::from("qa"), 1, true).unwrap();
        assert_eq!(found.captures.len(), 100000);
        assert!(found.captures.iter().all(|r| r == &Some(1..1)));
        let mut work = 0;
        assert_eq!(
            copy.find_with_work(&JsString::from("qa"), 1, true, |n| {
                work += n;
                if work > 1000 {
                    Err("explicit nested zero work")
                } else {
                    Ok(())
                }
            })
            .unwrap_err(),
            "explicit nested zero work"
        );
        let source = "(?!(?:".to_owned() + &"(".repeat(100000) + &")".repeat(100000) + "{2}){2}q)a";
        let found = ordinary(&source, false, false, false)
            .find(&JsString::from("a"), 0, true)
            .unwrap();
        assert_eq!(found.captures.len(), 100000);
        assert!(found.captures.iter().all(Option::is_none));
        let mut work = 0;
        let matcher = ordinary(
            &format!("(?:(^){{1,{huge}}}){{{huge},}}a"),
            false,
            false,
            false,
        );
        assert_eq!(
            matcher
                .find_with_work(&JsString::from("a"), 0, true, |n| {
                    work += n;
                    Ok::<_, ()>(())
                })
                .unwrap()
                .unwrap()
                .range,
            0..1
        );
        assert!(work < 100);
        let source = "(?:".repeat(10000) + "(){2}" + &"){2}".repeat(10000) + "a";
        let matcher = ordinary(&source, false, false, false);
        assert_eq!(
            &*matcher
                .find(&JsString::from("a"), 0, true)
                .unwrap()
                .captures,
            &[Some(0..0)]
        );
    }

    #[test]
    fn pure_zero_width_choice_execution_snapshot() {
        let mut rows = String::new();
        for source in [
            r"(|){2}a",
            r"(|){0}a",
            r"(?:|){2}a",
            r"(?:|)+",
            r"(()|()){2}a",
            r"(()|())*a",
            r"(()|())+?a",
            r"(^|$){2}a",
            r"(^|$)?a",
            r"(^|$){1,3}?a",
            r"(\b|\B)+a",
            r"(($)|(^)){2}a",
            r"((^)|()){2}a",
            r"(?:(^)|(\b)){2}a",
            r"(?:(^)(\B)|(\b)){2}a",
            r"((())|(())){2}a",
            r"(?:(^)|($)){2}a\1\2",
            r"(?:(^)|()){1,4}?a",
            r"(?:()|()){2}a\1\2",
            r"(?:()|())*a\1\2",
            r"(?:(?:()|()){2}){3}a",
            r"(?:(?:()|())?){2}a",
            r"(?:(?:()|()){2}a|b)+c",
            r"(?:(?:()|())?a|b)+c",
            r"(?<=(a))(?:()|()){2}b",
            r"(?:(?<=a)|^){2}b",
            r"(?!(?:()|()){2}q)a",
            r"(?=(?:()|()){2}a)a",
            r"(?:()|()){2}µΜ",
            r"((?:|){2})a",
        ] {
            for (ignore_case, multiline, dot_all) in [
                (false, false, false),
                (true, true, false),
                (false, false, true),
                (true, false, true),
            ] {
                let matcher = ordinary(source, ignore_case, multiline, dot_all);
                for text in [
                    "", "a", "b", "ab", "aab", "ba", " abc", "aaarc", "aabcd", "\nab", "aa\nb",
                    "a\r\n", "µµΜ", " a", "qa",
                ] {
                    let input = JsString::from(text);
                    for (start, sticky) in [(0, false), (0, true), (1, true)] {
                        writeln!(rows,"{source:?} i={ignore_case} m={multiline} s={dot_all} input={input:?} start={start} sticky={sticky} {:?}",matcher.find(&input,start,sticky)).unwrap();
                    }
                }
            }
        }
        insta::assert_snapshot!(rows);
    }

    #[test]
    fn pure_zero_width_choices_restore_failed_arms_and_skip_optional_slots() {
        for (source, text, range, captures) in [
            (
                r"(()|()){2}a",
                "qa",
                1..2,
                vec![Some(1..1), Some(1..1), None],
            ),
            (r"(()|())*a", "qa", 1..2, vec![None, None, None]),
            (r"(^|$)?a", "qa", 1..2, vec![None]),
            (
                r"(($)|(^)){2}a",
                "a",
                0..1,
                vec![Some(0..0), None, Some(0..0)],
            ),
            (
                r"(?:(^)(\B)|(\b)){2}a",
                "a",
                0..1,
                vec![None, None, Some(0..0)],
            ),
            (r"(?!(?:()|()){2}q)a", "a", 0..1, vec![None, None]),
            (r"(?:(?:()|())?){2}a", "qa", 1..2, vec![None, None]),
        ] {
            let found = ordinary(source, false, false, false)
                .find(&JsString::from(text), 0, false)
                .unwrap();
            assert_eq!(found.range, range, "{source}");
            assert_eq!(&*found.captures, &*captures, "{source}");
        }
        assert!(
            ordinary(r"(^|$)+a", false, false, false)
                .find(&JsString::from("qa"), 0, false)
                .is_none()
        );
        for source in [
            r"(?:a|())+b",
            r"(?:(\1)|()){2}a()",
            r"(?<=((a)|()){2})a",
            r"(?:(?:a|())+)*b",
        ] {
            assert!(
                RegExpBackreferenceMatcher::compile_ordinary_with_named_bindings_and_work(
                    &JsString::from(source),
                    false,
                    false,
                    false,
                    RegExpBackreferenceNamedBindings::default(),
                    |_| Ok::<_, ()>(())
                )
                .unwrap()
                .is_none(),
                "{source}"
            );
        }
    }

    #[test]
    fn pure_zero_width_choice_large_bounds_nested_slots_copies_and_work_are_flat() {
        let huge = "184467440737095516160000000000000000000";
        let source = "(?:".to_owned()
            + &"(".repeat(100000)
            + &")".repeat(100000)
            + &format!("|()){{{huge},}}a");
        let matcher = ordinary(&source, false, false, false);
        let copy = matcher.clone();
        drop(matcher);
        let found = copy.find(&JsString::from("qa"), 1, true).unwrap();
        assert_eq!(found.captures.len(), 100001);
        assert!(found.captures[..100000].iter().all(|r| r == &Some(1..1)));
        assert_eq!(found.captures[100000], None);
        let mut work = 0;
        assert_eq!(
            copy.find_with_work(&JsString::from("qa"), 1, true, |n| {
                work += n;
                if work > 1000 {
                    Err("explicit zero-choice work")
                } else {
                    Ok(())
                }
            })
            .unwrap_err(),
            "explicit zero-choice work"
        );
        for count in [format!("{{{huge}}}"), format!("{{{huge},}}?"), "+".into()] {
            let matcher = ordinary(&format!("(?:^|$){count}a"), false, false, false);
            let mut work = 0;
            assert_eq!(
                matcher
                    .find_with_work(&JsString::from("a"), 0, true, |n| {
                        work += n;
                        Ok::<_, ()>(())
                    })
                    .unwrap()
                    .unwrap()
                    .range,
                0..1
            );
            assert!(work < 100);
        }
    }

    #[test]
    fn zero_width_lookbehind_count_execution_snapshot() {
        let mut rows = String::new();
        for source in [
            r"(?<=()?)a",
            r"(?<=()*?)a",
            r"(?<=()+)a",
            r"(?<=()+?)a",
            r"(?<=(){0,2})a",
            r"(?<=(){1,2}?)a",
            r"(?<=(){2,})a",
            r"(?<=(?:)*)a",
            r"(?<=(()){1,3})a",
            r"(?<=(()()){1,})a",
            r"(?<=(^)?)a",
            r"(?<=(^)+)a",
            r"(?<=(\b){0,3})a",
            r"(?<=(\b){1,3}?)a",
            r"(?<=(\B)+)a",
            r"(?<=(^\b)+)a",
            r"(?<=(^$){0,2})a",
            r"(?<!()+)a",
            r"(?<!(^)+)a",
            r"(?<!()+q)b",
            r"(?<=()+a)b",
            r"(?<=a()*)b",
            r"(?<=()+|())a",
            r"(?<=()*|())a",
            r"(?<=()+)a\1",
            r"(?<=(?:()+)(?<=a))b",
            r"((?<=()+)){2}a\1\2",
            r"((?<=()+))*a\1\2",
            r"(?:(?<=()+)a|b)+c",
            r"(?<=((?:)*µ))Μ",
        ] {
            for (ignore_case, multiline, dot_all) in [
                (false, false, false),
                (true, true, false),
                (false, false, true),
                (true, false, true),
            ] {
                let matcher = ordinary(source, ignore_case, multiline, dot_all);
                for text in [
                    "", "a", "b", "ab", "aab", "ba", " abc", "aaarc", "aabcd", "\nab", "aa\nb",
                    "a\r\n", "µµΜ", " a", "qa",
                ] {
                    let input = JsString::from(text);
                    for (start, sticky) in [(0, false), (0, true), (1, true)] {
                        writeln!(rows,"{source:?} i={ignore_case} m={multiline} s={dot_all} input={input:?} start={start} sticky={sticky} {:?}",matcher.find(&input,start,sticky)).unwrap();
                    }
                }
            }
        }
        insta::assert_snapshot!(rows);
    }

    #[test]
    fn zero_width_lookbehind_counts_skip_optional_effects_and_keep_required_boundaries() {
        for (source, text, range, captures) in [
            (r"(?<=()?)a", "qa", 1..2, vec![None]),
            (r"(?<=()+?)a", "qa", 1..2, vec![Some(1..1)]),
            (r"(?<=(^)?)a", "qa", 1..2, vec![None]),
            (r"(?<=(^)+)a", "a", 0..1, vec![Some(0..0)]),
            (r"(?<=(\b){1,3}?)a", " a", 1..2, vec![Some(1..1)]),
            (r"(?<=()+a)b", "ab", 1..2, vec![Some(0..0)]),
            (r"(?<=a()*)b", "ab", 1..2, vec![None]),
            (r"(?<=()*|())a", "a", 0..1, vec![None, None]),
            (r"(?<=()+|())a", "a", 0..1, vec![Some(0..0), None]),
            (r"(?<!()+q)b", "ab", 1..2, vec![None]),
        ] {
            let found = ordinary(source, false, false, false)
                .find(&JsString::from(text), 0, false)
                .unwrap();
            assert_eq!(found.range, range, "{source}");
            assert_eq!(&*found.captures, &*captures, "{source}");
        }
        assert!(
            ordinary(r"(?<=(^)+)a", false, false, false)
                .find(&JsString::from("qa"), 0, false)
                .is_none()
        );
        for source in [
            r"(?<=a{1,2})b",
            r"(?<=(a?){1,2})b",
            r"(?<=((?=a+)){1,2})a",
            r"(?<=(|a)+)a",
            r"(?<=(\2)+)a()",
            r"(?<=((a*)+))a",
        ] {
            assert!(
                RegExpBackreferenceMatcher::compile_ordinary_with_named_bindings_and_work(
                    &JsString::from(source),
                    false,
                    false,
                    false,
                    RegExpBackreferenceNamedBindings::default(),
                    |_| Ok::<_, ()>(())
                )
                .unwrap()
                .is_none(),
                "{source}"
            );
        }
    }

    #[test]
    fn zero_width_lookbehind_unrepresentable_counts_and_deep_slots_stay_flat() {
        let enormous = "184467440737095516160000000000000000000";
        for count in [
            format!("{{{enormous}}}"),
            format!("{{{enormous},}}"),
            format!("{{{enormous},{enormous}}}?"),
            "+".into(),
        ] {
            let source = format!("(?<=(^){count})a");
            let matcher = ordinary(&source, false, false, false);
            let mut work = 0;
            let found = matcher
                .find_with_work(&JsString::from("a"), 0, true, |n| {
                    work += n;
                    Ok::<_, ()>(())
                })
                .unwrap()
                .unwrap();
            assert_eq!(&*found.captures, &[Some(0..0)]);
            assert!(work < 100);
            assert!(matcher.find(&JsString::from("qa"), 1, true).is_none());
        }
        let source = format!("(?<=(^){{0,{enormous}}})a");
        assert_eq!(
            &*ordinary(&source, false, false, false)
                .find(&JsString::from("qa"), 1, true)
                .unwrap()
                .captures,
            &[None]
        );
        let source = "(?<=".to_owned()
            + &"(".repeat(100000)
            + &")".repeat(100000)
            + &format!("{{{enormous},}})a");
        let matcher = ordinary(&source, false, false, false);
        let copy = matcher.clone();
        drop(matcher);
        let found = copy.find(&JsString::from("qa"), 1, true).unwrap();
        assert!(found.captures.iter().all(|r| r == &Some(1..1)));
        assert_eq!(found.captures.len(), 100000);
        let mut work = 0;
        assert_eq!(
            copy.find_with_work(&JsString::from("qa"), 1, true, |n| {
                work += n;
                if work > 1000 {
                    Err("explicit zero-width work")
                } else {
                    Ok(())
                }
            })
            .unwrap_err(),
            "explicit zero-width work"
        );
        let source = "(?<!".to_owned() + &"(".repeat(100000) + &")".repeat(100000) + "+q)b";
        let found = ordinary(&source, false, false, false)
            .find(&JsString::from("ab"), 1, true)
            .unwrap();
        assert!(found.captures.iter().all(Option::is_none));
    }

    #[test]
    fn fixed_lookbehind_empty_count_execution_snapshot() {
        let mut rows = String::new();
        for source in [
            r"(?<=(){2})a",
            r"(?<!(){2})a",
            r"(?<=(){0})a",
            r"(?<!(){0})a",
            r"(?<=(()){2})a",
            r"(?<=(()()){2})a",
            r"(?<=((?:){2}))a",
            r"(?<=(?:){2})a",
            r"(?<=(?:){0})a",
            r"(?<=(){2}a)b",
            r"(?<=a(){2})b",
            r"(?<=(((){2})a))b",
            r"(?<=(((){0})a))b",
            r"(?<!(){2}q)b",
            r"(?<=(a)((){2}))b",
            r"(?<=(){2}|())a",
            r"(?<=(){0}|())a",
            r"(?<=(?:(){2})(?<=a))b",
            r"(?<=(?:(){2})(?<!b))a",
            r"(?<=(){2})a\1",
            r"(?:(?<=(){2})a|b)+c",
            r"((?<=(){2})){2}a\1\2",
            r"((?<=(){2}))*a\1\2",
            r"(?<=((?:){2}µ))Μ",
        ] {
            for (ignore_case, multiline, dot_all) in [
                (false, false, false),
                (true, true, false),
                (false, false, true),
                (true, false, true),
            ] {
                let matcher = ordinary(source, ignore_case, multiline, dot_all);
                for text in [
                    "", "a", "b", "ab", "aab", "ba", " abc", "aaarc", "aabcd", "\nab", "aa\nb",
                    "a\r\n", "µµΜ", " a", "qa",
                ] {
                    let input = JsString::from(text);
                    for (start, sticky) in [(0, false), (0, true), (1, true)] {
                        writeln!(rows,"{source:?} i={ignore_case} m={multiline} s={dot_all} input={input:?} start={start} sticky={sticky} {:?}",matcher.find(&input,start,sticky)).unwrap();
                    }
                }
            }
        }
        insta::assert_snapshot!(rows);
    }

    #[test]
    fn fixed_empty_count_lookbehind_captures_use_their_local_boundary_and_zero_count_slots() {
        for (source, text, range, captures) in [
            (r"(?<=(){2})a", "qa", 1..2, vec![Some(1..1)]),
            (r"(?<=(){0})a", "qa", 1..2, vec![None]),
            (r"(?<=(()){2})a", "a", 0..1, vec![Some(0..0), Some(0..0)]),
            (r"(?<=(){2}a)b", "ab", 1..2, vec![Some(0..0)]),
            (r"(?<=a(){2})b", "ab", 1..2, vec![Some(1..1)]),
            (
                r"(?<=(((){2})a))b",
                "ab",
                1..2,
                vec![Some(0..1), Some(0..0), Some(0..0)],
            ),
            (
                r"(?<=(((){0})a))b",
                "ab",
                1..2,
                vec![Some(0..1), Some(0..0), None],
            ),
            (r"(?<!(){2}q)b", "ab", 1..2, vec![None]),
            (r"(?<=(){2}|())a", "a", 0..1, vec![Some(0..0), None]),
            (r"(?<=(){0}|())a", "a", 0..1, vec![None, None]),
        ] {
            let found = ordinary(source, false, false, false)
                .find(&JsString::from(text), 0, false)
                .unwrap();
            assert_eq!(found.range, range, "{source}");
            assert_eq!(&*found.captures, &*captures, "{source}");
        }
        for source in [
            r"(?<=((?=a+)){1,2})a",
            r"(?<=((?=a+)){2})a",
            r"(?<=(|a){2})a",
            r"(?<=(\2){1,2})a()",
            r"(?<=((a{0,2}){2}){2})a",
        ] {
            assert!(
                RegExpBackreferenceMatcher::compile_ordinary_with_named_bindings_and_work(
                    &JsString::from(source),
                    false,
                    false,
                    false,
                    RegExpBackreferenceNamedBindings::default(),
                    |_| Ok::<_, ()>(())
                )
                .unwrap()
                .is_none(),
                "{source}"
            );
        }
    }

    #[test]
    fn deep_fixed_empty_count_capture_slots_huge_counts_clones_and_work_stay_flat() {
        let source = "(?<=".to_owned()
            + &"(".repeat(100000)
            + &")".repeat(100000)
            + &format!("{{{}}})a", usize::MAX);
        let matcher = ordinary(&source, false, false, false);
        let copy = matcher.clone();
        drop(matcher);
        let found = copy.find(&JsString::from("a"), 0, true).unwrap();
        assert_eq!(found.captures.len(), 100000);
        assert!(found.captures.iter().all(|r| r == &Some(0..0)));
        let mut work = 0;
        assert_eq!(
            copy.find_with_work(&JsString::from("a"), 0, true, |n| {
                work += n;
                if work > 1000 {
                    Err("explicit empty capture work")
                } else {
                    Ok(())
                }
            })
            .unwrap_err(),
            "explicit empty capture work"
        );
        let source = format!("(?<=(?:){{{}}})a", usize::MAX);
        let matcher = ordinary(&source, false, false, false);
        let mut work = 0;
        assert_eq!(
            matcher
                .find_with_work(&JsString::from("a"), 0, true, |n| {
                    work += n;
                    Ok::<_, ()>(())
                })
                .unwrap()
                .unwrap()
                .range,
            0..1
        );
        assert!(work < 100);
        let source = "(?<!".to_owned() + &"(".repeat(100000) + &")".repeat(100000) + "{2}q)b";
        let found = ordinary(&source, false, false, false)
            .find(&JsString::from("ab"), 1, true)
            .unwrap();
        assert!(found.captures.iter().all(Option::is_none));
    }

    #[test]
    fn fixed_lookbehind_assertion_count_execution_snapshot() {
        let mut rows = String::new();
        for source in [
            r"(?<=(?:^){2})a",
            r"(?<!(?:^){2})a",
            r"(?<=(^){2})a",
            r"(?<=(^){0})a",
            r"(?<=((^){0}))a",
            r"(?<=(?:\b){2})a",
            r"(?<=(?:\B){2})a",
            r"(?<=(\b){2})a",
            r"(?<=(?:$){2})",
            r"(?<=($){2})",
            r"(?<=(a\B){2})b",
            r"(?<=((a)\B()){2})b",
            r"(?<=((\b)a(\B)){1})b",
            r"(?<!((a)\B){2}q)c",
            r"(?<=(?:^a){1})b",
            r"(?<=(?:\ba\B){1})b",
            r"(?<=(?:a\B){0})b",
            r"(?<=((?:a\B){0}))b",
            r"(?<=((^)){2}|((\b)){2})a",
            r"(?<=a(?<=(?:\b){2}))b",
            r"(?:(?<=(a\B){2})b|c)+d",
            r"((?<=(\b){2})){2}a\1\2",
            r"((?<=(\b){2}))*a\1\2",
            r"(?<=(µ\B){2})Μ",
        ] {
            for (ignore_case, multiline, dot_all) in [
                (false, false, false),
                (true, true, false),
                (false, false, true),
                (true, false, true),
            ] {
                let matcher = ordinary(source, ignore_case, multiline, dot_all);
                for text in [
                    "", "a", "b", "ab", "aab", "ba", " abc", "aaarc", "aabcd", "\nab", "aa\nb",
                    "a\r\n", "µµΜ", " a", "qa",
                ] {
                    let input = JsString::from(text);
                    for (start, sticky) in [(0, false), (0, true), (1, true)] {
                        writeln!(rows,"{source:?} i={ignore_case} m={multiline} s={dot_all} input={input:?} start={start} sticky={sticky} {:?}",matcher.find(&input,start,sticky)).unwrap();
                    }
                }
            }
        }
        insta::assert_snapshot!(rows);
    }

    #[test]
    fn counted_lookbehind_assertions_use_zero_width_offsets_full_context_and_rollback() {
        for (source, text, range, captures) in [
            (r"(?<=(^){2})a", "a", 0..1, vec![Some(0..0)]),
            (r"(?<=((^){0}))a", "qa", 1..2, vec![Some(1..1), None]),
            (r"(?<=(a\B){2})b", "aab", 2..3, vec![Some(0..1)]),
            (
                r"(?<=((a)\B()){2})b",
                "aab",
                2..3,
                vec![Some(0..1), Some(0..1), Some(1..1)],
            ),
            (
                r"(?<=((\b)a(\B)){1})b",
                "ab",
                1..2,
                vec![Some(0..1), Some(0..0), Some(1..1)],
            ),
            (r"(?<!((a)\B){2}q)c", "aaarc", 4..5, vec![None, None]),
            (
                r"(?<=((^)){2}|((\b)){2})a",
                " a",
                1..2,
                vec![None, None, Some(1..1), Some(1..1)],
            ),
            (r"(?<=(^){2})a", "\na", 1..2, vec![Some(1..1)]),
            (r"(?<=($){2})", "a", 1..1, vec![Some(1..1)]),
        ] {
            let found = ordinary(source, false, true, false)
                .find(&JsString::from(text), 0, false)
                .unwrap();
            assert_eq!(found.range, range, "{source}");
            assert_eq!(&*found.captures, &*captures, "{source}");
        }
        for source in [
            r"(?<=(?:a\B){1,2})b",
            r"(?<=(?:(?=a+)){2})a",
            r"(?<=(a\B|b\B){2})c",
            r"(?<=((a\B){2}){2})c",
            r"(?<=((?=a+)){2})a",
        ] {
            assert!(
                RegExpBackreferenceMatcher::compile_ordinary_with_named_bindings_and_work(
                    &JsString::from(source),
                    false,
                    false,
                    false,
                    RegExpBackreferenceNamedBindings::default(),
                    |_| Ok::<_, ()>(())
                )
                .unwrap()
                .is_none(),
                "{source}"
            );
        }
    }

    #[test]
    fn deep_counted_lookbehind_assertion_captures_huge_counts_and_work_are_flat() {
        let source = "(?<=".to_owned()
            + &"(".repeat(100000)
            + "^"
            + &")".repeat(100000)
            + &format!("{{{}}})a", usize::MAX);
        let matcher = ordinary(&source, false, false, false);
        let copy = matcher.clone();
        drop(matcher);
        let found = copy.find(&JsString::from("a"), 0, true).unwrap();
        assert_eq!(found.range, 0..1);
        assert_eq!(found.captures.len(), 100000);
        assert!(found.captures.iter().all(|r| r == &Some(0..0)));
        let mut work = 0;
        assert_eq!(
            copy.find_with_work(&JsString::from("a"), 0, true, |n| {
                work += n;
                if work > 1000 {
                    Err("explicit assertion capture work")
                } else {
                    Ok(())
                }
            })
            .unwrap_err(),
            "explicit assertion capture work"
        );
        let source = format!("(?<=(?:^){{{}}})a", usize::MAX);
        let matcher = ordinary(&source, false, false, false);
        let mut work = 0;
        assert_eq!(
            matcher
                .find_with_work(&JsString::from("a"), 0, true, |n| {
                    work += n;
                    Ok::<_, ()>(())
                })
                .unwrap()
                .unwrap()
                .range,
            0..1
        );
        assert!(work < 100);
        let source = "(?<!".to_owned() + &"(".repeat(100000) + "^" + &")".repeat(100000) + "{2}q)b";
        let found = ordinary(&source, false, false, false)
            .find(&JsString::from("ab"), 1, true)
            .unwrap();
        assert!(found.captures.iter().all(Option::is_none));
    }

    #[test]
    fn fixed_lookbehind_choice_execution_snapshot() {
        let mut rows = String::new();
        for source in [
            r"(?<=a|b)c",
            r"(?<!a|b)c",
            r"(?<=ab|cd)e",
            r"(?<=(a|b))c",
            r"(?<=(a)b|a(c))d",
            r"(?<=((a)b|a(c)))d",
            r"(?<!((a)b|a(c)))d",
            r"(?<=(a|b)(c|d))e",
            r"(?<=(a|b))\1",
            r"(?<=((a)|a))c\2",
            r"(?<=(a|(a)))c\2",
            r"(?<=()|())a",
            r"(?<!()|())a",
            r"(?<=(\b|^))a",
            r"(?<=(^a|\ba))b",
            r"(?<=([ab]){2}|(a){2})c",
            r"(?<=((?:ab){2}|(?:ba){2}))c",
            r"(?<=(?<=a|b))c",
            r"(?<=(a|b)(?<!(c|d)))e",
            r"(?:(?<=(a|b))c|d)+e",
            r"((?<=(a|b))){2}c\1\2",
            r"((?<=(a|b)))*c\1\2",
            r"(?=(ab(?<=(a|b)(a|b))c))\1\2\3",
            r"(?<=(µ|Μ))a",
        ] {
            for (ignore_case, multiline, dot_all) in [
                (false, false, false),
                (true, true, false),
                (false, false, true),
                (true, false, true),
            ] {
                let matcher = ordinary(source, ignore_case, multiline, dot_all);
                for text in [
                    "", "a", "abc", "acd", "abce", "bcde", "abcab", "ac", "abcc", "aabc", "ababc",
                    "bac", "abcdde", "\nab", "µa",
                ] {
                    let input = JsString::from(text);
                    for (start, sticky) in [(0, false), (0, true), (1, true)] {
                        writeln!(rows,"{source:?} i={ignore_case} m={multiline} s={dot_all} input={input:?} start={start} sticky={sticky} {:?}",matcher.find(&input,start,sticky)).unwrap();
                    }
                }
            }
        }
        insta::assert_snapshot!(rows);
    }

    #[test]
    fn fixed_lookbehind_choices_restore_partial_captures_keep_source_order_and_are_atomic() {
        for (source, text, range, captures) in [
            (r"(?<=(a)b|a(c))d", "acd", 2..3, vec![None, Some(1..2)]),
            (
                r"(?<=((a)b|a(c)))d",
                "acd",
                2..3,
                vec![Some(0..2), None, Some(1..2)],
            ),
            (r"(?<!((a)b|a(c)))d", "aqd", 2..3, vec![None, None, None]),
            (r"(?<=(a|(a)))c\2", "ac", 1..2, vec![Some(0..1), None]),
            (r"(?<=()|())a", "a", 0..1, vec![Some(0..0), None]),
            (
                r"(?<=([ab]){2}|(a){2})c",
                "abc",
                2..3,
                vec![Some(0..1), None],
            ),
            (r"(?<=(?<=a|b))c", "bc", 1..2, vec![]),
        ] {
            let found = ordinary(source, false, false, false)
                .find(&JsString::from(text), 0, false)
                .unwrap();
            assert_eq!(found.range, range, "{source}");
            assert_eq!(&*found.captures, &*captures, "{source}");
        }
        assert!(
            ordinary(r"(?<=((a)|a))c\2", false, false, false)
                .find(&JsString::from("ac"), 0, false)
                .is_none()
        );
        for source in [
            r"(?<=a|bb)c",
            r"(?<=(a|bb))c",
            r"(?<=(a|b){2})c",
            r"(?<=(a|b)\1)c",
            r"(?<=a|(?=a+)b)c",
        ] {
            assert!(
                RegExpBackreferenceMatcher::compile_ordinary_with_named_bindings_and_work(
                    &JsString::from(source),
                    false,
                    false,
                    false,
                    RegExpBackreferenceNamedBindings::default(),
                    |_| Ok::<_, ()>(())
                )
                .unwrap()
                .is_none(),
                "{source}"
            );
        }
    }

    #[test]
    fn deep_fixed_lookbehind_choice_proofs_frames_atomic_discard_and_work_are_flat() {
        let source = "(?<=".to_owned() + &"(?:".repeat(100000) + "a" + &"|a)".repeat(100000) + ")b";
        let matcher = ordinary(&source, false, false, false);
        let copy = matcher.clone();
        drop(matcher);
        assert_eq!(
            copy.find(&JsString::from("ab"), 1, true).unwrap().range,
            1..2
        );
        assert!(copy.find(&JsString::from("qb"), 1, true).is_none());
        let mut work = 0;
        assert_eq!(
            copy.find_with_work(&JsString::from("ab"), 1, true, |n| {
                work += n;
                if work > 1000 {
                    Err("explicit fixed choice work")
                } else {
                    Ok(())
                }
            })
            .unwrap_err(),
            "explicit fixed choice work"
        );
        let source = format!("(?<=a{{{0}}}|b{{{0}}})c", usize::MAX);
        let matcher = ordinary(&source, false, false, false);
        let mut work = 0;
        assert!(
            matcher
                .find_with_work(&JsString::from("abc"), 2, true, |n| {
                    work += n;
                    Ok::<_, ()>(())
                })
                .unwrap()
                .is_none()
        );
        assert!(work < 100);
    }

    #[test]
    fn fixed_lookbehind_repeated_capture_execution_snapshot() {
        let mut rows = String::new();
        for source in [
            r"(?<=([ab]){2})c",
            r"(?<!([ab]){2})c",
            r"(?<=(a){0})b",
            r"(?<!(a){0})b",
            r"(?<=((a)b){2})c",
            r"(?<=(a(b)){2})c",
            r"(?<=((a){2}))b",
            r"(?<=(a()){2})b",
            r"(?<=(a){2}?)b",
            r"(?<=(a){2,2})b",
            r"(?<=(a){0}())b",
            r"(?<=(a)(b){2})c",
            r"(?<=([ab]){2})\1",
            r"(?<=((a)b){2})\1\2",
            r"(?<!((a)b){2}q)c",
            r"(?<=([ab]){2}(?<=([ab]){2}))c",
            r"(?<=([ab]){2}(?<!(c){2}))c",
            r"(?<=^([ab]){2})c",
            r"(?<=(µ){2})Μ",
            r"(?<=(.){2})b",
            r"(?:(?<=([ab]){2})c|d)+e",
            r"((?<=([ab]){2})){2}c\1\2",
            r"((?<=([ab]){2}))*c\1\2",
            r"(?=(ab(?<=([ab]){2})c))\1\2",
        ] {
            for (ignore_case, multiline, dot_all) in [
                (false, false, false),
                (true, true, false),
                (false, false, true),
                (true, false, true),
            ] {
                let matcher = ordinary(source, ignore_case, multiline, dot_all);
                for text in [
                    "",
                    "a",
                    "c",
                    "abc",
                    "ababc",
                    "ababa",
                    "ababb",
                    "ababcdde",
                    "ababqc",
                    "a1a2c",
                    "µΜµΜa",
                    "\nababc",
                    "a\na\nb",
                    "ababc ababc",
                    "ababab",
                ] {
                    let input = JsString::from(text);
                    for (start, sticky) in [(0, false), (0, true), (1, true)] {
                        writeln!(rows,"{source:?} i={ignore_case} m={multiline} s={dot_all} input={input:?} start={start} sticky={sticky} {:?}",matcher.find(&input,start,sticky)).unwrap();
                    }
                }
            }
        }
        insta::assert_snapshot!(rows);
    }

    #[test]
    fn backward_counted_captures_keep_leftmost_iteration_and_clear_zero_counts() {
        for (source, text, range, captures) in [
            (r"(?<=([ab]){2})c", "abc", 2..3, vec![Some(0..1)]),
            (
                r"(?<=((a)b){2})c",
                "ababc",
                4..5,
                vec![Some(0..2), Some(0..1)],
            ),
            (
                r"(?<=(a(b)){2})c",
                "ababc",
                4..5,
                vec![Some(0..2), Some(1..2)],
            ),
            (r"(?<=((a){2}))b", "aab", 2..3, vec![Some(0..2), Some(0..1)]),
            (r"(?<=(a()){2})b", "aab", 2..3, vec![Some(0..1), Some(1..1)]),
            (r"(?<=(a){0}())b", "b", 0..1, vec![None, Some(0..0)]),
            (r"(?<!((a)b){2}q)c", "ababrc", 5..6, vec![None, None]),
            (r"(?<=([ab]){2})\1", "aba", 2..3, vec![Some(0..1)]),
            (
                r"(?<=([ab]){2}(?<!(c){2}))c",
                "abc",
                2..3,
                vec![Some(0..1), None],
            ),
        ] {
            let found = ordinary(source, false, false, false)
                .find(&JsString::from(text), 0, false)
                .unwrap();
            assert_eq!(found.range, range, "{source}");
            assert_eq!(&*found.captures, &*captures, "{source}");
        }
        for source in [
            r"(?<=(a){1,2})b",
            r"(?<=(a|b){2})c",
            r"(?<=(a(?=a)){2})c",
            r"(?<=((a){2}){2})c",
            r"(?<=(a\1){1,2})c",
            r"(?<=((?=a+)){2})c",
        ] {
            assert!(
                RegExpBackreferenceMatcher::compile_ordinary_with_named_bindings_and_work(
                    &JsString::from(source),
                    false,
                    false,
                    false,
                    RegExpBackreferenceNamedBindings::default(),
                    |_| Ok::<_, ()>(())
                )
                .unwrap()
                .is_none(),
                "{source}"
            );
        }
    }

    #[test]
    fn deep_backward_counted_capture_slots_clones_work_and_huge_prefixes_are_flat() {
        let source =
            "(?<=".to_owned() + &"(".repeat(100000) + "[ab]" + &")".repeat(100000) + "{2})c";
        let matcher = ordinary(&source, false, false, false);
        let copy = matcher.clone();
        drop(matcher);
        let found = copy.find(&JsString::from("abc"), 2, true).unwrap();
        assert_eq!(found.range, 2..3);
        assert_eq!(found.captures.len(), 100000);
        assert!(found.captures.iter().all(|r| r == &Some(0..1)));
        let mut work = 0;
        assert_eq!(
            copy.find_with_work(&JsString::from("abc"), 2, true, |n| {
                work += n;
                if work > 1000 {
                    Err("explicit counted capture work")
                } else {
                    Ok(())
                }
            })
            .unwrap_err(),
            "explicit counted capture work"
        );
        let source = format!("(?<=([ab]){{{}}})c", usize::MAX);
        let matcher = ordinary(&source, false, false, false);
        let mut work = 0;
        assert!(
            matcher
                .find_with_work(&JsString::from("abc"), 2, true, |n| {
                    work += n;
                    Ok::<_, ()>(())
                })
                .unwrap()
                .is_none()
        );
        assert!(work < 100);
        let source =
            "(?<!".to_owned() + &"(".repeat(100000) + "[ab]" + &")".repeat(100000) + "{2}q)c";
        let found = ordinary(&source, false, false, false)
            .find(&JsString::from("abrc"), 3, true)
            .unwrap();
        assert!(found.captures.iter().all(Option::is_none));
    }

    #[test]
    fn fixed_lookbehind_sequence_execution_snapshot() {
        let mut rows = String::new();
        for source in [
            r"(?<=(?:ab){2})c",
            r"(?<!(?:ab){2})c",
            r"(?<=(?:ab){0})c",
            r"(?<!(?:ab){0})c",
            r"(?<=(?:ab){2,2})c",
            r"(?<=(?:ab){2}?)c",
            r"(?<=((?:ab){2}))c",
            r"(?<=((?:ab){2}))\1",
            r"(?<!((?:ab){2})q)c",
            r"(?<=((?:[a-c]\d){2}))c",
            r"(?<=(?:a.){2})b",
            r"(?<=(?:µΜ){2})a",
            r"(?<=^(?:ab){2})c",
            r"(?<=(?:ab){2}\b)c",
            r"(?<=(?:ab){2}\B)c",
            r"(?<=a(?:ba){2})b",
            r"(?<=(?:ab){2}(?<=ab))c",
            r"(?<=(?:ab){2}(?<!ba))c",
            r"(?<=((?:ab){2})(?<!(ba)))c",
            r"(?:(?<=(?:ab){2})c|d)+e",
            r"((?<=(?:ab){2})){2}c\1",
            r"((?<=(?:ab){2}))*c\1",
            r"(?=(a(?:ba){2}(?<=((?:ba){2}))))\1\2",
            r"(?<=(?:a(?:b)){2})c",
        ] {
            for (ignore_case, multiline, dot_all) in [
                (false, false, false),
                (true, true, false),
                (false, false, true),
                (true, false, true),
            ] {
                let matcher = ordinary(source, ignore_case, multiline, dot_all);
                for text in [
                    "",
                    "a",
                    "c",
                    "abc",
                    "ababc",
                    "ababa",
                    "ababb",
                    "ababcdde",
                    "ababqc",
                    "a1a2c",
                    "µΜµΜa",
                    "\nababc",
                    "a\na\nb",
                    "ababc ababc",
                    "ababab",
                ] {
                    let input = JsString::from(text);
                    for (start, sticky) in [(0, false), (0, true), (1, true)] {
                        writeln!(rows,"{source:?} i={ignore_case} m={multiline} s={dot_all} input={input:?} start={start} sticky={sticky} {:?}",matcher.find(&input,start,sticky)).unwrap();
                    }
                }
            }
        }
        insta::assert_snapshot!(rows);
    }

    #[test]
    fn fixed_lookbehind_sequences_preserve_order_boundaries_and_unrepeated_captures() {
        for (source, text, range, captures) in [
            (r"(?<=((?:ab){2}))c", "ababc", 4..5, vec![Some(0..4)]),
            (r"(?<=((?:ab){2}))\1", "abababab", 4..8, vec![Some(0..4)]),
            (r"(?<!((?:ab){2})q)c", "ababrc", 5..6, vec![None]),
            (
                r"(?<=((?:ab){2})(?<!(ba)))c",
                "ababc",
                4..5,
                vec![Some(0..4), None],
            ),
            (r"(?<=^(?:ab){2})c", "\nababc", 5..6, vec![]),
            (r"(?<=((?:ab){0}))c", "c", 0..1, vec![Some(0..0)]),
        ] {
            let found = ordinary(source, false, true, false)
                .find(&JsString::from(text), 0, false)
                .unwrap();
            assert_eq!(found.range, range, "{source}");
            assert_eq!(&*found.captures, &*captures, "{source}");
        }
        for source in [
            r"(?<=(?:ab){1,2})c",
            r"(?<=(ab){1,2})c",
            r"(?<=(?:a(?=a)){2})c",
            r"(?<=(?:a(?<=a)){2})c",
            r"(a)(?<=(?:\1b){1,2})c",
            r"(?<=(?:(?:ab){2}){2})c",
        ] {
            assert!(
                RegExpBackreferenceMatcher::compile_ordinary_with_named_bindings_and_work(
                    &JsString::from(source),
                    false,
                    false,
                    false,
                    RegExpBackreferenceNamedBindings::default(),
                    |_| Ok::<_, ()>(())
                )
                .unwrap()
                .is_none(),
                "{source}"
            );
        }
    }

    #[test]
    fn deep_fixed_lookbehind_sequences_large_counts_overflow_and_work_stay_compact() {
        let source =
            "(?<=".to_owned() + &"(?:".repeat(100000) + "(?:ab){2}" + &")".repeat(100000) + ")c";
        let matcher = ordinary(&source, false, false, false);
        let copy = matcher.clone();
        drop(matcher);
        assert_eq!(
            copy.find(&JsString::from("ababc"), 4, true).unwrap().range,
            4..5
        );
        let source = format!("(?<=(?:ab){{{}}})c", usize::MAX / 2);
        let matcher = ordinary(&source, false, false, false);
        let mut work = 0;
        assert!(
            matcher
                .find_with_work(&JsString::from("abc"), 2, true, |n| {
                    work += n;
                    Ok::<_, ()>(())
                })
                .unwrap()
                .is_none()
        );
        assert!(work < 100);
        let source = format!("(?<=(?:ab){{{}}})c", usize::MAX / 2 + 1);
        assert!(
            RegExpBackreferenceMatcher::compile_ordinary_with_named_bindings_and_work(
                &JsString::from(source.as_str()),
                false,
                false,
                false,
                RegExpBackreferenceNamedBindings::default(),
                |_| Ok::<_, ()>(())
            )
            .unwrap()
            .is_none()
        );
        let source = "(?<=((?:ab){0}))c";
        assert_eq!(
            ordinary(source, false, false, false)
                .find(&JsString::from("c"), 0, true)
                .unwrap()
                .captures[0],
            Some(0..0)
        );
        let matcher = ordinary(r"(?<=(?:ab){1000})c", false, false, false);
        let mut work = 0;
        assert_eq!(
            matcher
                .find_with_work(
                    &JsString::from(("ab".repeat(1000) + "c").as_str()),
                    2000,
                    true,
                    |n| {
                        work += n;
                        if work > 1000 {
                            Err("explicit sequence work")
                        } else {
                            Ok(())
                        }
                    }
                )
                .unwrap_err(),
            "explicit sequence work"
        );
    }

    #[test]
    fn fixed_lookbehind_capture_execution_snapshot() {
        let mut rows = String::new();
        for source in [
            r"(?<=(a))b",
            r"(?<!(a))b",
            r"(?<=(a))\1",
            r"(?<=(ab))c",
            r"(?<=((a)b))c",
            r"(?<=(a(b)))c",
            r"(?<=(a{2}))b",
            r"(?<=([a-c]{2}))d",
            r"(?<=()a)b",
            r"(?<=a())b",
            r"(?<=(\b)a)b",
            r"(?<=(^a))b",
            r"(?<=a(?<=(a)))b",
            r"(?<=((?<=a)b))c",
            r"(?<=a(?<!(b)))b",
            r"(?<!((a)b))c",
            r"(?<=(a)(?<!(b)))b",
            r"(?<=((a)(?<!(b))))b",
            r"(a)(?<=(a))\1\2",
            r"(?<=(a))(b)\1\2",
            r"(?:(?<=(a))b|c)+d",
            r"(?:(?<=(a))b|(?<=(b))a)+c",
            r"(?:(?<=(a))|()){2}b\1\2",
            r"((?<=(a))){2}b\1\2",
            r"((?<=(a)))*b\1\2",
            r"(?=(a(?<=(a))b))\1\2",
            r"(?<=(µ))Μ",
            r"(?<=([^\n]))b",
        ] {
            for (ignore_case, multiline, dot_all) in [
                (false, false, false),
                (true, true, false),
                (false, false, true),
                (true, false, true),
            ] {
                let matcher = ordinary(source, ignore_case, multiline, dot_all);
                for text in [
                    "", "a", "b", "ab", "qb", "abb", "abc", "abab", "abccd", "aab", "aba", "\nab",
                    "ab\n", "µΜ", "aaa",
                ] {
                    let input = JsString::from(text);
                    for (start, sticky) in [(0, false), (0, true), (1, true)] {
                        writeln!(rows,"{source:?} i={ignore_case} m={multiline} s={dot_all} input={input:?} start={start} sticky={sticky} {:?}",matcher.find(&input,start,sticky)).unwrap();
                    }
                }
            }
        }
        insta::assert_snapshot!(rows);
    }

    #[test]
    fn fixed_lookbehind_captures_keep_before_match_ranges_and_restore_negative_children() {
        for (source, text, range, captures) in [
            (r"(?<=(a))b", "ab", 1..2, vec![Some(0..1)]),
            (r"(?<=(a))\1", "aa", 1..2, vec![Some(0..1)]),
            (r"(?<=((a)b))c", "abc", 2..3, vec![Some(0..2), Some(0..1)]),
            (r"(?<=(a(b)))c", "abc", 2..3, vec![Some(0..2), Some(1..2)]),
            (r"(?<=a(?<=(a)))b", "ab", 1..2, vec![Some(0..1)]),
            (r"(?<=a(?<!(b)))b", "ab", 1..2, vec![None]),
            (r"(?<=(a)(?<!(b)))b", "ab", 1..2, vec![Some(0..1), None]),
            (r"(?<!((a)b))c", "aac", 2..3, vec![None, None]),
            (
                r"(?<=(aa)(?<!((a)b)))c",
                "aac",
                2..3,
                vec![Some(0..2), None, None],
            ),
            (r"(?:(?<=(a))b|c)+d", "abcd", 1..4, vec![None]),
            (
                r"((?<=(a))){2}b\1\2",
                "aba",
                1..3,
                vec![Some(1..1), Some(0..1)],
            ),
        ] {
            let found = ordinary(source, false, false, false)
                .find(&JsString::from(text), 0, false)
                .unwrap();
            assert_eq!(found.range, range, "{source}");
            assert_eq!(&*found.captures, &*captures, "{source}");
        }
        for source in [
            r"(?<=(a)+)b",
            r"(?<=(a|bb))c",
            r"(?<=(a)\1)b",
            r"(?<=(a(?=a+)))b",
        ] {
            assert!(
                RegExpBackreferenceMatcher::compile_ordinary_with_named_bindings_and_work(
                    &JsString::from(source),
                    false,
                    false,
                    false,
                    RegExpBackreferenceNamedBindings::default(),
                    |_| Ok::<_, ()>(()),
                )
                .unwrap()
                .is_none(),
                "{source}"
            );
        }
    }

    #[test]
    fn deep_fixed_lookbehind_capture_ranges_checkpoints_clones_and_work_are_flat() {
        let source = "(?<=".to_owned() + &"(".repeat(100000) + "a" + &")".repeat(100000) + ")b";
        let matcher = ordinary(&source, false, false, false);
        let copy = matcher.clone();
        drop(matcher);
        let found = copy.find(&JsString::from("ab"), 1, true).unwrap();
        assert_eq!(found.range, 1..2);
        assert_eq!(found.captures.len(), 100000);
        assert!(found.captures.iter().all(|v| v == &Some(0..1)));
        let mut work = 0;
        assert_eq!(
            copy.find_with_work(&JsString::from("ab"), 1, true, |n| {
                work += n;
                if work > 1000 {
                    Err("explicit capture work")
                } else {
                    Ok(())
                }
            })
            .unwrap_err(),
            "explicit capture work"
        );
        let source = "(?<!".to_owned() + &"(".repeat(100000) + "a" + &")".repeat(100000) + "b)c";
        let matcher = ordinary(&source, false, false, false);
        let found = matcher.find(&JsString::from("aac"), 2, true).unwrap();
        assert!(found.captures.iter().all(Option::is_none));
        assert_eq!(
            RegExpBackreferenceMatcher::compile_ordinary_with_named_bindings_and_work(
                &JsString::from(source.as_str()),
                false,
                false,
                false,
                RegExpBackreferenceNamedBindings::default(),
                |_| Err("explicit construction work"),
            )
            .unwrap_err(),
            "explicit construction work"
        );
    }

    #[test]
    fn nested_fixed_lookbehind_execution_snapshot() {
        let mut rows = String::new();
        for source in [
            r"(?<=(?<=a))b",
            r"(?<!(?<=a))b",
            r"(?<=(?<!a))b",
            r"(?<!(?<!a))b",
            r"(?<=a(?<=a))b",
            r"(?<=a(?<!a))b",
            r"(?<!a(?<=a))b",
            r"(?<=a(?<=b))c",
            r"(?<=(?<=a)b)c",
            r"(?<=a(?<=a)b)c",
            r"(?<=a(?<!b)b)c",
            r"(?<=a(?<=a{1})b{1})c",
            r"(?<=a{2}(?<=a{2}))b",
            r"(?<=a{2}(?<!a{2}))b",
            r"(?<=(?<=^a)b)c",
            r"(?<=(?<=\ba)b)c",
            r"(?<=c(?<=\w))\w{3}",
            r"(?<=\B)(?<=c(?<=\w))\w{3}",
            r"(?<=.(?<=.))b",
            r"(?<=µ(?<=µ))Μ",
            r"(?<=a(?<=(?<=a)))b",
            r"(?<=a(?<!(?<!a)))b",
            r"(?<=a(?<=a$))",
            r"(?<=^a(?<=a))b",
            r"(?<=a(?<=a))(b)\1",
            r"(a)(?<=a(?<=a))\1",
            r"(?:(?<=a(?<=a))b|c)+d",
            r"(?:(?<=a(?<=a))|(?<!b)){2}b",
        ] {
            for (ignore_case, multiline, dot_all) in [
                (false, false, false),
                (true, true, false),
                (false, false, true),
                (true, false, true),
            ] {
                let matcher = ordinary(source, ignore_case, multiline, dot_all);
                for text in [
                    "", "a", "b", "ab", "qb", "abb", "abc", "qabc", "abccd", "aab", "\nab", "ab\n",
                    "ab cdef", "µΜ", "\nb",
                ] {
                    let input = JsString::from(text);
                    for (start, sticky) in [(0, false), (0, true), (1, true)] {
                        writeln!(rows,"{source:?} i={ignore_case} m={multiline} s={dot_all} input={input:?} start={start} sticky={sticky} {:?}",matcher.find(&input,start,sticky)).unwrap();
                    }
                }
            }
        }
        insta::assert_snapshot!(rows);
    }

    #[test]
    fn nested_fixed_lookbehind_negation_and_child_positions_keep_whole_input_context() {
        for (source, text, range, captures) in [
            (r"(?<=(?<=a))b", "ab", 1..2, vec![]),
            (r"(?<!(?<=a))b", "qb", 1..2, vec![]),
            (r"(?<=(?<!a))b", "qb", 1..2, vec![]),
            (r"(?<!(?<!a))b", "ab", 1..2, vec![]),
            (r"(?<=(?<=a)b)c", "abc", 2..3, vec![]),
            (r"(?<=a(?<=a)b)c", "abc", 2..3, vec![]),
            (r"(?<=a(?<!b)b)c", "abc", 2..3, vec![]),
            (r"(?<=\B)(?<=c(?<=\w))\w{3}", "ab cdef", 4..7, vec![]),
            (r"(?<=a(?<=a))(b)\1", "abb", 1..3, vec![Some(1..2)]),
            (r"((?<=a(?<=a))){2}b\1", "ab", 1..2, vec![Some(1..1)]),
        ] {
            let found = ordinary(source, false, false, false)
                .find(&JsString::from(text), 0, false)
                .unwrap();
            assert_eq!(found.range, range, "{source}");
            assert_eq!(&*found.captures, &*captures, "{source}");
        }
        assert!(
            ordinary(r"(?<=a(?<!a))b", false, false, false)
                .find(&JsString::from("ab"), 0, false)
                .is_none()
        );
        assert_eq!(
            ordinary(r"(?<=(?<=^a)b)c", false, true, false)
                .find(&JsString::from("q\nabc"), 0, false)
                .unwrap()
                .range,
            4..5
        );
        for source in [
            r"(?<=a(?=(b+)))b",
            r"(?<=(?<=a|bb))c",
            r"(?<=(?<=(a+)))b",
            r"(?<=a(?=a+))b",
            r"(?<=a(?<=a+))b",
        ] {
            assert!(
                RegExpBackreferenceMatcher::compile_ordinary_with_named_bindings_and_work(
                    &JsString::from(source),
                    false,
                    false,
                    false,
                    RegExpBackreferenceNamedBindings::default(),
                    |_| Ok::<_, ()>(()),
                )
                .unwrap()
                .is_none(),
                "{source}"
            );
        }
    }

    #[test]
    fn deep_nested_fixed_lookbehind_proofs_execution_clones_and_work_stay_flat() {
        let source = "(?<=".repeat(100000) + "a" + &")".repeat(100000) + "b";
        let matcher = ordinary(&source, false, false, false);
        let copy = matcher.clone();
        drop(matcher);
        assert_eq!(
            copy.find(&JsString::from("ab"), 1, true).unwrap().range,
            1..2
        );
        let source = "(?<!".repeat(100001) + "a" + &")".repeat(100001) + "b";
        let matcher = ordinary(&source, false, false, false);
        assert!(matcher.find(&JsString::from("ab"), 1, true).is_none());
        assert_eq!(
            matcher.find(&JsString::from("qb"), 1, true).unwrap().range,
            1..2
        );
        let mut work = 0;
        assert_eq!(
            matcher
                .find_with_work(&JsString::from("qb"), 1, true, |n| {
                    work += n;
                    if work > 1000 {
                        Err("explicit nested assertion work")
                    } else {
                        Ok(())
                    }
                })
                .unwrap_err(),
            "explicit nested assertion work"
        );
        assert_eq!(
            RegExpBackreferenceMatcher::compile_ordinary_with_named_bindings_and_work(
                &JsString::from(source.as_str()),
                false,
                false,
                false,
                RegExpBackreferenceNamedBindings::default(),
                |_| Err("explicit construction work"),
            )
            .unwrap_err(),
            "explicit construction work"
        );
    }

    #[test]
    fn fixed_count_lookbehind_execution_snapshot() {
        let mut rows = String::new();
        for source in [
            r"(?<=a{1})b",
            r"(?<=a{2})b",
            r"(?<!a{2})b",
            r"(?<=a{0})b",
            r"(?<!a{0})b",
            r"(?<=a{2,2})b",
            r"(?<=a{2}?)b",
            r"(?<!a{2}?)b",
            r"(?<=a{1}b{1})c",
            r"(?<=a[a-z]{2})d",
            r"(?<!a[a-z]{2})d",
            r"(?<=.{2})b",
            r"(?<=\w{2})b",
            r"(?<=\s{1})b",
            r"(?<=[^a]{2})b",
            r"(?<=\b[a-z]{2})c",
            r"(?<=a{2}\B)b",
            r"(?<=^a{2})b",
            r"(?<=a{2}$)",
            r"(?<=µ{2})Μ",
            r"(?<=(?:a){2})b",
            r"(?<=a{0}b)c",
            r"(?<=a{1})(b)\1",
            r"(a)(?<=a{1})\1",
            r"(?:(?<=a{2})b|c)+d",
            r"(?:(?<=a{2})|(?<!b{2})){2}b",
            r"((?<=a{2})){2}b\1",
            r"(?:(?<=a{2}))*b",
        ] {
            for (ignore_case, multiline, dot_all) in [
                (false, false, false),
                (true, true, false),
                (false, false, true),
                (true, false, true),
            ] {
                let matcher = ordinary(source, ignore_case, multiline, dot_all);
                for text in [
                    "", "a", "b", "ab", "aab", "aabb", "abcd", "qabc", "aabcd", "\naab", "a\nb",
                    "aa\n", " b ", "µµΜ", "\r\nb",
                ] {
                    let input = JsString::from(text);
                    for (start, sticky) in [(0, false), (0, true), (1, true)] {
                        writeln!(rows,"{source:?} i={ignore_case} m={multiline} s={dot_all} input={input:?} start={start} sticky={sticky} {:?}",matcher.find(&input,start,sticky)).unwrap();
                    }
                }
            }
        }
        insta::assert_snapshot!(rows);
    }

    #[test]
    fn fixed_count_lookbehind_zero_counts_boundaries_and_outside_captures_are_exact() {
        for (source, text, range, captures) in [
            (r"(?<=a{2})(b)\1", "aabb", 2..4, vec![Some(2..3)]),
            (r"(a)(?<=a{1})\1", "aa", 0..2, vec![Some(0..1)]),
            (r"((?<=a{2})){2}b\1", "aab", 2..3, vec![Some(2..2)]),
            (r"(?<=a{0}b)c", "bc", 1..2, vec![]),
            (r"(?<=\b[a-z]{2})c", " abc", 3..4, vec![]),
            (r"(?<=a{2}\B)b", "aab", 2..3, vec![]),
            (r"(?<=(?:a){2})b", "aab", 2..3, vec![]),
        ] {
            let found = ordinary(source, false, false, false)
                .find(&JsString::from(text), 0, false)
                .unwrap();
            assert_eq!(found.range, range, "{source}");
            assert_eq!(&*found.captures, &*captures, "{source}");
        }
        assert!(
            ordinary(r"(?<!a{0})b", false, false, false)
                .find(&JsString::from("b"), 0, false)
                .is_none()
        );
        assert_eq!(
            ordinary(r"(?<=^a{2})b", false, true, false)
                .find(&JsString::from("q\naab"), 0, false)
                .unwrap()
                .range,
            4..5
        );
        for source in [
            r"(?<=(a{1,2}))b",
            r"(?<=a{1,2})b",
            r"(?<=a+)b",
            r"(?<=(?:ab){1,2})c",
            r"(?<=(?:(?=a+)){2})a",
        ] {
            assert!(
                RegExpBackreferenceMatcher::compile_ordinary_with_named_bindings_and_work(
                    &JsString::from(source),
                    false,
                    false,
                    false,
                    RegExpBackreferenceNamedBindings::default(),
                    |_| Ok::<_, ()>(()),
                )
                .unwrap()
                .is_none(),
                "{source}"
            );
        }
    }

    #[test]
    fn fixed_count_lookbehind_counts_stay_compact_and_actual_unit_comparisons_are_fallible() {
        let source = JsString::from(r"(?<=a{10000})b");
        let matcher = ordinary(r"(?<=a{10000})b", false, false, false);
        assert!(matcher.0.instructions.len() < 20);
        let text = JsString::from(("a".repeat(10000) + "b").as_str());
        assert_eq!(
            matcher.find(&text, 10000, true).unwrap().range,
            10000..10001
        );
        let mut work = 0;
        assert_eq!(
            matcher
                .find_with_work(&text, 10000, true, |n| {
                    work += n;
                    if work > 1000 {
                        Err("explicit unit work")
                    } else {
                        Ok(())
                    }
                })
                .unwrap_err(),
            "explicit unit work"
        );
        assert_eq!(
            RegExpBackreferenceMatcher::compile_ordinary_with_named_bindings_and_work(
                &source,
                false,
                false,
                false,
                RegExpBackreferenceNamedBindings::default(),
                |_| Err("explicit construction work"),
            )
            .unwrap_err(),
            "explicit construction work"
        );
        let huge = ordinary(&format!("(?<=a{{{}}})b", usize::MAX), false, false, false);
        assert!(huge.find(&JsString::from("ab"), 0, false).is_none());
        let source =
            "(?<=".to_owned() + &"(?:".repeat(100000) + "a{2}" + &")".repeat(100000) + ")b";
        let matcher = ordinary(&source, false, false, false);
        let copy = matcher.clone();
        drop(matcher);
        assert_eq!(
            copy.find(&JsString::from("aab"), 2, true).unwrap().range,
            2..3
        );
    }

    #[test]
    fn fixed_lookbehind_execution_snapshot() {
        let mut rows = String::new();
        for source in [
            r"(?<=a)b",
            r"(?<!a)b",
            r"(?<=ab)c",
            r"(?<!ab)c",
            r"(?<=)a",
            r"(?<!)a",
            r"(?<=^a)b",
            r"(?<!^a)b",
            r"(?<=a$)",
            r"(?<!a$)",
            r"(?<=\ba)b",
            r"(?<=a\b)b",
            r"(?<=\Ba)b",
            r"(?<=a\B)b",
            r"(?<=[a-c])b",
            r"(?<=[^a])b",
            r"(?<=.)b",
            r"(?<=[\s\S])b",
            r"(?<=µ)Μ",
            r"(?<=\w)a",
            r"(?<=a(?:b))c",
            r"a(?<=a)b",
            r"(?<=a)(b)\1",
            r"(a)(?<=a)\1",
            r"(?:(?<=a)b|c)+d",
            r"(?:(?<=a)|(?<!b)){2}b",
            r"((?<=a)){2}b\1",
            r"(?:(?<=a))*b",
        ] {
            for (ignore_case, multiline, dot_all) in [
                (false, false, false),
                (true, true, false),
                (false, false, true),
                (true, false, true),
            ] {
                let matcher = ordinary(source, ignore_case, multiline, dot_all);
                for text in [
                    "",
                    "a",
                    "b",
                    "ab",
                    "abb",
                    "abc",
                    "qabc",
                    "ababd",
                    "\nab",
                    "a\nb",
                    "ab\n",
                    " b ",
                    "µΜ",
                    "\u{2028}b",
                    "\r\n",
                ] {
                    let input = JsString::from(text);
                    for (start, sticky) in [(0, false), (0, true), (1, true)] {
                        writeln!(rows,"{source:?} i={ignore_case} m={multiline} s={dot_all} input={input:?} start={start} sticky={sticky} {:?}",matcher.find(&input,start,sticky)).unwrap();
                    }
                }
            }
        }
        insta::assert_snapshot!(rows);
    }

    #[test]
    fn fixed_lookbehind_preserves_input_context_and_outside_capture_ranges() {
        for (source, text, range, captures) in [
            (r"(?<=a)(b)\1", "abb", 1..3, vec![Some(1..2)]),
            (r"(a)(?<=a)\1", "aa", 0..2, vec![Some(0..1)]),
            (r"((?<=a)){2}b\1", "ab", 1..2, vec![Some(1..1)]),
            (r"(?:(?<=a)b|c)+d", "abcd", 1..4, vec![]),
            (r"(?<=a\B)b", "ab", 1..2, vec![]),
            (r"(?<=\ba)b", " ab", 2..3, vec![]),
            (r"(?<=a$)", "a", 1..1, vec![]),
        ] {
            let found = ordinary(source, false, false, false)
                .find(&JsString::from(text), 0, false)
                .unwrap();
            assert_eq!(found.range, range, "{source}");
            assert_eq!(&*found.captures, &*captures, "{source}");
        }
        assert!(
            ordinary(r"(?<=a\b)b", false, false, false)
                .find(&JsString::from("ab"), 0, false)
                .is_none()
        );
        assert!(
            ordinary(r"(?<=^a)b", false, false, false)
                .find(&JsString::from("qab"), 0, false)
                .is_none()
        );
        assert_eq!(
            ordinary(r"(?<=^a)b", false, true, false)
                .find(&JsString::from("q\nab"), 0, false)
                .unwrap()
                .range,
            3..4
        );
        for source in [
            r"(?<=(a+))b",
            r"(?<=a|bb)c",
            r"(?<=a+)b",
            r"(?<=a{1,2})b",
            r"(?<=(\1))a",
            r"(?<=(?=a+))a",
        ] {
            assert!(
                RegExpBackreferenceMatcher::compile_ordinary_with_named_bindings_and_work(
                    &JsString::from(source),
                    false,
                    false,
                    false,
                    RegExpBackreferenceNamedBindings::default(),
                    |_| Ok::<_, ()>(()),
                )
                .unwrap()
                .is_none(),
                "{source}"
            );
        }
    }

    #[test]
    fn fixed_lookbehind_flat_wrappers_long_prefixes_and_actual_search_work_are_fallible() {
        let source = format!(
            "(?<={}{})b",
            "(?:".repeat(100000),
            "a".to_owned() + &")".repeat(100000)
        );
        let matcher = ordinary(&source, false, false, false);
        assert_eq!(
            matcher.find(&JsString::from("ab"), 0, false).unwrap().range,
            1..2
        );
        let copy = matcher.clone();
        drop(matcher);
        assert_eq!(
            copy.find(&JsString::from("ab"), 1, true).unwrap().range,
            1..2
        );
        let source = "(?<=".to_owned() + &"a".repeat(10000) + ")b";
        let matcher = ordinary(&source, false, false, false);
        let text = JsString::from(("a".repeat(10000) + "b").as_str());
        assert_eq!(
            matcher.find(&text, 10000, true).unwrap().range,
            10000..10001
        );
        let mut work = 0usize;
        assert_eq!(
            matcher
                .find_with_work(&text, 10000, true, |n| {
                    work += n;
                    if work > 1000 {
                        Err("explicit search work")
                    } else {
                        Ok(())
                    }
                })
                .unwrap_err(),
            "explicit search work"
        );
        let source = JsString::from(source.as_str());
        assert_eq!(
            RegExpBackreferenceMatcher::compile_ordinary_with_named_bindings_and_work(
                &source,
                false,
                false,
                false,
                RegExpBackreferenceNamedBindings::default(),
                |_| Err("explicit construction work"),
            )
            .unwrap_err(),
            "explicit construction work"
        );
    }

    #[test]
    fn repeated_branch_capture_execution_snapshot() {
        let mut rows = String::new();
        for source in [
            r"(a|ab)+b",
            r"(ab|a)+b",
            r"(a|ab)+?b",
            r"(ab|a)+?b",
            r"((a)|(b))+c",
            r"((a)|(b))+?c",
            r"(?:(a)|b)+c",
            r"(?:(a)|b)*c",
            r"(a|b){2,3}c",
            r"(a|b){2,3}?c",
            r"((a)+b|c)+d",
            r"((a)+?b|c)+?d",
            r"(a)(?:(b)\1|c)+\2",
            r"(a)(?:(b)\1|c)+?\2",
            r"((a|ab)+)+b",
            r"((a|ab)+?)+?b",
            r"(a|(b))+(c)\2",
            r"(a|\1b)+c",
            r"((\2a)|b)+c",
            r"((^a)|(b$))+",
            r"([ab]|c)+d",
            r"(.|a)+b",
            r"((a|b){2,3})+c",
            r"((a|b)*)c|a",
            r"((a|b)+){999999999999999999999999999999}",
            r"(?:(a)|b)+c|(d)\1",
            r"(?:(a)(b)((\1)\4\2)+){2}",
            r"(?:(a)(b)(\1(\2)\4)+){2}",
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
    fn repeated_branch_capture_resets_and_undo_restore_last_iteration_slots() {
        for (source, text, range, captures) in [
            (r"(a|ab)+b", "abb", 0..2, vec![Some(0..1)]),
            (r"(ab|a)+b", "abb", 0..3, vec![Some(0..2)]),
            (
                r"((a)|(b))+c",
                "abc",
                0..3,
                vec![Some(1..2), None, Some(1..2)],
            ),
            (
                r"(a|(b))+(c)\2",
                "abac",
                0..4,
                vec![Some(2..3), None, Some(3..4)],
            ),
            (r"((a|ab)+)+b", "abb", 0..2, vec![Some(0..1), Some(0..1)]),
            (r"(?:(a)|b)+c", "abc", 0..3, vec![None]),
            (r"(a|\1b)+c", "abac", 0..4, vec![Some(2..3)]),
        ] {
            let found = ordinary(source, false, false, false)
                .find(&JsString::from(text), 0, true)
                .unwrap();
            assert_eq!(found.range, range, "{source}");
            assert_eq!(&*found.captures, &*captures, "{source}");
        }
    }

    #[test]
    fn repeated_branch_capture_inventories_keep_deep_ranges_compact_and_fallible() {
        let source = "(".repeat(100000) + "a|b" + &")".repeat(100000) + "+";
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
        let found = matcher.find(&JsString::from("ab"), 0, true).unwrap();
        assert_eq!(found.range, 0..2);
        assert!(found.captures.iter().all(|r| *r == Some(1..2)));
        assert!(
            matcher
                .find_with_work(&JsString::from("ab"), 0, true, |_| Err::<(), _>("work"))
                .is_err()
        );
        let source = "(".repeat(1000) + "(?:a|b)" + &")+".repeat(1000);
        let found = ordinary(&source, false, false, false)
            .find(&JsString::from("a"), 0, true)
            .unwrap();
        assert!(found.captures.iter().all(|r| *r == Some(0..1)));
        for source in [r"(a|)+", r"((a|b)*)+", r"((?<=(a+))a|b)+"] {
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
            r"(?:(?:(?:(?:(a)|b)+c|d)+)|)*",
            r"(?:(?:\1|b)+c|d)+(a)",
            r"(?:(?<=(a+))a|b)+",
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
    fn deterministic_inner_quantifiers_keep_preparation_flat_and_reject_empty_paths() {
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
            r"(?:(?:(?:(a+)b|c)+)|)*",
            r"(?:(?:(?:(?:(a)|b)+c|d)+)|)*",
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
            r"(?:(?:(?:(a|)b)+)|)*",
            r"(?:(?:(?:(?:a|)(b+))+)|)*",
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
            r"(?:(?:((a)|(b))+)|)*",
            r"(?:(?:(?:(?:(ab)|c)+){2})|)*",
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
        for source in [
            r"(?:(?:(a|b)+)|)*",
            r"(?:(?:(a+)+)|)*",
            r"(?<=(a+))a",
            r"(?i:a)",
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
    fn invalid_named_bindings_and_empty_repeated_bodies_return_none_before_charging() {
        for (text, slots, escape, group) in [
            (r"(a)\k<x>", vec![0], 3..8, 1),
            (r"(a)\k<x>", vec![1], 3..8, 0),
            (r"(a)\k<x>", vec![], 3..8, 0),
            (r"(a)\k<x>", vec![0, 0], 3..8, 0),
            (r"(a)(b)\k<x>", vec![0, 1], 6..11, 0),
            (r"(a)\k<x>", vec![0], 3..9, 0),
            (r"(a)\k<x>", vec![0], 2..7, 0),
            (r"(a)[\k<x>]", vec![0], 4..9, 0),
            (r"(?:(a)(?:\k<x>)+|){2}", vec![0], 9..14, 0),
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
        for source in [
            r"(?:(?:((a+)+)\1)|)*",
            r"(?:(?:(a|b)+\1)|)*",
            r"((?<=(a+))a+)\1",
            r"((?i:a)+)\1",
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
        for source in [
            r"(?:(?:(a|b)+\1)|)*",
            r"(?:(?:((a)+)+\1)|)*",
            r"(?:(?:(?:(a+)\1){2})|)*",
            r"((?<=(a+))a)+\1",
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
            r"(?:(?:(a)(b)(a\1+\2)+)|)*",
            r"(a)(b)((?=a)\1\2)+",
            r"(?:(?:(?:(a)(b)(\b\1\2)+){2})|)*",
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
            r"(?:(?:(?:(a)(b)(a(\1)\4\2)+){2})|)*",
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
            r"(?:(?:(?:(a)(b)((\1)\4\2)+){2})|)*",
            r"(?:(?:(?:(a)((\1)\3)+){2})|)*",
            r"(?:(?:(?:(a)(b)(\1(\2)\4)+){2})|)*",
            r"(?:(?:(?:(a)(b)(a(\1)\2)+){2})|)*",
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
            (r"(?:(?:(?:(a)(b)((\1)\k<x>\2)+){2})|)*", &[3][..]),
            (r"(?:(?:(?:(a)(b)(\1(\2)\k<x>)+){2})|)*", &[3][..]),
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
            r"(?:(?:(?:(a)(b)((\1)\2)+){2})|)*",
            r"(?:(?:(?:(a)(b)(a\1()\2)+){2})|)*",
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
            r"(?:(?:(?:(a)(b)((\1)\2)+){2})|)*",
            r"(?:(?:(?:(a)(b)(a\1()\2)+){2})|)*",
            r"(?:(?:(?:(a)(b)(a\1\2)+){2})|)*",
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
            r"(?:(?:(?:(a)(b)(\1\2)+){2})|)*",
            r"(?:(?:(?:(a)(b)(?:()\1\2)+){2})|)*",
            r"(?:(?:(?:(a)(b)(?:a\1\2)+){2})|)*",
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
            r"(?:(?:(?:(a)(b)(?:\1\2)+){2})|)*",
            r"(?:(?:(?:(a)(?:a\1\1)+){2})|)*",
            r"(a)(?:\1|\1)+",
            r"(a)(?:\1+\1)+",
            r"(?:(?:(?:(a)(?:a\1()\1)+){2})|)*",
            r"(?:(?:(?:(a)(?:(\1)\1)+){2})|)*",
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
            r"(?:(?:(?:(a)(()a\1())+){2})|)*",
            r"(?:(?:(?:(a)(()a\1\1())+){2})|)*",
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
            r"(?:(?:(?:(a)(a\1\1)+){2})|)*",
            r"(?:(?:(?:(a)(a\1)+){2})|)*",
            r"(a)(\1|b)+",
            r"(a)(\1*)+",
            r"(?:(?:(?:(a)(()a\1)+){2})|)*",
            r"(?:(?:(?:(a)(\1a())+){2})|)*",
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
            r"(?:(?:(?:(a)(a\1)+){2})|)*",
            r"(?:(?:(?:(a)(?:a\1\1)+){2})|)*",
            r"(?:(?:(?:(a)(?:a\1)+){2})|)*",
            r"(a)(?:\1|b)+",
            r"(a)(?:\1*)+",
            r"(a)(?:\1(?=a))+",
            r"(?:(?:(?:(a)(?:a(\1))+){2})|)*",
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
            r"(?:(?:(?:(?:(a)\1){2}){2})|)*",
            r"(?:(?:(?:(a)(?:\1)+){2})|)*",
            r"(?:(?:(?:(a)+\1){2})|)*",
            r"(?<=(a+))\1+",
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
            r"(?:(?:(?:(?:(a|b)|c)(?:\1)+){2})|)*",
            r"(?:(?:(?:(?:a|(b|c))(?:\1)+){2})|)*",
            r"(?:(?:(?:((a|b)(c|d)|e)(?:\1)+){2})|)*",
            r"(?:(?:(?:(a|b)(c|d)(?:\1)+){2})|)*",
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
            r"(?:(?:(?:(\w)(?:\1)+){2})|)*",
            r"(?:(?:(?:^([ab])(?:\1)+){2})|)*",
            r"(?:(?:(?:([ab]|c)(?:\1)+){2})|)*",
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
    fn zero_width_lookahead_branch_execution_snapshot() {
        let mut rows = String::new();
        for source in [
            r"(?:(?=(a))|()){2}\1\2",
            r"(?:(?=(a))|())*\1\2",
            r"(?:(?=(a))|()){2,4}?\1\2",
            r"((?=(a))|()){2}\1\2\3",
            r"((?=(a))|()){0,2}\1\2\3",
            r"(?:(?=(a))|(?=(b))){2}\1\2",
            r"(?:(?=(a|ab))|(?=(ab))){2}\1b$",
            r"(?:(?=(ab|a))|(?=(a))){2}\1b$",
            r"(?:(?=(a))|(?=(ab))){2}\1\2b$",
            r"(?:(?!(a)b)|(?=(a))){2}\1\2a",
            r"(?:(?!(a)b)|(?!(b)a)){2}a",
            r"(?:(?=(a))|^){2}\1",
            r"(?:(?=(a))|$){2}\1",
            r"(?:(\b)(?=(a))|()){2}a",
            r"(?:(?=(a))()|(?=(b))()){2}\1\2\3\4",
            r"(?:(?=(a))|(?!(b))){2}a",
            r"(?:(?!(a))|(?=(b))){2}b",
            r"(?:(?:(?=(a))|()){2}){3}\1",
            r"(?:(?:(?=(a))|())?){2}a",
            r"(?:(?:(?=(a))|()){2}a|b)+c",
            r"(?:(?:(?=(a))|())?a|b)+c",
            r"(?:(?=(\uD800))|()){2}\1",
            r"(?:(?=(^a))|()){2}\1",
            r"(?:(?=(a|b)+)|()){2}\1",
            r"(?:(?=(a))|(?=(b))){0}c",
            r"(?:(?=(a))|(?=(b))){1}a|b",
            r"(?:(?=)|()){2}a",
            r"(?:(?!)|()){2}a",
        ] {
            for (ignore_case, multiline, dot_all) in [
                (false, false, false),
                (true, false, false),
                (false, true, true),
            ] {
                let matcher =
                    RegExpBackreferenceMatcher::compile_ordinary_with_named_bindings_and_work(
                        &JsString::from(source),
                        ignore_case,
                        multiline,
                        dot_all,
                        RegExpBackreferenceNamedBindings::default(),
                        |_| Ok::<_, ()>(()),
                    )
                    .unwrap()
                    .unwrap();
                for text in [
                    "", "a", "b", "ab", "abb", "aab", "aba", "baabac", "acbc", "abc", "AA", "q\na",
                    "a\n", "\u{d7ff}", "😀",
                ] {
                    let input = JsString::from(text);
                    for (start, sticky) in [(0, false), (0, true), (1, true), (2, false)] {
                        let result = matcher.find(&input, start, sticky);
                        writeln!(rows,"{source:?} i={ignore_case} m={multiline} s={dot_all} input={input:?} start={start} sticky={sticky} {result:?}").unwrap();
                    }
                }
            }
        }
        insta::assert_snapshot!(rows);
    }

    #[test]
    fn zero_width_branch_retries_restore_captures_before_the_outer_continuation() {
        let source = JsString::from(r"(?:(?=(a))|(?=(ab))){2}\1\2b$");
        let matcher = RegExpBackreferenceMatcher::compile(&source, false).unwrap();
        let found = matcher.find(&JsString::from("abb"), 0, true).unwrap();
        assert_eq!(found.range, 0..3);
        assert_eq!(&*found.captures, &[None, Some(0..2)]);
        let source = JsString::from(r"((?=(a))|()){2}\1\2\3");
        let matcher = RegExpBackreferenceMatcher::compile(&source, false).unwrap();
        let found = matcher.find(&JsString::from("b"), 0, true).unwrap();
        assert_eq!(found.range, 0..0);
        assert_eq!(&*found.captures, &[Some(0..0), None, Some(0..0)]);
    }

    #[test]
    fn nested_zero_width_branch_proofs_and_huge_counts_keep_flat_execution() {
        let source = JsString::from(
            format!(
                "{}(?:(?=(a))|()){}\\1",
                "(?:".repeat(100000),
                "){2}".repeat(100000)
            )
            .as_str(),
        );
        let matcher = RegExpBackreferenceMatcher::compile(&source, false).unwrap();
        let clone = matcher.clone();
        drop(matcher);
        assert_eq!(
            clone.find(&JsString::from("a"), 0, true).unwrap().captures[0],
            Some(0..1)
        );
        assert_eq!(
            clone
                .find_with_work(&JsString::from("a"), 0, true, |_| Err::<(), _>(
                    "explicit work"
                ))
                .unwrap_err(),
            "explicit work"
        );
        let source = JsString::from(format!("(?:(?=(a))|()){{{}}}\\1", "9".repeat(10000)).as_str());
        let matcher = RegExpBackreferenceMatcher::compile(&source, false).unwrap();
        let found = matcher.find(&JsString::from("a"), 0, true).unwrap();
        assert_eq!(found.range, 0..1);
        assert_eq!(&*found.captures, &[Some(0..1), None]);
    }

    #[test]
    fn zero_width_lookahead_repetition_execution_snapshot() {
        let mut rows = String::new();
        for source in [
            r"(?:(?=(abc)))a",
            r"(?:(?=(abc)))?a",
            r"(?:(?=(abc))){1,1}a",
            r"(?:(?=(abc))){0,1}a",
            r"((?=(a))){2}\1\2",
            r"((?=(a))){0,2}b",
            r"(?:(?!(a)b)){2}\1a",
            r"(?:(?!(a)b))*\1a",
            r"(?:(?=(a))){2,4}?a",
            r"(?:(?=(a))){4}a",
            r"(?:(?=(a))){0}a",
            r"(?:(?=(a))){1}a|b",
            r"(?:(?=(a|ab))){2}\1b$",
            r"(?:(?=(ab|a))){2}\1b$",
            r"(?:(?=(a+))){2}a*b\1",
            r"(?:(?=(a))^){2}a",
            r"(?:(\b)(?=(a))()){2}a",
            r"(?:(?:(?=(a))){2}){3}\1",
            r"(?:(?:(?=(a)))?){2}a",
            r"(?:(?:(?=(a))){2}a|b)+c",
            r"(?:(?:(?=(a)))?a|b)+c",
            r"(?:(?=(\uD800))){2}\1",
            r"(?:(?=(^a))){2}\1",
            r"(?:(?=(a|b)+)){2}\1",
        ] {
            for (ignore_case, multiline, dot_all) in [
                (false, false, false),
                (true, false, false),
                (false, true, true),
            ] {
                let matcher =
                    RegExpBackreferenceMatcher::compile_ordinary_with_named_bindings_and_work(
                        &JsString::from(source),
                        ignore_case,
                        multiline,
                        dot_all,
                        RegExpBackreferenceNamedBindings::default(),
                        |_| Ok::<_, ()>(()),
                    )
                    .unwrap()
                    .unwrap();
                for text in [
                    "", "a", "b", "ab", "abb", "aab", "aba", "baabac", "acbc", "abc", "AA", "q\na",
                    "a\n", "\u{d7ff}", "😀",
                ] {
                    let input = JsString::from(text);
                    for (start, sticky) in [(0, false), (0, true), (1, true), (2, false)] {
                        let result = matcher.find(&input, start, sticky);
                        writeln!(rows,"{source:?} i={ignore_case} m={multiline} s={dot_all} input={input:?} start={start} sticky={sticky} {result:?}").unwrap();
                    }
                }
            }
        }
        insta::assert_snapshot!(rows);
    }

    #[test]
    fn required_zero_width_lookahead_keeps_captures_while_optional_iterations_skip() {
        for (source, capture) in [
            (r"(?:(?=(abc)))?a", None),
            (r"(?:(?=(abc))){0,1}a", None),
            (r"(?:(?=(abc))){1,1}a", Some(0..3)),
            (r"(?:(?=(abc))){2,4}?a", Some(0..3)),
        ] {
            let matcher =
                RegExpBackreferenceMatcher::compile_ordinary_with_named_bindings_and_work(
                    &JsString::from(source),
                    false,
                    false,
                    false,
                    RegExpBackreferenceNamedBindings::default(),
                    |_| Ok::<_, ()>(()),
                )
                .unwrap()
                .unwrap();
            let found = matcher.find(&JsString::from("abc"), 0, true).unwrap();
            assert_eq!(found.range, 0..1);
            assert_eq!(found.captures[0], capture);
        }
    }

    #[test]
    fn nested_zero_width_lookahead_counts_remain_flat_huge_and_fallible() {
        let source = JsString::from(
            format!(
                "{}(?=(a)){}\\1",
                "(?:".repeat(100000),
                "){2}".repeat(100000)
            )
            .as_str(),
        );
        let matcher = RegExpBackreferenceMatcher::compile(&source, false).unwrap();
        let clone = matcher.clone();
        drop(matcher);
        assert_eq!(
            clone.find(&JsString::from("a"), 0, true).unwrap().captures[0],
            Some(0..1)
        );
        let huge = "9".repeat(10000);
        for (bounds, capture) in [
            (format!("{{{huge}}}"), Some(0..1)),
            (format!("{{0,{huge}}}"), None),
            (format!("{{1,{huge}}}?"), Some(0..1)),
        ] {
            let source = JsString::from(format!("(?:(?=(a))){bounds}a").as_str());
            let matcher =
                RegExpBackreferenceMatcher::compile_ordinary_with_named_bindings_and_work(
                    &source,
                    false,
                    false,
                    false,
                    RegExpBackreferenceNamedBindings::default(),
                    |_| Ok::<_, ()>(()),
                )
                .unwrap()
                .unwrap();
            assert_eq!(
                matcher
                    .find(&JsString::from("a"), 0, true)
                    .unwrap()
                    .captures[0],
                capture
            );
        }
        assert_eq!(
            clone
                .find_with_work(&JsString::from("a"), 0, true, |_| Err::<(), _>(
                    "explicit work"
                ))
                .unwrap_err(),
            "explicit work"
        );
    }

    #[test]
    fn lookahead_execution_snapshot() {
        let mut rows = String::new();
        for source in [
            r"(?=a)a",
            r"(?!a)b",
            r"a(?=(b))",
            r"(?=(a|ab))\1b",
            r"(?=(ab|a))\1b",
            r"(?=(a+))a*b\1",
            r"(?!(a)b)\1a",
            r"(?=(?=(a|b))\1)\1",
            r"(?!(?!a))a",
            r"(a)(?=\1)\1",
            r"(?=(a|b)+)\1",
            r"(?:(?=(a|b))\1c)+",
            r"(?:(?=(a|b))a|b)+c",
            r"(?:(?!(a)b)a|b)+c",
            r"(?=(a*))\1",
            r"(?!(a+))b",
            r"(?=(^a))\1",
            r"(?=(.$))a",
            r"(?=(\uD800))\1",
            r"((?=(a))a)|b",
            r"(?=(a|ab)+b)\1",
            r"(?=)(?!)|a",
            r"(?!(?=(a))\1b)a",
            r"(?=(?!(a)b)a)a",
            r"(?=(a))\1?b",
            r"(a|b)(?=\1$)\1",
            r"(?=(a))(?!b)\1",
            r"(?:(?=(a))a)+b",
        ] {
            for (ignore_case, multiline, dot_all) in [
                (false, false, false),
                (true, false, false),
                (false, true, true),
            ] {
                let matcher =
                    RegExpBackreferenceMatcher::compile_ordinary_with_named_bindings_and_work(
                        &JsString::from(source),
                        ignore_case,
                        multiline,
                        dot_all,
                        RegExpBackreferenceNamedBindings::default(),
                        |_| Ok::<_, ()>(()),
                    )
                    .unwrap()
                    .unwrap();
                for text in [
                    "", "a", "b", "ab", "abb", "aab", "aba", "baabac", "acbc", "abc", "AA", "q\na",
                    "a\n", "\u{d7ff}", "😀",
                ] {
                    let input = JsString::from(text);
                    for (start, sticky) in [(0, false), (0, true), (1, true), (2, false)] {
                        let result = matcher.find(&input, start, sticky);
                        writeln!(rows, "{source:?} i={ignore_case} m={multiline} s={dot_all} input={input:?} start={start} sticky={sticky} {result:?}").unwrap();
                    }
                }
            }
        }
        insta::assert_snapshot!(rows);
    }

    #[test]
    fn lookahead_commits_captures_and_unwinds_negative_and_nested_paths() {
        fn find(source: &str, text: &str) -> Option<RegExpBackreferenceMatch> {
            RegExpBackreferenceMatcher::compile_ordinary_with_named_bindings_and_work(
                &JsString::from(source),
                false,
                false,
                false,
                RegExpBackreferenceNamedBindings::default(),
                |_| Ok::<_, ()>(()),
            )
            .unwrap()
            .unwrap()
            .find(&JsString::from(text), 0, true)
        }
        let x = find(r"a(?=(b))", "ab").unwrap();
        assert_eq!(x.range, 0..1);
        assert_eq!(&*x.captures, &[Some(1..2)]);
        assert!(find(r"(?=(a|ab))\1b$", "abb").is_none());
        assert_eq!(
            find(r"(?=(ab|a))\1b$", "abb").unwrap().captures[0],
            Some(0..2)
        );
        assert_eq!(find(r"(?!(a)b)\1a", "a").unwrap().captures[0], None);
        assert_eq!(find(r"(?=(?!(a)b)a)a", "a").unwrap().captures[0], None);
        assert_eq!(
            find(r"(?:(?=(a|b))a|b)+c", "abc").unwrap().captures[0],
            None
        );
        assert_eq!(
            find(r"(?:(?=(a|b))\1c)+", "acbc").unwrap().captures[0],
            Some(2..3)
        );
        assert_eq!(find(r"(?!(?=(a))\1b)a", "a").unwrap().captures[0], None);
    }

    #[test]
    fn deeply_nested_lookahead_is_flat_and_progress_and_limits_remain_explicit() {
        let source = JsString::from(
            format!("{}(a){}\\1", "(?=".repeat(100000), ")".repeat(100000)).as_str(),
        );
        let matcher = RegExpBackreferenceMatcher::compile(&source, false).unwrap();
        let clone = matcher.clone();
        drop(matcher);
        let found = clone.find(&JsString::from("a"), 0, true).unwrap();
        assert_eq!(found.range, 0..1);
        assert_eq!(found.captures[0], Some(0..1));
        assert_eq!(
            clone
                .find_with_work(&JsString::from("a"), 0, true, |_| Err::<(), _>(
                    "explicit work"
                ))
                .unwrap_err(),
            "explicit work"
        );
        for depth in [100000, 100001] {
            let source =
                JsString::from(format!("{}b{}", "(?!".repeat(depth), ")".repeat(depth)).as_str());
            let matcher =
                RegExpBackreferenceMatcher::compile_ordinary_with_named_bindings_and_work(
                    &source,
                    false,
                    false,
                    false,
                    RegExpBackreferenceNamedBindings::default(),
                    |_| Ok::<_, ()>(()),
                )
                .unwrap()
                .unwrap();
            assert_eq!(
                matcher.find(&JsString::from("a"), 0, true).is_some(),
                depth % 2 == 1
            );
        }
        for text in [
            r"(?=a)+",
            r"(?!a)?",
            r"(?:(?<=(a+)))*",
            r"(?:(?=(a))|b)+",
            r"(?<=(a+))b",
            r"(?i:a)",
        ] {
            let mut charged = 0;
            assert!(
                RegExpBackreferenceMatcher::compile_ordinary_with_named_bindings_and_work(
                    &JsString::from(text),
                    false,
                    false,
                    false,
                    RegExpBackreferenceNamedBindings::default(),
                    |n| {
                        charged += n;
                        Ok::<_, ()>(())
                    },
                )
                .unwrap()
                .is_none(),
                "{text}"
            );
            assert_eq!(charged, 0);
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
            r"(?:(?:(?:(a)(?:\1)+){2})|)*",
            r"(?:(?:(?:(a|b)(?:\1)+){2})|)*",
            r"(?<x>a)\k<x>",
            r"(?:(?:(?:([ab])(?:\1)+){2})|)*",
            r"(?:(?:(?:^(a)(?:\1)+){2})|)*",
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
