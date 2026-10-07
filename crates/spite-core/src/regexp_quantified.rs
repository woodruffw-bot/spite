//! Single consuming Atom with a Quantifier (22.2.2.3, 22.2.2.5–6).

use crate::regexp_character::PreparedCharacter;
use crate::{JsString, RegExpCharacterMatcher, regexp_canonicalize_character};
use std::{ops::Range, sync::Arc};

/// Immutable ordinary-mode matcher for one quantified character or set atom.
///
/// The complete Pattern must already be validated without `u` or `v`. Transparent
/// capturing/noncapturing groups may wrap the atom or its one quantifier.
/// Concatenations, assertions, alternatives, multiple quantifiers and
/// backreferences remain outside this compiler. Repetition bounds and group
/// nesting never expand the atom or use native recursion.
#[derive(Clone, Debug)]
pub struct RegExpQuantifiedMatcher(Arc<Program>);

#[derive(Debug)]
struct Program {
    atom: Atom,
    // None means a minimum larger than any representable input length.
    min: Option<usize>,
    // None means unbounded, or larger than any representable input length.
    max: Option<usize>,
    greedy: bool,
    capture_count: usize,
    whole_captures: usize,
}

#[derive(Debug)]
enum Atom {
    Character { unit: u16, ignore_case: bool },
    Set(RegExpCharacterMatcher),
}

enum PreparedAtom {
    Character(u16),
    Set(PreparedCharacter),
}

impl RegExpQuantifiedMatcher {
    /// Compiles one atom followed by a complete greedy or lazy quantifier.
    pub fn compile(source: &JsString, ignore_case: bool, dot_all: bool) -> Option<Self> {
        Self::compile_with_work(source, ignore_case, dot_all, |_| {
            Ok::<(), std::convert::Infallible>(())
        })
        .unwrap_or_else(|never| match never {})
    }

    /// Accepts the complete subset before charging optional set construction.
    ///
    /// Decimal bounds are compared with input lengths, not converted through
    /// floating point. Overflow preserves their mathematical meaning: an
    /// oversized minimum cannot match, and an oversized maximum cannot constrain
    /// a representable input. Construction failures remain independent host errors.
    pub fn compile_with_work<E>(
        source: &JsString,
        ignore_case: bool,
        dot_all: bool,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, E> {
        let units = source.code_units();
        let Some((prepared, (min, max, greedy), end, captures, whole_captures)) =
            prepare_prefix(units, dot_all)
        else {
            return Ok(None);
        };
        if end != units.len() {
            return Ok(None);
        }
        charge(units.len())?;
        let atom = match prepared {
            PreparedAtom::Character(unit) => Atom::Character {
                unit: canonicalize(unit, ignore_case),
                ignore_case,
            },
            PreparedAtom::Set(set) => Atom::Set(set.compile_with_work(ignore_case, charge)?),
        };
        Ok(Some(Self(Arc::new(Program {
            atom,
            min,
            max,
            greedy,
            capture_count: captures,
            whole_captures,
        }))))
    }

    /// Finds the earliest start, then takes the longest greedy or shortest lazy run.
    ///
    /// Every repetition consumes one UTF-16 unit. Runs too short for the minimum
    /// are skipped together, so search is linear and allocates nothing. Sticky
    /// matching considers only the requested start but may consume its full suffix.
    pub fn find(&self, input: &JsString, start: usize, sticky: bool) -> Option<Range<usize>> {
        let units = input.code_units();
        units.get(start..)?;
        let min = self.0.min?;
        let mut candidate = start;
        loop {
            let remaining = units.len() - candidate;
            if min > remaining {
                return None;
            }
            let limit = if self.0.greedy {
                self.0.max.unwrap_or(remaining).min(remaining)
            } else {
                min
            };
            let count = units[candidate..candidate + limit]
                .iter()
                .take_while(|&&unit| self.0.atom.matches(unit))
                .count();
            if count >= min {
                return Some(candidate..candidate + count);
            }
            if sticky {
                return None;
            }
            // All starts inside this short run also fail the minimum; the next
            // unit fails the atom. min > 0 here, and min <= remaining above.
            candidate += count + 1;
        }
    }

    pub(crate) fn prefix_end(source: &JsString, dot_all: bool) -> Option<usize> {
        prepare_prefix(source.code_units(), dot_all).map(|(_, _, end, _, _)| end)
    }

    /// Number of ordinary capturing wrappers in source order.
    pub fn capture_count(&self) -> usize {
        self.0.capture_count
    }

    /// Absolute range for a capture of a successful match from this plan.
    ///
    /// Groups outside the quantifier capture the whole run; groups inside it
    /// capture only the final one-unit iteration (22.2.2.3.1). Zero iterations
    /// leave those inner groups undefined, while an outer group captures empty.
    pub fn capture_range(&self, index: usize, matched: &Range<usize>) -> Option<Range<usize>> {
        if index >= self.0.capture_count || matched.start > matched.end {
            return None;
        }
        if index < self.0.whole_captures {
            Some(matched.clone())
        } else {
            (matched.start < matched.end).then(|| matched.end - 1..matched.end)
        }
    }

    pub(crate) fn bounds(&self) -> Bounds {
        (self.0.min, self.0.max, self.0.greedy)
    }

    pub(crate) fn matches(&self, unit: u16) -> bool {
        self.0.atom.matches(unit)
    }
}

impl Atom {
    fn matches(&self, candidate: u16) -> bool {
        match self {
            Self::Character { unit, ignore_case } => canonicalize(candidate, *ignore_case) == *unit,
            Self::Set(set) => set.matches(candidate),
        }
    }
}

fn canonicalize(unit: u16, ignore_case: bool) -> u16 {
    regexp_canonicalize_character(u32::from(unit), ignore_case, false) as u16
}

fn prepare_atom(units: &[u16], dot_all: bool) -> Option<(PreparedAtom, usize)> {
    if let Some((set, end)) = PreparedCharacter::parse(units, dot_all) {
        return Some((PreparedAtom::Set(set), end));
    }
    let mut index = 1;
    let unit = *units.first()?;
    let unit = if unit == u16::from(b'\\') {
        crate::regexp_literal::character_escape(units, &mut index)?
    } else if crate::regexp_literal::is_syntax(unit) {
        return None;
    } else {
        unit
    };
    Some((PreparedAtom::Character(unit), index))
}

type Bounds = (Option<usize>, Option<usize>, bool);

type PreparedPrefix = (PreparedAtom, Bounds, usize, usize, usize);

fn prepare_prefix(units: &[u16], dot_all: bool) -> Option<PreparedPrefix> {
    let mut index = 0;
    let mut groups = 0usize;
    let mut captures = 0usize;
    while units.get(index) == Some(&0x28) {
        if units.get(index + 1) == Some(&0x3f) {
            if units.get(index..index + 3) != Some(&[0x28, 0x3f, 0x3a]) {
                return None;
            }
            index += 3;
        } else {
            index += 1;
            captures += 1;
        }
        groups += 1;
    }
    // All wrappers are nested around one atom. Their opening text itself is an
    // implicit stack, so closing them needs no recursive tree or wrapper vector.
    let mut opening_end = index;
    let mut inner_captures = 0usize;
    let (atom, end) = prepare_atom(&units[index..], dot_all)?;
    index += end;
    let mut bounds = None;
    loop {
        if matches!(units.get(index), Some(0x2a | 0x2b | 0x3f | 0x7b)) {
            if bounds.is_some() {
                return None;
            }
            let (parsed, consumed) = quantifier(&units[index..])?;
            bounds = Some(parsed);
            index += consumed;
        }
        if units.get(index) == Some(&0x29) && groups != 0 {
            if opening_end
                .checked_sub(3)
                .and_then(|start| units.get(start..opening_end))
                == Some(&[0x28, 0x3f, 0x3a])
            {
                opening_end -= 3;
            } else {
                opening_end -= 1;
                inner_captures += usize::from(bounds.is_none());
            }
            groups -= 1;
            index += 1;
        } else {
            return (groups == 0).then_some((
                atom,
                bounds?,
                index,
                captures,
                captures - inner_captures,
            ));
        }
    }
}

fn quantifier(units: &[u16]) -> Option<(Bounds, usize)> {
    let (min, max, mut index) = match *units.first()? {
        0x2a => (Some(0), None, 1),
        0x2b => (Some(1), None, 1),
        0x3f => (Some(0), Some(1), 1),
        0x7b => {
            let mut index = 1;
            let min = decimal(units, &mut index)?;
            let max = if units.get(index) == Some(&0x2c) {
                index += 1;
                if units.get(index) == Some(&0x7d) {
                    None
                } else {
                    decimal(units, &mut index)?
                }
            } else {
                min
            };
            if units.get(index) != Some(&0x7d) {
                return None;
            }
            (min, max, index + 1)
        }
        _ => return None,
    };
    let greedy = units.get(index) != Some(&0x3f);
    if !greedy {
        index += 1;
    }
    // Validated Patterns guarantee the mathematical minimum <= maximum.
    Some(((min, max, greedy), index))
}

fn decimal(units: &[u16], index: &mut usize) -> Option<Option<usize>> {
    let start = *index;
    let mut value = Some(0usize);
    while let Some(&digit @ 0x30..=0x39) = units.get(*index) {
        value = value.and_then(|value| {
            value
                .checked_mul(10)?
                .checked_add(usize::from(digit - 0x30))
        });
        *index += 1;
    }
    (*index != start).then_some(value)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fmt::Write;

    #[test]
    fn quantified_atoms_snapshot() {
        let mut rows = String::new();
        for source in [
            "a*",
            "a*?",
            "a+",
            "a+?",
            "a?",
            "a??",
            "a{0}",
            "a{2}",
            "a{2,}",
            "a{2,}?",
            "a{1,3}",
            "a{1,3}?",
            "a{00,02}",
            "a{0,999999999999999999999999999999}",
            "a{999999999999999999999999999999,}",
            "[ab]+",
            "[^b]{1,3}",
            "[]*",
            "[]+",
            "[^]{2}",
            r"\d+",
            r"\w+?",
            r"\s*",
            ".+",
            ".+?",
            r"\uD800+",
            r"\++",
            "a",
            "aa+",
            "(a)+",
            "(?:a)+",
            "a+|b",
            "^a+$",
            r"\1+",
            "[a]+b",
        ] {
            for (ignore_case, dot_all) in [(false, false), (true, false), (false, true)] {
                write!(rows, "{source:?} i={ignore_case} s={dot_all}").unwrap();
                if let Some(matcher) = matcher(source, ignore_case, dot_all) {
                    for input in [
                        "",
                        "a",
                        "aaa",
                        "baaab",
                        "abbaa",
                        "Aaa",
                        "12x",
                        " \n",
                        "\n",
                        "+++",
                        "\u{10000}",
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

    fn matcher(source: &str, i: bool, s: bool) -> Option<RegExpQuantifiedMatcher> {
        RegExpQuantifiedMatcher::compile(&JsString::from(source), i, s)
    }

    #[test]
    fn quantified_capture_snapshot() {
        let mut rows = String::new();
        for source in [
            "(a)+",
            "(a+)",
            "((a)+)",
            "((a))+",
            "((a+))",
            "(?:(a)+)",
            "((?:a)+)",
            "((?:a))+",
            "(?:((a)+))",
            "(a){0}",
            "(a{0})",
            "((a)){0}",
            "((a{0}))",
            "(a)?",
            "(a??)",
            "(a)??",
            "((a)?)",
            "([ab]){1,3}",
            "([ab]{1,3}?)",
            "([^])*",
            "([])*",
            "([]*)",
            r"(\d)+",
            "(.)+",
            "(.+?)",
            r"(\uD800)+",
            "(µ)+",
            "(ab)+",
            "(a)+b",
            "(?<x>a)+",
            r"(a)\1+",
            "(a*)?",
            "((a)+)+",
        ] {
            for (i, s) in [(false, false), (true, false), (false, true)] {
                write!(rows, "{source:?} i={i} s={s}").unwrap();
                if let Some(m) = matcher(source, i, s) {
                    write!(rows, " captures={}", m.capture_count()).unwrap();
                    for input in [
                        "",
                        "a",
                        "aaa",
                        "baaa",
                        "Aaa",
                        "abba",
                        "12x",
                        "\n",
                        "a\nb",
                        "Μµ",
                        "\u{10000}",
                    ] {
                        let input = JsString::from(input);
                        for (start, sticky) in [(0, false), (1, true)] {
                            let range = m.find(&input, start, sticky);
                            let captures = range.as_ref().map(|r| {
                                (0..m.capture_count())
                                    .map(|c| m.capture_range(c, r))
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

    #[test]
    fn nested_capture_placement_agrees_with_independent_final_iteration_ranges() {
        for (source, whole, last, min, max, greedy) in [
            ("((([ab]))+)", 1, 2, 1, None, true),
            ("((?:([ab])){0,2})", 1, 1, 0, Some(2), true),
            ("(?:(?:([ab])))??", 0, 1, 0, Some(1), false),
            ("((?:[ab]+?))", 1, 0, 1, None, false),
            ("(([ab]{0}))", 2, 0, 0, Some(0), true),
            ("(([ab])){0}", 0, 2, 0, Some(0), true),
        ] {
            let m = matcher(source, false, false).unwrap();
            assert_eq!(m.capture_count(), whole + last);
            for len in 0..=5u32 {
                for mut encoded in 0..3usize.pow(len) {
                    let mut units = Vec::new();
                    for _ in 0..len {
                        units.push([97, 98, 120][encoded % 3]);
                        encoded /= 3;
                    }
                    let input = JsString::from_code_units(units.clone());
                    for start in 0..=units.len() + 1 {
                        for sticky in [false, true] {
                            let expected = (start..=units.len())
                                .take(if sticky { 1 } else { usize::MAX })
                                .find_map(|candidate| {
                                    let available = units[candidate..]
                                        .iter()
                                        .take_while(|u| [97, 98].contains(u))
                                        .count();
                                    if available < min {
                                        return None;
                                    }
                                    let count = if greedy {
                                        max.unwrap_or(available).min(available)
                                    } else {
                                        min
                                    };
                                    Some(candidate..candidate + count)
                                });
                            assert_eq!(
                                m.find(&input, start, sticky),
                                expected,
                                "{source} {units:?} {start} {sticky}"
                            );
                            if let Some(r) = expected {
                                for index in 0..whole + last {
                                    let expected = if index < whole {
                                        Some(r.clone())
                                    } else if r.is_empty() {
                                        None
                                    } else {
                                        Some(r.end - 1..r.end)
                                    };
                                    assert_eq!(
                                        m.capture_range(index, &r),
                                        expected,
                                        "{source} capture={index}"
                                    );
                                }
                                assert_eq!(m.capture_range(whole + last, &r), None);
                            }
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn capture_layout_is_compact_at_deep_nesting_and_continuation_boundaries() {
        let depth = 100000;
        let outside =
            JsString::from(format!("{}a+{}", "(".repeat(depth), ")".repeat(depth)).as_str());
        let m = RegExpQuantifiedMatcher::compile(&outside, false, false).unwrap();
        assert_eq!(m.capture_count(), depth);
        let input = JsString::from("aaaa");
        let range = m.find(&input, 0, false).unwrap();
        assert_eq!(m.capture_range(0, &range), Some(0..4));
        assert_eq!(m.clone().capture_range(depth - 1, &range), Some(0..4));
        let inside =
            JsString::from(format!("{}a{}*", "(".repeat(depth), ")".repeat(depth)).as_str());
        let m = RegExpQuantifiedMatcher::compile(&inside, false, false).unwrap();
        assert_eq!(m.capture_count(), depth);
        let range = m.find(&input, 0, false).unwrap();
        assert_eq!(m.capture_range(0, &range), Some(3..4));
        assert_eq!(m.capture_range(depth - 1, &range), Some(3..4));
        let empty = m.find(&JsString::from(""), 0, false).unwrap();
        assert_eq!(m.capture_range(0, &empty), None);
        let suffix = crate::RegExpQuantifiedContinuationMatcher::compile(
            &JsString::from("((a)+)aa"),
            false,
            false,
        )
        .unwrap();
        assert_eq!(suffix.capture_count(), 2);
        let range = suffix.find(&input, 0, false).unwrap();
        assert_eq!(range, 0..4);
        assert_eq!(suffix.capture_range(0, &range), Some(0..2));
        assert_eq!(suffix.capture_range(1, &range), Some(1..2));
        let anchored =
            crate::RegExpAnchoredMatcher::compile(&JsString::from("^((a)+)aa$"), false, false)
                .unwrap();
        assert_eq!(anchored.capture_count(), 2);
        assert_eq!(
            anchored.capture_range(1, &anchored.find(&input, 0, false).unwrap()),
            Some(1..2)
        );
        let choices = crate::RegExpDisjunctionMatcher::compile(
            &JsString::from("([x])|((a)+)aa|([y])"),
            false,
        )
        .unwrap();
        assert_eq!(choices.capture_count(), 4);
        let (branch, range) = choices.find_branch(&input, 0, false).unwrap();
        assert_eq!(branch, 1);
        assert_eq!(choices.capture_range(branch, 0, &range), None);
        assert_eq!(choices.capture_range(branch, 1, &range), Some(0..2));
        assert_eq!(choices.capture_range(branch, 2, &range), Some(1..2));
        assert_eq!(choices.capture_range(branch, 3, &range), None);
        assert_eq!(choices.branch_captures(branch), None);
    }

    #[test]
    fn transparent_noncapturing_quantifier_snapshot() {
        let mut rows = String::new();
        for source in [
            "(?:a)+",
            "(?:a+)",
            "(?:(?:a)+?)",
            "(?:(?:a+?))",
            "(?:[ab]){2,3}",
            "(?:[ab]{2,3}?)",
            "(?:[])*",
            "(?:[^]{2})",
            r"(?:\d)+",
            "(?:.)+",
            "(?:a)",
            "(?:ab)+",
            "(?:(?:a)+)+",
            "(?:a*)?",
            "(a)+",
            "(?:a+)(?:)",
        ] {
            for (i, s) in [(false, false), (true, false), (false, true)] {
                write!(rows, "{source:?} i={i} s={s}").unwrap();
                if let Some(m) = matcher(source, i, s) {
                    for input in ["", "a", "baaa", "Aaa", "abba", "12x", "\n", "\u{10000}"] {
                        let input = JsString::from(input);
                        write!(
                            rows,
                            " {input:?}:{:?}/{:?}",
                            m.find(&input, 0, false),
                            m.find(&input, 1, true)
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
    fn arbitrary_transparent_nesting_preserves_runs_without_native_recursion() {
        for (body, suffix) in [("[ab]", "+?"), ("[ab]+?", ""), ("a", "{1,3}")] {
            let source = format!(
                "{}{}{}{}",
                "(?:".repeat(100_000),
                body,
                ")".repeat(100_000),
                suffix
            );
            let m = matcher(&source, false, false).unwrap();
            assert_eq!(
                m.clone().find(&JsString::from("aaa"), 0, false),
                Some(0..if body == "a" { 3 } else { 1 })
            );
        }
        let rejected = RegExpQuantifiedMatcher::compile_with_work(
            &JsString::from(r"(?:\d)+b"),
            false,
            false,
            |_| Err("must not charge"),
        );
        assert!(matches!(rejected, Ok(None)));
    }

    #[test]
    fn exhaustive_candidate_and_repetition_order_matches_independent_oracle() {
        let alphabet = [
            u16::from(b'a'),
            u16::from(b'b'),
            u16::from(b'A'),
            0x0a,
            0xd800,
        ];
        for (atom, predicate) in [
            ("[ab]", (|u| matches!(u, 0x61 | 0x62)) as fn(u16) -> bool),
            ("[]", (|_| false) as fn(u16) -> bool),
            ("[^]", (|_| true) as fn(u16) -> bool),
        ] {
            for (suffix, min, max, greedy) in [
                ("*", 0, usize::MAX, true),
                ("*?", 0, usize::MAX, false),
                ("+", 1, usize::MAX, true),
                ("+?", 1, usize::MAX, false),
                ("?", 0, 1, true),
                ("??", 0, 1, false),
                ("{0}", 0, 0, true),
                ("{2}", 2, 2, true),
                ("{2,}", 2, usize::MAX, true),
                ("{2,}?", 2, usize::MAX, false),
                ("{1,3}", 1, 3, true),
                ("{1,3}?", 1, 3, false),
            ] {
                let source = format!("{atom}{suffix}");
                let matcher = matcher(&source, false, false).unwrap();
                for len in 0..=5u32 {
                    for mut encoded in 0..5usize.pow(len) {
                        let mut units = Vec::new();
                        for _ in 0..len {
                            units.push(alphabet[encoded % 5]);
                            encoded /= 5;
                        }
                        let input = JsString::from_code_units(units);
                        for start in 0..=input.len() + 1 {
                            for sticky in [false, true] {
                                let expected = (start..=input.len())
                                    .take(if sticky { 1 } else { usize::MAX })
                                    .find_map(|candidate| {
                                        let mut lengths: Vec<_> = (min..=max
                                            .min(input.len() - candidate))
                                            .filter(|&len| {
                                                input.code_units()[candidate..candidate + len]
                                                    .iter()
                                                    .copied()
                                                    .all(predicate)
                                            })
                                            .collect();
                                        if greedy {
                                            lengths.reverse();
                                        }
                                        lengths.first().map(|len| candidate..candidate + len)
                                    });
                                assert_eq!(
                                    matcher.find(&input, start, sticky),
                                    expected,
                                    "{source} {input:?} start={start} sticky={sticky}"
                                );
                            }
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn huge_bounds_long_runs_and_optional_host_failures_stay_distinct() {
        let digits = "9".repeat(100_000);
        let m = matcher(&format!("a{{0,{digits}}}"), false, false).unwrap();
        let input = JsString::from_code_units(vec![u16::from(b'a'); 300_000]);
        assert_eq!(m.clone().find(&input, 0, true), Some(0..300_000));
        assert_eq!(
            matcher(&format!("a{{{digits},}}"), false, false)
                .unwrap()
                .find(&input, 0, false),
            None
        );
        assert_eq!(
            matcher(&format!("a{{0,{digits}}}?"), false, false)
                .unwrap()
                .find(&input, 0, false),
            Some(0..0)
        );
        let abort = RegExpQuantifiedMatcher::compile_with_work(
            &JsString::from(r"\d+"),
            false,
            false,
            |work| {
                if work == 65_536 {
                    Err("host work")
                } else {
                    Ok(())
                }
            },
        );
        assert!(matches!(abort, Err("host work")));
        let rejected = RegExpQuantifiedMatcher::compile_with_work(
            &JsString::from(r"\d+b"),
            false,
            false,
            |_| Err("must not charge"),
        );
        assert!(matches!(rejected, Ok(None)));
    }
}
