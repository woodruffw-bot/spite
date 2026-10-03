use crate::ast::Literal;
use spite_core::{Diagnostic, DiagnosticKind, JsString, Span};

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Kind {
    Literal(Literal),
    Word(String),
    Punct(&'static str),
    Eof,
}

#[derive(Clone, Debug)]
pub(crate) struct Token {
    pub kind: Kind,
    pub span: Span,
    pub newline: bool,
}

pub(crate) struct Lexer<'a> {
    source: &'a str,
    pos: usize,
}

use spite_core::{is_line_terminator as is_line, is_whitespace as is_space};

fn id_start(c: char) -> bool {
    c.is_ascii_alphabetic() || matches!(c, '_' | '$')
}
fn id_continue(c: char) -> bool {
    id_start(c) || c.is_ascii_digit()
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Self {
        Self { source, pos: 0 }
    }
    fn rest(&self) -> &'a str {
        &self.source[self.pos..]
    }
    fn peek(&self) -> Option<char> {
        self.rest().chars().next()
    }
    fn bump(&mut self) -> Option<char> {
        let c = self.peek()?;
        self.pos += c.len_utf8();
        Some(c)
    }
    fn error(&self, start: usize, kind: DiagnosticKind, message: &str) -> Diagnostic {
        Diagnostic::new(kind, Span::new(start, self.pos), message)
    }
    fn syntax(&self, start: usize, message: &str) -> Diagnostic {
        self.error(start, DiagnosticKind::Syntax, message)
    }

    pub fn next(&mut self) -> Result<Token, Diagnostic> {
        let mut newline = false;
        loop {
            while let Some(c) = self.peek() {
                if !is_space(c) && !is_line(c) {
                    break;
                }
                newline |= is_line(c);
                self.bump();
            }
            if self.rest().starts_with("//") || (self.pos == 0 && self.rest().starts_with("#!")) {
                while self.peek().is_some_and(|c| !is_line(c)) {
                    self.bump();
                }
            } else if self.rest().starts_with("/*") {
                let start = self.pos;
                self.pos += 2;
                while !self.rest().starts_with("*/") {
                    let Some(c) = self.bump() else {
                        return Err(self.syntax(start, "unterminated comment"));
                    };
                    newline |= is_line(c);
                }
                self.pos += 2;
            } else {
                break;
            }
        }
        let start = self.pos;
        let Some(c) = self.peek() else {
            return Ok(Token {
                kind: Kind::Eof,
                span: Span::new(start, start),
                newline,
            });
        };
        let kind = if id_start(c) {
            self.bump();
            while self.peek().is_some_and(id_continue) {
                self.bump();
            }
            if self
                .peek()
                .is_some_and(|c| c == '\\' || (!c.is_ascii() && !is_space(c) && !is_line(c)))
            {
                return Err(self.error(
                    start,
                    DiagnosticKind::Unsupported,
                    "Unicode identifiers are not implemented",
                ));
            }
            match &self.source[start..self.pos] {
                "null" => Kind::Literal(Literal::Null),
                "true" => Kind::Literal(Literal::Boolean(true)),
                "false" => Kind::Literal(Literal::Boolean(false)),
                word => Kind::Word(word.to_owned()),
            }
        } else if c.is_ascii_digit()
            || (c == '.'
                && self
                    .rest()
                    .as_bytes()
                    .get(1)
                    .is_some_and(u8::is_ascii_digit))
        {
            Kind::Literal(Literal::Number(self.number()?))
        } else if matches!(c, '\'' | '"') {
            Kind::Literal(Literal::String(self.string()?))
        } else if c == '`' || c == '\\' || !c.is_ascii() {
            self.bump();
            return Err(self.error(
                start,
                DiagnosticKind::Unsupported,
                "templates and Unicode identifiers are not implemented",
            ));
        } else {
            // Maximal munch prevents unsupported compound operators from being split.
            const PUNCT: &[&str] = &[
                ">>>=", "===", "!==", ">>>", "**=", "<<=", ">>=", "&&=", "||=", "??=", "...", "=>",
                "++", "--", "**", "==", "!=", "<=", ">=", "&&", "||", "??", "<<", ">>", "+=", "-=",
                "*=", "/=", "%=", "&=", "|=", "^=", "?.", "(", ")", "{", "}", "[", "]", ";", ",",
                ":", "?", "+", "-", "*", "/", "%", "!", "~", "<", ">", "&", "|", "^", "=", ".",
            ];
            let Some(punct) = PUNCT.iter().find(|p| {
                self.rest().starts_with(**p)
                    && !(**p == "?."
                        && self
                            .rest()
                            .as_bytes()
                            .get(2)
                            .is_some_and(u8::is_ascii_digit))
            }) else {
                self.bump();
                return Err(self.syntax(start, "unexpected character"));
            };
            self.pos += punct.len();
            Kind::Punct(punct)
        };
        Ok(Token {
            kind,
            span: Span::new(start, self.pos),
            newline,
        })
    }

    fn digits(&mut self, radix: u32, required: bool) -> Result<(), Diagnostic> {
        let start = self.pos;
        let mut had_digit = false;
        while let Some(c) = self.peek() {
            if c.is_digit(radix) {
                had_digit = true;
                self.bump();
            } else if c == '_' {
                self.bump();
                if !had_digit || !self.peek().is_some_and(|c| c.is_digit(radix)) {
                    return Err(self.syntax(start, "numeric separator must separate digits"));
                }
                had_digit = false;
            } else {
                break;
            }
        }
        if required && !had_digit {
            return Err(self.syntax(start, "expected a digit"));
        }
        Ok(())
    }

    fn number(&mut self) -> Result<f64, Diagnostic> {
        let start = self.pos;
        let mut radix = 10;
        if self.rest().starts_with('0') {
            radix = match self.rest().as_bytes().get(1) {
                Some(b'x' | b'X') => 16,
                Some(b'o' | b'O') => 8,
                Some(b'b' | b'B') => 2,
                _ => 10,
            };
        }
        if radix != 10 {
            self.pos += 2;
            self.digits(radix, true)?;
        } else {
            if self.peek() == Some('0')
                && self
                    .rest()
                    .as_bytes()
                    .get(1)
                    .is_some_and(|c| c.is_ascii_digit() || *c == b'_')
            {
                self.bump();
                return Err(self.error(
                    start,
                    DiagnosticKind::Unsupported,
                    "legacy leading-zero numeric literals are not supported",
                ));
            }
            self.digits(10, false)?;
            if self.peek() == Some('.') {
                self.bump();
                self.digits(10, false)?;
            }
            if matches!(self.peek(), Some('e' | 'E')) {
                self.bump();
                if matches!(self.peek(), Some('+' | '-')) {
                    self.bump();
                }
                self.digits(10, true)?;
            }
        }
        if self.peek() == Some('n') {
            self.bump();
            return Err(self.error(
                start,
                DiagnosticKind::Unsupported,
                "BigInt literals are not implemented",
            ));
        }
        if self.peek().is_some_and(|c| {
            id_continue(c) || c == '\\' || (!c.is_ascii() && !is_space(c) && !is_line(c))
        }) {
            self.bump();
            return Err(self.syntax(start, "invalid character after numeric literal"));
        }
        let clean = self.source[start..self.pos].replace('_', "");
        if radix == 10 {
            clean
                .parse()
                .map_err(|_| self.syntax(start, "invalid decimal literal"))
        } else {
            spite_core::parse_radix_integer(&clean[2..], radix)
                .ok_or_else(|| self.syntax(start, "invalid integer literal"))
        }
    }

    fn hex(&mut self, count: usize, start: usize) -> Result<u32, Diagnostic> {
        let mut value = 0;
        for _ in 0..count {
            let Some(digit) = self.bump().and_then(|c| c.to_digit(16)) else {
                return Err(self.syntax(start, "invalid hexadecimal escape"));
            };
            value = value * 16 + digit;
        }
        Ok(value)
    }

    fn string(&mut self) -> Result<JsString, Diagnostic> {
        let start = self.pos;
        let quote = self.bump();
        let mut units = Vec::new();
        loop {
            let Some(c) = self.bump() else {
                return Err(self.syntax(start, "unterminated string"));
            };
            if Some(c) == quote {
                return Ok(JsString::from_code_units(units));
            }
            if matches!(c, '\r' | '\n') {
                return Err(self.syntax(start, "line terminator in string"));
            }
            if c != '\\' {
                units.extend_from_slice(c.encode_utf16(&mut [0; 2]));
                continue;
            }
            let Some(escape) = self.bump() else {
                return Err(self.syntax(start, "unterminated escape"));
            };
            let unit = match escape {
                '\n' | '\u{2028}' | '\u{2029}' => continue,
                '\r' => {
                    if self.peek() == Some('\n') {
                        self.bump();
                    }
                    continue;
                }
                'n' => 10,
                'r' => 13,
                't' => 9,
                'b' => 8,
                'f' => 12,
                'v' => 11,
                '0' if !self.peek().is_some_and(|c| c.is_ascii_digit()) => 0,
                '0'..='9' => {
                    return Err(self.error(
                        start,
                        DiagnosticKind::Unsupported,
                        "legacy decimal escapes are not supported",
                    ));
                }
                'x' => self.hex(2, start)? as u16,
                'u' => {
                    if self.peek() != Some('{') {
                        self.hex(4, start)? as u16
                    } else {
                        self.bump();
                        let mut value = 0u32;
                        let mut digits = 0;
                        while self.peek() != Some('}') {
                            let Some(d) = self.bump().and_then(|c| c.to_digit(16)) else {
                                return Err(self.syntax(start, "invalid Unicode escape"));
                            };
                            value = value
                                .checked_mul(16)
                                .and_then(|v| v.checked_add(d))
                                .filter(|v| *v <= 0x10ffff)
                                .ok_or_else(|| self.syntax(start, "Unicode escape out of range"))?;
                            digits += 1;
                        }
                        self.bump();
                        if digits == 0 {
                            return Err(self.syntax(start, "empty Unicode escape"));
                        }
                        if value <= 0xffff {
                            value as u16
                        } else {
                            let value = value - 0x10000;
                            units.push(0xd800 + (value >> 10) as u16);
                            0xdc00 + (value & 0x3ff) as u16
                        }
                    }
                }
                c => {
                    units.extend_from_slice(c.encode_utf16(&mut [0; 2]));
                    continue;
                }
            };
            units.push(unit);
        }
    }
}
