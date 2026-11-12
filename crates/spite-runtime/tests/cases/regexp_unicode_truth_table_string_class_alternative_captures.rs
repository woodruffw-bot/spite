//! Required predicate choices preserve conditional lexical captures.
use crate::common::assert_true;
use crate::common::eval_string;
use crate::common::regexp_captures::{MatchExpectation, assert_named_captures};
use crate::common::regexp_matches::DESCRIBE_MATCH;
use spite_core::JsString;
use spite_runtime::{Realm, Value};
use test_case::test_case;
#[test]
fn matching_controls_and_string_consumers() {
    assert_true(
        r#"
            var pattern =
                "(?<n1>(?<n2>$)|(?<n3>\\b)){2,3}?(?<n4>(?<n5>)|(?<n6>[\\q{|a|ab}])){1}(?<n7>(?<n8>\\B^)|(?<n9>\\b$))",
                input = "!ab!",
                r = new RegExp(pattern, "dgv"),
                ok = r.source === pattern && r.global && r.hasIndices && r.unicodeSets === true && r
                .unicode === false;
            r.lastIndex = 0;
            var m = r.exec(input);
            ok && m === null && r.lastIndex === 0
        "#,
    );

    assert_true(
        r#"
            var pattern =
                "(?<n1>(?<n2>$)|(?<n3>\\b)){2,3}?(?<n4>(?<n5>)|(?<n6>[a-z\\q{ab}]){1}){1,1}?(?<n7>(?<n8>\\B^)|(?<n9>\\b$))",
                input = "ab\r\n",
                r = new RegExp(pattern, "dgvy"),
                ok = r.source === pattern && r.global && r.hasIndices && r.unicodeSets === true && r
                .unicode === false;
            r.lastIndex = 0;
            var m = r.exec(input);
            ok && m === null && r.lastIndex === 0
        "#,
    );

    assert_true(
        r#"
            var pattern =
                "(?<n1>(?<n2>^)|(?<n3>$)){3}(?<n4>(?<n5>)|(?<n6>[a-z\\q{ab}]){1}){1,1}?(?<n7>(?<n8>^)|(?<n9>\\B))",
                input = "",
                r = new RegExp(pattern, "dgvi"),
                ok = r.source === pattern && r.global && r.hasIndices && r.unicodeSets === true && r
                .unicode === false;
            r.lastIndex = 1;
            var m = r.exec(input);
            ok && m === null && r.lastIndex === 0
        "#,
    );
    assert_true(
        r#"
            var pattern =
                "(?<n1>(?<n2>\\b^)|(?<n3>$))(?<n4>(?<n5>[\\q{a|ab}])|(?<n6>))(?<n7>(?<n8>(?<n9>^)|(?<n10>\\b))\\B)",
                input = "!ab!",
                r = new RegExp(pattern, "dgvim"),
                ok = r.source === pattern && r.global && r.hasIndices && r.unicodeSets === true && r
                .unicode === false;
            r.lastIndex = 1;
            var m = r.exec(input);
            ok && m === null && r.lastIndex === 0
        "#,
    );
    assert_true(
        r#"
            var pattern =
                "(?<n1>(?<n2>\\b^)|(?<n3>$))(?<n4>(?<n5>)|(?<n6>[\\q{|a|ab}])){1}(?<n7>(?<n8>(?<n9>^)|(?<n10>\\b))\\B)",
                input = "\u017fa",
                r = new RegExp(pattern, "dgvy"),
                ok = r.source === pattern && r.global && r.hasIndices && r.unicodeSets === true && r
                .unicode === false;
            r.lastIndex = 0;
            var m = r.exec(input);
            ok && m === null && r.lastIndex === 0
        "#,
    );
    assert_true(
        r#"
            var pattern =
                "(?<n1>(?<n2>\\b^)|(?<n3>$))(?<n4>(?<n5>[\\p{Any}\\q{|ab}])|(?<n6>)){0001,0001}?(?<n7>(?<n8>(?<n9>^)|(?<n10>\\b))\\B)",
                input = "ab\r\n",
                r = new RegExp(pattern, "dgvi"),
                ok = r.source === pattern && r.global && r.hasIndices && r.unicodeSets === true && r
                .unicode === false;
            r.lastIndex = 1;
            var m = r.exec(input);
            ok && m === null && r.lastIndex === 0
        "#,
    );
    assert_true(
        r#"
            var pattern =
                "(?<n1>(?<n2>\\b^)|(?<n3>$))(?<n4>(?<n5>)|(?<n6>[a-z\\q{ab}]){1}){1,1}?(?<n7>(?<n8>(?<n9>^)|(?<n10>\\b))\\B)",
                input = "a",
                r = new RegExp(pattern, "dgvm"),
                ok = r.source === pattern && r.global && r.hasIndices && r.unicodeSets === true && r
                .unicode === false;
            r.lastIndex = 0;
            var m = r.exec(input);
            ok && m === null && r.lastIndex === 0
        "#,
    );
    assert_true(
        r#"
            var pattern =
                "(?<n1>(?<n2>\\B^)|(?<n3>\\b$))(?<n4>(?<n5>[\\q{a|ab}])|(?<n6>))(?<n7>(?<n8>$)|(?<n9>\\B))",
                input = "a\n!",
                r = new RegExp(pattern, "dgvy"),
                ok = r.source === pattern && r.global && r.hasIndices && r.unicodeSets === true && r
                .unicode === false;
            r.lastIndex = 0;
            var m = r.exec(input);
            ok && m === null && r.lastIndex === 0
        "#,
    );

    assert_true(
        r#"
            var pattern =
                "(?<n1>(?<n2>^)|(?<n3>\\B))(?<n4>(?<n5>[\\q{a|ab}])|(?<n6>))(?<n7>(?<n8>\\b^)|(?<n9>\\B$))",
                input = "\u017fa",
                r = new RegExp(pattern, "dgvi"),
                ok = r.source === pattern && r.global && r.hasIndices && r.unicodeSets === true && r
                .unicode === false;
            r.lastIndex = 1;
            var m = r.exec(input);
            ok && m === null && r.lastIndex === 0
        "#,
    );

    assert_true(
        r#"
            var pattern =
                "(?<n1>(?<n2>^)|(?<n3>\\B))(?<n4>(?<n5>[\\p{Any}\\q{|ab}])|(?<n6>)){0001,0001}?(?<n7>(?<n8>\\b^)|(?<n9>\\B$))",
                input = "a",
                r = new RegExp(pattern, "dgvi"),
                ok = r.source === pattern && r.global && r.hasIndices && r.unicodeSets === true && r
                .unicode === false;
            r.lastIndex = 1;
            var m = r.exec(input);
            ok && m === null && r.lastIndex === 0
        "#,
    );
    assert_true(
        r#"
            var pattern =
                "(?<n1>(?<n2>^)|(?<n3>\\B))(?<n4>(?<n5>)|(?<n6>[a-z\\q{ab}]){1}){1,1}?(?<n7>(?<n8>\\b^)|(?<n9>\\B$))",
                input = "!ab!",
                r = new RegExp(pattern, "dgvy"),
                ok = r.source === pattern && r.global && r.hasIndices && r.unicodeSets === true && r
                .unicode === false;
            r.lastIndex = 0;
            var m = r.exec(input);
            ok && m === null && r.lastIndex === 0
        "#,
    );
    assert_true(
        r#"
            var pattern =
                "(?<n1>(?<n2>(?<n3>^)|(?<n4>\\b))\\B)(?<n5>(?<n6>[\\q{a|ab}])|(?<n7>))(?<n8>(?<n9>\\b\\B)|(?<n10>\\b\\B))",
                input = "\ud83d\ude00a",
                r = new RegExp(pattern, "dgvm"),
                ok = r.source === pattern && r.global && r.hasIndices && r.unicodeSets === true && r
                .unicode === false;
            r.lastIndex = 0;
            var m = r.exec(input);
            ok && m === null && r.lastIndex === 0
        "#,
    );
    assert_true(
        r#"
            var pattern =
                "(?<n1>(?<n2>(?<n3>^)|(?<n4>\\b))\\B)(?<n5>(?<n6>)|(?<n7>[\\q{|a|ab}])){1}(?<n8>(?<n9>\\b\\B)|(?<n10>\\b\\B))",
                input = "",
                r = new RegExp(pattern, "dgvi"),
                ok = r.source === pattern && r.global && r.hasIndices && r.unicodeSets === true && r
                .unicode === false;
            r.lastIndex = 1;
            var m = r.exec(input);
            ok && m === null && r.lastIndex === 0
        "#,
    );
    assert_true(
        r#"
            var pattern =
                "(?<n1>(?<n2>(?<n3>^)|(?<n4>\\b))\\B)(?<n5>(?<n6>[\\p{Any}\\q{|ab}])|(?<n7>)){0001,0001}?(?<n8>(?<n9>\\b\\B)|(?<n10>\\b\\B))",
                input = "ab",
                r = new RegExp(pattern, "dgvy"),
                ok = r.source === pattern && r.global && r.hasIndices && r.unicodeSets === true && r
                .unicode === false;
            r.lastIndex = 0;
            var m = r.exec(input);
            ok && m === null && r.lastIndex === 0
        "#,
    );
    assert_true(
        r#"
            var pattern =
                "(?<n1>(?<n2>(?<n3>^)|(?<n4>\\b))\\B)(?<n5>(?<n6>)|(?<n7>[a-z\\q{ab}]){1}){1,1}?(?<n8>(?<n9>\\b\\B)|(?<n10>\\b\\B))",
                input = "a\n!",
                r = new RegExp(pattern, "dgvim"),
                ok = r.source === pattern && r.global && r.hasIndices && r.unicodeSets === true && r
                .unicode === false;
            r.lastIndex = 1;
            var m = r.exec(input);
            ok && m === null && r.lastIndex === 0
        "#,
    );
    assert_true(
        r#"
            var pattern =
                "(?<n1>(?<n2>$)|(?<n3>\\B))(?<n4>(?<n5>[\\q{a|ab}])|(?<n6>))(?<n7>(?<n8>^)|(?<n9>\\b))",
                input = "ab\r\n",
                r = new RegExp(pattern, "dgvi"),
                ok = r.source === pattern && r.global && r.hasIndices && r.unicodeSets === true && r
                .unicode === false;
            r.lastIndex = 1;
            var m = r.exec(input);
            ok && m === null && r.lastIndex === 0
        "#,
    );
    assert_true(
        r#"
            var pattern =
                "(?<n1>(?<n2>$)|(?<n3>\\B))(?<n4>(?<n5>)|(?<n6>[\\q{|a|ab}])){1}(?<n7>(?<n8>^)|(?<n9>\\b))",
                input = "a",
                r = new RegExp(pattern, "dgvy"),
                ok = r.source === pattern && r.global && r.hasIndices && r.unicodeSets === true && r
                .unicode === false;
            r.lastIndex = 0;
            var m = r.exec(input);
            ok && m === null && r.lastIndex === 0
        "#,
    );

    assert_true(
        r#"
            var pattern =
                "(?<n1>(?<n2>\\b^)|(?<n3>\\B$))(?<n4>(?<n5>)|(?<n6>[\\q{|a|ab}])){1}(?<n7>(?<n8>$)|(?<n9>\\b)){2,3}?",
                input = "ab",
                r = new RegExp(pattern, "dgvim"),
                ok = r.source === pattern && r.global && r.hasIndices && r.unicodeSets === true && r
                .unicode === false;
            r.lastIndex = 1;
            var m = r.exec(input);
            ok && m === null && r.lastIndex === 0
        "#,
    );

    assert_true(
        r#"
            var pattern =
                "(?<n1>(?<n2>\\b^)|(?<n3>\\B$))(?<n4>(?<n5>)|(?<n6>[a-z\\q{ab}]){1}){1,1}?(?<n7>(?<n8>$)|(?<n9>\\b)){2,3}?",
                input = "\ud83d\ude00a",
                r = new RegExp(pattern, "dgvi"),
                ok = r.source === pattern && r.global && r.hasIndices && r.unicodeSets === true && r
                .unicode === false;
            r.lastIndex = 1;
            var m = r.exec(input);
            ok && m === null && r.lastIndex === 0
        "#,
    );
    assert_true(
        r#"
            var pattern =
                "(?<n1>(?<n2>\\b\\B)|(?<n3>\\b\\B))(?<n4>(?<n5>[\\q{a|ab}])|(?<n6>))(?<n7>(?<n8>^)|(?<n9>$)){3}",
                input = "a",
                r = new RegExp(pattern, "dgvim"),
                ok = r.source === pattern && r.global && r.hasIndices && r.unicodeSets === true && r
                .unicode === false;
            r.lastIndex = 1;
            var m = r.exec(input);
            ok && m === null && r.lastIndex === 0
        "#,
    );
    assert_true(
        r#"
            var pattern =
                "(?<n1>(?<n2>\\b\\B)|(?<n3>\\b\\B))(?<n4>(?<n5>)|(?<n6>[\\q{|a|ab}])){1}(?<n7>(?<n8>^)|(?<n9>$)){3}",
                input = "!ab!",
                r = new RegExp(pattern, "dgv"),
                ok = r.source === pattern && r.global && r.hasIndices && r.unicodeSets === true && r
                .unicode === false;
            r.lastIndex = 0;
            var m = r.exec(input);
            ok && m === null && r.lastIndex === 0
        "#,
    );
    assert_true(
        r#"
            var pattern =
                "(?<n1>(?<n2>\\b\\B)|(?<n3>\\b\\B))(?<n4>(?<n5>[\\p{Any}\\q{|ab}])|(?<n6>)){0001,0001}?(?<n7>(?<n8>^)|(?<n9>$)){3}",
                input = "\u017fa",
                r = new RegExp(pattern, "dgvi"),
                ok = r.source === pattern && r.global && r.hasIndices && r.unicodeSets === true && r
                .unicode === false;
            r.lastIndex = 1;
            var m = r.exec(input);
            ok && m === null && r.lastIndex === 0
        "#,
    );
    assert_true(
        r#"
            var pattern =
                "(?<n1>(?<n2>\\b\\B)|(?<n3>\\b\\B))(?<n4>(?<n5>)|(?<n6>[a-z\\q{ab}]){1}){1,1}?(?<n7>(?<n8>^)|(?<n9>$)){3}",
                input = "ab\r\n",
                r = new RegExp(pattern, "dgvym"),
                ok = r.source === pattern && r.global && r.hasIndices && r.unicodeSets === true && r
                .unicode === false;
            r.lastIndex = 0;
            var m = r.exec(input);
            ok && m === null && r.lastIndex === 0
        "#,
    );

    let mut realm = Realm::default();
    realm.eval(r"var r=/(?<b>(?<s>^)|(?<w>\b))(?<whole>(?<c>[\q{a|ab}])|(?<e>))(?<a>(?<end>$)|(?<nw>\B))/dvg,clone=new RegExp(r),m=r.exec('ab!aa')").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval(r#"
        var n = clone.exec('ab');
        m[0] === 'a' && m.groups.s === '' && m.groups.w === undefined && m.groups.c === 'a' && m.groups
            .e === undefined && m.groups.end === undefined && m.groups.nw === '' && m.indices.groups
            .c === m.indices[5] && m.indices[5][0] === 0 && m.indices[5][1] === 1 && n.groups.c ===
            'ab' && n.groups.e === undefined && n.groups.end === '' && n.groups.nw === undefined &&
            clone.source === r.source && clone.lastIndex === 2 && r.lastIndex === 1
    "#),Ok(Value::Boolean(true)));
}

#[test]
fn match_all_preserves_capture_metadata() {
    let source = format!(
        r#"
            (function() {{
                {DESCRIBE_MATCH}

                var P =
                    "(?<b>(?<s>^)|(?<w>\\b))(?<whole>(?<c>[\\q{{|a|ab}}])|(?<e>))(?<a>(?<end>$)|(?<nw>\\B))";
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
            "(?<b>(?<s>^)|(?<w>\\b))(?<whole>(?<c>[\\q{|a|ab}])|(?<e>))(?<a>(?<end>$)|(?<nw>\\B))";
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
                "(?<b>(?<s>^)|(?<w>\\b))(?<whole>(?<c>[\\q{|a|ab}])|(?<e>))(?<a>(?<end>$)|(?<nw>\\B))";
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
            "<undefined>",
            "",
            "ab",
            "ab",
            "<undefined>",
            "",
            "<undefined>",
            "",
            "aa!"
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
            "(?<b>(?<s>^)|(?<w>\\b))(?<whole>(?<c>[\\q{|a|ab}])|(?<e>))(?<a>(?<end>$)|(?<nw>\\B))";
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
                    "(?<b>(?<s>^)|(?<w>\\b))(?<whole>(?<c>[\\q{{|a|ab}}])|(?<e>))(?<a>(?<end>$)|(?<nw>\\B))";
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
                    "(?<b>(?<s>^)|(?<w>\\b))(?<whole>(?<c>[a-z\\q{{|S|sa|aX}}])|(?<e>))(?<a>(?<end>$)|(?<nw>\\B))";
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
                    "(?<b>(?<s>^)|(?<w>\\b))(?<whole>(?<c>[\\q{{|a|ab}}])|(?<e>))(?<a>(?<end>$)|(?<nw>\\B))";
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
                    "(?<\\u{{1D49C}}>(?<s>^)|(?<w>\\b))(?<whole>(?<c>[\\q{{|a|ab}}])|(?<e>))(?<a>(?<end>$)|(?<nw>\\B))";
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
    r#"(?<n1>(?<n2>^)|(?<n3>\b))(?<n4>(?<n5>[\q{a|ab}])|(?<n6>))(?<n7>(?<n8>\b^)|(?<n9>$))"#,
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
            None,
            Some(""),
            Some(""),
            None,
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
            None,
            Some((0, 0)),
        ],
        last_index: 0,
    };
    "empty_input_from_0_variant_1"
)]
#[test_case(
    r#"(?<n1>(?<n2>^)|(?<n3>\b))(?<n4>(?<n5>)|(?<n6>[\q{|a|ab}])){1}(?<n7>(?<n8>\b^)|(?<n9>$))"#,
    "dgvi",
    JsString::from("ab"),
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
            Some(""),
            None,
            Some(""),
        ],
        capture_ranges: &[
            Some((2, 2)),
            None,
            Some((2, 2)),
            Some((2, 2)),
            Some((2, 2)),
            None,
            Some((2, 2)),
            None,
            Some((2, 2)),
        ],
        last_index: 2,
    };
    "input_ab_from_1_variant_2"
)]
#[test_case(
    r#"(?<n1>(?<n2>^)|(?<n3>\b))(?<n4>(?<n5>[\p{Any}\q{|ab}])|(?<n6>)){0001,0001}?(?<n7>(?<n8>\b^)|(?<n9>$))"#,
    "dgv",
    JsString::from("a\n!"),
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
            Some((0, 0)),
            Some((0, 0)),
            None,
        ],
        last_index: 0,
    };
    "input_a_from_0_variant_3"
)]
#[test_case(
    r#"(?<n1>(?<n2>^)|(?<n3>\b))(?<n4>(?<n5>)|(?<n6>[a-z\q{ab}]){1}){1,1}?(?<n7>(?<n8>\b^)|(?<n9>$))"#,
    "dgvim",
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
            None,
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
            None,
            Some((2, 3)),
            Some((3, 3)),
            None,
            Some((3, 3)),
        ],
        last_index: 3,
    };
    "unicode_input_from_1_variant_4"
)]
#[test_case(
    r#"(?<n1>(?<n2>$)|(?<n3>\b)){2,3}?(?<n4>(?<n5>[\q{a|ab}])|(?<n6>))(?<n7>(?<n8>\B^)|(?<n9>\b$))"#,
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
            None,
            Some(""),
            Some(""),
            None,
            Some(""),
        ],
        capture_ranges: &[
            Some((1, 1)),
            Some((1, 1)),
            None,
            Some((1, 1)),
            None,
            Some((1, 1)),
            Some((1, 1)),
            None,
            Some((1, 1)),
        ],
        last_index: 1,
    };
    "input_a_from_1_variant_5"
)]
#[test_case(
    r#"(?<n1>(?<n2>$)|(?<n3>\b)){2,3}?(?<n4>(?<n5>[\p{Any}\q{|ab}])|(?<n6>)){0001,0001}?(?<n7>(?<n8>\B^)|(?<n9>\b$))"#,
    "dgvim",
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
            Some(""),
            None,
            Some(""),
        ],
        capture_ranges: &[
            Some((2, 2)),
            Some((2, 2)),
            None,
            Some((2, 2)),
            Some((2, 2)),
            None,
            Some((2, 2)),
            None,
            Some((2, 2)),
        ],
        last_index: 2,
    };
    "unicode_input_from_1_variant_6"
)]
#[test_case(
    r#"(?<n1>(?<n2>^)|(?<n3>$)){3}(?<n4>(?<n5>[\q{a|ab}])|(?<n6>))(?<n7>(?<n8>^)|(?<n9>\B))"#,
    "dgv",
    JsString::from("ab"),
    0,
    MatchExpectation {
        text: "a",
        range: 0..1,
        captures: &[
            Some(""),
            Some(""),
            None,
            Some("a"),
            Some("a"),
            None,
            Some(""),
            None,
            Some(""),
        ],
        capture_ranges: &[
            Some((0, 0)),
            Some((0, 0)),
            None,
            Some((0, 1)),
            Some((0, 1)),
            None,
            Some((1, 1)),
            None,
            Some((1, 1)),
        ],
        last_index: 1,
    };
    "input_ab_from_0_variant_7"
)]
#[test_case(
    r#"(?<n1>(?<n2>^)|(?<n3>$)){3}(?<n4>(?<n5>)|(?<n6>[\q{|a|ab}])){1}(?<n7>(?<n8>^)|(?<n9>\B))"#,
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
            Some(""),
            Some(""),
            None,
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
            Some((2, 2)),
            Some((2, 2)),
            None,
        ],
        last_index: 2,
    };
    "input_a_from_1_variant_8"
)]
#[test_case(
    r#"(?<n1>(?<n2>^)|(?<n3>$)){3}(?<n4>(?<n5>[\p{Any}\q{|ab}])|(?<n6>)){0001,0001}?(?<n7>(?<n8>^)|(?<n9>\B))"#,
    "dgvy",
    JsString::from("😀a"),
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
            Some((0, 0)),
            Some((0, 0)),
            None,
        ],
        last_index: 0,
    };
    "unicode_input_from_0_variant_9"
)]
#[test_case(
    r#"(?<n1>(?<n2>\B^)|(?<n3>\b$))(?<n4>(?<n5>)|(?<n6>[\q{|a|ab}])){1}(?<n7>(?<n8>$)|(?<n9>\B))"#,
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
            Some(""),
            Some(""),
            None,
            Some(""),
            None,
            Some(""),
        ],
        capture_ranges: &[
            Some((0, 0)),
            Some((0, 0)),
            None,
            Some((0, 0)),
            Some((0, 0)),
            None,
            Some((0, 0)),
            None,
            Some((0, 0)),
        ],
        last_index: 0,
    };
    "unicode_input_from_1_variant_10"
)]
#[test_case(
    r#"(?<n1>(?<n2>\B^)|(?<n3>\b$))(?<n4>(?<n5>[\p{Any}\q{|ab}])|(?<n6>)){0001,0001}?(?<n7>(?<n8>$)|(?<n9>\B))"#,
    "dgvm",
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
            Some((0, 0)),
            Some((0, 0)),
            None,
        ],
        last_index: 0,
    };
    "empty_input_from_0_variant_11"
)]
#[test_case(
    r#"(?<n1>(?<n2>\B^)|(?<n3>\b$))(?<n4>(?<n5>)|(?<n6>[a-z\q{ab}]){1}){1,1}?(?<n7>(?<n8>$)|(?<n9>\B))"#,
    "dgvi",
    JsString::from("ab"),
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
            Some((2, 2)),
            Some((2, 2)),
            None,
        ],
        last_index: 2,
    };
    "input_ab_from_1_variant_12"
)]
#[test_case(
    r#"(?<n1>(?<n2>^)|(?<n3>\B))(?<n4>(?<n5>)|(?<n6>[\q{|a|ab}])){1}(?<n7>(?<n8>\b^)|(?<n9>\B$))"#,
    "dgvm",
    JsString::from("ab\r\n"),
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
            Some((0, 0)),
            Some((0, 0)),
            None,
        ],
        last_index: 0,
    };
    "input_ab_from_0_variant_13"
)]
#[test_case(
    r#"(?<n1>(?<n2>$)|(?<n3>\B))(?<n4>(?<n5>[\p{Any}\q{|ab}])|(?<n6>)){0001,0001}?(?<n7>(?<n8>^)|(?<n9>\b))"#,
    "dgvim",
    JsString::from("!ab!"),
    1,
    MatchExpectation {
        text: "b",
        range: 2..3,
        captures: &[
            Some(""),
            None,
            Some(""),
            Some("b"),
            Some("b"),
            None,
            Some(""),
            None,
            Some(""),
        ],
        capture_ranges: &[
            Some((2, 2)),
            None,
            Some((2, 2)),
            Some((2, 3)),
            Some((2, 3)),
            None,
            Some((3, 3)),
            None,
            Some((3, 3)),
        ],
        last_index: 3,
    };
    "input_ab_from_1_variant_14"
)]
#[test_case(
    r#"(?<n1>(?<n2>$)|(?<n3>\B))(?<n4>(?<n5>)|(?<n6>[a-z\q{ab}]){1}){1,1}?(?<n7>(?<n8>^)|(?<n9>\b))"#,
    "dgv",
    JsString::from("ſa"),
    0,
    MatchExpectation {
        text: "",
        range: 0..0,
        captures: &[
            Some(""),
            None,
            Some(""),
            Some(""),
            Some(""),
            None,
            Some(""),
            Some(""),
            None,
        ],
        capture_ranges: &[
            Some((0, 0)),
            None,
            Some((0, 0)),
            Some((0, 0)),
            Some((0, 0)),
            None,
            Some((0, 0)),
            Some((0, 0)),
            None,
        ],
        last_index: 0,
    };
    "unicode_input_from_0_variant_15"
)]
#[test_case(
    r#"(?<n1>(?<n2>\b^)|(?<n3>\B$))(?<n4>(?<n5>[\q{a|ab}])|(?<n6>))(?<n7>(?<n8>$)|(?<n9>\b)){2,3}?"#,
    "dgvy",
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
            Some(""),
            Some(""),
            None,
        ],
        capture_ranges: &[
            Some((0, 0)),
            None,
            Some((0, 0)),
            Some((0, 0)),
            None,
            Some((0, 0)),
            Some((0, 0)),
            Some((0, 0)),
            None,
        ],
        last_index: 0,
    };
    "empty_input_from_0_variant_16"
)]
#[test_case(
    r#"(?<n1>(?<n2>\b^)|(?<n3>\B$))(?<n4>(?<n5>[\p{Any}\q{|ab}])|(?<n6>)){0001,0001}?(?<n7>(?<n8>$)|(?<n9>\b)){2,3}?"#,
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
            Some("a"),
            None,
            Some(""),
            None,
            Some(""),
        ],
        capture_ranges: &[
            Some((0, 0)),
            Some((0, 0)),
            None,
            Some((0, 1)),
            Some((0, 1)),
            None,
            Some((1, 1)),
            None,
            Some((1, 1)),
        ],
        last_index: 1,
    };
    "input_a_from_0_variant_17"
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
fn truth_table_alternative_string_classes_keep_branch_ownership_and_unknown_proofs() {
    assert_true(
        r####"var pattern="(((\\b)|(\\b))([\\q{a|ab}]))|()",input="aba",r=new RegExp(pattern,"dv"),m=r.exec(input);r.source===pattern&&r.flags==="dv"&&r.hasIndices&&r.unicodeSets&&(m===null||(m.input===input&&m.groups===undefined&&m.indices.groups===undefined&&Object.hasOwn(m,"groups")&&Object.hasOwn(m.indices,"groups")))&&JSON.stringify(m===null?null:[Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex])==="[[\"ab\",\"ab\",\"\",\"\",\"<undefined>\",\"ab\",\"<undefined>\"],[[0,2],[0,2],[0,0],[0,0],\"<undefined>\",[0,2],\"<undefined>\"],0,0]""####,
    );
    assert_true(
        r####"var pattern="((\\b)|(\\b))(((\\b)|(\\b))([\\q{a|ab}])|())",input="aba",r=new RegExp(pattern,"dv"),m=r.exec(input);r.source===pattern&&r.flags==="dv"&&r.hasIndices&&r.unicodeSets&&(m===null||(m.input===input&&m.groups===undefined&&m.indices.groups===undefined&&Object.hasOwn(m,"groups")&&Object.hasOwn(m.indices,"groups")))&&JSON.stringify(m===null?null:[Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex])==="[[\"ab\",\"\",\"\",\"<undefined>\",\"ab\",\"\",\"\",\"<undefined>\",\"ab\",\"<undefined>\"],[[0,2],[0,0],[0,0],\"<undefined>\",[0,2],[0,0],[0,0],\"<undefined>\",[0,2],\"<undefined>\"],0,0]""####,
    );
    for pattern in [
        r"((\b)|z|())(([\q{a|ab}])|())",
        r"((\b)|\1|())(([\q{a|ab}])|())",
        r"((\b\B)|(\b\B))(()|([\q{ab}&&\p{RGI_Emoji}]))",
        r"((\b)|(\b))(([\q{a|ab}])|())[a]",
        r"((\b)|(\b))((([\q{a|ab}])|()){2}){2}",
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

#[test]
fn truth_table_alternative_string_classes_keep_branch_specific_reversed_line_gates() {
    assert_true(
        r#"
            var r = /((^)|(\b))(($(?<c>[\q{\n|\na}])^)|(?<e>))(($)|(\b))/dvm,
                m = r.exec('\na');
            m[0] === '\n' && m.groups.c === '\n' && m.groups.e === undefined && m.indices.groups.c[0] ===
                0 && m.indices.groups.c[1] === 1
        "#,
    );
    assert_true(
        r#"
            var r = /((^)|(\b))(?<whole>(?<e>)|(?<c>[\q{a|ab}]))(($)|(\B))/dv,
                m = r.exec('ab'),
                n = r.exec('');
            m[0] === 'ab' && m.groups.c === 'ab' && m.groups.e === undefined && n[0] === '' && n.groups
                .c === undefined && n.groups.e === ''
        "#,
    );
}
