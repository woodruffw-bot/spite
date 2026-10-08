//! Repeated reference bodies retain pinned one-unit predicates and local ranges.

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
fn repeated_reference_character_result_snapshot() {
    let mut rows = String::new();
    for (source, flags, text) in [
        (r"(a)(a\1)*", "d", "aaaaa"),
        (r"(a)(a\1)*?", "d", "aaaaa"),
        (r"(a)(b)(a\1\2)+", "d", "abaabaab"),
        (r"(a)(b)(\1b\2)+?", "d", "ababbabb"),
        (r"(a)(b)([ab]\1\2)?", "d", "abaab"),
        (r"(a)(b)(\1[^c]\2)??", "d", "ababb"),
        (r"(a)(bb)(([ab])\4\2){1,3}", "d", "abbaabbaabb"),
        (r"(a)(bb)(\2(.)\4){1,3}?", "d", "abbbbaa"),
        (r"(a)(b)(()(a)\5\2())*ab", "d", "qabaabaab"),
        (r"(a)(b)(()\1()([ab])\6())*?ab", "d", "qababb"),
        (r"((a)(b)((a)\5\3)+)\1", "d", "abaababaab"),
        (r"((a)(b)(\2([ab])\5)+?)\1", "d", "ababbababb"),
        (r"(?:(a)|(b))((a)\4\2)+c", "d", "baabc"),
        (r"(?:(a)|(b))((\w)\4\1)+c", "d", "bbbc"),
        (r"(a)(b)((a)\4\2)*\4", "d", "abaaba"),
        (r"(a)(b)(\1([ab])\4)*?\4", "d", "abbbbb"),
        (r"(a)(b)(\4(a)\4\2)+", "d", "abaabaab"),
        (r"(a)(b)((\4a)\4\2)+", "d", "abaabaab"),
        (r"()()((a)\4\1\2){2,3}", "d", "aaaaaa"),
        (
            r"(a)(b)(a\1\2){999999999999999999999999999999}",
            "d",
            "abaab",
        ),
        (
            r"(?:(?<x>a)|(?<x>b))(?<y>c)(?<last>(?<piece>a)\k<piece>\k<y>)+d",
            "d",
            "bcaacd",
        ),
        (
            r"(?<x>a)(?<y>b)(?<last>\k<piece>(?<piece>[ab])\k<piece>\k<y>)+",
            "dgi",
            "abaabaab",
        ),
        (r"(?<x>a)(?<last>(?<piece>a)\k<piece>)+", "dy", "aaaaa"),
        (
            r"^(?<x>.)(?<y>a)(?<last>(?<piece>.)\k<piece>\k<y>)+$",
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
fn repeated_character_terms_keep_current_named_values_and_retry_ranges() {
    check(
        r"let a=/(\1a)+/d.exec('aaa'),b=/((a)\1)+/d.exec('aaa'),c=/(?:(a)\1){2}/d.exec('aaaa');a[1]==='a'&&a.indices[1][0]===2&&b[1]==='a'&&b[2]==='a'&&b.indices[2][0]===2&&c[1]==='a'&&c.indices[1][0]===2",
    );
    check(
        r"let a=/(?<x>a)(?<y>b)(?<last>(?<piece>a)\k<piece>\k<y>)*aab$/d.exec('abaabaab');a[0]==='abaabaab'&&a.groups.last==='aab'&&a.groups.piece==='a'&&a.indices.groups.last===a.indices[3]&&a.indices.groups.last[0]===2&&a.indices.groups.piece===a.indices[4]&&a.indices[4][0]===2&&a.indices[4][1]===3",
    );
    check(
        r"let a=/((?<x>a)(?<y>b)(?<last>(?<piece>a)\k<piece>\k<y>)+?)\1/d.exec('abaababaab');a[0]==='abaababaab'&&a[1]==='abaab'&&a.groups.last==='aab'&&a.groups.piece==='a'&&a.indices.groups.piece===a.indices[5]&&a.indices[5][0]===2",
    );
    check(
        r"let a=/(?<x>a)(?<y>b)(?<last>(?<piece>[ab])\k<piece>\k<y>)+$/d.exec('abaabbbb');a[0]==='abaabbbb'&&a.groups.last==='bbb'&&a.groups.piece==='b'&&a.indices.groups.piece===a.indices[4]&&a.indices[4][0]===5&&a.indices[4][1]===6",
    );
    check(
        r"let a=/(?:(?<x>a)|(?<x>b))(?<y>c)(?<last>(?<piece>a)\k<piece>\k<y>)+/d.exec('bcaac');a[1]===undefined&&a.groups.x==='b'&&a.groups.piece==='a'&&a.indices.groups.piece===a.indices[5]&&a.indices[5][0]===2",
    );
    check(
        r"let a=/(?<x>a)(?<y>b)(?<last>(?<piece>.)\k<piece>\k<y>)+/ds.exec('ab\n\nb');a.groups.piece==='\n'&&a.indices.groups.piece[0]===2&&/(?<x>a)(?<y>b)(?<last>(?<piece>.)\k<piece>\k<y>)+/.exec('ab\n\nb')===null",
    );
    check(
        r"let a=/(?<x>a)(?<y>b)(?<last>(?<piece>.)\k<piece>\k<y>)+/d.exec('ab\uD800\uD800b');a[0].length===5&&a.groups.piece.charCodeAt(0)===0xD800&&a.indices.groups.piece===a.indices[4]&&a.indices[4][0]===2&&a.indices[4][1]===3",
    );
}

#[test]
fn repeated_character_consumers_preserve_partial_indices_and_final_values() {
    check(
        r"let r=/(?<x>a)(?<y>b)(?<last>(?<piece>[ab])\k<piece>\k<y>)+/dg,a=[...'abaab abaabbbb'.matchAll(r)];a.length===2&&a[0].groups.last==='aab'&&a[1].groups.last==='bbb'&&a[1].groups.piece==='b'&&a[1].indices.groups.piece===a[1].indices[4]&&a[1].indices[4][0]===11&&r.lastIndex===0",
    );
    check(
        r"'qabaabbbb'.replace(/(?<x>a)(?<y>b)(?<last>(?<piece>[ab])\k<piece>\k<y>)+/,'<$<last>:$<piece>>')==='q<bbb:b>'&&'qabaabbbb'.search(/(a)(b)(([ab])\4\2)+/)===1&&'qabaabbbbZ'.split(/(a)(b)(([ab])\4\2)+/).join(',')==='q,a,b,bbb,b,Z'",
    );
    check(
        r"let r=/(?<x>a)(?<y>b)(?<last>(?<piece>[ab])\k<piece>\k<y>)+/dy;r.lastIndex=1;let copy=new RegExp(r),a=r.exec('qabaabbbb');a.groups.piece==='b'&&a.indices.groups.piece[0]===6&&a.indices.groups.piece[1]===7&&r.lastIndex===9&&r.exec('qabaabbbb')===null&&r.lastIndex===0&&copy.source===r.source&&copy.lastIndex===0",
    );
    check(
        r"let a=/(?<x>a)(?<y>b)(?<last>(?<piece>[ab])\k<piece>\k<y>)*\k<piece>$/d.exec('abbbbb'),b=/(?<x>a)(?<y>b)(?<last>(?<piece>[ab])\k<piece>\k<y>)*?\k<piece>$/d.exec('abbbbb');a[0]==='abbbbb'&&b[0]==='abbbbb'&&a.groups.piece==='b'&&b.groups.piece==='b'&&a.indices.groups.piece[0]===2&&b.indices.groups.piece[0]===2",
    );
    check(
        r"let a=/(?<x>a)(?<last>a\k<x>)* /d.exec('a ');a.groups.last===undefined&&a.indices.groups.last===undefined",
    );
}

#[test]
fn deep_character_captures_and_huge_finite_bounds_copy_and_collect_unlimited() {
    let mut realm = Realm::default();
    assert_eq!(realm.eval("let required=new RegExp('(?<x>a)(?<last>(?<piece>a)\\\\k<piece>){'+'9'.repeat(10000)+'}','d'),optional=new RegExp('(?<x>a)(?<last>(?<piece>a)\\\\k<piece>){0,'+'9'.repeat(10000)+'}','d'),optionalResult=optional.exec('a');required.exec('a')===null&&optionalResult[0]==='a'&&optionalResult.groups.last===undefined&&optionalResult.groups.piece===undefined"),Ok(Value::Boolean(true)));
    realm.eval("let r=new RegExp('(?<x>a)(?<y>b)(?<last>(?<piece>'+'('.repeat(99999)+'a'+')'.repeat(100000)+'\\\\k<piece>\\\\k<y>)+','d'),copy=new RegExp(r)").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("let a=copy.exec('abaabaab');a[0]==='abaabaab'&&a.length===100004&&a.groups.last==='aab'&&a.groups.piece==='a'&&a[100003]==='a'&&a.indices[100003][0]===5&&a.indices[100003][1]===6&&a.indices.groups.piece===a.indices[4]&&copy.source===r.source"),Ok(Value::Boolean(true)));
    realm.collect(usize::MAX).unwrap();
}

#[test]
fn explicit_repeated_character_work_aborts_before_last_index_and_handlers() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm.eval(r"let marker=0,r=/(?<x>a)(?<y>b)(?<last>(?<piece>[ab])\k<piece>\k<y>)*c/g;r.lastIndex=1;let text='aab'.repeat(5000)").unwrap();
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
fn repeated_assertions_choices_and_nested_quantifiers_remain_unsupported() {
    for source in [
        r"/(?:(a)(b)(\b\1\2)+){2}/.test('abab')",
        r"/(a)(b)((\1)|\2)+/.test('abab')",
        r"/(a)(b)(a+\1\2)+/.test('aabaab')",
        r"/(a)(b)((?=a)\1\2)+/.test('abab')",
        r"/(a)(b)((?i:\1)\2)+/.test('abab')",
        r"/(a)(b)(a\1\2)+/u.test('aabaab')",
    ] {
        assert!(
            matches!(
                Realm::default().eval(source),
                Err(Error::Unsupported { .. })
            ),
            "{source}"
        );
    }
    check(r"/(a)(b)(a\1\2)+/.exec('abc')===null");
}
