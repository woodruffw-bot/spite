//! Unicode dot and character escapes with i (22.2.2.7.3, 22.2.2.9).
use crate::common::assert_true;
use crate::common::regexp_matches::last_index_cases;
use spite_core::JsString;
use spite_runtime::{Error, Limits, Realm, Value};
use std::fmt::Write;

#[test]
fn unicode_character_folding_results_snapshot() {
    let mut rows = String::new();
    let inputs = [
        JsString::from("ſK😀\n9 A\u{feff}"),
        JsString::from_code_units(vec![0xd800, 0xa, 0xdc00, 0xd83d, 0xde00]),
        JsString::from("ßẞİıﬀ\u{1df95}"),
        JsString::from("\r\nſ\u{2028}K\u{2029}"),
    ];
    for body in [".", r"\d", r"\D", r"\w", r"\W", r"\s", r"\S"] {
        for (left, right) in [("", ""), ("^", ""), ("", "$"), ("^", "$")] {
            let pattern = JsString::from(format!("{left}{body}{right}").as_str());
            for flags in ["diug", "diumy", "diumsg", "divg", "divmy", "divmsg"] {
                for input in &inputs {
                    for start in last_index_cases(input.len()) {
                        let script = format!(
                            r#"
                                let r = new RegExp({pattern:?}, '{flags}');
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
                        writeln!(rows, "{pattern:?} flags={flags:?} input={input:?} lastIndex={start} => {result}",
result = result.to_utf8().expect("JSON output must be valid UTF-8"),
).unwrap();
                    }
                }
            }
        }
    }
    insta::assert_snapshot!(rows);
}

#[test]
fn unicode_character_folding_consumers_and_original_text() {
    for mode in ["u", "v"] {
        assert_true(&format!(
            r"'ſK😀 9'.match(/\w/i{mode}g).join('|')==='ſ|K|9'"
        ));
        assert_true(&format!(
            r#"
                let a = [...'ſK😀 9'.matchAll(/\w/di{mode}g)];
                a.length === 3 && a[0][0] === 'ſ' && a[1][0] === 'K' && a[2].index === 5 && a[1].indices[0][
                    1
                ] === 2
            "#
        ));
        assert_true(&format!(
            r"'ſK😀 9'.split(/\w/i{mode}).join('|')==='||😀 |'"
        ));
        assert_true(&format!(
            r#"
                let seen = [];
                let s = 'ſK😀 9'.replace(/\w/i{mode}g, (m, i) => {{
                    seen.push(i);
                    return '_'
                }});
                s === '__😀 _' && seen.join(',') === '0,1,5'
            "#
        ));
        assert_true(&format!(
            r#"
                let r = /\W/di{mode}y;
                r.lastIndex = 3;
                let m = r.exec('ſK😀 9');
                m.index === 2 && m[0] === '😀' && m.indices[0][1] === 4 && r.lastIndex === 4
            "#
        ));
        assert_true(&format!(
            r"let r=/\w/i{mode}y;r.lastIndex=3;r.exec('ſK😀 9')===null&&r.lastIndex===0"
        ));
        assert_true(&format!(
            r"let r=/^\w$/i{mode}m;r.lastIndex=2;'😀\nſ'.search(r)===3&&r.lastIndex===2"
        ));
    }
}

#[test]
fn unicode_word_folding_is_simple_common_and_has_complete_boundaries() {
    assert_true(r"/\w/iu.test('ſ')&&/\w/iv.test('K')&&!/\w/u.test('ſ')&&!/\w/v.test('K')");
    assert_true(r"/\W/iu.exec('ſ')===null&&/\W/iv.exec('K')===null");
    assert_true(r"/\w/iu.exec('ßẞİıﬀ\u{1df95}')===null&&/\W/iv.test('ßẞİıﬀ\u{1df95}')");
    assert_true(r"/^\w$/ium.test('\rſ\n')&&/^\w$/ivm.test('\u2028K\u2029')");
    assert_true(r"/^\w$/iu.exec('ſ\n')===null&&/^\w$/iv.exec('K\r')===null");
    assert_true(r"/^.$/iu.test('😀')&&!/^.$/iv.test('\n')&&/^.$/ivs.test('\n')");
    assert_true(
        r"/^\d$/iu.test('9')&&/^\D$/iv.test('😀')&&/^\s$/iu.test('\uFEFF')&&/^\S$/iv.test('\u180E')",
    );
}

#[test]
fn unicode_character_folding_metadata_clones_coercions_and_strict_writes() {
    assert_true(
        r#"
            let r = /^\w$/dimv,
                c = new RegExp(r, 'diuy');
            c.source === '^\\w$' && c.ignoreCase && c.unicode && !c.unicodeSets && !c.multiline && c.exec(
                'K')[0] === 'K' && c.lastIndex === 1
        "#,
    );
    assert_true(
        r#"
            let t = '',
                r = /\w/iuy;
            r.lastIndex = {
                valueOf() {
                    t += 'i';
                    return 0
                }
            };
            let m = r.exec({
                toString() {
                    t += 's';
                    return 'ſ'
                }
            });
            t === 'si' && m[0] === 'ſ' && r.lastIndex === 1
        "#,
    );
    assert_true(
        r#"
            let r = /\w/iuy;
            Object.defineProperty(r, 'lastIndex', {
                writable: false
            });
            let caught = false;
            try {
                r.exec('ſ')
            } catch (e) {
                caught = e instanceof TypeError
            }
            caught && r.lastIndex === 0
        "#,
    );
    assert_true(
        r#"
            let r = /\w/ivy;
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
fn unicode_character_folding_opted_work_and_unproved_syntax() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval("let r=/\\w/iug,marker=0,text='😀'.repeat(10000);r.lastIndex=1")
        .unwrap();
    assert!(matches!(
        realm.eval("try{r.exec(text)}catch{marker=1}finally{marker=2}"),
        Err(Error::Limit { .. })
    ));
    assert_eq!(
        realm.eval("marker===0&&r.lastIndex===1"),
        Ok(Value::Boolean(true))
    );
    assert_true(
        r#"
            let m = /\p{Script_Extensions=Latin}/iv.exec('a');
            JSON.stringify(m) === '["a"]' && m.index === 0 && m.input === 'a'
        "#,
    );
    for source in [
        r"/([a])+b/iu.exec('a')",
        r"/([^a])+b/iv.exec('b')",
        r"/(\w)+b/iu.exec('a')",
        r"/\w+b/iv.exec('a')",
        r"/\w\d/iu.exec('a9')",
        r"/(?:\b)+/iu.exec('a')",
    ] {
        crate::common::regexp_oracle::assert_matches_reference(&mut Realm::default(), source);
    }
}

#[test]
fn unicode_character_folding_script_execution() {
    let mut rows = String::new();
    for source in [
        r"/^.$/iu.exec('a')",
        r"/./iu.exec('a')",
        r"/\W/iv.exec('a')",
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
        "/^.$/iu.exec('a')"
        [
          "a"
        ]

        "/./iu.exec('a')"
        [
          "a"
        ]

        "/\u005cW/iv.exec('a')"
        null
        "#);
}
