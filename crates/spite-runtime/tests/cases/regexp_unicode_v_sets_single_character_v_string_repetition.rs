//! Match results and consumers for quantified string alternatives in Unicode sets.

use crate::common::assert_true;
use spite_core::JsString;
use spite_runtime::{Error, Limits, Realm, Value};
use std::fmt::Write;

#[test]
fn single_character_v_string_repetition_state_context_and_consumers() {
    let expected_serialized_output = JsString::from(concat!(
        r#"[[false,"ab!",{"values":["a"],"indices":[[0,1]],"index":0},1],"#,
        r#"[false,"",{"values":[""],"indices":[[0,0]],"index":0},0],"#,
        r#"[false,"\n",{"values":[""],"indices":[[0,0]],"index":0},0],"#,
        r#"[false,"x\nab",{"values":[""],"indices":[[0,0]],"index":0},0],"#,
        r#"[true,"ab!",{"values":["a"],"indices":[[0,1]],"index":0},1],"#,
        r#"[true,"",{"values":[""],"indices":[[0,0]],"index":0},0],"#,
        r#"[true,"\n",{"values":[""],"indices":[[0,0]],"index":0},0],"#,
        r#"[true,"x\nab",{"values":[""],"indices":[[0,0]],"index":0},0]]"#,
    ));
    let expected_serialized_output_2 = JsString::from(concat!(
        r#"[[false,"ab!",{"values":[""],"indices":[[0,0]],"index":0},0],"#,
        r#"[false,"",{"values":[""],"indices":[[0,0]],"index":0},0],"#,
        r#"[false,"\n",{"values":[""],"indices":[[0,0]],"index":0},0],"#,
        r#"[false,"x\nab",{"values":[""],"indices":[[0,0]],"index":0},0],"#,
        r#"[true,"ab!",{"values":[""],"indices":[[0,0]],"index":0},0],"#,
        r#"[true,"",{"values":[""],"indices":[[0,0]],"index":0},0],"#,
        r#"[true,"\n",{"values":[""],"indices":[[0,0]],"index":0},0],"#,
        r#"[true,"x\nab",{"values":[""],"indices":[[0,0]],"index":0},0]]"#,
    ));

    assert_true(
        r#"
            let a = /[\q{a|b}]+/v.exec('zabba!'),
                b = /[\q{a|b}]+?/v.exec('zabba!');
            a.index === 1 && a[0] === 'abba' && b.index === 1 && b[0] === 'a'
        "#,
    );
    assert_true(
        r#"
            /^[a\q{b}]*$/v.test('abba') && /^[a\q{b}]?$/v.test('') && /^[a\q{b}]?$/v.test('b') && !
                /^[a\q{b}]?$/v.test('bb')
        "#,
    );
    assert_true(
        r#"
            let r = /[\q{a}]{1,2}\b/v,
                m = r.exec('aaa');
            m.index === 1 && m[0] === 'aa' && /[\q{a}]{1,2}?\b/v.exec('aaa')[0] === 'aa'
        "#,
    );
    assert_true(
        r#"
            let r = /[\q{a}]{1,2}\b/vy;
            r.lastIndex = 0;
            let a = r.exec('aaa');
            r.lastIndex = 1;
            let b = r.exec('aaa');
            a === null && b.index === 1 && b[0] === 'aa' && r.lastIndex === 3
        "#,
    );
    assert_true(
        r#"
            let r = /\b[\q{a}--a]{0}\b/dvg,
                m = r.exec('a');
            m.index === 0 && m[0] === '' && m.indices[0][0] === 0 && m.indices[0][1] === 0 && r
                .lastIndex === 0 && /\B[\q{a}--a]{0}\B/v.test('')
        "#,
    );

    assert_true(
        r#"/^[\q{a}&&[]]*$/v.test('')&&!/^[\q{a}&&[]]*$/v.test('a')&&!/[\q{a}&&[]]+/v.test('')"#,
    );
    assert_true(r#"/[\q{K}&&k]+/iv.exec('!KKk!')[0]==='KKk'&&!/[\q{K}&&k]+/v.test('KkK')"#);
    assert_true(r#"let r=/[^\q{K|s}]+?/iv,m=r.exec('Kſ!');m.index===2&&m[0]==='!'&&!r.test('Kſ')"#);
    assert_true(
        r#"/^\b[\q{K}&&k]+\b$/ivm.exec('x\nKKk\nz')[0]==='KKk'&&!/^\b[\q{K}&&k]+\b$/iv.test('x\nKKk\nz')"#,
    );
    assert_true(
        r#"let m=/^\B[[\q{\r|\n|\u2028}]--[\q{\r}]]{1,2}\B$/mv.exec('x\r\n\u2028y');m.index===2&&m[0]==='\n'"#,
    );
    assert_true(
        r#"/[\q{\{|\}|\?}]{1,2}?/v.exec('{}?')[0]==='{'&&/[\q{\{|\}|\?}]{1,2}/v.exec('{}?')[0]==='{}'"#,
    );
    assert_true(
        r#"
            let r = /(?<whole>(\b[\q{\uD83D\uDE00|x}&&[^\q{x}]]{1,2}\B))/dvy;
            r.lastIndex = 2;
            let m = r.exec('A😀😀!');
            m.index === 1 && m[0] === '😀😀' && m.groups.whole === '😀😀' && m.indices[0][0] === 1 && m
                .indices[0][1] === 5 && r.lastIndex === 5
        "#,
    );
    assert_true(
        r#"
            let m = /[\q{\uD800|x}--x]{2}/v.exec('A\uD800\uD800B');
            m.index === 1 && m[0].length === 2 && m[0].charCodeAt(0) === 0xD800 && m[0].charCodeAt(1) ===
                0xD800
        "#,
    );
    assert_true(
        r#"
            let r = /(?<whole>([\q{a|b}]+))/dvg,
                m = r.exec('zabba!');
            m.length === 3 && m[0] === 'abba' && m[1] === 'abba' && m[2] === 'abba' && m.groups.whole ===
                'abba' && m.indices[0][0] === 1 && m.indices[0][1] === 5 && m.indices.groups.whole[1] ===
                5 && r.lastIndex === 5
        "#,
    );
    assert_true(
        r#"
            let trace = '',
                r = /[\q{a|b}]+/vy;
            r.lastIndex = {
                valueOf() {
                    trace += 'i';
                    return 1
                }
            };
            let m = r.exec({
                toString() {
                    trace += 's';
                    return 'zabb'
                }
            });
            trace === 'si' && m[0] === 'abb' && m.index === 1 && r.lastIndex === 4
        "#,
    );
    assert_true(
        r#"
            let seen = 0,
                r = /[\q{a|b}]+/v;
            r.lastIndex = {
                valueOf() {
                    seen++;
                    return 3
                }
            };
            r.exec('zab').index === 1 && seen === 1 && typeof r.lastIndex === 'object'
        "#,
    );
    assert_true(
        r#"
            let r = /[\q{K}]+/iv;
            r.lastIndex = 7;
            let c = new RegExp(r, 'dvy');
            c.source === '[\\q{K}]+' && c.flags === 'dvy' && c.lastIndex === 0 && c.exec('K') === null && r
                .exec('K')[0] === 'K' && r.lastIndex === 7
        "#,
    );
    assert_true(
        r#"
            let a = /[\q{a}]+/vy,
                b = /[\q{a}]+/vy;
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
                a.exec('za')
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
            let r = /(?<whole>([\q{a|b}]+))/dvg,
                a = [...'a-bb!'.matchAll(r)];
            a.length === 2 && a[0].index === 0 && a[1].index === 2 && a[1].groups.whole === 'bb' && 'a-bb!'
                .replace(r, '<$<whole>>') === '<a>-<bb>!' && 'xa-bbz'.split(/[\q{a|b}]+/v).join('|') ===
                'x|-|z'
        "#,
    );
    assert_true(
        r#"
            let r = /[\q{a|b}]+/vg;
            r.lastIndex = 1;
            'zaab'.search(r) === 1 && r.lastIndex === 1 && 'ab-ab'.match(r).join('|') === 'ab|ab' && r
                .lastIndex === 0
        "#,
    );
    assert_true(
        r#"
            let r = /[\q{a}--a]*/vg,
                a = [...'A😀B'.matchAll(r)];
            a.map(m => m.index).join(',') === '0,1,3,4' && r.lastIndex === 0 && 'A😀B'.replace(r, '|') ===
                '|A|😀|B|'
        "#,
    );
    for pattern in [
        r"[\q{a}]{3,2}",
        r"[\q{a}]{,2}",
        r"[\q{a}]{1,2,3}",
        r"[^\q{ab}]{0}",
        r"[\q{\d}]+",
        r"[\q{a}-b]+",
        r"[\q{a}&&a--b]+",
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
    for source in [
        r"/([\q{a}])+/v.test('a')",
        r"/(?:[\q{a}])+/v.test('a')",
        r"/[^\q{ab}&&a]{0}/v.test('')",
        r"/[\q{a}&&[]&&\q{b|cd}]{0}/v.test('')",
    ] {
        let mut realm = Realm::default();
        realm.eval("var marker=0").unwrap();
        assert_eq!(
            realm.eval(&format!(
                "marker=1;try{{{source};}}catch{{marker=2}}finally{{marker=3}}"
            )),
            Ok(Value::Boolean(true)),
            "{source}"
        );
        assert_eq!(realm.eval("marker"), Ok(Value::Number(3.0)), "{source}");
    }
    assert_true(&format!(
        r#"
            var marker = 0,
                ok = false;
            try {{
                ok = (function() {{
                    var pattern = "[\\q{{a|}}]*",
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
                    var pattern = "[\\q{{}}]{{0}}",
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
    assert_true(&format!(
        r#"
            var marker = 0,
                ok = false;
            try {{
                ok = (function() {{
                    var pattern = "[\\q{{a|bc}}]{{0}}",
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

    assert_true(
        r#"
            var pattern = "[[\\q{}]&&\\p{RGI_Emoji}]{0}",
                input = "a😀b",
                r = new RegExp(pattern, "dgvy");
            r.lastIndex = 2;
            var m = r.exec(input);
            r.source === pattern && r.flags === "dgvy" && m.input === input && Object.hasOwn(m,
                "groups") && Object.hasOwn(m.indices, "groups") && (m.groups === undefined || Object
                    .getPrototypeOf(m.groups) === null && Object.getPrototypeOf(m.indices.groups) === null
                    ) && JSON.stringify([Array.from(m, v => v === undefined ? "<undefined>" : v), Array
                    .from(m.indices, v => v === undefined ? "<undefined>" : v), m.index, r.lastIndex, m
                    .groups === undefined ? null : Object.keys(m.groups).map(k => [k, m.groups[k] ===
                        undefined ? "<undefined>" : m.groups[k]
                    ]), m.indices.groups === undefined ? null : Object.keys(m.indices.groups).map(
                        k => [k, m.indices.groups[k] === undefined ? "<undefined>" : m.indices.groups[
                            k]])
                ]) === "[[\"\"],[[1,1]],1,1,null,null]"
        "#,
    );
    assert_true(
        r#"
            var pattern = "[[\\q{a|bc}]&&\\p{RGI_Emoji}]{0}",
                input = "a😀b",
                r = new RegExp(pattern, "dgvy");
            r.lastIndex = 2;
            var m = r.exec(input);
            r.source === pattern && r.flags === "dgvy" && m.input === input && Object.hasOwn(m,
                "groups") && Object.hasOwn(m.indices, "groups") && (m.groups === undefined || Object
                    .getPrototypeOf(m.groups) === null && Object.getPrototypeOf(m.indices.groups) === null
                    ) && JSON.stringify([Array.from(m, v => v === undefined ? "<undefined>" : v), Array
                    .from(m.indices, v => v === undefined ? "<undefined>" : v), m.index, r.lastIndex, m
                    .groups === undefined ? null : Object.keys(m.groups).map(k => [k, m.groups[k] ===
                        undefined ? "<undefined>" : m.groups[k]
                    ]), m.indices.groups === undefined ? null : Object.keys(m.indices.groups).map(
                        k => [k, m.indices.groups[k] === undefined ? "<undefined>" : m.indices.groups[
                            k]])
                ]) === "[[\"\"],[[1,1]],1,1,null,null]"
        "#,
    );
    assert_true(
        r#"
            var pattern = "[\\q{a}--a--\\p{RGI_Emoji}]{0}",
                input = "a😀b",
                r = new RegExp(pattern, "dgvy");
            r.lastIndex = 2;
            var m = r.exec(input);
            r.source === pattern && r.flags === "dgvy" && m.input === input && Object.hasOwn(m,
                "groups") && Object.hasOwn(m.indices, "groups") && (m.groups === undefined || Object
                    .getPrototypeOf(m.groups) === null && Object.getPrototypeOf(m.indices.groups) === null
                    ) && JSON.stringify([Array.from(m, v => v === undefined ? "<undefined>" : v), Array
                    .from(m.indices, v => v === undefined ? "<undefined>" : v), m.index, r.lastIndex, m
                    .groups === undefined ? null : Object.keys(m.groups).map(k => [k, m.groups[k] ===
                        undefined ? "<undefined>" : m.groups[k]
                    ]), m.indices.groups === undefined ? null : Object.keys(m.indices.groups).map(
                        k => [k, m.indices.groups[k] === undefined ? "<undefined>" : m.indices.groups[
                            k]])
                ]) === "[[\"\"],[[1,1]],1,1,null,null]"
        "#,
    );
    for source in [
        r"/[[\q{a|}]&&\p{RGI_Emoji}]*/v.test('a')",
        r"/[\q{a}]+b/v.test('ab')",
        r"/a[\q{b}]+/v.test('ab')",
        r"/\b^[\q{a}]+/v.test('a')",
        r"/[\q{a}]+$\b/v.test('a')",
        r"/[\q{a}]+[b]+/v.test('ab')",
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
        .eval(r#"var kept=/(?<whole>([\q{a|b}&&[b]]+))/dvg;kept.lastIndex=1"#)
        .unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval(r#"kept.source==='(?<whole>([\\q{a|b}&&[b]]+))'&&kept.lastIndex===1&&kept.exec('zbbb!').groups.whole==='bbb'&&kept.lastIndex===4"#),Ok(Value::Boolean(true)));
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm.eval(r#"var r=/[\q{z}&&z]+/vg,marker=0"#).unwrap();
    let setup = format!("var text='{}';r.lastIndex=1", "a".repeat(20_000));
    realm.eval(&setup).unwrap();
    assert!(matches!(
        realm.eval(r#"try{r.exec(text)}catch{marker=1}finally{marker=2}"#),
        Err(Error::Limit { .. })
    ));
    assert_eq!(
        realm.eval(r#"marker===0&&r.lastIndex===1"#),
        Ok(Value::Boolean(true))
    );
}

#[test]
fn single_character_v_string_repetition_results() {
    let mut rows = String::new();
    let mut row_count = 0;
    // Complete JSON and original UTF16 spans were frozen before Runtime source.
    for (pattern, input, flags, start) in [
        (
            r#"(?<whole>([\q{a|b}]+))"#,
            JsString::from_code_units(vec![122, 97, 98, 98, 97, 33]),
            "dvg",
            0usize,
        ),
        (
            r#"(?<whole>([\q{a|b}&&[b-z]]{1,3}?))"#,
            JsString::from_code_units(vec![97, 98, 98, 98, 33]),
            "dvg",
            0usize,
        ),
        (
            r#"(?<whole>(\b[\q{a}--a]{0}\b))"#,
            JsString::from_code_units(vec![97]),
            "dvg",
            0usize,
        ),
        (
            r#"(?<whole>([\q{K}&&k]+))"#,
            JsString::from_code_units(vec![33, 8490, 75, 107, 33]),
            "divg",
            0usize,
        ),
        (
            r#"(?<whole>([^\q{K|s}]+?))"#,
            JsString::from_code_units(vec![8490, 383, 33]),
            "divg",
            0usize,
        ),
        (
            r#"(?<whole>(\b[\q{\uD83D\uDE00|x}&&[^\q{x}]]{1,2}\B))"#,
            JsString::from_code_units(vec![65, 55357, 56832, 55357, 56832, 33]),
            "dvy",
            2usize,
        ),
        (
            r#"(?<whole>([\q{\uD800|x}--x]{2}))"#,
            JsString::from_code_units(vec![65, 55296, 55296, 66]),
            "dvg",
            0usize,
        ),
        (
            r#"^(?<whole>(\B[[\q{\r|\n|\u2028}]--[\q{\r}]]{1,2}\B))$"#,
            JsString::from_code_units(vec![120, 13, 10, 8232, 121]),
            "dgmv",
            0usize,
        ),
        (
            r#"(?<whole>([\q{a|b}--a]{2,3}))"#,
            JsString::from_code_units(vec![97, 98, 98, 33]),
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
    assert_eq!(row_count, 9);
    insta::assert_snapshot!(rows);
}
