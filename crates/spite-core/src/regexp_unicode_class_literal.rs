//! One checked UnicodeSets class and one literal character (22.2.2.3).

use crate::{
    JsString, RegExpUnicodeCharacterMatcher, RegExpUnicodeClassMatcher,
    RegExpUnicodeSetCompileError, regexp_literal::unicode_literal_atom,
};
use std::ops::Range;

/// One unquantified checked v-mode class and one literal Unicode character.
///
/// Separate constructors prove class-then-literal and literal-then-class order.
///
/// Requires complete validated v/vi Pattern input. Optional outside ^/$ use
/// original line context. Groups, word assertions, quantifiers, additional
/// characters, and string-set leaves remain separate caller proofs. Complete
/// enclosing captures may be proved by the caller; this plan has no captures.
/// Both terms consume complete original characters without flattening surrogate
/// atoms or allocating input-sized state. Existing compiler domains are unchanged.
#[derive(Clone, Debug)]
pub struct RegExpUnicodeClassLiteralMatcher {
    class: RegExpUnicodeClassMatcher,
    literal: RegExpUnicodeCharacterMatcher,
    order: Order,
}

#[derive(Clone, Copy, Debug)]
enum Order {
    ClassThenLiteral,
    LiteralThenClass,
}

impl RegExpUnicodeClassLiteralMatcher {
    /// Compiles the complete narrow body with checked construction work/storage.
    ///
    /// Source slices are borrowed. The complete class compiler must admit a
    /// nested class or set operator; no empty result may hide a string operand.
    /// Existing interval/frame reservations remain fallible and their final
    /// shared-storage allocation boundary is unchanged. This adapter adds no buffer.
    pub fn compile_v_set_with_work<E>(
        source: &JsString,
        ignore_case: bool,
        multiline: bool,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, RegExpUnicodeSetCompileError<E>> {
        let units = source.code_units();
        let begin = usize::from(units.first() == Some(&0x5e));
        if units.get(begin) != Some(&0x5b) {
            return Ok(None);
        }
        charge(units.len()).map_err(RegExpUnicodeSetCompileError::Work)?;
        let Some(end) = class_end(units, begin) else {
            return Ok(None);
        };
        let tail = &units[end..];
        // The suffix owns only an optional final line gate, never an inner ^.
        // Its complete decoder distinguishes an escaped dollar from that gate.
        if tail.is_empty() || tail.first() == Some(&0x5e) {
            return Ok(None);
        }
        let Some(literal) =
            RegExpUnicodeCharacterMatcher::compile_literal_units_with_flags_and_work(
                tail,
                ignore_case,
                multiline,
                &mut charge,
            )
            .map_err(RegExpUnicodeSetCompileError::Work)?
        else {
            return Ok(None);
        };
        let Some(class) = RegExpUnicodeClassMatcher::compile_v_set_units_with_work(
            &units[..end],
            ignore_case,
            multiline,
            &mut charge,
        )?
        else {
            return Ok(None);
        };
        Ok(Some(Self {
            class,
            literal,
            order: Order::ClassThenLiteral,
        }))
    }

    /// Compiles one literal Unicode atom followed by one checked v-mode class.
    ///
    /// The complete Pattern must already be validated in v/vi mode. An optional
    /// leading ^ belongs to the literal and an optional trailing $ to the class;
    /// internal assertions, groups, repetitions and extra atoms are excluded.
    /// At least one nested class or set operator is required. The complete class
    /// is compiled even if empty, so string leaves never acquire substitute
    /// semantics. Source slices are borrowed; no new buffer or shared allocation
    /// is introduced. The class-first constructor keeps its domain and callbacks.
    pub fn compile_literal_then_v_set_with_work<E>(
        source: &JsString,
        ignore_case: bool,
        multiline: bool,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, RegExpUnicodeSetCompileError<E>> {
        let units = source.code_units();
        // Braced escapes may contain source-sized runs of leading zeroes.
        charge(units.len()).map_err(RegExpUnicodeSetCompileError::Work)?;
        let mut end = usize::from(units.first() == Some(&0x5e));
        let Some(&first) = units.get(end) else {
            return Ok(None);
        };
        end += 1;
        if unicode_literal_atom(units, &mut end, first).is_none() || units.get(end) != Some(&0x5b) {
            return Ok(None);
        }
        let Some(literal) =
            RegExpUnicodeCharacterMatcher::compile_literal_units_with_flags_and_work(
                &units[..end],
                ignore_case,
                multiline,
                &mut charge,
            )
            .map_err(RegExpUnicodeSetCompileError::Work)?
        else {
            return Ok(None);
        };
        let Some(class) = RegExpUnicodeClassMatcher::compile_v_set_units_with_work(
            &units[end..],
            ignore_case,
            multiline,
            &mut charge,
        )?
        else {
            return Ok(None);
        };
        Ok(Some(Self {
            class,
            literal,
            order: Order::LiteralThenClass,
        }))
    }

    /// Bounds complete input scans and both predicates for opt-in search work.
    pub fn search_passes(&self) -> usize {
        self.class
            .search_passes()
            .saturating_add(self.literal.search_passes())
            .saturating_add(2)
    }

    /// Searches a precharged suffix with one additional fallible entry charge.
    ///
    /// Callers precharge search_passes() over the searched suffix. Sticky calls
    /// may bound that suffix to four units, including any initial pair rewind.
    pub fn find_with_work<E>(
        &self,
        input: &JsString,
        start: usize,
        sticky: bool,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Range<usize>>, E> {
        charge(1)?;
        Ok(self.find(input, start, sticky))
    }

    /// Finds the earliest complete pair, or checks only the sticky candidate.
    pub fn find(&self, input: &JsString, start: usize, sticky: bool) -> Option<Range<usize>> {
        let mut cursor = start;
        loop {
            let first = match self.order {
                Order::ClassThenLiteral => self.class.find(input, cursor, sticky),
                Order::LiteralThenClass => self.literal.find(input, cursor, sticky),
            }?;
            let second = match self.order {
                Order::ClassThenLiteral => self.literal.find(input, first.end, true),
                Order::LiteralThenClass => self.class.find(input, first.end, true),
            };
            if let Some(second) = second {
                return Some(first.start..second.end);
            }
            if sticky {
                return None;
            }
            // Either first term consumes exactly one complete character. Its
            // end is the next candidate and a safe boundary for the second term.
            cursor = first.end;
        }
    }
}

// Full v validation precedes this range proof. Escaped brackets are opaque;
// property/Unicode escape payloads cannot contain raw brackets in valid input.
// Reject q immediately rather than interpreting any string-disjunction payload.
fn class_end(units: &[u16], begin: usize) -> Option<usize> {
    let mut at = begin;
    let mut depth = 0usize;
    while let Some(&unit) = units.get(at) {
        match unit {
            0x5c => {
                let kind = *units.get(at.checked_add(1)?)?;
                if kind == 0x71 {
                    return None;
                }
                at = at.checked_add(2)?;
                continue;
            }
            0x5b => depth = depth.checked_add(1)?,
            0x5d => {
                depth = depth.checked_sub(1)?;
                if depth == 0 {
                    return at.checked_add(1);
                }
            }
            _ => {}
        }
        at = at.checked_add(1)?;
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Case<'a> {
        source: &'a str,
        class: u8,
        literal: u32,
        ignore_case: bool,
        multiline: bool,
        start_anchor: bool,
        end_anchor: bool,
    }

    // Independent finite oracle. The selected domain has only these nontrivial
    // simple/common folds: ASCII capitals, Kelvin sign and long s. Decode with
    // the standard library rather than a matcher or its character helper.
    fn fold(point: u32) -> u32 {
        match point {
            0x41..=0x5a => point + 0x20,
            0x17f => 0x73,
            0x212a => 0x6b,
            _ => point,
        }
    }

    fn expected(
        case: &Case<'_>,
        input: &JsString,
        start: usize,
        sticky: bool,
    ) -> Option<Range<usize>> {
        if start > input.len() {
            return None;
        }
        let mut points = Vec::new();
        let mut boundaries = vec![0];
        for decoded in char::decode_utf16(input.code_units().iter().copied()) {
            let (point, width) = match decoded {
                Ok(character) => (u32::from(character), character.len_utf16()),
                Err(error) => (u32::from(error.unpaired_surrogate()), 1),
            };
            points.push(point);
            boundaries.push(boundaries.last().unwrap() + width);
        }
        // An initial low-surrogate index checks its enclosing character, the
        // already documented Node-compatible Unicode search behavior.
        let first = boundaries.partition_point(|&at| at <= start) - 1;
        let line = |unit| matches!(unit, 0xa | 0xd | 0x2028 | 0x2029);
        for at in first..points.len().saturating_sub(1) {
            if sticky && at != first {
                break;
            }
            let point = points[at];
            let class_matches = match case.class {
                0 => {
                    if case.ignore_case {
                        fold(point) == 0x61
                    } else {
                        point == 0x61
                    }
                }
                1 => {
                    if case.ignore_case {
                        fold(point) == 0x6b
                    } else {
                        point == 0x4b
                    }
                }
                2 => false,
                3 => point == 0xd800,
                4 => point == 0x1f600,
                _ => unreachable!(),
            };
            let literal_matches = if case.ignore_case {
                fold(points[at + 1]) == fold(case.literal)
            } else {
                points[at + 1] == case.literal
            };
            let begin = boundaries[at];
            let end = boundaries[at + 2];
            let start_matches = !case.start_anchor
                || begin == 0
                || (case.multiline && line(input.code_units()[begin - 1]));
            let end_matches = !case.end_anchor
                || end == input.len()
                || (case.multiline && line(input.code_units()[end]));
            if class_matches && literal_matches && start_matches && end_matches {
                return Some(begin..end);
            }
        }
        None
    }

    #[test]
    fn checked_v_set_literal_atom_ranges_match_sparse_reference() {
        let cases = [
            Case {
                source: "[a--b]c",
                class: 0,
                literal: 0x63,
                ignore_case: false,
                multiline: false,
                start_anchor: false,
                end_anchor: false,
            },
            Case {
                source: "^[[a]]c$",
                class: 0,
                literal: 0x63,
                ignore_case: false,
                multiline: false,
                start_anchor: true,
                end_anchor: true,
            },
            Case {
                source: "^[[a]]c$",
                class: 0,
                literal: 0x63,
                ignore_case: false,
                multiline: true,
                start_anchor: true,
                end_anchor: true,
            },
            Case {
                source: "[[K]]s",
                class: 1,
                literal: 0x73,
                ignore_case: true,
                multiline: false,
                start_anchor: false,
                end_anchor: false,
            },
            Case {
                source: "[[K]]ſ",
                class: 1,
                literal: 0x17f,
                ignore_case: false,
                multiline: false,
                start_anchor: false,
                end_anchor: false,
            },
            Case {
                source: "[a--a]c",
                class: 2,
                literal: 0x63,
                ignore_case: false,
                multiline: false,
                start_anchor: false,
                end_anchor: false,
            },
            Case {
                source: r"[[\uD800]]\uDC00",
                class: 3,
                literal: 0xdc00,
                ignore_case: false,
                multiline: false,
                start_anchor: false,
                end_anchor: false,
            },
            Case {
                source: r"[[\u{1F600}]]\u{1F600}",
                class: 4,
                literal: 0x1f600,
                ignore_case: false,
                multiline: false,
                start_anchor: false,
                end_anchor: false,
            },
            Case {
                source: r"[a--b]\$",
                class: 0,
                literal: 0x24,
                ignore_case: false,
                multiline: false,
                start_anchor: false,
                end_anchor: false,
            },
            Case {
                source: r"[a--b]\]$",
                class: 0,
                literal: 0x5d,
                ignore_case: false,
                multiline: false,
                start_anchor: false,
                end_anchor: true,
            },
        ];
        let inputs = [
            JsString::from(""),
            JsString::from("ac aac"),
            JsString::from("x\nac\n"),
            JsString::from("KſKS"),
            JsString::from("😀😀"),
            JsString::from("a$a]"),
            JsString::from_code_units(vec![0xd800, 0xdc00, 0x61, 0xd800, 0x21, 0xdc00]),
            JsString::from("AKaKſ"),
        ];
        let mut queries = 0;
        for case in cases {
            let matcher = RegExpUnicodeClassLiteralMatcher::compile_v_set_with_work(
                &JsString::from(case.source),
                case.ignore_case,
                case.multiline,
                |_| Ok::<(), std::convert::Infallible>(()),
            )
            .unwrap()
            .unwrap();
            for input in &inputs {
                for start in 0..=input.len() + 1 {
                    for sticky in [false, true] {
                        assert_eq!(
                            matcher.find(input, start, sticky),
                            expected(&case, input, start, sticky),
                            "source={} input={input:?} start={start} sticky={sticky}",
                            case.source,
                        );
                        queries += 1;
                    }
                }
            }
        }
        assert_eq!(queries, 1000);
    }

    #[test]
    fn checked_v_set_literal_atom_domains_and_work_prefixes() {
        for source in [
            "",
            "ac",
            "[a]c",
            "[a--b]",
            "[a--b]cd",
            "a[a--b]",
            "[a--b][c]",
            r"[a--b].",
            r"[a--b]\d",
            "[a--b]+c",
            "[a--b]?c",
            "[a--b]{0}c",
            "([a--b])c",
            "([a--b]c)",
            r"\b[a--b]c",
            r"[a--b]c\b",
            "[a--b]^c",
            "[a--b]c$$",
            r"[[a--a]&&[\q{ab}]]c",
            r"[[a--a]&&[\p{RGI_Emoji}]]c",
            r"[[\p{RGI_Emoji}]]c",
        ] {
            assert!(
                RegExpUnicodeClassLiteralMatcher::compile_v_set_with_work(
                    &JsString::from(source),
                    false,
                    false,
                    |_| Ok::<(), std::convert::Infallible>(()),
                )
                .unwrap()
                .is_none(),
                "{source}",
            );
        }
        for (source, ignore_case) in [
            (r"[[\p{ASCII}]--[Q]]\u{1F600}$", false),
            (r"^[[K]&&[k]]ſ$", true),
        ] {
            let source = JsString::from(source);
            let mut complete = Vec::new();
            assert!(
                RegExpUnicodeClassLiteralMatcher::compile_v_set_with_work(
                    &source,
                    ignore_case,
                    false,
                    |amount| {
                        complete.push(amount);
                        Ok::<(), usize>(())
                    },
                )
                .unwrap()
                .is_some()
            );
            assert!(!complete.is_empty());
            for fail in 1..=complete.len() {
                let mut seen = Vec::new();
                let result = RegExpUnicodeClassLiteralMatcher::compile_v_set_with_work(
                    &source,
                    ignore_case,
                    false,
                    |amount| {
                        seen.push(amount);
                        if seen.len() == fail {
                            Err(fail)
                        } else {
                            Ok(())
                        }
                    },
                );
                assert!(
                    matches!(result, Err(RegExpUnicodeSetCompileError::Work(at)) if at == fail)
                );
                assert_eq!(seen, complete[..fail]);
            }
        }
        let matcher = RegExpUnicodeClassLiteralMatcher::compile_v_set_with_work(
            &JsString::from("[a--b]c"),
            false,
            false,
            |_| Ok::<(), std::convert::Infallible>(()),
        )
        .unwrap()
        .unwrap();
        let input = JsString::from("aac");
        let mut complete = Vec::new();
        assert_eq!(
            matcher.find_with_work(&input, 0, false, |amount| {
                complete.push(amount);
                Ok::<(), usize>(())
            }),
            Ok(Some(1..3)),
        );
        assert!(!complete.is_empty());
        for fail in 1..=complete.len() {
            let mut seen = Vec::new();
            assert_eq!(
                matcher.find_with_work(&input, 0, false, |amount| {
                    seen.push(amount);
                    if seen.len() == fail {
                        Err(fail)
                    } else {
                        Ok(())
                    }
                }),
                Err(fail),
            );
            assert_eq!(seen, complete[..fail]);
        }
    }
}

#[cfg(test)]
#[path = "regexp_unicode_literal_class_tests.rs"]
mod literal_first_tests;
