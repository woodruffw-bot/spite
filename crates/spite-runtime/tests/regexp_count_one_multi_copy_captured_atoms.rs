//! Count-one reference atoms preserve full capture spans and empty boundaries.
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
fn count_one_multi_copy_captured_atoms_result_snapshot() {
    let mut rows = String::new();
    for (source, flags, text) in [
        (r"(a)(\1\1){1}", "d", "aaa"),
        (r"(a)(()\1\1){1}", "d", "aaa"),
        (r"(a)(\1\1()){1}", "d", "aaa"),
        (r"(a)((\1\1)){1}", "d", "aaa"),
        (r"(a)((\1\1){1}b)+", "d", "aaabaab"),
        (r"(?=(a)(\1\1){1})a", "d", "aaa"),
        (r"(?<=(\2\2){1}(a))b", "d", "aaab"),
        (r"(?<=(\3\3()){1}(a))b", "d", "aaab"),
        (r"(?<=a(?=(b)(\1\1){1}))b", "d", "abbb"),
        (r"(?<=a(?=(b)(\1\1){1}?))b", "d", "abbb"),
        (r"(?<=a(?=(b)(\1\1){1}))b", "d", "abb"),
        (r"(?<=a(?=(b)(()\1\1){1}))b", "d", "abbb"),
        (r"(?<=a(?=(b)(\1\1()){1}))b", "d", "abbb"),
        (r"(?!(a)(\1\1){1})b", "d", "b"),
        (r"(?<!(\2\2){1}(a))b", "d", "b"),
        (r"((\3\3)+(b))*", "d", "bbb"),
        (r"((\3\3)+(b))*?", "d", "bbb"),
        (r"((\1\1){2})*a()", "d", "a"),
        (r"(?<x>a)(?<y>\k<x>\k<x>){1}", "d", "aaa"),
        (r"(?<=(?<x>\k<y>\k<y>){1}(?<y>a))b", "d", "aaab"),
        (r"(?<x>a)(?<y>()\k<x>\k<x>){1}", "di", "aAA"),
        (r"(a)(\1\1){1}", "d", "aa"),
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
fn full_spans_boundary_aliases_and_original_program_keep_values() {
    check(
        r"let a=/(?<x>a)(?<y>\k<x>\k<x>){1}/d.exec('aaa');a.groups.x==='a'&&a.groups.y==='aa'&&a.indices.groups.y===a.indices[2]&&a.indices[2][0]===1&&a.indices[2][1]===3",
    );
    check(r"let a=/(a)(()\1\1){1}/d.exec('aaa');a[2]==='aa'&&a[3]===''&&a.indices[3][0]===1");
    check(r"let a=/(a)(\1\1()){1}/d.exec('aaa');a[3]===''&&a.indices[3][0]===3");
    check(
        r"let a=/(?<=(\3\3()){1}(a))b/d.exec('aaab');a[1]==='aa'&&a[2]===''&&a[3]==='a'&&a.indices[2][0]===2",
    );
    check(
        r"let a=/(?<=a(?=(b)(\1\1){1}))b/d.exec('abbb');a.index===1&&a[1]==='b'&&a[2]==='bb'&&a.indices[2][0]===2&&a.indices[2][1]===4",
    );
    assert!(matches!(
        Realm::default().eval(r"/(?<=a(?=(b)(\1\1){1}))b/.exec('abbb')"),
        Ok(Value::Object(_))
    ));
    check(
        r"let a=/((\1\1){2})*a()/d.exec('a');a[1]===undefined&&a[2]===undefined&&a[3]===''&&a.indices[3][0]===1",
    );
    check(
        r"let a=/([\uD800])(\1\1){1}/d.exec('\uD800\uD800\uD800');a[2].length===2&&a.indices[2][1]===3",
    );
}
#[test]
fn consumers_sticky_callbacks_and_full_capture_ranges_agree() {
    check(
        r"let r=/(a)(\1\1){1}/dy;r.lastIndex=1;let a=r.exec('caaa');a[2]==='aa'&&a.indices[2][0]===2&&r.lastIndex===4&&r.exec('caaa')===null&&r.lastIndex===0",
    );
    check(
        r"let r=/(a)(\1\1){1}/dg,a=[...'aaa aaa'.matchAll(r)];a.length===2&&a[1].index===4&&a[1].indices[2][1]===7&&r.lastIndex===0",
    );
    check(
        r"let seen=[];let s='aaa aaa'.replace(/(a)(\1\1){1}/g,(m,x,y,i)=>{seen.push(x,y,i);return '_'});s==='_ _'&&seen.join('|')==='a|aa|0|a|aa|4'",
    );
    check(r"'caaa'.search(/(a)(\1\1){1}/)===1&&'aaa'.split(/(a)(\1\1){1}/).join('|')==='|a|aa|'");
}
#[test]
fn deep_full_capture_effects_clones_collection_and_long_search_stay_unlimited() {
    let mut realm = Realm::default();
    realm.eval(r"let n=100000,r=new RegExp('(a)'+'('.repeat(n)+'\\1\\1'+')'.repeat(n)+'{1}','d'),copy=new RegExp(r)").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("let a=copy.exec('aaa');a.length===n+2&&a[1]==='a'&&a[n+1]==='aa'&&a.indices[n+1][0]===1&&a.indices[n+1][1]===3&&copy.source===r.source"),Ok(Value::Boolean(true)));
    assert_eq!(realm.eval(r"let long=/(a)(\1\1){1}/d.exec('c'.repeat(10000)+'aaa');long.index===10000&&long.indices[2][1]===10003"),Ok(Value::Boolean(true)));
    realm.collect(usize::MAX).unwrap();
}
#[test]
fn opted_in_work_abort_keeps_last_index_and_bypasses_handlers() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval(r"let marker=0,r=/(a)(\1\1){1}/g;r.lastIndex=1;let text='c'.repeat(10000)+'aaa'")
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
fn partial_consuming_spans_optional_counts_and_unicode_stay_pending() {
    for source in [
        r"/(?<=a(?=(b)(?:(\1)\1){1}))b/.exec('abbb')",
        r"/(?<=a(?=(b)(\1\1){2}))b/.exec('abbbbb')",
        r"/(?<=a(?=(b)(\1\1){0,2}))b/.exec('abbbbb')",
        r"/(?<=(a)(\1\1){1})b/u.exec('ab')",
        r"/(?<=(a)(\1\1){1})b/v.exec('ab')",
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
