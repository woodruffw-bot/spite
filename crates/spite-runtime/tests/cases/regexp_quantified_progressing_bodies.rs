//! Deterministic capture-free inner quantifiers in progressing repeated bodies.

use spite_core::JsString;
use spite_runtime::{Error, Limits, Realm, Value};
use std::fmt::Write;

use crate::common::assert_true;

#[test]
fn quantified_progressing_body_result_snapshot() {
    let mut rows = String::new();
    for (source, flags, text) in [
        (r"(?:a+b|c)+d", "d", "aabccd"),
        (r"(?:a+?b|c)+?d", "d", "aabccd"),
        (r"(?:a*b|c)+d", "d", "bbcd"),
        (r"(?:a*?b|c)+?d", "d", "bbcd"),
        (r"(?:a{2,3}b|c){2,3}d", "d", "aaabccd"),
        (r"(?:a{2,3}?b|c){2,3}?d", "d", "aaabccd"),
        (r"(?:a?b|c)+d", "d", "abbcd"),
        (r"(?:a??b|c)+d", "d", "abbcd"),
        (r"(?:(?:ab)+c|d)+e", "d", "ababcdde"),
        (r"(?:(?:ab)+?c|d)+?e", "d", "ababcdde"),
        (r"(?:(?:ab)*c|d)+e", "d", "ccde"),
        (r"(?:(?:ab)*?c|d)+?e", "d", "ccde"),
        (r"((?:a+b|c)+)d", "d", "aabccd"),
        (r"((?:a+?b|c)+?)d", "d", "aabccd"),
        (r"(a)(?:\1*b|c)+\1", "d", "aaabca"),
        (r"(a)(?:\1*?b|c)+?\1", "d", "aaabca"),
        (r"(?:(?:a|)b+)+c", "d", "abbbc"),
        (r"(?:(?:a|)b+?)+?c", "d", "abbbc"),
        (r"(?:[ab]+c|d)+e", "di", "aABcdde"),
        (r"(?:.+b|a)+c", "ds", "\n\nbac"),
        (r"(?<all>(?:a+b|c)+)d", "d", "aabccd"),
        (r"(?<x>a)(?:\k<x>*b|c)+\k<x>", "dgi", "aaabca"),
        (r"(?:(?<x>(?:a+b|c)+)d|(?<x>a+))\k<x>", "d", "aabcdaabc"),
        (r"(?<all>(?:(?:^a)+b|c)+)", "dm", "q\nab"),
        (r"(?:a+b+){2}c", "d", "aabaabc"),
        (r"(?:a+?b+?){2}?c", "d", "aabaabc"),
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
        "(?:a+b|c)+d" flags="d" input="aabccd" "{\u0022matches\u0022:[\u0022aabccd\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022aabccd\u0022,\u0022indices\u0022:[[0,6]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?:a+b|c)+d\u0022}"
        "(?:a+?b|c)+?d" flags="d" input="aabccd" "{\u0022matches\u0022:[\u0022aabccd\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022aabccd\u0022,\u0022indices\u0022:[[0,6]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?:a+?b|c)+?d\u0022}"
        "(?:a*b|c)+d" flags="d" input="bbcd" "{\u0022matches\u0022:[\u0022bbcd\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022bbcd\u0022,\u0022indices\u0022:[[0,4]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?:a*b|c)+d\u0022}"
        "(?:a*?b|c)+?d" flags="d" input="bbcd" "{\u0022matches\u0022:[\u0022bbcd\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022bbcd\u0022,\u0022indices\u0022:[[0,4]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?:a*?b|c)+?d\u0022}"
        "(?:a{2,3}b|c){2,3}d" flags="d" input="aaabccd" "{\u0022matches\u0022:[\u0022aaabccd\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022aaabccd\u0022,\u0022indices\u0022:[[0,7]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?:a{2,3}b|c){2,3}d\u0022}"
        "(?:a{2,3}?b|c){2,3}?d" flags="d" input="aaabccd" "{\u0022matches\u0022:[\u0022aaabccd\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022aaabccd\u0022,\u0022indices\u0022:[[0,7]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?:a{2,3}?b|c){2,3}?d\u0022}"
        "(?:a?b|c)+d" flags="d" input="abbcd" "{\u0022matches\u0022:[\u0022abbcd\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022abbcd\u0022,\u0022indices\u0022:[[0,5]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?:a?b|c)+d\u0022}"
        "(?:a??b|c)+d" flags="d" input="abbcd" "{\u0022matches\u0022:[\u0022abbcd\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022abbcd\u0022,\u0022indices\u0022:[[0,5]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?:a??b|c)+d\u0022}"
        "(?:(?:ab)+c|d)+e" flags="d" input="ababcdde" "{\u0022matches\u0022:[\u0022ababcdde\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022ababcdde\u0022,\u0022indices\u0022:[[0,8]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?:(?:ab)+c|d)+e\u0022}"
        "(?:(?:ab)+?c|d)+?e" flags="d" input="ababcdde" "{\u0022matches\u0022:[\u0022ababcdde\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022ababcdde\u0022,\u0022indices\u0022:[[0,8]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?:(?:ab)+?c|d)+?e\u0022}"
        "(?:(?:ab)*c|d)+e" flags="d" input="ccde" "{\u0022matches\u0022:[\u0022ccde\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022ccde\u0022,\u0022indices\u0022:[[0,4]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?:(?:ab)*c|d)+e\u0022}"
        "(?:(?:ab)*?c|d)+?e" flags="d" input="ccde" "{\u0022matches\u0022:[\u0022ccde\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022ccde\u0022,\u0022indices\u0022:[[0,4]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?:(?:ab)*?c|d)+?e\u0022}"
        "((?:a+b|c)+)d" flags="d" input="aabccd" "{\u0022matches\u0022:[\u0022aabccd\u0022,\u0022aabcc\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022aabccd\u0022,\u0022indices\u0022:[[0,6],[0,5]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022((?:a+b|c)+)d\u0022}"
        "((?:a+?b|c)+?)d" flags="d" input="aabccd" "{\u0022matches\u0022:[\u0022aabccd\u0022,\u0022aabcc\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022aabccd\u0022,\u0022indices\u0022:[[0,6],[0,5]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022((?:a+?b|c)+?)d\u0022}"
        "(a)(?:\u005c1*b|c)+\u005c1" flags="d" input="aaabca" "{\u0022matches\u0022:[\u0022aaabca\u0022,\u0022a\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022aaabca\u0022,\u0022indices\u0022:[[0,6],[0,1]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a)(?:\u005c\u005c1*b|c)+\u005c\u005c1\u0022}"
        "(a)(?:\u005c1*?b|c)+?\u005c1" flags="d" input="aaabca" "{\u0022matches\u0022:[\u0022aaabca\u0022,\u0022a\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022aaabca\u0022,\u0022indices\u0022:[[0,6],[0,1]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a)(?:\u005c\u005c1*?b|c)+?\u005c\u005c1\u0022}"
        "(?:(?:a|)b+)+c" flags="d" input="abbbc" "{\u0022matches\u0022:[\u0022abbbc\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022abbbc\u0022,\u0022indices\u0022:[[0,5]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?:(?:a|)b+)+c\u0022}"
        "(?:(?:a|)b+?)+?c" flags="d" input="abbbc" "{\u0022matches\u0022:[\u0022abbbc\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022abbbc\u0022,\u0022indices\u0022:[[0,5]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?:(?:a|)b+?)+?c\u0022}"
        "(?:[ab]+c|d)+e" flags="di" input="aABcdde" "{\u0022matches\u0022:[\u0022aABcdde\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022aABcdde\u0022,\u0022indices\u0022:[[0,7]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?:[ab]+c|d)+e\u0022}"
        "(?:.+b|a)+c" flags="ds" input="\u000a\u000abac" "{\u0022matches\u0022:[\u0022\u005cn\u005cnbac\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022\u005cn\u005cnbac\u0022,\u0022indices\u0022:[[0,5]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?:.+b|a)+c\u0022}"
        "(?<all>(?:a+b|c)+)d" flags="d" input="aabccd" "{\u0022matches\u0022:[\u0022aabccd\u0022,\u0022aabcc\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022aabccd\u0022,\u0022groups\u0022:{\u0022all\u0022:\u0022aabcc\u0022},\u0022indices\u0022:[[0,6],[0,5]],\u0022indicesGroups\u0022:{\u0022all\u0022:[0,5]},\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<all>(?:a+b|c)+)d\u0022}"
        "(?<x>a)(?:\u005ck<x>*b|c)+\u005ck<x>" flags="dgi" input="aaabca" "{\u0022matches\u0022:[\u0022aaabca\u0022,\u0022a\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022aaabca\u0022,\u0022groups\u0022:{\u0022x\u0022:\u0022a\u0022},\u0022indices\u0022:[[0,6],[0,1]],\u0022indicesGroups\u0022:{\u0022x\u0022:[0,1]},\u0022lastIndex\u0022:6,\u0022source\u0022:\u0022(?<x>a)(?:\u005c\u005ck<x>*b|c)+\u005c\u005ck<x>\u0022}"
        "(?:(?<x>(?:a+b|c)+)d|(?<x>a+))\u005ck<x>" flags="d" input="aabcdaabc" "{\u0022matches\u0022:[\u0022aabcdaabc\u0022,\u0022aabc\u0022,null],\u0022index\u0022:0,\u0022input\u0022:\u0022aabcdaabc\u0022,\u0022groups\u0022:{\u0022x\u0022:\u0022aabc\u0022},\u0022indices\u0022:[[0,9],[0,4],null],\u0022indicesGroups\u0022:{\u0022x\u0022:[0,4]},\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?:(?<x>(?:a+b|c)+)d|(?<x>a+))\u005c\u005ck<x>\u0022}"
        "(?<all>(?:(?:^a)+b|c)+)" flags="dm" input="q\u000aab" "{\u0022matches\u0022:[\u0022ab\u0022,\u0022ab\u0022],\u0022index\u0022:2,\u0022input\u0022:\u0022q\u005cnab\u0022,\u0022groups\u0022:{\u0022all\u0022:\u0022ab\u0022},\u0022indices\u0022:[[2,4],[2,4]],\u0022indicesGroups\u0022:{\u0022all\u0022:[2,4]},\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<all>(?:(?:^a)+b|c)+)\u0022}"
        "(?:a+b+){2}c" flags="d" input="aabaabc" "{\u0022matches\u0022:[\u0022aabaabc\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022aabaabc\u0022,\u0022indices\u0022:[[0,7]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?:a+b+){2}c\u0022}"
        "(?:a+?b+?){2}?c" flags="d" input="aabaabc" "{\u0022matches\u0022:[\u0022aabaabc\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022aabaabc\u0022,\u0022indices\u0022:[[0,7]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?:a+?b+?){2}?c\u0022}"
        "#);
}

#[test]
fn inner_counts_retry_before_iteration_counts_and_outside_capture_closes() {
    assert_true(
        r"let a=/(?<all>(?:a+b+){2})c/d.exec('aabaabc');a[0]==='aabaabc'&&a.groups.all==='aabaab'&&a.indices.groups.all===a.indices[1]&&a.indices.groups.all[1]===6",
    );
    assert_true(
        r"let a=/(?<all>(?:a+a|b)+)a/d.exec('aaaa');a[0]==='aaaa'&&a.groups.all==='aaa'&&a.indices.groups.all===a.indices[1]&&a.indices.groups.all[1]===3",
    );
    assert_true(
        r"let a=/(?<all>(?:a+?a|b)+?)a/d.exec('aaaa');a[0]==='aaa'&&a.groups.all==='aa'&&a.indices.groups.all[1]===2",
    );
    assert_true(
        r"let a=/(?<x>a)(?:\k<x>*b|c)+\k<x>/d.exec('aaabca');a[0]==='aaabca'&&a.groups.x==='a'&&a.indices.groups.x===a.indices[1]&&a.indices.groups.x[1]===1",
    );
    assert_true(
        r"let a=/(?<all>(?:\k<all>*a|b)+)\k<all>/d.exec('abab');a[0]==='abab'&&a.groups.all==='ab'&&a.indices.groups.all[1]===2",
    );
    assert_true(
        r"let a=/(?<x>a)(?:\k<later>*b|c)+(?<later>\k<x>)/d.exec('abbca');a[0]==='abbca'&&a.groups.later==='a'&&a.indices.groups.later[0]===4",
    );
    assert_true(
        r"let a=/(?:(?<x>(?:a+b|c)+)d|(?<x>a+))\k<x>/d.exec('aabcdaabc');a[0]==='aabcdaabc'&&a.groups.x==='aabc'&&a[2]===undefined&&a.indices.groups.x===a.indices[1]",
    );
    assert_true(
        r"let a=/(?<all>(?:a+b|c)+(?:a+b|c)+)/d.exec('aabcc');a[0]==='aabcc'&&a.groups.all==='aabcc'&&a.indices.groups.all[1]===5",
    );
    assert_true(
        r"let a=/(?<all>(?:.+b|a)+)c/ds.exec('\uD800\nbac');a.groups.all.length===4&&a.groups.all.charCodeAt(0)===0xD800&&a.indices.groups.all[1]===4",
    );
}

#[test]
fn string_consumers_sticky_counts_and_zero_advancement_preserve_inner_quantifiers() {
    assert_true(
        r"let r=/(?<all>(?:a+b|c)+)d/dg,a=[...'aabccd abd'.matchAll(r)];a.length===2&&a[0][0]==='aabccd'&&a[0].groups.all==='aabcc'&&a[0].indices.groups.all[1]===5&&a[1].index===7&&a[1].groups.all==='ab'&&r.lastIndex===0",
    );
    assert_true(
        r"'qaabccdZ'.replace(/(?<all>(?:a+b|c)+)d/,'<$<all>>')==='q<aabcc>Z'&&'qaabccdZ'.search(/((?:a+b|c)+)d/)===1&&'qaabccdZ'.split(/((?:a+b|c)+)d/).join(',')==='q,aabcc,Z'",
    );
    assert_true(
        r"let r=/(?<all>(?:a+b|c)+)d/dy;r.lastIndex=1;let copy=new RegExp(r),a=r.exec('qaabccd');a.groups.all==='aabcc'&&a.indices.groups.all[0]===1&&r.lastIndex===7&&r.exec('qaabccd')===null&&r.lastIndex===0&&copy.source===r.source&&copy.lastIndex===0",
    );
    assert_true(
        r"let a=[...'q'.matchAll(/(?<all>(?:a+b|c)*)/dg)];a.length===2&&a[0].groups.all===''&&a[1].groups.all===''&&a[1].indices.groups.all[0]===1&&a[1].indices.groups.all[1]===1",
    );
    assert_true(
        r"let a=/((?:a{2,3}b|c){2,3})d/d.exec('aaabccd'),b=/(?:a+b|c)*d|a/.exec('a');a[0]==='aaabccd'&&a[1]==='aaabcc'&&a.indices[1][1]===6&&b[0]==='a'",
    );
}

#[test]
fn opted_in_inner_iteration_work_aborts_before_last_index_and_handlers() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval("let marker=0,r=/(?<all>(?:a+b|c)+)d/g;r.lastIndex=1;let text='ab'.repeat(5000)")
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
fn possibly_empty_bodies_inner_captures_and_nested_branch_loops_match_reference_results() {
    for source in [
        r"/(?:a*b*){2,3}/.test('ab')",
        r"/(?:(?:(?:(a+)b|c)+)|){2,3}/.test('aab')",
        r"/(?:(?:((?:(?:a|b)+c|d)+){2})|){2,3}/.test('abc')",
        r"/(?:\1*|b){2,3}(a)/.test('ba')",
        r"/(?:(?<=(a+))a+|b)+/.test('a')",
        r"/(?:a+b|c)+/u.test('aab')",
    ] {
        crate::common::regexp_oracle::assert_matches_reference(&mut Realm::default(), source);
    }
    assert_true(r"/(?:a+b|c)+d/.exec('aab')===null");
}
