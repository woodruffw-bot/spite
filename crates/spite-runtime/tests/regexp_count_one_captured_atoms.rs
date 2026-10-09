//! Count-one scalar reference children preserve their original capture effects.
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
fn count_one_captured_atoms_result_snapshot() {
    let mut rows = String::new();
    for (source, flags, text) in [
        (r"(?<=(a)(\1){1})b", "d", "ab"),
        (r"(?<=(a)(\1){1}?)b", "d", "ab"),
        (r"(?<=(\2){1}(a))b", "d", "aab"),
        (r"(?<=(\1){1}a)b", "d", "ab"),
        (r"(a)(?=(\1){1}b)", "d", "aab"),
        (r"(?=(a)(\1){1})a", "d", "aa"),
        (r"(?!(a)(\1){1})b", "d", "b"),
        (r"(?<!(a)(\1){1})b", "d", "b"),
        (r"(?<=a(?=(b)(\1){1}))b", "d", "abb"),
        (r"((\2){1}(b))+", "d", "bbb"),
        (r"((b)(\2){1})+", "d", "bbbb"),
        (r"(?<=((b)(\2){1}){2})c", "d", "bbc"),
        (r"(?<=((\3){1}(b)){2})c", "d", "bbbbc"),
        (r"(a)((\1){1}b)+", "d", "aabab"),
        (r"(?<=(a)(()\1){1})b", "d", "ab"),
        (r"(?<=(a)(\1()){1})b", "d", "ab"),
        (r"(?<=(?<x>a)(?<y>\k<x>){1})b", "d", "ab"),
        (r"(?<=(?<x>\k<y>){1}(?<y>a))b", "d", "aab"),
        (r"(?<=a(?=(?<x>b)(?<y>\k<x>){1}))b", "d", "abb"),
        (r"(a)((\1){1}b)+", "di", "aAbab"),
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
fn original_gap_capture_effects_names_and_both_directions_keep_exact_ranges() {
    assert!(matches!(
        Realm::default().eval(r"/(?<=a(?=(b)(\1){1}))b/.exec('abb')"),
        Ok(Value::Object(_))
    ));
    check(
        r"let a=/(?<=a(?=(b)(\1){1}))b/d.exec('abb');a.index===1&&a[1]==='b'&&a[2]==='b'&&a.indices[1][0]===1&&a.indices[2][0]===2",
    );
    check(
        r"let a=/(?<=(?<x>a)(?<y>\k<x>){1})b/d.exec('ab');a.groups.x==='a'&&a.groups.y===''&&a.indices.groups.y===a.indices[2]&&a.indices[2][0]===1",
    );
    check(
        r"let a=/(?<=(?<x>\k<y>){1}(?<y>a))b/d.exec('aab');a.index===2&&a.groups.x==='a'&&a.groups.y==='a'&&a.indices.groups.x[0]===0&&a.indices.groups.y[0]===1",
    );
    check(r"let a=/(?<=(\1){1}a)b/d.exec('ab');a[1]===''&&a.indices[1][0]===0");
    check(
        r"let a=/(?<=(a)(()\1){1})b/d.exec('ab');a[2]===''&&a[3]===''&&a.indices[2][0]===1&&a.indices[3][0]===1",
    );
    check(
        r"let a=/(?<!(a)(\1){1})b/d.exec('b');a[1]===undefined&&a[2]===undefined&&a.indices[2]===undefined",
    );
    check(
        r"let a=/([\uD800])(?=(\1){1}b)/d.exec('\uD800\uD800b');a[0].length===1&&a.indices[2][0]===1",
    );
}
#[test]
fn consumers_empty_global_sticky_advancement_and_callbacks_preserve_child_slots() {
    check(
        r"let r=/(?<=(a)(\1){1})b/dy;r.lastIndex=1;let a=r.exec('ab');a[2]===''&&r.lastIndex===2&&r.exec('ab')===null&&r.lastIndex===0",
    );
    check(
        r"let r=/(?<=(a)(\1){1})b/dg,a=[...'ab ab'.matchAll(r)];a.length===2&&a[0].index===1&&a[1].index===4&&a[1].indices[2][0]===4&&r.lastIndex===0",
    );
    check(
        r"let a=[...'aaa'.matchAll(/(?=(a)(\1){1})/dg)];a.length===2&&a[0][2]==='a'&&a[1].indices[2][0]===2",
    );
    check(
        r"let seen=[];let s='ab ab'.replace(/(?<=(a)(\1){1})b/g,(m,x,y,i)=>{seen.push(x,y,i);return '_'});s==='a_ a_'&&seen.join('|')==='a||1|a||4'",
    );
    check(
        r"'ab'.search(/(?<=(a)(\1){1})b/)===1&&'ab'.split(/(?<=(a)(\1){1})b/).join('|')==='a|a||'",
    );
}
#[test]
fn deep_child_effects_clones_collection_and_long_search_stay_unlimited() {
    let mut realm = Realm::default();
    realm.eval(r"let n=100000,r=new RegExp('(?<=(a)'+'('.repeat(n)+'\\1'+')'.repeat(n)+'{1})b','d'),copy=new RegExp(r)").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("let a=copy.exec('ab');a.length===n+2&&a.index===1&&a[1]==='a'&&a[n+1]===''&&a.indices[n+1][0]===1&&copy.source===r.source"),Ok(Value::Boolean(true)));
    assert_eq!(realm.eval(r"let long=/(?<=(a)(\1){1})b/d.exec('c'.repeat(10000)+'ab');long.index===10001&&long.indices[2][0]===10001"),Ok(Value::Boolean(true)));
    realm.collect(usize::MAX).unwrap();
}
#[test]
fn opted_in_work_abort_keeps_last_index_and_bypasses_handlers() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval(r"let marker=0,r=/(?<=(a)(\1){1})b/g;r.lastIndex=1;let text='c'.repeat(10000)+'ab'")
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
fn other_counts_partial_reference_spans_and_unicode_stay_pending() {
    for source in [
        r"/(?<=a(?=(b)(\1){2}))b/.exec('abbb')",
        r"/(?<=a(?=(b)(\1){1,2}))b/.exec('abbb')",
        r"/(?<=a(?=(b)(\1\1){1}))b/.exec('abbb')",
        r"/(?<=(a)(\1){1})b/u.exec('ab')",
        r"/(?<=(a)(\1){1})b/v.exec('ab')",
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
