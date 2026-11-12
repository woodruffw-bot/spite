//! Two-required counts retain successful starts and final-word captures.
use crate::common::assert_true;
use crate::common::eval_string;
use crate::common::regexp_captures::assert_no_match;
use crate::common::regexp_matches::DESCRIBE_MATCH;
use spite_core::JsString;
use spite_runtime::{Realm, Value};
use test_case::test_case;
#[test]
fn two_required_string_class_native_tables_consumers_and_successful_start_identity() {
    assert_true(
        r#"
            var pattern = "(?<before>)(?<whole>(?<iteration>(?<c>[\\q{aa|aaa}])){2})(?<after>)",
                input = "aaaa",
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
                "[[\"aaaa\",\"\",\"aaaa\",\"aa\",\"aa\",\"\"],[[0,4],[0,0],[0,4],[2,4],[2,4],[4,4]],0,4]"
        "#,
    );
    assert_true(
        r#"
            var pattern = "(?<before>)(?<whole>(?<iteration>(?<c>[\\q{aa|aaa}])){2,3}?)(?<after>)",
                input = "aaaaa",
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
                "[[\"aaaaa\",\"\",\"aaaaa\",\"aa\",\"aa\",\"\"],[[0,5],[0,0],[0,5],[3,5],[3,5],[5,5]],0,5]"
        "#,
    );
    assert_true(
        r#"
            var pattern = "(?<before>)(?<whole>(?<iteration>(?<c>[\\q{a|ab}])){2})(?<after>)",
                input = "aba",
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
                "[[\"aba\",\"\",\"aba\",\"a\",\"a\",\"\"],[[0,3],[0,0],[0,3],[2,3],[2,3],[3,3]],0,3]"
        "#,
    );

    assert_true(
        r#"
            var pattern = "(?<before>)(?<whole>(?<iteration>(?<c>[\\q{ab|abc|c}])){2})(?<after>)",
                input = "abc",
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
                "[[\"abc\",\"\",\"abc\",\"c\",\"c\",\"\"],[[0,3],[0,0],[0,3],[2,3],[2,3],[3,3]],0,3]"
        "#,
    );
    assert_true(
        r#"
            var pattern = "(?<before>)(?<whole>(?<iteration>(?<c>[\\q{ab|abc|c}])){2,3})(?<after>)",
                input = "abcab",
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
                "[[\"abcab\",\"\",\"abcab\",\"ab\",\"ab\",\"\"],[[0,5],[0,0],[0,5],[3,5],[3,5],[5,5]],0,5]"
        "#,
    );
    assert_true(
        r#"
            var pattern = "(?<before>)(?<whole>(?<iteration>(?<c>[\\q{a|aa}])){2,3})(?<after>)",
                input = "aaaaaa",
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
                "[[\"aaaaaa\",\"\",\"aaaaaa\",\"aa\",\"aa\",\"\"],[[0,6],[0,0],[0,6],[4,6],[4,6],[6,6]],0,6]"
        "#,
    );
    assert_true(
        r#"
            var pattern = "(?<before>)(?<whole>(?<iteration>(?<c>[\\q{a|aa}])){2,3}?)(?<after>)",
                input = "aaaaaa",
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
                "[[\"aaaa\",\"\",\"aaaa\",\"aa\",\"aa\",\"\"],[[0,4],[0,0],[0,4],[2,4],[2,4],[4,4]],0,4]"
        "#,
    );
    assert_true(
        r#"
            var pattern = "(?<before>)(?<whole>(?<iteration>(?<c>[\\q{aa|aaa}])){2})(?<after>)",
                input = "!aaaa",
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
                "[[\"aaaa\",\"\",\"aaaa\",\"aa\",\"aa\",\"\"],[[1,5],[1,1],[1,5],[3,5],[3,5],[5,5]],1,5]"
        "#,
    );

    assert_true(
        r#"
            var pattern =
                "(?<before>)(?<whole>(?<iteration>(?<c>[\\q{\ud83d\ude00|\ud83d\ude00a|aX}])){2})(?<after>)",
                input = "\ud83d\ude00aX",
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
                "[[\"\ud83d\ude00aX\",\"\",\"\ud83d\ude00aX\",\"aX\",\"aX\",\"\"],[[0,4],[0,0],[0,4],[2,4],[2,4],[4,4]],0,4]"
        "#,
    );
    assert_true(
        r#"
            var pattern = "(?<before>)(?<whole>(?<iteration>(?<c>[\\q{S|sa|aX}])){2})(?<after>)",
                input = "\u017faX",
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
                "[[\"\u017faX\",\"\",\"\u017faX\",\"aX\",\"aX\",\"\"],[[0,3],[0,0],[0,3],[1,3],[1,3],[3,3]],0,3]"
        "#,
    );
    assert_true(
        r#"
            var pattern = "(?<before>)(?<whole>(?<iteration>(?<c>[\\p{Any}\\q{ab}])){2})(?<after>)",
                input = "ab",
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
            ]) === "[[\"ab\",\"\",\"ab\",\"b\",\"b\",\"\"],[[0,2],[0,0],[0,2],[1,2],[1,2],[2,2]],0,2]"
        "#,
    );
    assert_true(
        r#"
            var pattern = "(?<before>)(?<whole>(?<iteration>(?<c>[\\p{Any}\\q{ab}])){2}?)(?<after>)",
                input = "abc",
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
                "[[\"abc\",\"\",\"abc\",\"c\",\"c\",\"\"],[[0,3],[0,0],[0,3],[2,3],[2,3],[3,3]],0,3]"
        "#,
    );
    assert_true(
        r#"
            var pattern = "(?<before>)(?<whole>(?<iteration>(?<c>[a-z\\q{ab}])){2})(?<after>)",
                input = "ab",
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
            ]) === "[[\"ab\",\"\",\"ab\",\"b\",\"b\",\"\"],[[0,2],[0,0],[0,2],[1,2],[1,2],[2,2]],0,2]"
        "#,
    );
    assert_true(
        r#"
            var pattern = "(?<before>)(?<whole>(?<iteration>(?<c>[\\p{Any}\\q{ab}])){2})(?<after>)",
                input = "\ud83d\ude00ab",
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
                "[[\"\ud83d\ude00ab\",\"\",\"\ud83d\ude00ab\",\"ab\",\"ab\",\"\"],[[0,4],[0,0],[0,4],[2,4],[2,4],[4,4]],0,4]"
        "#,
    );

    assert_true(
        r#"
            var pattern = "(?<before>)(?<whole>(?<c>[\\q{aa|aaa}]{2}))(?<after>)",
                input = "aaaa",
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
            ]) === "[[\"aaaa\",\"\",\"aaaa\",\"aaaa\",\"\"],[[0,4],[0,0],[0,4],[0,4],[4,4]],0,4]"
        "#,
    );
    assert_true(
        r#"
            var pattern = "(?<before>)(?<whole>(?<iteration>(?<c>[\\q{ab|aba}])){2})(?<after>)",
                input = "ababa",
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
                "[[\"ababa\",\"\",\"ababa\",\"aba\",\"aba\",\"\"],[[0,5],[0,0],[0,5],[2,5],[2,5],[5,5]],0,5]"
        "#,
    );
    assert_true(
        r#"
            var pattern = "(?<before>)(?<whole>(?<iteration>(?<c>[\\q{\\(|\\(a|aX}])){2})(?<after>)",
                input = "(aX",
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
                "[[\"(aX\",\"\",\"(aX\",\"aX\",\"aX\",\"\"],[[0,3],[0,0],[0,3],[1,3],[1,3],[3,3]],0,3]"
        "#,
    );

    let mut realm = Realm::default();
    realm.eval(r"var r=/(?<before>)(?<whole>(?<word>[\q{aa|aaa}]){2,3})(?<after>)/dvg,clone=new RegExp(r),m=r.exec('aaaaaa')").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval(r#"
        m.groups.whole === 'aaaaaa' && m.groups.word === 'aaa' && m.indices.groups.word === m.indices[
                3] && m.indices[3][0] === 3 && m.indices[3][1] === 6 && clone.exec('!') === null &&
            clone.source === r.source && clone.lastIndex === 0 && r.lastIndex === 6
    "#),Ok(Value::Boolean(true)));
}

#[test]
fn match_all_preserves_capture_metadata() {
    let source = format!(
        r#"
            (function() {{
                {DESCRIBE_MATCH}

                var r = /(?<whole>(?<word>[\q{{aa|aaa}}]){{2,3}})/dvg,
                    a = Array.from('!aaaaXaaaaaa!'.matchAll(r));
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
                "aaaa",
                "aaaa",
                "aa"
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
              "input": "!aaaaXaaaaaa!",
              "groupNames": [
                "whole",
                "word"
              ],
              "namedCaptures": [
                "aaaa",
                "aa"
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
                "aaaaaa",
                "aaaaaa",
                "aaa"
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
                  9,
                  12
                ]
              ],
              "index": 6,
              "input": "!aaaaXaaaaaa!",
              "groupNames": [
                "whole",
                "word"
              ],
              "namedCaptures": [
                "aaaaaa",
                "aaa"
              ],
              "namedIndices": [
                [
                  6,
                  12
                ],
                [
                  9,
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
                r = /(?<whole>(?<word>[\q{aa|aaa}]){2,3})/vg,
                out = '!aaaaXaaaaaa!'.replace(r, function(all, w, word, offset, input, g) {
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
          "![aa]X[aaa]!",
          [
            [
              "aaaa",
              "aaaa",
              "aa",
              1,
              "!aaaaXaaaaaa!",
              6,
              true,
              true
            ],
            [
              "aaaaaa",
              "aaaaaa",
              "aaa",
              6,
              "!aaaaXaaaaaa!",
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

            var r = /(?<whole>(?<word>[\q{ab|abc|c}]){2})/dvg;
            r.lastIndex = 9;
            var a = '!abc'.split(r),
                at = '!abc'.search(r);
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
            "abc",
            "c",
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

            var r = /[\q{aa|aaa}]{2}/vg;
            Object.defineProperty(r, 'lastIndex', {
                writable: false
            });
            var caught = false;
            try {
                r.exec('xaaaa')
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

                var r = /(?<\u{{1D49C}}>)(?<whole>(?<word>[\q{{😀|😀a|aX}}]){{2}})(?<after>)/dvgy;
                r.lastIndex = 1;
                var m = r.exec('😀aX!');
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
              "😀aX",
              "",
              "😀aX",
              "aX",
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
                2,
                4
              ],
              [
                4,
                4
              ]
            ],
            "index": 0,
            "input": "😀aX!",
            "groupNames": [
              "𝒜",
              "whole",
              "word",
              "after"
            ],
            "namedCaptures": [
              "",
              "😀aX",
              "aX",
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
                2,
                4
              ],
              [
                4,
                4
              ]
            ],
            "namedAliasesValid": true,
            "groupsPrototypeValid": true
          },
          4,
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

                var r = /(?<before>)(?<whole>(?<word>[\q{{S|sa|aX}}]){{2,3}})(?<after>)/dvi,
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
              "ſaX",
              "",
              "ſaX",
              "aX",
              ""
            ],
            "indices": [
              [
                0,
                3
              ],
              [
                0,
                0
              ],
              [
                0,
                3
              ],
              [
                1,
                3
              ],
              [
                3,
                3
              ]
            ],
            "index": 0,
            "input": "ſaX!",
            "groupNames": [
              "before",
              "whole",
              "word",
              "after"
            ],
            "namedCaptures": [
              "",
              "ſaX",
              "aX",
              ""
            ],
            "namedIndices": [
              [
                0,
                0
              ],
              [
                0,
                3
              ],
              [
                1,
                3
              ],
              [
                3,
                3
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

                var g = /(?<whole>(?<word>[\q{{aa|aaa}}]){{2,3}})/dv.exec('aaaaaa!'),
                    l = /(?<whole>(?<word>[\q{{aa|aaa}}]){{2,3}}?)/dv.exec('aaaaaa!');
                return JSON.stringify([describeMatch(g), describeMatch(l)], null, 2);
            }})()
        "#
    );
    let report = eval_string(&mut Realm::default(), &source);
    insta::assert_snapshot!(report, @r#"
        [
          {
            "matches": [
              "aaaaaa",
              "aaaaaa",
              "aaa"
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
                3,
                6
              ]
            ],
            "index": 0,
            "input": "aaaaaa!",
            "groupNames": [
              "whole",
              "word"
            ],
            "namedCaptures": [
              "aaaaaa",
              "aaa"
            ],
            "namedIndices": [
              [
                0,
                6
              ],
              [
                3,
                6
              ]
            ],
            "namedAliasesValid": true,
            "groupsPrototypeValid": true
          },
          {
            "matches": [
              "aaaaaa",
              "aaaaaa",
              "aaa"
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
                3,
                6
              ]
            ],
            "index": 0,
            "input": "aaaaaa!",
            "groupNames": [
              "whole",
              "word"
            ],
            "namedCaptures": [
              "aaaaaa",
              "aaa"
            ],
            "namedIndices": [
              [
                0,
                6
              ],
              [
                3,
                6
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

                var r = /(?<whole>(?<word>[\q{{ab|abc|c}}]){{2}})/dvg,
                    s = /(?<whole>(?<word>[\q{{ab|abc|c}}]){{2}})/dvgy,
                    m = r.exec('!abc'),
                    n = s.exec('!abc');
                s.lastIndex = 1;
                var p = s.exec('!abc');
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
              "abc",
              "abc",
              "c"
            ],
            "indices": [
              [
                1,
                4
              ],
              [
                1,
                4
              ],
              [
                3,
                4
              ]
            ],
            "index": 1,
            "input": "!abc",
            "groupNames": [
              "whole",
              "word"
            ],
            "namedCaptures": [
              "abc",
              "c"
            ],
            "namedIndices": [
              [
                1,
                4
              ],
              [
                3,
                4
              ]
            ],
            "namedAliasesValid": true,
            "groupsPrototypeValid": true
          },
          null,
          {
            "matches": [
              "abc",
              "abc",
              "c"
            ],
            "indices": [
              [
                1,
                4
              ],
              [
                1,
                4
              ],
              [
                3,
                4
              ]
            ],
            "index": 1,
            "input": "!abc",
            "groupNames": [
              "whole",
              "word"
            ],
            "namedCaptures": [
              "abc",
              "c"
            ],
            "namedIndices": [
              [
                1,
                4
              ],
              [
                3,
                4
              ]
            ],
            "namedAliasesValid": true,
            "groupsPrototypeValid": true
          },
          4,
          4
        ]
        "#);
}

#[test]
fn two_required_string_class_proofs_keep_other_minima_predicates_references_and_unknown_leaves_closed()
 {
    assert_true(
        r#"var marker=0,ok=false;try{ok=(function(){var pattern="()([\\q{aa|aaa}]){3}()",rows=[],metadata=true;
for(var multi of [false,true]){var r=new RegExp(pattern,'dvg'+(multi?'m':''));metadata=metadata&&r.source===pattern;
for(var input of ['aaaaaa','ababa','ſaXſ','😀aX😀']){r.lastIndex=0;var m=r.exec(input),result=null;if(m!==null)result={values:Array.from(m,v=>v===undefined?'<undefined>':v),indices:Array.from(m.indices,v=>v===undefined?'<undefined>':v),index:m.index};rows.push([multi,input,result,r.lastIndex]);}}
return metadata&&JSON.stringify(rows)==="[[false,\"aaaaaa\",{\"values\":[\"aaaaaa\",\"\",\"aa\",\"\"],\"indices\":[[0,6],[0,0],[4,6],[6,6]],\"index\":0},6],[false,\"ababa\",null,0],[false,\"\u017faX\u017f\",null,0],[false,\"\ud83d\ude00aX\ud83d\ude00\",null,0],[true,\"aaaaaa\",{\"values\":[\"aaaaaa\",\"\",\"aa\",\"\"],\"indices\":[[0,6],[0,0],[4,6],[6,6]],\"index\":0},6],[true,\"ababa\",null,0],[true,\"\u017faX\u017f\",null,0],[true,\"\ud83d\ude00aX\ud83d\ude00\",null,0]]"})();}finally{marker=3}ok&&marker===3"#,
    );
    assert_true(
        r#"var marker=0,ok=false;try{ok=(function(){var pattern="()([\\p{Any}\\q{ab}]){3}()",rows=[],metadata=true;
for(var multi of [false,true]){var r=new RegExp(pattern,'dvg'+(multi?'m':''));metadata=metadata&&r.source===pattern;
for(var input of ['aaaaaa','ababa','ſaXſ','😀aX😀']){r.lastIndex=0;var m=r.exec(input),result=null;if(m!==null)result={values:Array.from(m,v=>v===undefined?'<undefined>':v),indices:Array.from(m.indices,v=>v===undefined?'<undefined>':v),index:m.index};rows.push([multi,input,result,r.lastIndex]);}}
return metadata&&JSON.stringify(rows)==="[[false,\"aaaaaa\",{\"values\":[\"aaa\",\"\",\"a\",\"\"],\"indices\":[[0,3],[0,0],[2,3],[3,3]],\"index\":0},3],[false,\"ababa\",{\"values\":[\"ababa\",\"\",\"a\",\"\"],\"indices\":[[0,5],[0,0],[4,5],[5,5]],\"index\":0},5],[false,\"\u017faX\u017f\",{\"values\":[\"\u017faX\",\"\",\"X\",\"\"],\"indices\":[[0,3],[0,0],[2,3],[3,3]],\"index\":0},3],[false,\"\ud83d\ude00aX\ud83d\ude00\",{\"values\":[\"\ud83d\ude00aX\",\"\",\"X\",\"\"],\"indices\":[[0,4],[0,0],[3,4],[4,4]],\"index\":0},4],[true,\"aaaaaa\",{\"values\":[\"aaa\",\"\",\"a\",\"\"],\"indices\":[[0,3],[0,0],[2,3],[3,3]],\"index\":0},3],[true,\"ababa\",{\"values\":[\"ababa\",\"\",\"a\",\"\"],\"indices\":[[0,5],[0,0],[4,5],[5,5]],\"index\":0},5],[true,\"\u017faX\u017f\",{\"values\":[\"\u017faX\",\"\",\"X\",\"\"],\"indices\":[[0,3],[0,0],[2,3],[3,3]],\"index\":0},3],[true,\"\ud83d\ude00aX\ud83d\ude00\",{\"values\":[\"\ud83d\ude00aX\",\"\",\"X\",\"\"],\"indices\":[[0,4],[0,0],[3,4],[4,4]],\"index\":0},4]]"})();}finally{marker=3}ok&&marker===3"#,
    );
    assert_true(
        r#"var marker=0,ok=false;try{ok=(function(){var pattern="^()([\\q{aa|aaa}]){2}()",rows=[],metadata=true;
for(var multi of [false,true]){var r=new RegExp(pattern,'dvg'+(multi?'m':''));metadata=metadata&&r.source===pattern;
for(var input of ['aaaaaa','ababa','\nab\n','']){r.lastIndex=0;var m=r.exec(input),result=null;if(m!==null)result={values:Array.from(m,v=>v===undefined?'<undefined>':v),indices:Array.from(m.indices,v=>v===undefined?'<undefined>':v),index:m.index};rows.push([multi,input,result,r.lastIndex]);}}
return metadata&&JSON.stringify(rows)==="[[false,\"aaaaaa\",{\"values\":[\"aaaaaa\",\"\",\"aaa\",\"\"],\"indices\":[[0,6],[0,0],[3,6],[6,6]],\"index\":0},6],[false,\"ababa\",null,0],[false,\"\\nab\\n\",null,0],[false,\"\",null,0],[true,\"aaaaaa\",{\"values\":[\"aaaaaa\",\"\",\"aaa\",\"\"],\"indices\":[[0,6],[0,0],[3,6],[6,6]],\"index\":0},6],[true,\"ababa\",null,0],[true,\"\\nab\\n\",null,0],[true,\"\",null,0]]"})();}finally{marker=3}ok&&marker===3"#,
    );
    assert_true(
        r#"var marker=0,ok=false;try{ok=(function(){var pattern="()([\\q{aa|aaa}]){2}()$",rows=[],metadata=true;
for(var multi of [false,true]){var r=new RegExp(pattern,'dvg'+(multi?'m':''));metadata=metadata&&r.source===pattern;
for(var input of ['aaaaaa','ababa','\nab\n','']){r.lastIndex=0;var m=r.exec(input),result=null;if(m!==null)result={values:Array.from(m,v=>v===undefined?'<undefined>':v),indices:Array.from(m.indices,v=>v===undefined?'<undefined>':v),index:m.index};rows.push([multi,input,result,r.lastIndex]);}}
return metadata&&JSON.stringify(rows)==="[[false,\"aaaaaa\",{\"values\":[\"aaaaaa\",\"\",\"aaa\",\"\"],\"indices\":[[0,6],[0,0],[3,6],[6,6]],\"index\":0},6],[false,\"ababa\",null,0],[false,\"\\nab\\n\",null,0],[false,\"\",null,0],[true,\"aaaaaa\",{\"values\":[\"aaaaaa\",\"\",\"aaa\",\"\"],\"indices\":[[0,6],[0,0],[3,6],[6,6]],\"index\":0},6],[true,\"ababa\",null,0],[true,\"\\nab\\n\",null,0],[true,\"\",null,0]]"})();}finally{marker=3}ok&&marker===3"#,
    );
    for pattern in [
        r"()([[\q{aa|aaa}]&&\p{RGI_Emoji}]){3}()",
        r"()([[\p{Any}\q{ab}]&&\p{RGI_Emoji}]){3}()",
        r"^()([[\q{aa|aaa}]&&\p{RGI_Emoji}]){2}()",
        r"()([[\q{aa|aaa}]&&\p{RGI_Emoji}]){2}()$",
        r"()([\q{aa|aaa}]){2}()\1",
        r"()([\q{aa|aaa}]){2}()a",
        r"a()([\q{aa|aaa}]){2}()",
        r"()(([\q{aa|aaa}]){2}){3}()",
        r"()([\q{ab}&&\p{Any}&&\p{RGI_Emoji}]){2}()",
        r"()([\q{aa|aaa}]){2}()[\q{ef}]",
    ] {
        let mut realm = Realm::default();
        crate::common::regexp_oracle::assert_matches_reference(
            &mut realm,
            &format!(
                "var marker=0;try{{new RegExp({:?},'v').test('aaaa')}}catch{{marker=1}}finally{{marker=2}};marker",
                JsString::from(pattern)
            ),
        );
    }
}

#[test_case(
    r#"(?<before>)(?<whole>(?<iteration>(?<c>[\q{a|ab}])){2})(?<after>)"#,
    "dgv",
    JsString::from("ab"),
    0;
    "input_ab_from_0_variant_1"
)]
#[test_case(
    r#"(?<before>)(?<whole>(?<iteration>(?<c>[\q{aa|aaa}])){2})(?<after>)"#,
    "dgvy",
    JsString::from("!aaaa"),
    0;
    "input_aaaa_from_0_variant_2"
)]
fn no_match_resets_last_index(
    pattern: &str,
    flags: &str,
    input: JsString,
    initial_last_index: usize,
) {
    assert_no_match(pattern, flags, input, initial_last_index);
}
