//! Unconditional predicates retain member/count backtracking and lexical captures.
use crate::common::{assert_true, eval_string};

use crate::common::regexp_captures::assert_no_match;
use crate::common::regexp_matches::DESCRIBE_MATCH;
use spite_core::JsString;
use spite_runtime::{Realm, Value};
use test_case::test_case;
#[test]
fn asserted_repeated_string_class_native_tables_consumers_and_zero_iteration_scopes() {
    assert_true(
        r#"
            var pattern = "(?<before>^)(?<whole>(?<iteration>(?<c>[\\q{aa|aaa}]))*)(?<after>$)",
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
            var pattern = "(?<before>^)(?<whole>(?<iteration>(?<c>[\\q{aa|aaa}]))*?)(?<after>$)",
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
            var pattern = "(?<before>^)(?<whole>(?<iteration>(?<c>[\\q{aa|aaa}]))+)(?<after>$)",
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
            var pattern = "(?<before>^)(?<whole>(?<iteration>(?<c>[\\q{aa|aaa}]))+?)(?<after>$)",
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
            var pattern = "(?<before>^)(?<whole>(?<iteration>(?<c>[\\q{aa|aaa}])){3})(?<after>$)",
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
            var pattern = "(?<before>^)(?<whole>(?<iteration>(?<c>[\\q{aa|aaa}])){3,4}?)(?<after>$)",
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
            var pattern = "(?<before>^)(?<whole>(?<iteration>(?<c>[\\q{aa|aaa}])){0})(?<after>$)",
                input = "",
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
                "[[\"\",\"\",\"\",\"<undefined>\",\"<undefined>\",\"\"],[[0,0],[0,0],[0,0],\"<undefined>\",\"<undefined>\",[0,0]],0,0]"
        "#,
    );

    assert_true(
        r#"
            var pattern = "(?<before>^)(?<whole>(?<iteration>(?<c>[\\q{aa|aaa}])){0,2}?)(?<after>$)",
                input = "aaa",
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
                "[[\"aaa\",\"\",\"aaa\",\"aaa\",\"aaa\",\"\"],[[0,3],[0,0],[0,3],[0,3],[0,3],[3,3]],0,3]"
        "#,
    );
    assert_true(
        r#"
            var pattern = "(?<before>^)(?<whole>(?<iteration>(?<c>[\\q{aa|aaa}]))?)(?<after>$)",
                input = "aaa",
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
                "[[\"aaa\",\"\",\"aaa\",\"aaa\",\"aaa\",\"\"],[[0,3],[0,0],[0,3],[0,3],[0,3],[3,3]],0,3]"
        "#,
    );
    assert_true(
        r#"
            var pattern = "(?<before>^)(?<whole>(?<iteration>(?<c>[\\q{aa|aaa}]))??)(?<after>$)",
                input = "aaa",
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
                "[[\"aaa\",\"\",\"aaa\",\"aaa\",\"aaa\",\"\"],[[0,3],[0,0],[0,3],[0,3],[0,3],[3,3]],0,3]"
        "#,
    );
    assert_true(
        r#"
            var pattern = "(?<before>^)(?<whole>(?<iteration>(?<c>[\\q{aa|aaa}])){2,})(?<after>$)",
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
            var pattern = "(?<before>\\b)(?<whole>(?<iteration>(?<c>[\\q{a|aa}]))*)(?<after>\\b)",
                input = "aaaa!",
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
            var pattern = "(?<before>\\b)(?<whole>(?<iteration>(?<c>[\\q{a|aa}]))*?)(?<after>\\b)",
                input = "aaaa!",
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
                "[[\"\",\"\",\"\",\"<undefined>\",\"<undefined>\",\"\"],[[0,0],[0,0],[0,0],\"<undefined>\",\"<undefined>\",[0,0]],0,0]"
        "#,
    );
    assert_true(
        r#"
            var pattern = "(?<before>^)(?<whole>(?<iteration>(?<c>[\\q{ab|abc|c}])){2})(?<after>$)",
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
            var pattern = "(?<before>^)(?<whole>(?<iteration>(?<c>[\\p{Any}\\q{ab}])){2})(?<after>$)",
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
            var pattern = "(?<before>^)(?<whole>(?<iteration>(?<c>[a-z\\q{ab}])){2})(?<after>$)",
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
            var pattern =
                "(?<before>^)(?<whole>(?<iteration>(?<c>[\\q{\ud83d\ude00|\ud83d\ude00a|aX}])){2})(?<after>$)",
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
            var pattern = "(?<before>^)(?<whole>(?<iteration>(?<c>[\\q{S|sa|aX}])){2})(?<after>$)",
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
            var pattern = "(?<before>^)(?<whole>(?<iteration>(?<c>[\\q{a|aa}]))+?)(?<after>$)",
                input = "\naaaa\n",
                r = new RegExp(pattern, "dgvm"),
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
            var pattern = "(?<before>$)(?<whole>(?<iteration>(?<c>[\\q{\\n|\\na|aX}]))*)(?<after>^)",
                input = "\n\naX\n",
                r = new RegExp(pattern, "dgvm"),
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
                "[[\"\\n\\naX\\n\",\"\",\"\\n\\naX\\n\",\"\\n\",\"\\n\",\"\"],[[0,5],[0,0],[0,5],[4,5],[4,5],[5,5]],0,5]"
        "#,
    );
    assert_true(
        r#"
            var pattern = "(?<before>\\b)(?<whole>(?<iteration>(?<c>[\\q{a|aa}]))*)(?<after>\\b)",
                input = "!aaaa!",
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
            var pattern = "(?<before>^)(?<whole>(?<iteration>(?<c>[\\q{a|aa}]))*?)(?<after>\\b)",
                input = "x",
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
                "[[\"\",\"\",\"\",\"<undefined>\",\"<undefined>\",\"\"],[[0,0],[0,0],[0,0],\"<undefined>\",\"<undefined>\",[0,0]],0,0]"
        "#,
    );
    assert_true(
        r#"
            var pattern = "(?<before>\\B)(?<whole>(?<iteration>(?<c>[\\q{a|aa}]))*)(?<after>\\B)",
                input = "",
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
                "[[\"\",\"\",\"\",\"<undefined>\",\"<undefined>\",\"\"],[[0,0],[0,0],[0,0],\"<undefined>\",\"<undefined>\",[0,0]],0,0]"
        "#,
    );

    assert_true(
        r#"
            var pattern = "(?<before>^)(?<whole>(?<c>[\\q{ab|abc|c}]*?))(?<after>$)",
                input = "abcab",
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
            ]) === "[[\"abcab\",\"\",\"abcab\",\"abcab\",\"\"],[[0,5],[0,0],[0,5],[0,5],[5,5]],0,5]"
        "#,
    );

    assert_true(
        r#"
            var pattern = "(?<before>^)(?<whole>(?<iteration>(?<c>[\\q{ab}&&\\q{cd}]))*)(?<after>$)",
                input = "",
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
                "[[\"\",\"\",\"\",\"<undefined>\",\"<undefined>\",\"\"],[[0,0],[0,0],[0,0],\"<undefined>\",\"<undefined>\",[0,0]],0,0]"
        "#,
    );

    assert_true(
        r#"
            var pattern = "(?<before>^)(?<whole>(?<iteration>(?<c>[\\q{ab}&&\\q{cd}])){0})(?<after>$)",
                input = "",
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
                "[[\"\",\"\",\"\",\"<undefined>\",\"<undefined>\",\"\"],[[0,0],[0,0],[0,0],\"<undefined>\",\"<undefined>\",[0,0]],0,0]"
        "#,
    );

    let mut realm = Realm::default();
    realm
        .eval(r"var r=/^(?<whole>(?<word>[\q{aa|aaa}])*)$/dvg,clone=new RegExp(r),m=r.exec('aaaa')")
        .unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval(r#"
        var n = clone.exec('');
        m.groups.whole === 'aaaa' && m.groups.word === 'aa' && m.indices.groups.word === m.indices[
            2] && m.indices[2][0] === 2 && m.indices[2][1] === 4 && n[0] === '' && n.groups.whole ===
            '' && n.groups.word === undefined && n.indices.groups.word === undefined && clone
            .source === r.source && clone.lastIndex === 0 && r.lastIndex === 4
    "#),Ok(Value::Boolean(true)));
}

#[test]
fn multiline_match_all_preserves_capture_metadata() {
    let source = format!(
        r#"
            (function() {{
                {DESCRIBE_MATCH}

                var r = /^(?<whole>(?<word>[\q{{aa|aaa}}])*)$/dvgm,
                    a = Array.from('aaaa\n\naaaaaa'.matchAll(r));
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
                r = /^(?<whole>(?<word>[\q{a|aa}])*?)$/vgm,
                out = 'aaa\n\nbbbb'.replace(r, function(all, w, word, offset, input, g) {
                    calls.push([all, w, word === undefined ? '<undefined>' : word, offset,
                        input, arguments.length, g.word === word, Object
                        .getPrototypeOf(g) === null
                    ]);
                    return word === undefined ? '[E]' : '[' + word + ']'
                });
            return JSON.stringify([out, calls, r.lastIndex], null, 2);
        })()
    "#;
    let report = eval_string(&mut Realm::default(), source);
    insta::assert_snapshot!(report, @r#"
        [
          "[a]\n[E]\nbbbb",
          [
            [
              "aaa",
              "aaa",
              "a",
              0,
              "aaa\n\nbbbb",
              6,
              true,
              true
            ],
            [
              "",
              "",
              "<undefined>",
              4,
              "aaa\n\nbbbb",
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

            var r = /(?<before>\b)(?<whole>(?<word>[\q{a|aa}])*)(?<after>\b)/dvg;
            r.lastIndex = 9;
            var a = '!aaaa!'.split(r),
                at = '!aaaa!'.search(r);
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
            "aaaa",
            "aa",
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

            var r = /^[\q{aa|aaa}]*$/vg;
            Object.defineProperty(r, 'lastIndex', {
                writable: false
            });
            var caught = false;
            try {
                r.exec('aaaa')
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

                var r = /^(?<\u{{1D49C}}>)(?<whole>(?<word>[\q{{😀|😀a|aX}}]){{2}})(?<after>)$/dvgy;
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
            "input": "😀aX",
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

                var r = /(?<before>\b)(?<whole>(?<word>[\q{{S|sa|aX}}]){{2}})(?<after>\b)/dvi,
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

                var g = /^(?<whole>(?<word>[\q{{aa|aaa}}]){{0,2}})\b/dv.exec('aaaa!'),
                    l = /^(?<whole>(?<word>[\q{{aa|aaa}}]){{0,2}}?)\b/dv.exec('aaaa!');
                return JSON.stringify([describeMatch(g), describeMatch(l)], null, 2);
            }})()
        "#
    );
    let report = eval_string(&mut Realm::default(), &source);
    insta::assert_snapshot!(report, @r#"
        [
          {
            "matches": [
              "aaaa",
              "aaaa",
              "aa"
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
            "input": "aaaa!",
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
          },
          {
            "matches": [
              "",
              "",
              "<undefined>"
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
              "<undefined>"
            ],
            "index": 0,
            "input": "aaaa!",
            "groupNames": [
              "whole",
              "word"
            ],
            "namedCaptures": [
              "",
              "<undefined>"
            ],
            "namedIndices": [
              [
                0,
                0
              ],
              "<undefined>"
            ],
            "namedAliasesValid": true,
            "groupsPrototypeValid": true
          }
        ]
        "#);
}

#[test]
fn exec_preserves_capture_metadata() {
    let source = format!(
        r#"
            (function() {{
                {DESCRIBE_MATCH}

                var r =
                    /(?<before>^)(?:(?<zero>)|(?<never>))(?<whole>(?<word>[\q{{aa|aaa}}])*)(?<after>$)/dv,
                    m = r.exec('aaaa');
                return JSON.stringify([describeMatch(m)], null, 2);
            }})()
        "#
    );
    let report = eval_string(&mut Realm::default(), &source);
    insta::assert_snapshot!(report, @r#"
        [
          {
            "matches": [
              "aaaa",
              "",
              "",
              "<undefined>",
              "aaaa",
              "aa",
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
                0
              ],
              "<undefined>",
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
            "input": "aaaa",
            "groupNames": [
              "before",
              "zero",
              "never",
              "whole",
              "word",
              "after"
            ],
            "namedCaptures": [
              "",
              "",
              "<undefined>",
              "aaaa",
              "aa",
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
              "<undefined>",
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
          }
        ]
        "#);
}

#[test]
fn asserted_repeated_string_class_proofs_keep_conditional_predicates_and_unproved_leaves_closed() {
    let expected_serialized_output = JsString::from(concat!(
        r#"[[false,"aaaaaa",{"values":["aaaaaa","aaa"],"indices":[[0,6],[3,6]],"index":0},6],"#,
        r#"[false,"ababa",null,0],[false,"\nab\n",null,0],"#,
        r#"[false,"",{"values":["","<undefined>"],"indices":[[0,0],"<undefined>"],"index":0},0],"#,
        r#"[true,"aaaaaa",{"values":["aaaaaa","aaa"],"indices":[[0,6],[3,6]],"index":0},6],"#,
        r#"[true,"ababa",null,0],"#,
        r#"[true,"\nab\n","#,
        r#"{"values":["","<undefined>"],"indices":[[0,0],"<undefined>"],"index":0},0],"#,
        r#"[true,"",{"values":["","<undefined>"],"indices":[[0,0],"<undefined>"],"index":0},0]]"#,
    ));
    let expected_serialized_output_2 = JsString::from(concat!(
        r#"[[false,"aaaaaa","#,
        r#"{"values":["aaaaaa","aaa","aaa","<undefined>"],"#,
        r#""indices":[[0,6],[3,6],[3,6],"<undefined>"],"index":0},6],[false,"ababa",null,0],"#,
        r#"[false,"\nab\n",null,0],"#,
        r#"[false,"","#,
        r#"{"values":["","<undefined>","<undefined>","<undefined>"],"#,
        r#""indices":[[0,0],"<undefined>","<undefined>","<undefined>"],"index":0},0],"#,
        r#"[true,"aaaaaa","#,
        r#"{"values":["aaaaaa","aaa","aaa","<undefined>"],"#,
        r#""indices":[[0,6],[3,6],[3,6],"<undefined>"],"index":0},6],[true,"ababa",null,0],"#,
        r#"[true,"\nab\n","#,
        r#"{"values":["","<undefined>","<undefined>","<undefined>"],"#,
        r#""indices":[[0,0],"<undefined>","<undefined>","<undefined>"],"index":0},0],"#,
        r#"[true,"","#,
        r#"{"values":["","<undefined>","<undefined>","<undefined>"],"#,
        r#""indices":[[0,0],"<undefined>","<undefined>","<undefined>"],"index":0},0]]"#,
    ));

    assert_true(&format!(
        r#"
            var marker = 0,
                ok = false;
            try {{
                ok = (function() {{
                    var pattern = "^([\\q{{|aa|aaa}}])*$",
                        rows = [],
                        metadata = true;
                    for (var multi of [false, true]) {{
                        var r = new RegExp(pattern, 'dvg' + (multi ? 'm' : ''));
                        metadata = metadata && r.source === pattern;
                        for (var input of ['aaaaaa', 'ababa', '\nab\n', '']) {{
                            r.lastIndex = 0;
                            var m = r.exec(input),
                                result = null;
                            if (m !== null) result = {{
                                values: Array.from(m, v => v === undefined ? '<undefined>' :
                                    v),
                                indices: Array.from(m.indices, v => v === undefined ?
                                    '<undefined>' : v),
                                index: m.index
                            }};
                            rows.push([multi, input, result, r.lastIndex]);
                        }}
                    }}
                    return metadata && JSON.stringify(rows) === {expected_serialized_output:?}
                }})();
            }} finally {{
                marker = 3
            }}
            ok && marker === 3
        "#
    ));
    assert_true(&format!(
        r#"
            var marker = 0,
                ok = false;
            try {{
                ok = (function() {{
                    var pattern = "^(([\\q{{aa|aaa}}])|())*$",
                        rows = [],
                        metadata = true;
                    for (var multi of [false, true]) {{
                        var r = new RegExp(pattern, 'dvg' + (multi ? 'm' : ''));
                        metadata = metadata && r.source === pattern;
                        for (var input of ['aaaaaa', 'ababa', '\nab\n', '']) {{
                            r.lastIndex = 0;
                            var m = r.exec(input),
                                result = null;
                            if (m !== null) result = {{
                                values: Array.from(m, v => v === undefined ? '<undefined>' :
                                    v),
                                indices: Array.from(m.indices, v => v === undefined ?
                                    '<undefined>' : v),
                                index: m.index
                            }};
                            rows.push([multi, input, result, r.lastIndex]);
                        }}
                    }}
                    return metadata && JSON.stringify(rows) === {expected_serialized_output_2:?}
                }})();
            }} finally {{
                marker = 3
            }}
            ok && marker === 3
        "#
    ));
    assert_true(
        r#"
            var pattern = "(^)*([\\q{aa|aaa}])*$",
                input = "aba",
                r = new RegExp(pattern, "dv"),
                m = r.exec(input);
            r.source === pattern && r.flags === "dv" && r.hasIndices && r.unicodeSets && m !== null && m
                .input === input && m.groups === undefined && m.indices.groups === undefined && Object
                .hasOwn(m, "groups") && Object.hasOwn(m.indices, "groups") && JSON.stringify(m === null ?
                    null : [Array.from(m, v => v === undefined ? "<undefined>" : v), Array.from(m.indices,
                        v => v === undefined ? "<undefined>" : v), m.index, r.lastIndex]) ===
                "[[\"\",\"<undefined>\",\"<undefined>\"],[[3,3],\"<undefined>\",\"<undefined>\"],3,0]"
        "#,
    );
    assert_true(
        r#"
            var pattern = "(^|)([\\q{aa|aaa}])*$",
                input = "aba",
                r = new RegExp(pattern, "dv"),
                m = r.exec(input);
            r.source === pattern && r.flags === "dv" && r.hasIndices && r.unicodeSets && (m === null || (m
                .input === input && m.groups === undefined && m.indices.groups === undefined &&
                Object.hasOwn(m, "groups") && Object.hasOwn(m.indices, "groups"))) && JSON.stringify(
                m === null ? null : [Array.from(m, v => v === undefined ? "<undefined>" : v), Array
                    .from(m.indices, v => v === undefined ? "<undefined>" : v), m.index, r.lastIndex
                ]) === "[[\"\",\"\",\"<undefined>\"],[[3,3],[3,3],\"<undefined>\"],3,0]"
        "#,
    );
    assert_true(
        r#"
            var pattern = "^([\\q{aa|aaa}])*(\\b|)",
                input = "aba",
                r = new RegExp(pattern, "dv"),
                m = r.exec(input);
            r.source === pattern && r.flags === "dv" && r.hasIndices && r.unicodeSets && (m === null || (m
                .input === input && m.groups === undefined && m.indices.groups === undefined &&
                Object.hasOwn(m, "groups") && Object.hasOwn(m.indices, "groups"))) && JSON.stringify(
                m === null ? null : [Array.from(m, v => v === undefined ? "<undefined>" : v), Array
                    .from(m.indices, v => v === undefined ? "<undefined>" : v), m.index, r.lastIndex
                ]) === "[[\"\",\"<undefined>\",\"\"],[[0,0],\"<undefined>\",[0,0]],0,0]"
        "#,
    );
    for pattern in [
        r"^([[\q{|aa|aaa}]&&\p{RGI_Emoji}])*$",
        r"^(([[\q{aa|aaa}]&&\p{RGI_Emoji}])|())*$",
        r"(^([\q{aa|aaa}]))*$",
        r"^([\q{aa|aaa}])*\1",
        r"a^([\q{aa|aaa}])*$",
        r"^([\q{aa|aaa}])*a$",
        r"^(([\q{aa|aaa}])*){2}$",
        r"^([\q{aa|aaa}])*[\q{ab}]$",
        r"^([\q{ab}&&\p{Any}&&\p{RGI_Emoji}]){0}$",
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
    r#"(?<before>^)(?<whole>(?<iteration>(?<c>[\q{aa|aaa}])){0})(?<after>$)"#,
    "dgv",
    JsString::from("a"),
    0;
    "input_a_from_0_variant_1"
)]
#[test_case(
    r#"(?<before>\b)(?<whole>(?<iteration>(?<c>[\q{a|aa}]))*)(?<after>\b)"#,
    "dgvy",
    JsString::from("!aaaa!"),
    0;
    "input_aaaa_from_0_variant_2"
)]
#[test_case(
    r#"(?<before>\b\B)(?<whole>(?<iteration>(?<c>[\q{a|aa}]))*)(?<after>)"#,
    "dgv",
    JsString::from("a"),
    0;
    "input_a_from_0_variant_3"
)]
#[test_case(
    r#"(?<before>^)(?<whole>(?<iteration>(?<c>[\q{ab}&&\q{cd}]))+)(?<after>$)"#,
    "dgv",
    JsString::from(""),
    0;
    "empty_input_from_0_variant_4"
)]
fn no_match_resets_last_index(
    pattern: &str,
    flags: &str,
    input: JsString,
    initial_last_index: usize,
) {
    assert_no_match(pattern, flags, input, initial_last_index);
}
