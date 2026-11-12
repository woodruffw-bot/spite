//! Required predicate choices preserve conditional lexical captures.
use crate::common::assert_true;
use crate::common::assert_true_in;
use crate::common::eval_string;
use crate::common::regexp_captures::assert_no_match;
use crate::common::regexp_captures::{MatchExpectation, assert_named_captures};
use crate::common::regexp_matches::DESCRIBE_MATCH;
use spite_core::JsString;
use spite_runtime::Realm;
use test_case::test_case;
#[test]
fn capture_metadata_survives_collection() {
    let mut realm = Realm::default();
    realm.eval(r#"
        var regexp =
            /(?<outer>(?<b>(?<s>^)|(?<w>\b))(?<c>[\q{a|ab}])(?<a>(?<end>$)|(?<nw>\B))|(?<fallback>(?<wb>\b)|(?<nwb>\B))(?<empty>)){0001,0001}?/dvg,
            copy = new RegExp(regexp),
            match = regexp.exec('ab!aa')
    "#).unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_true_in(
        &mut realm,
        r#"
            var copiedMatch = copy.exec("ab"),
                p = regexp.exec("!");
            match[0] === "a" &&
                match.groups.outer === "a" &&
                match.groups.s === "" &&
                match.groups.w === undefined &&
                match.groups.c === "a" &&
                match.groups.fallback === undefined &&
                match.indices.groups.c === match.indices[5] &&
                match.indices[5][0] === 0 &&
                match.indices[5][1] === 1 &&
                copiedMatch.groups.c === "ab" &&
                copiedMatch.groups.outer === "ab" &&
                copiedMatch.groups.end === "" &&
                copy.source === regexp.source &&
                copy.lastIndex === 2 &&
                p[0] === "" &&
                p.groups.outer === "" &&
                p.groups.c === undefined &&
                p.groups.empty === "" &&
                p.groups.fallback === "" &&
                p.indices.groups.outer === p.indices[1] &&
                p.indices.groups.outer[0] === 1 &&
                p.indices.groups.outer[1] === 1 &&
                regexp.lastIndex === 1;
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
                    "(?<outer>(?<b>(?<s>^)|(?<w>\\b))(?<whole>(?<c>[\\q{{|a|ab}}]))(?<a>(?<end>$)|(?<nw>\\B))|(?<fallback>(?<wb>\\b)|(?<nwb>\\B))(?<empty>)){{1}}";
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
            "(?<outer>(?<b>(?<s>^)|(?<w>\\b))(?<whole>(?<c>[\\q{|a|ab}]))(?<a>(?<end>$)|(?<nw>\\B))|(?<fallback>(?<wb>\\b)|(?<nwb>\\B))(?<empty>)){1}";
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
                "(?<outer>(?<b>(?<s>^)|(?<w>\\b))(?<whole>(?<c>[\\q{|a|ab}]))(?<a>(?<end>$)|(?<nw>\\B))|(?<fallback>(?<wb>\\b)|(?<nwb>\\B))(?<empty>)){1}";
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
            "",
            "<undefined>",
            "",
            "",
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
            "",
            "",
            "<undefined>",
            "",
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
            "(?<outer>(?<b>(?<s>^)|(?<w>\\b))(?<whole>(?<c>[\\q{|a|ab}]))(?<a>(?<end>$)|(?<nw>\\B))|(?<fallback>(?<wb>\\b)|(?<nwb>\\B))(?<empty>)){1}";
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
                    "(?<outer>(?<b>(?<s>^)|(?<w>\\b))(?<whole>(?<c>[\\q{{|a|ab}}]))(?<a>(?<end>$)|(?<nw>\\B))|(?<fallback>(?<wb>\\b)|(?<nwb>\\B))(?<empty>)){{1}}";
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
                    "(?<outer>(?<b>(?<s>^)|(?<w>\\b))(?<whole>(?<c>[a-z\\q{{|S|sa|aX}}]))(?<a>(?<end>$)|(?<nw>\\B))|(?<fallback>(?<wb>\\b)|(?<nwb>\\B))(?<empty>)){{1}}";
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
                    "(?<outer>(?<b>(?<s>^)|(?<w>\\b))(?<whole>(?<c>[\\q{{|a|ab}}]))(?<a>(?<end>$)|(?<nw>\\B))|(?<fallback>(?<wb>\\b)|(?<nwb>\\B))(?<empty>)){{1}}";
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
                    "(?<outer>(?<\\u{{1D49C}}>(?<s>^)|(?<w>\\b))(?<whole>(?<c>[\\q{{|a|ab}}]))(?<a>(?<end>$)|(?<nw>\\B))|(?<fallback>(?<wb>\\b)|(?<nwb>\\B))(?<empty>)){{1}}";
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
    r#"(?<n1>(?<n2>(?<n3>^)|(?<n4>\b))(?<n5>[\q{a|ab}])(?<n6>(?<n7>\b^)|(?<n8>$))|(?<n9>(?<n10>^)|(?<n11>\B))(?<n12>(?<n13>){2,3})(?<n14>(?<n15>$)|(?<n16>\B))){1}"#,
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
            Some(""),
            Some(""),
            None,
            Some(""),
            Some(""),
            Some(""),
            Some(""),
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
            Some((0, 0)),
            Some((0, 0)),
            None,
            Some((0, 0)),
            Some((0, 0)),
            Some((0, 0)),
            Some((0, 0)),
            None,
        ],
        last_index: 0,
    };
    "empty_input_from_0_variant_1"
)]
#[test_case(
    r#"(?:(?<n1>(?<n2>(?<n3>(?<n4>^)|(?<n5>\B))(?<n6>(?<n7>){2,3})(?<n8>(?<n9>$)|(?<n10>\B))|(?<n11>(?<n12>^)|(?<n13>\b))(?<n14>(?<n15>[\q{|a|ab}])){1}(?<n16>(?<n17>\b^)|(?<n18>$))){1}){0001,0001}){1}?"#,
    "dgvi",
    JsString::from("ab"),
    1,
    MatchExpectation {
        text: "",
        range: 1..1,
        captures: &[
            Some(""),
            Some(""),
            Some(""),
            None,
            Some(""),
            Some(""),
            Some(""),
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
            Some((1, 1)),
            Some((1, 1)),
            Some((1, 1)),
            None,
            Some((1, 1)),
            Some((1, 1)),
            Some((1, 1)),
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
    "input_ab_from_1_variant_2"
)]
#[test_case(
    r#"(?<n1>(?:(?<n2>(?<n3>^)|(?<n4>\b))(?<n5>[\p{Any}\q{|ab}]){0001,0001}?(?<n6>(?<n7>\b^)|(?<n8>$))|(?<n9>(?<n10>^)|(?<n11>\B))(?<n12>(?<n13>){2,3})(?<n14>(?<n15>$)|(?<n16>\B))){1,1}){1}?"#,
    "dgv",
    JsString::from("a\n!"),
    0,
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
        ],
        capture_ranges: &[
            Some((0, 0)),
            Some((0, 0)),
            Some((0, 0)),
            None,
            Some((0, 0)),
            Some((0, 0)),
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
        ],
        last_index: 0,
    };
    "input_a_from_0_variant_3"
)]
#[test_case(
    r#"(?<n1>(?<n2>(?<n3>(?<n4>(?<n5>^)|(?<n6>\B))(?<n7>(?<n8>){2,3})(?<n9>(?<n10>$)|(?<n11>\B))|(?<n12>(?<n13>^)|(?<n14>\b))(?<n15>(?<n16>[a-z\q{ab}]){1}){1,1}?(?<n17>(?<n18>\b^)|(?<n19>$))){1}){1,1}){0001,0001}?"#,
    "dgvim",
    JsString::from("😀a"),
    1,
    MatchExpectation {
        text: "",
        range: 0..0,
        captures: &[
            Some(""),
            Some(""),
            Some(""),
            Some(""),
            Some(""),
            None,
            Some(""),
            Some(""),
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
            Some((0, 0)),
            Some((0, 0)),
            Some((0, 0)),
            Some((0, 0)),
            Some((0, 0)),
            None,
            Some((0, 0)),
            Some((0, 0)),
            Some((0, 0)),
            None,
            Some((0, 0)),
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
    r#"(?<n1>(?:(?<n2>(?<n3>$)|(?<n4>\b)){2,3}?(?<n5>[\p{Any}\q{|ab}]){0001,0001}?(?<n6>(?<n7>\B^)|(?<n8>\b$))|(?<n9>(?<n10>(?<n11>^)|(?<n12>\b))\B)(?<n13>(?<n14>){2,3})(?<n15>(?<n16>\b^)|(?<n17>\B$))){1,1}){1}?"#,
    "dgvim",
    JsString::from("ſa"),
    1,
    MatchExpectation {
        text: "",
        range: 2..2,
        captures: &[
            Some(""),
            Some(""),
            Some(""),
            None,
            Some(""),
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
            None,
        ],
        capture_ranges: &[
            Some((2, 2)),
            Some((2, 2)),
            Some((2, 2)),
            None,
            Some((2, 2)),
            Some((2, 2)),
            None,
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
        ],
        last_index: 2,
    };
    "unicode_input_from_1_variant_5"
)]
#[test_case(
    r#"(?<n1>(?<n2>(?<n3>^)|(?<n4>$)){3}(?<n5>[\q{a|ab}])(?<n6>(?<n7>^)|(?<n8>\B))|(?<n9>(?<n10>$)|(?<n11>\B))(?<n12>(?<n13>){2,3})(?<n14>(?<n15>\b\B)|(?<n16>\b\B))){1}"#,
    "dgv",
    JsString::from("ab"),
    0,
    MatchExpectation {
        text: "a",
        range: 0..1,
        captures: &[
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
    "input_ab_from_0_variant_6"
)]
#[test_case(
    r#"(?:(?<n1>(?<n2>(?<n3>(?<n4>$)|(?<n5>\B))(?<n6>(?<n7>){2,3})(?<n8>(?<n9>\b\B)|(?<n10>\b\B))|(?<n11>(?<n12>^)|(?<n13>$)){3}(?<n14>(?<n15>[\q{|a|ab}])){1}(?<n16>(?<n17>^)|(?<n18>\B))){1}){0001,0001}){1}?"#,
    "dgvim",
    JsString::from("a\n!"),
    1,
    MatchExpectation {
        text: "",
        range: 2..2,
        captures: &[
            Some(""),
            Some(""),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            Some(""),
            Some(""),
            None,
            Some(""),
            Some(""),
            Some(""),
            Some(""),
            None,
        ],
        capture_ranges: &[
            Some((2, 2)),
            Some((2, 2)),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            Some((2, 2)),
            Some((2, 2)),
            None,
            Some((2, 2)),
            Some((2, 2)),
            Some((2, 2)),
            Some((2, 2)),
            None,
        ],
        last_index: 2,
    };
    "input_a_from_1_variant_7"
)]
#[test_case(
    r#"(?<n1>(?:(?<n2>(?<n3>^)|(?<n4>$)){3}(?<n5>[\p{Any}\q{|ab}]){0001,0001}?(?<n6>(?<n7>^)|(?<n8>\B))|(?<n9>(?<n10>$)|(?<n11>\B))(?<n12>(?<n13>){2,3})(?<n14>(?<n15>\b\B)|(?<n16>\b\B))){1,1}){1}?"#,
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
            None,
            Some(""),
            Some(""),
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
        ],
        capture_ranges: &[
            Some((0, 0)),
            Some((0, 0)),
            Some((0, 0)),
            None,
            Some((0, 0)),
            Some((0, 0)),
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
        ],
        last_index: 0,
    };
    "unicode_input_from_0_variant_8"
)]
#[test_case(
    r#"(?<n1>(?<n2>(?<n3>(?<n4>(?<n5>\b^)|(?<n6>\B$))(?<n7>(?<n8>){2,3})(?<n9>(?<n10>^)|(?<n11>\b))|(?<n12>(?<n13>\b^)|(?<n14>$))(?<n15>(?<n16>[a-z\q{ab}]){1}){1,1}?(?<n17>(?<n18>(?<n19>^)|(?<n20>\b))\B)){1}){1,1}){0001,0001}?"#,
    "dgvm",
    JsString::from("a"),
    0,
    MatchExpectation {
        text: "",
        range: 0..0,
        captures: &[
            Some(""),
            Some(""),
            Some(""),
            Some(""),
            Some(""),
            None,
            Some(""),
            Some(""),
            Some(""),
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
        ],
        capture_ranges: &[
            Some((0, 0)),
            Some((0, 0)),
            Some((0, 0)),
            Some((0, 0)),
            Some((0, 0)),
            None,
            Some((0, 0)),
            Some((0, 0)),
            Some((0, 0)),
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
        ],
        last_index: 0,
    };
    "input_a_from_0_variant_9"
)]
#[test_case(
    r#"(?:(?<n1>(?<n2>(?<n3>(?<n4>\b\B)|(?<n5>\b\B))(?<n6>(?<n7>){2,3})(?<n8>(?<n9>$)|(?<n10>\b)){2,3}?|(?<n11>(?<n12>\B^)|(?<n13>\b$))(?<n14>(?<n15>[\q{|a|ab}])){1}(?<n16>(?<n17>$)|(?<n18>\B))){1}){0001,0001}){1}?"#,
    "dgvi",
    JsString::from("😀a"),
    1,
    MatchExpectation {
        text: "",
        range: 0..0,
        captures: &[
            Some(""),
            Some(""),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            Some(""),
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
            Some((0, 0)),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            Some((0, 0)),
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
    "unicode_input_from_1_variant_10"
)]
#[test_case(
    r#"(?<n1>(?:(?<n2>(?<n3>\B^)|(?<n4>\b$))(?<n5>[\p{Any}\q{|ab}]){0001,0001}?(?<n6>(?<n7>$)|(?<n8>\B))|(?<n9>(?<n10>\b\B)|(?<n11>\b\B))(?<n12>(?<n13>){2,3})(?<n14>(?<n15>$)|(?<n16>\b)){2,3}?){1,1}){1}?"#,
    "dgvm",
    JsString::from(""),
    0,
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
        ],
        capture_ranges: &[
            Some((0, 0)),
            Some((0, 0)),
            Some((0, 0)),
            None,
            Some((0, 0)),
            Some((0, 0)),
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
        ],
        last_index: 0,
    };
    "empty_input_from_0_variant_11"
)]
#[test_case(
    r#"(?<n1>(?<n2>(?<n3>^)|(?<n4>\B))(?<n5>[\q{a|ab}])(?<n6>(?<n7>\b^)|(?<n8>\B$))|(?<n9>(?<n10>^)|(?<n11>\b))(?<n12>(?<n13>){2,3})(?<n14>(?<n15>^)|(?<n16>$)){3}){1}"#,
    "dgvi",
    JsString::from("ſa"),
    1,
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
            Some(""),
            None,
            Some(""),
            Some(""),
            Some(""),
            Some(""),
            None,
            Some(""),
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
            Some((2, 2)),
            None,
            Some((2, 2)),
            Some((2, 2)),
            Some((2, 2)),
            Some((2, 2)),
            None,
            Some((2, 2)),
        ],
        last_index: 2,
    };
    "unicode_input_from_1_variant_12"
)]
#[test_case(
    r#"(?:(?<n1>(?<n2>(?<n3>(?<n4>^)|(?<n5>\b))(?<n6>(?<n7>){2,3})(?<n8>(?<n9>^)|(?<n10>$)){3}|(?<n11>(?<n12>^)|(?<n13>\B))(?<n14>(?<n15>[\q{|a|ab}])){1}(?<n16>(?<n17>\b^)|(?<n18>\B$))){1}){0001,0001}){1}?"#,
    "dgvm",
    JsString::from("ab\r\n"),
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
            Some(""),
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
        ],
        capture_ranges: &[
            Some((0, 0)),
            Some((0, 0)),
            Some((0, 0)),
            Some((0, 0)),
            None,
            Some((0, 0)),
            Some((0, 0)),
            Some((0, 0)),
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
        ],
        last_index: 0,
    };
    "input_ab_from_0_variant_13"
)]
#[test_case(
    r#"(?<n1>(?:(?<n2>(?<n3>^)|(?<n4>\B))(?<n5>[\p{Any}\q{|ab}]){0001,0001}?(?<n6>(?<n7>\b^)|(?<n8>\B$))|(?<n9>(?<n10>^)|(?<n11>\b))(?<n12>(?<n13>){2,3})(?<n14>(?<n15>^)|(?<n16>$)){3}){1,1}){1}?"#,
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
            Some(""),
            None,
            Some(""),
            Some(""),
            Some(""),
            Some(""),
            None,
            Some(""),
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
            Some((1, 1)),
            None,
            Some((1, 1)),
            Some((1, 1)),
            Some((1, 1)),
            Some((1, 1)),
            None,
            Some((1, 1)),
        ],
        last_index: 1,
    };
    "input_a_from_1_variant_14"
)]
#[test_case(
    r#"(?<n1>(?<n2>(?<n3>(?<n4>(?<n5>^)|(?<n6>\b))(?<n7>(?<n8>){2,3})(?<n9>(?<n10>^)|(?<n11>$)){3}|(?<n12>(?<n13>^)|(?<n14>\B))(?<n15>(?<n16>[a-z\q{ab}]){1}){1,1}?(?<n17>(?<n18>\b^)|(?<n19>\B$))){1}){1,1}){0001,0001}?"#,
    "dgvy",
    JsString::from("!ab!"),
    0,
    MatchExpectation {
        text: "",
        range: 0..0,
        captures: &[
            Some(""),
            Some(""),
            Some(""),
            Some(""),
            Some(""),
            None,
            Some(""),
            Some(""),
            Some(""),
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
        ],
        capture_ranges: &[
            Some((0, 0)),
            Some((0, 0)),
            Some((0, 0)),
            Some((0, 0)),
            Some((0, 0)),
            None,
            Some((0, 0)),
            Some((0, 0)),
            Some((0, 0)),
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
        ],
        last_index: 0,
    };
    "input_ab_from_0_variant_15"
)]
#[test_case(
    r#"(?<n1>(?<n2>(?<n3>(?<n4>^)|(?<n5>\b))\B)(?<n6>[\q{a|ab}])(?<n7>(?<n8>\b\B)|(?<n9>\b\B))|(?<n10>(?<n11>$)|(?<n12>\b)){2,3}?(?<n13>(?<n14>){2,3})(?<n15>(?<n16>\b^)|(?<n17>$))){1}"#,
    "dgvm",
    JsString::from("😀a"),
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
            Some(""),
            Some(""),
            None,
            Some(""),
            Some(""),
            Some(""),
            None,
            Some(""),
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
            Some((3, 3)),
            Some((3, 3)),
            None,
            Some((3, 3)),
            Some((3, 3)),
            Some((3, 3)),
            None,
            Some((3, 3)),
        ],
        last_index: 3,
    };
    "unicode_input_from_0_variant_16"
)]
#[test_case(
    r#"(?<n1>(?:(?<n2>(?<n3>(?<n4>^)|(?<n5>\b))\B)(?<n6>[\p{Any}\q{|ab}]){0001,0001}?(?<n7>(?<n8>\b\B)|(?<n9>\b\B))|(?<n10>(?<n11>$)|(?<n12>\b)){2,3}?(?<n13>(?<n14>){2,3})(?<n15>(?<n16>\b^)|(?<n17>$))){1,1}){1}?"#,
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
            Some(""),
            None,
            Some(""),
            Some(""),
            Some(""),
            Some(""),
            Some(""),
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
            Some((0, 0)),
            None,
            Some((0, 0)),
            Some((0, 0)),
            Some((0, 0)),
            Some((0, 0)),
            Some((0, 0)),
            None,
        ],
        last_index: 0,
    };
    "input_ab_from_0_variant_17"
)]
#[test_case(
    r#"(?<n1>(?<n2>(?<n3>(?<n4>(?<n5>$)|(?<n6>\b)){2,3}?(?<n7>(?<n8>){2,3})(?<n9>(?<n10>\b^)|(?<n11>$))|(?<n12>(?<n13>(?<n14>^)|(?<n15>\b))\B)(?<n16>(?<n17>[a-z\q{ab}]){1}){1,1}?(?<n18>(?<n19>\b\B)|(?<n20>\b\B))){1}){1,1}){0001,0001}?"#,
    "dgvim",
    JsString::from("a\n!"),
    1,
    MatchExpectation {
        text: "",
        range: 1..1,
        captures: &[
            Some(""),
            Some(""),
            Some(""),
            Some(""),
            Some(""),
            None,
            Some(""),
            Some(""),
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
            None,
        ],
        capture_ranges: &[
            Some((1, 1)),
            Some((1, 1)),
            Some((1, 1)),
            Some((1, 1)),
            Some((1, 1)),
            None,
            Some((1, 1)),
            Some((1, 1)),
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
            None,
        ],
        last_index: 1,
    };
    "input_a_from_1_variant_18"
)]
#[test_case(
    r#"(?<n1>(?:(?<n2>(?<n3>$)|(?<n4>\B))(?<n5>[\p{Any}\q{|ab}]){0001,0001}?(?<n6>(?<n7>^)|(?<n8>\b))|(?<n9>(?<n10>^)|(?<n11>$)){3}(?<n12>(?<n13>){2,3})(?<n14>(?<n15>\B^)|(?<n16>\b$))){1,1}){1}?"#,
    "dgvim",
    JsString::from("!ab!"),
    1,
    MatchExpectation {
        text: "b",
        range: 2..3,
        captures: &[
            Some("b"),
            Some(""),
            None,
            Some(""),
            Some("b"),
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
            Some((2, 3)),
            Some((2, 2)),
            None,
            Some((2, 2)),
            Some((2, 3)),
            Some((3, 3)),
            None,
            Some((3, 3)),
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
    "input_ab_from_1_variant_19"
)]
#[test_case(
    r#"(?<n1>(?<n2>(?<n3>(?<n4>(?<n5>^)|(?<n6>$)){3}(?<n7>(?<n8>){2,3})(?<n9>(?<n10>\B^)|(?<n11>\b$))|(?<n12>(?<n13>$)|(?<n14>\B))(?<n15>(?<n16>[a-z\q{ab}]){1}){1,1}?(?<n17>(?<n18>^)|(?<n19>\b))){1}){1,1}){0001,0001}?"#,
    "dgv",
    JsString::from("ſa"),
    0,
    MatchExpectation {
        text: "",
        range: 0..0,
        captures: &[
            Some(""),
            Some(""),
            Some(""),
            Some(""),
            Some(""),
            None,
            Some(""),
            Some(""),
            Some(""),
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
        ],
        capture_ranges: &[
            Some((0, 0)),
            Some((0, 0)),
            Some((0, 0)),
            Some((0, 0)),
            Some((0, 0)),
            None,
            Some((0, 0)),
            Some((0, 0)),
            Some((0, 0)),
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
        ],
        last_index: 0,
    };
    "unicode_input_from_0_variant_20"
)]
#[test_case(
    r#"(?<n1>(?<n2>(?<n3>\b^)|(?<n4>\B$))(?<n5>[\q{a|ab}])(?<n6>(?<n7>$)|(?<n8>\b)){2,3}?|(?<n9>(?<n10>\b^)|(?<n11>$))(?<n12>(?<n13>){2,3})(?<n14>(?<n15>^)|(?<n16>\B))){1}"#,
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
            Some(""),
            None,
            Some(""),
            Some(""),
            Some(""),
            Some(""),
            Some(""),
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
            Some((0, 0)),
            None,
            Some((0, 0)),
            Some((0, 0)),
            Some((0, 0)),
            Some((0, 0)),
            Some((0, 0)),
            None,
        ],
        last_index: 0,
    };
    "empty_input_from_0_variant_21"
)]
#[test_case(
    r#"(?<n1>(?:(?<n2>(?<n3>\b^)|(?<n4>\B$))(?<n5>[\p{Any}\q{|ab}]){0001,0001}?(?<n6>(?<n7>$)|(?<n8>\b)){2,3}?|(?<n9>(?<n10>\b^)|(?<n11>$))(?<n12>(?<n13>){2,3})(?<n14>(?<n15>^)|(?<n16>\B))){1,1}){1}?"#,
    "dgv",
    JsString::from("a\n!"),
    0,
    MatchExpectation {
        text: "a",
        range: 0..1,
        captures: &[
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
    "input_a_from_0_variant_22"
)]
#[test_case(
    r#"(?:(?<n1>(?<n2>(?<n3>(?<n4>\B^)|(?<n5>\b$))(?<n6>(?<n7>){2,3})(?<n8>(?<n9>(?<n10>^)|(?<n11>\b))\B)|(?<n12>(?<n13>\b\B)|(?<n14>\b\B))(?<n15>(?<n16>[\q{|a|ab}])){1}(?<n17>(?<n18>^)|(?<n19>$)){3}){1}){0001,0001}){1}?"#,
    "dgv",
    JsString::from("!ab!"),
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
            Some(""),
            Some(""),
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
        ],
        capture_ranges: &[
            Some((0, 0)),
            Some((0, 0)),
            Some((0, 0)),
            Some((0, 0)),
            None,
            Some((0, 0)),
            Some((0, 0)),
            Some((0, 0)),
            Some((0, 0)),
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
        ],
        last_index: 0,
    };
    "input_ab_from_0_variant_23"
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
fn truth_table_exact_grouped_alternative_string_classes_prove_unknown_branches_and_other_wrapper_counts()
 {
    assert_true(
        r####"var pattern="^(((^)|(\\b))([\\q{a|ab}])|()){1}",input="aba",r=new RegExp(pattern,"dv"),m=r.exec(input);r.source===pattern&&r.flags==="dv"&&r.hasIndices&&r.unicodeSets&&(m===null||(m.input===input&&m.groups===undefined&&m.indices.groups===undefined&&Object.hasOwn(m,"groups")&&Object.hasOwn(m.indices,"groups")))&&JSON.stringify(m===null?null:[Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex])==="[[\"ab\",\"ab\",\"\",\"\",\"<undefined>\",\"ab\",\"<undefined>\"],[[0,2],[0,2],[0,0],[0,0],\"<undefined>\",[0,2],\"<undefined>\"],0,0]""####,
    );
    for pattern in [
        r"(((\b\B)|(\b\B))([\q{ab}&&\p{RGI_Emoji}])|()){1}",
        r"(((\b\B)|(\b\B))([\q{|a|ab}])|((\b\B)|(\b\B))z){1}",
        r"(((^)|(\b))([\q{a|ab}])|\1){1}",
        r"(((^)|(\b))([\q{a|ab}])|([a])){1}",
        r"(((^)|(\b))([\q{a|ab}])|()){2}",
        r"(((^)|(\b))([\q{a|ab}])|()){1,2}",
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
    r#"(?<n1>(?<n2>(?<n3>$)|(?<n4>\b)){2,3}?(?<n5>[\q{a|ab}])(?<n6>(?<n7>\B^)|(?<n8>\b$))|(?<n9>(?<n10>(?<n11>^)|(?<n12>\b))\B)(?<n13>(?<n14>){2,3})(?<n15>(?<n16>\b^)|(?<n17>\B$))){1}"#,
    "dgvi",
    JsString::from("a"),
    1;
    "input_a_from_1_variant_1"
)]
#[test_case(
    r#"(?:(?<n1>(?<n2>(?<n3>(?<n4>(?<n5>^)|(?<n6>\b))\B)(?<n7>(?<n8>){2,3})(?<n9>(?<n10>\b^)|(?<n11>\B$))|(?<n12>(?<n13>$)|(?<n14>\b)){2,3}?(?<n15>(?<n16>[\q{|a|ab}])){1}(?<n17>(?<n18>\B^)|(?<n19>\b$))){1}){0001,0001}){1}?"#,
    "dgv",
    JsString::from("!ab!"),
    0;
    "input_ab_from_0_variant_2"
)]
#[test_case(
    r#"(?<n1>(?<n2>(?<n3>(?<n4>(?<n5>(?<n6>^)|(?<n7>\b))\B)(?<n8>(?<n9>){2,3})(?<n10>(?<n11>\b^)|(?<n12>\B$))|(?<n13>(?<n14>$)|(?<n15>\b)){2,3}?(?<n16>(?<n17>[a-z\q{ab}]){1}){1,1}?(?<n18>(?<n19>\B^)|(?<n20>\b$))){1}){1,1}){0001,0001}?"#,
    "dgvy",
    JsString::from("ab\r\n"),
    0;
    "input_ab_from_0_variant_3"
)]
#[test_case(
    r#"(?<n1>(?<n2>(?<n3>(?<n4>(?<n5>$)|(?<n6>\B))(?<n7>(?<n8>){2,3})(?<n9>(?<n10>\b\B)|(?<n11>\b\B))|(?<n12>(?<n13>^)|(?<n14>$)){3}(?<n15>(?<n16>[a-z\q{ab}]){1}){1,1}?(?<n17>(?<n18>^)|(?<n19>\B))){1}){1,1}){0001,0001}?"#,
    "dgvi",
    JsString::from(""),
    1;
    "empty_input_from_1_variant_4"
)]
#[test_case(
    r#"(?<n1>(?<n2>(?<n3>\b^)|(?<n4>$))(?<n5>[\q{a|ab}])(?<n6>(?<n7>(?<n8>^)|(?<n9>\b))\B)|(?<n10>(?<n11>\b^)|(?<n12>\B$))(?<n13>(?<n14>){2,3})(?<n15>(?<n16>^)|(?<n17>\b))){1}"#,
    "dgvim",
    JsString::from("!ab!"),
    1;
    "input_ab_from_1_variant_5"
)]
#[test_case(
    r#"(?:(?<n1>(?<n2>(?<n3>(?<n4>\b^)|(?<n5>\B$))(?<n6>(?<n7>){2,3})(?<n8>(?<n9>^)|(?<n10>\b))|(?<n11>(?<n12>\b^)|(?<n13>$))(?<n14>(?<n15>[\q{|a|ab}])){1}(?<n16>(?<n17>(?<n18>^)|(?<n19>\b))\B)){1}){0001,0001}){1}?"#,
    "dgvy",
    JsString::from("ſa"),
    0;
    "unicode_input_from_0_variant_6"
)]
#[test_case(
    r#"(?<n1>(?:(?<n2>(?<n3>\b^)|(?<n4>$))(?<n5>[\p{Any}\q{|ab}]){0001,0001}?(?<n6>(?<n7>(?<n8>^)|(?<n9>\b))\B)|(?<n10>(?<n11>\b^)|(?<n12>\B$))(?<n13>(?<n14>){2,3})(?<n15>(?<n16>^)|(?<n17>\b))){1,1}){1}?"#,
    "dgvi",
    JsString::from("ab\r\n"),
    1;
    "input_ab_from_1_variant_7"
)]
#[test_case(
    r#"(?<n1>(?<n2>(?<n3>\B^)|(?<n4>\b$))(?<n5>[\q{a|ab}])(?<n6>(?<n7>$)|(?<n8>\B))|(?<n9>(?<n10>\b\B)|(?<n11>\b\B))(?<n12>(?<n13>){2,3})(?<n14>(?<n15>$)|(?<n16>\b)){2,3}?){1}"#,
    "dgvy",
    JsString::from("a\n!"),
    0;
    "input_a_from_0_variant_8"
)]
#[test_case(
    r#"(?<n1>(?<n2>(?<n3>(?<n4>(?<n5>\b\B)|(?<n6>\b\B))(?<n7>(?<n8>){2,3})(?<n9>(?<n10>$)|(?<n11>\b)){2,3}?|(?<n12>(?<n13>\B^)|(?<n14>\b$))(?<n15>(?<n16>[a-z\q{ab}]){1}){1,1}?(?<n17>(?<n18>$)|(?<n19>\B))){1}){1,1}){0001,0001}?"#,
    "dgvi",
    JsString::from("ab"),
    1;
    "input_ab_from_1_variant_9"
)]
#[test_case(
    r#"(?:(?<n1>(?<n2>(?<n3>(?<n4>$)|(?<n5>\b)){2,3}?(?<n6>(?<n7>){2,3})(?<n8>(?<n9>\b^)|(?<n10>$))|(?<n11>(?<n12>(?<n13>^)|(?<n14>\b))\B)(?<n15>(?<n16>[\q{|a|ab}])){1}(?<n17>(?<n18>\b\B)|(?<n19>\b\B))){1}){0001,0001}){1}?"#,
    "dgvi",
    JsString::from(""),
    1;
    "empty_input_from_1_variant_10"
)]
#[test_case(
    r#"(?<n1>(?<n2>(?<n3>$)|(?<n4>\B))(?<n5>[\q{a|ab}])(?<n6>(?<n7>^)|(?<n8>\b))|(?<n9>(?<n10>^)|(?<n11>$)){3}(?<n12>(?<n13>){2,3})(?<n14>(?<n15>\B^)|(?<n16>\b$))){1}"#,
    "dgvi",
    JsString::from("ab\r\n"),
    1;
    "input_ab_from_1_variant_11"
)]
#[test_case(
    r#"(?:(?<n1>(?<n2>(?<n3>(?<n4>^)|(?<n5>$)){3}(?<n6>(?<n7>){2,3})(?<n8>(?<n9>\B^)|(?<n10>\b$))|(?<n11>(?<n12>$)|(?<n13>\B))(?<n14>(?<n15>[\q{|a|ab}])){1}(?<n16>(?<n17>^)|(?<n18>\b))){1}){0001,0001}){1}?"#,
    "dgvy",
    JsString::from("a"),
    0;
    "input_a_from_0_variant_12"
)]
#[test_case(
    r#"(?:(?<n1>(?<n2>(?<n3>(?<n4>\b^)|(?<n5>$))(?<n6>(?<n7>){2,3})(?<n8>(?<n9>^)|(?<n10>\B))|(?<n11>(?<n12>\b^)|(?<n13>\B$))(?<n14>(?<n15>[\q{|a|ab}])){1}(?<n16>(?<n17>$)|(?<n18>\b)){2,3}?){1}){0001,0001}){1}?"#,
    "dgvim",
    JsString::from("ab"),
    1;
    "input_ab_from_1_variant_13"
)]
#[test_case(
    r#"(?<n1>(?<n2>(?<n3>(?<n4>(?<n5>\b^)|(?<n6>$))(?<n7>(?<n8>){2,3})(?<n9>(?<n10>^)|(?<n11>\B))|(?<n12>(?<n13>\b^)|(?<n14>\B$))(?<n15>(?<n16>[a-z\q{ab}]){1}){1,1}?(?<n17>(?<n18>$)|(?<n19>\b)){2,3}?){1}){1,1}){0001,0001}?"#,
    "dgvi",
    JsString::from("😀a"),
    1;
    "unicode_input_from_1_variant_14"
)]
#[test_case(
    r#"(?<n1>(?<n2>(?<n3>\b\B)|(?<n4>\b\B))(?<n5>[\q{a|ab}])(?<n6>(?<n7>^)|(?<n8>$)){3}|(?<n9>(?<n10>\B^)|(?<n11>\b$))(?<n12>(?<n13>){2,3})(?<n14>(?<n15>(?<n16>^)|(?<n17>\b))\B)){1}"#,
    "dgvim",
    JsString::from("a"),
    1;
    "input_a_from_1_variant_15"
)]
#[test_case(
    r#"(?<n1>(?:(?<n2>(?<n3>\b\B)|(?<n4>\b\B))(?<n5>[\p{Any}\q{|ab}]){0001,0001}?(?<n6>(?<n7>^)|(?<n8>$)){3}|(?<n9>(?<n10>\B^)|(?<n11>\b$))(?<n12>(?<n13>){2,3})(?<n14>(?<n15>(?<n16>^)|(?<n17>\b))\B)){1,1}){1}?"#,
    "dgvi",
    JsString::from("ſa"),
    1;
    "unicode_input_from_1_variant_16"
)]
#[test_case(
    r#"(?<n1>(?<n2>(?<n3>(?<n4>(?<n5>\B^)|(?<n6>\b$))(?<n7>(?<n8>){2,3})(?<n9>(?<n10>(?<n11>^)|(?<n12>\b))\B)|(?<n13>(?<n14>\b\B)|(?<n15>\b\B))(?<n16>(?<n17>[a-z\q{ab}]){1}){1,1}?(?<n18>(?<n19>^)|(?<n20>$)){3}){1}){1,1}){0001,0001}?"#,
    "dgvym",
    JsString::from("ab\r\n"),
    0;
    "input_ab_from_0_variant_17"
)]
fn no_match_resets_last_index(
    pattern: &str,
    flags: &str,
    input: JsString,
    initial_last_index: usize,
) {
    assert_no_match(pattern, flags, input, initial_last_index);
}
