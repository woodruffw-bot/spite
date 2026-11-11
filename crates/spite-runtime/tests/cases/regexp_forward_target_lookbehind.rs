//! Fixed right-hand same-unit capture references in ordinary lookbehind.
use crate::common::assert_true;
use spite_core::JsString;
use spite_runtime::{Error, Limits, Realm, Value};
use std::fmt::Write;
#[test]
fn forward_target_lookbehind_result_snapshot() {
    let mut rows = String::new();
    for (source, flags, text) in [
        (r"(?<=(\2(a)){2})b", "d", "aaaab"),
        (r"(?<=(\2(a)){2}?)b", "d", "aaaab"),
        (r"(?<=(\2([ab])){2})c", "d", "aabbc"),
        (r"(?<=(\2(ab)){2})c", "d", "ababababc"),
        (r"(?<=(\2(\3(a))){2})b", "d", "aaaaaaaab"),
        (r"(?<=(\2(a)\2){2})b", "d", "aaaab"),
        (r"(?<=(\2a()){2})b", "d", "aab"),
        (r"(?<=(\2(a)){0})b", "d", "b"),
        (r"(?<!(\2(a)){2}q)b", "d", "aaaaxb"),
        (r"(?<=((a)\1){2}(?=(\4(b)){2}))b", "d", "aabbb"),
        (r"(?=(\2(a)){2})a", "d", "aa"),
        (r"(b)(?<=(\3(a)){2}\1)c", "d", "aaaabc"),
        (r"(?<=(\2(a)){2})b", "di", "aAaAb"),
        (r"(?<=(?<y>\k<x>(?<x>a)){2})b", "d", "aaaab"),
        (r"(?<=(?<z>\k<y>(?<y>\k<x>(?<x>a))){2})b", "d", "aaaaaaaab"),
        (r"(?<=(\2([\uD800])){2})[\uDC00]", "d", "surrogates"),
        (r"(?<=(\2(a)){2})b", "d", "aab"),
    ] {
        let source = JsString::from(source);
        let flags = JsString::from(flags);
        let input = if text == "surrogates" {
            JsString::from_code_units(vec![0xd800, 0xd800, 0xd800, 0xd800, 0xdc00])
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
        "(?<=(\u005c2(a)){2})b" flags="d" input="aaaab" "{\u0022matches\u0022:[\u0022b\u0022,\u0022aa\u0022,\u0022a\u0022],\u0022index\u0022:4,\u0022input\u0022:\u0022aaaab\u0022,\u0022indices\u0022:[[4,5],[0,2],[1,2]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=(\u005c\u005c2(a)){2})b\u0022}"
        "(?<=(\u005c2(a)){2}?)b" flags="d" input="aaaab" "{\u0022matches\u0022:[\u0022b\u0022,\u0022aa\u0022,\u0022a\u0022],\u0022index\u0022:4,\u0022input\u0022:\u0022aaaab\u0022,\u0022indices\u0022:[[4,5],[0,2],[1,2]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=(\u005c\u005c2(a)){2}?)b\u0022}"
        "(?<=(\u005c2([ab])){2})c" flags="d" input="aabbc" "{\u0022matches\u0022:[\u0022c\u0022,\u0022aa\u0022,\u0022a\u0022],\u0022index\u0022:4,\u0022input\u0022:\u0022aabbc\u0022,\u0022indices\u0022:[[4,5],[0,2],[1,2]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=(\u005c\u005c2([ab])){2})c\u0022}"
        "(?<=(\u005c2(ab)){2})c" flags="d" input="ababababc" "{\u0022matches\u0022:[\u0022c\u0022,\u0022abab\u0022,\u0022ab\u0022],\u0022index\u0022:8,\u0022input\u0022:\u0022ababababc\u0022,\u0022indices\u0022:[[8,9],[0,4],[2,4]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=(\u005c\u005c2(ab)){2})c\u0022}"
        "(?<=(\u005c2(\u005c3(a))){2})b" flags="d" input="aaaaaaaab" "{\u0022matches\u0022:[\u0022b\u0022,\u0022aaaa\u0022,\u0022aa\u0022,\u0022a\u0022],\u0022index\u0022:8,\u0022input\u0022:\u0022aaaaaaaab\u0022,\u0022indices\u0022:[[8,9],[0,4],[2,4],[3,4]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=(\u005c\u005c2(\u005c\u005c3(a))){2})b\u0022}"
        "(?<=(\u005c2(a)\u005c2){2})b" flags="d" input="aaaab" "{\u0022matches\u0022:[\u0022b\u0022,\u0022aa\u0022,\u0022a\u0022],\u0022index\u0022:4,\u0022input\u0022:\u0022aaaab\u0022,\u0022indices\u0022:[[4,5],[0,2],[1,2]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=(\u005c\u005c2(a)\u005c\u005c2){2})b\u0022}"
        "(?<=(\u005c2a()){2})b" flags="d" input="aab" "{\u0022matches\u0022:[\u0022b\u0022,\u0022a\u0022,\u0022\u0022],\u0022index\u0022:2,\u0022input\u0022:\u0022aab\u0022,\u0022indices\u0022:[[2,3],[0,1],[1,1]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=(\u005c\u005c2a()){2})b\u0022}"
        "(?<=(\u005c2(a)){0})b" flags="d" input="b" "{\u0022matches\u0022:[\u0022b\u0022,null,null],\u0022index\u0022:0,\u0022input\u0022:\u0022b\u0022,\u0022indices\u0022:[[0,1],null,null],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=(\u005c\u005c2(a)){0})b\u0022}"
        "(?<!(\u005c2(a)){2}q)b" flags="d" input="aaaaxb" "{\u0022matches\u0022:[\u0022b\u0022,null,null],\u0022index\u0022:5,\u0022input\u0022:\u0022aaaaxb\u0022,\u0022indices\u0022:[[5,6],null,null],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<!(\u005c\u005c2(a)){2}q)b\u0022}"
        "(?<=((a)\u005c1){2}(?=(\u005c4(b)){2}))b" flags="d" input="aabbb" "{\u0022matches\u0022:[\u0022b\u0022,\u0022a\u0022,\u0022a\u0022,\u0022b\u0022,\u0022b\u0022],\u0022index\u0022:2,\u0022input\u0022:\u0022aabbb\u0022,\u0022indices\u0022:[[2,3],[0,1],[0,1],[3,4],[3,4]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=((a)\u005c\u005c1){2}(?=(\u005c\u005c4(b)){2}))b\u0022}"
        "(?=(\u005c2(a)){2})a" flags="d" input="aa" "{\u0022matches\u0022:[\u0022a\u0022,\u0022a\u0022,\u0022a\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022aa\u0022,\u0022indices\u0022:[[0,1],[1,2],[1,2]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?=(\u005c\u005c2(a)){2})a\u0022}"
        "(b)(?<=(\u005c3(a)){2}\u005c1)c" flags="d" input="aaaabc" "{\u0022matches\u0022:[\u0022bc\u0022,\u0022b\u0022,\u0022aa\u0022,\u0022a\u0022],\u0022index\u0022:4,\u0022input\u0022:\u0022aaaabc\u0022,\u0022indices\u0022:[[4,6],[4,5],[0,2],[1,2]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(b)(?<=(\u005c\u005c3(a)){2}\u005c\u005c1)c\u0022}"
        "(?<=(\u005c2(a)){2})b" flags="di" input="aAaAb" "{\u0022matches\u0022:[\u0022b\u0022,\u0022aA\u0022,\u0022A\u0022],\u0022index\u0022:4,\u0022input\u0022:\u0022aAaAb\u0022,\u0022indices\u0022:[[4,5],[0,2],[1,2]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=(\u005c\u005c2(a)){2})b\u0022}"
        "(?<=(?<y>\u005ck<x>(?<x>a)){2})b" flags="d" input="aaaab" "{\u0022matches\u0022:[\u0022b\u0022,\u0022aa\u0022,\u0022a\u0022],\u0022index\u0022:4,\u0022input\u0022:\u0022aaaab\u0022,\u0022groups\u0022:{\u0022y\u0022:\u0022aa\u0022,\u0022x\u0022:\u0022a\u0022},\u0022indices\u0022:[[4,5],[0,2],[1,2]],\u0022indicesGroups\u0022:{\u0022y\u0022:[0,2],\u0022x\u0022:[1,2]},\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=(?<y>\u005c\u005ck<x>(?<x>a)){2})b\u0022}"
        "(?<=(?<z>\u005ck<y>(?<y>\u005ck<x>(?<x>a))){2})b" flags="d" input="aaaaaaaab" "{\u0022matches\u0022:[\u0022b\u0022,\u0022aaaa\u0022,\u0022aa\u0022,\u0022a\u0022],\u0022index\u0022:8,\u0022input\u0022:\u0022aaaaaaaab\u0022,\u0022groups\u0022:{\u0022z\u0022:\u0022aaaa\u0022,\u0022y\u0022:\u0022aa\u0022,\u0022x\u0022:\u0022a\u0022},\u0022indices\u0022:[[8,9],[0,4],[2,4],[3,4]],\u0022indicesGroups\u0022:{\u0022z\u0022:[0,4],\u0022y\u0022:[2,4],\u0022x\u0022:[3,4]},\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=(?<z>\u005c\u005ck<y>(?<y>\u005c\u005ck<x>(?<x>a))){2})b\u0022}"
        "(?<=(\u005c2([\u005cuD800])){2})[\u005cuDC00]" flags="d" input="\ud800\ud800\ud800\ud800\udc00" "{\u0022matches\u0022:[\u0022\u005cudc00\u0022,\u0022\u005cud800\u005cud800\u0022,\u0022\u005cud800\u0022],\u0022index\u0022:4,\u0022input\u0022:\u0022\u005cud800\u005cud800\u005cud800\ud800\udc00\u0022,\u0022indices\u0022:[[4,5],[0,2],[1,2]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=(\u005c\u005c2([\u005c\u005cuD800])){2})[\u005c\u005cuDC00]\u0022}"
        "(?<=(\u005c2(a)){2})b" flags="d" input="aab" "{\u0022match\u0022:null,\u0022lastIndex\u0022:0}"
        "#);
}
#[test]
fn nested_future_names_empty_slots_outside_imports_and_matching_directions_keep_exact_ranges() {
    assert_true(r"/(?<=((\3a)(b)){2})c/.exec('ababc')===null");
    assert_true(
        r"let a=/(?<=(?<y>\k<x>(?<x>a)){2})b/d.exec('aaaab');a.index===4&&a.groups.y==='aa'&&a.groups.x==='a'&&a.indices.groups.x===a.indices[2]&&a.indices.groups.x[0]===1&&a.indices.groups.y[0]===0",
    );
    assert_true(
        r"let a=/(?<=(?<z>\k<y>(?<y>\k<x>(?<x>a))){2})b/d.exec('aaaaaaaab');a.groups.z==='aaaa'&&a.groups.y==='aa'&&a.groups.x==='a'&&a.indices.groups.y[0]===2&&a.indices.groups.x[0]===3",
    );
    assert_true(
        r"let a=/(?<=(\2a()){2})b/d.exec('aab');a[1]==='a'&&a[2]===''&&a.indices[2][0]===1&&a.indices[2][1]===1",
    );
    assert_true(
        r"let a=/(?<!(?<y>\k<x>(?<x>a)){2}q)b/d.exec('aaaaxb');a.index===5&&a.groups.x===undefined&&a.groups.y===undefined&&a.indices.groups.x===undefined&&Object.hasOwn(a.groups,'y')",
    );
    assert_true(
        r"let a=/(?<=((a)\1){2}(?=(\4(b)){2}))b/d.exec('aabbb'),b=/(?=(\2(a)){2})a/d.exec('aa');a.indices[3][0]===3&&a.indices[4][0]===3&&b.indices[1][0]===1&&b.indices[2][0]===1",
    );
    assert_true(
        r"let a=/(b)(?<=(\3(a)){2}\1)c/d.exec('aaaabc');a.index===4&&a[1]==='b'&&a[2]==='aa'&&a.indices[3][0]===1",
    );
}
#[test]
fn consumers_sticky_global_zero_counts_and_callbacks_preserve_right_capture_positions() {
    assert_true(
        r"let r=/(?<=(\2(a)){2})b/dy;r.lastIndex=4;let a=r.exec('aaaab');a.index===4&&r.lastIndex===5&&r.exec('aaaab')===null&&r.lastIndex===0",
    );
    assert_true(
        r"let r=/(?<=(\2(a)){2})b/dg,a=[...'aaaab aaaab'.matchAll(r)];a.length===2&&a[1].index===10&&a[1].indices[2][0]===7&&r.lastIndex===0",
    );
    assert_true(
        r"let r=/(?<=(\2(a)){0})/dg,a=[...'ab'.matchAll(r)];a.length===3&&a[2].index===2&&a[2][1]===undefined&&a[2].indices[2]===undefined",
    );
    assert_true(
        r"let seen=[];let s='aaaab aaaab'.replace(/(?<=(\2(a)){2})b/g,(m,x,y,i)=>{seen.push(x,y,i);return '_'});s==='aaaa_ aaaa_'&&seen.join('|')==='aa|a|4|aa|a|10'",
    );
    assert_true(
        r"'aaaab'.replace(/(?<=(?<y>\k<x>(?<x>a)){2})b/,'<$<y>>')==='aaaa<aa>'&&'aaaab'.search(/(?<=(\2(a)){2})b/)===4&&'aaaab'.split(/(?<=(\2(a)){2})b/).join('|')==='aaaa|aa|a|'",
    );
}

#[test]
fn explicit_work_aborts_preserve_last_index_and_bypass_handlers() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval(r"let marker=0,r=/(?<=(\2(a)){2})b/g;r.lastIndex=1;let text='a'.repeat(5000)+'b'")
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
fn variable_counts_mixed_outside_units_choices_dependent_children_and_unicode_remain_pending() {
    for source in [
        r"/(?<=(\2(a)){1,2})b/.exec('aaaab')",
        r"/(a)(?<=(\1\3(b)){1,2})c/.exec('ababc')",
        r"/(?<=(\2(a)){2}|a)b/.exec('aaaab')",
        r"/(?<=(\2(a)){2}(?<=\2))b/.exec('aaaab')",
        r"/(?<=(\2(a)){2})b/u.exec('aaaab')",
        r"/(?<=(\2(a)){2})b/v.exec('aaaab')",
    ] {
        crate::common::regexp_oracle::assert_matches_reference(&mut Realm::default(), source);
    }
}
