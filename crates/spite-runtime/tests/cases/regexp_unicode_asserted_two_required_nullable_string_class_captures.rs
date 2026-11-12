//! Required empty iterations preserve class-versus-empty capture identity.
use crate::common::assert_true;
use crate::common::eval_string;
use crate::common::regexp_matches::DESCRIBE_MATCH;
use spite_core::JsString;
use spite_runtime::{Realm, Value};
#[test]
fn asserted_two_required_nullable_string_class_native_tables_consumers_and_required_empty_identity()
{
    assert_true(
        r#"
            var pattern = "(?<before>^)(?<whole>(?<iteration>(?<c>[\\q{|a|ab}])){2})(?<after>$)",
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
            ]) === "[[\"\",\"\",\"\",\"\",\"\",\"\"],[[0,0],[0,0],[0,0],[0,0],[0,0],[0,0]],0,0]"
        "#,
    );
    assert_true(
        r#"
            var pattern = "(?<before>^)(?<whole>(?<iteration>(?<c>[\\q{|a|ab}])){2}?)(?<after>$)",
                input = "a",
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
            ]) === "[[\"a\",\"\",\"a\",\"\",\"\",\"\"],[[0,1],[0,0],[0,1],[1,1],[1,1],[1,1]],0,1]"
        "#,
    );
    assert_true(
        r#"
            var pattern = "(?<before>^)(?<whole>(?<iteration>(?<c>[\\q{|a|ab}])){2})(?<after>$)",
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
            ]) === "[[\"ab\",\"\",\"ab\",\"\",\"\",\"\"],[[0,2],[0,0],[0,2],[2,2],[2,2],[2,2]],0,2]"
        "#,
    );
    assert_true(
        r#"
            var pattern = "(?<before>^)(?<whole>(?<iteration>(?<c>[\\q{|a|ab}])){2,3})(?<after>$)",
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
            var pattern = "(?<before>^)(?<whole>(?<iteration>(?<c>[\\q{|a|ab}])){2,3}?)(?<after>$)",
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
            var pattern = "(?<before>^)(?<whole>(?<iteration>(?<c>[\\q{|a|ab}])){2,})(?<after>$)",
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
                "[[\"aaaa\",\"\",\"aaaa\",\"a\",\"a\",\"\"],[[0,4],[0,0],[0,4],[3,4],[3,4],[4,4]],0,4]"
        "#,
    );

    assert_true(
        r#"
            var pattern = "(?<before>^)(?<whole>(?<iteration>(?<c>[\\q{|a|ab}])|(?<e>)){2})(?<after>$)",
                input = "",
                r = new RegExp(pattern, "dgv"),
                ok = r.source === pattern && r.global && r.hasIndices && r.unicodeSets === true && r
                .unicode === false;
            r.lastIndex = 0;
            var m = r.exec(input);
            if (m !== null) {
                var names = ["before", "whole", "iteration", "c", "e", "after"];
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
                "[[\"\",\"\",\"\",\"\",\"\",\"<undefined>\",\"\"],[[0,0],[0,0],[0,0],[0,0],[0,0],\"<undefined>\",[0,0]],0,0]"
        "#,
    );
    assert_true(
        r#"
            var pattern = "(?<before>^)(?<whole>(?<iteration>(?<c>[\\q{|a|ab}])|(?<e>)){2}?)(?<after>$)",
                input = "a",
                r = new RegExp(pattern, "dgv"),
                ok = r.source === pattern && r.global && r.hasIndices && r.unicodeSets === true && r
                .unicode === false;
            r.lastIndex = 0;
            var m = r.exec(input);
            if (m !== null) {
                var names = ["before", "whole", "iteration", "c", "e", "after"];
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
                "[[\"a\",\"\",\"a\",\"\",\"\",\"<undefined>\",\"\"],[[0,1],[0,0],[0,1],[1,1],[1,1],\"<undefined>\",[1,1]],0,1]"
        "#,
    );
    assert_true(
        r#"
            var pattern = "(?<before>^)(?<whole>(?<iteration>(?<c>[\\q{|a|ab}])|(?<e>)){2})(?<after>$)",
                input = "ab",
                r = new RegExp(pattern, "dgv"),
                ok = r.source === pattern && r.global && r.hasIndices && r.unicodeSets === true && r
                .unicode === false;
            r.lastIndex = 0;
            var m = r.exec(input);
            if (m !== null) {
                var names = ["before", "whole", "iteration", "c", "e", "after"];
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
                "[[\"ab\",\"\",\"ab\",\"\",\"\",\"<undefined>\",\"\"],[[0,2],[0,0],[0,2],[2,2],[2,2],\"<undefined>\",[2,2]],0,2]"
        "#,
    );
    assert_true(
        r#"
            var pattern = "(?<before>^)(?<whole>(?<iteration>(?<c>[\\q{|a|ab}])|(?<e>)){2,3})(?<after>$)",
                input = "aba",
                r = new RegExp(pattern, "dgv"),
                ok = r.source === pattern && r.global && r.hasIndices && r.unicodeSets === true && r
                .unicode === false;
            r.lastIndex = 0;
            var m = r.exec(input);
            if (m !== null) {
                var names = ["before", "whole", "iteration", "c", "e", "after"];
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
                "[[\"aba\",\"\",\"aba\",\"a\",\"a\",\"<undefined>\",\"\"],[[0,3],[0,0],[0,3],[2,3],[2,3],\"<undefined>\",[3,3]],0,3]"
        "#,
    );
    assert_true(
        r#"
            var pattern = "(?<before>^)(?<whole>(?<iteration>(?<c>[\\q{|a|ab}])|(?<e>)){2,3}?)(?<after>$)",
                input = "aba",
                r = new RegExp(pattern, "dgv"),
                ok = r.source === pattern && r.global && r.hasIndices && r.unicodeSets === true && r
                .unicode === false;
            r.lastIndex = 0;
            var m = r.exec(input);
            if (m !== null) {
                var names = ["before", "whole", "iteration", "c", "e", "after"];
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
                "[[\"aba\",\"\",\"aba\",\"a\",\"a\",\"<undefined>\",\"\"],[[0,3],[0,0],[0,3],[2,3],[2,3],\"<undefined>\",[3,3]],0,3]"
        "#,
    );
    assert_true(
        r#"
            var pattern = "(?<before>^)(?<whole>(?<iteration>(?<c>[\\q{|a|ab}])|(?<e>)){2,})(?<after>$)",
                input = "aaaa",
                r = new RegExp(pattern, "dgv"),
                ok = r.source === pattern && r.global && r.hasIndices && r.unicodeSets === true && r
                .unicode === false;
            r.lastIndex = 0;
            var m = r.exec(input);
            if (m !== null) {
                var names = ["before", "whole", "iteration", "c", "e", "after"];
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
                "[[\"aaaa\",\"\",\"aaaa\",\"a\",\"a\",\"<undefined>\",\"\"],[[0,4],[0,0],[0,4],[3,4],[3,4],\"<undefined>\",[4,4]],0,4]"
        "#,
    );

    assert_true(
        r#"
            var pattern = "(?<before>^)(?<whole>(?<iteration>(?<e>)|(?<c>[\\q{|a|ab}])){2})(?<after>$)",
                input = "",
                r = new RegExp(pattern, "dgv"),
                ok = r.source === pattern && r.global && r.hasIndices && r.unicodeSets === true && r
                .unicode === false;
            r.lastIndex = 0;
            var m = r.exec(input);
            if (m !== null) {
                var names = ["before", "whole", "iteration", "e", "c", "after"];
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
                "[[\"\",\"\",\"\",\"\",\"\",\"<undefined>\",\"\"],[[0,0],[0,0],[0,0],[0,0],[0,0],\"<undefined>\",[0,0]],0,0]"
        "#,
    );
    assert_true(
        r#"
            var pattern = "(?<before>^)(?<whole>(?<iteration>(?<e>)|(?<c>[\\q{|a|ab}])){2}?)(?<after>$)",
                input = "a",
                r = new RegExp(pattern, "dgv"),
                ok = r.source === pattern && r.global && r.hasIndices && r.unicodeSets === true && r
                .unicode === false;
            r.lastIndex = 0;
            var m = r.exec(input);
            if (m !== null) {
                var names = ["before", "whole", "iteration", "e", "c", "after"];
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
                "[[\"a\",\"\",\"a\",\"a\",\"<undefined>\",\"a\",\"\"],[[0,1],[0,0],[0,1],[0,1],\"<undefined>\",[0,1],[1,1]],0,1]"
        "#,
    );
    assert_true(
        r#"
            var pattern = "(?<before>^)(?<whole>(?<iteration>(?<e>)|(?<c>[\\q{|a|ab}])){2})(?<after>$)",
                input = "ab",
                r = new RegExp(pattern, "dgv"),
                ok = r.source === pattern && r.global && r.hasIndices && r.unicodeSets === true && r
                .unicode === false;
            r.lastIndex = 0;
            var m = r.exec(input);
            if (m !== null) {
                var names = ["before", "whole", "iteration", "e", "c", "after"];
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
                "[[\"ab\",\"\",\"ab\",\"ab\",\"<undefined>\",\"ab\",\"\"],[[0,2],[0,0],[0,2],[0,2],\"<undefined>\",[0,2],[2,2]],0,2]"
        "#,
    );
    assert_true(
        r#"
            var pattern = "(?<before>^)(?<whole>(?<iteration>(?<e>)|(?<c>[\\q{|a|ab}])){2,3})(?<after>$)",
                input = "aba",
                r = new RegExp(pattern, "dgv"),
                ok = r.source === pattern && r.global && r.hasIndices && r.unicodeSets === true && r
                .unicode === false;
            r.lastIndex = 0;
            var m = r.exec(input);
            if (m !== null) {
                var names = ["before", "whole", "iteration", "e", "c", "after"];
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
                "[[\"aba\",\"\",\"aba\",\"a\",\"<undefined>\",\"a\",\"\"],[[0,3],[0,0],[0,3],[2,3],\"<undefined>\",[2,3],[3,3]],0,3]"
        "#,
    );
    assert_true(
        r#"
            var pattern = "(?<before>^)(?<whole>(?<iteration>(?<e>)|(?<c>[\\q{|a|ab}])){2,3}?)(?<after>$)",
                input = "aba",
                r = new RegExp(pattern, "dgv"),
                ok = r.source === pattern && r.global && r.hasIndices && r.unicodeSets === true && r
                .unicode === false;
            r.lastIndex = 0;
            var m = r.exec(input);
            if (m !== null) {
                var names = ["before", "whole", "iteration", "e", "c", "after"];
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
                "[[\"aba\",\"\",\"aba\",\"a\",\"<undefined>\",\"a\",\"\"],[[0,3],[0,0],[0,3],[2,3],\"<undefined>\",[2,3],[3,3]],0,3]"
        "#,
    );
    assert_true(
        r#"
            var pattern = "(?<before>^)(?<whole>(?<iteration>(?<e>)|(?<c>[\\q{|a|ab}])){2,})(?<after>$)",
                input = "aaaa",
                r = new RegExp(pattern, "dgv"),
                ok = r.source === pattern && r.global && r.hasIndices && r.unicodeSets === true && r
                .unicode === false;
            r.lastIndex = 0;
            var m = r.exec(input);
            if (m !== null) {
                var names = ["before", "whole", "iteration", "e", "c", "after"];
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
                "[[\"aaaa\",\"\",\"aaaa\",\"a\",\"<undefined>\",\"a\",\"\"],[[0,4],[0,0],[0,4],[3,4],\"<undefined>\",[3,4],[4,4]],0,4]"
        "#,
    );

    assert_true(
        r#"
            var pattern = "(?<before>^)(?<whole>(?<iteration>(?<c>[\\q{a|ab}])|(?<e>)){2})(?<after>$)",
                input = "a",
                r = new RegExp(pattern, "dgv"),
                ok = r.source === pattern && r.global && r.hasIndices && r.unicodeSets === true && r
                .unicode === false;
            r.lastIndex = 0;
            var m = r.exec(input);
            if (m !== null) {
                var names = ["before", "whole", "iteration", "c", "e", "after"];
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
                "[[\"a\",\"\",\"a\",\"\",\"<undefined>\",\"\",\"\"],[[0,1],[0,0],[0,1],[1,1],\"<undefined>\",[1,1],[1,1]],0,1]"
        "#,
    );
    assert_true(
        r#"
            var pattern = "(?<before>^)(?<whole>(?<iteration>(?<c>[\\q{a|ab}])|(?<e>)){2})(?<after>$)",
                input = "ab",
                r = new RegExp(pattern, "dgv"),
                ok = r.source === pattern && r.global && r.hasIndices && r.unicodeSets === true && r
                .unicode === false;
            r.lastIndex = 0;
            var m = r.exec(input);
            if (m !== null) {
                var names = ["before", "whole", "iteration", "c", "e", "after"];
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
                "[[\"ab\",\"\",\"ab\",\"\",\"<undefined>\",\"\",\"\"],[[0,2],[0,0],[0,2],[2,2],\"<undefined>\",[2,2],[2,2]],0,2]"
        "#,
    );
    assert_true(
        r#"
            var pattern = "(?<before>^)(?<whole>(?<iteration>(?<c>[\\q{a|ab}])|(?<e>)){2,3})(?<after>$)",
                input = "aba",
                r = new RegExp(pattern, "dgv"),
                ok = r.source === pattern && r.global && r.hasIndices && r.unicodeSets === true && r
                .unicode === false;
            r.lastIndex = 0;
            var m = r.exec(input);
            if (m !== null) {
                var names = ["before", "whole", "iteration", "c", "e", "after"];
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
                "[[\"aba\",\"\",\"aba\",\"a\",\"a\",\"<undefined>\",\"\"],[[0,3],[0,0],[0,3],[2,3],[2,3],\"<undefined>\",[3,3]],0,3]"
        "#,
    );
    assert_true(
        r#"
            var pattern = "(?<before>^)(?<whole>(?<iteration>(?<e>)|(?<c>[\\q{a|ab}])){2})(?<after>$)",
                input = "a",
                r = new RegExp(pattern, "dgv"),
                ok = r.source === pattern && r.global && r.hasIndices && r.unicodeSets === true && r
                .unicode === false;
            r.lastIndex = 0;
            var m = r.exec(input);
            if (m !== null) {
                var names = ["before", "whole", "iteration", "e", "c", "after"];
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
                "[[\"a\",\"\",\"a\",\"a\",\"<undefined>\",\"a\",\"\"],[[0,1],[0,0],[0,1],[0,1],\"<undefined>\",[0,1],[1,1]],0,1]"
        "#,
    );
    assert_true(
        r#"
            var pattern = "(?<before>^)(?<whole>(?<iteration>(?<e>)|(?<c>[\\q{a|ab}])){2})(?<after>$)",
                input = "ab",
                r = new RegExp(pattern, "dgv"),
                ok = r.source === pattern && r.global && r.hasIndices && r.unicodeSets === true && r
                .unicode === false;
            r.lastIndex = 0;
            var m = r.exec(input);
            if (m !== null) {
                var names = ["before", "whole", "iteration", "e", "c", "after"];
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
                "[[\"ab\",\"\",\"ab\",\"ab\",\"<undefined>\",\"ab\",\"\"],[[0,2],[0,0],[0,2],[0,2],\"<undefined>\",[0,2],[2,2]],0,2]"
        "#,
    );
    assert_true(
        r#"
            var pattern = "(?<before>^)(?<whole>(?<iteration>(?<e>)|(?<c>[\\q{a|ab}])){2,3})(?<after>$)",
                input = "aba",
                r = new RegExp(pattern, "dgv"),
                ok = r.source === pattern && r.global && r.hasIndices && r.unicodeSets === true && r
                .unicode === false;
            r.lastIndex = 0;
            var m = r.exec(input);
            if (m !== null) {
                var names = ["before", "whole", "iteration", "e", "c", "after"];
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
                "[[\"aba\",\"\",\"aba\",\"a\",\"<undefined>\",\"a\",\"\"],[[0,3],[0,0],[0,3],[2,3],\"<undefined>\",[2,3],[3,3]],0,3]"
        "#,
    );
    assert_true(
        r#"
            var pattern =
                "(?<before>^)(?<whole>(?<iteration>(?<e>)|(?<c>[\\q{|\ud83d\ude00|\ud83d\ude00a|aX}])){2})(?<after>$)",
                input = "\ud83d\ude00aX",
                r = new RegExp(pattern, "dgvy"),
                ok = r.source === pattern && r.global && r.hasIndices && r.unicodeSets === true && r
                .unicode === false;
            r.lastIndex = 1;
            var m = r.exec(input);
            if (m !== null) {
                var names = ["before", "whole", "iteration", "e", "c", "after"];
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
                "[[\"\ud83d\ude00aX\",\"\",\"\ud83d\ude00aX\",\"aX\",\"<undefined>\",\"aX\",\"\"],[[0,4],[0,0],[0,4],[2,4],\"<undefined>\",[2,4],[4,4]],0,4]"
        "#,
    );
    assert_true(
        r#"
            var pattern = "(?<before>^)(?<whole>(?<iteration>(?<e>)|(?<c>[\\q{|S|sa|aX}])){2})(?<after>$)",
                input = "\u017faX",
                r = new RegExp(pattern, "dgvi"),
                ok = r.source === pattern && r.global && r.hasIndices && r.unicodeSets === true && r
                .unicode === false;
            r.lastIndex = 0;
            var m = r.exec(input);
            if (m !== null) {
                var names = ["before", "whole", "iteration", "e", "c", "after"];
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
                "[[\"\u017faX\",\"\",\"\u017faX\",\"aX\",\"<undefined>\",\"aX\",\"\"],[[0,3],[0,0],[0,3],[1,3],\"<undefined>\",[1,3],[3,3]],0,3]"
        "#,
    );
    assert_true(
        r#"
            var pattern =
                "(?<before>^)(?<whole>(?<iteration>(?<c>[\\p{Any}\\q{|ab}])|(?<e>)){2})(?<after>$)",
                input = "a",
                r = new RegExp(pattern, "dgv"),
                ok = r.source === pattern && r.global && r.hasIndices && r.unicodeSets === true && r
                .unicode === false;
            r.lastIndex = 0;
            var m = r.exec(input);
            if (m !== null) {
                var names = ["before", "whole", "iteration", "c", "e", "after"];
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
                "[[\"a\",\"\",\"a\",\"\",\"\",\"<undefined>\",\"\"],[[0,1],[0,0],[0,1],[1,1],[1,1],\"<undefined>\",[1,1]],0,1]"
        "#,
    );
    assert_true(
        r#"
            var pattern =
                "(?<before>^)(?<whole>(?<iteration>(?<e>)|(?<c>[\\p{Any}\\q{|ab}])){2})(?<after>$)",
                input = "a",
                r = new RegExp(pattern, "dgv"),
                ok = r.source === pattern && r.global && r.hasIndices && r.unicodeSets === true && r
                .unicode === false;
            r.lastIndex = 0;
            var m = r.exec(input);
            if (m !== null) {
                var names = ["before", "whole", "iteration", "e", "c", "after"];
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
                "[[\"a\",\"\",\"a\",\"a\",\"<undefined>\",\"a\",\"\"],[[0,1],[0,0],[0,1],[0,1],\"<undefined>\",[0,1],[1,1]],0,1]"
        "#,
    );
    assert_true(
        r#"
            var pattern =
                "(?<before>^)(?<whole>(?<iteration>(?<c>[\\q{ab}&&\\q{cd}])|(?<e>)){2})(?<after>$)",
                input = "",
                r = new RegExp(pattern, "dgv"),
                ok = r.source === pattern && r.global && r.hasIndices && r.unicodeSets === true && r
                .unicode === false;
            r.lastIndex = 0;
            var m = r.exec(input);
            if (m !== null) {
                var names = ["before", "whole", "iteration", "c", "e", "after"];
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
                "[[\"\",\"\",\"\",\"\",\"<undefined>\",\"\",\"\"],[[0,0],[0,0],[0,0],[0,0],\"<undefined>\",[0,0],[0,0]],0,0]"
        "#,
    );
    assert_true(
        r#"
            var pattern = "(?<before>^)(?<whole>(?<iteration>(?<c>[\\q{}])){2})(?<after>$)",
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
            ]) === "[[\"\",\"\",\"\",\"\",\"\",\"\"],[[0,0],[0,0],[0,0],[0,0],[0,0],[0,0]],0,0]"
        "#,
    );
    assert_true(
        r#"
            var pattern = "(?<before>\\b)(?<whole>(?<iteration>(?<c>[\\q{}])){2}?)(?<after>\\b)",
                input = "a",
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
            ]) === "[[\"\",\"\",\"\",\"\",\"\",\"\"],[[0,0],[0,0],[0,0],[0,0],[0,0],[0,0]],0,0]"
        "#,
    );
    assert_true(
        r#"
            var pattern = "(?<before>^)(?<whole>(?<iteration>(?<e>)|(?<c>[\\q{}])){2})(?<after>$)",
                input = "",
                r = new RegExp(pattern, "dgv"),
                ok = r.source === pattern && r.global && r.hasIndices && r.unicodeSets === true && r
                .unicode === false;
            r.lastIndex = 0;
            var m = r.exec(input);
            if (m !== null) {
                var names = ["before", "whole", "iteration", "e", "c", "after"];
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
                "[[\"\",\"\",\"\",\"\",\"\",\"<undefined>\",\"\"],[[0,0],[0,0],[0,0],[0,0],[0,0],\"<undefined>\",[0,0]],0,0]"
        "#,
    );
    assert_true(
        r#"
            var pattern = "(?<before>^)(?<whole>(?<c>[\\q{|a|ab}]{2}))(?<after>$)",
                input = "",
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
            ]) === "[[\"\",\"\",\"\",\"\",\"\"],[[0,0],[0,0],[0,0],[0,0],[0,0]],0,0]"
        "#,
    );

    let mut realm = Realm::default();
    realm.eval(r"var r=/^(?<whole>(?<iteration>(?<e>)|(?<c>[\q{a|ab}])){2,})(?<after>)$/dvg,clone=new RegExp(r),m=r.exec('ababa')").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval(r#"
        var n = clone.exec('');
        m.groups.whole === 'ababa' && m.groups.c === 'a' && m.groups.e === undefined && m.indices
            .groups.c === m.indices[4] && m.indices[4][0] === 4 && m.indices[4][1] === 5 && n[0] ===
            '' && n.groups.iteration === '' && n.groups.e === '' && n.groups.c === undefined && n
            .indices.groups.e === n.indices[3] && n.indices.groups.c === undefined && clone.source ===
            r.source && clone.lastIndex === 0 && r.lastIndex === 5
    "#),Ok(Value::Boolean(true)));
}

#[test]
fn match_all_preserves_capture_metadata() {
    let source = format!(
        r#"
            (function() {{
                {DESCRIBE_MATCH}

                var r =
                    /(?<before>\b)(?<whole>(?<iteration>(?<e>)|(?<c>[\q{{a|ab}}])){{2,}})(?<after>\b)/dvg,
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
                /(?<before>\b)(?<whole>(?<iteration>(?<c>[\q{|a|ab}])|(?<e>)){2,})(?<after>\b)/vg,
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
          "<C:><C:>!<C:>X<C:>!",
          [
            [
              "ab",
              "",
              "ab",
              "",
              "",
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
              "",
              "",
              "",
              "",
              "",
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
              "",
              "",
              "",
              "",
              "",
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
                /(?<before>\b)(?<whole>(?<iteration>(?<e>)|(?<c>[\q{a|ab}])){2,})(?<after>\b)/dvg;
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
            "!",
            "",
            "",
            "",
            "",
            "<undefined>",
            "",
            "X",
            "",
            "",
            "",
            "",
            "<undefined>",
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

            var r = /^(()|([\q{a|ab}])){2,}?\b/vg;
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
                    /^(?<\u{{1D49C}}>)(?<whole>(?<iteration>(?<e>)|(?<c>[\q{{|😀|😀a|aX}}])){{2,3}})(?<after>)$/dvgy;
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
              "<undefined>",
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
              "<undefined>",
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
              "iteration",
              "e",
              "c",
              "after"
            ],
            "namedCaptures": [
              "",
              "😀aX",
              "aX",
              "<undefined>",
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
              "<undefined>",
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

                var r =
                    /(?<before>\b)(?<whole>(?<iteration>(?<e>)|(?<c>[\q{{|S|sa|aX}}])){{2,}})(?<after>\b)/dvi,
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
              "<undefined>",
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
              "<undefined>",
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
              "iteration",
              "e",
              "c",
              "after"
            ],
            "namedCaptures": [
              "",
              "ſaX",
              "aX",
              "<undefined>",
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
              "<undefined>",
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

                var z = /^(?<whole>(?<iteration>(?<c>[\q{{|a|ab}}])|(?<e>))*?)\b/dv.exec('x'),
                    e = /^(?<whole>(?<iteration>(?<e>)|(?<c>[\q{{|a|ab}}])){{2,}}?)\b/dv.exec('x'),
                    c = /^(?<whole>(?<iteration>(?<c>[\q{{|a|ab}}])|(?<e>)){{2,}}?)\b/dv.exec('x');
                return JSON.stringify([describeMatch(z), describeMatch(e), describeMatch(c)], null, 2);
            }})()
        "#
    );
    let report = eval_string(&mut Realm::default(), &source);
    insta::assert_snapshot!(report);
}

#[test]
fn empty_input_preserves_capture_metadata() {
    let source = format!(
        r#"
            (function() {{
                {DESCRIBE_MATCH}

                var e = /^(?<whole>(?<iteration>(?<e>)|(?<c>[\q{{ab}}&&\q{{cd}}])){{2,}})$/dv.exec(''),
                    c = /^(?<whole>(?<c>[\q{{}}]){{2,}})$/dv.exec('');
                return JSON.stringify([describeMatch(e), describeMatch(c)], null, 2);
            }})()
        "#
    );
    let report = eval_string(&mut Realm::default(), &source);
    insta::assert_snapshot!(report);
}

#[test]
fn asserted_two_required_nullable_string_class_proofs_keep_other_minima_conditional_predicates_and_unknown_leaves_closed()
 {
    assert_true(
        r#"var marker=0,ok=false;try{ok=(function(){var pattern="^([\\q{|a|ab}]){3}$",rows=[],metadata=true;
for(var multi of [false,true]){var r=new RegExp(pattern,'dvg'+(multi?'m':''));metadata=metadata&&r.source===pattern;
for(var input of ['aaaaaa','ababa','\nab\n','']){r.lastIndex=0;var m=r.exec(input),result=null;if(m!==null)result={values:Array.from(m,v=>v===undefined?'<undefined>':v),indices:Array.from(m.indices,v=>v===undefined?'<undefined>':v),index:m.index};rows.push([multi,input,result,r.lastIndex]);}}
return metadata&&JSON.stringify(rows)==="[[false,\"aaaaaa\",null,0],[false,\"ababa\",{\"values\":[\"ababa\",\"a\"],\"indices\":[[0,5],[4,5]],\"index\":0},5],[false,\"\\nab\\n\",null,0],[false,\"\",{\"values\":[\"\",\"\"],\"indices\":[[0,0],[0,0]],\"index\":0},0],[true,\"aaaaaa\",null,0],[true,\"ababa\",{\"values\":[\"ababa\",\"a\"],\"indices\":[[0,5],[4,5]],\"index\":0},5],[true,\"\\nab\\n\",{\"values\":[\"\",\"\"],\"indices\":[[0,0],[0,0]],\"index\":0},0],[true,\"\",{\"values\":[\"\",\"\"],\"indices\":[[0,0],[0,0]],\"index\":0},0]]"})();}finally{marker=3}ok&&marker===3"#,
    );
    assert_true(
        r#"var marker=0,ok=false;try{ok=(function(){var pattern="^(()|([\\q{a|ab}])){3}$",rows=[],metadata=true;
for(var multi of [false,true]){var r=new RegExp(pattern,'dvg'+(multi?'m':''));metadata=metadata&&r.source===pattern;
for(var input of ['aaaaaa','ababa','\nab\n','']){r.lastIndex=0;var m=r.exec(input),result=null;if(m!==null)result={values:Array.from(m,v=>v===undefined?'<undefined>':v),indices:Array.from(m.indices,v=>v===undefined?'<undefined>':v),index:m.index};rows.push([multi,input,result,r.lastIndex]);}}
return metadata&&JSON.stringify(rows)==="[[false,\"aaaaaa\",null,0],[false,\"ababa\",{\"values\":[\"ababa\",\"a\",\"<undefined>\",\"a\"],\"indices\":[[0,5],[4,5],\"<undefined>\",[4,5]],\"index\":0},5],[false,\"\\nab\\n\",null,0],[false,\"\",{\"values\":[\"\",\"\",\"\",\"<undefined>\"],\"indices\":[[0,0],[0,0],[0,0],\"<undefined>\"],\"index\":0},0],[true,\"aaaaaa\",null,0],[true,\"ababa\",{\"values\":[\"ababa\",\"a\",\"<undefined>\",\"a\"],\"indices\":[[0,5],[4,5],\"<undefined>\",[4,5]],\"index\":0},5],[true,\"\\nab\\n\",{\"values\":[\"\",\"\",\"\",\"<undefined>\"],\"indices\":[[0,0],[0,0],[0,0],\"<undefined>\"],\"index\":0},0],[true,\"\",{\"values\":[\"\",\"\",\"\",\"<undefined>\"],\"indices\":[[0,0],[0,0],[0,0],\"<undefined>\"],\"index\":0},0]]"})();}finally{marker=3}ok&&marker===3"#,
    );
    assert_true(
        r####"var pattern="(^)*([\\q{|a|ab}]){2}$",input="aba",r=new RegExp(pattern,"dv"),m=r.exec(input);r.source===pattern&&r.flags==="dv"&&r.hasIndices&&r.unicodeSets&&m!==null&&m.input===input&&m.groups===undefined&&m.indices.groups===undefined&&Object.hasOwn(m,"groups")&&Object.hasOwn(m.indices,"groups")&&JSON.stringify(m===null?null:[Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex])==="[[\"aba\",\"<undefined>\",\"a\"],[[0,3],\"<undefined>\",[2,3]],0,0]""####,
    );
    assert_true(
        r####"var pattern="(^|)([\\q{|a|ab}]){2}$",input="aba",r=new RegExp(pattern,"dv"),m=r.exec(input);r.source===pattern&&r.flags==="dv"&&r.hasIndices&&r.unicodeSets&&(m===null||(m.input===input&&m.groups===undefined&&m.indices.groups===undefined&&Object.hasOwn(m,"groups")&&Object.hasOwn(m.indices,"groups")))&&JSON.stringify(m===null?null:[Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex])==="[[\"aba\",\"\",\"a\"],[[0,3],[0,0],[2,3]],0,0]""####,
    );
    assert_true(
        r####"var pattern="^([\\q{|a|ab}]){2}(\\b|)",input="aba",r=new RegExp(pattern,"dv"),m=r.exec(input);r.source===pattern&&r.flags==="dv"&&r.hasIndices&&r.unicodeSets&&(m===null||(m.input===input&&m.groups===undefined&&m.indices.groups===undefined&&Object.hasOwn(m,"groups")&&Object.hasOwn(m.indices,"groups")))&&JSON.stringify(m===null?null:[Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex])==="[[\"aba\",\"a\",\"\"],[[0,3],[2,3],[3,3]],0,0]""####,
    );
    for pattern in [
        r"^([[\q{|a|ab}]&&\p{RGI_Emoji}]){3}$",
        r"^(()|([[\q{a|ab}]&&\p{RGI_Emoji}])){3}$",
        r"(^([\q{|a|ab}])){2}$",
        r"^([\q{|a|ab}]){2}\1",
        r"^([\q{|a|ab}]){2}a$",
        r"^(([\q{|a|ab}]){2}){2}$",
        r"^([\q{|a|ab}]){2}[\q{ab}]$",
        r"^(()|([\q{ab}&&\p{Any}&&\p{RGI_Emoji}])){2}$",
    ] {
        let mut realm = Realm::default();
        crate::common::regexp_oracle::assert_matches_reference(
            &mut realm,
            &format!(
                "var marker=0;try{{new RegExp({:?},'v').test('aba')}}catch{{marker=1}}finally{{marker=2}};marker",
                JsString::from(pattern)
            ),
        );
    }
}
