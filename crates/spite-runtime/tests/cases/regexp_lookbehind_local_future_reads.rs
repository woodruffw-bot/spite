//! Cleared local captures lying to the left of their backward reads.
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
fn lookbehind_local_future_reads_result_snapshot() {
    let mut rows = String::new();
    for (source, flags, text) in [
        (r"(?<=((a)\2){2})b", "d", "aab"),
        (r"(?<=((a)\2){2}?)b", "d", "aaaab"),
        (r"(?<=((ab)\2){2})c", "d", "ababc"),
        (r"(?<=((a)(b)\2\3){2})c", "d", "ababc"),
        (r"(?<=((a)\2\2){2})b", "d", "aab"),
        (r"(?<=(([ab])\2){2})c", "d", "abc"),
        (r"(?<=((.)\2){2})b", "ds", "a\nb"),
        (r"(?<=((a)\2\B){2})b", "d", "aab"),
        (r"(?<=((a)\2){0})b", "d", "b"),
        (r"(?<!((a)\2){2}q)b", "d", "aaxb"),
        (r"(?<=((a)\2){2}(?=(b)))b", "d", "aab"),
        (r"(b)(?<=((a)\3){2}\1)c", "d", "aabc"),
        (r"(?<=((a)\2){2})b", "di", "aAb"),
        (r"(?<=(?<y>(?<x>a)\k<x>){2})b", "d", "aab"),
        (r"(?=(((a)\3){2}))a", "d", "aaaab"),
        (r"(?<=((a)\2){2})b", "d", "ab"),
        (r"(?<=(([\uD800])\2){2})[\uDC00]", "d", "surrogates"),
    ] {
        let source = JsString::from(source);
        let flags = JsString::from(flags);
        let input = if text == "surrogates" {
            JsString::from_code_units(vec![0xd800, 0xd800, 0xdc00])
        } else {
            JsString::from(text)
        };
        let program = format!(
            "let r=new RegExp({source:?},{flags:?}),a=r.exec({input:?});JSON.stringify(a===null?{{match:null,lastIndex:r.lastIndex}}:{{matches:[...a],index:a.index,input:a.input,groups:a.groups,indices:a.indices,indicesGroups:a.indices.groups,lastIndex:r.lastIndex,source:r.source}})"
        );
        let Value::String(result) = Realm::default().eval(&program).unwrap() else {
            panic!("expected JSON")
        };
        writeln!(
            rows,
            "{source:?} flags={flags:?} input={input:?} {result:?}"
        )
        .unwrap();
    }
    insta::assert_snapshot!(rows);
}
#[test]
fn names_aliases_backward_ranges_outside_imports_and_negative_rollback() {
    check(
        r"let a=/(?<=(?<y>(?<x>a)\k<x>){2})b/d.exec('aab');a.index===2&&a.groups.x==='a'&&a.groups.y==='a'&&a.indices.groups.x===a.indices[2]&&a.indices.groups.y[0]===0&&a.indices.groups.x[1]===1",
    );
    check(
        r"let a=/(?<=(?<y>(?<x>a)(?<z>b)\k<x>\k<z>){2})c/d.exec('ababc');a.groups.y==='ab'&&a.indices.groups.x[0]===0&&a.indices.groups.z[0]===1&&a.indices.groups.z===a.indices[3]",
    );
    check(
        r"let a=/(?<!(?<y>(?<x>a)\k<x>){2}q)b/d.exec('aaxb');a.index===3&&a.groups.x===undefined&&a.groups.y===undefined&&a.indices.groups.x===undefined&&Object.hasOwn(a.groups,'x')",
    );
    check(
        r"let a=/(b)(?<=((a)\3){2}\1)c/d.exec('aabc');a.index===2&&a[1]==='b'&&a[2]==='a'&&a.indices[2][0]===0",
    );
    check(
        r"let a=/(?<=((a)\2){2}(?=(b)))b/d.exec('aab'),b=/(?=(((a)\3){2}))a/d.exec('aaaab');a.indices[3][0]===2&&b[1]==='aaaa'&&b.indices[2][0]===2&&b.indices[3][1]===3",
    );
}
#[test]
fn sticky_global_empty_advancement_and_callbacks_keep_cleared_local_effects() {
    check(
        r"let r=/(?<=((a)\2){2})b/dy;r.lastIndex=2;let a=r.exec('aab');a.index===2&&r.lastIndex===3&&r.exec('aab')===null&&r.lastIndex===0",
    );
    check(
        r"let r=/(?<=((a)\2){2})b/dg,a=[...'aab aab'.matchAll(r)];a.length===2&&a[1].index===6&&a[1].indices[2][0]===4&&r.lastIndex===0",
    );
    check(
        r"let r=/(?<=((a)\2){0})/dg,a=[...'ab'.matchAll(r)];a.length===3&&a[2].index===2&&a[2][1]===undefined&&a[2].indices[2]===undefined",
    );
    check(
        r"let seen=[];let s='aab aab'.replace(/(?<=((a)\2){2})b/g,(m,x,y,i)=>{seen.push(x,y,i);return '_'});s==='aa_ aa_'&&seen.join('|')==='a|a|2|a|a|6'",
    );
    check(
        r"'aab'.replace(/(?<=(?<y>(?<x>a)\k<x>){2})b/,'<$<y>>')==='aa<a>'&&'aab'.search(/(?<=((a)\2){2})b/)===2&&'aab'.split(/(?<=((a)\2){2})b/).join('|')==='aa|a|a|'",
    );
}
#[test]
fn deep_scopes_clones_collection_completed_negative_restore_and_large_counts_are_unlimited() {
    let mut realm = Realm::default();
    realm.eval(r"let n=100000,r=new RegExp('(?<=('+'('.repeat(n)+'a'+')'.repeat(n)+'\\2){2})b','d'),copy=new RegExp(r)").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("let a=copy.exec('aab');a.index===2&&a.length===n+2&&a[n+1]==='a'&&a.indices[n+1][0]===0&&copy.source===r.source"),Ok(Value::Boolean(true)));
    assert_eq!(realm.eval(r"let r2=new RegExp('(?<!('+'('.repeat(n)+'a'+')'.repeat(n)+'\\2){2}q)b','d'),restored=r2.exec('aaxb');restored.index===3&&restored[1]===undefined&&restored[n+1]===undefined"),Ok(Value::Boolean(true)));
    assert_eq!(realm.eval(r"let long=/(?<=((a)\2){100000})b/dy;long.lastIndex=100000;let found=long.exec('a'.repeat(100000)+'b');found.index===100000&&found.indices[1][0]===0&&found.indices[2][1]===1"),Ok(Value::Boolean(true)));
    realm.collect(usize::MAX).unwrap();
}
#[test]
fn explicit_work_aborts_preserve_last_index_and_bypass_handlers() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval(r"let marker=0,r=/(?<=((a)\2){2})b/g;r.lastIndex=1;let text='a'.repeat(5000)+'b'")
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
fn consuming_forward_open_reads_variable_counts_mixed_outside_units_and_unicode_remain_pending() {
    for source in [
        r"/(?<=(\2(a)){1,2})b/.exec('aaaab')",
        r"/(?<=((a)\1){1,2})b/.exec('aab')",
        r"/(?<=((a)\2){1,2})b/.exec('aab')",
        r"/(a)(?<=(\1(b)\3){1,2})c/.exec('ababc')",
        r"/(?<=((a)\2){2}(?=(((a)\5){1,2})))b/.exec('aaaab')",
        r"/(?<=((a)\2){2})b/u.exec('aab')",
        r"/(?<=((a)\2){2})b/v.exec('aab')",
    ] {
        assert!(
            matches!(
                Realm::default().eval(source),
                Err(Error::Unsupported { .. })
            ),
            "{source}"
        );
    }
}
