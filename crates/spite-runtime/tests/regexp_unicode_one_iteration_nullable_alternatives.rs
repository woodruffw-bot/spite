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
fn unicode_one_iteration_nullable_alternative_results_snapshot() {
    let mut rows = String::new();
    for atom in [
        r"(a|){1}",
        r"(|a){1}",
        r"(a|){1,1}",
        r"(|a){1,1}?",
        r"(a|){0001}?",
        r"(|a){01,0001}",
        r"(a|(?:)){0001,01}?",
        r"\b((?:\b(?:\ba))|){01,0001}\B",
        r"\B(|[ſK]){0001}?\b",
        r"^(.|){01,0001}$",
        r"(😀|){0001}?",
        r"(|\uD800){01,0001}",
        r"(\p{Join_C}|){01,0001}",
        r"\B((?:\B^(?:.$)$)|){01,0001}\B$",
        r"(|\$){0001}?",
        r"\b(|a){0001}?\b",
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
fn unicode_one_iteration_nullable_alternative_mandatory_empty_and_branch_order() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let m=/(a|){{1}}/d{mode}.exec('');m[0]===''&&m[1]===''&&m.indices[1][0]===0&&m.indices[1][1]===0"
        ));
        check(&format!(
            r"let m=/(|a){{1}}/{mode}.exec('aaa');m[0]===''&&m[1]===''"
        ));
        check(&format!(
            r"let m=/(a|){{1}}?/{mode}.exec('aaa');m[0]==='a'&&m[1]==='a'"
        ));
        check(&format!(
            r"let m=/(a|){{0001,01}}/{mode}.exec('aaa');m[0]==='a'&&m[1]==='a'"
        ));
        check(&format!(
            r"let m=/(|a){{1,1}}?$/{mode}.exec('aaa');m.index===2&&m[0]==='a'&&m[1]==='a'"
        ));
        check(&format!(
            r"let m=/\B(|a){{1}}\B/{mode}.exec('baaa');m.index===1&&m[0]===''&&m[1]===''"
        ));
        check(&format!(
            r"let m=/\B(a|){{1}}\B/{mode}.exec('baaa');m.index===1&&m[0]==='a'&&m[1]==='a'"
        ));
        check(&format!(
            r"let m=/\b(a|){{1}}\b/{mode}.exec('ab');m.index===0&&m[0]===''&&m[1]===''"
        ));
    }
}

#[test]
fn unicode_one_iteration_nullable_alternative_names_surrogates_and_scopes() {
    check(
        r"let m=/(?<whole>(?<__proto__>(?:)|a){1})/dv.exec('aaa');m.groups.whole===''&&m.groups.__proto__===''&&m.indices.groups.__proto__[0]===0",
    );
    check(
        r"let r=/(?<whole>(?<last>😀|(?:(?:))){1})/dvy;r.lastIndex=1;let m=r.exec('😀😀😀');m.index===0&&m[0]==='😀'&&m.groups.last==='😀'&&m.indices.groups.last[0]===0&&m.indices.groups.last[1]===2&&r.lastIndex===2",
    );
    check(
        r"let r=/((?:)|😀){1}/duy;r.lastIndex=1;let m=r.exec('😀');m.index===0&&m[0]===''&&m[1]===''&&m.indices[1][0]===0&&r.lastIndex===0",
    );
    check(r"let m=/([ſK]|(?:)){1}/div.exec('SK');m[0]==='S'&&m[1]==='S'");
    check(r"let m=/((?:.$)|(?:)){1}$/dvms.exec('\r\n');m.index===0&&m[0]==='\r'&&m[1]==='\r'");
    check(r"let m=/((?:\b(?:\Ba))|){1}/du.exec('a');m[0]===''&&m[1]===''");
}

#[test]
fn unicode_one_iteration_nullable_alternative_consumers_and_coercions() {
    check(r"let a='aa'.match(/(a|){1}/ug);a.length===3&&a[0]==='a'&&a[1]==='a'&&a[2]===''");
    check(r"'aa'.replace(/(a|){1}/vg,'<$1>')==='<a><a><>'&&'ba'.search(/(a|){1}/u)===0");
    check(
        r"let a=[...'aa'.matchAll(/(?<x>(?:)|a){1}/dvg)];a.length===3&&a[1].index===1&&a[1].groups.x===''&&a[1].indices.groups.x[0]===1",
    );
    check(r"let a='aa'.split(/(a|){1}/u);a.length===5&&a.join('|')==='|a||a|'");
    check(
        r"let log='',r=/(a|){1}/vgy;r.lastIndex={valueOf(){log+='i';return 0}};let m=r.exec({toString(){log+='s';return 'aaa'}});log==='si'&&m[0]==='a'&&r.lastIndex===1",
    );
    check(
        r"let r=/(a|){1}/ug;Object.defineProperty(r,'lastIndex',{writable:false});let caught=false;try{r.exec('')}catch(e){caught=e instanceof TypeError}caught&&r.lastIndex===0",
    );
}

#[test]
fn unicode_one_iteration_nullable_alternative_deep_groups_and_collection() {
    assert_eq!(Limits::default().max_steps, None);
    assert_eq!(Limits::default().max_heap_entries, None);
    let mut realm = Realm::default();
    realm.eval(r"let p='('.repeat(6000)+'a|(?:)'+')'.repeat(6000)+'{01,0001}',r=new RegExp(p,'dvg'),c=new RegExp(r),text='a'.repeat(200001)").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval(r"let m=c.exec(text);m.length===6001&&m[6000]==='a'&&m.indices[6000][0]===0&&m.indices[6000][1]===1&&c.lastIndex===1"),Ok(Value::Boolean(true)));
    realm.eval("p=null;r=null;c=null;text=null;m=null").unwrap();
    realm.collect(usize::MAX).unwrap();
    realm.eval(r"p='('.repeat(6000)+'(?:'.repeat(6000)+')'.repeat(6000)+'|a'+')'.repeat(6000)+'{1}?';r=new RegExp(p,'dug');c=new RegExp(r);text='a'.repeat(200001)").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval(r"m=c.exec(text);m.length===6001&&m[6000]===''&&m.indices[6000][0]===0&&m.indices[6000][1]===0&&c.lastIndex===0"),Ok(Value::Boolean(true)));
    realm.eval("p=null;r=null;c=null;text=null;m=null").unwrap();
    realm.collect(usize::MAX).unwrap();
}

#[test]
fn unicode_one_iteration_nullable_alternative_opt_in_work_aborts_before_mutation() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval(r"let r=/(.|(?:)){1}$/usg,marker=0,text='😀'.repeat(10000);r.lastIndex=1")
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
