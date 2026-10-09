//! Required empty scalar atoms with multiple copies retain partial capture effects.
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
fn required_empty_multi_copy_captured_atoms_result_snapshot() {
    let mut rows = String::new();
    for (source, flags, text) in [
        (r"(?<=(a)(\1\1){2})b", "d", "ab"),
        (r"(?<=(a)(\1\1){1})b", "d", "ab"),
        (r"(?<=(a)(\1\1)+)b", "d", "ab"),
        (r"(?<=(a)(\1(\1)){2})b", "d", "ab"),
        (r"(?<=(a)((\1)\1){2})b", "d", "ab"),
        (r"(?<=(a)(\1()\1){2})b", "d", "ab"),
        (r"(?<=(a)(\1\1()){2})b", "d", "ab"),
        (r"(?=(\2\2){2}(a))a", "d", "a"),
        (r"(?=(\2(\2)){2}(a))a", "d", "a"),
        (r"(?=(\3(\3)){2}(a))a", "d", "a"),
        (r"((\3\3)+(b))+", "d", "bbb"),
        (r"((\2\2)+b)+", "d", "bbb"),
        (r"(?<=((b)(\2\2)+){2})c", "d", "bbc"),
        (r"(?<=a(?=(\2\2)+(b)))b", "d", "ab"),
        (r"(?<!(a)(\1\1){2})b", "d", "b"),
        (r"(?!(\2\2){2}(a))b", "d", "b"),
        (r"(?<=(?<x>a)(?<y>\k<x>(?<z>\k<x>)){2})b", "d", "ab"),
        (r"(?=(?<x>\k<z>(?<y>\k<z>)){2}(?<z>a))a", "d", "a"),
        (r"(?<=(?<x>a)(?<y>\k<x>()\k<x>){2})b", "d", "ab"),
        (r"(?<=a(?=(?<x>\k<y>\k<y>)+(?<y>b)))b", "d", "ab"),
        (r"(a)(\1){1}", "d", "aa"),
        (r"(?<=(a)(\1\1){2})b", "di", "Ab"),
    ] {
        let source = JsString::from(source);
        let flags = JsString::from(flags);
        let input = JsString::from(text);
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
fn partial_empty_effects_names_owned_reads_and_negative_rollback_keep_ranges() {
    check(
        r"let a=/(?<=(?<x>a)(?<y>\k<x>(?<z>\k<x>)){2})b/d.exec('ab');a.groups.x==='a'&&a.groups.y===''&&a.groups.z===''&&a.indices.groups.y===a.indices[2]&&a.indices[3][0]===1",
    );
    check(
        r"let a=/(?=(?<x>\k<z>(?<y>\k<z>)){2}(?<z>a))a/d.exec('a');a.groups.x===''&&a.groups.y===''&&a.groups.z==='a'&&a.indices.groups.y[0]===0",
    );
    check(
        r"let a=/(?<=(a)(\1()\1){2})b/d.exec('ab');a[2]===''&&a[3]===''&&a.indices[2][0]===1&&a.indices[3][0]===1",
    );
    check(
        r"let a=/(?<=a(?=(\2\2)+(b)))b/d.exec('ab');a.index===1&&a[1]===''&&a[2]==='b'&&a.indices[1][0]===1",
    );
    check(
        r"let a=/(?=(\2(\2)){2}(a))a/d.exec('a');a[1]===''&&a[2]===''&&a[3]==='a'&&a.indices[2][0]===0",
    );
    check(
        r"let a=/(?<!(a)(\1\1){2})b/d.exec('b');a[1]===undefined&&a[2]===undefined&&a.indices[2]===undefined",
    );
    check(
        r"let a=/(?<=([\uD800])(\1(\1)){2})b/d.exec('\uD800b');a.index===1&&a[2]===''&&a[3]===''&&a.indices[3][0]===1",
    );
}
#[test]
fn consumers_global_sticky_empty_advancement_and_callbacks_keep_partial_slots() {
    check(
        r"let r=/(?<=(a)(\1(\1)){2})b/dy;r.lastIndex=1;let a=r.exec('ab');a[2]===''&&a[3]===''&&r.lastIndex===2&&r.exec('ab')===null&&r.lastIndex===0",
    );
    check(
        r"let r=/(?<=(a)(\1(\1)){2})b/dg,a=[...'ab ab'.matchAll(r)];a.length===2&&a[0].index===1&&a[1].index===4&&a[1].indices[3][0]===4&&r.lastIndex===0",
    );
    check(
        r"let a=[...'aa'.matchAll(/(?=(\3(\3)){2}(a))a/dg)];a.length===2&&a[0][2]===''&&a[1].indices[2][0]===1",
    );
    check(
        r"let seen=[];let s='ab ab'.replace(/(?<=(a)(\1(\1)){2})b/g,(m,x,y,z,i)=>{seen.push(x,y,z,i);return '_'});s==='a_ a_'&&seen.join('|')==='a|||1|a|||4'",
    );
    check(
        r"'ab'.search(/(?<=(a)(\1(\1)){2})b/)===1&&'ab'.split(/(?<=(a)(\1(\1)){2})b/).join('|')==='a|a|||'",
    );
}
#[test]
fn deep_partial_effects_huge_counts_clones_collection_and_long_search_stay_unlimited() {
    let mut realm = Realm::default();
    realm.eval(r"let n=100000,count='9'.repeat(100),r=new RegExp('(?<=(a)'+'('.repeat(n)+'\\1(\\1)'+')'.repeat(n)+'{'+count+'})b','d'),copy=new RegExp(r)").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("let a=copy.exec('ab');a.length===n+3&&a.index===1&&a[1]==='a'&&a[n+2]===''&&a.indices[n+2][0]===1&&copy.source===r.source"),Ok(Value::Boolean(true)));
    assert_eq!(realm.eval(r"let ahead=new RegExp('(?=(\\3(\\3)){'+count+'}(a))a','d').exec('a');ahead[1]===''&&ahead[2]===''&&ahead[3]==='a'&&ahead.indices[2][0]===0"),Ok(Value::Boolean(true)));
    assert_eq!(realm.eval(r"let long=/(?<=(a)(\1(\1)){2})b/d.exec('c'.repeat(10000)+'ab');long.index===10001&&long.indices[3][0]===10001"),Ok(Value::Boolean(true)));
    realm.collect(usize::MAX).unwrap();
}
#[test]
fn opted_in_work_abort_keeps_last_index_and_bypasses_handlers() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval(
            r"let marker=0,r=/(?<=(a)(\1(\1)){2})b/g;r.lastIndex=1;let text='c'.repeat(10000)+'ab'",
        )
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
fn consuming_partial_spans_optional_effects_and_unicode_stay_pending() {
    for source in [
        r"/(?<=a(?=(b)(\1\1){2}))b/.exec('abbbbb')",
        r"/(?<=a(?=(b)(\1(\1)){2}))b/.exec('abbb')",
        r"/(?<=a(?=(b)(\1\1){0,2}))b/.exec('abbbbb')",
        r"/(?<=(a)(\1\1){2})b/u.exec('ab')",
        r"/(?<=(a)(\1\1){2})b/v.exec('ab')",
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
