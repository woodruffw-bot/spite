//! Capture-free reference sequences retain fixed external targets.

use spite_core::JsString;
use spite_runtime::{Error, Limits, Realm, Value};
use std::fmt::Write;

use crate::common::assert_true;

#[test]
fn reference_sequence_body_result_snapshot() {
    let mut rows = String::new();
    for (source, flags, text) in [
        (r"(a)(b)(?:\1\2)*", "d", "ababab"),
        (r"(a)(b)(?:\1\2)*?", "d", "ababab"),
        (r"(a)(b)(?:\1\2)+", "d", "ab"),
        (r"(a)(b)(?:\1\2)+?", "d", "ababab"),
        (r"(a)(b)(?:\1\2)?", "d", "abab"),
        (r"(a)(b)(?:\1\2)??", "d", "abab"),
        (r"(a)(bb)(?:\2\1){1,3}", "d", "abbabbabba"),
        (r"(a)(bb)(?:\2\1){1,3}?", "d", "abbabbabba"),
        (r"(a)(b)(?:\1\2)*ab", "d", "qababab"),
        (r"(a)(b)(?:\1\2)*?ab", "d", "qababab"),
        (r"((a)(b)(?:\2\3)+)\1", "d", "abababab"),
        (r"((a)(b)(?:\2\3)+?)\1", "d", "abababab"),
        (r"(?:(a)|(b))(?:\1\2)+c", "d", "bbc"),
        (r"(?:(a)|(b))(?:\2\1)+c", "d", "bbc"),
        (r"(a)(b)(?:\1\2)*(?:\2\1)*", "d", "ababbaba"),
        (r"(a)(b)(?:\1\2)*?(?:\2\1)*?", "d", "ababbaba"),
        (r"(?:\1\2)+(a)(b)", "d", "ab"),
        (r"((?:\1\2)+a)(b)", "d", "ab"),
        (r"()()(?:\1\2){2,3}", "d", "q"),
        (r"(?:(?<x>a)|(?<x>b))(?<y>c)(?:\k<x>\k<y>)+d", "d", "bcbcd"),
        (r"(?<x>(?:\k<x>\k<y>)+a)(?<y>b)", "dgi", "ab"),
        (r"(?<x>a)(?:\1\k<x>)+", "dy", "aaaaa"),
        (r"^(?<x>.)(?<y>a)(?:\k<x>\k<y>)+$", "dis", "µaΜAµa"),
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
        "(a)(b)(?:\u005c1\u005c2)*" flags="d" input="ababab" "{\u0022matches\u0022:[\u0022ababab\u0022,\u0022a\u0022,\u0022b\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022ababab\u0022,\u0022indices\u0022:[[0,6],[0,1],[1,2]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a)(b)(?:\u005c\u005c1\u005c\u005c2)*\u0022}"
        "(a)(b)(?:\u005c1\u005c2)*?" flags="d" input="ababab" "{\u0022matches\u0022:[\u0022ab\u0022,\u0022a\u0022,\u0022b\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022ababab\u0022,\u0022indices\u0022:[[0,2],[0,1],[1,2]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a)(b)(?:\u005c\u005c1\u005c\u005c2)*?\u0022}"
        "(a)(b)(?:\u005c1\u005c2)+" flags="d" input="ab" "{\u0022match\u0022:null,\u0022lastIndex\u0022:0}"
        "(a)(b)(?:\u005c1\u005c2)+?" flags="d" input="ababab" "{\u0022matches\u0022:[\u0022abab\u0022,\u0022a\u0022,\u0022b\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022ababab\u0022,\u0022indices\u0022:[[0,4],[0,1],[1,2]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a)(b)(?:\u005c\u005c1\u005c\u005c2)+?\u0022}"
        "(a)(b)(?:\u005c1\u005c2)?" flags="d" input="abab" "{\u0022matches\u0022:[\u0022abab\u0022,\u0022a\u0022,\u0022b\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022abab\u0022,\u0022indices\u0022:[[0,4],[0,1],[1,2]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a)(b)(?:\u005c\u005c1\u005c\u005c2)?\u0022}"
        "(a)(b)(?:\u005c1\u005c2)??" flags="d" input="abab" "{\u0022matches\u0022:[\u0022ab\u0022,\u0022a\u0022,\u0022b\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022abab\u0022,\u0022indices\u0022:[[0,2],[0,1],[1,2]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a)(b)(?:\u005c\u005c1\u005c\u005c2)??\u0022}"
        "(a)(bb)(?:\u005c2\u005c1){1,3}" flags="d" input="abbabbabba" "{\u0022match\u0022:null,\u0022lastIndex\u0022:0}"
        "(a)(bb)(?:\u005c2\u005c1){1,3}?" flags="d" input="abbabbabba" "{\u0022match\u0022:null,\u0022lastIndex\u0022:0}"
        "(a)(b)(?:\u005c1\u005c2)*ab" flags="d" input="qababab" "{\u0022matches\u0022:[\u0022ababab\u0022,\u0022a\u0022,\u0022b\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022qababab\u0022,\u0022indices\u0022:[[1,7],[1,2],[2,3]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a)(b)(?:\u005c\u005c1\u005c\u005c2)*ab\u0022}"
        "(a)(b)(?:\u005c1\u005c2)*?ab" flags="d" input="qababab" "{\u0022matches\u0022:[\u0022abab\u0022,\u0022a\u0022,\u0022b\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022qababab\u0022,\u0022indices\u0022:[[1,5],[1,2],[2,3]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a)(b)(?:\u005c\u005c1\u005c\u005c2)*?ab\u0022}"
        "((a)(b)(?:\u005c2\u005c3)+)\u005c1" flags="d" input="abababab" "{\u0022matches\u0022:[\u0022abababab\u0022,\u0022abab\u0022,\u0022a\u0022,\u0022b\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022abababab\u0022,\u0022indices\u0022:[[0,8],[0,4],[0,1],[1,2]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022((a)(b)(?:\u005c\u005c2\u005c\u005c3)+)\u005c\u005c1\u0022}"
        "((a)(b)(?:\u005c2\u005c3)+?)\u005c1" flags="d" input="abababab" "{\u0022matches\u0022:[\u0022abababab\u0022,\u0022abab\u0022,\u0022a\u0022,\u0022b\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022abababab\u0022,\u0022indices\u0022:[[0,8],[0,4],[0,1],[1,2]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022((a)(b)(?:\u005c\u005c2\u005c\u005c3)+?)\u005c\u005c1\u0022}"
        "(?:(a)|(b))(?:\u005c1\u005c2)+c" flags="d" input="bbc" "{\u0022matches\u0022:[\u0022bbc\u0022,null,\u0022b\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022bbc\u0022,\u0022indices\u0022:[[0,3],null,[0,1]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?:(a)|(b))(?:\u005c\u005c1\u005c\u005c2)+c\u0022}"
        "(?:(a)|(b))(?:\u005c2\u005c1)+c" flags="d" input="bbc" "{\u0022matches\u0022:[\u0022bbc\u0022,null,\u0022b\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022bbc\u0022,\u0022indices\u0022:[[0,3],null,[0,1]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?:(a)|(b))(?:\u005c\u005c2\u005c\u005c1)+c\u0022}"
        "(a)(b)(?:\u005c1\u005c2)*(?:\u005c2\u005c1)*" flags="d" input="ababbaba" "{\u0022matches\u0022:[\u0022ababbaba\u0022,\u0022a\u0022,\u0022b\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022ababbaba\u0022,\u0022indices\u0022:[[0,8],[0,1],[1,2]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a)(b)(?:\u005c\u005c1\u005c\u005c2)*(?:\u005c\u005c2\u005c\u005c1)*\u0022}"
        "(a)(b)(?:\u005c1\u005c2)*?(?:\u005c2\u005c1)*?" flags="d" input="ababbaba" "{\u0022matches\u0022:[\u0022ab\u0022,\u0022a\u0022,\u0022b\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022ababbaba\u0022,\u0022indices\u0022:[[0,2],[0,1],[1,2]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a)(b)(?:\u005c\u005c1\u005c\u005c2)*?(?:\u005c\u005c2\u005c\u005c1)*?\u0022}"
        "(?:\u005c1\u005c2)+(a)(b)" flags="d" input="ab" "{\u0022matches\u0022:[\u0022ab\u0022,\u0022a\u0022,\u0022b\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022ab\u0022,\u0022indices\u0022:[[0,2],[0,1],[1,2]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?:\u005c\u005c1\u005c\u005c2)+(a)(b)\u0022}"
        "((?:\u005c1\u005c2)+a)(b)" flags="d" input="ab" "{\u0022matches\u0022:[\u0022ab\u0022,\u0022a\u0022,\u0022b\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022ab\u0022,\u0022indices\u0022:[[0,2],[0,1],[1,2]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022((?:\u005c\u005c1\u005c\u005c2)+a)(b)\u0022}"
        "()()(?:\u005c1\u005c2){2,3}" flags="d" input="q" "{\u0022matches\u0022:[\u0022\u0022,\u0022\u0022,\u0022\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022q\u0022,\u0022indices\u0022:[[0,0],[0,0],[0,0]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022()()(?:\u005c\u005c1\u005c\u005c2){2,3}\u0022}"
        "(?:(?<x>a)|(?<x>b))(?<y>c)(?:\u005ck<x>\u005ck<y>)+d" flags="d" input="bcbcd" "{\u0022matches\u0022:[\u0022bcbcd\u0022,null,\u0022b\u0022,\u0022c\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022bcbcd\u0022,\u0022groups\u0022:{\u0022x\u0022:\u0022b\u0022,\u0022y\u0022:\u0022c\u0022},\u0022indices\u0022:[[0,5],null,[0,1],[1,2]],\u0022indicesGroups\u0022:{\u0022x\u0022:[0,1],\u0022y\u0022:[1,2]},\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?:(?<x>a)|(?<x>b))(?<y>c)(?:\u005c\u005ck<x>\u005c\u005ck<y>)+d\u0022}"
        "(?<x>(?:\u005ck<x>\u005ck<y>)+a)(?<y>b)" flags="dgi" input="ab" "{\u0022matches\u0022:[\u0022ab\u0022,\u0022a\u0022,\u0022b\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022ab\u0022,\u0022groups\u0022:{\u0022x\u0022:\u0022a\u0022,\u0022y\u0022:\u0022b\u0022},\u0022indices\u0022:[[0,2],[0,1],[1,2]],\u0022indicesGroups\u0022:{\u0022x\u0022:[0,1],\u0022y\u0022:[1,2]},\u0022lastIndex\u0022:2,\u0022source\u0022:\u0022(?<x>(?:\u005c\u005ck<x>\u005c\u005ck<y>)+a)(?<y>b)\u0022}"
        "(?<x>a)(?:\u005c1\u005ck<x>)+" flags="dy" input="aaaaa" "{\u0022matches\u0022:[\u0022aaaaa\u0022,\u0022a\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022aaaaa\u0022,\u0022groups\u0022:{\u0022x\u0022:\u0022a\u0022},\u0022indices\u0022:[[0,5],[0,1]],\u0022indicesGroups\u0022:{\u0022x\u0022:[0,1]},\u0022lastIndex\u0022:5,\u0022source\u0022:\u0022(?<x>a)(?:\u005c\u005c1\u005c\u005ck<x>)+\u0022}"
        "^(?<x>.)(?<y>a)(?:\u005ck<x>\u005ck<y>)+$" flags="dis" input="\u00b5a\u039cA\u00b5a" "{\u0022matches\u0022:[\u0022\u00b5a\u039cA\u00b5a\u0022,\u0022\u00b5\u0022,\u0022a\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022\u00b5a\u039cA\u00b5a\u0022,\u0022groups\u0022:{\u0022x\u0022:\u0022\u00b5\u0022,\u0022y\u0022:\u0022a\u0022},\u0022indices\u0022:[[0,6],[0,1],[1,2]],\u0022indicesGroups\u0022:{\u0022x\u0022:[0,1],\u0022y\u0022:[1,2]},\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022^(?<x>.)(?<y>a)(?:\u005c\u005ck<x>\u005c\u005ck<y>)+$\u0022}"
        "#);
}

#[test]
fn reference_sequence_retries_keep_prefix_ranges_and_original_aliases() {
    assert_true(
        r"let a=/(?<x>a)(?<y>b)(?:\k<x>\k<y>)*ab$/d.exec('ababab');a[0]==='ababab'&&a.groups.x==='a'&&a.groups.y==='b'&&a.indices.groups.x===a.indices[1]&&a.indices.groups.y===a.indices[2]&&a.indices[1][0]===0&&a.indices[2][0]===1",
    );
    assert_true(
        r"let a=/((?<x>a)(?<y>b)(?:\k<x>\k<y>)+?)\1/d.exec('abababab');a[0]==='abababab'&&a[1]==='abab'&&a.indices[1][1]===4&&a.indices.groups.x===a.indices[2]&&a.indices.groups.y===a.indices[3]",
    );
    assert_true(
        r"let a=/(?:(?<x>a)|(?<x>b))(?<y>c)(?:\k<x>\k<y>)+d/d.exec('bcbcd');a[0]==='bcbcd'&&a[1]===undefined&&a.groups.x==='b'&&a.groups.y==='c'&&a.indices.groups.x===a.indices[2]&&a.indices.groups.y===a.indices[3]",
    );
    assert_true(
        r"let a=/(?<x>a)(?:\1\k<x>)+/d.exec('aaaaa');a[0]==='aaaaa'&&a.groups.x==='a'&&a.indices.groups.x===a.indices[1]&&/^(?<x>a)(?:\1\k<x>)+$/.exec('aaaa')===null",
    );
    assert_true(
        r"let a=/(?<x>.)(?<y>a)(?:\k<x>\k<y>)+/d.exec('\uD800a\uD800a');a[0].length===4&&a.groups.x.charCodeAt(0)===0xD800&&a.indices.groups.x===a.indices[1]&&a.indices.groups.y===a.indices[2]&&a.indices[2][0]===1",
    );
}

#[test]
fn reference_sequence_consumers_and_empty_targets_keep_original_sources() {
    assert_true(
        r"let r=/(?<x>a)(?<y>b)(?:\k<x>\k<y>)+/dg,a=[...'abab ababab'.matchAll(r)];a.length===2&&a[0][0]==='abab'&&a[1][0]==='ababab'&&a[0].groups.x==='a'&&a[0].indices.groups.x===a[0].indices[1]&&a[1].indices.groups.y===a[1].indices[2]&&r.lastIndex===0",
    );
    assert_true(
        r"'qababab'.replace(/(?<x>a)(?<y>b)(?:\k<x>\k<y>)+/,'<$<x>:$<y>>')==='q<a:b>'&&'qababab'.search(/(a)(b)(?:\1\2)+/)===1&&'qabababZ'.split(/(a)(b)(?:\1\2)+/).join(',')==='q,a,b,Z'",
    );
    assert_true(
        r"let r=/(?<x>a)(?<y>b)(?:\k<x>\k<y>)+/dy;r.lastIndex=1;let copy=new RegExp(r),a=r.exec('qababab');a[0]==='ababab'&&r.lastIndex===7&&r.exec('qababab')===null&&r.lastIndex===0&&copy.source===r.source&&copy.lastIndex===0",
    );
    assert_true(
        r"let a=[...'q'.matchAll(/(?<x>)(?<y>)(?:\k<x>\k<y>)+/dg)],b=/(?<x>(?:\k<x>\k<y>)+a)(?<y>b)/d.exec('ab');a.length===2&&a[0].groups.x===''&&a[0].groups.y===''&&a[1].index===1&&b.groups.x==='a'&&b.groups.y==='b'&&b.indices.groups.x===b.indices[1]",
    );
}

#[test]
fn explicit_sequence_work_aborts_before_last_index_and_handlers() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm.eval(r"let marker=0,r=/(?<x>a)(?<y>b)(?:\k<x>\k<y>)*c/g;r.lastIndex=1;let text='ab'.repeat(5000)").unwrap();
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
fn capture_effects_and_other_sequence_bodies_match_reference_results() {
    for source in [
        r"/(?:(?:(?:(a)(b)(\1\2)+){2})|){2,3}/.test('abab')",
        r"/(?:(?:(?:(a)(b)(?:()\1\2)+){2})|){2,3}/.test('abab')",
        r"/(?:(?:(?:(a)(b)(?:a\1\2)+){2})|){2,3}/.test('abab')",
        r"/(a)(b)(?:\1|\2){2,3}/.test('abab')",
        r"/(a)(b)(?:\1+\2){2,3}/.test('abab')",
        r"/(a)(b)(?:\1\2)+/u.test('abab')",
    ] {
        crate::common::regexp_oracle::assert_matches_reference(&mut Realm::default(), source);
    }
    assert_true(r"/(a)(b)(?:\1\2)+/.exec('abc')===null");
}
