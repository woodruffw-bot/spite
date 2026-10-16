//! Required and optional zero-width lookahead wrappers (RepeatMatcher).
use spite_core::JsString;
use spite_runtime::{Error, Limits, Realm, Value};
use std::fmt::Write;

fn check(source: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::Boolean(true)),
        "{source}"
    );
}

#[test]
fn zero_width_lookahead_repetition_result_snapshot() {
    let mut rows = String::new();
    for (source, flags, text) in [
        (r"(?:(?=(abc)))a", "d", "abc"),
        (r"(?:(?=(abc)))?a", "d", "abc"),
        (r"(?:(?=(abc))){1,1}a", "d", "abc"),
        (r"(?:(?=(abc))){0,1}a", "d", "abc"),
        (r"((?=(a))){2}\1\2", "d", "a"),
        (r"((?=(a))){0,2}b", "d", "b"),
        (r"(?:(?!(a)b)){2}\1a", "d", "a"),
        (r"(?:(?!(a)b))*\1a", "d", "ab"),
        (r"(?:(?=(a))){2,4}?a", "d", "a"),
        (r"(?:(?=(a))){4}a", "d", "a"),
        (r"(?:(?=(a))){0}a", "d", "a"),
        (r"(?:(?=(a))){1}a|b", "d", "b"),
        (r"(?:(?=(a|ab))){2}\1b$", "d", "abb"),
        (r"(?:(?=(ab|a))){2}\1b$", "d", "abb"),
        (r"(?:(?=(a+))){2}a*b\1", "d", "baabac"),
        (r"(?:(?=(a))^){2}a", "d", "a"),
        (r"(?:(\b)(?=(a))()){2}a", "d", "a"),
        (r"(?:(?:(?=(a))){2}){3}\1", "d", "a"),
        (r"(?:(?:(?=(a)))?){2}a", "d", "a"),
        (r"(?:(?:(?=(a))){2}a|b)+c", "d", "abc"),
        (r"(?:(?:(?=(a)))?a|b)+c", "d", "abc"),
        (r"(?:(?=(^a))){2}\1", "dm", "q\na"),
        (r"(?:(?=(?<x>a|b)+)){2}\k<x>", "d", "ab"),
        (r"(?<all>(?=(?<x>a))){2}\k<x>", "d", "a"),
        (r"(?<all>(?=(?<x>a))){0,2}b", "d", "b"),
        (r"(?:(?=(?<x>µ))){2}\k<x>", "di", "Μ"),
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
    insta::assert_snapshot!(rows);
}

#[test]
fn required_and_optional_wrappers_preserve_empty_outer_and_external_capture_ranges() {
    check(
        r"let a=/(?:(?=(abc)))?a/d.exec('abc'),b=/(?:(?=(abc))){1,1}a/d.exec('abc');a[1]===undefined&&a.indices[1]===undefined&&b[0]==='a'&&b[1]==='abc'&&b.indices[0][1]===1&&b.indices[1][1]===3",
    );
    check(
        r"let a=/(?<all>(?=(?<x>a))){2}\k<x>/d.exec('a'),b=/(?<all>(?=(?<x>a))){0,2}b/d.exec('b');a.groups.all===''&&a.groups.x==='a'&&a.indices.groups.all===a.indices[1]&&a.indices[1][0]===0&&a.indices[1][1]===0&&a.indices.groups.x[1]===1&&b.groups.all===undefined&&b.groups.x===undefined&&b.indices.groups.all===undefined&&b.indices.groups.x===undefined",
    );
    check(
        r"let a=/(?:(\b)(?=(a))()){2}a/d.exec('a');a[1]===''&&a[2]==='a'&&a[3]===''&&a.indices[1][1]===0&&a.indices[2][1]===1&&a.indices[3][1]===0",
    );
    check(
        r"let a=/(?:(?!(?<x>a)b)){2}\k<x>a/d.exec('a');a.groups.x===undefined&&a.indices.groups.x===undefined&&/(?:(?!(a)b))*\1a/.exec('ab')[0]==='a'",
    );
    check(
        r"/(?:(?=(a|ab))){2}\1b$/.exec('abb')===null&&/(?:(?=(ab|a))){2}\1b$/.exec('abb')[1]==='ab'",
    );
    check(
        r"let a=/(?:(?:(?=(?<x>a))){2}a|b)+c/d.exec('abc');a[0]==='abc'&&a.groups.x===undefined&&a.indices.groups.x===undefined",
    );
    check(
        r"let a=/(?:(?=(\uD800))){2}\1/d.exec('\uD800');a[0].charCodeAt(0)===0xD800&&a[1].charCodeAt(0)===0xD800&&a.indices[1][0]===0&&a.indices[1][1]===1",
    );
}

#[test]
fn zero_width_repetition_consumers_global_sticky_and_named_callbacks_keep_state() {
    check(
        r"let r=/(?<all>(?=(?<x>a))){2}/dg,a=[...'aba'.matchAll(r)];a.length===2&&a[0][0]===''&&a[0].groups.all===''&&a[0].groups.x==='a'&&a[0].indices.groups.x[1]===1&&a[1].index===2&&r.lastIndex===0",
    );
    check(
        r"let r=/(?<all>(?=(?<x>a))){0,2}/dy;r.lastIndex=1;let a=r.exec('q');a[0]===''&&a.groups.all===undefined&&a.groups.x===undefined&&r.lastIndex===1",
    );
    check(
        r"let a=[];let result='aa'.replace(/(?:(?=(?<x>a))){2}/g,(m,c,i,s,g)=>{a.push(m,c,i,g.x);return '_'});result==='_a_a'&&a.length===8&&a[0]===''&&a[1]==='a'&&a[2]===0&&a[3]==='a'&&a[6]===1",
    );
    check(
        r"'aba'.replace(/(?:(?=(?<x>a))){2}/g,'<$<x>>')==='<a>ab<a>a'&&'aba'.search(/(?:(?=(a))){2}/)===0&&'aba'.split(/(?:(?=(a))){2}/).length===3",
    );
}

#[test]
fn zero_width_repetition_huge_counts_nested_wrappers_copies_and_collection_are_unlimited() {
    let mut realm = Realm::default();
    realm.eval("let n='9'.repeat(10000),r=new RegExp('(?<all>(?=(?<x>a))){'+n+'}\\\\k<x>','d'),copy=new RegExp(r),zero=new RegExp('(?<all>(?=(?<x>a))){0,'+n+'}b','d')").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("let a=copy.exec('a'),b=zero.exec('b');a[0]==='a'&&a.groups.all===''&&a.groups.x==='a'&&a.indices.groups.x===a.indices[2]&&b.groups.all===undefined&&b.groups.x===undefined&&copy.source===r.source"),Ok(Value::Boolean(true)));
    assert_eq!(realm.eval("let deep=new RegExp('(?:'.repeat(100000)+'(?=(?<x>a))'+'){2}'.repeat(100000)+'\\\\k<x>','d'),last=deep.exec('a');last[0]==='a'&&last.groups.x==='a'&&last.indices.groups.x[1]===1"),Ok(Value::Boolean(true)));
    realm.collect(usize::MAX).unwrap();
}

#[test]
fn opted_in_zero_width_repetition_search_work_aborts_before_last_index_and_handlers() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval(
            "let marker=0,r=/(?:(?=(?<x>a+))){2}\\k<x>b/g;r.lastIndex=1;let text='a'.repeat(5000)",
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
fn possibly_empty_choices_consuming_mixed_references_and_lookbehind_remain_unsupported() {
    for source in [
        r"/(?:(?=(a))|b){2,3}/.exec('a')",
        r"/(?:(?=(a))\1|b){2,3}/.exec('a')",
        r"/(?<=(a+))b/.exec('ab')",
        r"/(?:(?=(a))){2}\1/u.exec('a')",
        r"/(?:(?=(a))){2}\1/v.exec('a')",
    ] {
        assert!(
            matches!(
                Realm::default().eval(source),
                Err(Error::Unsupported { .. })
            ),
            "{source}"
        );
    }
}
