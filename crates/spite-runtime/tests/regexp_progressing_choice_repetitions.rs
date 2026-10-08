//! Capture-free repetition alternatives that always consume input.

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
fn progressing_choice_repetition_result_snapshot() {
    let mut rows = String::new();
    for (source, flags, text) in [
        (r"(?:ab|a)+b", "d", "ab"),
        (r"(?:a|ab)+b", "d", "abb"),
        (r"(?:ab|a)+?b", "d", "ababb"),
        (r"(?:ab|a)*b", "d", "ababb"),
        (r"(?:ab|a)*?b", "d", "ababb"),
        (r"(?:ab|c){2,3}d", "d", "abccd"),
        (r"(?:ab|c){2,3}?d", "d", "abccd"),
        (r"(?:ab|c)?d", "d", "abd"),
        (r"(?:ab|c)??d", "d", "abd"),
        (r"((?:ab|c)+)d", "d", "abccd"),
        (r"((?:ab|c)+?)d", "d", "abccd"),
        (r"(a)(?:\1b|b)+\1", "d", "aaba"),
        (r"(a)(?:\1b|b)+?\1", "d", "aaba"),
        (r"(a)(?:b\1|b)*\1", "d", "abbaa"),
        (r"(a)(?:b\1|b)*?\1", "d", "abbaa"),
        (r"(?:(?:ab|c)+)", "d", "abcc"),
        (r"(?:ab|c)+(?:bc|a)+d", "d", "ababcad"),
        (r"(?:ab|c)+?(?:bc|a)+?d", "d", "ababcad"),
        (r"(?:[ab]a|b)+c", "d", "aabbc"),
        (r"(?:.b|a)+c", "ds", "\nbac"),
        (r"(?<all>(?:ab|c)+)d", "d", "abccd"),
        (r"(?<x>a)(?:\k<x>b|b)+\k<x>", "dgi", "aaba"),
        (r"(?:(?<x>(?:ab|c)+)d|(?<x>a+))\k<x>", "d", "abcdabc"),
        (r"(?<all>(?:^a|b$)+)", "dm", "q\na"),
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
fn choice_iteration_retries_restore_counts_named_enclosures_and_source_order() {
    check(
        r"let a=/(?<all>(?:\k<all>a|b)+)\k<all>/d.exec('abab');a[0]==='abab'&&a.groups.all==='ab'&&a.indices.groups.all===a.indices[1]&&a.indices.groups.all[1]===2",
    );
    check(
        r"let a=/(?<x>a)(?:\k<later>b|a)+(?<later>\k<x>)/d.exec('abbaa');a[0]==='abbaa'&&a.groups.x==='a'&&a.groups.later==='a'&&a.indices.groups.x===a.indices[1]&&a.indices.groups.later===a.indices[2]&&a.indices.groups.later[0]===4",
    );
    check(
        r"let a=/(?<all>(?:ab|a)+)b/d.exec('ab'),b=/(?<all>(?:ab|a)+)b/d.exec('abb');a[0]==='ab'&&a.groups.all==='a'&&a.indices.groups.all===a.indices[1]&&a.indices.groups.all[1]===1&&b[0]==='abb'&&b.groups.all==='ab'&&b.indices.groups.all[1]===2",
    );
    check(
        r"let a=/(?<all>(?:a|ab)+)b/d.exec('abb'),b=/(?<all>(?:ab|a)+?)b/d.exec('ababb');a[0]==='ab'&&a.groups.all==='a'&&b[0]==='ababb'&&b.groups.all==='abab'&&b.indices.groups.all[1]===4",
    );
    check(
        r"let a=/(?<x>a)(?:\k<x>b|b)+\k<x>/d.exec('aaba');a[0]==='aaba'&&a.groups.x==='a'&&a.indices.groups.x===a.indices[1]&&a.indices.groups.x[0]===0&&a.indices.groups.x[1]===1",
    );
    check(
        r"let a=/(?:(?<x>(?:ab|c)+)d|(?<x>a+))\k<x>/d.exec('abcdabc');a[0]==='abcdabc'&&a.groups.x==='abc'&&a[2]===undefined&&a.indices.groups.x===a.indices[1]&&a.indices.groups.x[1]===3",
    );
    check(
        r"let a=/(?<all>(?:ab|c)+(?:ab|c)+)/d.exec('abc');a[0]==='abc'&&a.groups.all==='abc'&&a.indices.groups.all===a.indices[1]&&a.indices.groups.all[1]===3",
    );
    check(
        r"let a=/(?<all>(?:.b|a)+)c/ds.exec('\uD800bac'),b=/(?<all>(?:^a|b$)+)/dm.exec('q\na');a.groups.all.length===3&&a.groups.all.charCodeAt(0)===0xD800&&a.indices.groups.all[1]===3&&b.index===2&&b.groups.all==='a'",
    );
}

#[test]
fn progressing_choices_consumers_sticky_counts_and_zero_advancement_keep_semantics() {
    check(
        r"let r=/(?<all>(?:ab|c)+)d/dg,a=[...'abccd abd'.matchAll(r)];a.length===2&&a[0][0]==='abccd'&&a[0].groups.all==='abcc'&&a[0].indices.groups.all[1]===4&&a[1].index===6&&a[1].groups.all==='ab'&&r.lastIndex===0",
    );
    check(
        r"'qabccdZ'.replace(/(?<all>(?:ab|c)+)d/,'<$<all>>')==='q<abcc>Z'&&'qabccdZ'.search(/((?:ab|c)+)d/)===1&&'qabccdZ'.split(/((?:ab|c)+)d/).join(',')==='q,abcc,Z'",
    );
    check(
        r"let r=/(?<all>(?:ab|c)+)d/dy;r.lastIndex=1;let copy=new RegExp(r),a=r.exec('qabccd');a.groups.all==='abcc'&&a.indices.groups.all[0]===1&&r.lastIndex===6&&r.exec('qabccd')===null&&r.lastIndex===0&&copy.source===r.source&&copy.lastIndex===0",
    );
    check(
        r"let a=[...'q'.matchAll(/(?<all>(?:ab|c)*)/dg)];a.length===2&&a[0].groups.all===''&&a[1].groups.all===''&&a[1].indices.groups.all[0]===1&&a[1].indices.groups.all[1]===1",
    );
    check(
        r"let a=/((?:ab|c){2,3})d/d.exec('abccd'),b=/(?:ab|c)*d|a/.exec('a');a[0]==='abccd'&&a[1]==='abcc'&&a.indices[1][1]===4&&b[0]==='a'",
    );
}

#[test]
fn many_choice_iterations_deep_enclosures_huge_bounds_copy_and_collect_unlimited() {
    let mut realm = Realm::default();
    realm.eval("let r=new RegExp('(?<all>'+'('.repeat(99999)+'(?:ab|c)+'+')'.repeat(100000),'d'),copy=new RegExp(r)").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("let a=copy.exec('abc');a[0]==='abc'&&a.length===100001&&a.groups.all==='abc'&&a[100000]==='abc'&&a.indices[100000][0]===0&&a.indices[100000][1]===3&&a.indices.groups.all===a.indices[1]&&copy.source===r.source"), Ok(Value::Boolean(true)));
    assert_eq!(realm.eval("let many=/(?<all>(?:ab|a)+)b/d.exec('a'.repeat(50000)+'b'),n='9'.repeat(10000),huge=new RegExp('(?:ab|c){'+n+'}'),zero=new RegExp('(?<all>(?:ab|c){0,'+n+'})','d'),empty=zero.exec('q');many[0].length===50001&&many.groups.all.length===50000&&many.indices.groups.all[1]===50000&&huge.exec('abc')===null&&empty.groups.all===''&&empty.indices.groups.all===empty.indices[1]"), Ok(Value::Boolean(true)));
    realm.collect(usize::MAX).unwrap();
}

#[test]
fn explicit_choice_iteration_work_aborts_before_last_index_and_handlers() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval(r"let marker=0,r=/(?<all>(?:ab|a)+)c/g;r.lastIndex=1;let text='a'.repeat(5000)")
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
fn empty_capturing_and_nested_repeated_choices_remain_unsupported() {
    for source in [
        r"/(?:ab|)+/.test('ab')",
        r"/((ab)|(a))+/.test('ab')",
        r"/((?:(?:ab|c)+){2}){2}/.test('abcabc')",
        r"/(?:\1|b)+(a)/.test('ba')",
        r"/(?:(?=a)a|b)+/.test('a')",
        r"/(?:ab|c)+/u.test('ab')",
    ] {
        assert!(
            matches!(
                Realm::default().eval(source),
                Err(Error::Unsupported { .. })
            ),
            "{source}"
        );
    }
    check(r"/(?:ab|c)+d/.exec('ab')===null");
}
