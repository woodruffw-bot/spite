use super::*;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Copy)]
enum Leaf {
    Ranges(&'static [(u32, u32)], bool),
    Property(&'static str, bool),
}

struct Case {
    source: &'static str,
    operator: Operator,
    inverted: bool,
    leaves: &'static [Leaf],
}

const POINTS: &[u32] = &[
    0, 1, 0x41, 0x42, 0x49, 0x4b, 0x5a, 0x5b, 0x5d, 0x5f, 0x60, 0x61, 0x62, 0x63, 0x64, 0x6b, 0x73,
    0x7a, 0xdf, 0x130, 0x131, 0x17f, 0x345, 0x378, 0x399, 0x39c, 0x3b9, 0x3bc, 0x3c2, 0x3c3,
    0x1fbe, 0x212a, 0xd800, 0xdc00, 0xe000, 0x10400, 0x10428, 0x1df95, 0x10ffff,
];

const CASES: &[Case] = &[
    // operand folding precedes intersection
    Case {
        source: r"[A&&a]",
        operator: Operator::Intersection,
        inverted: false,
        leaves: &[
            Leaf::Ranges(&[(0x41, 0x41)], false),
            Leaf::Ranges(&[(0x61, 0x61)], false),
        ],
    },
    // operand folding precedes subtraction; no refold resurrection
    Case {
        source: r"[A--a]",
        operator: Operator::Subtraction,
        inverted: false,
        leaves: &[
            Leaf::Ranges(&[(0x41, 0x41)], false),
            Leaf::Ranges(&[(0x61, 0x61)], false),
        ],
    },
    // nested complement after operand folding
    Case {
        source: r"[[^A]&&[a]]",
        operator: Operator::Intersection,
        inverted: false,
        leaves: &[
            Leaf::Ranges(&[(0x41, 0x41)], true),
            Leaf::Ranges(&[(0x61, 0x61)], false),
        ],
    },
    // nested complement used as right subtraction operand
    Case {
        source: r"[[A]--[^a]]",
        operator: Operator::Subtraction,
        inverted: false,
        leaves: &[
            Leaf::Ranges(&[(0x41, 0x41)], false),
            Leaf::Ranges(&[(0x61, 0x61)], true),
        ],
    },
    // outer complement applies to the entire expression
    Case {
        source: r"[^A&&a]",
        operator: Operator::Intersection,
        inverted: true,
        leaves: &[
            Leaf::Ranges(&[(0x41, 0x41)], false),
            Leaf::Ranges(&[(0x61, 0x61)], false),
        ],
    },
    // left subtraction chain
    Case {
        source: r"[[a-c]--[b]--[c]]",
        operator: Operator::Subtraction,
        inverted: false,
        leaves: &[
            Leaf::Ranges(&[(0x61, 0x63)], false),
            Leaf::Ranges(&[(0x62, 0x62)], false),
            Leaf::Ranges(&[(0x63, 0x63)], false),
        ],
    },
    // homogeneous intersection chain
    Case {
        source: r"[[a-c]&&[b-d]&&[c-e]]",
        operator: Operator::Intersection,
        inverted: false,
        leaves: &[
            Leaf::Ranges(&[(0x61, 0x63)], false),
            Leaf::Ranges(&[(0x62, 0x64)], false),
            Leaf::Ranges(&[(0x63, 0x65)], false),
        ],
    },
    // raw range first; noncontiguous folded image and punctuation
    Case {
        source: r"[[Z-a]&&[A-z]]",
        operator: Operator::Intersection,
        inverted: false,
        leaves: &[
            Leaf::Ranges(&[(0x5a, 0x61)], false),
            Leaf::Ranges(&[(0x41, 0x7a)], false),
        ],
    },
    // three-member Kelvin equivalence class
    Case {
        source: r"[K&&k]",
        operator: Operator::Intersection,
        inverted: false,
        leaves: &[
            Leaf::Ranges(&[(0x4b, 0x4b)], false),
            Leaf::Ranges(&[(0x6b, 0x6b)], false),
        ],
    },
    // long s equivalence class
    Case {
        source: r"[s&&\u017f]",
        operator: Operator::Intersection,
        inverted: false,
        leaves: &[
            Leaf::Ranges(&[(0x73, 0x73)], false),
            Leaf::Ranges(&[(0x17f, 0x17f)], false),
        ],
    },
    // three-member sigma equivalence class
    Case {
        source: r"[\u03a3&&\u03c2]",
        operator: Operator::Intersection,
        inverted: false,
        leaves: &[
            Leaf::Ranges(&[(0x3a3, 0x3a3)], false),
            Leaf::Ranges(&[(0x3c2, 0x3c2)], false),
        ],
    },
    // micro sign and Greek mu
    Case {
        source: r"[\u00b5&&\u03bc]",
        operator: Operator::Intersection,
        inverted: false,
        leaves: &[
            Leaf::Ranges(&[(0xb5, 0xb5)], false),
            Leaf::Ranges(&[(0x3bc, 0x3bc)], false),
        ],
    },
    // Turkic mappings are excluded
    Case {
        source: r"[I&&\u0131]",
        operator: Operator::Intersection,
        inverted: false,
        leaves: &[
            Leaf::Ranges(&[(0x49, 0x49)], false),
            Leaf::Ranges(&[(0x131, 0x131)], false),
        ],
    },
    // positive character-property operand
    Case {
        source: r"[\p{gc=Lu}&&a]",
        operator: Operator::Intersection,
        inverted: false,
        leaves: &[
            Leaf::Property("gc=Lu", false),
            Leaf::Ranges(&[(0x61, 0x61)], false),
        ],
    },
    // vi property complement follows folding
    Case {
        source: r"[\P{gc=Lu}&&a]",
        operator: Operator::Intersection,
        inverted: false,
        leaves: &[
            Leaf::Property("gc=Lu", true),
            Leaf::Ranges(&[(0x61, 0x61)], false),
        ],
    },
    // Script Common/Greek cross-fold membership
    Case {
        source: r"[\p{sc=Common}&&\p{sc=Greek}]",
        operator: Operator::Intersection,
        inverted: false,
        leaves: &[
            Leaf::Property("sc=Common", false),
            Leaf::Property("sc=Greek", false),
        ],
    },
    // U+0345/Greek Script preimages
    Case {
        source: r"[\p{sc=Inherited}&&\p{sc=Greek}]",
        operator: Operator::Intersection,
        inverted: false,
        leaves: &[
            Leaf::Property("sc=Inherited", false),
            Leaf::Property("sc=Greek", false),
        ],
    },
    // explicit scx replacement changes the U+0345 result
    Case {
        source: r"[\p{scx=Inherited}&&\p{scx=Greek}]",
        operator: Operator::Intersection,
        inverted: false,
        leaves: &[
            Leaf::Property("scx=Inherited", false),
            Leaf::Property("scx=Greek", false),
        ],
    },
    // Common/Greek scx complement and subtraction
    Case {
        source: r"[\P{scx=Common}--\p{scx=Greek}]",
        operator: Operator::Subtraction,
        inverted: false,
        leaves: &[
            Leaf::Property("scx=Common", true),
            Leaf::Property("scx=Greek", false),
        ],
    },
    // Unknown fallback includes lone-surrogate and unassigned points
    Case {
        source: r"[\p{sc=Unknown}&&\p{scx=Unknown}]",
        operator: Operator::Intersection,
        inverted: false,
        leaves: &[
            Leaf::Property("sc=Unknown", false),
            Leaf::Property("scx=Unknown", false),
        ],
    },
    // normative valid-empty Hrkt, not a Node oracle row
    Case {
        source: r"[\p{sc=Hrkt}&&\p{Any}]",
        operator: Operator::Intersection,
        inverted: false,
        leaves: &[
            Leaf::Property("sc=Hrkt", false),
            Leaf::Property("Any", false),
        ],
    },
    // complement of valid-empty Hrkt
    Case {
        source: r"[\P{scx=Hrkt}--\p{Any}]",
        operator: Operator::Subtraction,
        inverted: false,
        leaves: &[
            Leaf::Property("scx=Hrkt", true),
            Leaf::Property("Any", false),
        ],
    },
    // supplementary Deseret fold collisions
    Case {
        source: r"[[\u{10400}-\u{10427}]&&[\u{10428}-\u{1044f}]]",
        operator: Operator::Intersection,
        inverted: false,
        leaves: &[
            Leaf::Ranges(&[(0x10400, 0x10427)], false),
            Leaf::Ranges(&[(0x10428, 0x1044f)], false),
        ],
    },
    // lone-high-surrogate set, never half of an input pair
    Case {
        source: r"[[\uD800-\uDFFF]--[\uDC00-\uDFFF]]",
        operator: Operator::Subtraction,
        inverted: false,
        leaves: &[
            Leaf::Ranges(&[(0xd800, 0xdfff)], false),
            Leaf::Ranges(&[(0xdc00, 0xdfff)], false),
        ],
    },
    // inclusive endpoint arithmetic
    Case {
        source: r"[[\u{0}-\u{10ffff}]--[\u{1}-\u{10fffe}]]",
        operator: Operator::Subtraction,
        inverted: false,
        leaves: &[
            Leaf::Ranges(&[(0x0, 0x10ffff)], false),
            Leaf::Ranges(&[(0x1, 0x10fffe)], false),
        ],
    },
    // Unicode18 BMP/supplementary simple-fold pair; screen before Node
    Case {
        source: r"[\u00df&&\u{1df95}]",
        operator: Operator::Intersection,
        inverted: false,
        leaves: &[
            Leaf::Ranges(&[(0xdf, 0xdf)], false),
            Leaf::Ranges(&[(0x1df95, 0x1df95)], false),
        ],
    },
    // empty nested flat operand
    Case {
        source: r"[[]&&a]",
        operator: Operator::Intersection,
        inverted: false,
        leaves: &[
            Leaf::Ranges(&[], false),
            Leaf::Ranges(&[(0x61, 0x61)], false),
        ],
    },
    // outer complement of empty result is universal
    Case {
        source: r"[^[]&&a]",
        operator: Operator::Intersection,
        inverted: true,
        leaves: &[
            Leaf::Ranges(&[], false),
            Leaf::Ranges(&[(0x61, 0x61)], false),
        ],
    },
    // universal nested flat operands subtract to empty
    Case {
        source: r"[[^]--[^]]",
        operator: Operator::Subtraction,
        inverted: false,
        leaves: &[Leaf::Ranges(&[], true), Leaf::Ranges(&[], true)],
    },
];

struct PointData {
    category: &'static str,
    script: &'static str,
    extensions: Option<Vec<&'static str>>,
}

struct Reference {
    fold: BTreeMap<u32, u32>,
    groups: BTreeMap<u32, Vec<u32>>,
    aliases: BTreeMap<&'static str, &'static str>,
    points: BTreeMap<u32, PointData>,
}

fn fields(source: &'static str) -> impl Iterator<Item = Vec<&'static str>> {
    source.lines().filter_map(|line| {
        let record = line.split('#').next().unwrap().trim();
        (!record.is_empty()).then(|| record.split(';').map(str::trim).collect())
    })
}

fn bounds(source: &str) -> (u32, u32) {
    let mut parts = source.split("..");
    let first = u32::from_str_radix(parts.next().unwrap(), 16).unwrap();
    let last = parts
        .next()
        .map_or(first, |last| u32::from_str_radix(last, 16).unwrap());
    (first, last)
}

impl Reference {
    fn new() -> Self {
        // Parse original pinned data, independently of generated tables and the
        // production canonicalizer/interval algebra. Store only sparse query
        // points and complete fold-preimage groups, never a Unicode owner map.
        let fold: BTreeMap<_, _> = fields(include_str!("testdata/CaseFolding-18.0.0.txt"))
            .filter(|record| matches!(record[1], "C" | "S"))
            .map(|record| {
                (
                    u32::from_str_radix(record[0], 16).unwrap(),
                    u32::from_str_radix(record[2], 16).unwrap(),
                )
            })
            .collect();
        assert_eq!(fold.len(), 1533);
        assert!(fold.values().all(|target| !fold.contains_key(target)));
        let mut groups = BTreeMap::new();
        for &point in POINTS {
            let canonical = fold.get(&point).copied().unwrap_or(point);
            groups.entry(canonical).or_insert_with(|| vec![canonical]);
        }
        for (&source, target) in &fold {
            if let Some(group) = groups.get_mut(target) {
                group.push(source);
            }
        }
        let mut candidates: BTreeSet<_> = POINTS.iter().copied().collect();
        candidates.extend(groups.values().flatten().copied());
        let mut points: BTreeMap<_, _> = candidates
            .into_iter()
            .map(|point| {
                (
                    point,
                    PointData {
                        category: "Cn",
                        script: "Unknown",
                        extensions: None,
                    },
                )
            })
            .collect();
        let aliases: BTreeMap<_, _> =
            fields(include_str!("testdata/PropertyValueAliases-18.0.0.txt"))
                .filter(|record| record[0] == "sc")
                .flat_map(|record| {
                    record[1..]
                        .iter()
                        .filter(|name| !name.is_empty())
                        .map(|&name| (name, record[2]))
                        .collect::<Vec<_>>()
                })
                .collect();
        let mut first = None;
        for record in fields(include_str!("testdata/UnicodeData-18.0.0.txt")) {
            let point = u32::from_str_radix(record[0], 16).unwrap();
            if record[1].ends_with(", First>") {
                first = Some((point, record[2]));
                continue;
            }
            let start = if record[1].ends_with(", Last>") {
                let (start, category) = first.take().unwrap();
                assert_eq!(category, record[2]);
                start
            } else {
                point
            };
            for data in points.range_mut(start..=point).map(|(_, data)| data) {
                data.category = record[2];
            }
        }
        assert!(first.is_none());
        for record in fields(include_str!("testdata/Scripts-18.0.0.txt")) {
            let (first, last) = bounds(record[0]);
            for data in points.range_mut(first..=last).map(|(_, data)| data) {
                data.script = record[1];
            }
        }
        for record in fields(include_str!("testdata/ScriptExtensions-18.0.0.txt")) {
            let (first, last) = bounds(record[0]);
            let owners: Vec<_> = record[1]
                .split_whitespace()
                .map(|name| aliases[name])
                .collect();
            for data in points.range_mut(first..=last).map(|(_, data)| data) {
                // An explicit set replaces the Script fallback.
                data.extensions = Some(owners.clone());
            }
        }
        Self {
            fold,
            groups,
            aliases,
            points,
        }
    }

    fn property(&self, name: &str, point: u32) -> bool {
        let data = &self.points[&point];
        match name {
            "Any" => true,
            "ASCII" => point < 128,
            "gc=Lu" => data.category == "Lu",
            "gc=Ll" => data.category == "Ll",
            "gc=L" => data.category.starts_with('L'),
            _ if name.starts_with("sc=") => data.script == self.aliases[&name[3..]],
            _ if name.starts_with("scx=") => {
                let owner = self.aliases[&name[4..]];
                data.extensions
                    .as_ref()
                    .map_or(data.script == owner, |owners| owners.contains(&owner))
            }
            _ => panic!("unexpected reference property {name}"),
        }
    }

    fn leaf(&self, leaf: Leaf, input: u32, ignore_case: bool) -> bool {
        let raw_member = |point| match leaf {
            Leaf::Ranges(ranges, _) => ranges
                .iter()
                .any(|&(first, last)| (first..=last).contains(&point)),
            Leaf::Property(name, _) => self.property(name, point),
        };
        let member = if ignore_case {
            let canonical = self.fold.get(&input).copied().unwrap_or(input);
            self.groups[&canonical]
                .iter()
                .any(|&point| raw_member(point))
        } else {
            raw_member(input)
        };
        let inverted = match leaf {
            Leaf::Ranges(_, inverted) | Leaf::Property(_, inverted) => inverted,
        };
        member != inverted
    }

    fn expected(&self, case: &Case, input: u32, ignore_case: bool) -> bool {
        let mut leaves = case.leaves.iter().copied();
        let mut member = self.leaf(leaves.next().unwrap(), input, ignore_case);
        for leaf in leaves {
            let right = self.leaf(leaf, input, ignore_case);
            member = match case.operator {
                Operator::Intersection => member && right,
                Operator::Subtraction => member && !right,
            };
        }
        member != case.inverted
    }
}

fn input(point: u32) -> JsString {
    if point < 0x10000 {
        JsString::from_code_units(vec![point as u16])
    } else {
        let point = point - 0x10000;
        JsString::from_code_units(vec![
            0xd800 + (point >> 10) as u16,
            0xdc00 + (point & 1023) as u16,
        ])
    }
}

fn plan(source: &str, ignore_case: bool, multiline: bool) -> RegExpUnicodeClassMatcher {
    RegExpUnicodeClassMatcher::compile_v_set_with_work(
        &JsString::from(source),
        ignore_case,
        multiline,
        |_| Ok::<(), ()>(()),
    )
    .unwrap()
    .unwrap()
}

#[test]
fn checked_v_set_sparse_pinned_reference_matches_both_modes() {
    let reference = Reference::new();
    for case in CASES {
        for ignore_case in [false, true] {
            let matcher = plan(case.source, ignore_case, false);
            for &point in POINTS {
                let text = input(point);
                let expected = reference
                    .expected(case, point, ignore_case)
                    .then_some(0..text.len());
                assert_eq!(
                    matcher.find(&text, 0, true),
                    expected,
                    "{} i={ignore_case} cp={point:x}",
                    case.source
                );
            }
        }
    }
    for (case, source) in [
        r"[[a]b]",
        r"[[A--a]b]",
        r"[[A&&a]b]",
        r"[A--[a--A]]",
        r"[[a-c]--[[b]c]]",
        r"[^[[^A]&&[a]]]",
        r"[[[K]&&[k]]\u017f]",
        r"[[^]--[[^]--a]]",
        r"[[][a--a]]",
        r"[[[Z-a]&&[A-z]]--[a]]",
        r"[[\p{ASCII}]--[[A]b]]",
    ]
    .into_iter()
    .enumerate()
    {
        for ignore_case in [false, true] {
            let matcher = plan(source, ignore_case, false);
            for &point in POINTS {
                let member = |ranges: &'static [(u32, u32)]| {
                    reference.leaf(Leaf::Ranges(ranges, false), point, ignore_case)
                };
                let upper_a = member(&[(0x41, 0x41)]);
                let lower_a = member(&[(0x61, 0x61)]);
                let lower_b = member(&[(0x62, 0x62)]);
                // Explicit Boolean predicates use the raw pinned leaf oracle,
                // not the production frame evaluator or interval operations.
                let expected = match case {
                    0 => lower_a || lower_b,
                    1 => (upper_a && !lower_a) || lower_b,
                    2 => (upper_a && lower_a) || lower_b,
                    3 => upper_a, // A -- (a -- A) is A, including under folding.
                    4 => member(&[(0x61, 0x63)]) && !(lower_b || member(&[(0x63, 0x63)])),
                    5 => upper_a || !lower_a,
                    6 => {
                        (member(&[(0x4b, 0x4b)]) && member(&[(0x6b, 0x6b)]))
                            || member(&[(0x17f, 0x17f)])
                    }
                    7 => lower_a,
                    8 => false,
                    9 => member(&[(0x5a, 0x61)]) && member(&[(0x41, 0x7a)]) && !lower_a,
                    10 => {
                        reference.leaf(Leaf::Property("ASCII", false), point, ignore_case)
                            && !(upper_a || lower_b)
                    }
                    _ => unreachable!(),
                };
                let text = input(point);
                assert_eq!(
                    matcher.find(&text, 0, true),
                    expected.then_some(0..text.len()),
                    "{source} i={ignore_case} cp={point:x}"
                );
            }
        }
    }
}

#[test]
fn checked_v_set_tokens_class_escapes_and_raw_ranges() {
    for (source, ignore_case, yes, no) in [
        (r"[\&--&]", false, &[][..], &[0x26][..]),
        (r"[\]&&\]]", false, &[0x5d][..], &[0x61][..]),
        (r"[\u{26}&&\&]", false, &[0x26][..], &[0x61][..]),
        (r"[[\--a]&&A]", false, &[0x41][..], &[0x62][..]),
        (r"[\b&&\x08]", false, &[8][..], &[0x62][..]),
        (
            r"[\d--[3-8]]",
            false,
            &[0x30, 0x31, 0x32, 0x39][..],
            &[0x33, 0x38, 0x61][..],
        ),
        (
            r"[\D&&\w]",
            false,
            &[0x41, 0x5f, 0x61][..],
            &[0x30, 0x17f, 0x212a][..],
        ),
        (
            r"[\D&&\w]",
            true,
            &[0x41, 0x5f, 0x61, 0x17f, 0x212a][..],
            &[0x30][..],
        ),
        (
            r"[\W&&[\u017f\u212aA]]",
            false,
            &[0x17f, 0x212a][..],
            &[0x41][..],
        ),
        (
            r"[\W&&[\u017f\u212aA]]",
            true,
            &[][..],
            &[0x17f, 0x212a, 0x41][..],
        ),
        (r"[\S--[\p{ASCII}]]", false, &[0x378][..], &[0xa0, 0x41][..]),
        (
            r"[[\uD801\uDC00]&&\u{10400}]",
            false,
            &[0x10400][..],
            &[0xd801, 0xdc00][..],
        ),
        (
            r"[[\u{D801}\u{DC00}]&&\u{10400}]",
            false,
            &[][..],
            &[0x10400, 0xd801, 0xdc00][..],
        ),
    ] {
        let matcher = plan(source, ignore_case, false);
        for (points, expected) in [(yes, true), (no, false)] {
            for &point in points {
                assert_eq!(
                    matcher.find(&input(point), 0, true).is_some(),
                    expected,
                    "{source} i={ignore_case} cp={point:x}"
                );
            }
        }
    }
}

#[test]
fn checked_v_set_complete_body_and_legacy_domains_remain_distinct() {
    for (source, expected) in [
        ("[[a]]", Some(0..1)),
        ("[[[a]]&&a]", Some(0..1)),
        ("[[a&&a]--a]", None),
        ("[a&&[b--c]]", None),
    ] {
        assert_eq!(
            plan(source, false, false).find(&JsString::from("a"), 0, true),
            expected,
            "{source}"
        );
    }
    for source in [
        "[a]",
        r"[\q{a}&&a]",
        r"[\q{ab}&&a]",
        r"[^\q{ab}&&a]",
        r"[\q{}&&a]",
        r"[a--\q{ab}]",
        r"[a--a--\q{a}]",
        r"[[]&&[\q{ab}]]",
        r"[a--a--\p{RGI_Emoji}]",
        r"[\p{Basic_Emoji}&&a]",
        "[a&&a]b",
        "[a&&a][b]",
        "[a&&a]+",
        "([a&&a])",
        r"\b[a&&a]\b",
        // Invalid source also cannot accidentally produce a proved plan; full
        // Runtime validation retains its separate SyntaxError classification.
        "[a&&]",
        "[a&&b--c]",
        "[a-z&&a]",
        "[a&&&b]",
        "[ab&&c]",
        "[a&&bc]",
        "[a&&b-c]",
        "[[a]-z]",
        "[a-[z]]",
        r"[[a--a]&&[[\q{ab}]]]",
        r"[[][^][\p{RGI_Emoji}]]",
    ] {
        assert!(
            RegExpUnicodeClassMatcher::compile_v_set_with_work(
                &JsString::from(source),
                false,
                false,
                |_| Ok::<(), ()>(()),
            )
            .unwrap()
            .is_none(),
            "{source}"
        );
    }
    for source in ["[a&&a]", "[[a-z]--[b]]", r"[\p{scx=Greek}&&A]"] {
        assert!(
            RegExpUnicodeClassMatcher::compile_with_flags(
                &JsString::from(source),
                true,
                true,
                false
            )
            .is_none()
        );
    }
    assert!(
        RegExpUnicodeClassMatcher::compile_with_flags(
            &JsString::from(r"[\&&]"),
            true,
            false,
            false
        )
        .is_some()
    );
    let depth = 20_000;
    let source = format!("{}a{}", "[".repeat(depth), "]".repeat(depth));
    assert_eq!(
        plan(&source, false, false).find(&JsString::from("a"), 0, true),
        Some(0..1)
    );
    let source = format!("[a{}]", "&&a".repeat(depth));
    assert_eq!(
        plan(&source, false, false).find(&JsString::from("a"), 0, true),
        Some(0..1)
    );
}

#[test]
fn checked_v_set_utf16_starts_and_line_assertions() {
    let matcher = plan(r"[[\u{10400}]--a]", false, false);
    let text = JsString::from("x𐐀𐐀y");
    for (start, expected) in [
        (0, Some(1..3)),
        (1, Some(1..3)),
        (2, Some(1..3)),
        (3, Some(3..5)),
        (4, Some(3..5)),
        (5, None),
        (6, None),
        (7, None),
    ] {
        assert_eq!(matcher.find(&text, start, false), expected, "start={start}");
    }
    assert_eq!(matcher.find(&text, 0, true), None);
    assert_eq!(matcher.find(&text, 2, true), Some(1..3));
    let lone = plan(r"[[\uD800-\uDBFF]--a]", false, false);
    let text = JsString::from_code_units(vec![0xd800, 0x61, 0xdc00, 0xd800, 0xdc00]);
    assert_eq!(lone.find(&text, 0, false), Some(0..1));
    assert_eq!(lone.find(&text, 1, false), None);
    for separator in ["\n", "\r", "\u{2028}", "\u{2029}", "\r\n"] {
        let source = "^[[a]--b]$";
        let text = JsString::from(format!("x{separator}a{separator}y").as_str());
        let start = 1 + separator.encode_utf16().count();
        assert_eq!(
            plan(source, false, true).find(&text, 0, false),
            Some(start..start + 1)
        );
        assert_eq!(plan(source, false, false).find(&text, 0, false), None);
    }
    assert_eq!(
        plan("^[[a]--b]$", false, false).find(&JsString::from("a\n"), 0, false),
        None
    );
    assert_eq!(
        plan("[^[]&&a]", false, false).find(&JsString::from(""), 0, false),
        None
    );
}

#[test]
fn checked_v_set_work_errors_preserve_every_callback_prefix() {
    for (source, ignore_case) in [
        (r"[[\p{ASCII}\dA-Z]--[Q]--a]", false),
        (r"[[K]&&[k]]", true),
        (r"[[a--[b]]c]", false),
    ] {
        let source = JsString::from(source);
        let mut charges = Vec::new();
        RegExpUnicodeClassMatcher::compile_v_set_with_work(&source, ignore_case, false, |work| {
            charges.push(work);
            Ok::<(), usize>(())
        })
        .unwrap()
        .unwrap();
        for failure in 1..=charges.len() {
            let mut seen = Vec::new();
            let result = RegExpUnicodeClassMatcher::compile_v_set_with_work(
                &source,
                ignore_case,
                false,
                |work| {
                    seen.push(work);
                    if seen.len() == failure {
                        Err(failure)
                    } else {
                        Ok(())
                    }
                },
            );
            assert!(matches!(result, Err(RegExpUnicodeSetCompileError::Work(at)) if at == failure));
            assert_eq!(seen, charges[..failure]);
        }
    }
}

#[test]
fn checked_v_set_capacity_failure_preserves_buffers() {
    let charges = std::cell::RefCell::new(Vec::new());
    let mut builder = Builder {
        units: &[],
        at: 0,
        ignore_case: false,
        charge: |work| {
            charges.borrow_mut().push(work);
            Ok::<(), ()>(())
        },
    };
    let mut existing = vec![(0, 0)];
    assert_eq!(
        builder.reserve(&mut existing, usize::MAX),
        Err(RegExpUnicodeSetCompileError::Capacity)
    );
    assert_eq!(existing, [(0, 0)]);
    assert!(charges.borrow().is_empty());
    let mut empty = Vec::new();
    assert_eq!(
        builder.reserve(&mut empty, usize::MAX),
        Err(RegExpUnicodeSetCompileError::Capacity)
    );
    assert!(empty.is_empty());
    assert_eq!(*charges.borrow(), [usize::MAX]);
    charges.borrow_mut().clear();
    let mut frames = vec![Frame::new(false)];
    assert_eq!(
        builder.reserve_frames(&mut frames, usize::MAX),
        Err(RegExpUnicodeSetCompileError::Capacity)
    );
    assert_eq!(frames.len(), 1);
    assert!(charges.borrow().is_empty());
    let mut frames = Vec::new();
    assert_eq!(
        builder.reserve_frames(&mut frames, usize::MAX),
        Err(RegExpUnicodeSetCompileError::Capacity)
    );
    assert!(frames.is_empty());
    assert_eq!(*charges.borrow(), [usize::MAX]);
}

#[test]
fn checked_v_set_closed_children_move_through_transparent_frames() {
    let compile = |depth: usize| {
        let source =
            JsString::from(format!("{}K{}", "[".repeat(depth), "]".repeat(depth)).as_str());
        let mut work = 0usize;
        let matcher =
            RegExpUnicodeClassMatcher::compile_v_set_with_work(&source, true, false, |amount| {
                work = work.checked_add(amount).unwrap();
                Ok::<(), ()>(())
            })
            .unwrap()
            .unwrap();
        for text in ["K", "k", "\u{212a}"] {
            assert_eq!(matcher.find(&JsString::from(text), 0, true), Some(0..1));
        }
        assert_eq!(matcher.find(&JsString::from("x"), 0, true), None);
        work
    };
    let shallow = compile(2);
    let depth = 512;
    let deep = compile(depth);
    // Source scans and geometric frame moves are linear in these wrappers.
    // Forty units per frame leaves ample room for geometric growth, while a
    // single repeated 1,533-record fold pass per wrapper exceeds this bound.
    assert!(
        deep <= shallow + 40 * depth,
        "shallow={shallow} deep={deep}"
    );
}

#[test]
fn checked_v_set_word_predicates_match_independent_neighbors() {
    // Only raw pinned C/S mappings enter this oracle. Neither production class
    // membership nor the production word/UTF-16 helpers supply expectations.
    let folds: BTreeMap<_, _> = fields(include_str!("testdata/CaseFolding-18.0.0.txt"))
        .filter(|record| matches!(record[1], "C" | "S"))
        .map(|record| {
            (
                u32::from_str_radix(record[0], 16).unwrap(),
                u32::from_str_radix(record[2], 16).unwrap(),
            )
        })
        .collect();
    let inputs: Vec<_> = [
        JsString::from(""),
        JsString::from("A A_ a"),
        JsString::from("K ſ αΑ"),
        JsString::from("A😀B"),
        JsString::from_code_units(vec![0x41, 0xd800, 0x5f, 0xdc00, 0x42]),
        JsString::from("x\r\nA\u{2028}K\u{2029}_\n1\r"),
        JsString::from("A\u{8}!"),
    ]
    .into_iter()
    .map(|text| {
        let mut offset = 0;
        let characters: Vec<_> = char::decode_utf16(text.code_units().iter().copied())
            .map(|decoded| {
                let (point, width) = match decoded {
                    Ok(character) => (u32::from(character), character.len_utf16()),
                    Err(error) => (u32::from(error.unpaired_surrogate()), 1),
                };
                let begin = offset;
                offset += width;
                (begin, offset, point)
            })
            .collect();
        (text, characters)
    })
    .collect();
    for (source, members, leading, trailing, start_anchor, end_anchor, multiline) in [
        (
            r"\b[[A]]\b",
            Some(&[0x41][..]),
            Some(true),
            Some(true),
            false,
            false,
            false,
        ),
        (
            r"\B[[A]]\B",
            Some(&[0x41][..]),
            Some(false),
            Some(false),
            false,
            false,
            false,
        ),
        (
            r"\b[[A]]\B",
            Some(&[0x41][..]),
            Some(true),
            Some(false),
            false,
            false,
            false,
        ),
        (
            r"\B[[A]]\b",
            Some(&[0x41][..]),
            Some(false),
            Some(true),
            false,
            false,
            false,
        ),
        (
            r"\b[[Kſα]]",
            Some(&[0x4b, 0x17f, 0x3b1][..]),
            Some(true),
            None,
            false,
            false,
            false,
        ),
        (
            r"\B[[Kſα]]",
            Some(&[0x4b, 0x17f, 0x3b1][..]),
            Some(false),
            None,
            false,
            false,
            false,
        ),
        (
            r"[[Kſα]]\b",
            Some(&[0x4b, 0x17f, 0x3b1][..]),
            None,
            Some(true),
            false,
            false,
            false,
        ),
        (
            r"[[Kſα]]\B",
            Some(&[0x4b, 0x17f, 0x3b1][..]),
            None,
            Some(false),
            false,
            false,
            false,
        ),
        (
            r"^\b[[Kſα]]\b$",
            Some(&[0x4b, 0x17f, 0x3b1][..]),
            Some(true),
            Some(true),
            true,
            true,
            false,
        ),
        (
            r"^\b[[Kſα]]\b$",
            Some(&[0x4b, 0x17f, 0x3b1][..]),
            Some(true),
            Some(true),
            true,
            true,
            true,
        ),
        (
            r"^\B[[\p{Any}]]\B$",
            None,
            Some(false),
            Some(false),
            true,
            true,
            true,
        ),
        (
            r"\b[[\b]]\B",
            Some(&[8][..]),
            Some(true),
            Some(false),
            false,
            false,
            false,
        ),
        (
            r"\b[[a--a]]\b",
            Some(&[][..]),
            Some(true),
            Some(true),
            false,
            false,
            false,
        ),
        (
            r"\B[[\p{Any}]]\B",
            None,
            Some(false),
            Some(false),
            false,
            false,
            false,
        ),
        (
            r"^\b[[A]]",
            Some(&[0x41][..]),
            Some(true),
            None,
            true,
            false,
            true,
        ),
        (
            r"[[A]]\b$",
            Some(&[0x41][..]),
            None,
            Some(true),
            false,
            true,
            true,
        ),
    ] {
        for ignore_case in [false, true] {
            let matcher = RegExpUnicodeClassMatcher::compile_v_set_with_word_boundaries_and_work(
                &JsString::from(source),
                ignore_case,
                multiline,
                |_| Ok::<(), ()>(()),
            )
            .unwrap()
            .unwrap();
            let canonical = |point| {
                if ignore_case {
                    folds.get(&point).copied().unwrap_or(point)
                } else {
                    point
                }
            };
            for (text, characters) in &inputs {
                let units = text.code_units();
                let word = |unit: Option<&u16>| {
                    unit.is_some_and(|&unit| matches!(canonical(u32::from(unit)), 0x30..=0x39 | 0x41..=0x5a | 0x5f | 0x61..=0x7a))
                };
                let boundary = |at: usize| {
                    word(at.checked_sub(1).and_then(|previous| units.get(previous)))
                        != word(units.get(at))
                };
                let line = |unit: u16| matches!(unit, 0x0a | 0x0d | 0x2028 | 0x2029);
                for start in 0..=units.len() + 1 {
                    let normalized = characters
                        .iter()
                        .find(|&&(begin, end, _)| begin <= start && start < end)
                        .map_or(start, |&(begin, _, _)| begin);
                    for sticky in [false, true] {
                        let expected = characters
                            .iter()
                            .find(|&&(begin, end, point)| {
                                start <= units.len()
                                    && begin >= normalized
                                    && (!sticky || begin == normalized)
                                    && (!start_anchor
                                        || begin == 0
                                        || (multiline && line(units[begin - 1])))
                                    && (!end_anchor
                                        || end == units.len()
                                        || (multiline && line(units[end])))
                                    && leading.is_none_or(|wanted| boundary(begin) == wanted)
                                    && trailing.is_none_or(|wanted| boundary(end) == wanted)
                                    && members.is_none_or(|points| {
                                        points.iter().any(|&raw| canonical(raw) == canonical(point))
                                    })
                            })
                            .map(|&(begin, end, _)| begin..end);
                        assert_eq!(
                            matcher.find(text, start, sticky),
                            expected,
                            "{source} i={ignore_case} m={multiline} input={text:?} start={start} sticky={sticky}"
                        );
                    }
                }
            }
        }
    }
}

#[test]
fn checked_v_set_word_domains_work_and_deep_bodies() {
    for source in [
        "",
        r"\b",
        r"^\b$",
        "[a&&a]",
        r"\b[A]\b",
        r"\b[^]\b",
        r"\b[[A]]+\b",
        r"\b([[A]])\b",
        r"(\b[[A]]\b)",
        r"\b[[A]][[B]]\b",
        r"\b[[A]]\bb",
        r"\b\b[[A]]\b",
        r"\b\B[[A]]\b",
        r"\b^[[A]]\b",
        r"\b[[A]]$\b",
        r"^^\b[[A]]\b$",
        r"\b[[A]]\b$$",
        r"\b[[\q{a}]]\b",
        r"\b[[\p{RGI_Emoji}]]\b",
        r"\b[[A]]\\b",
        r"\b[[A]]\$",
    ] {
        assert!(
            RegExpUnicodeClassMatcher::compile_v_set_with_word_boundaries_and_work(
                &JsString::from(source),
                false,
                false,
                |_| Ok::<(), ()>(()),
            )
            .unwrap()
            .is_none(),
            "{source}"
        );
    }
    for (source, ignore_case) in [(r"^\b[[A]b]\B$", false), (r"\b[[K]]\b", true)] {
        let source = JsString::from(source);
        let mut charges = Vec::new();
        RegExpUnicodeClassMatcher::compile_v_set_with_word_boundaries_and_work(
            &source,
            ignore_case,
            false,
            |work| {
                charges.push(work);
                Ok::<(), usize>(())
            },
        )
        .unwrap()
        .unwrap();
        for failure in 1..=charges.len() {
            let mut seen = Vec::new();
            let result = RegExpUnicodeClassMatcher::compile_v_set_with_word_boundaries_and_work(
                &source,
                ignore_case,
                false,
                |work| {
                    seen.push(work);
                    if seen.len() == failure {
                        Err(failure)
                    } else {
                        Ok(())
                    }
                },
            );
            assert!(matches!(result, Err(RegExpUnicodeSetCompileError::Work(at)) if at == failure));
            assert_eq!(seen, charges[..failure]);
        }
    }
    let depth = 512;
    let source =
        JsString::from(format!(r"\b{}K{}\b", "[".repeat(depth), "]".repeat(depth)).as_str());
    for (ignore_case, text) in [(false, " K "), (true, " K ")] {
        let matcher = RegExpUnicodeClassMatcher::compile_v_set_with_word_boundaries_and_work(
            &source,
            ignore_case,
            false,
            |_| Ok::<(), ()>(()),
        )
        .unwrap()
        .unwrap();
        assert_eq!(matcher.find(&JsString::from(text), 0, false), Some(1..2));
    }
}
