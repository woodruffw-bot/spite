//! Completed same-unit references in fixed forward assertions inside lookbehind.
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
fn forward_local_lookahead_result_snapshot() {
    let mut rows = String::new();
    for (source, flags, text) in [
        (r"(?<=a(?=((b)\2){2}))b", "d", "abbbb"),
        (r"(?<=a(?=((b)\2){2}?))b", "d", "abbbb"),
        (r"(?<=a(?=(([ab])\2){2}))b", "d", "abbaa"),
        (r"(?<=a(?=((b)(\2)\3){2}))b", "d", "abbbbbb"),
        (r"(?<=a(?=((b)\2(\1)){2}))b", "d", "abbbb"),
        (r"(?<=a(?=((b)\2(\2)){2}))b", "d", "abbbbbb"),
        (r"(?<=((a)\1){2}(?=((b)\4){2}))b", "d", "aabbbb"),
        (r"(?<=a(?=((b)\2){0}))b", "d", "ab"),
        (r"(?<=a(?!((b)\2){2}q))b", "d", "abbbbx"),
        (r"(?<=a(?=((b)\2\B){2}))b", "d", "abbbbb"),
        (r"(?<=a(?=((b)\2){1}))b", "d", "abb"),
        (r"(?=((b)\2){2})b", "d", "bbbb"),
        (r"(a)(?<=a(?=((b)\3){2}))b", "d", "abbbb"),
        (r"(?<=((a)\2){2}(?=(((a)\5){2})))b", "d", "aaaab"),
        (r"(?<=a(?=((b)\2){2}))b", "di", "aBBBb"),
        (r"(?<=a(?=(?<y>(?<x>b)\k<x>){2}))b", "d", "abbbb"),
        (
            r"(?<=a(?=(?<y>(?<x>b)(?<z>\k<x>)\k<z>){2}))b",
            "d",
            "abbbbbb",
        ),
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
fn nested_forward_dependencies_names_both_directions_and_rollback_keep_exact_ranges() {
    check(
        r"let a=/(?<=a(?=(?<y>(?<x>b)\k<x>){2}))b/d.exec('abbbb');a.index===1&&a.groups.y==='bb'&&a.groups.x==='b'&&a.indices.groups.x===a.indices[2]&&a.indices[1][0]===3&&a.indices[2][0]===3",
    );
    check(
        r"let a=/(?<=a(?=(?<y>(?<x>b)(?<z>\k<x>)\k<z>){2}))b/d.exec('abbbbbb');a.groups.y==='bbb'&&a.indices.groups.x[0]===4&&a.indices.groups.z[0]===5&&a.groups.z==='b'",
    );
    check(
        r"let a=/(?<=(?<left>(a)\k<left>){2}(?=(?<right>(b)\4){2}))b/d.exec('aabbbb');a.groups.left==='a'&&a.groups.right==='bb'&&a.indices.groups.left[0]===0&&a.indices.groups.right[0]===4&&a.indices[4][0]===4",
    );
    check(
        r"let a=/(?<=a(?!((b)\2){2}q))b/d.exec('abbbbx');a.index===1&&a[1]===undefined&&a[2]===undefined&&a.indices[1]===undefined",
    );
    check(r"/(?<=((a)\2){2}(?=(((a)\5){2})))b/.exec('aaaab')===null");
    check(
        r"let a=/(?<=a(?=(([ab])\2){2}))b/d.exec('abbaa');a[1]==='aa'&&a[2]==='a'&&a.indices[2][0]===3",
    );
    check(
        r"let a=/(?<=a(?=(([\uD800])\2){2}))[\uD800]/d.exec('a\uD800\uD800\uD800\uD800\uDC00');a.index===1&&a.indices[1][0]===3&&a.indices[2][0]===3",
    );
}

#[test]
fn consumers_global_sticky_empty_advance_callbacks_and_zero_skips_preserve_future_ranges() {
    check(
        r"let r=/(?<=a(?=((b)\2){2}))b/dy;r.lastIndex=1;let a=r.exec('abbbb');a.index===1&&r.lastIndex===2&&r.exec('abbbb')===null&&r.lastIndex===0",
    );
    check(
        r"let r=/(?<=a(?=((b)\2){2}))b/dg,a=[...'abbbb abbbb'.matchAll(r)];a.length===2&&a[1].index===7&&a[1].indices[2][0]===9&&r.lastIndex===0",
    );
    check(
        r"let a=/(?<=a(?=((b)\2){0}))b/d.exec('ab');a[1]===undefined&&a[2]===undefined&&a.indices[2]===undefined",
    );
    check(
        r"let a=[...'abbbb'.matchAll(/(?<=a(?=((b)\2){2}))/dg)];a.length===1&&a[0].index===1&&a[0].indices[2][0]===3",
    );
    check(
        r"let seen=[];let s='abbbb abbbb'.replace(/(?<=a(?=((b)\2){2}))b/g,(m,x,y,i)=>{seen.push(x,y,i);return '_'});s==='a_bbb a_bbb'&&seen.join('|')==='bb|b|1|bb|b|7'",
    );
    check(
        r"'abbbb'.search(/(?<=a(?=((b)\2){2}))b/)===1&&'abbbb'.split(/(?<=a(?=((b)\2){2}))b/).join('|')==='a|bb|b|bbb'",
    );
}

#[test]
fn deep_scopes_clones_collection_completed_negative_empty_counts_and_large_units_are_unlimited() {
    let mut realm = Realm::default();
    realm.eval(r"let n=100000,r=new RegExp('(?<=a(?=('+'('.repeat(n)+'b'+')'.repeat(n)+'\\2){2}))b','d'),copy=new RegExp(r)").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("let a=copy.exec('abbbb');a.index===1&&a.length===n+2&&a[1]==='bb'&&a[n+1]==='b'&&a.indices[n+1][0]===3&&copy.source===r.source"),Ok(Value::Boolean(true)));
    assert_eq!(realm.eval(r"let neg=new RegExp('(?<=a(?!('+'('.repeat(n)+'b'+')'.repeat(n)+'\\2){2}q))b','d'),b=neg.exec('abbbbx');b[1]===undefined&&b[n+1]===undefined"),Ok(Value::Boolean(true)));
    assert_eq!(realm.eval(r"let huge=new RegExp('(?<=a(?=((())\\2){'+'9'.repeat(100)+'}))b','d'),h=huge.exec('ab');h[3]===''&&h.indices[3][0]===1"),Ok(Value::Boolean(true)));
    assert_eq!(realm.eval(r"let long=/(?<=a(?=((b)\2){50000}))b/dy;long.lastIndex=1;let found=long.exec('a'+'b'.repeat(100000));found.index===1&&found.indices[1][0]===99999&&found.indices[2][0]===99999"),Ok(Value::Boolean(true)));
    realm.collect(usize::MAX).unwrap();
}

#[test]
fn explicit_work_aborts_preserve_last_index_and_bypass_handlers() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm.eval(r"let marker=0,r=/(?<=a(?=((b)\2){2}))b/g;r.lastIndex=1;let text='b'.repeat(5000)+'abbbb'").unwrap();
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
fn variable_counts_choices_bare_capture_dependencies_and_unicode_remain_pending() {
    for source in [
        r"/(?<=a(?=((b)\2){1,2}))b/.exec('abbbb')",
        r"/(?<=a(?=((b)\2){2}|b))b/.exec('abbbb')",
        r"/(?<=a(?=(b)\1))b/.exec('abbbb')",
        r"/(?<=a(?=((b)\2){2}))b/u.exec('abbbb')",
        r"/(?<=a(?=((b)\2){2}))b/v.exec('abbbb')",
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
