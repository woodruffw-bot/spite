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
    realm.eval(r"var r=/(?<outer>(?<nested>(?<b>(?<s>^)|(?<w>\b))(?<c>[\q{a|ab}])(?<a>(?<end>$)|(?<nw>\B))|(?<fallback>(?<wb>\b)|(?<nwb>\B))(?<empty>)))/dvg").unwrap();
    realm
        .eval(r"r=new RegExp('('+r.source+'){0}','dvg');var clone=new RegExp(r),m=r.exec('ab!aa')")
        .unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval(r#"
        var n = clone.exec('ab');
        r.lastIndex = 0;
        var p = r.exec('!');
        m[0] === '' && m.index === 0 && Array.from(m).slice(1).every(v => v === undefined) && m.indices
            .slice(1).every(v => v === undefined) && Object.hasOwn(m.groups, 'outer') && Object.hasOwn(
                m.groups, 'nested') && Object.hasOwn(m.indices.groups, 'c') && m.groups.c ===
            undefined && m.indices.groups.c === m.indices[7] && m.groups.outer === undefined && m
            .groups.nested === undefined && n[0] === '' && n.index === 0 && n.groups.c === undefined &&
            clone.source === r.source && clone.lastIndex === 0 && p[0] === '' && p.index === 0 && Array
            .from(p).slice(1).every(v => v === undefined) && p.indices.slice(1).every(v => v ===
                undefined) && p.groups.outer === undefined && p.groups.nested === undefined && p.groups
            .fallback === undefined && p.groups.empty === undefined && r.lastIndex === 0
    "#),Ok(Value::Boolean(true)));
}

#[test]
fn match_all_preserves_capture_metadata() {
    let source = format!(
        r#"
            (function() {{
                {DESCRIBE_MATCH}

                var P =
                    "(?<outer>(?<nested>(?<b>(?<s>^)|(?<w>\\b))(?<whole>(?<c>[\\q{{|a|ab}}]))(?<a>(?<end>$)|(?<nw>\\B))|(?<fallback>(?<wb>\\b)|(?<nwb>\\B))(?<empty>))){{0}}";
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
            "(?<outer>(?<nested>(?<b>(?<s>^)|(?<w>\\b))(?<whole>(?<c>[\\q{|a|ab}]))(?<a>(?<end>$)|(?<nw>\\B))|(?<fallback>(?<wb>\\b)|(?<nwb>\\B))(?<empty>))){0}";
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
                "(?<outer>(?<nested>(?<b>(?<s>^)|(?<w>\\b))(?<whole>(?<c>[\\q{|a|ab}]))(?<a>(?<end>$)|(?<nw>\\B))|(?<fallback>(?<wb>\\b)|(?<nwb>\\B))(?<empty>))){0}";
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
            "<undefined>",
            "a",
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
            "<undefined>",
            "b",
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
            "<undefined>",
            "a",
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
            "<undefined>",
            "a",
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
            "(?<outer>(?<nested>(?<b>(?<s>^)|(?<w>\\b))(?<whole>(?<c>[\\q{|a|ab}]))(?<a>(?<end>$)|(?<nw>\\B))|(?<fallback>(?<wb>\\b)|(?<nwb>\\B))(?<empty>))){0}";
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
                    "(?<outer>(?<nested>(?<b>(?<s>^)|(?<w>\\b))(?<whole>(?<c>[\\q{{|a|ab}}]))(?<a>(?<end>$)|(?<nw>\\B))|(?<fallback>(?<wb>\\b)|(?<nwb>\\B))(?<empty>))){{0}}";
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
                    "(?<outer>(?<nested>(?<b>(?<s>^)|(?<w>\\b))(?<whole>(?<c>[a-z\\q{{|S|sa|aX}}]))(?<a>(?<end>$)|(?<nw>\\B))|(?<fallback>(?<wb>\\b)|(?<nwb>\\B))(?<empty>))){{0}}";
                var r = new RegExp(P, "dvgi"),
                    a = r.exec("ſaX!");
                return JSON.stringify([describeMatch(a), r.lastIndex], null, 2);
            }})()
        "#
    );
    let report = eval_string(&mut Realm::default(), &source);
    insta::assert_snapshot!(report, @r#"
        [
          {
            "matches": [
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
              "<undefined>"
            ],
            "indices": [
              [
                0,
                0
              ],
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
              "<undefined>"
            ],
            "index": 0,
            "input": "ſaX!",
            "groupNames": [
              "outer",
              "nested",
              "b",
              "s",
              "w",
              "whole",
              "c",
              "a",
              "end",
              "nw",
              "fallback",
              "wb",
              "nwb",
              "empty"
            ],
            "namedCaptures": [
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
              "<undefined>"
            ],
            "namedIndices": [
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
              "<undefined>"
            ],
            "namedAliasesValid": true,
            "groupsPrototypeValid": true
          },
          0
        ]
        "#);
}

#[test]
fn multiline_match_all_preserves_capture_metadata() {
    let source = format!(
        r#"
            (function() {{
                {DESCRIBE_MATCH}

                var P =
                    "(?<outer>(?<nested>(?<b>(?<s>^)|(?<w>\\b))(?<whole>(?<c>[\\q{{|a|ab}}]))(?<a>(?<end>$)|(?<nw>\\B))|(?<fallback>(?<wb>\\b)|(?<nwb>\\B))(?<empty>))){{0}}";
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
                    "(?<outer>(?<nested>(?<\\u{{1D49C}}>(?<s>^)|(?<w>\\b))(?<whole>(?<c>[\\q{{|a|ab}}]))(?<a>(?<end>$)|(?<nw>\\B))|(?<fallback>(?<wb>\\b)|(?<nwb>\\B))(?<empty>))){{0}}";
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
    insta::assert_snapshot!(report, @r#"
        [
          {
            "matches": [
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
              "<undefined>"
            ],
            "indices": [
              [
                0,
                0
              ],
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
              "<undefined>"
            ],
            "index": 0,
            "input": "ab!",
            "groupNames": [
              "outer",
              "nested",
              "𝒜",
              "s",
              "w",
              "whole",
              "c",
              "a",
              "end",
              "nw",
              "fallback",
              "wb",
              "nwb",
              "empty"
            ],
            "namedCaptures": [
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
              "<undefined>"
            ],
            "namedIndices": [
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
              "<undefined>"
            ],
            "namedAliasesValid": true,
            "groupsPrototypeValid": true
          },
          true,
          true,
          0,
          3
        ]
        "#);
}

#[test_case(
    r#"(?<n1>(?<n2>(?<n3>(?<n4>^)|(?<n5>\b))(?<n6>[\q{a|ab}])(?<n7>(?<n8>\b^)|(?<n9>$))|(?<n10>(?<n11>^)|(?<n12>\B))(?<n13>(?<n14>){2,3})(?<n15>(?<n16>$)|(?<n17>\B)))){0}"#,
    "dgvym",
    JsString::from(""),
    0,
    MatchExpectation {
        text: "",
        range: 0..0,
        captures: &[None; 17],
        capture_ranges: &[None; 17],
        last_index: 0,
    };
    "empty_input_from_0_variant_1"
)]
#[test_case(
    r#"(?:(?:(?<n1>(?<n2>(?<n3>(?<n4>^)|(?<n5>\B))(?<n6>(?<n7>){2,3})(?<n8>(?<n9>$)|(?<n10>\B))|(?<n11>(?<n12>^)|(?<n13>\b))(?<n14>(?<n15>[\q{|a|ab}])){1}(?<n16>(?<n17>\b^)|(?<n18>$)))))){0,0}?"#,
    "dgvi",
    JsString::from("ab"),
    1,
    MatchExpectation {
        text: "",
        range: 1..1,
        captures: &[None; 18],
        capture_ranges: &[None; 18],
        last_index: 1,
    };
    "input_ab_from_1_variant_2"
)]
#[test_case(
    r#"(?<n1>(?<n2>(?:(?<n3>(?<n4>^)|(?<n5>\b))(?<n6>[\p{Any}\q{|ab}]){0001,0001}?(?<n7>(?<n8>\b^)|(?<n9>$))|(?<n10>(?<n11>^)|(?<n12>\B))(?<n13>(?<n14>){2,3})(?<n15>(?<n16>$)|(?<n17>\B))){1,1})){0000,0000}"#,
    "dgv",
    JsString::from("a\n!"),
    0,
    MatchExpectation {
        text: "",
        range: 0..0,
        captures: &[None; 17],
        capture_ranges: &[None; 17],
        last_index: 0,
    };
    "input_a_from_0_variant_3"
)]
#[test_case(
    r#"(?:(?<n1>(?<n2>(?<n3>(?<n4>(?<n5>^)|(?<n6>\B))(?<n7>(?<n8>){2,3})(?<n9>(?<n10>$)|(?<n11>\B))|(?<n12>(?<n13>^)|(?<n14>\b))(?<n15>(?<n16>[a-z\q{ab}]){1}){1,1}?(?<n17>(?<n18>\b^)|(?<n19>$)))){0001,0001}?)){0000}?"#,
    "dgvim",
    JsString::from("😀a"),
    1,
    MatchExpectation {
        text: "",
        range: 0..0,
        captures: &[None; 19],
        capture_ranges: &[None; 19],
        last_index: 0,
    };
    "unicode_input_from_1_variant_4"
)]
#[test_case(
    r#"(?<n1>(?<n2>(?<n3>(?<n4>$)|(?<n5>\b)){2,3}?(?<n6>[\q{a|ab}])(?<n7>(?<n8>\B^)|(?<n9>\b$))|(?<n10>(?<n11>(?<n12>^)|(?<n13>\b))\B)(?<n14>(?<n15>){2,3})(?<n16>(?<n17>\b^)|(?<n18>\B$)))){0}"#,
    "dgvi",
    JsString::from("a"),
    1,
    MatchExpectation {
        text: "",
        range: 1..1,
        captures: &[None; 18],
        capture_ranges: &[None; 18],
        last_index: 1,
    };
    "input_a_from_1_variant_5"
)]
#[test_case(
    r#"(?:(?:(?<n1>(?<n2>(?<n3>(?<n4>(?<n5>^)|(?<n6>\b))\B)(?<n7>(?<n8>){2,3})(?<n9>(?<n10>\b^)|(?<n11>\B$))|(?<n12>(?<n13>$)|(?<n14>\b)){2,3}?(?<n15>(?<n16>[\q{|a|ab}])){1}(?<n17>(?<n18>\B^)|(?<n19>\b$)))))){0,0}?"#,
    "dgv",
    JsString::from("!ab!"),
    0,
    MatchExpectation {
        text: "",
        range: 0..0,
        captures: &[None; 19],
        capture_ranges: &[None; 19],
        last_index: 0,
    };
    "input_ab_from_0_variant_6"
)]
#[test_case(
    r#"(?<n1>(?<n2>(?:(?<n3>(?<n4>$)|(?<n5>\b)){2,3}?(?<n6>[\p{Any}\q{|ab}]){0001,0001}?(?<n7>(?<n8>\B^)|(?<n9>\b$))|(?<n10>(?<n11>(?<n12>^)|(?<n13>\b))\B)(?<n14>(?<n15>){2,3})(?<n16>(?<n17>\b^)|(?<n18>\B$))){1,1})){0000,0000}"#,
    "dgvim",
    JsString::from("ſa"),
    1,
    MatchExpectation {
        text: "",
        range: 1..1,
        captures: &[None; 18],
        capture_ranges: &[None; 18],
        last_index: 1,
    };
    "unicode_input_from_1_variant_7"
)]
#[test_case(
    r#"(?:(?<n1>(?<n2>(?<n3>(?<n4>(?<n5>(?<n6>^)|(?<n7>\b))\B)(?<n8>(?<n9>){2,3})(?<n10>(?<n11>\b^)|(?<n12>\B$))|(?<n13>(?<n14>$)|(?<n15>\b)){2,3}?(?<n16>(?<n17>[a-z\q{ab}]){1}){1,1}?(?<n18>(?<n19>\B^)|(?<n20>\b$)))){0001,0001}?)){0000}?"#,
    "dgvy",
    JsString::from("ab\r\n"),
    0,
    MatchExpectation {
        text: "",
        range: 0..0,
        captures: &[None; 20],
        capture_ranges: &[None; 20],
        last_index: 0,
    };
    "input_ab_from_0_variant_8"
)]
#[test_case(
    r#"(?<n1>(?<n2>(?<n3>(?<n4>^)|(?<n5>$)){3}(?<n6>[\q{a|ab}])(?<n7>(?<n8>^)|(?<n9>\B))|(?<n10>(?<n11>$)|(?<n12>\B))(?<n13>(?<n14>){2,3})(?<n15>(?<n16>\b\B)|(?<n17>\b\B)))){0}"#,
    "dgv",
    JsString::from("ab"),
    0,
    MatchExpectation {
        text: "",
        range: 0..0,
        captures: &[None; 17],
        capture_ranges: &[None; 17],
        last_index: 0,
    };
    "input_ab_from_0_variant_9"
)]
#[test_case(
    r#"(?:(?:(?<n1>(?<n2>(?<n3>(?<n4>$)|(?<n5>\B))(?<n6>(?<n7>){2,3})(?<n8>(?<n9>\b\B)|(?<n10>\b\B))|(?<n11>(?<n12>^)|(?<n13>$)){3}(?<n14>(?<n15>[\q{|a|ab}])){1}(?<n16>(?<n17>^)|(?<n18>\B)))))){0,0}?"#,
    "dgvim",
    JsString::from("a\n!"),
    1,
    MatchExpectation {
        text: "",
        range: 1..1,
        captures: &[None; 18],
        capture_ranges: &[None; 18],
        last_index: 1,
    };
    "input_a_from_1_variant_10"
)]
#[test_case(
    r#"(?<n1>(?<n2>(?:(?<n3>(?<n4>^)|(?<n5>$)){3}(?<n6>[\p{Any}\q{|ab}]){0001,0001}?(?<n7>(?<n8>^)|(?<n9>\B))|(?<n10>(?<n11>$)|(?<n12>\B))(?<n13>(?<n14>){2,3})(?<n15>(?<n16>\b\B)|(?<n17>\b\B))){1,1})){0000,0000}"#,
    "dgvy",
    JsString::from("😀a"),
    0,
    MatchExpectation {
        text: "",
        range: 0..0,
        captures: &[None; 17],
        capture_ranges: &[None; 17],
        last_index: 0,
    };
    "unicode_input_from_0_variant_11"
)]
#[test_case(
    r#"(?<n1>(?<n2>(?<n3>(?<n4>\b^)|(?<n5>$))(?<n6>[\q{a|ab}])(?<n7>(?<n8>(?<n9>^)|(?<n10>\b))\B)|(?<n11>(?<n12>\b^)|(?<n13>\B$))(?<n14>(?<n15>){2,3})(?<n16>(?<n17>^)|(?<n18>\b)))){0}"#,
    "dgvim",
    JsString::from("!ab!"),
    1,
    MatchExpectation {
        text: "",
        range: 1..1,
        captures: &[None; 18],
        capture_ranges: &[None; 18],
        last_index: 1,
    };
    "input_ab_from_1_variant_12"
)]
#[test_case(
    r#"(?:(?:(?<n1>(?<n2>(?<n3>(?<n4>\b^)|(?<n5>\B$))(?<n6>(?<n7>){2,3})(?<n8>(?<n9>^)|(?<n10>\b))|(?<n11>(?<n12>\b^)|(?<n13>$))(?<n14>(?<n15>[\q{|a|ab}])){1}(?<n16>(?<n17>(?<n18>^)|(?<n19>\b))\B))))){0,0}?"#,
    "dgvy",
    JsString::from("ſa"),
    0,
    MatchExpectation {
        text: "",
        range: 0..0,
        captures: &[None; 19],
        capture_ranges: &[None; 19],
        last_index: 0,
    };
    "unicode_input_from_0_variant_13"
)]
#[test_case(
    r#"(?<n1>(?<n2>(?:(?<n3>(?<n4>\b^)|(?<n5>$))(?<n6>[\p{Any}\q{|ab}]){0001,0001}?(?<n7>(?<n8>(?<n9>^)|(?<n10>\b))\B)|(?<n11>(?<n12>\b^)|(?<n13>\B$))(?<n14>(?<n15>){2,3})(?<n16>(?<n17>^)|(?<n18>\b))){1,1})){0000,0000}"#,
    "dgvi",
    JsString::from("ab\r\n"),
    1,
    MatchExpectation {
        text: "",
        range: 1..1,
        captures: &[None; 18],
        capture_ranges: &[None; 18],
        last_index: 1,
    };
    "input_ab_from_1_variant_14"
)]
#[test_case(
    r#"(?:(?<n1>(?<n2>(?<n3>(?<n4>(?<n5>\b^)|(?<n6>\B$))(?<n7>(?<n8>){2,3})(?<n9>(?<n10>^)|(?<n11>\b))|(?<n12>(?<n13>\b^)|(?<n14>$))(?<n15>(?<n16>[a-z\q{ab}]){1}){1,1}?(?<n17>(?<n18>(?<n19>^)|(?<n20>\b))\B))){0001,0001}?)){0000}?"#,
    "dgvm",
    JsString::from("a"),
    0,
    MatchExpectation {
        text: "",
        range: 0..0,
        captures: &[None; 20],
        capture_ranges: &[None; 20],
        last_index: 0,
    };
    "input_a_from_0_variant_15"
)]
#[test_case(
    r#"(?<n1>(?<n2>(?<n3>(?<n4>\B^)|(?<n5>\b$))(?<n6>[\q{a|ab}])(?<n7>(?<n8>$)|(?<n9>\B))|(?<n10>(?<n11>\b\B)|(?<n12>\b\B))(?<n13>(?<n14>){2,3})(?<n15>(?<n16>$)|(?<n17>\b)){2,3}?)){0}"#,
    "dgvy",
    JsString::from("a\n!"),
    0,
    MatchExpectation {
        text: "",
        range: 0..0,
        captures: &[None; 17],
        capture_ranges: &[None; 17],
        last_index: 0,
    };
    "input_a_from_0_variant_16"
)]
#[test_case(
    r#"(?:(?:(?<n1>(?<n2>(?<n3>(?<n4>\b\B)|(?<n5>\b\B))(?<n6>(?<n7>){2,3})(?<n8>(?<n9>$)|(?<n10>\b)){2,3}?|(?<n11>(?<n12>\B^)|(?<n13>\b$))(?<n14>(?<n15>[\q{|a|ab}])){1}(?<n16>(?<n17>$)|(?<n18>\B)))))){0,0}?"#,
    "dgvi",
    JsString::from("😀a"),
    1,
    MatchExpectation {
        text: "",
        range: 0..0,
        captures: &[None; 18],
        capture_ranges: &[None; 18],
        last_index: 0,
    };
    "unicode_input_from_1_variant_17"
)]
#[test_case(
    r#"(?<n1>(?<n2>(?:(?<n3>(?<n4>\B^)|(?<n5>\b$))(?<n6>[\p{Any}\q{|ab}]){0001,0001}?(?<n7>(?<n8>$)|(?<n9>\B))|(?<n10>(?<n11>\b\B)|(?<n12>\b\B))(?<n13>(?<n14>){2,3})(?<n15>(?<n16>$)|(?<n17>\b)){2,3}?){1,1})){0000,0000}"#,
    "dgvm",
    JsString::from(""),
    0,
    MatchExpectation {
        text: "",
        range: 0..0,
        captures: &[None; 17],
        capture_ranges: &[None; 17],
        last_index: 0,
    };
    "empty_input_from_0_variant_18"
)]
#[test_case(
    r#"(?:(?<n1>(?<n2>(?<n3>(?<n4>(?<n5>\b\B)|(?<n6>\b\B))(?<n7>(?<n8>){2,3})(?<n9>(?<n10>$)|(?<n11>\b)){2,3}?|(?<n12>(?<n13>\B^)|(?<n14>\b$))(?<n15>(?<n16>[a-z\q{ab}]){1}){1,1}?(?<n17>(?<n18>$)|(?<n19>\B)))){0001,0001}?)){0000}?"#,
    "dgvi",
    JsString::from("ab"),
    1,
    MatchExpectation {
        text: "",
        range: 1..1,
        captures: &[None; 19],
        capture_ranges: &[None; 19],
        last_index: 1,
    };
    "input_ab_from_1_variant_19"
)]
#[test_case(
    r#"(?<n1>(?<n2>(?<n3>(?<n4>^)|(?<n5>\B))(?<n6>[\q{a|ab}])(?<n7>(?<n8>\b^)|(?<n9>\B$))|(?<n10>(?<n11>^)|(?<n12>\b))(?<n13>(?<n14>){2,3})(?<n15>(?<n16>^)|(?<n17>$)){3})){0}"#,
    "dgvi",
    JsString::from("ſa"),
    1,
    MatchExpectation {
        text: "",
        range: 1..1,
        captures: &[None; 17],
        capture_ranges: &[None; 17],
        last_index: 1,
    };
    "unicode_input_from_1_variant_20"
)]
#[test_case(
    r#"(?:(?:(?<n1>(?<n2>(?<n3>(?<n4>^)|(?<n5>\b))(?<n6>(?<n7>){2,3})(?<n8>(?<n9>^)|(?<n10>$)){3}|(?<n11>(?<n12>^)|(?<n13>\B))(?<n14>(?<n15>[\q{|a|ab}])){1}(?<n16>(?<n17>\b^)|(?<n18>\B$)))))){0,0}?"#,
    "dgvm",
    JsString::from("ab\r\n"),
    0,
    MatchExpectation {
        text: "",
        range: 0..0,
        captures: &[None; 18],
        capture_ranges: &[None; 18],
        last_index: 0,
    };
    "input_ab_from_0_variant_21"
)]
#[test_case(
    r#"(?<n1>(?<n2>(?:(?<n3>(?<n4>^)|(?<n5>\B))(?<n6>[\p{Any}\q{|ab}]){0001,0001}?(?<n7>(?<n8>\b^)|(?<n9>\B$))|(?<n10>(?<n11>^)|(?<n12>\b))(?<n13>(?<n14>){2,3})(?<n15>(?<n16>^)|(?<n17>$)){3}){1,1})){0000,0000}"#,
    "dgvi",
    JsString::from("a"),
    1,
    MatchExpectation {
        text: "",
        range: 1..1,
        captures: &[None; 17],
        capture_ranges: &[None; 17],
        last_index: 1,
    };
    "input_a_from_1_variant_22"
)]
#[test_case(
    r#"(?:(?<n1>(?<n2>(?<n3>(?<n4>(?<n5>^)|(?<n6>\b))(?<n7>(?<n8>){2,3})(?<n9>(?<n10>^)|(?<n11>$)){3}|(?<n12>(?<n13>^)|(?<n14>\B))(?<n15>(?<n16>[a-z\q{ab}]){1}){1,1}?(?<n17>(?<n18>\b^)|(?<n19>\B$)))){0001,0001}?)){0000}?"#,
    "dgvy",
    JsString::from("!ab!"),
    0,
    MatchExpectation {
        text: "",
        range: 0..0,
        captures: &[None; 19],
        capture_ranges: &[None; 19],
        last_index: 0,
    };
    "input_ab_from_0_variant_23"
)]
#[test_case(
    r#"(?<n1>(?<n2>(?<n3>(?<n4>(?<n5>^)|(?<n6>\b))\B)(?<n7>[\q{a|ab}])(?<n8>(?<n9>\b\B)|(?<n10>\b\B))|(?<n11>(?<n12>$)|(?<n13>\b)){2,3}?(?<n14>(?<n15>){2,3})(?<n16>(?<n17>\b^)|(?<n18>$)))){0}"#,
    "dgvm",
    JsString::from("😀a"),
    0,
    MatchExpectation {
        text: "",
        range: 0..0,
        captures: &[None; 18],
        capture_ranges: &[None; 18],
        last_index: 0,
    };
    "unicode_input_from_0_variant_24"
)]
#[test_case(
    r#"(?<n1>(?<n2>(?:(?<n3>(?<n4>(?<n5>^)|(?<n6>\b))\B)(?<n7>[\p{Any}\q{|ab}]){0001,0001}?(?<n8>(?<n9>\b\B)|(?<n10>\b\B))|(?<n11>(?<n12>$)|(?<n13>\b)){2,3}?(?<n14>(?<n15>){2,3})(?<n16>(?<n17>\b^)|(?<n18>$))){1,1})){0000,0000}"#,
    "dgvy",
    JsString::from("ab"),
    0,
    MatchExpectation {
        text: "",
        range: 0..0,
        captures: &[None; 18],
        capture_ranges: &[None; 18],
        last_index: 0,
    };
    "input_ab_from_0_variant_25"
)]
#[test_case(
    r#"(?:(?<n1>(?<n2>(?<n3>(?<n4>(?<n5>$)|(?<n6>\b)){2,3}?(?<n7>(?<n8>){2,3})(?<n9>(?<n10>\b^)|(?<n11>$))|(?<n12>(?<n13>(?<n14>^)|(?<n15>\b))\B)(?<n16>(?<n17>[a-z\q{ab}]){1}){1,1}?(?<n18>(?<n19>\b\B)|(?<n20>\b\B)))){0001,0001}?)){0000}?"#,
    "dgvim",
    JsString::from("a\n!"),
    1,
    MatchExpectation {
        text: "",
        range: 1..1,
        captures: &[None; 20],
        capture_ranges: &[None; 20],
        last_index: 1,
    };
    "input_a_from_1_variant_26"
)]
#[test_case(
    r#"(?<n1>(?<n2>(?<n3>(?<n4>$)|(?<n5>\B))(?<n6>[\q{a|ab}])(?<n7>(?<n8>^)|(?<n9>\b))|(?<n10>(?<n11>^)|(?<n12>$)){3}(?<n13>(?<n14>){2,3})(?<n15>(?<n16>\B^)|(?<n17>\b$)))){0}"#,
    "dgvi",
    JsString::from("ab\r\n"),
    1,
    MatchExpectation {
        text: "",
        range: 1..1,
        captures: &[None; 17],
        capture_ranges: &[None; 17],
        last_index: 1,
    };
    "input_ab_from_1_variant_27"
)]
#[test_case(
    r#"(?:(?:(?<n1>(?<n2>(?<n3>(?<n4>^)|(?<n5>$)){3}(?<n6>(?<n7>){2,3})(?<n8>(?<n9>\B^)|(?<n10>\b$))|(?<n11>(?<n12>$)|(?<n13>\B))(?<n14>(?<n15>[\q{|a|ab}])){1}(?<n16>(?<n17>^)|(?<n18>\b)))))){0,0}?"#,
    "dgvy",
    JsString::from("a"),
    0,
    MatchExpectation {
        text: "",
        range: 0..0,
        captures: &[None; 18],
        capture_ranges: &[None; 18],
        last_index: 0,
    };
    "input_a_from_0_variant_28"
)]
#[test_case(
    r#"(?<n1>(?<n2>(?:(?<n3>(?<n4>$)|(?<n5>\B))(?<n6>[\p{Any}\q{|ab}]){0001,0001}?(?<n7>(?<n8>^)|(?<n9>\b))|(?<n10>(?<n11>^)|(?<n12>$)){3}(?<n13>(?<n14>){2,3})(?<n15>(?<n16>\B^)|(?<n17>\b$))){1,1})){0000,0000}"#,
    "dgvim",
    JsString::from("!ab!"),
    1,
    MatchExpectation {
        text: "",
        range: 1..1,
        captures: &[None; 17],
        capture_ranges: &[None; 17],
        last_index: 1,
    };
    "input_ab_from_1_variant_29"
)]
#[test_case(
    r#"(?:(?<n1>(?<n2>(?<n3>(?<n4>(?<n5>^)|(?<n6>$)){3}(?<n7>(?<n8>){2,3})(?<n9>(?<n10>\B^)|(?<n11>\b$))|(?<n12>(?<n13>$)|(?<n14>\B))(?<n15>(?<n16>[a-z\q{ab}]){1}){1,1}?(?<n17>(?<n18>^)|(?<n19>\b)))){0001,0001}?)){0000}?"#,
    "dgv",
    JsString::from("ſa"),
    0,
    MatchExpectation {
        text: "",
        range: 0..0,
        captures: &[None; 19],
        capture_ranges: &[None; 19],
        last_index: 0,
    };
    "unicode_input_from_0_variant_30"
)]
#[test_case(
    r#"(?<n1>(?<n2>(?<n3>(?<n4>\b^)|(?<n5>\B$))(?<n6>[\q{a|ab}])(?<n7>(?<n8>$)|(?<n9>\b)){2,3}?|(?<n10>(?<n11>\b^)|(?<n12>$))(?<n13>(?<n14>){2,3})(?<n15>(?<n16>^)|(?<n17>\B)))){0}"#,
    "dgvy",
    JsString::from(""),
    0,
    MatchExpectation {
        text: "",
        range: 0..0,
        captures: &[None; 17],
        capture_ranges: &[None; 17],
        last_index: 0,
    };
    "empty_input_from_0_variant_31"
)]
#[test_case(
    r#"(?:(?:(?<n1>(?<n2>(?<n3>(?<n4>\b^)|(?<n5>$))(?<n6>(?<n7>){2,3})(?<n8>(?<n9>^)|(?<n10>\B))|(?<n11>(?<n12>\b^)|(?<n13>\B$))(?<n14>(?<n15>[\q{|a|ab}])){1}(?<n16>(?<n17>$)|(?<n18>\b)){2,3}?)))){0,0}?"#,
    "dgvim",
    JsString::from("ab"),
    1,
    MatchExpectation {
        text: "",
        range: 1..1,
        captures: &[None; 18],
        capture_ranges: &[None; 18],
        last_index: 1,
    };
    "input_ab_from_1_variant_32"
)]
#[test_case(
    r#"(?<n1>(?<n2>(?:(?<n3>(?<n4>\b^)|(?<n5>\B$))(?<n6>[\p{Any}\q{|ab}]){0001,0001}?(?<n7>(?<n8>$)|(?<n9>\b)){2,3}?|(?<n10>(?<n11>\b^)|(?<n12>$))(?<n13>(?<n14>){2,3})(?<n15>(?<n16>^)|(?<n17>\B))){1,1})){0000,0000}"#,
    "dgv",
    JsString::from("a\n!"),
    0,
    MatchExpectation {
        text: "",
        range: 0..0,
        captures: &[None; 17],
        capture_ranges: &[None; 17],
        last_index: 0,
    };
    "input_a_from_0_variant_33"
)]
#[test_case(
    r#"(?:(?<n1>(?<n2>(?<n3>(?<n4>(?<n5>\b^)|(?<n6>$))(?<n7>(?<n8>){2,3})(?<n9>(?<n10>^)|(?<n11>\B))|(?<n12>(?<n13>\b^)|(?<n14>\B$))(?<n15>(?<n16>[a-z\q{ab}]){1}){1,1}?(?<n17>(?<n18>$)|(?<n19>\b)){2,3}?)){0001,0001}?)){0000}?"#,
    "dgvi",
    JsString::from("😀a"),
    1,
    MatchExpectation {
        text: "",
        range: 0..0,
        captures: &[None; 19],
        capture_ranges: &[None; 19],
        last_index: 0,
    };
    "unicode_input_from_1_variant_34"
)]
#[test_case(
    r#"(?<n1>(?<n2>(?<n3>(?<n4>\b\B)|(?<n5>\b\B))(?<n6>[\q{a|ab}])(?<n7>(?<n8>^)|(?<n9>$)){3}|(?<n10>(?<n11>\B^)|(?<n12>\b$))(?<n13>(?<n14>){2,3})(?<n15>(?<n16>(?<n17>^)|(?<n18>\b))\B))){0}"#,
    "dgvim",
    JsString::from("a"),
    1,
    MatchExpectation {
        text: "",
        range: 1..1,
        captures: &[None; 18],
        capture_ranges: &[None; 18],
        last_index: 1,
    };
    "input_a_from_1_variant_35"
)]
#[test_case(
    r#"(?:(?:(?<n1>(?<n2>(?<n3>(?<n4>\B^)|(?<n5>\b$))(?<n6>(?<n7>){2,3})(?<n8>(?<n9>(?<n10>^)|(?<n11>\b))\B)|(?<n12>(?<n13>\b\B)|(?<n14>\b\B))(?<n15>(?<n16>[\q{|a|ab}])){1}(?<n17>(?<n18>^)|(?<n19>$)){3})))){0,0}?"#,
    "dgv",
    JsString::from("!ab!"),
    0,
    MatchExpectation {
        text: "",
        range: 0..0,
        captures: &[None; 19],
        capture_ranges: &[None; 19],
        last_index: 0,
    };
    "input_ab_from_0_variant_36"
)]
#[test_case(
    r#"(?<n1>(?<n2>(?:(?<n3>(?<n4>\b\B)|(?<n5>\b\B))(?<n6>[\p{Any}\q{|ab}]){0001,0001}?(?<n7>(?<n8>^)|(?<n9>$)){3}|(?<n10>(?<n11>\B^)|(?<n12>\b$))(?<n13>(?<n14>){2,3})(?<n15>(?<n16>(?<n17>^)|(?<n18>\b))\B)){1,1})){0000,0000}"#,
    "dgvi",
    JsString::from("ſa"),
    1,
    MatchExpectation {
        text: "",
        range: 1..1,
        captures: &[None; 18],
        capture_ranges: &[None; 18],
        last_index: 1,
    };
    "unicode_input_from_1_variant_37"
)]
#[test_case(
    r#"(?:(?<n1>(?<n2>(?<n3>(?<n4>(?<n5>\B^)|(?<n6>\b$))(?<n7>(?<n8>){2,3})(?<n9>(?<n10>(?<n11>^)|(?<n12>\b))\B)|(?<n13>(?<n14>\b\B)|(?<n15>\b\B))(?<n16>(?<n17>[a-z\q{ab}]){1}){1,1}?(?<n18>(?<n19>^)|(?<n20>$)){3})){0001,0001}?)){0000}?"#,
    "dgvym",
    JsString::from("ab\r\n"),
    0,
    MatchExpectation {
        text: "",
        range: 0..0,
        captures: &[None; 20],
        capture_ranges: &[None; 20],
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
fn truth_table_nested_zero_count_alternative_string_classes_validate_full_unexecuted_bodies() {
    assert_true(
        r####"var pattern="^((((^)|(\\b))([\\q{a|ab}])|())){0}",input="aba",r=new RegExp(pattern,"dv"),m=r.exec(input);r.source===pattern&&r.flags==="dv"&&r.hasIndices&&r.unicodeSets&&(m===null||(m.input===input&&Object.hasOwn(m,"groups")&&Object.hasOwn(m.indices,"groups")&&m.groups===undefined&&m.indices.groups===undefined))&&JSON.stringify(m===null?null:[Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex,m.groups===undefined?"<undefined>":Object.keys(m.groups).map(k=>[k,m.groups[k]===undefined?"<undefined>":m.groups[k]]),m.indices.groups===undefined?"<undefined>":Object.keys(m.indices.groups).map(k=>[k,m.indices.groups[k]===undefined?"<undefined>":m.indices.groups[k]])])==="[[\"\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],[[0,0],\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],0,0,\"<undefined>\",\"<undefined>\"]""####,
    );
    assert_true(
        r####"var pattern="((((^)|(\\b))([\\q{a|ab}])|())){0}$",input="aba",r=new RegExp(pattern,"dv"),m=r.exec(input);r.source===pattern&&r.flags==="dv"&&r.hasIndices&&r.unicodeSets&&(m===null||(m.input===input&&Object.hasOwn(m,"groups")&&Object.hasOwn(m.indices,"groups")&&m.groups===undefined&&m.indices.groups===undefined))&&JSON.stringify(m===null?null:[Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex,m.groups===undefined?"<undefined>":Object.keys(m.groups).map(k=>[k,m.groups[k]===undefined?"<undefined>":m.groups[k]]),m.indices.groups===undefined?"<undefined>":Object.keys(m.indices.groups).map(k=>[k,m.indices.groups[k]===undefined?"<undefined>":m.indices.groups[k]])])==="[[\"\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],[[3,3],\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],3,0,\"<undefined>\",\"<undefined>\"]""####,
    );
    assert_true(
        r####"var pattern="((((^)|(\\b))([\\q{a|ab}])|()){0,1}){0}",input="aba",r=new RegExp(pattern,"dv"),m=r.exec(input);r.source===pattern&&r.flags==="dv"&&r.hasIndices&&r.unicodeSets&&(m===null||(m.input===input&&Object.hasOwn(m,"groups")&&Object.hasOwn(m.indices,"groups")&&m.groups===undefined&&m.indices.groups===undefined))&&JSON.stringify(m===null?null:[Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex,m.groups===undefined?"<undefined>":Object.keys(m.groups).map(k=>[k,m.groups[k]===undefined?"<undefined>":m.groups[k]]),m.indices.groups===undefined?"<undefined>":Object.keys(m.indices.groups).map(k=>[k,m.indices.groups[k]===undefined?"<undefined>":m.indices.groups[k]])])==="[[\"\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],[[0,0],\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],0,0,\"<undefined>\",\"<undefined>\"]""####,
    );
    assert_true(
        r####"var pattern="((((\\b\\B)|(\\b\\B))([\\q{ab}&&\\p{RGI_Emoji}])|())){0}",input="a😀b",r=new RegExp(pattern,"dgvy");r.lastIndex=2;var m=r.exec(input);r.source===pattern&&r.flags==="dgvy"&&r.hasIndices&&r.unicodeSets&&m.input===input&&Object.hasOwn(m,"groups")&&Object.hasOwn(m.indices,"groups")&&m.groups===undefined&&m.indices.groups===undefined&&JSON.stringify([Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex])==="[[\"\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],[[1,1],\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],1,1]""####,
    );
    assert_true(
        r####"var pattern="((((^)|(\\b))([\\q{a|ab}])|z)){0}",input="a😀b",r=new RegExp(pattern,"dgvy");r.lastIndex=2;var m=r.exec(input);r.source===pattern&&r.flags==="dgvy"&&r.hasIndices&&r.unicodeSets&&m.input===input&&Object.hasOwn(m,"groups")&&Object.hasOwn(m.indices,"groups")&&m.groups===undefined&&m.indices.groups===undefined&&JSON.stringify([Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex])==="[[\"\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],[[1,1],\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],1,1]""####,
    );
    assert_true(
        r####"var pattern="((((^)|(\\b))([\\q{a|ab}])|\\1)){0}",input="a😀b",r=new RegExp(pattern,"dgvy");r.lastIndex=2;var m=r.exec(input);r.source===pattern&&r.flags==="dgvy"&&r.hasIndices&&r.unicodeSets&&m.input===input&&Object.hasOwn(m,"groups")&&Object.hasOwn(m.indices,"groups")&&m.groups===undefined&&m.indices.groups===undefined&&JSON.stringify([Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex])==="[[\"\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],[[1,1],\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],1,1]""####,
    );
    assert_true(
        r####"var pattern="((((^)|(\\b))([\\q{a|ab}])|([a]))){0}",input="a😀b",r=new RegExp(pattern,"dgvy");r.lastIndex=2;var m=r.exec(input);r.source===pattern&&r.flags==="dgvy"&&r.hasIndices&&r.unicodeSets&&m.input===input&&Object.hasOwn(m,"groups")&&Object.hasOwn(m.indices,"groups")&&m.groups===undefined&&m.indices.groups===undefined&&JSON.stringify([Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex])==="[[\"\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],[[1,1],\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],1,1]""####,
    );
    {
        let pattern = JsString::from(r"((((\b\B)|(\b\B))([\q{ab}&&\p{RGI_Emoji}])|())){0,1}");
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
    r#"(?:(?<n1>(?<n2>(?<n3>(?<n4>(?<n5>$)|(?<n6>\B))(?<n7>(?<n8>){2,3})(?<n9>(?<n10>\b\B)|(?<n11>\b\B))|(?<n12>(?<n13>^)|(?<n14>$)){3}(?<n15>(?<n16>[a-z\q{ab}]){1}){1,1}?(?<n17>(?<n18>^)|(?<n19>\B)))){0001,0001}?)){0000}?"#,
    "dgvi",
    JsString::from(""),
    1;
    "empty_input_from_1_variant_1"
)]
#[test_case(
    r#"(?:(?:(?<n1>(?<n2>(?<n3>(?<n4>$)|(?<n5>\b)){2,3}?(?<n6>(?<n7>){2,3})(?<n8>(?<n9>\b^)|(?<n10>$))|(?<n11>(?<n12>(?<n13>^)|(?<n14>\b))\B)(?<n15>(?<n16>[\q{|a|ab}])){1}(?<n17>(?<n18>\b\B)|(?<n19>\b\B)))))){0,0}?"#,
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
