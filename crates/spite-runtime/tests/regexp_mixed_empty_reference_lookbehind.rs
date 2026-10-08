//! Outside captures combined with proven empty same-unit backward reads.
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
fn mixed_empty_reference_lookbehind_result_snapshot() {
    let mut rows = String::new();
    for (source, flags, text) in [
        (r"(a)(?<=((b)\3\1){2})c", "d", "babac"),
        (r"(a)(?<=(b\1\2){2})c", "d", "babac"),
        (r"(a)(?<=(\1(b)\3){2})c", "d", "ababc"),
        (r"(a)(?<=(\1(a)\3){2})c", "d", "aaaac"),
        (r"(a)(?<=(\1(\2)){2})b", "d", "aab"),
        (r"(ab)(?<=((b)\3\1){2})c", "d", "babbabc"),
        (r"(a)(?<=(\3b()\1){2})c", "d", "babac"),
        (r"(a)(?<=((b)\3\1){2}?)c", "d", "babac"),
        (r"(a)(?<=((b)\3\1){0})c", "d", "ac"),
        (r"(a)(?<!((b)\3\1){2}q)c", "d", "baxac"),
        (r"(a)(?<=((b)\3\1\B){2})c", "d", "babac"),
        (r"()(?<=((())\3\1){2})b", "d", "b"),
        (r"(?<=((a)\2\3){2})(b)", "d", "aab"),
        (r"(a(?<=((a)\3\1){2}))b", "d", "aab"),
        (r"(a)(?<=((b)\3\1){2})c", "di", "bAbAc"),
        (r"(?<x>a)(?<=(?<y>(?<z>b)\k<z>\k<x>){2})c", "d", "babac"),
        (r"(?:(?<x>a)|(?<x>b))(?<=(c\k<x>\2){2})d", "d", "cbcbcd"),
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
fn names_aliases_open_future_and_closed_local_targets_keep_exact_ranges() {
    check(r"/(a)(?<=(b\1\2){2})c/.exec('ababc')===null");
    check(
        r"let a=/([\uD800])(?<=(([\uDC00])\3\1){2})c/d.exec('\uDC00\uD800\uDC00\uD800c');a.index===3&&a.indices[1][0]===3&&a.indices[2][0]===0&&a.indices[3][0]===0",
    );

    check(
        r"let a=/(?<x>a)(?<=(?<y>(?<z>b)\k<z>\k<x>){2})c/d.exec('babac');a.index===3&&a.groups.x==='a'&&a.groups.y==='ba'&&a.groups.z==='b'&&a.indices.groups.z===a.indices[3]&&a.indices[3][0]===0&&a.indices[1][0]===3",
    );
    check(
        r"let a=/(a)(?<=(b\1\2){2})c/d.exec('babac'),b=/(a)(?<=(\1(\2)){2})b/d.exec('aab');a[2]==='ba'&&b[2]==='a'&&b[3]===''&&b.indices[3][0]===1&&b.indices[1][0]===1",
    );
    check(
        r"let a=/(?<=((a)\2\3){2})(b)/d.exec('aab'),b=/(a(?<=((a)\3\1){2}))b/d.exec('aab');a.indices[1][0]===0&&a.indices[3][0]===2&&b.indices[1][0]===1&&b.indices[2][0]===0",
    );
    check(r"let a=/(?:(?<x>a)|(?<x>b))(?<=(c\k<x>\2){2})d/d.exec('cbcbcd');a===null");
    check(
        r"let a=/(?:(?<x>a)|(?<x>b))(?<=(c\k<x>\3){2})d/d.exec('cbcbd');a.index===3&&a[1]===undefined&&a[2]==='b'&&a[3]==='cb'&&a.groups.x==='b'&&a.indices.groups.x===a.indices[2]",
    );
}

#[test]
fn consumers_last_index_callbacks_and_skipped_capture_effects_are_preserved() {
    check(
        r"let r=/(a)(?<=((b)\3\1){2})c/dy;r.lastIndex=3;let a=r.exec('babac');a.index===3&&r.lastIndex===5&&r.exec('babac')===null&&r.lastIndex===0",
    );
    check(
        r"let r=/(a)(?<=((b)\3\1){2})c/dg,a=[...'babac babac'.matchAll(r)];a.length===2&&a[1].index===9&&a[1].indices[2][0]===6&&r.lastIndex===0",
    );
    check(
        r"let a=/(a)(?<=((b)\3\1){0})c/d.exec('ac');a[1]==='a'&&a[2]===undefined&&a[3]===undefined&&a.indices[2]===undefined",
    );
    check(
        r"let a=[...'ab'.matchAll(/()(?<=((())\3\1){2})/dg)];a.length===3&&a[2].index===2&&a[2].indices[4][0]===2",
    );
    check(
        r"let seen=[];let s='babac babac'.replace(/(a)(?<=((b)\3\1){2})c/g,(m,x,y,z,i)=>{seen.push(x,y,z,i);return '_'});s==='bab_ bab_'&&seen.join('|')==='a|ba|b|3|a|ba|b|9'",
    );
    check(
        r"'babac'.search(/(a)(?<=((b)\3\1){2})c/)===3&&'babac'.split(/(a)(?<=((b)\3\1){2})c/).join('|')==='bab|a|ba|b|'",
    );
}

#[test]
fn deep_owned_scopes_clones_collection_huge_empty_counts_and_negative_undo_stay_unlimited() {
    let mut realm = Realm::default();
    realm.eval(r"let n=100000,r=new RegExp('(a)(?<=('+'('.repeat(n)+'b'+')'.repeat(n)+'\\3\\1){2})c','d'),copy=new RegExp(r)").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("let a=copy.exec('babac');a.index===3&&a.length===n+3&&a[1]==='a'&&a[2]==='ba'&&a[n+2]==='b'&&a.indices[n+2][0]===0&&copy.source===r.source"),Ok(Value::Boolean(true)));
    assert_eq!(realm.eval(r"let neg=new RegExp('(a)(?<!('+'('.repeat(n)+'b'+')'.repeat(n)+'\\3\\1){2}q)c','d'),b=neg.exec('babaac');b[1]==='a'&&b[2]===undefined&&b[n+2]===undefined"),Ok(Value::Boolean(true)));
    assert_eq!(realm.eval(r"let huge=new RegExp('()(?<=((())\\3\\1){'+'9'.repeat(100)+'})b','d'),h=huge.exec('b');h[4]===''&&h.indices[4][0]===0"),Ok(Value::Boolean(true)));
    assert_eq!(realm.eval(r"let long=/(a)(?<=((b)\3\1){50000})c/dy;long.lastIndex=99999;let found=long.exec('ba'.repeat(50000)+'c');found.index===99999&&found.indices[1][0]===99999&&found.indices[2][0]===0&&found.indices[3][0]===0"),Ok(Value::Boolean(true)));
    realm.collect(usize::MAX).unwrap();
}

#[test]
fn explicit_work_aborts_preserve_last_index_and_bypass_handlers() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval(
            r"let marker=0,r=/(a)(?<=((b)\3\1){2})c/g;r.lastIndex=1;let text='ba'.repeat(5000)+'c'",
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
fn variable_counts_dependent_mixed_targets_choices_and_unicode_remain_pending() {
    for source in [
        r"/(a)(?<=((b)\3\1){1,2})c/.exec('babac')",
        r"/(a)(?<=(\1\3(b)){2})c/.exec('ababc')",
        r"/(a)(?<=((b)\3\1){2}|b)c/.exec('babac')",
        r"/(a)(?<=((b)\3\1){2})c/u.exec('babac')",
        r"/(a)(?<=((b)\3\1){2})c/v.exec('babac')",
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
