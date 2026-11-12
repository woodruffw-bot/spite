//! Repeated reference bodies retain pinned one-unit predicates and local ranges.

use spite_core::JsString;
use spite_runtime::{Error, Limits, Realm, Value};
use std::fmt::Write;

use crate::common::assert_true;

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
    insta::assert_snapshot!(rows, @r#"
        "(a)(a\u005c1)*" flags="d" input="aaaaa" "{\u0022matches\u0022:[\u0022aaaaa\u0022,\u0022a\u0022,\u0022aa\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022aaaaa\u0022,\u0022indices\u0022:[[0,5],[0,1],[3,5]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a)(a\u005c\u005c1)*\u0022}"
        "(a)(a\u005c1)*?" flags="d" input="aaaaa" "{\u0022matches\u0022:[\u0022a\u0022,\u0022a\u0022,null],\u0022index\u0022:0,\u0022input\u0022:\u0022aaaaa\u0022,\u0022indices\u0022:[[0,1],[0,1],null],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a)(a\u005c\u005c1)*?\u0022}"
        "(a)(b)(a\u005c1\u005c2)+" flags="d" input="abaabaab" "{\u0022matches\u0022:[\u0022abaabaab\u0022,\u0022a\u0022,\u0022b\u0022,\u0022aab\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022abaabaab\u0022,\u0022indices\u0022:[[0,8],[0,1],[1,2],[5,8]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a)(b)(a\u005c\u005c1\u005c\u005c2)+\u0022}"
        "(a)(b)(\u005c1b\u005c2)+?" flags="d" input="ababbabb" "{\u0022matches\u0022:[\u0022ababb\u0022,\u0022a\u0022,\u0022b\u0022,\u0022abb\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022ababbabb\u0022,\u0022indices\u0022:[[0,5],[0,1],[1,2],[2,5]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a)(b)(\u005c\u005c1b\u005c\u005c2)+?\u0022}"
        "(a)(b)([ab]\u005c1\u005c2)?" flags="d" input="abaab" "{\u0022matches\u0022:[\u0022abaab\u0022,\u0022a\u0022,\u0022b\u0022,\u0022aab\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022abaab\u0022,\u0022indices\u0022:[[0,5],[0,1],[1,2],[2,5]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a)(b)([ab]\u005c\u005c1\u005c\u005c2)?\u0022}"
        "(a)(b)(\u005c1[^c]\u005c2)??" flags="d" input="ababb" "{\u0022matches\u0022:[\u0022ab\u0022,\u0022a\u0022,\u0022b\u0022,null],\u0022index\u0022:0,\u0022input\u0022:\u0022ababb\u0022,\u0022indices\u0022:[[0,2],[0,1],[1,2],null],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a)(b)(\u005c\u005c1[^c]\u005c\u005c2)??\u0022}"
        "(a)(bb)(([ab])\u005c4\u005c2){1,3}" flags="d" input="abbaabbaabb" "{\u0022matches\u0022:[\u0022abbaabbaabb\u0022,\u0022a\u0022,\u0022bb\u0022,\u0022aabb\u0022,\u0022a\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022abbaabbaabb\u0022,\u0022indices\u0022:[[0,11],[0,1],[1,3],[7,11],[7,8]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a)(bb)(([ab])\u005c\u005c4\u005c\u005c2){1,3}\u0022}"
        "(a)(bb)(\u005c2(.)\u005c4){1,3}?" flags="d" input="abbbbaa" "{\u0022matches\u0022:[\u0022abbbbaa\u0022,\u0022a\u0022,\u0022bb\u0022,\u0022bbaa\u0022,\u0022a\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022abbbbaa\u0022,\u0022indices\u0022:[[0,7],[0,1],[1,3],[3,7],[5,6]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a)(bb)(\u005c\u005c2(.)\u005c\u005c4){1,3}?\u0022}"
        "(a)(b)(()(a)\u005c5\u005c2())*ab" flags="d" input="qabaabaab" "{\u0022match\u0022:null,\u0022lastIndex\u0022:0}"
        "(a)(b)(()\u005c1()([ab])\u005c6())*?ab" flags="d" input="qababb" "{\u0022matches\u0022:[\u0022abab\u0022,\u0022a\u0022,\u0022b\u0022,null,null,null,null,null],\u0022index\u0022:1,\u0022input\u0022:\u0022qababb\u0022,\u0022indices\u0022:[[1,5],[1,2],[2,3],null,null,null,null,null],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a)(b)(()\u005c\u005c1()([ab])\u005c\u005c6())*?ab\u0022}"
        "((a)(b)((a)\u005c5\u005c3)+)\u005c1" flags="d" input="abaababaab" "{\u0022matches\u0022:[\u0022abaababaab\u0022,\u0022abaab\u0022,\u0022a\u0022,\u0022b\u0022,\u0022aab\u0022,\u0022a\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022abaababaab\u0022,\u0022indices\u0022:[[0,10],[0,5],[0,1],[1,2],[2,5],[2,3]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022((a)(b)((a)\u005c\u005c5\u005c\u005c3)+)\u005c\u005c1\u0022}"
        "((a)(b)(\u005c2([ab])\u005c5)+?)\u005c1" flags="d" input="ababbababb" "{\u0022matches\u0022:[\u0022ababbababb\u0022,\u0022ababb\u0022,\u0022a\u0022,\u0022b\u0022,\u0022abb\u0022,\u0022b\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022ababbababb\u0022,\u0022indices\u0022:[[0,10],[0,5],[0,1],[1,2],[2,5],[3,4]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022((a)(b)(\u005c\u005c2([ab])\u005c\u005c5)+?)\u005c\u005c1\u0022}"
        "(?:(a)|(b))((a)\u005c4\u005c2)+c" flags="d" input="baabc" "{\u0022matches\u0022:[\u0022baabc\u0022,null,\u0022b\u0022,\u0022aab\u0022,\u0022a\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022baabc\u0022,\u0022indices\u0022:[[0,5],null,[0,1],[1,4],[1,2]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?:(a)|(b))((a)\u005c\u005c4\u005c\u005c2)+c\u0022}"
        "(?:(a)|(b))((\u005cw)\u005c4\u005c1)+c" flags="d" input="bbbc" "{\u0022matches\u0022:[\u0022bbbc\u0022,null,\u0022b\u0022,\u0022bb\u0022,\u0022b\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022bbbc\u0022,\u0022indices\u0022:[[0,4],null,[0,1],[1,3],[1,2]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?:(a)|(b))((\u005c\u005cw)\u005c\u005c4\u005c\u005c1)+c\u0022}"
        "(a)(b)((a)\u005c4\u005c2)*\u005c4" flags="d" input="abaaba" "{\u0022matches\u0022:[\u0022abaaba\u0022,\u0022a\u0022,\u0022b\u0022,\u0022aab\u0022,\u0022a\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022abaaba\u0022,\u0022indices\u0022:[[0,6],[0,1],[1,2],[2,5],[2,3]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a)(b)((a)\u005c\u005c4\u005c\u005c2)*\u005c\u005c4\u0022}"
        "(a)(b)(\u005c1([ab])\u005c4)*?\u005c4" flags="d" input="abbbbb" "{\u0022matches\u0022:[\u0022ab\u0022,\u0022a\u0022,\u0022b\u0022,null,null],\u0022index\u0022:0,\u0022input\u0022:\u0022abbbbb\u0022,\u0022indices\u0022:[[0,2],[0,1],[1,2],null,null],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a)(b)(\u005c\u005c1([ab])\u005c\u005c4)*?\u005c\u005c4\u0022}"
        "(a)(b)(\u005c4(a)\u005c4\u005c2)+" flags="d" input="abaabaab" "{\u0022matches\u0022:[\u0022abaabaab\u0022,\u0022a\u0022,\u0022b\u0022,\u0022aab\u0022,\u0022a\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022abaabaab\u0022,\u0022indices\u0022:[[0,8],[0,1],[1,2],[5,8],[5,6]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a)(b)(\u005c\u005c4(a)\u005c\u005c4\u005c\u005c2)+\u0022}"
        "(a)(b)((\u005c4a)\u005c4\u005c2)+" flags="d" input="abaabaab" "{\u0022matches\u0022:[\u0022abaabaab\u0022,\u0022a\u0022,\u0022b\u0022,\u0022aab\u0022,\u0022a\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022abaabaab\u0022,\u0022indices\u0022:[[0,8],[0,1],[1,2],[5,8],[5,6]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a)(b)((\u005c\u005c4a)\u005c\u005c4\u005c\u005c2)+\u0022}"
        "()()((a)\u005c4\u005c1\u005c2){2,3}" flags="d" input="aaaaaa" "{\u0022matches\u0022:[\u0022aaaaaa\u0022,\u0022\u0022,\u0022\u0022,\u0022aa\u0022,\u0022a\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022aaaaaa\u0022,\u0022indices\u0022:[[0,6],[0,0],[0,0],[4,6],[4,5]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022()()((a)\u005c\u005c4\u005c\u005c1\u005c\u005c2){2,3}\u0022}"
        "(?:(?<x>a)|(?<x>b))(?<y>c)(?<last>(?<piece>a)\u005ck<piece>\u005ck<y>)+d" flags="d" input="bcaacd" "{\u0022matches\u0022:[\u0022bcaacd\u0022,null,\u0022b\u0022,\u0022c\u0022,\u0022aac\u0022,\u0022a\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022bcaacd\u0022,\u0022groups\u0022:{\u0022x\u0022:\u0022b\u0022,\u0022y\u0022:\u0022c\u0022,\u0022last\u0022:\u0022aac\u0022,\u0022piece\u0022:\u0022a\u0022},\u0022indices\u0022:[[0,6],null,[0,1],[1,2],[2,5],[2,3]],\u0022indicesGroups\u0022:{\u0022x\u0022:[0,1],\u0022y\u0022:[1,2],\u0022last\u0022:[2,5],\u0022piece\u0022:[2,3]},\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?:(?<x>a)|(?<x>b))(?<y>c)(?<last>(?<piece>a)\u005c\u005ck<piece>\u005c\u005ck<y>)+d\u0022}"
        "(?<x>a)(?<y>b)(?<last>\u005ck<piece>(?<piece>[ab])\u005ck<piece>\u005ck<y>)+" flags="dgi" input="abaabaab" "{\u0022matches\u0022:[\u0022abaabaab\u0022,\u0022a\u0022,\u0022b\u0022,\u0022aab\u0022,\u0022a\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022abaabaab\u0022,\u0022groups\u0022:{\u0022x\u0022:\u0022a\u0022,\u0022y\u0022:\u0022b\u0022,\u0022last\u0022:\u0022aab\u0022,\u0022piece\u0022:\u0022a\u0022},\u0022indices\u0022:[[0,8],[0,1],[1,2],[5,8],[5,6]],\u0022indicesGroups\u0022:{\u0022x\u0022:[0,1],\u0022y\u0022:[1,2],\u0022last\u0022:[5,8],\u0022piece\u0022:[5,6]},\u0022lastIndex\u0022:8,\u0022source\u0022:\u0022(?<x>a)(?<y>b)(?<last>\u005c\u005ck<piece>(?<piece>[ab])\u005c\u005ck<piece>\u005c\u005ck<y>)+\u0022}"
        "(?<x>a)(?<last>(?<piece>a)\u005ck<piece>)+" flags="dy" input="aaaaa" "{\u0022matches\u0022:[\u0022aaaaa\u0022,\u0022a\u0022,\u0022aa\u0022,\u0022a\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022aaaaa\u0022,\u0022groups\u0022:{\u0022x\u0022:\u0022a\u0022,\u0022last\u0022:\u0022aa\u0022,\u0022piece\u0022:\u0022a\u0022},\u0022indices\u0022:[[0,5],[0,1],[3,5],[3,4]],\u0022indicesGroups\u0022:{\u0022x\u0022:[0,1],\u0022last\u0022:[3,5],\u0022piece\u0022:[3,4]},\u0022lastIndex\u0022:5,\u0022source\u0022:\u0022(?<x>a)(?<last>(?<piece>a)\u005c\u005ck<piece>)+\u0022}"
        "^(?<x>.)(?<y>a)(?<last>(?<piece>.)\u005ck<piece>\u005ck<y>)+$" flags="dis" input="\u00b5a\u039c\u00b5A\u00b5\u039ca" "{\u0022matches\u0022:[\u0022\u00b5a\u039c\u00b5A\u00b5\u039ca\u0022,\u0022\u00b5\u0022,\u0022a\u0022,\u0022\u00b5\u039ca\u0022,\u0022\u00b5\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022\u00b5a\u039c\u00b5A\u00b5\u039ca\u0022,\u0022groups\u0022:{\u0022x\u0022:\u0022\u00b5\u0022,\u0022y\u0022:\u0022a\u0022,\u0022last\u0022:\u0022\u00b5\u039ca\u0022,\u0022piece\u0022:\u0022\u00b5\u0022},\u0022indices\u0022:[[0,8],[0,1],[1,2],[5,8],[5,6]],\u0022indicesGroups\u0022:{\u0022x\u0022:[0,1],\u0022y\u0022:[1,2],\u0022last\u0022:[5,8],\u0022piece\u0022:[5,6]},\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022^(?<x>.)(?<y>a)(?<last>(?<piece>.)\u005c\u005ck<piece>\u005c\u005ck<y>)+$\u0022}"
        "#);
}

#[test]
fn repeated_character_terms_keep_current_named_values_and_retry_ranges() {
    assert_true(
        r#"
            let a = /(\1a)+/d.exec('aaa'),
                b = /((a)\1)+/d.exec('aaa'),
                c = /(?:(a)\1){2}/d.exec('aaaa');
            a[1] === 'a' && a.indices[1][0] === 2 && b[1] === 'a' && b[2] === 'a' && b.indices[2][0] ===
                2 && c[1] === 'a' && c.indices[1][0] === 2
        "#,
    );
    assert_true(
        r#"
            let a = /(?<x>a)(?<y>b)(?<last>(?<piece>a)\k<piece>\k<y>)*aab$/d.exec('abaabaab');
            a[0] === 'abaabaab' && a.groups.last === 'aab' && a.groups.piece === 'a' && a.indices.groups
                .last === a.indices[3] && a.indices.groups.last[0] === 2 && a.indices.groups.piece === a
                .indices[4] && a.indices[4][0] === 2 && a.indices[4][1] === 3
        "#,
    );
    assert_true(
        r#"
            let a = /((?<x>a)(?<y>b)(?<last>(?<piece>a)\k<piece>\k<y>)+?)\1/d.exec('abaababaab');
            a[0] === 'abaababaab' && a[1] === 'abaab' && a.groups.last === 'aab' && a.groups.piece ===
                'a' && a.indices.groups.piece === a.indices[5] && a.indices[5][0] === 2
        "#,
    );
    assert_true(
        r#"
            let a = /(?<x>a)(?<y>b)(?<last>(?<piece>[ab])\k<piece>\k<y>)+$/d.exec('abaabbbb');
            a[0] === 'abaabbbb' && a.groups.last === 'bbb' && a.groups.piece === 'b' && a.indices.groups
                .piece === a.indices[4] && a.indices[4][0] === 5 && a.indices[4][1] === 6
        "#,
    );
    assert_true(
        r#"
            let a = /(?:(?<x>a)|(?<x>b))(?<y>c)(?<last>(?<piece>a)\k<piece>\k<y>)+/d.exec('bcaac');
            a[1] === undefined && a.groups.x === 'b' && a.groups.piece === 'a' && a.indices.groups
                .piece === a.indices[5] && a.indices[5][0] === 2
        "#,
    );
    assert_true(
        r#"
            let a = /(?<x>a)(?<y>b)(?<last>(?<piece>.)\k<piece>\k<y>)+/ds.exec('ab\n\nb');
            a.groups.piece === '\n' && a.indices.groups.piece[0] === 2 &&
                /(?<x>a)(?<y>b)(?<last>(?<piece>.)\k<piece>\k<y>)+/.exec('ab\n\nb') === null
        "#,
    );
    assert_true(
        r#"
            let a = /(?<x>a)(?<y>b)(?<last>(?<piece>.)\k<piece>\k<y>)+/d.exec('ab\uD800\uD800b');
            a[0].length === 5 && a.groups.piece.charCodeAt(0) === 0xD800 && a.indices.groups.piece === a
                .indices[4] && a.indices[4][0] === 2 && a.indices[4][1] === 3
        "#,
    );
}

#[test]
fn repeated_character_consumers_preserve_partial_indices_and_final_values() {
    assert_true(
        r#"
            let r = /(?<x>a)(?<y>b)(?<last>(?<piece>[ab])\k<piece>\k<y>)+/dg,
                a = [...'abaab abaabbbb'.matchAll(r)];
            a.length === 2 && a[0].groups.last === 'aab' && a[1].groups.last === 'bbb' && a[1].groups
                .piece === 'b' && a[1].indices.groups.piece === a[1].indices[4] && a[1].indices[4][0] ===
                11 && r.lastIndex === 0
        "#,
    );
    assert_true(
        r#"
            'qabaabbbb'.replace(/(?<x>a)(?<y>b)(?<last>(?<piece>[ab])\k<piece>\k<y>)+/,
                    '<$<last>:$<piece>>') === 'q<bbb:b>' && 'qabaabbbb'.search(/(a)(b)(([ab])\4\2)+/) ===
                1 && 'qabaabbbbZ'.split(/(a)(b)(([ab])\4\2)+/).join(',') === 'q,a,b,bbb,b,Z'
        "#,
    );
    assert_true(
        r#"
            let r = /(?<x>a)(?<y>b)(?<last>(?<piece>[ab])\k<piece>\k<y>)+/dy;
            r.lastIndex = 1;
            let copy = new RegExp(r),
                a = r.exec('qabaabbbb');
            a.groups.piece === 'b' && a.indices.groups.piece[0] === 6 && a.indices.groups.piece[1] === 7 &&
                r.lastIndex === 9 && r.exec('qabaabbbb') === null && r.lastIndex === 0 && copy.source === r
                .source && copy.lastIndex === 0
        "#,
    );
    assert_true(
        r#"
            let a = /(?<x>a)(?<y>b)(?<last>(?<piece>[ab])\k<piece>\k<y>)*\k<piece>$/d.exec('abbbbb'),
                b = /(?<x>a)(?<y>b)(?<last>(?<piece>[ab])\k<piece>\k<y>)*?\k<piece>$/d.exec('abbbbb');
            a[0] === 'abbbbb' && b[0] === 'abbbbb' && a.groups.piece === 'b' && b.groups.piece === 'b' && a
                .indices.groups.piece[0] === 2 && b.indices.groups.piece[0] === 2
        "#,
    );
    assert_true(
        r#"
            let a = /(?<x>a)(?<last>a\k<x>)* /d.exec('a ');
            a.groups.last === undefined && a.indices.groups.last === undefined
        "#,
    );
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
fn repeated_assertions_choices_and_nested_quantifiers_match_reference_results() {
    for source in [
        r"/(?:(?:(?:(a)(b)(\b\1\2)+){2})|){2,3}/.test('abab')",
        r"/(a)(b)((\1)|\2){2,3}/.test('abab')",
        r"/(?:(?:(a)(b)(a+\1\2)+)|){2,3}/.test('aabaab')",
        r"/(a)(b)((?=a)\1\2){2,3}/.test('abab')",
        r"/(a)(b)((?i:\1)\2)+/.test('abab')",
        r"/(a)(b)(a\1\2)+/u.test('aabaab')",
    ] {
        crate::common::regexp_oracle::assert_matches_reference(&mut Realm::default(), source);
    }
    assert_true(r"/(a)(b)(a\1\2)+/.exec('abc')===null");
}
