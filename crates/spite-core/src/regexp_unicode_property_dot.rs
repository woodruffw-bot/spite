//! Captured property/dot concatenation over complete characters (22.2.2.3).
use crate::{JsString, RegExpUnicodeCharacterMatcher, RegExpUnicodeClassMatcher};
use std::ops::Range;

/// One captured property atom followed by one captured dot atom.
///
/// The complete Pattern must be validated in u/v mode and names normalized to
/// ordinary captures. Only `(\p{property})(.)` and `(\P{property})(.)` are proved;
/// property admission uses the existing exact-property matcher. Assertions,
/// nested groups, alternatives and quantifiers remain separate proofs.
/// Each term consumes a complete character; capture boundaries retain original
/// UTF-16 widths. Search allocates no input-sized state and uses no recursion.
#[derive(Clone, Debug)]
pub struct RegExpUnicodePropertyDotMatcher {
    property: RegExpUnicodeClassMatcher,
    dot: RegExpUnicodeCharacterMatcher,
}

/// A successful pair and its two distinct capture ranges.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RegExpUnicodePropertyDotMatch {
    /// Original whole-match UTF-16 range.
    pub range: Range<usize>,
    /// Ordered property and dot ranges in the same original input.
    pub captures: [Range<usize>; 2],
}

impl RegExpUnicodePropertyDotMatcher {
    /// Compiles the complete narrow body with fallible opt-in work accounting.
    pub fn compile_with_work<E>(
        source: &JsString,
        unicode_sets: bool,
        ignore_case: bool,
        dot_all: bool,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, E> {
        let units = source.code_units();
        charge(units.len())?;
        if !units.starts_with(&[0x28, 0x5c])
            || !matches!(units.get(2), Some(0x70 | 0x50))
            || units.get(3) != Some(&0x7b)
            || !units.ends_with(&[0x7d, 0x29, 0x28, 0x2e, 0x29])
        {
            return Ok(None);
        }
        // The delegated property proof consumes the complete substring, so no
        // internal terminator, continuation or additional group can be skipped.
        let Some(end) = units.len().checked_sub(4) else {
            return Ok(None);
        };
        charge(end)?;
        let property_source = JsString::from_code_units(units[1..end].to_vec());
        let Some(property) =
            RegExpUnicodeClassMatcher::compile_binary_property_with_flags_and_work(
                &property_source,
                unicode_sets,
                ignore_case,
                false,
                &mut charge,
            )?
        else {
            return Ok(None);
        };
        charge(1)?;
        let dot_source = JsString::from(".");
        let dot = RegExpUnicodeCharacterMatcher::compile_with_work(&dot_source, dot_all, charge)?
            .expect("one validated dot atom");
        Ok(Some(Self { property, dot }))
    }

    /// Conservative complete input passes for optional search accounting.
    pub fn search_passes(&self) -> usize {
        self.property
            .search_passes()
            .saturating_add(self.dot.search_passes())
            .saturating_add(2)
    }

    /// Searches a precharged input with an additional fallible candidate charge.
    ///
    /// Callers precharge `search_passes()` over the complete searched suffix;
    /// sticky callers may bound that suffix to four units plus any pair rewind.
    pub fn find_with_work<E>(
        &self,
        input: &JsString,
        start: usize,
        sticky: bool,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<RegExpUnicodePropertyDotMatch>, E> {
        charge(1)?;
        Ok(self.find(input, start, sticky))
    }

    /// Searches candidates in source order, retaining distinct original widths.
    pub fn find(
        &self,
        input: &JsString,
        start: usize,
        sticky: bool,
    ) -> Option<RegExpUnicodePropertyDotMatch> {
        let mut cursor = start;
        loop {
            let first = self.property.find(input, cursor, sticky)?;
            if let Some(second) = self.dot.find(input, first.end, true) {
                return Some(RegExpUnicodePropertyDotMatch {
                    range: first.start..second.end,
                    captures: [first, second],
                });
            }
            if sticky {
                return None;
            }
            // The property always consumes one character. Retrying from its end
            // considers the next complete boundary without rescanning a prefix.
            cursor = first.end;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fmt::Write;

    fn compile(
        source: &str,
        sets: bool,
        ignore_case: bool,
        dot_all: bool,
    ) -> RegExpUnicodePropertyDotMatcher {
        RegExpUnicodePropertyDotMatcher::compile_with_work(
            &JsString::from(source),
            sets,
            ignore_case,
            dot_all,
            |_| Ok::<(), ()>(()),
        )
        .unwrap()
        .unwrap()
    }

    #[test]
    fn property_dot_separate_capture_ranges_snapshot() {
        let mut rows = String::new();
        for source in [
            r"(\p{ASCII})(.)",
            r"(\P{ASCII})(.)",
            r"(\p{Join_C})(.)",
            r"(\p{sc=Han})(.)",
        ] {
            for sets in [false, true] {
                for ignore_case in [false, true] {
                    for dot_all in [false, true] {
                        let matcher = compile(source, sets, ignore_case, dot_all);
                        for input in [
                            JsString::from("𠮷a𠮷b𠮷"),
                            JsString::from("a\n𠮷😀\r\n\u{200c}X"),
                            JsString::from_code_units(vec![0xd800, 0x61, 0xdc00, 0xa, 0x62]),
                            JsString::from(""),
                        ] {
                            for start in 0..=input.len() + 1 {
                                for sticky in [false, true] {
                                    writeln!(rows,"{source:?} v={sets} i={ignore_case} s={dot_all} input={input:?} start={start} sticky={sticky} {:?}",matcher.find(&input,start,sticky)).unwrap();
                                }
                            }
                        }
                    }
                }
            }
        }
        insta::assert_snapshot!(rows);
    }

    #[test]
    fn property_dot_ranges_agree_with_independent_utf16_candidates() {
        let alphabet = [0x61, 0xa, 0xd, 0x200c, 0xd83d, 0xde00, 0xd800, 0xdc00];
        let mut plans = Vec::new();
        let mut comparisons = 0;
        for sets in [false, true] {
            for dot_all in [false, true] {
                for (source, kind) in [
                    (r"(\p{ASCII})(.)", 0),
                    (r"(\P{ASCII})(.)", 1),
                    (r"(\p{Join_C})(.)", 2),
                ] {
                    plans.push((
                        source,
                        kind,
                        sets,
                        dot_all,
                        compile(source, sets, false, dot_all),
                    ));
                }
            }
        }
        for length in 0..=4 {
            for mut ordinal in 0..alphabet.len().pow(length) {
                let units: Vec<_> = (0..length)
                    .map(|_| {
                        let unit = alphabet[ordinal % alphabet.len()];
                        ordinal /= alphabet.len();
                        unit
                    })
                    .collect();
                let mut at = 0;
                let decoded: Vec<_> = char::decode_utf16(units.iter().copied())
                    .map(|point| {
                        let (point, width) = match point {
                            Ok(c) => (u32::from(c), c.len_utf16()),
                            Err(e) => (u32::from(e.unpaired_surrogate()), 1),
                        };
                        let range = at..at + width;
                        at += width;
                        (point, range)
                    })
                    .collect();
                let input = JsString::from_code_units(units);
                for (source, kind, sets, dot_all, plan) in &plans {
                    for start in 0..=input.len() + 1 {
                        for sticky in [false, true] {
                            let first = decoded.iter().position(|(_, r)| r.end > start);
                            let expected = first.and_then(|first| {
                                decoded
                                    .windows(2)
                                    .enumerate()
                                    .skip(first)
                                    .take(if sticky { 1 } else { decoded.len() })
                                    .find(|(_, pair)| {
                                        let member = match kind {
                                            0 => pair[0].0 <= 0x7f,
                                            1 => pair[0].0 > 0x7f,
                                            _ => matches!(pair[0].0, 0x200c | 0x200d),
                                        };
                                        member
                                            && (*dot_all
                                                || !matches!(
                                                    pair[1].0,
                                                    0xa | 0xd | 0x2028 | 0x2029
                                                ))
                                    })
                                    .map(|(_, pair)| RegExpUnicodePropertyDotMatch {
                                        range: pair[0].1.start..pair[1].1.end,
                                        captures: [pair[0].1.clone(), pair[1].1.clone()],
                                    })
                            });
                            assert_eq!(
                                plan.find(&input, start, sticky),
                                expected,
                                "{source} v={sets} s={dot_all} input={input:?} start={start} sticky={sticky}"
                            );
                            comparisons += 1;
                        }
                    }
                }
            }
        }
        assert_eq!(comparisons, 658032);
    }

    #[test]
    fn property_dot_contracts_clones_and_every_construction_charge() {
        for source in [
            r"(\p{sc=Han})",
            r"\p{sc=Han}(.)",
            r"(\p{sc=Han})()",
            r"(\p{sc=Han})+(.)",
            r"((\p{sc=Han}))(.)",
            r"(\p{sc=Han}|a)(.)",
            r"(\p{sc=Han})(a)",
            r"(\p{sc=Han})(.)$",
            r"(\p{sc=Han})(.)x",
            r"(\p{Script_Extensions=Latin})(.)",
        ] {
            assert!(
                RegExpUnicodePropertyDotMatcher::compile_with_work(
                    &JsString::from(source),
                    true,
                    false,
                    false,
                    |_| Ok::<(), ()>(())
                )
                .unwrap()
                .is_none(),
                "{source}"
            );
        }
        let source = JsString::from(r"(\P{sc=Hani})(.)");
        let mut calls = 0;
        let plan =
            RegExpUnicodePropertyDotMatcher::compile_with_work(&source, true, true, true, |_| {
                calls += 1;
                Ok::<(), ()>(())
            })
            .unwrap()
            .unwrap();
        assert!(plan.search_passes() >= 4);
        assert_eq!(
            plan.find_with_work(&JsString::from("😀a"), 1, true, |_| Err(17)),
            Err(17)
        );
        for fail in 0..calls {
            let mut at = 0;
            assert!(
                RegExpUnicodePropertyDotMatcher::compile_with_work(
                    &source,
                    true,
                    true,
                    true,
                    |_| {
                        let error = at == fail;
                        at += 1;
                        if error { Err(()) } else { Ok(()) }
                    }
                )
                .is_err()
            );
        }
        assert_eq!(
            plan.clone().find(&JsString::from("😀a"), 1, true),
            Some(RegExpUnicodePropertyDotMatch {
                range: 0..3,
                captures: [0..2, 2..3]
            })
        );
        let plan = compile(r"(\p{sc=Han})(.)", true, false, false);
        assert_eq!(
            plan.find(&JsString::from("一\n𠮷a"), 0, false),
            Some(RegExpUnicodePropertyDotMatch {
                range: 2..5,
                captures: [2..4, 4..5]
            })
        );
    }
}
