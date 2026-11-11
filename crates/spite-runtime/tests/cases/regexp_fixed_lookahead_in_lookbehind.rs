//! Capture-free fixed lookahead inside fixed ordinary lookbehind.
use crate::common::assert_true;
use spite_core::JsString;
use spite_runtime::{Error, Limits, Realm, Value};
use std::fmt::Write;

#[test]
fn fixed_lookahead_in_lookbehind_result_snapshot() {
    let mut rows = String::new();
    for (source, flags, text) in [
        (r"(?<=a(?=b))b", "d", "ab"),
        (r"(?<=a(?!c))b", "d", "ab"),
        (r"(?<=(?=ab)a)b", "d", "ab"),
        (r"(?<=(?!bb)a)b", "d", "ab"),
        (r"(?<=a(?=b{2}))b", "d", "abb"),
        (r"(?<=a(?=[bc]|d))b", "d", "ab"),
        (r"(?<=a(?=b|c))b", "d", "ab"),
        (r"(?<=a(?!b|c))d", "d", "ad"),
        (r"(?<=a(?=(?=b)b))b", "d", "ab"),
        (r"(?<=a(?=(?!c)b))b", "d", "ab"),
        (r"(?<=a(?=(?<=a)b))b", "d", "ab"),
        (r"(?<=a(?=(?<!c)b))b", "d", "ab"),
        (r"(?<=a(?=(?:b){2}))bb", "d", "abb"),
        (r"(?<=a(?=b{0}))b", "d", "ab"),
        (r"(?<=a(?=(?:b|c){0}))b", "d", "ab"),
        (r"(?<=a(?=$))", "d", "a"),
        (r"(?<=a(?=^))b", "d", "ab"),
        (r"(?<=a(?=\b)) ", "d", "a "),
        (r"(?<=a(?=\B))b", "d", "ab"),
        (r"(?<=a(?=.))b", "d", "ab"),
        (r"(?<=a(?=b.))b", "d", "abb"),
        (r"(?<=a(?!b.))b", "d", "ab"),
        (r"(?<=(?=ab)(a))b", "d", "ab"),
        (r"(?<=((?=b)){2})b", "d", "b"),
        (r"(?<=((?!a)){2})b", "d", "b"),
        (r"(?<=((?=b))*)b", "d", "b"),
        (r"(?<=a(?=(?:(?=b)){2}b))b", "d", "ab"),
        (r"(?<=a(?=(?<=a(?=b))b))b", "d", "ab"),
        (r"(?<=µ(?=Μ))Μ", "di", "ΜΜ"),
        (r"(?<!a(?=b))b", "d", "qb"),
        (r"(?<!a(?!b))b", "d", "ab"),
        (r"(?<=a(?=b)|a(?!c))b", "d", "ab"),
        (r"(?:(?<=a(?=b))b|a)+c", "d", "ababc"),
        (r"(?=(?<=a(?=b))b)b", "d", "qab"),
        (r"(?<=[\uD800](?=[\uDC00]))[\uDC00]", "d", "surrogates"),
        (r"(?<=(?=ab)(?<x>a))b", "d", "qab"),
        (r"(?<=(?=ab)(?:(?<x>a)|(?<x>c)))b", "d", "qab"),
        (r"(?<=((?=b)){2})b", "dg", "qb"),
        (r"(?<=a(?=b$))b", "dm", "ab\nq"),
        (r"(?<=a(?=b.))b", "ds", "ab\n"),
    ] {
        let source = JsString::from(source);
        let flags = JsString::from(flags);
        let input = if text == "surrogates" {
            JsString::from_code_units(vec![0xd800, 0xdc00])
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
        "(?<=a(?=b))b" flags="d" input="ab" "{\u0022matches\u0022:[\u0022b\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022ab\u0022,\u0022indices\u0022:[[1,2]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=a(?=b))b\u0022}"
        "(?<=a(?!c))b" flags="d" input="ab" "{\u0022matches\u0022:[\u0022b\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022ab\u0022,\u0022indices\u0022:[[1,2]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=a(?!c))b\u0022}"
        "(?<=(?=ab)a)b" flags="d" input="ab" "{\u0022matches\u0022:[\u0022b\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022ab\u0022,\u0022indices\u0022:[[1,2]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=(?=ab)a)b\u0022}"
        "(?<=(?!bb)a)b" flags="d" input="ab" "{\u0022matches\u0022:[\u0022b\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022ab\u0022,\u0022indices\u0022:[[1,2]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=(?!bb)a)b\u0022}"
        "(?<=a(?=b{2}))b" flags="d" input="abb" "{\u0022matches\u0022:[\u0022b\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022abb\u0022,\u0022indices\u0022:[[1,2]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=a(?=b{2}))b\u0022}"
        "(?<=a(?=[bc]|d))b" flags="d" input="ab" "{\u0022matches\u0022:[\u0022b\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022ab\u0022,\u0022indices\u0022:[[1,2]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=a(?=[bc]|d))b\u0022}"
        "(?<=a(?=b|c))b" flags="d" input="ab" "{\u0022matches\u0022:[\u0022b\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022ab\u0022,\u0022indices\u0022:[[1,2]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=a(?=b|c))b\u0022}"
        "(?<=a(?!b|c))d" flags="d" input="ad" "{\u0022matches\u0022:[\u0022d\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022ad\u0022,\u0022indices\u0022:[[1,2]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=a(?!b|c))d\u0022}"
        "(?<=a(?=(?=b)b))b" flags="d" input="ab" "{\u0022matches\u0022:[\u0022b\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022ab\u0022,\u0022indices\u0022:[[1,2]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=a(?=(?=b)b))b\u0022}"
        "(?<=a(?=(?!c)b))b" flags="d" input="ab" "{\u0022matches\u0022:[\u0022b\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022ab\u0022,\u0022indices\u0022:[[1,2]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=a(?=(?!c)b))b\u0022}"
        "(?<=a(?=(?<=a)b))b" flags="d" input="ab" "{\u0022matches\u0022:[\u0022b\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022ab\u0022,\u0022indices\u0022:[[1,2]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=a(?=(?<=a)b))b\u0022}"
        "(?<=a(?=(?<!c)b))b" flags="d" input="ab" "{\u0022matches\u0022:[\u0022b\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022ab\u0022,\u0022indices\u0022:[[1,2]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=a(?=(?<!c)b))b\u0022}"
        "(?<=a(?=(?:b){2}))bb" flags="d" input="abb" "{\u0022matches\u0022:[\u0022bb\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022abb\u0022,\u0022indices\u0022:[[1,3]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=a(?=(?:b){2}))bb\u0022}"
        "(?<=a(?=b{0}))b" flags="d" input="ab" "{\u0022matches\u0022:[\u0022b\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022ab\u0022,\u0022indices\u0022:[[1,2]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=a(?=b{0}))b\u0022}"
        "(?<=a(?=(?:b|c){0}))b" flags="d" input="ab" "{\u0022matches\u0022:[\u0022b\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022ab\u0022,\u0022indices\u0022:[[1,2]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=a(?=(?:b|c){0}))b\u0022}"
        "(?<=a(?=$))" flags="d" input="a" "{\u0022matches\u0022:[\u0022\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022a\u0022,\u0022indices\u0022:[[1,1]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=a(?=$))\u0022}"
        "(?<=a(?=^))b" flags="d" input="ab" "{\u0022match\u0022:null,\u0022lastIndex\u0022:0}"
        "(?<=a(?=\u005cb)) " flags="d" input="a " "{\u0022matches\u0022:[\u0022 \u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022a \u0022,\u0022indices\u0022:[[1,2]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=a(?=\u005c\u005cb)) \u0022}"
        "(?<=a(?=\u005cB))b" flags="d" input="ab" "{\u0022matches\u0022:[\u0022b\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022ab\u0022,\u0022indices\u0022:[[1,2]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=a(?=\u005c\u005cB))b\u0022}"
        "(?<=a(?=.))b" flags="d" input="ab" "{\u0022matches\u0022:[\u0022b\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022ab\u0022,\u0022indices\u0022:[[1,2]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=a(?=.))b\u0022}"
        "(?<=a(?=b.))b" flags="d" input="abb" "{\u0022matches\u0022:[\u0022b\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022abb\u0022,\u0022indices\u0022:[[1,2]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=a(?=b.))b\u0022}"
        "(?<=a(?!b.))b" flags="d" input="ab" "{\u0022matches\u0022:[\u0022b\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022ab\u0022,\u0022indices\u0022:[[1,2]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=a(?!b.))b\u0022}"
        "(?<=(?=ab)(a))b" flags="d" input="ab" "{\u0022matches\u0022:[\u0022b\u0022,\u0022a\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022ab\u0022,\u0022indices\u0022:[[1,2],[0,1]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=(?=ab)(a))b\u0022}"
        "(?<=((?=b)){2})b" flags="d" input="b" "{\u0022matches\u0022:[\u0022b\u0022,\u0022\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022b\u0022,\u0022indices\u0022:[[0,1],[0,0]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=((?=b)){2})b\u0022}"
        "(?<=((?!a)){2})b" flags="d" input="b" "{\u0022matches\u0022:[\u0022b\u0022,\u0022\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022b\u0022,\u0022indices\u0022:[[0,1],[0,0]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=((?!a)){2})b\u0022}"
        "(?<=((?=b))*)b" flags="d" input="b" "{\u0022matches\u0022:[\u0022b\u0022,null],\u0022index\u0022:0,\u0022input\u0022:\u0022b\u0022,\u0022indices\u0022:[[0,1],null],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=((?=b))*)b\u0022}"
        "(?<=a(?=(?:(?=b)){2}b))b" flags="d" input="ab" "{\u0022matches\u0022:[\u0022b\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022ab\u0022,\u0022indices\u0022:[[1,2]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=a(?=(?:(?=b)){2}b))b\u0022}"
        "(?<=a(?=(?<=a(?=b))b))b" flags="d" input="ab" "{\u0022matches\u0022:[\u0022b\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022ab\u0022,\u0022indices\u0022:[[1,2]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=a(?=(?<=a(?=b))b))b\u0022}"
        "(?<=\u00b5(?=\u039c))\u039c" flags="di" input="\u039c\u039c" "{\u0022matches\u0022:[\u0022\u039c\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022\u039c\u039c\u0022,\u0022indices\u0022:[[1,2]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=\u00b5(?=\u039c))\u039c\u0022}"
        "(?<!a(?=b))b" flags="d" input="qb" "{\u0022matches\u0022:[\u0022b\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022qb\u0022,\u0022indices\u0022:[[1,2]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<!a(?=b))b\u0022}"
        "(?<!a(?!b))b" flags="d" input="ab" "{\u0022matches\u0022:[\u0022b\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022ab\u0022,\u0022indices\u0022:[[1,2]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<!a(?!b))b\u0022}"
        "(?<=a(?=b)|a(?!c))b" flags="d" input="ab" "{\u0022matches\u0022:[\u0022b\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022ab\u0022,\u0022indices\u0022:[[1,2]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=a(?=b)|a(?!c))b\u0022}"
        "(?:(?<=a(?=b))b|a)+c" flags="d" input="ababc" "{\u0022matches\u0022:[\u0022ababc\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022ababc\u0022,\u0022indices\u0022:[[0,5]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?:(?<=a(?=b))b|a)+c\u0022}"
        "(?=(?<=a(?=b))b)b" flags="d" input="qab" "{\u0022matches\u0022:[\u0022b\u0022],\u0022index\u0022:2,\u0022input\u0022:\u0022qab\u0022,\u0022indices\u0022:[[2,3]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?=(?<=a(?=b))b)b\u0022}"
        "(?<=[\u005cuD800](?=[\u005cuDC00]))[\u005cuDC00]" flags="d" input="\ud800\udc00" "{\u0022matches\u0022:[\u0022\u005cudc00\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022\ud800\udc00\u0022,\u0022indices\u0022:[[1,2]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=[\u005c\u005cuD800](?=[\u005c\u005cuDC00]))[\u005c\u005cuDC00]\u0022}"
        "(?<=(?=ab)(?<x>a))b" flags="d" input="qab" "{\u0022matches\u0022:[\u0022b\u0022,\u0022a\u0022],\u0022index\u0022:2,\u0022input\u0022:\u0022qab\u0022,\u0022groups\u0022:{\u0022x\u0022:\u0022a\u0022},\u0022indices\u0022:[[2,3],[1,2]],\u0022indicesGroups\u0022:{\u0022x\u0022:[1,2]},\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=(?=ab)(?<x>a))b\u0022}"
        "(?<=(?=ab)(?:(?<x>a)|(?<x>c)))b" flags="d" input="qab" "{\u0022matches\u0022:[\u0022b\u0022,\u0022a\u0022,null],\u0022index\u0022:2,\u0022input\u0022:\u0022qab\u0022,\u0022groups\u0022:{\u0022x\u0022:\u0022a\u0022},\u0022indices\u0022:[[2,3],[1,2],null],\u0022indicesGroups\u0022:{\u0022x\u0022:[1,2]},\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=(?=ab)(?:(?<x>a)|(?<x>c)))b\u0022}"
        "(?<=((?=b)){2})b" flags="dg" input="qb" "{\u0022matches\u0022:[\u0022b\u0022,\u0022\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022qb\u0022,\u0022indices\u0022:[[1,2],[1,1]],\u0022lastIndex\u0022:2,\u0022source\u0022:\u0022(?<=((?=b)){2})b\u0022}"
        "(?<=a(?=b$))b" flags="dm" input="ab\u000aq" "{\u0022matches\u0022:[\u0022b\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022ab\u005cnq\u0022,\u0022indices\u0022:[[1,2]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=a(?=b$))b\u0022}"
        "(?<=a(?=b.))b" flags="ds" input="ab\u000a" "{\u0022matches\u0022:[\u0022b\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022ab\u005cn\u0022,\u0022indices\u0022:[[1,2]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=a(?=b.))b\u0022}"
        "#);
}

#[test]
fn parent_named_ranges_optional_slots_atomic_negative_and_full_input_context() {
    assert_true(
        r"let a=/(?<=(?=ab)(?<x>a))b/d.exec('qab');a.groups.x==='a'&&a.indices.groups.x===a.indices[1]&&a.indices[1][0]===1&&a.indices[1][1]===2&&a.index===2",
    );
    assert_true(
        r"let a=/(?<=((?=b)){2})b/d.exec('qb'),b=/(?<=((?=b))*)b/d.exec('qb');a[1]===''&&a.indices[1][0]===1&&b[1]===undefined&&b.indices[1]===undefined",
    );
    assert_true(
        r"let a=/(?<=a(?=b.)|a(?!b.))b/d.exec('ab');a.index===1&&a[0]==='b'&&/(?<!a(?=b))b/.exec('ab')===null&&/(?<!a(?!b))b/.exec('ab').index===1",
    );
    assert_true(
        r"/(?<=a(?=b$))b/m.exec('ab\nq').index===1&&/(?<=a(?=b.))b/s.exec('ab\n').index===1&&/(?<=a(?=b.))b/.exec('ab\n')===null",
    );
}

#[test]
fn shared_consumers_sticky_global_empty_advancement_and_callback_ranges() {
    assert_true(
        r"let r=/(?<=(?=ab)(a))b/dg,a=[...'qab ab'.matchAll(r)];a.length===2&&a[1].index===5&&a[1].indices[1][0]===4&&r.lastIndex===0",
    );
    assert_true(
        r"let r=/(?<=a(?=b))b/dy;r.lastIndex=2;let a=r.exec('qab');a.index===2&&r.lastIndex===3&&r.exec('qab')===null&&r.lastIndex===0",
    );
    assert_true(
        r"let r=/(?<=((?=b)){2})/dg,a=[...'bb'.matchAll(r)];a.length===2&&a[1].index===1&&a[1][1]===''&&a[1].indices[1][0]===1&&r.lastIndex===0",
    );
    assert_true(
        r"let seen=[];let s='qab ab'.replace(/(?<=(?=ab)(a))b/g,(m,c,i)=>{seen.push(c,i);return '_'});s==='qa_ a_'&&seen.join('|')==='a|2|a|5'",
    );
    assert_true(
        r"'qab'.search(/(?<=a(?=b))b/)===2&&'ab ab'.split(/(?<=a(?=b))b/).join('|')==='a| a|'&&'qab'.replace(/(?<=(?=ab)(?<x>a))b/,'<$<x>>')==='qa<a>'",
    );
}

#[test]
fn explicit_work_aborts_keep_last_index_and_bypass_handlers() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval("let marker=0,r=/(?<=a(?=b))b/g;r.lastIndex=1;let text='a'.repeat(5000)")
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
fn composition_results_match_reference() {
    for source in [
        r"/(?<=a(?=(b+)))b/.exec('ab')",
        r"/(?<=a(?=b+))b/.exec('ab')",
        r"/(?<=a(?=b|cc))b/.exec('ab')",
        r"/(?<=a(?!b+))b/.exec('ab')",
        r"/(?<=a+)b/.exec('ab')",
        r"/(?<=a(?=b))b/u.exec('ab')",
        r"/(?<=a(?=b))b/v.exec('ab')",
    ] {
        crate::common::regexp_oracle::assert_matches_reference(&mut Realm::default(), source);
    }
}
