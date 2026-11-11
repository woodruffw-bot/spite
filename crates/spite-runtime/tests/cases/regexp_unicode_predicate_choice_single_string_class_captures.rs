//! Required predicate choices preserve conditional lexical captures.
use crate::common::assert_true;
use crate::common::regexp_captures::{MatchExpectation, assert_named_captures};
use spite_core::JsString;
use spite_runtime::{Realm, Value};
use test_case::test_case;
#[test]
fn matching_controls_and_string_consumers() {
    assert_true(
        r#"
            var pattern = "(?<n1>(?<n2>\\b)|(?<n3>\\B))(?<n4>[\\q{a|ab}])(?<n5>^|(?<n6>))",
                input = "",
                r = new RegExp(pattern, "dgvym"),
                ok = r.source === pattern && r.global && r.hasIndices && r.unicodeSets === true && r
                .unicode === false;
            r.lastIndex = 0;
            var m = r.exec(input);
            ok && m === null && r.lastIndex === 0
        "#,
    );

    assert_true(
        r#"
            var pattern = "(?<n1>(?<n2>\\B)|(?<n3>\\b)){1}(?<n4>[\\q{a|ab}])(?<n5>$|(?<n6>))",
                input = "a",
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
                "(?<n1>\\b|(?<n2>)){2,3}?(?<n3>(?<n4>[a-z\\q{ab}]){1}){1,1}?(?<n5>(?<n6>^)|(?<n7>$)|(?<n8>\\b)|(?<n9>\\B))",
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
                "(?<n1>(?<n2>^)|(?<n3>$)|(?<n4>\\b)|(?<n5>\\B))(?<n6>(?<n7>[a-z\\q{ab}]){1}){1,1}?(?<n8>(?<n9>\\b\\B)|(?<n10>))",
                input = "!ab!",
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
                "(?<n1>(?<n2>(?<n3>\\b)|(?<n4>\\B)){2,3})(?<n5>(?<n6>[\\q{|a|ab}])){1}(?<n7>(?<n8>)|(?<n9>\\b))",
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
                "(?<n1>(?<n2>(?<n3>\\b)|(?<n4>\\B)){2,3})(?<n5>(?<n6>[a-z\\q{ab}]){1}){1,1}?(?<n7>(?<n8>)|(?<n9>\\b))",
                input = "a\n!",
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
                "(?<n1>(?<n2>(?<n3>^)|(?<n4>)){0,3})(?<n5>[\\q{a|ab}])(?<n6>(?<n7>\\b)|(?<n8>\\B))",
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
            var pattern = "(?<n1>(?<n2>\\b\\B)|(?<n3>))(?<n4>[\\q{a|ab}])(?<n5>(?<n6>\\B)|(?<n7>\\b)){1}",
                input = "",
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
            var pattern = "(?<n1>(?<n2>)|(?<n3>\\b))(?<n4>[\\q{a|ab}])(?<n5>\\b|(?<n6>)){2,3}?",
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
            (function() {
                function row(m) {
                    if (m === null) return null;
                    var names = m.groups === undefined ? [] : Object.keys(m.groups);
                    return [Array.from(m, v => v === undefined ? '<undefined>' : v), Array.from(m
                            .indices, v => v === undefined ? '<undefined>' : v), m.index, m.input,
                        names, names.map(n => m.groups[n] === undefined ? '<undefined>' : m.groups[
                            n]), names.map(n => m.indices.groups[n] === undefined ? '<undefined>' :
                            m.indices.groups[n]), names.every((n, i) => Object.hasOwn(m.groups,
                            n) && Object.hasOwn(m.indices.groups, n) && m.groups[n] === m[i + 1] &&
                            m.indices.groups[n] === m.indices[i + 1]), m.groups === undefined ||
                        Object.getPrototypeOf(m.groups) === null
                    ]
                }
                var P =
                    "(?<b>(?<w>\\b)|(?<nw>\\B))(?<whole>(?<c>[\\q{|a|ab}]))(?<a>(?<s>^)|(?<end>$)|(?<wb>\\b)|(?<nwb>\\B))";
                var r = new RegExp(P, "dvg"),
                    a = Array.from("ab!aa".matchAll(r));
                return JSON.stringify([a.map(row), r.lastIndex]);
            })() ===
            "[[[[\"ab\",\"\",\"\",\"<undefined>\",\"ab\",\"ab\",\"\",\"<undefined>\",\"<undefined>\",\"\",\"<undefined>\"],[[0,2],[0,0],[0,0],\"<undefined>\",[0,2],[0,2],[2,2],\"<undefined>\",\"<undefined>\",[2,2],\"<undefined>\"],0,\"ab!aa\",[\"b\",\"w\",\"nw\",\"whole\",\"c\",\"a\",\"s\",\"end\",\"wb\",\"nwb\"],[\"\",\"\",\"<undefined>\",\"ab\",\"ab\",\"\",\"<undefined>\",\"<undefined>\",\"\",\"<undefined>\"],[[0,0],[0,0],\"<undefined>\",[0,2],[0,2],[2,2],\"<undefined>\",\"<undefined>\",[2,2],\"<undefined>\"],true,true],[[\"\",\"\",\"\",\"<undefined>\",\"\",\"\",\"\",\"<undefined>\",\"<undefined>\",\"\",\"<undefined>\"],[[2,2],[2,2],[2,2],\"<undefined>\",[2,2],[2,2],[2,2],\"<undefined>\",\"<undefined>\",[2,2],\"<undefined>\"],2,\"ab!aa\",[\"b\",\"w\",\"nw\",\"whole\",\"c\",\"a\",\"s\",\"end\",\"wb\",\"nwb\"],[\"\",\"\",\"<undefined>\",\"\",\"\",\"\",\"<undefined>\",\"<undefined>\",\"\",\"<undefined>\"],[[2,2],[2,2],\"<undefined>\",[2,2],[2,2],[2,2],\"<undefined>\",\"<undefined>\",[2,2],\"<undefined>\"],true,true],[[\"a\",\"\",\"\",\"<undefined>\",\"a\",\"a\",\"\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"\"],[[3,4],[3,3],[3,3],\"<undefined>\",[3,4],[3,4],[4,4],\"<undefined>\",\"<undefined>\",\"<undefined>\",[4,4]],3,\"ab!aa\",[\"b\",\"w\",\"nw\",\"whole\",\"c\",\"a\",\"s\",\"end\",\"wb\",\"nwb\"],[\"\",\"\",\"<undefined>\",\"a\",\"a\",\"\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"\"],[[3,3],[3,3],\"<undefined>\",[3,4],[3,4],[4,4],\"<undefined>\",\"<undefined>\",\"<undefined>\",[4,4]],true,true],[[\"a\",\"\",\"<undefined>\",\"\",\"a\",\"a\",\"\",\"<undefined>\",\"\",\"<undefined>\",\"<undefined>\"],[[4,5],[4,4],\"<undefined>\",[4,4],[4,5],[4,5],[5,5],\"<undefined>\",[5,5],\"<undefined>\",\"<undefined>\"],4,\"ab!aa\",[\"b\",\"w\",\"nw\",\"whole\",\"c\",\"a\",\"s\",\"end\",\"wb\",\"nwb\"],[\"\",\"<undefined>\",\"\",\"a\",\"a\",\"\",\"<undefined>\",\"\",\"<undefined>\",\"<undefined>\"],[[4,4],\"<undefined>\",[4,4],[4,5],[4,5],[5,5],\"<undefined>\",[5,5],\"<undefined>\",\"<undefined>\"],true,true],[[\"\",\"\",\"\",\"<undefined>\",\"\",\"\",\"\",\"<undefined>\",\"\",\"<undefined>\",\"<undefined>\"],[[5,5],[5,5],[5,5],\"<undefined>\",[5,5],[5,5],[5,5],\"<undefined>\",[5,5],\"<undefined>\",\"<undefined>\"],5,\"ab!aa\",[\"b\",\"w\",\"nw\",\"whole\",\"c\",\"a\",\"s\",\"end\",\"wb\",\"nwb\"],[\"\",\"\",\"<undefined>\",\"\",\"\",\"\",\"<undefined>\",\"\",\"<undefined>\",\"<undefined>\"],[[5,5],[5,5],\"<undefined>\",[5,5],[5,5],[5,5],\"<undefined>\",[5,5],\"<undefined>\",\"<undefined>\"],true,true]],0]"
        "#,
    );
    assert_true(
        r#"
            (function() {
                function row(m) {
                    if (m === null) return null;
                    var names = m.groups === undefined ? [] : Object.keys(m.groups);
                    return [Array.from(m, v => v === undefined ? '<undefined>' : v), Array.from(m
                            .indices, v => v === undefined ? '<undefined>' : v), m.index, m.input,
                        names, names.map(n => m.groups[n] === undefined ? '<undefined>' : m.groups[
                            n]), names.map(n => m.indices.groups[n] === undefined ? '<undefined>' :
                            m.indices.groups[n]), names.every((n, i) => Object.hasOwn(m.groups,
                            n) && Object.hasOwn(m.indices.groups, n) && m.groups[n] === m[i + 1] &&
                            m.indices.groups[n] === m.indices[i + 1]), m.groups === undefined ||
                        Object.getPrototypeOf(m.groups) === null
                    ]
                }
                var P =
                    "(?<b>(?<w>\\b)|(?<nw>\\B))(?<whole>(?<c>[\\q{|a|ab}]))(?<a>(?<s>^)|(?<end>$)|(?<wb>\\b)|(?<nwb>\\B))";
                var calls = [],
                    r = new RegExp(P, "vg"),
                    out = "ab!aa".replace(r, function() {
                        var a = Array.from(arguments);
                        calls.push(a.map(v => v === undefined ? "<undefined>" : typeof v ===
                            "object" ? Object.keys(v).map(k => [k, v[k] === undefined ?
                                "<undefined>" : v[k]
                            ]) : v));
                        return "[" + a[0] + "]"
                    });
                return JSON.stringify([out, calls, r.lastIndex]);
            })() ===
            "[\"[ab][]![a][a][]\",[[\"ab\",\"\",\"\",\"<undefined>\",\"ab\",\"ab\",\"\",\"<undefined>\",\"<undefined>\",\"\",\"<undefined>\",0,\"ab!aa\",[[\"b\",\"\"],[\"w\",\"\"],[\"nw\",\"<undefined>\"],[\"whole\",\"ab\"],[\"c\",\"ab\"],[\"a\",\"\"],[\"s\",\"<undefined>\"],[\"end\",\"<undefined>\"],[\"wb\",\"\"],[\"nwb\",\"<undefined>\"]]],[\"\",\"\",\"\",\"<undefined>\",\"\",\"\",\"\",\"<undefined>\",\"<undefined>\",\"\",\"<undefined>\",2,\"ab!aa\",[[\"b\",\"\"],[\"w\",\"\"],[\"nw\",\"<undefined>\"],[\"whole\",\"\"],[\"c\",\"\"],[\"a\",\"\"],[\"s\",\"<undefined>\"],[\"end\",\"<undefined>\"],[\"wb\",\"\"],[\"nwb\",\"<undefined>\"]]],[\"a\",\"\",\"\",\"<undefined>\",\"a\",\"a\",\"\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"\",3,\"ab!aa\",[[\"b\",\"\"],[\"w\",\"\"],[\"nw\",\"<undefined>\"],[\"whole\",\"a\"],[\"c\",\"a\"],[\"a\",\"\"],[\"s\",\"<undefined>\"],[\"end\",\"<undefined>\"],[\"wb\",\"<undefined>\"],[\"nwb\",\"\"]]],[\"a\",\"\",\"<undefined>\",\"\",\"a\",\"a\",\"\",\"<undefined>\",\"\",\"<undefined>\",\"<undefined>\",4,\"ab!aa\",[[\"b\",\"\"],[\"w\",\"<undefined>\"],[\"nw\",\"\"],[\"whole\",\"a\"],[\"c\",\"a\"],[\"a\",\"\"],[\"s\",\"<undefined>\"],[\"end\",\"\"],[\"wb\",\"<undefined>\"],[\"nwb\",\"<undefined>\"]]],[\"\",\"\",\"\",\"<undefined>\",\"\",\"\",\"\",\"<undefined>\",\"\",\"<undefined>\",\"<undefined>\",5,\"ab!aa\",[[\"b\",\"\"],[\"w\",\"\"],[\"nw\",\"<undefined>\"],[\"whole\",\"\"],[\"c\",\"\"],[\"a\",\"\"],[\"s\",\"<undefined>\"],[\"end\",\"\"],[\"wb\",\"<undefined>\"],[\"nwb\",\"<undefined>\"]]]],0]"
        "#,
    );
    assert_true(
        r#"
            (function() {
                function row(m) {
                    if (m === null) return null;
                    var names = m.groups === undefined ? [] : Object.keys(m.groups);
                    return [Array.from(m, v => v === undefined ? '<undefined>' : v), Array.from(m
                            .indices, v => v === undefined ? '<undefined>' : v), m.index, m.input,
                        names, names.map(n => m.groups[n] === undefined ? '<undefined>' : m.groups[
                            n]), names.map(n => m.indices.groups[n] === undefined ? '<undefined>' :
                            m.indices.groups[n]), names.every((n, i) => Object.hasOwn(m.groups,
                            n) && Object.hasOwn(m.indices.groups, n) && m.groups[n] === m[i + 1] &&
                            m.indices.groups[n] === m.indices[i + 1]), m.groups === undefined ||
                        Object.getPrototypeOf(m.groups) === null
                    ]
                }
                var P =
                    "(?<b>(?<w>\\b)|(?<nw>\\B))(?<whole>(?<c>[\\q{|a|ab}]))(?<a>(?<s>^)|(?<end>$)|(?<wb>\\b)|(?<nwb>\\B))";
                var r = new RegExp(P, "dvg");
                r.lastIndex = 9;
                var a = "!abaa!".split(r),
                    b = "!abaa!".search(r);
                return JSON.stringify([a.map(v => v === undefined ? "<undefined>" : v), b, r
                    .lastIndex]);
            })() ===
            "[[\"!\",\"\",\"\",\"<undefined>\",\"ab\",\"ab\",\"\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"\",\"\",\"\",\"<undefined>\",\"\",\"a\",\"a\",\"\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"\",\"\",\"\",\"<undefined>\",\"\",\"a\",\"a\",\"\",\"<undefined>\",\"<undefined>\",\"\",\"<undefined>\",\"!\"],0,9]"
        "#,
    );
    assert_true(
        r#"
            (function() {
                function row(m) {
                    if (m === null) return null;
                    var names = m.groups === undefined ? [] : Object.keys(m.groups);
                    return [Array.from(m, v => v === undefined ? '<undefined>' : v), Array.from(m
                            .indices, v => v === undefined ? '<undefined>' : v), m.index, m.input,
                        names, names.map(n => m.groups[n] === undefined ? '<undefined>' : m.groups[
                            n]), names.map(n => m.indices.groups[n] === undefined ? '<undefined>' :
                            m.indices.groups[n]), names.every((n, i) => Object.hasOwn(m.groups,
                            n) && Object.hasOwn(m.indices.groups, n) && m.groups[n] === m[i + 1] &&
                            m.indices.groups[n] === m.indices[i + 1]), m.groups === undefined ||
                        Object.getPrototypeOf(m.groups) === null
                    ]
                }
                var P =
                    "(?<b>(?<w>\\b)|(?<nw>\\B))(?<whole>(?<c>[\\q{|a|ab}]))(?<a>(?<s>^)|(?<end>$)|(?<wb>\\b)|(?<nwb>\\B))";
                var r = new RegExp(P, "vg");
                Object.defineProperty(r, "lastIndex", {
                    writable: false
                });
                var caught = false;
                try {
                    r.exec("ab")
                } catch (e) {
                    caught = e instanceof TypeError
                }
                return JSON.stringify([caught, r.lastIndex]);
            })() === "[true,0]"
        "#,
    );
    assert_true(
        r#"
            (function() {
                function row(m) {
                    if (m === null) return null;
                    var names = m.groups === undefined ? [] : Object.keys(m.groups);
                    return [Array.from(m, v => v === undefined ? '<undefined>' : v), Array.from(m
                            .indices, v => v === undefined ? '<undefined>' : v), m.index, m.input,
                        names, names.map(n => m.groups[n] === undefined ? '<undefined>' : m.groups[
                            n]), names.map(n => m.indices.groups[n] === undefined ? '<undefined>' :
                            m.indices.groups[n]), names.every((n, i) => Object.hasOwn(m.groups,
                            n) && Object.hasOwn(m.indices.groups, n) && m.groups[n] === m[i + 1] &&
                            m.indices.groups[n] === m.indices[i + 1]), m.groups === undefined ||
                        Object.getPrototypeOf(m.groups) === null
                    ]
                }
                var P =
                    "(?<b>(?<w>\\b)|(?<nw>\\B))(?<whole>(?<c>[\\q{|a|ab}]))(?<a>(?<s>^)|(?<end>$)|(?<wb>\\b)|(?<nwb>\\B))";
                var r = new RegExp(P, "dvg"),
                    a = r.exec(""),
                    b = r.exec("ab");
                return JSON.stringify([row(a), row(b), r.lastIndex]);
            })() ===
            "[[[\"\",\"\",\"<undefined>\",\"\",\"\",\"\",\"\",\"\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],[[0,0],[0,0],\"<undefined>\",[0,0],[0,0],[0,0],[0,0],[0,0],\"<undefined>\",\"<undefined>\",\"<undefined>\"],0,\"\",[\"b\",\"w\",\"nw\",\"whole\",\"c\",\"a\",\"s\",\"end\",\"wb\",\"nwb\"],[\"\",\"<undefined>\",\"\",\"\",\"\",\"\",\"\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],[[0,0],\"<undefined>\",[0,0],[0,0],[0,0],[0,0],[0,0],\"<undefined>\",\"<undefined>\",\"<undefined>\"],true,true],[[\"ab\",\"\",\"\",\"<undefined>\",\"ab\",\"ab\",\"\",\"<undefined>\",\"\",\"<undefined>\",\"<undefined>\"],[[0,2],[0,0],[0,0],\"<undefined>\",[0,2],[0,2],[2,2],\"<undefined>\",[2,2],\"<undefined>\",\"<undefined>\"],0,\"ab\",[\"b\",\"w\",\"nw\",\"whole\",\"c\",\"a\",\"s\",\"end\",\"wb\",\"nwb\"],[\"\",\"\",\"<undefined>\",\"ab\",\"ab\",\"\",\"<undefined>\",\"\",\"<undefined>\",\"<undefined>\"],[[0,0],[0,0],\"<undefined>\",[0,2],[0,2],[2,2],\"<undefined>\",[2,2],\"<undefined>\",\"<undefined>\"],true,true],2]"
        "#,
    );
    assert_true(
        r#"
            (function() {
                function row(m) {
                    if (m === null) return null;
                    var names = m.groups === undefined ? [] : Object.keys(m.groups);
                    return [Array.from(m, v => v === undefined ? '<undefined>' : v), Array.from(m
                            .indices, v => v === undefined ? '<undefined>' : v), m.index, m.input,
                        names, names.map(n => m.groups[n] === undefined ? '<undefined>' : m.groups[
                            n]), names.map(n => m.indices.groups[n] === undefined ? '<undefined>' :
                            m.indices.groups[n]), names.every((n, i) => Object.hasOwn(m.groups,
                            n) && Object.hasOwn(m.indices.groups, n) && m.groups[n] === m[i + 1] &&
                            m.indices.groups[n] === m.indices[i + 1]), m.groups === undefined ||
                        Object.getPrototypeOf(m.groups) === null
                    ]
                }
                var P =
                    "(?<b>(?<w>\\b)|(?<nw>\\B))(?<whole>(?<c>[a-z\\q{|S|sa|aX}]))(?<a>(?<s>^)|(?<end>$)|(?<wb>\\b)|(?<nwb>\\B))";
                var r = new RegExp(P, "dvgi"),
                    a = r.exec("ſaX!");
                return JSON.stringify([row(a), r.lastIndex]);
            })() ===
            "[[[\"\u017fa\",\"\",\"\",\"<undefined>\",\"\u017fa\",\"\u017fa\",\"\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"\"],[[0,2],[0,0],[0,0],\"<undefined>\",[0,2],[0,2],[2,2],\"<undefined>\",\"<undefined>\",\"<undefined>\",[2,2]],0,\"\u017faX!\",[\"b\",\"w\",\"nw\",\"whole\",\"c\",\"a\",\"s\",\"end\",\"wb\",\"nwb\"],[\"\",\"\",\"<undefined>\",\"\u017fa\",\"\u017fa\",\"\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"\"],[[0,0],[0,0],\"<undefined>\",[0,2],[0,2],[2,2],\"<undefined>\",\"<undefined>\",\"<undefined>\",[2,2]],true,true],2]"
        "#,
    );
    assert_true(
        r#"
            (function() {
                function row(m) {
                    if (m === null) return null;
                    var names = m.groups === undefined ? [] : Object.keys(m.groups);
                    return [Array.from(m, v => v === undefined ? '<undefined>' : v), Array.from(m
                            .indices, v => v === undefined ? '<undefined>' : v), m.index, m.input,
                        names, names.map(n => m.groups[n] === undefined ? '<undefined>' : m.groups[
                            n]), names.map(n => m.indices.groups[n] === undefined ? '<undefined>' :
                            m.indices.groups[n]), names.every((n, i) => Object.hasOwn(m.groups,
                            n) && Object.hasOwn(m.indices.groups, n) && m.groups[n] === m[i + 1] &&
                            m.indices.groups[n] === m.indices[i + 1]), m.groups === undefined ||
                        Object.getPrototypeOf(m.groups) === null
                    ]
                }
                var P =
                    "(?<b>(?<w>\\b)|(?<nw>\\B))(?<whole>(?<c>[\\q{|a|ab}]))(?<a>(?<s>^)|(?<end>$)|(?<wb>\\b)|(?<nwb>\\B))";
                var r = new RegExp(P, "dvgm"),
                    a = Array.from("ab\n\naa".matchAll(r));
                return JSON.stringify([a.map(row), r.lastIndex]);
            })() ===
            "[[[[\"ab\",\"\",\"\",\"<undefined>\",\"ab\",\"ab\",\"\",\"<undefined>\",\"\",\"<undefined>\",\"<undefined>\"],[[0,2],[0,0],[0,0],\"<undefined>\",[0,2],[0,2],[2,2],\"<undefined>\",[2,2],\"<undefined>\",\"<undefined>\"],0,\"ab\\n\\naa\",[\"b\",\"w\",\"nw\",\"whole\",\"c\",\"a\",\"s\",\"end\",\"wb\",\"nwb\"],[\"\",\"\",\"<undefined>\",\"ab\",\"ab\",\"\",\"<undefined>\",\"\",\"<undefined>\",\"<undefined>\"],[[0,0],[0,0],\"<undefined>\",[0,2],[0,2],[2,2],\"<undefined>\",[2,2],\"<undefined>\",\"<undefined>\"],true,true],[[\"\",\"\",\"\",\"<undefined>\",\"\",\"\",\"\",\"<undefined>\",\"\",\"<undefined>\",\"<undefined>\"],[[2,2],[2,2],[2,2],\"<undefined>\",[2,2],[2,2],[2,2],\"<undefined>\",[2,2],\"<undefined>\",\"<undefined>\"],2,\"ab\\n\\naa\",[\"b\",\"w\",\"nw\",\"whole\",\"c\",\"a\",\"s\",\"end\",\"wb\",\"nwb\"],[\"\",\"\",\"<undefined>\",\"\",\"\",\"\",\"<undefined>\",\"\",\"<undefined>\",\"<undefined>\"],[[2,2],[2,2],\"<undefined>\",[2,2],[2,2],[2,2],\"<undefined>\",[2,2],\"<undefined>\",\"<undefined>\"],true,true],[[\"\",\"\",\"<undefined>\",\"\",\"\",\"\",\"\",\"\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],[[3,3],[3,3],\"<undefined>\",[3,3],[3,3],[3,3],[3,3],[3,3],\"<undefined>\",\"<undefined>\",\"<undefined>\"],3,\"ab\\n\\naa\",[\"b\",\"w\",\"nw\",\"whole\",\"c\",\"a\",\"s\",\"end\",\"wb\",\"nwb\"],[\"\",\"<undefined>\",\"\",\"\",\"\",\"\",\"\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],[[3,3],\"<undefined>\",[3,3],[3,3],[3,3],[3,3],[3,3],\"<undefined>\",\"<undefined>\",\"<undefined>\"],true,true],[[\"a\",\"\",\"\",\"<undefined>\",\"a\",\"a\",\"\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"\"],[[4,5],[4,4],[4,4],\"<undefined>\",[4,5],[4,5],[5,5],\"<undefined>\",\"<undefined>\",\"<undefined>\",[5,5]],4,\"ab\\n\\naa\",[\"b\",\"w\",\"nw\",\"whole\",\"c\",\"a\",\"s\",\"end\",\"wb\",\"nwb\"],[\"\",\"\",\"<undefined>\",\"a\",\"a\",\"\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"\"],[[4,4],[4,4],\"<undefined>\",[4,5],[4,5],[5,5],\"<undefined>\",\"<undefined>\",\"<undefined>\",[5,5]],true,true],[[\"a\",\"\",\"<undefined>\",\"\",\"a\",\"a\",\"\",\"<undefined>\",\"\",\"<undefined>\",\"<undefined>\"],[[5,6],[5,5],\"<undefined>\",[5,5],[5,6],[5,6],[6,6],\"<undefined>\",[6,6],\"<undefined>\",\"<undefined>\"],5,\"ab\\n\\naa\",[\"b\",\"w\",\"nw\",\"whole\",\"c\",\"a\",\"s\",\"end\",\"wb\",\"nwb\"],[\"\",\"<undefined>\",\"\",\"a\",\"a\",\"\",\"<undefined>\",\"\",\"<undefined>\",\"<undefined>\"],[[5,5],\"<undefined>\",[5,5],[5,6],[5,6],[6,6],\"<undefined>\",[6,6],\"<undefined>\",\"<undefined>\"],true,true],[[\"\",\"\",\"\",\"<undefined>\",\"\",\"\",\"\",\"<undefined>\",\"\",\"<undefined>\",\"<undefined>\"],[[6,6],[6,6],[6,6],\"<undefined>\",[6,6],[6,6],[6,6],\"<undefined>\",[6,6],\"<undefined>\",\"<undefined>\"],6,\"ab\\n\\naa\",[\"b\",\"w\",\"nw\",\"whole\",\"c\",\"a\",\"s\",\"end\",\"wb\",\"nwb\"],[\"\",\"\",\"<undefined>\",\"\",\"\",\"\",\"<undefined>\",\"\",\"<undefined>\",\"<undefined>\"],[[6,6],[6,6],\"<undefined>\",[6,6],[6,6],[6,6],\"<undefined>\",[6,6],\"<undefined>\",\"<undefined>\"],true,true]],0]"
        "#,
    );
    assert_true(
        r#"
            (function() {
                function row(m) {
                    if (m === null) return null;
                    var names = m.groups === undefined ? [] : Object.keys(m.groups);
                    return [Array.from(m, v => v === undefined ? '<undefined>' : v), Array.from(m
                            .indices, v => v === undefined ? '<undefined>' : v), m.index, m.input,
                        names, names.map(n => m.groups[n] === undefined ? '<undefined>' : m.groups[
                            n]), names.map(n => m.indices.groups[n] === undefined ? '<undefined>' :
                            m.indices.groups[n]), names.every((n, i) => Object.hasOwn(m.groups,
                            n) && Object.hasOwn(m.indices.groups, n) && m.groups[n] === m[i + 1] &&
                            m.indices.groups[n] === m.indices[i + 1]), m.groups === undefined ||
                        Object.getPrototypeOf(m.groups) === null
                    ]
                }
                var P =
                    "(?<b>(?<w>\\b)|(?<nw>\\B))(?<whole>(?<c>[\\q{|a|ab}]))(?<a>(?<s>^)|(?<end>$)|(?<wb>\\b)|(?<nwb>\\B))";
                var r = new RegExp(P, "dvg"),
                    s = new RegExp(r);
                r.lastIndex = 3;
                var a = s.exec("ab!");
                return JSON.stringify([row(a), s.source === r.source, s.flags === r.flags, s.lastIndex,
                    r.lastIndex
                ]);
            })() ===
            "[[[\"ab\",\"\",\"\",\"<undefined>\",\"ab\",\"ab\",\"\",\"<undefined>\",\"<undefined>\",\"\",\"<undefined>\"],[[0,2],[0,0],[0,0],\"<undefined>\",[0,2],[0,2],[2,2],\"<undefined>\",\"<undefined>\",[2,2],\"<undefined>\"],0,\"ab!\",[\"b\",\"w\",\"nw\",\"whole\",\"c\",\"a\",\"s\",\"end\",\"wb\",\"nwb\"],[\"\",\"\",\"<undefined>\",\"ab\",\"ab\",\"\",\"<undefined>\",\"<undefined>\",\"\",\"<undefined>\"],[[0,0],[0,0],\"<undefined>\",[0,2],[0,2],[2,2],\"<undefined>\",\"<undefined>\",[2,2],\"<undefined>\"],true,true],true,true,2,3]"
        "#,
    );

    let mut realm = Realm::default();
    realm.eval(r"var r=/(?<b>(?<w>\b)|(?<nw>\B))(?<c>[\q{a|ab}])(?<a>(?<end>$)|())/dvg,clone=new RegExp(r),m=r.exec('ab')").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(
        realm.eval(
            r#"
                var n = clone.exec('a!');
                m[0] === 'ab' && m.groups.w === '' && m.groups.nw === undefined && m.groups.c === 'ab' && m
                    .groups.end === '' && m.indices.groups.c === m.indices[4] && m.indices[4][0] === 0 && n
                    .groups.end === undefined && n.groups.a === '' && clone.source === r.source && clone
                    .lastIndex === 1 && r.lastIndex === 2
            "#
        ),
        Ok(Value::Boolean(true))
    );
}

#[test_case(
    r#"(?<n1>(?<n2>\b)|(?<n3>\B))(?<n4>(?<n5>[\q{|a|ab}])){1}(?<n6>^|(?<n7>))"#,
    "dgvi",
    JsString::from("ab"),
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
            Some(""),
            Some(""),
        ],
        capture_ranges: &[
            Some((1, 1)),
            None,
            Some((1, 1)),
            Some((1, 1)),
            Some((1, 1)),
            Some((1, 1)),
            Some((1, 1)),
        ],
        last_index: 1,
    };
    "input_ab_from_1_variant_1"
)]
#[test_case(
    r#"(?<n1>(?<n2>\b)|(?<n3>\B))(?<n4>[\p{Any}\q{|ab}]){0001,0001}?(?<n5>^|(?<n6>))"#,
    "dgv",
    JsString::from("a\n!"),
    0,
    MatchExpectation {
        text: "a",
        range: 0..1,
        captures: &[Some(""), Some(""), None, Some("a"), Some(""), Some("")],
        capture_ranges: &[
            Some((0, 0)),
            Some((0, 0)),
            None,
            Some((0, 1)),
            Some((1, 1)),
            Some((1, 1)),
        ],
        last_index: 1,
    };
    "input_a_from_0_variant_2"
)]
#[test_case(
    r#"(?<n1>(?<n2>\b)|(?<n3>\B))(?<n4>(?<n5>[a-z\q{ab}]){1}){1,1}?(?<n6>^|(?<n7>))"#,
    "dgvim",
    JsString::from("😀a"),
    1,
    MatchExpectation {
        text: "a",
        range: 2..3,
        captures: &[
            Some(""),
            Some(""),
            None,
            Some("a"),
            Some("a"),
            Some(""),
            Some(""),
        ],
        capture_ranges: &[
            Some((2, 2)),
            Some((2, 2)),
            None,
            Some((2, 3)),
            Some((2, 3)),
            Some((3, 3)),
            Some((3, 3)),
        ],
        last_index: 3,
    };
    "unicode_input_from_1_variant_3"
)]
#[test_case(
    r#"(?<n1>(?<n2>\B)|(?<n3>\b)){1}(?<n4>(?<n5>[\q{|a|ab}])){1}(?<n6>$|(?<n7>))"#,
    "dgv",
    JsString::from("!ab!"),
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
            Some(""),
            Some(""),
        ],
        capture_ranges: &[
            Some((0, 0)),
            Some((0, 0)),
            None,
            Some((0, 0)),
            Some((0, 0)),
            Some((0, 0)),
            Some((0, 0)),
        ],
        last_index: 0,
    };
    "input_ab_from_0_variant_4"
)]
#[test_case(
    r#"(?<n1>(?<n2>\B)|(?<n3>\b)){1}(?<n4>[\p{Any}\q{|ab}]){0001,0001}?(?<n5>$|(?<n6>))"#,
    "dgvim",
    JsString::from("ſa"),
    1,
    MatchExpectation {
        text: "a",
        range: 1..2,
        captures: &[Some(""), Some(""), None, Some("a"), Some(""), None],
        capture_ranges: &[
            Some((1, 1)),
            Some((1, 1)),
            None,
            Some((1, 2)),
            Some((2, 2)),
            None,
        ],
        last_index: 2,
    };
    "unicode_input_from_1_variant_5"
)]
#[test_case(
    r#"(?<n1>(?<n2>\B)|(?<n3>\b)){1}(?<n4>(?<n5>[a-z\q{ab}]){1}){1,1}?(?<n6>$|(?<n7>))"#,
    "dgvy",
    JsString::from("ab\r\n"),
    0,
    MatchExpectation {
        text: "ab",
        range: 0..2,
        captures: &[
            Some(""),
            None,
            Some(""),
            Some("ab"),
            Some("ab"),
            Some(""),
            Some(""),
        ],
        capture_ranges: &[
            Some((0, 0)),
            None,
            Some((0, 0)),
            Some((0, 2)),
            Some((0, 2)),
            Some((2, 2)),
            Some((2, 2)),
        ],
        last_index: 2,
    };
    "input_ab_from_0_variant_6"
)]
#[test_case(
    r#"(?<n1>\b|(?<n2>)){2,3}?(?<n3>[\q{a|ab}])(?<n4>(?<n5>^)|(?<n6>$)|(?<n7>\b)|(?<n8>\B))"#,
    "dgv",
    JsString::from("ab"),
    0,
    MatchExpectation {
        text: "ab",
        range: 0..2,
        captures: &[
            Some(""),
            None,
            Some("ab"),
            Some(""),
            None,
            Some(""),
            None,
            None,
        ],
        capture_ranges: &[
            Some((0, 0)),
            None,
            Some((0, 2)),
            Some((2, 2)),
            None,
            Some((2, 2)),
            None,
            None,
        ],
        last_index: 2,
    };
    "input_ab_from_0_variant_7"
)]
#[test_case(
    r#"(?<n1>\b|(?<n2>)){2,3}?(?<n3>(?<n4>[\q{|a|ab}])){1}(?<n5>(?<n6>^)|(?<n7>$)|(?<n8>\b)|(?<n9>\B))"#,
    "dgvim",
    JsString::from("a\n!"),
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
            Some(""),
            None,
            None,
        ],
        capture_ranges: &[
            Some((1, 1)),
            None,
            Some((1, 1)),
            Some((1, 1)),
            Some((1, 1)),
            None,
            Some((1, 1)),
            None,
            None,
        ],
        last_index: 1,
    };
    "input_a_from_1_variant_8"
)]
#[test_case(
    r#"(?<n1>\b|(?<n2>)){2,3}?(?<n3>[\p{Any}\q{|ab}]){0001,0001}?(?<n4>(?<n5>^)|(?<n6>$)|(?<n7>\b)|(?<n8>\B))"#,
    "dgvy",
    JsString::from("😀a"),
    0,
    MatchExpectation {
        text: "😀",
        range: 0..2,
        captures: &[
            Some(""),
            Some(""),
            Some("😀"),
            Some(""),
            None,
            None,
            Some(""),
            None,
        ],
        capture_ranges: &[
            Some((0, 0)),
            Some((0, 0)),
            Some((0, 2)),
            Some((2, 2)),
            None,
            None,
            Some((2, 2)),
            None,
        ],
        last_index: 2,
    };
    "unicode_input_from_0_variant_9"
)]
#[test_case(
    r#"(?<n1>^|(?<n2>))(?<n3>[\q{a|ab}])(?<n4>(?<n5>(?<n6>\b)|(?<n7>\B)){2,3})"#,
    "dgvim",
    JsString::from("!ab!"),
    1,
    MatchExpectation {
        text: "ab",
        range: 1..3,
        captures: &[
            Some(""),
            Some(""),
            Some("ab"),
            Some(""),
            Some(""),
            Some(""),
            None,
        ],
        capture_ranges: &[
            Some((1, 1)),
            Some((1, 1)),
            Some((1, 3)),
            Some((3, 3)),
            Some((3, 3)),
            Some((3, 3)),
            None,
        ],
        last_index: 3,
    };
    "input_ab_from_1_variant_10"
)]
#[test_case(
    r#"(?<n1>^|(?<n2>))(?<n3>(?<n4>[\q{|a|ab}])){1}(?<n5>(?<n6>(?<n7>\b)|(?<n8>\B)){2,3})"#,
    "dgvy",
    JsString::from("ſa"),
    0,
    MatchExpectation {
        text: "",
        range: 0..0,
        captures: &[
            Some(""),
            None,
            Some(""),
            Some(""),
            Some(""),
            Some(""),
            None,
            Some(""),
        ],
        capture_ranges: &[
            Some((0, 0)),
            None,
            Some((0, 0)),
            Some((0, 0)),
            Some((0, 0)),
            Some((0, 0)),
            None,
            Some((0, 0)),
        ],
        last_index: 0,
    };
    "unicode_input_from_0_variant_11"
)]
#[test_case(
    r#"(?<n1>^|(?<n2>))(?<n3>[\p{Any}\q{|ab}]){0001,0001}?(?<n4>(?<n5>(?<n6>\b)|(?<n7>\B)){2,3})"#,
    "dgvi",
    JsString::from("ab\r\n"),
    1,
    MatchExpectation {
        text: "b",
        range: 1..2,
        captures: &[
            Some(""),
            Some(""),
            Some("b"),
            Some(""),
            Some(""),
            Some(""),
            None,
        ],
        capture_ranges: &[
            Some((1, 1)),
            Some((1, 1)),
            Some((1, 2)),
            Some((2, 2)),
            Some((2, 2)),
            Some((2, 2)),
            None,
        ],
        last_index: 2,
    };
    "input_ab_from_1_variant_12"
)]
#[test_case(
    r#"(?<n1>^|(?<n2>))(?<n3>(?<n4>[a-z\q{ab}]){1}){1,1}?(?<n5>(?<n6>(?<n7>\b)|(?<n8>\B)){2,3})"#,
    "dgvm",
    JsString::from("a"),
    0,
    MatchExpectation {
        text: "a",
        range: 0..1,
        captures: &[
            Some(""),
            None,
            Some("a"),
            Some("a"),
            Some(""),
            Some(""),
            Some(""),
            None,
        ],
        capture_ranges: &[
            Some((0, 0)),
            None,
            Some((0, 1)),
            Some((0, 1)),
            Some((1, 1)),
            Some((1, 1)),
            Some((1, 1)),
            None,
        ],
        last_index: 1,
    };
    "input_a_from_0_variant_13"
)]
#[test_case(
    r#"(?<n1>$|(?<n2>))(?<n3>[\q{a|ab}])(?<n4>(?<n5>(?<n6>^)|(?<n7>)){0,3})"#,
    "dgvy",
    JsString::from("a\n!"),
    0,
    MatchExpectation {
        text: "a",
        range: 0..1,
        captures: &[Some(""), Some(""), Some("a"), Some(""), None, None, None],
        capture_ranges: &[
            Some((0, 0)),
            Some((0, 0)),
            Some((0, 1)),
            Some((1, 1)),
            None,
            None,
            None,
        ],
        last_index: 1,
    };
    "input_a_from_0_variant_14"
)]
#[test_case(
    r#"(?<n1>$|(?<n2>))(?<n3>(?<n4>[\q{|a|ab}])){1}(?<n5>(?<n6>(?<n7>^)|(?<n8>)){0,3})"#,
    "dgvi",
    JsString::from("😀a"),
    1,
    MatchExpectation {
        text: "",
        range: 0..0,
        captures: &[
            Some(""),
            Some(""),
            Some(""),
            Some(""),
            Some(""),
            None,
            None,
            None,
        ],
        capture_ranges: &[
            Some((0, 0)),
            Some((0, 0)),
            Some((0, 0)),
            Some((0, 0)),
            Some((0, 0)),
            None,
            None,
            None,
        ],
        last_index: 0,
    };
    "unicode_input_from_1_variant_15"
)]
#[test_case(
    r#"(?<n1>$|(?<n2>))(?<n3>[\p{Any}\q{|ab}]){0001,0001}?(?<n4>(?<n5>(?<n6>^)|(?<n7>)){0,3})"#,
    "dgvm",
    JsString::from(""),
    0,
    MatchExpectation {
        text: "",
        range: 0..0,
        captures: &[Some(""), None, Some(""), Some(""), None, None, None],
        capture_ranges: &[
            Some((0, 0)),
            None,
            Some((0, 0)),
            Some((0, 0)),
            None,
            None,
            None,
        ],
        last_index: 0,
    };
    "empty_input_from_0_variant_16"
)]
#[test_case(
    r#"(?<n1>$|(?<n2>))(?<n3>(?<n4>[a-z\q{ab}]){1}){1,1}?(?<n5>(?<n6>(?<n7>^)|(?<n8>)){0,3})"#,
    "dgvi",
    JsString::from("ab"),
    1,
    MatchExpectation {
        text: "b",
        range: 1..2,
        captures: &[
            Some(""),
            Some(""),
            Some("b"),
            Some("b"),
            Some(""),
            None,
            None,
            None,
        ],
        capture_ranges: &[
            Some((1, 1)),
            Some((1, 1)),
            Some((1, 2)),
            Some((1, 2)),
            Some((2, 2)),
            None,
            None,
            None,
        ],
        last_index: 2,
    };
    "input_ab_from_1_variant_17"
)]
#[test_case(
    r#"(?<n1>(?<n2>^)|(?<n3>$)|(?<n4>\b)|(?<n5>\B))(?<n6>[\q{a|ab}])(?<n7>(?<n8>\b\B)|(?<n9>))"#,
    "dgvi",
    JsString::from("ſa"),
    1,
    MatchExpectation {
        text: "a",
        range: 1..2,
        captures: &[
            Some(""),
            None,
            None,
            None,
            Some(""),
            Some("a"),
            Some(""),
            None,
            Some(""),
        ],
        capture_ranges: &[
            Some((1, 1)),
            None,
            None,
            None,
            Some((1, 1)),
            Some((1, 2)),
            Some((2, 2)),
            None,
            Some((2, 2)),
        ],
        last_index: 2,
    };
    "unicode_input_from_1_variant_18"
)]
#[test_case(
    r#"(?<n1>(?<n2>^)|(?<n3>$)|(?<n4>\b)|(?<n5>\B))(?<n6>(?<n7>[\q{|a|ab}])){1}(?<n8>(?<n9>\b\B)|(?<n10>))"#,
    "dgvm",
    JsString::from("ab\r\n"),
    0,
    MatchExpectation {
        text: "ab",
        range: 0..2,
        captures: &[
            Some(""),
            Some(""),
            None,
            None,
            None,
            Some("ab"),
            Some("ab"),
            Some(""),
            None,
            Some(""),
        ],
        capture_ranges: &[
            Some((0, 0)),
            Some((0, 0)),
            None,
            None,
            None,
            Some((0, 2)),
            Some((0, 2)),
            Some((2, 2)),
            None,
            Some((2, 2)),
        ],
        last_index: 2,
    };
    "input_ab_from_0_variant_19"
)]
#[test_case(
    r#"(?<n1>(?<n2>^)|(?<n3>$)|(?<n4>\b)|(?<n5>\B))(?<n6>[\p{Any}\q{|ab}]){0001,0001}?(?<n7>(?<n8>\b\B)|(?<n9>))"#,
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
            None,
            None,
            Some(""),
            Some(""),
            None,
            Some(""),
        ],
        capture_ranges: &[
            Some((1, 1)),
            None,
            Some((1, 1)),
            None,
            None,
            Some((1, 1)),
            Some((1, 1)),
            None,
            Some((1, 1)),
        ],
        last_index: 1,
    };
    "input_a_from_1_variant_20"
)]
#[test_case(
    r#"(?<n1>(?<n2>(?<n3>\b)|(?<n4>\B)){2,3})(?<n5>[\q{a|ab}])(?<n6>(?<n7>)|(?<n8>\b))"#,
    "dgvm",
    JsString::from("😀a"),
    0,
    MatchExpectation {
        text: "a",
        range: 2..3,
        captures: &[
            Some(""),
            Some(""),
            Some(""),
            None,
            Some("a"),
            Some(""),
            Some(""),
            None,
        ],
        capture_ranges: &[
            Some((2, 2)),
            Some((2, 2)),
            Some((2, 2)),
            None,
            Some((2, 3)),
            Some((3, 3)),
            Some((3, 3)),
            None,
        ],
        last_index: 3,
    };
    "unicode_input_from_0_variant_21"
)]
#[test_case(
    r#"(?<n1>(?<n2>(?<n3>\b)|(?<n4>\B)){2,3})(?<n5>[\p{Any}\q{|ab}]){0001,0001}?(?<n6>(?<n7>)|(?<n8>\b))"#,
    "dgvy",
    JsString::from("ab"),
    0,
    MatchExpectation {
        text: "ab",
        range: 0..2,
        captures: &[
            Some(""),
            Some(""),
            Some(""),
            None,
            Some("ab"),
            Some(""),
            Some(""),
            None,
        ],
        capture_ranges: &[
            Some((0, 0)),
            Some((0, 0)),
            Some((0, 0)),
            None,
            Some((0, 2)),
            Some((2, 2)),
            Some((2, 2)),
            None,
        ],
        last_index: 2,
    };
    "input_ab_from_0_variant_22"
)]
#[test_case(
    r#"(?<n1>(?<n2>(?<n3>^)|(?<n4>)){0,3})(?<n5>(?<n6>[\q{|a|ab}])){1}(?<n7>(?<n8>\b)|(?<n9>\B))"#,
    "dgvy",
    JsString::from("a"),
    0,
    MatchExpectation {
        text: "a",
        range: 0..1,
        captures: &[
            Some(""),
            None,
            None,
            None,
            Some("a"),
            Some("a"),
            Some(""),
            Some(""),
            None,
        ],
        capture_ranges: &[
            Some((0, 0)),
            None,
            None,
            None,
            Some((0, 1)),
            Some((0, 1)),
            Some((1, 1)),
            Some((1, 1)),
            None,
        ],
        last_index: 1,
    };
    "input_a_from_0_variant_23"
)]
#[test_case(
    r#"(?<n1>(?<n2>(?<n3>^)|(?<n4>)){0,3})(?<n5>[\p{Any}\q{|ab}]){0001,0001}?(?<n6>(?<n7>\b)|(?<n8>\B))"#,
    "dgvim",
    JsString::from("!ab!"),
    1,
    MatchExpectation {
        text: "ab",
        range: 1..3,
        captures: &[
            Some(""),
            None,
            None,
            None,
            Some("ab"),
            Some(""),
            Some(""),
            None,
        ],
        capture_ranges: &[
            Some((1, 1)),
            None,
            None,
            None,
            Some((1, 3)),
            Some((3, 3)),
            Some((3, 3)),
            None,
        ],
        last_index: 3,
    };
    "input_ab_from_1_variant_24"
)]
#[test_case(
    r#"(?<n1>(?<n2>(?<n3>^)|(?<n4>)){0,3})(?<n5>(?<n6>[a-z\q{ab}]){1}){1,1}?(?<n7>(?<n8>\b)|(?<n9>\B))"#,
    "dgv",
    JsString::from("ſa"),
    0,
    MatchExpectation {
        text: "a",
        range: 1..2,
        captures: &[
            Some(""),
            None,
            None,
            None,
            Some("a"),
            Some("a"),
            Some(""),
            Some(""),
            None,
        ],
        capture_ranges: &[
            Some((1, 1)),
            None,
            None,
            None,
            Some((1, 2)),
            Some((1, 2)),
            Some((2, 2)),
            Some((2, 2)),
            None,
        ],
        last_index: 2,
    };
    "unicode_input_from_0_variant_25"
)]
#[test_case(
    r#"(?<n1>(?<n2>\b\B)|(?<n3>))(?<n4>(?<n5>[\q{|a|ab}])){1}(?<n6>(?<n7>\B)|(?<n8>\b)){1}"#,
    "dgvim",
    JsString::from("ab"),
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
            Some((1, 1)),
            Some((1, 1)),
            None,
        ],
        last_index: 1,
    };
    "input_ab_from_1_variant_26"
)]
#[test_case(
    r#"(?<n1>(?<n2>\b\B)|(?<n3>))(?<n4>[\p{Any}\q{|ab}]){0001,0001}?(?<n5>(?<n6>\B)|(?<n7>\b)){1}"#,
    "dgv",
    JsString::from("a\n!"),
    0,
    MatchExpectation {
        text: "a",
        range: 0..1,
        captures: &[
            Some(""),
            None,
            Some(""),
            Some("a"),
            Some(""),
            None,
            Some(""),
        ],
        capture_ranges: &[
            Some((0, 0)),
            None,
            Some((0, 0)),
            Some((0, 1)),
            Some((1, 1)),
            None,
            Some((1, 1)),
        ],
        last_index: 1,
    };
    "input_a_from_0_variant_27"
)]
#[test_case(
    r#"(?<n1>(?<n2>\b\B)|(?<n3>))(?<n4>(?<n5>[a-z\q{ab}]){1}){1,1}?(?<n6>(?<n7>\B)|(?<n8>\b)){1}"#,
    "dgvi",
    JsString::from("😀a"),
    1,
    MatchExpectation {
        text: "a",
        range: 2..3,
        captures: &[
            Some(""),
            None,
            Some(""),
            Some("a"),
            Some("a"),
            Some(""),
            None,
            Some(""),
        ],
        capture_ranges: &[
            Some((2, 2)),
            None,
            Some((2, 2)),
            Some((2, 3)),
            Some((2, 3)),
            Some((3, 3)),
            None,
            Some((3, 3)),
        ],
        last_index: 3,
    };
    "unicode_input_from_1_variant_28"
)]
#[test_case(
    r#"(?<n1>(?<n2>)|(?<n3>\b))(?<n4>(?<n5>[\q{|a|ab}])){1}(?<n6>\b|(?<n7>)){2,3}?"#,
    "dgv",
    JsString::from("!ab!"),
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
            Some(""),
            Some(""),
        ],
        capture_ranges: &[
            Some((0, 0)),
            Some((0, 0)),
            None,
            Some((0, 0)),
            Some((0, 0)),
            Some((0, 0)),
            Some((0, 0)),
        ],
        last_index: 0,
    };
    "input_ab_from_0_variant_29"
)]
#[test_case(
    r#"(?<n1>(?<n2>)|(?<n3>\b))(?<n4>[\p{Any}\q{|ab}]){0001,0001}?(?<n5>\b|(?<n6>)){2,3}?"#,
    "dgvi",
    JsString::from("ſa"),
    1,
    MatchExpectation {
        text: "a",
        range: 1..2,
        captures: &[Some(""), Some(""), None, Some("a"), Some(""), None],
        capture_ranges: &[
            Some((1, 1)),
            Some((1, 1)),
            None,
            Some((1, 2)),
            Some((2, 2)),
            None,
        ],
        last_index: 2,
    };
    "unicode_input_from_1_variant_30"
)]
#[test_case(
    r#"(?<n1>(?<n2>)|(?<n3>\b))(?<n4>(?<n5>[a-z\q{ab}]){1}){1,1}?(?<n6>\b|(?<n7>)){2,3}?"#,
    "dgvym",
    JsString::from("ab\r\n"),
    0,
    MatchExpectation {
        text: "ab",
        range: 0..2,
        captures: &[
            Some(""),
            Some(""),
            None,
            Some("ab"),
            Some("ab"),
            Some(""),
            None,
        ],
        capture_ranges: &[
            Some((0, 0)),
            Some((0, 0)),
            None,
            Some((0, 2)),
            Some((0, 2)),
            Some((2, 2)),
            None,
        ],
        last_index: 2,
    };
    "input_ab_from_0_variant_31"
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
fn predicate_choice_single_string_classes_reject_unknown_leaves_before_execution() {
    for pattern in [
        r"((\b)|(\B))([\q{a|ab}])[a]",
        r"((\b)|z|())([\q{a|ab}])",
        r"((\b)|\1|())([\q{a|ab}])",
        r"((\b)|(?=a)|())([\q{a|ab}])",
        r"((\b)|(\B))(()|([\q{ab}&&\p{RGI_Emoji}])){3}",
        r"((\b)|(\B))([\q{a|ab}])(z){0}",
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
