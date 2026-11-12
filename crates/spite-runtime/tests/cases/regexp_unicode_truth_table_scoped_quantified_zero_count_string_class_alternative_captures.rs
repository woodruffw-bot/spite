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
            /(?<ob>(?<os>^)|(?<ow>\b))(?<bp>)(?<enclosing>(?:(?<outer>(?<nested>(?<b>(?<s>^)|(?<w>\b))(?<c>[\q{a|ab}])+?(?<a>(?<end>$)|(?<nw>\B))|(?<fallback>(?<wb>\b)|(?<nwb>\B))(?<empty>))){0}))(?<ep>)(?<oa>(?<oe>$)|(?<onw>\B))/dvg,
            copy = new RegExp(regexp),
            match = regexp.exec('ab!aa')
    "#).unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_true_in(
        &mut realm,
        r#"
            var copiedMatch = copy.exec("ab");
            regexp.lastIndex = 0;
            var z = regexp.exec("");
            regexp.lastIndex = 0;
            var p = regexp.exec("!");
            match[0] === "" &&
                match.index === 5 &&
                match.groups.enclosing === "" &&
                match.groups.outer === undefined &&
                match.groups.nested === undefined &&
                match.groups.c === undefined &&
                match.groups.os === undefined &&
                match.groups.ow === "" &&
                match.groups.oe === "" &&
                match.groups.onw === undefined &&
                match.indices.groups.enclosing === match.indices[5] &&
                match.indices[5][0] === 5 &&
                match.indices[5][1] === 5 &&
                Array.from(match)
                    .slice(6, 19)
                    .every((v) => v === undefined) &&
                match.indices.slice(6, 19).every((v) => v === undefined) &&
                copiedMatch[0] === "" &&
                copiedMatch.index === 2 &&
                copiedMatch.groups.enclosing === "" &&
                copiedMatch.groups.outer === undefined &&
                copiedMatch.groups.nested === undefined &&
                copy.source === regexp.source &&
                copy.lastIndex === 2 &&
                z[0] === "" &&
                z.groups.enclosing === "" &&
                z.groups.outer === undefined &&
                z.groups.nested === undefined &&
                z.groups.os === "" &&
                z.groups.oe === "" &&
                p[0] === "" &&
                p.index === 0 &&
                p.groups.enclosing === "" &&
                p.groups.outer === undefined &&
                p.groups.nested === undefined &&
                p.groups.os === "" &&
                p.groups.onw === "" &&
                regexp.lastIndex === 0;
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
                    "(?<ob>(?<os>^)|(?<ow>\\b))(?<bp>)(?<outer>(?<nested>(?<b>(?<s>^)|(?<w>\\b))(?<c>[\\q{{|a|ab}}])+?(?<a>(?<end>$)|(?<nw>\\B))|(?<fallback>(?<wb>\\b)|(?<nwb>\\B))(?<empty>))){{0}}(?<ep>)(?<oa>(?<oe>$)|(?<onw>\\B))";
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
            "(?<ob>(?<os>^)|(?<ow>\\b))(?<bp>)(?<outer>(?<nested>(?<b>(?<s>^)|(?<w>\\b))(?<c>[\\q{|a|ab}])+?(?<a>(?<end>$)|(?<nw>\\B))|(?<fallback>(?<wb>\\b)|(?<nwb>\\B))(?<empty>))){0}(?<ep>)(?<oa>(?<oe>$)|(?<onw>\\B))";
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
                "(?<ob>(?<os>^)|(?<ow>\\b))(?<bp>)(?<outer>(?<nested>(?<b>(?<s>^)|(?<w>\\b))(?<c>[\\q{|a|ab}])+?(?<a>(?<end>$)|(?<nw>\\B))|(?<fallback>(?<wb>\\b)|(?<nwb>\\B))(?<empty>))){0}(?<ep>)(?<oa>(?<oe>$)|(?<onw>\\B))";
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
            "(?<ob>(?<os>^)|(?<ow>\\b))(?<bp>)(?<outer>(?<nested>(?<b>(?<s>^)|(?<w>\\b))(?<c>[\\q{|a|ab}])+?(?<a>(?<end>$)|(?<nw>\\B))|(?<fallback>(?<wb>\\b)|(?<nwb>\\B))(?<empty>))){0}(?<ep>)(?<oa>(?<oe>$)|(?<onw>\\B))";
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
fn greedy_and_lazy_quantifiers_preserve_captures() {
    let source = format!(
        r#"
            (function() {{
                {DESCRIBE_MATCH}

                var P =
                    "(?<ob>(?<os>^)|(?<ow>\\b))(?<bp>)(?<outer>(?<nested>(?<b>(?<s>^)|(?<w>\\b))(?<c>[\\q{{|a|ab}}])+?(?<a>(?<end>$)|(?<nw>\\B))|(?<fallback>(?<wb>\\b)|(?<nwb>\\B))(?<empty>))){{0}}(?<ep>)(?<oa>(?<oe>$)|(?<onw>\\B))";
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
                    "(?<ob>(?<os>^)|(?<ow>\\b))(?<bp>)(?<outer>(?<nested>(?<b>(?<s>^)|(?<w>\\b))(?<c>[a-z\\q{{|S|sa|aX}}])+?(?<a>(?<end>$)|(?<nw>\\B))|(?<fallback>(?<wb>\\b)|(?<nwb>\\B))(?<empty>))){{0}}(?<ep>)(?<oa>(?<oe>$)|(?<onw>\\B))";
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
                    "(?<ob>(?<os>^)|(?<ow>\\b))(?<bp>)(?<outer>(?<nested>(?<b>(?<s>^)|(?<w>\\b))(?<c>[\\q{{|a|ab}}])+?(?<a>(?<end>$)|(?<nw>\\B))|(?<fallback>(?<wb>\\b)|(?<nwb>\\B))(?<empty>))){{0}}(?<ep>)(?<oa>(?<oe>$)|(?<onw>\\B))";
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
                    "(?<ob>(?<os>^)|(?<ow>\\b))(?<bp>)(?<outer>(?<nested>(?<\\u{{1D49C}}>(?<s>^)|(?<w>\\b))(?<c>[\\q{{|a|ab}}])+?(?<a>(?<end>$)|(?<nw>\\B))|(?<fallback>(?<wb>\\b)|(?<nwb>\\B))(?<empty>))){{0}}(?<ep>)(?<oa>(?<oe>$)|(?<onw>\\B))";
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
    r#"(?:(?<n1>(?<n2>^)|(?<n3>$)){3})(?<n4>)(?<n5>(?<n6>(?<n7>(?<n8>(?<n9>^)|(?<n10>\b))(?:(?<n11>[\q{a|ab}]))*(?<n12>(?<n13>\b^)|(?<n14>$))|(?<n15>(?<n16>^)|(?<n17>\B))(?<n18>(?<n19>){2,3})(?<n20>(?<n21>$)|(?<n22>\B)))){0})(?<n23>)(?:(?<n24>(?<n25>(?<n26>^)|(?<n27>\b))\B)){2,3}?"#,
    "dgvym",
    JsString::from(""),
    0,
    MatchExpectation {
        text: "",
        range: 0..0,
        captures: &[
            Some(""),
            Some(""),
            None,
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
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            Some(""),
            Some(""),
            Some(""),
            Some(""),
            None,
        ],
        capture_ranges: &[
            Some((0, 0)),
            Some((0, 0)),
            None,
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
            None,
            None,
            None,
            None,
            None,
            None,
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
    r#"(?:(?<n1>(?<n2>^)|(?<n3>$)){3})?(?<n4>)(?:(?<n5>(?<n6>(?:(?:(?<n7>(?<n8>(?<n9>(?<n10>^)|(?<n11>\B))(?<n12>(?<n13>){2,3})(?<n14>(?<n15>$)|(?<n16>\B))|(?<n17>(?<n18>^)|(?<n19>\b))(?:(?<n20>(?<n21>[\q{|a|ab}])){1})+?(?<n22>(?<n23>\b^)|(?<n24>$)))))){0,0}?))){1}(?<n25>)(?:(?<n26>(?<n27>(?<n28>^)|(?<n29>\b))\B)){0,2}?"#,
    "dgvi",
    JsString::from("ab"),
    1,
    MatchExpectation {
        text: "",
        range: 1..1,
        captures: &[
            None,
            None,
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
            None,
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
            None,
            None,
            None,
        ],
        capture_ranges: &[
            None,
            None,
            None,
            Some((1, 1)),
            Some((1, 1)),
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
            None,
            Some((1, 1)),
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
    r#"(?:(?<n1>(?<n2>^)|(?<n3>$)){3}){0,2}?(?<n4>)(?<n5>(?<n6>(?<n7>(?:(?<n8>(?<n9>(?<n10>(?<n11>(?<n12>^)|(?<n13>\B))(?<n14>(?<n15>){2,3})(?<n16>(?<n17>$)|(?<n18>\B))|(?<n19>(?<n20>^)|(?<n21>\b))(?:(?<n22>(?<n23>[a-z\q{ab}]){1}){1,1}?){0,}?(?<n24>(?<n25>\b^)|(?<n26>$)))){0001,0001}?)){0000}?))){0001,0001}(?<n27>)(?:(?<n28>(?<n29>(?<n30>^)|(?<n31>\b))\B))?"#,
    "dgvim",
    JsString::from("😀a"),
    1,
    MatchExpectation {
        text: "",
        range: 0..0,
        captures: &[
            None,
            None,
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
            None,
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
            None,
            None,
            None,
        ],
        capture_ranges: &[
            None,
            None,
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
            None,
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
            None,
            None,
            None,
        ],
        last_index: 0,
    };
    "unicode_input_from_1_variant_3"
)]
#[test_case(
    r#"(?:(?<n1>(?<n2>\b^)|(?<n3>$)))(?<n4>)(?<n5>(?<n6>(?<n7>(?<n8>(?<n9>$)|(?<n10>\b)){2,3}?(?:(?<n11>[\q{a|ab}]))*(?<n12>(?<n13>\B^)|(?<n14>\b$))|(?<n15>(?<n16>(?<n17>^)|(?<n18>\b))\B)(?<n19>(?<n20>){2,3})(?<n21>(?<n22>\b^)|(?<n23>\B$)))){0})(?<n24>)(?:(?<n25>(?<n26>$)|(?<n27>\B))){2,3}?"#,
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
            Some(""),
            None,
        ],
        capture_ranges: &[
            Some((1, 1)),
            None,
            Some((1, 1)),
            Some((1, 1)),
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
            None,
            Some((1, 1)),
            Some((1, 1)),
            Some((1, 1)),
            None,
        ],
        last_index: 1,
    };
    "input_a_from_1_variant_4"
)]
#[test_case(
    r#"(?:(?<n1>(?<n2>\b^)|(?<n3>$)))?(?<n4>)(?:(?<n5>(?<n6>(?:(?:(?<n7>(?<n8>(?<n9>(?<n10>(?<n11>^)|(?<n12>\b))\B)(?<n13>(?<n14>){2,3})(?<n15>(?<n16>\b^)|(?<n17>\B$))|(?<n18>(?<n19>$)|(?<n20>\b)){2,3}?(?:(?<n21>(?<n22>[\q{|a|ab}])){1})+?(?<n23>(?<n24>\B^)|(?<n25>\b$)))))){0,0}?))){1}(?<n26>)(?:(?<n27>(?<n28>$)|(?<n29>\B))){0,2}?"#,
    "dgv",
    JsString::from("!ab!"),
    0,
    MatchExpectation {
        text: "",
        range: 0..0,
        captures: &[
            None,
            None,
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
            None,
            None,
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
            None,
            None,
        ],
        capture_ranges: &[
            None,
            None,
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
            None,
            None,
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
            None,
            None,
        ],
        last_index: 0,
    };
    "input_ab_from_0_variant_5"
)]
#[test_case(
    r#"(?:(?<n1>(?<n2>\b^)|(?<n3>$))){2,3}?(?<n4>)(?<n5>(?:(?<n6>(?<n7>(?:(?<n8>(?<n9>$)|(?<n10>\b)){2,3}?(?:(?<n11>[\p{Any}\q{|ab}]){0001,0001}?){2,3}(?<n12>(?<n13>\B^)|(?<n14>\b$))|(?<n15>(?<n16>(?<n17>^)|(?<n18>\b))\B)(?<n19>(?<n20>){2,3})(?<n21>(?<n22>\b^)|(?<n23>\B$))){1,1})){0000,0000})){1,1}?(?<n24>)(?:(?<n25>(?<n26>$)|(?<n27>\B)))"#,
    "dgvim",
    JsString::from("ſa"),
    1,
    MatchExpectation {
        text: "",
        range: 2..2,
        captures: &[
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
            None,
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
            Some(""),
            None,
        ],
        capture_ranges: &[
            Some((2, 2)),
            None,
            Some((2, 2)),
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
            None,
            None,
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
            Some((2, 2)),
            None,
        ],
        last_index: 2,
    };
    "unicode_input_from_1_variant_6"
)]
#[test_case(
    r#"(?:(?<n1>(?<n2>\b^)|(?<n3>$))){0,2}?(?<n4>)(?<n5>(?<n6>(?<n7>(?:(?<n8>(?<n9>(?<n10>(?<n11>(?<n12>(?<n13>^)|(?<n14>\b))\B)(?<n15>(?<n16>){2,3})(?<n17>(?<n18>\b^)|(?<n19>\B$))|(?<n20>(?<n21>$)|(?<n22>\b)){2,3}?(?:(?<n23>(?<n24>[a-z\q{ab}]){1}){1,1}?){0,}?(?<n25>(?<n26>\B^)|(?<n27>\b$)))){0001,0001}?)){0000}?))){0001,0001}(?<n28>)(?:(?<n29>(?<n30>$)|(?<n31>\B)))?"#,
    "dgvy",
    JsString::from("ab\r\n"),
    0,
    MatchExpectation {
        text: "",
        range: 0..0,
        captures: &[
            None,
            None,
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
            None,
            None,
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
            None,
            None,
        ],
        capture_ranges: &[
            None,
            None,
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
            None,
            None,
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
            None,
            None,
        ],
        last_index: 0,
    };
    "input_ab_from_0_variant_7"
)]
#[test_case(
    r#"(?:(?<n1>(?<n2>\B^)|(?<n3>\b$)))?(?<n4>)(?:(?<n5>(?<n6>(?:(?:(?<n7>(?<n8>(?<n9>(?<n10>$)|(?<n11>\B))(?<n12>(?<n13>){2,3})(?<n14>(?<n15>\b\B)|(?<n16>\b\B))|(?<n17>(?<n18>^)|(?<n19>$)){3}(?:(?<n20>(?<n21>[\q{|a|ab}])){1})+?(?<n22>(?<n23>^)|(?<n24>\B)))))){0,0}?))){1}(?<n25>)(?:(?<n26>(?<n27>\b^)|(?<n28>\B$))){0,2}?"#,
    "dgvim",
    JsString::from("a\n!"),
    1,
    MatchExpectation {
        text: "",
        range: 1..1,
        captures: &[
            None,
            None,
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
            None,
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
            None,
            None,
        ],
        capture_ranges: &[
            None,
            None,
            None,
            Some((1, 1)),
            Some((1, 1)),
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
            None,
            Some((1, 1)),
            None,
            None,
            None,
        ],
        last_index: 1,
    };
    "input_a_from_1_variant_8"
)]
#[test_case(
    r#"(?:(?<n1>(?<n2>^)|(?<n3>\B)))?(?<n4>)(?:(?<n5>(?<n6>(?:(?:(?<n7>(?<n8>(?<n9>(?<n10>\b^)|(?<n11>\B$))(?<n12>(?<n13>){2,3})(?<n14>(?<n15>^)|(?<n16>\b))|(?<n17>(?<n18>\b^)|(?<n19>$))(?:(?<n20>(?<n21>[\q{|a|ab}])){1})+?(?<n22>(?<n23>(?<n24>^)|(?<n25>\b))\B))))){0,0}?))){1}(?<n26>)(?:(?<n27>(?<n28>\b\B)|(?<n29>\b\B))){0,2}?"#,
    "dgvy",
    JsString::from("ſa"),
    0,
    MatchExpectation {
        text: "",
        range: 0..0,
        captures: &[
            None,
            None,
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
            None,
            None,
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
            None,
            None,
        ],
        capture_ranges: &[
            None,
            None,
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
            None,
            None,
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
            None,
            None,
        ],
        last_index: 0,
    };
    "unicode_input_from_0_variant_9"
)]
#[test_case(
    r#"(?:(?<n1>(?<n2>^)|(?<n3>\B))){0,2}?(?<n4>)(?<n5>(?<n6>(?<n7>(?:(?<n8>(?<n9>(?<n10>(?<n11>(?<n12>\b^)|(?<n13>\B$))(?<n14>(?<n15>){2,3})(?<n16>(?<n17>^)|(?<n18>\b))|(?<n19>(?<n20>\b^)|(?<n21>$))(?:(?<n22>(?<n23>[a-z\q{ab}]){1}){1,1}?){0,}?(?<n24>(?<n25>(?<n26>^)|(?<n27>\b))\B))){0001,0001}?)){0000}?))){0001,0001}(?<n28>)(?:(?<n29>(?<n30>\b\B)|(?<n31>\b\B)))?"#,
    "dgvm",
    JsString::from("a"),
    0,
    MatchExpectation {
        text: "",
        range: 0..0,
        captures: &[
            None,
            None,
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
            None,
            None,
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
            None,
            None,
        ],
        capture_ranges: &[
            None,
            None,
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
            None,
            None,
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
            None,
            None,
        ],
        last_index: 0,
    };
    "input_a_from_0_variant_10"
)]
#[test_case(
    r#"(?:(?<n1>(?<n2>(?<n3>^)|(?<n4>\b))\B))?(?<n5>)(?:(?<n6>(?<n7>(?:(?:(?<n8>(?<n9>(?<n10>(?<n11>\b\B)|(?<n12>\b\B))(?<n13>(?<n14>){2,3})(?<n15>(?<n16>$)|(?<n17>\b)){2,3}?|(?<n18>(?<n19>\B^)|(?<n20>\b$))(?:(?<n21>(?<n22>[\q{|a|ab}])){1})+?(?<n23>(?<n24>$)|(?<n25>\B)))))){0,0}?))){1}(?<n26>)(?:(?<n27>(?<n28>^)|(?<n29>\b))){0,2}?"#,
    "dgvi",
    JsString::from("😀a"),
    1,
    MatchExpectation {
        text: "",
        range: 0..0,
        captures: &[
            None,
            None,
            None,
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
            None,
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
            None,
            None,
        ],
        capture_ranges: &[
            None,
            None,
            None,
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
            None,
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
            None,
            None,
        ],
        last_index: 0,
    };
    "unicode_input_from_1_variant_11"
)]
#[test_case(
    r#"(?:(?<n1>(?<n2>(?<n3>^)|(?<n4>\b))\B)){2,3}?(?<n5>)(?<n6>(?:(?<n7>(?<n8>(?:(?<n9>(?<n10>\B^)|(?<n11>\b$))(?:(?<n12>[\p{Any}\q{|ab}]){0001,0001}?){2,3}(?<n13>(?<n14>$)|(?<n15>\B))|(?<n16>(?<n17>\b\B)|(?<n18>\b\B))(?<n19>(?<n20>){2,3})(?<n21>(?<n22>$)|(?<n23>\b)){2,3}?){1,1})){0000,0000})){1,1}?(?<n24>)(?:(?<n25>(?<n26>^)|(?<n27>\b)))"#,
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
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
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
            Some(""),
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
            None,
            None,
            None,
            None,
            None,
            None,
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
            Some((0, 0)),
            None,
        ],
        last_index: 0,
    };
    "empty_input_from_0_variant_12"
)]
#[test_case(
    r#"(?:(?<n1>(?<n2>(?<n3>^)|(?<n4>\b))\B)){0,2}?(?<n5>)(?<n6>(?<n7>(?<n8>(?:(?<n9>(?<n10>(?<n11>(?<n12>(?<n13>\b\B)|(?<n14>\b\B))(?<n15>(?<n16>){2,3})(?<n17>(?<n18>$)|(?<n19>\b)){2,3}?|(?<n20>(?<n21>\B^)|(?<n22>\b$))(?:(?<n23>(?<n24>[a-z\q{ab}]){1}){1,1}?){0,}?(?<n25>(?<n26>$)|(?<n27>\B)))){0001,0001}?)){0000}?))){0001,0001}(?<n28>)(?:(?<n29>(?<n30>^)|(?<n31>\b)))?"#,
    "dgvi",
    JsString::from("ab"),
    1,
    MatchExpectation {
        text: "",
        range: 1..1,
        captures: &[
            None,
            None,
            None,
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
            None,
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
            None,
            None,
        ],
        capture_ranges: &[
            None,
            None,
            None,
            None,
            Some((1, 1)),
            Some((1, 1)),
            Some((1, 1)),
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
            None,
            None,
            Some((1, 1)),
            None,
            None,
            None,
        ],
        last_index: 1,
    };
    "input_ab_from_1_variant_13"
)]
#[test_case(
    r#"(?:(?<n1>(?<n2>$)|(?<n3>\B)))(?<n4>)(?<n5>(?<n6>(?<n7>(?<n8>(?<n9>^)|(?<n10>\B))(?:(?<n11>[\q{a|ab}]))*(?<n12>(?<n13>\b^)|(?<n14>\B$))|(?<n15>(?<n16>^)|(?<n17>\b))(?<n18>(?<n19>){2,3})(?<n20>(?<n21>^)|(?<n22>$)){3})){0})(?<n23>)(?:(?<n24>(?<n25>$)|(?<n26>\b)){2,3}?){2,3}?"#,
    "dgvi",
    JsString::from("ſa"),
    1,
    MatchExpectation {
        text: "",
        range: 2..2,
        captures: &[
            Some(""),
            Some(""),
            None,
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
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            Some(""),
            Some(""),
            Some(""),
            None,
        ],
        capture_ranges: &[
            Some((2, 2)),
            Some((2, 2)),
            None,
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
            None,
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
            Some((2, 2)),
            None,
        ],
        last_index: 2,
    };
    "unicode_input_from_1_variant_14"
)]
#[test_case(
    r#"(?:(?<n1>(?<n2>$)|(?<n3>\B)))?(?<n4>)(?:(?<n5>(?<n6>(?:(?:(?<n7>(?<n8>(?<n9>(?<n10>^)|(?<n11>\b))(?<n12>(?<n13>){2,3})(?<n14>(?<n15>^)|(?<n16>$)){3}|(?<n17>(?<n18>^)|(?<n19>\B))(?:(?<n20>(?<n21>[\q{|a|ab}])){1})+?(?<n22>(?<n23>\b^)|(?<n24>\B$)))))){0,0}?))){1}(?<n25>)(?:(?<n26>(?<n27>$)|(?<n28>\b)){2,3}?){0,2}?"#,
    "dgvm",
    JsString::from("ab\r\n"),
    0,
    MatchExpectation {
        text: "",
        range: 0..0,
        captures: &[
            None,
            None,
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
            None,
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
            None,
            None,
        ],
        capture_ranges: &[
            None,
            None,
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
            None,
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
            None,
            None,
        ],
        last_index: 0,
    };
    "input_ab_from_0_variant_15"
)]
#[test_case(
    r#"(?:(?<n1>(?<n2>$)|(?<n3>\B))){2,3}?(?<n4>)(?<n5>(?:(?<n6>(?<n7>(?:(?<n8>(?<n9>^)|(?<n10>\B))(?:(?<n11>[\p{Any}\q{|ab}]){0001,0001}?){2,3}(?<n12>(?<n13>\b^)|(?<n14>\B$))|(?<n15>(?<n16>^)|(?<n17>\b))(?<n18>(?<n19>){2,3})(?<n20>(?<n21>^)|(?<n22>$)){3}){1,1})){0000,0000})){1,1}?(?<n23>)(?:(?<n24>(?<n25>$)|(?<n26>\b)){2,3}?)"#,
    "dgvi",
    JsString::from("a"),
    1,
    MatchExpectation {
        text: "",
        range: 1..1,
        captures: &[
            Some(""),
            Some(""),
            None,
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
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            Some(""),
            Some(""),
            Some(""),
            None,
        ],
        capture_ranges: &[
            Some((1, 1)),
            Some((1, 1)),
            None,
            Some((1, 1)),
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
            Some((1, 1)),
            Some((1, 1)),
            Some((1, 1)),
            None,
        ],
        last_index: 1,
    };
    "input_a_from_1_variant_16"
)]
#[test_case(
    r#"(?:(?<n1>(?<n2>$)|(?<n3>\B))){0,2}?(?<n4>)(?<n5>(?<n6>(?<n7>(?:(?<n8>(?<n9>(?<n10>(?<n11>(?<n12>^)|(?<n13>\b))(?<n14>(?<n15>){2,3})(?<n16>(?<n17>^)|(?<n18>$)){3}|(?<n19>(?<n20>^)|(?<n21>\B))(?:(?<n22>(?<n23>[a-z\q{ab}]){1}){1,1}?){0,}?(?<n24>(?<n25>\b^)|(?<n26>\B$)))){0001,0001}?)){0000}?))){0001,0001}(?<n27>)(?:(?<n28>(?<n29>$)|(?<n30>\b)){2,3}?)?"#,
    "dgvy",
    JsString::from("!ab!"),
    0,
    MatchExpectation {
        text: "",
        range: 0..0,
        captures: &[
            None,
            None,
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
            None,
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
            None,
            None,
        ],
        capture_ranges: &[
            None,
            None,
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
            None,
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
            None,
            None,
        ],
        last_index: 0,
    };
    "input_ab_from_0_variant_17"
)]
#[test_case(
    r#"(?:(?<n1>(?<n2>\b^)|(?<n3>\B$))){2,3}?(?<n4>)(?<n5>(?:(?<n6>(?<n7>(?:(?<n8>(?<n9>(?<n10>^)|(?<n11>\b))\B)(?:(?<n12>[\p{Any}\q{|ab}]){0001,0001}?){2,3}(?<n13>(?<n14>\b\B)|(?<n15>\b\B))|(?<n16>(?<n17>$)|(?<n18>\b)){2,3}?(?<n19>(?<n20>){2,3})(?<n21>(?<n22>\b^)|(?<n23>$))){1,1})){0000,0000})){1,1}?(?<n24>)(?:(?<n25>(?<n26>^)|(?<n27>$)){3})"#,
    "dgvy",
    JsString::from("ab"),
    0,
    MatchExpectation {
        text: "",
        range: 0..0,
        captures: &[
            Some(""),
            Some(""),
            None,
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
            Some(""),
            None,
        ],
        capture_ranges: &[
            Some((0, 0)),
            Some((0, 0)),
            None,
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
            Some((0, 0)),
            None,
        ],
        last_index: 0,
    };
    "input_ab_from_0_variant_18"
)]
#[test_case(
    r#"(?:(?<n1>(?<n2>\b^)|(?<n3>\B$))){0,2}?(?<n4>)(?<n5>(?<n6>(?<n7>(?:(?<n8>(?<n9>(?<n10>(?<n11>(?<n12>$)|(?<n13>\b)){2,3}?(?<n14>(?<n15>){2,3})(?<n16>(?<n17>\b^)|(?<n18>$))|(?<n19>(?<n20>(?<n21>^)|(?<n22>\b))\B)(?:(?<n23>(?<n24>[a-z\q{ab}]){1}){1,1}?){0,}?(?<n25>(?<n26>\b\B)|(?<n27>\b\B)))){0001,0001}?)){0000}?))){0001,0001}(?<n28>)(?:(?<n29>(?<n30>^)|(?<n31>$)){3})?"#,
    "dgvim",
    JsString::from("a\n!"),
    1,
    MatchExpectation {
        text: "",
        range: 1..1,
        captures: &[
            None,
            None,
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
            None,
            None,
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
            None,
            None,
        ],
        capture_ranges: &[
            None,
            None,
            None,
            Some((1, 1)),
            Some((1, 1)),
            Some((1, 1)),
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
            None,
            None,
            None,
            Some((1, 1)),
            None,
            None,
            None,
        ],
        last_index: 1,
    };
    "input_a_from_1_variant_19"
)]
#[test_case(
    r#"(?:(?<n1>(?<n2>\b\B)|(?<n3>\b\B)))?(?<n4>)(?:(?<n5>(?<n6>(?:(?:(?<n7>(?<n8>(?<n9>(?<n10>^)|(?<n11>$)){3}(?<n12>(?<n13>){2,3})(?<n14>(?<n15>\B^)|(?<n16>\b$))|(?<n17>(?<n18>$)|(?<n19>\B))(?:(?<n20>(?<n21>[\q{|a|ab}])){1})+?(?<n22>(?<n23>^)|(?<n24>\b)))))){0,0}?))){1}(?<n25>)(?:(?<n26>(?<n27>\b^)|(?<n28>$))){0,2}?"#,
    "dgvy",
    JsString::from("a"),
    0,
    MatchExpectation {
        text: "",
        range: 0..0,
        captures: &[
            None,
            None,
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
            None,
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
            None,
            None,
        ],
        capture_ranges: &[
            None,
            None,
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
            None,
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
            None,
            None,
        ],
        last_index: 0,
    };
    "input_a_from_0_variant_20"
)]
#[test_case(
    r#"(?:(?<n1>(?<n2>\b\B)|(?<n3>\b\B))){0,2}?(?<n4>)(?<n5>(?<n6>(?<n7>(?:(?<n8>(?<n9>(?<n10>(?<n11>(?<n12>^)|(?<n13>$)){3}(?<n14>(?<n15>){2,3})(?<n16>(?<n17>\B^)|(?<n18>\b$))|(?<n19>(?<n20>$)|(?<n21>\B))(?:(?<n22>(?<n23>[a-z\q{ab}]){1}){1,1}?){0,}?(?<n24>(?<n25>^)|(?<n26>\b)))){0001,0001}?)){0000}?))){0001,0001}(?<n27>)(?:(?<n28>(?<n29>\b^)|(?<n30>$)))?"#,
    "dgv",
    JsString::from("ſa"),
    0,
    MatchExpectation {
        text: "",
        range: 0..0,
        captures: &[
            None,
            None,
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
            None,
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
            None,
            None,
        ],
        capture_ranges: &[
            None,
            None,
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
            None,
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
            None,
            None,
        ],
        last_index: 0,
    };
    "unicode_input_from_0_variant_21"
)]
#[test_case(
    r#"(?:(?<n1>(?<n2>^)|(?<n3>\b)))(?<n4>)(?<n5>(?<n6>(?<n7>(?<n8>(?<n9>\b^)|(?<n10>\B$))(?:(?<n11>[\q{a|ab}]))*(?<n12>(?<n13>$)|(?<n14>\b)){2,3}?|(?<n15>(?<n16>\b^)|(?<n17>$))(?<n18>(?<n19>){2,3})(?<n20>(?<n21>^)|(?<n22>\B)))){0})(?<n23>)(?:(?<n24>(?<n25>\B^)|(?<n26>\b$))){2,3}?"#,
    "dgvy",
    JsString::from(""),
    0,
    MatchExpectation {
        text: "",
        range: 0..0,
        captures: &[
            Some(""),
            Some(""),
            None,
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
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            Some(""),
            Some(""),
            Some(""),
            None,
        ],
        capture_ranges: &[
            Some((0, 0)),
            Some((0, 0)),
            None,
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
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            Some((0, 0)),
            Some((0, 0)),
            Some((0, 0)),
            None,
        ],
        last_index: 0,
    };
    "empty_input_from_0_variant_22"
)]
#[test_case(
    r#"(?:(?<n1>(?<n2>^)|(?<n3>\b)))?(?<n4>)(?:(?<n5>(?<n6>(?:(?:(?<n7>(?<n8>(?<n9>(?<n10>\b^)|(?<n11>$))(?<n12>(?<n13>){2,3})(?<n14>(?<n15>^)|(?<n16>\B))|(?<n17>(?<n18>\b^)|(?<n19>\B$))(?:(?<n20>(?<n21>[\q{|a|ab}])){1})+?(?<n22>(?<n23>$)|(?<n24>\b)){2,3}?)))){0,0}?))){1}(?<n25>)(?:(?<n26>(?<n27>\B^)|(?<n28>\b$))){0,2}?"#,
    "dgvim",
    JsString::from("ab"),
    1,
    MatchExpectation {
        text: "",
        range: 1..1,
        captures: &[
            None,
            None,
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
            None,
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
            None,
            None,
        ],
        capture_ranges: &[
            None,
            None,
            None,
            Some((1, 1)),
            Some((1, 1)),
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
            None,
            Some((1, 1)),
            None,
            None,
            None,
        ],
        last_index: 1,
    };
    "input_ab_from_1_variant_23"
)]
#[test_case(
    r#"(?:(?<n1>(?<n2>^)|(?<n3>\b))){0,2}?(?<n4>)(?<n5>(?<n6>(?<n7>(?:(?<n8>(?<n9>(?<n10>(?<n11>(?<n12>\b^)|(?<n13>$))(?<n14>(?<n15>){2,3})(?<n16>(?<n17>^)|(?<n18>\B))|(?<n19>(?<n20>\b^)|(?<n21>\B$))(?:(?<n22>(?<n23>[a-z\q{ab}]){1}){1,1}?){0,}?(?<n24>(?<n25>$)|(?<n26>\b)){2,3}?)){0001,0001}?)){0000}?))){0001,0001}(?<n27>)(?:(?<n28>(?<n29>\B^)|(?<n30>\b$)))?"#,
    "dgvi",
    JsString::from("😀a"),
    1,
    MatchExpectation {
        text: "",
        range: 0..0,
        captures: &[
            None,
            None,
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
            None,
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
            None,
            None,
        ],
        capture_ranges: &[
            None,
            None,
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
            None,
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
            None,
            None,
        ],
        last_index: 0,
    };
    "unicode_input_from_1_variant_24"
)]
#[test_case(
    r#"(?:(?<n1>(?<n2>$)|(?<n3>\b)){2,3}?)?(?<n4>)(?:(?<n5>(?<n6>(?:(?:(?<n7>(?<n8>(?<n9>(?<n10>\B^)|(?<n11>\b$))(?<n12>(?<n13>){2,3})(?<n14>(?<n15>(?<n16>^)|(?<n17>\b))\B)|(?<n18>(?<n19>\b\B)|(?<n20>\b\B))(?:(?<n21>(?<n22>[\q{|a|ab}])){1})+?(?<n23>(?<n24>^)|(?<n25>$)){3})))){0,0}?))){1}(?<n26>)(?:(?<n27>(?<n28>^)|(?<n29>\B))){0,2}?"#,
    "dgv",
    JsString::from("!ab!"),
    0,
    MatchExpectation {
        text: "",
        range: 0..0,
        captures: &[
            None,
            None,
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
            None,
            None,
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
            None,
            None,
        ],
        capture_ranges: &[
            None,
            None,
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
            None,
            None,
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
            None,
            None,
        ],
        last_index: 0,
    };
    "input_ab_from_0_variant_25"
)]
#[test_case(
    r#"(?:(?<n1>(?<n2>$)|(?<n3>\b)){2,3}?){0,2}?(?<n4>)(?<n5>(?<n6>(?<n7>(?:(?<n8>(?<n9>(?<n10>(?<n11>(?<n12>\B^)|(?<n13>\b$))(?<n14>(?<n15>){2,3})(?<n16>(?<n17>(?<n18>^)|(?<n19>\b))\B)|(?<n20>(?<n21>\b\B)|(?<n22>\b\B))(?:(?<n23>(?<n24>[a-z\q{ab}]){1}){1,1}?){0,}?(?<n25>(?<n26>^)|(?<n27>$)){3})){0001,0001}?)){0000}?))){0001,0001}(?<n28>)(?:(?<n29>(?<n30>^)|(?<n31>\B)))?"#,
    "dgvym",
    JsString::from("ab\r\n"),
    0,
    MatchExpectation {
        text: "",
        range: 0..0,
        captures: &[
            None,
            None,
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
            None,
            None,
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
            None,
            None,
        ],
        capture_ranges: &[
            None,
            None,
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
            None,
            None,
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
            None,
            None,
        ],
        last_index: 0,
    };
    "input_ab_from_0_variant_26"
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
fn truth_table_scoped_quantified_zero_count_alternative_string_classes_validate_complete_unexecuted_regions()
 {
    assert_true(
        r####"var pattern="()((((^)|(\\b))([\\q{a|ab}]+?)|\\1)){0}()",input="A😀!",r=new RegExp(pattern,"dgv");r.lastIndex=2;var m=r.exec(input);var own=true;for(var i=0;i<m.length;i++)own=own&&Object.hasOwn(m,""+i)&&Object.hasOwn(m.indices,""+i);own&&r.source===pattern&&r.flags==="dgv"&&r.hasIndices&&m.input===input&&m.groups===undefined&&m.indices.groups===undefined&&Object.hasOwn(m,"groups")&&Object.hasOwn(m.indices,"groups")&&JSON.stringify([Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex])==="[[\"\",\"\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"\"],[[1,1],[1,1],\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",[1,1]],1,1]""####,
    );
    assert_true(
        r####"var pattern="()((((^)|(\\b))([\\q{a|ab}]+?)|([a]))){0}()",input="A😀!",r=new RegExp(pattern,"dgv");r.lastIndex=2;var m=r.exec(input);var own=true;for(var i=0;i<m.length;i++)own=own&&Object.hasOwn(m,""+i)&&Object.hasOwn(m.indices,""+i);own&&r.source===pattern&&r.flags==="dgv"&&r.hasIndices&&m.input===input&&m.groups===undefined&&m.indices.groups===undefined&&Object.hasOwn(m,"groups")&&Object.hasOwn(m.indices,"groups")&&JSON.stringify([Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex])==="[[\"\",\"\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"\"],[[1,1],[1,1],\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",[1,1]],1,1]""####,
    );
    assert_true(
        r####"var pattern="((^)|z){0}()((((^)|(\\b))([\\q{a|ab}]+?)|())){0}()",input="A😀!",r=new RegExp(pattern,"dgv");r.lastIndex=2;var m=r.exec(input);var own=true;for(var i=0;i<m.length;i++)own=own&&Object.hasOwn(m,""+i)&&Object.hasOwn(m.indices,""+i);own&&r.source===pattern&&r.flags==="dgv"&&r.hasIndices&&m.input===input&&m.groups===undefined&&m.indices.groups===undefined&&Object.hasOwn(m,"groups")&&Object.hasOwn(m.indices,"groups")&&JSON.stringify([Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex])==="[[\"\",\"<undefined>\",\"<undefined>\",\"\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"\"],[[1,1],\"<undefined>\",\"<undefined>\",[1,1],\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",[1,1]],1,1]""####,
    );
    for pattern in [
        r"((\b\B)|(\b\B))()((([\q{ab}&&\p{RGI_Emoji}]+?)|())){0}()",
        r"((^)|(\b))?()((((^)|(\b))([\q{a|ab}]+?)|z)){0}()",
        r"((\b\B)|([a]))?()((((^)|(\b))([\q{a|ab}]+?)|())){0}()",
        r"()((((^)|(\b))([\q{a|ab}]+?)|()){1}){0,1}()",
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
    r#"(?:(?<n1>(?<n2>^)|(?<n3>$)){3}){2,3}?(?<n4>)(?<n5>(?:(?<n6>(?<n7>(?:(?<n8>(?<n9>^)|(?<n10>\b))(?:(?<n11>[\p{Any}\q{|ab}]){0001,0001}?){2,3}(?<n12>(?<n13>\b^)|(?<n14>$))|(?<n15>(?<n16>^)|(?<n17>\B))(?<n18>(?<n19>){2,3})(?<n20>(?<n21>$)|(?<n22>\B))){1,1})){0000,0000})){1,1}?(?<n23>)(?:(?<n24>(?<n25>(?<n26>^)|(?<n27>\b))\B))"#,
    "dgv",
    JsString::from("a\n!"),
    0;
    "input_a_from_0_variant_1"
)]
#[test_case(
    r#"(?:(?<n1>(?<n2>\B^)|(?<n3>\b$)))(?<n4>)(?<n5>(?<n6>(?<n7>(?<n8>(?<n9>^)|(?<n10>$)){3}(?:(?<n11>[\q{a|ab}]))*(?<n12>(?<n13>^)|(?<n14>\B))|(?<n15>(?<n16>$)|(?<n17>\B))(?<n18>(?<n19>){2,3})(?<n20>(?<n21>\b\B)|(?<n22>\b\B)))){0})(?<n23>)(?:(?<n24>(?<n25>\b^)|(?<n26>\B$))){2,3}?"#,
    "dgv",
    JsString::from("ab"),
    0;
    "input_ab_from_0_variant_2"
)]
#[test_case(
    r#"(?:(?<n1>(?<n2>\B^)|(?<n3>\b$))){2,3}?(?<n4>)(?<n5>(?:(?<n6>(?<n7>(?:(?<n8>(?<n9>^)|(?<n10>$)){3}(?:(?<n11>[\p{Any}\q{|ab}]){0001,0001}?){2,3}(?<n12>(?<n13>^)|(?<n14>\B))|(?<n15>(?<n16>$)|(?<n17>\B))(?<n18>(?<n19>){2,3})(?<n20>(?<n21>\b\B)|(?<n22>\b\B))){1,1})){0000,0000})){1,1}?(?<n23>)(?:(?<n24>(?<n25>\b^)|(?<n26>\B$)))"#,
    "dgvy",
    JsString::from("😀a"),
    0;
    "unicode_input_from_0_variant_3"
)]
#[test_case(
    r#"(?:(?<n1>(?<n2>\B^)|(?<n3>\b$))){0,2}?(?<n4>)(?<n5>(?<n6>(?<n7>(?:(?<n8>(?<n9>(?<n10>(?<n11>(?<n12>$)|(?<n13>\B))(?<n14>(?<n15>){2,3})(?<n16>(?<n17>\b\B)|(?<n18>\b\B))|(?<n19>(?<n20>^)|(?<n21>$)){3}(?:(?<n22>(?<n23>[a-z\q{ab}]){1}){1,1}?){0,}?(?<n24>(?<n25>^)|(?<n26>\B)))){0001,0001}?)){0000}?))){0001,0001}(?<n27>)(?:(?<n28>(?<n29>\b^)|(?<n30>\B$)))?"#,
    "dgvi",
    JsString::from(""),
    1;
    "empty_input_from_1_variant_4"
)]
#[test_case(
    r#"(?:(?<n1>(?<n2>^)|(?<n3>\B)))(?<n4>)(?<n5>(?<n6>(?<n7>(?<n8>(?<n9>\b^)|(?<n10>$))(?:(?<n11>[\q{a|ab}]))*(?<n12>(?<n13>(?<n14>^)|(?<n15>\b))\B)|(?<n16>(?<n17>\b^)|(?<n18>\B$))(?<n19>(?<n20>){2,3})(?<n21>(?<n22>^)|(?<n23>\b)))){0})(?<n24>)(?:(?<n25>(?<n26>\b\B)|(?<n27>\b\B))){2,3}?"#,
    "dgvim",
    JsString::from("!ab!"),
    1;
    "input_ab_from_1_variant_5"
)]
#[test_case(
    r#"(?:(?<n1>(?<n2>^)|(?<n3>\B))){2,3}?(?<n4>)(?<n5>(?:(?<n6>(?<n7>(?:(?<n8>(?<n9>\b^)|(?<n10>$))(?:(?<n11>[\p{Any}\q{|ab}]){0001,0001}?){2,3}(?<n12>(?<n13>(?<n14>^)|(?<n15>\b))\B)|(?<n16>(?<n17>\b^)|(?<n18>\B$))(?<n19>(?<n20>){2,3})(?<n21>(?<n22>^)|(?<n23>\b))){1,1})){0000,0000})){1,1}?(?<n24>)(?:(?<n25>(?<n26>\b\B)|(?<n27>\b\B)))"#,
    "dgvi",
    JsString::from("ab\r\n"),
    1;
    "input_ab_from_1_variant_6"
)]
#[test_case(
    r#"(?:(?<n1>(?<n2>(?<n3>^)|(?<n4>\b))\B))(?<n5>)(?<n6>(?<n7>(?<n8>(?<n9>(?<n10>\B^)|(?<n11>\b$))(?:(?<n12>[\q{a|ab}]))*(?<n13>(?<n14>$)|(?<n15>\B))|(?<n16>(?<n17>\b\B)|(?<n18>\b\B))(?<n19>(?<n20>){2,3})(?<n21>(?<n22>$)|(?<n23>\b)){2,3}?)){0})(?<n24>)(?:(?<n25>(?<n26>^)|(?<n27>\b))){2,3}?"#,
    "dgvy",
    JsString::from("a\n!"),
    0;
    "input_a_from_0_variant_7"
)]
#[test_case(
    r#"(?:(?<n1>(?<n2>\b^)|(?<n3>\B$)))(?<n4>)(?<n5>(?<n6>(?<n7>(?<n8>(?<n9>(?<n10>^)|(?<n11>\b))\B)(?:(?<n12>[\q{a|ab}]))*(?<n13>(?<n14>\b\B)|(?<n15>\b\B))|(?<n16>(?<n17>$)|(?<n18>\b)){2,3}?(?<n19>(?<n20>){2,3})(?<n21>(?<n22>\b^)|(?<n23>$)))){0})(?<n24>)(?:(?<n25>(?<n26>^)|(?<n27>$)){3}){2,3}?"#,
    "dgvm",
    JsString::from("😀a"),
    0;
    "unicode_input_from_0_variant_8"
)]
#[test_case(
    r#"(?:(?<n1>(?<n2>\b^)|(?<n3>\B$)))?(?<n4>)(?:(?<n5>(?<n6>(?:(?:(?<n7>(?<n8>(?<n9>(?<n10>$)|(?<n11>\b)){2,3}?(?<n12>(?<n13>){2,3})(?<n14>(?<n15>\b^)|(?<n16>$))|(?<n17>(?<n18>(?<n19>^)|(?<n20>\b))\B)(?:(?<n21>(?<n22>[\q{|a|ab}])){1})+?(?<n23>(?<n24>\b\B)|(?<n25>\b\B)))))){0,0}?))){1}(?<n26>)(?:(?<n27>(?<n28>^)|(?<n29>$)){3}){0,2}?"#,
    "dgvi",
    JsString::from(""),
    1;
    "empty_input_from_1_variant_9"
)]
#[test_case(
    r#"(?:(?<n1>(?<n2>\b\B)|(?<n3>\b\B)))(?<n4>)(?<n5>(?<n6>(?<n7>(?<n8>(?<n9>$)|(?<n10>\B))(?:(?<n11>[\q{a|ab}]))*(?<n12>(?<n13>^)|(?<n14>\b))|(?<n15>(?<n16>^)|(?<n17>$)){3}(?<n18>(?<n19>){2,3})(?<n20>(?<n21>\B^)|(?<n22>\b$)))){0})(?<n23>)(?:(?<n24>(?<n25>\b^)|(?<n26>$))){2,3}?"#,
    "dgvi",
    JsString::from("ab\r\n"),
    1;
    "input_ab_from_1_variant_10"
)]
#[test_case(
    r#"(?:(?<n1>(?<n2>\b\B)|(?<n3>\b\B))){2,3}?(?<n4>)(?<n5>(?:(?<n6>(?<n7>(?:(?<n8>(?<n9>$)|(?<n10>\B))(?:(?<n11>[\p{Any}\q{|ab}]){0001,0001}?){2,3}(?<n12>(?<n13>^)|(?<n14>\b))|(?<n15>(?<n16>^)|(?<n17>$)){3}(?<n18>(?<n19>){2,3})(?<n20>(?<n21>\B^)|(?<n22>\b$))){1,1})){0000,0000})){1,1}?(?<n23>)(?:(?<n24>(?<n25>\b^)|(?<n26>$)))"#,
    "dgvim",
    JsString::from("!ab!"),
    1;
    "input_ab_from_1_variant_11"
)]
#[test_case(
    r#"(?:(?<n1>(?<n2>^)|(?<n3>\b))){2,3}?(?<n4>)(?<n5>(?:(?<n6>(?<n7>(?:(?<n8>(?<n9>\b^)|(?<n10>\B$))(?:(?<n11>[\p{Any}\q{|ab}]){0001,0001}?){2,3}(?<n12>(?<n13>$)|(?<n14>\b)){2,3}?|(?<n15>(?<n16>\b^)|(?<n17>$))(?<n18>(?<n19>){2,3})(?<n20>(?<n21>^)|(?<n22>\B))){1,1})){0000,0000})){1,1}?(?<n23>)(?:(?<n24>(?<n25>\B^)|(?<n26>\b$)))"#,
    "dgv",
    JsString::from("a\n!"),
    0;
    "input_a_from_0_variant_12"
)]
#[test_case(
    r#"(?:(?<n1>(?<n2>$)|(?<n3>\b)){2,3}?)(?<n4>)(?<n5>(?<n6>(?<n7>(?<n8>(?<n9>\b\B)|(?<n10>\b\B))(?:(?<n11>[\q{a|ab}]))*(?<n12>(?<n13>^)|(?<n14>$)){3}|(?<n15>(?<n16>\B^)|(?<n17>\b$))(?<n18>(?<n19>){2,3})(?<n20>(?<n21>(?<n22>^)|(?<n23>\b))\B))){0})(?<n24>)(?:(?<n25>(?<n26>^)|(?<n27>\B))){2,3}?"#,
    "dgvim",
    JsString::from("a"),
    1;
    "input_a_from_1_variant_13"
)]
#[test_case(
    r#"(?:(?<n1>(?<n2>$)|(?<n3>\b)){2,3}?){2,3}?(?<n4>)(?<n5>(?:(?<n6>(?<n7>(?:(?<n8>(?<n9>\b\B)|(?<n10>\b\B))(?:(?<n11>[\p{Any}\q{|ab}]){0001,0001}?){2,3}(?<n12>(?<n13>^)|(?<n14>$)){3}|(?<n15>(?<n16>\B^)|(?<n17>\b$))(?<n18>(?<n19>){2,3})(?<n20>(?<n21>(?<n22>^)|(?<n23>\b))\B)){1,1})){0000,0000})){1,1}?(?<n24>)(?:(?<n25>(?<n26>^)|(?<n27>\B)))"#,
    "dgvi",
    JsString::from("ſa"),
    1;
    "unicode_input_from_1_variant_14"
)]
fn no_match_resets_last_index(
    pattern: &str,
    flags: &str,
    input: JsString,
    initial_last_index: usize,
) {
    assert_no_match(pattern, flags, input, initial_last_index);
}
