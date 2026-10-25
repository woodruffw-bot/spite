//! Outside word predicates compose with per-iteration word and line scopes.
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
fn unicode_repeated_atom_composed_word_results_snapshot() {
    let mut rows = String::new();
    for atom in [
        r"\b(\ba)+\B",
        r"\B((a\b)){0,3}?\b",
        r"\b(?:(\ba\b)){2,4}\b",
        r"\B(?:\Ba)+",
        r"((?:a\B)){0}\B",
        r"\b(^\b[aſK])+\b",
        r"((.\B$)){1,3}\B$",
        r"^\b((^\b\D\B$)){0,2}?\B$",
        r"^\B(\B\p{Join_C}\B)+\B$",
        r"(\p{NChar}\B$){1,2}\B$",
        r"\B(\B😀\B)+\B",
        r"((\uD800\b)){0,1}\b",
        r"\b(\b\?)??",
        r"\B(?:(\B\W\B$))+\B",
        r"(\u{61}\b)?\b",
        r"\b((^\ba\b)){1,3}\b$",
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
fn unicode_repeated_atom_composed_word_counts_and_scopes() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let m=/\b(\ba)+\B/{mode}.exec('aaa');m[0]==='a'&&m[1]==='a'&&m.index===0&&/\ba+\B/{mode}.exec('aaa')[0]==='aa'"
        ));
        check(&format!(
            r"let m=/\B(a\b)+\b/{mode}.exec('aaa');m[0]==='a'&&m[1]==='a'&&m.index===2"
        ));
        check(&format!(
            r"/\b(\ba){{2}}\b/{mode}.exec('aa')===null&&/\B(a\b){{2}}\b/{mode}.exec('aa')===null&&/\b(\ba\b)+\b/{mode}.exec('aa')===null"
        ));
        check(&format!(
            r"let m=/\B(\ba)*\B/{mode}.exec('aa');m.index===1&&m[0]===''&&m[1]===undefined"
        ));
        check(&format!(
            r"let m=/\B(\Ba\B)+\B/{mode}.exec('BaaaB');m[0]==='aaa'&&m[1]==='a'&&m.index===1"
        ));
        check(&format!(
            r"let m=/\B(^\B.\B$)+\B/{mode}ms.exec('\r\n');m[0]==='\r\n'&&m[1]==='\n'&&/\B(^\B.\B$){{3}}\B/{mode}ms.exec('\r\n')===null"
        ));
    }
}
#[test]
fn unicode_repeated_atom_composed_word_fold_surrogates_and_names() {
    check(
        r"let m=/(?<whole>\b(?<__proto__>\b[ſK]\B)+\B)/dvi.exec('ſKB');m[0]==='ſ'&&m[1]==='ſ'&&m[2]==='ſ'&&m.groups.__proto__==='ſ'&&m.indices.groups.__proto__[0]===0&&m.indices.groups.__proto__[1]===1",
    );
    check(
        r"let m=/(?<whole>\B(?<last>\B😀\B)+\B)/dvg.exec('😀😀');m[0]==='😀😀'&&m.groups.last==='😀'&&m.indices.groups.last[0]===2&&m.indices.groups.last[1]===4",
    );
    check(
        r"let r=/\B(\B😀\B)+\B/duy;r.lastIndex=1;let m=r.exec('😀😀');m.index===0&&m[0]==='😀😀'&&m.indices[1][0]===2&&r.lastIndex===4",
    );
    check(
        r"let m=/\b(?<x>\ba\b){0}\b/dv.exec('aa');m[0]===''&&m[1]===undefined&&m.groups.x===undefined&&m.indices[1]===undefined",
    );
}
#[test]
fn unicode_repeated_atom_composed_word_string_consumers() {
    check(r"let a='aa aa'.match(/\b(\ba)+\B/ug);a.length===2&&a.join('|')==='a|a'");
    check(r"'aaa'.replace(/\B(a\b)+\b/v,'<$1>')==='aa<a>'&&'aaa'.search(/\B(a\b)+\b/v)===2");
    check(r"let a='aa aa'.split(/\b(\ba)+\B/u);a.length===5&&a.join('|')==='|a|a |a|a'");
    check(
        r"let a=[...'aa aa'.matchAll(/\b(?<x>\ba)+\B/dvg)];a.length===2&&a[0].index===0&&a[1].index===3&&a[1].groups.x==='a'&&a[1].indices.groups.x[0]===3",
    );
}
#[test]
fn unicode_repeated_atom_composed_word_coercions_and_last_index() {
    check(
        r"let log='',r=/\B(a\b)+\b/vgy;r.lastIndex={valueOf(){log+='i';return 2}};let m=r.exec({toString(){log+='s';return 'aaa'}});log==='si'&&m.index===2&&r.lastIndex===3",
    );
    check(
        r"let r=/\b(\ba)+\B/ug;r.lastIndex=1;Object.defineProperty(r,'lastIndex',{writable:false});let caught=false;try{r.exec('aaa')}catch(e){caught=e instanceof TypeError}caught&&r.lastIndex===1",
    );
    check(
        r"let r=/\B(a\b)+\b/v;Object.defineProperty(r,'lastIndex',{value:100,writable:false});let m=r.exec('aaa');m.index===2&&r.lastIndex===100",
    );
}

#[test]
fn unicode_repeated_atom_composed_word_opted_work_and_unproved_units() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm.eval(r"let r=(new RegExp('\\B(\\B\\P{NChar}\\B$){2}\\B','umg')),marker=0,text='😀'.repeat(10000);r.lastIndex=1").unwrap();
    assert!(matches!(
        realm.eval("try{r.exec(text)}catch{marker=1}finally{marker=2}"),
        Err(Error::Limit { .. })
    ));
    assert_eq!(
        realm.eval("marker===0&&r.lastIndex===1"),
        Ok(Value::Boolean(true))
    );
    for source in [
        r"/\b(\b)+/u.exec('a')",
        r"/\B(^\B$)+/v.exec('')",
        r"/\b(\ba?)+/u.exec('a')",
        r"/\b(\ba|b)+b/v.exec('a')",
        r"/\b(\bab)+/u.exec('ab')",
        r"/\b(\ba)+b/v.exec('ab')",
        r"/\b(\ba)+\bb/u.exec('a')",
        r"/\b(a^\b)+/v.exec('a')",
    ] {
        assert!(
            crate::common::regexp_oracle::check(&mut Realm::default(), source),
            "{source}"
        );
    }
}

#[test]
fn unicode_repeated_atom_composed_word_former_whole_programs_snapshot() {
    let mut rows = String::new();
    for source in [
        r"/\b(\ba)+/u.exec('a')",
        r"/(\ba)+\b/v.exec('a')",
        r"/\b(\b.)+\b/u.exec('a')",
        r"/\b(\b.)+/v.exec('a')",
        r"/\b(\b.)+\b/v.exec('a')",
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
