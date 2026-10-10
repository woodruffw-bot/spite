//! Case-sensitive Unicode single-atom boundary assertions (22.2.2.4).
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
fn unicode_character_assertion_results_snapshot() {
    let mut rows = String::new();
    let inputs = [
        JsString::from("😀"),
        JsString::from("\n9\r\n😀\u{2028}"),
        JsString::from("a\n"),
        JsString::from("\r\n"),
        JsString::from_code_units(vec![0xd800, 0xa, 0xdc00]),
    ];
    for pattern in ["^.$", "^.", ".$", r"^\D$", r"\W$", r"^\s", r"^\d$"] {
        for flags in [
            "dug", "duy", "dmug", "dusy", "dmusy", "dvg", "dvy", "dmvg", "dvsy", "dmvsy",
        ] {
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
fn unicode_character_assertions_consumers_and_initial_pair_offsets() {
    for mode in ["u", "v"] {
        check(&format!(
            r"'a\n😀\r\n9'.match(/^.$/m{mode}g).join('|')==='a|😀|9'"
        ));
        check(&format!(
            r"let a=[...'a\n😀\r\n9'.matchAll(/^.$/dm{mode}g)];a.length===3&&a[1].index===2&&a[1].indices[0][1]===4&&a[2].index===6"
        ));
        check(&format!(
            r"let a='a\n😀\r\n9'.split(/^.$/m{mode});a.length===4&&a[0]===''&&a[1]==='\n'&&a[2]==='\r\n'&&a[3]===''"
        ));
        check(&format!(
            r"let seen=[];let s='a\n😀\r\n9'.replace(/^.$/m{mode}g,(m,i)=>{{seen.push(i);return '_'}});s==='_\n_\r\n_'&&seen.join(',')==='0,2,6'"
        ));
        check(&format!(
            r"let r=/^.$/m{mode}g;r.lastIndex=1;'9a\n😀'.search(r)===3&&r.lastIndex===1"
        ));
        check(&format!(
            r"let r=/^.$/d{mode}y;r.lastIndex=1;let m=r.exec('😀');m.index===0&&m[0]==='😀'&&m.indices[0][1]===2&&r.lastIndex===2"
        ));
        check(&format!(
            r"let r=/^./m{mode}y;r.lastIndex=2;let m=r.exec('\n😀');m.index===1&&m[0]==='😀'&&r.lastIndex===3"
        ));
        check(&format!(
            r"let r=/^./{mode}y;r.lastIndex=2;r.exec('\n😀')===null&&r.lastIndex===0"
        ));
    }
}

#[test]
fn unicode_character_assertions_exact_line_contexts_and_end_semantics() {
    check(r"/.$/u.exec('a\n')===null&&/.$/su.exec('a\n')[0]==='\n'");
    check(r"/^.$/u.exec('a\n')===null&&/^.$/su.exec('a\n')===null&&/^.$/su.exec('\n')[0]==='\n'");
    check(r"/.$/mu.exec('a\r\n').index===0&&/.$/msv.exec('\r\n')[0]==='\r'");
    check(
        r"let r=/^./msuy;r.lastIndex=1;let m=r.exec('\r\n');m[0]==='\n'&&m.index===1&&r.lastIndex===2",
    );
    check(r"/^\s$/u.test('\uFEFF')&&/^\D$/v.test('😀')&&/^\w$/u.test('k')&&!/^\w$/u.test('K')");
    check(
        r"let r=/^.$/dmsv,c=new RegExp(r,'dug');c.source==='^.$'&&!c.multiline&&!c.dotAll&&c.unicode&&!c.unicodeSets&&c.exec('😀')[0]==='😀'&&c.lastIndex===2",
    );
}

#[test]
fn unicode_character_assertions_coercions_and_strict_writes() {
    check(
        r"let t='',r=/^.$/uy;r.lastIndex={valueOf(){t+='i';return 1}};let m=r.exec({toString(){t+='s';return '😀'}});t==='si'&&m.index===0&&r.lastIndex===2",
    );
    check(
        r"let t='',r=/^.$/v;r.lastIndex={valueOf(){t+='i';return 1}};let m=r.exec({toString(){t+='s';return '😀'}});t==='si'&&m.index===0&&typeof r.lastIndex==='object'",
    );
    check(
        r"let r=/^.$/uy;r.lastIndex=1;Object.defineProperty(r,'lastIndex',{writable:false});let caught=false;try{r.exec('😀')}catch(e){caught=e instanceof TypeError}caught&&r.lastIndex===1",
    );
    check(
        r"let r=/^\d$/vy;r.lastIndex=1;Object.defineProperty(r,'lastIndex',{writable:false});let caught=false;try{r.exec('😀')}catch(e){caught=e instanceof TypeError}caught&&r.lastIndex===1",
    );
}

#[test]
fn unicode_character_assertions_clones_collection_and_unlimited_defaults() {
    assert_eq!(Limits::default().max_steps, None);
    assert_eq!(Limits::default().max_heap_entries, None);
    let mut realm = Realm::default();
    realm
        .eval(r"let r=/^.$/mvg,c=new RegExp(r),text='9'.repeat(100000)+'\n😀'")
        .unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(
        realm.eval("let m=c.exec(text);m.index===100001&&m[0]==='😀'&&c.lastIndex===100003"),
        Ok(Value::Boolean(true))
    );
    realm.eval("r=null;c=null;text=null;m=null").unwrap();
    realm.collect(usize::MAX).unwrap();
}

#[test]
fn unicode_character_assertions_opted_work_and_unproved_compositions() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval("let r=/^.$/ug,marker=0,text='😀'.repeat(10000);r.lastIndex=1")
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
        r"/(^.$)/iu.exec('a')",
        r"/^(.)$/v.exec('a')",
        r"/^a$/u.exec('a')",
        r"/^([a])$/v.exec('a')",
        r"/^..$/u.exec('ab')",
        r"/^\D+$/v.exec('ab')",
        r"/^.$|b/u.exec('b')",
        r"/^\p{ASCII}$/v.exec('a')",
        r"/^$/.exec('')+/(?:^$)/u.exec('')",
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
fn original_unicode_single_atom_anchor_complete_program() {
    let source = r"/^./u.exec('a')";
    let Value::String(result) = Realm::default()
        .eval(&format!("JSON.stringify({source})"))
        .unwrap()
    else {
        panic!("{source}")
    };
    insta::assert_snapshot!(format!("{:?} {result:?}", JsString::from(source)));
}
