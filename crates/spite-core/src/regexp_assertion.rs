//! Ordinary outer assertions and WordCharacters (22.2.2.4, 22.2.2.9.3).

use std::ops::Range;

/// A conjunction at one input position, independent of consuming terms.
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct Assertions {
    at_start: bool,
    at_end: bool,
    word_boundary: Option<bool>,
    impossible: bool,
}

impl Assertions {
    fn add(&mut self, assertion: u16) {
        match assertion {
            94 => self.at_start = true,
            36 => self.at_end = true,
            98 | 66 => {
                let boundary = assertion == 98;
                self.impossible |= self.word_boundary.is_some_and(|old| old != boundary);
                self.word_boundary = Some(boundary);
            }
            _ => unreachable!("only assertion tokens are added"),
        }
    }

    pub(crate) fn has_word_boundary(self) -> bool {
        self.word_boundary.is_some()
    }

    pub(crate) fn accepts(self, units: &[u16], position: usize, multiline: bool) -> bool {
        if self.impossible || position > units.len() {
            return false;
        }
        let previous = position.checked_sub(1).and_then(|i| units.get(i)).copied();
        let next = units.get(position).copied();
        if self.at_start && position != 0 && !(multiline && previous.is_some_and(line_terminator)) {
            return false;
        }
        if self.at_end
            && position != units.len()
            && !(multiline && next.is_some_and(line_terminator))
        {
            return false;
        }
        self.word_boundary.is_none_or(|expected| {
            expected == (previous.is_some_and(word_character) != next.is_some_and(word_character))
        })
    }

    /// Return the first allowed position at or after a monotone lower bound.
    pub(crate) fn next_position(
        self,
        units: &[u16],
        cursor: &mut usize,
        minimum: usize,
        multiline: bool,
    ) -> Option<usize> {
        *cursor = (*cursor).max(minimum);
        if self.impossible || (self.at_start && !multiline && *cursor != 0) {
            return None;
        }
        while *cursor <= units.len() {
            if self.accepts(units, *cursor, multiline) {
                return Some(*cursor);
            }
            if *cursor == units.len() {
                break;
            }
            *cursor += 1;
        }
        None
    }
}

fn line_terminator(unit: u16) -> bool {
    matches!(unit, 0x0a | 0x0d | 0x2028 | 0x2029)
}

// Without Unicode mode, IgnoreCase does not extend the ASCII WordCharacters set.
fn word_character(unit: u16) -> bool {
    matches!(unit, 48..=57 | 65..=90 | 95 | 97..=122)
}

fn assertion_at(units: &[u16], index: usize) -> Option<(u16, usize)> {
    match *units.get(index)? {
        unit @ (94 | 36) => Some((unit, 1)),
        92 => match *units.get(index + 1)? {
            unit @ (98 | 66) => Some((unit, 2)),
            _ => None,
        },
        _ => None,
    }
}

/// Strip complete outer assertion sequences in one forward scan. Escapes,
/// classes and group contents remain opaque; no nested assertion is extracted.
pub(crate) fn split_outer_assertions(
    units: &[u16],
) -> Option<(Range<usize>, Assertions, Assertions)> {
    let mut start = 0;
    let mut leading = Assertions::default();
    while let Some((assertion, width)) = assertion_at(units, start) {
        leading.add(assertion);
        start += width;
    }
    let mut index = start;
    let mut end = start;
    let mut depth = 0usize;
    let mut trailing = Assertions::default();
    while let Some(&unit) = units.get(index) {
        if depth == 0 {
            if let Some((assertion, width)) = assertion_at(units, index) {
                trailing.add(assertion);
                index += width;
                continue;
            }
        }
        match unit {
            92 => index = index.saturating_add(2).min(units.len()),
            91 => {
                index += 1;
                while let Some(&class_unit) = units.get(index) {
                    index += 1;
                    if class_unit == 92 {
                        index = index.saturating_add(1).min(units.len());
                    } else if class_unit == 93 {
                        break;
                    }
                }
            }
            40 => {
                depth = depth.checked_add(1)?;
                index += 1;
            }
            41 => {
                depth = depth.checked_sub(1)?;
                index += 1;
            }
            _ => index += 1,
        }
        end = index;
        trailing = Assertions::default();
    }
    (start != 0 || end != units.len()).then_some((start..end, leading, trailing))
}
