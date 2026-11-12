//! Required counts retain successful starts and final-word captures.
use crate::common::assert_true;
use crate::common::eval_string;
use crate::common::regexp_captures::assert_no_match;
use crate::common::regexp_matches::DESCRIBE_MATCH;
use spite_core::JsString;
use spite_runtime::{Realm, Value};
use test_case::test_case;
#[test]
fn required_string_class_native_tables_consumers_and_successful_start_identity() {
    assert_true(
        r#"
            var pattern = "(?<before>)(?<whole>(?<iteration>(?<c>[\\q{aa|aaa}])){3})(?<after>)",
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
            var pattern = "(?<before>)(?<whole>(?<iteration>(?<c>[\\q{aa|aaa}])){3,4}?)(?<after>)",
                input = "aaaaaaaa",
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
                "[[\"aaaaaaaa\",\"\",\"aaaaaaaa\",\"aa\",\"aa\",\"\"],[[0,8],[0,0],[0,8],[6,8],[6,8],[8,8]],0,8]"
        "#,
    );
    assert_true(
        r#"
            var pattern = "(?<before>)(?<whole>(?<iteration>(?<c>[\\q{a|ab}])){3})(?<after>)",
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
                "[[\"ababa\",\"\",\"ababa\",\"a\",\"a\",\"\"],[[0,5],[0,0],[0,5],[4,5],[4,5],[5,5]],0,5]"
        "#,
    );

    assert_true(
        r#"
            var pattern = "(?<before>)(?<whole>(?<iteration>(?<c>[\\q{ab|abc|c}])){3})(?<after>)",
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
            var pattern = "(?<before>)(?<whole>(?<iteration>(?<c>[\\q{ab|abc|c}])){3,4})(?<after>)",
                input = "abcabcab",
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
                "[[\"abcabcab\",\"\",\"abcabcab\",\"ab\",\"ab\",\"\"],[[0,8],[0,0],[0,8],[6,8],[6,8],[8,8]],0,8]"
        "#,
    );
    assert_true(
        r#"
            var pattern = "(?<before>)(?<whole>(?<iteration>(?<c>[\\q{a|aa}])){3,4})(?<after>)",
                input = "aaaaaaaa",
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
                "[[\"aaaaaaaa\",\"\",\"aaaaaaaa\",\"aa\",\"aa\",\"\"],[[0,8],[0,0],[0,8],[6,8],[6,8],[8,8]],0,8]"
        "#,
    );
    assert_true(
        r#"
            var pattern = "(?<before>)(?<whole>(?<iteration>(?<c>[\\q{a|aa}])){3,4}?)(?<after>)",
                input = "aaaaaaaa",
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
            var pattern = "(?<before>)(?<whole>(?<iteration>(?<c>[\\q{aa|aaa}])){3})(?<after>)",
                input = "!aaaaaa",
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
                "[[\"aaaaaa\",\"\",\"aaaaaa\",\"aa\",\"aa\",\"\"],[[1,7],[1,1],[1,7],[5,7],[5,7],[7,7]],1,7]"
        "#,
    );

    assert_true(
        r#"
            var pattern =
                "(?<before>)(?<whole>(?<iteration>(?<c>[\\q{\ud83d\ude00|\ud83d\ude00a|aX}])){3})(?<after>)",
                input = "\ud83d\ude00aX\ud83d\ude00",
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
                "[[\"\ud83d\ude00aX\ud83d\ude00\",\"\",\"\ud83d\ude00aX\ud83d\ude00\",\"\ud83d\ude00\",\"\ud83d\ude00\",\"\"],[[0,6],[0,0],[0,6],[4,6],[4,6],[6,6]],0,6]"
        "#,
    );
    assert_true(
        r#"
            var pattern = "(?<before>)(?<whole>(?<iteration>(?<c>[\\q{S|sa|aX}])){3})(?<after>)",
                input = "\u017faX\u017f",
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
                "[[\"\u017faX\u017f\",\"\",\"\u017faX\u017f\",\"\u017f\",\"\u017f\",\"\"],[[0,4],[0,0],[0,4],[3,4],[3,4],[4,4]],0,4]"
        "#,
    );
    assert_true(
        r#"
            var pattern = "(?<before>)(?<whole>(?<iteration>(?<c>[\\p{Any}\\q{ab}])){3})(?<after>)",
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
            var pattern = "(?<before>)(?<whole>(?<iteration>(?<c>[\\p{Any}\\q{ab}])){3}?)(?<after>)",
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
            var pattern = "(?<before>)(?<whole>(?<iteration>(?<c>[a-z\\q{ab}])){3})(?<after>)",
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
            var pattern = "(?<before>)(?<whole>(?<iteration>(?<c>[\\p{Any}\\q{ab}])){3})(?<after>)",
                input = "\ud83d\ude00aba",
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
                "[[\"\ud83d\ude00aba\",\"\",\"\ud83d\ude00aba\",\"a\",\"a\",\"\"],[[0,5],[0,0],[0,5],[4,5],[4,5],[5,5]],0,5]"
        "#,
    );

    assert_true(
        r#"
            var pattern = "(?<before>)(?<whole>(?<c>[\\q{aa|aaa}]{3}))(?<after>)",
                input = "aaaaaa",
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
            ]) === "[[\"aaaaaa\",\"\",\"aaaaaa\",\"aaaaaa\",\"\"],[[0,6],[0,0],[0,6],[0,6],[6,6]],0,6]"
        "#,
    );
    assert_true(
        r#"
            var pattern = "(?<before>)(?<whole>(?<iteration>(?<c>[\\q{ab|aba}])){3})(?<after>)",
                input = "abababa",
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
                "[[\"abababa\",\"\",\"abababa\",\"aba\",\"aba\",\"\"],[[0,7],[0,0],[0,7],[4,7],[4,7],[7,7]],0,7]"
        "#,
    );
    assert_true(
        r#"
            var pattern = "(?<before>)(?<whole>(?<iteration>(?<c>[\\q{\\(|\\(a|aX}])){3})(?<after>)",
                input = "(aX(",
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
                "[[\"(aX(\",\"\",\"(aX(\",\"(\",\"(\",\"\"],[[0,4],[0,0],[0,4],[3,4],[3,4],[4,4]],0,4]"
        "#,
    );

    let mut realm = Realm::default();
    realm.eval(r"var r=/(?<before>)(?<whole>(?<word>[\q{aa|aaa}]){3,4})(?<after>)/dvg,clone=new RegExp(r),m=r.exec('aaaaaaaa')").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval(r#"
        m.groups.whole === 'aaaaaaaa' && m.groups.word === 'aa' && m.indices.groups.word === m.indices[
                3] && m.indices[3][0] === 6 && m.indices[3][1] === 8 && clone.exec('!') === null &&
            clone.source === r.source && clone.lastIndex === 0 && r.lastIndex === 8
    "#),Ok(Value::Boolean(true)));
}

#[test]
fn match_all_preserves_capture_metadata() {
    let source = format!(
        r#"
            (function() {{
                {DESCRIBE_MATCH}

                var r = /(?<whole>(?<word>[\q{{aa|aaa}}]){{3,4}})/dvg,
                    a = Array.from('!aaaaaaXaaaaaaaaaa!'.matchAll(r));
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
                "aaaaaa",
                "aaaaaa",
                "aa"
              ],
              "indices": [
                [
                  1,
                  7
                ],
                [
                  1,
                  7
                ],
                [
                  5,
                  7
                ]
              ],
              "index": 1,
              "input": "!aaaaaaXaaaaaaaaaa!",
              "groupNames": [
                "whole",
                "word"
              ],
              "namedCaptures": [
                "aaaaaa",
                "aa"
              ],
              "namedIndices": [
                [
                  1,
                  7
                ],
                [
                  5,
                  7
                ]
              ],
              "namedAliasesValid": true,
              "groupsPrototypeValid": true
            },
            {
              "matches": [
                "aaaaaaaaa",
                "aaaaaaaaa",
                "aaa"
              ],
              "indices": [
                [
                  8,
                  17
                ],
                [
                  8,
                  17
                ],
                [
                  14,
                  17
                ]
              ],
              "index": 8,
              "input": "!aaaaaaXaaaaaaaaaa!",
              "groupNames": [
                "whole",
                "word"
              ],
              "namedCaptures": [
                "aaaaaaaaa",
                "aaa"
              ],
              "namedIndices": [
                [
                  8,
                  17
                ],
                [
                  14,
                  17
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
                r = /(?<whole>(?<word>[\q{aa|aaa}]){3,4})/vg,
                out = '!aaaaaaXaaaaaaaaaa!'.replace(r, function(all, w, word, offset, input, g) {
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
          "![aa]X[aaa]a!",
          [
            [
              "aaaaaa",
              "aaaaaa",
              "aa",
              1,
              "!aaaaaaXaaaaaaaaaa!",
              6,
              true,
              true
            ],
            [
              "aaaaaaaaa",
              "aaaaaaaaa",
              "aaa",
              8,
              "!aaaaaaXaaaaaaaaaa!",
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

            var r = /(?<whole>(?<word>[\q{ab|abc|c}]){3})/dvg;
            r.lastIndex = 9;
            var a = '!abcab'.split(r),
                at = '!abcab'.search(r);
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
            "abcab",
            "ab",
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

            var r = /[\q{aa|aaa}]{3}/vg;
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

                var r = /(?<\u{{1D49C}}>)(?<whole>(?<word>[\q{{😀|😀a|aX}}]){{3}})(?<after>)/dvgy;
                r.lastIndex = 1;
                var m = r.exec('😀aX😀!');
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
              "😀aX😀",
              "",
              "😀aX😀",
              "😀",
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
                4,
                6
              ],
              [
                6,
                6
              ]
            ],
            "index": 0,
            "input": "😀aX😀!",
            "groupNames": [
              "𝒜",
              "whole",
              "word",
              "after"
            ],
            "namedCaptures": [
              "",
              "😀aX😀",
              "😀",
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
                4,
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

                var r = /(?<before>)(?<whole>(?<word>[\q{{S|sa|aX}}]){{3,4}})(?<after>)/dvi,
                    m = r.exec('ſaXſ!');
                return JSON.stringify([describeMatch(m)], null, 2);
            }})()
        "#
    );
    let report = eval_string(&mut Realm::default(), &source);
    insta::assert_snapshot!(report, @r#"
        [
          {
            "matches": [
              "ſaXſ",
              "",
              "ſaXſ",
              "ſ",
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
            "input": "ſaXſ!",
            "groupNames": [
              "before",
              "whole",
              "word",
              "after"
            ],
            "namedCaptures": [
              "",
              "ſaXſ",
              "ſ",
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

                var g = /(?<whole>(?<word>[\q{{aa|aaa}}]){{3,4}})/dv.exec('aaaaaaaa!'),
                    l = /(?<whole>(?<word>[\q{{aa|aaa}}]){{3,4}}?)/dv.exec('aaaaaaaa!');
                return JSON.stringify([describeMatch(g), describeMatch(l)], null, 2);
            }})()
        "#
    );
    let report = eval_string(&mut Realm::default(), &source);
    insta::assert_snapshot!(report, @r#"
        [
          {
            "matches": [
              "aaaaaaaa",
              "aaaaaaaa",
              "aa"
            ],
            "indices": [
              [
                0,
                8
              ],
              [
                0,
                8
              ],
              [
                6,
                8
              ]
            ],
            "index": 0,
            "input": "aaaaaaaa!",
            "groupNames": [
              "whole",
              "word"
            ],
            "namedCaptures": [
              "aaaaaaaa",
              "aa"
            ],
            "namedIndices": [
              [
                0,
                8
              ],
              [
                6,
                8
              ]
            ],
            "namedAliasesValid": true,
            "groupsPrototypeValid": true
          },
          {
            "matches": [
              "aaaaaaaa",
              "aaaaaaaa",
              "aa"
            ],
            "indices": [
              [
                0,
                8
              ],
              [
                0,
                8
              ],
              [
                6,
                8
              ]
            ],
            "index": 0,
            "input": "aaaaaaaa!",
            "groupNames": [
              "whole",
              "word"
            ],
            "namedCaptures": [
              "aaaaaaaa",
              "aa"
            ],
            "namedIndices": [
              [
                0,
                8
              ],
              [
                6,
                8
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

                var r = /(?<whole>(?<word>[\q{{ab|abc|c}}]){{3}})/dvg,
                    s = /(?<whole>(?<word>[\q{{ab|abc|c}}]){{3}})/dvgy,
                    m = r.exec('!abcab'),
                    n = s.exec('!abcab');
                s.lastIndex = 1;
                var p = s.exec('!abcab');
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
              "abcab",
              "abcab",
              "ab"
            ],
            "indices": [
              [
                1,
                6
              ],
              [
                1,
                6
              ],
              [
                4,
                6
              ]
            ],
            "index": 1,
            "input": "!abcab",
            "groupNames": [
              "whole",
              "word"
            ],
            "namedCaptures": [
              "abcab",
              "ab"
            ],
            "namedIndices": [
              [
                1,
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
          null,
          {
            "matches": [
              "abcab",
              "abcab",
              "ab"
            ],
            "indices": [
              [
                1,
                6
              ],
              [
                1,
                6
              ],
              [
                4,
                6
              ]
            ],
            "index": 1,
            "input": "!abcab",
            "groupNames": [
              "whole",
              "word"
            ],
            "namedCaptures": [
              "abcab",
              "ab"
            ],
            "namedIndices": [
              [
                1,
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
          6,
          6
        ]
        "#);
}

#[test]
fn required_string_class_proofs_keep_predicates_references_consuming_siblings_and_unknown_leaves_closed()
 {
    assert_true(
        r#"var marker=0,ok=false;try{ok=(function(){var pattern="^()([\\q{aa|aaa}]){3}()",rows=[],metadata=true;
for(var multi of [false,true]){var r=new RegExp(pattern,'dvg'+(multi?'m':''));metadata=metadata&&r.source===pattern;
for(var input of ['aaaaaa','ababa','\nab\n','']){r.lastIndex=0;var m=r.exec(input),result=null;if(m!==null)result={values:Array.from(m,v=>v===undefined?'<undefined>':v),indices:Array.from(m.indices,v=>v===undefined?'<undefined>':v),index:m.index};rows.push([multi,input,result,r.lastIndex]);}}
return metadata&&JSON.stringify(rows)==="[[false,\"aaaaaa\",{\"values\":[\"aaaaaa\",\"\",\"aa\",\"\"],\"indices\":[[0,6],[0,0],[4,6],[6,6]],\"index\":0},6],[false,\"ababa\",null,0],[false,\"\\nab\\n\",null,0],[false,\"\",null,0],[true,\"aaaaaa\",{\"values\":[\"aaaaaa\",\"\",\"aa\",\"\"],\"indices\":[[0,6],[0,0],[4,6],[6,6]],\"index\":0},6],[true,\"ababa\",null,0],[true,\"\\nab\\n\",null,0],[true,\"\",null,0]]"})();}finally{marker=3}ok&&marker===3"#,
    );
    assert_true(
        r#"var marker=0,ok=false;try{ok=(function(){var pattern="()([\\q{aa|aaa}]){3}()$",rows=[],metadata=true;
for(var multi of [false,true]){var r=new RegExp(pattern,'dvg'+(multi?'m':''));metadata=metadata&&r.source===pattern;
for(var input of ['aaaaaa','ababa','\nab\n','']){r.lastIndex=0;var m=r.exec(input),result=null;if(m!==null)result={values:Array.from(m,v=>v===undefined?'<undefined>':v),indices:Array.from(m.indices,v=>v===undefined?'<undefined>':v),index:m.index};rows.push([multi,input,result,r.lastIndex]);}}
return metadata&&JSON.stringify(rows)==="[[false,\"aaaaaa\",{\"values\":[\"aaaaaa\",\"\",\"aa\",\"\"],\"indices\":[[0,6],[0,0],[4,6],[6,6]],\"index\":0},6],[false,\"ababa\",null,0],[false,\"\\nab\\n\",null,0],[false,\"\",null,0],[true,\"aaaaaa\",{\"values\":[\"aaaaaa\",\"\",\"aa\",\"\"],\"indices\":[[0,6],[0,0],[4,6],[6,6]],\"index\":0},6],[true,\"ababa\",null,0],[true,\"\\nab\\n\",null,0],[true,\"\",null,0]]"})();}finally{marker=3}ok&&marker===3"#,
    );
    for pattern in [
        r"^()([[\q{aa|aaa}]&&\p{RGI_Emoji}]){3}()",
        r"()([[\q{aa|aaa}]&&\p{RGI_Emoji}]){3}()$",
        r"()([\q{aa|aaa}]){3}()\1",
        r"()([\q{aa|aaa}]){3}()a",
        r"a()([\q{aa|aaa}]){3}()",
        r"()(([\q{aa|aaa}]){3}){3}()",
        r"()([\q{ab}&&\p{Any}&&\p{RGI_Emoji}]){3}()",
        r"()([\q{aa|aaa}]){3}()[\q{ef}]",
    ] {
        let mut realm = Realm::default();
        crate::common::regexp_oracle::assert_matches_reference(
            &mut realm,
            &format!(
                "var marker=0;try{{new RegExp({:?},'v').test('aaaaaa')}}catch{{marker=1}}finally{{marker=2}};marker",
                JsString::from(pattern)
            ),
        );
    }
}

#[test_case(
    r#"(?<before>)(?<whole>(?<iteration>(?<c>[\q{a|ab}])){3})(?<after>)"#,
    "dgv",
    JsString::from("ab"),
    0;
    "input_ab_from_0_variant_1"
)]
#[test_case(
    r#"(?<before>)(?<whole>(?<iteration>(?<c>[\q{aa|aaa}])){3})(?<after>)"#,
    "dgvy",
    JsString::from("!aaaaaa"),
    0;
    "input_aaaaaa_from_0_variant_2"
)]
fn no_match_resets_last_index(
    pattern: &str,
    flags: &str,
    input: JsString,
    initial_last_index: usize,
) {
    assert_no_match(pattern, flags, input, initial_last_index);
}
