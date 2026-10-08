//! Capture-free fixed lookahead inside fixed ordinary lookbehind.
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
fn fixed_lookahead_in_lookbehind_result_snapshot() {
    let mut rows = String::new();
    for (source, flags, text) in [
        (r"(?<=a(?=b))b", "d", "ab"),
        (r"(?<=a(?!c))b", "d", "ab"),
        (r"(?<=(?=ab)a)b", "d", "ab"),
        (r"(?<=(?!bb)a)b", "d", "ab"),
        (r"(?<=a(?=b{2}))b", "d", "abb"),
        (r"(?<=a(?=[bc]|d))b", "d", "ab"),
        (r"(?<=a(?=b|c))b", "d", "ab"),
        (r"(?<=a(?!b|c))d", "d", "ad"),
        (r"(?<=a(?=(?=b)b))b", "d", "ab"),
        (r"(?<=a(?=(?!c)b))b", "d", "ab"),
        (r"(?<=a(?=(?<=a)b))b", "d", "ab"),
        (r"(?<=a(?=(?<!c)b))b", "d", "ab"),
        (r"(?<=a(?=(?:b){2}))bb", "d", "abb"),
        (r"(?<=a(?=b{0}))b", "d", "ab"),
        (r"(?<=a(?=(?:b|c){0}))b", "d", "ab"),
        (r"(?<=a(?=$))", "d", "a"),
        (r"(?<=a(?=^))b", "d", "ab"),
        (r"(?<=a(?=\b)) ", "d", "a "),
        (r"(?<=a(?=\B))b", "d", "ab"),
        (r"(?<=a(?=.))b", "d", "ab"),
        (r"(?<=a(?=b.))b", "d", "abb"),
        (r"(?<=a(?!b.))b", "d", "ab"),
        (r"(?<=(?=ab)(a))b", "d", "ab"),
        (r"(?<=((?=b)){2})b", "d", "b"),
        (r"(?<=((?!a)){2})b", "d", "b"),
        (r"(?<=((?=b))*)b", "d", "b"),
        (r"(?<=a(?=(?:(?=b)){2}b))b", "d", "ab"),
        (r"(?<=a(?=(?<=a(?=b))b))b", "d", "ab"),
        (r"(?<=µ(?=Μ))Μ", "di", "ΜΜ"),
        (r"(?<!a(?=b))b", "d", "qb"),
        (r"(?<!a(?!b))b", "d", "ab"),
        (r"(?<=a(?=b)|a(?!c))b", "d", "ab"),
        (r"(?:(?<=a(?=b))b|a)+c", "d", "ababc"),
        (r"(?=(?<=a(?=b))b)b", "d", "qab"),
        (r"(?<=[\uD800](?=[\uDC00]))[\uDC00]", "d", "surrogates"),
        (r"(?<=(?=ab)(?<x>a))b", "d", "qab"),
        (r"(?<=(?=ab)(?:(?<x>a)|(?<x>c)))b", "d", "qab"),
        (r"(?<=((?=b)){2})b", "dg", "qb"),
        (r"(?<=a(?=b$))b", "dm", "ab\nq"),
        (r"(?<=a(?=b.))b", "ds", "ab\n"),
    ] {
        let source = JsString::from(source);
        let flags = JsString::from(flags);
        let input = if text == "surrogates" {
            JsString::from_code_units(vec![0xd800, 0xdc00])
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
fn parent_named_ranges_optional_slots_atomic_negative_and_full_input_context() {
    check(
        r"let a=/(?<=(?=ab)(?<x>a))b/d.exec('qab');a.groups.x==='a'&&a.indices.groups.x===a.indices[1]&&a.indices[1][0]===1&&a.indices[1][1]===2&&a.index===2",
    );
    check(
        r"let a=/(?<=((?=b)){2})b/d.exec('qb'),b=/(?<=((?=b))*)b/d.exec('qb');a[1]===''&&a.indices[1][0]===1&&b[1]===undefined&&b.indices[1]===undefined",
    );
    check(
        r"let a=/(?<=a(?=b.)|a(?!b.))b/d.exec('ab');a.index===1&&a[0]==='b'&&/(?<!a(?=b))b/.exec('ab')===null&&/(?<!a(?!b))b/.exec('ab').index===1",
    );
    check(
        r"/(?<=a(?=b$))b/m.exec('ab\nq').index===1&&/(?<=a(?=b.))b/s.exec('ab\n').index===1&&/(?<=a(?=b.))b/.exec('ab\n')===null",
    );
}

#[test]
fn shared_consumers_sticky_global_empty_advancement_and_callback_ranges() {
    check(
        r"let r=/(?<=(?=ab)(a))b/dg,a=[...'qab ab'.matchAll(r)];a.length===2&&a[1].index===5&&a[1].indices[1][0]===4&&r.lastIndex===0",
    );
    check(
        r"let r=/(?<=a(?=b))b/dy;r.lastIndex=2;let a=r.exec('qab');a.index===2&&r.lastIndex===3&&r.exec('qab')===null&&r.lastIndex===0",
    );
    check(
        r"let r=/(?<=((?=b)){2})/dg,a=[...'bb'.matchAll(r)];a.length===2&&a[1].index===1&&a[1][1]===''&&a[1].indices[1][0]===1&&r.lastIndex===0",
    );
    check(
        r"let seen=[];let s='qab ab'.replace(/(?<=(?=ab)(a))b/g,(m,c,i)=>{seen.push(c,i);return '_'});s==='qa_ a_'&&seen.join('|')==='a|2|a|5'",
    );
    check(
        r"'qab'.search(/(?<=a(?=b))b/)===2&&'ab ab'.split(/(?<=a(?=b))b/).join('|')==='a| a|'&&'qab'.replace(/(?<=(?=ab)(?<x>a))b/,'<$<x>>')==='qa<a>'",
    );
}

#[test]
fn deep_assertions_clones_collection_and_long_parent_loops_have_unlimited_defaults() {
    let mut realm = Realm::default();
    realm.eval("let n=100000,r=new RegExp('(?<='+'(?='.repeat(n)+'b'+')'.repeat(n)+')b','d'),copy=new RegExp(r)").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(
        realm.eval("copy.exec('qb').index===1&&copy.source===r.source"),
        Ok(Value::Boolean(true))
    );
    assert_eq!(realm.eval("let a=/(?:(?<=a(?=b))b|a)+c/.exec('ab'.repeat(10000)+'c');a.index===0&&a[0].length===20001"),Ok(Value::Boolean(true)));
    realm.collect(usize::MAX).unwrap();
}

#[test]
fn explicit_work_aborts_keep_last_index_and_bypass_handlers() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval("let marker=0,r=/(?<=a(?=b))b/g;r.lastIndex=1;let text='a'.repeat(5000)")
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
fn capture_dependent_variable_width_and_unicode_lookahead_stay_unsupported() {
    for source in [
        r"/(?<=a(?=(b)))b/.exec('ab')",
        r"/(?<=a(?=b+))b/.exec('ab')",
        r"/(?<=a(?=b|cc))b/.exec('ab')",
        r"/(?<=a(?!b+))b/.exec('ab')",
        r"/(?<=a+)b/.exec('ab')",
        r"/(?<=a(?=b))b/u.exec('ab')",
        r"/(?<=a(?=b))b/v.exec('ab')",
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
