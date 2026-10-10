//! Complete-character Unicode class assertions (22.2.2.4, 22.2.2.9).
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
fn unicode_class_assertion_results_snapshot() {
    let mut rows = String::new();
    let inputs = [
        JsString::from("😀\na\r\nx\u{2028}😀\u{2029}"),
        JsString::from_code_units(vec![0xd800, 0xa, 0xdc00, 0xd83d, 0xde00]),
        JsString::from("\r\n\u{2028}\u{2029}"),
        JsString::from("$^😀"),
    ];
    for body in [
        "[^]",
        "[^a😀]",
        "[😀]",
        r"[\uD800-\uDFFF]",
        r"[\s]",
        r"[\$\^]",
    ] {
        for (left, right) in [("", ""), ("^", ""), ("", "$"), ("^", "$")] {
            let pattern = JsString::from(format!("{left}{body}{right}").as_str());
            for flags in ["dug", "dumy", "dumsg", "dvg", "dvmy", "dvmsg"] {
                for input in &inputs {
                    for start in [0, 1, 2, input.len(), input.len() + 1] {
                        let script = format!(
                            "let r=new RegExp({pattern:?},'{flags}');r.lastIndex={start};let m=r.exec({input:?});JSON.stringify(m===null?{{match:null,lastIndex:r.lastIndex}}:{{matches:[...m],index:m.index,input:m.input,groups:m.groups,indices:m.indices,lastIndex:r.lastIndex,source:r.source}})"
                        );
                        let Value::String(result) = Realm::default().eval(&script).unwrap() else {
                            panic!("{script}")
                        };
                        writeln!(rows, "{pattern:?} flags={flags:?} input={input:?} lastIndex={start} {result:?}").unwrap();
                    }
                }
            }
        }
    }
    insta::assert_snapshot!(rows);
}

#[test]
fn unicode_class_assertion_consumers_and_pair_boundaries() {
    for mode in ["u", "v"] {
        check(&format!(
            r"'😀\n\uD800\r\nx'.match(/^[^]$/{mode}mg).join('|')==='😀|\uD800|x'"
        ));
        check(&format!(
            r"let a=[...'😀\n\uD800\r\nx'.matchAll(/^[^]$/d{mode}mg)];a.length===3&&a[0].indices[0][1]===2&&a[1].index===3&&a[2].index===6"
        ));
        check(&format!(
            r"'😀\n\uD800\r\nx'.split(/^[^]$/{mode}m).join('|')==='|\n|\r\n|'"
        ));
        check(&format!(
            r"let seen=[];let s='😀\n\uD800\r\nx'.replace(/^[^]$/{mode}mg,(m,i)=>{{seen.push(i);return '_'}});s==='_\n_\r\n_'&&seen.join(',')==='0,3,6'"
        ));
        check(&format!(
            r"let r=/^[😀]$/d{mode}y;r.lastIndex=1;let m=r.exec('😀');m.index===0&&m[0]==='😀'&&m.indices[0][1]===2&&r.lastIndex===2"
        ));
        check(&format!(
            r"let r=/^[\uD800-\uDFFF]$/{mode}y;r.lastIndex=1;r.exec('😀')===null&&r.lastIndex===0"
        ));
        check(&format!(
            r"let r=/^[😀]$/{mode}my;r.lastIndex=3;let m=r.exec('x\n😀');m.index===2&&r.lastIndex===4"
        ));
        check(&format!(
            r"let r=/^[😀]$/{mode}m;r.lastIndex=2;'x\n😀'.search(r)===2&&r.lastIndex===2"
        ));
    }
}

#[test]
fn unicode_class_assertion_line_contexts_and_escaped_data() {
    check(r"/^[^]$/um.test('\r\n')&&/^[^]$/vm.test('\r\n')");
    check(r"/^[a]$/u.exec('a\n')===null&&/^[a]$/v.exec('a\r')===null");
    check(r"/^[a]$/um.test('a\n')&&/^[a]$/vm.test('a\r')");
    check(r"/^[a]$/um.test('\u2028a\u2029')&&/^[a]$/vm.test('\u2029a\u2028')");
    check(r"/^[😀]$/u.test('😀')&&!/^[😀]$/u.test('\uD83D')&&!/^[😀]$/v.test('\uDE00')");
    check(r"/^[\u{D800}\u{DC00}]$/v.test('\uD800')&&!/^[\u{D800}\u{DC00}]$/v.test('\uD800\uDC00')");
    check(r"/^[\$\^]$/u.test('$')&&/^[\$\^]$/v.test('^')");
    check(r"/^[\&\&\-\-]$/v.test('&')&&/^[\x2d\u005d\u{5b}]$/v.test(']')");
    check(
        r"/^[^❤️]$/u.exec('❤️')===null&&/^[^🧡]/u.exec('🧡')===null&&/[^💛]$/u.exec('💛')===null&&/[^💚]/u.exec('💚')===null",
    );
}

#[test]
fn unicode_class_assertion_metadata_coercions_clones_and_strict_writes() {
    check(
        r"let r=/^[^]$/dmv,c=new RegExp(r,'duy');c.source==='^[^]$'&&c.unicode&&!c.unicodeSets&&!c.multiline&&c.exec('😀')[0]==='😀'&&c.lastIndex===2",
    );
    check(
        r"let r=/^[^]$/dvmg,m=r.exec('😀');m.length===1&&m.groups===undefined&&m.indices.groups===undefined",
    );
    check(
        r"let t='',r=/^[^]$/uy;r.lastIndex={valueOf(){t+='i';return 1}};let m=r.exec({toString(){t+='s';return '😀'}});t==='si'&&m.index===0&&r.lastIndex===2",
    );
    check(
        r"let r=/^[^]$/uy;r.lastIndex=1;Object.defineProperty(r,'lastIndex',{writable:false});let caught=false;try{r.exec('😀')}catch(e){caught=e instanceof TypeError}caught&&r.lastIndex===1",
    );
    check(
        r"let r=/^[\uD800]$/vy;r.lastIndex=1;Object.defineProperty(r,'lastIndex',{writable:false});let caught=false;try{r.exec('😀')}catch(e){caught=e instanceof TypeError}caught&&r.lastIndex===1",
    );
}

#[test]
fn unicode_class_assertion_large_sources_collection_and_unlimited_defaults() {
    assert_eq!(Limits::default().max_steps, None);
    assert_eq!(Limits::default().max_heap_entries, None);
    let mut realm = Realm::default();
    realm.eval(r"let p='^['+'\\d'.repeat(100000)+'\\u{1f600}]$',r=new RegExp(p,'vmg'),c=new RegExp(r),text='x'.repeat(100000)+'\n😀'").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(
        realm.eval("let m=c.exec(text);m[0]==='😀'&&m.index===100001&&c.lastIndex===100003"),
        Ok(Value::Boolean(true))
    );
    realm.eval("p=null;r=null;c=null;text=null;m=null").unwrap();
    realm.collect(usize::MAX).unwrap();
}

#[test]
fn unicode_class_assertion_opted_work_and_unproved_syntax() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval("let r=/^[a]$/umg,marker=0,text='😀'.repeat(10000);r.lastIndex=1")
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
        r"/^[a]$/iu.exec('a')",
        r"/^([a])$/u.exec('a')",
        r"/^[a]+$/u.exec('a')",
        r"/^[a]b$/v.exec('ab')",
        r"/^[a&&b]$/v.exec('a')",
        r"/^[[a]]$/v.exec('a')",
        r"/^[\p{ASCII}]$/u.exec('a')",
        r"/^[\q{ab}]$/v.exec('ab')",
        r"/^[a]$|b/u.exec('b')",
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
fn original_unicode_class_assertion_complete_programs() {
    let mut rows = String::new();
    for source in [
        r"/^[a]$/u.test('a')",
        r"/^[a]$/v.test('a')",
        r"/^[a]$/v.exec('a')",
    ] {
        let Value::String(result) = Realm::default()
            .eval(&format!("JSON.stringify({source})"))
            .unwrap()
        else {
            panic!("{source}")
        };
        writeln!(rows, "{:?} {result:?}", JsString::from(source)).unwrap();
    }
    insta::assert_snapshot!(rows);
}
