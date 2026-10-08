//! Exact counts of single-unit character and class terms inside lookbehind.
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
fn fixed_count_lookbehind_result_snapshot() {
    let mut rows = String::new();
    for (source, flags, text) in [
        (r"(?<=a{1})b", "d", "ab"),
        (r"(?<=a{2})b", "d", "aab"),
        (r"(?<!a{2})b", "d", "ab"),
        (r"(?<=a{0})b", "d", "b"),
        (r"(?<!a{0})b", "d", "b"),
        (r"(?<=a{2,2})b", "d", "aab"),
        (r"(?<=a{2}?)b", "d", "aab"),
        (r"(?<!a{2}?)b", "d", "aab"),
        (r"(?<=a{1}b{1})c", "d", "abc"),
        (r"(?<=a[a-z]{2})d", "d", "abcd"),
        (r"(?<!a[a-z]{2})d", "d", "abcd"),
        (r"(?<=.{2})b", "ds", "\n\nb"),
        (r"(?<=.{2})b", "d", "\n\nb"),
        (r"(?<=\w{2})b", "d", "aab"),
        (r"(?<=\s{1})b", "d", " b"),
        (r"(?<=[^a]{2})b", "d", "ccb"),
        (r"(?<=\b[a-z]{2})c", "d", " abc"),
        (r"(?<=a{2}\B)b", "d", "aab"),
        (r"(?<=^a{2})b", "dm", "q\naab"),
        (r"(?<=a{2}$)", "dm", "aa\nb"),
        (r"(?<=µ{2})Μ", "di", "ΜµΜ"),
        (r"(?<=(?:a){2})b", "d", "aab"),
        (r"(?<=a{0}b)c", "d", "bc"),
        (r"(?<=a{1})(?<x>b)\k<x>", "d", "abb"),
        (r"(?:(?<=a{2})b|c)+d", "d", "aabcd"),
        (r"(?:(?<=a{2})|(?<!b{2})){2}b", "d", "aab"),
        (r"((?<=a{2})){2}b\1", "d", "aab"),
        (r"(?<=[\uD800]{2})b", "d", "lone surrogates b"),
    ] {
        let source = JsString::from(source);
        let flags = JsString::from(flags);
        let text = if text == "lone surrogates b" {
            JsString::from_code_units(vec![0xd800, 0xd800, 0x62])
        } else {
            JsString::from(text)
        };
        let program = format!(
            "let r=new RegExp({source:?},{flags:?}),a=r.exec({text:?});JSON.stringify(a===null?{{match:null,lastIndex:r.lastIndex}}:{{matches:[...a],index:a.index,input:a.input,groups:a.groups,indices:a.indices,indicesGroups:a.indices.groups,lastIndex:r.lastIndex,source:r.source}})"
        );
        let Value::String(value) = Realm::default().eval(&program).unwrap() else {
            panic!("expected JSON")
        };
        writeln!(rows, "{source:?} flags={flags:?} input={text:?} {value:?}").unwrap();
    }
    insta::assert_snapshot!(rows);
}

#[test]
fn fixed_count_lookbehind_ranges_named_aliases_zero_counts_and_outer_retries_are_exact() {
    check(
        r"let a=/(?<=a{2})(?<x>b)\k<x>/d.exec('aabb');a[0]==='bb'&&a.index===2&&a.groups.x==='b'&&a.indices.groups.x===a.indices[1]&&a.indices[1][0]===2&&a.indices[1][1]===3",
    );
    check(
        r"let a=/(?:(?<x>a)(?<!a{1})|(?<x>a)(?<=a{1}))\k<x>/d.exec('aa');a[0]==='aa'&&a[1]===undefined&&a[2]==='a'&&a.indices[1]===undefined&&a.indices.groups.x===a.indices[2]",
    );
    check(
        r"let a=/((?<=a{2})){2}b\1/d.exec('aab'),b=/((?<=a{2}))*b\1/d.exec('aab');a[1]===''&&a.indices[1][0]===2&&a.indices[1][1]===2&&b[1]===undefined&&b.indices[1]===undefined",
    );
    check(
        r"/(?<=a{0}b)c/.exec('bc').index===1&&/(?<!a{0})b/.exec('b')===null&&/(?<=^a{2})b/m.exec('q\naab').index===4&&/(?<=a{2}$)/m.exec('aa\nb').index===2",
    );
    check(
        r"let a=/(?=(?<x>a{2}(?<=a{2})b))\k<x>/d.exec('aab');a[0]==='aab'&&a.groups.x==='aab'&&a.indices.groups.x[1]===3",
    );
}

#[test]
fn fixed_count_lookbehind_consumers_global_sticky_and_empty_advancement_preserve_positions() {
    check(
        r"let r=/(?<=a{2})(?<x>b)/dg,a=[...'aabqaab'.matchAll(r)];a.length===2&&a[0].index===2&&a[1].index===6&&a[1].groups.x==='b'&&a[1].indices.groups.x[0]===6&&r.lastIndex===0",
    );
    check(
        r"let r=/(?<=a{2})b/dy;r.lastIndex=2;let a=r.exec('aab');a.index===2&&r.lastIndex===3&&r.exec('aab')===null&&r.lastIndex===0",
    );
    check(
        r"let r=/(?<=a{2})/dg,a=[...'aaa'.matchAll(r)];a.length===2&&a[0][0]===''&&a[0].index===2&&a[1].index===3&&r.lastIndex===0",
    );
    check(
        r"let seen=[];let s='aabqaab'.replace(/(?<=a{2})(?<x>b)/g,(m,c,i,s,g)=>{seen.push(m,c,i,s,g.x);return '_'});s==='aa_qaa_'&&seen.length===10&&seen[2]===2&&seen[4]==='b'&&seen[7]===6",
    );
    check(
        r"'aabqaab'.replace(/(?<=a{2})(?<x>b)/g,'<$<x>>')==='aa<b>qaa<b>'&&'aabqaab'.search(/(?<=a{2})b/)===2&&'aabqaab'.split(/(?<=a{2})b/).join('|')==='aa|qaa|'",
    );
}

#[test]
fn fixed_count_lookbehind_deep_wrappers_compact_large_counts_copies_and_gc_are_unlimited() {
    let mut realm = Realm::default();
    realm.eval("let r=new RegExp('(?<='+'(?:'.repeat(100000)+'a{2}'+')'.repeat(100000)+')(?<x>b)','d'),copy=new RegExp(r)").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("let a=copy.exec('aab');a.index===2&&a.groups.x==='b'&&a.indices.groups.x===a.indices[1]&&copy.source===r.source"),Ok(Value::Boolean(true)));
    assert_eq!(realm.eval("let long=new RegExp('(?<=a{10000})b','dy');long.lastIndex=10000;let last=long.exec('a'.repeat(10000)+'b');last.index===10000&&last[0]==='b'&&long.lastIndex===10001"),Ok(Value::Boolean(true)));
    assert_eq!(realm.eval("let many=/(?:(?<=a{2})b|c)+d/.exec('aab'+'c'.repeat(10000)+'d');many.index===2&&many[0].length===10002"),Ok(Value::Boolean(true)));
    realm.collect(usize::MAX).unwrap();
}

#[test]
fn explicit_fixed_count_lookbehind_unit_work_aborts_before_last_index_and_handlers() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval("let marker=0,r=/(?<=a{8})b/g;r.lastIndex=1;let text='a'.repeat(5000)")
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
fn variable_counts_capture_effects_multiple_terms_and_nested_lookbehind_remain_unsupported() {
    for source in [
        r"/(?<=a{1,2})b/.exec('ab')",
        r"/(?<=(a{2}))b/.exec('aab')",
        r"/(?<=(?:ab){2})c/.exec('ababc')",
        r"/(?<=(?:^){2})a/.exec('a')",
        r"/(?<=a{2}(?=a))b/.exec('aab')",
        r"/(?<=a{2})b/u.exec('aab')",
        r"/(?<=a{2})b/v.exec('aab')",
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
