//! Patterns blocked by an independently always-failing assertion (22.2.2.4).

use crate::{
    JsString, RegExpZeroCountSequenceMatcher, regexp_zero_count_group::enclosing_group_body,
    regexp_zero_count_sequence::quantified_enclosing_scope,
};
use std::ops::Range;

/// A validated root sequence containing an independently always-failing assertion.
///
/// The complete original Pattern/flags must be validated and named capture
/// heads normalized first. A negative assertion whose complete body always
/// has a match, or a positive assertion over an independently impossible body,
/// must fail at every position (22.2.2.4). Other validated sequence terms cannot
/// rescue the failure; no matching data is needed.
#[derive(Clone, Debug)]
pub struct RegExpImpossibleMatcher {
    captures: usize,
}

impl RegExpImpossibleMatcher {
    /// Proves a root sequence blocked by a negative assertion over an empty tree.
    ///
    /// Root choices and assertions inside enclosing groups require other proofs.
    /// A flat mode-aware scan counts every original capture. Projected assertion
    /// bodies keep original bindings; only independently proved empty bodies
    /// qualify. Source/copy work precedes allocation; retained storage is constant.
    pub fn compile_negative_validated_with_work<E>(
        source: &JsString,
        unicode: bool,
        sets: bool,
        charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, E> {
        Self::compile_assertion_validated_with_work::<0, E>(source, unicode, sets, charge)
    }

    /// Proves a root sequence blocked by a positive assertion over an impossible body.
    ///
    /// Full original validation and normalized capture heads are required.
    /// A positive assertion whose complete body cannot match must fail in either
    /// direction (22.2.2.4). Projected bodies retain original bindings and use
    /// only the independently established negative-empty impossibility proofs.
    pub fn compile_positive_impossible_validated_with_work<E>(
        source: &JsString,
        unicode: bool,
        sets: bool,
        charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, E> {
        Self::compile_assertion_validated_with_work::<1, E>(source, unicode, sets, charge)
    }

    /// Proves root sequences blocked by a negative assertion over a composite empty body.
    ///
    /// Full original validation and normalized capture heads are required. A
    /// complete operand proved empty by a negative-impossible assertion, spine,
    /// sequence or root choice forces the outside negative assertion to fail
    /// in either direction (22.2.2.4). The finite operand chain uses only earlier
    /// constructors and never re-enters this proof. Original slots are retained.
    pub fn compile_composite_negative_validated_with_work<E>(
        source: &JsString,
        unicode: bool,
        sets: bool,
        charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, E> {
        Self::compile_assertion_validated_with_work::<2, E>(source, unicode, sets, charge)
    }

    /// Proves root sequences blocked by a negative assertion over an always-successful group spine.
    ///
    /// Full original validation and normalized capture heads are required.
    /// A complete ordinary/modifier spine with a zero-minimum wrapper always
    /// has a successful path, even when a greedy operand could consume input
    /// (22.2.2.3.1). Its negative assertion must fail (22.2.2.4). This proves
    /// impossibility only; the operand must not be treated as unconditionally
    /// empty or used to fabricate capture masks. Original slots are retained.
    pub fn compile_nullable_group_negative_validated_with_work<E>(
        source: &JsString,
        unicode: bool,
        sets: bool,
        charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, E> {
        Self::compile_assertion_validated_with_work::<3, E>(source, unicode, sets, charge)
    }

    /// Proves root negative assertions over a complete zero-minimum atom or enclosing spine.
    ///
    /// Full original validation and normalized capture heads are required.
    /// A complete safe literal/class/property atom with a zero minimum always
    /// has a successful path, regardless of matching data (22.2.2.3.1).
    /// Ordinary/modifier wrappers preserve that existence proof. The negative
    /// assertion fails (22.2.2.4); greedy consumption is not an empty-match proof.
    /// Mode-aware surrogate boundaries reuse the established tree scanner.
    pub fn compile_nullable_atom_negative_validated_with_work<E>(
        source: &JsString,
        unicode: bool,
        sets: bool,
        charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, E> {
        Self::compile_assertion_validated_with_work::<4, E>(source, unicode, sets, charge)
    }

    /// Proves root negative assertions over complete sequences of independently nullable terms.
    ///
    /// Full original validation and normalized capture heads are required.
    /// Each complete group/atom term has an unconditional successful path;
    /// concatenation and complete enclosing spines preserve that existence
    /// proof (22.2.2.2, 22.2.2.3.1). Greedy terms may consume, but their negative
    /// assertion must fail (22.2.2.4). No operand capture mask is fabricated.
    /// Source/frame/copy work precedes allocation and scans stay source-sized.
    pub fn compile_nullable_sequence_negative_validated_with_work<E>(
        source: &JsString,
        unicode: bool,
        sets: bool,
        charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, E> {
        Self::compile_assertion_validated_with_work::<5, E>(source, unicode, sets, charge)
    }

    /// Proves root negative assertions over choices with an unconditionally successful arm.
    ///
    /// Full original validation and normalized capture heads are required.
    /// One empty or independently nullable arm guarantees a successful operand
    /// path (22.2.2.2), so the negative assertion fails (22.2.2.4). Earlier arms
    /// may consume; their capture values are never exposed. Complete outside
    /// spines preserve existence of a match. Source/copy/frame work precedes
    /// allocation and the flat arm scans remain source-sized.
    pub fn compile_nullable_alternatives_negative_validated_with_work<E>(
        source: &JsString,
        unicode: bool,
        sets: bool,
        charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, E> {
        Self::compile_assertion_validated_with_work::<6, E>(source, unicode, sets, charge)
    }

    /// Proves root negative assertions over a positive assertion with an always-successful operand.
    ///
    /// Full original validation and normalized capture heads are required.
    /// A complete positive assertion over a nullable group, atom, sequence or
    /// choice always succeeds (22.2.2.4), so its outside negative assertion
    /// fails. Required outside ordinary/modifier spines preserve that existence
    /// proof. Operand captures may contain data and are never fabricated.
    /// This finite proof handles one positive assertion layer without recursion.
    pub fn compile_nullable_positive_negative_validated_with_work<E>(
        source: &JsString,
        unicode: bool,
        sets: bool,
        charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, E> {
        Self::compile_assertion_validated_with_work::<7, E>(source, unicode, sets, charge)
    }

    fn compile_assertion_validated_with_work<const MODE: u8, E>(
        source: &JsString,
        unicode: bool,
        sets: bool,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, E> {
        charge(source.len().saturating_mul(8).saturating_add(16))?;
        if sets && !unicode {
            return Ok(None);
        }
        let units = source.code_units();
        let mut cursor = 0;
        let mut depth = 0usize;
        let mut class_depth = 0usize;
        let mut captures = 0;
        let mut assertion_body = None;
        let mut impossible = false;
        while let Some(&unit) = units.get(cursor) {
            if unit == 0x5c {
                cursor += 2;
                continue;
            }
            if class_depth != 0 {
                if unit == 0x5d {
                    class_depth -= 1;
                } else if sets && unit == 0x5b {
                    class_depth += 1;
                }
            } else {
                match unit {
                    0x5b => class_depth = 1,
                    0x28 => {
                        if depth == 0 {
                            let kind = if MODE == 1 { 0x3d } else { 0x21 };
                            assertion_body = if units[cursor..].starts_with(&[0x28, 0x3f, kind]) {
                                Some(cursor + 3)
                            } else if units[cursor..].starts_with(&[0x28, 0x3f, 0x3c, kind]) {
                                Some(cursor + 4)
                            } else {
                                None
                            };
                        }
                        depth += 1;
                        captures += usize::from(units.get(cursor + 1) != Some(&0x3f));
                    }
                    0x29 => {
                        let Some(next) = depth.checked_sub(1) else {
                            return Ok(None);
                        };
                        depth = next;
                        if depth == 0 {
                            if let Some(start) = assertion_body.take().filter(|_| !impossible) {
                                let body = JsString::from_code_units(units[start..cursor].to_vec());
                                impossible |= if MODE == 1 {
                                    Self::negative_impossibility_with_work(
                                        &body,
                                        unicode,
                                        sets,
                                        &mut charge,
                                    )?
                                } else if MODE == 2 {
                                    Self::composite_empty_with_work(
                                        &body,
                                        unicode,
                                        sets,
                                        &mut charge,
                                    )?
                                } else if MODE == 3 {
                                    Self::nullable_group_with_work(
                                        &body,
                                        unicode,
                                        sets,
                                        &mut charge,
                                    )?
                                } else if MODE == 4 {
                                    Self::nullable_atom_with_work(
                                        &body,
                                        unicode,
                                        sets,
                                        &mut charge,
                                    )?
                                } else if MODE == 5 {
                                    Self::nullable_sequence_with_work(
                                        &body,
                                        unicode,
                                        sets,
                                        &mut charge,
                                    )?
                                } else if MODE == 6 {
                                    Self::nullable_alternatives_with_work(
                                        &body,
                                        unicode,
                                        sets,
                                        &mut charge,
                                    )?
                                } else if MODE == 7 {
                                    Self::nullable_positive_with_work(
                                        &body,
                                        unicode,
                                        sets,
                                        &mut charge,
                                    )?
                                } else {
                                    RegExpZeroCountSequenceMatcher::compile_surrogate_tree_validated_with_work(
                                        &body, unicode, sets, &mut charge,
                                    )?.is_some()
                                };
                            }
                        }
                    }
                    0x7c if depth == 0 => return Ok(None),
                    _ => {}
                }
            }
            cursor += 1;
        }
        Ok((impossible && depth == 0 && class_depth == 0).then_some(Self { captures }))
    }

    // This finite proof chain never calls the positive-assertion constructor.
    fn negative_impossibility_with_work<E>(
        body: &JsString,
        unicode: bool,
        sets: bool,
        mut charge: &mut dyn FnMut(usize) -> Result<(), E>,
    ) -> Result<bool, E> {
        if Self::compile_negative_validated_with_work(body, unicode, sets, &mut charge)?.is_some()
            || Self::compile_enclosing_negative_validated_with_work(
                body,
                unicode,
                sets,
                &mut charge,
            )?
            .is_some()
            || Self::compile_alternatives_negative_validated_with_work(
                body,
                unicode,
                sets,
                &mut charge,
            )?
            .is_some()
            || Self::compile_required_enclosing_negative_validated_with_work(
                body, unicode, sets, charge,
            )?
            .is_some()
        {
            return Ok(true);
        }
        Ok(false)
    }

    // Earlier empty constructors use only the direct/positive negative-empty
    // impossibility proofs. Type erasure keeps this finite dependency boundary.
    fn composite_empty_with_work<E>(
        body: &JsString,
        unicode: bool,
        sets: bool,
        mut charge: &mut dyn FnMut(usize) -> Result<(), E>,
    ) -> Result<bool, E> {
        if RegExpZeroCountSequenceMatcher::compile_negative_impossible_validated_with_work(
            body, unicode, sets, &mut charge,
        )?.is_some()
            || RegExpZeroCountSequenceMatcher::compile_enclosing_negative_impossible_validated_with_work(
                body, unicode, sets, &mut charge,
            )?.is_some()
            || RegExpZeroCountSequenceMatcher::compile_first_negative_impossible_validated_with_work(
                body, unicode, sets, &mut charge,
            )?.is_some()
            || RegExpZeroCountSequenceMatcher::compile_negative_impossible_sequence_validated_with_work(
                body, unicode, sets, &mut charge,
            )?.is_some()
            || RegExpZeroCountSequenceMatcher::compile_enclosing_negative_impossible_sequence_validated_with_work(
                body, unicode, sets, charge,
            )?.is_some()
        {return Ok(true)}
        Ok(false)
    }

    // Existence of a successful path is sufficient for a negative assertion;
    // greedy consumption and operand capture values are never observed.
    fn nullable_group_with_work<E>(
        body: &JsString,
        unicode: bool,
        sets: bool,
        charge: &mut dyn FnMut(usize) -> Result<(), E>,
    ) -> Result<bool, E> {
        charge(body.len().saturating_mul(32).saturating_add(16))?;
        if sets && !unicode {
            return Ok(false);
        }
        Ok(quantified_enclosing_scope(body.code_units(), sets).is_some_and(|scope| scope.optional))
    }

    fn nullable_atom_with_work<E>(
        body: &JsString,
        unicode: bool,
        sets: bool,
        charge: &mut dyn FnMut(usize) -> Result<(), E>,
    ) -> Result<bool, E> {
        charge(body.len().saturating_mul(32).saturating_add(16))?;
        if sets && !unicode {
            return Ok(false);
        }
        let units = body.code_units();
        let atom = if let Some(scope) = quantified_enclosing_scope(units, sets) {
            &units[scope.body]
        } else {
            units
        };
        let Some(end) = crate::regexp_zero_count_tree::surrogate_atom_end(atom, 0, unicode, sets)
        else {
            return Ok(false);
        };
        let Some((end, minimum_zero, _)) = crate::regexp_zero_count_spine::quantifier(atom, end)
        else {
            return Ok(false);
        };
        Ok(minimum_zero && end == atom.len())
    }

    fn nullable_sequence_with_work<E>(
        body: &JsString,
        unicode: bool,
        sets: bool,
        mut charge: &mut dyn FnMut(usize) -> Result<(), E>,
    ) -> Result<bool, E> {
        charge(body.len().saturating_mul(48).saturating_add(16))?;
        if sets && !unicode {
            return Ok(false);
        }
        let units = body.code_units();
        let units = if let Some(scope) = quantified_enclosing_scope(units, sets) {
            &units[scope.body]
        } else {
            units
        };
        let mut cursor = 0;
        let mut terms = 0;
        while cursor < units.len() {
            let end = if units[cursor] == 0x28 {
                crate::regexp_zero_count_sequence::group_end(units, cursor, sets)
            } else {
                crate::regexp_zero_count_tree::surrogate_atom_end(units, cursor, unicode, sets)
            };
            let Some(end) = end else { return Ok(false) };
            let Some((end, _, _)) = crate::regexp_zero_count_spine::quantifier(units, end) else {
                return Ok(false);
            };
            let term = JsString::from_code_units(units[cursor..end].to_vec());
            if !Self::nullable_group_with_work(&term, unicode, sets, &mut charge)?
                && !Self::nullable_atom_with_work(&term, unicode, sets, &mut charge)?
            {
                return Ok(false);
            }
            cursor = end;
            terms += 1;
        }
        Ok(terms >= 2)
    }

    fn nullable_alternatives_with_work<E>(
        body: &JsString,
        unicode: bool,
        sets: bool,
        mut charge: &mut dyn FnMut(usize) -> Result<(), E>,
    ) -> Result<bool, E> {
        charge(body.len().saturating_mul(48).saturating_add(16))?;
        if sets && !unicode {
            return Ok(false);
        }
        let units = body.code_units();
        let units = if let Some(scope) = quantified_enclosing_scope(units, sets) {
            &units[scope.body]
        } else {
            units
        };
        let mut ranges = Vec::new();
        let mut start = 0;
        let mut cursor = 0;
        let mut depth = 0usize;
        let mut class_depth = 0usize;
        while let Some(&unit) = units.get(cursor) {
            if unit == 0x5c {
                cursor += 2;
                continue;
            }
            if class_depth != 0 {
                if unit == 0x5d {
                    class_depth -= 1
                } else if sets && unit == 0x5b {
                    class_depth += 1
                }
            } else {
                match unit {
                    0x5b => class_depth = 1,
                    0x28 => depth += 1,
                    0x29 => {
                        let Some(next) = depth.checked_sub(1) else {
                            return Ok(false);
                        };
                        depth = next;
                    }
                    0x7c if depth == 0 => {
                        ranges.push(start..cursor);
                        start = cursor + 1;
                    }
                    _ => {}
                }
            }
            cursor += 1;
        }
        if depth != 0 || class_depth != 0 || ranges.is_empty() {
            return Ok(false);
        }
        ranges.push(start..units.len());
        for range in ranges {
            if range.is_empty() {
                return Ok(true);
            }
            let arm = JsString::from_code_units(units[range].to_vec());
            if Self::nullable_group_with_work(&arm, unicode, sets, &mut charge)?
                || Self::nullable_atom_with_work(&arm, unicode, sets, &mut charge)?
                || Self::nullable_sequence_with_work(&arm, unicode, sets, &mut charge)?
            {
                return Ok(true);
            }
        }
        Ok(false)
    }

    fn nullable_positive_with_work<E>(
        body: &JsString,
        unicode: bool,
        sets: bool,
        mut charge: &mut dyn FnMut(usize) -> Result<(), E>,
    ) -> Result<bool, E> {
        charge(body.len().saturating_mul(48).saturating_add(16))?;
        if sets && !unicode {
            return Ok(false);
        }
        let units = body.code_units();
        let units = if let Some(scope) = quantified_enclosing_scope(units, sets) {
            &units[scope.body]
        } else {
            units
        };
        let start = if units.starts_with(&[0x28, 0x3f, 0x3d]) {
            3
        } else if units.starts_with(&[0x28, 0x3f, 0x3c, 0x3d]) {
            4
        } else {
            return Ok(false);
        };
        if crate::regexp_zero_count_sequence::group_end(units, 0, sets) != Some(units.len()) {
            return Ok(false);
        }
        let operand = JsString::from_code_units(units[start..units.len() - 1].to_vec());
        Ok(
            Self::nullable_group_with_work(&operand, unicode, sets, &mut charge)?
                || Self::nullable_atom_with_work(&operand, unicode, sets, &mut charge)?
                || Self::nullable_sequence_with_work(&operand, unicode, sets, &mut charge)?
                || Self::nullable_alternatives_with_work(&operand, unicode, sets, charge)?,
        )
    }

    /// Full original lexical capture count, including unobservable slots.
    pub fn capture_count(&self) -> usize {
        self.captures
    }

    /// Proves mandatory ordinary groups enclosing an impossible root sequence.
    ///
    /// Full original validation and normalized capture heads are required.
    /// The flat scope proof excludes quantified/modifier wrappers and outside
    /// terms. Every enclosing capture remains in the original inventory even
    /// though no match can expose it. References keep their original bindings.
    pub fn compile_enclosing_negative_validated_with_work<E>(
        source: &JsString,
        unicode: bool,
        sets: bool,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, E> {
        charge(source.len().saturating_mul(8).saturating_add(16))?;
        if sets && !unicode {
            return Ok(None);
        }
        let Some(scope) = enclosing_group_body(source.code_units(), sets) else {
            return Ok(None);
        };
        let body = JsString::from_code_units(source.code_units()[scope.body].to_vec());
        Ok(
            Self::compile_negative_validated_with_work(&body, unicode, sets, charge)?.map(|body| {
                Self {
                    captures: scope.captures + body.captures,
                }
            }),
        )
    }

    /// Proves a root choice only when every alternative is independently impossible.
    ///
    /// Requires full original validation and normalized capture heads. Empty or
    /// unproved alternatives prevent this proof. Each projected arm retains its
    /// original binding context and uses the direct or mandatory-enclosing proof.
    /// Source/copy work precedes allocation; branch scans remain source-sized.
    pub fn compile_alternatives_negative_validated_with_work<E>(
        source: &JsString,
        unicode: bool,
        sets: bool,
        charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, E> {
        Self::compile_alternatives_assertion_validated_with_work::<false, E>(
            source, unicode, sets, charge,
        )
    }

    /// Proves root choices whose every arm independently has a composite-negative failure.
    ///
    /// Full original validation and normalized capture heads are required.
    /// Direct or required-enclosing composite-negative arms keep their original
    /// bindings and lexical inventories. Empty, possible or unproved arms reject
    /// the proof (22.2.2.2). Source/copy work precedes allocation, using the same
    /// flat separator scan as the earlier negative-empty choice constructor.
    pub fn compile_alternatives_composite_negative_validated_with_work<E>(
        source: &JsString,
        unicode: bool,
        sets: bool,
        charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, E> {
        Self::compile_alternatives_assertion_validated_with_work::<true, E>(
            source, unicode, sets, charge,
        )
    }

    fn compile_alternatives_assertion_validated_with_work<const COMPOSITE: bool, E>(
        source: &JsString,
        unicode: bool,
        sets: bool,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, E> {
        charge(source.len().saturating_mul(8).saturating_add(16))?;
        if sets && !unicode {
            return Ok(None);
        }
        let units = source.code_units();
        let mut ranges = Vec::new();
        let mut start = 0;
        let mut cursor = 0;
        let mut depth = 0usize;
        let mut class_depth = 0usize;
        while let Some(&unit) = units.get(cursor) {
            if unit == 0x5c {
                cursor += 2;
                continue;
            }
            if class_depth != 0 {
                if unit == 0x5d {
                    class_depth -= 1;
                } else if sets && unit == 0x5b {
                    class_depth += 1;
                }
            } else {
                match unit {
                    0x5b => class_depth = 1,
                    0x28 => depth += 1,
                    0x29 => {
                        let Some(next) = depth.checked_sub(1) else {
                            return Ok(None);
                        };
                        depth = next;
                    }
                    0x7c if depth == 0 => {
                        ranges.push(start..cursor);
                        start = cursor + 1;
                    }
                    _ => {}
                }
            }
            cursor += 1;
        }
        if ranges.is_empty() || depth != 0 || class_depth != 0 {
            return Ok(None);
        }
        ranges.push(start..units.len());
        let mut captures = 0;
        for range in ranges {
            if range.is_empty() {
                return Ok(None);
            }
            let arm = JsString::from_code_units(units[range].to_vec());
            let mut plan = if COMPOSITE {
                Self::compile_composite_negative_validated_with_work(
                    &arm,
                    unicode,
                    sets,
                    &mut charge,
                )?
            } else {
                Self::compile_negative_validated_with_work(&arm, unicode, sets, &mut charge)?
            };
            if plan.is_none() {
                plan = if COMPOSITE {
                    Self::compile_wrapped_composite_negative_validated_with_work(
                        &arm,
                        unicode,
                        sets,
                        &mut charge,
                    )?
                } else {
                    Self::compile_enclosing_negative_validated_with_work(
                        &arm,
                        unicode,
                        sets,
                        &mut charge,
                    )?
                };
            }
            let Some(plan) = plan else {
                return Ok(None);
            };
            captures += plan.captures;
        }
        Ok(Some(Self { captures }))
    }

    /// Proves required ordinary/modifier spines enclosing impossible sequences or choices.
    ///
    /// Full original validation and normalized capture heads are required.
    /// Every outside minimum must be positive: an optional wrapper could skip
    /// the impossible body. Bounds stay symbolic and modifier flags cannot
    /// rescue a body blocked at every position (22.2.2.3.1, 22.2.2.4).
    pub fn compile_required_enclosing_negative_validated_with_work<E>(
        source: &JsString,
        unicode: bool,
        sets: bool,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, E> {
        charge(source.len().saturating_mul(32).saturating_add(16))?;
        if sets && !unicode {
            return Ok(None);
        }
        let Some(scope) = quantified_enclosing_scope(source.code_units(), sets) else {
            return Ok(None);
        };
        if scope.optional {
            return Ok(None);
        }
        let body = JsString::from_code_units(source.code_units()[scope.body].to_vec());
        let mut plan =
            Self::compile_negative_validated_with_work(&body, unicode, sets, &mut charge)?;
        if plan.is_none() {
            plan = Self::compile_alternatives_negative_validated_with_work(
                &body, unicode, sets, charge,
            )?;
        }
        Ok(plan.map(|plan| Self {
            captures: scope.captures + plan.captures,
        }))
    }

    /// Proves required ordinary/modifier spines enclosing a composite-negative failure.
    ///
    /// Full original validation and normalized capture heads are required.
    /// Every outside minimum must be positive: optional wrappers can skip the
    /// impossible body (22.2.2.3.1). Bounds remain symbolic; original references
    /// and full lexical inventories are preserved. The flat scope scan and
    /// projected body proof charge source work before allocation.
    pub fn compile_wrapped_composite_negative_validated_with_work<E>(
        source: &JsString,
        unicode: bool,
        sets: bool,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, E> {
        charge(source.len().saturating_mul(32).saturating_add(16))?;
        if sets && !unicode {
            return Ok(None);
        }
        let Some(scope) = quantified_enclosing_scope(source.code_units(), sets) else {
            return Ok(None);
        };
        if scope.optional {
            return Ok(None);
        }
        let body = JsString::from_code_units(source.code_units()[scope.body].to_vec());
        Ok(
            Self::compile_composite_negative_validated_with_work(&body, unicode, sets, charge)?
                .map(|body| Self {
                    captures: scope.captures + body.captures,
                }),
        )
    }

    /// Proves required ordinary/modifier spines around impossible composite alternatives.
    ///
    /// Full original validation and normalized capture heads are required.
    /// Every enclosing minimum must be positive; the complete body must be a
    /// root choice whose every arm independently proves a composite-negative
    /// failure (22.2.2.2, 22.2.2.3.1). Original bindings and full inventories
    /// remain intact. Source/frame/copy work precedes allocation; the finite
    /// proof chain never recursively processes outside wrappers.
    pub fn compile_enclosing_composite_alternatives_validated_with_work<E>(
        source: &JsString,
        unicode: bool,
        sets: bool,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, E> {
        charge(source.len().saturating_mul(32).saturating_add(16))?;
        if sets && !unicode {
            return Ok(None);
        }
        let Some(scope) = quantified_enclosing_scope(source.code_units(), sets) else {
            return Ok(None);
        };
        if scope.optional {
            return Ok(None);
        }
        let body = JsString::from_code_units(source.code_units()[scope.body].to_vec());
        Ok(
            Self::compile_alternatives_composite_negative_validated_with_work(
                &body, unicode, sets, charge,
            )?
            .map(|body| Self {
                captures: scope.captures + body.captures,
            }),
        )
    }

    /// Proves required ordinary/modifier spines around a nullable-operand negative failure.
    ///
    /// Full original validation and normalized capture heads are required.
    /// Every outside minimum must be positive; optional wrappers could skip the
    /// failing body (22.2.2.3.1). The body uses only the finite nullable group,
    /// atom, sequence, choice or positive-assertion negative proofs. Original
    /// bindings, symbolic bounds and full inventories remain intact. Source,
    /// frame and copy work precedes allocation; retained storage stays constant.
    pub fn compile_wrapped_nullable_negative_validated_with_work<E>(
        source: &JsString,
        unicode: bool,
        sets: bool,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, E> {
        charge(source.len().saturating_mul(32).saturating_add(16))?;
        if sets && !unicode {
            return Ok(None);
        }
        let Some(scope) = quantified_enclosing_scope(source.code_units(), sets) else {
            return Ok(None);
        };
        if scope.optional {
            return Ok(None);
        }
        let body = JsString::from_code_units(source.code_units()[scope.body].to_vec());
        let mut plan = Self::compile_nullable_group_negative_validated_with_work(
            &body,
            unicode,
            sets,
            &mut charge,
        )?;
        if plan.is_none() {
            plan = Self::compile_nullable_atom_negative_validated_with_work(
                &body,
                unicode,
                sets,
                &mut charge,
            )?;
        }
        if plan.is_none() {
            plan = Self::compile_nullable_sequence_negative_validated_with_work(
                &body,
                unicode,
                sets,
                &mut charge,
            )?;
        }
        if plan.is_none() {
            plan = Self::compile_nullable_alternatives_negative_validated_with_work(
                &body,
                unicode,
                sets,
                &mut charge,
            )?;
        }
        if plan.is_none() {
            plan = Self::compile_nullable_positive_negative_validated_with_work(
                &body, unicode, sets, charge,
            )?;
        }
        Ok(plan.map(|body| Self {
            captures: scope.captures + body.captures,
        }))
    }

    /// Returns no match, charging actual work before any possible input access.
    pub fn find_with_work<E>(
        &self,
        _input: &JsString,
        _start: usize,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Range<usize>>, E> {
        charge(1)?;
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn compile(source: &str, unicode: bool, sets: bool) -> Option<RegExpImpossibleMatcher> {
        RegExpImpossibleMatcher::compile_negative_validated_with_work(
            &JsString::from(source),
            unicode,
            sets,
            |_| Ok::<(), ()>(()),
        )
        .unwrap()
    }

    #[test]
    fn negative_empty_assertions_block_arbitrary_root_terms_and_count_every_capture() {
        for (source, count) in [
            (r"(?!)", 0),
            (r"(?<!a{0})", 0),
            (r"a(?!(b{0}))c", 1),
            (r"[\p{RGI_Emoji}]+(?!(a{0}))\1", 1),
            (r"(?<!(\uD83D\uDE00{0}))(\p{RGI_Emoji})+", 2),
            (r"(a|b)(?!(?=(c{0}))(?<=(d{0})))(e)", 4),
            (r"[[\q{a|ab}]&&\p{RGI_Emoji}](?!(\uD800{0}))", 1),
            (r"\((?!(a{0}))\)", 1),
        ] {
            assert_eq!(
                compile(source, true, true).unwrap().capture_count(),
                count,
                "{source}"
            );
        }
        assert!(compile("(?!a{0})", false, false).is_some());
    }

    #[test]
    fn root_choices_nested_assertions_and_conditional_bodies_need_other_proofs() {
        for source in [
            "",
            "(?=a{0})",
            "(?!a)",
            "(?<!a)",
            "(?!)|a{0}",
            "a{0}|(?!)",
            "(?:(?!a{0}))?",
            "(?:(?!a{0}))",
            "[()?!]",
        ] {
            assert!(compile(source, true, true).is_none(), "{source}");
        }
        assert!(compile("(?!)", false, true).is_none());
    }

    #[test]
    fn deep_prefixes_and_many_assertions_have_flat_source_work_and_constant_retained_state() {
        let source = format!(
            "{}a{}(?!(b{{0}}))",
            "(".repeat(100_000),
            ")".repeat(100_000)
        );
        let plan = compile(&source, true, true).unwrap();
        assert_eq!(plan.capture_count(), 100_001);
        let source = "(?!a{0})".repeat(20_000);
        let mut work = 0;
        assert!(
            RegExpImpossibleMatcher::compile_negative_validated_with_work(
                &JsString::from(source.as_str()),
                true,
                true,
                |n| {
                    work += n;
                    Ok::<(), ()>(())
                }
            )
            .unwrap()
            .is_some()
        );
        assert!(work <= source.len() * 9 + 256);
    }

    #[test]
    fn constructor_abort_propagates_before_source_and_each_projected_body_step() {
        let source = JsString::from("(?!a)(?!(b{0}))(c)");
        let mut work = Vec::new();
        assert!(
            RegExpImpossibleMatcher::compile_negative_validated_with_work(
                &source,
                true,
                true,
                |n| {
                    work.push(n);
                    Ok::<(), ()>(())
                }
            )
            .unwrap()
            .is_some()
        );
        assert_eq!(work.len(), 3);
        assert_eq!(work[0], source.len() * 8 + 16);
        for stop in 0..work.len() {
            let mut seen = Vec::new();
            assert!(
                matches!(RegExpImpossibleMatcher::compile_negative_validated_with_work(
                &source, true, true, |n| {seen.push(n); if seen.len()==stop+1 {Err(stop)}else{Ok(())}}
            ), Err(n) if n==stop)
            );
            assert_eq!(seen, work[..=stop]);
        }
    }

    #[test]
    fn no_match_work_is_constant_and_aborts_before_any_input_access() {
        let plan = compile(r"[\p{RGI_Emoji}]+(?!(a{0}))", true, true).unwrap();
        for input in ["", "a😀ab!", "a\r\nb"] {
            let input = JsString::from(input);
            for start in [0, 1, input.len(), input.len() + 1, usize::MAX] {
                let mut work = 0;
                assert_eq!(
                    plan.find_with_work(&input, start, |n| {
                        work += n;
                        Ok::<(), ()>(())
                    }),
                    Ok(None)
                );
                assert_eq!(work, 1);
                assert_eq!(plan.find_with_work(&input, start, |_| Err(53)), Err(53));
            }
        }
    }

    fn enclosing(source: &str) -> Option<RegExpImpossibleMatcher> {
        RegExpImpossibleMatcher::compile_enclosing_negative_validated_with_work(
            &JsString::from(source),
            true,
            true,
            |_| Ok::<(), ()>(()),
        )
        .unwrap()
    }

    #[test]
    fn mandatory_enclosing_impossible_groups_keep_all_original_capture_slots() {
        for (source, count) in [
            (r"([\p{RGI_Emoji}]+(?!(a{0})))", 2),
            (r"(?:((?<!\uD800{0})(a)))", 2),
            (r"((a|b)(?!(c{0}))(d))", 4),
            (r"(((?!(\2){0})[\p{RGI_Emoji}]+))", 3),
        ] {
            assert_eq!(
                enclosing(source).unwrap().capture_count(),
                count,
                "{source}"
            );
        }
    }

    #[test]
    fn enclosing_no_match_scope_excludes_optional_groups_siblings_and_outer_choices() {
        for source in [
            "(?!a{0})",
            "((?!a{0}))?",
            "((?!a{0})){0}",
            "(?i:(?!a{0}))",
            "((?!a{0}))|b{0}",
            "((?!a{0}))(x)",
            "x((?!a{0}))",
        ] {
            assert!(enclosing(source).is_none(), "{source}");
        }
    }

    #[test]
    fn hundred_thousand_enclosing_impossible_groups_need_no_native_recursion() {
        let source = format!(
            "{}a(?!(b{{0}})){}",
            "(".repeat(100_000),
            ")".repeat(100_000)
        );
        assert_eq!(enclosing(&source).unwrap().capture_count(), 100_001);
    }

    #[test]
    fn enclosing_impossible_work_aborts_before_each_projection_or_operand_step() {
        let source = JsString::from(r"(([\p{RGI_Emoji}]+(?!(a{0}))))");
        let mut work = Vec::new();
        assert!(
            RegExpImpossibleMatcher::compile_enclosing_negative_validated_with_work(
                &source,
                true,
                true,
                |n| {
                    work.push(n);
                    Ok::<(), ()>(())
                }
            )
            .unwrap()
            .is_some()
        );
        assert_eq!(work.len(), 3);
        for stop in 0..work.len() {
            let mut seen = Vec::new();
            assert!(
                matches!(RegExpImpossibleMatcher::compile_enclosing_negative_validated_with_work(
                &source, true, true, |n| {seen.push(n); if seen.len()==stop+1 {Err(stop)}else{Ok(())}}
            ), Err(n) if n==stop)
            );
            assert_eq!(seen, work[..=stop]);
        }
    }

    fn alternatives(source: &str) -> Option<RegExpImpossibleMatcher> {
        RegExpImpossibleMatcher::compile_alternatives_negative_validated_with_work(
            &JsString::from(source),
            true,
            true,
            |_| Ok::<(), ()>(()),
        )
        .unwrap()
    }

    #[test]
    fn impossible_alternatives_reserve_all_original_slots_without_rebinding_arms() {
        for (source, captures) in [
            (r"(?!)|(?<!a{0})", 0),
            (r"[\p{RGI_Emoji}]+(?!(a{0}))|((?<!b{0})(c))", 3),
            (r"((?!(\3){0})[\p{RGI_Emoji}]+)|(?<!(\uD800{0}))(a)", 4),
            (r"\|(?!(a{0}))|[\q{a|ab}](?<!b{0})", 1),
        ] {
            assert_eq!(
                alternatives(source).unwrap().capture_count(),
                captures,
                "{source}"
            );
        }
    }

    #[test]
    fn any_empty_possible_or_unproved_alternative_prevents_no_match_proof() {
        for source in [
            "(?!)",
            "(?!)|a{0}",
            "a|(?!)",
            "(?!)|",
            "|(?!)",
            "(?!)|(?:(?!a{0}))?",
            "(?!(a))|(?!)",
            "(?:(?!)|a{0})",
        ] {
            assert!(alternatives(source).is_none(), "{source}");
        }
    }

    #[test]
    fn many_impossible_alternatives_have_linear_source_work_and_constant_retained_state() {
        let source = std::iter::repeat_n("(?!a{0})", 20_000)
            .collect::<Vec<_>>()
            .join("|");
        let mut work = 0;
        let plan = RegExpImpossibleMatcher::compile_alternatives_negative_validated_with_work(
            &JsString::from(source.as_str()),
            true,
            true,
            |n| {
                work += n;
                Ok::<(), ()>(())
            },
        )
        .unwrap()
        .unwrap();
        assert_eq!(plan.capture_count(), 0);
        assert!(work <= source.len() * 128);
    }

    #[test]
    fn impossible_alternative_work_aborts_before_each_arm_projection_and_operand_step() {
        let source = JsString::from("(?!a{0})|((?<!b{0}))");
        let mut work = Vec::new();
        assert!(
            RegExpImpossibleMatcher::compile_alternatives_negative_validated_with_work(
                &source,
                true,
                true,
                |n| {
                    work.push(n);
                    Ok::<(), ()>(())
                }
            )
            .unwrap()
            .is_some()
        );
        assert!(work.len() >= 5);
        for stop in 0..work.len() {
            let mut seen = Vec::new();
            assert!(
                matches!(RegExpImpossibleMatcher::compile_alternatives_negative_validated_with_work(
                &source,true,true,|n|{seen.push(n);if seen.len()==stop+1{Err(stop)}else{Ok(())}}
            ),Err(n)if n==stop)
            );
            assert_eq!(seen, work[..=stop]);
        }
    }

    fn required(source: &str) -> Option<RegExpImpossibleMatcher> {
        RegExpImpossibleMatcher::compile_required_enclosing_negative_validated_with_work(
            &JsString::from(source),
            true,
            true,
            |_| Ok::<(), ()>(()),
        )
        .unwrap()
    }

    #[test]
    fn required_impossible_spines_preserve_modifier_counts_and_original_inventories() {
        for (source, captures) in [
            (r"([\p{RGI_Emoji}]+(?!(a{0})))+", 2),
            (r"(?i:((?<!\uD800{0})(a))){2,3}?", 2),
            (r"((a|b)(?!(c{0}))(d)){1,}", 4),
            (r"(?im-s:((?!(\2){0})[\p{RGI_Emoji}]+)){0001,0002}", 2),
            (r"((?!)|((?<!a{0})(b)))+", 3),
        ] {
            assert_eq!(
                required(source).unwrap().capture_count(),
                captures,
                "{source}"
            );
        }
    }

    #[test]
    fn optional_wrappers_unproved_bodies_and_siblings_cannot_prove_no_match() {
        for source in [
            "(?!a{0})",
            "((?!a{0}))?",
            "((?!a{0}))*",
            "((?!a{0})){0,2}",
            "(?i:((?!a{0}))?)+",
            "((?!a{0}))+x",
            "x((?!a{0}))+",
            "((?!a{0}))+|a{0}",
            "((?!)|a{0})+",
            "((?!a))+",
        ] {
            assert!(required(source).is_none(), "{source}");
        }
    }

    #[test]
    fn required_impossible_depth_and_large_minima_remain_symbolic() {
        let source = format!(
            "{}(?!(a{{0}})){}",
            "(".repeat(100_000),
            ")+".repeat(100_000)
        );
        assert_eq!(required(&source).unwrap().capture_count(), 100_001);
        let source = format!(
            r"([\p{{RGI_Emoji}}]+(?!(a{{0}}))){{{}}}",
            "9".repeat(20_000)
        );
        assert_eq!(required(&source).unwrap().capture_count(), 2);
    }

    #[test]
    fn required_impossible_work_aborts_before_each_scope_projection_or_body_step() {
        let source = JsString::from(r"(?i:((?!)|((?<!a{0})(b)))){2,3}");
        let mut work = Vec::new();
        assert!(
            RegExpImpossibleMatcher::compile_required_enclosing_negative_validated_with_work(
                &source,
                true,
                true,
                |n| {
                    work.push(n);
                    Ok::<(), ()>(())
                }
            )
            .unwrap()
            .is_some()
        );
        assert_eq!(work[0], source.len() * 32 + 16);
        for stop in 0..work.len() {
            let mut seen = Vec::new();
            assert!(
                matches!(RegExpImpossibleMatcher::compile_required_enclosing_negative_validated_with_work(
                &source,true,true,|n|{seen.push(n);if seen.len()==stop+1{Err(stop)}else{Ok(())}}
            ),Err(n)if n==stop)
            );
            assert_eq!(seen, work[..=stop]);
        }
    }

    fn positive(source: &str) -> Option<RegExpImpossibleMatcher> {
        RegExpImpossibleMatcher::compile_positive_impossible_validated_with_work(
            &JsString::from(source),
            true,
            true,
            |_| Ok::<(), ()>(()),
        )
        .unwrap()
    }

    #[test]
    fn positive_assertions_over_impossible_bodies_preserve_all_original_slots() {
        for (source, captures) in [
            (r"[\p{RGI_Emoji}]+(?=(?!(a{0})))(b)", 2),
            (r"(?<=(?<!\uD800{0})(a))(b)", 2),
            (r"(?=(((?!(a{0})))))(b)", 4),
            (r"(?<=((?!)|((?<!a{0})(b))))(c)", 4),
            (r"(?=(?i:((?!(\3){0})(a))){2,3})(b)", 4),
        ] {
            assert_eq!(
                positive(source).unwrap().capture_count(),
                captures,
                "{source}"
            );
        }
    }

    #[test]
    fn positive_impossible_proof_excludes_possible_optional_and_nonroot_bodies() {
        for source in [
            r"(?=a)",
            r"(?<=a)",
            r"(?=(?!)|)",
            r"(?=(((?!))?))",
            r"(?=(?!a))",
            r"(?=(?=a{0}))",
            r"(?=(?!))|a{0}",
            r"((?=(?!)))",
            r"(?!(?!))",
            r"(?<!(?!))",
        ] {
            assert!(positive(source).is_none(), "{source}");
        }
    }

    #[test]
    fn positive_impossible_work_is_flat_and_aborts_before_every_body_proof() {
        let source = JsString::from(r"(?=(?i:((?!)|((?<!a{0})(b)))){2,3})");
        let mut work = Vec::new();
        assert!(
            RegExpImpossibleMatcher::compile_positive_impossible_validated_with_work(
                &source,
                true,
                true,
                |n| {
                    work.push(n);
                    Ok::<(), ()>(())
                }
            )
            .unwrap()
            .is_some()
        );
        assert_eq!(work[0], source.len() * 8 + 16);
        assert!(work.iter().sum::<usize>() < source.len() * 1024);
        for stop in 0..work.len() {
            let mut seen = Vec::new();
            assert!(
                matches!(RegExpImpossibleMatcher::compile_positive_impossible_validated_with_work(
                &source,true,true,|n| {seen.push(n);if seen.len()==stop+1 {Err(stop)}else{Ok(())}}
            ),Err(n) if n==stop)
            );
            assert_eq!(seen, work[..=stop]);
        }
        let source = format!(
            "(?={}(?!(a{{0}})){})",
            "(".repeat(100_000),
            ")+".repeat(100_000)
        );
        assert_eq!(positive(&source).unwrap().capture_count(), 100_001);
    }

    #[test]
    fn positive_impossible_match_charges_before_input_or_start_access() {
        let plan = positive(r"(?=(?!))").unwrap();
        assert_eq!(
            plan.find_with_work(&JsString::from("😀"), 1, |n| {
                assert_eq!(n, 1);
                Ok::<(), ()>(())
            }),
            Ok(None)
        );
        assert_eq!(
            plan.find_with_work(&JsString::from(""), usize::MAX, |n| {
                assert_eq!(n, 1);
                Err(17)
            }),
            Err(17)
        );
    }
    fn composite_negative(source: &str) -> Option<RegExpImpossibleMatcher> {
        RegExpImpossibleMatcher::compile_composite_negative_validated_with_work(
            &JsString::from(source),
            true,
            true,
            |_| Ok::<(), ()>(()),
        )
        .unwrap()
    }

    #[test]
    fn composite_negative_assertions_prove_complete_empty_operands_without_losing_slots() {
        for (source, captures) in [
            (r"[\p{RGI_Emoji}]+(?!(?!(?!(a{0}))))(b)", 2),
            (r"(?<!(a{0})(?!(?!(b{0}))))(c)", 3),
            (r"(?!(?i:((a{0})(?!(?!(b{0}))))){2,3})(c)", 4),
            (r"(?!(?!(?!(a{0})))|[\p{RGI_Emoji}]+(b))", 2),
            (r"(?!(a{0})(?!(?!(\3){0}))(b{0}))", 3),
        ] {
            assert_eq!(
                composite_negative(source).unwrap().capture_count(),
                captures,
                "{source}"
            );
        }
    }

    #[test]
    fn composite_negative_assertions_exclude_unproved_conditions_and_outside_contexts() {
        for source in [
            r"(?!a)",
            r"(?!(?!a))",
            r"(?!(?!(?=a)))",
            r"(?!(?!(?!))a)",
            r"(?=(?!(?!)))",
            r"((?!(?!(?!))))",
            r"(?!(?!(?!)))|a{0}",
        ] {
            assert!(composite_negative(source).is_none(), "{source}");
        }
    }

    #[test]
    fn composite_negative_assertions_keep_deep_operand_work_flat_and_interruptible() {
        let source = format!(
            "(?<!{}(?!(?!(a{{0}}))){})",
            "(".repeat(100_000),
            ")+".repeat(100_000)
        );
        assert_eq!(
            composite_negative(&source).unwrap().capture_count(),
            100_001
        );
        let source = JsString::from(r"(?!(?!(?!(a{0})))|(b))");
        let mut work = Vec::new();
        assert!(
            RegExpImpossibleMatcher::compile_composite_negative_validated_with_work(
                &source,
                true,
                true,
                |n| {
                    work.push(n);
                    Ok::<(), ()>(())
                }
            )
            .unwrap()
            .is_some()
        );
        assert_eq!(work[0], source.len() * 8 + 16);
        assert!(work.iter().sum::<usize>() < source.len() * 2048);
        for stop in 0..work.len() {
            let mut seen = Vec::new();
            assert!(
                matches!(RegExpImpossibleMatcher::compile_composite_negative_validated_with_work(
                &source,true,true,|n|{seen.push(n);if seen.len()==stop+1{Err(stop)}else{Ok(())}}
            ),Err(n)if n==stop)
            );
            assert_eq!(seen, work[..=stop]);
        }
    }

    #[test]
    fn composite_negative_no_match_charges_before_input_and_unrepresentable_start_access() {
        let plan = composite_negative(r"(?!(?!(?!)))").unwrap();
        assert_eq!(
            plan.find_with_work(&JsString::from("😀"), 1, |n| {
                assert_eq!(n, 1);
                Ok::<(), ()>(())
            }),
            Ok(None)
        );
        assert_eq!(
            plan.find_with_work(&JsString::from(""), usize::MAX, |n| {
                assert_eq!(n, 1);
                Err(19)
            }),
            Err(19)
        );
    }
    fn wrapped_composite_negative(source: &str) -> Option<RegExpImpossibleMatcher> {
        RegExpImpossibleMatcher::compile_wrapped_composite_negative_validated_with_work(
            &JsString::from(source),
            true,
            true,
            |_| Ok::<(), ()>(()),
        )
        .unwrap()
    }

    #[test]
    fn wrapped_composite_negative_assertions_keep_required_counts_modifiers_and_inventory() {
        for (source, captures) in [
            (r"((?!(?!(?!(a{0})))))+", 2),
            (r"(?i:((a{0})(?!(?!(?!(b{0})))))){2,3}", 3),
            (
                r"(?im-s:(([\p{RGI_Emoji}]+)(?<!(?!(?!(a{0})))))){0002,0003}?",
                3,
            ),
            (r"((?!(?!(?!(\2){0}))))+", 2),
        ] {
            assert_eq!(
                wrapped_composite_negative(source).unwrap().capture_count(),
                captures,
                "{source}"
            );
        }
    }

    #[test]
    fn wrapped_composite_negative_assertions_exclude_optional_wrappers_and_outer_contexts() {
        for source in [
            r"(?!(?!(?!)))",
            r"((?!(?!(?!))))?",
            r"((?!(?!(?!))))*",
            r"((?!(?!(?!)))){0,2}",
            r"(?i:((?!(?!(?!))))?)+",
            r"((?!(?!(?!))))+a",
            r"a((?!(?!(?!))))+",
            r"((?!(?!(?!))))+|a",
            r"((?!(?!(?!)))|a)+",
            r"((?!a))+",
        ] {
            assert!(wrapped_composite_negative(source).is_none(), "{source}");
        }
    }

    #[test]
    fn wrapped_composite_negative_depth_and_huge_minima_keep_constant_retained_storage() {
        let source = format!(
            "{}(?!(?!(?!(a{{0}})))){}",
            "(".repeat(100_000),
            ")+".repeat(100_000)
        );
        assert_eq!(
            wrapped_composite_negative(&source).unwrap().capture_count(),
            100_001
        );
        let body = r"((?!(?!(?!(a{0})))))";
        let source = format!("{body}{{{}}}", "9".repeat(20_000));
        assert_eq!(
            wrapped_composite_negative(&source).unwrap().capture_count(),
            2
        );
    }

    #[test]
    fn wrapped_composite_negative_assertions_interrupt_before_scope_and_operand_steps() {
        let source = JsString::from(r"(?i:((a{0})(?!(?!(?!(b{0})))))){2,3}");
        let mut work = Vec::new();
        assert!(
            RegExpImpossibleMatcher::compile_wrapped_composite_negative_validated_with_work(
                &source,
                true,
                true,
                |n| {
                    work.push(n);
                    Ok::<(), ()>(())
                }
            )
            .unwrap()
            .is_some()
        );
        assert_eq!(work[0], source.len() * 32 + 16);
        for stop in 0..work.len() {
            let mut seen = Vec::new();
            assert!(
                matches!(RegExpImpossibleMatcher::compile_wrapped_composite_negative_validated_with_work(
                &source,true,true,|n|{seen.push(n);if seen.len()==stop+1{Err(stop)}else{Ok(())}}
            ),Err(n)if n==stop)
            );
            assert_eq!(seen, work[..=stop]);
        }
    }
    fn composite_alternatives(source: &str) -> Option<RegExpImpossibleMatcher> {
        RegExpImpossibleMatcher::compile_alternatives_composite_negative_validated_with_work(
            &JsString::from(source),
            true,
            true,
            |_| Ok::<(), ()>(()),
        )
        .unwrap()
    }

    #[test]
    fn composite_alternatives_prove_every_arm_and_keep_original_cross_arm_inventories() {
        for (source, captures) in [
            (r"(?!(?!(?!(a{0}))))|((?!(?!(?!(b{0})))))+", 3),
            (r"(?!(?!(?!(\3){0})))|((?!(?!(?!(a{0})))))+", 3),
            (r"[\q{\||\)}]+(?!(?!(?!)))|(?<!(?!(?!)))", 0),
            (r"(a|b)*(?!(?!(?!)))|\u007C(?<!(?!(?!)))", 1),
        ] {
            assert_eq!(
                composite_alternatives(source).unwrap().capture_count(),
                captures,
                "{source}"
            );
        }
    }

    #[test]
    fn composite_alternatives_reject_empty_possible_unproved_and_nonroot_arms() {
        for source in [
            r"(?!(?!(?!)))",
            r"(?!(?!(?!)))|",
            r"a|(?!(?!(?!)))",
            r"(?!(?!(?!)))|(?=a)",
            r"((?!(?!(?!)))|(?!(?!(?!))))",
            r"(?!(?!(?!)))|((?!(?!(?!))))?",
            r"(?!(?!(?!)))|(?!)",
        ] {
            assert!(composite_alternatives(source).is_none(), "{source}");
        }
    }

    #[test]
    fn many_composite_alternatives_have_source_sized_work_and_constant_retained_plan() {
        let source = std::iter::repeat_n(r"(?!(?!(?!(a{0}))))", 20_000)
            .collect::<Vec<_>>()
            .join("|");
        let source = JsString::from(source.as_str());
        let mut work = 0usize;
        let plan =
            RegExpImpossibleMatcher::compile_alternatives_composite_negative_validated_with_work(
                &source,
                true,
                true,
                |n| {
                    work = work.saturating_add(n);
                    Ok::<(), ()>(())
                },
            )
            .unwrap()
            .unwrap();
        assert_eq!(plan.capture_count(), 20_000);
        assert!(work < source.len() * 2048);
    }

    #[test]
    fn composite_alternative_work_interrupts_before_splitting_projection_and_every_arm_proof() {
        let source = JsString::from(r"(?!(?!(?!(a{0}))))|(?i:((?!(?!(?!(b{0})))))){2,3}");
        let mut work = Vec::new();
        assert!(
            RegExpImpossibleMatcher::compile_alternatives_composite_negative_validated_with_work(
                &source,
                true,
                true,
                |n| {
                    work.push(n);
                    Ok::<(), ()>(())
                }
            )
            .unwrap()
            .is_some()
        );
        assert_eq!(work[0], source.len() * 8 + 16);
        for stop in 0..work.len() {
            let mut seen = Vec::new();
            assert!(
                matches!(RegExpImpossibleMatcher::compile_alternatives_composite_negative_validated_with_work(
                &source,true,true,|n|{seen.push(n);if seen.len()==stop+1{Err(stop)}else{Ok(())}}
            ),Err(n)if n==stop)
            );
            assert_eq!(seen, work[..=stop]);
        }
    }
    fn enclosing_composite_alternatives(source: &str) -> Option<RegExpImpossibleMatcher> {
        RegExpImpossibleMatcher::compile_enclosing_composite_alternatives_validated_with_work(
            &JsString::from(source),
            true,
            true,
            |_| Ok::<(), ()>(()),
        )
        .unwrap()
    }

    #[test]
    fn enclosing_composite_alternatives_keep_required_counts_modifiers_and_all_original_slots() {
        for (source, captures) in [
            (r"((?!(?!(?!(a{0}))))|(?<!(?!(?!(b{0})))))+", 3),
            (
                r"(?im-s:((?!(?!(?!(\3){0})))|((?!(?!(?!(a{0})))))+)){2,3}?",
                4,
            ),
            (r"((?!(?!(?!)))|(?<!(?!(?!)))){0001,0002}", 1),
        ] {
            assert_eq!(
                enclosing_composite_alternatives(source)
                    .unwrap()
                    .capture_count(),
                captures,
                "{source}"
            );
        }
    }

    #[test]
    fn enclosing_composite_alternatives_reject_optional_wrappers_possible_bodies_and_outside_terms()
    {
        for source in [
            r"(?!(?!(?!)))|(?<!(?!(?!)))",
            r"((?!(?!(?!)))|(?<!(?!(?!))))?",
            r"((?!(?!(?!)))|(?<!(?!(?!))))*",
            r"((?!(?!(?!)))|(?<!(?!(?!)))){0,2}",
            r"((?!(?!(?!)))|a)+",
            r"((?!(?!(?!)))|(?<!(?!(?!))))+a",
            r"a((?!(?!(?!)))|(?<!(?!(?!))))+",
            r"((?!(?!(?!)))|(?<!(?!(?!))))+|a",
            r"((?!(?!(?!))))+",
        ] {
            assert!(
                enclosing_composite_alternatives(source).is_none(),
                "{source}"
            );
        }
    }

    #[test]
    fn enclosing_composite_alternatives_keep_deep_inventories_and_huge_minima_symbolic() {
        let body = r"(?!(?!(?!(a{0}))))|(?<!(?!(?!(b{0}))))";
        let source = format!("{}{body}{}", "(".repeat(100_000), ")+".repeat(100_000));
        assert_eq!(
            enclosing_composite_alternatives(&source)
                .unwrap()
                .capture_count(),
            100_002
        );
        let source = format!("({body}){{{}}}", "9".repeat(20_000));
        assert_eq!(
            enclosing_composite_alternatives(&source)
                .unwrap()
                .capture_count(),
            3
        );
    }

    #[test]
    fn enclosing_composite_alternatives_interrupt_before_scope_projection_and_each_arm_step() {
        let source = JsString::from(r"(?i:((?!(?!(?!(a{0}))))|(?<!(?!(?!(b{0})))))){2,3}");
        let mut work = Vec::new();
        assert!(
            RegExpImpossibleMatcher::compile_enclosing_composite_alternatives_validated_with_work(
                &source,
                true,
                true,
                |n| {
                    work.push(n);
                    Ok::<(), ()>(())
                }
            )
            .unwrap()
            .is_some()
        );
        assert_eq!(work[0], source.len() * 32 + 16);
        for stop in 0..work.len() {
            let mut seen = Vec::new();
            assert!(
                matches!(RegExpImpossibleMatcher::compile_enclosing_composite_alternatives_validated_with_work(
                &source,true,true,|n|{seen.push(n);if seen.len()==stop+1{Err(stop)}else{Ok(())}}
            ),Err(n)if n==stop)
            );
            assert_eq!(seen, work[..=stop]);
        }
    }
    fn nullable_group_negative(source: &str) -> Option<RegExpImpossibleMatcher> {
        RegExpImpossibleMatcher::compile_nullable_group_negative_validated_with_work(
            &JsString::from(source),
            true,
            true,
            |_| Ok::<(), ()>(()),
        )
        .unwrap()
    }

    #[test]
    fn negative_nullable_group_spines_fail_even_when_greedy_operands_could_consume() {
        for (source, captures) in [
            (r"[\p{RGI_Emoji}]+(?!((a|ab)(b))?)", 3),
            (r"(?<!((?=a)(b)){0,3})(c)", 3),
            (r"(?!(?i:((a)\1)?){2,3})(b)", 3),
            (r"(?!(?:[\q{ab}](a)){0000,0002}?)", 1),
            (r"(a)(?!((\1)(b))*?)(c)", 5),
        ] {
            assert_eq!(
                nullable_group_negative(source).unwrap().capture_count(),
                captures,
                "{source}"
            );
        }
    }

    #[test]
    fn nullable_group_negative_proof_excludes_required_spines_siblings_and_other_contexts() {
        for source in [
            r"(?!a*)",
            r"(?!(a)+)",
            r"(?!(?=(a)?))",
            r"(?!(?:(a)?)*b)",
            r"(?!(a)?b)",
            r"((?!(a)?))",
            r"(?!(a)?)|x",
            r"(?=(a)?)",
            r"(?!^)",
        ] {
            assert!(nullable_group_negative(source).is_none(), "{source}");
        }
    }

    #[test]
    fn nullable_group_negative_depth_and_huge_bounds_need_no_matching_data() {
        let source = format!(
            "(?<!{}((a|ab)(b))?{})",
            "(".repeat(100_000),
            ")+".repeat(100_000)
        );
        assert_eq!(
            nullable_group_negative(&source).unwrap().capture_count(),
            100_003
        );
        let source = format!(r"(?!([\p{{RGI_Emoji}}]+(a)){{0,{}}})", "9".repeat(20_000));
        assert_eq!(nullable_group_negative(&source).unwrap().capture_count(), 2);
    }

    #[test]
    fn nullable_group_negative_work_interrupts_before_source_scope_and_input_access() {
        let source = JsString::from(r"(a)(?!(?im-s:((\1)(b))?){2,3})(c)");
        let mut work = Vec::new();
        let plan = RegExpImpossibleMatcher::compile_nullable_group_negative_validated_with_work(
            &source,
            true,
            true,
            |n| {
                work.push(n);
                Ok::<(), ()>(())
            },
        )
        .unwrap()
        .unwrap();
        assert_eq!(work[0], source.len() * 8 + 16);
        for stop in 0..work.len() {
            let mut seen = Vec::new();
            assert!(
                matches!(RegExpImpossibleMatcher::compile_nullable_group_negative_validated_with_work(
                &source,true,true,|n|{seen.push(n);if seen.len()==stop+1{Err(stop)}else{Ok(())}}
            ),Err(n)if n==stop)
            );
            assert_eq!(seen, work[..=stop]);
        }
        assert_eq!(
            plan.find_with_work(&JsString::from("a😀ab"), 1, |n| {
                assert_eq!(n, 1);
                Ok::<(), ()>(())
            }),
            Ok(None)
        );
        assert_eq!(
            plan.find_with_work(&JsString::from(""), usize::MAX, |_| Err(31)),
            Err(31)
        );
    }
    fn nullable_atom_negative(
        source: &str,
        unicode: bool,
        sets: bool,
    ) -> Option<RegExpImpossibleMatcher> {
        RegExpImpossibleMatcher::compile_nullable_atom_negative_validated_with_work(
            &JsString::from(source),
            unicode,
            sets,
            |_| Ok::<(), ()>(()),
        )
        .unwrap()
    }

    #[test]
    fn nullable_atom_negative_proofs_accept_complete_literal_class_property_and_wrapper_operands() {
        for (source, captures) in [
            (r"(a)(?![\p{RGI_Emoji}]*)(b)", 2),
            (r"(?<!\p{RGI_Emoji}{0,3})(a)", 1),
            (r"(?!(?im-s:(([\q{ab}]*))){2,3})", 2),
            (r"(?!\x28??)\u007C", 0),
            (r"(?!(a*))", 1),
        ] {
            assert_eq!(
                nullable_atom_negative(source, true, true)
                    .unwrap()
                    .capture_count(),
                captures,
                "{source}"
            );
        }
    }

    #[test]
    fn nullable_atom_negative_proofs_preserve_unicode_surrogate_atom_boundaries() {
        for source in [r"(?!\uD83D\uDE00*)", r"(?!😀*)"] {
            for (unicode, sets) in [(false, false), (true, false), (true, true)] {
                assert_eq!(
                    nullable_atom_negative(source, unicode, sets).is_some(),
                    unicode,
                    "{source}"
                );
            }
        }
        for source in [r"(?!\u{D800}?)", r"(?<!\uD800*)", r"(?!\u007C{0,2})"] {
            assert!(
                nullable_atom_negative(source, true, true).is_some(),
                "{source}"
            );
        }
        assert!(nullable_atom_negative(r"(?!\u{D800}\uDC00*)", true, true).is_none());
    }

    #[test]
    fn nullable_atom_negative_proofs_exclude_multiple_terms_conditions_and_required_atoms() {
        for source in [
            r"(?!a+)",
            r"(?!ab*)",
            r"(?!(a*)b)",
            r"(?!(?=a*))",
            r"(?!(a|b)?)",
            r"((?!a*))",
            r"(?!a*)|b",
            r"(?=a*)",
        ] {
            assert!(
                nullable_atom_negative(source, true, true).is_none(),
                "{source}"
            );
        }
    }

    #[test]
    fn nullable_atom_negative_depth_bounds_and_work_are_flat_and_interruptible() {
        let source = format!(
            "(?<!{}[\\p{{RGI_Emoji}}]*{})",
            "(".repeat(100_000),
            ")+".repeat(100_000)
        );
        assert_eq!(
            nullable_atom_negative(&source, true, true)
                .unwrap()
                .capture_count(),
            100_000
        );
        let source =
            JsString::from(format!(r"(?!(?i:([\q{{ab}}]{{0,{}}})))", "9".repeat(20_000)).as_str());
        let mut work = Vec::new();
        let plan = RegExpImpossibleMatcher::compile_nullable_atom_negative_validated_with_work(
            &source,
            true,
            true,
            |n| {
                work.push(n);
                Ok::<(), ()>(())
            },
        )
        .unwrap()
        .unwrap();
        assert_eq!(work[0], source.len() * 8 + 16);
        for stop in 0..work.len() {
            let mut seen = Vec::new();
            assert!(
                matches!(RegExpImpossibleMatcher::compile_nullable_atom_negative_validated_with_work(
                &source,true,true,|n|{seen.push(n);if seen.len()==stop+1{Err(stop)}else{Ok(())}}
            ),Err(n)if n==stop)
            );
            assert_eq!(seen, work[..=stop]);
        }
        assert_eq!(
            plan.find_with_work(&JsString::from("😀ab"), 1, |n| {
                assert_eq!(n, 1);
                Ok::<(), ()>(())
            }),
            Ok(None)
        );
    }
    fn nullable_sequence_negative(source: &str) -> Option<RegExpImpossibleMatcher> {
        RegExpImpossibleMatcher::compile_nullable_sequence_negative_validated_with_work(
            &JsString::from(source),
            true,
            true,
            |_| Ok::<(), ()>(()),
        )
        .unwrap()
    }

    #[test]
    fn nullable_sequences_make_negative_assertions_fail_with_consuming_greedy_terms() {
        for (source, captures) in [
            (r"[\p{RGI_Emoji}]+(?!(a)?[\p{RGI_Emoji}]*[\q{ab}]?)", 1),
            (r"(?<!((a)?[\p{RGI_Emoji}]*)+)(b)", 3),
            (r"(?!(?im-s:((a)?)(b)*[\q{ab}]?){2,3})", 3),
            (r"(a)(?!(\1)?((b)?){2,3})(c)", 5),
            (r"(?!(?:\x28*\u007C?))", 0),
        ] {
            assert_eq!(
                nullable_sequence_negative(source).unwrap().capture_count(),
                captures,
                "{source}"
            );
        }
    }

    #[test]
    fn nullable_sequence_negative_proofs_reject_required_conditional_choice_and_nonroot_terms() {
        for source in [
            r"(?!a*)",
            r"(?!a*b)",
            r"(?!(a)*b)",
            r"(?!^a*)",
            r"(?!a*(?=b))",
            r"(?!(a)?b+)",
            r"(?!a*|b)",
            r"((?!a*b*))",
            r"(?=a*b*)",
        ] {
            assert!(nullable_sequence_negative(source).is_none(), "{source}");
        }
    }

    #[test]
    fn nullable_sequence_negative_many_terms_deep_wrappers_and_work_stay_flat() {
        let source = format!("(?!{}(a)?)", r"[\p{RGI_Emoji}]*".repeat(20_000));
        assert_eq!(
            nullable_sequence_negative(&source).unwrap().capture_count(),
            1
        );
        let source = format!(
            "(?<!{}a*[\\p{{RGI_Emoji}}]?(b)?{})",
            "(".repeat(100_000),
            ")+".repeat(100_000)
        );
        assert_eq!(
            nullable_sequence_negative(&source).unwrap().capture_count(),
            100_001
        );
    }

    #[test]
    fn nullable_sequence_negative_work_interrupts_before_each_segment_proof_or_input_access() {
        let source = JsString::from(r"(a)(?!(?i:(\1)?[\p{RGI_Emoji}]*){2,3})(b)");
        let mut work = Vec::new();
        let plan = RegExpImpossibleMatcher::compile_nullable_sequence_negative_validated_with_work(
            &source,
            true,
            true,
            |n| {
                work.push(n);
                Ok::<(), ()>(())
            },
        )
        .unwrap()
        .unwrap();
        assert_eq!(work[0], source.len() * 8 + 16);
        for stop in 0..work.len() {
            let mut seen = Vec::new();
            assert!(
                matches!(RegExpImpossibleMatcher::compile_nullable_sequence_negative_validated_with_work(
                &source,true,true,|n|{seen.push(n);if seen.len()==stop+1{Err(stop)}else{Ok(())}}
            ),Err(n)if n==stop)
            );
            assert_eq!(seen, work[..=stop]);
        }
        assert_eq!(
            plan.find_with_work(&JsString::from("a😀ab"), 1, |n| {
                assert_eq!(n, 1);
                Ok::<(), ()>(())
            }),
            Ok(None)
        );
    }
    fn nullable_alternatives_negative(source: &str) -> Option<RegExpImpossibleMatcher> {
        RegExpImpossibleMatcher::compile_nullable_alternatives_negative_validated_with_work(
            &JsString::from(source),
            true,
            true,
            |_| Ok::<(), ()>(()),
        )
        .unwrap()
    }

    #[test]
    fn negative_nullable_alternatives_need_only_one_successful_arm_and_keep_all_slots() {
        for (source, captures) in [
            (r"[\p{RGI_Emoji}]+(?!a|(b)?)", 1),
            (r"(?<!(?i:((a)+|[\p{RGI_Emoji}]*)){2,3})(b)", 3),
            (r"(a)(?!(\1)+|(?:[\q{ab}])?)(b)", 3),
            (r"(?![\p{RGI_Emoji}]+|)", 0),
            (r"(?!|a(b))", 1),
            (r"(?!\u007C|b*)", 0),
        ] {
            assert_eq!(
                nullable_alternatives_negative(source)
                    .unwrap()
                    .capture_count(),
                captures,
                "{source}"
            );
        }
    }

    #[test]
    fn negative_nullable_alternatives_exclude_unproved_arms_siblings_and_nonroot_contexts() {
        for source in [
            r"(?!a*)",
            r"(?!a|b)",
            r"(?!^|a)",
            r"(?!a*(?=b)|c+)",
            r"((?!a|b*))",
            r"(?!a|b*)|c",
            r"(?=a|b*)",
        ] {
            assert!(nullable_alternatives_negative(source).is_none(), "{source}");
        }
    }

    #[test]
    fn negative_nullable_alternative_depth_many_arms_and_work_remain_flat() {
        let source = format!(
            "(?<!{}a|[\\p{{RGI_Emoji}}]*{})",
            "(".repeat(100_000),
            ")+".repeat(100_000)
        );
        assert_eq!(
            nullable_alternatives_negative(&source)
                .unwrap()
                .capture_count(),
            100_000
        );
        let source = format!(
            "(?!{}|)",
            std::iter::repeat_n(r"([\p{RGI_Emoji}]+)", 20_000)
                .collect::<Vec<_>>()
                .join("|")
        );
        assert_eq!(
            nullable_alternatives_negative(&source)
                .unwrap()
                .capture_count(),
            20_000
        );
    }

    #[test]
    fn negative_nullable_alternative_work_interrupts_before_scope_ranges_and_each_arm_proof() {
        let source = JsString::from(r"(a)(?!(?i:(\1)+|[\p{RGI_Emoji}]*){2,3})(b)");
        let mut work = Vec::new();
        let plan =
            RegExpImpossibleMatcher::compile_nullable_alternatives_negative_validated_with_work(
                &source,
                true,
                true,
                |n| {
                    work.push(n);
                    Ok::<(), ()>(())
                },
            )
            .unwrap()
            .unwrap();
        assert_eq!(work[0], source.len() * 8 + 16);
        for stop in 0..work.len() {
            let mut seen = Vec::new();
            assert!(
                matches!(RegExpImpossibleMatcher::compile_nullable_alternatives_negative_validated_with_work(
                &source,true,true,|n|{seen.push(n);if seen.len()==stop+1{Err(stop)}else{Ok(())}}
            ),Err(n)if n==stop)
            );
            assert_eq!(seen, work[..=stop]);
        }
        assert_eq!(
            plan.find_with_work(&JsString::from("a😀ab"), 1, |n| {
                assert_eq!(n, 1);
                Ok::<(), ()>(())
            }),
            Ok(None)
        );
    }
    fn nullable_positive_negative(source: &str) -> Option<RegExpImpossibleMatcher> {
        RegExpImpossibleMatcher::compile_nullable_positive_negative_validated_with_work(
            &JsString::from(source),
            true,
            true,
            |_| Ok::<(), ()>(()),
        )
        .unwrap()
    }

    #[test]
    fn negative_assertions_over_successful_positive_assertions_keep_full_original_slots() {
        for (source, captures) in [
            (r"(?!(?=a*))", 0),
            (r"(?<!(?<=[\p{RGI_Emoji}]*))", 0),
            (r"(?!(?im-s:((?=(a)?[\q{ab}]*))){2,3})", 2),
            (r"(a)(?!(?<=(\1)*|[\p{RGI_Emoji}]+))(b)", 3),
            (r"(?!((?=a*)))", 1),
        ] {
            assert_eq!(
                nullable_positive_negative(source).unwrap().capture_count(),
                captures,
                "{source}"
            );
        }
    }

    #[test]
    fn negative_positive_nullable_proofs_exclude_conditions_siblings_other_layers_and_contexts() {
        for source in [
            r"(?!(?=a))",
            r"(?!(?<=a))",
            r"(?!(?=(?=a*)))",
            r"(?=(?=a*))",
            r"(?!(?=a*)b)",
            r"(?!(?=a*)|b)",
            r"((?!(?=a*)))",
            r"(?!(?=a*))|b",
        ] {
            assert!(nullable_positive_negative(source).is_none(), "{source}");
        }
    }

    #[test]
    fn negative_positive_nullable_depth_and_huge_counts_keep_flat_constant_retained_plans() {
        let source = format!(
            "(?<!{}(?=(a)?[\\p{{RGI_Emoji}}]*){})",
            "(".repeat(100_000),
            ")+".repeat(100_000)
        );
        assert_eq!(
            nullable_positive_negative(&source).unwrap().capture_count(),
            100_001
        );
        let source = format!("(?!((?=a*)){{{}}})", "9".repeat(20_000));
        assert_eq!(
            nullable_positive_negative(&source).unwrap().capture_count(),
            1
        );
    }

    #[test]
    fn negative_positive_nullable_work_interrupts_before_scope_projection_and_each_operand_step() {
        let source = JsString::from(r"(a)(?!(?i:((?<=(\1)+|[\p{RGI_Emoji}]*))){2,3})(b)");
        let mut work = Vec::new();
        let plan = RegExpImpossibleMatcher::compile_nullable_positive_negative_validated_with_work(
            &source,
            true,
            true,
            |n| {
                work.push(n);
                Ok::<(), ()>(())
            },
        )
        .unwrap()
        .unwrap();
        assert_eq!(work[0], source.len() * 8 + 16);
        for stop in 0..work.len() {
            let mut seen = Vec::new();
            assert!(
                matches!(RegExpImpossibleMatcher::compile_nullable_positive_negative_validated_with_work(
                &source,true,true,|n|{seen.push(n);if seen.len()==stop+1{Err(stop)}else{Ok(())}}
            ),Err(n)if n==stop)
            );
            assert_eq!(seen, work[..=stop]);
        }
        assert_eq!(
            plan.find_with_work(&JsString::from(""), usize::MAX, |_| Err(33)),
            Err(33)
        );
    }
    fn wrapped_nullable_negative(source: &str) -> Option<RegExpImpossibleMatcher> {
        RegExpImpossibleMatcher::compile_wrapped_nullable_negative_validated_with_work(
            &JsString::from(source),
            true,
            true,
            |_| Ok::<(), ()>(()),
        )
        .unwrap()
    }

    #[test]
    fn wrapped_nullable_negative_failures_keep_required_counts_modifiers_and_original_inventories()
    {
        for (source, captures) in [
            (r"((?!(a|ab)?))+", 2),
            (r"((?!a*))+", 1),
            (r"((?!a*(b)?)){2,3}?", 2),
            (r"(?im-s:((?!a|(b)?))){0001,0002}", 2),
            (r"((?!(?=a*)))+", 1),
            (r"(?im-s:((a)(?!(\1)?[\p{RGI_Emoji}]*)(b))){2,3}", 4),
        ] {
            assert_eq!(
                wrapped_nullable_negative(source).unwrap().capture_count(),
                captures,
                "{source}"
            );
        }
    }

    #[test]
    fn wrapped_nullable_negative_proofs_reject_optional_outer_spines_siblings_and_unproved_bodies()
    {
        for source in [
            r"(?!a*)",
            r"((?!a*))?",
            r"((?!a*))*",
            r"((?!a*)){0,2}",
            r"(?i:((?!a*))?)+",
            r"((?!a*))+b",
            r"a((?!a*))+",
            r"((?!a*))+|b",
            r"((?!a))+",
            r"((?!a*)|b)+",
        ] {
            assert!(wrapped_nullable_negative(source).is_none(), "{source}");
        }
    }

    #[test]
    fn wrapped_nullable_negative_depth_and_huge_positive_minima_need_no_matching_data() {
        let source = format!(
            "{}(?![\\p{{RGI_Emoji}}]*){}",
            "(".repeat(100_000),
            ")+".repeat(100_000)
        );
        assert_eq!(
            wrapped_nullable_negative(&source).unwrap().capture_count(),
            100_000
        );
        let body = r"((?!(?=[\p{RGI_Emoji}]*)))";
        let source = format!("{body}{{{}}}", "9".repeat(20_000));
        assert_eq!(
            wrapped_nullable_negative(&source).unwrap().capture_count(),
            1
        );
    }

    #[test]
    fn wrapped_nullable_negative_work_interrupts_before_every_scope_projection_and_body_step() {
        let source = JsString::from(r"(?im-s:((a)(?!(\1)?[\p{RGI_Emoji}]*)(b))){2,3}");
        let mut work = Vec::new();
        assert!(
            RegExpImpossibleMatcher::compile_wrapped_nullable_negative_validated_with_work(
                &source,
                true,
                true,
                |n| {
                    work.push(n);
                    Ok::<(), ()>(())
                }
            )
            .unwrap()
            .is_some()
        );
        assert_eq!(work[0], source.len() * 32 + 16);
        for stop in 0..work.len() {
            let mut seen = Vec::new();
            assert!(
                matches!(RegExpImpossibleMatcher::compile_wrapped_nullable_negative_validated_with_work(
                &source,true,true,|n|{seen.push(n);if seen.len()==stop+1{Err(stop)}else{Ok(())}}
            ),Err(n)if n==stop)
            );
            assert_eq!(seen, work[..=stop]);
        }
    }
}
