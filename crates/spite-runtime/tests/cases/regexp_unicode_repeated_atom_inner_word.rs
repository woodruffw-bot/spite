//! Word and line predicates are checked at each consuming iteration.
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
fn unicode_repeated_atom_inner_word_results_snapshot() {
    let mut rows = String::new();
    for atom in [
        r"(\ba)+",
        r"((a\b)){0,3}?",
        r"(?:(\ba\b)){2,4}",
        r"(?:\Ba)+",
        r"((?:a\B)){0}",
        r"(^\b[aſK])+",
        r"((.\B$)){1,3}$",
        r"^((^\b\D\B$)){0,2}?$",
        r"^(\B\p{Join_C}\B)+$",
        r"(\p{NChar}\B$){1,2}$",
        r"(\B😀\B)+",
        r"((\uD800\b)){0,1}",
        r"(\b\?)??",
        r"(?:(\B\W\B$))+",
        r"(\u{61}\b)?",
        r"((^\ba\b)){1,3}$",
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
fn unicode_repeated_atom_inner_word_counts_and_scopes() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let m=/(\ba)+/{mode}.exec('aaa');m[0]==='a'&&m[1]==='a'&&m.index===0&&/\ba+/{mode}.exec('aaa')[0]==='aaa'"
        ));
        check(&format!(
            r"let m=/(a\b)+/{mode}.exec('aaa');m[0]==='a'&&m.index===2&&/a+\b/{mode}.exec('aaa')[0]==='aaa'"
        ));
        check(&format!(
            r"/(\ba){{2}}/{mode}.exec('aa')===null&&/(a\b){{2}}/{mode}.exec('aa')===null&&/(\ba\b)+/{mode}.exec('aa')===null"
        ));
        check(&format!(
            r"let m=/(\ba\b)*/{mode}.exec('aa');m[0]===''&&m[1]===undefined"
        ));
        check(&format!(
            r"let m=/(\Ba\B)+/{mode}.exec('BaaaB');m[0]==='aaa'&&m[1]==='a'&&m.index===1"
        ));
        check(&format!(
            r"let m=/(^\B.\B$)+/{mode}ms.exec('\r\n');m[0]==='\r\n'&&m[1]==='\n'&&/(^\B.\B$){{3}}/{mode}ms.exec('\r\n')===null"
        ));
    }
}
#[test]
fn unicode_repeated_atom_inner_word_fold_surrogates_and_names() {
    check(
        r"let m=/(?<whole>(?<__proto__>\b[ſK]\B)+)/dvi.exec('ſKB');m[0]==='ſ'&&m[1]==='ſ'&&m[2]==='ſ'&&m.groups.__proto__==='ſ'&&m.indices.groups.__proto__[0]===0&&m.indices.groups.__proto__[1]===1",
    );
    check(
        r"let m=/(?<whole>(?<last>\B😀\B)+)/dvg.exec('😀😀');m[0]==='😀😀'&&m.groups.last==='😀'&&m.indices.groups.last[0]===2&&m.indices.groups.last[1]===4",
    );
    check(
        r"let r=/(\B😀\B)+/duy;r.lastIndex=1;let m=r.exec('😀😀');m.index===0&&m[0]==='😀😀'&&m.indices[1][0]===2&&r.lastIndex===4",
    );
    check(
        r"let m=/(?<x>\ba\b){0}/dv.exec('aa');m[0]===''&&m[1]===undefined&&m.groups.x===undefined&&m.indices[1]===undefined",
    );
}
#[test]
fn unicode_repeated_atom_inner_word_string_consumers() {
    check(r"let a='aa aa'.match(/(\ba)+/ug);a.length===2&&a.join('|')==='a|a'");
    check(r"'aaa'.replace(/(a\b)+/v,'<$1>')==='aa<a>'&&'aaa'.search(/(a\b)+/v)===2");
    check(r"let a='aa aa'.split(/(\ba)+/u);a.length===5&&a.join('|')==='|a|a |a|a'");
    check(
        r"let a=[...'aa aa'.matchAll(/(?<x>\ba)+/dvg)];a.length===2&&a[0].index===0&&a[1].index===3&&a[1].groups.x==='a'&&a[1].indices.groups.x[0]===3",
    );
}
#[test]
fn unicode_repeated_atom_inner_word_coercions_and_last_index() {
    check(
        r"let log='',r=/(a\b)+/vgy;r.lastIndex={valueOf(){log+='i';return 2}};let m=r.exec({toString(){log+='s';return 'aaa'}});log==='si'&&m.index===2&&r.lastIndex===3",
    );
    check(
        r"let r=/(\ba)+/ug;r.lastIndex=1;Object.defineProperty(r,'lastIndex',{writable:false});let caught=false;try{r.exec('aaa')}catch(e){caught=e instanceof TypeError}caught&&r.lastIndex===1",
    );
    check(
        r"let r=/(a\b)+/v;Object.defineProperty(r,'lastIndex',{value:100,writable:false});let m=r.exec('aaa');m.index===2&&r.lastIndex===100",
    );
}

#[test]
fn unicode_repeated_atom_inner_word_opted_work_and_unproved_units() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm.eval(r"let r=(new RegExp('(\\B\\P{NChar}\\B$){2}','umg')),marker=0,text='😀'.repeat(10000);r.lastIndex=1").unwrap();
    assert!(matches!(
        realm.eval("try{r.exec(text)}catch{marker=1}finally{marker=2}"),
        Err(Error::Limit { .. })
    ));
    assert_eq!(
        realm.eval("marker===0&&r.lastIndex===1"),
        Ok(Value::Boolean(true))
    );
    for source in [
        r"/(\b)+/u.exec('a')",
        r"/(^\B$)+/v.exec('')",
        r"/(\ba?)+/u.exec('a')",
        r"/(\ba|b)+b/v.exec('a')",
        r"/(\bab)+/u.exec('ab')",
        r"/(\ba)+b/v.exec('ab')",
        r"/\b(\ba)+b/u.exec('a')",
        r"/(\ba)+\bb/v.exec('a')",
    ] {
        assert!(
            crate::common::regexp_oracle::check(&mut Realm::default(), source),
            "{source}"
        );
    }
}

#[test]
fn unicode_repeated_atom_inner_word_former_whole_programs_snapshot() {
    let mut rows = String::new();
    {
        let source = r"/(\b.)+/u.exec('a')";
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
