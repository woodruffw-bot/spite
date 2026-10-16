//! Shared predicates surround the complete repeated alternative group.
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
fn unicode_repeated_atom_shared_alternative_assertions_results_snapshot() {
    let mut rows = String::new();
    for atom in [
        r"(\b(a|b)\B)+",
        r"(\B((a|[ſK]))\b){0,3}?",
        r"(\B(?:.|\n)\B){2,4}",
        r"(\b(\d|\D)\b)*",
        r"^(\B(\p{Join_C}|\p{NChar})\B)+$",
        r"(\B(😀|a)\B){1,3}$",
        r"(\B(\uD800|\uDC00)\B){0,2}?",
        r"(\b(\||a)\B)+",
        r"(\b(\ba|b\b)\B)+",
        r"(\b^(^a|b$)$\b){1,2}$",
        r"(\B([a\|]|b)\B)??",
        r"(\B(\B.|\b[aſK])\b){1,3}",
        r"(\B(a$|^b)\b)+",
        r"^(\b([a-z]|\p{Join_C})\B){0,2}$",
        r"(\b(\u{61}|\u{1f600})\B)?",
        r"(\B^(^\B.$|\ba\b)\B){1,3}$",
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
fn unicode_repeated_atom_shared_alternative_assertions_counts_and_scopes() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let m=/(\b(a|b))+/{mode}.exec('aaa');m[0]==='a'&&m[1]==='a'&&m[2]==='a'&&m.index===0&&/(a|b)+/{mode}.exec('aaa')[0]==='aaa'"
        ));
        check(&format!(
            r"let m=/((a|b)\b)+/{mode}.exec('aaa');m[0]==='a'&&m[1]==='a'&&m[2]==='a'&&m.index===2"
        ));
        check(&format!(
            r"/(\b(a|b)\b){{2}}/{mode}.exec('aa')===null&&/(\b(a|b)\b)+/{mode}.exec('aa')===null"
        ));
        check(&format!(
            r"let m=/(\B(a|b)\B)+/{mode}.exec('BabbaB');m[0]==='abba'&&m[1]==='a'&&m[2]==='a'&&m.index===1"
        ));
        check(&format!(
            r"let m=/(\b(a|b)\b)*?/{mode}.exec('aa');m[0]===''&&m[1]===undefined&&m[2]===undefined"
        ));
        check(&format!(
            r"let m=/(\B^(.|\n)$\B)+/{mode}ms.exec('\r\n');m[0]==='\r\n'&&m[1]==='\n'&&m[2]==='\n'"
        ));
    }
}
#[test]
fn unicode_repeated_atom_shared_alternative_assertions_fold_surrogates_and_names() {
    check(
        r"let m=/(?<whole>(\B(?<__proto__>ſ|K)\B)+)/dvi.exec('XSkB');m[0]==='Sk'&&m[1]==='Sk'&&m[2]==='k'&&m.groups.__proto__==='k'&&m.indices.groups.__proto__[0]===2&&m.indices.groups.__proto__[1]===3",
    );
    check(
        r"let m=/(?<whole>(\B(?<last>😀|a)\B)+)/dvg.exec('😀😀');m[0]==='😀😀'&&m.groups.last==='😀'&&m.indices.groups.last[0]===2&&m.indices.groups.last[1]===4",
    );
    check(
        r"let r=/(\B(😀|a)\B)+/duy;r.lastIndex=1;let m=r.exec('😀😀');m.index===0&&m[0]==='😀😀'&&m.indices[1][0]===2&&r.lastIndex===4",
    );
    check(
        r"let m=/(\b(?<x>a|b)\b){0}/dv.exec('ab');m[0]===''&&m[1]===undefined&&m.groups.x===undefined&&m.indices[2]===undefined",
    );
    check(
        r"let m=/(\B(\uD800|\uDC00)\B)+/du.exec('!\uD800!');m[0]==='\uD800'&&m.indices[2][0]===1&&m.indices[2][1]===2&&/(\B(\uD83D|\uDE00)\B)+/u.exec('😀')===null",
    );
}
#[test]
fn unicode_repeated_atom_shared_alternative_assertions_string_consumers() {
    check(r"let a='ab ba'.match(/(\b(a|b)\B)+/ug);a.length===2&&a.join('|')==='a|b'");
    check(
        r"'ab ba'.replace(/(\b(a|b)\B)+/vg,'<$2>')==='<a>b <b>a'&&'Xab'.search(/((a|b)\b)+/v)===2",
    );
    check(r"let a='ab ba'.split(/(\b(a|b)\B)+/u);a.length===7&&a.join('|')==='|a|a|b |b|b|a'");
    check(
        r"let a=[...'ab ba'.matchAll(/((?<x>a|b)\b)+/dvg)];a.length===2&&a[0].index===1&&a[1].index===4&&a[1].groups.x==='a'&&a[1].indices.groups.x[0]===4",
    );
}
#[test]
fn unicode_repeated_atom_shared_alternative_assertions_coercions_and_last_index() {
    check(
        r"let log='',r=/((a|b)\b)+/vgy;r.lastIndex={valueOf(){log+='i';return 2}};let m=r.exec({toString(){log+='s';return 'Xab'}});log==='si'&&m.index===2&&r.lastIndex===3",
    );
    check(
        r"let r=/((a|b)\b)+/ug;r.lastIndex=2;Object.defineProperty(r,'lastIndex',{writable:false});let caught=false;try{r.exec('Xab')}catch(e){caught=e instanceof TypeError}caught&&r.lastIndex===2",
    );
    check(
        r"let r=/((a|b)\b)+/v;Object.defineProperty(r,'lastIndex',{value:100,writable:false});let m=r.exec('Xab');m.index===2&&r.lastIndex===100",
    );
}
#[test]
fn unicode_repeated_atom_shared_alternative_assertions_large_groups_and_collection() {
    assert_eq!(Limits::default().max_steps, None);
    assert_eq!(Limits::default().max_heap_entries, None);
    let mut realm = Realm::default();
    realm.eval(r"let p='('.repeat(1500)+'\\B'+'('.repeat(4500)+'.|\\n'+')'.repeat(4500)+'\\B'+')'.repeat(1500)+'+$',r=new RegExp(p,'dmvg'),c=new RegExp(r),text='\n'.repeat(200001)").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval(r"let m=c.exec(text);m.index===0&&m.length===6001&&m[1]==='\n'&&m[6000]==='\n'&&m.indices[6000][0]===200000&&m.indices[6000][1]===200001&&c.lastIndex===200001"),Ok(Value::Boolean(true)));
    realm.eval("p=null;r=null;c=null;text=null;m=null").unwrap();
    realm.collect(usize::MAX).unwrap();
}
#[test]
fn unicode_repeated_atom_shared_alternative_assertions_opt_in_work_failure() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval(r"let r=/(\B(\P{NChar}|a)\B){2}$/umg,marker=0,text='😀'.repeat(10000);r.lastIndex=1")
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
fn unicode_repeated_atom_shared_alternative_assertions_former_whole_program_snapshot() {
    let mut rows = String::new();
    {
        let source = r"/(\b(a|b))+/v.exec('a')";
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
