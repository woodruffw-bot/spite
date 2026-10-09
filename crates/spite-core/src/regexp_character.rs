//! One ordinary or proved BMP Unicode CharacterSetMatcher atom (22.2.2.7.1).

use crate::{JsString, is_line_terminator, is_whitespace, regexp_canonicalize_character};
use std::{ops::Range, sync::Arc};

/// Immutable matcher for one ordinary atom or a proved BMP Unicode character set.
///
/// Ordinary compilation requires validation without `u` or `v`; the BMP Unicode
/// compiler requires `u` validation and only admits case-sensitive nonsurrogate
/// BMP membership. Groups, concatenations, alternatives and quantifiers remain
/// unsupported. Ordinary membership canonicalizes before inversion.
#[derive(Clone, Debug)]
pub struct RegExpCharacterMatcher(Arc<Program>);

#[derive(Debug)]
struct Program {
    bits: Box<[u64; 1024]>,
    inverted: bool,
    ignore_case: bool,
}

#[derive(Clone, Copy)]
enum Atom {
    Character(u16),
    Set(u16),
    Range(u16, u16),
    Dot(bool),
}

/// Parsed atom retained until the complete containing Pattern is accepted.
pub(crate) struct PreparedCharacter {
    atoms: Vec<Atom>,
    inverted: bool,
    source_len: usize,
    alternatives: Vec<PreparedAlternative>,
}

struct PreparedAlternative {
    atoms: Vec<Atom>,
    inverted: bool,
}

impl RegExpCharacterMatcher {
    /// Compiles one case-sensitive character-set atom from a validated u Pattern.
    ///
    /// Admitted membership contains only nonsurrogate BMP characters. Each
    /// successful match therefore consumes one Unicode character and one UTF-16
    /// unit; starts and ends cannot split a surrogate pair (22.2.2.7.1, 22.2.7.2).
    /// UnicodeSets syntax, case folding, inverted sets and supplementary values
    /// require separate proofs. The caller must validate the complete u Pattern.
    pub fn compile_bmp_unicode(source: &JsString) -> Option<Self> {
        Self::compile_bmp_unicode_with_work(source, |_| Ok::<(), std::convert::Infallible>(()))
            .unwrap_or_else(|never| match never {})
    }

    /// Compiles the same Unicode subset with fallible opt-in construction work.
    ///
    /// Unsupported atoms are rejected before set construction. Accepted atoms
    /// retain the ordinary compiler's charges and immutable membership storage.
    pub fn compile_bmp_unicode_with_work<E>(
        source: &JsString,
        charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, E> {
        let Some((prepared, end)) = PreparedCharacter::parse(source.code_units(), false) else {
            return Ok(None);
        };
        if end != source.len()
            || prepared.inverted
            || !prepared.atoms.iter().all(|atom| match *atom {
                Atom::Character(unit) => !(0xd800..=0xdfff).contains(&unit),
                Atom::Range(start, end) => !(start <= 0xdfff && end >= 0xd800),
                Atom::Set(kind) => matches!(kind, 0x64 | 0x73 | 0x77),
                Atom::Dot(_) => false,
            })
        {
            return Ok(None);
        }
        prepared.compile_with_work(false, charge).map(Some)
    }

    /// Compiles exactly one character-set atom, without expanding Pattern size.
    pub fn compile(source: &JsString, ignore_case: bool, dot_all: bool) -> Option<Self> {
        Self::compile_with_work(source, ignore_case, dot_all, |_| {
            Ok::<(), std::convert::Infallible>(())
        })
        .unwrap_or_else(|never| match never {})
    }

    /// Compiles the same subset with fallible, opt-in construction work accounting.
    ///
    /// After the linear parse, charges source length and every atom's construction
    /// loop bound before filling the immutable set. Separate charges avoid work
    /// total overflow. Rejected syntax returns Ok(None); a charge failure remains
    /// an independent host error rather than a compilation or matching failure.
    pub fn compile_with_work<E>(
        source: &JsString,
        ignore_case: bool,
        dot_all: bool,
        charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, E> {
        let Some((prepared, end)) = PreparedCharacter::parse(source.code_units(), dot_all) else {
            return Ok(None);
        };
        if end != source.len() {
            return Ok(None);
        }
        prepared.compile_with_work(ignore_case, charge).map(Some)
    }

    /// Finds the first matching UTF-16 unit, or only the requested sticky unit.
    ///
    /// Empty input and an offset at/past its end cannot match a consuming atom.
    /// Lone surrogate values and both halves of pairs remain ordinary units.
    pub fn find(&self, input: &JsString, start: usize, sticky: bool) -> Option<Range<usize>> {
        let units = input.code_units();
        let suffix = units.get(start..)?;
        suffix
            .iter()
            .take(if sticky { 1 } else { suffix.len() })
            .enumerate()
            .find_map(|(offset, &unit)| {
                self.matches(unit)
                    .then_some(start + offset..start + offset + 1)
            })
    }

    pub(crate) fn matches(&self, unit: u16) -> bool {
        let unit = canonicalize(unit, self.0.ignore_case);
        let contains = self.0.bits[usize::from(unit) / 64] & (1u64 << (unit % 64)) != 0;
        contains != self.0.inverted
    }
}

impl PreparedCharacter {
    pub(crate) fn parse(units: &[u16], dot_all: bool) -> Option<(Self, usize)> {
        let (atoms, inverted, source_len) = prepare(units, dot_all)?;
        Some((
            Self {
                atoms,
                inverted,
                source_len,
                alternatives: Vec::new(),
            },
            source_len,
        ))
    }

    pub(crate) fn literal(unit: u16) -> Self {
        Self {
            atoms: vec![Atom::Character(unit)],
            inverted: false,
            source_len: 1,
            alternatives: Vec::new(),
        }
    }

    pub(crate) fn union(parts: Vec<Self>, source_len: usize) -> Option<Self> {
        let mut atoms = Vec::new();
        let mut alternatives = Vec::new();
        for part in parts {
            // Branch parsing produces flat atoms, never nested union programs.
            if !part.alternatives.is_empty() {
                return None;
            }
            if part.inverted {
                alternatives.push(PreparedAlternative {
                    atoms: part.atoms,
                    inverted: true,
                });
            } else {
                atoms.extend(part.atoms);
            }
        }
        Some(Self {
            atoms,
            inverted: false,
            source_len,
            alternatives,
        })
    }

    /// Emits a unionable ordinary class body without fusing range boundaries.
    /// Outer inverted classes need predicate unions and are not flattened here.
    pub(crate) fn append_union_body(&self, output: &mut Vec<u16>) -> Option<()> {
        if self.inverted || !self.alternatives.is_empty() {
            return None;
        }
        for &atom in &self.atoms {
            match atom {
                Atom::Character(unit) => append_class_unit(output, unit),
                Atom::Range(start, end) => append_class_range(output, start, end),
                Atom::Set(kind) => output.extend_from_slice(&[92, kind]),
                Atom::Dot(true) => append_class_range(output, 0, u16::MAX),
                Atom::Dot(false) => {
                    // Dot excludes exactly LF, CR, LS and PS in ordinary mode.
                    for (start, end) in [(0, 9), (11, 12), (14, 0x2027), (0x202a, u16::MAX)] {
                        append_class_range(output, start, end);
                    }
                }
            }
        }
        Some(())
    }

    pub(crate) fn compile_with_work<E>(
        self,
        ignore_case: bool,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<RegExpCharacterMatcher, E> {
        charge(self.source_len)?;
        charge(1024)?;
        for atom in &self.atoms {
            charge(match *atom {
                Atom::Character(_) => 1,
                Atom::Range(start, end) => usize::from(end) - usize::from(start) + 1,
                Atom::Set(_) | Atom::Dot(_) => 65_536,
            })?;
        }
        for alternative in &self.alternatives {
            charge(1024)?; // Clear the reusable temporary predicate.
            charge(1024)?; // Merge its membership after inversion.
            for atom in &alternative.atoms {
                charge(match *atom {
                    Atom::Character(_) => 1,
                    Atom::Range(start, end) => usize::from(end) - usize::from(start) + 1,
                    Atom::Set(_) | Atom::Dot(_) => 65_536,
                })?;
            }
        }
        let mut program = Program {
            bits: Box::new([0; 1024]),
            inverted: self.inverted,
            ignore_case,
        };
        for atom in self.atoms {
            program.add(atom);
        }
        if !self.alternatives.is_empty() {
            let mut branch = Program {
                bits: Box::new([0; 1024]),
                inverted: false,
                ignore_case,
            };
            for alternative in self.alternatives {
                branch.bits.fill(0);
                branch.inverted = alternative.inverted;
                for atom in alternative.atoms {
                    branch.add(atom);
                }
                for (target, &bits) in program.bits.iter_mut().zip(branch.bits.iter()) {
                    *target |= if branch.inverted { !bits } else { bits };
                }
            }
        }
        Ok(RegExpCharacterMatcher(Arc::new(program)))
    }
}

impl Program {
    fn insert(&mut self, unit: u16) {
        let unit = canonicalize(unit, self.ignore_case);
        self.bits[usize::from(unit) / 64] |= 1u64 << (unit % 64);
    }

    fn add(&mut self, atom: Atom) {
        match atom {
            Atom::Character(unit) => self.insert(unit),
            Atom::Range(start, end) => {
                for unit in start..=end {
                    self.insert(unit);
                }
            }
            Atom::Dot(dot_all) => {
                for unit in 0..=u16::MAX {
                    if dot_all || !char::from_u32(u32::from(unit)).is_some_and(is_line_terminator) {
                        self.insert(unit);
                    }
                }
            }
            Atom::Set(kind) => {
                let lower = kind | 0x20;
                let inverted = kind != lower;
                for unit in 0..=u16::MAX {
                    let included = match lower {
                        0x64 => (0x30..=0x39).contains(&unit),
                        0x77 => matches!(unit, 0x30..=0x39 | 0x41..=0x5a | 0x5f | 0x61..=0x7a),
                        0x73 => char::from_u32(u32::from(unit))
                            .is_some_and(|c| is_whitespace(c) || is_line_terminator(c)),
                        _ => unreachable!("validated class escape"),
                    };
                    if included != inverted {
                        self.insert(unit);
                    }
                }
            }
        }
    }
}

pub(crate) fn append_class_unit(output: &mut Vec<u16>, unit: u16) {
    output.extend_from_slice(&[92, 117]);
    for shift in [12, 8, 4, 0] {
        let digit = (unit >> shift) & 15;
        output.push(if digit < 10 { 48 + digit } else { 87 + digit });
    }
}

fn append_class_range(output: &mut Vec<u16>, start: u16, end: u16) {
    append_class_unit(output, start);
    output.push(45);
    append_class_unit(output, end);
}

fn prepare(units: &[u16], dot_all: bool) -> Option<(Vec<Atom>, bool, usize)> {
    if units.first() == Some(&u16::from(b'.')) {
        return Some((vec![Atom::Dot(dot_all)], false, 1));
    }
    if units.len() >= 2 && units[0] == u16::from(b'\\') && is_class_escape(units[1]) {
        return Some((vec![Atom::Set(units[1])], false, 2));
    }
    if units.first() != Some(&u16::from(b'[')) {
        return None;
    }
    let mut index = 1;
    let inverted = units.get(index) == Some(&u16::from(b'^'));
    if inverted {
        index += 1;
    }
    let mut atoms = Vec::new();
    while units.get(index) != Some(&u16::from(b']')) {
        let atom = class_atom(units, &mut index)?;
        if units.get(index) == Some(&u16::from(b'-'))
            && units.get(index + 1) != Some(&u16::from(b']'))
        {
            index += 1;
            let end = class_atom(units, &mut index)?;
            let (Atom::Character(start), Atom::Character(end)) = (atom, end) else {
                return None;
            };
            if start > end {
                return None;
            }
            atoms.push(Atom::Range(start, end));
        } else {
            atoms.push(atom);
        }
    }
    Some((atoms, inverted, index + 1))
}

fn canonicalize(unit: u16, ignore_case: bool) -> u16 {
    regexp_canonicalize_character(u32::from(unit), ignore_case, false) as u16
}

fn is_class_escape(unit: u16) -> bool {
    matches!(unit, 0x64 | 0x44 | 0x73 | 0x53 | 0x77 | 0x57)
}

fn class_atom(source: &[u16], index: &mut usize) -> Option<Atom> {
    let unit = *source.get(*index)?;
    *index += 1;
    if unit == u16::from(b'\\') {
        let escaped = *source.get(*index)?;
        if is_class_escape(escaped) {
            *index += 1;
            Some(Atom::Set(escaped))
        } else if escaped == u16::from(b'b') {
            *index += 1;
            Some(Atom::Character(8))
        } else {
            crate::regexp_literal::character_escape(source, index).map(Atom::Character)
        }
    } else if unit == u16::from(b']') {
        None
    } else {
        Some(Atom::Character(unit))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fmt::Write;

    #[test]
    fn unicode_bmp_character_ranges_snapshot() {
        let mut rows = String::new();
        for source in [
            "[]",
            "[a]",
            "[a-b]",
            "[é]",
            "[éσ]",
            "[K]",
            "[[]",
            r"[\b]",
            r"[\cA\x61\u0062\0]",
            r"[\u0000-\u0002]",
            r"[\uD7FF\uE000\uFFFF]",
            r"[\uE000-\uFFFF]",
            r"[\d]",
            r"[\w\s]",
            r"\d",
            r"\w",
            r"\s",
        ] {
            let p = JsString::from(source);
            let matcher = RegExpCharacterMatcher::compile_bmp_unicode(&p).unwrap();
            for units in [
                vec![],
                vec![0x61, 0x62, 0xe9, 0x3c3, 0x212a],
                vec![0xd800, 0xdc00, 0x61],
                vec![0xdc00, 0xd800, 0x62],
                vec![0, 1, 2, 8, 0x20, 0xa, 0x2028, 0x2029, 0x180e, 0xfeff],
                vec![0xd7ff, 0xd800, 0xdfff, 0xe000, 0xffff],
                vec![0x30, 0x39, 0x41, 0x5f, 0x7a],
            ] {
                let input = JsString::from_code_units(units);
                for start in 0..=input.len() + 1 {
                    for sticky in [false, true] {
                        let range = matcher
                            .find(&input, start, sticky)
                            .map(|r| [r.start, r.end]);
                        writeln!(
                            rows,
                            "{p:?} input={input:?} start={start} sticky={sticky} range={range:?}"
                        )
                        .unwrap();
                    }
                }
            }
        }
        insta::assert_snapshot!(rows);
    }

    #[test]
    fn unicode_bmp_character_membership_and_unproved_domains_are_distinct() {
        let source = JsString::from(r"[\u0000-\uD7FF\uE000-\uFFFF]");
        let matcher = RegExpCharacterMatcher::compile_bmp_unicode(&source).unwrap();
        for value in 0..=u16::MAX {
            assert_eq!(matcher.matches(value), !(0xd800..=0xdfff).contains(&value));
        }
        for source in [
            ".",
            "[^]",
            "[^a]",
            r"\D",
            r"\W",
            r"\S",
            r"[\D]",
            r"[\d\S]",
            r"[\uD800]",
            r"[\uD7FF-\uE000]",
            r"[\uD800\uDC00]",
            r"[\u{61}]",
            "(a)",
            "[a]b",
            "[a]*",
            r"[\p{ASCII}]",
            "[[a]&&[b]]",
        ] {
            assert!(
                RegExpCharacterMatcher::compile_bmp_unicode(&JsString::from(source)).is_none(),
                "{source}"
            );
        }
        let matcher = RegExpCharacterMatcher::compile_bmp_unicode(&JsString::from("[a]")).unwrap();
        let input = JsString::from("😀a");
        assert_eq!(matcher.find(&input, 1, true), None);
        assert_eq!(matcher.find(&input, 1, false), Some(2..3));
        assert_eq!(matcher.find(&input, 2, true), Some(2..3));
    }

    #[test]
    fn unicode_bmp_character_flat_construction_clones_and_opt_in_failure() {
        let source = JsString::from(format!("[{}]", "a".repeat(100000)).as_str());
        let matcher = RegExpCharacterMatcher::compile_bmp_unicode(&source).unwrap();
        let cloned = matcher.clone();
        drop(matcher);
        let input = JsString::from(format!("{}a", "b".repeat(100000)).as_str());
        assert_eq!(cloned.find(&input, 0, false), Some(100000..100001));
        let mut charges = Vec::new();
        let result =
            RegExpCharacterMatcher::compile_bmp_unicode_with_work(&JsString::from(r"\d"), |n| {
                charges.push(n);
                if n == 65536 { Err("work") } else { Ok(()) }
            });
        assert!(matches!(result, Err("work")));
        assert_eq!(charges, [2, 1024, 65536]);
        let result =
            RegExpCharacterMatcher::compile_bmp_unicode_with_work(&JsString::from("."), |_| {
                Err("should not construct")
            });
        assert!(matches!(result, Ok(None)));
    }

    #[test]
    fn ordinary_character_set_snapshot() {
        let mut rows = String::new();
        for source in [
            "[]",
            "[^]",
            "[a]",
            "[^a]",
            "[a-z]",
            "[µσ]",
            "[ſK]",
            "[--a]",
            "[-a]",
            "[a-]",
            "[[a]",
            r"[\b\-\]\^]",
            r"[\cA\x61\u0062\0]",
            r"[\uD800-\uDFFF]",
            r"\d",
            r"\D",
            r"\s",
            r"\S",
            r"\w",
            r"\W",
            r"[^\d\s]",
            ".",
            "a",
            "[a]b",
            "[a]*",
            "([a])",
            "[a]|b",
            "[a",
            "[z-a]",
            r"[a-\d]",
        ] {
            for ignore_case in [false, true] {
                for dot_all in [false, true] {
                    if source != "." && dot_all {
                        continue;
                    }
                    let source = JsString::from(source);
                    write!(rows, "{source:?} i={ignore_case} s={dot_all}").unwrap();
                    if let Some(matcher) =
                        RegExpCharacterMatcher::compile(&source, ignore_case, dot_all)
                    {
                        for input in [
                            "", "a", "A", "b1", "ς", "Μ", "ſK", "\n", "\u{feff}", "x𐀀y", "-_]",
                        ] {
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
        }
        insta::assert_snapshot!(rows);
    }

    #[test]
    fn class_escapes_agree_with_specification_sets_over_every_utf16_unit() {
        let sources = [r"\d", r"\D", r"\s", r"\S", r"\w", r"\W"];
        let matchers: Vec<_> = sources
            .iter()
            .map(|s| RegExpCharacterMatcher::compile(&JsString::from(*s), false, false).unwrap())
            .collect();
        for unit in 0..=u16::MAX {
            let digit = (0x30..=0x39).contains(&unit);
            let word = digit || matches!(unit,0x41..=0x5a|0x5f|0x61..=0x7a);
            let space = matches!(unit,0x09..=0x0d|0x20|0xa0|0x1680|0x2000..=0x200a|0x2028|0x2029|0x202f|0x205f|0x3000|0xfeff);
            let input = JsString::from_code_units(vec![unit]);
            for (matcher, expected) in matchers
                .iter()
                .zip([digit, !digit, space, !space, word, !word])
            {
                assert_eq!(
                    matcher.find(&input, 0, true).is_some(),
                    expected,
                    "unit={unit:04x}"
                );
            }
        }
    }

    #[test]
    fn opted_in_construction_work_failure_remains_distinct_from_unsupported_syntax() {
        let mut remaining = 65_537usize;
        let result = RegExpCharacterMatcher::compile_with_work(
            &JsString::from(r"\d"),
            false,
            false,
            |work| {
                remaining = remaining.checked_sub(work).ok_or("work")?;
                Ok(())
            },
        );
        assert!(matches!(result, Err("work")));
        let result = RegExpCharacterMatcher::compile_with_work(
            &JsString::from("[a]b"),
            false,
            false,
            |_| -> Result<(), &str> { panic!("unsupported syntax must not construct a set") },
        );
        assert!(matches!(result, Ok(None)));
        assert_eq!(
            RegExpCharacterMatcher::compile(&JsString::from(r"\d"), false, false)
                .unwrap()
                .find(&JsString::from("1"), 0, true),
            Some(0..1)
        );
    }

    #[test]
    fn long_flat_classes_shared_plans_and_searches_have_no_default_cap() {
        let source = format!("[{}b]", "a".repeat(100_000));
        let matcher =
            RegExpCharacterMatcher::compile(&JsString::from(source.as_str()), false, false)
                .unwrap();
        let clone = matcher.clone();
        assert!(Arc::ptr_eq(&matcher.0, &clone.0));
        drop(matcher);
        assert_eq!(
            clone.find(
                &JsString::from(format!("{}b", "x".repeat(120_000)).as_str()),
                0,
                false
            ),
            Some(120_000..120_001)
        );
        assert_eq!(clone.find(&JsString::from("xb"), 0, true), None);
    }
}
