//! Concatenated erased terms (ECMA-262 22.2.2.2, 22.2.2.3.1).

use crate::{
    JsString,
    regexp_literal::unicode_start,
    regexp_zero_count_spine::{RegExpZeroCountSpineMatcher, atom_end, quantifier},
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

fn group_end(units: &[u16], start: usize, sets: bool) -> Option<usize> {
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
}
