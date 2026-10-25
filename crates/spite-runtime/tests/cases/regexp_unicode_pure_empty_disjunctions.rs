//! Capture-free pure-empty Unicode alternatives preserve original boundaries.
use spite_core::JsString;
use spite_runtime::{Error, Limits, Realm, Value};
use std::fmt::Write;
fn check(source: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::Boolean(true)),
        "{source}"
    );
}

#[test]
fn unicode_pure_empty_disjunction_results_snapshot() {
    let mut rows = String::new();
    for pattern in [
        "|",
        "||",
        "(?:)|",
        "|(?:|)*",
        "(?:|)+|(?:){3,}",
        "(?:){2,4}?|(?:||)*",
        "(?:|){0}|(?:|){7,9}",
        "(?:(?:|)*|(?:)+?)|",
        "(?:|)*|(?:)+?|(?:){3,}",
        "|(?:(?:|){0,3}?|(?:){7})",
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
fn unicode_pure_empty_disjunction_results_and_consumers() {
    check(
        "let m=/(?:|)*|/du.exec('😀');m.length===1&&m[0]===''&&m.index===0&&m.groups===undefined&&m.indices.length===1&&m.indices.groups===undefined",
    );
    check(
        "let a=[...'😀a'.matchAll(/|(?:|)+?/dvg)];a.length===3&&a[0].index===0&&a[1].index===2&&a[2].index===3&&a[1].length===1&&a[1].indices[0][0]===2",
    );
    check("JSON.stringify('😀a'.split(/(?:|)*|/u))===JSON.stringify(['😀','a'])");
    check("'😀a'.replace(/(?:|)+|(?:){3,}/ug,'-')==='-😀-a-'&&'😀a'.match(/|(?:)+?/vg).length===3");
    check("'😀a'.search(/(?:){2,4}?|/v)===0&&/||/u.test('')");
    check(
        "let r=/(?:|)*|/dvy;r.lastIndex=1;let m=r.exec('😀');m.index===0&&m.length===1&&m.indices[0][0]===0&&r.lastIndex===0",
    );
}

#[test]
fn unicode_pure_empty_disjunction_coercion_and_strict_last_index() {
    check(
        "let log='',r=/(?:){3,}|/uy;r.lastIndex={valueOf(){log+='i';return 1}};let m=r.exec({toString(){log+='s';return '😀'}});log==='si'&&m.index===0&&m.length===1&&r.lastIndex===0",
    );
    check(
        "let log='',r=/|(?:)+/vi,i={valueOf(){log+='i';return 1}};r.lastIndex=i;let m=r.exec({toString(){log+='s';return '😀'}});log==='si'&&m[0]===''&&r.lastIndex===i",
    );
    check(
        "let r=/(?:|)*|/uy;r.lastIndex=1;Object.defineProperty(r,'lastIndex',{writable:false});let caught=false;try{r.test('😀')}catch(e){caught=e instanceof TypeError}caught&&r.lastIndex===1",
    );
    check(
        "let r=/|(?:)+/v;r.lastIndex=99;Object.defineProperty(r,'lastIndex',{writable:false});r.test('😀')&&r.lastIndex===99",
    );
    check("let r=/(?:|)*|/uy;r.lastIndex=3;r.exec('😀')===null&&r.lastIndex===0");
}

#[test]
fn unicode_pure_empty_disjunction_explicit_work_abort_preserves_state() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval("let marker=0,r=/(?:|)*|/ug;r.lastIndex=1;let text='b'.repeat(20000)")
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
fn unicode_pure_empty_disjunction_preserves_earlier_empty_and_capture_results() {
    check(
        "let m=/(())()/du.exec('a');m.length===4&&m[1]===''&&m[2]===''&&m[3]===''&&m.indices[3][0]===0&&m.indices[3][1]===0",
    );
    check(
        "let m=/(?<x>(?:|))()/dvi.exec('😀');m.length===3&&m.groups.x===''&&m[2]===''&&m.indices.groups.x===m.indices[1]",
    );
    check(
        "let m=/(?<x>)+()/du.exec('');m.length===3&&m.groups.x===''&&m[2]===''&&m.indices.groups.x===m.indices[1]",
    );
}
