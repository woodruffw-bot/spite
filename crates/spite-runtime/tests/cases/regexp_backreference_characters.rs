//! Character predicates and references share ordinary input capture semantics.

use spite_core::JsString;
use spite_runtime::{Realm, Value};
use std::fmt::Write;

use crate::common::assert_true;

#[test]
fn character_reference_result_snapshot() {
    let mut rows = String::new();
    for (source, flags, text) in [
        (r"([ab])\1", "d", "qbb"),
        (r"([a-z])\1", "di", "qAa"),
        (r"([^a])\1", "d", "qbb"),
        (r"([^µ])\1", "di", "µΣσ"),
        (r"(\w)\1", "d", "q__"),
        (r"(\W)\1", "di", "µΜ"),
        (r"(\d)\1", "d", "q11"),
        (r"(\D)\1", "d", "11aa"),
        (r"(\s)\1", "d", "q  "),
        (r"(\S)\1", "d", "  aa"),
        (r"(.)\1", "d", "\n\n"),
        (r"(.)\1", "ds", "\n\n"),
        (r"([^])\1", "d", "\n\n"),
        (r"([])\1", "d", "aa"),
        (r"\1(\w)", "d", "qa"),
        (r"(\w\1)", "d", "qa"),
        (r"([ab])(\1.)\2", "d", "qaacac"),
        (r"(?<x>[a-z])\k<x>", "di", "qAa"),
        (r"(?<x>.)\k<x>", "ds", "\n\n"),
        (r"(?<x>\w)\k<x>", "dg", "qaa"),
        (r"(?<x>\w)\k<x>", "dy", "qaa"),
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
        "([ab])\u005c1" flags="d" input="qbb" "{\u0022matches\u0022:[\u0022bb\u0022,\u0022b\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022qbb\u0022,\u0022indices\u0022:[[1,3],[1,2]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022([ab])\u005c\u005c1\u0022}"
        "([a-z])\u005c1" flags="di" input="qAa" "{\u0022matches\u0022:[\u0022Aa\u0022,\u0022A\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022qAa\u0022,\u0022indices\u0022:[[1,3],[1,2]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022([a-z])\u005c\u005c1\u0022}"
        "([^a])\u005c1" flags="d" input="qbb" "{\u0022matches\u0022:[\u0022bb\u0022,\u0022b\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022qbb\u0022,\u0022indices\u0022:[[1,3],[1,2]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022([^a])\u005c\u005c1\u0022}"
        "([^\u00b5])\u005c1" flags="di" input="\u00b5\u03a3\u03c3" "{\u0022matches\u0022:[\u0022\u03a3\u03c3\u0022,\u0022\u03a3\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022\u00b5\u03a3\u03c3\u0022,\u0022indices\u0022:[[1,3],[1,2]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022([^\u00b5])\u005c\u005c1\u0022}"
        "(\u005cw)\u005c1" flags="d" input="q__" "{\u0022matches\u0022:[\u0022__\u0022,\u0022_\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022q__\u0022,\u0022indices\u0022:[[1,3],[1,2]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(\u005c\u005cw)\u005c\u005c1\u0022}"
        "(\u005cW)\u005c1" flags="di" input="\u00b5\u039c" "{\u0022matches\u0022:[\u0022\u00b5\u039c\u0022,\u0022\u00b5\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022\u00b5\u039c\u0022,\u0022indices\u0022:[[0,2],[0,1]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(\u005c\u005cW)\u005c\u005c1\u0022}"
        "(\u005cd)\u005c1" flags="d" input="q11" "{\u0022matches\u0022:[\u002211\u0022,\u00221\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022q11\u0022,\u0022indices\u0022:[[1,3],[1,2]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(\u005c\u005cd)\u005c\u005c1\u0022}"
        "(\u005cD)\u005c1" flags="d" input="11aa" "{\u0022matches\u0022:[\u0022aa\u0022,\u0022a\u0022],\u0022index\u0022:2,\u0022input\u0022:\u002211aa\u0022,\u0022indices\u0022:[[2,4],[2,3]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(\u005c\u005cD)\u005c\u005c1\u0022}"
        "(\u005cs)\u005c1" flags="d" input="q  " "{\u0022matches\u0022:[\u0022  \u0022,\u0022 \u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022q  \u0022,\u0022indices\u0022:[[1,3],[1,2]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(\u005c\u005cs)\u005c\u005c1\u0022}"
        "(\u005cS)\u005c1" flags="d" input="  aa" "{\u0022matches\u0022:[\u0022aa\u0022,\u0022a\u0022],\u0022index\u0022:2,\u0022input\u0022:\u0022  aa\u0022,\u0022indices\u0022:[[2,4],[2,3]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(\u005c\u005cS)\u005c\u005c1\u0022}"
        "(.)\u005c1" flags="d" input="\u000a\u000a" "{\u0022match\u0022:null,\u0022lastIndex\u0022:0}"
        "(.)\u005c1" flags="ds" input="\u000a\u000a" "{\u0022matches\u0022:[\u0022\u005cn\u005cn\u0022,\u0022\u005cn\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022\u005cn\u005cn\u0022,\u0022indices\u0022:[[0,2],[0,1]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(.)\u005c\u005c1\u0022}"
        "([^])\u005c1" flags="d" input="\u000a\u000a" "{\u0022matches\u0022:[\u0022\u005cn\u005cn\u0022,\u0022\u005cn\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022\u005cn\u005cn\u0022,\u0022indices\u0022:[[0,2],[0,1]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022([^])\u005c\u005c1\u0022}"
        "([])\u005c1" flags="d" input="aa" "{\u0022match\u0022:null,\u0022lastIndex\u0022:0}"
        "\u005c1(\u005cw)" flags="d" input="qa" "{\u0022matches\u0022:[\u0022q\u0022,\u0022q\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022qa\u0022,\u0022indices\u0022:[[0,1],[0,1]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022\u005c\u005c1(\u005c\u005cw)\u0022}"
        "(\u005cw\u005c1)" flags="d" input="qa" "{\u0022matches\u0022:[\u0022q\u0022,\u0022q\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022qa\u0022,\u0022indices\u0022:[[0,1],[0,1]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(\u005c\u005cw\u005c\u005c1)\u0022}"
        "([ab])(\u005c1.)\u005c2" flags="d" input="qaacac" "{\u0022matches\u0022:[\u0022aacac\u0022,\u0022a\u0022,\u0022ac\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022qaacac\u0022,\u0022indices\u0022:[[1,6],[1,2],[2,4]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022([ab])(\u005c\u005c1.)\u005c\u005c2\u0022}"
        "(?<x>[a-z])\u005ck<x>" flags="di" input="qAa" "{\u0022matches\u0022:[\u0022Aa\u0022,\u0022A\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022qAa\u0022,\u0022groups\u0022:{\u0022x\u0022:\u0022A\u0022},\u0022indices\u0022:[[1,3],[1,2]],\u0022indicesGroups\u0022:{\u0022x\u0022:[1,2]},\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<x>[a-z])\u005c\u005ck<x>\u0022}"
        "(?<x>.)\u005ck<x>" flags="ds" input="\u000a\u000a" "{\u0022matches\u0022:[\u0022\u005cn\u005cn\u0022,\u0022\u005cn\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022\u005cn\u005cn\u0022,\u0022groups\u0022:{\u0022x\u0022:\u0022\u005cn\u0022},\u0022indices\u0022:[[0,2],[0,1]],\u0022indicesGroups\u0022:{\u0022x\u0022:[0,1]},\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<x>.)\u005c\u005ck<x>\u0022}"
        "(?<x>\u005cw)\u005ck<x>" flags="dg" input="qaa" "{\u0022matches\u0022:[\u0022aa\u0022,\u0022a\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022qaa\u0022,\u0022groups\u0022:{\u0022x\u0022:\u0022a\u0022},\u0022indices\u0022:[[1,3],[1,2]],\u0022indicesGroups\u0022:{\u0022x\u0022:[1,2]},\u0022lastIndex\u0022:3,\u0022source\u0022:\u0022(?<x>\u005c\u005cw)\u005c\u005ck<x>\u0022}"
        "(?<x>\u005cw)\u005ck<x>" flags="dy" input="qaa" "{\u0022match\u0022:null,\u0022lastIndex\u0022:0}"
        "#);
}

#[test]
fn classes_dot_case_and_surrogates_retain_input_captures_and_indices_aliases() {
    assert_true(
        r#"
            let a = /(?<x>[a-z])\k<x>/di.exec('qAa');
            a[0] === 'Aa' && a.groups.x === 'A' && a.indices.groups.x === a.indices[1] && a.indices[1][
                0] === 1 && a.indices[1][1] === 2
        "#,
    );
    assert_true(
        r#"
            let a = /(?<x>.)\k<x>/ds.exec('\n\n');
            a[0] === '\n\n' && a.groups.x === '\n' && a.indices.groups.x === a.indices[1] && a.indices[0][
                1] === 2
        "#,
    );
    assert_true(
        r#"
            let a = /(.)\1/d.exec('\uD800\uD800');
            a[0].length === 2 && a[1].length === 1 && a[1].charCodeAt(0) === 0xD800 && a.indices[1][1] ===
                1
        "#,
    );
    assert_true(
        r"/(\w)\1/i.exec('ſſ')===null&&/(\W)\1/i.exec('ſſ')[0]==='ſſ'&&/([^µ])\1/i.exec('µΜ')===null",
    );
    assert_true(
        r#"
            let a = /([ab])(\1.)\2/d.exec('qaacac');
            a[0] === 'aacac' && a[1] === 'a' && a[2] === 'ac' && a.indices[2][0] === 2 && a.indices[2][
                1] === 4
        "#,
    );
}

#[test]
fn generic_consumers_and_stateful_exec_use_character_reference_matches() {
    assert_true(
        r#"
            let r = /(?<x>\w)\k<x>/dgi,
                a = [...'aa bB cc'.matchAll(r)];
            a.length === 3 && a[1].groups.x === 'b' && a[1].indices.groups.x === a[1].indices[1] && r
                .lastIndex === 0
        "#,
    );
    assert_true(
        r"'qaa'.replace(/(?<x>\w)\k<x>/,'<$<x>>')==='q<a>'&&'qaa'.split(/(\w)\1/).join(',')==='q,a,'&&'qaa'.search(/(\w)\1/)===1",
    );
    assert_true(
        r#"
            let r = /(?<x>[ab])\k<x>/dy;
            r.lastIndex = 1;
            let a = r.exec('qbb');
            a.index === 1 && r.lastIndex === 3 && r.exec('qbb') === null && r.lastIndex === 0
        "#,
    );
}

#[test]
fn remaining_reference_compositions_keep_explicit_unsupported_outcomes() {
    for source in [
        r"/(?:(?:(?:([ab])(?:\1)+){2})|){2,3}/.test('aa')",
        r"/(?:(?:(?:(a|b)(?:\1)+){2})|){2,3}/.test('aa')",
        r"/(?:(?:(?:^([ab])(?:\1)+){2})|){2,3}/.test('aa')",
        r"/(?<x>[ab])\k<x>/u.test('aa')",
    ] {
        crate::common::regexp_oracle::assert_matches_reference(&mut Realm::default(), source);
    }
    assert_true(r"/([ab])\1/.exec('ab')===null&&/([])\1/.exec('aa')===null");
}
