//! Outside word predicates compose with proved one-character alternatives.
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
fn unicode_repeated_atom_word_alternatives_results_snapshot() {
    let mut rows = String::new();
    for atom in [
        r"\b(a|b)+\B",
        r"\B((a|[ſK])){0,3}?\b",
        r"\b(?:.|\n){2,4}\b",
        r"\B(\d|\D)*",
        r"^(\p{Join_C}|\p{NChar})+\B$",
        r"\b(😀|a){1,3}\b$",
        r"(\uD800|\uDC00){0,2}?\b",
        r"\b(\||a)+",
        r"\b(\ba|b\b)+\B",
        r"(^a|b$){1,2}\b$",
        r"\B([a\|]|b)??\B",
        r"\B(\B.|\b[aſK]){1,3}\b",
        r"(a$|^b)+\b",
        r"^\b([a-z]|\p{Join_C}){0,2}\B$",
        r"(\u{61}|\u{1f600})?\b",
        r"\B(^\B.$|\ba\b){1,3}\B$",
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
fn unicode_repeated_atom_word_alternatives_counts_and_scopes() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let m=/\b(a|b)+\B/{mode}.exec('abba');m[0]==='abb'&&m[1]==='b'&&m.index===0"
        ));
        check(&format!(
            r"let m=/\B(a|b)+\b/{mode}.exec('abba');m[0]==='bba'&&m[1]==='a'&&m.index===1"
        ));
        check(&format!(
            r"let m=/\b(\ba|b\b)+\B/{mode}.exec('aaa');m[0]==='a'&&m[1]==='a'&&m.index===0"
        ));
        check(&format!(
            r"let m=/\B(^a|b$)+\b/{mode}m.exec('aa\nbb');m[0]==='b'&&m[1]==='b'&&m.index===4"
        ));
        check(&format!(
            r"let m=/\b(a|b)*?\b/{mode}.exec('ab');m[0]===''&&m[1]===undefined"
        ));
        check(&format!(
            r"let m=/\B(^\B.$|\ba\b)+\B/{mode}ms.exec('\r\n');m[0]==='\r\n'&&m[1]==='\n'"
        ));
    }
}
#[test]
fn unicode_repeated_atom_word_alternatives_fold_surrogates_and_names() {
    check(
        r"let m=/(?<whole>\B(?<__proto__>ſ|K)+\b)/dvi.exec('XSk');m[0]==='Sk'&&m[1]==='Sk'&&m[2]==='k'&&m.groups.__proto__==='k'&&m.indices.groups.__proto__[0]===2&&m.indices.groups.__proto__[1]===3",
    );
    check(
        r"let m=/(?<whole>\B(?<last>😀|a)+\B)/dvg.exec('😀😀');m[0]==='😀😀'&&m.groups.last==='😀'&&m.indices.groups.last[0]===2&&m.indices.groups.last[1]===4",
    );
    check(
        r"let r=/\B(😀|a)+\B/duy;r.lastIndex=1;let m=r.exec('😀😀');m.index===0&&m[0]==='😀😀'&&m.indices[1][0]===2&&r.lastIndex===4",
    );
    check(
        r"let m=/\b(?<x>a|b){0}\b/dv.exec('ab');m[0]===''&&m[1]===undefined&&m.groups.x===undefined&&m.indices[1]===undefined",
    );
    check(
        r"let m=/\B(\uD800|\uDC00)+/du.exec('\uD800x\uDC00');m[0]==='\uD800'&&m.indices[1][1]===1&&/\B(\uD83D|\uDE00)+/u.exec('😀')===null",
    );
}
#[test]
fn unicode_repeated_atom_word_alternatives_string_consumers() {
    check(r"let a='ab ba'.match(/\b(a|b)+\B/ug);a.length===2&&a.join('|')==='a|b'");
    check(r"'ab ba'.replace(/\b(a|b)+\B/vg,'<$1>')==='<a>b <b>a'&&'Xab'.search(/\B(a|b)+\b/v)===1");
    check(r"let a='ab ba'.split(/\b(a|b)+\B/u);a.length===5&&a.join('|')==='|a|b |b|a'");
    check(
        r"let a=[...'ab ba'.matchAll(/\B(?<x>a|b)+\b/dvg)];a.length===2&&a[0].index===1&&a[1].index===4&&a[1].groups.x==='a'&&a[1].indices.groups.x[0]===4",
    );
}
#[test]
fn unicode_repeated_atom_word_alternatives_coercions_and_last_index() {
    check(
        r"let log='',r=/\B(a|b)+\b/vgy;r.lastIndex={valueOf(){log+='i';return 1}};let m=r.exec({toString(){log+='s';return 'Xab'}});log==='si'&&m.index===1&&r.lastIndex===3",
    );
    check(
        r"let r=/\B(a|b)+\b/ug;r.lastIndex=1;Object.defineProperty(r,'lastIndex',{writable:false});let caught=false;try{r.exec('Xab')}catch(e){caught=e instanceof TypeError}caught&&r.lastIndex===1",
    );
    check(
        r"let r=/\B(a|b)+\b/v;Object.defineProperty(r,'lastIndex',{value:100,writable:false});let m=r.exec('Xab');m.index===1&&r.lastIndex===100",
    );
}
#[test]
fn unicode_repeated_atom_word_alternatives_large_groups_and_collection() {
    assert_eq!(Limits::default().max_steps, None);
    assert_eq!(Limits::default().max_heap_entries, None);
    let mut realm = Realm::default();
    realm.eval(r"let p='\\B'+'('.repeat(6000)+'.|\\n'+')'.repeat(6000)+'+\\B$',r=new RegExp(p,'dmvg'),c=new RegExp(r),text='\n'.repeat(200001)").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval(r"let m=c.exec(text);m.index===0&&m.length===6001&&m[1]==='\n'&&m[6000]==='\n'&&m.indices[6000][0]===200000&&m.indices[6000][1]===200001&&c.lastIndex===200001"),Ok(Value::Boolean(true)));
    realm.eval("p=null;r=null;c=null;text=null;m=null").unwrap();
    realm.collect(usize::MAX).unwrap();
}
#[test]
fn unicode_repeated_atom_word_alternatives_opt_in_work_failure() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval(r"let r=/\B(\P{NChar}|a){2}\B$/umg,marker=0,text='😀'.repeat(10000);r.lastIndex=1")
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
fn unicode_repeated_atom_word_alternatives_former_whole_programs_snapshot() {
    let mut rows = String::new();
    for source in [
        r"/\b(\ba|b)+/v.exec('a')",
        r"/\b(a|b)+\b/v.exec('a')",
        r"/\b(a|b)+/v.exec('a')",
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
