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
    realm.eval(r"var r=/^(?<outer>(?<b>(?<s>^)|(?<w>\b))(?<c>[\q{a|ab}])(?<a>(?<end>$)|(?<nw>\B))|(?<fallback>(?<wb>\b)|(?<nwb>\B))(?<empty>))?\B/dvg,clone=new RegExp(r),m=r.exec('ab!aa')").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval(r#"
        var n = clone.exec('ab');
        r.lastIndex = 0;
        var p = r.exec('!');
        m[0] === 'a' && m.groups.outer === 'a' && m.groups.s === '' && m.groups.w === undefined && m
            .groups.c === 'a' && m.groups.fallback === undefined && m.indices.groups.c === m.indices[
            5] && m.indices[5][0] === 0 && m.indices[5][1] === 1 && n.groups.c === 'a' && n.groups
            .outer === 'a' && n.groups.nw === '' && clone.source === r.source && clone.lastIndex ===
            1 && p[0] === '' && p.index === 0 && Array.from(p).slice(1).every(v => v === undefined) &&
            p.indices.slice(1).every(v => v === undefined) && p.groups.outer === undefined && r
            .lastIndex === 0
    "#),Ok(Value::Boolean(true)));
}

#[test]
fn match_all_preserves_capture_metadata() {
    let source = format!(
        r#"
            (function() {{
                {DESCRIBE_MATCH}

                var P =
                    "\\b(?<outer>(?<b>(?<s>^)|(?<w>\\b))(?<whole>(?<c>[\\q{{|a|ab}}]))(?<a>(?<end>$)|(?<nw>\\B))|(?<fallback>(?<wb>\\b)|(?<nwb>\\B))(?<empty>))?$";
                var r = new RegExp(P, "dvg"),
                    a = Array.from("ab!aa".matchAll(r));
                return JSON.stringify([a.map(describeMatch), r.lastIndex], null, 2);
            }})()
        "#
    );
    let report = eval_string(&mut Realm::default(), &source);
    insta::assert_snapshot!(report, @r#"
        [
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
                "<undefined>"
              ],
              "indices": [
                [
                  5,
                  5
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
                "<undefined>"
              ],
              "index": 5,
              "input": "ab!aa",
              "groupNames": [
                "outer",
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
                "<undefined>"
              ],
              "namedAliasesValid": true,
              "groupsPrototypeValid": true
            }
          ],
          0
        ]
        "#);
}

#[test]
fn replacement_callback_receives_captures() {
    let source = r#"
        (function () {
          var P =
            "\\b(?<outer>(?<b>(?<s>^)|(?<w>\\b))(?<whole>(?<c>[\\q{|a|ab}]))(?<a>(?<end>$)|(?<nw>\\B))|(?<fallback>(?<wb>\\b)|(?<nwb>\\B))(?<empty>))?$";
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
    insta::assert_snapshot!(report, @r#"
        [
          "ab!aa[]",
          [
            [
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
              5,
              "ab!aa",
              [
                [
                  "outer",
                  "<undefined>"
                ],
                [
                  "b",
                  "<undefined>"
                ],
                [
                  "s",
                  "<undefined>"
                ],
                [
                  "w",
                  "<undefined>"
                ],
                [
                  "whole",
                  "<undefined>"
                ],
                [
                  "c",
                  "<undefined>"
                ],
                [
                  "a",
                  "<undefined>"
                ],
                [
                  "end",
                  "<undefined>"
                ],
                [
                  "nw",
                  "<undefined>"
                ],
                [
                  "fallback",
                  "<undefined>"
                ],
                [
                  "wb",
                  "<undefined>"
                ],
                [
                  "nwb",
                  "<undefined>"
                ],
                [
                  "empty",
                  "<undefined>"
                ]
              ]
            ]
          ],
          0
        ]
        "#);
}

#[test]
fn split_and_search_preserve_last_index() {
    let source = r#"
        (function() {

            var P =
                "\\b(?<outer>(?<b>(?<s>^)|(?<w>\\b))(?<whole>(?<c>[\\q{|a|ab}]))(?<a>(?<end>$)|(?<nw>\\B))|(?<fallback>(?<wb>\\b)|(?<nwb>\\B))(?<empty>))?$";
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
            "!abaa!"
          ],
          -1,
          9
        ]
        "#);
}

#[test]
fn failed_last_index_write_throws_type_error() {
    let source = r#"
        (function () {
          var P =
            "\\b(?<outer>(?<b>(?<s>^)|(?<w>\\b))(?<whole>(?<c>[\\q{|a|ab}]))(?<a>(?<end>$)|(?<nw>\\B))|(?<fallback>(?<wb>\\b)|(?<nwb>\\B))(?<empty>))?$";
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
fn exec_recovers_after_a_failed_match() {
    let source = format!(
        r#"
            (function() {{
                {DESCRIBE_MATCH}

                var P =
                    "\\b(?<outer>(?<b>(?<s>^)|(?<w>\\b))(?<whole>(?<c>[\\q{{|a|ab}}]))(?<a>(?<end>$)|(?<nw>\\B))|(?<fallback>(?<wb>\\b)|(?<nwb>\\B))(?<empty>))?$";
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
                    "\\b(?<outer>(?<b>(?<s>^)|(?<w>\\b))(?<whole>(?<c>[a-z\\q{{|S|sa|aX}}]))(?<a>(?<end>$)|(?<nw>\\B))|(?<fallback>(?<wb>\\b)|(?<nwb>\\B))(?<empty>))?$";
                var r = new RegExp(P, "dvgi"),
                    a = r.exec("ſaX!");
                return JSON.stringify([describeMatch(a), r.lastIndex], null, 2);
            }})()
        "#
    );
    let report = eval_string(&mut Realm::default(), &source);
    insta::assert_snapshot!(report, @r#"
        [
          null,
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
                    "\\b(?<outer>(?<b>(?<s>^)|(?<w>\\b))(?<whole>(?<c>[\\q{{|a|ab}}]))(?<a>(?<end>$)|(?<nw>\\B))|(?<fallback>(?<wb>\\b)|(?<nwb>\\B))(?<empty>))?$";
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
                    "\\b(?<outer>(?<\\u{{1D49C}}>(?<s>^)|(?<w>\\b))(?<whole>(?<c>[\\q{{|a|ab}}]))(?<a>(?<end>$)|(?<nw>\\B))|(?<fallback>(?<wb>\\b)|(?<nwb>\\B))(?<empty>))?$";
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
          null,
          true,
          true,
          0,
          3
        ]
        "#);
}

#[test_case(
    r#"^(?<n1>(?<n2>(?<n3>^)|(?<n4>\b))(?<n5>[\q{a|ab}])(?<n6>(?<n7>\b^)|(?<n8>$))|(?<n9>(?<n10>^)|(?<n11>\B))(?<n12>(?<n13>){2,3})(?<n14>(?<n15>$)|(?<n16>\B)))?$"#,
    "dgvym",
    JsString::from(""),
    0,
    MatchExpectation {
        text: "",
        range: 0..0,
        captures: &[None; 16],
        capture_ranges: &[None; 16],
        last_index: 0,
    };
    "empty_input_from_0_variant_1"
)]
#[test_case(
    r#"(?:(?<n1>(?<n2>(?<n3>(?<n4>^)|(?<n5>\b))(?<n6>[\p{Any}\q{|ab}]){0001,0001}?(?<n7>(?<n8>\b^)|(?<n9>$))|(?<n10>(?<n11>^)|(?<n12>\B))(?<n13>(?<n14>){2,3})(?<n15>(?<n16>$)|(?<n17>\B))){0,1})){1,1}?$"#,
    "dgv",
    JsString::from("a\n!"),
    0,
    MatchExpectation {
        text: "",
        range: 3..3,
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
            Some((3, 3)),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
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
        last_index: 3,
    };
    "input_a_from_0_variant_2"
)]
#[test_case(
    r#"(?<n1>(?:(?<n2>(?<n3>(?<n4>^)|(?<n5>\b))\B)(?<n6>(?<n7>){2,3})(?<n8>(?<n9>\b^)|(?<n10>\B$))|(?<n11>(?<n12>$)|(?<n13>\b)){2,3}?(?<n14>(?<n15>[\q{|a|ab}])){1}(?<n16>(?<n17>\B^)|(?<n18>\b$)))??)$"#,
    "dgv",
    JsString::from("!ab!"),
    0,
    MatchExpectation {
        text: "",
        range: 4..4,
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
            Some((4, 4)),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
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
        last_index: 4,
    };
    "input_ab_from_0_variant_3"
)]
#[test_case(
    r#"(?<n1>(?<n2>(?<n3>^)|(?<n4>$)){3}(?<n5>[\q{a|ab}])(?<n6>(?<n7>^)|(?<n8>\B))|(?<n9>(?<n10>$)|(?<n11>\B))(?<n12>(?<n13>){2,3})(?<n14>(?<n15>\b\B)|(?<n16>\b\B)))?$"#,
    "dgv",
    JsString::from("ab"),
    0,
    MatchExpectation {
        text: "",
        range: 2..2,
        captures: &[None; 16],
        capture_ranges: &[None; 16],
        last_index: 2,
    };
    "input_ab_from_0_variant_4"
)]
#[test_case(
    r#"(?<n1>(?:(?:(?<n2>(?<n3>\b^)|(?<n4>\B$))(?<n5>(?<n6>){2,3})(?<n7>(?<n8>^)|(?<n9>\b))|(?<n10>(?<n11>\b^)|(?<n12>$))(?<n13>(?<n14>[a-z\q{ab}]){1}){1,1}?(?<n15>(?<n16>(?<n17>^)|(?<n18>\b))\B)){0000,0001}?){0001})$"#,
    "dgvm",
    JsString::from("a"),
    0,
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
    "input_a_from_0_variant_5"
)]
#[test_case(
    r#"(?:(?<n1>(?<n2>(?<n3>(?<n4>\B^)|(?<n5>\b$))(?<n6>[\p{Any}\q{|ab}]){0001,0001}?(?<n7>(?<n8>$)|(?<n9>\B))|(?<n10>(?<n11>\b\B)|(?<n12>\b\B))(?<n13>(?<n14>){2,3})(?<n15>(?<n16>$)|(?<n17>\b)){2,3}?){0,1})){1,1}?$"#,
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
    "empty_input_from_0_variant_6"
)]
#[test_case(
    r#"(?<n1>(?:(?<n2>(?<n3>^)|(?<n4>\b))(?<n5>(?<n6>){2,3})(?<n7>(?<n8>^)|(?<n9>$)){3}|(?<n10>(?<n11>^)|(?<n12>\B))(?<n13>(?<n14>[\q{|a|ab}])){1}(?<n15>(?<n16>\b^)|(?<n17>\B$)))??)$"#,
    "dgvm",
    JsString::from("ab\r\n"),
    0,
    MatchExpectation {
        text: "",
        range: 2..2,
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
            Some((2, 2)),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
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
        last_index: 2,
    };
    "input_ab_from_0_variant_7"
)]
#[test_case(
    r#"(?<n1>(?<n2>(?<n3>(?<n4>^)|(?<n5>\b))\B)(?<n6>[\q{a|ab}])(?<n7>(?<n8>\b\B)|(?<n9>\b\B))|(?<n10>(?<n11>$)|(?<n12>\b)){2,3}?(?<n13>(?<n14>){2,3})(?<n15>(?<n16>\b^)|(?<n17>$)))?$"#,
    "dgvm",
    JsString::from("😀a"),
    0,
    MatchExpectation {
        text: "",
        range: 3..3,
        captures: &[None; 17],
        capture_ranges: &[None; 17],
        last_index: 3,
    };
    "unicode_input_from_0_variant_8"
)]
#[test_case(
    r#"(?<n1>(?:(?:(?<n2>(?<n3>^)|(?<n4>$)){3}(?<n5>(?<n6>){2,3})(?<n7>(?<n8>\B^)|(?<n9>\b$))|(?<n10>(?<n11>$)|(?<n12>\B))(?<n13>(?<n14>[a-z\q{ab}]){1}){1,1}?(?<n15>(?<n16>^)|(?<n17>\b))){0000,0001}?){0001})$"#,
    "dgv",
    JsString::from("ſa"),
    0,
    MatchExpectation {
        text: "",
        range: 2..2,
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
            Some((2, 2)),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
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
        last_index: 2,
    };
    "unicode_input_from_0_variant_9"
)]
#[test_case(
    r#"^(?<n1>(?<n2>(?<n3>\b^)|(?<n4>\B$))(?<n5>[\q{a|ab}])(?<n6>(?<n7>$)|(?<n8>\b)){2,3}?|(?<n9>(?<n10>\b^)|(?<n11>$))(?<n12>(?<n13>){2,3})(?<n14>(?<n15>^)|(?<n16>\B)))?$"#,
    "dgvy",
    JsString::from(""),
    0,
    MatchExpectation {
        text: "",
        range: 0..0,
        captures: &[None; 16],
        capture_ranges: &[None; 16],
        last_index: 0,
    };
    "empty_input_from_0_variant_10"
)]
#[test_case(
    r#"(?:(?<n1>(?<n2>(?<n3>(?<n4>\b^)|(?<n5>\B$))(?<n6>[\p{Any}\q{|ab}]){0001,0001}?(?<n7>(?<n8>$)|(?<n9>\b)){2,3}?|(?<n10>(?<n11>\b^)|(?<n12>$))(?<n13>(?<n14>){2,3})(?<n15>(?<n16>^)|(?<n17>\B))){0,1})){1,1}?$"#,
    "dgv",
    JsString::from("a\n!"),
    0,
    MatchExpectation {
        text: "",
        range: 3..3,
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
            Some((3, 3)),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
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
        last_index: 3,
    };
    "input_a_from_0_variant_11"
)]
#[test_case(
    r#"(?<n1>(?:(?<n2>(?<n3>\B^)|(?<n4>\b$))(?<n5>(?<n6>){2,3})(?<n7>(?<n8>(?<n9>^)|(?<n10>\b))\B)|(?<n11>(?<n12>\b\B)|(?<n13>\b\B))(?<n14>(?<n15>[\q{|a|ab}])){1}(?<n16>(?<n17>^)|(?<n18>$)){3})??)$"#,
    "dgv",
    JsString::from("!ab!"),
    0,
    MatchExpectation {
        text: "",
        range: 4..4,
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
            Some((4, 4)),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
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
        last_index: 4,
    };
    "input_ab_from_0_variant_12"
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
fn truth_table_surrounding_optional_alternative_string_classes_validate_unknowns_before_zero_shortcuts()
 {
    assert_true(
        r####"var pattern="(?<outside>^)\\b(((^)|(\\b))([\\q{a|ab}])|())??$",input="aba",r=new RegExp(pattern,"dv"),m=r.exec(input);r.source===pattern&&r.flags==="dv"&&r.hasIndices&&r.unicodeSets&&(m===null||(m.input===input&&Object.hasOwn(m,"groups")&&Object.hasOwn(m.indices,"groups")&&Object.getPrototypeOf(m.groups)===null&&Object.getPrototypeOf(m.indices.groups)===null&&m.groups["outside"]===m[1]&&m.indices.groups["outside"]===m.indices[1]))&&JSON.stringify(m===null?null:[Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex,m.groups===undefined?"<undefined>":Object.keys(m.groups).map(k=>[k,m.groups[k]===undefined?"<undefined>":m.groups[k]]),m.indices.groups===undefined?"<undefined>":Object.keys(m.indices.groups).map(k=>[k,m.indices.groups[k]===undefined?"<undefined>":m.indices.groups[k]])])==="null""####,
    );
    for pattern in [
        r"\b\B(((\b\B)|(\b\B))([\q{ab}&&\p{RGI_Emoji}])|())??$",
        r"^(((^)|(\b))([\q{a|ab}])|z)??$",
        r"\b\B(((^)|(\b))([\q{a|ab}])|\1)??$",
        r"^(((^)|(\b))([\q{a|ab}])|([a]))?$",
        r"^((((^)|(\b))([\q{a|ab}])|())?){2}$",
        r"^((((^)|(\b))([\q{a|ab}])|())?){0,1}$",
        r"^(((^)|(\b))([\q{a|ab}])|())?z$",
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
    r#"\b(?<n1>(?:(?<n2>(?<n3>^)|(?<n4>\B))(?<n5>(?<n6>){2,3})(?<n7>(?<n8>$)|(?<n9>\B))|(?<n10>(?<n11>^)|(?<n12>\b))(?<n13>(?<n14>[\q{|a|ab}])){1}(?<n15>(?<n16>\b^)|(?<n17>$)))??)\B"#,
    "dgvi",
    JsString::from("ab"),
    1;
    "input_ab_from_1_variant_1"
)]
#[test_case(
    r#"$(?<n1>(?:(?:(?<n2>(?<n3>^)|(?<n4>\B))(?<n5>(?<n6>){2,3})(?<n7>(?<n8>$)|(?<n9>\B))|(?<n10>(?<n11>^)|(?<n12>\b))(?<n13>(?<n14>[a-z\q{ab}]){1}){1,1}?(?<n15>(?<n16>\b^)|(?<n17>$))){0000,0001}?){0001})^"#,
    "dgvim",
    JsString::from("😀a"),
    1;
    "unicode_input_from_1_variant_2"
)]
#[test_case(
    r#"\b(?<n1>(?<n2>(?<n3>$)|(?<n4>\b)){2,3}?(?<n5>[\q{a|ab}])(?<n6>(?<n7>\B^)|(?<n8>\b$))|(?<n9>(?<n10>(?<n11>^)|(?<n12>\b))\B)(?<n13>(?<n14>){2,3})(?<n15>(?<n16>\b^)|(?<n17>\B$)))?\B"#,
    "dgvi",
    JsString::from("a"),
    1;
    "input_a_from_1_variant_3"
)]
#[test_case(
    r#"$(?:(?<n1>(?<n2>(?<n3>(?<n4>$)|(?<n5>\b)){2,3}?(?<n6>[\p{Any}\q{|ab}]){0001,0001}?(?<n7>(?<n8>\B^)|(?<n9>\b$))|(?<n10>(?<n11>(?<n12>^)|(?<n13>\b))\B)(?<n14>(?<n15>){2,3})(?<n16>(?<n17>\b^)|(?<n18>\B$))){0,1})){1,1}?^"#,
    "dgvim",
    JsString::from("ſa"),
    1;
    "unicode_input_from_1_variant_4"
)]
#[test_case(
    r#"^(?<n1>(?:(?:(?<n2>(?<n3>(?<n4>^)|(?<n5>\b))\B)(?<n6>(?<n7>){2,3})(?<n8>(?<n9>\b^)|(?<n10>\B$))|(?<n11>(?<n12>$)|(?<n13>\b)){2,3}?(?<n14>(?<n15>[a-z\q{ab}]){1}){1,1}?(?<n16>(?<n17>\B^)|(?<n18>\b$))){0000,0001}?){0001})$"#,
    "dgvy",
    JsString::from("ab\r\n"),
    0;
    "input_ab_from_0_variant_5"
)]
#[test_case(
    r#"$(?<n1>(?:(?<n2>(?<n3>$)|(?<n4>\B))(?<n5>(?<n6>){2,3})(?<n7>(?<n8>\b\B)|(?<n9>\b\B))|(?<n10>(?<n11>^)|(?<n12>$)){3}(?<n13>(?<n14>[\q{|a|ab}])){1}(?<n15>(?<n16>^)|(?<n17>\B)))??)^"#,
    "dgvim",
    JsString::from("a\n!"),
    1;
    "input_a_from_1_variant_6"
)]
#[test_case(
    r#"^(?:(?<n1>(?<n2>(?<n3>(?<n4>^)|(?<n5>$)){3}(?<n6>[\p{Any}\q{|ab}]){0001,0001}?(?<n7>(?<n8>^)|(?<n9>\B))|(?<n10>(?<n11>$)|(?<n12>\B))(?<n13>(?<n14>){2,3})(?<n15>(?<n16>\b\B)|(?<n17>\b\B))){0,1})){1,1}?$"#,
    "dgvy",
    JsString::from("😀a"),
    0;
    "unicode_input_from_0_variant_7"
)]
#[test_case(
    r#"\b(?<n1>(?:(?:(?<n2>(?<n3>$)|(?<n4>\B))(?<n5>(?<n6>){2,3})(?<n7>(?<n8>\b\B)|(?<n9>\b\B))|(?<n10>(?<n11>^)|(?<n12>$)){3}(?<n13>(?<n14>[a-z\q{ab}]){1}){1,1}?(?<n15>(?<n16>^)|(?<n17>\B))){0000,0001}?){0001})\B"#,
    "dgvi",
    JsString::from(""),
    1;
    "empty_input_from_1_variant_8"
)]
#[test_case(
    r#"$(?<n1>(?<n2>(?<n3>\b^)|(?<n4>$))(?<n5>[\q{a|ab}])(?<n6>(?<n7>(?<n8>^)|(?<n9>\b))\B)|(?<n10>(?<n11>\b^)|(?<n12>\B$))(?<n13>(?<n14>){2,3})(?<n15>(?<n16>^)|(?<n17>\b)))?^"#,
    "dgvim",
    JsString::from("!ab!"),
    1;
    "input_ab_from_1_variant_9"
)]
#[test_case(
    r#"^(?<n1>(?:(?<n2>(?<n3>\b^)|(?<n4>\B$))(?<n5>(?<n6>){2,3})(?<n7>(?<n8>^)|(?<n9>\b))|(?<n10>(?<n11>\b^)|(?<n12>$))(?<n13>(?<n14>[\q{|a|ab}])){1}(?<n15>(?<n16>(?<n17>^)|(?<n18>\b))\B))??)$"#,
    "dgvy",
    JsString::from("ſa"),
    0;
    "unicode_input_from_0_variant_10"
)]
#[test_case(
    r#"\b(?:(?<n1>(?<n2>(?<n3>(?<n4>\b^)|(?<n5>$))(?<n6>[\p{Any}\q{|ab}]){0001,0001}?(?<n7>(?<n8>(?<n9>^)|(?<n10>\b))\B)|(?<n11>(?<n12>\b^)|(?<n13>\B$))(?<n14>(?<n15>){2,3})(?<n16>(?<n17>^)|(?<n18>\b))){0,1})){1,1}?\B"#,
    "dgvi",
    JsString::from("ab\r\n"),
    1;
    "input_ab_from_1_variant_11"
)]
#[test_case(
    r#"^(?<n1>(?<n2>(?<n3>\B^)|(?<n4>\b$))(?<n5>[\q{a|ab}])(?<n6>(?<n7>$)|(?<n8>\B))|(?<n9>(?<n10>\b\B)|(?<n11>\b\B))(?<n12>(?<n13>){2,3})(?<n14>(?<n15>$)|(?<n16>\b)){2,3}?)?$"#,
    "dgvy",
    JsString::from("a\n!"),
    0;
    "input_a_from_0_variant_12"
)]
#[test_case(
    r#"\b(?<n1>(?:(?<n2>(?<n3>\b\B)|(?<n4>\b\B))(?<n5>(?<n6>){2,3})(?<n7>(?<n8>$)|(?<n9>\b)){2,3}?|(?<n10>(?<n11>\B^)|(?<n12>\b$))(?<n13>(?<n14>[\q{|a|ab}])){1}(?<n15>(?<n16>$)|(?<n17>\B)))??)\B"#,
    "dgvi",
    JsString::from("😀a"),
    1;
    "unicode_input_from_1_variant_13"
)]
#[test_case(
    r#"$(?<n1>(?:(?:(?<n2>(?<n3>\b\B)|(?<n4>\b\B))(?<n5>(?<n6>){2,3})(?<n7>(?<n8>$)|(?<n9>\b)){2,3}?|(?<n10>(?<n11>\B^)|(?<n12>\b$))(?<n13>(?<n14>[a-z\q{ab}]){1}){1,1}?(?<n15>(?<n16>$)|(?<n17>\B))){0000,0001}?){0001})^"#,
    "dgvi",
    JsString::from("ab"),
    1;
    "input_ab_from_1_variant_14"
)]
#[test_case(
    r#"\b(?<n1>(?<n2>(?<n3>^)|(?<n4>\B))(?<n5>[\q{a|ab}])(?<n6>(?<n7>\b^)|(?<n8>\B$))|(?<n9>(?<n10>^)|(?<n11>\b))(?<n12>(?<n13>){2,3})(?<n14>(?<n15>^)|(?<n16>$)){3})?\B"#,
    "dgvi",
    JsString::from("ſa"),
    1;
    "unicode_input_from_1_variant_15"
)]
#[test_case(
    r#"$(?:(?<n1>(?<n2>(?<n3>(?<n4>^)|(?<n5>\B))(?<n6>[\p{Any}\q{|ab}]){0001,0001}?(?<n7>(?<n8>\b^)|(?<n9>\B$))|(?<n10>(?<n11>^)|(?<n12>\b))(?<n13>(?<n14>){2,3})(?<n15>(?<n16>^)|(?<n17>$)){3}){0,1})){1,1}?^"#,
    "dgvi",
    JsString::from("a"),
    1;
    "input_a_from_1_variant_16"
)]
#[test_case(
    r#"^(?<n1>(?:(?:(?<n2>(?<n3>^)|(?<n4>\b))(?<n5>(?<n6>){2,3})(?<n7>(?<n8>^)|(?<n9>$)){3}|(?<n10>(?<n11>^)|(?<n12>\B))(?<n13>(?<n14>[a-z\q{ab}]){1}){1,1}?(?<n15>(?<n16>\b^)|(?<n17>\B$))){0000,0001}?){0001})$"#,
    "dgvy",
    JsString::from("!ab!"),
    0;
    "input_ab_from_0_variant_17"
)]
#[test_case(
    r#"$(?<n1>(?:(?<n2>(?<n3>$)|(?<n4>\b)){2,3}?(?<n5>(?<n6>){2,3})(?<n7>(?<n8>\b^)|(?<n9>$))|(?<n10>(?<n11>(?<n12>^)|(?<n13>\b))\B)(?<n14>(?<n15>[\q{|a|ab}])){1}(?<n16>(?<n17>\b\B)|(?<n18>\b\B)))??)^"#,
    "dgvi",
    JsString::from(""),
    1;
    "empty_input_from_1_variant_18"
)]
#[test_case(
    r#"^(?:(?<n1>(?<n2>(?<n3>(?<n4>(?<n5>^)|(?<n6>\b))\B)(?<n7>[\p{Any}\q{|ab}]){0001,0001}?(?<n8>(?<n9>\b\B)|(?<n10>\b\B))|(?<n11>(?<n12>$)|(?<n13>\b)){2,3}?(?<n14>(?<n15>){2,3})(?<n16>(?<n17>\b^)|(?<n18>$))){0,1})){1,1}?$"#,
    "dgvy",
    JsString::from("ab"),
    0;
    "input_ab_from_0_variant_19"
)]
#[test_case(
    r#"\b(?<n1>(?:(?:(?<n2>(?<n3>$)|(?<n4>\b)){2,3}?(?<n5>(?<n6>){2,3})(?<n7>(?<n8>\b^)|(?<n9>$))|(?<n10>(?<n11>(?<n12>^)|(?<n13>\b))\B)(?<n14>(?<n15>[a-z\q{ab}]){1}){1,1}?(?<n16>(?<n17>\b\B)|(?<n18>\b\B))){0000,0001}?){0001})\B"#,
    "dgvim",
    JsString::from("a\n!"),
    1;
    "input_a_from_1_variant_20"
)]
#[test_case(
    r#"$(?<n1>(?<n2>(?<n3>$)|(?<n4>\B))(?<n5>[\q{a|ab}])(?<n6>(?<n7>^)|(?<n8>\b))|(?<n9>(?<n10>^)|(?<n11>$)){3}(?<n12>(?<n13>){2,3})(?<n14>(?<n15>\B^)|(?<n16>\b$)))?^"#,
    "dgvi",
    JsString::from("ab\r\n"),
    1;
    "input_ab_from_1_variant_21"
)]
#[test_case(
    r#"^(?<n1>(?:(?<n2>(?<n3>^)|(?<n4>$)){3}(?<n5>(?<n6>){2,3})(?<n7>(?<n8>\B^)|(?<n9>\b$))|(?<n10>(?<n11>$)|(?<n12>\B))(?<n13>(?<n14>[\q{|a|ab}])){1}(?<n15>(?<n16>^)|(?<n17>\b)))??)$"#,
    "dgvy",
    JsString::from("a"),
    0;
    "input_a_from_0_variant_22"
)]
#[test_case(
    r#"\b(?:(?<n1>(?<n2>(?<n3>(?<n4>$)|(?<n5>\B))(?<n6>[\p{Any}\q{|ab}]){0001,0001}?(?<n7>(?<n8>^)|(?<n9>\b))|(?<n10>(?<n11>^)|(?<n12>$)){3}(?<n13>(?<n14>){2,3})(?<n15>(?<n16>\B^)|(?<n17>\b$))){0,1})){1,1}?\B"#,
    "dgvim",
    JsString::from("!ab!"),
    1;
    "input_ab_from_1_variant_23"
)]
#[test_case(
    r#"\b(?<n1>(?:(?<n2>(?<n3>\b^)|(?<n4>$))(?<n5>(?<n6>){2,3})(?<n7>(?<n8>^)|(?<n9>\B))|(?<n10>(?<n11>\b^)|(?<n12>\B$))(?<n13>(?<n14>[\q{|a|ab}])){1}(?<n15>(?<n16>$)|(?<n17>\b)){2,3}?)??)\B"#,
    "dgvim",
    JsString::from("ab"),
    1;
    "input_ab_from_1_variant_24"
)]
#[test_case(
    r#"$(?<n1>(?:(?:(?<n2>(?<n3>\b^)|(?<n4>$))(?<n5>(?<n6>){2,3})(?<n7>(?<n8>^)|(?<n9>\B))|(?<n10>(?<n11>\b^)|(?<n12>\B$))(?<n13>(?<n14>[a-z\q{ab}]){1}){1,1}?(?<n15>(?<n16>$)|(?<n17>\b)){2,3}?){0000,0001}?){0001})^"#,
    "dgvi",
    JsString::from("😀a"),
    1;
    "unicode_input_from_1_variant_25"
)]
#[test_case(
    r#"\b(?<n1>(?<n2>(?<n3>\b\B)|(?<n4>\b\B))(?<n5>[\q{a|ab}])(?<n6>(?<n7>^)|(?<n8>$)){3}|(?<n9>(?<n10>\B^)|(?<n11>\b$))(?<n12>(?<n13>){2,3})(?<n14>(?<n15>(?<n16>^)|(?<n17>\b))\B))?\B"#,
    "dgvim",
    JsString::from("a"),
    1;
    "input_a_from_1_variant_26"
)]
#[test_case(
    r#"$(?:(?<n1>(?<n2>(?<n3>(?<n4>\b\B)|(?<n5>\b\B))(?<n6>[\p{Any}\q{|ab}]){0001,0001}?(?<n7>(?<n8>^)|(?<n9>$)){3}|(?<n10>(?<n11>\B^)|(?<n12>\b$))(?<n13>(?<n14>){2,3})(?<n15>(?<n16>(?<n17>^)|(?<n18>\b))\B)){0,1})){1,1}?^"#,
    "dgvi",
    JsString::from("ſa"),
    1;
    "unicode_input_from_1_variant_27"
)]
#[test_case(
    r#"^(?<n1>(?:(?:(?<n2>(?<n3>\B^)|(?<n4>\b$))(?<n5>(?<n6>){2,3})(?<n7>(?<n8>(?<n9>^)|(?<n10>\b))\B)|(?<n11>(?<n12>\b\B)|(?<n13>\b\B))(?<n14>(?<n15>[a-z\q{ab}]){1}){1,1}?(?<n16>(?<n17>^)|(?<n18>$)){3}){0000,0001}?){0001})$"#,
    "dgvym",
    JsString::from("ab\r\n"),
    0;
    "input_ab_from_0_variant_28"
)]
fn no_match_resets_last_index(
    pattern: &str,
    flags: &str,
    input: JsString,
    initial_last_index: usize,
) {
    assert_no_match(pattern, flags, input, initial_last_index);
}
