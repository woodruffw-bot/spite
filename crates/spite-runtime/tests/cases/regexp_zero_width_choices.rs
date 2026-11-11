//! Pure empty and boundary alternatives in zero-width repetition bodies.

use spite_core::JsString;
use spite_runtime::{Error, Limits, Realm, Value};
use std::fmt::Write;

use crate::common::assert_true;

#[test]
fn pure_zero_width_choice_result_snapshot() {
    let mut rows = String::new();
    for (source, flags, text) in [
        (r"(|){2}a", "d", "qa"),
        (r"(|){0}a", "d", "qa"),
        (r"(?:|){2}a", "d", "qa"),
        (r"(?:|)+", "d", "a"),
        (r"(()|()){2}a", "d", "qa"),
        (r"(()|())*a", "d", "qa"),
        (r"(()|())+?a", "d", "qa"),
        (r"(^|$){2}a", "d", "qa"),
        (r"(^|$)?a", "d", "qa"),
        (r"(^|$){1,3}?a", "dm", "q\na"),
        (r"(\b|\B)+a", "d", "qa"),
        (r"(($)|(^)){2}a", "d", "a"),
        (r"((^)|()){2}a", "d", "qa"),
        (r"(?:(^)|(\b)){2}a", "d", " a"),
        (r"(?:(^)(\B)|(\b)){2}a", "d", "a"),
        (r"((())|(())){2}a", "d", "qa"),
        (r"(?:(^)|($)){2}a\1\2", "d", "a"),
        (r"(?:(^)|()){1,4}?a", "d", "qa"),
        (r"(?:()|()){2}a\1\2", "d", "qa"),
        (r"(?:()|())*a\1\2", "d", "qa"),
        (r"(?:(?:()|()){2}){3}a", "d", "qa"),
        (r"(?:(?:()|())?){2}a", "d", "qa"),
        (r"(?:(?:()|()){2}a|b)+c", "d", "aabbc"),
        (r"(?:(?:()|())?a|b)+c", "d", "aabbc"),
        (r"(?<=(a))(?:()|()){2}b", "d", "ab"),
        (r"(?:(?<=a)|^){2}b", "d", "ab"),
        (r"(?!(?:()|()){2}q)a", "d", "a"),
        (r"(?=(?:()|()){2}a)a", "d", "a"),
        (r"(?:()|()){2}µΜ", "di", "ΜΜ"),
        (r"((?:|){2})a", "d", "qa"),
        (r"(?:(?<x>)|(?<x>)){2}a\k<x>", "d", "qa"),
        (r"(?:(?<x>)|(?<x>))*a\k<x>", "d", "qa"),
        (r"(?:()|()){2}[\uD800]b", "d", "lone surrogates"),
    ] {
        let source = JsString::from(source);
        let flags = JsString::from(flags);
        let input = if text == "lone surrogates" {
            JsString::from_code_units(vec![0xd800, 0x62])
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
        "(|){2}a" flags="d" input="qa" "{\u0022matches\u0022:[\u0022a\u0022,\u0022\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022qa\u0022,\u0022indices\u0022:[[1,2],[1,1]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(|){2}a\u0022}"
        "(|){0}a" flags="d" input="qa" "{\u0022matches\u0022:[\u0022a\u0022,null],\u0022index\u0022:1,\u0022input\u0022:\u0022qa\u0022,\u0022indices\u0022:[[1,2],null],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(|){0}a\u0022}"
        "(?:|){2}a" flags="d" input="qa" "{\u0022matches\u0022:[\u0022a\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022qa\u0022,\u0022indices\u0022:[[1,2]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?:|){2}a\u0022}"
        "(?:|)+" flags="d" input="a" "{\u0022matches\u0022:[\u0022\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022a\u0022,\u0022indices\u0022:[[0,0]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?:|)+\u0022}"
        "(()|()){2}a" flags="d" input="qa" "{\u0022matches\u0022:[\u0022a\u0022,\u0022\u0022,\u0022\u0022,null],\u0022index\u0022:1,\u0022input\u0022:\u0022qa\u0022,\u0022indices\u0022:[[1,2],[1,1],[1,1],null],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(()|()){2}a\u0022}"
        "(()|())*a" flags="d" input="qa" "{\u0022matches\u0022:[\u0022a\u0022,null,null,null],\u0022index\u0022:1,\u0022input\u0022:\u0022qa\u0022,\u0022indices\u0022:[[1,2],null,null,null],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(()|())*a\u0022}"
        "(()|())+?a" flags="d" input="qa" "{\u0022matches\u0022:[\u0022a\u0022,\u0022\u0022,\u0022\u0022,null],\u0022index\u0022:1,\u0022input\u0022:\u0022qa\u0022,\u0022indices\u0022:[[1,2],[1,1],[1,1],null],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(()|())+?a\u0022}"
        "(^|$){2}a" flags="d" input="qa" "{\u0022match\u0022:null,\u0022lastIndex\u0022:0}"
        "(^|$)?a" flags="d" input="qa" "{\u0022matches\u0022:[\u0022a\u0022,null],\u0022index\u0022:1,\u0022input\u0022:\u0022qa\u0022,\u0022indices\u0022:[[1,2],null],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(^|$)?a\u0022}"
        "(^|$){1,3}?a" flags="dm" input="q\u000aa" "{\u0022matches\u0022:[\u0022a\u0022,\u0022\u0022],\u0022index\u0022:2,\u0022input\u0022:\u0022q\u005cna\u0022,\u0022indices\u0022:[[2,3],[2,2]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(^|$){1,3}?a\u0022}"
        "(\u005cb|\u005cB)+a" flags="d" input="qa" "{\u0022matches\u0022:[\u0022a\u0022,\u0022\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022qa\u0022,\u0022indices\u0022:[[1,2],[1,1]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(\u005c\u005cb|\u005c\u005cB)+a\u0022}"
        "(($)|(^)){2}a" flags="d" input="a" "{\u0022matches\u0022:[\u0022a\u0022,\u0022\u0022,null,\u0022\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022a\u0022,\u0022indices\u0022:[[0,1],[0,0],null,[0,0]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(($)|(^)){2}a\u0022}"
        "((^)|()){2}a" flags="d" input="qa" "{\u0022matches\u0022:[\u0022a\u0022,\u0022\u0022,null,\u0022\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022qa\u0022,\u0022indices\u0022:[[1,2],[1,1],null,[1,1]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022((^)|()){2}a\u0022}"
        "(?:(^)|(\u005cb)){2}a" flags="d" input=" a" "{\u0022matches\u0022:[\u0022a\u0022,null,\u0022\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022 a\u0022,\u0022indices\u0022:[[1,2],null,[1,1]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?:(^)|(\u005c\u005cb)){2}a\u0022}"
        "(?:(^)(\u005cB)|(\u005cb)){2}a" flags="d" input="a" "{\u0022matches\u0022:[\u0022a\u0022,null,null,\u0022\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022a\u0022,\u0022indices\u0022:[[0,1],null,null,[0,0]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?:(^)(\u005c\u005cB)|(\u005c\u005cb)){2}a\u0022}"
        "((())|(())){2}a" flags="d" input="qa" "{\u0022matches\u0022:[\u0022a\u0022,\u0022\u0022,\u0022\u0022,\u0022\u0022,null,null],\u0022index\u0022:1,\u0022input\u0022:\u0022qa\u0022,\u0022indices\u0022:[[1,2],[1,1],[1,1],[1,1],null,null],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022((())|(())){2}a\u0022}"
        "(?:(^)|($)){2}a\u005c1\u005c2" flags="d" input="a" "{\u0022matches\u0022:[\u0022a\u0022,\u0022\u0022,null],\u0022index\u0022:0,\u0022input\u0022:\u0022a\u0022,\u0022indices\u0022:[[0,1],[0,0],null],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?:(^)|($)){2}a\u005c\u005c1\u005c\u005c2\u0022}"
        "(?:(^)|()){1,4}?a" flags="d" input="qa" "{\u0022matches\u0022:[\u0022a\u0022,null,\u0022\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022qa\u0022,\u0022indices\u0022:[[1,2],null,[1,1]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?:(^)|()){1,4}?a\u0022}"
        "(?:()|()){2}a\u005c1\u005c2" flags="d" input="qa" "{\u0022matches\u0022:[\u0022a\u0022,\u0022\u0022,null],\u0022index\u0022:1,\u0022input\u0022:\u0022qa\u0022,\u0022indices\u0022:[[1,2],[1,1],null],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?:()|()){2}a\u005c\u005c1\u005c\u005c2\u0022}"
        "(?:()|())*a\u005c1\u005c2" flags="d" input="qa" "{\u0022matches\u0022:[\u0022a\u0022,null,null],\u0022index\u0022:1,\u0022input\u0022:\u0022qa\u0022,\u0022indices\u0022:[[1,2],null,null],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?:()|())*a\u005c\u005c1\u005c\u005c2\u0022}"
        "(?:(?:()|()){2}){3}a" flags="d" input="qa" "{\u0022matches\u0022:[\u0022a\u0022,\u0022\u0022,null],\u0022index\u0022:1,\u0022input\u0022:\u0022qa\u0022,\u0022indices\u0022:[[1,2],[1,1],null],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?:(?:()|()){2}){3}a\u0022}"
        "(?:(?:()|())?){2}a" flags="d" input="qa" "{\u0022matches\u0022:[\u0022a\u0022,null,null],\u0022index\u0022:1,\u0022input\u0022:\u0022qa\u0022,\u0022indices\u0022:[[1,2],null,null],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?:(?:()|())?){2}a\u0022}"
        "(?:(?:()|()){2}a|b)+c" flags="d" input="aabbc" "{\u0022matches\u0022:[\u0022aabbc\u0022,null,null],\u0022index\u0022:0,\u0022input\u0022:\u0022aabbc\u0022,\u0022indices\u0022:[[0,5],null,null],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?:(?:()|()){2}a|b)+c\u0022}"
        "(?:(?:()|())?a|b)+c" flags="d" input="aabbc" "{\u0022matches\u0022:[\u0022aabbc\u0022,null,null],\u0022index\u0022:0,\u0022input\u0022:\u0022aabbc\u0022,\u0022indices\u0022:[[0,5],null,null],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?:(?:()|())?a|b)+c\u0022}"
        "(?<=(a))(?:()|()){2}b" flags="d" input="ab" "{\u0022matches\u0022:[\u0022b\u0022,\u0022a\u0022,\u0022\u0022,null],\u0022index\u0022:1,\u0022input\u0022:\u0022ab\u0022,\u0022indices\u0022:[[1,2],[0,1],[1,1],null],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=(a))(?:()|()){2}b\u0022}"
        "(?:(?<=a)|^){2}b" flags="d" input="ab" "{\u0022matches\u0022:[\u0022b\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022ab\u0022,\u0022indices\u0022:[[1,2]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?:(?<=a)|^){2}b\u0022}"
        "(?!(?:()|()){2}q)a" flags="d" input="a" "{\u0022matches\u0022:[\u0022a\u0022,null,null],\u0022index\u0022:0,\u0022input\u0022:\u0022a\u0022,\u0022indices\u0022:[[0,1],null,null],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?!(?:()|()){2}q)a\u0022}"
        "(?=(?:()|()){2}a)a" flags="d" input="a" "{\u0022matches\u0022:[\u0022a\u0022,\u0022\u0022,null],\u0022index\u0022:0,\u0022input\u0022:\u0022a\u0022,\u0022indices\u0022:[[0,1],[0,0],null],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?=(?:()|()){2}a)a\u0022}"
        "(?:()|()){2}\u00b5\u039c" flags="di" input="\u039c\u039c" "{\u0022matches\u0022:[\u0022\u039c\u039c\u0022,\u0022\u0022,null],\u0022index\u0022:0,\u0022input\u0022:\u0022\u039c\u039c\u0022,\u0022indices\u0022:[[0,2],[0,0],null],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?:()|()){2}\u00b5\u039c\u0022}"
        "((?:|){2})a" flags="d" input="qa" "{\u0022matches\u0022:[\u0022a\u0022,\u0022\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022qa\u0022,\u0022indices\u0022:[[1,2],[1,1]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022((?:|){2})a\u0022}"
        "(?:(?<x>)|(?<x>)){2}a\u005ck<x>" flags="d" input="qa" "{\u0022matches\u0022:[\u0022a\u0022,\u0022\u0022,null],\u0022index\u0022:1,\u0022input\u0022:\u0022qa\u0022,\u0022groups\u0022:{\u0022x\u0022:\u0022\u0022},\u0022indices\u0022:[[1,2],[1,1],null],\u0022indicesGroups\u0022:{\u0022x\u0022:[1,1]},\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?:(?<x>)|(?<x>)){2}a\u005c\u005ck<x>\u0022}"
        "(?:(?<x>)|(?<x>))*a\u005ck<x>" flags="d" input="qa" "{\u0022matches\u0022:[\u0022a\u0022,null,null],\u0022index\u0022:1,\u0022input\u0022:\u0022qa\u0022,\u0022groups\u0022:{},\u0022indices\u0022:[[1,2],null,null],\u0022indicesGroups\u0022:{},\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?:(?<x>)|(?<x>))*a\u005c\u005ck<x>\u0022}"
        "(?:()|()){2}[\u005cuD800]b" flags="d" input="\ud800b" "{\u0022matches\u0022:[\u0022\u005cud800b\u0022,\u0022\u0022,null],\u0022index\u0022:0,\u0022input\u0022:\u0022\u005cud800b\u0022,\u0022indices\u0022:[[0,2],[0,0],null],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?:()|()){2}[\u005c\u005cuD800]b\u0022}"
        "#);
}

#[test]
fn pure_zero_choices_keep_source_order_names_and_restore_partial_failed_arms() {
    assert_true(
        r"/^(^|$)+$/.test('')&&/(^|$)+/.test('a')&&/(^|$)*/.test('')&&/($|^)+/.test('')&&/(\b|\B){2}/.test('')",
    );
    assert_true(
        r"let a=/(?:(?<x>)|(?<x>)){2}a\k<x>/d.exec('qa');a.index===1&&a[1]===''&&a[2]===undefined&&a.groups.x===''&&a.indices.groups.x===a.indices[1]&&a.indices[1][0]===1",
    );
    assert_true(
        r"let a=/(?:(?<x>)|(?<x>))*a\k<x>/d.exec('qa');a[1]===undefined&&a[2]===undefined&&a.groups.x===undefined&&a.indices.groups.x===undefined",
    );
    assert_true(
        r"let a=/(?:(^)(\B)|(\b)){2}a/d.exec('a');a[1]===undefined&&a[2]===undefined&&a[3]===''&&a.indices[1]===undefined&&a.indices[3][0]===0",
    );
    assert_true(
        r"let a=/((^)|()){2}a/d.exec('qa'),b=/((^)|())?a/d.exec('qa');a[1]===''&&a[2]===undefined&&a[3]===''&&b[1]===undefined&&b[3]===undefined",
    );
    assert_true(
        r"let a=/(?!(?:()|()){2}q)a/d.exec('a');a[1]===undefined&&a[2]===undefined&&a.indices[1]===undefined",
    );
    assert_true(
        r"let a=/(?:(?:()|()){2}){3}a/d.exec('qa'),b=/(?:(?:()|())?){2}a/d.exec('qa');a[1]===''&&a[2]===undefined&&b[1]===undefined&&b[2]===undefined",
    );
    assert_true(
        r"let a=/(?<=(a))(?:()|()){2}b/d.exec('ab');a.index===1&&a[1]==='a'&&a.indices[1][0]===0&&a[2]===''&&a[3]===undefined",
    );
}

#[test]
fn pure_zero_choices_consumers_empty_advancement_sticky_and_callbacks_preserve_slots() {
    assert_true(
        r#"
            let r = /(?:(?<x>)|(?<x>)){2}a/dg,
                a = [...'qa a'.matchAll(r)];
            a.length === 2 && a[1].index === 3 && a[1].groups.x === '' && a[1].indices.groups.x[0] === 3 &&
                r.lastIndex === 0
        "#,
    );
    assert_true(
        r#"
            let r = /(?:()|()){2}a/dy;
            r.lastIndex = 1;
            let a = r.exec('qa');
            a[1] === '' && a[2] === undefined && r.lastIndex === 2 && r.exec('qa') === null && r
                .lastIndex === 0
        "#,
    );
    assert_true(
        r#"
            let r = /(?:()|())+/dg,
                a = [...'ab'.matchAll(r)];
            a.length === 3 && a[2].index === 2 && a[2][1] === '' && a[2][2] === undefined && a[2].indices[
                1][0] === 2 && r.lastIndex === 0
        "#,
    );
    assert_true(
        r#"
            let seen = [];
            let s = 'qa a'.replace(/(?:(?<x>)|(?<x>)){2}a/g, (m, c, d, i, s, g) => {
                seen.push(c, d, i, g.x);
                return '_'
            });
            s === 'q_ _' && seen.length === 8 && seen[0] === '' && seen[1] === undefined && seen[2] ===
                1 && seen[3] === '' && seen[6] === 3
        "#,
    );
    assert_true(
        r"'qa a'.replace(/(?:(?<x>)|(?<x>)){2}a/g,'<$<x>>')==='q<> <>'&&'qa'.search(/(?:()|()){2}a/)===1&&'qa a'.split(/(?:()|()){2}a/).join('|')==='q||| |||'",
    );
}

#[test]
fn explicit_pure_zero_choice_work_aborts_before_last_index_and_handlers() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval("let marker=0,r=/(?:()|()){2}a/g;r.lastIndex=1;let text='b'.repeat(5000)")
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
fn mixed_empty_consuming_choices_internal_references_counted_behind_and_unicode_are_pending() {
    for source in [
        r"/(?:a|()){2,3}b/.exec('ab')",
        r"/(?:(\1)|()){2}a()/.exec('a')",
        r"/(?<=((a)|()){2})a/.exec('a')",
        r"/(?:(?:a|()){2,3})*b/.exec('ab')",
        r"/(?:()|()){2}a/u.exec('a')",
        r"/(?:()|()){2}a/v.exec('a')",
    ] {
        crate::common::regexp_oracle::assert_matches_reference(&mut Realm::default(), source);
    }
}
