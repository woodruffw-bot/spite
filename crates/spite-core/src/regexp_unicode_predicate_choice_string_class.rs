use super::{
    Body, PlainBody, RegExpUnicodeUnassertedZeroCountStringClassCaptureMatcher,
    RegExpUnicodeZeroCountAssertedStringClassCaptureMatcher, asserted_repeat_count, counted_hole,
    simple_count,
};
use crate::{
    JsString, RegExpUnicodeMatchError, RegExpUnicodeRepeatedStringClassMatch,
    RegExpUnicodeSetCompileError,
};
use std::ops::Range;

#[derive(Clone, Debug)]
enum Plan {
    Asserted(RegExpUnicodeZeroCountAssertedStringClassCaptureMatcher),
    Plain(RegExpUnicodeUnassertedZeroCountStringClassCaptureMatcher),
    Single(crate::RegExpUnicodeSingleStringClassCaptureMatcher),
    PlainAlternative(super::RegExpUnicodeUnassertedZeroCountStringClassAlternativeMatcher),
    AssertedAlternative(super::RegExpUnicodeZeroCountAssertedStringClassAlternativeMatcher),
}
/// String-class repetition beside always-successful predicate choices.
///
/// Requires validated Unicode Sets source and normalized capture names. Outside
/// choices contain only line/word assertions, empty groups and counts. Every
/// selected scope succeeds at all eight endpoint truth profiles; ordered branch
/// selection still determines which captures participate. The class body and
/// remaining predicates retain their complete proofs (22.2.2.3, 22.2.2.4).
/// https://262.ecma-international.org/17.0/#sec-patterns
#[derive(Clone, Debug)]
pub struct RegExpUnicodePredicateChoiceStringClassCaptureMatcher {
    plan: Plan,
    overlays: Box<[Option<Overlay>]>,
    ignore_case: bool,
    multiline: bool,
}
/// Repetition result and the assertion truth profiles at its UTF-16 endpoints.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RegExpUnicodePredicateChoiceStringClassMatch {
    /// Whole range and final accepted class-body iteration.
    pub repeated: RegExpUnicodeRepeatedStringClassMatch,
    /// Start-line, end-line and word-boundary bits at the whole-match start.
    pub before: u8,
    /// Start-line, end-line and word-boundary bits at the whole-match end.
    pub after: u8,
}
#[derive(Clone, Copy, Debug)]
struct Overlay {
    at_end: bool,
    mask: u8,
}
impl RegExpUnicodePredicateChoiceStringClassCaptureMatcher {
    /// Proves complete outside choices before compiling the class continuation.
    pub fn compile_with_work<E>(
        source: &JsString,
        ignore_case: bool,
        multiline: bool,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, RegExpUnicodeSetCompileError<E>> {
        let Some(Projection { shadow, overlays }) =
            project_choices(source, false, false, false, &mut charge)?
        else {
            return Ok(None);
        };
        Self::compile_projection(
            Projection { shadow, overlays },
            ignore_case,
            multiline,
            charge,
        )
    }
    fn compile_projection<E>(
        Projection { shadow, overlays }: Projection,
        ignore_case: bool,
        multiline: bool,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, RegExpUnicodeSetCompileError<E>> {
        macro_rules! attempt {
            ($matcher:ty,$family:ident,$variant:ident,$wrapper:ident,$plan:ident) => {
                if let Some(p) =
                    <$matcher>::compile_with_work(&shadow, ignore_case, multiline, &mut charge)?
                {
                    if p.capture_count() != overlays.len() {
                        return Ok(None);
                    }
                    return Ok(Some(Self {
                        plan: Plan::$plan($wrapper {
                            body: $family::$variant(p),
                        }),
                        overlays: overlays.into_boxed_slice(),
                        ignore_case,
                        multiline,
                    }));
                }
            };
        }
        attempt!(
            crate::RegExpUnicodeAssertedRepeatedStringClassCaptureMatcher,
            Body,
            Nonnullable,
            RegExpUnicodeZeroCountAssertedStringClassCaptureMatcher,
            Asserted
        );
        attempt!(
            crate::RegExpUnicodeAssertedNullableStringClassCaptureMatcher,
            Body,
            Nullable,
            RegExpUnicodeZeroCountAssertedStringClassCaptureMatcher,
            Asserted
        );
        attempt!(
            crate::RegExpUnicodeAssertedTwoRequiredNullableStringClassCaptureMatcher,
            Body,
            Two,
            RegExpUnicodeZeroCountAssertedStringClassCaptureMatcher,
            Asserted
        );
        attempt!(
            crate::RegExpUnicodeAssertedCountedNullableStringClassCaptureMatcher,
            Body,
            Counted,
            RegExpUnicodeZeroCountAssertedStringClassCaptureMatcher,
            Asserted
        );
        attempt!(
            crate::RegExpUnicodeAssertedCountedNullableScalarStringClassCaptureMatcher,
            Body,
            Scalar,
            RegExpUnicodeZeroCountAssertedStringClassCaptureMatcher,
            Asserted
        );
        attempt!(
            crate::RegExpUnicodeRepeatedStringClassCaptureMatcher,
            PlainBody,
            Simple,
            RegExpUnicodeUnassertedZeroCountStringClassCaptureMatcher,
            Plain
        );
        attempt!(
            crate::RegExpUnicodeBoundedStringClassCaptureMatcher,
            PlainBody,
            Bounded,
            RegExpUnicodeUnassertedZeroCountStringClassCaptureMatcher,
            Plain
        );
        attempt!(
            crate::RegExpUnicodeCountedNullableStringClassCaptureMatcher,
            PlainBody,
            Nullable,
            RegExpUnicodeUnassertedZeroCountStringClassCaptureMatcher,
            Plain
        );
        attempt!(
            crate::RegExpUnicodeTwoRequiredStringClassCaptureMatcher,
            PlainBody,
            Two,
            RegExpUnicodeUnassertedZeroCountStringClassCaptureMatcher,
            Plain
        );
        attempt!(
            crate::RegExpUnicodeRequiredStringClassCaptureMatcher,
            PlainBody,
            Required,
            RegExpUnicodeUnassertedZeroCountStringClassCaptureMatcher,
            Plain
        );
        Ok(None)
    }
    /// Counts all original lexical capture slots, including unselected branches.
    pub fn capture_count(&self) -> usize {
        self.overlays.len()
    }
    /// Projects branch participation using the truth profiles of this result.
    pub fn capture_range(
        &self,
        index: usize,
        found: &RegExpUnicodePredicateChoiceStringClassMatch,
    ) -> Option<Range<usize>> {
        if let Some(overlay) = self.overlays.get(index).copied().flatten() {
            let (truth, point) = if overlay.at_end {
                (found.after, found.repeated.range.end)
            } else {
                (found.before, found.repeated.range.start)
            };
            return (overlay.mask & 1u8.checked_shl(u32::from(truth)).unwrap_or(0) != 0)
                .then_some(point..point);
        }
        match &self.plan {
            Plan::Asserted(p) => p.capture_range(index, &found.repeated),
            Plan::Plain(p) => p.capture_range(index, &found.repeated),
            Plan::Single(p) => p.capture_range(index, &found.repeated.range),
            Plan::PlainAlternative(p) => p.capture_range(index, &found.repeated),
            Plan::AssertedAlternative(p) => p.capture_range(index, &found.repeated),
        }
    }
    /// Executes the proved repetition and records endpoint assertion truth.
    pub fn find_with_captures_and_work<E>(
        &self,
        input: &JsString,
        requested: usize,
        sticky: bool,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<RegExpUnicodePredicateChoiceStringClassMatch>, RegExpUnicodeMatchError<E>>
    {
        let found = match &self.plan {
            Plan::Asserted(p) => {
                p.find_with_captures_and_work(input, requested, sticky, &mut charge)?
            }
            Plan::Plain(p) => {
                p.find_with_captures_and_work(input, requested, sticky, &mut charge)?
            }
            Plan::Single(p) => p
                .find_with_captures_and_work(input, requested, sticky, &mut charge)
                .map_err(RegExpUnicodeMatchError::Work)?
                .map(|range| RegExpUnicodeRepeatedStringClassMatch {
                    range: range.clone(),
                    last_iteration: Some(range),
                    class_selected: true,
                }),
            Plan::PlainAlternative(p) => {
                p.find_with_captures_and_work(input, requested, sticky, &mut charge)?
            }
            Plan::AssertedAlternative(p) => {
                p.find_with_captures_and_work(input, requested, sticky, &mut charge)?
            }
        };
        let Some(repeated) = found else {
            return Ok(None);
        };
        charge(16).map_err(RegExpUnicodeMatchError::Work)?;
        let truth = |at| {
            use crate::regexp_unicode_character::{
                unicode_assertions_match, unicode_word_boundary,
            };
            let units = input.code_units();
            u8::from(unicode_assertions_match(
                units,
                at,
                at,
                true,
                false,
                self.multiline,
            )) | u8::from(unicode_assertions_match(
                units,
                at,
                at,
                false,
                true,
                self.multiline,
            )) << 1
                | u8::from(unicode_word_boundary(units, at, self.ignore_case)) << 2
        };
        Ok(Some(RegExpUnicodePredicateChoiceStringClassMatch {
            before: truth(repeated.range.start),
            after: truth(repeated.range.end),
            repeated,
        }))
    }
}
#[derive(Clone, Copy, Debug)]
enum Kind {
    Empty,
    Predicate,
    Payload,
    Concat(usize, usize),
    Choice(usize, usize),
    Group(usize, Option<usize>),
    Repeat(usize, bool),
}
#[derive(Debug)]
struct Node {
    kind: Kind,
    success: u8,
    predicate: bool,
    payload: bool,
    span: Range<usize>,
}
#[derive(Debug)]
struct Frame {
    open: usize,
    capture: Option<usize>,
    sequence: Option<usize>,
    choices: Option<usize>,
}
fn leaf(
    nodes: &mut Vec<Node>,
    kind: Kind,
    success: u8,
    predicate: bool,
    payload: bool,
    span: Range<usize>,
) -> usize {
    let id = nodes.len();
    nodes.push(Node {
        kind,
        success,
        predicate,
        payload,
        span,
    });
    id
}
fn pair(
    nodes: &mut Vec<Node>,
    left: usize,
    right: usize,
    choice: bool,
    alternative: bool,
) -> Option<usize> {
    let (a, b) = (&nodes[left], &nodes[right]);
    if choice && (a.payload || b.payload) && !alternative {
        return None;
    }
    let (success, predicate, payload, span) = (
        if choice {
            a.success | b.success
        } else {
            a.success & b.success
        },
        a.predicate || b.predicate,
        a.payload || b.payload,
        a.span.start..b.span.end,
    );
    Some(leaf(
        nodes,
        if choice {
            Kind::Choice(left, right)
        } else {
            Kind::Concat(left, right)
        },
        success,
        predicate,
        payload,
        span,
    ))
}
fn append(nodes: &mut Vec<Node>, frame: &mut Frame, node: usize, alternative: bool) -> Option<()> {
    frame.sequence = Some(if let Some(left) = frame.sequence {
        pair(nodes, left, node, false, alternative)?
    } else {
        node
    });
    Some(())
}
fn finish(nodes: &mut Vec<Node>, frame: &mut Frame, at: usize, alternative: bool) -> Option<usize> {
    let sequence = frame
        .sequence
        .take()
        .unwrap_or_else(|| leaf(nodes, Kind::Empty, 0xff, false, false, at..at));
    Some(if let Some(left) = frame.choices.take() {
        pair(nodes, left, sequence, true, alternative)?
    } else {
        sequence
    })
}
struct Projection {
    shadow: JsString,
    overlays: Vec<Option<Overlay>>,
}
fn project_choices<E>(
    source: &JsString,
    single: bool,
    alternative: bool,
    conjunctive: bool,
    charge: &mut impl FnMut(usize) -> Result<(), E>,
) -> Result<Option<Projection>, RegExpUnicodeSetCompileError<E>> {
    use RegExpUnicodeSetCompileError::{Capacity, Work};
    let units = source.code_units();
    charge(units.len().saturating_mul(128).saturating_add(256)).map_err(Work)?;
    let Some(hole) = counted_hole(units, |u, at| {
        if single {
            if at == 0 || u.get(at - 1) != Some(&0x5d) {
                return None;
            }
            if let Some((end, min, delta, greedy)) = asserted_repeat_count(u, at) {
                return (min == 1 && delta == Some(0)).then_some((end, true, true, greedy));
            }
            return simple_count(u, at)
                .is_none()
                .then_some((at, true, true, true));
        }
        if let Some(c) = simple_count(u, at) {
            return Some(c);
        }
        let (end, min, delta, greedy) = asserted_repeat_count(u, at)?;
        if min == 1 && delta == Some(0) {
            return None;
        }
        Some((
            end,
            min > 0,
            min.saturating_add(delta.unwrap_or(usize::MAX)) == 1,
            greedy,
        ))
    }) else {
        return Ok(None);
    };
    // At most two binary nodes plus a group/count node per source unit.
    // All parser and traversal state is flat, including deeply nested scopes.
    let capacity = units
        .len()
        .checked_mul(4)
        .and_then(|n| n.checked_add(8))
        .ok_or(Capacity)?;
    let mut nodes = Vec::new();
    nodes.try_reserve_exact(capacity).map_err(|_| Capacity)?;
    let mut frames = Vec::new();
    frames
        .try_reserve_exact(units.len().checked_add(1).ok_or(Capacity)?)
        .map_err(|_| Capacity)?;
    frames.push(Frame {
        open: 0,
        capture: None,
        sequence: None,
        choices: None,
    });
    let (mut at, mut captures) = (0, 0);
    while at < units.len() {
        let node;
        if at == hole.atom.start {
            if captures != hole.before {
                return Ok(None);
            }
            let mut p = at;
            while p < hole.count_end {
                match units[p] {
                    0x5b => {
                        let Some(end) = super::super::class_end(units, p) else {
                            return Ok(None);
                        };
                        p = end;
                    }
                    0x5c => p += 2,
                    0x28 => {
                        if units.get(p + 1) != Some(&0x3f) {
                            captures += 1
                        }
                        p += 1;
                    }
                    _ => p += 1,
                }
            }
            node = leaf(
                &mut nodes,
                Kind::Payload,
                0,
                false,
                true,
                at..hole.count_end,
            );
            at = hole.count_end;
        } else {
            let start = at;
            match units[at] {
                0x28 => {
                    let capture = if units.get(at + 1) == Some(&0x3f) {
                        if units.get(at + 2) != Some(&0x3a) {
                            return Ok(None);
                        }
                        at += 3;
                        None
                    } else {
                        let slot = captures;
                        captures += 1;
                        at += 1;
                        Some(slot)
                    };
                    frames.push(Frame {
                        open: start,
                        capture,
                        sequence: None,
                        choices: None,
                    });
                    continue;
                }
                0x7c => {
                    let Some(frame) = frames.last_mut() else {
                        return Ok(None);
                    };
                    let Some(done) = finish(&mut nodes, frame, at, alternative) else {
                        return Ok(None);
                    };
                    frame.choices = Some(done);
                    at += 1;
                    continue;
                }
                0x29 => {
                    if frames.len() == 1 {
                        return Ok(None);
                    }
                    let Some(mut frame) = frames.pop() else {
                        return Ok(None);
                    };
                    let Some(child) = finish(&mut nodes, &mut frame, at, alternative) else {
                        return Ok(None);
                    };
                    at += 1;
                    let n = &nodes[child];
                    let (success, predicate, payload) = (n.success, n.predicate, n.payload);
                    let group = leaf(
                        &mut nodes,
                        Kind::Group(child, frame.capture),
                        success,
                        predicate,
                        payload,
                        frame.open..at,
                    );
                    let count = if let Some((end, min, delta, _)) = asserted_repeat_count(units, at)
                    {
                        Some((end, min == 0, min == 1 && delta == Some(0)))
                    } else {
                        simple_count(units, at).map(|(end, one, max, _)| (end, !one, one && max))
                    };
                    node = if let Some((end, zero, exact_one)) = count {
                        if payload && !(single && exact_one) {
                            return Ok(None);
                        }
                        at = end;
                        leaf(
                            &mut nodes,
                            Kind::Repeat(group, zero),
                            if zero { 0xff } else { success },
                            predicate,
                            payload,
                            frame.open..at,
                        )
                    } else {
                        group
                    };
                }
                0x5e | 0x24 => {
                    at += 1;
                    node = leaf(
                        &mut nodes,
                        Kind::Predicate,
                        if units[start] == 0x5e { 0xaa } else { 0xcc },
                        true,
                        false,
                        start..at,
                    );
                }
                0x5c if units.get(at + 1).is_some_and(|u| matches!(*u, 0x62 | 0x42)) => {
                    at += 2;
                    node = leaf(
                        &mut nodes,
                        Kind::Predicate,
                        if units[start + 1] == 0x62 { 0xf0 } else { 0x0f },
                        true,
                        false,
                        start..at,
                    );
                }
                _ => return Ok(None),
            }
        }
        let Some(frame) = frames.last_mut() else {
            return Ok(None);
        };
        if append(&mut nodes, frame, node, alternative).is_none() {
            return Ok(None);
        }
    }
    if frames.len() != 1 {
        return Ok(None);
    }
    let Some(mut frame) = frames.pop() else {
        return Ok(None);
    };
    let Some(root) = finish(&mut nodes, &mut frame, at, alternative) else {
        return Ok(None);
    };
    let mut overlays = Vec::new();
    overlays.try_reserve_exact(captures).map_err(|_| Capacity)?;
    overlays.resize(captures, None);
    let mut erased = Vec::new();
    erased
        .try_reserve_exact(units.len())
        .map_err(|_| Capacity)?;
    erased.resize(units.len(), false);
    let mut gates = Vec::new();
    if conjunctive {
        gates.try_reserve_exact(units.len()).map_err(|_| Capacity)?;
        gates.resize(units.len(), 0xff);
    }
    let mut pending = Vec::new();
    pending
        .try_reserve_exact(nodes.len())
        .map_err(|_| Capacity)?;
    pending.push((root, None, false));
    let mut selected = false;
    while let Some((id, active, gated)) = pending.pop() {
        let n = &nodes[id];
        let active = if !gated
            && active.is_none()
            && !n.payload
            && n.predicate
            && (n.success == 0xff || (conjunctive && canonical_assertions(n.success).is_some()))
            && matches!(n.kind, Kind::Group(..) | Kind::Repeat(..))
        {
            selected = true;
            if conjunctive {
                gates[n.span.start] = n.success;
            }
            Some(0xff)
        } else {
            active
        };
        match n.kind {
            Kind::Predicate => {
                if active.is_some() {
                    erased[n.span.start] = true
                }
            }
            Kind::Group(child, capture) => {
                if let (Some(mask), Some(slot)) = (active, capture) {
                    overlays[slot] = Some(Overlay {
                        at_end: n.span.start >= hole.count_end,
                        mask: mask & n.success,
                    });
                }
                pending.push((child, active, gated));
            }
            Kind::Repeat(child, zero) => {
                pending.push((child, active.map(|mask| if zero { 0 } else { mask }), gated))
            }
            Kind::Concat(a, b) => {
                pending.push((b, active, gated));
                pending.push((a, active, gated));
            }
            Kind::Choice(a, b) => {
                // A selected scope below an unselected choice could be skipped.
                // Such an ancestor must itself have been proved always successful.
                if active.is_none() && n.predicate && !(alternative && n.payload) {
                    return Ok(None);
                }
                pending.push((
                    b,
                    active.map(|mask| mask & !nodes[a].success & nodes[b].success),
                    gated || (alternative && n.payload),
                ));
                pending.push((
                    a,
                    active.map(|mask| mask & nodes[a].success),
                    gated || (alternative && n.payload),
                ));
            }
            Kind::Empty | Kind::Payload => {}
        }
    }
    if !selected {
        return Ok(None);
    }
    let mut shadow = Vec::new();
    shadow
        .try_reserve_exact(
            units
                .len()
                .checked_mul(if conjunctive { 8 } else { 4 })
                .ok_or(Capacity)?,
        )
        .map_err(|_| Capacity)?;
    let mut at = 0;
    while at < units.len() {
        if let Some(&mask) = gates.get(at) {
            if let Some(flags) = canonical_assertions(mask) {
                for (required, tokens) in [
                    (flags[0], &[0x5e][..]),
                    (flags[1], &[0x24][..]),
                    (flags[2], &[0x5c, 0x62][..]),
                    (flags[3], &[0x5c, 0x42][..]),
                ] {
                    if required {
                        shadow.extend_from_slice(tokens);
                    }
                }
            }
        }
        if erased[at] {
            shadow.extend_from_slice(&[0x28, 0x3f, 0x3a, 0x29]);
            at += if units[at] == 0x5c { 2 } else { 1 };
        } else {
            shadow.push(units[at]);
            at += 1;
        }
    }
    Ok(Some(Projection {
        shadow: JsString::from_code_units(shadow),
        overlays,
    }))
}

/// One required class beside always-successful outside predicate choices.
///
/// Requires validated Unicode Sets source and normalized capture names. Bare or
/// exact-one class counts preserve required empty class identity; ordered outside
/// choices preserve conditional capture participation at actual endpoints.
/// https://262.ecma-international.org/17.0/#sec-repeatmatcher
#[derive(Clone, Debug)]
pub struct RegExpUnicodePredicateChoiceSingleStringClassCaptureMatcher {
    plan: RegExpUnicodePredicateChoiceStringClassCaptureMatcher,
}
impl RegExpUnicodePredicateChoiceSingleStringClassCaptureMatcher {
    /// Proves complete outside choices and a bare or exact-one class spine.
    pub fn compile_with_work<E>(
        source: &JsString,
        ignore_case: bool,
        multiline: bool,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, RegExpUnicodeSetCompileError<E>> {
        let Some(Projection { shadow, overlays }) =
            project_choices(source, true, false, false, &mut charge)?
        else {
            return Ok(None);
        };
        Self::compile_projection(
            Projection { shadow, overlays },
            ignore_case,
            multiline,
            charge,
        )
    }
    fn compile_projection<E>(
        Projection { shadow, overlays }: Projection,
        ignore_case: bool,
        multiline: bool,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, RegExpUnicodeSetCompileError<E>> {
        use crate::RegExpUnicodeSingleStringClassCaptureMatcher as P;
        macro_rules! attempt {
            ($method:ident) => {
                if let Some(p) = P::$method(&shadow, ignore_case, multiline, &mut charge)? {
                    if p.capture_count() != overlays.len() {
                        return Ok(None);
                    }
                    return Ok(Some(Self {
                        plan: RegExpUnicodePredicateChoiceStringClassCaptureMatcher {
                            plan: Plan::Single(p),
                            overlays: overlays.into_boxed_slice(),
                            ignore_case,
                            multiline,
                        },
                    }));
                }
            };
        }
        attempt!(compile_with_work);
        attempt!(compile_exact_single_counts_with_work);
        attempt!(compile_unconditional_assertions_with_work);
        attempt!(compile_reversed_line_assertions_with_work);
        Ok(None)
    }
    /// Counts all original lexical captures, including unselected branches.
    pub fn capture_count(&self) -> usize {
        self.plan.capture_count()
    }
    /// Projects original slots using this result's endpoint truth profiles.
    pub fn capture_range(
        &self,
        index: usize,
        found: &RegExpUnicodePredicateChoiceStringClassMatch,
    ) -> Option<Range<usize>> {
        self.plan.capture_range(index, found)
    }
    /// Executes one class and records assertion truth at its actual endpoints.
    pub fn find_with_captures_and_work<E>(
        &self,
        input: &JsString,
        requested: usize,
        sticky: bool,
        charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<RegExpUnicodePredicateChoiceStringClassMatch>, RegExpUnicodeMatchError<E>>
    {
        self.plan
            .find_with_captures_and_work(input, requested, sticky, charge)
    }
}

/// A required class/empty alternative beside always-successful predicate choices.
///
/// Requires validated Unicode Sets source and normalized capture names. Projected
/// scopes must be mandatory outside every class-bearing alternative. Full branch
/// proofs retain source priority, endpoint gates and required empty class identity.
/// https://262.ecma-international.org/17.0/#sec-disjunction
#[derive(Clone, Debug)]
pub struct RegExpUnicodePredicateChoiceStringClassAlternativeMatcher {
    plan: RegExpUnicodePredicateChoiceStringClassCaptureMatcher,
}
impl RegExpUnicodePredicateChoiceStringClassAlternativeMatcher {
    /// Proves mandatory outside choices before complete class/empty branches.
    pub fn compile_with_work<E>(
        source: &JsString,
        ignore_case: bool,
        multiline: bool,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, RegExpUnicodeSetCompileError<E>> {
        let Some(Projection { shadow, overlays }) =
            project_choices(source, true, true, false, &mut charge)?
        else {
            return Ok(None);
        };
        charge(shadow.len().saturating_mul(24).saturating_add(128))
            .map_err(RegExpUnicodeSetCompileError::Work)?;
        let source = super::super::normalize_ones(shadow.code_units());
        macro_rules! attempt {
            ($matcher:ty,$method:ident,$family:ident,$variant:ident,$wrapper:ident,$plan:ident) => {
                if let Some(p) = <$matcher>::$method(&source, ignore_case, multiline, &mut charge)?
                {
                    if p.capture_count() != overlays.len() {
                        return Ok(None);
                    }
                    return Ok(Some(Self {
                        plan: RegExpUnicodePredicateChoiceStringClassCaptureMatcher {
                            plan: Plan::$plan(super::$wrapper {
                                body: super::$family::$variant(p),
                            }),
                            overlays: overlays.into_boxed_slice(),
                            ignore_case,
                            multiline,
                        },
                    }));
                }
            };
        }
        macro_rules! asserted {
            ($method:ident) => {
                attempt!(
                    crate::RegExpUnicodeAssertedStringClassAlternativeMatcher,
                    $method,
                    AssertedAlternativeBody,
                    Root,
                    RegExpUnicodeZeroCountAssertedStringClassAlternativeMatcher,
                    AssertedAlternative
                );
            };
        }
        asserted!(compile_with_work);
        asserted!(compile_grouped_with_work);
        attempt!(
            crate::RegExpUnicodeSurroundingAssertedStringClassAlternativeMatcher,
            compile_with_work,
            AssertedAlternativeBody,
            Surrounding,
            RegExpUnicodeZeroCountAssertedStringClassAlternativeMatcher,
            AssertedAlternative
        );
        asserted!(compile_surrounding_line_assertions_with_work);
        asserted!(compile_surrounding_word_assertions_with_work);
        asserted!(compile_nested_assertion_wrappers_with_work);
        asserted!(compile_reversed_surrounding_line_assertions_with_work);
        asserted!(compile_nested_endpoint_assertion_wrappers_with_work);
        attempt!(
            crate::RegExpUnicodeNullableStringClassCaptureMatcher,
            compile_with_work,
            UnassertedAlternativeBody,
            Nonempty,
            RegExpUnicodeUnassertedZeroCountStringClassAlternativeMatcher,
            PlainAlternative
        );
        attempt!(
            crate::RegExpUnicodeNullableStringClassCaptureMatcher,
            compile_grouped_with_work,
            UnassertedAlternativeBody,
            Nonempty,
            RegExpUnicodeUnassertedZeroCountStringClassAlternativeMatcher,
            PlainAlternative
        );
        attempt!(
            crate::RegExpUnicodeEmptyStringClassCaptureMatcher,
            compile_with_work,
            UnassertedAlternativeBody,
            Empty,
            RegExpUnicodeUnassertedZeroCountStringClassAlternativeMatcher,
            PlainAlternative
        );
        attempt!(
            crate::RegExpUnicodeEmptyStringClassCaptureMatcher,
            compile_grouped_with_work,
            UnassertedAlternativeBody,
            Empty,
            RegExpUnicodeUnassertedZeroCountStringClassAlternativeMatcher,
            PlainAlternative
        );
        Ok(None)
    }
    /// Counts original lexical slots, including unselected class/empty branches.
    pub fn capture_count(&self) -> usize {
        self.plan.capture_count()
    }
    /// Projects class selection and conditional outside points at actual endpoints.
    pub fn capture_range(
        &self,
        index: usize,
        found: &RegExpUnicodePredicateChoiceStringClassMatch,
    ) -> Option<Range<usize>> {
        self.plan.capture_range(index, found)
    }
    /// Executes original branch order and records assertion truth at both endpoints.
    pub fn find_with_captures_and_work<E>(
        &self,
        input: &JsString,
        requested: usize,
        sticky: bool,
        charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<RegExpUnicodePredicateChoiceStringClassMatch>, RegExpUnicodeMatchError<E>>
    {
        self.plan
            .find_with_captures_and_work(input, requested, sticky, charge)
    }
}

// Eight endpoint profiles share the same assertion masks used by the parser.
// A conjunction is inserted outside its complete zero-width scope so the
// existing matcher, including endpoint retries, still evaluates a live gate.
fn canonical_assertions(mask: u8) -> Option<[bool; 4]> {
    for bits in 0..16u8 {
        let flags = [bits & 1 != 0, bits & 2 != 0, bits & 4 != 0, bits & 8 != 0];
        let mut success = 0xff;
        for (required, predicate) in flags.into_iter().zip([0xaa, 0xcc, 0xf0, 0x0f]) {
            if required {
                success &= predicate;
            }
        }
        if success == mask {
            return Some(flags);
        }
    }
    None
}
/// String-class repetition beside predicate choices equivalent to live gates.
///
/// Requires validated Unicode Sets source and normalized capture names. Every
/// selected scope succeeds exactly when a conjunction of line/word assertions
/// holds. Complete scope projection retains ordered conditional captures while
/// live gates preserve class-member and repetition endpoint retries.
/// https://262.ecma-international.org/17.0/#sec-compileassertion
#[derive(Clone, Debug)]
pub struct RegExpUnicodeConjunctivePredicateChoiceStringClassCaptureMatcher {
    plan: RegExpUnicodePredicateChoiceStringClassCaptureMatcher,
}
impl RegExpUnicodeConjunctivePredicateChoiceStringClassCaptureMatcher {
    /// Proves outside predicate choices and their complete class continuation.
    pub fn compile_with_work<E>(
        source: &JsString,
        ignore_case: bool,
        multiline: bool,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, RegExpUnicodeSetCompileError<E>> {
        let Some(projection) = project_choices(source, false, false, true, &mut charge)? else {
            return Ok(None);
        };
        RegExpUnicodePredicateChoiceStringClassCaptureMatcher::compile_projection(
            projection,
            ignore_case,
            multiline,
            charge,
        )
        .map(|plan| plan.map(|plan| Self { plan }))
    }
    /// Counts original lexical captures, including unselected predicate branches.
    pub fn capture_count(&self) -> usize {
        self.plan.capture_count()
    }
    /// Projects branch participation at the proved match's actual endpoints.
    pub fn capture_range(
        &self,
        index: usize,
        found: &RegExpUnicodePredicateChoiceStringClassMatch,
    ) -> Option<Range<usize>> {
        self.plan.capture_range(index, found)
    }
    /// Executes live conjunction gates with original class and count priority.
    pub fn find_with_captures_and_work<E>(
        &self,
        input: &JsString,
        requested: usize,
        sticky: bool,
        charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<RegExpUnicodePredicateChoiceStringClassMatch>, RegExpUnicodeMatchError<E>>
    {
        self.plan
            .find_with_captures_and_work(input, requested, sticky, charge)
    }
}

/// One required class beside predicate choices equivalent to live conjunctions.
///
/// Requires validated Unicode Sets source and normalized capture names. Complete
/// predicate scopes retain ordered conditional captures; live gates preserve
/// member retries and required empty class identity on bare/exact-one paths.
/// https://262.ecma-international.org/17.0/#sec-compileassertion
#[derive(Clone, Debug)]
pub struct RegExpUnicodeConjunctivePredicateChoiceSingleStringClassCaptureMatcher {
    plan: RegExpUnicodePredicateChoiceSingleStringClassCaptureMatcher,
}
impl RegExpUnicodeConjunctivePredicateChoiceSingleStringClassCaptureMatcher {
    /// Proves complete outside choices and a bare or exact-one class spine.
    pub fn compile_with_work<E>(
        source: &JsString,
        ignore_case: bool,
        multiline: bool,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, RegExpUnicodeSetCompileError<E>> {
        let Some(projection) = project_choices(source, true, false, true, &mut charge)? else {
            return Ok(None);
        };
        RegExpUnicodePredicateChoiceSingleStringClassCaptureMatcher::compile_projection(
            projection,
            ignore_case,
            multiline,
            charge,
        )
        .map(|plan| plan.map(|plan| Self { plan }))
    }
    /// Counts original lexical captures, including unselected assertion branches.
    pub fn capture_count(&self) -> usize {
        self.plan.capture_count()
    }
    /// Projects original slots using this result's actual endpoint truth.
    pub fn capture_range(
        &self,
        index: usize,
        found: &RegExpUnicodePredicateChoiceStringClassMatch,
    ) -> Option<Range<usize>> {
        self.plan.capture_range(index, found)
    }
    /// Executes the proved class and live gates with original member priority.
    pub fn find_with_captures_and_work<E>(
        &self,
        input: &JsString,
        requested: usize,
        sticky: bool,
        charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<RegExpUnicodePredicateChoiceStringClassMatch>, RegExpUnicodeMatchError<E>>
    {
        self.plan
            .find_with_captures_and_work(input, requested, sticky, charge)
    }
}
