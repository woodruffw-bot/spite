//! Effect-free counted reference atoms executed exactly once in assertions.
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
fn count_one_reference_assertions_result_snapshot() {
    let mut rows = String::new();
    for (source, flags, text) in [
        (r"(?<=a(?=(b)\1{1}))b", "d", "abb"),
        (r"(?<=a(?=(b)\1{1}?))b", "d", "abb"),
        (r"(?<=a(?=(b)(\1{1})\2{1}))b", "d", "abbb"),
        (r"(?<=(\2{1})(b))c", "d", "bbc"),
        (r"(?<=(\2{1}?)(b))c", "d", "bbc"),
        (r"(?<=(\2{1})(\3{1})(b))c", "d", "bbbc"),
        (r"(a)(?<=a(?=(b)\2{1}\1{1}))b", "d", "abbab"),
        (r"(a)(?<=(\3{1})(\1{1}))c", "d", "aaac"),
        (r"(ab)(?<=b(?=(\1{1})\2{1}))a", "d", "ababab"),
        (r"(?<=(\1{1}))a", "d", "a"),
        (r"(?<=(a)\1{1})b", "d", "aab"),
        (r"(?<=a(?!((b)\2{1})q))b", "d", "abbx"),
        (r"(?<!(\2{1})(b))c|c", "d", "bbc"),
        (r"(?<=a(?=(\1{1}b)))b", "d", "ab"),
        (r"(a)(?<=\1{1}\1{1})b", "d", "aab"),
        (r"(a)(?<=\1\1{1})b", "d", "aab"),
        (r"(a)(?<=a(?=\1{1}\1{1}))a", "d", "aaa"),
        (r"(?<=a(?=(?<x>b)\k<x>{1}))b", "d", "abb"),
        (r"(?<=(\k<x>{1})(?<x>b))c", "d", "bbc"),
        (r"(?<=(?<x>\k<x>{1}))a", "d", "a"),
        (r"(a)(?<=a(?=(?<x>b)\k<x>{1}\1{1}))b", "d", "abbab"),
        (r"(?<=a(?=(b)\1{1}))b", "di", "aBb"),
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
fn named_ranges_input_copies_dependent_targets_and_negative_rollback_are_exact() {
    check(
        r"let a=/(?<=a(?=(?<x>b)\k<x>{1}))b/d.exec('abb');a.groups.x==='b'&&a.indices.groups.x===a.indices[1]&&a.indices[1][0]===1",
    );
    check(
        r"let a=/(?<=(\k<x>{1})(?<x>b))c/d.exec('bbc');a[1]==='b'&&a.groups.x==='b'&&a.indices[1][0]===0&&a.indices.groups.x[0]===1",
    );
    check(
        r"let a=/(a)(?<=\1{1}\1{1})b/d.exec('aab');a.index===1&&a[0]==='ab'&&a.indices[1][0]===1",
    );
    check(
        r"let a=/(?<=a(?!(b)\1{1}))b|(?<=a)b/d.exec('abb');a.index===1&&a[1]===undefined&&a.indices[1]===undefined",
    );
    check(
        r"let a=/(?<=a(?=([\uD800])\1{1}))[\uD800]/d.exec('a\uD800\uD800\uDC00');a.index===1&&a.indices[1][0]===1",
    );
}
#[test]
fn consumers_keep_sticky_global_empty_advancement_and_callback_positions() {
    check(
        r"let r=/(?<=a(?=(b)\1{1}))b/dy;r.lastIndex=1;let a=r.exec('abb');a[1]==='b'&&r.lastIndex===2&&r.exec('abb')===null&&r.lastIndex===0",
    );
    check(
        r"let r=/(?<=a(?=(b)\1{1}))b/dg,a=[...'abb abb'.matchAll(r)];a.length===2&&a[1].index===5&&a[1].indices[1][0]===5&&r.lastIndex===0",
    );
    check(
        r"let a=[...'abb'.matchAll(/(?<=a(?=(b)\1{1}))/dg)];a.length===1&&a[0].index===1&&a[0][0]===''",
    );
    check(
        r"let seen=[];let s='abb abb'.replace(/(?<=a(?=(b)\1{1}))b/g,(m,x,i)=>{seen.push(x,i);return '_'});s==='a_b a_b'&&seen.join('|')==='b|1|b|5'",
    );
    check(
        r"'abb'.search(/(?<=a(?=(b)\1{1}))b/)===1&&'abb'.split(/(?<=a(?=(b)\1{1}))b/).join('|')==='a|b|b'",
    );
}
#[test]
fn deep_capture_scopes_dependency_chains_copies_and_gc_keep_unlimited_defaults() {
    let mut realm = Realm::default();
    realm.eval(r"let n=100000,r=new RegExp('(?<=a(?=('+'('.repeat(n)+'b'+')'.repeat(n)+'\\2{1})))b','d'),copy=new RegExp(r)").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("let a=copy.exec('abb');a.length===n+2&&a[1]==='bb'&&a[n+1]==='b'&&a.indices[n+1][0]===1&&copy.source===r.source"),Ok(Value::Boolean(true)));
    assert_eq!(realm.eval(r"let neg=new RegExp('(?<=a(?!('+'('.repeat(n)+'b'+')'.repeat(n)+'\\2{1})q))b','d'),b=neg.exec('abbbx');b[1]===undefined&&b[n+1]===undefined"),Ok(Value::Boolean(true)));
    assert_eq!(realm.eval(r"let chain='(?<=a(?=(b)';for(let i=1;i<10000;i++)chain+='(\\'+i+'{1})';chain+='\\10000{1}))b';let c=new RegExp(chain,'dy');c.lastIndex=1;let found=c.exec('a'+'b'.repeat(10001));found.indices[10000][0]===10000"),Ok(Value::Boolean(true)));
    realm.collect(usize::MAX).unwrap();
}
#[test]
fn opted_in_work_abort_preserves_last_index_and_bypasses_handlers() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval(
            r"let marker=0,r=/(?<=a(?=(b)\1{1}))b/g;r.lastIndex=1;let text='b'.repeat(5000)+'abb'",
        )
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
fn larger_consuming_counts_capture_effects_choices_and_unicode_remain_pending() {
    for source in [
        r"/(?<=a(?=(b)\1{1,2}))b/.exec('abb')",
        r"/(?<=(\2{1,2})(b))c/.exec('bbc')",
        r"/(?<=a(?=(b)(\1){1}))b/.exec('abb')",
        r"/(?<=(\2{1}|a)(b))c/.exec('bbc')",
        r"/(?<=a(?=(b)\1{1}))b/u.exec('abb')",
        r"/(?<=a(?=(b)\1{1}))b/v.exec('abb')",
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
