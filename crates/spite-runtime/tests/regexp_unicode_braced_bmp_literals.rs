//! Braced Unicode escapes decode only proved nonsurrogate BMP literal units.
use spite_core::JsString;
use spite_runtime::{Error, ExceptionKind, Limits, Realm, Value};
use std::fmt::Write;
fn check(source: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::Boolean(true)),
        "{source}"
    );
}

#[test]
fn unicode_braced_bmp_results_snapshot() {
    let mut rows = String::new();
    for (pattern, input) in [
        (r"\u{61}", "😀a"),
        (r"(\u{61}())\u{62}", "😀ab"),
        (r"(?<x>\u{61})(?<y>)\u{62}", "😀ab"),
        (r"\u{0000000000000061}", "a"),
        (r"\u{0}", "😀\0"),
        (r"\u{1}", "\u{1}"),
        (r"\u{D7FF}\u{E000}\u{FFFF}", "\u{d7ff}\u{e000}\u{ffff}"),
        (r"\u{00E9}", "é"),
        (r"\u{2028}", "\u{2028}"),
        (r"\u{212A}", "kK"),
    ] {
        for flags in ["dug", "duy", "dvg", "dvy"] {
            for start in [0, 1, 2, 4] {
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
fn unicode_braced_bmp_consumers_and_ordinary_escape_semantics() {
    assert!(matches!(
        Realm::default().eval(r"new RegExp('\\u{61}').test('u'.repeat(61))"),
        Err(Error::Exception {
            kind: ExceptionKind::SyntaxError,
            ..
        })
    ));
    check(
        r"let ok=true;for(let n of [94,36,92,46,42,43,63,40,41,91,93,123,125,124,47]){let c=String.fromCharCode(n),p='\\u{'+n.toString(16)+'}',m=new RegExp(p,'v').exec(c);if(m===null||m.length!==1||m[0]!==c)ok=false;}ok",
    );
    check(
        r"let a=[...'😀ab😀ab'.matchAll(/(?<x>\u{61}())\u{62}/dvg)];a.length===2&&a[0].index===2&&a[1].index===6&&a[0].groups.x==='a'&&a[0].indices.groups.x===a[0].indices[1]",
    );
    check(r"'😀ab'.split(/(\u{61}())\u{62}/u).join('|')==='😀|a||'");
    check(
        r"let seen=[];let x='😀ab😀ab'.replace(/(\u{61}())\u{62}/vg,(m,a,b,i)=>{seen.push(i);return '_'});x==='😀_😀_'&&seen.join(',')==='2,6'",
    );
    check(r"'😀aa'.match(/\u{61}/ug).join('|')==='a|a'");
    check(r"let r=/\u{61}/vg;r.lastIndex=1;'😀a'.search(r)===2&&r.lastIndex===1");
    check(r"new RegExp('\\u{61}','u').test('a')&&new RegExp('\\u{61}','v').test('a')");
}

#[test]
fn unicode_braced_bmp_coercions_strict_state_and_names() {
    check(
        r"let t='',r=/(?<x>\u{61})/dvy;r.lastIndex={valueOf(){t+='i';return 2;}};let m=r.exec({toString(){t+='s';return '😀a'}});t==='si'&&m.index===2&&m.groups.x==='a'&&m.indices.groups.x===m.indices[1]&&r.lastIndex===3",
    );
    check(
        r"let r=/\u{61}/u,t='';r.lastIndex={valueOf(){t+='i';return 1;}};let m=r.exec({toString(){t+='s';return 'a'}});t==='si'&&m.index===0&&typeof r.lastIndex==='object'",
    );
    check(
        r"let r=/\u{61}/uy;r.lastIndex=1;Object.defineProperty(r,'lastIndex',{writable:false});let caught=false;try{r.exec('ba')}catch(e){caught=e instanceof TypeError}caught&&r.lastIndex===1",
    );
    check(r"let r=/\u{61}/vy;r.lastIndex=1;r.exec('😀a')===null&&r.lastIndex===0");
    check(
        r"let r=/\u{61}/vy;r.lastIndex=2;Object.defineProperty(r,'lastIndex',{writable:false});let caught=false;try{r.exec('ba')}catch(e){caught=e instanceof TypeError}caught&&r.lastIndex===2",
    );
}

#[test]
fn unicode_braced_bmp_deep_zeros_scopes_clones_and_collection() {
    assert_eq!(Limits::default().max_steps, None);
    assert_eq!(Limits::default().max_heap_entries, None);
    let mut realm = Realm::default();
    realm.eval(r"let p='('.repeat(100000)+'\\u{'+'0'.repeat(100000)+'61}'+')'.repeat(100000),r=new RegExp(p,'v'),c=new RegExp(r);let a=c.exec('a')").unwrap();
    assert_eq!(
        realm.eval("a.length===100001&&a[100000]==='a'&&a.index===0"),
        Ok(Value::Boolean(true))
    );
    realm.collect(usize::MAX).unwrap();
    assert_eq!(
        realm.eval("c.exec('b'.repeat(100000)+'a').index===100000"),
        Ok(Value::Boolean(true))
    );
    realm.eval("r=null;c=null;a=null;p=null").unwrap();
    realm.collect(usize::MAX).unwrap();
}

#[test]
fn unicode_braced_bmp_opted_work_aborts_and_other_bodies_remain_distinct() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval(r"let marker=0,r=new RegExp('\\u{61}','vg');r.lastIndex=1;let text='b'.repeat(20000)")
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
        r"/\u{10000}/u.exec('a')",
        r"/\u{D800}/v.exec('\uD800')",
        r"/\u{61}/iu.exec('a')",
        r"/[\u{61}]/u.exec('a')",
        r"/\u{61}+/v.exec('aa')",
        r"/(\u{61})\1/u.exec('aa')",
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
fn unicode_braced_bmp_former_complete_programs_and_ordered_calls() {
    let mut rows = String::new();
    for source in [
        r"/(?:\u{61})/u.test('a')",
        r"/(?:\u{61})/v.test('a')",
        r"/(?<x>\u{61})/u.test('a')",
        r"/(\u{61})/u.test('a')",
        r"/\u{61}/u.exec('a')",
        r"/\u{61}/v.exec('a')",
    ] {
        let result = Realm::default().eval(source).unwrap();
        if source.contains(".exec(") {
            assert!(matches!(result, Value::Object(_)));
        } else {
            assert_eq!(result, Value::Boolean(true));
        }
        let script = format!(
            "JSON.stringify((()=>{{let a={source};return typeof a==='boolean'?a:{{matches:[...a],index:a.index,input:a.input,groups:a.groups}}}})())"
        );
        let Value::String(result) = Realm::default().eval(&script).unwrap() else {
            panic!("expected JSON")
        };
        writeln!(rows, "{source:?} {result:?}").unwrap();
    }
    for flag in ["u", "v"] {
        let setup = format!(
            r"let r=new RegExp('\\u{{61}}','{flag}'),t='';r.lastIndex={{valueOf(){{t+='i';return 0;}}}};"
        );
        let operation = "r.exec({toString(){t+='s';return 'a';}})";
        let mut realm = Realm::default();
        realm.eval(&setup).unwrap();
        assert!(matches!(realm.eval(operation), Ok(Value::Object(_))));
        assert_eq!(
            realm.eval("t==='si'&&typeof r.lastIndex==='object'"),
            Ok(Value::Boolean(true))
        );
        let script = format!(
            "{setup}let a={operation};JSON.stringify({{matches:[...a],index:a.index,input:a.input,trace:t,indexKind:typeof r.lastIndex}})"
        );
        let Value::String(result) = Realm::default().eval(&script).unwrap() else {
            panic!("expected JSON")
        };
        writeln!(rows, "{script:?} {result:?}").unwrap();
    }
    insta::assert_snapshot!("original_braced_unicode_programs", rows);
}
