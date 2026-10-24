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
    use RegExpUnicodeSetCompileError::{Capacity, Work};
    let units = source.code_units();
    charge(units.len().saturating_mul(64).saturating_add(128)).map_err(Work)?;
    let Some(hole) = counted_hole(units, |units, at| {
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
                if let Some((_, minimum, delta, _)) = asserted_repeat_count(units, at + 1) {
                    if minimum == 0 && delta == Some(0) {
                        if contains_body {
                            return Ok(None);
                        }
                        starts[open] = true;
                        ends[at + 1] = true;
                    }
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
