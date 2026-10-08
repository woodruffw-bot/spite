//! Exact-zero progressing choices in fixed lookbehind and zero-width wrappers.

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
fn zero_count_lookbehind_choice_result_snapshot() {
    let mut rows = String::new();
    for (source, flags, text) in [
        (r"(?<=(a|aa){0})b", "d", "qb"),
        (r"(?<=(a|aa){0}?)b", "d", "qb"),
        (r"(?<=((a)|(bb)){0})c", "d", "qc"),
        (r"(?<=(a|bb|ccc){0})b", "d", "qb"),
        (r"(?<=(a|aa){0}q)b", "d", "qb"),
        (r"(?<=q(a|aa){0})b", "d", "qb"),
        (r"(?<!((a|aa){0}){2}q)b", "d", "xb"),
        (r"(?<!(a|aa){0})b", "d", "xb"),
        (r"(?<=((a|aa){0}))b", "d", "qb"),
        (r"(?<=((a|aa){0}){2})b", "d", "qb"),
        (r"(?<=((a|aa){0}){0,2})b", "d", "qb"),
        (r"(?<=((a|aa){0})+)b", "d", "qb"),
        (r"(?<=((a|aa){0})*)b", "d", "qb"),
        (r"(?<=((a|aa){0}){2}q)b", "d", "qb"),
        (r"(?<=(a|aa){0}q|r)b", "d", "qb"),
        (r"(?<=(a|aa){0}q(?<=q))b", "d", "qb"),
        (r"(?<=((a)\2|b){0})c", "d", "qc"),
        (r"(?<=(a*?b|cc){0})b", "d", "qb"),
        (r"(?<=((?=a)a|b){0})c", "d", "qc"),
        (r"(?<=((?<=(a))a|b){0})c", "d", "qc"),
        (r"(?<=(a|aa){0})b\1", "d", "qb"),
        (r"(?<=((a|aa){0}){2})b\1\2", "d", "qb"),
        (r"(?<=((a|aa){0}){2}µ)Μ", "di", "ΜΜ"),
        (r"(?<=(a|aa){0}[\uD800])b", "d", "lone surrogates"),
        (r"(?<=(a|aa){0})", "d", "qb"),
        (r"(?:(?<=((a|aa){0}){2})b|a)+c", "d", "qc"),
        (r"(?=(?<=(a|aa){0})b)b", "d", "qb"),
        (r"((?<=((a|aa){0}){2})){2}b", "d", "qb"),
        (r"((a|aa){0}){2}b", "d", "qb"),
        (r"((a|aa){0})*b", "d", "qb"),
        (r"(?:(a|aa){0}){2}b", "d", "qb"),
        (r"(a)(?<=(a\1|bb){0})b\1", "d", "aba"),
        (r"(?<=(?:(?<x>a)|(?<x>bb)){0})b\k<x>", "d", "qb"),
        (r"(?<=((?:(?<x>a)|(?<x>bb)){0}){2})b\k<x>", "d", "qb"),
        (r"(?<=((?:(?<x>a)|(?<x>bb)){0})*)b\k<x>", "d", "qb"),
        (r"(?<=(?:(?<x>a)\k<x>|(?<x>bb)\k<x>){0})b\k<x>", "d", "qb"),
        (r"(?<x>a)(?<=(a\k<x>|bb){0})b\k<x>", "d", "aba"),
        (r"(?<!((?:(?<x>a)|(?<x>bb)){0}){2}q)b", "d", "xb"),
        (r"((?:(?<x>a)|(?<x>bb)){0}){2}b\k<x>", "d", "qb"),
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
fn skipped_choices_keep_named_aliases_enclosing_ranges_outside_targets_and_negative_undo() {
    check(
        r"let a=/(?<=(?:(?<x>a)|(?<x>bb)){0})b\k<x>/d.exec('qb');a.groups.x===undefined&&a[1]===undefined&&a[2]===undefined&&a.indices.groups.x===undefined&&Object.hasOwn(a.groups,'x')&&Object.hasOwn(a.indices.groups,'x')",
    );
    check(
        r"let a=/(?<=((?:(?<x>a)|(?<x>bb)){0}){2})b\k<x>/d.exec('qb'),b=/(?<=((?:(?<x>a)|(?<x>bb)){0})*)b\k<x>/d.exec('qb');a[1]===''&&a[2]===undefined&&a[3]===undefined&&a.indices[1][0]===1&&a.groups.x===undefined&&b[1]===undefined&&b.indices[1]===undefined",
    );
    check(
        r"let a=/(?<x>a)(?<=(a\k<x>|bb){0})b\k<x>/d.exec('aba');a[0]==='aba'&&a.groups.x==='a'&&a[2]===undefined&&a.indices.groups.x===a.indices[1]&&a.indices[1][0]===0&&a.indices[1][1]===1",
    );
    check(
        r"let a=/(?<=((a|aa){0}){2}q)b/d.exec('qb'),b=/(?<!((a|aa){0}){2}q)b/d.exec('xb');a[1]===''&&a[2]===undefined&&a.indices[1][0]===0&&b[1]===undefined&&b[2]===undefined",
    );
    check(
        r"let a=/((a|aa){0}){2}b/d.exec('qb'),b=/((a|aa){0})*b/d.exec('qb');a.index===1&&a[1]===''&&a[2]===undefined&&b[1]===undefined&&b.indices[2]===undefined",
    );
}

#[test]
fn zero_count_choice_consumers_keep_last_index_empty_advancement_and_callback_slots() {
    check(
        r"let r=/(?<=((a|aa){0}){2})b/dg,a=[...'qb b'.matchAll(r)];a.length===2&&a[1].index===3&&a[1][1]===''&&a[1][2]===undefined&&a[1].indices[1][0]===3&&r.lastIndex===0",
    );
    check(
        r"let r=/(?<=(a|aa){0})b/dy;r.lastIndex=1;let a=r.exec('qb');a[1]===undefined&&a.index===1&&r.lastIndex===2&&r.exec('qb')===null&&r.lastIndex===0",
    );
    check(
        r"let r=/(?<=((a|aa){0}){2})/dg,a=[...'ab'.matchAll(r)];a.length===3&&a[2].index===2&&a[2][1]===''&&a[2][2]===undefined&&a[2].indices[1][0]===2&&r.lastIndex===0",
    );
    check(
        r"let seen=[];let s='qb b'.replace(/(?<=((a|aa){0}){2})b/g,(m,c,d,i,s)=>{seen.push(c,d,i);return '_'});s==='q_ _'&&seen.length===6&&seen[0]===''&&seen[1]===undefined&&seen[2]===1&&seen[5]===3",
    );
    check(
        r"'qb b'.replace(/(?<=(?:(?<x>a)|(?<x>bb)){0})b/g,'<$<x>>')==='q<> <>'&&'qb'.search(/(?<=(a|aa){0})b/)===1&&'qb b'.split(/(?<=(a|aa){0})b/).join('|')==='q|| ||'",
    );
}

#[test]
fn zero_count_choices_deep_skipped_and_completed_captures_flat_frames_copies_and_gc_are_unlimited()
{
    let mut realm = Realm::default();
    realm.eval("let n=100000,body='(?:'+'('.repeat(n)+'a'+')'.repeat(n)+'|bb)',r=new RegExp('(?<='+body+'{0})c','d'),copy=new RegExp(r)").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("let a=copy.exec('qc');a.index===1&&a.length===n+1&&a[1]===undefined&&a[n]===undefined&&a.indices[n]===undefined&&copy.source===r.source"),Ok(Value::Boolean(true)));
    assert_eq!(realm.eval("let completed='(?:'+'('.repeat(n)+'(?:a|bb){0}'+')'.repeat(n)+')',positive=new RegExp('(?<='+completed+'{2})b','d'),p=positive.exec('qb'),negative=new RegExp('(?<!'+completed+'{2}q)b','d'),z=negative.exec('xb');p.length===n+1&&p[1]===''&&p[n]===''&&p.indices[1][0]===1&&p.indices[n][1]===1&&p.indices[1]!==p.indices[n]&&z.length===n+1&&z[1]===undefined&&z[n]===undefined&&z.indices[n]===undefined"),Ok(Value::Boolean(true)));
    assert_eq!(realm.eval("let deep=new RegExp('(?<='+'(?:'.repeat(10000)+'(a|aa){0}'+'){2}'.repeat(10000)+')b','d').exec('b');deep[1]===undefined&&deep.indices[1]===undefined"),Ok(Value::Boolean(true)));
    assert_eq!(realm.eval("let padded=new RegExp('(?<=(a|aa){'+'0'.repeat(10000)+'})b','d').exec('qb');padded[1]===undefined&&padded.index===1"),Ok(Value::Boolean(true)));
    assert_eq!(realm.eval("let many=/(?:(?<=((a|aa){0}){2})b|a)+c/.exec('b'+'a'.repeat(10000)+'c');many.index===0&&many[0].length===10002&&many[1]===undefined&&many[2]===undefined"),Ok(Value::Boolean(true)));
    realm.collect(usize::MAX).unwrap();
}

#[test]
fn explicit_skipped_choice_work_aborts_before_last_index_and_handlers() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval("let marker=0,r=/(?<=(a|aa){0})b/g;r.lastIndex=1;let text='q'.repeat(5000)")
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
fn nonzero_unequal_choices_mixed_empty_paths_unproved_bodies_and_unicode_are_pending() {
    for source in [
        r"/(?<=(a|aa){1})b/.exec('ab')",
        r"/(?<=(a|aa){0,1})b/.exec('ab')",
        r"/(?<=(|a){0})b/.exec('b')",
        r"/(?<=((?=a)\1){0})b/.exec('b')",
        r"/(?<=(a|aa){0})b/u.exec('b')",
        r"/(?<=(a|aa){0})b/v.exec('b')",
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
