//! Match results and consumers for character classes next to word assertions.

use crate::common::assert_true;
use spite_core::JsString;
use spite_runtime::{Error, Limits, Realm, Value};
use std::fmt::Write;

#[test]
fn checked_v_set_word_state_context_and_consumers() {
    assert_true(
        r#"
            let w = /\b[[Kſα]]\b/iv,
                n = /\B[[Kſα]]\B/iv;
            w.test('K') && w.test('ſ') && !w.test('α') && n.test('α') && !n.test('K') && !n.test('ſ') &&
                /\B[[Kſα]]\B/v.test('ſ')
        "#,
    );
    assert_true(
        r#"
            /\b[a--b]\B/v.exec('aa').index === 0 && /\B[a--b]\b/v.exec('aa').index === 1 && /\b[a--b]/v
                .test('aa') && /[a--b]\b/v.exec('aa').index === 1 && !/\b[[a--a]]\b/v.test('a') && !
                /\B[[a--a]]\B/iv.test('!')
        "#,
    );
    assert_true(
        r#"
            let a = /\b[[\p{Any}]]\b/vy,
                b = /\B[[\p{Any}]]\b/vy;
            a.lastIndex = 1;
            b.lastIndex = 1;
            let m = b.exec('😀B');
            a.exec('😀B') === null && a.lastIndex === 0 && m.index === 0 && m[0] === '😀' && b
                .lastIndex === 2 && b.exec('😀B') === null && b.lastIndex === 0
        "#,
    );
    assert_true(
        r#"
            let r = /\b[[\P{space}]]\b/dvy;
            r.lastIndex = 1;
            let m = r.exec('A\uD800B');
            m.index === 1 && m[0] === '\uD800' && m.indices[0][0] === 1 && m.indices[0][1] === 2 && r
                .lastIndex === 2
        "#,
    );
    assert_true(
        r"/^\b[[K]]\b$/iv.test('K')&&!/^\b[[K]]\b$/iv.test('K\n')&&/^\b[[K]]\b$/imv.exec('x\r\nK\u2028').index===3",
    );
    assert_true(
        r#"
            let r = /\b[[Kſ]]\b/ivg;
            r.lastIndex = 1;
            'A K ſ K_'.search(r) === 2 && r.lastIndex === 1 && 'K ſ K_'.replace(/(?<whole>\b[[Kſ]]\b)/ivg,
                '<$<whole>>') === '<K> <ſ> K_'
        "#,
    );
    assert_true(
        r#"
            let r = /(?<whole>(\b[[Kſ]]\b))/divg,
                a = [...'K ſ K_'.matchAll(r)];
            a.length === 2 && a[0][1] === 'K' && a[0][2] === 'K' && a[0].index === 0 && a[0].indices.groups
                .whole === a[0].indices[1] && a[1].groups.whole === 'ſ' && a[1].index === 2 && a[1]
                .indices[2][1] === 3 && r.lastIndex === 0
        "#,
    );
    assert_true(
        r#"
            let t = '',
                r = /\b[[\p{Any}]]\b/vy;
            r.lastIndex = {
                valueOf() {
                    t += 'i';
                    return 2
                }
            };
            let m = r.exec({
                toString() {
                    t += 's';
                    return 'A😀B'
                }
            });
            t === 'si' && m.index === 1 && m[0] === '😀' && r.lastIndex === 3
        "#,
    );
    assert_true(
        r#"
            let r = /\b[a--b]\b/vy;
            r.lastIndex = 1;
            Object.defineProperty(r, 'lastIndex', {
                writable: false
            });
            let caught = false;
            try {
                r.exec('aa')
            } catch (e) {
                caught = e instanceof TypeError
            }
            caught && r.lastIndex === 1
        "#,
    );
    assert_true(
        r#"
            let marker = 0,
                result, caught = false;
            try {
                marker = 1;
                result = /\b[a--b]+\b/v.test('a')
            } catch {
                caught = true
            } finally {
                marker = 3
            }!caught && marker === 3 && result && !/\b[a--b]+\b/v.test('b') && /\b[a--b]+\b/v.exec('aaa!')[
                0] === 'aaa'
        "#,
    );
    for source in [r"/\b^[[K]]/v.test('K')", r"/[[K]]$\b/v.test('K')"] {
        crate::common::regexp_oracle::assert_matches_reference(&mut Realm::default(), source);
    }
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm.eval(r"var r=/\b[[a--b]]\b/vg,marker=0").unwrap();
    // Construct this nonmember input outside the opted-in JavaScript work.
    let setup = format!("var text='{}';r.lastIndex=1", "b".repeat(20_000));
    realm.eval(&setup).unwrap();
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
fn checked_v_set_word_results() {
    let mut rows = String::new();
    let mut row_count = 0;
    for (pattern, input, start, flag_modes) in [
        (
            r"(?<whole>(\b[[Kſα]]\b))",
            JsString::from("K ſ α"),
            0,
            ["dvg", "divg"],
        ),
        (
            r"\B(?<whole>([[Kſα]]))\B",
            JsString::from("K ſ α"),
            0,
            ["dvg", "divg"],
        ),
        (
            r"(?<whole>(\b[[\p{Any}]]\b))",
            JsString::from("A😀B"),
            2,
            ["dvy", "divy"],
        ),
        (
            r"^(?<whole>(\b[[K]]\b))$",
            JsString::from("x\r\nK\u{2028}K\u{2029}"),
            0,
            ["dvmg", "divmg"],
        ),
        (
            r"(?<whole>(\b[[\b]]\B))",
            JsString::from("A\u{8}!"),
            0,
            ["dvg", "divg"],
        ),
    ] {
        let pattern = JsString::from(pattern);
        for flags in flag_modes {
            let script = format!(
                r#"
                    let r = new RegExp({pattern:?}, '{flags}');
                    r.lastIndex = {start};
                    let m = r.exec({input:?});
                    JSON.stringify(m === null ? {{
                        match: null,
                        lastIndex: r.lastIndex
                    }} : {{
                        matches: [...m],
                        index: m.index,
                        input: m.input,
                        groups: m.groups,
                        indices: m.indices,
                        indexGroups: m.indices.groups,
                        lastIndex: r.lastIndex,
                        source: r.source
                    }}, null, 2)
                "#
            );
            let Value::String(result) = Realm::default().eval(&script).unwrap() else {
                panic!("{script}")
            };
            writeln!(
                rows,
                "{pattern:?} flags={flags:?} input={input:?} lastIndex={start}\n{result}\n",
                result = result.to_utf8().expect("JSON output must be valid UTF-8"),
            )
            .unwrap();
            row_count += 1;
        }
    }
    assert_eq!(row_count, 10);
    insta::assert_snapshot!(rows);
}
