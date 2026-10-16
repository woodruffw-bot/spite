//! Empty Unicode bodies normalize UTF-16 offsets to code-point boundaries.
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
fn unicode_empty_results_snapshot() {
    let mut rows = String::new();
    for pattern in ["", "()", "(())()", "(?<x>)(?<y>(?:))"] {
        for flags in ["du", "dug", "duy", "diuy", "dvg", "dviy"] {
            for input in ["", "😀", "a😀b", "\u{d7ff}"] {
                let p = JsString::from(pattern);
                let input = JsString::from(input);
                for start in 0..=input.len() + 1 {
                    let script = format!(
                        "let r=new RegExp({p:?},'{flags}');r.lastIndex={start};let m=r.exec({input:?});JSON.stringify(m===null?{{match:null,lastIndex:r.lastIndex}}:{{matches:[...m],index:m.index,indices:m.indices,groups:m.groups,indicesGroups:m.indices.groups,lastIndex:r.lastIndex,source:r.source}})"
                    );
                    let Value::String(result) = Realm::default().eval(&script).unwrap() else {
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
fn unicode_empty_consumers_advance_by_code_point() {
    assert!(matches!(
        Realm::default().eval("/(?:)/u.exec('a')"),
        Ok(Value::Object(_))
    ));
    check(
        "let a=[...'😀a'.matchAll(/(?<x>)/dug)];a.length===3&&a[0].index===0&&a[1].index===2&&a[2].index===3&&a[1].groups.x===''&&a[1].indices.groups.x===a[1].indices[1]",
    );
    check("JSON.stringify('😀a'.split(/(?:)/u))===JSON.stringify(['😀','a'])");
    check("'😀a'.replace(/(?:)/ug,'-')==='-😀-a-'");
    check("'😀a'.match(/(?:)/ug).length===3");
    check("let r=/(?:)/uy;r.lastIndex=1;let m=r.exec('😀');m.index===0&&r.lastIndex===0");
    check(
        r"let r=/(?:)/uy;r.lastIndex=2;let m=r.exec('\udc00\ud800');m.index===2&&r.lastIndex===2",
    );
}

#[test]
fn unicode_empty_ordered_coercion_and_state() {
    check(
        "let t='',r=/()/uy;r.lastIndex={valueOf(){t+='i';return 1;}};let m=r.exec({toString(){t+='s';return '😀';}});t==='si'&&m.index===0&&m[1]===''&&r.lastIndex===0",
    );
    check(
        "let t='',r=/()/u,i={valueOf(){t+='i';return 1;}};r.lastIndex=i;let m=r.exec({toString(){t+='s';return '😀';}});t==='si'&&m.index===0&&r.lastIndex===i",
    );
    check(
        "let r=/()/uy;r.lastIndex=1;Object.defineProperty(r,'lastIndex',{writable:false});let caught=false;try{r.exec('😀');}catch(e){caught=e instanceof TypeError;}caught&&r.lastIndex===1",
    );
    check("let r=/()/uy;r.lastIndex=3;let m=r.exec('😀');m===null&&r.lastIndex===0");
}

#[test]
fn unicode_empty_deep_captures_clones_and_collection() {
    assert_eq!(Limits::default().max_steps, None);
    assert_eq!(Limits::default().max_heap_entries, None);
    let mut realm = Realm::default();
    realm.eval("let r=new RegExp('('.repeat(100000)+')'.repeat(100000),'v'),c=new RegExp(r,'dvy');c.lastIndex=1;let a=c.exec('😀')").unwrap();
    assert_eq!(realm.eval("a.length===100001&&a[0]===''&&a[100000]===''&&a.index===0&&a.indices[100000][0]===0&&c.lastIndex===0"),Ok(Value::Boolean(true)));
    realm.collect(usize::MAX).unwrap();
    assert_eq!(
        realm.eval("c.lastIndex=100000;c.exec('b'.repeat(100000)).index===100000"),
        Ok(Value::Boolean(true))
    );
    realm.eval("r=null;c=null;a=null").unwrap();
    realm.collect(usize::MAX).unwrap();
}

#[test]
fn unicode_empty_explicit_work_abort_preserves_state() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval("let marker=0,r=/()/ug;r.lastIndex=1;let text='b'.repeat(20000)")
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
