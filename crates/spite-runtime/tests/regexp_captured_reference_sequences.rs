//! Captured reference sequences preserve final-iteration ranges.

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
fn captured_reference_sequence_result_snapshot() {
    let mut rows = String::new();
    for (source, flags, text) in [
        (r"(a)(b)(\1\2)*", "d", "ababab"),
        (r"(a)(b)(\1\2)*?", "d", "ababab"),
        (r"(a)(b)(\1\2)+", "d", "ab"),
        (r"(a)(b)(\1\2)+?", "d", "ababab"),
        (r"(a)(b)(\1\2)?", "d", "abab"),
        (r"(a)(b)(\1\2)??", "d", "abab"),
        (r"(a)(bb)(\2\1){1,3}", "d", "abbabbabba"),
        (r"(a)(bb)(\2\1){1,3}?", "d", "abbabbabba"),
        (r"(a)(b)(()\1\2())*ab", "d", "qababab"),
        (r"(a)(b)(()\1\2())*?ab", "d", "qababab"),
        (r"((a)(b)(\2\3)+)\1", "d", "abababab"),
        (r"((a)(b)(\2\3)+?)\1", "d", "abababab"),
        (r"(?:(a)|(b))(\1\2)+c", "d", "bbc"),
        (r"(?:(a)|(b))(\2\1)+c", "d", "bbc"),
        (r"(a)(b)(\1\2)*(\2\1)*", "d", "ababbaba"),
        (r"(a)(b)(\1\2)*?(\2\1)*?", "d", "ababbaba"),
        (r"(\2\3)+(a)(b)", "d", "ab"),
        (r"((\1\3)+a)(b)", "d", "ab"),
        (r"()()(\1\2){2,3}", "d", "q"),
        (r"(a)(b)(\1\2){999999999999999999999999999999}", "d", "abab"),
        (
            r"(?:(?<x>a)|(?<x>b))(?<y>c)(?<last>(?<before>)\k<x>\k<y>(?<after>))+d",
            "d",
            "bcbcd",
        ),
        (r"(?<x>(\k<x>\k<y>)+a)(?<y>b)", "dgi", "ab"),
        (r"(?<x>a)(?<last>\1\k<x>)+", "dy", "aaaaa"),
        (
            r"^(?<x>.)(?<y>a)(?<last>(?<before>)\k<x>\k<y>(?<after>))+$",
            "dis",
            "µaΜAµa",
        ),
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
fn captured_sequence_retries_keep_final_named_ranges_and_empty_positions() {
    check(
        r"let a=/(?<x>a)(?<y>b)(?<last>(?<before>)\k<x>\k<y>(?<after>))*ab$/d.exec('ababab');a[0]==='ababab'&&a.groups.last==='ab'&&a.indices.groups.last===a.indices[3]&&a.indices[3][0]===2&&a.indices[3][1]===4&&a.indices.groups.before===a.indices[4]&&a.indices[4][0]===2&&a.indices.groups.after===a.indices[5]&&a.indices[5][0]===4",
    );
    check(
        r"let a=/((?<x>a)(?<y>b)(?<last>\k<x>\k<y>)+?)\1/d.exec('abababab');a[0]==='abababab'&&a[1]==='abab'&&a.groups.last==='ab'&&a.indices[1][1]===4&&a.indices.groups.last===a.indices[4]&&a.indices[4][0]===2&&a.indices[4][1]===4",
    );
    check(
        r"let a=/(?:(?<x>a)|(?<x>b))(?<y>c)(?<last>(?<before>)\k<x>\k<y>(?<after>))+d/d.exec('bcbcd');a[0]==='bcbcd'&&a[1]===undefined&&a.groups.x==='b'&&a.groups.last==='bc'&&a.indices.groups.last===a.indices[4]&&a.indices.groups.before===a.indices[5]&&a.indices[5][0]===2&&a.indices.groups.after===a.indices[6]&&a.indices[6][0]===4",
    );
    check(
        r"let a=/(?<x>a)(?<last>\1\k<x>)+/d.exec('aaaaa');a[0]==='aaaaa'&&a.groups.last==='aa'&&a.indices.groups.last===a.indices[2]&&a.indices[2][0]===3&&a.indices[2][1]===5&&/^(?<x>a)(?<last>\1\k<x>)+$/.exec('aaaa')===null",
    );
    check(
        r"let a=/(?<x>.)(?<y>a)(?<last>\k<x>\k<y>)+/d.exec('\uD800a\uD800a');a[0].length===4&&a.groups.last.charCodeAt(0)===0xD800&&a.indices.groups.last===a.indices[3]&&a.indices[3][0]===2&&a.indices[3][1]===4",
    );
}

#[test]
fn captured_sequence_consumers_and_zero_iterations_keep_original_sources() {
    check(
        r"let r=/(?<x>a)(?<y>b)(?<last>\k<x>\k<y>)+/dg,a=[...'abab ababab'.matchAll(r)];a.length===2&&a[0][0]==='abab'&&a[1][0]==='ababab'&&a[0].groups.last==='ab'&&a[0].indices.groups.last===a[0].indices[3]&&a[0].indices[3][0]===2&&r.lastIndex===0",
    );
    check(
        r"'qababab'.replace(/(?<x>a)(?<y>b)(?<last>\k<x>\k<y>)+/,'<$<last>>')==='q<ab>'&&'qababab'.search(/(a)(b)(\1\2)+/)===1&&'qabababZ'.split(/(a)(b)(\1\2)+/).join(',')==='q,a,b,ab,Z'",
    );
    check(
        r"let r=/(?<x>a)(?<y>b)(?<last>\k<x>\k<y>)+/dy;r.lastIndex=1;let copy=new RegExp(r),a=r.exec('qababab');a[0]==='ababab'&&a.indices[3][0]===5&&r.lastIndex===7&&r.exec('qababab')===null&&r.lastIndex===0&&copy.source===r.source&&copy.lastIndex===0",
    );
    check(
        r"let a=[...'q'.matchAll(/(?<x>)(?<y>)(?<last>\k<x>\k<y>)*/dg)],b=/(?<x>(\k<x>\k<y>)+a)(?<y>b)/d.exec('ab');a.length===2&&a[0].groups.last===undefined&&a[0].indices.groups.last===undefined&&a[1].index===1&&b.groups.x==='a'&&b.groups.y==='b'&&b[2]===''&&b.indices[2][0]===0",
    );
}

#[test]
fn deep_captured_sequences_and_empty_minimums_copy_and_collect_unlimited() {
    let mut realm = Realm::default();
    assert_eq!(realm.eval("let huge=new RegExp('(?<x>)(?<y>)(?<last>(?<before>)\\\\k<x>\\\\k<y>(?<after>)){'+'9'.repeat(10000)+'}','d'),empty=huge.exec('q');empty[0]===''&&empty.groups.last===''&&empty.groups.before===''&&empty.groups.after===''&&empty.indices.groups.before===empty.indices[4]&&empty.indices.groups.after===empty.indices[5]"),Ok(Value::Boolean(true)));
    realm.eval("let r=new RegExp('(?<x>a)(?<y>b)(?<last>'+'('.repeat(99999)+'\\\\k<x>\\\\k<y>'+')'.repeat(100000)+'+','d'),copy=new RegExp(r)").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("let a=copy.exec('ababab');a[0]==='ababab'&&a.length===100003&&a.groups.last==='ab'&&a[100002]==='ab'&&a.indices[100002][0]===4&&a.indices[100002][1]===6&&a.indices.groups.last===a.indices[3]&&copy.source===r.source"),Ok(Value::Boolean(true)));
    realm.collect(usize::MAX).unwrap();
}

#[test]
fn explicit_captured_sequence_work_aborts_before_last_index_and_handlers() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm.eval(r"let marker=0,r=/(?<x>a)(?<y>b)(?<last>(?<before>)\k<x>\k<y>(?<after>))*c/g;r.lastIndex=1;let text='ab'.repeat(5000)").unwrap();
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
fn partial_capture_effects_and_other_sequence_bodies_remain_unsupported() {
    for source in [
        r"/(?:(?:(?:(a)(b)((\1)\2)+){2})|)*/.test('abab')",
        r"/(?:(?:(?:(a)(b)(\1()\2)+){2})|)*/.test('abab')",
        r"/(?:(?:(?:(a)(b)(a\1\2)+){2})|)*/.test('abab')",
        r"/(a)(b)(\1|\2)+/.test('abab')",
        r"/(a)(b)(\1+\2)+/.test('abab')",
        r"/(a)(b)(\1\2)+/u.test('abab')",
    ] {
        assert!(
            matches!(
                Realm::default().eval(source),
                Err(Error::Unsupported { .. })
            ),
            "{source}"
        );
    }
    check(r"/(a)(b)(\1\2)+/.exec('abc')===null");
}
