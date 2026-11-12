//! Single case-sensitive Unicode dot and character escapes (22.2.2.7.1, 22.2.2.9).
use spite_core::JsString;
use spite_runtime::{Error, Limits, Realm, Value};
use std::fmt::Write;

use crate::common::assert_true;
use crate::common::regexp_matches::last_index_cases;

#[test]
fn unicode_dot_and_escape_results_snapshot() {
    let mut rows = String::new();
    let inputs = [
        JsString::from("😀a9_"),
        JsString::from("\n\r\u{2028}\u{2029}x"),
        JsString::from("\t \u{a0}\u{feff}x"),
        JsString::from("Kſéσ"),
        JsString::from_code_units(vec![0xd800, 0x61, 0xdc00, 0xd83d, 0xde00]),
    ];
    for pattern in [".", r"\d", r"\D", r"\s", r"\S", r"\w", r"\W"] {
        for flags in ["dug", "duy", "dusy", "dvg", "dvy", "dvsy"] {
            for input in &inputs {
                for start in last_index_cases(input.len()) {
                    let p = JsString::from(pattern);
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
    }
    insta::assert_snapshot!(rows);
}

#[test]
fn unicode_dot_escapes_consumers_and_complete_character_boundaries() {
    for mode in ["u", "v"] {
        assert_true(&format!(
            r"'😀a\n\uD800'.match(/./{mode}g).join('|')==='😀|a|\uD800'"
        ));
        assert_true(&format!(
            r#"
                let a = [...'😀a\n'.matchAll(/./d{mode}g)];
                a.length === 2 && a[0].index === 0 && a[0].indices[0][1] === 2 && a[1].index === 2
            "#
        ));
        assert_true(&format!(r"'😀a😀b'.split(/\D/{mode}).join('|')==='||||'"));
        assert_true(&format!(
            r#"
                let seen = [];
                let s = '😀9\uD800'.replace(/\D/{mode}g, (m, i) => {{
                    seen.push(i);
                    return '_'
                }});
                s === '_9_' && seen.join(',') === '0,3'
            "#
        ));
        assert_true(&format!(
            r"let r=/\W/{mode}g;r.lastIndex=1;'a😀'.search(r)===1&&r.lastIndex===1"
        ));
        assert_true(&format!(
            r#"
                let r = /./d{mode}y;
                r.lastIndex = 1;
                let m = r.exec('😀');
                m[0] === '😀' && m.index === 0 && m.indices[0][0] === 0 && m.indices[0][1] === 2 && r
                    .lastIndex === 2
            "#
        ));
        assert_true(&format!(
            r"let r=/\d/{mode}y;r.lastIndex=1;r.exec('😀9')===null&&r.lastIndex===0"
        ));
        assert_true(&format!(
            r"let r=/\D/{mode}g;r.lastIndex=1;let m=r.exec('😀9');m.index===0&&m[0]==='😀'&&r.lastIndex===2"
        ));
        assert_true(&format!(
            r#"
                let r = /./{mode}y;
                r.lastIndex = 1;
                let m = r.exec('\uDC00\uD800');
                m[0] === '\uD800' && m.index === 1 && r.lastIndex === 2
            "#
        ));
    }
}

#[test]
fn unicode_dot_escapes_exact_membership_and_original_metadata() {
    assert_true(r"/./u.exec('\n\r\u2028\u2029x').index===4&&/./su.exec('\n')[0]==='\n'");
    assert_true(r"/\s/v.exec('\u180E\uFEFF').index===1&&/\S/v.exec('\uFEFF😀')[0]==='😀'");
    assert_true(r"/\w/u.exec('Kſk').index===2&&/\W/v.exec('kK')[0]==='K'");
    assert_true(r"/\d/u.exec('９9').index===1&&/\D/v.exec('9９')[0]==='９'");
    assert_true(
        r#"
            let r = /./dsv,
                c = new RegExp(r, 'dug');
            c.source === '.' && c.unicode && !c.unicodeSets && !c.dotAll && c.exec('\nx').index === 1 && c
                .lastIndex === 2
        "#,
    );
    assert_true(
        r#"
            let r = /\D/dvg,
                m = r.exec('😀');
            m.length === 1 && m.groups === undefined && m.indices.groups === undefined && r.source ===
                '\\D'
        "#,
    );
    assert_true(r"/./.exec('😀')[0]==='\uD83D'&&/./u.exec('😀')[0]==='😀'");
}

#[test]
fn unicode_dot_escapes_ordered_coercions_and_strict_lastindex() {
    assert_true(
        r#"
            let t = '',
                r = /./uy;
            r.lastIndex = {
                valueOf() {
                    t += 'i';
                    return 1
                }
            };
            let m = r.exec({
                toString() {
                    t += 's';
                    return '😀'
                }
            });
            t === 'si' && m.index === 0 && r.lastIndex === 2
        "#,
    );
    assert_true(
        r#"
            let t = '',
                r = /\D/v;
            r.lastIndex = {
                valueOf() {
                    t += 'i';
                    return 1
                }
            };
            let m = r.exec({
                toString() {
                    t += 's';
                    return '😀'
                }
            });
            t === 'si' && m.index === 0 && typeof r.lastIndex === 'object'
        "#,
    );
    assert_true(
        r#"
            let r = /./uy,
                n = 0,
                e = {};
            r.lastIndex = {
                valueOf() {
                    n++;
                    return 0
                }
            };
            let caught = false;
            try {
                r.exec({
                    toString() {
                        throw e
                    }
                })
            } catch (x) {
                caught = x === e
            }
            caught && n === 0 && typeof r.lastIndex === 'object'
        "#,
    );
    assert_true(
        r#"
            let r = /./uy;
            r.lastIndex = 1;
            Object.defineProperty(r, 'lastIndex', {
                writable: false
            });
            let caught = false;
            try {
                r.exec('😀')
            } catch (e) {
                caught = e instanceof TypeError
            }
            caught && r.lastIndex === 1
        "#,
    );
    assert_true(
        r#"
            let r = /\d/vy;
            r.lastIndex = 1;
            Object.defineProperty(r, 'lastIndex', {
                writable: false
            });
            let caught = false;
            try {
                r.exec('😀')
            } catch (e) {
                caught = e instanceof TypeError
            }
            caught && r.lastIndex === 1
        "#,
    );
}

#[test]
fn unicode_dot_escapes_opted_work_preserves_state_and_pending_completion() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval("let r=/\\d/ug,marker=0,text='😀'.repeat(10000);r.lastIndex=1")
        .unwrap();
    assert!(matches!(
        realm.eval("try{r.exec(text)}catch{marker=1}finally{marker=2}"),
        Err(Error::Limit { .. })
    ));
    assert_eq!(
        realm.eval("marker===0&&r.lastIndex===1"),
        Ok(Value::Boolean(true))
    );
}

#[test]
fn unicode_dot_escapes_unproved_syntax_matches_reference_results() {
    assert_true(
        r#"
            let m = /[\q{ab}]/v.exec('ab');
            m.length === 1 && m[0] === 'ab' && m.index === 0 && m.input === 'ab' && m.groups ===
                undefined && /[\q{ab}]/v.exec('AB') === null
        "#,
    );
    assert_true(
        r#"
            let m = /\p{Script_Extensions=Latin}/u.exec('a');
            JSON.stringify(m) === '["a"]' && m.index === 0 && m.input === 'a'
        "#,
    );
    for source in [
        r"/(.)+b/iu.exec('a')",
        r"/(\W)+b/iv.exec('a')",
        r"/./v.exec('a')+/(.)+b/v.exec('a')",
        r"/\D+b/u.exec('a')",
        r"/a./v.exec('ab')",
        r"/^../u.exec('a')",
        r"/([\D])+b/u.exec('a')",
        r"/([^a])+b/v.exec('b')",
        r"/\d|a/u.exec('a')",
    ] {
        crate::common::regexp_oracle::assert_matches_reference(&mut Realm::default(), source);
    }
}

#[test]
fn unicode_dot_escape_script_execution() {
    let mut rows = String::new();
    for source in [
        r"/./v.test('a')",
        r"/\D/u.exec('a')",
        r"/./u.exec('a')",
        r"/\D/v.exec('a')",
        r"/./v.exec('a')",
    ] {
        let Value::String(result) = Realm::default()
            .eval(&format!(
                r#"
                JSON.stringify({source}, null, 2)
            "#
            ))
            .unwrap()
        else {
            panic!("{source}")
        };
        writeln!(
            rows,
            "{:?}\n{result}\n",
            JsString::from(source),
            result = result.to_utf8().expect("JSON output must be valid UTF-8"),
        )
        .unwrap();
    }
    insta::assert_snapshot!(rows, @r#"
        "/./v.test('a')"
        true

        "/\u005cD/u.exec('a')"
        [
          "a"
        ]

        "/./u.exec('a')"
        [
          "a"
        ]

        "/\u005cD/v.exec('a')"
        [
          "a"
        ]

        "/./v.exec('a')"
        [
          "a"
        ]
        "#);
}
