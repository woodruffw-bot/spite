//! Exact backward units with outside captures and dependent right-hand ranges.
use spite_core::JsString;
use spite_runtime::{Error, Limits, Realm, Value};
use std::fmt::Write;

use crate::common::assert_true;

#[test]
fn dependent_outside_lookbehind_result_snapshot() {
    let mut rows = String::new();
    for (source, flags, text) in [
        (r"(a)(?<=(\3(b)\1){2})c", "d", "bbabbac"),
        (r"(a)(?<=(\3(b)\1){2}?)c", "d", "bbabbac"),
        (r"(ab)(?<=(\3(\1)){2})c", "d", "ababababc"),
        (r"(a+)(?<=(\3(\1)){2})b", "d", "aaaaaaaab"),
        (r"(a)(?<=(\3(\4(b))\1){2})c", "d", "bbbbabbbbac"),
        (r"(a)(?<=(\3(b)\1\3){2})c", "d", "bbabbac"),
        (r"(a)(?<=(\1\3(b)){2})c", "d", "ababc"),
        (r"(a)(?<!(\3(b)\1){2}q)c", "d", "bbabbaac"),
        (r"(a)(?<=(\3(\1)){0})b", "d", "ab"),
        (r"()(?<=(\3(\1)){2})b", "d", "b"),
        (r"(?<=(\2(\3)\3){2})(a)", "d", "a"),
        (r"(a(?<=(\3(\1)){2}))b", "d", "ab"),
        (r"(a)(?<=(\3(b)\1){2})c", "di", "bBAbbAc"),
        (r"(?<x>a)(?<=(?<y>\k<z>(?<z>b)\k<x>){2})c", "d", "bbabbac"),
        (r"(?<x>ab)(?<=(?<y>\k<z>(?<z>\k<x>)){2})c", "d", "ababababc"),
        (r"(?:(?<x>a)|(?<x>b))(?<=(\4(\k<x>)){2})c", "d", "bbbbc"),
        (r"(a+)(?<=(\3(\1)){2})b", "d", "aaaab"),
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
    insta::assert_snapshot!(rows, @r#"
        "(a)(?<=(\u005c3(b)\u005c1){2})c" flags="d" input="bbabbac" "{\u0022matches\u0022:[\u0022ac\u0022,\u0022a\u0022,\u0022bba\u0022,\u0022b\u0022],\u0022index\u0022:5,\u0022input\u0022:\u0022bbabbac\u0022,\u0022indices\u0022:[[5,7],[5,6],[0,3],[1,2]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a)(?<=(\u005c\u005c3(b)\u005c\u005c1){2})c\u0022}"
        "(a)(?<=(\u005c3(b)\u005c1){2}?)c" flags="d" input="bbabbac" "{\u0022matches\u0022:[\u0022ac\u0022,\u0022a\u0022,\u0022bba\u0022,\u0022b\u0022],\u0022index\u0022:5,\u0022input\u0022:\u0022bbabbac\u0022,\u0022indices\u0022:[[5,7],[5,6],[0,3],[1,2]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a)(?<=(\u005c\u005c3(b)\u005c\u005c1){2}?)c\u0022}"
        "(ab)(?<=(\u005c3(\u005c1)){2})c" flags="d" input="ababababc" "{\u0022matches\u0022:[\u0022abc\u0022,\u0022ab\u0022,\u0022abab\u0022,\u0022ab\u0022],\u0022index\u0022:6,\u0022input\u0022:\u0022ababababc\u0022,\u0022indices\u0022:[[6,9],[6,8],[0,4],[2,4]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(ab)(?<=(\u005c\u005c3(\u005c\u005c1)){2})c\u0022}"
        "(a+)(?<=(\u005c3(\u005c1)){2})b" flags="d" input="aaaaaaaab" "{\u0022matches\u0022:[\u0022aab\u0022,\u0022aa\u0022,\u0022aaaa\u0022,\u0022aa\u0022],\u0022index\u0022:6,\u0022input\u0022:\u0022aaaaaaaab\u0022,\u0022indices\u0022:[[6,9],[6,8],[0,4],[2,4]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a+)(?<=(\u005c\u005c3(\u005c\u005c1)){2})b\u0022}"
        "(a)(?<=(\u005c3(\u005c4(b))\u005c1){2})c" flags="d" input="bbbbabbbbac" "{\u0022matches\u0022:[\u0022ac\u0022,\u0022a\u0022,\u0022bbbba\u0022,\u0022bb\u0022,\u0022b\u0022],\u0022index\u0022:9,\u0022input\u0022:\u0022bbbbabbbbac\u0022,\u0022indices\u0022:[[9,11],[9,10],[0,5],[2,4],[3,4]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a)(?<=(\u005c\u005c3(\u005c\u005c4(b))\u005c\u005c1){2})c\u0022}"
        "(a)(?<=(\u005c3(b)\u005c1\u005c3){2})c" flags="d" input="bbabbac" "{\u0022matches\u0022:[\u0022ac\u0022,\u0022a\u0022,\u0022bba\u0022,\u0022b\u0022],\u0022index\u0022:5,\u0022input\u0022:\u0022bbabbac\u0022,\u0022indices\u0022:[[5,7],[5,6],[0,3],[1,2]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a)(?<=(\u005c\u005c3(b)\u005c\u005c1\u005c\u005c3){2})c\u0022}"
        "(a)(?<=(\u005c1\u005c3(b)){2})c" flags="d" input="ababc" "{\u0022match\u0022:null,\u0022lastIndex\u0022:0}"
        "(a)(?<!(\u005c3(b)\u005c1){2}q)c" flags="d" input="bbabbaac" "{\u0022matches\u0022:[\u0022ac\u0022,\u0022a\u0022,null,null],\u0022index\u0022:6,\u0022input\u0022:\u0022bbabbaac\u0022,\u0022indices\u0022:[[6,8],[6,7],null,null],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a)(?<!(\u005c\u005c3(b)\u005c\u005c1){2}q)c\u0022}"
        "(a)(?<=(\u005c3(\u005c1)){0})b" flags="d" input="ab" "{\u0022matches\u0022:[\u0022ab\u0022,\u0022a\u0022,null,null],\u0022index\u0022:0,\u0022input\u0022:\u0022ab\u0022,\u0022indices\u0022:[[0,2],[0,1],null,null],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a)(?<=(\u005c\u005c3(\u005c\u005c1)){0})b\u0022}"
        "()(?<=(\u005c3(\u005c1)){2})b" flags="d" input="b" "{\u0022matches\u0022:[\u0022b\u0022,\u0022\u0022,\u0022\u0022,\u0022\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022b\u0022,\u0022indices\u0022:[[0,1],[0,0],[0,0],[0,0]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022()(?<=(\u005c\u005c3(\u005c\u005c1)){2})b\u0022}"
        "(?<=(\u005c2(\u005c3)\u005c3){2})(a)" flags="d" input="a" "{\u0022matches\u0022:[\u0022a\u0022,\u0022\u0022,\u0022\u0022,\u0022a\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022a\u0022,\u0022indices\u0022:[[0,1],[0,0],[0,0],[0,1]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=(\u005c\u005c2(\u005c\u005c3)\u005c\u005c3){2})(a)\u0022}"
        "(a(?<=(\u005c3(\u005c1)){2}))b" flags="d" input="ab" "{\u0022matches\u0022:[\u0022ab\u0022,\u0022a\u0022,\u0022\u0022,\u0022\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022ab\u0022,\u0022indices\u0022:[[0,2],[0,1],[1,1],[1,1]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a(?<=(\u005c\u005c3(\u005c\u005c1)){2}))b\u0022}"
        "(a)(?<=(\u005c3(b)\u005c1){2})c" flags="di" input="bBAbbAc" "{\u0022matches\u0022:[\u0022Ac\u0022,\u0022A\u0022,\u0022bBA\u0022,\u0022B\u0022],\u0022index\u0022:5,\u0022input\u0022:\u0022bBAbbAc\u0022,\u0022indices\u0022:[[5,7],[5,6],[0,3],[1,2]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a)(?<=(\u005c\u005c3(b)\u005c\u005c1){2})c\u0022}"
        "(?<x>a)(?<=(?<y>\u005ck<z>(?<z>b)\u005ck<x>){2})c" flags="d" input="bbabbac" "{\u0022matches\u0022:[\u0022ac\u0022,\u0022a\u0022,\u0022bba\u0022,\u0022b\u0022],\u0022index\u0022:5,\u0022input\u0022:\u0022bbabbac\u0022,\u0022groups\u0022:{\u0022x\u0022:\u0022a\u0022,\u0022y\u0022:\u0022bba\u0022,\u0022z\u0022:\u0022b\u0022},\u0022indices\u0022:[[5,7],[5,6],[0,3],[1,2]],\u0022indicesGroups\u0022:{\u0022x\u0022:[5,6],\u0022y\u0022:[0,3],\u0022z\u0022:[1,2]},\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<x>a)(?<=(?<y>\u005c\u005ck<z>(?<z>b)\u005c\u005ck<x>){2})c\u0022}"
        "(?<x>ab)(?<=(?<y>\u005ck<z>(?<z>\u005ck<x>)){2})c" flags="d" input="ababababc" "{\u0022matches\u0022:[\u0022abc\u0022,\u0022ab\u0022,\u0022abab\u0022,\u0022ab\u0022],\u0022index\u0022:6,\u0022input\u0022:\u0022ababababc\u0022,\u0022groups\u0022:{\u0022x\u0022:\u0022ab\u0022,\u0022y\u0022:\u0022abab\u0022,\u0022z\u0022:\u0022ab\u0022},\u0022indices\u0022:[[6,9],[6,8],[0,4],[2,4]],\u0022indicesGroups\u0022:{\u0022x\u0022:[6,8],\u0022y\u0022:[0,4],\u0022z\u0022:[2,4]},\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<x>ab)(?<=(?<y>\u005c\u005ck<z>(?<z>\u005c\u005ck<x>)){2})c\u0022}"
        "(?:(?<x>a)|(?<x>b))(?<=(\u005c4(\u005ck<x>)){2})c" flags="d" input="bbbbc" "{\u0022matches\u0022:[\u0022bc\u0022,null,\u0022b\u0022,\u0022bb\u0022,\u0022b\u0022],\u0022index\u0022:3,\u0022input\u0022:\u0022bbbbc\u0022,\u0022groups\u0022:{\u0022x\u0022:\u0022b\u0022},\u0022indices\u0022:[[3,5],null,[3,4],[0,2],[1,2]],\u0022indicesGroups\u0022:{\u0022x\u0022:[3,4]},\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?:(?<x>a)|(?<x>b))(?<=(\u005c\u005c4(\u005c\u005ck<x>)){2})c\u0022}"
        "(a+)(?<=(\u005c3(\u005c1)){2})b" flags="d" input="aaaab" "{\u0022matches\u0022:[\u0022ab\u0022,\u0022a\u0022,\u0022aa\u0022,\u0022a\u0022],\u0022index\u0022:3,\u0022input\u0022:\u0022aaaab\u0022,\u0022indices\u0022:[[3,5],[3,4],[0,2],[1,2]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a+)(?<=(\u005c\u005c3(\u005c\u005c1)){2})b\u0022}"
        "#);
}

#[test]
fn nested_dependencies_import_retries_named_aliases_and_negative_effects_keep_ranges() {
    assert_true(
        r"let a=/(?<x>a)(?<=(?<y>\k<z>(?<z>b)\k<x>){2})c/d.exec('bbabbac');a.index===5&&a.groups.x==='a'&&a.groups.y==='bba'&&a.groups.z==='b'&&a.indices.groups.z===a.indices[3]&&a.indices[3][0]===1&&a.indices[1][0]===5",
    );
    assert_true(
        r"let a=/(?<x>ab)(?<=(?<y>\k<z>(?<z>\k<x>)){2})c/d.exec('ababababc');a.groups.y==='abab'&&a.groups.z==='ab'&&a.indices[1][0]===6&&a.indices[2][0]===0&&a.indices[3][0]===2",
    );
    assert_true(
        r"let a=/(a+)(?<=(\3(\1)){2})b/d.exec('aaaaaaaab');a.index===6&&a[1]==='aa'&&a[2]==='aaaa'&&a[3]==='aa'&&a.indices[3][0]===2",
    );
    assert_true(
        r"let a=/(?:(?<x>a)|(?<x>b))(?<=(\4(\k<x>)){2})c/d.exec('bbbbc');a.index===3&&a[1]===undefined&&a[2]==='b'&&a[3]==='bb'&&a[4]==='b'&&a.groups.x==='b'&&a.indices.groups.x===a.indices[2]",
    );
    assert_true(
        r"let a=/(a)(?<!(\3(b)\1){2}q)c/d.exec('bbabbaac');a.index===6&&a[1]==='a'&&a[2]===undefined&&a[3]===undefined&&a.indices[1][0]===6",
    );
    assert_true(r"/(a)(?<=(\1\3(b)){2})c/.exec('ababc')===null");
    assert_true(
        r"let a=/([\uD800])(?<=(\3([\uDC00])\1){2})c/d.exec('\uDC00\uDC00\uD800\uDC00\uDC00\uD800c');a.index===5&&a.indices[2][0]===0&&a.indices[3][0]===1",
    );
}

#[test]
fn consumers_global_sticky_empty_skip_and_callbacks_use_resolved_unit_ranges() {
    assert_true(
        r"let r=/(a)(?<=(\3(b)\1){2})c/dy;r.lastIndex=5;let a=r.exec('bbabbac');a.index===5&&r.lastIndex===7&&r.exec('bbabbac')===null&&r.lastIndex===0",
    );
    assert_true(
        r"let r=/(a)(?<=(\3(b)\1){2})c/dg,a=[...'bbabbac bbabbac'.matchAll(r)];a.length===2&&a[1].index===13&&a[1].indices[3][0]===9&&r.lastIndex===0",
    );
    assert_true(
        r"let a=/(a)(?<=(\3(\1)){0})b/d.exec('ab');a[1]==='a'&&a[2]===undefined&&a.indices[3]===undefined",
    );
    assert_true(
        r"let a=[...'ab'.matchAll(/()(?<=(\3(\1)){2})/dg)];a.length===3&&a[2].index===2&&a[2].indices[3][0]===2",
    );
    assert_true(
        r"let seen=[];let s='bbabbac bbabbac'.replace(/(a)(?<=(\3(b)\1){2})c/g,(m,x,y,z,i)=>{seen.push(x,y,z,i);return '_'});s==='bbabb_ bbabb_'&&seen.join('|')==='a|bba|b|5|a|bba|b|13'",
    );
    assert_true(
        r"'bbabbac'.search(/(a)(?<=(\3(b)\1){2})c/)===5&&'bbabbac'.split(/(a)(?<=(\3(b)\1){2})c/).join('|')==='bbabb|a|bba|b|'",
    );
}

#[test]
fn explicit_work_aborts_preserve_last_index_and_bypass_handlers() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval(&format!(
            r"let marker=0,r=/(a)(?<=(\3(b)\1){{2}})c/g;r.lastIndex=1;let text='{}c'",
            "bba".repeat(5000)
        ))
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
fn variable_counts_choices_dependent_children_and_unicode_remain_pending() {
    for source in [
        r"/(a)(?<=(\3(b)\1){1,2})c/.exec('bbabbac')",
        r"/(a)(?<=(\3(b)\1){2}|b)c/.exec('bbabbac')",
        r"/(a)(?<=(\3(b)\1){2}(?<=\3))c/.exec('bbabbac')",
        r"/(a)(?<=(\3(b)\1){2})c/u.exec('bbabbac')",
        r"/(a)(?<=(\3(b)\1){2})c/v.exec('bbabbac')",
    ] {
        crate::common::regexp_oracle::assert_matches_reference(&mut Realm::default(), source);
    }
}
