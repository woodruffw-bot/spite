//! Equal-width ordinary lookbehind alternatives and atomic results (22.2.2.8).
use spite_core::JsString;
use spite_runtime::{Error, Limits, Realm, Value};
use std::fmt::Write;

use crate::common::assert_true;

#[test]
fn fixed_lookbehind_choice_result_snapshot() {
    let mut rows = String::new();
    for (source, flags, text) in [
        (r"(?<=a|b)c", "d", "bc"),
        (r"(?<!a|b)c", "d", "bc"),
        (r"(?<!a|b)c", "d", "qc"),
        (r"(?<=ab|cd)e", "d", "cde"),
        (r"(?<=(a|b))c", "d", "bc"),
        (r"(?<=(a)b|a(c))d", "d", "acd"),
        (r"(?<=((a)b|a(c)))d", "d", "acd"),
        (r"(?<!((a)b|a(c)))d", "d", "aqd"),
        (r"(?<=(a|b)(c|d))e", "d", "bde"),
        (r"(?<=(a|b))\1", "d", "bb"),
        (r"(?<=((a)|a))c\2", "d", "ac"),
        (r"(?<=(a|(a)))c\2", "d", "ac"),
        (r"(?<=()|())a", "d", "a"),
        (r"(?<!()|())a", "d", "a"),
        (r"(?<=(\b|^))a", "d", " a"),
        (r"(?<=(^a|\ba))b", "dm", "q\nab"),
        (r"(?<=([ab]){2}|(a){2})c", "d", "abc"),
        (r"(?<=((?:ab){2}|(?:ba){2}))c", "d", "babac"),
        (r"(?<=(?<=a|b))c", "d", "bc"),
        (r"(?<=(a|b)(?<!(c|d)))e", "d", "be"),
        (r"(?:(?<=(a|b))c|d)+e", "d", "abcdde"),
        (r"((?<=(a|b))){2}c\1\2", "d", "bcb"),
        (r"((?<=(a|b)))*c\1\2", "d", "bc"),
        (r"(?=(ab(?<=(a|b)(a|b))c))\1\2\3", "d", "abcab"),
        (r"(?<=(µ|Μ))a", "di", "Μa"),
        (r"(?<=(?<x>a)|(?<x>b))c", "d", "bc"),
        (r"(?<=(?<x>a)b|a(?<x>c))d", "d", "acd"),
        (r"(?<=[\uD800]|[\uDC00])c", "d", "lone surrogates"),
    ] {
        let source = JsString::from(source);
        let flags = JsString::from(flags);
        let input = if text == "lone surrogates" {
            JsString::from_code_units(vec![0xdc00, 0x63])
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
        "(?<=a|b)c" flags="d" input="bc" "{\u0022matches\u0022:[\u0022c\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022bc\u0022,\u0022indices\u0022:[[1,2]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=a|b)c\u0022}"
        "(?<!a|b)c" flags="d" input="bc" "{\u0022match\u0022:null,\u0022lastIndex\u0022:0}"
        "(?<!a|b)c" flags="d" input="qc" "{\u0022matches\u0022:[\u0022c\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022qc\u0022,\u0022indices\u0022:[[1,2]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<!a|b)c\u0022}"
        "(?<=ab|cd)e" flags="d" input="cde" "{\u0022matches\u0022:[\u0022e\u0022],\u0022index\u0022:2,\u0022input\u0022:\u0022cde\u0022,\u0022indices\u0022:[[2,3]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=ab|cd)e\u0022}"
        "(?<=(a|b))c" flags="d" input="bc" "{\u0022matches\u0022:[\u0022c\u0022,\u0022b\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022bc\u0022,\u0022indices\u0022:[[1,2],[0,1]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=(a|b))c\u0022}"
        "(?<=(a)b|a(c))d" flags="d" input="acd" "{\u0022matches\u0022:[\u0022d\u0022,null,\u0022c\u0022],\u0022index\u0022:2,\u0022input\u0022:\u0022acd\u0022,\u0022indices\u0022:[[2,3],null,[1,2]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=(a)b|a(c))d\u0022}"
        "(?<=((a)b|a(c)))d" flags="d" input="acd" "{\u0022matches\u0022:[\u0022d\u0022,\u0022ac\u0022,null,\u0022c\u0022],\u0022index\u0022:2,\u0022input\u0022:\u0022acd\u0022,\u0022indices\u0022:[[2,3],[0,2],null,[1,2]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=((a)b|a(c)))d\u0022}"
        "(?<!((a)b|a(c)))d" flags="d" input="aqd" "{\u0022matches\u0022:[\u0022d\u0022,null,null,null],\u0022index\u0022:2,\u0022input\u0022:\u0022aqd\u0022,\u0022indices\u0022:[[2,3],null,null,null],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<!((a)b|a(c)))d\u0022}"
        "(?<=(a|b)(c|d))e" flags="d" input="bde" "{\u0022matches\u0022:[\u0022e\u0022,\u0022b\u0022,\u0022d\u0022],\u0022index\u0022:2,\u0022input\u0022:\u0022bde\u0022,\u0022indices\u0022:[[2,3],[0,1],[1,2]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=(a|b)(c|d))e\u0022}"
        "(?<=(a|b))\u005c1" flags="d" input="bb" "{\u0022matches\u0022:[\u0022b\u0022,\u0022b\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022bb\u0022,\u0022indices\u0022:[[1,2],[0,1]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=(a|b))\u005c\u005c1\u0022}"
        "(?<=((a)|a))c\u005c2" flags="d" input="ac" "{\u0022match\u0022:null,\u0022lastIndex\u0022:0}"
        "(?<=(a|(a)))c\u005c2" flags="d" input="ac" "{\u0022matches\u0022:[\u0022c\u0022,\u0022a\u0022,null],\u0022index\u0022:1,\u0022input\u0022:\u0022ac\u0022,\u0022indices\u0022:[[1,2],[0,1],null],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=(a|(a)))c\u005c\u005c2\u0022}"
        "(?<=()|())a" flags="d" input="a" "{\u0022matches\u0022:[\u0022a\u0022,\u0022\u0022,null],\u0022index\u0022:0,\u0022input\u0022:\u0022a\u0022,\u0022indices\u0022:[[0,1],[0,0],null],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=()|())a\u0022}"
        "(?<!()|())a" flags="d" input="a" "{\u0022match\u0022:null,\u0022lastIndex\u0022:0}"
        "(?<=(\u005cb|^))a" flags="d" input=" a" "{\u0022matches\u0022:[\u0022a\u0022,\u0022\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022 a\u0022,\u0022indices\u0022:[[1,2],[1,1]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=(\u005c\u005cb|^))a\u0022}"
        "(?<=(^a|\u005cba))b" flags="dm" input="q\u000aab" "{\u0022matches\u0022:[\u0022b\u0022,\u0022a\u0022],\u0022index\u0022:3,\u0022input\u0022:\u0022q\u005cnab\u0022,\u0022indices\u0022:[[3,4],[2,3]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=(^a|\u005c\u005cba))b\u0022}"
        "(?<=([ab]){2}|(a){2})c" flags="d" input="abc" "{\u0022matches\u0022:[\u0022c\u0022,\u0022a\u0022,null],\u0022index\u0022:2,\u0022input\u0022:\u0022abc\u0022,\u0022indices\u0022:[[2,3],[0,1],null],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=([ab]){2}|(a){2})c\u0022}"
        "(?<=((?:ab){2}|(?:ba){2}))c" flags="d" input="babac" "{\u0022matches\u0022:[\u0022c\u0022,\u0022baba\u0022],\u0022index\u0022:4,\u0022input\u0022:\u0022babac\u0022,\u0022indices\u0022:[[4,5],[0,4]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=((?:ab){2}|(?:ba){2}))c\u0022}"
        "(?<=(?<=a|b))c" flags="d" input="bc" "{\u0022matches\u0022:[\u0022c\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022bc\u0022,\u0022indices\u0022:[[1,2]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=(?<=a|b))c\u0022}"
        "(?<=(a|b)(?<!(c|d)))e" flags="d" input="be" "{\u0022matches\u0022:[\u0022e\u0022,\u0022b\u0022,null],\u0022index\u0022:1,\u0022input\u0022:\u0022be\u0022,\u0022indices\u0022:[[1,2],[0,1],null],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=(a|b)(?<!(c|d)))e\u0022}"
        "(?:(?<=(a|b))c|d)+e" flags="d" input="abcdde" "{\u0022matches\u0022:[\u0022cdde\u0022,null],\u0022index\u0022:2,\u0022input\u0022:\u0022abcdde\u0022,\u0022indices\u0022:[[2,6],null],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?:(?<=(a|b))c|d)+e\u0022}"
        "((?<=(a|b))){2}c\u005c1\u005c2" flags="d" input="bcb" "{\u0022matches\u0022:[\u0022cb\u0022,\u0022\u0022,\u0022b\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022bcb\u0022,\u0022indices\u0022:[[1,3],[1,1],[0,1]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022((?<=(a|b))){2}c\u005c\u005c1\u005c\u005c2\u0022}"
        "((?<=(a|b)))*c\u005c1\u005c2" flags="d" input="bc" "{\u0022matches\u0022:[\u0022c\u0022,null,null],\u0022index\u0022:1,\u0022input\u0022:\u0022bc\u0022,\u0022indices\u0022:[[1,2],null,null],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022((?<=(a|b)))*c\u005c\u005c1\u005c\u005c2\u0022}"
        "(?=(ab(?<=(a|b)(a|b))c))\u005c1\u005c2\u005c3" flags="d" input="abcab" "{\u0022matches\u0022:[\u0022abcab\u0022,\u0022abc\u0022,\u0022a\u0022,\u0022b\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022abcab\u0022,\u0022indices\u0022:[[0,5],[0,3],[0,1],[1,2]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?=(ab(?<=(a|b)(a|b))c))\u005c\u005c1\u005c\u005c2\u005c\u005c3\u0022}"
        "(?<=(\u00b5|\u039c))a" flags="di" input="\u039ca" "{\u0022matches\u0022:[\u0022a\u0022,\u0022\u039c\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022\u039ca\u0022,\u0022indices\u0022:[[1,2],[0,1]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=(\u00b5|\u039c))a\u0022}"
        "(?<=(?<x>a)|(?<x>b))c" flags="d" input="bc" "{\u0022matches\u0022:[\u0022c\u0022,null,\u0022b\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022bc\u0022,\u0022groups\u0022:{\u0022x\u0022:\u0022b\u0022},\u0022indices\u0022:[[1,2],null,[0,1]],\u0022indicesGroups\u0022:{\u0022x\u0022:[0,1]},\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=(?<x>a)|(?<x>b))c\u0022}"
        "(?<=(?<x>a)b|a(?<x>c))d" flags="d" input="acd" "{\u0022matches\u0022:[\u0022d\u0022,null,\u0022c\u0022],\u0022index\u0022:2,\u0022input\u0022:\u0022acd\u0022,\u0022groups\u0022:{\u0022x\u0022:\u0022c\u0022},\u0022indices\u0022:[[2,3],null,[1,2]],\u0022indicesGroups\u0022:{\u0022x\u0022:[1,2]},\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=(?<x>a)b|a(?<x>c))d\u0022}"
        "(?<=[\u005cuD800]|[\u005cuDC00])c" flags="d" input="\udc00c" "{\u0022matches\u0022:[\u0022c\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022\u005cudc00c\u0022,\u0022indices\u0022:[[1,2]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=[\u005c\u005cuD800]|[\u005c\u005cuDC00])c\u0022}"
        "#);
}

#[test]
fn fixed_lookbehind_choices_partial_capture_undo_named_aliases_and_atomicity_are_exact() {
    assert_true(
        r"let a=/(?<=(?<x>a)b|a(?<x>c))d\k<x>/d.exec('acdc');a[0]==='dc'&&a.index===2&&a[1]===undefined&&a[2]==='c'&&a.groups.x==='c'&&a.indices[1]===undefined&&a.indices.groups.x===a.indices[2]&&a.indices[2][0]===1&&a.indices[2][1]===2",
    );
    assert_true(
        r"let a=/(?<=((a)b|a(c)))d/d.exec('acd');a[1]==='ac'&&a[2]===undefined&&a[3]==='c'&&a.indices[1][0]===0&&a.indices[1][1]===2&&a.indices[2]===undefined&&a.indices[3][0]===1",
    );
    assert_true(
        r"let a=/(?<!((a)b|a(c)))d/d.exec('aqd');a.index===2&&a[1]===undefined&&a[2]===undefined&&a[3]===undefined&&a.indices[1]===undefined&&a.indices[2]===undefined&&a.indices[3]===undefined",
    );
    assert_true(r"let r=/(?<=((a)|a))c\2/dy;r.lastIndex=1;r.exec('ac')===null&&r.lastIndex===0");
    assert_true(
        r"let a=/(?<=(a|(a)))c\2/d.exec('ac');a[0]==='c'&&a[1]==='a'&&a[2]===undefined&&a.indices[2]===undefined",
    );
    assert_true(
        r"let a=/(?<=()|())a/d.exec('a');a[1]===''&&a[2]===undefined&&a.indices[1][0]===0&&a.indices[1][1]===0&&a.indices[2]===undefined",
    );
    assert_true(
        r"let a=/(?<=(?<x>a)|(?<x>b))c/d.exec('bc');a[1]===undefined&&a[2]==='b'&&a.groups.x==='b'&&a.indices.groups.x===a.indices[2]",
    );
    assert_true(
        r"let a=/(?<=([ab]){2}|(a){2})c/d.exec('abc');a[1]==='a'&&a[2]===undefined&&a.indices[1][0]===0&&a.indices[2]===undefined",
    );
    assert_true(
        r"let a=/((?<=(a|b))){2}c\1\2/d.exec('bcb'),b=/((?<=(a|b)))*c\1\2/d.exec('bc');a[1]===''&&a[2]==='b'&&a.indices[2][0]===0&&b[1]===undefined&&b[2]===undefined",
    );
}

#[test]
fn fixed_lookbehind_choice_consumers_global_sticky_empty_and_callbacks_preserve_ranges() {
    assert_true(
        r"let r=/(?<=(?<x>a)|(?<x>b))c/dg,a=[...'acqbc'.matchAll(r)];a.length===2&&a[0].index===1&&a[0].groups.x==='a'&&a[1].index===4&&a[1].groups.x==='b'&&a[1].indices.groups.x[0]===3&&r.lastIndex===0",
    );
    assert_true(
        r"let r=/(?<=(a|b))c/dy;r.lastIndex=1;let a=r.exec('bc');a[1]==='b'&&a.index===1&&r.lastIndex===2&&r.exec('bc')===null&&r.lastIndex===0",
    );
    assert_true(
        r"let r=/(?<=(a|b))/dg,a=[...'ab'.matchAll(r)];a.length===2&&a[0].index===1&&a[0][1]==='a'&&a[1].index===2&&a[1][1]==='b'&&r.lastIndex===0",
    );
    assert_true(
        r"let seen=[];let s='acqbc'.replace(/(?<=(?<x>a)|(?<x>b))c/g,(m,a,b,i,s,g)=>{seen.push(m,a,b,i,s,g.x);return '_'});s==='a_qb_'&&seen.length===12&&seen[1]==='a'&&seen[2]===undefined&&seen[3]===1&&seen[5]==='a'&&seen[7]===undefined&&seen[8]==='b'&&seen[9]===4&&seen[11]==='b'",
    );
    assert_true(
        r"'acqbc'.replace(/(?<=(?<x>a)|(?<x>b))c/g,'<$<x>>')==='a<a>qb<b>'&&'acqbc'.search(/(?<=(a|b))c/)===1&&'acqbc'.split(/(?<=(a|b))c/).join('|')==='a|a|qb|b|'",
    );
}

#[test]
fn explicit_fixed_lookbehind_branch_work_aborts_before_last_index_and_handlers() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval("let marker=0,r=/(?<=(a|b))c/g;r.lastIndex=1;let text='ab'.repeat(5000)")
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
fn differing_widths_repeated_choices_internal_references_and_lookahead_match_reference_results() {
    for source in [
        r"/(?<=a|bb)c/.exec('bbc')",
        r"/(?<=(a|bb))c/.exec('bbc')",
        r"/(?<=(a|b){2})c/.exec('abc')",
        r"/(?<=(a|b)\1)c/.exec('aac')",
        r"/(?<=a|(?=a+)b)c/.exec('bc')",
        r"/(?<=(a|b))c/u.exec('bc')",
        r"/(?<=(a|b))c/v.exec('bc')",
    ] {
        crate::common::regexp_oracle::assert_matches_reference(&mut Realm::default(), source);
    }
}
