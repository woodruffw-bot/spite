//! Cleared local captures lying to the left of their backward reads.
use crate::common::assert_true;
use spite_core::JsString;
use spite_runtime::{Error, Limits, Realm, Value};
use std::fmt::Write;
#[test]
fn lookbehind_local_future_reads_result_snapshot() {
    let mut rows = String::new();
    for (source, flags, text) in [
        (r"(?<=((a)\2){2})b", "d", "aab"),
        (r"(?<=((a)\2){2}?)b", "d", "aaaab"),
        (r"(?<=((ab)\2){2})c", "d", "ababc"),
        (r"(?<=((a)(b)\2\3){2})c", "d", "ababc"),
        (r"(?<=((a)\2\2){2})b", "d", "aab"),
        (r"(?<=(([ab])\2){2})c", "d", "abc"),
        (r"(?<=((.)\2){2})b", "ds", "a\nb"),
        (r"(?<=((a)\2\B){2})b", "d", "aab"),
        (r"(?<=((a)\2){0})b", "d", "b"),
        (r"(?<!((a)\2){2}q)b", "d", "aaxb"),
        (r"(?<=((a)\2){2}(?=(b)))b", "d", "aab"),
        (r"(b)(?<=((a)\3){2}\1)c", "d", "aabc"),
        (r"(?<=((a)\2){2})b", "di", "aAb"),
        (r"(?<=(?<y>(?<x>a)\k<x>){2})b", "d", "aab"),
        (r"(?=(((a)\3){2}))a", "d", "aaaab"),
        (r"(?<=((a)\2){2})b", "d", "ab"),
        (r"(?<=(([\uD800])\2){2})[\uDC00]", "d", "surrogates"),
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
        "(?<=((a)\u005c2){2})b" flags="d" input="aab" "{\u0022matches\u0022:[\u0022b\u0022,\u0022a\u0022,\u0022a\u0022],\u0022index\u0022:2,\u0022input\u0022:\u0022aab\u0022,\u0022indices\u0022:[[2,3],[0,1],[0,1]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=((a)\u005c\u005c2){2})b\u0022}"
        "(?<=((a)\u005c2){2}?)b" flags="d" input="aaaab" "{\u0022matches\u0022:[\u0022b\u0022,\u0022a\u0022,\u0022a\u0022],\u0022index\u0022:4,\u0022input\u0022:\u0022aaaab\u0022,\u0022indices\u0022:[[4,5],[2,3],[2,3]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=((a)\u005c\u005c2){2}?)b\u0022}"
        "(?<=((ab)\u005c2){2})c" flags="d" input="ababc" "{\u0022matches\u0022:[\u0022c\u0022,\u0022ab\u0022,\u0022ab\u0022],\u0022index\u0022:4,\u0022input\u0022:\u0022ababc\u0022,\u0022indices\u0022:[[4,5],[0,2],[0,2]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=((ab)\u005c\u005c2){2})c\u0022}"
        "(?<=((a)(b)\u005c2\u005c3){2})c" flags="d" input="ababc" "{\u0022matches\u0022:[\u0022c\u0022,\u0022ab\u0022,\u0022a\u0022,\u0022b\u0022],\u0022index\u0022:4,\u0022input\u0022:\u0022ababc\u0022,\u0022indices\u0022:[[4,5],[0,2],[0,1],[1,2]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=((a)(b)\u005c\u005c2\u005c\u005c3){2})c\u0022}"
        "(?<=((a)\u005c2\u005c2){2})b" flags="d" input="aab" "{\u0022matches\u0022:[\u0022b\u0022,\u0022a\u0022,\u0022a\u0022],\u0022index\u0022:2,\u0022input\u0022:\u0022aab\u0022,\u0022indices\u0022:[[2,3],[0,1],[0,1]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=((a)\u005c\u005c2\u005c\u005c2){2})b\u0022}"
        "(?<=(([ab])\u005c2){2})c" flags="d" input="abc" "{\u0022matches\u0022:[\u0022c\u0022,\u0022a\u0022,\u0022a\u0022],\u0022index\u0022:2,\u0022input\u0022:\u0022abc\u0022,\u0022indices\u0022:[[2,3],[0,1],[0,1]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=(([ab])\u005c\u005c2){2})c\u0022}"
        "(?<=((.)\u005c2){2})b" flags="ds" input="a\u000ab" "{\u0022matches\u0022:[\u0022b\u0022,\u0022a\u0022,\u0022a\u0022],\u0022index\u0022:2,\u0022input\u0022:\u0022a\u005cnb\u0022,\u0022indices\u0022:[[2,3],[0,1],[0,1]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=((.)\u005c\u005c2){2})b\u0022}"
        "(?<=((a)\u005c2\u005cB){2})b" flags="d" input="aab" "{\u0022matches\u0022:[\u0022b\u0022,\u0022a\u0022,\u0022a\u0022],\u0022index\u0022:2,\u0022input\u0022:\u0022aab\u0022,\u0022indices\u0022:[[2,3],[0,1],[0,1]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=((a)\u005c\u005c2\u005c\u005cB){2})b\u0022}"
        "(?<=((a)\u005c2){0})b" flags="d" input="b" "{\u0022matches\u0022:[\u0022b\u0022,null,null],\u0022index\u0022:0,\u0022input\u0022:\u0022b\u0022,\u0022indices\u0022:[[0,1],null,null],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=((a)\u005c\u005c2){0})b\u0022}"
        "(?<!((a)\u005c2){2}q)b" flags="d" input="aaxb" "{\u0022matches\u0022:[\u0022b\u0022,null,null],\u0022index\u0022:3,\u0022input\u0022:\u0022aaxb\u0022,\u0022indices\u0022:[[3,4],null,null],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<!((a)\u005c\u005c2){2}q)b\u0022}"
        "(?<=((a)\u005c2){2}(?=(b)))b" flags="d" input="aab" "{\u0022matches\u0022:[\u0022b\u0022,\u0022a\u0022,\u0022a\u0022,\u0022b\u0022],\u0022index\u0022:2,\u0022input\u0022:\u0022aab\u0022,\u0022indices\u0022:[[2,3],[0,1],[0,1],[2,3]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=((a)\u005c\u005c2){2}(?=(b)))b\u0022}"
        "(b)(?<=((a)\u005c3){2}\u005c1)c" flags="d" input="aabc" "{\u0022matches\u0022:[\u0022bc\u0022,\u0022b\u0022,\u0022a\u0022,\u0022a\u0022],\u0022index\u0022:2,\u0022input\u0022:\u0022aabc\u0022,\u0022indices\u0022:[[2,4],[2,3],[0,1],[0,1]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(b)(?<=((a)\u005c\u005c3){2}\u005c\u005c1)c\u0022}"
        "(?<=((a)\u005c2){2})b" flags="di" input="aAb" "{\u0022matches\u0022:[\u0022b\u0022,\u0022a\u0022,\u0022a\u0022],\u0022index\u0022:2,\u0022input\u0022:\u0022aAb\u0022,\u0022indices\u0022:[[2,3],[0,1],[0,1]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=((a)\u005c\u005c2){2})b\u0022}"
        "(?<=(?<y>(?<x>a)\u005ck<x>){2})b" flags="d" input="aab" "{\u0022matches\u0022:[\u0022b\u0022,\u0022a\u0022,\u0022a\u0022],\u0022index\u0022:2,\u0022input\u0022:\u0022aab\u0022,\u0022groups\u0022:{\u0022y\u0022:\u0022a\u0022,\u0022x\u0022:\u0022a\u0022},\u0022indices\u0022:[[2,3],[0,1],[0,1]],\u0022indicesGroups\u0022:{\u0022y\u0022:[0,1],\u0022x\u0022:[0,1]},\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=(?<y>(?<x>a)\u005c\u005ck<x>){2})b\u0022}"
        "(?=(((a)\u005c3){2}))a" flags="d" input="aaaab" "{\u0022matches\u0022:[\u0022a\u0022,\u0022aaaa\u0022,\u0022aa\u0022,\u0022a\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022aaaab\u0022,\u0022indices\u0022:[[0,1],[0,4],[2,4],[2,3]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?=(((a)\u005c\u005c3){2}))a\u0022}"
        "(?<=((a)\u005c2){2})b" flags="d" input="ab" "{\u0022match\u0022:null,\u0022lastIndex\u0022:0}"
        "(?<=(([\u005cuD800])\u005c2){2})[\u005cuDC00]" flags="d" input="\ud800\ud800\udc00" "{\u0022matches\u0022:[\u0022\u005cudc00\u0022,\u0022\u005cud800\u0022,\u0022\u005cud800\u0022],\u0022index\u0022:2,\u0022input\u0022:\u0022\u005cud800\ud800\udc00\u0022,\u0022indices\u0022:[[2,3],[0,1],[0,1]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=(([\u005c\u005cuD800])\u005c\u005c2){2})[\u005c\u005cuDC00]\u0022}"
        "#);
}
#[test]
fn names_aliases_backward_ranges_outside_imports_and_negative_rollback() {
    assert_true(
        r#"
            let a = /(?<=(?<y>(?<x>a)\k<x>){2})b/d.exec('aab');
            a.index === 2 && a.groups.x === 'a' && a.groups.y === 'a' && a.indices.groups.x === a.indices[
                2] && a.indices.groups.y[0] === 0 && a.indices.groups.x[1] === 1
        "#,
    );
    assert_true(
        r#"
            let a = /(?<=(?<y>(?<x>a)(?<z>b)\k<x>\k<z>){2})c/d.exec('ababc');
            a.groups.y === 'ab' && a.indices.groups.x[0] === 0 && a.indices.groups.z[0] === 1 && a.indices
                .groups.z === a.indices[3]
        "#,
    );
    assert_true(
        r#"
            let a = /(?<!(?<y>(?<x>a)\k<x>){2}q)b/d.exec('aaxb');
            a.index === 3 && a.groups.x === undefined && a.groups.y === undefined && a.indices.groups.x ===
                undefined && Object.hasOwn(a.groups, 'x')
        "#,
    );
    assert_true(
        r"let a=/(b)(?<=((a)\3){2}\1)c/d.exec('aabc');a.index===2&&a[1]==='b'&&a[2]==='a'&&a.indices[2][0]===0",
    );
    assert_true(
        r#"
            let a = /(?<=((a)\2){2}(?=(b)))b/d.exec('aab'),
                b = /(?=(((a)\3){2}))a/d.exec('aaaab');
            a.indices[3][0] === 2 && b[1] === 'aaaa' && b.indices[2][0] === 2 && b.indices[3][1] === 3
        "#,
    );
}
#[test]
fn sticky_global_empty_advancement_and_callbacks_keep_cleared_local_effects() {
    assert_true(
        r#"
            let r = /(?<=((a)\2){2})b/dy;
            r.lastIndex = 2;
            let a = r.exec('aab');
            a.index === 2 && r.lastIndex === 3 && r.exec('aab') === null && r.lastIndex === 0
        "#,
    );
    assert_true(
        r#"
            let r = /(?<=((a)\2){2})b/dg,
                a = [...'aab aab'.matchAll(r)];
            a.length === 2 && a[1].index === 6 && a[1].indices[2][0] === 4 && r.lastIndex === 0
        "#,
    );
    assert_true(
        r#"
            let r = /(?<=((a)\2){0})/dg,
                a = [...'ab'.matchAll(r)];
            a.length === 3 && a[2].index === 2 && a[2][1] === undefined && a[2].indices[2] === undefined
        "#,
    );
    assert_true(
        r#"
            let seen = [];
            let s = 'aab aab'.replace(/(?<=((a)\2){2})b/g, (m, x, y, i) => {
                seen.push(x, y, i);
                return '_'
            });
            s === 'aa_ aa_' && seen.join('|') === 'a|a|2|a|a|6'
        "#,
    );
    assert_true(
        r"'aab'.replace(/(?<=(?<y>(?<x>a)\k<x>){2})b/,'<$<y>>')==='aa<a>'&&'aab'.search(/(?<=((a)\2){2})b/)===2&&'aab'.split(/(?<=((a)\2){2})b/).join('|')==='aa|a|a|'",
    );
}

#[test]
fn explicit_work_aborts_preserve_last_index_and_bypass_handlers() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval(r"let marker=0,r=/(?<=((a)\2){2})b/g;r.lastIndex=1;let text='a'.repeat(5000)+'b'")
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
fn consuming_forward_open_reads_variable_counts_mixed_outside_units_and_unicode_remain_pending() {
    for source in [
        r"/(?<=(\2(a)){1,2})b/.exec('aaaab')",
        r"/(?<=((a)\1){1,2})b/.exec('aab')",
        r"/(?<=((a)\2){1,2})b/.exec('aab')",
        r"/(a)(?<=(\1(b)\3){1,2})c/.exec('ababc')",
        r"/(?<=((a)\2){2}(?=(((a)\5){1,2})))b/.exec('aaaab')",
        r"/(?<=((a)\2){2})b/u.exec('aab')",
        r"/(?<=((a)\2){2})b/v.exec('aab')",
    ] {
        crate::common::regexp_oracle::assert_matches_reference(&mut Realm::default(), source);
    }
}
