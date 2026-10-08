//! Shared exact-zero native group scopes with unchanged syntax validation.

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
fn zero_count_native_scope_result_snapshot() {
    let mut rows = String::new();
    for (source, flags, text) in [
        (r"(?<=(|a){0})b", "d", "qb"),
        (r"(?<=(a|){0}?)b", "d", "qb"),
        (r"(?<=((?=a)\1){0})b", "d", "qb"),
        (r"(?<=((?!(a))\2){0})b", "d", "qb"),
        (r"(?<=((?=a)(\2)){0})b", "d", "qb"),
        (r"(?<=((?=a)|b){0})c", "d", "qc"),
        (r"(?<=((a|){0}){2})b", "d", "qb"),
        (r"(?<=((a|){0})*)b", "d", "qb"),
        (r"(?<=((a|){0}){2}q)b", "d", "qb"),
        (r"(a)(?<=((?=a)\1){0})b\1", "d", "aba"),
        (r"(?<=((?=a)|()){0})b", "d", "qb"),
        (r"((?=a)\1){0}b", "d", "qb"),
        (r"(?:(a|){0}){2}b", "d", "qb"),
        (r"(?:(a|){0})*b", "d", "qb"),
        (r"(?<=((?=a)\1){0}q)b", "d", "qb"),
        (r"(?<=q((?=a)\1){0})b", "d", "qb"),
        (r"(?<!((?=a)\1){0}q)b", "d", "xb"),
        (r"(?<!((?=a)\1){0})b", "d", "xb"),
        (r"(?<=((?=a)\1){0}|())b", "d", "qb"),
        (r"(?<=([abc]|){0})b", "d", "qb"),
        (r"(?<=((^|a)\1){0})b", "d", "qb"),
        (r"(?<=((\b|a)\1){0})b", "d", "qb"),
        (r"(?<=((a|){0}){2}µ)Μ", "di", "ΜΜ"),
        (r"(?<=((?=a)\1){0}[\uD800])b", "d", "lone surrogates"),
        (r"(?<=((?=a)\1){0})", "d", "qb"),
        (r"(?:(?<=((a|){0}){2})b|a)+c", "d", "qc"),
        (r"(?=(?<=((?=a)\1){0})b)b", "d", "qb"),
        (r"((?<=((a|){0}){2})){2}b", "d", "qb"),
        (r"((a|){0}){2}b", "d", "qb"),
        (r"((a|){0})*b", "d", "qb"),
        (r"(?<=(?:(a|){0}){0})b", "d", "qb"),
        (r"(a)(?<=(\1|){0})b\1", "d", "aba"),
        (r"(?<=(?:(?<x>)|(?<x>a)){0})b\k<x>", "d", "qb"),
        (r"(?<=((?:(?<x>)|(?<x>a)){0}){2})b\k<x>", "d", "qb"),
        (r"(?<=((?:(?<x>)|(?<x>a)){0})*)b\k<x>", "d", "qb"),
        (r"(?<=((?=a)(?<x>\k<x>)){0})b\k<x>", "d", "qb"),
        (r"(?<x>a)(?<=((?=a)\k<x>){0})b\k<x>", "d", "aba"),
        (r"(?<!((?=a)(?<x>\k<x>)){0}q)b", "d", "xb"),
        (r"((?:(?<x>)|(?<x>a)){0}){2}b\k<x>", "d", "qb"),
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
fn zero_scopes_keep_named_inventory_owned_undefined_slots_and_outside_ranges() {
    check(
        r"let a=/(?<=(?:(?<x>)|(?<x>a)){0})b\k<x>/d.exec('qb');a.groups.x===undefined&&a[1]===undefined&&a[2]===undefined&&a.indices.groups.x===undefined&&Object.hasOwn(a.groups,'x')&&Object.hasOwn(a.indices.groups,'x')",
    );
    check(
        r"let a=/(?<=((?:(?<x>)|(?<x>a)){0}){2})b\k<x>/d.exec('qb'),b=/(?<=((?:(?<x>)|(?<x>a)){0})*)b\k<x>/d.exec('qb');a[1]===''&&a[2]===undefined&&a[3]===undefined&&a.indices[1][0]===1&&a.groups.x===undefined&&b[1]===undefined&&b.indices[1]===undefined",
    );
    check(
        r"let a=/(?<x>a)(?<=((?=a)\k<x>){0})b\k<x>/d.exec('aba');a[0]==='aba'&&a.groups.x==='a'&&a[2]===undefined&&a.indices.groups.x===a.indices[1]&&a.indices[1][0]===0&&a.indices[1][1]===1",
    );
    check(
        r"let a=/(?<=((?=a)(?<x>\k<x>)){0})b\k<x>/d.exec('qb');a[1]===undefined&&a[2]===undefined&&a.groups.x===undefined&&a.indices.groups.x===undefined",
    );
    check(
        r"let a=/(?<=((a|){0}){2}q)b/d.exec('qb'),b=/(?<!((?=a)(?<x>\k<x>)){0}q)b/d.exec('xb');a[1]===''&&a[2]===undefined&&a.indices[1][0]===0&&b[1]===undefined&&b.groups.x===undefined",
    );
}

#[test]
fn zero_scope_consumers_keep_last_index_empty_advancement_and_callback_slots() {
    check(
        r"let r=/(?<=((a|){0}){2})b/dg,a=[...'qb b'.matchAll(r)];a.length===2&&a[1].index===3&&a[1][1]===''&&a[1][2]===undefined&&a[1].indices[1][0]===3&&r.lastIndex===0",
    );
    check(
        r"let r=/(?<=((?=a)\1){0})b/dy;r.lastIndex=1;let a=r.exec('qb');a[1]===undefined&&a.index===1&&r.lastIndex===2&&r.exec('qb')===null&&r.lastIndex===0",
    );
    check(
        r"let r=/(?<=((a|){0}){2})/dg,a=[...'ab'.matchAll(r)];a.length===3&&a[2].index===2&&a[2][1]===''&&a[2][2]===undefined&&a[2].indices[1][0]===2&&r.lastIndex===0",
    );
    check(
        r"let seen=[];let s='qb b'.replace(/(?<=((a|){0}){2})b/g,(m,c,d,i,s)=>{seen.push(c,d,i);return '_'});s==='q_ _'&&seen.length===6&&seen[0]===''&&seen[1]===undefined&&seen[2]===1&&seen[5]===3",
    );
    check(
        r"'qb b'.replace(/(?<=(?:(?<x>)|(?<x>a)){0})b/g,'<$<x>>')==='q<> <>'&&'qb'.search(/(?<=((?=a)\1){0})b/)===1&&'qb b'.split(/(?<=((?=a)\1){0})b/).join('|')==='q|| ||'",
    );
}

#[test]
fn zero_scope_deep_unused_regions_clones_gc_and_parent_loops_have_no_default_quotas() {
    let mut realm = Realm::default();
    realm.eval("let n=100000,r=new RegExp('(?<='+'('.repeat(n)+'(?=a)|b'+'){0}'.repeat(n)+')c','d'),copy=new RegExp(r)").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("let a=copy.exec('qc');a.index===1&&a.length===n+1&&a[1]===undefined&&a[n]===undefined&&a.indices[n]===undefined&&copy.source===r.source"),Ok(Value::Boolean(true)));
    assert_eq!(realm.eval("let flat=new RegExp('(?:'.repeat(10000)+'(?=a)|b'+'){0}'.repeat(10000)+'c').exec('qc');flat.index===1&&flat[0]==='c'&&flat.length===1"),Ok(Value::Boolean(true)));
    assert_eq!(realm.eval(r"let deep=new RegExp('(?<='+'(?:'.repeat(10000)+'((?=a)\\1){0}'+'){2}'.repeat(10000)+')b','d').exec('b');deep[1]===undefined&&deep.indices[1]===undefined"),Ok(Value::Boolean(true)));
    assert_eq!(realm.eval("let many=/(?:(?<=((a|){0}){2})b|a)+c/.exec('b'+'a'.repeat(10000)+'c');many.index===0&&many[0].length===10002&&many[1]===undefined&&many[2]===undefined"),Ok(Value::Boolean(true)));
    realm.collect(usize::MAX).unwrap();
}

#[test]
fn explicit_zero_scope_work_aborts_before_last_index_and_handlers() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval(r"let marker=0,r=/(?<=((?=a)\1){0})b/g;r.lastIndex=1;let text='q'.repeat(5000)")
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
fn syntax_validation_and_unlowered_children_nonzero_choices_and_unicode_stay_separate() {
    check(
        r"let n=0;for(let p of ['(a|{0}','(?<x>)(?<x>){0}','(?:(a){0}){2,1}']){try{new RegExp(p)}catch(e){if(e instanceof SyntaxError)n++}}n===3",
    );
    for source in [
        r"/(?<=(|a){1})b/.exec('b')",
        r"/(?<=((?=a)\1){1})b/.exec('b')",
        r"/(?<=((a|a*){1}){0})b/.exec('b')",
        r"/(?<=a+)b/.exec('ab')",
        r"/(?<=(|a){0})b/u.exec('b')",
        r"/(?<=(|a){0})b/v.exec('b')",
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
