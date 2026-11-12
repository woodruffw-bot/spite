//! Prefix-free counts retain successful starts and final-word captures.
use crate::common::assert_true;
use crate::common::eval_string;
use crate::common::regexp_captures::assert_no_match;
use crate::common::regexp_matches::DESCRIBE_MATCH;
use spite_core::JsString;
use spite_runtime::{Realm, Value};
use test_case::test_case;
#[test]
fn prefix_free_string_class_native_tables_consumers_and_successful_start_identity() {
    assert_true(
        r#"
            var pattern = "(?<before>)(?<whole>(?<iteration>(?<c>[\\q{ab|cd}])){2})(?<after>)",
                input = "abcd!",
                r = new RegExp(pattern, "dgv"),
                ok = r.source === pattern && r.global && r.hasIndices && r.unicodeSets === true && r
                .unicode === false;
            r.lastIndex = 0;
            var m = r.exec(input);
            if (m !== null) {
                var names = ["before", "whole", "iteration", "c", "after"];
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
                "[[\"abcd\",\"\",\"abcd\",\"cd\",\"cd\",\"\"],[[0,4],[0,0],[0,4],[2,4],[2,4],[4,4]],0,4]"
        "#,
    );
    assert_true(
        r#"
            var pattern = "(?<before>)(?<whole>(?<iteration>(?<c>[\\q{ab|cd}])){2,3})(?<after>)",
                input = "abcdab!",
                r = new RegExp(pattern, "dgv"),
                ok = r.source === pattern && r.global && r.hasIndices && r.unicodeSets === true && r
                .unicode === false;
            r.lastIndex = 0;
            var m = r.exec(input);
            if (m !== null) {
                var names = ["before", "whole", "iteration", "c", "after"];
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
                "[[\"abcdab\",\"\",\"abcdab\",\"ab\",\"ab\",\"\"],[[0,6],[0,0],[0,6],[4,6],[4,6],[6,6]],0,6]"
        "#,
    );
    assert_true(
        r#"
            var pattern = "(?<before>)(?<whole>(?<iteration>(?<c>[\\q{ab|cd}])){2,3}?)(?<after>)",
                input = "abcdab!",
                r = new RegExp(pattern, "dgv"),
                ok = r.source === pattern && r.global && r.hasIndices && r.unicodeSets === true && r
                .unicode === false;
            r.lastIndex = 0;
            var m = r.exec(input);
            if (m !== null) {
                var names = ["before", "whole", "iteration", "c", "after"];
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
                "[[\"abcd\",\"\",\"abcd\",\"cd\",\"cd\",\"\"],[[0,4],[0,0],[0,4],[2,4],[2,4],[4,4]],0,4]"
        "#,
    );
    assert_true(
        r#"
            var pattern = "(?<before>)(?<whole>(?<iteration>(?<c>[\\q{ab|cde}])){2})(?<after>)",
                input = "abcde!",
                r = new RegExp(pattern, "dgv"),
                ok = r.source === pattern && r.global && r.hasIndices && r.unicodeSets === true && r
                .unicode === false;
            r.lastIndex = 0;
            var m = r.exec(input);
            if (m !== null) {
                var names = ["before", "whole", "iteration", "c", "after"];
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
                "[[\"abcde\",\"\",\"abcde\",\"cde\",\"cde\",\"\"],[[0,5],[0,0],[0,5],[2,5],[2,5],[5,5]],0,5]"
        "#,
    );
    assert_true(
        r#"
            var pattern = "(?<before>)(?<whole>(?<iteration>(?<c>[\\q{abcd|bc|da}])){2})(?<after>)",
                input = "abcda",
                r = new RegExp(pattern, "dgv"),
                ok = r.source === pattern && r.global && r.hasIndices && r.unicodeSets === true && r
                .unicode === false;
            r.lastIndex = 0;
            var m = r.exec(input);
            if (m !== null) {
                var names = ["before", "whole", "iteration", "c", "after"];
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
                "[[\"bcda\",\"\",\"bcda\",\"da\",\"da\",\"\"],[[1,5],[1,1],[1,5],[3,5],[3,5],[5,5]],1,5]"
        "#,
    );

    assert_true(
        r#"
            var pattern = "(?<before>)(?<whole>(?<iteration>(?<c>[\\q{ab|cd}])){2})(?<after>)",
                input = "!abcd",
                r = new RegExp(pattern, "dgv"),
                ok = r.source === pattern && r.global && r.hasIndices && r.unicodeSets === true && r
                .unicode === false;
            r.lastIndex = 0;
            var m = r.exec(input);
            if (m !== null) {
                var names = ["before", "whole", "iteration", "c", "after"];
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
                "[[\"abcd\",\"\",\"abcd\",\"cd\",\"cd\",\"\"],[[1,5],[1,1],[1,5],[3,5],[3,5],[5,5]],1,5]"
        "#,
    );

    assert_true(
        r#"
            var pattern =
                "(?<before>)(?<whole>(?<iteration>(?<c>[\\q{\ud83d\ude00a|\ud83d\ude00b}])){2})(?<after>)",
                input = "\ud83d\ude00a\ud83d\ude00b!",
                r = new RegExp(pattern, "dgvy"),
                ok = r.source === pattern && r.global && r.hasIndices && r.unicodeSets === true && r
                .unicode === false;
            r.lastIndex = 1;
            var m = r.exec(input);
            if (m !== null) {
                var names = ["before", "whole", "iteration", "c", "after"];
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
                "[[\"\ud83d\ude00a\ud83d\ude00b\",\"\",\"\ud83d\ude00a\ud83d\ude00b\",\"\ud83d\ude00b\",\"\ud83d\ude00b\",\"\"],[[0,6],[0,0],[0,6],[3,6],[3,6],[6,6]],0,6]"
        "#,
    );
    assert_true(
        r#"
            var pattern = "(?<before>)(?<whole>(?<iteration>(?<c>[\\q{a|bc}])){2,3})(?<after>)",
                input = "abca!",
                r = new RegExp(pattern, "dgv"),
                ok = r.source === pattern && r.global && r.hasIndices && r.unicodeSets === true && r
                .unicode === false;
            r.lastIndex = 0;
            var m = r.exec(input);
            if (m !== null) {
                var names = ["before", "whole", "iteration", "c", "after"];
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
                "[[\"abca\",\"\",\"abca\",\"a\",\"a\",\"\"],[[0,4],[0,0],[0,4],[3,4],[3,4],[4,4]],0,4]"
        "#,
    );
    assert_true(
        r#"
            var pattern = "(?<before>)(?<whole>(?<iteration>(?<c>[\\q{Sx|k}])){2})(?<after>)",
                input = "\u017fx\u212a!",
                r = new RegExp(pattern, "dgvi"),
                ok = r.source === pattern && r.global && r.hasIndices && r.unicodeSets === true && r
                .unicode === false;
            r.lastIndex = 0;
            var m = r.exec(input);
            if (m !== null) {
                var names = ["before", "whole", "iteration", "c", "after"];
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
                "[[\"\u017fx\u212a\",\"\",\"\u017fx\u212a\",\"\u212a\",\"\u212a\",\"\"],[[0,3],[0,0],[0,3],[2,3],[2,3],[3,3]],0,3]"
        "#,
    );

    assert_true(
        r#"
            var pattern = "(?<before>)(?<whole>(?<c>[\\q{ab|cd}]{2}))(?<after>)",
                input = "abcd!",
                r = new RegExp(pattern, "dgv"),
                ok = r.source === pattern && r.global && r.hasIndices && r.unicodeSets === true && r
                .unicode === false;
            r.lastIndex = 0;
            var m = r.exec(input);
            if (m !== null) {
                var names = ["before", "whole", "c", "after"];
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
            ]) === "[[\"abcd\",\"\",\"abcd\",\"abcd\",\"\"],[[0,4],[0,0],[0,4],[0,4],[4,4]],0,4]"
        "#,
    );
    assert_true(
        r#"
            var pattern = "(?<before>)(?<whole>(?<iteration>(?<c>[\\q{\\n|ab}])){2})(?<after>)",
                input = "\nab!",
                r = new RegExp(pattern, "dgv"),
                ok = r.source === pattern && r.global && r.hasIndices && r.unicodeSets === true && r
                .unicode === false;
            r.lastIndex = 0;
            var m = r.exec(input);
            if (m !== null) {
                var names = ["before", "whole", "iteration", "c", "after"];
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
                "[[\"\\nab\",\"\",\"\\nab\",\"ab\",\"ab\",\"\"],[[0,3],[0,0],[0,3],[1,3],[1,3],[3,3]],0,3]"
        "#,
    );

    let mut realm = Realm::default();
    realm.eval(r"var r=/(?<before>)(?<whole>(?<word>[\q{ab|cd}]){2,3})(?<after>)/dvg,clone=new RegExp(r),m=r.exec('abcdab')").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval(r#"
        m.groups.whole === 'abcdab' && m.groups.word === 'ab' && m.indices.groups.word === m.indices[
            3] && m.indices[3][0] === 4 && m.indices[3][1] === 6 && clone.exec('!') === null && clone
            .source === r.source && clone.lastIndex === 0 && r.lastIndex === 6
    "#),Ok(Value::Boolean(true)));
}

#[test]
fn match_all_preserves_capture_metadata() {
    let source = format!(
        r#"
            (function() {{
                {DESCRIBE_MATCH}

                var r = /(?<whole>(?<word>[\q{{ab|cd}}]){{2,3}})/dvg,
                    a = Array.from('!abcdXcdabab!'.matchAll(r));
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
                "abcd",
                "abcd",
                "cd"
              ],
              "indices": [
                [
                  1,
                  5
                ],
                [
                  1,
                  5
                ],
                [
                  3,
                  5
                ]
              ],
              "index": 1,
              "input": "!abcdXcdabab!",
              "groupNames": [
                "whole",
                "word"
              ],
              "namedCaptures": [
                "abcd",
                "cd"
              ],
              "namedIndices": [
                [
                  1,
                  5
                ],
                [
                  3,
                  5
                ]
              ],
              "namedAliasesValid": true,
              "groupsPrototypeValid": true
            },
            {
              "matches": [
                "cdabab",
                "cdabab",
                "ab"
              ],
              "indices": [
                [
                  6,
                  12
                ],
                [
                  6,
                  12
                ],
                [
                  10,
                  12
                ]
              ],
              "index": 6,
              "input": "!abcdXcdabab!",
              "groupNames": [
                "whole",
                "word"
              ],
              "namedCaptures": [
                "cdabab",
                "ab"
              ],
              "namedIndices": [
                [
                  6,
                  12
                ],
                [
                  10,
                  12
                ]
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
        (function() {

            var calls = [],
                r = /(?<whole>(?<word>[\q{ab|cd}]){2,3})/vg,
                out = '!abcdXcdabab!'.replace(r, function(all, w, word, offset, input, g) {
                    calls.push([all, w, word, offset, input, arguments.length, g.word === word,
                        Object.getPrototypeOf(g) === null
                    ]);
                    return '[' + word + ']'
                });
            return JSON.stringify([out, calls, r.lastIndex], null, 2);
        })()
    "#;
    let report = eval_string(&mut Realm::default(), source);
    insta::assert_snapshot!(report, @r#"
        [
          "![cd]X[ab]!",
          [
            [
              "abcd",
              "abcd",
              "cd",
              1,
              "!abcdXcdabab!",
              6,
              true,
              true
            ],
            [
              "cdabab",
              "cdabab",
              "ab",
              6,
              "!abcdXcdabab!",
              6,
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

            var r = /(?<whole>(?<word>[\q{abcd|bc|da}]){2})/dvg;
            r.lastIndex = 9;
            var a = 'abcda'.split(r),
                at = 'abcda'.search(r);
            return JSON.stringify([a.map(v => v === undefined ? '<undefined>' : v), at, r
                .lastIndex
            ], null, 2);
        })()
    "#;
    let report = eval_string(&mut Realm::default(), source);
    insta::assert_snapshot!(report, @r#"
        [
          [
            "a",
            "bcda",
            "da",
            ""
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

            var r = /[\q{ab|cd}]{2}/vg;
            Object.defineProperty(r, 'lastIndex', {
                writable: false
            });
            var caught = false;
            try {
                r.exec('xabcd')
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

                var r = /(?<\u{{1D49C}}>)(?<whole>(?<word>[\q{{😀a|😀b}}]){{2}})(?<after>)/dvgy;
                r.lastIndex = 1;
                var m = r.exec('😀a😀b!');
                return JSON.stringify([describeMatch(m), r.lastIndex, r.source.indexOf('1D49C') !== -
                    1], null, 2);
            }})()
        "#
    );
    let report = eval_string(&mut Realm::default(), &source);
    insta::assert_snapshot!(report, @r#"
        [
          {
            "matches": [
              "😀a😀b",
              "",
              "😀a😀b",
              "😀b",
              ""
            ],
            "indices": [
              [
                0,
                6
              ],
              [
                0,
                0
              ],
              [
                0,
                6
              ],
              [
                3,
                6
              ],
              [
                6,
                6
              ]
            ],
            "index": 0,
            "input": "😀a😀b!",
            "groupNames": [
              "𝒜",
              "whole",
              "word",
              "after"
            ],
            "namedCaptures": [
              "",
              "😀a😀b",
              "😀b",
              ""
            ],
            "namedIndices": [
              [
                0,
                0
              ],
              [
                0,
                6
              ],
              [
                3,
                6
              ],
              [
                6,
                6
              ]
            ],
            "namedAliasesValid": true,
            "groupsPrototypeValid": true
          },
          6,
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

                var r = /(?<before>)(?<whole>(?<word>[\q{{Sx|k}}]){{2,3}})(?<after>)/dvi,
                    m = r.exec('ſxKk!');
                return JSON.stringify([describeMatch(m)], null, 2);
            }})()
        "#
    );
    let report = eval_string(&mut Realm::default(), &source);
    insta::assert_snapshot!(report, @r#"
        [
          {
            "matches": [
              "ſxKk",
              "",
              "ſxKk",
              "k",
              ""
            ],
            "indices": [
              [
                0,
                4
              ],
              [
                0,
                0
              ],
              [
                0,
                4
              ],
              [
                3,
                4
              ],
              [
                4,
                4
              ]
            ],
            "index": 0,
            "input": "ſxKk!",
            "groupNames": [
              "before",
              "whole",
              "word",
              "after"
            ],
            "namedCaptures": [
              "",
              "ſxKk",
              "k",
              ""
            ],
            "namedIndices": [
              [
                0,
                0
              ],
              [
                0,
                4
              ],
              [
                3,
                4
              ],
              [
                4,
                4
              ]
            ],
            "namedAliasesValid": true,
            "groupsPrototypeValid": true
          }
        ]
        "#);
}

#[test]
fn greedy_and_lazy_quantifiers_preserve_captures() {
    let source = format!(
        r#"
            (function() {{
                {DESCRIBE_MATCH}

                var g = /(?<whole>(?<word>[\q{{ab|cd}}]){{2,3}})/dv.exec('abcdab!'),
                    l = /(?<whole>(?<word>[\q{{ab|cd}}]){{2,3}}?)/dv.exec('abcdab!');
                return JSON.stringify([describeMatch(g), describeMatch(l)], null, 2);
            }})()
        "#
    );
    let report = eval_string(&mut Realm::default(), &source);
    insta::assert_snapshot!(report, @r#"
        [
          {
            "matches": [
              "abcdab",
              "abcdab",
              "ab"
            ],
            "indices": [
              [
                0,
                6
              ],
              [
                0,
                6
              ],
              [
                4,
                6
              ]
            ],
            "index": 0,
            "input": "abcdab!",
            "groupNames": [
              "whole",
              "word"
            ],
            "namedCaptures": [
              "abcdab",
              "ab"
            ],
            "namedIndices": [
              [
                0,
                6
              ],
              [
                4,
                6
              ]
            ],
            "namedAliasesValid": true,
            "groupsPrototypeValid": true
          },
          {
            "matches": [
              "abcd",
              "abcd",
              "cd"
            ],
            "indices": [
              [
                0,
                4
              ],
              [
                0,
                4
              ],
              [
                2,
                4
              ]
            ],
            "index": 0,
            "input": "abcdab!",
            "groupNames": [
              "whole",
              "word"
            ],
            "namedCaptures": [
              "abcd",
              "cd"
            ],
            "namedIndices": [
              [
                0,
                4
              ],
              [
                2,
                4
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

                var r = /(?<whole>(?<word>[\q{{abcd|bc|da}}]){{2}})/dvg,
                    s = /(?<whole>(?<word>[\q{{abcd|bc|da}}]){{2}})/dvgy,
                    m = r.exec('abcda'),
                    n = s.exec('abcda');
                s.lastIndex = 1;
                var p = s.exec('abcda');
                return JSON.stringify([describeMatch(m), describeMatch(n), describeMatch(p), r
                    .lastIndex, s.lastIndex
                ], null, 2);
            }})()
        "#
    );
    let report = eval_string(&mut Realm::default(), &source);
    insta::assert_snapshot!(report, @r#"
        [
          {
            "matches": [
              "bcda",
              "bcda",
              "da"
            ],
            "indices": [
              [
                1,
                5
              ],
              [
                1,
                5
              ],
              [
                3,
                5
              ]
            ],
            "index": 1,
            "input": "abcda",
            "groupNames": [
              "whole",
              "word"
            ],
            "namedCaptures": [
              "bcda",
              "da"
            ],
            "namedIndices": [
              [
                1,
                5
              ],
              [
                3,
                5
              ]
            ],
            "namedAliasesValid": true,
            "groupsPrototypeValid": true
          },
          null,
          {
            "matches": [
              "bcda",
              "bcda",
              "da"
            ],
            "indices": [
              [
                1,
                5
              ],
              [
                1,
                5
              ],
              [
                3,
                5
              ]
            ],
            "index": 1,
            "input": "abcda",
            "groupNames": [
              "whole",
              "word"
            ],
            "namedCaptures": [
              "bcda",
              "da"
            ],
            "namedIndices": [
              [
                1,
                5
              ],
              [
                3,
                5
              ]
            ],
            "namedAliasesValid": true,
            "groupsPrototypeValid": true
          },
          5,
          5
        ]
        "#);
}

#[test]
fn prefix_free_string_class_proofs_keep_word_backtracking_predicates_references_and_unknown_leaves_closed()
 {
    assert_true(
        r#"var marker=0,ok=false;try{ok=(function(){var pattern="()([\\q{a|ab}]){2}()",rows=[],metadata=true;
for(var multi of [false,true]){var r=new RegExp(pattern,'dvg'+(multi?'m':''));metadata=metadata&&r.source===pattern;
for(var input of ['aaaa','ab','ſaX','😀aX']){r.lastIndex=0;var m=r.exec(input),result=null;if(m!==null)result={values:Array.from(m,v=>v===undefined?'<undefined>':v),indices:Array.from(m.indices,v=>v===undefined?'<undefined>':v),index:m.index};rows.push([multi,input,result,r.lastIndex]);}}
return metadata&&JSON.stringify(rows)==="[[false,\"aaaa\",{\"values\":[\"aa\",\"\",\"a\",\"\"],\"indices\":[[0,2],[0,0],[1,2],[2,2]],\"index\":0},2],[false,\"ab\",null,0],[false,\"\u017faX\",null,0],[false,\"\ud83d\ude00aX\",null,0],[true,\"aaaa\",{\"values\":[\"aa\",\"\",\"a\",\"\"],\"indices\":[[0,2],[0,0],[1,2],[2,2]],\"index\":0},2],[true,\"ab\",null,0],[true,\"\u017faX\",null,0],[true,\"\ud83d\ude00aX\",null,0]]"})();}finally{marker=3}ok&&marker===3"#,
    );
    assert_true(
        r#"var marker=0,ok=false;try{ok=(function(){var pattern="()([\\q{aa|aaa}]){2}()",rows=[],metadata=true;
for(var multi of [false,true]){var r=new RegExp(pattern,'dvg'+(multi?'m':''));metadata=metadata&&r.source===pattern;
for(var input of ['aaaa','ab','ſaX','😀aX']){r.lastIndex=0;var m=r.exec(input),result=null;if(m!==null)result={values:Array.from(m,v=>v===undefined?'<undefined>':v),indices:Array.from(m.indices,v=>v===undefined?'<undefined>':v),index:m.index};rows.push([multi,input,result,r.lastIndex]);}}
return metadata&&JSON.stringify(rows)==="[[false,\"aaaa\",{\"values\":[\"aaaa\",\"\",\"aa\",\"\"],\"indices\":[[0,4],[0,0],[2,4],[4,4]],\"index\":0},4],[false,\"ab\",null,0],[false,\"\u017faX\",null,0],[false,\"\ud83d\ude00aX\",null,0],[true,\"aaaa\",{\"values\":[\"aaaa\",\"\",\"aa\",\"\"],\"indices\":[[0,4],[0,0],[2,4],[4,4]],\"index\":0},4],[true,\"ab\",null,0],[true,\"\u017faX\",null,0],[true,\"\ud83d\ude00aX\",null,0]]"})();}finally{marker=3}ok&&marker===3"#,
    );
    assert_true(
        r#"var marker=0,ok=false;try{ok=(function(){var pattern="([\\p{Any}\\q{ab}]){2}",rows=[],metadata=true;
for(var multi of [false,true]){var r=new RegExp(pattern,'dvg'+(multi?'m':''));metadata=metadata&&r.source===pattern;
for(var input of ['aaaa','ab','ſaX','😀aX']){r.lastIndex=0;var m=r.exec(input),result=null;if(m!==null)result={values:Array.from(m,v=>v===undefined?'<undefined>':v),indices:Array.from(m.indices,v=>v===undefined?'<undefined>':v),index:m.index};rows.push([multi,input,result,r.lastIndex]);}}
return metadata&&JSON.stringify(rows)==="[[false,\"aaaa\",{\"values\":[\"aa\",\"a\"],\"indices\":[[0,2],[1,2]],\"index\":0},2],[false,\"ab\",{\"values\":[\"ab\",\"b\"],\"indices\":[[0,2],[1,2]],\"index\":0},2],[false,\"\u017faX\",{\"values\":[\"\u017fa\",\"a\"],\"indices\":[[0,2],[1,2]],\"index\":0},2],[false,\"\ud83d\ude00aX\",{\"values\":[\"\ud83d\ude00a\",\"a\"],\"indices\":[[0,3],[2,3]],\"index\":0},3],[true,\"aaaa\",{\"values\":[\"aa\",\"a\"],\"indices\":[[0,2],[1,2]],\"index\":0},2],[true,\"ab\",{\"values\":[\"ab\",\"b\"],\"indices\":[[0,2],[1,2]],\"index\":0},2],[true,\"\u017faX\",{\"values\":[\"\u017fa\",\"a\"],\"indices\":[[0,2],[1,2]],\"index\":0},2],[true,\"\ud83d\ude00aX\",{\"values\":[\"\ud83d\ude00a\",\"a\"],\"indices\":[[0,3],[2,3]],\"index\":0},3]]"})();}finally{marker=3}ok&&marker===3"#,
    );
    assert_true(
        r#"var marker=0,ok=false;try{ok=(function(){var pattern="^()([\\q{ab|cd}]){2}()",rows=[],metadata=true;
for(var multi of [false,true]){var r=new RegExp(pattern,'dvg'+(multi?'m':''));metadata=metadata&&r.source===pattern;
for(var input of ['aaaaaa','ababa','\nab\n','']){r.lastIndex=0;var m=r.exec(input),result=null;if(m!==null)result={values:Array.from(m,v=>v===undefined?'<undefined>':v),indices:Array.from(m.indices,v=>v===undefined?'<undefined>':v),index:m.index};rows.push([multi,input,result,r.lastIndex]);}}
return metadata&&JSON.stringify(rows)==="[[false,\"aaaaaa\",null,0],[false,\"ababa\",{\"values\":[\"abab\",\"\",\"ab\",\"\"],\"indices\":[[0,4],[0,0],[2,4],[4,4]],\"index\":0},4],[false,\"\\nab\\n\",null,0],[false,\"\",null,0],[true,\"aaaaaa\",null,0],[true,\"ababa\",{\"values\":[\"abab\",\"\",\"ab\",\"\"],\"indices\":[[0,4],[0,0],[2,4],[4,4]],\"index\":0},4],[true,\"\\nab\\n\",null,0],[true,\"\",null,0]]"})();}finally{marker=3}ok&&marker===3"#,
    );
    assert_true(
        r#"var marker=0,ok=false;try{ok=(function(){var pattern="()([\\q{ab|cd}]){2}()$",rows=[],metadata=true;
for(var multi of [false,true]){var r=new RegExp(pattern,'dvg'+(multi?'m':''));metadata=metadata&&r.source===pattern;
for(var input of ['aaaaaa','ababa','\nab\n','']){r.lastIndex=0;var m=r.exec(input),result=null;if(m!==null)result={values:Array.from(m,v=>v===undefined?'<undefined>':v),indices:Array.from(m.indices,v=>v===undefined?'<undefined>':v),index:m.index};rows.push([multi,input,result,r.lastIndex]);}}
return metadata&&JSON.stringify(rows)==="[[false,\"aaaaaa\",null,0],[false,\"ababa\",null,0],[false,\"\\nab\\n\",null,0],[false,\"\",null,0],[true,\"aaaaaa\",null,0],[true,\"ababa\",null,0],[true,\"\\nab\\n\",null,0],[true,\"\",null,0]]"})();}finally{marker=3}ok&&marker===3"#,
    );
    for pattern in [
        r"()([[\q{a|ab}]&&\p{RGI_Emoji}]){2}()",
        r"()([[\q{aa|aaa}]&&\p{RGI_Emoji}]){2}()",
        r"([[\p{Any}\q{ab}]&&\p{RGI_Emoji}]){2}",
        r"^()([[\q{ab|cd}]&&\p{RGI_Emoji}]){2}()",
        r"()([[\q{ab|cd}]&&\p{RGI_Emoji}]){2}()$",
        r"()([\q{ab|cd}]){2}()\1",
        r"()([\q{ab|cd}]){2}()a",
        r"a()([\q{ab|cd}]){2}()",
        r"()(([\q{ab|cd}]){2}){3}()",
        r"()([\q{ab|cd}]){2}()[\q{ef}]",
    ] {
        let mut realm = Realm::default();
        crate::common::regexp_oracle::assert_matches_reference(
            &mut realm,
            &format!(
                "var marker=0;try{{new RegExp({:?},'v').test('abcd')}}catch{{marker=1}}finally{{marker=2}};marker",
                JsString::from(pattern)
            ),
        );
    }
    assert_true(
        r#"var marker=0,ok=false;try{ok=(function(){var pattern="([\\q{A|ab}]){2}",rows=[],metadata=true;for(var multi of [false,true]){var r=new RegExp(pattern,'dvgi'+(multi?'m':''));metadata=metadata&&r.source===pattern;for(var input of ['aab','Aab','ab','']){r.lastIndex=0;var m=r.exec(input),result=null;if(m!==null)result={values:Array.from(m,v=>v===undefined?'<undefined>':v),indices:Array.from(m.indices,v=>v===undefined?'<undefined>':v),index:m.index};rows.push([multi,input,result,r.lastIndex]);}}return metadata&&JSON.stringify(rows)==="[[false,\"aab\",{\"values\":[\"aab\",\"ab\"],\"indices\":[[0,3],[1,3]],\"index\":0},3],[false,\"Aab\",{\"values\":[\"Aab\",\"ab\"],\"indices\":[[0,3],[1,3]],\"index\":0},3],[false,\"ab\",null,0],[false,\"\",null,0],[true,\"aab\",{\"values\":[\"aab\",\"ab\"],\"indices\":[[0,3],[1,3]],\"index\":0},3],[true,\"Aab\",{\"values\":[\"Aab\",\"ab\"],\"indices\":[[0,3],[1,3]],\"index\":0},3],[true,\"ab\",null,0],[true,\"\",null,0]]"})();}finally{marker=3}ok&&marker===3"#,
    );
}

#[test_case(
    r#"(?<before>)(?<whole>(?<iteration>(?<c>[\q{abcd|bc|da}])){2})(?<after>)"#,
    "dgvy",
    JsString::from("abcda"),
    0;
    "input_abcda_from_0_variant_1"
)]
#[test_case(
    r#"(?<before>)(?<whole>(?<iteration>(?<c>[\q{ab|cd}])){2})(?<after>)"#,
    "dgvy",
    JsString::from("!abcd"),
    0;
    "input_abcd_from_0_variant_2"
)]
#[test_case(
    r#"(?<before>)(?<whole>(?<iteration>(?<c>[\q{ab|cd}])){2})(?<after>)"#,
    "dgv",
    JsString::from("ab"),
    0;
    "input_ab_from_0_variant_3"
)]
fn no_match_resets_last_index(
    pattern: &str,
    flags: &str,
    input: JsString,
    initial_last_index: usize,
) {
    assert_no_match(pattern, flags, input, initial_last_index);
}
