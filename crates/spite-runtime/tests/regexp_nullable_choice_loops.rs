//! RepeatMatcher rejects optional empty iterations while retrying nullable bodies.
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
fn nullable_choice_loops_result_snapshot() {
    let mut rows = String::new();
    for (source, flags, text) in [
        (r"(a|)*", "d", ""),
        (r"(a|)+", "d", ""),
        (r"(a|)*", "d", "aa"),
        (r"(a|)*?b", "d", "aab"),
        (r"(a|)+?b", "d", "aab"),
        (r"(a|){0,3}b", "d", "aab"),
        (r"(a|){1,3}b", "d", "b"),
        (r"((a?)|b)*c", "d", "bbc"),
        (r"((a)|)*\2", "d", "aaa"),
        (r"(a)((\1?)|b)*c", "d", "abac"),
        (r"((a?)*|b)+c", "d", "bbc"),
        (r"((a|)*)+b", "d", "aab"),
        (r"(?=((a|)*))\1b", "d", "aab"),
        (r"(?!((a|)*c))(a|)*b", "d", "aab"),
        (r"(?:(?=(a?))a?|b)*c", "d", "bbc"),
        (r"(?:(\1)|a)*c", "d", "aac"),
        (r"((.*\n?)*?)c", "d", "a\nbc"),
        (r"(?<x>a|)*", "d", "aa"),
        (r"(?<x>a|)+", "d", ""),
        (r"([a-z]|)*", "di", "aAA"),
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
fn empty_attempts_restore_names_indices_nested_body_and_atomic_assertion_effects() {
    check(
        r"let a=/(?<x>a|)*/d.exec('aa');a[0]==='aa'&&a.groups.x==='a'&&a.indices.groups.x===a.indices[1]&&a.indices[1][0]===1",
    );
    check(
        r"let a=/(?<x>a|)*/d.exec('');a[0]===''&&a.groups.x===undefined&&a.indices.groups.x===undefined",
    );
    check(r"let a=/(?<x>a|)+/d.exec('');a.groups.x===''&&a.indices[1][0]===0&&a.indices[1][1]===0");
    check(
        r"let a=/((a?)|b)*c/d.exec('bbc');a[1]==='b'&&a[2]===undefined&&a.indices[1][0]===1&&a.indices[2]===undefined",
    );
    check(r"let a=/(?=((a|)*))\1b/d.exec('aab');a[1]==='aa'&&a[2]==='a'&&a.indices[2][0]===1");
    check(
        r"let a=/(?!((a|)*c))(a|)*b/d.exec('aab');a[1]===undefined&&a[2]===undefined&&a[3]==='a'&&a.indices[3][0]===1",
    );
    check(
        r"let a=/([\uD800]|)*/d.exec('\uD800\uD800\uDC00');a[0].length===2&&a.indices[1][0]===1&&a.indices[1][1]===2",
    );
}

#[test]
fn consumers_global_sticky_empty_advancement_and_callbacks_keep_restored_slots() {
    check(
        r"let r=/(a|)*b/dy;r.lastIndex=1;let a=r.exec('xaab');a.index===1&&a[1]==='a'&&a.indices[1][0]===2&&r.lastIndex===4&&r.exec('xaab')===null&&r.lastIndex===0",
    );
    check(
        r"let r=/(a|)*/dg,a=[...'aa b'.matchAll(r)];a.length===4&&a[0][0]==='aa'&&a[1].index===2&&a[1][1]===undefined&&a[3].index===4&&r.lastIndex===0",
    );
    check(
        r"let rows=[],s='aa b'.replace(/(a|)*/g,(m,x,i)=>{rows.push([m,x,i]);return '#'});s==='## #b#'&&JSON.stringify(rows)===JSON.stringify([['aa','a',0],['',undefined,2],['',undefined,3],['',undefined,4]])",
    );
    check(r"let a='aab'.split(/(a|)*b/);a.length===3&&a[0]===''&&a[1]==='a'&&a[2]===''");
    check(
        r"let r=/(a|)*/g;r.lastIndex=1;let i='aa'.search(r);i===0&&r.lastIndex===1&&'aa'.match(r).length===2",
    );
}

#[test]
fn copies_collection_deep_slots_and_long_consuming_paths_preserve_last_iteration() {
    let body = "(".repeat(100000) + "a|" + &")".repeat(100000);
    let source = JsString::from((body.clone() + "*").as_str());
    let required = JsString::from((body + "+").as_str());
    let program = format!(
        "let r=new RegExp({source:?},'d'),copy=new RegExp(r),required=new RegExp({required:?},'d');copy.exec('a');true"
    );
    let mut realm = Realm::default();
    assert_eq!(realm.eval(&program), Ok(Value::Boolean(true)));
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("let a=copy.exec('a'),z=copy.exec(''),e=required.exec('');a.length===100001&&a[100000]==='a'&&a.indices[100000][0]===0&&z[100000]===undefined&&e[100000]===''&&e.indices[100000][1]===0"), Ok(Value::Boolean(true)));
    check(
        r"let a=/((a?)|b)*c/d.exec('b'.repeat(50000)+'c');a[0].length===50001&&a[1]==='b'&&a[2]===undefined&&a.indices[1][0]===49999",
    );
}

#[test]
fn nullable_composition_preserves_ordered_coercions_and_normal_eval_completion() {
    check(
        "let it='x'.matchAll('(?:(?:(?:(?:a(a|bc)){2}){2})|)*'),flag=0,a;try{a=it.next()}catch{flag=1}finally{flag=2}a.value[0]===''&&!a.done&&flag===2&&it.next().value.index===1&&it.next().done",
    );
    for pattern in [
        "(?:(?:(?:(?:a(a|bc)){2}){2})|)*",
        "(?:(?:((?:(?:a|[b]+[a]+){2}){2}){2})|)*",
        "(?:(?:((?:(?:.a*b*){2}){2}){2})|)*",
        "(?:(?:((?:(?:[a]a+b+){2}){2}){2})|)*",
        "(?:(?:((?:(?:ab+c+){2}){2}){2})|)*",
    ] {
        check(&format!(
            "let r=new RegExp('{pattern}'),t='';r.lastIndex={{valueOf(){{t+='i';return 0}}}};let a=r.exec({{toString(){{t+='s';return 'a'}}}});t==='si'&&a[0]===''&&typeof r.lastIndex==='object'"
        ));
    }
    check(
        "let marker=0,result;try{result=eval('marker=1; /(?:(?:((?:(?:.a*b*){2}){2}){2})|)*/g.test(0);')}catch{marker=2}finally{marker=3}result===true&&marker===3",
    );
    check(
        "let marker=0,result;try{result=eval('marker=1; /(?:(?:((?:(?:[a-z]a+b+){2}){2}){2})|)*/.test(0);')}catch{marker=2}finally{marker=3}result===true&&marker===3",
    );
    check(
        r"let marker=0,result;try{result=eval('marker=1; /(?:(?:(?:(?<a>a)|(?<a>b)(?:\\k<a>)+){2})|)*/.test(0);')}catch{marker=2}finally{marker=3}result===true&&marker===3",
    );
}

#[test]
fn explicit_work_abort_preserves_last_index_and_skips_callbacks() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    assert_eq!(
        realm.eval(
            "var r=/((a?)|b)*c/g;r.lastIndex=1;var input='x'+'b'.repeat(5000)+'c',called=false;true"
        ),
        Ok(Value::Boolean(true))
    );
    assert!(matches!(
        realm.eval("input.replace(r,()=>{called=true;return 'x'})"),
        Err(Error::Limit { .. })
    ));
    assert_eq!(
        realm.eval("r.lastIndex===0&&!called"),
        Ok(Value::Boolean(true))
    );
}

#[test]
fn larger_required_nullable_counts_backward_bodies_and_unicode_remain_pending() {
    for source in [
        "new RegExp('(a|){2}').exec('aa')",
        "new RegExp('((a?)|b){2,3}').exec('bbc')",
        "new RegExp('(?<=(a|)+)b').exec('ab')",
        "new RegExp('(a|)*','u').exec('aa')",
        "new RegExp('(a|)*','v').exec('aa')",
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
