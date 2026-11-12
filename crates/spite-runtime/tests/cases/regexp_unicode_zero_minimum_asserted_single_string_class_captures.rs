//! Required empty iterations preserve class-versus-empty capture identity.
use crate::common::{assert_true, eval_string};

use crate::common::regexp_captures::assert_no_match;
use crate::common::regexp_matches::DESCRIBE_MATCH;
use spite_core::JsString;
use spite_runtime::{Realm, Value};
use test_case::test_case;
#[test]
fn zero_minimum_asserted_single_string_class_native_tables_consumers_and_required_empty_identity() {
    assert_true(
        r#"
            var pattern =
                "(?<before>^)(?<z>(?<z1>^)|(?<z2>\\b\\B))?(?<whole>(?<iteration>(?<c>[\\q{|a|ab}])))(?<after>$)",
                input = "",
                r = new RegExp(pattern, "dgvym"),
                ok = r.source === pattern && r.global && r.hasIndices && r.unicodeSets === true && r
                .unicode === false;
            r.lastIndex = 0;
            var m = r.exec(input);
            if (m !== null) {
                var names = ["before", "z", "z1", "z2", "whole", "iteration", "c", "after"];
                for (var n of names) {
                    var slot = names.indexOf(n) + 1;
                    ok = ok && Object.hasOwn(m.groups, n) && Object.hasOwn(m.indices.groups, n) && m
                        .groups[n] === m[slot] && m.indices.groups[n] === m.indices[slot]
                }
                ok = ok && Object.getPrototypeOf(m.groups) === null && m.input === input
            } else ok = false;
            ok && m !== null && JSON.stringify([Array.from(m, v => v === undefined ? "<undefined>" : v),
                    Array.from(m.indices, v => v === undefined ? "<undefined>" : v), m.index, r
                    .lastIndex
                ]) ===
                "[[\"\",\"\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"\",\"\",\"\",\"\"],[[0,0],[0,0],\"<undefined>\",\"<undefined>\",\"<undefined>\",[0,0],[0,0],[0,0],[0,0]],0,0]"
        "#,
    );

    assert_true(
        r#"
            var pattern =
                "(?<before>\\b)(?<whole>(?<iteration>(?<c>[\\q{|a|ab}])){0001,0001}?)(?<z>(?<z1>\\B)*|(?<z2>\\b)+)*?(?<after>\\b)",
                input = "\u017faX",
                r = new RegExp(pattern, "dgv"),
                ok = r.source === pattern && r.global && r.hasIndices && r.unicodeSets === true && r
                .unicode === false;
            r.lastIndex = 0;
            var m = r.exec(input);
            if (m !== null) {
                var names = ["before", "whole", "iteration", "c", "z", "z1", "z2", "after"];
                for (var n of names) {
                    var slot = names.indexOf(n) + 1;
                    ok = ok && Object.hasOwn(m.groups, n) && Object.hasOwn(m.indices.groups, n) && m
                        .groups[n] === m[slot] && m.indices.groups[n] === m.indices[slot]
                }
                ok = ok && Object.getPrototypeOf(m.groups) === null && m.input === input
            } else ok = false;
            ok && m !== null && JSON.stringify([Array.from(m, v => v === undefined ? "<undefined>" : v),
                    Array.from(m.indices, v => v === undefined ? "<undefined>" : v), m.index, r
                    .lastIndex
                ]) ===
                "[[\"\",\"\",\"\",\"\",\"\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"\"],[[1,1],[1,1],[1,1],[1,1],[1,1],\"<undefined>\",\"<undefined>\",\"<undefined>\",[1,1]],1,1]"
        "#,
    );
    assert_true(
        r#"
            var pattern =
                "(?<before>\\b)(?<whole>(?<iteration>(?<c>[\\q{a|ab}])))(?<z>(?<z1>\\B)*|(?<z2>\\b)+)*?(?<after>\\b)",
                input = "\n\nab\n",
                r = new RegExp(pattern, "dgvm"),
                ok = r.source === pattern && r.global && r.hasIndices && r.unicodeSets === true && r
                .unicode === false;
            r.lastIndex = 0;
            var m = r.exec(input);
            if (m !== null) {
                var names = ["before", "whole", "iteration", "c", "z", "z1", "z2", "after"];
                for (var n of names) {
                    var slot = names.indexOf(n) + 1;
                    ok = ok && Object.hasOwn(m.groups, n) && Object.hasOwn(m.indices.groups, n) && m
                        .groups[n] === m[slot] && m.indices.groups[n] === m.indices[slot]
                }
                ok = ok && Object.getPrototypeOf(m.groups) === null && m.input === input
            } else ok = false;
            ok && m !== null && JSON.stringify([Array.from(m, v => v === undefined ? "<undefined>" : v),
                    Array.from(m.indices, v => v === undefined ? "<undefined>" : v), m.index, r
                    .lastIndex
                ]) ===
                "[[\"ab\",\"\",\"ab\",\"ab\",\"ab\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"\"],[[2,4],[2,2],[2,4],[2,4],[2,4],\"<undefined>\",\"<undefined>\",\"<undefined>\",[4,4]],2,4]"
        "#,
    );

    assert_true(
        r#"
            var pattern =
                "(?<before>^)(?<z>(?<z1>^)|(?<z2>\\b\\B))?(?<whole>(?<iteration>(?<c>[\\p{Any}\\q{|ab}])){0001,0001}?)(?<after>\\b)",
                input = "a",
                r = new RegExp(pattern, "dgv"),
                ok = r.source === pattern && r.global && r.hasIndices && r.unicodeSets === true && r
                .unicode === false;
            r.lastIndex = 0;
            var m = r.exec(input);
            if (m !== null) {
                var names = ["before", "z", "z1", "z2", "whole", "iteration", "c", "after"];
                for (var n of names) {
                    var slot = names.indexOf(n) + 1;
                    ok = ok && Object.hasOwn(m.groups, n) && Object.hasOwn(m.indices.groups, n) && m
                        .groups[n] === m[slot] && m.indices.groups[n] === m.indices[slot]
                }
                ok = ok && Object.getPrototypeOf(m.groups) === null && m.input === input
            } else ok = false;
            ok && m !== null && JSON.stringify([Array.from(m, v => v === undefined ? "<undefined>" : v),
                    Array.from(m.indices, v => v === undefined ? "<undefined>" : v), m.index, r
                    .lastIndex
                ]) ===
                "[[\"a\",\"\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"a\",\"a\",\"a\",\"\"],[[0,1],[0,0],\"<undefined>\",\"<undefined>\",\"<undefined>\",[0,1],[0,1],[0,1],[1,1]],0,1]"
        "#,
    );

    assert_true(
        r#"
            var pattern =
                "(?<before>^)(?<whole>(?<iteration>(?<c>[\\q{|a|ab}])){1})(?<z>(?<z1>\\B)*|(?<z2>\\b)+)*?(?<after>$)",
                input = "",
                r = new RegExp(pattern, "dgv"),
                ok = r.source === pattern && r.global && r.hasIndices && r.unicodeSets === true && r
                .unicode === false;
            r.lastIndex = 0;
            var m = r.exec(input);
            if (m !== null) {
                var names = ["before", "whole", "iteration", "c", "z", "z1", "z2", "after"];
                for (var n of names) {
                    var slot = names.indexOf(n) + 1;
                    ok = ok && Object.hasOwn(m.groups, n) && Object.hasOwn(m.indices.groups, n) && m
                        .groups[n] === m[slot] && m.indices.groups[n] === m.indices[slot]
                }
                ok = ok && Object.getPrototypeOf(m.groups) === null && m.input === input
            } else ok = false;
            ok && m !== null && JSON.stringify([Array.from(m, v => v === undefined ? "<undefined>" : v),
                    Array.from(m.indices, v => v === undefined ? "<undefined>" : v), m.index, r
                    .lastIndex
                ]) ===
                "[[\"\",\"\",\"\",\"\",\"\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"\"],[[0,0],[0,0],[0,0],[0,0],[0,0],\"<undefined>\",\"<undefined>\",\"<undefined>\",[0,0]],0,0]"
        "#,
    );

    assert_true(
        r#"
            var pattern =
                "(?<before>\\B)(?<z>(?<z1>^)|(?<z2>\\b\\B))?(?<whole>(?<iteration>(?<c>[\\q{|a|ab}])){0001,0001}?)(?<after>\\B)",
                input = "",
                r = new RegExp(pattern, "dgvy"),
                ok = r.source === pattern && r.global && r.hasIndices && r.unicodeSets === true && r
                .unicode === false;
            r.lastIndex = 0;
            var m = r.exec(input);
            if (m !== null) {
                var names = ["before", "z", "z1", "z2", "whole", "iteration", "c", "after"];
                for (var n of names) {
                    var slot = names.indexOf(n) + 1;
                    ok = ok && Object.hasOwn(m.groups, n) && Object.hasOwn(m.indices.groups, n) && m
                        .groups[n] === m[slot] && m.indices.groups[n] === m.indices[slot]
                }
                ok = ok && Object.getPrototypeOf(m.groups) === null && m.input === input
            } else ok = false;
            ok && m !== null && JSON.stringify([Array.from(m, v => v === undefined ? "<undefined>" : v),
                    Array.from(m.indices, v => v === undefined ? "<undefined>" : v), m.index, r
                    .lastIndex
                ]) ===
                "[[\"\",\"\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"\",\"\",\"\",\"\"],[[0,0],[0,0],\"<undefined>\",\"<undefined>\",\"<undefined>\",[0,0],[0,0],[0,0],[0,0]],0,0]"
        "#,
    );

    assert_true(
        r#"
            var pattern =
                "(?<before>\\B)(?<z>(?<z1>^)|(?<z2>\\b\\B))?(?<whole>(?<iteration>(?<c>[\\p{Any}\\q{|ab}])){1})(?<after>\\B)",
                input = "aaaa",
                r = new RegExp(pattern, "dgv"),
                ok = r.source === pattern && r.global && r.hasIndices && r.unicodeSets === true && r
                .unicode === false;
            r.lastIndex = 0;
            var m = r.exec(input);
            if (m !== null) {
                var names = ["before", "z", "z1", "z2", "whole", "iteration", "c", "after"];
                for (var n of names) {
                    var slot = names.indexOf(n) + 1;
                    ok = ok && Object.hasOwn(m.groups, n) && Object.hasOwn(m.indices.groups, n) && m
                        .groups[n] === m[slot] && m.indices.groups[n] === m.indices[slot]
                }
                ok = ok && Object.getPrototypeOf(m.groups) === null && m.input === input
            } else ok = false;
            ok && m !== null && JSON.stringify([Array.from(m, v => v === undefined ? "<undefined>" : v),
                    Array.from(m.indices, v => v === undefined ? "<undefined>" : v), m.index, r
                    .lastIndex
                ]) ===
                "[[\"a\",\"\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"a\",\"a\",\"a\",\"\"],[[1,2],[1,1],\"<undefined>\",\"<undefined>\",\"<undefined>\",[1,2],[1,2],[1,2],[2,2]],1,2]"
        "#,
    );
    assert_true(
        r#"
            var pattern =
                "(?<before>\\B)(?<whole>(?<iteration>(?<c>[a-z\\q{|S|sa|aX}])){1}?)(?<z>(?<z1>\\B)*|(?<z2>\\b)+)*?(?<after>\\B)",
                input = "\ud83d\ude00aX",
                r = new RegExp(pattern, "dgvyi"),
                ok = r.source === pattern && r.global && r.hasIndices && r.unicodeSets === true && r
                .unicode === false;
            r.lastIndex = 1;
            var m = r.exec(input);
            if (m !== null) {
                var names = ["before", "whole", "iteration", "c", "z", "z1", "z2", "after"];
                for (var n of names) {
                    var slot = names.indexOf(n) + 1;
                    ok = ok && Object.hasOwn(m.groups, n) && Object.hasOwn(m.indices.groups, n) && m
                        .groups[n] === m[slot] && m.indices.groups[n] === m.indices[slot]
                }
                ok = ok && Object.getPrototypeOf(m.groups) === null && m.input === input
            } else ok = false;
            ok && m !== null && JSON.stringify([Array.from(m, v => v === undefined ? "<undefined>" : v),
                    Array.from(m.indices, v => v === undefined ? "<undefined>" : v), m.index, r
                    .lastIndex
                ]) ===
                "[[\"\",\"\",\"\",\"\",\"\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"\"],[[0,0],[0,0],[0,0],[0,0],[0,0],\"<undefined>\",\"<undefined>\",\"<undefined>\",[0,0]],0,0]"
        "#,
    );

    assert_true(
        r#"
            var pattern =
                "(?<before>\\b)(?<z>(?<z1>^)|(?<z2>\\b\\B))?(?<whole>(?<iteration>(?<c>[\\q{a|ab}])){1}?)(?<after>\\b)",
                input = "\n\nab\n",
                r = new RegExp(pattern, "dgv"),
                ok = r.source === pattern && r.global && r.hasIndices && r.unicodeSets === true && r
                .unicode === false;
            r.lastIndex = 0;
            var m = r.exec(input);
            if (m !== null) {
                var names = ["before", "z", "z1", "z2", "whole", "iteration", "c", "after"];
                for (var n of names) {
                    var slot = names.indexOf(n) + 1;
                    ok = ok && Object.hasOwn(m.groups, n) && Object.hasOwn(m.indices.groups, n) && m
                        .groups[n] === m[slot] && m.indices.groups[n] === m.indices[slot]
                }
                ok = ok && Object.getPrototypeOf(m.groups) === null && m.input === input
            } else ok = false;
            ok && m !== null && JSON.stringify([Array.from(m, v => v === undefined ? "<undefined>" : v),
                    Array.from(m.indices, v => v === undefined ? "<undefined>" : v), m.index, r
                    .lastIndex
                ]) ===
                "[[\"ab\",\"\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"ab\",\"ab\",\"ab\",\"\"],[[2,4],[2,2],\"<undefined>\",\"<undefined>\",\"<undefined>\",[2,4],[2,4],[2,4],[4,4]],2,4]"
        "#,
    );
    assert_true(
        r#"
            var pattern =
                "(?<before>\\b)(?<z>(?<z1>^)|(?<z2>\\b\\B))?(?<whole>(?<iteration>(?<c>[\\p{Any}\\q{|ab}])){1,1})(?<after>\\b)",
                input = "a",
                r = new RegExp(pattern, "dgv"),
                ok = r.source === pattern && r.global && r.hasIndices && r.unicodeSets === true && r
                .unicode === false;
            r.lastIndex = 0;
            var m = r.exec(input);
            if (m !== null) {
                var names = ["before", "z", "z1", "z2", "whole", "iteration", "c", "after"];
                for (var n of names) {
                    var slot = names.indexOf(n) + 1;
                    ok = ok && Object.hasOwn(m.groups, n) && Object.hasOwn(m.indices.groups, n) && m
                        .groups[n] === m[slot] && m.indices.groups[n] === m.indices[slot]
                }
                ok = ok && Object.getPrototypeOf(m.groups) === null && m.input === input
            } else ok = false;
            ok && m !== null && JSON.stringify([Array.from(m, v => v === undefined ? "<undefined>" : v),
                    Array.from(m.indices, v => v === undefined ? "<undefined>" : v), m.index, r
                    .lastIndex
                ]) ===
                "[[\"a\",\"\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"a\",\"a\",\"a\",\"\"],[[0,1],[0,0],\"<undefined>\",\"<undefined>\",\"<undefined>\",[0,1],[0,1],[0,1],[1,1]],0,1]"
        "#,
    );

    let mut realm = Realm::default();
    realm.eval(r"var r=/(?:(?:\b)|(?:\B))?^(?<whole>(?<iteration>(?<e>){0}(?<c>[\p{Any}\q{|ababa|a|ab}])){1})(?<after>)$/dvg,clone=new RegExp(r),m=r.exec('ababa')").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval(r#"
        var n = clone.exec('');
        m.groups.whole === 'ababa' && m.groups.c === 'ababa' && m.groups.e === undefined && m.indices
            .groups.c === m.indices[4] && m.indices[4][0] === 0 && m.indices[4][1] === 5 && n[0] ===
            '' && n.groups.iteration === '' && n.groups.e === undefined && n.groups.c === '' && n
            .indices.groups.e === undefined && n.indices.groups.c === n.indices[4] && n.indices[4][
            0] === 0 && n.indices[4][1] === 0 && clone.source === r.source && clone.lastIndex === 0 &&
            r.lastIndex === 5
    "#),Ok(Value::Boolean(true)));
}

#[test]
fn match_all_preserves_capture_metadata() {
    let source = format!(
        r#"
            (function() {{
                {DESCRIBE_MATCH}

                var r =
                    /(?:(?:\b)|(?:\B))?(?<before>\b)(?<whole>(?<iteration>(?<e>){{0}}(?<c>[\p{{Any}}\q{{a|ab}}])){{1}})(?<after>\b)/dvg,
                    a = Array.from('ab!aa!'.matchAll(r));
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
        (function() {

            var calls = [],
                r =
                /(?:(?:\b)|(?:\B))?(?<before>\b)(?<whole>(?<iteration>(?<c>[\p{Any}\q{|a|ab}])(?<e>){0}){1})(?<after>\b)/vg,
                out = 'ab!X!'.replace(r, function(all, b, w, it, c, e, a, offset, input, g) {
                    calls.push([all, b, w, it, c === undefined ? '<undefined>' : c, e ===
                        undefined ? '<undefined>' : e, a, offset, input, arguments
                        .length, g.c === c, g.e === e, Object.getPrototypeOf(g) ===
                        null
                    ]);
                    return c === undefined ? '<E>' : '<C:' + c + '>'
                });
            return JSON.stringify([out, calls, r.lastIndex], null, 2);
        })()
    "#;
    let report = eval_string(&mut Realm::default(), source);
    insta::assert_snapshot!(report, @r#"
        [
          "<C:ab><C:!><C:X><C:>!",
          [
            [
              "ab",
              "",
              "ab",
              "ab",
              "ab",
              "<undefined>",
              "",
              0,
              "ab!X!",
              10,
              true,
              true,
              true
            ],
            [
              "!",
              "",
              "!",
              "!",
              "!",
              "<undefined>",
              "",
              2,
              "ab!X!",
              10,
              true,
              true,
              true
            ],
            [
              "X",
              "",
              "X",
              "X",
              "X",
              "<undefined>",
              "",
              3,
              "ab!X!",
              10,
              true,
              true,
              true
            ],
            [
              "",
              "",
              "",
              "",
              "",
              "<undefined>",
              "",
              4,
              "ab!X!",
              10,
              true,
              true,
              true
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

            var r =
                /(?:(?:\b)|(?:\B))?(?<before>\b)(?<whole>(?<iteration>(?<e>){0}(?<c>[\p{Any}\q{a|ab}])){1})(?<after>\b)/dvg;
            r.lastIndex = 9;
            var a = '!ab!X!'.split(r),
                at = '!ab!X!'.search(r);
            return JSON.stringify([a.map(v => v === undefined ? '<undefined>' : v), at, r
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
            "ab",
            "ab",
            "<undefined>",
            "ab",
            "",
            "",
            "",
            "!",
            "!",
            "<undefined>",
            "!",
            "",
            "",
            "",
            "X",
            "X",
            "<undefined>",
            "X",
            "",
            "!"
          ],
          1,
          9
        ]
        "#);
}

#[test]
fn failed_last_index_write_throws_type_error() {
    let source = r#"
        (function() {

            var r = /(?:(?:\b)|(?:\B))?^((){0}([\p{Any}\q{a|ab}])){1}\b/vg;
            Object.defineProperty(r, 'lastIndex', {
                writable: false
            });
            var caught = false;
            try {
                r.exec('x')
            } catch (e) {
                caught = e instanceof TypeError
            }
            return JSON.stringify([caught, r.lastIndex], null, 2);
        })()
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
fn supplementary_capture_names_preserve_utf16_ranges() {
    let source = format!(
        r#"
            (function() {{
                {DESCRIBE_MATCH}

                var r =
                    /(?:(?:\b)|(?:\B))?^(?<\u{{1D49C}}>)(?<whole>(?<iteration>(?<e>){{0}}(?<c>[\p{{Any}}\q{{|😀|😀a|aX}}])){{1}})(?<after>)$/dvgy;
                r.lastIndex = 1;
                var m = r.exec('😀aX');
                return JSON.stringify([describeMatch(m), r.lastIndex, r.source.indexOf('1D49C') !== -
                    1], null, 2);
            }})()
        "#
    );
    let report = eval_string(&mut Realm::default(), &source);
    insta::assert_snapshot!(report, @r#"
        [
          null,
          0,
          true
        ]
        "#);
}

#[test]
fn ignore_case_preserves_capture_metadata() {
    let source = format!(
        r#"
            (function() {{
                {DESCRIBE_MATCH}

                var r =
                    /(?:(?:\b)|(?:\B))?(?<before>\b)(?<whole>(?<iteration>(?<e>){{0}}(?<c>[\p{{Any}}\q{{|S|sa|aX}}])){{1}})(?<after>\b)/dvi,
                    m = r.exec('ſaX!');
                return JSON.stringify([describeMatch(m)], null, 2);
            }})()
        "#
    );
    let report = eval_string(&mut Realm::default(), &source);
    insta::assert_snapshot!(report, @r#"
        [
          {
            "matches": [
              "",
              "",
              "",
              "",
              "<undefined>",
              "",
              ""
            ],
            "indices": [
              [
                0,
                0
              ],
              [
                0,
                0
              ],
              [
                0,
                0
              ],
              [
                0,
                0
              ],
              "<undefined>",
              [
                0,
                0
              ],
              [
                0,
                0
              ]
            ],
            "index": 0,
            "input": "ſaX!",
            "groupNames": [
              "before",
              "whole",
              "iteration",
              "e",
              "c",
              "after"
            ],
            "namedCaptures": [
              "",
              "",
              "",
              "<undefined>",
              "",
              ""
            ],
            "namedIndices": [
              [
                0,
                0
              ],
              [
                0,
                0
              ],
              [
                0,
                0
              ],
              "<undefined>",
              [
                0,
                0
              ],
              [
                0,
                0
              ]
            ],
            "namedAliasesValid": true,
            "groupsPrototypeValid": true
          }
        ]
        "#);
}

#[test]
fn exec_preserves_alternative_capture_metadata() {
    let source = format!(
        r#"
            (function() {{
                {DESCRIBE_MATCH}

                var z =
                    /(?:(?:\b)|(?:\B))?^(?<whole>(?<iteration>(?<c>[\p{{Any}}\q{{|a|ab}}])(?<e>){{0}}){{1}})\b/dv
                    .exec('x'),
                    e =
                    /(?:(?:\b)|(?:\B))?^(?<whole>(?<iteration>(?<e>){{0}}(?<c>[\p{{Any}}\q{{|a|ab}}])){{1}})\b/dv
                    .exec('x'),
                    c =
                    /(?:(?:\b)|(?:\B))?^(?<whole>(?<iteration>(?<c>[\p{{Any}}\q{{|a|ab}}])(?<e>){{0}}){{1}})\b/dv
                    .exec('x');
                return JSON.stringify([describeMatch(z), describeMatch(e), describeMatch(c)], null, 2);
            }})()
        "#
    );
    let report = eval_string(&mut Realm::default(), &source);
    insta::assert_snapshot!(report);
}

#[test]
fn exec_recovers_after_a_failed_match() {
    let source = format!(
        r#"
            (function() {{
                {DESCRIBE_MATCH}

                var e =
                    /(?:(?:\b)|(?:\B))?^(?<whole>(?<iteration>(?<e>){{0}}(?<c>[\q{{ab}}&&\q{{cd}}])){{1}})$/dv
                    .exec(''),
                    c = /(?:(?:\b)|(?:\B))?^(?<whole>(?<c>[\p{{Any}}\q{{}}]){{1}})$/dv.exec('');
                return JSON.stringify([describeMatch(e), describeMatch(c)], null, 2);
            }})()
        "#
    );
    let report = eval_string(&mut Realm::default(), &source);
    insta::assert_snapshot!(report, @r#"
        [
          null,
          {
            "matches": [
              "",
              "",
              ""
            ],
            "indices": [
              [
                0,
                0
              ],
              [
                0,
                0
              ],
              [
                0,
                0
              ]
            ],
            "index": 0,
            "input": "",
            "groupNames": [
              "whole",
              "c"
            ],
            "namedCaptures": [
              "",
              ""
            ],
            "namedIndices": [
              [
                0,
                0
              ],
              [
                0,
                0
              ]
            ],
            "namedAliasesValid": true,
            "groupsPrototypeValid": true
          }
        ]
        "#);
}

#[test]
fn composition_results_match_reference() {
    assert_true(
        r#"
            var pattern = "^(?:(?:\\b)|(?:\\B)){0}((\\b)|(\\B))?([\\q{|a|ab}]){3}$",
                input = "aba",
                r = new RegExp(pattern, "dv"),
                m = r.exec(input);
            r.source === pattern && r.flags === "dv" && r.hasIndices && r.unicodeSets && m !== null && m
                .input === input && m.groups === undefined && m.indices.groups === undefined && Object
                .hasOwn(m, "groups") && Object.hasOwn(m.indices, "groups") && JSON.stringify(m === null ?
                    null : [Array.from(m, v => v === undefined ? "<undefined>" : v), Array.from(m.indices,
                        v => v === undefined ? "<undefined>" : v), m.index, r.lastIndex]) ===
                "[[\"aba\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"\"],[[0,3],\"<undefined>\",\"<undefined>\",\"<undefined>\",[3,3]],0,0]"
        "#,
    );
    assert_true(
        r#"
            var pattern = "^(?:(?:\\b)|(?:\\B))?((\\b)|(\\B)){1}([\\q{|a|ab}]){3}$",
                input = "aba",
                r = new RegExp(pattern, "dv"),
                m = r.exec(input);
            r.source === pattern && r.flags === "dv" && r.hasIndices && r.unicodeSets && (m === null || (m
                    .input === input && m.groups === undefined && m.indices.groups === undefined &&
                    Object.hasOwn(m, "groups") && Object.hasOwn(m.indices, "groups"))) && JSON.stringify(
                    m === null ? null : [Array.from(m, v => v === undefined ? "<undefined>" : v), Array
                        .from(m.indices, v => v === undefined ? "<undefined>" : v), m.index, r.lastIndex
                    ]) ===
                "[[\"aba\",\"\",\"\",\"<undefined>\",\"\"],[[0,3],[0,0],[0,0],\"<undefined>\",[3,3]],0,0]"
        "#,
    );
    assert_true(
        r#"
            var pattern = "^(?:(?:\\b)|(?:\\B))?([\\q{|a|ab}]){3}(\\b|)$",
                input = "aba",
                r = new RegExp(pattern, "dv"),
                m = r.exec(input);
            r.source === pattern && r.flags === "dv" && r.hasIndices && r.unicodeSets && (m === null || (m
                .input === input && m.groups === undefined && m.indices.groups === undefined &&
                Object.hasOwn(m, "groups") && Object.hasOwn(m.indices, "groups"))) && JSON.stringify(
                m === null ? null : [Array.from(m, v => v === undefined ? "<undefined>" : v), Array
                    .from(m.indices, v => v === undefined ? "<undefined>" : v), m.index, r.lastIndex
                ]) === "[[\"aba\",\"\",\"\"],[[0,3],[3,3],[3,3]],0,0]"
        "#,
    );
    for pattern in [
        r"(^([\q{|a|ab}])){3}",
        r"([\q{|a|ab}]){3}\1",
        r"(([\q{|a|ab}]){3}){2}",
        r"([\q{|a|ab}]){3}[\q{ab}]",
        r"(()|([\q{ab}&&\p{RGI_Emoji}])){3}",
    ] {
        let pattern = JsString::from(format!(r"^(?:(?:\b)|(?:\B))?{pattern}$").as_str());
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
    r#"(?<before>^)(?<z>(?<z1>^)|(?<z2>\b\B))?(?<whole>(?<iteration>(?<c>[\q{a|ab}])){1})(?<after>$)"#,
    "dgv",
    JsString::from("aba"),
    0;
    "input_aba_from_0_variant_1"
)]
#[test_case(
    r#"(?<before>^)(?<z>(?<z1>^)|(?<z2>\b\B))?(?<whole>(?<iteration>(?<c>[\p{Any}\q{|ab}])){1}?)(?<after>$)"#,
    "dgv",
    JsString::from("aaaa"),
    0;
    "input_aaaa_from_0_variant_2"
)]
#[test_case(
    r#"(?<before>^)(?<whole>(?<iteration>(?<c>[a-z\q{|S|sa|aX}])){1,1})(?<z>(?<z1>\B)*|(?<z2>\b)+)*?(?<after>$)"#,
    "dgvyi",
    JsString::from("😀aX"),
    1;
    "unicode_input_from_1_variant_3"
)]
#[test_case(
    r#"(?<before>^)(?<z>(?<z1>^)|(?<z2>\b\B))?(?<whole>(?<iteration>(?<c>[\q{|a|ab}])){1}?)(?<after>\b)"#,
    "dgvy",
    JsString::from("ſaX"),
    0;
    "unicode_input_from_0_variant_4"
)]
#[test_case(
    r#"(?<before>^)(?<z>(?<z1>^)|(?<z2>\b\B))?(?<whole>(?<iteration>(?<c>[\q{a|ab}])){1,1})(?<after>\b)"#,
    "dgv",
    JsString::from("\n\nab\n"),
    0;
    "input_ab_from_0_variant_5"
)]
#[test_case(
    r#"(?<before>^)(?<whole>(?<iteration>(?<c>[a-z\q{|S|sa|aX}])))(?<z>(?<z1>\B)*|(?<z2>\b)+)*?(?<after>\b)"#,
    "dgvyim",
    JsString::from("!ab!"),
    0;
    "input_ab_from_0_variant_6"
)]
#[test_case(
    r#"(?<before>^)(?<whole>(?<iteration>(?<c>[\q{a|ab}])){1}?)(?<z>(?<z1>\B)*|(?<z2>\b)+)*?(?<after>$)"#,
    "dgv",
    JsString::from("aba"),
    0;
    "input_aba_from_0_variant_7"
)]
#[test_case(
    r#"(?<before>\B)(?<z>(?<z1>^)|(?<z2>\b\B))?(?<whole>(?<iteration>(?<c>[\q{a|ab}])))(?<after>\B)"#,
    "dgvm",
    JsString::from("aba"),
    0;
    "input_aba_from_0_variant_8"
)]
#[test_case(
    r#"(?<before>^)(?<whole>(?<iteration>(?<c>[\q{|a|ab}])){1,1})(?<z>(?<z1>\B)*|(?<z2>\b)+)*?(?<after>\b)"#,
    "dgv",
    JsString::from("ſaX"),
    0;
    "unicode_input_from_0_variant_9"
)]
#[test_case(
    r#"(?<before>^)(?<whole>(?<iteration>(?<c>[\q{a|ab}])){0001,0001}?)(?<z>(?<z1>\B)*|(?<z2>\b)+)*?(?<after>\b)"#,
    "dgv",
    JsString::from("\n\nab\n"),
    0;
    "input_ab_from_0_variant_10"
)]
#[test_case(
    r#"(?<before>\b)(?<z>(?<z1>^)|(?<z2>\b\B))?(?<whole>(?<iteration>(?<c>[\q{|a|ab}])){1})(?<after>\b)"#,
    "dgvy",
    JsString::from("ſaX"),
    0;
    "unicode_input_from_0_variant_11"
)]
#[test_case(
    r#"(?<before>\b)(?<whole>(?<iteration>(?<c>[a-z\q{|S|sa|aX}])){0001,0001}?)(?<z>(?<z1>\B)*|(?<z2>\b)+)*?(?<after>\b)"#,
    "dgvyi",
    JsString::from("!ab!"),
    0;
    "input_ab_from_0_variant_12"
)]
fn no_match_resets_last_index(
    pattern: &str,
    flags: &str,
    input: JsString,
    initial_last_index: usize,
) {
    assert_no_match(pattern, flags, input, initial_last_index);
}
