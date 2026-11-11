//! References to completed body captures use the current iteration input.

use spite_core::JsString;
use spite_runtime::{Error, Limits, Realm, Value};
use std::fmt::Write;

use crate::common::assert_true;

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
    insta::assert_snapshot!(rows, @r#"
        "(a)((\u005c1)\u005c3)*" flags="d" input="aaaaa" "{\u0022matches\u0022:[\u0022aaaaa\u0022,\u0022a\u0022,\u0022aa\u0022,\u0022a\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022aaaaa\u0022,\u0022indices\u0022:[[0,5],[0,1],[3,5],[3,4]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a)((\u005c\u005c1)\u005c\u005c3)*\u0022}"
        "(a)((\u005c1)\u005c3)*?" flags="d" input="aaaaa" "{\u0022matches\u0022:[\u0022a\u0022,\u0022a\u0022,null,null],\u0022index\u0022:0,\u0022input\u0022:\u0022aaaaa\u0022,\u0022indices\u0022:[[0,1],[0,1],null,null],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a)((\u005c\u005c1)\u005c\u005c3)*?\u0022}"
        "(a)(b)((\u005c1)\u005c4\u005c2)+" flags="d" input="abaabaab" "{\u0022matches\u0022:[\u0022abaabaab\u0022,\u0022a\u0022,\u0022b\u0022,\u0022aab\u0022,\u0022a\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022abaabaab\u0022,\u0022indices\u0022:[[0,8],[0,1],[1,2],[5,8],[5,6]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a)(b)((\u005c\u005c1)\u005c\u005c4\u005c\u005c2)+\u0022}"
        "(a)(b)(\u005c1(\u005c2)\u005c4)+?" flags="d" input="ababb" "{\u0022matches\u0022:[\u0022ababb\u0022,\u0022a\u0022,\u0022b\u0022,\u0022abb\u0022,\u0022b\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022ababb\u0022,\u0022indices\u0022:[[0,5],[0,1],[1,2],[2,5],[3,4]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a)(b)(\u005c\u005c1(\u005c\u005c2)\u005c\u005c4)+?\u0022}"
        "(a)(b)((\u005c1)\u005c4\u005c2)?" flags="d" input="abaab" "{\u0022matches\u0022:[\u0022abaab\u0022,\u0022a\u0022,\u0022b\u0022,\u0022aab\u0022,\u0022a\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022abaab\u0022,\u0022indices\u0022:[[0,5],[0,1],[1,2],[2,5],[2,3]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a)(b)((\u005c\u005c1)\u005c\u005c4\u005c\u005c2)?\u0022}"
        "(a)(b)(\u005c1(\u005c2)\u005c4)??" flags="d" input="ababb" "{\u0022matches\u0022:[\u0022ab\u0022,\u0022a\u0022,\u0022b\u0022,null,null],\u0022index\u0022:0,\u0022input\u0022:\u0022ababb\u0022,\u0022indices\u0022:[[0,2],[0,1],[1,2],null,null],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a)(b)(\u005c\u005c1(\u005c\u005c2)\u005c\u005c4)??\u0022}"
        "(a)(bb)((\u005c2)\u005c4\u005c1){1,3}" flags="d" input="abbbbbbabbbba" "{\u0022matches\u0022:[\u0022abbbbbbabbbba\u0022,\u0022a\u0022,\u0022bb\u0022,\u0022bbbba\u0022,\u0022bb\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022abbbbbbabbbba\u0022,\u0022indices\u0022:[[0,13],[0,1],[1,3],[8,13],[8,10]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a)(bb)((\u005c\u005c2)\u005c\u005c4\u005c\u005c1){1,3}\u0022}"
        "(a)(bb)(\u005c2(\u005c1)\u005c4){1,3}?" flags="d" input="abbbababba" "{\u0022match\u0022:null,\u0022lastIndex\u0022:0}"
        "(a)(b)(()(\u005c1)()\u005c5\u005c2())*ab" flags="d" input="qabaabaab" "{\u0022match\u0022:null,\u0022lastIndex\u0022:0}"
        "(a)(b)(()\u005c1()(\u005c2)\u005c6())*?ab" flags="d" input="qababb" "{\u0022matches\u0022:[\u0022abab\u0022,\u0022a\u0022,\u0022b\u0022,null,null,null,null,null],\u0022index\u0022:1,\u0022input\u0022:\u0022qababb\u0022,\u0022indices\u0022:[[1,5],[1,2],[2,3],null,null,null,null,null],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a)(b)(()\u005c\u005c1()(\u005c\u005c2)\u005c\u005c6())*?ab\u0022}"
        "((a)(b)((\u005c2)\u005c5\u005c3)+)\u005c1" flags="d" input="abaababaab" "{\u0022matches\u0022:[\u0022abaababaab\u0022,\u0022abaab\u0022,\u0022a\u0022,\u0022b\u0022,\u0022aab\u0022,\u0022a\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022abaababaab\u0022,\u0022indices\u0022:[[0,10],[0,5],[0,1],[1,2],[2,5],[2,3]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022((a)(b)((\u005c\u005c2)\u005c\u005c5\u005c\u005c3)+)\u005c\u005c1\u0022}"
        "((a)(b)(\u005c2(\u005c3)\u005c5)+?)\u005c1" flags="d" input="ababbababb" "{\u0022matches\u0022:[\u0022ababbababb\u0022,\u0022ababb\u0022,\u0022a\u0022,\u0022b\u0022,\u0022abb\u0022,\u0022b\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022ababbababb\u0022,\u0022indices\u0022:[[0,10],[0,5],[0,1],[1,2],[2,5],[3,4]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022((a)(b)(\u005c\u005c2(\u005c\u005c3)\u005c\u005c5)+?)\u005c\u005c1\u0022}"
        "(?:(a)|(b))((\u005c1)\u005c4\u005c2)+c" flags="d" input="bbc" "{\u0022matches\u0022:[\u0022bbc\u0022,null,\u0022b\u0022,\u0022b\u0022,\u0022\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022bbc\u0022,\u0022indices\u0022:[[0,3],null,[0,1],[1,2],[1,1]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?:(a)|(b))((\u005c\u005c1)\u005c\u005c4\u005c\u005c2)+c\u0022}"
        "(?:(a)|(b))((\u005c2)\u005c4\u005c1)+c" flags="d" input="bbbc" "{\u0022matches\u0022:[\u0022bbbc\u0022,null,\u0022b\u0022,\u0022bb\u0022,\u0022b\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022bbbc\u0022,\u0022indices\u0022:[[0,4],null,[0,1],[1,3],[1,2]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?:(a)|(b))((\u005c\u005c2)\u005c\u005c4\u005c\u005c1)+c\u0022}"
        "(a)(b)((\u005c1)\u005c4\u005c2)*\u005c4" flags="d" input="abaaba" "{\u0022matches\u0022:[\u0022abaaba\u0022,\u0022a\u0022,\u0022b\u0022,\u0022aab\u0022,\u0022a\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022abaaba\u0022,\u0022indices\u0022:[[0,6],[0,1],[1,2],[2,5],[2,3]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a)(b)((\u005c\u005c1)\u005c\u005c4\u005c\u005c2)*\u005c\u005c4\u0022}"
        "(a)(b)(\u005c1(\u005c2)\u005c4)*?\u005c4" flags="d" input="ababbb" "{\u0022matches\u0022:[\u0022ab\u0022,\u0022a\u0022,\u0022b\u0022,null,null],\u0022index\u0022:0,\u0022input\u0022:\u0022ababbb\u0022,\u0022indices\u0022:[[0,2],[0,1],[1,2],null,null],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a)(b)(\u005c\u005c1(\u005c\u005c2)\u005c\u005c4)*?\u005c\u005c4\u0022}"
        "(a)(b)(\u005c4(\u005c1)\u005c4\u005c2)+" flags="d" input="abaabaab" "{\u0022matches\u0022:[\u0022abaabaab\u0022,\u0022a\u0022,\u0022b\u0022,\u0022aab\u0022,\u0022a\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022abaabaab\u0022,\u0022indices\u0022:[[0,8],[0,1],[1,2],[5,8],[5,6]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a)(b)(\u005c\u005c4(\u005c\u005c1)\u005c\u005c4\u005c\u005c2)+\u0022}"
        "(a)(b)((\u005c4\u005c1)\u005c4\u005c2)+" flags="d" input="abaabaab" "{\u0022matches\u0022:[\u0022abaabaab\u0022,\u0022a\u0022,\u0022b\u0022,\u0022aab\u0022,\u0022a\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022abaabaab\u0022,\u0022indices\u0022:[[0,8],[0,1],[1,2],[5,8],[5,6]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a)(b)((\u005c\u005c4\u005c\u005c1)\u005c\u005c4\u005c\u005c2)+\u0022}"
        "()()((\u005c1)\u005c4\u005c2){2,3}" flags="d" input="q" "{\u0022matches\u0022:[\u0022\u0022,\u0022\u0022,\u0022\u0022,\u0022\u0022,\u0022\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022q\u0022,\u0022indices\u0022:[[0,0],[0,0],[0,0],[0,0],[0,0]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022()()((\u005c\u005c1)\u005c\u005c4\u005c\u005c2){2,3}\u0022}"
        "(?:(?<x>a)|(?<x>b))(?<y>c)(?<last>(?<piece>\u005ck<x>)\u005ck<piece>\u005ck<y>)+d" flags="d" input="bcbbcd" "{\u0022matches\u0022:[\u0022bcbbcd\u0022,null,\u0022b\u0022,\u0022c\u0022,\u0022bbc\u0022,\u0022b\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022bcbbcd\u0022,\u0022groups\u0022:{\u0022x\u0022:\u0022b\u0022,\u0022y\u0022:\u0022c\u0022,\u0022last\u0022:\u0022bbc\u0022,\u0022piece\u0022:\u0022b\u0022},\u0022indices\u0022:[[0,6],null,[0,1],[1,2],[2,5],[2,3]],\u0022indicesGroups\u0022:{\u0022x\u0022:[0,1],\u0022y\u0022:[1,2],\u0022last\u0022:[2,5],\u0022piece\u0022:[2,3]},\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?:(?<x>a)|(?<x>b))(?<y>c)(?<last>(?<piece>\u005c\u005ck<x>)\u005c\u005ck<piece>\u005c\u005ck<y>)+d\u0022}"
        "(?<x>a)(?<y>b)(?<last>\u005ck<piece>(?<piece>\u005ck<x>)\u005ck<piece>\u005ck<y>)+" flags="dgi" input="abaabaab" "{\u0022matches\u0022:[\u0022abaabaab\u0022,\u0022a\u0022,\u0022b\u0022,\u0022aab\u0022,\u0022a\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022abaabaab\u0022,\u0022groups\u0022:{\u0022x\u0022:\u0022a\u0022,\u0022y\u0022:\u0022b\u0022,\u0022last\u0022:\u0022aab\u0022,\u0022piece\u0022:\u0022a\u0022},\u0022indices\u0022:[[0,8],[0,1],[1,2],[5,8],[5,6]],\u0022indicesGroups\u0022:{\u0022x\u0022:[0,1],\u0022y\u0022:[1,2],\u0022last\u0022:[5,8],\u0022piece\u0022:[5,6]},\u0022lastIndex\u0022:8,\u0022source\u0022:\u0022(?<x>a)(?<y>b)(?<last>\u005c\u005ck<piece>(?<piece>\u005c\u005ck<x>)\u005c\u005ck<piece>\u005c\u005ck<y>)+\u0022}"
        "(?<x>a)(?<last>(?<piece>\u005c1)\u005ck<piece>)+" flags="dy" input="aaaaa" "{\u0022matches\u0022:[\u0022aaaaa\u0022,\u0022a\u0022,\u0022aa\u0022,\u0022a\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022aaaaa\u0022,\u0022groups\u0022:{\u0022x\u0022:\u0022a\u0022,\u0022last\u0022:\u0022aa\u0022,\u0022piece\u0022:\u0022a\u0022},\u0022indices\u0022:[[0,5],[0,1],[3,5],[3,4]],\u0022indicesGroups\u0022:{\u0022x\u0022:[0,1],\u0022last\u0022:[3,5],\u0022piece\u0022:[3,4]},\u0022lastIndex\u0022:5,\u0022source\u0022:\u0022(?<x>a)(?<last>(?<piece>\u005c\u005c1)\u005c\u005ck<piece>)+\u0022}"
        "^(?<x>.)(?<y>a)(?<last>(?<piece>\u005ck<x>)\u005ck<piece>\u005ck<y>)+$" flags="dis" input="\u00b5a\u039c\u00b5A\u00b5\u039ca" "{\u0022matches\u0022:[\u0022\u00b5a\u039c\u00b5A\u00b5\u039ca\u0022,\u0022\u00b5\u0022,\u0022a\u0022,\u0022\u00b5\u039ca\u0022,\u0022\u00b5\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022\u00b5a\u039c\u00b5A\u00b5\u039ca\u0022,\u0022groups\u0022:{\u0022x\u0022:\u0022\u00b5\u0022,\u0022y\u0022:\u0022a\u0022,\u0022last\u0022:\u0022\u00b5\u039ca\u0022,\u0022piece\u0022:\u0022\u00b5\u0022},\u0022indices\u0022:[[0,8],[0,1],[1,2],[5,8],[5,6]],\u0022indicesGroups\u0022:{\u0022x\u0022:[0,1],\u0022y\u0022:[1,2],\u0022last\u0022:[5,8],\u0022piece\u0022:[5,6]},\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022^(?<x>.)(?<y>a)(?<last>(?<piece>\u005c\u005ck<x>)\u005c\u005ck<piece>\u005c\u005ck<y>)+$\u0022}"
        "#);
}

#[test]
fn local_named_targets_use_current_iteration_ranges_and_restore_retries() {
    assert_true(
        r"let a=/(?<x>a)(?<y>b)(?<last>(?<piece>\k<x>)\k<piece>\k<y>)*aab$/d.exec('abaabaab');a[0]==='abaabaab'&&a.groups.last==='aab'&&a.groups.piece==='a'&&a.indices.groups.last===a.indices[3]&&a.indices.groups.last[0]===2&&a.indices.groups.piece===a.indices[4]&&a.indices[4][0]===2&&a.indices[4][1]===3",
    );
    assert_true(
        r"let a=/((?<x>a)(?<y>b)(?<last>(?<piece>\k<x>)\k<piece>\k<y>)+?)\1/d.exec('abaababaab');a[0]==='abaababaab'&&a[1]==='abaab'&&a.groups.last==='aab'&&a.groups.piece==='a'&&a.indices.groups.piece===a.indices[5]&&a.indices[5][0]===2",
    );
    assert_true(
        r"let a=/(?<x>a)(?<y>b)(?<last>\k<x>(?<piece>\k<y>\k<x>)\k<piece>)+$/d.exec('abababa');a.groups.last==='ababa'&&a.groups.piece==='ba'&&a.indices.groups.piece===a.indices[4]&&a.indices[4][0]===3&&a.indices[4][1]===5",
    );
    assert_true(
        r"let a=/(?:(?<x>a)|(?<x>b))(?<y>c)(?<last>(?<piece>\k<x>)\k<piece>\k<y>)+/d.exec('bcbbc');a[1]===undefined&&a.groups.last==='bbc'&&a.groups.piece==='b'&&a.indices.groups.piece===a.indices[5]&&a.indices[5][0]===2",
    );
    assert_true(
        r"let a=/(?<x>a)(?<y>b)(?<last>(?<piece>\k<x>)\4\k<piece>\k<y>)+/d.exec('abaaab');a[0]==='abaaab'&&a.groups.last==='aaab'&&a.groups.piece==='a'&&a.indices.groups.piece===a.indices[4]&&a.indices[4][0]===2",
    );
    assert_true(
        r"let a=/(?<x>.)(?<y>a)(?<last>(?<piece>\k<x>)\k<piece>\k<y>)+/d.exec('\uD800a\uD800\uD800a');a[0].length===5&&a.groups.piece.charCodeAt(0)===0xD800&&a.indices.groups.piece===a.indices[4]&&a.indices[4][0]===2&&a.indices[4][1]===3",
    );
}

#[test]
fn local_reference_consumers_and_following_captures_keep_original_numbering() {
    assert_true(
        r"let r=/(?<x>a)(?<y>b)(?<last>(?<piece>\k<x>)\k<piece>\k<y>)+/dg,a=[...'abaab abaabaab'.matchAll(r)];a.length===2&&a[0].groups.last==='aab'&&a[0].indices.groups.piece===a[0].indices[4]&&a[0].indices[4][0]===2&&a[1].indices[4][0]===11&&r.lastIndex===0",
    );
    assert_true(
        r"'qabaabaab'.replace(/(?<x>a)(?<y>b)(?<last>(?<piece>\k<x>)\k<piece>\k<y>)+/,'<$<last>:$<piece>>')==='q<aab:a>'&&'qabaabaab'.search(/(a)(b)((\1)\4\2)+/)===1&&'qabaabaabZ'.split(/(a)(b)((\1)\4\2)+/).join(',')==='q,a,b,aab,a,Z'",
    );
    assert_true(
        r"let r=/(?<x>a)(?<y>b)(?<last>(?<piece>\k<x>)\k<piece>\k<y>)+/dy;r.lastIndex=1;let copy=new RegExp(r),a=r.exec('qabaabaab');a.indices.groups.piece[0]===6&&a.indices.groups.piece[1]===7&&r.lastIndex===9&&r.exec('qabaabaab')===null&&r.lastIndex===0&&copy.source===r.source&&copy.lastIndex===0",
    );
    assert_true(
        r"let a=/(?<x>a)(?<y>b)(?<last>(?<piece>\k<x>)\k<piece>\k<y>)*\k<piece>$/d.exec('abaaba'),b=/(?<x>a)(?<y>b)(?<last>(?<piece>\k<x>)\k<piece>\k<y>)*?\k<piece>$/d.exec('abaaba');a[0]==='abaaba'&&b[0]==='abaaba'&&a.groups.piece==='a'&&b.groups.piece==='a'&&a.indices.groups.piece[0]===2&&b.indices.groups.piece[0]===2",
    );
    assert_true(
        r"let a=[...'q'.matchAll(/(?<x>)(?<y>)(?<last>(?<piece>\k<x>)\k<piece>\k<y>)*/dg)];a.length===2&&a[0].groups.piece===undefined&&a[0].indices.groups.piece===undefined&&a[1].index===1",
    );
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
fn other_consuming_terms_choices_and_nested_repetition_match_reference_results() {
    for source in [
        r"/(?:(?:(?:(a)(b)(a(\1)\4\2)+){2})|){2,3}/.test('aabaab')",
        r"/(a)(b)((\1)|\2){2,3}/.test('abab')",
        r"/(a)(b)((\1)+\4\2){2,3}/.test('aabaab')",
        r"/(a)(b)((?=a)\1\2){2,3}/.test('abab')",
        r"/(a)(b)((?i:\1)\2)+/.test('abab')",
        r"/(a)(b)((\1)\4\2)+/u.test('aabaab')",
    ] {
        crate::common::regexp_oracle::assert_matches_reference(&mut Realm::default(), source);
    }
    assert_true(r"/(a)(b)((\1)\4\2)+/.exec('abc')===null");
}
