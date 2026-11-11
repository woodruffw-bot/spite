//! Capturing reference wrappers preserve final-iteration ranges.

use spite_core::JsString;
use spite_runtime::{Error, Limits, Realm, Value};
use std::fmt::Write;

use crate::common::assert_true;

#[test]
fn captured_quantified_reference_result_snapshot() {
    let mut rows = String::new();
    for (source, flags, text) in [
        (r"(a)(\1)*", "d", "aaaa"),
        (r"(a)(\1)*?", "d", "aaaa"),
        (r"(a)(\1)+", "d", "a"),
        (r"(a)(\1)+?", "d", "aaaa"),
        (r"(a)(\1)?", "d", "aa"),
        (r"(a)(\1)??", "d", "aa"),
        (r"(ab)((?:\1)){2,3}", "d", "abababab"),
        (r"(ab)((?:\1)){1,3}?", "d", "abababab"),
        (r"(a)((?:)\1(?:))*ab", "d", "qaaaab"),
        (r"(a)((?:)\1(?:))*?ab", "d", "qaaaab"),
        (r"((a)(\2)+)\1", "d", "aaaaaa"),
        (r"((a)((?:\2))+?)\1", "d", "aaaaaa"),
        (r"(a|ab)(\1)+b", "d", "ababb"),
        (r"(?:(a)|(ab))(\1)*(\2)+", "d", "abab"),
        (r"(a)(b)((\1))*((?:\2))*", "d", "abaaabbb"),
        (r"(a)(b)((\1))*?((?:\2))*?", "d", "abaaabbb"),
        (r"(\2)+(a)", "d", "a"),
        (r"(\1)+a\1", "d", "a"),
        (r"()(\1){2,3}", "d", "q"),
        (r"(?:(?<x>a)|(?<x>ab))(?<last>\k<x>)+b", "d", "ababb"),
        (r"(?<x>\k<x>)+", "dgi", "q"),
        (r"(?<x>a)(?<last>\k<x>)+\k<last>", "dy", "aaaa"),
        (r"^(?<x>.)(?<last>(?:)\k<x>(?:))+$", "dis", "µΜµ"),
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
        "(a)(\u005c1)*" flags="d" input="aaaa" "{\u0022matches\u0022:[\u0022aaaa\u0022,\u0022a\u0022,\u0022a\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022aaaa\u0022,\u0022indices\u0022:[[0,4],[0,1],[3,4]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a)(\u005c\u005c1)*\u0022}"
        "(a)(\u005c1)*?" flags="d" input="aaaa" "{\u0022matches\u0022:[\u0022a\u0022,\u0022a\u0022,null],\u0022index\u0022:0,\u0022input\u0022:\u0022aaaa\u0022,\u0022indices\u0022:[[0,1],[0,1],null],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a)(\u005c\u005c1)*?\u0022}"
        "(a)(\u005c1)+" flags="d" input="a" "{\u0022match\u0022:null,\u0022lastIndex\u0022:0}"
        "(a)(\u005c1)+?" flags="d" input="aaaa" "{\u0022matches\u0022:[\u0022aa\u0022,\u0022a\u0022,\u0022a\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022aaaa\u0022,\u0022indices\u0022:[[0,2],[0,1],[1,2]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a)(\u005c\u005c1)+?\u0022}"
        "(a)(\u005c1)?" flags="d" input="aa" "{\u0022matches\u0022:[\u0022aa\u0022,\u0022a\u0022,\u0022a\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022aa\u0022,\u0022indices\u0022:[[0,2],[0,1],[1,2]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a)(\u005c\u005c1)?\u0022}"
        "(a)(\u005c1)??" flags="d" input="aa" "{\u0022matches\u0022:[\u0022a\u0022,\u0022a\u0022,null],\u0022index\u0022:0,\u0022input\u0022:\u0022aa\u0022,\u0022indices\u0022:[[0,1],[0,1],null],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a)(\u005c\u005c1)??\u0022}"
        "(ab)((?:\u005c1)){2,3}" flags="d" input="abababab" "{\u0022matches\u0022:[\u0022abababab\u0022,\u0022ab\u0022,\u0022ab\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022abababab\u0022,\u0022indices\u0022:[[0,8],[0,2],[6,8]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(ab)((?:\u005c\u005c1)){2,3}\u0022}"
        "(ab)((?:\u005c1)){1,3}?" flags="d" input="abababab" "{\u0022matches\u0022:[\u0022abab\u0022,\u0022ab\u0022,\u0022ab\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022abababab\u0022,\u0022indices\u0022:[[0,4],[0,2],[2,4]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(ab)((?:\u005c\u005c1)){1,3}?\u0022}"
        "(a)((?:)\u005c1(?:))*ab" flags="d" input="qaaaab" "{\u0022matches\u0022:[\u0022aaaab\u0022,\u0022a\u0022,\u0022a\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022qaaaab\u0022,\u0022indices\u0022:[[1,6],[1,2],[3,4]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a)((?:)\u005c\u005c1(?:))*ab\u0022}"
        "(a)((?:)\u005c1(?:))*?ab" flags="d" input="qaaaab" "{\u0022matches\u0022:[\u0022aaaab\u0022,\u0022a\u0022,\u0022a\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022qaaaab\u0022,\u0022indices\u0022:[[1,6],[1,2],[3,4]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a)((?:)\u005c\u005c1(?:))*?ab\u0022}"
        "((a)(\u005c2)+)\u005c1" flags="d" input="aaaaaa" "{\u0022matches\u0022:[\u0022aaaaaa\u0022,\u0022aaa\u0022,\u0022a\u0022,\u0022a\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022aaaaaa\u0022,\u0022indices\u0022:[[0,6],[0,3],[0,1],[2,3]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022((a)(\u005c\u005c2)+)\u005c\u005c1\u0022}"
        "((a)((?:\u005c2))+?)\u005c1" flags="d" input="aaaaaa" "{\u0022matches\u0022:[\u0022aaaa\u0022,\u0022aa\u0022,\u0022a\u0022,\u0022a\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022aaaaaa\u0022,\u0022indices\u0022:[[0,4],[0,2],[0,1],[1,2]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022((a)((?:\u005c\u005c2))+?)\u005c\u005c1\u0022}"
        "(a|ab)(\u005c1)+b" flags="d" input="ababb" "{\u0022matches\u0022:[\u0022ababb\u0022,\u0022ab\u0022,\u0022ab\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022ababb\u0022,\u0022indices\u0022:[[0,5],[0,2],[2,4]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a|ab)(\u005c\u005c1)+b\u0022}"
        "(?:(a)|(ab))(\u005c1)*(\u005c2)+" flags="d" input="abab" "{\u0022matches\u0022:[\u0022a\u0022,\u0022a\u0022,null,null,\u0022\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022abab\u0022,\u0022indices\u0022:[[0,1],[0,1],null,null,[1,1]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?:(a)|(ab))(\u005c\u005c1)*(\u005c\u005c2)+\u0022}"
        "(a)(b)((\u005c1))*((?:\u005c2))*" flags="d" input="abaaabbb" "{\u0022matches\u0022:[\u0022abaaabbb\u0022,\u0022a\u0022,\u0022b\u0022,\u0022a\u0022,\u0022a\u0022,\u0022b\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022abaaabbb\u0022,\u0022indices\u0022:[[0,8],[0,1],[1,2],[4,5],[4,5],[7,8]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a)(b)((\u005c\u005c1))*((?:\u005c\u005c2))*\u0022}"
        "(a)(b)((\u005c1))*?((?:\u005c2))*?" flags="d" input="abaaabbb" "{\u0022matches\u0022:[\u0022ab\u0022,\u0022a\u0022,\u0022b\u0022,null,null,null],\u0022index\u0022:0,\u0022input\u0022:\u0022abaaabbb\u0022,\u0022indices\u0022:[[0,2],[0,1],[1,2],null,null,null],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a)(b)((\u005c\u005c1))*?((?:\u005c\u005c2))*?\u0022}"
        "(\u005c2)+(a)" flags="d" input="a" "{\u0022matches\u0022:[\u0022a\u0022,\u0022\u0022,\u0022a\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022a\u0022,\u0022indices\u0022:[[0,1],[0,0],[0,1]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(\u005c\u005c2)+(a)\u0022}"
        "(\u005c1)+a\u005c1" flags="d" input="a" "{\u0022matches\u0022:[\u0022a\u0022,\u0022\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022a\u0022,\u0022indices\u0022:[[0,1],[0,0]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(\u005c\u005c1)+a\u005c\u005c1\u0022}"
        "()(\u005c1){2,3}" flags="d" input="q" "{\u0022matches\u0022:[\u0022\u0022,\u0022\u0022,\u0022\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022q\u0022,\u0022indices\u0022:[[0,0],[0,0],[0,0]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022()(\u005c\u005c1){2,3}\u0022}"
        "(?:(?<x>a)|(?<x>ab))(?<last>\u005ck<x>)+b" flags="d" input="ababb" "{\u0022matches\u0022:[\u0022ababb\u0022,null,\u0022ab\u0022,\u0022ab\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022ababb\u0022,\u0022groups\u0022:{\u0022x\u0022:\u0022ab\u0022,\u0022last\u0022:\u0022ab\u0022},\u0022indices\u0022:[[0,5],null,[0,2],[2,4]],\u0022indicesGroups\u0022:{\u0022x\u0022:[0,2],\u0022last\u0022:[2,4]},\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?:(?<x>a)|(?<x>ab))(?<last>\u005c\u005ck<x>)+b\u0022}"
        "(?<x>\u005ck<x>)+" flags="dgi" input="q" "{\u0022matches\u0022:[\u0022\u0022,\u0022\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022q\u0022,\u0022groups\u0022:{\u0022x\u0022:\u0022\u0022},\u0022indices\u0022:[[0,0],[0,0]],\u0022indicesGroups\u0022:{\u0022x\u0022:[0,0]},\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<x>\u005c\u005ck<x>)+\u0022}"
        "(?<x>a)(?<last>\u005ck<x>)+\u005ck<last>" flags="dy" input="aaaa" "{\u0022matches\u0022:[\u0022aaaa\u0022,\u0022a\u0022,\u0022a\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022aaaa\u0022,\u0022groups\u0022:{\u0022x\u0022:\u0022a\u0022,\u0022last\u0022:\u0022a\u0022},\u0022indices\u0022:[[0,4],[0,1],[2,3]],\u0022indicesGroups\u0022:{\u0022x\u0022:[0,1],\u0022last\u0022:[2,3]},\u0022lastIndex\u0022:4,\u0022source\u0022:\u0022(?<x>a)(?<last>\u005c\u005ck<x>)+\u005c\u005ck<last>\u0022}"
        "^(?<x>.)(?<last>(?:)\u005ck<x>(?:))+$" flags="dis" input="\u00b5\u039c\u00b5" "{\u0022matches\u0022:[\u0022\u00b5\u039c\u00b5\u0022,\u0022\u00b5\u0022,\u0022\u00b5\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022\u00b5\u039c\u00b5\u0022,\u0022groups\u0022:{\u0022x\u0022:\u0022\u00b5\u0022,\u0022last\u0022:\u0022\u00b5\u0022},\u0022indices\u0022:[[0,3],[0,1],[2,3]],\u0022indicesGroups\u0022:{\u0022x\u0022:[0,1],\u0022last\u0022:[2,3]},\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022^(?<x>.)(?<last>(?:)\u005c\u005ck<x>(?:))+$\u0022}"
        "#);
}

#[test]
fn captured_reference_retries_keep_final_ranges_and_name_aliases() {
    assert_true(
        r"let a=/(?<x>a)(?<last>\k<x>)+\k<last>/d.exec('aaaa');a[0]==='aaaa'&&a.groups.last==='a'&&a.indices.groups.last===a.indices[2]&&a.indices[2][0]===2&&a.indices[2][1]===3",
    );
    assert_true(
        r"let a=/((?<x>a)(?<last>\k<x>)+?)\1/d.exec('aaaaaa');a[0]==='aaaa'&&a[1]==='aa'&&a.groups.last==='a'&&a.indices.groups.last===a.indices[3]&&a.indices[3][0]===1&&a.indices[3][1]===2",
    );
    assert_true(
        r"let a=/(?:(?<x>a)|(?<x>ab))(?<last>\k<x>)+b/d.exec('qabababb');a.index===1&&a[0]==='abababb'&&a[1]===undefined&&a.groups.x==='ab'&&a.groups.last==='ab'&&a.indices.groups.x===a.indices[2]&&a.indices.groups.last===a.indices[3]&&a.indices[3][0]===5&&a.indices[3][1]===7",
    );
    assert_true(
        r"let a=/(?<x>a)(?<last>(\k<x>))*/d.exec('aaaa');a.groups.last==='a'&&a[3]==='a'&&a.indices[2][0]===3&&a.indices[3][0]===3&&a.indices.groups.last===a.indices[2]",
    );
    assert_true(
        r"let a=/(?<x>.)(?<last>\k<x>)+/d.exec('\uD800\uD800\uD800');a[0].length===3&&a.groups.last.charCodeAt(0)===0xD800&&a.indices.groups.last===a.indices[2]&&a.indices[2][0]===2&&a.indices[2][1]===3",
    );
}

#[test]
fn captured_consumers_and_zero_iterations_keep_sources_and_aliases() {
    assert_true(
        r"let r=/(?<x>a)(?<last>\k<x>)+/dg,a=[...'aaa aa'.matchAll(r)];a.length===2&&a[0][0]==='aaa'&&a[1][0]==='aa'&&a[0].groups.last==='a'&&a[0].indices.groups.last===a[0].indices[2]&&a[0].indices[2][0]===2&&r.lastIndex===0",
    );
    assert_true(
        r"'qaaaa'.replace(/(?<x>a)(?<last>\k<x>)+/,'<$<last>>')==='q<a>'&&'qaaaa'.search(/(a)(\1)+/)===1&&'qaaaaZ'.split(/(a)(\1)+/).join(',')==='q,a,a,Z'",
    );
    assert_true(
        r"let r=/(?<x>a)(?<last>\k<x>)+/dy;r.lastIndex=1;let copy=new RegExp(r),a=r.exec('qaaa');a[0]==='aaa'&&a.indices[2][0]===3&&r.lastIndex===4&&r.exec('qaaa')===null&&r.lastIndex===0&&copy.source===r.source&&copy.lastIndex===0",
    );
    assert_true(
        r"let a=[...'q'.matchAll(/(?<x>)(?<last>\k<x>)*/dg)],b=/(?<self>\k<self>)+/d.exec('q');a.length===2&&a[0].groups.last===undefined&&a[0].indices.groups.last===undefined&&a[1].index===1&&b.groups.self===''&&b.indices.groups.self===b.indices[1]",
    );
}

#[test]
fn explicit_captured_reference_work_aborts_before_last_index_and_handlers() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval("let marker=0,r=/(?<x>a)(?<last>\\k<x>)*b/g;r.lastIndex=1;let text='a'.repeat(10000)")
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
fn composite_reference_repetitions_match_reference_results_and_plain_failures_return_null() {
    for source in [
        r"/(?:(?:(?:(a)(\1\1)+){2})|){2,3}/.test('aa')",
        r"/(?:(?:(?:(a)(a\1)+){2})|){2,3}/.test('aa')",
        r"/(a)(\1|b){2,3}/.test('aa')",
        r"/(a)(\1*){2,3}/.test('aa')",
        r"/(a)(\1(?=a)){2,3}/.test('aa')",
        r"/(a)(\1)+/u.test('aa')",
    ] {
        crate::common::regexp_oracle::assert_matches_reference(&mut Realm::default(), source);
    }
    assert_true(r"/(a)(\1)+/.exec('ab')===null");
}
