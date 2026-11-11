//! Nonempty case-sensitive BMP literals preserve UnicodeSets matching boundaries.

use spite_core::JsString;
use spite_runtime::{Error, Limits, Realm, Value};
use std::fmt::Write;

use crate::common::assert_true;

#[test]
fn unicode_sets_bmp_literal_results_snapshot() {
    let mut rows = String::new();
    for (pattern, flags, input, initial) in [
        ("a", "dv", "ba", 9),
        ("a", "dvg", "😀ab", 0),
        ("a", "dvg", "😀ab", 1),
        ("a", "dvy", "😀ab", 1),
        ("a", "dvy", "😀ab", 2),
        ("a", "dvg", "a😀b", 1),
        ("ab", "dvg", "😀ab😀ab", 1),
        ("ab", "dvg", "😀ab😀ab", 4),
        ("ab", "dvg", "😀ab😀ab", 5),
        ("ab", "dvy", "😀ab😀ab", 5),
        ("ab", "dvy", "😀ab😀ab", 6),
        ("(a())b", "dvg", "😀ab", 1),
        ("()(ab)()", "dvy", "😀ab", 2),
        ("(?<x>a(?<y>()))b", "dvg", "😀ab", 1),
        ("(?<π>a)b", "dv", "😀ab", 0),
        ("((a)(?:b))(c)", "dvms", "😀abc", 0),
        (r"\u0061\x62", "dvg", "😀ab", 1),
        (r"\(a\)", "dv", "😀(a)", 0),
        (r"\0", "dvg", "😀\0x", 1),
        (r"\cA", "dvg", "😀\u{1}x", 1),
        (r"\x7f", "dvg", "😀\u{7f}x", 1),
        (r"\n", "dvg", "😀\nx", 1),
        ("a", "dvg", "a", 2),
        ("a", "dvy", "b", 0),
        ("é", "dvg", "😀é", 1),
        ("(σ())", "dvg", "😀σΣ", 1),
        ("K", "dv", "kK", 0),
        (r"\u00e9", "dvy", "😀é", 2),
        (r"\u2028", "dv", "😀\u{2028}", 0),
        ("(?:(µ))", "dv", "Μµ", 0),
    ] {
        let pattern = JsString::from(pattern);
        let input = JsString::from(input);
        let script = format!(
            "let r=new RegExp({pattern:?},'{flags}');r.lastIndex={initial};let m=r.exec({input:?});JSON.stringify(m===null?{{match:null,lastIndex:r.lastIndex}}:{{matches:[...m],index:m.index,input:m.input,groups:m.groups,indices:m.indices,indicesGroups:m.indices.groups,lastIndex:r.lastIndex,source:r.source}})"
        );
        let Value::String(result) = Realm::default().eval(&script).unwrap() else {
            panic!("expected JSON: {script}");
        };
        writeln!(
            rows,
            "{pattern:?} flags={flags:?} input={input:?} lastIndex={initial} {result:?}"
        )
        .unwrap();
    }
    insta::assert_snapshot!(rows, @r#"
        "a" flags="dv" input="ba" lastIndex=9 "{\u0022matches\u0022:[\u0022a\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022ba\u0022,\u0022indices\u0022:[[1,2]],\u0022lastIndex\u0022:9,\u0022source\u0022:\u0022a\u0022}"
        "a" flags="dvg" input="\ud83d\ude00ab" lastIndex=0 "{\u0022matches\u0022:[\u0022a\u0022],\u0022index\u0022:2,\u0022input\u0022:\u0022\ud83d\ude00ab\u0022,\u0022indices\u0022:[[2,3]],\u0022lastIndex\u0022:3,\u0022source\u0022:\u0022a\u0022}"
        "a" flags="dvg" input="\ud83d\ude00ab" lastIndex=1 "{\u0022matches\u0022:[\u0022a\u0022],\u0022index\u0022:2,\u0022input\u0022:\u0022\ud83d\ude00ab\u0022,\u0022indices\u0022:[[2,3]],\u0022lastIndex\u0022:3,\u0022source\u0022:\u0022a\u0022}"
        "a" flags="dvy" input="\ud83d\ude00ab" lastIndex=1 "{\u0022match\u0022:null,\u0022lastIndex\u0022:0}"
        "a" flags="dvy" input="\ud83d\ude00ab" lastIndex=2 "{\u0022matches\u0022:[\u0022a\u0022],\u0022index\u0022:2,\u0022input\u0022:\u0022\ud83d\ude00ab\u0022,\u0022indices\u0022:[[2,3]],\u0022lastIndex\u0022:3,\u0022source\u0022:\u0022a\u0022}"
        "a" flags="dvg" input="a\ud83d\ude00b" lastIndex=1 "{\u0022match\u0022:null,\u0022lastIndex\u0022:0}"
        "ab" flags="dvg" input="\ud83d\ude00ab\ud83d\ude00ab" lastIndex=1 "{\u0022matches\u0022:[\u0022ab\u0022],\u0022index\u0022:2,\u0022input\u0022:\u0022\ud83d\ude00ab\ud83d\ude00ab\u0022,\u0022indices\u0022:[[2,4]],\u0022lastIndex\u0022:4,\u0022source\u0022:\u0022ab\u0022}"
        "ab" flags="dvg" input="\ud83d\ude00ab\ud83d\ude00ab" lastIndex=4 "{\u0022matches\u0022:[\u0022ab\u0022],\u0022index\u0022:6,\u0022input\u0022:\u0022\ud83d\ude00ab\ud83d\ude00ab\u0022,\u0022indices\u0022:[[6,8]],\u0022lastIndex\u0022:8,\u0022source\u0022:\u0022ab\u0022}"
        "ab" flags="dvg" input="\ud83d\ude00ab\ud83d\ude00ab" lastIndex=5 "{\u0022matches\u0022:[\u0022ab\u0022],\u0022index\u0022:6,\u0022input\u0022:\u0022\ud83d\ude00ab\ud83d\ude00ab\u0022,\u0022indices\u0022:[[6,8]],\u0022lastIndex\u0022:8,\u0022source\u0022:\u0022ab\u0022}"
        "ab" flags="dvy" input="\ud83d\ude00ab\ud83d\ude00ab" lastIndex=5 "{\u0022match\u0022:null,\u0022lastIndex\u0022:0}"
        "ab" flags="dvy" input="\ud83d\ude00ab\ud83d\ude00ab" lastIndex=6 "{\u0022matches\u0022:[\u0022ab\u0022],\u0022index\u0022:6,\u0022input\u0022:\u0022\ud83d\ude00ab\ud83d\ude00ab\u0022,\u0022indices\u0022:[[6,8]],\u0022lastIndex\u0022:8,\u0022source\u0022:\u0022ab\u0022}"
        "(a())b" flags="dvg" input="\ud83d\ude00ab" lastIndex=1 "{\u0022matches\u0022:[\u0022ab\u0022,\u0022a\u0022,\u0022\u0022],\u0022index\u0022:2,\u0022input\u0022:\u0022\ud83d\ude00ab\u0022,\u0022indices\u0022:[[2,4],[2,3],[3,3]],\u0022lastIndex\u0022:4,\u0022source\u0022:\u0022(a())b\u0022}"
        "()(ab)()" flags="dvy" input="\ud83d\ude00ab" lastIndex=2 "{\u0022matches\u0022:[\u0022ab\u0022,\u0022\u0022,\u0022ab\u0022,\u0022\u0022],\u0022index\u0022:2,\u0022input\u0022:\u0022\ud83d\ude00ab\u0022,\u0022indices\u0022:[[2,4],[2,2],[2,4],[4,4]],\u0022lastIndex\u0022:4,\u0022source\u0022:\u0022()(ab)()\u0022}"
        "(?<x>a(?<y>()))b" flags="dvg" input="\ud83d\ude00ab" lastIndex=1 "{\u0022matches\u0022:[\u0022ab\u0022,\u0022a\u0022,\u0022\u0022,\u0022\u0022],\u0022index\u0022:2,\u0022input\u0022:\u0022\ud83d\ude00ab\u0022,\u0022groups\u0022:{\u0022x\u0022:\u0022a\u0022,\u0022y\u0022:\u0022\u0022},\u0022indices\u0022:[[2,4],[2,3],[3,3],[3,3]],\u0022indicesGroups\u0022:{\u0022x\u0022:[2,3],\u0022y\u0022:[3,3]},\u0022lastIndex\u0022:4,\u0022source\u0022:\u0022(?<x>a(?<y>()))b\u0022}"
        "(?<\u03c0>a)b" flags="dv" input="\ud83d\ude00ab" lastIndex=0 "{\u0022matches\u0022:[\u0022ab\u0022,\u0022a\u0022],\u0022index\u0022:2,\u0022input\u0022:\u0022\ud83d\ude00ab\u0022,\u0022groups\u0022:{\u0022\u03c0\u0022:\u0022a\u0022},\u0022indices\u0022:[[2,4],[2,3]],\u0022indicesGroups\u0022:{\u0022\u03c0\u0022:[2,3]},\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<\u03c0>a)b\u0022}"
        "((a)(?:b))(c)" flags="dvms" input="\ud83d\ude00abc" lastIndex=0 "{\u0022matches\u0022:[\u0022abc\u0022,\u0022ab\u0022,\u0022a\u0022,\u0022c\u0022],\u0022index\u0022:2,\u0022input\u0022:\u0022\ud83d\ude00abc\u0022,\u0022indices\u0022:[[2,5],[2,4],[2,3],[4,5]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022((a)(?:b))(c)\u0022}"
        "\u005cu0061\u005cx62" flags="dvg" input="\ud83d\ude00ab" lastIndex=1 "{\u0022matches\u0022:[\u0022ab\u0022],\u0022index\u0022:2,\u0022input\u0022:\u0022\ud83d\ude00ab\u0022,\u0022indices\u0022:[[2,4]],\u0022lastIndex\u0022:4,\u0022source\u0022:\u0022\u005c\u005cu0061\u005c\u005cx62\u0022}"
        "\u005c(a\u005c)" flags="dv" input="\ud83d\ude00(a)" lastIndex=0 "{\u0022matches\u0022:[\u0022(a)\u0022],\u0022index\u0022:2,\u0022input\u0022:\u0022\ud83d\ude00(a)\u0022,\u0022indices\u0022:[[2,5]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022\u005c\u005c(a\u005c\u005c)\u0022}"
        "\u005c0" flags="dvg" input="\ud83d\ude00\u0000x" lastIndex=1 "{\u0022matches\u0022:[\u0022\u005cu0000\u0022],\u0022index\u0022:2,\u0022input\u0022:\u0022\ud83d\ude00\u005cu0000x\u0022,\u0022indices\u0022:[[2,3]],\u0022lastIndex\u0022:3,\u0022source\u0022:\u0022\u005c\u005c0\u0022}"
        "\u005ccA" flags="dvg" input="\ud83d\ude00\u0001x" lastIndex=1 "{\u0022matches\u0022:[\u0022\u005cu0001\u0022],\u0022index\u0022:2,\u0022input\u0022:\u0022\ud83d\ude00\u005cu0001x\u0022,\u0022indices\u0022:[[2,3]],\u0022lastIndex\u0022:3,\u0022source\u0022:\u0022\u005c\u005ccA\u0022}"
        "\u005cx7f" flags="dvg" input="\ud83d\ude00\u007fx" lastIndex=1 "{\u0022matches\u0022:[\u0022\u007f\u0022],\u0022index\u0022:2,\u0022input\u0022:\u0022\ud83d\ude00\u007fx\u0022,\u0022indices\u0022:[[2,3]],\u0022lastIndex\u0022:3,\u0022source\u0022:\u0022\u005c\u005cx7f\u0022}"
        "\u005cn" flags="dvg" input="\ud83d\ude00\u000ax" lastIndex=1 "{\u0022matches\u0022:[\u0022\u005cn\u0022],\u0022index\u0022:2,\u0022input\u0022:\u0022\ud83d\ude00\u005cnx\u0022,\u0022indices\u0022:[[2,3]],\u0022lastIndex\u0022:3,\u0022source\u0022:\u0022\u005c\u005cn\u0022}"
        "a" flags="dvg" input="a" lastIndex=2 "{\u0022match\u0022:null,\u0022lastIndex\u0022:0}"
        "a" flags="dvy" input="b" lastIndex=0 "{\u0022match\u0022:null,\u0022lastIndex\u0022:0}"
        "\u00e9" flags="dvg" input="\ud83d\ude00\u00e9" lastIndex=1 "{\u0022matches\u0022:[\u0022\u00e9\u0022],\u0022index\u0022:2,\u0022input\u0022:\u0022\ud83d\ude00\u00e9\u0022,\u0022indices\u0022:[[2,3]],\u0022lastIndex\u0022:3,\u0022source\u0022:\u0022\u00e9\u0022}"
        "(\u03c3())" flags="dvg" input="\ud83d\ude00\u03c3\u03a3" lastIndex=1 "{\u0022matches\u0022:[\u0022\u03c3\u0022,\u0022\u03c3\u0022,\u0022\u0022],\u0022index\u0022:2,\u0022input\u0022:\u0022\ud83d\ude00\u03c3\u03a3\u0022,\u0022indices\u0022:[[2,3],[2,3],[3,3]],\u0022lastIndex\u0022:3,\u0022source\u0022:\u0022(\u03c3())\u0022}"
        "\u212a" flags="dv" input="k\u212a" lastIndex=0 "{\u0022matches\u0022:[\u0022\u212a\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022k\u212a\u0022,\u0022indices\u0022:[[1,2]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022\u212a\u0022}"
        "\u005cu00e9" flags="dvy" input="\ud83d\ude00\u00e9" lastIndex=2 "{\u0022matches\u0022:[\u0022\u00e9\u0022],\u0022index\u0022:2,\u0022input\u0022:\u0022\ud83d\ude00\u00e9\u0022,\u0022indices\u0022:[[2,3]],\u0022lastIndex\u0022:3,\u0022source\u0022:\u0022\u005c\u005cu00e9\u0022}"
        "\u005cu2028" flags="dv" input="\ud83d\ude00\u2028" lastIndex=0 "{\u0022matches\u0022:[\u0022\u2028\u0022],\u0022index\u0022:2,\u0022input\u0022:\u0022\ud83d\ude00\u2028\u0022,\u0022indices\u0022:[[2,3]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022\u005c\u005cu2028\u0022}"
        "(?:(\u00b5))" flags="dv" input="\u039c\u00b5" lastIndex=0 "{\u0022matches\u0022:[\u0022\u00b5\u0022,\u0022\u00b5\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022\u039c\u00b5\u0022,\u0022indices\u0022:[[1,2],[1,2]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?:(\u00b5))\u0022}"
        "#);
}

#[test]
fn unicode_sets_bmp_names_lone_surrogates_and_matching_boundaries_are_exact() {
    assert_true(
        "let r=/a/v,c=new RegExp(r),u=new RegExp(r,'u');r.unicodeSets&&!r.unicode&&r.flags==='v'&&c.unicodeSets&&!c.unicode&&u.unicode&&!u.unicodeSets&&u.exec('a')[0]==='a'",
    );
    let mut rows = String::new();
    for source in [
        "/(a)/v.test('a')",
        "/a/v.exec('a')",
        "/(?<x>a)/v.test('a')",
        "/(?:a)/v.test('a')",
    ] {
        let result = Realm::default().eval(source).unwrap();
        if source.contains(".exec(") {
            assert!(matches!(result, Value::Object(_)), "{source}");
        } else {
            assert_eq!(result, Value::Boolean(true), "{source}");
        }
        let script = format!(
            "JSON.stringify((()=>{{let a={source};return typeof a==='boolean'?a:{{matches:[...a],index:a.index,input:a.input,groups:a.groups}}}})())"
        );
        let Value::String(result) = Realm::default().eval(&script).unwrap() else {
            panic!("expected JSON");
        };
        writeln!(rows, "{source:?} {result:?}").unwrap();
    }
    let setup = "let r=new RegExp('a','v'),t='';r.lastIndex={valueOf(){t+='i';return 0;}};";
    let operation = "r.exec({toString(){t+='s';return 'a';}})";
    let mut realm = Realm::default();
    realm.eval(setup).unwrap();
    assert!(matches!(realm.eval(operation), Ok(Value::Object(_))));
    assert_eq!(
        realm.eval("t==='si'&&typeof r.lastIndex==='object'"),
        Ok(Value::Boolean(true))
    );
    let script = format!(
        "{setup}let a={operation};JSON.stringify({{matches:[...a],index:a.index,input:a.input,trace:t,indexKind:typeof r.lastIndex}})"
    );
    let Value::String(result) = Realm::default().eval(&script).unwrap() else {
        panic!("expected ordered JSON");
    };
    writeln!(rows, "{script:?} {result:?}").unwrap();
    insta::assert_snapshot!(rows, @r#"
        "/(a)/v.test('a')" "true"
        "/a/v.exec('a')" "{\u0022matches\u0022:[\u0022a\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022a\u0022}"
        "/(?<x>a)/v.test('a')" "true"
        "/(?:a)/v.test('a')" "true"
        "let r=new RegExp('a','v'),t='';r.lastIndex={valueOf(){t+='i';return 0;}};let a=r.exec({toString(){t+='s';return 'a';}});JSON.stringify({matches:[...a],index:a.index,input:a.input,trace:t,indexKind:typeof r.lastIndex})" "{\u0022matches\u0022:[\u0022a\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022a\u0022,\u0022trace\u0022:\u0022si\u0022,\u0022indexKind\u0022:\u0022object\u0022}"
        "#);
    assert_true(
        r"let r=/a/vg;r.lastIndex=1;let a=r.exec('\uD800a\uDC00');a.index===1&&r.lastIndex===2",
    );
    assert_true(
        r"let r=/a/vy;r.lastIndex=1;let a=r.exec('\uD800a\uDC00');a.index===1&&r.lastIndex===2",
    );
    assert_true(
        r"let r=/(?<x>a(?<y>()))b/dvg;r.lastIndex=1;let a=r.exec('\uD83D\uDE00ab');a.index===2&&a.groups.x==='a'&&a.groups.y===''&&a.indices.groups.x===a.indices[1]&&a.indices.groups.y===a.indices[2]&&a.indices[2][0]===3",
    );
    assert_true(r"let r=/a/vy;r.lastIndex=1;r.exec('\uD83D\uDE00a')===null&&r.lastIndex===0");
    assert_true(
        r"let r=/a/vg;r.lastIndex=1;let a=r.exec('\uD83D\uDE00a');a.index===2&&r.lastIndex===3",
    );
    assert_true(
        r"let r=/a/vy;r.lastIndex=2;let a=r.exec('\uD83D\uDE00a');a.index===2&&r.lastIndex===3",
    );
    assert_true(r"let r=/a/v;r.exec('A')===null&&r.unicode===false&&r.unicodeSets===true");
}

#[test]
fn unicode_sets_bmp_consumers_callbacks_species_and_copies_keep_code_unit_indices() {
    assert_true(
        r#"
            let r = /(a())b/vg;
            r.lastIndex = 1;
            let a = [...'\uD83D\uDE00ab\uD83D\uDE00ab'.matchAll(r)];
            a.length === 2 && a[0].index === 2 && a[1].index === 6 && a[0][1] === 'a' && a[0][2] === '' &&
                r.lastIndex === 1
        "#,
    );
    assert_true(
        r#"
            let seen = [];
            let s = '\uD83D\uDE00ab\uD83D\uDE00ab'.replace(/(a())b/vg, (m, x, y, i) => {
                seen.push(i);
                return '_'
            });
            s === '\uD83D\uDE00_\uD83D\uDE00_' && seen.join(',') === '2,6'
        "#,
    );
    assert_true(r"'\uD83D\uDE00ab\uD83D\uDE00ab'.match(/ab/vg).join('|')==='ab|ab'");
    assert_true(r"'\uD83D\uDE00ab'.split(/(a())b/v).join('|')==='\uD83D\uDE00|a||'");
    assert_true(r"let r=/ab/vg;r.lastIndex=1;'\uD83D\uDE00ab'.search(r)===2&&r.lastIndex===1");
    assert_true(
        r#"
            let r = /(?<x>ab)/dvg,
                c = new RegExp(r);
            let a = c.exec('\uD83D\uDE00ab');
            a.groups.x === 'ab' && a.indices.groups.x === a.indices[1] && r.lastIndex === 0 && c
                .lastIndex === 4
        "#,
    );
}

#[test]
fn unicode_sets_bmp_input_index_coercion_and_strict_writes_remain_ordered() {
    assert_true(
        r#"
            let r = /a/v,
                t = '';
            r.lastIndex = {
                valueOf() {
                    t += 'i';
                    return 2;
                }
            };
            let a = r.exec({
                toString() {
                    t += 's';
                    return 'ba'
                }
            });
            t === 'si' && a.index === 1 && typeof r.lastIndex === 'object'
        "#,
    );
    assert_true(
        r#"
            let r = /a/vg,
                t = '';
            r.lastIndex = {
                valueOf() {
                    t += 'i';
                    return 1;
                }
            };
            let a = r.exec({
                toString() {
                    t += 's';
                    return 'ba'
                }
            });
            t === 'si' && a.index === 1 && r.lastIndex === 2
        "#,
    );
    assert_true(
        r#"
            let r = /a/vy;
            r.lastIndex = 1;
            Object.defineProperty(r, 'lastIndex', {
                writable: false
            });
            let caught = false;
            try {
                r.exec('ba')
            } catch (e) {
                caught = e instanceof TypeError
            }
            caught && r.lastIndex === 1
        "#,
    );
    assert_true(
        r#"
            let r = /a/vy;
            r.lastIndex = 1;
            Object.defineProperty(r, 'lastIndex', {
                writable: false
            });
            let caught = false;
            try {
                r.exec('bb')
            } catch (e) {
                caught = e instanceof TypeError
            }
            caught && r.lastIndex === 1
        "#,
    );
}

#[test]
fn unicode_sets_bmp_opted_in_work_abort_and_unproved_modes_remain_distinct() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm.eval("let marker=0,r=new RegExp('a'.repeat(64),'vg');r.lastIndex=1;let text='b'.repeat(20000)").unwrap();
    assert!(matches!(
        realm.eval("try{r.exec(text)}catch{marker=1}finally{marker=2}"),
        Err(Error::Limit { .. })
    ));
    assert_eq!(
        realm.eval("marker===0&&r.lastIndex===1"),
        Ok(Value::Boolean(true))
    );
    for source in [
        r"/(?:a?)b/v.exec('a')",
        r"/(?:a)+b/iv.exec('a')",
        r"/ab*/v.exec('a')",
        r"/\uD800()\uDC00/v.exec('\uD800')",
        r"/\u{D800}\u{DC00}/v.exec('a')",
        r"/([\u{1f600}])+b/v.exec('a')",
        r"/(?:^a)+b/v.exec('a')",
        r"/(a)\1/v.exec('aa')",
    ] {
        crate::common::regexp_oracle::assert_matches_reference(&mut Realm::default(), source);
    }
}
