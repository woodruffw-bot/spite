//! Complete outer noncapturing groups preserve their body's matcher (22.2.2.3).

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
    let mut remaining = opening_end / 3;
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
    Some(complete * 3..units.len() - complete)
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
    fn paired_group_oracle(source: &JsString) -> Option<Range<usize>> {
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
        while units[body.clone()].starts_with(&[40, 63, 58])
            && pairs.contains(&(body.start, body.end - 1))
        {
            body.start += 3;
            body.end -= 1;
        }
        (body.start != 0).then_some(body)
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
                                paired_group_oracle(&source),
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
}
