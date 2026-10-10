//! Case-sensitive u/v literal atoms preserve scalar and capture boundaries.
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
fn unicode_scalar_literal_results_snapshot() {
    let mut rows = String::new();
    for (pattern, input) in [
        ("😀", "😀😀"),
        (r"(\u{1f600})", "😀"),
        (r"(?<x>\uD83D\uDE00)(?<y>)", "a😀"),
        (r"()(\u{1f600}())\u{61}", "😀a😀a"),
        (r"\u{10000}", "\u{10000}"),
        (r"\u{10ffff}", "\u{10ffff}"),
        (r"\u{0}(\u{1f600})", "\0😀"),
        ("é😀", "é😀"),
        (r"\u{000001f600}", "😀"),
        (r"(😀)(😀)", "😀😀"),
        (r"\u{1f600}", "\u{1f601}"),
    ] {
        for flags in ["dug", "duy", "dvg", "dvy"] {
            for start in [0, 1, 2, 3, 6] {
                let p = JsString::from(pattern);
                let input = JsString::from(input);
                let script = format!(
                    "let r=new RegExp({p:?},'{flags}');r.lastIndex={start};let m=r.exec({input:?});JSON.stringify(m===null?{{match:null,lastIndex:r.lastIndex}}:{{matches:[...m],index:m.index,input:m.input,groups:m.groups,indices:m.indices,indicesGroups:m.indices.groups,lastIndex:r.lastIndex,source:r.source}})"
                );
                let Value::String(result) = Realm::default().eval(&script).unwrap() else {
                    panic!("{script}")
                };
                writeln!(
                    rows,
                    "{p:?} flags={flags:?} input={input:?} lastIndex={start} {result:?}"
                )
                .unwrap();
            }
        }
    }
    insta::assert_snapshot!(rows);
}

#[test]
fn unicode_scalar_literal_consumers_and_original_metadata() {
    check(
        r"let a=[...'😀a😀a'.matchAll(/(?<x>\u{1f600})(a)/dvg)];a.length===2&&a[0].index===0&&a[1].index===3&&a[0].groups.x==='😀'&&a[0].indices.groups.x===a[0].indices[1]",
    );
    check(
        r"let r=/(\u{1f600})/dug;r.lastIndex=1;let a=[...'😀😀'.matchAll(r)];a.length===2&&a[0].index===0&&a[1].index===2&&r.lastIndex===1",
    );
    check(r"'a😀b'.split(/(\uD83D\uDE00)()/v).join('|')==='a|😀||b'");
    check(
        r"let seen=[];let x='😀a😀'.replace(/(\u{1f600})/ug,(m,a,i)=>{seen.push(i);return '_'});x==='_a_'&&seen.join(',')==='0,3'",
    );
    check(r"'😀😀'.match(/\u{1f600}/vg).join('|')==='😀|😀'");
    check(r"let r=/\u{1f600}/vy;r.lastIndex=1;'😀'.search(r)===0&&r.lastIndex===1");
    check(
        r"let r=new RegExp('(?<x>\\u{1f600})','v'),c=new RegExp(r);c.source===r.source&&c.flags==='v'&&c.exec('😀').groups.x==='😀'",
    );
    check(
        r"let r=/\u{1f600}/u;r.lastIndex=1;let a=r.exec('😀');a.index===0&&a[0]==='😀'&&r.lastIndex===1",
    );
}

#[test]
fn unicode_scalar_literal_coercions_and_strict_last_index() {
    check(
        r"let t='',r=/(?<x>\u{1f600})/dvy;r.lastIndex={valueOf(){t+='i';return 1}};let m=r.exec({toString(){t+='s';return '😀'}});t==='si'&&m.index===0&&m[0]==='😀'&&m.groups.x==='😀'&&m.indices[0][0]===0&&m.indices[1][1]===2&&r.lastIndex===2",
    );
    check(
        r"let r=/\u{1f600}/uy;r.lastIndex=1;Object.defineProperty(r,'lastIndex',{writable:false});let caught=false;try{r.exec('😀')}catch(e){caught=e instanceof TypeError}caught&&r.lastIndex===1",
    );
    check(
        r"let r=/\u{1f600}/vy;r.lastIndex=4;Object.defineProperty(r,'lastIndex',{writable:false});let caught=false;try{r.exec('😀')}catch(e){caught=e instanceof TypeError}caught&&r.lastIndex===4",
    );
    check(
        r"let r=/\u{1f600}/vg;r.lastIndex=1;let mark={},caught=false;try{r.exec({toString(){throw mark}})}catch(e){caught=e===mark}caught&&r.lastIndex===1",
    );
    check(
        r"let r=/\u{1f600}/uy;r.lastIndex={valueOf(){throw 42}};let caught=false;try{r.exec('😀')}catch(e){caught=e===42}caught&&typeof r.lastIndex==='object'",
    );
}

#[test]
fn unicode_scalar_literal_lone_input_units_remain_distinct() {
    check(
        r"let a=['\uD83D','\uDE00','\uD83Da\uDE00','\uD83D\uDE01'];a.every(s=>/\uD83D\uDE00/u.exec(s)===null)",
    );
    check(
        r"let r=/😀/vy;r.lastIndex=1;let m=r.exec('\uD83D😀\uDE00');m.index===1&&m[0]==='😀'&&r.lastIndex===3",
    );
    check(
        r"let r=/😀/uy;r.lastIndex=2;let m=r.exec('\uD83D😀\uDE00');m.index===1&&r.lastIndex===3",
    );
    check(
        r"let r=/😀/vg;r.lastIndex=1;r.exec('😀x').index===0&&r.lastIndex===2&&r.exec('😀x')===null&&r.lastIndex===0",
    );
}

#[test]
fn unicode_scalar_literal_deep_storage_clones_and_collection() {
    assert_eq!(Limits::default().max_steps, None);
    assert_eq!(Limits::default().max_heap_entries, None);
    let mut realm = Realm::default();
    realm.eval(r"let p='('.repeat(100000)+'\\u{'+'0'.repeat(100000)+'1f600}'+')'.repeat(100000),r=new RegExp(p,'v'),c=new RegExp(r),a=c.exec('😀')").unwrap();
    assert_eq!(
        realm.eval("a.length===100001&&a[100000]==='😀'&&a.index===0"),
        Ok(Value::Boolean(true))
    );
    realm.collect(usize::MAX).unwrap();
    assert_eq!(
        realm.eval("c.exec('a'.repeat(100000)+'😀').index===100000"),
        Ok(Value::Boolean(true))
    );
    realm.eval("r=null;c=null;a=null;p=null").unwrap();
    realm.collect(usize::MAX).unwrap();
}

#[test]
fn unicode_scalar_literal_opted_work_abort_preserves_state_and_unproved_atoms() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval(r"let marker=0,r=/\u{1f600}/vg;r.lastIndex=1;let text='a'.repeat(20000)")
        .unwrap();
    assert!(matches!(
        realm.eval("try{r.exec(text)}catch{marker=1}finally{marker=2}"),
        Err(Error::Limit { .. })
    ));
    assert_eq!(
        realm.eval("marker===0&&r.lastIndex===1"),
        Ok(Value::Boolean(true))
    );
    for source in [
        r"/(\uD83D)(\uDE00)/u.exec('😀')",
        r"/\uD83D(?:)\uDE00/v.exec('😀')",
        r"/\u{D83D}\u{DE00}/u.exec('😀')",
        r"/\uD83D\u{DE00}/v.exec('😀')",
        r"/\u{1f600}/iu.exec('😀')",
        r"/([\u{1f600}])+/v.exec('😀')",
        r"/😀+/u.exec('😀')",
        r"/(😀)\1/u.exec('😀😀')",
        r"/\uD800()\uDC00/u.exec('\uD800')",
        r"/(?=😀)/v.exec('😀')",
    ] {
        assert!(
            matches!(
                Realm::default().eval(source),
                Err(Error::Unsupported { .. })
            ),
            "{source}"
        );
    }
}

#[test]
fn unicode_scalar_literal_original_complete_programs_and_ordered_calls() {
    let mut rows = String::new();
    for source in [
        r"/(\u{1f600})/u.test('a')",
        r"/\u{1f600}/u.exec('a')",
        r"/\u{1f600}/v.exec('a')",
        r"/(?<x>\u{1f600})/u.test('a')",
        r"/(?:\u{1f600})/u.test('a')",
        r"/(?:\u{1f600})/v.test('a')",
        r"/\u{10000}/u.exec('a')",
    ] {
        let result = Realm::default().eval(source).unwrap();
        if source.contains(".exec(") {
            assert_eq!(result, Value::Null);
        } else {
            assert_eq!(result, Value::Boolean(false));
        }
        let script = format!("JSON.stringify({source})");
        let Value::String(result) = Realm::default().eval(&script).unwrap() else {
            panic!("expected JSON")
        };
        writeln!(rows, "{source:?} {result:?}").unwrap();
    }
    for flag in ["u", "v"] {
        let setup = format!(
            r"let r=new RegExp('\\u{{1f600}}','{flag}'),t='';r.lastIndex={{valueOf(){{t+='i';return 0;}}}};"
        );
        let operation = "r.exec({toString(){t+='s';return 'a';}})";
        let mut realm = Realm::default();
        realm.eval(&setup).unwrap();
        assert_eq!(realm.eval(operation), Ok(Value::Null));
        assert_eq!(
            realm.eval("t==='si'&&typeof r.lastIndex==='object'"),
            Ok(Value::Boolean(true))
        );
        let script = format!(
            "{setup}let a={operation};JSON.stringify({{match:a,trace:t,indexKind:typeof r.lastIndex}})"
        );
        let Value::String(result) = Realm::default().eval(&script).unwrap() else {
            panic!("expected JSON")
        };
        writeln!(rows, "{script:?} {result:?}").unwrap();
    }
    insta::assert_snapshot!("original_scalar_unicode_programs", rows);
}
