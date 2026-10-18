use super::{JsString, RegExpUnicodeClassMatcher, RegExpUnicodeSetCompileError};
use std::ops::Range;

struct Input {
    text: JsString,
    points: Vec<u32>,
    boundaries: Vec<usize>,
}

impl Input {
    fn new(text: JsString) -> Self {
        let mut points = Vec::new();
        let mut boundaries = vec![0];
        for decoded in char::decode_utf16(text.code_units().iter().copied()) {
            let (point, width) = match decoded {
                Ok(character) => (u32::from(character), character.len_utf16()),
                Err(error) => (u32::from(error.unpaired_surrogate()), 1),
            };
            points.push(point);
            boundaries.push(boundaries.last().unwrap() + width);
        }
        Self {
            text,
            points,
            boundaries,
        }
    }
}

// A hand-written finite expression model, not a parser or production interval
// implementation. These are all fold aliases of the characters used below.
#[derive(Clone, Copy)]
enum Leaf {
    Points(&'static [u32]),
    Range(u32, u32),
    Ascii,
    Digit,
    Word,
    Space,
}

enum Expr {
    Leaf(Leaf),
    Not(&'static Self),
    Union(&'static [Self]),
    Intersection(&'static [Self]),
    Subtraction(&'static [Self]),
}

impl Leaf {
    fn raw(self, point: u32) -> bool {
        match self {
            Self::Points(points) => points.contains(&point),
            Self::Range(first, last) => (first..=last).contains(&point),
            Self::Ascii => point <= 0x7f,
            Self::Digit => (0x30..=0x39).contains(&point),
            Self::Word => matches!(point, 0x30..=0x39 | 0x41..=0x5a | 0x5f | 0x61..=0x7a),
            Self::Space => matches!(
                point,
                9..=0xd | 0x20 | 0xa0 | 0x1680 | 0x2000..=0x200a | 0x2028 | 0x2029
                    | 0x202f | 0x205f | 0x3000 | 0xfeff
            ),
        }
    }

    fn contains(self, point: u32, ignore_case: bool) -> bool {
        if self.raw(point) {
            return true;
        }
        if !ignore_case {
            return false;
        }
        // Closure of each leaf precedes Boolean operations and complements.
        let aliases: &[u32] = match point {
            0x4b | 0x6b | 0x212a => &[0x4b, 0x6b, 0x212a],
            0x53 | 0x73 | 0x17f => &[0x53, 0x73, 0x17f],
            0x10400 | 0x10428 => &[0x10400, 0x10428],
            _ => &[],
        };
        aliases.iter().any(|&alias| self.raw(alias))
            || matches!(point, 0x41..=0x5a) && self.raw(point + 0x20)
            || matches!(point, 0x61..=0x7a) && self.raw(point - 0x20)
    }
}

impl Expr {
    fn contains(&self, point: u32, ignore_case: bool) -> bool {
        match self {
            Self::Leaf(leaf) => leaf.contains(point, ignore_case),
            Self::Not(child) => !child.contains(point, ignore_case),
            Self::Union(children) => children
                .iter()
                .any(|child| child.contains(point, ignore_case)),
            Self::Intersection(children) => children
                .iter()
                .all(|child| child.contains(point, ignore_case)),
            Self::Subtraction(children) => {
                children[0].contains(point, ignore_case)
                    && children[1..]
                        .iter()
                        .all(|child| !child.contains(point, ignore_case))
            }
        }
    }
}

#[derive(Clone, Copy)]
struct Gates {
    anchors: [bool; 2],
    words: [Option<bool>; 2],
}

fn expected(
    expression: &Expr,
    gates: Gates,
    flags: [bool; 2],
    input: &Input,
    start: usize,
    sticky: bool,
) -> Option<Range<usize>> {
    let [ignore_case, multiline] = flags;
    if start > input.text.len() {
        return None;
    }
    let first = input.boundaries.partition_point(|&at| at <= start) - 1;
    let word = |point| Leaf::Word.raw(point) || (ignore_case && matches!(point, 0x17f | 0x212a));
    let boundary = |at: usize| {
        let left = at.checked_sub(1).and_then(|at| input.points.get(at));
        let right = input.points.get(at);
        left.is_some_and(|&point| word(point)) != right.is_some_and(|&point| word(point))
    };
    let line = |point| matches!(point, 0xa | 0xd | 0x2028 | 0x2029);
    for at in first..input.points.len() {
        if sticky && at != first {
            break;
        }
        if expression.contains(input.points[at], ignore_case)
            && (!gates.anchors[0] || at == 0 || (multiline && line(input.points[at - 1])))
            && (!gates.anchors[1]
                || at + 1 == input.points.len()
                || (multiline && line(input.points[at + 1])))
            && gates.words[0].is_none_or(|wanted| boundary(at) == wanted)
            && gates.words[1].is_none_or(|wanted| boundary(at + 1) == wanted)
        {
            return Some(input.boundaries[at]..input.boundaries[at + 1]);
        }
    }
    None
}

fn recording(
    source: &JsString,
    ignore_case: bool,
    multiline: bool,
    work: &mut Vec<usize>,
    fail: Option<usize>,
) -> Result<Option<RegExpUnicodeClassMatcher>, RegExpUnicodeSetCompileError<usize>> {
    RegExpUnicodeClassMatcher::compile_v_set_with_single_character_strings_and_work(
        source,
        ignore_case,
        multiline,
        |amount| {
            work.push(amount);
            if fail == Some(work.len()) {
                Err(work.len())
            } else {
                Ok(())
            }
        },
    )
}

fn compile(source: &str, ignore_case: bool, multiline: bool) -> RegExpUnicodeClassMatcher {
    recording(
        &JsString::from(source),
        ignore_case,
        multiline,
        &mut Vec::new(),
        None,
    )
    .unwrap()
    .unwrap()
}

#[test]
fn single_character_string_operands_match_independent_expression_oracle() {
    use Expr::{Intersection as And, Leaf as Set, Not, Subtraction as Minus, Union as Or};
    use Leaf::{Ascii, Digit, Points, Range as Span, Space, Word};
    let cases: &[(&str, Expr)] = &[
        (
            r"[a\q{K|s|K}\d]",
            Or(&[Set(Points(&[0x61])), Set(Points(&[0x4b, 0x73])), Set(Digit)]),
        ),
        (
            r"[\q{K|s}a-z]",
            Or(&[Set(Points(&[0x4b, 0x73])), Set(Span(0x61, 0x7a))]),
        ),
        (
            r"[[^\q{K|s}]&&[A-z]]",
            And(&[Not(&Set(Points(&[0x4b, 0x73]))), Set(Span(0x41, 0x7a))]),
        ),
        (
            r"[\q{K|s}--[k]]",
            Minus(&[Set(Points(&[0x4b, 0x73])), Set(Points(&[0x6b]))]),
        ),
        (
            r"[[k]--\q{K|s}]",
            Minus(&[Set(Points(&[0x6b])), Set(Points(&[0x4b, 0x73]))]),
        ),
        (
            r"[^[[\q{K|s}]&&[a-z]]]",
            Not(&And(&[Set(Points(&[0x4b, 0x73])), Set(Span(0x61, 0x7a))])),
        ),
        (
            r"[[\q{K|s}b][^\q{a|s}]]",
            Or(&[
                Set(Points(&[0x4b, 0x73, 0x62])),
                Not(&Set(Points(&[0x61, 0x73]))),
            ]),
        ),
        (
            r"[[\q{K|s}]&&[A-z]&&[^s]]",
            And(&[
                Set(Points(&[0x4b, 0x73])),
                Set(Span(0x41, 0x7a)),
                Not(&Set(Points(&[0x73]))),
            ]),
        ),
        (
            r"[\q{K|s|a}--[k]--[s]]",
            Minus(&[
                Set(Points(&[0x4b, 0x73, 0x61])),
                Set(Points(&[0x6b])),
                Set(Points(&[0x73])),
            ]),
        ),
        (
            r"[\q{\uD83D\uDE00|\u{D800}}[\u{DC00}]]",
            Set(Points(&[0x1f600, 0xd800, 0xdc00])),
        ),
        (
            r"[\q{😀|𐐀}&&[^\u{10428}]]",
            And(&[
                Set(Points(&[0x1f600, 0x10400])),
                Not(&Set(Points(&[0x10428]))),
            ]),
        ),
        (
            r"[\q{\b|\n|a}&&[\s\b]]",
            And(&[
                Set(Points(&[8, 0xa, 0x61])),
                Or(&[Set(Space), Set(Points(&[8]))]),
            ]),
        ),
        (
            r"[\p{ASCII}&&\q{K|ſ|😀}]",
            And(&[Set(Ascii), Set(Points(&[0x4b, 0x17f, 0x1f600]))]),
        ),
        (
            r"[\q{K|ſ|😀}&&\P{ASCII}]",
            And(&[Set(Points(&[0x4b, 0x17f, 0x1f600])), Not(&Set(Ascii))]),
        ),
        (
            r"[\w--\q{K|s}]",
            Minus(&[Set(Word), Set(Points(&[0x4b, 0x73]))]),
        ),
        (
            r"[\q{K|s|a}&&[^\q{K|s}]]",
            And(&[
                Set(Points(&[0x4b, 0x73, 0x61])),
                Not(&Set(Points(&[0x4b, 0x73]))),
            ]),
        ),
    ];
    let inputs = [
        JsString::from(""),
        JsString::from("aAKkKsſz0_!"),
        JsString::from("\r\nK\u{2028}s\n!"),
        JsString::from("😀𐐀𐐨"),
        JsString::from_code_units(vec![0xd800, 0x61, 0xdc00, 0xd83d, 0xde00]),
        JsString::from("\u{8}\t\n 1!"),
        JsString::from("ßİıΣςσ"),
        JsString::from("😀aK😀"),
    ]
    .map(Input::new);
    let gates = [
        Gates {
            anchors: [false, false],
            words: [None, None],
        },
        Gates {
            anchors: [true, true],
            words: [None, None],
        },
        Gates {
            anchors: [false, false],
            words: [Some(true), Some(true)],
        },
        Gates {
            anchors: [false, false],
            words: [Some(false), Some(false)],
        },
        Gates {
            anchors: [true, false],
            words: [Some(true), None],
        },
        Gates {
            anchors: [false, true],
            words: [None, Some(false)],
        },
        Gates {
            anchors: [true, true],
            words: [Some(false), Some(true)],
        },
        Gates {
            anchors: [false, false],
            words: [None, Some(true)],
        },
    ];
    for (body, expression) in cases {
        for gates in gates {
            let word_gate = |gate| match gate {
                None => "",
                Some(false) => r"\B",
                Some(true) => r"\b",
            };
            let source = format!(
                "{}{}{}{}{}",
                if gates.anchors[0] { "^" } else { "" },
                word_gate(gates.words[0]),
                body,
                word_gate(gates.words[1]),
                if gates.anchors[1] { "$" } else { "" }
            );
            for ignore_case in [false, true] {
                for multiline in [false, true] {
                    let original = compile(&source, ignore_case, multiline);
                    let matcher = original.clone();
                    drop(original);
                    for input in &inputs {
                        for start in 0..=input.text.len() + 1 {
                            for sticky in [false, true] {
                                assert_eq!(
                                    matcher.find(&input.text, start, sticky),
                                    expected(
                                        expression,
                                        gates,
                                        [ignore_case, multiline],
                                        input,
                                        start,
                                        sticky
                                    ),
                                    "{source} i={ignore_case} m={multiline} input={:?} start={start} sticky={sticky}",
                                    input.text
                                );
                            }
                        }
                    }
                }
            }
        }
    }
    // Actual consumers of i and m, and a subtraction that becomes empty after
    // folding: closure must not resurrect an element after the set operation.
    let source = r"^\b[\q{K}&&[k]]\b$";
    let input = JsString::from("\nK\n");
    assert_eq!(
        compile(source, true, true).find(&input, 0, false),
        Some(1..2)
    );
    assert!(
        compile(source, false, true)
            .find(&input, 0, false)
            .is_none()
    );
    assert!(
        compile(source, true, false)
            .find(&input, 0, false)
            .is_none()
    );
    let input = JsString::from("KkK");
    assert_eq!(
        compile(r"[\q{K}--k]", false, false).find(&input, 0, false),
        Some(0..1)
    );
    assert!(
        compile(r"[\q{K}--k]", true, false)
            .find(&input, 0, false)
            .is_none()
    );
}

#[test]
fn single_character_string_operands_domains_and_work_prefixes() {
    // The braced leaf terminator must distinguish escaped } and | characters
    // from the actual close/separator, then resume ordinary class membership.
    let input = JsString::from("|}/a");
    let matcher = compile(r"[\q{\||\}|\/}a]", false, false);
    for start in 0..input.len() {
        assert_eq!(matcher.find(&input, start, true), Some(start..start + 1));
    }
    for (text, supported, ignore_case, multiline) in [
        (r"[\q{a}]", true, false, false),
        (r"[a\q{b|b}c]", true, false, false),
        (r"[[\q{a}]b]", true, false, false),
        (r"[[]&&\q{a}]", true, false, false),
        (r"[\q{a}--a--\q{b}]", true, false, false),
        (r"^\b[^\q{K|s}&&[a-z]]\B$", true, true, true),
        (r"[\q{\||\}|\/}a]", true, false, false),
        (r"[[]&&\q{a|bc}]", false, false, false),
        (r"[\q{a}--a--\q{a|bc}]", false, false, false),
        (r"[\q{a}&&[]&&\q{a|}]", false, false, false),
        (r"[\q{a}--a--\p{RGI_Emoji}]", false, false, false),
        (r"[a\q{\u{D800}\u{DC00}}]", false, false, false),
        (r"[a\q{\uD800\u{DC00}}]", false, false, false),
    ] {
        let source = JsString::from(text);
        let mut complete = Vec::new();
        assert_eq!(
            recording(&source, ignore_case, multiline, &mut complete, None)
                .unwrap()
                .is_some(),
            supported,
            "{text}"
        );
        assert_eq!(complete[0], source.len(), "{text}");
        for fail in 1..=complete.len() {
            let mut aborted = Vec::new();
            assert!(matches!(
                recording(&source, ignore_case, multiline, &mut aborted, Some(fail)),
                Err(RegExpUnicodeSetCompileError::Work(at)) if at == fail
            ));
            assert_eq!(aborted, complete[..fail], "{text} abort={fail}");
            let mut fresh = Vec::new();
            assert_eq!(
                recording(&source, ignore_case, multiline, &mut fresh, None)
                    .unwrap()
                    .is_some(),
                supported
            );
            assert_eq!(fresh, complete, "{text} after abort={fail}");
        }
    }
    for text in [
        "",
        "^",
        "$",
        "^$",
        "a",
        "[a]",
        "[[a]]",
        "[a&&b]",
        "[]",
        "[^]",
        r"\q{a}",
        r"[\q{}]",
        r"[a\q{a|}]",
        r"[a\q{|b}]",
        r"[a\q{a||b}]",
        r"[a\q{bc}]",
        r"[a\q{\d}]",
        r"[a\q{\p{ASCII}}]",
        r"[a\q{\a}]",
        r"[a\q{(}]",
        r"[a\q{/}]",
        r"[a\q{\u{110000}}]",
        r"[a\q{b]",
        r"[\q{a}b",
        r"[\q{a|b]",
        r"[\q{a}&&]",
        r"[&&\q{a}]",
        r"[\q{a}&&&a]",
        r"[\q{a}&&a--b]",
        r"[\q{a}b&&c]",
        r"[a-z&&\q{b}]",
        r"[a-\q{b}]",
        r"[\q{a}-b]",
        r"[\q{a}][b]",
        r"a[\q{b}]",
        r"[\q{a}]b",
        r"[\q{a}]+",
        r"[\q{a}]{2}",
        r"([\q{a}])",
        r"(?<whole>[\q{a}])",
        r"\b^[\q{a}]",
        r"[\q{a}]$\b",
        r"\b\b[\q{a}]",
        r"[\q{a}]\b\B",
        r"[\q{a}\p{Basic_Emoji}]",
        r"[\q{a}\P{RGI_Emoji}]",
    ] {
        let source = JsString::from(text);
        let mut seen = Vec::new();
        assert!(
            recording(&source, true, true, &mut seen, None)
                .unwrap()
                .is_none(),
            "{text}"
        );
        assert_eq!(seen[0], source.len(), "{text}");
        let mut aborted = Vec::new();
        assert!(matches!(
            recording(&source, true, true, &mut aborted, Some(1)),
            Err(RegExpUnicodeSetCompileError::Work(1))
        ));
        assert_eq!(aborted, seen[..1]);
    }
    // q syntax in a nested/composite source does not widen any previous API.
    for text in [
        r"[a\q{b}]",
        r"[\q{a}b]",
        r"[[\q{a}]]",
        r"[\q{a}&&[a]]",
        r"[[a]--\q{b}]",
        r"^\b[^\q{K|s}&&[a-z]]\B$",
    ] {
        let source = JsString::from(text);
        let accept: fn(usize) -> Result<(), usize> = |_| Ok(());
        assert!(
            RegExpUnicodeClassMatcher::compile_with_flags_and_work(
                &source, true, true, true, accept
            )
            .unwrap()
            .is_none(),
            "{text}"
        );
        assert!(
            RegExpUnicodeClassMatcher::compile_with_word_boundaries_and_work(
                &source, true, true, true, accept
            )
            .unwrap()
            .is_none(),
            "{text}"
        );
        assert!(
            RegExpUnicodeClassMatcher::compile_v_set_with_work(&source, true, true, accept)
                .unwrap()
                .is_none(),
            "{text}"
        );
        assert!(
            RegExpUnicodeClassMatcher::compile_v_set_with_word_boundaries_and_work(
                &source, true, true, accept
            )
            .unwrap()
            .is_none(),
            "{text}"
        );
        let mut seen = Vec::new();
        assert!(
            RegExpUnicodeClassMatcher::compile_single_character_strings_v_set_with_work(
                &source,
                true,
                true,
                |amount| {
                    seen.push(amount);
                    Ok::<(), usize>(())
                }
            )
            .unwrap()
            .is_none(),
            "{text}"
        );
        assert_eq!(seen, [source.len() * 3], "{text}");
    }
}
