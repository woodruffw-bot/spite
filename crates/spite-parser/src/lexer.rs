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
    pub escaped: bool,
}

pub(crate) struct Lexer<'a> {
    source: &'a str,
    pos: usize,
}

use spite_core::{
    is_identifier_part as id_continue, is_identifier_start as id_start,
    is_line_terminator as is_line, is_whitespace as is_space,
};

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
                escaped: false,
            });
        };
        let mut escaped = false;
        let kind = if id_start(c) || c == '\\' {
            let (name, had_escape) = self.identifier()?;
            escaped = had_escape;
            match (name.as_str(), escaped) {
                ("null", false) => Kind::Literal(Literal::Null),
                ("true", false) => Kind::Literal(Literal::Boolean(true)),
                ("false", false) => Kind::Literal(Literal::Boolean(false)),
                _ => Kind::Word(name),
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
        } else if c == '`' {
            self.bump();
            return Err(self.error(
                start,
                DiagnosticKind::Unsupported,
                "templates are not implemented",
            ));
        } else {
            // Maximal munch keeps multi-character operators intact.
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
            escaped,
        })
    }

    // https://262.ecma-international.org/17.0/#sec-identifier-names
    fn identifier(&mut self) -> Result<(String, bool), Diagnostic> {
        let mut name = String::new();
        let mut escaped = false;
        while let Some(c) = self.peek() {
            let valid = if name.is_empty() {
                id_start
            } else {
                id_continue
            };
            if c == '\\' {
                let start = self.pos;
                self.bump();
                if self.bump() != Some('u') {
                    return Err(self.syntax(start, "identifier escape must use \\u"));
                }
                let cp = self.unicode_escape(start)?;
                let c = char::from_u32(cp).filter(|c| valid(*c)).ok_or_else(|| {
                    self.syntax(
                        start,
                        if name.is_empty() {
                            "invalid identifier start escape"
                        } else {
                            "invalid identifier part escape"
                        },
                    )
                })?;
                name.push(c);
                escaped = true;
            } else if valid(c) {
                self.bump();
                name.push(c);
            } else {
                break;
            }
        }
        Ok((name, escaped))
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
        if self
            .peek()
            .is_some_and(|c| id_start(c) || c.is_ascii_digit() || c == '\\')
        {
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

    // Called after the escape's `u`. String escapes may denote lone surrogates.
    // Identifier callers separately validate the code point and its position.
    fn unicode_escape(&mut self, start: usize) -> Result<u32, Diagnostic> {
        if self.peek() != Some('{') {
            return self.hex(4, start);
        }
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
                    let value = self.unicode_escape(start)?;
                    if value <= 0xffff {
                        value as u16
                    } else {
                        let value = value - 0x10000;
                        units.push(0xd800 + (value >> 10) as u16);
                        0xdc00 + (value & 0x3ff) as u16
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::{collections::BTreeSet, fs, path::PathBuf};

    fn fixture_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/test262")
    }

    fn fixture_path(name: &str) -> String {
        format!("test/language/identifiers/{name}.js")
    }

    fn fixture(name: &str) -> String {
        fs::read_to_string(fixture_root().join("upstream").join(fixture_path(name))).unwrap()
    }

    fn inventory(mode: &str) -> BTreeSet<String> {
        fs::read_to_string(fixture_root().join("manifest.tsv"))
            .unwrap()
            .lines()
            .filter_map(|line| {
                let fields: Vec<_> = line.split('\t').collect();
                if fields.first() == Some(&mode) {
                    Some(fields[2].to_owned())
                } else {
                    None
                }
            })
            .collect()
    }

    fn tokens(source: &str) -> Result<Vec<Kind>, Diagnostic> {
        let mut lexer = Lexer::new(source);
        let mut tokens = Vec::new();
        loop {
            let token = lexer.next()?;
            let done = token.kind == Kind::Eof;
            tokens.push(token.kind);
            if done {
                return Ok(tokens);
            }
        }
    }

    #[test]
    fn upstream_identifier_spellings_have_identical_tokens() {
        let cases = [
            ("start-unicode-16.0.0", 4302),
            ("part-unicode-16.0.0", 1),
            ("start-unicode-17.0.0", 4647),
            ("part-unicode-17.0.0", 1),
        ];
        let mut checked = BTreeSet::new();
        for (name, declarations) in cases {
            let escaped = format!("{name}-escaped");
            let literal_tokens = tokens(&fixture(name)).unwrap_or_else(|e| panic!("{name}: {e}"));
            let escaped_tokens =
                tokens(&fixture(&escaped)).unwrap_or_else(|e| panic!("{escaped}: {e}"));
            // Each fixture contains only `var IdentifierName ;` declarations.
            // This is a lexical comparison, not a full Test262 execution.
            assert_eq!(literal_tokens.len(), declarations * 3 + 1, "{name}");
            assert_eq!(literal_tokens, escaped_tokens, "{name}");
            checked.insert(fixture_path(name));
            checked.insert(fixture_path(&escaped));
        }
        assert_eq!(checked, inventory("identifier-tokens"));
    }

    #[test]
    fn upstream_invalid_identifier_escapes_reject_the_expected_code_point() {
        let cases = [
            (
                "start-zwj-escaped",
                "invalid identifier start escape",
                r"\u200D",
            ),
            (
                "start-zwnj-escaped",
                "invalid identifier start escape",
                r"\u200C",
            ),
            (
                "vertical-tilde-start-escaped",
                "invalid identifier start escape",
                r"\u2E2F",
            ),
            (
                "vertical-tilde-continue-escaped",
                "invalid identifier part escape",
                r"\u2E2F",
            ),
            (
                "unicode-escape-nls-err",
                "invalid Unicode escape",
                r"\u{00_",
            ),
        ];
        let mut checked = BTreeSet::new();
        for (name, message, fragment) in cases {
            let source = fixture(name);
            assert!(source.contains("\nnegative:\n  phase: parse\n  type: SyntaxError\n"));
            let error = tokens(&source).unwrap_err();
            assert_eq!(error.kind, DiagnosticKind::Syntax, "{name}");
            assert_eq!(error.message, message, "{name}");
            assert_eq!(
                &source[error.span.start..error.span.end],
                fragment,
                "{name}"
            );
            checked.insert(fixture_path(name));
        }
        assert_eq!(checked, inventory("identifier-error"));
    }
}
