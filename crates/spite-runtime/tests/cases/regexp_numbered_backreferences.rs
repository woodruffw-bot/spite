//! Ordinary numbered references retain input captures and host-abort semantics.

use spite_core::JsString;
use spite_runtime::{Error, Limits, Realm, Value};
use std::fmt::Write;

use crate::common::assert_true;

#[test]
fn numbered_reference_result_snapshot() {
    let mut rows = String::new();
    for (source, flags, text) in [
        (r"(a)\1", "d", "qaa"),
        (r"(a)\1", "di", "qAa"),
        (r"(ab)(\1c)\2", "d", "qabab cabc"),
        (r"(ab)(\1c)\2", "d", "qababcabc"),
        (r"\1(a)", "d", "qa"),
        (r"(a\1)", "d", "qa"),
        (r"(a(\1b)\2)\1", "d", "qabbabb"),
        (r"()(a)\1\2", "d", "qaa"),
        (r"(?:q(a))\1", "d", "qaa"),
        (r"\2(a)(b)\1\2", "di", "qABab"),
        (r"()\1", "dg", "q"),
        (r"(a)\1", "dg", "qaa"),
        (r"(a)\1", "dy", "qaa"),
        (r"(a)\1", "dy", "aa"),
        (r"(a)\1", "d", "qaba"),
        (r"(µ)\1", "di", "qΜµ"),
        (r"(ſ)\1", "di", "ſS"),
        (r"(ſ)\1", "di", "ſſ"),
        (r"(\uD800\uDC00)\1", "d", "𐀀𐀀"),
        (r"(a)\x31\1", "d", "a1a"),
        (r"(a)\\1\1", "d", r"a\1a"),
        (r"()()()()()()()()()(a)\10", "d", "aa"),
        (r"(?<x>a)\1", "d", "qaa"),
        (r"\1(?<x>a)", "d", "qa"),
        (r"(?<outer>a(?<x>\1b)\2)\1", "d", "qabbabb"),
    ] {
        let source = JsString::from(source);
        let flags = JsString::from(flags);
        let text = JsString::from(text);
        let program = format!(
            "let r=new RegExp({source:?},{flags:?}),a=r.exec({text:?});JSON.stringify(a===null?{{match:null,lastIndex:r.lastIndex}}:{{matches:[...a],index:a.index,input:a.input,groups:a.groups,indices:a.indices,indicesGroups:a.indices.groups,lastIndex:r.lastIndex}})"
        );
        let Value::String(value) = Realm::default().eval(&program).unwrap() else {
            panic!("expected JSON");
        };
        writeln!(rows, "{source:?} flags={flags:?} input={text:?} {value:?}").unwrap();
    }
    insta::assert_snapshot!(rows, @r#"
        "(a)\u005c1" flags="d" input="qaa" "{\u0022matches\u0022:[\u0022aa\u0022,\u0022a\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022qaa\u0022,\u0022indices\u0022:[[1,3],[1,2]],\u0022lastIndex\u0022:0}"
        "(a)\u005c1" flags="di" input="qAa" "{\u0022matches\u0022:[\u0022Aa\u0022,\u0022A\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022qAa\u0022,\u0022indices\u0022:[[1,3],[1,2]],\u0022lastIndex\u0022:0}"
        "(ab)(\u005c1c)\u005c2" flags="d" input="qabab cabc" "{\u0022match\u0022:null,\u0022lastIndex\u0022:0}"
        "(ab)(\u005c1c)\u005c2" flags="d" input="qababcabc" "{\u0022matches\u0022:[\u0022ababcabc\u0022,\u0022ab\u0022,\u0022abc\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022qababcabc\u0022,\u0022indices\u0022:[[1,9],[1,3],[3,6]],\u0022lastIndex\u0022:0}"
        "\u005c1(a)" flags="d" input="qa" "{\u0022matches\u0022:[\u0022a\u0022,\u0022a\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022qa\u0022,\u0022indices\u0022:[[1,2],[1,2]],\u0022lastIndex\u0022:0}"
        "(a\u005c1)" flags="d" input="qa" "{\u0022matches\u0022:[\u0022a\u0022,\u0022a\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022qa\u0022,\u0022indices\u0022:[[1,2],[1,2]],\u0022lastIndex\u0022:0}"
        "(a(\u005c1b)\u005c2)\u005c1" flags="d" input="qabbabb" "{\u0022matches\u0022:[\u0022abbabb\u0022,\u0022abb\u0022,\u0022b\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022qabbabb\u0022,\u0022indices\u0022:[[1,7],[1,4],[2,3]],\u0022lastIndex\u0022:0}"
        "()(a)\u005c1\u005c2" flags="d" input="qaa" "{\u0022matches\u0022:[\u0022aa\u0022,\u0022\u0022,\u0022a\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022qaa\u0022,\u0022indices\u0022:[[1,3],[1,1],[1,2]],\u0022lastIndex\u0022:0}"
        "(?:q(a))\u005c1" flags="d" input="qaa" "{\u0022matches\u0022:[\u0022qaa\u0022,\u0022a\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022qaa\u0022,\u0022indices\u0022:[[0,3],[1,2]],\u0022lastIndex\u0022:0}"
        "\u005c2(a)(b)\u005c1\u005c2" flags="di" input="qABab" "{\u0022matches\u0022:[\u0022ABab\u0022,\u0022A\u0022,\u0022B\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022qABab\u0022,\u0022indices\u0022:[[1,5],[1,2],[2,3]],\u0022lastIndex\u0022:0}"
        "()\u005c1" flags="dg" input="q" "{\u0022matches\u0022:[\u0022\u0022,\u0022\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022q\u0022,\u0022indices\u0022:[[0,0],[0,0]],\u0022lastIndex\u0022:0}"
        "(a)\u005c1" flags="dg" input="qaa" "{\u0022matches\u0022:[\u0022aa\u0022,\u0022a\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022qaa\u0022,\u0022indices\u0022:[[1,3],[1,2]],\u0022lastIndex\u0022:3}"
        "(a)\u005c1" flags="dy" input="qaa" "{\u0022match\u0022:null,\u0022lastIndex\u0022:0}"
        "(a)\u005c1" flags="dy" input="aa" "{\u0022matches\u0022:[\u0022aa\u0022,\u0022a\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022aa\u0022,\u0022indices\u0022:[[0,2],[0,1]],\u0022lastIndex\u0022:2}"
        "(a)\u005c1" flags="d" input="qaba" "{\u0022match\u0022:null,\u0022lastIndex\u0022:0}"
        "(\u00b5)\u005c1" flags="di" input="q\u039c\u00b5" "{\u0022matches\u0022:[\u0022\u039c\u00b5\u0022,\u0022\u039c\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022q\u039c\u00b5\u0022,\u0022indices\u0022:[[1,3],[1,2]],\u0022lastIndex\u0022:0}"
        "(\u017f)\u005c1" flags="di" input="\u017fS" "{\u0022match\u0022:null,\u0022lastIndex\u0022:0}"
        "(\u017f)\u005c1" flags="di" input="\u017f\u017f" "{\u0022matches\u0022:[\u0022\u017f\u017f\u0022,\u0022\u017f\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022\u017f\u017f\u0022,\u0022indices\u0022:[[0,2],[0,1]],\u0022lastIndex\u0022:0}"
        "(\u005cuD800\u005cuDC00)\u005c1" flags="d" input="\ud800\udc00\ud800\udc00" "{\u0022matches\u0022:[\u0022\ud800\udc00\ud800\udc00\u0022,\u0022\ud800\udc00\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022\ud800\udc00\ud800\udc00\u0022,\u0022indices\u0022:[[0,4],[0,2]],\u0022lastIndex\u0022:0}"
        "(a)\u005cx31\u005c1" flags="d" input="a1a" "{\u0022matches\u0022:[\u0022a1a\u0022,\u0022a\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022a1a\u0022,\u0022indices\u0022:[[0,3],[0,1]],\u0022lastIndex\u0022:0}"
        "(a)\u005c\u005c1\u005c1" flags="d" input="a\u005c1a" "{\u0022matches\u0022:[\u0022a\u005c\u005c1a\u0022,\u0022a\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022a\u005c\u005c1a\u0022,\u0022indices\u0022:[[0,4],[0,1]],\u0022lastIndex\u0022:0}"
        "()()()()()()()()()(a)\u005c10" flags="d" input="aa" "{\u0022matches\u0022:[\u0022aa\u0022,\u0022\u0022,\u0022\u0022,\u0022\u0022,\u0022\u0022,\u0022\u0022,\u0022\u0022,\u0022\u0022,\u0022\u0022,\u0022\u0022,\u0022a\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022aa\u0022,\u0022indices\u0022:[[0,2],[0,0],[0,0],[0,0],[0,0],[0,0],[0,0],[0,0],[0,0],[0,0],[0,1]],\u0022lastIndex\u0022:0}"
        "(?<x>a)\u005c1" flags="d" input="qaa" "{\u0022matches\u0022:[\u0022aa\u0022,\u0022a\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022qaa\u0022,\u0022groups\u0022:{\u0022x\u0022:\u0022a\u0022},\u0022indices\u0022:[[1,3],[1,2]],\u0022indicesGroups\u0022:{\u0022x\u0022:[1,2]},\u0022lastIndex\u0022:0}"
        "\u005c1(?<x>a)" flags="d" input="qa" "{\u0022matches\u0022:[\u0022a\u0022,\u0022a\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022qa\u0022,\u0022groups\u0022:{\u0022x\u0022:\u0022a\u0022},\u0022indices\u0022:[[1,2],[1,2]],\u0022indicesGroups\u0022:{\u0022x\u0022:[1,2]},\u0022lastIndex\u0022:0}"
        "(?<outer>a(?<x>\u005c1b)\u005c2)\u005c1" flags="d" input="qabbabb" "{\u0022matches\u0022:[\u0022abbabb\u0022,\u0022abb\u0022,\u0022b\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022qabbabb\u0022,\u0022groups\u0022:{\u0022outer\u0022:\u0022abb\u0022,\u0022x\u0022:\u0022b\u0022},\u0022indices\u0022:[[1,7],[1,4],[2,3]],\u0022indicesGroups\u0022:{\u0022outer\u0022:[1,4],\u0022x\u0022:[2,3]},\u0022lastIndex\u0022:0}"
        "#);
}

#[test]
fn native_results_keep_captures_indices_aliases_and_case_from_the_input() {
    assert_true(
        r#"
            for(const flags of ['u','ui','v','vi']) {
                for(const suffix of ['\uDC00','\uD834','A','']) {
                    const a=new RegExp('foo(.+)bar\\1',flags).exec('foo\uD834bar\uD834'+suffix);
                    if(suffix==='\uDC00' ? a!==null : a===null||a[1]!=='\uD834')throw 1;
                }
                if(new RegExp('(?<=\\1#(.+))$',flags).test('\uD834\uDC00#\uDC00'))throw 2;
                if(!new RegExp('(?<=\\1#(.+))$',flags).test('\uD834\uDC00#\uD834\uDC00'))throw 3;
                if(new RegExp('^(?<x>.+)#\\k<x>',flags).test('\uD834#\uD834\uDC00'))throw 4;
            }
            new RegExp('^(.+)#\\1').test('\uD834#\uD834\uDC00')&&
                new RegExp('(?<=\\1#(.+))$').test('\uD834\uDC00#\uDC00')
        "#,
    );
    assert_true(
        r#"
            let a = /(?<x>a)\1/di.exec('qAa');
            a[0] === 'Aa' && a[1] === 'A' && a.groups.x === 'A' && a.indices[0][0] === 1 && a.indices[0][
                1
            ] === 3 && a.indices.groups.x === a.indices[1] && a.indices[1][0] === 1 && a.indices[1][
                1
            ] === 2
        "#,
    );
    assert_true(
        r#"
            let a = /(a(\1b)\2)\1/d.exec('qabbabb');
            a[0] === 'abbabb' && a[1] === 'abb' && a[2] === 'b' && a.indices[1][0] === 1 && a.indices[1][
                1
            ] === 4 && a.indices[2][0] === 2 && a.indices[2][1] === 3
        "#,
    );
    assert_true(
        r#"
            let a = /\1(a)/d.exec('qa'),
                b = /(a\1)/d.exec('qa'),
                c = /()\1/d.exec('q');
            a[0] === 'a' && a[1] === 'a' && b[0] === 'a' && b[1] === 'a' && c[0] === '' && c[1] === '' && c
                .indices[1][0] === 0 && c.indices[1][1] === 0
        "#,
    );
    assert_true(
        r#"
            let a = /()()()()()()()()()(a)\10/d.exec('qaa');
            a.length === 11 && a[10] === 'a' && a.indices[10][0] === 1 && a.indices[10][1] === 2 && a[
                0] === 'aa'
        "#,
    );
}

#[test]
fn stateful_exec_and_generic_consumers_use_the_same_reference_program() {
    assert_true(
        r#"
            let r = /(a)\1/dg,
                a = r.exec('qaa aa'),
                b = r.exec('qaa aa'),
                c = r.exec('qaa aa');
            a.index === 1 && b.index === 4 && c === null && r.lastIndex === 0
        "#,
    );
    assert_true(
        r#"
            let r = /(a)\1/dy;
            r.lastIndex = 1;
            let a = r.exec('qaa');
            a.index === 1 && r.lastIndex === 3 && r.exec('qaa') === null && r.lastIndex === 0
        "#,
    );
    assert_true(
        r"let r=/()\1/g,a=r.exec('q');a[0]===''&&r.lastIndex===0&&[...'q'.matchAll(r)].length===2",
    );
    assert_true(
        r"'qaa'.replace(/(?<x>a)\1/,'<$<x>>')==='q<a>'&&'qaa'.split(/(a)\1/).join(',')==='q,a,'&&'qaa'.search(/(a)\1/)===1",
    );
    assert_true(
        r#"
            let seen;
            'qAa'.replace(/(?<x>a)\1/i, (whole, x, index, input, groups) => {
                seen = [whole, x, index, input, groups.x];
                return 'z'
            });
            seen.join(',') === 'Aa,A,1,qAa,A'
        "#,
    );
    assert_true(
        r#"
            let r = /(?<x>a)\1/dg;
            Object.defineProperty(r, 'source', {
                get() {
                    throw 7
                }
            });
            let copy = new RegExp(r);
            copy.source === '(?<x>a)\\1' && copy.exec('aa').groups.x === 'a' && copy.lastIndex === 2 && r
                .lastIndex === 0
        "#,
    );
}

#[test]
fn explicit_search_work_aborts_before_last_index_writes_and_language_handlers() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval(r"let r=/(a)\1/g,flag=0,text='ab'.repeat(5000);r.lastIndex=1")
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
        r"/(?:(?:(?:(a)(?:\1)+){2})|){2,3}/.test('aa')",
        r"/(?:(?:(?:(a|b)(?:\1)+){2})|){2,3}/.test('aa')",
        r"/(?:(?:(?:([ab])(?:\1)+){2})|){2,3}/.test('aa')",
        r"/(?:(?:(?:^(a)(?:\1)+){2})|){2,3}/.test('aa')",
        r"/(?:(?:(?:(?<x>a)(?:\k<x>)+){2})|){2,3}/.test('aa')",
        r"/(a)\1/u.test('aa')",
    ] {
        crate::common::regexp_oracle::assert_matches_reference(&mut Realm::default(), source);
    }
    assert_true(r"/(a)\1/.exec('ab')===null");
}
