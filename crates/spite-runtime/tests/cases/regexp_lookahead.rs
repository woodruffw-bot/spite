//! Ordinary positive and negative lookahead and atomic capture state.
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
fn lookahead_result_snapshot() {
    let mut rows = String::new();
    for (source, flags, text) in [
        (r"a(?=(b))", "d", "ab"),
        (r"(?=(a|ab))\1b$", "d", "abb"),
        (r"(?=(ab|a))\1b$", "d", "abb"),
        (r"(?=(a+))a*b\1", "d", "baabac"),
        (r"(?!(a)b)\1a", "d", "a"),
        (r"(?=(?=(a|b))\1)\1", "d", "b"),
        (r"(?!(?!a))a", "d", "ba"),
        (r"(?<prefix>a)(?=\k<prefix>)\k<prefix>", "d", "aa"),
        (r"(?=(?<x>a|b)+)\k<x>", "d", "ab"),
        (r"(?:(?=(?<x>a|b))\k<x>c)+", "d", "acbc"),
        (r"(?:(?=(?<x>a|b))a|b)+c", "d", "abc"),
        (r"(?:(?!(?<x>a)b)a|b)+c", "d", "abc"),
        (r"(?=(?<x>a*))\k<x>", "d", "q"),
        (r"(?!(?<x>a+))b", "d", "b"),
        (r"(?=(^a))\1", "dm", "q\na"),
        (r"(?=(.$))a", "ds", "a"),
        (r"(?=(\uD800))\1", "d", "😀"),
        (r"((?=(a))a)|b", "d", "b"),
        (r"(?=(a|ab)+b)\1", "d", "abb"),
        (r"(?=)(?!)|a", "d", "a"),
        (r"(?!(?=(a))\1b)a", "d", "a"),
        (r"(?=(?!(a)b)a)a", "d", "a"),
        (r"(?=(a))\1?b", "d", "ab"),
        (r"(?:(?=(?<x>a))a|(?=(?<x>b))b){2}\k<x>", "d", "abb"),
        (r"(?=(?<x>µ))\k<x>", "di", "Μ"),
        (r"(?=(?<x>ſ))\k<x>", "di", "S"),
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
fn lookahead_atomic_choices_keep_external_ranges_and_negative_captures_undefined() {
    check(
        r"let a=/a(?=(?<x>b))/d.exec('ab');a[0]==='a'&&a.groups.x==='b'&&a.indices[0][1]===1&&a.indices.groups.x===a.indices[1]&&a.indices[1][0]===1&&a.indices[1][1]===2",
    );
    check(r"/(?=(a|ab))\1b$/.exec('abb')===null&&/(?=(ab|a))\1b$/.exec('abb')[1]==='ab'");
    check(
        r"let a=/(?=(a+))a*b\1/d.exec('baabac');a[0]==='aba'&&a.index===2&&a[1]==='a'&&a.indices[1][0]===2",
    );
    check(
        r"let a=/(?!(?<x>a)b)\k<x>a/d.exec('a'),b=/(?=(?!(?<x>a)b)a)a/d.exec('a');a.groups.x===undefined&&a.indices.groups.x===undefined&&b.groups.x===undefined&&b.indices.groups.x===undefined",
    );
    check(
        r"let a=/(?:(?=(?<x>a|b))a|b)+c/d.exec('abc'),b=/(?:(?=(?<x>a|b))\k<x>c)+/d.exec('acbc');a.groups.x===undefined&&a.indices.groups.x===undefined&&b[0]==='acbc'&&b.groups.x==='b'&&b.indices.groups.x[0]===2",
    );
    check(
        r"let a=/(?:(?=(?<x>a))a|(?=(?<x>b))b){2}\k<x>/d.exec('abb');a[0]==='abb'&&a[1]===undefined&&a.groups.x==='b'&&a.indices.groups.x===a.indices[2]",
    );
}

#[test]
fn lookahead_consumers_advance_empty_matches_and_preserve_prefix_captures() {
    check(
        r"let r=/(?=(?<x>a))/dg,a=[...'aba'.matchAll(r)];a.length===2&&a[0][0]===''&&a[0].groups.x==='a'&&a[0].indices[0][1]===0&&a[0].indices.groups.x[1]===1&&a[1].index===2&&r.lastIndex===0",
    );
    check(
        r"let r=/(?=(a))/gy;r.lastIndex=1;let a=r.exec('ba');a[0]===''&&a[1]==='a'&&r.lastIndex===1&&r.exec('ba')[1]==='a'",
    );
    check(
        r#"'aba'.replace(/(?=(?<x>a))/g,'<$<x>>')==='<a>ab<a>a'&&'aba'.search(/(?=(a))/)===0&&JSON.stringify('aba'.split(/(?=(a))/))==='["ab","a","a"]'"#,
    );
    check(
        r#"let a=[];let text='aa'.replace(/(?=(a))/g,(m,c,i,s)=>{a.push(m,c,i,s);return '_'});text==='_a_a'&&JSON.stringify(a)==='["","a",0,"aa","","a",1,"aa"]'"#,
    );
    check(
        r"let r=/(?<prefix>a)(?=\k<prefix>)/dy;r.lastIndex=1;let a=r.exec('qaa');a[0]==='a'&&a.groups.prefix==='a'&&r.lastIndex===2&&r.exec('qaa')===null&&r.lastIndex===0",
    );
}

#[test]
fn flat_lookahead_depth_copies_and_searches_survive_collection_without_quotas() {
    let mut realm = Realm::default();
    realm.eval("let r=new RegExp('(?='.repeat(100000)+'(?<x>a)'+')'.repeat(100000)+'\\\\k<x>','d'),copy=new RegExp(r)").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("let a=copy.exec('a');a[0]==='a'&&a.groups.x==='a'&&a.indices.groups.x===a.indices[1]&&a.indices[1][1]===1&&copy.source===r.source"),Ok(Value::Boolean(true)));
    assert_eq!(realm.eval("let many=/(?:(?=(?<x>a|b))\\k<x>c)+/d.exec('acbc'.repeat(10000));many[0].length===40000&&many.groups.x==='b'&&many.indices.groups.x[0]===39998"),Ok(Value::Boolean(true)));
    realm.collect(usize::MAX).unwrap();
}

#[test]
fn explicit_lookahead_work_aborts_before_last_index_and_javascript_handlers() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval("let marker=0,r=/(?=(?<x>a+))\\k<x>b/g;r.lastIndex=1;let text='a'.repeat(5000)")
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
fn lookbehind_unicode_and_possibly_empty_repetitions_keep_their_unsupported_boundary() {
    for source in [
        "/(?<=(a+))b/.exec('a')",
        "/(?<!(a+))b/.exec('a')",
        "/(?=a)a/u.exec('a')",
        "/(?=a)a/v.exec('a')",
        "/(?:(?<=(a+)))*/.exec('a')",
        "/(?:(?=(a))|b){2,3}/.exec('a')",
    ] {
        assert!(
            matches!(
                Realm::default().eval(source),
                Err(Error::Unsupported { .. })
            ),
            "{source}"
        );
    }
}
