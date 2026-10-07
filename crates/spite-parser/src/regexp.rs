//! Literal flags and core Pattern validation with ordinary classes and names.

use spite_core::{
    Diagnostic, DiagnosticKind, JsString, Span, is_identifier_part, is_identifier_start,
};
use std::{cmp::Ordering, collections::HashSet, mem, ops::Range};

mod properties;
mod property_data;
mod sets;

type Failure = (DiagnosticKind, &'static str);

fn syntax(message: &'static str) -> Failure {
    (DiagnosticKind::Syntax, message)
}

#[derive(Clone, Copy)]
struct Mode {
    unicode: bool,
    sets: bool,
}

#[derive(Default)]
struct Names {
    alternatives: HashSet<String>,
    current: HashSet<String>,
}

impl Names {
    fn add(&mut self, name: String) -> Result<(), Failure> {
        if !self.current.insert(name) {
            return Err(syntax(
                "regular expression capture names might both participate",
            ));
        }
        Ok(())
    }

    fn extend(&mut self, names: HashSet<String>) -> Result<(), Failure> {
        // Captures in separate terms of one alternative can both participate,
        // even when each term contains its own inner disjunction.
        if !self.current.is_disjoint(&names) {
            return Err(syntax(
                "regular expression capture names might both participate",
            ));
        }
        union_names(&mut self.current, names);
        Ok(())
    }

    fn next_alternative(&mut self) {
        // A separating disjunction permits repeated names across alternatives.
        // https://262.ecma-international.org/17.0/#sec-mightbothparticipate
        union_names(&mut self.alternatives, mem::take(&mut self.current));
    }

    fn finish(mut self) -> HashSet<String> {
        self.next_alternative();
        self.alternatives
    }
}

fn union_names(target: &mut HashSet<String>, mut source: HashSet<String>) {
    // Move the larger allocation rather than rehashing every name at each
    // enclosing group. This also keeps deeply nested names iterative.
    if target.len() < source.len() {
        mem::swap(target, &mut source);
    }
    target.extend(source);
}

#[derive(Default)]
struct Group {
    assertion: bool,
    names: Names,
}

/// One decoded named capture and its source-order capture index.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RegExpNamedCapture {
    /// Decoded CapturingGroupName, without normalization or case folding.
    pub name: JsString,
    /// One-based capture index, including preceding unnamed captures.
    pub index: u32,
    /// UTF-16 GroupSpecifier range, after `(` through the closing `>`.
    pub specifier: Range<usize>,
}

/// Capture metadata retained after the complete Pattern and flags are valid.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RegExpPatternMetadata {
    /// CountLeftCapturingParensWithin for the complete Pattern.
    pub capture_count: u32,
    /// Named captures in source order, including permitted duplicate names.
    pub named_captures: Vec<RegExpNamedCapture>,
}

/// Parses a UTF-16 Pattern and flags, retaining named capture metadata.
///
/// Names use CapturingGroupName's Unicode identifier rules in every mode.
/// Duplicate-name and reference early errors remain the same as validation.
/// Specifier offsets always refer to the original UTF-16 body, including when
/// Unicode-mode parsing treats surrogate pairs as single code points.
pub fn parse_regexp_pattern(
    body: &JsString,
    flags: &JsString,
    span: Span,
) -> Result<RegExpPatternMetadata, Diagnostic> {
    let mode =
        pattern_mode(flags).map_err(|failure| Diagnostic::new(failure.0, span, failure.1))?;
    let mut pattern = Pattern::new(body, mode);
    pattern.named_captures = Some(Vec::new());
    pattern
        .validate()
        .map_err(|failure| Diagnostic::new(failure.0, span, failure.1))?;
    let mut point = 0;
    let mut unit = 0;
    let named_captures = pattern
        .named_captures
        .take()
        .expect("metadata requested")
        .into_iter()
        .map(|mut capture| {
            // Every GroupSpecifier is disjoint and follows the previous one.
            // Translate code-point boundaries with one forward UTF-16 scan.
            while point < capture.specifier.start {
                unit += if pattern.points[point] > 0xffff { 2 } else { 1 };
                point += 1;
            }
            let start = unit;
            while point < capture.specifier.end {
                unit += if pattern.points[point] > 0xffff { 2 } else { 1 };
                point += 1;
            }
            capture.specifier = start..unit;
            capture
        })
        .collect();
    Ok(RegExpPatternMetadata {
        capture_count: pattern.captures,
        named_captures,
    })
}

/// Validates a UTF-16 Pattern and its flags, returning the capturing-group count.
///
/// The body has no literal delimiters and may contain raw line terminators or
/// lone surrogates. Diagnostics use the caller's source span; matching is a
/// separate operation.
pub fn validate_regexp_pattern(
    body: &JsString,
    flags: &JsString,
    span: Span,
) -> Result<u32, Diagnostic> {
    let mode =
        pattern_mode(flags).map_err(|failure| Diagnostic::new(failure.0, span, failure.1))?;
    let mut pattern = Pattern::new(body, mode);
    pattern
        .validate()
        .map_err(|failure| Diagnostic::new(failure.0, span, failure.1))?;
    Ok(pattern.captures)
}

fn pattern_mode(flags: &JsString) -> Result<Mode, Failure> {
    // https://262.ecma-international.org/17.0/#sec-isvalidregularexpressionliteral
    // checks flags and duplicates before #sec-parsepattern rejects simultaneous
    // Unicode modes.
    const FLAGS: &[u8] = b"dgimsuvy";
    const UNICODE_MODES: u8 = (1 << 5) | (1 << 6);
    let mut seen = 0u8;
    for unit in flags.code_units() {
        let Some(index) = FLAGS.iter().position(|flag| u16::from(*flag) == *unit) else {
            return Err(syntax("invalid regular expression flag"));
        };
        let bit = 1u8 << index;
        if seen & bit != 0 {
            return Err(syntax("duplicate regular expression flag"));
        }
        seen |= bit;
    }
    if seen & UNICODE_MODES == UNICODE_MODES {
        return Err(syntax(
            "regular expression flags u and v are mutually exclusive",
        ));
    }
    Ok(Mode {
        unicode: seen & UNICODE_MODES != 0,
        sets: seen & (1 << 6) != 0,
    })
}

struct Pattern {
    points: Vec<u32>,
    pos: usize,
    mode: Mode,
    captures: u32,
    largest_reference: Option<Range<usize>>,
    named_references: HashSet<String>,
    named_captures: Option<Vec<RegExpNamedCapture>>,
}

impl Pattern {
    fn new(body: &JsString, mode: Mode) -> Self {
        // ParsePattern interprets UTF-16 units separately without u/v. In either
        // Unicode mode, decode pairs while retaining unpaired surrogates.
        let points = if mode.unicode {
            char::decode_utf16(body.code_units().iter().copied())
                .map(|point| {
                    point
                        .map(u32::from)
                        .unwrap_or_else(|e| u32::from(e.unpaired_surrogate()))
                })
                .collect()
        } else {
            body.code_units()
                .iter()
                .map(|unit| u32::from(*unit))
                .collect()
        };
        Self {
            points,
            pos: 0,
            mode,
            captures: 0,
            largest_reference: None,
            named_references: HashSet::new(),
            named_captures: None,
        }
    }

    fn peek(&self) -> Option<u32> {
        self.points.get(self.pos).copied()
    }

    fn eat(&mut self, ascii: u8) -> bool {
        if self.peek() == Some(u32::from(ascii)) {
            self.pos += 1;
            true
        } else {
            false
        }
    }

    fn validate(&mut self) -> Result<(), Failure> {
        // https://262.ecma-international.org/17.0/#sec-patterns
        // Each entry retains the enclosing disjunction's names and whether the
        // group is an Assertion. No native recursion quota is required.
        let mut groups = Vec::new();
        let mut current = Group::default();
        let mut can_quantify = false;
        while let Some(point) = self.peek() {
            self.pos += 1;
            match point {
                0x28 => {
                    // (
                    let (assertion, name) = self.group()?;
                    if let Some(name) = name {
                        // The GroupSpecifier is outside its own Disjunction.
                        // Its name can therefore participate with any inner name.
                        current.names.add(name)?;
                    }
                    groups.push(mem::replace(
                        &mut current,
                        Group {
                            assertion,
                            names: Names::default(),
                        },
                    ));
                    can_quantify = false;
                }
                0x29 => {
                    // )
                    let Some(parent) = groups.pop() else {
                        return Err(syntax("unmatched regular expression closing parenthesis"));
                    };
                    can_quantify = !current.assertion;
                    let names = current.names.finish();
                    current = parent;
                    current.names.extend(names)?;
                }
                0x7c => {
                    // |
                    current.names.next_alternative();
                    can_quantify = false;
                }
                0x5e | 0x24 => can_quantify = false, // ^ $
                0x2a | 0x2b | 0x3f | 0x7b => {
                    // * + ? {
                    if !can_quantify {
                        return Err(syntax("regular expression quantifier requires an atom"));
                    }
                    if point == 0x7b {
                        self.braced_quantifier()?;
                    }
                    self.eat(b'?'); // One optional lazy suffix, never a second quantifier.
                    can_quantify = false;
                }
                0x5c => can_quantify = self.escape()?, // \
                0x5b => {
                    self.character_class()?;
                    can_quantify = true;
                }
                0x5d | 0x7d => {
                    return Err(syntax("unexpected regular expression syntax character"));
                }
                _ => can_quantify = true,
            }
        }
        if !groups.is_empty() {
            return Err(syntax("unterminated regular expression group"));
        }
        let names = current.names.finish();
        // GroupSpecifiersThatMatch permits forward and alternative references.
        // https://262.ecma-international.org/17.0/#sec-groupspecifiersthatmatch
        if !self.named_references.is_subset(&names) {
            return Err(syntax(
                "regular expression named backreference has no capture",
            ));
        }
        // DecimalEscape permits forward references. Compare only after the
        // complete supported Pattern has established its capture count.
        if let Some(reference) = &self.largest_reference {
            let count: Vec<_> = self.captures.to_string().bytes().map(u32::from).collect();
            if decimal_cmp(&self.points[reference.clone()], &count) == Ordering::Greater {
                return Err(syntax(
                    "regular expression backreference exceeds the capture count",
                ));
            }
        }
        Ok(())
    }

    fn capture(&mut self) -> Result<(), Failure> {
        // 22.2.1.1 rejects CountLeftCapturingParensWithin >= 2^32 - 1.
        // This is a grammar early error, not a host resource quota.
        if self.captures >= u32::MAX - 2 {
            return Err(syntax("too many regular expression capturing groups"));
        }
        self.captures += 1;
        Ok(())
    }

    fn group(&mut self) -> Result<(bool, Option<String>), Failure> {
        let specifier_start = self.pos;
        if !self.eat(b'?') {
            self.capture()?;
            return Ok((false, None));
        }
        if self.eat(b'=') || self.eat(b'!') {
            return Ok((true, None));
        }
        if self.eat(b'<') {
            if self.eat(b'=') || self.eat(b'!') {
                return Ok((true, None));
            }
            let name = self.group_name()?;
            self.capture()?;
            if let Some(captures) = &mut self.named_captures {
                captures.push(RegExpNamedCapture {
                    name: JsString::from(name.as_str()),
                    index: self.captures,
                    specifier: specifier_start..self.pos,
                });
            }
            return Ok((false, Some(name)));
        }
        // Includes (?:...), whose first modifier list is empty.
        let enabled = self.modifiers()?;
        let disabled = if self.eat(b'-') {
            Some(self.modifiers()?)
        } else {
            None
        };
        if !self.eat(b':') {
            return Err(syntax("invalid regular expression group prefix"));
        }
        if disabled.is_some_and(|disabled| enabled | disabled == 0 || enabled & disabled != 0) {
            return Err(syntax("invalid regular expression modifier groups"));
        }
        Ok((false, None))
    }

    // The opening '<' has been consumed. CapturingGroupName decodes all names
    // without normalizing or case-folding their identifier code points.
    fn group_name(&mut self) -> Result<String, Failure> {
        // https://262.ecma-international.org/17.0/#sec-regexpidentifiercodepoint
        // https://262.ecma-international.org/17.0/#sec-static-semantics-capturinggroupname
        let mut name = String::new();
        loop {
            let Some(mut point) = self.peek() else {
                return Err(syntax("unterminated regular expression group name"));
            };
            self.pos += 1;
            if point == u32::from(b'>') {
                if name.is_empty() {
                    return Err(syntax("empty regular expression group name"));
                }
                return Ok(name);
            }
            if point == u32::from(b'\\') {
                if !self.eat(b'u') {
                    return Err(syntax(
                        "regular expression group name requires a Unicode escape",
                    ));
                }
                // RegExpIdentifierStart/Part explicitly use +UnicodeMode for
                // escapes even when the enclosing Pattern has no u/v flag.
                point = self.unicode_escape(true)?;
            } else if !self.mode.unicode && (0xd800..=0xdbff).contains(&point) {
                if let Some(trail) = self
                    .peek()
                    .filter(|point| (0xdc00..=0xdfff).contains(point))
                {
                    self.pos += 1;
                    point = 0x10000 + (point - 0xd800) * 0x400 + trail - 0xdc00;
                }
            }
            let Some(character) = char::from_u32(point).filter(|character| {
                if name.is_empty() {
                    is_identifier_start(*character)
                } else {
                    is_identifier_part(*character)
                }
            }) else {
                return Err(syntax("invalid regular expression group name identifier"));
            };
            name.push(character);
        }
    }

    fn modifiers(&mut self) -> Result<u8, Failure> {
        let mut seen = 0;
        while let Some(point) = self.peek() {
            let bit = match point {
                0x69 => 1,
                0x6d => 2,
                0x73 => 4,
                _ => break,
            };
            if seen & bit != 0 {
                return Err(syntax("duplicate regular expression modifier"));
            }
            seen |= bit;
            self.pos += 1;
        }
        Ok(seen)
    }

    fn digits(&mut self) -> Result<Range<usize>, Failure> {
        let start = self.pos;
        while self
            .peek()
            .is_some_and(|point| (0x30..=0x39).contains(&point))
        {
            self.pos += 1;
        }
        if self.pos == start {
            Err(syntax("expected regular expression decimal digits"))
        } else {
            Ok(start..self.pos)
        }
    }

    fn braced_quantifier(&mut self) -> Result<(), Failure> {
        let min = self.digits()?;
        if self.eat(b',') && self.peek() != Some(u32::from(b'}')) {
            let max = self.digits()?;
            if decimal_cmp(&self.points[min], &self.points[max]) == Ordering::Greater {
                return Err(syntax("regular expression quantifier bounds are reversed"));
            }
        }
        if !self.eat(b'}') {
            return Err(syntax("unterminated regular expression quantifier"));
        }
        Ok(())
    }

    fn escape(&mut self) -> Result<bool, Failure> {
        let Some(point) = self.peek() else {
            return Err(syntax("unterminated regular expression escape"));
        };
        self.pos += 1;
        match point {
            0x62 | 0x42 => return Ok(false), // b B assertions
            0x31..=0x39 => {
                self.pos -= 1;
                let reference = self.digits()?;
                if self.largest_reference.as_ref().is_none_or(|largest| {
                    decimal_cmp(
                        &self.points[reference.clone()],
                        &self.points[largest.clone()],
                    ) == Ordering::Greater
                }) {
                    self.largest_reference = Some(reference);
                }
            }
            0x64 | 0x44 | 0x73 | 0x53 | 0x77 | 0x57 => {} // d D s S w W
            0x70 | 0x50 if self.mode.unicode => {
                self.property_escape(point == 0x50)?;
            }
            0x6b => {
                if !self.eat(b'<') {
                    return Err(syntax(
                        "regular expression named backreference requires a group name",
                    ));
                }
                let name = self.group_name()?;
                self.named_references.insert(name);
            }
            _ => {
                self.character_escape(point)?;
            }
        }
        Ok(true)
    }

    fn character_escape(&mut self, point: u32) -> Result<u32, Failure> {
        // CharacterValue is shared by ordinary atoms and class range endpoints.
        // https://262.ecma-international.org/17.0/#sec-patterns-static-semantics-character-value
        match point {
            0x66 => Ok(12), // f
            0x6e => Ok(10), // n
            0x72 => Ok(13), // r
            0x74 => Ok(9),  // t
            0x76 => Ok(11), // v
            0x63 => {
                // c AsciiLetter
                if !self
                    .peek()
                    .is_some_and(|point| matches!(point, 0x41..=0x5a | 0x61..=0x7a))
                {
                    return Err(syntax("invalid regular expression control escape"));
                }
                let letter = self.points[self.pos];
                self.pos += 1;
                Ok(letter % 32)
            }
            0x30 => {
                if self
                    .peek()
                    .is_some_and(|point| (0x30..=0x39).contains(&point))
                {
                    return Err(syntax(
                        "legacy regular expression octal escapes are not in the core grammar",
                    ));
                }
                Ok(0)
            }
            0x78 => self.hex_digits(2), // x Hex2Digits
            0x75 => self.unicode_escape(self.mode.unicode),
            _ => {
                let valid = if self.mode.unicode {
                    matches!(
                        point,
                        0x5e | 0x24
                            | 0x5c
                            | 0x2e
                            | 0x2a
                            | 0x2b
                            | 0x3f
                            | 0x28
                            | 0x29
                            | 0x5b
                            | 0x5d
                            | 0x7b
                            | 0x7d
                            | 0x7c
                            | 0x2f
                    )
                } else {
                    // IdentityEscape excludes Unicode ID_Continue, which differs
                    // from IdentifierPartChar exactly by the added '$'.
                    point == 0x24 || !char::from_u32(point).is_some_and(is_identifier_part)
                };
                if !valid {
                    return Err(syntax("invalid regular expression identity escape"));
                }
                Ok(point)
            }
        }
    }

    fn unicode_escape(&mut self, unicode: bool) -> Result<u32, Failure> {
        if unicode && self.eat(b'{') {
            let start = self.pos;
            let mut value = 0u32;
            while let Some(digit) = self.peek().and_then(hex_value) {
                value = value
                    .checked_mul(16)
                    .and_then(|v| v.checked_add(digit))
                    .filter(|v| *v <= 0x10ffff)
                    .ok_or_else(|| syntax("regular expression Unicode escape is out of range"))?;
                self.pos += 1;
            }
            if self.pos == start || !self.eat(b'}') {
                return Err(syntax("invalid regular expression Unicode escape"));
            }
            Ok(value)
        } else {
            let lead = self.hex_digits(4)?;
            if unicode && (0xd800..=0xdbff).contains(&lead) {
                // Pair only the nearest following \\u HexTrailSurrogate,
                // never a brace escape or a raw surrogate source unit.
                let trail = self.points[self.pos..]
                    .strip_prefix(&[0x5c, 0x75])
                    .and_then(|rest| rest.get(..4))
                    .and_then(|hex| {
                        hex.iter()
                            .try_fold(0u32, |value, point| Some(value * 16 + hex_value(*point)?))
                    })
                    .filter(|value| (0xdc00..=0xdfff).contains(value));
                if let Some(trail) = trail {
                    self.pos += 6;
                    return Ok(0x10000 + (lead - 0xd800) * 0x400 + trail - 0xdc00);
                }
            }
            Ok(lead)
        }
    }

    fn hex_digits(&mut self, count: usize) -> Result<u32, Failure> {
        let mut value = 0;
        for _ in 0..count {
            let Some(digit) = self.peek().and_then(hex_value) else {
                return Err(syntax("invalid regular expression hexadecimal escape"));
            };
            // All callers request either two or four hexadecimal digits.
            value = value * 16 + digit;
            self.pos += 1;
        }
        Ok(value)
    }

    fn character_class(&mut self) -> Result<(), Failure> {
        // NonemptyClassRanges and NonemptyClassRangesNoDash traverse ranges
        // left to right. A trailing '-' is a ClassAtom, not a range separator.
        // https://262.ecma-international.org/17.0/#sec-patterns
        // https://262.ecma-international.org/17.0/#sec-patterns-static-semantics-early-errors
        if self.mode.sets {
            return self.unicode_sets_class();
        }
        self.eat(b'^');
        while !self.eat(b']') {
            let left = self.class_atom()?;
            if self.peek() == Some(u32::from(b'-'))
                && self.points.get(self.pos + 1) != Some(&u32::from(b']'))
            {
                self.pos += 1;
                let right = self.class_atom()?;
                // Class ranges cannot have CharacterClassEscape endpoints,
                // including in non-Unicode mode without Annex B extensions.
                let (Some(left), Some(right)) = (left, right) else {
                    return Err(syntax(
                        "regular expression class range requires character endpoints",
                    ));
                };
                if left > right {
                    return Err(syntax("regular expression class range is reversed"));
                }
            }
        }
        Ok(())
    }

    // None denotes CharacterClassEscape, not an absent or empty ClassAtom.
    fn class_atom(&mut self) -> Result<Option<u32>, Failure> {
        let Some(point) = self.peek() else {
            return Err(syntax("unterminated regular expression character class"));
        };
        if point == u32::from(b']') {
            return Err(syntax("expected regular expression class atom"));
        }
        self.pos += 1;
        if point != u32::from(b'\\') {
            return Ok(Some(point));
        }
        let Some(point) = self.peek() else {
            return Err(syntax("unterminated regular expression class escape"));
        };
        self.pos += 1;
        match point {
            0x62 => Ok(Some(8)),                         // b is backspace inside a class.
            0x2d if self.mode.unicode => Ok(Some(0x2d)), // -
            0x64 | 0x44 | 0x73 | 0x53 | 0x77 | 0x57 => Ok(None), // d D s S w W
            0x70 | 0x50 if self.mode.unicode => {
                self.property_escape(point == 0x50)?;
                Ok(None)
            }
            _ => self.character_escape(point).map(Some),
        }
    }
}

fn hex_value(point: u32) -> Option<u32> {
    match point {
        0x30..=0x39 => Some(point - 0x30),
        0x41..=0x46 => Some(point - 0x41 + 10),
        0x61..=0x66 => Some(point - 0x61 + 10),
        _ => None,
    }
}

fn decimal_cmp(left: &[u32], right: &[u32]) -> Ordering {
    let significant = |digits: &[u32]| {
        digits
            .iter()
            .position(|point| *point != 0x30)
            .unwrap_or(digits.len())
    };
    let left = &left[significant(left)..];
    let right = &right[significant(right)..];
    left.len().cmp(&right.len()).then_with(|| left.cmp(right))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn capturing_group_count_rejects_the_first_forbidden_total_before_incrementing() {
        // Pattern early errors (22.2.1.1) require a total strictly below 2^32-1.
        // Exercise the counter directly so the boundary needs no huge source.
        for mode in [
            Mode {
                unicode: false,
                sets: false,
            },
            Mode {
                unicode: true,
                sets: false,
            },
            Mode {
                unicode: true,
                sets: true,
            },
        ] {
            let mut pattern = Pattern::new(&JsString::default(), mode);
            pattern.captures = u32::MAX - 3;
            assert_eq!(pattern.capture(), Ok(()));
            assert_eq!(pattern.captures, u32::MAX - 2);
            for count in [u32::MAX - 2, u32::MAX - 1, u32::MAX] {
                pattern.captures = count;
                assert_eq!(
                    pattern.capture(),
                    Err((
                        DiagnosticKind::Syntax,
                        "too many regular expression capturing groups"
                    ))
                );
                assert_eq!(pattern.captures, count);
            }
        }
    }
}
