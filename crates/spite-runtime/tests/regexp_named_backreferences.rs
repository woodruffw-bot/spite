//! Ordinary named references reuse input capture state and preserve source text.

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
    insta::assert_snapshot!(rows);
}

#[test]
fn decoded_names_original_ranges_and_reference_digit_boundaries_are_preserved() {
    check(
        r"let a=/(?<x>a)\k<x>0/d.exec('qaa0');a[0]==='aa0'&&a[1]==='a'&&a.groups.x==='a'&&a.indices.groups.x===a.indices[1]&&a.indices[0][1]===4",
    );
    check(
        r"let a=/()()()()()()()()()(?<x>a)\k<x>0/d.exec('qaa0');a.length===11&&a[10]==='a'&&a.groups.x==='a'&&a.indices.groups.x===a.indices[10]&&a[0]==='aa0'",
    );
    check(
        r"let a=/(?<outer>a(?<x>\k<outer>b)\k<x>)\k<outer>/d.exec('qabbabb');a[0]==='abbabb'&&a.groups.outer==='abb'&&a.groups.x==='b'&&a.indices.groups.outer===a.indices[1]&&a.indices.groups.x===a.indices[2]&&a.indices[2][0]===2&&a.indices[2][1]===3",
    );
    check(
        r"let a=/\k<x>(?<x>a)/d.exec('qa'),b=/(?<x>a\k<x>)/d.exec('qa');a[0]==='a'&&a.groups.x==='a'&&b[0]==='a'&&b.groups.x==='a'",
    );
    check(
        r"let a=/(?<\u0078>a)\k<\u{78}>/di.exec('qAa');a[0]==='Aa'&&a.groups.x==='A'&&Object.keys(a.groups).join(',')==='x'&&Object.getPrototypeOf(a.groups)===null",
    );
    check(
        r"let a=/(?<__proto__>a)\k<__proto__>/d.exec('aa');Object.getPrototypeOf(a.groups)===null&&a.groups.__proto__==='a'&&a.indices.groups.__proto__===a.indices[1]",
    );
}

#[test]
fn native_consumers_and_copies_share_reference_results_and_original_source() {
    check(
        r"'qaa'.replace(/(?<x>a)\k<x>/,'<$<x>>')==='q<a>'&&'qaa'.split(/(?<x>a)\k<x>/).join(',')==='q,a,'&&'qaa'.search(/(?<x>a)\k<x>/)===1",
    );
    check(
        r"let seen;'qAa'.replace(/(?<x>a)\k<x>/i,(whole,x,index,input,groups)=>{seen=[whole,x,index,input,groups.x];return 'z'});seen.join(',')==='Aa,A,1,qAa,A'",
    );
    check(
        r"let r=/(?<x>a)\k<x>/dg,a=[...'qaa aa'.matchAll(r)];a.length===2&&a[0].index===1&&a[1].index===4&&a[1].indices.groups.x===a[1].indices[1]&&r.lastIndex===0",
    );
    check(
        r"let r=/(?<x>)\k<x>/g,a=r.exec('q');a[0]===''&&r.lastIndex===0&&[...'q'.matchAll(r)].length===2",
    );
    check(
        r"let r=/(?<\u0078>a)\k<\u{78}>/dg;Object.defineProperty(r,'source',{get(){throw 7}});let copy=new RegExp(r);copy.source==='(?<\\u0078>a)\\k<\\u{78}>'&&copy.exec('aa').groups.x==='a'&&copy.lastIndex===2&&r.lastIndex===0",
    );
    check(
        r"let r=/(?<x>a)\k<x>/gy;r.lastIndex=1;let a=r.exec('qaa');a.index===1&&r.lastIndex===3&&r.exec('qaa')===null&&r.lastIndex===0",
    );
}

#[test]
fn deep_named_reference_programs_survive_copy_and_collection_without_default_quotas() {
    let mut realm = Realm::default();
    realm.eval("let r=new RegExp('('.repeat(100000)+'(?<x>a)'+')'.repeat(100000)+'\\\\k<x>','d'),copy=new RegExp(r)").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("let a=copy.exec('qaa');a[0]==='aa'&&a.length===100002&&a.groups.x==='a'&&a.indices.groups.x===a.indices[100001]&&a.indices[100001][0]===1&&a.indices[100001][1]===2&&copy.source===r.source"),Ok(Value::Boolean(true)));
    realm.collect(usize::MAX).unwrap();
    assert_eq!(
        realm.eval("a.groups.x==='a'&&a.indices.groups.x===a.indices[100001]"),
        Ok(Value::Boolean(true))
    );
    assert_eq!(realm.eval("new RegExp('(?<x>a)'+'\\\\k<x>'.repeat(100000)).exec('a'.repeat(100001)).groups.x==='a'"),Ok(Value::Boolean(true)));
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
fn pending_reference_compositions_remain_unsupported_and_failed_matches_return_null() {
    for source in [
        r"/(?:(?:(?:(?<x>a)(?:\k<x>)+){2})|){2,3}/.test('aa')",
        r"/(?:(?:(?:(?<x>a)|(?<x>b)(?:\k<x>)+){2})|){2,3}/.test('bb')",
        r"/(?:(?:(?:(?<x>[ab])(?:\k<x>)+){2})|){2,3}/.test('aa')",
        r"/(?:(?:(?:^(?<x>a)(?:\k<x>)+){2})|){2,3}/.test('aa')",
        r"/(?<x>a)\k<x>/u.test('aa')",
    ] {
        assert!(
            matches!(
                Realm::default().eval(source),
                Err(Error::Unsupported { .. })
            ),
            "{source}"
        );
    }
    check(r"/(?<x>a)\k<x>/.exec('ab')===null");
}
