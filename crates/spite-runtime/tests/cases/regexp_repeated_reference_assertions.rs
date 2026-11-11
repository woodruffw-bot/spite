//! Assertions in repeated reference bodies use the complete input context.

use spite_core::JsString;
use spite_runtime::{Error, Limits, Realm, Value};
use std::fmt::Write;

use crate::common::assert_true;

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
    insta::assert_snapshot!(rows, @r#"
        "(a)(b)(\u005cb\u005c1\u005c2)*" flags="d" input="ababab" "{\u0022matches\u0022:[\u0022ab\u0022,\u0022a\u0022,\u0022b\u0022,null],\u0022index\u0022:0,\u0022input\u0022:\u0022ababab\u0022,\u0022indices\u0022:[[0,2],[0,1],[1,2],null],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a)(b)(\u005c\u005cb\u005c\u005c1\u005c\u005c2)*\u0022}"
        "(a)(b)(\u005cb\u005c1\u005c2)*?" flags="d" input="ababab" "{\u0022matches\u0022:[\u0022ab\u0022,\u0022a\u0022,\u0022b\u0022,null],\u0022index\u0022:0,\u0022input\u0022:\u0022ababab\u0022,\u0022indices\u0022:[[0,2],[0,1],[1,2],null],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a)(b)(\u005c\u005cb\u005c\u005c1\u005c\u005c2)*?\u0022}"
        "(a)(b)(\u005cB\u005c1\u005c2)+" flags="d" input="ababab" "{\u0022matches\u0022:[\u0022ababab\u0022,\u0022a\u0022,\u0022b\u0022,\u0022ab\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022ababab\u0022,\u0022indices\u0022:[[0,6],[0,1],[1,2],[4,6]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a)(b)(\u005c\u005cB\u005c\u005c1\u005c\u005c2)+\u0022}"
        "(a)(b)(\u005c1\u005cB\u005c2)+?" flags="d" input="ababab" "{\u0022matches\u0022:[\u0022abab\u0022,\u0022a\u0022,\u0022b\u0022,\u0022ab\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022ababab\u0022,\u0022indices\u0022:[[0,4],[0,1],[1,2],[2,4]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a)(b)(\u005c\u005c1\u005c\u005cB\u005c\u005c2)+?\u0022}"
        "(a)(b)(\u005c1\u005c2$)?" flags="d" input="abab" "{\u0022matches\u0022:[\u0022abab\u0022,\u0022a\u0022,\u0022b\u0022,\u0022ab\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022abab\u0022,\u0022indices\u0022:[[0,4],[0,1],[1,2],[2,4]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a)(b)(\u005c\u005c1\u005c\u005c2$)?\u0022}"
        "(a)(b)(^\u005c1\u005c2)??" flags="d" input="abab" "{\u0022matches\u0022:[\u0022ab\u0022,\u0022a\u0022,\u0022b\u0022,null],\u0022index\u0022:0,\u0022input\u0022:\u0022abab\u0022,\u0022indices\u0022:[[0,2],[0,1],[1,2],null],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a)(b)(^\u005c\u005c1\u005c\u005c2)??\u0022}"
        "(a)(bb)((\u005cB\u005c2)\u005c4\u005c1){1,3}" flags="d" input="abbbbbba" "{\u0022matches\u0022:[\u0022abbbbbba\u0022,\u0022a\u0022,\u0022bb\u0022,\u0022bbbba\u0022,\u0022bb\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022abbbbbba\u0022,\u0022indices\u0022:[[0,8],[0,1],[1,3],[3,8],[3,5]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a)(bb)((\u005c\u005cB\u005c\u005c2)\u005c\u005c4\u005c\u005c1){1,3}\u0022}"
        "(a)(bb)(\u005c2(.)(\u005cB)\u005c4){1,3}?" flags="d" input="abbbbaa" "{\u0022matches\u0022:[\u0022abbbbaa\u0022,\u0022a\u0022,\u0022bb\u0022,\u0022bbaa\u0022,\u0022a\u0022,\u0022\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022abbbbaa\u0022,\u0022indices\u0022:[[0,7],[0,1],[1,3],[3,7],[5,6],[6,6]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a)(bb)(\u005c\u005c2(.)(\u005c\u005cB)\u005c\u005c4){1,3}?\u0022}"
        "(a)(b)(()(\u005cBa)\u005c5\u005c2())*ab" flags="d" input="qabaabab" "{\u0022matches\u0022:[\u0022abaabab\u0022,\u0022a\u0022,\u0022b\u0022,\u0022aab\u0022,\u0022\u0022,\u0022a\u0022,\u0022\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022qabaabab\u0022,\u0022indices\u0022:[[1,8],[1,2],[2,3],[3,6],[3,3],[3,4],[6,6]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a)(b)(()(\u005c\u005cBa)\u005c\u005c5\u005c\u005c2())*ab\u0022}"
        "(a)(b)(()\u005c1()([ab])(\u005cB)\u005c6())*?ab" flags="d" input="qababbab" "{\u0022matches\u0022:[\u0022abab\u0022,\u0022a\u0022,\u0022b\u0022,null,null,null,null,null,null],\u0022index\u0022:1,\u0022input\u0022:\u0022qababbab\u0022,\u0022indices\u0022:[[1,5],[1,2],[2,3],null,null,null,null,null,null],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a)(b)(()\u005c\u005c1()([ab])(\u005c\u005cB)\u005c\u005c6())*?ab\u0022}"
        "((a)(b)((\u005cBa)\u005c5\u005c3)+)\u005c1" flags="d" input="abaababaab" "{\u0022matches\u0022:[\u0022abaababaab\u0022,\u0022abaab\u0022,\u0022a\u0022,\u0022b\u0022,\u0022aab\u0022,\u0022a\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022abaababaab\u0022,\u0022indices\u0022:[[0,10],[0,5],[0,1],[1,2],[2,5],[2,3]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022((a)(b)((\u005c\u005cBa)\u005c\u005c5\u005c\u005c3)+)\u005c\u005c1\u0022}"
        "((a)(b)(\u005c2([ab])\u005cB\u005c5)+?)\u005c1" flags="d" input="ababbababb" "{\u0022matches\u0022:[\u0022ababbababb\u0022,\u0022ababb\u0022,\u0022a\u0022,\u0022b\u0022,\u0022abb\u0022,\u0022b\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022ababbababb\u0022,\u0022indices\u0022:[[0,10],[0,5],[0,1],[1,2],[2,5],[3,4]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022((a)(b)(\u005c\u005c2([ab])\u005c\u005cB\u005c\u005c5)+?)\u005c\u005c1\u0022}"
        "(?:(a)|(b))((\u005cBa)\u005c4\u005c2)+c" flags="d" input="baabc" "{\u0022matches\u0022:[\u0022baabc\u0022,null,\u0022b\u0022,\u0022aab\u0022,\u0022a\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022baabc\u0022,\u0022indices\u0022:[[0,5],null,[0,1],[1,4],[1,2]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?:(a)|(b))((\u005c\u005cBa)\u005c\u005c4\u005c\u005c2)+c\u0022}"
        "(?:(a)|(b))((\u005cw)\u005cB\u005c4\u005c1)+c" flags="d" input="bbbc" "{\u0022matches\u0022:[\u0022bbbc\u0022,null,\u0022b\u0022,\u0022bb\u0022,\u0022b\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022bbbc\u0022,\u0022indices\u0022:[[0,4],null,[0,1],[1,3],[1,2]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?:(a)|(b))((\u005c\u005cw)\u005c\u005cB\u005c\u005c4\u005c\u005c1)+c\u0022}"
        "(a)(b)((a)\u005cB\u005c4\u005c2)*\u005c4" flags="d" input="abaaba" "{\u0022matches\u0022:[\u0022abaaba\u0022,\u0022a\u0022,\u0022b\u0022,\u0022aab\u0022,\u0022a\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022abaaba\u0022,\u0022indices\u0022:[[0,6],[0,1],[1,2],[2,5],[2,3]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a)(b)((a)\u005c\u005cB\u005c\u005c4\u005c\u005c2)*\u005c\u005c4\u0022}"
        "(a)(b)(\u005c1([ab])\u005cB\u005c4)*?\u005c4" flags="d" input="abbbbb" "{\u0022matches\u0022:[\u0022ab\u0022,\u0022a\u0022,\u0022b\u0022,null,null],\u0022index\u0022:0,\u0022input\u0022:\u0022abbbbb\u0022,\u0022indices\u0022:[[0,2],[0,1],[1,2],null,null],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a)(b)(\u005c\u005c1([ab])\u005c\u005cB\u005c\u005c4)*?\u005c\u005c4\u0022}"
        "(a)(b)(\u005c4(a)\u005cB\u005c4\u005c2)+" flags="d" input="abaabaab" "{\u0022matches\u0022:[\u0022abaabaab\u0022,\u0022a\u0022,\u0022b\u0022,\u0022aab\u0022,\u0022a\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022abaabaab\u0022,\u0022indices\u0022:[[0,8],[0,1],[1,2],[5,8],[5,6]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a)(b)(\u005c\u005c4(a)\u005c\u005cB\u005c\u005c4\u005c\u005c2)+\u0022}"
        "(a)(b)((\u005c4a)\u005cB\u005c4\u005c2)+" flags="d" input="abaabaab" "{\u0022matches\u0022:[\u0022abaabaab\u0022,\u0022a\u0022,\u0022b\u0022,\u0022aab\u0022,\u0022a\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022abaabaab\u0022,\u0022indices\u0022:[[0,8],[0,1],[1,2],[5,8],[5,6]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a)(b)((\u005c\u005c4a)\u005c\u005cB\u005c\u005c4\u005c\u005c2)+\u0022}"
        "()()((^)\u005c1\u005c2){2,3}" flags="d" input="q" "{\u0022matches\u0022:[\u0022\u0022,\u0022\u0022,\u0022\u0022,\u0022\u0022,\u0022\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022q\u0022,\u0022indices\u0022:[[0,0],[0,0],[0,0],[0,0],[0,0]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022()()((^)\u005c\u005c1\u005c\u005c2){2,3}\u0022}"
        "(?:(?<x>a)|(?<x>b))(?<y>c)(?<last>(?<piece>a)\u005cB\u005ck<piece>\u005ck<y>)+d" flags="d" input="bcaacd" "{\u0022matches\u0022:[\u0022bcaacd\u0022,null,\u0022b\u0022,\u0022c\u0022,\u0022aac\u0022,\u0022a\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022bcaacd\u0022,\u0022groups\u0022:{\u0022x\u0022:\u0022b\u0022,\u0022y\u0022:\u0022c\u0022,\u0022last\u0022:\u0022aac\u0022,\u0022piece\u0022:\u0022a\u0022},\u0022indices\u0022:[[0,6],null,[0,1],[1,2],[2,5],[2,3]],\u0022indicesGroups\u0022:{\u0022x\u0022:[0,1],\u0022y\u0022:[1,2],\u0022last\u0022:[2,5],\u0022piece\u0022:[2,3]},\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?:(?<x>a)|(?<x>b))(?<y>c)(?<last>(?<piece>a)\u005c\u005cB\u005c\u005ck<piece>\u005c\u005ck<y>)+d\u0022}"
        "(?<x>a)(?<y>b)(?<last>\u005ck<piece>(?<piece>[ab])\u005cB\u005ck<piece>\u005ck<y>)+" flags="dgi" input="abaabaab" "{\u0022matches\u0022:[\u0022abaabaab\u0022,\u0022a\u0022,\u0022b\u0022,\u0022aab\u0022,\u0022a\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022abaabaab\u0022,\u0022groups\u0022:{\u0022x\u0022:\u0022a\u0022,\u0022y\u0022:\u0022b\u0022,\u0022last\u0022:\u0022aab\u0022,\u0022piece\u0022:\u0022a\u0022},\u0022indices\u0022:[[0,8],[0,1],[1,2],[5,8],[5,6]],\u0022indicesGroups\u0022:{\u0022x\u0022:[0,1],\u0022y\u0022:[1,2],\u0022last\u0022:[5,8],\u0022piece\u0022:[5,6]},\u0022lastIndex\u0022:8,\u0022source\u0022:\u0022(?<x>a)(?<y>b)(?<last>\u005c\u005ck<piece>(?<piece>[ab])\u005c\u005cB\u005c\u005ck<piece>\u005c\u005ck<y>)+\u0022}"
        "(?<x>(^\u005ck<x>)+)" flags="dgy" input="q" "{\u0022matches\u0022:[\u0022\u0022,\u0022\u0022,\u0022\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022q\u0022,\u0022groups\u0022:{\u0022x\u0022:\u0022\u0022},\u0022indices\u0022:[[0,0],[0,0],[0,0]],\u0022indicesGroups\u0022:{\u0022x\u0022:[0,0]},\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<x>(^\u005c\u005ck<x>)+)\u0022}"
        "(?<x>)(?<last>(?<point>^)\u005ck<x>){2,3}" flags="dm" input="\u000aq" "{\u0022matches\u0022:[\u0022\u0022,\u0022\u0022,\u0022\u0022,\u0022\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022\u005cnq\u0022,\u0022groups\u0022:{\u0022x\u0022:\u0022\u0022,\u0022last\u0022:\u0022\u0022,\u0022point\u0022:\u0022\u0022},\u0022indices\u0022:[[0,0],[0,0],[0,0],[0,0]],\u0022indicesGroups\u0022:{\u0022x\u0022:[0,0],\u0022last\u0022:[0,0],\u0022point\u0022:[0,0]},\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<x>)(?<last>(?<point>^)\u005c\u005ck<x>){2,3}\u0022}"
        "#);
}

#[test]
fn repeated_assertions_keep_named_positions_and_whole_body_retry_ranges() {
    assert_true(
        r#"
            let a = /(?<x>a)(?<y>b)(?<last>(?<piece>a)\B\k<piece>\k<y>)*aab$/d.exec('abaabaab');
            a[0] === 'abaabaab' && a.groups.last === 'aab' && a.groups.piece === 'a' && a.indices.groups
                .last === a.indices[3] && a.indices.groups.last[0] === 2 && a.indices.groups.piece === a
                .indices[4] && a.indices[4][0] === 2 && a.indices[4][1] === 3
        "#,
    );
    assert_true(
        r#"
            let a = /((?<x>a)(?<y>b)(?<last>(?<piece>a)\B\k<piece>\k<y>)+?)\1/d.exec('abaababaab');
            a[0] === 'abaababaab' && a[1] === 'abaab' && a.groups.piece === 'a' && a.indices.groups
                .piece === a.indices[5] && a.indices[5][0] === 2
        "#,
    );
    assert_true(
        r#"
            let a = /(?<x>a)(?<y>b)(?<last>(?<piece>[ab])\B\k<piece>\k<y>)+$/d.exec('abaabbbb');
            a[0] === 'abaabbbb' && a.groups.last === 'bbb' && a.groups.piece === 'b' && a.indices.groups
                .piece === a.indices[4] && a.indices[4][0] === 5 && a.indices[4][1] === 6
        "#,
    );
    assert_true(
        r#"
            let a = /(.)(a)((.)$\4\2){1,2}b/dms.exec('aa\n\nab');
            a[0] === 'aa\n\nab' && a[3] === '\n\na' && a[4] === '\n' && a.indices[4][0] === 2 &&
                /(.)(a)((.)$\4\2){1,2}b/ds.exec('aa\n\nab') === null
        "#,
    );
    assert_true(
        r#"
            let r = /(?<x>)(?<last>(?<point>^)\k<x>)+/dmy;
            r.lastIndex = 1;
            let a = r.exec('\nq');
            a.index === 1 && a.groups.last === '' && a.indices.groups.point === a.indices[3] && a.indices[
                    3][0] === 1 && r.lastIndex === 1 && /(?<x>)(?<last>(?<point>^)\k<x>)+/dy.exec('\nq')
                .index === 0
        "#,
    );
    assert_true(
        r#"
            let r = /()((\b)\1)+/dy;
            r.lastIndex = 1;
            let a = r.exec('x');
            a.index === 1 && a[2] === '' && a.indices[3][0] === 1 && /()((\b)\1)+/.exec(' ') === null &&
                /()((\B)\1)+/.exec(' ')[2] === ''
        "#,
    );
}

#[test]
fn required_empty_assertions_and_consumers_preserve_zero_progress_rules() {
    assert_true(
        r#"
            let a = [...'q'.matchAll(/(?<x>)(?<last>(?<point>^)\k<x>)*/dg)],
                b = [...'q'.matchAll(/(?<x>)(?<last>(?<point>^)\k<x>)+/dg)];
            a.length === 2 && a[0].groups.last === undefined && a[1].groups.point === undefined && b
                .length === 1 && b[0].groups.point === '' && b[0].indices.groups.point === b[0].indices[
                3] && b[0].indices[3][0] === 0
        "#,
    );
    assert_true(
        r#"
            let r = /(?<x>)(?<last>(?<point>^)\k<x>){2,3}/dy;
            r.lastIndex = 1;
            r.exec('q') === null && r.lastIndex === 0 && 'q'.replace(/(?<x>)(?<last>(?<point>^)\k<x>)+/g,
                '<$<point>>') === '<>q' && 'q'.search(/()((^)\1)+/) === 0
        "#,
    );
    assert_true(
        r#"
            let a = /(?<x>)(?<last>(?<point>^)\k<x>){0,2}/dy,
                b = /(?<x>)(?<last>(?<point>^)\k<x>)+/dy;
            a.lastIndex = 1;
            let x = a.exec('q');
            x.index === 1 && x.groups.last === undefined && x.groups.point === undefined && a.lastIndex ===
                1 && b.exec('q').groups.point === ''
        "#,
    );
    assert_true(
        r#"
            let r = /(?<x>a)(?<y>b)(?<last>(?<piece>[ab])\B\k<piece>\k<y>)+/dg,
                a = [...'abaab abaabbbb'.matchAll(r)];
            a.length === 2 && a[1].groups.last === 'bbb' && a[1].indices.groups.piece[0] === 11 && r
                .lastIndex === 0 && 'qabaabbbbZ'.split(r).join(',') === 'q,a,b,bbb,b,Z'
        "#,
    );
    assert_true(
        r#"
            let r = /(?<x>a)(?<y>b)(?<last>(?<piece>[ab])\B\k<piece>\k<y>)+/dy;
            r.lastIndex = 1;
            let copy = new RegExp(r),
                a = r.exec('qabaabbbb');
            a.groups.piece === 'b' && a.indices.groups.piece[0] === 6 && r.lastIndex === 9 && copy
                .source === r.source && copy.lastIndex === 0
        "#,
    );
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
fn repeated_choices_nested_quantifiers_and_lookaround_match_reference_results() {
    for source in [
        r"/(a)(b)(^\1|\2){2,3}/.test('abab')",
        r"/(?:(?:(a)(b)(a+\1\2)+)|){2,3}/.test('aabaab')",
        r"/(a)(b)(^\1+\2){2,3}/.test('abab')",
        r"/(a)(b)((?=a)\1\2){2,3}/.test('abab')",
        r"/(a)(b)((?i:\1)\2)+/.test('abab')",
        r"/(a)(b)(\b\1\2)+/u.test('aabaab')",
    ] {
        crate::common::regexp_oracle::assert_matches_reference(&mut Realm::default(), source);
    }
    assert_true(r"/(a)(b)(\B\1\2)+/.exec('abc')===null");
}
