//! Skipped exact-zero native reference bodies in ordinary lookbehind.

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
fn zero_count_lookbehind_reference_result_snapshot() {
    let mut rows = String::new();
    for (source, flags, text) in [
        (r"(?<=((a)\2){0})b", "d", "qb"),
        (r"(?<=((\2)a){0})b", "d", "qb"),
        (r"(?<=((ab)\2){0})b", "d", "qb"),
        (r"(?<=((a)(\2)\3){0})b", "d", "qb"),
        (r"(?<=((a)\2){0}q)b", "d", "qb"),
        (r"(?<=q((a)\2){0})b", "d", "qb"),
        (r"(?<!((a)\2){0})b", "d", "xb"),
        (r"(?<!((a)\2){0}q)b", "d", "xb"),
        (r"(a)(?<=(?:\1){0})b", "d", "aba"),
        (r"(a)(?<=(\1){0})b\1", "d", "aba"),
        (r"(?<=(\2){0})b()", "d", "qb"),
        (r"(?<=\1{0})b()", "d", "qb"),
        (r"(?<=((a)\2){0}µ)Μ", "di", "ΜΜ"),
        (r"(?<=((.)\2){0})b", "d", "qb"),
        (r"(?<=(([a-z])\2){0})b", "d", "qb"),
        (r"(?<=((^a)\2){0})b", "d", "qb"),
        (r"(?<=((\ba)\2){0})b", "d", "qb"),
        (r"(?<=((()\3)a){0})b", "d", "qb"),
        (r"(?<=((a)\2){0}?|())b", "d", "qb"),
        (r"(?<=((a)\2){0}|())b\1\2\3", "d", "qb"),
        (r"(?<=((a)\2){0})", "d", "qb"),
        (r"(?<=((a)\2){0}[\uD800])b", "d", "lone surrogates"),
        (r"(?<=(\2\2){0})b()", "d", "qb"),
        (r"(a)(?<=((\1)){0})b\1", "d", "aba"),
        (r"(?<=((a)\2){0})b\1\2", "d", "qb"),
        (r"(?:(?<=((a)\2){0})b|a)+c", "d", "qb"),
        (r"(?<=((()\3){0}){2})b", "d", "qb"),
        (r"(?<=((a)\2){0}q|r)b", "d", "qb"),
        (r"(?<=((a)\2){0}q(?<=q))b", "d", "qb"),
        (r"(?=(?<=((a)\2){0})b)b", "d", "qb"),
        (r"((?<=((a)\3){0})){2}b", "d", "qb"),
        (r"(?:(?<=((a)\2){0})b|a)+c\1\2", "d", "qb"),
        (r"(?<=((?<x>a)\k<x>){0})b", "d", "qb"),
        (r"(?<=((\k<x>)a){0})b(?<x>)", "d", "qb"),
        (r"(?<x>a)(?<=(\k<x>){0})b\k<x>", "d", "aba"),
        (r"(?<=(?:(?<x>a)\k<x>){0}|(?<x>))b\k<x>", "d", "qb"),
        (r"(?<=(?:(?<x>a)\k<x>){0}q|(?<x>r))b\k<x>", "d", "rbr"),
        (r"(?<!((?<x>a)\k<x>){0}q)b", "d", "xb"),
        (r"(?<=((?<x>a)(?<y>\k<x>)\k<y>){0})b", "d", "qb"),
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
fn zero_count_lookbehind_bodies_keep_outside_names_and_undefined_owned_aliases() {
    check(
        r"let a=/(?<x>a)(?<=(\k<x>){0})b\k<x>/d.exec('aba');a[0]==='aba'&&a[1]==='a'&&a[2]===undefined&&a.groups.x==='a'&&a.indices.groups.x===a.indices[1]&&a.indices[1][0]===0&&a.indices[1][1]===1&&a.indices[2]===undefined",
    );
    check(
        r"let a=/(?<=((?<x>a)(?<y>\k<x>)\k<y>){0})b/d.exec('qb');a.index===1&&a[1]===undefined&&a.groups.x===undefined&&a.groups.y===undefined&&a.indices.groups.x===undefined&&a.indices.groups.y===undefined",
    );
    check(
        r"let a=/(?<=((\k<x>)a){0})b(?<x>)/d.exec('qb');a.index===1&&a[1]===undefined&&a[2]===undefined&&a.groups.x===''&&a.indices.groups.x===a.indices[3]&&a.indices[3][0]===2",
    );
    check(
        r"let a=/(?<=(?:(?<x>a)\k<x>){0}|(?<x>))b\k<x>/d.exec('qb'),b=/(?<=(?:(?<x>a)\k<x>){0}q|(?<x>r))b\k<x>/d.exec('rbr');a.groups.x===undefined&&a.indices.groups.x===undefined&&b[0]==='br'&&b[1]===undefined&&b.groups.x==='r'&&b.indices.groups.x===b.indices[2]&&b.indices[2][0]===0&&b.indices[2][1]===1",
    );
    check(
        r"let a=/(?<=((a)\2){0}q)b/d.exec('qb'),b=/(?<!((a)\2){0}q)b/d.exec('xb');a.index===1&&a[1]===undefined&&a[2]===undefined&&b.index===1&&b.indices[2]===undefined&&!/(?<!((a)\2){0})b/.test('b')",
    );
}

#[test]
fn zero_count_lookbehind_reference_consumers_keep_last_index_empty_advancement_and_callbacks() {
    check(
        r"let r=/(?<=((?<x>a)\k<x>){0})b/dg,a=[...'qb b'.matchAll(r)];a.length===2&&a[1].index===3&&a[1].groups.x===undefined&&a[1].indices.groups.x===undefined&&r.lastIndex===0",
    );
    check(
        r"let r=/(?<=((a)\2){0})b/dy;r.lastIndex=1;let a=r.exec('qb');a[1]===undefined&&a[2]===undefined&&a.index===1&&r.lastIndex===2&&r.exec('qb')===null&&r.lastIndex===0",
    );
    check(
        r"let r=/(?<=((a)\2){0})/dg,a=[...'ab'.matchAll(r)];a.length===3&&a[2].index===2&&a[2][1]===undefined&&a[2].indices[2]===undefined&&r.lastIndex===0",
    );
    check(
        r"let seen=[];let s='qb b'.replace(/(?<=((?<x>a)\k<x>){0})b/g,(m,c,d,i,s,g)=>{seen.push(c,d,i,g.x);return '_'});s==='q_ _'&&seen.length===8&&seen[0]===undefined&&seen[1]===undefined&&seen[2]===1&&seen[3]===undefined&&seen[6]===3",
    );
    check(
        r"'qb b'.replace(/(?<=((?<x>a)\k<x>){0})b/g,'<$<x>>')==='q<> <>'&&'qb'.search(/(?<=((a)\2){0})b/)===1&&'qb b'.split(/(?<=((a)\2){0})b/).join('|')==='q||| |||'",
    );
}

#[test]
fn zero_count_lookbehind_deep_skipped_slots_zero_bounds_frames_copies_and_gc_are_unlimited() {
    let mut realm = Realm::default();
    realm.eval(r"let n=100000,body='(?:'+'('.repeat(n)+'a'+')'.repeat(n)+'\\'+n+')',r=new RegExp('(?<='+body+'{0})b','d'),copy=new RegExp(r)").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("let a=copy.exec('qb');a.index===1&&a.length===n+1&&a[1]===undefined&&a[n]===undefined&&a.indices[1]===undefined&&a.indices[n]===undefined&&copy.source===r.source"),Ok(Value::Boolean(true)));
    assert_eq!(realm.eval(r"let padded=new RegExp('(?<=((a)\\2){'+'0'.repeat(10000)+'})b','d').exec('qb');padded[1]===undefined&&padded[2]===undefined&&padded.index===1"),Ok(Value::Boolean(true)));
    assert_eq!(realm.eval(r"let deep=new RegExp('(?<='+'(?:'.repeat(10000)+'((a)\\2){0}'+'){2}'.repeat(10000)+')b','d').exec('b');deep[1]===undefined&&deep[2]===undefined&&deep.indices[2]===undefined"),Ok(Value::Boolean(true)));
    assert_eq!(realm.eval(r"let many=/(?:(?<=((a)\2){0})b|a)+c/.exec('b'+'a'.repeat(10000)+'c');many.index===0&&many[0].length===10002&&many[1]===undefined&&many[2]===undefined"),Ok(Value::Boolean(true)));
    realm.collect(usize::MAX).unwrap();
}

#[test]
fn explicit_skipped_reference_work_aborts_before_last_index_and_handlers() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval(r"let marker=0,r=/(?<=((a)\2){0})b/g;r.lastIndex=1;let text='q'.repeat(5000)")
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
fn nonzero_consuming_references_outside_optional_reads_unproved_choices_and_unicode_are_pending() {
    for source in [
        r"/(?<=((a)\2){0,1})b/.exec('aab')",
        r"/(?<=((\2)a){1})b/.exec('ab')",
        r"/(?<=(\2){0,2})b()/.exec('b')",
        r"/(?<=(a|aa){0})b/.exec('b')",
        r"/(?<=((?=a)\1){0})b/.exec('b')",
        r"/(?<=((a)\2){0})b/u.exec('b')",
        r"/(?<=((a)\2){0})b/v.exec('b')",
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
