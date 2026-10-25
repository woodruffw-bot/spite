//! RepeatMatcher preserves mandatory empty iterations and branch order.
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
fn unicode_noncapturing_empty_alternative_results_snapshot() {
    let mut rows = String::new();
    for atom in [
        r"(a|(?:))*",
        r"((?:)|a)*?",
        r"(a|(?:))+",
        r"((?:)|a)+?",
        r"(a|(?:))+?",
        r"((?:)|a)?",
        r"(a|(?:))??",
        r"\b((?:\b(?:\ba))|(?:))+\B",
        r"\B((?:)|[ſK])+?\b",
        r"^(.|(?:))+$",
        r"(😀|(?:))+?",
        r"((?:)|\uD800)+",
        r"(\p{Join_C}|(?:))*",
        r"\B((?:\B^(?:.$)$)|(?:))+\B$",
        r"((?:)|\$)+?",
        r"\b((?:)|a)+?\b",
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
fn unicode_noncapturing_empty_alternative_mandatory_empty_and_source_order() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let m=/(a|(?:))+/d{mode}.exec('');m[0]===''&&m[1]===''&&m.indices[1][0]===0&&m.indices[1][1]===0"
        ));
        check(&format!(
            r"let m=/(a|(?:))*/d{mode}.exec('');m[0]===''&&m[1]===undefined&&m.indices[1]===undefined"
        ));
        check(&format!(
            r"let m=/(a|(?:))??/d{mode}.exec('aaa');m[0]===''&&m[1]===undefined"
        ));
        check(&format!(
            r"let m=/(a|(?:))+?/{mode}.exec('baaa');m.index===0&&m[0]===''&&m[1]===''"
        ));
        check(&format!(
            r"let m=/(a|(?:))+?/{mode}.exec('aaa');m[0]==='a'&&m[1]==='a'"
        ));
        check(&format!(
            r"let m=/((?:)|a)+?/{mode}.exec('aaa');m[0]===''&&m[1]===''"
        ));
        check(&format!(
            r"let m=/((?:)|a)+/{mode}.exec('aaa');m[0]==='aaa'&&m[1]==='a'"
        ));
        check(&format!(
            r"let m=/((?:)|a)+?$/{mode}.exec('aaa');m.index===0&&m[0]==='aaa'&&m[1]==='a'"
        ));
        check(&format!(
            r"let m=/\B(a|(?:))+?\B/{mode}.exec('baaa');m.index===1&&m[0]==='a'&&m[1]==='a'"
        ));
        check(&format!(
            r"let m=/\B((?:)|a)+?\B/{mode}.exec('baaa');m.index===1&&m[0]===''&&m[1]===''"
        ));
    }
}

#[test]
fn unicode_noncapturing_empty_alternative_names_surrogates_and_scopes() {
    check(
        r"let m=/(?<whole>(?<__proto__>a|(?:))+?)/dv.exec('');m.groups.whole===''&&m.groups.__proto__===''&&m.indices.groups.__proto__[0]===0&&m.source===undefined",
    );
    check(
        r"let r=/(?<whole>(?<last>😀|(?:(?:)))+?)/dvy;r.lastIndex=1;let m=r.exec('😀😀');m.index===0&&m[0]==='😀'&&m.groups.last==='😀'&&m.indices.groups.last[0]===0&&m.indices.groups.last[1]===2&&r.lastIndex===2",
    );
    check(
        r"let m=/((?:)|\uD800)+/du.exec('\uD800x');m[0]==='\uD800'&&m[1]==='\uD800'&&/(\uD83D|(?:))+/u.exec('😀')[0]===''",
    );
    check(r"let m=/((?:)|[ſK])+/div.exec('Sk');m[0]==='Sk'&&m[1]==='k'");
    check(r"let m=/\B((?:\B^(?:.$)$)|(?:))+\B$/dvms.exec('\r\n');m[0]==='\r\n'&&m[1]==='\n'");
    check(r"let m=/((?:\b(?:\Ba))|)+/du.exec('a');m[0]===''&&m[1]===''");
}

#[test]
fn unicode_noncapturing_empty_alternative_consumers_and_coercions() {
    check(r"let a='aa'.match(/(a|(?:))+/ug);a.length===2&&a[0]==='aa'&&a[1]===''");
    check(r"'aa'.replace(/(a|(?:))+/vg,'<$1>')==='<a><>'&&'ba'.search(/(a|(?:))+?/u)===0");
    check(
        r"let a=[...'aa'.matchAll(/(?<x>(?:)|a)+?/dvg)];a.length===3&&a[1].index===1&&a[1].groups.x===''&&a[1].indices.groups.x[0]===1",
    );
    check(r"let a='aa'.split(/(a|(?:))+/u);a.length===3&&a[0]===''&&a[1]==='a'&&a[2]===''");
    check(
        r"let log='',r=/(a|(?:))+?/vgy;r.lastIndex={valueOf(){log+='i';return 0}};let m=r.exec({toString(){log+='s';return 'aaa'}});log==='si'&&m[0]==='a'&&r.lastIndex===1",
    );
    check(
        r"let r=/(a|(?:))+/ug;Object.defineProperty(r,'lastIndex',{writable:false});let caught=false;try{r.exec('')}catch(e){caught=e instanceof TypeError}caught&&r.lastIndex===0",
    );
}

#[test]
fn unicode_noncapturing_empty_alternative_opt_in_work_aborts_before_mutation() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval(r"let r=/(.|(?:))+?$/umsg,marker=0,text='😀'.repeat(10000);r.lastIndex=1")
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
