//! Fixed ordinary-mode Atom concatenation and capture ranges (22.2.2.3, 22.2.2.7).

use crate::regexp_assertion::Assertions;
use crate::regexp_character::PreparedCharacter;
use crate::{JsString, RegExpCharacterMatcher, regexp_canonicalize_character};
use std::{collections::HashMap, ops::Range, sync::Arc};

/// Immutable fixed-width sequence of ordinary characters and character-set atoms.
///
/// The complete Pattern must already be validated without `u` or `v`. Ordinary
/// capturing/noncapturing groups and word assertions are flattened iteratively.
/// One-unit literal/class/escape/dot choices preserve static capture endpoints;
/// other choices,
/// quantifiers, backreferences and named/modifier groups remain unsupported.
/// Search checks candidate starts in order without allocations or backtracking.
/// Its conservative work is input length times consuming terms and assertion offsets.
#[derive(Clone, Debug)]
pub struct RegExpSequenceMatcher(Arc<Program>);

#[derive(Debug)]
struct Program {
    terms: Vec<Term>,
    captures: Vec<Range<usize>>,
    boundaries: Vec<(usize, Assertions)>,
    ignore_case: bool,
    multiline: bool,
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

struct PreparedSequence {
    terms: Vec<PreparedTerm>,
    captures: Vec<Range<usize>>,
    boundaries: Vec<(usize, Assertions)>,
}

impl RegExpSequenceMatcher {
    pub(crate) fn fixed_capture_layout(
        source: &JsString,
        dot_all: bool,
    ) -> Option<(usize, Vec<Range<usize>>)> {
        let (prepared, _, _) = prepare_source(source, dot_all, true)?;
        Some((prepared.terms.len(), prepared.captures))
    }

    pub(crate) fn repeated_atom_width(
        source: &JsString,
        dot_all: bool,
        assertions: bool,
    ) -> Option<usize> {
        let (prepared, _, _) = prepare_source(source, dot_all, assertions)?;
        if assertions {
            Some(prepared.terms.len())
        } else {
            (prepared.terms.len() >= 2 && prepared.boundaries.is_empty())
                .then_some(prepared.terms.len())
        }
    }

    /// Compiles fixed terms and word assertions, returning None for other syntax.
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
        charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, E> {
        Self::compile_plan(source, ignore_case, false, dot_all, false, charge)
    }

    /// Compiles fixed bodies with input/line assertions and an explicit multiline flag.
    ///
    /// The complete Pattern must be validated without `u` or `v`. Assertions in
    /// ordinary groups use their consuming offsets; this does not add quantified
    /// groups or variable concatenations. The existing word-only entry points
    /// preserve their flag-independent behavior for quantified compositions.
    pub fn compile_with_assertions_and_work<E>(
        source: &JsString,
        ignore_case: bool,
        multiline: bool,
        dot_all: bool,
        charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, E> {
        Self::compile_plan(source, ignore_case, multiline, dot_all, true, charge)
    }

    fn compile_plan<E>(
        source: &JsString,
        ignore_case: bool,
        multiline: bool,
        dot_all: bool,
        input_assertions: bool,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, E> {
        let Some((
            PreparedSequence {
                terms: prepared,
                captures,
                boundaries,
            },
            matching_source,
            preparation_passes,
        )) = prepare_source(source, dot_all, input_assertions)
        else {
            return Ok(None);
        };
        charge(source.len())?;
        for _ in 0..preparation_passes {
            charge(matching_source.len())?;
        }
        charge(prepared.len())?;
        if !boundaries.is_empty() {
            charge(boundaries.len())?;
        }
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
                    let key = &matching_source.code_units()[range];
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
            boundaries,
            ignore_case,
            multiline,
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

    /// Conservative optional passes including boundary checks at fixed offsets.
    pub fn search_passes(&self, sticky: bool) -> usize {
        let consuming = if sticky { 1 } else { self.atom_count().max(1) };
        consuming.saturating_add(self.0.boundaries.len().saturating_mul(2))
    }

    pub(crate) fn has_assertions(&self) -> bool {
        !self.0.boundaries.is_empty()
    }

    /// Finds the earliest matching start, or checks only the requested sticky start.
    ///
    /// Every consuming atom uses one UTF-16 unit, including lone surrogates.
    /// Empty consuming bodies still test assertions through the input's end.
    pub fn find(&self, input: &JsString, start: usize, sticky: bool) -> Option<Range<usize>> {
        self.find_if(input, start, sticky, |_| true)
    }

    /// Streams overlapping fixed-width matches in increasing candidate order.
    /// Every candidate is checked once; search allocates no result list.
    pub(crate) fn matches_from<'a>(
        &'a self,
        input: &'a [u16],
        start: usize,
    ) -> Option<SequenceMatches<'a>> {
        input.get(start..)?;
        let last = input.len().checked_sub(self.atom_count())?;
        Some(SequenceMatches {
            matcher: self,
            input,
            next: (start <= last).then_some(start),
            last,
        })
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
        if !sticky {
            return self.matches_from(input, start)?.find(&mut accept);
        }
        let range = start..start + self.atom_count();
        matches_at(&self.0, input, start)
            .then_some(range)
            .filter(&mut accept)
    }
}

pub(crate) struct SequenceMatches<'a> {
    matcher: &'a RegExpSequenceMatcher,
    input: &'a [u16],
    next: Option<usize>,
    last: usize,
}

impl Iterator for SequenceMatches<'_> {
    type Item = Range<usize>;
    fn next(&mut self) -> Option<Self::Item> {
        while let Some(candidate) = self.next {
            self.next = candidate.checked_add(1).filter(|&next| next <= self.last);
            let range = candidate..candidate + self.matcher.atom_count();
            if matches_at(&self.matcher.0, self.input, candidate) {
                return Some(range);
            }
        }
        None
    }
}

fn matches_at(program: &Program, input: &[u16], start: usize) -> bool {
    if !program
        .boundaries
        .iter()
        .all(|&(offset, assertions)| assertions.accepts(input, start + offset, program.multiline))
    {
        return false;
    }
    let input = &input[start..start + program.terms.len()];
    program
        .terms
        .iter()
        .zip(input)
        .all(|(term, &unit)| match term {
            Term::Character(expected) => canonicalize(unit, program.ignore_case) == *expected,
            Term::Set(set) => set.matches(unit),
        })
}

fn canonicalize(unit: u16, ignore_case: bool) -> u16 {
    regexp_canonicalize_character(u32::from(unit), ignore_case, false) as u16
}

fn prepare_source(
    source: &JsString,
    dot_all: bool,
    input_assertions: bool,
) -> Option<(PreparedSequence, JsString, usize)> {
    if let Some(prepared) = prepare_units(source.code_units(), dot_all, input_assertions) {
        return Some((prepared, source.clone(), 0));
    }
    if let Some(normalized) = normalize_unit_choices(source.code_units(), dot_all) {
        let normalized = JsString::from_code_units(normalized);
        let prepared = prepare_units(normalized.code_units(), dot_all, input_assertions)?;
        return Some((prepared, normalized, 1));
    }
    let mut choices = HashMap::new();
    for range in choice_group_ranges(source.code_units())? {
        let (plan, captures) = unit_choice_plan(&source.code_units()[range.clone()], dot_all)?;
        choices.insert(range.start, (range.end, plan, captures));
    }
    if choices.is_empty() {
        return None;
    }
    let prepared =
        prepare_units_with_choices(source.code_units(), dot_all, input_assertions, choices)?;
    // Account for the additional group/branch/fixed preparation passes.
    Some((prepared, source.clone(), 3))
}

pub(crate) fn normalize_unit_choices(units: &[u16], dot_all: bool) -> Option<Vec<u16>> {
    let ranges = choice_group_ranges(units)?;
    let mut normalized = Vec::new();
    let mut start = 0;
    for range in ranges {
        normalized.extend_from_slice(units.get(start..range.start)?);
        normalized.extend(unit_choice_atom(&units[range.clone()], dot_all)?);
        start = range.end;
    }
    normalized.extend_from_slice(&units[start..]);
    Some(normalized)
}

pub(crate) fn choice_group_ranges(units: &[u16]) -> Option<Vec<Range<usize>>> {
    // Inventory direct choice groups iteratively. The containing plan validates
    // their branch widths/captures before any predicate construction.
    if !units.contains(&124) {
        return None;
    }
    let mut groups = Vec::new();
    let mut ranges = Vec::new();
    let mut index = 0;
    let mut in_class = false;
    while let Some(&unit) = units.get(index) {
        index += 1;
        if in_class {
            if unit == 92 {
                units.get(index)?;
                index += 1;
            } else if unit == 93 {
                in_class = false;
            }
            continue;
        }
        match unit {
            92 => {
                units.get(index)?;
                index += 1;
            }
            91 => in_class = true,
            40 => {
                let start = index - 1;
                if units.get(index) == Some(&63) {
                    if units.get(index..index + 2)? != [63, 58] {
                        return None;
                    }
                    index += 2;
                }
                groups.push((start, false));
            }
            41 => {
                let (start, choices) = groups.pop()?;
                if choices {
                    ranges.push(start..index);
                }
            }
            124 => groups.last_mut()?.1 = true,
            _ => {}
        }
    }
    if !groups.is_empty() || in_class || ranges.is_empty() {
        return None;
    }
    Some(ranges)
}

fn prepare_units(
    source: &[u16],
    dot_all: bool,
    input_assertions: bool,
) -> Option<PreparedSequence> {
    prepare_units_with_choices(source, dot_all, input_assertions, HashMap::new())
}

fn prepare_units_with_choices(
    source: &[u16],
    dot_all: bool,
    input_assertions: bool,
    mut choices: HashMap<usize, (usize, PreparedCharacter, usize)>,
) -> Option<PreparedSequence> {
    let mut index = 0;
    let mut terms = Vec::new();
    let mut captures = Vec::new();
    let mut groups = Vec::new();
    let mut boundaries: Vec<(usize, Assertions)> = Vec::new();
    while let Some(&unit) = source.get(index) {
        if input_assertions && (unit == 94 || unit == 36) {
            let position = terms.len();
            if boundaries
                .last()
                .is_none_or(|(offset, _)| *offset != position)
            {
                boundaries.push((position, Assertions::default()));
            }
            boundaries.last_mut()?.1.add_input_boundary(unit == 94);
            index += 1;
        } else if unit == 92
            && source
                .get(index + 1)
                .is_some_and(|&unit| unit == 98 || unit == 66)
        {
            let position = terms.len();
            if boundaries
                .last()
                .is_none_or(|(offset, _)| *offset != position)
            {
                boundaries.push((position, Assertions::default()));
            }
            boundaries
                .last_mut()?
                .1
                .add_word_boundary(source[index + 1] == 98);
            index += 2;
        } else if unit == u16::from(b'(') {
            if let Some((end, plan, count)) = choices.remove(&index) {
                let offset = terms.len();
                captures.extend(std::iter::repeat_n(offset..offset + 1, count));
                terms.push(PreparedTerm::Set {
                    plan,
                    source: index..end,
                });
                index = end;
                continue;
            }
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
    groups.is_empty().then_some(PreparedSequence {
        terms,
        captures,
        boundaries,
    })
}

pub(crate) fn unit_choice_atom(units: &[u16], dot_all: bool) -> Option<Vec<u16>> {
    // Capture-free one-unit branches have identical endpoints and no
    // branch-specific slots. One ordinary set preserves their union predicate.
    if units.first() != Some(&40) || units.last() != Some(&41) {
        return None;
    }
    let source = JsString::from_code_units(units.to_vec());
    let group = crate::regexp_outer_group_body(&source)?;
    let end = group.body.end;
    let mut index = group.body.start;
    let mut body = Vec::new();
    let mut branches = 0usize;
    while index < end {
        if let Some((prepared, width)) = PreparedCharacter::parse(&units[index..end], dot_all) {
            prepared.append_union_body(&mut body)?;
            index += width;
        } else {
            let unit = units[index];
            index += 1;
            let character = if unit == 92 {
                crate::regexp_literal::character_escape(units, &mut index)?
            } else if crate::regexp_literal::is_syntax(unit) {
                return None;
            } else {
                unit
            };
            crate::regexp_character::append_class_unit(&mut body, character);
        }
        branches += 1;
        if index == end {
            break;
        }
        if units.get(index) != Some(&124) {
            return None;
        }
        index += 1;
        if index == end {
            return None;
        }
    }
    if branches < 2 {
        return None;
    }
    let mut atom = if group.captures == 0 {
        vec![40, 63, 58]
    } else {
        vec![40; group.captures]
    };
    atom.push(91);
    atom.extend(body);
    atom.push(93);
    atom.extend(std::iter::repeat_n(41, group.captures.max(1)));
    Some(atom)
}

pub(crate) fn unit_choice_plan(units: &[u16], dot_all: bool) -> Option<(PreparedCharacter, usize)> {
    let source = JsString::from_code_units(units.to_vec());
    let group = crate::regexp_outer_group_body(&source)?;
    let mut index = group.body.start;
    let end = group.body.end;
    let mut parts = Vec::new();
    while index < end {
        if let Some((part, consumed)) = PreparedCharacter::parse(&units[index..end], dot_all) {
            parts.push(part);
            index += consumed;
        } else {
            let unit = units[index];
            index += 1;
            let unit = if unit == 92 {
                crate::regexp_literal::character_escape(units, &mut index)?
            } else if crate::regexp_literal::is_syntax(unit) {
                return None;
            } else {
                unit
            };
            parts.push(PreparedCharacter::literal(unit));
        }
        if index == end {
            break;
        }
        if units.get(index) != Some(&124) {
            return None;
        }
        index += 1;
        if index == end {
            return None;
        }
    }
    if parts.len() < 2 {
        return None;
    }
    Some((
        PreparedCharacter::union(parts, units.len())?,
        group.captures,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fmt::Write;

    type UnitPredicate = fn(u16) -> bool;

    #[test]
    fn ordinary_inverted_unit_choices_snapshot() {
        let mut rows = String::new();
        for source in [
            "([^a]|b)",
            "(a|[^b])",
            "([^a]|a)",
            "([^a]|[^b])",
            "([^a]|[^a])",
            "([^]|a)",
            "([^]|[])",
            "([]|[^])",
            "([^a]|[])",
            "([^a-z]|[A])",
            "([^µ]|[Μ])",
            "([^ſ]|S)",
            "([^σ]|[ς])",
            r"([^\w]|\w)",
            r"([^\D]|[1])",
            r"([^\W]|\d)",
            r"([^\s]|\s)",
            r"([^\S]|[\n])",
            r"([^\n]|.)",
            r"([^\r\n\u2028\u2029]|.)",
            r"([^\uD800]|\uDC00)",
            "x([^a]|b)y",
            "(x([^a]|b)y)",
            "(x)([^a]|b)(y)",
            "([^a]|b)([^c]|d)",
            "(([^a]|b))",
            "(?:(?:[^a]|b))",
            "((?:[^a]|b))",
            "(?:[^a]|b)()",
            r"\0()([^a]|b)1",
            r"\b([^a]|b)\b",
            r"([^a]|b)(\B)",
            "^([^a]|b)$",
            "(^)([^a]|b)($)",
            r"([^a]|b)$\n^c",
            "([^a]|b)()",
            "([^a]|bc)",
            "(ab|[^c])",
            "([^a]|)",
            "(|[^b])",
            "(([^a])|b)",
            "([^a]|(b))",
            "([^a]|b)+",
            "(x([^a]|b))+",
            "([^a]|b)|x",
            "(([^a]|b)|c)",
            "(?<n>[^a]|b)",
            "(?i:[^a]|b)",
            "(?=[^a]|b)",
            r"([^a]|b)\1",
            "([^a]|b)[",
            "([^a]|b",
        ] {
            for (i, m, s) in [
                (false, false, false),
                (true, false, false),
                (false, true, false),
                (false, false, true),
            ] {
                write!(rows, "{source:?} i={i} m={m} s={s}").unwrap();
                let matcher = RegExpSequenceMatcher::compile_with_assertions_and_work(
                    &JsString::from(source),
                    i,
                    m,
                    s,
                    |_| Ok::<(), ()>(()),
                )
                .unwrap();
                if let Some(matcher) = matcher {
                    write!(
                        rows,
                        " width={} captures={:?}",
                        matcher.atom_count(),
                        matcher.capture_ranges()
                    )
                    .unwrap();
                    for text in [
                        "",
                        "a",
                        "b",
                        "ab",
                        "xabcy",
                        "xay",
                        "xby",
                        "ac",
                        "bd",
                        "AB",
                        "µΜ",
                        "ſSσς",
                        "a\nc",
                        "\ra\r\n",
                        "\0a1",
                        "()[]|?",
                        "surrogates",
                    ] {
                        let input = if text == "surrogates" {
                            JsString::from_code_units(vec![0xd800, 0xdc00])
                        } else {
                            JsString::from(text)
                        };
                        for (start, sticky) in [
                            (0, false),
                            (1, false),
                            (0, true),
                            (1, true),
                            (input.len(), true),
                        ] {
                            let found = matcher.find(&input, start, sticky).map(|r| {
                                let captures = matcher
                                    .capture_ranges()
                                    .iter()
                                    .map(|c| Some(r.start + c.start..r.start + c.end))
                                    .collect::<Vec<_>>();
                                (r, captures)
                            });
                            write!(rows, " {input:?}@{start}/{sticky}:{found:?}").unwrap();
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
    fn inverted_choice_membership_agrees_with_canonicalize_before_inversion() {
        for ignore_case in [false, true] {
            let canon = |unit| canonicalize(unit, ignore_case);
            let cases: [(&str, usize); 8] = [
                ("([^a]|b)", 0),
                ("([^a]|a)", 1),
                ("([^a]|[^b])", 1),
                (r"([^\w]|\w)", 1),
                (r"([^\D]|[1])", 2),
                ("([^µ]|[Μ])", 3),
                ("([^a-z]|[A])", 4),
                ("([^ſ]|S)", 5),
            ];
            for (source, kind) in cases {
                let matcher =
                    RegExpSequenceMatcher::compile(&JsString::from(source), ignore_case, false)
                        .unwrap();
                assert_eq!(
                    matcher.capture_ranges(),
                    std::iter::once(0..1).collect::<Vec<_>>()
                );
                for unit in 0..=u16::MAX {
                    let c = canon(unit);
                    let expected = match kind {
                        0 => c != canon(97) || c == canon(98),
                        1 => true,
                        2 => matches!(c, 48..=57),
                        3 => c != canon(0xb5) || c == canon(0x39c),
                        4 => !(97..=122).any(|u| canon(u) == c) || c == canon(65),
                        5 => c != canon(0x17f) || c == canon(83),
                        _ => unreachable!(),
                    };
                    assert_eq!(
                        matcher.find(&JsString::from_code_units(vec![unit]), 0, true),
                        expected.then_some(0..1),
                        "{source} i={ignore_case} {unit:#06x}"
                    );
                }
            }
        }
    }

    #[test]
    fn deep_inverted_choices_share_flat_predicates_and_keep_optional_work_fallible() {
        let source = JsString::from(
            format!("{}([^a]|b){}", "(".repeat(100_000), ")".repeat(100_000)).as_str(),
        );
        let matcher = RegExpSequenceMatcher::compile(&source, true, false).unwrap();
        assert_eq!(matcher.capture_ranges().len(), 100_001);
        assert!(matcher.capture_ranges().iter().all(|r| *r == (0..1)));
        assert_eq!(
            matcher.clone().find(&JsString::from("b"), 0, true),
            Some(0..1)
        );
        assert_eq!(matcher.find(&JsString::from("A"), 0, true), None);
        let source = JsString::from("([^a]|b)".repeat(10_000).as_str());
        let mut remaining = 600_000usize;
        let matcher = RegExpSequenceMatcher::compile_with_work(&source, false, false, |work| {
            remaining = remaining.checked_sub(work).ok_or("abort")?;
            Ok::<(), &str>(())
        })
        .unwrap()
        .unwrap();
        assert_eq!(matcher.capture_ranges()[9999], 9999..10_000);
        assert_eq!(
            matcher.find(&JsString::from("b".repeat(10_000).as_str()), 0, true),
            Some(0..10_000)
        );
        let source = JsString::from(format!("({}[^a])", "[^a]|".repeat(1000)).as_str());
        let matcher = RegExpSequenceMatcher::compile(&source, false, false).unwrap();
        assert_eq!(matcher.find(&JsString::from("b"), 0, true), Some(0..1));
        assert_eq!(matcher.find(&JsString::from("a"), 0, true), None);
        assert!(matches!(
            RegExpSequenceMatcher::compile_with_work(
                &JsString::from("([^a]|b)"),
                false,
                false,
                |work| if work == 1024 { Err("abort") } else { Ok(()) }
            ),
            Err("abort")
        ));
        for source in ["([^a]|bc)", "([^a]|b)c+", "(([^a])|b)", "([^a]|b)|c"] {
            assert!(
                RegExpSequenceMatcher::compile_with_work(
                    &JsString::from(source),
                    false,
                    false,
                    |_| Err::<(), _>("unexpected charge")
                )
                .unwrap()
                .is_none()
            );
        }
    }

    #[test]
    fn ordinary_character_unit_choices_snapshot() {
        let mut rows = String::new();
        for source in [
            "(a|[b])",
            "([a]|b)",
            "([a-z]|[0-9])",
            "([a-]|[-b])",
            "([a]|[-b])",
            "([a-]|[b-c])",
            r"([\[\]]|[\-])",
            r"([\b]|b)",
            "(.|a)",
            "(a|.)",
            r"(.|[\n])",
            "(?:.|[a])",
            "(.|.)",
            r"(\d|a)",
            r"(\D|a)",
            r"(\w|\d)",
            r"(\W|[a-z])",
            r"(\s|a)",
            r"(\S|[\n])",
            r"([\dA-Z]|[a-z])",
            r"([\D]|[0])",
            r"([\s\S]|a)",
            "([]|a)",
            "(a|[])",
            "([]|[])",
            "([]|.)",
            "([µ]|[Μ])",
            "([ſ]|S)",
            "([σ]|[ς])",
            r"([\uD800]|\uDC00)",
            "x([ab]|c)y",
            "(x([ab]|c)y)",
            "([ab]|c)([de]|f)",
            "((?:[ab]|c))()",
            "((a|[b]))",
            r"\0()([ab]|c)1",
            r"([a-]|[-b])()(c|\d)",
            r"\b([a-z]|[0-9])\b",
            "^([ab]|c)$",
            r"([ab]|c)$\n^d",
            "([ab]|c)(^)",
            "(.|[a])()",
            "([^a]|b)",
            "(a|[^b])",
            "([^a]|[^b])",
            "([^]|a)",
            "([ab]|cd)",
            "([ab]|)",
            "(|[ab])",
            "(([a])|b)",
            "([a]|(b))",
            "([ab]|c)+",
            "(x([ab]|c))+",
            "([ab]|c)|x",
            "(?<n>[ab]|c)",
            "(?i:[ab]|c)",
            r"([ab]|c)\1",
            "([ab]|c)[",
            "([ab]|c",
            "([ab]|c+)",
        ] {
            for (i, m, s) in [
                (false, false, false),
                (true, false, false),
                (false, true, false),
                (false, false, true),
            ] {
                write!(rows, "{source:?} i={i} m={m} s={s}").unwrap();
                let matcher = RegExpSequenceMatcher::compile_with_assertions_and_work(
                    &JsString::from(source),
                    i,
                    m,
                    s,
                    |_| Ok::<(), ()>(()),
                )
                .unwrap();
                if let Some(matcher) = matcher {
                    write!(
                        rows,
                        " width={} captures={:?}",
                        matcher.atom_count(),
                        matcher.capture_ranges()
                    )
                    .unwrap();
                    for text in [
                        "",
                        "a",
                        "b",
                        "ab",
                        "xabcy",
                        "xay",
                        "xby",
                        "ac",
                        "bd",
                        "AB",
                        "µΜ",
                        "ſSσς",
                        "a\nc",
                        "\ra\r\n",
                        "\0a1",
                        "()[]|?",
                        "surrogates",
                    ] {
                        let input = if text == "surrogates" {
                            JsString::from_code_units(vec![0xd800, 0xdc00])
                        } else {
                            JsString::from(text)
                        };
                        for (start, sticky) in [
                            (0, false),
                            (1, false),
                            (0, true),
                            (1, true),
                            (input.len(), true),
                        ] {
                            let found = matcher.find(&input, start, sticky).map(|r| {
                                let captures = matcher
                                    .capture_ranges()
                                    .iter()
                                    .map(|c| Some(r.start + c.start..r.start + c.end))
                                    .collect::<Vec<_>>();
                                (r, captures)
                            });
                            write!(rows, " {input:?}@{start}/{sticky}:{found:?}").unwrap();
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
    fn ordinary_unit_choice_unions_agree_with_all_utf16_predicates() {
        let cases: [(&str, UnitPredicate); 9] = [
            ("(a|[b-c])", |u| matches!(u, 97..=99)),
            ("([a-]|[-b])", |u| matches!(u, 45 | 97 | 98)),
            (r"(\d|a)", |u| matches!(u, 48..=57 | 97)),
            (r"(\D|[1])", |u| !(48..=57).contains(&u) || u == 49),
            (r"(\w|\d)", |u| matches!(u,48..=57|65..=90|95|97..=122)),
            (r"(\W|a)", |u| {
                !matches!(u,48..=57|65..=90|95|97..=122) || u == 97
            }),
            ("([]|a)", |u| u == 97),
            ("([]|[])", |_| false),
            (r"([\uD800]|\uDC00)", |u| matches!(u, 0xd800 | 0xdc00)),
        ];
        for (source, predicate) in cases {
            let matcher =
                RegExpSequenceMatcher::compile(&JsString::from(source), false, false).unwrap();
            assert_eq!(
                matcher.capture_ranges(),
                std::iter::once(0..1).collect::<Vec<_>>()
            );
            for unit in 0..=u16::MAX {
                let input = JsString::from_code_units(vec![unit]);
                assert_eq!(
                    matcher.find(&input, 0, true),
                    predicate(unit).then_some(0..1),
                    "{source} {unit:#06x}"
                );
            }
        }
        for dot_all in [false, true] {
            let matcher =
                RegExpSequenceMatcher::compile(&JsString::from(r"(.|[\n])"), false, dot_all)
                    .unwrap();
            for unit in 0..=u16::MAX {
                let expected = dot_all || !matches!(unit, 13 | 0x2028 | 0x2029);
                assert_eq!(
                    matcher.find(&JsString::from_code_units(vec![unit]), 0, true),
                    expected.then_some(0..1),
                    "dot_all={dot_all} {unit:#06x}"
                );
            }
        }
    }

    #[test]
    fn class_choice_preparation_shares_predicates_and_retains_deep_capture_boundaries() {
        let source = JsString::from(
            format!("{}([a-]|[-b]){}", "(".repeat(100_000), ")".repeat(100_000)).as_str(),
        );
        let matcher = RegExpSequenceMatcher::compile(&source, false, false).unwrap();
        assert_eq!(matcher.capture_ranges().len(), 100_001);
        assert!(matcher.capture_ranges().iter().all(|r| *r == (0..1)));
        assert_eq!(
            matcher.clone().find(&JsString::from("-"), 0, true),
            Some(0..1)
        );
        let source = JsString::from(r"(\w|\d)".repeat(10_000).as_str());
        let mut constructions = 0;
        let matcher = RegExpSequenceMatcher::compile_with_work(&source, false, false, |work| {
            constructions += usize::from(work == 1024);
            Ok::<(), ()>(())
        })
        .unwrap()
        .unwrap();
        assert_eq!(constructions, 1);
        assert_eq!(matcher.capture_ranges()[9999], 9999..10_000);
        assert_eq!(
            matcher.find(&JsString::from("a".repeat(10_000).as_str()), 0, true),
            Some(0..10_000)
        );
        let matcher = RegExpSequenceMatcher::compile(
            &JsString::from(format!("({})", r"[a-]|".repeat(100_000) + "[-b]").as_str()),
            false,
            false,
        )
        .unwrap();
        assert_eq!(matcher.find(&JsString::from("-"), 0, true), Some(0..1));
        assert_eq!(matcher.find(&JsString::from("c"), 0, true), None);
        assert!(matches!(
            RegExpSequenceMatcher::compile_with_work(
                &JsString::from("([a]|b)"),
                false,
                false,
                |work| if work == 1024 { Err("abort") } else { Ok(()) }
            ),
            Err("abort")
        ));
        for source in ["([^a]|bc)", "([a]|bc)", "([a]|b)+c+", "(([a])|b)"] {
            assert!(
                RegExpSequenceMatcher::compile_with_work(
                    &JsString::from(source),
                    false,
                    false,
                    |_| Err::<(), _>("unexpected charge")
                )
                .unwrap()
                .is_none()
            );
        }
    }

    #[test]
    fn fixed_literal_unit_choices_snapshot() {
        let mut rows = String::new();
        for source in [
            "(a|b)",
            "(?:a|b)",
            "((a|b))",
            "(?:(a|b))",
            "((?:a|b))",
            "x(a|b)y",
            "(x(a|b)y)",
            "(x)(a|b)(y)",
            "x((a|b)y)",
            "(a|b)(c|d)",
            "((a|b)(c|d))",
            "(?:a|b)()(?:c|d)",
            "(a|a|b|a)",
            "(µ|Μ)",
            "(ſ|S)",
            "(σ|ς)",
            r"(\x61|\u0062)",
            r"(\0|a)()1",
            r"\0()(a|b)1",
            r"(\(|\))",
            r"(\[|\])",
            r"(\||\?)",
            r"(\n|\r)",
            r"(\cA|\x01)",
            r"(\uD800|\uDC00)",
            r".(a|b).",
            "[ab](a|b)[^c]",
            r"\b(a|b)\b",
            r"(\b(a|b))",
            r"(a|b)(\B)",
            "^(a|b)$",
            "(^)(a|b)($)",
            "((^)(a|b)($))",
            r"(a|b)$\n^c",
            r"(\r)(^)(a|b)",
            r"(a|b)($)(\r)(\n)",
            "(a|b)()",
            "(a|bc)",
            "(ab|c)",
            "(a|)",
            "(|b)",
            "(a|(b))",
            "((a)|b)",
            "(a|[b])",
            "([a]|b)",
            "(.|a)",
            r"(\w|a)",
            r"(\b|a)",
            "(a|b)+",
            "x(a|b)+y",
            "(x(a|b))+",
            "(a|b)|x",
            "((a|b)|c)",
            "(?<n>a|b)",
            "(?i:a|b)",
            "(?=a|b)",
            r"(a|b)\1",
            "(a|b",
            "(a|b)[",
        ] {
            for (i, m, s) in [
                (false, false, false),
                (true, false, false),
                (false, true, false),
                (false, false, true),
            ] {
                write!(rows, "{source:?} i={i} m={m} s={s}").unwrap();
                let matcher = RegExpSequenceMatcher::compile_with_assertions_and_work(
                    &JsString::from(source),
                    i,
                    m,
                    s,
                    |_| Ok::<(), ()>(()),
                )
                .unwrap();
                if let Some(matcher) = matcher {
                    write!(
                        rows,
                        " width={} captures={:?}",
                        matcher.atom_count(),
                        matcher.capture_ranges()
                    )
                    .unwrap();
                    for text in [
                        "",
                        "a",
                        "b",
                        "ab",
                        "xabcy",
                        "xay",
                        "xby",
                        "ac",
                        "bd",
                        "AB",
                        "µΜ",
                        "ſSσς",
                        "a\nc",
                        "\ra\r\n",
                        "\0a1",
                        "()[]|?",
                        "surrogates",
                    ] {
                        let input = if text == "surrogates" {
                            JsString::from_code_units(vec![0xd800, 0xdc00])
                        } else {
                            JsString::from(text)
                        };
                        for (start, sticky) in [
                            (0, false),
                            (1, false),
                            (0, true),
                            (1, true),
                            (input.len(), true),
                        ] {
                            let found = matcher.find(&input, start, sticky).map(|r| {
                                let captures = matcher
                                    .capture_ranges()
                                    .iter()
                                    .map(|c| Some(r.start + c.start..r.start + c.end))
                                    .collect::<Vec<_>>();
                                (r, captures)
                            });
                            write!(rows, " {input:?}@{start}/{sticky}:{found:?}").unwrap();
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
    fn fixed_choice_candidates_and_captures_agree_with_independent_unit_predicates() {
        let alphabet = [97, 98, 99, 120, 10, 0xd800];
        for (source, width, captures) in [
            ("(a|b)", 1, std::iter::once(0..1).collect()),
            ("((a|b))(a|b)", 2, vec![0..1, 0..1, 1..2]),
            ("x(a|b)(c|x)", 3, vec![1..2, 2..3]),
        ] {
            let matcher =
                RegExpSequenceMatcher::compile(&JsString::from(source), false, false).unwrap();
            assert_eq!(matcher.capture_ranges(), captures);
            for length in 0..=4u32 {
                for mut encoded in 0..alphabet.len().pow(length) {
                    let mut units = Vec::new();
                    for _ in 0..length {
                        units.push(alphabet[encoded % alphabet.len()]);
                        encoded /= alphabet.len();
                    }
                    let input = JsString::from_code_units(units.clone());
                    for start in 0..=units.len() + 1 {
                        for sticky in [false, true] {
                            let expected = (start..=units.len()).find_map(|at| {
                                if sticky && at != start {
                                    return None;
                                }
                                let candidate = units.get(at..at + width)?;
                                let accepts = if width == 3 {
                                    candidate[0] == 120
                                        && matches!(candidate[1], 97 | 98)
                                        && matches!(candidate[2], 99 | 120)
                                } else {
                                    candidate.iter().all(|u| matches!(u, 97 | 98))
                                };
                                accepts.then_some(at..at + width)
                            });
                            assert_eq!(
                                matcher.find(&input, start, sticky),
                                expected,
                                "{source} {units:?} {start} {sticky}"
                            );
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn fixed_choice_normalization_stays_linear_and_sets_share_only_predicates() {
        let source =
            JsString::from(format!("{}(a|b){}", "(".repeat(100_000), ")".repeat(100_000)).as_str());
        let matcher = RegExpSequenceMatcher::compile(&source, false, false).unwrap();
        assert_eq!(matcher.capture_ranges().len(), 100_001);
        assert!(matcher.capture_ranges().iter().all(|r| *r == (0..1)));
        assert_eq!(
            matcher.clone().find(&JsString::from("b"), 0, true),
            Some(0..1)
        );
        let source = JsString::from("(a|b)".repeat(10_000).as_str());
        let mut constructions = 0;
        let matcher = RegExpSequenceMatcher::compile_with_work(&source, false, false, |work| {
            constructions += usize::from(work == 1024);
            Ok::<(), ()>(())
        })
        .unwrap()
        .unwrap();
        assert_eq!(constructions, 1);
        assert_eq!(matcher.atom_count(), 10_000);
        assert_eq!(matcher.capture_ranges()[9999], 9999..10_000);
        assert_eq!(
            matcher.find(&JsString::from("b".repeat(10_000).as_str()), 0, true),
            Some(0..10_000)
        );
        assert!(matches!(
            RegExpSequenceMatcher::compile_with_work(
                &JsString::from("(a|b)"),
                false,
                false,
                |_| Err::<(), _>("abort")
            ),
            Err("abort")
        ));
        for source in ["(a|b)[a]*", "(a|bc)", "((a)|b)", "(a|b)|c"] {
            assert!(
                RegExpSequenceMatcher::compile_with_work(
                    &JsString::from(source),
                    false,
                    false,
                    |_| Err::<(), _>("unexpected charge")
                )
                .unwrap()
                .is_none()
            );
        }
        let matcher =
            RegExpSequenceMatcher::compile(&JsString::from(r"\0()(a|b)1"), false, false).unwrap();
        assert_eq!(matcher.capture_ranges(), [1..1, 1..2]);
        assert_eq!(
            matcher.find(&JsString::from_code_units(vec![0, 98, 49]), 0, true),
            Some(0..3)
        );
    }

    #[test]
    fn fixed_input_line_assertion_snapshot() {
        let mut rows = String::new();
        for source in [
            "a^b",
            "a$b",
            "(^a)",
            "(a$)",
            "(a)(^)(b)",
            "(a)($)(b)",
            r"a$\n^b",
            r"(a)($)(\n)(^)(b)",
            r"([\s\S])(^)(.)",
            r"([\s\S])($)(.)",
            r"(\r)(^)(\n)(^)(a)",
            r"(a)($)(\r)($)(\n)",
            r"(.)(^)(.)",
            r"(.)(\b^)(a)",
            r"(a)($\b)(.)",
            r"(\r)(^\b)(a)",
            "(^)()",
            "($)()",
            "(^$)()",
            r"(\B^$)()",
            "(^^)(a)($$)",
            "^((^a))$",
            "^((a$))$",
            r"(\^)(a)(\$)",
            r"([\^$])(^)(a)",
            r"([\b])(^)(a)",
            r"(\uD800)(^)(a)",
            r"(\uD800)($)(\uDC00)",
            r"(^\b)([a-z])($\b)",
            r"(a$)+",
            r"a^b+",
            r"a+($)",
            r"(a|b)^",
            r"(?<n>a)^",
            r"(a)\1^",
        ] {
            for (i, m, s) in [
                (false, false, false),
                (false, true, false),
                (true, true, false),
                (false, false, true),
                (false, true, true),
            ] {
                let matcher = RegExpSequenceMatcher::compile_with_assertions_and_work(
                    &JsString::from(source),
                    i,
                    m,
                    s,
                    |_| Ok::<(), ()>(()),
                )
                .unwrap();
                write!(rows, "{source:?} i={i} m={m} s={s}").unwrap();
                if let Some(matcher) = matcher {
                    write!(
                        rows,
                        " captures={} width={} passes={}/{}",
                        matcher.capture_ranges().len(),
                        matcher.atom_count(),
                        matcher.search_passes(false),
                        matcher.search_passes(true)
                    )
                    .unwrap();
                    for input in [
                        "",
                        "a",
                        "A",
                        "ab",
                        "a\nb",
                        "x\na\ny",
                        "\na",
                        "a\n",
                        "\ra",
                        "a\r\n",
                        "\r\na",
                        "\u{2028}a",
                        "a\u{2029}",
                        "^a$",
                        "💩a",
                        "\u{8}a",
                    ] {
                        let input = JsString::from(input);
                        let matches: Vec<_> = [
                            (0, false),
                            (1, false),
                            (0, true),
                            (1, true),
                            (input.len(), true),
                        ]
                        .into_iter()
                        .map(|(start, sticky)| {
                            matcher.find(&input, start, sticky).map(|r| {
                                let captures: Vec<_> = matcher
                                    .capture_ranges()
                                    .iter()
                                    .map(|c| Some(r.start + c.start..r.start + c.end))
                                    .collect();
                                (r, captures)
                            })
                        })
                        .collect();
                        write!(rows, " {input:?}:{matches:?}").unwrap();
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
    fn input_line_offsets_agree_with_independent_position_oracle() {
        fn accepts(assertion: &str, units: &[u16], p: usize, m: bool) -> bool {
            let lt = |c: u16| [10, 13, 0x2028, 0x2029].contains(&c);
            match assertion {
                "^" => p == 0 || (m && lt(units[p - 1])),
                "$" => p == units.len() || (m && lt(units[p])),
                "" => true,
                _ => unreachable!(),
            }
        }
        for before in ["", "^", "$"] {
            for middle in ["", "^", "$"] {
                for after in ["", "^", "$"] {
                    for m in [false, true] {
                        let source = format!(r"({before})([\s\S])({middle})([\s\S])({after})()");
                        let matcher = RegExpSequenceMatcher::compile_with_assertions_and_work(
                            &JsString::from(source.as_str()),
                            false,
                            m,
                            false,
                            |_| Ok::<(), ()>(()),
                        )
                        .unwrap()
                        .unwrap();
                        assert_eq!(
                            matcher.capture_ranges(),
                            [0..0, 0..1, 1..1, 1..2, 2..2, 2..2]
                        );
                        let alphabet = [97, 10, 13, 0x2028, 0xd800];
                        for length in 0..=4u32 {
                            for mut n in 0..alphabet.len().pow(length) {
                                let units: Vec<_> = (0..length)
                                    .map(|_| {
                                        let c = alphabet[n % alphabet.len()];
                                        n /= alphabet.len();
                                        c
                                    })
                                    .collect();
                                let input = JsString::from_code_units(units.clone());
                                for start in 0..=units.len() + 1 {
                                    for sticky in [false, true] {
                                        let expected = (start..units.len().saturating_sub(1))
                                            .filter(|&p| !sticky || p == start)
                                            .find(|&p| {
                                                accepts(before, &units, p, m)
                                                    && accepts(middle, &units, p + 1, m)
                                                    && accepts(after, &units, p + 2, m)
                                            })
                                            .map(|p| p..p + 2);
                                        assert_eq!(
                                            matcher.find(&input, start, sticky),
                                            expected,
                                            "{source} {units:?} m={m} {start} {sticky}"
                                        );
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
    fn repeated_input_assertions_collapse_with_shared_captures_and_explicit_flags() {
        let source = JsString::from(format!("a({})", "$".repeat(100_000)).as_str());
        let plain = RegExpSequenceMatcher::compile_with_assertions_and_work(
            &source,
            false,
            false,
            false,
            |_| Ok::<(), ()>(()),
        )
        .unwrap()
        .unwrap();
        let multiline = RegExpSequenceMatcher::compile_with_assertions_and_work(
            &source,
            false,
            true,
            false,
            |_| Ok::<(), ()>(()),
        )
        .unwrap()
        .unwrap()
        .clone();
        let input = JsString::from("a\n");
        assert_eq!(plain.find(&input, 0, true), None);
        assert_eq!(multiline.find(&input, 0, true), Some(0..1));
        assert_eq!(multiline.capture_ranges().len(), 1);
        assert_eq!(multiline.capture_ranges().first(), Some(&(1..1)));
        assert_eq!(multiline.search_passes(false), 3);
        assert_eq!(multiline.search_passes(true), 3);
        assert!(RegExpSequenceMatcher::compile(&source, false, false).is_none());
        let source = JsString::from(format!("({})a", "^".repeat(100_000)).as_str());
        let matcher = RegExpSequenceMatcher::compile_with_assertions_and_work(
            &source,
            false,
            true,
            false,
            |_| Ok::<(), ()>(()),
        )
        .unwrap()
        .unwrap();
        assert_eq!(matcher.find(&JsString::from("x\na"), 2, true), Some(2..3));
        assert_eq!(matcher.find(&JsString::from("xaa"), 1, true), None);
        assert_eq!(matcher.find(&JsString::from("a"), usize::MAX, false), None);
    }

    #[test]
    fn fixed_word_assertion_snapshot() {
        let mut rows = String::new();
        for source in [
            r"\b",
            r"\B",
            r"\b\B",
            r"a\Bb",
            r"a\bb",
            r"-\ba\b-",
            r"(a)(\B)(b)",
            r"((a\B)b)()",
            r"(\b)()a",
            r"a(\b)",
            r"a(\B)",
            r"a(\B\B)b",
            r"a(\b\B)b",
            r"(?:a\B)(b)",
            r"\b(foo)\b",
            r"([a]\B)([b])",
            r"([a-z]\B)([0-9_])",
            r"([^a]\b)(a)",
            r"(µ\b)(a)",
            r"(ſ\b)(a)",
            r"(K\b)(a)",
            r"(.\b)(a)",
            r"([^]\b)(a)",
            r"(\uD800\B)(\uDC00)",
            r"(\uD800\b)(a)",
            r"(\\b\B)(a)",
            r"([\b]\b)(a)",
            r"a+(\B)b",
            r"a\Bb+",
            r"^a\Bb$",
            r"(a|b)\B",
            r"(?<n>a)\B",
            r"(a)\1\B",
        ] {
            for (i, s) in [(false, false), (true, false), (false, true)] {
                let matcher = RegExpSequenceMatcher::compile(&JsString::from(source), i, s);
                write!(rows, "{source:?} i={i} m=false s={s}").unwrap();
                if let Some(matcher) = matcher {
                    write!(
                        rows,
                        " captures={} width={} passes={}/{}",
                        matcher.capture_ranges().len(),
                        matcher.atom_count(),
                        matcher.search_passes(false),
                        matcher.search_passes(true)
                    )
                    .unwrap();
                    for input in [
                        "", "a", "aa", "ab", "Ab", "-a-", "a0", "a_", "µa", "ſa", "Ka", "💩a",
                        " foo ", "a\nb", "\u{8}a", "\\ba",
                    ] {
                        let input = JsString::from(input);
                        let matches: Vec<_> = [
                            (0, false),
                            (1, false),
                            (0, true),
                            (1, true),
                            (input.len(), true),
                        ]
                        .into_iter()
                        .map(|(start, sticky)| {
                            matcher.find(&input, start, sticky).map(|r| {
                                let captures: Vec<_> = matcher
                                    .capture_ranges()
                                    .iter()
                                    .map(|c| Some(r.start + c.start..r.start + c.end))
                                    .collect();
                                (r, captures)
                            })
                        })
                        .collect();
                        write!(rows, " {input:?}:{matches:?}").unwrap();
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
    fn fixed_assertion_offsets_and_captures_agree_with_independent_neighbor_oracle() {
        fn word(c: u16) -> bool {
            c == 95 || (48..=57).contains(&c) || (65..=90).contains(&c) || (97..=122).contains(&c)
        }
        fn boundary(u: &[u16], p: usize) -> bool {
            (p > 0 && word(u[p - 1])) != (p < u.len() && word(u[p]))
        }
        for before in [r"\b", r"\B"] {
            for middle in [r"\b", r"\B"] {
                for after in [r"\b", r"\B"] {
                    let source = format!("({before})([ab])({middle})([^b])({after})()");
                    let matcher = RegExpSequenceMatcher::compile(
                        &JsString::from(source.as_str()),
                        false,
                        false,
                    )
                    .unwrap();
                    assert_eq!(
                        matcher.capture_ranges(),
                        [0..0, 0..1, 1..1, 1..2, 2..2, 2..2]
                    );
                    let alphabet = [97, 98, 32, 0xd800];
                    for length in 0..=5u32 {
                        for mut n in 0..alphabet.len().pow(length) {
                            let units: Vec<_> = (0..length)
                                .map(|_| {
                                    let c = alphabet[n % alphabet.len()];
                                    n /= alphabet.len();
                                    c
                                })
                                .collect();
                            let input = JsString::from_code_units(units.clone());
                            for start in 0..=units.len() + 1 {
                                for sticky in [false, true] {
                                    let expected = (start..units.len().saturating_sub(1))
                                        .filter(|&p| !sticky || p == start)
                                        .find(|&p| {
                                            (units[p] == 97 || units[p] == 98)
                                                && units[p + 1] != 98
                                                && boundary(&units, p) == (before == r"\b")
                                                && boundary(&units, p + 1) == (middle == r"\b")
                                                && boundary(&units, p + 2) == (after == r"\b")
                                        })
                                        .map(|p| p..p + 2);
                                    assert_eq!(
                                        matcher.find(&input, start, sticky),
                                        expected,
                                        "{source} {units:?} {start} {sticky}"
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn duplicate_assertions_collapse_and_shared_fixed_captures_keep_consuming_width() {
        let source = JsString::from(format!("a{}b", r"\B".repeat(100_000)).as_str());
        let matcher = RegExpSequenceMatcher::compile(&source, false, false)
            .unwrap()
            .clone();
        assert_eq!(matcher.atom_count(), 2);
        assert_eq!(matcher.search_passes(false), 4);
        assert_eq!(matcher.search_passes(true), 3);
        assert_eq!(matcher.find(&JsString::from("xab"), 0, false), Some(1..3));
        let source = JsString::from(format!("{}([a]\\b)", r"([a]\B)".repeat(999)).as_str());
        let mut remaining = 100_000usize;
        let matcher = RegExpSequenceMatcher::compile_with_work(&source, false, false, |work| {
            remaining = remaining.checked_sub(work).ok_or("abort")?;
            Ok::<(), &str>(())
        })
        .unwrap()
        .unwrap();
        assert_eq!(matcher.atom_count(), 1000);
        assert_eq!(matcher.capture_ranges().len(), 1000);
        assert_eq!(matcher.capture_ranges()[999], 999..1000);
        assert_eq!(matcher.search_passes(false), 3000);
        assert_eq!(matcher.search_passes(true), 2001);
        assert_eq!(
            matcher.find(
                &JsString::from(format!(" {} ", "a".repeat(1000)).as_str()),
                0,
                false
            ),
            Some(1..1001)
        );
        assert_eq!(
            matcher.find(&JsString::from("a".repeat(999).as_str()), 0, false),
            None
        );
        assert_eq!(matcher.find(&JsString::from("ab"), usize::MAX, true), None);
    }

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
