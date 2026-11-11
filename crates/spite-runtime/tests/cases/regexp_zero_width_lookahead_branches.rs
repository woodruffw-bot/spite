//! Source-order alternatives in proven zero-width lookahead repetition bodies.
use spite_core::JsString;
use spite_runtime::{Error, Limits, Realm, Value};
use std::fmt::Write;

use crate::common::assert_true;

#[test]
fn zero_width_lookahead_branch_result_snapshot() {
    let mut rows = String::new();
    for (source, flags, text) in [
        (r"(?:(?=(a))|()){2}\1\2", "d", "a"),
        (r"(?:(?=(a))|())*\1\2", "d", "a"),
        (r"(?:(?=(a))|()){2,4}?\1\2", "d", "b"),
        (r"((?=(a))|()){2}\1\2\3", "d", "b"),
        (r"((?=(a))|()){0,2}\1\2\3", "d", "a"),
        (r"(?:(?=(?<x>a))|(?=(?<x>b))){2}\k<x>", "d", "b"),
        (r"(?:(?=(a|ab))|(?=(ab))){2}\1b$", "d", "abb"),
        (r"(?:(?=(ab|a))|(?=(a))){2}\1b$", "d", "abb"),
        (r"(?:(?=(a))|(?=(ab))){2}\1\2b$", "d", "abb"),
        (r"(?:(?!(a)b)|(?=(a))){2}\1\2a", "d", "a"),
        (r"(?:(?!(a)b)|(?!(b)a)){2}a", "d", "ab"),
        (r"(?:(?=(a))|^){2}\1", "d", "b"),
        (r"(?:(?=(a))|$){2}\1", "d", "b"),
        (r"(?:(\b)(?=(a))|()){2}a", "d", "a"),
        (r"(?:(?=(a))()|(?=(b))()){2}\1\2\3\4", "d", "b"),
        (r"(?:(?=(a))|(?!(b))){2}a", "d", "a"),
        (r"(?:(?!(a))|(?=(b))){2}b", "d", "b"),
        (r"(?:(?:(?=(a))|()){2}){3}\1", "d", "a"),
        (r"(?:(?:(?=(a))|())?){2}a", "d", "a"),
        (r"(?:(?:(?=(?<x>a))|()){2}a|b)+c", "d", "abc"),
        (r"(?:(?:(?=(a))|())?a|b)+c", "d", "abc"),
        (r"(?:(?=(^a))|()){2}\1", "dm", "q\na"),
        (r"(?:(?=(a|b)+)|()){2}\1", "d", "ab"),
        (r"(?:(?=(?<x>a))|(?=(?<x>b))){0}c", "d", "ac"),
        (r"(?:(?=(a))|(?=(b))){1}a|b", "d", "b"),
        (r"(?:(?=)|()){2}a", "d", "a"),
        (r"(?:(?!)|()){2}a", "d", "a"),
        (r"(?:(?=(?<x>µ))|()){2}\k<x>", "di", "Μ"),
    ] {
        let source = JsString::from(source);
        let flags = JsString::from(flags);
        let text = JsString::from(text);
        let program = format!(
            "let r=new RegExp({source:?},{flags:?}),a=r.exec({text:?});JSON.stringify(a===null?{{match:null,lastIndex:r.lastIndex}}:{{matches:[...a],index:a.index,input:a.input,groups:a.groups,indices:a.indices,indicesGroups:a.indices.groups,lastIndex:r.lastIndex,source:r.source}})"
        );
        let Value::String(value) = Realm::default().eval(&program).unwrap() else {
            panic!("expected JSON")
        };
        writeln!(rows, "{source:?} flags={flags:?} input={text:?} {value:?}").unwrap();
    }
    insta::assert_snapshot!(rows, @r#"
        "(?:(?=(a))|()){2}\u005c1\u005c2" flags="d" input="a" "{\u0022matches\u0022:[\u0022a\u0022,\u0022a\u0022,null],\u0022index\u0022:0,\u0022input\u0022:\u0022a\u0022,\u0022indices\u0022:[[0,1],[0,1],null],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?:(?=(a))|()){2}\u005c\u005c1\u005c\u005c2\u0022}"
        "(?:(?=(a))|())*\u005c1\u005c2" flags="d" input="a" "{\u0022matches\u0022:[\u0022\u0022,null,null],\u0022index\u0022:0,\u0022input\u0022:\u0022a\u0022,\u0022indices\u0022:[[0,0],null,null],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?:(?=(a))|())*\u005c\u005c1\u005c\u005c2\u0022}"
        "(?:(?=(a))|()){2,4}?\u005c1\u005c2" flags="d" input="b" "{\u0022matches\u0022:[\u0022\u0022,null,\u0022\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022b\u0022,\u0022indices\u0022:[[0,0],null,[0,0]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?:(?=(a))|()){2,4}?\u005c\u005c1\u005c\u005c2\u0022}"
        "((?=(a))|()){2}\u005c1\u005c2\u005c3" flags="d" input="b" "{\u0022matches\u0022:[\u0022\u0022,\u0022\u0022,null,\u0022\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022b\u0022,\u0022indices\u0022:[[0,0],[0,0],null,[0,0]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022((?=(a))|()){2}\u005c\u005c1\u005c\u005c2\u005c\u005c3\u0022}"
        "((?=(a))|()){0,2}\u005c1\u005c2\u005c3" flags="d" input="a" "{\u0022matches\u0022:[\u0022\u0022,null,null,null],\u0022index\u0022:0,\u0022input\u0022:\u0022a\u0022,\u0022indices\u0022:[[0,0],null,null,null],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022((?=(a))|()){0,2}\u005c\u005c1\u005c\u005c2\u005c\u005c3\u0022}"
        "(?:(?=(?<x>a))|(?=(?<x>b))){2}\u005ck<x>" flags="d" input="b" "{\u0022matches\u0022:[\u0022b\u0022,null,\u0022b\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022b\u0022,\u0022groups\u0022:{\u0022x\u0022:\u0022b\u0022},\u0022indices\u0022:[[0,1],null,[0,1]],\u0022indicesGroups\u0022:{\u0022x\u0022:[0,1]},\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?:(?=(?<x>a))|(?=(?<x>b))){2}\u005c\u005ck<x>\u0022}"
        "(?:(?=(a|ab))|(?=(ab))){2}\u005c1b$" flags="d" input="abb" "{\u0022match\u0022:null,\u0022lastIndex\u0022:0}"
        "(?:(?=(ab|a))|(?=(a))){2}\u005c1b$" flags="d" input="abb" "{\u0022matches\u0022:[\u0022abb\u0022,\u0022ab\u0022,null],\u0022index\u0022:0,\u0022input\u0022:\u0022abb\u0022,\u0022indices\u0022:[[0,3],[0,2],null],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?:(?=(ab|a))|(?=(a))){2}\u005c\u005c1b$\u0022}"
        "(?:(?=(a))|(?=(ab))){2}\u005c1\u005c2b$" flags="d" input="abb" "{\u0022matches\u0022:[\u0022abb\u0022,null,\u0022ab\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022abb\u0022,\u0022indices\u0022:[[0,3],null,[0,2]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?:(?=(a))|(?=(ab))){2}\u005c\u005c1\u005c\u005c2b$\u0022}"
        "(?:(?!(a)b)|(?=(a))){2}\u005c1\u005c2a" flags="d" input="a" "{\u0022matches\u0022:[\u0022a\u0022,null,null],\u0022index\u0022:0,\u0022input\u0022:\u0022a\u0022,\u0022indices\u0022:[[0,1],null,null],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?:(?!(a)b)|(?=(a))){2}\u005c\u005c1\u005c\u005c2a\u0022}"
        "(?:(?!(a)b)|(?!(b)a)){2}a" flags="d" input="ab" "{\u0022matches\u0022:[\u0022a\u0022,null,null],\u0022index\u0022:0,\u0022input\u0022:\u0022ab\u0022,\u0022indices\u0022:[[0,1],null,null],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?:(?!(a)b)|(?!(b)a)){2}a\u0022}"
        "(?:(?=(a))|^){2}\u005c1" flags="d" input="b" "{\u0022matches\u0022:[\u0022\u0022,null],\u0022index\u0022:0,\u0022input\u0022:\u0022b\u0022,\u0022indices\u0022:[[0,0],null],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?:(?=(a))|^){2}\u005c\u005c1\u0022}"
        "(?:(?=(a))|$){2}\u005c1" flags="d" input="b" "{\u0022matches\u0022:[\u0022\u0022,null],\u0022index\u0022:1,\u0022input\u0022:\u0022b\u0022,\u0022indices\u0022:[[1,1],null],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?:(?=(a))|$){2}\u005c\u005c1\u0022}"
        "(?:(\u005cb)(?=(a))|()){2}a" flags="d" input="a" "{\u0022matches\u0022:[\u0022a\u0022,\u0022\u0022,\u0022a\u0022,null],\u0022index\u0022:0,\u0022input\u0022:\u0022a\u0022,\u0022indices\u0022:[[0,1],[0,0],[0,1],null],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?:(\u005c\u005cb)(?=(a))|()){2}a\u0022}"
        "(?:(?=(a))()|(?=(b))()){2}\u005c1\u005c2\u005c3\u005c4" flags="d" input="b" "{\u0022matches\u0022:[\u0022b\u0022,null,null,\u0022b\u0022,\u0022\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022b\u0022,\u0022indices\u0022:[[0,1],null,null,[0,1],[0,0]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?:(?=(a))()|(?=(b))()){2}\u005c\u005c1\u005c\u005c2\u005c\u005c3\u005c\u005c4\u0022}"
        "(?:(?=(a))|(?!(b))){2}a" flags="d" input="a" "{\u0022matches\u0022:[\u0022a\u0022,\u0022a\u0022,null],\u0022index\u0022:0,\u0022input\u0022:\u0022a\u0022,\u0022indices\u0022:[[0,1],[0,1],null],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?:(?=(a))|(?!(b))){2}a\u0022}"
        "(?:(?!(a))|(?=(b))){2}b" flags="d" input="b" "{\u0022matches\u0022:[\u0022b\u0022,null,null],\u0022index\u0022:0,\u0022input\u0022:\u0022b\u0022,\u0022indices\u0022:[[0,1],null,null],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?:(?!(a))|(?=(b))){2}b\u0022}"
        "(?:(?:(?=(a))|()){2}){3}\u005c1" flags="d" input="a" "{\u0022matches\u0022:[\u0022a\u0022,\u0022a\u0022,null],\u0022index\u0022:0,\u0022input\u0022:\u0022a\u0022,\u0022indices\u0022:[[0,1],[0,1],null],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?:(?:(?=(a))|()){2}){3}\u005c\u005c1\u0022}"
        "(?:(?:(?=(a))|())?){2}a" flags="d" input="a" "{\u0022matches\u0022:[\u0022a\u0022,null,null],\u0022index\u0022:0,\u0022input\u0022:\u0022a\u0022,\u0022indices\u0022:[[0,1],null,null],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?:(?:(?=(a))|())?){2}a\u0022}"
        "(?:(?:(?=(?<x>a))|()){2}a|b)+c" flags="d" input="abc" "{\u0022matches\u0022:[\u0022abc\u0022,null,null],\u0022index\u0022:0,\u0022input\u0022:\u0022abc\u0022,\u0022groups\u0022:{},\u0022indices\u0022:[[0,3],null,null],\u0022indicesGroups\u0022:{},\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?:(?:(?=(?<x>a))|()){2}a|b)+c\u0022}"
        "(?:(?:(?=(a))|())?a|b)+c" flags="d" input="abc" "{\u0022matches\u0022:[\u0022abc\u0022,null,null],\u0022index\u0022:0,\u0022input\u0022:\u0022abc\u0022,\u0022indices\u0022:[[0,3],null,null],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?:(?:(?=(a))|())?a|b)+c\u0022}"
        "(?:(?=(^a))|()){2}\u005c1" flags="dm" input="q\u000aa" "{\u0022matches\u0022:[\u0022\u0022,null,\u0022\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022q\u005cna\u0022,\u0022indices\u0022:[[0,0],null,[0,0]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?:(?=(^a))|()){2}\u005c\u005c1\u0022}"
        "(?:(?=(a|b)+)|()){2}\u005c1" flags="d" input="ab" "{\u0022matches\u0022:[\u0022\u0022,null,\u0022\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022ab\u0022,\u0022indices\u0022:[[0,0],null,[0,0]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?:(?=(a|b)+)|()){2}\u005c\u005c1\u0022}"
        "(?:(?=(?<x>a))|(?=(?<x>b))){0}c" flags="d" input="ac" "{\u0022matches\u0022:[\u0022c\u0022,null,null],\u0022index\u0022:1,\u0022input\u0022:\u0022ac\u0022,\u0022groups\u0022:{},\u0022indices\u0022:[[1,2],null,null],\u0022indicesGroups\u0022:{},\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?:(?=(?<x>a))|(?=(?<x>b))){0}c\u0022}"
        "(?:(?=(a))|(?=(b))){1}a|b" flags="d" input="b" "{\u0022matches\u0022:[\u0022b\u0022,null,null],\u0022index\u0022:0,\u0022input\u0022:\u0022b\u0022,\u0022indices\u0022:[[0,1],null,null],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?:(?=(a))|(?=(b))){1}a|b\u0022}"
        "(?:(?=)|()){2}a" flags="d" input="a" "{\u0022matches\u0022:[\u0022a\u0022,null],\u0022index\u0022:0,\u0022input\u0022:\u0022a\u0022,\u0022indices\u0022:[[0,1],null],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?:(?=)|()){2}a\u0022}"
        "(?:(?!)|()){2}a" flags="d" input="a" "{\u0022matches\u0022:[\u0022a\u0022,\u0022\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022a\u0022,\u0022indices\u0022:[[0,1],[0,0]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?:(?!)|()){2}a\u0022}"
        "(?:(?=(?<x>\u00b5))|()){2}\u005ck<x>" flags="di" input="\u039c" "{\u0022matches\u0022:[\u0022\u039c\u0022,\u0022\u039c\u0022,null],\u0022index\u0022:0,\u0022input\u0022:\u0022\u039c\u0022,\u0022groups\u0022:{\u0022x\u0022:\u0022\u039c\u0022},\u0022indices\u0022:[[0,1],[0,1],null],\u0022indicesGroups\u0022:{\u0022x\u0022:[0,1]},\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?:(?=(?<x>\u00b5))|()){2}\u005c\u005ck<x>\u0022}"
        "#);
}

#[test]
fn zero_width_branch_retries_restore_inactive_slots_named_aliases_and_atomic_assertions() {
    assert_true(
        r"let a=/(?:(?=(?<x>a))|(?=(?<x>ab))){2}\k<x>b$/d.exec('abb');a[0]==='abb'&&a[1]===undefined&&a[2]==='ab'&&a.groups.x==='ab'&&a.indices.groups.x===a.indices[2]&&a.indices[1]===undefined&&a.indices[2][1]===2",
    );
    assert_true(
        r"let a=/((?=(a))|()){2}\1\2\3/d.exec('b');a[0]===''&&a[1]===''&&a[2]===undefined&&a[3]===''&&a.indices[1][1]===0&&a.indices[2]===undefined&&a.indices[3][1]===0",
    );
    assert_true(
        r"/(?:(?=(a|ab))|(?=(ab))){2}\1b$/.exec('abb')===null&&/(?:(?=(ab|a))|(?=(a))){2}\1b$/.exec('abb')[1]==='ab'",
    );
    assert_true(
        r"let a=/(?:(?=(?<x>a))|(?=(?<x>b))){0,3}\k<x>c/d.exec('ac');a[0]==='c'&&a.index===1&&a.groups.x===undefined&&a.indices.groups.x===undefined",
    );
    assert_true(
        r"let a=/(?:(?!(?<x>a)b)|(?=(?<x>a))){2}a/d.exec('a');a.groups.x===undefined&&a.indices.groups.x===undefined&&a[1]===undefined&&a[2]===undefined",
    );
    assert_true(
        r"let a=/(?:(?:(?=(?<x>a))|()){2}a|b)+c/d.exec('abc');a[0]==='abc'&&a.groups.x===undefined&&a.indices.groups.x===undefined",
    );
    assert_true(
        r"let a=/(?:(?=(\uD800))|()){2}\1/d.exec('\uD800');a[0].charCodeAt(0)===0xD800&&a[1].charCodeAt(0)===0xD800&&a.indices[1][1]===1&&a[2]===undefined",
    );
}

#[test]
fn zero_width_branch_consumers_empty_global_sticky_results_and_callbacks_keep_slots() {
    assert_true(
        r"let r=/(?:(?=(?<x>a))|(?=(?<x>b))){2}/dg,a=[...'ab'.matchAll(r)];a.length===2&&a[0][0]===''&&a[0].groups.x==='a'&&a[0].indices.groups.x===a[0].indices[1]&&a[1].index===1&&a[1][1]===undefined&&a[1].groups.x==='b'&&a[1].indices.groups.x===a[1].indices[2]&&r.lastIndex===0",
    );
    assert_true(
        r"let r=/(?:(?=(?<x>a))|(?=(?<x>b))){2}/dy;r.lastIndex=1;let a=r.exec('qb');a[0]===''&&a.groups.x==='b'&&a.indices.groups.x[0]===1&&r.lastIndex===1&&r.exec('qb').groups.x==='b'",
    );
    assert_true(
        r"let a=[];let result='ab'.replace(/(?:(?=(?<x>a))|(?=(?<x>b))){2}/g,(m,c1,c2,i,s,g)=>{a.push(m,c1,c2,i,g.x);return '_'});result==='_a_b'&&a.length===10&&a[0]===''&&a[1]==='a'&&a[2]===undefined&&a[3]===0&&a[4]==='a'&&a[6]===undefined&&a[7]==='b'&&a[8]===1&&a[9]==='b'",
    );
    assert_true(
        r"'ab'.replace(/(?:(?=(?<x>a))|(?=(?<x>b))){2}/g,'<$<x>>')==='<a>a<b>b'&&'ab'.search(/(?:(?=(a))|(?=(b))){2}/)===0&&'ab'.split(/(?:(?=(a))|(?=(b))){2}/).length===4",
    );
}

#[test]
fn explicit_zero_width_branch_search_work_aborts_before_last_index_and_handlers() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm.eval("let marker=0,r=/(?:(?=(?<x>a+))|()){2}\\k<x>b/g;r.lastIndex=1;let text='a'.repeat(5000)").unwrap();
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
fn mixed_progress_unknown_references_lookbehind_and_unicode_match_reference_results() {
    for source in [
        r"/(?:(?=(a))|b){2,3}/.exec('a')",
        r"/(?:(?=(a))\1|b){2,3}/.exec('a')",
        r"/(?<=(a+))b/.exec('ab')",
        r"/(?:(?=(a))|()){2}\1/u.exec('a')",
        r"/(?:(?=(a))|()){2}\1/v.exec('a')",
    ] {
        crate::common::regexp_oracle::assert_matches_reference(&mut Realm::default(), source);
    }
}
