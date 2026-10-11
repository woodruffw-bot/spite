//! Unicode atom quantifiers and proved boundary assertions (22.2.2.3, 22.2.2.4).
use crate::{
    JsString, RegExpUnicodeCharacterMatcher, RegExpUnicodeClassMatcher,
    regexp_literal::unicode_start,
    regexp_unicode_character::{
        UnicodeWordAssertions, unicode_input_character, unicode_word_assertions_with_work,
        unicode_word_boundary,
    },
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
/// outside line and word assertions, and complete wrappers inside the repeated
/// atom. Scoped entry points preserve word and line predicates at each consuming
/// iteration. Alternatives and concatenations are excluded.
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
    start_word: Option<bool>,
    end_word: Option<bool>,
    word_ignore_case: bool,
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

#[derive(Clone, Copy, PartialEq, Eq)]
enum AtomAssertions {
    None,
    Line,
    Word,
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
        charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, E> {
        Self::compile_atom_repetition_with_work(
            source,
            unicode_sets,
            ignore_case,
            false,
            dot_all,
            AtomAssertions::None,
            charge,
        )
    }

    // Line assertions in this atom are checked at every selected iteration.
    // Only a complete repeated-group proof may request this scoped mode.
    fn compile_atom_repetition_with_work<E>(
        source: &JsString,
        unicode_sets: bool,
        ignore_case: bool,
        multiline: bool,
        dot_all: bool,
        assertions: AtomAssertions,
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
            if assertions == AtomAssertions::None
                && (body.first() == Some(&0x5e)
                    || (body.last() == Some(&0x24) && !escaped(body, body.len() - 1)))
            {
                continue;
            }
            charge(body.len())?;
            let body = JsString::from_code_units(body.to_vec());
            let atom = if let Some(m) = RegExpUnicodeCharacterMatcher::compile_with_flags_and_work(
                &body,
                ignore_case,
                multiline,
                dot_all,
                &mut charge,
            )? {
                Some(Atom::Character(m))
            } else if let Some(m) =
                RegExpUnicodeCharacterMatcher::compile_literal_with_flags_and_work(
                    &body,
                    ignore_case,
                    multiline,
                    &mut charge,
                )?
            {
                Some(Atom::Character(m))
            } else if let Some(m) = RegExpUnicodeClassMatcher::compile_with_flags_and_work(
                &body,
                unicode_sets,
                ignore_case,
                multiline,
                &mut charge,
            )? {
                Some(Atom::Class(m))
            } else {
                let atom = RegExpUnicodeClassMatcher::compile_binary_property_with_flags_and_work(
                    &body,
                    unicode_sets,
                    ignore_case,
                    multiline,
                    &mut charge,
                )?
                .map(Atom::Class);
                if atom.is_some() || assertions != AtomAssertions::Word {
                    atom
                } else if let Some(matcher) =
                    RegExpUnicodeCharacterMatcher::compile_with_word_boundaries_and_work(
                        &body,
                        ignore_case,
                        multiline,
                        dot_all,
                        &mut charge,
                    )?
                {
                    Some(Atom::Character(matcher))
                } else {
                    RegExpUnicodeClassMatcher::compile_with_word_boundaries_and_work(
                        &body,
                        unicode_sets,
                        ignore_case,
                        multiline,
                        &mut charge,
                    )?
                    .map(Atom::Class)
                }
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
                    start_word: None,
                    end_word: None,
                    word_ignore_case: false,
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
        charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, E> {
        Self::compile_wrapped_atom_with_work(
            source,
            unicode_sets,
            ignore_case,
            multiline,
            dot_all,
            AtomAssertions::None,
            charge,
        )
    }

    /// Proves optional line assertions inside complete repeated atom groups.
    ///
    /// Requires complete u/v validation and normalized named groups. Each
    /// selected consuming iteration checks its own ^/$ against original input
    /// context. Complete wrappers capture the final iteration; optional outside
    /// line assertions retain their distinct scopes. Empty units, word assertions,
    /// alternatives and concatenations remain excluded.
    pub fn compile_capturing_atom_with_line_assertions_and_work<E>(
        source: &JsString,
        unicode_sets: bool,
        ignore_case: bool,
        multiline: bool,
        dot_all: bool,
        charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, E> {
        Self::compile_wrapped_atom_with_work(
            source,
            unicode_sets,
            ignore_case,
            multiline,
            dot_all,
            AtomAssertions::Line,
            charge,
        )
    }

    /// Proves word and line assertions inside complete repeated atom groups.
    ///
    /// Requires complete u/v validation and normalized named groups. Each
    /// consuming iteration checks its retained assertions against the original
    /// input context. Optional outside line assertions keep their own scopes.
    /// Empty units, alternatives, concatenations and outside word assertions
    /// remain excluded. Complete inner captures retain the final iteration.
    pub fn compile_capturing_atom_with_assertions_and_work<E>(
        source: &JsString,
        unicode_sets: bool,
        ignore_case: bool,
        multiline: bool,
        dot_all: bool,
        charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, E> {
        Self::compile_wrapped_atom_with_work(
            source,
            unicode_sets,
            ignore_case,
            multiline,
            dot_all,
            AtomAssertions::Word,
            charge,
        )
    }

    fn compile_wrapped_atom_with_work<E>(
        source: &JsString,
        unicode_sets: bool,
        ignore_case: bool,
        multiline: bool,
        dot_all: bool,
        assertions: AtomAssertions,
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
            // Retained assertions belong to the consuming atom when explicitly
            // requested. They must never be moved onto the whole repetition.
            charge(body.len())?;
            let mut repeated = Vec::with_capacity(body.len());
            repeated.extend_from_slice(&body[at..inner_end]);
            repeated.extend_from_slice(&body[atom_end..]);
            let repeated = JsString::from_code_units(repeated);
            let Some(mut plan) = Self::compile_atom_repetition_with_work(
                &repeated,
                unicode_sets,
                ignore_case,
                if assertions != AtomAssertions::None {
                    multiline
                } else {
                    false
                },
                dot_all,
                assertions,
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

    /// Compiles a leading word assertion outside one complete atom repetition.
    ///
    /// Requires complete u/v validation and normalized named groups. An optional
    /// ^ precedes the word assertion; an optional $ follows the repetition.
    /// Complete inner wrappers retain their final-iteration captures. Trailing
    /// word assertions and assertions inside the repeated atom remain excluded.
    pub fn compile_with_start_word_boundary_and_work<E>(
        source: &JsString,
        unicode_sets: bool,
        ignore_case: bool,
        multiline: bool,
        dot_all: bool,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, E> {
        let Some(assertions) = unicode_word_assertions_with_work(source, &mut charge)? else {
            return Ok(None);
        };
        if assertions.start.is_none() || assertions.end.is_some() {
            return Ok(None);
        }
        Self::compile_word_boundary_body_with_work(
            assertions,
            unicode_sets,
            ignore_case,
            multiline,
            dot_all,
            charge,
        )
    }

    /// Compiles optional leading and trailing word assertions on a repetition.
    ///
    /// Requires complete u/v validation and normalized named groups. At least
    /// one outside word assertion is required. Optional ^/$ keep their line
    /// scopes, and complete inner groups retain final selected captures.
    pub fn compile_with_word_boundaries_and_work<E>(
        source: &JsString,
        unicode_sets: bool,
        ignore_case: bool,
        multiline: bool,
        dot_all: bool,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, E> {
        let Some(assertions) = unicode_word_assertions_with_work(source, &mut charge)? else {
            return Ok(None);
        };
        Self::compile_word_boundary_body_with_work(
            assertions,
            unicode_sets,
            ignore_case,
            multiline,
            dot_all,
            charge,
        )
    }

    /// Compiles outside word assertions on complete repeated atom groups.
    ///
    /// Requires complete u/v validation and normalized named groups. The retained
    /// body must pass the consuming-group proof, including any per-iteration line
    /// assertions. Word predicates remain outside the repetition; optional
    /// outside ^/$ keep their separate original scopes. Inner word assertions,
    /// empty units and other compositions remain excluded.
    pub fn compile_with_scoped_word_boundaries_and_work<E>(
        source: &JsString,
        unicode_sets: bool,
        ignore_case: bool,
        multiline: bool,
        dot_all: bool,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, E> {
        let Some(assertions) = unicode_word_assertions_with_work(source, &mut charge)? else {
            return Ok(None);
        };
        let Some(mut plan) = Self::compile_capturing_atom_with_line_assertions_and_work(
            &assertions.source,
            unicode_sets,
            ignore_case,
            multiline,
            dot_all,
            charge,
        )?
        else {
            return Ok(None);
        };
        plan.start_word = assertions.start;
        plan.end_word = assertions.end;
        plan.word_ignore_case = ignore_case;
        Ok(Some(plan))
    }

    /// Compiles outside word assertions on complete repeated atom groups.
    ///
    /// Requires complete u/v validation and normalized named groups. The retained
    /// body must pass the consuming-group proof, including any per-iteration word or line
    /// assertions. Word predicates remain outside the repetition; optional
    /// outside ^/$ keep their separate original scopes. Empty units and other compositions remain excluded.
    pub fn compile_with_composed_word_boundaries_and_work<E>(
        source: &JsString,
        unicode_sets: bool,
        ignore_case: bool,
        multiline: bool,
        dot_all: bool,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, E> {
        let Some(assertions) = unicode_word_assertions_with_work(source, &mut charge)? else {
            return Ok(None);
        };
        let Some(mut plan) = Self::compile_capturing_atom_with_assertions_and_work(
            &assertions.source,
            unicode_sets,
            ignore_case,
            multiline,
            dot_all,
            charge,
        )?
        else {
            return Ok(None);
        };
        plan.start_word = assertions.start;
        plan.end_word = assertions.end;
        plan.word_ignore_case = ignore_case;
        Ok(Some(plan))
    }

    fn compile_word_boundary_body_with_work<E>(
        assertions: UnicodeWordAssertions,
        unicode_sets: bool,
        ignore_case: bool,
        multiline: bool,
        dot_all: bool,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, E> {
        let matcher = Self::compile_with_line_anchors_and_work(
            &assertions.source,
            unicode_sets,
            ignore_case,
            multiline,
            dot_all,
            &mut charge,
        )?;
        let matcher = match matcher {
            Some(matcher) => Some(matcher),
            None => Self::compile_capturing_atom_with_work(
                &assertions.source,
                unicode_sets,
                ignore_case,
                multiline,
                dot_all,
                charge,
            )?,
        };
        Ok(matcher.map(|mut matcher| {
            matcher.start_word = assertions.start;
            matcher.end_word = assertions.end;
            matcher.word_ignore_case = ignore_case;
            matcher
        }))
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
    fn find_with_boundary_assertions_and_work<E>(
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
            if self.start_anchor || self.start_word.is_some() {
                while {
                    charge(1 + 8 * usize::from(self.start_word.is_some()))?;
                    (self.start_anchor
                        && candidate != 0
                        && !(self.multiline
                            && matches!(units[candidate - 1], 0x0a | 0x0d | 0x2028 | 0x2029)))
                        || self.start_word.is_some_and(|expected| {
                            unicode_word_boundary(units, candidate, self.word_ignore_case)
                                != expected
                        })
                } {
                    if sticky || (self.start_anchor && !self.multiline) {
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
            charge(1 + 8 * usize::from(self.end_word.is_some()))?;
            let at_end = !self.end_anchor
                || cursor == units.len()
                || (self.multiline && matches!(units[cursor], 0x0a | 0x0d | 0x2028 | 0x2029));
            let word_end = self.end_word.is_none_or(|expected| {
                unicode_word_boundary(units, cursor, self.word_ignore_case) == expected
            });
            if at_end && word_end && count >= minimum {
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
        if self.start_word.is_some() || self.end_word.is_some() {
            return self.find_with_boundary_assertions_and_work(input, start, sticky, charge);
        }
        if self.end_anchor {
            return if self.multiline {
                self.find_with_boundary_assertions_and_work(input, start, sticky, charge)
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
#[cfg(test)]
mod start_word_tests {
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
        (r"\b(a)+", 0, 1, None, true, 1, false, false),
        (r"\B((a)){0,3}?", 0, 0, Some(3), false, 2, false, false),
        (r"\b(?:(a)){2,4}", 0, 2, Some(4), true, 1, false, false),
        (r"\B(?:a)+", 0, 1, None, true, 0, false, false),
        (r"\B((?:a)){0}", 0, 0, Some(0), true, 1, false, false),
        (r"\b([aſK])+", 1, 1, None, true, 1, false, false),
        (r"\B((.)){1,3}$", 2, 1, Some(3), true, 2, false, true),
        (r"^\b((\D)){0,2}?$", 3, 0, Some(2), false, 2, true, true),
        (r"^\b(\p{Join_C})+$", 4, 1, None, true, 1, true, true),
        (r"\B(\p{NChar}){1,2}$", 5, 1, Some(2), true, 1, false, true),
        (r"\b(😀)+", 6, 1, None, true, 1, false, false),
        (r"\B((\uD800)){0,1}", 7, 0, Some(1), true, 2, false, false),
        (r"\b(\?)??", 8, 0, Some(1), false, 1, false, false),
        (r"\B(?:(\W))+", 9, 1, None, true, 1, false, false),
        (r"\b(\u{61})?", 0, 0, Some(1), true, 1, false, false),
        (r"\B((a)){1,3}$", 0, 1, Some(3), true, 2, false, true),
    ];
    fn plan(source: &str, v: bool, i: bool, m: bool, s: bool) -> RegExpUnicodeRepeatedAtomMatcher {
        RegExpUnicodeRepeatedAtomMatcher::compile_with_start_word_boundary_and_work(
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
    fn unicode_repeated_atom_start_word_ranges_snapshot() {
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
    fn unicode_repeated_atom_start_words_agree_with_independent_counts_and_final_character() {
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
                                                let word = |c: Option<&u32>| c.is_some_and(|&c| {
                                                    matches!(c, 0x30..=0x39 | 0x41..=0x5a | 0x5f | 0x61..=0x7a)
                                                        || (i && matches!(c, 0x17f | 0x212a))
                                                });
                                                let boundary = word(at.checked_sub(1).and_then(|p| points.get(p))) != word(points.get(at));
                                                let positive = case.0.contains(r"\b");
                                                if boundary != positive || (sticky && at != first)
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
    fn unicode_repeated_atom_start_word_linear_work_and_contracts() {
        let source = format!(r"\B{}😀{}+", "(".repeat(3000), ")".repeat(3000));
        let matcher = plan(&source, true, false, false, false).clone();
        let input = JsString::from("😀".repeat(100000).as_str());
        let mut work = 0;
        assert_eq!(matcher.capture_count(), 3000);
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
        assert!(work < 40 * input.len(), "{work}");
        let input = JsString::from(format!("{}X", "a".repeat(200000)).as_str());
        let matcher = plan(r"\Ba{3}$", false, false, false, false);
        work = 0;
        assert_eq!(
            matcher
                .find_with_work(&input, 0, false, |n| {
                    work += n;
                    Ok::<(), ()>(())
                })
                .unwrap(),
            None
        );
        assert!(work < 40 * input.len(), "{work}");
        let matcher = plan(r"\b((a)){2,3}$", false, false, true, false);
        let input = JsString::from("aaa\n");
        let mut calls = 0;
        matcher
            .find_with_captures_and_work(&input, 0, false, |_| {
                calls += 1;
                Ok::<(), ()>(())
            })
            .unwrap();
        for fail in 1..=calls {
            let mut n = 0;
            assert!(
                matcher
                    .find_with_captures_and_work(&input, 0, false, |_| {
                        n += 1;
                        if n == fail { Err(()) } else { Ok(()) }
                    })
                    .is_err()
            );
        }
        for source in [
            r"a+",
            r"\ba+\b",
            r"a+\b",
            r"\b(^a)+",
            r"\b(a$)+",
            r"\b(a?)+",
            r"\b(a|b)+",
            r"\b()+",
            r"\b(ab)+",
            r"\b(a)+b",
            r"\b(?i:a)+",
            r"\b(\b.)+",
            r"\b(\B)+",
        ] {
            assert!(
                RegExpUnicodeRepeatedAtomMatcher::compile_with_start_word_boundary_and_work(
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
        for source in [r"\ba+", r"\B(a)+"] {
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
                .is_none()
            );
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
                .is_none()
            );
        }
    }
}

#[cfg(test)]
mod word_boundary_tests {
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
        Option<bool>,
        bool,
    );
    const CASES: [Case; 16] = [
        (r"(a)+\b", 0, 1, None, true, 1, false, false, None, true),
        (
            r"\B((a)){0,3}?\B",
            0,
            0,
            Some(3),
            false,
            2,
            false,
            false,
            Some(false),
            false,
        ),
        (
            r"\b(?:(a)){2,4}\B",
            0,
            2,
            Some(4),
            true,
            1,
            false,
            false,
            Some(true),
            false,
        ),
        (r"(?:a)+\B", 0, 1, None, true, 0, false, false, None, false),
        (
            r"\B((?:a)){0}\b",
            0,
            0,
            Some(0),
            true,
            1,
            false,
            false,
            Some(false),
            true,
        ),
        (
            r"\b([aſK])+\b",
            1,
            1,
            None,
            true,
            1,
            false,
            false,
            Some(true),
            true,
        ),
        (
            r"((.)){1,3}\b$",
            2,
            1,
            Some(3),
            true,
            2,
            false,
            true,
            None,
            true,
        ),
        (
            r"^\b((\D)){0,2}?\B$",
            3,
            0,
            Some(2),
            false,
            2,
            true,
            true,
            Some(true),
            false,
        ),
        (
            r"^\b(\p{Join_C})+\B$",
            4,
            1,
            None,
            true,
            1,
            true,
            true,
            Some(true),
            false,
        ),
        (
            r"(\p{NChar}){1,2}\B$",
            5,
            1,
            Some(2),
            true,
            1,
            false,
            true,
            None,
            false,
        ),
        (r"(😀)+\B", 6, 1, None, true, 1, false, false, None, false),
        (
            r"((\uD800)){0,1}\b",
            7,
            0,
            Some(1),
            true,
            2,
            false,
            false,
            None,
            true,
        ),
        (
            r"(\?)??\B",
            8,
            0,
            Some(1),
            false,
            1,
            false,
            false,
            None,
            false,
        ),
        (
            r"(?:(\W))+\b",
            9,
            1,
            None,
            true,
            1,
            false,
            false,
            None,
            true,
        ),
        (
            r"\b(\u{61})?\b",
            0,
            0,
            Some(1),
            true,
            1,
            false,
            false,
            Some(true),
            true,
        ),
        (
            r"\B((a)){1,3}\b$",
            0,
            1,
            Some(3),
            true,
            2,
            false,
            true,
            Some(false),
            true,
        ),
    ];
    fn plan(source: &str, v: bool, i: bool, m: bool, s: bool) -> RegExpUnicodeRepeatedAtomMatcher {
        RegExpUnicodeRepeatedAtomMatcher::compile_with_word_boundaries_and_work(
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
    fn unicode_repeated_atom_word_boundary_ranges_snapshot() {
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
    fn unicode_repeated_atom_word_boundaries_agree_with_independent_counts_and_final_character() {
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
                                                let word = |c: Option<&u32>| c.is_some_and(|&c| {
                                                    matches!(c, 0x30..=0x39 | 0x41..=0x5a | 0x5f | 0x61..=0x7a)
                                                        || (i && matches!(c, 0x17f | 0x212a))
                                                });
                                                let boundary = word(at.checked_sub(1).and_then(|p| points.get(p))) != word(points.get(at));
                                                if case.8.is_some_and(|positive| boundary != positive) || (sticky && at != first)
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
                                                    let boundary = word(end.checked_sub(1).and_then(|p| points.get(p))) != word(points.get(end));
                                                    if boundary != case.9 {return None;}
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
    fn unicode_repeated_atom_word_boundary_linear_work_and_contracts() {
        let source = format!(r"\B{}😀{}+\B", "(".repeat(3000), ")".repeat(3000));
        let matcher = plan(&source, true, false, false, false).clone();
        let input = JsString::from("😀".repeat(100000).as_str());
        let mut work = 0;
        assert_eq!(matcher.capture_count(), 3000);
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
        assert!(work < 40 * input.len(), "{work}");
        let input = JsString::from(format!("{}X", "a".repeat(200000)).as_str());
        let matcher = plan(r"\Ba{3}\b$", false, false, false, false);
        work = 0;
        assert_eq!(
            matcher
                .find_with_work(&input, 0, false, |n| {
                    work += n;
                    Ok::<(), ()>(())
                })
                .unwrap(),
            None
        );
        assert!(work < 40 * input.len(), "{work}");
        let matcher = plan(r"\b((a)){2,3}\b$", false, false, true, false);
        let input = JsString::from("aaa\n");
        let mut calls = 0;
        matcher
            .find_with_captures_and_work(&input, 0, false, |_| {
                calls += 1;
                Ok::<(), ()>(())
            })
            .unwrap();
        for fail in 1..=calls {
            let mut n = 0;
            assert!(
                matcher
                    .find_with_captures_and_work(&input, 0, false, |_| {
                        n += 1;
                        if n == fail { Err(()) } else { Ok(()) }
                    })
                    .is_err()
            );
        }
        for source in [
            r"a+",
            r"\b(^a)+",
            r"\b(a$)+",
            r"\b(a?)+",
            r"\b(a|b)+",
            r"\b()+",
            r"\b(ab)+",
            r"\b(a)+b",
            r"\b(?i:a)+",
            r"\b(\b.)+",
            r"\b(\B)+",
        ] {
            assert!(
                RegExpUnicodeRepeatedAtomMatcher::compile_with_word_boundaries_and_work(
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
        for source in [r"\ba+", r"\B(a)+"] {
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
                .is_none()
            );
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
                .is_none()
            );
        }
    }
}
#[cfg(test)]
mod inner_line_tests {
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
        bool,
        bool,
    );
    const CASES: [Case; 16] = [
        (r"(^a)+", 0, 1, None, true, 1, false, false, true, false),
        (
            r"((a$)){0,3}?",
            0,
            0,
            Some(3),
            false,
            2,
            false,
            false,
            false,
            true,
        ),
        (
            r"(?:(^a$)){2,4}",
            0,
            2,
            Some(4),
            true,
            1,
            false,
            false,
            true,
            true,
        ),
        (r"(?:^a)+", 0, 1, None, true, 0, false, false, true, false),
        (
            r"((?:a$)){0}",
            0,
            0,
            Some(0),
            true,
            1,
            false,
            false,
            false,
            true,
        ),
        (r"(^[aſK])+", 1, 1, None, true, 1, false, false, true, false),
        (
            r"((.$)){1,3}$",
            2,
            1,
            Some(3),
            true,
            2,
            false,
            true,
            false,
            true,
        ),
        (
            r"^((^\D$)){0,2}?$",
            3,
            0,
            Some(2),
            false,
            2,
            true,
            true,
            true,
            true,
        ),
        (
            r"^(^\p{Join_C})+$",
            4,
            1,
            None,
            true,
            1,
            true,
            true,
            true,
            false,
        ),
        (
            r"(\p{NChar}$){1,2}$",
            5,
            1,
            Some(2),
            true,
            1,
            false,
            true,
            false,
            true,
        ),
        (r"(^😀)+", 6, 1, None, true, 1, false, false, true, false),
        (
            r"((\uD800$)){0,1}",
            7,
            0,
            Some(1),
            true,
            2,
            false,
            false,
            false,
            true,
        ),
        (
            r"(^\?)??",
            8,
            0,
            Some(1),
            false,
            1,
            false,
            false,
            true,
            false,
        ),
        (
            r"(?:(\W$))+",
            9,
            1,
            None,
            true,
            1,
            false,
            false,
            false,
            true,
        ),
        (
            r"(^\u{61})?",
            0,
            0,
            Some(1),
            true,
            1,
            false,
            false,
            true,
            false,
        ),
        (
            r"((^a)){1,3}$",
            0,
            1,
            Some(3),
            true,
            2,
            false,
            true,
            true,
            false,
        ),
    ];
    fn plan(source: &str, v: bool, i: bool, m: bool, s: bool) -> RegExpUnicodeRepeatedAtomMatcher {
        RegExpUnicodeRepeatedAtomMatcher::compile_capturing_atom_with_line_assertions_and_work(
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
    fn unicode_repeated_atom_inner_line_ranges_snapshot() {
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
    fn unicode_repeated_atom_inner_lines_agree_with_independent_counts_and_final_character() {
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
                                                let available = (at..points.len())
                                                    .take_while(|&p| {
                                                        member(case.1, points[p])
                                                            && (!case.8
                                                                || p == 0
                                                                || (m && lt(points[p - 1])))
                                                            && (!case.9
                                                                || p + 1 == points.len()
                                                                || (m && lt(points[p + 1])))
                                                    })
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
    fn unicode_repeated_atom_inner_line_work_clones_and_scope_contracts() {
        let source = format!("{}^.${}+", "(".repeat(3000), ")".repeat(3000));
        let matcher = plan(&source, true, false, true, true).clone();
        let input = JsString::from("\n".repeat(200000).as_str());
        let mut work = 0;
        assert_eq!(matcher.capture_count(), 3000);
        assert_eq!(
            matcher
                .find_with_captures_and_work(&input, 0, false, |n| {
                    work += n;
                    Ok::<(), ()>(())
                })
                .unwrap(),
            Some(RegExpUnicodeRepeatedAtomMatch {
                range: 0..200000,
                atom_capture: Some(199999..200000)
            })
        );
        assert!(work < 40 * input.len(), "{work}");
        let matcher = plan("(a$){2}", false, false, false, false);
        let input = JsString::from("a".repeat(200000).as_str());
        work = 0;
        assert_eq!(
            matcher
                .find_with_work(&input, 0, false, |n| {
                    work += n;
                    Ok::<(), ()>(())
                })
                .unwrap(),
            None
        );
        assert!(work < 40 * input.len(), "{work}");
        let matcher = plan("((^.$)){2,3}$", true, false, true, true);
        let input = JsString::from("\r\n");
        let mut calls = 0;
        matcher
            .find_with_captures_and_work(&input, 0, false, |_| {
                calls += 1;
                Ok::<(), ()>(())
            })
            .unwrap();
        for fail in 1..=calls {
            let mut n = 0;
            assert!(
                matcher
                    .find_with_captures_and_work(&input, 0, false, |_| {
                        n += 1;
                        if n == fail { Err(()) } else { Ok(()) }
                    })
                    .is_err()
            );
        }
        for source in [
            r"(^$)+",
            r"(a?)+",
            r"(a|b)+",
            r"()+",
            r"(ab)+",
            r"(^a)+b",
            r"(?i:a)+",
            r"(\b.)+",
            r"(\B)+",
            r"(^a(b))+",
            r"(a^)+",
            r"($a)+",
        ] {
            assert!(RegExpUnicodeRepeatedAtomMatcher::compile_capturing_atom_with_line_assertions_and_work(&JsString::from(source),true,false,true,true,|_|Ok::<(),()>(())).unwrap().is_none(),"{source}");
        }
        for source in ["(^a)+", "(a$)+", "((^.$))+", r"(^(?:a))+", r"(^\D$)+"] {
            assert!(
                RegExpUnicodeRepeatedAtomMatcher::compile_capturing_atom_with_work(
                    &JsString::from(source),
                    true,
                    false,
                    true,
                    true,
                    |_| Ok::<(), ()>(())
                )
                .unwrap()
                .is_none()
            );
        }
    }
}

#[cfg(test)]
mod scoped_word_tests {
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
        bool,
        bool,
        Option<bool>,
        Option<bool>,
    );
    const CASES: [Case; 16] = [
        (
            r"\b(^a)+\B",
            0,
            1,
            None,
            true,
            1,
            false,
            false,
            true,
            false,
            Some(true),
            Some(false),
        ),
        (
            r"\B((a$)){0,3}?",
            0,
            0,
            Some(3),
            false,
            2,
            false,
            false,
            false,
            true,
            Some(false),
            None,
        ),
        (
            r"\b(?:(^a$)){2,4}\b",
            0,
            2,
            Some(4),
            true,
            1,
            false,
            false,
            true,
            true,
            Some(true),
            Some(true),
        ),
        (
            r"(?:^a)+\b",
            0,
            1,
            None,
            true,
            0,
            false,
            false,
            true,
            false,
            None,
            Some(true),
        ),
        (
            r"\B((?:a$)){0}\B",
            0,
            0,
            Some(0),
            true,
            1,
            false,
            false,
            false,
            true,
            Some(false),
            Some(false),
        ),
        (
            r"\b(^[aſK])+\b",
            1,
            1,
            None,
            true,
            1,
            false,
            false,
            true,
            false,
            Some(true),
            Some(true),
        ),
        (
            r"((.$)){1,3}\b$",
            2,
            1,
            Some(3),
            true,
            2,
            false,
            true,
            false,
            true,
            None,
            Some(true),
        ),
        (
            r"^\b((^\D$)){0,2}?\B$",
            3,
            0,
            Some(2),
            false,
            2,
            true,
            true,
            true,
            true,
            Some(true),
            Some(false),
        ),
        (
            r"^\B(^\p{Join_C})+\B$",
            4,
            1,
            None,
            true,
            1,
            true,
            true,
            true,
            false,
            Some(false),
            Some(false),
        ),
        (
            r"(\p{NChar}$){1,2}\B$",
            5,
            1,
            Some(2),
            true,
            1,
            false,
            true,
            false,
            true,
            None,
            Some(false),
        ),
        (
            r"\B(^😀)+\B",
            6,
            1,
            None,
            true,
            1,
            false,
            false,
            true,
            false,
            Some(false),
            Some(false),
        ),
        (
            r"((\uD800$)){0,1}\b",
            7,
            0,
            Some(1),
            true,
            2,
            false,
            false,
            false,
            true,
            None,
            Some(true),
        ),
        (
            r"\b(^\?)??",
            8,
            0,
            Some(1),
            false,
            1,
            false,
            false,
            true,
            false,
            Some(true),
            None,
        ),
        (
            r"\B(?:(\W$))+\B",
            9,
            1,
            None,
            true,
            1,
            false,
            false,
            false,
            true,
            Some(false),
            Some(false),
        ),
        (
            r"(^\u{61})?\b",
            0,
            0,
            Some(1),
            true,
            1,
            false,
            false,
            true,
            false,
            None,
            Some(true),
        ),
        (
            r"\b((^a)){1,3}\b$",
            0,
            1,
            Some(3),
            true,
            2,
            false,
            true,
            true,
            false,
            Some(true),
            Some(true),
        ),
    ];
    fn plan(source: &str, v: bool, i: bool, m: bool, s: bool) -> RegExpUnicodeRepeatedAtomMatcher {
        RegExpUnicodeRepeatedAtomMatcher::compile_with_scoped_word_boundaries_and_work(
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
    fn unicode_repeated_atom_scoped_word_ranges_snapshot() {
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
    fn unicode_repeated_atom_scoped_words_agree_with_independent_counts_and_final_character() {
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
                                                let word=|c:Option<&u32>|c.is_some_and(|&c|matches!(c,0x30..=0x39|0x41..=0x5a|0x5f|0x61..=0x7a)||(i&&matches!(c,0x17f|0x212a)));
                                                let boundary=word(at.checked_sub(1).and_then(|p|points.get(p)))!=word(points.get(at));
                                                if case.10.is_some_and(|positive|boundary!=positive) || (sticky && at != first)
                                                    || (case.6
                                                        && at != 0
                                                        && !(m && lt(points[at - 1])))
                                                {
                                                    return None;
                                                }
                                                let available = (at..points.len())
                                                    .take_while(|&p| {
                                                        member(case.1, points[p])
                                                            && (!case.8
                                                                || p == 0
                                                                || (m && lt(points[p - 1])))
                                                            && (!case.9
                                                                || p + 1 == points.len()
                                                                || (m && lt(points[p + 1])))
                                                    })
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
                                                    let boundary=word(end.checked_sub(1).and_then(|p|points.get(p)))!=word(points.get(end));
                                                    if case.11.is_some_and(|positive|boundary!=positive){return None;}
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
    fn unicode_repeated_atom_scoped_word_work_clones_and_scope_contracts() {
        let source = format!(r"\B{}^.${}+\B", "(".repeat(3000), ")".repeat(3000));
        let matcher = plan(&source, true, false, true, true).clone();
        let input = JsString::from("\n".repeat(200000).as_str());
        let mut work = 0;
        assert_eq!(matcher.capture_count(), 3000);
        assert_eq!(
            matcher
                .find_with_captures_and_work(&input, 0, false, |n| {
                    work += n;
                    Ok::<(), ()>(())
                })
                .unwrap(),
            Some(RegExpUnicodeRepeatedAtomMatch {
                range: 0..200000,
                atom_capture: Some(199999..200000)
            })
        );
        assert!(work < 40 * input.len(), "{work}");
        let matcher = plan(r"\B(a$){2}\b", false, false, false, false);
        let input = JsString::from("a".repeat(200000).as_str());
        work = 0;
        assert_eq!(
            matcher
                .find_with_work(&input, 0, false, |n| {
                    work += n;
                    Ok::<(), ()>(())
                })
                .unwrap(),
            None
        );
        assert!(work < 40 * input.len(), "{work}");
        let matcher = plan(r"\B((^.$)){2,3}\B$", true, false, true, true);
        let input = JsString::from("\r\n");
        let mut calls = 0;
        matcher
            .find_with_captures_and_work(&input, 0, false, |_| {
                calls += 1;
                Ok::<(), ()>(())
            })
            .unwrap();
        for fail in 1..=calls {
            let mut n = 0;
            assert!(
                matcher
                    .find_with_captures_and_work(&input, 0, false, |_| {
                        n += 1;
                        if n == fail { Err(()) } else { Ok(()) }
                    })
                    .is_err()
            );
        }
        for source in [
            r"(^a)+",
            r"\b(^$)+",
            r"\b(a?)+",
            r"\b(a|b)+",
            r"\b()+",
            r"\b(ab)+",
            r"\b(^a)+b",
            r"\b(?i:a)+",
            r"\b(\b.)+",
            r"\b(\B)+",
            r"\b(^a(b))+",
            r"\b(a^)+",
        ] {
            assert!(
                RegExpUnicodeRepeatedAtomMatcher::compile_with_scoped_word_boundaries_and_work(
                    &JsString::from(source),
                    true,
                    false,
                    true,
                    true,
                    |_| Ok::<(), ()>(())
                )
                .unwrap()
                .is_none(),
                "{source}"
            );
        }
        for source in [r"\b(^a)+", r"(a$)+\b", r"\B((^.$))+\B"] {
            assert!(
                RegExpUnicodeRepeatedAtomMatcher::compile_with_word_boundaries_and_work(
                    &JsString::from(source),
                    true,
                    false,
                    true,
                    true,
                    |_| Ok::<(), ()>(())
                )
                .unwrap()
                .is_none()
            );
        }
    }
}

#[cfg(test)]
mod inner_word_tests {
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
        bool,
        bool,
        Option<bool>,
        Option<bool>,
    );
    const CASES: [Case; 16] = [
        (
            r"(\ba)+",
            0,
            1,
            None,
            true,
            1,
            false,
            false,
            false,
            false,
            Some(true),
            None,
        ),
        (
            r"((a\b)){0,3}?",
            0,
            0,
            Some(3),
            false,
            2,
            false,
            false,
            false,
            false,
            None,
            Some(true),
        ),
        (
            r"(?:(\ba\b)){2,4}",
            0,
            2,
            Some(4),
            true,
            1,
            false,
            false,
            false,
            false,
            Some(true),
            Some(true),
        ),
        (
            r"(?:\Ba)+",
            0,
            1,
            None,
            true,
            0,
            false,
            false,
            false,
            false,
            Some(false),
            None,
        ),
        (
            r"((?:a\B)){0}",
            0,
            0,
            Some(0),
            true,
            1,
            false,
            false,
            false,
            false,
            None,
            Some(false),
        ),
        (
            r"(^\b[aſK])+",
            1,
            1,
            None,
            true,
            1,
            false,
            false,
            true,
            false,
            Some(true),
            None,
        ),
        (
            r"((.\B$)){1,3}$",
            2,
            1,
            Some(3),
            true,
            2,
            false,
            true,
            false,
            true,
            None,
            Some(false),
        ),
        (
            r"^((^\b\D\B$)){0,2}?$",
            3,
            0,
            Some(2),
            false,
            2,
            true,
            true,
            true,
            true,
            Some(true),
            Some(false),
        ),
        (
            r"^(\B\p{Join_C}\B)+$",
            4,
            1,
            None,
            true,
            1,
            true,
            true,
            false,
            false,
            Some(false),
            Some(false),
        ),
        (
            r"(\p{NChar}\B$){1,2}$",
            5,
            1,
            Some(2),
            true,
            1,
            false,
            true,
            false,
            true,
            None,
            Some(false),
        ),
        (
            r"(\B😀\B)+",
            6,
            1,
            None,
            true,
            1,
            false,
            false,
            false,
            false,
            Some(false),
            Some(false),
        ),
        (
            r"((\uD800\b)){0,1}",
            7,
            0,
            Some(1),
            true,
            2,
            false,
            false,
            false,
            false,
            None,
            Some(true),
        ),
        (
            r"(\b\?)??",
            8,
            0,
            Some(1),
            false,
            1,
            false,
            false,
            false,
            false,
            Some(true),
            None,
        ),
        (
            r"(?:(\B\W\B$))+",
            9,
            1,
            None,
            true,
            1,
            false,
            false,
            false,
            true,
            Some(false),
            Some(false),
        ),
        (
            r"(\u{61}\b)?",
            0,
            0,
            Some(1),
            true,
            1,
            false,
            false,
            false,
            false,
            None,
            Some(true),
        ),
        (
            r"((^\ba\b)){1,3}$",
            0,
            1,
            Some(3),
            true,
            2,
            false,
            true,
            true,
            false,
            Some(true),
            Some(true),
        ),
    ];
    fn plan(source: &str, v: bool, i: bool, m: bool, s: bool) -> RegExpUnicodeRepeatedAtomMatcher {
        RegExpUnicodeRepeatedAtomMatcher::compile_capturing_atom_with_assertions_and_work(
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
    fn unicode_repeated_atom_inner_word_ranges_snapshot() {
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
    fn unicode_repeated_atom_inner_words_agree_with_independent_counts_and_final_character() {
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
                                                let word=|c:Option<&u32>|c.is_some_and(|&c|matches!(c,0x30..=0x39|0x41..=0x5a|0x5f|0x61..=0x7a)||(i&&matches!(c,0x17f|0x212a)));
                                                if (sticky && at != first)
                                                    || (case.6
                                                        && at != 0
                                                        && !(m && lt(points[at - 1])))
                                                {
                                                    return None;
                                                }
                                                let available = (at..points.len())
                                                    .take_while(|&p| {
                                                        member(case.1, points[p])
                                                            && !case.10.is_some_and(|positive| (word(p.checked_sub(1).and_then(|n|points.get(n))) != word(points.get(p))) != positive)
                                                            && !case.11.is_some_and(|positive| (word(points.get(p)) != word(points.get(p+1))) != positive)
                                                            && (!case.8
                                                                || p == 0
                                                                || (m && lt(points[p - 1])))
                                                            && (!case.9
                                                                || p + 1 == points.len()
                                                                || (m && lt(points[p + 1])))
                                                    })
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
    fn unicode_repeated_atom_inner_word_work_clones_and_scope_contracts() {
        let source = format!(r"{}^\B.\B${}+", "(".repeat(3000), ")".repeat(3000));
        let matcher = plan(&source, true, false, true, true).clone();
        let input = JsString::from("\n".repeat(200000).as_str());
        let mut work = 0;
        assert_eq!(matcher.capture_count(), 3000);
        assert_eq!(
            matcher
                .find_with_captures_and_work(&input, 0, false, |n| {
                    work += n;
                    Ok::<(), ()>(())
                })
                .unwrap(),
            Some(RegExpUnicodeRepeatedAtomMatch {
                range: 0..200000,
                atom_capture: Some(199999..200000)
            })
        );
        assert!(work < 50 * input.len(), "{work}");
        let matcher = plan(r"(\ba){2}", false, false, false, false);
        let input = JsString::from("a".repeat(200000).as_str());
        work = 0;
        assert_eq!(
            matcher
                .find_with_work(&input, 0, false, |n| {
                    work += n;
                    Ok::<(), ()>(())
                })
                .unwrap(),
            None
        );
        assert!(work < 50 * input.len(), "{work}");
        let matcher = plan(r"(^\B.\B$){2,3}$", true, false, true, true);
        let input = JsString::from("\r\n");
        let mut calls = 0;
        matcher
            .find_with_captures_and_work(&input, 0, false, |_| {
                calls += 1;
                Ok::<(), ()>(())
            })
            .unwrap();
        for fail in 1..=calls {
            let mut n = 0;
            assert!(
                matcher
                    .find_with_captures_and_work(&input, 0, false, |_| {
                        n += 1;
                        if n == fail { Err(()) } else { Ok(()) }
                    })
                    .is_err()
            );
        }
        for source in [
            r"(\b)+",
            r"(^\B$)+",
            r"(\ba?)+",
            r"(\ba|b)+",
            r"(\bab)+",
            r"(\ba)+b",
            r"\b(\ba)+",
            r"(\ba)+\b",
            r"(a^\b)+",
            r"(\ba(b))+",
            r"(?i:\ba)+",
        ] {
            assert!(
                RegExpUnicodeRepeatedAtomMatcher::compile_capturing_atom_with_assertions_and_work(
                    &JsString::from(source),
                    true,
                    false,
                    true,
                    true,
                    |_| Ok::<(), ()>(())
                )
                .unwrap()
                .is_none(),
                "{source}"
            );
        }
        for source in [r"(\ba)+", r"(a\b)+", r"(^\B.\B$)+"] {
            assert!(RegExpUnicodeRepeatedAtomMatcher::compile_capturing_atom_with_line_assertions_and_work(&JsString::from(source),true,false,true,true,|_|Ok::<(),()>(())).unwrap().is_none(),"{source}");
            assert!(
                RegExpUnicodeRepeatedAtomMatcher::compile_capturing_atom_with_work(
                    &JsString::from(source),
                    true,
                    false,
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
}

#[cfg(test)]
mod composed_word_tests {
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
        bool,
        bool,
        Option<bool>,
        Option<bool>,
        Option<bool>,
        Option<bool>,
    );
    const CASES: [Case; 16] = [
        (
            r"\b(\ba)+\B",
            0,
            1,
            None,
            true,
            1,
            false,
            false,
            false,
            false,
            Some(true),
            None,
            Some(true),
            Some(false),
        ),
        (
            r"\B((a\b)){0,3}?\b",
            0,
            0,
            Some(3),
            false,
            2,
            false,
            false,
            false,
            false,
            None,
            Some(true),
            Some(false),
            Some(true),
        ),
        (
            r"\b(?:(\ba\b)){2,4}\b",
            0,
            2,
            Some(4),
            true,
            1,
            false,
            false,
            false,
            false,
            Some(true),
            Some(true),
            Some(true),
            Some(true),
        ),
        (
            r"\B(?:\Ba)+",
            0,
            1,
            None,
            true,
            0,
            false,
            false,
            false,
            false,
            Some(false),
            None,
            Some(false),
            None,
        ),
        (
            r"((?:a\B)){0}\B",
            0,
            0,
            Some(0),
            true,
            1,
            false,
            false,
            false,
            false,
            None,
            Some(false),
            None,
            Some(false),
        ),
        (
            r"\b(^\b[aſK])+\b",
            1,
            1,
            None,
            true,
            1,
            false,
            false,
            true,
            false,
            Some(true),
            None,
            Some(true),
            Some(true),
        ),
        (
            r"((.\B$)){1,3}\B$",
            2,
            1,
            Some(3),
            true,
            2,
            false,
            true,
            false,
            true,
            None,
            Some(false),
            None,
            Some(false),
        ),
        (
            r"^\b((^\b\D\B$)){0,2}?\B$",
            3,
            0,
            Some(2),
            false,
            2,
            true,
            true,
            true,
            true,
            Some(true),
            Some(false),
            Some(true),
            Some(false),
        ),
        (
            r"^\B(\B\p{Join_C}\B)+\B$",
            4,
            1,
            None,
            true,
            1,
            true,
            true,
            false,
            false,
            Some(false),
            Some(false),
            Some(false),
            Some(false),
        ),
        (
            r"(\p{NChar}\B$){1,2}\B$",
            5,
            1,
            Some(2),
            true,
            1,
            false,
            true,
            false,
            true,
            None,
            Some(false),
            None,
            Some(false),
        ),
        (
            r"\B(\B😀\B)+\B",
            6,
            1,
            None,
            true,
            1,
            false,
            false,
            false,
            false,
            Some(false),
            Some(false),
            Some(false),
            Some(false),
        ),
        (
            r"((\uD800\b)){0,1}\b",
            7,
            0,
            Some(1),
            true,
            2,
            false,
            false,
            false,
            false,
            None,
            Some(true),
            None,
            Some(true),
        ),
        (
            r"\b(\b\?)??",
            8,
            0,
            Some(1),
            false,
            1,
            false,
            false,
            false,
            false,
            Some(true),
            None,
            Some(true),
            None,
        ),
        (
            r"\B(?:(\B\W\B$))+\B",
            9,
            1,
            None,
            true,
            1,
            false,
            false,
            false,
            true,
            Some(false),
            Some(false),
            Some(false),
            Some(false),
        ),
        (
            r"(\u{61}\b)?\b",
            0,
            0,
            Some(1),
            true,
            1,
            false,
            false,
            false,
            false,
            None,
            Some(true),
            None,
            Some(true),
        ),
        (
            r"\b((^\ba\b)){1,3}\b$",
            0,
            1,
            Some(3),
            true,
            2,
            false,
            true,
            true,
            false,
            Some(true),
            Some(true),
            Some(true),
            Some(true),
        ),
    ];
    fn plan(source: &str, v: bool, i: bool, m: bool, s: bool) -> RegExpUnicodeRepeatedAtomMatcher {
        RegExpUnicodeRepeatedAtomMatcher::compile_with_composed_word_boundaries_and_work(
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
    fn unicode_repeated_atom_composed_word_ranges_snapshot() {
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
    fn unicode_repeated_atom_composed_words_agree_with_independent_counts_and_final_character() {
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
                                                let word=|c:Option<&u32>|c.is_some_and(|&c|matches!(c,0x30..=0x39|0x41..=0x5a|0x5f|0x61..=0x7a)||(i&&matches!(c,0x17f|0x212a)));
                                                let start_boundary=word(at.checked_sub(1).and_then(|n|points.get(n)))!=word(points.get(at));
                                                if case.12.is_some_and(|expected|start_boundary!=expected) || (sticky && at != first)
                                                    || (case.6
                                                        && at != 0
                                                        && !(m && lt(points[at - 1])))
                                                {
                                                    return None;
                                                }
                                                let available = (at..points.len())
                                                    .take_while(|&p| {
                                                        member(case.1, points[p])
                                                            && !case.10.is_some_and(|positive| (word(p.checked_sub(1).and_then(|n|points.get(n))) != word(points.get(p))) != positive)
                                                            && !case.11.is_some_and(|positive| (word(points.get(p)) != word(points.get(p+1))) != positive)
                                                            && (!case.8
                                                                || p == 0
                                                                || (m && lt(points[p - 1])))
                                                            && (!case.9
                                                                || p + 1 == points.len()
                                                                || (m && lt(points[p + 1])))
                                                    })
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
                                                    let end_boundary=word(end.checked_sub(1).and_then(|n|points.get(n)))!=word(points.get(end));
                                                    if case.13.is_some_and(|expected|end_boundary!=expected){return None;}
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
    fn unicode_repeated_atom_composed_word_work_clones_and_scope_contracts() {
        let source = format!(r"\B{}^\B.\B${}+\B", "(".repeat(3000), ")".repeat(3000));
        let matcher = plan(&source, true, false, true, true).clone();
        let input = JsString::from("\n".repeat(200000).as_str());
        let mut work = 0;
        assert_eq!(matcher.capture_count(), 3000);
        assert_eq!(
            matcher
                .find_with_captures_and_work(&input, 0, false, |n| {
                    work += n;
                    Ok::<(), ()>(())
                })
                .unwrap(),
            Some(RegExpUnicodeRepeatedAtomMatch {
                range: 0..200000,
                atom_capture: Some(199999..200000)
            })
        );
        assert!(work < 50 * input.len(), "{work}");
        let matcher = plan(r"\b(\ba){2}\b", false, false, false, false);
        let input = JsString::from("a".repeat(200000).as_str());
        work = 0;
        assert_eq!(
            matcher
                .find_with_work(&input, 0, false, |n| {
                    work += n;
                    Ok::<(), ()>(())
                })
                .unwrap(),
            None
        );
        assert!(work < 50 * input.len(), "{work}");
        let matcher = plan(r"\B(^\B.\B$){2,3}\B$", true, false, true, true);
        let input = JsString::from("\r\n");
        let mut calls = 0;
        matcher
            .find_with_captures_and_work(&input, 0, false, |_| {
                calls += 1;
                Ok::<(), ()>(())
            })
            .unwrap();
        for fail in 1..=calls {
            let mut n = 0;
            assert!(
                matcher
                    .find_with_captures_and_work(&input, 0, false, |_| {
                        n += 1;
                        if n == fail { Err(()) } else { Ok(()) }
                    })
                    .is_err()
            );
        }
        for source in [
            r"(\ba)+",
            r"\b(\b)+",
            r"\B(^\B$)+",
            r"\b(\ba?)+",
            r"\b(\ba|b)+",
            r"\b(\bab)+",
            r"\b(\ba)+b",
            r"\b(a^\b)+",
            r"\b(\ba(b))+",
            r"\b(?i:\ba)+",
        ] {
            assert!(
                RegExpUnicodeRepeatedAtomMatcher::compile_with_composed_word_boundaries_and_work(
                    &JsString::from(source),
                    true,
                    false,
                    true,
                    true,
                    |_| Ok::<(), ()>(())
                )
                .unwrap()
                .is_none(),
                "{source}"
            );
        }
        for source in [r"\b(\ba)+", r"(a\b)+\b", r"\B(^\B.\B$)+\B"] {
            assert!(
                RegExpUnicodeRepeatedAtomMatcher::compile_with_scoped_word_boundaries_and_work(
                    &JsString::from(source),
                    true,
                    false,
                    true,
                    true,
                    |_| Ok::<(), ()>(())
                )
                .unwrap()
                .is_none(),
                "{source}"
            );
            assert!(
                RegExpUnicodeRepeatedAtomMatcher::compile_with_word_boundaries_and_work(
                    &JsString::from(source),
                    true,
                    false,
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
}
