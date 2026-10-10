//! Single case-sensitive Unicode dot and character escapes (22.2.2.7.1, 22.2.2.9).
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
fn unicode_dot_and_escape_results_snapshot() {
    let mut rows = String::new();
    let inputs = [
        JsString::from("😀a9_"),
        JsString::from("\n\r\u{2028}\u{2029}x"),
        JsString::from("\t \u{a0}\u{feff}x"),
        JsString::from("Kſéσ"),
        JsString::from_code_units(vec![0xd800, 0x61, 0xdc00, 0xd83d, 0xde00]),
    ];
    for pattern in [".", r"\d", r"\D", r"\s", r"\S", r"\w", r"\W"] {
        for flags in ["dug", "duy", "dusy", "dvg", "dvy", "dvsy"] {
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
fn unicode_dot_escapes_consumers_and_complete_character_boundaries() {
    for mode in ["u", "v"] {
        check(&format!(
            r"'😀a\n\uD800'.match(/./{mode}g).join('|')==='😀|a|\uD800'"
        ));
        check(&format!(
            r"let a=[...'😀a\n'.matchAll(/./d{mode}g)];a.length===2&&a[0].index===0&&a[0].indices[0][1]===2&&a[1].index===2"
        ));
        check(&format!(r"'😀a😀b'.split(/\D/{mode}).join('|')==='||||'"));
        check(&format!(
            r"let seen=[];let s='😀9\uD800'.replace(/\D/{mode}g,(m,i)=>{{seen.push(i);return '_'}});s==='_9_'&&seen.join(',')==='0,3'"
        ));
        check(&format!(
            r"let r=/\W/{mode}g;r.lastIndex=1;'a😀'.search(r)===1&&r.lastIndex===1"
        ));
        check(&format!(
            r"let r=/./d{mode}y;r.lastIndex=1;let m=r.exec('😀');m[0]==='😀'&&m.index===0&&m.indices[0][0]===0&&m.indices[0][1]===2&&r.lastIndex===2"
        ));
        check(&format!(
            r"let r=/\d/{mode}y;r.lastIndex=1;r.exec('😀9')===null&&r.lastIndex===0"
        ));
        check(&format!(
            r"let r=/\D/{mode}g;r.lastIndex=1;let m=r.exec('😀9');m.index===0&&m[0]==='😀'&&r.lastIndex===2"
        ));
        check(&format!(
            r"let r=/./{mode}y;r.lastIndex=1;let m=r.exec('\uDC00\uD800');m[0]==='\uD800'&&m.index===1&&r.lastIndex===2"
        ));
    }
}

#[test]
fn unicode_dot_escapes_exact_membership_and_original_metadata() {
    check(r"/./u.exec('\n\r\u2028\u2029x').index===4&&/./su.exec('\n')[0]==='\n'");
    check(r"/\s/v.exec('\u180E\uFEFF').index===1&&/\S/v.exec('\uFEFF😀')[0]==='😀'");
    check(r"/\w/u.exec('Kſk').index===2&&/\W/v.exec('kK')[0]==='K'");
    check(r"/\d/u.exec('９9').index===1&&/\D/v.exec('9９')[0]==='９'");
    check(
        r"let r=/./dsv,c=new RegExp(r,'dug');c.source==='.'&&c.unicode&&!c.unicodeSets&&!c.dotAll&&c.exec('\nx').index===1&&c.lastIndex===2",
    );
    check(
        r"let r=/\D/dvg,m=r.exec('😀');m.length===1&&m.groups===undefined&&m.indices.groups===undefined&&r.source==='\\D'",
    );
    check(r"/./.exec('😀')[0]==='\uD83D'&&/./u.exec('😀')[0]==='😀'");
}

#[test]
fn unicode_dot_escapes_ordered_coercions_and_strict_lastindex() {
    check(
        r"let t='',r=/./uy;r.lastIndex={valueOf(){t+='i';return 1}};let m=r.exec({toString(){t+='s';return '😀'}});t==='si'&&m.index===0&&r.lastIndex===2",
    );
    check(
        r"let t='',r=/\D/v;r.lastIndex={valueOf(){t+='i';return 1}};let m=r.exec({toString(){t+='s';return '😀'}});t==='si'&&m.index===0&&typeof r.lastIndex==='object'",
    );
    check(
        r"let r=/./uy,n=0,e={};r.lastIndex={valueOf(){n++;return 0}};let caught=false;try{r.exec({toString(){throw e}})}catch(x){caught=x===e}caught&&n===0&&typeof r.lastIndex==='object'",
    );
    check(
        r"let r=/./uy;r.lastIndex=1;Object.defineProperty(r,'lastIndex',{writable:false});let caught=false;try{r.exec('😀')}catch(e){caught=e instanceof TypeError}caught&&r.lastIndex===1",
    );
    check(
        r"let r=/\d/vy;r.lastIndex=1;Object.defineProperty(r,'lastIndex',{writable:false});let caught=false;try{r.exec('😀')}catch(e){caught=e instanceof TypeError}caught&&r.lastIndex===1",
    );
}

#[test]
fn unicode_dot_escapes_clones_collection_and_unlimited_defaults() {
    assert_eq!(Limits::default().max_steps, None);
    assert_eq!(Limits::default().max_heap_entries, None);
    let mut realm = Realm::default();
    realm
        .eval("let r=/\\D/vg,c=new RegExp(r),text='9'.repeat(100000)+'😀'")
        .unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(
        realm.eval("let m=c.exec(text);m.index===100000&&m[0]==='😀'&&c.lastIndex===100002"),
        Ok(Value::Boolean(true))
    );
    realm.eval("r=null;c=null;text=null;m=null").unwrap();
    realm.collect(usize::MAX).unwrap();
}

#[test]
fn unicode_dot_escapes_opted_work_preserves_state_and_pending_completion() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval("let r=/\\d/ug,marker=0,text='😀'.repeat(10000);r.lastIndex=1")
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
fn unicode_dot_escapes_unproved_syntax_remains_unsupported() {
    for source in [
        r"/./iu.exec('a')",
        r"/\W/iv.exec('a')",
        r"/(.)/u.exec('a')",
        r"/./v.exec('a')+/(.)/v.exec('a')",
        r"/\D+/u.exec('a')",
        r"/a./v.exec('ab')",
        r"/^../u.exec('a')",
        r"/([\D])/u.exec('a')",
        r"/([^a])/v.exec('b')",
        r"/\p{ASCII}/u.exec('a')",
        r"/[\q{ab}]/v.exec('ab')",
        r"/\d|a/u.exec('a')",
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
fn original_unicode_dot_escape_complete_programs() {
    let mut rows = String::new();
    for source in [
        r"/./v.test('a')",
        r"/\D/u.exec('a')",
        r"/./u.exec('a')",
        r"/\D/v.exec('a')",
        r"/./v.exec('a')",
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
