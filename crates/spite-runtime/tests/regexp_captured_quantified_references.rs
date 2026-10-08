//! Capturing reference wrappers preserve final-iteration ranges.

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
fn captured_quantified_reference_result_snapshot() {
    let mut rows = String::new();
    for (source, flags, text) in [
        (r"(a)(\1)*", "d", "aaaa"),
        (r"(a)(\1)*?", "d", "aaaa"),
        (r"(a)(\1)+", "d", "a"),
        (r"(a)(\1)+?", "d", "aaaa"),
        (r"(a)(\1)?", "d", "aa"),
        (r"(a)(\1)??", "d", "aa"),
        (r"(ab)((?:\1)){2,3}", "d", "abababab"),
        (r"(ab)((?:\1)){1,3}?", "d", "abababab"),
        (r"(a)((?:)\1(?:))*ab", "d", "qaaaab"),
        (r"(a)((?:)\1(?:))*?ab", "d", "qaaaab"),
        (r"((a)(\2)+)\1", "d", "aaaaaa"),
        (r"((a)((?:\2))+?)\1", "d", "aaaaaa"),
        (r"(a|ab)(\1)+b", "d", "ababb"),
        (r"(?:(a)|(ab))(\1)*(\2)+", "d", "abab"),
        (r"(a)(b)((\1))*((?:\2))*", "d", "abaaabbb"),
        (r"(a)(b)((\1))*?((?:\2))*?", "d", "abaaabbb"),
        (r"(\2)+(a)", "d", "a"),
        (r"(\1)+a\1", "d", "a"),
        (r"()(\1){2,3}", "d", "q"),
        (r"(a)(\1){999999999999999999999999999999}", "d", "aa"),
        (r"(?:(?<x>a)|(?<x>ab))(?<last>\k<x>)+b", "d", "ababb"),
        (r"(?<x>\k<x>)+", "dgi", "q"),
        (r"(?<x>a)(?<last>\k<x>)+\k<last>", "dy", "aaaa"),
        (r"^(?<x>.)(?<last>(?:)\k<x>(?:))+$", "dis", "µΜµ"),
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
fn captured_reference_retries_keep_final_ranges_and_name_aliases() {
    check(
        r"let a=/(?<x>a)(?<last>\k<x>)+\k<last>/d.exec('aaaa');a[0]==='aaaa'&&a.groups.last==='a'&&a.indices.groups.last===a.indices[2]&&a.indices[2][0]===2&&a.indices[2][1]===3",
    );
    check(
        r"let a=/((?<x>a)(?<last>\k<x>)+?)\1/d.exec('aaaaaa');a[0]==='aaaa'&&a[1]==='aa'&&a.groups.last==='a'&&a.indices.groups.last===a.indices[3]&&a.indices[3][0]===1&&a.indices[3][1]===2",
    );
    check(
        r"let a=/(?:(?<x>a)|(?<x>ab))(?<last>\k<x>)+b/d.exec('qabababb');a.index===1&&a[0]==='abababb'&&a[1]===undefined&&a.groups.x==='ab'&&a.groups.last==='ab'&&a.indices.groups.x===a.indices[2]&&a.indices.groups.last===a.indices[3]&&a.indices[3][0]===5&&a.indices[3][1]===7",
    );
    check(
        r"let a=/(?<x>a)(?<last>(\k<x>))*/d.exec('aaaa');a.groups.last==='a'&&a[3]==='a'&&a.indices[2][0]===3&&a.indices[3][0]===3&&a.indices.groups.last===a.indices[2]",
    );
    check(
        r"let a=/(?<x>.)(?<last>\k<x>)+/d.exec('\uD800\uD800\uD800');a[0].length===3&&a.groups.last.charCodeAt(0)===0xD800&&a.indices.groups.last===a.indices[2]&&a.indices[2][0]===2&&a.indices[2][1]===3",
    );
}

#[test]
fn captured_consumers_and_zero_iterations_keep_sources_and_aliases() {
    check(
        r"let r=/(?<x>a)(?<last>\k<x>)+/dg,a=[...'aaa aa'.matchAll(r)];a.length===2&&a[0][0]==='aaa'&&a[1][0]==='aa'&&a[0].groups.last==='a'&&a[0].indices.groups.last===a[0].indices[2]&&a[0].indices[2][0]===2&&r.lastIndex===0",
    );
    check(
        r"'qaaaa'.replace(/(?<x>a)(?<last>\k<x>)+/,'<$<last>>')==='q<a>'&&'qaaaa'.search(/(a)(\1)+/)===1&&'qaaaaZ'.split(/(a)(\1)+/).join(',')==='q,a,a,Z'",
    );
    check(
        r"let r=/(?<x>a)(?<last>\k<x>)+/dy;r.lastIndex=1;let copy=new RegExp(r),a=r.exec('qaaa');a[0]==='aaa'&&a.indices[2][0]===3&&r.lastIndex===4&&r.exec('qaaa')===null&&r.lastIndex===0&&copy.source===r.source&&copy.lastIndex===0",
    );
    check(
        r"let a=[...'q'.matchAll(/(?<x>)(?<last>\k<x>)*/dg)],b=/(?<self>\k<self>)+/d.exec('q');a.length===2&&a[0].groups.last===undefined&&a[0].indices.groups.last===undefined&&a[1].index===1&&b.groups.self===''&&b.indices.groups.self===b.indices[1]",
    );
}

#[test]
fn deep_captured_wrappers_and_empty_minimums_copy_and_collect_with_unlimited_defaults() {
    let mut realm = Realm::default();
    assert_eq!(realm.eval("let huge=new RegExp('(?<x>)(?<last>\\\\k<x>){'+'9'.repeat(10000)+'}','d'),empty=huge.exec('q');empty[0]===''&&empty.groups.last===''&&empty.indices.groups.last===empty.indices[2]"),Ok(Value::Boolean(true)));
    realm.eval("let r=new RegExp('(?<x>a)(?<last>'+'('.repeat(99999)+'\\\\k<x>'+')'.repeat(100000)+'+','d'),copy=new RegExp(r)").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("let a=copy.exec('aaaa');a[0]==='aaaa'&&a.length===100002&&a.groups.last==='a'&&a[100001]==='a'&&a.indices[100001][0]===3&&a.indices.groups.last===a.indices[2]&&copy.source===r.source"),Ok(Value::Boolean(true)));
    realm.collect(usize::MAX).unwrap();
}

#[test]
fn explicit_captured_reference_work_aborts_before_last_index_and_handlers() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval("let marker=0,r=/(?<x>a)(?<last>\\k<x>)*b/g;r.lastIndex=1;let text='a'.repeat(10000)")
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
fn composite_reference_repetitions_remain_unsupported_and_plain_failures_return_null() {
    for source in [
        r"/(?:(?:(?:(a)(\1\1)+){2})|){2,3}/.test('aa')",
        r"/(?:(?:(?:(a)(a\1)+){2})|){2,3}/.test('aa')",
        r"/(a)(\1|b){2,3}/.test('aa')",
        r"/(a)(\1*){2,3}/.test('aa')",
        r"/(a)(\1(?=a)){2,3}/.test('aa')",
        r"/(a)(\1)+/u.test('aa')",
    ] {
        assert!(
            matches!(
                Realm::default().eval(source),
                Err(Error::Unsupported { .. })
            ),
            "{source}"
        );
    }
    check(r"/(a)(\1)+/.exec('ab')===null");
}
