//! Concatenated erased terms (ECMA-262 22.2.2.2, 22.2.2.3.1).

use crate::{
    JsString, RegExpImpossibleMatcher,
    regexp_literal::unicode_start,
    regexp_zero_count_group::enclosing_group_body,
    regexp_zero_count_spine::{RegExpZeroCountSpineMatcher, atom_end, opener, quantifier},
};
use std::ops::Range;

/// An unconditional empty match from validated erased terms or a root choice.
///
/// Requires original Pattern/flag validation and normalized capture heads.
/// Executed terms must independently match empty without reading input or
/// erased matching data. Capture roles are stored in lexical order, including
/// undefined slots in erased bodies or unselected root alternatives.
#[derive(Clone, Debug)]
pub struct RegExpZeroCountSequenceMatcher {
    unicode: bool,
    defined: Box<[bool]>,
}

impl RegExpZeroCountSequenceMatcher {
    /// Proves an arbitrary tree whose every active alternative stays empty.
    ///
    /// Requires complete original validation and normalized capture heads.
    /// Ordinary/modifier groups, sequences and choices compose only when every
    /// active arm is proved empty. Exact-zero atoms/groups erase arbitrary
    /// matching work; optional empty groups roll capture writes back. The flat
    /// parser and capture-clear markers avoid recursive or quadratic work
    /// (22.2.2.2, 22.2.2.3.1, 22.2.2.7).
    pub fn compile_tree_validated_with_work<E>(
        source: &JsString,
        unicode: bool,
        unicode_sets: bool,
        charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, E> {
        Ok(
            crate::regexp_zero_count_tree::compile_with_work(
                source,
                unicode,
                unicode_sets,
                charge,
            )?
            .map(|defined| Self { unicode, defined }),
        )
    }

    /// Proves empty trees including positive lookahead/lookbehind bodies.
    ///
    /// Requires complete original validation and normalized capture heads.
    /// Positive assertions whose complete bodies always succeed empty export
    /// their capture masks without reading input (22.2.2.4). All active arms
    /// must independently stay empty; negative assertions and unproved consuming
    /// bodies require other proofs. Earlier constructors remain unchanged.
    pub fn compile_positive_tree_validated_with_work<E>(
        source: &JsString,
        unicode: bool,
        unicode_sets: bool,
        charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, E> {
        Ok(crate::regexp_zero_count_tree::compile_positive_with_work(
            source,
            unicode,
            unicode_sets,
            charge,
        )?
        .map(|defined| Self { unicode, defined }))
    }

    /// Proves positive empty trees with zero-erased hexadecimal/Unicode atoms.
    ///
    /// Requires complete original validation and normalized capture heads.
    /// Fixed hexadecimal escapes and non-surrogate Unicode escapes are scanned
    /// as complete literal atoms (22.2.1). Unicode braced escapes require u/v;
    /// surrogate escapes stay unproved to avoid combining adjacent code units
    /// incorrectly. Earlier tree constructors and their work remain unchanged.
    pub fn compile_escaped_tree_validated_with_work<E>(
        source: &JsString,
        unicode: bool,
        unicode_sets: bool,
        charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, E> {
        Ok(crate::regexp_zero_count_tree::compile_escaped_with_work(
            source,
            unicode,
            unicode_sets,
            charge,
        )?
        .map(|defined| Self { unicode, defined }))
    }

    /// Proves empty trees with mode-aware escaped surrogate atom boundaries.
    ///
    /// Requires complete original validation and normalized capture heads.
    /// Adjacent fixed lead/trail Unicode escapes combine into one atom only in
    /// u/v mode (22.2.1). Braced or mixed literal/escaped spellings retain their
    /// separate boundaries. Exact-zero atoms need no matching data; earlier
    /// constructors keep their surrogate exclusions and work.
    pub fn compile_surrogate_tree_validated_with_work<E>(
        source: &JsString,
        unicode: bool,
        unicode_sets: bool,
        charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, E> {
        Ok(crate::regexp_zero_count_tree::compile_surrogate_with_work(
            source,
            unicode,
            unicode_sets,
            charge,
        )?
        .map(|defined| Self { unicode, defined }))
    }

    /// Proves a complete negative assertion over an independently impossible body.
    ///
    /// Requires full original validation and normalized capture heads. Both
    /// negative assertion directions succeed empty when their complete operand
    /// cannot match; none of the operand's captures are exported (22.2.2.4).
    /// Siblings, outside groups and quantifiers require other proofs. The finite
    /// operand proof chain never calls this constructor. Work precedes copies
    /// and capture-mask allocation; construction and storage are source-sized.
    pub fn compile_negative_impossible_validated_with_work<E>(
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
        let start = if units.starts_with(&[0x28, 0x3f, 0x21]) {
            3
        } else if units.starts_with(&[0x28, 0x3f, 0x3c, 0x21]) {
            4
        } else {
            return Ok(None);
        };
        if group_end(units, 0, sets) != Some(units.len()) {
            return Ok(None);
        }
        let body = JsString::from_code_units(units[start..units.len() - 1].to_vec());
        let mut plan = RegExpImpossibleMatcher::compile_negative_validated_with_work(
            &body,
            unicode,
            sets,
            &mut charge,
        )?;
        if plan.is_none() {
            plan = RegExpImpossibleMatcher::compile_enclosing_negative_validated_with_work(
                &body,
                unicode,
                sets,
                &mut charge,
            )?;
        }
        if plan.is_none() {
            plan = RegExpImpossibleMatcher::compile_alternatives_negative_validated_with_work(
                &body,
                unicode,
                sets,
                &mut charge,
            )?;
        }
        if plan.is_none() {
            plan =
                RegExpImpossibleMatcher::compile_required_enclosing_negative_validated_with_work(
                    &body,
                    unicode,
                    sets,
                    &mut charge,
                )?;
        }
        if plan.is_none() {
            plan = RegExpImpossibleMatcher::compile_positive_impossible_validated_with_work(
                &body,
                unicode,
                sets,
                &mut charge,
            )?;
        }
        let Some(plan) = plan else { return Ok(None) };
        charge(plan.capture_count())?;
        Ok(Some(Self {
            unicode,
            defined: vec![false; plan.capture_count()].into_boxed_slice(),
        }))
    }

    /// Proves enclosing ordinary/modifier spines around a negative impossible assertion.
    ///
    /// Full original validation and normalized capture heads are required.
    /// Every inner attempt stays empty, so required repetitions export empty
    /// enclosing captures and optional repetitions roll descendant writes back
    /// (22.2.2.3.1, 22.2.2.7). Operand captures always remain undefined. Bounds
    /// stay symbolic; flat scope and mask work precede allocation.
    pub fn compile_enclosing_negative_impossible_validated_with_work<E>(
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
        let body = JsString::from_code_units(source.code_units()[scope.body].to_vec());
        let Some(inner) = Self::compile_negative_impossible_validated_with_work(
            &body,
            unicode,
            sets,
            &mut charge,
        )?
        else {
            return Ok(None);
        };
        let captures = scope.captures + inner.capture_count();
        charge(captures)?;
        let mut defined = vec![false; captures];
        defined[..scope.defined].fill(true);
        Ok(Some(Self {
            unicode,
            defined: defined.into_boxed_slice(),
        }))
    }

    /// Proves a root choice starting with an empty negative-impossible assertion or spine.
    ///
    /// Full original validation and normalized capture heads are required. The
    /// first arm always succeeds empty with the final successful continuation
    /// (22.2.2.2). Later arms are skipped, retaining their original slots as
    /// undefined. This root-only proof must not be used as a repeat operand:
    /// a failing continuation could backtrack into a consuming later arm.
    /// Source/copy and full-mask work precede allocation.
    pub fn compile_first_negative_impossible_validated_with_work<E>(
        source: &JsString,
        unicode: bool,
        sets: bool,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, E> {
        charge(source.len().saturating_mul(8).saturating_add(16))?;
        if sets && !unicode {
            return Ok(None);
        }
        let Some((first_end, captures)) = choice_layout(source.code_units(), sets) else {
            return Ok(None);
        };
        let first = JsString::from_code_units(source.code_units()[..first_end].to_vec());
        let mut plan = Self::compile_negative_impossible_validated_with_work(
            &first,
            unicode,
            sets,
            &mut charge,
        )?;
        if plan.is_none() {
            plan = Self::compile_enclosing_negative_impossible_validated_with_work(
                &first,
                unicode,
                sets,
                &mut charge,
            )?;
        }
        let Some(plan) = plan else { return Ok(None) };
        charge(captures)?;
        let mut defined = plan.defined.into_vec();
        defined.resize(captures, false);
        Ok(Some(Self {
            unicode,
            defined: defined.into_boxed_slice(),
        }))
    }

    /// Proves concatenated empty trees and negative-impossible assertions or spines.
    ///
    /// Full original validation and normalized capture heads are required. Each
    /// complete root term independently stays empty; at least one term must
    /// use the negative-impossible proof. Root choices and consuming/unproved
    /// terms reject this constructor. Segment masks retain original lexical
    /// order and bindings (22.2.2.2, 22.2.2.4). Source, copy and mask work
    /// precedes allocation; disjoint segments keep construction source-sized.
    pub fn compile_negative_impossible_sequence_validated_with_work<E>(
        source: &JsString,
        unicode: bool,
        sets: bool,
        charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, E> {
        Self::compile_impossible_empty_sequence_with_work::<0, E>(source, unicode, sets, charge)
    }

    /// Proves optional impossible spines concatenated with independently empty terms.
    ///
    /// Full original validation and normalized capture heads are required.
    /// At least one complete root term must use the optional-impossible proof;
    /// other terms use the existing negative-impossible or empty-tree proofs.
    /// Each term independently stays empty, retaining original interleaved masks
    /// and bindings (22.2.2.2, 22.2.2.3.1). Root choices and consuming/unproved
    /// terms reject the proof. Source, copy and mask work precedes allocation.
    pub fn compile_optional_impossible_sequence_validated_with_work<E>(
        source: &JsString,
        unicode: bool,
        sets: bool,
        charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, E> {
        Self::compile_impossible_empty_sequence_with_work::<1, E>(source, unicode, sets, charge)
    }

    /// Proves optional established-impossible terms concatenated with independently empty terms.
    ///
    /// Full original validation and normalized capture heads are required.
    /// At least one complete root term must use the optional-known-impossible
    /// proof. Other terms use earlier negative-impossible or empty-tree proofs.
    /// Every selected term always stays empty, so independently composed masks
    /// and original bindings survive concatenation (22.2.2.2, 22.2.2.4).
    /// Root choices, consuming and unproved terms reject the constructor.
    /// Disjoint source/copy/mask work precedes allocation; earlier modes retain
    /// their domains and work.
    pub fn compile_optional_known_impossible_sequence_validated_with_work<E>(
        source: &JsString,
        unicode: bool,
        sets: bool,
        charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, E> {
        Self::compile_impossible_empty_sequence_with_work::<2, E>(source, unicode, sets, charge)
    }

    fn compile_impossible_empty_sequence_with_work<const MODE: u8, E>(
        source: &JsString,
        unicode: bool,
        sets: bool,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, E> {
        charge(source.len().saturating_mul(16).saturating_add(16))?;
        if sets && !unicode {
            return Ok(None);
        }
        let units = source.code_units();
        let mut cursor = 0;
        let mut terms = 0;
        let mut selected = false;
        let mut defined = Vec::new();
        while cursor < units.len() {
            let end = if units[cursor] == 0x28 {
                group_end(units, cursor, sets)
            } else {
                crate::regexp_zero_count_tree::surrogate_atom_end(units, cursor, unicode, sets)
            };
            let Some(end) = end else { return Ok(None) };
            let Some((end, _, _)) = quantifier(units, end) else {
                return Ok(None);
            };
            let segment = JsString::from_code_units(units[cursor..end].to_vec());
            let mut plan = if MODE == 2 {
                Self::compile_optional_known_impossible_validated_with_work(
                    &segment,
                    unicode,
                    sets,
                    &mut charge,
                )?
            } else if MODE == 1 {
                Self::compile_optional_impossible_validated_with_work(
                    &segment,
                    unicode,
                    sets,
                    &mut charge,
                )?
            } else {
                None
            };
            if MODE != 0 {
                selected |= plan.is_some();
            }
            if plan.is_none() {
                plan = Self::compile_negative_impossible_validated_with_work(
                    &segment,
                    unicode,
                    sets,
                    &mut charge,
                )?;
            }
            if plan.is_none() {
                plan = Self::compile_enclosing_negative_impossible_validated_with_work(
                    &segment,
                    unicode,
                    sets,
                    &mut charge,
                )?;
            }
            if MODE == 0 {
                selected |= plan.is_some();
            }
            if plan.is_none() {
                plan = Self::compile_surrogate_tree_validated_with_work(
                    &segment,
                    unicode,
                    sets,
                    &mut charge,
                )?;
            }
            let Some(plan) = plan else { return Ok(None) };
            charge(plan.capture_count())?;
            defined.extend(plan.defined);
            cursor = end;
            terms += 1;
        }
        Ok((terms >= 2 && selected).then(|| Self {
            unicode,
            defined: defined.into_boxed_slice(),
        }))
    }

    /// Proves ordinary/modifier spines enclosing independently empty negative-impossible sequences.
    ///
    /// Full original validation and normalized capture heads are required.
    /// Every body term stays empty, so repeat continuations cannot expose a
    /// consuming alternative. Required wrappers export empty captures; optional
    /// repetitions roll descendant masks back (22.2.2.3.1, 22.2.2.7). Bounds
    /// remain symbolic and original references keep their bindings. All scope,
    /// copy and full-mask work precedes allocation.
    pub fn compile_enclosing_negative_impossible_sequence_validated_with_work<E>(
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
        let body = JsString::from_code_units(source.code_units()[scope.body].to_vec());
        let Some(inner) = Self::compile_negative_impossible_sequence_validated_with_work(
            &body,
            unicode,
            sets,
            &mut charge,
        )?
        else {
            return Ok(None);
        };
        let captures = scope.captures + inner.capture_count();
        charge(captures)?;
        let mut defined = Vec::with_capacity(captures);
        defined.resize(scope.defined, true);
        defined.resize(scope.captures, false);
        if scope.optional {
            defined.resize(captures, false)
        } else {
            defined.extend(inner.defined)
        }
        Ok(Some(Self {
            unicode,
            defined: defined.into_boxed_slice(),
        }))
    }

    /// Proves optional ordinary/modifier spines whose complete inner body cannot match.
    ///
    /// Full original validation and normalized capture heads are required.
    /// A zero-minimum wrapper can skip the impossible body; every attempted body
    /// fails, so no operand capture is exported (22.2.2.3.1). Required captures
    /// outside the first optional wrapper remain defined empty. Original bindings
    /// and symbolic bounds are preserved. The finite operand chain never calls
    /// this constructor; source/frame/copy and mask work precede allocation.
    pub fn compile_optional_impossible_validated_with_work<E>(
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
        if !scope.optional {
            return Ok(None);
        }
        let body = JsString::from_code_units(source.code_units()[scope.body].to_vec());
        let mut plan = RegExpImpossibleMatcher::compile_negative_validated_with_work(
            &body,
            unicode,
            sets,
            &mut charge,
        )?;
        if plan.is_none() {
            plan = RegExpImpossibleMatcher::compile_enclosing_negative_validated_with_work(
                &body,
                unicode,
                sets,
                &mut charge,
            )?;
        }
        if plan.is_none() {
            plan = RegExpImpossibleMatcher::compile_alternatives_negative_validated_with_work(
                &body,
                unicode,
                sets,
                &mut charge,
            )?;
        }
        if plan.is_none() {
            plan =
                RegExpImpossibleMatcher::compile_required_enclosing_negative_validated_with_work(
                    &body,
                    unicode,
                    sets,
                    &mut charge,
                )?;
        }
        if plan.is_none() {
            plan = RegExpImpossibleMatcher::compile_positive_impossible_validated_with_work(
                &body,
                unicode,
                sets,
                &mut charge,
            )?;
        }
        if plan.is_none() {
            plan = RegExpImpossibleMatcher::compile_composite_negative_validated_with_work(
                &body,
                unicode,
                sets,
                &mut charge,
            )?;
        }
        if plan.is_none() {
            plan = RegExpImpossibleMatcher::compile_wrapped_composite_negative_validated_with_work(
                &body,
                unicode,
                sets,
                &mut charge,
            )?;
        }
        if plan.is_none() {
            plan=RegExpImpossibleMatcher::compile_alternatives_composite_negative_validated_with_work(&body,unicode,sets,&mut charge)?;
        }
        if plan.is_none() {
            plan=RegExpImpossibleMatcher::compile_enclosing_composite_alternatives_validated_with_work(&body,unicode,sets,&mut charge)?;
        }
        let Some(plan) = plan else { return Ok(None) };
        let captures = scope.captures + plan.capture_count();
        charge(captures)?;
        let mut defined = vec![false; captures];
        defined[..scope.defined].fill(true);
        Ok(Some(Self {
            unicode,
            defined: defined.into_boxed_slice(),
        }))
    }

    /// Proves ordinary/modifier spines around independently empty optional-impossible sequences.
    ///
    /// Full original validation and normalized capture heads are required.
    /// Every body term independently stays empty; required wrappers retain the
    /// body's mask and optional repeats roll descendant writes back (22.2.2.3.1,
    /// 22.2.2.7). Original bindings and symbolic bounds remain intact. All flat
    /// scope, body-copy and full-mask work precedes allocation.
    pub fn compile_enclosing_optional_impossible_sequence_validated_with_work<E>(
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
        let body = JsString::from_code_units(source.code_units()[scope.body].to_vec());
        let Some(inner) = Self::compile_optional_impossible_sequence_validated_with_work(
            &body,
            unicode,
            sets,
            &mut charge,
        )?
        else {
            return Ok(None);
        };
        let captures = scope.captures + inner.capture_count();
        charge(captures)?;
        let mut defined = Vec::with_capacity(captures);
        defined.resize(scope.defined, true);
        defined.resize(scope.captures, false);
        if scope.optional {
            defined.resize(captures, false)
        } else {
            defined.extend(inner.defined)
        }
        Ok(Some(Self {
            unicode,
            defined: defined.into_boxed_slice(),
        }))
    }

    /// Proves ordinary/modifier spines around independently empty optional established-impossible sequences.
    ///
    /// Full original validation and normalized capture heads are required.
    /// Every body term independently stays empty; required wrappers retain the
    /// body's mask and optional repeats roll descendant writes back (22.2.2.3.1,
    /// 22.2.2.7). Original bindings and symbolic bounds remain intact. All flat
    /// scope, body-copy and full-mask work precedes allocation.
    pub fn compile_enclosing_optional_known_impossible_sequence_validated_with_work<E>(
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
        let body = JsString::from_code_units(source.code_units()[scope.body].to_vec());
        let Some(inner) = Self::compile_optional_known_impossible_sequence_validated_with_work(
            &body,
            unicode,
            sets,
            &mut charge,
        )?
        else {
            return Ok(None);
        };
        let captures = scope.captures + inner.capture_count();
        charge(captures)?;
        let mut defined = Vec::with_capacity(captures);
        defined.resize(scope.defined, true);
        defined.resize(scope.captures, false);
        if scope.optional {
            defined.resize(captures, false)
        } else {
            defined.extend(inner.defined)
        }
        Ok(Some(Self {
            unicode,
            defined: defined.into_boxed_slice(),
        }))
    }

    /// Proves a root choice starting with an independently empty optional-impossible arm.
    ///
    /// Full original validation and normalized capture heads are required. The
    /// first arm uses the optional, sequence or enclosing-sequence proof. With
    /// the final successful continuation it always succeeds empty; all later
    /// arms are skipped with undefined slots (22.2.2.2). This root-only proof
    /// must not be used as a repeat operand: a failing continuation could retry
    /// a consuming later arm. Source/copy and full-mask work precede allocation.
    pub fn compile_first_optional_impossible_validated_with_work<E>(
        source: &JsString,
        unicode: bool,
        sets: bool,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, E> {
        charge(source.len().saturating_mul(8).saturating_add(16))?;
        if sets && !unicode {
            return Ok(None);
        }
        let Some((first_end, captures)) = choice_layout(source.code_units(), sets) else {
            return Ok(None);
        };
        let first = JsString::from_code_units(source.code_units()[..first_end].to_vec());
        let mut plan = Self::compile_optional_impossible_validated_with_work(
            &first,
            unicode,
            sets,
            &mut charge,
        )?;
        if plan.is_none() {
            plan = Self::compile_optional_impossible_sequence_validated_with_work(
                &first,
                unicode,
                sets,
                &mut charge,
            )?;
        }
        if plan.is_none() {
            plan = Self::compile_enclosing_optional_impossible_sequence_validated_with_work(
                &first,
                unicode,
                sets,
                &mut charge,
            )?;
        }
        let Some(plan) = plan else { return Ok(None) };
        charge(captures)?;
        let mut defined = plan.defined.into_vec();
        defined.resize(captures, false);
        Ok(Some(Self {
            unicode,
            defined: defined.into_boxed_slice(),
        }))
    }

    /// Proves a root choice starting with an independently empty optional established-impossible arm.
    ///
    /// Full original validation and normalized capture heads are required. The
    /// first arm uses the optional, sequence or enclosing-sequence proof. With
    /// the final successful continuation it always succeeds empty; all later
    /// arms are skipped with undefined slots (22.2.2.2). This root-only proof
    /// must not be used as a repeat operand: a failing continuation could retry
    /// a consuming later arm. Source/copy and full-mask work precede allocation.
    pub fn compile_first_optional_known_impossible_validated_with_work<E>(
        source: &JsString,
        unicode: bool,
        sets: bool,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, E> {
        charge(source.len().saturating_mul(8).saturating_add(16))?;
        if sets && !unicode {
            return Ok(None);
        }
        let Some((first_end, captures)) = choice_layout(source.code_units(), sets) else {
            return Ok(None);
        };
        let first = JsString::from_code_units(source.code_units()[..first_end].to_vec());
        let mut plan = Self::compile_optional_known_impossible_validated_with_work(
            &first,
            unicode,
            sets,
            &mut charge,
        )?;
        if plan.is_none() {
            plan = Self::compile_optional_known_impossible_sequence_validated_with_work(
                &first,
                unicode,
                sets,
                &mut charge,
            )?;
        }
        if plan.is_none() {
            plan = Self::compile_enclosing_optional_known_impossible_sequence_validated_with_work(
                &first,
                unicode,
                sets,
                &mut charge,
            )?;
        }
        let Some(plan) = plan else { return Ok(None) };
        charge(captures)?;
        let mut defined = plan.defined.into_vec();
        defined.resize(captures, false);
        Ok(Some(Self {
            unicode,
            defined: defined.into_boxed_slice(),
        }))
    }

    /// Proves a complete positive assertion over independently empty optional-impossible forms.
    ///
    /// Full original validation and normalized capture heads are required.
    /// Both positive assertion directions export the operand's empty/undefined
    /// mask (22.2.2.4). The complete operand uses its final successful
    /// continuation, so the first-empty-root-choice proof is valid there.
    /// Siblings and outside groups require other proofs. Source/copy work
    /// precedes allocation; the finite operand chain never calls this method.
    pub fn compile_positive_optional_impossible_validated_with_work<E>(
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
        let start = if units.starts_with(&[0x28, 0x3f, 0x3d]) {
            3
        } else if units.starts_with(&[0x28, 0x3f, 0x3c, 0x3d]) {
            4
        } else {
            return Ok(None);
        };
        if group_end(units, 0, sets) != Some(units.len()) {
            return Ok(None);
        }
        let body = JsString::from_code_units(units[start..units.len() - 1].to_vec());
        let mut plan = Self::compile_optional_impossible_validated_with_work(
            &body,
            unicode,
            sets,
            &mut charge,
        )?;
        if plan.is_none() {
            plan = Self::compile_optional_impossible_sequence_validated_with_work(
                &body,
                unicode,
                sets,
                &mut charge,
            )?;
        }
        if plan.is_none() {
            plan = Self::compile_enclosing_optional_impossible_sequence_validated_with_work(
                &body,
                unicode,
                sets,
                &mut charge,
            )?;
        }
        if plan.is_none() {
            plan = Self::compile_first_optional_impossible_validated_with_work(
                &body, unicode, sets, charge,
            )?;
        }
        Ok(plan)
    }

    /// Proves a complete positive assertion over independently empty optional established-impossible forms.
    ///
    /// Full original validation and normalized capture heads are required.
    /// Both positive assertion directions export the operand's empty/undefined
    /// mask (22.2.2.4). The complete operand uses its final successful
    /// continuation, so the first-empty-root-choice proof is valid there.
    /// Siblings and outside groups require other proofs. Source/copy work
    /// precedes allocation; the finite operand chain never calls this method.
    pub fn compile_positive_optional_known_impossible_validated_with_work<E>(
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
        let start = if units.starts_with(&[0x28, 0x3f, 0x3d]) {
            3
        } else if units.starts_with(&[0x28, 0x3f, 0x3c, 0x3d]) {
            4
        } else {
            return Ok(None);
        };
        if group_end(units, 0, sets) != Some(units.len()) {
            return Ok(None);
        }
        let body = JsString::from_code_units(units[start..units.len() - 1].to_vec());
        let mut plan = Self::compile_optional_known_impossible_validated_with_work(
            &body,
            unicode,
            sets,
            &mut charge,
        )?;
        if plan.is_none() {
            plan = Self::compile_optional_known_impossible_sequence_validated_with_work(
                &body,
                unicode,
                sets,
                &mut charge,
            )?;
        }
        if plan.is_none() {
            plan = Self::compile_enclosing_optional_known_impossible_sequence_validated_with_work(
                &body,
                unicode,
                sets,
                &mut charge,
            )?;
        }
        if plan.is_none() {
            plan = Self::compile_first_optional_known_impossible_validated_with_work(
                &body, unicode, sets, charge,
            )?;
        }
        Ok(plan)
    }

    /// Proves optional ordinary/modifier spines over established impossible operands.
    ///
    /// Full original validation and normalized capture heads are required.
    /// At least one outside zero minimum skips a body whose every attempt must
    /// fail (22.2.2.3.1). Required captures before the first optional wrapper stay
    /// defined empty; all skipped descendants are undefined. Earlier, nullable-
    /// guard and mixed-choice impossibility families retain original bindings.
    /// The finite operand chain never calls this new empty proof. Source/frame/
    /// copy and full-mask work precedes allocation and remain source-sized.
    pub fn compile_optional_known_impossible_validated_with_work<E>(
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
        if !scope.optional {
            return Ok(None);
        }
        let body = JsString::from_code_units(source.code_units()[scope.body].to_vec());
        let mut plan =
            RegExpImpossibleMatcher::known_arm_with_work(&body, unicode, sets, &mut charge)?;
        if plan.is_none() {
            plan = RegExpImpossibleMatcher::compile_known_alternatives_validated_with_work(
                &body,
                unicode,
                sets,
                &mut charge,
            )?;
        }
        if plan.is_none() {
            plan = RegExpImpossibleMatcher::compile_wrapped_known_validated_with_work(
                &body,
                unicode,
                sets,
                &mut charge,
            )?;
        }
        let Some(plan) = plan else { return Ok(None) };
        let captures = scope.captures + plan.capture_count();
        charge(captures)?;
        let mut defined = vec![false; captures];
        defined[..scope.defined].fill(true);
        Ok(Some(Self {
            unicode,
            defined: defined.into_boxed_slice(),
        }))
    }

    /// Proves concatenated class/property or arbitrary zero-group spines.
    ///
    /// A flat mode-aware scan separates complete top-level terms. Each segment
    /// keeps its original binding context: references inside erased bodies are
    /// never revalidated or rebound in isolation. Source and temporary storage
    /// work are charged before allocation. Construction and retained storage
    /// are source-sized; matching and individual capture queries are constant.
    pub fn compile_validated_with_work<E>(
        source: &JsString,
        unicode: bool,
        unicode_sets: bool,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, E> {
        charge(source.len().saturating_mul(8).saturating_add(16))?;
        if unicode_sets && !unicode {
            return Ok(None);
        }
        let units = source.code_units();
        let mut cursor = 0;
        let mut terms = 0;
        let mut defined = Vec::new();
        while cursor < units.len() {
            let end = if units[cursor] == 0x28 {
                group_end(units, cursor, unicode_sets)
            } else {
                atom_end(units, cursor, unicode, unicode_sets)
            };
            let Some(end) = end else {
                return Ok(None);
            };
            let Some((end, _, _)) = quantifier(units, end) else {
                return Ok(None);
            };
            let segment = JsString::from_code_units(units[cursor..end].to_vec());
            let mut plan = RegExpZeroCountSpineMatcher::compile_validated_with_work(
                &segment,
                unicode,
                unicode_sets,
                &mut charge,
            )?;
            if plan.is_none() {
                plan = RegExpZeroCountSpineMatcher::compile_group_validated_with_work(
                    &segment,
                    unicode,
                    unicode_sets,
                    &mut charge,
                )?;
            }
            let Some(plan) = plan else {
                return Ok(None);
            };
            defined.extend(std::iter::repeat_n(true, plan.defined_capture_count()));
            defined.extend(std::iter::repeat_n(
                false,
                plan.capture_count() - plan.defined_capture_count(),
            ));
            cursor = end;
            terms += 1;
        }
        Ok((terms >= 2).then(|| Self {
            unicode,
            defined: defined.into_boxed_slice(),
        }))
    }

    /// Proves a root choice whose first alternative always succeeds empty.
    ///
    /// The first alternative must be empty or a proven zero-erased spine or
    /// sequence. Later alternatives may contain any fully validated body: the
    /// first alternative's successful continuation prevents their execution
    /// (22.2.2.2). Their original lexical slots are reserved as undefined.
    /// This proof assumes the final successful continuation; repetition needs
    /// an additional proof that backtracking cannot select a consuming arm.
    /// Validation and capture-head normalization requirements are unchanged.
    pub fn compile_first_alternative_validated_with_work<E>(
        source: &JsString,
        unicode: bool,
        unicode_sets: bool,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, E> {
        charge(source.len().saturating_mul(8).saturating_add(16))?;
        if unicode_sets && !unicode {
            return Ok(None);
        }
        let Some((first_end, captures)) = choice_layout(source.code_units(), unicode_sets) else {
            return Ok(None);
        };
        let mut defined = Vec::new();
        if first_end != 0 {
            let first = JsString::from_code_units(source.code_units()[..first_end].to_vec());
            let mut spine = RegExpZeroCountSpineMatcher::compile_validated_with_work(
                &first,
                unicode,
                unicode_sets,
                &mut charge,
            )?;
            if spine.is_none() {
                spine = RegExpZeroCountSpineMatcher::compile_group_validated_with_work(
                    &first,
                    unicode,
                    unicode_sets,
                    &mut charge,
                )?;
            }
            if let Some(spine) = spine {
                defined.extend(std::iter::repeat_n(true, spine.defined_capture_count()));
                defined.resize(spine.capture_count(), false);
            } else {
                let Some(sequence) =
                    Self::compile_validated_with_work(&first, unicode, unicode_sets, &mut charge)?
                else {
                    return Ok(None);
                };
                defined = sequence.defined.into_vec();
            }
        }
        defined.resize(captures, false);
        Ok(Some(Self {
            unicode,
            defined: defined.into_boxed_slice(),
        }))
    }

    /// Proves mandatory ordinary groups enclosing a zero sequence or choice.
    ///
    /// Full original validation and normalized capture heads are required.
    /// Enclosing captures are defined empty (22.2.2.7); the body's lexical mask
    /// retains interleaved or unselected slots. Original references inside
    /// erased bodies keep their bindings, including enclosing targets. The
    /// flat scope scan excludes quantified/modifier wrappers and outside terms.
    pub fn compile_enclosing_validated_with_work<E>(
        source: &JsString,
        unicode: bool,
        unicode_sets: bool,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, E> {
        charge(source.len().saturating_mul(8).saturating_add(16))?;
        if unicode_sets && !unicode {
            return Ok(None);
        }
        let Some(group) = enclosing_group_body(source.code_units(), unicode_sets) else {
            return Ok(None);
        };
        let body = JsString::from_code_units(source.code_units()[group.body].to_vec());
        let mut inner =
            Self::compile_validated_with_work(&body, unicode, unicode_sets, &mut charge)?;
        if inner.is_none() {
            inner = Self::compile_first_alternative_validated_with_work(
                &body,
                unicode,
                unicode_sets,
                &mut charge,
            )?;
        }
        Ok(inner.map(|inner| {
            let mut defined = Vec::with_capacity(group.captures + inner.capture_count());
            defined.resize(group.captures, true);
            defined.extend(inner.defined);
            Self {
                unicode,
                defined: defined.into_boxed_slice(),
            }
        }))
    }

    /// Proves quantified ordinary/modifier spines around an empty sequence or choice.
    ///
    /// Full original validation and normalized capture heads are required.
    /// Required empty repetitions retain the body's mask; optional empty
    /// repetitions roll back all descendant writes (22.2.2.3.1). Every arm of
    /// an inner choice must itself be zero-erased: an empty iteration's failing
    /// continuation can backtrack into later arms. Bounds remain
    /// symbolic, including oversized minima. Modifier flags cannot change an
    /// unconditional empty body's result. All source/frame work precedes allocation.
    pub fn compile_quantified_enclosing_validated_with_work<E>(
        source: &JsString,
        unicode: bool,
        unicode_sets: bool,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, E> {
        charge(source.len().saturating_mul(32).saturating_add(16))?;
        if unicode_sets && !unicode {
            return Ok(None);
        }
        let Some(scope) = quantified_enclosing_scope(source.code_units(), unicode_sets) else {
            return Ok(None);
        };
        let body = JsString::from_code_units(source.code_units()[scope.body].to_vec());
        let mut inner =
            Self::compile_validated_with_work(&body, unicode, unicode_sets, &mut charge)?;
        if inner.is_none() {
            inner = Self::compile_backtracking_safe_choice_with_work(
                &body,
                unicode,
                unicode_sets,
                &mut charge,
            )?;
        }
        Ok(inner.map(|inner| {
            let mut defined = Vec::with_capacity(scope.captures + inner.capture_count());
            defined.resize(scope.defined, true);
            defined.resize(scope.captures, false);
            if scope.optional {
                defined.resize(scope.captures + inner.capture_count(), false);
            } else {
                defined.extend(inner.defined);
            }
            Self {
                unicode,
                defined: defined.into_boxed_slice(),
            }
        }))
    }

    fn compile_backtracking_safe_choice_with_work<E>(
        source: &JsString,
        unicode: bool,
        unicode_sets: bool,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, E> {
        let Some(first) = Self::compile_first_alternative_validated_with_work(
            source,
            unicode,
            unicode_sets,
            &mut charge,
        )?
        else {
            return Ok(None);
        };
        let units = source.code_units();
        let mut cursor = 0;
        let mut depth = 0usize;
        let mut class_depth = 0usize;
        let mut arm_start = None;
        while cursor <= units.len() {
            let unit = units.get(cursor).copied();
            if unit == Some(0x5c) {
                cursor += 2;
                continue;
            }
            if class_depth != 0 {
                if unit == Some(0x5d) {
                    class_depth -= 1;
                } else if unicode_sets && unit == Some(0x5b) {
                    class_depth += 1;
                }
            } else {
                match unit {
                    Some(0x5b) => class_depth = 1,
                    Some(0x28) => depth += 1,
                    Some(0x29) => depth -= 1,
                    Some(0x7c) | None if depth == 0 => {
                        if let Some(start) = arm_start {
                            if start != cursor {
                                let arm = JsString::from_code_units(units[start..cursor].to_vec());
                                let mut spine =
                                    RegExpZeroCountSpineMatcher::compile_validated_with_work(
                                        &arm,
                                        unicode,
                                        unicode_sets,
                                        &mut charge,
                                    )?;
                                if spine.is_none() {
                                    spine = RegExpZeroCountSpineMatcher::compile_group_validated_with_work(
                                        &arm, unicode, unicode_sets, &mut charge,
                                    )?;
                                }
                                if spine.is_none()
                                    && Self::compile_validated_with_work(
                                        &arm,
                                        unicode,
                                        unicode_sets,
                                        &mut charge,
                                    )?
                                    .is_none()
                                {
                                    return Ok(None);
                                }
                            }
                        }
                        arm_start = Some(cursor + 1);
                    }
                    _ => {}
                }
            }
            cursor += 1;
        }
        Ok(Some(first))
    }

    /// Original lexical capture count, excluding the whole match (22.2.7.2).
    pub fn capture_count(&self) -> usize {
        self.defined.len()
    }

    /// A participating empty capture, or an undefined/out-of-range slot.
    pub fn capture_range(&self, index: usize, matched: &Range<usize>) -> Option<Range<usize>> {
        self.defined
            .get(index)
            .is_some_and(|&defined| defined)
            .then(|| matched.clone())
    }

    /// Matches empty at the requested boundary, charging before input access.
    ///
    /// Unicode starts inside a surrogate pair use its leading boundary under
    /// the project's approved RegExpBuiltinExec interpretation.
    pub fn find_with_work<E>(
        &self,
        input: &JsString,
        start: usize,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Range<usize>>, E> {
        charge(1)?;
        let start = if self.unicode {
            unicode_start(input, start)
        } else {
            (start <= input.len()).then_some(start)
        };
        Ok(start.map(|start| start..start))
    }
}

struct EnclosingFrame {
    before: usize,
    captures: usize,
    body_start: usize,
    close: usize,
    end: usize,
    minimum_zero: bool,
}

pub(crate) struct EnclosingScope {
    pub(crate) body: Range<usize>,
    pub(crate) captures: usize,
    defined: usize,
    pub(crate) optional: bool,
}

pub(crate) fn quantified_enclosing_scope(units: &[u16], sets: bool) -> Option<EnclosingScope> {
    let mut opening_end = 0;
    let mut captures = 0;
    let mut frames = Vec::new();
    while let Some((width, capturing)) = opener(units, opening_end) {
        let before = captures;
        captures += usize::from(capturing);
        opening_end += width;
        frames.push(EnclosingFrame {
            before,
            captures,
            body_start: opening_end,
            close: 0,
            end: 0,
            minimum_zero: false,
        });
    }
    if frames.is_empty() {
        return None;
    }
    let mut remaining = frames.len();
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
                    depth = depth.checked_sub(1)?;
                    if depth < remaining {
                        remaining = depth;
                        let (end, minimum_zero, _) = quantifier(units, cursor + 1)?;
                        frames[depth].close = cursor;
                        frames[depth].end = end;
                        frames[depth].minimum_zero = minimum_zero;
                    }
                }
                _ => {}
            }
        }
        cursor += 1;
    }
    if depth != 0 || class_depth != 0 || remaining != 0 {
        return None;
    }
    let mut complete = 0usize;
    let mut defined = usize::MAX;
    let mut optional = false;
    for (index, frame) in frames.iter().enumerate() {
        let expected_end = if index == 0 {
            units.len()
        } else {
            frames[index - 1].close
        };
        if frame.end != expected_end {
            break;
        }
        complete += 1;
        if frame.minimum_zero {
            optional = true;
            defined = defined.min(frame.before);
        }
    }
    let frame = frames.get(complete.checked_sub(1)?)?;
    Some(EnclosingScope {
        body: frame.body_start..frame.close,
        captures: frame.captures,
        defined: defined.min(frame.captures),
        optional,
    })
}

// First root alternative and all lexical capture slots, including skipped arms.
fn choice_layout(units: &[u16], sets: bool) -> Option<(usize, usize)> {
    let mut cursor = 0;
    let mut depth = 0usize;
    let mut class_depth = 0usize;
    let mut captures = 0;
    let mut first_end = None;
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
                    depth += 1;
                    captures += usize::from(units.get(cursor + 1) != Some(&0x3f));
                }
                0x29 => depth = depth.checked_sub(1)?,
                0x7c if depth == 0 => {
                    first_end.get_or_insert(cursor);
                }
                _ => {}
            }
        }
        cursor += 1;
    }
    (depth == 0 && class_depth == 0).then_some((first_end?, captures))
}

pub(crate) fn group_end(units: &[u16], start: usize, sets: bool) -> Option<usize> {
    let mut cursor = start;
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
                    depth = depth.checked_sub(1)?;
                    if depth == 0 {
                        return Some(cursor + 1);
                    }
                }
                _ => {}
            }
        }
        cursor += 1;
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn compile(source: &str, unicode: bool, sets: bool) -> Option<RegExpZeroCountSequenceMatcher> {
        RegExpZeroCountSequenceMatcher::compile_validated_with_work(
            &JsString::from(source),
            unicode,
            sets,
            |_| Ok::<(), ()>(()),
        )
        .unwrap()
    }

    #[test]
    fn native_sequence_capture_masks_keep_each_terms_lexical_slots() {
        for (source, unicode, sets, expected) in [
            (r"([a]{0})?([b]{0})+", false, false, &[false, true][..]),
            (
                r"((a)(?=(b))){0}([c]{0})+",
                true,
                false,
                &[false, false, false, true][..],
            ),
            (
                r"([\p{RGI_Emoji}]{0})+(\p{RGI_Emoji}{0})?([\q{a|ab}]{0})+",
                true,
                true,
                &[true, false, true][..],
            ),
            (
                r"(?i:((a|\2){0})+)([a]{0})?((?<=b)(c)){0}",
                true,
                true,
                &[true, false, false, false, false][..],
            ),
            (
                r"([\q{\(|\)}]a){0}([a]{0})+",
                true,
                true,
                &[false, true][..],
            ),
            (r"[a]{0}(?:b){0}\p{RGI_Emoji}{0}", true, true, &[][..]),
            (r"([\(\)]){0}([[a]{0})+", false, false, &[false, true][..]),
        ] {
            let plan = compile(source, unicode, sets).unwrap_or_else(|| panic!("{source}"));
            assert_eq!(plan.capture_count(), expected.len(), "{source}");
            for (index, &defined) in expected.iter().enumerate() {
                assert_eq!(
                    plan.capture_range(index, &(2..2)),
                    defined.then_some(2..2),
                    "{source}: {index}"
                );
            }
            assert_eq!(plan.capture_range(expected.len(), &(2..2)), None);
        }
    }

    #[test]
    fn sequences_exclude_unproved_terms_choices_and_shared_outer_scope() {
        for source in [
            "",
            "[a]{0}",
            "([a]{0})+",
            "[a]{0}x",
            "x[a]{0}",
            "[a]{0}|[b]{0}",
            "([a]{0}[b]{0})+",
            "(?=[a]{0})[b]{0}",
            "[a]*[b]{0}",
            r"([a]{0})\1",
            "[a]{0}^",
            "[a]{0}[b]{0}x",
        ] {
            assert!(compile(source, true, true).is_none(), "{source}");
        }
        assert!(compile("[a]{0}[b]{0}", false, true).is_none());
    }

    #[test]
    fn large_sequences_and_counts_have_flat_storage_and_explicit_work() {
        let source = "([a]{0})?([b]{0})+".repeat(50_000);
        let plan = compile(&source, true, true).unwrap();
        assert_eq!(plan.capture_count(), 100_000);
        for index in 0..plan.capture_count() {
            assert_eq!(
                plan.capture_range(index, &(1..1)),
                (index % 2 == 1).then_some(1..1)
            );
        }
        let huge = format!("([a]{{0}}){{{}}}(b){{0}}", "9".repeat(20_000));
        let plan = compile(&huge, true, true).unwrap();
        assert_eq!(plan.capture_range(0, &(0..0)), Some(0..0));
        assert_eq!(plan.capture_range(1, &(0..0)), None);
        let source = JsString::from(source.as_str());
        let mut charges = Vec::new();
        assert!(
            RegExpZeroCountSequenceMatcher::compile_validated_with_work(
                &source,
                true,
                true,
                |work| {
                    charges.push(work);
                    Err(17)
                }
            )
            .is_err()
        );
        assert_eq!(charges, [source.len() * 8 + 16]);
    }

    #[test]
    fn matching_boundaries_and_abort_precede_input_inspection() {
        let input = JsString::from("A😀!");
        for unicode in [false, true] {
            let plan = compile("[a]{0}([b]{0})+", unicode, false).unwrap();
            for start in 0..=input.len() + 1 {
                let expected = if start > input.len() {
                    None
                } else if unicode && start == 2 {
                    Some(1..1)
                } else {
                    Some(start..start)
                };
                let mut work = 0;
                assert_eq!(
                    plan.find_with_work(&input, start, |n| {
                        work += n;
                        Ok::<(), ()>(())
                    }),
                    Ok(expected)
                );
                assert_eq!(work, 1);
                assert_eq!(plan.find_with_work(&input, start, |_| Err(23)), Err(23));
            }
            let copy = plan.clone();
            drop(plan);
            assert_eq!(copy.capture_range(0, &(0..0)), Some(0..0));
        }
    }

    fn choice(source: &str, unicode: bool, sets: bool) -> Option<RegExpZeroCountSequenceMatcher> {
        RegExpZeroCountSequenceMatcher::compile_first_alternative_validated_with_work(
            &JsString::from(source),
            unicode,
            sets,
            |_| Ok::<(), ()>(()),
        )
        .unwrap()
    }

    #[test]
    fn first_empty_alternative_reserves_all_unselected_lexical_captures() {
        for (source, unicode, sets, expected) in [
            (r"|(?=(a))[\q{|}]", true, true, &[false][..]),
            (r"|[[]((a))", false, false, &[false, false][..]),
            (
                r"([a]{0})+|((?<=b)(c))",
                true,
                true,
                &[true, false, false][..],
            ),
            (
                r"([a]{0})?([b]{0})+|((a)|(?=(b)))",
                true,
                true,
                &[false, true, false, false, false][..],
            ),
            (
                r"((a|b)\3){0}|([\p{RGI_Emoji}])|(?<=(x))",
                true,
                true,
                &[false, false, false, false][..],
            ),
            (
                r"([\q{\(|\)|\|}]{0})+|(?=(a))",
                true,
                true,
                &[true, false][..],
            ),
            (r"\p{RGI_Emoji}{0}|", true, true, &[][..]),
            (r"|", true, true, &[][..]),
        ] {
            let plan = choice(source, unicode, sets).unwrap_or_else(|| panic!("{source}"));
            assert_eq!(plan.capture_count(), expected.len(), "{source}");
            for (index, &defined) in expected.iter().enumerate() {
                assert_eq!(
                    plan.capture_range(index, &(2..2)),
                    defined.then_some(2..2),
                    "{source}: {index}"
                );
            }
        }
    }

    #[test]
    fn first_alternative_proof_excludes_conditions_consumption_and_nested_choices() {
        for source in [
            "",
            "([a]{0})+",
            "a|[b]{0}",
            "^|[a]{0}",
            "[a]*|",
            "(?=[a]{0})|",
            "(?:[a]{0}|x)",
            "[a]{0}x|",
            r"\1|(a)",
            r"[\q{a|b}]+|",
        ] {
            assert!(choice(source, true, true).is_none(), "{source}");
        }
        assert!(choice("|a", false, true).is_none());
    }

    #[test]
    fn deep_unselected_alternatives_remain_flat_and_keep_all_undefined_slots() {
        let source = format!(
            "([a]{{0}})+|{}x{}",
            "(".repeat(100_000),
            ")".repeat(100_000)
        );
        let plan = choice(&source, true, true).unwrap();
        assert_eq!(plan.capture_count(), 100_001);
        assert_eq!(plan.capture_range(0, &(0..0)), Some(0..0));
        for index in 1..plan.capture_count() {
            assert_eq!(plan.capture_range(index, &(0..0)), None);
        }
        let copy = plan.clone();
        drop(plan);
        assert_eq!(copy.capture_count(), 100_001);
    }

    #[test]
    fn first_alternative_construction_propagates_every_source_work_abort() {
        let source = JsString::from(r"([a]{0})?((b|c)){0}|(?=(d))[\p{RGI_Emoji}]+");
        let mut work = Vec::new();
        assert!(
            RegExpZeroCountSequenceMatcher::compile_first_alternative_validated_with_work(
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
        assert!(work.len() >= 4);
        assert_eq!(work[0], source.len() * 8 + 16);
        for stop in 0..work.len() {
            let mut seen = Vec::new();
            let result =
                RegExpZeroCountSequenceMatcher::compile_first_alternative_validated_with_work(
                    &source,
                    true,
                    true,
                    |n| {
                        seen.push(n);
                        if seen.len() == stop + 1 {
                            Err(stop)
                        } else {
                            Ok(())
                        }
                    },
                );
            assert!(matches!(result, Err(n) if n==stop));
            assert_eq!(seen, work[..=stop]);
        }
    }

    fn enclosing(source: &str) -> Option<RegExpZeroCountSequenceMatcher> {
        RegExpZeroCountSequenceMatcher::compile_enclosing_validated_with_work(
            &JsString::from(source),
            true,
            true,
            |_| Ok::<(), ()>(()),
        )
        .unwrap()
    }

    #[test]
    fn mandatory_enclosing_captures_precede_sequence_and_choice_masks() {
        for (source, expected) in [
            (r"((([a]{0})?([b]{0})+))", &[true, true, false, true][..]),
            (r"(?:(|((a))))", &[true, false, false][..]),
            (r"(((a)\1){0}|(b))", &[true, false, false, false][..]),
            (
                r"((([a]{0})+|((b))))",
                &[true, true, true, false, false][..],
            ),
            (
                r"((([\q{\(|\)|\|}]{0})+)([\p{RGI_Emoji}]{0})?)",
                &[true, true, true, false][..],
            ),
            (r"(?:([a]{0})([b]{0}))", &[true, true][..]),
        ] {
            let plan = enclosing(source).unwrap_or_else(|| panic!("{source}"));
            assert_eq!(plan.capture_count(), expected.len(), "{source}");
            for (index, &defined) in expected.iter().enumerate() {
                assert_eq!(
                    plan.capture_range(index, &(2..2)),
                    defined.then_some(2..2),
                    "{source}: {index}"
                );
            }
        }
    }

    #[test]
    fn enclosing_scope_excludes_quantified_modifier_and_later_sibling_groups() {
        for source in [
            "[a]{0}[b]{0}",
            "([a]{0}[b]{0})+",
            "(?i:[a]{0}[b]{0})",
            "(([a]{0})?([b]{0})+)x",
            "(([a]{0})?([b]{0})+)(x)",
            "x(([a]{0})?([b]{0})+)",
            "(([a]{0})?([b]{0})+)|x",
            "(a|[b]{0})",
            "([a]{0})",
            "(?=([a]{0}[b]{0}))",
        ] {
            assert!(enclosing(source).is_none(), "{source}");
        }
        assert!(
            RegExpZeroCountSequenceMatcher::compile_enclosing_validated_with_work(
                &JsString::from("([a]{0}[b]{0})"),
                false,
                true,
                |_| Ok::<(), ()>(()),
            )
            .unwrap()
            .is_none()
        );
    }

    #[test]
    fn hundred_thousand_enclosing_groups_keep_flat_interleaved_capture_storage() {
        let source = format!(
            "{}([a]{{0}})?([b]{{0}})+{}",
            "(".repeat(100_000),
            ")".repeat(100_000)
        );
        let plan = enclosing(&source).unwrap();
        assert_eq!(plan.capture_count(), 100_002);
        for index in 0..100_000 {
            assert_eq!(plan.capture_range(index, &(1..1)), Some(1..1));
        }
        assert_eq!(plan.capture_range(100_000, &(1..1)), None);
        assert_eq!(plan.capture_range(100_001, &(1..1)), Some(1..1));
        let copy = plan.clone();
        drop(plan);
        assert_eq!(copy.capture_count(), 100_002);
    }

    #[test]
    fn enclosing_construction_work_aborts_before_each_delegated_step() {
        let source = JsString::from(r"(((a)\1){0}|(b))");
        let mut work = Vec::new();
        assert!(
            RegExpZeroCountSequenceMatcher::compile_enclosing_validated_with_work(
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
        assert!(work.len() >= 4);
        assert_eq!(work[0], source.len() * 8 + 16);
        for stop in 0..work.len() {
            let mut seen = Vec::new();
            let result = RegExpZeroCountSequenceMatcher::compile_enclosing_validated_with_work(
                &source,
                true,
                true,
                |n| {
                    seen.push(n);
                    if seen.len() == stop + 1 {
                        Err(stop)
                    } else {
                        Ok(())
                    }
                },
            );
            assert!(matches!(result, Err(n) if n==stop));
            assert_eq!(seen, work[..=stop]);
        }
    }

    fn quantified(source: &str) -> Option<RegExpZeroCountSequenceMatcher> {
        RegExpZeroCountSequenceMatcher::compile_quantified_enclosing_validated_with_work(
            &JsString::from(source),
            true,
            true,
            |_| Ok::<(), ()>(()),
        )
        .unwrap()
    }

    #[test]
    fn quantified_enclosing_masks_distinguish_required_and_rolled_back_captures() {
        for (source, expected) in [
            (r"(([a]{0})?([b]{0})+)+", &[true, false, true][..]),
            (r"(([a]{0})?([b]{0})+)*", &[false, false, false][..]),
            (
                r"((([a]{0})?([b]{0})+)+)?",
                &[false, false, false, false][..],
            ),
            (r"(?i:(([a]{0})?([b]{0})+)){2,3}", &[true, false, true][..]),
            (
                r"(?i:(([a]{0})?([b]{0})+)){0,2}",
                &[false, false, false][..],
            ),
            (
                r"((([a]{0})+|((b){0}))){2,3}?",
                &[true, true, true, false, false][..],
            ),
            (r"(?-i:(|((a){0})))+", &[true, false, false][..]),
            (r"((|((a){0})){0})+", &[true, false, false, false][..]),
        ] {
            let plan = quantified(source).unwrap_or_else(|| panic!("{source}"));
            assert_eq!(plan.capture_count(), expected.len(), "{source}");
            for (index, &defined) in expected.iter().enumerate() {
                assert_eq!(
                    plan.capture_range(index, &(2..2)),
                    defined.then_some(2..2),
                    "{source}: {index}"
                );
            }
        }
    }

    #[test]
    fn quantified_enclosing_scope_excludes_siblings_conditions_and_nonempty_bodies() {
        for source in [
            "[a]{0}[b]{0}",
            "([a]{0}[b]{0})+x",
            "([a]{0}[b]{0})+(x)",
            "x([a]{0}[b]{0})+",
            "([a]{0}[b]{0})+|x",
            "(a|[b]{0})+",
            "(?=[a]{0}[b]{0})",
            "([a]+[b]{0})*",
            "([a]{0})+",
            "(|a)+",
            "(|a)*",
            "(|a){2,3}",
            "((|a)){2,3}?",
            r"(?i:(|[\p{RGI_Emoji}]+))+",
        ] {
            assert!(quantified(source).is_none(), "{source}");
        }
    }

    #[test]
    fn quantified_enclosing_depth_and_huge_required_bounds_remain_symbolic() {
        let source = format!(
            "{}([a]{{0}})?([b]{{0}})+{}",
            "(".repeat(100_000),
            ")+".repeat(100_000)
        );
        let plan = quantified(&source).unwrap();
        assert_eq!(plan.capture_count(), 100_002);
        assert_eq!(plan.capture_range(99_999, &(0..0)), Some(0..0));
        assert_eq!(plan.capture_range(100_000, &(0..0)), None);
        assert_eq!(plan.capture_range(100_001, &(0..0)), Some(0..0));
        let source = format!("(([a]{{0}})?([b]{{0}})+){{{}}}", "9".repeat(20_000));
        let plan = quantified(&source).unwrap();
        assert_eq!(plan.capture_count(), 3);
        assert_eq!(plan.capture_range(0, &(0..0)), Some(0..0));
        assert_eq!(plan.capture_range(1, &(0..0)), None);
        assert_eq!(plan.capture_range(2, &(0..0)), Some(0..0));
    }

    #[test]
    fn quantified_enclosing_work_aborts_before_every_source_or_frame_step() {
        let source = JsString::from(r"(?i:(([a]{0})?([b]{0})+)){2,3}");
        let mut work = Vec::new();
        assert!(
            RegExpZeroCountSequenceMatcher::compile_quantified_enclosing_validated_with_work(
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
        assert!(work.len() >= 4);
        assert_eq!(work[0], source.len() * 32 + 16);
        for stop in 0..work.len() {
            let mut seen = Vec::new();
            let result =
                RegExpZeroCountSequenceMatcher::compile_quantified_enclosing_validated_with_work(
                    &source,
                    true,
                    true,
                    |n| {
                        seen.push(n);
                        if seen.len() == stop + 1 {
                            Err(stop)
                        } else {
                            Ok(())
                        }
                    },
                );
            assert!(matches!(result, Err(n) if n==stop));
            assert_eq!(seen, work[..=stop]);
        }
    }

    #[test]
    fn quantified_choices_charge_each_backtracking_safe_arm_before_construction() {
        let source = JsString::from(r"(?i:(|([a]{0})+)){2,3}");
        let mut work = Vec::new();
        let plan =
            RegExpZeroCountSequenceMatcher::compile_quantified_enclosing_validated_with_work(
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
        assert_eq!(plan.capture_count(), 2);
        assert_eq!(plan.capture_range(0, &(0..0)), Some(0..0));
        assert_eq!(plan.capture_range(1, &(0..0)), None);
        for stop in 0..work.len() {
            let mut seen = Vec::new();
            let result =
                RegExpZeroCountSequenceMatcher::compile_quantified_enclosing_validated_with_work(
                    &source,
                    true,
                    true,
                    |n| {
                        seen.push(n);
                        if seen.len() == stop + 1 {
                            Err(stop)
                        } else {
                            Ok(())
                        }
                    },
                );
            assert!(matches!(result, Err(n) if n==stop));
            assert_eq!(seen, work[..=stop]);
        }
    }
    fn negative_impossible(
        source: &str,
        unicode: bool,
        sets: bool,
    ) -> Option<RegExpZeroCountSequenceMatcher> {
        RegExpZeroCountSequenceMatcher::compile_negative_impossible_validated_with_work(
            &JsString::from(source),
            unicode,
            sets,
            |_| Ok::<(), ()>(()),
        )
        .unwrap()
    }

    #[test]
    fn negative_impossible_assertions_succeed_empty_without_exporting_operand_captures() {
        for (source, captures) in [
            (r"(?!(?!(a{0})))", 1),
            (r"(?<!(?<!\uD800{0})(a))", 1),
            (r"(?!(?i:((?!(\3){0})(a))){2,3})", 3),
            (r"(?<!((?!)|((?<!a{0})(b))))", 3),
            (r"(?!(?=([\p{RGI_Emoji}]+)(?!(a{0}))))", 2),
        ] {
            for (unicode, sets) in [(false, false), (true, false), (true, true)] {
                if source.contains("RGI_Emoji") && !sets {
                    continue;
                }
                let plan = negative_impossible(source, unicode, sets).unwrap();
                assert_eq!(plan.capture_count(), captures, "{source}");
                for slot in 0..captures + 1 {
                    assert_eq!(plan.capture_range(slot, &(1..1)), None);
                }
                assert_eq!(
                    plan.find_with_work(&JsString::from("a😀!"), 1, |_| Ok::<(), ()>(()))
                        .unwrap(),
                    Some(1..1)
                );
            }
        }
    }

    #[test]
    fn negative_impossible_assertions_reject_possible_bodies_and_outside_contexts() {
        for source in [
            r"(?!)",
            r"(?!a)",
            r"(?<!a)",
            r"(?!(?!)|)",
            r"(?= (?!) )",
            r"((?!(?!)))",
            r"(?!(?!))a",
            r"a(?!(?!))",
            r"(?!(?!))|",
            r"(?!(?!))?",
            r"(?!(?!)){0}",
        ] {
            assert!(
                negative_impossible(source, true, true).is_none(),
                "{source}"
            );
        }
        assert!(negative_impossible(r"(?!(?!))", false, true).is_none());
    }

    #[test]
    fn negative_impossible_empty_matches_keep_unicode_boundaries_and_actual_work() {
        for (unicode, sets) in [(false, false), (true, false), (true, true)] {
            let plan = negative_impossible(r"(?!(?!(a{0})))", unicode, sets).unwrap();
            let input = JsString::from("😀");
            assert_eq!(
                plan.find_with_work(&input, 1, |n| {
                    assert_eq!(n, 1);
                    Ok::<(), ()>(())
                })
                .unwrap(),
                Some(usize::from(!unicode)..usize::from(!unicode))
            );
            assert_eq!(
                plan.find_with_work(&input, 3, |_| Ok::<(), ()>(()))
                    .unwrap(),
                None
            );
            assert_eq!(
                plan.find_with_work(&input, usize::MAX, |_| Err(11)),
                Err(11)
            );
        }
    }

    #[test]
    fn negative_impossible_source_depth_capture_masks_and_interruptions_are_explicit() {
        let source = JsString::from(r"(?<!(?i:((?!)|((?<!a{0})(b)))){2,3})");
        let mut work = Vec::new();
        let plan = RegExpZeroCountSequenceMatcher::compile_negative_impossible_validated_with_work(
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
        assert_eq!(*work.last().unwrap(), plan.capture_count());
        for stop in 0..work.len() {
            let mut seen = Vec::new();
            assert!(
                matches!(RegExpZeroCountSequenceMatcher::compile_negative_impossible_validated_with_work(
                &source,true,true,|n|{seen.push(n);if seen.len()==stop+1{Err(stop)}else{Ok(())}}
            ),Err(n) if n==stop)
            );
            assert_eq!(seen, work[..=stop]);
        }
        let source = format!(
            "(?<!{}(?!(a{{0}})){})",
            "(".repeat(100_000),
            ")+".repeat(100_000)
        );
        let plan = negative_impossible(&source, true, true).unwrap();
        assert_eq!(plan.capture_count(), 100_001);
        assert!((0..plan.capture_count()).all(|slot| plan.capture_range(slot, &(0..0)).is_none()));
    }
    fn enclosing_negative_impossible(source: &str) -> Option<RegExpZeroCountSequenceMatcher> {
        RegExpZeroCountSequenceMatcher::compile_enclosing_negative_impossible_validated_with_work(
            &JsString::from(source),
            true,
            true,
            |_| Ok::<(), ()>(()),
        )
        .unwrap()
    }

    #[test]
    fn enclosing_negative_impossible_masks_keep_required_and_optional_capture_roles() {
        for (source, mask) in [
            (r"((?!(?!(a{0}))))", &[true, false][..]),
            (r"((?!(?!(a{0}))))+", &[true, false][..]),
            (r"((?!(?!(a{0}))))?", &[false, false][..]),
            (r"(?i:(((?<!(?<!a{0})(b)))){2,3})", &[true, true, false][..]),
            (r"(((?!(?!(a{0}))))?)+", &[true, false, false][..]),
            (r"(((?!(?!(a{0}))))+)?", &[false, false, false][..]),
            (
                r"((?!(?i:((?!(\1){0})(a))){2,3})){1,}",
                &[true, false, false, false][..],
            ),
        ] {
            let plan = enclosing_negative_impossible(source).unwrap();
            assert_eq!(plan.defined.as_ref(), mask, "{source}");
        }
    }

    #[test]
    fn enclosing_negative_impossible_scope_excludes_siblings_choices_and_possible_bodies() {
        for source in [
            r"(?!(?!))",
            r"((?!(?!)))a",
            r"a((?!(?!)))",
            r"((?!(?!)))|",
            r"((?!a))",
            r"((?!(?!)|))",
            r"((?!(?!))|a{0})",
        ] {
            assert!(enclosing_negative_impossible(source).is_none(), "{source}");
        }
    }

    #[test]
    fn enclosing_negative_impossible_deep_masks_and_huge_counts_remain_source_sized() {
        let source = format!(
            "{}(?!(?!(a{{0}}))){}",
            "(".repeat(100_000),
            ")+".repeat(100_000)
        );
        let plan = enclosing_negative_impossible(&source).unwrap();
        assert_eq!(plan.capture_count(), 100_001);
        assert!(plan.defined[..100_000].iter().all(|&v| v));
        assert!(!plan.defined[100_000]);
        for (minimum, expected) in [("0".to_owned(), false), ("9".repeat(20_000), true)] {
            let source = format!("((?!(?!(a{{0}})))){{{minimum}}}");
            assert_eq!(
                enclosing_negative_impossible(&source)
                    .unwrap()
                    .defined
                    .as_ref(),
                &[expected, false]
            );
        }
    }

    #[test]
    fn enclosing_negative_impossible_interrupts_before_scope_body_and_mask_allocations() {
        let source = JsString::from(r"(?i:(((?!(?!(a{0}))))?)+)");
        let mut work = Vec::new();
        let plan=RegExpZeroCountSequenceMatcher::compile_enclosing_negative_impossible_validated_with_work(
            &source,true,true,|n|{work.push(n);Ok::<(),()>(())}
        ).unwrap().unwrap();
        assert_eq!(work[0], source.len() * 32 + 16);
        assert_eq!(*work.last().unwrap(), plan.capture_count());
        for stop in 0..work.len() {
            let mut seen = Vec::new();
            assert!(
                matches!(RegExpZeroCountSequenceMatcher::compile_enclosing_negative_impossible_validated_with_work(
                &source,true,true,|n|{seen.push(n);if seen.len()==stop+1{Err(stop)}else{Ok(())}}
            ),Err(n)if n==stop)
            );
            assert_eq!(seen, work[..=stop]);
        }
    }
    fn first_negative_impossible(source: &str) -> Option<RegExpZeroCountSequenceMatcher> {
        RegExpZeroCountSequenceMatcher::compile_first_negative_impossible_validated_with_work(
            &JsString::from(source),
            true,
            true,
            |_| Ok::<(), ()>(()),
        )
        .unwrap()
    }

    #[test]
    fn first_negative_impossible_choices_keep_first_masks_and_skip_all_later_slots() {
        for (source, mask) in [
            (r"(?!(?!(a{0})))|(b)", &[false, false][..]),
            (
                r"((?!(?!(a{0}))))+|([\p{RGI_Emoji}]+)",
                &[true, false, false][..],
            ),
            (
                r"((?!(?!(a{0}))))?|((b)(c))",
                &[false, false, false, false, false][..],
            ),
            (r"((?!(?!(\3){0})))|(a)", &[true, false, false][..]),
            (r"(?<!(?<!a{0})[\q{\||\)}])|(b)|(c)", &[false, false][..]),
        ] {
            assert_eq!(
                first_negative_impossible(source).unwrap().defined.as_ref(),
                mask,
                "{source}"
            );
        }
    }

    #[test]
    fn first_negative_impossible_choices_reject_nonroot_and_unproved_first_arms() {
        for source in [
            r"(?!(?!))",
            r"a|(?!(?!))",
            r"|(?!(?!))",
            r"(?!a)|(b)",
            r"(((?!(?!)))|a)+",
            r"(?!(?!)|)|(b)",
            r"((?!(?!)))a|(b)",
        ] {
            assert!(first_negative_impossible(source).is_none(), "{source}");
        }
    }

    #[test]
    fn first_negative_impossible_choices_keep_large_lexical_inventories_flat() {
        let source = format!(
            "(?!(?!(a{{0}})))|{}b{}",
            "(".repeat(100_000),
            ")".repeat(100_000)
        );
        let plan = first_negative_impossible(&source).unwrap();
        assert_eq!(plan.capture_count(), 100_001);
        assert!(plan.defined.iter().all(|&v| !v));
    }

    #[test]
    fn first_negative_impossible_choices_interrupt_before_source_projection_and_mask_growth() {
        let source = JsString::from(r"((?!(?!(a{0}))))+|((b)(c))");
        let mut work = Vec::new();
        let plan =
            RegExpZeroCountSequenceMatcher::compile_first_negative_impossible_validated_with_work(
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
        assert_eq!(*work.last().unwrap(), plan.capture_count());
        for stop in 0..work.len() {
            let mut seen = Vec::new();
            assert!(
                matches!(RegExpZeroCountSequenceMatcher::compile_first_negative_impossible_validated_with_work(
                &source,true,true,|n|{seen.push(n);if seen.len()==stop+1{Err(stop)}else{Ok(())}}
            ),Err(n)if n==stop)
            );
            assert_eq!(seen, work[..=stop]);
        }
    }
    fn negative_impossible_sequence(
        source: &str,
        unicode: bool,
        sets: bool,
    ) -> Option<RegExpZeroCountSequenceMatcher> {
        RegExpZeroCountSequenceMatcher::compile_negative_impossible_sequence_validated_with_work(
            &JsString::from(source),
            unicode,
            sets,
            |_| Ok::<(), ()>(()),
        )
        .unwrap()
    }

    #[test]
    fn negative_impossible_sequences_preserve_interleaved_masks_and_original_bindings() {
        for (source, mask) in [
            (r"a{0}(?!(?!(b{0})))(c{0})", &[false, true][..]),
            (
                r"(a{0})(?<!(?<!b{0})(c))((d{0}))?",
                &[true, false, false, false][..],
            ),
            (r"(a{0})(?!(?!(\3){0}))(b{0})", &[true, false, true][..]),
            (
                r"(?i:(((?!(?!(a{0}))))?)+)(b{0})",
                &[true, false, false, true][..],
            ),
            (r"\x28{0}\u007C{0}(?!(?!))[\p{RGI_Emoji}]{0}", &[][..]),
        ] {
            assert_eq!(
                negative_impossible_sequence(source, true, true)
                    .unwrap()
                    .defined
                    .as_ref(),
                mask,
                "{source}"
            );
        }
    }

    #[test]
    fn negative_impossible_sequences_require_only_independently_empty_root_terms() {
        for source in [
            r"(?!(?!))",
            r"a{0}b{0}",
            r"a(?!(?!))",
            r"(?!(?!))a",
            r"(?!(?!))|a{0}",
            r"(?!a)(b{0})",
            r"(a{0})(?!(?!))\1",
            r"(?!(?!))(?=a)",
        ] {
            assert!(
                negative_impossible_sequence(source, true, true).is_none(),
                "{source}"
            );
        }
        assert!(negative_impossible_sequence(r"(?!(?!))a{0}", false, true).is_none());
    }

    #[test]
    fn negative_impossible_sequences_share_mode_aware_surrogate_atom_boundaries() {
        for mode in [(false, false), (true, false), (true, true)] {
            let plan = negative_impossible_sequence(r"\uD83D\uDE00{0}(?!(?!))", mode.0, mode.1);
            assert_eq!(plan.is_some(), mode.0);
        }
        for source in [
            r"\u{D800}{0}(?!(?!))",
            r"\uD800{0}(?!(?!))",
            r"😀{0}(?!(?!))",
        ] {
            let plan = negative_impossible_sequence(source, true, true).unwrap();
            assert_eq!(
                plan.find_with_work(&JsString::from("😀"), 1, |_| Ok::<(), ()>(()))
                    .unwrap(),
                Some(0..0)
            );
        }
        assert!(negative_impossible_sequence(r"\u{D800}\uDC00{0}(?!(?!))", true, true).is_none());
    }

    #[test]
    fn negative_impossible_sequence_masks_and_interruptions_stay_source_sized() {
        let source = format!("{}(a{{0}})", r"(?!(?!(a{0})))".repeat(20_000));
        let plan = negative_impossible_sequence(&source, true, true).unwrap();
        assert_eq!(plan.capture_count(), 20_001);
        assert!(plan.defined[..20_000].iter().all(|&v| !v));
        assert!(plan.defined[20_000]);
        let source = JsString::from(r"(a{0})(?!(?!(\3){0}))(b{0})");
        let mut work = Vec::new();
        assert!(RegExpZeroCountSequenceMatcher::compile_negative_impossible_sequence_validated_with_work(
            &source,true,true,|n|{work.push(n);Ok::<(),()>(())}
        ).unwrap().is_some());
        assert_eq!(work[0], source.len() * 16 + 16);
        assert!(work.iter().sum::<usize>() < source.len() * 1024);
        for stop in 0..work.len() {
            let mut seen = Vec::new();
            assert!(
                matches!(RegExpZeroCountSequenceMatcher::compile_negative_impossible_sequence_validated_with_work(
                &source,true,true,|n|{seen.push(n);if seen.len()==stop+1{Err(stop)}else{Ok(())}}
            ),Err(n)if n==stop)
            );
            assert_eq!(seen, work[..=stop]);
        }
    }
    fn enclosing_negative_impossible_sequence(
        source: &str,
    ) -> Option<RegExpZeroCountSequenceMatcher> {
        RegExpZeroCountSequenceMatcher::compile_enclosing_negative_impossible_sequence_validated_with_work(
            &JsString::from(source),true,true,|_|Ok::<(),()>(()),
        ).unwrap()
    }

    #[test]
    fn enclosing_negative_impossible_sequences_preserve_required_optional_and_inner_masks() {
        for (source, mask) in [
            (r"((a{0})(?!(?!(b{0}))))+", &[true, true, false][..]),
            (r"((a{0})(?!(?!(b{0}))))?", &[false, false, false][..]),
            (
                r"(((a{0})(?!(?!(b{0}))))?)+",
                &[true, false, false, false][..],
            ),
            (
                r"(?i:((a{0})(?!(?!(\4){0}))(b{0}))){2,3}",
                &[true, true, false, true][..],
            ),
        ] {
            assert_eq!(
                enclosing_negative_impossible_sequence(source)
                    .unwrap()
                    .defined
                    .as_ref(),
                mask,
                "{source}"
            );
        }
    }

    #[test]
    fn enclosing_negative_impossible_sequences_exclude_siblings_choices_and_unproved_bodies() {
        for source in [
            r"a{0}(?!(?!))",
            r"((a{0})(?!(?!)))a",
            r"a((a{0})(?!(?!)))",
            r"((a{0})(?!(?!)))|",
            r"((a)(?!(?!)))",
            r"((a{0})(?!(?!))|a)",
            r"((?!(?!)))",
        ] {
            assert!(
                enclosing_negative_impossible_sequence(source).is_none(),
                "{source}"
            );
        }
    }

    #[test]
    fn enclosing_negative_impossible_sequence_depth_and_bounds_keep_flat_storage() {
        let source = format!(
            "{}(a{{0}})(?!(?!(b{{0}}))){}",
            "(".repeat(100_000),
            ")+".repeat(100_000)
        );
        let plan = enclosing_negative_impossible_sequence(&source).unwrap();
        assert_eq!(plan.capture_count(), 100_002);
        assert!(plan.defined[..100_001].iter().all(|&v| v));
        assert!(!plan.defined[100_001]);
        for (minimum, expected) in [("0".to_owned(), false), ("9".repeat(20_000), true)] {
            let body = r"((a{0})(?!(?!(b{0}))))";
            let source = format!("{body}{{{minimum}}}");
            assert_eq!(
                enclosing_negative_impossible_sequence(&source)
                    .unwrap()
                    .defined
                    .as_ref(),
                &[expected, expected, false]
            );
        }
    }

    #[test]
    fn enclosing_negative_impossible_sequences_interrupt_before_scope_body_and_mask_steps() {
        let source = JsString::from(r"(?i:((a{0})(?!(?!(b{0}))))?)+");
        let mut work = Vec::new();
        let plan=RegExpZeroCountSequenceMatcher::compile_enclosing_negative_impossible_sequence_validated_with_work(
            &source,true,true,|n|{work.push(n);Ok::<(),()>(())}
        ).unwrap().unwrap();
        assert_eq!(work[0], source.len() * 32 + 16);
        assert_eq!(*work.last().unwrap(), plan.capture_count());
        for stop in 0..work.len() {
            let mut seen = Vec::new();
            assert!(
                matches!(RegExpZeroCountSequenceMatcher::compile_enclosing_negative_impossible_sequence_validated_with_work(
                &source,true,true,|n|{seen.push(n);if seen.len()==stop+1{Err(stop)}else{Ok(())}}
            ),Err(n)if n==stop)
            );
            assert_eq!(seen, work[..=stop]);
        }
    }
    fn optional_impossible(source: &str) -> Option<RegExpZeroCountSequenceMatcher> {
        RegExpZeroCountSequenceMatcher::compile_optional_impossible_validated_with_work(
            &JsString::from(source),
            true,
            true,
            |_| Ok::<(), ()>(()),
        )
        .unwrap()
    }

    #[test]
    fn optional_impossible_bodies_keep_outside_required_and_skipped_capture_masks() {
        for (source, mask) in [
            (r"((?!(a{0})))?", &[false, false][..]),
            (r"(((?!(a{0})))?)+", &[true, false, false][..]),
            (r"(?i:(((?!(\1){0}))*){2,3})", &[true, false, false][..]),
            (r"((?=(?!(a{0})))){0,3}", &[false, false][..]),
            (r"((?!(?!(?!(a{0})))))?", &[false, false][..]),
            (
                r"((?!(?!(?!(a{0}))))|(?<!(?!(?!(b{0})))))*",
                &[false, false, false][..],
            ),
        ] {
            assert_eq!(
                optional_impossible(source).unwrap().defined.as_ref(),
                mask,
                "{source}"
            );
        }
    }

    #[test]
    fn optional_impossible_proof_excludes_required_only_conditional_and_outside_contexts() {
        for source in [
            r"(?!)",
            r"((?!))+",
            r"((?!)){2,3}",
            r"((?!a))?",
            r"(a)?",
            r"((?!))?a",
            r"a((?!))?",
            r"((?!))?|a",
            r"((?!)|a)?",
        ] {
            assert!(optional_impossible(source).is_none(), "{source}");
        }
    }

    #[test]
    fn optional_impossible_depth_masks_and_large_bounds_remain_source_sized() {
        let source = format!(
            "{}((?!(a{{0}})))?{}",
            "(".repeat(100_000),
            ")+".repeat(100_000)
        );
        let plan = optional_impossible(&source).unwrap();
        assert_eq!(plan.capture_count(), 100_002);
        assert!(plan.defined[..100_000].iter().all(|&v| v));
        assert!(plan.defined[100_000..].iter().all(|&v| !v));
        let source = format!("((?!(a{{0}}))){{0,{}}}", "9".repeat(20_000));
        assert_eq!(
            optional_impossible(&source).unwrap().defined.as_ref(),
            &[false, false]
        );
    }

    #[test]
    fn optional_impossible_work_interrupts_before_scope_operand_mask_and_input_access() {
        let source = JsString::from(r"(?i:(((?!(a{0})))?)+)");
        let mut work = Vec::new();
        let plan = RegExpZeroCountSequenceMatcher::compile_optional_impossible_validated_with_work(
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
        assert_eq!(work[0], source.len() * 32 + 16);
        assert_eq!(*work.last().unwrap(), plan.capture_count());
        for stop in 0..work.len() {
            let mut seen = Vec::new();
            assert!(
                matches!(RegExpZeroCountSequenceMatcher::compile_optional_impossible_validated_with_work(
                &source,true,true,|n|{seen.push(n);if seen.len()==stop+1{Err(stop)}else{Ok(())}}
            ),Err(n)if n==stop)
            );
            assert_eq!(seen, work[..=stop]);
        }
        assert_eq!(
            plan.find_with_work(&JsString::from("😀"), 1, |n| {
                assert_eq!(n, 1);
                Ok::<(), ()>(())
            })
            .unwrap(),
            Some(0..0)
        );
        assert_eq!(
            plan.find_with_work(&JsString::from(""), usize::MAX, |_| Err(23)),
            Err(23)
        );
    }
    fn optional_impossible_sequence(source: &str) -> Option<RegExpZeroCountSequenceMatcher> {
        RegExpZeroCountSequenceMatcher::compile_optional_impossible_sequence_validated_with_work(
            &JsString::from(source),
            true,
            true,
            |_| Ok::<(), ()>(()),
        )
        .unwrap()
    }

    #[test]
    fn optional_impossible_sequences_preserve_interleaved_required_and_skipped_masks() {
        for (source, mask) in [
            (
                r"(a{0})((?!(b{0})))?(c{0})",
                &[true, false, false, true][..],
            ),
            (
                r"(?i:(((?!(a{0})))?)+)(b{0})",
                &[true, false, false, true][..],
            ),
            (
                r"(a{0})((?!(\4){0}))?(b{0})",
                &[true, false, false, true][..],
            ),
            (
                r"((?!(a{0})))?(?!(?!(b{0})))(c{0})",
                &[false, false, false, true][..],
            ),
            (r"\uD83D\uDE00{0}((?!))?[\p{RGI_Emoji}]{0}", &[false][..]),
        ] {
            assert_eq!(
                optional_impossible_sequence(source)
                    .unwrap()
                    .defined
                    .as_ref(),
                mask,
                "{source}"
            );
        }
    }

    #[test]
    fn optional_impossible_sequences_exclude_unproved_terms_root_choices_and_missing_optional_proofs()
     {
        for source in [
            r"((?!))?",
            r"a{0}b{0}",
            r"(?!(?!))(a{0})",
            r"a((?!))?",
            r"((?!))?a",
            r"((?!))?|a{0}",
            r"((?!a))?(b{0})",
            r"((?!))?(a{0})\1",
        ] {
            assert!(optional_impossible_sequence(source).is_none(), "{source}");
        }
    }

    #[test]
    fn optional_impossible_sequences_keep_many_term_masks_and_unicode_actual_work_flat() {
        let source = format!("{}(a{{0}})", r"((?!(a{0})))?".repeat(20_000));
        let plan = optional_impossible_sequence(&source).unwrap();
        assert_eq!(plan.capture_count(), 40_001);
        assert!(plan.defined[..40_000].iter().all(|&v| !v));
        assert!(plan.defined[40_000]);
        assert_eq!(
            plan.find_with_work(&JsString::from("😀"), 1, |n| {
                assert_eq!(n, 1);
                Ok::<(), ()>(())
            })
            .unwrap(),
            Some(0..0)
        );
    }

    #[test]
    fn optional_impossible_sequences_interrupt_before_terms_proofs_and_mask_growth() {
        let source = JsString::from(r"(a{0})((?!(\4){0}))?(b{0})");
        let mut work = Vec::new();
        assert!(RegExpZeroCountSequenceMatcher::compile_optional_impossible_sequence_validated_with_work(
            &source,true,true,|n|{work.push(n);Ok::<(),()>(())}
        ).unwrap().is_some());
        assert_eq!(work[0], source.len() * 16 + 16);
        for stop in 0..work.len() {
            let mut seen = Vec::new();
            assert!(
                matches!(RegExpZeroCountSequenceMatcher::compile_optional_impossible_sequence_validated_with_work(
                &source,true,true,|n|{seen.push(n);if seen.len()==stop+1{Err(stop)}else{Ok(())}}
            ),Err(n)if n==stop)
            );
            assert_eq!(seen, work[..=stop]);
        }
    }
    fn enclosing_optional_impossible_sequence(
        source: &str,
    ) -> Option<RegExpZeroCountSequenceMatcher> {
        RegExpZeroCountSequenceMatcher::compile_enclosing_optional_impossible_sequence_validated_with_work(
            &JsString::from(source),true,true,|_|Ok::<(),()>(()),
        ).unwrap()
    }

    #[test]
    fn enclosing_optional_impossible_sequences_keep_required_optional_and_interleaved_masks() {
        for (source, mask) in [
            (
                r"((a{0})((?!(b{0})))?(c{0}))+",
                &[true, true, false, false, true][..],
            ),
            (
                r"((a{0})((?!(b{0})))?(c{0}))?",
                &[false, false, false, false, false][..],
            ),
            (
                r"(((a{0})((?!(b{0})))?(c{0}))?)+",
                &[true, false, false, false, false, false][..],
            ),
            (
                r"(?im-s:((a{0})((?!(\5){0}))?(b{0}))){2,3}",
                &[true, true, false, false, true][..],
            ),
        ] {
            assert_eq!(
                enclosing_optional_impossible_sequence(source)
                    .unwrap()
                    .defined
                    .as_ref(),
                mask,
                "{source}"
            );
        }
    }

    #[test]
    fn enclosing_optional_impossible_sequences_exclude_siblings_choices_and_unproved_bodies() {
        for source in [
            r"(a{0})((?!))?",
            r"((a{0})((?!))?)a",
            r"a((a{0})((?!))?)",
            r"((a{0})((?!))?)|",
            r"((a)((?!))?)",
            r"((a{0})((?!))?|a)",
            r"(((?!))?)",
        ] {
            assert!(
                enclosing_optional_impossible_sequence(source).is_none(),
                "{source}"
            );
        }
    }

    #[test]
    fn enclosing_optional_impossible_sequence_depth_and_large_bounds_remain_flat() {
        let source = format!(
            "{}(a{{0}})((?!(b{{0}})))?(c{{0}}){}",
            "(".repeat(100_000),
            ")+".repeat(100_000)
        );
        let plan = enclosing_optional_impossible_sequence(&source).unwrap();
        assert_eq!(plan.capture_count(), 100_004);
        assert!(plan.defined[..100_001].iter().all(|&v| v));
        assert_eq!(&plan.defined[100_001..], &[false, false, true]);
        let body = r"((a{0})((?!(b{0})))?(c{0}))";
        for (minimum, mask) in [
            ("0".to_owned(), &[false, false, false, false, false][..]),
            ("9".repeat(20_000), &[true, true, false, false, true][..]),
        ] {
            let source = format!("{body}{{{minimum}}}");
            assert_eq!(
                enclosing_optional_impossible_sequence(&source)
                    .unwrap()
                    .defined
                    .as_ref(),
                mask
            );
        }
    }

    #[test]
    fn enclosing_optional_impossible_sequences_interrupt_before_scope_body_and_mask_steps() {
        let source = JsString::from(r"(?i:((a{0})((?!(b{0})))?(c{0}))?)+");
        let mut work = Vec::new();
        let plan=RegExpZeroCountSequenceMatcher::compile_enclosing_optional_impossible_sequence_validated_with_work(
            &source,true,true,|n|{work.push(n);Ok::<(),()>(())}
        ).unwrap().unwrap();
        assert_eq!(work[0], source.len() * 32 + 16);
        assert_eq!(*work.last().unwrap(), plan.capture_count());
        for stop in 0..work.len() {
            let mut seen = Vec::new();
            assert!(
                matches!(RegExpZeroCountSequenceMatcher::compile_enclosing_optional_impossible_sequence_validated_with_work(
                &source,true,true,|n|{seen.push(n);if seen.len()==stop+1{Err(stop)}else{Ok(())}}
            ),Err(n)if n==stop)
            );
            assert_eq!(seen, work[..=stop]);
        }
    }
    fn first_optional_impossible(source: &str) -> Option<RegExpZeroCountSequenceMatcher> {
        RegExpZeroCountSequenceMatcher::compile_first_optional_impossible_validated_with_work(
            &JsString::from(source),
            true,
            true,
            |_| Ok::<(), ()>(()),
        )
        .unwrap()
    }

    #[test]
    fn first_optional_impossible_choices_keep_first_masks_and_all_later_undefined_slots() {
        for (source, mask) in [
            (r"((?!(a{0})))?|(b)", &[false, false, false][..]),
            (r"(a{0})((?!(b{0})))?|(c)", &[true, false, false, false][..]),
            (
                r"((a{0})((?!(b{0})))?(c{0}))+|(d)",
                &[true, true, false, false, true, false][..],
            ),
            (
                r"(a{0})((?!(\4){0}))?|(b)",
                &[true, false, false, false][..],
            ),
            (r"((?!))?|[\q{\||\)}]+(b)", &[false, false][..]),
        ] {
            assert_eq!(
                first_optional_impossible(source).unwrap().defined.as_ref(),
                mask,
                "{source}"
            );
        }
    }

    #[test]
    fn first_optional_impossible_choices_exclude_unproved_first_arms_and_repeat_contexts() {
        for source in [
            r"((?!))?",
            r"a|((?!))?",
            r"|((?!))?",
            r"((?!a))?|b",
            r"(((?!))?|a)+",
            r"((?!))?a|(b)",
        ] {
            assert!(first_optional_impossible(source).is_none(), "{source}");
        }
    }

    #[test]
    fn first_optional_impossible_choices_keep_large_later_inventories_flat() {
        let source = format!(
            "((?!))?|{}[\\p{{RGI_Emoji}}]{}",
            "(".repeat(100_000),
            ")".repeat(100_000)
        );
        let plan = first_optional_impossible(&source).unwrap();
        assert_eq!(plan.capture_count(), 100_001);
        assert!(plan.defined.iter().all(|&v| !v));
    }

    #[test]
    fn first_optional_impossible_choices_interrupt_before_source_first_arm_and_mask_growth() {
        let source = JsString::from(r"(a{0})((?!(\4){0}))?|(b)");
        let mut work = Vec::new();
        let plan =
            RegExpZeroCountSequenceMatcher::compile_first_optional_impossible_validated_with_work(
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
        assert_eq!(*work.last().unwrap(), plan.capture_count());
        for stop in 0..work.len() {
            let mut seen = Vec::new();
            assert!(
                matches!(RegExpZeroCountSequenceMatcher::compile_first_optional_impossible_validated_with_work(
                &source,true,true,|n|{seen.push(n);if seen.len()==stop+1{Err(stop)}else{Ok(())}}
            ),Err(n)if n==stop)
            );
            assert_eq!(seen, work[..=stop]);
        }
    }
    fn positive_optional_impossible(source: &str) -> Option<RegExpZeroCountSequenceMatcher> {
        RegExpZeroCountSequenceMatcher::compile_positive_optional_impossible_validated_with_work(
            &JsString::from(source),
            true,
            true,
            |_| Ok::<(), ()>(()),
        )
        .unwrap()
    }

    #[test]
    fn positive_optional_impossible_assertions_export_masks_in_both_directions() {
        for (source, mask) in [
            (r"(?=((?!(a{0})))?)", &[false, false][..]),
            (r"(?<=(((?!(a{0})))?)+)", &[true, false, false][..]),
            (
                r"(?=(a{0})((?!(b{0})))?(c{0}))",
                &[true, false, false, true][..],
            ),
            (
                r"(?<=((a{0})((?!(b{0})))?(c{0}))+)",
                &[true, true, false, false, true][..],
            ),
            (
                r"(?=(a{0})((?!(\4){0}))?|(b))",
                &[true, false, false, false][..],
            ),
        ] {
            assert_eq!(
                positive_optional_impossible(source)
                    .unwrap()
                    .defined
                    .as_ref(),
                mask,
                "{source}"
            );
        }
    }

    #[test]
    fn positive_optional_impossible_assertions_exclude_conditions_siblings_and_outside_contexts() {
        for source in [
            r"(?=a)",
            r"(?<=a)",
            r"(?=a{0})",
            r"(?=((?!a))?)",
            r"a(?=((?!))?)",
            r"(?=((?!))?)a",
            r"((?=((?!))?))",
            r"(?=((?!))?)|a",
            r"(?!(a{0})((?!))?)",
        ] {
            assert!(positive_optional_impossible(source).is_none(), "{source}");
        }
    }

    #[test]
    fn positive_optional_impossible_depth_and_actual_unicode_work_remain_flat() {
        let source = format!(
            "(?<={}((?!(a{{0}})))?{})",
            "(".repeat(100_000),
            ")+".repeat(100_000)
        );
        let plan = positive_optional_impossible(&source).unwrap();
        assert_eq!(plan.capture_count(), 100_002);
        assert!(plan.defined[..100_000].iter().all(|&v| v));
        assert_eq!(&plan.defined[100_000..], &[false, false]);
        let matched = plan
            .find_with_work(&JsString::from("😀"), 1, |n| {
                assert_eq!(n, 1);
                Ok::<(), ()>(())
            })
            .unwrap()
            .unwrap();
        assert_eq!(matched, 0..0);
        assert_eq!(plan.capture_range(0, &matched), Some(0..0));
        assert_eq!(plan.capture_range(100_001, &matched), None);
    }

    #[test]
    fn positive_optional_impossible_assertions_interrupt_before_source_body_and_input_access() {
        let source = JsString::from(r"(?<=(a{0})((?!(\4){0}))?|(b))");
        let mut work = Vec::new();
        let plan=RegExpZeroCountSequenceMatcher::compile_positive_optional_impossible_validated_with_work(
            &source,true,true,|n|{work.push(n);Ok::<(),()>(())}
        ).unwrap().unwrap();
        assert_eq!(work[0], source.len() * 8 + 16);
        for stop in 0..work.len() {
            let mut seen = Vec::new();
            assert!(
                matches!(RegExpZeroCountSequenceMatcher::compile_positive_optional_impossible_validated_with_work(
                &source,true,true,|n|{seen.push(n);if seen.len()==stop+1{Err(stop)}else{Ok(())}}
            ),Err(n)if n==stop)
            );
            assert_eq!(seen, work[..=stop]);
        }
        assert_eq!(
            plan.find_with_work(&JsString::from(""), usize::MAX, |_| Err(29)),
            Err(29)
        );
    }

    fn optional_known_impossible(source: &str) -> Option<RegExpZeroCountSequenceMatcher> {
        RegExpZeroCountSequenceMatcher::compile_optional_known_impossible_validated_with_work(
            &JsString::from(source),
            true,
            true,
            |_| Ok::<(), ()>(()),
        )
        .unwrap()
    }

    #[test]
    fn optional_known_impossible_bodies_keep_required_prefix_and_all_skipped_descendant_masks() {
        for (source, mask) in [
            (r"((?![\p{RGI_Emoji}]*))?", &[false][..]),
            (r"(((?![\p{RGI_Emoji}]*))?)+", &[true, false][..]),
            (r"(?i:(((?!(a|b)?))*){2,3})", &[true, false, false][..]),
            (r"((?!(?=(a)*)))?", &[false, false][..]),
            (r"((?![\p{RGI_Emoji}]*)|(?!(a{0})))?", &[false, false][..]),
            (
                r"(((?![\p{RGI_Emoji}]*)|(?!(a{0})))?)+",
                &[true, false, false][..],
            ),
        ] {
            assert_eq!(
                optional_known_impossible(source).unwrap().defined.as_ref(),
                mask,
                "{source}"
            );
        }
    }

    #[test]
    fn optional_known_impossible_proofs_reject_required_only_possible_and_outside_contexts() {
        for source in [
            r"(?!a*)",
            r"((?!a*))+",
            r"((?!a*)){2,3}",
            r"((?!a))?",
            r"(a)?",
            r"((?!a*))?a",
            r"a((?!a*))?",
            r"((?!a*))?|a",
            r"((?!a*)|a)?",
        ] {
            assert!(optional_known_impossible(source).is_none(), "{source}");
        }
    }

    #[test]
    fn optional_known_impossible_depth_and_large_bounds_keep_masks_and_work_flat() {
        let source = format!(
            "{}((?![\\p{{RGI_Emoji}}]*))?{}",
            "(".repeat(100_000),
            ")+".repeat(100_000)
        );
        let plan = optional_known_impossible(&source).unwrap();
        assert_eq!(plan.capture_count(), 100_001);
        assert!(plan.defined[..100_000].iter().all(|&v| v));
        assert!(!plan.defined[100_000]);
        let source = format!(r"((?![\p{{RGI_Emoji}}]*)){{0,{}}}", "9".repeat(20_000));
        assert_eq!(
            optional_known_impossible(&source).unwrap().defined.as_ref(),
            &[false]
        );
    }

    #[test]
    fn optional_known_impossible_work_interrupts_before_scope_body_mask_and_input_access() {
        let source = JsString::from(r"(?i:(((?![\p{RGI_Emoji}]*))?)+)");
        let mut work = Vec::new();
        let plan =
            RegExpZeroCountSequenceMatcher::compile_optional_known_impossible_validated_with_work(
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
        assert_eq!(work[0], source.len() * 32 + 16);
        assert_eq!(*work.last().unwrap(), plan.capture_count());
        for stop in 0..work.len() {
            let mut seen = Vec::new();
            assert!(matches!(
            RegExpZeroCountSequenceMatcher::compile_optional_known_impossible_validated_with_work(
                &source,true,true,|n|{seen.push(n);if seen.len()==stop+1{Err(stop)}else{Ok(())}}
            ),Err(n)if n==stop));
            assert_eq!(seen, work[..=stop]);
        }
        assert_eq!(
            plan.find_with_work(&JsString::from("😀"), 1, |n| {
                assert_eq!(n, 1);
                Ok::<(), ()>(())
            })
            .unwrap(),
            Some(0..0)
        );
        assert_eq!(
            plan.find_with_work(&JsString::from(""), usize::MAX, |_| Err(37)),
            Err(37)
        );
    }
    fn optional_known_sequence(source: &str) -> Option<RegExpZeroCountSequenceMatcher> {
        RegExpZeroCountSequenceMatcher::compile_optional_known_impossible_sequence_validated_with_work(
            &JsString::from(source), true, true, |_| Ok::<(), ()>(()),
        ).unwrap()
    }

    #[test]
    fn optional_known_sequences_preserve_interleaved_required_skipped_and_erased_global_slots() {
        for (source, mask) in [
            (
                r"((?![\p{RGI_Emoji}]*))?((a)\3){0}",
                &[false, false, false][..],
            ),
            (
                r"((a)\4){0}(((?![\p{RGI_Emoji}]*))?)+([b]{0})+",
                &[false, false, true, false, true][..],
            ),
            (
                r"((?!a*)|(?!(b{0})))?((?![\p{RGI_Emoji}]*))*",
                &[false, false, false][..],
            ),
            (r"((?!(?=(a)*)))?(?!(?!(b{0})))", &[false, false, false][..]),
            (r"((?![\p{RGI_Emoji}]*))?\uD83D\uDE00{0}", &[false][..]),
        ] {
            let plan = optional_known_sequence(source).unwrap_or_else(|| panic!("{source}"));
            assert_eq!(plan.defined.as_ref(), mask, "{source}");
        }
    }

    #[test]
    fn optional_known_sequences_reject_root_choices_possible_terms_and_continuation_sensitive_arms()
    {
        for source in [
            r"((?!a*))?",
            r"((?!a*))?a",
            r"a((?!a*))?",
            r"((?!a*))?|([a]{0})+",
            r"((?!a*))?([a]*)+",
            r"((?!a*))?(|a)+",
            r"((?!a))?([a]{0})+",
            r"(?=a)((?!a*))?",
            r"[a]{0}[b]{0}",
        ] {
            assert!(optional_known_sequence(source).is_none(), "{source}");
        }
        assert!(RegExpZeroCountSequenceMatcher::compile_optional_known_impossible_sequence_validated_with_work(
            &JsString::from(r"((?!a*))?[b]{0}"), false, true, |_| Ok::<(), ()>(()),
        ).unwrap().is_none());
    }

    #[test]
    fn optional_known_sequences_keep_many_disjoint_masks_flat_and_bounds_symbolic() {
        let source = r"(((?![\p{RGI_Emoji}]*))?)+((a)\1){0}".repeat(20_000);
        let plan = optional_known_sequence(&source).unwrap();
        assert_eq!(plan.capture_count(), 80_000);
        assert!(
            plan.defined
                .chunks_exact(4)
                .all(|mask| mask == [true, false, false, false])
        );
        let source = format!(
            r"((?![\p{{RGI_Emoji}}]*)){{0,{}}}([a]{{0}})+",
            "9".repeat(20_000)
        );
        assert_eq!(
            optional_known_sequence(&source).unwrap().defined.as_ref(),
            &[false, true]
        );
    }

    #[test]
    fn optional_known_sequence_work_interrupts_before_each_segment_proof_and_mask() {
        let source = JsString::from(r"((?![\p{RGI_Emoji}]*))?((a)\1){0}([b]{0})+");
        let mut work = Vec::new();
        let plan=RegExpZeroCountSequenceMatcher::compile_optional_known_impossible_sequence_validated_with_work(
            &source,true,true,|n|{work.push(n);Ok::<(),()>(())},
        ).unwrap().unwrap();
        assert_eq!(work[0], source.len() * 16 + 16);
        for stop in 0..work.len() {
            let mut seen = Vec::new();
            assert!(matches!(
            RegExpZeroCountSequenceMatcher::compile_optional_known_impossible_sequence_validated_with_work(
                &source,true,true,|n|{seen.push(n);if seen.len()==stop+1{Err(stop)}else{Ok(())}},
            ),Err(n)if n==stop));
            assert_eq!(seen, work[..=stop]);
        }
        assert_eq!(
            plan.find_with_work(&JsString::from("😀"), 1, |n| {
                assert_eq!(n, 1);
                Ok::<(), ()>(())
            })
            .unwrap(),
            Some(0..0)
        );
        assert_eq!(
            plan.find_with_work(&JsString::from(""), usize::MAX, |_| Err(41)),
            Err(41)
        );
    }
    fn enclosing_optional_known_sequence(source: &str) -> Option<RegExpZeroCountSequenceMatcher> {
        RegExpZeroCountSequenceMatcher::compile_enclosing_optional_known_impossible_sequence_validated_with_work(
            &JsString::from(source),true,true,|_|Ok::<(),()>(()),
        ).unwrap()
    }

    #[test]
    fn enclosing_optional_known_sequences_preserve_required_and_optional_prefix_masks() {
        for (source, mask) in [
            (
                r"(((?![\p{RGI_Emoji}]*))?([a]{0})+)+",
                &[true, false, true][..],
            ),
            (
                r"(((?![\p{RGI_Emoji}]*))?([a]{0})+)?",
                &[false, false, false][..],
            ),
            (
                r"((((?![\p{RGI_Emoji}]*))?([a]{0})+)*)+",
                &[true, false, false, false][..],
            ),
            (
                r"(?im-s:(((?![\p{RGI_Emoji}]*))?((a)\4){0})){2,3}?",
                &[true, false, false, false][..],
            ),
            (
                r"(((?!a*)|(?!(b{0})))?((?![\p{RGI_Emoji}]*))*){0001,0002}",
                &[true, false, false, false][..],
            ),
        ] {
            assert_eq!(
                enclosing_optional_known_sequence(source)
                    .unwrap()
                    .defined
                    .as_ref(),
                mask,
                "{source}"
            );
        }
    }

    #[test]
    fn enclosing_optional_known_sequences_reject_unwrapped_single_and_possible_contexts() {
        for source in [
            r"((?!a*))?[a]{0}",
            r"(((?!a*))?)+",
            r"(((?!a*))?a)+",
            r"(((?!a*))?[a]{0})+a",
            r"a(((?!a*))?[a]{0})+",
            r"(((?!a*))?[a]{0})+|a",
            r"(((?!a*))?(|a)+)+",
        ] {
            assert!(
                enclosing_optional_known_sequence(source).is_none(),
                "{source}"
            );
        }
    }

    #[test]
    fn enclosing_optional_known_sequences_keep_deep_masks_and_huge_minima_flat() {
        let body = r"((?![\p{RGI_Emoji}]*))?([a]{0})+";
        let source = format!("{}{body}{}", "(".repeat(100_000), ")+".repeat(100_000));
        let plan = enclosing_optional_known_sequence(&source).unwrap();
        assert_eq!(plan.capture_count(), 100_002);
        assert!(plan.defined[..100_000].iter().all(|&v| v));
        assert_eq!(&plan.defined[100_000..], &[false, true]);
        let source = format!("({body}){{{}}}", "9".repeat(20_000));
        assert_eq!(
            enclosing_optional_known_sequence(&source)
                .unwrap()
                .defined
                .as_ref(),
            &[true, false, true]
        );
    }

    #[test]
    fn enclosing_optional_known_sequence_work_interrupts_before_scope_body_and_full_mask() {
        let source = JsString::from(r"(?im-s:(((?![\p{RGI_Emoji}]*))?([a]{0})+)*)+");
        let mut work = Vec::new();
        let plan=RegExpZeroCountSequenceMatcher::compile_enclosing_optional_known_impossible_sequence_validated_with_work(&source,true,true,|n|{work.push(n);Ok::<(),()>(())}).unwrap().unwrap();
        assert_eq!(work[0], source.len() * 32 + 16);
        assert_eq!(*work.last().unwrap(), plan.capture_count());
        for stop in 0..work.len() {
            let mut seen = Vec::new();
            assert!(
                matches!(RegExpZeroCountSequenceMatcher::compile_enclosing_optional_known_impossible_sequence_validated_with_work(&source,true,true,|n|{seen.push(n);if seen.len()==stop+1{Err(stop)}else{Ok(())}}),Err(n)if n==stop)
            );
            assert_eq!(seen, work[..=stop]);
        }
        assert_eq!(
            plan.find_with_work(&JsString::from("😀"), 1, |n| {
                assert_eq!(n, 1);
                Ok::<(), ()>(())
            })
            .unwrap(),
            Some(0..0)
        );
    }
    fn first_optional_known(source: &str) -> Option<RegExpZeroCountSequenceMatcher> {
        RegExpZeroCountSequenceMatcher::compile_first_optional_known_impossible_validated_with_work(
            &JsString::from(source),
            true,
            true,
            |_| Ok::<(), ()>(()),
        )
        .unwrap()
    }

    #[test]
    fn first_optional_known_choices_keep_first_mask_and_all_later_global_slots() {
        for (source, mask) in [
            (
                r"((?![\p{RGI_Emoji}]*))?|((a)\1)+",
                &[false, false, false][..],
            ),
            (
                r"(((?![\p{RGI_Emoji}]*))?)+([a]{0})+|(b)(c)*",
                &[true, false, true, false, false][..],
            ),
            (
                r"((((?![\p{RGI_Emoji}]*))?([a]{0})+)*)+|(?=(b))c",
                &[true, false, false, false, false][..],
            ),
            (r"((?!(\3)*))?|((a)\1)+", &[false, false, false, false][..]),
        ] {
            assert_eq!(
                first_optional_known(source).unwrap().defined.as_ref(),
                mask,
                "{source}"
            );
        }
    }

    #[test]
    fn first_optional_known_choices_reject_nonroot_choices_possible_first_arms_and_siblings() {
        for source in [
            r"((?!a*))?",
            r"a|((?!a*))?",
            r"(((?!a*))?|a)+",
            r"(((?!a*))?|a)?",
            r"((?!a))?|b",
            r"((?!a*))?a|b",
            r"[a]{0}|((?!a*))?",
            r"(?=((?!a*))?|a)",
        ] {
            assert!(first_optional_known(source).is_none(), "{source}");
        }
    }

    #[test]
    fn first_optional_known_choices_keep_many_unselected_capture_slots_and_large_bounds_flat() {
        let source = format!(r"((?![\p{{RGI_Emoji}}]*))?|{}", "(a)|".repeat(20_000));
        let plan = first_optional_known(&source).unwrap();
        assert_eq!(plan.capture_count(), 20_001);
        assert!(plan.defined.iter().all(|&v| !v));
        let source = format!(r"((?![\p{{RGI_Emoji}}]*)){{0,{}}}|(a)", "9".repeat(20_000));
        assert_eq!(
            first_optional_known(&source).unwrap().defined.as_ref(),
            &[false, false]
        );
    }

    #[test]
    fn first_optional_known_choice_work_interrupts_before_root_projection_proof_and_full_mask() {
        let source = JsString::from(r"(((?![\p{RGI_Emoji}]*))?)+([a]{0})+|(b)(c)*");
        let mut work = Vec::new();
        let plan=RegExpZeroCountSequenceMatcher::compile_first_optional_known_impossible_validated_with_work(&source,true,true,|n|{work.push(n);Ok::<(),()>(())}).unwrap().unwrap();
        assert_eq!(work[0], source.len() * 8 + 16);
        assert_eq!(*work.last().unwrap(), plan.capture_count());
        for stop in 0..work.len() {
            let mut seen = Vec::new();
            assert!(
                matches!(RegExpZeroCountSequenceMatcher::compile_first_optional_known_impossible_validated_with_work(&source,true,true,|n|{seen.push(n);if seen.len()==stop+1{Err(stop)}else{Ok(())}}),Err(n)if n==stop)
            );
            assert_eq!(seen, work[..=stop]);
        }
        assert_eq!(
            plan.find_with_work(&JsString::from("😀"), 1, |n| {
                assert_eq!(n, 1);
                Ok::<(), ()>(())
            })
            .unwrap(),
            Some(0..0)
        );
    }
    fn positive_optional_known(source: &str) -> Option<RegExpZeroCountSequenceMatcher> {
        RegExpZeroCountSequenceMatcher::compile_positive_optional_known_impossible_validated_with_work(
            &JsString::from(source),true,true,|_|Ok::<(),()>(()),
        ).unwrap()
    }

    #[test]
    fn positive_optional_known_assertions_export_each_original_operand_mask_in_both_directions() {
        for operand in [
            r"((?![\p{RGI_Emoji}]*))?",
            r"(((?![\p{RGI_Emoji}]*))?)+([a]{0})+",
            r"((((?![\p{RGI_Emoji}]*))?([a]{0})+)*)+",
            r"(((?![\p{RGI_Emoji}]*))?)+([a]{0})+|(b)(c)*",
        ] {
            let expected = if operand.contains('|') {
                first_optional_known(operand)
            } else {
                optional_known_impossible(operand)
                    .or_else(|| optional_known_sequence(operand))
                    .or_else(|| enclosing_optional_known_sequence(operand))
            }
            .unwrap();
            for head in ["(?=", "(?<="] {
                let source = format!("{head}{operand})");
                assert_eq!(
                    positive_optional_known(&source).unwrap().defined,
                    expected.defined,
                    "{source}"
                );
            }
        }
        assert_eq!(
            positive_optional_known(r"(?<=((?!(\3)*))?|((a)\1)+)")
                .unwrap()
                .defined
                .as_ref(),
            &[false, false, false, false]
        );
    }

    #[test]
    fn positive_optional_known_assertions_reject_unproved_siblings_outer_groups_and_extra_layers() {
        for source in [
            r"((?!a*))?",
            r"(?=((?!a))?)",
            r"(?=((?!a*))?a)",
            r"(?=a|((?!a*))?)",
            r"(?=((?!a*))?)a",
            r"a(?=((?!a*))?)",
            r"((?=((?!a*))?))+",
            r"(?=(?=((?!a*))?))",
            r"(?<=(?<=((?!a*))?))",
        ] {
            assert!(positive_optional_known(source).is_none(), "{source}");
        }
    }

    #[test]
    fn positive_optional_known_assertions_keep_many_slots_and_symbolic_bounds_flat() {
        let source = format!(r"(?<=((?![\p{{RGI_Emoji}}]*))?|{})", "(a)|".repeat(20_000));
        let plan = positive_optional_known(&source).unwrap();
        assert_eq!(plan.capture_count(), 20_001);
        assert!(plan.defined.iter().all(|&v| !v));
        let source = format!(r"(?=(((?![\p{{RGI_Emoji}}]*))?){{{}}})", "9".repeat(20_000));
        assert_eq!(
            positive_optional_known(&source).unwrap().defined.as_ref(),
            &[true, false]
        );
    }

    #[test]
    fn positive_optional_known_assertion_work_interrupts_before_projection_operand_and_actual_input()
     {
        let source = JsString::from(r"(?<=(((?![\p{RGI_Emoji}]*))?)+([a]{0})+|(b)(c)*)");
        let mut work = Vec::new();
        let plan=RegExpZeroCountSequenceMatcher::compile_positive_optional_known_impossible_validated_with_work(&source,true,true,|n|{work.push(n);Ok::<(),()>(())}).unwrap().unwrap();
        assert_eq!(work[0], source.len() * 8 + 16);
        for stop in 0..work.len() {
            let mut seen = Vec::new();
            assert!(
                matches!(RegExpZeroCountSequenceMatcher::compile_positive_optional_known_impossible_validated_with_work(&source,true,true,|n|{seen.push(n);if seen.len()==stop+1{Err(stop)}else{Ok(())}}),Err(n)if n==stop)
            );
            assert_eq!(seen, work[..=stop]);
        }
        assert_eq!(
            plan.find_with_work(&JsString::from("😀"), 1, |n| {
                assert_eq!(n, 1);
                Ok::<(), ()>(())
            })
            .unwrap(),
            Some(0..0)
        );
        assert_eq!(
            plan.find_with_work(&JsString::from(""), usize::MAX, |_| Err(43)),
            Err(43)
        );
    }
}
