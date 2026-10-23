//! A standalone ClassStringDisjunction with variable-length members (22.2.2.7).

use crate::{
    JsString, RegExpUnicodeSetCompileError,
    case_data::SIMPLE_CASE_FOLD,
    regexp_canonicalize_character,
    regexp_literal::unicode_start,
    regexp_unicode_character::{
        unicode_assertions_match, unicode_input_character, unicode_word_boundary,
    },
    regexp_unicode_class::single_string_character,
};
use std::{collections::HashMap, ops::Range, sync::Arc};

/// Immutable forward matcher for a complete positive v/vi `\q` string class.
///
/// Requires complete validated v/vi Pattern input. Separate constructors prove
/// one ClassStringDisjunction operand or a union of adjacent string operands,
/// with at least one empty or multi-character alternative overall. Canonical
/// outside line/word assertions are admitted. Captures,
/// repetitions, further consuming terms, other class algebra and string properties
/// remain separate caller proofs. Complete enclosing captures may be projected
/// by the caller. Earlier character-only compilers retain their domains.
#[derive(Clone, Debug)]
pub struct RegExpUnicodeStringSetMatcher {
    program: Arc<Program>,
}

#[derive(Debug)]
struct Program {
    edges: HashMap<(usize, u32), usize>,
    terminal: Vec<bool>,
    max_characters: usize,
    ignore_case: bool,
    multiline: bool,
    start_anchor: bool,
    end_anchor: bool,
    start_word: Option<bool>,
    end_word: Option<bool>,
}

impl RegExpUnicodeStringSetMatcher {
    /// Compiles a complete borrowed single-q class with checked construction work.
    ///
    /// The prefix trie preserves shared prefixes without source/input copies or
    /// recursive nodes. Both vector and hash-index growth reserve fallibly.
    /// The final shared allocation retains the existing Arc allocation boundary.
    /// Empty alternatives follow all consuming alternatives; longer strings
    /// precede shorter ones regardless of their source order (22.2.2.7).
    /// https://262.ecma-international.org/17.0/#sec-compileatom
    pub fn compile_with_work<E>(
        source: &JsString,
        ignore_case: bool,
        multiline: bool,
        charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, RegExpUnicodeSetCompileError<E>> {
        Self::compile_domain(source, ignore_case, multiline, false, charge)
    }

    /// Compiles a positive class consisting of at least two adjacent `\q`
    /// operands, with at least one empty or multi-character member overall.
    ///
    /// The union shares prefix storage and longest-member priority across all
    /// operands. Every complete operand is decoded in source order, including
    /// after an empty member. Other class operands and set operations decline.
    /// The standalone constructor retains its single-operand proof domain.
    /// https://262.ecma-international.org/17.0/#sec-compiletocharset
    pub fn compile_string_union_with_work<E>(
        source: &JsString,
        ignore_case: bool,
        multiline: bool,
        charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, RegExpUnicodeSetCompileError<E>> {
        Self::compile_domain(source, ignore_case, multiline, true, charge)
    }

    fn compile_domain<E>(
        source: &JsString,
        ignore_case: bool,
        multiline: bool,
        union: bool,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, RegExpUnicodeSetCompileError<E>> {
        let units = source.code_units();
        charge(units.len().saturating_mul(4)).map_err(RegExpUnicodeSetCompileError::Work)?;
        let start_anchor = units.first() == Some(&0x5e);
        let end_anchor = units.last() == Some(&0x24)
            && units[..units.len() - 1]
                .iter()
                .rev()
                .take_while(|&&u| u == 0x5c)
                .count()
                % 2
                == 0;
        let mut begin = usize::from(start_anchor);
        let Some(mut end) = units.len().checked_sub(usize::from(end_anchor)) else {
            return Ok(None);
        };
        let start_word = match units.get(begin..end).and_then(|body| body.get(..2)) {
            Some([0x5c, kind @ (0x62 | 0x42)]) => {
                begin += 2;
                Some(*kind == 0x62)
            }
            _ => None,
        };
        let end_word = if end >= begin + 2
            && units[end - 2] == 0x5c
            && matches!(units[end - 1], 0x62 | 0x42)
            && units[..end - 1]
                .iter()
                .rev()
                .take_while(|&&u| u == 0x5c)
                .count()
                % 2
                == 1
        {
            end -= 2;
            Some(units[end + 1] == 0x62)
        } else {
            None
        };
        let Some(body) = units.get(begin..end) else {
            return Ok(None);
        };
        if !body.starts_with(&[0x5b, 0x5c, 0x71, 0x7b]) || !body.ends_with(&[0x7d, 0x5d]) {
            return Ok(None);
        }
        let inner = &body[4..body.len() - 2];
        let mut terminal = Vec::new();
        charge(1).map_err(RegExpUnicodeSetCompileError::Work)?;
        terminal
            .try_reserve(1)
            .map_err(|_| RegExpUnicodeSetCompileError::Capacity)?;
        terminal.push(false);
        let mut edges = HashMap::new();
        let mut at = 0;
        let mut node = 0;
        let mut length = 0usize;
        let mut max_characters = 0;
        let mut has_strings = false;
        let mut has_union = false;
        loop {
            // Only an unescaped closing brace followed immediately by another
            // complete q prefix separates operands. The character decoder
            // consumes escaped braces and backslashes before we reach here.
            let next_operand = union && inner[at..].starts_with(&[0x7d, 0x5c, 0x71, 0x7b]);
            if at == inner.len() || inner.get(at) == Some(&0x7c) || next_operand {
                charge(1).map_err(RegExpUnicodeSetCompileError::Work)?;
                terminal[node] = true;
                has_strings |= length != 1;
                max_characters = max_characters.max(length);
                node = 0;
                length = 0;
                if at == inner.len() {
                    break;
                }
                at += if next_operand { 4 } else { 1 };
                has_union |= next_operand;
                continue;
            }
            let Some(point) = single_string_character(inner, &mut at) else {
                return Ok(None);
            };
            let point = if ignore_case {
                charge(2 + (usize::BITS - SIMPLE_CASE_FOLD.len().leading_zeros()) as usize)
                    .map_err(RegExpUnicodeSetCompileError::Work)?;
                regexp_canonicalize_character(point, true, true)
            } else {
                point
            };
            charge(1).map_err(RegExpUnicodeSetCompileError::Work)?;
            node = if let Some(&next) = edges.get(&(node, point)) {
                next
            } else {
                let next = terminal.len();
                let needed = next
                    .checked_add(1)
                    .ok_or(RegExpUnicodeSetCompileError::Capacity)?;
                let edge_needed = edges
                    .len()
                    .checked_add(1)
                    .ok_or(RegExpUnicodeSetCompileError::Capacity)?;
                let movement = if needed > terminal.capacity() {
                    terminal.len()
                } else {
                    0
                };
                let rehash = if edge_needed > edges.capacity() {
                    edges.len()
                } else {
                    0
                };
                charge(movement.saturating_add(rehash).saturating_add(2))
                    .map_err(RegExpUnicodeSetCompileError::Work)?;
                terminal
                    .try_reserve(1)
                    .map_err(|_| RegExpUnicodeSetCompileError::Capacity)?;
                edges
                    .try_reserve(1)
                    .map_err(|_| RegExpUnicodeSetCompileError::Capacity)?;
                terminal.push(false);
                edges.insert((node, point), next);
                next
            };
            length = length
                .checked_add(1)
                .ok_or(RegExpUnicodeSetCompileError::Capacity)?;
        }
        if !has_strings || (union && !has_union) {
            return Ok(None);
        }
        Ok(Some(Self {
            program: Arc::new(Program {
                edges,
                terminal,
                max_characters,
                ignore_case,
                multiline,
                start_anchor,
                end_anchor,
                start_word,
                end_word,
            }),
        }))
    }

    /// Bounds UTF-16 consumption by any one member; empty-only sets return zero.
    pub fn max_match_code_units(&self) -> usize {
        self.program.max_characters.saturating_mul(2)
    }

    /// Conservative input-work multiplier for a sticky or searching invocation.
    /// Searching can walk a longest-member prefix at each candidate position.
    pub fn search_passes(&self, sticky: bool) -> usize {
        let p = &self.program;
        let folding = if p.ignore_case {
            2 + (usize::BITS - SIMPLE_CASE_FOLD.len().leading_zeros()) as usize
        } else {
            0
        };
        let per_character = 3usize
            .saturating_add(folding)
            .saturating_add(usize::from(p.start_anchor) + usize::from(p.end_anchor))
            .saturating_add(
                8 * (usize::from(p.start_word.is_some()) + usize::from(p.end_word.is_some())),
            );
        if sticky {
            per_character
        } else {
            per_character.saturating_mul(p.max_characters.saturating_add(1))
        }
    }

    /// Finds the earliest start and longest member satisfying outside assertions.
    ///
    /// A rejected longer member leaves shorter successful continuations eligible.
    /// Search allocates no state and preserves original UTF-16 ranges. Initial
    /// offsets within a surrogate pair use the approved leading-boundary rule.
    pub fn find(&self, input: &JsString, start: usize, sticky: bool) -> Option<Range<usize>> {
        let p = &self.program;
        let units = input.code_units();
        let mut candidate = unicode_start(input, start)?;
        loop {
            let mut node = 0;
            let mut at = candidate;
            let mut best = if p.terminal[0] && p.assertions_match(units, candidate, candidate) {
                Some(candidate)
            } else {
                None
            };
            for _ in 0..p.max_characters {
                let Some(&first) = units.get(at) else { break };
                let (point, width) = unicode_input_character(units, at, first);
                let point = if p.ignore_case {
                    regexp_canonicalize_character(point, true, true)
                } else {
                    point
                };
                let Some(&next) = p.edges.get(&(node, point)) else {
                    break;
                };
                node = next;
                at += width;
                if p.terminal[node] && p.assertions_match(units, candidate, at) {
                    best = Some(at)
                }
            }
            if let Some(end) = best {
                return Some(candidate..end);
            }
            if sticky {
                return None;
            }
            let &first = units.get(candidate)?;
            candidate += unicode_input_character(units, candidate, first).1;
        }
    }
}

impl Program {
    fn assertions_match(&self, units: &[u16], start: usize, end: usize) -> bool {
        unicode_assertions_match(
            units,
            start,
            end,
            self.start_anchor,
            self.end_anchor,
            self.multiline,
        ) && self
            .start_word
            .is_none_or(|wanted| unicode_word_boundary(units, start, self.ignore_case) == wanted)
            && self
                .end_word
                .is_none_or(|wanted| unicode_word_boundary(units, end, self.ignore_case) == wanted)
    }
}

#[cfg(test)]
#[path = "regexp_unicode_string_set_tests.rs"]
mod tests;
