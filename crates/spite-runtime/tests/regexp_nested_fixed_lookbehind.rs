//! Flat nested fixed capture-free lookbehind and absolute boundary positions.
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
fn nested_fixed_lookbehind_result_snapshot() {
    let mut rows = String::new();
    for (source, flags, text) in [
        (r"(?<=(?<=a))b", "d", "ab"),
        (r"(?<!(?<=a))b", "d", "ab"),
        (r"(?<=(?<!a))b", "d", "qb"),
        (r"(?<!(?<!a))b", "d", "ab"),
        (r"(?<=a(?<=a))b", "d", "ab"),
        (r"(?<=a(?<!a))b", "d", "ab"),
        (r"(?<!a(?<=a))b", "d", "qb"),
        (r"(?<=a(?<=b))c", "d", "abc"),
        (r"(?<=(?<=a)b)c", "d", "abc"),
        (r"(?<=a(?<=a)b)c", "d", "abc"),
        (r"(?<=a(?<!b)b)c", "d", "abc"),
        (r"(?<=a(?<=a{1})b{1})c", "d", "abc"),
        (r"(?<=a{2}(?<=a{2}))b", "d", "aab"),
        (r"(?<=a{2}(?<!a{2}))b", "d", "aab"),
        (r"(?<=(?<=^a)b)c", "dm", "q\nabc"),
        (r"(?<=(?<=\ba)b)c", "d", " abc"),
        (r"(?<=c(?<=\w))\w{3}", "d", "ab cdef"),
        (r"(?<=\B)(?<=c(?<=\w))\w{3}", "d", "ab cdef"),
        (r"(?<=.(?<=.))b", "ds", "\nb"),
        (r"(?<=µ(?<=µ))Μ", "di", "Μµ"),
        (r"(?<=a(?<=(?<=a)))b", "d", "ab"),
        (r"(?<=a(?<!(?<!a)))b", "d", "ab"),
        (r"(?<=a(?<=a$))", "d", "a"),
        (r"(?<=^a(?<=a))b", "dm", "q\nab"),
        (r"(?<=a(?<=a))(?<x>b)\k<x>", "d", "abb"),
        (r"(?:(?<=a(?<=a))b|c)+d", "d", "abccd"),
        (r"(?:(?<=a(?<=a))|(?<!b)){2}b", "d", "ab"),
        (r"(?<=\uD800(?<=\uD800))b", "d", "lone surrogate b"),
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
fn nested_fixed_lookbehind_child_positions_negation_and_outside_aliases_are_exact() {
    check(
        r"let a=/(?<=a(?<=a))(?<x>b)\k<x>/d.exec('abb');a[0]==='bb'&&a.index===1&&a.groups.x==='b'&&a.indices.groups.x===a.indices[1]&&a.indices[1][0]===1&&a.indices[1][1]===2",
    );
    check(
        r"let a=/(?:(?<x>a)(?<!a(?<=a))|(?<x>a)(?<=a(?<=a)))\k<x>/d.exec('aa');a[0]==='aa'&&a[1]===undefined&&a[2]==='a'&&a.indices[1]===undefined&&a.indices.groups.x===a.indices[2]",
    );
    check(
        r"let a=/((?<=a(?<=a))){2}b\1/d.exec('ab'),b=/((?<=a(?<=a)))*b\1/d.exec('ab');a[1]===''&&a.indices[1][0]===1&&a.indices[1][1]===1&&b[1]===undefined&&b.indices[1]===undefined",
    );
    check(
        r"/(?<=(?<=a))b/.test('ab')&&!/(?<!(?<=a))b/.test('ab')&&/(?<=(?<!a))b/.test('qb')&&/(?<!(?<!a))b/.test('ab')&&!/(?<=a(?<!a))b/.test('ab')",
    );
    check(
        r"/(?<=(?<=^a)b)c/m.exec('q\nabc').index===4&&/(?<=(?<=^a)b)c/.exec('q\nabc')===null&&/(?<=\B)(?<=c(?<=\w))\w{3}/.exec('ab cdef')[0]==='def'",
    );
    check(
        r"let a=/(?=(?<x>a(?<=a(?<=a))b))\k<x>/d.exec('ab');a[0]==='ab'&&a.groups.x==='ab'&&a.indices.groups.x[1]===2",
    );
}

#[test]
fn nested_fixed_lookbehind_consumers_global_sticky_and_empty_matches_preserve_positions() {
    check(
        r"let r=/(?<=a(?<=a))(?<x>b)/dg,a=[...'abqab'.matchAll(r)];a.length===2&&a[0].index===1&&a[1].index===4&&a[1].groups.x==='b'&&a[1].indices.groups.x[0]===4&&r.lastIndex===0",
    );
    check(
        r"let r=/(?<=a(?<=a))b/dy;r.lastIndex=1;let a=r.exec('ab');a.index===1&&r.lastIndex===2&&r.exec('ab')===null&&r.lastIndex===0",
    );
    check(
        r"let r=/(?<=a(?<=a))/dg,a=[...'aa'.matchAll(r)];a.length===2&&a[0][0]===''&&a[0].index===1&&a[1].index===2&&r.lastIndex===0",
    );
    check(
        r"let seen=[];let s='abqab'.replace(/(?<=a(?<=a))(?<x>b)/g,(m,c,i,s,g)=>{seen.push(m,c,i,s,g.x);return '_'});s==='a_qa_'&&seen.length===10&&seen[2]===1&&seen[4]==='b'&&seen[7]===4",
    );
    check(
        r"'abqab'.replace(/(?<=a(?<=a))(?<x>b)/g,'<$<x>>')==='a<b>qa<b>'&&'abqab'.search(/(?<=a(?<=a))b/)===1&&'abqab'.split(/(?<=a(?<=a))b/).join('|')==='a|qa|'",
    );
}

#[test]
fn deeply_nested_fixed_lookbehind_positive_negative_copies_and_gc_are_unlimited() {
    let mut realm = Realm::default();
    realm.eval("let r=new RegExp('(?<='.repeat(100000)+'a'+')'.repeat(100000)+'(?<x>b)','d'),copy=new RegExp(r)").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("let a=copy.exec('ab');a.index===1&&a.groups.x==='b'&&a.indices.groups.x===a.indices[1]&&copy.source===r.source"),Ok(Value::Boolean(true)));
    assert_eq!(realm.eval("let negative=new RegExp('(?<!'.repeat(100001)+'a'+')'.repeat(100001)+'b');negative.exec('ab')===null&&negative.exec('qb').index===1"),Ok(Value::Boolean(true)));
    assert_eq!(realm.eval("let many=/(?:(?<=a(?<=a))b|c)+d/.exec('ab'+'c'.repeat(10000)+'d');many.index===1&&many[0].length===10002"),Ok(Value::Boolean(true)));
    realm.collect(usize::MAX).unwrap();
}

#[test]
fn explicit_nested_fixed_lookbehind_work_aborts_before_last_index_and_handlers() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval("let marker=0,r=/(?<=a(?<=a))b/g;r.lastIndex=1;let text='a'.repeat(5000)")
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
fn captures_choices_variable_counts_references_inner_lookahead_and_unicode_remain_unsupported() {
    for source in [
        r"/(?<=(?<=(a+)))b/.exec('ab')",
        r"/(?<=(?<=a|bb))c/.exec('ac')",
        r"/(?<=a(?<=a+))b/.exec('ab')",
        r"/(?<=a(?=a))b/.exec('ab')",
        r"/(?<=(?<=\1))(a)/.exec('a')",
        r"/(?<=a(?<=a))b/u.exec('ab')",
        r"/(?<=a(?<=a))b/v.exec('ab')",
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
