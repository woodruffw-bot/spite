//! Match results and consumers for quantified string alternatives in Unicode sets.

use crate::common::assert_true;
use spite_core::JsString;
use spite_runtime::{Error, Limits, Realm, Value};
use std::fmt::Write;

#[test]
fn raw_empty_single_character_v_string_repetition_state_context_and_consumers() {
    let expected_serialized_output = JsString::from(concat!(
        r#"[[false,"ab!",{"values":["",""],"indices":[[0,0],[0,0]],"index":0},0],"#,
        r#"[false,"",{"values":["",""],"indices":[[0,0],[0,0]],"index":0},0],"#,
        r#"[false,"\n",{"values":["",""],"indices":[[0,0],[0,0]],"index":0},0],"#,
        r#"[false,"x\nab",{"values":["",""],"indices":[[0,0],[0,0]],"index":0},0],"#,
        r#"[true,"ab!",{"values":["",""],"indices":[[0,0],[0,0]],"index":0},0],"#,
        r#"[true,"",{"values":["",""],"indices":[[0,0],[0,0]],"index":0},0],"#,
        r#"[true,"\n",{"values":["",""],"indices":[[0,0],[0,0]],"index":0},0],"#,
        r#"[true,"x\nab",{"values":["",""],"indices":[[0,0],[0,0]],"index":0},0]]"#,
    ));
    let expected_serialized_output_2 = JsString::from(concat!(
        r#"[[false,"ab!",{"values":["a","a"],"indices":[[0,1],[0,1]],"index":0},1],"#,
        r#"[false,"",{"values":["","<undefined>"],"indices":[[0,0],"<undefined>"],"index":0},0],"#,
        r#"[false,"\n",{"values":["","<undefined>"],"indices":[[0,0],"<undefined>"],"index":0},"#,
        r#"0],"#,
        r#"[false,"x\nab","#,
        r#"{"values":["","<undefined>"],"indices":[[0,0],"<undefined>"],"index":0},0],"#,
        r#"[true,"ab!",{"values":["a","a"],"indices":[[0,1],[0,1]],"index":0},1],"#,
        r#"[true,"",{"values":["","<undefined>"],"indices":[[0,0],"<undefined>"],"index":0},0],"#,
        r#"[true,"\n",{"values":["","<undefined>"],"indices":[[0,0],"<undefined>"],"index":0},"#,
        r#"0],"#,
        r#"[true,"x\nab","#,
        r#"{"values":["","<undefined>"],"indices":[[0,0],"<undefined>"],"index":0},0]]"#,
    ));

    assert_true(
        r#"
            let a = /([\q{a}]|)*/dv.exec('aaa!'),
                b = /([\q{a}]|)*?/dv.exec('aaa!'),
                c = /([\q{a}]|)?/dv.exec('aaa!'),
                d = /([\q{a}]|)??/dv.exec('aaa!');
            a[0] === 'aaa' && a[1] === 'a' && b[0] === '' && b[1] === undefined && c[0] === 'a' && d[0] ===
                '' && d[1] === undefined
        "#,
    );
    assert_true(
        r#"
            let a = /(?<whole>(?<unit>([\q{a}--a]|))+)/dv.exec('z'),
                b = /(?<whole>(?<unit>([\q{a}]|))*?)/dv.exec('a');
            a[0] === '' && a.groups.whole === '' && a[2] === '' && a[3] === '' && a.groups.unit === '' && a
                .indices[2][0] === 0 && a.indices[2][1] === 0 && b.groups.whole === '' && b[2] ===
                undefined && b[3] === undefined && Object.hasOwn(b, '2') && Object.hasOwn(b.indices,
                '3') && Object.hasOwn(b.groups, 'unit') && b.groups.unit === undefined && Object.hasOwn(b
                    .indices.groups, 'unit') && b.indices.groups.unit === undefined
        "#,
    );
    assert_true(
        r#"
            let a = /([\q{a}]|)+?\b/dv.exec('aa!'),
                b = /(|[\q{a}])+?/dv.exec('a'),
                c = /([\q{a}]|)+?/dv.exec('baaa');
            a.index === 0 && a[0] === 'aa' && a[1] === 'a' && a.indices[1][0] === 1 && b[0] === '' && b[
                1] === '' && c.index === 0 && c[0] === '' && c[1] === ''
        "#,
    );
    assert_true(r#"let m=/([\q{\||\)}]|)+/v.exec('|)');m[0]==='|)'&&m[1]===')'"#);
    assert_true(
        r#"
            let r = /([\q{a}]|)+/vg,
                a = r.exec('aa!'),
                b = r.exec('aa!');
            a[0] === 'aa' && r.lastIndex === 2 && b.index === 2 && b[0] === '' && b[1] === '' && r.exec(
                'aa!').index === 2 && (r.lastIndex = 4, r.exec('aa!') === null) && r.lastIndex === 0
        "#,
    );
    assert_true(
        r#"
            let r = /\B([\q{a}]|)+?\b/dvy;
            r.lastIndex = 0;
            let a = r.exec('aaa!');
            r.lastIndex = 1;
            let b = r.exec('aaa!');
            a === null && b.index === 1 && b[0] === 'aa' && b[1] === 'a' && b.indices[1][0] === 2 && r
                .lastIndex === 3
        "#,
    );
    assert_true(
        r#"
            let a = /^\b(|[\q{K}&&k])+?\b$/iv.exec('KK'),
                b = /([^\q{K|s}]|)+?/iv.exec('K!');
            a[0] === 'KK' && a[1] === 'K' && b.index === 0 && b[0] === '' && b[1] === ''
        "#,
    );
    assert_true(
        r#"
            let r = /\b([\q{\uD83D\uDE00}]|)+\B/dvy;
            r.lastIndex = 2;
            let m = r.exec('A😀😀!');
            m.index === 1 && m[0] === '😀😀' && m[1] === '😀' && m.indices[1][0] === 3 && m.indices[1][
                1] === 5 && r.lastIndex === 5
        "#,
    );
    assert_true(
        r#"
            let r = /^\B(|[\q{\uD800}])+\B$/dgmvy;
            r.lastIndex = 3;
            let m = r.exec('x\r\n\uD800\uD800\r\nz');
            m.index === 3 && m[0].length === 2 && m[1].length === 1 && m[1].charCodeAt(0) === 0xD800 && m
                .indices[1][0] === 4 && m.indices[1][1] === 5 && r.lastIndex === 5
        "#,
    );
    assert_true(
        r#"
            let trace = '',
                r = /([\q{a}]|)+?/vgy;
            r.lastIndex = {
                valueOf() {
                    trace += 'i';
                    return 0
                }
            };
            let m = r.exec({
                toString() {
                    trace += 's';
                    return 'aaa'
                }
            });
            trace === 'si' && m[0] === 'a' && m[1] === 'a' && r.lastIndex === 1
        "#,
    );
    assert_true(
        r#"
            let seen = 0,
                r = /([\q{a}]|)+/v;
            r.lastIndex = {
                valueOf() {
                    seen++;
                    return 3
                }
            };
            r.exec('aaa')[1] === 'a' && seen === 1 && typeof r.lastIndex === 'object'
        "#,
    );
    assert_true(
        r#"
            let r = /([\q{K}&&k]|)+/iv;
            r.lastIndex = 7;
            let c = new RegExp(r, 'dvy'),
                m = c.exec('K');
            c.source === '([\\q{K}&&k]|)+' && c.flags === 'dvy' && m[0] === '' && m[1] === '' && c
                .lastIndex === 0 && r.exec('K')[1] === 'K' && r.lastIndex === 7
        "#,
    );
    assert_true(
        r#"
            let a = /([\q{a}]|)+/vy,
                b = /([\q{a}]|)+/vy;
            a.lastIndex = 1;
            Object.defineProperty(a, 'lastIndex', {
                writable: false
            });
            Object.defineProperty(b, 'lastIndex', {
                writable: false
            });
            let x = false,
                y = false;
            try {
                a.exec('zaa')
            } catch (e) {
                x = e instanceof TypeError
            }
            try {
                b.exec('z')
            } catch (e) {
                y = e instanceof TypeError
            }
            x && y && a.lastIndex === 1 && b.lastIndex === 0
        "#,
    );
    assert_true(
        r#"
            let r = /(?<whole>(?<unit>([\q{a}]|))+)/dvg,
                a = [...'aa'.matchAll(r)];
            a.length === 2 && a[0].groups.whole === 'aa' && a[0].groups.unit === 'a' && a[1].index === 2 &&
                a[1].groups.unit === '' && 'aa'.replace(r, '<$<unit>>') === '<a><>' && 'aa'.split(
                    /([\q{a}]|)+/v).join('|') === '|a|' && r.lastIndex === 0
        "#,
    );
    assert_true(
        r#"
            let r = /([\q{a}]|)+/vg;
            r.lastIndex = 3;
            'ba'.search(r) === 0 && r.lastIndex === 3 && 'aa'.match(r).join('|') === 'aa|' && r
                .lastIndex === 0
        "#,
    );
    assert_true(
        r#"
            let r = /(?<unit>(|[\q{a}]))+?/dvg,
                a = [...'A😀B'.matchAll(r)];
            a.map(m => m.index).join(',') === '0,1,3,4' && a.every(m => m[1] === '' && m[2] === '' && m
                .groups.unit === '' && Object.hasOwn(m.indices, '2')) && r.lastIndex === 0
        "#,
    );
    for pattern in [
        r"([\q{\d}]|)+",
        r"([^\q{ab}]|)*",
        r"([\q{a}-b]|)+",
        r"([\q{a}]|)**",
    ] {
        let code = JsString::from(format!("marker=1; /{pattern}/v.test('a');").as_str());
        assert_true(&format!(
            r#"
                let marker = 0,
                    caught = false;
                try {{
                    eval({code:?})
                }} catch (e) {{
                    caught = e instanceof SyntaxError
                }}
                caught && marker === 0
            "#
        ));
    }
    assert_true(&format!(
        r#"
            var marker = 0,
                ok = false;
            try {{
                ok = (function() {{
                    var pattern = "([\\q{{}}]|)+",
                        rows = [],
                        metadata = true;
                    for (var multi of [false, true]) {{
                        var r = new RegExp(pattern, 'dvg' + (multi ? 'm' : ''));
                        metadata = metadata && r.source === pattern;
                        for (var input of ['ab!', '', '\n', 'x\nab']) {{
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
                    var pattern = "([\\q{{a|bc}}]|)*",
                        rows = [],
                        metadata = true;
                    for (var multi of [false, true]) {{
                        var r = new RegExp(pattern, 'dvg' + (multi ? 'm' : ''));
                        metadata = metadata && r.source === pattern;
                        for (var input of ['ab!', '', '\n', 'x\nab']) {{
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
    for source in [
        r"/(([\q{a}])|(?:X|)?)+/v.test('a')",
        r"/([\q{a}]|(\b)?)+/v.test('a')",
        r"/(?:[\q{a}]|(?:\b|()))+/v.test('a')",
        r"/([\q{a}]||)+/v.test('a')",
        r"/(|[\q{a}]|)+/v.test('a')",
        r"/([\q{a}]|b)+/v.test('ab')",
        r"/(\b[\q{a}]|)+/v.test('a')",
        r"/([\q{a}]|\b)+/v.test('a')",
        r"/([\q{a}]|)+b/v.test('ab')",
        r"/a([\q{b}]|)+/v.test('ab')",
        r"/([[\q{}]&&\p{RGI_Emoji}]|)+/v.test('')",
        r"/([[\q{a|bc}]&&\p{RGI_Emoji}]|)*/v.test('')",
        r"/([\q{a}&&[]&&\p{RGI_Emoji}]|)??/v.test('')",
        r"/\b^([\q{a}]|)+/v.test('a')",
        r"/([\q{a}]|)+$$/v.test('a')",
        r"/\b\b([\q{a}]|)+/v.test('a')",
        r"/([\q{a}]|)+\b\b/v.test('a')",
    ] {
        let mut realm = Realm::default();
        realm.eval("var marker=0").unwrap();
        crate::common::regexp_oracle::assert_matches_reference(
            &mut realm,
            &format!("marker=1;try{{{source};}}catch{{marker=2}}finally{{marker=3}}"),
        );
        assert_eq!(realm.eval("marker"), Ok(Value::Number(3.0)), "{source}");
    }
    let mut realm = Realm::default();
    realm
        .eval(r#"var kept=/(?<whole>(?<unit>([\q{a|b}]|))+)/dvg;kept.lastIndex=1"#)
        .unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval(r#"kept.source==='(?<whole>(?<unit>([\\q{a|b}]|))+)'&&kept.lastIndex===1&&kept.exec('zabb!').groups.unit==='b'&&kept.lastIndex===4"#), Ok(Value::Boolean(true)));
    // This opt-in allowance targets construction; search/capture work has a direct unit.
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm.eval("var r=7,marker=0").unwrap();
    assert!(matches!(
        realm.eval(&format!(
            r"try{{r=/{}/vg}}catch{{marker=1}}finally{{marker=2}}",
            r"\d".repeat(5000)
        )),
        Err(Error::Limit { .. })
    ));
    assert_eq!(realm.eval("marker===0&&r===7"), Ok(Value::Boolean(true)));
}

#[test]
fn raw_empty_single_character_v_string_repetition_results() {
    let mut rows = String::new();
    let mut row_count = 0;
    // Complete JSON, participation and UTF16 spans were frozen before this source.
    for (pattern, input, flags, start) in [
        (
            r#"(?<whole>(?<unit>([\q{a}--a]|))+)"#,
            JsString::from_code_units(vec![122]),
            "dvg",
            0usize,
        ),
        (
            r#"(?<whole>(?<unit>([\q{a}]|))*?)"#,
            JsString::from_code_units(vec![97]),
            "dvg",
            0usize,
        ),
        (
            r#"(?<whole>(?<unit>([\q{a}]|))+?\b)"#,
            JsString::from_code_units(vec![97, 97, 33]),
            "dvg",
            0usize,
        ),
        (
            r#"(?<whole>(?<unit>(|[\q{a}]))+?)"#,
            JsString::from_code_units(vec![97]),
            "dvg",
            0usize,
        ),
        (
            r#"(?<whole>(?<unit>([^\q{K|s}]|))+?)"#,
            JsString::from_code_units(vec![8490, 33]),
            "divg",
            0usize,
        ),
        (
            r#"^(?<whole>\b(?<unit>(|[\q{K}&&k]))+?\b)$"#,
            JsString::from_code_units(vec![8490, 75]),
            "divg",
            0usize,
        ),
        (
            r#"(?<whole>\b(?<unit>([\q{\uD83D\uDE00}]|))+\B)"#,
            JsString::from_code_units(vec![65, 55357, 56832, 55357, 56832, 33]),
            "dvy",
            2usize,
        ),
        (
            r#"^(?<whole>\B(?<unit>(|[\q{\uD800}]))+\B)$"#,
            JsString::from_code_units(vec![120, 13, 10, 55296, 55296, 13, 10, 122]),
            "dgmvy",
            3usize,
        ),
        (
            r#"(?<whole>\b(?<unit>([\q{a}--a]|))+\b)"#,
            JsString::from_code_units(vec![]),
            "dvg",
            0usize,
        ),
        (
            r#"(?<whole>\B(?<unit>([\q{a}]|))+?\b)"#,
            JsString::from_code_units(vec![97, 97, 97, 33]),
            "dvy",
            0usize,
        ),
    ] {
        let script = format!(
            r#"
                let r = new RegExp({pattern:?}, {flags:?});
                r.lastIndex = {start};
                let m = r.exec({input:?});
                JSON.stringify(m === null ? {{
                    match: null,
                    lastIndex: r.lastIndex
                }} : {{
                    matches: Array.from(m),
                    index: m.index,
                    input: m.input,
                    groups: m.groups,
                    indices: Array.from(m.indices),
                    indexGroups: m.indices.groups,
                    lastIndex: r.lastIndex,
                    source: r.source
                }}, null, 2)
            "#
        );
        let Value::String(result) = Realm::default().eval(&script).unwrap() else {
            panic!("{script}")
        };

        writeln!(
            rows,
            "{pattern:?} flags={flags:?} input_units={:?} lastIndex={start}\n{result}\n",
            input.code_units(),
            result = result.to_utf8().expect("JSON output must be valid UTF-8"),
        )
        .unwrap();
        row_count += 1;
    }
    assert_eq!(row_count, 10);
    insta::assert_snapshot!(rows);
}
