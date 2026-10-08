//! Repeated copies of one reference retain whole-body repetition counts.

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
fn repeated_reference_body_result_snapshot() {
    let mut rows = String::new();
    for (source, flags, text) in [
        (r"(a)(?:\1\1)*", "d", "aaaa"),
        (r"(a)(?:\1\1)*?", "d", "aaaa"),
        (r"(a)(?:\1\1)+", "d", "a"),
        (r"(a)(?:\1\1)+?", "d", "aaaa"),
        (r"(a)(?:\1\1)?", "d", "aa"),
        (r"(a)(?:\1\1)??", "d", "aa"),
        (r"(ab)(?:\1\1){2,3}", "d", "abababababab"),
        (r"(ab)(?:\1\1){1,3}?", "d", "abababababab"),
        (r"(a)(?:\1\1)*ab", "d", "qaaaab"),
        (r"(a)(?:\1\1)*?ab", "d", "qaaaab"),
        (r"((a)(\2\2)+)\1", "d", "aaaaaa"),
        (r"((a)(\2\2)+?)\1", "d", "aaaaaa"),
        (r"(a|ab)(?:\1\1)+b", "d", "abababb"),
        (r"(?:(a)|(ab))(?:\1\1)*(?:\2\2)+", "d", "ababab"),
        (r"(a)(b)(?:\1\1)*(?:\2\2)*", "d", "abaaaabbbb"),
        (r"(a)(b)(?:\1\1)*?(?:\2\2)*?", "d", "abaaaabbbb"),
        (r"(?:\1\1)+(a)", "d", "a"),
        (r"(\1\1)+a\1", "d", "a"),
        (r"()(?:\1\1){2,3}", "d", "q"),
        (r"(a)(?:\1\1){999999999999999999999999999999}", "d", "aa"),
        (r"(?:(?<x>a)|(?<x>ab))(?<last>\k<x>\k<x>)+b", "d", "abababb"),
        (r"(?<x>\k<x>\k<x>)+", "dgi", "q"),
        (
            r"(?<x>a)(?<last>(?<before>)\k<x>\k<x>(?<after>))+",
            "dy",
            "aaaaa",
        ),
        (r"^(?<x>.)(?:\k<x>\k<x>\k<x>)+$", "dis", "µΜµΜ"),
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
fn whole_reference_body_retries_keep_original_named_ranges_and_aliases() {
    check(
        r"let a=/(?<x>a)(?<last>\k<x>\k<x>)+\k<last>/d.exec('aaaaaa');a[0]==='aaaaa'&&a.groups.last==='aa'&&a.indices.groups.last===a.indices[2]&&a.indices[2][0]===1&&a.indices[2][1]===3",
    );
    check(
        r"let a=/((?<x>a)(?<last>\k<x>\k<x>)+?)\1/d.exec('aaaaaa');a[0]==='aaaaaa'&&a[1]==='aaa'&&a.groups.last==='aa'&&a.indices.groups.last===a.indices[3]&&a.indices[3][0]===1&&a.indices[3][1]===3",
    );
    check(
        r"let a=/(?:(?<x>a)|(?<x>ab))(?<last>\k<x>\k<x>)+b/d.exec('qabababb');a.index===1&&a[0]==='abababb'&&a[1]===undefined&&a.groups.x==='ab'&&a.groups.last==='abab'&&a.indices.groups.x===a.indices[2]&&a.indices.groups.last===a.indices[3]&&a.indices[3][0]===3&&a.indices[3][1]===7",
    );
    check(
        r"let a=/(?<x>a)(?<last>(?<before>)\k<x>\k<x>(?<after>))+/d.exec('aaaaa');a.groups.last==='aa'&&a.groups.before===''&&a.groups.after===''&&a.indices.groups.before===a.indices[3]&&a.indices[3][0]===3&&a.indices.groups.after===a.indices[4]&&a.indices[4][0]===5&&/^(a)(?:\1\1)+$/.exec('aaaa')===null",
    );
    check(
        r"let a=/(?<x>.)(?<last>\k<x>\k<x>)+/d.exec('\uD800\uD800\uD800');a[0].length===3&&a.groups.last.length===2&&a.indices.groups.last===a.indices[2]&&a.indices[2][0]===1&&a.indices[2][1]===3",
    );
}

#[test]
fn repeated_reference_body_consumers_and_empty_targets_keep_source() {
    check(
        r"let r=/(?<x>a)(?<last>\k<x>\k<x>)+/dg,a=[...'aaaaa aaa'.matchAll(r)];a.length===2&&a[0][0]==='aaaaa'&&a[1][0]==='aaa'&&a[0].groups.last==='aa'&&a[0].indices.groups.last===a[0].indices[2]&&a[0].indices[2][0]===3&&r.lastIndex===0",
    );
    check(
        r"'qaaaaa'.replace(/(?<x>a)(?<last>\k<x>\k<x>)+/,'<$<last>>')==='q<aa>'&&'qaaaaa'.search(/(a)(?:\1\1)+/)===1&&'qaaaaaZ'.split(/(a)(\1\1)+/).join(',')==='q,a,aa,Z'",
    );
    check(
        r"let r=/(?<x>a)(?<last>\k<x>\k<x>)+/dy;r.lastIndex=1;let copy=new RegExp(r),a=r.exec('qaaaaa');a[0]==='aaaaa'&&a.indices[2][0]===4&&r.lastIndex===6&&r.exec('qaaaaa')===null&&r.lastIndex===0&&copy.source===r.source&&copy.lastIndex===0",
    );
    check(
        r"let a=[...'q'.matchAll(/(?<x>)(?<last>\k<x>\k<x>)*/dg)],b=/(?<self>\k<self>\k<self>)+/d.exec('q');a.length===2&&a[0].groups.last===undefined&&a[1].index===1&&b.groups.self===''&&b.indices.groups.self===b.indices[1]",
    );
}

#[test]
fn large_reference_bodies_and_huge_empty_minimums_copy_and_collect_unlimited() {
    let mut realm = Realm::default();
    assert_eq!(realm.eval("let huge=new RegExp('(?<x>)(?<last>\\\\k<x>\\\\k<x>){'+'9'.repeat(10000)+'}','d'),empty=huge.exec('q');empty[0]===''&&empty.groups.last===''&&empty.indices.groups.last===empty.indices[2]"),Ok(Value::Boolean(true)));
    realm.eval("let r=new RegExp('(?<x>a)(?<last>'+'\\\\k<x>'.repeat(100000)+')+','d'),copy=new RegExp(r)").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("let a=copy.exec('a'.repeat(100001));a[0].length===100001&&a.groups.last.length===100000&&a.indices[2][0]===1&&a.indices[2][1]===100001&&a.indices.groups.last===a.indices[2]&&copy.source===r.source"),Ok(Value::Boolean(true)));
    realm.collect(usize::MAX).unwrap();
}

#[test]
fn explicit_reference_body_work_aborts_before_last_index_and_handlers() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm.eval(r"let marker=0,r=/(?<x>a)(?<last>\k<x>\k<x>)*b/g;r.lastIndex=1;let text='a'.repeat(10000)").unwrap();
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
fn other_reference_bodies_remain_unsupported_and_plain_failures_return_null() {
    for source in [
        r"/(?:(a)(b)(?:\1\2)+){2}/.test('abab')",
        r"/(?:(a)(?:a\1\1)+){2}/.test('aaa')",
        r"/(a)(?:\1|\1)+/.test('aaa')",
        r"/(a)(?:\1+\1)+/.test('aaa')",
        r"/(?:(a)(?:\1()\1)+){2}/.test('aaa')",
        r"/(a)(?:\1\1)+/u.test('aaa')",
    ] {
        assert!(
            matches!(
                Realm::default().eval(source),
                Err(Error::Unsupported { .. })
            ),
            "{source}"
        );
    }
    check(r"/(a)(?:\1\1)+/.exec('ab')===null");
}
