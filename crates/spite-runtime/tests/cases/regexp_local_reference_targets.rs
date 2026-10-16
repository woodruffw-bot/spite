//! References to completed body captures use the current iteration input.

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
fn local_reference_target_result_snapshot() {
    let mut rows = String::new();
    for (source, flags, text) in [
        (r"(a)((\1)\3)*", "d", "aaaaa"),
        (r"(a)((\1)\3)*?", "d", "aaaaa"),
        (r"(a)(b)((\1)\4\2)+", "d", "abaabaab"),
        (r"(a)(b)(\1(\2)\4)+?", "d", "ababb"),
        (r"(a)(b)((\1)\4\2)?", "d", "abaab"),
        (r"(a)(b)(\1(\2)\4)??", "d", "ababb"),
        (r"(a)(bb)((\2)\4\1){1,3}", "d", "abbbbbbabbbba"),
        (r"(a)(bb)(\2(\1)\4){1,3}?", "d", "abbbababba"),
        (r"(a)(b)(()(\1)()\5\2())*ab", "d", "qabaabaab"),
        (r"(a)(b)(()\1()(\2)\6())*?ab", "d", "qababb"),
        (r"((a)(b)((\2)\5\3)+)\1", "d", "abaababaab"),
        (r"((a)(b)(\2(\3)\5)+?)\1", "d", "ababbababb"),
        (r"(?:(a)|(b))((\1)\4\2)+c", "d", "bbc"),
        (r"(?:(a)|(b))((\2)\4\1)+c", "d", "bbbc"),
        (r"(a)(b)((\1)\4\2)*\4", "d", "abaaba"),
        (r"(a)(b)(\1(\2)\4)*?\4", "d", "ababbb"),
        (r"(a)(b)(\4(\1)\4\2)+", "d", "abaabaab"),
        (r"(a)(b)((\4\1)\4\2)+", "d", "abaabaab"),
        (r"()()((\1)\4\2){2,3}", "d", "q"),
        (
            r"(a)(b)((\1)\4\2){999999999999999999999999999999}",
            "d",
            "abaab",
        ),
        (
            r"(?:(?<x>a)|(?<x>b))(?<y>c)(?<last>(?<piece>\k<x>)\k<piece>\k<y>)+d",
            "d",
            "bcbbcd",
        ),
        (
            r"(?<x>a)(?<y>b)(?<last>\k<piece>(?<piece>\k<x>)\k<piece>\k<y>)+",
            "dgi",
            "abaabaab",
        ),
        (r"(?<x>a)(?<last>(?<piece>\1)\k<piece>)+", "dy", "aaaaa"),
        (
            r"^(?<x>.)(?<y>a)(?<last>(?<piece>\k<x>)\k<piece>\k<y>)+$",
            "dis",
            "µaΜµAµΜa",
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
fn local_named_targets_use_current_iteration_ranges_and_restore_retries() {
    check(
        r"let a=/(?<x>a)(?<y>b)(?<last>(?<piece>\k<x>)\k<piece>\k<y>)*aab$/d.exec('abaabaab');a[0]==='abaabaab'&&a.groups.last==='aab'&&a.groups.piece==='a'&&a.indices.groups.last===a.indices[3]&&a.indices.groups.last[0]===2&&a.indices.groups.piece===a.indices[4]&&a.indices[4][0]===2&&a.indices[4][1]===3",
    );
    check(
        r"let a=/((?<x>a)(?<y>b)(?<last>(?<piece>\k<x>)\k<piece>\k<y>)+?)\1/d.exec('abaababaab');a[0]==='abaababaab'&&a[1]==='abaab'&&a.groups.last==='aab'&&a.groups.piece==='a'&&a.indices.groups.piece===a.indices[5]&&a.indices[5][0]===2",
    );
    check(
        r"let a=/(?<x>a)(?<y>b)(?<last>\k<x>(?<piece>\k<y>\k<x>)\k<piece>)+$/d.exec('abababa');a.groups.last==='ababa'&&a.groups.piece==='ba'&&a.indices.groups.piece===a.indices[4]&&a.indices[4][0]===3&&a.indices[4][1]===5",
    );
    check(
        r"let a=/(?:(?<x>a)|(?<x>b))(?<y>c)(?<last>(?<piece>\k<x>)\k<piece>\k<y>)+/d.exec('bcbbc');a[1]===undefined&&a.groups.last==='bbc'&&a.groups.piece==='b'&&a.indices.groups.piece===a.indices[5]&&a.indices[5][0]===2",
    );
    check(
        r"let a=/(?<x>a)(?<y>b)(?<last>(?<piece>\k<x>)\4\k<piece>\k<y>)+/d.exec('abaaab');a[0]==='abaaab'&&a.groups.last==='aaab'&&a.groups.piece==='a'&&a.indices.groups.piece===a.indices[4]&&a.indices[4][0]===2",
    );
    check(
        r"let a=/(?<x>.)(?<y>a)(?<last>(?<piece>\k<x>)\k<piece>\k<y>)+/d.exec('\uD800a\uD800\uD800a');a[0].length===5&&a.groups.piece.charCodeAt(0)===0xD800&&a.indices.groups.piece===a.indices[4]&&a.indices[4][0]===2&&a.indices[4][1]===3",
    );
}

#[test]
fn local_reference_consumers_and_following_captures_keep_original_numbering() {
    check(
        r"let r=/(?<x>a)(?<y>b)(?<last>(?<piece>\k<x>)\k<piece>\k<y>)+/dg,a=[...'abaab abaabaab'.matchAll(r)];a.length===2&&a[0].groups.last==='aab'&&a[0].indices.groups.piece===a[0].indices[4]&&a[0].indices[4][0]===2&&a[1].indices[4][0]===11&&r.lastIndex===0",
    );
    check(
        r"'qabaabaab'.replace(/(?<x>a)(?<y>b)(?<last>(?<piece>\k<x>)\k<piece>\k<y>)+/,'<$<last>:$<piece>>')==='q<aab:a>'&&'qabaabaab'.search(/(a)(b)((\1)\4\2)+/)===1&&'qabaabaabZ'.split(/(a)(b)((\1)\4\2)+/).join(',')==='q,a,b,aab,a,Z'",
    );
    check(
        r"let r=/(?<x>a)(?<y>b)(?<last>(?<piece>\k<x>)\k<piece>\k<y>)+/dy;r.lastIndex=1;let copy=new RegExp(r),a=r.exec('qabaabaab');a.indices.groups.piece[0]===6&&a.indices.groups.piece[1]===7&&r.lastIndex===9&&r.exec('qabaabaab')===null&&r.lastIndex===0&&copy.source===r.source&&copy.lastIndex===0",
    );
    check(
        r"let a=/(?<x>a)(?<y>b)(?<last>(?<piece>\k<x>)\k<piece>\k<y>)*\k<piece>$/d.exec('abaaba'),b=/(?<x>a)(?<y>b)(?<last>(?<piece>\k<x>)\k<piece>\k<y>)*?\k<piece>$/d.exec('abaaba');a[0]==='abaaba'&&b[0]==='abaaba'&&a.groups.piece==='a'&&b.groups.piece==='a'&&a.indices.groups.piece[0]===2&&b.indices.groups.piece[0]===2",
    );
    check(
        r"let a=[...'q'.matchAll(/(?<x>)(?<y>)(?<last>(?<piece>\k<x>)\k<piece>\k<y>)*/dg)];a.length===2&&a[0].groups.piece===undefined&&a[0].indices.groups.piece===undefined&&a[1].index===1",
    );
}

#[test]
fn many_local_references_empty_minimums_and_width_overflow_copy_and_collect() {
    check(
        r"let slash=String.fromCharCode(92),body='('+slash+'k<x>)';for(let slot=4;slot<74;slot++){body+='('+slash+slot+slash+slot+')'}let r=new RegExp('(?<x>a)(?<y>b)(?<last>'+body+')*','d'),required=new RegExp('(?<x>a)(?<y>b)(?<last>'+body+')+','d'),a=r.exec('ab');a[0]==='ab'&&a.slice(3).every(x=>x===undefined)&&required.exec('ab')===null",
    );
    let mut realm = Realm::default();
    assert_eq!(realm.eval("let huge=new RegExp('(?<x>)(?<y>)(?<last>(?<piece>\\\\k<x>)\\\\k<piece>\\\\k<y>){'+'9'.repeat(10000)+'}','d'),empty=huge.exec('q');empty[0]===''&&empty.groups.last===''&&empty.groups.piece===''&&empty.indices.groups.piece===empty.indices[4]&&empty.indices[4][0]===0"),Ok(Value::Boolean(true)));
    realm.eval("let r=new RegExp('(?<x>a)(?<y>b)(?<last>(?<piece>\\\\k<x>)'+'\\\\k<piece>'.repeat(100000)+'\\\\k<y>)+','d'),copy=new RegExp(r),text='ab'+'a'.repeat(100001)+'b'").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("let a=copy.exec(text);a[0]===text&&a.groups.last.length===100002&&a.groups.piece==='a'&&a.indices.groups.piece===a.indices[4]&&a.indices[4][0]===2&&a.indices[4][1]===3&&copy.source===r.source"),Ok(Value::Boolean(true)));
    realm.collect(usize::MAX).unwrap();
}

#[test]
fn explicit_local_reference_work_aborts_before_last_index_and_handlers() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm.eval(r"let marker=0,r=/(?<x>a)(?<y>b)(?<last>(?<piece>\k<x>)\k<piece>\k<y>)*c/g;r.lastIndex=1;let text='aab'.repeat(5000)").unwrap();
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
fn other_consuming_terms_choices_and_nested_repetition_remain_unsupported() {
    for source in [
        r"/(?:(?:(?:(a)(b)(a(\1)\4\2)+){2})|){2,3}/.test('aabaab')",
        r"/(a)(b)((\1)|\2){2,3}/.test('abab')",
        r"/(a)(b)((\1)+\4\2){2,3}/.test('aabaab')",
        r"/(a)(b)((?=a)\1\2){2,3}/.test('abab')",
        r"/(a)(b)((?i:\1)\2)+/.test('abab')",
        r"/(a)(b)((\1)\4\2)+/u.test('aabaab')",
    ] {
        assert!(
            matches!(
                Realm::default().eval(source),
                Err(Error::Unsupported { .. })
            ),
            "{source}"
        );
    }
    check(r"/(a)(b)((\1)\4\2)+/.exec('abc')===null");
}
