//! Partial captures in repeated reference bodies retain final-iteration ranges.

use spite_core::JsString;
use spite_runtime::{Error, Limits, Realm, Value};
use std::fmt::Write;

use crate::common::assert_true;

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
    insta::assert_snapshot!(rows, @r#"
        "(a)((\u005c1)\u005c1)*" flags="d" input="aaaaa" "{\u0022matches\u0022:[\u0022aaaaa\u0022,\u0022a\u0022,\u0022aa\u0022,\u0022a\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022aaaaa\u0022,\u0022indices\u0022:[[0,5],[0,1],[3,5],[3,4]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a)((\u005c\u005c1)\u005c\u005c1)*\u0022}"
        "(a)((\u005c1)\u005c1)*?" flags="d" input="aaaaa" "{\u0022matches\u0022:[\u0022a\u0022,\u0022a\u0022,null,null],\u0022index\u0022:0,\u0022input\u0022:\u0022aaaaa\u0022,\u0022indices\u0022:[[0,1],[0,1],null,null],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a)((\u005c\u005c1)\u005c\u005c1)*?\u0022}"
        "(a)(b)((\u005c1)\u005c2)+" flags="d" input="ababab" "{\u0022matches\u0022:[\u0022ababab\u0022,\u0022a\u0022,\u0022b\u0022,\u0022ab\u0022,\u0022a\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022ababab\u0022,\u0022indices\u0022:[[0,6],[0,1],[1,2],[4,6],[4,5]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a)(b)((\u005c\u005c1)\u005c\u005c2)+\u0022}"
        "(a)(b)(\u005c1(\u005c2))+?" flags="d" input="ababab" "{\u0022matches\u0022:[\u0022abab\u0022,\u0022a\u0022,\u0022b\u0022,\u0022ab\u0022,\u0022b\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022ababab\u0022,\u0022indices\u0022:[[0,4],[0,1],[1,2],[2,4],[3,4]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a)(b)(\u005c\u005c1(\u005c\u005c2))+?\u0022}"
        "(a)(b)((\u005c1)\u005c2)?" flags="d" input="abab" "{\u0022matches\u0022:[\u0022abab\u0022,\u0022a\u0022,\u0022b\u0022,\u0022ab\u0022,\u0022a\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022abab\u0022,\u0022indices\u0022:[[0,4],[0,1],[1,2],[2,4],[2,3]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a)(b)((\u005c\u005c1)\u005c\u005c2)?\u0022}"
        "(a)(b)(\u005c1(\u005c2))??" flags="d" input="abab" "{\u0022matches\u0022:[\u0022ab\u0022,\u0022a\u0022,\u0022b\u0022,null,null],\u0022index\u0022:0,\u0022input\u0022:\u0022abab\u0022,\u0022indices\u0022:[[0,2],[0,1],[1,2],null,null],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a)(b)(\u005c\u005c1(\u005c\u005c2))??\u0022}"
        "(a)(bb)((\u005c2)\u005c1){1,3}" flags="d" input="abbbbabba" "{\u0022matches\u0022:[\u0022abbbbabba\u0022,\u0022a\u0022,\u0022bb\u0022,\u0022bba\u0022,\u0022bb\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022abbbbabba\u0022,\u0022indices\u0022:[[0,9],[0,1],[1,3],[6,9],[6,8]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a)(bb)((\u005c\u005c2)\u005c\u005c1){1,3}\u0022}"
        "(a)(bb)(\u005c2(\u005c1)){1,3}?" flags="d" input="abbbbabba" "{\u0022matches\u0022:[\u0022abbbba\u0022,\u0022a\u0022,\u0022bb\u0022,\u0022bba\u0022,\u0022a\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022abbbbabba\u0022,\u0022indices\u0022:[[0,6],[0,1],[1,3],[3,6],[5,6]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a)(bb)(\u005c\u005c2(\u005c\u005c1)){1,3}?\u0022}"
        "(a)(b)(()(\u005c1)()\u005c2())*ab" flags="d" input="qababab" "{\u0022matches\u0022:[\u0022ababab\u0022,\u0022a\u0022,\u0022b\u0022,\u0022ab\u0022,\u0022\u0022,\u0022a\u0022,\u0022\u0022,\u0022\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022qababab\u0022,\u0022indices\u0022:[[1,7],[1,2],[2,3],[3,5],[3,3],[3,4],[4,4],[5,5]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a)(b)(()(\u005c\u005c1)()\u005c\u005c2())*ab\u0022}"
        "(a)(b)(()\u005c1()(\u005c2)())*?ab" flags="d" input="qababab" "{\u0022matches\u0022:[\u0022abab\u0022,\u0022a\u0022,\u0022b\u0022,null,null,null,null,null],\u0022index\u0022:1,\u0022input\u0022:\u0022qababab\u0022,\u0022indices\u0022:[[1,5],[1,2],[2,3],null,null,null,null,null],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a)(b)(()\u005c\u005c1()(\u005c\u005c2)())*?ab\u0022}"
        "((a)(b)((\u005c2)\u005c3)+)\u005c1" flags="d" input="abababab" "{\u0022matches\u0022:[\u0022abababab\u0022,\u0022abab\u0022,\u0022a\u0022,\u0022b\u0022,\u0022ab\u0022,\u0022a\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022abababab\u0022,\u0022indices\u0022:[[0,8],[0,4],[0,1],[1,2],[2,4],[2,3]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022((a)(b)((\u005c\u005c2)\u005c\u005c3)+)\u005c\u005c1\u0022}"
        "((a)(b)(\u005c2(\u005c3))+?)\u005c1" flags="d" input="abababab" "{\u0022matches\u0022:[\u0022abababab\u0022,\u0022abab\u0022,\u0022a\u0022,\u0022b\u0022,\u0022ab\u0022,\u0022b\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022abababab\u0022,\u0022indices\u0022:[[0,8],[0,4],[0,1],[1,2],[2,4],[3,4]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022((a)(b)(\u005c\u005c2(\u005c\u005c3))+?)\u005c\u005c1\u0022}"
        "(?:(a)|(b))((\u005c1)\u005c2)+c" flags="d" input="bbc" "{\u0022matches\u0022:[\u0022bbc\u0022,null,\u0022b\u0022,\u0022b\u0022,\u0022\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022bbc\u0022,\u0022indices\u0022:[[0,3],null,[0,1],[1,2],[1,1]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?:(a)|(b))((\u005c\u005c1)\u005c\u005c2)+c\u0022}"
        "(?:(a)|(b))((\u005c2)\u005c1)+c" flags="d" input="bbc" "{\u0022matches\u0022:[\u0022bbc\u0022,null,\u0022b\u0022,\u0022b\u0022,\u0022b\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022bbc\u0022,\u0022indices\u0022:[[0,3],null,[0,1],[1,2],[1,2]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?:(a)|(b))((\u005c\u005c2)\u005c\u005c1)+c\u0022}"
        "(a)(b)((\u005c1)\u005c2)*\u005c4" flags="d" input="ababa" "{\u0022matches\u0022:[\u0022ababa\u0022,\u0022a\u0022,\u0022b\u0022,\u0022ab\u0022,\u0022a\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022ababa\u0022,\u0022indices\u0022:[[0,5],[0,1],[1,2],[2,4],[2,3]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a)(b)((\u005c\u005c1)\u005c\u005c2)*\u005c\u005c4\u0022}"
        "(a)(b)(\u005c1(\u005c2))*?\u005c4" flags="d" input="ababb" "{\u0022matches\u0022:[\u0022ab\u0022,\u0022a\u0022,\u0022b\u0022,null,null],\u0022index\u0022:0,\u0022input\u0022:\u0022ababb\u0022,\u0022indices\u0022:[[0,2],[0,1],[1,2],null,null],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a)(b)(\u005c\u005c1(\u005c\u005c2))*?\u005c\u005c4\u0022}"
        "(a)(b)(\u005c4(\u005c1)\u005c2)+" flags="d" input="ababab" "{\u0022matches\u0022:[\u0022ababab\u0022,\u0022a\u0022,\u0022b\u0022,\u0022ab\u0022,\u0022a\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022ababab\u0022,\u0022indices\u0022:[[0,6],[0,1],[1,2],[4,6],[4,5]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a)(b)(\u005c\u005c4(\u005c\u005c1)\u005c\u005c2)+\u0022}"
        "(a)(b)((\u005c4\u005c1)\u005c2)+" flags="d" input="ababab" "{\u0022matches\u0022:[\u0022ababab\u0022,\u0022a\u0022,\u0022b\u0022,\u0022ab\u0022,\u0022a\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022ababab\u0022,\u0022indices\u0022:[[0,6],[0,1],[1,2],[4,6],[4,5]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a)(b)((\u005c\u005c4\u005c\u005c1)\u005c\u005c2)+\u0022}"
        "()()((\u005c1)\u005c2){2,3}" flags="d" input="q" "{\u0022matches\u0022:[\u0022\u0022,\u0022\u0022,\u0022\u0022,\u0022\u0022,\u0022\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022q\u0022,\u0022indices\u0022:[[0,0],[0,0],[0,0],[0,0],[0,0]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022()()((\u005c\u005c1)\u005c\u005c2){2,3}\u0022}"
        "(?:(?<x>a)|(?<x>b))(?<y>c)(?<last>(?<piece>\u005ck<x>)\u005ck<y>)+d" flags="d" input="bcbcd" "{\u0022matches\u0022:[\u0022bcbcd\u0022,null,\u0022b\u0022,\u0022c\u0022,\u0022bc\u0022,\u0022b\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022bcbcd\u0022,\u0022groups\u0022:{\u0022x\u0022:\u0022b\u0022,\u0022y\u0022:\u0022c\u0022,\u0022last\u0022:\u0022bc\u0022,\u0022piece\u0022:\u0022b\u0022},\u0022indices\u0022:[[0,5],null,[0,1],[1,2],[2,4],[2,3]],\u0022indicesGroups\u0022:{\u0022x\u0022:[0,1],\u0022y\u0022:[1,2],\u0022last\u0022:[2,4],\u0022piece\u0022:[2,3]},\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?:(?<x>a)|(?<x>b))(?<y>c)(?<last>(?<piece>\u005c\u005ck<x>)\u005c\u005ck<y>)+d\u0022}"
        "(?<x>a)(?<y>b)(?<last>\u005ck<piece>(?<piece>\u005ck<x>)\u005ck<y>)+" flags="dgi" input="ababab" "{\u0022matches\u0022:[\u0022ababab\u0022,\u0022a\u0022,\u0022b\u0022,\u0022ab\u0022,\u0022a\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022ababab\u0022,\u0022groups\u0022:{\u0022x\u0022:\u0022a\u0022,\u0022y\u0022:\u0022b\u0022,\u0022last\u0022:\u0022ab\u0022,\u0022piece\u0022:\u0022a\u0022},\u0022indices\u0022:[[0,6],[0,1],[1,2],[4,6],[4,5]],\u0022indicesGroups\u0022:{\u0022x\u0022:[0,1],\u0022y\u0022:[1,2],\u0022last\u0022:[4,6],\u0022piece\u0022:[4,5]},\u0022lastIndex\u0022:6,\u0022source\u0022:\u0022(?<x>a)(?<y>b)(?<last>\u005c\u005ck<piece>(?<piece>\u005c\u005ck<x>)\u005c\u005ck<y>)+\u0022}"
        "(?<x>a)(?<last>(?<piece>\u005c1)\u005ck<x>)+" flags="dy" input="aaaaa" "{\u0022matches\u0022:[\u0022aaaaa\u0022,\u0022a\u0022,\u0022aa\u0022,\u0022a\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022aaaaa\u0022,\u0022groups\u0022:{\u0022x\u0022:\u0022a\u0022,\u0022last\u0022:\u0022aa\u0022,\u0022piece\u0022:\u0022a\u0022},\u0022indices\u0022:[[0,5],[0,1],[3,5],[3,4]],\u0022indicesGroups\u0022:{\u0022x\u0022:[0,1],\u0022last\u0022:[3,5],\u0022piece\u0022:[3,4]},\u0022lastIndex\u0022:5,\u0022source\u0022:\u0022(?<x>a)(?<last>(?<piece>\u005c\u005c1)\u005c\u005ck<x>)+\u0022}"
        "^(?<x>.)(?<y>a)(?<last>(?<piece>\u005ck<x>)\u005ck<y>)+$" flags="dis" input="\u00b5a\u039cA\u00b5a" "{\u0022matches\u0022:[\u0022\u00b5a\u039cA\u00b5a\u0022,\u0022\u00b5\u0022,\u0022a\u0022,\u0022\u00b5a\u0022,\u0022\u00b5\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022\u00b5a\u039cA\u00b5a\u0022,\u0022groups\u0022:{\u0022x\u0022:\u0022\u00b5\u0022,\u0022y\u0022:\u0022a\u0022,\u0022last\u0022:\u0022\u00b5a\u0022,\u0022piece\u0022:\u0022\u00b5\u0022},\u0022indices\u0022:[[0,6],[0,1],[1,2],[4,6],[4,5]],\u0022indicesGroups\u0022:{\u0022x\u0022:[0,1],\u0022y\u0022:[1,2],\u0022last\u0022:[4,6],\u0022piece\u0022:[4,5]},\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022^(?<x>.)(?<y>a)(?<last>(?<piece>\u005c\u005ck<x>)\u005c\u005ck<y>)+$\u0022}"
        "#);
}

#[test]
fn partial_named_ranges_restore_after_whole_body_retries() {
    assert_true(
        r#"
            let a = /(?<x>a)(?<y>b)(?<last>(?<piece>\k<x>)\k<y>)*ab$/d.exec('ababab');
            a[0] === 'ababab' && a.groups.last === 'ab' && a.groups.piece === 'a' && a.indices.groups
                .last === a.indices[3] && a.indices.groups.last[0] === 2 && a.indices.groups.piece === a
                .indices[4] && a.indices[4][0] === 2 && a.indices[4][1] === 3
        "#,
    );
    assert_true(
        r#"
            let a = /((?<x>a)(?<y>b)(?<last>\k<x>(?<piece>\k<y>))+?)\1/d.exec('abababab');
            a[0] === 'abababab' && a[1] === 'abab' && a.groups.piece === 'b' && a.indices.groups.piece ===
                a.indices[5] && a.indices[5][0] === 3 && a.indices[5][1] === 4
        "#,
    );
    assert_true(
        r#"
            let a = /(?<x>a)(?<y>bb)(?<last>\k<y>(?<piece>\k<x>))+$/d.exec('abbbbabba');
            a.groups.last === 'bba' && a.groups.piece === 'a' && a.indices.groups.piece === a.indices[4] &&
                a.indices[4][0] === 8 && a.indices[4][1] === 9
        "#,
    );
    assert_true(
        r#"
            let a = /(?:(?<x>a)|(?<x>b))(?<last>(?<piece>\1)\2)+c/d.exec('bbc');
            a[0] === 'bbc' && a[1] === undefined && a.groups.x === 'b' && a.groups.piece === '' && a
                .indices.groups.piece === a.indices[4] && a.indices[4][0] === 1 && a.indices[4][1] === 1
        "#,
    );
    assert_true(
        r#"
            let a = /(?<x>a)(?<y>b)(?<last>\k<piece>(?<piece>\k<x>)\k<y>)+/d.exec('ababab'),
                b = /(?<x>a)(?<y>b)(?<last>(?<piece>\k<piece>\k<x>)\k<y>)+/d.exec('ababab');
            a.groups.piece === 'a' && b.groups.piece === 'a' && a.indices.groups.piece[0] === 4 && b
                .indices.groups.piece[1] === 5
        "#,
    );
    assert_true(
        r#"
            let a = /(?<x>.)(?<y>a)(?<last>(?<piece>\k<x>)\k<y>)+/d.exec('\uD800a\uD800a');
            a[0].length === 4 && a.groups.piece.charCodeAt(0) === 0xD800 && a.indices.groups.piece === a
                .indices[4] && a.indices[4][0] === 2 && a.indices[4][1] === 3
        "#,
    );
}

#[test]
fn partial_capture_consumers_and_external_following_references_keep_ranges() {
    assert_true(
        r#"
            let r = /(?<x>a)(?<y>b)(?<last>(?<piece>\k<x>)\k<y>)+/dg,
                a = [...'abab ababab'.matchAll(r)];
            a.length === 2 && a[0].groups.piece === 'a' && a[0].indices.groups.piece === a[0].indices[4] &&
                a[0].indices[4][0] === 2 && a[1].indices[4][0] === 9 && r.lastIndex === 0
        "#,
    );
    assert_true(
        r#"
            'qababab'.replace(/(?<x>a)(?<y>b)(?<last>(?<piece>\k<x>)\k<y>)+/, '<$<last>:$<piece>>') ===
                'q<ab:a>' && 'qababab'.search(/(a)(b)((\1)\2)+/) === 1 && 'qabababZ'.split(
                    /(a)(b)((\1)\2)+/).join(',') === 'q,a,b,ab,a,Z'
        "#,
    );
    assert_true(
        r#"
            let r = /(?<x>a)(?<y>b)(?<last>(?<piece>\k<x>)\k<y>)+/dy;
            r.lastIndex = 1;
            let copy = new RegExp(r),
                a = r.exec('qababab');
            a.indices.groups.piece[0] === 5 && a.indices.groups.piece[1] === 6 && r.lastIndex === 7 && r
                .exec('qababab') === null && r.lastIndex === 0 && copy.source === r.source && copy
                .lastIndex === 0
        "#,
    );
    assert_true(
        r#"
            let a = /(?<x>a)(?<y>b)(?<last>(?<piece>\k<x>)\k<y>)*\k<piece>$/d.exec('ababa'),
                b = /(?<x>a)(?<y>b)(?<last>\k<x>(?<piece>\k<y>))*?\k<piece>$/d.exec('ababb');
            a[0] === 'ababa' && a.groups.piece === 'a' && a.indices.groups.piece[0] === 2 && b[0] ===
                'ababb' && b.groups.piece === 'b' && b.indices.groups.piece[0] === 3
        "#,
    );
    assert_true(
        r#"
            let a = [...'q'.matchAll(/(?<x>)(?<y>)(?<last>(?<piece>\k<x>)\k<y>)*/dg)];
            a.length === 2 && a[0].groups.piece === undefined && a[0].indices.groups.piece === undefined &&
                a[1].index === 1
        "#,
    );
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
fn completed_local_targets_and_other_repeated_reference_bodies_match_reference_results() {
    for source in [
        r"/(?:(?:(?:(a)(b)((\1)\4\2)+){2})|){2,3}/.test('abab')",
        r"/(?:(?:(?:(?<x>a)(?<y>b)(?<last>(?<piece>\k<x>)\k<piece>\k<y>)+){2})|){2,3}/.test('aabaab')",
        r"/(?:(?:(?:(a)(b)(a(\1)\2)+){2})|){2,3}/.test('abab')",
        r"/(a)(b)((\1)+\2){2,3}/.test('abab')",
        r"/(a)(b)((\1)\2)+/u.test('abab')",
    ] {
        crate::common::regexp_oracle::assert_matches_reference(&mut Realm::default(), source);
    }
    assert_true(r"/(a)(b)((\1)\2)+/.exec('abc')===null");
}
