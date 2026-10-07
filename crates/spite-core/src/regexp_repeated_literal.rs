//! Repetition of ordinary literal groups (22.2.2.3.1, 22.2.2.5–7).

use crate::{JsString, RegExpLiteralMatcher, regexp_canonicalize_character};
use std::{ops::Range, sync::Arc};

/// Immutable matcher for a consuming literal group and one complete quantifier.
///
/// The Pattern must already be validated without `u` or `v`. The group must
/// decode to at least two UTF-16 units; nested ordinary groups and character
/// escapes are supported. Assertions, alternatives, character sets, surrounding
/// terms and additional quantifiers remain unsupported. Storage depends on the
/// group text, never on the repetition count. Search is linear and allocates
/// nothing, including when a large minimum fails at many candidate starts.
#[derive(Clone, Debug)]
pub struct RegExpRepeatedLiteralMatcher(Arc<Program>);

#[derive(Debug)]
struct Program {
    atom: RegExpLiteralMatcher,
    failure: Vec<usize>,
    period: usize,
    min: Option<usize>,
    max: Option<usize>,
    greedy: bool,
    ignore_case: bool,
}

impl RegExpRepeatedLiteralMatcher {
    /// Compiles a complete quantified literal group, or returns `None`.
    pub fn compile(source: &JsString, ignore_case: bool) -> Option<Self> {
        Self::compile_with_work(source, ignore_case, |_| {
            Ok::<(), std::convert::Infallible>(())
        })
        .unwrap_or_else(|never| match never {})
    }

    /// Charges accepted construction through an optional host work callback.
    ///
    /// Oversized decimal bounds retain their mathematical meaning: an oversized
    /// minimum cannot match and an oversized maximum cannot restrict an input.
    /// The failure table contains only two copies of the decoded group's width.
    pub fn compile_with_work<E>(
        source: &JsString,
        ignore_case: bool,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, E> {
        let units = source.code_units();
        let Some(end) = group_end(units) else {
            return Ok(None);
        };
        let Some((min, max, greedy)) = crate::regexp_quantified::complete_quantifier(&units[end..])
        else {
            return Ok(None);
        };
        let Some(atom) = RegExpLiteralMatcher::compile(
            &JsString::from_code_units(units[..end].to_vec()),
            ignore_case,
        ) else {
            return Ok(None);
        };
        let width = atom.matched_len();
        if width < 2 {
            return Ok(None);
        }
        let Some(table_len) = width
            .checked_mul(2)
            .filter(|&length| length <= isize::MAX as usize / std::mem::size_of::<usize>())
        else {
            return Ok(None);
        };
        charge(units.len())?;
        charge(table_len)?;
        let word = atom.matched_units();
        let mut failure = vec![0; table_len];
        for index in 1..table_len {
            let mut matched = failure[index - 1];
            while matched > 0 && word[index % width] != word[matched % width] {
                matched = failure[matched - 1];
            }
            if word[index % width] == word[matched % width] {
                matched += 1;
            }
            failure[index] = matched;
        }
        let mut period = width - failure[width - 1];
        if width % period != 0 {
            period = width;
        }
        Ok(Some(Self(Arc::new(Program {
            atom,
            failure,
            period,
            min,
            max,
            greedy,
            ignore_case,
        }))))
    }

    /// Finds the earliest start and its greedy or lazy complete repetitions.
    ///
    /// Sticky search considers only `start`. A zero minimum always admits that
    /// start, even if the first consuming group occurs later in the input.
    pub fn find(&self, input: &JsString, start: usize, sticky: bool) -> Option<Range<usize>> {
        let program = &self.0;
        let units = input.code_units();
        let suffix = units.get(start..)?;
        let min = program.min?;
        let width = program.atom.matched_len();
        if min > suffix.len() / width {
            return None;
        }
        let mut count = min;
        let mut range = if min == 0 {
            start..start
        } else if sticky {
            let needed = min * width;
            suffix[..needed]
                .iter()
                .enumerate()
                .all(|(index, &unit)| {
                    canonicalize(unit, program.ignore_case)
                        == program.atom.matched_units()[index % width]
                })
                .then_some(start..start + needed)?
        } else {
            program.find_minimum(units, start, min * width)?
        };
        if !program.greedy || program.max == Some(count) {
            return Some(range);
        }
        for next in program.atom.matches_from(units, range.end)? {
            if next.start < range.end {
                continue;
            }
            if next.start != range.end {
                break;
            }
            range.end = next.end;
            count += 1;
            if program.max == Some(count) {
                break;
            }
        }
        Some(range)
    }

    /// Number of capturing groups inside the repeated atom, in source order.
    pub fn capture_count(&self) -> usize {
        self.0.atom.capture_ranges().len()
    }

    /// Capture range from the last iteration, or `None` for zero iterations.
    ///
    /// RepeatMatcher (22.2.2.3.1) clears the atom's captures before each iteration.
    /// Each group in this literal subset participates in every consuming iteration;
    /// empty inner groups retain their final empty range.
    pub fn capture_range(&self, index: usize, matched: &Range<usize>) -> Option<Range<usize>> {
        let relative = self.0.atom.capture_ranges().get(index)?;
        let last = matched.end.checked_sub(self.0.atom.matched_len())?;
        if last < matched.start {
            return None;
        }
        Some(last.checked_add(relative.start)?..last.checked_add(relative.end)?)
    }

    /// Conservative complete-input passes for optional host work accounting.
    pub fn search_passes(&self) -> usize {
        2
    }
}

impl Program {
    fn failure_at(&self, matched: usize) -> usize {
        if matched <= self.failure.len() {
            self.failure[matched - 1]
        } else {
            // Beyond two whole words, the longest proper border of a repeated
            // word drops its primitive period. Shorter prefixes use the exact
            // table, including words whose finite prefixes have shorter periods.
            matched - self.period
        }
    }

    fn find_minimum(&self, input: &[u16], start: usize, needed: usize) -> Option<Range<usize>> {
        let word = self.atom.matched_units();
        let mut matched = 0;
        for (offset, &unit) in input[start..].iter().enumerate() {
            let unit = canonicalize(unit, self.ignore_case);
            while matched > 0 && unit != word[matched % word.len()] {
                matched = self.failure_at(matched);
            }
            if unit == word[matched % word.len()] {
                matched += 1;
            }
            if matched == needed {
                let end = start + offset + 1;
                return Some(end - needed..end);
            }
        }
        None
    }
}

fn canonicalize(unit: u16, ignore_case: bool) -> u16 {
    regexp_canonicalize_character(u32::from(unit), ignore_case, false) as u16
}

pub(crate) fn group_end(units: &[u16]) -> Option<usize> {
    if units.first() != Some(&0x28) {
        return None;
    }
    let mut depth = 0usize;
    let mut class = false;
    let mut index = 0;
    while let Some(&unit) = units.get(index) {
        index += 1;
        if unit == 0x5c {
            units.get(index)?;
            index += 1;
        } else if unit == 0x5b && !class {
            class = true;
        } else if unit == 0x5d && class {
            class = false;
        } else if !class && unit == 0x28 {
            depth += 1;
        } else if !class && unit == 0x29 {
            depth -= 1;
            if depth == 0 {
                return Some(index);
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fmt::Write;

    #[test]
    fn repeated_literal_groups_snapshot() {
        let mut rows = String::new();
        for source in [
            "(ab)*",
            "(ab)*?",
            "(ab)+",
            "(ab)+?",
            "(ab)?",
            "(ab)??",
            "(ab){0}",
            "(ab){0,2}",
            "(ab){0,2}?",
            "(ab){2,3}",
            "(ab){2,3}?",
            "(ab){4}",
            "(ab){999999999999999999999999999999}",
            "(ab){1,999999999999999999999999999999}",
            "(?:ab)+",
            "((a)(b)())*",
            "((a)(b)())*?",
            "((a)(b)()){2}",
            "((?:ab))?",
            "(a()b)+",
            "(()ab())+",
            "(a(b))+",
            "(abab)+",
            "(aba){2,}",
            "(aa){3,4}",
            "(aab){2,3}",
            "(µb)+",
            "(Kb)+",
            "(ſb)+",
            "(💩)+",
            r"(\uD83D\uDCA9)+",
            r"(\uD800a)+",
            r"(\n\r)+",
            r"(\x61\u0062)+",
            r"(\(\))+",
            r"(\[\])+",
            r"(a\|b)+",
            r"(a\\b)+",
            r"(\0b)+",
            "(a)+",
            "()*",
            "(?:)*",
            "((ab)+)",
            "^(ab)+$",
            "(ab)+c",
            "c(ab)+",
            "([ab]c)+",
            "(a.b)+",
            "(ab|cd)+",
            r"(a\bb)+",
            "(ab$)+",
            "(?<x>ab)+",
            r"(ab)\1",
            "(ab)+(cd)+",
            "(ab)+|x",
            "(?=ab)+",
            "(a{2})+",
            "(a(b+))+",
        ] {
            for ignore_case in [false, true] {
                write!(rows, "{source:?} i={ignore_case}").unwrap();
                if let Some(matcher) = SelfMatcher::compile(&JsString::from(source), ignore_case) {
                    write!(rows, " captures={}", matcher.capture_count()).unwrap();
                    for text in [
                        "",
                        "ab",
                        "xabababz",
                        "aababab",
                        "ababxababab",
                        "ABab",
                        "aaaaaaa",
                        "abaaba",
                        "Μbµb",
                        "Kbkb",
                        "ſbsb",
                        "💩💩",
                        "\n\r\n\r",
                        "()()",
                        "a|ba|b",
                        "\0b\0b",
                    ] {
                        let input = JsString::from(text);
                        for (start, sticky) in [(0, false), (1, true)] {
                            let range = matcher.find(&input, start, sticky);
                            let captures = range.as_ref().map(|range| {
                                (0..matcher.capture_count())
                                    .map(|index| matcher.capture_range(index, range))
                                    .collect::<Vec<_>>()
                            });
                            write!(rows, " {input:?}@{start}/{sticky}:{range:?}:{captures:?}")
                                .unwrap();
                        }
                    }
                } else {
                    rows.push_str(" unsupported");
                }
                rows.push('\n');
            }
        }
        insta::assert_snapshot!(rows);
    }

    type SelfMatcher = RegExpRepeatedLiteralMatcher;

    #[test]
    fn minimum_search_and_last_iteration_captures_agree_with_candidate_oracle() {
        for word in ["ab", "aa", "aba", "abab", "aab"] {
            let word_units = word.encode_utf16().collect::<Vec<_>>();
            let width = word_units.len();
            for (min, max) in [
                (0, None),
                (1, None),
                (2, Some(3)),
                (3, Some(4)),
                (0, Some(0)),
                (0, Some(2)),
            ] {
                for greedy in [false, true] {
                    let bounds =
                        max.map_or_else(|| format!("{{{min},}}"), |max| format!("{{{min},{max}}}"));
                    let source = format!(
                        "(({})({})()){}{}",
                        &word[..1],
                        &word[1..],
                        bounds,
                        if greedy { "" } else { "?" }
                    );
                    for ignore_case in [false, true] {
                        let matcher =
                            SelfMatcher::compile(&JsString::from(source.as_str()), ignore_case)
                                .unwrap();
                        for length in 0..=6 {
                            for mut number in 0..3usize.pow(length) {
                                let mut input = vec![0; length as usize];
                                for unit in &mut input {
                                    *unit = [u16::from(b'a'), u16::from(b'b'), u16::from(b'A')]
                                        [number % 3];
                                    number /= 3;
                                }
                                let text = JsString::from_code_units(input.clone());
                                for start in 0..=input.len() + 1 {
                                    for sticky in [false, true] {
                                        let expected = (start..=input.len())
                                            .take(if sticky { 1 } else { usize::MAX })
                                            .find_map(|candidate| {
                                                let limit = max
                                                    .unwrap_or((input.len() - candidate) / width)
                                                    .min((input.len() - candidate) / width);
                                                let mut count = 0;
                                                while count < limit
                                                    && input[candidate + count * width
                                                        ..candidate + (count + 1) * width]
                                                        .iter()
                                                        .zip(&word_units)
                                                        .all(|(&got, &want)| {
                                                            let got = if ignore_case && got <= 0x7f
                                                            {
                                                                u16::from(
                                                                    (got as u8)
                                                                        .to_ascii_lowercase(),
                                                                )
                                                            } else {
                                                                got
                                                            };
                                                            got == want
                                                        })
                                                {
                                                    count += 1;
                                                }
                                                (count >= min).then_some(
                                                    candidate
                                                        ..candidate
                                                            + if greedy { count } else { min }
                                                                * width,
                                                )
                                            });
                                        let actual = matcher.find(&text, start, sticky);
                                        assert_eq!(
                                            actual, expected,
                                            "{source} i={ignore_case} {input:?} @{start}/{sticky}"
                                        );
                                        if let Some(range) = actual {
                                            let last = (range.start < range.end)
                                                .then(|| range.end - width);
                                            let expected = [
                                                last.map(|last| last..range.end),
                                                last.map(|last| last..last + 1),
                                                last.map(|last| last + 1..range.end),
                                                last.map(|_| range.end..range.end),
                                            ];
                                            for (index, expected) in
                                                expected.into_iter().enumerate()
                                            {
                                                assert_eq!(
                                                    matcher.capture_range(index, &range),
                                                    expected
                                                );
                                            }
                                            assert_eq!(matcher.capture_range(4, &range), None);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn compact_failure_tables_large_bounds_clones_and_optional_work() {
        for length in 2..=7 {
            for mut number in 0..2usize.pow(length) {
                let word = (0..length)
                    .map(|_| {
                        let unit = if number % 2 == 0 { 'a' } else { 'b' };
                        number /= 2;
                        unit
                    })
                    .collect::<String>();
                let matcher =
                    SelfMatcher::compile(&JsString::from(format!("({word})+").as_str()), false)
                        .unwrap();
                let expanded = word.repeat(14).into_bytes();
                for end in 1..=expanded.len() {
                    let expected = (0..end)
                        .rev()
                        .find(|&border| expanded[..border] == expanded[end - border..end])
                        .unwrap();
                    assert_eq!(
                        matcher.0.failure_at(end),
                        expected,
                        "word={word:?} prefix={end}"
                    );
                }
            }
        }
        let source = JsString::from("((a)(b)()){100000}");
        let matcher = SelfMatcher::compile(&source, false).unwrap();
        let copy = matcher.clone();
        drop(matcher);
        let input =
            JsString::from(format!("{}x{}", "ab".repeat(99_999), "ab".repeat(100_000)).as_str());
        let range = copy.find(&input, 0, false).unwrap();
        assert_eq!(range, 199_999..399_999);
        assert_eq!(copy.capture_range(0, &range), Some(399_997..399_999));
        assert_eq!(copy.capture_range(3, &range), Some(399_999..399_999));
        assert_eq!(copy.find(&input, 0, true), None);
        assert_eq!(copy.find(&input, usize::MAX, false), None);
        assert_eq!(copy.search_passes(), 2);
        let deep = format!("{}ab{}+", "(".repeat(100_000), ")".repeat(100_000));
        let deep = SelfMatcher::compile(&JsString::from(deep.as_str()), false).unwrap();
        assert_eq!(deep.capture_count(), 100_000);
        let range = deep.find(&JsString::from("abab"), 0, true).unwrap();
        assert_eq!(range, 0..4);
        assert_eq!(deep.capture_range(0, &range), Some(2..4));
        assert_eq!(deep.capture_range(99_999, &range), Some(2..4));
        let huge = SelfMatcher::compile(
            &JsString::from("(ab){999999999999999999999999999999}"),
            false,
        )
        .unwrap();
        assert_eq!(huge.find(&input, 0, false), None);
        let mut charges = Vec::new();
        let aborted = SelfMatcher::compile_with_work(&source, false, |work| {
            charges.push(work);
            if charges.len() == 2 {
                Err("host")
            } else {
                Ok(())
            }
        });
        assert!(matches!(aborted, Err("host")));
        assert_eq!(charges, [source.len(), 4]);
        assert!(
            SelfMatcher::compile_with_work(&JsString::from("(a|b)+"), false, |_| Err("unexpected"))
                .unwrap()
                .is_none()
        );
    }
}
