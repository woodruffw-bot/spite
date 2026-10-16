//! Fixed lookahead captures retain their forward iteration ranges.
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
fn fixed_lookahead_capture_result_snapshot() {
    let mut rows = String::new();
    for (source, flags, text) in [
        (r"(?<=a(?=(b)))b", "d", "ab"),
        (r"(?<=a(?=(b){2}))b", "d", "abb"),
        (r"(?<=a(?=((b)){2}))b", "d", "abb"),
        (r"(?<=a(?=(b){0}))b", "d", "ab"),
        (r"(?<=a(?=((b){0}){2}))b", "d", "ab"),
        (r"(?<=a(?=((b){0})*))b", "d", "ab"),
        (r"(?<=a(?=(b|c)))b", "d", "ab"),
        (r"(?<=a(?=(b)|(c)))b", "d", "ab"),
        (r"(?<=a(?=(b.)|(bc)))b", "d", "abc"),
        (r"(?<=a(?!((b){2})))b", "d", "abb"),
        (r"(?<!(?=(b))q)c", "d", "bc"),
        (r"(?<=(?=(a))(a))b", "d", "ab"),
        (r"(?<=a(?=(?=(b))b))b", "d", "ab"),
        (r"(?<=a(?=(?!(c))b))b", "d", "ab"),
        (r"(?<=a(?=(?<=(a))b))b", "d", "ab"),
        (r"(?<=a(?=(?<!((c)))b))b", "d", "ab"),
        (r"(?<=a(?=(b{2})))b", "d", "abb"),
        (r"(?<=a(?=((b)b){2}))b", "d", "abbbb"),
        (r"(?<=a(?=(b()){2}))b", "d", "abb"),
        (r"(?<=a(?=(()b){2}))b", "d", "abb"),
        (r"(?<=a(?=(\b){2}b))b", "d", "ab"),
        (r"(?<=a(?=((?=b)){2}b))b", "d", "ab"),
        (r"(?<=((?=(b))){2})b", "d", "b"),
        (r"(?<=((?=(b)))*)b", "d", "b"),
        (r"(?<=a(?=(b)))b\1", "d", "abb"),
        (r"(?<=a(?=(b){2}))b\1", "d", "abb"),
        (r"(?<=a(?=(b))|a(?=(c)))b", "d", "ab"),
        (r"(?<!a(?=(b)))b", "d", "ab"),
        (r"(?<=µ(?=(Μ){2}))Μ", "di", "ΜΜΜ"),
        (r"(?<=[\uD800](?=([\uDC00]){2}))[\uDC00]", "d", "surrogates"),
        (r"(?:(?<=a(?=(b)))b|a)+c", "d", "ababc"),
        (r"(?=(?<=a(?=(b)))b)b", "d", "qab"),
        (r"(?<=a(?=(?<x>b){2}))b", "d", "qabb"),
        (r"(?<=a(?=(?:(?<x>b)|(?<x>c))))b", "d", "qab"),
        (r"(?<=a(?=(?<=(?<x>a){2})b))b", "d", "qaab"),
        (r"(?<!(?=(?<x>b))q)c", "d", "qbc"),
        (r"(?<=a(?=(b){2}))b", "dg", "qabb"),
        (r"(?<=a(?=(b.)))b", "ds", "ab\n"),
        (r"(?<=a(?=(b$)))b", "dm", "ab\nq"),
    ] {
        let source = JsString::from(source);
        let flags = JsString::from(flags);
        let input = if text == "surrogates" {
            JsString::from_code_units(vec![0xd800, 0xdc00, 0xdc00])
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
fn named_forward_ranges_after_the_match_backward_children_and_negative_restoration() {
    check(
        r"let a=/(?<=a(?=(?<x>b){2}))b/d.exec('qabb');a.index===2&&a.groups.x==='b'&&a.indices.groups.x===a.indices[1]&&a.indices[1][0]===3&&a.indices[1][1]===4",
    );
    check(
        r"let a=/(?<=a(?=(?:(?<x>b)|(?<x>c))))b/d.exec('qab');a.groups.x==='b'&&a[1]==='b'&&a[2]===undefined&&a.indices.groups.x===a.indices[1]&&a.indices[1][0]===2",
    );
    check(
        r"let a=/(?<=a(?=(?<=(?<x>a){2})b))b/d.exec('qaab');a.groups.x==='a'&&a.index===3&&a.indices.groups.x[0]===1&&a.indices.groups.x[1]===2",
    );
    check(
        r"let a=/(?<!(?=(?<x>b))q)c/d.exec('qbc');a.index===2&&a.groups.x===undefined&&Object.hasOwn(a.groups,'x')&&a.indices.groups.x===undefined&&Object.hasOwn(a.indices.groups,'x')",
    );
    check(
        r"let a=/(?<=((?=(b))){2})b/d.exec('qb'),b=/(?<=((?=(b)))*)b/d.exec('qb');a[1]===''&&a[2]==='b'&&a.indices[1][0]===1&&a.indices[2][1]===2&&b[1]===undefined&&b[2]===undefined",
    );
}

#[test]
fn consumers_callbacks_imported_ranges_sticky_and_empty_advancement() {
    check(
        r"let r=/(?<=a(?=(b){2}))b/dg,a=[...'qabb abb'.matchAll(r)];a.length===2&&a[1].index===6&&a[1].indices[1][0]===7&&r.lastIndex===0",
    );
    check(
        r"let r=/(?<=a(?=(b){2}))b/dy;r.lastIndex=2;let a=r.exec('qabb');a.index===2&&a.indices[1][0]===3&&r.lastIndex===3&&r.exec('qabb')===null&&r.lastIndex===0",
    );
    check(
        r"let r=/(?<=((?=(b))){2})/dg,a=[...'bb'.matchAll(r)];a.length===2&&a[1].index===1&&a[1][1]===''&&a[1][2]==='b'&&a[1].indices[2][0]===1&&r.lastIndex===0",
    );
    check(
        r"let seen=[];let s='qabb abb'.replace(/(?<=a(?=(b){2}))b/g,(m,c,i)=>{seen.push(c,i);return '_'});s==='qa_b a_b'&&seen.join('|')==='b|2|b|6'",
    );
    check(
        r"'qabb'.replace(/(?<=a(?=(?<x>b){2}))b/,'<$<x>>')==='qa<b>b'&&'qabb'.search(/(?<=a(?=(b){2}))b/)===2&&'abb'.split(/(?<=a(?=(b){2}))b/).join('|')==='a|b|b'",
    );
}

#[test]
fn deep_captures_clones_collection_completed_negative_rollback_and_unlimited_parent_loops() {
    let mut realm = Realm::default();
    realm.eval("let n=100000,r=new RegExp('(?<=(?='+'('.repeat(n)+'b'+')'.repeat(n)+'))b','d'),copy=new RegExp(r)").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("let a=copy.exec('qb');a.index===1&&a.length===n+1&&a[1]==='b'&&a[n]==='b'&&a.indices[n][0]===1&&a.indices[n][1]===2&&copy.source===r.source"),Ok(Value::Boolean(true)));
    assert_eq!(realm.eval("let negative=new RegExp('(?<!(?='+'('.repeat(n)+'b'+')'.repeat(n)+')q)c','d'),restored=negative.exec('bc');restored.index===1&&restored[1]===undefined&&restored[n]===undefined&&restored.indices[n]===undefined"),Ok(Value::Boolean(true)));
    assert_eq!(realm.eval("let many=/(?:(?<=a(?=(b)))b|a)+c/.exec('ab'.repeat(10000)+'c');many.index===0&&many[0].length===20001&&many[1]==='b'"),Ok(Value::Boolean(true)));
    realm.collect(usize::MAX).unwrap();
}

#[test]
fn explicit_work_aborts_keep_last_index_and_bypass_handlers() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval("let marker=0,r=/(?<=a(?=(b){2}))b/g;r.lastIndex=1;let text='a'.repeat(5000)")
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
fn variable_width_dependent_continuations_and_unicode_modes_stay_unsupported() {
    for source in [
        r"/(?<=a(?=(b+)))b/.exec('abb')",
        r"/(?<=a(?=(b{1,2})\1))b/.exec('abb')",
        r"/(?<=a(?=(b|cc)))b/.exec('abb')",
        r"/(?<=a(?=(b){1,2}))b/.exec('abb')",
        r"/(?<=a(?=(b)))b/u.exec('ab')",
        r"/(?<=a(?=(b)))b/v.exec('ab')",
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
