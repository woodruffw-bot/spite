//! Ordinary literal sequences with outer input/line anchors (22.2.2.4).

use crate::{JsString, RegExpLiteralMatcher};
use std::ops::Range;

/// A fixed literal sequence with leading `^`, trailing `$`, or both.
///
/// Patterns must already be validated without `u` or `v`. Assertions inside
/// groups or alternatives remain unsupported. Capture ranges retain the body's
/// relative UTF-16 offsets. Compilation and search are linear and iterative.
#[derive(Clone, Debug)]
pub struct RegExpAnchoredMatcher {
    body: RegExpLiteralMatcher,
    at_start: bool,
    at_end: bool,
    multiline: bool,
}

impl RegExpAnchoredMatcher {
    /// Compiles an anchored ordinary literal sequence, preserving escapes.
    pub fn compile(source: &JsString, ignore_case: bool, multiline: bool) -> Option<Self> {
        let units = source.code_units();
        let at_start = units.first() == Some(&u16::from(b'^'));
        let start = usize::from(at_start);
        let mut end = units.len();
        let at_end = if units.last() == Some(&u16::from(b'$')) {
            let mut cursor = end - 1;
            while cursor > start && units[cursor - 1] == u16::from(b'\\') {
                cursor -= 1;
            }
            (end - 1 - cursor) % 2 == 0
        } else {
            false
        };
        if at_end {
            end -= 1;
        }
        if !at_start && !at_end {
            return None;
        }
        let body = JsString::from_code_units(units.get(start..end)?.to_vec());
        Some(Self {
            body: RegExpLiteralMatcher::compile(&body, ignore_case)?,
            at_start,
            at_end,
            multiline,
        })
    }

    /// Relative ranges of the literal body's ordered captures.
    pub fn capture_ranges(&self) -> &[Range<usize>] {
        self.body.capture_ranges()
    }

    /// Finds the earliest literal match whose anchors both succeed.
    ///
    /// Sticky matching never reinterprets `^` as the requested start. Only the
    /// complete input's beginning, or a preceding LineTerminator in multiline
    /// mode, satisfies it. `$` without multiline requires the input's exact end.
    pub fn find(&self, input: &JsString, start: usize, sticky: bool) -> Option<Range<usize>> {
        let units = input.code_units();
        self.body.find_if(input, start, sticky, |range| {
            let begins = !self.at_start
                || range.start == 0
                || (self.multiline && is_line_terminator(units[range.start - 1]));
            let ends = !self.at_end
                || range.end == units.len()
                || (self.multiline && is_line_terminator(units[range.end]));
            begins && ends
        })
    }
}

fn is_line_terminator(unit: u16) -> bool {
    matches!(unit, 0x0a | 0x0d | 0x2028 | 0x2029)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fmt::Write;

    #[test]
    fn literal_input_and_line_anchor_snapshot() {
        let mut rows = String::new();
        for source in [
            "^", "$", "^$", "^a", "a$", "^a$", "^(a)()$", r"^a\$", r"a\\$", r"\^a$", "^(?:a)$",
            "a", "^a|b$", "(^a)", "a$b", "^a*", "^^a",
        ] {
            for multiline in [false, true] {
                let source = JsString::from(source);
                write!(rows, "{source:?} m={multiline}").unwrap();
                if let Some(matcher) = RegExpAnchoredMatcher::compile(&source, false, multiline) {
                    write!(rows, " captures={:?}", matcher.capture_ranges()).unwrap();
                    for input in [
                        "",
                        "a",
                        "ba",
                        "a\n",
                        "x\na\ry",
                        "\r\n",
                        "x\u{2028}a\u{2029}y",
                        "a$",
                        "a\\",
                        "^a",
                    ] {
                        let input = JsString::from(input);
                        write!(
                            rows,
                            " {input:?}:{:?}/{:?}",
                            matcher.find(&input, 0, false),
                            matcher.find(&input, 1, true)
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
    fn rejected_overlapping_candidates_continue_the_linear_scan() {
        let matcher =
            RegExpAnchoredMatcher::compile(&JsString::from("aba$"), false, false).unwrap();
        assert_eq!(matcher.find(&JsString::from("ababa"), 0, false), Some(2..5));
        let source = format!("{}$", "a".repeat(60_000));
        let matcher =
            RegExpAnchoredMatcher::compile(&JsString::from(source.as_str()), false, false).unwrap();
        assert_eq!(
            matcher.find(&JsString::from("a".repeat(120_000).as_str()), 0, false),
            Some(60_000..120_000)
        );
        let matcher = RegExpAnchoredMatcher::compile(&JsString::from("^()$"), false, true).unwrap();
        assert_eq!(
            matcher.find(&JsString::from("x\r\ny"), 0, false),
            Some(2..2)
        );
    }

    #[test]
    fn anchors_agree_with_an_independent_position_and_boundary_oracle() {
        let mut inputs = vec![Vec::new()];
        let mut previous = vec![Vec::new()];
        for _ in 0..6 {
            let mut next = Vec::new();
            for word in previous {
                for unit in [u16::from(b'a'), 0x0a, 0x0d] {
                    let mut word = word.clone();
                    word.push(unit);
                    inputs.push(word.clone());
                    next.push(word);
                }
            }
            previous = next;
        }
        for body in ["", "a", "aa", "aaa"] {
            for (begin, end) in [(true, false), (false, true), (true, true)] {
                let source = format!(
                    "{}{body}{}",
                    if begin { "^" } else { "" },
                    if end { "$" } else { "" }
                );
                for multiline in [false, true] {
                    let matcher = RegExpAnchoredMatcher::compile(
                        &JsString::from(source.as_str()),
                        false,
                        multiline,
                    )
                    .unwrap();
                    for units in &inputs {
                        let input = JsString::from_code_units(units.clone());
                        for start in 0..=units.len() + 1 {
                            for sticky in [false, true] {
                                let expected = (start..=units.len()).find_map(|offset| {
                                    if sticky && offset != start {
                                        return None;
                                    }
                                    let end_offset = offset.checked_add(body.len())?;
                                    let candidate = units.get(offset..end_offset)?;
                                    if !candidate.iter().copied().eq(body.bytes().map(u16::from)) {
                                        return None;
                                    }
                                    let begin_ok = !begin
                                        || offset == 0
                                        || (multiline && [0x0a, 0x0d].contains(&units[offset - 1]));
                                    let end_ok = !end
                                        || end_offset == units.len()
                                        || (multiline && [0x0a, 0x0d].contains(&units[end_offset]));
                                    (begin_ok && end_ok).then_some(offset..end_offset)
                                });
                                assert_eq!(
                                    matcher.find(&input, start, sticky),
                                    expected,
                                    "{source:?} {input:?} start={start} m={multiline} y={sticky}"
                                );
                            }
                        }
                    }
                }
            }
        }
    }
}
