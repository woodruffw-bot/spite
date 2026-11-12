//! Required predicate choices preserve conditional lexical captures.
use crate::common::assert_true;
use crate::common::eval_string;
use crate::common::regexp_captures::assert_no_match;
use crate::common::regexp_captures::{MatchExpectation, assert_named_captures};
use crate::common::regexp_matches::DESCRIBE_MATCH;
use spite_core::JsString;
use spite_runtime::{Realm, Value};
use test_case::test_case;
#[test]
fn matching_controls_and_string_consumers() {
    let mut realm = Realm::default();
    realm.eval(r"var r=/(?<enclosing>(?:(?<outer>(?<b>(?<s>^)|(?<w>\b))(?<c>[\q{a|ab}])(?<a>(?<end>$)|(?<nw>\B))|(?<fallback>(?<wb>\b)|(?<nwb>\B))(?<empty>))?))/dvg,clone=new RegExp(r),m=r.exec('ab!aa')").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval(r#"
        var n = clone.exec('ab'),
            p = r.exec('!');
        m[0] === 'a' && m.groups.enclosing === 'a' && m.groups.outer === 'a' && m.groups.s === '' && m
            .groups.w === undefined && m.groups.c === 'a' && m.groups.fallback === undefined && m
            .indices.groups.c === m.indices[6] && m.indices[6][0] === 0 && m.indices[6][1] === 1 && n
            .groups.c === 'ab' && n.groups.enclosing === 'ab' && clone.source === r.source && clone
            .lastIndex === 2 && p[0] === '' && p.index === 1 && p.groups.enclosing === '' && p.indices
            .groups.enclosing === p.indices[1] && p.indices[1][0] === 1 && p.indices[1][1] === 1 &&
            Array.from(p).slice(2).every(v => v === undefined) && p.indices.slice(2).every(v => v ===
                undefined) && p.groups.outer === undefined && p.groups.fallback === undefined && p
            .groups.empty === undefined && r.lastIndex === 1
    "#),Ok(Value::Boolean(true)));
}

#[test]
fn match_all_preserves_capture_metadata() {
    let source = format!(
        r#"
            (function() {{
                {DESCRIBE_MATCH}

                var P =
                    "(?<enclosing>(?:(?<outer>(?<b>(?<s>^)|(?<w>\\b))(?<whole>(?<c>[\\q{{|a|ab}}]))(?<a>(?<end>$)|(?<nw>\\B))|(?<fallback>(?<wb>\\b)|(?<nwb>\\B))(?<empty>))?))";
                var r = new RegExp(P, "dvg"),
                    a = Array.from("ab!aa".matchAll(r));
                return JSON.stringify([a.map(describeMatch), r.lastIndex], null, 2);
            }})()
        "#
    );
    let report = eval_string(&mut Realm::default(), &source);
    insta::assert_snapshot!(report);
}

#[test]
fn replacement_callback_receives_captures() {
    let source = r#"
        (function () {
          var P =
            "(?<enclosing>(?:(?<outer>(?<b>(?<s>^)|(?<w>\\b))(?<whole>(?<c>[\\q{|a|ab}]))(?<a>(?<end>$)|(?<nw>\\B))|(?<fallback>(?<wb>\\b)|(?<nwb>\\B))(?<empty>))?))";
          var calls = [],
            r = new RegExp(P, "vg"),
            out = "ab!aa".replace(r, function () {
              var a = Array.from(arguments);
              calls.push(
                a.map((v) =>
                  v === undefined
                    ? "<undefined>"
                    : typeof v === "object"
                      ? Object.keys(v).map((k) => [k, v[k] === undefined ? "<undefined>" : v[k]])
                      : v,
                ),
              );
              return "[" + a[0] + "]";
            });
          return JSON.stringify([out, calls, r.lastIndex], null, 2);
        })();
    "#;
    let report = eval_string(&mut Realm::default(), source);
    insta::assert_snapshot!(report);
}

#[test]
fn split_and_search_preserve_last_index() {
    let source = r#"
        (function() {

            var P =
                "(?<enclosing>(?:(?<outer>(?<b>(?<s>^)|(?<w>\\b))(?<whole>(?<c>[\\q{|a|ab}]))(?<a>(?<end>$)|(?<nw>\\B))|(?<fallback>(?<wb>\\b)|(?<nwb>\\B))(?<empty>))?))";
            var r = new RegExp(P, "dvg");
            r.lastIndex = 9;
            var a = "!abaa!".split(r),
                b = "!abaa!".search(r);
            return JSON.stringify([a.map(v => v === undefined ? "<undefined>" : v), b, r
                .lastIndex
            ], null, 2);
        })()
    "#;
    let report = eval_string(&mut Realm::default(), source);
    insta::assert_snapshot!(report, @r#"
        [
          [
            "!",
            "ab",
            "ab",
            "",
            "<undefined>",
            "",
            "ab",
            "ab",
            "",
            "<undefined>",
            "",
            "<undefined>",
            "<undefined>",
            "<undefined>",
            "<undefined>",
            "a",
            "",
            "<undefined>",
            "<undefined>",
            "<undefined>",
            "<undefined>",
            "<undefined>",
            "<undefined>",
            "<undefined>",
            "<undefined>",
            "<undefined>",
            "<undefined>",
            "<undefined>",
            "<undefined>",
            "<undefined>",
            "a",
            "",
            "<undefined>",
            "<undefined>",
            "<undefined>",
            "<undefined>",
            "<undefined>",
            "<undefined>",
            "<undefined>",
            "<undefined>",
            "<undefined>",
            "<undefined>",
            "<undefined>",
            "<undefined>",
            "<undefined>",
            "!"
          ],
          0,
          9
        ]
        "#);
}

#[test]
fn failed_last_index_write_throws_type_error() {
    let source = r#"
        (function () {
          var P =
            "(?<enclosing>(?:(?<outer>(?<b>(?<s>^)|(?<w>\\b))(?<whole>(?<c>[\\q{|a|ab}]))(?<a>(?<end>$)|(?<nw>\\B))|(?<fallback>(?<wb>\\b)|(?<nwb>\\B))(?<empty>))?))";
          var r = new RegExp(P, "vg");
          Object.defineProperty(r, "lastIndex", {
            writable: false,
          });
          var caught = false;
          try {
            r.exec("ab");
          } catch (e) {
            caught = e instanceof TypeError;
          }
          return JSON.stringify([caught, r.lastIndex], null, 2);
        })();
    "#;
    let report = eval_string(&mut Realm::default(), source);
    insta::assert_snapshot!(report, @r#"
        [
          true,
          0
        ]
        "#);
}

#[test]
fn exec_preserves_alternative_capture_metadata() {
    let source = format!(
        r#"
            (function() {{
                {DESCRIBE_MATCH}

                var P =
                    "(?<enclosing>(?:(?<outer>(?<b>(?<s>^)|(?<w>\\b))(?<whole>(?<c>[\\q{{|a|ab}}]))(?<a>(?<end>$)|(?<nw>\\B))|(?<fallback>(?<wb>\\b)|(?<nwb>\\B))(?<empty>))?))";
                var r = new RegExp(P, "dvg"),
                    a = r.exec(""),
                    b = r.exec("ab");
                return JSON.stringify([describeMatch(a), describeMatch(b), r.lastIndex], null, 2);
            }})()
        "#
    );
    let report = eval_string(&mut Realm::default(), &source);
    insta::assert_snapshot!(report);
}

#[test]
fn ignore_case_preserves_capture_metadata() {
    let source = format!(
        r#"
            (function() {{
                {DESCRIBE_MATCH}

                var P =
                    "(?<enclosing>(?:(?<outer>(?<b>(?<s>^)|(?<w>\\b))(?<whole>(?<c>[a-z\\q{{|S|sa|aX}}]))(?<a>(?<end>$)|(?<nw>\\B))|(?<fallback>(?<wb>\\b)|(?<nwb>\\B))(?<empty>))?))";
                var r = new RegExp(P, "dvgi"),
                    a = r.exec("ſaX!");
                return JSON.stringify([describeMatch(a), r.lastIndex], null, 2);
            }})()
        "#
    );
    let report = eval_string(&mut Realm::default(), &source);
    insta::assert_snapshot!(report);
}

#[test]
fn multiline_match_all_preserves_capture_metadata() {
    let source = format!(
        r#"
            (function() {{
                {DESCRIBE_MATCH}

                var P =
                    "(?<enclosing>(?:(?<outer>(?<b>(?<s>^)|(?<w>\\b))(?<whole>(?<c>[\\q{{|a|ab}}]))(?<a>(?<end>$)|(?<nw>\\B))|(?<fallback>(?<wb>\\b)|(?<nwb>\\B))(?<empty>))?))";
                var r = new RegExp(P, "dvgm"),
                    a = Array.from("ab\n\naa".matchAll(r));
                return JSON.stringify([a.map(describeMatch), r.lastIndex], null, 2);
            }})()
        "#
    );
    let report = eval_string(&mut Realm::default(), &source);
    insta::assert_snapshot!(report);
}

#[test]
fn supplementary_capture_names_preserve_utf16_ranges() {
    let source = format!(
        r#"
            (function() {{
                {DESCRIBE_MATCH}

                var P =
                    "(?<enclosing>(?:(?<outer>(?<\\u{{1D49C}}>(?<s>^)|(?<w>\\b))(?<whole>(?<c>[\\q{{|a|ab}}]))(?<a>(?<end>$)|(?<nw>\\B))|(?<fallback>(?<wb>\\b)|(?<nwb>\\B))(?<empty>))?))";
                var r = new RegExp(P, "dvg"),
                    s = new RegExp(r);
                r.lastIndex = 3;
                var a = s.exec("ab!");
                return JSON.stringify([describeMatch(a), s.source === r.source, s.flags === r.flags, s
                    .lastIndex,
                    r.lastIndex
                ], null, 2);
            }})()
        "#
    );
    let report = eval_string(&mut Realm::default(), &source);
    insta::assert_snapshot!(report);
}

#[test_case(
    r#"(?<n1>(?:(?<n2>(?<n3>(?<n4>^)|(?<n5>\b))(?<n6>[\q{a|ab}])(?<n7>(?<n8>\b^)|(?<n9>$))|(?<n10>(?<n11>^)|(?<n12>\B))(?<n13>(?<n14>){2,3})(?<n15>(?<n16>$)|(?<n17>\B)))?))"#,
    "dgvym",
    JsString::from(""),
    0,
    MatchExpectation {
        text: "",
        range: 0..0,
        captures: &[
            Some(""),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        ],
        capture_ranges: &[
            Some((0, 0)),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        ],
        last_index: 0,
    };
    "empty_input_from_0_variant_1"
)]
#[test_case(
    r#"(?<n1>(?:(?:(?<n2>(?<n3>^)|(?<n4>\B))(?<n5>(?<n6>){2,3})(?<n7>(?<n8>$)|(?<n9>\B))|(?<n10>(?<n11>^)|(?<n12>\b))(?<n13>(?<n14>[\q{|a|ab}])){1}(?<n15>(?<n16>\b^)|(?<n17>$)))??))"#,
    "dgvi",
    JsString::from("ab"),
    1,
    MatchExpectation {
        text: "",
        range: 1..1,
        captures: &[
            Some(""),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        ],
        capture_ranges: &[
            Some((1, 1)),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        ],
        last_index: 1,
    };
    "input_ab_from_1_variant_2"
)]
#[test_case(
    r#"(?<n1>(?:(?<n2>(?<n3>(?<n4>^)|(?<n5>\b))(?<n6>[\p{Any}\q{|ab}]){0001,0001}?(?<n7>(?<n8>\b^)|(?<n9>$))|(?<n10>(?<n11>^)|(?<n12>\B))(?<n13>(?<n14>){2,3})(?<n15>(?<n16>$)|(?<n17>\B))){0,1}))"#,
    "dgv",
    JsString::from("a\n!"),
    0,
    MatchExpectation {
        text: "",
        range: 0..0,
        captures: &[
            Some(""),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        ],
        capture_ranges: &[
            Some((0, 0)),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        ],
        last_index: 0,
    };
    "input_a_from_0_variant_3"
)]
#[test_case(
    r#"(?<n1>(?:(?:(?<n2>(?<n3>^)|(?<n4>\B))(?<n5>(?<n6>){2,3})(?<n7>(?<n8>$)|(?<n9>\B))|(?<n10>(?<n11>^)|(?<n12>\b))(?<n13>(?<n14>[a-z\q{ab}]){1}){1,1}?(?<n15>(?<n16>\b^)|(?<n17>$))){0000,0001}?))"#,
    "dgvim",
    JsString::from("😀a"),
    1,
    MatchExpectation {
        text: "",
        range: 0..0,
        captures: &[
            Some(""),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        ],
        capture_ranges: &[
            Some((0, 0)),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        ],
        last_index: 0,
    };
    "unicode_input_from_1_variant_4"
)]
#[test_case(
    r#"(?<n1>(?:(?<n2>(?<n3>(?<n4>$)|(?<n5>\b)){2,3}?(?<n6>[\q{a|ab}])(?<n7>(?<n8>\B^)|(?<n9>\b$))|(?<n10>(?<n11>(?<n12>^)|(?<n13>\b))\B)(?<n14>(?<n15>){2,3})(?<n16>(?<n17>\b^)|(?<n18>\B$)))?))"#,
    "dgvi",
    JsString::from("a"),
    1,
    MatchExpectation {
        text: "",
        range: 1..1,
        captures: &[
            Some(""),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        ],
        capture_ranges: &[
            Some((1, 1)),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        ],
        last_index: 1,
    };
    "input_a_from_1_variant_5"
)]
#[test_case(
    r#"(?<n1>(?:(?:(?<n2>(?<n3>(?<n4>^)|(?<n5>\b))\B)(?<n6>(?<n7>){2,3})(?<n8>(?<n9>\b^)|(?<n10>\B$))|(?<n11>(?<n12>$)|(?<n13>\b)){2,3}?(?<n14>(?<n15>[\q{|a|ab}])){1}(?<n16>(?<n17>\B^)|(?<n18>\b$)))??))"#,
    "dgv",
    JsString::from("!ab!"),
    0,
    MatchExpectation {
        text: "",
        range: 0..0,
        captures: &[
            Some(""),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        ],
        capture_ranges: &[
            Some((0, 0)),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        ],
        last_index: 0,
    };
    "input_ab_from_0_variant_6"
)]
#[test_case(
    r#"(?<n1>(?:(?<n2>(?<n3>(?<n4>$)|(?<n5>\b)){2,3}?(?<n6>[\p{Any}\q{|ab}]){0001,0001}?(?<n7>(?<n8>\B^)|(?<n9>\b$))|(?<n10>(?<n11>(?<n12>^)|(?<n13>\b))\B)(?<n14>(?<n15>){2,3})(?<n16>(?<n17>\b^)|(?<n18>\B$))){0,1}))"#,
    "dgvim",
    JsString::from("ſa"),
    1,
    MatchExpectation {
        text: "",
        range: 1..1,
        captures: &[
            Some(""),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        ],
        capture_ranges: &[
            Some((1, 1)),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        ],
        last_index: 1,
    };
    "unicode_input_from_1_variant_7"
)]
#[test_case(
    r#"(?<n1>(?:(?:(?<n2>(?<n3>(?<n4>^)|(?<n5>\b))\B)(?<n6>(?<n7>){2,3})(?<n8>(?<n9>\b^)|(?<n10>\B$))|(?<n11>(?<n12>$)|(?<n13>\b)){2,3}?(?<n14>(?<n15>[a-z\q{ab}]){1}){1,1}?(?<n16>(?<n17>\B^)|(?<n18>\b$))){0000,0001}?))"#,
    "dgvy",
    JsString::from("ab\r\n"),
    0,
    MatchExpectation {
        text: "",
        range: 0..0,
        captures: &[
            Some(""),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        ],
        capture_ranges: &[
            Some((0, 0)),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        ],
        last_index: 0,
    };
    "input_ab_from_0_variant_8"
)]
#[test_case(
    r#"(?<n1>(?:(?<n2>(?<n3>(?<n4>^)|(?<n5>$)){3}(?<n6>[\q{a|ab}])(?<n7>(?<n8>^)|(?<n9>\B))|(?<n10>(?<n11>$)|(?<n12>\B))(?<n13>(?<n14>){2,3})(?<n15>(?<n16>\b\B)|(?<n17>\b\B)))?))"#,
    "dgv",
    JsString::from("ab"),
    0,
    MatchExpectation {
        text: "a",
        range: 0..1,
        captures: &[
            Some("a"),
            Some("a"),
            Some(""),
            Some(""),
            None,
            Some("a"),
            Some(""),
            None,
            Some(""),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        ],
        capture_ranges: &[
            Some((0, 1)),
            Some((0, 1)),
            Some((0, 0)),
            Some((0, 0)),
            None,
            Some((0, 1)),
            Some((1, 1)),
            None,
            Some((1, 1)),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        ],
        last_index: 1,
    };
    "input_ab_from_0_variant_9"
)]
#[test_case(
    r#"(?<n1>(?:(?:(?<n2>(?<n3>$)|(?<n4>\B))(?<n5>(?<n6>){2,3})(?<n7>(?<n8>\b\B)|(?<n9>\b\B))|(?<n10>(?<n11>^)|(?<n12>$)){3}(?<n13>(?<n14>[\q{|a|ab}])){1}(?<n15>(?<n16>^)|(?<n17>\B)))??))"#,
    "dgvim",
    JsString::from("a\n!"),
    1,
    MatchExpectation {
        text: "",
        range: 1..1,
        captures: &[
            Some(""),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        ],
        capture_ranges: &[
            Some((1, 1)),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        ],
        last_index: 1,
    };
    "input_a_from_1_variant_10"
)]
#[test_case(
    r#"(?<n1>(?:(?<n2>(?<n3>(?<n4>^)|(?<n5>$)){3}(?<n6>[\p{Any}\q{|ab}]){0001,0001}?(?<n7>(?<n8>^)|(?<n9>\B))|(?<n10>(?<n11>$)|(?<n12>\B))(?<n13>(?<n14>){2,3})(?<n15>(?<n16>\b\B)|(?<n17>\b\B))){0,1}))"#,
    "dgvy",
    JsString::from("😀a"),
    0,
    MatchExpectation {
        text: "",
        range: 0..0,
        captures: &[
            Some(""),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        ],
        capture_ranges: &[
            Some((0, 0)),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        ],
        last_index: 0,
    };
    "unicode_input_from_0_variant_11"
)]
#[test_case(
    r#"(?<n1>(?:(?<n2>(?<n3>(?<n4>\b^)|(?<n5>$))(?<n6>[\q{a|ab}])(?<n7>(?<n8>(?<n9>^)|(?<n10>\b))\B)|(?<n11>(?<n12>\b^)|(?<n13>\B$))(?<n14>(?<n15>){2,3})(?<n16>(?<n17>^)|(?<n18>\b)))?))"#,
    "dgvim",
    JsString::from("!ab!"),
    1,
    MatchExpectation {
        text: "",
        range: 1..1,
        captures: &[
            Some(""),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        ],
        capture_ranges: &[
            Some((1, 1)),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        ],
        last_index: 1,
    };
    "input_ab_from_1_variant_12"
)]
#[test_case(
    r#"(?<n1>(?:(?:(?<n2>(?<n3>\b^)|(?<n4>\B$))(?<n5>(?<n6>){2,3})(?<n7>(?<n8>^)|(?<n9>\b))|(?<n10>(?<n11>\b^)|(?<n12>$))(?<n13>(?<n14>[\q{|a|ab}])){1}(?<n15>(?<n16>(?<n17>^)|(?<n18>\b))\B))??))"#,
    "dgvy",
    JsString::from("ſa"),
    0,
    MatchExpectation {
        text: "",
        range: 0..0,
        captures: &[
            Some(""),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        ],
        capture_ranges: &[
            Some((0, 0)),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        ],
        last_index: 0,
    };
    "unicode_input_from_0_variant_13"
)]
#[test_case(
    r#"(?<n1>(?:(?<n2>(?<n3>(?<n4>\b^)|(?<n5>$))(?<n6>[\p{Any}\q{|ab}]){0001,0001}?(?<n7>(?<n8>(?<n9>^)|(?<n10>\b))\B)|(?<n11>(?<n12>\b^)|(?<n13>\B$))(?<n14>(?<n15>){2,3})(?<n16>(?<n17>^)|(?<n18>\b))){0,1}))"#,
    "dgvi",
    JsString::from("ab\r\n"),
    1,
    MatchExpectation {
        text: "",
        range: 1..1,
        captures: &[
            Some(""),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        ],
        capture_ranges: &[
            Some((1, 1)),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        ],
        last_index: 1,
    };
    "input_ab_from_1_variant_14"
)]
#[test_case(
    r#"(?<n1>(?:(?:(?<n2>(?<n3>\b^)|(?<n4>\B$))(?<n5>(?<n6>){2,3})(?<n7>(?<n8>^)|(?<n9>\b))|(?<n10>(?<n11>\b^)|(?<n12>$))(?<n13>(?<n14>[a-z\q{ab}]){1}){1,1}?(?<n15>(?<n16>(?<n17>^)|(?<n18>\b))\B)){0000,0001}?))"#,
    "dgvm",
    JsString::from("a"),
    0,
    MatchExpectation {
        text: "",
        range: 0..0,
        captures: &[
            Some(""),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        ],
        capture_ranges: &[
            Some((0, 0)),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        ],
        last_index: 0,
    };
    "input_a_from_0_variant_15"
)]
#[test_case(
    r#"(?<n1>(?:(?<n2>(?<n3>(?<n4>\B^)|(?<n5>\b$))(?<n6>[\q{a|ab}])(?<n7>(?<n8>$)|(?<n9>\B))|(?<n10>(?<n11>\b\B)|(?<n12>\b\B))(?<n13>(?<n14>){2,3})(?<n15>(?<n16>$)|(?<n17>\b)){2,3}?)?))"#,
    "dgvy",
    JsString::from("a\n!"),
    0,
    MatchExpectation {
        text: "",
        range: 0..0,
        captures: &[
            Some(""),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        ],
        capture_ranges: &[
            Some((0, 0)),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        ],
        last_index: 0,
    };
    "input_a_from_0_variant_16"
)]
#[test_case(
    r#"(?<n1>(?:(?:(?<n2>(?<n3>\b\B)|(?<n4>\b\B))(?<n5>(?<n6>){2,3})(?<n7>(?<n8>$)|(?<n9>\b)){2,3}?|(?<n10>(?<n11>\B^)|(?<n12>\b$))(?<n13>(?<n14>[\q{|a|ab}])){1}(?<n15>(?<n16>$)|(?<n17>\B)))??))"#,
    "dgvi",
    JsString::from("😀a"),
    1,
    MatchExpectation {
        text: "",
        range: 0..0,
        captures: &[
            Some(""),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        ],
        capture_ranges: &[
            Some((0, 0)),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        ],
        last_index: 0,
    };
    "unicode_input_from_1_variant_17"
)]
#[test_case(
    r#"(?<n1>(?:(?<n2>(?<n3>(?<n4>\B^)|(?<n5>\b$))(?<n6>[\p{Any}\q{|ab}]){0001,0001}?(?<n7>(?<n8>$)|(?<n9>\B))|(?<n10>(?<n11>\b\B)|(?<n12>\b\B))(?<n13>(?<n14>){2,3})(?<n15>(?<n16>$)|(?<n17>\b)){2,3}?){0,1}))"#,
    "dgvm",
    JsString::from(""),
    0,
    MatchExpectation {
        text: "",
        range: 0..0,
        captures: &[
            Some(""),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        ],
        capture_ranges: &[
            Some((0, 0)),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        ],
        last_index: 0,
    };
    "empty_input_from_0_variant_18"
)]
#[test_case(
    r#"(?<n1>(?:(?:(?<n2>(?<n3>\b\B)|(?<n4>\b\B))(?<n5>(?<n6>){2,3})(?<n7>(?<n8>$)|(?<n9>\b)){2,3}?|(?<n10>(?<n11>\B^)|(?<n12>\b$))(?<n13>(?<n14>[a-z\q{ab}]){1}){1,1}?(?<n15>(?<n16>$)|(?<n17>\B))){0000,0001}?))"#,
    "dgvi",
    JsString::from("ab"),
    1,
    MatchExpectation {
        text: "",
        range: 1..1,
        captures: &[
            Some(""),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        ],
        capture_ranges: &[
            Some((1, 1)),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        ],
        last_index: 1,
    };
    "input_ab_from_1_variant_19"
)]
#[test_case(
    r#"(?<n1>(?:(?<n2>(?<n3>(?<n4>^)|(?<n5>\B))(?<n6>[\q{a|ab}])(?<n7>(?<n8>\b^)|(?<n9>\B$))|(?<n10>(?<n11>^)|(?<n12>\b))(?<n13>(?<n14>){2,3})(?<n15>(?<n16>^)|(?<n17>$)){3})?))"#,
    "dgvi",
    JsString::from("ſa"),
    1,
    MatchExpectation {
        text: "",
        range: 1..1,
        captures: &[
            Some(""),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        ],
        capture_ranges: &[
            Some((1, 1)),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        ],
        last_index: 1,
    };
    "unicode_input_from_1_variant_20"
)]
#[test_case(
    r#"(?<n1>(?:(?:(?<n2>(?<n3>^)|(?<n4>\b))(?<n5>(?<n6>){2,3})(?<n7>(?<n8>^)|(?<n9>$)){3}|(?<n10>(?<n11>^)|(?<n12>\B))(?<n13>(?<n14>[\q{|a|ab}])){1}(?<n15>(?<n16>\b^)|(?<n17>\B$)))??))"#,
    "dgvm",
    JsString::from("ab\r\n"),
    0,
    MatchExpectation {
        text: "",
        range: 0..0,
        captures: &[
            Some(""),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        ],
        capture_ranges: &[
            Some((0, 0)),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        ],
        last_index: 0,
    };
    "input_ab_from_0_variant_21"
)]
#[test_case(
    r#"(?<n1>(?:(?<n2>(?<n3>(?<n4>^)|(?<n5>\B))(?<n6>[\p{Any}\q{|ab}]){0001,0001}?(?<n7>(?<n8>\b^)|(?<n9>\B$))|(?<n10>(?<n11>^)|(?<n12>\b))(?<n13>(?<n14>){2,3})(?<n15>(?<n16>^)|(?<n17>$)){3}){0,1}))"#,
    "dgvi",
    JsString::from("a"),
    1,
    MatchExpectation {
        text: "",
        range: 1..1,
        captures: &[
            Some(""),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        ],
        capture_ranges: &[
            Some((1, 1)),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        ],
        last_index: 1,
    };
    "input_a_from_1_variant_22"
)]
#[test_case(
    r#"(?<n1>(?:(?:(?<n2>(?<n3>^)|(?<n4>\b))(?<n5>(?<n6>){2,3})(?<n7>(?<n8>^)|(?<n9>$)){3}|(?<n10>(?<n11>^)|(?<n12>\B))(?<n13>(?<n14>[a-z\q{ab}]){1}){1,1}?(?<n15>(?<n16>\b^)|(?<n17>\B$))){0000,0001}?))"#,
    "dgvy",
    JsString::from("!ab!"),
    0,
    MatchExpectation {
        text: "",
        range: 0..0,
        captures: &[
            Some(""),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        ],
        capture_ranges: &[
            Some((0, 0)),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        ],
        last_index: 0,
    };
    "input_ab_from_0_variant_23"
)]
#[test_case(
    r#"(?<n1>(?:(?<n2>(?<n3>(?<n4>(?<n5>^)|(?<n6>\b))\B)(?<n7>[\q{a|ab}])(?<n8>(?<n9>\b\B)|(?<n10>\b\B))|(?<n11>(?<n12>$)|(?<n13>\b)){2,3}?(?<n14>(?<n15>){2,3})(?<n16>(?<n17>\b^)|(?<n18>$)))?))"#,
    "dgvm",
    JsString::from("😀a"),
    0,
    MatchExpectation {
        text: "",
        range: 0..0,
        captures: &[
            Some(""),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        ],
        capture_ranges: &[
            Some((0, 0)),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        ],
        last_index: 0,
    };
    "unicode_input_from_0_variant_24"
)]
#[test_case(
    r#"(?<n1>(?:(?<n2>(?<n3>(?<n4>(?<n5>^)|(?<n6>\b))\B)(?<n7>[\p{Any}\q{|ab}]){0001,0001}?(?<n8>(?<n9>\b\B)|(?<n10>\b\B))|(?<n11>(?<n12>$)|(?<n13>\b)){2,3}?(?<n14>(?<n15>){2,3})(?<n16>(?<n17>\b^)|(?<n18>$))){0,1}))"#,
    "dgvy",
    JsString::from("ab"),
    0,
    MatchExpectation {
        text: "",
        range: 0..0,
        captures: &[
            Some(""),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        ],
        capture_ranges: &[
            Some((0, 0)),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        ],
        last_index: 0,
    };
    "input_ab_from_0_variant_25"
)]
#[test_case(
    r#"(?<n1>(?:(?:(?<n2>(?<n3>$)|(?<n4>\b)){2,3}?(?<n5>(?<n6>){2,3})(?<n7>(?<n8>\b^)|(?<n9>$))|(?<n10>(?<n11>(?<n12>^)|(?<n13>\b))\B)(?<n14>(?<n15>[a-z\q{ab}]){1}){1,1}?(?<n16>(?<n17>\b\B)|(?<n18>\b\B))){0000,0001}?))"#,
    "dgvim",
    JsString::from("a\n!"),
    1,
    MatchExpectation {
        text: "",
        range: 1..1,
        captures: &[
            Some(""),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        ],
        capture_ranges: &[
            Some((1, 1)),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        ],
        last_index: 1,
    };
    "input_a_from_1_variant_26"
)]
#[test_case(
    r#"(?<n1>(?:(?<n2>(?<n3>(?<n4>$)|(?<n5>\B))(?<n6>[\q{a|ab}])(?<n7>(?<n8>^)|(?<n9>\b))|(?<n10>(?<n11>^)|(?<n12>$)){3}(?<n13>(?<n14>){2,3})(?<n15>(?<n16>\B^)|(?<n17>\b$)))?))"#,
    "dgvi",
    JsString::from("ab\r\n"),
    1,
    MatchExpectation {
        text: "",
        range: 1..1,
        captures: &[
            Some(""),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        ],
        capture_ranges: &[
            Some((1, 1)),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        ],
        last_index: 1,
    };
    "input_ab_from_1_variant_27"
)]
#[test_case(
    r#"(?<n1>(?:(?:(?<n2>(?<n3>^)|(?<n4>$)){3}(?<n5>(?<n6>){2,3})(?<n7>(?<n8>\B^)|(?<n9>\b$))|(?<n10>(?<n11>$)|(?<n12>\B))(?<n13>(?<n14>[\q{|a|ab}])){1}(?<n15>(?<n16>^)|(?<n17>\b)))??))"#,
    "dgvy",
    JsString::from("a"),
    0,
    MatchExpectation {
        text: "",
        range: 0..0,
        captures: &[
            Some(""),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        ],
        capture_ranges: &[
            Some((0, 0)),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        ],
        last_index: 0,
    };
    "input_a_from_0_variant_28"
)]
#[test_case(
    r#"(?<n1>(?:(?<n2>(?<n3>(?<n4>$)|(?<n5>\B))(?<n6>[\p{Any}\q{|ab}]){0001,0001}?(?<n7>(?<n8>^)|(?<n9>\b))|(?<n10>(?<n11>^)|(?<n12>$)){3}(?<n13>(?<n14>){2,3})(?<n15>(?<n16>\B^)|(?<n17>\b$))){0,1}))"#,
    "dgvim",
    JsString::from("!ab!"),
    1,
    MatchExpectation {
        text: "",
        range: 1..1,
        captures: &[
            Some(""),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        ],
        capture_ranges: &[
            Some((1, 1)),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        ],
        last_index: 1,
    };
    "input_ab_from_1_variant_29"
)]
#[test_case(
    r#"(?<n1>(?:(?:(?<n2>(?<n3>^)|(?<n4>$)){3}(?<n5>(?<n6>){2,3})(?<n7>(?<n8>\B^)|(?<n9>\b$))|(?<n10>(?<n11>$)|(?<n12>\B))(?<n13>(?<n14>[a-z\q{ab}]){1}){1,1}?(?<n15>(?<n16>^)|(?<n17>\b))){0000,0001}?))"#,
    "dgv",
    JsString::from("ſa"),
    0,
    MatchExpectation {
        text: "",
        range: 0..0,
        captures: &[
            Some(""),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        ],
        capture_ranges: &[
            Some((0, 0)),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        ],
        last_index: 0,
    };
    "unicode_input_from_0_variant_30"
)]
#[test_case(
    r#"(?<n1>(?:(?<n2>(?<n3>(?<n4>\b^)|(?<n5>\B$))(?<n6>[\q{a|ab}])(?<n7>(?<n8>$)|(?<n9>\b)){2,3}?|(?<n10>(?<n11>\b^)|(?<n12>$))(?<n13>(?<n14>){2,3})(?<n15>(?<n16>^)|(?<n17>\B)))?))"#,
    "dgvy",
    JsString::from(""),
    0,
    MatchExpectation {
        text: "",
        range: 0..0,
        captures: &[
            Some(""),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        ],
        capture_ranges: &[
            Some((0, 0)),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        ],
        last_index: 0,
    };
    "empty_input_from_0_variant_31"
)]
#[test_case(
    r#"(?<n1>(?:(?:(?<n2>(?<n3>\b^)|(?<n4>$))(?<n5>(?<n6>){2,3})(?<n7>(?<n8>^)|(?<n9>\B))|(?<n10>(?<n11>\b^)|(?<n12>\B$))(?<n13>(?<n14>[\q{|a|ab}])){1}(?<n15>(?<n16>$)|(?<n17>\b)){2,3}?)??))"#,
    "dgvim",
    JsString::from("ab"),
    1,
    MatchExpectation {
        text: "",
        range: 1..1,
        captures: &[
            Some(""),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        ],
        capture_ranges: &[
            Some((1, 1)),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        ],
        last_index: 1,
    };
    "input_ab_from_1_variant_32"
)]
#[test_case(
    r#"(?<n1>(?:(?<n2>(?<n3>(?<n4>\b^)|(?<n5>\B$))(?<n6>[\p{Any}\q{|ab}]){0001,0001}?(?<n7>(?<n8>$)|(?<n9>\b)){2,3}?|(?<n10>(?<n11>\b^)|(?<n12>$))(?<n13>(?<n14>){2,3})(?<n15>(?<n16>^)|(?<n17>\B))){0,1}))"#,
    "dgv",
    JsString::from("a\n!"),
    0,
    MatchExpectation {
        text: "a",
        range: 0..1,
        captures: &[
            Some("a"),
            Some("a"),
            Some(""),
            Some(""),
            None,
            Some("a"),
            Some(""),
            None,
            Some(""),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        ],
        capture_ranges: &[
            Some((0, 1)),
            Some((0, 1)),
            Some((0, 0)),
            Some((0, 0)),
            None,
            Some((0, 1)),
            Some((1, 1)),
            None,
            Some((1, 1)),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        ],
        last_index: 1,
    };
    "input_a_from_0_variant_33"
)]
#[test_case(
    r#"(?<n1>(?:(?:(?<n2>(?<n3>\b^)|(?<n4>$))(?<n5>(?<n6>){2,3})(?<n7>(?<n8>^)|(?<n9>\B))|(?<n10>(?<n11>\b^)|(?<n12>\B$))(?<n13>(?<n14>[a-z\q{ab}]){1}){1,1}?(?<n15>(?<n16>$)|(?<n17>\b)){2,3}?){0000,0001}?))"#,
    "dgvi",
    JsString::from("😀a"),
    1,
    MatchExpectation {
        text: "",
        range: 0..0,
        captures: &[
            Some(""),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        ],
        capture_ranges: &[
            Some((0, 0)),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        ],
        last_index: 0,
    };
    "unicode_input_from_1_variant_34"
)]
#[test_case(
    r#"(?<n1>(?:(?<n2>(?<n3>(?<n4>\b\B)|(?<n5>\b\B))(?<n6>[\q{a|ab}])(?<n7>(?<n8>^)|(?<n9>$)){3}|(?<n10>(?<n11>\B^)|(?<n12>\b$))(?<n13>(?<n14>){2,3})(?<n15>(?<n16>(?<n17>^)|(?<n18>\b))\B))?))"#,
    "dgvim",
    JsString::from("a"),
    1,
    MatchExpectation {
        text: "",
        range: 1..1,
        captures: &[
            Some(""),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        ],
        capture_ranges: &[
            Some((1, 1)),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        ],
        last_index: 1,
    };
    "input_a_from_1_variant_35"
)]
#[test_case(
    r#"(?<n1>(?:(?:(?<n2>(?<n3>\B^)|(?<n4>\b$))(?<n5>(?<n6>){2,3})(?<n7>(?<n8>(?<n9>^)|(?<n10>\b))\B)|(?<n11>(?<n12>\b\B)|(?<n13>\b\B))(?<n14>(?<n15>[\q{|a|ab}])){1}(?<n16>(?<n17>^)|(?<n18>$)){3})??))"#,
    "dgv",
    JsString::from("!ab!"),
    0,
    MatchExpectation {
        text: "",
        range: 0..0,
        captures: &[
            Some(""),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        ],
        capture_ranges: &[
            Some((0, 0)),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        ],
        last_index: 0,
    };
    "input_ab_from_0_variant_36"
)]
#[test_case(
    r#"(?<n1>(?:(?<n2>(?<n3>(?<n4>\b\B)|(?<n5>\b\B))(?<n6>[\p{Any}\q{|ab}]){0001,0001}?(?<n7>(?<n8>^)|(?<n9>$)){3}|(?<n10>(?<n11>\B^)|(?<n12>\b$))(?<n13>(?<n14>){2,3})(?<n15>(?<n16>(?<n17>^)|(?<n18>\b))\B)){0,1}))"#,
    "dgvi",
    JsString::from("ſa"),
    1,
    MatchExpectation {
        text: "",
        range: 1..1,
        captures: &[
            Some(""),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        ],
        capture_ranges: &[
            Some((1, 1)),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        ],
        last_index: 1,
    };
    "unicode_input_from_1_variant_37"
)]
#[test_case(
    r#"(?<n1>(?:(?:(?<n2>(?<n3>\B^)|(?<n4>\b$))(?<n5>(?<n6>){2,3})(?<n7>(?<n8>(?<n9>^)|(?<n10>\b))\B)|(?<n11>(?<n12>\b\B)|(?<n13>\b\B))(?<n14>(?<n15>[a-z\q{ab}]){1}){1,1}?(?<n16>(?<n17>^)|(?<n18>$)){3}){0000,0001}?))"#,
    "dgvym",
    JsString::from("ab\r\n"),
    0,
    MatchExpectation {
        text: "",
        range: 0..0,
        captures: &[
            Some(""),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        ],
        capture_ranges: &[
            Some((0, 0)),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        ],
        last_index: 0,
    };
    "input_ab_from_0_variant_38"
)]
fn named_capture_results(
    pattern: &str,
    flags: &str,
    input: JsString,
    initial_last_index: usize,
    expected: MatchExpectation<'_>,
) {
    assert_named_captures(pattern, flags, input, initial_last_index, expected);
}

#[test]
fn truth_table_grouped_optional_alternative_string_classes_validate_unknowns_inside_mandatory_wrappers()
 {
    assert_true(
        r####"var pattern="^((((^)|(\\b))([\\q{a|ab}])|())?)",input="aba",r=new RegExp(pattern,"dv"),m=r.exec(input);r.source===pattern&&r.flags==="dv"&&r.hasIndices&&r.unicodeSets&&(m===null||(m.input===input&&m.groups===undefined&&m.indices.groups===undefined&&Object.hasOwn(m,"groups")&&Object.hasOwn(m.indices,"groups")))&&JSON.stringify(m===null?null:[Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex])==="[[\"ab\",\"ab\",\"ab\",\"\",\"\",\"<undefined>\",\"ab\",\"<undefined>\"],[[0,2],[0,2],[0,2],[0,0],[0,0],\"<undefined>\",[0,2],\"<undefined>\"],0,0]""####,
    );
    assert_true(
        r####"var pattern="((((^)|(\\b))([\\q{a|ab}])|())?)$",input="aba",r=new RegExp(pattern,"dv"),m=r.exec(input);r.source===pattern&&r.flags==="dv"&&r.hasIndices&&r.unicodeSets&&(m===null||(m.input===input&&m.groups===undefined&&m.indices.groups===undefined&&Object.hasOwn(m,"groups")&&Object.hasOwn(m.indices,"groups")))&&JSON.stringify(m===null?null:[Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex])==="[[\"\",\"\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],[[3,3],[3,3],\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],3,0]""####,
    );
    for pattern in [
        r"((((\b\B)|(\b\B))([\q{ab}&&\p{RGI_Emoji}])|())??)",
        r"((((^)|(\b))([\q{a|ab}])|z)??)",
        r"((((^)|(\b))([\q{a|ab}])|\1)?)",
        r"((((^)|(\b))([\q{a|ab}])|([a]))?)",
        r"((((^)|(\b))([\q{a|ab}])|())?){2}",
    ] {
        let pattern = JsString::from(pattern);
        let mut realm = Realm::default();
        crate::common::regexp_oracle::assert_matches_reference(
            &mut realm,
            &format!(
                "var marker=0;try{{new RegExp({pattern:?},'v').test('aba')}}catch{{marker=1}}finally{{marker=2}};marker"
            ),
        );
    }
}

#[test_case(
    r#"(?<n1>(?:(?:(?<n2>(?<n3>$)|(?<n4>\B))(?<n5>(?<n6>){2,3})(?<n7>(?<n8>\b\B)|(?<n9>\b\B))|(?<n10>(?<n11>^)|(?<n12>$)){3}(?<n13>(?<n14>[a-z\q{ab}]){1}){1,1}?(?<n15>(?<n16>^)|(?<n17>\B))){0000,0001}?))"#,
    "dgvi",
    JsString::from(""),
    1;
    "empty_input_from_1_variant_1"
)]
#[test_case(
    r#"(?<n1>(?:(?:(?<n2>(?<n3>$)|(?<n4>\b)){2,3}?(?<n5>(?<n6>){2,3})(?<n7>(?<n8>\b^)|(?<n9>$))|(?<n10>(?<n11>(?<n12>^)|(?<n13>\b))\B)(?<n14>(?<n15>[\q{|a|ab}])){1}(?<n16>(?<n17>\b\B)|(?<n18>\b\B)))??))"#,
    "dgvi",
    JsString::from(""),
    1;
    "empty_input_from_1_variant_2"
)]
fn no_match_resets_last_index(
    pattern: &str,
    flags: &str,
    input: JsString,
    initial_last_index: usize,
) {
    assert_no_match(pattern, flags, input, initial_last_index);
}
