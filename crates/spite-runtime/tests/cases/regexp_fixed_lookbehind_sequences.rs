//! Exact-count ordinary character sequences in fixed lookbehind (22.2.2.8).
use spite_core::JsString;
use spite_runtime::{Error, Limits, Realm, Value};
use std::fmt::Write;

use crate::common::assert_true;

#[test]
fn fixed_lookbehind_sequence_result_snapshot() {
    let mut rows = String::new();
    for (source, flags, text) in [
        (r"(?<=(?:ab){2})c", "d", "ababc"),
        (r"(?<!(?:ab){2})c", "d", "ababc"),
        (r"(?<!(?:ab){2})c", "d", "abc"),
        (r"(?<=(?:ab){0})c", "d", "c"),
        (r"(?<!(?:ab){0})c", "d", "c"),
        (r"(?<=(?:ab){2,2})c", "d", "ababc"),
        (r"(?<=(?:ab){2}?)c", "d", "ababc"),
        (r"(?<=((?:ab){2}))c", "d", "ababc"),
        (r"(?<=((?:ab){2}))\1", "d", "abababab"),
        (r"(?<!((?:ab){2})q)c", "d", "ababrc"),
        (r"(?<=((?:[a-c]\d){2}))c", "d", "a1a2c"),
        (r"(?<=(?:a.){2})b", "ds", "a\na\nb"),
        (r"(?<=(?:a.){2})b", "d", "a\na\nb"),
        (r"(?<=(?:µΜ){2})a", "di", "ΜµµΜa"),
        (r"(?<=^(?:ab){2})c", "dm", "q\nababc"),
        (r"(?<=(?:ab){2}\b)c", "d", "ababc"),
        (r"(?<=(?:ab){2}\B)c", "d", "ababc"),
        (r"(?<=a(?:ba){2})b", "d", "ababab"),
        (r"(?<=(?:ab){2}(?<=ab))c", "d", "ababc"),
        (r"(?<=(?:ab){2}(?<!ba))c", "d", "ababc"),
        (r"(?<=((?:ab){2})(?<!(ba)))c", "d", "ababc"),
        (r"(?:(?<=(?:ab){2})c|d)+e", "d", "ababcdde"),
        (r"((?<=(?:ab){2})){2}c\1", "d", "ababc"),
        (r"((?<=(?:ab){2}))*c\1", "d", "ababc"),
        (r"(?=(a(?:ba){2}(?<=((?:ba){2}))))\1\2", "d", "ababababa"),
        (r"(?<=(?:a(?:b)){2})c", "d", "ababc"),
        (r"(?<=((?:ab){0}))c", "d", "c"),
        (r"(?<=(?:[\uD800]b){2})c", "d", "lone surrogates"),
    ] {
        let source = JsString::from(source);
        let flags = JsString::from(flags);
        let input = if text == "lone surrogates" {
            JsString::from_code_units(vec![0xd800, 0x62, 0xd800, 0x62, 0x63])
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
        "(?<=(?:ab){2})c" flags="d" input="ababc" "{\u0022matches\u0022:[\u0022c\u0022],\u0022index\u0022:4,\u0022input\u0022:\u0022ababc\u0022,\u0022indices\u0022:[[4,5]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=(?:ab){2})c\u0022}"
        "(?<!(?:ab){2})c" flags="d" input="ababc" "{\u0022match\u0022:null,\u0022lastIndex\u0022:0}"
        "(?<!(?:ab){2})c" flags="d" input="abc" "{\u0022matches\u0022:[\u0022c\u0022],\u0022index\u0022:2,\u0022input\u0022:\u0022abc\u0022,\u0022indices\u0022:[[2,3]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<!(?:ab){2})c\u0022}"
        "(?<=(?:ab){0})c" flags="d" input="c" "{\u0022matches\u0022:[\u0022c\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022c\u0022,\u0022indices\u0022:[[0,1]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=(?:ab){0})c\u0022}"
        "(?<!(?:ab){0})c" flags="d" input="c" "{\u0022match\u0022:null,\u0022lastIndex\u0022:0}"
        "(?<=(?:ab){2,2})c" flags="d" input="ababc" "{\u0022matches\u0022:[\u0022c\u0022],\u0022index\u0022:4,\u0022input\u0022:\u0022ababc\u0022,\u0022indices\u0022:[[4,5]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=(?:ab){2,2})c\u0022}"
        "(?<=(?:ab){2}?)c" flags="d" input="ababc" "{\u0022matches\u0022:[\u0022c\u0022],\u0022index\u0022:4,\u0022input\u0022:\u0022ababc\u0022,\u0022indices\u0022:[[4,5]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=(?:ab){2}?)c\u0022}"
        "(?<=((?:ab){2}))c" flags="d" input="ababc" "{\u0022matches\u0022:[\u0022c\u0022,\u0022abab\u0022],\u0022index\u0022:4,\u0022input\u0022:\u0022ababc\u0022,\u0022indices\u0022:[[4,5],[0,4]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=((?:ab){2}))c\u0022}"
        "(?<=((?:ab){2}))\u005c1" flags="d" input="abababab" "{\u0022matches\u0022:[\u0022abab\u0022,\u0022abab\u0022],\u0022index\u0022:4,\u0022input\u0022:\u0022abababab\u0022,\u0022indices\u0022:[[4,8],[0,4]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=((?:ab){2}))\u005c\u005c1\u0022}"
        "(?<!((?:ab){2})q)c" flags="d" input="ababrc" "{\u0022matches\u0022:[\u0022c\u0022,null],\u0022index\u0022:5,\u0022input\u0022:\u0022ababrc\u0022,\u0022indices\u0022:[[5,6],null],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<!((?:ab){2})q)c\u0022}"
        "(?<=((?:[a-c]\u005cd){2}))c" flags="d" input="a1a2c" "{\u0022matches\u0022:[\u0022c\u0022,\u0022a1a2\u0022],\u0022index\u0022:4,\u0022input\u0022:\u0022a1a2c\u0022,\u0022indices\u0022:[[4,5],[0,4]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=((?:[a-c]\u005c\u005cd){2}))c\u0022}"
        "(?<=(?:a.){2})b" flags="ds" input="a\u000aa\u000ab" "{\u0022matches\u0022:[\u0022b\u0022],\u0022index\u0022:4,\u0022input\u0022:\u0022a\u005cna\u005cnb\u0022,\u0022indices\u0022:[[4,5]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=(?:a.){2})b\u0022}"
        "(?<=(?:a.){2})b" flags="d" input="a\u000aa\u000ab" "{\u0022match\u0022:null,\u0022lastIndex\u0022:0}"
        "(?<=(?:\u00b5\u039c){2})a" flags="di" input="\u039c\u00b5\u00b5\u039ca" "{\u0022matches\u0022:[\u0022a\u0022],\u0022index\u0022:4,\u0022input\u0022:\u0022\u039c\u00b5\u00b5\u039ca\u0022,\u0022indices\u0022:[[4,5]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=(?:\u00b5\u039c){2})a\u0022}"
        "(?<=^(?:ab){2})c" flags="dm" input="q\u000aababc" "{\u0022matches\u0022:[\u0022c\u0022],\u0022index\u0022:6,\u0022input\u0022:\u0022q\u005cnababc\u0022,\u0022indices\u0022:[[6,7]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=^(?:ab){2})c\u0022}"
        "(?<=(?:ab){2}\u005cb)c" flags="d" input="ababc" "{\u0022match\u0022:null,\u0022lastIndex\u0022:0}"
        "(?<=(?:ab){2}\u005cB)c" flags="d" input="ababc" "{\u0022matches\u0022:[\u0022c\u0022],\u0022index\u0022:4,\u0022input\u0022:\u0022ababc\u0022,\u0022indices\u0022:[[4,5]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=(?:ab){2}\u005c\u005cB)c\u0022}"
        "(?<=a(?:ba){2})b" flags="d" input="ababab" "{\u0022matches\u0022:[\u0022b\u0022],\u0022index\u0022:5,\u0022input\u0022:\u0022ababab\u0022,\u0022indices\u0022:[[5,6]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=a(?:ba){2})b\u0022}"
        "(?<=(?:ab){2}(?<=ab))c" flags="d" input="ababc" "{\u0022matches\u0022:[\u0022c\u0022],\u0022index\u0022:4,\u0022input\u0022:\u0022ababc\u0022,\u0022indices\u0022:[[4,5]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=(?:ab){2}(?<=ab))c\u0022}"
        "(?<=(?:ab){2}(?<!ba))c" flags="d" input="ababc" "{\u0022matches\u0022:[\u0022c\u0022],\u0022index\u0022:4,\u0022input\u0022:\u0022ababc\u0022,\u0022indices\u0022:[[4,5]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=(?:ab){2}(?<!ba))c\u0022}"
        "(?<=((?:ab){2})(?<!(ba)))c" flags="d" input="ababc" "{\u0022matches\u0022:[\u0022c\u0022,\u0022abab\u0022,null],\u0022index\u0022:4,\u0022input\u0022:\u0022ababc\u0022,\u0022indices\u0022:[[4,5],[0,4],null],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=((?:ab){2})(?<!(ba)))c\u0022}"
        "(?:(?<=(?:ab){2})c|d)+e" flags="d" input="ababcdde" "{\u0022matches\u0022:[\u0022cdde\u0022],\u0022index\u0022:4,\u0022input\u0022:\u0022ababcdde\u0022,\u0022indices\u0022:[[4,8]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?:(?<=(?:ab){2})c|d)+e\u0022}"
        "((?<=(?:ab){2})){2}c\u005c1" flags="d" input="ababc" "{\u0022matches\u0022:[\u0022c\u0022,\u0022\u0022],\u0022index\u0022:4,\u0022input\u0022:\u0022ababc\u0022,\u0022indices\u0022:[[4,5],[4,4]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022((?<=(?:ab){2})){2}c\u005c\u005c1\u0022}"
        "((?<=(?:ab){2}))*c\u005c1" flags="d" input="ababc" "{\u0022matches\u0022:[\u0022c\u0022,null],\u0022index\u0022:4,\u0022input\u0022:\u0022ababc\u0022,\u0022indices\u0022:[[4,5],null],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022((?<=(?:ab){2}))*c\u005c\u005c1\u0022}"
        "(?=(a(?:ba){2}(?<=((?:ba){2}))))\u005c1\u005c2" flags="d" input="ababababa" "{\u0022matches\u0022:[\u0022ababababa\u0022,\u0022ababa\u0022,\u0022baba\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022ababababa\u0022,\u0022indices\u0022:[[0,9],[0,5],[1,5]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?=(a(?:ba){2}(?<=((?:ba){2}))))\u005c\u005c1\u005c\u005c2\u0022}"
        "(?<=(?:a(?:b)){2})c" flags="d" input="ababc" "{\u0022matches\u0022:[\u0022c\u0022],\u0022index\u0022:4,\u0022input\u0022:\u0022ababc\u0022,\u0022indices\u0022:[[4,5]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=(?:a(?:b)){2})c\u0022}"
        "(?<=((?:ab){0}))c" flags="d" input="c" "{\u0022matches\u0022:[\u0022c\u0022,\u0022\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022c\u0022,\u0022indices\u0022:[[0,1],[0,0]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=((?:ab){0}))c\u0022}"
        "(?<=(?:[\u005cuD800]b){2})c" flags="d" input="\ud800b\ud800bc" "{\u0022matches\u0022:[\u0022c\u0022],\u0022index\u0022:4,\u0022input\u0022:\u0022\u005cud800b\u005cud800bc\u0022,\u0022indices\u0022:[[4,5]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=(?:[\u005c\u005cuD800]b){2})c\u0022}"
        "#);
}

#[test]
fn sequence_lookbehind_captures_negative_restore_and_outer_retry_preserve_ranges() {
    assert_true(
        r"let a=/(?<=(?<x>(?:ab){2}))\k<x>/d.exec('abababab');a[0]==='abab'&&a.index===4&&a.groups.x==='abab'&&a.indices.groups.x===a.indices[1]&&a.indices[1][0]===0&&a.indices[1][1]===4",
    );
    assert_true(
        r"let a=/(?<!((?:ab){2})q)c/d.exec('ababrc');a.index===5&&a[1]===undefined&&a.indices[1]===undefined",
    );
    assert_true(
        r"let a=/(?<=((?:ab){2})(?<!(?<y>ba)))c/d.exec('ababc');a[1]==='abab'&&a.groups.y===undefined&&a.indices[1][0]===0&&a.indices[1][1]===4&&a.indices.groups.y===undefined",
    );
    assert_true(
        r"let a=/(?:(?<=(?<x>(?:ab){2}))q|(?<=(?<x>(?:ab){2}))c)\k<x>/d.exec('ababcabab');a[0]==='cabab'&&a.index===4&&a[1]===undefined&&a[2]==='abab'&&a.indices[1]===undefined&&a.indices.groups.x===a.indices[2]",
    );
    assert_true(
        r"let a=/((?<=(?:ab){2})){2}c\1/d.exec('ababc'),b=/((?<=(?:ab){2}))*c\1/d.exec('ababc');a[1]===''&&a.indices[1][0]===4&&a.indices[1][1]===4&&b[1]===undefined&&b.indices[1]===undefined",
    );
    assert_true(
        r"let a=/(?<=((?:ab){0}))c/d.exec('c');a.index===0&&a[1]===''&&a.indices[1][0]===0&&a.indices[1][1]===0",
    );
}

#[test]
fn sequence_lookbehind_consumers_global_sticky_and_callbacks_keep_input_positions() {
    assert_true(
        r"let r=/(?<=(?<x>(?:ab){2}))c/dg,a=[...'ababcqababc'.matchAll(r)];a.length===2&&a[0].index===4&&a[1].index===10&&a[1].groups.x==='abab'&&a[1].indices.groups.x[0]===6&&r.lastIndex===0",
    );
    assert_true(
        r"let r=/(?<=(?:ab){2})c/dy;r.lastIndex=4;let a=r.exec('ababc');a.index===4&&r.lastIndex===5&&r.exec('ababc')===null&&r.lastIndex===0",
    );
    assert_true(
        r"let r=/(?<=(?:ab){2})/dg,a=[...'ababab'.matchAll(r)];a.length===2&&a[0].index===4&&a[1].index===6&&r.lastIndex===0",
    );
    assert_true(
        r"let seen=[];let s='ababcqababc'.replace(/(?<=(?<x>(?:ab){2}))c/g,(m,c,i,s,g)=>{seen.push(m,c,i,s,g.x);return '_'});s==='abab_qabab_'&&seen.length===10&&seen[0]==='c'&&seen[1]==='abab'&&seen[2]===4&&seen[4]==='abab'&&seen[7]===10",
    );
    assert_true(
        r"'ababcqababc'.replace(/(?<=(?<x>(?:ab){2}))c/g,'<$<x>>')==='abab<abab>qabab<abab>'&&'ababcqababc'.search(/(?<=(?:ab){2})c/)===4&&'ababcqababc'.split(/(?<=(?:ab){2})c/).join('|')==='abab|qabab|'",
    );
}

#[test]
fn explicit_sequence_lookbehind_work_aborts_before_last_index_and_handlers() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval("let marker=0,r=/(?<=(?:ab){8})c/g;r.lastIndex=1;let text='ab'.repeat(5000)")
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
fn variable_repeated_capture_assertion_reference_and_nested_counts_match_reference_results() {
    for source in [
        r"/(?<=(?:ab){1,2})c/.exec('ababc')",
        r"/(?<=(ab){1,2})c/.exec('ababc')",
        r"/(?<=(?:a(?=a)){2})c/.exec('ababc')",
        r"/(?<=(?:a(?<=a)){2})c/.exec('ababc')",
        r"/(a)(?<=(?:\1b){1,2})c/.exec('ababc')",
        r"/(?<=(?:(?:ab){2}){2})c/.exec('ababababc')",
        r"/(?<=(?:ab){2})c/u.exec('ababc')",
        r"/(?<=(?:ab){2})c/v.exec('ababc')",
    ] {
        crate::common::regexp_oracle::assert_matches_reference(&mut Realm::default(), source);
    }
    let source = format!(
        "new RegExp('(?<=(?:ab){{{},{}}})c').exec('ababc')",
        usize::MAX / 2 + 1,
        usize::MAX / 2 + 2
    );
    crate::common::regexp_oracle::assert_matches_reference(&mut Realm::default(), &source);
}
