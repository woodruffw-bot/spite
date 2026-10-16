//! Proved empty counted reference terms in ordinary assertions.
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
fn undefined_counted_assertions_result_snapshot() {
    let mut rows = String::new();
    for (source, flags, text) in [
        (r"(?<=(\1{1,2}))a", "d", "a"),
        (r"(?<=(\1{1,2}?))a", "d", "a"),
        (r"(?<=(a)\1*?)b", "d", "ab"),
        (r"(?<=(\1*))a", "d", "a"),
        (r"(?<=(\1{0}))a", "d", "a"),
        (r"(?<=a(?=(\1{1,2}b)))b", "d", "ab"),
        (r"(?<=a(?=(\2*(b))))b", "d", "ab"),
        (r"(?<=a(?=(\2{1,2}(b))))b", "d", "ab"),
        (r"(?<=(a)\1{1,2})b", "d", "ab"),
        (r"(?<=(a)\1*)b", "d", "ab"),
        (r"(?<=(a)\1{0})b", "d", "ab"),
        (r"(?<=(\2{0})(b))c", "d", "bc"),
        (r"(?<=a(?=(b)\1{0}))b", "d", "ab"),
        (r"(?<=a(?!((\2{1,2})b)q))b", "d", "abbx"),
        (r"(a)(?<=(b)\2{1,2}\1)c", "d", "bac"),
        (r"(a)(?<=a(?=(\2*\1)))a", "d", "aa"),
        (r"(?<=(?<x>\k<x>{1,2}))a", "d", "a"),
        (r"(?<=(?<x>a)\k<x>*)b", "d", "ab"),
        (r"(?<=a(?=(\k<x>{1,2}(?<x>b))))b", "d", "ab"),
        (r"(?<=(\k<x>{0})(?<x>b))c", "d", "bc"),
        (r"(?<=(a)\1*)b", "di", "Ab"),
        (r"(?<!(\1{1,2}))a|a", "d", "a"),
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
fn named_alias_ranges_required_captures_negative_undo_and_consumers_are_exact() {
    check(
        r"let a=/(?<=(?<x>\k<x>{1,2}))a/d.exec('a');a[1]===''&&a.groups.x===''&&a.indices[1][0]===0&&a.indices.groups.x===a.indices[1]",
    );
    check(
        r"let a=/(?<=a(?=(\k<x>*(?<x>b))))b/d.exec('ab');a[1]==='b'&&a.groups.x==='b'&&a.indices[2][0]===1&&a.indices.groups.x===a.indices[2]",
    );
    check(r"let a=/(?<!(\1{1,2}))a|a/d.exec('a');a[1]===undefined&&a.indices[1]===undefined");
    check(
        r"let r=/(?<=(a)\1*)b/dy;r.lastIndex=1;let a=r.exec('ab');a[1]==='a'&&r.lastIndex===2&&r.exec('ab')===null&&r.lastIndex===0",
    );
    check(
        r"let r=/(?<=(\1{1,2}))a/dg,a=[...'a a'.matchAll(r)];a.length===2&&a[0][1]===''&&a[1].index===2&&a[1].indices[1][0]===2&&r.lastIndex===0",
    );
    check(
        r"let a=[...'a'.matchAll(/(?<=(\1*))/dg)];a.length===2&&a[0].index===0&&a[1].index===1&&a[1][1]===''",
    );
    check(
        r"let seen=[];let s='a a'.replace(/(?<=(\1{1,2}))a/g,(m,x,i)=>{seen.push(x,i);return '_'});s==='_ _'&&seen.join('|')==='|0||2'",
    );
    check(r"'ab'.search(/(?<=(a)\1*)b/)===1&&'ab'.split(/(?<=(a)\1*)b/).join('|')==='a|a|'");
    check(
        r"let a=/(?<=a(?=(\2*([\uD800]))))[\uD800]/d.exec('a\uD800\uDC00');a[1]===a[2]&&a.indices[2][0]===1&&a.indices[2][1]===2",
    );
}
#[test]
fn huge_bounds_deep_scopes_copies_and_collection_need_no_default_quota() {
    let mut realm = Realm::default();
    realm.eval(r"let n=100000,count='9'.repeat(100),r=new RegExp('(?<='+'('.repeat(n)+'\\1{'+count+'}'+')'.repeat(n)+')a','d'),copy=new RegExp(r)").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("let a=copy.exec('a');a.length===n+1&&a[1]===''&&a[n]===''&&a.indices[n][0]===0&&copy.source===r.source"),Ok(Value::Boolean(true)));
    assert_eq!(realm.eval(r"let neg=new RegExp('(?<!'+'('.repeat(n)+'\\1{'+count+'}'+')'.repeat(n)+'q)a','d'),b=neg.exec('a');b[1]===undefined&&b[n]===undefined"),Ok(Value::Boolean(true)));
    assert_eq!(realm.eval(r"let forward=new RegExp('(?<=a(?=('+'('.repeat(n)+'\\2{'+count+'}'+')'.repeat(n)+'b)))b','d'),c=forward.exec('ab');c.length===n+2&&c[1]==='b'&&c[n+1]===''&&c.indices[n+1][0]===1"),Ok(Value::Boolean(true)));
    assert_eq!(realm.eval(r"let local=new RegExp('(?<=(a)\\1{'+count+',})b','d'),e=local.exec('ab');e[1]==='a'&&e.indices[1][0]===0"),Ok(Value::Boolean(true)));
    realm.collect(usize::MAX).unwrap();
}
#[test]
fn explicit_work_abort_keeps_last_index_and_bypasses_language_handlers() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval(r"let marker=0,r=/(?<=(a)\1*)b/g;r.lastIndex=1;let text='b'.repeat(5000)+'ab'")
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
fn consuming_counts_choices_and_unicode_remain_pending_invalid_reads_stay_syntax_errors() {
    for source in [
        r"/(?<=(\1{1,2}a{1,2}))a/.exec('a')",
        r"/(?<=a(?=(b)\1{1,2}))b/.exec('abb')",
        r"/(?<=(\2{1,2})(b))c/.exec('bbc')",
        r"/(?<=(\1{1,2}|a))a/.exec('a')",
        r"/(?<=(\1{1,2}))a/u.exec('a')",
        r"/(?<=(\1{1,2}))a/v.exec('a')",
    ] {
        assert!(
            matches!(
                Realm::default().eval(source),
                Err(Error::Unsupported { .. })
            ),
            "{source}"
        );
    }
    check(
        r"let ok=false;try{new RegExp('(?<=\\9{0})a','u')}catch(e){ok=e instanceof SyntaxError}ok",
    );
}
