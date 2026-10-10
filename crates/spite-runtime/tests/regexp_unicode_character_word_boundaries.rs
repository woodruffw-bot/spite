//! Word assertions around one complete Unicode literal/dot/escape atom.
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
fn unicode_character_word_boundary_results_snapshot() {
    let mut rows = String::new();
    for body in [
        r"(?<x>\bA\b)",
        r"(?<x>\B.\B)",
        r"(?<x>^\b\d\b$)",
        r"\b\D\B",
        r"\b\s\B",
        r"\B\S\b",
        r"\b\w\b",
        r"\b\W\b",
        r"(?<x>\b\u{1f600}\b)",
        r"(?<x>\b\uD800\b)",
        r"\b\$\B",
        r"\b\\\B",
        r"\bA",
        r".\B",
        r"(?<x>^\B\u{10400}\B$)",
        r"\b\u{000041}\b",
    ] {
        let pattern = JsString::from(body);
        for flags in ["dug", "diug", "divg", "divmsy"] {
            for input in [
                JsString::from("A😀B a_ſK\n"),
                JsString::from("x\r\n8\u{2028}𐐨\u{2029}"),
                JsString::from("A$!A\\!A \u{85}\u{feff}"),
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
fn unicode_character_word_boundaries_single_sides_and_independent_neighbor_folding() {
    for mode in ["u", "v"] {
        check(&format!(
            r"/\bA\b/{mode}.test('A')&&!/\bA\b/{mode}.test('AA')&&/\bA\B/{mode}.exec('AA').index===0&&/\BA\b/{mode}.exec('AA').index===1&&/\bA/{mode}.test('AA')&&/A\b/{mode}.exec('AA').index===1"
        ));
        check(&format!(
            r"/\B.\B/{mode}.test('ſ')&&!/\B.\B/i{mode}.test('ſ')&&/\b.\b/i{mode}.test('ſ')&&/\b.\b/i{mode}.test('K')&&!/\b.\b/{mode}.test('K')"
        ));
        check(&format!(
            r"/\b\d\b/{mode}.test('8ſ')&&!/\b\d\b/i{mode}.test('8ſ')&&/\b\d\B/i{mode}.test('8K')===true"
        ));
        check(&format!(
            r"/\B\s\b/{mode}.test(' ſ')===false&&/\B\s\b/i{mode}.test(' ſ')&&/\b\s\B/i{mode}.test('K ')"
        ));
        check(&format!(
            r"/\b\w\b/i{mode}.test('ſ')&&/\b\w\b/i{mode}.test('K')&&!/\b\W\b/i{mode}.test('ſ')&&/\b\W\b/{mode}.test('A😀B')"
        ));
        check(&format!(
            r"/\b\$\B/{mode}.exec('A$!').index===1&&/\b\\\B/{mode}.exec('A\\!').index===1&&/\b\s\B/{mode}.test('A\uFEFF!')&&!/\b\s\B/{mode}.test('A\u0085!')"
        ));
    }
}

#[test]
fn unicode_character_word_boundaries_captures_lines_dotall_and_pair_starts() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let r=/(?<x>(^\B\u{{10400}}\B$))/dim{mode}g,m=r.exec('x\r\n𐐨\u2028');m.index===3&&m[1]==='𐐨'&&m[2]==='𐐨'&&m.indices.groups.x===m.indices[1]&&m.indices[2][1]===5&&r.lastIndex===5"
        ));
        check(&format!(
            r"let r=/(?<__proto__>\b.\b)/d{mode}y;r.lastIndex=2;let m=r.exec('A😀B');m.index===1&&m[1]==='😀'&&m.indices[1][1]===3&&r.lastIndex===3&&Object.getPrototypeOf(m.groups)===null&&m.indices.groups.__proto__===m.indices[1]"
        ));
        check(&format!(
            r"let r=/\b\u{{1f600}}\b/{mode}y;r.lastIndex=1;r.exec('😀B')===null&&r.lastIndex===0"
        ));
        check(&format!(
            r"let a=/\b\uD800\b/{mode}y,b=/\b\uDC00\b/{mode}y;a.lastIndex=1;b.lastIndex=1;a.exec('A\uD800B')[0]==='\uD800'&&b.exec('A\uDC00B')[0]==='\uDC00'"
        ));
        check(&format!(
            r"/\b.\B/s{mode}.exec('A\n!').index===1&&!/\b.\B/{mode}.test('A\n!')"
        ));
    }
}

#[test]
fn unicode_character_word_boundary_consumers_keep_original_captures() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let a=[...'A A a'.matchAll(/(?<x>\bA\b)/di{mode}g)];a.length===3&&a[2][1]==='a'&&a[2].index===4&&a[2].indices.groups.x===a[2].indices[1]"
        ));
        check(&format!(
            r"'A A_ a'.replace(/(?<x>\bA\b)/i{mode}g,'<$1:$<x>>')==='<A:A> A_ <a:a>'"
        ));
        check(&format!(
            r"'A B A'.split(/(\bA\b)/{mode}).join('|')==='|A| B |A|'&&'xx A!'.search(/\bA\b/{mode})===3&&'A A_ a'.match(/\bA\b/i{mode}g).join('')==='Aa'"
        ));
        check(&format!(
            r"let t='';let x='A a'.replace(/(\bA\b)/i{mode}g,(m,c,p)=>{{t+=p;return c.toLowerCase()}});x==='a a'&&t==='02'"
        ));
    }
}

#[test]
fn unicode_character_word_boundary_clones_coercions_and_strict_writes() {
    check(
        r"let r=/(?<x>\bA\b)/diug,c=new RegExp(r,'dvy'),b=new RegExp(c,'diuy');c.source==='(?<x>\\bA\\b)'&&!c.ignoreCase&&c.unicodeSets&&c.exec('a')===null&&b.exec('a').groups.x==='a'&&b.lastIndex===1",
    );
    check(
        r"let t='',r=/\b.\b/uy;r.lastIndex={valueOf(){t+='i';return 2}};let m=r.exec({toString(){t+='s';return 'A😀B'}});t==='si'&&m[0]==='😀'&&m.index===1&&r.lastIndex===3",
    );
    check(
        r"let r=/\bA\b/vy;Object.defineProperty(r,'lastIndex',{writable:false});let caught=false;try{r.exec('A')}catch(e){caught=e instanceof TypeError}caught&&r.lastIndex===0",
    );
    check(
        r"let r=/\bA\b/vy;r.lastIndex=1;Object.defineProperty(r,'lastIndex',{writable:false});let caught=false;try{r.exec('AA')}catch(e){caught=e instanceof TypeError}caught&&r.lastIndex===1",
    );
}

#[test]
fn unicode_character_word_boundaries_deep_sources_and_collection_are_unlimited() {
    assert_eq!(Limits::default().max_steps, None);
    assert_eq!(Limits::default().max_heap_entries, None);
    let mut realm = Realm::default();
    realm.eval("let p='('.repeat(3000)+'\\\\b\\\\u{'+'0'.repeat(10000)+'41}\\\\b'+')'.repeat(3000),r=new RegExp(p,'dvg'),c=new RegExp(r),text='😀'.repeat(100000)+' A '").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("let m=c.exec(text);m.length===3001&&m[1]==='A'&&m[3000]==='A'&&m.indices[3000][0]===200001&&m.indices[3000][1]===200002&&c.lastIndex===200002"),Ok(Value::Boolean(true)));
    realm.eval("p=null;r=null;c=null;text=null;m=null").unwrap();
    realm.collect(usize::MAX).unwrap();
}

#[test]
fn unicode_character_word_boundaries_opted_work_and_remaining_compositions() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval("let r=/\\bA\\b/ug,marker=0,text='😀'.repeat(10000);r.lastIndex=1")
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
        r"/\b(A)\b/u.exec('A')",
        r"/\bA+\b/u.exec('AA')",
        r"/\bAB\b/v.exec('AB')",
        r"/\b\bA\b/u.exec('A')",
        r"/\b(?=A)A\b/u.exec('A')",
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
fn unicode_character_word_boundary_former_whole_program_snapshot() {
    let mut rows = String::new();
    for source in [r"/\bA\b/u.exec('A')", r"/\ba\b/u.test('a')"] {
        let mut realm = Realm::default();
        realm.eval(source).unwrap();
        let Value::String(result) = realm.eval(&format!("JSON.stringify({source})")).unwrap()
        else {
            panic!("{source}")
        };
        writeln!(rows, "{:?} {result:?}", JsString::from(source)).unwrap();
    }
    insta::assert_snapshot!(rows);
}
