//! Required predicate choices preserve conditional lexical captures.
use crate::common::{assert_true_in, eval_string};

use crate::common::regexp_captures::{MatchExpectation, assert_named_captures, assert_no_match};

use crate::common::regexp_matches::DESCRIBE_MATCH;
use spite_core::JsString;
use spite_runtime::Realm;
use test_case::test_case;
#[test]
fn capture_metadata_survives_collection() {
    let mut realm = Realm::default();
    realm
        .eval(
            r#"
                var regexp = /(?<b>(?<w>\b)|(?<nw>\B))(?<c>[\q{a|ab}])+(?<a>(?<end>$)|())/dvg,
                    copy = new RegExp(regexp),
                    match = regexp.exec('ababa')
            "#,
        )
        .unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_true_in(
        &mut realm,
        r#"
            var copiedMatch = copy.exec("a!");
            match[0] === "ababa" &&
                match.groups.w === "" &&
                match.groups.nw === undefined &&
                match.groups.c === "a" &&
                match.groups.end === "" &&
                match.indices.groups.c === match.indices[4] &&
                match.indices[4][0] === 4 &&
                copiedMatch.groups.end === undefined &&
                copiedMatch.groups.a === "" &&
                copy.source === regexp.source &&
                copy.lastIndex === 1 &&
                regexp.lastIndex === 5;
        "#,
    );
}

#[test]
fn match_all_preserves_capture_metadata() {
    let source = format!(
        r#"
            (function() {{
                {DESCRIBE_MATCH}

                var P =
                    "(?<b>(?<w>\\b)|(?<nw>\\B))(?<whole>(?<it>(?<c>[\\q{{|a|ab}}])|(?<e>))+)(?<a>(?<s>^)|(?<end>$)|(?<wb>\\b)|(?<nwb>\\B))";
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
            "(?<b>(?<w>\\b)|(?<nw>\\B))(?<whole>(?<it>(?<c>[\\q{|a|ab}])|(?<e>))+)(?<a>(?<s>^)|(?<end>$)|(?<wb>\\b)|(?<nwb>\\B))";
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
                "(?<b>(?<w>\\b)|(?<nw>\\B))(?<whole>(?<it>(?<c>[\\q{|a|ab}])|(?<e>))+)(?<a>(?<s>^)|(?<end>$)|(?<wb>\\b)|(?<nwb>\\B))";
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
            "",
            "",
            "<undefined>",
            "abaa",
            "a",
            "a",
            "<undefined>",
            "",
            "<undefined>",
            "<undefined>",
            "",
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
            "(?<b>(?<w>\\b)|(?<nw>\\B))(?<whole>(?<it>(?<c>[\\q{|a|ab}])|(?<e>))+)(?<a>(?<s>^)|(?<end>$)|(?<wb>\\b)|(?<nwb>\\B))";
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
                    "(?<b>(?<w>\\b)|(?<nw>\\B))(?<whole>(?<it>(?<c>[\\q{{|a|ab}}])|(?<e>))+)(?<a>(?<s>^)|(?<end>$)|(?<wb>\\b)|(?<nwb>\\B))";
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
                    "(?<b>(?<w>\\b)|(?<nw>\\B))(?<whole>(?<it>(?<c>[a-z\\q{{|S|sa|aX}}])|(?<e>))+)(?<a>(?<s>^)|(?<end>$)|(?<wb>\\b)|(?<nwb>\\B))";
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
                    "(?<b>(?<w>\\b)|(?<nw>\\B))(?<whole>(?<it>(?<c>[\\q{{|a|ab}}])|(?<e>))+)(?<a>(?<s>^)|(?<end>$)|(?<wb>\\b)|(?<nwb>\\B))";
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
fn exec_preserves_capture_metadata() {
    let source = format!(
        r#"
            (function() {{
                {DESCRIBE_MATCH}

                var P =
                    "(?<b>(?<w>\\b)|(?<nw>\\B))(?<whole>(?<it>(?<c>[\\q{{|a|ab}}])|(?<e>))+)(?<a>(?<s>^)|(?<end>$)|(?<wb>\\b)|(?<nwb>\\B))";
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
    r#"(?<n1>(?<n2>\b)|(?<n3>\B))(?<n4>(?<n5>[\q{a|ab}])|(?<n6>))+?(?<n7>^|(?<n8>))"#,
    "dgvi",
    JsString::from("ab"),
    1,
    MatchExpectation {
        text: "",
        range: 1..1,
        captures: &[
            Some(""),
            None,
            Some(""),
            Some(""),
            None,
            Some(""),
            Some(""),
            Some(""),
        ],
        capture_ranges: &[
            Some((1, 1)),
            None,
            Some((1, 1)),
            Some((1, 1)),
            None,
            Some((1, 1)),
            Some((1, 1)),
            Some((1, 1)),
        ],
        last_index: 1,
    };
    "input_ab_from_1_variant_1"
)]
#[test_case(
    r#"(?<n1>(?<n2>\b)|(?<n3>\B))(?<n4>(?<n5>)|(?<n6>[\q{|a|ab}])){2,3}(?<n7>^|(?<n8>))"#,
    "dgv",
    JsString::from("a\n!"),
    0,
    MatchExpectation {
        text: "a",
        range: 0..1,
        captures: &[
            Some(""),
            Some(""),
            None,
            Some("a"),
            None,
            Some("a"),
            Some(""),
            Some(""),
        ],
        capture_ranges: &[
            Some((0, 0)),
            Some((0, 0)),
            None,
            Some((0, 1)),
            None,
            Some((0, 1)),
            Some((1, 1)),
            Some((1, 1)),
        ],
        last_index: 1,
    };
    "input_a_from_0_variant_2"
)]
#[test_case(
    r#"(?<n1>(?<n2>\b)|(?<n3>\B))(?<n4>[a-z\q{ab}]){1,3}(?<n5>^|(?<n6>))"#,
    "dgvim",
    JsString::from("😀a"),
    1,
    MatchExpectation {
        text: "a",
        range: 2..3,
        captures: &[Some(""), Some(""), None, Some("a"), Some(""), Some("")],
        capture_ranges: &[
            Some((2, 2)),
            Some((2, 2)),
            None,
            Some((2, 3)),
            Some((3, 3)),
            Some((3, 3)),
        ],
        last_index: 3,
    };
    "unicode_input_from_1_variant_3"
)]
#[test_case(
    r#"(?<n1>(?<n2>\B)|(?<n3>\b)){1}(?<n4>(?<n5>[\q{a|ab}])|(?<n6>))+?(?<n7>$|(?<n8>))"#,
    "dgv",
    JsString::from("!ab!"),
    0,
    MatchExpectation {
        text: "",
        range: 0..0,
        captures: &[
            Some(""),
            Some(""),
            None,
            Some(""),
            None,
            Some(""),
            Some(""),
            Some(""),
        ],
        capture_ranges: &[
            Some((0, 0)),
            Some((0, 0)),
            None,
            Some((0, 0)),
            None,
            Some((0, 0)),
            Some((0, 0)),
            Some((0, 0)),
        ],
        last_index: 0,
    };
    "input_ab_from_0_variant_4"
)]
#[test_case(
    r#"(?<n1>(?<n2>\B)|(?<n3>\b)){1}(?<n4>(?<n5>)|(?<n6>[\q{|a|ab}])){2,3}(?<n7>$|(?<n8>))"#,
    "dgvim",
    JsString::from("ſa"),
    1,
    MatchExpectation {
        text: "a",
        range: 1..2,
        captures: &[
            Some(""),
            Some(""),
            None,
            Some("a"),
            None,
            Some("a"),
            Some(""),
            None,
        ],
        capture_ranges: &[
            Some((1, 1)),
            Some((1, 1)),
            None,
            Some((1, 2)),
            None,
            Some((1, 2)),
            Some((2, 2)),
            None,
        ],
        last_index: 2,
    };
    "unicode_input_from_1_variant_5"
)]
#[test_case(
    r#"(?<n1>(?<n2>\B)|(?<n3>\b)){1}(?<n4>[a-z\q{ab}]){1,3}(?<n5>$|(?<n6>))"#,
    "dgvy",
    JsString::from("ab\r\n"),
    0,
    MatchExpectation {
        text: "ab",
        range: 0..2,
        captures: &[Some(""), None, Some(""), Some("ab"), Some(""), Some("")],
        capture_ranges: &[
            Some((0, 0)),
            None,
            Some((0, 0)),
            Some((0, 2)),
            Some((2, 2)),
            Some((2, 2)),
        ],
        last_index: 2,
    };
    "input_ab_from_0_variant_6"
)]
#[test_case(
    r#"(?<n1>\b|(?<n2>)){2,3}?(?<n3>[\q{a|ab}])+(?<n4>(?<n5>^)|(?<n6>$)|(?<n7>\b)|(?<n8>\B))"#,
    "dgv",
    JsString::from("ab"),
    0,
    MatchExpectation {
        text: "ab",
        range: 0..2,
        captures: &[
            Some(""),
            None,
            Some("ab"),
            Some(""),
            None,
            Some(""),
            None,
            None,
        ],
        capture_ranges: &[
            Some((0, 0)),
            None,
            Some((0, 2)),
            Some((2, 2)),
            None,
            Some((2, 2)),
            None,
            None,
        ],
        last_index: 2,
    };
    "input_ab_from_0_variant_7"
)]
#[test_case(
    r#"(?<n1>\b|(?<n2>)){2,3}?(?<n3>(?<n4>[\q{a|ab}])|(?<n5>))+?(?<n6>(?<n7>^)|(?<n8>$)|(?<n9>\b)|(?<n10>\B))"#,
    "dgvim",
    JsString::from("a\n!"),
    1,
    MatchExpectation {
        text: "",
        range: 1..1,
        captures: &[
            Some(""),
            None,
            Some(""),
            None,
            Some(""),
            Some(""),
            None,
            Some(""),
            None,
            None,
        ],
        capture_ranges: &[
            Some((1, 1)),
            None,
            Some((1, 1)),
            None,
            Some((1, 1)),
            Some((1, 1)),
            None,
            Some((1, 1)),
            None,
            None,
        ],
        last_index: 1,
    };
    "input_a_from_1_variant_8"
)]
#[test_case(
    r#"(?<n1>\b|(?<n2>)){2,3}?(?<n3>(?<n4>)|(?<n5>[\q{|a|ab}])){2,3}(?<n6>(?<n7>^)|(?<n8>$)|(?<n9>\b)|(?<n10>\B))"#,
    "dgvy",
    JsString::from("😀a"),
    0,
    MatchExpectation {
        text: "",
        range: 0..0,
        captures: &[
            Some(""),
            Some(""),
            Some(""),
            Some(""),
            None,
            Some(""),
            Some(""),
            None,
            None,
            None,
        ],
        capture_ranges: &[
            Some((0, 0)),
            Some((0, 0)),
            Some((0, 0)),
            Some((0, 0)),
            None,
            Some((0, 0)),
            Some((0, 0)),
            None,
            None,
            None,
        ],
        last_index: 0,
    };
    "unicode_input_from_0_variant_9"
)]
#[test_case(
    r#"(?<n1>^|(?<n2>))(?<n3>[\q{a|ab}])+(?<n4>(?<n5>(?<n6>\b)|(?<n7>\B)){2,3})"#,
    "dgvim",
    JsString::from("!ab!"),
    1,
    MatchExpectation {
        text: "ab",
        range: 1..3,
        captures: &[
            Some(""),
            Some(""),
            Some("ab"),
            Some(""),
            Some(""),
            Some(""),
            None,
        ],
        capture_ranges: &[
            Some((1, 1)),
            Some((1, 1)),
            Some((1, 3)),
            Some((3, 3)),
            Some((3, 3)),
            Some((3, 3)),
            None,
        ],
        last_index: 3,
    };
    "input_ab_from_1_variant_10"
)]
#[test_case(
    r#"(?<n1>^|(?<n2>))(?<n3>(?<n4>[\q{a|ab}])|(?<n5>))+?(?<n6>(?<n7>(?<n8>\b)|(?<n9>\B)){2,3})"#,
    "dgvy",
    JsString::from("ſa"),
    0,
    MatchExpectation {
        text: "",
        range: 0..0,
        captures: &[
            Some(""),
            None,
            Some(""),
            None,
            Some(""),
            Some(""),
            Some(""),
            None,
            Some(""),
        ],
        capture_ranges: &[
            Some((0, 0)),
            None,
            Some((0, 0)),
            None,
            Some((0, 0)),
            Some((0, 0)),
            Some((0, 0)),
            None,
            Some((0, 0)),
        ],
        last_index: 0,
    };
    "unicode_input_from_0_variant_11"
)]
#[test_case(
    r#"(?<n1>^|(?<n2>))(?<n3>(?<n4>)|(?<n5>[\q{|a|ab}])){2,3}(?<n6>(?<n7>(?<n8>\b)|(?<n9>\B)){2,3})"#,
    "dgvi",
    JsString::from("ab\r\n"),
    1,
    MatchExpectation {
        text: "",
        range: 1..1,
        captures: &[
            Some(""),
            Some(""),
            Some(""),
            Some(""),
            None,
            Some(""),
            Some(""),
            None,
            Some(""),
        ],
        capture_ranges: &[
            Some((1, 1)),
            Some((1, 1)),
            Some((1, 1)),
            Some((1, 1)),
            None,
            Some((1, 1)),
            Some((1, 1)),
            None,
            Some((1, 1)),
        ],
        last_index: 1,
    };
    "input_ab_from_1_variant_12"
)]
#[test_case(
    r#"(?<n1>^|(?<n2>))(?<n3>[a-z\q{ab}]){1,3}(?<n4>(?<n5>(?<n6>\b)|(?<n7>\B)){2,3})"#,
    "dgvm",
    JsString::from("a"),
    0,
    MatchExpectation {
        text: "a",
        range: 0..1,
        captures: &[
            Some(""),
            None,
            Some("a"),
            Some(""),
            Some(""),
            Some(""),
            None,
        ],
        capture_ranges: &[
            Some((0, 0)),
            None,
            Some((0, 1)),
            Some((1, 1)),
            Some((1, 1)),
            Some((1, 1)),
            None,
        ],
        last_index: 1,
    };
    "input_a_from_0_variant_13"
)]
#[test_case(
    r#"(?<n1>$|(?<n2>))(?<n3>[\q{a|ab}])+(?<n4>(?<n5>(?<n6>^)|(?<n7>)){0,3})"#,
    "dgvy",
    JsString::from("a\n!"),
    0,
    MatchExpectation {
        text: "a",
        range: 0..1,
        captures: &[Some(""), Some(""), Some("a"), Some(""), None, None, None],
        capture_ranges: &[
            Some((0, 0)),
            Some((0, 0)),
            Some((0, 1)),
            Some((1, 1)),
            None,
            None,
            None,
        ],
        last_index: 1,
    };
    "input_a_from_0_variant_14"
)]
#[test_case(
    r#"(?<n1>$|(?<n2>))(?<n3>(?<n4>[\q{a|ab}])|(?<n5>))+?(?<n6>(?<n7>(?<n8>^)|(?<n9>)){0,3})"#,
    "dgvi",
    JsString::from("😀a"),
    1,
    MatchExpectation {
        text: "",
        range: 0..0,
        captures: &[
            Some(""),
            Some(""),
            Some(""),
            None,
            Some(""),
            Some(""),
            None,
            None,
            None,
        ],
        capture_ranges: &[
            Some((0, 0)),
            Some((0, 0)),
            Some((0, 0)),
            None,
            Some((0, 0)),
            Some((0, 0)),
            None,
            None,
            None,
        ],
        last_index: 0,
    };
    "unicode_input_from_1_variant_15"
)]
#[test_case(
    r#"(?<n1>$|(?<n2>))(?<n3>(?<n4>)|(?<n5>[\q{|a|ab}])){2,3}(?<n6>(?<n7>(?<n8>^)|(?<n9>)){0,3})"#,
    "dgvm",
    JsString::from(""),
    0,
    MatchExpectation {
        text: "",
        range: 0..0,
        captures: &[
            Some(""),
            None,
            Some(""),
            Some(""),
            None,
            Some(""),
            None,
            None,
            None,
        ],
        capture_ranges: &[
            Some((0, 0)),
            None,
            Some((0, 0)),
            Some((0, 0)),
            None,
            Some((0, 0)),
            None,
            None,
            None,
        ],
        last_index: 0,
    };
    "empty_input_from_0_variant_16"
)]
#[test_case(
    r#"(?<n1>$|(?<n2>))(?<n3>[a-z\q{ab}]){1,3}(?<n4>(?<n5>(?<n6>^)|(?<n7>)){0,3})"#,
    "dgvi",
    JsString::from("ab"),
    1,
    MatchExpectation {
        text: "b",
        range: 1..2,
        captures: &[Some(""), Some(""), Some("b"), Some(""), None, None, None],
        capture_ranges: &[
            Some((1, 1)),
            Some((1, 1)),
            Some((1, 2)),
            Some((2, 2)),
            None,
            None,
            None,
        ],
        last_index: 2,
    };
    "input_ab_from_1_variant_17"
)]
#[test_case(
    r#"(?<n1>(?<n2>^)|(?<n3>$)|(?<n4>\b)|(?<n5>\B))(?<n6>[\q{a|ab}])+(?<n7>(?<n8>\b\B)|(?<n9>))"#,
    "dgvi",
    JsString::from("ſa"),
    1,
    MatchExpectation {
        text: "a",
        range: 1..2,
        captures: &[
            Some(""),
            None,
            None,
            None,
            Some(""),
            Some("a"),
            Some(""),
            None,
            Some(""),
        ],
        capture_ranges: &[
            Some((1, 1)),
            None,
            None,
            None,
            Some((1, 1)),
            Some((1, 2)),
            Some((2, 2)),
            None,
            Some((2, 2)),
        ],
        last_index: 2,
    };
    "unicode_input_from_1_variant_18"
)]
#[test_case(
    r#"(?<n1>(?<n2>^)|(?<n3>$)|(?<n4>\b)|(?<n5>\B))(?<n6>(?<n7>[\q{a|ab}])|(?<n8>))+?(?<n9>(?<n10>\b\B)|(?<n11>))"#,
    "dgvm",
    JsString::from("ab\r\n"),
    0,
    MatchExpectation {
        text: "ab",
        range: 0..2,
        captures: &[
            Some(""),
            Some(""),
            None,
            None,
            None,
            Some("ab"),
            Some("ab"),
            None,
            Some(""),
            None,
            Some(""),
        ],
        capture_ranges: &[
            Some((0, 0)),
            Some((0, 0)),
            None,
            None,
            None,
            Some((0, 2)),
            Some((0, 2)),
            None,
            Some((2, 2)),
            None,
            Some((2, 2)),
        ],
        last_index: 2,
    };
    "input_ab_from_0_variant_19"
)]
#[test_case(
    r#"(?<n1>(?<n2>^)|(?<n3>$)|(?<n4>\b)|(?<n5>\B))(?<n6>(?<n7>)|(?<n8>[\q{|a|ab}])){2,3}(?<n9>(?<n10>\b\B)|(?<n11>))"#,
    "dgvi",
    JsString::from("a"),
    1,
    MatchExpectation {
        text: "",
        range: 1..1,
        captures: &[
            Some(""),
            None,
            Some(""),
            None,
            None,
            Some(""),
            Some(""),
            None,
            Some(""),
            None,
            Some(""),
        ],
        capture_ranges: &[
            Some((1, 1)),
            None,
            Some((1, 1)),
            None,
            None,
            Some((1, 1)),
            Some((1, 1)),
            None,
            Some((1, 1)),
            None,
            Some((1, 1)),
        ],
        last_index: 1,
    };
    "input_a_from_1_variant_20"
)]
#[test_case(
    r#"(?<n1>(?<n2>(?<n3>\b)|(?<n4>\B)){2,3})(?<n5>[\q{a|ab}])+(?<n6>(?<n7>)|(?<n8>\b))"#,
    "dgvm",
    JsString::from("😀a"),
    0,
    MatchExpectation {
        text: "a",
        range: 2..3,
        captures: &[
            Some(""),
            Some(""),
            Some(""),
            None,
            Some("a"),
            Some(""),
            Some(""),
            None,
        ],
        capture_ranges: &[
            Some((2, 2)),
            Some((2, 2)),
            Some((2, 2)),
            None,
            Some((2, 3)),
            Some((3, 3)),
            Some((3, 3)),
            None,
        ],
        last_index: 3,
    };
    "unicode_input_from_0_variant_21"
)]
#[test_case(
    r#"(?<n1>(?<n2>(?<n3>\b)|(?<n4>\B)){2,3})(?<n5>(?<n6>)|(?<n7>[\q{|a|ab}])){2,3}(?<n8>(?<n9>)|(?<n10>\b))"#,
    "dgvy",
    JsString::from("ab"),
    0,
    MatchExpectation {
        text: "ab",
        range: 0..2,
        captures: &[
            Some(""),
            Some(""),
            Some(""),
            None,
            Some("ab"),
            None,
            Some("ab"),
            Some(""),
            Some(""),
            None,
        ],
        capture_ranges: &[
            Some((0, 0)),
            Some((0, 0)),
            Some((0, 0)),
            None,
            Some((0, 2)),
            None,
            Some((0, 2)),
            Some((2, 2)),
            Some((2, 2)),
            None,
        ],
        last_index: 2,
    };
    "input_ab_from_0_variant_22"
)]
#[test_case(
    r#"(?<n1>(?<n2>(?<n3>^)|(?<n4>)){0,3})(?<n5>(?<n6>[\q{a|ab}])|(?<n7>))+?(?<n8>(?<n9>\b)|(?<n10>\B))"#,
    "dgvy",
    JsString::from("a"),
    0,
    MatchExpectation {
        text: "a",
        range: 0..1,
        captures: &[
            Some(""),
            None,
            None,
            None,
            Some("a"),
            Some("a"),
            None,
            Some(""),
            Some(""),
            None,
        ],
        capture_ranges: &[
            Some((0, 0)),
            None,
            None,
            None,
            Some((0, 1)),
            Some((0, 1)),
            None,
            Some((1, 1)),
            Some((1, 1)),
            None,
        ],
        last_index: 1,
    };
    "input_a_from_0_variant_23"
)]
#[test_case(
    r#"(?<n1>(?<n2>(?<n3>^)|(?<n4>)){0,3})(?<n5>(?<n6>)|(?<n7>[\q{|a|ab}])){2,3}(?<n8>(?<n9>\b)|(?<n10>\B))"#,
    "dgvim",
    JsString::from("!ab!"),
    1,
    MatchExpectation {
        text: "ab",
        range: 1..3,
        captures: &[
            Some(""),
            None,
            None,
            None,
            Some("ab"),
            None,
            Some("ab"),
            Some(""),
            Some(""),
            None,
        ],
        capture_ranges: &[
            Some((1, 1)),
            None,
            None,
            None,
            Some((1, 3)),
            None,
            Some((1, 3)),
            Some((3, 3)),
            Some((3, 3)),
            None,
        ],
        last_index: 3,
    };
    "input_ab_from_1_variant_24"
)]
#[test_case(
    r#"(?<n1>(?<n2>(?<n3>^)|(?<n4>)){0,3})(?<n5>[a-z\q{ab}]){1,3}(?<n6>(?<n7>\b)|(?<n8>\B))"#,
    "dgv",
    JsString::from("ſa"),
    0,
    MatchExpectation {
        text: "a",
        range: 1..2,
        captures: &[
            Some(""),
            None,
            None,
            None,
            Some("a"),
            Some(""),
            Some(""),
            None,
        ],
        capture_ranges: &[
            Some((1, 1)),
            None,
            None,
            None,
            Some((1, 2)),
            Some((2, 2)),
            Some((2, 2)),
            None,
        ],
        last_index: 2,
    };
    "unicode_input_from_0_variant_25"
)]
#[test_case(
    r#"(?<n1>(?<n2>\b\B)|(?<n3>))(?<n4>(?<n5>[\q{a|ab}])|(?<n6>))+?(?<n7>(?<n8>\B)|(?<n9>\b)){1}"#,
    "dgvim",
    JsString::from("ab"),
    1,
    MatchExpectation {
        text: "",
        range: 1..1,
        captures: &[
            Some(""),
            None,
            Some(""),
            Some(""),
            None,
            Some(""),
            Some(""),
            Some(""),
            None,
        ],
        capture_ranges: &[
            Some((1, 1)),
            None,
            Some((1, 1)),
            Some((1, 1)),
            None,
            Some((1, 1)),
            Some((1, 1)),
            Some((1, 1)),
            None,
        ],
        last_index: 1,
    };
    "input_ab_from_1_variant_26"
)]
#[test_case(
    r#"(?<n1>(?<n2>\b\B)|(?<n3>))(?<n4>(?<n5>)|(?<n6>[\q{|a|ab}])){2,3}(?<n7>(?<n8>\B)|(?<n9>\b)){1}"#,
    "dgv",
    JsString::from("a\n!"),
    0,
    MatchExpectation {
        text: "a",
        range: 0..1,
        captures: &[
            Some(""),
            None,
            Some(""),
            Some("a"),
            None,
            Some("a"),
            Some(""),
            None,
            Some(""),
        ],
        capture_ranges: &[
            Some((0, 0)),
            None,
            Some((0, 0)),
            Some((0, 1)),
            None,
            Some((0, 1)),
            Some((1, 1)),
            None,
            Some((1, 1)),
        ],
        last_index: 1,
    };
    "input_a_from_0_variant_27"
)]
#[test_case(
    r#"(?<n1>(?<n2>\b\B)|(?<n3>))(?<n4>[a-z\q{ab}]){1,3}(?<n5>(?<n6>\B)|(?<n7>\b)){1}"#,
    "dgvi",
    JsString::from("😀a"),
    1,
    MatchExpectation {
        text: "a",
        range: 2..3,
        captures: &[
            Some(""),
            None,
            Some(""),
            Some("a"),
            Some(""),
            None,
            Some(""),
        ],
        capture_ranges: &[
            Some((2, 2)),
            None,
            Some((2, 2)),
            Some((2, 3)),
            Some((3, 3)),
            None,
            Some((3, 3)),
        ],
        last_index: 3,
    };
    "unicode_input_from_1_variant_28"
)]
#[test_case(
    r#"(?<n1>(?<n2>)|(?<n3>\b))(?<n4>(?<n5>[\q{a|ab}])|(?<n6>))+?(?<n7>\b|(?<n8>)){2,3}?"#,
    "dgv",
    JsString::from("!ab!"),
    0,
    MatchExpectation {
        text: "",
        range: 0..0,
        captures: &[
            Some(""),
            Some(""),
            None,
            Some(""),
            None,
            Some(""),
            Some(""),
            Some(""),
        ],
        capture_ranges: &[
            Some((0, 0)),
            Some((0, 0)),
            None,
            Some((0, 0)),
            None,
            Some((0, 0)),
            Some((0, 0)),
            Some((0, 0)),
        ],
        last_index: 0,
    };
    "input_ab_from_0_variant_29"
)]
#[test_case(
    r#"(?<n1>(?<n2>)|(?<n3>\b))(?<n4>(?<n5>)|(?<n6>[\q{|a|ab}])){2,3}(?<n7>\b|(?<n8>)){2,3}?"#,
    "dgvi",
    JsString::from("ſa"),
    1,
    MatchExpectation {
        text: "a",
        range: 1..2,
        captures: &[
            Some(""),
            Some(""),
            None,
            Some("a"),
            None,
            Some("a"),
            Some(""),
            None,
        ],
        capture_ranges: &[
            Some((1, 1)),
            Some((1, 1)),
            None,
            Some((1, 2)),
            None,
            Some((1, 2)),
            Some((2, 2)),
            None,
        ],
        last_index: 2,
    };
    "unicode_input_from_1_variant_30"
)]
#[test_case(
    r#"(?<n1>(?<n2>)|(?<n3>\b))(?<n4>[a-z\q{ab}]){1,3}(?<n5>\b|(?<n6>)){2,3}?"#,
    "dgvym",
    JsString::from("ab\r\n"),
    0,
    MatchExpectation {
        text: "ab",
        range: 0..2,
        captures: &[Some(""), Some(""), None, Some("ab"), Some(""), None],
        capture_ranges: &[
            Some((0, 0)),
            Some((0, 0)),
            None,
            Some((0, 2)),
            Some((2, 2)),
            None,
        ],
        last_index: 2,
    };
    "input_ab_from_0_variant_31"
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
fn predicate_choice_string_classes_reject_unknown_leaves_before_execution() {
    for pattern in [
        r"((\b)|(\B))([\q{a|ab}])+[a]",
        r"((\b)|z|())([\q{a|ab}])+",
        r"((\b)|\1|())([\q{a|ab}])+",
        r"((\b)|(?=a)|())([\q{a|ab}])+",
        r"((\b)|(\B))(()|([\q{ab}&&\p{RGI_Emoji}])){3}",
        r"((\b)|(\B))([\q{a|ab}])+(z){0}",
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
    r#"(?<n1>(?<n2>\b)|(?<n3>\B))(?<n4>[\q{a|ab}])+(?<n5>^|(?<n6>))"#,
    "dgvym",
    JsString::from(""),
    0;
    "empty_input_from_0_variant_1"
)]
#[test_case(
    r#"(?<n1>(?<n2>\B)|(?<n3>\b)){1}(?<n4>[\q{a|ab}])+(?<n5>$|(?<n6>))"#,
    "dgvi",
    JsString::from("a"),
    1;
    "input_a_from_1_variant_2"
)]
#[test_case(
    r#"(?<n1>\b|(?<n2>)){2,3}?(?<n3>[a-z\q{ab}]){1,3}(?<n4>(?<n5>^)|(?<n6>$)|(?<n7>\b)|(?<n8>\B))"#,
    "dgvi",
    JsString::from(""),
    1;
    "empty_input_from_1_variant_3"
)]
#[test_case(
    r#"(?<n1>(?<n2>^)|(?<n3>$)|(?<n4>\b)|(?<n5>\B))(?<n6>[a-z\q{ab}]){1,3}(?<n7>(?<n8>\b\B)|(?<n9>))"#,
    "dgvy",
    JsString::from("!ab!"),
    0;
    "input_ab_from_0_variant_4"
)]
#[test_case(
    r#"(?<n1>(?<n2>(?<n3>\b)|(?<n4>\B)){2,3})(?<n5>(?<n6>[\q{a|ab}])|(?<n7>))+?(?<n8>(?<n9>)|(?<n10>\b))"#,
    "dgvi",
    JsString::from(""),
    1;
    "empty_input_from_1_variant_5"
)]
#[test_case(
    r#"(?<n1>(?<n2>(?<n3>\b)|(?<n4>\B)){2,3})(?<n5>[a-z\q{ab}]){1,3}(?<n6>(?<n7>)|(?<n8>\b))"#,
    "dgvim",
    JsString::from("a\n!"),
    1;
    "input_a_from_1_variant_6"
)]
#[test_case(
    r#"(?<n1>(?<n2>(?<n3>^)|(?<n4>)){0,3})(?<n5>[\q{a|ab}])+(?<n6>(?<n7>\b)|(?<n8>\B))"#,
    "dgvi",
    JsString::from("ab\r\n"),
    1;
    "input_ab_from_1_variant_7"
)]
#[test_case(
    r#"(?<n1>(?<n2>\b\B)|(?<n3>))(?<n4>[\q{a|ab}])+(?<n5>(?<n6>\B)|(?<n7>\b)){1}"#,
    "dgvy",
    JsString::from(""),
    0;
    "empty_input_from_0_variant_8"
)]
#[test_case(
    r#"(?<n1>(?<n2>)|(?<n3>\b))(?<n4>[\q{a|ab}])+(?<n5>\b|(?<n6>)){2,3}?"#,
    "dgvim",
    JsString::from("a"),
    1;
    "input_a_from_1_variant_9"
)]
fn no_match_resets_last_index(
    pattern: &str,
    flags: &str,
    input: JsString,
    initial_last_index: usize,
) {
    assert_no_match(pattern, flags, input, initial_last_index);
}
