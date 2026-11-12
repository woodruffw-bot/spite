//! Single Unicode literal atoms and simple/common folding (22.2.2.7.3).
use spite_core::JsString;
use spite_runtime::{Error, Limits, Realm, Value};
use std::fmt::Write;

use crate::common::assert_true;

#[test]
fn unicode_literal_character_folding_results_snapshot() {
    let mut rows = String::new();
    let inputs = [
        JsString::from("😀ſKaA$^.\n"),
        JsString::from("\u{10400}\u{10428}ßẞ"),
        JsString::from_code_units(vec![0xd800, 0xa, 0xdc00]),
        JsString::from("İıiI"),
    ];
    for body in [
        "a",
        "s",
        "k",
        "ſ",
        "K",
        r"\$",
        r"\^",
        r"\.",
        r"\u{1f600}",
        r"\uD800",
        r"\u{10400}",
        "ß",
    ] {
        for (left, right) in [("", ""), ("((", "))"), ("(?<x>^", "$)")] {
            let pattern = JsString::from(format!("{left}{body}{right}").as_str());
            for flags in ["diug", "diumy", "divg", "divmy"] {
                for input in &inputs {
                    for start in [0, 1, 2, input.len() + 1] {
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
                                    indexGroups: m.indices.groups,
                                    lastIndex: r.lastIndex,
                                    source: r.source
                                }})
                            "#
                        );
                        let Value::String(result) = Realm::default().eval(&script).unwrap() else {
                            panic!("{script}")
                        };
                        writeln!(rows,"{pattern:?} flags={flags:?} input={input:?} lastIndex={start} => {result}",
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
fn unicode_literal_character_folding_unicode_18_widths_snapshot() {
    let mut rows = String::new();
    for body in ["ß", r"\u{1df95}"] {
        for mode in ["u", "v"] {
            let pattern = JsString::from(body);
            for input in [JsString::from("ß"), JsString::from("\u{1df95}")] {
                for start in [0, 1] {
                    let script = format!(
                        r#"
                            let r = new RegExp({pattern:?}, 'di{mode}g');
                            r.lastIndex = {start};
                            let m = r.exec({input:?});
                            JSON.stringify(m === null ? {{
                                match: null,
                                lastIndex: r.lastIndex
                            }} : {{
                                text: m[0],
                                index: m.index,
                                indices: m.indices[0],
                                lastIndex: r.lastIndex
                            }}, null, 2)
                        "#
                    );
                    let Value::String(result) = Realm::default().eval(&script).unwrap() else {
                        panic!("{script}")
                    };
                    writeln!(
                        rows,
                        "{pattern:?} mode={mode:?} input={input:?} start={start}\n{result}\n",
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
fn unicode_literal_character_folding_escaped_anchors_and_fold_distinctions() {
    assert_true(r"/a/iu.test('A')&&/s/iv.test('ſ')&&/k/iu.test('K')&&/ſ/iv.test('S')");
    assert_true(r"/ß/iu.test('ẞ')&&!/ß/iv.test('ss')&&!/İ/iu.test('i')&&!/ı/iv.test('I')");
    assert_true(r"/\u1fd3/iu.test('\u0390')&&/\ufb05/iv.test('\ufb06')");
    assert_true(
        r"/^a$/iu.test('A')&&!/^a$/u.test('A')&&/^a$/ium.test('\rA\n')&&!/^a$/iu.test('A\n')",
    );
    assert_true(
        r"/\$/iu.exec('a$')[0]==='$'&&/^\$$/iv.test('$')&&/\^/iu.test('^')&&/\./iv.exec('😀.')[0]==='.'",
    );
    assert_true(r"/\\$/iu.test('\\')&&/\u0024/iv.test('$')");
    assert_true(r"/\uD800/iu.exec('😀')===null&&/\uD800/iv.test('\uD800')&&!/\uDE00/iu.test('😀')");
    assert_true(r"/^\u{1f600}$/u.test('😀')&&/^a$/vm.test('\na\n')");
}

#[test]
fn unicode_literal_character_folding_captures_consumers_and_original_text() {
    for mode in ["u", "v"] {
        assert_true(&format!(
            r#"
                let r = /(?<x>\u{{10400}})/di{mode}y;
                r.lastIndex = 1;
                let m = r.exec('\u{{10428}}');
                m[0] === '\u{{10428}}' && m[1] === '\u{{10428}}' && m.groups.x === m[1] && m.index === 0 && m
                    .indices[1][1] === 2 && m.indices.groups.x === m.indices[1] && r.lastIndex === 2
            "#
        ));
        assert_true(&format!(
            r"let a=[...'😀ſS'.matchAll(/(s)/di{mode}g)];a.length===2&&a[0][1]==='ſ'&&a[0].indices[1][0]===2&&a[1][1]==='S'"
        ));
        assert_true(&format!(
            r"'ſS'.replace(/(?<x>s)/i{mode}g,'<$1:$<x>>')==='<ſ:ſ><S:S>'"
        ));
        assert_true(&format!(
            r"'😀ſS'.split(/(s)/i{mode}).join('|')==='😀|ſ||S|'"
        ));
        assert_true(&format!(
            r"let r=/s/i{mode};r.lastIndex=8;'😀ſ'.search(r)===2&&r.lastIndex===8"
        ));
    }
}

#[test]
fn unicode_literal_character_folding_metadata_coercions_and_strict_writes() {
    assert_true(
        r#"
            let r = /^a$/dimv,
                c = new RegExp(r, 'diuy');
            c.source === '^a$' && c.ignoreCase && c.unicode && !c.multiline && c.exec('A')[0] === 'A' && c
                .lastIndex === 1
        "#,
    );
    assert_true(
        r#"
            let t = '',
                r = /s/iuy;
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
            let r = /s/iuy;
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
            let r = /s/ivy;
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
fn unicode_literal_character_folding_opted_work_and_additional_compositions() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval("let r=/s/iug,marker=0,text='😀'.repeat(10000);r.lastIndex=1")
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
        r#"let m=/(?=a)/iv.exec('A');m!==null&&m.length===1&&m[0]===''&&m.index===0&&m.input==='A'&&m.groups===undefined"#,
    );
    for source in [
        r"/(?:ab)+/iu.exec('AB')",
        r"/a+b/iv.exec('A')",
        r"/(?:(a)b)+/iu.exec('AB')",
        r"/a|b/iv.exec('A')",
        r"/^(a)+b$/iu.exec('A')",
        r"/\u{D800}\u{DC00}/iu.exec('😀')",
    ] {
        crate::common::regexp_oracle::assert_matches_reference(&mut Realm::default(), source);
    }
}

#[test]
fn unicode_literal_character_folding_script_execution() {
    let mut rows = String::new();
    for source in [
        r"/^a$/u.test('a')",
        r"/^a$/v.test('a')",
        r"/(^a)/u.test('a')",
        r"/a/iu.exec('a')",
        r"/^a/u.exec('a')",
        r"/\u{61}/iu.exec('a')",
        r"/^a$/u.exec('a')",
        r"/\u{1f600}/iu.exec('😀')",
        r"/a/iv.exec('a')",
        r"/^a/v.exec('a')",
        r"/\uD800/iu.exec('\uD800')",
    ] {
        let program = format!(
            r#"
            JSON.stringify(eval({:?}), null, 2)
        "#,
            JsString::from(source)
        );
        let Value::String(result) = Realm::default().eval(&program).unwrap() else {
            panic!("{program}")
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
        "/^a$/u.test('a')"
        true

        "/^a$/v.test('a')"
        true

        "/(^a)/u.test('a')"
        true

        "/a/iu.exec('a')"
        [
          "a"
        ]

        "/^a/u.exec('a')"
        [
          "a"
        ]

        "/\u005cu{61}/iu.exec('a')"
        [
          "a"
        ]

        "/^a$/u.exec('a')"
        [
          "a"
        ]

        "/\u005cu{1f600}/iu.exec('\ud83d\ude00')"
        [
          "😀"
        ]

        "/a/iv.exec('a')"
        [
          "a"
        ]

        "/^a/v.exec('a')"
        [
          "a"
        ]

        "/\u005cuD800/iu.exec('\u005cuD800')"
        [
          "\ud800"
        ]
        "#);
}
