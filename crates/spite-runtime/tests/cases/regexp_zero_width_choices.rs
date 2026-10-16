//! Pure empty and boundary alternatives in zero-width repetition bodies.

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
fn pure_zero_width_choice_result_snapshot() {
    let mut rows = String::new();
    for (source, flags, text) in [
        (r"(|){2}a", "d", "qa"),
        (r"(|){0}a", "d", "qa"),
        (r"(?:|){2}a", "d", "qa"),
        (r"(?:|)+", "d", "a"),
        (r"(()|()){2}a", "d", "qa"),
        (r"(()|())*a", "d", "qa"),
        (r"(()|())+?a", "d", "qa"),
        (r"(^|$){2}a", "d", "qa"),
        (r"(^|$)?a", "d", "qa"),
        (r"(^|$){1,3}?a", "dm", "q\na"),
        (r"(\b|\B)+a", "d", "qa"),
        (r"(($)|(^)){2}a", "d", "a"),
        (r"((^)|()){2}a", "d", "qa"),
        (r"(?:(^)|(\b)){2}a", "d", " a"),
        (r"(?:(^)(\B)|(\b)){2}a", "d", "a"),
        (r"((())|(())){2}a", "d", "qa"),
        (r"(?:(^)|($)){2}a\1\2", "d", "a"),
        (r"(?:(^)|()){1,4}?a", "d", "qa"),
        (r"(?:()|()){2}a\1\2", "d", "qa"),
        (r"(?:()|())*a\1\2", "d", "qa"),
        (r"(?:(?:()|()){2}){3}a", "d", "qa"),
        (r"(?:(?:()|())?){2}a", "d", "qa"),
        (r"(?:(?:()|()){2}a|b)+c", "d", "aabbc"),
        (r"(?:(?:()|())?a|b)+c", "d", "aabbc"),
        (r"(?<=(a))(?:()|()){2}b", "d", "ab"),
        (r"(?:(?<=a)|^){2}b", "d", "ab"),
        (r"(?!(?:()|()){2}q)a", "d", "a"),
        (r"(?=(?:()|()){2}a)a", "d", "a"),
        (r"(?:()|()){2}µΜ", "di", "ΜΜ"),
        (r"((?:|){2})a", "d", "qa"),
        (r"(?:(?<x>)|(?<x>)){2}a\k<x>", "d", "qa"),
        (r"(?:(?<x>)|(?<x>))*a\k<x>", "d", "qa"),
        (r"(?:()|()){2}[\uD800]b", "d", "lone surrogates"),
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
fn pure_zero_choices_keep_source_order_names_and_restore_partial_failed_arms() {
    check(
        r"/^(^|$)+$/.test('')&&/(^|$)+/.test('a')&&/(^|$)*/.test('')&&/($|^)+/.test('')&&/(\b|\B){2}/.test('')",
    );
    check(
        r"let a=/(?:(?<x>)|(?<x>)){2}a\k<x>/d.exec('qa');a.index===1&&a[1]===''&&a[2]===undefined&&a.groups.x===''&&a.indices.groups.x===a.indices[1]&&a.indices[1][0]===1",
    );
    check(
        r"let a=/(?:(?<x>)|(?<x>))*a\k<x>/d.exec('qa');a[1]===undefined&&a[2]===undefined&&a.groups.x===undefined&&a.indices.groups.x===undefined",
    );
    check(
        r"let a=/(?:(^)(\B)|(\b)){2}a/d.exec('a');a[1]===undefined&&a[2]===undefined&&a[3]===''&&a.indices[1]===undefined&&a.indices[3][0]===0",
    );
    check(
        r"let a=/((^)|()){2}a/d.exec('qa'),b=/((^)|())?a/d.exec('qa');a[1]===''&&a[2]===undefined&&a[3]===''&&b[1]===undefined&&b[3]===undefined",
    );
    check(
        r"let a=/(?!(?:()|()){2}q)a/d.exec('a');a[1]===undefined&&a[2]===undefined&&a.indices[1]===undefined",
    );
    check(
        r"let a=/(?:(?:()|()){2}){3}a/d.exec('qa'),b=/(?:(?:()|())?){2}a/d.exec('qa');a[1]===''&&a[2]===undefined&&b[1]===undefined&&b[2]===undefined",
    );
    check(
        r"let a=/(?<=(a))(?:()|()){2}b/d.exec('ab');a.index===1&&a[1]==='a'&&a.indices[1][0]===0&&a[2]===''&&a[3]===undefined",
    );
}

#[test]
fn pure_zero_choices_consumers_empty_advancement_sticky_and_callbacks_preserve_slots() {
    check(
        r"let r=/(?:(?<x>)|(?<x>)){2}a/dg,a=[...'qa a'.matchAll(r)];a.length===2&&a[1].index===3&&a[1].groups.x===''&&a[1].indices.groups.x[0]===3&&r.lastIndex===0",
    );
    check(
        r"let r=/(?:()|()){2}a/dy;r.lastIndex=1;let a=r.exec('qa');a[1]===''&&a[2]===undefined&&r.lastIndex===2&&r.exec('qa')===null&&r.lastIndex===0",
    );
    check(
        r"let r=/(?:()|())+/dg,a=[...'ab'.matchAll(r)];a.length===3&&a[2].index===2&&a[2][1]===''&&a[2][2]===undefined&&a[2].indices[1][0]===2&&r.lastIndex===0",
    );
    check(
        r"let seen=[];let s='qa a'.replace(/(?:(?<x>)|(?<x>)){2}a/g,(m,c,d,i,s,g)=>{seen.push(c,d,i,g.x);return '_'});s==='q_ _'&&seen.length===8&&seen[0]===''&&seen[1]===undefined&&seen[2]===1&&seen[3]===''&&seen[6]===3",
    );
    check(
        r"'qa a'.replace(/(?:(?<x>)|(?<x>)){2}a/g,'<$<x>>')==='q<> <>'&&'qa'.search(/(?:()|()){2}a/)===1&&'qa a'.split(/(?:()|()){2}a/).join('|')==='q||| |||'",
    );
}

#[test]
fn pure_zero_choice_huge_counts_deep_slots_copies_collection_and_parent_loops_are_unlimited() {
    let mut realm = Realm::default();
    realm.eval("let n=100000,huge='184467440737095516160000000000000000000',r=new RegExp('(?:'+'('.repeat(n)+')'.repeat(n)+'|()){'+huge+',}a','d'),copy=new RegExp(r)").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("let a=copy.exec('qa');a.index===1&&a.length===n+2&&a[1]===''&&a[n]===''&&a[n+1]===undefined&&a.indices[1][0]===1&&a.indices[n][1]===1&&a.indices[1]!==a.indices[n]&&copy.source===r.source"),Ok(Value::Boolean(true)));
    assert_eq!(realm.eval("new RegExp('(?:^|$){'+huge+',}?a').exec('a').index===0&&new RegExp('(?:^|$){0,'+huge+'}a').exec('qa').index===1"),Ok(Value::Boolean(true)));
    assert_eq!(realm.eval("let many=/(?:(?:()|()){2}a|b)+c/.exec('a'+'b'.repeat(10000)+'c');many.index===0&&many[0].length===10002&&many[1]===undefined&&many[2]===undefined"),Ok(Value::Boolean(true)));
    realm.collect(usize::MAX).unwrap();
}

#[test]
fn explicit_pure_zero_choice_work_aborts_before_last_index_and_handlers() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval("let marker=0,r=/(?:()|()){2}a/g;r.lastIndex=1;let text='b'.repeat(5000)")
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
fn mixed_empty_consuming_choices_internal_references_counted_behind_and_unicode_are_pending() {
    for source in [
        r"/(?:a|()){2,3}b/.exec('ab')",
        r"/(?:(\1)|()){2}a()/.exec('a')",
        r"/(?<=((a)|()){2})a/.exec('a')",
        r"/(?:(?:a|()){2,3})*b/.exec('ab')",
        r"/(?:()|()){2}a/u.exec('a')",
        r"/(?:()|()){2}a/v.exec('a')",
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
