//! Once-counted scalar atoms retain original partial capture boundaries.
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
fn count_one_partial_multi_copy_captured_atoms_result_snapshot() {
    let mut rows = String::new();
    for (source, flags, text) in [
        ("(?<=a(?=(b)(\\1(\\1)){1}))b", "d", "abbb"),
        ("(?<=a(?=(b)((\\1)\\1){1}))b", "d", "abbb"),
        ("(?<=a(?=(b)(\\1()\\1){1}))b", "d", "abbb"),
        ("(?<=a(?=(b)(\\1(\\1)\\1){1}))b", "d", "abbbb"),
        ("(?<=a(?=(b)(\\1(\\1\\1)\\1){1}))b", "d", "abbbbb"),
        ("(?<=(\\3(\\3)){1}(a))b", "d", "aaab"),
        ("(?<=((\\3)\\3){1}(a))b", "d", "aaab"),
        ("(?<=(\\3()\\3){1}(a))b", "d", "aaab"),
        ("(a)(?=(\\1(\\1)){1})a", "d", "aaa"),
        ("(a)((\\1(\\1)){1}b)+", "d", "aaabaab"),
        ("(a)(((\\1)\\1){1}b)+", "d", "aaabaab"),
        ("(a)((\\1()\\1){1}b)+", "d", "aaabaab"),
        ("(?=(\\3(\\3)){1}(a))a", "d", "a"),
        ("(?<=(a)(\\1(\\1)){1})b", "d", "ab"),
        ("(?<=a(?!(b)(\\1(\\1)){1}))b", "d", "ab"),
        ("(a)(?!(\\1(\\1)){1})b", "d", "ab"),
        ("(?<=a(?=(b)(?:(\\1)\\1){1}))b", "d", "abbb"),
        ("(?<=a(?=(?<x>b)(?<y>\\k<x>(?<z>\\k<x>)){1}))b", "d", "abbb"),
        ("(?<=a(?=(?<x>b)(?<y>\\k<x>()\\k<x>){1}))b", "d", "abbb"),
        ("(?<=(?<x>\\k<z>(?<y>\\k<z>)){1}(?<z>a))b", "d", "aaab"),
        ("(?<=(?<x>a)(?<y>\\k<x>(?<z>\\k<x>)){1})b", "d", "ab"),
        ("(?<=a(?!(?<x>b)(?<y>\\k<x>(?<z>\\k<x>)){1}))b", "d", "ab"),
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
fn partial_ranges_names_indices_original_programs_and_negative_rollback_are_exact() {
    check(
        "let a=/(?<=a(?=(?<x>b)(?<y>\\k<x>(?<z>\\k<x>)){1}))b/d.exec('abbb');a.groups.x==='b'&&a.groups.y==='bb'&&a.groups.z==='b'&&a.indices.groups.z===a.indices[3]&&a.indices[3][0]===3&&a.indices[3][1]===4",
    );
    check(
        "let a=/(?<=(?<x>\\k<z>(?<y>\\k<z>)){1}(?<z>a))b/d.exec('aaab');a.groups.x==='aa'&&a.groups.y==='a'&&a.indices[2][0]===1&&a.indices[3][0]===2",
    );
    check(
        "let a=/(?<=a(?=(b)(\\1()\\1){1}))b/d.exec('abbb');a[2]==='bb'&&a[3]===''&&a.indices[3][0]===3&&a.indices[3][1]===3",
    );
    check(
        "let a=/(?<=a(?!(b)(\\1(\\1)){1}))b/d.exec('ab');a[1]===undefined&&a[2]===undefined&&a.indices[3]===undefined",
    );
    check(
        "let a=/(?<=a(?=([\\uD800])(\\1(\\1)){1}))[\\uD800]/d.exec('a\\uD800\\uD800\\uD800');a.index===1&&a[2].length===2&&a[3].charCodeAt(0)===55296&&a.indices[3][0]===3",
    );
    assert!(matches!(
        Realm::default().eval(r"/(?<=a(?=(b)(?:(\1)\1){1}))b/.exec('abbb')"),
        Ok(Value::Object(_))
    ));
    assert!(matches!(
        Realm::default().eval(r"/(?<=a(?=(b)(\1(\1)){1}))b/.exec('abbb')"),
        Ok(Value::Object(_))
    ));
}

#[test]
fn consumers_sticky_global_callbacks_and_required_empty_paths_keep_partial_captures() {
    check(
        "let r=/(?<=a(?=(b)(\\1(\\1)){1}))b/dy;r.lastIndex=1;let a=r.exec('abbb');a[3]==='b'&&a.indices[3][0]===3&&r.lastIndex===2&&r.exec('abbb')===null&&r.lastIndex===0",
    );
    check(
        "let r=/(?<=a(?=(b)(\\1(\\1)){1}))b/dg,a=[...'abbb abbb'.matchAll(r)];a.length===2&&a[0].index===1&&a[1].index===6&&a[1].indices[3][0]===8&&r.lastIndex===0",
    );
    check(
        "let seen=[];let s='abbb abbb'.replace(/(?<=a(?=(b)(\\1(\\1)){1}))b/g,(m,x,y,z,i)=>{seen.push(x,y,z,i);return '_'});s==='a_bb a_bb'&&seen.join('|')==='b|bb|b|1|b|bb|b|6'",
    );
    check(
        "'abbb'.search(/(?<=a(?=(b)(\\1(\\1)){1}))b/)===1&&'abbb'.split(/(?<=a(?=(b)(\\1(\\1)){1}))b/).join('|')==='a|b|bb|b|bb'",
    );
    check(
        "let a=/(a)((\\1(\\1)){1}b)+/d.exec('aaabaab');a[2]==='aab'&&a[3]==='aa'&&a[4]==='a'&&a.indices[4][0]===5",
    );
    check("let a=/(?=(\\3(\\3)){1}(a))a/d.exec('a');a[1]===''&&a[2]===''&&a[3]==='a'");
    check("/(?<=a(?=(b)(\\1(\\1)){1}))b/.exec('abb')===null");
}

#[test]
fn deep_partial_capture_boundaries_clones_collection_and_long_search_stay_unlimited() {
    let mut realm = Realm::default();
    realm.eval(r"let n=100000,r=new RegExp('(?<=a(?=(b)(\\1'+'('.repeat(n)+'\\1'+')'.repeat(n)+'){1}))b','d'),copy=new RegExp(r)").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("let a=copy.exec('abbb');a.length===n+3&&a.index===1&&a[2]==='bb'&&a[n+2]==='b'&&a.indices[n+2][0]===3&&copy.source===r.source"),Ok(Value::Boolean(true)));
    assert_eq!(realm.eval(r"let long=/(?<=a(?=(b)(\1(\1)){1}))b/d.exec('c'.repeat(10000)+'abbb');long.index===10001&&long.indices[3][0]===10003"),Ok(Value::Boolean(true)));
    realm.collect(usize::MAX).unwrap();
}
#[test]
fn opted_in_work_abort_preserves_last_index_and_bypasses_handlers() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm.eval(r"let marker=0,r=/(?<=a(?=(b)(\1(\1)){1}))b/g;r.lastIndex=1;let text='c'.repeat(10000)+'abbb'").unwrap();
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
fn other_child_counts_optional_effects_and_unicode_remain_pending() {
    for source in [
        r"/(?<=a(?=(b)(\1(\1)){2}))b/.exec('abbbbbb')",
        r"/(?<=a(?=(b)(\1(\1)){0,1}))b/.exec('abbb')",
        r"/(?<=a(?=(b)(\1(\1)){1}))b/u.exec('abbb')",
        r"/(?<=a(?=(b)(\1(\1)){1}))b/v.exec('abbb')",
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
