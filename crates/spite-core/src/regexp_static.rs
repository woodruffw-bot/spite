//! Flat static Pattern proofs (ECMA-262 22.2.2.2–4 and 22.2.2.7).
//!
//! Every source and flag combination must be validated before this proof runs.
//! Capture heads are normalized, but original references are never rebound.

use crate::{
    JsString, RegExpImpossibleMatcher, RegExpZeroCountSequenceMatcher,
    regexp_zero_count_spine::{opener, quantifier},
    regexp_zero_count_tree::surrogate_atom_end,
};
use std::ops::Range;

/// A validated Pattern proved independently empty or impossible.
///
/// Flat frames compose ordinary/modifier groups, choices, sequences, both
/// assertion directions and symbolic quantifiers. Every successful choice arm
/// must independently stay empty before its mask can be exported. A merely
/// nullable operand can consume and capture data; it only proves that an outside
/// negative assertion fails. Unsupported atom boundaries reject the proof.
#[derive(Clone, Debug)]
pub enum RegExpStaticMatcher {
    /// Every successful attempt stays empty with the exact original capture mask.
    Empty(RegExpZeroCountSequenceMatcher),
    /// Every attempt fails; only the full lexical capture count is retained.
    Impossible(RegExpImpossibleMatcher),
}

impl RegExpStaticMatcher {
    /// Proves a complete validated Pattern without constructing matching data.
    ///
    /// Full original grammar/flag validation and normalized capture heads are
    /// required. No projected operand is revalidated or rebound. Safe literal,
    /// class/property and surrogate atom boundaries use the existing tree
    /// scanner. Unknown escapes and conditional consuming forms remain unproved.
    /// Source work is charged before scanning and frame/mask allocation. Flat
    /// frames and prefix capture clears keep construction source-sized without
    /// recursive calls or count expansion (22.2.2.2–4, 22.2.2.7).
    pub fn compile_validated_with_work<E>(
        source: &JsString,
        unicode: bool,
        sets: bool,
        charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, E> {
        Ok(
            compile_facts_with_work::<false, _>(source, unicode, sets, charge)?.map(
                |(fact, defined)| match fact {
                    Fact::Empty => Self::Empty(RegExpZeroCountSequenceMatcher::from_static_mask(
                        unicode, defined,
                    )),
                    Fact::Impossible => Self::Impossible(
                        RegExpImpossibleMatcher::from_static_capture_count(defined.len()),
                    ),
                    Fact::Unknown(_) => unreachable!("unproved roots return None"),
                },
            ),
        )
    }

    /// Extends static proofs with capture-free guaranteed-successful positive assertions.
    ///
    /// Full original validation and normalized capture heads are required.
    /// A positive assertion always consumes zero. If its complete operand has
    /// a successful path at every position and contains no lexical captures,
    /// no matching data can escape, even when the selected operand consumes
    /// (22.2.2.4). Both directions therefore compose as independently empty.
    /// Capturing operands retain the earlier conservative facts. Source work,
    /// flat frames, symbolic bounds and capture-clear invariants are unchanged.
    pub fn compile_capture_free_positive_validated_with_work<E>(
        source: &JsString,
        unicode: bool,
        sets: bool,
        charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, E> {
        Ok(
            compile_facts_with_work::<true, _>(source, unicode, sets, charge)?.map(
                |(fact, defined)| match fact {
                    Fact::Empty => Self::Empty(RegExpZeroCountSequenceMatcher::from_static_mask(
                        unicode, defined,
                    )),
                    Fact::Impossible => Self::Impossible(
                        RegExpImpossibleMatcher::from_static_capture_count(defined.len()),
                    ),
                    Fact::Unknown(_) => unreachable!("unproved roots return None"),
                },
            ),
        )
    }

    /// Full original lexical capture count, including skipped arms (22.2.7.2).
    pub fn capture_count(&self) -> usize {
        match self {
            Self::Empty(plan) => plan.capture_count(),
            Self::Impossible(plan) => plan.capture_count(),
        }
    }

    /// Empty span for a participating group, otherwise an undefined slot.
    pub fn capture_range(&self, index: usize, matched: &Range<usize>) -> Option<Range<usize>> {
        match self {
            Self::Empty(plan) => plan.capture_range(index, matched),
            Self::Impossible(_) => None,
        }
    }

    /// Executes the proof with constant work charged before any input access.
    ///
    /// Empty matches use the existing Unicode leading-boundary policy; an
    /// impossible plan never inspects the input or start index (22.2.7.2).
    pub fn find_with_work<E>(
        &self,
        input: &JsString,
        start: usize,
        charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Range<usize>>, E> {
        match self {
            Self::Empty(plan) => plan.find_with_work(input, start, charge),
            Self::Impossible(plan) => plan.find_with_work(input, start, charge),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Fact {
    Empty,
    Impossible,
    // A successful path exists at every position when true; its actual
    // consumption and capture values remain unproved.
    Unknown(bool),
}
type CaptureProof = (Fact, Box<[bool]>);

impl Fact {
    fn nullable(self) -> bool {
        matches!(self, Self::Empty | Self::Unknown(true))
    }
    fn sequence(self, other: Self) -> Self {
        match (self, other) {
            (Self::Impossible, _) | (_, Self::Impossible) => Self::Impossible,
            (Self::Empty, Self::Empty) => Self::Empty,
            _ => Self::Unknown(self.nullable() && other.nullable()),
        }
    }
    fn alternative(self, other: Self) -> Self {
        match (self, other) {
            (Self::Impossible, v) | (v, Self::Impossible) => v,
            (Self::Empty, Self::Empty) => Self::Empty,
            _ => Self::Unknown(self.nullable() || other.nullable()),
        }
    }
    fn repeat(self, minimum_zero: bool, maximum_zero: bool) -> Self {
        // Nullability alone must never fabricate an empty match or mask.
        if maximum_zero || minimum_zero && matches!(self, Self::Empty | Self::Impossible) {
            Self::Empty
        } else {
            match self {
                Self::Unknown(nullable) => Self::Unknown(nullable || minimum_zero),
                v => v,
            }
        }
    }
}
#[derive(Clone, Copy)]
enum Kind {
    Ordinary,
    Positive,
    Negative,
}
struct Frame {
    kind: Kind,
    start: usize,
    branch_start: usize,
    sequence: Fact,
    choices: Fact,
    has_possible_arm: bool,
}
impl Frame {
    fn new(kind: Kind, start: usize, branch_start: usize) -> Self {
        Self {
            kind,
            start,
            branch_start,
            sequence: Fact::Empty,
            choices: Fact::Impossible,
            has_possible_arm: false,
        }
    }
    fn finish_arm(&mut self, clears: &mut [usize]) {
        // A proved-empty choice keeps its first possible arm. Failed and later
        // arms roll their capture writes back (22.2.2.2).
        if self.sequence == Fact::Impossible || self.has_possible_arm {
            clear(clears, self.branch_start);
        }
        self.has_possible_arm |= self.sequence != Fact::Impossible;
        self.choices = self.choices.alternative(self.sequence);
        self.sequence = Fact::Empty;
        self.branch_start = clears.len();
    }
}
// Prefix maxima apply overlapping/nested capture clears once after parsing.
fn clear(ends: &mut [usize], start: usize) {
    let end = ends.len();
    if let Some(marker) = ends.get_mut(start) {
        *marker = (*marker).max(end);
    }
}

fn compile_facts_with_work<const CAPTURE_FREE_POSITIVE: bool, E>(
    source: &JsString,
    unicode: bool,
    sets: bool,
    mut charge: impl FnMut(usize) -> Result<(), E>,
) -> Result<Option<CaptureProof>, E> {
    charge(source.len().saturating_mul(96).saturating_add(16))?;
    if sets && !unicode {
        return Ok(None);
    }
    let units = source.code_units();
    let mut frames = vec![Frame::new(Kind::Ordinary, 0, 0)];
    let mut clears = Vec::new();
    let mut cursor = 0;
    while let Some(&unit) = units.get(cursor) {
        match unit {
            0x28 => {
                let (width, capture, kind) = if let Some((width, capture)) = opener(units, cursor) {
                    (width, capture, Kind::Ordinary)
                } else if units[cursor..].starts_with(&[0x28, 0x3f, 0x3d]) {
                    (3, false, Kind::Positive)
                } else if units[cursor..].starts_with(&[0x28, 0x3f, 0x21]) {
                    (3, false, Kind::Negative)
                } else if units[cursor..].starts_with(&[0x28, 0x3f, 0x3c, 0x3d]) {
                    (4, false, Kind::Positive)
                } else if units[cursor..].starts_with(&[0x28, 0x3f, 0x3c, 0x21]) {
                    (4, false, Kind::Negative)
                } else {
                    return Ok(None);
                };
                let start = clears.len();
                if capture {
                    clears.push(0)
                }
                frames.push(Frame::new(kind, start, clears.len()));
                cursor += width;
            }
            0x29 => {
                if frames.len() == 1 {
                    return Ok(None);
                }
                let mut frame = frames.pop().expect("nonroot frame");
                frame.finish_arm(&mut clears);
                let operand = frame.choices;
                let fact = match frame.kind {
                    Kind::Ordinary => operand,
                    Kind::Positive
                        if CAPTURE_FREE_POSITIVE
                            && operand.nullable()
                            && clears.len() == frame.start =>
                    {
                        Fact::Empty
                    }
                    Kind::Positive => operand,
                    Kind::Negative => match operand {
                        Fact::Impossible => {
                            clear(&mut clears, frame.start);
                            Fact::Empty
                        }
                        v if v.nullable() => Fact::Impossible,
                        _ => Fact::Unknown(false),
                    },
                };
                let Some((end, minimum_zero, maximum_zero)) = quantifier(units, cursor + 1) else {
                    return Ok(None);
                };
                let repeated = fact.repeat(minimum_zero, maximum_zero);
                if repeated == Fact::Empty && (minimum_zero || maximum_zero) {
                    clear(&mut clears, frame.start)
                }
                let parent = frames.last_mut().expect("root frame");
                parent.sequence = parent.sequence.sequence(repeated);
                cursor = end;
            }
            0x7c => {
                frames
                    .last_mut()
                    .expect("root frame")
                    .finish_arm(&mut clears);
                cursor += 1;
            }
            _ => {
                let Some(end) = surrogate_atom_end(units, cursor, unicode, sets) else {
                    return Ok(None);
                };
                let Some((end, minimum_zero, maximum_zero)) = quantifier(units, end) else {
                    return Ok(None);
                };
                let fact = if maximum_zero {
                    Fact::Empty
                } else {
                    Fact::Unknown(minimum_zero)
                };
                let frame = frames.last_mut().expect("root frame");
                frame.sequence = frame.sequence.sequence(fact);
                cursor = end;
            }
        }
    }
    if frames.len() != 1 {
        return Ok(None);
    }
    let mut root = frames.pop().expect("root frame");
    root.finish_arm(&mut clears);
    if matches!(root.choices, Fact::Unknown(_)) {
        return Ok(None);
    }
    let mut until = 0;
    let mask = clears
        .into_iter()
        .enumerate()
        .map(|(index, end)| {
            until = until.max(end);
            index >= until
        })
        .collect();
    Ok(Some((root.choices, mask)))
}

#[cfg(test)]
mod tests {
    use super::*;
    fn plan(source: &str) -> Option<(Fact, Box<[bool]>)> {
        compile_facts_with_work::<false, _>(&JsString::from(source), true, true, |_| {
            Ok::<(), ()>(())
        })
        .unwrap()
    }
    #[test]
    fn compositional_negative_and_optional_facts_cross_multiple_layers() {
        for (source, fact, mask) in [
            (r"((?![\p{RGI_Emoji}]*))?", Fact::Empty, &[false][..]),
            (
                r"((?!a*)|(?!(b){0}))+",
                Fact::Impossible,
                &[false, false][..],
            ),
            (r"((?!a*)|((b){0}))+", Fact::Empty, &[true, true, false][..]),
            (
                r"(?!(?=(((?!a*))?([b]{0})+)))",
                Fact::Impossible,
                &[true, false, true][..],
            ),
            (r"(?<!(?!(?!(a*))))", Fact::Impossible, &[false][..]),
        ] {
            let got = plan(source).unwrap_or_else(|| panic!("{source}"));
            assert_eq!(got.0, fact, "{source}");
            if fact == Fact::Empty {
                assert_eq!(got.1.as_ref(), mask, "{source}");
            }
        }
    }
    #[test]
    fn choices_skip_impossible_arms_and_reserve_all_lexical_slots() {
        for (source, mask) in [
            (r"((?!a*))|((b){0})", &[false, true, false][..]),
            (
                r"((a){0})|((?!b*))|((c){0})",
                &[true, false, false, false, false][..],
            ),
            (r"(((?!a*))|((b){0})){2,3}", &[true, false, true, false][..]),
            (r"(((?!a*))|((b){0}))*", &[false, false, false, false][..]),
        ] {
            let (fact, got) = plan(source).unwrap_or_else(|| panic!("{source}"));
            assert_eq!(fact, Fact::Empty);
            assert_eq!(got.as_ref(), mask, "{source}");
        }
    }
    #[test]
    fn nullable_paths_do_not_fabricate_capture_values_or_empty_repeat_plans() {
        for source in [
            r"(a)*",
            r"(|a)+",
            r"(?=(a)*)",
            r"((?!a*)|a)?",
            r"(?=a*)",
            r"a{0}|a",
            r"((a{0})|b)+",
            r"(a)\1{0}",
        ] {
            assert!(plan(source).is_none(), "{source}");
        }
        assert_eq!(plan(r"(?!(|a)+)").unwrap().0, Fact::Impossible);
    }
    #[test]
    fn work_is_charged_before_scanning_and_allocations() {
        let source = JsString::from(r"((?!a*))?");
        let mut seen = Vec::new();
        assert_eq!(
            compile_facts_with_work::<false, _>(&source, true, true, |n| {
                seen.push(n);
                Err(19)
            }),
            Err(19)
        );
        assert_eq!(seen, [source.len() * 96 + 16]);
    }
    #[test]
    fn deep_ordinary_and_assertion_layers_use_flat_capture_storage() {
        let source = format!(
            "{}(?!a*)|((b){{0}}){}",
            "(".repeat(100_000),
            ")+".repeat(100_000)
        );
        let (fact, mask) = plan(&source).unwrap();
        assert_eq!(fact, Fact::Empty);
        assert_eq!(mask.len(), 100_002);
        assert!(mask[..100_001].iter().all(|&v| v));
        assert!(!mask[100_001]);
        for depth in [100_000, 100_001] {
            let source = format!("{}a*{}", "(?!".repeat(depth), ")".repeat(depth));
            assert_eq!(
                plan(&source).unwrap().0,
                if depth % 2 == 0 {
                    Fact::Empty
                } else {
                    Fact::Impossible
                }
            );
        }
    }
    #[test]
    fn symbolic_counts_and_unsafe_atom_boundaries_stay_conservative() {
        let source = format!("(((?!a*))?){{{}}}", "9".repeat(20_000));
        let (fact, mask) = plan(&source).unwrap();
        assert_eq!(fact, Fact::Empty);
        assert_eq!(mask.as_ref(), &[true, false]);
        for source in [r"(a)\1{0}", r"(?!(a)\1*)", r"(?=a{0})$", r"((a)\2){0}"] {
            assert!(plan(source).is_none(), "{source}");
        }
    }

    fn public_plan(source: &str, unicode: bool) -> RegExpStaticMatcher {
        RegExpStaticMatcher::compile_validated_with_work(
            &JsString::from(source),
            unicode,
            unicode,
            |_| Ok::<(), ()>(()),
        )
        .unwrap()
        .unwrap()
    }
    #[test]
    fn public_plans_keep_capture_masks_ownership_and_constant_actual_work() {
        let plan = public_plan(r"((?!a*)|((b){0}))+", true);
        assert!(matches!(plan, RegExpStaticMatcher::Empty(_)));
        assert_eq!(plan.capture_count(), 3);
        let copy = plan.clone();
        drop(plan);
        let range = 4..4;
        assert_eq!(copy.capture_range(0, &range), Some(range.clone()));
        assert_eq!(copy.capture_range(1, &range), Some(range.clone()));
        assert_eq!(copy.capture_range(2, &range), None);
        assert_eq!(copy.capture_range(3, &range), None);
        let mut work = 0;
        assert_eq!(
            copy.find_with_work(&JsString::from("A😀!"), 2, |n| {
                work += n;
                Ok::<(), ()>(())
            })
            .unwrap(),
            Some(1..1)
        );
        assert_eq!(work, 1);
        assert_eq!(
            copy.find_with_work(&JsString::from(""), usize::MAX, |_| Err(23)),
            Err(23)
        );
    }
    #[test]
    fn public_impossible_plans_retain_only_inventory_and_never_read_input() {
        let source = format!("{}(?!a*){}", "(".repeat(100_000), ")+".repeat(100_000));
        let plan = public_plan(&source, true);
        assert!(matches!(plan, RegExpStaticMatcher::Impossible(_)));
        assert_eq!(plan.capture_count(), 100_000);
        let copy = plan.clone();
        drop(plan);
        drop(source);
        let mut work = 0;
        assert_eq!(
            copy.find_with_work(&JsString::from(""), usize::MAX, |n| {
                work += n;
                Ok::<(), ()>(())
            })
            .unwrap(),
            None
        );
        assert_eq!(work, 1);
        assert_eq!(copy.capture_range(0, &(0..0)), None);
        assert_eq!(
            copy.find_with_work(&JsString::from("😀"), 1, |_| Err(29)),
            Err(29)
        );
    }
    #[test]
    fn public_empty_boundary_modes_preserve_surrogate_spelling_and_undefined_slots() {
        for unicode in [false, true] {
            let plan = public_plan(r"(\uD83D\uDE00){0}", unicode);
            assert_eq!(plan.capture_count(), 1);
            assert_eq!(plan.capture_range(0, &(1..1)), None);
            assert_eq!(
                plan.find_with_work(&JsString::from("😀"), 1, |_| Ok::<(), ()>(()))
                    .unwrap(),
                Some(if unicode { 0..0 } else { 1..1 })
            );
        }
        assert!(
            RegExpStaticMatcher::compile_validated_with_work(
                &JsString::from("a{0}"),
                false,
                true,
                |_| Ok::<(), ()>(())
            )
            .unwrap()
            .is_none()
        );
    }
    #[test]
    fn public_compilation_interrupts_before_frames_or_capture_storage() {
        let source = JsString::from(r"((?!a*)|((b){0}))+");
        let mut seen = Vec::new();
        assert!(matches!(
            RegExpStaticMatcher::compile_validated_with_work(&source, true, true, |n| {
                seen.push(n);
                Err(31)
            }),
            Err(31)
        ));
        assert_eq!(seen, [source.len() * 96 + 16]);
        for source in [r"(|a)+", r"(?=(a)*)", r"((?!a*)|a)?"] {
            assert!(
                RegExpStaticMatcher::compile_validated_with_work(
                    &JsString::from(source),
                    true,
                    true,
                    |_| Ok::<(), ()>(())
                )
                .unwrap()
                .is_none(),
                "{source}"
            );
        }
    }
    fn capture_free_plan(source: &str) -> Option<RegExpStaticMatcher> {
        RegExpStaticMatcher::compile_capture_free_positive_validated_with_work(
            &JsString::from(source),
            true,
            true,
            |_| Ok::<(), ()>(()),
        )
        .unwrap()
    }
    #[test]
    fn capture_free_successful_positive_assertions_export_only_proved_outside_empty_masks() {
        for (source, mask) in [
            (r"(?=a*)", &[][..]),
            (r"(?<=a*b*)", &[][..]),
            (r"((?=[\p{RGI_Emoji}]*))+", &[true][..]),
            (r"((?<=[\q{a|ab}]*))?", &[false][..]),
            (r"(?im-s:((?=a*b*))+(?<=[\p{RGI_Emoji}]*))", &[true][..]),
            (r"((?=(?<=a*)))+", &[true][..]),
        ] {
            assert!(
                RegExpStaticMatcher::compile_validated_with_work(
                    &JsString::from(source),
                    true,
                    true,
                    |_| Ok::<(), ()>(())
                )
                .unwrap()
                .is_none(),
                "earlier domain changed: {source}"
            );
            let plan = capture_free_plan(source).unwrap_or_else(|| panic!("rejected {source}"));
            assert_eq!(plan.capture_count(), mask.len(), "{source}");
            for (index, &defined) in mask.iter().enumerate() {
                assert_eq!(
                    plan.capture_range(index, &(2..2)),
                    defined.then_some(2..2),
                    "{source}"
                );
            }
        }
    }
    #[test]
    fn capture_free_positive_proofs_keep_capturing_consuming_and_continuation_sensitive_forms_unproved()
     {
        for source in [
            r"(?=(a)*)",
            r"(?<=(a)*)",
            r"(?=(?i:(a)*))",
            r"((?=(a)*))+",
            r"(|a)+",
            r"a(?=b*)",
            r"(?=a*)a",
            r"(?=a*)|a",
            r"((?=a*)|a)+",
            r"(a*)",
        ] {
            assert!(capture_free_plan(source).is_none(), "{source}");
        }
    }
    #[test]
    fn capture_free_positive_layers_and_symbolic_minima_keep_flat_outside_capture_masks() {
        let source = format!(
            "{}{}a*{}{}",
            "(".repeat(100_000),
            "(?=".repeat(100_000),
            ")".repeat(100_000),
            ")+".repeat(100_000)
        );
        let plan = capture_free_plan(&source).unwrap();
        assert_eq!(plan.capture_count(), 100_000);
        for index in 0..plan.capture_count() {
            assert_eq!(plan.capture_range(index, &(0..0)), Some(0..0));
        }
        let source = format!("((?=a*)){{{}}}", "9".repeat(20_000));
        let plan = capture_free_plan(&source).unwrap();
        assert_eq!(plan.capture_count(), 1);
        assert_eq!(plan.capture_range(0, &(0..0)), Some(0..0));
    }
    #[test]
    fn capture_free_positive_work_interrupts_before_frames_and_actual_unicode_input_access() {
        let source = JsString::from(r"((?=[\p{RGI_Emoji}]*))+");
        let mut seen = Vec::new();
        assert!(matches!(
            RegExpStaticMatcher::compile_capture_free_positive_validated_with_work(
                &source,
                true,
                true,
                |n| {
                    seen.push(n);
                    Err(41)
                }
            ),
            Err(41)
        ));
        assert_eq!(seen, [source.len() * 96 + 16]);
        let plan = capture_free_plan(r"((?=[\p{RGI_Emoji}]*))+").unwrap();
        let mut work = 0;
        assert_eq!(
            plan.find_with_work(&JsString::from("😀"), 1, |n| {
                work += n;
                Ok::<(), ()>(())
            })
            .unwrap(),
            Some(0..0)
        );
        assert_eq!(work, 1);
        assert_eq!(
            plan.find_with_work(&JsString::from(""), usize::MAX, |_| Err(43)),
            Err(43)
        );
    }
}
