//! Effect-free flat exact-count reference atoms in assertions.
use crate::common::assert_true;
use spite_core::JsString;
use spite_runtime::{Error, Limits, Realm, Value};
use std::fmt::Write;
#[test]
fn exact_reference_assertions_result_snapshot() {
    let mut rows = String::new();
    for (source, flags, text) in [
        (r"(?<=a(?=(b)\1{2}))b", "d", "abbb"),
        (r"(?<=a(?=(b)\1{2}?))b", "d", "abbb"),
        (r"(?<=a(?=(b)(\1{2})\2{2}))b", "d", "abbbbbbb"),
        (r"(?<=(\2{2})(b))c", "d", "bbbc"),
        (r"(?<=(\2{2}?)(b))c", "d", "bbbc"),
        (r"(?<=(\2{2})(\3{2})(b))c", "d", "bbbbbbbc"),
        (r"(a)(?<=a(?=(b)\2{2}\1{2}))b", "d", "abbbaa"),
        (r"(a)(?<=(\3{2})(\1{2}))c", "d", "aaaaaac"),
        (r"(ab)(?<=b(?=(\1{2})\2{2}))a", "d", "ababababababab"),
        (r"(?<=(\1{2}))a", "d", "a"),
        (r"(?<=(a)\1{2})b", "d", "ab"),
        (r"(?<=a(?!((b)\2{2})q))b", "d", "abbbx"),
        (r"(?<!(\2{2})(b))c|c", "d", "bbbc"),
        (r"(a)(?<=\1{2}\1{2})b", "d", "aaaab"),
        (r"(a)(?<=\1\1{2})b", "d", "aaab"),
        (r"(a)(?<=a(?=\1{2}\1{2}))a", "d", "aaaaa"),
        (r"(?<=a(?=(b)(?:\1\1){2}))b", "d", "abbbbb"),
        (r"(?<=a(?=(?<x>b)\k<x>{2}))b", "d", "abbb"),
        (r"(?<=(\k<x>{2})(?<x>b))c", "d", "bbbc"),
        (r"(?<=(?<x>\k<x>{2}))a", "d", "a"),
        (r"(a)(?<=a(?=(?<x>b)\k<x>{2}\1{2}))b", "d", "abbbaa"),
        (r"(?<=a(?=(b)\1{2}))b", "di", "aBbb"),
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
        "(?<=a(?=(b)\u005c1{2}))b" flags="d" input="abbb" "{\u0022matches\u0022:[\u0022b\u0022,\u0022b\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022abbb\u0022,\u0022indices\u0022:[[1,2],[1,2]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=a(?=(b)\u005c\u005c1{2}))b\u0022}"
        "(?<=a(?=(b)\u005c1{2}?))b" flags="d" input="abbb" "{\u0022matches\u0022:[\u0022b\u0022,\u0022b\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022abbb\u0022,\u0022indices\u0022:[[1,2],[1,2]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=a(?=(b)\u005c\u005c1{2}?))b\u0022}"
        "(?<=a(?=(b)(\u005c1{2})\u005c2{2}))b" flags="d" input="abbbbbbb" "{\u0022matches\u0022:[\u0022b\u0022,\u0022b\u0022,\u0022bb\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022abbbbbbb\u0022,\u0022indices\u0022:[[1,2],[1,2],[2,4]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=a(?=(b)(\u005c\u005c1{2})\u005c\u005c2{2}))b\u0022}"
        "(?<=(\u005c2{2})(b))c" flags="d" input="bbbc" "{\u0022matches\u0022:[\u0022c\u0022,\u0022bb\u0022,\u0022b\u0022],\u0022index\u0022:3,\u0022input\u0022:\u0022bbbc\u0022,\u0022indices\u0022:[[3,4],[0,2],[2,3]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=(\u005c\u005c2{2})(b))c\u0022}"
        "(?<=(\u005c2{2}?)(b))c" flags="d" input="bbbc" "{\u0022matches\u0022:[\u0022c\u0022,\u0022bb\u0022,\u0022b\u0022],\u0022index\u0022:3,\u0022input\u0022:\u0022bbbc\u0022,\u0022indices\u0022:[[3,4],[0,2],[2,3]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=(\u005c\u005c2{2}?)(b))c\u0022}"
        "(?<=(\u005c2{2})(\u005c3{2})(b))c" flags="d" input="bbbbbbbc" "{\u0022matches\u0022:[\u0022c\u0022,\u0022bbbb\u0022,\u0022bb\u0022,\u0022b\u0022],\u0022index\u0022:7,\u0022input\u0022:\u0022bbbbbbbc\u0022,\u0022indices\u0022:[[7,8],[0,4],[4,6],[6,7]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=(\u005c\u005c2{2})(\u005c\u005c3{2})(b))c\u0022}"
        "(a)(?<=a(?=(b)\u005c2{2}\u005c1{2}))b" flags="d" input="abbbaa" "{\u0022matches\u0022:[\u0022ab\u0022,\u0022a\u0022,\u0022b\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022abbbaa\u0022,\u0022indices\u0022:[[0,2],[0,1],[1,2]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a)(?<=a(?=(b)\u005c\u005c2{2}\u005c\u005c1{2}))b\u0022}"
        "(a)(?<=(\u005c3{2})(\u005c1{2}))c" flags="d" input="aaaaaac" "{\u0022matches\u0022:[\u0022ac\u0022,\u0022a\u0022,\u0022aaaa\u0022,\u0022aa\u0022],\u0022index\u0022:5,\u0022input\u0022:\u0022aaaaaac\u0022,\u0022indices\u0022:[[5,7],[5,6],[0,4],[4,6]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a)(?<=(\u005c\u005c3{2})(\u005c\u005c1{2}))c\u0022}"
        "(ab)(?<=b(?=(\u005c1{2})\u005c2{2}))a" flags="d" input="ababababababab" "{\u0022matches\u0022:[\u0022aba\u0022,\u0022ab\u0022,\u0022abab\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022ababababababab\u0022,\u0022indices\u0022:[[0,3],[0,2],[2,6]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(ab)(?<=b(?=(\u005c\u005c1{2})\u005c\u005c2{2}))a\u0022}"
        "(?<=(\u005c1{2}))a" flags="d" input="a" "{\u0022matches\u0022:[\u0022a\u0022,\u0022\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022a\u0022,\u0022indices\u0022:[[0,1],[0,0]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=(\u005c\u005c1{2}))a\u0022}"
        "(?<=(a)\u005c1{2})b" flags="d" input="ab" "{\u0022matches\u0022:[\u0022b\u0022,\u0022a\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022ab\u0022,\u0022indices\u0022:[[1,2],[0,1]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=(a)\u005c\u005c1{2})b\u0022}"
        "(?<=a(?!((b)\u005c2{2})q))b" flags="d" input="abbbx" "{\u0022matches\u0022:[\u0022b\u0022,null,null],\u0022index\u0022:1,\u0022input\u0022:\u0022abbbx\u0022,\u0022indices\u0022:[[1,2],null,null],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=a(?!((b)\u005c\u005c2{2})q))b\u0022}"
        "(?<!(\u005c2{2})(b))c|c" flags="d" input="bbbc" "{\u0022matches\u0022:[\u0022c\u0022,null,null],\u0022index\u0022:3,\u0022input\u0022:\u0022bbbc\u0022,\u0022indices\u0022:[[3,4],null,null],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<!(\u005c\u005c2{2})(b))c|c\u0022}"
        "(a)(?<=\u005c1{2}\u005c1{2})b" flags="d" input="aaaab" "{\u0022matches\u0022:[\u0022ab\u0022,\u0022a\u0022],\u0022index\u0022:3,\u0022input\u0022:\u0022aaaab\u0022,\u0022indices\u0022:[[3,5],[3,4]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a)(?<=\u005c\u005c1{2}\u005c\u005c1{2})b\u0022}"
        "(a)(?<=\u005c1\u005c1{2})b" flags="d" input="aaab" "{\u0022matches\u0022:[\u0022ab\u0022,\u0022a\u0022],\u0022index\u0022:2,\u0022input\u0022:\u0022aaab\u0022,\u0022indices\u0022:[[2,4],[2,3]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a)(?<=\u005c\u005c1\u005c\u005c1{2})b\u0022}"
        "(a)(?<=a(?=\u005c1{2}\u005c1{2}))a" flags="d" input="aaaaa" "{\u0022matches\u0022:[\u0022aa\u0022,\u0022a\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022aaaaa\u0022,\u0022indices\u0022:[[0,2],[0,1]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a)(?<=a(?=\u005c\u005c1{2}\u005c\u005c1{2}))a\u0022}"
        "(?<=a(?=(b)(?:\u005c1\u005c1){2}))b" flags="d" input="abbbbb" "{\u0022matches\u0022:[\u0022b\u0022,\u0022b\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022abbbbb\u0022,\u0022indices\u0022:[[1,2],[1,2]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=a(?=(b)(?:\u005c\u005c1\u005c\u005c1){2}))b\u0022}"
        "(?<=a(?=(?<x>b)\u005ck<x>{2}))b" flags="d" input="abbb" "{\u0022matches\u0022:[\u0022b\u0022,\u0022b\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022abbb\u0022,\u0022groups\u0022:{\u0022x\u0022:\u0022b\u0022},\u0022indices\u0022:[[1,2],[1,2]],\u0022indicesGroups\u0022:{\u0022x\u0022:[1,2]},\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=a(?=(?<x>b)\u005c\u005ck<x>{2}))b\u0022}"
        "(?<=(\u005ck<x>{2})(?<x>b))c" flags="d" input="bbbc" "{\u0022matches\u0022:[\u0022c\u0022,\u0022bb\u0022,\u0022b\u0022],\u0022index\u0022:3,\u0022input\u0022:\u0022bbbc\u0022,\u0022groups\u0022:{\u0022x\u0022:\u0022b\u0022},\u0022indices\u0022:[[3,4],[0,2],[2,3]],\u0022indicesGroups\u0022:{\u0022x\u0022:[2,3]},\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=(\u005c\u005ck<x>{2})(?<x>b))c\u0022}"
        "(?<=(?<x>\u005ck<x>{2}))a" flags="d" input="a" "{\u0022matches\u0022:[\u0022a\u0022,\u0022\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022a\u0022,\u0022groups\u0022:{\u0022x\u0022:\u0022\u0022},\u0022indices\u0022:[[0,1],[0,0]],\u0022indicesGroups\u0022:{\u0022x\u0022:[0,0]},\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=(?<x>\u005c\u005ck<x>{2}))a\u0022}"
        "(a)(?<=a(?=(?<x>b)\u005ck<x>{2}\u005c1{2}))b" flags="d" input="abbbaa" "{\u0022matches\u0022:[\u0022ab\u0022,\u0022a\u0022,\u0022b\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022abbbaa\u0022,\u0022groups\u0022:{\u0022x\u0022:\u0022b\u0022},\u0022indices\u0022:[[0,2],[0,1],[1,2]],\u0022indicesGroups\u0022:{\u0022x\u0022:[1,2]},\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a)(?<=a(?=(?<x>b)\u005c\u005ck<x>{2}\u005c\u005c1{2}))b\u0022}"
        "(?<=a(?=(b)\u005c1{2}))b" flags="di" input="aBbb" "{\u0022matches\u0022:[\u0022B\u0022,\u0022B\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022aBbb\u0022,\u0022indices\u0022:[[1,2],[1,2]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=a(?=(b)\u005c\u005c1{2}))b\u0022}"
        "#);
}
#[test]
fn named_ranges_input_copies_dependent_targets_and_negative_rollback_are_exact() {
    assert_true(
        r#"
            let a = /(?<=a(?=(?<x>b)\k<x>{2}))b/d.exec('abbb');
            a.groups.x === 'b' && a.indices.groups.x === a.indices[1] && a.indices[1][0] === 1
        "#,
    );
    assert_true(
        r#"
            let a = /(?<=(\k<x>{2})(?<x>b))c/d.exec('bbbc');
            a[1] === 'bb' && a.groups.x === 'b' && a.indices[1][0] === 0 && a.indices.groups.x[0] === 2
        "#,
    );
    assert_true(
        r"let a=/(a)(?<=\1{2}\1{2})b/d.exec('aaaab');a.index===3&&a[0]==='ab'&&a.indices[1][0]===3",
    );
    assert_true(
        r"let a=/(?<=a(?!(b)\1{2}))b|(?<=a)b/d.exec('abbb');a.index===1&&a[1]===undefined&&a.indices[1]===undefined",
    );
    assert_true(
        r"let a=/(?<=a(?=([\uD800])\1{2}))[\uD800]/d.exec('a\uD800\uD800\uD800\uDC00');a.index===1&&a.indices[1][0]===1",
    );
}
#[test]
fn consumers_keep_sticky_global_empty_advancement_and_callback_positions() {
    assert_true(
        r#"
            let r = /(?<=a(?=(b)\1{2}))b/dy;
            r.lastIndex = 1;
            let a = r.exec('abbb');
            a[1] === 'b' && r.lastIndex === 2 && r.exec('abbb') === null && r.lastIndex === 0
        "#,
    );
    assert_true(
        r#"
            let r = /(?<=a(?=(b)\1{2}))b/dg,
                a = [...'abbb abbb'.matchAll(r)];
            a.length === 2 && a[1].index === 6 && a[1].indices[1][0] === 6 && r.lastIndex === 0
        "#,
    );
    assert_true(
        r"let a=[...'abbb'.matchAll(/(?<=a(?=(b)\1{2}))/dg)];a.length===1&&a[0].index===1&&a[0][0]===''",
    );
    assert_true(
        r#"
            let seen = [];
            let s = 'abbb abbb'.replace(/(?<=a(?=(b)\1{2}))b/g, (m, x, i) => {
                seen.push(x, i);
                return '_'
            });
            s === 'a_bb a_bb' && seen.join('|') === 'b|1|b|6'
        "#,
    );
    assert_true(
        r"'abbb'.search(/(?<=a(?=(b)\1{2}))b/)===1&&'abbb'.split(/(?<=a(?=(b)\1{2}))b/).join('|')==='a|b|bb'",
    );
}

#[test]
fn opted_in_work_abort_preserves_last_index_and_bypasses_handlers() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval(
            r"let marker=0,r=/(?<=a(?=(b)\1{2}))b/g;r.lastIndex=1;let text='b'.repeat(5000)+'abbb'",
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
fn variable_counts_capture_effects_choices_and_unicode_remain_pending() {
    for source in [
        r"/(?<=a(?=(b)\1{1,2}))b/.exec('abbb')",
        r"/(?<=(\2{1,2})(b))c/.exec('bbbc')",
        r"/(?<=a(?=(b)(\1){2,3}))b/.exec('abbb')",
        r"/(?<=(\2{2}|a)(b))c/.exec('bbbc')",
        r"/(?<=a(?=(b)\1{2}))b/u.exec('abbb')",
        r"/(?<=a(?=(b)\1{2}))b/v.exec('abbb')",
    ] {
        crate::common::regexp_oracle::assert_matches_reference(&mut Realm::default(), source);
    }
}
