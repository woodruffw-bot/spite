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
                "(?<n1>(?<n2>$)|(?<n3>\\b)){2,3}?(?<n4>(?<n5>[\\q{a|ab}])|(?<n6>)){3,4}(?<n7>(?<n8>\\B^)|(?<n9>\\b$))",
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
                "(?<n1>(?<n2>$)|(?<n3>\\b)){2,3}?(?<n4>[\\p{Any}\\q{|ab}]){3,4}(?<n5>(?<n6>\\B^)|(?<n7>\\b$))",
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
                "(?<n1>(?<n2>^)|(?<n3>$)){3}(?<n4>[\\p{Any}\\q{|ab}]){3,4}(?<n5>(?<n6>^)|(?<n7>\\B))",
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
                "(?<n1>(?<n2>\\b^)|(?<n3>$))(?<n4>[\\q{|a|ab}]){3}(?<n5>(?<n6>(?<n7>^)|(?<n8>\\b))\\B)",
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
                "(?<n1>(?<n2>\\b^)|(?<n3>$))(?<n4>(?<n5>[\\q{a|ab}])|(?<n6>)){3,4}(?<n7>(?<n8>(?<n9>^)|(?<n10>\\b))\\B)",
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
                "(?<n1>(?<n2>\\b^)|(?<n3>$))(?<n4>(?<n5>)|(?<n6>[\\q{|a|ab}])){3,}(?<n7>(?<n8>(?<n9>^)|(?<n10>\\b))\\B)",
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
                "(?<n1>(?<n2>\\b^)|(?<n3>$))(?<n4>[\\p{Any}\\q{|ab}]){3,4}(?<n5>(?<n6>(?<n7>^)|(?<n8>\\b))\\B)",
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
            var pattern = "(?<n1>(?<n2>\\B^)|(?<n3>\\b$))(?<n4>[\\q{|a|ab}]){3}(?<n5>(?<n6>$)|(?<n7>\\B))",
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
            var pattern = "(?<n1>(?<n2>^)|(?<n3>\\B))(?<n4>[\\q{|a|ab}]){3}(?<n5>(?<n6>\\b^)|(?<n7>\\B$))",
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
                "(?<n1>(?<n2>^)|(?<n3>\\B))(?<n4>(?<n5>)|(?<n6>[\\q{|a|ab}])){3,}(?<n7>(?<n8>\\b^)|(?<n9>\\B$))",
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
                "(?<n1>(?<n2>(?<n3>^)|(?<n4>\\b))\\B)(?<n5>[\\q{|a|ab}]){3}(?<n6>(?<n7>\\b\\B)|(?<n8>\\b\\B))",
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
                "(?<n1>(?<n2>(?<n3>^)|(?<n4>\\b))\\B)(?<n5>(?<n6>[\\q{a|ab}])|(?<n7>)){3,4}(?<n8>(?<n9>\\b\\B)|(?<n10>\\b\\B))",
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
                "(?<n1>(?<n2>(?<n3>^)|(?<n4>\\b))\\B)(?<n5>(?<n6>)|(?<n7>[\\q{|a|ab}])){3,}(?<n8>(?<n9>\\b\\B)|(?<n10>\\b\\B))",
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
                "(?<n1>(?<n2>(?<n3>^)|(?<n4>\\b))\\B)(?<n5>[\\p{Any}\\q{|ab}]){3,4}(?<n6>(?<n7>\\b\\B)|(?<n8>\\b\\B))",
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
            var pattern = "(?<n1>(?<n2>$)|(?<n3>\\B))(?<n4>[\\q{|a|ab}]){3}(?<n5>(?<n6>^)|(?<n7>\\b))",
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
                "(?<n1>(?<n2>$)|(?<n3>\\B))(?<n4>(?<n5>[\\q{a|ab}])|(?<n6>)){3,4}(?<n7>(?<n8>^)|(?<n9>\\b))",
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
                "(?<n1>(?<n2>$)|(?<n3>\\B))(?<n4>(?<n5>)|(?<n6>[\\q{|a|ab}])){3,}(?<n7>(?<n8>^)|(?<n9>\\b))",
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
                "(?<n1>(?<n2>\\b^)|(?<n3>\\B$))(?<n4>(?<n5>[\\q{a|ab}])|(?<n6>)){3,4}(?<n7>(?<n8>$)|(?<n9>\\b)){2,3}?",
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
                "(?<n1>(?<n2>\\b^)|(?<n3>\\B$))(?<n4>[\\p{Any}\\q{|ab}]){3,4}(?<n5>(?<n6>$)|(?<n7>\\b)){2,3}?",
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
                "(?<n1>(?<n2>\\b\\B)|(?<n3>\\b\\B))(?<n4>[\\q{|a|ab}]){3}(?<n5>(?<n6>^)|(?<n7>$)){3}",
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
                "(?<n1>(?<n2>\\b\\B)|(?<n3>\\b\\B))(?<n4>(?<n5>[\\q{a|ab}])|(?<n6>)){3,4}(?<n7>(?<n8>^)|(?<n9>$)){3}",
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
                "(?<n1>(?<n2>\\b\\B)|(?<n3>\\b\\B))(?<n4>(?<n5>)|(?<n6>[\\q{|a|ab}])){3,}(?<n7>(?<n8>^)|(?<n9>$)){3}",
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
                "(?<n1>(?<n2>\\b\\B)|(?<n3>\\b\\B))(?<n4>[\\p{Any}\\q{|ab}]){3,4}(?<n5>(?<n6>^)|(?<n7>$)){3}",
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
    realm.eval(r"var r=/(?<b>(?<s>^)|(?<w>\b))(?<whole>(?<it>(?<e>)|(?<c>[\q{a|ab}])){3})(?<a>(?<end>$)|(?<nw>\B))/dvg,clone=new RegExp(r),m=r.exec('ab!aa')").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval(r#"
        var n = clone.exec('ab');
        m[0] === 'a' && m.groups.s === '' && m.groups.w === undefined && m.groups.c === 'a' && m.groups
            .e === undefined && m.groups.end === undefined && m.groups.nw === '' && m.indices.groups
            .c === m.indices[7] && m.indices[7][0] === 0 && m.indices[7][1] === 1 && n.groups.c ===
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
                    "(?<b>(?<s>^)|(?<w>\\b))(?<whole>(?<it>(?<e>)|(?<c>[\\q{{a|ab}}])){{3}})(?<a>(?<end>$)|(?<nw>\\B))";
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
            "(?<b>(?<s>^)|(?<w>\\b))(?<whole>(?<it>(?<e>)|(?<c>[\\q{a|ab}])){3})(?<a>(?<end>$)|(?<nw>\\B))";
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
                "(?<b>(?<s>^)|(?<w>\\b))(?<whole>(?<it>(?<e>)|(?<c>[\\q{a|ab}])){3})(?<a>(?<end>$)|(?<nw>\\B))";
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
            "ab",
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
            "(?<b>(?<s>^)|(?<w>\\b))(?<whole>(?<it>(?<e>)|(?<c>[\\q{a|ab}])){3})(?<a>(?<end>$)|(?<nw>\\B))";
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
                    "(?<b>(?<s>^)|(?<w>\\b))(?<whole>(?<it>(?<e>)|(?<c>[\\q{{a|ab}}])){{3}})(?<a>(?<end>$)|(?<nw>\\B))";
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
                    "(?<b>(?<s>^)|(?<w>\\b))(?<whole>(?<it>(?<e>)|(?<c>[a-z\\q{{|S|sa|aX}}])){{3}})(?<a>(?<end>$)|(?<nw>\\B))";
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
                    "(?<b>(?<s>^)|(?<w>\\b))(?<whole>(?<it>(?<e>)|(?<c>[\\q{{a|ab}}])){{3}})(?<a>(?<end>$)|(?<nw>\\B))";
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
                    "(?<\\u{{1D49C}}>(?<s>^)|(?<w>\\b))(?<whole>(?<it>(?<e>)|(?<c>[\\q{{a|ab}}])){{3}})(?<a>(?<end>$)|(?<nw>\\B))";
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
    r#"(?<n1>(?<n2>^)|(?<n3>\b))(?<n4>[\q{|a|ab}]){3}(?<n5>(?<n6>\b^)|(?<n7>$))"#,
    "dgvym",
    JsString::from(""),
    0,
    MatchExpectation {
        text: "",
        range: 0..0,
        captures: &[Some(""), Some(""), None, Some(""), Some(""), None, Some("")],
        capture_ranges: &[
            Some((0, 0)),
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
    r#"(?<n1>(?<n2>^)|(?<n3>\b))(?<n4>(?<n5>[\q{a|ab}])|(?<n6>)){3,4}(?<n7>(?<n8>\b^)|(?<n9>$))"#,
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
            None,
            Some(""),
            Some(""),
            None,
            Some(""),
        ],
        capture_ranges: &[
            Some((2, 2)),
            None,
            Some((2, 2)),
            Some((2, 2)),
            None,
            Some((2, 2)),
            Some((2, 2)),
            None,
            Some((2, 2)),
        ],
        last_index: 2,
    };
    "input_ab_from_1_variant_2"
)]
#[test_case(
    r#"(?<n1>(?<n2>^)|(?<n3>\b))(?<n4>(?<n5>)|(?<n6>[\q{|a|ab}])){3,}(?<n7>(?<n8>\b^)|(?<n9>$))"#,
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
    r#"(?<n1>(?<n2>^)|(?<n3>\b))(?<n4>[\p{Any}\q{|ab}]){3,4}(?<n5>(?<n6>\b^)|(?<n7>$))"#,
    "dgvim",
    JsString::from("😀a"),
    1,
    MatchExpectation {
        text: "😀a",
        range: 0..3,
        captures: &[Some(""), Some(""), None, Some(""), Some(""), None, Some("")],
        capture_ranges: &[
            Some((0, 0)),
            Some((0, 0)),
            None,
            Some((3, 3)),
            Some((3, 3)),
            None,
            Some((3, 3)),
        ],
        last_index: 3,
    };
    "unicode_input_from_1_variant_4"
)]
#[test_case(
    r#"(?<n1>(?<n2>$)|(?<n3>\b)){2,3}?(?<n4>[\q{|a|ab}]){3}(?<n5>(?<n6>\B^)|(?<n7>\b$))"#,
    "dgvi",
    JsString::from("a"),
    1,
    MatchExpectation {
        text: "",
        range: 1..1,
        captures: &[Some(""), Some(""), None, Some(""), Some(""), None, Some("")],
        capture_ranges: &[
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
    "input_a_from_1_variant_5"
)]
#[test_case(
    r#"(?<n1>(?<n2>$)|(?<n3>\b)){2,3}?(?<n4>(?<n5>)|(?<n6>[\q{|a|ab}])){3,}(?<n7>(?<n8>\B^)|(?<n9>\b$))"#,
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
    r#"(?<n1>(?<n2>^)|(?<n3>$)){3}(?<n4>[\q{|a|ab}]){3}(?<n5>(?<n6>^)|(?<n7>\B))"#,
    "dgv",
    JsString::from("ab"),
    0,
    MatchExpectation {
        text: "a",
        range: 0..1,
        captures: &[Some(""), Some(""), None, Some(""), Some(""), None, Some("")],
        capture_ranges: &[
            Some((0, 0)),
            Some((0, 0)),
            None,
            Some((1, 1)),
            Some((1, 1)),
            None,
            Some((1, 1)),
        ],
        last_index: 1,
    };
    "input_ab_from_0_variant_7"
)]
#[test_case(
    r#"(?<n1>(?<n2>^)|(?<n3>$)){3}(?<n4>(?<n5>[\q{a|ab}])|(?<n6>)){3,4}(?<n7>(?<n8>^)|(?<n9>\B))"#,
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
            None,
            Some((2, 2)),
            Some((2, 2)),
            Some((2, 2)),
            None,
        ],
        last_index: 2,
    };
    "input_a_from_1_variant_8"
)]
#[test_case(
    r#"(?<n1>(?<n2>^)|(?<n3>$)){3}(?<n4>(?<n5>)|(?<n6>[\q{|a|ab}])){3,}(?<n7>(?<n8>^)|(?<n9>\B))"#,
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
    r#"(?<n1>(?<n2>\B^)|(?<n3>\b$))(?<n4>(?<n5>[\q{a|ab}])|(?<n6>)){3,4}(?<n7>(?<n8>$)|(?<n9>\B))"#,
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
    "unicode_input_from_1_variant_10"
)]
#[test_case(
    r#"(?<n1>(?<n2>\B^)|(?<n3>\b$))(?<n4>(?<n5>)|(?<n6>[\q{|a|ab}])){3,}(?<n7>(?<n8>$)|(?<n9>\B))"#,
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
    r#"(?<n1>(?<n2>\B^)|(?<n3>\b$))(?<n4>[\p{Any}\q{|ab}]){3,4}(?<n5>(?<n6>$)|(?<n7>\B))"#,
    "dgvi",
    JsString::from("ab"),
    1,
    MatchExpectation {
        text: "",
        range: 2..2,
        captures: &[Some(""), None, Some(""), Some(""), Some(""), Some(""), None],
        capture_ranges: &[
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
    "input_ab_from_1_variant_12"
)]
#[test_case(
    r#"(?<n1>(?<n2>^)|(?<n3>\B))(?<n4>(?<n5>[\q{a|ab}])|(?<n6>)){3,4}(?<n7>(?<n8>\b^)|(?<n9>\B$))"#,
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
            None,
            Some((0, 0)),
            Some((0, 0)),
            Some((0, 0)),
            None,
        ],
        last_index: 0,
    };
    "input_ab_from_0_variant_13"
)]
#[test_case(
    r#"(?<n1>(?<n2>^)|(?<n3>\B))(?<n4>[\p{Any}\q{|ab}]){3,4}(?<n5>(?<n6>\b^)|(?<n7>\B$))"#,
    "dgvy",
    JsString::from("!ab!"),
    0,
    MatchExpectation {
        text: "!ab!",
        range: 0..4,
        captures: &[
            Some(""),
            Some(""),
            None,
            Some("!"),
            Some(""),
            None,
            Some(""),
        ],
        capture_ranges: &[
            Some((0, 0)),
            Some((0, 0)),
            None,
            Some((3, 4)),
            Some((4, 4)),
            None,
            Some((4, 4)),
        ],
        last_index: 4,
    };
    "input_ab_from_0_variant_14"
)]
#[test_case(
    r#"(?<n1>(?<n2>$)|(?<n3>\B))(?<n4>[\p{Any}\q{|ab}]){3,4}(?<n5>(?<n6>^)|(?<n7>\b))"#,
    "dgv",
    JsString::from("ſa"),
    0,
    MatchExpectation {
        text: "ſa",
        range: 0..2,
        captures: &[Some(""), None, Some(""), Some(""), Some(""), None, Some("")],
        capture_ranges: &[
            Some((0, 0)),
            None,
            Some((0, 0)),
            Some((2, 2)),
            Some((2, 2)),
            None,
            Some((2, 2)),
        ],
        last_index: 2,
    };
    "unicode_input_from_0_variant_15"
)]
#[test_case(
    r#"(?<n1>(?<n2>\b^)|(?<n3>\B$))(?<n4>[\q{|a|ab}]){3}(?<n5>(?<n6>$)|(?<n7>\b)){2,3}?"#,
    "dgvy",
    JsString::from(""),
    0,
    MatchExpectation {
        text: "",
        range: 0..0,
        captures: &[Some(""), None, Some(""), Some(""), Some(""), Some(""), None],
        capture_ranges: &[
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
    "empty_input_from_0_variant_16"
)]
#[test_case(
    r#"(?<n1>(?<n2>\b^)|(?<n3>\B$))(?<n4>(?<n5>)|(?<n6>[\q{|a|ab}])){3,}(?<n7>(?<n8>$)|(?<n9>\b)){2,3}?"#,
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
            None,
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
fn truth_table_counted_nullable_string_classes_prove_unknown_leaves_before_never_gates() {
    for pattern in [
        r"((\b\B)|(\b\B))(([\q{ab}&&\p{RGI_Emoji}])|()){3}",
        r"((\b\B)|(\b\B))(([\q{|a|ab}])|()){3}[a]",
        r"((^)|(\b)|z)(([\q{|a|ab}])|()){3}",
        r"((^)|(\b)|\1)(([\q{|a|ab}])|()){3}",
        r"((^)|(\b))((([\q{|a|ab}])|()){3}){2}",
        r"((^)|(\b))(([\q{|a|ab}])|()){3}|()",
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
