//! Complete Unicode positive lookahead over one repeated atom (22.2.2.3–4).

use crate::{
    JsString, RegExpUnicodeRepeatedAtomMatcher, regexp_zero_count_group::enclosing_group_body,
    regexp_zero_count_sequence::group_end,
};
use std::ops::Range;

/// A zero-width lookahead match and its independently retained operand extent.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RegExpUnicodePositiveAtomMatch {
    /// Whole assertion match at the original UTF-16 candidate boundary.
    pub range: Range<usize>,
    /// Selected operand range, which may extend beyond the empty whole match.
    pub operand: Range<usize>,
}

/// A complete positive lookahead over one quantified Unicode character atom.
///
/// The original u/v Pattern and flags must be validated and capture heads
/// normalized first. Complete unquantified ordinary/noncapturing wrappers may
/// surround the assertion and its operand. Outside captures stay empty; operand
/// wrappers capture the selected run, including a defined empty zero-iteration
/// run (22.2.2.3.1, 22.2.2.4). Internal captures, modifiers, alternatives,
/// continuations and lookbehind require other compilers.
#[derive(Clone, Debug)]
pub struct RegExpUnicodePositiveAtomMatcher {
    operand: RegExpUnicodeRepeatedAtomMatcher,
    outside_captures: usize,
    captures: usize,
}

impl RegExpUnicodePositiveAtomMatcher {
    /// Compiles a complete validated Unicode lookahead without rebinding references.
    ///
    /// Original full validation and normalized capture heads are required.
    /// Operand projections retain their original binding context; unsupported
    /// reference syntax is rejected by the existing atom compiler. Source work
    /// precedes scans/copies; wrapper depth and symbolic bounds never recurse or
    /// expand. Existing atom compilation retains its work and flag semantics.
    pub fn compile_validated_with_work<E>(
        source: &JsString,
        unicode_sets: bool,
        ignore_case: bool,
        multiline: bool,
        dot_all: bool,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, E> {
        charge(source.len().saturating_mul(16).saturating_add(16))?;
        let units = source.code_units();
        let (assertion, outside_captures) =
            if let Some(scope) = enclosing_group_body(units, unicode_sets) {
                (&units[scope.body], scope.captures)
            } else {
                (units, 0)
            };
        if !assertion.starts_with(&[0x28, 0x3f, 0x3d])
            || group_end(assertion, 0, unicode_sets) != Some(assertion.len())
        {
            return Ok(None);
        }
        let body = &assertion[3..assertion.len() - 1];
        let (body, captures) = if let Some(scope) = enclosing_group_body(body, unicode_sets) {
            (&body[scope.body], scope.captures)
        } else {
            (body, 0)
        };
        let body = JsString::from_code_units(body.to_vec());
        let Some(operand) = RegExpUnicodeRepeatedAtomMatcher::compile_with_line_anchors_and_work(
            &body,
            unicode_sets,
            ignore_case,
            multiline,
            dot_all,
            &mut charge,
        )?
        else {
            return Ok(None);
        };
        if operand.capture_count() != 0 {
            return Ok(None);
        }
        Ok(Some(Self {
            operand,
            outside_captures,
            captures: outside_captures + captures,
        }))
    }

    /// Full original lexical capture count, including outside assertion wrappers.
    pub fn capture_count(&self) -> usize {
        self.captures
    }

    /// Searches using the operand's original context while retaining its capture extent.
    ///
    /// The underlying Unicode repetition keeps greedy/lazy counts, line anchors,
    /// sticky candidates and the approved leading-surrogate boundary policy.
    /// Its callback charges actual work before input access; no capture-sized
    /// allocation or rescanning is needed (22.2.2.4, 22.2.7.2).
    pub fn find_with_work<E>(
        &self,
        input: &JsString,
        start: usize,
        sticky: bool,
        charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<RegExpUnicodePositiveAtomMatch>, E> {
        Ok(self
            .operand
            .find_with_work(input, start, sticky, charge)?
            .map(|operand| RegExpUnicodePositiveAtomMatch {
                range: operand.start..operand.start,
                operand,
            }))
    }

    /// Exact empty outside span or full operand capture span, in constant work.
    pub fn capture_range(
        &self,
        index: usize,
        matched: &RegExpUnicodePositiveAtomMatch,
    ) -> Option<Range<usize>> {
        if index >= self.captures {
            None
        } else if index < self.outside_captures {
            Some(matched.range.clone())
        } else {
            Some(matched.operand.clone())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn plan(source: &str) -> RegExpUnicodePositiveAtomMatcher {
        RegExpUnicodePositiveAtomMatcher::compile_validated_with_work(
            &JsString::from(source),
            false,
            false,
            false,
            false,
            |_| Ok::<(), ()>(()),
        )
        .unwrap()
        .unwrap()
    }
    #[test]
    fn unicode_lookahead_retains_consumed_capture_ranges_beyond_empty_whole_match() {
        let p = plan("((?=((a*))))");
        let m = p
            .find_with_work(&JsString::from("aaa"), 0, false, |_| Ok::<(), ()>(()))
            .unwrap()
            .unwrap();
        assert_eq!(m.range, 0..0);
        assert_eq!(m.operand, 0..3);
        assert_eq!(p.capture_count(), 3);
        assert_eq!(p.capture_range(0, &m), Some(0..0));
        assert_eq!(p.capture_range(1, &m), Some(0..3));
        assert_eq!(p.capture_range(2, &m), Some(0..3));
        assert_eq!(p.capture_range(3, &m), None);
        let p = plan("(?=(a*?))");
        let m = p
            .find_with_work(&JsString::from("aaa"), 0, false, |_| Ok::<(), ()>(()))
            .unwrap()
            .unwrap();
        assert_eq!(m.operand, 0..0);
        assert_eq!(p.capture_range(0, &m), Some(0..0));
    }
    #[test]
    fn unicode_lookahead_search_sticky_flags_and_surrogate_candidates_keep_original_positions() {
        let p = plan("(?=(😀+))");
        let input = JsString::from("x😀😀y");
        let m = p
            .find_with_work(&input, 0, false, |_| Ok::<(), ()>(()))
            .unwrap()
            .unwrap();
        assert_eq!(m.range, 1..1);
        assert_eq!(m.operand, 1..5);
        assert!(
            p.find_with_work(&input, 0, true, |_| Ok::<(), ()>(()))
                .unwrap()
                .is_none()
        );
        let m = p
            .find_with_work(&input, 2, true, |_| Ok::<(), ()>(()))
            .unwrap()
            .unwrap();
        assert_eq!(m.range, 1..1);
        assert_eq!(m.operand, 1..5);
    }

    #[test]
    fn unicode_lookahead_ascii_property_uses_the_specification_folded_set() {
        // CompileToCharSet and CharacterSetMatcher fold both the positive
        // property and the input. See the documented Node iv discrepancy.
        for sets in [false, true] {
            for ignore_case in [false, true] {
                let p = RegExpUnicodePositiveAtomMatcher::compile_validated_with_work(
                    &JsString::from(r"(?=(\p{ASCII}+))"),
                    sets,
                    ignore_case,
                    false,
                    false,
                    |_| Ok::<(), ()>(()),
                )
                .unwrap()
                .unwrap();
                let input = JsString::from("ſKa");
                let m = p
                    .find_with_work(&input, 0, false, |_| Ok::<(), ()>(()))
                    .unwrap()
                    .unwrap();
                let start = if ignore_case { 0 } else { 2 };
                assert_eq!(m.range, start..start);
                assert_eq!(m.operand, start..3);
                assert_eq!(p.capture_range(0, &m), Some(start..3));
            }
        }
    }

    #[test]
    fn unicode_lookahead_rejects_continuations_modifiers_quantified_wrappers_and_internal_captures()
    {
        for source in [
            "(?<=(a*))",
            "(?!(a*))",
            "(?=(a*))a",
            "a(?=(a*))",
            "(?=(a*))|b",
            "((?=(a*)))*",
            "(?i:(?=(a*)))",
            "(?=(a)* )",
            "(?=((a)*))",
            "(?=(a*b*))",
        ] {
            assert!(
                RegExpUnicodePositiveAtomMatcher::compile_validated_with_work(
                    &JsString::from(source),
                    false,
                    false,
                    false,
                    false,
                    |_| Ok::<(), ()>(())
                )
                .unwrap()
                .is_none(),
                "{source}"
            );
        }
    }
    #[test]
    fn unicode_lookahead_deep_wrappers_symbolic_bounds_and_work_interruptions_are_flat() {
        let source = format!(
            "{}(?={}a*{}){}",
            "(".repeat(100_000),
            "(".repeat(100_000),
            ")".repeat(100_000),
            ")".repeat(100_000)
        );
        let p = plan(&source);
        assert_eq!(p.capture_count(), 200_000);
        let source = format!("(?=(a{{{}}}))", "9".repeat(20_000));
        let p = plan(&source);
        assert!(
            p.find_with_work(&JsString::from("aaaa"), 0, false, |_| Ok::<(), ()>(()))
                .unwrap()
                .is_none()
        );
        let source = JsString::from("((?=(a*)))");
        let mut work = Vec::new();
        RegExpUnicodePositiveAtomMatcher::compile_validated_with_work(
            &source,
            false,
            false,
            false,
            false,
            |n| {
                work.push(n);
                Ok::<(), ()>(())
            },
        )
        .unwrap()
        .unwrap();
        assert_eq!(work[0], source.len() * 16 + 16);
        for stop in 0..work.len() {
            let mut seen = Vec::new();
            assert!(
                matches!(RegExpUnicodePositiveAtomMatcher::compile_validated_with_work(&source,false,false,false,false,|n|{seen.push(n);if seen.len()==stop+1{Err(stop)}else{Ok(())}}),Err(n)if n==stop)
            );
            assert_eq!(seen, work[..=stop]);
        }
        assert_eq!(
            plan("(?=(a*))").find_with_work(&JsString::from(""), usize::MAX, false, |_| Err(59)),
            Err(59)
        );
    }
}
