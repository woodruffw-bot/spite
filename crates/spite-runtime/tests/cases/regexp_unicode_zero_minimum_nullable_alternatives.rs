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
fn unicode_zero_minimum_nullable_alternative_results_snapshot() {
    let mut rows = String::new();
    for atom in [
        r"(a|){0}",
        r"(|a){0,0}?",
        r"(a|){0,3}",
        r"(|a){0,3}?",
        r"(a|){000,0003}",
        r"(|a){0,}",
        r"(a|){0,999999999999999999999999}?",
        r"\b((?:\b(?:\ba))|){0,3}\B",
        r"\B((?:)|[ſK]){0,3}?\b",
        r"^(.|(?:)){0,3}$",
        r"(😀|(?:(?:))){0,3}?",
        r"((?:)|\uD800){0,3}",
        r"(\p{Join_C}|){0,}",
        r"\B((?:\B^(?:.$)$)|(?:)){0,3}\B$",
        r"((?:)|\$){0,3}?",
        r"\b(|a){0,999999999999999999999999}?\b",
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
fn unicode_zero_minimum_nullable_alternative_counts_and_undefined_slots() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let m=/(a|){{0}}/d{mode}.exec('aaa');m[0]===''&&m[1]===undefined&&m.indices[1]===undefined"
        ));
        check(&format!(
            r"let m=/(|a){{0,0}}/d{mode}.exec('');m[0]===''&&m[1]===undefined"
        ));
        check(&format!(
            r"let m=/(a|){{000,0002}}/{mode}.exec('aaa');m[0]==='aa'&&m[1]==='a'"
        ));
        check(&format!(
            r"let m=/(a|){{0,3}}?/{mode}.exec('aaa');m[0]===''&&m[1]===undefined"
        ));
        check(&format!(
            r"let m=/(|a){{0,}}/{mode}.exec('aaa');m[0]==='aaa'&&m[1]==='a'"
        ));
        check(&format!(
            r"let m=/(a|){{0,999999999999999999999999}}/{mode}.exec('aaa');m[0]==='aaa'&&m[1]==='a'"
        ));
        check(&format!(
            r"let m=/(a|){{0,999999999999999999999999}}?$/{mode}.exec('aaa');m.index===0&&m[0]==='aaa'&&m[1]==='a'"
        ));
        check(&format!(
            r"let m=/\B(a|){{0,3}}?\B/{mode}.exec('baaa');m.index===1&&m[0]===''&&m[1]===undefined"
        ));
    }
}

#[test]
fn unicode_zero_minimum_nullable_alternative_names_surrogates_and_scopes() {
    check(
        r"let m=/(?<whole>(?<__proto__>a|(?:)){0})/dv.exec('aaa');m.groups.whole===''&&m.groups.__proto__===undefined&&m.indices.groups.__proto__===undefined",
    );
    check(
        r"let r=/(?<whole>(?<last>😀|(?:(?:))){0,2})/dvy;r.lastIndex=1;let m=r.exec('😀😀😀');m.index===0&&m[0]==='😀😀'&&m.groups.last==='😀'&&m.indices.groups.last[0]===2&&m.indices.groups.last[1]===4&&r.lastIndex===4",
    );
    check(
        r"let m=/((?:)|\uD800){0,3}/du.exec('\uD800x');m[0]==='\uD800'&&m[1]==='\uD800'&&/(\uD83D|){0,3}/u.exec('😀')[1]===undefined",
    );
    check(r"let m=/((?:)|[ſK]){0,2}/div.exec('SKSK');m[0]==='SK'&&m[1]==='K'");
    check(r"let m=/\B((?:\B^(?:.$)$)|(?:)){0,3}\B$/dvms.exec('\r\n');m[0]==='\r\n'&&m[1]==='\n'");
    check(r"let m=/((?:\b(?:\Ba))|){0,3}/du.exec('a');m[0]===''&&m[1]===undefined");
}

#[test]
fn unicode_zero_minimum_nullable_alternative_consumers_and_coercions() {
    check(r"let a='aaa'.match(/(a|){0,2}/ug);a.length===3&&a[0]==='aa'&&a[1]==='a'&&a[2]===''");
    check(r"'aaa'.replace(/(a|){0,2}/vg,'<$1>')==='<a><a><>'&&'ba'.search(/(a|){0,2}/u)===0");
    check(
        r"let a=[...'aa'.matchAll(/(?<x>(?:)|a){0,3}?/dvg)];a.length===3&&a[1].index===1&&a[1].groups.x===undefined&&a[1].indices.groups.x===undefined",
    );
    check(r"let a='aa'.split(/(a|){0,2}/u);a.length===3&&a[0]===''&&a[1]==='a'&&a[2]===''");
    check(
        r"let log='',r=/(a|){0,2}/vgy;r.lastIndex={valueOf(){log+='i';return 0}};let m=r.exec({toString(){log+='s';return 'aaa'}});log==='si'&&m[0]==='aa'&&r.lastIndex===2",
    );
    check(
        r"let r=/(a|){0}/ug;Object.defineProperty(r,'lastIndex',{writable:false});let caught=false;try{r.exec('')}catch(e){caught=e instanceof TypeError}caught&&r.lastIndex===0",
    );
}

#[test]
fn unicode_zero_minimum_nullable_alternative_deep_groups_and_collection() {
    assert_eq!(Limits::default().max_steps, None);
    assert_eq!(Limits::default().max_heap_entries, None);
    let mut realm = Realm::default();
    realm.eval(r"let p='('.repeat(6000)+'a|(?:)'+')'.repeat(6000)+'{0,999999999999999999999999}',r=new RegExp(p,'dvg'),c=new RegExp(r),text='a'.repeat(200001)").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval(r"let m=c.exec(text);m.length===6001&&m[6000]==='a'&&m.indices[6000][0]===200000&&m.indices[6000][1]===200001&&c.lastIndex===200001"),Ok(Value::Boolean(true)));
    realm.eval("p=null;r=null;c=null;text=null;m=null").unwrap();
    realm.collect(usize::MAX).unwrap();
    realm.eval(r"p='('.repeat(6000)+'(?:'.repeat(6000)+')'.repeat(6000)+'|a'+')'.repeat(6000)+'{000,0000}';r=new RegExp(p,'dug');c=new RegExp(r);text='a'.repeat(200001)").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval(r"m=c.exec(text);m.length===6001&&m[6000]===undefined&&m.indices[6000]===undefined&&c.lastIndex===0"),Ok(Value::Boolean(true)));
    realm.eval("p=null;r=null;c=null;text=null;m=null").unwrap();
    realm.collect(usize::MAX).unwrap();
}

#[test]
fn unicode_zero_minimum_nullable_alternative_opt_in_work_aborts_before_mutation() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval(r"let r=/(.|(?:)){0,}?$/umsg,marker=0,text='😀'.repeat(10000);r.lastIndex=1")
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
