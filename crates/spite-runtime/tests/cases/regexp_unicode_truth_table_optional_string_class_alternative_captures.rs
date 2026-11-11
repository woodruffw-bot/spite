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
            var pattern =
                "(?:(?<n1>(?<n2>$)|(?<n3>\\B))(?<n4>(?<n5>){2,3})(?<n6>(?<n7>\\b\\B)|(?<n8>\\b\\B))|(?<n9>(?<n10>^)|(?<n11>$)){3}(?<n12>(?<n13>[a-z\\q{ab}]){1}){1,1}?(?<n14>(?<n15>^)|(?<n16>\\B))){0000,0001}?",
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
                "(?:(?<n1>(?<n2>$)|(?<n3>\\b)){2,3}?(?<n4>(?<n5>){2,3})(?<n6>(?<n7>\\b^)|(?<n8>$))|(?<n9>(?<n10>(?<n11>^)|(?<n12>\\b))\\B)(?<n13>(?<n14>[\\q{|a|ab}])){1}(?<n15>(?<n16>\\b\\B)|(?<n17>\\b\\B)))??",
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
                    "(?<outer>(?<b>(?<s>^)|(?<w>\\b))(?<whole>(?<c>[\\q{|a|ab}]))(?<a>(?<end>$)|(?<nw>\\B))|(?<fallback>(?<wb>\\b)|(?<nwb>\\B))(?<empty>))?";
                var r = new RegExp(P, "dvg"),
                    a = Array.from("ab!aa".matchAll(r));
                return JSON.stringify([a.map(row), r.lastIndex]);
            })() ===
            "[[[[\"a\",\"a\",\"\",\"\",\"<undefined>\",\"a\",\"a\",\"\",\"<undefined>\",\"\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],[[0,1],[0,1],[0,0],[0,0],\"<undefined>\",[0,1],[0,1],[1,1],\"<undefined>\",[1,1],\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],0,\"ab!aa\",[\"outer\",\"b\",\"s\",\"w\",\"whole\",\"c\",\"a\",\"end\",\"nw\",\"fallback\",\"wb\",\"nwb\",\"empty\"],[\"a\",\"\",\"\",\"<undefined>\",\"a\",\"a\",\"\",\"<undefined>\",\"\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],[[0,1],[0,0],[0,0],\"<undefined>\",[0,1],[0,1],[1,1],\"<undefined>\",[1,1],\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],true,true],[[\"\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],[[1,1],\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],1,\"ab!aa\",[\"outer\",\"b\",\"s\",\"w\",\"whole\",\"c\",\"a\",\"end\",\"nw\",\"fallback\",\"wb\",\"nwb\",\"empty\"],[\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],[\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],true,true],[[\"\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],[[2,2],\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],2,\"ab!aa\",[\"outer\",\"b\",\"s\",\"w\",\"whole\",\"c\",\"a\",\"end\",\"nw\",\"fallback\",\"wb\",\"nwb\",\"empty\"],[\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],[\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],true,true],[[\"a\",\"a\",\"\",\"<undefined>\",\"\",\"a\",\"a\",\"\",\"<undefined>\",\"\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],[[3,4],[3,4],[3,3],\"<undefined>\",[3,3],[3,4],[3,4],[4,4],\"<undefined>\",[4,4],\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],3,\"ab!aa\",[\"outer\",\"b\",\"s\",\"w\",\"whole\",\"c\",\"a\",\"end\",\"nw\",\"fallback\",\"wb\",\"nwb\",\"empty\"],[\"a\",\"\",\"<undefined>\",\"\",\"a\",\"a\",\"\",\"<undefined>\",\"\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],[[3,4],[3,3],\"<undefined>\",[3,3],[3,4],[3,4],[4,4],\"<undefined>\",[4,4],\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],true,true],[[\"\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],[[4,4],\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],4,\"ab!aa\",[\"outer\",\"b\",\"s\",\"w\",\"whole\",\"c\",\"a\",\"end\",\"nw\",\"fallback\",\"wb\",\"nwb\",\"empty\"],[\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],[\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],true,true],[[\"\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],[[5,5],\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],5,\"ab!aa\",[\"outer\",\"b\",\"s\",\"w\",\"whole\",\"c\",\"a\",\"end\",\"nw\",\"fallback\",\"wb\",\"nwb\",\"empty\"],[\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],[\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],true,true]],0]"
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
                    "(?<outer>(?<b>(?<s>^)|(?<w>\\b))(?<whole>(?<c>[\\q{|a|ab}]))(?<a>(?<end>$)|(?<nw>\\B))|(?<fallback>(?<wb>\\b)|(?<nwb>\\B))(?<empty>))?";
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
            "[\"[a][]b[]![a][]a[]\",[[\"a\",\"a\",\"\",\"\",\"<undefined>\",\"a\",\"a\",\"\",\"<undefined>\",\"\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",0,\"ab!aa\",[[\"outer\",\"a\"],[\"b\",\"\"],[\"s\",\"\"],[\"w\",\"<undefined>\"],[\"whole\",\"a\"],[\"c\",\"a\"],[\"a\",\"\"],[\"end\",\"<undefined>\"],[\"nw\",\"\"],[\"fallback\",\"<undefined>\"],[\"wb\",\"<undefined>\"],[\"nwb\",\"<undefined>\"],[\"empty\",\"<undefined>\"]]],[\"\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",1,\"ab!aa\",[[\"outer\",\"<undefined>\"],[\"b\",\"<undefined>\"],[\"s\",\"<undefined>\"],[\"w\",\"<undefined>\"],[\"whole\",\"<undefined>\"],[\"c\",\"<undefined>\"],[\"a\",\"<undefined>\"],[\"end\",\"<undefined>\"],[\"nw\",\"<undefined>\"],[\"fallback\",\"<undefined>\"],[\"wb\",\"<undefined>\"],[\"nwb\",\"<undefined>\"],[\"empty\",\"<undefined>\"]]],[\"\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",2,\"ab!aa\",[[\"outer\",\"<undefined>\"],[\"b\",\"<undefined>\"],[\"s\",\"<undefined>\"],[\"w\",\"<undefined>\"],[\"whole\",\"<undefined>\"],[\"c\",\"<undefined>\"],[\"a\",\"<undefined>\"],[\"end\",\"<undefined>\"],[\"nw\",\"<undefined>\"],[\"fallback\",\"<undefined>\"],[\"wb\",\"<undefined>\"],[\"nwb\",\"<undefined>\"],[\"empty\",\"<undefined>\"]]],[\"a\",\"a\",\"\",\"<undefined>\",\"\",\"a\",\"a\",\"\",\"<undefined>\",\"\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",3,\"ab!aa\",[[\"outer\",\"a\"],[\"b\",\"\"],[\"s\",\"<undefined>\"],[\"w\",\"\"],[\"whole\",\"a\"],[\"c\",\"a\"],[\"a\",\"\"],[\"end\",\"<undefined>\"],[\"nw\",\"\"],[\"fallback\",\"<undefined>\"],[\"wb\",\"<undefined>\"],[\"nwb\",\"<undefined>\"],[\"empty\",\"<undefined>\"]]],[\"\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",4,\"ab!aa\",[[\"outer\",\"<undefined>\"],[\"b\",\"<undefined>\"],[\"s\",\"<undefined>\"],[\"w\",\"<undefined>\"],[\"whole\",\"<undefined>\"],[\"c\",\"<undefined>\"],[\"a\",\"<undefined>\"],[\"end\",\"<undefined>\"],[\"nw\",\"<undefined>\"],[\"fallback\",\"<undefined>\"],[\"wb\",\"<undefined>\"],[\"nwb\",\"<undefined>\"],[\"empty\",\"<undefined>\"]]],[\"\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",5,\"ab!aa\",[[\"outer\",\"<undefined>\"],[\"b\",\"<undefined>\"],[\"s\",\"<undefined>\"],[\"w\",\"<undefined>\"],[\"whole\",\"<undefined>\"],[\"c\",\"<undefined>\"],[\"a\",\"<undefined>\"],[\"end\",\"<undefined>\"],[\"nw\",\"<undefined>\"],[\"fallback\",\"<undefined>\"],[\"wb\",\"<undefined>\"],[\"nwb\",\"<undefined>\"],[\"empty\",\"<undefined>\"]]]],0]"
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
                    "(?<outer>(?<b>(?<s>^)|(?<w>\\b))(?<whole>(?<c>[\\q{|a|ab}]))(?<a>(?<end>$)|(?<nw>\\B))|(?<fallback>(?<wb>\\b)|(?<nwb>\\B))(?<empty>))?";
                var r = new RegExp(P, "dvg");
                r.lastIndex = 9;
                var a = "!abaa!".split(r),
                    b = "!abaa!".search(r);
                return JSON.stringify([a.map(v => v === undefined ? "<undefined>" : v), b, r
                    .lastIndex]);
            })() ===
            "[[\"!\",\"ab\",\"\",\"<undefined>\",\"\",\"ab\",\"ab\",\"\",\"<undefined>\",\"\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"a\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"a\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"!\"],0,9]"
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
                    "(?<outer>(?<b>(?<s>^)|(?<w>\\b))(?<whole>(?<c>[\\q{|a|ab}]))(?<a>(?<end>$)|(?<nw>\\B))|(?<fallback>(?<wb>\\b)|(?<nwb>\\B))(?<empty>))?";
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
                    "(?<outer>(?<b>(?<s>^)|(?<w>\\b))(?<whole>(?<c>[\\q{|a|ab}]))(?<a>(?<end>$)|(?<nw>\\B))|(?<fallback>(?<wb>\\b)|(?<nwb>\\B))(?<empty>))?";
                var r = new RegExp(P, "dvg"),
                    a = r.exec(""),
                    b = r.exec("ab");
                return JSON.stringify([row(a), row(b), r.lastIndex]);
            })() ===
            "[[[\"\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],[[0,0],\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],0,\"\",[\"outer\",\"b\",\"s\",\"w\",\"whole\",\"c\",\"a\",\"end\",\"nw\",\"fallback\",\"wb\",\"nwb\",\"empty\"],[\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],[\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],true,true],[[\"ab\",\"ab\",\"\",\"\",\"<undefined>\",\"ab\",\"ab\",\"\",\"\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],[[0,2],[0,2],[0,0],[0,0],\"<undefined>\",[0,2],[0,2],[2,2],[2,2],\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],0,\"ab\",[\"outer\",\"b\",\"s\",\"w\",\"whole\",\"c\",\"a\",\"end\",\"nw\",\"fallback\",\"wb\",\"nwb\",\"empty\"],[\"ab\",\"\",\"\",\"<undefined>\",\"ab\",\"ab\",\"\",\"\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],[[0,2],[0,0],[0,0],\"<undefined>\",[0,2],[0,2],[2,2],[2,2],\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],true,true],2]"
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
                    "(?<outer>(?<b>(?<s>^)|(?<w>\\b))(?<whole>(?<c>[a-z\\q{|S|sa|aX}]))(?<a>(?<end>$)|(?<nw>\\B))|(?<fallback>(?<wb>\\b)|(?<nwb>\\B))(?<empty>))?";
                var r = new RegExp(P, "dvgi"),
                    a = r.exec("ſaX!");
                return JSON.stringify([row(a), r.lastIndex]);
            })() ===
            "[[[\"\u017fa\",\"\u017fa\",\"\",\"\",\"<undefined>\",\"\u017fa\",\"\u017fa\",\"\",\"<undefined>\",\"\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],[[0,2],[0,2],[0,0],[0,0],\"<undefined>\",[0,2],[0,2],[2,2],\"<undefined>\",[2,2],\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],0,\"\u017faX!\",[\"outer\",\"b\",\"s\",\"w\",\"whole\",\"c\",\"a\",\"end\",\"nw\",\"fallback\",\"wb\",\"nwb\",\"empty\"],[\"\u017fa\",\"\",\"\",\"<undefined>\",\"\u017fa\",\"\u017fa\",\"\",\"<undefined>\",\"\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],[[0,2],[0,0],[0,0],\"<undefined>\",[0,2],[0,2],[2,2],\"<undefined>\",[2,2],\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],true,true],2]"
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
                    "(?<outer>(?<b>(?<s>^)|(?<w>\\b))(?<whole>(?<c>[\\q{|a|ab}]))(?<a>(?<end>$)|(?<nw>\\B))|(?<fallback>(?<wb>\\b)|(?<nwb>\\B))(?<empty>))?";
                var r = new RegExp(P, "dvgm"),
                    a = Array.from("ab\n\naa".matchAll(r));
                return JSON.stringify([a.map(row), r.lastIndex]);
            })() ===
            "[[[[\"ab\",\"ab\",\"\",\"\",\"<undefined>\",\"ab\",\"ab\",\"\",\"\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],[[0,2],[0,2],[0,0],[0,0],\"<undefined>\",[0,2],[0,2],[2,2],[2,2],\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],0,\"ab\\n\\naa\",[\"outer\",\"b\",\"s\",\"w\",\"whole\",\"c\",\"a\",\"end\",\"nw\",\"fallback\",\"wb\",\"nwb\",\"empty\"],[\"ab\",\"\",\"\",\"<undefined>\",\"ab\",\"ab\",\"\",\"\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],[[0,2],[0,0],[0,0],\"<undefined>\",[0,2],[0,2],[2,2],[2,2],\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],true,true],[[\"\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],[[2,2],\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],2,\"ab\\n\\naa\",[\"outer\",\"b\",\"s\",\"w\",\"whole\",\"c\",\"a\",\"end\",\"nw\",\"fallback\",\"wb\",\"nwb\",\"empty\"],[\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],[\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],true,true],[[\"\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],[[3,3],\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],3,\"ab\\n\\naa\",[\"outer\",\"b\",\"s\",\"w\",\"whole\",\"c\",\"a\",\"end\",\"nw\",\"fallback\",\"wb\",\"nwb\",\"empty\"],[\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],[\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],true,true],[[\"a\",\"a\",\"\",\"\",\"<undefined>\",\"a\",\"a\",\"\",\"<undefined>\",\"\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],[[4,5],[4,5],[4,4],[4,4],\"<undefined>\",[4,5],[4,5],[5,5],\"<undefined>\",[5,5],\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],4,\"ab\\n\\naa\",[\"outer\",\"b\",\"s\",\"w\",\"whole\",\"c\",\"a\",\"end\",\"nw\",\"fallback\",\"wb\",\"nwb\",\"empty\"],[\"a\",\"\",\"\",\"<undefined>\",\"a\",\"a\",\"\",\"<undefined>\",\"\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],[[4,5],[4,4],[4,4],\"<undefined>\",[4,5],[4,5],[5,5],\"<undefined>\",[5,5],\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],true,true],[[\"\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],[[5,5],\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],5,\"ab\\n\\naa\",[\"outer\",\"b\",\"s\",\"w\",\"whole\",\"c\",\"a\",\"end\",\"nw\",\"fallback\",\"wb\",\"nwb\",\"empty\"],[\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],[\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],true,true],[[\"\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],[[6,6],\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],6,\"ab\\n\\naa\",[\"outer\",\"b\",\"s\",\"w\",\"whole\",\"c\",\"a\",\"end\",\"nw\",\"fallback\",\"wb\",\"nwb\",\"empty\"],[\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],[\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],true,true]],0]"
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
                    "(?<outer>(?<\\u{1D49C}>(?<s>^)|(?<w>\\b))(?<whole>(?<c>[\\q{|a|ab}]))(?<a>(?<end>$)|(?<nw>\\B))|(?<fallback>(?<wb>\\b)|(?<nwb>\\B))(?<empty>))?";
                var r = new RegExp(P, "dvg"),
                    s = new RegExp(r);
                r.lastIndex = 3;
                var a = s.exec("ab!");
                return JSON.stringify([row(a), s.source === r.source, s.flags === r.flags, s.lastIndex,
                    r.lastIndex
                ]);
            })() ===
            "[[[\"a\",\"a\",\"\",\"\",\"<undefined>\",\"a\",\"a\",\"\",\"<undefined>\",\"\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],[[0,1],[0,1],[0,0],[0,0],\"<undefined>\",[0,1],[0,1],[1,1],\"<undefined>\",[1,1],\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],0,\"ab!\",[\"outer\",\"\ud835\udc9c\",\"s\",\"w\",\"whole\",\"c\",\"a\",\"end\",\"nw\",\"fallback\",\"wb\",\"nwb\",\"empty\"],[\"a\",\"\",\"\",\"<undefined>\",\"a\",\"a\",\"\",\"<undefined>\",\"\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],[[0,1],[0,0],[0,0],\"<undefined>\",[0,1],[0,1],[1,1],\"<undefined>\",[1,1],\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],true,true],true,true,1,3]"
        "#,
    );

    let mut realm = Realm::default();
    realm.eval(r"var r=/(?<outer>(?<b>(?<s>^)|(?<w>\b))(?<c>[\q{a|ab}])(?<a>(?<end>$)|(?<nw>\B))|(?<fallback>(?<wb>\b)|(?<nwb>\B))(?<empty>))?/dvg,clone=new RegExp(r),m=r.exec('ab!aa')").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval(r#"
        var n = clone.exec('ab'),
            p = r.exec('!');
        m[0] === 'a' && m.groups.outer === 'a' && m.groups.s === '' && m.groups.w === undefined && m
            .groups.c === 'a' && m.groups.fallback === undefined && m.indices.groups.c === m.indices[
            5] && m.indices[5][0] === 0 && m.indices[5][1] === 1 && n.groups.c === 'ab' && n.groups
            .outer === 'ab' && n.groups.end === '' && clone.source === r.source && clone.lastIndex ===
            2 && p[0] === '' && p.index === 1 && Array.from(p).slice(1).every(v => v === undefined) &&
            p.indices.slice(1).every(v => v === undefined) && p.groups.outer === undefined && p.groups
            .fallback === undefined && p.groups.empty === undefined && r.lastIndex === 1
    "#),Ok(Value::Boolean(true)));
}

#[test_case(
    r#"(?<n1>(?<n2>(?<n3>^)|(?<n4>\b))(?<n5>[\q{a|ab}])(?<n6>(?<n7>\b^)|(?<n8>$))|(?<n9>(?<n10>^)|(?<n11>\B))(?<n12>(?<n13>){2,3})(?<n14>(?<n15>$)|(?<n16>\B)))?"#,
    "dgvym",
    JsString::from(""),
    0,
    MatchExpectation {
        text: "",
        range: 0..0,
        captures: &[None; 16],
        capture_ranges: &[None; 16],
        last_index: 0,
    };
    "empty_input_from_0_variant_1"
)]
#[test_case(
    r#"(?:(?<n1>(?<n2>^)|(?<n3>\B))(?<n4>(?<n5>){2,3})(?<n6>(?<n7>$)|(?<n8>\B))|(?<n9>(?<n10>^)|(?<n11>\b))(?<n12>(?<n13>[\q{|a|ab}])){1}(?<n14>(?<n15>\b^)|(?<n16>$)))??"#,
    "dgvi",
    JsString::from("ab"),
    1,
    MatchExpectation {
        text: "",
        range: 1..1,
        captures: &[None; 16],
        capture_ranges: &[None; 16],
        last_index: 1,
    };
    "input_ab_from_1_variant_2"
)]
#[test_case(
    r#"(?<n1>(?<n2>(?<n3>^)|(?<n4>\b))(?<n5>[\p{Any}\q{|ab}]){0001,0001}?(?<n6>(?<n7>\b^)|(?<n8>$))|(?<n9>(?<n10>^)|(?<n11>\B))(?<n12>(?<n13>){2,3})(?<n14>(?<n15>$)|(?<n16>\B))){0,1}"#,
    "dgv",
    JsString::from("a\n!"),
    0,
    MatchExpectation {
        text: "",
        range: 0..0,
        captures: &[None; 16],
        capture_ranges: &[None; 16],
        last_index: 0,
    };
    "input_a_from_0_variant_3"
)]
#[test_case(
    r#"(?:(?<n1>(?<n2>^)|(?<n3>\B))(?<n4>(?<n5>){2,3})(?<n6>(?<n7>$)|(?<n8>\B))|(?<n9>(?<n10>^)|(?<n11>\b))(?<n12>(?<n13>[a-z\q{ab}]){1}){1,1}?(?<n14>(?<n15>\b^)|(?<n16>$))){0000,0001}?"#,
    "dgvim",
    JsString::from("😀a"),
    1,
    MatchExpectation {
        text: "",
        range: 0..0,
        captures: &[None; 16],
        capture_ranges: &[None; 16],
        last_index: 0,
    };
    "unicode_input_from_1_variant_4"
)]
#[test_case(
    r#"(?<n1>(?<n2>(?<n3>$)|(?<n4>\b)){2,3}?(?<n5>[\q{a|ab}])(?<n6>(?<n7>\B^)|(?<n8>\b$))|(?<n9>(?<n10>(?<n11>^)|(?<n12>\b))\B)(?<n13>(?<n14>){2,3})(?<n15>(?<n16>\b^)|(?<n17>\B$)))?"#,
    "dgvi",
    JsString::from("a"),
    1,
    MatchExpectation {
        text: "",
        range: 1..1,
        captures: &[None; 17],
        capture_ranges: &[None; 17],
        last_index: 1,
    };
    "input_a_from_1_variant_5"
)]
#[test_case(
    r#"(?:(?<n1>(?<n2>(?<n3>^)|(?<n4>\b))\B)(?<n5>(?<n6>){2,3})(?<n7>(?<n8>\b^)|(?<n9>\B$))|(?<n10>(?<n11>$)|(?<n12>\b)){2,3}?(?<n13>(?<n14>[\q{|a|ab}])){1}(?<n15>(?<n16>\B^)|(?<n17>\b$)))??"#,
    "dgv",
    JsString::from("!ab!"),
    0,
    MatchExpectation {
        text: "",
        range: 0..0,
        captures: &[None; 17],
        capture_ranges: &[None; 17],
        last_index: 0,
    };
    "input_ab_from_0_variant_6"
)]
#[test_case(
    r#"(?<n1>(?<n2>(?<n3>$)|(?<n4>\b)){2,3}?(?<n5>[\p{Any}\q{|ab}]){0001,0001}?(?<n6>(?<n7>\B^)|(?<n8>\b$))|(?<n9>(?<n10>(?<n11>^)|(?<n12>\b))\B)(?<n13>(?<n14>){2,3})(?<n15>(?<n16>\b^)|(?<n17>\B$))){0,1}"#,
    "dgvim",
    JsString::from("ſa"),
    1,
    MatchExpectation {
        text: "",
        range: 1..1,
        captures: &[None; 17],
        capture_ranges: &[None; 17],
        last_index: 1,
    };
    "unicode_input_from_1_variant_7"
)]
#[test_case(
    r#"(?:(?<n1>(?<n2>(?<n3>^)|(?<n4>\b))\B)(?<n5>(?<n6>){2,3})(?<n7>(?<n8>\b^)|(?<n9>\B$))|(?<n10>(?<n11>$)|(?<n12>\b)){2,3}?(?<n13>(?<n14>[a-z\q{ab}]){1}){1,1}?(?<n15>(?<n16>\B^)|(?<n17>\b$))){0000,0001}?"#,
    "dgvy",
    JsString::from("ab\r\n"),
    0,
    MatchExpectation {
        text: "",
        range: 0..0,
        captures: &[None; 17],
        capture_ranges: &[None; 17],
        last_index: 0,
    };
    "input_ab_from_0_variant_8"
)]
#[test_case(
    r#"(?<n1>(?<n2>(?<n3>^)|(?<n4>$)){3}(?<n5>[\q{a|ab}])(?<n6>(?<n7>^)|(?<n8>\B))|(?<n9>(?<n10>$)|(?<n11>\B))(?<n12>(?<n13>){2,3})(?<n14>(?<n15>\b\B)|(?<n16>\b\B)))?"#,
    "dgv",
    JsString::from("ab"),
    0,
    MatchExpectation {
        text: "a",
        range: 0..1,
        captures: &[
            Some("a"),
            Some(""),
            Some(""),
            None,
            Some("a"),
            Some(""),
            None,
            Some(""),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        ],
        capture_ranges: &[
            Some((0, 1)),
            Some((0, 0)),
            Some((0, 0)),
            None,
            Some((0, 1)),
            Some((1, 1)),
            None,
            Some((1, 1)),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        ],
        last_index: 1,
    };
    "input_ab_from_0_variant_9"
)]
#[test_case(
    r#"(?:(?<n1>(?<n2>$)|(?<n3>\B))(?<n4>(?<n5>){2,3})(?<n6>(?<n7>\b\B)|(?<n8>\b\B))|(?<n9>(?<n10>^)|(?<n11>$)){3}(?<n12>(?<n13>[\q{|a|ab}])){1}(?<n14>(?<n15>^)|(?<n16>\B)))??"#,
    "dgvim",
    JsString::from("a\n!"),
    1,
    MatchExpectation {
        text: "",
        range: 1..1,
        captures: &[None; 16],
        capture_ranges: &[None; 16],
        last_index: 1,
    };
    "input_a_from_1_variant_10"
)]
#[test_case(
    r#"(?<n1>(?<n2>(?<n3>^)|(?<n4>$)){3}(?<n5>[\p{Any}\q{|ab}]){0001,0001}?(?<n6>(?<n7>^)|(?<n8>\B))|(?<n9>(?<n10>$)|(?<n11>\B))(?<n12>(?<n13>){2,3})(?<n14>(?<n15>\b\B)|(?<n16>\b\B))){0,1}"#,
    "dgvy",
    JsString::from("😀a"),
    0,
    MatchExpectation {
        text: "",
        range: 0..0,
        captures: &[None; 16],
        capture_ranges: &[None; 16],
        last_index: 0,
    };
    "unicode_input_from_0_variant_11"
)]
#[test_case(
    r#"(?<n1>(?<n2>(?<n3>\b^)|(?<n4>$))(?<n5>[\q{a|ab}])(?<n6>(?<n7>(?<n8>^)|(?<n9>\b))\B)|(?<n10>(?<n11>\b^)|(?<n12>\B$))(?<n13>(?<n14>){2,3})(?<n15>(?<n16>^)|(?<n17>\b)))?"#,
    "dgvim",
    JsString::from("!ab!"),
    1,
    MatchExpectation {
        text: "",
        range: 1..1,
        captures: &[None; 17],
        capture_ranges: &[None; 17],
        last_index: 1,
    };
    "input_ab_from_1_variant_12"
)]
#[test_case(
    r#"(?:(?<n1>(?<n2>\b^)|(?<n3>\B$))(?<n4>(?<n5>){2,3})(?<n6>(?<n7>^)|(?<n8>\b))|(?<n9>(?<n10>\b^)|(?<n11>$))(?<n12>(?<n13>[\q{|a|ab}])){1}(?<n14>(?<n15>(?<n16>^)|(?<n17>\b))\B))??"#,
    "dgvy",
    JsString::from("ſa"),
    0,
    MatchExpectation {
        text: "",
        range: 0..0,
        captures: &[None; 17],
        capture_ranges: &[None; 17],
        last_index: 0,
    };
    "unicode_input_from_0_variant_13"
)]
#[test_case(
    r#"(?<n1>(?<n2>(?<n3>\b^)|(?<n4>$))(?<n5>[\p{Any}\q{|ab}]){0001,0001}?(?<n6>(?<n7>(?<n8>^)|(?<n9>\b))\B)|(?<n10>(?<n11>\b^)|(?<n12>\B$))(?<n13>(?<n14>){2,3})(?<n15>(?<n16>^)|(?<n17>\b))){0,1}"#,
    "dgvi",
    JsString::from("ab\r\n"),
    1,
    MatchExpectation {
        text: "",
        range: 1..1,
        captures: &[None; 17],
        capture_ranges: &[None; 17],
        last_index: 1,
    };
    "input_ab_from_1_variant_14"
)]
#[test_case(
    r#"(?:(?<n1>(?<n2>\b^)|(?<n3>\B$))(?<n4>(?<n5>){2,3})(?<n6>(?<n7>^)|(?<n8>\b))|(?<n9>(?<n10>\b^)|(?<n11>$))(?<n12>(?<n13>[a-z\q{ab}]){1}){1,1}?(?<n14>(?<n15>(?<n16>^)|(?<n17>\b))\B)){0000,0001}?"#,
    "dgvm",
    JsString::from("a"),
    0,
    MatchExpectation {
        text: "",
        range: 0..0,
        captures: &[None; 17],
        capture_ranges: &[None; 17],
        last_index: 0,
    };
    "input_a_from_0_variant_15"
)]
#[test_case(
    r#"(?<n1>(?<n2>(?<n3>\B^)|(?<n4>\b$))(?<n5>[\q{a|ab}])(?<n6>(?<n7>$)|(?<n8>\B))|(?<n9>(?<n10>\b\B)|(?<n11>\b\B))(?<n12>(?<n13>){2,3})(?<n14>(?<n15>$)|(?<n16>\b)){2,3}?)?"#,
    "dgvy",
    JsString::from("a\n!"),
    0,
    MatchExpectation {
        text: "",
        range: 0..0,
        captures: &[None; 16],
        capture_ranges: &[None; 16],
        last_index: 0,
    };
    "input_a_from_0_variant_16"
)]
#[test_case(
    r#"(?:(?<n1>(?<n2>\b\B)|(?<n3>\b\B))(?<n4>(?<n5>){2,3})(?<n6>(?<n7>$)|(?<n8>\b)){2,3}?|(?<n9>(?<n10>\B^)|(?<n11>\b$))(?<n12>(?<n13>[\q{|a|ab}])){1}(?<n14>(?<n15>$)|(?<n16>\B)))??"#,
    "dgvi",
    JsString::from("😀a"),
    1,
    MatchExpectation {
        text: "",
        range: 0..0,
        captures: &[None; 16],
        capture_ranges: &[None; 16],
        last_index: 0,
    };
    "unicode_input_from_1_variant_17"
)]
#[test_case(
    r#"(?<n1>(?<n2>(?<n3>\B^)|(?<n4>\b$))(?<n5>[\p{Any}\q{|ab}]){0001,0001}?(?<n6>(?<n7>$)|(?<n8>\B))|(?<n9>(?<n10>\b\B)|(?<n11>\b\B))(?<n12>(?<n13>){2,3})(?<n14>(?<n15>$)|(?<n16>\b)){2,3}?){0,1}"#,
    "dgvm",
    JsString::from(""),
    0,
    MatchExpectation {
        text: "",
        range: 0..0,
        captures: &[None; 16],
        capture_ranges: &[None; 16],
        last_index: 0,
    };
    "empty_input_from_0_variant_18"
)]
#[test_case(
    r#"(?:(?<n1>(?<n2>\b\B)|(?<n3>\b\B))(?<n4>(?<n5>){2,3})(?<n6>(?<n7>$)|(?<n8>\b)){2,3}?|(?<n9>(?<n10>\B^)|(?<n11>\b$))(?<n12>(?<n13>[a-z\q{ab}]){1}){1,1}?(?<n14>(?<n15>$)|(?<n16>\B))){0000,0001}?"#,
    "dgvi",
    JsString::from("ab"),
    1,
    MatchExpectation {
        text: "",
        range: 1..1,
        captures: &[None; 16],
        capture_ranges: &[None; 16],
        last_index: 1,
    };
    "input_ab_from_1_variant_19"
)]
#[test_case(
    r#"(?<n1>(?<n2>(?<n3>^)|(?<n4>\B))(?<n5>[\q{a|ab}])(?<n6>(?<n7>\b^)|(?<n8>\B$))|(?<n9>(?<n10>^)|(?<n11>\b))(?<n12>(?<n13>){2,3})(?<n14>(?<n15>^)|(?<n16>$)){3})?"#,
    "dgvi",
    JsString::from("ſa"),
    1,
    MatchExpectation {
        text: "",
        range: 1..1,
        captures: &[None; 16],
        capture_ranges: &[None; 16],
        last_index: 1,
    };
    "unicode_input_from_1_variant_20"
)]
#[test_case(
    r#"(?:(?<n1>(?<n2>^)|(?<n3>\b))(?<n4>(?<n5>){2,3})(?<n6>(?<n7>^)|(?<n8>$)){3}|(?<n9>(?<n10>^)|(?<n11>\B))(?<n12>(?<n13>[\q{|a|ab}])){1}(?<n14>(?<n15>\b^)|(?<n16>\B$)))??"#,
    "dgvm",
    JsString::from("ab\r\n"),
    0,
    MatchExpectation {
        text: "",
        range: 0..0,
        captures: &[None; 16],
        capture_ranges: &[None; 16],
        last_index: 0,
    };
    "input_ab_from_0_variant_21"
)]
#[test_case(
    r#"(?<n1>(?<n2>(?<n3>^)|(?<n4>\B))(?<n5>[\p{Any}\q{|ab}]){0001,0001}?(?<n6>(?<n7>\b^)|(?<n8>\B$))|(?<n9>(?<n10>^)|(?<n11>\b))(?<n12>(?<n13>){2,3})(?<n14>(?<n15>^)|(?<n16>$)){3}){0,1}"#,
    "dgvi",
    JsString::from("a"),
    1,
    MatchExpectation {
        text: "",
        range: 1..1,
        captures: &[None; 16],
        capture_ranges: &[None; 16],
        last_index: 1,
    };
    "input_a_from_1_variant_22"
)]
#[test_case(
    r#"(?:(?<n1>(?<n2>^)|(?<n3>\b))(?<n4>(?<n5>){2,3})(?<n6>(?<n7>^)|(?<n8>$)){3}|(?<n9>(?<n10>^)|(?<n11>\B))(?<n12>(?<n13>[a-z\q{ab}]){1}){1,1}?(?<n14>(?<n15>\b^)|(?<n16>\B$))){0000,0001}?"#,
    "dgvy",
    JsString::from("!ab!"),
    0,
    MatchExpectation {
        text: "",
        range: 0..0,
        captures: &[None; 16],
        capture_ranges: &[None; 16],
        last_index: 0,
    };
    "input_ab_from_0_variant_23"
)]
#[test_case(
    r#"(?<n1>(?<n2>(?<n3>(?<n4>^)|(?<n5>\b))\B)(?<n6>[\q{a|ab}])(?<n7>(?<n8>\b\B)|(?<n9>\b\B))|(?<n10>(?<n11>$)|(?<n12>\b)){2,3}?(?<n13>(?<n14>){2,3})(?<n15>(?<n16>\b^)|(?<n17>$)))?"#,
    "dgvm",
    JsString::from("😀a"),
    0,
    MatchExpectation {
        text: "",
        range: 0..0,
        captures: &[None; 17],
        capture_ranges: &[None; 17],
        last_index: 0,
    };
    "unicode_input_from_0_variant_24"
)]
#[test_case(
    r#"(?<n1>(?<n2>(?<n3>(?<n4>^)|(?<n5>\b))\B)(?<n6>[\p{Any}\q{|ab}]){0001,0001}?(?<n7>(?<n8>\b\B)|(?<n9>\b\B))|(?<n10>(?<n11>$)|(?<n12>\b)){2,3}?(?<n13>(?<n14>){2,3})(?<n15>(?<n16>\b^)|(?<n17>$))){0,1}"#,
    "dgvy",
    JsString::from("ab"),
    0,
    MatchExpectation {
        text: "",
        range: 0..0,
        captures: &[None; 17],
        capture_ranges: &[None; 17],
        last_index: 0,
    };
    "input_ab_from_0_variant_25"
)]
#[test_case(
    r#"(?:(?<n1>(?<n2>$)|(?<n3>\b)){2,3}?(?<n4>(?<n5>){2,3})(?<n6>(?<n7>\b^)|(?<n8>$))|(?<n9>(?<n10>(?<n11>^)|(?<n12>\b))\B)(?<n13>(?<n14>[a-z\q{ab}]){1}){1,1}?(?<n15>(?<n16>\b\B)|(?<n17>\b\B))){0000,0001}?"#,
    "dgvim",
    JsString::from("a\n!"),
    1,
    MatchExpectation {
        text: "",
        range: 1..1,
        captures: &[None; 17],
        capture_ranges: &[None; 17],
        last_index: 1,
    };
    "input_a_from_1_variant_26"
)]
#[test_case(
    r#"(?<n1>(?<n2>(?<n3>$)|(?<n4>\B))(?<n5>[\q{a|ab}])(?<n6>(?<n7>^)|(?<n8>\b))|(?<n9>(?<n10>^)|(?<n11>$)){3}(?<n12>(?<n13>){2,3})(?<n14>(?<n15>\B^)|(?<n16>\b$)))?"#,
    "dgvi",
    JsString::from("ab\r\n"),
    1,
    MatchExpectation {
        text: "",
        range: 1..1,
        captures: &[None; 16],
        capture_ranges: &[None; 16],
        last_index: 1,
    };
    "input_ab_from_1_variant_27"
)]
#[test_case(
    r#"(?:(?<n1>(?<n2>^)|(?<n3>$)){3}(?<n4>(?<n5>){2,3})(?<n6>(?<n7>\B^)|(?<n8>\b$))|(?<n9>(?<n10>$)|(?<n11>\B))(?<n12>(?<n13>[\q{|a|ab}])){1}(?<n14>(?<n15>^)|(?<n16>\b)))??"#,
    "dgvy",
    JsString::from("a"),
    0,
    MatchExpectation {
        text: "",
        range: 0..0,
        captures: &[None; 16],
        capture_ranges: &[None; 16],
        last_index: 0,
    };
    "input_a_from_0_variant_28"
)]
#[test_case(
    r#"(?<n1>(?<n2>(?<n3>$)|(?<n4>\B))(?<n5>[\p{Any}\q{|ab}]){0001,0001}?(?<n6>(?<n7>^)|(?<n8>\b))|(?<n9>(?<n10>^)|(?<n11>$)){3}(?<n12>(?<n13>){2,3})(?<n14>(?<n15>\B^)|(?<n16>\b$))){0,1}"#,
    "dgvim",
    JsString::from("!ab!"),
    1,
    MatchExpectation {
        text: "",
        range: 1..1,
        captures: &[None; 16],
        capture_ranges: &[None; 16],
        last_index: 1,
    };
    "input_ab_from_1_variant_29"
)]
#[test_case(
    r#"(?:(?<n1>(?<n2>^)|(?<n3>$)){3}(?<n4>(?<n5>){2,3})(?<n6>(?<n7>\B^)|(?<n8>\b$))|(?<n9>(?<n10>$)|(?<n11>\B))(?<n12>(?<n13>[a-z\q{ab}]){1}){1,1}?(?<n14>(?<n15>^)|(?<n16>\b))){0000,0001}?"#,
    "dgv",
    JsString::from("ſa"),
    0,
    MatchExpectation {
        text: "",
        range: 0..0,
        captures: &[None; 16],
        capture_ranges: &[None; 16],
        last_index: 0,
    };
    "unicode_input_from_0_variant_30"
)]
#[test_case(
    r#"(?<n1>(?<n2>(?<n3>\b^)|(?<n4>\B$))(?<n5>[\q{a|ab}])(?<n6>(?<n7>$)|(?<n8>\b)){2,3}?|(?<n9>(?<n10>\b^)|(?<n11>$))(?<n12>(?<n13>){2,3})(?<n14>(?<n15>^)|(?<n16>\B)))?"#,
    "dgvy",
    JsString::from(""),
    0,
    MatchExpectation {
        text: "",
        range: 0..0,
        captures: &[None; 16],
        capture_ranges: &[None; 16],
        last_index: 0,
    };
    "empty_input_from_0_variant_31"
)]
#[test_case(
    r#"(?:(?<n1>(?<n2>\b^)|(?<n3>$))(?<n4>(?<n5>){2,3})(?<n6>(?<n7>^)|(?<n8>\B))|(?<n9>(?<n10>\b^)|(?<n11>\B$))(?<n12>(?<n13>[\q{|a|ab}])){1}(?<n14>(?<n15>$)|(?<n16>\b)){2,3}?)??"#,
    "dgvim",
    JsString::from("ab"),
    1,
    MatchExpectation {
        text: "",
        range: 1..1,
        captures: &[None; 16],
        capture_ranges: &[None; 16],
        last_index: 1,
    };
    "input_ab_from_1_variant_32"
)]
#[test_case(
    r#"(?<n1>(?<n2>(?<n3>\b^)|(?<n4>\B$))(?<n5>[\p{Any}\q{|ab}]){0001,0001}?(?<n6>(?<n7>$)|(?<n8>\b)){2,3}?|(?<n9>(?<n10>\b^)|(?<n11>$))(?<n12>(?<n13>){2,3})(?<n14>(?<n15>^)|(?<n16>\B))){0,1}"#,
    "dgv",
    JsString::from("a\n!"),
    0,
    MatchExpectation {
        text: "a",
        range: 0..1,
        captures: &[
            Some("a"),
            Some(""),
            Some(""),
            None,
            Some("a"),
            Some(""),
            None,
            Some(""),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        ],
        capture_ranges: &[
            Some((0, 1)),
            Some((0, 0)),
            Some((0, 0)),
            None,
            Some((0, 1)),
            Some((1, 1)),
            None,
            Some((1, 1)),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        ],
        last_index: 1,
    };
    "input_a_from_0_variant_33"
)]
#[test_case(
    r#"(?:(?<n1>(?<n2>\b^)|(?<n3>$))(?<n4>(?<n5>){2,3})(?<n6>(?<n7>^)|(?<n8>\B))|(?<n9>(?<n10>\b^)|(?<n11>\B$))(?<n12>(?<n13>[a-z\q{ab}]){1}){1,1}?(?<n14>(?<n15>$)|(?<n16>\b)){2,3}?){0000,0001}?"#,
    "dgvi",
    JsString::from("😀a"),
    1,
    MatchExpectation {
        text: "",
        range: 0..0,
        captures: &[None; 16],
        capture_ranges: &[None; 16],
        last_index: 0,
    };
    "unicode_input_from_1_variant_34"
)]
#[test_case(
    r#"(?<n1>(?<n2>(?<n3>\b\B)|(?<n4>\b\B))(?<n5>[\q{a|ab}])(?<n6>(?<n7>^)|(?<n8>$)){3}|(?<n9>(?<n10>\B^)|(?<n11>\b$))(?<n12>(?<n13>){2,3})(?<n14>(?<n15>(?<n16>^)|(?<n17>\b))\B))?"#,
    "dgvim",
    JsString::from("a"),
    1,
    MatchExpectation {
        text: "",
        range: 1..1,
        captures: &[None; 17],
        capture_ranges: &[None; 17],
        last_index: 1,
    };
    "input_a_from_1_variant_35"
)]
#[test_case(
    r#"(?:(?<n1>(?<n2>\B^)|(?<n3>\b$))(?<n4>(?<n5>){2,3})(?<n6>(?<n7>(?<n8>^)|(?<n9>\b))\B)|(?<n10>(?<n11>\b\B)|(?<n12>\b\B))(?<n13>(?<n14>[\q{|a|ab}])){1}(?<n15>(?<n16>^)|(?<n17>$)){3})??"#,
    "dgv",
    JsString::from("!ab!"),
    0,
    MatchExpectation {
        text: "",
        range: 0..0,
        captures: &[None; 17],
        capture_ranges: &[None; 17],
        last_index: 0,
    };
    "input_ab_from_0_variant_36"
)]
#[test_case(
    r#"(?<n1>(?<n2>(?<n3>\b\B)|(?<n4>\b\B))(?<n5>[\p{Any}\q{|ab}]){0001,0001}?(?<n6>(?<n7>^)|(?<n8>$)){3}|(?<n9>(?<n10>\B^)|(?<n11>\b$))(?<n12>(?<n13>){2,3})(?<n14>(?<n15>(?<n16>^)|(?<n17>\b))\B)){0,1}"#,
    "dgvi",
    JsString::from("ſa"),
    1,
    MatchExpectation {
        text: "",
        range: 1..1,
        captures: &[None; 17],
        capture_ranges: &[None; 17],
        last_index: 1,
    };
    "unicode_input_from_1_variant_37"
)]
#[test_case(
    r#"(?:(?<n1>(?<n2>\B^)|(?<n3>\b$))(?<n4>(?<n5>){2,3})(?<n6>(?<n7>(?<n8>^)|(?<n9>\b))\B)|(?<n10>(?<n11>\b\B)|(?<n12>\b\B))(?<n13>(?<n14>[a-z\q{ab}]){1}){1,1}?(?<n15>(?<n16>^)|(?<n17>$)){3}){0000,0001}?"#,
    "dgvym",
    JsString::from("ab\r\n"),
    0,
    MatchExpectation {
        text: "",
        range: 0..0,
        captures: &[None; 17],
        capture_ranges: &[None; 17],
        last_index: 0,
    };
    "input_ab_from_0_variant_38"
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
fn truth_table_optional_alternative_string_classes_validate_unknowns_even_in_lazy_never_bodies() {
    assert_true(
        r####"var pattern="((((^)|(\\b))([\\q{a|ab}])|())?)",input="aba",r=new RegExp(pattern,"dv"),m=r.exec(input);r.source===pattern&&r.flags==="dv"&&r.hasIndices&&r.unicodeSets&&(m===null||(m.input===input&&m.groups===undefined&&m.indices.groups===undefined&&Object.hasOwn(m,"groups")&&Object.hasOwn(m.indices,"groups")))&&JSON.stringify(m===null?null:[Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex])==="[[\"ab\",\"ab\",\"ab\",\"\",\"\",\"<undefined>\",\"ab\",\"<undefined>\"],[[0,2],[0,2],[0,2],[0,0],[0,0],\"<undefined>\",[0,2],\"<undefined>\"],0,0]""####,
    );
    assert_true(
        r####"var pattern="^(((^)|(\\b))([\\q{a|ab}])|())?",input="aba",r=new RegExp(pattern,"dv"),m=r.exec(input);r.source===pattern&&r.flags==="dv"&&r.hasIndices&&r.unicodeSets&&(m===null||(m.input===input&&m.groups===undefined&&m.indices.groups===undefined&&Object.hasOwn(m,"groups")&&Object.hasOwn(m.indices,"groups")))&&JSON.stringify(m===null?null:[Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex])==="[[\"ab\",\"ab\",\"\",\"\",\"<undefined>\",\"ab\",\"<undefined>\"],[[0,2],[0,2],[0,0],[0,0],\"<undefined>\",[0,2],\"<undefined>\"],0,0]""####,
    );
    assert_true(
        r####"var pattern="(((^)|(\\b))([\\q{a|ab}])|())?$",input="aba",r=new RegExp(pattern,"dv"),m=r.exec(input);r.source===pattern&&r.flags==="dv"&&r.hasIndices&&r.unicodeSets&&(m===null||(m.input===input&&m.groups===undefined&&m.indices.groups===undefined&&Object.hasOwn(m,"groups")&&Object.hasOwn(m.indices,"groups")))&&JSON.stringify(m===null?null:[Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex])==="[[\"\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],[[3,3],\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],3,0]""####,
    );
    for pattern in [
        r"(((\b\B)|(\b\B))([\q{ab}&&\p{RGI_Emoji}])|())??",
        r"(((^)|(\b))([\q{a|ab}])|z)??",
        r"(((^)|(\b))([\q{a|ab}])|\1)?",
        r"(((^)|(\b))([\q{a|ab}])|([a]))?",
        r"(((^)|(\b))([\q{a|ab}])|()){0,2}",
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
