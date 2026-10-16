//! Exact positive child counts retain final-iteration capture ranges.
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
fn exact_count_captured_atoms_result_snapshot() {
    let mut rows = String::new();
    for (source, flags, text) in [
        ("(?<=a(?=(b)(\\1){2}))b", "d", "abbb"),
        ("(?<=a(?=(b)(\\1){3}))b", "d", "abbbb"),
        ("(?<=a(?=(b)(\\1\\1){2}))b", "d", "abbbbb"),
        ("(?<=a(?=(b)(\\1(\\1)){2}))b", "d", "abbbbb"),
        ("(?<=a(?=(b)((\\1)\\1){2}))b", "d", "abbbbb"),
        ("(?<=a(?=(b)(\\1()\\1){2}))b", "d", "abbbbb"),
        ("(?<=a(?=(b)(\\1(\\1\\1)\\1){2}))b", "d", "abbbbbbbbb"),
        ("(?<=(\\3(\\3)){2}(a))b", "d", "aaaaab"),
        ("(?<=((\\3)\\3){2}(a))b", "d", "aaaaab"),
        ("(?<=(\\3()\\3){2}(a))b", "d", "aaaaab"),
        ("(a)(?=(\\1){2})a", "d", "aaa"),
        ("(a)((\\1){2}b)+", "d", "aaabaab"),
        ("(a)((\\1(\\1)){2}b)+", "d", "aaaaabaaaab"),
        ("(?=(\\3(\\3)){2}(a))a", "d", "a"),
        ("(?<=(a)(\\1(\\1)){2})b", "d", "ab"),
        ("(?<=a(?!(b)(\\1(\\1)){2}))b", "d", "ab"),
        ("(?<=a(?=(b)(?:(\\1)\\1){2}))b", "d", "abbbbb"),
        ("(?<=a(?=(?<x>b)(?<y>\\k<x>){2}))b", "d", "abbb"),
        (
            "(?<=a(?=(?<x>b)(?<y>\\k<x>(?<z>\\k<x>)){2}))b",
            "d",
            "abbbbb",
        ),
        ("(?<=(?<x>\\k<z>(?<y>\\k<z>)){2}(?<z>a))b", "d", "aaaaab"),
        ("(?<=(?<x>a)(?<y>\\k<x>(?<z>\\k<x>)){2})b", "d", "ab"),
        ("(?<=a(?!(?<x>b)(?<y>\\k<x>(?<z>\\k<x>)){2}))b", "d", "ab"),
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
fn final_iteration_ranges_names_indices_original_programs_and_rollback_are_exact() {
    check(
        "let a=/(?<=a(?=(?<x>b)(?<y>\\k<x>){2}))b/d.exec('abbb');a.groups.x==='b'&&a.groups.y==='b'&&a.indices.groups.y===a.indices[2]&&a.indices[2][0]===3",
    );
    check(
        "let a=/(?<=a(?=(?<x>b)(?<y>\\k<x>(?<z>\\k<x>)){2}))b/d.exec('abbbbb');a.groups.y==='bb'&&a.groups.z==='b'&&a.indices.groups.z===a.indices[3]&&a.indices[2][0]===4&&a.indices[3][0]===5",
    );
    check(
        "let a=/(?<=(?<x>\\k<z>(?<y>\\k<z>)){2}(?<z>a))b/d.exec('aaaaab');a.groups.x==='aa'&&a.groups.y==='a'&&a.indices[2][0]===1&&a.indices[3][0]===4",
    );
    check(
        "let a=/(?<=a(?=(b)(\\1()\\1){2}))b/d.exec('abbbbb');a[2]==='bb'&&a[3]===''&&a.indices[3][0]===5&&a.indices[3][1]===5",
    );
    check(
        "let a=/(?<=a(?!(b)(\\1(\\1)){2}))b/d.exec('ab');a[1]===undefined&&a[2]===undefined&&a.indices[3]===undefined",
    );
    check(
        "let a=/(?<=a(?=([\\uD800])(\\1(\\1)){2}))[\\uD800]/d.exec('a\\uD800\\uD800\\uD800\\uD800\\uD800');a.index===1&&a[2].length===2&&a[3].charCodeAt(0)===55296&&a.indices[3][0]===5",
    );
    check(
        "let a=/(?<=a(?=(bc)(\\1(\\1)){2}))b/d.exec('a'+'bc'.repeat(5));a.index===1&&a.indices[2][0]===7&&a.indices[2][1]===11&&a.indices[3][0]===9&&a.indices[3][1]===11",
    );
    check(
        "let a=/(?<=(\\3(\\3)){2}(bc))a/d.exec('bc'.repeat(5)+'a');a.index===10&&a.indices[1][0]===0&&a.indices[1][1]===4&&a.indices[2][0]===2&&a.indices[2][1]===4&&a.indices[3][0]===8",
    );
    check(
        "let a=/(?<=a(?=(bc)(\\1){3}))b/d.exec('a'+'bc'.repeat(4));a.index===1&&a.indices[2][0]===7&&a.indices[2][1]===9",
    );
    check(
        "let a=/(?<=(\\2){3}(bc))a/d.exec('bc'.repeat(4)+'a');a.index===8&&a.indices[1][0]===0&&a.indices[1][1]===2&&a.indices[2][0]===6",
    );
    check(
        "let a=/(?<=a(?=(bc)(\\1()\\1){2}))b/d.exec('a'+'bc'.repeat(5));a.indices[3][0]===9&&a.indices[3][1]===9",
    );
    let mut rows = String::new();
    for (source, object) in [
        ("/(?<=a(?=(b)(?:(\\1)\\1){2}))b/.exec('abbb')", false),
        ("/(?<=a(?=(b)(\\1(\\1)){2}))b/.exec('abbb')", false),
        ("/(?<=a(?=(b)(\\1(\\1)){2}))b/.exec('abbbbbb')", true),
        ("/(?<=a(?=(b)(\\1){2}))b/.exec('abb')", false),
        ("/(?<=a(?=(b)(\\1){2}))b/.exec('abbb')", true),
        ("/(?<=a(?=(b)(\\1\\1){2}))b/.exec('abbbbb')", true),
    ] {
        let result = Realm::default().eval(source).unwrap();
        assert!(
            if object {
                matches!(result, Value::Object(_))
            } else {
                result == Value::Null
            },
            "{source}"
        );
        let program = format!(
            "JSON.stringify((()=>{{let a={source};return a===null?null:{{matches:[...a],index:a.index,input:a.input,groups:a.groups}}}})())"
        );
        let Value::String(result) = Realm::default().eval(&program).unwrap() else {
            panic!("expected JSON")
        };
        writeln!(rows, "{source:?} {result:?}").unwrap();
    }
    insta::assert_snapshot!("original_exact_count_programs", rows);
}

#[test]
fn consumers_sticky_global_callbacks_and_required_empty_paths_keep_last_effects() {
    check(
        "let r=/(?<=a(?=(b)(\\1(\\1)){2}))b/dy;r.lastIndex=1;let a=r.exec('abbbbb');a[3]==='b'&&a.indices[3][0]===5&&r.lastIndex===2&&r.exec('abbbbb')===null&&r.lastIndex===0",
    );
    check(
        "let r=/(?<=a(?=(b)(\\1(\\1)){2}))b/dg,a=[...'abbbbb abbbbb'.matchAll(r)];a.length===2&&a[0].index===1&&a[1].index===8&&a[1].indices[3][0]===12&&r.lastIndex===0",
    );
    check(
        "let seen=[];let s='abbbbb abbbbb'.replace(/(?<=a(?=(b)(\\1(\\1)){2}))b/g,(m,x,y,z,i)=>{seen.push(x,y,z,i);return '_'});s==='a_bbbb a_bbbb'&&seen.join('|')==='b|bb|b|1|b|bb|b|8'",
    );
    check(
        "'abbbbb'.search(/(?<=a(?=(b)(\\1(\\1)){2}))b/)===1&&'abbbbb'.split(/(?<=a(?=(b)(\\1(\\1)){2}))b/).join('|')==='a|b|bb|b|bbbb'",
    );
    check(
        "let a=/(a)((\\1(\\1)){2}b)+/d.exec('aaaaabaaaab');a[2]==='aaaab'&&a[3]==='aa'&&a[4]==='a'&&a.indices[4][0]===9",
    );
    check("let a=/(?=(\\3(\\3)){2}(a))a/d.exec('a');a[1]===''&&a[2]===''&&a[3]==='a'");
    check("/(?<=a(?=(b)(\\1){1000000}))b/.exec('abbb')===null");
}

#[test]
fn deep_final_iteration_effects_clones_collection_and_long_search_stay_unlimited() {
    let mut realm = Realm::default();
    realm.eval(r"let n=100000,r=new RegExp('(?<=a(?=(b)(\\1'+'('.repeat(n)+'\\1'+')'.repeat(n)+'){2}))b','d'),copy=new RegExp(r)").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("let a=copy.exec('abbbbb');a.length===n+3&&a.index===1&&a[2]==='bb'&&a[n+2]==='b'&&a.indices[n+2][0]===5&&copy.source===r.source"),Ok(Value::Boolean(true)));
    assert_eq!(realm.eval(r"let long=/(?<=a(?=(b)(\1(\1)){2}))b/d.exec('c'.repeat(10000)+'abbbbb');long.index===10001&&long.indices[3][0]===10005"),Ok(Value::Boolean(true)));
    for count in [usize::MAX.to_string(), "9".repeat(100)] {
        for source in [
            format!(
                r"let a=/(?<=(a)(\1(\1)){{{count}}})b/d.exec('ab');a[1]==='a'&&a[2]===''&&a[3]===''&&a.indices[3][0]===1"
            ),
            format!(
                r"let a=/(?=(\3(\3)){{{count}}}(a))a/d.exec('a');a[1]===''&&a[2]===''&&a[3]==='a'&&a.indices[2][0]===0"
            ),
        ] {
            assert_eq!(
                Realm::default().eval(&source),
                Ok(Value::Boolean(true)),
                "{source}"
            );
        }
    }
    realm.collect(usize::MAX).unwrap();
}
#[test]
fn opted_in_work_abort_preserves_last_index_and_bypasses_handlers() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm.eval(r"let marker=0,r=/(?<=a(?=(b)(\1(\1)){2}))b/g;r.lastIndex=1;let text='c'.repeat(10000)+'abbbbb'").unwrap();
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
fn variable_child_counts_optional_effects_and_unicode_remain_pending() {
    for source in [
        r"/(?<=a(?=(b)(\1(\1)){2,3}))b/.exec('abbbbbb')",
        r"/(?<=a(?=(b)(\1(\1)){0,2}))b/.exec('abbbbbb')",
        r"/(?<=a(?=(b)(\1(\1)){2}))b/u.exec('abbbbb')",
        r"/(?<=a(?=(b)(\1(\1)){2}))b/v.exec('abbbbb')",
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
