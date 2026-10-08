//! Captures inside progressing repeated branches and nested branch loops.
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
fn repeated_branch_capture_result_snapshot() {
    let mut rows = String::new();
    for (source, flags, text) in [
        (r"(a|ab)+b", "d", "abb"),
        (r"(ab|a)+b", "d", "abb"),
        (r"(a|ab)+?b", "d", "ababb"),
        (r"(ab|a)+?b", "d", "ababb"),
        (r"((a)|(b))+c", "d", "abc"),
        (r"((a)|(b))+?c", "d", "abc"),
        (r"(?:(a)|b)+c", "d", "abc"),
        (r"(?:(a)|b)*c", "d", "c"),
        (r"(a|b){2,3}c", "d", "abbc"),
        (r"(a|b){2,3}?c", "d", "abbc"),
        (r"((a)+b|c)+d", "d", "aabccd"),
        (r"((a)+?b|c)+?d", "d", "aabccd"),
        (r"(a)(?:(b)\1|c)+\2", "d", "abacc"),
        (r"(a)(?:(b)\1|c)+?\2", "d", "abacc"),
        (r"((a|ab)+)+b", "d", "abb"),
        (r"((a|ab)+?)+?b", "d", "abb"),
        (r"(a|(b))+(c)\2", "d", "abac"),
        (r"(a|\1b)+c", "d", "abac"),
        (r"((\2a)|b)+c", "d", "abc"),
        (r"((^a)|(b$))+", "dm", "q\na"),
        (r"(?<x>[ab]|c)+d", "di", "aABcd"),
        (r"(?<x>.|a)+b", "ds", "\n\nab"),
        (r"(?:(?<x>a)|(?<x>b)){2}\k<x>", "d", "abb"),
        (r"^(?:(?<x>a)|(?<x>b)|c){2}\k<x>$", "d", "ac"),
        (r"(?<x>a|\k<x>b)+c", "d", "abac"),
        (r"(?<outer>(?<inner>a|ab)+)+b", "d", "abb"),
        (r"(?:(a)(b)((?<x>\1)\k<x>\2)+){2}", "d", "abaababaab"),
        (r"(?:(a)(b)(\1(?<x>\2)\k<x>)+){2}", "d", "ababbababb"),
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
fn repeated_branch_capture_clearing_and_undo_preserve_names_refs_and_last_ranges() {
    check(
        r"let a=/(?:(a)(b)((?<x>\1)\k<x>\2)+){2}/d.exec('abaababaab');a[0]==='abaababaab'&&a.groups.x==='a'&&a.indices.groups.x===a.indices[4]&&a.indices[4][0]===7&&a.indices[4][1]===8",
    );
    check(
        r"let a=/(?:(a)(b)(\1(?<x>\2)\k<x>)+){2}/d.exec('ababbababb');a[0]==='ababbababb'&&a.groups.x==='b'&&a.indices.groups.x===a.indices[4]&&a.indices[4][0]===8&&a.indices[4][1]===9",
    );
    check(
        r"let a=/(?<x>a|ab)+b/d.exec('abb'),b=/(?<x>ab|a)+b/d.exec('abb');a[0]==='ab'&&a.groups.x==='a'&&a.indices.groups.x===a.indices[1]&&a.indices[1][1]===1&&b[0]==='abb'&&b.groups.x==='ab'&&b.indices[1][1]===2",
    );
    check(
        r"let a=/(?<all>(?:(?<a>a)|(?<b>b))+c)/d.exec('abc');a.groups.all==='abc'&&a.groups.a===undefined&&a.groups.b==='b'&&a.indices.groups.a===undefined&&a.indices.groups.b===a.indices[3]&&a.indices[3][0]===1",
    );
    check(
        r"let a=/(?:(?<x>a)|(?<x>b)){2}\k<x>/d.exec('abb'),b=/^(?:(?<x>a)|(?<x>b)|c){2}\k<x>$/d.exec('ac');a[0]==='abb'&&a[1]===undefined&&a[2]==='b'&&a.groups.x==='b'&&a.indices.groups.x===a.indices[2]&&b[0]==='ac'&&b.groups.x===undefined&&b.indices.groups.x===undefined&&!/^(?:(?<x>a)|(?<x>b)|c){2}\k<x>$/.test('aca')",
    );
    check(
        r"let a=/(?<x>a)(?:(?<y>b)\k<x>|c)+\k<y>/d.exec('abacc');a[0]==='abacc'&&a.groups.x==='a'&&a.groups.y===undefined&&a.indices.groups.y===undefined&&a.indices[2]===undefined",
    );
    check(
        r"let a=/(?<x>a|\k<x>b)+c/d.exec('abac');a[0]==='abac'&&a.groups.x==='a'&&a.indices.groups.x===a.indices[1]&&a.indices[1][0]===2",
    );
    check(
        r"let a=/(?<outer>(?<inner>a|ab)+)+b/d.exec('abb');a[0]==='ab'&&a.groups.outer==='a'&&a.groups.inner==='a'&&a.indices.groups.outer===a.indices[1]&&a.indices.groups.inner===a.indices[2]&&a.indices[1][1]===1&&a.indices[2][1]===1",
    );
    check(
        r"let a=/(?<x>(\k<later>a)|b)+(?<later>c)/d.exec('abc');a[0]==='abc'&&a.groups.x==='b'&&a[2]===undefined&&a.groups.later==='c'&&a.indices.groups.x[0]===1&&a.indices.groups.later[0]===2",
    );
    check(
        r"let a=/(?<x>.|a)+c/ds.exec('\uD800ac');a.groups.x==='a'&&a.indices.groups.x[0]===1&&a.indices.groups.x[1]===2",
    );
}

#[test]
fn repeated_branch_capture_consumers_sticky_zero_iterations_and_undefined_slots() {
    check(
        r"let r=/(?:(?<x>a)|b)+c/dg,a=[...'abc ac'.matchAll(r)];a.length===2&&a[0][0]==='abc'&&a[0].groups.x===undefined&&a[0].indices.groups.x===undefined&&a[1].index===4&&a[1].groups.x==='a'&&a[1].indices.groups.x===a[1].indices[1]&&a[1].indices[1][0]===4&&r.lastIndex===0",
    );
    check(
        r"let a='abc'.split(/(?:(a)|b)+c/);'abc'.replace(/(?:(?<x>a)|b)+c/,'<$<x>>')==='<>'&&'abc'.search(/(?:(a)|b)+c/)===0&&a.length===3&&a[1]===undefined",
    );
    check(
        r"let r=/(?:(?<x>a)|b)+c/dy;r.lastIndex=1;let copy=new RegExp(r),a=r.exec('qabc');a.groups.x===undefined&&a.indices.groups.x===undefined&&r.lastIndex===4&&r.exec('qabc')===null&&r.lastIndex===0&&copy.source===r.source&&copy.lastIndex===0",
    );
    check(
        r"let a=[...'q'.matchAll(/(?<all>(?:(a)|b)*)/dg)];a.length===2&&a[0].groups.all===''&&a[0][2]===undefined&&a[1][2]===undefined&&a[1].indices.groups.all[0]===1&&a[1].indices.groups.all[1]===1",
    );
    check(
        r"let a=/(?<x>a|b){2,3}c/d.exec('abbc'),b=/(?<x>a|b){2,3}?c/d.exec('abbc');a[0]==='abbc'&&a.groups.x==='b'&&a.indices.groups.x[0]===2&&b.groups.x==='b'&&b.indices.groups.x[0]===2",
    );
}

#[test]
fn repeated_branch_capture_deep_inventories_nested_loops_runs_and_gc_are_unlimited() {
    let mut realm = Realm::default();
    realm.eval("let r=new RegExp('(?<all>'+'('.repeat(99999)+'a|b'+')'.repeat(100000)+'+','d'),copy=new RegExp(r)").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("let a=copy.exec('ab');a[0]==='ab'&&a.length===100001&&a.groups.all==='b'&&a[100000]==='b'&&a.indices.groups.all===a.indices[1]&&a.indices[100000][0]===1&&a.indices[100000][1]===2&&copy.source===r.source"),Ok(Value::Boolean(true)));
    assert_eq!(realm.eval("let deep=new RegExp('('.repeat(1000)+'(?:a|b)'+')+'.repeat(1000),'d'),d=deep.exec('a');d.length===1001&&d[1]==='a'&&d[1000]==='a'&&d.indices[1000][1]===1"),Ok(Value::Boolean(true)));
    assert_eq!(realm.eval("let many=/(?<all>(?<last>a|b)+)c/d.exec('ab'.repeat(50000)+'c'),n='9'.repeat(10000),huge=new RegExp('(a|b){'+n+'}'),zero=new RegExp('(?<all>(a|b){0,'+n+'})','d'),empty=zero.exec('q');many[0].length===100001&&many.groups.last==='b'&&many.indices.groups.last[0]===99999&&many.groups.all.length===100000&&huge.exec('ab')===null&&empty.groups.all===''&&empty[2]===undefined&&empty.indices[2]===undefined"),Ok(Value::Boolean(true)));
    realm.collect(usize::MAX).unwrap();
}

#[test]
fn opted_in_branch_capture_work_aborts_before_last_index_and_handlers() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval("let marker=0,r=/(?<x>a|ab)+c/g;r.lastIndex=1;let text='a'.repeat(5000)")
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
fn empty_complete_paths_lookaround_and_unicode_remain_distinct_from_match_failures() {
    for source in [
        r"/(a|){2,3}/.test('ab')",
        r"/((a|b)*){2,3}/.test('ab')",
        r"/((?<=(a+))a|b)+/.test('a')",
        r"/(a|b)+/u.test('ab')",
        r"/(a|b)+/v.test('ab')",
        r"/((?:a|b)?){2,3}/.test('ab')",
    ] {
        assert!(
            matches!(
                Realm::default().eval(source),
                Err(Error::Unsupported { .. })
            ),
            "{source}"
        );
    }
    check(r"/(a|b)+c/.exec('ab')===null");
}
