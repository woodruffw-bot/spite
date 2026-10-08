//! Assertions in repeated reference bodies use the complete input context.

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
fn repeated_reference_assertion_result_snapshot() {
    let mut rows = String::new();
    for (source, flags, text) in [
        (r"(a)(b)(\b\1\2)*", "d", "ababab"),
        (r"(a)(b)(\b\1\2)*?", "d", "ababab"),
        (r"(a)(b)(\B\1\2)+", "d", "ababab"),
        (r"(a)(b)(\1\B\2)+?", "d", "ababab"),
        (r"(a)(b)(\1\2$)?", "d", "abab"),
        (r"(a)(b)(^\1\2)??", "d", "abab"),
        (r"(a)(bb)((\B\2)\4\1){1,3}", "d", "abbbbbba"),
        (r"(a)(bb)(\2(.)(\B)\4){1,3}?", "d", "abbbbaa"),
        (r"(a)(b)(()(\Ba)\5\2())*ab", "d", "qabaabab"),
        (r"(a)(b)(()\1()([ab])(\B)\6())*?ab", "d", "qababbab"),
        (r"((a)(b)((\Ba)\5\3)+)\1", "d", "abaababaab"),
        (r"((a)(b)(\2([ab])\B\5)+?)\1", "d", "ababbababb"),
        (r"(?:(a)|(b))((\Ba)\4\2)+c", "d", "baabc"),
        (r"(?:(a)|(b))((\w)\B\4\1)+c", "d", "bbbc"),
        (r"(a)(b)((a)\B\4\2)*\4", "d", "abaaba"),
        (r"(a)(b)(\1([ab])\B\4)*?\4", "d", "abbbbb"),
        (r"(a)(b)(\4(a)\B\4\2)+", "d", "abaabaab"),
        (r"(a)(b)((\4a)\B\4\2)+", "d", "abaabaab"),
        (r"()()((^)\1\2){2,3}", "d", "q"),
        (
            r"(a)(b)(a\B\1\2){999999999999999999999999999999}",
            "d",
            "abaab",
        ),
        (
            r"(?:(?<x>a)|(?<x>b))(?<y>c)(?<last>(?<piece>a)\B\k<piece>\k<y>)+d",
            "d",
            "bcaacd",
        ),
        (
            r"(?<x>a)(?<y>b)(?<last>\k<piece>(?<piece>[ab])\B\k<piece>\k<y>)+",
            "dgi",
            "abaabaab",
        ),
        (r"(?<x>(^\k<x>)+)", "dgy", "q"),
        (r"(?<x>)(?<last>(?<point>^)\k<x>){2,3}", "dm", "\nq"),
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
fn repeated_assertions_keep_named_positions_and_whole_body_retry_ranges() {
    check(
        r"let a=/(?<x>a)(?<y>b)(?<last>(?<piece>a)\B\k<piece>\k<y>)*aab$/d.exec('abaabaab');a[0]==='abaabaab'&&a.groups.last==='aab'&&a.groups.piece==='a'&&a.indices.groups.last===a.indices[3]&&a.indices.groups.last[0]===2&&a.indices.groups.piece===a.indices[4]&&a.indices[4][0]===2&&a.indices[4][1]===3",
    );
    check(
        r"let a=/((?<x>a)(?<y>b)(?<last>(?<piece>a)\B\k<piece>\k<y>)+?)\1/d.exec('abaababaab');a[0]==='abaababaab'&&a[1]==='abaab'&&a.groups.piece==='a'&&a.indices.groups.piece===a.indices[5]&&a.indices[5][0]===2",
    );
    check(
        r"let a=/(?<x>a)(?<y>b)(?<last>(?<piece>[ab])\B\k<piece>\k<y>)+$/d.exec('abaabbbb');a[0]==='abaabbbb'&&a.groups.last==='bbb'&&a.groups.piece==='b'&&a.indices.groups.piece===a.indices[4]&&a.indices[4][0]===5&&a.indices[4][1]===6",
    );
    check(
        r"let a=/(.)(a)((.)$\4\2){1,2}b/dms.exec('aa\n\nab');a[0]==='aa\n\nab'&&a[3]==='\n\na'&&a[4]==='\n'&&a.indices[4][0]===2&&/(.)(a)((.)$\4\2){1,2}b/ds.exec('aa\n\nab')===null",
    );
    check(
        r"let r=/(?<x>)(?<last>(?<point>^)\k<x>)+/dmy;r.lastIndex=1;let a=r.exec('\nq');a.index===1&&a.groups.last===''&&a.indices.groups.point===a.indices[3]&&a.indices[3][0]===1&&r.lastIndex===1&&/(?<x>)(?<last>(?<point>^)\k<x>)+/dy.exec('\nq').index===0",
    );
    check(
        r"let r=/()((\b)\1)+/dy;r.lastIndex=1;let a=r.exec('x');a.index===1&&a[2]===''&&a.indices[3][0]===1&&/()((\b)\1)+/.exec(' ')===null&&/()((\B)\1)+/.exec(' ')[2]===''",
    );
}

#[test]
fn required_empty_assertions_and_consumers_preserve_zero_progress_rules() {
    check(
        r"let a=[...'q'.matchAll(/(?<x>)(?<last>(?<point>^)\k<x>)*/dg)],b=[...'q'.matchAll(/(?<x>)(?<last>(?<point>^)\k<x>)+/dg)];a.length===2&&a[0].groups.last===undefined&&a[1].groups.point===undefined&&b.length===1&&b[0].groups.point===''&&b[0].indices.groups.point===b[0].indices[3]&&b[0].indices[3][0]===0",
    );
    check(
        r"let r=/(?<x>)(?<last>(?<point>^)\k<x>){2,3}/dy;r.lastIndex=1;r.exec('q')===null&&r.lastIndex===0&&'q'.replace(/(?<x>)(?<last>(?<point>^)\k<x>)+/g,'<$<point>>')==='<>q'&&'q'.search(/()((^)\1)+/)===0",
    );
    check(
        r"let a=/(?<x>)(?<last>(?<point>^)\k<x>){0,2}/dy,b=/(?<x>)(?<last>(?<point>^)\k<x>)+/dy;a.lastIndex=1;let x=a.exec('q');x.index===1&&x.groups.last===undefined&&x.groups.point===undefined&&a.lastIndex===1&&b.exec('q').groups.point===''",
    );
    check(
        r"let r=/(?<x>a)(?<y>b)(?<last>(?<piece>[ab])\B\k<piece>\k<y>)+/dg,a=[...'abaab abaabbbb'.matchAll(r)];a.length===2&&a[1].groups.last==='bbb'&&a[1].indices.groups.piece[0]===11&&r.lastIndex===0&&'qabaabbbbZ'.split(r).join(',')==='q,a,b,bbb,b,Z'",
    );
    check(
        r"let r=/(?<x>a)(?<y>b)(?<last>(?<piece>[ab])\B\k<piece>\k<y>)+/dy;r.lastIndex=1;let copy=new RegExp(r),a=r.exec('qabaabbbb');a.groups.piece==='b'&&a.indices.groups.piece[0]===6&&r.lastIndex===9&&copy.source===r.source&&copy.lastIndex===0",
    );
}

#[test]
fn many_empty_assertion_captures_and_finite_minimums_copy_and_collect_unlimited() {
    let mut realm = Realm::default();
    assert_eq!(realm.eval("let required=new RegExp('(?<x>)(?<last>(?<point>^)\\\\k<x>){'+'9'.repeat(10000)+'}','dy'),empty=required.exec('q');empty[0]===''&&empty.groups.last===''&&empty.groups.point===''&&empty.indices.groups.point===empty.indices[3]&&empty.indices[3][0]===0"),Ok(Value::Boolean(true)));
    assert_eq!(
        realm.eval("required.lastIndex=1;required.exec('q')===null&&required.lastIndex===0"),
        Ok(Value::Boolean(true))
    );
    realm.eval("let r=new RegExp('(?<x>)(?<last>(?<point>\\\\b)'+'(\\\\b)'.repeat(99999)+'\\\\k<x>)+','d'),copy=new RegExp(r)").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("let a=copy.exec('x');a[0]===''&&a.length===100003&&a.groups.last===''&&a.groups.point===''&&a[100002]===''&&a.indices[100002][0]===0&&a.indices.groups.point===a.indices[3]&&copy.source===r.source"),Ok(Value::Boolean(true)));
    realm.collect(usize::MAX).unwrap();
}

#[test]
fn explicit_repeated_assertion_work_aborts_before_last_index_and_handlers() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm.eval(r"let marker=0,r=/(?<x>a)(?<y>b)(?<last>(?<piece>[ab])\B\k<piece>\k<y>)*c/g;r.lastIndex=1;let text='aab'.repeat(5000)").unwrap();
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
fn repeated_choices_nested_quantifiers_and_lookaround_remain_unsupported() {
    for source in [
        r"/(a)(b)(^\1|\2)+/.test('abab')",
        r"/(a)(b)(a+\1\2)+/.test('aabaab')",
        r"/(a)(b)(^\1+\2)+/.test('abab')",
        r"/(a)(b)((?=a)\1\2)+/.test('abab')",
        r"/(a)(b)((?i:\1)\2)+/.test('abab')",
        r"/(a)(b)(\b\1\2)+/u.test('aabaab')",
    ] {
        assert!(
            matches!(
                Realm::default().eval(source),
                Err(Error::Unsupported { .. })
            ),
            "{source}"
        );
    }
    check(r"/(a)(b)(\B\1\2)+/.exec('abc')===null");
}
