//! Nested ordinary alternatives use flat control flow and capture checkpoints.

use spite_core::JsString;
use spite_runtime::{Error, Limits, Realm, Value};
use std::fmt::Write;

use crate::common::assert_true;

#[test]
fn nested_alternative_reference_result_snapshot() {
    let mut rows = String::new();
    for (source, flags, text) in [
        (r"(?:(a|ab)|b)\1", "d", "qabab"),
        (r"(?:a|(b|bc))\1", "d", "bcbc"),
        (r"((a|ab)(c|)|e)\1", "d", "abab"),
        (r"((a|ab)(c|)|e)\1", "d", "ee"),
        (r"((a)|(ab|abc))\1\2\3", "d", "abcabcabc"),
        (r"(?:(?:(a)|(ab))|(b))\1\2\3", "d", "qabab"),
        (r"(q)(?:(a|ab)|b)(c|)\1\2\3", "d", "qabqab"),
        (r"(?:(?:(a)|)|(?:(b)|))\1\2", "d", "bb"),
        (r"((a|)|(b|))\1", "d", "b"),
        (r"(?:(a\1|b)|c)\1", "d", "aa"),
        (r"((\1a|b)|c)\1", "d", "aa"),
        (r"(?:(\w|\W)|(\d|\D))\1\2", "di", "µΜ"),
        (r"([^µ]|(?:[ab]|µ))\1", "di", "µΜ"),
        (r"\b((a|\w)|b)\1\b", "d", " bb "),
        (r"((a|ab)|c)(b|(?:c|))\1\3", "d", "abab"),
        (r"(?:(a|ab)|b)\1|(c|cd)\2", "d", "cdcd"),
        (r"(?:(a|ab)|b)\1|", "dg", "q"),
        (r"((?:(a)|b)|())\1\2\3", "d", "b"),
        (r"x(((a)|(?:b|c)))\2y", "d", "xccy"),
        (r"(?:(?:(?<x>a)|(?<x>ab))|(?<x>b))\k<x>", "d", "abab"),
        (r"(?:(?:(?<x>a)|(?<x>b))|z)\k<x>", "d", "z"),
        (r"(?:(?:(?<x>a)|(?<x>b))|(?<x>c))\k<x>", "dgi", "cC"),
        (r"(?:(?:(?<x>a)|(?<x>b))|(?<x>c))\k<x>", "dy", "qcc"),
        (r"((.|(?:\w|\W))(.|\d))\1", "ds", "\n1\n1"),
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
        "(?:(a|ab)|b)\u005c1" flags="d" input="qabab" "{\u0022matches\u0022:[\u0022abab\u0022,\u0022ab\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022qabab\u0022,\u0022indices\u0022:[[1,5],[1,3]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?:(a|ab)|b)\u005c\u005c1\u0022}"
        "(?:a|(b|bc))\u005c1" flags="d" input="bcbc" "{\u0022matches\u0022:[\u0022bcbc\u0022,\u0022bc\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022bcbc\u0022,\u0022indices\u0022:[[0,4],[0,2]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?:a|(b|bc))\u005c\u005c1\u0022}"
        "((a|ab)(c|)|e)\u005c1" flags="d" input="abab" "{\u0022matches\u0022:[\u0022abab\u0022,\u0022ab\u0022,\u0022ab\u0022,\u0022\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022abab\u0022,\u0022indices\u0022:[[0,4],[0,2],[0,2],[2,2]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022((a|ab)(c|)|e)\u005c\u005c1\u0022}"
        "((a|ab)(c|)|e)\u005c1" flags="d" input="ee" "{\u0022matches\u0022:[\u0022ee\u0022,\u0022e\u0022,null,null],\u0022index\u0022:0,\u0022input\u0022:\u0022ee\u0022,\u0022indices\u0022:[[0,2],[0,1],null,null],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022((a|ab)(c|)|e)\u005c\u005c1\u0022}"
        "((a)|(ab|abc))\u005c1\u005c2\u005c3" flags="d" input="abcabcabc" "{\u0022matches\u0022:[\u0022abcabcabc\u0022,\u0022abc\u0022,null,\u0022abc\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022abcabcabc\u0022,\u0022indices\u0022:[[0,9],[0,3],null,[0,3]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022((a)|(ab|abc))\u005c\u005c1\u005c\u005c2\u005c\u005c3\u0022}"
        "(?:(?:(a)|(ab))|(b))\u005c1\u005c2\u005c3" flags="d" input="qabab" "{\u0022matches\u0022:[\u0022abab\u0022,null,\u0022ab\u0022,null],\u0022index\u0022:1,\u0022input\u0022:\u0022qabab\u0022,\u0022indices\u0022:[[1,5],null,[1,3],null],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?:(?:(a)|(ab))|(b))\u005c\u005c1\u005c\u005c2\u005c\u005c3\u0022}"
        "(q)(?:(a|ab)|b)(c|)\u005c1\u005c2\u005c3" flags="d" input="qabqab" "{\u0022matches\u0022:[\u0022qabqab\u0022,\u0022q\u0022,\u0022ab\u0022,\u0022\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022qabqab\u0022,\u0022indices\u0022:[[0,6],[0,1],[1,3],[3,3]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(q)(?:(a|ab)|b)(c|)\u005c\u005c1\u005c\u005c2\u005c\u005c3\u0022}"
        "(?:(?:(a)|)|(?:(b)|))\u005c1\u005c2" flags="d" input="bb" "{\u0022matches\u0022:[\u0022\u0022,null,null],\u0022index\u0022:0,\u0022input\u0022:\u0022bb\u0022,\u0022indices\u0022:[[0,0],null,null],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?:(?:(a)|)|(?:(b)|))\u005c\u005c1\u005c\u005c2\u0022}"
        "((a|)|(b|))\u005c1" flags="d" input="b" "{\u0022matches\u0022:[\u0022\u0022,\u0022\u0022,\u0022\u0022,null],\u0022index\u0022:0,\u0022input\u0022:\u0022b\u0022,\u0022indices\u0022:[[0,0],[0,0],[0,0],null],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022((a|)|(b|))\u005c\u005c1\u0022}"
        "(?:(a\u005c1|b)|c)\u005c1" flags="d" input="aa" "{\u0022matches\u0022:[\u0022aa\u0022,\u0022a\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022aa\u0022,\u0022indices\u0022:[[0,2],[0,1]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?:(a\u005c\u005c1|b)|c)\u005c\u005c1\u0022}"
        "((\u005c1a|b)|c)\u005c1" flags="d" input="aa" "{\u0022matches\u0022:[\u0022aa\u0022,\u0022a\u0022,\u0022a\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022aa\u0022,\u0022indices\u0022:[[0,2],[0,1],[0,1]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022((\u005c\u005c1a|b)|c)\u005c\u005c1\u0022}"
        "(?:(\u005cw|\u005cW)|(\u005cd|\u005cD))\u005c1\u005c2" flags="di" input="\u00b5\u039c" "{\u0022matches\u0022:[\u0022\u00b5\u039c\u0022,\u0022\u00b5\u0022,null],\u0022index\u0022:0,\u0022input\u0022:\u0022\u00b5\u039c\u0022,\u0022indices\u0022:[[0,2],[0,1],null],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?:(\u005c\u005cw|\u005c\u005cW)|(\u005c\u005cd|\u005c\u005cD))\u005c\u005c1\u005c\u005c2\u0022}"
        "([^\u00b5]|(?:[ab]|\u00b5))\u005c1" flags="di" input="\u00b5\u039c" "{\u0022matches\u0022:[\u0022\u00b5\u039c\u0022,\u0022\u00b5\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022\u00b5\u039c\u0022,\u0022indices\u0022:[[0,2],[0,1]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022([^\u00b5]|(?:[ab]|\u00b5))\u005c\u005c1\u0022}"
        "\u005cb((a|\u005cw)|b)\u005c1\u005cb" flags="d" input=" bb " "{\u0022matches\u0022:[\u0022bb\u0022,\u0022b\u0022,\u0022b\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022 bb \u0022,\u0022indices\u0022:[[1,3],[1,2],[1,2]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022\u005c\u005cb((a|\u005c\u005cw)|b)\u005c\u005c1\u005c\u005cb\u0022}"
        "((a|ab)|c)(b|(?:c|))\u005c1\u005c3" flags="d" input="abab" "{\u0022matches\u0022:[\u0022abab\u0022,\u0022a\u0022,\u0022a\u0022,\u0022b\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022abab\u0022,\u0022indices\u0022:[[0,4],[0,1],[0,1],[1,2]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022((a|ab)|c)(b|(?:c|))\u005c\u005c1\u005c\u005c3\u0022}"
        "(?:(a|ab)|b)\u005c1|(c|cd)\u005c2" flags="d" input="cdcd" "{\u0022matches\u0022:[\u0022cdcd\u0022,null,\u0022cd\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022cdcd\u0022,\u0022indices\u0022:[[0,4],null,[0,2]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?:(a|ab)|b)\u005c\u005c1|(c|cd)\u005c\u005c2\u0022}"
        "(?:(a|ab)|b)\u005c1|" flags="dg" input="q" "{\u0022matches\u0022:[\u0022\u0022,null],\u0022index\u0022:0,\u0022input\u0022:\u0022q\u0022,\u0022indices\u0022:[[0,0],null],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?:(a|ab)|b)\u005c\u005c1|\u0022}"
        "((?:(a)|b)|())\u005c1\u005c2\u005c3" flags="d" input="b" "{\u0022matches\u0022:[\u0022\u0022,\u0022\u0022,null,\u0022\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022b\u0022,\u0022indices\u0022:[[0,0],[0,0],null,[0,0]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022((?:(a)|b)|())\u005c\u005c1\u005c\u005c2\u005c\u005c3\u0022}"
        "x(((a)|(?:b|c)))\u005c2y" flags="d" input="xccy" "{\u0022matches\u0022:[\u0022xccy\u0022,\u0022c\u0022,\u0022c\u0022,null],\u0022index\u0022:0,\u0022input\u0022:\u0022xccy\u0022,\u0022indices\u0022:[[0,4],[1,2],[1,2],null],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022x(((a)|(?:b|c)))\u005c\u005c2y\u0022}"
        "(?:(?:(?<x>a)|(?<x>ab))|(?<x>b))\u005ck<x>" flags="d" input="abab" "{\u0022matches\u0022:[\u0022abab\u0022,null,\u0022ab\u0022,null],\u0022index\u0022:0,\u0022input\u0022:\u0022abab\u0022,\u0022groups\u0022:{\u0022x\u0022:\u0022ab\u0022},\u0022indices\u0022:[[0,4],null,[0,2],null],\u0022indicesGroups\u0022:{\u0022x\u0022:[0,2]},\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?:(?:(?<x>a)|(?<x>ab))|(?<x>b))\u005c\u005ck<x>\u0022}"
        "(?:(?:(?<x>a)|(?<x>b))|z)\u005ck<x>" flags="d" input="z" "{\u0022matches\u0022:[\u0022z\u0022,null,null],\u0022index\u0022:0,\u0022input\u0022:\u0022z\u0022,\u0022groups\u0022:{},\u0022indices\u0022:[[0,1],null,null],\u0022indicesGroups\u0022:{},\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?:(?:(?<x>a)|(?<x>b))|z)\u005c\u005ck<x>\u0022}"
        "(?:(?:(?<x>a)|(?<x>b))|(?<x>c))\u005ck<x>" flags="dgi" input="cC" "{\u0022matches\u0022:[\u0022cC\u0022,null,null,\u0022c\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022cC\u0022,\u0022groups\u0022:{\u0022x\u0022:\u0022c\u0022},\u0022indices\u0022:[[0,2],null,null,[0,1]],\u0022indicesGroups\u0022:{\u0022x\u0022:[0,1]},\u0022lastIndex\u0022:2,\u0022source\u0022:\u0022(?:(?:(?<x>a)|(?<x>b))|(?<x>c))\u005c\u005ck<x>\u0022}"
        "(?:(?:(?<x>a)|(?<x>b))|(?<x>c))\u005ck<x>" flags="dy" input="qcc" "{\u0022match\u0022:null,\u0022lastIndex\u0022:0}"
        "((.|(?:\u005cw|\u005cW))(.|\u005cd))\u005c1" flags="ds" input="\u000a1\u000a1" "{\u0022matches\u0022:[\u0022\u005cn1\u005cn1\u0022,\u0022\u005cn1\u0022,\u0022\u005cn\u0022,\u00221\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022\u005cn1\u005cn1\u0022,\u0022indices\u0022:[[0,4],[0,2],[0,1],[1,2]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022((.|(?:\u005c\u005cw|\u005c\u005cW))(.|\u005c\u005cd))\u005c\u005c1\u0022}"
        "#);
}

#[test]
fn nested_failure_restores_prior_names_and_enclosing_capture_slots() {
    assert_true(
        r#"
            let a = /(?:(?:(?<x>a)|(?<x>ab))|(?<x>b))\k<x>/d.exec('qabab');
            a.index === 1 && a.groups.x === 'ab' && a[1] === undefined && a[3] === undefined && a.indices
                .groups.x === a.indices[2] && a.indices[2][0] === 1 && a.indices[2][1] === 3
        "#,
    );
    assert_true(
        r#"
            let a = /(?<q>q)(?:(?:(?<x>a)|(?<x>ab))|(?<x>b))\k<q>\k<x>/d.exec('qabqab');
            a[0] === 'qabqab' && a.groups.q === 'q' && a.groups.x === 'ab' && a.indices.groups.q === a
                .indices[1] && a.indices.groups.x === a.indices[3]
        "#,
    );
    assert_true(
        r#"
            let a = /((a|ab)(c|)|e)\1/d.exec('ee');
            a[0] === 'ee' && a[1] === 'e' && a[2] === undefined && a[3] === undefined && a.indices[2] ===
                undefined && a.indices[1][1] === 1
        "#,
    );
    assert_true(
        r#"
            let a = /(?:(?:(?<x>a)|(?<x>b))|z)\k<x>/d.exec('z');
            a[0] === 'z' && a.groups.x === undefined && a.indices.groups.x === undefined
        "#,
    );
    assert_true(
        r#"
            let a = /(?:(a)|(?:(?<x>b)|(?<x>.)))\k<x>/d.exec('\uD800\uD800');
            a.groups.x.charCodeAt(0) === 0xD800 && a.indices.groups.x === a.indices[3] && a.indices[1] ===
                undefined
        "#,
    );
}

#[test]
fn nested_consumers_and_empty_choices_keep_source_order_and_original_text() {
    assert_true(
        r"let r=/(?:(?:(?<x>a)|(?<x>ab))|(?<x>b))\k<x>/dg,a=[...'abab bb'.matchAll(r)];a.length===2&&a[0].groups.x==='ab'&&a[1].groups.x==='b'&&a[0].indices.groups.x===a[0].indices[2]&&r.lastIndex===0",
    );
    assert_true(
        r"'qabab'.replace(/(?:(?:(?<x>a)|(?<x>ab))|(?<x>b))\k<x>/,'<$<x>>')==='q<ab>'&&'qabab'.search(/(?:(a|ab)|b)\1/)===1&&'qababz'.split(/(?:(?:(a)|(ab))|(b))\1\2\3/).join(',')==='q,,ab,,z'",
    );
    assert_true(
        r"let r=/(?:(?:(?<x>a)|(?<x>b))|(?<x>c))\k<x>/dy;r.lastIndex=1;let copy=new RegExp(r),a=r.exec('qcc');a.groups.x==='c'&&r.lastIndex===3&&r.exec('qcc')===null&&r.lastIndex===0&&copy.source===r.source&&copy.lastIndex===0",
    );
    assert_true(
        r"let a=[...'q'.matchAll(/(?:(?:(?<x>a)|)|(?<x>b))\k<x>/dg)];a.length===2&&a[0].index===0&&a[1].index===1&&a[0].groups.x===undefined",
    );
}

#[test]
fn explicit_nested_choice_work_aborts_before_last_index_and_handlers() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm.eval(r"let r=/(?:(?:(?<x>a)|(?<x>b))|(?<x>c))\k<x>d/g,flag=0,text='ab'.repeat(5000);r.lastIndex=1").unwrap();
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
fn quantified_reference_bodies_and_lookaround_match_reference_results() {
    for source in [
        r"/(?:(?:(?:(?:(a|b)|c)(?:\1)+){2})|){2,3}/.test('aa')",
        r"/(?:(?:(?:(?:a|(b|c))(?:\1)+){2})|){2,3}/.test('bb')",
        r"/(?:(?:(?:((a|b)(c|d)|e)(?:\1)+){2})|){2,3}/.test('acac')",
        r"/(?:(?:(?<x>a)|(?<x>b))|(?<x>c))\k<x>/u.test('cc')",
        r"/(?<=(a|bb))\1/.test('aa')",
    ] {
        crate::common::regexp_oracle::assert_matches_reference(&mut Realm::default(), source);
    }
    assert_true(r"/(?:(a|b)|c)\1/.exec('ab')===null");
}
