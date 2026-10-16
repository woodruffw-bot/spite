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
fn unicode_unrepresentable_finite_nullable_alternative_results_snapshot() {
    let mut rows = String::new();
    for atom in [
        r"(a|){999999999999999999999999,1000000000000000000000000}",
        r"(|a){999999999999999999999999,1000000000000000000000000}",
        r"(a|){999999999999999999999999,1000000000000000000000000}?",
        r"(|a){999999999999999999999999,1000000000000000000000000}?",
        r"(a|(?:)){000999999999999999999999999,1000000000000000000000000}",
        r"((?:)|a){999999999999999999999999,0001000000000000000000000000}?",
        r"(a|){18446744073709551616,18446744073709551617}?",
        r"\b((?:\b(?:\ba))|){999999999999999999999999,1000000000000000000000000}\B",
        r"\B(|[ſK]){999999999999999999999999,1000000000000000000000000}?\b",
        r"^(.|){999999999999999999999999,1000000000000000000000000}$",
        r"(😀|){999999999999999999999999,1000000000000000000000000}?",
        r"(|\uD800){999999999999999999999999,1000000000000000000000000}",
        r"(\p{Join_C}|){999999999999999999999999,1000000000000000000000000}",
        r"\B((?:\B^(?:.$)$)|){999999999999999999999999,1000000000000000000000000}\B$",
        r"(|\$){999999999999999999999999,1000000000000000000000000}?",
        r"\b(|a){999999999999999999999999,1000000000000000000000000}?\b",
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
fn unicode_unrepresentable_finite_nullable_alternative_mandatory_empty_and_branch_order() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let m=/(a|){{999999999999999999999999,1000000000000000000000000}}/d{mode}.exec('a');m[0]==='a'&&m[1]===''&&m.indices[1][0]===1&&m.indices[1][1]===1"
        ));
        check(&format!(
            r"let m=/(|a){{999999999999999999999999,1000000000000000000000000}}/{mode}.exec('a');m[0]==='a'&&m[1]==='a'"
        ));
        check(&format!(
            r"let m=/(a|){{999999999999999999999999,1000000000000000000000000}}?/{mode}.exec('aaa');m[0]==='aaa'&&m[1]===''"
        ));
        check(&format!(
            r"let m=/(|a){{999999999999999999999999,1000000000000000000000000}}?/{mode}.exec('aaa');m[0]===''&&m[1]===''"
        ));
        check(&format!(
            r"let m=/(a|){{000999999999999999999999999,1000000000000000000000000}}?/{mode}.exec('aa');m[0]==='aa'&&m[1]===''"
        ));
        check(&format!(
            r"let m=/(a|){{999999999999999999999999,1000000000000000000000000}}?/{mode}.exec('baaa');m.index===0&&m[0]===''&&m[1]===''"
        ));
        check(&format!(
            r"let m=/(a|){{999999999999999999999999,1000000000000000000000000}}?$/{mode}.exec('aaa');m.index===0&&m[0]==='aaa'&&m[1]===''"
        ));
        check(&format!(
            r"let m=/\B(a|){{999999999999999999999999,1000000000000000000000000}}?\B/{mode}.exec('baaa');m.index===1&&m[0]==='aa'&&m[1]===''"
        ));
        check(&format!(
            r"let m=/\b(a|){{999999999999999999999999,1000000000000000000000000}}?\b/{mode}.exec('ab');m.index===0&&m[0]===''&&m[1]===''"
        ));
        check(&format!(
            r"let m=/(a|){{999999999999999999999999,1000000000000000000000000}}/d{mode}.exec('');m[0]===''&&m[1]===''&&m.indices[1][0]===0"
        ));
    }
    check(
        r"let m=/(a|){999999999999999999999999,1000000000000000000000002}?/du.exec('aaaaaaa');m[0]==='aaaaaaa'&&m[1]===''&&m.indices[1][0]===7",
    );
    check(
        r"let m=/(|a){999999999999999999999999,1000000000000000000000002}/dv.exec('aaaaaaa');m[0]==='aaa'&&m[1]==='a'",
    );
    check(
        r"let m=/(|a){18446744073709551616,36893488147419103231}/du.exec('aaa');m[0]==='aaa'&&m[1]==='a'",
    );
    check(
        r"let m=/(|a){18446744073709551616,36893488147419103232}/dv.exec('aaa');m[0]==='aaa'&&m[1]==='a'",
    );
}

#[test]
fn unicode_unrepresentable_finite_nullable_alternative_names_surrogates_and_scopes() {
    check(
        r"let p='(a|){000999999999999999999999999,1000000000000000000000000}?',r=new RegExp(p,'dvg'),c=new RegExp(r);let m=c.exec('aaa');r.source===p&&c.source===p&&c.flags==='dgv'&&m[0]==='aaa'&&m[1]===''",
    );
    check(
        r"let m=/(?<whole>(?<__proto__>a|(?:)){999999999999999999999999,1000000000000000000000000})/dv.exec('aa');m.groups.whole==='aa'&&m.groups.__proto__===''&&m.indices.groups.__proto__[0]===2",
    );
    check(
        r"let r=/(?<whole>(?<last>😀|(?:(?:))){999999999999999999999999,1000000000000000000000000}?)/dvy;r.lastIndex=1;let m=r.exec('😀😀😀');m.index===0&&m[0]==='😀😀😀'&&m.groups.last===''&&m.indices.groups.last[0]===6&&m.indices.groups.last[1]===6&&r.lastIndex===6",
    );
    check(
        r"let r=/(😀|(?:)){999999999999999999999999,1000000000000000000000000}/duy;r.lastIndex=1;let m=r.exec('😀');m.index===0&&m[0]==='😀'&&m[1]===''&&m.indices[1][0]===2&&r.lastIndex===2",
    );
    check(
        r"let m=/([ſK]|(?:)){999999999999999999999999,1000000000000000000000000}/div.exec('SK');m[0]==='SK'&&m[1]===''",
    );
    check(
        r"let m=/((?:.$)|(?:)){999999999999999999999999,1000000000000000000000000}$/dvms.exec('\r\n');m.index===0&&m[0]==='\r\n'&&m[1]===''",
    );
    check(
        r"let m=/((?:\b(?:\Ba))|){999999999999999999999999,1000000000000000000000000}/du.exec('a');m[0]===''&&m[1]===''",
    );
}

#[test]
fn unicode_unrepresentable_finite_nullable_alternative_consumers_and_coercions() {
    check(
        r"let a='aaa'.match(/(a|){999999999999999999999999,1000000000000000000000000}?/ug);a.length===2&&a[0]==='aaa'&&a[1]===''",
    );
    check(
        r"'aaa'.replace(/(a|){999999999999999999999999,1000000000000000000000000}?/vg,'<$1>')==='<><>'&&'ba'.search(/(a|){999999999999999999999999,1000000000000000000000000}/u)===0",
    );
    check(
        r"let a=[...'aa'.matchAll(/(?<x>(?:)|a){999999999999999999999999,1000000000000000000000000}?/dvg)];a.length===3&&a[1].index===1&&a[1].groups.x===''&&a[1].indices.groups.x[0]===1",
    );
    check(
        r"let a='aa'.split(/(a|){999999999999999999999999,1000000000000000000000000}/u);a.length===3&&a.join('|')==='||'",
    );
    check(
        r"let log='',r=/(a|){999999999999999999999999,1000000000000000000000000}?/vgy;r.lastIndex={valueOf(){log+='i';return 0}};let m=r.exec({toString(){log+='s';return 'aaa'}});log==='si'&&m[0]==='aaa'&&m[1]===''&&r.lastIndex===3",
    );
    check(
        r"let r=/(a|){999999999999999999999999,1000000000000000000000000}/ug;Object.defineProperty(r,'lastIndex',{writable:false});let caught=false;try{r.exec('')}catch(e){caught=e instanceof TypeError}caught&&r.lastIndex===0",
    );
}

#[test]
fn unicode_unrepresentable_finite_nullable_alternative_deep_groups_and_collection() {
    assert_eq!(Limits::default().max_steps, None);
    assert_eq!(Limits::default().max_heap_entries, None);
    let mut realm = Realm::default();
    assert_eq!(
        realm.eval(
            r"let z=/(a|){999999999999999999999999,1000000000000000000000000}/du.exec('aaa');z[0]==='aaa'&&z[1]===''&&z.indices[1][0]===3"
        ),
        Ok(Value::Boolean(true))
    );
    assert_eq!(
        realm.eval(r"z=/(|a){999999999999999999999999,1000000000000000000000000}/dv.exec('aaa');z[0]==='a'&&z[1]==='a'"),
        Ok(Value::Boolean(true))
    );
    realm.eval(r"let p='('.repeat(6000)+'a|(?:)'+')'.repeat(6000)+'{999999999999999999999999,1000000000000000000000000}?',r=new RegExp(p,'dvg'),c=new RegExp(r),text='a'.repeat(200001)").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval(r"let m=c.exec(text);m.length===6001&&m[6000]===''&&m.indices[6000][0]===200001&&m.indices[6000][1]===200001&&c.lastIndex===200001"),Ok(Value::Boolean(true)));
    realm.eval("p=null;r=null;c=null;text=null;m=null").unwrap();
    realm.collect(usize::MAX).unwrap();
    realm.eval(r"p='('.repeat(6000)+'(?:'.repeat(6000)+')'.repeat(6000)+'|a'+')'.repeat(6000)+'{999999999999999999999999,1000000000000000000000000}';r=new RegExp(p,'dug');c=new RegExp(r);text='a'.repeat(200001)").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval(r"m=c.exec(text);m.length===6001&&m[6000]==='a'&&m.indices[6000][0]===0&&m.indices[6000][1]===1&&c.lastIndex===1"),Ok(Value::Boolean(true)));
    realm.eval("p=null;r=null;c=null;text=null;m=null").unwrap();
    realm.collect(usize::MAX).unwrap();
    realm
        .eval(r"r=/(a|){999999999999999999999999,1000000000000000000000000}?/du;text='a'.repeat(200001)")
        .unwrap();
    assert_eq!(
        realm.eval(r"m=r.exec(text);m[0].length===200001&&m[1]===''&&m.indices[1][0]===200001"),
        Ok(Value::Boolean(true))
    );
}

#[test]
fn unicode_unrepresentable_finite_nullable_alternative_opt_in_work_aborts_before_mutation() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval(r"let r=/(.|(?:)){999999999999999999999999,1000000000000000000000000}?$/usg,marker=0")
        .unwrap();
    realm
        .eval(r"let text='😀'.repeat(10000);r.lastIndex=1")
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
