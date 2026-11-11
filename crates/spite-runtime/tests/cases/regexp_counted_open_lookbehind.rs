//! Same-unit enclosing references remain unclosed in either matching direction.
use crate::common::assert_true;
use spite_core::JsString;
use spite_runtime::{Error, Limits, Realm, Value};
use std::fmt::Write;
#[test]
fn counted_open_lookbehind_result_snapshot() {
    let mut rows = String::new();
    for (source, flags, text) in [
        (r"(?<=(a\1){2})b", "d", "aab"),
        (r"(?<=((a)\1){2})b", "d", "aab"),
        (r"(?<=((a)\2\1){2})b", "d", "aab"),
        (r"(?<=((\2)a){2})b", "d", "aab"),
        (r"(?<=(a\1){2}?)b", "d", "aaaab"),
        (r"(?<=([ab]\1){2})c", "d", "abc"),
        (r"(?<=(a\1){0})b", "d", "b"),
        (r"(?<!(a\1){2}q)b", "d", "aaxb"),
        (r"(?<=((a)\1){2}(?=(b\3){2}))b", "d", "aabbb"),
        (r"(?=(a\1){2})a", "d", "aa"),
        (r"(b)(?<=(a\2){2}\1)c", "d", "aabc"),
        (r"(?<=((a)\1\B){2})b", "d", "aab"),
        (r"(?<=(a\1){2})b", "di", "aAb"),
        (r"(?<=(?<x>a\k<x>){2})b", "d", "aab"),
        (r"(?<=(?<x>(?<y>a)\k<x>){2})b", "d", "aab"),
        (r"(?<=([\uD800]\1){2})[\uDC00]", "d", "surrogates"),
        (r"(?<=(a\1){2})b", "d", "ab"),
    ] {
        let source = JsString::from(source);
        let flags = JsString::from(flags);
        let input = if text == "surrogates" {
            JsString::from_code_units(vec![0xd800, 0xd800, 0xdc00])
        } else {
            JsString::from(text)
        };
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
        "(?<=(a\u005c1){2})b" flags="d" input="aab" "{\u0022matches\u0022:[\u0022b\u0022,\u0022a\u0022],\u0022index\u0022:2,\u0022input\u0022:\u0022aab\u0022,\u0022indices\u0022:[[2,3],[0,1]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=(a\u005c\u005c1){2})b\u0022}"
        "(?<=((a)\u005c1){2})b" flags="d" input="aab" "{\u0022matches\u0022:[\u0022b\u0022,\u0022a\u0022,\u0022a\u0022],\u0022index\u0022:2,\u0022input\u0022:\u0022aab\u0022,\u0022indices\u0022:[[2,3],[0,1],[0,1]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=((a)\u005c\u005c1){2})b\u0022}"
        "(?<=((a)\u005c2\u005c1){2})b" flags="d" input="aab" "{\u0022matches\u0022:[\u0022b\u0022,\u0022a\u0022,\u0022a\u0022],\u0022index\u0022:2,\u0022input\u0022:\u0022aab\u0022,\u0022indices\u0022:[[2,3],[0,1],[0,1]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=((a)\u005c\u005c2\u005c\u005c1){2})b\u0022}"
        "(?<=((\u005c2)a){2})b" flags="d" input="aab" "{\u0022matches\u0022:[\u0022b\u0022,\u0022a\u0022,\u0022\u0022],\u0022index\u0022:2,\u0022input\u0022:\u0022aab\u0022,\u0022indices\u0022:[[2,3],[0,1],[0,0]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=((\u005c\u005c2)a){2})b\u0022}"
        "(?<=(a\u005c1){2}?)b" flags="d" input="aaaab" "{\u0022matches\u0022:[\u0022b\u0022,\u0022a\u0022],\u0022index\u0022:4,\u0022input\u0022:\u0022aaaab\u0022,\u0022indices\u0022:[[4,5],[2,3]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=(a\u005c\u005c1){2}?)b\u0022}"
        "(?<=([ab]\u005c1){2})c" flags="d" input="abc" "{\u0022matches\u0022:[\u0022c\u0022,\u0022a\u0022],\u0022index\u0022:2,\u0022input\u0022:\u0022abc\u0022,\u0022indices\u0022:[[2,3],[0,1]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=([ab]\u005c\u005c1){2})c\u0022}"
        "(?<=(a\u005c1){0})b" flags="d" input="b" "{\u0022matches\u0022:[\u0022b\u0022,null],\u0022index\u0022:0,\u0022input\u0022:\u0022b\u0022,\u0022indices\u0022:[[0,1],null],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=(a\u005c\u005c1){0})b\u0022}"
        "(?<!(a\u005c1){2}q)b" flags="d" input="aaxb" "{\u0022matches\u0022:[\u0022b\u0022,null],\u0022index\u0022:3,\u0022input\u0022:\u0022aaxb\u0022,\u0022indices\u0022:[[3,4],null],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<!(a\u005c\u005c1){2}q)b\u0022}"
        "(?<=((a)\u005c1){2}(?=(b\u005c3){2}))b" flags="d" input="aabbb" "{\u0022matches\u0022:[\u0022b\u0022,\u0022a\u0022,\u0022a\u0022,\u0022b\u0022],\u0022index\u0022:2,\u0022input\u0022:\u0022aabbb\u0022,\u0022indices\u0022:[[2,3],[0,1],[0,1],[3,4]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=((a)\u005c\u005c1){2}(?=(b\u005c\u005c3){2}))b\u0022}"
        "(?=(a\u005c1){2})a" flags="d" input="aa" "{\u0022matches\u0022:[\u0022a\u0022,\u0022a\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022aa\u0022,\u0022indices\u0022:[[0,1],[1,2]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?=(a\u005c\u005c1){2})a\u0022}"
        "(b)(?<=(a\u005c2){2}\u005c1)c" flags="d" input="aabc" "{\u0022matches\u0022:[\u0022bc\u0022,\u0022b\u0022,\u0022a\u0022],\u0022index\u0022:2,\u0022input\u0022:\u0022aabc\u0022,\u0022indices\u0022:[[2,4],[2,3],[0,1]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(b)(?<=(a\u005c\u005c2){2}\u005c\u005c1)c\u0022}"
        "(?<=((a)\u005c1\u005cB){2})b" flags="d" input="aab" "{\u0022matches\u0022:[\u0022b\u0022,\u0022a\u0022,\u0022a\u0022],\u0022index\u0022:2,\u0022input\u0022:\u0022aab\u0022,\u0022indices\u0022:[[2,3],[0,1],[0,1]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=((a)\u005c\u005c1\u005c\u005cB){2})b\u0022}"
        "(?<=(a\u005c1){2})b" flags="di" input="aAb" "{\u0022matches\u0022:[\u0022b\u0022,\u0022a\u0022],\u0022index\u0022:2,\u0022input\u0022:\u0022aAb\u0022,\u0022indices\u0022:[[2,3],[0,1]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=(a\u005c\u005c1){2})b\u0022}"
        "(?<=(?<x>a\u005ck<x>){2})b" flags="d" input="aab" "{\u0022matches\u0022:[\u0022b\u0022,\u0022a\u0022],\u0022index\u0022:2,\u0022input\u0022:\u0022aab\u0022,\u0022groups\u0022:{\u0022x\u0022:\u0022a\u0022},\u0022indices\u0022:[[2,3],[0,1]],\u0022indicesGroups\u0022:{\u0022x\u0022:[0,1]},\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=(?<x>a\u005c\u005ck<x>){2})b\u0022}"
        "(?<=(?<x>(?<y>a)\u005ck<x>){2})b" flags="d" input="aab" "{\u0022matches\u0022:[\u0022b\u0022,\u0022a\u0022,\u0022a\u0022],\u0022index\u0022:2,\u0022input\u0022:\u0022aab\u0022,\u0022groups\u0022:{\u0022x\u0022:\u0022a\u0022,\u0022y\u0022:\u0022a\u0022},\u0022indices\u0022:[[2,3],[0,1],[0,1]],\u0022indicesGroups\u0022:{\u0022x\u0022:[0,1],\u0022y\u0022:[0,1]},\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=(?<x>(?<y>a)\u005c\u005ck<x>){2})b\u0022}"
        "(?<=([\u005cuD800]\u005c1){2})[\u005cuDC00]" flags="d" input="\ud800\ud800\udc00" "{\u0022matches\u0022:[\u0022\u005cudc00\u0022,\u0022\u005cud800\u0022],\u0022index\u0022:2,\u0022input\u0022:\u0022\u005cud800\ud800\udc00\u0022,\u0022indices\u0022:[[2,3],[0,1]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=([\u005c\u005cuD800]\u005c\u005c1){2})[\u005c\u005cuDC00]\u0022}"
        "(?<=(a\u005c1){2})b" flags="d" input="ab" "{\u0022match\u0022:null,\u0022lastIndex\u0022:0}"
        "#);
}
#[test]
fn names_open_local_reads_outside_aliases_and_assertion_directions_keep_exact_ranges() {
    assert_true(
        r"let a=/(?<=(?<x>a\k<x>){2})b/d.exec('aab');a.index===2&&a.groups.x==='a'&&a.indices.groups.x===a.indices[1]&&a.indices.groups.x[0]===0&&a.indices.groups.x[1]===1",
    );
    assert_true(
        r"let a=/(?<=(?<x>(?<y>a)\k<y>\k<x>){2})b/d.exec('aab');a.groups.x==='a'&&a.groups.y==='a'&&a.indices.groups.y===a.indices[2]&&a.indices.groups.y[0]===0",
    );
    assert_true(
        r"let a=/(?<=((\2)a){2})b/d.exec('aab');a[1]==='a'&&a[2]===''&&a.indices[2][0]===0&&a.indices[2][1]===0",
    );
    assert_true(
        r"let a=/(?<!(?<x>a\k<x>){2}q)b/d.exec('aaxb');a.index===3&&a.groups.x===undefined&&a.indices.groups.x===undefined&&Object.hasOwn(a.groups,'x')",
    );
    assert_true(
        r"let a=/(?<=((a)\1){2}(?=(b\3){2}))b/d.exec('aabbb'),b=/(?=(a\1){2})a/d.exec('aa');a[3]==='b'&&a.indices[3][0]===3&&b[1]==='a'&&b.indices[1][0]===1",
    );
    assert_true(
        r"let a=/(?:(?<x>a)|(?<x>b))(?<=(c\3){2}\k<x>)d/d.exec('ccad');a.index===2&&a.groups.x==='a'&&a[2]===undefined&&a[3]==='c'&&a.indices[3][0]===0&&a.indices.groups.x===a.indices[1]",
    );
}
#[test]
fn consumers_global_sticky_zero_counts_and_callbacks_keep_open_reads_undefined() {
    assert_true(
        r"let r=/(?<=(a\1){2})b/dy;r.lastIndex=2;let a=r.exec('aab');a.index===2&&r.lastIndex===3&&r.exec('aab')===null&&r.lastIndex===0",
    );
    assert_true(
        r"let r=/(?<=(a\1){2})b/dg,a=[...'aab aab'.matchAll(r)];a.length===2&&a[1].index===6&&a[1].indices[1][0]===4&&r.lastIndex===0",
    );
    assert_true(
        r"let r=/(?<=(a\1){0})/dg,a=[...'ab'.matchAll(r)];a.length===3&&a[2].index===2&&a[2][1]===undefined&&a[2].indices[1]===undefined",
    );
    assert_true(
        r"let seen=[];let s='aab aab'.replace(/(?<=(a\1){2})b/g,(m,x,i)=>{seen.push(x,i);return '_'});s==='aa_ aa_'&&seen.join('|')==='a|2|a|6'",
    );
    assert_true(
        r"'aab'.replace(/(?<=(?<x>a\k<x>){2})b/,'<$<x>>')==='aa<a>'&&'aab'.search(/(?<=(a\1){2})b/)===2&&'aab'.split(/(?<=(a\1){2})b/).join('|')==='aa|a|'",
    );
}

#[test]
fn explicit_work_aborts_preserve_last_index_and_bypass_handlers() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval(r"let marker=0,r=/(?<=((a)\1){2})b/g;r.lastIndex=1;let text='a'.repeat(5000)+'b'")
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
fn consuming_forward_targets_variable_counts_mixed_outside_units_choices_and_unicode_remain_pending()
 {
    for source in [
        r"/(?<=(\2(a)){1,2})b/.exec('aaaab')",
        r"/(?<=((\3a)(b)){1,2})c/.exec('ababc')",
        r"/(?<=(a\1){1,2})b/.exec('aab')",
        r"/(a)(?<=(b\1\2){1,2})c/.exec('ababc')",
        r"/(?<=(a\1){2}|a)b/.exec('aab')",
        r"/(?<=(a\1){2})b/u.exec('aab')",
        r"/(?<=(a\1){2})b/v.exec('aab')",
    ] {
        crate::common::regexp_oracle::assert_matches_reference(&mut Realm::default(), source);
    }
}
