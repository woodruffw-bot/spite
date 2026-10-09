//! Stable imported captures in nested ordinary backward assertions.
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
fn nested_outside_lookbehind_result_snapshot() {
    let mut rows = String::new();
    for (source, flags, text) in [
        (r"(a)(?<=a(?<=\1))b", "d", "ab"),
        (r"(a)(?<=a(?<=(\1)))b", "d", "ab"),
        (r"(a)(?<=a(?<=((b)\1){2}))c", "d", "babac"),
        (r"(a)(?<=a(?<=((b)\1){2}?))c", "d", "babac"),
        (r"(a)(?<=\1(?<=(b\1){2}))c", "d", "babac"),
        (r"(a)(?<=a(?<!((b)\1){2}q))c", "d", "babac"),
        (r"(a)(?<=a(?<=((b)\1){0}))c", "d", "ac"),
        (r"(a)(?<=a(?=(?<=((b)\1){2})c))c", "d", "babac"),
        (r"(a)(?<=a(?<=(\3(b)\1){2}))c", "d", "bbabbac"),
        (r"(a)(?<=a(?<=(?<=\1)\1))b", "d", "ab"),
        (r"(a)(?<=(a\1){2}(?<=\1))b", "d", "aaaab"),
        (r"(a)(?<=a{2}(?<=\1))b", "d", "aab"),
        (r"(a)(?<=(?<=\1))b", "d", "ab"),
        (r"(?<x>a)(?<=a(?<=(?<y>(?<z>b)\k<x>){2}))c", "d", "babac"),
        (
            r"(?<x>a)(?<=a(?<=(?<y>\k<z>(?<z>b)\k<x>){2}))c",
            "d",
            "bbabbac",
        ),
        (r"(a)(?<=a(?<=((b)\1){2}))c", "di", "bAbAc"),
        (r"()(a)(?<=a(?<=(\1\2)))b", "d", "ab"),
        (r"(a)?(?<=a(?<=(\1)))b", "d", "ab"),
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
fn nested_names_backward_dependencies_negative_rollback_and_surrogates_keep_precise_ranges() {
    check(
        r"let a=/(?<x>a)(?<=a(?<=(?<y>(?<z>b)\k<x>){2}))c/d.exec('babac');a.groups.x==='a'&&a.groups.y==='ba'&&a.groups.z==='b'&&a.indices.groups.z===a.indices[3]&&a.indices[1][0]===3&&a.indices[2][0]===0",
    );
    check(
        r"let a=/(?<x>a)(?<=a(?<=(?<y>\k<z>(?<z>b)\k<x>){2}))c/d.exec('bbabbac');a.groups.y==='bba'&&a.indices.groups.z[0]===1",
    );
    check(
        r"let a=/(a)(?<=a(?!q)(?<!((b)\1){2}q))c/d.exec('babac');a[1]==='a'&&a[2]===undefined&&a[3]===undefined&&a.indices[2]===undefined",
    );
    check(
        r"let a=/(a)(?<=a(?<=((b)\1){0}))c/d.exec('ac');a[1]==='a'&&a[2]===undefined&&a[3]===undefined",
    );
    check(
        r"let a=/([\uD800])(?<=[\uD800](?<=(([\uDC00])\1){2}))c/d.exec('\uDC00\uD800\uDC00\uD800c');a.index===3&&a.indices[2][0]===0&&a.indices[3][0]===0",
    );
}
#[test]
fn consumers_sticky_global_empty_advance_and_callbacks_preserve_backward_child_positions() {
    check(
        r"let r=/(a)(?<=a(?<=((b)\1){2}))c/dy;r.lastIndex=3;let a=r.exec('babac');a.index===3&&r.lastIndex===5&&r.exec('babac')===null&&r.lastIndex===0",
    );
    check(
        r"let r=/(a)(?<=a(?<=((b)\1){2}))c/dg,a=[...'babac babac'.matchAll(r)];a.length===2&&a[1].index===9&&a[1].indices[2][0]===6&&r.lastIndex===0",
    );
    check(
        r"let a=[...'babac'.matchAll(/(?<=(a))(?<=a(?<=((b)\1){2}))/dg)];a.length===1&&a[0].index===4&&a[0][0]===''&&a[0].indices[2][0]===0",
    );
    check(
        r"let seen=[];let s='babac babac'.replace(/(a)(?<=a(?<=((b)\1){2}))c/g,(m,x,y,z,i)=>{seen.push(x,y,z,i);return '_'});s==='bab_ bab_'&&seen.join('|')==='a|ba|b|3|a|ba|b|9'",
    );
    check(
        r"'babac'.search(/(a)(?<=a(?<=((b)\1){2}))c/)===3&&'babac'.split(/(a)(?<=a(?<=((b)\1){2}))c/).join('|')==='bab|a|ba|b|'",
    );
}
#[test]
fn deep_scopes_clones_collection_nested_assertions_huge_empty_counts_and_long_units_are_unlimited()
{
    let mut realm = Realm::default();
    realm.eval(r"let n=100000,r=new RegExp('(a)(?<=a(?<=('+'('.repeat(n)+'b'+')'.repeat(n)+'\\1){2}))c','d'),copy=new RegExp(r)").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("let a=copy.exec('babac');a.length===n+3&&a[1]==='a'&&a[2]==='ba'&&a[n+2]==='b'&&a.indices[n+2][0]===0&&copy.source===r.source"),Ok(Value::Boolean(true)));
    assert_eq!(realm.eval(r"let neg=new RegExp('(a)(?<=a(?<!('+'('.repeat(n)+'b'+')'.repeat(n)+'\\1){2}q))c','d'),b=neg.exec('babac');b[1]==='a'&&b[2]===undefined&&b[n+2]===undefined"),Ok(Value::Boolean(true)));
    assert_eq!(realm.eval(r"let huge=new RegExp('()(?<=a(?<=((\\1)){'+'9'.repeat(100)+'}))b','d'),h=huge.exec('ab');h[3]===''&&h.indices[3][0]===1"),Ok(Value::Boolean(true)));
    assert_eq!(realm.eval(r"let nested=new RegExp('(a)(?<=a'+'(?<='.repeat(10000)+'\\1'+')'.repeat(10000)+')b');nested.test('ab')"),Ok(Value::Boolean(true)));
    assert_eq!(realm.eval(r"let long=/(a)(?<=a(?<=((b)\1){50000}))c/dy;long.lastIndex=99999;let found=long.exec('ba'.repeat(50000)+'c');found.indices[2][0]===0&&found.indices[3][0]===0"),Ok(Value::Boolean(true)));
    realm.collect(usize::MAX).unwrap();
}
#[test]
fn opted_in_work_abort_preserves_last_index_and_bypasses_handlers() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm.eval(r"let marker=0,r=/(a)(?<=a(?<=((b)\1){2}))c/g;r.lastIndex=1;let text='b'.repeat(5000)+'babac'").unwrap();
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
fn changing_owner_captures_variable_counts_choices_and_unicode_remain_pending() {
    for source in [
        r"/(a)(?<=a(?<=((b)\1){1,2}))c/.exec('babac')",
        r"/(a)(?<=a(?<=(\1|b)))c/.exec('ac')",
        r"/(a)(?<=a(?<=\2))(b)/.exec('ab')",
        r"/(a)(?<=((b)(?<=\2)))c/.exec('abc')",
        r"/(a)(?<=a(?=(\1)(?<=\2)))a/.exec('aaa')",
        r"/(a)(?<=a(?<=\1))b/u.exec('ab')",
        r"/(a)(?<=a(?<=\1))b/v.exec('ab')",
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
