//! Required empty iterations preserve class-versus-empty capture identity.
use crate::common::{assert_true, eval_string};

use crate::common::regexp_matches::DESCRIBE_MATCH;
use spite_core::JsString;
use spite_runtime::{Realm, Value};
#[test]
fn unasserted_zero_count_string_class_native_tables_consumers_and_skipped_capture_identity() {
    assert_true(
        r#"
            var pattern =
                "(?<before>^){0}(?<z>(?<z1>^)|(?<z2>\\b\\B)){0}(?<whole>(?<iteration>(?<c>[\\q{|a|ab}]))*)(?<after>$){0}",
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
                "[[\"\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],[[0,0],\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",[0,0],\"<undefined>\",\"<undefined>\",\"<undefined>\"],0,0]"
        "#,
    );
    assert_true(
        r#"
            var pattern =
                "(?<before>^){0}(?<z>(?<z1>^)|(?<z2>\\b\\B)){0}(?<whole>(?<iteration>(?<c>[\\q{a|ab}])|(?<e>))*?)(?<after>$){0}",
                input = "aba",
                r = new RegExp(pattern, "dgv"),
                ok = r.source === pattern && r.global && r.hasIndices && r.unicodeSets === true && r
                .unicode === false;
            r.lastIndex = 0;
            var m = r.exec(input);
            if (m !== null) {
                var names = ["before", "z", "z1", "z2", "whole", "iteration", "c", "e", "after"];
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
                "[[\"\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],[[0,0],\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",[0,0],\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],0,0]"
        "#,
    );
    assert_true(
        r#"
            var pattern =
                "(?<before>^){0}(?<z>(?<z1>^)|(?<z2>\\b\\B)){0}(?<whole>(?<iteration>(?<e>)|(?<c>[\\p{Any}\\q{|ab}]))+)(?<after>$){0}",
                input = "aaaa",
                r = new RegExp(pattern, "dgv"),
                ok = r.source === pattern && r.global && r.hasIndices && r.unicodeSets === true && r
                .unicode === false;
            r.lastIndex = 0;
            var m = r.exec(input);
            if (m !== null) {
                var names = ["before", "z", "z1", "z2", "whole", "iteration", "e", "c", "after"];
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
                "[[\"aaaa\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"aaaa\",\"a\",\"<undefined>\",\"a\",\"<undefined>\"],[[0,4],\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",[0,4],[3,4],\"<undefined>\",[3,4],\"<undefined>\"],0,4]"
        "#,
    );
    assert_true(
        r#"
            var pattern =
                "(?<before>^){0}(?<whole>(?<iteration>(?<c>[a-z\\q{|S|sa|aX}]))??)(?<z>(?<z1>\\B)*|(?<z2>\\b)+){0}?(?<after>$){0}",
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
                "[[\"\",\"<undefined>\",\"\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],[[0,0],\"<undefined>\",[0,0],\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],0,0]"
        "#,
    );
    assert_true(
        r#"
            var pattern =
                "(?<before>\\b){0}(?<whole>(?<iteration>(?<c>[\\q{|a|ab}])|(?<e>)){0})(?<z>(?<z1>\\B)*|(?<z2>\\b)+){0}?(?<after>\\b){0}",
                input = "\u017faX",
                r = new RegExp(pattern, "dgv"),
                ok = r.source === pattern && r.global && r.hasIndices && r.unicodeSets === true && r
                .unicode === false;
            r.lastIndex = 0;
            var m = r.exec(input);
            if (m !== null) {
                var names = ["before", "whole", "iteration", "c", "e", "z", "z1", "z2", "after"];
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
                "[[\"\",\"<undefined>\",\"\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],[[0,0],\"<undefined>\",[0,0],\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],0,0]"
        "#,
    );
    assert_true(
        r#"
            var pattern =
                "(?<before>\\b){0}(?<whole>(?<iteration>(?<e>)|(?<c>[\\q{a|ab}])){2,3})(?<z>(?<z1>\\B)*|(?<z2>\\b)+){0}?(?<after>\\b){0}",
                input = "\n\nab\n",
                r = new RegExp(pattern, "dgvm"),
                ok = r.source === pattern && r.global && r.hasIndices && r.unicodeSets === true && r
                .unicode === false;
            r.lastIndex = 0;
            var m = r.exec(input);
            if (m !== null) {
                var names = ["before", "whole", "iteration", "e", "c", "z", "z1", "z2", "after"];
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
                "[[\"\",\"<undefined>\",\"\",\"\",\"\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],[[0,0],\"<undefined>\",[0,0],[0,0],[0,0],\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],0,0]"
        "#,
    );
    assert_true(
        r#"
            var pattern =
                "(?<before>\\b){0}(?<z>(?<z1>\\b){1,}|(?<z2>\\B)?){0}(?<whole>(?<iteration>(?<c>[\\p{Any}\\q{|ab}])){2,3}?)(?<s>(?<s1>$)|(?<s2>^)){000,000}?(?<after>\\b){0}",
                input = "a",
                r = new RegExp(pattern, "dgvy"),
                ok = r.source === pattern && r.global && r.hasIndices && r.unicodeSets === true && r
                .unicode === false;
            r.lastIndex = 0;
            var m = r.exec(input);
            if (m !== null) {
                var names = ["before", "z", "z1", "z2", "whole", "iteration", "c", "s", "s1", "s2",
                    "after"];
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
                "[[\"a\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"a\",\"\",\"\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],[[0,1],\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",[0,1],[1,1],[1,1],\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],0,1]"
        "#,
    );
    assert_true(
        r#"
            var pattern =
                "(?<before>\\b){0}(?<z>(?<z1>\\b){1,}|(?<z2>\\B)?){0}(?<whole>(?<iteration>(?<c>[a-z\\q{|S|sa|aX}])|(?<e>)){3,4})(?<s>(?<s1>$)|(?<s2>^)){000,000}?(?<after>\\b){0}",
                input = "!ab!",
                r = new RegExp(pattern, "dgvi"),
                ok = r.source === pattern && r.global && r.hasIndices && r.unicodeSets === true && r
                .unicode === false;
            r.lastIndex = 0;
            var m = r.exec(input);
            if (m !== null) {
                var names = ["before", "z", "z1", "z2", "whole", "iteration", "c", "e", "s", "s1", "s2",
                    "after"
                ];
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
                "[[\"\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"\",\"\",\"\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],[[0,0],\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",[0,0],[0,0],[0,0],\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],0,0]"
        "#,
    );
    assert_true(
        r#"
            var pattern =
                "(?<before>\\B){0}(?<z>(?<z1>\\b){1,}|(?<z2>\\B)?){0}(?<whole>(?<iteration>(?<e>)|(?<c>[\\q{|a|ab}])){3,4}?)(?<s>(?<s1>$)|(?<s2>^)){000,000}?(?<after>\\B){0}",
                input = "",
                r = new RegExp(pattern, "dgv"),
                ok = r.source === pattern && r.global && r.hasIndices && r.unicodeSets === true && r
                .unicode === false;
            r.lastIndex = 0;
            var m = r.exec(input);
            if (m !== null) {
                var names = ["before", "z", "z1", "z2", "whole", "iteration", "e", "c", "s", "s1", "s2",
                    "after"
                ];
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
                "[[\"\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"\",\"\",\"\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],[[0,0],\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",[0,0],[0,0],[0,0],\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],0,0]"
        "#,
    );

    assert_true(
        r#"
            var pattern =
                "(?<before>^){0}(?<z>(?<z1>^)|(?<z2>\\b\\B)){0}(?<whole>(?<iteration>(?<c>[\\q{|a|ab}]))+)(?<after>\\b){0}",
                input = "\u017faX",
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
                "[[\"\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"\",\"\",\"\",\"<undefined>\"],[[0,0],\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",[0,0],[0,0],[0,0],\"<undefined>\"],0,0]"
        "#,
    );
    assert_true(
        r#"
            var pattern =
                "(?<before>^){0}(?<z>(?<z1>^)|(?<z2>\\b\\B)){0}(?<whole>(?<iteration>(?<c>[\\q{a|ab}])|(?<e>))??)(?<after>\\b){0}",
                input = "\n\nab\n",
                r = new RegExp(pattern, "dgv"),
                ok = r.source === pattern && r.global && r.hasIndices && r.unicodeSets === true && r
                .unicode === false;
            r.lastIndex = 0;
            var m = r.exec(input);
            if (m !== null) {
                var names = ["before", "z", "z1", "z2", "whole", "iteration", "c", "e", "after"];
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
                "[[\"\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],[[0,0],\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",[0,0],\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],0,0]"
        "#,
    );
    assert_true(
        r#"
            var pattern =
                "(?<before>^){0}(?<z>(?<z1>^)|(?<z2>\\b\\B)){0}(?<whole>(?<iteration>(?<e>)|(?<c>[\\p{Any}\\q{|ab}])){0})(?<after>\\b){0}",
                input = "a",
                r = new RegExp(pattern, "dgv"),
                ok = r.source === pattern && r.global && r.hasIndices && r.unicodeSets === true && r
                .unicode === false;
            r.lastIndex = 0;
            var m = r.exec(input);
            if (m !== null) {
                var names = ["before", "z", "z1", "z2", "whole", "iteration", "e", "c", "after"];
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
                "[[\"\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],[[0,0],\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",[0,0],\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],0,0]"
        "#,
    );
    assert_true(
        r#"
            var pattern =
                "(?<before>^){0}(?<whole>(?<iteration>(?<c>[a-z\\q{|S|sa|aX}])){2,3})(?<z>(?<z1>\\B)*|(?<z2>\\b)+){0}?(?<after>\\b){0}",
                input = "!ab!",
                r = new RegExp(pattern, "dgvyim"),
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
                "[[\"\",\"<undefined>\",\"\",\"\",\"\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],[[0,0],\"<undefined>\",[0,0],[0,0],[0,0],\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],0,0]"
        "#,
    );
    assert_true(
        r#"
            var pattern =
                "(?<before>^){0}(?<whole>(?<iteration>(?<c>[\\q{|a|ab}])|(?<e>)){2,3}?)(?<z>(?<z1>\\B)*|(?<z2>\\b)+){0}?(?<after>$){0}",
                input = "",
                r = new RegExp(pattern, "dgv"),
                ok = r.source === pattern && r.global && r.hasIndices && r.unicodeSets === true && r
                .unicode === false;
            r.lastIndex = 0;
            var m = r.exec(input);
            if (m !== null) {
                var names = ["before", "whole", "iteration", "c", "e", "z", "z1", "z2", "after"];
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
                "[[\"\",\"<undefined>\",\"\",\"\",\"\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],[[0,0],\"<undefined>\",[0,0],[0,0],[0,0],\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],0,0]"
        "#,
    );
    assert_true(
        r#"
            var pattern =
                "(?<before>^){0}(?<whole>(?<iteration>(?<e>)|(?<c>[\\q{a|ab}])){3,4})(?<z>(?<z1>\\B)*|(?<z2>\\b)+){0}?(?<after>$){0}",
                input = "aba",
                r = new RegExp(pattern, "dgv"),
                ok = r.source === pattern && r.global && r.hasIndices && r.unicodeSets === true && r
                .unicode === false;
            r.lastIndex = 0;
            var m = r.exec(input);
            if (m !== null) {
                var names = ["before", "whole", "iteration", "e", "c", "z", "z1", "z2", "after"];
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
                "[[\"ab\",\"<undefined>\",\"ab\",\"ab\",\"<undefined>\",\"ab\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],[[0,2],\"<undefined>\",[0,2],[0,2],\"<undefined>\",[0,2],\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],0,2]"
        "#,
    );
    assert_true(
        r#"
            var pattern =
                "(?<before>^){0}(?<z>(?<z1>\\b){1,}|(?<z2>\\B)?){0}(?<whole>(?<iteration>(?<c>[\\p{Any}\\q{|ab}])){3,4}?)(?<s>(?<s1>$)|(?<s2>^)){000,000}?(?<after>$){0}",
                input = "aaaa",
                r = new RegExp(pattern, "dgvy"),
                ok = r.source === pattern && r.global && r.hasIndices && r.unicodeSets === true && r
                .unicode === false;
            r.lastIndex = 0;
            var m = r.exec(input);
            if (m !== null) {
                var names = ["before", "z", "z1", "z2", "whole", "iteration", "c", "s", "s1", "s2",
                    "after"];
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
                "[[\"aaa\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"aaa\",\"a\",\"a\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],[[0,3],\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",[0,3],[2,3],[2,3],\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],0,3]"
        "#,
    );

    assert_true(
        r#"
            var pattern =
                "(?<before>\\b){0}(?<z>(?<z1>\\b){1,}|(?<z2>\\B)?){0}(?<whole>(?<iteration>(?<e>)|(?<c>[\\q{|a|ab}]))*)(?<s>(?<s1>$)|(?<s2>^)){000,000}?(?<after>\\b){0}",
                input = "\u017faX",
                r = new RegExp(pattern, "dgvm"),
                ok = r.source === pattern && r.global && r.hasIndices && r.unicodeSets === true && r
                .unicode === false;
            r.lastIndex = 0;
            var m = r.exec(input);
            if (m !== null) {
                var names = ["before", "z", "z1", "z2", "whole", "iteration", "e", "c", "s", "s1", "s2",
                    "after"
                ];
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
                "[[\"\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],[[0,0],\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",[0,0],\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],0,0]"
        "#,
    );

    assert_true(
        r#"
            var pattern =
                "(?<before>\\B){0}(?<z>(?<z1>^)|(?<z2>\\b\\B)){0}(?<whole>(?<iteration>(?<c>[\\q{|a|ab}])){0})(?<after>\\B){0}",
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
                "[[\"\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],[[0,0],\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",[0,0],\"<undefined>\",\"<undefined>\",\"<undefined>\"],0,0]"
        "#,
    );
    assert_true(
        r#"
            var pattern =
                "(?<before>\\B){0}(?<z>(?<z1>^)|(?<z2>\\b\\B)){0}(?<whole>(?<iteration>(?<c>[\\q{a|ab}])|(?<e>)){2,3})(?<after>\\B){0}",
                input = "aba",
                r = new RegExp(pattern, "dgvm"),
                ok = r.source === pattern && r.global && r.hasIndices && r.unicodeSets === true && r
                .unicode === false;
            r.lastIndex = 0;
            var m = r.exec(input);
            if (m !== null) {
                var names = ["before", "z", "z1", "z2", "whole", "iteration", "c", "e", "after"];
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
                "[[\"aba\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"aba\",\"a\",\"a\",\"<undefined>\",\"<undefined>\"],[[0,3],\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",[0,3],[2,3],[2,3],\"<undefined>\",\"<undefined>\"],0,3]"
        "#,
    );
    assert_true(
        r#"
            var pattern =
                "(?<before>\\B){0}(?<z>(?<z1>^)|(?<z2>\\b\\B)){0}(?<whole>(?<iteration>(?<e>)|(?<c>[\\p{Any}\\q{|ab}])){2,3}?)(?<after>\\B){0}",
                input = "aaaa",
                r = new RegExp(pattern, "dgv"),
                ok = r.source === pattern && r.global && r.hasIndices && r.unicodeSets === true && r
                .unicode === false;
            r.lastIndex = 0;
            var m = r.exec(input);
            if (m !== null) {
                var names = ["before", "z", "z1", "z2", "whole", "iteration", "e", "c", "after"];
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
                "[[\"\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"\",\"\",\"\",\"<undefined>\",\"<undefined>\"],[[0,0],\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",[0,0],[0,0],[0,0],\"<undefined>\",\"<undefined>\"],0,0]"
        "#,
    );
    assert_true(
        r#"
            var pattern =
                "(?<before>\\B){0}(?<whole>(?<iteration>(?<c>[a-z\\q{|S|sa|aX}])){3,4})(?<z>(?<z1>\\B)*|(?<z2>\\b)+){0}?(?<after>\\B){0}",
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
                "[[\"\",\"<undefined>\",\"\",\"\",\"\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],[[0,0],\"<undefined>\",[0,0],[0,0],[0,0],\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],0,0]"
        "#,
    );
    assert_true(
        r#"
            var pattern =
                "(?<before>^){0}(?<whole>(?<iteration>(?<c>[\\q{|a|ab}])|(?<e>)){3,4}?)(?<z>(?<z1>\\B)*|(?<z2>\\b)+){0}?(?<after>\\b){0}",
                input = "\u017faX",
                r = new RegExp(pattern, "dgv"),
                ok = r.source === pattern && r.global && r.hasIndices && r.unicodeSets === true && r
                .unicode === false;
            r.lastIndex = 0;
            var m = r.exec(input);
            if (m !== null) {
                var names = ["before", "whole", "iteration", "c", "e", "z", "z1", "z2", "after"];
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
                "[[\"\",\"<undefined>\",\"\",\"\",\"\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],[[0,0],\"<undefined>\",[0,0],[0,0],[0,0],\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],0,0]"
        "#,
    );

    assert_true(
        r#"
            var pattern =
                "(?<before>^){0}(?<z>(?<z1>\\b){1,}|(?<z2>\\B)?){0}(?<whole>(?<iteration>(?<c>[\\p{Any}\\q{|ab}]))*)(?<s>(?<s1>$)|(?<s2>^)){000,000}?(?<after>\\b){0}",
                input = "a",
                r = new RegExp(pattern, "dgvym"),
                ok = r.source === pattern && r.global && r.hasIndices && r.unicodeSets === true && r
                .unicode === false;
            r.lastIndex = 0;
            var m = r.exec(input);
            if (m !== null) {
                var names = ["before", "z", "z1", "z2", "whole", "iteration", "c", "s", "s1", "s2",
                    "after"];
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
                "[[\"a\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"a\",\"a\",\"a\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],[[0,1],\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",[0,1],[0,1],[0,1],\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],0,1]"
        "#,
    );
    assert_true(
        r#"
            var pattern =
                "(?<before>^){0}(?<z>(?<z1>\\b){1,}|(?<z2>\\B)?){0}(?<whole>(?<iteration>(?<c>[a-z\\q{|S|sa|aX}])|(?<e>))*?)(?<s>(?<s1>$)|(?<s2>^)){000,000}?(?<after>\\b){0}",
                input = "!ab!",
                r = new RegExp(pattern, "dgvi"),
                ok = r.source === pattern && r.global && r.hasIndices && r.unicodeSets === true && r
                .unicode === false;
            r.lastIndex = 0;
            var m = r.exec(input);
            if (m !== null) {
                var names = ["before", "z", "z1", "z2", "whole", "iteration", "c", "e", "s", "s1", "s2",
                    "after"
                ];
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
                "[[\"\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],[[0,0],\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",[0,0],\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],0,0]"
        "#,
    );
    assert_true(
        r#"
            var pattern =
                "(?<before>^){0}(?<z>(?<z1>\\b){1,}|(?<z2>\\B)?){0}(?<whole>(?<iteration>(?<e>)|(?<c>[\\q{|a|ab}]))+)(?<s>(?<s1>$)|(?<s2>^)){000,000}?(?<after>$){0}",
                input = "",
                r = new RegExp(pattern, "dgv"),
                ok = r.source === pattern && r.global && r.hasIndices && r.unicodeSets === true && r
                .unicode === false;
            r.lastIndex = 0;
            var m = r.exec(input);
            if (m !== null) {
                var names = ["before", "z", "z1", "z2", "whole", "iteration", "e", "c", "s", "s1", "s2",
                    "after"
                ];
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
                "[[\"\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"\",\"\",\"\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],[[0,0],\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",[0,0],[0,0],[0,0],\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],0,0]"
        "#,
    );

    assert_true(
        r#"
            var pattern =
                "(?<before>\\b){0}(?<z>(?<z1>^)|(?<z2>\\b\\B)){0}(?<whole>(?<iteration>(?<c>[\\q{|a|ab}])){2,3}?)(?<after>\\b){0}",
                input = "\u017faX",
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
                "[[\"\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"\",\"\",\"\",\"<undefined>\"],[[0,0],\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",[0,0],[0,0],[0,0],\"<undefined>\"],0,0]"
        "#,
    );
    assert_true(
        r#"
            var pattern =
                "(?<before>\\b){0}(?<z>(?<z1>^)|(?<z2>\\b\\B)){0}(?<whole>(?<iteration>(?<c>[\\q{a|ab}])|(?<e>)){3,4})(?<after>\\b){0}",
                input = "\n\nab\n",
                r = new RegExp(pattern, "dgv"),
                ok = r.source === pattern && r.global && r.hasIndices && r.unicodeSets === true && r
                .unicode === false;
            r.lastIndex = 0;
            var m = r.exec(input);
            if (m !== null) {
                var names = ["before", "z", "z1", "z2", "whole", "iteration", "c", "e", "after"];
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
                "[[\"\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"\",\"\",\"<undefined>\",\"\",\"<undefined>\"],[[0,0],\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",[0,0],[0,0],\"<undefined>\",[0,0],\"<undefined>\"],0,0]"
        "#,
    );
    assert_true(
        r#"
            var pattern =
                "(?<before>\\b){0}(?<z>(?<z1>^)|(?<z2>\\b\\B)){0}(?<whole>(?<iteration>(?<e>)|(?<c>[\\p{Any}\\q{|ab}])){3,4}?)(?<after>\\b){0}",
                input = "a",
                r = new RegExp(pattern, "dgv"),
                ok = r.source === pattern && r.global && r.hasIndices && r.unicodeSets === true && r
                .unicode === false;
            r.lastIndex = 0;
            var m = r.exec(input);
            if (m !== null) {
                var names = ["before", "z", "z1", "z2", "whole", "iteration", "e", "c", "after"];
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
                "[[\"\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"\",\"\",\"\",\"<undefined>\",\"<undefined>\"],[[0,0],\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",[0,0],[0,0],[0,0],\"<undefined>\",\"<undefined>\"],0,0]"
        "#,
    );

    let mut realm = Realm::default();
    realm.eval(r"var r=/(?:(?:(?:\b){0})|(?:(?:\B){0})){0}(?:^){0}(?<whole>(?<iteration>(?<e>)|(?<c>[\p{Any}\q{a|ab}])){3,})(?<after>)(?:$){0}/dvg,clone=new RegExp(r),m=r.exec('ababa')").unwrap();
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
                    /(?:(?:\b)|(?:\B)){{0}}(?<before>(?:\b){{0}})(?<whole>(?<iteration>(?<e>)|(?<c>[\p{{Any}}\q{{a|ab}}])){{3,}})(?<after>(?:\b){{0}})/dvg,
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
                /(?:(?:\b)|(?:\B)){0}(?<before>(?:\b){0})(?<whole>(?<iteration>(?<c>[\p{Any}\q{|a|ab}])|(?<e>)){3,})(?<after>(?:\b){0})/vg,
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
          "<C:!><C:>",
          [
            [
              "ab!X!",
              "",
              "ab!X!",
              "!",
              "!",
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
              5,
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
                /(?:(?:\b)|(?:\B)){0}(?<before>(?:\b){0})(?<whole>(?<iteration>(?<e>)|(?<c>[\p{Any}\q{a|ab}])){3,})(?<after>(?:\b){0})/dvg;
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
            "",
            "",
            "!ab!X!",
            "!",
            "<undefined>",
            "!",
            "",
            ""
          ],
          0,
          9
        ]
        "#);
}

#[test]
fn failed_last_index_write_throws_type_error() {
    let source = r#"
        (function() {

            var r = /(?:(?:\b)|(?:\B)){0}(?:^){0}(()|([\p{Any}\q{a|ab}])){3,}?(?:\b){0}/vg;
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
                    /(?:(?:\b)|(?:\B)){{0}}(?:^){{0}}(?<\u{{1D49C}}>)(?<whole>(?<iteration>(?<e>)|(?<c>[\p{{Any}}\q{{|😀|😀a|aX}}])){{3,4}})(?<after>)(?:$){{0}}/dvgy;
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
              "😀a",
              "",
              "😀a",
              "😀a",
              "<undefined>",
              "😀a",
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
                0,
                3
              ],
              "<undefined>",
              [
                0,
                3
              ],
              [
                3,
                3
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
              "😀a",
              "😀a",
              "<undefined>",
              "😀a",
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
                0,
                3
              ],
              "<undefined>",
              [
                0,
                3
              ],
              [
                3,
                3
              ]
            ],
            "namedAliasesValid": true,
            "groupsPrototypeValid": true
          },
          3,
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
                    /(?:(?:\b)|(?:\B)){{0}}(?<before>(?:\b){{0}})(?<whole>(?<iteration>(?<e>)|(?<c>[\p{{Any}}\q{{|S|sa|aX}}])){{3,}})(?<after>(?:\b){{0}})/dvi,
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
              "ſaX!",
              "",
              "ſaX!",
              "!",
              "<undefined>",
              "!",
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
              "<undefined>",
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
              "ſaX!",
              "!",
              "<undefined>",
              "!",
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
              "<undefined>",
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

                var z =
                    /(?:(?:\b)|(?:\B)){{0}}(?:^){{0}}(?<whole>(?<iteration>(?<c>[\p{{Any}}\q{{|a|ab}}])|(?<e>))*?)(?:\b){{0}}/dv
                    .exec('x'),
                    e =
                    /(?:(?:\b)|(?:\B)){{0}}(?:^){{0}}(?<whole>(?<iteration>(?<e>)|(?<c>[\p{{Any}}\q{{|a|ab}}])){{3,}}?)(?:\b){{0}}/dv
                    .exec('x'),
                    c =
                    /(?:(?:\b)|(?:\B)){{0}}(?:^){{0}}(?<whole>(?<iteration>(?<c>[\p{{Any}}\q{{|a|ab}}])|(?<e>)){{3,}}?)(?:\b){{0}}/dv
                    .exec('x');
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

                var e =
                    /(?:(?:\b)|(?:\B)){{0}}(?:^){{0}}(?<whole>(?<iteration>(?<e>)|(?<c>[\q{{ab}}&&\q{{cd}}])){{3,}})(?:$){{0}}/dv
                    .exec(''),
                    c = /(?:(?:\b)|(?:\B)){{0}}(?:^){{0}}(?<whole>(?<c>[\p{{Any}}\q{{}}]){{3,}})(?:$){{0}}/dv.exec(
                        '');
                return JSON.stringify([describeMatch(e), describeMatch(c)], null, 2);
            }})()
        "#
    );
    let report = eval_string(&mut Realm::default(), &source);
    insta::assert_snapshot!(report);
}

#[test]
fn composition_results_match_reference() {
    assert_true(
        r####"var pattern="(?:(?:\\b)|(?:\\B)){0}((\\b)|(\\B))?([\\q{|a|ab}]){3}",input="aba",r=new RegExp(pattern,"dv"),m=r.exec(input);r.source===pattern&&r.flags==="dv"&&r.hasIndices&&r.unicodeSets&&m!==null&&m.input===input&&m.groups===undefined&&m.indices.groups===undefined&&Object.hasOwn(m,"groups")&&Object.hasOwn(m.indices,"groups")&&JSON.stringify(m===null?null:[Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex])==="[[\"aba\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"\"],[[0,3],\"<undefined>\",\"<undefined>\",\"<undefined>\",[3,3]],0,0]""####,
    );
    assert_true(
        r####"var pattern="(?:(?:\\b)|(?:\\B)){0}((\\b)|(\\B)){1}([\\q{|a|ab}]){3}",input="aba",r=new RegExp(pattern,"dv"),m=r.exec(input);r.source===pattern&&r.flags==="dv"&&r.hasIndices&&r.unicodeSets&&(m===null||(m.input===input&&m.groups===undefined&&m.indices.groups===undefined&&Object.hasOwn(m,"groups")&&Object.hasOwn(m.indices,"groups")))&&JSON.stringify(m===null?null:[Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex])==="[[\"aba\",\"\",\"\",\"<undefined>\",\"\"],[[0,3],[0,0],[0,0],\"<undefined>\",[3,3]],0,0]""####,
    );
    assert_true(
        r####"var pattern="(?:(?:\\b)|(?:\\B)){0}([\\q{|a|ab}]){3}(\\b|)",input="aba",r=new RegExp(pattern,"dv"),m=r.exec(input);r.source===pattern&&r.flags==="dv"&&r.hasIndices&&r.unicodeSets&&(m===null||(m.input===input&&m.groups===undefined&&m.indices.groups===undefined&&Object.hasOwn(m,"groups")&&Object.hasOwn(m.indices,"groups")))&&JSON.stringify(m===null?null:[Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex])==="[[\"aba\",\"\",\"\"],[[0,3],[3,3],[3,3]],0,0]""####,
    );
    for pattern in [
        r"(^([\q{|a|ab}])){3}",
        r"([\q{|a|ab}]){3}\1",
        r"(([\q{|a|ab}]){3}){2}",
        r"([\q{|a|ab}]){3}[\q{ab}]",
        r"(()|([\q{ab}&&\p{RGI_Emoji}])){3}",
    ] {
        let pattern = JsString::from(format!(r"(?:(?:\b)|(?:\B)){{0}}{pattern}").as_str());
        let mut realm = Realm::default();
        crate::common::regexp_oracle::assert_matches_reference(
            &mut realm,
            &format!(
                "var marker=0;try{{new RegExp({pattern:?},'v').test('aba')}}catch{{marker=1}}finally{{marker=2}};marker"
            ),
        );
    }
}
