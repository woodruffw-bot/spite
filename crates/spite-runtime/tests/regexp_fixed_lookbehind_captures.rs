//! Fixed lookbehind capture ranges and assertion checkpoint restoration.
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
fn fixed_lookbehind_capture_result_snapshot() {
    let mut rows = String::new();
    for (source, flags, text) in [
        (r"(?<=(a))b", "d", "ab"),
        (r"(?<!(a))b", "d", "qb"),
        (r"(?<=(a))\1", "d", "aa"),
        (r"(?<=(ab))c", "d", "abc"),
        (r"(?<=((a)b))c", "d", "abc"),
        (r"(?<=(a(b)))c", "d", "abc"),
        (r"(?<=(a{2}))b", "d", "aab"),
        (r"(?<=([a-c]{2}))d", "d", "abcd"),
        (r"(?<=()a)b", "d", "ab"),
        (r"(?<=a())b", "d", "ab"),
        (r"(?<=(\b)a)b", "d", " ab"),
        (r"(?<=(^a))b", "dm", "q\nab"),
        (r"(?<=a(?<=(a)))b", "d", "ab"),
        (r"(?<=((?<=a)b))c", "d", "abc"),
        (r"(?<=a(?<!(b)))b", "d", "ab"),
        (r"(?<!((a)b))c", "d", "aac"),
        (r"(?<=(a)(?<!(b)))b", "d", "ab"),
        (r"(?<=((a)(?<!(b))))b", "d", "ab"),
        (r"(?<=(?<x>a))b\k<x>", "d", "aba"),
        (r"(?:(?<=(?<x>a))b|(?<=(?<x>b))a)+c", "d", "abac"),
        (r"(?:(?<=(?<x>a))b|c)+d", "d", "abcd"),
        (r"(?:(?<=(a))|()){2}b\1\2", "d", "aba"),
        (r"((?<=(a))){2}b\1\2", "d", "aba"),
        (r"((?<=(a)))*b\1\2", "d", "aba"),
        (r"(?=(a(?<=(a))b))\1\2", "d", "aba"),
        (r"(?<=(µ))Μ", "di", "Μµ"),
        (r"(?<=(?<x>a)(?<!(?<y>b)))b", "d", "ab"),
        (r"(?<=(\uD800))b", "d", "lone surrogate b"),
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
fn fixed_lookbehind_captures_keep_before_match_ranges_negative_slots_and_outer_alias_retries() {
    check(
        r"let a=/(?<=(?<x>a))b\k<x>/d.exec('aba');a[0]==='ba'&&a.index===1&&a.groups.x==='a'&&a.indices.groups.x===a.indices[1]&&a.indices[1][0]===0&&a.indices[1][1]===1&&a.indices[0][0]===1",
    );
    check(
        r"let a=/(?:(?<=(?<x>a))bc|(?<=(?<x>a))b)/d.exec('ab');a[0]==='b'&&a[1]===undefined&&a[2]==='a'&&a.indices[1]===undefined&&a.indices.groups.x===a.indices[2]",
    );
    check(
        r"let a=/(?<=(?<x>a)(?<!(?<y>b)))b/d.exec('ab');a.groups.x==='a'&&a.groups.y===undefined&&a.indices.groups.x===a.indices[1]&&a.indices.groups.y===undefined",
    );
    check(
        r"let a=/(?<!((a)b))c/d.exec('aac');a[1]===undefined&&a[2]===undefined&&a.indices[1]===undefined&&a.indices[2]===undefined",
    );
    check(
        r"let a=/(?<=(?<x>aa)(?<!((?<y>a)b)))c/d.exec('aac');a.groups.x==='aa'&&a.groups.y===undefined&&a[2]===undefined&&a[3]===undefined&&a.indices.groups.x===a.indices[1]&&a.indices.groups.x[0]===0&&a.indices.groups.x[1]===2&&a.indices.groups.y===undefined",
    );
    check(
        r"let a=/((?<=(a))){2}b\1\2/d.exec('aba'),b=/((?<=(a)))*b\1\2/d.exec('aba');a[0]==='ba'&&a[1]===''&&a[2]==='a'&&a.indices[1][0]===1&&a.indices[2][0]===0&&b[1]===undefined&&b[2]===undefined",
    );
    check(
        r"let a=/(?:(?<=(?<x>a))b|c)+d/d.exec('abcd');a[0]==='bcd'&&a.groups.x===undefined&&a.indices.groups.x===undefined",
    );
    check(
        r"let a=/(?=(?<x>a(?<=(?<y>a))b))\k<x>\k<y>/d.exec('aba');a[0]==='aba'&&a.groups.x==='ab'&&a.groups.y==='a'&&a.indices.groups.x[1]===2&&a.indices.groups.y[0]===0",
    );
}

#[test]
fn fixed_lookbehind_captures_consumers_global_sticky_empty_matches_and_callback_arguments() {
    check(
        r"let r=/(?<=(?<x>a))b/dg,a=[...'abqab'.matchAll(r)];a.length===2&&a[0].index===1&&a[0].groups.x==='a'&&a[0].indices.groups.x[0]===0&&a[1].index===4&&a[1].indices.groups.x[0]===3&&r.lastIndex===0",
    );
    check(
        r"let r=/(?<=(?<x>a))b/dy;r.lastIndex=1;let a=r.exec('ab');a.index===1&&a.indices.groups.x[0]===0&&r.lastIndex===2&&r.exec('ab')===null&&r.lastIndex===0",
    );
    check(
        r"let r=/(?<=(?<x>a))/dg,a=[...'aa'.matchAll(r)];a.length===2&&a[0][0]===''&&a[0].index===1&&a[0].groups.x==='a'&&a[0].indices.groups.x[0]===0&&a[1].index===2&&a[1].indices.groups.x[0]===1&&r.lastIndex===0",
    );
    check(
        r"let seen=[];let s='abqab'.replace(/(?<=(?<x>a))b/g,(m,c,i,s,g)=>{seen.push(m,c,i,s,g.x);return '_'});s==='a_qa_'&&seen.length===10&&seen[0]==='b'&&seen[1]==='a'&&seen[2]===1&&seen[4]==='a'&&seen[7]===4",
    );
    check(
        r"'abqab'.replace(/(?<=(?<x>a))b/g,'<$<x>>')==='a<a>qa<a>'&&'abqab'.search(/(?<=(a))b/)===1&&'abqab'.split(/(?<=(a))b/).join('|')==='a|a|qa|a|'",
    );
}

#[test]
fn fixed_lookbehind_many_capture_ranges_nested_negatives_copies_and_gc_are_unlimited() {
    let mut realm = Realm::default();
    realm.eval("let n=100000,r=new RegExp('(?<='+'('.repeat(n)+'a'+')'.repeat(n)+')b','d'),copy=new RegExp(r)").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("let a=copy.exec('ab');a.length===n+1&&a.index===1&&a[1]==='a'&&a[n]==='a'&&a.indices[1][0]===0&&a.indices[n][1]===1&&a.indices[1]!==a.indices[n]&&copy.source===r.source"),Ok(Value::Boolean(true)));
    assert_eq!(realm.eval("let negative=new RegExp('(?<!'+'('.repeat(n)+'a'+')'.repeat(n)+'b)c','d'),last=negative.exec('aac');last.length===n+1&&last[1]===undefined&&last[n]===undefined&&last.indices[1]===undefined&&last.indices[n]===undefined"),Ok(Value::Boolean(true)));
    assert_eq!(realm.eval("let many=/(?:(?<=(?<x>a))b|c)+d/d.exec('ab'+'c'.repeat(10000)+'d');many.index===1&&many[0].length===10002&&many.groups.x===undefined"),Ok(Value::Boolean(true)));
    realm.collect(usize::MAX).unwrap();
}

#[test]
fn explicit_fixed_lookbehind_capture_work_aborts_before_last_index_and_handlers() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval("let marker=0,r=/(?<=(a{8}))b/g;r.lastIndex=1;let text='a'.repeat(5000)")
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
fn repeated_captures_choices_inner_references_lookahead_and_unicode_remain_unsupported() {
    for source in [
        r"/(?<=(a)+)b/.exec('ab')",
        r"/(?<=(a|b))c/.exec('ac')",
        r"/(?<=(a)\1)b/.exec('aab')",
        r"/(?<=(a(?=a)))b/.exec('ab')",
        r"/(?<=(a))b/u.exec('ab')",
        r"/(?<=(a))b/v.exec('ab')",
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
