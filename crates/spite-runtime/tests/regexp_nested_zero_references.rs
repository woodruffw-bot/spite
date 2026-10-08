//! Nested sequence repetitions with proven empty local, open and forward references.

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
fn nested_zero_reference_result_snapshot() {
    let mut rows = String::new();
    for (source, flags, text) in [
        (r"((()\3){2}){2}a", "d", "qa"),
        (r"((()\3){0,2}){2}a", "d", "qa"),
        (r"((()\3){2})*a", "d", "qa"),
        (r"((\2){2}){2}a", "d", "qa"),
        (r"((\3()){2}){2}a", "d", "qa"),
        (r"((()\3\3){2}){2}a", "d", "qa"),
        (r"((()(\3)){2}){2}a", "d", "qa"),
        (r"((\2){0,2}){2}a", "d", "qa"),
        (r"((()\3)+)+a", "d", "qa"),
        (r"((()\3)*)+a", "d", "qa"),
        (r"((()\3)+)*a", "d", "qa"),
        (r"((()\3){1,3}?){2}a", "d", "qa"),
        (r"((^()\3){2}){2}a", "d", "a"),
        (r"((\b()\3){2}){2}a", "d", "qa"),
        (r"((\B()\3){2}){2}a", "d", "qa"),
        (r"((()^\3){0,2}){2}a", "d", "a"),
        (r"(?:(?:(()\2){2}){2}|()){2}a", "d", "qa"),
        (r"(?:(?:(()\2){0,2}){2}|()){2}a", "d", "qa"),
        (r"(?!(?:(()\2){2}){2}q)a", "d", "a"),
        (r"(?=(?:(()\2){2}){2}a)a", "d", "a"),
        (r"(?<=(a))((()\4){2}){2}b", "d", "ab"),
        (r"(?:(?:(()\2){2}){2}a|b)+c", "d", "aabbc"),
        (r"(?:(?:(()\2){2})?a|b)+c", "d", "aabbc"),
        (r"((()\3){2}){2}a\1\2\3", "d", "qa"),
        (r"((()\3){0,2}){2}a\1\2\3", "d", "qa"),
        (r"((()\3){2}){2}µΜ", "di", "ΜΜ"),
        (r"((\2\2){2}){2}a", "d", "qa"),
        (r"((\3()\3){2}){2}a", "d", "qa"),
        (r"((()\3\4()){2}){2}a", "d", "qa"),
        (r"(?:(()\2){0}){2}a", "d", "qa"),
        (r"(?:((?<x>)\k<x>){2}){2}a\k<x>", "d", "qa"),
        (r"(?:((?<x>)\k<x>){0,2}){2}a\k<x>", "d", "qa"),
        (r"(?:(\k<x>(?<x>)){2}){2}a", "d", "qa"),
        (r"(?:((?<x>\k<x>)){2}){2}a", "d", "qa"),
        (r"(?:(?:(?<x>)\k<x>){2}|(?<x>)){2}a\k<x>", "d", "qa"),
        (r"((()(\3)\4){2}){2}a", "d", "qa"),
        (r"((()(\3)(\4)\5){2}){2}a", "d", "qa"),
        (r"(?:((?<x>)(?<y>\k<x>)\k<y>){2}){2}a\k<x>\k<y>", "d", "qa"),
        (
            r"(?:(\k<y>(?<x>\k<y>)(?<y>)\k<x>){2}){2}a\k<x>\k<y>",
            "d",
            "qa",
        ),
    ] {
        let source = JsString::from(source);
        let flags = JsString::from(flags);
        let input = if text == "lone surrogates" {
            JsString::from_code_units(vec![0xd800, 0x62])
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
fn nested_empty_references_keep_named_aliases_forward_open_slots_and_negative_undo() {
    check(
        r"let a=/(?:((?<x>)(?<y>\k<x>)\k<y>){2}){2}a\k<x>\k<y>/d.exec('qa'),b=/(?:(\k<y>(?<x>\k<y>)(?<y>)\k<x>){2}){2}a\k<x>\k<y>/d.exec('qa');a[1]===''&&a.groups.x===''&&a.groups.y===''&&a.indices.groups.x===a.indices[2]&&a.indices.groups.y===a.indices[3]&&a.indices[2]!==a.indices[3]&&b.groups.x===''&&b.groups.y===''&&b.indices.groups.y[0]===1",
    );
    check(
        r"let a=/(?:((?<x>)\k<x>){2}){2}a\k<x>/d.exec('qa'),b=/(?:((?<x>)\k<x>){0,2}){2}a\k<x>/d.exec('qa');a[1]===''&&a.groups.x===''&&a.indices.groups.x===a.indices[2]&&a.indices[2][0]===1&&b[1]===undefined&&b.groups.x===undefined&&b.indices[2]===undefined",
    );
    check(
        r"let a=/(?:(\k<x>(?<x>)){2}){2}a/d.exec('qa');a[1]===''&&a.groups.x===''&&a.indices.groups.x===a.indices[2]&&a.indices[2][0]===1",
    );
    check(
        r"let a=/(?:((?<x>\k<x>)){2}){2}a/d.exec('qa');a[1]===''&&a.groups.x===''&&a.indices.groups.x===a.indices[2]&&a.index===1",
    );
    check(
        r"let a=/(?:(?:(?<x>)\k<x>){2}|(?<x>)){2}a\k<x>/d.exec('qa');a[1]===''&&a[2]===undefined&&a.groups.x===''&&a.indices.groups.x===a.indices[1]",
    );
    check(
        r"let a=/((\3()\3){2}){2}a/d.exec('qa');a[1]===''&&a[2]===''&&a[3]===''&&a.indices[1][0]===1&&a.indices[3][1]===1&&a.indices[1]!==a.indices[2]",
    );
    check(
        r"let a=/((\b()\3){2}){2}a/d.exec(' a'),b=/((()^\3){0,2}){2}a/d.exec('qa');a.index===1&&a[3]===''&&a.indices[3][0]===1&&b[1]===''&&b[2]===undefined&&b[3]===undefined",
    );
    check(
        r"let a=/(?!(?:(?:(?<x>)\k<x>){2}){2}q)a/d.exec('a');a[1]===undefined&&a.groups.x===undefined&&a.indices[1]===undefined&&a.indices.groups.x===undefined",
    );
    check(
        r"let a=/(?<=(a))((()\4){2}){2}b/d.exec('ab');a.index===1&&a[1]==='a'&&a[2]===''&&a[3]===''&&a[4]===''&&a.indices[1][0]===0&&a.indices[4][0]===1",
    );
}

#[test]
fn nested_empty_reference_consumers_keep_sticky_global_empty_advancement_and_callbacks() {
    check(
        r"let r=/(?:((?<x>)\k<x>){2}){2}a/dg,a=[...'qa a'.matchAll(r)];a.length===2&&a[1].index===3&&a[1].groups.x===''&&a[1].indices.groups.x[0]===3&&r.lastIndex===0",
    );
    check(
        r"let r=/((()\3){2}){2}a/dy;r.lastIndex=1;let a=r.exec('qa');a[3]===''&&a.index===1&&r.lastIndex===2&&r.exec('qa')===null&&r.lastIndex===0",
    );
    check(
        r"let r=/((()\3){0,2}){2}/dg,a=[...'ab'.matchAll(r)];a.length===3&&a[2].index===2&&a[2][1]===''&&a[2][3]===undefined&&a[2].indices[1][0]===2&&r.lastIndex===0",
    );
    check(
        r"let seen=[];let s='qa a'.replace(/(?:((?<x>)\k<x>){2}){2}a/g,(m,c,d,i,s,g)=>{seen.push(c,d,i,g.x);return '_'});s==='q_ _'&&seen.length===8&&seen[0]===''&&seen[1]===''&&seen[2]===1&&seen[3]===''&&seen[6]===3",
    );
    check(
        r"'qa a'.replace(/(?:((?<x>)\k<x>){2}){2}a/g,'<$<x>>')==='q<> <>'&&'qa'.search(/((()\3){2}){2}a/)===1&&'qa a'.split(/((()\3){2}){2}a/).join('|')==='q|||| ||||'",
    );
}

#[test]
fn nested_empty_references_deep_captures_huge_counts_copies_gc_and_parent_loops_are_unlimited() {
    let mut realm = Realm::default();
    realm.eval(r"let n=100000,huge='184467440737095516160000000000000000000',body='(?:'+'('.repeat(n)+')'.repeat(n)+'\\'+n+'){2}',r=new RegExp('(?:'+body+'){'+huge+',}a','d'),copy=new RegExp(r)").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("let a=copy.exec('qa');a.index===1&&a.length===n+1&&a[1]===''&&a[n]===''&&a.indices[1][0]===1&&a.indices[n][1]===1&&a.indices[1]!==a.indices[n]&&copy.source===r.source"),Ok(Value::Boolean(true)));
    assert_eq!(realm.eval("let negative=new RegExp('(?!(?:'+body+'){2}q)a','d'),b=negative.exec('a');b.length===n+1&&b[1]===undefined&&b[n]===undefined&&b.indices[n]===undefined"),Ok(Value::Boolean(true)));
    assert_eq!(
        realm.eval(r"new RegExp('(?:(()\\2){2}){'+huge+',}a').exec('qa')[2]===''"),
        Ok(Value::Boolean(true))
    );
    assert_eq!(realm.eval("let many=/(?:(?:(()\\2){2}){2}a|b)+c/.exec('a'+'b'.repeat(10000)+'c');many.index===0&&many[0].length===10002&&many[1]===undefined&&many[2]===undefined"),Ok(Value::Boolean(true)));
    assert_eq!(realm.eval(r"let deep=new RegExp('(?:'.repeat(10000)+'(()\\2){2}'+'){2}'.repeat(10000)+'a','d').exec('a');deep[1]===''&&deep[2]===''&&deep.indices[2][0]===0"),Ok(Value::Boolean(true)));
    realm.collect(usize::MAX).unwrap();
}

#[test]
fn explicit_nested_empty_reference_work_aborts_before_last_index_and_handlers() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval("let marker=0,r=/((()\\3){2}){2}a/g;r.lastIndex=1;let text='b'.repeat(5000)")
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
fn outside_references_unproved_local_spans_consuming_children_behind_and_unicode_are_pending() {
    for source in [
        r"/((\1){1,2}){2}a()/.exec('a')",
        r"/(((a)(\3)\4){0,2}){2}a/.exec('a')",
        r"/(((a)\3){0,2}){2}a/.exec('a')",
        r"/(?<=((()\3){2}){2})a/.exec('a')",
        r"/((()\3){2}){2}a/u.exec('a')",
        r"/((()\3){2}){2}a/v.exec('a')",
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
