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
/// one ClassStringDisjunction operand, adjacent string operands, or strings
/// beside ordinary literal characters, with an empty or longer q member.
/// Canonical
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
        Self::compile_domain(source, ignore_case, multiline, Domain::Single, charge)
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
        Self::compile_domain(source, ignore_case, multiline, Domain::Strings, charge)
    }

    /// Compiles a positive class containing ordinary literal characters and
    /// ClassStringDisjunction operands, with an empty or longer q member.
    ///
    /// All operands contribute to the same union. Character-class escapes,
    /// properties, ranges, nesting and intersection/subtraction decline. The
    /// earlier q-only constructors keep their exact proof domains.
    /// https://262.ecma-international.org/17.0/#sec-compiletocharset
    pub fn compile_character_union_with_work<E>(
        source: &JsString,
        ignore_case: bool,
        multiline: bool,
        charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, RegExpUnicodeSetCompileError<E>> {
        Self::compile_domain(source, ignore_case, multiline, Domain::Characters, charge)
    }

    fn compile_domain<E>(
        source: &JsString,
        ignore_case: bool,
        multiline: bool,
        domain: Domain,
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
        let inner = if domain == Domain::Characters {
            if body.len() < 2 || body[0] != 0x5b || body[1] == 0x5e || body.last() != Some(&0x5d) {
                return Ok(None);
            }
            &body[1..body.len() - 1]
        } else {
            if !body.starts_with(&[0x5b, 0x5c, 0x71, 0x7b]) || !body.ends_with(&[0x7d, 0x5d]) {
                return Ok(None);
            }
            &body[4..body.len() - 2]
        };
        let mut builder = TrieBuilder::new(ignore_case, charge)?;
        let mut at = 0;
        let proved = if domain == Domain::Characters {
            let mut has_q = false;
            let mut has_character = false;
            while at < inner.len() {
                if inner[at..].starts_with(&[0x5c, 0x71, 0x7b]) {
                    has_q = true;
                    at += 3;
                    loop {
                        let Some(&unit) = inner.get(at) else {
                            return Ok(None);
                        };
                        if matches!(unit, 0x7c | 0x7d) {
                            builder.finish_member()?;
                            at += 1;
                            if unit == 0x7d {
                                break;
                            }
                        } else {
                            let Some(point) = single_string_character(inner, &mut at) else {
                                return Ok(None);
                            };
                            builder.push(point)?;
                        }
                    }
                } else {
                    // Two raw ampersands form intersection, never two literal
                    // operands. The decoder rejects ranges and nested classes.
                    if inner[at..].starts_with(&[0x26, 0x26]) {
                        return Ok(None);
                    }
                    let Some(point) = single_string_character(inner, &mut at) else {
                        return Ok(None);
                    };
                    builder.push(point)?;
                    builder.finish_member()?;
                    has_character = true;
                }
            }
            has_q && has_character
        } else {
            let mut proved = domain == Domain::Single;
            loop {
                // Escaped braces/backslashes are consumed by the decoder;
                // only a raw closing brace can separate adjacent q operands.
                let next_operand =
                    domain == Domain::Strings && inner[at..].starts_with(&[0x7d, 0x5c, 0x71, 0x7b]);
                if at == inner.len() || inner.get(at) == Some(&0x7c) || next_operand {
                    builder.finish_member()?;
                    if at == inner.len() {
                        break;
                    }
                    at += if next_operand { 4 } else { 1 };
                    proved |= next_operand;
                } else {
                    let Some(point) = single_string_character(inner, &mut at) else {
                        return Ok(None);
                    };
                    builder.push(point)?;
                }
            }
            proved
        };
        if !proved || !builder.has_strings {
            return Ok(None);
        }
        let TrieBuilder {
            edges,
            terminal,
            max_characters,
            ..
        } = builder;
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

#[derive(Clone, Copy, PartialEq, Eq)]
enum Domain {
    Single,
    Strings,
    Characters,
}

struct TrieBuilder<F> {
    edges: HashMap<(usize, u32), usize>,
    terminal: Vec<bool>,
    node: usize,
    length: usize,
    max_characters: usize,
    has_strings: bool,
    ignore_case: bool,
    charge: F,
}

impl<E, F: FnMut(usize) -> Result<(), E>> TrieBuilder<F> {
    fn new(ignore_case: bool, mut charge: F) -> Result<Self, RegExpUnicodeSetCompileError<E>> {
        let mut terminal = Vec::new();
        charge(1).map_err(RegExpUnicodeSetCompileError::Work)?;
        terminal
            .try_reserve(1)
            .map_err(|_| RegExpUnicodeSetCompileError::Capacity)?;
        terminal.push(false);
        Ok(Self {
            edges: HashMap::new(),
            terminal,
            node: 0,
            length: 0,
            max_characters: 0,
            has_strings: false,
            ignore_case,
            charge,
        })
    }

    fn finish_member(&mut self) -> Result<(), RegExpUnicodeSetCompileError<E>> {
        (self.charge)(1).map_err(RegExpUnicodeSetCompileError::Work)?;
        self.terminal[self.node] = true;
        self.has_strings |= self.length != 1;
        self.max_characters = self.max_characters.max(self.length);
        self.node = 0;
        self.length = 0;
        Ok(())
    }

    fn push(&mut self, point: u32) -> Result<(), RegExpUnicodeSetCompileError<E>> {
        let point = if self.ignore_case {
            (self.charge)(2 + (usize::BITS - SIMPLE_CASE_FOLD.len().leading_zeros()) as usize)
                .map_err(RegExpUnicodeSetCompileError::Work)?;
            regexp_canonicalize_character(point, true, true)
        } else {
            point
        };
        (self.charge)(1).map_err(RegExpUnicodeSetCompileError::Work)?;
        self.node = if let Some(&next) = self.edges.get(&(self.node, point)) {
            next
        } else {
            let next = self.terminal.len();
            let needed = next
                .checked_add(1)
                .ok_or(RegExpUnicodeSetCompileError::Capacity)?;
            let edge_needed = self
                .edges
                .len()
                .checked_add(1)
                .ok_or(RegExpUnicodeSetCompileError::Capacity)?;
            let movement = if needed > self.terminal.capacity() {
                self.terminal.len()
            } else {
                0
            };
            let rehash = if edge_needed > self.edges.capacity() {
                self.edges.len()
            } else {
                0
            };
            (self.charge)(movement.saturating_add(rehash).saturating_add(2))
                .map_err(RegExpUnicodeSetCompileError::Work)?;
            self.terminal
                .try_reserve(1)
                .map_err(|_| RegExpUnicodeSetCompileError::Capacity)?;
            self.edges
                .try_reserve(1)
                .map_err(|_| RegExpUnicodeSetCompileError::Capacity)?;
            self.terminal.push(false);
            self.edges.insert((self.node, point), next);
            next
        };
        self.length = self
            .length
            .checked_add(1)
            .ok_or(RegExpUnicodeSetCompileError::Capacity)?;
        Ok(())
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
