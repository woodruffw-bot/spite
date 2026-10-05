use crate::ast::{Literal, TemplateElement};
use crate::source::SourceText;
use spite_core::{Diagnostic, DiagnosticKind, JsString, Span};

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Kind {
    Literal(Literal),
    Word(String),
    Punct(&'static str),
    Template {
        element: TemplateElement,
        tail: bool,
        continuation: bool,
    },
    Eof,
}

#[derive(Clone, Debug)]
pub(crate) struct Token {
    pub kind: Kind,
    pub span: Span,
    pub newline: bool,
    pub escaped: bool,
    // ECMA-262 12.9.3.1 and 12.9.4.1 prohibit these forms in strict code.
    pub legacy: bool,
}

pub(crate) struct Lexer<'a> {
    source: &'a SourceText,
    pos: usize,
    template_braces: Vec<usize>,
}

use spite_core::{
    is_identifier_part as id_continue, is_identifier_start as id_start,
    is_line_terminator as is_line, is_whitespace as is_space,
};

impl<'a> Lexer<'a> {
    pub fn new(source: &'a SourceText) -> Self {
        Self {
            source,
            pos: 0,
            template_braces: Vec::new(),
        }
    }
    fn rest(&self) -> &'a str {
        &self.source.lexical_text()[self.pos..]
    }
    fn peek(&self) -> Option<char> {
        self.rest().chars().next()
    }
    fn bump(&mut self) -> Option<char> {
        let c = self.peek()?;
        self.pos += c.len_utf8();
        Some(c)
    }
    fn append_original(&self, offset: usize, scalar: char, units: &mut Vec<u16>) {
        let point = self.source.original_code_point(offset, scalar);
        if point <= 0xffff {
            units.push(point as u16);
        } else {
            units.extend_from_slice(scalar.encode_utf16(&mut [0; 2]));
        }
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
                legacy: false,
            });
        };
        let mut escaped = false;
        let mut legacy = false;
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
            let (literal, is_legacy) = self.number()?;
            legacy = is_legacy;
            Kind::Literal(literal)
        } else if matches!(c, '\'' | '"') {
            let (string, is_legacy) = self.string()?;
            legacy = is_legacy;
            Kind::Literal(Literal::String(string))
        } else if c == '`' || (c == '}' && self.template_braces.last() == Some(&0)) {
            let continuation = c == '}';
            self.bump();
            if continuation {
                self.template_braces.pop();
            }
            let (element, tail) = self.template_component(start)?;
            if !tail {
                if self.template_braces.len() >= crate::MAX_DEPTH {
                    return Err(self.error(
                        start,
                        DiagnosticKind::Limit,
                        "template nesting limit exceeded",
                    ));
                }
                self.template_braces.push(0);
            }
            Kind::Template {
                element,
                tail,
                continuation,
            }
        } else {
            // Maximal munch keeps multi-character operators intact.
            const PUNCT: &[&str] = &[
                ">>>=", "===", "!==", ">>>", "**=", "<<=", ">>=", "&&=", "||=", "??=", "...", "=>",
                "++", "--", "**", "==", "!=", "<=", ">=", "&&", "||", "??", "<<", ">>", "+=", "-=",
                "*=", "/=", "%=", "&=", "|=", "^=", "?.", "(", ")", "{", "}", "[", "]", ";", ",",
                ":", "?", "+", "-", "*", "/", "%", "!", "~", "<", ">", "&", "|", "^", "=", ".",
                "#",
            ];
            let Some(punct) = PUNCT.iter().find(|p| {
                self.rest().starts_with(**p)
                    // PrivateIdentifier's hash must immediately precede an
                    // IdentifierName; malformed hashes stay lexical errors.
                    && !(**p == "#"
                        && !self
                            .rest()
                            .chars()
                            .nth(1)
                            .is_some_and(|c| id_start(c) || c == '\\'))
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
            if let Some(depth) = self.template_braces.last_mut() {
                if *punct == "{" {
                    *depth += 1;
                } else if *punct == "}" {
                    *depth -= 1;
                }
            }
            Kind::Punct(punct)
        };
        Ok(Token {
            kind,
            span: Span::new(start, self.pos),
            newline,
            escaped,
            legacy,
        })
    }

    // ECMA-262 12.9.6: TV interprets escapes while TRV preserves them. Both
    // normalize CR and CRLF to LF. Invalid escapes have undefined TV so that
    // the parser can reject invalid untagged escapes and preserve tagged ones.
    fn template_component(&mut self, start: usize) -> Result<(TemplateElement, bool), Diagnostic> {
        let content_start = self.pos;
        let mut cooked = Vec::new();
        let mut valid = true;
        loop {
            if self.peek() == Some('`') || self.rest().starts_with("${") {
                let content_end = self.pos;
                let tail = self.peek() == Some('`');
                self.pos += if tail { 1 } else { 2 };
                let original = self.source.to_js_string(content_start..content_end);
                let mut raw = Vec::new();
                let mut units = original.code_units().iter().copied().peekable();
                while let Some(unit) = units.next() {
                    if unit == 13 {
                        if units.peek() == Some(&10) {
                            units.next();
                        }
                        raw.push(10);
                    } else {
                        raw.push(unit);
                    }
                }
                return Ok((
                    TemplateElement {
                        cooked: valid.then(|| JsString::from_code_units(cooked)),
                        raw: JsString::from_code_units(raw),
                        span: Span::new(content_start, content_end),
                    },
                    tail,
                ));
            }
            let offset = self.pos;
            let Some(c) = self.bump() else {
                return Err(self.syntax(start, "unterminated template"));
            };
            if c == '\r' {
                if self.peek() == Some('\n') {
                    self.bump();
                }
                cooked.push(10);
                continue;
            }
            if c != '\\' {
                self.append_original(offset, c, &mut cooked);
                continue;
            }
            let escape_offset = self.pos;
            let Some(escape) = self.bump() else {
                return Err(self.syntax(start, "unterminated template escape"));
            };
            let cp = match escape {
                '\n' | '\u{2028}' | '\u{2029}' => continue,
                '\r' => {
                    if self.peek() == Some('\n') {
                        self.bump();
                    }
                    continue;
                }
                'n' => Some(10),
                'r' => Some(13),
                't' => Some(9),
                'b' => Some(8),
                'f' => Some(12),
                'v' => Some(11),
                '0' if !self.peek().is_some_and(|c| c.is_ascii_digit()) => Some(0),
                '0'..='9' => None,
                'x' => self.template_hex(2),
                'u' => self.template_unicode(),
                c => Some(self.source.original_code_point(escape_offset, c)),
            };
            if let Some(cp) = cp {
                if cp <= 0xffff {
                    cooked.push(cp as u16);
                } else {
                    cooked.extend_from_slice(
                        char::from_u32(cp)
                            .expect("validated template code point")
                            .encode_utf16(&mut [0; 2]),
                    );
                }
            } else {
                valid = false;
            }
        }
    }

    fn template_hex(&mut self, count: usize) -> Option<u32> {
        let mut value = 0;
        for _ in 0..count {
            // Leave non-digits for template scanning, especially ` and ${.
            let digit = self.peek()?.to_digit(16)?;
            self.bump();
            value = value * 16 + digit;
        }
        Some(value)
    }

    fn template_unicode(&mut self) -> Option<u32> {
        if self.peek() != Some('{') {
            return self.template_hex(4);
        }
        self.bump();
        let mut value = Some(0u32);
        let mut digits = 0;
        while let Some(digit) = self.peek().and_then(|c| c.to_digit(16)) {
            self.bump();
            digits += 1;
            value = value
                .and_then(|n| n.checked_mul(16))
                .and_then(|n| n.checked_add(digit))
                .filter(|n| *n <= 0x10ffff);
        }
        if self.peek() != Some('}') {
            return None;
        }
        self.bump();
        value.filter(|_| digits != 0)
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

    fn number(&mut self) -> Result<(Literal, bool), Diagnostic> {
        let start = self.pos;
        let mut digits_start = start;
        let mut radix = 10;
        let mut integer = true;
        let mut leading_zero = false;
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
            digits_start = self.pos;
            self.digits(radix, true)?;
        } else {
            if self.rest().starts_with("0_") {
                self.pos += 2;
                return Err(self.syntax(start, "separator cannot follow a leading zero"));
            }
            leading_zero = self.peek() == Some('0')
                && self
                    .rest()
                    .as_bytes()
                    .get(1)
                    .is_some_and(u8::is_ascii_digit);
            if leading_zero {
                while self.peek().is_some_and(|c| c.is_ascii_digit()) {
                    self.bump();
                }
                // Legacy octal is an integer-only production. Leading-zero
                // sequences containing 8 or 9 are decimal and can have a fraction
                // or exponent, but neither form allows separators in its integer part.
                if self.source.lexical_text()[start..self.pos]
                    .bytes()
                    .all(|b| b <= b'7')
                {
                    radix = 8;
                }
            } else {
                self.digits(10, false)?;
            }
            if radix == 10 && self.peek() == Some('.') {
                integer = false;
                self.bump();
                self.digits(10, false)?;
            }
            if radix == 10 && matches!(self.peek(), Some('e' | 'E')) {
                integer = false;
                self.bump();
                if matches!(self.peek(), Some('+' | '-')) {
                    self.bump();
                }
                self.digits(10, true)?;
            }
        }
        let end = self.pos;
        let bigint = self.peek() == Some('n');
        if bigint {
            self.bump();
            if !integer || leading_zero {
                return Err(self.syntax(start, "invalid BigInt literal"));
            }
        }
        if self
            .peek()
            .is_some_and(|c| id_start(c) || c.is_ascii_digit() || c == '\\')
        {
            self.bump();
            return Err(self.syntax(start, "invalid character after numeric literal"));
        }
        let clean = self.source.lexical_text()[digits_start..end].replace('_', "");
        if bigint {
            return Ok((
                Literal::BigInt {
                    digits: clean,
                    radix,
                },
                false,
            ));
        }
        if radix == 10 {
            clean
                .parse()
                .map(|value| (Literal::Number(value), leading_zero))
                .map_err(|_| self.syntax(start, "invalid decimal literal"))
        } else {
            spite_core::parse_radix_integer(&clean, radix)
                .map(|value| (Literal::Number(value), leading_zero))
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

    fn string(&mut self) -> Result<(JsString, bool), Diagnostic> {
        let start = self.pos;
        let quote = self.bump();
        let mut units = Vec::new();
        let mut legacy = false;
        loop {
            let offset = self.pos;
            let Some(c) = self.bump() else {
                return Err(self.syntax(start, "unterminated string"));
            };
            if Some(c) == quote {
                return Ok((JsString::from_code_units(units), legacy));
            }
            if matches!(c, '\r' | '\n') {
                return Err(self.syntax(start, "line terminator in string"));
            }
            if c != '\\' {
                self.append_original(offset, c, &mut units);
                continue;
            }
            let escape_offset = self.pos;
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
                '0'..='7' => {
                    legacy = true;
                    let mut value = escape as u16 - b'0' as u16;
                    let extra = if escape <= '3' { 2 } else { 1 };
                    for _ in 0..extra {
                        let Some(digit) = self.peek().and_then(|c| c.to_digit(8)) else {
                            break;
                        };
                        self.bump();
                        value = value * 8 + digit as u16;
                    }
                    value
                }
                '8' | '9' => {
                    legacy = true;
                    escape as u16
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
                    self.append_original(escape_offset, c, &mut units);
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
        let source = SourceText::from_str(source);
        let mut lexer = Lexer::new(&source);
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
