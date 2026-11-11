//! Count-one linear assertion units with Local, Open and Future references.
use crate::common::assert_true;
use spite_core::JsString;
use spite_runtime::{Error, Limits, Realm, Value};
use std::fmt::Write;
#[test]
fn linear_reference_assertions_result_snapshot() {
    let mut rows = String::new();
    for (source, flags, text) in [
        (r"(?<=a(?=(b)\1))b", "d", "abb"),
        (r"(?<=a(?=(b)\1))b", "d", "abbbb"),
        (r"(?<=a(?=(b)(\1)\2))b", "d", "abbb"),
        (r"(?<=a(?=(b)\1\1))b", "d", "abbb"),
        (r"(?<=a(?=(\1b)))b", "d", "ab"),
        (r"(?<=a(?=(\2(b))))b", "d", "ab"),
        (r"(?<=a(?!((b)\2)q))b", "d", "abbx"),
        (r"(?<=(\2)(b))c", "d", "bbc"),
        (r"(?<=(\2)(\3)(b))c", "d", "bbbc"),
        (r"(?<=(a)\1)b", "d", "aab"),
        (r"(?<=(\1))a", "d", "a"),
        (r"(a)(?<=a(?=(b)\2\1))b", "d", "abbab"),
        (r"(a)(?<=(\3(b)\1))c", "d", "bbac"),
        (r"(ab)(?<=b(?=(\1)\2))a", "d", "ababab"),
        (r"(a)(?<=(\3)(\1))c", "d", "aaac"),
        (r"(?<=a(?=(?<x>b)\k<x>))b", "d", "abb"),
        (r"(?<=(?<x>\k<y>)(?<y>b))c", "d", "bbc"),
        (r"(?<=(?<x>a)\k<x>)b", "d", "aab"),
        (r"(?<=(?<x>\k<x>))a", "d", "a"),
        (r"(?<=a(?=(b)\1))b", "di", "aBb"),
        (r"(?<=a(?!(b)\1))b|(?<=a)b", "d", "abb"),
        (r"(?<!(\2)(b))c|c", "d", "bbc"),
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
        "(?<=a(?=(b)\u005c1))b" flags="d" input="abb" "{\u0022matches\u0022:[\u0022b\u0022,\u0022b\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022abb\u0022,\u0022indices\u0022:[[1,2],[1,2]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=a(?=(b)\u005c\u005c1))b\u0022}"
        "(?<=a(?=(b)\u005c1))b" flags="d" input="abbbb" "{\u0022matches\u0022:[\u0022b\u0022,\u0022b\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022abbbb\u0022,\u0022indices\u0022:[[1,2],[1,2]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=a(?=(b)\u005c\u005c1))b\u0022}"
        "(?<=a(?=(b)(\u005c1)\u005c2))b" flags="d" input="abbb" "{\u0022matches\u0022:[\u0022b\u0022,\u0022b\u0022,\u0022b\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022abbb\u0022,\u0022indices\u0022:[[1,2],[1,2],[2,3]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=a(?=(b)(\u005c\u005c1)\u005c\u005c2))b\u0022}"
        "(?<=a(?=(b)\u005c1\u005c1))b" flags="d" input="abbb" "{\u0022matches\u0022:[\u0022b\u0022,\u0022b\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022abbb\u0022,\u0022indices\u0022:[[1,2],[1,2]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=a(?=(b)\u005c\u005c1\u005c\u005c1))b\u0022}"
        "(?<=a(?=(\u005c1b)))b" flags="d" input="ab" "{\u0022matches\u0022:[\u0022b\u0022,\u0022b\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022ab\u0022,\u0022indices\u0022:[[1,2],[1,2]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=a(?=(\u005c\u005c1b)))b\u0022}"
        "(?<=a(?=(\u005c2(b))))b" flags="d" input="ab" "{\u0022matches\u0022:[\u0022b\u0022,\u0022b\u0022,\u0022b\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022ab\u0022,\u0022indices\u0022:[[1,2],[1,2],[1,2]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=a(?=(\u005c\u005c2(b))))b\u0022}"
        "(?<=a(?!((b)\u005c2)q))b" flags="d" input="abbx" "{\u0022matches\u0022:[\u0022b\u0022,null,null],\u0022index\u0022:1,\u0022input\u0022:\u0022abbx\u0022,\u0022indices\u0022:[[1,2],null,null],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=a(?!((b)\u005c\u005c2)q))b\u0022}"
        "(?<=(\u005c2)(b))c" flags="d" input="bbc" "{\u0022matches\u0022:[\u0022c\u0022,\u0022b\u0022,\u0022b\u0022],\u0022index\u0022:2,\u0022input\u0022:\u0022bbc\u0022,\u0022indices\u0022:[[2,3],[0,1],[1,2]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=(\u005c\u005c2)(b))c\u0022}"
        "(?<=(\u005c2)(\u005c3)(b))c" flags="d" input="bbbc" "{\u0022matches\u0022:[\u0022c\u0022,\u0022b\u0022,\u0022b\u0022,\u0022b\u0022],\u0022index\u0022:3,\u0022input\u0022:\u0022bbbc\u0022,\u0022indices\u0022:[[3,4],[0,1],[1,2],[2,3]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=(\u005c\u005c2)(\u005c\u005c3)(b))c\u0022}"
        "(?<=(a)\u005c1)b" flags="d" input="aab" "{\u0022matches\u0022:[\u0022b\u0022,\u0022a\u0022],\u0022index\u0022:2,\u0022input\u0022:\u0022aab\u0022,\u0022indices\u0022:[[2,3],[1,2]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=(a)\u005c\u005c1)b\u0022}"
        "(?<=(\u005c1))a" flags="d" input="a" "{\u0022matches\u0022:[\u0022a\u0022,\u0022\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022a\u0022,\u0022indices\u0022:[[0,1],[0,0]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=(\u005c\u005c1))a\u0022}"
        "(a)(?<=a(?=(b)\u005c2\u005c1))b" flags="d" input="abbab" "{\u0022matches\u0022:[\u0022ab\u0022,\u0022a\u0022,\u0022b\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022abbab\u0022,\u0022indices\u0022:[[0,2],[0,1],[1,2]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a)(?<=a(?=(b)\u005c\u005c2\u005c\u005c1))b\u0022}"
        "(a)(?<=(\u005c3(b)\u005c1))c" flags="d" input="bbac" "{\u0022matches\u0022:[\u0022ac\u0022,\u0022a\u0022,\u0022bba\u0022,\u0022b\u0022],\u0022index\u0022:2,\u0022input\u0022:\u0022bbac\u0022,\u0022indices\u0022:[[2,4],[2,3],[0,3],[1,2]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a)(?<=(\u005c\u005c3(b)\u005c\u005c1))c\u0022}"
        "(ab)(?<=b(?=(\u005c1)\u005c2))a" flags="d" input="ababab" "{\u0022matches\u0022:[\u0022aba\u0022,\u0022ab\u0022,\u0022ab\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022ababab\u0022,\u0022indices\u0022:[[0,3],[0,2],[2,4]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(ab)(?<=b(?=(\u005c\u005c1)\u005c\u005c2))a\u0022}"
        "(a)(?<=(\u005c3)(\u005c1))c" flags="d" input="aaac" "{\u0022matches\u0022:[\u0022ac\u0022,\u0022a\u0022,\u0022a\u0022,\u0022a\u0022],\u0022index\u0022:2,\u0022input\u0022:\u0022aaac\u0022,\u0022indices\u0022:[[2,4],[2,3],[1,2],[2,3]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a)(?<=(\u005c\u005c3)(\u005c\u005c1))c\u0022}"
        "(?<=a(?=(?<x>b)\u005ck<x>))b" flags="d" input="abb" "{\u0022matches\u0022:[\u0022b\u0022,\u0022b\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022abb\u0022,\u0022groups\u0022:{\u0022x\u0022:\u0022b\u0022},\u0022indices\u0022:[[1,2],[1,2]],\u0022indicesGroups\u0022:{\u0022x\u0022:[1,2]},\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=a(?=(?<x>b)\u005c\u005ck<x>))b\u0022}"
        "(?<=(?<x>\u005ck<y>)(?<y>b))c" flags="d" input="bbc" "{\u0022matches\u0022:[\u0022c\u0022,\u0022b\u0022,\u0022b\u0022],\u0022index\u0022:2,\u0022input\u0022:\u0022bbc\u0022,\u0022groups\u0022:{\u0022x\u0022:\u0022b\u0022,\u0022y\u0022:\u0022b\u0022},\u0022indices\u0022:[[2,3],[0,1],[1,2]],\u0022indicesGroups\u0022:{\u0022x\u0022:[0,1],\u0022y\u0022:[1,2]},\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=(?<x>\u005c\u005ck<y>)(?<y>b))c\u0022}"
        "(?<=(?<x>a)\u005ck<x>)b" flags="d" input="aab" "{\u0022matches\u0022:[\u0022b\u0022,\u0022a\u0022],\u0022index\u0022:2,\u0022input\u0022:\u0022aab\u0022,\u0022groups\u0022:{\u0022x\u0022:\u0022a\u0022},\u0022indices\u0022:[[2,3],[1,2]],\u0022indicesGroups\u0022:{\u0022x\u0022:[1,2]},\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=(?<x>a)\u005c\u005ck<x>)b\u0022}"
        "(?<=(?<x>\u005ck<x>))a" flags="d" input="a" "{\u0022matches\u0022:[\u0022a\u0022,\u0022\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022a\u0022,\u0022groups\u0022:{\u0022x\u0022:\u0022\u0022},\u0022indices\u0022:[[0,1],[0,0]],\u0022indicesGroups\u0022:{\u0022x\u0022:[0,0]},\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=(?<x>\u005c\u005ck<x>))a\u0022}"
        "(?<=a(?=(b)\u005c1))b" flags="di" input="aBb" "{\u0022matches\u0022:[\u0022B\u0022,\u0022B\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022aBb\u0022,\u0022indices\u0022:[[1,2],[1,2]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=a(?=(b)\u005c\u005c1))b\u0022}"
        "(?<=a(?!(b)\u005c1))b|(?<=a)b" flags="d" input="abb" "{\u0022matches\u0022:[\u0022b\u0022,null],\u0022index\u0022:1,\u0022input\u0022:\u0022abb\u0022,\u0022indices\u0022:[[1,2],null],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=a(?!(b)\u005c\u005c1))b|(?<=a)b\u0022}"
        "(?<!(\u005c2)(b))c|c" flags="d" input="bbc" "{\u0022matches\u0022:[\u0022c\u0022,null,null],\u0022index\u0022:2,\u0022input\u0022:\u0022bbc\u0022,\u0022indices\u0022:[[2,3],null,null],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<!(\u005c\u005c2)(b))c|c\u0022}"
        "#);
}
#[test]
fn previous_gap_programs_snapshot() {
    let mut rows = String::new();
    for source in [
        r"/(?<=(\1))a/.exec('a')",
        r"/(?<=a(?=(b)\1))b/.exec('abb')",
        r"/(?<=(a)\1)b/.exec('aab')",
        r"/(?<=(\1))a/.exec('a')",
        r"/(?<=a(?=(b)\1))b/.exec('abbbb')",
        r"/(?<=(\1))a/.exec('a')",
        r"/(?<=a(?=(b)\1))b/.exec('abb')",
        r"/(?<=(\1))a/.exec('a')",
    ] {
        let source = JsString::from(source);
        let program = format!("JSON.stringify({{result:eval({source:?})}})");
        let Value::String(result) = Realm::default().eval(&program).unwrap() else {
            panic!("expected JSON")
        };
        writeln!(rows, "{source:?} {result:?}").unwrap();
    }
    insta::assert_snapshot!(rows, @r#"
        "/(?<=(\u005c1))a/.exec('a')" "{\u0022result\u0022:[\u0022a\u0022,\u0022\u0022]}"
        "/(?<=a(?=(b)\u005c1))b/.exec('abb')" "{\u0022result\u0022:[\u0022b\u0022,\u0022b\u0022]}"
        "/(?<=(a)\u005c1)b/.exec('aab')" "{\u0022result\u0022:[\u0022b\u0022,\u0022a\u0022]}"
        "/(?<=(\u005c1))a/.exec('a')" "{\u0022result\u0022:[\u0022a\u0022,\u0022\u0022]}"
        "/(?<=a(?=(b)\u005c1))b/.exec('abbbb')" "{\u0022result\u0022:[\u0022b\u0022,\u0022b\u0022]}"
        "/(?<=(\u005c1))a/.exec('a')" "{\u0022result\u0022:[\u0022a\u0022,\u0022\u0022]}"
        "/(?<=a(?=(b)\u005c1))b/.exec('abb')" "{\u0022result\u0022:[\u0022b\u0022,\u0022b\u0022]}"
        "/(?<=(\u005c1))a/.exec('a')" "{\u0022result\u0022:[\u0022a\u0022,\u0022\u0022]}"
        "#);
}
#[test]
fn local_open_future_named_ranges_imports_negative_commit_and_surrogates_are_exact() {
    assert_true(
        r"let a=/(?<=a(?=(?<x>b)\k<x>))b/d.exec('abb');a.groups.x==='b'&&a.indices.groups.x===a.indices[1]&&a.indices[1][0]===1",
    );
    assert_true(
        r"let a=/(?<=(?<x>\k<y>)(?<y>b))c/d.exec('bbc');a.groups.x==='b'&&a.groups.y==='b'&&a.indices.groups.x[0]===0&&a.indices.groups.y[0]===1",
    );
    assert_true(
        r"let a=/(?<=(?<x>\k<x>))a/d.exec('a');a.groups.x===''&&a.indices.groups.x[0]===0&&a.indices.groups.x[1]===0",
    );
    assert_true(
        r"let a=/(?<=a(?!(b)\1))b|(?<=a)b/d.exec('abb');a[1]===undefined&&a.indices[1]===undefined",
    );
    assert_true(
        r"let a=/(?<!(\2)(b))c|c/d.exec('bbc');a[1]===undefined&&a[2]===undefined&&a.indices[1]===undefined",
    );
    assert_true(
        r"let a=/(ab)(?<=b(?=(\1)\2))a/d.exec('ababab');a[1]==='ab'&&a[2]==='ab'&&a.indices[1][0]===0&&a.indices[2][0]===2",
    );
    assert_true(
        r"let a=/(?<=a(?=([\uD800])\1))[\uD800]/d.exec('a\uD800\uD800\uDC00');a.index===1&&a.indices[1][0]===1",
    );
}
#[test]
fn consumers_global_sticky_empty_advance_and_callbacks_preserve_reference_positions() {
    assert_true(
        r"let r=/(?<=a(?=(b)\1))b/dy;r.lastIndex=1;let a=r.exec('abb');a.index===1&&r.lastIndex===2&&r.exec('abb')===null&&r.lastIndex===0",
    );
    assert_true(
        r"let r=/(?<=a(?=(b)\1))b/dg,a=[...'abb abb'.matchAll(r)];a.length===2&&a[1].index===5&&a[1].indices[1][0]===5&&r.lastIndex===0",
    );
    assert_true(
        r"let a=[...'abb'.matchAll(/(?<=a(?=(b)\1))/dg)];a.length===1&&a[0].index===1&&a[0][0]===''&&a[0].indices[1][0]===1",
    );
    assert_true(
        r"let seen=[];let s='abb abb'.replace(/(?<=a(?=(b)\1))b/g,(m,x,i)=>{seen.push(x,i);return '_'});s==='a_b a_b'&&seen.join('|')==='b|1|b|5'",
    );
    assert_true(
        r"'abb'.search(/(?<=a(?=(b)\1))b/)===1&&'abb'.split(/(?<=a(?=(b)\1))b/).join('|')==='a|b|b'",
    );
}

#[test]
fn explicit_work_abort_keeps_last_index_and_bypasses_language_handlers() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval(r"let marker=0,r=/(?<=a(?=(b)\1))b/g;r.lastIndex=1;let text='b'.repeat(5000)+'abb'")
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
fn unproved_counts_choices_owner_reads_and_unicode_remain_pending() {
    for source in [
        r"/(?<=a(?=(b{1,2})\1))b/.exec('abbbb')",
        r"/(?<=(\1{1,2}a{1,2}))a/.exec('a')",
        r"/(?<=(a{1,2})\1)b/.exec('aab')",
        r"/(?<=a(?=(b|aa)\1))b/.exec('abb')",
        r"/(a)(?<=((b)(?=\2)))c/.exec('abc')",
        r"/(?<=a(?=(b)\1))b/u.exec('abb')",
        r"/(?<=a(?=(b)\1))b/v.exec('abb')",
    ] {
        crate::common::regexp_oracle::assert_matches_reference(&mut Realm::default(), source);
    }
}
