//! Capture-free nested branch repetitions with persistent iteration contexts.

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
fn nested_choice_loop_result_snapshot() {
    let mut rows = String::new();
    for (source, flags, text) in [
        (r"(?:(?:a|aa)+b|c)+d", "d", "aabccd"),
        (r"(?:(?:a|aa)+?b|c)+?d", "d", "aabccd"),
        (r"(?:(?:a|aa)*b|c)+d", "d", "bbcd"),
        (r"(?:(?:a|aa)*?b|c)+?d", "d", "bbcd"),
        (r"(?:(?:a|aa){2,3}b|c){2,3}d", "d", "aaabccd"),
        (r"(?:(?:a|aa){2,3}?b|c){2,3}?d", "d", "aaabccd"),
        (r"(?:a?b|c)+d", "d", "abbcd"),
        (r"(?:a??b|c)+d", "d", "abbcd"),
        (r"(?:(?:ab|a)+c|d)+e", "d", "ababcdde"),
        (r"(?:(?:ab|a)+?c|d)+?e", "d", "ababcdde"),
        (r"(?:(?:ab|a)*c|d)+e", "d", "ccde"),
        (r"(?:(?:ab|a)*?c|d)+?e", "d", "ccde"),
        (r"((?:(?:a|aa)+b|c)+)d", "d", "aabccd"),
        (r"((?:(?:a|aa)+?b|c)+?)d", "d", "aabccd"),
        (r"(a)(?:(?:\1a|b)*b|c)+\1", "d", "aaabca"),
        (r"(a)(?:(?:\1a|b)*?b|c)+?\1", "d", "aaabca"),
        (r"(?:(?:(?:a|)b)+)+c", "d", "abbbc"),
        (r"(?:(?:(?:a|)b)+?)+?c", "d", "abbbc"),
        (r"(?:(?:[ab]|c)+c|d)+e", "di", "aABcdde"),
        (r"(?:(?:.|a)+b|a)+c", "ds", "\n\nbac"),
        (r"(?<all>(?:(?:a|aa)+b|c)+)d", "d", "aabccd"),
        (r"(?<x>a)(?:(?:\k<x>a|b)*b|c)+\k<x>", "dgi", "aaabca"),
        (
            r"(?:(?<x>(?:(?:a|aa)+b|c)+)d|(?<x>a+))\k<x>",
            "d",
            "aabcdaabc",
        ),
        (r"(?<all>(?:(?:^a|a)+b|c)+)", "dm", "q\nab"),
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
    insta::assert_snapshot!(rows);
}

#[test]
fn child_loop_retries_restore_parent_contexts_and_outside_capture_closes() {
    check(
        r"let a=/(?<all>(?:a+b+){2})c/d.exec('aabaabc');a[0]==='aabaabc'&&a.groups.all==='aabaab'&&a.indices.groups.all===a.indices[1]&&a.indices.groups.all[1]===6",
    );
    check(
        r"let a=/(?<all>(?:(?:a|aa)+a|b)+)a/d.exec('aaaa');a[0]==='aaaa'&&a.groups.all==='aaa'&&a.indices.groups.all===a.indices[1]&&a.indices.groups.all[1]===3",
    );
    check(
        r"let a=/(?<all>(?:(?:a|aa)+?a|b)+?)a/d.exec('aaaa');a[0]==='aaa'&&a.groups.all==='aa'&&a.indices.groups.all[1]===2",
    );
    check(
        r"let a=/(?<x>a)(?:(?:\k<x>a|b)*b|c)+\k<x>/d.exec('aaabca');a[0]==='aaabca'&&a.groups.x==='a'&&a.indices.groups.x===a.indices[1]&&a.indices.groups.x[1]===1",
    );
    check(
        r"let a=/(?<all>(?:(?:\k<all>a|b)+)+)\k<all>/d.exec('abab');a[0]==='abab'&&a.groups.all==='ab'&&a.indices.groups.all[1]===2",
    );
    check(
        r"let a=/(?<x>a)(?:(?:\k<later>b|c)+)+(?<later>\k<x>)/d.exec('abbca');a[0]==='abbca'&&a.groups.later==='a'&&a.indices.groups.later[0]===4",
    );
    check(
        r"let a=/(?:(?<x>(?:(?:a|aa)+b|c)+)d|(?<x>a+))\k<x>/d.exec('aabcdaabc');a[0]==='aabcdaabc'&&a.groups.x==='aabc'&&a[2]===undefined&&a.indices.groups.x===a.indices[1]",
    );
    check(
        r"let a=/(?<all>(?:(?:a|aa)+b|c)+(?:(?:a|aa)+b|c)+)/d.exec('aabcc');a[0]==='aabcc'&&a.groups.all==='aabcc'&&a.indices.groups.all[1]===5",
    );
    check(
        r"let a=/(?<all>(?:(?:.|a)+b|a)+)c/ds.exec('\uD800\nbac');a.groups.all.length===4&&a.groups.all.charCodeAt(0)===0xD800&&a.indices.groups.all[1]===4",
    );
}

#[test]
fn nested_loop_consumers_sticky_counts_and_zero_advancement() {
    check(
        r"let r=/(?<all>(?:(?:a|aa)+b|c)+)d/dg,a=[...'aabccd abd'.matchAll(r)];a.length===2&&a[0][0]==='aabccd'&&a[0].groups.all==='aabcc'&&a[0].indices.groups.all[1]===5&&a[1].index===7&&a[1].groups.all==='ab'&&r.lastIndex===0",
    );
    check(
        r"'qaabccdZ'.replace(/(?<all>(?:(?:a|aa)+b|c)+)d/,'<$<all>>')==='q<aabcc>Z'&&'qaabccdZ'.search(/((?:(?:a|aa)+b|c)+)d/)===1&&'qaabccdZ'.split(/((?:(?:a|aa)+b|c)+)d/).join(',')==='q,aabcc,Z'",
    );
    check(
        r"let r=/(?<all>(?:(?:a|aa)+b|c)+)d/dy;r.lastIndex=1;let copy=new RegExp(r),a=r.exec('qaabccd');a.groups.all==='aabcc'&&a.indices.groups.all[0]===1&&r.lastIndex===7&&r.exec('qaabccd')===null&&r.lastIndex===0&&copy.source===r.source&&copy.lastIndex===0",
    );
    check(
        r"let a=[...'q'.matchAll(/(?<all>(?:(?:a|aa)+b|c)*)/dg)];a.length===2&&a[0].groups.all===''&&a[1].groups.all===''&&a[1].indices.groups.all[0]===1&&a[1].indices.groups.all[1]===1",
    );
    check(
        r"let a=/((?:(?:a|aa){2,3}b|c){2,3})d/d.exec('aaabccd'),b=/(?:(?:a|aa)+b|c)*d|a/.exec('a');a[0]==='aaabccd'&&a[1]==='aaabcc'&&a.indices[1][1]===6&&b[0]==='a'",
    );
}

#[test]
fn deep_nested_loops_enclosures_iterations_copies_and_collection_are_unlimited() {
    let mut realm = Realm::default();
    realm.eval("let r=new RegExp('(?<all>'+'('.repeat(99999)+'(?:(?:a|aa)+b|c)+'+')'.repeat(100000),'d'),copy=new RegExp(r)").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("let deep=new RegExp('(?<deep>'+'(?:'.repeat(100000)+'(?:a|b)'+')+'.repeat(100000)+')','d'),a2=deep.exec('a');a2[0]==='a'&&a2.groups.deep==='a'&&a2.indices.groups.deep===a2.indices[1]&&a2.indices[1][1]===1"),Ok(Value::Boolean(true)));
    assert_eq!(realm.eval("let a=copy.exec('aabcc');a[0]==='aabcc'&&a.length===100001&&a.groups.all==='aabcc'&&a[100000]==='aabcc'&&a.indices[100000][1]===5&&a.indices.groups.all===a.indices[1]&&copy.source===r.source"),Ok(Value::Boolean(true)));
    assert_eq!(realm.eval("let many=/(?<all>(?:(?:a|aa)+b|c)+)d/d.exec('ab'.repeat(50000)+'d'),n='9'.repeat(10000),huge=new RegExp('(?:a{'+n+'}|b)+c'),zero=new RegExp('(?<all>(?:(?:a|aa)+b|c){0,'+n+'})','d'),empty=zero.exec('q');many[0].length===100001&&many.groups.all.length===100000&&many.indices.groups.all[1]===100000&&huge.exec('aac')===null&&empty.groups.all===''&&empty.indices.groups.all===empty.indices[1]"),Ok(Value::Boolean(true)));
    realm.collect(usize::MAX).unwrap();
}

#[test]
fn opted_in_nested_loop_work_aborts_before_last_index_and_handlers() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval(
            "let marker=0,r=/(?<all>(?:(?:a|aa)+b|c)+)d/g;r.lastIndex=1;let text='ab'.repeat(5000)",
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
fn possibly_empty_paths_body_captures_lookaround_and_unicode_remain_unsupported() {
    for source in [
        r"/(?:a*b*)+/.test('ab')",
        r"/(?:(?:(?:(a+)b|c)+)|)*/.test('aab')",
        r"/(?:(?:(?:(?:(a)|b)+c|d)+)|)*/.test('abc')",
        r"/(?:\1*|b)+(a)/.test('ba')",
        r"/(?:(?=a)a+|b)+/.test('a')",
        r"/(?:(?:a|aa)+b|c)+/u.test('aab')",
    ] {
        assert!(
            matches!(
                Realm::default().eval(source),
                Err(Error::Unsupported { .. })
            ),
            "{source}"
        );
    }
    check(r"/(?:(?:a|aa)+b|c)+d/.exec('aab')===null");
}
