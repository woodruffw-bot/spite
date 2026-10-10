//! Unicode dot and character escapes with i (22.2.2.7.3, 22.2.2.9).
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
fn unicode_character_folding_results_snapshot() {
    let mut rows = String::new();
    let inputs = [
        JsString::from("ſK😀\n9 A\u{feff}"),
        JsString::from_code_units(vec![0xd800, 0xa, 0xdc00, 0xd83d, 0xde00]),
        JsString::from("ßẞİıﬀ\u{1df95}"),
        JsString::from("\r\nſ\u{2028}K\u{2029}"),
    ];
    for body in [".", r"\d", r"\D", r"\w", r"\W", r"\s", r"\S"] {
        for (left, right) in [("", ""), ("^", ""), ("", "$"), ("^", "$")] {
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
                        writeln!(rows, "{pattern:?} flags={flags:?} input={input:?} lastIndex={start} {result:?}").unwrap();
                    }
                }
            }
        }
    }
    insta::assert_snapshot!(rows);
}

#[test]
fn unicode_character_folding_consumers_and_original_text() {
    for mode in ["u", "v"] {
        check(&format!(
            r"'ſK😀 9'.match(/\w/i{mode}g).join('|')==='ſ|K|9'"
        ));
        check(&format!(
            r"let a=[...'ſK😀 9'.matchAll(/\w/di{mode}g)];a.length===3&&a[0][0]==='ſ'&&a[1][0]==='K'&&a[2].index===5&&a[1].indices[0][1]===2"
        ));
        check(&format!(
            r"'ſK😀 9'.split(/\w/i{mode}).join('|')==='||😀 |'"
        ));
        check(&format!(
            r"let seen=[];let s='ſK😀 9'.replace(/\w/i{mode}g,(m,i)=>{{seen.push(i);return '_'}});s==='__😀 _'&&seen.join(',')==='0,1,5'"
        ));
        check(&format!(
            r"let r=/\W/di{mode}y;r.lastIndex=3;let m=r.exec('ſK😀 9');m.index===2&&m[0]==='😀'&&m.indices[0][1]===4&&r.lastIndex===4"
        ));
        check(&format!(
            r"let r=/\w/i{mode}y;r.lastIndex=3;r.exec('ſK😀 9')===null&&r.lastIndex===0"
        ));
        check(&format!(
            r"let r=/^\w$/i{mode}m;r.lastIndex=2;'😀\nſ'.search(r)===3&&r.lastIndex===2"
        ));
    }
}

#[test]
fn unicode_word_folding_is_simple_common_and_has_complete_boundaries() {
    check(r"/\w/iu.test('ſ')&&/\w/iv.test('K')&&!/\w/u.test('ſ')&&!/\w/v.test('K')");
    check(r"/\W/iu.exec('ſ')===null&&/\W/iv.exec('K')===null");
    check(r"/\w/iu.exec('ßẞİıﬀ\u{1df95}')===null&&/\W/iv.test('ßẞİıﬀ\u{1df95}')");
    check(r"/^\w$/ium.test('\rſ\n')&&/^\w$/ivm.test('\u2028K\u2029')");
    check(r"/^\w$/iu.exec('ſ\n')===null&&/^\w$/iv.exec('K\r')===null");
    check(r"/^.$/iu.test('😀')&&!/^.$/iv.test('\n')&&/^.$/ivs.test('\n')");
    check(
        r"/^\d$/iu.test('9')&&/^\D$/iv.test('😀')&&/^\s$/iu.test('\uFEFF')&&/^\S$/iv.test('\u180E')",
    );
}

#[test]
fn unicode_character_folding_metadata_clones_coercions_and_strict_writes() {
    check(
        r"let r=/^\w$/dimv,c=new RegExp(r,'diuy');c.source==='^\\w$'&&c.ignoreCase&&c.unicode&&!c.unicodeSets&&!c.multiline&&c.exec('K')[0]==='K'&&c.lastIndex===1",
    );
    check(
        r"let t='',r=/\w/iuy;r.lastIndex={valueOf(){t+='i';return 0}};let m=r.exec({toString(){t+='s';return 'ſ'}});t==='si'&&m[0]==='ſ'&&r.lastIndex===1",
    );
    check(
        r"let r=/\w/iuy;Object.defineProperty(r,'lastIndex',{writable:false});let caught=false;try{r.exec('ſ')}catch(e){caught=e instanceof TypeError}caught&&r.lastIndex===0",
    );
    check(
        r"let r=/\w/ivy;r.lastIndex=1;Object.defineProperty(r,'lastIndex',{writable:false});let caught=false;try{r.exec('😀')}catch(e){caught=e instanceof TypeError}caught&&r.lastIndex===1",
    );
}

#[test]
fn unicode_character_folding_collection_long_searches_and_unlimited_defaults() {
    assert_eq!(Limits::default().max_steps, None);
    assert_eq!(Limits::default().max_heap_entries, None);
    let mut realm = Realm::default();
    realm
        .eval(r"let r=/\w/ivg,c=new RegExp(r),text='😀'.repeat(100000)+'K'")
        .unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(
        realm.eval("let m=c.exec(text);m[0]==='K'&&m.index===200000&&c.lastIndex===200001"),
        Ok(Value::Boolean(true))
    );
    realm.eval("r=null;c=null;text=null;m=null").unwrap();
    realm.collect(usize::MAX).unwrap();
}

#[test]
fn unicode_character_folding_opted_work_and_unproved_syntax() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval("let r=/\\w/iug,marker=0,text='😀'.repeat(10000);r.lastIndex=1")
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
        r"/([^a])+/iv.exec('b')",
        r"/(\w)+/iu.exec('a')",
        r"/\w+/iv.exec('a')",
        r"/\w\d/iu.exec('a9')",
        r"/\p{Assigned}/iv.exec('a')",
        r"/\b/iu.exec('a')",
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
fn original_unicode_character_folding_complete_programs() {
    let mut rows = String::new();
    for source in [
        r"/^.$/iu.exec('a')",
        r"/./iu.exec('a')",
        r"/\W/iv.exec('a')",
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
