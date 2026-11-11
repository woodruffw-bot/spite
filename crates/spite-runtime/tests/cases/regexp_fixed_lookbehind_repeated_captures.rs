//! Leftmost iteration captures for exact-count lookbehind (22.2.2.3.1, 22.2.2.8).
use spite_core::JsString;
use spite_runtime::{Error, Limits, Realm, Value};
use std::fmt::Write;

use crate::common::assert_true;

#[test]
fn fixed_lookbehind_repeated_capture_result_snapshot() {
    let mut rows = String::new();
    for (source, flags, text) in [
        (r"(?<=([ab]){2})c", "d", "abc"),
        (r"(?<!([ab]){2})c", "d", "abc"),
        (r"(?<!([ab]){2})c", "d", "qbc"),
        (r"(?<=(a){0})b", "d", "b"),
        (r"(?<!(a){0})b", "d", "b"),
        (r"(?<=((a)b){2})c", "d", "ababc"),
        (r"(?<=(a(b)){2})c", "d", "ababc"),
        (r"(?<=((a){2}))b", "d", "aab"),
        (r"(?<=(a()){2})b", "d", "aab"),
        (r"(?<=(a){2}?)b", "d", "aab"),
        (r"(?<=(a){2,2})b", "d", "aab"),
        (r"(?<=(a){0}())b", "d", "b"),
        (r"(?<=(a)(b){2})c", "d", "abbc"),
        (r"(?<=([ab]){2})\1", "d", "aba"),
        (r"(?<=((a)b){2})\1\2", "d", "abababa"),
        (r"(?<!((a)b){2}q)c", "d", "ababrc"),
        (r"(?<=([ab]){2}(?<=([ab]){2}))c", "d", "abc"),
        (r"(?<=([ab]){2}(?<!(c){2}))c", "d", "abc"),
        (r"(?<=^([ab]){2})c", "dm", "q\nabc"),
        (r"(?<=(µ){2})Μ", "di", "ΜµΜ"),
        (r"(?<=(.){2})b", "ds", "\n\rb"),
        (r"(?<=(.){2})b", "d", "\n\rb"),
        (r"(?:(?<=([ab]){2})c|d)+e", "d", "abcdde"),
        (r"((?<=([ab]){2})){2}c\1\2", "d", "abca"),
        (r"((?<=([ab]){2}))*c\1\2", "d", "abc"),
        (r"(?=(ab(?<=([ab]){2})c))\1\2", "d", "abca"),
        (r"(?<=(?<x>[ab]){2})c", "d", "abc"),
        (r"(?<=([\uD800\uDC00]){2})c", "d", "lone surrogates"),
    ] {
        let source = JsString::from(source);
        let flags = JsString::from(flags);
        let input = if text == "lone surrogates" {
            JsString::from_code_units(vec![0xd800, 0xdc00, 0x63])
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
        "(?<=([ab]){2})c" flags="d" input="abc" "{\u0022matches\u0022:[\u0022c\u0022,\u0022a\u0022],\u0022index\u0022:2,\u0022input\u0022:\u0022abc\u0022,\u0022indices\u0022:[[2,3],[0,1]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=([ab]){2})c\u0022}"
        "(?<!([ab]){2})c" flags="d" input="abc" "{\u0022match\u0022:null,\u0022lastIndex\u0022:0}"
        "(?<!([ab]){2})c" flags="d" input="qbc" "{\u0022matches\u0022:[\u0022c\u0022,null],\u0022index\u0022:2,\u0022input\u0022:\u0022qbc\u0022,\u0022indices\u0022:[[2,3],null],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<!([ab]){2})c\u0022}"
        "(?<=(a){0})b" flags="d" input="b" "{\u0022matches\u0022:[\u0022b\u0022,null],\u0022index\u0022:0,\u0022input\u0022:\u0022b\u0022,\u0022indices\u0022:[[0,1],null],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=(a){0})b\u0022}"
        "(?<!(a){0})b" flags="d" input="b" "{\u0022match\u0022:null,\u0022lastIndex\u0022:0}"
        "(?<=((a)b){2})c" flags="d" input="ababc" "{\u0022matches\u0022:[\u0022c\u0022,\u0022ab\u0022,\u0022a\u0022],\u0022index\u0022:4,\u0022input\u0022:\u0022ababc\u0022,\u0022indices\u0022:[[4,5],[0,2],[0,1]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=((a)b){2})c\u0022}"
        "(?<=(a(b)){2})c" flags="d" input="ababc" "{\u0022matches\u0022:[\u0022c\u0022,\u0022ab\u0022,\u0022b\u0022],\u0022index\u0022:4,\u0022input\u0022:\u0022ababc\u0022,\u0022indices\u0022:[[4,5],[0,2],[1,2]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=(a(b)){2})c\u0022}"
        "(?<=((a){2}))b" flags="d" input="aab" "{\u0022matches\u0022:[\u0022b\u0022,\u0022aa\u0022,\u0022a\u0022],\u0022index\u0022:2,\u0022input\u0022:\u0022aab\u0022,\u0022indices\u0022:[[2,3],[0,2],[0,1]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=((a){2}))b\u0022}"
        "(?<=(a()){2})b" flags="d" input="aab" "{\u0022matches\u0022:[\u0022b\u0022,\u0022a\u0022,\u0022\u0022],\u0022index\u0022:2,\u0022input\u0022:\u0022aab\u0022,\u0022indices\u0022:[[2,3],[0,1],[1,1]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=(a()){2})b\u0022}"
        "(?<=(a){2}?)b" flags="d" input="aab" "{\u0022matches\u0022:[\u0022b\u0022,\u0022a\u0022],\u0022index\u0022:2,\u0022input\u0022:\u0022aab\u0022,\u0022indices\u0022:[[2,3],[0,1]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=(a){2}?)b\u0022}"
        "(?<=(a){2,2})b" flags="d" input="aab" "{\u0022matches\u0022:[\u0022b\u0022,\u0022a\u0022],\u0022index\u0022:2,\u0022input\u0022:\u0022aab\u0022,\u0022indices\u0022:[[2,3],[0,1]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=(a){2,2})b\u0022}"
        "(?<=(a){0}())b" flags="d" input="b" "{\u0022matches\u0022:[\u0022b\u0022,null,\u0022\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022b\u0022,\u0022indices\u0022:[[0,1],null,[0,0]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=(a){0}())b\u0022}"
        "(?<=(a)(b){2})c" flags="d" input="abbc" "{\u0022matches\u0022:[\u0022c\u0022,\u0022a\u0022,\u0022b\u0022],\u0022index\u0022:3,\u0022input\u0022:\u0022abbc\u0022,\u0022indices\u0022:[[3,4],[0,1],[1,2]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=(a)(b){2})c\u0022}"
        "(?<=([ab]){2})\u005c1" flags="d" input="aba" "{\u0022matches\u0022:[\u0022a\u0022,\u0022a\u0022],\u0022index\u0022:2,\u0022input\u0022:\u0022aba\u0022,\u0022indices\u0022:[[2,3],[0,1]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=([ab]){2})\u005c\u005c1\u0022}"
        "(?<=((a)b){2})\u005c1\u005c2" flags="d" input="abababa" "{\u0022matches\u0022:[\u0022aba\u0022,\u0022ab\u0022,\u0022a\u0022],\u0022index\u0022:4,\u0022input\u0022:\u0022abababa\u0022,\u0022indices\u0022:[[4,7],[0,2],[0,1]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=((a)b){2})\u005c\u005c1\u005c\u005c2\u0022}"
        "(?<!((a)b){2}q)c" flags="d" input="ababrc" "{\u0022matches\u0022:[\u0022c\u0022,null,null],\u0022index\u0022:5,\u0022input\u0022:\u0022ababrc\u0022,\u0022indices\u0022:[[5,6],null,null],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<!((a)b){2}q)c\u0022}"
        "(?<=([ab]){2}(?<=([ab]){2}))c" flags="d" input="abc" "{\u0022matches\u0022:[\u0022c\u0022,\u0022a\u0022,\u0022a\u0022],\u0022index\u0022:2,\u0022input\u0022:\u0022abc\u0022,\u0022indices\u0022:[[2,3],[0,1],[0,1]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=([ab]){2}(?<=([ab]){2}))c\u0022}"
        "(?<=([ab]){2}(?<!(c){2}))c" flags="d" input="abc" "{\u0022matches\u0022:[\u0022c\u0022,\u0022a\u0022,null],\u0022index\u0022:2,\u0022input\u0022:\u0022abc\u0022,\u0022indices\u0022:[[2,3],[0,1],null],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=([ab]){2}(?<!(c){2}))c\u0022}"
        "(?<=^([ab]){2})c" flags="dm" input="q\u000aabc" "{\u0022matches\u0022:[\u0022c\u0022,\u0022a\u0022],\u0022index\u0022:4,\u0022input\u0022:\u0022q\u005cnabc\u0022,\u0022indices\u0022:[[4,5],[2,3]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=^([ab]){2})c\u0022}"
        "(?<=(\u00b5){2})\u039c" flags="di" input="\u039c\u00b5\u039c" "{\u0022matches\u0022:[\u0022\u039c\u0022,\u0022\u039c\u0022],\u0022index\u0022:2,\u0022input\u0022:\u0022\u039c\u00b5\u039c\u0022,\u0022indices\u0022:[[2,3],[0,1]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=(\u00b5){2})\u039c\u0022}"
        "(?<=(.){2})b" flags="ds" input="\u000a\u000db" "{\u0022matches\u0022:[\u0022b\u0022,\u0022\u005cn\u0022],\u0022index\u0022:2,\u0022input\u0022:\u0022\u005cn\u005crb\u0022,\u0022indices\u0022:[[2,3],[0,1]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=(.){2})b\u0022}"
        "(?<=(.){2})b" flags="d" input="\u000a\u000db" "{\u0022match\u0022:null,\u0022lastIndex\u0022:0}"
        "(?:(?<=([ab]){2})c|d)+e" flags="d" input="abcdde" "{\u0022matches\u0022:[\u0022cdde\u0022,null],\u0022index\u0022:2,\u0022input\u0022:\u0022abcdde\u0022,\u0022indices\u0022:[[2,6],null],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?:(?<=([ab]){2})c|d)+e\u0022}"
        "((?<=([ab]){2})){2}c\u005c1\u005c2" flags="d" input="abca" "{\u0022matches\u0022:[\u0022ca\u0022,\u0022\u0022,\u0022a\u0022],\u0022index\u0022:2,\u0022input\u0022:\u0022abca\u0022,\u0022indices\u0022:[[2,4],[2,2],[0,1]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022((?<=([ab]){2})){2}c\u005c\u005c1\u005c\u005c2\u0022}"
        "((?<=([ab]){2}))*c\u005c1\u005c2" flags="d" input="abc" "{\u0022matches\u0022:[\u0022c\u0022,null,null],\u0022index\u0022:2,\u0022input\u0022:\u0022abc\u0022,\u0022indices\u0022:[[2,3],null,null],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022((?<=([ab]){2}))*c\u005c\u005c1\u005c\u005c2\u0022}"
        "(?=(ab(?<=([ab]){2})c))\u005c1\u005c2" flags="d" input="abca" "{\u0022matches\u0022:[\u0022abca\u0022,\u0022abc\u0022,\u0022a\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022abca\u0022,\u0022indices\u0022:[[0,4],[0,3],[0,1]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?=(ab(?<=([ab]){2})c))\u005c\u005c1\u005c\u005c2\u0022}"
        "(?<=(?<x>[ab]){2})c" flags="d" input="abc" "{\u0022matches\u0022:[\u0022c\u0022,\u0022a\u0022],\u0022index\u0022:2,\u0022input\u0022:\u0022abc\u0022,\u0022groups\u0022:{\u0022x\u0022:\u0022a\u0022},\u0022indices\u0022:[[2,3],[0,1]],\u0022indicesGroups\u0022:{\u0022x\u0022:[0,1]},\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=(?<x>[ab]){2})c\u0022}"
        "(?<=([\u005cuD800\u005cuDC00]){2})c" flags="d" input="\ud800\udc00c" "{\u0022matches\u0022:[\u0022c\u0022,\u0022\u005cud800\u0022],\u0022index\u0022:2,\u0022input\u0022:\u0022\ud800\udc00c\u0022,\u0022indices\u0022:[[2,3],[0,1]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=([\u005c\u005cuD800\u005c\u005cuDC00]){2})c\u0022}"
        "#);
}

#[test]
fn counted_lookbehind_capture_ranges_named_aliases_zero_counts_and_outer_retry_are_backward() {
    assert_true(
        r"let a=/(?<=(?<x>[ab]){2})\k<x>/d.exec('aba');a[0]==='a'&&a.index===2&&a.groups.x==='a'&&a.indices.groups.x===a.indices[1]&&a.indices[1][0]===0&&a.indices[1][1]===1",
    );
    assert_true(
        r"let a=/(?<=((a)b){2})c/d.exec('ababc');a[1]==='ab'&&a[2]==='a'&&a.indices[1][0]===0&&a.indices[1][1]===2&&a.indices[2][0]===0&&a.indices[2][1]===1",
    );
    assert_true(
        r"let a=/(?<=(a()){2})b/d.exec('aab');a[1]==='a'&&a[2]===''&&a.indices[2][0]===1&&a.indices[2][1]===1",
    );
    assert_true(
        r"let a=/(?<=(?<x>a){0}())b/d.exec('b');a[1]===undefined&&a.groups.x===undefined&&a.indices.groups.x===undefined&&a[2]===''&&a.indices[2][0]===0&&a.indices[2][1]===0",
    );
    assert_true(
        r"let a=/(?<!((?<x>a)b){2}q)c/d.exec('ababrc');a.index===5&&a[1]===undefined&&a[2]===undefined&&a.groups.x===undefined&&a.indices[1]===undefined&&a.indices.groups.x===undefined",
    );
    assert_true(
        r"let a=/(?:(?<=(?<x>[ab]){2})q|(?<=(?<x>[ab]){2})c)\k<x>/d.exec('abca');a.index===2&&a[0]==='ca'&&a[1]===undefined&&a[2]==='a'&&a.indices.groups.x===a.indices[2]&&a.indices[2][0]===0",
    );
    assert_true(
        r"let a=/(?<=([ab]){2}(?<!(?<y>c){2}))c/d.exec('abc');a[1]==='a'&&a.groups.y===undefined&&a.indices[1][0]===0&&a.indices[1][1]===1&&a.indices.groups.y===undefined",
    );
    assert_true(
        r"let a=/((?<=([ab]){2})){2}c\1\2/d.exec('abca'),b=/((?<=([ab]){2}))*c\1\2/d.exec('abc');a[1]===''&&a[2]==='a'&&a.indices[2][0]===0&&b[1]===undefined&&b[2]===undefined",
    );
}

#[test]
fn counted_lookbehind_consumers_global_sticky_and_callbacks_use_leftmost_captures() {
    assert_true(
        r"let r=/(?<=(?<x>[ab]){2})c/dg,a=[...'abcqbac'.matchAll(r)];a.length===2&&a[0].index===2&&a[0].groups.x==='a'&&a[1].index===6&&a[1].groups.x==='b'&&a[1].indices.groups.x[0]===4&&r.lastIndex===0",
    );
    assert_true(
        r"let r=/(?<=([ab]){2})c/dy;r.lastIndex=2;let a=r.exec('abc');a[1]==='a'&&a.index===2&&r.lastIndex===3&&r.exec('abc')===null&&r.lastIndex===0",
    );
    assert_true(
        r"let r=/(?<=([ab]){2})/dg,a=[...'aba'.matchAll(r)];a.length===2&&a[0].index===2&&a[0][1]==='a'&&a[1].index===3&&a[1][1]==='b'&&a[1].indices[1][0]===1&&r.lastIndex===0",
    );
    assert_true(
        r"let seen=[];let s='abcqbac'.replace(/(?<=(?<x>[ab]){2})c/g,(m,c,i,s,g)=>{seen.push(m,c,i,s,g.x);return '_'});s==='ab_qba_'&&seen.length===10&&seen[0]==='c'&&seen[1]==='a'&&seen[2]===2&&seen[4]==='a'&&seen[6]==='b'&&seen[7]===6&&seen[9]==='b'",
    );
    assert_true(
        r"'abcqbac'.replace(/(?<=(?<x>[ab]){2})c/g,'<$<x>>')==='ab<a>qba<b>'&&'abcqbac'.search(/(?<=([ab]){2})c/)===2&&'abcqbac'.split(/(?<=([ab]){2})c/).join('|')==='ab|a|qba|b|'",
    );
}

#[test]
fn explicit_counted_lookbehind_capture_work_aborts_before_last_index_and_handlers() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval("let marker=0,r=/(?<=([ab]){8})c/g;r.lastIndex=1;let text='ab'.repeat(5000)")
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
fn variable_choice_reference_assertion_empty_and_nested_count_bodies_match_reference_results() {
    for source in [
        r"/(?<=(a){1,2})b/.exec('aab')",
        r"/(?<=(a|b){2})c/.exec('abc')",
        r"/(?<=(a(?=a)){2})c/.exec('abc')",
        r"/(?<=((a){2}){2})c/.exec('aaaac')",
        r"/(?<=(a\1){1,2})c/.exec('aac')",
        r"/(?<=((?=a+)){2})c/.exec('c')",
        r"/(?<=([ab]){2})c/u.exec('abc')",
        r"/(?<=([ab]){2})c/v.exec('abc')",
    ] {
        crate::common::regexp_oracle::assert_matches_reference(&mut Realm::default(), source);
    }
}
