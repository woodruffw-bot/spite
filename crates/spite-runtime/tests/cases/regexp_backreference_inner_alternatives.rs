//! Inner choices share surrounding terms and preserve capture checkpoints.

use spite_core::JsString;
use spite_runtime::{Error, Limits, Realm, Value};
use std::fmt::Write;

use crate::common::assert_true;

#[test]
fn inner_alternative_reference_result_snapshot() {
    let mut rows = String::new();
    for (source, flags, text) in [
        (r"(a|b)\1", "d", "qbb"),
        (r"(a|ab)\1", "d", "abab"),
        (r"(ab|a)\1", "d", "aa"),
        (r"x(a|ab)\1y", "d", "xababy"),
        (r"(a|ab)\1b", "d", "ababb"),
        (r"((a)|(ab))\1", "d", "qabab"),
        (r"((a)|(ab))\2\3", "d", "abab"),
        (r"(?:(a)|(b))\1\2", "d", "bb"),
        (r"(?:(a)|)\1", "d", "b"),
        (r"(?:|(a))\1", "d", "aa"),
        (r"(q)(?:(a)|(b))\1\2\3", "d", "qbqb"),
        (r"(?:(a\1)|(b\2))\1\2", "d", "bb"),
        (r"((\1a)|b)\1", "d", "aa"),
        (r"^(\w|\W)\1$", "di", "µΜ"),
        (r"([^µ]|[ab])\1", "di", "µΜ"),
        (r"\b(a|\w)\1\b", "d", " bb "),
        (r"((a)|())\1", "d", "b"),
        (r"x(((a)|b))\2y", "d", "xbby"),
        (r"(?:(?<x>a)|(?<x>ab))\k<x>", "d", "abab"),
        (r"^(?:(?<x>a)|(?<x>b)|z)\k<x>$", "d", "z"),
        (r"(q)(?:(?<x>a)|(?<x>b))\1\k<x>", "dgi", "qbqB"),
        (r"(?:(?<x>a)|(?<x>b))\k<x>", "dy", "qbb"),
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
        "(a|b)\u005c1" flags="d" input="qbb" "{\u0022matches\u0022:[\u0022bb\u0022,\u0022b\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022qbb\u0022,\u0022indices\u0022:[[1,3],[1,2]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a|b)\u005c\u005c1\u0022}"
        "(a|ab)\u005c1" flags="d" input="abab" "{\u0022matches\u0022:[\u0022abab\u0022,\u0022ab\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022abab\u0022,\u0022indices\u0022:[[0,4],[0,2]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a|ab)\u005c\u005c1\u0022}"
        "(ab|a)\u005c1" flags="d" input="aa" "{\u0022matches\u0022:[\u0022aa\u0022,\u0022a\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022aa\u0022,\u0022indices\u0022:[[0,2],[0,1]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(ab|a)\u005c\u005c1\u0022}"
        "x(a|ab)\u005c1y" flags="d" input="xababy" "{\u0022matches\u0022:[\u0022xababy\u0022,\u0022ab\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022xababy\u0022,\u0022indices\u0022:[[0,6],[1,3]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022x(a|ab)\u005c\u005c1y\u0022}"
        "(a|ab)\u005c1b" flags="d" input="ababb" "{\u0022matches\u0022:[\u0022ababb\u0022,\u0022ab\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022ababb\u0022,\u0022indices\u0022:[[0,5],[0,2]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a|ab)\u005c\u005c1b\u0022}"
        "((a)|(ab))\u005c1" flags="d" input="qabab" "{\u0022matches\u0022:[\u0022abab\u0022,\u0022ab\u0022,null,\u0022ab\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022qabab\u0022,\u0022indices\u0022:[[1,5],[1,3],null,[1,3]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022((a)|(ab))\u005c\u005c1\u0022}"
        "((a)|(ab))\u005c2\u005c3" flags="d" input="abab" "{\u0022matches\u0022:[\u0022abab\u0022,\u0022ab\u0022,null,\u0022ab\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022abab\u0022,\u0022indices\u0022:[[0,4],[0,2],null,[0,2]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022((a)|(ab))\u005c\u005c2\u005c\u005c3\u0022}"
        "(?:(a)|(b))\u005c1\u005c2" flags="d" input="bb" "{\u0022matches\u0022:[\u0022bb\u0022,null,\u0022b\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022bb\u0022,\u0022indices\u0022:[[0,2],null,[0,1]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?:(a)|(b))\u005c\u005c1\u005c\u005c2\u0022}"
        "(?:(a)|)\u005c1" flags="d" input="b" "{\u0022matches\u0022:[\u0022\u0022,null],\u0022index\u0022:0,\u0022input\u0022:\u0022b\u0022,\u0022indices\u0022:[[0,0],null],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?:(a)|)\u005c\u005c1\u0022}"
        "(?:|(a))\u005c1" flags="d" input="aa" "{\u0022matches\u0022:[\u0022\u0022,null],\u0022index\u0022:0,\u0022input\u0022:\u0022aa\u0022,\u0022indices\u0022:[[0,0],null],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?:|(a))\u005c\u005c1\u0022}"
        "(q)(?:(a)|(b))\u005c1\u005c2\u005c3" flags="d" input="qbqb" "{\u0022matches\u0022:[\u0022qbqb\u0022,\u0022q\u0022,null,\u0022b\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022qbqb\u0022,\u0022indices\u0022:[[0,4],[0,1],null,[1,2]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(q)(?:(a)|(b))\u005c\u005c1\u005c\u005c2\u005c\u005c3\u0022}"
        "(?:(a\u005c1)|(b\u005c2))\u005c1\u005c2" flags="d" input="bb" "{\u0022matches\u0022:[\u0022bb\u0022,null,\u0022b\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022bb\u0022,\u0022indices\u0022:[[0,2],null,[0,1]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?:(a\u005c\u005c1)|(b\u005c\u005c2))\u005c\u005c1\u005c\u005c2\u0022}"
        "((\u005c1a)|b)\u005c1" flags="d" input="aa" "{\u0022matches\u0022:[\u0022aa\u0022,\u0022a\u0022,\u0022a\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022aa\u0022,\u0022indices\u0022:[[0,2],[0,1],[0,1]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022((\u005c\u005c1a)|b)\u005c\u005c1\u0022}"
        "^(\u005cw|\u005cW)\u005c1$" flags="di" input="\u00b5\u039c" "{\u0022matches\u0022:[\u0022\u00b5\u039c\u0022,\u0022\u00b5\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022\u00b5\u039c\u0022,\u0022indices\u0022:[[0,2],[0,1]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022^(\u005c\u005cw|\u005c\u005cW)\u005c\u005c1$\u0022}"
        "([^\u00b5]|[ab])\u005c1" flags="di" input="\u00b5\u039c" "{\u0022match\u0022:null,\u0022lastIndex\u0022:0}"
        "\u005cb(a|\u005cw)\u005c1\u005cb" flags="d" input=" bb " "{\u0022matches\u0022:[\u0022bb\u0022,\u0022b\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022 bb \u0022,\u0022indices\u0022:[[1,3],[1,2]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022\u005c\u005cb(a|\u005c\u005cw)\u005c\u005c1\u005c\u005cb\u0022}"
        "((a)|())\u005c1" flags="d" input="b" "{\u0022matches\u0022:[\u0022\u0022,\u0022\u0022,null,\u0022\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022b\u0022,\u0022indices\u0022:[[0,0],[0,0],null,[0,0]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022((a)|())\u005c\u005c1\u0022}"
        "x(((a)|b))\u005c2y" flags="d" input="xbby" "{\u0022matches\u0022:[\u0022xbby\u0022,\u0022b\u0022,\u0022b\u0022,null],\u0022index\u0022:0,\u0022input\u0022:\u0022xbby\u0022,\u0022indices\u0022:[[0,4],[1,2],[1,2],null],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022x(((a)|b))\u005c\u005c2y\u0022}"
        "(?:(?<x>a)|(?<x>ab))\u005ck<x>" flags="d" input="abab" "{\u0022matches\u0022:[\u0022abab\u0022,null,\u0022ab\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022abab\u0022,\u0022groups\u0022:{\u0022x\u0022:\u0022ab\u0022},\u0022indices\u0022:[[0,4],null,[0,2]],\u0022indicesGroups\u0022:{\u0022x\u0022:[0,2]},\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?:(?<x>a)|(?<x>ab))\u005c\u005ck<x>\u0022}"
        "^(?:(?<x>a)|(?<x>b)|z)\u005ck<x>$" flags="d" input="z" "{\u0022matches\u0022:[\u0022z\u0022,null,null],\u0022index\u0022:0,\u0022input\u0022:\u0022z\u0022,\u0022groups\u0022:{},\u0022indices\u0022:[[0,1],null,null],\u0022indicesGroups\u0022:{},\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022^(?:(?<x>a)|(?<x>b)|z)\u005c\u005ck<x>$\u0022}"
        "(q)(?:(?<x>a)|(?<x>b))\u005c1\u005ck<x>" flags="dgi" input="qbqB" "{\u0022matches\u0022:[\u0022qbqB\u0022,\u0022q\u0022,null,\u0022b\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022qbqB\u0022,\u0022groups\u0022:{\u0022x\u0022:\u0022b\u0022},\u0022indices\u0022:[[0,4],[0,1],null,[1,2]],\u0022indicesGroups\u0022:{\u0022x\u0022:[1,2]},\u0022lastIndex\u0022:4,\u0022source\u0022:\u0022(q)(?:(?<x>a)|(?<x>b))\u005c\u005c1\u005c\u005ck<x>\u0022}"
        "(?:(?<x>a)|(?<x>b))\u005ck<x>" flags="dy" input="qbb" "{\u0022match\u0022:null,\u0022lastIndex\u0022:0}"
        "#);
}

#[test]
fn common_enclosing_and_inactive_capture_slots_survive_failed_continuations() {
    assert_true(
        r#"
            let a = /(?:(?<x>a)|(?<x>ab))\k<x>/d.exec('qabab');
            a.index === 1 && a[1] === undefined && a[2] === 'ab' && a.groups.x === 'ab' && a.indices.groups
                .x === a.indices[2] && a.indices[2][0] === 1 && a.indices[2][1] === 3
        "#,
    );
    assert_true(
        r#"
            let a = /((a)|(ab))\1/d.exec('qabab');
            a[0] === 'abab' && a[1] === 'ab' && a[2] === undefined && a[3] === 'ab' && a.indices[1][0] ===
                1 && a.indices[1][1] === 3
        "#,
    );
    assert_true(
        r#"
            let a = /(?<q>q)(?:(?<x>a)|(?<x>b))\k<q>\k<x>/d.exec('qbqb');
            a.groups.q === 'q' && a.groups.x === 'b' && a[2] === undefined && a.indices.groups.q === a
                .indices[1] && a.indices.groups.x === a.indices[3]
        "#,
    );
    assert_true(
        r#"
            let a = /((a)|())\1/d.exec('b');
            a[0] === '' && a[1] === '' && a[2] === undefined && a[3] === '' && a.indices[3][0] === 0 && a
                .indices[3][1] === 0
        "#,
    );
    assert_true(
        r#"
            let a = /(?:(?<x>a)|\B(?<x>.)\B)\k<x>/d.exec('\uD800\uD800');
            a.groups.x.charCodeAt(0) === 0xD800 && a.indices.groups.x === a.indices[2] && a.indices[2][
                1] === 1
        "#,
    );
}

#[test]
fn consumers_sticky_positions_and_empty_choices_keep_order_and_original_sources() {
    assert_true(
        r#"
            let r = /(?:(?<x>a)|(?<x>ab))\k<x>/dg,
                a = [...'abab aa'.matchAll(r)];
            a.length === 2 && a[0].groups.x === 'ab' && a[1].groups.x === 'a' && a[0].indices.groups.x ===
                a[0].indices[2] && r.lastIndex === 0
        "#,
    );
    assert_true(
        r"'qabab'.replace(/(?:(?<x>a)|(?<x>ab))\k<x>/,'<$<x>>')==='q<ab>'&&'qabab'.search(/(a|ab)\1/)===1&&'qababz'.split(/(?:(a)|(ab))\1\2/).join(',')==='q,,ab,z'",
    );
    assert_true(
        r#"
            let r = /(?:(?<x>a)|(?<x>b))\k<x>/dy;
            r.lastIndex = 1;
            let copy = new RegExp(r),
                a = r.exec('qbb');
            a.groups.x === 'b' && r.lastIndex === 3 && r.exec('qbb') === null && r.lastIndex === 0 && copy
                .source === r.source && copy.lastIndex === 0
        "#,
    );
    assert_true(
        r#"
            let a = [...'q'.matchAll(/(?:(?<x>a)|)\k<x>/dg)];
            a.length === 2 && a[0].index === 0 && a[1].index === 1 && a[0].groups.x === undefined
        "#,
    );
}

#[test]
fn explicit_inner_choice_search_work_aborts_before_last_index_and_handlers() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval(r"let r=/(?:(?<x>a)|(?<x>b))\k<x>c/g,flag=0,text='ab'.repeat(5000);r.lastIndex=1")
        .unwrap();
    assert!(matches!(
        realm.eval("try{r.exec(text)}catch{flag=1}finally{flag=2}"),
        Err(Error::Limit { .. })
    ));
    assert_eq!(
        realm.eval("flag===0&&r.lastIndex===1"),
        Ok(Value::Boolean(true))
    );
}

#[test]
fn pattern_variants_match_reference_results() {
    for source in [
        r"/(?:(?:(?:(a|b)(c|d)(?:\1)+){2})|){2,3}/.test('aca')",
        r"/(?:(?:(?:(?:(a|b)|c)(?:\1)+){2})|){2,3}/.test('aa')",
        r"/(?:(?:(?:(?:(?<x>a)|(?<x>b))(?:\k<x>)+){2})|){2,3}/.test('bb')",
        r"/(?:(?<x>a)|(?<x>b))\k<x>/u.test('bb')",
    ] {
        crate::common::regexp_oracle::assert_matches_reference(&mut Realm::default(), source);
    }
    assert_true(r"/(?:(?<x>a)|(?<x>b))\k<x>/.exec('ab')===null");
}
