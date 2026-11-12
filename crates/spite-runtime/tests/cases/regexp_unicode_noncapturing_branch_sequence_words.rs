//! Outside word predicates compose with proved one-character alternatives.
use crate::common::assert_true;
use crate::common::regexp_matches::last_index_cases;
use spite_core::JsString;
use spite_runtime::{Realm, Value};
use std::fmt::Write;
#[test]
fn unicode_noncapturing_branch_sequence_words_results_snapshot() {
    let mut rows = String::new();
    for atom in [
        r"\b((?:a)|(?:b))+\B",
        r"\B(((?:a)|(?:[ſK]))){0,3}?\b",
        r"\b(?:(?:.)|(?:\n)){2,4}\b",
        r"\B((?:\d)|(?:\D))*",
        r"^((?:\p{Join_C})|(?:\p{NChar}))+\B$",
        r"\b((?:😀)|(?:a)){1,3}\b$",
        r"((?:\uD800)|(?:\uDC00)){0,2}?\b",
        r"\b((?:\|)|(?:a))+",
        r"\b(\b\b(?:a)|(?:b)\b\b)+\B",
        r"(^^(?:a)|(?:b)$$){1,2}\b$",
        r"\B((?:[a\|])|(?:b))??\B",
        r"\B(\B\B(?:.)|\b\b(?:[aſK])){1,3}\b",
        r"((?:a)$$|^^(?:b))+\b",
        r"^\b((?:[a-z])|(?:\p{Join_C})){0,2}\B$",
        r"((?:\u{61})|(?:\u{1f600}))?\b",
        r"\B(\B^\B(?:.)$$|\b\b(?:a)\b\b){1,3}\B$",
    ] {
        let pattern = JsString::from(format!("(?<outer>{atom})").as_str());
        for flags in ["dug", "diumg", "divmg", "divy", "dsumg", "disvmy"] {
            for input in [
                JsString::from("Aaaa\nAA"),
                JsString::from("A\r\n\u{200c}\u{200d}\u{ffff}\u{10ffff}"),
                JsString::from("😀??aaſK"),
                JsString::from_code_units(vec![0x61, 0xd800, 0x61, 0xdc00]),
                JsString::from(""),
            ] {
                for start in last_index_cases(input.len()) {
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
                            }})
                        "#
                    );
                    let Value::String(result) = Realm::default().eval(&script).unwrap() else {
                        panic!("{script}")
                    };
                    writeln!(
                        rows,
                        "{pattern:?} flags={flags:?} input={input:?} lastIndex={start} => {result}",
                        result = result.to_utf8().expect("JSON output must be valid UTF-8"),
                    )
                    .unwrap();
                }
            }
        }
    }
    insta::assert_snapshot!(rows);
}

#[test]
fn unicode_noncapturing_branch_sequence_words_counts_and_scopes() {
    for source in [
        r"/\B((^(?:.)|a)+)\B$b/vms.exec('\n')",
        r"/\B(?<whole>(^(?:.)|a)+)\B$b/vms.exec('\n')",
    ] {
        crate::common::regexp_oracle::assert_matches_reference(&mut Realm::default(), source);
    }

    for mode in ["u", "v"] {
        assert_true(&format!(
            r"let m=/\b((?:a)|(?:b))+\B/{mode}.exec('abba');m[0]==='abb'&&m[1]==='b'&&m.index===0"
        ));
        assert_true(&format!(
            r"let m=/\B((?:a)|(?:b))+\b/{mode}.exec('abba');m[0]==='bba'&&m[1]==='a'&&m.index===1"
        ));
        assert_true(&format!(
            r"let m=/\b(\b\b(?:a)|(?:b)\b\b)+\B/{mode}.exec('aaa');m[0]==='a'&&m[1]==='a'&&m.index===0"
        ));
        assert_true(&format!(
            r"let m=/\B(^^(?:a)|(?:b)$$)+\b/{mode}m.exec('aa\nbb');m[0]==='b'&&m[1]==='b'&&m.index===4"
        ));
        assert_true(&format!(
            r"let m=/\b((?:a)|(?:b))*?\b/{mode}.exec('ab');m[0]===''&&m[1]===undefined"
        ));
        assert_true(&format!(
            r"let m=/\B(\B^\B(?:.)$$|\b\b(?:a)\b\b)+\B/{mode}ms.exec('\r\n');m[0]==='\r\n'&&m[1]==='\n'"
        ));
    }
}

#[test]
fn unicode_noncapturing_branch_sequence_words_contradictions_empty_counts_and_literals() {
    assert_true(r"let r=/\b(\b\B(?:a)|b)+\b/v;r.exec('a')===null&&r.exec('b')[0]==='b'");
    assert_true(r"let r=/\b((?:a)\b\B|b)+\b/u;r.exec('a')===null&&r.exec('b')[0]==='b'");
    assert_true(
        r#"
            let m = /\b(\b\B(?:a)|\b\B(?:b))*\b/dv.exec('ab');
            m[0] === '' && m[1] === undefined && m.indices[1] === undefined &&
                /\b(\b\B(?:a)|\b\B(?:b))+\b/v.exec('ab') === null
        "#,
    );
    assert_true(
        r#"
            let m = /\B(?<whole>(\b\B(?:a)|(?:b)\b\B)*)\B/du.exec('');
            m[0] === '' && m[1] === '' && m.groups.whole === '' && m[2] === undefined && m.indices[1][
                0
            ] === 0 && m.indices[1][1] === 0 && m.indices[2] === undefined
        "#,
    );
}
