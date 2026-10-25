//! Mandatory nullable repetitions retain source order and their final iteration.
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
fn unicode_positive_exact_nullable_alternative_results_snapshot() {
    let mut rows = String::new();
    for atom in [
        r"(a|){2}",
        r"(|a){2}",
        r"(a|){2}?",
        r"(|a){2}?",
        r"(a|(?:)){0003,003}",
        r"((?:)|a){7}?",
        r"(a|){7}?",
        r"\b((?:\b(?:\ba))|){2}\B",
        r"\B(|[ſK]){3}?\b",
        r"^(.|){3}$",
        r"(😀|){2}?",
        r"(|\uD800){3}",
        r"(\p{Join_C}|){3}",
        r"\B((?:\B^(?:.$)$)|){2}\B$",
        r"(|\$){7}?",
        r"\b(|a){2}?\b",
    ] {
        let pattern = JsString::from(format!("(?<whole>{atom})").as_str());
        for flags in ["dug", "diumg", "divmg", "divy", "dsumg", "disvmy"] {
            for input in [
                JsString::from("Aaaa\nAA"),
                JsString::from("baaa$\u{200c}"),
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
fn unicode_positive_exact_nullable_alternative_mandatory_empty_and_branch_order() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let m=/(a|){{2}}/d{mode}.exec('a');m[0]==='a'&&m[1]===''&&m.indices[1][0]===1&&m.indices[1][1]===1"
        ));
        check(&format!(
            r"let m=/(|a){{2}}/{mode}.exec('a');m[0]===''&&m[1]===''"
        ));
        check(&format!(
            r"let m=/(a|){{2}}?/{mode}.exec('aaa');m[0]==='aa'&&m[1]==='a'"
        ));
        check(&format!(
            r"let m=/(|a){{2}}?/{mode}.exec('aaa');m[0]===''&&m[1]===''"
        ));
        check(&format!(
            r"let m=/(a|){{0003,003}}?/{mode}.exec('aa');m[0]==='aa'&&m[1]===''"
        ));
        check(&format!(
            r"let m=/(a|){{3}}?/{mode}.exec('baaa');m.index===0&&m[0]===''&&m[1]===''"
        ));
        check(&format!(
            r"let m=/(a|){{2}}?$/{mode}.exec('aaa');m.index===1&&m[0]==='aa'&&m[1]==='a'"
        ));
        check(&format!(
            r"let m=/\B(a|){{3}}?\B/{mode}.exec('baaa');m.index===1&&m[0]==='aa'&&m[1]===''"
        ));
        check(&format!(
            r"let m=/\b(a|){{2}}?\b/{mode}.exec('ab');m.index===0&&m[0]===''&&m[1]===''"
        ));
        check(&format!(
            r"let m=/(a|){{7}}/d{mode}.exec('');m[0]===''&&m[1]===''&&m.indices[1][0]===0"
        ));
    }
}

#[test]
fn unicode_positive_exact_nullable_alternative_names_surrogates_and_scopes() {
    check(
        r"let m=/(?<whole>(?<__proto__>a|(?:)){3})/dv.exec('aa');m.groups.whole==='aa'&&m.groups.__proto__===''&&m.indices.groups.__proto__[0]===2",
    );
    check(
        r"let r=/(?<whole>(?<last>😀|(?:(?:))){2}?)/dvy;r.lastIndex=1;let m=r.exec('😀😀😀');m.index===0&&m[0]==='😀😀'&&m.groups.last==='😀'&&m.indices.groups.last[0]===2&&m.indices.groups.last[1]===4&&r.lastIndex===4",
    );
    check(
        r"let r=/(😀|(?:)){3}/duy;r.lastIndex=1;let m=r.exec('😀');m.index===0&&m[0]==='😀'&&m[1]===''&&m.indices[1][0]===2&&r.lastIndex===2",
    );
    check(r"let m=/([ſK]|(?:)){3}/div.exec('SK');m[0]==='SK'&&m[1]===''");
    check(r"let m=/((?:.$)|(?:)){3}$/dvms.exec('\r\n');m.index===0&&m[0]==='\r\n'&&m[1]===''");
    check(r"let m=/((?:\b(?:\Ba))|){3}/du.exec('a');m[0]===''&&m[1]===''");
}

#[test]
fn unicode_positive_exact_nullable_alternative_consumers_and_coercions() {
    check(r"let a='aaa'.match(/(a|){2}?/ug);a.length===3&&a[0]==='aa'&&a[1]==='a'&&a[2]===''");
    check(r"'aaa'.replace(/(a|){2}?/vg,'<$1>')==='<a><><>'&&'ba'.search(/(a|){2}/u)===0");
    check(
        r"let a=[...'aa'.matchAll(/(?<x>(?:)|a){3}?/dvg)];a.length===3&&a[1].index===1&&a[1].groups.x===''&&a[1].indices.groups.x[0]===1",
    );
    check(r"let a='aa'.split(/(a|){3}/u);a.length===3&&a.join('|')==='||'");
    check(
        r"let log='',r=/(a|){2}?/vgy;r.lastIndex={valueOf(){log+='i';return 0}};let m=r.exec({toString(){log+='s';return 'aaa'}});log==='si'&&m[0]==='aa'&&r.lastIndex===2",
    );
    check(
        r"let r=/(a|){2}/ug;Object.defineProperty(r,'lastIndex',{writable:false});let caught=false;try{r.exec('')}catch(e){caught=e instanceof TypeError}caught&&r.lastIndex===0",
    );
}

#[test]
fn unicode_positive_exact_nullable_alternative_opt_in_work_aborts_before_mutation() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval(r"let r=/(.|(?:)){3}?$/usg,marker=0,text='😀'.repeat(10000);r.lastIndex=1")
        .unwrap();
    assert!(matches!(
        realm.eval("try{r.exec(text)}catch(e){marker=1}finally{marker=2}"),
        Err(Error::Limit { .. })
    ));
    assert_eq!(
        realm.eval("marker===0&&r.lastIndex===1"),
        Ok(Value::Boolean(true))
    );
}
