//! Flat Unicode classes with simple/common folding (22.2.2.7.3, 22.2.2.9).
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
fn unicode_class_folding_results_snapshot() {
    let mut rows = String::new();
    let inputs = [
        JsString::from("ſKaZ😀"),
        JsString::from("\u{390}\u{1fd3}\u{fb05}\u{fb06}"),
        JsString::from_code_units(vec![0x30, 0xa, 0xd800, 0xdc00]),
        JsString::from("\u{10400}\u{10428}ßẞ"),
    ];
    for body in [
        "[A-Z]",
        "[^A-Z]",
        r"[\w]",
        r"[\W]",
        r"[^\w]",
        r"[\u1FD3]",
        r"[\u0390]",
        r"[\u{10400}-\u{10427}]",
        r"[\uD800-\uDFFF]",
        r"[\s\D]",
    ] {
        for (left, right) in [("", ""), ("^", "$")] {
            let pattern = JsString::from(format!("{left}{body}{right}").as_str());
            for flags in ["diug", "diumy", "diumsg", "divg", "divmy", "divmsg"] {
                for input in &inputs {
                    for start in [0, 1, 2, input.len(), input.len() + 1] {
                        let script = format!(
                            "let r=new RegExp({pattern:?},'{flags}');r.lastIndex={start};let m=r.exec({input:?});JSON.stringify(m===null?{{match:null,lastIndex:r.lastIndex}}:{{matches:[...m],index:m.index,input:m.input,groups:m.groups,indices:m.indices,lastIndex:r.lastIndex,source:r.source}})"
                        );
                        let Value::String(result) = Realm::default().eval(&script).unwrap() else {
                            panic!("{script}")
                        };
                        writeln!(rows,"{pattern:?} flags={flags:?} input={input:?} lastIndex={start} {result:?}").unwrap();
                    }
                }
            }
        }
    }
    insta::assert_snapshot!(rows);
}

#[test]
fn unicode_class_folding_consumers_and_original_text() {
    for mode in ["u", "v"] {
        check(&format!(
            r"'ſK😀 9'.match(/[\w]/i{mode}g).join('|')==='ſ|K|9'"
        ));
        check(&format!(
            r"let a=[...'ſK😀 9'.matchAll(/[A-Z]/di{mode}g)];a.length===2&&a[0][0]==='ſ'&&a[1][0]==='K'&&a[1].indices[0][1]===2"
        ));
        check(&format!(
            r"'ſK😀 9'.split(/[\w]/i{mode}).join('|')==='||😀 |'"
        ));
        check(&format!(
            r"let seen=[];let s='ſK😀 9'.replace(/[\w]/i{mode}g,(m,i)=>{{seen.push(i);return '_'}});s==='__😀 _'&&seen.join(',')==='0,1,5'"
        ));
        check(&format!(
            r"let r=/[\W]/di{mode}y;r.lastIndex=3;let m=r.exec('ſK😀 9');m.index===2&&m[0]==='😀'&&m.indices[0][1]===4&&r.lastIndex===4"
        ));
        check(&format!(
            r"let r=/[^\W]/i{mode}y;r.lastIndex=3;r.exec('ſK😀 9')===null&&r.lastIndex===0"
        ));
        check(&format!(
            r"let r=/^[A-Z]$/i{mode}m;r.lastIndex=2;'😀\nſ'.search(r)===3&&r.lastIndex===2"
        ));
    }
}

#[test]
fn unicode_class_folding_ranges_inversion_and_word_complements() {
    check(r"/[A-Z]/iu.test('ſK')&&/[^A-Z]/iv.exec('ſK')===null");
    check(r"/[\W]/iu.exec('ſK')===null&&/[^\w]/iv.exec('ſK')===null");
    check(r"/[^\W]/iu.test('ſ')&&/[\w]/iv.test('K')");
    check(r"/[\u1fd3]/iu.test('\u0390')&&/[\u0390]/iv.test('\u1fd3')&&/[\ufb05]/iu.test('\ufb06')");
    check(
        r"/[\u{10400}-\u{10427}]/iu.test('\u{10428}')&&!/[^\u{10400}-\u{10427}]/iv.test('\u{10428}')",
    );
    check(r"/[\uD800-\uDFFF]/iu.exec('😀')===null&&/[\uD800-\uDFFF]/iv.test('\uD800')");
    check(r"/[ß]/iu.exec('ss')===null&&/[İ]/iv.exec('i')===null&&/[ı]/iu.exec('I')===null");
    check(r"/^[A-Z]$/ium.test('\rſ\n')&&/^[A-Z]$/iv.exec('ſ\n')===null");
}

#[test]
fn unicode_class_folding_metadata_clones_coercions_and_strict_writes() {
    check(
        r"let r=/^[A-Z]$/dimv,c=new RegExp(r,'diuy');c.source==='^[A-Z]$'&&c.ignoreCase&&c.unicode&&!c.unicodeSets&&!c.multiline&&c.exec('K')[0]==='K'&&c.lastIndex===1",
    );
    check(
        r"let t='',r=/[A-Z]/iuy;r.lastIndex={valueOf(){t+='i';return 0}};let m=r.exec({toString(){t+='s';return 'ſ'}});t==='si'&&m[0]==='ſ'&&r.lastIndex===1",
    );
    check(
        r"let r=/[A-Z]/iuy;Object.defineProperty(r,'lastIndex',{writable:false});let caught=false;try{r.exec('ſ')}catch(e){caught=e instanceof TypeError}caught&&r.lastIndex===0",
    );
    check(
        r"let r=/[A-Z]/ivy;r.lastIndex=1;Object.defineProperty(r,'lastIndex',{writable:false});let caught=false;try{r.exec('😀')}catch(e){caught=e instanceof TypeError}caught&&r.lastIndex===1",
    );
}

#[test]
fn unicode_class_folding_large_sources_collection_and_unlimited_defaults() {
    assert_eq!(Limits::default().max_steps, None);
    assert_eq!(Limits::default().max_heap_entries, None);
    let mut realm = Realm::default();
    realm.eval(r"let p='['+'\\w'.repeat(100000)+']',r=new RegExp(p,'ivg'),c=new RegExp(r),text='😀'.repeat(100000)+'K'").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(
        realm.eval("let m=c.exec(text);m[0]==='K'&&m.index===200000&&c.lastIndex===200001"),
        Ok(Value::Boolean(true))
    );
    realm.eval("p=null;r=null;c=null;text=null;m=null").unwrap();
    realm.collect(usize::MAX).unwrap();
}

#[test]
fn unicode_class_folding_opted_work_and_unproved_syntax() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval("let r=/[A-Z]/iug,marker=0,text='😀'.repeat(10000);r.lastIndex=1")
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
        r"/([a])+/iu.exec('a')",
        r"/[a]+/iv.exec('a')",
        r"/[a]b/iu.exec('ab')",
        r"/[a&&b]/iv.exec('a')",
        r"/[[a]]/iv.exec('a')",
        r"/[\p{ASCII}]/iv.exec('a')",
        r"/[\q{ab}]/iv.exec('ab')",
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
fn unicode_18_class_cross_plane_folding_keeps_original_widths() {
    let mut rows = String::new();
    for mode in ["u", "v"] {
        for pattern in ["[ß]", r"[\u{1df95}]", r"[^\u{10000}-\u{10ffff}]"] {
            for input in ["ß", "\u{1df95}"] {
                for start in [0, 1] {
                    let p = JsString::from(pattern);
                    let t = JsString::from(input);
                    let script = format!(
                        "let r=new RegExp({p:?},'di{mode}y');r.lastIndex={start};let m=r.exec({t:?});JSON.stringify(m===null?{{match:null,lastIndex:r.lastIndex}}:{{text:m[0],index:m.index,indices:m.indices[0],lastIndex:r.lastIndex}})"
                    );
                    let Value::String(result) = Realm::default().eval(&script).unwrap() else {
                        panic!("{script}")
                    };
                    writeln!(
                        rows,
                        "{p:?} mode={mode:?} input={t:?} start={start} {result:?}"
                    )
                    .unwrap();
                }
            }
        }
    }
    insta::assert_snapshot!(rows);
}

#[test]
fn original_unicode_class_folding_complete_programs() {
    let mut rows = String::new();
    for source in [
        r"/[a]/iu.exec('a')",
        r"/[\u{61}]/iv.exec('a')",
        r"/[^a]/iv.exec('b')",
        r"/^[a]$/iu.exec('a')",
        r"/[\W]/iv.exec('a')",
        r"/[a]/iv.exec('a')",
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
