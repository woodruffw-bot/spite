//! Unicode literal word boundaries with partial captures and original widths.
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
fn unicode_literal_word_boundary_results_snapshot() {
    let mut rows = String::new();
    for body in [
        r"\b(?<x>a)(a)\b",
        r"\B(?<x>a)(a)\b",
        r"\b(?<x>a())(a)\B",
        r"((\b(?<x>a)(s)\b))",
        r"\b(?<x>ſ)(K)\b",
        r"^\b(?<x>a)(a)\b$",
        r"\B(?<x>\u{1f600})(a)\b",
        r"\b(?<x>a)(\u{1f600})\B",
        r"\b(?<x>\uD800)(a)\b",
        r"\b(?<x>(a)b)\b",
        r"\B(?<x>a)(a)\B",
        r"(\b(?<x>a)(a)\b$)",
    ] {
        let pattern = JsString::from(body);
        for flags in ["dug", "diug", "divg", "divmy"] {
            for input in [
                JsString::from("aaa aa as ſK\n"),
                JsString::from("A\r\nAA\u{2028}as\u{2029}"),
                JsString::from("😀a a😀"),
                JsString::from_code_units(vec![0x41, 0xd800, 0x61, 0x20, 0xdc00, 0x41]),
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
fn unicode_literal_word_boundaries_overlap_line_context_and_folding() {
    for mode in ["u", "v"] {
        check(&format!(
            r"/\B(a)(a)\b/{mode}.exec('aaaa').index===2&&!/\b(a)(a)\b/{mode}.test('aaaa')&&/\b(a)(a)\b/{mode}.exec('aaaa aa').index===5"
        ));
        check(&format!(
            r"let m=/((\b(?<x>a)(s)\b))/di{mode}.exec('AS');m[0]==='AS'&&m[1]==='AS'&&m[2]==='AS'&&m.groups.x==='A'&&m.indices.groups.x===m.indices[3]&&m.indices[4][0]===1"
        ));
        check(&format!(
            r"let m=/\b(?<x>ſ)(K)\b/di{mode}.exec('SK');m[0]==='SK'&&m.groups.x==='S'&&m[2]==='K'&&!/\b(ſ)(K)\b/{mode}.test('ſK')"
        ));
        check(&format!(
            r"/^\b(a)(a)\b$/mi{mode}.exec('x\r\nAA\u2028').index===3&&!/^\b(a)(a)\b$/i{mode}.test('AA\n')"
        ));
        check(&format!(
            r"let m=/\b(?<x>a())(a)\B/d{mode}.exec('aaa');m.index===0&&m[2]===''&&m.indices[2][0]===1&&m.indices[2][1]===1&&m.indices.groups.x[1]===1"
        ));
    }
}

#[test]
fn unicode_literal_word_boundaries_original_pair_and_lone_surrogate_captures() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let r=/\B(?<__proto__>\u{{1f600}})(a)\b/d{mode}y;r.lastIndex=1;let m=r.exec('😀a');m.index===0&&m[0]==='😀a'&&m[1]==='😀'&&m.indices[1][1]===2&&m.indices[2][0]===2&&m.indices.groups.__proto__===m.indices[1]&&r.lastIndex===3"
        ));
        check(&format!(
            r"let m=/\b(a)(\u{{1f600}})\B/d{mode}.exec('a😀');m[0]==='a😀'&&m.indices[2][0]===1&&m.indices[2][1]===3"
        ));
        check(&format!(
            r"let m=/\b(?<x>\uD800)(a)\b/d{mode}.exec('A\uD800a');m.index===1&&m[1]==='\uD800'&&m.indices.groups.x[0]===1&&m.indices.groups.x[1]===2"
        ));
        check(&format!(
            r"let r=/\B(\uD800)(a)\b/{mode}g;r.exec('😀a')===null&&r.lastIndex===0"
        ));
        check(&format!(
            r"let m=/\b(\u{{10400}})(a)\b/di{mode}.exec('A\u{{10428}}a');m.index===1&&m[1]==='\u{{10428}}'&&m.indices[1][1]===3"
        ));
    }
}

#[test]
fn unicode_literal_word_boundary_consumers_keep_partial_original_captures() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let a=[...'aa AAA aa'.matchAll(/\b(?<x>a)(a)\b/d{mode}g)];a.map(m=>m.index).join(',')==='0,7'&&a.every(m=>m.groups.x==='a'&&m.indices.groups.x===m.indices[1])"
        ));
        check(&format!(
            r"'AA aa'.replace(/\b(?<x>a)(a)\b/i{mode}g,'<$1:$2:$<x>>')==='<A:A:A> <a:a:a>'"
        ));
        check(&format!(
            r"'x aa y'.split(/\b(a)(a)\b/{mode}).join('|')==='x |a|a| y'&&'x aa y'.search(/\b(a)(a)\b/{mode})===2"
        ));
        check(&format!(
            r"'aa aaaa aa'.match(/\b(a)(a)\b/{mode}g).join(',')==='aa,aa'"
        ));
        check(&format!(
            r"let t='',a='AS as'.replace(/\b(?<x>a)(s)\b/i{mode}g,(m,a,s,p,text,g)=>{{t+=p+g.x;return s+a}});a==='SA sa'&&t==='0A3a'"
        ));
    }
}

#[test]
fn unicode_literal_word_boundary_clones_coercions_and_strict_writes() {
    check(
        r"let r=/\b(?<x>ſ)(K)\b/diug,c=new RegExp(r,'dvy'),b=new RegExp(c,'diuy');!c.ignoreCase&&c.unicodeSets&&c.exec('ſK')===null&&b.exec('SK').groups.x==='S'&&b.lastIndex===2",
    );
    check(
        r"let t='',r=/\B(😀)(a)\b/uy;r.lastIndex={valueOf(){t+='i';return 1}};let m=r.exec({toString(){t+='s';return '😀a'}});t==='si'&&m.index===0&&r.lastIndex===3",
    );
    check(
        r"let r=/\b(a)(a)\b/vy;Object.defineProperty(r,'lastIndex',{writable:false});let caught=false;try{r.exec('aa')}catch(e){caught=e instanceof TypeError}caught&&r.lastIndex===0",
    );
    check(
        r"let r=/\b(a)(a)\b/vy;r.lastIndex=1;Object.defineProperty(r,'lastIndex',{writable:false});let caught=false;try{r.exec('aaaa')}catch(e){caught=e instanceof TypeError}caught&&r.lastIndex===1",
    );
}

#[test]
fn unicode_literal_word_boundaries_opted_work_and_remaining_compositions() {
    check(r"/\b[a]b\b/v.test('ab')");
    check(
        r#"let r=/\b[a]b\b/v,m=r.exec('ab');m!==null&&Array.isArray(m)&&m.length===1&&m[0]==='ab'&&m.index===0&&m.input==='ab'&&m.groups===undefined&&m.indices===undefined&&r.source==='\\b[a]b\\b'&&r.lastIndex===0&&r.exec('xb')===null&&r.exec('ax')===null&&r.exec('a')===null&&r.exec('xab')===null&&r.exec('!ab').index===1&&r.exec('abx')===null"#,
    );
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval(r"let r=/\b(a)(a)\b/ug,marker=0,text='a'.repeat(10000);r.lastIndex=1")
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
        r"/\b(a)+\bb/u.test('a')",
        r"/\b(a|b)\b/u.test('a')",
        r"/(\ba)b/u.test('ab')",
        r"/\b(a)(?=b)/u.test('ab')",
    ] {
        assert!(
            crate::common::regexp_oracle::check(&mut Realm::default(), source),
            "{source}"
        );
    }
}

#[test]
fn unicode_literal_word_boundary_former_whole_programs_snapshot() {
    let mut rows = String::new();
    for (source, expected) in [
        (r"/\b(A)\b/u.exec('A')", r#"["A","A"]"#),
        (r"/\bAB\b/v.exec('AB')", r#"["AB"]"#),
    ] {
        let mut realm = Realm::default();
        realm.eval(source).unwrap();
        let Value::String(result) = realm.eval(&format!("JSON.stringify({source})")).unwrap()
        else {
            panic!("{source}")
        };
        assert_eq!(result, JsString::from(expected));
        writeln!(rows, "{:?} {result:?}", JsString::from(source)).unwrap();
    }
    insta::assert_snapshot!(rows);
}
