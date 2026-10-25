//! Outside word predicates compose with proved one-character alternatives.
use spite_core::JsString;
use spite_runtime::{Error, Limits, Realm, Value};
use std::fmt::Write;
fn check(source: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::Boolean(true)),
        "{source}"
    );
}
#[test]
fn unicode_word_asserted_whole_captures_results_snapshot() {
    let mut rows = String::new();
    for pattern in [
        r"\b(?<outer>((?:a)|(?:b))+)\B",
        r"\B(?<outer>(((?:a)|(?:[ſK]))){0,3}?)\b",
        r"\b(?<outer>(?:(?:.)|(?:\n)){2,4})\b",
        r"\B(?<outer>((?:\d)|(?:\D))*)",
        r"^(?<outer>((?:\p{Join_C})|(?:\p{NChar}))+)\B$",
        r"\b(?<outer>((?:😀)|(?:a)){1,3})\b$",
        r"(?<outer>((?:\uD800)|(?:\uDC00)){0,2}?)\b",
        r"\b(?<outer>((?:\|)|(?:a))+)",
        r"\b(?<outer>((?:\ba)|(?:b\b))+)\B",
        r"(?<outer>(^(?:a)|(?:b)$){1,2})\b$",
        r"\B(?<outer>((?:[a\|])|(?:b))??)\B",
        r"\B(?<outer>((?:\B.)|(?:\b[aſK])){1,3})\b",
        r"(?<outer>((?:a)$|^(?:b))+)\b",
        r"^\b(?<outer>((?:[a-z])|(?:\p{Join_C})){0,2})\B$",
        r"(?<outer>((?:\u{61})|(?:\u{1f600}))?)\b",
        r"\B(?<outer>(^(?:\B.)$|(?:\ba\b)){1,3})\B$",
    ] {
        let pattern = JsString::from(pattern);
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
fn unicode_word_asserted_whole_captures_counts_and_scopes() {
    for source in [
        r"/\B((^(?:.)|a)+)\B$/vms.exec('\n')",
        r"/\B(?<whole>(^(?:.)|a)+)\B$/vms.exec('\n')",
    ] {
        check(&format!(
            "let m={source};m!==null&&m[0]==='\\n'&&m[1]==='\\n'&&m[2]==='\\n'"
        ));
    }

    for mode in ["u", "v"] {
        check(&format!(
            r"let m=/\b((?:a)|(?:b))+\B/{mode}.exec('abba');m[0]==='abb'&&m[1]==='b'&&m.index===0"
        ));
        check(&format!(
            r"let m=/\B((?:a)|(?:b))+\b/{mode}.exec('abba');m[0]==='bba'&&m[1]==='a'&&m.index===1"
        ));
        check(&format!(
            r"let m=/\b((?:\ba)|(?:b\b))+\B/{mode}.exec('aaa');m[0]==='a'&&m[1]==='a'&&m.index===0"
        ));
        check(&format!(
            r"let m=/\B(^(?:a)|(?:b)$)+\b/{mode}m.exec('aa\nbb');m[0]==='b'&&m[1]==='b'&&m.index===4"
        ));
        check(&format!(
            r"let m=/\b((?:a)|(?:b))*?\b/{mode}.exec('ab');m[0]===''&&m[1]===undefined"
        ));
        check(&format!(
            r"let m=/\B(^(?:\B.)$|(?:\ba\b))+\B/{mode}ms.exec('\r\n');m[0]==='\r\n'&&m[1]==='\n'"
        ));
    }
}
#[test]
fn unicode_word_asserted_whole_captures_fold_surrogates_and_names() {
    check(
        r"let m=/(?<whole>\B(?<__proto__>(?:ſ)|(?:K))+\b)/dvi.exec('XSk');m[0]==='Sk'&&m[1]==='Sk'&&m[2]==='k'&&m.groups.__proto__==='k'&&m.indices.groups.__proto__[0]===2&&m.indices.groups.__proto__[1]===3",
    );
    check(
        r"let m=/(?<whole>\B(?<last>(?:😀)|(?:a))+\B)/dvg.exec('😀😀');m[0]==='😀😀'&&m.groups.last==='😀'&&m.indices.groups.last[0]===2&&m.indices.groups.last[1]===4",
    );
    check(
        r"let r=/\B((?:😀)|(?:a))+\B/duy;r.lastIndex=1;let m=r.exec('😀😀');m.index===0&&m[0]==='😀😀'&&m.indices[1][0]===2&&r.lastIndex===4",
    );
    check(
        r"let m=/\b(?<x>(?:a)|(?:b)){0}\b/dv.exec('ab');m[0]===''&&m[1]===undefined&&m.groups.x===undefined&&m.indices[1]===undefined",
    );
    check(
        r"let m=/\B((?:\uD800)|(?:\uDC00))+/du.exec('\uD800x\uDC00');m[0]==='\uD800'&&m.indices[1][1]===1&&/\B((?:\uD83D)|(?:\uDE00))+/u.exec('😀')===null",
    );
}
#[test]
fn unicode_word_asserted_whole_captures_string_consumers() {
    check(r"let a='ab ba'.match(/\b((?:a)|(?:b))+\B/ug);a.length===2&&a.join('|')==='a|b'");
    check(
        r"'ab ba'.replace(/\b((?:a)|(?:b))+\B/vg,'<$1>')==='<a>b <b>a'&&'Xab'.search(/\B((?:a)|(?:b))+\b/v)===1",
    );
    check(r"let a='ab ba'.split(/\b((?:a)|(?:b))+\B/u);a.length===5&&a.join('|')==='|a|b |b|a'");
    check(
        r"let a=[...'ab ba'.matchAll(/\B(?<x>(?:a)|(?:b))+\b/dvg)];a.length===2&&a[0].index===1&&a[1].index===4&&a[1].groups.x==='a'&&a[1].indices.groups.x[0]===4",
    );
}
#[test]
fn unicode_word_asserted_whole_captures_coercions_and_last_index() {
    check(
        r"let log='',r=/\B((?:a)|(?:b))+\b/vgy;r.lastIndex={valueOf(){log+='i';return 1}};let m=r.exec({toString(){log+='s';return 'Xab'}});log==='si'&&m.index===1&&r.lastIndex===3",
    );
    check(
        r"let r=/\B((?:a)|(?:b))+\b/ug;r.lastIndex=1;Object.defineProperty(r,'lastIndex',{writable:false});let caught=false;try{r.exec('Xab')}catch(e){caught=e instanceof TypeError}caught&&r.lastIndex===1",
    );
    check(
        r"let r=/\B((?:a)|(?:b))+\b/v;Object.defineProperty(r,'lastIndex',{value:100,writable:false});let m=r.exec('Xab');m.index===1&&r.lastIndex===100",
    );
}

#[test]
fn unicode_word_asserted_whole_captures_opt_in_work_failure() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval(r"let r=/\B((?:\P{NChar})|(?:a)){2}\B$/umg,marker=0,text='😀'.repeat(10000);r.lastIndex=1")
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
fn unicode_word_asserted_whole_captures_single_runes_empty_counts_and_aliases() {
    check(
        r"let m=/(\b)\b/du.exec('A');m[0]===''&&m[1]===''&&m.index===0&&m.indices[1][0]===0&&m.indices[1][1]===0",
    );
    check(
        r"let m=/\b([A])\b/du.exec('A');m[0]==='A'&&m[1]==='A'&&m.index===0&&m.indices[1][0]===0&&m.indices[1][1]===1",
    );

    check(
        r"let m=/\b(a)\b/du.exec('a');m.length===2&&m[0]==='a'&&m[1]==='a'&&m.indices[1][0]===0&&m.indices[1][1]===1",
    );
    check(
        r"let m=/^\b((?:[aſK]))\b$/dvi.exec('S');m[0]==='S'&&m[1]==='S'&&m.indices[1][0]===0&&m.indices[1][1]===1",
    );
    check(
        r"let r=/\B(?<__proto__>😀)\B/dvy;r.lastIndex=1;let m=r.exec('😀');m.index===0&&m.groups.__proto__==='😀'&&m.indices.groups.__proto__[0]===0&&m.indices.groups.__proto__[1]===2&&r.lastIndex===2",
    );
    check(
        r"let m=/\B(?<whole>(a|b)*)\B/dv.exec('');m[0]===''&&m[1]===''&&m.groups.whole===''&&m[2]===undefined&&m.indices[2]===undefined&&m.indices[1][0]===0&&m.indices[1][1]===0",
    );
    check(
        r"let r=/\b(?<whole>(a|b)+)\b$/dvg,m=r.exec(' ab');m[0]==='ab'&&m.index===1&&m.groups.whole==='ab'&&m[2]==='b'&&m.indices[1][0]===1&&m.indices[1][1]===3&&m.indices[2][0]===2&&m.indices[2][1]===3&&r.lastIndex===3&&r.source==='\\b(?<whole>(a|b)+)\\b$'",
    );
}

#[test]
fn unicode_word_asserted_whole_captures_former_whole_programs_snapshot() {
    let mut rows = String::new();
    for source in [
        r"/(\b)\b/u.test('A')",
        r"/\b([A])\b/u.exec('A')",
        r"/\B((^(?:.)|a)+)\B$/vms.exec('\n')",
        r"/\B(?<whole>(^(?:.)|a)+)\B$/vms.exec('\n')",
    ] {
        let Value::String(result) = Realm::default()
            .eval(&format!("JSON.stringify({source})"))
            .unwrap()
        else {
            panic!("{source}")
        };
        writeln!(rows, "{source} {result:?}").unwrap();
    }
    insta::assert_snapshot!(rows);
}
