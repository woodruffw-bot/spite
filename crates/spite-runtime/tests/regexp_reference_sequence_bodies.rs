//! Capture-free reference sequences retain fixed external targets.

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
fn reference_sequence_body_result_snapshot() {
    let mut rows = String::new();
    for (source, flags, text) in [
        (r"(a)(b)(?:\1\2)*", "d", "ababab"),
        (r"(a)(b)(?:\1\2)*?", "d", "ababab"),
        (r"(a)(b)(?:\1\2)+", "d", "ab"),
        (r"(a)(b)(?:\1\2)+?", "d", "ababab"),
        (r"(a)(b)(?:\1\2)?", "d", "abab"),
        (r"(a)(b)(?:\1\2)??", "d", "abab"),
        (r"(a)(bb)(?:\2\1){1,3}", "d", "abbabbabba"),
        (r"(a)(bb)(?:\2\1){1,3}?", "d", "abbabbabba"),
        (r"(a)(b)(?:\1\2)*ab", "d", "qababab"),
        (r"(a)(b)(?:\1\2)*?ab", "d", "qababab"),
        (r"((a)(b)(?:\2\3)+)\1", "d", "abababab"),
        (r"((a)(b)(?:\2\3)+?)\1", "d", "abababab"),
        (r"(?:(a)|(b))(?:\1\2)+c", "d", "bbc"),
        (r"(?:(a)|(b))(?:\2\1)+c", "d", "bbc"),
        (r"(a)(b)(?:\1\2)*(?:\2\1)*", "d", "ababbaba"),
        (r"(a)(b)(?:\1\2)*?(?:\2\1)*?", "d", "ababbaba"),
        (r"(?:\1\2)+(a)(b)", "d", "ab"),
        (r"((?:\1\2)+a)(b)", "d", "ab"),
        (r"()()(?:\1\2){2,3}", "d", "q"),
        (
            r"(a)(b)(?:\1\2){999999999999999999999999999999}",
            "d",
            "abab",
        ),
        (r"(?:(?<x>a)|(?<x>b))(?<y>c)(?:\k<x>\k<y>)+d", "d", "bcbcd"),
        (r"(?<x>(?:\k<x>\k<y>)+a)(?<y>b)", "dgi", "ab"),
        (r"(?<x>a)(?:\1\k<x>)+", "dy", "aaaaa"),
        (r"^(?<x>.)(?<y>a)(?:\k<x>\k<y>)+$", "dis", "µaΜAµa"),
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
fn reference_sequence_retries_keep_prefix_ranges_and_original_aliases() {
    check(
        r"let a=/(?<x>a)(?<y>b)(?:\k<x>\k<y>)*ab$/d.exec('ababab');a[0]==='ababab'&&a.groups.x==='a'&&a.groups.y==='b'&&a.indices.groups.x===a.indices[1]&&a.indices.groups.y===a.indices[2]&&a.indices[1][0]===0&&a.indices[2][0]===1",
    );
    check(
        r"let a=/((?<x>a)(?<y>b)(?:\k<x>\k<y>)+?)\1/d.exec('abababab');a[0]==='abababab'&&a[1]==='abab'&&a.indices[1][1]===4&&a.indices.groups.x===a.indices[2]&&a.indices.groups.y===a.indices[3]",
    );
    check(
        r"let a=/(?:(?<x>a)|(?<x>b))(?<y>c)(?:\k<x>\k<y>)+d/d.exec('bcbcd');a[0]==='bcbcd'&&a[1]===undefined&&a.groups.x==='b'&&a.groups.y==='c'&&a.indices.groups.x===a.indices[2]&&a.indices.groups.y===a.indices[3]",
    );
    check(
        r"let a=/(?<x>a)(?:\1\k<x>)+/d.exec('aaaaa');a[0]==='aaaaa'&&a.groups.x==='a'&&a.indices.groups.x===a.indices[1]&&/^(?<x>a)(?:\1\k<x>)+$/.exec('aaaa')===null",
    );
    check(
        r"let a=/(?<x>.)(?<y>a)(?:\k<x>\k<y>)+/d.exec('\uD800a\uD800a');a[0].length===4&&a.groups.x.charCodeAt(0)===0xD800&&a.indices.groups.x===a.indices[1]&&a.indices.groups.y===a.indices[2]&&a.indices[2][0]===1",
    );
}

#[test]
fn reference_sequence_consumers_and_empty_targets_keep_original_sources() {
    check(
        r"let r=/(?<x>a)(?<y>b)(?:\k<x>\k<y>)+/dg,a=[...'abab ababab'.matchAll(r)];a.length===2&&a[0][0]==='abab'&&a[1][0]==='ababab'&&a[0].groups.x==='a'&&a[0].indices.groups.x===a[0].indices[1]&&a[1].indices.groups.y===a[1].indices[2]&&r.lastIndex===0",
    );
    check(
        r"'qababab'.replace(/(?<x>a)(?<y>b)(?:\k<x>\k<y>)+/,'<$<x>:$<y>>')==='q<a:b>'&&'qababab'.search(/(a)(b)(?:\1\2)+/)===1&&'qabababZ'.split(/(a)(b)(?:\1\2)+/).join(',')==='q,a,b,Z'",
    );
    check(
        r"let r=/(?<x>a)(?<y>b)(?:\k<x>\k<y>)+/dy;r.lastIndex=1;let copy=new RegExp(r),a=r.exec('qababab');a[0]==='ababab'&&r.lastIndex===7&&r.exec('qababab')===null&&r.lastIndex===0&&copy.source===r.source&&copy.lastIndex===0",
    );
    check(
        r"let a=[...'q'.matchAll(/(?<x>)(?<y>)(?:\k<x>\k<y>)+/dg)],b=/(?<x>(?:\k<x>\k<y>)+a)(?<y>b)/d.exec('ab');a.length===2&&a[0].groups.x===''&&a[0].groups.y===''&&a[1].index===1&&b.groups.x==='a'&&b.groups.y==='b'&&b.indices.groups.x===b.indices[1]",
    );
}

#[test]
fn long_reference_sequences_and_empty_minimums_copy_and_collect_unlimited() {
    let mut realm = Realm::default();
    assert_eq!(realm.eval("let huge=new RegExp('(?<x>)(?<y>)(?:\\\\k<x>\\\\k<y>){'+'9'.repeat(10000)+'}','d'),empty=huge.exec('q');empty[0]===''&&empty.groups.x===''&&empty.groups.y===''&&empty.indices.groups.x===empty.indices[1]&&empty.indices.groups.y===empty.indices[2]"),Ok(Value::Boolean(true)));
    realm.eval("let r=new RegExp('(?<x>a)(?<y>b)(?:'+'\\\\k<x>\\\\k<y>'.repeat(50000)+')+','d'),copy=new RegExp(r)").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("let a=copy.exec('ab'.repeat(50001));a[0].length===100002&&a.groups.x==='a'&&a.groups.y==='b'&&a.indices[1][0]===0&&a.indices[2][0]===1&&a.indices.groups.x===a.indices[1]&&copy.source===r.source"),Ok(Value::Boolean(true)));
    realm.collect(usize::MAX).unwrap();
}

#[test]
fn explicit_sequence_work_aborts_before_last_index_and_handlers() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm.eval(r"let marker=0,r=/(?<x>a)(?<y>b)(?:\k<x>\k<y>)*c/g;r.lastIndex=1;let text='ab'.repeat(5000)").unwrap();
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
fn capture_effects_and_other_sequence_bodies_remain_unsupported() {
    for source in [
        r"/(?:(?:(?:(a)(b)(\1\2)+){2})|)*/.test('abab')",
        r"/(?:(?:(?:(a)(b)(?:()\1\2)+){2})|)*/.test('abab')",
        r"/(?:(?:(?:(a)(b)(?:a\1\2)+){2})|)*/.test('abab')",
        r"/(a)(b)(?:\1|\2)+/.test('abab')",
        r"/(a)(b)(?:\1+\2)+/.test('abab')",
        r"/(a)(b)(?:\1\2)+/u.test('abab')",
    ] {
        assert!(
            matches!(
                Realm::default().eval(source),
                Err(Error::Unsupported { .. })
            ),
            "{source}"
        );
    }
    check(r"/(a)(b)(?:\1\2)+/.exec('abc')===null");
}
