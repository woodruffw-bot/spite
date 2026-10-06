//! Fixed ordinary-mode Atom concatenation and capture ranges (22.2.2.3, 22.2.2.7).

use crate::regexp_character::PreparedCharacter;
use crate::{JsString, RegExpCharacterMatcher, regexp_canonicalize_character};
use std::{collections::HashMap, ops::Range, sync::Arc};

/// Immutable fixed-width sequence of ordinary characters and character-set atoms.
///
/// The complete Pattern must already be validated without `u` or `v`. Ordinary
/// capturing/noncapturing groups are flattened iteratively. Assertions, choices,
/// quantifiers, backreferences and named/modifier groups remain unsupported.
/// Search checks candidate starts in order without allocations or backtracking.
/// Its conservative worst-case work is input length times consuming atom count.
#[derive(Clone, Debug)]
pub struct RegExpSequenceMatcher(Arc<Program>);

#[derive(Debug)]
struct Program {
    terms: Vec<Term>,
    captures: Vec<Range<usize>>,
    ignore_case: bool,
}

#[derive(Debug)]
enum Term {
    Character(u16),
    Set(RegExpCharacterMatcher),
}

enum PreparedTerm {
    Character(u16),
    Set {
        plan: PreparedCharacter,
        source: Range<usize>,
    },
}

impl RegExpSequenceMatcher {
    /// Compiles the complete fixed-width subset, returning None for other syntax.
    pub fn compile(source: &JsString, ignore_case: bool, dot_all: bool) -> Option<Self> {
        Self::compile_with_work(source, ignore_case, dot_all, |_| {
            Ok::<(), std::convert::Infallible>(())
        })
        .unwrap_or_else(|never| match never {})
    }

    /// Parses the complete Pattern before constructing sets or charging their work.
    ///
    /// Unsupported syntax returns Ok(None). Construction charge failures remain
    /// independent host errors. Identical atom source within this compilation
    /// shares its immutable set plan. Cache setup/lookups and first construction
    /// retain their work charges; literal terms need one canonicalization each.
    pub fn compile_with_work<E>(
        source: &JsString,
        ignore_case: bool,
        dot_all: bool,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, E> {
        let Some((prepared, captures)) = prepare(source.code_units(), dot_all) else {
            return Ok(None);
        };
        charge(source.len())?;
        charge(prepared.len())?;
        // Reserving for all set terms prevents rehashing already charged keys.
        // Cache keys borrow this compilation's immutable source, never the plan.
        let set_count = prepared
            .iter()
            .filter(|term| matches!(term, PreparedTerm::Set { .. }))
            .count();
        charge(prepared.len())?;
        charge(set_count)?;
        charge(set_count)?;
        let mut sets: HashMap<&[u16], RegExpCharacterMatcher> = HashMap::with_capacity(set_count);
        let mut terms = Vec::with_capacity(prepared.len());
        for term in prepared {
            terms.push(match term {
                PreparedTerm::Character(unit) => Term::Character(canonicalize(unit, ignore_case)),
                PreparedTerm::Set {
                    plan,
                    source: range,
                } => {
                    let key = &source.code_units()[range];
                    charge(key.len())?;
                    charge(key.len())?;
                    let set = if let Some(set) = sets.get(key) {
                        set.clone()
                    } else {
                        charge(key.len())?;
                        let set = plan.compile_with_work(ignore_case, &mut charge)?;
                        sets.insert(key, set.clone());
                        set
                    };
                    Term::Set(set)
                }
            });
        }
        Ok(Some(Self(Arc::new(Program {
            terms,
            captures,
            ignore_case,
        }))))
    }

    /// Ordered capture endpoints relative to a successful match's UTF-16 start.
    pub fn capture_ranges(&self) -> &[Range<usize>] {
        &self.0.captures
    }

    /// Number of consuming atoms, for conservative optional search accounting.
    pub fn atom_count(&self) -> usize {
        self.0.terms.len()
    }

    /// Finds the earliest matching start, or checks only the requested sticky start.
    ///
    /// Every consuming atom uses one UTF-16 unit, including lone surrogates.
    /// Empty sequences match at the requested start through the input's end.
    pub fn find(&self, input: &JsString, start: usize, sticky: bool) -> Option<Range<usize>> {
        self.find_if(input, start, sticky, |_| true)
    }

    /// Continues candidate search after an outer boundary rejects a complete match.
    pub(crate) fn find_if(
        &self,
        input: &JsString,
        start: usize,
        sticky: bool,
        mut accept: impl FnMut(&Range<usize>) -> bool,
    ) -> Option<Range<usize>> {
        let input = input.code_units();
        input.get(start..)?;
        let last = input.len().checked_sub(self.0.terms.len())?;
        if start > last {
            return None;
        }
        let end = if sticky { start } else { last };
        (start..=end).find_map(|candidate| {
            let matched = self
                .0
                .terms
                .iter()
                .zip(&input[candidate..])
                .all(|(term, &unit)| match term {
                    Term::Character(expected) => {
                        canonicalize(unit, self.0.ignore_case) == *expected
                    }
                    Term::Set(set) => set.matches(unit),
                });
            matched
                .then_some(candidate..candidate + self.0.terms.len())
                .filter(&mut accept)
        })
    }
}

fn canonicalize(unit: u16, ignore_case: bool) -> u16 {
    regexp_canonicalize_character(u32::from(unit), ignore_case, false) as u16
}

fn prepare(source: &[u16], dot_all: bool) -> Option<(Vec<PreparedTerm>, Vec<Range<usize>>)> {
    let mut index = 0;
    let mut terms = Vec::new();
    let mut captures = Vec::new();
    let mut groups = Vec::new();
    while let Some(&unit) = source.get(index) {
        if unit == u16::from(b'(') {
            index += 1;
            let capture = if source.get(index) == Some(&u16::from(b'?')) {
                if source.get(index..index + 2)? != [u16::from(b'?'), u16::from(b':')] {
                    return None;
                }
                index += 2;
                None
            } else {
                let capture = captures.len();
                captures.push(terms.len()..terms.len());
                Some(capture)
            };
            groups.push(capture);
        } else if unit == u16::from(b')') {
            index += 1;
            if let Some(capture) = groups.pop()? {
                captures[capture].end = terms.len();
            }
        } else if let Some((prepared, consumed)) =
            PreparedCharacter::parse(&source[index..], dot_all)
        {
            terms.push(PreparedTerm::Set {
                plan: prepared,
                source: index..index + consumed,
            });
            index += consumed;
        } else {
            index += 1;
            let unit = if unit == u16::from(b'\\') {
                crate::regexp_literal::character_escape(source, &mut index)?
            } else if crate::regexp_literal::is_syntax(unit) {
                return None;
            } else {
                unit
            };
            terms.push(PreparedTerm::Character(unit));
        }
    }
    groups.is_empty().then_some((terms, captures))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fmt::Write;

    type UnitPredicate = fn(u16) -> bool;

    #[test]
    fn fixed_sequences_and_capture_ranges_snapshot() {
        let mut rows = String::new();
        for source in [
            "[Nn]evermore",
            "a[ab]b",
            "[ab][^b]",
            r"\d\w\s",
            "a.b",
            "([a])",
            "([a](b))()",
            "(?:[a])([b])",
            "([a])([b])",
            "[]a",
            "[^]a",
            r"[\b\-\]]a",
            r"\uD800.\uDC00",
            "",
            "[a]*b",
            "[a]+",
            "[a]{1}",
            "[a]|b",
            "([a]|b)",
            "^[a]",
            "[a]$",
            r"([a])\1",
            "(?<x>[a])",
            "(?i:[a])",
            "[a",
            "([a]",
        ] {
            for ignore_case in [false, true] {
                let matcher =
                    RegExpSequenceMatcher::compile(&JsString::from(source), ignore_case, false);
                let matches = matcher.as_ref().map(|m| {
                    [
                        "Nevermore nevermore",
                        "Aabb",
                        "1a ",
                        "a\nb",
                        "ab",
                        "\u{8}-]a",
                    ]
                    .map(|input| m.find(&JsString::from(input), 0, false))
                });
                writeln!(
                    rows,
                    "{source:?} i={ignore_case}: captures={:?} matches={matches:?}",
                    matcher.as_ref().map(|m| m.capture_ranges())
                )
                .unwrap();
            }
        }
        insta::assert_snapshot!(rows);
    }

    #[test]
    fn candidate_order_and_sticky_ranges_agree_with_independent_position_oracle() {
        let cases: [(&str, &[UnitPredicate]); 5] = [
            (
                "[ab]a[ab]",
                &[|c| c == 97 || c == 98, |c| c == 97, |c| c == 97 || c == 98],
            ),
            ("[ab][^b]", &[|c| c == 97 || c == 98, |c| c != 98]),
            (
                "a.[ab]",
                &[
                    |c| c == 97,
                    |c| !matches!(c, 10 | 13 | 0x2028 | 0x2029),
                    |c| c == 97 || c == 98,
                ],
            ),
            (
                r"\d\wa",
                &[
                    |c| (48..=57).contains(&c),
                    |c| matches!(c, 48..=57 | 65..=90 | 95 | 97..=122),
                    |c| c == 97,
                ],
            ),
            ("([a])()([b])", &[|c| c == 97, |c| c == 98]),
        ];
        let alphabet = [97, 98, 65, 49, 10, 0xd800];
        for length in 0..=4u32 {
            for mut encoded in 0..alphabet.len().pow(length) {
                let mut units = Vec::new();
                for _ in 0..length {
                    units.push(alphabet[encoded % alphabet.len()]);
                    encoded /= alphabet.len();
                }
                let input = JsString::from_code_units(units.clone());
                for (source, predicates) in &cases {
                    let matcher =
                        RegExpSequenceMatcher::compile(&JsString::from(*source), false, false)
                            .unwrap();
                    for start in 0..=units.len() + 1 {
                        for sticky in [false, true] {
                            let expected = (start..=units.len()).find_map(|candidate| {
                                if sticky && candidate != start {
                                    return None;
                                }
                                let end = candidate.checked_add(predicates.len())?;
                                let candidate_units = units.get(candidate..end)?;
                                candidate_units
                                    .iter()
                                    .zip(*predicates)
                                    .all(|(&unit, predicate)| predicate(unit))
                                    .then_some(candidate..end)
                            });
                            assert_eq!(
                                matcher.find(&input, start, sticky),
                                expected,
                                "{source} {units:?} start={start} sticky={sticky}"
                            );
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn deep_groups_and_long_patterns_compile_clone_and_drop_without_recursion() {
        let source =
            JsString::from(format!("{}[a]{}", "(".repeat(10_000), ")".repeat(10_000)).as_str());
        let matcher = RegExpSequenceMatcher::compile(&source, false, false).unwrap();
        assert_eq!(matcher.capture_ranges().len(), 10_000);
        assert!(
            matcher
                .capture_ranges()
                .iter()
                .all(|range| *range == (0..1))
        );
        assert_eq!(
            matcher.clone().find(&JsString::from("a"), 0, true),
            Some(0..1)
        );
        let source = JsString::from(format!("[a]{}", "a".repeat(100_000)).as_str());
        let matcher = RegExpSequenceMatcher::compile(&source, false, false).unwrap();
        assert_eq!(matcher.atom_count(), 100_001);
        assert_eq!(
            matcher.find(&JsString::from("a".repeat(100_001).as_str()), 0, true),
            Some(0..100_001)
        );
    }

    #[test]
    fn construction_accounting_and_unsupported_syntax_remain_distinct() {
        let mut charges = Vec::new();
        let matcher = RegExpSequenceMatcher::compile_with_work(
            &JsString::from("[a]b\\d"),
            false,
            false,
            |work| {
                charges.push(work);
                Ok::<(), ()>(())
            },
        )
        .unwrap()
        .unwrap();
        assert_eq!(
            charges,
            [6, 3, 3, 2, 2, 3, 3, 3, 3, 1024, 1, 2, 2, 2, 2, 1024, 65_536]
        );
        assert_eq!(matcher.atom_count(), 3);
        let result = RegExpSequenceMatcher::compile_with_work(
            &JsString::from("[a]b\\d"),
            false,
            false,
            |work| {
                if work == 65_536 {
                    Err("host abort")
                } else {
                    Ok(())
                }
            },
        );
        assert!(matches!(result, Err("host abort")));
        assert!(
            RegExpSequenceMatcher::compile_with_work(
                &JsString::from("[a][b]*"),
                false,
                false,
                |_| -> Result<(), ()> {
                    panic!("complete unsupported Pattern must be rejected before construction")
                }
            )
            .unwrap()
            .is_none()
        );
    }

    #[test]
    fn repeated_sets_fit_opted_in_work_and_keep_independent_capture_positions() {
        let source = JsString::from("[a]".repeat(1000).as_str());
        let mut remaining = 20_000usize;
        let matcher = RegExpSequenceMatcher::compile_with_work(&source, false, false, |work| {
            remaining = remaining.checked_sub(work).ok_or("host abort")?;
            Ok::<(), &str>(())
        })
        .unwrap()
        .unwrap();
        assert_eq!(
            matcher.find(&JsString::from("a".repeat(1000).as_str()), 0, true),
            Some(0..1000)
        );
        let mut remaining = 70_000usize;
        let matcher = RegExpSequenceMatcher::compile_with_work(
            &JsString::from(r"(\d)(\d)"),
            false,
            false,
            |work| {
                remaining = remaining.checked_sub(work).ok_or("host abort")?;
                Ok::<(), &str>(())
            },
        )
        .unwrap()
        .unwrap();
        assert_eq!(matcher.capture_ranges(), [0..1, 1..2]);
        assert_eq!(matcher.find(&JsString::from("12"), 0, true), Some(0..2));
        let mut remaining = 70_000usize;
        assert!(matches!(
            RegExpSequenceMatcher::compile_with_work(
                &JsString::from(r"\d\w"),
                false,
                false,
                |work| {
                    remaining = remaining.checked_sub(work).ok_or("host abort")?;
                    Ok::<(), &str>(())
                }
            ),
            Err("host abort")
        ));
        let matcher =
            RegExpSequenceMatcher::compile(&JsString::from("([a])([^a])([a])"), true, false)
                .unwrap();
        assert_eq!(matcher.capture_ranges(), [0..1, 1..2, 2..3]);
        assert_eq!(matcher.find(&JsString::from("AbA"), 0, true), Some(0..3));
    }

    #[test]
    fn large_repeated_set_sequences_compile_clone_and_match_with_unlimited_defaults() {
        let source = JsString::from("[a]".repeat(100_000).as_str());
        let matcher = RegExpSequenceMatcher::compile(&source, false, false).unwrap();
        let input = JsString::from("a".repeat(100_000).as_str());
        assert_eq!(matcher.clone().find(&input, 0, true), Some(0..100_000));
        assert_eq!(matcher.atom_count(), 100_000);
    }
}
