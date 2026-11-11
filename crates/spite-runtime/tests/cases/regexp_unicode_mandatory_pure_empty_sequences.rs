//! Mandatory empty captures throughout concatenations retain original spans.
use crate::common::assert_true;
use spite_core::JsString;
use spite_runtime::{Error, Limits, Realm, Value};
use std::fmt::Write;

#[test]
fn unicode_mandatory_pure_empty_results_snapshot() {
    let mut rows = String::new();
    for pattern in [
        "((?:|))()",
        "(())(?:|)*()",
        "(?:(())(?:||)*())",
        "((?:|)())()",
        "(()(?:|))()(()(?:)*?())",
        "(?:|){3,9}()(?:|)*(()())",
        "(?<first>(?:|))(?<second>)(?:|){0,3}?",
        "(?<__proto__>(?:|)+?)()",
        "(?:(?<x>(?:|){7,9})())",
        "((?:|)?)()",
    ] {
        for flags in ["du", "dug", "duy", "diumy", "dvg", "disvmy"] {
            for input in [
                JsString::from(""),
                JsString::from("a😀b"),
                JsString::from("\r\n"),
                JsString::from_code_units(vec![0xd800, 0x61, 0xdc00, 0xd800, 0xdc00]),
            ] {
                for start in 0..=input.len() + 1 {
                    let p = JsString::from(pattern);
                    let script = format!(
                        "let r=new RegExp({p:?},'{flags}');r.lastIndex={start};let m=r.exec({input:?});JSON.stringify(m===null?{{match:null,lastIndex:r.lastIndex}}:{{matches:[...m],index:m.index,input:m.input,indices:m.indices,groups:m.groups,indicesGroups:m.indices.groups,lastIndex:r.lastIndex,source:r.source}})"
                    );
                    let Value::String(result) = Realm::default()
                        .eval(&script)
                        .unwrap_or_else(|e| panic!("{script}: {e:?}"))
                    else {
                        panic!("{script}")
                    };
                    writeln!(
                        rows,
                        "{p:?} flags={flags:?} input={input:?} start={start} {result:?}"
                    )
                    .unwrap();
                }
            }
        }
    }
    insta::assert_snapshot!(rows);
}

#[test]
fn unicode_mandatory_pure_empty_names_indices_and_consumers() {
    assert_true(
        r#"
            let m = /(?<first>(?:|)*)(?<__proto__>)/du.exec('😀');
            m.length === 3 && m[1] === '' && m[2] === '' && m.groups.first === '' && m.groups.__proto__ ===
                '' && m.indices.groups.first === m.indices[1] && m.indices.groups.__proto__ === m.indices[
                    2] && m.indices[1] !== m.indices[2] && Object.getPrototypeOf(m.groups) === null
        "#,
    );
    assert_true(
        r#"
            let a = [...'😀a'.matchAll(/(?<x>(?:|)+?)()/dvg)];
            a.length === 3 && a[0].index === 0 && a[1].index === 2 && a[2].index === 3 && a[1][2] === '' &&
                a[1].groups.x === '' && a[1].indices.groups.x === a[1].indices[1]
        "#,
    );
    assert_true("JSON.stringify('😀a'.split(/((?:|){3,})()/u))===JSON.stringify(['😀','','','a'])");
    assert_true(
        "'😀a'.replace(/((?:||)*?)()/ug,'<$1:$2>')==='<:>😀<:>a<:>'&&'😀a'.match(/((?:|){7,9})()/vg).length===3",
    );
    assert_true("'😀a'.search(/((?:|)?)()/v)===0&&/((?:|)+)()/u.test('')");
    assert_true(
        r#"
            let r = /(?<x>(?:|)*)()/dvy;
            r.lastIndex = 1;
            let m = r.exec('😀');
            m.index === 0 && m.indices[2][0] === 0 && m.indices.groups.x === m.indices[1] && r
                .lastIndex === 0
        "#,
    );
}

#[test]
fn unicode_mandatory_pure_empty_coercion_and_strict_last_index() {
    assert_true(
        r#"
            let log = '',
                r = /((?:|){3,})()/uy;
            r.lastIndex = {
                valueOf() {
                    log += 'i';
                    return 1
                }
            };
            let m = r.exec({
                toString() {
                    log += 's';
                    return '😀'
                }
            });
            log === 'si' && m.index === 0 && m[1] === '' && m[2] === '' && r.lastIndex === 0
        "#,
    );
    assert_true(
        r#"
            let log = '',
                r = /((?:|)+)()/vi,
                i = {
                    valueOf() {
                        log += 'i';
                        return 1
                    }
                };
            r.lastIndex = i;
            let m = r.exec({
                toString() {
                    log += 's';
                    return '😀'
                }
            });
            log === 'si' && m[2] === '' && r.lastIndex === i
        "#,
    );
    assert_true(
        r#"
            let r = /((?:|)*)()/uy;
            r.lastIndex = 1;
            Object.defineProperty(r, 'lastIndex', {
                writable: false
            });
            let caught = false;
            try {
                r.test('😀')
            } catch (e) {
                caught = e instanceof TypeError
            }
            caught && r.lastIndex === 1
        "#,
    );
    assert_true(
        r#"
            let r = /((?:|)+)()/v;
            r.lastIndex = 99;
            Object.defineProperty(r, 'lastIndex', {
                writable: false
            });
            r.test('😀') && r.lastIndex === 99
        "#,
    );
    assert_true("let r=/((?:|)+)()/uy;r.lastIndex=3;r.exec('😀')===null&&r.lastIndex===0");
}

#[test]
fn unicode_mandatory_pure_empty_explicit_work_abort_preserves_state() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval("let marker=0,r=/((?:|)*)()/ug;r.lastIndex=1;let text='b'.repeat(20000)")
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
