//! Complete ordinary capture layouts after specialized matcher preparation.

use spite_core::JsString;
use spite_runtime::{Error, Limits, Realm, Value};
use std::fmt::Write;

use crate::common::assert_true;

#[test]
fn ordinary_fallback_result_snapshot() {
    let mut rows = String::new();
    for (source, flags, text) in [
        (r"((1)|(12))((3)|(23))", "d", "123"),
        (r"(a|ab)b", "d", "abb"),
        (r"(ab|a)b", "d", "abb"),
        (r"((a|ab)(b|))", "d", "ab"),
        (r"(a|)(b|a)", "d", "aba"),
        (r"(?:ab|cd)\d?", "dg", "cd2"),
        (r"(?:(a)|(b))(c|)", "d", "bc"),
        (r"(a+)(b+)", "d", "aaabbbb"),
        (r"(a+?)(b+?)", "d", "aaabbbb"),
        (r"([ab]*)c", "d", "aacc"),
        (r"([ab]*?)c", "d", "aacc"),
        (r"((a+)(b*))", "d", "aaabb"),
        (r"(?:(a*)b(c*))", "d", "aabcc"),
        (r"^(a|ab)b$", "dm", "q\nabb\nz"),
        (r"\b(a+)\b", "d", " a "),
        (r"(.+)(.)", "ds", "\n\n"),
        (r"(.+?)(.)", "ds", "\n\n"),
        (r"(a?)", "d", "q"),
        (r"()?", "d", "q"),
        (r"()+", "d", "q"),
        (r"(?<x>a+)(?<y>b+)", "d", "aaabbbb"),
        (r"(?<all>(?<x>a+)(?<y>b*))", "d", "aaabb"),
        (r"(?:(?<x>a+)|(?<x>b+))(c|)", "d", "bbbc"),
        (r"(?<x>[ab]*)c", "dgy", "q"),
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
        "((1)|(12))((3)|(23))" flags="d" input="123" "{\u0022matches\u0022:[\u0022123\u0022,\u00221\u0022,\u00221\u0022,null,\u002223\u0022,null,\u002223\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022123\u0022,\u0022indices\u0022:[[0,3],[0,1],[0,1],null,[1,3],null,[1,3]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022((1)|(12))((3)|(23))\u0022}"
        "(a|ab)b" flags="d" input="abb" "{\u0022matches\u0022:[\u0022ab\u0022,\u0022a\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022abb\u0022,\u0022indices\u0022:[[0,2],[0,1]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a|ab)b\u0022}"
        "(ab|a)b" flags="d" input="abb" "{\u0022matches\u0022:[\u0022abb\u0022,\u0022ab\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022abb\u0022,\u0022indices\u0022:[[0,3],[0,2]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(ab|a)b\u0022}"
        "((a|ab)(b|))" flags="d" input="ab" "{\u0022matches\u0022:[\u0022ab\u0022,\u0022ab\u0022,\u0022a\u0022,\u0022b\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022ab\u0022,\u0022indices\u0022:[[0,2],[0,2],[0,1],[1,2]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022((a|ab)(b|))\u0022}"
        "(a|)(b|a)" flags="d" input="aba" "{\u0022matches\u0022:[\u0022ab\u0022,\u0022a\u0022,\u0022b\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022aba\u0022,\u0022indices\u0022:[[0,2],[0,1],[1,2]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a|)(b|a)\u0022}"
        "(?:ab|cd)\u005cd?" flags="dg" input="cd2" "{\u0022matches\u0022:[\u0022cd2\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022cd2\u0022,\u0022indices\u0022:[[0,3]],\u0022lastIndex\u0022:3,\u0022source\u0022:\u0022(?:ab|cd)\u005c\u005cd?\u0022}"
        "(?:(a)|(b))(c|)" flags="d" input="bc" "{\u0022matches\u0022:[\u0022bc\u0022,null,\u0022b\u0022,\u0022c\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022bc\u0022,\u0022indices\u0022:[[0,2],null,[0,1],[1,2]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?:(a)|(b))(c|)\u0022}"
        "(a+)(b+)" flags="d" input="aaabbbb" "{\u0022matches\u0022:[\u0022aaabbbb\u0022,\u0022aaa\u0022,\u0022bbbb\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022aaabbbb\u0022,\u0022indices\u0022:[[0,7],[0,3],[3,7]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a+)(b+)\u0022}"
        "(a+?)(b+?)" flags="d" input="aaabbbb" "{\u0022matches\u0022:[\u0022aaab\u0022,\u0022aaa\u0022,\u0022b\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022aaabbbb\u0022,\u0022indices\u0022:[[0,4],[0,3],[3,4]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a+?)(b+?)\u0022}"
        "([ab]*)c" flags="d" input="aacc" "{\u0022matches\u0022:[\u0022aac\u0022,\u0022aa\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022aacc\u0022,\u0022indices\u0022:[[0,3],[0,2]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022([ab]*)c\u0022}"
        "([ab]*?)c" flags="d" input="aacc" "{\u0022matches\u0022:[\u0022aac\u0022,\u0022aa\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022aacc\u0022,\u0022indices\u0022:[[0,3],[0,2]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022([ab]*?)c\u0022}"
        "((a+)(b*))" flags="d" input="aaabb" "{\u0022matches\u0022:[\u0022aaabb\u0022,\u0022aaabb\u0022,\u0022aaa\u0022,\u0022bb\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022aaabb\u0022,\u0022indices\u0022:[[0,5],[0,5],[0,3],[3,5]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022((a+)(b*))\u0022}"
        "(?:(a*)b(c*))" flags="d" input="aabcc" "{\u0022matches\u0022:[\u0022aabcc\u0022,\u0022aa\u0022,\u0022cc\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022aabcc\u0022,\u0022indices\u0022:[[0,5],[0,2],[3,5]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?:(a*)b(c*))\u0022}"
        "^(a|ab)b$" flags="dm" input="q\u000aabb\u000az" "{\u0022matches\u0022:[\u0022abb\u0022,\u0022ab\u0022],\u0022index\u0022:2,\u0022input\u0022:\u0022q\u005cnabb\u005cnz\u0022,\u0022indices\u0022:[[2,5],[2,4]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022^(a|ab)b$\u0022}"
        "\u005cb(a+)\u005cb" flags="d" input=" a " "{\u0022matches\u0022:[\u0022a\u0022,\u0022a\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022 a \u0022,\u0022indices\u0022:[[1,2],[1,2]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022\u005c\u005cb(a+)\u005c\u005cb\u0022}"
        "(.+)(.)" flags="ds" input="\u000a\u000a" "{\u0022matches\u0022:[\u0022\u005cn\u005cn\u0022,\u0022\u005cn\u0022,\u0022\u005cn\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022\u005cn\u005cn\u0022,\u0022indices\u0022:[[0,2],[0,1],[1,2]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(.+)(.)\u0022}"
        "(.+?)(.)" flags="ds" input="\u000a\u000a" "{\u0022matches\u0022:[\u0022\u005cn\u005cn\u0022,\u0022\u005cn\u0022,\u0022\u005cn\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022\u005cn\u005cn\u0022,\u0022indices\u0022:[[0,2],[0,1],[1,2]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(.+?)(.)\u0022}"
        "(a?)" flags="d" input="q" "{\u0022matches\u0022:[\u0022\u0022,\u0022\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022q\u0022,\u0022indices\u0022:[[0,0],[0,0]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a?)\u0022}"
        "()?" flags="d" input="q" "{\u0022matches\u0022:[\u0022\u0022,null],\u0022index\u0022:0,\u0022input\u0022:\u0022q\u0022,\u0022indices\u0022:[[0,0],null],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022()?\u0022}"
        "()+" flags="d" input="q" "{\u0022matches\u0022:[\u0022\u0022,\u0022\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022q\u0022,\u0022indices\u0022:[[0,0],[0,0]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022()+\u0022}"
        "(?<x>a+)(?<y>b+)" flags="d" input="aaabbbb" "{\u0022matches\u0022:[\u0022aaabbbb\u0022,\u0022aaa\u0022,\u0022bbbb\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022aaabbbb\u0022,\u0022groups\u0022:{\u0022x\u0022:\u0022aaa\u0022,\u0022y\u0022:\u0022bbbb\u0022},\u0022indices\u0022:[[0,7],[0,3],[3,7]],\u0022indicesGroups\u0022:{\u0022x\u0022:[0,3],\u0022y\u0022:[3,7]},\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<x>a+)(?<y>b+)\u0022}"
        "(?<all>(?<x>a+)(?<y>b*))" flags="d" input="aaabb" "{\u0022matches\u0022:[\u0022aaabb\u0022,\u0022aaabb\u0022,\u0022aaa\u0022,\u0022bb\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022aaabb\u0022,\u0022groups\u0022:{\u0022all\u0022:\u0022aaabb\u0022,\u0022x\u0022:\u0022aaa\u0022,\u0022y\u0022:\u0022bb\u0022},\u0022indices\u0022:[[0,5],[0,5],[0,3],[3,5]],\u0022indicesGroups\u0022:{\u0022all\u0022:[0,5],\u0022x\u0022:[0,3],\u0022y\u0022:[3,5]},\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<all>(?<x>a+)(?<y>b*))\u0022}"
        "(?:(?<x>a+)|(?<x>b+))(c|)" flags="d" input="bbbc" "{\u0022matches\u0022:[\u0022bbbc\u0022,null,\u0022bbb\u0022,\u0022c\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022bbbc\u0022,\u0022groups\u0022:{\u0022x\u0022:\u0022bbb\u0022},\u0022indices\u0022:[[0,4],null,[0,3],[3,4]],\u0022indicesGroups\u0022:{\u0022x\u0022:[0,3]},\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?:(?<x>a+)|(?<x>b+))(c|)\u0022}"
        "(?<x>[ab]*)c" flags="dgy" input="q" "{\u0022match\u0022:null,\u0022lastIndex\u0022:0}"
        "#);
}

#[test]
fn nested_choices_and_quantified_prefixes_keep_complete_original_capture_layouts() {
    assert_true(
        r#"
            let a = /((1)|(12))((3)|(23))/d.exec('123');
            a[0] === '123' && a.length === 7 && a[1] === '1' && a[2] === '1' && a[3] === undefined && a[
                4] === '23' && a[5] === undefined && a[6] === '23' && a.indices[4][0] === 1 && a.indices[6]
                [1] === 3
        "#,
    );
    assert_true(
        r#"
            let a = /(?<all>(?<x>a+)(?<y>b*))/d.exec('aaabb');
            a[0] === 'aaabb' && a.groups.all === 'aaabb' && a.groups.x === 'aaa' && a.groups.y === 'bb' &&
                a.indices.groups.all === a.indices[1] && a.indices.groups.x === a.indices[2] && a.indices
                .groups.y === a.indices[3] && a.indices[3][0] === 3
        "#,
    );
    assert_true(
        r#"
            let a = /(?:(?<x>a+)|(?<x>b+))(c|)/d.exec('bbbc');
            a[0] === 'bbbc' && a[1] === undefined && a.groups.x === 'bbb' && a.indices.groups.x === a
                .indices[2] && a[3] === 'c' && a.indices[3][0] === 3
        "#,
    );
    assert_true(
        r#"
            let a = /(a|ab)b/d.exec('abb'),
                b = /(ab|a)b/d.exec('abb');
            a[0] === 'ab' && a[1] === 'a' && b[0] === 'abb' && b[1] === 'ab' && b.indices[1][1] === 2
        "#,
    );
    assert_true(
        r#"
            let a = /(a+?)(b+?)/d.exec('aaabbbb');
            a[0] === 'aaab' && a[1] === 'aaa' && a[2] === 'b' && a.indices[1][1] === 3 && a.indices[2][
                0] === 3 && a.indices[2][1] === 4
        "#,
    );
    assert_true(
        r#"
            let a = /(?<x>.+)(.)/d.exec('\uD800\uD800\uD800'),
                b = /(?<x>.+)(.)/ds.exec('\n\n');
            a.groups.x.length === 2 && a[2].charCodeAt(0) === 0xD800 && a.indices.groups.x === a.indices[
                1] && b.groups.x === '\n' && b[2] === '\n'
        "#,
    );
}

#[test]
fn ordinary_fallback_consumers_sticky_positions_and_empty_slots_keep_semantics() {
    assert_true(
        r#"
            let r = /(?:ab|cd)\d?/dg,
                a = [...'ab cd2 ab34 cd'.matchAll(r)];
            a.length === 4 && a[0][0] === 'ab' && a[1][0] === 'cd2' && a[1].index === 3 && a[2][0] ===
                'ab3' && a[2].index === 7 && a[3].index === 12 && r.lastIndex === 0
        "#,
    );
    assert_true(
        r"'qaaabbZ'.replace(/(?<x>a+)(?<y>b*)/,'<$<x>:$<y>>')==='q<aaa:bb>Z'&&'qaaabbZ'.search(/(a+)(b*)/)===1&&'qaaabbZ'.split(/(a+)(b*)/).join(',')==='q,aaa,bb,Z'",
    );
    assert_true(
        r#"
            let r = /(?<x>a+)(?<y>b*)/dy;
            r.lastIndex = 1;
            let copy = new RegExp(r),
                a = r.exec('qaaabb');
            a.groups.x === 'aaa' && a.groups.y === 'bb' && a.indices.groups.x[0] === 1 && a.indices.groups
                .y[0] === 4 && r.lastIndex === 6 && r.exec('qaaabb') === null && r.lastIndex === 0 && copy
                .source === r.source && copy.lastIndex === 0
        "#,
    );
    assert_true(
        r#"
            let a = [...'q'.matchAll(/(?<x>a?)/dg)],
                b = [...'q'.matchAll(/(?<x>)?/dg)],
                c = [...'q'.matchAll(/(?<x>)+/dg)];
            a.length === 2 && a[1].groups.x === '' && a[1].indices.groups.x[0] === 1 && b.length === 2 &&
                b[1].groups.x === undefined && b[1].indices.groups.x === undefined && c.length === 2 && c[
                    1].groups.x === '' && c[1].indices.groups.x[0] === 1
        "#,
    );
    assert_true(
        r#"
            let r = /^(?<x>a|ab)b$/dmy;
            r.lastIndex = 2;
            let a = r.exec('q\nabb\nz');
            a.index === 2 && a[0] === 'abb' && a.groups.x === 'ab' && a.indices.groups.x === a.indices[
                1] && a.indices.groups.x[0] === 2 && r.lastIndex === 5
        "#,
    );
}

#[test]
fn explicit_ordinary_fallback_work_aborts_before_last_index_and_handlers() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval(r"let marker=0,r=/(?<x>a+)(b*)c/g;r.lastIndex=1;let text='a'.repeat(5000)")
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
fn general_repeated_choices_nested_quantifiers_and_lookaround_match_reference_results() {
    for source in [
        r"/(?:(?:(ab|a)+)|){2,3}/.test('abaa')",
        r"/(?:(?:(a+)+)|){2,3}/.test('aaaa')",
        r"/(?:(?:(?:(a+)(b*)){2})|){2,3}/.test('aabb')",
        r"/(?<=(a+))a/.test('a')",
        r"/(?i:a)/.test('a')",
        r"/(a+)(b*)/u.test('ab')",
    ] {
        crate::common::regexp_oracle::assert_matches_reference(&mut Realm::default(), source);
    }
    assert_true(r"/(a+)(b+)c/.exec('ab')===null");
}
