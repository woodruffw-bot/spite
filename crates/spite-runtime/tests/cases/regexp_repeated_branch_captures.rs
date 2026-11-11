//! Captures inside progressing repeated branches and nested branch loops.
use crate::common::assert_true;
use spite_core::JsString;
use spite_runtime::{Error, Limits, Realm, Value};
use std::fmt::Write;

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
    insta::assert_snapshot!(rows, @r#"
        "(a|ab)+b" flags="d" input="abb" "{\u0022matches\u0022:[\u0022ab\u0022,\u0022a\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022abb\u0022,\u0022indices\u0022:[[0,2],[0,1]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a|ab)+b\u0022}"
        "(ab|a)+b" flags="d" input="abb" "{\u0022matches\u0022:[\u0022abb\u0022,\u0022ab\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022abb\u0022,\u0022indices\u0022:[[0,3],[0,2]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(ab|a)+b\u0022}"
        "(a|ab)+?b" flags="d" input="ababb" "{\u0022matches\u0022:[\u0022ab\u0022,\u0022a\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022ababb\u0022,\u0022indices\u0022:[[0,2],[0,1]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a|ab)+?b\u0022}"
        "(ab|a)+?b" flags="d" input="ababb" "{\u0022matches\u0022:[\u0022ababb\u0022,\u0022ab\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022ababb\u0022,\u0022indices\u0022:[[0,5],[2,4]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(ab|a)+?b\u0022}"
        "((a)|(b))+c" flags="d" input="abc" "{\u0022matches\u0022:[\u0022abc\u0022,\u0022b\u0022,null,\u0022b\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022abc\u0022,\u0022indices\u0022:[[0,3],[1,2],null,[1,2]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022((a)|(b))+c\u0022}"
        "((a)|(b))+?c" flags="d" input="abc" "{\u0022matches\u0022:[\u0022abc\u0022,\u0022b\u0022,null,\u0022b\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022abc\u0022,\u0022indices\u0022:[[0,3],[1,2],null,[1,2]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022((a)|(b))+?c\u0022}"
        "(?:(a)|b)+c" flags="d" input="abc" "{\u0022matches\u0022:[\u0022abc\u0022,null],\u0022index\u0022:0,\u0022input\u0022:\u0022abc\u0022,\u0022indices\u0022:[[0,3],null],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?:(a)|b)+c\u0022}"
        "(?:(a)|b)*c" flags="d" input="c" "{\u0022matches\u0022:[\u0022c\u0022,null],\u0022index\u0022:0,\u0022input\u0022:\u0022c\u0022,\u0022indices\u0022:[[0,1],null],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?:(a)|b)*c\u0022}"
        "(a|b){2,3}c" flags="d" input="abbc" "{\u0022matches\u0022:[\u0022abbc\u0022,\u0022b\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022abbc\u0022,\u0022indices\u0022:[[0,4],[2,3]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a|b){2,3}c\u0022}"
        "(a|b){2,3}?c" flags="d" input="abbc" "{\u0022matches\u0022:[\u0022abbc\u0022,\u0022b\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022abbc\u0022,\u0022indices\u0022:[[0,4],[2,3]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a|b){2,3}?c\u0022}"
        "((a)+b|c)+d" flags="d" input="aabccd" "{\u0022matches\u0022:[\u0022aabccd\u0022,\u0022c\u0022,null],\u0022index\u0022:0,\u0022input\u0022:\u0022aabccd\u0022,\u0022indices\u0022:[[0,6],[4,5],null],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022((a)+b|c)+d\u0022}"
        "((a)+?b|c)+?d" flags="d" input="aabccd" "{\u0022matches\u0022:[\u0022aabccd\u0022,\u0022c\u0022,null],\u0022index\u0022:0,\u0022input\u0022:\u0022aabccd\u0022,\u0022indices\u0022:[[0,6],[4,5],null],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022((a)+?b|c)+?d\u0022}"
        "(a)(?:(b)\u005c1|c)+\u005c2" flags="d" input="abacc" "{\u0022matches\u0022:[\u0022abacc\u0022,\u0022a\u0022,null],\u0022index\u0022:0,\u0022input\u0022:\u0022abacc\u0022,\u0022indices\u0022:[[0,5],[0,1],null],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a)(?:(b)\u005c\u005c1|c)+\u005c\u005c2\u0022}"
        "(a)(?:(b)\u005c1|c)+?\u005c2" flags="d" input="abacc" "{\u0022matches\u0022:[\u0022abac\u0022,\u0022a\u0022,null],\u0022index\u0022:0,\u0022input\u0022:\u0022abacc\u0022,\u0022indices\u0022:[[0,4],[0,1],null],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a)(?:(b)\u005c\u005c1|c)+?\u005c\u005c2\u0022}"
        "((a|ab)+)+b" flags="d" input="abb" "{\u0022matches\u0022:[\u0022ab\u0022,\u0022a\u0022,\u0022a\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022abb\u0022,\u0022indices\u0022:[[0,2],[0,1],[0,1]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022((a|ab)+)+b\u0022}"
        "((a|ab)+?)+?b" flags="d" input="abb" "{\u0022matches\u0022:[\u0022ab\u0022,\u0022a\u0022,\u0022a\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022abb\u0022,\u0022indices\u0022:[[0,2],[0,1],[0,1]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022((a|ab)+?)+?b\u0022}"
        "(a|(b))+(c)\u005c2" flags="d" input="abac" "{\u0022matches\u0022:[\u0022abac\u0022,\u0022a\u0022,null,\u0022c\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022abac\u0022,\u0022indices\u0022:[[0,4],[2,3],null,[3,4]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a|(b))+(c)\u005c\u005c2\u0022}"
        "(a|\u005c1b)+c" flags="d" input="abac" "{\u0022matches\u0022:[\u0022abac\u0022,\u0022a\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022abac\u0022,\u0022indices\u0022:[[0,4],[2,3]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a|\u005c\u005c1b)+c\u0022}"
        "((\u005c2a)|b)+c" flags="d" input="abc" "{\u0022matches\u0022:[\u0022abc\u0022,\u0022b\u0022,null],\u0022index\u0022:0,\u0022input\u0022:\u0022abc\u0022,\u0022indices\u0022:[[0,3],[1,2],null],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022((\u005c\u005c2a)|b)+c\u0022}"
        "((^a)|(b$))+" flags="dm" input="q\u000aa" "{\u0022matches\u0022:[\u0022a\u0022,\u0022a\u0022,\u0022a\u0022,null],\u0022index\u0022:2,\u0022input\u0022:\u0022q\u005cna\u0022,\u0022indices\u0022:[[2,3],[2,3],[2,3],null],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022((^a)|(b$))+\u0022}"
        "(?<x>[ab]|c)+d" flags="di" input="aABcd" "{\u0022matches\u0022:[\u0022aABcd\u0022,\u0022c\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022aABcd\u0022,\u0022groups\u0022:{\u0022x\u0022:\u0022c\u0022},\u0022indices\u0022:[[0,5],[3,4]],\u0022indicesGroups\u0022:{\u0022x\u0022:[3,4]},\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<x>[ab]|c)+d\u0022}"
        "(?<x>.|a)+b" flags="ds" input="\u000a\u000aab" "{\u0022matches\u0022:[\u0022\u005cn\u005cnab\u0022,\u0022a\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022\u005cn\u005cnab\u0022,\u0022groups\u0022:{\u0022x\u0022:\u0022a\u0022},\u0022indices\u0022:[[0,4],[2,3]],\u0022indicesGroups\u0022:{\u0022x\u0022:[2,3]},\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<x>.|a)+b\u0022}"
        "(?:(?<x>a)|(?<x>b)){2}\u005ck<x>" flags="d" input="abb" "{\u0022matches\u0022:[\u0022abb\u0022,null,\u0022b\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022abb\u0022,\u0022groups\u0022:{\u0022x\u0022:\u0022b\u0022},\u0022indices\u0022:[[0,3],null,[1,2]],\u0022indicesGroups\u0022:{\u0022x\u0022:[1,2]},\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?:(?<x>a)|(?<x>b)){2}\u005c\u005ck<x>\u0022}"
        "^(?:(?<x>a)|(?<x>b)|c){2}\u005ck<x>$" flags="d" input="ac" "{\u0022matches\u0022:[\u0022ac\u0022,null,null],\u0022index\u0022:0,\u0022input\u0022:\u0022ac\u0022,\u0022groups\u0022:{},\u0022indices\u0022:[[0,2],null,null],\u0022indicesGroups\u0022:{},\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022^(?:(?<x>a)|(?<x>b)|c){2}\u005c\u005ck<x>$\u0022}"
        "(?<x>a|\u005ck<x>b)+c" flags="d" input="abac" "{\u0022matches\u0022:[\u0022abac\u0022,\u0022a\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022abac\u0022,\u0022groups\u0022:{\u0022x\u0022:\u0022a\u0022},\u0022indices\u0022:[[0,4],[2,3]],\u0022indicesGroups\u0022:{\u0022x\u0022:[2,3]},\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<x>a|\u005c\u005ck<x>b)+c\u0022}"
        "(?<outer>(?<inner>a|ab)+)+b" flags="d" input="abb" "{\u0022matches\u0022:[\u0022ab\u0022,\u0022a\u0022,\u0022a\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022abb\u0022,\u0022groups\u0022:{\u0022outer\u0022:\u0022a\u0022,\u0022inner\u0022:\u0022a\u0022},\u0022indices\u0022:[[0,2],[0,1],[0,1]],\u0022indicesGroups\u0022:{\u0022outer\u0022:[0,1],\u0022inner\u0022:[0,1]},\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<outer>(?<inner>a|ab)+)+b\u0022}"
        "(?:(a)(b)((?<x>\u005c1)\u005ck<x>\u005c2)+){2}" flags="d" input="abaababaab" "{\u0022matches\u0022:[\u0022abaababaab\u0022,\u0022a\u0022,\u0022b\u0022,\u0022aab\u0022,\u0022a\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022abaababaab\u0022,\u0022groups\u0022:{\u0022x\u0022:\u0022a\u0022},\u0022indices\u0022:[[0,10],[5,6],[6,7],[7,10],[7,8]],\u0022indicesGroups\u0022:{\u0022x\u0022:[7,8]},\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?:(a)(b)((?<x>\u005c\u005c1)\u005c\u005ck<x>\u005c\u005c2)+){2}\u0022}"
        "(?:(a)(b)(\u005c1(?<x>\u005c2)\u005ck<x>)+){2}" flags="d" input="ababbababb" "{\u0022matches\u0022:[\u0022ababbababb\u0022,\u0022a\u0022,\u0022b\u0022,\u0022abb\u0022,\u0022b\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022ababbababb\u0022,\u0022groups\u0022:{\u0022x\u0022:\u0022b\u0022},\u0022indices\u0022:[[0,10],[5,6],[6,7],[7,10],[8,9]],\u0022indicesGroups\u0022:{\u0022x\u0022:[8,9]},\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?:(a)(b)(\u005c\u005c1(?<x>\u005c\u005c2)\u005c\u005ck<x>)+){2}\u0022}"
        "#);
}

#[test]
fn repeated_branch_capture_clearing_and_undo_preserve_names_refs_and_last_ranges() {
    assert_true(
        r#"
            let a = /(?:(a)(b)((?<x>\1)\k<x>\2)+){2}/d.exec('abaababaab');
            a[0] === 'abaababaab' && a.groups.x === 'a' && a.indices.groups.x === a.indices[4] && a
                .indices[4][0] === 7 && a.indices[4][1] === 8
        "#,
    );
    assert_true(
        r#"
            let a = /(?:(a)(b)(\1(?<x>\2)\k<x>)+){2}/d.exec('ababbababb');
            a[0] === 'ababbababb' && a.groups.x === 'b' && a.indices.groups.x === a.indices[4] && a
                .indices[4][0] === 8 && a.indices[4][1] === 9
        "#,
    );
    assert_true(
        r#"
            let a = /(?<x>a|ab)+b/d.exec('abb'),
                b = /(?<x>ab|a)+b/d.exec('abb');
            a[0] === 'ab' && a.groups.x === 'a' && a.indices.groups.x === a.indices[1] && a.indices[1][
                1] === 1 && b[0] === 'abb' && b.groups.x === 'ab' && b.indices[1][1] === 2
        "#,
    );
    assert_true(
        r#"
            let a = /(?<all>(?:(?<a>a)|(?<b>b))+c)/d.exec('abc');
            a.groups.all === 'abc' && a.groups.a === undefined && a.groups.b === 'b' && a.indices.groups
                .a === undefined && a.indices.groups.b === a.indices[3] && a.indices[3][0] === 1
        "#,
    );
    assert_true(
        r#"
            let a = /(?:(?<x>a)|(?<x>b)){2}\k<x>/d.exec('abb'),
                b = /^(?:(?<x>a)|(?<x>b)|c){2}\k<x>$/d.exec('ac');
            a[0] === 'abb' && a[1] === undefined && a[2] === 'b' && a.groups.x === 'b' && a.indices.groups
                .x === a.indices[2] && b[0] === 'ac' && b.groups.x === undefined && b.indices.groups.x ===
                undefined && !/^(?:(?<x>a)|(?<x>b)|c){2}\k<x>$/.test('aca')
        "#,
    );
    assert_true(
        r#"
            let a = /(?<x>a)(?:(?<y>b)\k<x>|c)+\k<y>/d.exec('abacc');
            a[0] === 'abacc' && a.groups.x === 'a' && a.groups.y === undefined && a.indices.groups.y ===
                undefined && a.indices[2] === undefined
        "#,
    );
    assert_true(
        r#"
            let a = /(?<x>a|\k<x>b)+c/d.exec('abac');
            a[0] === 'abac' && a.groups.x === 'a' && a.indices.groups.x === a.indices[1] && a.indices[1][
                0] === 2
        "#,
    );
    assert_true(
        r#"
            let a = /(?<outer>(?<inner>a|ab)+)+b/d.exec('abb');
            a[0] === 'ab' && a.groups.outer === 'a' && a.groups.inner === 'a' && a.indices.groups.outer ===
                a.indices[1] && a.indices.groups.inner === a.indices[2] && a.indices[1][1] === 1 && a
                .indices[2][1] === 1
        "#,
    );
    assert_true(
        r#"
            let a = /(?<x>(\k<later>a)|b)+(?<later>c)/d.exec('abc');
            a[0] === 'abc' && a.groups.x === 'b' && a[2] === undefined && a.groups.later === 'c' && a
                .indices.groups.x[0] === 1 && a.indices.groups.later[0] === 2
        "#,
    );
    assert_true(
        r"let a=/(?<x>.|a)+c/ds.exec('\uD800ac');a.groups.x==='a'&&a.indices.groups.x[0]===1&&a.indices.groups.x[1]===2",
    );
}

#[test]
fn repeated_branch_capture_consumers_sticky_zero_iterations_and_undefined_slots() {
    assert_true(
        r#"
            let r = /(?:(?<x>a)|b)+c/dg,
                a = [...'abc ac'.matchAll(r)];
            a.length === 2 && a[0][0] === 'abc' && a[0].groups.x === undefined && a[0].indices.groups.x ===
                undefined && a[1].index === 4 && a[1].groups.x === 'a' && a[1].indices.groups.x === a[1]
                .indices[1] && a[1].indices[1][0] === 4 && r.lastIndex === 0
        "#,
    );
    assert_true(
        r#"
            let a = 'abc'.split(/(?:(a)|b)+c/);
            'abc'.replace(/(?:(?<x>a)|b)+c/, '<$<x>>') === '<>' && 'abc'.search(/(?:(a)|b)+c/) === 0 && a
                .length === 3 && a[1] === undefined
        "#,
    );
    assert_true(
        r#"
            let r = /(?:(?<x>a)|b)+c/dy;
            r.lastIndex = 1;
            let copy = new RegExp(r),
                a = r.exec('qabc');
            a.groups.x === undefined && a.indices.groups.x === undefined && r.lastIndex === 4 && r.exec(
                    'qabc') === null && r.lastIndex === 0 && copy.source === r.source && copy.lastIndex ===
                0
        "#,
    );
    assert_true(
        r#"
            let a = [...'q'.matchAll(/(?<all>(?:(a)|b)*)/dg)];
            a.length === 2 && a[0].groups.all === '' && a[0][2] === undefined && a[1][2] === undefined &&
                a[1].indices.groups.all[0] === 1 && a[1].indices.groups.all[1] === 1
        "#,
    );
    assert_true(
        r#"
            let a = /(?<x>a|b){2,3}c/d.exec('abbc'),
                b = /(?<x>a|b){2,3}?c/d.exec('abbc');
            a[0] === 'abbc' && a.groups.x === 'b' && a.indices.groups.x[0] === 2 && b.groups.x === 'b' && b
                .indices.groups.x[0] === 2
        "#,
    );
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
        r"/(a|b)+b/u.test('ab')",
        r"/(a|b)+b/v.test('ab')",
        r"/((?:a|b)?){2,3}/.test('ab')",
    ] {
        crate::common::regexp_oracle::assert_matches_reference(&mut Realm::default(), source);
    }
    assert_true(r"/(a|b)+c/.exec('ab')===null");
}
