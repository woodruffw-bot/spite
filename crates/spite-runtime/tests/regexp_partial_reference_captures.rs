//! Partial captures in repeated reference bodies retain final-iteration ranges.

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
fn partial_reference_capture_result_snapshot() {
    let mut rows = String::new();
    for (source, flags, text) in [
        (r"(a)((\1)\1)*", "d", "aaaaa"),
        (r"(a)((\1)\1)*?", "d", "aaaaa"),
        (r"(a)(b)((\1)\2)+", "d", "ababab"),
        (r"(a)(b)(\1(\2))+?", "d", "ababab"),
        (r"(a)(b)((\1)\2)?", "d", "abab"),
        (r"(a)(b)(\1(\2))??", "d", "abab"),
        (r"(a)(bb)((\2)\1){1,3}", "d", "abbbbabba"),
        (r"(a)(bb)(\2(\1)){1,3}?", "d", "abbbbabba"),
        (r"(a)(b)(()(\1)()\2())*ab", "d", "qababab"),
        (r"(a)(b)(()\1()(\2)())*?ab", "d", "qababab"),
        (r"((a)(b)((\2)\3)+)\1", "d", "abababab"),
        (r"((a)(b)(\2(\3))+?)\1", "d", "abababab"),
        (r"(?:(a)|(b))((\1)\2)+c", "d", "bbc"),
        (r"(?:(a)|(b))((\2)\1)+c", "d", "bbc"),
        (r"(a)(b)((\1)\2)*\4", "d", "ababa"),
        (r"(a)(b)(\1(\2))*?\4", "d", "ababb"),
        (r"(a)(b)(\4(\1)\2)+", "d", "ababab"),
        (r"(a)(b)((\4\1)\2)+", "d", "ababab"),
        (r"()()((\1)\2){2,3}", "d", "q"),
        (
            r"(a)(b)((\1)\2){999999999999999999999999999999}",
            "d",
            "abab",
        ),
        (
            r"(?:(?<x>a)|(?<x>b))(?<y>c)(?<last>(?<piece>\k<x>)\k<y>)+d",
            "d",
            "bcbcd",
        ),
        (
            r"(?<x>a)(?<y>b)(?<last>\k<piece>(?<piece>\k<x>)\k<y>)+",
            "dgi",
            "ababab",
        ),
        (r"(?<x>a)(?<last>(?<piece>\1)\k<x>)+", "dy", "aaaaa"),
        (
            r"^(?<x>.)(?<y>a)(?<last>(?<piece>\k<x>)\k<y>)+$",
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
fn partial_named_ranges_restore_after_whole_body_retries() {
    check(
        r"let a=/(?<x>a)(?<y>b)(?<last>(?<piece>\k<x>)\k<y>)*ab$/d.exec('ababab');a[0]==='ababab'&&a.groups.last==='ab'&&a.groups.piece==='a'&&a.indices.groups.last===a.indices[3]&&a.indices.groups.last[0]===2&&a.indices.groups.piece===a.indices[4]&&a.indices[4][0]===2&&a.indices[4][1]===3",
    );
    check(
        r"let a=/((?<x>a)(?<y>b)(?<last>\k<x>(?<piece>\k<y>))+?)\1/d.exec('abababab');a[0]==='abababab'&&a[1]==='abab'&&a.groups.piece==='b'&&a.indices.groups.piece===a.indices[5]&&a.indices[5][0]===3&&a.indices[5][1]===4",
    );
    check(
        r"let a=/(?<x>a)(?<y>bb)(?<last>\k<y>(?<piece>\k<x>))+$/d.exec('abbbbabba');a.groups.last==='bba'&&a.groups.piece==='a'&&a.indices.groups.piece===a.indices[4]&&a.indices[4][0]===8&&a.indices[4][1]===9",
    );
    check(
        r"let a=/(?:(?<x>a)|(?<x>b))(?<last>(?<piece>\1)\2)+c/d.exec('bbc');a[0]==='bbc'&&a[1]===undefined&&a.groups.x==='b'&&a.groups.piece===''&&a.indices.groups.piece===a.indices[4]&&a.indices[4][0]===1&&a.indices[4][1]===1",
    );
    check(
        r"let a=/(?<x>a)(?<y>b)(?<last>\k<piece>(?<piece>\k<x>)\k<y>)+/d.exec('ababab'),b=/(?<x>a)(?<y>b)(?<last>(?<piece>\k<piece>\k<x>)\k<y>)+/d.exec('ababab');a.groups.piece==='a'&&b.groups.piece==='a'&&a.indices.groups.piece[0]===4&&b.indices.groups.piece[1]===5",
    );
    check(
        r"let a=/(?<x>.)(?<y>a)(?<last>(?<piece>\k<x>)\k<y>)+/d.exec('\uD800a\uD800a');a[0].length===4&&a.groups.piece.charCodeAt(0)===0xD800&&a.indices.groups.piece===a.indices[4]&&a.indices[4][0]===2&&a.indices[4][1]===3",
    );
}

#[test]
fn partial_capture_consumers_and_external_following_references_keep_ranges() {
    check(
        r"let r=/(?<x>a)(?<y>b)(?<last>(?<piece>\k<x>)\k<y>)+/dg,a=[...'abab ababab'.matchAll(r)];a.length===2&&a[0].groups.piece==='a'&&a[0].indices.groups.piece===a[0].indices[4]&&a[0].indices[4][0]===2&&a[1].indices[4][0]===9&&r.lastIndex===0",
    );
    check(
        r"'qababab'.replace(/(?<x>a)(?<y>b)(?<last>(?<piece>\k<x>)\k<y>)+/,'<$<last>:$<piece>>')==='q<ab:a>'&&'qababab'.search(/(a)(b)((\1)\2)+/)===1&&'qabababZ'.split(/(a)(b)((\1)\2)+/).join(',')==='q,a,b,ab,a,Z'",
    );
    check(
        r"let r=/(?<x>a)(?<y>b)(?<last>(?<piece>\k<x>)\k<y>)+/dy;r.lastIndex=1;let copy=new RegExp(r),a=r.exec('qababab');a.indices.groups.piece[0]===5&&a.indices.groups.piece[1]===6&&r.lastIndex===7&&r.exec('qababab')===null&&r.lastIndex===0&&copy.source===r.source&&copy.lastIndex===0",
    );
    check(
        r"let a=/(?<x>a)(?<y>b)(?<last>(?<piece>\k<x>)\k<y>)*\k<piece>$/d.exec('ababa'),b=/(?<x>a)(?<y>b)(?<last>\k<x>(?<piece>\k<y>))*?\k<piece>$/d.exec('ababb');a[0]==='ababa'&&a.groups.piece==='a'&&a.indices.groups.piece[0]===2&&b[0]==='ababb'&&b.groups.piece==='b'&&b.indices.groups.piece[0]===3",
    );
    check(
        r"let a=[...'q'.matchAll(/(?<x>)(?<y>)(?<last>(?<piece>\k<x>)\k<y>)*/dg)];a.length===2&&a[0].groups.piece===undefined&&a[0].indices.groups.piece===undefined&&a[1].index===1",
    );
}

#[test]
fn deep_partial_captures_copy_and_collect_with_default_limits() {
    let mut realm = Realm::default();
    assert_eq!(realm.eval("let huge=new RegExp('(?<x>)(?<y>)(?<last>(?<piece>\\\\k<x>)\\\\k<y>){'+'9'.repeat(10000)+'}','d'),empty=huge.exec('q');empty[0]===''&&empty.groups.last===''&&empty.groups.piece===''&&empty.indices.groups.piece===empty.indices[4]&&empty.indices[4][0]===0"),Ok(Value::Boolean(true)));
    realm.eval("let r=new RegExp('(?<x>a)(?<y>b)(?<last>(?<piece>'+'('.repeat(99999)+'\\\\k<x>'+')'.repeat(100000)+'\\\\k<y>)+','d'),copy=new RegExp(r)").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("let a=copy.exec('ababab');a[0]==='ababab'&&a.length===100004&&a.groups.last==='ab'&&a.groups.piece==='a'&&a[100003]==='a'&&a.indices[100003][0]===4&&a.indices[100003][1]===5&&a.indices.groups.piece===a.indices[4]&&copy.source===r.source"),Ok(Value::Boolean(true)));
    realm.collect(usize::MAX).unwrap();
}

#[test]
fn explicit_partial_capture_work_aborts_before_last_index_and_handlers() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm.eval(r"let marker=0,r=/(?<x>a)(?<y>b)(?<last>(?<piece>\k<x>)\k<y>)*c/g;r.lastIndex=1;let text='ab'.repeat(5000)").unwrap();
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
fn completed_local_targets_and_other_repeated_reference_bodies_remain_unsupported() {
    for source in [
        r"/(?:(?:(?:(a)(b)((\1)\4\2)+){2})|)*/.test('abab')",
        r"/(?:(?:(?:(?<x>a)(?<y>b)(?<last>(?<piece>\k<x>)\k<piece>\k<y>)+){2})|)*/.test('aabaab')",
        r"/(?:(?:(?:(a)(b)(a(\1)\2)+){2})|)*/.test('abab')",
        r"/(a)(b)((\1)|\2)+/.test('abab')",
        r"/(a)(b)((\1)+\2)+/.test('abab')",
        r"/(a)(b)((\1)\2)+/u.test('abab')",
    ] {
        assert!(
            matches!(
                Realm::default().eval(source),
                Err(Error::Unsupported { .. })
            ),
            "{source}"
        );
    }
    check(r"/(a)(b)((\1)\2)+/.exec('abc')===null");
}
