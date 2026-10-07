//! Ordinary literal concatenations with DecimalEscape references (22.2.2.9).

use crate::regexp_literal::{character_escape, is_syntax};
use crate::{JsString, regexp_canonicalize_character};
use std::{ops::Range, sync::Arc};

/// A flat, immutable ordinary-mode literal and numbered-backreference program.
///
/// The complete Pattern must already be validated without `u` or `v`. Capturing
/// and noncapturing groups are accepted, including empty, nested and forward
/// references. Alternatives, quantifiers, assertions and character sets return
/// `None`. At least one numbered reference is required; plain literals retain
/// their existing linear-search matcher. References are never expanded into
/// source or compiled literal strings.
#[derive(Clone, Debug)]
pub struct RegExpBackreferenceMatcher(Arc<Program>);

#[derive(Debug)]
struct Program {
    instructions: Vec<Instruction>,
    capture_count: usize,
    ignore_case: bool,
}

#[derive(Debug)]
enum Instruction {
    Character(u16),
    Open(usize),
    Close(usize),
    Reference(usize),
}

/// Absolute UTF-16 endpoints from one successful execution.
#[derive(Debug)]
pub struct RegExpBackreferenceMatch {
    /// Whole-match endpoints.
    pub range: Range<usize>,
    /// Opening-parenthesis order, excluding the whole match.
    pub captures: Box<[Option<Range<usize>>]>,
}

impl RegExpBackreferenceMatcher {
    /// Compiles the complete supported subset without charging host work.
    pub fn compile(source: &JsString, ignore_case: bool) -> Option<Self> {
        Self::compile_with_work(source, ignore_case, |_| {
            Ok::<_, std::convert::Infallible>(())
        })
        .unwrap_or_else(|never| match never {})
    }

    /// Checks the entire syntax before charging optional construction work.
    pub fn compile_with_work<E>(
        source: &JsString,
        ignore_case: bool,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, E> {
        if !source
            .code_units()
            .windows(2)
            .any(|units| units[0] == 0x5c && (0x31..=0x39).contains(&units[1]))
        {
            return Ok(None);
        }
        let Some((instructions, capture_count)) = prepare(source.code_units(), ignore_case) else {
            return Ok(None);
        };
        charge(source.len())?;
        charge(instructions.len())?;
        Ok(Some(Self(Arc::new(Program {
            instructions,
            capture_count,
            ignore_case,
        }))))
    }

    /// Number of source-order capturing groups.
    pub fn capture_count(&self) -> usize {
        self.0.capture_count
    }

    /// Finds the earliest match, or checks exactly `start` when sticky.
    pub fn find(
        &self,
        input: &JsString,
        start: usize,
        sticky: bool,
    ) -> Option<RegExpBackreferenceMatch> {
        self.find_with_work(input, start, sticky, |_| {
            Ok::<_, std::convert::Infallible>(())
        })
        .unwrap_or_else(|never| match never {})
    }

    /// Charges capture resets, executed instructions and actual unit comparisons.
    ///
    /// A single capture buffer is reused across ordered candidate starts. An open
    /// or forward capture has no completed range and its reference matches the
    /// empty string, as required by BackreferenceMatcher (22.2.2.9.2). Ignore-case
    /// comparison canonicalizes both input units with the ordinary ASCII boundary.
    /// No recursion, backtracking stack or expanded reference string is used.
    pub fn find_with_work<E>(
        &self,
        input: &JsString,
        start: usize,
        sticky: bool,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<RegExpBackreferenceMatch>, E> {
        let input = input.code_units();
        if start > input.len() {
            return Ok(None);
        }
        charge(self.0.capture_count)?;
        let mut captures: Vec<Option<Range<usize>>> = vec![None; self.0.capture_count];
        // Starts are needed independently of completed ranges: an open capture
        // must remain undefined to references even after some body has consumed.
        charge(self.0.capture_count)?;
        let mut starts = vec![0; self.0.capture_count];
        let end = if sticky { start } else { input.len() };
        'candidate: for candidate in start..=end {
            charge(captures.len())?;
            captures.fill(None);
            let mut cursor = candidate;
            for instruction in &self.0.instructions {
                charge(1)?;
                match *instruction {
                    Instruction::Character(expected) => {
                        let Some(&unit) = input.get(cursor) else {
                            continue 'candidate;
                        };
                        charge(1)?;
                        if canonicalize(unit, self.0.ignore_case) != expected {
                            continue 'candidate;
                        }
                        cursor += 1;
                    }
                    Instruction::Open(index) => starts[index] = cursor,
                    Instruction::Close(index) => captures[index] = Some(starts[index]..cursor),
                    Instruction::Reference(index) => {
                        let Some(range) = &captures[index] else {
                            continue;
                        };
                        let Some(next) = cursor.checked_add(range.len()) else {
                            continue 'candidate;
                        };
                        let Some(candidate_units) = input.get(cursor..next) else {
                            continue 'candidate;
                        };
                        for (&actual, &captured) in
                            candidate_units.iter().zip(&input[range.clone()])
                        {
                            charge(2)?;
                            if canonicalize(actual, self.0.ignore_case)
                                != canonicalize(captured, self.0.ignore_case)
                            {
                                continue 'candidate;
                            }
                        }
                        cursor = next;
                    }
                }
            }
            return Ok(Some(RegExpBackreferenceMatch {
                range: candidate..cursor,
                captures: captures.into_boxed_slice(),
            }));
        }
        Ok(None)
    }
}

fn canonicalize(unit: u16, ignore_case: bool) -> u16 {
    regexp_canonicalize_character(u32::from(unit), ignore_case, false) as u16
}

fn prepare(source: &[u16], ignore_case: bool) -> Option<(Vec<Instruction>, usize)> {
    let mut instructions = Vec::new();
    let mut groups = Vec::new();
    let mut capture_count = 0usize;
    let mut references = 0usize;
    let mut cursor = 0;
    while let Some(&unit) = source.get(cursor) {
        cursor += 1;
        match unit {
            0x28 => {
                let group = if source.get(cursor) == Some(&0x3f) {
                    if source.get(cursor..cursor + 2)? != [0x3f, 0x3a] {
                        return None;
                    }
                    cursor += 2;
                    None
                } else {
                    let index = capture_count;
                    capture_count = capture_count.checked_add(1)?;
                    instructions.push(Instruction::Open(index));
                    Some(index)
                };
                groups.push(group);
            }
            0x29 => {
                if let Some(index) = groups.pop()? {
                    instructions.push(Instruction::Close(index));
                }
            }
            0x5c if source
                .get(cursor)
                .is_some_and(|u| (0x31..=0x39).contains(u)) =>
            {
                let mut number = 0usize;
                while let Some(&digit @ 0x30..=0x39) = source.get(cursor) {
                    number = number
                        .checked_mul(10)?
                        .checked_add(usize::from(digit - 0x30))?;
                    cursor += 1;
                }
                instructions.push(Instruction::Reference(number.checked_sub(1)?));
                references += 1;
            }
            0x5c => instructions.push(Instruction::Character(canonicalize(
                character_escape(source, &mut cursor)?,
                ignore_case,
            ))),
            _ if is_syntax(unit) || unit == 0x7c => return None,
            _ => instructions.push(Instruction::Character(canonicalize(unit, ignore_case))),
        }
    }
    if !groups.is_empty() || references == 0 {
        return None;
    }
    if instructions.iter().any(|instruction| {
        matches!(instruction, Instruction::Reference(index) if *index >= capture_count)
    }) {
        return None;
    }
    Some((instructions, capture_count))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fmt::Write;

    #[test]
    fn numbered_reference_execution_snapshot() {
        let mut rows = String::new();
        for (source, ignore_case) in [
            (r"(a)\1", false),
            (r"(a)\1", true),
            (r"\1(a)", false),
            (r"(a\1)", false),
            (r"(a(\1b)\2)\1", false),
            (r"()(a)\1\2", false),
            (r"(ab)(\1c)\2", false),
            (r"(?:q(a))\1", false),
            (r"(µ)\1", true),
            (r"(ſ)\1", true),
            (r"(\uD800)\1", false),
            (r"(\uD800\uDC00)\1", false),
            (r"(a)\x31\1", false),
            (r"(a)\\1\1", false),
            (r"()()()()()()()()()(a)\10", false),
        ] {
            let matcher =
                RegExpBackreferenceMatcher::compile(&JsString::from(source), ignore_case).unwrap();
            for text in [
                "",
                "a",
                "aa",
                "aA",
                "Aa",
                "qababcabc",
                "abbabb",
                "qaa",
                "µΜ",
                "ſS",
                "𐀀𐀀",
                "a1a",
                r"a\1a",
            ] {
                let input = JsString::from(text);
                for (start, sticky) in [(0, false), (0, true), (1, true)] {
                    let result = matcher.find(&input, start, sticky);
                    writeln!(rows, "{source:?} i={ignore_case} input={input:?} start={start} sticky={sticky} {result:?}").unwrap();
                }
            }
        }
        insta::assert_snapshot!(rows);
    }

    #[test]
    fn captures_references_and_canonicalization_follow_input_endpoints() {
        let matcher =
            RegExpBackreferenceMatcher::compile(&JsString::from(r"(a(\1b)\2)\1"), false).unwrap();
        let result = matcher.find(&JsString::from("qabbabb"), 0, false).unwrap();
        assert_eq!(result.range, 1..7);
        assert_eq!(&*result.captures, &[Some(1..4), Some(2..3)]);
        let matcher =
            RegExpBackreferenceMatcher::compile(&JsString::from(r"\2(a)(b)\1\2"), true).unwrap();
        let result = matcher.find(&JsString::from("qABab"), 0, false).unwrap();
        assert_eq!(result.range, 1..5);
        assert_eq!(&*result.captures, &[Some(1..2), Some(2..3)]);
        let matcher = RegExpBackreferenceMatcher::compile(&JsString::from(r"(ſ)\1"), true).unwrap();
        assert!(matcher.find(&JsString::from("ſS"), 0, false).is_none());
        assert!(matcher.find(&JsString::from("ſſ"), 0, false).is_some());
    }

    #[test]
    fn flat_programs_keep_deep_groups_and_references_compact_and_work_fallible() {
        let source = format!("{}a{}\\100000", "(".repeat(100000), ")".repeat(100000));
        let matcher =
            RegExpBackreferenceMatcher::compile(&JsString::from(source.as_str()), false).unwrap();
        let cloned = matcher.clone();
        drop(matcher);
        let result = cloned.find(&JsString::from("aa"), 0, true).unwrap();
        assert_eq!(result.range, 0..2);
        assert_eq!(result.captures.len(), 100000);
        assert!(result.captures.iter().all(|range| *range == Some(0..1)));
        let source = JsString::from(format!("(a){}", r"\1".repeat(100000)).as_str());
        let matcher = RegExpBackreferenceMatcher::compile(&source, false).unwrap();
        assert_eq!(
            matcher
                .find(&JsString::from("a".repeat(100001).as_str()), 0, true)
                .unwrap()
                .range,
            0..100001
        );
        let mut work = 0;
        assert_eq!(
            matcher
                .find_with_work(&JsString::from("a".repeat(100001).as_str()), 0, true, |n| {
                    work += n;
                    if work > 1000 {
                        Err("explicit work")
                    } else {
                        Ok(())
                    }
                })
                .unwrap_err(),
            "explicit work"
        );
        // Potential exponential expansion is retained as references and fails on
        // the short input without constructing an expanded string.
        let mut source = String::from("(a)");
        for index in 1..1000 {
            write!(source, "(\\{index}\\{index})").unwrap();
        }
        let source = JsString::from(source.as_str());
        let matcher = RegExpBackreferenceMatcher::compile(&source, false).unwrap();
        assert!(matcher.find(&JsString::from("aa"), 0, false).is_none());
        for source in [
            r"(a)\1+",
            r"(a|b)\1",
            r"(?<x>a)\k<x>",
            r"([ab])\1",
            r"^(a)\1",
            r"(a)\2",
            "(a)",
        ] {
            let mut charged = 0;
            assert!(
                RegExpBackreferenceMatcher::compile_with_work(
                    &JsString::from(source),
                    false,
                    |n| {
                        charged += n;
                        Ok::<_, ()>(())
                    }
                )
                .unwrap()
                .is_none()
            );
            assert_eq!(charged, 0, "{source}");
        }
    }
}
