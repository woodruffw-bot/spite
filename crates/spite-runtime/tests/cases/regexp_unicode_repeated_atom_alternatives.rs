//! Complete alternatives consume one Unicode character per selected iteration.
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
fn unicode_repeated_atom_alternatives_results_snapshot() {
    let mut rows = String::new();
    for atom in [
        r"(a|b)+",
        r"((a|[ſK])){0,3}?",
        r"(?:.|\n){2,4}",
        r"(\d|\D)*",
        r"^(\p{Join_C}|\p{NChar})+$",
        r"(😀|a){1,3}$",
        r"(\uD800|\uDC00){0,2}?",
        r"(\||a)+",
        r"(\ba|b\b)+",
        r"(^a|b$){1,2}$",
        r"([a\|]|b)??",
        r"(\B.|\b[aſK]){1,3}",
        r"(a$|^b)+",
        r"^([a-z]|\p{Join_C}){0,2}$",
        r"(\u{61}|\u{1f600})?",
        r"(^\B.$|\ba\b){1,3}$",
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
fn unicode_repeated_atom_alternatives_counts_and_branch_scopes() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let m=/(a|b)+/{mode}.exec('abba');m[0]==='abba'&&m[1]==='a'&&m.index===0"
        ));
        check(&format!(
            r"let m=/(\ba|b\b)+/{mode}.exec('aaa bbb');m[0]==='a'&&m[1]==='a'&&/(\ba|b\b){{2}}/{mode}.exec('aaa bbb')===null"
        ));
        check(&format!(
            r"let m=/(^a|b$)+/{mode}m.exec('aa\nbb');m[0]==='a'&&m[1]==='a'&&m.index===0"
        ));
        check(&format!(
            r"let m=/((a|b))*?/{mode}.exec('ab');m[0]===''&&m[1]===undefined&&m[2]===undefined"
        ));
        check(&format!(
            r"let m=/(a|b){{2,3}}/{mode}.exec('abba');m[0]==='abb'&&m[1]==='b'&&/(a|b){{2,3}}?/{mode}.exec('abba')[0]==='ab'"
        ));
        check(&format!(
            r"let m=/([a\|]|b)+/{mode}.exec('a|b');m[0]==='a|b'&&m[1]==='b'"
        ));
        check(&format!(
            r"let m=/(\||a)+/{mode}.exec('|aa');m[0]==='|aa'&&m[1]==='a'"
        ));
        check(&format!(
            r"let m=/(^\B.$|\ba\b)+/{mode}ms.exec('\r\n');m[0]==='\r\n'&&m[1]==='\n'"
        ));
    }
}
#[test]
fn unicode_repeated_atom_alternatives_fold_surrogates_and_names() {
    check(
        r"let m=/(?<whole>(?<__proto__>ſ|K)+)/dvi.exec('Sk');m[0]==='Sk'&&m[1]==='Sk'&&m[2]==='k'&&m.groups.__proto__==='k'&&m.indices.groups.__proto__[0]===1&&m.indices.groups.__proto__[1]===2",
    );
    check(
        r"let m=/(?<whole>(?<last>😀|a)+)/dvg.exec('😀a😀');m[0]==='😀a😀'&&m.groups.last==='😀'&&m.indices.groups.last[0]===3&&m.indices.groups.last[1]===5",
    );
    check(
        r"let r=/(😀|a)+/duy;r.lastIndex=1;let m=r.exec('😀a');m.index===0&&m[0]==='😀a'&&m.indices[1][0]===2&&r.lastIndex===3",
    );
    check(
        r"let m=/(?<x>a|b){0}/dv.exec('ab');m[0]===''&&m[1]===undefined&&m.groups.x===undefined&&m.indices[1]===undefined",
    );
    check(
        r"let m=/(\uD800|\uDC00)+/du.exec('\uD800x\uDC00');m[0]==='\uD800'&&m.indices[1][1]===1&&/(\uD83D|\uDE00)+/u.exec('😀')===null",
    );
}
#[test]
fn unicode_repeated_atom_alternatives_string_consumers() {
    check(r"let a='ab ba'.match(/(a|b)+/ug);a.length===2&&a.join('|')==='ab|ba'");
    check(r"'ab ba'.replace(/(a|b)+/vg,'<$1>')==='<b> <a>'&&'Xab'.search(/(a|b)+/v)===1");
    check(r"let a='ab ba'.split(/(a|b)+/u);a.length===5&&a.join('|')==='|b| |a|'");
    check(
        r"let a=[...'ab ba'.matchAll(/(?<x>a|b)+/dvg)];a.length===2&&a[0].index===0&&a[1].index===3&&a[1].groups.x==='a'&&a[1].indices.groups.x[0]===4",
    );
}
#[test]
fn unicode_repeated_atom_alternatives_coercions_and_last_index() {
    check(
        r"let log='',r=/(a|b)+/vgy;r.lastIndex={valueOf(){log+='i';return 1}};let m=r.exec({toString(){log+='s';return 'Xab'}});log==='si'&&m.index===1&&r.lastIndex===3",
    );
    check(
        r"let r=/(a|b)+/ug;r.lastIndex=1;Object.defineProperty(r,'lastIndex',{writable:false});let caught=false;try{r.exec('Xab')}catch(e){caught=e instanceof TypeError}caught&&r.lastIndex===1",
    );
    check(
        r"let r=/(a|b)+/v;Object.defineProperty(r,'lastIndex',{value:100,writable:false});let m=r.exec('Xab');m.index===1&&r.lastIndex===100",
    );
}

#[test]
fn unicode_repeated_atom_alternatives_opt_in_work_failure() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval(r"let r=/(\P{NChar}|a){2}$/umg,marker=0,text='😀'.repeat(10000);r.lastIndex=1")
        .unwrap();
    assert!(matches!(
        realm.eval("try{r.exec(text)}catch{marker=1}finally{marker=2}"),
        Err(Error::Limit { .. })
    ));
    assert_eq!(
        realm.eval("marker===0&&r.lastIndex===1"),
        Ok(Value::Boolean(true))
    );
}

#[test]
fn unicode_repeated_atom_alternatives_former_whole_programs_snapshot() {
    let mut rows = String::new();
    for source in [
        r"/(a|b)+/u.test('ab')",
        r"/(a|b)+/v.test('ab')",
        r"/(a|b)+/v.exec('aa')",
        r"/(a|b)+/v.exec('a')",
        r"/(\ba|b)+/v.exec('a')",
        r"/((a|b))+/u.test('ab')",
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
