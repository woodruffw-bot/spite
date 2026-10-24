//! A standalone ClassStringDisjunction with variable-length members (22.2.2.7).

use crate::{
    JsString, RegExpUnicodeClassMatcher, RegExpUnicodeSetCompileError,
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
/// beside ordinary scalar operands, with an empty or longer q member.
/// Canonical
/// outside line/word assertions are admitted. Captures,
/// repetitions, further consuming terms, negated nested classes, other set operations and string properties
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
    scalars: Option<RegExpUnicodeClassMatcher>,
    max_characters: usize,
    ignore_case: bool,
    multiline: bool,
    start_anchor: bool,
    end_anchor: bool,
    start_end_anchor: bool,
    end_start_anchor: bool,
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

    /// Compiles a positive flat union of q strings with supported character
    /// escapes, Unicode character properties, or ranges, optionally with literal
    /// characters. At least one q member must be empty or multi-character.
    ///
    /// Scalar membership reuses checked, folded intervals. String properties,
    /// nested classes, intersection/subtraction, and repetition remain separate
    /// proofs. Earlier literal-only and q-only entries retain their domains.
    /// https://262.ecma-international.org/17.0/#sec-compiletocharset
    pub fn compile_scalar_union_with_work<E>(
        source: &JsString,
        ignore_case: bool,
        multiline: bool,
        charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, RegExpUnicodeSetCompileError<E>> {
        Self::compile_domain(source, ignore_case, multiline, Domain::Scalars, charge)
    }

    /// Compiles positive nested unions containing an empty or longer q member.
    ///
    /// Requires at least one positive nested class. Literal characters, ranges,
    /// character escapes and supported character properties share the same
    /// union. Positive union associativity permits flat storage and two borrowed
    /// source passes, independently of nesting depth. Negated nested classes,
    /// set operations, string properties and repetition decline.
    /// https://262.ecma-international.org/17.0/#sec-compiletocharset
    pub fn compile_positive_nested_union_with_work<E>(
        source: &JsString,
        ignore_case: bool,
        multiline: bool,
        charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, RegExpUnicodeSetCompileError<E>> {
        Self::compile_domain(
            source,
            ignore_case,
            multiline,
            Domain::PositiveNested,
            charge,
        )
    }

    /// Compiles a positive flat chain of q-only intersections or subtractions.
    ///
    /// At least one operand member must be empty or multi-character. Every
    /// operand is decoded and folded completely, including after an empty
    /// intermediate result. Result members are a subset of the first operand;
    /// later operands probe its trie without adding nodes or copying strings.
    /// Nested/scalar operands, mixed operators and repetition decline.
    /// https://262.ecma-international.org/17.0/#sec-compiletocharset
    pub fn compile_string_operations_with_work<E>(
        source: &JsString,
        ignore_case: bool,
        multiline: bool,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, RegExpUnicodeSetCompileError<E>> {
        Self::compile_string_operations_with_work_impl(source, ignore_case, multiline, &mut charge)
    }

    // Share compilation across callback types; callers keep their original
    // work events and errors without duplicating the delegated compiler.
    #[inline(never)]
    fn compile_string_operations_with_work_impl<E>(
        source: &JsString,
        ignore_case: bool,
        multiline: bool,
        mut charge: &mut dyn FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, RegExpUnicodeSetCompileError<E>> {
        let Some(Framing {
            body,
            start_anchor,
            end_anchor,
            start_word,
            end_word,
        }) = frame_source(source, &mut charge)?
        else {
            return Ok(None);
        };
        if !body.starts_with(&[0x5b, 0x5c, 0x71, 0x7b]) || body.last() != Some(&0x5d) {
            return Ok(None);
        }
        let inner = &body[4..body.len() - 1];
        // Prepaid borrowed framing proves a first q operand and an operator
        // before any trie storage is reserved.
        let mut first_end = 0;
        loop {
            let Some(&unit) = inner.get(first_end) else {
                return Ok(None);
            };
            if unit == 0x7d {
                break;
            }
            if unit == 0x7c {
                first_end += 1;
            } else if single_string_character(inner, &mut first_end).is_none() {
                return Ok(None);
            }
        }
        let Some(operator) = StringSetOperator::at(inner, first_end + 1) else {
            return Ok(None);
        };
        let mut builder = TrieBuilder::new(ignore_case, charge)?;
        let mut members = Vec::new();
        let mut at = 0;
        while at <= first_end {
            if at == first_end || inner[at] == 0x7c {
                builder.finish_left_member(&mut members)?;
                if at == first_end {
                    break;
                }
                at += 1;
            } else {
                let Some(point) = single_string_character(&inner[..first_end], &mut at) else {
                    return Ok(None);
                };
                builder.push(point)?;
            }
        }
        (builder.charge)(builder.terminal.len()).map_err(RegExpUnicodeSetCompileError::Work)?;
        let mut marked = Vec::new();
        marked
            .try_reserve(builder.terminal.len())
            .map_err(|_| RegExpUnicodeSetCompileError::Capacity)?;
        marked.resize(builder.terminal.len(), false);
        let mut matched = Vec::new();
        let mut active = members.len();
        at = first_end + 1;
        loop {
            if StringSetOperator::at(inner, at) != Some(operator) {
                return Ok(None);
            }
            at += 2;
            if !inner[at..].starts_with(&[0x5c, 0x71, 0x7b]) {
                return Ok(None);
            }
            at += 3;
            let mut node = if active == 0 { None } else { Some(0) };
            let mut length = 0usize;
            loop {
                let Some(&unit) = inner.get(at) else {
                    return Ok(None);
                };
                if matches!(unit, 0x7c | 0x7d) {
                    builder.finish_probe_member(node, length, &mut marked, &mut matched)?;
                    at += 1;
                    if unit == 0x7d {
                        break;
                    }
                    node = if active == 0 { None } else { Some(0) };
                    length = 0;
                } else {
                    let Some(point) = single_string_character(inner, &mut at) else {
                        return Ok(None);
                    };
                    node = builder.probe(node, point)?;
                    length = length
                        .checked_add(1)
                        .ok_or(RegExpUnicodeSetCompileError::Capacity)?;
                }
            }
            builder.apply_operation(
                operator,
                &mut members,
                &mut marked,
                &mut matched,
                &mut active,
            )?;
            if at == inner.len() {
                break;
            }
        }
        if !builder.has_strings {
            return Ok(None);
        }
        (builder.charge)(members.len()).map_err(RegExpUnicodeSetCompileError::Work)?;
        let max_characters = members
            .iter()
            .filter(|&&(node, _)| builder.terminal[node])
            .map(|&(_, length)| length)
            .max()
            .unwrap_or(0);
        let TrieBuilder {
            edges, terminal, ..
        } = builder;
        Ok(Some(Self {
            program: Arc::new(Program {
                edges,
                terminal,
                scalars: None,
                max_characters,
                ignore_case,
                multiline,
                start_anchor,
                end_anchor,
                start_end_anchor: false,
                end_start_anchor: false,
                start_word,
                end_word,
            }),
        }))
    }

    /// Compiles positive set-operation chains over finite character/string sets.
    ///
    /// Operands are q disjunctions, literal characters, or positive nested unions
    /// of those members. At least one operand is not a bare q disjunction, and
    /// at least one q member is empty or multi-character. Ranges, class/property
    /// escapes, negation, inner set operations and repetition remain separate
    /// proofs. Empty classes emit no members; empty q branches emit one.
    /// https://262.ecma-international.org/17.0/#sec-compiletocharset
    pub fn compile_finite_string_operations_with_work<E>(
        source: &JsString,
        ignore_case: bool,
        multiline: bool,
        charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, RegExpUnicodeSetCompileError<E>> {
        Self::compile_finite_operations_domain(source, ignore_case, multiline, false, charge)
    }

    /// Compiles finite left operands filtered by character-only right operands.
    ///
    /// The positive outer class is a homogeneous intersection/subtraction chain.
    /// The left operand is a q disjunction or positive finite q/literal union,
    /// with an empty or longer q member somewhere in the chain. Later operands
    /// may also be finite or character-only classes and class/property escapes.
    /// At least one later operand requires the scalar proof. Scalar classes may
    /// nest, negate, or use character-only set operations, but contain no q.
    /// Ranges are kept as intervals and only one-character words can belong to
    /// a scalar set. Further atoms and repetition remain separate proofs.
    /// https://262.ecma-international.org/17.0/#sec-compiletocharset
    pub fn compile_scalar_rhs_string_operations_with_work<E>(
        source: &JsString,
        ignore_case: bool,
        multiline: bool,
        charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, RegExpUnicodeSetCompileError<E>> {
        Self::compile_finite_operations_domain(source, ignore_case, multiline, true, charge)
    }

    fn compile_finite_operations_domain<E>(
        source: &JsString,
        ignore_case: bool,
        multiline: bool,
        scalar_rhs: bool,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, RegExpUnicodeSetCompileError<E>> {
        Self::compile_finite_operations_domain_impl(
            source,
            ignore_case,
            multiline,
            scalar_rhs,
            &mut charge,
        )
    }

    // Share compilation across callback types; callers keep their original
    // work events and errors without duplicating the delegated compiler.
    #[inline(never)]
    fn compile_finite_operations_domain_impl<E>(
        source: &JsString,
        ignore_case: bool,
        multiline: bool,
        scalar_rhs: bool,
        mut charge: &mut dyn FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, RegExpUnicodeSetCompileError<E>> {
        let Some(Framing {
            body,
            start_anchor,
            end_anchor,
            start_word,
            end_word,
        }) = frame_source(source, &mut charge)?
        else {
            return Ok(None);
        };
        if body.len() < 2 || body[0] != 0x5b || body[1] == 0x5e || body.last() != Some(&0x5d) {
            return Ok(None);
        }
        let inner = &body[1..body.len() - 1];
        let mut builder = TrieBuilder::new(ignore_case, charge)?;
        let mut members = Vec::new();
        let mut at = 0;
        let Some(mut extended) = finite_operand(inner, &mut at, |event| match event {
            WordEvent::Point(point) => builder.push(point),
            WordEvent::End => builder.finish_left_member(&mut members),
        })?
        else {
            return Ok(None);
        };
        let Some(operator) = StringSetOperator::at(inner, at) else {
            return Ok(None);
        };
        (builder.charge)(builder.terminal.len()).map_err(RegExpUnicodeSetCompileError::Work)?;
        let mut marked = Vec::new();
        marked
            .try_reserve(builder.terminal.len())
            .map_err(|_| RegExpUnicodeSetCompileError::Capacity)?;
        marked.resize(builder.terminal.len(), false);
        let mut matched = Vec::new();
        let mut active = members.len();
        let mut has_scalar_rhs = false;
        loop {
            if StringSetOperator::at(inner, at) != Some(operator) {
                return Ok(None);
            }
            at += 2;
            let scalar = if scalar_rhs {
                scalar_operand(inner, &mut at, ignore_case, &mut builder.charge)?
            } else {
                None
            };
            if let Some(scalar) = scalar {
                has_scalar_rhs = true;
                builder.mark_scalar_members(&scalar, &mut marked, &mut matched)?;
                builder.apply_operation(
                    operator,
                    &mut members,
                    &mut marked,
                    &mut matched,
                    &mut active,
                )?;
                if at == inner.len() {
                    break;
                }
                continue;
            }
            let mut node = if active == 0 { None } else { Some(0) };
            let mut length = 0usize;
            let Some(complex) = finite_operand(inner, &mut at, |event| {
                match event {
                    WordEvent::Point(point) => {
                        node = builder.probe(node, point)?;
                        length = length
                            .checked_add(1)
                            .ok_or(RegExpUnicodeSetCompileError::Capacity)?;
                    }
                    WordEvent::End => {
                        builder.finish_probe_member(node, length, &mut marked, &mut matched)?;
                        node = if active == 0 { None } else { Some(0) };
                        length = 0;
                    }
                }
                Ok(())
            })?
            else {
                return Ok(None);
            };
            extended |= complex;
            builder.apply_operation(
                operator,
                &mut members,
                &mut marked,
                &mut matched,
                &mut active,
            )?;
            if at == inner.len() {
                break;
            }
        }
        if !(if scalar_rhs { has_scalar_rhs } else { extended }) || !builder.has_strings {
            return Ok(None);
        }
        (builder.charge)(members.len()).map_err(RegExpUnicodeSetCompileError::Work)?;
        let max_characters = members
            .iter()
            .filter(|&&(node, _)| builder.terminal[node])
            .map(|&(_, length)| length)
            .max()
            .unwrap_or(0);
        let TrieBuilder {
            edges, terminal, ..
        } = builder;
        Ok(Some(Self {
            program: Arc::new(Program {
                edges,
                terminal,
                scalars: None,
                max_characters,
                ignore_case,
                multiline,
                start_anchor,
                end_anchor,
                start_end_anchor: false,
                end_start_anchor: false,
                start_word,
                end_word,
            }),
        }))
    }

    fn compile_domain<E>(
        source: &JsString,
        ignore_case: bool,
        multiline: bool,
        domain: Domain,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, RegExpUnicodeSetCompileError<E>> {
        Self::compile_domain_impl(source, ignore_case, multiline, domain, &mut charge)
    }

    // Share compilation across callback types; callers keep their original
    // work events and errors without duplicating the delegated compiler.
    #[inline(never)]
    fn compile_domain_impl<E>(
        source: &JsString,
        ignore_case: bool,
        multiline: bool,
        domain: Domain,
        mut charge: &mut dyn FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, RegExpUnicodeSetCompileError<E>> {
        let Some(Framing {
            body,
            start_anchor,
            end_anchor,
            start_word,
            end_word,
        }) = frame_source(source, &mut charge)?
        else {
            return Ok(None);
        };
        let mixed = matches!(
            domain,
            Domain::Characters | Domain::Scalars | Domain::PositiveNested
        );
        let inner = if mixed {
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
        let scalars = if matches!(domain, Domain::Scalars | Domain::PositiveNested) {
            let plan = if domain == Domain::PositiveNested {
                RegExpUnicodeClassMatcher::compile_positive_nested_string_union_portion_with_work(
                    body,
                    ignore_case,
                    &mut charge,
                )?
            } else {
                RegExpUnicodeClassMatcher::compile_scalar_union_portion_with_work(
                    body,
                    ignore_case,
                    &mut charge,
                )?
            };
            let Some(plan) = plan else {
                return Ok(None);
            };
            if domain == Domain::PositiveNested && plan.is_empty_character_union() {
                None
            } else {
                Some(plan)
            }
        } else {
            None
        };
        let mut builder = TrieBuilder::new(ignore_case, charge)?;
        let mut at = 0;
        let proved = if mixed {
            let mut has_q = false;
            let mut has_character = false;
            while at < inner.len() {
                // The first pass proved every positive class boundary. The
                // character/term decoders consume escaped brackets themselves.
                if domain == Domain::PositiveNested && matches!(inner[at], 0x5b | 0x5d) {
                    at += 1;
                    continue;
                }
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
                    if matches!(domain, Domain::Scalars | Domain::PositiveNested) {
                        let Some(end) =
                            RegExpUnicodeClassMatcher::string_union_scalar_term_end(inner, at)
                        else {
                            return Ok(None);
                        };
                        at = end;
                        has_character = true;
                        continue;
                    }
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
            has_q && (has_character || domain == Domain::PositiveNested)
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
            mut max_characters,
            ..
        } = builder;
        if scalars.is_some() {
            max_characters = max_characters.max(1);
        }
        Ok(Some(Self {
            program: Arc::new(Program {
                edges,
                terminal,
                scalars,
                max_characters,
                ignore_case,
                multiline,
                start_anchor,
                end_anchor,
                start_end_anchor: false,
                end_start_anchor: false,
                start_word,
                end_word,
            }),
        }))
    }

    /// Charges a live canonical trie scan before proving prefix-free membership.
    /// Scalar fallbacks retain a separate proof, even if disjoint from words.
    pub(crate) fn has_prefix_free_string_members_with_work<E>(
        &self,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<bool, RegExpUnicodeSetCompileError<E>> {
        let program = &self.program;
        if program.scalars.is_some() || program.terminal[0] {
            return Ok(false);
        }
        charge(
            program
                .edges
                .capacity()
                .saturating_add(program.terminal.len().saturating_mul(3))
                .saturating_add(1),
        )
        .map_err(RegExpUnicodeSetCompileError::Work)?;
        let mut parents = Vec::<usize>::new();
        parents
            .try_reserve_exact(program.terminal.len())
            .map_err(|_| RegExpUnicodeSetCompileError::Capacity)?;
        parents.resize(program.terminal.len(), 0);
        let mut live = Vec::<bool>::new();
        live.try_reserve_exact(program.terminal.len())
            .map_err(|_| RegExpUnicodeSetCompileError::Capacity)?;
        live.extend_from_slice(&program.terminal);
        for (&(parent, _), &child) in &program.edges {
            // TrieBuilder appends each child after its unique parent. Algebra
            // may clear terminals while retaining their now-dead suffix edges.
            debug_assert!(parent < child);
            parents[child] = parent;
        }
        for child in (1..live.len()).rev() {
            if live[child] {
                let parent = parents[child];
                if program.terminal[parent] {
                    return Ok(false);
                }
                live[parent] = true;
            }
        }
        Ok(true)
    }

    /// Whether ordinary scalar matching supplements the finite word trie.
    pub(crate) fn has_scalar_fallback(&self) -> bool {
        self.program.scalars.is_some()
    }

    /// Charges the finite word count used to bound endpoint continuation probes.
    pub(crate) fn prefix_candidate_bound_with_work<E>(
        &self,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<usize, E> {
        charge(self.program.terminal.len().saturating_add(1))?;
        Ok(self
            .program
            .terminal
            .iter()
            .filter(|&&terminal| terminal)
            .count()
            .saturating_add(usize::from(self.program.scalars.is_some())))
    }
    /// Finds the preferred first member whose preferred second member exists.
    /// The caller proves nonnullability/no assertions and charges all probes.
    pub(crate) fn prefix_pair(
        &self,
        input: &JsString,
        start: usize,
    ) -> Option<(Range<usize>, Range<usize>)> {
        debug_assert!(!self.has_empty_member() && !self.has_outside_assertions());
        let p = &self.program;
        let units = input.code_units();
        let (mut node, mut at, mut best) = (0usize, start, None);
        for _ in 0..p.max_characters {
            let Some(&first) = units.get(at) else { break };
            let (point, width) = unicode_input_character(units, at, first);
            if at == start
                && p.scalars
                    .as_ref()
                    .is_some_and(|plan| plan.matches_original_character(point))
            {
                if let Some(second) = self.find(input, at + width, true) {
                    best = Some((start..at + width, second));
                }
            }
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
            if p.terminal[node] {
                if let Some(second) = self.find(input, at, true) {
                    best = Some((start..at, second));
                }
            }
        }
        best
    }

    /// Returns the preferred consuming member ending strictly before a bound.
    /// The caller proves no assertions and charges the full prefix inspection.
    pub(crate) fn prefix_before(
        &self,
        input: &JsString,
        start: usize,
        before: usize,
    ) -> Option<Range<usize>> {
        debug_assert!(!self.has_outside_assertions());
        let p = &self.program;
        let units = input.code_units();
        let mut best = None;
        if let Some(&first) = units.get(start) {
            let (point, width) = unicode_input_character(units, start, first);
            if start + width < before
                && p.scalars
                    .as_ref()
                    .is_some_and(|plan| plan.matches_original_character(point))
            {
                best = Some(start..start + width);
            }
        }
        let (mut node, mut at) = (0usize, start);
        for _ in 0..p.max_characters {
            let Some(&first) = units.get(at) else { break };
            let (point, width) = unicode_input_character(units, at, first);
            at += width;
            if at >= before {
                break;
            }
            let point = if p.ignore_case {
                regexp_canonicalize_character(point, true, true)
            } else {
                point
            };
            let Some(&next) = p.edges.get(&(node, point)) else {
                break;
            };
            node = next;
            if p.terminal[node] {
                best = Some(start..at);
            }
        }
        best
    }

    /// Whether the underlying class contains the empty string, before assertions.
    pub(crate) fn has_empty_member(&self) -> bool {
        self.program.terminal[0]
    }

    /// Whether matching a class member also requires outside assertions.
    pub(crate) fn has_outside_assertions(&self) -> bool {
        let p = &self.program;
        p.start_anchor
            || p.end_anchor
            || p.start_end_anchor
            || p.end_start_anchor
            || p.start_word.is_some()
            || p.end_word.is_some()
    }

    /// Adds endpoint line predicates to a freshly compiled, uniquely owned plan.
    pub(crate) fn with_endpoint_line_assertions(
        mut self,
        start_end_anchor: bool,
        end_start_anchor: bool,
    ) -> Option<Self> {
        let program = Arc::get_mut(&mut self.program)?;
        program.start_end_anchor = start_end_anchor;
        program.end_start_anchor = end_start_anchor;
        Some(self)
    }

    /// Bounds UTF-16 consumption per member; q-only empty sets return zero.
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
            .saturating_add(
                p.scalars
                    .as_ref()
                    .map_or(0, RegExpUnicodeClassMatcher::search_passes),
            )
            .saturating_add(usize::from(p.start_anchor) + usize::from(p.end_anchor))
            .saturating_add(usize::from(p.start_end_anchor) + usize::from(p.end_start_anchor))
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
        match self.find_with_endpoint_filter(input, start, sticky, |_, _| {
            Ok::<bool, std::convert::Infallible>(true)
        }) {
            Ok(found) => found,
            Err(never) => match never {},
        }
    }

    // A continuation rejects individual complete members before longest-member
    // selection, leaving shorter members and subsequent search starts eligible.
    pub(crate) fn find_with_endpoint_filter<E>(
        &self,
        input: &JsString,
        start: usize,
        sticky: bool,
        mut accept: impl FnMut(usize, usize) -> Result<bool, E>,
    ) -> Result<Option<Range<usize>>, E> {
        let p = &self.program;
        let units = input.code_units();
        let Some(mut candidate) = unicode_start(input, start) else {
            return Ok(None);
        };
        loop {
            let mut node = 0;
            let mut at = candidate;
            let mut best = if p.terminal[0]
                && p.assertions_match(units, candidate, candidate)
                && accept(candidate, candidate)?
            {
                Some(candidate)
            } else {
                None
            };
            for _ in 0..p.max_characters {
                let Some(&first) = units.get(at) else { break };
                let (point, width) = unicode_input_character(units, at, first);
                if at == candidate
                    && p.scalars
                        .as_ref()
                        .is_some_and(|plan| plan.matches_original_character(point))
                    && p.assertions_match(units, candidate, at + width)
                    && accept(candidate, at + width)?
                {
                    best = Some(at + width);
                }
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
                if p.terminal[node]
                    && p.assertions_match(units, candidate, at)
                    && accept(candidate, at)?
                {
                    best = Some(at)
                }
            }
            if let Some(end) = best {
                return Ok(Some(candidate..end));
            }
            if sticky {
                return Ok(None);
            }
            let Some(&first) = units.get(candidate) else {
                return Ok(None);
            };
            candidate += unicode_input_character(units, candidate, first).1;
        }
    }
}

enum WordEvent {
    Point(u32),
    End,
}

fn mark_terminal_node<E>(
    node: Option<usize>,
    terminal: &[bool],
    marked: &mut [bool],
    matched: &mut Vec<usize>,
    mut charge: impl FnMut(usize) -> Result<(), E>,
) -> Result<(), RegExpUnicodeSetCompileError<E>> {
    let Some(node) = node.filter(|&node| terminal[node] && !marked[node]) else {
        return Ok(());
    };
    let needed = matched
        .len()
        .checked_add(1)
        .ok_or(RegExpUnicodeSetCompileError::Capacity)?;
    let movement = if needed > matched.capacity() {
        matched.len()
    } else {
        0
    };
    charge(movement.saturating_add(2)).map_err(RegExpUnicodeSetCompileError::Work)?;
    matched
        .try_reserve(1)
        .map_err(|_| RegExpUnicodeSetCompileError::Capacity)?;
    matched.push(node);
    marked[node] = true;
    Ok(())
}

fn scalar_operand<E>(
    units: &[u16],
    at: &mut usize,
    ignore_case: bool,
    charge: impl FnMut(usize) -> Result<(), E>,
) -> Result<Option<RegExpUnicodeClassMatcher>, RegExpUnicodeSetCompileError<E>> {
    let begin = *at;
    let end;
    let matcher = if units.get(begin) == Some(&0x5b) {
        let Some(class_end) = crate::regexp_unicode_class_literal::class_end(units, begin) else {
            return Ok(None);
        };
        let mut finite_end = begin;
        if finite_operand(units, &mut finite_end, |_| {
            Ok::<(), RegExpUnicodeSetCompileError<E>>(())
        })?
        .is_some()
        {
            return Ok(None);
        }
        end = class_end;
        RegExpUnicodeClassMatcher::compile_scalar_class_units_with_work(
            &units[begin..end],
            ignore_case,
            charge,
        )?
    } else {
        let Some(tail) = units.get(begin..) else {
            return Ok(None);
        };
        end = match tail.get(..2) {
            Some([0x5c, 0x64 | 0x44 | 0x73 | 0x53 | 0x77 | 0x57]) => begin + 2,
            Some([0x5c, 0x70 | 0x50]) if tail.get(2) == Some(&0x7b) => {
                let Some(close) = tail.iter().position(|&unit| unit == 0x7d) else {
                    return Ok(None);
                };
                begin + close + 1
            }
            _ => return Ok(None),
        };
        RegExpUnicodeClassMatcher::compile_scalar_atom_units_with_work(
            &units[begin..end],
            ignore_case,
            charge,
        )?
    };
    if matcher.is_some() {
        *at = end;
    }
    Ok(matcher)
}

// These lexers borrow a prepaid complete source. Callback state owns all trie
// and word-set work; positive nesting uses only a counter, never recursion.
fn q_words<E>(
    units: &[u16],
    at: &mut usize,
    emit: &mut impl FnMut(WordEvent) -> Result<(), RegExpUnicodeSetCompileError<E>>,
) -> Result<Option<()>, RegExpUnicodeSetCompileError<E>> {
    *at += 3;
    loop {
        let Some(&unit) = units.get(*at) else {
            return Ok(None);
        };
        if matches!(unit, 0x7c | 0x7d) {
            emit(WordEvent::End)?;
            *at += 1;
            if unit == 0x7d {
                return Ok(Some(()));
            }
        } else {
            let Some(point) = single_string_character(units, at) else {
                return Ok(None);
            };
            emit(WordEvent::Point(point))?;
        }
    }
}

fn finite_operand<E>(
    units: &[u16],
    at: &mut usize,
    mut emit: impl FnMut(WordEvent) -> Result<(), RegExpUnicodeSetCompileError<E>>,
) -> Result<Option<bool>, RegExpUnicodeSetCompileError<E>> {
    if units
        .get(*at..)
        .is_some_and(|tail| tail.starts_with(&[0x5c, 0x71, 0x7b]))
    {
        return q_words(units, at, &mut emit).map(|parsed| parsed.map(|()| false));
    }
    if units.get(*at) == Some(&0x5b) {
        *at += 1;
        let mut depth = 1usize;
        if units.get(*at) == Some(&0x5e) {
            return Ok(None);
        }
        while depth != 0 {
            let Some(&unit) = units.get(*at) else {
                return Ok(None);
            };
            match unit {
                0x5b => {
                    depth = depth
                        .checked_add(1)
                        .ok_or(RegExpUnicodeSetCompileError::Capacity)?;
                    *at += 1;
                    if units.get(*at) == Some(&0x5e) {
                        return Ok(None);
                    }
                }
                0x5d => {
                    depth -= 1;
                    *at += 1;
                }
                _ if units[*at..].starts_with(&[0x5c, 0x71, 0x7b]) => {
                    if q_words(units, at, &mut emit)?.is_none() {
                        return Ok(None);
                    }
                }
                _ => {
                    if StringSetOperator::at(units, *at).is_some() {
                        return Ok(None);
                    }
                    let Some(point) = single_string_character(units, at) else {
                        return Ok(None);
                    };
                    emit(WordEvent::Point(point))?;
                    emit(WordEvent::End)?;
                }
            }
        }
    } else {
        if StringSetOperator::at(units, *at).is_some() {
            return Ok(None);
        }
        let Some(point) = single_string_character(units, at) else {
            return Ok(None);
        };
        emit(WordEvent::Point(point))?;
        emit(WordEvent::End)?;
    }
    Ok(Some(true))
}

pub(crate) struct Framing<'a> {
    pub(crate) body: &'a [u16],
    pub(crate) start_anchor: bool,
    pub(crate) end_anchor: bool,
    pub(crate) start_word: Option<bool>,
    pub(crate) end_word: Option<bool>,
}

pub(crate) fn frame_source<'a, E>(
    source: &'a JsString,
    charge: &mut impl FnMut(usize) -> Result<(), E>,
) -> Result<Option<Framing<'a>>, RegExpUnicodeSetCompileError<E>> {
    frame_units(source.code_units(), charge)
}

pub(crate) fn frame_units<'a, E>(
    units: &'a [u16],
    charge: &mut impl FnMut(usize) -> Result<(), E>,
) -> Result<Option<Framing<'a>>, RegExpUnicodeSetCompileError<E>> {
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
    Ok(Some(Framing {
        body,
        start_anchor,
        end_anchor,
        start_word,
        end_word,
    }))
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum StringSetOperator {
    Intersection,
    Subtraction,
}

impl StringSetOperator {
    fn at(units: &[u16], at: usize) -> Option<Self> {
        let tail = units.get(at..)?;
        if tail.starts_with(&[0x26, 0x26]) {
            Some(Self::Intersection)
        } else if tail.starts_with(&[0x2d, 0x2d]) {
            Some(Self::Subtraction)
        } else {
            None
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Domain {
    Single,
    Strings,
    Characters,
    Scalars,
    PositiveNested,
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

    fn apply_operation(
        &mut self,
        operator: StringSetOperator,
        members: &mut Vec<(usize, usize)>,
        marked: &mut [bool],
        matched: &mut Vec<usize>,
        active: &mut usize,
    ) -> Result<(), RegExpUnicodeSetCompileError<E>> {
        match operator {
            StringSetOperator::Subtraction => {
                (self.charge)(matched.len()).map_err(RegExpUnicodeSetCompileError::Work)?;
                for &node in matched.iter() {
                    self.terminal[node] = false;
                }
                *active -= matched.len();
            }
            StringSetOperator::Intersection if matched.len() != *active => {
                (self.charge)(members.len().saturating_mul(2))
                    .map_err(RegExpUnicodeSetCompileError::Work)?;
                members.retain(|&(node, _)| {
                    let keep = self.terminal[node] && marked[node];
                    self.terminal[node] = keep;
                    keep
                });
                *active = matched.len();
            }
            StringSetOperator::Intersection => {}
        }
        (self.charge)(matched.len()).map_err(RegExpUnicodeSetCompileError::Work)?;
        for &node in matched.iter() {
            marked[node] = false;
        }
        matched.clear();
        Ok(())
    }

    fn finish_left_member(
        &mut self,
        members: &mut Vec<(usize, usize)>,
    ) -> Result<(), RegExpUnicodeSetCompileError<E>> {
        if !self.terminal[self.node] {
            let needed = members
                .len()
                .checked_add(1)
                .ok_or(RegExpUnicodeSetCompileError::Capacity)?;
            let movement = if needed > members.capacity() {
                members.len()
            } else {
                0
            };
            (self.charge)(movement.saturating_add(1))
                .map_err(RegExpUnicodeSetCompileError::Work)?;
            members
                .try_reserve(1)
                .map_err(|_| RegExpUnicodeSetCompileError::Capacity)?;
            members.push((self.node, self.length));
        }
        self.finish_member()
    }

    fn probe(
        &mut self,
        node: Option<usize>,
        point: u32,
    ) -> Result<Option<usize>, RegExpUnicodeSetCompileError<E>> {
        let point = if self.ignore_case {
            (self.charge)(2 + (usize::BITS - SIMPLE_CASE_FOLD.len().leading_zeros()) as usize)
                .map_err(RegExpUnicodeSetCompileError::Work)?;
            regexp_canonicalize_character(point, true, true)
        } else {
            point
        };
        (self.charge)(1).map_err(RegExpUnicodeSetCompileError::Work)?;
        Ok(node.and_then(|node| self.edges.get(&(node, point)).copied()))
    }

    fn finish_probe_member(
        &mut self,
        node: Option<usize>,
        length: usize,
        marked: &mut [bool],
        matched: &mut Vec<usize>,
    ) -> Result<(), RegExpUnicodeSetCompileError<E>> {
        (self.charge)(1).map_err(RegExpUnicodeSetCompileError::Work)?;
        self.has_strings |= length != 1;
        mark_terminal_node(node, &self.terminal, marked, matched, &mut self.charge)
    }

    fn mark_scalar_members(
        &mut self,
        scalar: &RegExpUnicodeClassMatcher,
        marked: &mut [bool],
        matched: &mut Vec<usize>,
    ) -> Result<(), RegExpUnicodeSetCompileError<E>> {
        // HashMap iteration visits capacity, and scalar lookup includes folding.
        // Root edges represent exactly the one-character words, never empty or
        // longer members, including unmatchable separate braced surrogates.
        (self.charge)(
            self.edges
                .capacity()
                .saturating_mul(scalar.search_passes().saturating_add(2)),
        )
        .map_err(RegExpUnicodeSetCompileError::Work)?;
        for (&(parent, point), &node) in &self.edges {
            if parent == 0 && self.terminal[node] && scalar.matches_original_character(point) {
                mark_terminal_node(
                    Some(node),
                    &self.terminal,
                    marked,
                    matched,
                    &mut self.charge,
                )?;
            }
        }
        Ok(())
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
        ) && (!self.start_end_anchor
            || unicode_assertions_match(units, start, start, false, true, self.multiline))
            && (!self.end_start_anchor
                || unicode_assertions_match(units, end, end, true, false, self.multiline))
            && self.start_word.is_none_or(|wanted| {
                unicode_word_boundary(units, start, self.ignore_case) == wanted
            })
            && self
                .end_word
                .is_none_or(|wanted| unicode_word_boundary(units, end, self.ignore_case) == wanted)
    }
}

#[cfg(test)]
#[path = "regexp_unicode_string_set_tests.rs"]
mod tests;
