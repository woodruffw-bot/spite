//! Standalone zero-width Unicode line assertions with original empty captures.
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
fn unicode_line_assertion_results_snapshot() {
    let mut rows = String::new();
    for body in [r"(?<x>^)", r"(?<x>$)", r"(?<x>^$)"] {
        let pattern = JsString::from(body);
        for flags in ["dug", "diug", "divg", "divmy"] {
            for input in [
                JsString::from("A😀B\n"),
                JsString::from("A\r\nB\u{2028}\u{2029}"),
                JsString::from("😀"),
                JsString::from_code_units(vec![0xd800, 10, 0xdc00]),
                JsString::from(""),
            ] {
                for start in [0, 1, 2, input.len(), input.len() + 1] {
                    let script = format!(
                        "let r=new RegExp({pattern:?},'{flags}');r.lastIndex={start};let m=r.exec({input:?});JSON.stringify(m===null?{{match:null,lastIndex:r.lastIndex}}:{{matches:[...m],index:m.index,input:m.input,groups:m.groups,indices:m.indices,indexGroups:m.indices.groups,lastIndex:r.lastIndex,source:r.source}})"
                    );
                    let Value::String(result) = Realm::default().eval(&script).unwrap() else {
                        panic!("{script}")
                    };
                    writeln!(
                        rows,
                        "{pattern:?} flags={flags:?} input={input:?} lastIndex={start} {result:?}"
                    )
                    .unwrap();
                }
            }
        }
    }
    insta::assert_snapshot!(rows);
}

#[test]
fn unicode_line_assertions_empty_endpoints_and_multiline_context() {
    for mode in ["u", "v"] {
        check(&format!(
            r"/^$/{mode}.test('')&&!/^$/{mode}.test('A')&&!/^$/{mode}.test('\n')&&/^$/m{mode}.test('\n')&&/^$/m{mode}.test('\r\n')&&/^$/m{mode}.test('\u2028')&&/^$/m{mode}.test('\u2029')"
        ));
        check(&format!(
            r"let r=/$/{mode}g,m=r.exec('A\n');m.index===2&&m[0]===''&&r.lastIndex===2&&r.exec('A\n').index===2"
        ));
        check(&format!(
            r"let r=/$/m{mode}g,m=r.exec('A\n');m.index===1&&m[0]===''&&r.lastIndex===1"
        ));
        check(&format!(
            r"let r=/^/m{mode}g;r.lastIndex=1;let m=r.exec('A\r\nB');m.index===2&&m[0]===''&&r.lastIndex===2"
        ));
        check(&format!(
            r"let r=/^/{mode}g;r.lastIndex=2;r.exec('AB')===null&&r.lastIndex===0&&/^$/i{mode}.test('')"
        ));
    }
}

#[test]
fn unicode_line_assertions_original_empty_captures_and_pair_starts() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let r=/(?<__proto__>(^))/d{mode}y;r.lastIndex=1;let m=r.exec('😀');m.index===0&&m[0]===''&&m[1]===''&&m[2]===''&&m.indices[1][0]===0&&m.indices[1][1]===0&&m.indices.groups.__proto__===m.indices[1]&&r.lastIndex===0"
        ));
        check(&format!(
            r"let r=/(?<x>$)/d{mode}g;r.lastIndex=1;let m=r.exec('😀');m.index===2&&m.indices.groups.x[0]===2&&m.indices.groups.x[1]===2&&r.lastIndex===2"
        ));
        check(&format!(
            r"let r=/$/{mode}y;r.lastIndex=1;r.exec('😀')===null&&r.lastIndex===0"
        ));
        check(&format!(
            r"let r=/^/{mode}g;r.lastIndex=3;r.exec('😀')===null&&r.lastIndex===0"
        ));
        check(&format!(
            r"let r=/(?<x>(^$))/d{mode};let m=r.exec('');m.groups.x===''&&m.indices[2][0]===0&&m.indices[2][1]===0"
        ));
        check(&format!(
            r"[...'A\uD800\n\uDC00'.matchAll(/^/m{mode}g)].map(m=>m.index).join(',')==='0,3'"
        ));
    }
}

#[test]
fn unicode_line_assertion_consumers_advance_after_original_empty_matches() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let a=[...'A\r\n😀\n'.matchAll(/(?<x>^)/dm{mode}g)];a.map(m=>m.index).join(',')==='0,2,3,6'&&a.every(m=>m[0]===''&&m[1]===''&&m.indices.groups.x===m.indices[1])"
        ));
        check(&format!(
            r"'A\n😀'.replace(/(?<x>^)/m{mode}g,'<$1$<x>>')==='<>A\n<>😀'"
        ));
        check(&format!(
            r"'A\n😀'.split(/^/m{mode}).join('|')==='A\n|😀'&&'A\n😀'.split(/(^)/m{mode}).join('|')==='A\n||😀'&&'😀'.split(/$/{mode}).join('|')==='😀'"
        ));
        check(&format!(
            r"'😀'.search(/$/{mode})===2&&'A\n😀'.match(/^/m{mode}g).length===2&&'😀'.match(/$/{mode}g).length===1"
        ));
        check(&format!(
            r"let t='',a='A\n😀'.replace(/(^)/m{mode}g,(m,c,p)=>{{t+=p;return '|'}});a==='|A\n|😀'&&t==='02'"
        ));
    }
}

#[test]
fn unicode_line_assertion_clones_coercions_and_strict_writes() {
    check(
        r"let r=/(?<x>^$)/dimug,c=new RegExp(r,'dvy'),b=new RegExp(c,'dimuy');c.source==='(?<x>^$)'&&!c.multiline&&c.unicodeSets&&c.exec('\n')===null&&b.exec('\n').groups.x===''&&b.lastIndex===0",
    );
    check(
        r"let t='',r=/^/uy;r.lastIndex={valueOf(){t+='i';return 1}};let m=r.exec({toString(){t+='s';return '😀'}});t==='si'&&m[0]===''&&m.index===0&&r.lastIndex===0",
    );
    check(
        r"let r=/^$/vy;Object.defineProperty(r,'lastIndex',{writable:false});let caught=false;try{r.exec('')}catch(e){caught=e instanceof TypeError}caught&&r.lastIndex===0",
    );
    check(
        r"let r=/$/vy;r.lastIndex=1;Object.defineProperty(r,'lastIndex',{writable:false});let caught=false;try{r.exec('😀')}catch(e){caught=e instanceof TypeError}caught&&r.lastIndex===1",
    );
}

#[test]
fn unicode_line_assertion_captures_collection_and_long_search_are_unlimited() {
    assert_eq!(Limits::default().max_steps, None);
    assert_eq!(Limits::default().max_heap_entries, None);
    let mut realm = Realm::default();
    realm.eval("let p='('.repeat(3000)+'^$'+')'.repeat(3000),r=new RegExp(p,'dmvg'),c=new RegExp(r),text='😀'.repeat(100000)+'\\n'").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("let m=c.exec(text);m.length===3001&&m[1]===''&&m[3000]===''&&m.indices[3000][0]===200001&&m.indices[3000][1]===200001&&c.lastIndex===200001"),Ok(Value::Boolean(true)));
    realm.eval("p=null;r=null;c=null;text=null;m=null").unwrap();
    realm.collect(usize::MAX).unwrap();
}

#[test]
fn unicode_line_assertions_opted_work_and_remaining_compositions() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval("let r=/$/ug,marker=0,text='😀'.repeat(10000);r.lastIndex=1")
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
        r"/^^/u.test('A')",
        r"/$$/u.test('A')",
        r"/$^/u.test('A')",
        r"/(?:^)+/u.test('A')",
        r"/^|$/v.test('A')",
        r"/^(?=A)/u.test('A')",
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
fn unicode_line_assertion_former_whole_program_snapshot() {
    let source = r"/^$/.exec('')+/(?:^$)/u.exec('')";
    let mut realm = Realm::default();
    assert_eq!(realm.eval(source), Ok(Value::String(JsString::from(""))));
    let Value::String(result) = realm.eval(&format!("JSON.stringify({source})")).unwrap() else {
        panic!("{source}")
    };
    insta::assert_snapshot!(format!("{:?} {result:?}", JsString::from(source)));
}
