//! Source-order alternatives in proven zero-width lookahead repetition bodies.
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
    insta::assert_snapshot!(rows);
}

#[test]
fn zero_width_branch_retries_restore_inactive_slots_named_aliases_and_atomic_assertions() {
    check(
        r"let a=/(?:(?=(?<x>a))|(?=(?<x>ab))){2}\k<x>b$/d.exec('abb');a[0]==='abb'&&a[1]===undefined&&a[2]==='ab'&&a.groups.x==='ab'&&a.indices.groups.x===a.indices[2]&&a.indices[1]===undefined&&a.indices[2][1]===2",
    );
    check(
        r"let a=/((?=(a))|()){2}\1\2\3/d.exec('b');a[0]===''&&a[1]===''&&a[2]===undefined&&a[3]===''&&a.indices[1][1]===0&&a.indices[2]===undefined&&a.indices[3][1]===0",
    );
    check(
        r"/(?:(?=(a|ab))|(?=(ab))){2}\1b$/.exec('abb')===null&&/(?:(?=(ab|a))|(?=(a))){2}\1b$/.exec('abb')[1]==='ab'",
    );
    check(
        r"let a=/(?:(?=(?<x>a))|(?=(?<x>b))){0,3}\k<x>c/d.exec('ac');a[0]==='c'&&a.index===1&&a.groups.x===undefined&&a.indices.groups.x===undefined",
    );
    check(
        r"let a=/(?:(?!(?<x>a)b)|(?=(?<x>a))){2}a/d.exec('a');a.groups.x===undefined&&a.indices.groups.x===undefined&&a[1]===undefined&&a[2]===undefined",
    );
    check(
        r"let a=/(?:(?:(?=(?<x>a))|()){2}a|b)+c/d.exec('abc');a[0]==='abc'&&a.groups.x===undefined&&a.indices.groups.x===undefined",
    );
    check(
        r"let a=/(?:(?=(\uD800))|()){2}\1/d.exec('\uD800');a[0].charCodeAt(0)===0xD800&&a[1].charCodeAt(0)===0xD800&&a.indices[1][1]===1&&a[2]===undefined",
    );
}

#[test]
fn zero_width_branch_consumers_empty_global_sticky_results_and_callbacks_keep_slots() {
    check(
        r"let r=/(?:(?=(?<x>a))|(?=(?<x>b))){2}/dg,a=[...'ab'.matchAll(r)];a.length===2&&a[0][0]===''&&a[0].groups.x==='a'&&a[0].indices.groups.x===a[0].indices[1]&&a[1].index===1&&a[1][1]===undefined&&a[1].groups.x==='b'&&a[1].indices.groups.x===a[1].indices[2]&&r.lastIndex===0",
    );
    check(
        r"let r=/(?:(?=(?<x>a))|(?=(?<x>b))){2}/dy;r.lastIndex=1;let a=r.exec('qb');a[0]===''&&a.groups.x==='b'&&a.indices.groups.x[0]===1&&r.lastIndex===1&&r.exec('qb').groups.x==='b'",
    );
    check(
        r"let a=[];let result='ab'.replace(/(?:(?=(?<x>a))|(?=(?<x>b))){2}/g,(m,c1,c2,i,s,g)=>{a.push(m,c1,c2,i,g.x);return '_'});result==='_a_b'&&a.length===10&&a[0]===''&&a[1]==='a'&&a[2]===undefined&&a[3]===0&&a[4]==='a'&&a[6]===undefined&&a[7]==='b'&&a[8]===1&&a[9]==='b'",
    );
    check(
        r"'ab'.replace(/(?:(?=(?<x>a))|(?=(?<x>b))){2}/g,'<$<x>>')==='<a>a<b>b'&&'ab'.search(/(?:(?=(a))|(?=(b))){2}/)===0&&'ab'.split(/(?:(?=(a))|(?=(b))){2}/).length===4",
    );
}

#[test]
fn zero_width_branch_huge_counts_nested_wrappers_long_parent_runs_copies_and_gc_are_unlimited() {
    let mut realm = Realm::default();
    realm.eval("let n='9'.repeat(10000),r=new RegExp('(?:(?=(?<x>a))|(?=(?<x>b))){'+n+'}\\\\k<x>','d'),copy=new RegExp(r)").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("let a=copy.exec('b');a[0]==='b'&&a[1]===undefined&&a.groups.x==='b'&&a.indices.groups.x===a.indices[2]&&copy.source===r.source"),Ok(Value::Boolean(true)));
    assert_eq!(realm.eval("let deep=new RegExp('(?:'.repeat(100000)+'(?:(?=(?<x>a))|())'+'){2}'.repeat(100000)+'\\\\k<x>','d'),last=deep.exec('a');last[0]==='a'&&last.groups.x==='a'&&last.indices.groups.x[1]===1"),Ok(Value::Boolean(true)));
    assert_eq!(realm.eval("let many=/(?:(?:(?=(?<x>a))|()){2}a|b)+c/d.exec('ab'.repeat(10000)+'c');many[0].length===20001&&many.groups.x===undefined&&many.indices.groups.x===undefined"),Ok(Value::Boolean(true)));
    realm.collect(usize::MAX).unwrap();
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
fn mixed_progress_unknown_references_lookbehind_and_unicode_remain_unsupported() {
    for source in [
        r"/(?:(?=(a))|b)+/.exec('a')",
        r"/(?:(?=(a))\1|b)+/.exec('a')",
        r"/(?<=(a+))b/.exec('ab')",
        r"/(?:(?=(a))|()){2}\1/u.exec('a')",
        r"/(?:(?=(a))|()){2}\1/v.exec('a')",
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
