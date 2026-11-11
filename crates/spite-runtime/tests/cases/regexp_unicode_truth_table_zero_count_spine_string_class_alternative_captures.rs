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
                "(?:(?<n1>(?<n2>$)|(?<n3>\\B))(?<n4>(?<n5>){2,3})(?<n6>(?<n7>\\b\\B)|(?<n8>\\b\\B))|(?<n9>(?<n10>^)|(?<n11>$)){3}(?<n12>(?<n13>(?<n14>[a-z\\q{ab}]){1}){1,1}?){0,0}?(?<n15>(?<n16>^)|(?<n17>\\B))){0000,0000}?",
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
                "(?:(?<n1>(?<n2>$)|(?<n3>\\b)){2,3}?(?<n4>(?<n5>){2,3})(?<n6>(?<n7>\\b^)|(?<n8>$))|(?<n9>(?<n10>(?<n11>^)|(?<n12>\\b))\\B)(?<n13>(?<n14>(?<n15>[\\q{|a|ab}])){1})??(?<n16>(?<n17>\\b\\B)|(?<n18>\\b\\B))){0,0}?",
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
                    "(?<outer>(?<b>(?<s>^)|(?<w>\\b))(?<whole>(?<c>[\\q{|a|ab}]))??(?<a>(?<end>$)|(?<nw>\\B))|(?<fallback>(?<wb>\\b)|(?<nwb>\\B))(?<empty>)){0000,0000}?";
                var r = new RegExp(P, "dvg"),
                    a = Array.from("ab!aa".matchAll(r));
                return JSON.stringify([a.map(row), r.lastIndex]);
            })() ===
            "[[[[\"\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],[[0,0],\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],0,\"ab!aa\",[\"outer\",\"b\",\"s\",\"w\",\"whole\",\"c\",\"a\",\"end\",\"nw\",\"fallback\",\"wb\",\"nwb\",\"empty\"],[\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],[\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],true,true],[[\"\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],[[1,1],\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],1,\"ab!aa\",[\"outer\",\"b\",\"s\",\"w\",\"whole\",\"c\",\"a\",\"end\",\"nw\",\"fallback\",\"wb\",\"nwb\",\"empty\"],[\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],[\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],true,true],[[\"\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],[[2,2],\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],2,\"ab!aa\",[\"outer\",\"b\",\"s\",\"w\",\"whole\",\"c\",\"a\",\"end\",\"nw\",\"fallback\",\"wb\",\"nwb\",\"empty\"],[\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],[\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],true,true],[[\"\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],[[3,3],\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],3,\"ab!aa\",[\"outer\",\"b\",\"s\",\"w\",\"whole\",\"c\",\"a\",\"end\",\"nw\",\"fallback\",\"wb\",\"nwb\",\"empty\"],[\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],[\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],true,true],[[\"\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],[[4,4],\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],4,\"ab!aa\",[\"outer\",\"b\",\"s\",\"w\",\"whole\",\"c\",\"a\",\"end\",\"nw\",\"fallback\",\"wb\",\"nwb\",\"empty\"],[\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],[\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],true,true],[[\"\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],[[5,5],\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],5,\"ab!aa\",[\"outer\",\"b\",\"s\",\"w\",\"whole\",\"c\",\"a\",\"end\",\"nw\",\"fallback\",\"wb\",\"nwb\",\"empty\"],[\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],[\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],true,true]],0]"
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
                    "(?<outer>(?<b>(?<s>^)|(?<w>\\b))(?<whole>(?<c>[\\q{|a|ab}]))??(?<a>(?<end>$)|(?<nw>\\B))|(?<fallback>(?<wb>\\b)|(?<nwb>\\B))(?<empty>)){0000,0000}?";
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
            "[\"[]a[]b[]![]a[]a[]\",[[\"\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",0,\"ab!aa\",[[\"outer\",\"<undefined>\"],[\"b\",\"<undefined>\"],[\"s\",\"<undefined>\"],[\"w\",\"<undefined>\"],[\"whole\",\"<undefined>\"],[\"c\",\"<undefined>\"],[\"a\",\"<undefined>\"],[\"end\",\"<undefined>\"],[\"nw\",\"<undefined>\"],[\"fallback\",\"<undefined>\"],[\"wb\",\"<undefined>\"],[\"nwb\",\"<undefined>\"],[\"empty\",\"<undefined>\"]]],[\"\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",1,\"ab!aa\",[[\"outer\",\"<undefined>\"],[\"b\",\"<undefined>\"],[\"s\",\"<undefined>\"],[\"w\",\"<undefined>\"],[\"whole\",\"<undefined>\"],[\"c\",\"<undefined>\"],[\"a\",\"<undefined>\"],[\"end\",\"<undefined>\"],[\"nw\",\"<undefined>\"],[\"fallback\",\"<undefined>\"],[\"wb\",\"<undefined>\"],[\"nwb\",\"<undefined>\"],[\"empty\",\"<undefined>\"]]],[\"\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",2,\"ab!aa\",[[\"outer\",\"<undefined>\"],[\"b\",\"<undefined>\"],[\"s\",\"<undefined>\"],[\"w\",\"<undefined>\"],[\"whole\",\"<undefined>\"],[\"c\",\"<undefined>\"],[\"a\",\"<undefined>\"],[\"end\",\"<undefined>\"],[\"nw\",\"<undefined>\"],[\"fallback\",\"<undefined>\"],[\"wb\",\"<undefined>\"],[\"nwb\",\"<undefined>\"],[\"empty\",\"<undefined>\"]]],[\"\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",3,\"ab!aa\",[[\"outer\",\"<undefined>\"],[\"b\",\"<undefined>\"],[\"s\",\"<undefined>\"],[\"w\",\"<undefined>\"],[\"whole\",\"<undefined>\"],[\"c\",\"<undefined>\"],[\"a\",\"<undefined>\"],[\"end\",\"<undefined>\"],[\"nw\",\"<undefined>\"],[\"fallback\",\"<undefined>\"],[\"wb\",\"<undefined>\"],[\"nwb\",\"<undefined>\"],[\"empty\",\"<undefined>\"]]],[\"\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",4,\"ab!aa\",[[\"outer\",\"<undefined>\"],[\"b\",\"<undefined>\"],[\"s\",\"<undefined>\"],[\"w\",\"<undefined>\"],[\"whole\",\"<undefined>\"],[\"c\",\"<undefined>\"],[\"a\",\"<undefined>\"],[\"end\",\"<undefined>\"],[\"nw\",\"<undefined>\"],[\"fallback\",\"<undefined>\"],[\"wb\",\"<undefined>\"],[\"nwb\",\"<undefined>\"],[\"empty\",\"<undefined>\"]]],[\"\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",5,\"ab!aa\",[[\"outer\",\"<undefined>\"],[\"b\",\"<undefined>\"],[\"s\",\"<undefined>\"],[\"w\",\"<undefined>\"],[\"whole\",\"<undefined>\"],[\"c\",\"<undefined>\"],[\"a\",\"<undefined>\"],[\"end\",\"<undefined>\"],[\"nw\",\"<undefined>\"],[\"fallback\",\"<undefined>\"],[\"wb\",\"<undefined>\"],[\"nwb\",\"<undefined>\"],[\"empty\",\"<undefined>\"]]]],0]"
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
                    "(?<outer>(?<b>(?<s>^)|(?<w>\\b))(?<whole>(?<c>[\\q{|a|ab}]))??(?<a>(?<end>$)|(?<nw>\\B))|(?<fallback>(?<wb>\\b)|(?<nwb>\\B))(?<empty>)){0000,0000}?";
                var r = new RegExp(P, "dvg");
                r.lastIndex = 9;
                var a = "!abaa!".split(r),
                    b = "!abaa!".search(r);
                return JSON.stringify([a.map(v => v === undefined ? "<undefined>" : v), b, r
                    .lastIndex]);
            })() ===
            "[[\"!\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"a\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"b\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"a\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"a\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"!\"],0,9]"
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
                    "(?<outer>(?<b>(?<s>^)|(?<w>\\b))(?<whole>(?<c>[\\q{|a|ab}]))??(?<a>(?<end>$)|(?<nw>\\B))|(?<fallback>(?<wb>\\b)|(?<nwb>\\B))(?<empty>)){0000,0000}?";
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
                    "(?<outer>(?<b>(?<s>^)|(?<w>\\b))(?<whole>(?<c>[\\q{|a|ab}]))??(?<a>(?<end>$)|(?<nw>\\B))|(?<fallback>(?<wb>\\b)|(?<nwb>\\B))(?<empty>)){0000,0000}?";
                var r = new RegExp(P, "dvg"),
                    a = r.exec(""),
                    b = r.exec("ab");
                return JSON.stringify([row(a), row(b), r.lastIndex]);
            })() ===
            "[[[\"\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],[[0,0],\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],0,\"\",[\"outer\",\"b\",\"s\",\"w\",\"whole\",\"c\",\"a\",\"end\",\"nw\",\"fallback\",\"wb\",\"nwb\",\"empty\"],[\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],[\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],true,true],[[\"\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],[[0,0],\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],0,\"ab\",[\"outer\",\"b\",\"s\",\"w\",\"whole\",\"c\",\"a\",\"end\",\"nw\",\"fallback\",\"wb\",\"nwb\",\"empty\"],[\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],[\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],true,true],0]"
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
                    "(?<outer>(?<b>(?<s>^)|(?<w>\\b))(?<whole>(?<c>[a-z\\q{|S|sa|aX}]))??(?<a>(?<end>$)|(?<nw>\\B))|(?<fallback>(?<wb>\\b)|(?<nwb>\\B))(?<empty>)){0000,0000}?";
                var r = new RegExp(P, "dvgi"),
                    a = r.exec("ſaX!");
                return JSON.stringify([row(a), r.lastIndex]);
            })() ===
            "[[[\"\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],[[0,0],\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],0,\"\u017faX!\",[\"outer\",\"b\",\"s\",\"w\",\"whole\",\"c\",\"a\",\"end\",\"nw\",\"fallback\",\"wb\",\"nwb\",\"empty\"],[\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],[\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],true,true],0]"
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
                    "(?<outer>(?<b>(?<s>^)|(?<w>\\b))(?<whole>(?<c>[\\q{|a|ab}]))??(?<a>(?<end>$)|(?<nw>\\B))|(?<fallback>(?<wb>\\b)|(?<nwb>\\B))(?<empty>)){0000,0000}?";
                var r = new RegExp(P, "dvgm"),
                    a = Array.from("ab\n\naa".matchAll(r));
                return JSON.stringify([a.map(row), r.lastIndex]);
            })() ===
            "[[[[\"\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],[[0,0],\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],0,\"ab\\n\\naa\",[\"outer\",\"b\",\"s\",\"w\",\"whole\",\"c\",\"a\",\"end\",\"nw\",\"fallback\",\"wb\",\"nwb\",\"empty\"],[\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],[\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],true,true],[[\"\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],[[1,1],\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],1,\"ab\\n\\naa\",[\"outer\",\"b\",\"s\",\"w\",\"whole\",\"c\",\"a\",\"end\",\"nw\",\"fallback\",\"wb\",\"nwb\",\"empty\"],[\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],[\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],true,true],[[\"\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],[[2,2],\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],2,\"ab\\n\\naa\",[\"outer\",\"b\",\"s\",\"w\",\"whole\",\"c\",\"a\",\"end\",\"nw\",\"fallback\",\"wb\",\"nwb\",\"empty\"],[\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],[\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],true,true],[[\"\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],[[3,3],\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],3,\"ab\\n\\naa\",[\"outer\",\"b\",\"s\",\"w\",\"whole\",\"c\",\"a\",\"end\",\"nw\",\"fallback\",\"wb\",\"nwb\",\"empty\"],[\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],[\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],true,true],[[\"\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],[[4,4],\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],4,\"ab\\n\\naa\",[\"outer\",\"b\",\"s\",\"w\",\"whole\",\"c\",\"a\",\"end\",\"nw\",\"fallback\",\"wb\",\"nwb\",\"empty\"],[\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],[\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],true,true],[[\"\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],[[5,5],\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],5,\"ab\\n\\naa\",[\"outer\",\"b\",\"s\",\"w\",\"whole\",\"c\",\"a\",\"end\",\"nw\",\"fallback\",\"wb\",\"nwb\",\"empty\"],[\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],[\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],true,true],[[\"\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],[[6,6],\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],6,\"ab\\n\\naa\",[\"outer\",\"b\",\"s\",\"w\",\"whole\",\"c\",\"a\",\"end\",\"nw\",\"fallback\",\"wb\",\"nwb\",\"empty\"],[\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],[\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],true,true]],0]"
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
                    "(?<outer>(?<\\u{1D49C}>(?<s>^)|(?<w>\\b))(?<whole>(?<c>[\\q{|a|ab}]))??(?<a>(?<end>$)|(?<nw>\\B))|(?<fallback>(?<wb>\\b)|(?<nwb>\\B))(?<empty>)){0000,0000}?";
                var r = new RegExp(P, "dvg"),
                    s = new RegExp(r);
                r.lastIndex = 3;
                var a = s.exec("ab!");
                return JSON.stringify([row(a), s.source === r.source, s.flags === r.flags, s.lastIndex,
                    r.lastIndex
                ]);
            })() ===
            "[[[\"\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],[[0,0],\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],0,\"ab!\",[\"outer\",\"\ud835\udc9c\",\"s\",\"w\",\"whole\",\"c\",\"a\",\"end\",\"nw\",\"fallback\",\"wb\",\"nwb\",\"empty\"],[\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],[\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],true,true],true,true,0,3]"
        "#,
    );

    let mut realm = Realm::default();
    realm.eval(r"var r=/(?<root>(?<scope>(?<c>[\q{a|ab}])??)??|(?<e>)){0}/dvg,clone=new RegExp(r),m=r.exec('ab')").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval(r#"
        var n = clone.exec('ab');
        m[0] === '' && n[0] === '' && Array.from(m).slice(1).every(v => v === undefined) && m.indices
            .slice(1).every(v => v === undefined) && m.groups.root === undefined && m.groups.scope ===
            undefined && m.groups.c === undefined && m.groups.e === undefined && Object.hasOwn(m
                .groups, 'scope') && Object.hasOwn(m.indices.groups, 'scope') && clone.source === r
            .source && clone.flags === r.flags && r.lastIndex === 0 && clone.lastIndex === 0
    "#),Ok(Value::Boolean(true)));
}

#[test_case(
    r#"(?<n1>(?<n2>(?<n3>^)|(?<n4>\b))(?<n5>(?<n6>[\q{a|ab}]))?(?<n7>(?<n8>\b^)|(?<n9>$))|(?<n10>(?<n11>^)|(?<n12>\B))(?<n13>(?<n14>){2,3})(?<n15>(?<n16>$)|(?<n17>\B))){0}"#,
    "dgvym",
    JsString::from(""),
    0,
    MatchExpectation {
        text: "",
        range: 0..0,
        captures: &[None; 17],
        capture_ranges: &[None; 17],
        last_index: 0,
    };
    "empty_input_from_0_variant_1"
)]
#[test_case(
    r#"(?:(?<n1>(?<n2>^)|(?<n3>\B))(?<n4>(?<n5>){2,3})(?<n6>(?<n7>$)|(?<n8>\B))|(?<n9>(?<n10>^)|(?<n11>\b))(?<n12>(?<n13>(?<n14>[\q{|a|ab}])){1})??(?<n15>(?<n16>\b^)|(?<n17>$))){0,0}?"#,
    "dgvi",
    JsString::from("ab"),
    1,
    MatchExpectation {
        text: "",
        range: 1..1,
        captures: &[None; 17],
        capture_ranges: &[None; 17],
        last_index: 1,
    };
    "input_ab_from_1_variant_2"
)]
#[test_case(
    r#"(?<n1>(?<n2>(?<n3>^)|(?<n4>\b))(?<n5>(?<n6>[\p{Any}\q{|ab}]){0001,0001}?){0000,0001}(?<n7>(?<n8>\b^)|(?<n9>$))|(?<n10>(?<n11>^)|(?<n12>\B))(?<n13>(?<n14>){2,3})(?<n15>(?<n16>$)|(?<n17>\B))){0000}"#,
    "dgv",
    JsString::from("a\n!"),
    0,
    MatchExpectation {
        text: "",
        range: 0..0,
        captures: &[None; 17],
        capture_ranges: &[None; 17],
        last_index: 0,
    };
    "input_a_from_0_variant_3"
)]
#[test_case(
    r#"(?:(?<n1>(?<n2>^)|(?<n3>\B))(?<n4>(?<n5>){2,3})(?<n6>(?<n7>$)|(?<n8>\B))|(?<n9>(?<n10>^)|(?<n11>\b))(?<n12>(?<n13>(?<n14>[a-z\q{ab}]){1}){1,1}?){0,0}?(?<n15>(?<n16>\b^)|(?<n17>$))){0000,0000}?"#,
    "dgvim",
    JsString::from("😀a"),
    1,
    MatchExpectation {
        text: "",
        range: 0..0,
        captures: &[None; 17],
        capture_ranges: &[None; 17],
        last_index: 0,
    };
    "unicode_input_from_1_variant_4"
)]
#[test_case(
    r#"(?<n1>(?<n2>(?<n3>$)|(?<n4>\b)){2,3}?(?<n5>(?<n6>[\q{a|ab}]))?(?<n7>(?<n8>\B^)|(?<n9>\b$))|(?<n10>(?<n11>(?<n12>^)|(?<n13>\b))\B)(?<n14>(?<n15>){2,3})(?<n16>(?<n17>\b^)|(?<n18>\B$))){0}"#,
    "dgvi",
    JsString::from("a"),
    1,
    MatchExpectation {
        text: "",
        range: 1..1,
        captures: &[None; 18],
        capture_ranges: &[None; 18],
        last_index: 1,
    };
    "input_a_from_1_variant_5"
)]
#[test_case(
    r#"(?:(?<n1>(?<n2>(?<n3>^)|(?<n4>\b))\B)(?<n5>(?<n6>){2,3})(?<n7>(?<n8>\b^)|(?<n9>\B$))|(?<n10>(?<n11>$)|(?<n12>\b)){2,3}?(?<n13>(?<n14>(?<n15>[\q{|a|ab}])){1})??(?<n16>(?<n17>\B^)|(?<n18>\b$))){0,0}?"#,
    "dgv",
    JsString::from("!ab!"),
    0,
    MatchExpectation {
        text: "",
        range: 0..0,
        captures: &[None; 18],
        capture_ranges: &[None; 18],
        last_index: 0,
    };
    "input_ab_from_0_variant_6"
)]
#[test_case(
    r#"(?<n1>(?<n2>(?<n3>$)|(?<n4>\b)){2,3}?(?<n5>(?<n6>[\p{Any}\q{|ab}]){0001,0001}?){0000,0001}(?<n7>(?<n8>\B^)|(?<n9>\b$))|(?<n10>(?<n11>(?<n12>^)|(?<n13>\b))\B)(?<n14>(?<n15>){2,3})(?<n16>(?<n17>\b^)|(?<n18>\B$))){0000}"#,
    "dgvim",
    JsString::from("ſa"),
    1,
    MatchExpectation {
        text: "",
        range: 1..1,
        captures: &[None; 18],
        capture_ranges: &[None; 18],
        last_index: 1,
    };
    "unicode_input_from_1_variant_7"
)]
#[test_case(
    r#"(?:(?<n1>(?<n2>(?<n3>^)|(?<n4>\b))\B)(?<n5>(?<n6>){2,3})(?<n7>(?<n8>\b^)|(?<n9>\B$))|(?<n10>(?<n11>$)|(?<n12>\b)){2,3}?(?<n13>(?<n14>(?<n15>[a-z\q{ab}]){1}){1,1}?){0,0}?(?<n16>(?<n17>\B^)|(?<n18>\b$))){0000,0000}?"#,
    "dgvy",
    JsString::from("ab\r\n"),
    0,
    MatchExpectation {
        text: "",
        range: 0..0,
        captures: &[None; 18],
        capture_ranges: &[None; 18],
        last_index: 0,
    };
    "input_ab_from_0_variant_8"
)]
#[test_case(
    r#"(?<n1>(?<n2>(?<n3>^)|(?<n4>$)){3}(?<n5>(?<n6>[\q{a|ab}]))?(?<n7>(?<n8>^)|(?<n9>\B))|(?<n10>(?<n11>$)|(?<n12>\B))(?<n13>(?<n14>){2,3})(?<n15>(?<n16>\b\B)|(?<n17>\b\B))){0}"#,
    "dgv",
    JsString::from("ab"),
    0,
    MatchExpectation {
        text: "",
        range: 0..0,
        captures: &[None; 17],
        capture_ranges: &[None; 17],
        last_index: 0,
    };
    "input_ab_from_0_variant_9"
)]
#[test_case(
    r#"(?:(?<n1>(?<n2>$)|(?<n3>\B))(?<n4>(?<n5>){2,3})(?<n6>(?<n7>\b\B)|(?<n8>\b\B))|(?<n9>(?<n10>^)|(?<n11>$)){3}(?<n12>(?<n13>(?<n14>[\q{|a|ab}])){1})??(?<n15>(?<n16>^)|(?<n17>\B))){0,0}?"#,
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
    "input_a_from_1_variant_10"
)]
#[test_case(
    r#"(?<n1>(?<n2>(?<n3>^)|(?<n4>$)){3}(?<n5>(?<n6>[\p{Any}\q{|ab}]){0001,0001}?){0000,0001}(?<n7>(?<n8>^)|(?<n9>\B))|(?<n10>(?<n11>$)|(?<n12>\B))(?<n13>(?<n14>){2,3})(?<n15>(?<n16>\b\B)|(?<n17>\b\B))){0000}"#,
    "dgvy",
    JsString::from("😀a"),
    0,
    MatchExpectation {
        text: "",
        range: 0..0,
        captures: &[None; 17],
        capture_ranges: &[None; 17],
        last_index: 0,
    };
    "unicode_input_from_0_variant_11"
)]
#[test_case(
    r#"(?<n1>(?<n2>(?<n3>\b^)|(?<n4>$))(?<n5>(?<n6>[\q{a|ab}]))?(?<n7>(?<n8>(?<n9>^)|(?<n10>\b))\B)|(?<n11>(?<n12>\b^)|(?<n13>\B$))(?<n14>(?<n15>){2,3})(?<n16>(?<n17>^)|(?<n18>\b))){0}"#,
    "dgvim",
    JsString::from("!ab!"),
    1,
    MatchExpectation {
        text: "",
        range: 1..1,
        captures: &[None; 18],
        capture_ranges: &[None; 18],
        last_index: 1,
    };
    "input_ab_from_1_variant_12"
)]
#[test_case(
    r#"(?:(?<n1>(?<n2>\b^)|(?<n3>\B$))(?<n4>(?<n5>){2,3})(?<n6>(?<n7>^)|(?<n8>\b))|(?<n9>(?<n10>\b^)|(?<n11>$))(?<n12>(?<n13>(?<n14>[\q{|a|ab}])){1})??(?<n15>(?<n16>(?<n17>^)|(?<n18>\b))\B)){0,0}?"#,
    "dgvy",
    JsString::from("ſa"),
    0,
    MatchExpectation {
        text: "",
        range: 0..0,
        captures: &[None; 18],
        capture_ranges: &[None; 18],
        last_index: 0,
    };
    "unicode_input_from_0_variant_13"
)]
#[test_case(
    r#"(?<n1>(?<n2>(?<n3>\b^)|(?<n4>$))(?<n5>(?<n6>[\p{Any}\q{|ab}]){0001,0001}?){0000,0001}(?<n7>(?<n8>(?<n9>^)|(?<n10>\b))\B)|(?<n11>(?<n12>\b^)|(?<n13>\B$))(?<n14>(?<n15>){2,3})(?<n16>(?<n17>^)|(?<n18>\b))){0000}"#,
    "dgvi",
    JsString::from("ab\r\n"),
    1,
    MatchExpectation {
        text: "",
        range: 1..1,
        captures: &[None; 18],
        capture_ranges: &[None; 18],
        last_index: 1,
    };
    "input_ab_from_1_variant_14"
)]
#[test_case(
    r#"(?:(?<n1>(?<n2>\b^)|(?<n3>\B$))(?<n4>(?<n5>){2,3})(?<n6>(?<n7>^)|(?<n8>\b))|(?<n9>(?<n10>\b^)|(?<n11>$))(?<n12>(?<n13>(?<n14>[a-z\q{ab}]){1}){1,1}?){0,0}?(?<n15>(?<n16>(?<n17>^)|(?<n18>\b))\B)){0000,0000}?"#,
    "dgvm",
    JsString::from("a"),
    0,
    MatchExpectation {
        text: "",
        range: 0..0,
        captures: &[None; 18],
        capture_ranges: &[None; 18],
        last_index: 0,
    };
    "input_a_from_0_variant_15"
)]
#[test_case(
    r#"(?<n1>(?<n2>(?<n3>\B^)|(?<n4>\b$))(?<n5>(?<n6>[\q{a|ab}]))?(?<n7>(?<n8>$)|(?<n9>\B))|(?<n10>(?<n11>\b\B)|(?<n12>\b\B))(?<n13>(?<n14>){2,3})(?<n15>(?<n16>$)|(?<n17>\b)){2,3}?){0}"#,
    "dgvy",
    JsString::from("a\n!"),
    0,
    MatchExpectation {
        text: "",
        range: 0..0,
        captures: &[None; 17],
        capture_ranges: &[None; 17],
        last_index: 0,
    };
    "input_a_from_0_variant_16"
)]
#[test_case(
    r#"(?:(?<n1>(?<n2>\b\B)|(?<n3>\b\B))(?<n4>(?<n5>){2,3})(?<n6>(?<n7>$)|(?<n8>\b)){2,3}?|(?<n9>(?<n10>\B^)|(?<n11>\b$))(?<n12>(?<n13>(?<n14>[\q{|a|ab}])){1})??(?<n15>(?<n16>$)|(?<n17>\B))){0,0}?"#,
    "dgvi",
    JsString::from("😀a"),
    1,
    MatchExpectation {
        text: "",
        range: 0..0,
        captures: &[None; 17],
        capture_ranges: &[None; 17],
        last_index: 0,
    };
    "unicode_input_from_1_variant_17"
)]
#[test_case(
    r#"(?<n1>(?<n2>(?<n3>\B^)|(?<n4>\b$))(?<n5>(?<n6>[\p{Any}\q{|ab}]){0001,0001}?){0000,0001}(?<n7>(?<n8>$)|(?<n9>\B))|(?<n10>(?<n11>\b\B)|(?<n12>\b\B))(?<n13>(?<n14>){2,3})(?<n15>(?<n16>$)|(?<n17>\b)){2,3}?){0000}"#,
    "dgvm",
    JsString::from(""),
    0,
    MatchExpectation {
        text: "",
        range: 0..0,
        captures: &[None; 17],
        capture_ranges: &[None; 17],
        last_index: 0,
    };
    "empty_input_from_0_variant_18"
)]
#[test_case(
    r#"(?:(?<n1>(?<n2>\b\B)|(?<n3>\b\B))(?<n4>(?<n5>){2,3})(?<n6>(?<n7>$)|(?<n8>\b)){2,3}?|(?<n9>(?<n10>\B^)|(?<n11>\b$))(?<n12>(?<n13>(?<n14>[a-z\q{ab}]){1}){1,1}?){0,0}?(?<n15>(?<n16>$)|(?<n17>\B))){0000,0000}?"#,
    "dgvi",
    JsString::from("ab"),
    1,
    MatchExpectation {
        text: "",
        range: 1..1,
        captures: &[None; 17],
        capture_ranges: &[None; 17],
        last_index: 1,
    };
    "input_ab_from_1_variant_19"
)]
#[test_case(
    r#"(?<n1>(?<n2>(?<n3>^)|(?<n4>\B))(?<n5>(?<n6>[\q{a|ab}]))?(?<n7>(?<n8>\b^)|(?<n9>\B$))|(?<n10>(?<n11>^)|(?<n12>\b))(?<n13>(?<n14>){2,3})(?<n15>(?<n16>^)|(?<n17>$)){3}){0}"#,
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
    "unicode_input_from_1_variant_20"
)]
#[test_case(
    r#"(?:(?<n1>(?<n2>^)|(?<n3>\b))(?<n4>(?<n5>){2,3})(?<n6>(?<n7>^)|(?<n8>$)){3}|(?<n9>(?<n10>^)|(?<n11>\B))(?<n12>(?<n13>(?<n14>[\q{|a|ab}])){1})??(?<n15>(?<n16>\b^)|(?<n17>\B$))){0,0}?"#,
    "dgvm",
    JsString::from("ab\r\n"),
    0,
    MatchExpectation {
        text: "",
        range: 0..0,
        captures: &[None; 17],
        capture_ranges: &[None; 17],
        last_index: 0,
    };
    "input_ab_from_0_variant_21"
)]
#[test_case(
    r#"(?<n1>(?<n2>(?<n3>^)|(?<n4>\B))(?<n5>(?<n6>[\p{Any}\q{|ab}]){0001,0001}?){0000,0001}(?<n7>(?<n8>\b^)|(?<n9>\B$))|(?<n10>(?<n11>^)|(?<n12>\b))(?<n13>(?<n14>){2,3})(?<n15>(?<n16>^)|(?<n17>$)){3}){0000}"#,
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
    "input_a_from_1_variant_22"
)]
#[test_case(
    r#"(?:(?<n1>(?<n2>^)|(?<n3>\b))(?<n4>(?<n5>){2,3})(?<n6>(?<n7>^)|(?<n8>$)){3}|(?<n9>(?<n10>^)|(?<n11>\B))(?<n12>(?<n13>(?<n14>[a-z\q{ab}]){1}){1,1}?){0,0}?(?<n15>(?<n16>\b^)|(?<n17>\B$))){0000,0000}?"#,
    "dgvy",
    JsString::from("!ab!"),
    0,
    MatchExpectation {
        text: "",
        range: 0..0,
        captures: &[None; 17],
        capture_ranges: &[None; 17],
        last_index: 0,
    };
    "input_ab_from_0_variant_23"
)]
#[test_case(
    r#"(?<n1>(?<n2>(?<n3>(?<n4>^)|(?<n5>\b))\B)(?<n6>(?<n7>[\q{a|ab}]))?(?<n8>(?<n9>\b\B)|(?<n10>\b\B))|(?<n11>(?<n12>$)|(?<n13>\b)){2,3}?(?<n14>(?<n15>){2,3})(?<n16>(?<n17>\b^)|(?<n18>$))){0}"#,
    "dgvm",
    JsString::from("😀a"),
    0,
    MatchExpectation {
        text: "",
        range: 0..0,
        captures: &[None; 18],
        capture_ranges: &[None; 18],
        last_index: 0,
    };
    "unicode_input_from_0_variant_24"
)]
#[test_case(
    r#"(?<n1>(?<n2>(?<n3>(?<n4>^)|(?<n5>\b))\B)(?<n6>(?<n7>[\p{Any}\q{|ab}]){0001,0001}?){0000,0001}(?<n8>(?<n9>\b\B)|(?<n10>\b\B))|(?<n11>(?<n12>$)|(?<n13>\b)){2,3}?(?<n14>(?<n15>){2,3})(?<n16>(?<n17>\b^)|(?<n18>$))){0000}"#,
    "dgvy",
    JsString::from("ab"),
    0,
    MatchExpectation {
        text: "",
        range: 0..0,
        captures: &[None; 18],
        capture_ranges: &[None; 18],
        last_index: 0,
    };
    "input_ab_from_0_variant_25"
)]
#[test_case(
    r#"(?:(?<n1>(?<n2>$)|(?<n3>\b)){2,3}?(?<n4>(?<n5>){2,3})(?<n6>(?<n7>\b^)|(?<n8>$))|(?<n9>(?<n10>(?<n11>^)|(?<n12>\b))\B)(?<n13>(?<n14>(?<n15>[a-z\q{ab}]){1}){1,1}?){0,0}?(?<n16>(?<n17>\b\B)|(?<n18>\b\B))){0000,0000}?"#,
    "dgvim",
    JsString::from("a\n!"),
    1,
    MatchExpectation {
        text: "",
        range: 1..1,
        captures: &[None; 18],
        capture_ranges: &[None; 18],
        last_index: 1,
    };
    "input_a_from_1_variant_26"
)]
#[test_case(
    r#"(?<n1>(?<n2>(?<n3>$)|(?<n4>\B))(?<n5>(?<n6>[\q{a|ab}]))?(?<n7>(?<n8>^)|(?<n9>\b))|(?<n10>(?<n11>^)|(?<n12>$)){3}(?<n13>(?<n14>){2,3})(?<n15>(?<n16>\B^)|(?<n17>\b$))){0}"#,
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
    "input_ab_from_1_variant_27"
)]
#[test_case(
    r#"(?:(?<n1>(?<n2>^)|(?<n3>$)){3}(?<n4>(?<n5>){2,3})(?<n6>(?<n7>\B^)|(?<n8>\b$))|(?<n9>(?<n10>$)|(?<n11>\B))(?<n12>(?<n13>(?<n14>[\q{|a|ab}])){1})??(?<n15>(?<n16>^)|(?<n17>\b))){0,0}?"#,
    "dgvy",
    JsString::from("a"),
    0,
    MatchExpectation {
        text: "",
        range: 0..0,
        captures: &[None; 17],
        capture_ranges: &[None; 17],
        last_index: 0,
    };
    "input_a_from_0_variant_28"
)]
#[test_case(
    r#"(?<n1>(?<n2>(?<n3>$)|(?<n4>\B))(?<n5>(?<n6>[\p{Any}\q{|ab}]){0001,0001}?){0000,0001}(?<n7>(?<n8>^)|(?<n9>\b))|(?<n10>(?<n11>^)|(?<n12>$)){3}(?<n13>(?<n14>){2,3})(?<n15>(?<n16>\B^)|(?<n17>\b$))){0000}"#,
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
    "input_ab_from_1_variant_29"
)]
#[test_case(
    r#"(?:(?<n1>(?<n2>^)|(?<n3>$)){3}(?<n4>(?<n5>){2,3})(?<n6>(?<n7>\B^)|(?<n8>\b$))|(?<n9>(?<n10>$)|(?<n11>\B))(?<n12>(?<n13>(?<n14>[a-z\q{ab}]){1}){1,1}?){0,0}?(?<n15>(?<n16>^)|(?<n17>\b))){0000,0000}?"#,
    "dgv",
    JsString::from("ſa"),
    0,
    MatchExpectation {
        text: "",
        range: 0..0,
        captures: &[None; 17],
        capture_ranges: &[None; 17],
        last_index: 0,
    };
    "unicode_input_from_0_variant_30"
)]
#[test_case(
    r#"(?<n1>(?<n2>(?<n3>\b^)|(?<n4>\B$))(?<n5>(?<n6>[\q{a|ab}]))?(?<n7>(?<n8>$)|(?<n9>\b)){2,3}?|(?<n10>(?<n11>\b^)|(?<n12>$))(?<n13>(?<n14>){2,3})(?<n15>(?<n16>^)|(?<n17>\B))){0}"#,
    "dgvy",
    JsString::from(""),
    0,
    MatchExpectation {
        text: "",
        range: 0..0,
        captures: &[None; 17],
        capture_ranges: &[None; 17],
        last_index: 0,
    };
    "empty_input_from_0_variant_31"
)]
#[test_case(
    r#"(?:(?<n1>(?<n2>\b^)|(?<n3>$))(?<n4>(?<n5>){2,3})(?<n6>(?<n7>^)|(?<n8>\B))|(?<n9>(?<n10>\b^)|(?<n11>\B$))(?<n12>(?<n13>(?<n14>[\q{|a|ab}])){1})??(?<n15>(?<n16>$)|(?<n17>\b)){2,3}?){0,0}?"#,
    "dgvim",
    JsString::from("ab"),
    1,
    MatchExpectation {
        text: "",
        range: 1..1,
        captures: &[None; 17],
        capture_ranges: &[None; 17],
        last_index: 1,
    };
    "input_ab_from_1_variant_32"
)]
#[test_case(
    r#"(?<n1>(?<n2>(?<n3>\b^)|(?<n4>\B$))(?<n5>(?<n6>[\p{Any}\q{|ab}]){0001,0001}?){0000,0001}(?<n7>(?<n8>$)|(?<n9>\b)){2,3}?|(?<n10>(?<n11>\b^)|(?<n12>$))(?<n13>(?<n14>){2,3})(?<n15>(?<n16>^)|(?<n17>\B))){0000}"#,
    "dgv",
    JsString::from("a\n!"),
    0,
    MatchExpectation {
        text: "",
        range: 0..0,
        captures: &[None; 17],
        capture_ranges: &[None; 17],
        last_index: 0,
    };
    "input_a_from_0_variant_33"
)]
#[test_case(
    r#"(?:(?<n1>(?<n2>\b^)|(?<n3>$))(?<n4>(?<n5>){2,3})(?<n6>(?<n7>^)|(?<n8>\B))|(?<n9>(?<n10>\b^)|(?<n11>\B$))(?<n12>(?<n13>(?<n14>[a-z\q{ab}]){1}){1,1}?){0,0}?(?<n15>(?<n16>$)|(?<n17>\b)){2,3}?){0000,0000}?"#,
    "dgvi",
    JsString::from("😀a"),
    1,
    MatchExpectation {
        text: "",
        range: 0..0,
        captures: &[None; 17],
        capture_ranges: &[None; 17],
        last_index: 0,
    };
    "unicode_input_from_1_variant_34"
)]
#[test_case(
    r#"(?<n1>(?<n2>(?<n3>\b\B)|(?<n4>\b\B))(?<n5>(?<n6>[\q{a|ab}]))?(?<n7>(?<n8>^)|(?<n9>$)){3}|(?<n10>(?<n11>\B^)|(?<n12>\b$))(?<n13>(?<n14>){2,3})(?<n15>(?<n16>(?<n17>^)|(?<n18>\b))\B)){0}"#,
    "dgvim",
    JsString::from("a"),
    1,
    MatchExpectation {
        text: "",
        range: 1..1,
        captures: &[None; 18],
        capture_ranges: &[None; 18],
        last_index: 1,
    };
    "input_a_from_1_variant_35"
)]
#[test_case(
    r#"(?:(?<n1>(?<n2>\B^)|(?<n3>\b$))(?<n4>(?<n5>){2,3})(?<n6>(?<n7>(?<n8>^)|(?<n9>\b))\B)|(?<n10>(?<n11>\b\B)|(?<n12>\b\B))(?<n13>(?<n14>(?<n15>[\q{|a|ab}])){1})??(?<n16>(?<n17>^)|(?<n18>$)){3}){0,0}?"#,
    "dgv",
    JsString::from("!ab!"),
    0,
    MatchExpectation {
        text: "",
        range: 0..0,
        captures: &[None; 18],
        capture_ranges: &[None; 18],
        last_index: 0,
    };
    "input_ab_from_0_variant_36"
)]
#[test_case(
    r#"(?<n1>(?<n2>(?<n3>\b\B)|(?<n4>\b\B))(?<n5>(?<n6>[\p{Any}\q{|ab}]){0001,0001}?){0000,0001}(?<n7>(?<n8>^)|(?<n9>$)){3}|(?<n10>(?<n11>\B^)|(?<n12>\b$))(?<n13>(?<n14>){2,3})(?<n15>(?<n16>(?<n17>^)|(?<n18>\b))\B)){0000}"#,
    "dgvi",
    JsString::from("ſa"),
    1,
    MatchExpectation {
        text: "",
        range: 1..1,
        captures: &[None; 18],
        capture_ranges: &[None; 18],
        last_index: 1,
    };
    "unicode_input_from_1_variant_37"
)]
#[test_case(
    r#"(?:(?<n1>(?<n2>\B^)|(?<n3>\b$))(?<n4>(?<n5>){2,3})(?<n6>(?<n7>(?<n8>^)|(?<n9>\b))\B)|(?<n10>(?<n11>\b\B)|(?<n12>\b\B))(?<n13>(?<n14>(?<n15>[a-z\q{ab}]){1}){1,1}?){0,0}?(?<n16>(?<n17>^)|(?<n18>$)){3}){0000,0000}?"#,
    "dgvym",
    JsString::from("ab\r\n"),
    0,
    MatchExpectation {
        text: "",
        range: 0..0,
        captures: &[None; 18],
        capture_ranges: &[None; 18],
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
fn truth_table_zero_count_spine_alternative_string_classes_validate_all_erased_bodies() {
    assert_true(
        r####"var pattern="((([\\q{a|ab}])*)?|()){0}",input="aba",r=new RegExp(pattern,"dv"),m=r.exec(input);r.source===pattern&&r.flags==="dv"&&r.hasIndices&&r.unicodeSets&&(m===null||(m.input===input&&Object.hasOwn(m,"groups")&&Object.hasOwn(m.indices,"groups")&&m.groups===undefined&&m.indices.groups===undefined))&&JSON.stringify(m===null?null:[Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex,m.groups===undefined?"<undefined>":Object.keys(m.groups).map(k=>[k,m.groups[k]===undefined?"<undefined>":m.groups[k]]),m.indices.groups===undefined?"<undefined>":Object.keys(m.indices.groups).map(k=>[k,m.indices.groups[k]===undefined?"<undefined>":m.indices.groups[k]])])==="[[\"\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],[[0,0],\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],0,0,\"<undefined>\",\"<undefined>\"]""####,
    );
    assert_true(
        r####"var pattern="((([\\q{a|ab}]){0,2})?|()){0}",input="aba",r=new RegExp(pattern,"dv"),m=r.exec(input);r.source===pattern&&r.flags==="dv"&&r.hasIndices&&r.unicodeSets&&(m===null||(m.input===input&&Object.hasOwn(m,"groups")&&Object.hasOwn(m.indices,"groups")&&m.groups===undefined&&m.indices.groups===undefined))&&JSON.stringify(m===null?null:[Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex,m.groups===undefined?"<undefined>":Object.keys(m.groups).map(k=>[k,m.groups[k]===undefined?"<undefined>":m.groups[k]]),m.indices.groups===undefined?"<undefined>":Object.keys(m.indices.groups).map(k=>[k,m.indices.groups[k]===undefined?"<undefined>":m.indices.groups[k]])])==="[[\"\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],[[0,0],\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],0,0,\"<undefined>\",\"<undefined>\"]""####,
    );
    assert_true(
        r####"var pattern="^((([\\q{a|ab}])?)?|()){0}",input="aba",r=new RegExp(pattern,"dv"),m=r.exec(input);r.source===pattern&&r.flags==="dv"&&r.hasIndices&&r.unicodeSets&&(m===null||(m.input===input&&Object.hasOwn(m,"groups")&&Object.hasOwn(m.indices,"groups")&&m.groups===undefined&&m.indices.groups===undefined))&&JSON.stringify(m===null?null:[Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex,m.groups===undefined?"<undefined>":Object.keys(m.groups).map(k=>[k,m.groups[k]===undefined?"<undefined>":m.groups[k]]),m.indices.groups===undefined?"<undefined>":Object.keys(m.indices.groups).map(k=>[k,m.indices.groups[k]===undefined?"<undefined>":m.indices.groups[k]])])==="[[\"\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],[[0,0],\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],0,0,\"<undefined>\",\"<undefined>\"]""####,
    );
    assert_true(
        r####"var pattern="((([\\q{a|ab}])?)?|()){0}$",input="aba",r=new RegExp(pattern,"dv"),m=r.exec(input);r.source===pattern&&r.flags==="dv"&&r.hasIndices&&r.unicodeSets&&(m===null||(m.input===input&&Object.hasOwn(m,"groups")&&Object.hasOwn(m.indices,"groups")&&m.groups===undefined&&m.indices.groups===undefined))&&JSON.stringify(m===null?null:[Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex,m.groups===undefined?"<undefined>":Object.keys(m.groups).map(k=>[k,m.groups[k]===undefined?"<undefined>":m.groups[k]]),m.indices.groups===undefined?"<undefined>":Object.keys(m.indices.groups).map(k=>[k,m.indices.groups[k]===undefined?"<undefined>":m.indices.groups[k]])])==="[[\"\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],[[3,3],\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],3,0,\"<undefined>\",\"<undefined>\"]""####,
    );
    assert_true(
        r####"var pattern="((([\\q{ab}&&\\p{RGI_Emoji}]){0})?|()){0}",input="a😀b",r=new RegExp(pattern,"dgvy");r.lastIndex=2;var m=r.exec(input);r.source===pattern&&r.flags==="dgvy"&&r.hasIndices&&r.unicodeSets&&m.input===input&&Object.hasOwn(m,"groups")&&Object.hasOwn(m.indices,"groups")&&m.groups===undefined&&m.indices.groups===undefined&&JSON.stringify([Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex])==="[[\"\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],[[1,1],\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],1,1]""####,
    );
    assert_true(
        r####"var pattern="((([\\q{a|ab}]){0})?|z){0}",input="a😀b",r=new RegExp(pattern,"dgvy");r.lastIndex=2;var m=r.exec(input);r.source===pattern&&r.flags==="dgvy"&&r.hasIndices&&r.unicodeSets&&m.input===input&&Object.hasOwn(m,"groups")&&Object.hasOwn(m.indices,"groups")&&m.groups===undefined&&m.indices.groups===undefined&&JSON.stringify([Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex])==="[[\"\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],[[1,1],\"<undefined>\",\"<undefined>\",\"<undefined>\"],1,1]""####,
    );
    assert_true(
        r####"var pattern="((([\\q{a|ab}]){0})?|\\1){0}",input="a😀b",r=new RegExp(pattern,"dgvy");r.lastIndex=2;var m=r.exec(input);r.source===pattern&&r.flags==="dgvy"&&r.hasIndices&&r.unicodeSets&&m.input===input&&Object.hasOwn(m,"groups")&&Object.hasOwn(m.indices,"groups")&&m.groups===undefined&&m.indices.groups===undefined&&JSON.stringify([Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex])==="[[\"\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],[[1,1],\"<undefined>\",\"<undefined>\",\"<undefined>\"],1,1]""####,
    );
    assert_true(
        r####"var pattern="((([\\q{a|ab}]){0})?|([a])){0}",input="a😀b",r=new RegExp(pattern,"dgvy");r.lastIndex=2;var m=r.exec(input);r.source===pattern&&r.flags==="dgvy"&&r.hasIndices&&r.unicodeSets&&m.input===input&&Object.hasOwn(m,"groups")&&Object.hasOwn(m.indices,"groups")&&m.groups===undefined&&m.indices.groups===undefined&&JSON.stringify([Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex])==="[[\"\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],[[1,1],\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],1,1]""####,
    );
    {
        let pattern = JsString::from(r"((([\q{a|ab}])?)?|()){0,2}");
        let mut realm = Realm::default();
        crate::common::regexp_oracle::assert_matches_reference(
            &mut realm,
            &format!(
                "var marker=0;try{{new RegExp({pattern:?},'v').test('aba')}}catch{{marker=1}}finally{{marker=2}};marker"
            ),
        );
    }
}
