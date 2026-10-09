//! Literal Pattern compilation and UTF-16 matching (22.2.2).

use crate::{JsString, is_identifier_part, regexp_canonicalize_character};
use std::{ops::Range, sync::Arc};

/// An immutable literal matcher for an ordinary Pattern or a proved Unicode subset.
///
/// Compilation accepts literal characters and their escapes, with ordinary
/// capturing and noncapturing groups containing the same subset. Other syntax
/// returns `None`, distinct from a failed match. `compile` requires ordinary
/// Pattern validation; Unicode compilation requires Unicode validation.
/// Match ranges use UTF-16 code-unit offsets, not byte spans.
/// Compilation and search are linear in Pattern and input length respectively.
#[derive(Clone, Debug)]
pub struct RegExpLiteralMatcher(Arc<Program>);

#[derive(Debug)]
struct Program {
    units: Vec<u16>,
    failure: Vec<usize>,
    ignore_case: bool,
    captures: Vec<Range<usize>>,
    mode: LiteralMode,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum LiteralMode {
    Ordinary,
    BmpUnicode,
    ScalarUnicode,
    CodePointUnicode,
}

impl RegExpLiteralMatcher {
    pub(crate) fn matched_len(&self) -> usize {
        self.0.units.len()
    }

    pub(crate) fn matched_units(&self) -> &[u16] {
        &self.0.units
    }

    /// Compiles a validated `u` or `v` Pattern with a nonempty BMP literal body.
    ///
    /// This case-sensitive subset excludes surrogate code units and may contain
    /// capturing and noncapturing groups, including braced escapes for BMP units.
    /// Every consumed character has one code unit and every successful endpoint
    /// is a code-point boundary. A scan through a surrogate pair cannot create
    /// a BMP occurrence; an initial offset inside a pair cannot match when
    /// sticky. Other Unicode bodies, empty bodies and ignore-case matching need
    /// separate proofs (22.2.2.2, 22.2.7.2). The caller must validate the Pattern
    /// in Unicode mode first. Storage and matching retain the ordinary flat plan.
    pub fn compile_bmp_unicode(source: &JsString) -> Option<Self> {
        let matcher = Self::compile_with_unicode_escapes(source, false, LiteralMode::BmpUnicode)?;
        (!matcher.0.units.is_empty()
            && matcher
                .0
                .units
                .iter()
                .all(|&unit| !(0xd800..=0xdfff).contains(&unit)))
        .then_some(matcher)
    }

    /// Compiles a case-sensitive Unicode concatenation of scalar literal atoms.
    ///
    /// Raw supplementary characters, braced scalar escapes, and directly adjacent
    /// fixed lead/trail Unicode escapes each consume one code point (22.2.1,
    /// 22.2.2.7). Groups may surround atoms but cannot split one. Lone surrogate
    /// atoms, empty bodies, other syntax and ignore-case matching are excluded.
    /// The caller must first validate the Pattern in u/v mode. Flattening retains
    /// relative UTF-16 capture endpoints and the linear ordinary search plan.
    pub fn compile_unicode_scalars(source: &JsString) -> Option<Self> {
        let matcher =
            Self::compile_with_unicode_escapes(source, false, LiteralMode::ScalarUnicode)?;
        (!matcher.0.units.is_empty()).then_some(matcher)
    }

    /// Finds a scalar Unicode literal at or after the initial code-point boundary.
    ///
    /// Each atom is a nonsurrogate BMP unit or a complete surrogate pair. Thus
    /// every literal occurrence and capture endpoint is a code-point boundary;
    /// scanning UTF-16 cannot create a partial supplementary occurrence. Node/V8's
    /// approved leading boundary is used when start is inside an input pair
    /// (the documented edition 17 RegExpBuiltinExec discrepancy, 22.2.7.2).
    /// Only plans produced by compile_unicode_scalars are admitted here.
    pub fn find_unicode_scalars(
        &self,
        input: &JsString,
        start: usize,
        sticky: bool,
    ) -> Option<Range<usize>> {
        if self.0.mode != LiteralMode::ScalarUnicode {
            return None;
        }
        self.find(input, unicode_start(input, start)?, sticky)
    }

    /// Compiles a case-sensitive Unicode literal concatenation, including lone
    /// surrogate atoms (22.2.1, 22.2.2.7). The caller must validate u/v grammar.
    ///
    /// Two distinct surrogate atoms which would flatten into one pair remain
    /// excluded: no Unicode input can match that concatenation. All admitted
    /// internal atom and capture boundaries agree with UTF-16 decoding. Empty
    /// bodies and other syntax are excluded; use find_unicode_code_points to
    /// enforce input boundaries for successful occurrences.
    pub fn compile_unicode_code_points(source: &JsString) -> Option<Self> {
        let matcher =
            Self::compile_with_unicode_escapes(source, false, LiteralMode::CodePointUnicode)?;
        (!matcher.0.units.is_empty()).then_some(matcher)
    }

    /// Searches Unicode literal occurrences with complete input characters.
    ///
    /// An occurrence cannot consume half of an input pair. A rejected boundary
    /// resumes the same KMP scan, retaining linear search time. Internal boundaries
    /// follow the compiled atom proof; only the two external bounds need checking.
    /// Initial pair offsets use the documented Node/V8 leading boundary rule.
    pub fn find_unicode_code_points(
        &self,
        input: &JsString,
        start: usize,
        sticky: bool,
    ) -> Option<Range<usize>> {
        if self.0.mode != LiteralMode::CodePointUnicode {
            return None;
        }
        let units = input.code_units();
        self.find_if(input, unicode_start(input, start)?, sticky, |range| {
            unicode_boundary(units, range.start) && unicode_boundary(units, range.end)
        })
    }

    /// Compiles a validated Unicode Pattern with only mandatory empty groups.
    ///
    /// Unicode and UnicodeSets validation are the caller's responsibility.
    /// Ignore-case does not affect an empty body (22.2.2.2).
    pub fn compile_unicode_empty(source: &JsString) -> Option<Self> {
        let matcher = Self::compile(source, false)?;
        matcher.0.units.is_empty().then_some(matcher)
    }

    /// Matches an empty Unicode body at the initial code-point boundary.
    ///
    /// An initial UTF-16 offset inside a surrogate pair identifies its complete
    /// Unicode character. Following Node/V8 for the documented edition 17
    /// RegExpBuiltinExec result-start inconsistency (22.2.7.2), an empty match
    /// uses that character's leading boundary. Offsets past the input fail,
    /// and lone surrogates retain their boundaries. Nonempty programs are rejected.
    pub fn find_unicode_empty(&self, input: &JsString, start: usize) -> Option<Range<usize>> {
        if !self.0.units.is_empty() {
            return None;
        }
        self.find(input, unicode_start(input, start)?, true)
    }

    /// Compiles the literal-only subset, returning `None` for other syntax.
    pub fn compile(source: &JsString, ignore_case: bool) -> Option<Self> {
        Self::compile_with_unicode_escapes(source, ignore_case, LiteralMode::Ordinary)
    }

    fn compile_with_unicode_escapes(
        source: &JsString,
        ignore_case: bool,
        mode: LiteralMode,
    ) -> Option<Self> {
        let source = source.code_units();
        let mut index = 0;
        let mut groups = Vec::new();
        let mut captures = Vec::new();
        let mut units = Vec::new();
        while let Some(&unit) = source.get(index) {
            index += 1;
            // CompileAtom group productions (22.2.2.7). An unquantified group whose
            // body is a literal concatenation has exactly that body's matcher,
            // including the empty body. Captures retain their relative UTF-16
            // endpoints in opening-parenthesis order. Flatten
            // nested groups iteratively so compilation and storage never add
            // native recursion. Named/assertion/modifier groups remain unsupported.
            if unit == u16::from(b'(') {
                let capture = if source.get(index) == Some(&u16::from(b'?')) {
                    if source.get(index..index + 2)? != [u16::from(b'?'), u16::from(b':')] {
                        return None;
                    }
                    index += 2;
                    None
                } else {
                    let capture = captures.len();
                    captures.push(units.len()..units.len());
                    Some(capture)
                };
                groups.push(capture);
                continue;
            }
            if unit == u16::from(b')') {
                if let Some(capture) = groups.pop()? {
                    captures[capture].end = units.len();
                }
                continue;
            }
            if matches!(
                mode,
                LiteralMode::ScalarUnicode | LiteralMode::CodePointUnicode
            ) {
                let value = unicode_literal_atom(source, &mut index, unit)?;
                if mode == LiteralMode::ScalarUnicode {
                    let scalar = char::from_u32(value)?;
                    units.extend_from_slice(scalar.encode_utf16(&mut [0; 2]));
                } else if value <= 0xffff {
                    let unit = value as u16;
                    // Distinct lone atoms cannot denote a well-formed input pair,
                    // even when empty or noncapturing groups separate them.
                    if (0xdc00..=0xdfff).contains(&unit)
                        && units
                            .last()
                            .is_some_and(|unit| (0xd800..=0xdbff).contains(unit))
                    {
                        return None;
                    }
                    units.push(unit);
                } else {
                    units.extend_from_slice(char::from_u32(value)?.encode_utf16(&mut [0; 2]));
                }
                continue;
            }
            let unit = if unit == u16::from(b'\\') {
                if mode == LiteralMode::BmpUnicode
                    && source.get(index..index + 2) == Some(&[0x75, 0x7b])
                {
                    braced_bmp_escape(source, &mut index)?
                } else {
                    character_escape(source, &mut index)?
                }
            } else if is_syntax(unit) {
                return None;
            } else {
                unit
            };
            units.push(canonicalize(unit, ignore_case));
        }
        if !groups.is_empty() {
            return None;
        }
        let mut failure = vec![0; units.len()];
        let mut matched = 0;
        for index in 1..units.len() {
            while matched > 0 && units[index] != units[matched] {
                matched = failure[matched - 1];
            }
            if units[index] == units[matched] {
                matched += 1;
            }
            failure[index] = matched;
        }
        Some(Self(Arc::new(Program {
            units,
            failure,
            ignore_case,
            captures,
            mode,
        })))
    }

    /// Capture ranges relative to a successful match's UTF-16 start offset.
    ///
    /// The whole match is excluded. Opening-parenthesis order determines the
    /// capture order; all groups participate because this subset has neither
    /// alternatives nor quantifiers. Empty groups retain empty ranges. Every
    /// endpoint is within the matched literal sequence.
    pub fn capture_ranges(&self) -> &[Range<usize>] {
        &self.0.captures
    }

    /// Finds the first match at or after `start`, or exactly there when sticky.
    ///
    /// Empty Patterns match at `start` through the input's end, inclusively.
    /// An offset past the end produces no match. Search does not allocate.
    pub fn find(&self, input: &JsString, start: usize, sticky: bool) -> Option<Range<usize>> {
        self.find_if(input, start, sticky, |_| true)
    }

    /// Streams nonempty literal occurrences, retaining the failure state between
    /// overlapping matches. Each input unit is visited only once.
    pub(crate) fn matches_from<'a>(
        &'a self,
        input: &'a [u16],
        start: usize,
    ) -> Option<LiteralMatches<'a>> {
        if self.0.units.is_empty() {
            return None;
        }
        input.get(start..)?;
        Some(LiteralMatches {
            program: &self.0,
            input,
            cursor: start,
            matched: 0,
        })
    }

    /// Continues the same linear scan after a boundary assertion rejects a match.
    pub(crate) fn find_if(
        &self,
        input: &JsString,
        start: usize,
        sticky: bool,
        mut accept: impl FnMut(&Range<usize>) -> bool,
    ) -> Option<Range<usize>> {
        let input = input.code_units();
        let suffix = input.get(start..)?;
        let program = &self.0;
        if program.units.is_empty() {
            if sticky {
                let range = start..start;
                return accept(&range).then_some(range);
            }
            return (start..=input.len()).find_map(|offset| {
                let range = offset..offset;
                accept(&range).then_some(range)
            });
        }
        if sticky {
            let candidate = suffix.get(..program.units.len())?;
            return candidate
                .iter()
                .zip(&program.units)
                .all(|(&unit, &expected)| canonicalize(unit, program.ignore_case) == expected)
                .then_some(start..start + program.units.len())
                .filter(&mut accept);
        }
        self.matches_from(input, start)?.find(&mut accept)
    }
}

// GetStringIndex maps an initial low-surrogate offset to its character's
// leading boundary. Use that boundary for the whole match and captures too.
pub(crate) fn unicode_start(input: &JsString, start: usize) -> Option<usize> {
    let units = input.code_units();
    units.get(start..)?;
    Some(
        if start > 0
            && units
                .get(start)
                .is_some_and(|unit| (0xdc00..=0xdfff).contains(unit))
            && (0xd800..=0xdbff).contains(&units[start - 1])
        {
            start - 1
        } else {
            start
        },
    )
}

// Decode one Pattern atom before flattening groups. RegExpUnicodeEscapeSequence
// joins only directly adjacent fixed lead/trail escapes, not braced surrogates
// or escapes separated by a group. Raw source pairs form one SourceCharacter.
fn unicode_literal_atom(source: &[u16], index: &mut usize, unit: u16) -> Option<u32> {
    let value = if unit == 0x5c {
        if source.get(*index..*index + 2) == Some(&[0x75, 0x7b]) {
            *index += 2;
            let mut value = 0u32;
            let mut has_digit = false;
            loop {
                if source.get(*index) == Some(&0x7d) {
                    *index += 1;
                    break has_digit.then_some(value)?;
                }
                let digit = u32::from(hex_escape(source, index, 1)?);
                value = value.checked_mul(16)?.checked_add(digit)?;
                if value > 0x10ffff {
                    return None;
                }
                has_digit = true;
            }
        } else {
            let fixed_unicode = source.get(*index) == Some(&0x75);
            let first = character_escape(source, index)?;
            if fixed_unicode
                && (0xd800..=0xdbff).contains(&first)
                && source.get(*index..*index + 2) == Some(&[0x5c, 0x75])
            {
                let mut next = *index + 2;
                if let Some(value) = hex_escape(source, &mut next, 4)
                    .and_then(|second| surrogate_scalar(first, second))
                {
                    *index = next;
                    value
                } else {
                    u32::from(first)
                }
            } else {
                u32::from(first)
            }
        }
    } else if is_syntax(unit) {
        return None;
    } else if (0xd800..=0xdbff).contains(&unit) {
        if let Some(value) = source
            .get(*index)
            .and_then(|&second| surrogate_scalar(unit, second))
        {
            *index += 1;
            value
        } else {
            u32::from(unit)
        }
    } else {
        u32::from(unit)
    };
    Some(value)
}

fn unicode_boundary(input: &[u16], offset: usize) -> bool {
    offset == 0
        || offset == input.len()
        || !(0xdc00..=0xdfff).contains(&input[offset])
        || !(0xd800..=0xdbff).contains(&input[offset - 1])
}

fn surrogate_scalar(first: u16, second: u16) -> Option<u32> {
    (0xdc00..=0xdfff)
        .contains(&second)
        .then(|| 0x10000 + ((u32::from(first) - 0xd800) << 10) + (u32::from(second) - 0xdc00))
}

pub(crate) struct LiteralMatches<'a> {
    program: &'a Program,
    input: &'a [u16],
    cursor: usize,
    matched: usize,
}

impl Iterator for LiteralMatches<'_> {
    type Item = Range<usize>;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(&unit) = self.input.get(self.cursor) {
            self.cursor += 1;
            let unit = canonicalize(unit, self.program.ignore_case);
            while self.matched > 0 && unit != self.program.units[self.matched] {
                self.matched = self.program.failure[self.matched - 1];
            }
            if unit == self.program.units[self.matched] {
                self.matched += 1;
            }
            if self.matched == self.program.units.len() {
                let range = self.cursor - self.matched..self.cursor;
                self.matched = self.program.failure[self.matched - 1];
                return Some(range);
            }
        }
        None
    }
}

/// CharacterEscape decoding shared by ordinary atoms and class characters.
pub(crate) fn character_escape(source: &[u16], index: &mut usize) -> Option<u16> {
    let escaped = *source.get(*index)?;
    *index += 1;
    let unit = match escaped {
        0x66 => 0x0c,
        0x6e => 0x0a,
        0x72 => 0x0d,
        0x74 => 0x09,
        0x76 => 0x0b,
        0x30 if !source
            .get(*index)
            .is_some_and(|u| (0x30..=0x39).contains(u)) =>
        {
            0
        }
        0x63 => {
            let letter = *source.get(*index)?;
            if !(0x41..=0x5a).contains(&letter) && !(0x61..=0x7a).contains(&letter) {
                return None;
            }
            *index += 1;
            letter % 32
        }
        0x78 => hex_escape(source, index, 2)?,
        0x75 => hex_escape(source, index, 4)?,
        // Ordinary IdentityEscape excludes Unicode ID_Continue.
        // IdentifierPartChar adds '$' to that pinned property, so
        // allow it explicitly; lone surrogate units also remain
        // characters rather than being replaced or rejected.
        _ if escaped == 0x24
            || !char::from_u32(u32::from(escaped)).is_some_and(is_identifier_part) =>
        {
            escaped
        }
        _ => return None,
    };
    Some(unit)
}

fn canonicalize(unit: u16, ignore_case: bool) -> u16 {
    regexp_canonicalize_character(u32::from(unit), ignore_case, false) as u16
}

pub(crate) fn is_syntax(unit: u16) -> bool {
    matches!(unit, 0x24 | 0x28..=0x2b | 0x2e | 0x3f | 0x5b | 0x5d | 0x5e | 0x7b..=0x7d)
}

// Validated Unicode RegExpUnicodeEscapeSequence :: u{ CodePoint } (22.2.1).
// Checked u16 accumulation admits only BMP values; the containing literal or
// character-set proof excludes surrogates. Leading zeros need no extra storage.
pub(crate) fn braced_bmp_escape(source: &[u16], index: &mut usize) -> Option<u16> {
    *index += 2;
    let mut value = 0u16;
    let mut has_digit = false;
    loop {
        if source.get(*index) == Some(&0x7d) {
            *index += 1;
            return has_digit.then_some(value);
        }
        let digit = hex_escape(source, index, 1)?;
        value = value.checked_mul(16)?.checked_add(digit)?;
        has_digit = true;
    }
}

fn hex_escape(source: &[u16], index: &mut usize, count: usize) -> Option<u16> {
    let mut value = 0;
    for _ in 0..count {
        let digit = match *source.get(*index)? {
            unit @ 0x30..=0x39 => unit - 0x30,
            unit @ 0x41..=0x46 => unit - 0x41 + 10,
            unit @ 0x61..=0x66 => unit - 0x61 + 10,
            _ => return None,
        };
        *index += 1;
        value = value * 16 + digit;
    }
    Some(value)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fmt::Write;

    #[test]
    fn unicode_surrogate_literal_ranges_snapshot() {
        let mut rows = String::new();
        for pattern in [
            r"\uD83D",
            r"\uDE00",
            r"\u{D800}",
            r"\u{DFFF}",
            r"(\uD83D)",
            r"()(\uDE00())",
            r"a\uD83D",
            r"\uDE00a",
            r"\uD83D\uD83D",
            r"\uDE00\uDE00",
            r"(\uD83D)\uD83D\uDE00",
            r"\uD83D\uDE00(\uDE00)",
            r"(\uD83D)a(\uDE00)",
            r"\u{D800}\u{10000}",
            r"(\uD800)\uD800\uDC00",
            r"\u{1f600}\u{DFFF}",
        ] {
            let source = JsString::from(pattern);
            let matcher = RegExpLiteralMatcher::compile_unicode_code_points(&source).unwrap();
            for units in [
                vec![],
                vec![0xd83d],
                vec![0xde00],
                vec![0xd83d, 0xde00],
                vec![0xd83d, 0xd83d, 0xde00, 0xde00],
                vec![0x61, 0xd83d, 0x61, 0xde00, 0x61],
                vec![0xd800, 0xd800, 0xdc00, 0xdfff, 0xd83d, 0xde00, 0xdfff],
            ] {
                let input = JsString::from_code_units(units);
                for start in 0..=input.len() + 1 {
                    for sticky in [false, true] {
                        let ranges =
                            matcher
                                .find_unicode_code_points(&input, start, sticky)
                                .map(|range| {
                                    let mut result = vec![[range.start, range.end]];
                                    result.extend(matcher.capture_ranges().iter().map(|capture| {
                                        [range.start + capture.start, range.start + capture.end]
                                    }));
                                    result
                                });
                        writeln!(rows,"{source:?} input={input:?} start={start} sticky={sticky} ranges={ranges:?}").unwrap();
                    }
                }
            }
        }
        insta::assert_snapshot!(rows);
    }

    #[test]
    fn unicode_surrogate_atom_boundaries_survive_decoding_and_group_flattening() {
        for source in [
            r"(\uD83D)(\uDE00)",
            r"\uD83D(?:)\uDE00",
            r"\u{D83D}\u{DE00}",
            r"\uD83D\u{DE00}",
            r"\u{D83D}\uDE00",
            r"\uD83D()\uDE00",
            "",
            "()",
            r"\uD800+",
            r"[\uD800]",
        ] {
            assert!(
                RegExpLiteralMatcher::compile_unicode_code_points(&JsString::from(source))
                    .is_none(),
                "{source}"
            );
        }
        for unit in [0xd800, 0xdbff, 0xdc00, 0xdfff] {
            let source = JsString::from_code_units(vec![unit]);
            let matcher = RegExpLiteralMatcher::compile_unicode_code_points(&source).unwrap();
            assert_eq!(
                matcher.find_unicode_code_points(&source, 0, true),
                Some(0..1)
            );
            assert!(matcher.find_unicode_scalars(&source, 0, true).is_none());
        }
        let input = JsString::from_code_units(vec![0xd83d, 0xde00, 0xd83d, 0xde00, 0xde00]);
        let matcher =
            RegExpLiteralMatcher::compile_unicode_code_points(&JsString::from(r"\uDE00")).unwrap();
        assert_eq!(
            matcher.find_unicode_code_points(&input, 1, false),
            Some(4..5)
        );
        assert_eq!(matcher.find_unicode_code_points(&input, 1, true), None);
        assert_eq!(matcher.find_unicode_code_points(&input, 6, false), None);
    }

    #[test]
    fn unicode_surrogate_boundary_rejections_resume_linear_search_and_flat_captures() {
        let mut units = [0xd83d, 0xde00].repeat(100000);
        units.push(0xde00);
        let input = JsString::from_code_units(units);
        let source = format!(
            "{}\\u{{{}de00}}{}",
            "(".repeat(100000),
            "0".repeat(100000),
            ")".repeat(100000)
        );
        let matcher =
            RegExpLiteralMatcher::compile_unicode_code_points(&JsString::from(source.as_str()))
                .unwrap();
        let copy = matcher.clone();
        drop(matcher);
        assert_eq!(copy.capture_ranges().len(), 100000);
        assert!(copy.capture_ranges().iter().all(|range| range == &(0..1)));
        assert_eq!(
            copy.find_unicode_code_points(&input, 1, false),
            Some(200000..200001)
        );
        assert_eq!(copy.find_unicode_code_points(&input, 199999, true), None);
    }

    #[test]
    fn unicode_scalar_literal_ranges_snapshot() {
        let mut rows = String::new();
        for pattern in [
            "😀",
            "(😀)",
            "()(😀())",
            "a😀",
            "😀a",
            "(😀)(?:😀)",
            r"\u{1f600}",
            r"(\uD83D\uDE00)",
            r"\u{10000}",
            r"(\uDBFF\uDFFF)",
            r"(\u{61})(\u{1f600})()",
            r"\u{000000000001f600}",
            r"\u{0}\u{1f600}",
            r"((?:\u{1f600})(\u{1f601}))",
            "é😀",
            "😀😀a😀",
        ] {
            let source = JsString::from(pattern);
            let matcher = RegExpLiteralMatcher::compile_unicode_scalars(&source).unwrap();
            for units in [
                vec![],
                vec![0xd83d, 0xde00],
                vec![0x61, 0xd83d, 0xde00, 0x61],
                vec![0xd83d, 0xd83d, 0xde00, 0xde01, 0xd83d],
                vec![0xd800, 0xdc00, 0xdbff, 0xdfff, 0, 0xd83d, 0xde00],
                vec![0xe9, 0xd83d, 0xde00, 0xd83d, 0xde00, 0x61, 0xd83d, 0xde00],
                vec![0xd83d, 0xde00, 0xd83d, 0xde01],
            ] {
                let input = JsString::from_code_units(units);
                for start in 0..=input.len() + 1 {
                    for sticky in [false, true] {
                        let ranges =
                            matcher
                                .find_unicode_scalars(&input, start, sticky)
                                .map(|range| {
                                    let mut result = vec![[range.start, range.end]];
                                    result.extend(matcher.capture_ranges().iter().map(|capture| {
                                        [range.start + capture.start, range.start + capture.end]
                                    }));
                                    result
                                });
                        writeln!(rows,"{source:?} input={input:?} start={start} sticky={sticky} ranges={ranges:?}").unwrap();
                    }
                }
            }
        }
        insta::assert_snapshot!(rows);
    }

    #[test]
    fn unicode_scalar_atoms_cannot_be_joined_across_groups_or_escape_kinds() {
        for source in [
            r"\uD83D",
            r"\uDE00",
            r"(\uD83D)(\uDE00)",
            r"\uD83D(?:)\uDE00",
            r"\u{D83D}\u{DE00}",
            r"\uD83D\u{DE00}",
            r"\u{D83D}\uDE00",
            r"\uD83D\uD83D",
            r"\u{110000}",
            r"\u{}",
            r"\u{G}",
            r"\u{ffffffffffffffff}",
            "",
            "()",
            "[😀]",
            "😀+",
            "😀|a",
            "(?=😀)",
        ] {
            assert!(
                RegExpLiteralMatcher::compile_unicode_scalars(&JsString::from(source)).is_none(),
                "{source}"
            );
        }
        for units in [
            vec![0xd83d],
            vec![0xde00],
            vec![0xd83d, 0x61, 0xde00],
            vec![0xd83d, 0x28, 0x29, 0xde00],
        ] {
            assert!(
                RegExpLiteralMatcher::compile_unicode_scalars(&JsString::from_code_units(units))
                    .is_none()
            );
        }
        assert!(
            RegExpLiteralMatcher::compile(&JsString::from("😀"), false)
                .unwrap()
                .find_unicode_scalars(&JsString::from("😀"), 1, true)
                .is_none()
        );
        for pattern in ["😀", r"\u{1f600}", r"\uD83D\uDE00"] {
            let matcher =
                RegExpLiteralMatcher::compile_unicode_scalars(&JsString::from(pattern)).unwrap();
            assert_eq!(
                matcher.find_unicode_scalars(&JsString::from("😀"), 1, true),
                Some(0..2)
            );
            assert_eq!(
                matcher.find_unicode_scalars(&JsString::from("😀"), 3, false),
                None
            );
            assert_eq!(
                matcher.find_unicode_scalars(
                    &JsString::from_code_units(vec![0xd83d, 0x61, 0xde00]),
                    0,
                    false
                ),
                None
            );
        }
    }

    #[test]
    fn unicode_scalar_literal_flat_storage_handles_deep_groups_long_zeros_and_overlap() {
        let source = format!(
            "{}\\u{{{}1f600}}{}",
            "(".repeat(100000),
            "0".repeat(100000),
            ")".repeat(100000)
        );
        let matcher =
            RegExpLiteralMatcher::compile_unicode_scalars(&JsString::from(source.as_str()))
                .unwrap();
        let copy = matcher.clone();
        drop(matcher);
        assert_eq!(copy.capture_ranges().len(), 100000);
        assert!(copy.capture_ranges().iter().all(|range| range == &(0..2)));
        assert_eq!(
            copy.find_unicode_scalars(
                &JsString::from(format!("{}😀", "a".repeat(100000)).as_str()),
                0,
                false
            ),
            Some(100000..100002)
        );
        let needle = format!("{}a😀", "😀".repeat(100000));
        let input = format!("{}a😀", "😀".repeat(200000));
        let matcher =
            RegExpLiteralMatcher::compile_unicode_scalars(&JsString::from(needle.as_str()))
                .unwrap();
        assert_eq!(
            matcher.find_unicode_scalars(&JsString::from(input.as_str()), 1, false),
            Some(200000..400003)
        );
    }

    #[test]
    fn unicode_braced_bmp_ranges_snapshot() {
        let mut rows = String::new();
        for pattern in [
            r"\u{61}",
            r"(\u{61}())\u{62}",
            r"\u{0}",
            r"\u{D7FF}",
            r"\u{E000}",
            r"\u{FFFF}",
            r"\u{00E9}",
            r"(a)\u{62}",
            r"()\u{1}()",
            r"\u{0000000000000061}",
            r"\u{2028}",
            r"\u{212A}",
        ] {
            let source = JsString::from(pattern);
            let matcher = RegExpLiteralMatcher::compile_bmp_unicode(&source).unwrap();
            for units in [
                vec![],
                vec![0x61],
                vec![0x61, 0x62],
                vec![0xd800, 0xdc00, 0x61, 0x62],
                vec![0xdc00, 0x61, 0xd800],
                vec![0, 1, 0xd7ff, 0xe000, 0xffff, 0xe9, 0x2028, 0x212a],
            ] {
                let input = JsString::from_code_units(units);
                for start in 0..=input.len() + 1 {
                    for sticky in [false, true] {
                        let ranges = matcher.find(&input, start, sticky).map(|range| {
                            let mut rows = vec![[range.start, range.end]];
                            rows.extend(matcher.capture_ranges().iter().map(|capture| {
                                [range.start + capture.start, range.start + capture.end]
                            }));
                            rows
                        });
                        writeln!(rows,"{source:?} input={input:?} start={start} sticky={sticky} ranges={ranges:?}").unwrap();
                    }
                }
            }
        }
        insta::assert_snapshot!(rows);
    }

    #[test]
    fn unicode_braced_bmp_decode_is_checked_and_ordinary_compile_is_separate() {
        let matcher =
            RegExpLiteralMatcher::compile_bmp_unicode(&JsString::from(r"()(\u{61}())\u{62}"))
                .unwrap();
        assert_eq!(matcher.matched_units(), &[0x61, 0x62]);
        assert_eq!(matcher.capture_ranges(), &[0..0, 0..1, 1..1]);
        assert_eq!(matcher.find(&JsString::from("😀ab"), 1, false), Some(2..4));
        assert_eq!(matcher.find(&JsString::from("😀ab"), 1, true), None);
        assert!(RegExpLiteralMatcher::compile(&JsString::from(r"\u{61}"), false).is_none());
        assert!(RegExpLiteralMatcher::compile_unicode_empty(&JsString::from(r"\u{0}")).is_none());
        for pattern in [
            r"\u{}",
            r"\u{",
            r"\u{G}",
            r"\u{10000}",
            r"\u{10ffff}",
            r"\u{110000}",
            r"\u{D800}",
            r"\u{DFFF}",
            r"\u{ffffffffffffffffffffffff}",
            r"[\u{61}]",
        ] {
            assert!(
                RegExpLiteralMatcher::compile_bmp_unicode(&JsString::from(pattern)).is_none(),
                "{pattern}"
            );
        }
    }

    #[test]
    fn unicode_braced_bmp_long_zeros_deep_captures_and_clones_are_flat() {
        let source = format!(
            "{}\\u{{{}61}}{}",
            "(".repeat(100000),
            "0".repeat(100000),
            ")".repeat(100000)
        );
        let matcher =
            RegExpLiteralMatcher::compile_bmp_unicode(&JsString::from(source.as_str())).unwrap();
        let copy = matcher.clone();
        drop(matcher);
        assert_eq!(copy.capture_ranges().len(), 100000);
        assert!(copy.capture_ranges().iter().all(|range| range == &(0..1)));
        assert_eq!(
            copy.find(
                &JsString::from(format!("😀{}a", "b".repeat(100000)).as_str()),
                1,
                false
            ),
            Some(100002..100003)
        );
    }

    #[test]
    fn unicode_empty_boundaries_and_captures() {
        for pattern in ["", "()", "(?:)", "(())()"] {
            let matcher =
                RegExpLiteralMatcher::compile_unicode_empty(&JsString::from(pattern)).unwrap();
            assert!(
                matcher
                    .capture_ranges()
                    .iter()
                    .all(|range| range == &(0..0))
            );
            for (units, expected) in [
                (vec![], vec![Some(0), None]),
                (vec![0xd800, 0xdc00], vec![Some(0), Some(0), Some(2), None]),
                (vec![0xdc00, 0xd800], vec![Some(0), Some(1), Some(2), None]),
                (
                    vec![0x61, 0xd800, 0xdc00, 0x62],
                    vec![Some(0), Some(1), Some(1), Some(3), Some(4), None],
                ),
            ] {
                let input = JsString::from_code_units(units);
                for (start, expected) in expected.into_iter().enumerate() {
                    assert_eq!(
                        matcher.find_unicode_empty(&input, start),
                        expected.map(|point| point..point)
                    );
                }
            }
        }
        for pattern in ["a", "a?", "[]", "^", "(?=)", r"\1"] {
            assert!(
                RegExpLiteralMatcher::compile_unicode_empty(&JsString::from(pattern)).is_none()
            );
        }
        assert!(
            RegExpLiteralMatcher::compile(&JsString::from("a"), false)
                .unwrap()
                .find_unicode_empty(&JsString::from("a"), 0)
                .is_none()
        );
    }

    #[test]
    fn unicode_bmp_literal_ranges_snapshot() {
        let mut rows = String::new();
        for source in [
            "a",
            "ab",
            "aba",
            "a()",
            "(a)b",
            "((a)(?:b))(c)",
            r"\0",
            r"\x7f",
            r"\u0061\x62",
            r"\cA",
            r"\(a\)",
            "()(ab)()",
            "é",
            "(σ())",
            "K",
            r"\u00e9",
            r"\u2028",
            "(?:(µ))",
        ] {
            let source = JsString::from(source);
            let matcher = RegExpLiteralMatcher::compile_bmp_unicode(&source).unwrap();
            for input in [
                vec![],
                vec![0x61],
                vec![0x61, 0x62],
                vec![0x61, 0x62, 0x61],
                vec![0x61, 0x62, 0x63],
                vec![0xd83d, 0xde00, 0x61, 0x62],
                vec![0x61, 0xd83d, 0xde00, 0x62],
                vec![0xd800, 0x61, 0xdc00, 0x62],
                vec![0xdc00, 0x61, 0xd800, 0x62],
                vec![0xd800, 0xd800, 0xdc00, 0xdc00],
                vec![0x41, 0x61, 0x62, 0x61, 0x62],
                vec![0x00, 0x7f, 0x01, 0x61],
                vec![0x28, 0x61, 0x29],
                vec![0x212a, 0x6b, 0x17f, 0x73],
                vec![0xd83d, 0xde00, 0xe9, 0x3c3, 0x212a, 0xb5, 0x2028],
                vec![0xd800, 0xe9, 0xdc00, 0x3a3, 0xb5],
            ] {
                let input = JsString::from_code_units(input);
                for start in 0..=input.len() + 1 {
                    for sticky in [false, true] {
                        let ranges = matcher.find(&input, start, sticky).map(|range| {
                            let mut result = vec![[range.start, range.end]];
                            result.extend(matcher.capture_ranges().iter().map(|capture| {
                                [range.start + capture.start, range.start + capture.end]
                            }));
                            result
                        });
                        writeln!(rows, "{source:?} input={input:?} start={start} sticky={sticky} ranges={ranges:?}").unwrap();
                    }
                }
            }
        }
        insta::assert_snapshot!(rows);
    }

    #[test]
    fn unicode_bmp_boundary_ranges_and_surrogate_empty_bodies_are_separate() {
        let input =
            JsString::from_code_units(vec![0xd83d, 0xde00, 0x61, 0x62, 0xd800, 0x61, 0xdc00]);
        let matcher =
            RegExpLiteralMatcher::compile_bmp_unicode(&JsString::from("()(a())b")).unwrap();
        assert_eq!(matcher.find(&input, 1, false), Some(2..4));
        assert_eq!(matcher.find(&input, 1, true), None);
        assert_eq!(matcher.find(&input, 2, true), Some(2..4));
        assert_eq!(matcher.capture_ranges(), &[0..0, 0..1, 1..1]);
        assert_eq!(matcher.find(&input, 4, false), None);
        for source in [
            "",
            "()",
            "(?:)",
            r"\uD800",
            r"\uD83D\uDE00",
            r"\u{1f600}",
            ".",
            "[a]",
            "a?",
            "a|b",
            "^a",
            r"(a)\1",
        ] {
            assert!(
                RegExpLiteralMatcher::compile_bmp_unicode(&JsString::from(source)).is_none(),
                "{source}"
            );
        }
    }

    #[test]
    fn unicode_bmp_deep_capture_programs_clones_and_search_stay_flat() {
        let depth = 100_000;
        let source = format!("{}a{}", "(".repeat(depth), ")".repeat(depth));
        let matcher =
            RegExpLiteralMatcher::compile_bmp_unicode(&JsString::from(source.as_str())).unwrap();
        assert_eq!(matcher.capture_ranges().len(), depth);
        assert!(
            matcher
                .capture_ranges()
                .iter()
                .all(|range| *range == (0..1))
        );
        let copy = matcher.clone();
        drop(matcher);
        let mut units = vec![0xd83d, 0xde00];
        units.extend(std::iter::repeat_n(0x62, 100_000));
        units.push(0x61);
        assert_eq!(
            copy.find(&JsString::from_code_units(units), 1, false),
            Some(100_002..100_003)
        );
    }

    #[test]
    fn literal_compilation_and_search_snapshot() {
        let cases = [
            ("", "abc"),
            ("a", "baab"),
            ("aba", "aababa"),
            ("ababc", "ababababc"),
            ("aaab", "aaaaaaaa"),
            ("a/b-c", "xa/b-c"),
            (r"\$\(\)\*\+\.\?\[\]\^\{\|\}\\\/\-", "$()*+.?[]^{|}\\/-"),
            (r"\f\n\r\t\v", "\u{c}\n\r\t\u{b}"),
            (r"\cA\cz\cZ", "\u{1}\u{1a}\u{1a}"),
            (r"\x61\u0062\0", "ab\0"),
            (r"\uD800", "x"),
            (r"\uD83D\uDCA9", "x💩y"),
            ("💩", "x💩y"),
            ("\n\u{2028}", "x\n\u{2028}"),
            ("abc", "xABC"),
            ("µ", "xΜ"),
            ("σ", "xς"),
            ("s", "xſ"),
            ("k", "xK"),
            ("ß", "SS"),
            ("ß", "ẞ"),
            ("ΐ", "Ι\u{308}\u{301}"),
            ("(a)", "a"),
            ("(?:)", ""),
            ("a|b", "a"),
            ("[a]", "a"),
            (".", "a"),
            ("^a$", "a"),
            ("a*", "aaa"),
            ("a+", "aaa"),
            ("a?", "a"),
            ("a{2}", "aa"),
            (r"\b", "a"),
            (r"\B", "a"),
            (r"\d", "1"),
            (r"\s", " "),
            (r"\w", "a"),
            (r"\p{ASCII}", "a"),
            (r"\u{61}", "a"),
            (r"\1", "1"),
            (r"\01", "\u{1}"),
            (r"\c1", "1"),
            (r"\x0", "0"),
            (r"\u000", "0"),
            (r"\a", "a"),
            (r"\", ""),
        ];
        let mut rows = String::new();
        for (source, input) in cases {
            for ignore_case in [false, true] {
                let source = JsString::from(source);
                let input = JsString::from(input);
                write!(rows, "{source:?} i={ignore_case} input={input:?}").unwrap();
                if let Some(matcher) = RegExpLiteralMatcher::compile(&source, ignore_case) {
                    for start in [0, 1, input.len(), input.len() + 1] {
                        write!(
                            rows,
                            " {start}:{:?}/{:?}",
                            matcher.find(&input, start, false),
                            matcher.find(&input, start, true)
                        )
                        .unwrap();
                    }
                } else {
                    rows.push_str(" unsupported");
                }
                rows.push('\n');
            }
        }
        insta::assert_snapshot!(rows);
    }

    fn words(max_length: usize) -> Vec<JsString> {
        let alphabet = [0x61, 0x41, 0xd800, 0x3c2];
        let mut result = vec![JsString::default()];
        let mut previous = vec![Vec::new()];
        for _ in 0..max_length {
            let mut next = Vec::new();
            for word in previous {
                for unit in alphabet {
                    let mut word = word.clone();
                    word.push(unit);
                    result.push(JsString::from_code_units(word.clone()));
                    next.push(word);
                }
            }
            previous = next;
        }
        result
    }

    #[test]
    fn search_agrees_with_independent_sliding_window_oracle() {
        let inputs = words(5);
        for source in words(3) {
            for ignore_case in [false, true] {
                let matcher = RegExpLiteralMatcher::compile(&source, ignore_case).unwrap();
                for input in &inputs {
                    for start in 0..=input.len() + 1 {
                        for sticky in [false, true] {
                            let expected = (start..=input.len()).find_map(|offset| {
                                if sticky && offset != start {
                                    return None;
                                }
                                let end = offset.checked_add(source.len())?;
                                let candidate = input.code_units().get(offset..end)?;
                                candidate
                                    .iter()
                                    .zip(source.code_units())
                                    .all(|(&a, &b)| {
                                        canonicalize(a, ignore_case) == canonicalize(b, ignore_case)
                                    })
                                    .then_some(offset..end)
                            });
                            assert_eq!(
                                matcher.find(input, start, sticky),
                                expected,
                                "{source:?} {input:?} start={start} i={ignore_case} y={sticky}"
                            );
                        }
                    }
                }
                assert_eq!(matcher.find(&JsString::default(), usize::MAX, false), None);
            }
        }
    }

    #[test]
    fn matching_preserves_code_units_and_ordinary_case_boundaries() {
        for (source, input, expected) in [
            (vec![0xd800], vec![0xd800, 0xdc00], Some(0..1)),
            (vec![0xdc00], vec![0xd800, 0xdc00], Some(1..2)),
            (vec![0x3c3], vec![0x3c2], Some(0..1)),
            (vec![0x73], vec![0x17f], None),
            (vec![0x6b], vec![0x212a], None),
            (vec![0xdf], vec![0x53, 0x53], None),
            (vec![0xdf], vec![0x1e9e], None),
        ] {
            let matcher =
                RegExpLiteralMatcher::compile(&JsString::from_code_units(source), true).unwrap();
            assert_eq!(
                matcher.find(&JsString::from_code_units(input), 0, false),
                expected
            );
        }
    }

    #[test]
    fn long_repeated_prefixes_and_cloned_programs_have_no_default_size_cap() {
        let source = JsString::from(format!("{}b", "a".repeat(60_000)).as_str());
        let matcher = RegExpLiteralMatcher::compile(&source, false).unwrap();
        let clone = matcher.clone();
        assert!(Arc::ptr_eq(&matcher.0, &clone.0));
        drop(matcher);
        let input = JsString::from(format!("{}b", "a".repeat(120_000)).as_str());
        assert_eq!(clone.find(&input, 0, false), Some(60_000..120_001));
        assert_eq!(clone.find(&input, 0, true), None);
        let miss = JsString::from(format!("{}c", "a".repeat(120_000)).as_str());
        assert_eq!(clone.find(&miss, 0, false), None);
    }

    #[test]
    fn noncapturing_group_compilation_snapshot() {
        let mut rows = String::new();
        for source in [
            "(?:)",
            "a(?:)b",
            "(?:ab)",
            "(?:a(?:b)c)",
            "(?:(?:))",
            "(?:a)(?:b)",
            r"(?:\(\))",
            r"(?:\uD83D)(?:\uDCA9)",
            "(?:σ)",
            "(?:s)",
            "(?:a|b)",
            "(?:a*)",
            "(?:a)*",
            "(?:a){1}",
            "(?:a)?",
            "(?i:a)",
            "(?=a)",
            "(?!a)",
            "(?<=a)",
            "(?<!a)",
            "(?<x>a)",
            "(a)",
            "(?:a",
            ")",
            "(?:))",
        ] {
            for ignore_case in [false, true] {
                let source = JsString::from(source);
                write!(rows, "{source:?} i={ignore_case}").unwrap();
                if let Some(matcher) = RegExpLiteralMatcher::compile(&source, ignore_case) {
                    for input in ["", "xabcy", "AB", "()", "x💩y", "ς", "ſ"] {
                        let input = JsString::from(input);
                        write!(
                            rows,
                            " {input:?}:{:?}/{:?}",
                            matcher.find(&input, 0, false),
                            matcher.find(&input, 0, true)
                        )
                        .unwrap();
                    }
                } else {
                    rows.push_str(" unsupported");
                }
                rows.push('\n');
            }
        }
        insta::assert_snapshot!(rows);
    }

    #[test]
    fn nested_groups_agree_with_independent_sliding_window_oracle() {
        let inputs = words(4);
        for units in words(2) {
            let mut source = "(?:(?:)".encode_utf16().collect::<Vec<_>>();
            for &unit in units.code_units() {
                source.extend("(?:".encode_utf16());
                source.push(unit);
                source.push(u16::from(b')'));
            }
            source.extend(")(?:)".encode_utf16());
            let source = JsString::from_code_units(source);
            for ignore_case in [false, true] {
                let matcher = RegExpLiteralMatcher::compile(&source, ignore_case).unwrap();
                for input in &inputs {
                    for start in 0..=input.len() + 1 {
                        for sticky in [false, true] {
                            let expected = (start..=input.len()).find_map(|offset| {
                                if sticky && offset != start {
                                    return None;
                                }
                                let end = offset.checked_add(units.len())?;
                                input
                                    .code_units()
                                    .get(offset..end)?
                                    .iter()
                                    .zip(units.code_units())
                                    .all(|(&a, &b)| {
                                        canonicalize(a, ignore_case) == canonicalize(b, ignore_case)
                                    })
                                    .then_some(offset..end)
                            });
                            assert_eq!(
                                matcher.find(input, start, sticky),
                                expected,
                                "{source:?} {input:?} start={start} i={ignore_case} y={sticky}"
                            );
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn deeply_nested_noncapturing_groups_compile_match_and_drop_iteratively() {
        let source = format!("{}a{}", "(?:".repeat(100_000), ")".repeat(100_000));
        let matcher =
            RegExpLiteralMatcher::compile(&JsString::from(source.as_str()), false).unwrap();
        assert_eq!(matcher.0.units, [u16::from(b'a')]);
        assert_eq!(matcher.find(&JsString::from("ba"), 0, false), Some(1..2));
        assert_eq!(matcher.find(&JsString::from("ba"), 0, true), None);
        drop(matcher);
    }

    #[test]
    fn ordinary_identity_escape_compilation_snapshot() {
        let mut rows = String::new();
        for source in [
            r"\!\#\%\&\,\:\;\<\=\>\@\`\~",
            "\\ ",
            "\\\t",
            "\\\n",
            "\\\u{a0}",
            "\\\u{2028}",
            "\\\u{2603}",
            r"\$\-\/\\",
            r"(?:\!)(?:\ )",
            r"\a",
            r"\_",
            "\\\u{200c}",
            "\\\u{200d}",
            "\\\u{3b1}",
            "\\\u{301}",
            "\\\u{660}",
        ] {
            for ignore_case in [false, true] {
                let source = JsString::from(source);
                write!(rows, "{source:?} i={ignore_case}").unwrap();
                if let Some(matcher) = RegExpLiteralMatcher::compile(&source, ignore_case) {
                    write!(
                        rows,
                        " units={:?}",
                        JsString::from_code_units(matcher.0.units.clone())
                    )
                    .unwrap();
                } else {
                    rows.push_str(" unsupported");
                }
                rows.push('\n');
            }
        }
        for unit in [0xd800, 0xdc00] {
            let source = JsString::from_code_units(vec![u16::from(b'\\'), unit]);
            let matcher = RegExpLiteralMatcher::compile(&source, false).unwrap();
            writeln!(
                rows,
                "{source:?} units={:?}",
                JsString::from_code_units(matcher.0.units.clone())
            )
            .unwrap();
        }
        insta::assert_snapshot!(rows);
    }

    #[test]
    fn identity_escape_matching_preserves_every_supported_code_unit() {
        for unit in 0..=u16::MAX {
            // These productions are ControlEscape or the zero CharacterEscape,
            // whose semantics are covered separately, not IdentityEscape.
            if matches!(unit, 0x30 | 0x66 | 0x6e | 0x72 | 0x74 | 0x76) {
                continue;
            }
            let source = JsString::from_code_units(vec![u16::from(b'\\'), unit]);
            if let Some(matcher) = RegExpLiteralMatcher::compile(&source, false) {
                assert_eq!(matcher.0.units, [unit]);
                let input = JsString::from_code_units(vec![0x61, unit, 0x62]);
                assert_eq!(matcher.find(&input, 0, false), Some(1..2));
                assert_eq!(matcher.find(&input, 0, true), None);
                assert_eq!(matcher.find(&input, 1, true), Some(1..2));
            }
        }
    }

    #[test]
    fn ordinary_capture_compilation_snapshot() {
        let mut rows = String::new();
        for source in [
            "(a)",
            "((a)b)",
            "(a)(b)",
            "a(b(c))d",
            "()",
            "(())",
            "(?:a)(b)(?:c)",
            "(a(?:b)c)",
            "(?:())()",
            r"(\(x\))",
            r"(\uD83D)(\uDCA9)",
            "(a|b)",
            "(a)*",
            "(?<x>a)",
            "(?i:a)",
            "(?=a)",
            "((a)",
            ")",
        ] {
            for ignore_case in [false, true] {
                let source = JsString::from(source);
                write!(rows, "{source:?} i={ignore_case}").unwrap();
                if let Some(matcher) = RegExpLiteralMatcher::compile(&source, ignore_case) {
                    write!(
                        rows,
                        " units={:?} captures={:?}",
                        JsString::from_code_units(matcher.0.units.clone()),
                        matcher.capture_ranges()
                    )
                    .unwrap();
                } else {
                    rows.push_str(" unsupported");
                }
                rows.push('\n');
            }
        }
        insta::assert_snapshot!(rows);
    }

    #[test]
    fn captures_follow_opening_order_and_decoded_character_offsets() {
        for (source, expected) in [
            ("((a)b)", vec![0..2, 0..1]),
            ("a(b(c))d", vec![1..3, 2..3]),
            ("()a(())", vec![0..0, 1..1, 1..1]),
            ("(?:a)(b)(?:c)", std::iter::once(1..2).collect()),
            (r"(\x61)(\u0062)", vec![0..1, 1..2]),
            (r"(\uD83D)(\uDCA9)", vec![0..1, 1..2]),
        ] {
            let matcher = RegExpLiteralMatcher::compile(&JsString::from(source), false).unwrap();
            assert_eq!(matcher.capture_ranges(), expected);
        }
        let source = "(abc)".repeat(1000);
        let matcher =
            RegExpLiteralMatcher::compile(&JsString::from(source.as_str()), false).unwrap();
        assert_eq!(matcher.capture_ranges().len(), 1000);
        for (index, range) in matcher.capture_ranges().iter().enumerate() {
            assert_eq!(range, &(index * 3..index * 3 + 3));
        }
    }

    #[test]
    fn deeply_nested_captures_compile_clone_and_drop_without_recursion() {
        let source = format!("{}a{}", "(".repeat(100_000), ")".repeat(100_000));
        let matcher =
            RegExpLiteralMatcher::compile(&JsString::from(source.as_str()), false).unwrap();
        assert_eq!(matcher.capture_ranges().len(), 100_000);
        assert!(
            matcher
                .capture_ranges()
                .iter()
                .all(|range| range == &(0..1))
        );
        let clone = matcher.clone();
        assert!(Arc::ptr_eq(&matcher.0, &clone.0));
        drop(matcher);
        assert_eq!(clone.find(&JsString::from("ba"), 0, false), Some(1..2));
        drop(clone);
    }
}
