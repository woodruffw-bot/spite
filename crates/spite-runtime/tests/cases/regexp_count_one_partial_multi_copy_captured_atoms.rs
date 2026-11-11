//! Once-counted scalar atoms retain original partial capture boundaries.
use crate::common::assert_true;
use spite_core::JsString;
use spite_runtime::{Error, Limits, Realm, Value};
use std::fmt::Write;
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
    insta::assert_snapshot!(rows, @r#"
        "(?<=a(?=(b)(\u005c1(\u005c1)){1}))b" flags="d" input="abbb" "{\u0022matches\u0022:[\u0022b\u0022,\u0022b\u0022,\u0022bb\u0022,\u0022b\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022abbb\u0022,\u0022indices\u0022:[[1,2],[1,2],[2,4],[3,4]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=a(?=(b)(\u005c\u005c1(\u005c\u005c1)){1}))b\u0022}"
        "(?<=a(?=(b)((\u005c1)\u005c1){1}))b" flags="d" input="abbb" "{\u0022matches\u0022:[\u0022b\u0022,\u0022b\u0022,\u0022bb\u0022,\u0022b\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022abbb\u0022,\u0022indices\u0022:[[1,2],[1,2],[2,4],[2,3]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=a(?=(b)((\u005c\u005c1)\u005c\u005c1){1}))b\u0022}"
        "(?<=a(?=(b)(\u005c1()\u005c1){1}))b" flags="d" input="abbb" "{\u0022matches\u0022:[\u0022b\u0022,\u0022b\u0022,\u0022bb\u0022,\u0022\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022abbb\u0022,\u0022indices\u0022:[[1,2],[1,2],[2,4],[3,3]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=a(?=(b)(\u005c\u005c1()\u005c\u005c1){1}))b\u0022}"
        "(?<=a(?=(b)(\u005c1(\u005c1)\u005c1){1}))b" flags="d" input="abbbb" "{\u0022matches\u0022:[\u0022b\u0022,\u0022b\u0022,\u0022bbb\u0022,\u0022b\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022abbbb\u0022,\u0022indices\u0022:[[1,2],[1,2],[2,5],[3,4]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=a(?=(b)(\u005c\u005c1(\u005c\u005c1)\u005c\u005c1){1}))b\u0022}"
        "(?<=a(?=(b)(\u005c1(\u005c1\u005c1)\u005c1){1}))b" flags="d" input="abbbbb" "{\u0022matches\u0022:[\u0022b\u0022,\u0022b\u0022,\u0022bbbb\u0022,\u0022bb\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022abbbbb\u0022,\u0022indices\u0022:[[1,2],[1,2],[2,6],[3,5]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=a(?=(b)(\u005c\u005c1(\u005c\u005c1\u005c\u005c1)\u005c\u005c1){1}))b\u0022}"
        "(?<=(\u005c3(\u005c3)){1}(a))b" flags="d" input="aaab" "{\u0022matches\u0022:[\u0022b\u0022,\u0022aa\u0022,\u0022a\u0022,\u0022a\u0022],\u0022index\u0022:3,\u0022input\u0022:\u0022aaab\u0022,\u0022indices\u0022:[[3,4],[0,2],[1,2],[2,3]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=(\u005c\u005c3(\u005c\u005c3)){1}(a))b\u0022}"
        "(?<=((\u005c3)\u005c3){1}(a))b" flags="d" input="aaab" "{\u0022matches\u0022:[\u0022b\u0022,\u0022aa\u0022,\u0022a\u0022,\u0022a\u0022],\u0022index\u0022:3,\u0022input\u0022:\u0022aaab\u0022,\u0022indices\u0022:[[3,4],[0,2],[0,1],[2,3]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=((\u005c\u005c3)\u005c\u005c3){1}(a))b\u0022}"
        "(?<=(\u005c3()\u005c3){1}(a))b" flags="d" input="aaab" "{\u0022matches\u0022:[\u0022b\u0022,\u0022aa\u0022,\u0022\u0022,\u0022a\u0022],\u0022index\u0022:3,\u0022input\u0022:\u0022aaab\u0022,\u0022indices\u0022:[[3,4],[0,2],[1,1],[2,3]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=(\u005c\u005c3()\u005c\u005c3){1}(a))b\u0022}"
        "(a)(?=(\u005c1(\u005c1)){1})a" flags="d" input="aaa" "{\u0022matches\u0022:[\u0022aa\u0022,\u0022a\u0022,\u0022aa\u0022,\u0022a\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022aaa\u0022,\u0022indices\u0022:[[0,2],[0,1],[1,3],[2,3]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a)(?=(\u005c\u005c1(\u005c\u005c1)){1})a\u0022}"
        "(a)((\u005c1(\u005c1)){1}b)+" flags="d" input="aaabaab" "{\u0022matches\u0022:[\u0022aaabaab\u0022,\u0022a\u0022,\u0022aab\u0022,\u0022aa\u0022,\u0022a\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022aaabaab\u0022,\u0022indices\u0022:[[0,7],[0,1],[4,7],[4,6],[5,6]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a)((\u005c\u005c1(\u005c\u005c1)){1}b)+\u0022}"
        "(a)(((\u005c1)\u005c1){1}b)+" flags="d" input="aaabaab" "{\u0022matches\u0022:[\u0022aaabaab\u0022,\u0022a\u0022,\u0022aab\u0022,\u0022aa\u0022,\u0022a\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022aaabaab\u0022,\u0022indices\u0022:[[0,7],[0,1],[4,7],[4,6],[4,5]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a)(((\u005c\u005c1)\u005c\u005c1){1}b)+\u0022}"
        "(a)((\u005c1()\u005c1){1}b)+" flags="d" input="aaabaab" "{\u0022matches\u0022:[\u0022aaabaab\u0022,\u0022a\u0022,\u0022aab\u0022,\u0022aa\u0022,\u0022\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022aaabaab\u0022,\u0022indices\u0022:[[0,7],[0,1],[4,7],[4,6],[5,5]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a)((\u005c\u005c1()\u005c\u005c1){1}b)+\u0022}"
        "(?=(\u005c3(\u005c3)){1}(a))a" flags="d" input="a" "{\u0022matches\u0022:[\u0022a\u0022,\u0022\u0022,\u0022\u0022,\u0022a\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022a\u0022,\u0022indices\u0022:[[0,1],[0,0],[0,0],[0,1]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?=(\u005c\u005c3(\u005c\u005c3)){1}(a))a\u0022}"
        "(?<=(a)(\u005c1(\u005c1)){1})b" flags="d" input="ab" "{\u0022matches\u0022:[\u0022b\u0022,\u0022a\u0022,\u0022\u0022,\u0022\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022ab\u0022,\u0022indices\u0022:[[1,2],[0,1],[1,1],[1,1]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=(a)(\u005c\u005c1(\u005c\u005c1)){1})b\u0022}"
        "(?<=a(?!(b)(\u005c1(\u005c1)){1}))b" flags="d" input="ab" "{\u0022matches\u0022:[\u0022b\u0022,null,null,null],\u0022index\u0022:1,\u0022input\u0022:\u0022ab\u0022,\u0022indices\u0022:[[1,2],null,null,null],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=a(?!(b)(\u005c\u005c1(\u005c\u005c1)){1}))b\u0022}"
        "(a)(?!(\u005c1(\u005c1)){1})b" flags="d" input="ab" "{\u0022matches\u0022:[\u0022ab\u0022,\u0022a\u0022,null,null],\u0022index\u0022:0,\u0022input\u0022:\u0022ab\u0022,\u0022indices\u0022:[[0,2],[0,1],null,null],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a)(?!(\u005c\u005c1(\u005c\u005c1)){1})b\u0022}"
        "(?<=a(?=(b)(?:(\u005c1)\u005c1){1}))b" flags="d" input="abbb" "{\u0022matches\u0022:[\u0022b\u0022,\u0022b\u0022,\u0022b\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022abbb\u0022,\u0022indices\u0022:[[1,2],[1,2],[2,3]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=a(?=(b)(?:(\u005c\u005c1)\u005c\u005c1){1}))b\u0022}"
        "(?<=a(?=(?<x>b)(?<y>\u005ck<x>(?<z>\u005ck<x>)){1}))b" flags="d" input="abbb" "{\u0022matches\u0022:[\u0022b\u0022,\u0022b\u0022,\u0022bb\u0022,\u0022b\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022abbb\u0022,\u0022groups\u0022:{\u0022x\u0022:\u0022b\u0022,\u0022y\u0022:\u0022bb\u0022,\u0022z\u0022:\u0022b\u0022},\u0022indices\u0022:[[1,2],[1,2],[2,4],[3,4]],\u0022indicesGroups\u0022:{\u0022x\u0022:[1,2],\u0022y\u0022:[2,4],\u0022z\u0022:[3,4]},\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=a(?=(?<x>b)(?<y>\u005c\u005ck<x>(?<z>\u005c\u005ck<x>)){1}))b\u0022}"
        "(?<=a(?=(?<x>b)(?<y>\u005ck<x>()\u005ck<x>){1}))b" flags="d" input="abbb" "{\u0022matches\u0022:[\u0022b\u0022,\u0022b\u0022,\u0022bb\u0022,\u0022\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022abbb\u0022,\u0022groups\u0022:{\u0022x\u0022:\u0022b\u0022,\u0022y\u0022:\u0022bb\u0022},\u0022indices\u0022:[[1,2],[1,2],[2,4],[3,3]],\u0022indicesGroups\u0022:{\u0022x\u0022:[1,2],\u0022y\u0022:[2,4]},\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=a(?=(?<x>b)(?<y>\u005c\u005ck<x>()\u005c\u005ck<x>){1}))b\u0022}"
        "(?<=(?<x>\u005ck<z>(?<y>\u005ck<z>)){1}(?<z>a))b" flags="d" input="aaab" "{\u0022matches\u0022:[\u0022b\u0022,\u0022aa\u0022,\u0022a\u0022,\u0022a\u0022],\u0022index\u0022:3,\u0022input\u0022:\u0022aaab\u0022,\u0022groups\u0022:{\u0022x\u0022:\u0022aa\u0022,\u0022y\u0022:\u0022a\u0022,\u0022z\u0022:\u0022a\u0022},\u0022indices\u0022:[[3,4],[0,2],[1,2],[2,3]],\u0022indicesGroups\u0022:{\u0022x\u0022:[0,2],\u0022y\u0022:[1,2],\u0022z\u0022:[2,3]},\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=(?<x>\u005c\u005ck<z>(?<y>\u005c\u005ck<z>)){1}(?<z>a))b\u0022}"
        "(?<=(?<x>a)(?<y>\u005ck<x>(?<z>\u005ck<x>)){1})b" flags="d" input="ab" "{\u0022matches\u0022:[\u0022b\u0022,\u0022a\u0022,\u0022\u0022,\u0022\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022ab\u0022,\u0022groups\u0022:{\u0022x\u0022:\u0022a\u0022,\u0022y\u0022:\u0022\u0022,\u0022z\u0022:\u0022\u0022},\u0022indices\u0022:[[1,2],[0,1],[1,1],[1,1]],\u0022indicesGroups\u0022:{\u0022x\u0022:[0,1],\u0022y\u0022:[1,1],\u0022z\u0022:[1,1]},\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=(?<x>a)(?<y>\u005c\u005ck<x>(?<z>\u005c\u005ck<x>)){1})b\u0022}"
        "(?<=a(?!(?<x>b)(?<y>\u005ck<x>(?<z>\u005ck<x>)){1}))b" flags="d" input="ab" "{\u0022matches\u0022:[\u0022b\u0022,null,null,null],\u0022index\u0022:1,\u0022input\u0022:\u0022ab\u0022,\u0022groups\u0022:{},\u0022indices\u0022:[[1,2],null,null,null],\u0022indicesGroups\u0022:{},\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=a(?!(?<x>b)(?<y>\u005c\u005ck<x>(?<z>\u005c\u005ck<x>)){1}))b\u0022}"
        "#);
}

#[test]
fn partial_ranges_names_indices_original_programs_and_negative_rollback_are_exact() {
    assert_true(
        r#"
            let a = /(?<=a(?=(?<x>b)(?<y>\k<x>(?<z>\k<x>)){1}))b/d.exec('abbb');
            a.groups.x === 'b' && a.groups.y === 'bb' && a.groups.z === 'b' && a.indices.groups.z === a
                .indices[3] && a.indices[3][0] === 3 && a.indices[3][1] === 4
        "#,
    );
    assert_true(
        r#"
            let a = /(?<=(?<x>\k<z>(?<y>\k<z>)){1}(?<z>a))b/d.exec('aaab');
            a.groups.x === 'aa' && a.groups.y === 'a' && a.indices[2][0] === 1 && a.indices[3][0] === 2
        "#,
    );
    assert_true(
        r#"
            let a = /(?<=a(?=(b)(\1()\1){1}))b/d.exec('abbb');
            a[2] === 'bb' && a[3] === '' && a.indices[3][0] === 3 && a.indices[3][1] === 3
        "#,
    );
    assert_true(
        "let a=/(?<=a(?!(b)(\\1(\\1)){1}))b/d.exec('ab');a[1]===undefined&&a[2]===undefined&&a.indices[3]===undefined",
    );
    assert_true(
        r#"
            let a = /(?<=a(?=([\uD800])(\1(\1)){1}))[\uD800]/d.exec('a\uD800\uD800\uD800');
            a.index === 1 && a[2].length === 2 && a[3].charCodeAt(0) === 55296 && a.indices[3][0] === 3
        "#,
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
    assert_true(
        r#"
            let r = /(?<=a(?=(b)(\1(\1)){1}))b/dy;
            r.lastIndex = 1;
            let a = r.exec('abbb');
            a[3] === 'b' && a.indices[3][0] === 3 && r.lastIndex === 2 && r.exec('abbb') === null && r
                .lastIndex === 0
        "#,
    );
    assert_true(
        r#"
            let r = /(?<=a(?=(b)(\1(\1)){1}))b/dg,
                a = [...'abbb abbb'.matchAll(r)];
            a.length === 2 && a[0].index === 1 && a[1].index === 6 && a[1].indices[3][0] === 8 && r
                .lastIndex === 0
        "#,
    );
    assert_true(
        r#"
            let seen = [];
            let s = 'abbb abbb'.replace(/(?<=a(?=(b)(\1(\1)){1}))b/g, (m, x, y, z, i) => {
                seen.push(x, y, z, i);
                return '_'
            });
            s === 'a_bb a_bb' && seen.join('|') === 'b|bb|b|1|b|bb|b|6'
        "#,
    );
    assert_true(
        "'abbb'.search(/(?<=a(?=(b)(\\1(\\1)){1}))b/)===1&&'abbb'.split(/(?<=a(?=(b)(\\1(\\1)){1}))b/).join('|')==='a|b|bb|b|bb'",
    );
    assert_true(
        "let a=/(a)((\\1(\\1)){1}b)+/d.exec('aaabaab');a[2]==='aab'&&a[3]==='aa'&&a[4]==='a'&&a.indices[4][0]===5",
    );
    assert_true("let a=/(?=(\\3(\\3)){1}(a))a/d.exec('a');a[1]===''&&a[2]===''&&a[3]==='a'");
    assert_true("/(?<=a(?=(b)(\\1(\\1)){1}))b/.exec('abb')===null");
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
        r"/(?<=a(?=(b)(\1(\1)){2,3}))b/.exec('abbbbbb')",
        r"/(?<=a(?=(b)(\1(\1)){0,1}))b/.exec('abbb')",
        r"/(?<=a(?=(b)(\1(\1)){1}))b/u.exec('abbb')",
        r"/(?<=a(?=(b)(\1(\1)){1}))b/v.exec('abbb')",
    ] {
        crate::common::regexp_oracle::assert_matches_reference(&mut Realm::default(), source);
    }
}
