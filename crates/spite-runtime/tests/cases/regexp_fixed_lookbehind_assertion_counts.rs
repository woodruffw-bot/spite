//! Exact-count boundary predicates and capture unit offsets in lookbehind.
use spite_core::JsString;
use spite_runtime::{Error, Limits, Realm, Value};
use std::fmt::Write;

use crate::common::assert_true;

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
    insta::assert_snapshot!(rows, @r#"
        "(?<=(?:^){2})a" flags="d" input="a" "{\u0022matches\u0022:[\u0022a\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022a\u0022,\u0022indices\u0022:[[0,1]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=(?:^){2})a\u0022}"
        "(?<!(?:^){2})a" flags="d" input="qa" "{\u0022matches\u0022:[\u0022a\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022qa\u0022,\u0022indices\u0022:[[1,2]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<!(?:^){2})a\u0022}"
        "(?<=(^){2})a" flags="d" input="a" "{\u0022matches\u0022:[\u0022a\u0022,\u0022\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022a\u0022,\u0022indices\u0022:[[0,1],[0,0]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=(^){2})a\u0022}"
        "(?<=(^){0})a" flags="d" input="qa" "{\u0022matches\u0022:[\u0022a\u0022,null],\u0022index\u0022:1,\u0022input\u0022:\u0022qa\u0022,\u0022indices\u0022:[[1,2],null],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=(^){0})a\u0022}"
        "(?<=((^){0}))a" flags="d" input="qa" "{\u0022matches\u0022:[\u0022a\u0022,\u0022\u0022,null],\u0022index\u0022:1,\u0022input\u0022:\u0022qa\u0022,\u0022indices\u0022:[[1,2],[1,1],null],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=((^){0}))a\u0022}"
        "(?<=(?:\u005cb){2})a" flags="d" input=" a" "{\u0022matches\u0022:[\u0022a\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022 a\u0022,\u0022indices\u0022:[[1,2]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=(?:\u005c\u005cb){2})a\u0022}"
        "(?<=(?:\u005cB){2})a" flags="d" input="ba" "{\u0022matches\u0022:[\u0022a\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022ba\u0022,\u0022indices\u0022:[[1,2]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=(?:\u005c\u005cB){2})a\u0022}"
        "(?<=(\u005cb){2})a" flags="d" input=" a" "{\u0022matches\u0022:[\u0022a\u0022,\u0022\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022 a\u0022,\u0022indices\u0022:[[1,2],[1,1]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=(\u005c\u005cb){2})a\u0022}"
        "(?<=(?:$){2})" flags="d" input="a" "{\u0022matches\u0022:[\u0022\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022a\u0022,\u0022indices\u0022:[[1,1]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=(?:$){2})\u0022}"
        "(?<=($){2})" flags="d" input="a" "{\u0022matches\u0022:[\u0022\u0022,\u0022\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022a\u0022,\u0022indices\u0022:[[1,1],[1,1]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=($){2})\u0022}"
        "(?<=(a\u005cB){2})b" flags="d" input="aab" "{\u0022matches\u0022:[\u0022b\u0022,\u0022a\u0022],\u0022index\u0022:2,\u0022input\u0022:\u0022aab\u0022,\u0022indices\u0022:[[2,3],[0,1]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=(a\u005c\u005cB){2})b\u0022}"
        "(?<=((a)\u005cB()){2})b" flags="d" input="aab" "{\u0022matches\u0022:[\u0022b\u0022,\u0022a\u0022,\u0022a\u0022,\u0022\u0022],\u0022index\u0022:2,\u0022input\u0022:\u0022aab\u0022,\u0022indices\u0022:[[2,3],[0,1],[0,1],[1,1]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=((a)\u005c\u005cB()){2})b\u0022}"
        "(?<=((\u005cb)a(\u005cB)){1})b" flags="d" input="ab" "{\u0022matches\u0022:[\u0022b\u0022,\u0022a\u0022,\u0022\u0022,\u0022\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022ab\u0022,\u0022indices\u0022:[[1,2],[0,1],[0,0],[1,1]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=((\u005c\u005cb)a(\u005c\u005cB)){1})b\u0022}"
        "(?<!((a)\u005cB){2}q)c" flags="d" input="aaarc" "{\u0022matches\u0022:[\u0022c\u0022,null,null],\u0022index\u0022:4,\u0022input\u0022:\u0022aaarc\u0022,\u0022indices\u0022:[[4,5],null,null],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<!((a)\u005c\u005cB){2}q)c\u0022}"
        "(?<=(?:^a){1})b" flags="dm" input="q\u000aab" "{\u0022matches\u0022:[\u0022b\u0022],\u0022index\u0022:3,\u0022input\u0022:\u0022q\u005cnab\u0022,\u0022indices\u0022:[[3,4]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=(?:^a){1})b\u0022}"
        "(?<=(?:\u005cba\u005cB){1})b" flags="d" input=" ab" "{\u0022matches\u0022:[\u0022b\u0022],\u0022index\u0022:2,\u0022input\u0022:\u0022 ab\u0022,\u0022indices\u0022:[[2,3]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=(?:\u005c\u005cba\u005c\u005cB){1})b\u0022}"
        "(?<=(?:a\u005cB){0})b" flags="d" input="b" "{\u0022matches\u0022:[\u0022b\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022b\u0022,\u0022indices\u0022:[[0,1]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=(?:a\u005c\u005cB){0})b\u0022}"
        "(?<=((?:a\u005cB){0}))b" flags="d" input="b" "{\u0022matches\u0022:[\u0022b\u0022,\u0022\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022b\u0022,\u0022indices\u0022:[[0,1],[0,0]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=((?:a\u005c\u005cB){0}))b\u0022}"
        "(?<=((^)){2}|((\u005cb)){2})a" flags="d" input=" a" "{\u0022matches\u0022:[\u0022a\u0022,null,null,\u0022\u0022,\u0022\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022 a\u0022,\u0022indices\u0022:[[1,2],null,null,[1,1],[1,1]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=((^)){2}|((\u005c\u005cb)){2})a\u0022}"
        "(?<=a(?<=(?:\u005cb){2}))b" flags="d" input="ab" "{\u0022match\u0022:null,\u0022lastIndex\u0022:0}"
        "(?:(?<=(a\u005cB){2})b|c)+d" flags="d" input="aabccd" "{\u0022matches\u0022:[\u0022bccd\u0022,null],\u0022index\u0022:2,\u0022input\u0022:\u0022aabccd\u0022,\u0022indices\u0022:[[2,6],null],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?:(?<=(a\u005c\u005cB){2})b|c)+d\u0022}"
        "((?<=(\u005cb){2})){2}a\u005c1\u005c2" flags="d" input="a" "{\u0022matches\u0022:[\u0022a\u0022,\u0022\u0022,\u0022\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022a\u0022,\u0022indices\u0022:[[0,1],[0,0],[0,0]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022((?<=(\u005c\u005cb){2})){2}a\u005c\u005c1\u005c\u005c2\u0022}"
        "((?<=(\u005cb){2}))*a\u005c1\u005c2" flags="d" input="a" "{\u0022matches\u0022:[\u0022a\u0022,null,null],\u0022index\u0022:0,\u0022input\u0022:\u0022a\u0022,\u0022indices\u0022:[[0,1],null,null],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022((?<=(\u005c\u005cb){2}))*a\u005c\u005c1\u005c\u005c2\u0022}"
        "(?<=(\u00b5\u005cB){2})\u039c" flags="di" input="\u039c\u00b5\u039c" "{\u0022matches\u0022:[\u0022\u039c\u0022,\u0022\u039c\u0022],\u0022index\u0022:2,\u0022input\u0022:\u0022\u039c\u00b5\u039c\u0022,\u0022indices\u0022:[[2,3],[0,1]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=(\u00b5\u005c\u005cB){2})\u039c\u0022}"
        "(?<=(^){2})a" flags="dm" input="q\u000aa" "{\u0022matches\u0022:[\u0022a\u0022,\u0022\u0022],\u0022index\u0022:2,\u0022input\u0022:\u0022q\u005cna\u0022,\u0022indices\u0022:[[2,3],[2,2]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=(^){2})a\u0022}"
        "(?<=($){2})" flags="dm" input="a\u000ab" "{\u0022matches\u0022:[\u0022\u0022,\u0022\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022a\u005cnb\u0022,\u0022indices\u0022:[[1,1],[1,1]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=($){2})\u0022}"
        "(?<=((.)(?:\u005cB)){2})b" flags="ds" input="\u000a\u000ab" "{\u0022match\u0022:null,\u0022lastIndex\u0022:0}"
        "(?<=([\u005cuD800]\u005cB){2})" flags="d" input="\ud800\ud800" "{\u0022matches\u0022:[\u0022\u0022,\u0022\u005cud800\u0022],\u0022index\u0022:2,\u0022input\u0022:\u0022\u005cud800\u005cud800\u0022,\u0022indices\u0022:[[2,2],[0,1]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=([\u005c\u005cuD800]\u005c\u005cB){2})\u0022}"
        "#);
}

#[test]
fn counted_lookbehind_boundary_offsets_empty_captures_named_aliases_and_rollback_are_exact() {
    assert_true(
        r"let a=/(?<=((?<x>a)\B()){2})b/d.exec('aab');a.index===2&&a[1]==='a'&&a.groups.x==='a'&&a[3]===''&&a.indices[1][0]===0&&a.indices[1][1]===1&&a.indices.groups.x===a.indices[2]&&a.indices[3][0]===1&&a.indices[3][1]===1",
    );
    assert_true(
        r"let a=/(?<=((\b)a(\B)){1})b/d.exec('ab');a[1]==='a'&&a[2]===''&&a[3]===''&&a.indices[2][0]===0&&a.indices[2][1]===0&&a.indices[3][0]===1&&a.indices[3][1]===1",
    );
    assert_true(
        r"let a=/(?<=(?<x>^){2})a/d.exec('a');a.groups.x===''&&a.indices.groups.x===a.indices[1]&&a.indices[1][0]===0&&a.indices[1][1]===0",
    );
    assert_true(
        r"let a=/(?<=((?<x>^){0}))a/d.exec('qa');a.index===1&&a[1]===''&&a.groups.x===undefined&&a.indices.groups.x===undefined&&a.indices[1][0]===1&&a.indices[1][1]===1",
    );
    assert_true(
        r"let a=/(?<!((?<x>a)\B){2}q)c/d.exec('aaarc');a.index===4&&a[1]===undefined&&a[2]===undefined&&a.groups.x===undefined&&a.indices[1]===undefined&&a.indices.groups.x===undefined",
    );
    assert_true(
        r"let a=/(?<=((^)){2}|((\b)){2})a/d.exec(' a');a[1]===undefined&&a[2]===undefined&&a[3]===''&&a[4]===''&&a.indices[3][0]===1&&a.indices[4][0]===1",
    );
    assert_true(
        r"let a=/(?<=(?<x>^){2}|(?<x>\b){2})a/d.exec(' a');a[1]===undefined&&a[2]===''&&a.groups.x===''&&a.indices.groups.x===a.indices[2]&&a.indices[2][0]===1",
    );
    assert_true(
        r"let a=/((?<=(\b){2})){2}a\1\2/d.exec('a'),b=/((?<=(\b){2}))*a\1\2/d.exec('a');a[1]===''&&a[2]===''&&a.indices[1][0]===0&&a.indices[2][0]===0&&b[1]===undefined&&b[2]===undefined",
    );
}

#[test]
fn counted_lookbehind_boundary_consumers_global_sticky_empty_and_callbacks_keep_positions() {
    assert_true(
        r"let r=/(?<=(?<x>\b){2})a/dg,a=[...'a a'.matchAll(r)];a.length===2&&a[0].index===0&&a[1].index===2&&a[1].groups.x===''&&a[1].indices.groups.x[0]===2&&r.lastIndex===0",
    );
    assert_true(
        r"let r=/(?<=(a\B){2})b/dy;r.lastIndex=2;let a=r.exec('aab');a[1]==='a'&&a.index===2&&r.lastIndex===3&&r.exec('aab')===null&&r.lastIndex===0",
    );
    assert_true(
        r"let r=/(?<=(\b){2})/dg,a=[...'ab cd'.matchAll(r)];a.length===4&&a[0].index===0&&a[1].index===2&&a[2].index===3&&a[3].index===5&&a[3][1]===''&&a[3].indices[1][0]===5&&r.lastIndex===0",
    );
    assert_true(
        r"let seen=[];let s='a a'.replace(/(?<=(?<x>\b){2})a/g,(m,c,i,s,g)=>{seen.push(m,c,i,s,g.x);return '_'});s==='_ _'&&seen.length===10&&seen[0]==='a'&&seen[1]===''&&seen[2]===0&&seen[4]===''&&seen[7]===2&&seen[9]===''",
    );
    assert_true(
        r"'a a'.replace(/(?<=(?<x>\b){2})a/g,'<$<x>>')==='<> <>'&&' a'.search(/(?<=(\b){2})a/)===1&&'a a'.split(/(?<=(\b){2})a/).join('|')==='|| ||'",
    );
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
fn variable_counts_and_nested_assertions_match_reference() {
    for source in [
        r"/(?<=(?:a\B){1,2})b/.exec('aab')",
        r"/(?<=(?:(?=a+)){2})a/.exec('a')",
        r"/(?<=(a\B|b\B){2})c/.exec('abc')",
        r"/(?<=((a\B){2}){2})c/.exec('aaaac')",
        r"/(?<=((?=a+)){2})a/.exec('a')",
        r"/(?<=(\b){2})a/u.exec('a')",
        r"/(?<=(\b){2})a/v.exec('a')",
    ] {
        crate::common::regexp_oracle::assert_matches_reference(&mut Realm::default(), source);
    }
}
