//! Literal-only ordinary-mode Pattern compilation and UTF-16 matching (22.2.2).

use crate::{JsString, is_identifier_part, regexp_canonicalize_character};
use std::{ops::Range, sync::Arc};

/// An immutable literal-only matcher for a validated non-Unicode Pattern.
///
/// Compilation accepts literal characters and their escapes, with ordinary
/// capturing and noncapturing groups containing the same subset. Other syntax
/// returns `None`, distinct from a failed match. The input must already have passed Pattern validation
/// without `u` or `v`. Match ranges use UTF-16 code-unit offsets, not byte spans.
/// Compilation and search are linear in Pattern and input length respectively.
#[derive(Clone, Debug)]
pub struct RegExpLiteralMatcher(Arc<Program>);

#[derive(Debug)]
struct Program {
    units: Vec<u16>,
    failure: Vec<usize>,
    ignore_case: bool,
    captures: Vec<Range<usize>>,
}

impl RegExpLiteralMatcher {
    /// Compiles the literal-only subset, returning `None` for other syntax.
    pub fn compile(source: &JsString, ignore_case: bool) -> Option<Self> {
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
            let unit = if unit == u16::from(b'\\') {
                character_escape(source, &mut index)?
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
        let mut matched = 0;
        for (index, &unit) in suffix.iter().enumerate() {
            let unit = canonicalize(unit, program.ignore_case);
            while matched > 0 && unit != program.units[matched] {
                matched = program.failure[matched - 1];
            }
            if unit == program.units[matched] {
                matched += 1;
            }
            if matched == program.units.len() {
                let end = start + index + 1;
                let range = end - matched..end;
                if accept(&range) {
                    return Some(range);
                }
                matched = program.failure[matched - 1];
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

fn is_syntax(unit: u16) -> bool {
    matches!(unit, 0x24 | 0x28..=0x2b | 0x2e | 0x3f | 0x5b | 0x5d | 0x5e | 0x7b..=0x7d)
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
