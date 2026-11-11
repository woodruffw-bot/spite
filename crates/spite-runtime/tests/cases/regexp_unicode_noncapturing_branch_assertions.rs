//! Complete noncapturing branches consume one character with scoped predicates.
use crate::common::assert_true;
use spite_core::JsString;
use spite_runtime::{Realm, Value};
use std::fmt::Write;
#[test]
fn unicode_noncapturing_branch_assertions_results_snapshot() {
    let mut rows = String::new();
    for atom in [
        r"((?:a)|(?:b))+",
        r"(((?:a)|(?:[ſK]))){0,3}?",
        r"(?:(?:.)|(?:\n)){2,4}",
        r"((?:\d)|(?:\D))*",
        r"^((?:\p{Join_C})|(?:\p{NChar}))+$",
        r"((?:😀)|(?:a)){1,3}$",
        r"((?:\uD800)|(?:\uDC00)){0,2}?",
        r"((?:\|)|(?:a))+",
        r"(\b(?:a)|(?:b)\b)+",
        r"(^(?:a)|(?:b)$){1,2}$",
        r"((?:[a\|])|(?:b))??",
        r"(\B(?:.)|\b(?:[aſK])){1,3}",
        r"((?:a)$|^(?:b))+",
        r"^((?:[a-z])|(?:\p{Join_C})){0,2}$",
        r"((?:\u{61})|(?:\u{1f600}))?",
        r"(^\B(?:.)$|\b(?:a)\b){1,3}$",
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
                for start in [0, 1, 2, input.len(), input.len() + 1] {
                    let script = format!(
                        "let r=new RegExp({pattern:?},'{flags}');r.lastIndex={start};let m=r.exec({input:?});JSON.stringify(m===null?{{match:null,lastIndex:r.lastIndex}}:{{matches:[...m],index:m.index,input:m.input,groups:m.groups,indices:m.indices,indexGroups:m.indices.groups,lastIndex:r.lastIndex,source:r.source}})"
                    );
                    let Value::String(result) = Realm::default().eval(&script).unwrap() else {
                        panic!("{script}")
                    };
                    writeln!(
                        rows,
                        "{pattern:?} flags={flags:?} input={input:?} lastIndex={start} {result:?}"
                    )
                    .unwrap();
                }
            }
        }
    }
    insta::assert_snapshot!(rows);
}

#[test]
fn unicode_noncapturing_branch_assertions_counts_and_branch_scopes() {
    assert_true(r"let r=/(\b(?:\Ba)|b)+/v;r.exec('a')===null&&r.exec('b')[0]==='b'");
    assert_true(r"let r=/((?:a\b)\B|b)+/u;r.exec('a')===null&&r.exec('b')[0]==='b'");
    for mode in ["u", "v"] {
        assert_true(&format!(
            r"let m=/((?:a)|(?:b))+/{mode}.exec('abba');m[0]==='abba'&&m[1]==='a'&&m.index===0"
        ));
        assert_true(&format!(
            r#"
                let m = /(\b(?:a)|(?:b)\b)+/{mode}.exec('aaa bbb');
                m[0] === 'a' && m[1] === 'a' && /(\b(?:a)|(?:b)\b){{2}}/{mode}.exec('aaa bbb') === null
            "#
        ));
        assert_true(&format!(
            r"let m=/(^(?:a)|(?:b)$)+/{mode}m.exec('aa\nbb');m[0]==='a'&&m[1]==='a'&&m.index===0"
        ));
        assert_true(&format!(
            r"let m=/(((?:a)|(?:b)))*?/{mode}.exec('ab');m[0]===''&&m[1]===undefined&&m[2]===undefined"
        ));
        assert_true(&format!(
            r#"
                let m = /((?:a)|(?:b)){{2,3}}/{mode}.exec('abba');
                m[0] === 'abb' && m[1] === 'b' && /((?:a)|(?:b)){{2,3}}?/{mode}.exec('abba')[0] === 'ab'
            "#
        ));
        assert_true(&format!(
            r"let m=/((?:[a\|])|(?:b))+/{mode}.exec('a|b');m[0]==='a|b'&&m[1]==='b'"
        ));
        assert_true(&format!(
            r"let m=/((?:\|)|(?:a))+/{mode}.exec('|aa');m[0]==='|aa'&&m[1]==='a'"
        ));
        assert_true(&format!(
            r"let m=/(^\B(?:.)$|\b(?:a)\b)+/{mode}ms.exec('\r\n');m[0]==='\r\n'&&m[1]==='\n'"
        ));
    }
}
