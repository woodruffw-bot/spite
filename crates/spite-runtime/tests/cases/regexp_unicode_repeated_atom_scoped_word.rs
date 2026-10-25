//! Outside word predicates preserve inner line and capture scopes.
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
fn unicode_repeated_atom_scoped_word_results_snapshot() {
    let mut rows = String::new();
    for atom in [
        r"\b(^a)+\B",
        r"\B((a$)){0,3}?",
        r"\b(?:(^a$)){2,4}\b",
        r"(?:^a)+\b",
        r"\B((?:a$)){0}\B",
        r"\b(^[aſK])+\b",
        r"((.$)){1,3}\b$",
        r"^\b((^\D$)){0,2}?\B$",
        r"^\B(^\p{Join_C})+\B$",
        r"(\p{NChar}$){1,2}\B$",
        r"\B(^😀)+\B",
        r"((\uD800$)){0,1}\b",
        r"\b(^\?)??",
        r"\B(?:(\W$))+\B",
        r"(^\u{61})?\b",
        r"\b((^a)){1,3}\b$",
    ] {
        let pattern = JsString::from(format!("(?<outer>{atom})").as_str());
        for flags in ["dug", "diumg", "divmg", "divy", "dsumg", "disvmy"] {
            for input in [
                JsString::from("Aaaa\nAA"),
                JsString::from("A\r\n\u{200c}\u{200d}\u{ffff}\u{10ffff}"),
                JsString::from("😀??aaſK"),
                JsString::from_code_units(vec![0x61, 0xd800, 0x61, 0xdc00]),
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
fn unicode_repeated_atom_scoped_word_counts_and_scopes() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let a=/\b(^a)+\B/{mode}.exec('aaa');a[0]==='a'&&a[1]==='a'&&a.index===0"
        ));
        check(&format!(
            r"/\b(^a)+\b/{mode}.exec('aaa')===null&&/(a$)+\b/{mode}.exec('aaa').index===2"
        ));
        check(&format!(r"/\b(?:(^a$)){{2}}\b/{mode}m.exec('a\na')===null"));
        check(&format!(
            r"let a=/\B((a$)){{0,3}}?/{mode}.exec('aa');a.index===1&&a[0]===''&&a[1]===undefined&&a[2]===undefined&&Object.hasOwn(a,'1')&&Object.hasOwn(a,'2')"
        ));
        check(&format!(
            r"let r=/\B((?:a$)){{0}}\B/d{mode}y;r.lastIndex=2;let m=r.exec('aaa');m.index===2&&m[0]===''&&m[1]===undefined&&m.indices[0][0]===2&&r.lastIndex===2"
        ));
        check(&format!(
            r"let a=/\B(^.$){{2,3}}\B/ds{mode}m.exec('\r\n\nA');a[0]==='\r\n'&&a[1]==='\n'&&a.indices[1][0]===1&&a.indices[1][1]===2"
        ));
        check(&format!(
            r"let a=/^\B((^.$))+?\B$/ds{mode}m.exec('\r\n\n'),b=/^\B((^.$))+\B$/ds{mode}m.exec('\r\n\n');a[0]==='\r'&&a[1]==='\r'&&b[0]==='\r\n\n'&&b[1]==='\n'&&b.indices[2][0]===2"
        ));
    }
}
#[test]
fn unicode_repeated_atom_scoped_word_names_pairs_and_folding() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let r=/(?<whole>\B(?<__proto__>^😀)+\B)/d{mode}y;r.lastIndex=1;let m=r.exec('😀😀');m.index===0&&m.groups.whole==='😀'&&m.groups.__proto__==='😀'&&m.indices.groups.__proto__[0]===0&&m.indices.groups.__proto__[1]===2&&m.indices.groups.__proto__===m.indices[2]&&r.lastIndex===2"
        ));
        check(&format!(
            r"let r=/\b((a$))+\b/di{mode}m,a=r.exec('A\nA'),b=new RegExp(r,'di{mode}').exec('A\nA');a.index===0&&a[0]==='A'&&a[1]==='A'&&b.index===2&&b[2]==='A'"
        ));
        check(&format!(
            r"let a=/(\uD800$)+\B/{mode}m.exec('B\uD800\n\uD800'),b=/\B(^\uD800)+\B/{mode}m.exec('\n\uD800\uD800');a.index===1&&a[0]==='\uD800'&&b.index===1&&b[0]==='\uD800'&&!/\B(\uD800$)+\B/{mode}.test('😀')"
        ));
    }
}
#[test]
fn unicode_repeated_atom_scoped_word_consumers_and_empty_advance() {
    for mode in ["u", "v"] {
        check(&format!(
            r"'aaa'.replace(/(?<whole>\b(?<last>^a)+\B)/{mode},'<$1:$2:$<last>>')==='<a:a:a>aa'"
        ));
        check(&format!(
            r"'aaa'.split(/((a$))+\b/{mode}).join('|')==='aa|a|a|'&&'baa'.search(/(a$)+\b/{mode})===2"
        ));
        check(&format!(
            r"let a=[...'a\na'.matchAll(/\b(?<x>^a$)+\b/d{mode}mg)];a.length===2&&a[0].index===0&&a[0].groups.x==='a'&&a[1].index===2&&a[1].indices.groups.x[0]===2"
        ));
        check(&format!(
            r"let a=[...'😀'.matchAll(/\B(?<x>^a){{0}}\B/d{mode}g)];a.length===2&&a[0].index===0&&a[1].index===2&&a.every(m=>m.groups.x===undefined&&Object.hasOwn(m.groups,'x')&&m.indices[1]===undefined)"
        ));
    }
}
#[test]
fn unicode_repeated_atom_scoped_word_clones_coercion_and_strict_writes() {
    check(
        r"let r=/\B(?<x>^.$)+\B/dsmug,c=new RegExp(r,'dvmy'),b=new RegExp(c,'dsumy');c.exec('\r\n')===null&&b.exec('\r\n').groups.x==='\n'&&b.lastIndex===2",
    );
    check(
        r"let t='',r=/\B(^😀)+\B/uy;r.lastIndex={valueOf(){t+='i';return 1}};let m=r.exec({toString(){t+='s';return '😀😀'}});t==='si'&&m.index===0&&m[1]==='😀'&&r.lastIndex===2",
    );
    check(
        r"let r=/\B(^a)*\B/vmy;Object.defineProperty(r,'lastIndex',{writable:false});let caught=false;try{r.exec('')}catch(e){caught=e instanceof TypeError}caught&&r.lastIndex===0",
    );
    check(
        r"let r=/\b(a$)+\b/vmy;r.lastIndex=1;Object.defineProperty(r,'lastIndex',{writable:false});let caught=false;try{r.exec('BBB')}catch(e){caught=e instanceof TypeError}caught&&r.lastIndex===1",
    );
}

#[test]
fn unicode_repeated_atom_scoped_word_opted_work_and_unproved_units() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval(r"let r=/\B(\P{NChar}$){2}\B/umg,marker=0,text='😀'.repeat(10000);r.lastIndex=1")
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
        r"/\b(^$)+\b/u.exec('')",
        r"/\b(\b.)+\bb/u.exec('a')",
        r"/\b(a?)+\b/u.exec('a')",
        r"/\b(a|b)+\bb/v.exec('a')",
        r"/\b()+\b/u.exec('a')",
        r"/\b(ab)+\b/v.exec('ab')",
        r"/\b(^a)+b\b/u.exec('ab')",
        r"/\b(^a(b))+\b/um.exec('ab')",
        r"/\b(a^)+\b/u.exec('a')",
    ] {
        assert!(
            crate::common::regexp_oracle::check(&mut Realm::default(), source),
            "{source}"
        );
    }
}

#[test]
fn unicode_repeated_atom_scoped_word_former_whole_programs_snapshot() {
    let mut rows = String::new();
    for source in [
        r"/\b(^a)+\b/um.exec('a')",
        r"/\b(^a)+/um.exec('a')",
        r"/\b(a$)+/um.exec('a')",
        r"/\b(a$)+\b/um.exec('a')",
    ] {
        let Value::String(result) = Realm::default()
            .eval(&format!("JSON.stringify({source})"))
            .unwrap()
        else {
            panic!("{source}")
        };
        writeln!(rows, "{source} {result:?}").unwrap();
    }
    insta::assert_snapshot!(rows);
}
