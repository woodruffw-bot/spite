//! Exact-count boundary predicates and capture unit offsets in lookbehind.
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
fn fixed_lookbehind_assertion_count_result_snapshot() {
    let mut rows = String::new();
    for (source, flags, text) in [
        (r"(?<=(?:^){2})a", "d", "a"),
        (r"(?<!(?:^){2})a", "d", "qa"),
        (r"(?<=(^){2})a", "d", "a"),
        (r"(?<=(^){0})a", "d", "qa"),
        (r"(?<=((^){0}))a", "d", "qa"),
        (r"(?<=(?:\b){2})a", "d", " a"),
        (r"(?<=(?:\B){2})a", "d", "ba"),
        (r"(?<=(\b){2})a", "d", " a"),
        (r"(?<=(?:$){2})", "d", "a"),
        (r"(?<=($){2})", "d", "a"),
        (r"(?<=(a\B){2})b", "d", "aab"),
        (r"(?<=((a)\B()){2})b", "d", "aab"),
        (r"(?<=((\b)a(\B)){1})b", "d", "ab"),
        (r"(?<!((a)\B){2}q)c", "d", "aaarc"),
        (r"(?<=(?:^a){1})b", "dm", "q\nab"),
        (r"(?<=(?:\ba\B){1})b", "d", " ab"),
        (r"(?<=(?:a\B){0})b", "d", "b"),
        (r"(?<=((?:a\B){0}))b", "d", "b"),
        (r"(?<=((^)){2}|((\b)){2})a", "d", " a"),
        (r"(?<=a(?<=(?:\b){2}))b", "d", "ab"),
        (r"(?:(?<=(a\B){2})b|c)+d", "d", "aabccd"),
        (r"((?<=(\b){2})){2}a\1\2", "d", "a"),
        (r"((?<=(\b){2}))*a\1\2", "d", "a"),
        (r"(?<=(µ\B){2})Μ", "di", "ΜµΜ"),
        (r"(?<=(^){2})a", "dm", "q\na"),
        (r"(?<=($){2})", "dm", "a\nb"),
        (r"(?<=((.)(?:\B)){2})b", "ds", "\n\nb"),
        (r"(?<=([\uD800]\B){2})", "d", "lone surrogates"),
    ] {
        let source = JsString::from(source);
        let flags = JsString::from(flags);
        let input = if text == "lone surrogates" {
            JsString::from_code_units(vec![0xd800, 0xd800])
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
fn counted_lookbehind_boundary_offsets_empty_captures_named_aliases_and_rollback_are_exact() {
    check(
        r"let a=/(?<=((?<x>a)\B()){2})b/d.exec('aab');a.index===2&&a[1]==='a'&&a.groups.x==='a'&&a[3]===''&&a.indices[1][0]===0&&a.indices[1][1]===1&&a.indices.groups.x===a.indices[2]&&a.indices[3][0]===1&&a.indices[3][1]===1",
    );
    check(
        r"let a=/(?<=((\b)a(\B)){1})b/d.exec('ab');a[1]==='a'&&a[2]===''&&a[3]===''&&a.indices[2][0]===0&&a.indices[2][1]===0&&a.indices[3][0]===1&&a.indices[3][1]===1",
    );
    check(
        r"let a=/(?<=(?<x>^){2})a/d.exec('a');a.groups.x===''&&a.indices.groups.x===a.indices[1]&&a.indices[1][0]===0&&a.indices[1][1]===0",
    );
    check(
        r"let a=/(?<=((?<x>^){0}))a/d.exec('qa');a.index===1&&a[1]===''&&a.groups.x===undefined&&a.indices.groups.x===undefined&&a.indices[1][0]===1&&a.indices[1][1]===1",
    );
    check(
        r"let a=/(?<!((?<x>a)\B){2}q)c/d.exec('aaarc');a.index===4&&a[1]===undefined&&a[2]===undefined&&a.groups.x===undefined&&a.indices[1]===undefined&&a.indices.groups.x===undefined",
    );
    check(
        r"let a=/(?<=((^)){2}|((\b)){2})a/d.exec(' a');a[1]===undefined&&a[2]===undefined&&a[3]===''&&a[4]===''&&a.indices[3][0]===1&&a.indices[4][0]===1",
    );
    check(
        r"let a=/(?<=(?<x>^){2}|(?<x>\b){2})a/d.exec(' a');a[1]===undefined&&a[2]===''&&a.groups.x===''&&a.indices.groups.x===a.indices[2]&&a.indices[2][0]===1",
    );
    check(
        r"let a=/((?<=(\b){2})){2}a\1\2/d.exec('a'),b=/((?<=(\b){2}))*a\1\2/d.exec('a');a[1]===''&&a[2]===''&&a.indices[1][0]===0&&a.indices[2][0]===0&&b[1]===undefined&&b[2]===undefined",
    );
}

#[test]
fn counted_lookbehind_boundary_consumers_global_sticky_empty_and_callbacks_keep_positions() {
    check(
        r"let r=/(?<=(?<x>\b){2})a/dg,a=[...'a a'.matchAll(r)];a.length===2&&a[0].index===0&&a[1].index===2&&a[1].groups.x===''&&a[1].indices.groups.x[0]===2&&r.lastIndex===0",
    );
    check(
        r"let r=/(?<=(a\B){2})b/dy;r.lastIndex=2;let a=r.exec('aab');a[1]==='a'&&a.index===2&&r.lastIndex===3&&r.exec('aab')===null&&r.lastIndex===0",
    );
    check(
        r"let r=/(?<=(\b){2})/dg,a=[...'ab cd'.matchAll(r)];a.length===4&&a[0].index===0&&a[1].index===2&&a[2].index===3&&a[3].index===5&&a[3][1]===''&&a[3].indices[1][0]===5&&r.lastIndex===0",
    );
    check(
        r"let seen=[];let s='a a'.replace(/(?<=(?<x>\b){2})a/g,(m,c,i,s,g)=>{seen.push(m,c,i,s,g.x);return '_'});s==='_ _'&&seen.length===10&&seen[0]==='a'&&seen[1]===''&&seen[2]===0&&seen[4]===''&&seen[7]===2&&seen[9]===''",
    );
    check(
        r"'a a'.replace(/(?<=(?<x>\b){2})a/g,'<$<x>>')==='<> <>'&&' a'.search(/(?<=(\b){2})a/)===1&&'a a'.split(/(?<=(\b){2})a/).join('|')==='|| ||'",
    );
}

#[test]
fn counted_lookbehind_assertion_deep_captures_huge_zero_width_counts_copies_and_gc_are_unlimited() {
    let mut realm = Realm::default();
    let maximum = usize::MAX;
    realm.eval(&format!("let n=100000,r=new RegExp('(?<='+'('.repeat(n)+'^'+')'.repeat(n)+'{{{maximum}}})a','d'),copy=new RegExp(r)")).unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("let a=copy.exec('a');a.index===0&&a.length===n+1&&a[1]===''&&a[n]===''&&a.indices[1][0]===0&&a.indices[n][1]===0&&a.indices[1]!==a.indices[n]&&copy.source===r.source"),Ok(Value::Boolean(true)));
    assert_eq!(realm.eval("let negative=new RegExp('(?<!'+'('.repeat(n)+'^'+')'.repeat(n)+'{2}q)b','d'),last=negative.exec('ab');last.length===n+1&&last[1]===undefined&&last[n]===undefined&&last.indices[1]===undefined&&last.indices[n]===undefined"),Ok(Value::Boolean(true)));
    assert_eq!(realm.eval(&format!("new RegExp('(?<=(?:^){{{maximum}}})a').exec('a').index===0&&new RegExp('(?<!((?:^)){{{maximum}}})a').exec('qa')[1]===undefined")),Ok(Value::Boolean(true)));
    assert_eq!(realm.eval("let long=/(?<=((a)\\B){10000})b/dy;long.lastIndex=10000;let lastLong=long.exec('a'.repeat(10000)+'b');lastLong.index===10000&&lastLong[1]==='a'&&lastLong[2]==='a'&&lastLong.indices[2][0]===0&&lastLong.indices[2][1]===1&&long.lastIndex===10001"),Ok(Value::Boolean(true)));
    assert_eq!(realm.eval("let many=/(?:(?<=(a\\B){2})b|c)+d/.exec('aab'+'c'.repeat(10000)+'d');many.index===2&&many[0].length===10002&&many[1]===undefined"),Ok(Value::Boolean(true)));
    realm.collect(usize::MAX).unwrap();
}

#[test]
fn explicit_counted_lookbehind_assertion_work_aborts_before_last_index_and_handlers() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval("let marker=0,r=/(?<=(a\\B){8})b/g;r.lastIndex=1;let text='a'.repeat(5000)")
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
fn variable_counts_nested_lookahead_repeated_choices_empty_and_nested_counts_remain_unsupported() {
    for source in [
        r"/(?<=(?:a\B){1,2})b/.exec('aab')",
        r"/(?<=(?:(?=a)){2})a/.exec('a')",
        r"/(?<=(a\B|b\B){2})c/.exec('abc')",
        r"/(?<=((a\B){2}){2})c/.exec('aaaac')",
        r"/(?<=((?=a)){2})a/.exec('a')",
        r"/(?<=(\b){2})a/u.exec('a')",
        r"/(?<=(\b){2})a/v.exec('a')",
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
