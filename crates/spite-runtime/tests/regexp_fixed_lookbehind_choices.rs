//! Equal-width ordinary lookbehind alternatives and atomic results (22.2.2.8).
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
fn fixed_lookbehind_choice_result_snapshot() {
    let mut rows = String::new();
    for (source, flags, text) in [
        (r"(?<=a|b)c", "d", "bc"),
        (r"(?<!a|b)c", "d", "bc"),
        (r"(?<!a|b)c", "d", "qc"),
        (r"(?<=ab|cd)e", "d", "cde"),
        (r"(?<=(a|b))c", "d", "bc"),
        (r"(?<=(a)b|a(c))d", "d", "acd"),
        (r"(?<=((a)b|a(c)))d", "d", "acd"),
        (r"(?<!((a)b|a(c)))d", "d", "aqd"),
        (r"(?<=(a|b)(c|d))e", "d", "bde"),
        (r"(?<=(a|b))\1", "d", "bb"),
        (r"(?<=((a)|a))c\2", "d", "ac"),
        (r"(?<=(a|(a)))c\2", "d", "ac"),
        (r"(?<=()|())a", "d", "a"),
        (r"(?<!()|())a", "d", "a"),
        (r"(?<=(\b|^))a", "d", " a"),
        (r"(?<=(^a|\ba))b", "dm", "q\nab"),
        (r"(?<=([ab]){2}|(a){2})c", "d", "abc"),
        (r"(?<=((?:ab){2}|(?:ba){2}))c", "d", "babac"),
        (r"(?<=(?<=a|b))c", "d", "bc"),
        (r"(?<=(a|b)(?<!(c|d)))e", "d", "be"),
        (r"(?:(?<=(a|b))c|d)+e", "d", "abcdde"),
        (r"((?<=(a|b))){2}c\1\2", "d", "bcb"),
        (r"((?<=(a|b)))*c\1\2", "d", "bc"),
        (r"(?=(ab(?<=(a|b)(a|b))c))\1\2\3", "d", "abcab"),
        (r"(?<=(µ|Μ))a", "di", "Μa"),
        (r"(?<=(?<x>a)|(?<x>b))c", "d", "bc"),
        (r"(?<=(?<x>a)b|a(?<x>c))d", "d", "acd"),
        (r"(?<=[\uD800]|[\uDC00])c", "d", "lone surrogates"),
    ] {
        let source = JsString::from(source);
        let flags = JsString::from(flags);
        let input = if text == "lone surrogates" {
            JsString::from_code_units(vec![0xdc00, 0x63])
        } else {
            JsString::from(text)
        };
        let program = format!(
            "let r=new RegExp({source:?},{flags:?}),a=r.exec({input:?});JSON.stringify(a===null?{{match:null,lastIndex:r.lastIndex}}:{{matches:[...a],index:a.index,input:a.input,groups:a.groups,indices:a.indices,indicesGroups:a.indices.groups,lastIndex:r.lastIndex,source:r.source}})"
        );
        let Value::String(result) = Realm::default().eval(&program).unwrap() else {
            panic!("expected JSON")
        };
        writeln!(
            rows,
            "{source:?} flags={flags:?} input={input:?} {result:?}"
        )
        .unwrap();
    }
    insta::assert_snapshot!(rows);
}

#[test]
fn fixed_lookbehind_choices_partial_capture_undo_named_aliases_and_atomicity_are_exact() {
    check(
        r"let a=/(?<=(?<x>a)b|a(?<x>c))d\k<x>/d.exec('acdc');a[0]==='dc'&&a.index===2&&a[1]===undefined&&a[2]==='c'&&a.groups.x==='c'&&a.indices[1]===undefined&&a.indices.groups.x===a.indices[2]&&a.indices[2][0]===1&&a.indices[2][1]===2",
    );
    check(
        r"let a=/(?<=((a)b|a(c)))d/d.exec('acd');a[1]==='ac'&&a[2]===undefined&&a[3]==='c'&&a.indices[1][0]===0&&a.indices[1][1]===2&&a.indices[2]===undefined&&a.indices[3][0]===1",
    );
    check(
        r"let a=/(?<!((a)b|a(c)))d/d.exec('aqd');a.index===2&&a[1]===undefined&&a[2]===undefined&&a[3]===undefined&&a.indices[1]===undefined&&a.indices[2]===undefined&&a.indices[3]===undefined",
    );
    check(r"let r=/(?<=((a)|a))c\2/dy;r.lastIndex=1;r.exec('ac')===null&&r.lastIndex===0");
    check(
        r"let a=/(?<=(a|(a)))c\2/d.exec('ac');a[0]==='c'&&a[1]==='a'&&a[2]===undefined&&a.indices[2]===undefined",
    );
    check(
        r"let a=/(?<=()|())a/d.exec('a');a[1]===''&&a[2]===undefined&&a.indices[1][0]===0&&a.indices[1][1]===0&&a.indices[2]===undefined",
    );
    check(
        r"let a=/(?<=(?<x>a)|(?<x>b))c/d.exec('bc');a[1]===undefined&&a[2]==='b'&&a.groups.x==='b'&&a.indices.groups.x===a.indices[2]",
    );
    check(
        r"let a=/(?<=([ab]){2}|(a){2})c/d.exec('abc');a[1]==='a'&&a[2]===undefined&&a.indices[1][0]===0&&a.indices[2]===undefined",
    );
    check(
        r"let a=/((?<=(a|b))){2}c\1\2/d.exec('bcb'),b=/((?<=(a|b)))*c\1\2/d.exec('bc');a[1]===''&&a[2]==='b'&&a.indices[2][0]===0&&b[1]===undefined&&b[2]===undefined",
    );
}

#[test]
fn fixed_lookbehind_choice_consumers_global_sticky_empty_and_callbacks_preserve_ranges() {
    check(
        r"let r=/(?<=(?<x>a)|(?<x>b))c/dg,a=[...'acqbc'.matchAll(r)];a.length===2&&a[0].index===1&&a[0].groups.x==='a'&&a[1].index===4&&a[1].groups.x==='b'&&a[1].indices.groups.x[0]===3&&r.lastIndex===0",
    );
    check(
        r"let r=/(?<=(a|b))c/dy;r.lastIndex=1;let a=r.exec('bc');a[1]==='b'&&a.index===1&&r.lastIndex===2&&r.exec('bc')===null&&r.lastIndex===0",
    );
    check(
        r"let r=/(?<=(a|b))/dg,a=[...'ab'.matchAll(r)];a.length===2&&a[0].index===1&&a[0][1]==='a'&&a[1].index===2&&a[1][1]==='b'&&r.lastIndex===0",
    );
    check(
        r"let seen=[];let s='acqbc'.replace(/(?<=(?<x>a)|(?<x>b))c/g,(m,a,b,i,s,g)=>{seen.push(m,a,b,i,s,g.x);return '_'});s==='a_qb_'&&seen.length===12&&seen[1]==='a'&&seen[2]===undefined&&seen[3]===1&&seen[5]==='a'&&seen[7]===undefined&&seen[8]==='b'&&seen[9]===4&&seen[11]==='b'",
    );
    check(
        r"'acqbc'.replace(/(?<=(?<x>a)|(?<x>b))c/g,'<$<x>>')==='a<a>qb<b>'&&'acqbc'.search(/(?<=(a|b))c/)===1&&'acqbc'.split(/(?<=(a|b))c/).join('|')==='a|a|qb|b|'",
    );
}

#[test]
fn fixed_lookbehind_choice_deep_pending_frames_nested_assertions_copies_and_gc_are_unlimited() {
    let mut realm = Realm::default();
    realm.eval("let n=100000,r=new RegExp('(?<='+'(?:'.repeat(n)+'a'+'|a)'.repeat(n)+')(?<x>b)','d'),copy=new RegExp(r)").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("let a=copy.exec('ab');a.index===1&&a.groups.x==='b'&&a.indices.groups.x===a.indices[1]&&copy.source===r.source&&copy.exec('qb')===null"),Ok(Value::Boolean(true)));
    assert_eq!(realm.eval("let nested=new RegExp('(?<='.repeat(n)+'a|b'+')'.repeat(n)+'c');nested.exec('bc').index===1"),Ok(Value::Boolean(true)));
    assert_eq!(realm.eval("let many=/(?:(?<=(a|b))c|d)+e/.exec('bc'+'d'.repeat(10000)+'e');many.index===1&&many[0].length===10002&&many[1]===undefined"),Ok(Value::Boolean(true)));
    let maximum = usize::MAX;
    assert_eq!(realm.eval(&format!("new RegExp('(?<=a{{{maximum}}}|b{{{maximum}}})c').exec('abc')===null&&new RegExp('(?<!a{{{maximum}}}|b{{{maximum}}})c').exec('abc').index===2")),Ok(Value::Boolean(true)));
    realm.collect(usize::MAX).unwrap();
}

#[test]
fn explicit_fixed_lookbehind_branch_work_aborts_before_last_index_and_handlers() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval("let marker=0,r=/(?<=(a|b))c/g;r.lastIndex=1;let text='ab'.repeat(5000)")
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
fn differing_widths_repeated_choices_internal_references_and_lookahead_remain_unsupported() {
    for source in [
        r"/(?<=a|bb)c/.exec('bbc')",
        r"/(?<=(a|bb))c/.exec('bbc')",
        r"/(?<=(a|b){2})c/.exec('abc')",
        r"/(?<=(a|b)\1)c/.exec('aac')",
        r"/(?<=a|(?=a)b)c/.exec('bc')",
        r"/(?<=(a|b))c/u.exec('bc')",
        r"/(?<=(a|b))c/v.exec('bc')",
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
