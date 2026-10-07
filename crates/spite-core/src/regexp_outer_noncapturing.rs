//! Complete outer ordinary groups preserve their body's matcher (22.2.2.3).

use crate::JsString;
use std::ops::Range;

/// Returns the body of all complete ordinary outer `(?:...)` wrappers.
///
/// The Pattern must already be validated without `u` or `v`. Only whole,
/// unquantified wrappers are removed; a group followed by a continuation or
/// quantifier remains in the returned body. Escapes and bracket classes are
/// opaque to delimiter scanning. The two scans use constant space and no native
/// recursion. Returned offsets borrow the original UTF-16 source.
pub fn regexp_outer_noncapturing_body(source: &JsString) -> Option<Range<usize>> {
    let units = source.code_units();
    let mut opening_end = 0usize;
    while units.get(opening_end..)?.starts_with(&[0x28, 0x3f, 0x3a]) {
        opening_end += 3;
    }
    let remaining = opening_end / 3;
    if remaining == 0 {
        return None;
    }
    let complete = complete_outer_count(units, remaining)?;
    Some(complete * 3..units.len() - complete)
}

/// A complete ordinary outer-group body and its source-order whole-match captures.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RegExpOuterGroupBody {
    /// Borrowed UTF-16 body after removing all enclosing ordinary groups.
    pub body: Range<usize>,
    /// Capturing wrappers preceding every capture in the retained body.
    pub captures: usize,
}

/// Removes complete ordinary capturing/noncapturing outer groups (22.2.2.3).
///
/// The Pattern must already be validated without `u` or `v`. Quantified groups,
/// groups followed by continuations and assertion/named/modifier groups remain
/// inside the body. Three linear scans use constant space and no native recursion.
pub fn regexp_outer_group_body(source: &JsString) -> Option<RegExpOuterGroupBody> {
    let units = source.code_units();
    let mut opening_end = 0;
    let mut leading = 0;
    while let Some(width) = ordinary_opener_width(units, opening_end) {
        opening_end += width;
        leading += 1;
    }
    let complete = complete_outer_count(units, leading)?;
    let mut body_start = 0;
    let mut captures = 0;
    for _ in 0..complete {
        let width = ordinary_opener_width(units, body_start)?;
        captures += usize::from(width == 1);
        body_start += width;
    }
    Some(RegExpOuterGroupBody {
        body: body_start..units.len() - complete,
        captures,
    })
}

fn ordinary_opener_width(units: &[u16], start: usize) -> Option<usize> {
    let suffix = units.get(start..)?;
    if suffix.starts_with(&[40, 63, 58]) {
        Some(3)
    } else if suffix.first() == Some(&40) && suffix.get(1) != Some(&63) {
        Some(1)
    } else {
        None
    }
}

fn complete_outer_count(units: &[u16], mut remaining: usize) -> Option<usize> {
    if remaining == 0 {
        return None;
    }
    let mut depth = 0usize;
    let mut complete = 0usize;
    let mut in_class = false;
    let mut index = 0usize;
    while let Some(&unit) = units.get(index) {
        index += 1;
        if unit == 0x5c {
            units.get(index)?;
            index += 1;
            continue;
        }
        if in_class {
            in_class = unit != 0x5d;
            continue;
        }
        match unit {
            0x5b => in_class = true,
            0x28 => depth = depth.checked_add(1)?,
            0x29 => {
                depth = depth.checked_sub(1)?;
                // Only the first closure at each leading wrapper's depth belongs
                // to that wrapper. Later sibling groups must not replace it.
                if depth < remaining {
                    remaining = depth;
                    if index == units.len().checked_sub(depth)? {
                        complete = complete.max(depth + 1);
                    }
                }
            }
            _ => {}
        }
    }
    if depth != 0 || in_class || complete == 0 {
        return None;
    }
    Some(complete)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn complete_outer_group_snapshot() {
        let mut rows = String::new();
        for source in [
            "",
            "a",
            "(?:)",
            "(?:(?:))",
            "(?:a+b)",
            "(?:(?:a+b))",
            "(?:a+|b)",
            "(?:(?:a|b))",
            "(?:(?:a)|(?:b))",
            "(?:(?:a)(?:b))",
            "(?:(?:a)b)",
            "(?:(?:a)(b))",
            "(?:(a+)(b))",
            "(?:^a+$)",
            "(?:(?:^a+$))",
            "(?:a)+(?:b)",
            "(?:a)+",
            "(?:a|b)+",
            "(?:(?:a|b)+)",
            "(?:a)b",
            "(?:(?:a))b",
            "(?:a)|(?:b)",
            "(?:[()|])",
            "(?:[(])",
            "(?:[)])",
            "(?:[[])",
            "(?:[])",
            "(?:[^])",
            r"(?:[\]])",
            r"(?:\(\))",
            r"(?:a\))",
            r"(?:a\\)",
            r"(?:(?:\(|\)))",
            "(?:µ)",
            "(?:(?:😀))",
            "(?:a(?=b))",
            "(?:(?<x>a))",
            "(?:(?i:a))",
            "(?:a$|^b)",
        ] {
            let body = regexp_outer_noncapturing_body(&JsString::from(source));
            rows.push_str(&format!("{source:?} => {body:?}\n"));
        }
        let source = JsString::from_code_units(vec![40, 63, 58, 0xd800, 41]);
        rows.push_str(&format!(
            "lone surrogate => {:?}\n",
            regexp_outer_noncapturing_body(&source)
        ));
        insta::assert_snapshot!(rows);
    }

    // Store every group boundary independently, then peel enclosing pairs.
    fn paired_group_oracle(source: &JsString, capturing: bool) -> Option<RegExpOuterGroupBody> {
        let units = source.code_units();
        let mut stack = Vec::new();
        let mut pairs = Vec::new();
        let mut class = false;
        let mut index = 0;
        while index < units.len() {
            match units[index] {
                92 => index += 1,
                93 if class => class = false,
                _ if class => {}
                91 => class = true,
                40 => stack.push(index),
                41 => pairs.push((stack.pop().unwrap(), index)),
                _ => {}
            }
            index += 1;
        }
        assert!(stack.is_empty());
        let mut body = 0..units.len();
        let mut captures = 0;
        loop {
            let suffix = &units[body.clone()];
            let width = if suffix.starts_with(&[40, 63, 58]) {
                3
            } else if capturing && suffix.first() == Some(&40) && suffix.get(1) != Some(&63) {
                1
            } else {
                break;
            };
            if !pairs.contains(&(body.start, body.end - 1)) {
                break;
            }
            captures += usize::from(width == 1);
            body.start += width;
            body.end -= 1;
        }
        (body.start != 0).then_some(RegExpOuterGroupBody { body, captures })
    }

    #[test]
    fn wrapper_ranges_agree_with_independent_group_pairing() {
        for left in ["", "a", "(?:a)", "(?:a+)", "(a)", "[()]", r"\("] {
            for right in ["", "b", "(?:b)", "(?:b*)", "(b)", "[[]", r"\)"] {
                for separator in ["", "|"] {
                    for depth in 0..=4 {
                        for continuation in ["", "x", "(?:x)", "|x", "*", "{2}"] {
                            let source = JsString::from(&*format!(
                                "{}(?:{left}{separator}{right}){}{continuation}",
                                "(?:".repeat(depth),
                                ")".repeat(depth)
                            ));
                            assert_eq!(
                                regexp_outer_noncapturing_body(&source),
                                paired_group_oracle(&source, false).map(|v| v.body),
                                "{source:?}"
                            );
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn deeply_nested_wrappers_use_constant_space_and_keep_partial_inner_groups() {
        let source = JsString::from(&*format!(
            "{}a+b{}",
            "(?:".repeat(100_000),
            ")".repeat(100_000)
        ));
        assert_eq!(
            regexp_outer_noncapturing_body(&source),
            Some(300_000..300_003)
        );
        let source = JsString::from(&*format!(
            "{}(?:a)(?:b){}",
            "(?:".repeat(100_000),
            ")".repeat(100_000)
        ));
        assert_eq!(
            regexp_outer_noncapturing_body(&source),
            Some(300_000..300_010)
        );
        for source in ["(?:a", "(?:a))", "(?:[a)", "(?:a\\"] {
            assert_eq!(
                regexp_outer_noncapturing_body(&JsString::from(source)),
                None
            );
        }
    }
    #[test]
    fn enclosing_capture_body_snapshot() {
        let mut rows = String::new();
        for source in [
            "",
            "a",
            "()",
            "(())",
            "((?:))",
            "(?:(()))",
            "(a+b)",
            "((a+b))",
            "(?:(a+b))",
            "((?:a+b))",
            "(a+|b)",
            "((x)|(a)+(b)|(y))",
            "(a$|^b)",
            "(^a+$)",
            "((a)*())",
            "((a*)())",
            "((a)*)",
            "((a)+)",
            "((?:a)*)",
            "(?:(a)*)",
            "(a)*",
            "(a)+",
            "(a+b)*",
            "((a|b)+)",
            "(a+b)c",
            "((a+b))c",
            "(a)|(b)",
            "((a)(b))",
            "((?:a)(?:b))",
            "(?:(a)(b))",
            "([()|])",
            "([[])",
            r"(\(\))",
            "(µ)",
            "((😀))",
            "(?<x>a)",
            "(?=a)",
            "(?!a)",
            "(?<=a)",
            "(?i:a)",
            "((?<x>a))",
            "((?=a))",
        ] {
            rows.push_str(&format!(
                "{source:?} => {:?}\n",
                regexp_outer_group_body(&JsString::from(source))
            ));
        }
        insta::assert_snapshot!(rows);
    }

    #[test]
    fn enclosing_capture_ranges_and_counts_agree_with_independent_pairs() {
        for left in ["", "a", "(a)", "(?:a+)", "[()]", r"\("] {
            for right in ["", "b", "(b)", "(?:b*)", "[[]", r"\)"] {
                for separator in ["", "|"] {
                    for depth in 0..=4 {
                        for mixed in [false, true] {
                            let mut text = format!("({left}{separator}{right})");
                            for level in 0..depth {
                                text = if mixed && level % 2 == 0 {
                                    format!("(?:{text})")
                                } else {
                                    format!("({text})")
                                };
                            }
                            for continuation in ["", "x", "()", "|x", "*", "{2}"] {
                                let source = JsString::from((text.clone() + continuation).as_str());
                                assert_eq!(
                                    regexp_outer_group_body(&source),
                                    paired_group_oracle(&source, true),
                                    "{source:?}"
                                );
                            }
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn deeply_mixed_capturing_wrappers_keep_whole_match_prefix_counts() {
        let text = format!("{}a+b{}", "((?:".repeat(50_000), "))".repeat(50_000));
        assert_eq!(
            regexp_outer_group_body(&JsString::from(text.as_str())),
            Some(RegExpOuterGroupBody {
                body: 200_000..200_003,
                captures: 50_000
            })
        );
        let source = JsString::from("((a)(b))");
        assert_eq!(
            regexp_outer_group_body(&source),
            Some(RegExpOuterGroupBody {
                body: 1..7,
                captures: 1
            })
        );
        for source in ["(a", "(a))", "([a)", "(a\\"] {
            assert_eq!(regexp_outer_group_body(&JsString::from(source)), None);
        }
    }
}
