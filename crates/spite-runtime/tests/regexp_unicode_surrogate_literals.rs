//! Unicode lone surrogate atoms match only complete input characters.
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
fn unicode_surrogate_literal_results_snapshot() {
    let mut rows = String::new();
    for (pattern, units) in [
        (r"\uD83D", vec![0xd83d, 0xde00, 0xd83d]),
        (r"\uDE00", vec![0xd83d, 0xde00, 0xde00]),
        (r"(?<x>\u{D800})(?<y>)", vec![0xd800]),
        (r"()(\u{DFFF}())", vec![0xdbff, 0xdfff, 0xdfff]),
        (r"(\uD83D)\uD83D\uDE00", vec![0xd83d, 0xd83d, 0xde00]),
        (r"\uD83D\uDE00(\uDE00)", vec![0xd83d, 0xde00, 0xde00]),
        (r"\uD83D\uD83D", vec![0xd83d, 0xd83d, 0xde00]),
        (r"(\uD83D)a(\uDE00)", vec![0xd83d, 0x61, 0xde00]),
    ] {
        for flags in ["dug", "duy", "dvg", "dvy"] {
            for start in [0, 1, 2, 3, 5] {
                let p = JsString::from(pattern);
                let input = JsString::from_code_units(units.clone());
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
fn unicode_surrogate_literal_consumers_preserve_complete_input_boundaries() {
    check(
        r"let ok=true;for(let f of ['u','v'])for(let n=0xD800;n<=0xDFFF;n++){let c=String.fromCharCode(n),r=new RegExp(c,f);if(r.source!==c||r.exec(c)[0]!==c)ok=false;if(n<0xDC00&&r.exec(c+'\uDC00')!==null)ok=false;if(n>=0xDC00&&r.exec('\uD800'+c)!==null)ok=false;}ok",
    );
    check(r"'😀\uDE00a'.split(/(\uDE00)/u).join('|')==='😀|\uDE00|a'");
    check(r"'😀\uDE00😀\uDE00'.match(/\uDE00/vg).length===2");
    check(
        r"let a=[...'😀\uDE00😀\uDE00'.matchAll(/(?<x>\uDE00)/dug)];a.length===2&&a[0].index===2&&a[1].index===5&&a[0].groups.x==='\uDE00'&&a[0].indices.groups.x===a[0].indices[1]",
    );
    check(
        r"let seen=[];let s='😀\uDE00😀\uDE00'.replace(/(\uDE00)/vg,(m,a,i)=>{seen.push(i);return '_'});s==='😀_😀_'&&seen.join(',')==='2,5'",
    );
    check(
        r"let r=/\uD83D/vg;r.lastIndex=1;let m=r.exec('😀\uD83D');m.index===2&&m[0]==='\uD83D'&&r.lastIndex===3",
    );
    check(r"let r=/\uDE00/uy;r.lastIndex=1;r.exec('😀')===null&&r.lastIndex===0");
    check(r"let r=/\uDE00/vg;r.lastIndex=1;'😀\uDE00'.search(r)===2&&r.lastIndex===1");
    check(
        r"let r=new RegExp('(?<x>\\u{D800})','u'),c=new RegExp(r);c.source===r.source&&c.flags==='u'&&c.exec('\uD800').groups.x==='\uD800'",
    );
}
#[test]
fn unicode_surrogate_literal_coercions_and_strict_writes() {
    check(
        r"let t='',r=/(?<x>\uDE00)/dvg;r.lastIndex={valueOf(){t+='i';return 1}};let m=r.exec({toString(){t+='s';return '😀\uDE00'}});t==='si'&&m.index===2&&m[0]==='\uDE00'&&m.indices.groups.x===m.indices[1]&&r.lastIndex===3",
    );
    check(
        r"let r=/\uDE00/uy;r.lastIndex=1;Object.defineProperty(r,'lastIndex',{writable:false});let caught=false;try{r.exec('a\uDE00')}catch(e){caught=e instanceof TypeError}caught&&r.lastIndex===1",
    );
    check(
        r"let r=/\uDE00/vy;r.lastIndex=1;Object.defineProperty(r,'lastIndex',{writable:false});let caught=false;try{r.exec('😀')}catch(e){caught=e instanceof TypeError}caught&&r.lastIndex===1",
    );
    check(
        r"let r=/\uD800/ug;r.lastIndex=1;let mark={},caught=false;try{r.exec({toString(){throw mark}})}catch(e){caught=e===mark}caught&&r.lastIndex===1",
    );
}
#[test]
fn unicode_surrogate_deep_storage_clones_collection_and_many_rejected_occurrences() {
    assert_eq!(Limits::default().max_steps, None);
    assert_eq!(Limits::default().max_heap_entries, None);
    let mut realm = Realm::default();
    realm.eval(r"let p='('.repeat(100000)+'\\u{'+'0'.repeat(100000)+'de00}'+')'.repeat(100000),r=new RegExp(p,'v'),c=new RegExp(r),a=c.exec('😀'.repeat(100000)+'\uDE00')").unwrap();
    assert_eq!(
        realm.eval(r"a.length===100001&&a[100000]==='\uDE00'&&a.index===200000"),
        Ok(Value::Boolean(true))
    );
    realm.collect(usize::MAX).unwrap();
    assert_eq!(
        realm.eval(r"c.exec('😀\uDE00').index===2"),
        Ok(Value::Boolean(true))
    );
    realm.eval("r=null;c=null;a=null;p=null").unwrap();
    realm.collect(usize::MAX).unwrap();
}
#[test]
fn unicode_surrogate_opted_work_aborts_and_unproved_atom_joins() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval(r"let marker=0,r=/\uDE00/vg;r.lastIndex=1;let text='😀'.repeat(10000)")
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
        r"/\uD800/iu.exec('\uD800')",
        r"/([\uD800])+/v.exec('\uD800')",
        r"/\uDE00+/u.exec('\uDE00')",
        r"/(\uDE00)\1/u.exec('\uDE00\uDE00')",
        r"/(?=\uD800)/v.exec('\uD800')",
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
fn unicode_surrogate_literal_original_complete_programs_and_ordered_calls() {
    let mut rows = String::new();
    for source in [
        r"/(\u{D800})/u.test('a')",
        r"/\u{D800}/u.exec('a')",
        r"/\u{D800}/v.exec('a')",
        r"/(?<x>\u{D800})/u.test('a')",
        r"/(?:\u{D800})/u.test('a')",
        r"/(?:\u{D800})/v.test('a')",
        r"/\uD800/u.exec('\uD800')",
        r"/\u{D800}/v.exec('\uD800')",
        r"/\uD800/v.exec('\uD800')",
    ] {
        let result = Realm::default().eval(source).unwrap();
        if source.contains("test(") {
            assert_eq!(result, Value::Boolean(false));
        } else if source.contains("exec('a')") {
            assert_eq!(result, Value::Null);
        } else {
            assert!(matches!(result, Value::Object(_)));
        }
        let script = format!("JSON.stringify({source})");
        let Value::String(result) = Realm::default().eval(&script).unwrap() else {
            panic!("expected JSON")
        };
        writeln!(rows, "{source:?} {result:?}").unwrap();
    }
    for flag in ["u", "v"] {
        let setup = format!(
            r"let r=new RegExp('\\u{{D800}}','{flag}'),t='';r.lastIndex={{valueOf(){{t+='i';return 0;}}}};"
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
    insta::assert_snapshot!("original_surrogate_unicode_programs", rows);
}
