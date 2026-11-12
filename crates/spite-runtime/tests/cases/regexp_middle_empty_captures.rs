//! Empty captures between repeated references retain their final positions.

use spite_core::JsString;
use spite_runtime::{Error, Limits, Realm, Value};
use std::fmt::Write;

use crate::common::assert_true;

#[test]
fn middle_empty_capture_result_snapshot() {
    let mut rows = String::new();
    for (source, flags, text) in [
        (r"(a)(\1()\1)*", "d", "aaaaa"),
        (r"(a)(\1()\1)*?", "d", "aaaaa"),
        (r"(a)(b)(\1()\2)+", "d", "ababab"),
        (r"(a)(b)(\1()\2)+?", "d", "ababab"),
        (r"(a)(b)(\1()\2)?", "d", "abab"),
        (r"(a)(b)(\1()\2)??", "d", "abab"),
        (r"(a)(bb)(\2()\1){1,3}", "d", "abbbbabba"),
        (r"(a)(bb)(\2()\1){1,3}?", "d", "abbbbabba"),
        (r"(a)(b)(()\1()\2())*ab", "d", "qababab"),
        (r"(a)(b)(()\1()\2())*?ab", "d", "qababab"),
        (r"((a)(b)(\2()\3)+)\1", "d", "abababab"),
        (r"((a)(b)(\2()\3)+?)\1", "d", "abababab"),
        (r"(?:(a)|(b))(\1()\2)+c", "d", "bbc"),
        (r"(?:(a)|(b))(\2()\1)+c", "d", "bbc"),
        (r"(a)(b)(\1()\4\2)*(\2()\1)*", "d", "ababbaba"),
        (r"(a)(b)(\1()\4\2)*?(\2()\1)*?", "d", "ababbaba"),
        (r"(\3()\4)+(a)(b)", "d", "ab"),
        (r"((\1()\4)+a)(b)", "d", "ab"),
        (r"()()(\1()\2){2,3}", "d", "q"),
        (
            r"(?:(?<x>a)|(?<x>b))(?<y>c)(?<last>(?<before>)\k<x>(?<mid>)\k<y>(?<after>))+d",
            "d",
            "bcbcd",
        ),
        (r"(?<x>(\k<x>(?<mid>)\k<y>)+a)(?<y>b)", "dgi", "ab"),
        (r"(?<x>a)(?<last>\1(?<mid>)\k<x>)+", "dy", "aaaaa"),
        (
            r"^(?<x>.)(?<y>a)(?<last>(?<before>)\k<x>(?<mid>)\k<y>(?<after>))+$",
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
        "(a)(\u005c1()\u005c1)*" flags="d" input="aaaaa" "{\u0022matches\u0022:[\u0022aaaaa\u0022,\u0022a\u0022,\u0022aa\u0022,\u0022\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022aaaaa\u0022,\u0022indices\u0022:[[0,5],[0,1],[3,5],[4,4]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a)(\u005c\u005c1()\u005c\u005c1)*\u0022}"
        "(a)(\u005c1()\u005c1)*?" flags="d" input="aaaaa" "{\u0022matches\u0022:[\u0022a\u0022,\u0022a\u0022,null,null],\u0022index\u0022:0,\u0022input\u0022:\u0022aaaaa\u0022,\u0022indices\u0022:[[0,1],[0,1],null,null],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a)(\u005c\u005c1()\u005c\u005c1)*?\u0022}"
        "(a)(b)(\u005c1()\u005c2)+" flags="d" input="ababab" "{\u0022matches\u0022:[\u0022ababab\u0022,\u0022a\u0022,\u0022b\u0022,\u0022ab\u0022,\u0022\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022ababab\u0022,\u0022indices\u0022:[[0,6],[0,1],[1,2],[4,6],[5,5]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a)(b)(\u005c\u005c1()\u005c\u005c2)+\u0022}"
        "(a)(b)(\u005c1()\u005c2)+?" flags="d" input="ababab" "{\u0022matches\u0022:[\u0022abab\u0022,\u0022a\u0022,\u0022b\u0022,\u0022ab\u0022,\u0022\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022ababab\u0022,\u0022indices\u0022:[[0,4],[0,1],[1,2],[2,4],[3,3]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a)(b)(\u005c\u005c1()\u005c\u005c2)+?\u0022}"
        "(a)(b)(\u005c1()\u005c2)?" flags="d" input="abab" "{\u0022matches\u0022:[\u0022abab\u0022,\u0022a\u0022,\u0022b\u0022,\u0022ab\u0022,\u0022\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022abab\u0022,\u0022indices\u0022:[[0,4],[0,1],[1,2],[2,4],[3,3]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a)(b)(\u005c\u005c1()\u005c\u005c2)?\u0022}"
        "(a)(b)(\u005c1()\u005c2)??" flags="d" input="abab" "{\u0022matches\u0022:[\u0022ab\u0022,\u0022a\u0022,\u0022b\u0022,null,null],\u0022index\u0022:0,\u0022input\u0022:\u0022abab\u0022,\u0022indices\u0022:[[0,2],[0,1],[1,2],null,null],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a)(b)(\u005c\u005c1()\u005c\u005c2)??\u0022}"
        "(a)(bb)(\u005c2()\u005c1){1,3}" flags="d" input="abbbbabba" "{\u0022matches\u0022:[\u0022abbbbabba\u0022,\u0022a\u0022,\u0022bb\u0022,\u0022bba\u0022,\u0022\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022abbbbabba\u0022,\u0022indices\u0022:[[0,9],[0,1],[1,3],[6,9],[8,8]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a)(bb)(\u005c\u005c2()\u005c\u005c1){1,3}\u0022}"
        "(a)(bb)(\u005c2()\u005c1){1,3}?" flags="d" input="abbbbabba" "{\u0022matches\u0022:[\u0022abbbba\u0022,\u0022a\u0022,\u0022bb\u0022,\u0022bba\u0022,\u0022\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022abbbbabba\u0022,\u0022indices\u0022:[[0,6],[0,1],[1,3],[3,6],[5,5]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a)(bb)(\u005c\u005c2()\u005c\u005c1){1,3}?\u0022}"
        "(a)(b)(()\u005c1()\u005c2())*ab" flags="d" input="qababab" "{\u0022matches\u0022:[\u0022ababab\u0022,\u0022a\u0022,\u0022b\u0022,\u0022ab\u0022,\u0022\u0022,\u0022\u0022,\u0022\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022qababab\u0022,\u0022indices\u0022:[[1,7],[1,2],[2,3],[3,5],[3,3],[4,4],[5,5]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a)(b)(()\u005c\u005c1()\u005c\u005c2())*ab\u0022}"
        "(a)(b)(()\u005c1()\u005c2())*?ab" flags="d" input="qababab" "{\u0022matches\u0022:[\u0022abab\u0022,\u0022a\u0022,\u0022b\u0022,null,null,null,null],\u0022index\u0022:1,\u0022input\u0022:\u0022qababab\u0022,\u0022indices\u0022:[[1,5],[1,2],[2,3],null,null,null,null],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a)(b)(()\u005c\u005c1()\u005c\u005c2())*?ab\u0022}"
        "((a)(b)(\u005c2()\u005c3)+)\u005c1" flags="d" input="abababab" "{\u0022matches\u0022:[\u0022abababab\u0022,\u0022abab\u0022,\u0022a\u0022,\u0022b\u0022,\u0022ab\u0022,\u0022\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022abababab\u0022,\u0022indices\u0022:[[0,8],[0,4],[0,1],[1,2],[2,4],[3,3]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022((a)(b)(\u005c\u005c2()\u005c\u005c3)+)\u005c\u005c1\u0022}"
        "((a)(b)(\u005c2()\u005c3)+?)\u005c1" flags="d" input="abababab" "{\u0022matches\u0022:[\u0022abababab\u0022,\u0022abab\u0022,\u0022a\u0022,\u0022b\u0022,\u0022ab\u0022,\u0022\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022abababab\u0022,\u0022indices\u0022:[[0,8],[0,4],[0,1],[1,2],[2,4],[3,3]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022((a)(b)(\u005c\u005c2()\u005c\u005c3)+?)\u005c\u005c1\u0022}"
        "(?:(a)|(b))(\u005c1()\u005c2)+c" flags="d" input="bbc" "{\u0022matches\u0022:[\u0022bbc\u0022,null,\u0022b\u0022,\u0022b\u0022,\u0022\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022bbc\u0022,\u0022indices\u0022:[[0,3],null,[0,1],[1,2],[1,1]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?:(a)|(b))(\u005c\u005c1()\u005c\u005c2)+c\u0022}"
        "(?:(a)|(b))(\u005c2()\u005c1)+c" flags="d" input="bbc" "{\u0022matches\u0022:[\u0022bbc\u0022,null,\u0022b\u0022,\u0022b\u0022,\u0022\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022bbc\u0022,\u0022indices\u0022:[[0,3],null,[0,1],[1,2],[2,2]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?:(a)|(b))(\u005c\u005c2()\u005c\u005c1)+c\u0022}"
        "(a)(b)(\u005c1()\u005c4\u005c2)*(\u005c2()\u005c1)*" flags="d" input="ababbaba" "{\u0022matches\u0022:[\u0022ababbaba\u0022,\u0022a\u0022,\u0022b\u0022,\u0022ab\u0022,\u0022\u0022,\u0022ba\u0022,\u0022\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022ababbaba\u0022,\u0022indices\u0022:[[0,8],[0,1],[1,2],[2,4],[3,3],[6,8],[7,7]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a)(b)(\u005c\u005c1()\u005c\u005c4\u005c\u005c2)*(\u005c\u005c2()\u005c\u005c1)*\u0022}"
        "(a)(b)(\u005c1()\u005c4\u005c2)*?(\u005c2()\u005c1)*?" flags="d" input="ababbaba" "{\u0022matches\u0022:[\u0022ab\u0022,\u0022a\u0022,\u0022b\u0022,null,null,null,null],\u0022index\u0022:0,\u0022input\u0022:\u0022ababbaba\u0022,\u0022indices\u0022:[[0,2],[0,1],[1,2],null,null,null,null],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a)(b)(\u005c\u005c1()\u005c\u005c4\u005c\u005c2)*?(\u005c\u005c2()\u005c\u005c1)*?\u0022}"
        "(\u005c3()\u005c4)+(a)(b)" flags="d" input="ab" "{\u0022matches\u0022:[\u0022ab\u0022,\u0022\u0022,\u0022\u0022,\u0022a\u0022,\u0022b\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022ab\u0022,\u0022indices\u0022:[[0,2],[0,0],[0,0],[0,1],[1,2]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(\u005c\u005c3()\u005c\u005c4)+(a)(b)\u0022}"
        "((\u005c1()\u005c4)+a)(b)" flags="d" input="ab" "{\u0022matches\u0022:[\u0022ab\u0022,\u0022a\u0022,\u0022\u0022,\u0022\u0022,\u0022b\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022ab\u0022,\u0022indices\u0022:[[0,2],[0,1],[0,0],[0,0],[1,2]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022((\u005c\u005c1()\u005c\u005c4)+a)(b)\u0022}"
        "()()(\u005c1()\u005c2){2,3}" flags="d" input="q" "{\u0022matches\u0022:[\u0022\u0022,\u0022\u0022,\u0022\u0022,\u0022\u0022,\u0022\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022q\u0022,\u0022indices\u0022:[[0,0],[0,0],[0,0],[0,0],[0,0]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022()()(\u005c\u005c1()\u005c\u005c2){2,3}\u0022}"
        "(?:(?<x>a)|(?<x>b))(?<y>c)(?<last>(?<before>)\u005ck<x>(?<mid>)\u005ck<y>(?<after>))+d" flags="d" input="bcbcd" "{\u0022matches\u0022:[\u0022bcbcd\u0022,null,\u0022b\u0022,\u0022c\u0022,\u0022bc\u0022,\u0022\u0022,\u0022\u0022,\u0022\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022bcbcd\u0022,\u0022groups\u0022:{\u0022x\u0022:\u0022b\u0022,\u0022y\u0022:\u0022c\u0022,\u0022last\u0022:\u0022bc\u0022,\u0022before\u0022:\u0022\u0022,\u0022mid\u0022:\u0022\u0022,\u0022after\u0022:\u0022\u0022},\u0022indices\u0022:[[0,5],null,[0,1],[1,2],[2,4],[2,2],[3,3],[4,4]],\u0022indicesGroups\u0022:{\u0022x\u0022:[0,1],\u0022y\u0022:[1,2],\u0022last\u0022:[2,4],\u0022before\u0022:[2,2],\u0022mid\u0022:[3,3],\u0022after\u0022:[4,4]},\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?:(?<x>a)|(?<x>b))(?<y>c)(?<last>(?<before>)\u005c\u005ck<x>(?<mid>)\u005c\u005ck<y>(?<after>))+d\u0022}"
        "(?<x>(\u005ck<x>(?<mid>)\u005ck<y>)+a)(?<y>b)" flags="dgi" input="ab" "{\u0022matches\u0022:[\u0022ab\u0022,\u0022a\u0022,\u0022\u0022,\u0022\u0022,\u0022b\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022ab\u0022,\u0022groups\u0022:{\u0022x\u0022:\u0022a\u0022,\u0022mid\u0022:\u0022\u0022,\u0022y\u0022:\u0022b\u0022},\u0022indices\u0022:[[0,2],[0,1],[0,0],[0,0],[1,2]],\u0022indicesGroups\u0022:{\u0022x\u0022:[0,1],\u0022mid\u0022:[0,0],\u0022y\u0022:[1,2]},\u0022lastIndex\u0022:2,\u0022source\u0022:\u0022(?<x>(\u005c\u005ck<x>(?<mid>)\u005c\u005ck<y>)+a)(?<y>b)\u0022}"
        "(?<x>a)(?<last>\u005c1(?<mid>)\u005ck<x>)+" flags="dy" input="aaaaa" "{\u0022matches\u0022:[\u0022aaaaa\u0022,\u0022a\u0022,\u0022aa\u0022,\u0022\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022aaaaa\u0022,\u0022groups\u0022:{\u0022x\u0022:\u0022a\u0022,\u0022last\u0022:\u0022aa\u0022,\u0022mid\u0022:\u0022\u0022},\u0022indices\u0022:[[0,5],[0,1],[3,5],[4,4]],\u0022indicesGroups\u0022:{\u0022x\u0022:[0,1],\u0022last\u0022:[3,5],\u0022mid\u0022:[4,4]},\u0022lastIndex\u0022:5,\u0022source\u0022:\u0022(?<x>a)(?<last>\u005c\u005c1(?<mid>)\u005c\u005ck<x>)+\u0022}"
        "^(?<x>.)(?<y>a)(?<last>(?<before>)\u005ck<x>(?<mid>)\u005ck<y>(?<after>))+$" flags="dis" input="\u00b5a\u039cA\u00b5a" "{\u0022matches\u0022:[\u0022\u00b5a\u039cA\u00b5a\u0022,\u0022\u00b5\u0022,\u0022a\u0022,\u0022\u00b5a\u0022,\u0022\u0022,\u0022\u0022,\u0022\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022\u00b5a\u039cA\u00b5a\u0022,\u0022groups\u0022:{\u0022x\u0022:\u0022\u00b5\u0022,\u0022y\u0022:\u0022a\u0022,\u0022last\u0022:\u0022\u00b5a\u0022,\u0022before\u0022:\u0022\u0022,\u0022mid\u0022:\u0022\u0022,\u0022after\u0022:\u0022\u0022},\u0022indices\u0022:[[0,6],[0,1],[1,2],[4,6],[4,4],[5,5],[6,6]],\u0022indicesGroups\u0022:{\u0022x\u0022:[0,1],\u0022y\u0022:[1,2],\u0022last\u0022:[4,6],\u0022before\u0022:[4,4],\u0022mid\u0022:[5,5],\u0022after\u0022:[6,6]},\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022^(?<x>.)(?<y>a)(?<last>(?<before>)\u005c\u005ck<x>(?<mid>)\u005c\u005ck<y>(?<after>))+$\u0022}"
        "#);
}

#[test]
fn middle_empty_captures_restore_named_positions_on_greedy_and_lazy_retries() {
    assert_true(
        r#"
            let a = /(?<x>a)(?<y>b)(?<last>(?<before>)\k<x>(?<mid>)\k<y>(?<after>))*ab$/d.exec('ababab');
            a[0] === 'ababab' && a.groups.last === 'ab' && a.indices.groups.last === a.indices[3] && a
                .indices[3][0] === 2 && a.indices[3][1] === 4 && a.indices.groups.before[0] === 2 && a
                .indices.groups.mid === a.indices[5] && a.indices[5][0] === 3 && a.indices[5][1] === 3 && a
                .indices.groups.after[0] === 4
        "#,
    );
    assert_true(
        r#"
            let a = /((?<x>a)(?<y>b)(?<last>\k<x>(?<mid>)\k<y>)+?)\1/d.exec('abababab');
            a[0] === 'abababab' && a[1] === 'abab' && a.groups.last === 'ab' && a.indices.groups.mid === a
                .indices[5] && a.indices[5][0] === 3
        "#,
    );
    assert_true(
        r#"
            let a = /(?<x>a)(?<y>bb)(?<last>\k<y>(?<mid>)\k<x>)+$/d.exec('abbbbabba');
            a[0] === 'abbbbabba' && a.groups.last === 'bba' && a.indices.groups.last[0] === 6 && a.indices
                .groups.mid === a.indices[4] && a.indices[4][0] === 8
        "#,
    );
    assert_true(
        r#"
            let a = /(?:(?<x>a)|(?<x>b))(?<last>\1(?<mid>)\2)+c/d.exec('bbc');
            a[0] === 'bbc' && a[1] === undefined && a.groups.x === 'b' && a.groups.mid === '' && a.indices
                .groups.mid === a.indices[4] && a.indices[4][0] === 1
        "#,
    );
    assert_true(
        r#"
            let a = /(?<x>a)(?<last>\1(?<mid>)\k<x>)+/d.exec('aaaaa');
            a[0] === 'aaaaa' && a.groups.last === 'aa' && a.indices.groups.mid === a.indices[3] && a
                .indices[3][0] === 4
        "#,
    );
    assert_true(
        r#"
            let a = /(?<x>.)(?<y>a)(?<last>\k<x>(?<mid>)\k<y>)+/d.exec('\uD800a\uD800a');
            a[0].length === 4 && a.groups.last.charCodeAt(0) === 0xD800 && a.indices.groups.mid === a
                .indices[4] && a.indices[4][0] === 3
        "#,
    );
}

#[test]
fn middle_capture_consumers_and_empty_iterations_keep_original_numbering() {
    assert_true(
        r#"
            let r = /(?<x>a)(?<y>b)(?<last>\k<x>(?<mid>)\k<y>)+/dg,
                a = [...'abab ababab'.matchAll(r)];
            a.length === 2 && a[0].groups.mid === '' && a[0].indices.groups.mid === a[0].indices[4] && a[0]
                .indices[4][0] === 3 && a[1].indices[4][0] === 10 && r.lastIndex === 0
        "#,
    );
    assert_true(
        r#"
            'qababab'.replace(/(?<x>a)(?<y>b)(?<last>\k<x>(?<mid>)\k<y>)+/, '<$<last>:$<mid>>') ===
                'q<ab:>' && 'qababab'.search(/(a)(b)(\1()\2)+/) === 1 && 'qabababZ'.split(
                    /(a)(b)(\1()\2)+/).join(',') === 'q,a,b,ab,,Z'
        "#,
    );
    assert_true(
        r#"
            let r = /(?<x>a)(?<y>b)(?<last>\k<x>(?<mid>)\k<y>)+/dy;
            r.lastIndex = 1;
            let copy = new RegExp(r),
                a = r.exec('qababab');
            a[0] === 'ababab' && a.indices.groups.mid[0] === 6 && r.lastIndex === 7 && r.exec(
                    'qababab') === null && r.lastIndex === 0 && copy.source === r.source && copy
                .lastIndex ===
                0
        "#,
    );
    assert_true(
        r#"
            let a = [...'q'.matchAll(/(?<x>)(?<y>)(?<last>\k<x>(?<mid>)\k<y>)*/dg)],
                b = /(?<x>(\k<x>(?<mid>)\k<y>)+a)(?<y>b)/d.exec('ab');
            a.length === 2 && a[0].groups.last === undefined && a[0].groups.mid === undefined && a[0]
                .indices.groups.mid === undefined && a[1].index === 1 && b.groups.mid === '' && b.indices
                .groups.mid[0] === 0
        "#,
    );
}

#[test]
fn explicit_middle_capture_work_aborts_before_last_index_and_handlers() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm.eval(r"let marker=0,r=/(?<x>a)(?<y>b)(?<last>\k<x>(?<mid>)\k<y>)*c/g;r.lastIndex=1;let text='ab'.repeat(5000)").unwrap();
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
fn partial_nonempty_captures_and_other_reference_bodies_match_reference_results() {
    for source in [
        r"/(?:(?:(?:(a)(b)(\1(\2))+){2})|){2,3}/.test('abab')",
        r"/(?:(?:(?:(a)(b)(a\1()\2)+){2})|){2,3}/.test('abab')",
        r"/(a)(b)(\1|()\2){2,3}/.test('abab')",
        r"/(a)(b)(\1+()\2){2,3}/.test('abab')",
        r"/(a)(b)(\1()\2)+/u.test('abab')",
    ] {
        crate::common::regexp_oracle::assert_matches_reference(&mut Realm::default(), source);
    }
    assert_true(r"/(a)(b)(\1()\2)+/.exec('abc')===null");
}
