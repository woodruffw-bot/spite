//! Required and optional counts of empty and boundary bodies in ordinary lookbehind.

use spite_core::JsString;
use spite_runtime::{Error, Limits, Realm, Value};
use std::fmt::Write;

use crate::common::assert_true;

#[test]
fn zero_width_lookbehind_count_result_snapshot() {
    let mut rows = String::new();
    for (source, flags, text) in [
        (r"(?<=()?)a", "d", "qa"),
        (r"(?<=()*?)a", "d", "qa"),
        (r"(?<=()+)a", "d", "qa"),
        (r"(?<=()+?)a", "d", "qa"),
        (r"(?<=(){0,2})a", "d", "qa"),
        (r"(?<=(){1,2}?)a", "d", "qa"),
        (r"(?<=(){2,})a", "d", "qa"),
        (r"(?<=(?:)*)a", "d", "qa"),
        (r"(?<=(()){1,3})a", "d", "qa"),
        (r"(?<=(()()){1,})a", "d", "qa"),
        (r"(?<=(^)?)a", "d", "qa"),
        (r"(?<=(^)+)a", "d", "qa"),
        (r"(?<=(^)+)a", "dm", "q\na"),
        (r"(?<=(\b){0,3})a", "d", "qa"),
        (r"(?<=(\b){1,3}?)a", "d", " a"),
        (r"(?<=(\B)+)a", "d", "qa"),
        (r"(?<=(^\b)+)a", "d", "a"),
        (r"(?<=(^$){0,2})a", "d", "qa"),
        (r"(?<!()+)a", "d", "a"),
        (r"(?<!(^)+)a", "d", "qa"),
        (r"(?<!()+q)b", "d", "ab"),
        (r"(?<=()+a)b", "d", "ab"),
        (r"(?<=a()*)b", "d", "ab"),
        (r"(?<=()+|())a", "d", "a"),
        (r"(?<=()*|())a", "d", "a"),
        (r"(?<=()+)a\1", "d", "a"),
        (r"(?<=(?:()+)(?<=a))b", "d", "ab"),
        (r"((?<=()+)){2}a\1\2", "d", "a"),
        (r"((?<=()+))*a\1\2", "d", "a"),
        (r"(?:(?<=()+)a|b)+c", "d", "aabbc"),
        (r"(?<=((?:)*µ))Μ", "di", "ΜΜ"),
        (r"(?<=(?<x>)+)a", "d", "qa"),
        (r"(?<=(?<x>)*)a", "d", "qa"),
        (r"(?<=()+)", "d", "a"),
        (r"(?<=()+[\uD800])b", "d", "lone surrogates"),
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
        "(?<=()?)a" flags="d" input="qa" "{\u0022matches\u0022:[\u0022a\u0022,null],\u0022index\u0022:1,\u0022input\u0022:\u0022qa\u0022,\u0022indices\u0022:[[1,2],null],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=()?)a\u0022}"
        "(?<=()*?)a" flags="d" input="qa" "{\u0022matches\u0022:[\u0022a\u0022,null],\u0022index\u0022:1,\u0022input\u0022:\u0022qa\u0022,\u0022indices\u0022:[[1,2],null],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=()*?)a\u0022}"
        "(?<=()+)a" flags="d" input="qa" "{\u0022matches\u0022:[\u0022a\u0022,\u0022\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022qa\u0022,\u0022indices\u0022:[[1,2],[1,1]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=()+)a\u0022}"
        "(?<=()+?)a" flags="d" input="qa" "{\u0022matches\u0022:[\u0022a\u0022,\u0022\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022qa\u0022,\u0022indices\u0022:[[1,2],[1,1]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=()+?)a\u0022}"
        "(?<=(){0,2})a" flags="d" input="qa" "{\u0022matches\u0022:[\u0022a\u0022,null],\u0022index\u0022:1,\u0022input\u0022:\u0022qa\u0022,\u0022indices\u0022:[[1,2],null],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=(){0,2})a\u0022}"
        "(?<=(){1,2}?)a" flags="d" input="qa" "{\u0022matches\u0022:[\u0022a\u0022,\u0022\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022qa\u0022,\u0022indices\u0022:[[1,2],[1,1]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=(){1,2}?)a\u0022}"
        "(?<=(){2,})a" flags="d" input="qa" "{\u0022matches\u0022:[\u0022a\u0022,\u0022\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022qa\u0022,\u0022indices\u0022:[[1,2],[1,1]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=(){2,})a\u0022}"
        "(?<=(?:)*)a" flags="d" input="qa" "{\u0022matches\u0022:[\u0022a\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022qa\u0022,\u0022indices\u0022:[[1,2]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=(?:)*)a\u0022}"
        "(?<=(()){1,3})a" flags="d" input="qa" "{\u0022matches\u0022:[\u0022a\u0022,\u0022\u0022,\u0022\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022qa\u0022,\u0022indices\u0022:[[1,2],[1,1],[1,1]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=(()){1,3})a\u0022}"
        "(?<=(()()){1,})a" flags="d" input="qa" "{\u0022matches\u0022:[\u0022a\u0022,\u0022\u0022,\u0022\u0022,\u0022\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022qa\u0022,\u0022indices\u0022:[[1,2],[1,1],[1,1],[1,1]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=(()()){1,})a\u0022}"
        "(?<=(^)?)a" flags="d" input="qa" "{\u0022matches\u0022:[\u0022a\u0022,null],\u0022index\u0022:1,\u0022input\u0022:\u0022qa\u0022,\u0022indices\u0022:[[1,2],null],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=(^)?)a\u0022}"
        "(?<=(^)+)a" flags="d" input="qa" "{\u0022match\u0022:null,\u0022lastIndex\u0022:0}"
        "(?<=(^)+)a" flags="dm" input="q\u000aa" "{\u0022matches\u0022:[\u0022a\u0022,\u0022\u0022],\u0022index\u0022:2,\u0022input\u0022:\u0022q\u005cna\u0022,\u0022indices\u0022:[[2,3],[2,2]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=(^)+)a\u0022}"
        "(?<=(\u005cb){0,3})a" flags="d" input="qa" "{\u0022matches\u0022:[\u0022a\u0022,null],\u0022index\u0022:1,\u0022input\u0022:\u0022qa\u0022,\u0022indices\u0022:[[1,2],null],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=(\u005c\u005cb){0,3})a\u0022}"
        "(?<=(\u005cb){1,3}?)a" flags="d" input=" a" "{\u0022matches\u0022:[\u0022a\u0022,\u0022\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022 a\u0022,\u0022indices\u0022:[[1,2],[1,1]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=(\u005c\u005cb){1,3}?)a\u0022}"
        "(?<=(\u005cB)+)a" flags="d" input="qa" "{\u0022matches\u0022:[\u0022a\u0022,\u0022\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022qa\u0022,\u0022indices\u0022:[[1,2],[1,1]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=(\u005c\u005cB)+)a\u0022}"
        "(?<=(^\u005cb)+)a" flags="d" input="a" "{\u0022matches\u0022:[\u0022a\u0022,\u0022\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022a\u0022,\u0022indices\u0022:[[0,1],[0,0]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=(^\u005c\u005cb)+)a\u0022}"
        "(?<=(^$){0,2})a" flags="d" input="qa" "{\u0022matches\u0022:[\u0022a\u0022,null],\u0022index\u0022:1,\u0022input\u0022:\u0022qa\u0022,\u0022indices\u0022:[[1,2],null],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=(^$){0,2})a\u0022}"
        "(?<!()+)a" flags="d" input="a" "{\u0022match\u0022:null,\u0022lastIndex\u0022:0}"
        "(?<!(^)+)a" flags="d" input="qa" "{\u0022matches\u0022:[\u0022a\u0022,null],\u0022index\u0022:1,\u0022input\u0022:\u0022qa\u0022,\u0022indices\u0022:[[1,2],null],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<!(^)+)a\u0022}"
        "(?<!()+q)b" flags="d" input="ab" "{\u0022matches\u0022:[\u0022b\u0022,null],\u0022index\u0022:1,\u0022input\u0022:\u0022ab\u0022,\u0022indices\u0022:[[1,2],null],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<!()+q)b\u0022}"
        "(?<=()+a)b" flags="d" input="ab" "{\u0022matches\u0022:[\u0022b\u0022,\u0022\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022ab\u0022,\u0022indices\u0022:[[1,2],[0,0]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=()+a)b\u0022}"
        "(?<=a()*)b" flags="d" input="ab" "{\u0022matches\u0022:[\u0022b\u0022,null],\u0022index\u0022:1,\u0022input\u0022:\u0022ab\u0022,\u0022indices\u0022:[[1,2],null],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=a()*)b\u0022}"
        "(?<=()+|())a" flags="d" input="a" "{\u0022matches\u0022:[\u0022a\u0022,\u0022\u0022,null],\u0022index\u0022:0,\u0022input\u0022:\u0022a\u0022,\u0022indices\u0022:[[0,1],[0,0],null],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=()+|())a\u0022}"
        "(?<=()*|())a" flags="d" input="a" "{\u0022matches\u0022:[\u0022a\u0022,null,null],\u0022index\u0022:0,\u0022input\u0022:\u0022a\u0022,\u0022indices\u0022:[[0,1],null,null],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=()*|())a\u0022}"
        "(?<=()+)a\u005c1" flags="d" input="a" "{\u0022matches\u0022:[\u0022a\u0022,\u0022\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022a\u0022,\u0022indices\u0022:[[0,1],[0,0]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=()+)a\u005c\u005c1\u0022}"
        "(?<=(?:()+)(?<=a))b" flags="d" input="ab" "{\u0022matches\u0022:[\u0022b\u0022,\u0022\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022ab\u0022,\u0022indices\u0022:[[1,2],[1,1]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=(?:()+)(?<=a))b\u0022}"
        "((?<=()+)){2}a\u005c1\u005c2" flags="d" input="a" "{\u0022matches\u0022:[\u0022a\u0022,\u0022\u0022,\u0022\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022a\u0022,\u0022indices\u0022:[[0,1],[0,0],[0,0]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022((?<=()+)){2}a\u005c\u005c1\u005c\u005c2\u0022}"
        "((?<=()+))*a\u005c1\u005c2" flags="d" input="a" "{\u0022matches\u0022:[\u0022a\u0022,null,null],\u0022index\u0022:0,\u0022input\u0022:\u0022a\u0022,\u0022indices\u0022:[[0,1],null,null],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022((?<=()+))*a\u005c\u005c1\u005c\u005c2\u0022}"
        "(?:(?<=()+)a|b)+c" flags="d" input="aabbc" "{\u0022matches\u0022:[\u0022aabbc\u0022,null],\u0022index\u0022:0,\u0022input\u0022:\u0022aabbc\u0022,\u0022indices\u0022:[[0,5],null],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?:(?<=()+)a|b)+c\u0022}"
        "(?<=((?:)*\u00b5))\u039c" flags="di" input="\u039c\u039c" "{\u0022matches\u0022:[\u0022\u039c\u0022,\u0022\u039c\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022\u039c\u039c\u0022,\u0022indices\u0022:[[1,2],[0,1]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=((?:)*\u00b5))\u039c\u0022}"
        "(?<=(?<x>)+)a" flags="d" input="qa" "{\u0022matches\u0022:[\u0022a\u0022,\u0022\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022qa\u0022,\u0022groups\u0022:{\u0022x\u0022:\u0022\u0022},\u0022indices\u0022:[[1,2],[1,1]],\u0022indicesGroups\u0022:{\u0022x\u0022:[1,1]},\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=(?<x>)+)a\u0022}"
        "(?<=(?<x>)*)a" flags="d" input="qa" "{\u0022matches\u0022:[\u0022a\u0022,null],\u0022index\u0022:1,\u0022input\u0022:\u0022qa\u0022,\u0022groups\u0022:{},\u0022indices\u0022:[[1,2],null],\u0022indicesGroups\u0022:{},\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=(?<x>)*)a\u0022}"
        "(?<=()+)" flags="d" input="a" "{\u0022matches\u0022:[\u0022\u0022,\u0022\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022a\u0022,\u0022indices\u0022:[[0,0],[0,0]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=()+)\u0022}"
        "(?<=()+[\u005cuD800])b" flags="d" input="\ud800b" "{\u0022matches\u0022:[\u0022b\u0022,\u0022\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022\u005cud800b\u0022,\u0022indices\u0022:[[1,2],[0,0]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=()+[\u005c\u005cuD800])b\u0022}"
        "#);
}

#[test]
fn zero_width_counts_restore_names_and_distinguish_required_and_optional_captures() {
    assert_true(
        r#"
            let a = /(?<=(?<x>)+a)b/d.exec('qab'),
                b = /(?<=a(?<x>)*)b/d.exec('qab');
            a.groups.x === '' && a.indices.groups.x === a.indices[1] && a.indices[1][0] === 1 && b.groups
                .x === undefined && b.indices.groups.x === undefined
        "#,
    );
    assert_true(
        r#"
            let a = /(?<=(^)?)a/d.exec('qa'),
                b = /(?<=(^)+)a/dm.exec('q\na');
            a[1] === undefined && a.indices[1] === undefined && b.index === 2 && b[1] === '' && b.indices[
                1][0] === 2
        "#,
    );
    assert_true(
        r#"
            let a = /(?<=(?<x>)+|(?<x>))a\k<x>/d.exec('a'),
                b = /(?<=(?<x>)*|(?<x>))a\k<x>/d.exec('a');
            a[1] === '' && a[2] === undefined && a.groups.x === '' && a.indices.groups.x === a.indices[
                1] && b[1] === undefined && b[2] === undefined && b.groups.x === undefined
        "#,
    );
    assert_true(
        r#"
            let a = /(?<!((?<x>))+q)b/d.exec('ab');
            a[1] === undefined && a[2] === undefined && a.groups.x === undefined && a.indices.groups.x ===
                undefined
        "#,
    );
    assert_true(
        r#"
            let a = /((?<=()+)){2}a\1\2/d.exec('a'),
                b = /((?<=()+))*a\1\2/d.exec('a');
            a[1] === '' && a[2] === '' && b[1] === undefined && b[2] === undefined
        "#,
    );
    assert_true(
        r"let a=/(?<=(?:()+)(?<=a))b/d.exec('ab');a.index===1&&a[1]===''&&a.indices[1][0]===1",
    );
    assert_true(
        r#"
            let a = /(?<=(^\b)+)a/d.exec('a'),
                b = /(?<=(\b){1,3}?)a/d.exec(' a');
            a[1] === '' && b.index === 1 && b.indices[1][0] === 1 && /(?<=(^$){0,2})a/.test('qa') &&
                /(?<!(^)+)a/.test('qa')
        "#,
    );
}

#[test]
fn zero_width_count_consumers_keep_last_index_empty_advancement_and_callback_slots() {
    assert_true(
        r#"
            let r = /(?<=(?<x>)+)a/dg,
                a = [...'qa a'.matchAll(r)];
            a.length === 2 && a[1].index === 3 && a[1].groups.x === '' && a[1].indices.groups.x[0] === 3 &&
                r.lastIndex === 0
        "#,
    );
    assert_true(
        r#"
            let r = /(?<=()+)a/dy;
            r.lastIndex = 1;
            let a = r.exec('qa');
            a[1] === '' && a.index === 1 && r.lastIndex === 2 && r.exec('qa') === null && r.lastIndex === 0
        "#,
    );
    assert_true(
        r#"
            let r = /(?<=()*)/dg,
                a = [...'ab'.matchAll(r)];
            a.length === 3 && a[2].index === 2 && a[2][1] === undefined && a[2].indices[1] === undefined &&
                r.lastIndex === 0
        "#,
    );
    assert_true(
        r#"
            let seen = [];
            let s = 'qa a'.replace(/(?<=(?<x>)+)a/g, (m, c, i, s, g) => {
                seen.push(m, c, i, s, g.x);
                return '_'
            });
            s === 'q_ _' && seen.length === 10 && seen[1] === '' && seen[2] === 1 && seen[4] === '' &&
                seen[7] === 3
        "#,
    );
    assert_true(
        r"'qa a'.replace(/(?<=(?<x>)+)a/g,'<$<x>>')==='q<> <>'&&'qa'.search(/(?<=()+)a/)===1&&'qa a'.split(/(?<=()+)a/).join('|')==='q|| ||'",
    );
}

#[test]
fn explicit_zero_width_count_work_aborts_before_last_index_and_handlers() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval("let marker=0,r=/(?<=()+)a/g;r.lastIndex=1;let text='b'.repeat(5000)")
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
fn consuming_variable_counts_repeated_choices_references_nested_counts_and_lookaround_are_pending()
{
    for source in [
        r"/(?<=a{1,2})b/.exec('ab')",
        r"/(?<=(a?){1,2})b/.exec('ab')",
        r"/(?<=((?=a+)){1,2})a/.exec('a')",
        r"/(?<=(|a)+)a/.exec('a')",
        r"/(?<=(\2)+)a()/.exec('a')",
        r"/(?<=((a*)+))a/.exec('a')",
        r"/(?<=()+)a/u.exec('a')",
        r"/(?<=()+)a/v.exec('a')",
    ] {
        crate::common::regexp_oracle::assert_matches_reference(&mut Realm::default(), source);
    }
}
