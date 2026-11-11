//! Proved empty counted reference terms in ordinary assertions.
use crate::common::assert_true;
use spite_core::JsString;
use spite_runtime::{Error, Limits, Realm, Value};
use std::fmt::Write;
#[test]
fn undefined_counted_assertions_result_snapshot() {
    let mut rows = String::new();
    for (source, flags, text) in [
        (r"(?<=(\1{1,2}))a", "d", "a"),
        (r"(?<=(\1{1,2}?))a", "d", "a"),
        (r"(?<=(a)\1*?)b", "d", "ab"),
        (r"(?<=(\1*))a", "d", "a"),
        (r"(?<=(\1{0}))a", "d", "a"),
        (r"(?<=a(?=(\1{1,2}b)))b", "d", "ab"),
        (r"(?<=a(?=(\2*(b))))b", "d", "ab"),
        (r"(?<=a(?=(\2{1,2}(b))))b", "d", "ab"),
        (r"(?<=(a)\1{1,2})b", "d", "ab"),
        (r"(?<=(a)\1*)b", "d", "ab"),
        (r"(?<=(a)\1{0})b", "d", "ab"),
        (r"(?<=(\2{0})(b))c", "d", "bc"),
        (r"(?<=a(?=(b)\1{0}))b", "d", "ab"),
        (r"(?<=a(?!((\2{1,2})b)q))b", "d", "abbx"),
        (r"(a)(?<=(b)\2{1,2}\1)c", "d", "bac"),
        (r"(a)(?<=a(?=(\2*\1)))a", "d", "aa"),
        (r"(?<=(?<x>\k<x>{1,2}))a", "d", "a"),
        (r"(?<=(?<x>a)\k<x>*)b", "d", "ab"),
        (r"(?<=a(?=(\k<x>{1,2}(?<x>b))))b", "d", "ab"),
        (r"(?<=(\k<x>{0})(?<x>b))c", "d", "bc"),
        (r"(?<=(a)\1*)b", "di", "Ab"),
        (r"(?<!(\1{1,2}))a|a", "d", "a"),
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
        "(?<=(\u005c1{1,2}))a" flags="d" input="a" "{\u0022matches\u0022:[\u0022a\u0022,\u0022\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022a\u0022,\u0022indices\u0022:[[0,1],[0,0]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=(\u005c\u005c1{1,2}))a\u0022}"
        "(?<=(\u005c1{1,2}?))a" flags="d" input="a" "{\u0022matches\u0022:[\u0022a\u0022,\u0022\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022a\u0022,\u0022indices\u0022:[[0,1],[0,0]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=(\u005c\u005c1{1,2}?))a\u0022}"
        "(?<=(a)\u005c1*?)b" flags="d" input="ab" "{\u0022matches\u0022:[\u0022b\u0022,\u0022a\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022ab\u0022,\u0022indices\u0022:[[1,2],[0,1]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=(a)\u005c\u005c1*?)b\u0022}"
        "(?<=(\u005c1*))a" flags="d" input="a" "{\u0022matches\u0022:[\u0022a\u0022,\u0022\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022a\u0022,\u0022indices\u0022:[[0,1],[0,0]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=(\u005c\u005c1*))a\u0022}"
        "(?<=(\u005c1{0}))a" flags="d" input="a" "{\u0022matches\u0022:[\u0022a\u0022,\u0022\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022a\u0022,\u0022indices\u0022:[[0,1],[0,0]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=(\u005c\u005c1{0}))a\u0022}"
        "(?<=a(?=(\u005c1{1,2}b)))b" flags="d" input="ab" "{\u0022matches\u0022:[\u0022b\u0022,\u0022b\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022ab\u0022,\u0022indices\u0022:[[1,2],[1,2]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=a(?=(\u005c\u005c1{1,2}b)))b\u0022}"
        "(?<=a(?=(\u005c2*(b))))b" flags="d" input="ab" "{\u0022matches\u0022:[\u0022b\u0022,\u0022b\u0022,\u0022b\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022ab\u0022,\u0022indices\u0022:[[1,2],[1,2],[1,2]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=a(?=(\u005c\u005c2*(b))))b\u0022}"
        "(?<=a(?=(\u005c2{1,2}(b))))b" flags="d" input="ab" "{\u0022matches\u0022:[\u0022b\u0022,\u0022b\u0022,\u0022b\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022ab\u0022,\u0022indices\u0022:[[1,2],[1,2],[1,2]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=a(?=(\u005c\u005c2{1,2}(b))))b\u0022}"
        "(?<=(a)\u005c1{1,2})b" flags="d" input="ab" "{\u0022matches\u0022:[\u0022b\u0022,\u0022a\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022ab\u0022,\u0022indices\u0022:[[1,2],[0,1]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=(a)\u005c\u005c1{1,2})b\u0022}"
        "(?<=(a)\u005c1*)b" flags="d" input="ab" "{\u0022matches\u0022:[\u0022b\u0022,\u0022a\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022ab\u0022,\u0022indices\u0022:[[1,2],[0,1]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=(a)\u005c\u005c1*)b\u0022}"
        "(?<=(a)\u005c1{0})b" flags="d" input="ab" "{\u0022matches\u0022:[\u0022b\u0022,\u0022a\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022ab\u0022,\u0022indices\u0022:[[1,2],[0,1]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=(a)\u005c\u005c1{0})b\u0022}"
        "(?<=(\u005c2{0})(b))c" flags="d" input="bc" "{\u0022matches\u0022:[\u0022c\u0022,\u0022\u0022,\u0022b\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022bc\u0022,\u0022indices\u0022:[[1,2],[0,0],[0,1]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=(\u005c\u005c2{0})(b))c\u0022}"
        "(?<=a(?=(b)\u005c1{0}))b" flags="d" input="ab" "{\u0022matches\u0022:[\u0022b\u0022,\u0022b\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022ab\u0022,\u0022indices\u0022:[[1,2],[1,2]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=a(?=(b)\u005c\u005c1{0}))b\u0022}"
        "(?<=a(?!((\u005c2{1,2})b)q))b" flags="d" input="abbx" "{\u0022matches\u0022:[\u0022b\u0022,null,null],\u0022index\u0022:1,\u0022input\u0022:\u0022abbx\u0022,\u0022indices\u0022:[[1,2],null,null],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=a(?!((\u005c\u005c2{1,2})b)q))b\u0022}"
        "(a)(?<=(b)\u005c2{1,2}\u005c1)c" flags="d" input="bac" "{\u0022matches\u0022:[\u0022ac\u0022,\u0022a\u0022,\u0022b\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022bac\u0022,\u0022indices\u0022:[[1,3],[1,2],[0,1]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a)(?<=(b)\u005c\u005c2{1,2}\u005c\u005c1)c\u0022}"
        "(a)(?<=a(?=(\u005c2*\u005c1)))a" flags="d" input="aa" "{\u0022matches\u0022:[\u0022aa\u0022,\u0022a\u0022,\u0022a\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022aa\u0022,\u0022indices\u0022:[[0,2],[0,1],[1,2]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a)(?<=a(?=(\u005c\u005c2*\u005c\u005c1)))a\u0022}"
        "(?<=(?<x>\u005ck<x>{1,2}))a" flags="d" input="a" "{\u0022matches\u0022:[\u0022a\u0022,\u0022\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022a\u0022,\u0022groups\u0022:{\u0022x\u0022:\u0022\u0022},\u0022indices\u0022:[[0,1],[0,0]],\u0022indicesGroups\u0022:{\u0022x\u0022:[0,0]},\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=(?<x>\u005c\u005ck<x>{1,2}))a\u0022}"
        "(?<=(?<x>a)\u005ck<x>*)b" flags="d" input="ab" "{\u0022matches\u0022:[\u0022b\u0022,\u0022a\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022ab\u0022,\u0022groups\u0022:{\u0022x\u0022:\u0022a\u0022},\u0022indices\u0022:[[1,2],[0,1]],\u0022indicesGroups\u0022:{\u0022x\u0022:[0,1]},\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=(?<x>a)\u005c\u005ck<x>*)b\u0022}"
        "(?<=a(?=(\u005ck<x>{1,2}(?<x>b))))b" flags="d" input="ab" "{\u0022matches\u0022:[\u0022b\u0022,\u0022b\u0022,\u0022b\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022ab\u0022,\u0022groups\u0022:{\u0022x\u0022:\u0022b\u0022},\u0022indices\u0022:[[1,2],[1,2],[1,2]],\u0022indicesGroups\u0022:{\u0022x\u0022:[1,2]},\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=a(?=(\u005c\u005ck<x>{1,2}(?<x>b))))b\u0022}"
        "(?<=(\u005ck<x>{0})(?<x>b))c" flags="d" input="bc" "{\u0022matches\u0022:[\u0022c\u0022,\u0022\u0022,\u0022b\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022bc\u0022,\u0022groups\u0022:{\u0022x\u0022:\u0022b\u0022},\u0022indices\u0022:[[1,2],[0,0],[0,1]],\u0022indicesGroups\u0022:{\u0022x\u0022:[0,1]},\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=(\u005c\u005ck<x>{0})(?<x>b))c\u0022}"
        "(?<=(a)\u005c1*)b" flags="di" input="Ab" "{\u0022matches\u0022:[\u0022b\u0022,\u0022A\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022Ab\u0022,\u0022indices\u0022:[[1,2],[0,1]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=(a)\u005c\u005c1*)b\u0022}"
        "(?<!(\u005c1{1,2}))a|a" flags="d" input="a" "{\u0022matches\u0022:[\u0022a\u0022,null],\u0022index\u0022:0,\u0022input\u0022:\u0022a\u0022,\u0022indices\u0022:[[0,1],null],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<!(\u005c\u005c1{1,2}))a|a\u0022}"
        "#);
}
#[test]
fn named_alias_ranges_required_captures_negative_undo_and_consumers_are_exact() {
    assert_true(
        r#"
            let a = /(?<=(?<x>\k<x>{1,2}))a/d.exec('a');
            a[1] === '' && a.groups.x === '' && a.indices[1][0] === 0 && a.indices.groups.x === a.indices[
                1]
        "#,
    );
    assert_true(
        r#"
            let a = /(?<=a(?=(\k<x>*(?<x>b))))b/d.exec('ab');
            a[1] === 'b' && a.groups.x === 'b' && a.indices[2][0] === 1 && a.indices.groups.x === a
                .indices[2]
        "#,
    );
    assert_true(r"let a=/(?<!(\1{1,2}))a|a/d.exec('a');a[1]===undefined&&a.indices[1]===undefined");
    assert_true(
        r#"
            let r = /(?<=(a)\1*)b/dy;
            r.lastIndex = 1;
            let a = r.exec('ab');
            a[1] === 'a' && r.lastIndex === 2 && r.exec('ab') === null && r.lastIndex === 0
        "#,
    );
    assert_true(
        r#"
            let r = /(?<=(\1{1,2}))a/dg,
                a = [...'a a'.matchAll(r)];
            a.length === 2 && a[0][1] === '' && a[1].index === 2 && a[1].indices[1][0] === 2 && r
                .lastIndex === 0
        "#,
    );
    assert_true(
        r"let a=[...'a'.matchAll(/(?<=(\1*))/dg)];a.length===2&&a[0].index===0&&a[1].index===1&&a[1][1]===''",
    );
    assert_true(
        r#"
            let seen = [];
            let s = 'a a'.replace(/(?<=(\1{1,2}))a/g, (m, x, i) => {
                seen.push(x, i);
                return '_'
            });
            s === '_ _' && seen.join('|') === '|0||2'
        "#,
    );
    assert_true(r"'ab'.search(/(?<=(a)\1*)b/)===1&&'ab'.split(/(?<=(a)\1*)b/).join('|')==='a|a|'");
    assert_true(
        r#"
            let a = /(?<=a(?=(\2*([\uD800]))))[\uD800]/d.exec('a\uD800\uDC00');
            a[1] === a[2] && a.indices[2][0] === 1 && a.indices[2][1] === 2
        "#,
    );
}

#[test]
fn explicit_work_abort_keeps_last_index_and_bypasses_language_handlers() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval(r"let marker=0,r=/(?<=(a)\1*)b/g;r.lastIndex=1;let text='b'.repeat(5000)+'ab'")
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
fn consuming_counts_choices_and_unicode_remain_pending_invalid_reads_stay_syntax_errors() {
    for source in [
        r"/(?<=(\1{1,2}a{1,2}))a/.exec('a')",
        r"/(?<=a(?=(b)\1{1,2}))b/.exec('abb')",
        r"/(?<=(\2{1,2})(b))c/.exec('bbc')",
        r"/(?<=(\1{1,2}|a))a/.exec('a')",
        r"/(?<=(\1{1,2}))a/u.exec('a')",
        r"/(?<=(\1{1,2}))a/v.exec('a')",
    ] {
        crate::common::regexp_oracle::assert_matches_reference(&mut Realm::default(), source);
    }
    assert_true(
        r"let ok=false;try{new RegExp('(?<=\\9{0})a','u')}catch(e){ok=e instanceof SyntaxError}ok",
    );
}
