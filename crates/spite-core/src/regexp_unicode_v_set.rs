//! Checked character-only UnicodeSets intersection and subtraction (22.2.2.9).

use super::{
    ClassAtom, RegExpUnicodeClassMatcher, SIMPLE_CASE_FOLD, append_escape, class_atom,
    interval_contains, lookup_levels, property_ranges,
};
use crate::JsString;
use std::fmt;

/// A host construction failure from the checked UnicodeSets compiler.
///
/// Valid but unimplemented Pattern shapes return `Ok(None)` instead. Capacity
/// failures are host resource failures, not JavaScript exceptions. New interval
/// buffers reserve fallibly; final shared matcher storage retains the existing
/// infallible `Arc` allocation boundary of the other class constructors.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RegExpUnicodeSetCompileError<E> {
    /// The embedding's opt-in construction work callback failed.
    Work(E),
    /// Interval storage exceeds the allocator's or platform's capacity.
    Capacity,
}

impl<E: fmt::Display> fmt::Display for RegExpUnicodeSetCompileError<E> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Work(error) => write!(formatter, "Unicode set construction work failed: {error}"),
            Self::Capacity => {
                formatter.write_str("Unicode set construction exceeds platform capacity")
            }
        }
    }
}

impl<E: std::error::Error + 'static> std::error::Error for RegExpUnicodeSetCompileError<E> {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Work(error) => Some(error),
            Self::Capacity => None,
        }
    }
}

type Intervals = Vec<(u32, u32)>;
type Checked<T, E> = Result<T, RegExpUnicodeSetCompileError<E>>;
const LIMIT: u32 = 0x110000;
const PROPERTY_COUNT: usize = crate::regexp_script_extensions_data::KIND_COUNT as usize;
const PROPERTY_WORDS: usize = PROPERTY_COUNT.div_ceil(u64::BITS as usize);

#[derive(Clone, Copy, PartialEq, Eq)]
enum Operator {
    Intersection,
    Subtraction,
}

struct Builder<'a, F> {
    units: &'a [u16],
    at: usize,
    ignore_case: bool,
    charge: F,
}

impl RegExpUnicodeClassMatcher {
    /// Compiles a character-only v-mode intersection or subtraction chain.
    ///
    /// The caller must validate the complete Pattern in v/vi mode. Operands are
    /// characters, supported class/property escapes, or nested flat classes and
    /// complements. Optional outside line assertions are admitted. String leaves,
    /// deeper nesting, mixed nested operations and consuming concatenations remain
    /// unimplemented. This does not widen the legacy flat-class constructors.
    ///
    /// Case folding precedes every set operation. Work is opt-in and storage is
    /// proportional to source intervals and the fixed simple-fold table; ranges
    /// are never expanded into individual code points. New buffer reservations
    /// fail as `Capacity`; the final Arc allocation retains its existing boundary.
    pub fn compile_v_set_with_work<E>(
        source: &JsString,
        ignore_case: bool,
        multiline: bool,
        charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Checked<Option<Self>, E> {
        Self::compile_v_set_units_with_work(source.code_units(), ignore_case, multiline, charge)
    }

    pub(crate) fn compile_v_set_units_with_work<E>(
        units: &[u16],
        ignore_case: bool,
        multiline: bool,
        charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Checked<Option<Self>, E> {
        let start_anchor = units.first() == Some(&0x5e);
        let end_anchor = units.last() == Some(&0x24);
        let Some(end) = units.len().checked_sub(usize::from(end_anchor)) else {
            return Ok(None);
        };
        let Some(body) = units.get(usize::from(start_anchor)..end) else {
            return Ok(None);
        };
        if body.first() != Some(&0x5b) || body.last() != Some(&0x5d) {
            return Ok(None);
        }
        let inverted = body.get(1) == Some(&0x5e);
        let mut builder = Builder {
            units: body,
            at: 1 + usize::from(inverted),
            ignore_case,
            charge,
        };
        // Token and escape recognition visit the retained source iteratively.
        builder.work(units.len().saturating_mul(4))?;
        let Some(mut ranges) = builder.operand()? else {
            return Ok(None);
        };
        let Some(operator) = builder.operator() else {
            // A standalone nested class is intentionally a separate proof.
            return Ok(None);
        };
        loop {
            if builder.operator() != Some(operator) {
                return Ok(None);
            }
            builder.at += 2;
            let Some(right) = builder.operand()? else {
                return Ok(None);
            };
            // Do not skip an operand when ranges is empty: unimplemented string
            // leaves remain unimplemented even if MayContainStrings is false.
            ranges = builder.combine(ranges, &right, operator)?;
            if builder.units.get(builder.at) == Some(&0x5d) {
                if builder.at + 1 != builder.units.len() {
                    return Ok(None);
                }
                break;
            }
        }
        builder.work(ranges.len())?;
        Ok(Some(Self {
            ranges: ranges.into(),
            inverted,
            start_anchor,
            end_anchor,
            multiline,
            ignore_case,
            start_word: None,
            end_word: None,
        }))
    }
}

impl<E, F: FnMut(usize) -> Result<(), E>> Builder<'_, F> {
    fn work(&mut self, amount: usize) -> Checked<(), E> {
        (self.charge)(amount).map_err(RegExpUnicodeSetCompileError::Work)
    }

    fn reserve(&mut self, ranges: &mut Intervals, additional: usize) -> Checked<(), E> {
        let needed = ranges
            .len()
            .checked_add(additional)
            .ok_or(RegExpUnicodeSetCompileError::Capacity)?;
        // A growing reserve can move every existing interval. Output writes are
        // also charged before the reserve, even when existing capacity suffices.
        let movement = if needed > ranges.capacity() {
            ranges.len()
        } else {
            0
        };
        self.work(movement.saturating_add(additional))?;
        ranges
            .try_reserve(additional)
            .map_err(|_| RegExpUnicodeSetCompileError::Capacity)
    }

    fn operator(&self) -> Option<Operator> {
        let tail = self.units.get(self.at..)?;
        if tail.starts_with(&[0x26, 0x26]) {
            Some(Operator::Intersection)
        } else if tail.starts_with(&[0x2d, 0x2d]) {
            Some(Operator::Subtraction)
        } else {
            None
        }
    }

    fn operand(&mut self) -> Checked<Option<Intervals>, E> {
        if self.units.get(self.at) == Some(&0x5b) {
            return self.flat_class();
        }
        let Some(atom) = class_atom(self.units, &mut self.at, true) else {
            return Ok(None);
        };
        let mut ranges = Vec::new();
        self.append_atom(&mut ranges, atom)?;
        self.close(ranges).map(Some)
    }

    fn flat_class(&mut self) -> Checked<Option<Intervals>, E> {
        self.at += 1;
        let inverted = self.units.get(self.at) == Some(&0x5e);
        self.at += usize::from(inverted);
        let mut ranges = Vec::new();
        let mut escapes = 0u8;
        let mut properties = [0u64; PROPERTY_WORDS];
        while self.units.get(self.at) != Some(&0x5d) {
            let Some(atom) = class_atom(self.units, &mut self.at, true) else {
                return Ok(None);
            };
            if self.units.get(self.at) == Some(&0x2d) {
                // Flat ranges use raw endpoints; '--' is never a range marker.
                if self.units.get(self.at + 1) == Some(&0x2d) {
                    return Ok(None);
                }
                self.at += 1;
                let Some(ClassAtom::Point(last)) = class_atom(self.units, &mut self.at, true)
                else {
                    return Ok(None);
                };
                let ClassAtom::Point(first) = atom else {
                    return Ok(None);
                };
                if first > last {
                    return Ok(None);
                }
                self.reserve(&mut ranges, 1)?;
                ranges.push((first, last));
            } else {
                match atom {
                    ClassAtom::Point(point) => {
                        self.reserve(&mut ranges, 1)?;
                        ranges.push((point, point));
                    }
                    ClassAtom::Escape(kind) => escapes |= 1 << kind,
                    ClassAtom::Property(kind) => {
                        let kind = usize::from(kind);
                        if kind >= PROPERTY_COUNT {
                            return Ok(None);
                        }
                        properties[kind / u64::BITS as usize] |=
                            1u64 << (kind % u64::BITS as usize);
                    }
                }
            }
        }
        self.at += 1;
        self.work(6 + PROPERTY_WORDS)?;
        for kind in 0..6 {
            if escapes & (1 << kind) != 0 {
                self.append_atom(&mut ranges, ClassAtom::Escape(kind))?;
            }
        }
        for (word, mut remaining) in properties.into_iter().enumerate() {
            while remaining != 0 {
                let bit = remaining.trailing_zeros() as usize;
                remaining &= remaining - 1;
                // Bits are set only after the kind bound above, including the
                // last partial word; no narrowing or padding tag can escape it.
                let kind = (word * u64::BITS as usize + bit) as u16;
                self.append_atom(&mut ranges, ClassAtom::Property(kind))?;
            }
        }
        let ranges = self.close(ranges)?;
        if inverted {
            self.complement(&ranges).map(Some)
        } else {
            Ok(Some(ranges))
        }
    }

    fn append_atom(&mut self, ranges: &mut Intervals, atom: ClassAtom) -> Checked<(), E> {
        match atom {
            ClassAtom::Point(point) => {
                self.reserve(ranges, 1)?;
                ranges.push((point, point));
            }
            ClassAtom::Escape(kind) => {
                // Ten source ranges, at most eleven complement ranges.
                self.work(11)?;
                self.reserve(ranges, 11)?;
                append_escape(ranges, kind, self.ignore_case);
            }
            ClassAtom::Property(kind) => {
                self.work(16)?;
                let intervals = property_ranges(kind, true, self.ignore_case);
                self.reserve(ranges, intervals.len())?;
                ranges.extend_from_slice(intervals);
            }
        }
        Ok(())
    }

    fn close(&mut self, mut ranges: Intervals) -> Checked<Intervals, E> {
        ranges = self.normalize(ranges)?;
        if !self.ignore_case {
            return Ok(ranges);
        }
        // F is idempotent. First retain R and add F(R), then add every source
        // whose fixed target is present. The result is F^-1(F(R)). Boolean set
        // operations and U-complements preserve this full preimage closure.
        // https://262.ecma-international.org/17.0/#sec-maybesimplecasefolding
        for inverse in [false, true] {
            let original = ranges.len();
            self.work(
                SIMPLE_CASE_FOLD
                    .len()
                    .saturating_mul(3 + lookup_levels(original)),
            )?;
            self.reserve(&mut ranges, SIMPLE_CASE_FOLD.len())?;
            for &(source, target) in SIMPLE_CASE_FOLD {
                let (probe, member) = if inverse {
                    (target, source)
                } else {
                    (source, target)
                };
                if interval_contains(&ranges[..original], probe) {
                    ranges.push((member, member));
                }
            }
            ranges = self.normalize(ranges)?;
        }
        Ok(ranges)
    }

    fn normalize(&mut self, mut ranges: Intervals) -> Checked<Intervals, E> {
        if ranges.len() > 1 {
            // Same bounded three-byte radix passes as the legacy normalizer,
            // with a checked scratch reservation instead of infallible vec!.
            let mut buffer = Vec::new();
            self.reserve(&mut buffer, ranges.len())?;
            buffer.resize(ranges.len(), (0, 0));
            for shift in [0, 8, 16] {
                self.work(256)?;
                let mut positions = [0usize; 256];
                self.work(ranges.len())?;
                for &(first, _) in &ranges {
                    positions[((first >> shift) & 255) as usize] += 1;
                }
                self.work(256)?;
                let mut offset = 0;
                for position in &mut positions {
                    let count = *position;
                    *position = offset;
                    offset += count;
                }
                self.work(ranges.len())?;
                for &range in &ranges {
                    let bucket = ((range.0 >> shift) & 255) as usize;
                    buffer[positions[bucket]] = range;
                    positions[bucket] += 1;
                }
                std::mem::swap(&mut ranges, &mut buffer);
            }
        }
        self.work(ranges.len())?;
        let mut write = 0;
        for read in 0..ranges.len() {
            let range = ranges[read];
            if write > 0 && range.0 <= ranges[write - 1].1 + 1 {
                ranges[write - 1].1 = ranges[write - 1].1.max(range.1);
            } else {
                ranges[write] = range;
                write += 1;
            }
        }
        ranges.truncate(write);
        Ok(ranges)
    }

    fn combine(
        &mut self,
        left: Intervals,
        right: &[(u32, u32)],
        operator: Operator,
    ) -> Checked<Intervals, E> {
        let bound = left
            .len()
            .checked_add(right.len())
            .ok_or(RegExpUnicodeSetCompileError::Capacity)?;
        self.work(bound.saturating_mul(3))?;
        let mut result = Vec::new();
        self.reserve(&mut result, bound)?;
        match operator {
            Operator::Intersection => {
                let (mut a, mut b) = (0, 0);
                while let (Some(&(first_a, last_a)), Some(&(first_b, last_b))) =
                    (left.get(a), right.get(b))
                {
                    let first = first_a.max(first_b);
                    let last = last_a.min(last_b);
                    if first <= last {
                        result.push((first, last));
                    }
                    if last_a <= last_b {
                        a += 1;
                    } else {
                        b += 1;
                    }
                }
            }
            Operator::Subtraction => {
                let mut b = 0;
                for (first, last) in left {
                    let mut cursor = first;
                    while right.get(b).is_some_and(|&(_, end)| end < cursor) {
                        b += 1;
                    }
                    while let Some(&(start, end)) = right.get(b) {
                        if start > last {
                            break;
                        }
                        if cursor < start {
                            result.push((cursor, start - 1));
                        }
                        if end >= last {
                            cursor = last + 1;
                            break;
                        }
                        cursor = end + 1;
                        b += 1;
                    }
                    if cursor <= last {
                        result.push((cursor, last));
                    }
                }
            }
        }
        Ok(result)
    }

    fn complement(&mut self, ranges: &[(u32, u32)]) -> Checked<Intervals, E> {
        let bound = ranges
            .len()
            .checked_add(1)
            .ok_or(RegExpUnicodeSetCompileError::Capacity)?;
        self.work(ranges.len())?;
        let mut result = Vec::new();
        self.reserve(&mut result, bound)?;
        let mut cursor = 0;
        for &(first, last) in ranges {
            if cursor < first {
                result.push((cursor, first - 1));
            }
            cursor = last + 1;
        }
        if cursor < LIMIT {
            result.push((cursor, LIMIT - 1));
        }
        Ok(result)
    }
}

#[cfg(test)]
#[path = "regexp_unicode_v_set_tests.rs"]
mod tests;
