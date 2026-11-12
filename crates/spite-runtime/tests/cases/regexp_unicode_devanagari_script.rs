//! Exact Devanagari Script aliases, complements, consumers, and Unicode 18 changes.
use spite_core::JsString;
use spite_runtime::{Error, Limits, Realm, Value};
use std::fmt::Write;

use crate::common::assert_true;

#[test]
fn unicode_devanagari_script_aliases_complements_and_fold_modes() {
    assert_true(
        r#"
            let patterns = ["^\\p{Script=Deva}$", "^\\P{Script=Deva}$", "^[\\p{Script=Deva}]$",
                    "^[\\P{Script=Deva}]$", "^[^\\p{Script=Deva}]$", "^[^\\P{Script=Deva}]$"
                ],
                points = [0, 10, 32, 65, 97, 181, 768, 2325, 2326, 2304, 2384, 2404, 2416, 2303, 2432,
                    1488, 8205, 55296, 56320, 57344, 64976, 72448, 72457, 128512, 1114111
                ],
                flags = "u";
            patterns.map(pattern => {
                    let r = new RegExp(pattern, flags);
                    return points.map(cp => r.test(String.fromCodePoint(cp)) ? '1' : '0').join('')
                }).join('|') ===
                "0000000111101000000001100|1111111000010111111110011|0000000111101000000001100|1111111000010111111110011|1111111000010111111110011|0000000111101000000001100"
        "#,
    );
    assert_true(
        r#"
            let patterns = ["^\\p{Script=Deva}$", "^\\P{Script=Deva}$", "^[\\p{Script=Deva}]$",
                    "^[\\P{Script=Deva}]$", "^[^\\p{Script=Deva}]$", "^[^\\P{Script=Deva}]$"
                ],
                points = [0, 10, 32, 65, 97, 181, 768, 2325, 2326, 2304, 2384, 2404, 2416, 2303, 2432,
                    1488, 8205, 55296, 56320, 57344, 64976, 72448, 72457, 128512, 1114111
                ],
                flags = "ui";
            patterns.map(pattern => {
                    let r = new RegExp(pattern, flags);
                    return points.map(cp => r.test(String.fromCodePoint(cp)) ? '1' : '0').join('')
                }).join('|') ===
                "0000000111101000000001100|1111111000010111111110011|0000000111101000000001100|1111111000010111111110011|1111111000010111111110011|0000000111101000000001100"
        "#,
    );
    assert_true(
        r#"
            let patterns = ["^\\p{Script=Deva}$", "^\\P{Script=Deva}$", "^[\\p{Script=Deva}]$",
                    "^[\\P{Script=Deva}]$", "^[^\\p{Script=Deva}]$", "^[^\\P{Script=Deva}]$"
                ],
                points = [0, 10, 32, 65, 97, 181, 768, 2325, 2326, 2304, 2384, 2404, 2416, 2303, 2432,
                    1488, 8205, 55296, 56320, 57344, 64976, 72448, 72457, 128512, 1114111
                ],
                flags = "v";
            patterns.map(pattern => {
                    let r = new RegExp(pattern, flags);
                    return points.map(cp => r.test(String.fromCodePoint(cp)) ? '1' : '0').join('')
                }).join('|') ===
                "0000000111101000000001100|1111111000010111111110011|0000000111101000000001100|1111111000010111111110011|1111111000010111111110011|0000000111101000000001100"
        "#,
    );
    assert_true(
        r#"
            let patterns = ["^\\p{Script=Deva}$", "^\\P{Script=Deva}$", "^[\\p{Script=Deva}]$",
                    "^[\\P{Script=Deva}]$", "^[^\\p{Script=Deva}]$", "^[^\\P{Script=Deva}]$"
                ],
                points = [0, 10, 32, 65, 97, 181, 768, 2325, 2326, 2304, 2384, 2404, 2416, 2303, 2432,
                    1488, 8205, 55296, 56320, 57344, 64976, 72448, 72457, 128512, 1114111
                ],
                flags = "vi";
            patterns.map(pattern => {
                    let r = new RegExp(pattern, flags);
                    return points.map(cp => r.test(String.fromCodePoint(cp)) ? '1' : '0').join('')
                }).join('|') ===
                "0000000111101000000001100|1111111000010111111110011|0000000111101000000001100|1111111000010111111110011|1111111000010111111110011|0000000111101000000001100"
        "#,
    );
    assert_true(
        r#"
            let patterns = ["^\\p{Script=Devanagari}$", "^\\P{Script=Devanagari}$",
                    "^[\\p{Script=Devanagari}]$", "^[\\P{Script=Devanagari}]$",
                    "^[^\\p{Script=Devanagari}]$", "^[^\\P{Script=Devanagari}]$"
                ],
                points = [0, 10, 32, 65, 97, 181, 768, 2325, 2326, 2304, 2384, 2404, 2416, 2303, 2432,
                    1488, 8205, 55296, 56320, 57344, 64976, 72448, 72457, 128512, 1114111
                ],
                flags = "u";
            patterns.map(pattern => {
                    let r = new RegExp(pattern, flags);
                    return points.map(cp => r.test(String.fromCodePoint(cp)) ? '1' : '0').join('')
                }).join('|') ===
                "0000000111101000000001100|1111111000010111111110011|0000000111101000000001100|1111111000010111111110011|1111111000010111111110011|0000000111101000000001100"
        "#,
    );
    assert_true(
        r#"
            let patterns = ["^\\p{Script=Devanagari}$", "^\\P{Script=Devanagari}$",
                    "^[\\p{Script=Devanagari}]$", "^[\\P{Script=Devanagari}]$",
                    "^[^\\p{Script=Devanagari}]$", "^[^\\P{Script=Devanagari}]$"
                ],
                points = [0, 10, 32, 65, 97, 181, 768, 2325, 2326, 2304, 2384, 2404, 2416, 2303, 2432,
                    1488, 8205, 55296, 56320, 57344, 64976, 72448, 72457, 128512, 1114111
                ],
                flags = "ui";
            patterns.map(pattern => {
                    let r = new RegExp(pattern, flags);
                    return points.map(cp => r.test(String.fromCodePoint(cp)) ? '1' : '0').join('')
                }).join('|') ===
                "0000000111101000000001100|1111111000010111111110011|0000000111101000000001100|1111111000010111111110011|1111111000010111111110011|0000000111101000000001100"
        "#,
    );
    assert_true(
        r#"
            let patterns = ["^\\p{Script=Devanagari}$", "^\\P{Script=Devanagari}$",
                    "^[\\p{Script=Devanagari}]$", "^[\\P{Script=Devanagari}]$",
                    "^[^\\p{Script=Devanagari}]$", "^[^\\P{Script=Devanagari}]$"
                ],
                points = [0, 10, 32, 65, 97, 181, 768, 2325, 2326, 2304, 2384, 2404, 2416, 2303, 2432,
                    1488, 8205, 55296, 56320, 57344, 64976, 72448, 72457, 128512, 1114111
                ],
                flags = "v";
            patterns.map(pattern => {
                    let r = new RegExp(pattern, flags);
                    return points.map(cp => r.test(String.fromCodePoint(cp)) ? '1' : '0').join('')
                }).join('|') ===
                "0000000111101000000001100|1111111000010111111110011|0000000111101000000001100|1111111000010111111110011|1111111000010111111110011|0000000111101000000001100"
        "#,
    );
    assert_true(
        r#"
            let patterns = ["^\\p{Script=Devanagari}$", "^\\P{Script=Devanagari}$",
                    "^[\\p{Script=Devanagari}]$", "^[\\P{Script=Devanagari}]$",
                    "^[^\\p{Script=Devanagari}]$", "^[^\\P{Script=Devanagari}]$"
                ],
                points = [0, 10, 32, 65, 97, 181, 768, 2325, 2326, 2304, 2384, 2404, 2416, 2303, 2432,
                    1488, 8205, 55296, 56320, 57344, 64976, 72448, 72457, 128512, 1114111
                ],
                flags = "vi";
            patterns.map(pattern => {
                    let r = new RegExp(pattern, flags);
                    return points.map(cp => r.test(String.fromCodePoint(cp)) ? '1' : '0').join('')
                }).join('|') ===
                "0000000111101000000001100|1111111000010111111110011|0000000111101000000001100|1111111000010111111110011|1111111000010111111110011|0000000111101000000001100"
        "#,
    );
    assert_true(
        r#"
            let patterns = ["^\\p{sc=Deva}$", "^\\P{sc=Deva}$", "^[\\p{sc=Deva}]$", "^[\\P{sc=Deva}]$",
                    "^[^\\p{sc=Deva}]$", "^[^\\P{sc=Deva}]$"
                ],
                points = [0, 10, 32, 65, 97, 181, 768, 2325, 2326, 2304, 2384, 2404, 2416, 2303, 2432,
                    1488, 8205, 55296, 56320, 57344, 64976, 72448, 72457, 128512, 1114111
                ],
                flags = "u";
            patterns.map(pattern => {
                    let r = new RegExp(pattern, flags);
                    return points.map(cp => r.test(String.fromCodePoint(cp)) ? '1' : '0').join('')
                }).join('|') ===
                "0000000111101000000001100|1111111000010111111110011|0000000111101000000001100|1111111000010111111110011|1111111000010111111110011|0000000111101000000001100"
        "#,
    );
    assert_true(
        r#"
            let patterns = ["^\\p{sc=Deva}$", "^\\P{sc=Deva}$", "^[\\p{sc=Deva}]$", "^[\\P{sc=Deva}]$",
                    "^[^\\p{sc=Deva}]$", "^[^\\P{sc=Deva}]$"
                ],
                points = [0, 10, 32, 65, 97, 181, 768, 2325, 2326, 2304, 2384, 2404, 2416, 2303, 2432,
                    1488, 8205, 55296, 56320, 57344, 64976, 72448, 72457, 128512, 1114111
                ],
                flags = "ui";
            patterns.map(pattern => {
                    let r = new RegExp(pattern, flags);
                    return points.map(cp => r.test(String.fromCodePoint(cp)) ? '1' : '0').join('')
                }).join('|') ===
                "0000000111101000000001100|1111111000010111111110011|0000000111101000000001100|1111111000010111111110011|1111111000010111111110011|0000000111101000000001100"
        "#,
    );
    assert_true(
        r#"
            let patterns = ["^\\p{sc=Deva}$", "^\\P{sc=Deva}$", "^[\\p{sc=Deva}]$", "^[\\P{sc=Deva}]$",
                    "^[^\\p{sc=Deva}]$", "^[^\\P{sc=Deva}]$"
                ],
                points = [0, 10, 32, 65, 97, 181, 768, 2325, 2326, 2304, 2384, 2404, 2416, 2303, 2432,
                    1488, 8205, 55296, 56320, 57344, 64976, 72448, 72457, 128512, 1114111
                ],
                flags = "v";
            patterns.map(pattern => {
                    let r = new RegExp(pattern, flags);
                    return points.map(cp => r.test(String.fromCodePoint(cp)) ? '1' : '0').join('')
                }).join('|') ===
                "0000000111101000000001100|1111111000010111111110011|0000000111101000000001100|1111111000010111111110011|1111111000010111111110011|0000000111101000000001100"
        "#,
    );
    assert_true(
        r#"
            let patterns = ["^\\p{sc=Deva}$", "^\\P{sc=Deva}$", "^[\\p{sc=Deva}]$", "^[\\P{sc=Deva}]$",
                    "^[^\\p{sc=Deva}]$", "^[^\\P{sc=Deva}]$"
                ],
                points = [0, 10, 32, 65, 97, 181, 768, 2325, 2326, 2304, 2384, 2404, 2416, 2303, 2432,
                    1488, 8205, 55296, 56320, 57344, 64976, 72448, 72457, 128512, 1114111
                ],
                flags = "vi";
            patterns.map(pattern => {
                    let r = new RegExp(pattern, flags);
                    return points.map(cp => r.test(String.fromCodePoint(cp)) ? '1' : '0').join('')
                }).join('|') ===
                "0000000111101000000001100|1111111000010111111110011|0000000111101000000001100|1111111000010111111110011|1111111000010111111110011|0000000111101000000001100"
        "#,
    );
    assert_true(
        r#"
            let patterns = ["^\\p{sc=Devanagari}$", "^\\P{sc=Devanagari}$", "^[\\p{sc=Devanagari}]$",
                    "^[\\P{sc=Devanagari}]$", "^[^\\p{sc=Devanagari}]$", "^[^\\P{sc=Devanagari}]$"
                ],
                points = [0, 10, 32, 65, 97, 181, 768, 2325, 2326, 2304, 2384, 2404, 2416, 2303, 2432,
                    1488, 8205, 55296, 56320, 57344, 64976, 72448, 72457, 128512, 1114111
                ],
                flags = "u";
            patterns.map(pattern => {
                    let r = new RegExp(pattern, flags);
                    return points.map(cp => r.test(String.fromCodePoint(cp)) ? '1' : '0').join('')
                }).join('|') ===
                "0000000111101000000001100|1111111000010111111110011|0000000111101000000001100|1111111000010111111110011|1111111000010111111110011|0000000111101000000001100"
        "#,
    );
    assert_true(
        r#"
            let patterns = ["^\\p{sc=Devanagari}$", "^\\P{sc=Devanagari}$", "^[\\p{sc=Devanagari}]$",
                    "^[\\P{sc=Devanagari}]$", "^[^\\p{sc=Devanagari}]$", "^[^\\P{sc=Devanagari}]$"
                ],
                points = [0, 10, 32, 65, 97, 181, 768, 2325, 2326, 2304, 2384, 2404, 2416, 2303, 2432,
                    1488, 8205, 55296, 56320, 57344, 64976, 72448, 72457, 128512, 1114111
                ],
                flags = "ui";
            patterns.map(pattern => {
                    let r = new RegExp(pattern, flags);
                    return points.map(cp => r.test(String.fromCodePoint(cp)) ? '1' : '0').join('')
                }).join('|') ===
                "0000000111101000000001100|1111111000010111111110011|0000000111101000000001100|1111111000010111111110011|1111111000010111111110011|0000000111101000000001100"
        "#,
    );
    assert_true(
        r#"
            let patterns = ["^\\p{sc=Devanagari}$", "^\\P{sc=Devanagari}$", "^[\\p{sc=Devanagari}]$",
                    "^[\\P{sc=Devanagari}]$", "^[^\\p{sc=Devanagari}]$", "^[^\\P{sc=Devanagari}]$"
                ],
                points = [0, 10, 32, 65, 97, 181, 768, 2325, 2326, 2304, 2384, 2404, 2416, 2303, 2432,
                    1488, 8205, 55296, 56320, 57344, 64976, 72448, 72457, 128512, 1114111
                ],
                flags = "v";
            patterns.map(pattern => {
                    let r = new RegExp(pattern, flags);
                    return points.map(cp => r.test(String.fromCodePoint(cp)) ? '1' : '0').join('')
                }).join('|') ===
                "0000000111101000000001100|1111111000010111111110011|0000000111101000000001100|1111111000010111111110011|1111111000010111111110011|0000000111101000000001100"
        "#,
    );
    assert_true(
        r#"
            let patterns = ["^\\p{sc=Devanagari}$", "^\\P{sc=Devanagari}$", "^[\\p{sc=Devanagari}]$",
                    "^[\\P{sc=Devanagari}]$", "^[^\\p{sc=Devanagari}]$", "^[^\\P{sc=Devanagari}]$"
                ],
                points = [0, 10, 32, 65, 97, 181, 768, 2325, 2326, 2304, 2384, 2404, 2416, 2303, 2432,
                    1488, 8205, 55296, 56320, 57344, 64976, 72448, 72457, 128512, 1114111
                ],
                flags = "vi";
            patterns.map(pattern => {
                    let r = new RegExp(pattern, flags);
                    return points.map(cp => r.test(String.fromCodePoint(cp)) ? '1' : '0').join('')
                }).join('|') ===
                "0000000111101000000001100|1111111000010111111110011|0000000111101000000001100|1111111000010111111110011|1111111000010111111110011|0000000111101000000001100"
        "#,
    );
}

#[test]
fn unicode_devanagari_script_unicode18_additions() {
    assert_true(
        r#"
            let patterns = ["^\\p{sc=Deva}$", "^\\P{sc=Deva}$", "^[\\p{sc=Deva}]$", "^[\\P{sc=Deva}]$",
                    "^[^\\p{sc=Deva}]$", "^[^\\P{sc=Deva}]$"
                ],
                points = [72458],
                flags = "u";
            patterns.map(pattern => {
                let r = new RegExp(pattern, flags);
                return points.map(cp => r.test(String.fromCodePoint(cp)) ? '1' : '0').join('')
            }).join('|') === "1|0|1|0|0|1"
        "#,
    );
    assert_true(
        r#"
            let patterns = ["^\\p{sc=Deva}$", "^\\P{sc=Deva}$", "^[\\p{sc=Deva}]$", "^[\\P{sc=Deva}]$",
                    "^[^\\p{sc=Deva}]$", "^[^\\P{sc=Deva}]$"
                ],
                points = [72458],
                flags = "ui";
            patterns.map(pattern => {
                let r = new RegExp(pattern, flags);
                return points.map(cp => r.test(String.fromCodePoint(cp)) ? '1' : '0').join('')
            }).join('|') === "1|0|1|0|0|1"
        "#,
    );
    assert_true(
        r#"
            let patterns = ["^\\p{sc=Deva}$", "^\\P{sc=Deva}$", "^[\\p{sc=Deva}]$", "^[\\P{sc=Deva}]$",
                    "^[^\\p{sc=Deva}]$", "^[^\\P{sc=Deva}]$"
                ],
                points = [72458],
                flags = "v";
            patterns.map(pattern => {
                let r = new RegExp(pattern, flags);
                return points.map(cp => r.test(String.fromCodePoint(cp)) ? '1' : '0').join('')
            }).join('|') === "1|0|1|0|0|1"
        "#,
    );
    assert_true(
        r#"
            let patterns = ["^\\p{sc=Deva}$", "^\\P{sc=Deva}$", "^[\\p{sc=Deva}]$", "^[\\P{sc=Deva}]$",
                    "^[^\\p{sc=Deva}]$", "^[^\\P{sc=Deva}]$"
                ],
                points = [72458],
                flags = "vi";
            patterns.map(pattern => {
                let r = new RegExp(pattern, flags);
                return points.map(cp => r.test(String.fromCodePoint(cp)) ? '1' : '0').join('')
            }).join('|') === "1|0|1|0|0|1"
        "#,
    );
}

#[test]
fn unicode_devanagari_script_results_snapshot() {
    let mut rows = String::new();
    for name in [
        "Script=Devanagari",
        "Script=Deva",
        "sc=Devanagari",
        "sc=Deva",
    ] {
        for body in [
            format!(r"\p{{{name}}}"),
            format!(r"\P{{{name}}}"),
            format!(r"(?<x>[\p{{{name}}}K])"),
        ] {
            let pattern = JsString::from(body.as_str());
            for flags in ["dug", "diug", "divg", "divy"] {
                for input in ["कखΣ😀𑬀\n", "X\r\nक\u{2028}א", ""] {
                    let input = JsString::from(input);
                    for start in [0, 1, 3] {
                        let script = format!(
                            r#"
                                let r = new RegExp({pattern:?}, {flags:?});
                                r.lastIndex = {start};
                                let m = r.exec({input:?});
                                JSON.stringify({{
                                    m,
                                    index: m?.index,
                                    groups: m?.groups,
                                    indices: m?.indices,
                                    indexGroups: m?.indices?.groups,
                                    lastIndex: r.lastIndex,
                                    source: r.source
                                }})
                            "#
                        );
                        let Value::String(result) = Realm::default().eval(&script).unwrap() else {
                            panic!("{script}")
                        };
                        writeln!(
                            rows,
                            "{body:?} flags={flags:?} input={input:?} lastIndex={start} => {result}"
                        ,
result = result.to_utf8().expect("JSON output must be valid UTF-8"),
)
                        .unwrap();
                    }
                }
            }
        }
    }
    insta::assert_snapshot!(rows);
}
#[test]
fn unicode_devanagari_script_captures_context_and_pair_starts() {
    for mode in ["u", "v"] {
        assert_true(&format!(
            r#"
                let r = /^(?<x>([\p{{sc=Deva}}K]))$/dim{mode}g,
                    m = r.exec('A\r\nक\u2028');
                m.index === 3 && m[1] === 'क' && m[2] === 'क' && m.indices.groups.x === m.indices[1] && m
                    .indices[2][1] === 4 && r.lastIndex === 4
            "#
        ));
        assert_true(&format!(
            r#"
                let r = /(?<__proto__>\p{{Script=Devanagari}})/d{mode}y;
                r.lastIndex = 1;
                let m = r.exec('𑬀A');
                m.index === 0 && m[1] === '𑬀' && m.indices[1][1] === 2 && r.lastIndex === 2 && Object
                    .getPrototypeOf(m.groups) === null && m.indices.groups.__proto__ === m.indices[1]
            "#
        ));
        assert_true(&format!(
            r#"
                let r = /\P{{sc=Devanagari}}/d{mode}g;
                r.lastIndex = 1;
                let m = r.exec('𑬀\uD800');
                m.index === 2 && m[0] === '\uD800' && r.lastIndex === 3
            "#
        ));
        assert_true(&format!(
            r"let r=/\P{{Script=Deva}}/{mode}y;r.lastIndex=1;r.exec('𑬀\uD800')===null&&r.lastIndex===0"
        ));
    }
}

#[test]
fn unicode_devanagari_script_consumers_preserve_original_text() {
    for mode in ["u", "v"] {
        assert_true(&format!(
            r#"
                let a = [...'क😀Aख'.matchAll(/(?<x>\P{{sc=Devanagari}})/d{mode}g)];
                a.length === 2 && a[0][1] === '😀' && a[0].index === 1 && a[0].indices[1][1] === 3 && a[1][
                    1
                ] === 'A' && a[1].index === 3
            "#
        ));
        assert_true(&format!(
            r"'कखA'.replace(/(?<x>\p{{sc=Deva}})/{mode}g,'<$1:$<x>>')==='<क:क><ख:ख>A'"
        ));
        assert_true(&format!(
            r#"
                'AकखA'.split(/(\p{{Script=Devanagari}})/{mode}).join('|') === 'A|क||ख|A' && 'A😀क'
                    .search(/\p{{sc=Deva}}/{mode}) === 3 && 'कखA'.match(
                        /\p{{Script=Deva}}/{mode}g).join('') === 'कख'
            "#
        ));
        assert_true(&format!(
            r#"
                let t = '',
                    x = 'कखAक'.replace(/(\p{{sc=Devanagari}})/{mode}g, (m, c, p) => {{
                        t += p;
                        return '_'
                    }});
                x === '__A_' && t === '013'
            "#
        ));
    }
}

#[test]
fn unicode_devanagari_script_clones_coercions_and_strict_writes() {
    assert_true(
        r#"
            let r = /^(?<x>\p{sc=Deva})$/dmu,
                c = new RegExp(r, 'divy');
            c.source === '^(?<x>\\p{sc=Deva})$' && c.unicodeSets && !c.unicode && !c.multiline && c.exec(
                'क').groups.x === 'क' && c.lastIndex === 1
        "#,
    );
    assert_true(
        r#"
            let t = '',
                r = /\p{Script=Devanagari}/uy;
            r.lastIndex = {
                valueOf() {
                    t += 'i';
                    return 1
                }
            };
            let m = r.exec({
                toString() {
                    t += 's';
                    return '𑬀'
                }
            });
            t === 'si' && m[0] === '𑬀' && m.index === 0 && r.lastIndex === 2
        "#,
    );
    assert_true(
        r#"
            let r = /\p{sc=Devanagari}/vy;
            Object.defineProperty(r, 'lastIndex', {
                writable: false
            });
            let caught = false;
            try {
                r.exec('क')
            } catch (e) {
                caught = e instanceof TypeError
            }
            caught && r.lastIndex === 0
        "#,
    );
    assert_true(
        r#"
            let r = /\P{Script=Deva}/vy;
            r.lastIndex = 1;
            Object.defineProperty(r, 'lastIndex', {
                writable: false
            });
            let caught = false;
            try {
                r.exec('𑬀\uD800')
            } catch (e) {
                caught = e instanceof TypeError
            }
            caught && r.lastIndex === 1
        "#,
    );
}

#[test]
fn unicode_devanagari_script_opted_work_and_remaining_sets() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval("let r=/\\p{Script=Devanagari}/ug,marker=0,text='A'.repeat(10000);r.lastIndex=1")
        .unwrap();
    assert!(matches!(
        realm.eval("try{r.exec(text)}catch{marker=1}finally{marker=2}"),
        Err(Error::Limit { .. })
    ));
    assert_eq!(
        realm.eval("marker===0&&r.lastIndex===1"),
        Ok(Value::Boolean(true))
    );
    assert_true(r"/\p{Script_Extensions=Devanagari}/u.exec('A')===null");
    assert_true(r"/[\p{sc=Deva}&&A]/v.exec('A')===null");
    assert_true(r"/[\p{sc=Deva}--A]/v.exec('A')===null");
    assert_true(
        r#"
            let s = String.fromCodePoint(0x915),
                m = /[\p{sc=Deva}--A]/v.exec(s);
            m !== null && m.length === 1 && m[0] === s && m.index === 0 && m.input === s
        "#,
    );
    for source in [
        r"/\p{sc=Deva}+b/u.exec('AA')",
        r"/\p{sc=Deva}A/v.exec('AA')",
    ] {
        crate::common::regexp_oracle::assert_matches_reference(&mut Realm::default(), source);
    }
}

#[test]
fn unicode_devanagari_script_repetitions_retain_last_capture_and_outside_words() {
    for mode in ["u", "v"] {
        assert_true(&format!(
            r#"
                let r = /\B(?<x>\p{{sc=Deva}})+\B/d{mode}g,
                    m = r.exec('µ𑬀𑬀𑬀µ');
                m.index === 1 && m[0] === '𑬀𑬀𑬀' && m.groups.x === '𑬀' && m.indices.groups.x[0] === 5 && m
                    .indices.groups.x[1] === 7 && r.lastIndex === 7
            "#
        ));
    }
}

#[test]
fn unicode_devanagari_script_direct_eval_preserves_surrounding_execution() {
    assert_true(
        r#"
            let marker = 0,
                result;
            try {
                result = eval("marker=1; /\\p{Script=Devanagari}/u.test('क');")
            } catch {
                marker = 2
            } finally {
                marker = 3
            }
            marker === 3 && result === true
        "#,
    );
}
