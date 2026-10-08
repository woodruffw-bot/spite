//! Empty captures around a reference preserve final-iteration positions.

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
fn empty_capture_quantified_reference_result_snapshot() {
    let mut rows = String::new();
    for (source, flags, text) in [
        (r"(a)(()\1)*", "d", "aaaa"),
        (r"(a)(()\1)*?", "d", "aaaa"),
        (r"(a)(\1())+", "d", "a"),
        (r"(a)(\1())+?", "d", "aaaa"),
        (r"(a)(()\1())?", "d", "aa"),
        (r"(a)(()\1())??", "d", "aa"),
        (r"(ab)(()(\1)()){2,3}", "d", "abababab"),
        (r"(ab)(()(\1)()){1,3}?", "d", "abababab"),
        (r"(a)(()\1())*ab", "d", "qaaaab"),
        (r"(a)(()\1())*?ab", "d", "qaaaab"),
        (r"((a)(()\2())+)\1", "d", "aaaaaa"),
        (r"((a)(()\2())+?)\1", "d", "aaaaaa"),
        (r"(a|ab)(()\1())+b", "d", "ababb"),
        (r"(?:(a)|(ab))(()\1())*(\2())+", "d", "abab"),
        (r"(a)(b)(()\1())*(\2())*", "d", "abaaabbb"),
        (r"(a)(b)(()\1())*?(\2())*?", "d", "abaaabbb"),
        (r"(()\4())+(a)", "d", "a"),
        (r"(()\2())+a\1", "d", "a"),
        (r"()(()\1()){2,3}", "d", "q"),
        (r"(a)(()\1()){999999999999999999999999999999}", "d", "aa"),
        (
            r"(?:(?<x>a)|(?<x>ab))(?<last>(?<before>)\k<x>(?<after>))+b",
            "d",
            "ababb",
        ),
        (r"(?<last>(?<x>)\k<x>(?<after>))+", "dgi", "q"),
        (r"(?<x>a)(?<last>(?<before>)\k<x>(?<after>))+", "dy", "aaaa"),
        (
            r"^(?<x>.)(?<last>(?<before>)\k<x>(?<after>))+$",
            "dis",
            "µΜµ",
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
fn empty_sibling_retries_keep_final_named_ranges_and_aliases() {
    check(
        r"let a=/(?<x>a)(?<last>(?<before>)\k<x>(?<after>))+\k<last>/d.exec('aaaa');a[0]==='aaaa'&&a.groups.last==='a'&&a.groups.before===''&&a.groups.after===''&&a.indices.groups.last===a.indices[2]&&a.indices[2][0]===2&&a.indices[2][1]===3&&a.indices.groups.before===a.indices[3]&&a.indices[3][0]===2&&a.indices[3][1]===2&&a.indices.groups.after===a.indices[4]&&a.indices[4][0]===3&&a.indices[4][1]===3",
    );
    check(
        r"let a=/((?<x>a)(?<last>(?<before>)\k<x>(?<after>))+?)\1/d.exec('aaaaaa');a[0]==='aaaa'&&a[1]==='aa'&&a.groups.last==='a'&&a.indices.groups.last===a.indices[3]&&a.indices[3][0]===1&&a.indices.groups.before===a.indices[4]&&a.indices[4][0]===1&&a.indices.groups.after===a.indices[5]&&a.indices[5][0]===2",
    );
    check(
        r"let a=/(?:(?<x>a)|(?<x>ab))(?<last>(?<before>)\k<x>(?<after>))+b/d.exec('qabababb');a.index===1&&a[0]==='abababb'&&a[1]===undefined&&a.groups.x==='ab'&&a.groups.last==='ab'&&a.indices.groups.x===a.indices[2]&&a.indices.groups.before===a.indices[4]&&a.indices[4][0]===5&&a.indices.groups.after===a.indices[5]&&a.indices[5][0]===7",
    );
    check(
        r"let a=/(?<last>(?<before>)\k<before>(?<after>))+/d.exec('q'),b=/(?<last>(?<before>)\k<before>(?<after>))*/d.exec('q');a.groups.last===''&&a.groups.before===''&&a.groups.after===''&&a.indices.groups.before===a.indices[2]&&a.indices[2][0]===0&&b.groups.last===undefined&&b.groups.before===undefined&&b.groups.after===undefined",
    );
    check(
        r"let a=/(?<x>.)(?<last>(?<before>)\k<x>(?<after>))+/d.exec('\uD800\uD800\uD800');a[0].length===3&&a.groups.last.charCodeAt(0)===0xD800&&a.indices.groups.before===a.indices[3]&&a.indices[3][0]===2&&a.indices.groups.after===a.indices[4]&&a.indices[4][0]===3",
    );
}

#[test]
fn empty_sibling_consumers_and_zero_iterations_keep_original_sources() {
    check(
        r"let r=/(?<x>a)(?<last>(?<before>)\k<x>(?<after>))+/dg,a=[...'aaa aa'.matchAll(r)];a.length===2&&a[0][0]==='aaa'&&a[1][0]==='aa'&&a[0].groups.before===''&&a[0].indices.groups.before===a[0].indices[3]&&a[0].indices[3][0]===2&&r.lastIndex===0",
    );
    check(
        r"'qaaaa'.replace(/(?<x>a)(?<last>(?<before>)\k<x>(?<after>))+/,'<$<last>>')==='q<a>'&&'qaaaa'.search(/(a)(()\1())+/)===1&&'qaaaaZ'.split(/(a)(()\1())+/).join(',')==='q,a,a,,,Z'",
    );
    check(
        r"let r=/(?<x>a)(?<last>(?<before>)\k<x>(?<after>))+/dy;r.lastIndex=1;let copy=new RegExp(r),a=r.exec('qaaa');a[0]==='aaa'&&a.indices[3][0]===3&&a.indices[4][0]===4&&r.lastIndex===4&&r.exec('qaaa')===null&&r.lastIndex===0&&copy.source===r.source&&copy.lastIndex===0",
    );
    check(
        r"let a=[...'q'.matchAll(/(?<last>(?<before>)\k<before>(?<after>))*/dg)];a.length===2&&a[0].groups.last===undefined&&a[0].groups.before===undefined&&a[0].groups.after===undefined&&a[1].index===1",
    );
}

#[test]
fn many_empty_siblings_and_huge_minimums_copy_and_collect_with_unlimited_defaults() {
    let mut realm = Realm::default();
    assert_eq!(realm.eval("let huge=new RegExp('(?<x>)(?<last>(?<before>)\\\\k<x>(?<after>)){'+'9'.repeat(10000)+'}','d'),empty=huge.exec('q');empty[0]===''&&empty.groups.last===''&&empty.groups.before===''&&empty.groups.after===''&&empty.indices.groups.before===empty.indices[3]&&empty.indices.groups.after===empty.indices[4]"),Ok(Value::Boolean(true)));
    realm.eval("let r=new RegExp('(?<x>a)(?<last>'+'()'.repeat(50000)+'\\\\k<x>'+'()'.repeat(50000)+')+','d'),copy=new RegExp(r)").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("let a=copy.exec('aaaa');a[0]==='aaaa'&&a.length===100003&&a.groups.last==='a'&&a[100002]===''&&a.indices[3][0]===3&&a.indices[100002][0]===4&&a.indices.groups.last===a.indices[2]&&copy.source===r.source"),Ok(Value::Boolean(true)));
    realm.collect(usize::MAX).unwrap();
}

#[test]
fn explicit_empty_sibling_work_aborts_before_last_index_and_handlers() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm.eval(r"let marker=0,r=/(?<x>a)(?<last>(?<before>)\k<x>(?<after>))*b/g;r.lastIndex=1;let text='a'.repeat(10000)").unwrap();
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
fn composite_reference_bodies_remain_unsupported_and_plain_failures_return_null() {
    for source in [
        r"/(?:(a)(()\1\1())+){2}/.test('aa')",
        r"/(?:(a)(()a\1())+){2}/.test('aa')",
        r"/(a)(()\1|b())+/.test('aa')",
        r"/(a)(()\1*())+/.test('aa')",
        r"/(a)(()\1(?=a))+/.test('aa')",
        r"/(a)(()\1())+/u.test('aa')",
    ] {
        assert!(
            matches!(
                Realm::default().eval(source),
                Err(Error::Unsupported { .. })
            ),
            "{source}"
        );
    }
    check(r"/(a)(()\1())+/.exec('ab')===null");
}
