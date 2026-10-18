//! Checked nested character-only UnicodeSets expressions (22.2.2.9).

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
/// and construction-frame buffers reserve fallibly; shared matcher storage
/// retains the existing infallible `Arc` allocation boundary.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RegExpUnicodeSetCompileError<E> {
    /// The embedding's opt-in construction work callback failed.
    Work(E),
    /// Interval or construction-frame storage exceeds platform capacity.
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

enum Term {
    Raw(ClassAtom),
    Range(u32, u32),
    Closed(Intervals),
}

struct Union {
    raw: Intervals,
    closed: Intervals,
    escapes: u8,
    properties: [u64; PROPERTY_WORDS],
}

enum Expression {
    Empty,
    Single(Term),
    Union(Union),
    Operation(Operator, Intervals),
}

struct Frame {
    expression: Expression,
    inverted: bool,
    need_operand: bool,
}

impl Frame {
    fn new(inverted: bool) -> Self {
        Self {
            expression: Expression::Empty,
            inverted,
            need_operand: true,
        }
    }
}

struct Builder<'a, F> {
    units: &'a [u16],
    at: usize,
    ignore_case: bool,
    charge: F,
}

impl RegExpUnicodeClassMatcher {
    /// Compiles a nested character-only v-mode set expression.
    ///
    /// The caller must validate the complete Pattern in v/vi mode. Operands are
    /// characters, supported class/property escapes, or nested unions,
    /// intersections, subtractions and complements. At least one nested class or
    /// set operator is required. Optional outside line assertions are admitted.
    /// String leaves and consuming concatenations remain unimplemented. This
    /// does not widen the legacy flat-class constructors.
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

    /// Compiles outside word assertions around one checked v-mode class.
    ///
    /// The complete Pattern must already be validated in v/vi. This admits an
    /// optional ^, optional leading \b/\B, one unquantified nested/operator
    /// class, optional trailing \b/\B, and optional $, in that order. At least
    /// one word assertion is required. Other assertion orders, strings, captures,
    /// quantifiers and consuming continuations remain separate caller proofs.
    ///
    /// The class units are borrowed without a source copy. Work errors and new
    /// construction capacity failures retain the checked compiler's host-error
    /// contract and final shared-storage allocation boundary. Legacy flat-class
    /// and repetition constructors retain their existing domains and callbacks.
    pub fn compile_v_set_with_word_boundaries_and_work<E>(
        source: &JsString,
        ignore_case: bool,
        multiline: bool,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Checked<Option<Self>, E> {
        let units = source.code_units();
        let start_anchor = units.first() == Some(&0x5e);
        let end_anchor = units.last() == Some(&0x24);
        let mut begin = usize::from(start_anchor);
        let Some(mut end) = units.len().checked_sub(usize::from(end_anchor)) else {
            return Ok(None);
        };
        let start_word = match units.get(begin..end).and_then(|body| body.get(..2)) {
            Some([0x5c, kind @ (0x62 | 0x42)]) => {
                begin += 2;
                Some(*kind == 0x62)
            }
            _ => None,
        };
        let end_word = if end >= begin + 2 {
            match &units[end - 2..end] {
                [0x5c, kind @ (0x62 | 0x42)] => {
                    end -= 2;
                    Some(*kind == 0x62)
                }
                _ => None,
            }
        } else {
            None
        };
        if start_word.is_none() && end_word.is_none() {
            return Ok(None);
        }
        let Some(body) = units.get(begin..end) else {
            return Ok(None);
        };
        // A bare complete class proves that neither edge token was escaped:
        // removing an escaped trailing $ or \b would leave a backslash here.
        // It also excludes leftover line gates and repeated word assertions.
        if body.first() != Some(&0x5b) || body.last() != Some(&0x5d) {
            return Ok(None);
        }
        charge(units.len()).map_err(RegExpUnicodeSetCompileError::Work)?;
        let Some(mut matcher) =
            Self::compile_v_set_units_with_work(body, ignore_case, multiline, &mut charge)?
        else {
            return Ok(None);
        };
        matcher.start_anchor = start_anchor;
        matcher.end_anchor = end_anchor;
        matcher.start_word = start_word;
        matcher.end_word = end_word;
        Ok(Some(matcher))
    }

    pub(crate) fn compile_v_set_units_with_work<E>(
        units: &[u16],
        ignore_case: bool,
        multiline: bool,
        charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Checked<Option<Self>, E> {
        Self::compile_v_set_units_with_string_operands_and_work(
            units,
            ignore_case,
            multiline,
            false,
            charge,
        )
    }

    pub(crate) fn compile_v_set_units_with_single_character_strings_and_work<E>(
        units: &[u16],
        ignore_case: bool,
        multiline: bool,
        charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Checked<Option<Self>, E> {
        Self::compile_v_set_units_with_string_operands_and_work(
            units,
            ignore_case,
            multiline,
            true,
            charge,
        )
    }

    fn compile_v_set_units_with_string_operands_and_work<E>(
        units: &[u16],
        ignore_case: bool,
        multiline: bool,
        allow_string_operands: bool,
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
        // Token recognition and constant-size frame transitions visit the
        // retained source iteratively. Growing stack movement is charged below.
        builder.work(units.len().saturating_mul(4))?;
        builder.work(1)?;
        let mut current = Frame::new(inverted);
        let mut parents = Vec::new();
        let mut eligible = false;
        let mut has_string_operand = false;
        let ranges;
        loop {
            if builder.units.get(builder.at) == Some(&0x5d) {
                if current.need_operand && !matches!(current.expression, Expression::Empty) {
                    return Ok(None);
                }
                builder.at += 1;
                let mut completed = builder.finish(current.expression)?;
                let Some(mut parent) = parents.pop() else {
                    let eligible = if allow_string_operands {
                        has_string_operand
                    } else {
                        eligible
                    };
                    if builder.at != builder.units.len() || !eligible {
                        return Ok(None);
                    }
                    // The root's inversion remains the matcher's flag. A child
                    // inversion instead becomes its parent's closed operand.
                    ranges = completed;
                    break;
                };
                if current.inverted {
                    completed = builder.complement(&completed)?;
                }
                builder.accept(&mut parent, Term::Closed(completed))?;
                current = parent;
                continue;
            }
            if current.need_operand {
                if builder.units.get(builder.at) == Some(&0x5b) {
                    eligible = true;
                    builder.reserve_frames(&mut parents, 1)?;
                    parents.push(current);
                    builder.at += 1;
                    let inverted = builder.units.get(builder.at) == Some(&0x5e);
                    builder.at += usize::from(inverted);
                    current = Frame::new(inverted);
                } else {
                    let term = if allow_string_operands
                        && builder
                            .units
                            .get(builder.at..)
                            .is_some_and(|tail| tail.starts_with(&[0x5c, 0x71, 0x7b]))
                    {
                        builder.at += 3;
                        let Some(ranges) = builder.single_character_string_ranges(true)? else {
                            return Ok(None);
                        };
                        has_string_operand = true;
                        Term::Closed(ranges)
                    } else {
                        let Some(term) = builder.term() else {
                            return Ok(None);
                        };
                        term
                    };
                    if matches!(term, Term::Range(..))
                        && matches!(current.expression, Expression::Operation(..))
                    {
                        return Ok(None);
                    }
                    builder.accept(&mut current, term)?;
                }
                continue;
            }
            if let Some(operator) = builder.operator() {
                if builder.units.get(builder.at + 2) == Some(&0x26)
                    && operator == Operator::Intersection
                {
                    return Ok(None);
                }
                let expression = std::mem::replace(&mut current.expression, Expression::Empty);
                current.expression = match expression {
                    Expression::Single(Term::Range(..)) => return Ok(None),
                    Expression::Single(term) => {
                        Expression::Operation(operator, builder.close_term(term)?)
                    }
                    Expression::Operation(previous, left) if previous == operator => {
                        Expression::Operation(operator, left)
                    }
                    _ => return Ok(None),
                };
                eligible = true;
                builder.at += 2;
                current.need_operand = true;
            } else if matches!(
                current.expression,
                Expression::Single(_) | Expression::Union(_)
            ) {
                current.need_operand = true;
            } else {
                return Ok(None);
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

impl RegExpUnicodeClassMatcher {
    /// Compiles a standalone v-mode ClassStringDisjunction whose alternatives
    /// each contain exactly one decoded character, with optional canonical
    /// ^, leading word, trailing word and $ assertions. Complete captures remain
    /// a caller proof. Other class members and all string-valued branches decline.
    ///
    /// This entry proves ClassString syntax itself, including its backspace and
    /// reserved-punctuation escapes. Folding precedes outer class inversion.
    /// https://262.ecma-international.org/17.0/#prod-ClassStringDisjunction
    pub fn compile_single_character_strings_v_set_with_work<E>(
        source: &JsString,
        ignore_case: bool,
        multiline: bool,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, RegExpUnicodeSetCompileError<E>> {
        let units = source.code_units();
        // Full borrowed framing and character decoding precede normalization.
        charge(units.len().saturating_mul(3)).map_err(RegExpUnicodeSetCompileError::Work)?;
        let start_anchor = units.first() == Some(&0x5e);
        let end_anchor = units.last() == Some(&0x24);
        let mut begin = usize::from(start_anchor);
        let mut end = units.len() - usize::from(end_anchor);
        let start_word = match units.get(begin..end).and_then(|body| body.get(..2)) {
            Some([0x5c, kind @ (0x62 | 0x42)]) => {
                begin += 2;
                Some(*kind == 0x62)
            }
            _ => None,
        };
        let end_word = if end >= begin + 2 {
            match &units[end - 2..end] {
                [0x5c, kind @ (0x62 | 0x42)] => {
                    end -= 2;
                    Some(*kind == 0x62)
                }
                _ => None,
            }
        } else {
            None
        };
        let Some(body) = units.get(begin..end) else {
            return Ok(None);
        };
        if body.first() != Some(&0x5b) || body.last() != Some(&0x5d) {
            return Ok(None);
        }
        let inverted = body.get(1) == Some(&0x5e);
        let begin = 1 + usize::from(inverted);
        let Some(strings) = body
            .get(begin..body.len() - 1)
            .and_then(|class| class.strip_prefix(&[0x5c, 0x71, 0x7b]))
            .and_then(|strings| strings.strip_suffix(&[0x7d]))
        else {
            return Ok(None);
        };
        let mut builder = Builder {
            units: strings,
            at: 0,
            ignore_case,
            charge: &mut charge,
        };
        let Some(ranges) = builder.single_character_string_ranges(false)? else {
            return Ok(None);
        };
        builder.work(ranges.len())?;
        Ok(Some(Self {
            ranges: ranges.into(),
            inverted,
            start_anchor,
            end_anchor,
            multiline,
            ignore_case,
            start_word,
            end_word,
        }))
    }
}

impl RegExpUnicodeClassMatcher {
    /// Compiles one checked v-mode class containing character-only string leaves.
    ///
    /// The complete Pattern must already be validated in v/vi mode. At least one
    /// ClassStringDisjunction is required, and every alternative of every such
    /// operand must decode to exactly one Unicode character. Those finite leaves
    /// compose with supported character operands, unions, nested complements,
    /// intersections and subtractions. Folding precedes every set operation.
    ///
    /// Optional outside ^, leading \b/\B, trailing \b/\B and $ assertions are
    /// admitted in that order. Complete captures remain a caller proof. Empty or
    /// longer string alternatives, quantifiers and consuming continuations decline,
    /// even when an earlier operand makes the resulting character set empty.
    /// Legacy constructors retain their domains and construction work callbacks.
    /// https://262.ecma-international.org/17.0/#sec-compiletocharset
    pub fn compile_v_set_with_single_character_strings_and_work<E>(
        source: &JsString,
        ignore_case: bool,
        multiline: bool,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, RegExpUnicodeSetCompileError<E>> {
        let units = source.code_units();
        charge(units.len()).map_err(RegExpUnicodeSetCompileError::Work)?;
        let start_anchor = units.first() == Some(&0x5e);
        let end_anchor = units.last() == Some(&0x24);
        let mut begin = usize::from(start_anchor);
        let mut end = units.len() - usize::from(end_anchor);
        let start_word = match units.get(begin..end).and_then(|body| body.get(..2)) {
            Some([0x5c, kind @ (0x62 | 0x42)]) => {
                begin += 2;
                Some(*kind == 0x62)
            }
            _ => None,
        };
        let end_word = if end >= begin + 2 {
            match &units[end - 2..end] {
                [0x5c, kind @ (0x62 | 0x42)] => {
                    end -= 2;
                    Some(*kind == 0x62)
                }
                _ => None,
            }
        } else {
            None
        };
        let Some(body) = units.get(begin..end) else {
            return Ok(None);
        };
        if body.first() != Some(&0x5b) || body.last() != Some(&0x5d) {
            return Ok(None);
        }
        let Some(mut matcher) = Self::compile_v_set_units_with_single_character_strings_and_work(
            body,
            ignore_case,
            multiline,
            &mut charge,
        )?
        else {
            return Ok(None);
        };
        matcher.start_anchor = start_anchor;
        matcher.end_anchor = end_anchor;
        matcher.start_word = start_word;
        matcher.end_word = end_word;
        Ok(Some(matcher))
    }
}

// ClassSetCharacter in ClassString context. Unicode identity escapes are narrower
// than the legacy ordinary-atom helper; \b is backspace and \d/\p are not chars.
// Adjacent fixed Unicode lead/trail escapes combine; braced escapes do not.
fn single_string_character(units: &[u16], at: &mut usize) -> Option<u32> {
    let unit = *units.get(*at)?;
    *at += 1;
    if unit == 0x5c {
        let escaped = *units.get(*at)?;
        if escaped == 0x62 {
            *at += 1;
            return Some(8);
        }
        if b"&-!#%,:;<=>@`~^$\\.*+?()[]{}|/".contains(&(u8::try_from(escaped).ok()?)) {
            *at += 1;
            return Some(u32::from(escaped));
        }
        if !matches!(
            escaped,
            0x30 | 0x63 | 0x66 | 0x6e | 0x72 | 0x74 | 0x76 | 0x78 | 0x75
        ) {
            return None;
        }
        return super::unicode_literal_atom(units, at, unit);
    }
    if b"()[]{}/-|".contains(&(u8::try_from(unit).unwrap_or(0))) {
        return None;
    }
    if (0xd800..=0xdbff).contains(&unit) {
        if let Some(&trail @ 0xdc00..=0xdfff) = units.get(*at) {
            *at += 1;
            return Some(0x10000 + (u32::from(unit) - 0xd800) * 0x400 + u32::from(trail) - 0xdc00);
        }
    }
    Some(u32::from(unit))
}

impl<E, F: FnMut(usize) -> Result<(), E>> Builder<'_, F> {
    // Both callers prove the same one-character alternatives. The standalone
    // entry has already stripped its final brace; algebra operands consume it.
    fn single_character_string_ranges(&mut self, braced: bool) -> Checked<Option<Intervals>, E> {
        let mut ranges = Vec::new();
        loop {
            let Some(point) = single_string_character(self.units, &mut self.at) else {
                return Ok(None);
            };
            let end = if braced {
                self.units.get(self.at) == Some(&0x7d)
            } else {
                self.at == self.units.len()
            };
            // Do not deduplicate or fold before proving every complete branch.
            if !end && self.units.get(self.at) != Some(&0x7c) {
                return Ok(None);
            }
            self.reserve(&mut ranges, 1)?;
            ranges.push((point, point));
            if end {
                self.at += usize::from(braced);
                break;
            }
            self.at += 1;
        }
        self.close(ranges).map(Some)
    }

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

    fn reserve_frames(&mut self, frames: &mut Vec<Frame>, additional: usize) -> Checked<(), E> {
        let needed = frames
            .len()
            .checked_add(additional)
            .ok_or(RegExpUnicodeSetCompileError::Capacity)?;
        let movement = if needed > frames.capacity() {
            frames.len()
        } else {
            0
        };
        self.work(movement.saturating_add(additional))?;
        frames
            .try_reserve(additional)
            .map_err(|_| RegExpUnicodeSetCompileError::Capacity)
    }

    fn term(&mut self) -> Option<Term> {
        // A raw '-' is a range separator, never an operand. A decoded escaped
        // '-' remains a Character and can be an endpoint.
        if self.units.get(self.at) == Some(&0x2d) {
            return None;
        }
        let atom = class_atom(self.units, &mut self.at, true)?;
        if matches!(&atom, ClassAtom::Property(kind) if usize::from(*kind) >= PROPERTY_COUNT) {
            return None;
        }
        if self.units.get(self.at) == Some(&0x2d) && self.units.get(self.at + 1) != Some(&0x2d) {
            self.at += 1;
            let ClassAtom::Point(first) = atom else {
                return None;
            };
            let ClassAtom::Point(last) = class_atom(self.units, &mut self.at, true)? else {
                return None;
            };
            (first <= last).then_some(Term::Range(first, last))
        } else {
            Some(Term::Raw(atom))
        }
    }

    fn accept(&mut self, frame: &mut Frame, term: Term) -> Checked<(), E> {
        let expression = std::mem::replace(&mut frame.expression, Expression::Empty);
        frame.expression = match expression {
            Expression::Empty => Expression::Single(term),
            Expression::Single(first) => {
                self.work(2 + PROPERTY_WORDS)?;
                let mut union = Union {
                    raw: Vec::new(),
                    closed: Vec::new(),
                    escapes: 0,
                    properties: [0; PROPERTY_WORDS],
                };
                self.union_term(&mut union, first)?;
                self.union_term(&mut union, term)?;
                Expression::Union(union)
            }
            Expression::Union(mut union) => {
                self.union_term(&mut union, term)?;
                Expression::Union(union)
            }
            Expression::Operation(operator, left) => {
                let right = self.close_term(term)?;
                Expression::Operation(operator, self.combine(left, &right, operator)?)
            }
        };
        frame.need_operand = false;
        Ok(())
    }

    fn union_term(&mut self, union: &mut Union, term: Term) -> Checked<(), E> {
        match term {
            Term::Raw(ClassAtom::Point(point)) => {
                self.reserve(&mut union.raw, 1)?;
                union.raw.push((point, point));
            }
            Term::Range(first, last) => {
                self.reserve(&mut union.raw, 1)?;
                union.raw.push((first, last));
            }
            Term::Raw(ClassAtom::Escape(kind)) => union.escapes |= 1 << kind,
            Term::Raw(ClassAtom::Property(kind)) => {
                let kind = usize::from(kind);
                // class_atom only returns kinds from the fixed registry.
                debug_assert!(kind < PROPERTY_COUNT);
                union.properties[kind / u64::BITS as usize] |= 1u64 << (kind % u64::BITS as usize);
            }
            Term::Closed(ranges) => {
                if union.closed.is_empty() {
                    union.closed = ranges;
                } else {
                    self.reserve(&mut union.closed, ranges.len())?;
                    union.closed.extend_from_slice(&ranges);
                }
            }
        }
        Ok(())
    }

    fn close_term(&mut self, term: Term) -> Checked<Intervals, E> {
        let mut ranges = Vec::new();
        match term {
            Term::Closed(closed) => return Ok(closed),
            Term::Range(first, last) => {
                self.reserve(&mut ranges, 1)?;
                ranges.push((first, last));
            }
            Term::Raw(atom) => self.append_atom(&mut ranges, atom)?,
        }
        if ranges.is_empty() {
            Ok(ranges)
        } else {
            self.close(ranges)
        }
    }

    fn finish(&mut self, expression: Expression) -> Checked<Intervals, E> {
        let mut union = match expression {
            Expression::Empty => return Ok(Vec::new()),
            Expression::Single(term) => return self.close_term(term),
            Expression::Operation(_, ranges) => return Ok(ranges),
            Expression::Union(union) => union,
        };
        self.work(6 + PROPERTY_WORDS)?;
        for kind in 0..6 {
            if union.escapes & (1 << kind) != 0 {
                self.append_atom(&mut union.raw, ClassAtom::Escape(kind))?;
            }
        }
        for (word, mut remaining) in union.properties.into_iter().enumerate() {
            while remaining != 0 {
                let bit = remaining.trailing_zeros() as usize;
                remaining &= remaining - 1;
                let kind = (word * u64::BITS as usize + bit) as u16;
                self.append_atom(&mut union.raw, ClassAtom::Property(kind))?;
            }
        }
        if !union.raw.is_empty() {
            let raw = self.close(union.raw)?;
            if union.closed.is_empty() {
                return Ok(raw);
            }
            self.reserve(&mut union.closed, raw.len())?;
            union.closed.extend_from_slice(&raw);
        }
        // All child chunks are already closed. Normalize once after batching;
        // never refold children or merge a growing accumulator per sibling.
        self.normalize(union.closed)
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

#[cfg(test)]
#[path = "regexp_unicode_single_character_strings_tests.rs"]
mod single_character_strings_tests;

#[cfg(test)]
#[path = "regexp_unicode_single_character_string_operands_tests.rs"]
mod single_character_string_operands_tests;
