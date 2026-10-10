//! Standalone zero-width Unicode word assertions with i/m and original captures.
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
fn unicode_word_assertion_results_snapshot() {
    let mut rows = String::new();
    for body in [
        r"(?<x>\b)",
        r"(?<x>\B)",
        r"(?<x>^\b)",
        r"(?<x>^\B)",
        r"\b$",
        r"\B$",
        r"(?<x>^\b$)",
        r"(?<x>^\B$)",
    ] {
        let pattern = JsString::from(body);
        for flags in ["dug", "diug", "divg", "divmy"] {
            for input in [
                JsString::from("A😀B a_ſK\n"),
                JsString::from("x\r\n \u{2028}A\u{2029}"),
                JsString::from("😀"),
                JsString::from_code_units(vec![0x41, 0xd800, 0x5f, 0xdc00, 0x42]),
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
fn unicode_word_assertions_empty_endpoints_case_neighbors_and_multiline() {
    for mode in ["u", "v"] {
        check(&format!(
            r"/\B/{mode}.test('')&&!/\b/{mode}.test('')&&/^\B$/{mode}.test('')&&!/^\b$/{mode}.test('')"
        ));
        check(&format!(
            r"/\B/{mode}.exec('ſ').index===0&&/\b/i{mode}.exec('ſ').index===0&&!/\B/i{mode}.test('ſ')&&/\b/i{mode}.test('K')&&!/\b/{mode}.test('K')"
        ));
        check(&format!(
            r"let r=/\b$/{mode}g,m=r.exec('A');m.index===1&&m[0]===''&&r.lastIndex===1&&r.exec('A').index===1"
        ));
        check(&format!(
            r"/^\B$/m{mode}.test('\n')&&/^\B$/m{mode}.test('\r\n')&&/^\B$/m{mode}.test('\u2028')&&/^\B$/m{mode}.test('\u2029')&&!/^\B$/{mode}.test('\n')"
        ));
        check(&format!(
            r"let r=/\b/{mode}g;r.exec('A').index===0&&r.lastIndex===0&&r.exec('A').index===0"
        ));
    }
}

#[test]
fn unicode_word_assertions_original_empty_captures_and_pair_starts() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let r=/(?<__proto__>(\b))/d{mode}y;r.lastIndex=2;let m=r.exec('A😀B');m.index===1&&m[0]===''&&m[1]===''&&m[2]===''&&m.indices[1][0]===1&&m.indices[1][1]===1&&m.indices.groups.__proto__===m.indices[1]&&r.lastIndex===1"
        ));
        check(&format!(
            r"let r=/(?<x>\B)/d{mode}y;r.lastIndex=1;let m=r.exec('😀');m.index===0&&m.indices.groups.x[0]===0&&m.indices.groups.x[1]===0&&r.lastIndex===0"
        ));
        check(&format!(
            r"let r=/\b/{mode}y;r.lastIndex=1;r.exec('😀')===null&&r.lastIndex===0"
        ));
        check(&format!(
            r"let r=/\B/{mode}g;r.lastIndex=3;r.exec('😀')===null&&r.lastIndex===0"
        ));
        check(&format!(
            r"let r=/^(?<x>(\B))$/d{mode};let m=r.exec('');m.groups.x===''&&m.indices[2][0]===0&&m.indices[2][1]===0"
        ));
        check(&format!(
            r"[...'A\uD800B'.matchAll(/\b/{mode}g)].map(m=>m.index).join(',')==='0,1,2,3'"
        ));
    }
}

#[test]
fn unicode_word_assertion_consumers_advance_after_original_empty_matches() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let a=[...'A😀B'.matchAll(/(?<x>\b)/d{mode}g)];a.map(m=>m.index).join(',')==='0,1,3,4'&&a.every(m=>m[0]===''&&m[1]===''&&m.indices.groups.x===m.indices[1])"
        ));
        check(&format!(
            r"'A😀B'.replace(/(?<x>\b)/{mode}g,'<$1$<x>>')==='<>A<>😀<>B<>'"
        ));
        check(&format!(
            r"'A😀B'.split(/\b/{mode}).join('|')==='A|😀|B'&&'A😀B'.split(/(\b)/{mode}).join('|')==='A||😀||B'&&'😀'.split(/\B/{mode}).join('|')==='😀'"
        ));
        check(&format!(
            r"'😀B'.search(/\b/{mode})===2&&'A😀B'.match(/\b/{mode}g).length===4&&'😀'.match(/\B/{mode}g).length===2"
        ));
        check(&format!(
            r"let t='',a='A😀B'.replace(/(\b)/{mode}g,(m,c,p)=>{{t+=p;return '|'}});a==='|A|😀|B|'&&t==='0134'"
        ));
    }
}

#[test]
fn unicode_word_assertion_clones_coercions_and_strict_writes() {
    check(
        r"let r=/(?<x>\b)/diug,c=new RegExp(r,'dvy'),b=new RegExp(c,'diuy');c.source==='(?<x>\\b)'&&!c.ignoreCase&&c.unicodeSets&&c.exec('ſ')===null&&b.exec('ſ').groups.x===''&&b.lastIndex===0",
    );
    check(
        r"let t='',r=/\b/uy;r.lastIndex={valueOf(){t+='i';return 2}};let m=r.exec({toString(){t+='s';return 'A😀B'}});t==='si'&&m[0]===''&&m.index===1&&r.lastIndex===1",
    );
    check(
        r"let r=/\B/vy;Object.defineProperty(r,'lastIndex',{writable:false});let caught=false;try{r.exec('')}catch(e){caught=e instanceof TypeError}caught&&r.lastIndex===0",
    );
    check(
        r"let r=/\b/vy;r.lastIndex=1;Object.defineProperty(r,'lastIndex',{writable:false});let caught=false;try{r.exec('😀')}catch(e){caught=e instanceof TypeError}caught&&r.lastIndex===1",
    );
}

#[test]
fn unicode_word_assertion_captures_collection_and_long_search_are_unlimited() {
    assert_eq!(Limits::default().max_steps, None);
    assert_eq!(Limits::default().max_heap_entries, None);
    let mut realm = Realm::default();
    realm.eval("let p='('.repeat(3000)+'\\\\b'+')'.repeat(3000),r=new RegExp(p,'dvg'),c=new RegExp(r),text='😀'.repeat(100000)+'A'").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("let m=c.exec(text);m.length===3001&&m[1]===''&&m[3000]===''&&m.indices[3000][0]===200000&&m.indices[3000][1]===200000&&c.lastIndex===200000"),Ok(Value::Boolean(true)));
    realm.eval("p=null;r=null;c=null;text=null;m=null").unwrap();
    realm.collect(usize::MAX).unwrap();
}

#[test]
fn unicode_word_assertions_opted_work_and_remaining_compositions() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval("let r=/\\b/ug,marker=0,text='😀'.repeat(10000);r.lastIndex=1")
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
        r"/\b\b/u.test('A')",
        r"/\b\B/u.test('A')",
        r"/(?:\b)+/u.test('A')",
        r"/\b|\B/v.test('A')",
        r"/\b(?=A)/u.test('A')",
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
fn unicode_word_assertion_former_whole_program_snapshot() {
    let source = r"/\b/iu.exec('a')";
    let mut realm = Realm::default();
    realm.eval(source).unwrap();
    let Value::String(result) = realm.eval(&format!("JSON.stringify({source})")).unwrap() else {
        panic!("{source}")
    };
    insta::assert_snapshot!(format!("{:?} {result:?}", JsString::from(source)));
}
