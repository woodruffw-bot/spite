//! Braced Unicode escapes retain their literal values in proved BMP classes.
use crate::common::assert_true;
use spite_core::JsString;
use spite_runtime::{Error, ExceptionKind, Limits, Realm, Value};
use std::fmt::Write;

#[test]
fn unicode_braced_bmp_character_results_snapshot() {
    let mut rows = String::new();
    for (pattern, input) in [
        (r"[\u{61}-\u{63}]", "😀ba"),
        (r"[\u{e9}\u{3c3}]", "😀σ"),
        (r"[\u{8}]", "😀\u{8}"),
        (r"[\u{0}-\u{2}]", "😀\0a"),
        (r"[\u{D7FF}\u{E000}\u{FFFF}]", "\u{d7ff}\u{e000}\u{ffff}"),
        (r"[\u{61}\d]", "😀9"),
        (r"[\u{61}\s]", "😀\u{2028}a"),
        (r"[\u{000000000061}]", "😀a"),
        (r"[\u{5b}\u{5d}]", "😀]"),
        (r"[\u{26}\u{26}]", "😀&"),
        (r"[\u{2d}-a]", "😀_"),
    ] {
        for flags in ["dug", "duy", "dmusy", "dvg", "dvy", "dmvsy"] {
            for start in [0, 1, 2, 4] {
                let p = JsString::from(pattern);
                let input = JsString::from(input);
                let script = format!(
                    r#"
                        let r = new RegExp({p:?}, '{flags}');
                        r.lastIndex = {start};
                        let m = r.exec({input:?});
                        JSON.stringify(m === null ? {{
                            match: null,
                            lastIndex: r.lastIndex
                        }} : {{
                            matches: [...m],
                            index: m.index,
                            input: m.input,
                            groups: m.groups,
                            indices: m.indices,
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
                    "{p:?} flags={flags:?} input={input:?} lastIndex={start} => {result}",
                    result = result.to_utf8().expect("JSON output must be valid UTF-8"),
                )
                .unwrap();
            }
        }
    }
    insta::assert_snapshot!(rows);
}

#[test]
fn unicode_braced_bmp_character_consumers_and_literal_punctuation() {
    assert!(matches!(
        Realm::default().eval(r"new RegExp('[\\u{61}]').test('a')"),
        Err(Error::Exception {
            kind: ExceptionKind::SyntaxError,
            ..
        })
    ));
    assert_true(r"'😀a😀b'.match(/[\u{61}-\u{62}]/ug).join('|')==='a|b'");
    assert_true(
        r#"
            let a = [...'😀a😀b'.matchAll(/[\u{61}-\u{62}]/dvg)];
            a.length === 2 && a[0].index === 2 && a[1].index === 5 && a[0].indices[0][1] === 3
        "#,
    );
    assert_true(r"'😀a😀b'.split(/[\u{61}-\u{62}]/u).join('|')==='😀|😀|'");
    assert_true(
        r#"
            let seen = [];
            let s = '😀a😀b'.replace(/[\u{61}-\u{62}]/vg, (m, i) => {
                seen.push(i);
                return '_'
            });
            s === '😀_😀_' && seen.join(',') === '2,5'
        "#,
    );
    assert_true(
        r#"
            let ok = true;
            for (let n of [94, 36, 92, 46, 42, 43, 63, 40, 41, 91, 93, 123, 125, 124, 47, 38, 45]) {
                let c = String.fromCharCode(n),
                    p = '[\\u{' + n.toString(16) + '}]',
                    m = new RegExp(p, 'v').exec(c);
                if (m === null || m.length !== 1 || m[0] !== c) ok = false;
            }
            ok
        "#,
    );
    assert_true(r"/[\u{26}\u{26}]/v.exec('&')[0]==='&'&&/[\u{2d}-a]/v.exec('_')[0]==='_'");
}

#[test]
fn unicode_braced_bmp_character_boundaries_sources_flags_and_state() {
    assert_true(
        r#"
            let r = /[\u{61}-\u{62}]/dvg,
                c = new RegExp(r, 'uy');
            c.unicode && !c.unicodeSets && c.source === r.source && c.exec('a')[0] === 'a' && c
                .lastIndex === 1
        "#,
    );
    assert_true(r"let r=/[\u{61}]/duy;r.lastIndex=1;r.exec('😀a')===null&&r.lastIndex===0");
    assert_true(r"/[\u{61}]/v.exec('\uDC00a').index===1&&/[\u{61}]/u.exec('\uD800a').index===1");
    assert_true(r"let r=/[\u{61}]/vg;r.lastIndex=1;'😀a'.search(r)===2&&r.lastIndex===1");
    assert_true(
        r#"
            let r = /[\u{61}]/uy;
            r.lastIndex = 1;
            Object.defineProperty(r, 'lastIndex', {
                writable: false
            });
            let caught = false;
            try {
                r.exec('ba')
            } catch (e) {
                caught = e instanceof TypeError
            }
            caught && r.lastIndex === 1
        "#,
    );
    assert_true(
        r#"
            let r = /[\u{61}]/vy;
            r.lastIndex = 2;
            Object.defineProperty(r, 'lastIndex', {
                writable: false
            });
            let caught = false;
            try {
                r.exec('ba')
            } catch (e) {
                caught = e instanceof TypeError
            }
            caught && r.lastIndex === 2
        "#,
    );
}

#[test]
fn unicode_braced_bmp_character_ordered_and_abrupt_coercions() {
    assert_true(
        r#"
            let t = '',
                r = /[\u{61}]/vy;
            r.lastIndex = {
                valueOf() {
                    t += 'i';
                    return 2;
                }
            };
            let m = r.exec({
                toString() {
                    t += 's';
                    return '😀a';
                }
            });
            t === 'si' && m.index === 2 && r.lastIndex === 3
        "#,
    );
    assert_true(
        r#"
            let t = '',
                r = /[\u{61}]/u;
            r.lastIndex = {
                valueOf() {
                    t += 'i';
                    return 1;
                }
            };
            let m = r.exec({
                toString() {
                    t += 's';
                    return 'a';
                }
            });
            t === 'si' && m.index === 0 && typeof r.lastIndex === 'object'
        "#,
    );
    assert_true(
        r#"
            let r = /[\u{61}]/v,
                n = 0,
                e = {};
            r.lastIndex = {
                valueOf() {
                    n++;
                    return 0;
                }
            };
            let caught = false;
            try {
                r.exec({
                    toString() {
                        throw e;
                    }
                })
            } catch (x) {
                caught = x === e
            }
            caught && n === 0 && typeof r.lastIndex === 'object'
        "#,
    );
}

#[test]
fn unicode_braced_bmp_character_opted_work_and_unproved_sets() {
    let original = Realm::default()
        .eval(r"/[\u{61}]\u{62}/v.exec('ab')")
        .unwrap();
    assert!(matches!(original, Value::Object(_)));
    assert_true(
        r#"
            let r = /[\u{61}]\u{62}/v,
                m = r.exec('ab');
            Array.isArray(m) && m.length === 1 && m[0] === 'ab' && m.index === 0 && m.input === 'ab' && m
                .groups === undefined && m.indices === undefined && r.source === '[\\u{61}]\\u{62}' && r
                .lastIndex === 0 && r.exec('xb') === null && r.exec('ax') === null && r.exec('a') ===
                null && r.exec('xab').index === 1
        "#,
    );
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval(r"let r=/[\u{61}]/ug,marker=0,text='b'.repeat(20000);r.lastIndex=1")
        .unwrap();
    assert!(matches!(
        realm.eval("try{r.exec(text)}catch{marker=1}finally{marker=2}"),
        Err(Error::Limit { .. })
    ));
    assert_eq!(
        realm.eval("marker===0&&r.lastIndex===1"),
        Ok(Value::Boolean(true))
    );
    assert_true(r"/[\u{61}&&\u{62}]/v.exec('a')===null");
    assert_true(
        r"let m=/[[\u{61}]]/v.exec('a');m!==null&&m.length===1&&m[0]==='a'&&m.index===0&&m.input==='a'",
    );
    for source in [
        r"/([\u{10000}])+b/u.exec('a')",
        r"/([\u{D800}])+b/v.exec('\uD800')",
        r"/([\u{D7FF}-\u{E000}])+b/u.exec('a')",
        r"/([\u{61}])+b/iv.exec('a')",
        r"/([^\u{61}])+b/u.exec('b')",
        r"/[\u{61}]*b/u.exec('a')",
        r"/([\u{61}])+b/u.exec('a')",
    ] {
        crate::common::regexp_oracle::assert_matches_reference(&mut Realm::default(), source);
    }
}

#[test]
fn unicode_braced_bmp_character_script_execution() {
    let mut rows = String::new();
    for source in [r"/[\u{61}]/u.exec('a')", r"/[\u{61}]/v.exec('a')"] {
        let result = Realm::default().eval(source).unwrap();
        if source.contains(".exec(") {
            assert!(matches!(result, Value::Object(_)));
        } else {
            assert_eq!(result, Value::Boolean(true));
        }
        let script = format!(
            r#"
                JSON.stringify((() => {{
                    let a = {source};
                    return typeof a === 'boolean' ? a : {{
                        matches: [...a],
                        index: a.index,
                        input: a.input,
                        groups: a.groups
                    }}
                }})())
            "#
        );
        let Value::String(result) = Realm::default().eval(&script).unwrap() else {
            panic!("expected JSON")
        };
        writeln!(
            rows,
            "{source:?} => {result}",
            result = result.to_utf8().expect("JSON output must be valid UTF-8"),
        )
        .unwrap();
    }
    insta::assert_snapshot!(rows, @r#"
        "/[\\u{61}]/u.exec('a')" => {"matches":["a"],"index":0,"input":"a"}
        "/[\\u{61}]/v.exec('a')" => {"matches":["a"],"index":0,"input":"a"}
        "#);
}
