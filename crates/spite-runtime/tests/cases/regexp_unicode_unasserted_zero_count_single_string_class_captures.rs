//! Required empty iterations preserve class-versus-empty capture identity.
use crate::common::{assert_true, eval_string};
use spite_core::JsString;

use crate::common::regexp_matches::DESCRIBE_MATCH;
use spite_runtime::{Realm, Value};
#[test]
fn unasserted_zero_count_single_string_class_native_tables_consumers_and_required_class_identity() {
    let expected_serialized_output = JsString::from(concat!(
        r#"[["a","<undefined>","<undefined>","<undefined>","<undefined>","a","a","a","#,
        r#""<undefined>","<undefined>","<undefined>","<undefined>"],"#,
        r#"[[0,1],"<undefined>","<undefined>","<undefined>","<undefined>",[0,1],[0,1],[0,1],"#,
        r#""<undefined>","<undefined>","<undefined>","<undefined>"],0,1]"#,
    ));
    let expected_serialized_output_2 = JsString::from(concat!(
        r#"[["","<undefined>","<undefined>","<undefined>","<undefined>","","","","<undefined>","#,
        r#""<undefined>","<undefined>","<undefined>"],"#,
        r#"[[0,0],"<undefined>","<undefined>","<undefined>","<undefined>",[0,0],[0,0],[0,0],"#,
        r#""<undefined>","<undefined>","<undefined>","<undefined>"],0,0]"#,
    ));

    assert_true(
        r#"
            var pattern =
                "(?<before>^){0}(?<z>(?<z1>^)|(?<z2>\\b\\B)){0}(?<whole>(?<iteration>(?<c>[\\q{|a|ab}])))(?<after>$){0}",
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
                "[[\"\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"\",\"\",\"\",\"<undefined>\"],[[0,0],\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",[0,0],[0,0],[0,0],\"<undefined>\"],0,0]"
        "#,
    );
    assert_true(
        r#"
            var pattern =
                "(?<before>^){0}(?<z>(?<z1>^)|(?<z2>\\b\\B)){0}(?<whole>(?<iteration>(?<c>[\\q{a|ab}])){1})(?<after>$){0}",
                input = "aba",
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
                "[[\"ab\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"ab\",\"ab\",\"ab\",\"<undefined>\"],[[0,2],\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",[0,2],[0,2],[0,2],\"<undefined>\"],0,2]"
        "#,
    );
    assert_true(
        r#"
            var pattern =
                "(?<before>^){0}(?<z>(?<z1>^)|(?<z2>\\b\\B)){0}(?<whole>(?<iteration>(?<c>[\\p{Any}\\q{|ab}])){1}?)(?<after>$){0}",
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
                "[[\"a\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"a\",\"a\",\"a\",\"<undefined>\"],[[0,1],\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",[0,1],[0,1],[0,1],\"<undefined>\"],0,1]"
        "#,
    );
    assert_true(
        r#"
            var pattern =
                "(?<before>^){0}(?<whole>(?<iteration>(?<c>[a-z\\q{|S|sa|aX}])){1,1})(?<z>(?<z1>\\B)*|(?<z2>\\b)+){0}?(?<after>$){0}",
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
                "(?<before>\\b){0}(?<whole>(?<iteration>(?<c>[\\q{|a|ab}])){0001,0001}?)(?<z>(?<z1>\\B)*|(?<z2>\\b)+){0}?(?<after>\\b){0}",
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
                "[[\"\",\"<undefined>\",\"\",\"\",\"\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],[[0,0],\"<undefined>\",[0,0],[0,0],[0,0],\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],0,0]"
        "#,
    );
    assert_true(
        r#"
            var pattern =
                "(?<before>\\b){0}(?<whole>(?<iteration>(?<c>[\\q{a|ab}])))(?<z>(?<z1>\\B)*|(?<z2>\\b)+){0}?(?<after>\\b){0}",
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
                "[[\"ab\",\"<undefined>\",\"ab\",\"ab\",\"ab\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],[[2,4],\"<undefined>\",[2,4],[2,4],[2,4],\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],2,4]"
        "#,
    );
    assert_true(&format!(
        r#"
            var pattern =
                "(?<before>\\b){{0}}(?<z>(?<z1>\\b){{1,}}|(?<z2>\\B)?){{0}}(?<whole>(?<iteration>(?<c>[\\p{{Any}}\\q{{|ab}}])){{1}})(?<s>(?<s1>$)|(?<s2>^)){{000,000}}?(?<after>\\b){{0}}",
                input = "a",
                r = new RegExp(pattern, "dgvy"),
                ok = r.source === pattern && r.global && r.hasIndices && r.unicodeSets === true && r
                .unicode === false;
            r.lastIndex = 0;
            var m = r.exec(input);
            if (m !== null) {{
                var names = ["before", "z", "z1", "z2", "whole", "iteration", "c", "s", "s1", "s2",
                    "after"
                ];
                for (var n of names) {{
                    var slot = names.indexOf(n) + 1;
                    ok = ok && Object.hasOwn(m.groups, n) && Object.hasOwn(m.indices.groups, n) && m
                        .groups[n] === m[slot] && m.indices.groups[n] === m.indices[slot]
                }}
                ok = ok && Object.getPrototypeOf(m.groups) === null && m.input === input
            }} else ok = false;
            ok && m !== null && JSON.stringify([Array.from(m, v => v === undefined ? "<undefined>" : v),
                    Array.from(m.indices, v => v === undefined ? "<undefined>" : v), m.index, r
                    .lastIndex
                ]) ===
                {expected_serialized_output:?}
        "#
    ));
    assert_true(&format!(
        r#"
            var pattern =
                "(?<before>\\b){{0}}(?<z>(?<z1>\\b){{1,}}|(?<z2>\\B)?){{0}}(?<whole>(?<iteration>(?<c>[a-z\\q{{|S|sa|aX}}])){{1}}?)(?<s>(?<s1>$)|(?<s2>^)){{000,000}}?(?<after>\\b){{0}}",
                input = "!ab!",
                r = new RegExp(pattern, "dgvi"),
                ok = r.source === pattern && r.global && r.hasIndices && r.unicodeSets === true && r
                .unicode === false;
            r.lastIndex = 0;
            var m = r.exec(input);
            if (m !== null) {{
                var names = ["before", "z", "z1", "z2", "whole", "iteration", "c", "s", "s1", "s2",
                    "after"
                ];
                for (var n of names) {{
                    var slot = names.indexOf(n) + 1;
                    ok = ok && Object.hasOwn(m.groups, n) && Object.hasOwn(m.indices.groups, n) && m
                        .groups[n] === m[slot] && m.indices.groups[n] === m.indices[slot]
                }}
                ok = ok && Object.getPrototypeOf(m.groups) === null && m.input === input
            }} else ok = false;
            ok && m !== null && JSON.stringify([Array.from(m, v => v === undefined ? "<undefined>" : v),
                    Array.from(m.indices, v => v === undefined ? "<undefined>" : v), m.index, r
                    .lastIndex
                ]) ===
                {expected_serialized_output_2:?}
        "#
    ));
    assert_true(&format!(
        r#"
            var pattern =
                "(?<before>\\B){{0}}(?<z>(?<z1>\\b){{1,}}|(?<z2>\\B)?){{0}}(?<whole>(?<iteration>(?<c>[\\q{{|a|ab}}])){{1,1}})(?<s>(?<s1>$)|(?<s2>^)){{000,000}}?(?<after>\\B){{0}}",
                input = "",
                r = new RegExp(pattern, "dgv"),
                ok = r.source === pattern && r.global && r.hasIndices && r.unicodeSets === true && r
                .unicode === false;
            r.lastIndex = 0;
            var m = r.exec(input);
            if (m !== null) {{
                var names = ["before", "z", "z1", "z2", "whole", "iteration", "c", "s", "s1", "s2",
                    "after"
                ];
                for (var n of names) {{
                    var slot = names.indexOf(n) + 1;
                    ok = ok && Object.hasOwn(m.groups, n) && Object.hasOwn(m.indices.groups, n) && m
                        .groups[n] === m[slot] && m.indices.groups[n] === m.indices[slot]
                }}
                ok = ok && Object.getPrototypeOf(m.groups) === null && m.input === input
            }} else ok = false;
            ok && m !== null && JSON.stringify([Array.from(m, v => v === undefined ? "<undefined>" : v),
                    Array.from(m.indices, v => v === undefined ? "<undefined>" : v), m.index, r
                    .lastIndex
                ]) ===
                {expected_serialized_output_2:?}
        "#
    ));

    assert_true(
        r#"
            var pattern =
                "(?<before>^){0}(?<z>(?<z1>^)|(?<z2>\\b\\B)){0}(?<whole>(?<iteration>(?<c>[\\q{|a|ab}])){1}?)(?<after>\\b){0}",
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
                "(?<before>^){0}(?<z>(?<z1>^)|(?<z2>\\b\\B)){0}(?<whole>(?<iteration>(?<c>[\\q{a|ab}])){1,1})(?<after>\\b){0}",
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
                "[[\"ab\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"ab\",\"ab\",\"ab\",\"<undefined>\"],[[2,4],\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",[2,4],[2,4],[2,4],\"<undefined>\"],2,4]"
        "#,
    );
    assert_true(
        r#"
            var pattern =
                "(?<before>^){0}(?<z>(?<z1>^)|(?<z2>\\b\\B)){0}(?<whole>(?<iteration>(?<c>[\\p{Any}\\q{|ab}])){0001,0001}?)(?<after>\\b){0}",
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
                "[[\"a\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"a\",\"a\",\"a\",\"<undefined>\"],[[0,1],\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",[0,1],[0,1],[0,1],\"<undefined>\"],0,1]"
        "#,
    );
    assert_true(
        r#"
            var pattern =
                "(?<before>^){0}(?<whole>(?<iteration>(?<c>[a-z\\q{|S|sa|aX}])))(?<z>(?<z1>\\B)*|(?<z2>\\b)+){0}?(?<after>\\b){0}",
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
                "(?<before>^){0}(?<whole>(?<iteration>(?<c>[\\q{|a|ab}])){1})(?<z>(?<z1>\\B)*|(?<z2>\\b)+){0}?(?<after>$){0}",
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
                "[[\"\",\"<undefined>\",\"\",\"\",\"\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],[[0,0],\"<undefined>\",[0,0],[0,0],[0,0],\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],0,0]"
        "#,
    );
    assert_true(
        r#"
            var pattern =
                "(?<before>^){0}(?<whole>(?<iteration>(?<c>[\\q{a|ab}])){1}?)(?<z>(?<z1>\\B)*|(?<z2>\\b)+){0}?(?<after>$){0}",
                input = "aba",
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
                "[[\"ab\",\"<undefined>\",\"ab\",\"ab\",\"ab\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],[[0,2],\"<undefined>\",[0,2],[0,2],[0,2],\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],0,2]"
        "#,
    );
    assert_true(&format!(
        r#"
            var pattern =
                "(?<before>^){{0}}(?<z>(?<z1>\\b){{1,}}|(?<z2>\\B)?){{0}}(?<whole>(?<iteration>(?<c>[\\p{{Any}}\\q{{|ab}}])){{1,1}})(?<s>(?<s1>$)|(?<s2>^)){{000,000}}?(?<after>$){{0}}",
                input = "aaaa",
                r = new RegExp(pattern, "dgvy"),
                ok = r.source === pattern && r.global && r.hasIndices && r.unicodeSets === true && r
                .unicode === false;
            r.lastIndex = 0;
            var m = r.exec(input);
            if (m !== null) {{
                var names = ["before", "z", "z1", "z2", "whole", "iteration", "c", "s", "s1", "s2",
                    "after"
                ];
                for (var n of names) {{
                    var slot = names.indexOf(n) + 1;
                    ok = ok && Object.hasOwn(m.groups, n) && Object.hasOwn(m.indices.groups, n) && m
                        .groups[n] === m[slot] && m.indices.groups[n] === m.indices[slot]
                }}
                ok = ok && Object.getPrototypeOf(m.groups) === null && m.input === input
            }} else ok = false;
            ok && m !== null && JSON.stringify([Array.from(m, v => v === undefined ? "<undefined>" : v),
                    Array.from(m.indices, v => v === undefined ? "<undefined>" : v), m.index, r
                    .lastIndex
                ]) ===
                {expected_serialized_output:?}
        "#
    ));
    assert_true(&format!(
        r#"
            var pattern =
                "(?<before>^){{0}}(?<z>(?<z1>\\b){{1,}}|(?<z2>\\B)?){{0}}(?<whole>(?<iteration>(?<c>[a-z\\q{{|S|sa|aX}}])){{0001,0001}}?)(?<s>(?<s1>$)|(?<s2>^)){{000,000}}?(?<after>$){{0}}",
                input = "\ud83d\ude00aX",
                r = new RegExp(pattern, "dgvi"),
                ok = r.source === pattern && r.global && r.hasIndices && r.unicodeSets === true && r
                .unicode === false;
            r.lastIndex = 1;
            var m = r.exec(input);
            if (m !== null) {{
                var names = ["before", "z", "z1", "z2", "whole", "iteration", "c", "s", "s1", "s2",
                    "after"
                ];
                for (var n of names) {{
                    var slot = names.indexOf(n) + 1;
                    ok = ok && Object.hasOwn(m.groups, n) && Object.hasOwn(m.indices.groups, n) && m
                        .groups[n] === m[slot] && m.indices.groups[n] === m.indices[slot]
                }}
                ok = ok && Object.getPrototypeOf(m.groups) === null && m.input === input
            }} else ok = false;
            ok && m !== null && JSON.stringify([Array.from(m, v => v === undefined ? "<undefined>" : v),
                    Array.from(m.indices, v => v === undefined ? "<undefined>" : v), m.index, r
                    .lastIndex
                ]) ===
                {expected_serialized_output_2:?}
        "#
    ));
    assert_true(&format!(
        r#"
            var pattern =
                "(?<before>\\b){{0}}(?<z>(?<z1>\\b){{1,}}|(?<z2>\\B)?){{0}}(?<whole>(?<iteration>(?<c>[\\q{{|a|ab}}])))(?<s>(?<s1>$)|(?<s2>^)){{000,000}}?(?<after>\\b){{0}}",
                input = "\u017faX",
                r = new RegExp(pattern, "dgvm"),
                ok = r.source === pattern && r.global && r.hasIndices && r.unicodeSets === true && r
                .unicode === false;
            r.lastIndex = 0;
            var m = r.exec(input);
            if (m !== null) {{
                var names = ["before", "z", "z1", "z2", "whole", "iteration", "c", "s", "s1", "s2",
                    "after"
                ];
                for (var n of names) {{
                    var slot = names.indexOf(n) + 1;
                    ok = ok && Object.hasOwn(m.groups, n) && Object.hasOwn(m.indices.groups, n) && m
                        .groups[n] === m[slot] && m.indices.groups[n] === m.indices[slot]
                }}
                ok = ok && Object.getPrototypeOf(m.groups) === null && m.input === input
            }} else ok = false;
            ok && m !== null && JSON.stringify([Array.from(m, v => v === undefined ? "<undefined>" : v),
                    Array.from(m.indices, v => v === undefined ? "<undefined>" : v), m.index, r
                    .lastIndex
                ]) ===
                {expected_serialized_output_2:?}
        "#
    ));

    assert_true(
        r#"
            var pattern =
                "(?<before>\\B){0}(?<z>(?<z1>^)|(?<z2>\\b\\B)){0}(?<whole>(?<iteration>(?<c>[\\q{|a|ab}])){0001,0001}?)(?<after>\\B){0}",
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
                "[[\"\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"\",\"\",\"\",\"<undefined>\"],[[0,0],\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",[0,0],[0,0],[0,0],\"<undefined>\"],0,0]"
        "#,
    );
    assert_true(
        r#"
            var pattern =
                "(?<before>\\B){0}(?<z>(?<z1>^)|(?<z2>\\b\\B)){0}(?<whole>(?<iteration>(?<c>[\\q{a|ab}])))(?<after>\\B){0}",
                input = "aba",
                r = new RegExp(pattern, "dgvm"),
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
                "[[\"ab\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"ab\",\"ab\",\"ab\",\"<undefined>\"],[[0,2],\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",[0,2],[0,2],[0,2],\"<undefined>\"],0,2]"
        "#,
    );
    assert_true(
        r#"
            var pattern =
                "(?<before>\\B){0}(?<z>(?<z1>^)|(?<z2>\\b\\B)){0}(?<whole>(?<iteration>(?<c>[\\p{Any}\\q{|ab}])){1})(?<after>\\B){0}",
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
                "[[\"a\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"a\",\"a\",\"a\",\"<undefined>\"],[[0,1],\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",[0,1],[0,1],[0,1],\"<undefined>\"],0,1]"
        "#,
    );
    assert_true(
        r#"
            var pattern =
                "(?<before>\\B){0}(?<whole>(?<iteration>(?<c>[a-z\\q{|S|sa|aX}])){1}?)(?<z>(?<z1>\\B)*|(?<z2>\\b)+){0}?(?<after>\\B){0}",
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
                "(?<before>^){0}(?<whole>(?<iteration>(?<c>[\\q{|a|ab}])){1,1})(?<z>(?<z1>\\B)*|(?<z2>\\b)+){0}?(?<after>\\b){0}",
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
                "[[\"\",\"<undefined>\",\"\",\"\",\"\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],[[0,0],\"<undefined>\",[0,0],[0,0],[0,0],\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],0,0]"
        "#,
    );
    assert_true(
        r#"
            var pattern =
                "(?<before>^){0}(?<whole>(?<iteration>(?<c>[\\q{a|ab}])){0001,0001}?)(?<z>(?<z1>\\B)*|(?<z2>\\b)+){0}?(?<after>\\b){0}",
                input = "\n\nab\n",
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
                "[[\"ab\",\"<undefined>\",\"ab\",\"ab\",\"ab\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],[[2,4],\"<undefined>\",[2,4],[2,4],[2,4],\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],2,4]"
        "#,
    );
    assert_true(&format!(
        r#"
            var pattern =
                "(?<before>^){{0}}(?<z>(?<z1>\\b){{1,}}|(?<z2>\\B)?){{0}}(?<whole>(?<iteration>(?<c>[\\p{{Any}}\\q{{|ab}}])))(?<s>(?<s1>$)|(?<s2>^)){{000,000}}?(?<after>\\b){{0}}",
                input = "a",
                r = new RegExp(pattern, "dgvym"),
                ok = r.source === pattern && r.global && r.hasIndices && r.unicodeSets === true && r
                .unicode === false;
            r.lastIndex = 0;
            var m = r.exec(input);
            if (m !== null) {{
                var names = ["before", "z", "z1", "z2", "whole", "iteration", "c", "s", "s1", "s2",
                    "after"
                ];
                for (var n of names) {{
                    var slot = names.indexOf(n) + 1;
                    ok = ok && Object.hasOwn(m.groups, n) && Object.hasOwn(m.indices.groups, n) && m
                        .groups[n] === m[slot] && m.indices.groups[n] === m.indices[slot]
                }}
                ok = ok && Object.getPrototypeOf(m.groups) === null && m.input === input
            }} else ok = false;
            ok && m !== null && JSON.stringify([Array.from(m, v => v === undefined ? "<undefined>" : v),
                    Array.from(m.indices, v => v === undefined ? "<undefined>" : v), m.index, r
                    .lastIndex
                ]) ===
                {expected_serialized_output:?}
        "#
    ));
    assert_true(&format!(
        r#"
            var pattern =
                "(?<before>^){{0}}(?<z>(?<z1>\\b){{1,}}|(?<z2>\\B)?){{0}}(?<whole>(?<iteration>(?<c>[a-z\\q{{|S|sa|aX}}])){{1}})(?<s>(?<s1>$)|(?<s2>^)){{000,000}}?(?<after>\\b){{0}}",
                input = "!ab!",
                r = new RegExp(pattern, "dgvi"),
                ok = r.source === pattern && r.global && r.hasIndices && r.unicodeSets === true && r
                .unicode === false;
            r.lastIndex = 0;
            var m = r.exec(input);
            if (m !== null) {{
                var names = ["before", "z", "z1", "z2", "whole", "iteration", "c", "s", "s1", "s2",
                    "after"
                ];
                for (var n of names) {{
                    var slot = names.indexOf(n) + 1;
                    ok = ok && Object.hasOwn(m.groups, n) && Object.hasOwn(m.indices.groups, n) && m
                        .groups[n] === m[slot] && m.indices.groups[n] === m.indices[slot]
                }}
                ok = ok && Object.getPrototypeOf(m.groups) === null && m.input === input
            }} else ok = false;
            ok && m !== null && JSON.stringify([Array.from(m, v => v === undefined ? "<undefined>" : v),
                    Array.from(m.indices, v => v === undefined ? "<undefined>" : v), m.index, r
                    .lastIndex
                ]) ===
                {expected_serialized_output_2:?}
        "#
    ));
    assert_true(&format!(
        r#"
            var pattern =
                "(?<before>^){{0}}(?<z>(?<z1>\\b){{1,}}|(?<z2>\\B)?){{0}}(?<whole>(?<iteration>(?<c>[\\q{{|a|ab}}])){{1}}?)(?<s>(?<s1>$)|(?<s2>^)){{000,000}}?(?<after>$){{0}}",
                input = "",
                r = new RegExp(pattern, "dgv"),
                ok = r.source === pattern && r.global && r.hasIndices && r.unicodeSets === true && r
                .unicode === false;
            r.lastIndex = 0;
            var m = r.exec(input);
            if (m !== null) {{
                var names = ["before", "z", "z1", "z2", "whole", "iteration", "c", "s", "s1", "s2",
                    "after"
                ];
                for (var n of names) {{
                    var slot = names.indexOf(n) + 1;
                    ok = ok && Object.hasOwn(m.groups, n) && Object.hasOwn(m.indices.groups, n) && m
                        .groups[n] === m[slot] && m.indices.groups[n] === m.indices[slot]
                }}
                ok = ok && Object.getPrototypeOf(m.groups) === null && m.input === input
            }} else ok = false;
            ok && m !== null && JSON.stringify([Array.from(m, v => v === undefined ? "<undefined>" : v),
                    Array.from(m.indices, v => v === undefined ? "<undefined>" : v), m.index, r
                    .lastIndex
                ]) ===
                {expected_serialized_output_2:?}
        "#
    ));

    assert_true(
        r#"
            var pattern =
                "(?<before>\\b){0}(?<z>(?<z1>^)|(?<z2>\\b\\B)){0}(?<whole>(?<iteration>(?<c>[\\q{|a|ab}])){1})(?<after>\\b){0}",
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
                "(?<before>\\b){0}(?<z>(?<z1>^)|(?<z2>\\b\\B)){0}(?<whole>(?<iteration>(?<c>[\\q{a|ab}])){1}?)(?<after>\\b){0}",
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
                "[[\"ab\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"ab\",\"ab\",\"ab\",\"<undefined>\"],[[2,4],\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",[2,4],[2,4],[2,4],\"<undefined>\"],2,4]"
        "#,
    );
    assert_true(
        r#"
            var pattern =
                "(?<before>\\b){0}(?<z>(?<z1>^)|(?<z2>\\b\\B)){0}(?<whole>(?<iteration>(?<c>[\\p{Any}\\q{|ab}])){1,1})(?<after>\\b){0}",
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
                "[[\"a\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"a\",\"a\",\"a\",\"<undefined>\"],[[0,1],\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",[0,1],[0,1],[0,1],\"<undefined>\"],0,1]"
        "#,
    );
    assert_true(
        r#"
            var pattern =
                "(?<before>\\b){0}(?<whole>(?<iteration>(?<c>[a-z\\q{|S|sa|aX}])){0001,0001}?)(?<z>(?<z1>\\B)*|(?<z2>\\b)+){0}?(?<after>\\b){0}",
                input = "!ab!",
                r = new RegExp(pattern, "dgvyi"),
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

    let mut realm = Realm::default();
    realm.eval(r"var r=/(?:(?:(?:\b){0})|(?:(?:\B){0})){0}(?:^){0}(?<whole>(?<iteration>(?<e>){0}(?<c>[\p{Any}\q{|ababa|a|ab}])){1})(?<after>)(?:$){0}/dvg,clone=new RegExp(r),m=r.exec('ababa')").unwrap();
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
                    /(?:(?:\b)|(?:\B)){{0}}(?<before>(?:\b){{0}})(?<whole>(?<iteration>(?<e>){{0}}(?<c>[\p{{Any}}\q{{a|ab}}])){{1}})(?<after>(?:\b){{0}})/dvg,
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
                /(?:(?:\b)|(?:\B)){0}(?<before>(?:\b){0})(?<whole>(?<iteration>(?<c>[\p{Any}\q{|a|ab}])(?<e>){0}){1})(?<after>(?:\b){0})/vg,
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
          "<C:ab><C:!><C:X><C:!><C:>",
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
              "!",
              "",
              "!",
              "!",
              "!",
              "<undefined>",
              "",
              4,
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
                /(?:(?:\b)|(?:\B)){0}(?<before>(?:\b){0})(?<whole>(?<iteration>(?<e>){0}(?<c>[\p{Any}\q{a|ab}])){1})(?<after>(?:\b){0})/dvg;
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
            "!",
            "!",
            "<undefined>",
            "!",
            "",
            "",
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
            "",
            "",
            "!",
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

            var r = /(?:(?:\b)|(?:\B)){0}(?:^){0}((){0}([\p{Any}\q{a|ab}])){1}(?:\b){0}/vg;
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
                    /(?:(?:\b)|(?:\B)){{0}}(?:^){{0}}(?<\u{{1D49C}}>)(?<whole>(?<iteration>(?<e>){{0}}(?<c>[\p{{Any}}\q{{|😀|😀a|aX}}])){{1}})(?<after>)(?:$){{0}}/dvgy;
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
                    /(?:(?:\b)|(?:\B)){{0}}(?<before>(?:\b){{0}})(?<whole>(?<iteration>(?<e>){{0}}(?<c>[\p{{Any}}\q{{|S|sa|aX}}])){{1}})(?<after>(?:\b){{0}})/dvi,
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
              "ſa",
              "",
              "ſa",
              "ſa",
              "<undefined>",
              "ſa",
              ""
            ],
            "indices": [
              [
                0,
                2
              ],
              [
                0,
                0
              ],
              [
                0,
                2
              ],
              [
                0,
                2
              ],
              "<undefined>",
              [
                0,
                2
              ],
              [
                2,
                2
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
              "ſa",
              "ſa",
              "<undefined>",
              "ſa",
              ""
            ],
            "namedIndices": [
              [
                0,
                0
              ],
              [
                0,
                2
              ],
              [
                0,
                2
              ],
              "<undefined>",
              [
                0,
                2
              ],
              [
                2,
                2
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
                    /(?:(?:\b)|(?:\B)){{0}}(?:^){{0}}(?<whole>(?<iteration>(?<c>[\p{{Any}}\q{{|a|ab}}])(?<e>){{0}}){{1}})(?:\b){{0}}/dv
                    .exec('x'),
                    e =
                    /(?:(?:\b)|(?:\B)){{0}}(?:^){{0}}(?<whole>(?<iteration>(?<e>){{0}}(?<c>[\p{{Any}}\q{{|a|ab}}])){{1}})(?:\b){{0}}/dv
                    .exec('x'),
                    c =
                    /(?:(?:\b)|(?:\B)){{0}}(?:^){{0}}(?<whole>(?<iteration>(?<c>[\p{{Any}}\q{{|a|ab}}])(?<e>){{0}}){{1}})(?:\b){{0}}/dv
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
                    /(?:(?:\b)|(?:\B)){{0}}(?:^){{0}}(?<whole>(?<iteration>(?<e>){{0}}(?<c>[\q{{ab}}&&\q{{cd}}])){{1}})(?:$){{0}}/dv
                    .exec(''),
                    c = /(?:(?:\b)|(?:\B)){{0}}(?:^){{0}}(?<whole>(?<c>[\p{{Any}}\q{{}}]){{1}})(?:$){{0}}/dv.exec(
                        '');
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
