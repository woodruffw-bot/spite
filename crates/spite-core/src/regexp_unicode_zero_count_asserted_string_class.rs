use super::{asserted_repeat_count, counted_hole, simple_count};
use crate::{
    JsString, RegExpUnicodeAssertedCountedNullableScalarStringClassCaptureMatcher,
    RegExpUnicodeAssertedCountedNullableStringClassCaptureMatcher,
    RegExpUnicodeAssertedNullableStringClassCaptureMatcher,
    RegExpUnicodeAssertedRepeatedStringClassCaptureMatcher,
    RegExpUnicodeAssertedTwoRequiredNullableStringClassCaptureMatcher, RegExpUnicodeMatchError,
    RegExpUnicodeRepeatedStringClassMatch, RegExpUnicodeSetCompileError,
};
use std::ops::Range;

#[derive(Clone, Debug)]
enum Body {
    Nonnullable(RegExpUnicodeAssertedRepeatedStringClassCaptureMatcher),
    Nullable(RegExpUnicodeAssertedNullableStringClassCaptureMatcher),
    Two(RegExpUnicodeAssertedTwoRequiredNullableStringClassCaptureMatcher),
    Counted(RegExpUnicodeAssertedCountedNullableStringClassCaptureMatcher),
    Scalar(RegExpUnicodeAssertedCountedNullableScalarStringClassCaptureMatcher),
}
/// String-class repetition with skipped outside assertion scopes.
///
/// Requires validated v/vi input and normalized capture names. At least one
/// outside assertion is enclosed in an exact-zero group, and other outside
/// assertions satisfy the unconditional repetition proof. Skipped scopes never
/// participate, so their captures remain undefined even if their assertions
/// would succeed. The class and every live term retain their complete proofs.
/// https://262.ecma-international.org/17.0/#sec-repeatmatcher
#[derive(Clone, Debug)]
pub struct RegExpUnicodeZeroCountAssertedStringClassCaptureMatcher {
    body: Body,
}
impl RegExpUnicodeZeroCountAssertedStringClassCaptureMatcher {
    /// Proves exact-zero outside scopes before compiling the live repetition.
    pub fn compile_with_work<E>(
        source: &JsString,
        ignore_case: bool,
        multiline: bool,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, RegExpUnicodeSetCompileError<E>> {
        let Some(source) = skipped_assertions(source, &mut charge)? else {
            return Ok(None);
        };
        macro_rules! attempt {
            ($matcher:ty, $variant:ident) => {
                if let Some(plan) =
                    <$matcher>::compile_with_work(&source, ignore_case, multiline, &mut charge)?
                {
                    return Ok(Some(Self {
                        body: Body::$variant(plan),
                    }));
                }
            };
        }
        attempt!(
            RegExpUnicodeAssertedRepeatedStringClassCaptureMatcher,
            Nonnullable
        );
        attempt!(
            RegExpUnicodeAssertedNullableStringClassCaptureMatcher,
            Nullable
        );
        attempt!(
            RegExpUnicodeAssertedTwoRequiredNullableStringClassCaptureMatcher,
            Two
        );
        attempt!(
            RegExpUnicodeAssertedCountedNullableStringClassCaptureMatcher,
            Counted
        );
        attempt!(
            RegExpUnicodeAssertedCountedNullableScalarStringClassCaptureMatcher,
            Scalar
        );
        Ok(None)
    }
    /// Counts every original lexical capture, including skipped groups.
    pub fn capture_count(&self) -> usize {
        match &self.body {
            Body::Nonnullable(p) => p.capture_count(),
            Body::Nullable(p) => p.capture_count(),
            Body::Two(p) => p.capture_count(),
            Body::Counted(p) => p.capture_count(),
            Body::Scalar(p) => p.capture_count(),
        }
    }
    /// Projects original slots; exact-zero groups retain undefined captures.
    pub fn capture_range(
        &self,
        index: usize,
        found: &RegExpUnicodeRepeatedStringClassMatch,
    ) -> Option<Range<usize>> {
        match &self.body {
            Body::Nonnullable(p) => p.capture_range(index, found),
            Body::Nullable(p) => p.capture_range(index, found),
            Body::Two(p) => p.capture_range(index, found),
            Body::Counted(p) => p.capture_range(index, found),
            Body::Scalar(p) => p.capture_range(index, found),
        }
    }
    /// Executes only live predicates and the proved repetition continuation.
    pub fn find_with_captures_and_work<E>(
        &self,
        input: &JsString,
        requested: usize,
        sticky: bool,
        charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<RegExpUnicodeRepeatedStringClassMatch>, RegExpUnicodeMatchError<E>> {
        match &self.body {
            Body::Nonnullable(p) => p.find_with_captures_and_work(input, requested, sticky, charge),
            Body::Nullable(p) => p.find_with_captures_and_work(input, requested, sticky, charge),
            Body::Two(p) => p.find_with_captures_and_work(input, requested, sticky, charge),
            Body::Counted(p) => p.find_with_captures_and_work(input, requested, sticky, charge),
            Body::Scalar(p) => p.find_with_captures_and_work(input, requested, sticky, charge),
        }
    }
}

fn skipped_assertions<E>(
    source: &JsString,
    charge: &mut impl FnMut(usize) -> Result<(), E>,
) -> Result<Option<JsString>, RegExpUnicodeSetCompileError<E>> {
    skipped_assertions_with_shape(source, false, charge)
}
fn skipped_assertions_with_shape<E>(
    source: &JsString,
    single: bool,
    charge: &mut impl FnMut(usize) -> Result<(), E>,
) -> Result<Option<JsString>, RegExpUnicodeSetCompileError<E>> {
    skipped_predicates_with_shape(source, single, false, charge)
}
fn skipped_predicates_with_shape<E>(
    source: &JsString,
    single: bool,
    zero_minimum: bool,
    charge: &mut impl FnMut(usize) -> Result<(), E>,
) -> Result<Option<JsString>, RegExpUnicodeSetCompileError<E>> {
    use RegExpUnicodeSetCompileError::{Capacity, Work};
    let units = source.code_units();
    charge(units.len().saturating_mul(64).saturating_add(128)).map_err(Work)?;
    let Some(hole) = counted_hole(units, |units, at| {
        if single {
            // Only the class leaf supplies the single occurrence. Exact-one
            // ancestor counts are validated by the delegated single-class plan.
            if at == 0 || units.get(at - 1) != Some(&0x5d) {
                return None;
            }
            if let Some((end, minimum, delta, greedy)) = asserted_repeat_count(units, at) {
                return (minimum == 1 && delta == Some(0)).then_some((end, true, true, greedy));
            }
            return simple_count(units, at)
                .is_none()
                .then_some((at, true, true, true));
        }
        if let Some(count) = simple_count(units, at) {
            return Some(count);
        }
        let (end, minimum, delta, greedy) = asserted_repeat_count(units, at)?;
        if minimum == 1 && delta == Some(0) {
            return None;
        }
        Some((
            end,
            minimum > 0,
            minimum.saturating_add(delta.unwrap_or(usize::MAX)) == 1,
            greedy,
        ))
    }) else {
        return Ok(None);
    };
    let capacity = units.len().checked_add(1).ok_or(Capacity)?;
    let mut starts = Vec::<bool>::new();
    starts.try_reserve_exact(capacity).map_err(|_| Capacity)?;
    starts.resize(capacity, false);
    let mut ends = Vec::<bool>::new();
    ends.try_reserve_exact(capacity).map_err(|_| Capacity)?;
    ends.resize(capacity, false);
    let mut scopes = Vec::<(usize, bool)>::new();
    scopes.try_reserve_exact(capacity).map_err(|_| Capacity)?;
    let mut at = 0;
    while at < units.len() {
        if at == hole.atom.start {
            if let Some(scope) = scopes.last_mut() {
                scope.1 = true;
            }
            at = hole.count_end;
            continue;
        }
        match units[at] {
            0x28 => {
                let width = if units.get(at + 1) == Some(&0x3f) {
                    if units.get(at + 2) != Some(&0x3a) {
                        return Ok(None);
                    }
                    3
                } else {
                    1
                };
                scopes.push((at, false));
                at += width;
            }
            0x29 => {
                let Some((open, contains_body)) = scopes.pop() else {
                    return Ok(None);
                };
                let skipped =
                    if let Some((_, minimum, delta, _)) = asserted_repeat_count(units, at + 1) {
                        minimum == 0 && (delta == Some(0) || zero_minimum)
                    } else {
                        zero_minimum
                            && simple_count(units, at + 1)
                                .is_some_and(|(_, minimum_one, _, _)| !minimum_one)
                    };
                if skipped {
                    if contains_body {
                        return Ok(None);
                    }
                    starts[open] = true;
                    ends[at + 1] = true;
                }
                if let Some(parent) = scopes.last_mut() {
                    parent.1 |= contains_body;
                }
                at += 1;
            }
            0x5c => at = (at + 2).min(units.len()),
            0x5b => return Ok(None),
            _ => at += 1,
        }
    }
    if !scopes.is_empty() {
        return Ok(None);
    }
    let mut shadow = Vec::new();
    shadow
        .try_reserve_exact(units.len().checked_mul(4).ok_or(Capacity)?)
        .map_err(|_| Capacity)?;
    let (mut at, mut depth, mut erased) = (0, 0usize, false);
    while at < units.len() {
        if ends[at] {
            depth -= 1;
        }
        if starts[at] {
            depth += 1;
        }
        if at == hole.atom.start {
            shadow.extend_from_slice(&units[at..hole.count_end]);
            at = hole.count_end;
            continue;
        }
        let width = match units[at] {
            0x5e | 0x24 if depth > 0 => 1,
            0x5c if depth > 0 && units.get(at + 1).is_some_and(|u| matches!(*u, 0x62 | 0x42)) => 2,
            _ => 0,
        };
        if width > 0 {
            shadow.extend_from_slice(&[0x28, 0x3f, 0x3a, 0x29]);
            erased = true;
            at += width;
        } else if units[at] == 0x5c && at + 1 < units.len() {
            shadow.extend_from_slice(&units[at..at + 2]);
            at += 2;
        } else {
            shadow.push(units[at]);
            at += 1;
        }
    }
    Ok(erased.then(|| JsString::from_code_units(shadow)))
}

#[derive(Clone, Debug)]
enum PlainBody {
    Simple(crate::RegExpUnicodeRepeatedStringClassCaptureMatcher),
    Bounded(crate::RegExpUnicodeBoundedStringClassCaptureMatcher),
    Nullable(crate::RegExpUnicodeCountedNullableStringClassCaptureMatcher),
    Two(crate::RegExpUnicodeTwoRequiredStringClassCaptureMatcher),
    Required(crate::RegExpUnicodeRequiredStringClassCaptureMatcher),
}
/// A repetition whose only outside assertions occur in exact-zero scopes.
///
/// Requires validated v/vi input and normalized capture names. All assertion
/// terms must be skipped; otherwise the separate asserted proof applies. The
/// remaining class and pure-empty continuation keep their complete proofs and
/// original capture slots. Zero-count scopes never participate (22.2.2.3).
/// https://262.ecma-international.org/17.0/#sec-repeatmatcher
#[derive(Clone, Debug)]
pub struct RegExpUnicodeUnassertedZeroCountStringClassCaptureMatcher {
    body: PlainBody,
}
impl RegExpUnicodeUnassertedZeroCountStringClassCaptureMatcher {
    /// Proves skipped assertions and an assertion-free live repetition.
    pub fn compile_with_work<E>(
        source: &JsString,
        ignore_case: bool,
        multiline: bool,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, RegExpUnicodeSetCompileError<E>> {
        let Some(source) = skipped_assertions(source, &mut charge)? else {
            return Ok(None);
        };
        if has_live_assertions(&source, &mut charge)? {
            return Ok(None);
        }
        macro_rules! attempt {
            ($matcher:ty, $variant:ident) => {
                if let Some(plan) =
                    <$matcher>::compile_with_work(&source, ignore_case, multiline, &mut charge)?
                {
                    return Ok(Some(Self {
                        body: PlainBody::$variant(plan),
                    }));
                }
            };
        }
        attempt!(
            crate::RegExpUnicodeRepeatedStringClassCaptureMatcher,
            Simple
        );
        attempt!(
            crate::RegExpUnicodeBoundedStringClassCaptureMatcher,
            Bounded
        );
        attempt!(
            crate::RegExpUnicodeCountedNullableStringClassCaptureMatcher,
            Nullable
        );
        attempt!(
            crate::RegExpUnicodeTwoRequiredStringClassCaptureMatcher,
            Two
        );
        attempt!(
            crate::RegExpUnicodeRequiredStringClassCaptureMatcher,
            Required
        );
        Ok(None)
    }
    /// Counts all lexical capture slots, including skipped scopes.
    pub fn capture_count(&self) -> usize {
        match &self.body {
            PlainBody::Simple(p) => p.capture_count(),
            PlainBody::Bounded(p) => p.capture_count(),
            PlainBody::Nullable(p) => p.capture_count(),
            PlainBody::Two(p) => p.capture_count(),
            PlainBody::Required(p) => p.capture_count(),
        }
    }
    /// Projects original slots; skipped scopes remain undefined.
    pub fn capture_range(
        &self,
        index: usize,
        found: &RegExpUnicodeRepeatedStringClassMatch,
    ) -> Option<Range<usize>> {
        match &self.body {
            PlainBody::Simple(p) => p.capture_range(index, found),
            PlainBody::Bounded(p) => p.capture_range(index, found),
            PlainBody::Nullable(p) => p.capture_range(index, found),
            PlainBody::Two(p) => p.capture_range(index, found),
            PlainBody::Required(p) => p.capture_range(index, found),
        }
    }
    /// Executes the class/count plan with its original search and empty-body ordering.
    pub fn find_with_captures_and_work<E>(
        &self,
        input: &JsString,
        requested: usize,
        sticky: bool,
        charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<RegExpUnicodeRepeatedStringClassMatch>, RegExpUnicodeMatchError<E>> {
        match &self.body {
            PlainBody::Simple(p) => p
                .find_with_captures_and_work(input, requested, sticky, charge)
                .map_err(RegExpUnicodeMatchError::Work),
            PlainBody::Bounded(p) => p
                .find_with_captures_and_work(input, requested, sticky, charge)
                .map_err(RegExpUnicodeMatchError::Work),
            PlainBody::Nullable(p) => p
                .find_with_captures_and_work(input, requested, sticky, charge)
                .map_err(RegExpUnicodeMatchError::Work),
            PlainBody::Two(p) => p
                .find_with_captures_and_work(input, requested, sticky, charge)
                .map_err(RegExpUnicodeMatchError::Work),
            PlainBody::Required(p) => {
                p.find_with_captures_and_work(input, requested, sticky, charge)
            }
        }
    }
}

fn has_live_assertions<E>(
    source: &JsString,
    charge: &mut impl FnMut(usize) -> Result<(), E>,
) -> Result<bool, RegExpUnicodeSetCompileError<E>> {
    let units = source.code_units();
    charge(units.len().saturating_mul(8).saturating_add(16))
        .map_err(RegExpUnicodeSetCompileError::Work)?;
    let mut at = 0;
    while at < units.len() {
        match units[at] {
            0x5b => {
                let Some(end) = super::class_end(units, at) else {
                    return Ok(true);
                };
                at = end;
            }
            0x5e | 0x24 => return Ok(true),
            0x5c => {
                if units.get(at + 1).is_some_and(|u| matches!(*u, 0x62 | 0x42)) {
                    return Ok(true);
                }
                at = (at + 2).min(units.len());
            }
            _ => at += 1,
        }
    }
    Ok(false)
}

/// One class whose outside predicates all occur in exact-zero scopes.
///
/// Requires validated v/vi input and normalized capture names. The class occurs
/// once, either directly or through exact-one counts. Skipped captures remain
/// undefined; ordinary enclosing and point captures preserve participation.
/// Every class leaf and remaining term retains the single-class proof.
/// https://262.ecma-international.org/17.0/#sec-repeatmatcher
#[derive(Clone, Debug)]
pub struct RegExpUnicodeUnassertedZeroCountSingleStringClassCaptureMatcher {
    plan: crate::RegExpUnicodeSingleStringClassCaptureMatcher,
}
impl RegExpUnicodeUnassertedZeroCountSingleStringClassCaptureMatcher {
    /// Proves one class with skipped outside predicates and exact-one class counts.
    pub fn compile_with_work<E>(
        source: &JsString,
        ignore_case: bool,
        multiline: bool,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, RegExpUnicodeSetCompileError<E>> {
        let Some(source) = skipped_assertions_with_shape(source, true, &mut charge)? else {
            return Ok(None);
        };
        if has_live_assertions(&source, &mut charge)? {
            return Ok(None);
        }
        let plan = if let Some(plan) =
            crate::RegExpUnicodeSingleStringClassCaptureMatcher::compile_with_work(
                &source,
                ignore_case,
                multiline,
                &mut charge,
            )? {
            Some(plan)
        } else {
            crate::RegExpUnicodeSingleStringClassCaptureMatcher::compile_exact_single_counts_with_work(&source,ignore_case,multiline,&mut charge)?
        };
        Ok(plan.map(|plan| Self { plan }))
    }
    /// Counts every original lexical capture, including skipped scopes.
    pub fn capture_count(&self) -> usize {
        self.plan.capture_count()
    }
    /// Projects lexical scopes on the class range or their original empty points.
    pub fn capture_range(
        &self,
        index: usize,
        found: &RegExpUnicodeRepeatedStringClassMatch,
    ) -> Option<Range<usize>> {
        self.plan.capture_range(index, &found.range)
    }
    /// Executes the single class with original Unicode search and member priority.
    pub fn find_with_captures_and_work<E>(
        &self,
        input: &JsString,
        requested: usize,
        sticky: bool,
        charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<RegExpUnicodeRepeatedStringClassMatch>, RegExpUnicodeMatchError<E>> {
        self.plan
            .find_with_captures_and_work(input, requested, sticky, charge)
            .map(|found| {
                found.map(|range| RegExpUnicodeRepeatedStringClassMatch {
                    range: range.clone(),
                    last_iteration: Some(range),
                    class_selected: true,
                })
            })
            .map_err(RegExpUnicodeMatchError::Work)
    }
}

/// One class with skipped scopes and unconditional live outside predicates.
///
/// Requires validated v/vi input and normalized capture names. The class occurs
/// once, either directly or through exact-one counts. Skipped captures remain
/// undefined; ordinary enclosing and point captures preserve participation.
/// Every class leaf and remaining term retains the single-class proof.
/// https://262.ecma-international.org/17.0/#sec-repeatmatcher
#[derive(Clone, Debug)]
pub struct RegExpUnicodeZeroCountAssertedSingleStringClassCaptureMatcher {
    plan: crate::RegExpUnicodeSingleStringClassCaptureMatcher,
}
impl RegExpUnicodeZeroCountAssertedSingleStringClassCaptureMatcher {
    /// Proves skipped scopes, live predicates and a bare or exact-one class path.
    pub fn compile_with_work<E>(
        source: &JsString,
        ignore_case: bool,
        multiline: bool,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, RegExpUnicodeSetCompileError<E>> {
        let Some(source) = skipped_assertions_with_shape(source, true, &mut charge)? else {
            return Ok(None);
        };
        if !has_live_assertions(&source, &mut charge)? {
            return Ok(None);
        }
        charge(source.len().saturating_mul(24).saturating_add(128))
            .map_err(RegExpUnicodeSetCompileError::Work)?;
        let source = super::normalize_ones(source.code_units());
        let plan=if let Some(plan)=crate::RegExpUnicodeSingleStringClassCaptureMatcher::compile_unconditional_assertions_with_work(&source,ignore_case,multiline,&mut charge)? {
            Some(plan)
        } else {
            crate::RegExpUnicodeSingleStringClassCaptureMatcher::compile_reversed_line_assertions_with_work(&source,ignore_case,multiline,&mut charge)?
        };
        Ok(plan.map(|plan| Self { plan }))
    }
    /// Counts every original lexical capture, including skipped scopes.
    pub fn capture_count(&self) -> usize {
        self.plan.capture_count()
    }
    /// Projects lexical scopes on the class range or their original empty points.
    pub fn capture_range(
        &self,
        index: usize,
        found: &RegExpUnicodeRepeatedStringClassMatch,
    ) -> Option<Range<usize>> {
        self.plan.capture_range(index, &found.range)
    }
    /// Executes the single class with original Unicode search and member priority.
    pub fn find_with_captures_and_work<E>(
        &self,
        input: &JsString,
        requested: usize,
        sticky: bool,
        charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<RegExpUnicodeRepeatedStringClassMatch>, RegExpUnicodeMatchError<E>> {
        self.plan
            .find_with_captures_and_work(input, requested, sticky, charge)
            .map(|found| {
                found.map(|range| RegExpUnicodeRepeatedStringClassMatch {
                    range: range.clone(),
                    last_iteration: Some(range),
                    class_selected: true,
                })
            })
            .map_err(RegExpUnicodeMatchError::Work)
    }
}

#[derive(Clone, Debug)]
enum UnassertedAlternativeBody {
    Nonempty(crate::RegExpUnicodeNullableStringClassCaptureMatcher),
    Empty(crate::RegExpUnicodeEmptyStringClassCaptureMatcher),
}
/// One class/empty alternative with every outside predicate in a skipped scope.
///
/// Requires validated v/vi input and normalized capture names. Both original
/// alternatives and every class leaf must be proved, including an unselected
/// class branch. Bare or exact-one class paths retain original source order and
/// required empty class identity. Skipped predicate captures remain undefined.
/// https://262.ecma-international.org/17.0/#sec-disjunction
#[derive(Clone, Debug)]
pub struct RegExpUnicodeUnassertedZeroCountStringClassAlternativeMatcher {
    body: UnassertedAlternativeBody,
}
impl RegExpUnicodeUnassertedZeroCountStringClassAlternativeMatcher {
    /// Proves skipped predicates before the complete class/empty choice.
    pub fn compile_with_work<E>(
        source: &JsString,
        ignore_case: bool,
        multiline: bool,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, RegExpUnicodeSetCompileError<E>> {
        let Some(source) = skipped_assertions_with_shape(source, true, &mut charge)? else {
            return Ok(None);
        };
        if has_live_assertions(&source, &mut charge)? {
            return Ok(None);
        }
        macro_rules! attempt {
            ($matcher:ty,$method:ident,$variant:ident) => {
                if let Some(plan) =
                    <$matcher>::$method(&source, ignore_case, multiline, &mut charge)?
                {
                    return Ok(Some(Self {
                        body: UnassertedAlternativeBody::$variant(plan),
                    }));
                }
            };
        }
        attempt!(
            crate::RegExpUnicodeNullableStringClassCaptureMatcher,
            compile_with_work,
            Nonempty
        );
        attempt!(
            crate::RegExpUnicodeNullableStringClassCaptureMatcher,
            compile_grouped_with_work,
            Nonempty
        );
        attempt!(
            crate::RegExpUnicodeEmptyStringClassCaptureMatcher,
            compile_with_work,
            Empty
        );
        attempt!(
            crate::RegExpUnicodeEmptyStringClassCaptureMatcher,
            compile_grouped_with_work,
            Empty
        );
        attempt!(
            crate::RegExpUnicodeNullableStringClassCaptureMatcher,
            compile_exact_single_counts_with_work,
            Nonempty
        );
        attempt!(
            crate::RegExpUnicodeEmptyStringClassCaptureMatcher,
            compile_exact_single_counts_with_work,
            Empty
        );
        Ok(None)
    }
    /// Counts original lexical slots, including both branches and skipped scopes.
    pub fn capture_count(&self) -> usize {
        match &self.body {
            UnassertedAlternativeBody::Nonempty(plan) => plan.capture_count(),
            UnassertedAlternativeBody::Empty(plan) => plan.capture_count(),
        }
    }
    /// Projects original captures using the selected class/empty branch identity.
    pub fn capture_range(
        &self,
        index: usize,
        found: &RegExpUnicodeRepeatedStringClassMatch,
    ) -> Option<Range<usize>> {
        match &self.body {
            UnassertedAlternativeBody::Nonempty(plan) => {
                plan.capture_range(index, Some(&found.range))
            }
            UnassertedAlternativeBody::Empty(plan) => {
                plan.capture_range(index, &found.range, found.class_selected)
            }
        }
    }
    /// Selects the first original alternative at the initial Unicode boundary.
    pub fn find_with_captures_and_work<E>(
        &self,
        input: &JsString,
        requested: usize,
        sticky: bool,
        charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<RegExpUnicodeRepeatedStringClassMatch>, RegExpUnicodeMatchError<E>> {
        let found = match &self.body {
            UnassertedAlternativeBody::Nonempty(plan) => plan
                .find_with_captures_and_work(input, requested, sticky, charge)
                .map(|found| {
                    found.map(|found| RegExpUnicodeRepeatedStringClassMatch {
                        class_selected: !found.range.is_empty(),
                        last_iteration: Some(found.range.clone()),
                        range: found.range,
                    })
                }),
            UnassertedAlternativeBody::Empty(plan) => plan
                .find_with_captures_and_work(input, requested, sticky, charge)
                .map(|found| {
                    found.map(|found| RegExpUnicodeRepeatedStringClassMatch {
                        class_selected: found.class_selected,
                        last_iteration: Some(found.range.clone()),
                        range: found.range,
                    })
                }),
        };
        found.map_err(RegExpUnicodeMatchError::Work)
    }
}

#[derive(Clone, Debug)]
enum AssertedAlternativeBody {
    Root(crate::RegExpUnicodeAssertedStringClassAlternativeMatcher),
    Surrounding(crate::RegExpUnicodeSurroundingAssertedStringClassAlternativeMatcher),
}
/// One class/empty choice with skipped scopes and remaining live predicates.
///
/// Requires validated v/vi input and normalized capture names. Every original
/// branch and class leaf retains its full proof. Bare/exact-one class paths
/// preserve lexical slots, outside predicates and ordered empty/class selection.
/// https://262.ecma-international.org/17.0/#sec-disjunction
#[derive(Clone, Debug)]
pub struct RegExpUnicodeZeroCountAssertedStringClassAlternativeMatcher {
    body: AssertedAlternativeBody,
}
impl RegExpUnicodeZeroCountAssertedStringClassAlternativeMatcher {
    /// Proves skipped scopes before complete branch and outside-spine proofs.
    pub fn compile_with_work<E>(
        source: &JsString,
        ignore_case: bool,
        multiline: bool,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, RegExpUnicodeSetCompileError<E>> {
        let Some(source) = skipped_assertions_with_shape(source, true, &mut charge)? else {
            return Ok(None);
        };
        if !has_live_assertions(&source, &mut charge)? {
            return Ok(None);
        }
        charge(source.len().saturating_mul(24).saturating_add(128))
            .map_err(RegExpUnicodeSetCompileError::Work)?;
        let source = super::normalize_ones(source.code_units());
        macro_rules! attempt {
            ($method:ident) => {
                if let Some(plan) =
                    crate::RegExpUnicodeAssertedStringClassAlternativeMatcher::$method(
                        &source,
                        ignore_case,
                        multiline,
                        &mut charge,
                    )?
                {
                    return Ok(Some(Self {
                        body: AssertedAlternativeBody::Root(plan),
                    }));
                }
            };
        }
        attempt!(compile_with_work);
        attempt!(compile_grouped_with_work);
        if let Some(plan) =
            crate::RegExpUnicodeSurroundingAssertedStringClassAlternativeMatcher::compile_with_work(
                &source,
                ignore_case,
                multiline,
                &mut charge,
            )?
        {
            return Ok(Some(Self {
                body: AssertedAlternativeBody::Surrounding(plan),
            }));
        }
        attempt!(compile_surrounding_line_assertions_with_work);
        attempt!(compile_surrounding_word_assertions_with_work);
        attempt!(compile_nested_assertion_wrappers_with_work);
        attempt!(compile_reversed_surrounding_line_assertions_with_work);
        attempt!(compile_nested_endpoint_assertion_wrappers_with_work);
        Ok(None)
    }
    /// Counts every original lexical slot, including skipped and unselected scopes.
    pub fn capture_count(&self) -> usize {
        match &self.body {
            AssertedAlternativeBody::Root(plan) => plan.capture_count(),
            AssertedAlternativeBody::Surrounding(plan) => plan.capture_count(),
        }
    }
    /// Projects original slots with actual endpoints and explicit branch identity.
    pub fn capture_range(
        &self,
        index: usize,
        found: &RegExpUnicodeRepeatedStringClassMatch,
    ) -> Option<Range<usize>> {
        let branch = crate::RegExpUnicodeStringClassBranchMatch {
            range: found.range.clone(),
            class_selected: found.class_selected,
        };
        match &self.body {
            AssertedAlternativeBody::Root(plan) => plan.capture_range(index, &branch),
            AssertedAlternativeBody::Surrounding(plan) => plan.capture_range(index, &branch),
        }
    }
    /// Searches both proved branches and applies original source order at ties.
    pub fn find_with_captures_and_work<E>(
        &self,
        input: &JsString,
        requested: usize,
        sticky: bool,
        charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<RegExpUnicodeRepeatedStringClassMatch>, RegExpUnicodeMatchError<E>> {
        let found = match &self.body {
            AssertedAlternativeBody::Root(plan) => {
                plan.find_with_captures_and_work(input, requested, sticky, charge)
            }
            AssertedAlternativeBody::Surrounding(plan) => {
                plan.find_with_captures_and_work(input, requested, sticky, charge)
            }
        };
        found
            .map(|found| {
                found.map(|found| RegExpUnicodeRepeatedStringClassMatch {
                    class_selected: found.class_selected,
                    last_iteration: Some(found.range.clone()),
                    range: found.range,
                })
            })
            .map_err(RegExpUnicodeMatchError::Work)
    }
}

/// String-class repetition with zero-minimum outside assertion scopes.
///
/// Requires validated v/vi input and normalized capture names. At least one
/// outside assertion is enclosed in a zero-minimum group, and other outside
/// assertions satisfy the unconditional repetition proof. Skipped scopes never
/// participate: a zero-width body cannot make optional repetition progress.
/// Their captures remain undefined even if their assertions
/// would succeed. The class and every live term retain their complete proofs.
/// https://262.ecma-international.org/17.0/#sec-repeatmatcher
#[derive(Clone, Debug)]
pub struct RegExpUnicodeZeroMinimumAssertedStringClassCaptureMatcher {
    body: Body,
}
impl RegExpUnicodeZeroMinimumAssertedStringClassCaptureMatcher {
    /// Proves zero-minimum outside scopes before compiling the live repetition.
    pub fn compile_with_work<E>(
        source: &JsString,
        ignore_case: bool,
        multiline: bool,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, RegExpUnicodeSetCompileError<E>> {
        let Some(source) = skipped_predicates_with_shape(source, false, true, &mut charge)? else {
            return Ok(None);
        };
        macro_rules! attempt {
            ($matcher:ty, $variant:ident) => {
                if let Some(plan) =
                    <$matcher>::compile_with_work(&source, ignore_case, multiline, &mut charge)?
                {
                    return Ok(Some(Self {
                        body: Body::$variant(plan),
                    }));
                }
            };
        }
        attempt!(
            RegExpUnicodeAssertedRepeatedStringClassCaptureMatcher,
            Nonnullable
        );
        attempt!(
            RegExpUnicodeAssertedNullableStringClassCaptureMatcher,
            Nullable
        );
        attempt!(
            RegExpUnicodeAssertedTwoRequiredNullableStringClassCaptureMatcher,
            Two
        );
        attempt!(
            RegExpUnicodeAssertedCountedNullableStringClassCaptureMatcher,
            Counted
        );
        attempt!(
            RegExpUnicodeAssertedCountedNullableScalarStringClassCaptureMatcher,
            Scalar
        );
        Ok(None)
    }
    /// Counts every original lexical capture, including skipped groups.
    pub fn capture_count(&self) -> usize {
        match &self.body {
            Body::Nonnullable(p) => p.capture_count(),
            Body::Nullable(p) => p.capture_count(),
            Body::Two(p) => p.capture_count(),
            Body::Counted(p) => p.capture_count(),
            Body::Scalar(p) => p.capture_count(),
        }
    }
    /// Projects original slots; exact-zero groups retain undefined captures.
    pub fn capture_range(
        &self,
        index: usize,
        found: &RegExpUnicodeRepeatedStringClassMatch,
    ) -> Option<Range<usize>> {
        match &self.body {
            Body::Nonnullable(p) => p.capture_range(index, found),
            Body::Nullable(p) => p.capture_range(index, found),
            Body::Two(p) => p.capture_range(index, found),
            Body::Counted(p) => p.capture_range(index, found),
            Body::Scalar(p) => p.capture_range(index, found),
        }
    }
    /// Executes only live predicates and the proved repetition continuation.
    pub fn find_with_captures_and_work<E>(
        &self,
        input: &JsString,
        requested: usize,
        sticky: bool,
        charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<RegExpUnicodeRepeatedStringClassMatch>, RegExpUnicodeMatchError<E>> {
        match &self.body {
            Body::Nonnullable(p) => p.find_with_captures_and_work(input, requested, sticky, charge),
            Body::Nullable(p) => p.find_with_captures_and_work(input, requested, sticky, charge),
            Body::Two(p) => p.find_with_captures_and_work(input, requested, sticky, charge),
            Body::Counted(p) => p.find_with_captures_and_work(input, requested, sticky, charge),
            Body::Scalar(p) => p.find_with_captures_and_work(input, requested, sticky, charge),
        }
    }
}

/// A repetition whose only outside assertions occur in zero-minimum scopes.
///
/// Requires validated v/vi input and normalized capture names. All assertion
/// terms must be skipped; otherwise the separate asserted proof applies. The
/// remaining class and pure-empty continuation keep their complete proofs and
/// original capture slots. Zero-minimum predicate scopes cannot make progress, so their captures never
/// participate (22.2.2.3).
/// https://262.ecma-international.org/17.0/#sec-repeatmatcher
#[derive(Clone, Debug)]
pub struct RegExpUnicodeUnassertedZeroMinimumStringClassCaptureMatcher {
    body: PlainBody,
}
impl RegExpUnicodeUnassertedZeroMinimumStringClassCaptureMatcher {
    /// Proves skipped assertions and an assertion-free live repetition.
    pub fn compile_with_work<E>(
        source: &JsString,
        ignore_case: bool,
        multiline: bool,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, RegExpUnicodeSetCompileError<E>> {
        let Some(source) = skipped_predicates_with_shape(source, false, true, &mut charge)? else {
            return Ok(None);
        };
        if has_live_assertions(&source, &mut charge)? {
            return Ok(None);
        }
        macro_rules! attempt {
            ($matcher:ty, $variant:ident) => {
                if let Some(plan) =
                    <$matcher>::compile_with_work(&source, ignore_case, multiline, &mut charge)?
                {
                    return Ok(Some(Self {
                        body: PlainBody::$variant(plan),
                    }));
                }
            };
        }
        attempt!(
            crate::RegExpUnicodeRepeatedStringClassCaptureMatcher,
            Simple
        );
        attempt!(
            crate::RegExpUnicodeBoundedStringClassCaptureMatcher,
            Bounded
        );
        attempt!(
            crate::RegExpUnicodeCountedNullableStringClassCaptureMatcher,
            Nullable
        );
        attempt!(
            crate::RegExpUnicodeTwoRequiredStringClassCaptureMatcher,
            Two
        );
        attempt!(
            crate::RegExpUnicodeRequiredStringClassCaptureMatcher,
            Required
        );
        Ok(None)
    }
    /// Counts all lexical capture slots, including skipped scopes.
    pub fn capture_count(&self) -> usize {
        match &self.body {
            PlainBody::Simple(p) => p.capture_count(),
            PlainBody::Bounded(p) => p.capture_count(),
            PlainBody::Nullable(p) => p.capture_count(),
            PlainBody::Two(p) => p.capture_count(),
            PlainBody::Required(p) => p.capture_count(),
        }
    }
    /// Projects original slots; skipped scopes remain undefined.
    pub fn capture_range(
        &self,
        index: usize,
        found: &RegExpUnicodeRepeatedStringClassMatch,
    ) -> Option<Range<usize>> {
        match &self.body {
            PlainBody::Simple(p) => p.capture_range(index, found),
            PlainBody::Bounded(p) => p.capture_range(index, found),
            PlainBody::Nullable(p) => p.capture_range(index, found),
            PlainBody::Two(p) => p.capture_range(index, found),
            PlainBody::Required(p) => p.capture_range(index, found),
        }
    }
    /// Executes the class/count plan with its original search and empty-body ordering.
    pub fn find_with_captures_and_work<E>(
        &self,
        input: &JsString,
        requested: usize,
        sticky: bool,
        charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<RegExpUnicodeRepeatedStringClassMatch>, RegExpUnicodeMatchError<E>> {
        match &self.body {
            PlainBody::Simple(p) => p
                .find_with_captures_and_work(input, requested, sticky, charge)
                .map_err(RegExpUnicodeMatchError::Work),
            PlainBody::Bounded(p) => p
                .find_with_captures_and_work(input, requested, sticky, charge)
                .map_err(RegExpUnicodeMatchError::Work),
            PlainBody::Nullable(p) => p
                .find_with_captures_and_work(input, requested, sticky, charge)
                .map_err(RegExpUnicodeMatchError::Work),
            PlainBody::Two(p) => p
                .find_with_captures_and_work(input, requested, sticky, charge)
                .map_err(RegExpUnicodeMatchError::Work),
            PlainBody::Required(p) => {
                p.find_with_captures_and_work(input, requested, sticky, charge)
            }
        }
    }
}

/// One class whose outside predicates all occur in zero-minimum scopes.
///
/// Requires validated v/vi input and normalized capture names. The class occurs
/// once, either directly or through exact-one counts. Skipped captures remain
/// undefined; ordinary enclosing and point captures preserve participation.
/// Every class leaf and remaining term retains the single-class proof.
/// https://262.ecma-international.org/17.0/#sec-repeatmatcher
#[derive(Clone, Debug)]
pub struct RegExpUnicodeUnassertedZeroMinimumSingleStringClassCaptureMatcher {
    plan: crate::RegExpUnicodeSingleStringClassCaptureMatcher,
}
impl RegExpUnicodeUnassertedZeroMinimumSingleStringClassCaptureMatcher {
    /// Proves one class with skipped outside predicates and exact-one class counts.
    pub fn compile_with_work<E>(
        source: &JsString,
        ignore_case: bool,
        multiline: bool,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, RegExpUnicodeSetCompileError<E>> {
        let Some(source) = skipped_predicates_with_shape(source, true, true, &mut charge)? else {
            return Ok(None);
        };
        if has_live_assertions(&source, &mut charge)? {
            return Ok(None);
        }
        let plan = if let Some(plan) =
            crate::RegExpUnicodeSingleStringClassCaptureMatcher::compile_with_work(
                &source,
                ignore_case,
                multiline,
                &mut charge,
            )? {
            Some(plan)
        } else {
            crate::RegExpUnicodeSingleStringClassCaptureMatcher::compile_exact_single_counts_with_work(&source,ignore_case,multiline,&mut charge)?
        };
        Ok(plan.map(|plan| Self { plan }))
    }
    /// Counts every original lexical capture, including skipped scopes.
    pub fn capture_count(&self) -> usize {
        self.plan.capture_count()
    }
    /// Projects lexical scopes on the class range or their original empty points.
    pub fn capture_range(
        &self,
        index: usize,
        found: &RegExpUnicodeRepeatedStringClassMatch,
    ) -> Option<Range<usize>> {
        self.plan.capture_range(index, &found.range)
    }
    /// Executes the single class with original Unicode search and member priority.
    pub fn find_with_captures_and_work<E>(
        &self,
        input: &JsString,
        requested: usize,
        sticky: bool,
        charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<RegExpUnicodeRepeatedStringClassMatch>, RegExpUnicodeMatchError<E>> {
        self.plan
            .find_with_captures_and_work(input, requested, sticky, charge)
            .map(|found| {
                found.map(|range| RegExpUnicodeRepeatedStringClassMatch {
                    range: range.clone(),
                    last_iteration: Some(range),
                    class_selected: true,
                })
            })
            .map_err(RegExpUnicodeMatchError::Work)
    }
}

/// One class with skipped scopes and unconditional live outside predicates.
///
/// Requires validated v/vi input and normalized capture names. The class occurs
/// once, either directly or through exact-one counts. Skipped captures remain
/// undefined; ordinary enclosing and point captures preserve participation.
/// Every class leaf and remaining term retains the single-class proof.
/// https://262.ecma-international.org/17.0/#sec-repeatmatcher
#[derive(Clone, Debug)]
pub struct RegExpUnicodeZeroMinimumAssertedSingleStringClassCaptureMatcher {
    plan: crate::RegExpUnicodeSingleStringClassCaptureMatcher,
}
impl RegExpUnicodeZeroMinimumAssertedSingleStringClassCaptureMatcher {
    /// Proves skipped scopes, live predicates and a bare or exact-one class path.
    pub fn compile_with_work<E>(
        source: &JsString,
        ignore_case: bool,
        multiline: bool,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, RegExpUnicodeSetCompileError<E>> {
        let Some(source) = skipped_predicates_with_shape(source, true, true, &mut charge)? else {
            return Ok(None);
        };
        if !has_live_assertions(&source, &mut charge)? {
            return Ok(None);
        }
        charge(source.len().saturating_mul(24).saturating_add(128))
            .map_err(RegExpUnicodeSetCompileError::Work)?;
        let source = super::normalize_ones(source.code_units());
        let plan=if let Some(plan)=crate::RegExpUnicodeSingleStringClassCaptureMatcher::compile_unconditional_assertions_with_work(&source,ignore_case,multiline,&mut charge)? {
            Some(plan)
        } else {
            crate::RegExpUnicodeSingleStringClassCaptureMatcher::compile_reversed_line_assertions_with_work(&source,ignore_case,multiline,&mut charge)?
        };
        Ok(plan.map(|plan| Self { plan }))
    }
    /// Counts every original lexical capture, including skipped scopes.
    pub fn capture_count(&self) -> usize {
        self.plan.capture_count()
    }
    /// Projects lexical scopes on the class range or their original empty points.
    pub fn capture_range(
        &self,
        index: usize,
        found: &RegExpUnicodeRepeatedStringClassMatch,
    ) -> Option<Range<usize>> {
        self.plan.capture_range(index, &found.range)
    }
    /// Executes the single class with original Unicode search and member priority.
    pub fn find_with_captures_and_work<E>(
        &self,
        input: &JsString,
        requested: usize,
        sticky: bool,
        charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<RegExpUnicodeRepeatedStringClassMatch>, RegExpUnicodeMatchError<E>> {
        self.plan
            .find_with_captures_and_work(input, requested, sticky, charge)
            .map(|found| {
                found.map(|range| RegExpUnicodeRepeatedStringClassMatch {
                    range: range.clone(),
                    last_iteration: Some(range),
                    class_selected: true,
                })
            })
            .map_err(RegExpUnicodeMatchError::Work)
    }
}

/// One class/empty alternative with every outside predicate in a skipped scope.
///
/// Requires validated v/vi input and normalized capture names. Both original
/// alternatives and every class leaf must be proved, including an unselected
/// class branch. Bare or exact-one class paths retain original source order and
/// required empty class identity. Skipped predicate captures remain undefined.
/// https://262.ecma-international.org/17.0/#sec-disjunction
#[derive(Clone, Debug)]
pub struct RegExpUnicodeUnassertedZeroMinimumStringClassAlternativeMatcher {
    body: UnassertedAlternativeBody,
}
impl RegExpUnicodeUnassertedZeroMinimumStringClassAlternativeMatcher {
    /// Proves skipped predicates before the complete class/empty choice.
    pub fn compile_with_work<E>(
        source: &JsString,
        ignore_case: bool,
        multiline: bool,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, RegExpUnicodeSetCompileError<E>> {
        let Some(source) = skipped_predicates_with_shape(source, true, true, &mut charge)? else {
            return Ok(None);
        };
        if has_live_assertions(&source, &mut charge)? {
            return Ok(None);
        }
        macro_rules! attempt {
            ($matcher:ty,$method:ident,$variant:ident) => {
                if let Some(plan) =
                    <$matcher>::$method(&source, ignore_case, multiline, &mut charge)?
                {
                    return Ok(Some(Self {
                        body: UnassertedAlternativeBody::$variant(plan),
                    }));
                }
            };
        }
        attempt!(
            crate::RegExpUnicodeNullableStringClassCaptureMatcher,
            compile_with_work,
            Nonempty
        );
        attempt!(
            crate::RegExpUnicodeNullableStringClassCaptureMatcher,
            compile_grouped_with_work,
            Nonempty
        );
        attempt!(
            crate::RegExpUnicodeEmptyStringClassCaptureMatcher,
            compile_with_work,
            Empty
        );
        attempt!(
            crate::RegExpUnicodeEmptyStringClassCaptureMatcher,
            compile_grouped_with_work,
            Empty
        );
        attempt!(
            crate::RegExpUnicodeNullableStringClassCaptureMatcher,
            compile_exact_single_counts_with_work,
            Nonempty
        );
        attempt!(
            crate::RegExpUnicodeEmptyStringClassCaptureMatcher,
            compile_exact_single_counts_with_work,
            Empty
        );
        Ok(None)
    }
    /// Counts original lexical slots, including both branches and skipped scopes.
    pub fn capture_count(&self) -> usize {
        match &self.body {
            UnassertedAlternativeBody::Nonempty(plan) => plan.capture_count(),
            UnassertedAlternativeBody::Empty(plan) => plan.capture_count(),
        }
    }
    /// Projects original captures using the selected class/empty branch identity.
    pub fn capture_range(
        &self,
        index: usize,
        found: &RegExpUnicodeRepeatedStringClassMatch,
    ) -> Option<Range<usize>> {
        match &self.body {
            UnassertedAlternativeBody::Nonempty(plan) => {
                plan.capture_range(index, Some(&found.range))
            }
            UnassertedAlternativeBody::Empty(plan) => {
                plan.capture_range(index, &found.range, found.class_selected)
            }
        }
    }
    /// Selects the first original alternative at the initial Unicode boundary.
    pub fn find_with_captures_and_work<E>(
        &self,
        input: &JsString,
        requested: usize,
        sticky: bool,
        charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<RegExpUnicodeRepeatedStringClassMatch>, RegExpUnicodeMatchError<E>> {
        let found = match &self.body {
            UnassertedAlternativeBody::Nonempty(plan) => plan
                .find_with_captures_and_work(input, requested, sticky, charge)
                .map(|found| {
                    found.map(|found| RegExpUnicodeRepeatedStringClassMatch {
                        class_selected: !found.range.is_empty(),
                        last_iteration: Some(found.range.clone()),
                        range: found.range,
                    })
                }),
            UnassertedAlternativeBody::Empty(plan) => plan
                .find_with_captures_and_work(input, requested, sticky, charge)
                .map(|found| {
                    found.map(|found| RegExpUnicodeRepeatedStringClassMatch {
                        class_selected: found.class_selected,
                        last_iteration: Some(found.range.clone()),
                        range: found.range,
                    })
                }),
        };
        found.map_err(RegExpUnicodeMatchError::Work)
    }
}

/// One class/empty choice with skipped scopes and remaining live predicates.
///
/// Requires validated v/vi input and normalized capture names. Every original
/// branch and class leaf retains its full proof. Bare/exact-one class paths
/// preserve lexical slots, outside predicates and ordered empty/class selection.
/// https://262.ecma-international.org/17.0/#sec-disjunction
#[derive(Clone, Debug)]
pub struct RegExpUnicodeZeroMinimumAssertedStringClassAlternativeMatcher {
    body: AssertedAlternativeBody,
}
impl RegExpUnicodeZeroMinimumAssertedStringClassAlternativeMatcher {
    /// Proves skipped scopes before complete branch and outside-spine proofs.
    pub fn compile_with_work<E>(
        source: &JsString,
        ignore_case: bool,
        multiline: bool,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, RegExpUnicodeSetCompileError<E>> {
        let Some(source) = skipped_predicates_with_shape(source, true, true, &mut charge)? else {
            return Ok(None);
        };
        if !has_live_assertions(&source, &mut charge)? {
            return Ok(None);
        }
        charge(source.len().saturating_mul(24).saturating_add(128))
            .map_err(RegExpUnicodeSetCompileError::Work)?;
        let source = super::normalize_ones(source.code_units());
        macro_rules! attempt {
            ($method:ident) => {
                if let Some(plan) =
                    crate::RegExpUnicodeAssertedStringClassAlternativeMatcher::$method(
                        &source,
                        ignore_case,
                        multiline,
                        &mut charge,
                    )?
                {
                    return Ok(Some(Self {
                        body: AssertedAlternativeBody::Root(plan),
                    }));
                }
            };
        }
        attempt!(compile_with_work);
        attempt!(compile_grouped_with_work);
        if let Some(plan) =
            crate::RegExpUnicodeSurroundingAssertedStringClassAlternativeMatcher::compile_with_work(
                &source,
                ignore_case,
                multiline,
                &mut charge,
            )?
        {
            return Ok(Some(Self {
                body: AssertedAlternativeBody::Surrounding(plan),
            }));
        }
        attempt!(compile_surrounding_line_assertions_with_work);
        attempt!(compile_surrounding_word_assertions_with_work);
        attempt!(compile_nested_assertion_wrappers_with_work);
        attempt!(compile_reversed_surrounding_line_assertions_with_work);
        attempt!(compile_nested_endpoint_assertion_wrappers_with_work);
        Ok(None)
    }
    /// Counts every original lexical slot, including skipped and unselected scopes.
    pub fn capture_count(&self) -> usize {
        match &self.body {
            AssertedAlternativeBody::Root(plan) => plan.capture_count(),
            AssertedAlternativeBody::Surrounding(plan) => plan.capture_count(),
        }
    }
    /// Projects original slots with actual endpoints and explicit branch identity.
    pub fn capture_range(
        &self,
        index: usize,
        found: &RegExpUnicodeRepeatedStringClassMatch,
    ) -> Option<Range<usize>> {
        let branch = crate::RegExpUnicodeStringClassBranchMatch {
            range: found.range.clone(),
            class_selected: found.class_selected,
        };
        match &self.body {
            AssertedAlternativeBody::Root(plan) => plan.capture_range(index, &branch),
            AssertedAlternativeBody::Surrounding(plan) => plan.capture_range(index, &branch),
        }
    }
    /// Searches both proved branches and applies original source order at ties.
    pub fn find_with_captures_and_work<E>(
        &self,
        input: &JsString,
        requested: usize,
        sticky: bool,
        charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<RegExpUnicodeRepeatedStringClassMatch>, RegExpUnicodeMatchError<E>> {
        let found = match &self.body {
            AssertedAlternativeBody::Root(plan) => {
                plan.find_with_captures_and_work(input, requested, sticky, charge)
            }
            AssertedAlternativeBody::Surrounding(plan) => {
                plan.find_with_captures_and_work(input, requested, sticky, charge)
            }
        };
        found
            .map(|found| {
                found.map(|found| RegExpUnicodeRepeatedStringClassMatch {
                    class_selected: found.class_selected,
                    last_iteration: Some(found.range.clone()),
                    range: found.range,
                })
            })
            .map_err(RegExpUnicodeMatchError::Work)
    }
}

mod predicate_choices {
    include!("regexp_unicode_predicate_choice_string_class.rs");
}
pub use predicate_choices::{RegExpUnicodeTruthTableCountedNullableStringClassCaptureMatcher,
    RegExpUnicodeTruthTableSingleStringClassCaptureMatcher,
    RegExpUnicodeTruthTableStringClassAlternativeMatcher,
    RegExpUnicodeTruthTableRootStringClassAlternativeMatcher,
    RegExpUnicodeTruthTableGroupedStringClassAlternativeMatcher,
    RegExpUnicodeTruthTableExactOneGroupedStringClassAlternativeMatcher,
    RegExpUnicodeTruthTableSurroundingStringClassAlternativeMatcher,
    RegExpUnicodeTruthTableStringClassCaptureMatcher,
    RegExpUnicodeConjunctivePredicateChoiceStringClassAlternativeMatcher,
    RegExpUnicodeConjunctivePredicateChoiceSingleStringClassCaptureMatcher,
    RegExpUnicodeConjunctivePredicateChoiceStringClassCaptureMatcher,
    RegExpUnicodePredicateChoiceStringClassAlternativeMatcher,
    RegExpUnicodePredicateChoiceSingleStringClassCaptureMatcher, RegExpUnicodePredicateChoiceStringClassCaptureMatcher, RegExpUnicodePredicateChoiceStringClassMatch};
