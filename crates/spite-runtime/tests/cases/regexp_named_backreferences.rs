//! Ordinary named references reuse input capture state and preserve source text.

use spite_core::JsString;
use spite_runtime::{Error, Limits, Realm, Value};
use std::fmt::Write;

use crate::common::assert_true;

#[test]
fn named_reference_result_snapshot() {
    let mut rows = String::new();
    for (source, flags, text) in [
        (r"(?<x>a)\k<x>", "d", "qaa"),
        (r"(?<x>a)\k<x>", "di", "qAa"),
        (r"\k<x>(?<x>a)", "d", "qa"),
        (r"(?<x>a\k<x>)", "d", "qa"),
        (r"(?<outer>a(?<x>\k<outer>b)\k<x>)\k<outer>", "d", "qabbabb"),
        (r"(?<x>)(?<y>a)\k<x>\k<y>", "d", "qaa"),
        (r"(?<x>ab)(?<y>\k<x>c)\k<y>", "d", "qababcabc"),
        (r"(?<x>a)\k<x>0", "d", "qaa0"),
        (r"(?<x>a)\k<x>12", "d", "qaa12"),
        (r"(?<x>a)\k<x>\k<x>", "dg", "qaaa"),
        (r"(?<\u0078>a)\k<x>", "d", "qaa"),
        (r"(?<x>a)\k<\u{78}>", "d", "qaa"),
        (r"(?<𐐀>a)\k<\u{10400}>", "d", "qaa"),
        (r"(?<__proto__>a)\k<__proto__>", "d", "qaa"),
        (r"(?<µ>a)\k<µ>", "di", "qAa"),
        (r"(?<x>µ)\k<x>", "di", "qΜµ"),
        (r"(?<x>ſ)\k<x>", "di", "ſS"),
        (r"(?<x>\uD800\uDC00)\k<x>", "d", "𐀀𐀀"),
        (r"(?<x>a)\1\k<x>", "d", "qaaa"),
        (r"()()()()()()()()()(?<x>a)\k<x>0", "d", "qaa0"),
        (r"(?<x>)\k<x>", "dg", "q"),
        (r"(?<x>a)\k<x>", "dy", "qaa"),
        (r"(?<x>a)\k<x>", "d", "qaba"),
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
        "(?<x>a)\u005ck<x>" flags="d" input="qaa" "{\u0022matches\u0022:[\u0022aa\u0022,\u0022a\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022qaa\u0022,\u0022groups\u0022:{\u0022x\u0022:\u0022a\u0022},\u0022indices\u0022:[[1,3],[1,2]],\u0022indicesGroups\u0022:{\u0022x\u0022:[1,2]},\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<x>a)\u005c\u005ck<x>\u0022}"
        "(?<x>a)\u005ck<x>" flags="di" input="qAa" "{\u0022matches\u0022:[\u0022Aa\u0022,\u0022A\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022qAa\u0022,\u0022groups\u0022:{\u0022x\u0022:\u0022A\u0022},\u0022indices\u0022:[[1,3],[1,2]],\u0022indicesGroups\u0022:{\u0022x\u0022:[1,2]},\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<x>a)\u005c\u005ck<x>\u0022}"
        "\u005ck<x>(?<x>a)" flags="d" input="qa" "{\u0022matches\u0022:[\u0022a\u0022,\u0022a\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022qa\u0022,\u0022groups\u0022:{\u0022x\u0022:\u0022a\u0022},\u0022indices\u0022:[[1,2],[1,2]],\u0022indicesGroups\u0022:{\u0022x\u0022:[1,2]},\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022\u005c\u005ck<x>(?<x>a)\u0022}"
        "(?<x>a\u005ck<x>)" flags="d" input="qa" "{\u0022matches\u0022:[\u0022a\u0022,\u0022a\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022qa\u0022,\u0022groups\u0022:{\u0022x\u0022:\u0022a\u0022},\u0022indices\u0022:[[1,2],[1,2]],\u0022indicesGroups\u0022:{\u0022x\u0022:[1,2]},\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<x>a\u005c\u005ck<x>)\u0022}"
        "(?<outer>a(?<x>\u005ck<outer>b)\u005ck<x>)\u005ck<outer>" flags="d" input="qabbabb" "{\u0022matches\u0022:[\u0022abbabb\u0022,\u0022abb\u0022,\u0022b\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022qabbabb\u0022,\u0022groups\u0022:{\u0022outer\u0022:\u0022abb\u0022,\u0022x\u0022:\u0022b\u0022},\u0022indices\u0022:[[1,7],[1,4],[2,3]],\u0022indicesGroups\u0022:{\u0022outer\u0022:[1,4],\u0022x\u0022:[2,3]},\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<outer>a(?<x>\u005c\u005ck<outer>b)\u005c\u005ck<x>)\u005c\u005ck<outer>\u0022}"
        "(?<x>)(?<y>a)\u005ck<x>\u005ck<y>" flags="d" input="qaa" "{\u0022matches\u0022:[\u0022aa\u0022,\u0022\u0022,\u0022a\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022qaa\u0022,\u0022groups\u0022:{\u0022x\u0022:\u0022\u0022,\u0022y\u0022:\u0022a\u0022},\u0022indices\u0022:[[1,3],[1,1],[1,2]],\u0022indicesGroups\u0022:{\u0022x\u0022:[1,1],\u0022y\u0022:[1,2]},\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<x>)(?<y>a)\u005c\u005ck<x>\u005c\u005ck<y>\u0022}"
        "(?<x>ab)(?<y>\u005ck<x>c)\u005ck<y>" flags="d" input="qababcabc" "{\u0022matches\u0022:[\u0022ababcabc\u0022,\u0022ab\u0022,\u0022abc\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022qababcabc\u0022,\u0022groups\u0022:{\u0022x\u0022:\u0022ab\u0022,\u0022y\u0022:\u0022abc\u0022},\u0022indices\u0022:[[1,9],[1,3],[3,6]],\u0022indicesGroups\u0022:{\u0022x\u0022:[1,3],\u0022y\u0022:[3,6]},\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<x>ab)(?<y>\u005c\u005ck<x>c)\u005c\u005ck<y>\u0022}"
        "(?<x>a)\u005ck<x>0" flags="d" input="qaa0" "{\u0022matches\u0022:[\u0022aa0\u0022,\u0022a\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022qaa0\u0022,\u0022groups\u0022:{\u0022x\u0022:\u0022a\u0022},\u0022indices\u0022:[[1,4],[1,2]],\u0022indicesGroups\u0022:{\u0022x\u0022:[1,2]},\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<x>a)\u005c\u005ck<x>0\u0022}"
        "(?<x>a)\u005ck<x>12" flags="d" input="qaa12" "{\u0022matches\u0022:[\u0022aa12\u0022,\u0022a\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022qaa12\u0022,\u0022groups\u0022:{\u0022x\u0022:\u0022a\u0022},\u0022indices\u0022:[[1,5],[1,2]],\u0022indicesGroups\u0022:{\u0022x\u0022:[1,2]},\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<x>a)\u005c\u005ck<x>12\u0022}"
        "(?<x>a)\u005ck<x>\u005ck<x>" flags="dg" input="qaaa" "{\u0022matches\u0022:[\u0022aaa\u0022,\u0022a\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022qaaa\u0022,\u0022groups\u0022:{\u0022x\u0022:\u0022a\u0022},\u0022indices\u0022:[[1,4],[1,2]],\u0022indicesGroups\u0022:{\u0022x\u0022:[1,2]},\u0022lastIndex\u0022:4,\u0022source\u0022:\u0022(?<x>a)\u005c\u005ck<x>\u005c\u005ck<x>\u0022}"
        "(?<\u005cu0078>a)\u005ck<x>" flags="d" input="qaa" "{\u0022matches\u0022:[\u0022aa\u0022,\u0022a\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022qaa\u0022,\u0022groups\u0022:{\u0022x\u0022:\u0022a\u0022},\u0022indices\u0022:[[1,3],[1,2]],\u0022indicesGroups\u0022:{\u0022x\u0022:[1,2]},\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<\u005c\u005cu0078>a)\u005c\u005ck<x>\u0022}"
        "(?<x>a)\u005ck<\u005cu{78}>" flags="d" input="qaa" "{\u0022matches\u0022:[\u0022aa\u0022,\u0022a\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022qaa\u0022,\u0022groups\u0022:{\u0022x\u0022:\u0022a\u0022},\u0022indices\u0022:[[1,3],[1,2]],\u0022indicesGroups\u0022:{\u0022x\u0022:[1,2]},\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<x>a)\u005c\u005ck<\u005c\u005cu{78}>\u0022}"
        "(?<\ud801\udc00>a)\u005ck<\u005cu{10400}>" flags="d" input="qaa" "{\u0022matches\u0022:[\u0022aa\u0022,\u0022a\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022qaa\u0022,\u0022groups\u0022:{\u0022\ud801\udc00\u0022:\u0022a\u0022},\u0022indices\u0022:[[1,3],[1,2]],\u0022indicesGroups\u0022:{\u0022\ud801\udc00\u0022:[1,2]},\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<\ud801\udc00>a)\u005c\u005ck<\u005c\u005cu{10400}>\u0022}"
        "(?<__proto__>a)\u005ck<__proto__>" flags="d" input="qaa" "{\u0022matches\u0022:[\u0022aa\u0022,\u0022a\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022qaa\u0022,\u0022groups\u0022:{\u0022__proto__\u0022:\u0022a\u0022},\u0022indices\u0022:[[1,3],[1,2]],\u0022indicesGroups\u0022:{\u0022__proto__\u0022:[1,2]},\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<__proto__>a)\u005c\u005ck<__proto__>\u0022}"
        "(?<\u00b5>a)\u005ck<\u00b5>" flags="di" input="qAa" "{\u0022matches\u0022:[\u0022Aa\u0022,\u0022A\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022qAa\u0022,\u0022groups\u0022:{\u0022\u00b5\u0022:\u0022A\u0022},\u0022indices\u0022:[[1,3],[1,2]],\u0022indicesGroups\u0022:{\u0022\u00b5\u0022:[1,2]},\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<\u00b5>a)\u005c\u005ck<\u00b5>\u0022}"
        "(?<x>\u00b5)\u005ck<x>" flags="di" input="q\u039c\u00b5" "{\u0022matches\u0022:[\u0022\u039c\u00b5\u0022,\u0022\u039c\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022q\u039c\u00b5\u0022,\u0022groups\u0022:{\u0022x\u0022:\u0022\u039c\u0022},\u0022indices\u0022:[[1,3],[1,2]],\u0022indicesGroups\u0022:{\u0022x\u0022:[1,2]},\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<x>\u00b5)\u005c\u005ck<x>\u0022}"
        "(?<x>\u017f)\u005ck<x>" flags="di" input="\u017fS" "{\u0022match\u0022:null,\u0022lastIndex\u0022:0}"
        "(?<x>\u005cuD800\u005cuDC00)\u005ck<x>" flags="d" input="\ud800\udc00\ud800\udc00" "{\u0022matches\u0022:[\u0022\ud800\udc00\ud800\udc00\u0022,\u0022\ud800\udc00\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022\ud800\udc00\ud800\udc00\u0022,\u0022groups\u0022:{\u0022x\u0022:\u0022\ud800\udc00\u0022},\u0022indices\u0022:[[0,4],[0,2]],\u0022indicesGroups\u0022:{\u0022x\u0022:[0,2]},\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<x>\u005c\u005cuD800\u005c\u005cuDC00)\u005c\u005ck<x>\u0022}"
        "(?<x>a)\u005c1\u005ck<x>" flags="d" input="qaaa" "{\u0022matches\u0022:[\u0022aaa\u0022,\u0022a\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022qaaa\u0022,\u0022groups\u0022:{\u0022x\u0022:\u0022a\u0022},\u0022indices\u0022:[[1,4],[1,2]],\u0022indicesGroups\u0022:{\u0022x\u0022:[1,2]},\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<x>a)\u005c\u005c1\u005c\u005ck<x>\u0022}"
        "()()()()()()()()()(?<x>a)\u005ck<x>0" flags="d" input="qaa0" "{\u0022matches\u0022:[\u0022aa0\u0022,\u0022\u0022,\u0022\u0022,\u0022\u0022,\u0022\u0022,\u0022\u0022,\u0022\u0022,\u0022\u0022,\u0022\u0022,\u0022\u0022,\u0022a\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022qaa0\u0022,\u0022groups\u0022:{\u0022x\u0022:\u0022a\u0022},\u0022indices\u0022:[[1,4],[1,1],[1,1],[1,1],[1,1],[1,1],[1,1],[1,1],[1,1],[1,1],[1,2]],\u0022indicesGroups\u0022:{\u0022x\u0022:[1,2]},\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022()()()()()()()()()(?<x>a)\u005c\u005ck<x>0\u0022}"
        "(?<x>)\u005ck<x>" flags="dg" input="q" "{\u0022matches\u0022:[\u0022\u0022,\u0022\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022q\u0022,\u0022groups\u0022:{\u0022x\u0022:\u0022\u0022},\u0022indices\u0022:[[0,0],[0,0]],\u0022indicesGroups\u0022:{\u0022x\u0022:[0,0]},\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<x>)\u005c\u005ck<x>\u0022}"
        "(?<x>a)\u005ck<x>" flags="dy" input="qaa" "{\u0022match\u0022:null,\u0022lastIndex\u0022:0}"
        "(?<x>a)\u005ck<x>" flags="d" input="qaba" "{\u0022match\u0022:null,\u0022lastIndex\u0022:0}"
        "#);
}

#[test]
fn decoded_names_original_ranges_and_reference_digit_boundaries_are_preserved() {
    assert_true(
        r#"
            let a = /(?<x>a)\k<x>0/d.exec('qaa0');
            a[0] === 'aa0' && a[1] === 'a' && a.groups.x === 'a' && a.indices.groups.x === a.indices[1] &&
                a.indices[0][1] === 4
        "#,
    );
    assert_true(
        r#"
            let a = /()()()()()()()()()(?<x>a)\k<x>0/d.exec('qaa0');
            a.length === 11 && a[10] === 'a' && a.groups.x === 'a' && a.indices.groups.x === a.indices[
                10] && a[0] === 'aa0'
        "#,
    );
    assert_true(
        r#"
            let a = /(?<outer>a(?<x>\k<outer>b)\k<x>)\k<outer>/d.exec('qabbabb');
            a[0] === 'abbabb' && a.groups.outer === 'abb' && a.groups.x === 'b' && a.indices.groups
                .outer === a.indices[1] && a.indices.groups.x === a.indices[2] && a.indices[2][0] === 2 &&
                a.indices[2][1] === 3
        "#,
    );
    assert_true(
        r#"
            let a = /\k<x>(?<x>a)/d.exec('qa'),
                b = /(?<x>a\k<x>)/d.exec('qa');
            a[0] === 'a' && a.groups.x === 'a' && b[0] === 'a' && b.groups.x === 'a'
        "#,
    );
    assert_true(
        r#"
            let a = /(?<\u0078>a)\k<\u{78}>/di.exec('qAa');
            a[0] === 'Aa' && a.groups.x === 'A' && Object.keys(a.groups).join(',') === 'x' && Object
                .getPrototypeOf(a.groups) === null
        "#,
    );
    assert_true(
        r#"
            let a = /(?<__proto__>a)\k<__proto__>/d.exec('aa');
            Object.getPrototypeOf(a.groups) === null && a.groups.__proto__ === 'a' && a.indices.groups
                .__proto__ === a.indices[1]
        "#,
    );
}

#[test]
fn native_consumers_and_copies_share_reference_results_and_original_source() {
    assert_true(
        r"'qaa'.replace(/(?<x>a)\k<x>/,'<$<x>>')==='q<a>'&&'qaa'.split(/(?<x>a)\k<x>/).join(',')==='q,a,'&&'qaa'.search(/(?<x>a)\k<x>/)===1",
    );
    assert_true(
        r#"
            let seen;
            'qAa'.replace(/(?<x>a)\k<x>/i, (whole, x, index, input, groups) => {
                seen = [whole, x, index, input, groups.x];
                return 'z'
            });
            seen.join(',') === 'Aa,A,1,qAa,A'
        "#,
    );
    assert_true(
        r#"
            let r = /(?<x>a)\k<x>/dg,
                a = [...'qaa aa'.matchAll(r)];
            a.length === 2 && a[0].index === 1 && a[1].index === 4 && a[1].indices.groups.x === a[1]
                .indices[1] && r.lastIndex === 0
        "#,
    );
    assert_true(
        r"let r=/(?<x>)\k<x>/g,a=r.exec('q');a[0]===''&&r.lastIndex===0&&[...'q'.matchAll(r)].length===2",
    );
    assert_true(
        r#"
            let r = /(?<\u0078>a)\k<\u{78}>/dg;
            Object.defineProperty(r, 'source', {
                get() {
                    throw 7
                }
            });
            let copy = new RegExp(r);
            copy.source === '(?<\\u0078>a)\\k<\\u{78}>' && copy.exec('aa').groups.x === 'a' && copy
                .lastIndex === 2 && r.lastIndex === 0
        "#,
    );
    assert_true(
        r#"
            let r = /(?<x>a)\k<x>/gy;
            r.lastIndex = 1;
            let a = r.exec('qaa');
            a.index === 1 && r.lastIndex === 3 && r.exec('qaa') === null && r.lastIndex === 0
        "#,
    );
}

#[test]
fn explicit_search_work_aborts_before_last_index_and_language_handlers() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval(r"let r=/(?<x>a)\k<x>/g,flag=0,text='ab'.repeat(5000);r.lastIndex=1")
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
        r"/(?:(?:(?:(?<x>a)(?:\k<x>)+){2})|){2,3}/.test('aa')",
        r"/(?:(?:(?:(?<x>a)|(?<x>b)(?:\k<x>)+){2})|){2,3}/.test('bb')",
        r"/(?:(?:(?:(?<x>[ab])(?:\k<x>)+){2})|){2,3}/.test('aa')",
        r"/(?:(?:(?:^(?<x>a)(?:\k<x>)+){2})|){2,3}/.test('aa')",
        r"/(?<x>a)\k<x>/u.test('aa')",
    ] {
        crate::common::regexp_oracle::assert_matches_reference(&mut Realm::default(), source);
    }
    assert_true(r"/(?<x>a)\k<x>/.exec('ab')===null");
}
