//! Fixtures for Unicode-set matches with sequentially named capture groups.

use super::{assert_true, assert_value};
use spite_core::JsString;
use spite_runtime::{Realm, Value};
use std::ops::Range;

/// Checks a failed global, indexed Unicode-set match and its lastIndex reset.
#[track_caller]
pub fn assert_no_match(pattern: &str, flags: &str, input: JsString, initial_last_index: usize) {
    let pattern = JsString::from(pattern);
    let flags = JsString::from(flags);
    assert_true(&format!(
        r#"
            var pattern = {pattern:?},
                input = {input:?},
                regexp = new RegExp(pattern, {flags:?}),
                metadataValid = regexp.source === pattern && regexp.global &&
                    regexp.hasIndices && regexp.unicodeSets === true && regexp.unicode === false;
            regexp.lastIndex = {initial_last_index};
            var result = regexp.exec(input);
            metadataValid && result === null && regexp.lastIndex === 0
        "#
    ));
}

/// Expected match text, capture slots, UTF-16 ranges, and final lastIndex.
pub struct MatchExpectation<'a> {
    pub text: &'a str,
    pub range: Range<usize>,
    pub captures: &'a [Option<&'a str>],
    pub capture_ranges: &'a [Option<(usize, usize)>],
    pub last_index: usize,
}

/// Checks match contents and the identity of named and numbered capture aliases.
#[track_caller]
pub fn assert_named_captures(
    pattern: &str,
    flags: &str,
    input: JsString,
    initial_last_index: usize,
    expected: MatchExpectation<'_>,
) {
    let pattern = JsString::from(pattern);
    let capture_count = expected.captures.len();
    assert_eq!(expected.capture_ranges.len(), capture_count);

    let mut realm = Realm::default();
    realm
        .eval(&format!(
            r#"
                const pattern = {pattern:?};
                const input = {input:?};
                const regexp = new RegExp(pattern, "{flags}");
                if (regexp.source !== pattern || !regexp.global || !regexp.hasIndices ||
                    regexp.unicodeSets !== true || regexp.unicode !== false) {{
                    throw new Error("RegExp source or flags changed");
                }}

                regexp.lastIndex = {initial_last_index};
                const match = regexp.exec(input);
                if (match === null) throw new Error("expected a match");
                if (Object.getPrototypeOf(match.groups) !== null || match.input !== input) {{
                    throw new Error("incorrect groups prototype or match input");
                }}

                for (let slot = 1; slot <= {capture_count}; slot++) {{
                    const name = "n" + slot;
                    if (!Object.hasOwn(match.groups, name) ||
                        !Object.hasOwn(match.indices.groups, name) ||
                        match.groups[name] !== match[slot] ||
                        match.indices.groups[name] !== match.indices[slot]) {{
                        throw new Error("named capture alias differs at " + name);
                    }}
                }}
            "#
        ))
        .unwrap_or_else(|error| panic!("pattern={pattern:?} flags={flags}: {error:?}"));

    let length = Value::Number((capture_count + 1) as f64);
    assert_value(&mut realm, "match.length", length.clone());
    assert_value(&mut realm, "match.indices.length", length);
    assert_value(&mut realm, "match[0]", Value::String(expected.text.into()));
    assert_value(
        &mut realm,
        "match.index",
        Value::Number(expected.range.start as f64),
    );

    for (offset, capture) in expected.captures.iter().enumerate() {
        let value = capture.map_or(Value::Undefined, |text| Value::String(text.into()));
        assert_value(&mut realm, &format!("match[{}]", offset + 1), value);
    }

    let whole_match = Some((expected.range.start, expected.range.end));
    for (slot, range) in std::iter::once(&whole_match)
        .chain(expected.capture_ranges)
        .enumerate()
    {
        let expression = format!("match.indices[{slot}]");
        if let Some((start, end)) = range {
            assert_value(
                &mut realm,
                &format!("{expression}.length"),
                Value::Number(2.0),
            );
            assert_value(
                &mut realm,
                &format!("{expression}[0]"),
                Value::Number(*start as f64),
            );
            assert_value(
                &mut realm,
                &format!("{expression}[1]"),
                Value::Number(*end as f64),
            );
        } else {
            assert_value(&mut realm, &expression, Value::Undefined);
        }
    }
    assert_value(
        &mut realm,
        "regexp.lastIndex",
        Value::Number(expected.last_index as f64),
    );
}
