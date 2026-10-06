//! One ordinary-mode CharacterSetMatcher atom (22.2.2.7.1, 22.2.2.8–9).

use crate::{JsString, is_line_terminator, is_whitespace, regexp_canonicalize_character};
use std::{ops::Range, sync::Arc};

/// Immutable matcher for one ordinary class, class escape, or dot atom.
///
/// The complete Pattern must already be validated without `u` or `v`. Groups,
/// concatenations, alternatives and quantifiers of these atoms remain unsupported.
/// Membership uses the pinned ordinary Canonicalize operation before inversion.
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

impl RegExpCharacterMatcher {
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
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, E> {
        let Some((atoms, inverted)) = prepare(source.code_units(), dot_all) else {
            return Ok(None);
        };
        charge(source.len())?;
        charge(1024)?; // Initialize the fixed UTF-16 membership bitmap.
        for atom in &atoms {
            let work = match *atom {
                Atom::Character(_) => 1,
                Atom::Range(start, end) => usize::from(end) - usize::from(start) + 1,
                Atom::Set(_) | Atom::Dot(_) => 65_536,
            };
            charge(work)?;
        }
        let mut program = Program {
            bits: Box::new([0; 1024]),
            inverted,
            ignore_case,
        };
        for atom in atoms {
            program.add(atom);
        }
        Ok(Some(Self(Arc::new(program))))
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
                let unit = canonicalize(unit, self.0.ignore_case);
                let contains = self.0.bits[usize::from(unit) / 64] & (1u64 << (unit % 64)) != 0;
                (contains != self.0.inverted).then_some(start + offset..start + offset + 1)
            })
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

fn prepare(units: &[u16], dot_all: bool) -> Option<(Vec<Atom>, bool)> {
    if units == [u16::from(b'.')] {
        return Some((vec![Atom::Dot(dot_all)], false));
    }
    if units.len() == 2 && units[0] == u16::from(b'\\') && is_class_escape(units[1]) {
        return Some((vec![Atom::Set(units[1])], false));
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
    (index + 1 == units.len()).then_some((atoms, inverted))
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
