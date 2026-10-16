//! Word assertions around one complete Unicode class or fixed property.
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
fn unicode_class_word_boundary_results_snapshot() {
    let mut rows = String::new();
    for body in [
        r"(?<x>\b[A]\b)",
        r"(?<x>\B[\p{Any}]\B)",
        r"(?<x>^\b[\p{AHex}K]\b$)",
        r"\b\p{space}\B",
        r"\B\P{space}\b",
        r"\b[\b]\B",
        r"\b[A]",
        r"[\p{Any}]\B",
    ] {
        let pattern = JsString::from(body);
        for flags in ["dug", "diug", "divg", "divmy"] {
            for input in [
                JsString::from("A😀B a_ſK\n"),
                JsString::from("x\r\nK\u{2028}A\u{2029}"),
                JsString::from("A\u{8}!\u{85} "),
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
fn unicode_class_word_boundaries_single_sides_complements_and_case_neighbors() {
    for mode in ["u", "v"] {
        check(&format!(
            r"/\b[A]\b/{mode}.test('A')&&!/\b[A]\b/{mode}.test('AA')&&/\b[A]\B/{mode}.exec('AA').index===0&&/\B[A]\b/{mode}.exec('AA').index===1&&/\b[A]/{mode}.test('AA')&&/[A]\b/{mode}.exec('AA').index===1"
        ));
        check(&format!(
            r"/\B\p{{Any}}\B/{mode}.test('ſ')&&!/\B\p{{Any}}\B/i{mode}.test('ſ')&&/\b\p{{Any}}\b/i{mode}.test('ſ')&&/\b\p{{Any}}\b/i{mode}.test('K')&&!/\b\p{{Any}}\b/{mode}.test('K')"
        ));
        check(&format!(
            r"/\b\p{{space}}\B/{mode}.exec('A\u0085!').index===1&&/\b[\b]\B/{mode}.exec('A\b!')[0]==='\b'&&!/\b\P{{Any}}\b/{mode}.test('A')"
        ));
    }
}

#[test]
fn unicode_class_word_boundaries_captures_line_context_and_pair_starts() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let r=/(?<x>(^\b[\p{{AHex}}K]\b$))/dim{mode}g,m=r.exec('x\r\nK\u2028');m.index===3&&m[1]==='K'&&m[2]==='K'&&m.indices.groups.x===m.indices[1]&&m.indices[2][1]===4&&r.lastIndex===4"
        ));
        check(&format!(
            r"let r=/(?<__proto__>\b[\p{{Any}}]\b)/d{mode}y;r.lastIndex=2;let m=r.exec('A😀B');m.index===1&&m[1]==='😀'&&m.indices[1][1]===3&&r.lastIndex===3&&Object.getPrototypeOf(m.groups)===null&&m.indices.groups.__proto__===m.indices[1]"
        ));
        check(&format!(
            r"let r=/\b[\p{{Any}}]\b/{mode}y;r.lastIndex=1;r.exec('😀B')===null&&r.lastIndex===0"
        ));
        check(&format!(
            r"let a=/\b[\P{{space}}]\b/{mode}y,b=/\b[\P{{space}}]\b/{mode}y;a.lastIndex=1;b.lastIndex=1;a.exec('A\uD800B')[0]==='\uD800'&&b.exec('A\uDC00B')[0]==='\uDC00'"
        ));
    }
}

#[test]
fn unicode_class_word_boundary_consumers_keep_original_captures() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let a=[...'A A a'.matchAll(/(?<x>\b[A]\b)/di{mode}g)];a.length===3&&a[2][1]==='a'&&a[2].index===4&&a[2].indices.groups.x===a[2].indices[1]"
        ));
        check(&format!(
            r"'A A_ a'.replace(/(?<x>\b[A]\b)/i{mode}g,'<$1:$<x>>')==='<A:A> A_ <a:a>'"
        ));
        check(&format!(
            r"'A B A'.split(/(\b[A]\b)/{mode}).join('|')==='|A| B |A|'&&'xx A!'.search(/\b[A]\b/{mode})===3&&'A A_ a'.match(/\b[A]\b/i{mode}g).join('')==='Aa'"
        ));
        check(&format!(
            r"let t='';let x='A a'.replace(/(\b[A]\b)/i{mode}g,(m,c,p)=>{{t+=p;return c.toLowerCase()}});x==='a a'&&t==='02'"
        ));
    }
}

#[test]
fn unicode_class_word_boundary_clones_coercions_and_strict_writes() {
    check(
        r"let r=/(?<x>\b[\p{AHex}K]\b)/diug,c=new RegExp(r,'dvy');c.source==='(?<x>\\b[\\p{AHex}K]\\b)'&&!c.ignoreCase&&c.unicodeSets&&c.exec('K')===null&&c.exec('K').groups.x==='K'&&c.lastIndex===1",
    );
    check(
        r"let t='',r=/\b[\p{Any}]\b/uy;r.lastIndex={valueOf(){t+='i';return 2}};let m=r.exec({toString(){t+='s';return 'A😀B'}});t==='si'&&m[0]==='😀'&&m.index===1&&r.lastIndex===3",
    );
    check(
        r"let r=/\b[A]\b/vy;Object.defineProperty(r,'lastIndex',{writable:false});let caught=false;try{r.exec('A')}catch(e){caught=e instanceof TypeError}caught&&r.lastIndex===0",
    );
    check(
        r"let r=/\b[A]\b/vy;r.lastIndex=1;Object.defineProperty(r,'lastIndex',{writable:false});let caught=false;try{r.exec('AA')}catch(e){caught=e instanceof TypeError}caught&&r.lastIndex===1",
    );
}

#[test]
fn unicode_class_word_boundaries_deep_sources_and_collection_are_unlimited() {
    assert_eq!(Limits::default().max_steps, None);
    assert_eq!(Limits::default().max_heap_entries, None);
    let mut realm = Realm::default();
    realm.eval("let p='('.repeat(3000)+'\\\\b['+'\\\\p{AHex}'.repeat(10000)+']\\\\b'+')'.repeat(3000),r=new RegExp(p,'dvg'),c=new RegExp(r),text='😀'.repeat(100000)+' F '").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("let m=c.exec(text);m.length===3001&&m[1]==='F'&&m[3000]==='F'&&m.indices[3000][0]===200001&&m.indices[3000][1]===200002&&c.lastIndex===200002"),Ok(Value::Boolean(true)));
    realm.eval("p=null;r=null;c=null;text=null;m=null").unwrap();
    realm.collect(usize::MAX).unwrap();
}

#[test]
fn unicode_class_word_boundaries_opted_work_and_remaining_compositions() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval("let r=/\\b[A]\\b/ug,marker=0,text='😀'.repeat(10000);r.lastIndex=1")
        .unwrap();
    assert!(matches!(
        realm.eval("try{r.exec(text)}catch{marker=1}finally{marker=2}"),
        Err(Error::Limit { .. })
    ));
    assert_eq!(
        realm.eval("marker===0&&r.lastIndex===1"),
        Ok(Value::Boolean(true))
    );
    check(
        r"let m=/\b[[A]]\b/v.exec('A');m!==null&&m.length===1&&m[0]==='A'&&m.index===0&&m.input==='A'&&!/\b[[A]]\b/v.test('a')&&!/\b[[A]]\b/v.test('B')&&/\b[[A]]\b/iv.test('a')&&!/\b[[A]]\b/v.test('AA')",
    );
    for source in [
        r"/\bA+\bb/u.exec('A')",
        r"/\b([A])\bb/u.exec('A')",
        r"/\b[A]+\bb/u.exec('AA')",
        r"/\b[A][B]\b/v.exec('AB')",
        r"/\b\b[A]\b/u.exec('A')",
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
