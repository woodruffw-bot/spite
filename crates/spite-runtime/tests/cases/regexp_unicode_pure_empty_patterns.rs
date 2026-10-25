//! Unicode effect-free noncapturing choices use the existing empty-match engine.
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
fn unicode_pure_empty_results_snapshot() {
    let mut rows = String::new();
    for pattern in [
        "(?:|)",
        "(?:||(?:))*",
        "(?:(?:)*|(?:)+?)",
        "(?:|){0,3}?",
        "(?:)*(?:(?:|)|(?:){3,9})+",
        "(?:(?:|){3}|(?:){0}){3,}",
        "(?:(?:|){3}|(?:){0}){0,0}",
        "(?:|)+?",
        "(?:|){7,9}",
        "(?:|)?",
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
fn unicode_pure_empty_consumers_and_last_index() {
    check(
        "let a=[...'😀a'.matchAll(/(?:|)+?/dvg)];a.length===3&&a[0].index===0&&a[1].index===2&&a[2].index===3&&a[1].length===1&&a[1].groups===undefined&&a[1].indices.length===1",
    );
    check("JSON.stringify('😀a'.split(/(?:|){3,}/u))===JSON.stringify(['😀','a'])");
    check("'😀a'.replace(/(?:||)*?/ug,'-')==='-😀-a-'&&'😀a'.match(/(?:|){7,9}/vg).length===3");
    check("'😀a'.search(/(?:|)?/v)===0&&/(?:|)+/u.test('')");
    check(
        "let r=/(?:|)*/dvy;r.lastIndex=1;let m=r.exec('😀');m.index===0&&m.indices[0][0]===0&&m.length===1&&r.lastIndex===0",
    );
    check("let r=/(?:|)+/uy;r.lastIndex=3;r.exec('😀')===null&&r.lastIndex===0");
}

#[test]
fn unicode_pure_empty_coercion_order_and_strict_writes() {
    check(
        "let log='',r=/(?:|){3,}/uy;r.lastIndex={valueOf(){log+='i';return 1}};let m=r.exec({toString(){log+='s';return '😀'}});log==='si'&&m.index===0&&r.lastIndex===0",
    );
    check(
        "let log='',r=/(?:|)+/vi,i={valueOf(){log+='i';return 1}};r.lastIndex=i;let m=r.exec({toString(){log+='s';return '😀'}});log==='si'&&m.index===0&&r.lastIndex===i",
    );
    check(
        "let r=/(?:|)*/uy;r.lastIndex=1;Object.defineProperty(r,'lastIndex',{writable:false});let caught=false;try{r.test('😀')}catch(e){caught=e instanceof TypeError}caught&&r.lastIndex===1",
    );
    check(
        "let r=/(?:|)+/v;r.lastIndex=99;Object.defineProperty(r,'lastIndex',{writable:false});r.test('😀')&&r.lastIndex===99",
    );
    check(
        "let p='(?:(?:)*|(?:)+?){7,9}',r=new RegExp(p,'iv'),c=new RegExp(r,'dvy');c.source===p&&c.flags==='dvy'&&c.exec('😀').length===1",
    );
}

#[test]
fn unicode_pure_empty_explicit_work_abort_preserves_state() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval("let marker=0,r=/(?:|)*/ug;r.lastIndex=1;let text='b'.repeat(20000)")
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
fn unicode_pure_empty_modifier_and_skipped_capture_results_keep_native_metadata() {
    assert_eq!(Realm::default().eval(r####"var pattern="(?:|())*",input="A😀!",r=new RegExp(pattern,"dgu");r.lastIndex=2;var m=r.exec(input);var own=true;for(var i=0;i<m.length;i++)own=own&&Object.hasOwn(m,""+i)&&Object.hasOwn(m.indices,""+i);own&&r.source===pattern&&r.flags==="dgu"&&r.hasIndices&&r.unicode&&m.input===input&&m.groups===undefined&&m.indices.groups===undefined&&Object.hasOwn(m,"groups")&&Object.hasOwn(m.indices,"groups")&&JSON.stringify([Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex])==="[[\"\",\"<undefined>\"],[[1,1],\"<undefined>\"],1,1]""####),Ok(Value::Boolean(true)));
    assert_eq!(Realm::default().eval(r####"var pattern="(?i:|)*",input="A😀!",r=new RegExp(pattern,"dgu");r.lastIndex=2;var m=r.exec(input);var own=true;for(var i=0;i<m.length;i++)own=own&&Object.hasOwn(m,""+i)&&Object.hasOwn(m.indices,""+i);own&&r.source===pattern&&r.flags==="dgu"&&r.hasIndices&&r.unicode&&m.input===input&&m.groups===undefined&&m.indices.groups===undefined&&Object.hasOwn(m,"groups")&&Object.hasOwn(m.indices,"groups")&&JSON.stringify([Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex])==="[[\"\"],[[1,1]],1,1]""####),Ok(Value::Boolean(true)));
    check(
        "let m=/(?:|a){0}/dv.exec('a');m.length===1&&m[0]===''&&m.index===0&&m.indices[0][0]===0&&m.indices[0][1]===0",
    );
}
