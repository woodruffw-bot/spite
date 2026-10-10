//! Pure Unicode line/word assertion sequences with original empty captures.
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
fn unicode_assertion_sequence_results_snapshot() {
    let mut rows = String::new();
    for body in [
        r"(?<x>^^)",
        r"(?<x>$$)",
        r"(?<x>$^)",
        r"(?<x>\b\b)",
        r"(?<x>\B\B)",
        r"(?<x>\b\B)",
        r"(?<x>\B^)",
        r"(?<x>$\B)",
        r"(?<x>\b^$\b)",
        r"(?<x>\B$^\B)",
        r"(?<x>^\b\B$)",
        r"(?<x>$\b^)",
    ] {
        let pattern = JsString::from(body);
        for flags in ["dug", "diug", "divg", "divmy"] {
            for input in [
                JsString::from("A😀B a_ſK\n"),
                JsString::from("A\r\n \u{2028}\u{2029}"),
                JsString::from("😀"),
                JsString::from_code_units(vec![0x41, 0xd800, 10, 0xdc00, 0x42]),
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
fn unicode_assertion_sequences_conjunctions_neighbors_and_multiline() {
    for mode in ["u", "v"] {
        check(&format!(
            r"/^^/{mode}.test('A')&&/$$/{mode}.exec('A').index===1&&/$^/{mode}.test('')&&!/$^/{mode}.test('A')"
        ));
        check(&format!(
            r"/\b\b/{mode}.exec('A').index===0&&!/\b\B/{mode}.test('A')&&!/\B\b/{mode}.test('')&&/\B\B/{mode}.test('')"
        ));
        check(&format!(
            r"/\B\B/{mode}.exec('ſ').index===0&&!/\B\B/i{mode}.test('ſ')&&/\b\b/i{mode}.exec('ſ').index===0&&/\b\b/i{mode}.test('K')"
        ));
        check(&format!(
            r"/\B$^\B/m{mode}.test('\n')&&/\B$^\B/m{mode}.exec('\r\n').index===0&&/\B$^\B/m{mode}.test('\u2028')&&/\B$^\B/m{mode}.test('\u2029')&&!/\B$^\B/{mode}.test('\n')"
        ));
        check(&format!(
            r"let r=/$\b\b/{mode}g,m=r.exec('A');m.index===1&&m[0]===''&&r.lastIndex===1&&r.exec('A').index===1"
        ));
        check(&format!(
            r"!/^\b\B$/m{mode}.test('\n')&&!/\b^$\b/{mode}.test('')&&!/\b^$\b/{mode}.test('A')"
        ));
    }
}

#[test]
fn unicode_assertion_sequences_empty_captures_and_pair_starts() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let r=/(?<__proto__>(\b\b))/d{mode}y;r.lastIndex=2;let m=r.exec('A😀B');m.index===1&&m[0]===''&&m[1]===''&&m[2]===''&&m.indices[1][0]===1&&m.indices[1][1]===1&&m.indices.groups.__proto__===m.indices[1]&&r.lastIndex===1"
        ));
        check(&format!(
            r"let r=/(?<x>^^)/d{mode}y;r.lastIndex=1;let m=r.exec('😀');m.index===0&&m.indices.groups.x[0]===0&&m.indices.groups.x[1]===0&&r.lastIndex===0"
        ));
        check(&format!(
            r"let r=/(?<x>$$)/d{mode}g;r.lastIndex=1;let m=r.exec('😀');m.index===2&&m.indices.groups.x[0]===2&&m.indices.groups.x[1]===2&&r.lastIndex===2"
        ));
        check(&format!(
            r"let r=/\b\b/{mode}y;r.lastIndex=1;r.exec('😀')===null&&r.lastIndex===0"
        ));
        check(&format!(
            r"let r=/\B\B/{mode}g;r.lastIndex=3;r.exec('😀')===null&&r.lastIndex===0"
        ));
        check(&format!(
            r"let m=/(?<x>($^))/d{mode}.exec('');m.groups.x===''&&m.indices[2][0]===0&&m.indices[2][1]===0"
        ));
    }
}

#[test]
fn unicode_assertion_sequence_consumers_advance_empty_matches() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let a=[...'A😀B'.matchAll(/(?<x>\b\b)/d{mode}g)];a.map(m=>m.index).join(',')==='0,1,3,4'&&a.every(m=>m[0]===''&&m[1]===''&&m.indices.groups.x===m.indices[1])"
        ));
        check(&format!(
            r"'A😀B'.replace(/(?<x>\b\b)/{mode}g,'<$1$<x>>')==='<>A<>😀<>B<>'"
        ));
        check(&format!(
            r"'A😀B'.split(/\b\b/{mode}).join('|')==='A|😀|B'&&'A😀B'.split(/(\b\b)/{mode}).join('|')==='A||😀||B'"
        ));
        check(&format!(
            r"'😀B'.search(/\b\b/{mode})===2&&'A😀B'.match(/\b\b/{mode}g).length===4&&'😀'.match(/^^/{mode}g).length===1"
        ));
        check(&format!(
            r"let t='',a='A\n😀'.replace(/(^^)/m{mode}g,(m,c,p)=>{{t+=p;return '|'}});a==='|A\n|😀'&&t==='02'"
        ));
    }
}

#[test]
fn unicode_assertion_sequence_clones_coercions_and_strict_writes() {
    check(
        r"let r=/(?<x>\b\b)/diug,c=new RegExp(r,'dvy'),b=new RegExp(c,'diuy');c.source==='(?<x>\\b\\b)'&&!c.ignoreCase&&c.unicodeSets&&c.exec('ſ')===null&&b.exec('ſ').groups.x===''&&b.lastIndex===0",
    );
    check(
        r"let t='',r=/\b\b/uy;r.lastIndex={valueOf(){t+='i';return 2}};let m=r.exec({toString(){t+='s';return 'A😀B'}});t==='si'&&m[0]===''&&m.index===1&&r.lastIndex===1",
    );
    check(
        r"let r=/\B\B/vy;Object.defineProperty(r,'lastIndex',{writable:false});let caught=false;try{r.exec('')}catch(e){caught=e instanceof TypeError}caught&&r.lastIndex===0",
    );
    check(
        r"let r=/\b\B/vy;r.lastIndex=1;Object.defineProperty(r,'lastIndex',{writable:false});let caught=false;try{r.exec('A')}catch(e){caught=e instanceof TypeError}caught&&r.lastIndex===1",
    );
}

#[test]
fn unicode_assertion_sequences_long_sources_captures_and_search_are_unlimited() {
    assert_eq!(Limits::default().max_steps, None);
    assert_eq!(Limits::default().max_heap_entries, None);
    let mut realm = Realm::default();
    realm.eval(r"let p='('.repeat(3000)+'\\b\\b'+')'.repeat(3000),r=new RegExp(p,'dvg'),c=new RegExp(r),text='😀'.repeat(100000)+'A',long=new RegExp('^'.repeat(40000),'v')").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("let m=c.exec(text);m.length===3001&&m[1]===''&&m[3000]===''&&m.indices[3000][0]===200000&&m.indices[3000][1]===200000&&c.lastIndex===200000&&long.exec(text).index===0"), Ok(Value::Boolean(true)));
    realm
        .eval("p=null;r=null;c=null;text=null;m=null;long=null")
        .unwrap();
    realm.collect(usize::MAX).unwrap();
}

#[test]
fn unicode_assertion_sequences_opted_work_and_remaining_compositions() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval(r"let r=/\b\B/ug,marker=0,text='😀'.repeat(10000);r.lastIndex=1")
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
        r"/(?:^^)+/u.test('A')",
        r"/(?:\b\b)+/u.test('A')",
        r"/\b\bA/u.test('A')",
        r"/^^|$$/v.test('A')",
        r"/^\b(?=A)/u.test('A')",
        r"/(\b)\b/u.test('A')",
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
fn unicode_assertion_sequence_former_whole_programs_snapshot() {
    let mut rows = String::new();
    for (source, expected) in [
        (r"/^^/u.test('A')", true),
        (r"/$$/u.test('A')", true),
        (r"/$^/u.test('A')", false),
        (r"/\b\b/u.test('A')", true),
        (r"/\b\B/u.test('A')", false),
    ] {
        let mut realm = Realm::default();
        assert_eq!(realm.eval(source), Ok(Value::Boolean(expected)));
        let Value::String(result) = realm.eval(&format!("JSON.stringify({source})")).unwrap()
        else {
            panic!("{source}")
        };
        writeln!(rows, "{:?} {result:?}", JsString::from(source)).unwrap();
    }
    insta::assert_snapshot!(rows);
}
