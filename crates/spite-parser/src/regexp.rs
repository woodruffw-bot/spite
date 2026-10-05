//! Literal flags and the non-class, unnamed portion of the Pattern grammar.

use spite_core::{Diagnostic, DiagnosticKind, JsString, Span, is_identifier_part};
use std::{cmp::Ordering, ops::Range};

type Failure = (DiagnosticKind, &'static str);

fn syntax(message: &'static str) -> Failure {
    (DiagnosticKind::Syntax, message)
}

fn unsupported(message: &'static str) -> Failure {
    (DiagnosticKind::Unsupported, message)
}

pub(super) fn literal_diagnostic(body: &JsString, flags: &JsString, span: Span) -> Diagnostic {
    let failure = match unicode_mode(flags) {
        Err(failure) => failure,
        Ok(unicode) => Pattern::new(body, unicode)
            .validate()
            .err()
            .unwrap_or_else(|| unsupported("regular expression matching is not implemented")),
    };
    Diagnostic::new(failure.0, span, failure.1)
}

fn unicode_mode(flags: &JsString) -> Result<bool, Failure> {
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
    Ok(seen & UNICODE_MODES != 0)
}

struct Pattern {
    points: Vec<u32>,
    pos: usize,
    unicode: bool,
    captures: u32,
    largest_reference: Option<Range<usize>>,
}

impl Pattern {
    fn new(body: &JsString, unicode: bool) -> Self {
        // ParsePattern interprets UTF-16 units separately without u/v. In either
        // Unicode mode, decode pairs while retaining unpaired surrogates.
        let points = if unicode {
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
            unicode,
            captures: 0,
            largest_reference: None,
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
        // Each entry records whether the enclosing group is an Assertion. The
        // grammar is traversed iteratively, without a native recursion quota.
        let mut groups = Vec::new();
        let mut can_quantify = false;
        while let Some(point) = self.peek() {
            self.pos += 1;
            match point {
                0x28 => {
                    // (
                    groups.push(self.group()?);
                    can_quantify = false;
                }
                0x29 => {
                    // )
                    let Some(assertion) = groups.pop() else {
                        return Err(syntax("unmatched regular expression closing parenthesis"));
                    };
                    can_quantify = !assertion;
                }
                0x7c | 0x5e | 0x24 => can_quantify = false, // | ^ $
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
                    return Err(unsupported(
                        "regular expression character class validation is not implemented",
                    ));
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

    fn group(&mut self) -> Result<bool, Failure> {
        if !self.eat(b'?') {
            // The specification rejects CountLeftCapturingParens >= 2^32 - 1.
            // This is a grammar early error, not a host resource quota.
            if self.captures == u32::MAX - 1 {
                return Err(syntax("too many regular expression capturing groups"));
            }
            self.captures += 1;
            return Ok(false);
        }
        if self.eat(b'=') || self.eat(b'!') {
            return Ok(true);
        }
        if self.eat(b'<') {
            if self.eat(b'=') || self.eat(b'!') {
                return Ok(true);
            }
            return Err(unsupported(
                "regular expression named capture validation is not implemented",
            ));
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
        Ok(false)
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
                    decimal_cmp(&self.points[reference.clone()], &self.points[largest.clone()]) == Ordering::Greater
                }) {
                    self.largest_reference = Some(reference);
                }
            }
            0x64 | 0x44 | 0x73 | 0x53 | 0x77 | 0x57 | // d D s S w W
            0x66 | 0x6e | 0x72 | 0x74 | 0x76 => {} // f n r t v
            0x63 => { // c AsciiLetter
                if !self.peek().is_some_and(|point| matches!(point, 0x41..=0x5a | 0x61..=0x7a)) {
                    return Err(syntax("invalid regular expression control escape"));
                }
                self.pos += 1;
            }
            0x30 => {
                if self.peek().is_some_and(|point| (0x30..=0x39).contains(&point)) {
                    return Err(syntax("legacy regular expression octal escapes are not in the core grammar"));
                }
            }
            0x78 => { self.hex_digits(2)?; } // x Hex2Digits
            0x75 => { // u Hex4Digits, or u{CodePoint} in either Unicode mode
                if self.unicode && self.eat(b'{') {
                    let start = self.pos;
                    let mut value = 0u32;
                    while let Some(digit) = self.peek().and_then(hex_value) {
                        value = value.checked_mul(16).and_then(|v| v.checked_add(digit))
                            .filter(|v| *v <= 0x10ffff)
                            .ok_or_else(|| syntax("regular expression Unicode escape is out of range"))?;
                        self.pos += 1;
                    }
                    if self.pos == start || !self.eat(b'}') {
                        return Err(syntax("invalid regular expression Unicode escape"));
                    }
                } else {
                    self.hex_digits(4)?;
                }
            }
            0x70 | 0x50 if self.unicode => return Err(unsupported("regular expression Unicode property validation is not implemented")),
            0x6b => return Err(unsupported("regular expression named backreference validation is not implemented")),
            _ => {
                let valid = if self.unicode {
                    matches!(point, 0x5e | 0x24 | 0x5c | 0x2e | 0x2a | 0x2b | 0x3f | 0x28 | 0x29 | 0x5b | 0x5d | 0x7b | 0x7d | 0x7c | 0x2f)
                } else {
                    // IdentityEscape excludes Unicode ID_Continue, which differs
                    // from IdentifierPartChar exactly by the added '$'.
                    point == 0x24 || !char::from_u32(point).is_some_and(is_identifier_part)
                };
                if !valid {
                    return Err(syntax("invalid regular expression identity escape"));
                }
            }
        }
        Ok(true)
    }

    fn hex_digits(&mut self, count: usize) -> Result<(), Failure> {
        for _ in 0..count {
            if self.peek().and_then(hex_value).is_none() {
                return Err(syntax("invalid regular expression hexadecimal escape"));
            }
            self.pos += 1;
        }
        Ok(())
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
