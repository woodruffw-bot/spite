//! Strict ECMA-404 JSON grammar, with UTF-16 source ranges (25.5.1, ParseJSON).

use spite_core::JsString;
use std::{fmt, ops::Range};

/// A flat, postorder JSON tree. Child indices always precede their parents.
///
/// Parsing and dropping deeply nested input do not recurse on the native stack.
#[derive(Debug, PartialEq)]
pub struct JsonDocument {
    /// Parsed nodes, including lexically preceding duplicate object entries.
    pub nodes: Vec<JsonNode>,
    /// Index of the root value.
    pub root: usize,
}

/// A JSON value and its exact source lexeme, excluding surrounding whitespace.
#[derive(Debug, PartialEq)]
pub struct JsonNode {
    /// Half-open UTF-16 code-unit offsets into the original input.
    pub source: Range<usize>,
    /// The value's syntactic content.
    pub kind: JsonKind,
}

/// JSON values; object members preserve source order and duplicate names.
#[derive(Debug, PartialEq)]
pub enum JsonKind {
    /// JSON null.
    Null,
    /// JSON true or false.
    Boolean(bool),
    /// A correctly rounded binary64 Number, preserving negative zero.
    Number(f64),
    /// Decoded UTF-16 string, preserving lone surrogates.
    String(JsString),
    /// Array elements, as indices into the document's nodes.
    Array(Vec<usize>),
    /// Object names and value indices, including duplicates.
    Object(Vec<(JsString, usize)>),
}

/// A JSON syntax or host-resource failure. Offsets use UTF-16 code units.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct JsonError {
    /// True for an opted-in work failure or checked allocation failure.
    pub limit: bool,
    /// Code-unit offset at which parsing stopped.
    pub offset: usize,
    /// Human-readable reason.
    pub message: &'static str,
}

impl fmt::Display for JsonError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} at UTF-16 offset {}", self.message, self.offset)
    }
}

impl std::error::Error for JsonError {}

/// Parses JSON without a default work or nesting quota.
pub fn parse_json(source: &JsString) -> Result<JsonDocument, JsonError> {
    parse_json_with_work(source, |_| true)
}

/// Parses JSON while charging work through a caller-supplied host budget.
///
/// Returning false from `charge` aborts with a resource failure, separately from
/// JSON syntax errors. Every growing buffer uses checked, fallible reservation.
pub fn parse_json_with_work(
    source: &JsString,
    charge: impl FnMut(usize) -> bool,
) -> Result<JsonDocument, JsonError> {
    Parser {
        units: source.code_units(),
        offset: 0,
        charge,
    }
    .document()
}

struct Parser<'a, F> {
    units: &'a [u16],
    offset: usize,
    charge: F,
}

#[derive(Clone, Copy)]
enum State {
    ArrayFirst,
    ArrayValue,
    ArrayAfter,
    ObjectFirst,
    ObjectKey,
    ObjectValue,
    ObjectAfter,
}

struct Frame {
    start: usize,
    state: State,
    kind: JsonKind,
    key: Option<JsString>,
}

impl<F: FnMut(usize) -> bool> Parser<'_, F> {
    fn document(mut self) -> Result<JsonDocument, JsonError> {
        let mut nodes = Vec::new();
        let mut frames: Vec<Frame> = Vec::new();
        let mut root = None;
        let mut completed = None;
        loop {
            self.work(1)?;
            if let Some(index) = completed.take() {
                if let Some(frame) = frames.last_mut() {
                    match &mut frame.kind {
                        JsonKind::Array(elements) => {
                            self.push(elements, index)?;
                            frame.state = State::ArrayAfter;
                        }
                        JsonKind::Object(entries) => {
                            self.push(
                                entries,
                                (frame.key.take().expect("parsed object key"), index),
                            )?;
                            frame.state = State::ObjectAfter;
                        }
                        _ => unreachable!("container frames"),
                    }
                } else {
                    root = Some(index);
                }
            }
            self.whitespace()?;
            if let Some(root) = root {
                if self.peek().is_some() {
                    return Err(self.syntax("unexpected trailing JSON text"));
                }
                return Ok(JsonDocument { nodes, root });
            }
            if let Some(frame) = frames.last_mut() {
                let state = frame.state;
                let closing = match state {
                    State::ArrayFirst | State::ArrayAfter if self.peek() == Some(0x5d) => true,
                    State::ObjectFirst | State::ObjectAfter if self.peek() == Some(0x7d) => true,
                    _ => false,
                };
                if closing {
                    self.bump()?;
                    let frame = frames.pop().expect("open container");
                    let index = nodes.len();
                    self.push(
                        &mut nodes,
                        JsonNode {
                            source: frame.start..self.offset,
                            kind: frame.kind,
                        },
                    )?;
                    completed = Some(index);
                    continue;
                }
                match state {
                    State::ArrayAfter | State::ObjectAfter => {
                        if self.peek() != Some(0x2c) {
                            return Err(self.syntax("expected a JSON comma or closing delimiter"));
                        }
                        self.bump()?;
                        frame.state = match state {
                            State::ArrayAfter => State::ArrayValue,
                            _ => State::ObjectKey,
                        };
                        continue;
                    }
                    State::ObjectFirst | State::ObjectKey => {
                        frame.key = Some(self.string()?);
                        self.whitespace()?;
                        if self.peek() != Some(0x3a) {
                            return Err(self.syntax("expected a JSON object colon"));
                        }
                        self.bump()?;
                        frame.state = State::ObjectValue;
                        continue;
                    }
                    _ => {}
                }
            }
            let start = self.offset;
            let kind = match self.peek() {
                Some(0x5b | 0x7b) => {
                    let array = self.bump()? == 0x5b;
                    self.push(
                        &mut frames,
                        Frame {
                            start,
                            state: if array {
                                State::ArrayFirst
                            } else {
                                State::ObjectFirst
                            },
                            kind: if array {
                                JsonKind::Array(Vec::new())
                            } else {
                                JsonKind::Object(Vec::new())
                            },
                            key: None,
                        },
                    )?;
                    continue;
                }
                Some(0x22) => JsonKind::String(self.string()?),
                Some(0x6e) => {
                    self.literal(b"null")?;
                    JsonKind::Null
                }
                Some(0x74) => {
                    self.literal(b"true")?;
                    JsonKind::Boolean(true)
                }
                Some(0x66) => {
                    self.literal(b"false")?;
                    JsonKind::Boolean(false)
                }
                Some(0x2d | 0x30..=0x39) => JsonKind::Number(self.number()?),
                _ => return Err(self.syntax("expected a JSON value")),
            };
            let index = nodes.len();
            self.push(
                &mut nodes,
                JsonNode {
                    source: start..self.offset,
                    kind,
                },
            )?;
            completed = Some(index);
        }
    }

    fn syntax(&self, message: &'static str) -> JsonError {
        JsonError {
            limit: false,
            offset: self.offset,
            message,
        }
    }

    fn limit(&self) -> JsonError {
        JsonError {
            limit: true,
            offset: self.offset,
            message: "JSON exceeds host work budget or platform capacity",
        }
    }

    fn work(&mut self, amount: usize) -> Result<(), JsonError> {
        if (self.charge)(amount) {
            Ok(())
        } else {
            Err(self.limit())
        }
    }

    fn push<T>(&mut self, values: &mut Vec<T>, value: T) -> Result<(), JsonError> {
        self.work(1)?;
        values.try_reserve(1).map_err(|_| self.limit())?;
        values.push(value);
        Ok(())
    }

    fn peek(&self) -> Option<u16> {
        self.units.get(self.offset).copied()
    }

    fn bump(&mut self) -> Result<u16, JsonError> {
        let unit = self
            .peek()
            .ok_or_else(|| self.syntax("unexpected end of JSON"))?;
        self.work(1)?;
        self.offset += 1;
        Ok(unit)
    }

    fn whitespace(&mut self) -> Result<(), JsonError> {
        // ECMA-404 permits exactly TAB, LF, CR, and SPACE, excluding ECMAScript's
        // additional whitespace and line terminators, including BOM and NBSP.
        while matches!(self.peek(), Some(0x09 | 0x0a | 0x0d | 0x20)) {
            self.bump()?;
        }
        Ok(())
    }

    fn literal(&mut self, expected: &[u8]) -> Result<(), JsonError> {
        for &unit in expected {
            if self.bump()? != u16::from(unit) {
                return Err(self.syntax("invalid JSON literal"));
            }
        }
        Ok(())
    }

    fn string(&mut self) -> Result<JsString, JsonError> {
        if self.peek() != Some(0x22) {
            return Err(self.syntax("expected a double-quoted JSON string"));
        }
        self.bump()?;
        let mut units = Vec::new();
        loop {
            let unit = self.bump()?;
            let unit = match unit {
                0x22 => return Ok(JsString::from_code_units(units)),
                0x00..=0x1f => return Err(self.syntax("unescaped JSON string control character")),
                0x5c => match self.bump()? {
                    0x22 => 0x22,
                    0x5c => 0x5c,
                    0x2f => 0x2f,
                    0x62 => 0x08,
                    0x66 => 0x0c,
                    0x6e => 0x0a,
                    0x72 => 0x0d,
                    0x74 => 0x09,
                    0x75 => {
                        let mut point = 0u16;
                        for _ in 0..4 {
                            let digit = match self.bump()? {
                                unit @ 0x30..=0x39 => unit - 0x30,
                                unit @ 0x41..=0x46 => unit - 0x41 + 10,
                                unit @ 0x61..=0x66 => unit - 0x61 + 10,
                                _ => return Err(self.syntax("invalid JSON Unicode escape")),
                            };
                            point = point * 16 + digit;
                        }
                        point
                    }
                    _ => return Err(self.syntax("invalid JSON string escape")),
                },
                unit => unit,
            };
            self.push(&mut units, unit)?;
        }
    }

    fn digits(&mut self) -> Result<usize, JsonError> {
        let start = self.offset;
        while matches!(self.peek(), Some(0x30..=0x39)) {
            self.bump()?;
        }
        Ok(self.offset - start)
    }

    fn number(&mut self) -> Result<f64, JsonError> {
        let start = self.offset;
        if self.peek() == Some(0x2d) {
            self.bump()?;
        }
        match self.peek() {
            Some(0x30) => {
                self.bump()?;
            }
            Some(0x31..=0x39) => {
                self.digits()?;
            }
            _ => return Err(self.syntax("expected JSON integer digits")),
        }
        if self.peek() == Some(0x2e) {
            self.bump()?;
            if self.digits()? == 0 {
                return Err(self.syntax("expected JSON fraction digits"));
            }
        }
        if matches!(self.peek(), Some(0x45 | 0x65)) {
            self.bump()?;
            if matches!(self.peek(), Some(0x2b | 0x2d)) {
                self.bump()?;
            }
            if self.digits()? == 0 {
                return Err(self.syntax("expected JSON exponent digits"));
            }
        }
        let length = self.offset - start;
        self.work(length)?;
        let mut text = String::new();
        text.try_reserve(length).map_err(|_| self.limit())?;
        for &unit in &self.units[start..self.offset] {
            text.push(char::from(unit as u8));
        }
        // The validated decimal grammar is ASCII. Rust's decimal conversion is
        // correctly rounded, including overflow, underflow, and negative zero.
        text.parse().map_err(|_| self.syntax("invalid JSON number"))
    }
}
