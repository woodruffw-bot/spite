//! Stable imported captures in forward assertions inside ordinary lookbehind.
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
fn outside_lookahead_in_lookbehind_result_snapshot() {
    let mut rows = String::new();
    for (source, flags, text) in [
        (r"(a)(?<=a(?=\1))a", "d", "aa"),
        (r"(a)(?<=a(?=((\1)b){2}))a", "d", "aabab"),
        (r"(a)(?<=a(?=((\1)b){2}?))a", "d", "aabab"),
        (r"(a)(?<=a(?=((\1)\3){2}))a", "d", "aaaaa"),
        (r"(a)(?<=a(?!((\1)b){2}q))a", "d", "aabab"),
        (r"(a)(?<=a(?=((\1)b){0}))b", "d", "ab"),
        (r"(a)(?<=a(?=(?=(\1))\1))a", "d", "aa"),
        (r"(a)(?<=a(?=((?<=a)\1)))a", "d", "aa"),
        (r"(a)(?<=a(?=(\1)(?=\1)))a", "d", "aaa"),
        (r"(a)(?<=(a\1){2}(?=\1))b", "d", "aaaab"),
        (r"(a)(?<=a{2}(?=\1)\1)b", "d", "aaab"),
        (r"(a)(?<=(?=\1)\1)b", "d", "ab"),
        (r"(?<x>a)(?<=a(?=(?<y>(\k<x>)b){2}))a", "d", "aabab"),
        (r"(?<x>a)(?<=a(?=(?<y>(?<z>\k<x>)\k<z>){2}))a", "d", "aaaaa"),
        (r"(a)(?<=a(?=((\1)b){2}))a", "di", "aABab"),
        (r"(a)?(?<=a(?=(\1)))a", "d", "aa"),
        (r"()(a)(?<=a(?=(\1\2)))a", "d", "aa"),
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
fn imported_named_ranges_local_dependencies_negative_rollback_and_surrogates_are_exact() {
    check(
        r"let a=/(?<x>a)(?<=a(?=(?<y>(?<z>\k<x>)\k<z>){2}))a/d.exec('aaaaa');a.groups.x==='a'&&a.groups.y==='aa'&&a.groups.z==='a'&&a.indices.groups.z===a.indices[3]&&a.indices[2][0]===3&&a.indices[3][0]===3",
    );
    check(
        r"let a=/(a)(?<=a(?!((\1)b){2}q))a/d.exec('aabab');a[1]==='a'&&a[2]===undefined&&a[3]===undefined&&a.indices[2]===undefined",
    );
    check(
        r"let a=/(a)(?<=a(?=((\1)b){0}))b/d.exec('ab');a[1]==='a'&&a[2]===undefined&&a[3]===undefined",
    );
    check(
        r"let a=/([\uD800])(?<=[\uD800](?=((\1)\3){2}))[\uD800]/d.exec('\uD800\uD800\uD800\uD800\uD800\uDC00');a.index===0&&a.indices[2][0]===3&&a.indices[3][0]===3",
    );
    check(r"let a=/(a)(?<=a(?=(?=(\1))\1))a/d.exec('aa');a[2]==='a'&&a.indices[2][0]===1");
}

#[test]
fn consumers_sticky_global_empty_advance_and_callbacks_keep_imported_and_future_ranges() {
    check(
        r"let a=[...'aabab'.matchAll(/(?=(a))(?<=a(?=((\1)b){2}))/dg)];a.length===1&&a[0].index===1&&a[0][0]===''&&a[0].indices[2][0]===3",
    );
    check(
        r"let r=/(a)(?<=a(?=((\1)b){2}))a/dy;let a=r.exec('aabab');a.index===0&&r.lastIndex===2&&r.exec('aabab')===null&&r.lastIndex===0",
    );
    check(
        r"let r=/(a)(?<=a(?=((\1)b){2}))a/dg,a=[...'aabab aabab'.matchAll(r)];a.length===2&&a[1].index===6&&a[1].indices[2][0]===9&&r.lastIndex===0",
    );
    check(
        r"let a=[...'aabab'.matchAll(/()(a)(?<=a(?=((\2)b){2}))/dg)];a.length===1&&a[0].index===0&&a[0].indices[3][0]===3",
    );
    check(
        r"let seen=[];let s='aabab aabab'.replace(/(a)(?<=a(?=((\1)b){2}))a/g,(m,x,y,z,i)=>{seen.push(x,y,z,i);return '_'});s==='_bab _bab'&&seen.join('|')==='a|ab|a|0|a|ab|a|6'",
    );
    check(
        r"'aabab'.search(/(a)(?<=a(?=((\1)b){2}))a/)===0&&'aabab'.split(/(a)(?<=a(?=((\1)b){2}))a/).join('|')==='|a|ab|a|bab'",
    );
}

#[test]
fn deep_capture_scopes_clones_gc_nested_assertions_and_huge_empty_counts_are_unlimited() {
    let mut realm = Realm::default();
    realm.eval(r"let n=100000,r=new RegExp('(a)(?<=a(?=('+'('.repeat(n)+'\\1'+')'.repeat(n)+'b){2}))a','d'),copy=new RegExp(r)").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("let a=copy.exec('aabab');a.length===n+3&&a[1]==='a'&&a[2]==='ab'&&a[n+2]==='a'&&a.indices[n+2][0]===3&&r.source===copy.source"),Ok(Value::Boolean(true)));
    assert_eq!(realm.eval(r"let neg=new RegExp('(a)(?<=a(?!('+'('.repeat(n)+'\\1'+')'.repeat(n)+'b){2}q))a','d'),b=neg.exec('aabab');b[1]==='a'&&b[2]===undefined&&b[n+2]===undefined"),Ok(Value::Boolean(true)));
    assert_eq!(realm.eval(r"let huge=new RegExp('()(?<=a(?=((\\1)){'+'9'.repeat(100)+'}))b','d'),h=huge.exec('ab');h[3]===''&&h.indices[3][0]===1"),Ok(Value::Boolean(true)));
    assert_eq!(realm.eval(r"let nested=new RegExp('(a)(?<=a'+'(?='.repeat(10000)+'\\1'+')'.repeat(10000)+')a');nested.test('aa')"),Ok(Value::Boolean(true)));
    assert_eq!(realm.eval(r"let long=/(a)(?<=a(?=((\1)b){50000}))a/d;let found=long.exec('a'+'ab'.repeat(50000));found.indices[2][0]===99999&&found.indices[3][0]===99999"),Ok(Value::Boolean(true)));
    realm.collect(usize::MAX).unwrap();
}

#[test]
fn opted_in_work_aborts_keep_last_index_and_bypass_handlers() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm.eval(r"let marker=0,r=/(a)(?<=a(?=((\1)b){2}))a/g;r.lastIndex=1;let text='b'.repeat(5000)+'aabab'").unwrap();
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
fn changing_owner_targets_variable_counts_choices_and_unicode_remain_pending() {
    for source in [
        r"/(?<=a(?=(b)\1))b/.exec('abb')",
        r"/(a)(?<=a(?=((\1)b){1,2}))a/.exec('aabab')",
        r"/(a)(?<=a(?=(\1|b)))a/.exec('aa')",
        r"/(a)(?<=a(?=(\1)(?=\2)))a/.exec('aaa')",
        r"/(a)(?<=a(?=(\1)(?<=a(?=\2))))a/.exec('aaa')",
        r"/(a)(?<=a(?=\2))(b)/.exec('ab')",
        r"/(a)(?<=a(?=\1))a/u.exec('aa')",
        r"/(a)(?<=a(?=\1))a/v.exec('aa')",
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
