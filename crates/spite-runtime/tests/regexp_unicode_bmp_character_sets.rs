//! Case-sensitive u-mode single atoms with nonsurrogate BMP membership.
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
fn unicode_bmp_character_results_snapshot() {
    let mut rows = String::new();
    for (pattern, input) in [
        ("[]", "😀a"),
        ("[a-b]", "😀ba"),
        ("[éσ]", "😀σ"),
        (r"[\b]", "😀\u{8}"),
        (r"[\cA\x61\u0062\0]", "😀\0a"),
        (r"[\uD7FF\uE000\uFFFF]", "\u{d7ff}\u{e000}\u{ffff}"),
        (r"[\d]", "😀9"),
        (r"[\w\s]", "😀\u{2028}_"),
        (r"\d", "😀7"),
        (r"\w", "😀Kk"),
        (r"\s", "😀\u{180e}\u{feff}"),
    ] {
        for flags in ["dug", "duy", "dmusy"] {
            for start in [0, 1, 2, 4] {
                let p = JsString::from(pattern);
                let input = JsString::from(input);
                let script = format!(
                    "let r=new RegExp({p:?},'{flags}');r.lastIndex={start};let m=r.exec({input:?});JSON.stringify(m===null?{{match:null,lastIndex:r.lastIndex}}:{{matches:[...m],index:m.index,input:m.input,groups:m.groups,indices:m.indices,lastIndex:r.lastIndex,source:r.source}})"
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
fn unicode_bmp_character_consumers_and_unicode_boundaries() {
    check(r"'😀a😀b'.match(/[a-b]/ug).join('|')==='a|b'");
    check(
        r"let a=[...'😀a😀b'.matchAll(/[a-b]/dug)];a.length===2&&a[0].index===2&&a[1].index===5&&a[0].indices[0][0]===2&&a[0].indices[0][1]===3",
    );
    check(r"'😀a😀b'.split(/[a-b]/u).join('|')==='😀|😀|'");
    check(
        r"let seen=[];let s='😀a😀b'.replace(/[a-b]/ug,(m,i)=>{seen.push(i);return '_'});s==='😀_😀_'&&seen.join(',')==='2,5'",
    );
    check(r"let r=/[a-b]/ug;r.lastIndex=1;'😀b'.search(r)===2&&r.lastIndex===1");
    check(r"let r=/[a-b]/uy;r.lastIndex=1;let a=r.exec('😀a');a===null&&r.lastIndex===0");
    check(r"/[a-b]/u.exec('\uDC00a').index===1&&/[a-b]/u.exec('\uD800a').index===1");
    check(r"let r=/[]/uy;r.lastIndex=1;r.exec('😀')===null&&r.lastIndex===0");
}

#[test]
fn unicode_bmp_character_sets_metadata_clones_and_state() {
    check(
        r"let r=/[a-b]/dug,c=new RegExp(r,'uy');c.unicode&&!c.unicodeSets&&c.source==='[a-b]'&&c.exec('a')[0]==='a'&&c.lastIndex===1",
    );
    check(
        r"let r=/[\w\s]/dums;let m=r.exec('😀\u2028');m.length===1&&m.index===2&&m.indices[0][1]===3&&m.groups===undefined&&m.indices.groups===undefined",
    );
    check(
        r"let r=/[a]/uy;r.lastIndex=1;Object.defineProperty(r,'lastIndex',{writable:false});let caught=false;try{r.exec('ba')}catch(e){caught=e instanceof TypeError}caught&&r.lastIndex===1",
    );
    check(
        r"let r=/[a]/uy;r.lastIndex=2;Object.defineProperty(r,'lastIndex',{writable:false});let caught=false;try{r.exec('ba')}catch(e){caught=e instanceof TypeError}caught&&r.lastIndex===2",
    );
}

#[test]
fn unicode_bmp_character_ordered_coercions_and_abrupt_input() {
    check(
        r"let t='',r=/[a]/uy;r.lastIndex={valueOf(){t+='i';return 2;}};let m=r.exec({toString(){t+='s';return '😀a';}});t==='si'&&m.index===2&&r.lastIndex===3",
    );
    check(
        r"let t='',r=/[a]/u;r.lastIndex={valueOf(){t+='i';return 1;}};let m=r.exec({toString(){t+='s';return 'a';}});t==='si'&&m.index===0&&typeof r.lastIndex==='object'",
    );
    check(
        r"let r=/\d/u,n=0,e={};r.lastIndex={valueOf(){n++;return 0;}};let caught=false;try{r.exec({toString(){throw e;}})}catch(x){caught=x===e}caught&&n===0&&typeof r.lastIndex==='object'",
    );
}

#[test]
fn unicode_bmp_character_large_sources_clones_collection_and_unlimited_defaults() {
    assert_eq!(Limits::default().max_steps, None);
    assert_eq!(Limits::default().max_heap_entries, None);
    let mut realm = Realm::default();
    realm
        .eval("let p='['+'a'.repeat(100000)+']',r=new RegExp(p,'u'),c=new RegExp(r),a=c.exec('a')")
        .unwrap();
    assert_eq!(
        realm.eval("a.length===1&&a[0]==='a'"),
        Ok(Value::Boolean(true))
    );
    realm.collect(usize::MAX).unwrap();
    assert_eq!(
        realm.eval("c.exec('b'.repeat(100000)+'a').index===100000"),
        Ok(Value::Boolean(true))
    );
    realm.eval("p=null;r=null;c=null;a=null").unwrap();
    realm.collect(usize::MAX).unwrap();
}

#[test]
fn unicode_bmp_character_opted_work_and_unproved_atoms() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval("let r=/[a]/ug,marker=0,text='b'.repeat(20000);r.lastIndex=1")
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
        r"/([\u{1f600}])/v.exec('a')",
        r"/[a]/iu.exec('a')",
        r"/([^a])/u.exec('b')",
        r"/(\D)/u.exec('a')",
        r"/([\uD800])/u.exec('\uD800')",
        r"/([\uD7FF-\uE000])/u.exec('a')",
        r"/([\u{1f600}])/u.exec('a')",
        r"/[a]b/u.exec('ab')",
        r"/([a])/u.exec('a')",
        r"/[a]*/u.exec('a')",
        r"/(.)/u.exec('a')",
        r"/[\p{ASCII}]/u.exec('a')",
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
fn unicode_bmp_character_former_complete_programs() {
    let mut rows = String::new();
    for source in [r"/[a]/u.exec('a')", r"/[a]/u.test('a')"] {
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
    insta::assert_snapshot!("original_bmp_character_programs", rows);
}
