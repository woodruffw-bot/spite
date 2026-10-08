//! Quantified references retain targets and restore failed continuations.

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
fn quantified_reference_result_snapshot() {
    let mut rows = String::new();
    for (source, flags, text) in [
        (r"(a)\1*", "d", "aaaa"),
        (r"(a)\1*?", "d", "aaaa"),
        (r"(a)\1+", "d", "a"),
        (r"(a)\1+?", "d", "aaaa"),
        (r"(a)\1?", "d", "aa"),
        (r"(a)\1??", "d", "aa"),
        (r"(ab)\1{2,3}", "d", "abababab"),
        (r"(ab)\1{1,3}?", "d", "abababab"),
        (r"(a)\1*ab", "d", "qaaaab"),
        (r"(a)\1*?ab", "d", "qaaaab"),
        (r"((a)\2+)\1", "d", "aaaaaa"),
        (r"((a)\2+?)\1", "d", "aaaaaa"),
        (r"(a|ab)\1+b", "d", "ababb"),
        (r"(?:(a)|(ab))\1*\2+", "d", "abab"),
        (r"(a)(b)\1*\2*", "d", "abaaabbb"),
        (r"(a)(b)\1*?\2*?", "d", "abaaabbb"),
        (r"\1+(a)", "d", "a"),
        (r"(\1+a)\1", "d", "aa"),
        (r"()\1{2,3}", "d", "q"),
        (r"(a)\1{999999999999999999999999999999}", "d", "aa"),
        (r"(?:(?<x>a)|(?<x>ab))\k<x>+b", "d", "ababb"),
        (r"(?<x>\k<x>+a)\k<x>+", "dgi", "aAaA"),
        (r"(?<x>a)(?<y>b)\k<x>*\k<y>*", "dy", "qabaaabbb"),
        (r"^(?<x>.)\k<x>+$", "dis", "µΜµ"),
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
fn greedy_lazy_and_failed_suffixes_preserve_prefix_and_enclosing_capture_ranges() {
    check(
        r"let a=/((?<x>a)\k<x>+)\1/d.exec('aaaaaa');a[0]==='aaaaaa'&&a[1]==='aaa'&&a.groups.x==='a'&&a.indices[1][1]===3&&a.indices.groups.x===a.indices[2]",
    );
    check(
        r"let a=/((?<x>a)\k<x>+?)\1/d.exec('aaaaaa');a[0]==='aaaa'&&a[1]==='aa'&&a.indices[1][1]===2&&a.groups.x==='a'",
    );
    check(
        r"let a=/(?:(?<x>a)|(?<x>ab))\k<x>+b/d.exec('qabababb');a.index===1&&a[0]==='abababb'&&a[1]===undefined&&a.groups.x==='ab'&&a.indices.groups.x===a.indices[2]",
    );
    check(
        r"let a=/(?<x>a)(?<y>b)\k<x>*\k<y>*c/d.exec('abaaabbbc');a[0]==='abaaabbbc'&&a.groups.x==='a'&&a.groups.y==='b'&&a.indices.groups.x===a.indices[1]&&a.indices.groups.y===a.indices[2]",
    );
    check(
        r"let a=/(?<x>.)\k<x>+/d.exec('\uD800\uD800\uD800');a[0].length===3&&a.groups.x.charCodeAt(0)===0xD800&&a.indices.groups.x===a.indices[1]&&a.indices[1][1]===1",
    );
}

#[test]
fn consumers_empty_targets_and_sticky_failures_keep_original_sources_and_aliases() {
    check(
        r"let r=/(?<x>a)\k<x>+/dg,a=[...'aaa aa'.matchAll(r)];a.length===2&&a[0][0]==='aaa'&&a[1][0]==='aa'&&a[0].groups.x==='a'&&a[0].indices.groups.x===a[0].indices[1]&&r.lastIndex===0",
    );
    check(
        r"'qaaaa'.replace(/(?<x>a)\k<x>+/,'<$<x>>')==='q<a>'&&'qaaaa'.search(/(a)\1+/)===1&&'qaaaaZ'.split(/(a)\1+/).join(',')==='q,a,Z'",
    );
    check(
        r"let r=/(?<x>a)\k<x>+/dy;r.lastIndex=1;let copy=new RegExp(r),a=r.exec('qaaa');a[0]==='aaa'&&r.lastIndex===4&&r.exec('qaaa')===null&&r.lastIndex===0&&copy.source===r.source&&copy.lastIndex===0",
    );
    check(
        r"let a=[...'q'.matchAll(/(?<x>)\k<x>+/dg)];a.length===2&&a[0].index===0&&a[1].index===1&&a[0].groups.x===''&&a[0].indices.groups.x===a[0].indices[1]",
    );
}

#[test]
fn huge_empty_minimums_and_many_reference_atoms_use_unlimited_defaults_and_flat_storage() {
    let mut realm = Realm::default();
    // RepeatMatcher has no capture effects inside a reference atom. Finite huge
    // minimums over an empty target preserve the same state without native loops.
    assert_eq!(realm.eval("let huge=new RegExp('(?<x>)\\\\k<x>{'+'9'.repeat(10000)+'}','d'),empty=huge.exec('q');empty[0]===''&&empty.groups.x===''&&empty.indices.groups.x===empty.indices[1]"),Ok(Value::Boolean(true)));
    assert_eq!(realm.eval("new RegExp('(?<x>\\\\k<x>{'+'9'.repeat(10000)+'}a)\\\\k<x>+','d').exec('aa').groups.x==='a'"),Ok(Value::Boolean(true)));
    realm
        .eval("let r=new RegExp('(a)'+'\\\\1*?'.repeat(100000)+'b','d'),copy=new RegExp(r)")
        .unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("let a=copy.exec('ab');a[0]==='ab'&&a[1]==='a'&&a.indices[1][0]===0&&copy.source===r.source"),Ok(Value::Boolean(true)));
    realm.collect(usize::MAX).unwrap();
    assert_eq!(
        realm.eval("a[1]==='a'&&a.indices[1][1]===1"),
        Ok(Value::Boolean(true))
    );
}

#[test]
fn explicit_quantified_reference_work_aborts_before_last_index_and_handlers() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval(r"let r=/(?<x>a)\k<x>*b/g,flag=0,text='a'.repeat(10000);r.lastIndex=1")
        .unwrap();
    assert!(matches!(
        realm.eval("try{r.exec(text)}catch{flag=1}finally{flag=2}"),
        Err(Error::Limit { .. })
    ));
    assert_eq!(
        realm.eval("flag===0&&r.lastIndex===1"),
        Ok(Value::Boolean(true))
    );
}

#[test]
fn repeated_reference_bodies_other_quantified_atoms_and_unicode_remain_unsupported() {
    for source in [
        r"/(?:(?:(?:(?:(a)\1){2}){2})|)*/.test('aaaa')",
        r"/(?:(?:(?:(a)(?:\1)+){2})|)*/.test('aa')",
        r"/(?:(?:(?:(a)+\1){2})|)*/.test('aa')",
        r"/(?=(a))\1+/.test('aa')",
        r"/(?<x>a)\k<x>+/u.test('aa')",
    ] {
        assert!(
            matches!(
                Realm::default().eval(source),
                Err(Error::Unsupported { .. })
            ),
            "{source}"
        );
    }
    check(r"/(?<x>a)\k<x>+b/.exec('aaa')===null");
}
