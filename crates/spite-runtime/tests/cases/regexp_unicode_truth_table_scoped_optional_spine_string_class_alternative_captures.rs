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
                "(?:(?<n1>(?<n2>^)|(?<n3>$)){3}){2,3}?(?<n4>)(?<n5>(?:(?<n6>(?<n7>(?:(?<n8>(?<n9>^)|(?<n10>\\b))(?<n11>[\\p{Any}\\q{|ab}]{0,1}){0001,0001}?(?<n12>(?<n13>\\b^)|(?<n14>$))|(?<n15>(?<n16>^)|(?<n17>\\B))(?<n18>(?<n19>){2,3})(?<n20>(?<n21>$)|(?<n22>\\B))){1,1})){0,1})){1,1}?(?<n23>)(?:(?<n24>(?<n25>(?<n26>^)|(?<n27>\\b))\\B))",
                input = "a\n!",
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
                "(?:(?<n1>(?<n2>\\B^)|(?<n3>\\b$)))(?<n4>)(?<n5>(?<n6>(?<n7>(?<n8>(?<n9>^)|(?<n10>$)){3}(?<n11>[\\q{a|ab}]?)(?<n12>(?<n13>^)|(?<n14>\\B))|(?<n15>(?<n16>$)|(?<n17>\\B))(?<n18>(?<n19>){2,3})(?<n20>(?<n21>\\b\\B)|(?<n22>\\b\\B))))?)(?<n23>)(?:(?<n24>(?<n25>\\b^)|(?<n26>\\B$))){2,3}?",
                input = "ab",
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
                "(?:(?<n1>(?<n2>\\B^)|(?<n3>\\b$))){2,3}?(?<n4>)(?<n5>(?:(?<n6>(?<n7>(?:(?<n8>(?<n9>^)|(?<n10>$)){3}(?<n11>[\\p{Any}\\q{|ab}]{0,1}){0001,0001}?(?<n12>(?<n13>^)|(?<n14>\\B))|(?<n15>(?<n16>$)|(?<n17>\\B))(?<n18>(?<n19>){2,3})(?<n20>(?<n21>\\b\\B)|(?<n22>\\b\\B))){1,1})){0,1})){1,1}?(?<n23>)(?:(?<n24>(?<n25>\\b^)|(?<n26>\\B$)))",
                input = "\ud83d\ude00a",
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
                "(?:(?<n1>(?<n2>\\B^)|(?<n3>\\b$))){0,2}?(?<n4>)(?<n5>(?<n6>(?<n7>(?:(?<n8>(?<n9>(?<n10>(?<n11>(?<n12>$)|(?<n13>\\B))(?<n14>(?<n15>){2,3})(?<n16>(?<n17>\\b\\B)|(?<n18>\\b\\B))|(?<n19>(?<n20>^)|(?<n21>$)){3}(?<n22>(?<n23>[a-z\\q{ab}]{0000,0001}?){1}){1,1}?(?<n24>(?<n25>^)|(?<n26>\\B)))){0001,0001}?)){0000,0001}?))){0001,0001}(?<n27>)(?:(?<n28>(?<n29>\\b^)|(?<n30>\\B$)))?",
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
                "(?:(?<n1>(?<n2>^)|(?<n3>\\B)))(?<n4>)(?<n5>(?<n6>(?<n7>(?<n8>(?<n9>\\b^)|(?<n10>$))(?<n11>[\\q{a|ab}]?)(?<n12>(?<n13>(?<n14>^)|(?<n15>\\b))\\B)|(?<n16>(?<n17>\\b^)|(?<n18>\\B$))(?<n19>(?<n20>){2,3})(?<n21>(?<n22>^)|(?<n23>\\b))))?)(?<n24>)(?:(?<n25>(?<n26>\\b\\B)|(?<n27>\\b\\B))){2,3}?",
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
                "(?:(?<n1>(?<n2>^)|(?<n3>\\B))){2,3}?(?<n4>)(?<n5>(?:(?<n6>(?<n7>(?:(?<n8>(?<n9>\\b^)|(?<n10>$))(?<n11>[\\p{Any}\\q{|ab}]{0,1}){0001,0001}?(?<n12>(?<n13>(?<n14>^)|(?<n15>\\b))\\B)|(?<n16>(?<n17>\\b^)|(?<n18>\\B$))(?<n19>(?<n20>){2,3})(?<n21>(?<n22>^)|(?<n23>\\b))){1,1})){0,1})){1,1}?(?<n24>)(?:(?<n25>(?<n26>\\b\\B)|(?<n27>\\b\\B)))",
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
                "(?:(?<n1>(?<n2>(?<n3>^)|(?<n4>\\b))\\B))(?<n5>)(?<n6>(?<n7>(?<n8>(?<n9>(?<n10>\\B^)|(?<n11>\\b$))(?<n12>[\\q{a|ab}]?)(?<n13>(?<n14>$)|(?<n15>\\B))|(?<n16>(?<n17>\\b\\B)|(?<n18>\\b\\B))(?<n19>(?<n20>){2,3})(?<n21>(?<n22>$)|(?<n23>\\b)){2,3}?))?)(?<n24>)(?:(?<n25>(?<n26>^)|(?<n27>\\b))){2,3}?",
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
                "(?:(?<n1>(?<n2>\\b^)|(?<n3>\\B$)))(?<n4>)(?<n5>(?<n6>(?<n7>(?<n8>(?<n9>(?<n10>^)|(?<n11>\\b))\\B)(?<n12>[\\q{a|ab}]?)(?<n13>(?<n14>\\b\\B)|(?<n15>\\b\\B))|(?<n16>(?<n17>$)|(?<n18>\\b)){2,3}?(?<n19>(?<n20>){2,3})(?<n21>(?<n22>\\b^)|(?<n23>$))))?)(?<n24>)(?:(?<n25>(?<n26>^)|(?<n27>$)){3}){2,3}?",
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
                "(?:(?<n1>(?<n2>\\b^)|(?<n3>\\B$)))?(?<n4>)(?:(?<n5>(?<n6>(?:(?:(?<n7>(?<n8>(?<n9>(?<n10>$)|(?<n11>\\b)){2,3}?(?<n12>(?<n13>){2,3})(?<n14>(?<n15>\\b^)|(?<n16>$))|(?<n17>(?<n18>(?<n19>^)|(?<n20>\\b))\\B)(?<n21>(?<n22>[\\q{|a|ab}]??)){1}(?<n23>(?<n24>\\b\\B)|(?<n25>\\b\\B))))))??))){1}(?<n26>)(?:(?<n27>(?<n28>^)|(?<n29>$)){3}){0,2}?",
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
                "(?:(?<n1>(?<n2>\\b\\B)|(?<n3>\\b\\B)))(?<n4>)(?<n5>(?<n6>(?<n7>(?<n8>(?<n9>$)|(?<n10>\\B))(?<n11>[\\q{a|ab}]?)(?<n12>(?<n13>^)|(?<n14>\\b))|(?<n15>(?<n16>^)|(?<n17>$)){3}(?<n18>(?<n19>){2,3})(?<n20>(?<n21>\\B^)|(?<n22>\\b$))))?)(?<n23>)(?:(?<n24>(?<n25>\\b^)|(?<n26>$))){2,3}?",
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
                "(?:(?<n1>(?<n2>\\b\\B)|(?<n3>\\b\\B))){2,3}?(?<n4>)(?<n5>(?:(?<n6>(?<n7>(?:(?<n8>(?<n9>$)|(?<n10>\\B))(?<n11>[\\p{Any}\\q{|ab}]{0,1}){0001,0001}?(?<n12>(?<n13>^)|(?<n14>\\b))|(?<n15>(?<n16>^)|(?<n17>$)){3}(?<n18>(?<n19>){2,3})(?<n20>(?<n21>\\B^)|(?<n22>\\b$))){1,1})){0,1})){1,1}?(?<n23>)(?:(?<n24>(?<n25>\\b^)|(?<n26>$)))",
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
                "(?:(?<n1>(?<n2>^)|(?<n3>\\b))){2,3}?(?<n4>)(?<n5>(?:(?<n6>(?<n7>(?:(?<n8>(?<n9>\\b^)|(?<n10>\\B$))(?<n11>[\\p{Any}\\q{|ab}]{0,1}){0001,0001}?(?<n12>(?<n13>$)|(?<n14>\\b)){2,3}?|(?<n15>(?<n16>\\b^)|(?<n17>$))(?<n18>(?<n19>){2,3})(?<n20>(?<n21>^)|(?<n22>\\B))){1,1})){0,1})){1,1}?(?<n23>)(?:(?<n24>(?<n25>\\B^)|(?<n26>\\b$)))",
                input = "a\n!",
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
                "(?:(?<n1>(?<n2>$)|(?<n3>\\b)){2,3}?)(?<n4>)(?<n5>(?<n6>(?<n7>(?<n8>(?<n9>\\b\\B)|(?<n10>\\b\\B))(?<n11>[\\q{a|ab}]?)(?<n12>(?<n13>^)|(?<n14>$)){3}|(?<n15>(?<n16>\\B^)|(?<n17>\\b$))(?<n18>(?<n19>){2,3})(?<n20>(?<n21>(?<n22>^)|(?<n23>\\b))\\B)))?)(?<n24>)(?:(?<n25>(?<n26>^)|(?<n27>\\B))){2,3}?",
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
                "(?:(?<n1>(?<n2>$)|(?<n3>\\b)){2,3}?){2,3}?(?<n4>)(?<n5>(?:(?<n6>(?<n7>(?:(?<n8>(?<n9>\\b\\B)|(?<n10>\\b\\B))(?<n11>[\\p{Any}\\q{|ab}]{0,1}){0001,0001}?(?<n12>(?<n13>^)|(?<n14>$)){3}|(?<n15>(?<n16>\\B^)|(?<n17>\\b$))(?<n18>(?<n19>){2,3})(?<n20>(?<n21>(?<n22>^)|(?<n23>\\b))\\B)){1,1})){0,1})){1,1}?(?<n24>)(?:(?<n25>(?<n26>^)|(?<n27>\\B)))",
                input = "\u017fa",
                r = new RegExp(pattern, "dgvi"),
                ok = r.source === pattern && r.global && r.hasIndices && r.unicodeSets === true && r
                .unicode === false;
            r.lastIndex = 1;
            var m = r.exec(input);
            ok && m === null && r.lastIndex === 0
        "#,
    );

    let mut realm = Realm::default();
    realm.eval(r"var r=/(?<ob>(?<os>^)|(?<ow>\b))(?<bp>)(?<enclosing>(?:(?<outer>(?<nested>(?<b>(?<s>^)|(?<w>\b))(?<c>[\q{a|ab}]??)(?<a>(?<end>$)|(?<nw>\B))|(?<fallback>(?<wb>\b)|(?<nwb>\B))(?<empty>)))??))(?<ep>)(?<oa>(?<oe>$)|(?<onw>\B))/dvg,clone=new RegExp(r),m=r.exec('ab!aa')").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval(r#"
        var n = clone.exec('ab');
        r.lastIndex = 0;
        var p = r.exec('');
        m[0] === 'a' && m.groups.enclosing === 'a' && m.groups.outer === 'a' && m.groups.nested ===
            'a' && m.groups.os === '' && m.groups.ow === undefined && m.groups.c === 'a' && m.groups
            .fallback === undefined && m.indices.groups.c === m.indices[11] && m.indices[11][0] ===
            0 && m.indices[11][1] === 1 && n.groups.c === 'ab' && n.groups.outer === 'ab' && n.groups
            .nested === 'ab' && n.groups.enclosing === 'ab' && clone.source === r.source && clone
            .lastIndex === 2 && p[0] === '' && p.groups.enclosing === '' && p.indices.groups
            .enclosing === p.indices[5] && p.indices[5][0] === 0 && p.indices[5][1] === 0 && Array
            .from(p).slice(6, 19).every(v => v === undefined) && p.indices.slice(6, 19).every(v =>
                v === undefined) && p.groups.outer === undefined && p.groups.nested === undefined && p
            .groups.c === undefined && p.groups.os === '' && p.groups.oe === '' && r.lastIndex === 0
    "#),Ok(Value::Boolean(true)));
}

#[test]
fn match_all_preserves_capture_metadata() {
    let source = format!(
        r#"
            (function() {{
                {DESCRIBE_MATCH}

                var P =
                    "(?<ob>(?<os>^)|(?<ow>\\b))(?<bp>)(?<outer>(?<nested>(?<b>(?<s>^)|(?<w>\\b))(?<c>[\\q{{|a|ab}}]??)(?<a>(?<end>$)|(?<nw>\\B))|(?<fallback>(?<wb>\\b)|(?<nwb>\\B))(?<empty>)))??(?<ep>)(?<oa>(?<oe>$)|(?<onw>\\B))";
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
            "(?<ob>(?<os>^)|(?<ow>\\b))(?<bp>)(?<outer>(?<nested>(?<b>(?<s>^)|(?<w>\\b))(?<c>[\\q{|a|ab}]??)(?<a>(?<end>$)|(?<nw>\\B))|(?<fallback>(?<wb>\\b)|(?<nwb>\\B))(?<empty>)))??(?<ep>)(?<oa>(?<oe>$)|(?<onw>\\B))";
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
                "(?<ob>(?<os>^)|(?<ow>\\b))(?<bp>)(?<outer>(?<nested>(?<b>(?<s>^)|(?<w>\\b))(?<c>[\\q{|a|ab}]??)(?<a>(?<end>$)|(?<nw>\\B))|(?<fallback>(?<wb>\\b)|(?<nwb>\\B))(?<empty>)))??(?<ep>)(?<oa>(?<oe>$)|(?<onw>\\B))";
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
            "",
            "ab",
            "ab",
            "",
            "<undefined>",
            "",
            "ab",
            "",
            "<undefined>",
            "",
            "<undefined>",
            "<undefined>",
            "<undefined>",
            "<undefined>",
            "",
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
            "(?<ob>(?<os>^)|(?<ow>\\b))(?<bp>)(?<outer>(?<nested>(?<b>(?<s>^)|(?<w>\\b))(?<c>[\\q{|a|ab}]??)(?<a>(?<end>$)|(?<nw>\\B))|(?<fallback>(?<wb>\\b)|(?<nwb>\\B))(?<empty>)))??(?<ep>)(?<oa>(?<oe>$)|(?<onw>\\B))";
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
                    "(?<ob>(?<os>^)|(?<ow>\\b))(?<bp>)(?<outer>(?<nested>(?<b>(?<s>^)|(?<w>\\b))(?<c>[\\q{{|a|ab}}]??)(?<a>(?<end>$)|(?<nw>\\B))|(?<fallback>(?<wb>\\b)|(?<nwb>\\B))(?<empty>)))??(?<ep>)(?<oa>(?<oe>$)|(?<onw>\\B))";
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
                    "(?<ob>(?<os>^)|(?<ow>\\b))(?<bp>)(?<outer>(?<nested>(?<b>(?<s>^)|(?<w>\\b))(?<c>[a-z\\q{{|S|sa|aX}}]??)(?<a>(?<end>$)|(?<nw>\\B))|(?<fallback>(?<wb>\\b)|(?<nwb>\\B))(?<empty>)))??(?<ep>)(?<oa>(?<oe>$)|(?<onw>\\B))";
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
                    "(?<ob>(?<os>^)|(?<ow>\\b))(?<bp>)(?<outer>(?<nested>(?<b>(?<s>^)|(?<w>\\b))(?<c>[\\q{{|a|ab}}]??)(?<a>(?<end>$)|(?<nw>\\B))|(?<fallback>(?<wb>\\b)|(?<nwb>\\B))(?<empty>)))??(?<ep>)(?<oa>(?<oe>$)|(?<onw>\\B))";
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
                    "(?<ob>(?<os>^)|(?<ow>\\b))(?<bp>)(?<outer>(?<nested>(?<\\u{{1D49C}}>(?<s>^)|(?<w>\\b))(?<c>[\\q{{|a|ab}}]??)(?<a>(?<end>$)|(?<nw>\\B))|(?<fallback>(?<wb>\\b)|(?<nwb>\\B))(?<empty>)))??(?<ep>)(?<oa>(?<oe>$)|(?<onw>\\B))";
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
    r#"(?:(?<n1>(?<n2>^)|(?<n3>$)){3})(?<n4>)(?<n5>(?<n6>(?<n7>(?<n8>(?<n9>^)|(?<n10>\b))(?<n11>[\q{a|ab}]?)(?<n12>(?<n13>\b^)|(?<n14>$))|(?<n15>(?<n16>^)|(?<n17>\B))(?<n18>(?<n19>){2,3})(?<n20>(?<n21>$)|(?<n22>\B))))?)(?<n23>)(?:(?<n24>(?<n25>(?<n26>^)|(?<n27>\b))\B)){2,3}?"#,
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
    r#"(?:(?<n1>(?<n2>^)|(?<n3>$)){3})?(?<n4>)(?:(?<n5>(?<n6>(?:(?:(?<n7>(?<n8>(?<n9>(?<n10>^)|(?<n11>\B))(?<n12>(?<n13>){2,3})(?<n14>(?<n15>$)|(?<n16>\B))|(?<n17>(?<n18>^)|(?<n19>\b))(?<n20>(?<n21>[\q{|a|ab}]??)){1}(?<n22>(?<n23>\b^)|(?<n24>$))))))??))){1}(?<n25>)(?:(?<n26>(?<n27>(?<n28>^)|(?<n29>\b))\B)){0,2}?"#,
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
    r#"(?:(?<n1>(?<n2>^)|(?<n3>$)){3}){0,2}?(?<n4>)(?<n5>(?<n6>(?<n7>(?:(?<n8>(?<n9>(?<n10>(?<n11>(?<n12>^)|(?<n13>\B))(?<n14>(?<n15>){2,3})(?<n16>(?<n17>$)|(?<n18>\B))|(?<n19>(?<n20>^)|(?<n21>\b))(?<n22>(?<n23>[a-z\q{ab}]{0000,0001}?){1}){1,1}?(?<n24>(?<n25>\b^)|(?<n26>$)))){0001,0001}?)){0000,0001}?))){0001,0001}(?<n27>)(?:(?<n28>(?<n29>(?<n30>^)|(?<n31>\b))\B))?"#,
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
    r#"(?:(?<n1>(?<n2>\b^)|(?<n3>$)))(?<n4>)(?<n5>(?<n6>(?<n7>(?<n8>(?<n9>$)|(?<n10>\b)){2,3}?(?<n11>[\q{a|ab}]?)(?<n12>(?<n13>\B^)|(?<n14>\b$))|(?<n15>(?<n16>(?<n17>^)|(?<n18>\b))\B)(?<n19>(?<n20>){2,3})(?<n21>(?<n22>\b^)|(?<n23>\B$))))?)(?<n24>)(?:(?<n25>(?<n26>$)|(?<n27>\B))){2,3}?"#,
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
    r#"(?:(?<n1>(?<n2>\b^)|(?<n3>$)))?(?<n4>)(?:(?<n5>(?<n6>(?:(?:(?<n7>(?<n8>(?<n9>(?<n10>(?<n11>^)|(?<n12>\b))\B)(?<n13>(?<n14>){2,3})(?<n15>(?<n16>\b^)|(?<n17>\B$))|(?<n18>(?<n19>$)|(?<n20>\b)){2,3}?(?<n21>(?<n22>[\q{|a|ab}]??)){1}(?<n23>(?<n24>\B^)|(?<n25>\b$))))))??))){1}(?<n26>)(?:(?<n27>(?<n28>$)|(?<n29>\B))){0,2}?"#,
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
    r#"(?:(?<n1>(?<n2>\b^)|(?<n3>$))){2,3}?(?<n4>)(?<n5>(?:(?<n6>(?<n7>(?:(?<n8>(?<n9>$)|(?<n10>\b)){2,3}?(?<n11>[\p{Any}\q{|ab}]{0,1}){0001,0001}?(?<n12>(?<n13>\B^)|(?<n14>\b$))|(?<n15>(?<n16>(?<n17>^)|(?<n18>\b))\B)(?<n19>(?<n20>){2,3})(?<n21>(?<n22>\b^)|(?<n23>\B$))){1,1})){0,1})){1,1}?(?<n24>)(?:(?<n25>(?<n26>$)|(?<n27>\B)))"#,
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
    r#"(?:(?<n1>(?<n2>\b^)|(?<n3>$))){0,2}?(?<n4>)(?<n5>(?<n6>(?<n7>(?:(?<n8>(?<n9>(?<n10>(?<n11>(?<n12>(?<n13>^)|(?<n14>\b))\B)(?<n15>(?<n16>){2,3})(?<n17>(?<n18>\b^)|(?<n19>\B$))|(?<n20>(?<n21>$)|(?<n22>\b)){2,3}?(?<n23>(?<n24>[a-z\q{ab}]{0000,0001}?){1}){1,1}?(?<n25>(?<n26>\B^)|(?<n27>\b$)))){0001,0001}?)){0000,0001}?))){0001,0001}(?<n28>)(?:(?<n29>(?<n30>$)|(?<n31>\B)))?"#,
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
    r#"(?:(?<n1>(?<n2>\B^)|(?<n3>\b$)))?(?<n4>)(?:(?<n5>(?<n6>(?:(?:(?<n7>(?<n8>(?<n9>(?<n10>$)|(?<n11>\B))(?<n12>(?<n13>){2,3})(?<n14>(?<n15>\b\B)|(?<n16>\b\B))|(?<n17>(?<n18>^)|(?<n19>$)){3}(?<n20>(?<n21>[\q{|a|ab}]??)){1}(?<n22>(?<n23>^)|(?<n24>\B))))))??))){1}(?<n25>)(?:(?<n26>(?<n27>\b^)|(?<n28>\B$))){0,2}?"#,
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
    r#"(?:(?<n1>(?<n2>^)|(?<n3>\B)))?(?<n4>)(?:(?<n5>(?<n6>(?:(?:(?<n7>(?<n8>(?<n9>(?<n10>\b^)|(?<n11>\B$))(?<n12>(?<n13>){2,3})(?<n14>(?<n15>^)|(?<n16>\b))|(?<n17>(?<n18>\b^)|(?<n19>$))(?<n20>(?<n21>[\q{|a|ab}]??)){1}(?<n22>(?<n23>(?<n24>^)|(?<n25>\b))\B)))))??))){1}(?<n26>)(?:(?<n27>(?<n28>\b\B)|(?<n29>\b\B))){0,2}?"#,
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
    r#"(?:(?<n1>(?<n2>^)|(?<n3>\B))){0,2}?(?<n4>)(?<n5>(?<n6>(?<n7>(?:(?<n8>(?<n9>(?<n10>(?<n11>(?<n12>\b^)|(?<n13>\B$))(?<n14>(?<n15>){2,3})(?<n16>(?<n17>^)|(?<n18>\b))|(?<n19>(?<n20>\b^)|(?<n21>$))(?<n22>(?<n23>[a-z\q{ab}]{0000,0001}?){1}){1,1}?(?<n24>(?<n25>(?<n26>^)|(?<n27>\b))\B))){0001,0001}?)){0000,0001}?))){0001,0001}(?<n28>)(?:(?<n29>(?<n30>\b\B)|(?<n31>\b\B)))?"#,
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
    r#"(?:(?<n1>(?<n2>(?<n3>^)|(?<n4>\b))\B))?(?<n5>)(?:(?<n6>(?<n7>(?:(?:(?<n8>(?<n9>(?<n10>(?<n11>\b\B)|(?<n12>\b\B))(?<n13>(?<n14>){2,3})(?<n15>(?<n16>$)|(?<n17>\b)){2,3}?|(?<n18>(?<n19>\B^)|(?<n20>\b$))(?<n21>(?<n22>[\q{|a|ab}]??)){1}(?<n23>(?<n24>$)|(?<n25>\B))))))??))){1}(?<n26>)(?:(?<n27>(?<n28>^)|(?<n29>\b))){0,2}?"#,
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
    r#"(?:(?<n1>(?<n2>(?<n3>^)|(?<n4>\b))\B)){2,3}?(?<n5>)(?<n6>(?:(?<n7>(?<n8>(?:(?<n9>(?<n10>\B^)|(?<n11>\b$))(?<n12>[\p{Any}\q{|ab}]{0,1}){0001,0001}?(?<n13>(?<n14>$)|(?<n15>\B))|(?<n16>(?<n17>\b\B)|(?<n18>\b\B))(?<n19>(?<n20>){2,3})(?<n21>(?<n22>$)|(?<n23>\b)){2,3}?){1,1})){0,1})){1,1}?(?<n24>)(?:(?<n25>(?<n26>^)|(?<n27>\b)))"#,
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
    r#"(?:(?<n1>(?<n2>(?<n3>^)|(?<n4>\b))\B)){0,2}?(?<n5>)(?<n6>(?<n7>(?<n8>(?:(?<n9>(?<n10>(?<n11>(?<n12>(?<n13>\b\B)|(?<n14>\b\B))(?<n15>(?<n16>){2,3})(?<n17>(?<n18>$)|(?<n19>\b)){2,3}?|(?<n20>(?<n21>\B^)|(?<n22>\b$))(?<n23>(?<n24>[a-z\q{ab}]{0000,0001}?){1}){1,1}?(?<n25>(?<n26>$)|(?<n27>\B)))){0001,0001}?)){0000,0001}?))){0001,0001}(?<n28>)(?:(?<n29>(?<n30>^)|(?<n31>\b)))?"#,
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
    r#"(?:(?<n1>(?<n2>$)|(?<n3>\B)))(?<n4>)(?<n5>(?<n6>(?<n7>(?<n8>(?<n9>^)|(?<n10>\B))(?<n11>[\q{a|ab}]?)(?<n12>(?<n13>\b^)|(?<n14>\B$))|(?<n15>(?<n16>^)|(?<n17>\b))(?<n18>(?<n19>){2,3})(?<n20>(?<n21>^)|(?<n22>$)){3}))?)(?<n23>)(?:(?<n24>(?<n25>$)|(?<n26>\b)){2,3}?){2,3}?"#,
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
    r#"(?:(?<n1>(?<n2>$)|(?<n3>\B)))?(?<n4>)(?:(?<n5>(?<n6>(?:(?:(?<n7>(?<n8>(?<n9>(?<n10>^)|(?<n11>\b))(?<n12>(?<n13>){2,3})(?<n14>(?<n15>^)|(?<n16>$)){3}|(?<n17>(?<n18>^)|(?<n19>\B))(?<n20>(?<n21>[\q{|a|ab}]??)){1}(?<n22>(?<n23>\b^)|(?<n24>\B$))))))??))){1}(?<n25>)(?:(?<n26>(?<n27>$)|(?<n28>\b)){2,3}?){0,2}?"#,
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
    r#"(?:(?<n1>(?<n2>$)|(?<n3>\B))){2,3}?(?<n4>)(?<n5>(?:(?<n6>(?<n7>(?:(?<n8>(?<n9>^)|(?<n10>\B))(?<n11>[\p{Any}\q{|ab}]{0,1}){0001,0001}?(?<n12>(?<n13>\b^)|(?<n14>\B$))|(?<n15>(?<n16>^)|(?<n17>\b))(?<n18>(?<n19>){2,3})(?<n20>(?<n21>^)|(?<n22>$)){3}){1,1})){0,1})){1,1}?(?<n23>)(?:(?<n24>(?<n25>$)|(?<n26>\b)){2,3}?)"#,
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
    r#"(?:(?<n1>(?<n2>$)|(?<n3>\B))){0,2}?(?<n4>)(?<n5>(?<n6>(?<n7>(?:(?<n8>(?<n9>(?<n10>(?<n11>(?<n12>^)|(?<n13>\b))(?<n14>(?<n15>){2,3})(?<n16>(?<n17>^)|(?<n18>$)){3}|(?<n19>(?<n20>^)|(?<n21>\B))(?<n22>(?<n23>[a-z\q{ab}]{0000,0001}?){1}){1,1}?(?<n24>(?<n25>\b^)|(?<n26>\B$)))){0001,0001}?)){0000,0001}?))){0001,0001}(?<n27>)(?:(?<n28>(?<n29>$)|(?<n30>\b)){2,3}?)?"#,
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
    r#"(?:(?<n1>(?<n2>\b^)|(?<n3>\B$))){2,3}?(?<n4>)(?<n5>(?:(?<n6>(?<n7>(?:(?<n8>(?<n9>(?<n10>^)|(?<n11>\b))\B)(?<n12>[\p{Any}\q{|ab}]{0,1}){0001,0001}?(?<n13>(?<n14>\b\B)|(?<n15>\b\B))|(?<n16>(?<n17>$)|(?<n18>\b)){2,3}?(?<n19>(?<n20>){2,3})(?<n21>(?<n22>\b^)|(?<n23>$))){1,1})){0,1})){1,1}?(?<n24>)(?:(?<n25>(?<n26>^)|(?<n27>$)){3})"#,
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
    r#"(?:(?<n1>(?<n2>\b^)|(?<n3>\B$))){0,2}?(?<n4>)(?<n5>(?<n6>(?<n7>(?:(?<n8>(?<n9>(?<n10>(?<n11>(?<n12>$)|(?<n13>\b)){2,3}?(?<n14>(?<n15>){2,3})(?<n16>(?<n17>\b^)|(?<n18>$))|(?<n19>(?<n20>(?<n21>^)|(?<n22>\b))\B)(?<n23>(?<n24>[a-z\q{ab}]{0000,0001}?){1}){1,1}?(?<n25>(?<n26>\b\B)|(?<n27>\b\B)))){0001,0001}?)){0000,0001}?))){0001,0001}(?<n28>)(?:(?<n29>(?<n30>^)|(?<n31>$)){3})?"#,
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
    r#"(?:(?<n1>(?<n2>\b\B)|(?<n3>\b\B)))?(?<n4>)(?:(?<n5>(?<n6>(?:(?:(?<n7>(?<n8>(?<n9>(?<n10>^)|(?<n11>$)){3}(?<n12>(?<n13>){2,3})(?<n14>(?<n15>\B^)|(?<n16>\b$))|(?<n17>(?<n18>$)|(?<n19>\B))(?<n20>(?<n21>[\q{|a|ab}]??)){1}(?<n22>(?<n23>^)|(?<n24>\b))))))??))){1}(?<n25>)(?:(?<n26>(?<n27>\b^)|(?<n28>$))){0,2}?"#,
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
    r#"(?:(?<n1>(?<n2>\b\B)|(?<n3>\b\B))){0,2}?(?<n4>)(?<n5>(?<n6>(?<n7>(?:(?<n8>(?<n9>(?<n10>(?<n11>(?<n12>^)|(?<n13>$)){3}(?<n14>(?<n15>){2,3})(?<n16>(?<n17>\B^)|(?<n18>\b$))|(?<n19>(?<n20>$)|(?<n21>\B))(?<n22>(?<n23>[a-z\q{ab}]{0000,0001}?){1}){1,1}?(?<n24>(?<n25>^)|(?<n26>\b)))){0001,0001}?)){0000,0001}?))){0001,0001}(?<n27>)(?:(?<n28>(?<n29>\b^)|(?<n30>$)))?"#,
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
    r#"(?:(?<n1>(?<n2>^)|(?<n3>\b)))(?<n4>)(?<n5>(?<n6>(?<n7>(?<n8>(?<n9>\b^)|(?<n10>\B$))(?<n11>[\q{a|ab}]?)(?<n12>(?<n13>$)|(?<n14>\b)){2,3}?|(?<n15>(?<n16>\b^)|(?<n17>$))(?<n18>(?<n19>){2,3})(?<n20>(?<n21>^)|(?<n22>\B))))?)(?<n23>)(?:(?<n24>(?<n25>\B^)|(?<n26>\b$))){2,3}?"#,
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
    r#"(?:(?<n1>(?<n2>^)|(?<n3>\b)))?(?<n4>)(?:(?<n5>(?<n6>(?:(?:(?<n7>(?<n8>(?<n9>(?<n10>\b^)|(?<n11>$))(?<n12>(?<n13>){2,3})(?<n14>(?<n15>^)|(?<n16>\B))|(?<n17>(?<n18>\b^)|(?<n19>\B$))(?<n20>(?<n21>[\q{|a|ab}]??)){1}(?<n22>(?<n23>$)|(?<n24>\b)){2,3}?))))??))){1}(?<n25>)(?:(?<n26>(?<n27>\B^)|(?<n28>\b$))){0,2}?"#,
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
    r#"(?:(?<n1>(?<n2>^)|(?<n3>\b))){0,2}?(?<n4>)(?<n5>(?<n6>(?<n7>(?:(?<n8>(?<n9>(?<n10>(?<n11>(?<n12>\b^)|(?<n13>$))(?<n14>(?<n15>){2,3})(?<n16>(?<n17>^)|(?<n18>\B))|(?<n19>(?<n20>\b^)|(?<n21>\B$))(?<n22>(?<n23>[a-z\q{ab}]{0000,0001}?){1}){1,1}?(?<n24>(?<n25>$)|(?<n26>\b)){2,3}?)){0001,0001}?)){0000,0001}?))){0001,0001}(?<n27>)(?:(?<n28>(?<n29>\B^)|(?<n30>\b$)))?"#,
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
    r#"(?:(?<n1>(?<n2>$)|(?<n3>\b)){2,3}?)?(?<n4>)(?:(?<n5>(?<n6>(?:(?:(?<n7>(?<n8>(?<n9>(?<n10>\B^)|(?<n11>\b$))(?<n12>(?<n13>){2,3})(?<n14>(?<n15>(?<n16>^)|(?<n17>\b))\B)|(?<n18>(?<n19>\b\B)|(?<n20>\b\B))(?<n21>(?<n22>[\q{|a|ab}]??)){1}(?<n23>(?<n24>^)|(?<n25>$)){3}))))??))){1}(?<n26>)(?:(?<n27>(?<n28>^)|(?<n29>\B))){0,2}?"#,
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
    r#"(?:(?<n1>(?<n2>$)|(?<n3>\b)){2,3}?){0,2}?(?<n4>)(?<n5>(?<n6>(?<n7>(?:(?<n8>(?<n9>(?<n10>(?<n11>(?<n12>\B^)|(?<n13>\b$))(?<n14>(?<n15>){2,3})(?<n16>(?<n17>(?<n18>^)|(?<n19>\b))\B)|(?<n20>(?<n21>\b\B)|(?<n22>\b\B))(?<n23>(?<n24>[a-z\q{ab}]{0000,0001}?){1}){1,1}?(?<n25>(?<n26>^)|(?<n27>$)){3})){0001,0001}?)){0000,0001}?))){0001,0001}(?<n28>)(?:(?<n29>(?<n30>^)|(?<n31>\B)))?"#,
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
fn truth_table_scoped_optional_spine_alternative_string_classes_validate_every_inner_and_outside_term()
 {
    for pattern in [
        r"((\b\B)|(\b\B))()((([\q{ab}&&\p{RGI_Emoji}]??)|()))??()",
        r"((^)|(\b))?()((((^)|(\b))([\q{a|ab}]??)|z))??()",
        r"()((((^)|(\b))([\q{a|ab}]??)|\1))??()",
        r"()((((^)|(\b))([\q{a|ab}]??)|([a])))??()",
        r"()((((^)|(\b))([\q{a|ab}]??)|())){0,2}()",
        r"((^)|z){0}()((((^)|(\b))([\q{a|ab}]??)|()))??()",
        r"((\b\B)|([a]))?()((((^)|(\b))([\q{a|ab}]??)|()))??()",
        r"()((((^)|(\b))([\q{a|ab}]??)|()){0,1})??()",
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
