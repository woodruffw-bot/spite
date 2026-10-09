//! Nonempty case-sensitive BMP literals preserve UnicodeSets matching boundaries.

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
fn unicode_sets_bmp_literal_results_snapshot() {
    let mut rows = String::new();
    for (pattern, flags, input, initial) in [
        ("a", "dv", "ba", 9),
        ("a", "dvg", "😀ab", 0),
        ("a", "dvg", "😀ab", 1),
        ("a", "dvy", "😀ab", 1),
        ("a", "dvy", "😀ab", 2),
        ("a", "dvg", "a😀b", 1),
        ("ab", "dvg", "😀ab😀ab", 1),
        ("ab", "dvg", "😀ab😀ab", 4),
        ("ab", "dvg", "😀ab😀ab", 5),
        ("ab", "dvy", "😀ab😀ab", 5),
        ("ab", "dvy", "😀ab😀ab", 6),
        ("(a())b", "dvg", "😀ab", 1),
        ("()(ab)()", "dvy", "😀ab", 2),
        ("(?<x>a(?<y>()))b", "dvg", "😀ab", 1),
        ("(?<π>a)b", "dv", "😀ab", 0),
        ("((a)(?:b))(c)", "dvms", "😀abc", 0),
        (r"\u0061\x62", "dvg", "😀ab", 1),
        (r"\(a\)", "dv", "😀(a)", 0),
        (r"\0", "dvg", "😀\0x", 1),
        (r"\cA", "dvg", "😀\u{1}x", 1),
        (r"\x7f", "dvg", "😀\u{7f}x", 1),
        (r"\n", "dvg", "😀\nx", 1),
        ("a", "dvg", "a", 2),
        ("a", "dvy", "b", 0),
        ("é", "dvg", "😀é", 1),
        ("(σ())", "dvg", "😀σΣ", 1),
        ("K", "dv", "kK", 0),
        (r"\u00e9", "dvy", "😀é", 2),
        (r"\u2028", "dv", "😀\u{2028}", 0),
        ("(?:(µ))", "dv", "Μµ", 0),
    ] {
        let pattern = JsString::from(pattern);
        let input = JsString::from(input);
        let script = format!(
            "let r=new RegExp({pattern:?},'{flags}');r.lastIndex={initial};let m=r.exec({input:?});JSON.stringify(m===null?{{match:null,lastIndex:r.lastIndex}}:{{matches:[...m],index:m.index,input:m.input,groups:m.groups,indices:m.indices,indicesGroups:m.indices.groups,lastIndex:r.lastIndex,source:r.source}})"
        );
        let Value::String(result) = Realm::default().eval(&script).unwrap() else {
            panic!("expected JSON: {script}");
        };
        writeln!(
            rows,
            "{pattern:?} flags={flags:?} input={input:?} lastIndex={initial} {result:?}"
        )
        .unwrap();
    }
    insta::assert_snapshot!(rows);
}

#[test]
fn unicode_sets_bmp_names_lone_surrogates_and_matching_boundaries_are_exact() {
    check(
        "let r=/a/v,c=new RegExp(r),u=new RegExp(r,'u');r.unicodeSets&&!r.unicode&&r.flags==='v'&&c.unicodeSets&&!c.unicode&&u.unicode&&!u.unicodeSets&&u.exec('a')[0]==='a'",
    );
    let mut rows = String::new();
    for source in [
        "/(a)/v.test('a')",
        "/a/v.exec('a')",
        "/(?<x>a)/v.test('a')",
        "/(?:a)/v.test('a')",
    ] {
        let result = Realm::default().eval(source).unwrap();
        if source.contains(".exec(") {
            assert!(matches!(result, Value::Object(_)), "{source}");
        } else {
            assert_eq!(result, Value::Boolean(true), "{source}");
        }
        let script = format!(
            "JSON.stringify((()=>{{let a={source};return typeof a==='boolean'?a:{{matches:[...a],index:a.index,input:a.input,groups:a.groups}}}})())"
        );
        let Value::String(result) = Realm::default().eval(&script).unwrap() else {
            panic!("expected JSON");
        };
        writeln!(rows, "{source:?} {result:?}").unwrap();
    }
    let setup = "let r=new RegExp('a','v'),t='';r.lastIndex={valueOf(){t+='i';return 0;}};";
    let operation = "r.exec({toString(){t+='s';return 'a';}})";
    let mut realm = Realm::default();
    realm.eval(setup).unwrap();
    assert!(matches!(realm.eval(operation), Ok(Value::Object(_))));
    assert_eq!(
        realm.eval("t==='si'&&typeof r.lastIndex==='object'"),
        Ok(Value::Boolean(true))
    );
    let script = format!(
        "{setup}let a={operation};JSON.stringify({{matches:[...a],index:a.index,input:a.input,trace:t,indexKind:typeof r.lastIndex}})"
    );
    let Value::String(result) = Realm::default().eval(&script).unwrap() else {
        panic!("expected ordered JSON");
    };
    writeln!(rows, "{script:?} {result:?}").unwrap();
    insta::assert_snapshot!("original_unicode_sets_literal_programs", rows);
    check(r"let r=/a/vg;r.lastIndex=1;let a=r.exec('\uD800a\uDC00');a.index===1&&r.lastIndex===2");
    check(r"let r=/a/vy;r.lastIndex=1;let a=r.exec('\uD800a\uDC00');a.index===1&&r.lastIndex===2");
    check(
        r"let r=/(?<x>a(?<y>()))b/dvg;r.lastIndex=1;let a=r.exec('\uD83D\uDE00ab');a.index===2&&a.groups.x==='a'&&a.groups.y===''&&a.indices.groups.x===a.indices[1]&&a.indices.groups.y===a.indices[2]&&a.indices[2][0]===3",
    );
    check(r"let r=/a/vy;r.lastIndex=1;r.exec('\uD83D\uDE00a')===null&&r.lastIndex===0");
    check(r"let r=/a/vg;r.lastIndex=1;let a=r.exec('\uD83D\uDE00a');a.index===2&&r.lastIndex===3");
    check(r"let r=/a/vy;r.lastIndex=2;let a=r.exec('\uD83D\uDE00a');a.index===2&&r.lastIndex===3");
    check(r"let r=/a/v;r.exec('A')===null&&r.unicode===false&&r.unicodeSets===true");
}

#[test]
fn unicode_sets_bmp_consumers_callbacks_species_and_copies_keep_code_unit_indices() {
    check(
        r"let r=/(a())b/vg;r.lastIndex=1;let a=[...'\uD83D\uDE00ab\uD83D\uDE00ab'.matchAll(r)];a.length===2&&a[0].index===2&&a[1].index===6&&a[0][1]==='a'&&a[0][2]===''&&r.lastIndex===1",
    );
    check(
        r"let seen=[];let s='\uD83D\uDE00ab\uD83D\uDE00ab'.replace(/(a())b/vg,(m,x,y,i)=>{seen.push(i);return '_'});s==='\uD83D\uDE00_\uD83D\uDE00_'&&seen.join(',')==='2,6'",
    );
    check(r"'\uD83D\uDE00ab\uD83D\uDE00ab'.match(/ab/vg).join('|')==='ab|ab'");
    check(r"'\uD83D\uDE00ab'.split(/(a())b/v).join('|')==='\uD83D\uDE00|a||'");
    check(r"let r=/ab/vg;r.lastIndex=1;'\uD83D\uDE00ab'.search(r)===2&&r.lastIndex===1");
    check(
        r"let r=/(?<x>ab)/dvg,c=new RegExp(r);let a=c.exec('\uD83D\uDE00ab');a.groups.x==='ab'&&a.indices.groups.x===a.indices[1]&&r.lastIndex===0&&c.lastIndex===4",
    );
}

#[test]
fn unicode_sets_bmp_input_index_coercion_and_strict_writes_remain_ordered() {
    check(
        "let r=/a/v,t='';r.lastIndex={valueOf(){t+='i';return 2;}};let a=r.exec({toString(){t+='s';return 'ba'}});t==='si'&&a.index===1&&typeof r.lastIndex==='object'",
    );
    check(
        "let r=/a/vg,t='';r.lastIndex={valueOf(){t+='i';return 1;}};let a=r.exec({toString(){t+='s';return 'ba'}});t==='si'&&a.index===1&&r.lastIndex===2",
    );
    check(
        "let r=/a/vy;r.lastIndex=1;Object.defineProperty(r,'lastIndex',{writable:false});let caught=false;try{r.exec('ba')}catch(e){caught=e instanceof TypeError}caught&&r.lastIndex===1",
    );
    check(
        "let r=/a/vy;r.lastIndex=1;Object.defineProperty(r,'lastIndex',{writable:false});let caught=false;try{r.exec('bb')}catch(e){caught=e instanceof TypeError}caught&&r.lastIndex===1",
    );
}

#[test]
fn unicode_sets_bmp_deep_capture_scopes_clones_collection_and_long_search_stay_unlimited() {
    assert_eq!(Limits::default().max_steps, None);
    assert_eq!(Limits::default().max_heap_entries, None);
    let mut realm = Realm::default();
    realm.eval("let r=new RegExp('('.repeat(100000)+'a'+')'.repeat(100000),'v'),c=new RegExp(r);let a=c.exec('a')").unwrap();
    assert_eq!(
        realm.eval("a.length===100001&&a[0]==='a'&&a[100000]==='a'&&a.index===0"),
        Ok(Value::Boolean(true))
    );
    realm.collect(usize::MAX).unwrap();
    assert_eq!(
        realm.eval("c.exec('b'.repeat(100000)+'a').index===100000"),
        Ok(Value::Boolean(true))
    );
    realm.eval("r=null;c=null;a=null").unwrap();
    realm.collect(usize::MAX).unwrap();
}

#[test]
fn unicode_sets_bmp_opted_in_work_abort_and_unproved_modes_remain_distinct() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm.eval("let marker=0,r=new RegExp('a'.repeat(64),'vg');r.lastIndex=1;let text='b'.repeat(20000)").unwrap();
    assert!(matches!(
        realm.eval("try{r.exec(text)}catch{marker=1}finally{marker=2}"),
        Err(Error::Limit { .. })
    ));
    assert_eq!(
        realm.eval("marker===0&&r.lastIndex===1"),
        Ok(Value::Boolean(true))
    );
    for source in [
        r"/(?:a?)/v.exec('a')",
        r"/a/iv.exec('a')",
        r"/ab*/v.exec('a')",
        r"/\uD800/v.exec('\uD800')",
        r"/\u{D800}/v.exec('a')",
        r"/[\u{1f600}]/v.exec('a')",
        r"/^a/v.exec('a')",
        r"/(a)\1/v.exec('aa')",
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
