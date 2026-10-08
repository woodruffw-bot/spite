//! Nested zero-width repetitions in fixed ordinary lookbehind.

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
fn fixed_lookbehind_zero_wrapper_result_snapshot() {
    let mut rows = String::new();
    for (source, flags, text) in [
        (r"(?<=(()|()){2})a", "d", "qa"),
        (r"(?<=(|)+)a", "d", "qa"),
        (r"(?<=(|){2})a", "d", "qa"),
        (r"(?<=(|){0,2})a", "d", "qa"),
        (r"(?<=(|)+?)a", "d", "qa"),
        (r"(?<=(|)*?)a", "d", "qa"),
        (r"(?<=((){2}){2})a", "d", "qa"),
        (r"(?<=((){0,2}){2})a", "d", "qa"),
        (r"(?<=((){2})*)a", "d", "qa"),
        (r"(?<=(()+)+)a", "d", "qa"),
        (r"(?<=(()*)+)a", "d", "qa"),
        (r"(?<=(()+)*)a", "d", "qa"),
        (r"(?<=((^|$){2}){2})a", "dm", "qa"),
        (r"(?<=((\b|\B){2}){2})a", "d", "qa"),
        (r"(?<=((^){0,2}){2})a", "d", "qa"),
        (r"(?<=((^){1,2}){2})a", "d", "qa"),
        (r"(?<=((a){0}){2})b", "d", "ab"),
        (r"(?<=(([a-z]){0}){2})b", "d", "ab"),
        (r"(?<=((.){0}){2})b", "d", "ab"),
        (r"(?<=(?:(()^)|()){2})a", "d", "qa"),
        (r"(?<=(?:(()$)|()){2})a", "d", "qa"),
        (r"(?<=(?:((){2}){2}|()){2})a", "d", "qa"),
        (r"(?<=(?:((){0}){2}|()){2})a", "d", "qa"),
        (r"(?<=(((){2})?){2})a", "d", "qa"),
        (r"(?<!((|){2}){2}q)a", "d", "xa"),
        (r"(?<!((^|$){2}){2})a", "dm", "xa"),
        (r"(?<=((|){2}){2}a)b", "d", "ab"),
        (r"(?<=a((|){2}){2})b", "d", "ab"),
        (r"(?<=((^|$){2}){2}a)b", "dm", "ab"),
        (r"(?<=a((\b|\B){2}){2})b", "d", "ab"),
        (r"(?<=((?<=a)){2})b", "d", "ab"),
        (r"(?<=((?<=(a))){2})b", "d", "ab"),
        (r"(?<=((?<=(a))){0,2})b", "d", "ab"),
        (r"(?<=((|){2}){2})a\1\2", "d", "qa"),
        (r"((?<=((|){2}){2})){2}a", "d", "qa"),
        (r"(?:(?<=((|){2}){2})a|b)+c", "d", "qa"),
        (r"(?<=((a){0}){2}µ)Μ", "di", "ΜΜ"),
        (r"(?<=((|){2}){2}[\uD800])b", "d", "lone surrogates"),
        (r"(?<=(?:(?<x>)|(?<x>)){2})a\k<x>", "d", "qa"),
        (r"(?<=(?:(?<x>)|(?<x>)){0,2})a\k<x>", "d", "qa"),
        (r"(?<=(?:(()^)|(?<x>)){2})a", "d", "qa"),
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
fn fixed_zero_wrappers_keep_alias_identity_optional_slots_and_failed_arm_rollback() {
    check(
        r"let a=/(?<=(?:(?<x>)|(?<x>)){2})a\k<x>/d.exec('qa'),b=/(?<=(?:(?<x>)|(?<x>)){0,2})a\k<x>/d.exec('qa');a.groups.x===''&&a[1]===''&&a[2]===undefined&&a.indices.groups.x===a.indices[1]&&a.indices[1][0]===1&&b.groups.x===undefined&&b.indices[1]===undefined&&b.indices[2]===undefined",
    );
    check(
        r"let a=/(?<=(?:(()^)|(?<x>)){2})a/d.exec('qa');a[1]===undefined&&a[2]===undefined&&a.groups.x===''&&a.indices.groups.x===a.indices[3]&&a.indices[3][0]===1",
    );
    check(
        r"let a=/(?<=((^|$){2}){2}a)b/d.exec('ab'),b=/(?<=((?<=(a))){2})b/d.exec('ab');a.index===1&&a.indices[1][0]===0&&a.indices[2][1]===0&&b[1]===''&&b[2]==='a'&&b.indices[1][0]===1&&b.indices[2][0]===0&&b.indices[2][1]===1",
    );
    check(
        r"let a=/(?<=((){0,2}){2})a/d.exec('qa'),b=/(?<=((){2})*)a/d.exec('qa');a[1]===''&&a[2]===undefined&&b[1]===undefined&&b[2]===undefined",
    );
    check(
        r"let a=/(?<!((|){2}){2}q)a/d.exec('xa');a.index===1&&a[1]===undefined&&a[2]===undefined&&a.indices[1]===undefined&&a.indices[2]===undefined",
    );
}

#[test]
fn fixed_zero_wrapper_consumers_keep_last_index_empty_advancement_and_callback_slots() {
    check(
        r"let r=/(?<=(?:(?<x>)|){2})a/dg,a=[...'qa a'.matchAll(r)];a.length===2&&a[1].index===3&&a[1].groups.x===''&&a[1].indices.groups.x[0]===3&&r.lastIndex===0",
    );
    check(
        r"let r=/(?<=(|){2})a/dy;r.lastIndex=1;let a=r.exec('qa');a[1]===''&&a.index===1&&r.lastIndex===2&&r.exec('qa')===null&&r.lastIndex===0",
    );
    check(
        r"let r=/(?<=(|){0,2})/dg,a=[...'ab'.matchAll(r)];a.length===3&&a[2].index===2&&a[2][1]===undefined&&a[2].indices[1]===undefined&&r.lastIndex===0",
    );
    check(
        r"let seen=[];let s='qa a'.replace(/(?<=(?:(?<x>)|){2})a/g,(m,c,i,s,g)=>{seen.push(c,i,g.x);return '_'});s==='q_ _'&&seen.length===6&&seen[0]===''&&seen[1]===1&&seen[2]===''&&seen[4]===3",
    );
    check(
        r"'qa a'.replace(/(?<=(?:(?<x>)|){2})a/g,'<$<x>>')==='q<> <>'&&'qa'.search(/(?<=(|){2})a/)===1&&'qa a'.split(/(?<=(|){2})a/).join('|')==='q|| ||'",
    );
}

#[test]
fn fixed_zero_wrappers_deep_captures_huge_counts_nested_frames_copies_and_gc_are_unlimited() {
    let mut realm = Realm::default();
    realm.eval("let n=100000,huge='184467440737095516160000000000000000000',body='(?:'+'('.repeat(n)+')'.repeat(n)+'|)',r=new RegExp('(?<='+body+'{'+huge+',})a','d'),copy=new RegExp(r)").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("let a=copy.exec('qa');a.index===1&&a.length===n+1&&a[1]===''&&a[n]===''&&a.indices[1][0]===1&&a.indices[n][1]===1&&a.indices[1]!==a.indices[n]&&copy.source===r.source"),Ok(Value::Boolean(true)));
    assert_eq!(realm.eval("let negative=new RegExp('(?<!'+body+'{2}q)a','d'),b=negative.exec('xa');b.index===1&&b.length===n+1&&b[1]===undefined&&b[n]===undefined&&b.indices[n]===undefined"),Ok(Value::Boolean(true)));
    assert_eq!(realm.eval("let deep=new RegExp('(?<='+'(?:'.repeat(10000)+'(|){2}'+'){2}'.repeat(10000)+')a','d').exec('a');deep[1]===''&&deep.indices[1][0]===0"),Ok(Value::Boolean(true)));
    assert_eq!(realm.eval("let many=/(?:(?<=((|){2}){2})a|b)+c/.exec('a'+'b'.repeat(10000)+'c');many.index===0&&many[0].length===10002&&many[1]===undefined&&many[2]===undefined"),Ok(Value::Boolean(true)));
    realm.collect(usize::MAX).unwrap();
}

#[test]
fn explicit_fixed_zero_wrapper_work_aborts_before_last_index_and_handlers() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval("let marker=0,r=/(?<=(|){2})a/g;r.lastIndex=1;let text='b'.repeat(5000)")
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
fn variable_consuming_wrappers_capture_reads_lookahead_and_unicode_are_pending() {
    for source in [
        r"/(?<=(|a){2})a/.exec('a')",
        r"/(?<=((a{0,2}){2}){2})a/.exec('a')",
        r"/(?<=((?=a)){2})a/.exec('a')",
        r"/(?<=(((a)\3){0,2}){2})a/.exec('a')",
        r"/(?<=(|){2})a/u.exec('a')",
        r"/(?<=(|){2})a/v.exec('a')",
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
