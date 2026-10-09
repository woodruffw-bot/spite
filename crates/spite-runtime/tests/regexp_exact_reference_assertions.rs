//! Effect-free flat exact-count reference atoms in assertions.
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
fn exact_reference_assertions_result_snapshot() {
    let mut rows = String::new();
    for (source, flags, text) in [
        (r"(?<=a(?=(b)\1{2}))b", "d", "abbb"),
        (r"(?<=a(?=(b)\1{2}?))b", "d", "abbb"),
        (r"(?<=a(?=(b)(\1{2})\2{2}))b", "d", "abbbbbbb"),
        (r"(?<=(\2{2})(b))c", "d", "bbbc"),
        (r"(?<=(\2{2}?)(b))c", "d", "bbbc"),
        (r"(?<=(\2{2})(\3{2})(b))c", "d", "bbbbbbbc"),
        (r"(a)(?<=a(?=(b)\2{2}\1{2}))b", "d", "abbbaa"),
        (r"(a)(?<=(\3{2})(\1{2}))c", "d", "aaaaaac"),
        (r"(ab)(?<=b(?=(\1{2})\2{2}))a", "d", "ababababababab"),
        (r"(?<=(\1{2}))a", "d", "a"),
        (r"(?<=(a)\1{2})b", "d", "ab"),
        (r"(?<=a(?!((b)\2{2})q))b", "d", "abbbx"),
        (r"(?<!(\2{2})(b))c|c", "d", "bbbc"),
        (r"(a)(?<=\1{2}\1{2})b", "d", "aaaab"),
        (r"(a)(?<=\1\1{2})b", "d", "aaab"),
        (r"(a)(?<=a(?=\1{2}\1{2}))a", "d", "aaaaa"),
        (r"(?<=a(?=(b)(?:\1\1){2}))b", "d", "abbbbb"),
        (r"(?<=a(?=(?<x>b)\k<x>{2}))b", "d", "abbb"),
        (r"(?<=(\k<x>{2})(?<x>b))c", "d", "bbbc"),
        (r"(?<=(?<x>\k<x>{2}))a", "d", "a"),
        (r"(a)(?<=a(?=(?<x>b)\k<x>{2}\1{2}))b", "d", "abbbaa"),
        (r"(?<=a(?=(b)\1{2}))b", "di", "aBbb"),
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
        r"let a=/(?<=a(?=(?<x>b)\k<x>{2}))b/d.exec('abbb');a.groups.x==='b'&&a.indices.groups.x===a.indices[1]&&a.indices[1][0]===1",
    );
    check(
        r"let a=/(?<=(\k<x>{2})(?<x>b))c/d.exec('bbbc');a[1]==='bb'&&a.groups.x==='b'&&a.indices[1][0]===0&&a.indices.groups.x[0]===2",
    );
    check(
        r"let a=/(a)(?<=\1{2}\1{2})b/d.exec('aaaab');a.index===3&&a[0]==='ab'&&a.indices[1][0]===3",
    );
    check(
        r"let a=/(?<=a(?!(b)\1{2}))b|(?<=a)b/d.exec('abbb');a.index===1&&a[1]===undefined&&a.indices[1]===undefined",
    );
    check(
        r"let a=/(?<=a(?=([\uD800])\1{2}))[\uD800]/d.exec('a\uD800\uD800\uD800\uDC00');a.index===1&&a.indices[1][0]===1",
    );
}
#[test]
fn consumers_keep_sticky_global_empty_advancement_and_callback_positions() {
    check(
        r"let r=/(?<=a(?=(b)\1{2}))b/dy;r.lastIndex=1;let a=r.exec('abbb');a[1]==='b'&&r.lastIndex===2&&r.exec('abbb')===null&&r.lastIndex===0",
    );
    check(
        r"let r=/(?<=a(?=(b)\1{2}))b/dg,a=[...'abbb abbb'.matchAll(r)];a.length===2&&a[1].index===6&&a[1].indices[1][0]===6&&r.lastIndex===0",
    );
    check(
        r"let a=[...'abbb'.matchAll(/(?<=a(?=(b)\1{2}))/dg)];a.length===1&&a[0].index===1&&a[0][0]===''",
    );
    check(
        r"let seen=[];let s='abbb abbb'.replace(/(?<=a(?=(b)\1{2}))b/g,(m,x,i)=>{seen.push(x,i);return '_'});s==='a_bb a_bb'&&seen.join('|')==='b|1|b|6'",
    );
    check(
        r"'abbb'.search(/(?<=a(?=(b)\1{2}))b/)===1&&'abbb'.split(/(?<=a(?=(b)\1{2}))b/).join('|')==='a|b|bb'",
    );
}
#[test]
fn deep_capture_scopes_dependency_chains_copies_and_gc_keep_unlimited_defaults() {
    let mut realm = Realm::default();
    realm.eval(r"let n=100000,r=new RegExp('(?<=a(?=('+'('.repeat(n)+'b'+')'.repeat(n)+'\\2{2})))b','d'),copy=new RegExp(r)").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("let a=copy.exec('abbb');a.length===n+2&&a[1]==='bbb'&&a[n+1]==='b'&&a.indices[n+1][0]===1&&copy.source===r.source"),Ok(Value::Boolean(true)));
    assert_eq!(realm.eval(r"let neg=new RegExp('(?<=a(?!('+'('.repeat(n)+'b'+')'.repeat(n)+'\\2{2})q))b','d'),b=neg.exec('abbbx');b[1]===undefined&&b[n+1]===undefined"),Ok(Value::Boolean(true)));
    assert_eq!(realm.eval(r"let chain='(?<=a(?=(b)';for(let i=1;i<10000;i++)chain+='(\\1{2})';chain+='\\10000{2}))b';let c=new RegExp(chain,'dy');c.lastIndex=1;let found=c.exec('a'+'b'.repeat(20003));found.indices[10000][0]===19998"),Ok(Value::Boolean(true)));
    let source = JsString::from(format!(r"(?<=a(?=(b)\1{{{}}}))b", usize::MAX).as_str());
    check(&format!(
        "let r=new RegExp({source:?},'d');r.exec('abbb')===null"
    ));
    let source = JsString::from(format!(r"(?<=a(?!(b)\1{{{}}}))b", usize::MAX).as_str());
    check(&format!(
        "let r=new RegExp({source:?},'d'),a=r.exec('ab');a[1]===undefined&&a.indices[1]===undefined"
    ));
    let source = JsString::from(format!(r"(?<=a(?=()\1{{{}}}))b", usize::MAX).as_str());
    check(&format!(
        "let r=new RegExp({source:?},'d'),a=r.exec('ab');a[1]===''&&a.indices[1][0]===1"
    ));
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
            r"let marker=0,r=/(?<=a(?=(b)\1{2}))b/g;r.lastIndex=1;let text='b'.repeat(5000)+'abbb'",
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
fn variable_counts_capture_effects_choices_and_unicode_remain_pending() {
    for source in [
        r"/(?<=a(?=(b)\1{1,2}))b/.exec('abbb')",
        r"/(?<=(\2{1,2})(b))c/.exec('bbbc')",
        r"/(?<=a(?=(b)(\1){2,3}))b/.exec('abbb')",
        r"/(?<=(\2{2}|a)(b))c/.exec('bbbc')",
        r"/(?<=a(?=(b)\1{2}))b/u.exec('abbb')",
        r"/(?<=a(?=(b)\1{2}))b/v.exec('abbb')",
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
