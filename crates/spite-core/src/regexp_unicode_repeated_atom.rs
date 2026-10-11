//! Unicode atom quantifiers and proved line assertions (22.2.2.3, 22.2.2.4).
use crate::{
    JsString, RegExpUnicodeCharacterMatcher, RegExpUnicodeClassMatcher,
    regexp_literal::unicode_start, regexp_unicode_character::unicode_input_character,
};
use std::ops::Range;

/// A selected Unicode repetition and the last captured atom iteration.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RegExpUnicodeRepeatedAtomMatch {
    /// Original UTF-16 range of the complete repetition.
    pub range: Range<usize>,
    /// Last selected iteration, or None without inner captures or iterations.
    pub atom_capture: Option<Range<usize>>,
}

/// A complete single consuming Unicode atom with one greedy or lazy quantifier.
///
/// The caller must validate the whole u/v Pattern. Dedicated entry points admit
/// a leading ^ and trailing $, including multiline contexts. Capturing groups in the repeated
/// atom are admitted only by the dedicated complete-wrapper entry point; other
/// inner assertions, alternatives and concatenations are excluded.
/// Complete enclosing groups may be proved separately by the caller.
/// Construction and search add no native recursion or input-sized storage;
/// work quotas remain opt-in.
#[derive(Clone, Debug)]
pub struct RegExpUnicodeRepeatedAtomMatcher {
    atom: Atom,
    minimum: Option<usize>,
    maximum: Option<usize>,
    greedy: bool,
    start_anchor: bool,
    multiline: bool,
    end_anchor: bool,
    atom_captures: usize,
}

#[cfg(test)]
mod start_assertion_tests {
    use super::*;
    use std::fmt::Write;

    fn plan(source: &str, v: bool, i: bool, m: bool, s: bool) -> RegExpUnicodeRepeatedAtomMatcher {
        RegExpUnicodeRepeatedAtomMatcher::compile_with_start_anchor_and_work(
            &JsString::from(source),
            v,
            i,
            m,
            s,
            |_| Ok::<(), ()>(()),
        )
        .unwrap()
        .unwrap()
    }

    const CASES: [(&str, u8, usize, Option<usize>, bool); 10] = [
        ("^a+", 0, 1, None, true),
        ("^a*?", 0, 0, None, false),
        ("^a??", 0, 0, Some(1), false),
        ("^a{2,3}", 0, 2, Some(3), true),
        ("^.+", 1, 1, None, true),
        (r"^\D{1,2}", 2, 1, Some(2), true),
        (r"^\W+", 3, 1, None, true),
        (r"^\p{Join_C}+", 4, 1, None, true),
        (r"^\p{NChar}{1,2}", 5, 1, Some(2), true),
        ("^[aſK]+", 6, 1, None, true),
    ];

    #[test]
    fn unicode_repeated_atom_start_ranges_snapshot() {
        let mut rows = String::new();
        for (source, ..) in CASES {
            let pattern = JsString::from(source);
            for v in [false, true] {
                for i in [false, true] {
                    for m in [false, true] {
                        for s in [false, true] {
                            let matcher = plan(source, v, i, m, s);
                            for input in [
                                JsString::from("B\raaa\nAAſK\u{2028}"),
                                JsString::from("\r\n\u{200c}\u{200d}\u{ffff}\u{10ffff}"),
                                JsString::from("😀\naaa"),
                                JsString::from_code_units(vec![0x61, 0xd800, 0x0a, 0x61, 0xdc00]),
                                JsString::from(""),
                            ] {
                                for start in [0, 1, 2, input.len(), input.len() + 1] {
                                    for sticky in [false, true] {
                                        writeln!(rows,"{pattern:?} v={v} i={i} m={m} s={s} input={input:?} start={start} sticky={sticky} {:?}", matcher.find_with_work(&input,start,sticky,|_|Ok::<(),()>(())).unwrap()).unwrap();
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        insta::assert_snapshot!(rows);
    }

    #[test]
    fn unicode_repeated_atom_start_matches_independent_candidate_oracle() {
        let alphabet: [u16; 10] = [
            0x41, 0x61, 0x0a, 0x0d, 0x2028, 0x17f, 0x200c, 0xffff, 0xd83d, 0xde00,
        ];
        let lt = |c| matches!(c, 0x0a | 0x0d | 0x2028 | 0x2029);
        for i in [false, true] {
            for m in [false, true] {
                for s in [false, true] {
                    let fold = |c| {
                        if i {
                            match c {
                                0x41 => 0x61,
                                0x17f => 0x73,
                                0x212a => 0x6b,
                                _ => c,
                            }
                        } else {
                            c
                        }
                    };
                    let member = |kind, c| match kind {
                        0 => fold(c) == 0x61,
                        1 => s || !lt(c),
                        2 => !(0x30..=0x39).contains(&c),
                        3 => !char::from_u32(fold(c))
                            .is_some_and(|c| c.is_ascii_alphanumeric() || c == '_'),
                        4 => (0x200c..=0x200d).contains(&c),
                        5 => matches!(c, 0xfdd0..=0xfdef) || (c & 0xffff) >= 0xfffe,
                        6 => [0x61, fold(0x17f), fold(0x212a)].contains(&fold(c)),
                        _ => unreachable!(),
                    };
                    let plans: Vec<_> = CASES.iter().map(|c| plan(c.0, false, i, m, s)).collect();
                    for len in 0..=4 {
                        for ordinal in 0..alphabet.len().pow(len) {
                            let units: Vec<_> = (0..len)
                                .map(|j| alphabet[ordinal / alphabet.len().pow(j) % alphabet.len()])
                                .collect();
                            let mut points = Vec::new();
                            let mut bounds = vec![0];
                            let mut j = 0;
                            while j < units.len() {
                                let mut c = u32::from(units[j]);
                                j += 1;
                                if (0xd800..=0xdbff).contains(&c)
                                    && j < units.len()
                                    && (0xdc00..=0xdfff).contains(&units[j])
                                {
                                    c = 65536 + (c - 0xd800) * 1024 + u32::from(units[j]) - 0xdc00;
                                    j += 1;
                                }
                                points.push(c);
                                bounds.push(j);
                            }
                            let input = JsString::from_code_units(units);
                            for start in 0..=input.len() + 1 {
                                for sticky in [false, true] {
                                    for (case, matcher) in CASES.iter().zip(&plans) {
                                        let expected = if start > input.len() {
                                            None
                                        } else {
                                            let first = bounds.partition_point(|&b| b <= start) - 1;
                                            (first..=points.len()).find_map(|at| {
                                                if (sticky && at != first)
                                                    || (at != 0 && !(m && lt(points[at - 1])))
                                                {
                                                    return None;
                                                }
                                                let available = points[at..]
                                                    .iter()
                                                    .take_while(|&&c| member(case.1, c))
                                                    .count();
                                                let count =
                                                    available.min(case.3.unwrap_or(available));
                                                if count < case.2 {
                                                    return None;
                                                }
                                                Some(
                                                    bounds[at]
                                                        ..bounds[at
                                                            + if case.4 { count } else { case.2 }],
                                                )
                                            })
                                        };
                                        assert_eq!(
                                            matcher
                                                .find_with_work(&input, start, sticky, |_| Ok::<
                                                    (),
                                                    (),
                                                >(
                                                    ()
                                                ))
                                                .unwrap(),
                                            expected,
                                            "{} i={i} m={m} s={s} {input:?} start={start} sticky={sticky}",
                                            case.0
                                        );
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn unicode_repeated_atom_start_linear_work_clone_and_contracts() {
        let input = JsString::from("aaB\n".repeat(50000).as_str());
        let mut work = 0;
        assert_eq!(
            plan("^a{3}", true, false, true, false)
                .clone()
                .find_with_work(&input, 0, false, |n| {
                    work += n;
                    Ok::<(), ()>(())
                })
                .unwrap(),
            None
        );
        assert!(work < 6 * input.len(), "{work}");
        assert_eq!(
            plan("^a*?", false, false, true, false)
                .find_with_work(&JsString::from("X\r\n"), 2, true, |_| Ok::<(), ()>(()))
                .unwrap(),
            Some(2..2)
        );
        assert_eq!(
            plan("^.+", true, false, false, false)
                .find_with_work(&JsString::from("😀"), 1, true, |_| Ok::<(), ()>(()))
                .unwrap(),
            Some(0..2)
        );
        assert!(
            plan("^.+", true, false, true, true)
                .find_with_work(&input, 0, false, |_| Err(()))
                .is_err()
        );
        let source = JsString::from("^a{2,3}");
        let mut calls = 0;
        RegExpUnicodeRepeatedAtomMatcher::compile_with_start_anchor_and_work(
            &source,
            true,
            true,
            true,
            false,
            |_| {
                calls += 1;
                Ok::<(), ()>(())
            },
        )
        .unwrap()
        .unwrap();
        for fail in 1..=calls {
            let mut n = 0;
            assert!(
                RegExpUnicodeRepeatedAtomMatcher::compile_with_start_anchor_and_work(
                    &source,
                    true,
                    true,
                    true,
                    false,
                    |_| {
                        n += 1;
                        if n == fail { Err(()) } else { Ok(()) }
                    }
                )
                .is_err()
            );
        }
        for source in [
            "^^a+",
            "^a+$",
            r"^\ba+",
            "^(a)+",
            "^a+b",
            r"^[a--b]+",
            r"^\p{Assigned}+",
        ] {
            assert!(
                RegExpUnicodeRepeatedAtomMatcher::compile_with_start_anchor_and_work(
                    &JsString::from(source),
                    true,
                    true,
                    true,
                    false,
                    |_| Ok::<(), ()>(())
                )
                .unwrap()
                .is_none(),
                "{source}"
            );
        }
    }
}

#[derive(Clone, Debug)]
enum Atom {
    Character(RegExpUnicodeCharacterMatcher),
    Class(RegExpUnicodeClassMatcher),
}

#[cfg(test)]
mod end_assertion_tests {
    use super::*;
    use std::fmt::Write;
    const CASES: [(&str, u8, usize, Option<usize>); 14] = [
        ("a+$", 0, 1, None),
        ("a*?$", 0, 0, None),
        ("a??$", 0, 0, Some(1)),
        ("a{2,3}$", 0, 2, Some(3)),
        ("a{002,03}?$", 0, 2, Some(3)),
        (".+$", 1, 1, None),
        (r"\D{1,2}$", 2, 1, Some(2)),
        (r"\W+$", 3, 1, None),
        (r"\p{Join_C}+$", 4, 1, None),
        (r"\p{NChar}{1,2}$", 5, 1, Some(2)),
        (r"\u{61}?$", 0, 0, Some(1)),
        (r"\??$", 6, 0, Some(1)),
        ("a{0,0}$", 0, 0, Some(0)),
        ("[aſK]+$", 7, 1, None),
    ];
    fn plan(source: &str, v: bool, i: bool, s: bool) -> RegExpUnicodeRepeatedAtomMatcher {
        RegExpUnicodeRepeatedAtomMatcher::compile_with_line_anchors_and_work(
            &JsString::from(source),
            v,
            i,
            false,
            s,
            |_| Ok::<(), ()>(()),
        )
        .unwrap()
        .unwrap()
    }
    #[test]
    fn unicode_repeated_atom_end_ranges_snapshot() {
        let mut rows = String::new();
        for (source, ..) in CASES {
            for begin in [false, true] {
                let source = format!("{}{source}", if begin { "^" } else { "" });
                let pattern = JsString::from(source.as_str());
                for v in [false, true] {
                    for i in [false, true] {
                        for s in [false, true] {
                            let matcher = plan(&source, v, i, s);
                            for input in [
                                JsString::from("Aaaa"),
                                JsString::from("A\r\n\u{200c}\u{200d}\u{ffff}\u{10ffff}"),
                                JsString::from("😀??aaaſK"),
                                JsString::from_code_units(vec![0x61, 0xd800, 0x61, 0xdc00]),
                                JsString::from(""),
                            ] {
                                for start in [0, 1, 2, input.len(), input.len() + 1] {
                                    for sticky in [false, true] {
                                        writeln!(rows,"{pattern:?} v={v} i={i} s={s} input={input:?} start={start} sticky={sticky} {:?}",matcher.find_with_work(&input,start,sticky,|_|Ok::<(),()>(())).unwrap()).unwrap();
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        insta::assert_snapshot!(rows);
    }
    #[test]
    fn unicode_repeated_atom_end_agrees_with_independent_start_and_count_oracle() {
        let alphabet: [u16; 10] = [
            0x41, 0x61, 0x0a, 0x17f, 0x212a, 0x200c, 0xffff, 0x3f, 0xd83d, 0xde00,
        ];
        for i in [false, true] {
            for s in [false, true] {
                let fold = |c| {
                    if i {
                        match c {
                            0x41 => 0x61,
                            0x17f => 0x73,
                            0x212a => 0x6b,
                            _ => c,
                        }
                    } else {
                        c
                    }
                };
                let member = |kind, c| match kind {
                    0 => fold(c) == 0x61,
                    1 => s || ![10, 13, 0x2028, 0x2029].contains(&c),
                    2 => !(0x30..=0x39).contains(&c),
                    3 => !char::from_u32(fold(c))
                        .is_some_and(|c| c.is_ascii_alphanumeric() || c == '_'),
                    4 => (0x200c..=0x200d).contains(&c),
                    5 => matches!(c, 0xfdd0..=0xfdef) || (c & 0xffff) >= 0xfffe,
                    6 => c == 0x3f,
                    7 => [0x61, fold(0x17f), fold(0x212a)].contains(&fold(c)),
                    _ => unreachable!(),
                };
                let plans: Vec<_> = CASES
                    .iter()
                    .flat_map(|c| {
                        [
                            plan(c.0, false, i, s),
                            plan(&format!("^{}", c.0), false, i, s),
                        ]
                    })
                    .collect();
                for len in 0..=4 {
                    for ordinal in 0..alphabet.len().pow(len) {
                        let units: Vec<_> = (0..len)
                            .map(|j| alphabet[ordinal / alphabet.len().pow(j) % alphabet.len()])
                            .collect();
                        let mut points = Vec::new();
                        let mut bounds = vec![0];
                        let mut j = 0;
                        while j < units.len() {
                            let mut c = u32::from(units[j]);
                            j += 1;
                            if (0xd800..=0xdbff).contains(&c)
                                && j < units.len()
                                && (0xdc00..=0xdfff).contains(&units[j])
                            {
                                c = 65536 + (c - 0xd800) * 1024 + u32::from(units[j]) - 0xdc00;
                                j += 1;
                            }
                            points.push(c);
                            bounds.push(j);
                        }
                        let input = JsString::from_code_units(units);
                        for start in 0..=input.len() + 1 {
                            for sticky in [false, true] {
                                for (ordinal, matcher) in plans.iter().enumerate() {
                                    let case = &CASES[ordinal / 2];
                                    let begin = ordinal % 2 != 0;
                                    let expected = if start > input.len() {
                                        None
                                    } else {
                                        let first = bounds.partition_point(|&b| b <= start) - 1;
                                        (first..=points.len()).find_map(|at| {
                                            if (sticky && at != first) || (begin && at != 0) {
                                                return None;
                                            }
                                            let count = points.len() - at;
                                            if count < case.2
                                                || case.3.is_some_and(|upper| count > upper)
                                                || !points[at..].iter().all(|&c| member(case.1, c))
                                            {
                                                return None;
                                            }
                                            Some(bounds[at]..input.len())
                                        })
                                    };
                                    assert_eq!(
                                        matcher
                                            .find_with_work(
                                                &input,
                                                start,
                                                sticky,
                                                |_| Ok::<(), ()>(())
                                            )
                                            .unwrap(),
                                        expected,
                                        "{} begin={begin} i={i} s={s} {input:?} start={start} sticky={sticky}",
                                        case.0
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    #[test]
    fn unicode_repeated_atom_end_work_bounds_clone_and_complete_contracts() {
        let input = JsString::from("a".repeat(200000).as_str());
        let mut work = 0;
        assert_eq!(
            plan("a{50000,100000}?$", true, false, false)
                .clone()
                .find_with_work(&input, 0, false, |n| {
                    work += n;
                    Ok::<(), ()>(())
                })
                .unwrap(),
            Some(100000..200000)
        );
        assert!(work < 4 * input.len(), "{work}");
        assert_eq!(
            plan("^a{50000,100000}$", true, false, false)
                .find_with_work(&input, 0, false, |_| Ok::<(), ()>(()))
                .unwrap(),
            None
        );
        assert_eq!(
            plan("a{0}$", false, false, false)
                .find_with_work(&input, 1, false, |_| Ok::<(), ()>(()))
                .unwrap(),
            Some(input.len()..input.len())
        );
        assert_eq!(
            plan("a{0}$", false, false, false)
                .find_with_work(&input, 1, true, |_| Ok::<(), ()>(()))
                .unwrap(),
            None
        );
        assert!(
            plan(".+$", true, false, true)
                .find_with_work(&input, 0, false, |_| Err(()))
                .is_err()
        );
        let source = JsString::from("^a{2,3}$");
        let mut calls = 0;
        RegExpUnicodeRepeatedAtomMatcher::compile_with_line_anchors_and_work(
            &source,
            true,
            true,
            false,
            false,
            |_| {
                calls += 1;
                Ok::<(), ()>(())
            },
        )
        .unwrap()
        .unwrap();
        for fail in 1..=calls {
            let mut n = 0;
            assert!(
                RegExpUnicodeRepeatedAtomMatcher::compile_with_line_anchors_and_work(
                    &source,
                    true,
                    true,
                    false,
                    false,
                    |_| {
                        n += 1;
                        if n == fail { Err(()) } else { Ok(()) }
                    }
                )
                .is_err()
            );
        }
        for (source, m) in [
            ("a+b$", true),
            ("^a+b$", true),
            ("a+$$", false),
            ("(a)+$", false),
            ("a+b$", false),
            (r"\ba+$", false),
            (r"[a--b]+$", false),
        ] {
            assert!(
                RegExpUnicodeRepeatedAtomMatcher::compile_with_line_anchors_and_work(
                    &JsString::from(source),
                    true,
                    true,
                    m,
                    false,
                    |_| Ok::<(), ()>(())
                )
                .unwrap()
                .is_none(),
                "{source}"
            );
        }
    }
}

impl Atom {
    fn find(&self, input: &JsString, at: usize) -> Option<Range<usize>> {
        match self {
            Self::Character(m) => m.find(input, at, true),
            Self::Class(m) => m.find(input, at, true),
        }
    }
    fn work(&self) -> usize {
        match self {
            Self::Character(m) => m.search_passes(),
            Self::Class(m) => m.search_passes(),
        }
    }
}

impl RegExpUnicodeRepeatedAtomMatcher {
    /// Compiles one unanchored consuming atom and quantifier after full validation.
    ///
    /// Exact decimal bounds beyond native storage are retained semantically:
    /// an impossible minimum never matches, and an unreachable maximum cannot
    /// restrict a representable input. Large counts do not allocate repetitions.
    pub fn compile_with_flags_and_work<E>(
        source: &JsString,
        unicode_sets: bool,
        ignore_case: bool,
        dot_all: bool,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, E> {
        charge(source.len().saturating_mul(4))?;
        let units = source.code_units();
        // A braced Unicode escape can look like a numeric quantifier before ?.
        // Prove its retained atom completely before choosing either parse.
        for (end, greedy) in [
            (
                units
                    .len()
                    .checked_sub(1)
                    .filter(|_| units.last() == Some(&0x3f)),
                false,
            ),
            (Some(units.len()), true),
        ] {
            let Some(end) = end else { continue };
            let Some((body_end, minimum, maximum)) = quantifier(units, end) else {
                continue;
            };
            let body = &units[..body_end];
            if body.first() == Some(&0x5e)
                || (body.last() == Some(&0x24) && !escaped(body, body.len() - 1))
            {
                continue;
            }
            charge(body.len())?;
            let body = JsString::from_code_units(body.to_vec());
            let atom = if let Some(m) = RegExpUnicodeCharacterMatcher::compile_with_flags_and_work(
                &body,
                ignore_case,
                false,
                dot_all,
                &mut charge,
            )? {
                Some(Atom::Character(m))
            } else if let Some(m) =
                RegExpUnicodeCharacterMatcher::compile_literal_with_flags_and_work(
                    &body,
                    ignore_case,
                    false,
                    &mut charge,
                )?
            {
                Some(Atom::Character(m))
            } else if let Some(m) = RegExpUnicodeClassMatcher::compile_with_flags_and_work(
                &body,
                unicode_sets,
                ignore_case,
                false,
                &mut charge,
            )? {
                Some(Atom::Class(m))
            } else {
                RegExpUnicodeClassMatcher::compile_binary_property_with_flags_and_work(
                    &body,
                    unicode_sets,
                    ignore_case,
                    false,
                    &mut charge,
                )?
                .map(Atom::Class)
            };
            if let Some(atom) = atom {
                return Ok(Some(Self {
                    atom,
                    minimum,
                    maximum,
                    greedy,
                    start_anchor: false,
                    multiline: false,
                    end_anchor: false,
                    atom_captures: 0,
                }));
            }
        }
        Ok(None)
    }

    /// Compiles one consuming atom and quantifier with an optional leading ^.
    ///
    /// The caller validates the whole u/v Pattern with the supplied flags.
    /// Only the actual beginning or a position after a LineTerminator may pass
    /// the assertion (22.2.2.4). Trailing assertions remain separate proofs.
    pub fn compile_with_start_anchor_and_work<E>(
        source: &JsString,
        unicode_sets: bool,
        ignore_case: bool,
        multiline: bool,
        dot_all: bool,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, E> {
        if source.code_units().first() != Some(&0x5e) {
            return Self::compile_with_flags_and_work(
                source,
                unicode_sets,
                ignore_case,
                dot_all,
                charge,
            );
        }
        charge(source.len())?;
        let body = JsString::from_code_units(source.code_units()[1..].to_vec());
        let Some(mut plan) = Self::compile_with_flags_and_work(
            &body,
            unicode_sets,
            ignore_case,
            dot_all,
            &mut charge,
        )?
        else {
            return Ok(None);
        };
        plan.start_anchor = true;
        plan.multiline = multiline;
        Ok(Some(plan))
    }

    /// Compiles one consuming atom and quantifier with proved line assertions.
    ///
    /// A leading ^ retains its multiline contexts. A trailing $ requires the
    /// actual input end or, with m, a position before a LineTerminator (22.2.2.4).
    /// The caller validates the complete u/v Pattern with these flags first.
    pub fn compile_with_line_anchors_and_work<E>(
        source: &JsString,
        unicode_sets: bool,
        ignore_case: bool,
        multiline: bool,
        dot_all: bool,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, E> {
        let units = source.code_units();
        let end_anchor = units.last() == Some(&0x24) && !escaped(units, units.len() - 1);
        if !end_anchor {
            return Self::compile_with_start_anchor_and_work(
                source,
                unicode_sets,
                ignore_case,
                multiline,
                dot_all,
                charge,
            );
        }
        charge(units.len())?;
        let body = JsString::from_code_units(units[..units.len() - 1].to_vec());
        let Some(mut plan) = Self::compile_with_start_anchor_and_work(
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
        plan.end_anchor = true;
        plan.multiline = multiline;
        Ok(Some(plan))
    }

    /// Proves complete ordinary/noncapturing wrappers inside the repeated atom.
    ///
    /// The caller validates the whole u/v Pattern and normalizes named groups.
    /// Every retained inner body must be one consuming atom. Assertions inside
    /// these wrappers, concatenations, alternatives and empty bodies are excluded.
    /// Optional outer line assertions retain their original scopes (22.2.2.7).
    pub fn compile_capturing_atom_with_work<E>(
        source: &JsString,
        unicode_sets: bool,
        ignore_case: bool,
        multiline: bool,
        dot_all: bool,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, E> {
        let units = source.code_units();
        charge(units.len())?;
        let start_anchor = units.first() == Some(&0x5e);
        let end_anchor = units.last() == Some(&0x24) && !escaped(units, units.len() - 1);
        let start = usize::from(start_anchor);
        let end = units.len().saturating_sub(usize::from(end_anchor));
        let Some(body) = units.get(start..end) else {
            return Ok(None);
        };
        for end in [
            body.len()
                .checked_sub(1)
                .filter(|_| body.last() == Some(&0x3f)),
            Some(body.len()),
        ] {
            let Some(end) = end else { continue };
            let Some((atom_end, _, _)) = quantifier(body, end) else {
                continue;
            };
            let mut at = 0;
            let mut wrappers = 0;
            let mut captures = 0;
            while body.get(at) == Some(&0x28) {
                if body.get(at + 1) == Some(&0x3f) {
                    if body.get(at + 2) != Some(&0x3a) {
                        break;
                    }
                    at += 3;
                } else {
                    at += 1;
                    captures += 1;
                }
                wrappers += 1;
            }
            let Some(inner_end) = atom_end.checked_sub(wrappers) else {
                continue;
            };
            if wrappers == 0
                || at >= inner_end
                || !body[inner_end..atom_end].iter().all(|&c| c == 0x29)
            {
                continue;
            }
            // Prove the retained consuming body without line assertions first.
            // An assertion inside a repeated group must never move outside it.
            charge(body.len())?;
            let mut repeated = Vec::with_capacity(body.len());
            repeated.extend_from_slice(&body[at..inner_end]);
            repeated.extend_from_slice(&body[atom_end..]);
            let repeated = JsString::from_code_units(repeated);
            let Some(mut plan) = Self::compile_with_flags_and_work(
                &repeated,
                unicode_sets,
                ignore_case,
                dot_all,
                &mut charge,
            )?
            else {
                continue;
            };
            plan.start_anchor = start_anchor;
            plan.end_anchor = end_anchor;
            plan.multiline = multiline;
            plan.atom_captures = captures;
            return Ok(Some(plan));
        }
        Ok(None)
    }

    /// Returns the number of complete ordinary wrappers inside the repeated atom.
    pub fn capture_count(&self) -> usize {
        self.atom_captures
    }

    /// Finds the repetition and its final selected complete-character capture.
    ///
    /// Complete inner captures all share the final iteration, and are undefined
    /// when no iteration was selected. This adds no per-capture allocation.
    pub fn find_with_captures_and_work<E>(
        &self,
        input: &JsString,
        start: usize,
        sticky: bool,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<RegExpUnicodeRepeatedAtomMatch>, E> {
        let Some(range) = self.find_with_work(input, start, sticky, &mut charge)? else {
            return Ok(None);
        };
        let atom_capture = if self.atom_captures != 0 && !range.is_empty() {
            charge(2)?;
            let units = input.code_units();
            let mut at = range.end - 1;
            if at > range.start
                && (0xdc00..=0xdfff).contains(&units[at])
                && (0xd800..=0xdbff).contains(&units[at - 1])
            {
                at -= 1;
            }
            Some(at..range.end)
        } else {
            None
        };
        Ok(Some(RegExpUnicodeRepeatedAtomMatch {
            range,
            atom_capture,
        }))
    }

    // With no multiline $, only the actual input end can satisfy the assertion.
    // Enumerating starts in order is equivalent to choosing the earliest start
    // of the final matching run that leaves a count within the exact bounds.
    // A second monotone pointer trims a finite maximum by complete characters.
    fn find_at_actual_end_with_work<E>(
        &self,
        input: &JsString,
        start: usize,
        sticky: bool,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Range<usize>>, E> {
        charge(1)?;
        let Some(minimum) = self.minimum else {
            return Ok(None);
        };
        let Some(mut candidate) = unicode_start(input, start) else {
            return Ok(None);
        };
        if self.start_anchor && candidate != 0 {
            return Ok(None);
        };
        let units = input.code_units();
        if self.maximum == Some(0) {
            return Ok((!(sticky || self.start_anchor) || candidate == units.len())
                .then_some(units.len()..units.len()));
        }
        let mut cursor = candidate;
        let mut count = 0;
        while cursor < units.len() {
            charge(1 + self.atom.work())?;
            if let Some(found) = self.atom.find(input, cursor) {
                cursor = found.end;
                count += 1;
                if self.maximum.is_some_and(|upper| count > upper) {
                    if sticky || self.start_anchor {
                        return Ok(None);
                    };
                    charge(2)?;
                    let (_, width) = unicode_input_character(units, candidate, units[candidate]);
                    candidate += width;
                    count -= 1;
                }
            } else {
                if sticky || self.start_anchor {
                    return Ok(None);
                };
                charge(2)?;
                let (_, width) = unicode_input_character(units, cursor, units[cursor]);
                cursor += width;
                candidate = cursor;
                count = 0;
            }
        }
        Ok((count >= minimum).then_some(candidate..cursor))
    }

    // Complete-character endpoints are visited in order. The earliest start
    // that can reach an eligible endpoint never moves backwards: a finite
    // maximum or a leading assertion can only trim the matching run. Once that
    // start has succeeded, greedy search extends only its endpoint; lazy search
    // returns immediately. Both pointers advance at most once over each input
    // character. Ordinary membership and assertions invoke no user code.
    fn find_at_multiline_end_with_work<E>(
        &self,
        input: &JsString,
        start: usize,
        sticky: bool,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Range<usize>>, E> {
        charge(1)?;
        let Some(minimum) = self.minimum else {
            return Ok(None);
        };
        let Some(mut candidate) = unicode_start(input, start) else {
            return Ok(None);
        };
        let units = input.code_units();
        let mut cursor = candidate;
        let mut count = 0;
        let mut best = None;
        loop {
            if self.start_anchor {
                charge(1)?;
                while candidate != 0
                    && !matches!(units[candidate - 1], 0x0a | 0x0d | 0x2028 | 0x2029)
                {
                    if sticky {
                        return Ok(None);
                    }
                    let Some(&first) = units.get(candidate) else {
                        return Ok(None);
                    };
                    charge(3)?;
                    let (_, width) = unicode_input_character(units, candidate, first);
                    candidate += width;
                    if candidate > cursor {
                        cursor = candidate;
                        count = 0;
                    } else {
                        count -= 1;
                    }
                }
            }
            charge(1)?;
            let at_end =
                cursor == units.len() || matches!(units[cursor], 0x0a | 0x0d | 0x2028 | 0x2029);
            if at_end && count >= minimum {
                best = Some(candidate..cursor);
                if !self.greedy {
                    return Ok(best);
                }
            }
            if cursor == units.len() || (best.is_some() && self.maximum == Some(count)) {
                return Ok(best);
            }
            charge(1 + self.atom.work())?;
            if let Some(found) = self.atom.find(input, cursor) {
                cursor = found.end;
                count += 1;
                if self.maximum.is_some_and(|upper| count > upper) {
                    if sticky {
                        return Ok(None);
                    }
                    charge(2)?;
                    let (_, width) = unicode_input_character(units, candidate, units[candidate]);
                    candidate += width;
                    count -= 1;
                }
            } else {
                if best.is_some() || sticky {
                    return Ok(best);
                }
                charge(2)?;
                let (_, width) = unicode_input_character(units, cursor, units[cursor]);
                cursor += width;
                candidate = cursor;
                count = 0;
            }
        }
    }

    /// Finds the earliest complete-character candidate and its required run.
    ///
    /// A failed short run cannot satisfy the same minimum at a later position
    /// inside that run. Skip it and its failing character once, keeping search
    /// linear and constant in storage. Initial pair offsets use the approved
    /// leading boundary; returned empty ranges use their actual selected position.
    pub fn find_with_work<E>(
        &self,
        input: &JsString,
        start: usize,
        sticky: bool,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Range<usize>>, E> {
        if self.end_anchor {
            return if self.multiline {
                self.find_at_multiline_end_with_work(input, start, sticky, charge)
            } else {
                self.find_at_actual_end_with_work(input, start, sticky, charge)
            };
        }
        charge(1)?;
        let Some(minimum) = self.minimum else {
            return Ok(None);
        };
        let Some(mut candidate) = unicode_start(input, start) else {
            return Ok(None);
        };
        let units = input.code_units();
        loop {
            if self.start_anchor {
                charge(1)?;
                if candidate != 0
                    && !(self.multiline
                        && matches!(units[candidate - 1], 0x0a | 0x0d | 0x2028 | 0x2029))
                {
                    if sticky || !self.multiline {
                        return Ok(None);
                    }
                    let Some(&first) = units.get(candidate) else {
                        return Ok(None);
                    };
                    charge(2)?;
                    let (_, width) = unicode_input_character(units, candidate, first);
                    candidate += width;
                    continue;
                }
            }
            let mut cursor = candidate;
            let mut count = 0;
            while self.maximum.is_none_or(|upper| count < upper) {
                if !self.greedy && count == minimum {
                    break;
                }
                charge(1 + self.atom.work())?;
                let Some(found) = self.atom.find(input, cursor) else {
                    break;
                };
                cursor = found.end;
                count += 1;
            }
            if count >= minimum {
                return Ok(Some(candidate..cursor));
            }
            if sticky {
                return Ok(None);
            }
            let Some(&first) = units.get(cursor) else {
                return Ok(None);
            };
            charge(2)?;
            let (_, width) = unicode_input_character(units, cursor, first);
            candidate = cursor + width;
        }
    }
}

fn escaped(units: &[u16], at: usize) -> bool {
    units[..at].iter().rev().take_while(|&&u| u == 0x5c).count() % 2 != 0
}

#[cfg(test)]
mod captured_atom_tests {
    use super::*;
    use std::fmt::Write;

    type Case = (
        &'static str,
        u8,
        usize,
        Option<usize>,
        bool,
        usize,
        bool,
        bool,
    );
    const CASES: [Case; 16] = [
        ("(a)+", 0, 1, None, true, 1, false, false),
        ("((a)){0,3}?", 0, 0, Some(3), false, 2, false, false),
        ("(?:(a)){2,4}", 0, 2, Some(4), true, 1, false, false),
        ("(?:a)+", 0, 1, None, true, 0, false, false),
        ("((?:a)){0}", 0, 0, Some(0), true, 1, false, false),
        ("([aſK])+", 1, 1, None, true, 1, false, false),
        ("((.)){1,3}$", 2, 1, Some(3), true, 2, false, true),
        (r"^((\D)){0,2}?$", 3, 0, Some(2), false, 2, true, true),
        (r"^(\p{Join_C})+$", 4, 1, None, true, 1, true, true),
        (r"(\p{NChar}){1,2}$", 5, 1, Some(2), true, 1, false, true),
        ("(😀)+", 6, 1, None, true, 1, false, false),
        (r"((\uD800)){0,1}", 7, 0, Some(1), true, 2, false, false),
        (r"(\?)??", 8, 0, Some(1), false, 1, false, false),
        (r"(?:(\W))+", 9, 1, None, true, 1, false, false),
        (r"(\u{61})?", 0, 0, Some(1), true, 1, false, false),
        ("((a)){1,3}$", 0, 1, Some(3), true, 2, false, true),
    ];
    fn plan(source: &str, v: bool, i: bool, m: bool, s: bool) -> RegExpUnicodeRepeatedAtomMatcher {
        RegExpUnicodeRepeatedAtomMatcher::compile_capturing_atom_with_work(
            &JsString::from(source),
            v,
            i,
            m,
            s,
            |_| Ok::<(), ()>(()),
        )
        .unwrap()
        .unwrap()
    }

    #[test]
    fn unicode_captured_atom_ranges_snapshot() {
        let mut rows = String::new();
        for (source, ..) in CASES {
            let pattern = JsString::from(source);
            for v in [false, true] {
                for i in [false, true] {
                    for m in [false, true] {
                        for s in [false, true] {
                            let matcher = plan(source, v, i, m, s);
                            for input in [
                                JsString::from("Aaaa\nAA"),
                                JsString::from("A\r\n\u{200c}\u{200d}\u{ffff}\u{10ffff}"),
                                JsString::from("😀??aaſK"),
                                JsString::from_code_units(vec![0x61, 0xd800, 0x61, 0xdc00]),
                                JsString::from(""),
                            ] {
                                for start in [0, 1, input.len(), input.len() + 1] {
                                    for sticky in [false, true] {
                                        let found = matcher
                                            .find_with_captures_and_work(
                                                &input,
                                                start,
                                                sticky,
                                                |_| Ok::<(), ()>(()),
                                            )
                                            .unwrap();
                                        writeln!(rows,"{pattern:?} v={v} i={i} m={m} s={s} input={input:?} start={start} sticky={sticky} count={} range={:?} capture={:?}",matcher.capture_count(),found.as_ref().map(|f|f.range.clone()),found.and_then(|f|f.atom_capture)).unwrap();
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        insta::assert_snapshot!(rows);
    }

    #[test]
    fn unicode_captured_atoms_agree_with_independent_counts_and_final_character() {
        let alphabet: [u16; 10] = [
            0x41, 0x61, 10, 13, 0x17f, 0x212a, 0xd83d, 0xde00, 0xd800, 0xdc00,
        ];
        let lt = |c| [10, 13, 0x2028, 0x2029].contains(&c);
        for i in [false, true] {
            for m in [false, true] {
                for s in [false, true] {
                    let plans: Vec<_> = CASES.iter().map(|c| plan(c.0, true, i, m, s)).collect();
                    let fold = |c| {
                        if i {
                            match c {
                                0x41 => 0x61,
                                0x17f => 0x73,
                                0x212a => 0x6b,
                                _ => c,
                            }
                        } else {
                            c
                        }
                    };
                    let member = |kind, c| match kind {
                        0 => fold(c) == 0x61,
                        1 => [0x61, fold(0x17f), fold(0x212a)].contains(&fold(c)),
                        2 => s || !lt(c),
                        3 => !(0x30..=0x39).contains(&c),
                        4 => (0x200c..=0x200d).contains(&c),
                        5 => matches!(c, 0xfdd0..=0xfdef) || (c & 0xffff) >= 0xfffe,
                        6 => c == 0x1f600,
                        7 => c == 0xd800,
                        8 => c == 0x3f,
                        9 => !char::from_u32(fold(c))
                            .is_some_and(|c| c.is_ascii_alphanumeric() || c == '_'),
                        _ => unreachable!(),
                    };
                    for len in 0..=4 {
                        for ordinal in 0..alphabet.len().pow(len) {
                            let units: Vec<_> = (0..len)
                                .map(|j| alphabet[ordinal / alphabet.len().pow(j) % alphabet.len()])
                                .collect();
                            let mut points = Vec::new();
                            let mut bounds = vec![0];
                            let mut j = 0;
                            while j < units.len() {
                                let mut c = u32::from(units[j]);
                                j += 1;
                                if (0xd800..=0xdbff).contains(&c)
                                    && j < units.len()
                                    && (0xdc00..=0xdfff).contains(&units[j])
                                {
                                    c = 65536 + (c - 0xd800) * 1024 + u32::from(units[j]) - 0xdc00;
                                    j += 1;
                                }
                                points.push(c);
                                bounds.push(j);
                            }
                            let input = JsString::from_code_units(units);
                            for start in 0..=input.len() + 1 {
                                for sticky in [false, true] {
                                    for (case, matcher) in CASES.iter().zip(&plans) {
                                        let expected = if start > input.len() {
                                            None
                                        } else {
                                            let first = bounds.partition_point(|&b| b <= start) - 1;
                                            (first..=points.len()).find_map(|at| {
                                                if (sticky && at != first)
                                                    || (case.6
                                                        && at != 0
                                                        && !(m && lt(points[at - 1])))
                                                {
                                                    return None;
                                                }
                                                let available = points[at..]
                                                    .iter()
                                                    .take_while(|&&c| member(case.1, c))
                                                    .count();
                                                let upper =
                                                    available.min(case.3.unwrap_or(available));
                                                if upper < case.2 {
                                                    return None;
                                                }
                                                let endpoint = |n: usize| {
                                                    let end = at + n;
                                                    if case.7
                                                        && end != points.len()
                                                        && !(m && lt(points[end]))
                                                    {
                                                        return None;
                                                    }
                                                    Some(RegExpUnicodeRepeatedAtomMatch {
                                                        range: bounds[at]..bounds[end],
                                                        atom_capture: (case.5 != 0 && n != 0)
                                                            .then(|| bounds[end - 1]..bounds[end]),
                                                    })
                                                };
                                                let mut counts = case.2..=upper;
                                                if case.4 {
                                                    counts.rev().find_map(endpoint)
                                                } else {
                                                    counts.find_map(endpoint)
                                                }
                                            })
                                        };
                                        assert_eq!(matcher.capture_count(), case.5, "{}", case.0);
                                        assert_eq!(
                                            matcher
                                                .find_with_captures_and_work(
                                                    &input,
                                                    start,
                                                    sticky,
                                                    |_| Ok::<(), ()>(())
                                                )
                                                .unwrap(),
                                            expected,
                                            "{} i={i} m={m} s={s} {input:?} start={start} sticky={sticky}",
                                            case.0
                                        );
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn unicode_captured_atoms_large_wrappers_charges_clones_and_scope_contracts() {
        let source = format!("{}😀{}+", "(".repeat(3000), ")".repeat(3000));
        let matcher = plan(&source, true, false, false, false).clone();
        let input = JsString::from("😀".repeat(100000).as_str());
        assert_eq!(matcher.capture_count(), 3000);
        let mut work = 0;
        assert_eq!(
            matcher
                .find_with_captures_and_work(&input, 0, false, |n| {
                    work += n;
                    Ok::<(), ()>(())
                })
                .unwrap(),
            Some(RegExpUnicodeRepeatedAtomMatch {
                range: 0..200000,
                atom_capture: Some(199998..200000)
            })
        );
        assert!(work < 6 * input.len(), "{work}");
        let matcher = plan("((?:a)){2,3}$", false, false, true, false);
        let mut calls = 0;
        matcher
            .find_with_captures_and_work(&JsString::from("aaa\n"), 0, false, |_| {
                calls += 1;
                Ok::<(), ()>(())
            })
            .unwrap();
        for fail in 1..=calls {
            let mut n = 0;
            assert!(
                matcher
                    .find_with_captures_and_work(&JsString::from("aaa\n"), 0, false, |_| {
                        n += 1;
                        if n == fail { Err(()) } else { Ok(()) }
                    })
                    .is_err()
            );
        }
        for source in [
            "(^a)+",
            "(a$)+",
            "(a?)+",
            "(a|b)+",
            "()+",
            "(ab)+",
            "(?:a)(a)+",
            "(a)+b",
            "(?i:a)+",
            r"(\b.)+",
            r"(\b)+",
            r"(\B)+",
        ] {
            assert!(
                RegExpUnicodeRepeatedAtomMatcher::compile_capturing_atom_with_work(
                    &JsString::from(source),
                    true,
                    false,
                    true,
                    false,
                    |_| Ok::<(), ()>(())
                )
                .unwrap()
                .is_none(),
                "{source}"
            );
        }
        assert!(
            RegExpUnicodeRepeatedAtomMatcher::compile_with_flags_and_work(
                &JsString::from("(a)+"),
                true,
                false,
                false,
                |_| Ok::<(), ()>(())
            )
            .unwrap()
            .is_none()
        );
    }
}

// The outer Option distinguishes malformed digits from a storage-sized overflow.
fn decimal(units: &[u16]) -> Option<Option<usize>> {
    if units.is_empty() || !units.iter().all(|u| (0x30..=0x39).contains(u)) {
        return None;
    }
    let mut n = Some(0usize);
    for &unit in units {
        n = n.and_then(|n| n.checked_mul(10)?.checked_add(usize::from(unit - 0x30)));
    }
    Some(n)
}

fn significant(units: &[u16]) -> &[u16] {
    &units[units.iter().position(|&u| u != 0x30).unwrap_or(units.len())..]
}

fn quantifier(units: &[u16], end: usize) -> Option<(usize, Option<usize>, Option<usize>)> {
    let last = end.checked_sub(1)?;
    if escaped(units, last) {
        return None;
    }
    match units[last] {
        0x2a => Some((last, Some(0), None)),
        0x2b => Some((last, Some(1), None)),
        0x3f => Some((last, Some(0), Some(1))),
        0x7d => {
            let open = units[..last].iter().rposition(|&u| u == 0x7b)?;
            if escaped(units, open) {
                return None;
            }
            let contents = &units[open + 1..last];
            if let Some(comma) = contents.iter().position(|&u| u == 0x2c) {
                let lower = &contents[..comma];
                let upper = &contents[comma + 1..];
                let minimum = decimal(lower)?;
                let maximum = if upper.is_empty() {
                    None
                } else {
                    let maximum = decimal(upper)?;
                    let (lower, upper) = (significant(lower), significant(upper));
                    if lower.len() > upper.len() || (lower.len() == upper.len() && lower > upper) {
                        return None;
                    }
                    maximum
                };
                Some((open, minimum, maximum))
            } else {
                let exact = decimal(contents)?;
                Some((open, exact, exact))
            }
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fmt::Write;

    fn plan(source: &str, v: bool, i: bool, s: bool) -> RegExpUnicodeRepeatedAtomMatcher {
        RegExpUnicodeRepeatedAtomMatcher::compile_with_flags_and_work(
            &JsString::from(source),
            v,
            i,
            s,
            |_| Ok::<(), ()>(()),
        )
        .unwrap()
        .unwrap()
    }

    #[test]
    fn unicode_repeated_atom_original_ranges_snapshot() {
        let mut rows = String::new();
        for source in [
            r"a+",
            r"a*?",
            r"a??",
            r"a{2,3}",
            r"a{002,03}?",
            r".+",
            r"\D{1,2}",
            r"\W+",
            r"\p{Join_C}+",
            r"\p{NChar}{1,2}",
            r"\u{61}?",
            r"\??",
            r"a{0,0}",
            r"[aſK]+",
        ] {
            for v in [false, true] {
                for i in [false, true] {
                    for s in [false, true] {
                        let matcher = plan(source, v, i, s);
                        for input in [
                            JsString::from("aaa AA1 ſK\n"),
                            JsString::from("A\r\n\u{200c}\u{200d}\u{ffff}\u{10ffff}"),
                            JsString::from("😀??a"),
                            JsString::from_code_units(vec![0x61, 0xd800, 0x61, 0xdc00]),
                            JsString::from(""),
                        ] {
                            for start in [0, 1, 2, input.len(), input.len() + 1] {
                                for sticky in [false, true] {
                                    let found = matcher
                                        .find_with_work(&input, start, sticky, |_| Ok::<(), ()>(()))
                                        .unwrap();
                                    writeln!(rows,"{:?} v={v} i={i} s={s} input={input:?} start={start} sticky={sticky} {found:?}",JsString::from(source)).unwrap();
                                }
                            }
                        }
                    }
                }
            }
        }
        insta::assert_snapshot!(rows);
    }

    #[test]
    fn unicode_repeated_atoms_agree_with_independent_candidate_and_count_oracle() {
        type Case = (&'static str, u8, usize, Option<usize>, bool);
        let cases: &[Case] = &[
            ("a+", 0, 1, None, true),
            ("a*?", 0, 0, None, false),
            ("a??", 0, 0, Some(1), false),
            ("a{2,3}", 0, 2, Some(3), true),
            ("a{002,03}?", 0, 2, Some(3), false),
            (".+", 1, 1, None, true),
            (r"\D{1,2}", 2, 1, Some(2), true),
            (r"\W+", 3, 1, None, true),
            (r"\p{Join_C}+", 4, 1, None, true),
            (r"\p{NChar}{1,2}", 5, 1, Some(2), true),
            (r"\u{61}?", 0, 0, Some(1), true),
            (r"\??", 6, 0, Some(1), true),
            ("a{0,0}", 0, 0, Some(0), true),
            ("[aſK]+", 7, 1, None, true),
        ];
        let alphabet = [
            0x41, 0x61, 0x31, 10, 0x17f, 0x212a, 0x200c, 0xffff, 0xd83d, 0xde00,
        ];
        for i in [false, true] {
            for s in [false, true] {
                let fold = |c| {
                    if i {
                        match c {
                            0x41 => 0x61,
                            0x17f => 0x73,
                            0x212a => 0x6b,
                            _ => c,
                        }
                    } else {
                        c
                    }
                };
                let wanted = |kind, cp| match kind {
                    0 => fold(cp) == 0x61,
                    1 => s || ![10, 13, 0x2028, 0x2029].contains(&cp),
                    2 => !(0x30..=0x39).contains(&cp),
                    3 => !char::from_u32(fold(cp))
                        .is_some_and(|c| c.is_ascii_alphanumeric() || c == '_'),
                    4 => (0x200c..=0x200d).contains(&cp),
                    5 => matches!(cp, 0xfdd0..=0xfdef) || (cp & 0xffff) >= 0xfffe,
                    6 => cp == 0x3f,
                    7 => [0x61, fold(0x17f), fold(0x212a)].contains(&fold(cp)),
                    _ => unreachable!(),
                };
                let plans: Vec<_> = cases.iter().map(|c| plan(c.0, false, i, s)).collect();
                for len in 0..=4 {
                    for ordinal in 0..alphabet.len().pow(len) {
                        let units: Vec<_> = (0..len)
                            .map(|j| alphabet[ordinal / alphabet.len().pow(j) % alphabet.len()])
                            .collect();
                        let mut points = Vec::new();
                        let mut offsets = vec![0];
                        let mut j = 0;
                        while j < units.len() {
                            let mut cp = u32::from(units[j]);
                            j += 1;
                            if (0xd800..=0xdbff).contains(&cp)
                                && j < units.len()
                                && (0xdc00..=0xdfff).contains(&units[j])
                            {
                                cp = 65536 + (cp - 0xd800) * 1024 + u32::from(units[j]) - 0xdc00;
                                j += 1;
                            }
                            points.push(cp);
                            offsets.push(j);
                        }
                        let input = JsString::from_code_units(units);
                        for start in 0..=input.len() + 1 {
                            for sticky in [false, true] {
                                for (case, matcher) in cases.iter().zip(&plans) {
                                    let expected = if start > input.len() {
                                        None
                                    } else {
                                        let first = offsets.partition_point(|&n| n <= start) - 1;
                                        (first..=points.len()).find_map(|at| {
                                            if sticky && at != first {
                                                return None;
                                            }
                                            let available = points[at..]
                                                .iter()
                                                .take_while(|&&cp| wanted(case.1, cp))
                                                .count();
                                            let maximum =
                                                available.min(case.3.unwrap_or(available));
                                            if maximum < case.2 {
                                                return None;
                                            }
                                            let count = if case.4 { maximum } else { case.2 };
                                            Some(offsets[at]..offsets[at + count])
                                        })
                                    };
                                    assert_eq!(
                                        matcher
                                            .find_with_work(
                                                &input,
                                                start,
                                                sticky,
                                                |_| Ok::<(), ()>(())
                                            )
                                            .unwrap(),
                                        expected,
                                        "{} i={i} s={s} {input:?} start={start} sticky={sticky}",
                                        case.0
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn unicode_repeated_atoms_huge_counts_linear_work_clones_and_complete_contracts() {
        let huge = "9".repeat(200);
        let minimum = plan(&format!("a{{{huge}}}"), true, true, false);
        assert_eq!(
            minimum
                .find_with_work(&JsString::from("aaa"), 0, false, |_| Ok::<(), ()>(()))
                .unwrap(),
            None
        );
        assert_eq!(
            plan(&format!("a{{0,{huge}}}"), false, false, false)
                .find_with_work(&JsString::from("aaa"), 0, false, |_| Ok::<(), ()>(()))
                .unwrap(),
            Some(0..3)
        );
        let source = format!("a{{{}2}}", "0".repeat(100000));
        assert_eq!(
            plan(&source, false, false, false)
                .find_with_work(&JsString::from("aaa"), 0, false, |_| Ok::<(), ()>(()))
                .unwrap(),
            Some(0..2)
        );
        let input = JsString::from("aaB".repeat(50000).as_str());
        let mut work = 0;
        assert_eq!(
            plan("a{3}", false, false, false)
                .clone()
                .find_with_work(&input, 0, false, |n| {
                    work += n;
                    Ok::<(), ()>(())
                })
                .unwrap(),
            None
        );
        assert!(work < 3 * input.len(), "{work}");
        assert!(
            plan(".+", true, false, true)
                .find_with_work(&input, 0, false, |_| Err(()))
                .is_err()
        );
        assert!(
            RegExpUnicodeRepeatedAtomMatcher::compile_with_flags_and_work(
                &JsString::from("a+"),
                true,
                true,
                false,
                |_| Err(())
            )
            .is_err()
        );
        for source in [
            "a",
            "ab+",
            "(a)+",
            "^a+",
            "a+$",
            r"\ba+",
            r"[a--b]+",
            r"[\q{ab}]+",
            r"\p{Assigned}+",
            "a{2,1}",
        ] {
            assert!(
                RegExpUnicodeRepeatedAtomMatcher::compile_with_flags_and_work(
                    &JsString::from(source),
                    true,
                    true,
                    false,
                    |_| Ok::<(), ()>(())
                )
                .unwrap()
                .is_none(),
                "{source}"
            );
        }
    }
}

#[cfg(test)]
mod multiline_end_assertion_tests {
    use super::*;
    use std::fmt::Write;
    const CASES: [(&str, u8, usize, Option<usize>, bool); 18] = [
        ("a+$", 0, 1, None, true),
        ("a*?$", 0, 0, None, false),
        ("a??$", 0, 0, Some(1), false),
        ("a{2,3}$", 0, 2, Some(3), true),
        ("a{002,03}?$", 0, 2, Some(3), false),
        (".+$", 1, 1, None, true),
        (r"\D{1,2}$", 2, 1, Some(2), true),
        (r"\W+$", 3, 1, None, true),
        (r"\p{Join_C}+$", 4, 1, None, true),
        (r"\p{NChar}{1,2}$", 5, 1, Some(2), true),
        (r"\u{61}?$", 0, 0, Some(1), true),
        (r"\??$", 6, 0, Some(1), true),
        ("a{0,0}$", 0, 0, Some(0), true),
        ("[aſK]+$", 7, 1, None, true),
        (".+?$", 1, 1, None, false),
        (r"\D+?$", 2, 1, None, false),
        (r"\D{1,3}?$", 2, 1, Some(3), false),
        ("a{1,3}?$", 0, 1, Some(3), false),
    ];
    fn plan(source: &str, v: bool, i: bool, s: bool) -> RegExpUnicodeRepeatedAtomMatcher {
        RegExpUnicodeRepeatedAtomMatcher::compile_with_line_anchors_and_work(
            &JsString::from(source),
            v,
            i,
            true,
            s,
            |_| Ok::<(), ()>(()),
        )
        .unwrap()
        .unwrap()
    }
    #[test]
    fn unicode_repeated_atom_multiline_end_ranges_snapshot() {
        let mut rows = String::new();
        for (source, ..) in CASES {
            for begin in [false, true] {
                let source = format!("{}{source}", if begin { "^" } else { "" });
                let pattern = JsString::from(source.as_str());
                for v in [false, true] {
                    for i in [false, true] {
                        for s in [false, true] {
                            let matcher = plan(&source, v, i, s);
                            for input in [
                                JsString::from("Aaaa\nAA"),
                                JsString::from("A\r\n\u{200c}\u{200d}\u{ffff}\u{10ffff}"),
                                JsString::from("😀??aaaſK"),
                                JsString::from_code_units(vec![0x61, 0xd800, 0x61, 0xdc00]),
                                JsString::from(""),
                            ] {
                                for start in [0, 1, 2, input.len(), input.len() + 1] {
                                    for sticky in [false, true] {
                                        writeln!(rows,"{pattern:?} v={v} i={i} s={s} input={input:?} start={start} sticky={sticky} {:?}",matcher.find_with_work(&input,start,sticky,|_|Ok::<(),()>(())).unwrap()).unwrap();
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        insta::assert_snapshot!(rows);
    }
    #[test]
    fn unicode_repeated_atom_multiline_end_agrees_with_independent_start_and_count_oracle() {
        let alphabet: [u16; 10] = [
            0x41, 0x61, 0x0a, 0x17f, 0x212a, 0x200c, 0xffff, 0x3f, 0xd83d, 0xde00,
        ];
        for i in [false, true] {
            for s in [false, true] {
                let fold = |c| {
                    if i {
                        match c {
                            0x41 => 0x61,
                            0x17f => 0x73,
                            0x212a => 0x6b,
                            _ => c,
                        }
                    } else {
                        c
                    }
                };
                let member = |kind, c| match kind {
                    0 => fold(c) == 0x61,
                    1 => s || ![10, 13, 0x2028, 0x2029].contains(&c),
                    2 => !(0x30..=0x39).contains(&c),
                    3 => !char::from_u32(fold(c))
                        .is_some_and(|c| c.is_ascii_alphanumeric() || c == '_'),
                    4 => (0x200c..=0x200d).contains(&c),
                    5 => matches!(c, 0xfdd0..=0xfdef) || (c & 0xffff) >= 0xfffe,
                    6 => c == 0x3f,
                    7 => [0x61, fold(0x17f), fold(0x212a)].contains(&fold(c)),
                    _ => unreachable!(),
                };
                let plans: Vec<_> = CASES
                    .iter()
                    .flat_map(|c| {
                        [
                            plan(c.0, false, i, s),
                            plan(&format!("^{}", c.0), false, i, s),
                        ]
                    })
                    .collect();
                for len in 0..=4 {
                    for ordinal in 0..alphabet.len().pow(len) {
                        let units: Vec<_> = (0..len)
                            .map(|j| alphabet[ordinal / alphabet.len().pow(j) % alphabet.len()])
                            .collect();
                        let mut points = Vec::new();
                        let mut bounds = vec![0];
                        let mut j = 0;
                        while j < units.len() {
                            let mut c = u32::from(units[j]);
                            j += 1;
                            if (0xd800..=0xdbff).contains(&c)
                                && j < units.len()
                                && (0xdc00..=0xdfff).contains(&units[j])
                            {
                                c = 65536 + (c - 0xd800) * 1024 + u32::from(units[j]) - 0xdc00;
                                j += 1;
                            }
                            points.push(c);
                            bounds.push(j);
                        }
                        let input = JsString::from_code_units(units);
                        for start in 0..=input.len() + 1 {
                            for sticky in [false, true] {
                                for (ordinal, matcher) in plans.iter().enumerate() {
                                    let case = &CASES[ordinal / 2];
                                    let begin = ordinal % 2 != 0;
                                    let expected = if start > input.len() {
                                        None
                                    } else {
                                        let first = bounds.partition_point(|&b| b <= start) - 1;
                                        (first..=points.len()).find_map(|at| {
                                            if (sticky && at != first)
                                                || (begin
                                                    && at != 0
                                                    && ![10, 13, 0x2028, 0x2029]
                                                        .contains(&points[at - 1]))
                                            {
                                                return None;
                                            }
                                            let upper = (points.len() - at)
                                                .min(case.3.unwrap_or(usize::MAX));
                                            if upper < case.2 {
                                                return None;
                                            }
                                            let endpoint = |count: usize| {
                                                let end = at + count;
                                                (points[at..end].iter().all(|&c| member(case.1, c))
                                                    && (end == points.len()
                                                        || [10, 13, 0x2028, 0x2029]
                                                            .contains(&points[end])))
                                                .then_some(bounds[at]..bounds[end])
                                            };
                                            let counts = case.2..=upper;
                                            if case.4 {
                                                counts.rev().find_map(endpoint)
                                            } else {
                                                counts.into_iter().find_map(endpoint)
                                            }
                                        })
                                    };
                                    assert_eq!(
                                        matcher
                                            .find_with_work(
                                                &input,
                                                start,
                                                sticky,
                                                |_| Ok::<(), ()>(())
                                            )
                                            .unwrap(),
                                        expected,
                                        "{} begin={begin} i={i} s={s} {input:?} start={start} sticky={sticky}",
                                        case.0
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    #[test]
    fn unicode_repeated_atom_multiline_end_work_bounds_clone_and_complete_contracts() {
        let input = JsString::from("aaB\n".repeat(50000).as_str());
        let mut work = 0;
        assert_eq!(
            plan("^a{3}$", true, false, false)
                .clone()
                .find_with_work(&input, 0, false, |n| {
                    work += n;
                    Ok::<(), ()>(())
                })
                .unwrap(),
            None
        );
        assert!(work < 12 * input.len(), "{work}");
        let input = JsString::from("aaaa\n".repeat(40000).as_str());
        for (pattern, expected) in [
            (r"\D+$", Some(0..input.len())),
            (r"\D+?$", Some(0..4)),
            (r"\D{1,2}$", Some(2..4)),
        ] {
            let mut work = 0;
            assert_eq!(
                plan(pattern, false, false, false)
                    .find_with_work(&input, 0, false, |n| {
                        work += n;
                        Ok::<(), ()>(())
                    })
                    .unwrap(),
                expected
            );
            assert!(work < 12 * input.len(), "{work}");
        }
        assert_eq!(
            plan("^a{0}$", false, false, false)
                .find_with_work(&JsString::from("\r\n"), 1, true, |_| Ok::<(), ()>(()))
                .unwrap(),
            Some(1..1)
        );
        assert_eq!(
            plan("a{0}$", false, false, false)
                .find_with_work(&JsString::from("😀\n"), 1, true, |_| Ok::<(), ()>(()))
                .unwrap(),
            None
        );
        assert_eq!(
            plan(".+$", true, false, false)
                .find_with_work(&JsString::from("😀\n"), 1, true, |_| Ok::<(), ()>(()))
                .unwrap(),
            Some(0..2)
        );
        let matcher = plan(r"\D+$", false, false, false);
        let mut calls = 0;
        matcher
            .find_with_work(&JsString::from("a\na"), 0, false, |_| {
                calls += 1;
                Ok::<(), ()>(())
            })
            .unwrap();
        for fail in 1..=calls {
            let mut n = 0;
            assert!(
                matcher
                    .find_with_work(&JsString::from("a\na"), 0, false, |_| {
                        n += 1;
                        if n == fail { Err(()) } else { Ok(()) }
                    })
                    .is_err()
            );
        }
        for source in ["a+b$", "(a)+$", "a+$$", r"\ba+$", r"[a--b]+$"] {
            assert!(
                RegExpUnicodeRepeatedAtomMatcher::compile_with_line_anchors_and_work(
                    &JsString::from(source),
                    true,
                    false,
                    true,
                    false,
                    |_| Ok::<(), ()>(())
                )
                .unwrap()
                .is_none(),
                "{source}"
            );
        }
        assert!(
            RegExpUnicodeRepeatedAtomMatcher::compile_with_flags_and_work(
                &JsString::from("a+$"),
                true,
                false,
                false,
                |_| Ok::<(), ()>(())
            )
            .unwrap()
            .is_none()
        );
    }
}
