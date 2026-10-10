//! Complete-character Unicode class unions, ranges and inversion (22.2.2.9).
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
fn unicode_class_results_snapshot() {
    let mut rows = String::new();
    let inputs = [
        JsString::from("😀a9\u{feff}"),
        JsString::from_code_units(vec![0xd800, 0x61, 0xdc00, 0xdbff, 0xdfff]),
        JsString::from("\t\n\u{180e}\u{feff}_K"),
        JsString::from("[-]&a"),
    ];
    for pattern in [
        "[^]",
        "[^a😀]",
        "[😀]",
        r"[\u{10000}-\u{10ffff}]",
        r"[\uD800-\uDFFF]",
        r"[^\uD800-\uDFFF]",
        r"[\uD800\uDC00]",
        r"[\u{D800}\u{DC00}]",
        r"[\uD800\u{DC00}]",
        r"[\d]",
        r"[\D]",
        r"[\w]",
        r"[\W]",
        r"[\s]",
        r"[\S]",
        r"[^\d\D]",
        r"[\x2d\u005d\u{5b}]",
        r"[\w\u{1f600}]",
    ] {
        for flags in ["dug", "duy", "dmusy", "dvg", "dvy", "dmvsy"] {
            for input in &inputs {
                for start in [0, 1, 2, input.len(), input.len() + 1] {
                    let p = JsString::from(pattern);
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
    }
    insta::assert_snapshot!(rows);
}

#[test]
fn unicode_class_consumers_and_complete_character_boundaries() {
    for mode in ["u", "v"] {
        check(&format!(
            r"'😀9\uD800'.match(/[\D]/{mode}g).join('|')==='😀|\uD800'"
        ));
        check(&format!(
            r"let a=[...'😀a\uDC00'.matchAll(/[^a]/d{mode}g)];a.length===2&&a[0].indices[0][1]===2&&a[1].index===3&&a[1][0]==='\uDC00'"
        ));
        check(&format!(
            r"'😀9\uD800'.split(/[\D]/{mode}).join('|')==='|9|'"
        ));
        check(&format!(
            r"let seen=[];let s='😀9\uD800'.replace(/[\D]/{mode}g,(m,i)=>{{seen.push(i);return '_'}});s==='_9_'&&seen.join(',')==='0,3'"
        ));
        check(&format!(
            r"let r=/[😀]/{mode}g;r.lastIndex=1;'a😀'.search(r)===1&&r.lastIndex===1"
        ));
        check(&format!(
            r"let r=/[😀]/d{mode}y;r.lastIndex=1;let m=r.exec('😀');m.index===0&&m[0]==='😀'&&m.indices[0][1]===2&&r.lastIndex===2"
        ));
        check(&format!(
            r"let r=/[\uD800-\uDFFF]/{mode}y;r.lastIndex=1;r.exec('😀')===null&&r.lastIndex===0"
        ));
        check(&format!(
            r"let r=/[\uD800-\uDFFF]/{mode}g;r.lastIndex=1;let m=r.exec('😀\uD800');m.index===2&&m[0]==='\uD800'&&r.lastIndex===3"
        ));
    }
}

#[test]
fn unicode_class_source_atoms_and_reserved_punctuation_remain_data() {
    check(r"/[\uD800\uDC00]/u.test('\uD800\uDC00')&&!/[\u{D800}\u{DC00}]/u.test('\uD800\uDC00')");
    check(r"/[\u{D800}\u{DC00}]/v.test('\uD800')&&/[\uD800\u{DC00}]/v.test('\uDC00')");
    check(r"/[\x2d\u005d\u{5b}]/u.test('-')&&/[\x2d\u005d\u{5b}]/v.test(']')");
    check(r"/[\&\&\-\-]/v.test('&')&&/[\&&]/v.test('&')&&/[a&&b]/u.test('&')");
    check(r"/[[]/u.test('[')&&/[-a]/u.test('-')");
    check(r"/[\s]/v.test('\uFEFF')&&!/[\s]/v.test('\u180E')&&/[\W]/u.test('K')");
    check(r"/[\d\D]/v.test('😀')&&/[^\d\D]/v.exec('😀')===null");
}

#[test]
fn unicode_class_metadata_clones_coercions_and_strict_writes() {
    check(
        r"let r=/[^a]/dsv,c=new RegExp(r,'duy');c.source==='[^a]'&&c.unicode&&!c.unicodeSets&&!c.dotAll&&c.exec('😀')[0]==='😀'&&c.lastIndex===2",
    );
    check(
        r"let r=/[\D]/dvg,m=r.exec('😀');m.length===1&&m.groups===undefined&&m.indices.groups===undefined",
    );
    check(
        r"let t='',r=/[^a]/uy;r.lastIndex={valueOf(){t+='i';return 1}};let m=r.exec({toString(){t+='s';return '😀'}});t==='si'&&m.index===0&&r.lastIndex===2",
    );
    check(
        r"let t='',r=/[^a]/v;r.lastIndex={valueOf(){t+='i';return 1}};let m=r.exec({toString(){t+='s';return '😀'}});t==='si'&&m.index===0&&typeof r.lastIndex==='object'",
    );
    check(
        r"let r=/[^a]/uy;r.lastIndex=1;Object.defineProperty(r,'lastIndex',{writable:false});let caught=false;try{r.exec('😀')}catch(e){caught=e instanceof TypeError}caught&&r.lastIndex===1",
    );
    check(
        r"let r=/[\uD800]/vy;r.lastIndex=1;Object.defineProperty(r,'lastIndex',{writable:false});let caught=false;try{r.exec('😀')}catch(e){caught=e instanceof TypeError}caught&&r.lastIndex===1",
    );
}

#[test]
fn unicode_class_large_sources_clones_collection_and_unlimited_defaults() {
    assert_eq!(Limits::default().max_steps, None);
    assert_eq!(Limits::default().max_heap_entries, None);
    let mut realm = Realm::default();
    realm.eval(r"let p='['+'\\d'.repeat(100000)+'\\u{1f600}]',r=new RegExp(p,'vg'),c=new RegExp(r),text='x'.repeat(100000)+'😀'").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(
        realm.eval("let m=c.exec(text);m[0]==='😀'&&m.index===100000&&c.lastIndex===100002"),
        Ok(Value::Boolean(true))
    );
    realm.eval("p=null;r=null;c=null;text=null;m=null").unwrap();
    realm.collect(usize::MAX).unwrap();
}

#[test]
fn unicode_class_opted_work_preserves_state_and_unproved_syntax() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval("let r=/[a]/ug,marker=0,text='😀'.repeat(10000);r.lastIndex=1")
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
        r"/([a])/iu.exec('a')",
        r"/([\W])/iv.exec('a')",
        r"/([a])/u.exec('a')",
        r"/^([a])$/v.exec('a')",
        r"/[a]+/u.exec('a')",
        r"/[a]b/v.exec('ab')",
        r"/[a&&b]/v.exec('a')",
        r"/[a--b]/v.exec('a')",
        r"/[[a]]/v.exec('a')",
        r"/[\p{ASCII}]/u.exec('a')",
        r"/[\q{ab}]/v.exec('ab')",
        r"/[a]|b/u.exec('b')",
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
fn original_unicode_class_complete_programs() {
    let mut rows = String::new();
    for source in [
        r"/[\u{1f600}]/u.test('a')",
        r"/[\u{1f600}]/v.exec('a')",
        r"/[^a]/u.exec('b')",
        r"/[\uD800]/u.exec('\uD800')",
        r"/[\uD7FF-\uE000]/u.exec('a')",
        r"/[\u{1f600}]/u.exec('a')",
        r"/[\u{10000}]/u.exec('a')",
        r"/[\u{D800}]/v.exec('\uD800')",
        r"/[\u{D7FF}-\u{E000}]/u.exec('a')",
        r"/[^\u{61}]/u.exec('b')",
        r"/[\D]/u.exec('a')",
        r"/[^a]/v.exec('b')",
        r"/[\u{1f600}]/v.exec('😀')",
        r"/[\uD800]/v.exec('\uD800')",
        r"/[\uD7FF-\uE000]/v.exec('a')",
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

#[test]
fn original_unicode_class_direct_eval_completion() {
    let mut realm = Realm::default();
    realm.eval("var marker=0;").unwrap();
    let code = r"marker=1; /[\u{1f600}]/v.test('a');";
    let operation = format!("try{{eval({code:?});}}catch{{marker=2;}}finally{{marker=3;}}");
    let result = realm.eval(&operation).unwrap();
    let marker = realm.eval("marker").unwrap();
    assert_eq!(result, Value::Boolean(false));
    assert_eq!(marker, Value::Number(3.0));
    insta::assert_snapshot!(format!(
        "{:?} result={result:?} marker={marker:?}",
        JsString::from(operation.as_str())
    ));
}
