//! Complete mandatory captures around pure empty choices retain original indices.
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
fn unicode_enclosing_pure_empty_results_snapshot() {
    let mut rows = String::new();
    for pattern in [
        "((?:|))",
        "(?:((?:||)*))",
        "(((?:(?:)*|(?:)+?)))",
        "((?:|){0,3}?)",
        "(((?:)*(?:(?:|)|(?:){3,9})+))",
        "((?:(?:|){3}|(?:){0}){3,})",
        "(?<whole>(?<inner>(?:(?:|){3}|(?:){0}){0,0}))",
        "(?<__proto__>(?:|)+?)",
        "(?:(?:(?<x>(?:|){7,9})))",
        "((?:|)?)",
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
fn unicode_enclosing_pure_empty_names_indices_and_consumers() {
    check(
        "let m=/(?<whole>(?<__proto__>(?:|)*))/du.exec('😀');m.length===3&&m[1]===''&&m[2]===''&&m.groups.whole===''&&m.groups.__proto__===''&&m.indices.groups.whole===m.indices[1]&&m.indices.groups.__proto__===m.indices[2]&&Object.getPrototypeOf(m.groups)===null",
    );
    check(
        "let a=[...'😀a'.matchAll(/(?<x>(?:|)+?)/dvg)];a.length===3&&a[0].index===0&&a[1].index===2&&a[2].index===3&&a[1].groups.x===''&&a[1].indices.groups.x===a[1].indices[1]",
    );
    check("JSON.stringify('😀a'.split(/((?:|){3,})/u))===JSON.stringify(['😀','','a'])");
    check(
        "'😀a'.replace(/((?:||)*?)/ug,'<$1>')==='<>😀<>a<>'&&'😀a'.match(/((?:|){7,9})/vg).length===3",
    );
    check("'😀a'.search(/((?:|)?)/v)===0&&/((?:|)+)/u.test('')");
    check(
        "let r=/(?<x>(?:|)*)/dvy;r.lastIndex=1;let m=r.exec('😀');m.index===0&&m.indices[1][0]===0&&m.indices.groups.x===m.indices[1]&&r.lastIndex===0",
    );
}

#[test]
fn unicode_enclosing_pure_empty_coercion_and_strict_last_index() {
    check(
        "let log='',r=/((?:|){3,})/uy;r.lastIndex={valueOf(){log+='i';return 1}};let m=r.exec({toString(){log+='s';return '😀'}});log==='si'&&m.index===0&&m[1]===''&&r.lastIndex===0",
    );
    check(
        "let log='',r=/((?:|)+)/vi,i={valueOf(){log+='i';return 1}};r.lastIndex=i;let m=r.exec({toString(){log+='s';return '😀'}});log==='si'&&m[1]===''&&r.lastIndex===i",
    );
    check(
        "let r=/((?:|)*)/uy;r.lastIndex=1;Object.defineProperty(r,'lastIndex',{writable:false});let caught=false;try{r.test('😀')}catch(e){caught=e instanceof TypeError}caught&&r.lastIndex===1",
    );
    check(
        "let r=/((?:|)+)/v;r.lastIndex=99;Object.defineProperty(r,'lastIndex',{writable:false});r.test('😀')&&r.lastIndex===99",
    );
    check("let r=/((?:|)+)/uy;r.lastIndex=3;r.exec('😀')===null&&r.lastIndex===0");
}

#[test]
fn unicode_enclosing_pure_empty_deep_groups_counts_clones_and_collection() {
    assert_eq!(Limits::default().max_steps, None);
    assert_eq!(Limits::default().max_heap_entries, None);
    let mut realm = Realm::default();
    realm.eval("let p='('.repeat(100000)+'(?:|)*'+')'.repeat(100000),r=new RegExp(p,'v'),c=new RegExp(r,'dvy');c.lastIndex=1;let a=c.exec('😀')").unwrap();
    assert_eq!(realm.eval("a.length===100001&&a[100000]===''&&a.index===0&&a.indices[100000][0]===0&&a.indices[100000][1]===0&&c.source===p&&c.lastIndex===0"),Ok(Value::Boolean(true)));
    realm.collect(usize::MAX).unwrap();
    assert_eq!(
        realm.eval("c.lastIndex=200001;c.exec('b'.repeat(200001)).indices[100000][0]===200001"),
        Ok(Value::Boolean(true))
    );
    realm.eval("let digits='9'.repeat(10000),q='(?<x>(?:|){'+digits+','+digits+'}?)',h=new RegExp(q,'div');h.source===q").unwrap();
    assert_eq!(
        realm.eval("h.exec('😀').groups.x===''&&h.exec('').indices.groups.x[0]===0&&h.source===q"),
        Ok(Value::Boolean(true))
    );
    realm.eval("r=null;c=null;a=null;h=null").unwrap();
    realm.collect(usize::MAX).unwrap();
}

#[test]
fn unicode_enclosing_pure_empty_explicit_work_abort_preserves_state() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval("let marker=0,r=/((?:|)*)/ug;r.lastIndex=1;let text='b'.repeat(20000)")
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
fn unicode_enclosing_pure_empty_preserves_earlier_empty_capture_results() {
    check(
        "let m=/(())()/du.exec('a');m.length===4&&m[1]===''&&m[2]===''&&m[3]===''&&m.indices[3][0]===0&&m.indices[3][1]===0",
    );
    check(
        "let m=/(?<x>)(?<y>(?:))/dvi.exec('😀');m.groups.x===''&&m.groups.y===''&&m.indices.groups.x===m.indices[1]&&m.indices.groups.y===m.indices[2]",
    );
    check(
        "let p='(?<x>((?:|)*))',r=new RegExp(p,'iv'),c=new RegExp(r,'dvy'),m=c.exec('');c.source===p&&c.flags==='dvy'&&m.groups.x===''&&m[2]===''&&m.indices.groups.x===m.indices[1]",
    );
}
