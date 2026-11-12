//! Required empty iterations preserve class-versus-empty capture identity.
use crate::common::assert_true;

use crate::common::regexp_captures::assert_no_match;
use spite_core::JsString;
use spite_runtime::{Realm, Value};
use test_case::test_case;
#[test]
fn zero_minimum_asserted_alternative_string_class_native_tables_consumers_and_required_empty_identity()
 {
    assert_true(
        r#"
            var pattern =
                "(?<before>^)(?<z>(?<z1>^)|(?<z2>\\b\\B))?(?<whole>(?<iteration>(?<c>[\\q{|a|ab}])|(?<e>)))(?<after>$)",
                input = "",
                r = new RegExp(pattern, "dgvym"),
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
                "[[\"\",\"\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"\",\"\",\"\",\"<undefined>\",\"\"],[[0,0],[0,0],\"<undefined>\",\"<undefined>\",\"<undefined>\",[0,0],[0,0],[0,0],\"<undefined>\",[0,0]],0,0]"
        "#,
    );

    assert_true(
        r#"
            var pattern =
                "(?<before>\\b)(?<whole>(?<iteration>(?<c>[\\q{|a|ab}])|(?<e>)){0001,0001}?)(?<z>(?<z1>\\B)*|(?<z2>\\b)+)*?(?<after>\\b)",
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
                "[[\"\",\"\",\"\",\"\",\"\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"\"],[[1,1],[1,1],[1,1],[1,1],[1,1],\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",[1,1]],1,1]"
        "#,
    );
    assert_true(
        r#"
            var pattern =
                "(?<before>\\b)(?<whole>(?<iteration>(?<e>)|(?<c>[\\q{a|ab}])))(?<z>(?<z1>\\B)*|(?<z2>\\b)+)*?(?<after>\\b)",
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
                "[[\"\",\"\",\"\",\"\",\"\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"\"],[[2,2],[2,2],[2,2],[2,2],[2,2],\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",[2,2]],2,2]"
        "#,
    );

    assert_true(
        r#"
            var pattern =
                "(?<before>^)(?<z>(?<z1>^)|(?<z2>\\b\\B))?(?<whole>(?<iteration>(?<c>[\\p{Any}\\q{|ab}])|(?<e>)){0001,0001}?)(?<after>\\b)",
                input = "a",
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
                "[[\"a\",\"\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"a\",\"a\",\"a\",\"<undefined>\",\"\"],[[0,1],[0,0],\"<undefined>\",\"<undefined>\",\"<undefined>\",[0,1],[0,1],[0,1],\"<undefined>\",[1,1]],0,1]"
        "#,
    );

    assert_true(
        r#"
            var pattern =
                "(?<before>^)(?<whole>(?<iteration>(?<c>[\\q{|a|ab}])|(?<e>)){1})(?<z>(?<z1>\\B)*|(?<z2>\\b)+)*?(?<after>$)",
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
                "[[\"\",\"\",\"\",\"\",\"\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"\"],[[0,0],[0,0],[0,0],[0,0],[0,0],\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",[0,0]],0,0]"
        "#,
    );

    assert_true(
        r#"
            var pattern =
                "(?<before>\\B)(?<z>(?<z1>^)|(?<z2>\\b\\B))?(?<whole>(?<iteration>(?<c>[\\q{|a|ab}])|(?<e>)){0001,0001}?)(?<after>\\B)",
                input = "",
                r = new RegExp(pattern, "dgvy"),
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
                "[[\"\",\"\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"\",\"\",\"\",\"<undefined>\",\"\"],[[0,0],[0,0],\"<undefined>\",\"<undefined>\",\"<undefined>\",[0,0],[0,0],[0,0],\"<undefined>\",[0,0]],0,0]"
        "#,
    );
    assert_true(
        r#"
            var pattern =
                "(?<before>\\B)(?<z>(?<z1>^)|(?<z2>\\b\\B))?(?<whole>(?<iteration>(?<e>)|(?<c>[\\q{a|ab}])))(?<after>\\B)",
                input = "aba",
                r = new RegExp(pattern, "dgvm"),
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
                "[[\"\",\"\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"\",\"\",\"\",\"<undefined>\",\"\"],[[1,1],[1,1],\"<undefined>\",\"<undefined>\",\"<undefined>\",[1,1],[1,1],[1,1],\"<undefined>\",[1,1]],1,1]"
        "#,
    );
    assert_true(
        r#"
            var pattern =
                "(?<before>\\B)(?<z>(?<z1>^)|(?<z2>\\b\\B))?(?<whole>(?<iteration>(?<c>[\\p{Any}\\q{|ab}])|(?<e>)){1})(?<after>\\B)",
                input = "aaaa",
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
                "[[\"a\",\"\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"a\",\"a\",\"a\",\"<undefined>\",\"\"],[[1,2],[1,1],\"<undefined>\",\"<undefined>\",\"<undefined>\",[1,2],[1,2],[1,2],\"<undefined>\",[2,2]],1,2]"
        "#,
    );
    assert_true(
        r#"
            var pattern =
                "(?<before>\\B)(?<whole>(?<iteration>(?<e>)|(?<c>[a-z\\q{|S|sa|aX}])){1}?)(?<z>(?<z1>\\B)*|(?<z2>\\b)+)*?(?<after>\\B)",
                input = "\ud83d\ude00aX",
                r = new RegExp(pattern, "dgvyi"),
                ok = r.source === pattern && r.global && r.hasIndices && r.unicodeSets === true && r
                .unicode === false;
            r.lastIndex = 1;
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
                "[[\"\",\"\",\"\",\"\",\"\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"\"],[[0,0],[0,0],[0,0],[0,0],[0,0],\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",[0,0]],0,0]"
        "#,
    );

    assert_true(
        r#"
            var pattern =
                "(?<before>\\b)(?<z>(?<z1>^)|(?<z2>\\b\\B))?(?<whole>(?<iteration>(?<e>)|(?<c>[\\q{a|ab}])){1}?)(?<after>\\b)",
                input = "\n\nab\n",
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
                "[[\"\",\"\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"\",\"\",\"\",\"<undefined>\",\"\"],[[2,2],[2,2],\"<undefined>\",\"<undefined>\",\"<undefined>\",[2,2],[2,2],[2,2],\"<undefined>\",[2,2]],2,2]"
        "#,
    );
    assert_true(
        r#"
            var pattern =
                "(?<before>\\b)(?<z>(?<z1>^)|(?<z2>\\b\\B))?(?<whole>(?<iteration>(?<c>[\\p{Any}\\q{|ab}])|(?<e>)){1,1})(?<after>\\b)",
                input = "a",
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
                "[[\"a\",\"\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"a\",\"a\",\"a\",\"<undefined>\",\"\"],[[0,1],[0,0],\"<undefined>\",\"<undefined>\",\"<undefined>\",[0,1],[0,1],[0,1],\"<undefined>\",[1,1]],0,1]"
        "#,
    );

    let mut realm = Realm::default();
    realm.eval(r"var r=/(?:(?:\b)|(?:\B))?^(?<whole>(?<iteration>(?<c>[\p{Any}\q{|ababa|a|ab}])|(?<e>)){1})(?<after>)$/dvg,clone=new RegExp(r),m=r.exec('ababa')").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval(r#"
        var n = clone.exec('');
        m.groups.whole === 'ababa' && m.groups.c === 'ababa' && m.groups.e === undefined && m.indices
            .groups.c === m.indices[3] && m.indices[3][0] === 0 && m.indices[3][1] === 5 && n[0] ===
            '' && n.groups.iteration === '' && n.groups.e === undefined && n.groups.c === '' && n
            .indices.groups.e === undefined && n.indices.groups.c === n.indices[3] && n.indices[3][
            0] === 0 && n.indices[3][1] === 0 && clone.source === r.source && clone.lastIndex === 0 &&
            r.lastIndex === 5
    "#),Ok(Value::Boolean(true)));
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
        r"(()|([\q{ab}&&\p{RGI_Emoji}])){1}",
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
    r#"(?<before>^)(?<z>(?<z1>^)|(?<z2>\b\B))?(?<whole>(?<iteration>(?<e>)|(?<c>[\q{a|ab}])){1})(?<after>$)"#,
    "dgv",
    JsString::from("aba"),
    0;
    "input_aba_from_0_variant_1"
)]
#[test_case(
    r#"(?<before>^)(?<z>(?<z1>^)|(?<z2>\b\B))?(?<whole>(?<iteration>(?<c>[\p{Any}\q{|ab}])|(?<e>)){1}?)(?<after>$)"#,
    "dgv",
    JsString::from("aaaa"),
    0;
    "input_aaaa_from_0_variant_2"
)]
#[test_case(
    r#"(?<before>^)(?<whole>(?<iteration>(?<e>)|(?<c>[a-z\q{|S|sa|aX}])){1,1})(?<z>(?<z1>\B)*|(?<z2>\b)+)*?(?<after>$)"#,
    "dgvyi",
    JsString::from("😀aX"),
    1;
    "unicode_input_from_1_variant_3"
)]
#[test_case(
    r#"(?<before>^)(?<z>(?<z1>^)|(?<z2>\b\B))?(?<whole>(?<iteration>(?<c>[\q{|a|ab}])|(?<e>)){1}?)(?<after>\b)"#,
    "dgvy",
    JsString::from("ſaX"),
    0;
    "unicode_input_from_0_variant_4"
)]
#[test_case(
    r#"(?<before>^)(?<z>(?<z1>^)|(?<z2>\b\B))?(?<whole>(?<iteration>(?<e>)|(?<c>[\q{a|ab}])){1,1})(?<after>\b)"#,
    "dgv",
    JsString::from("\n\nab\n"),
    0;
    "input_ab_from_0_variant_5"
)]
#[test_case(
    r#"(?<before>^)(?<whole>(?<iteration>(?<e>)|(?<c>[a-z\q{|S|sa|aX}])))(?<z>(?<z1>\B)*|(?<z2>\b)+)*?(?<after>\b)"#,
    "dgvyim",
    JsString::from("!ab!"),
    0;
    "input_ab_from_0_variant_6"
)]
#[test_case(
    r#"(?<before>^)(?<whole>(?<iteration>(?<e>)|(?<c>[\q{a|ab}])){1}?)(?<z>(?<z1>\B)*|(?<z2>\b)+)*?(?<after>$)"#,
    "dgv",
    JsString::from("aba"),
    0;
    "input_aba_from_0_variant_7"
)]
#[test_case(
    r#"(?<before>^)(?<whole>(?<iteration>(?<c>[\q{|a|ab}])|(?<e>)){1,1})(?<z>(?<z1>\B)*|(?<z2>\b)+)*?(?<after>\b)"#,
    "dgv",
    JsString::from("ſaX"),
    0;
    "unicode_input_from_0_variant_8"
)]
#[test_case(
    r#"(?<before>^)(?<whole>(?<iteration>(?<e>)|(?<c>[\q{a|ab}])){0001,0001}?)(?<z>(?<z1>\B)*|(?<z2>\b)+)*?(?<after>\b)"#,
    "dgv",
    JsString::from("\n\nab\n"),
    0;
    "input_ab_from_0_variant_9"
)]
#[test_case(
    r#"(?<before>\b)(?<z>(?<z1>^)|(?<z2>\b\B))?(?<whole>(?<iteration>(?<c>[\q{|a|ab}])|(?<e>)){1})(?<after>\b)"#,
    "dgvy",
    JsString::from("ſaX"),
    0;
    "unicode_input_from_0_variant_10"
)]
#[test_case(
    r#"(?<before>\b)(?<whole>(?<iteration>(?<e>)|(?<c>[a-z\q{|S|sa|aX}])){0001,0001}?)(?<z>(?<z1>\B)*|(?<z2>\b)+)*?(?<after>\b)"#,
    "dgvyi",
    JsString::from("!ab!"),
    0;
    "input_ab_from_0_variant_11"
)]
fn no_match_resets_last_index(
    pattern: &str,
    flags: &str,
    input: JsString,
    initial_last_index: usize,
) {
    assert_no_match(pattern, flags, input, initial_last_index);
}
