//! Fixed capture-free ordinary lookbehind with full input assertion context.
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
fn fixed_lookbehind_result_snapshot() {
    let mut rows = String::new();
    for (source, flags, text) in [
        (r"(?<=a)b", "d", "ab"),
        (r"(?<!a)b", "d", "ab"),
        (r"(?<=ab)c", "d", "qabc"),
        (r"(?<!ab)c", "d", "ac"),
        (r"(?<=)a", "d", "a"),
        (r"(?<!)a", "d", "a"),
        (r"(?<=^a)b", "d", "qab"),
        (r"(?<=^a)b", "dm", "q\nab"),
        (r"(?<=a$)", "d", "a"),
        (r"(?<=a$)", "dm", "a\nb"),
        (r"(?<=\ba)b", "d", " ab"),
        (r"(?<=a\b)b", "d", "ab"),
        (r"(?<=\Ba)b", "d", "qab"),
        (r"(?<=a\B)b", "d", "ab"),
        (r"(?<=[a-c])b", "d", "cb"),
        (r"(?<=[^a])b", "d", "ab"),
        (r"(?<=.)b", "d", "\nb"),
        (r"(?<=.)b", "ds", "\nb"),
        (r"(?<=µ)Μ", "di", "Μµ"),
        (r"(?<=\w)a", "di", "ſa"),
        (r"(?<=a(?:b))c", "d", "abc"),
        (r"(a)(?<=a)\1", "d", "aa"),
        (r"(?<=a)(?<x>b)\k<x>", "d", "abb"),
        (r"(?:(?<=a)b|c)+d", "d", "abcd"),
        (r"(?:(?<=a)|(?<!b)){2}b", "d", "ab"),
        (r"((?<=a)){2}b\1", "d", "ab"),
        (r"((?<=a))*b\1", "d", "ab"),
        (r"(?<=\uD800)b", "d", "lone surrogate b"),
    ] {
        let source = JsString::from(source);
        let flags = JsString::from(flags);
        let text = if text == "lone surrogate b" {
            JsString::from_code_units(vec![0xd800, 0x62])
        } else {
            JsString::from(text)
        };
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
fn fixed_lookbehind_keeps_outside_ranges_named_aliases_and_source_order_retries() {
    check(
        r"let a=/(?<=a)(?<x>b)\k<x>/d.exec('abb');a[0]==='bb'&&a.index===1&&a.groups.x==='b'&&a.indices.groups.x===a.indices[1]&&a.indices[1][0]===1&&a.indices[1][1]===2",
    );
    check(
        r"let a=/(?:(?<x>a)(?<!a)|(?<x>a)(?<=a))\k<x>/d.exec('aa');a[0]==='aa'&&a[1]===undefined&&a[2]==='a'&&a.groups.x==='a'&&a.indices[1]===undefined&&a.indices.groups.x===a.indices[2]",
    );
    check(
        r"let a=/((?<=a)){2}b\1/d.exec('ab'),b=/((?<=a))*b\1/d.exec('ab');a[1]===''&&a.indices[1][0]===1&&a.indices[1][1]===1&&b[1]===undefined&&b.indices[1]===undefined",
    );
    check(
        r"/(?<=a\b)b/.exec('ab')===null&&/(?<=a\B)b/.exec('ab').index===1&&/(?<=^a)b/m.exec('q\nab').index===3&&/(?<=a$)/m.exec('a\nb').index===1",
    );
    check(
        r"let a=/(?=(?<x>a(?<=a)b))\k<x>/d.exec('ab');a[0]==='ab'&&a.groups.x==='ab'&&a.indices.groups.x[1]===2",
    );
}

#[test]
fn fixed_lookbehind_consumers_global_sticky_empty_advancement_and_callback_arguments() {
    check(
        r"let r=/(?<=a)(?<x>b)/dg,a=[...'abqab'.matchAll(r)];a.length===2&&a[0].index===1&&a[1].index===4&&a[0].groups.x==='b'&&a[1].indices.groups.x[0]===4&&r.lastIndex===0",
    );
    check(
        r"let r=/(?<=a)b/dy;r.lastIndex=1;let a=r.exec('ab');a.index===1&&r.lastIndex===2&&r.exec('ab')===null&&r.lastIndex===0",
    );
    check(
        r"let r=/(?<=a)/dg,a=[...'aa'.matchAll(r)];a.length===2&&a[0][0]===''&&a[0].index===1&&a[1].index===2&&r.lastIndex===0",
    );
    check(
        r"let seen=[];let s='abqab'.replace(/(?<=a)(?<x>b)/g,(m,c,i,s,g)=>{seen.push(m,c,i,s,g.x);return '_'});s==='a_qa_'&&seen.length===10&&seen[0]==='b'&&seen[1]==='b'&&seen[2]===1&&seen[4]==='b'&&seen[7]===4",
    );
    check(
        r"'abqab'.replace(/(?<=a)(?<x>b)/g,'<$<x>>')==='a<b>qa<b>'&&'abqab'.search(/(?<=a)b/)===1&&'abqab'.split(/(?<=a)b/).join('|')==='a|qa|'",
    );
}

#[test]
fn fixed_lookbehind_deep_transparent_wrappers_long_bodies_copies_and_gc_are_unlimited() {
    let mut realm = Realm::default();
    realm.eval("let r=new RegExp('(?<='+'(?:'.repeat(100000)+'a'+')'.repeat(100000)+')(?<x>b)','d'),copy=new RegExp(r)").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("let a=copy.exec('ab');a[0]==='b'&&a.index===1&&a.groups.x==='b'&&a.indices.groups.x===a.indices[1]&&copy.source===r.source"), Ok(Value::Boolean(true)));
    assert_eq!(realm.eval("let n=10000,long=new RegExp('(?<='+'a'.repeat(n)+')b','dy');long.lastIndex=n;let last=long.exec('a'.repeat(n)+'b');last.index===n&&last[0]==='b'&&long.lastIndex===n+1"), Ok(Value::Boolean(true)));
    assert_eq!(realm.eval("let many=/(?:(?<=a)b|c)+d/.exec('ab'+'c'.repeat(10000)+'d');many.index===1&&many[0].length===10002"), Ok(Value::Boolean(true)));
    realm.collect(usize::MAX).unwrap();
}

#[test]
fn explicit_fixed_lookbehind_work_aborts_before_last_index_and_handlers() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval("let marker=0,r=/(?<=aaaaaaaa)b/g;r.lastIndex=1;let text='a'.repeat(5000)")
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
fn capturing_alternative_repeated_nested_and_unicode_lookbehind_remain_unsupported() {
    for source in [
        r"/(?<=(a))b/.exec('ab')",
        r"/(?<=a|b)c/.exec('ac')",
        r"/(?<=a+)b/.exec('ab')",
        r"/(?<=a{1})b/.exec('ab')",
        r"/(?<=(?=a))a/.exec('a')",
        r"/(?<=a)b/u.exec('ab')",
        r"/(?<=a)b/v.exec('ab')",
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
