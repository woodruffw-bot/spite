//! Count-one linear assertion units with Local, Open and Future references.
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
fn linear_reference_assertions_result_snapshot() {
    let mut rows = String::new();
    for (source, flags, text) in [
        (r"(?<=a(?=(b)\1))b", "d", "abb"),
        (r"(?<=a(?=(b)\1))b", "d", "abbbb"),
        (r"(?<=a(?=(b)(\1)\2))b", "d", "abbb"),
        (r"(?<=a(?=(b)\1\1))b", "d", "abbb"),
        (r"(?<=a(?=(\1b)))b", "d", "ab"),
        (r"(?<=a(?=(\2(b))))b", "d", "ab"),
        (r"(?<=a(?!((b)\2)q))b", "d", "abbx"),
        (r"(?<=(\2)(b))c", "d", "bbc"),
        (r"(?<=(\2)(\3)(b))c", "d", "bbbc"),
        (r"(?<=(a)\1)b", "d", "aab"),
        (r"(?<=(\1))a", "d", "a"),
        (r"(a)(?<=a(?=(b)\2\1))b", "d", "abbab"),
        (r"(a)(?<=(\3(b)\1))c", "d", "bbac"),
        (r"(ab)(?<=b(?=(\1)\2))a", "d", "ababab"),
        (r"(a)(?<=(\3)(\1))c", "d", "aaac"),
        (r"(?<=a(?=(?<x>b)\k<x>))b", "d", "abb"),
        (r"(?<=(?<x>\k<y>)(?<y>b))c", "d", "bbc"),
        (r"(?<=(?<x>a)\k<x>)b", "d", "aab"),
        (r"(?<=(?<x>\k<x>))a", "d", "a"),
        (r"(?<=a(?=(b)\1))b", "di", "aBb"),
        (r"(?<=a(?!(b)\1))b|(?<=a)b", "d", "abb"),
        (r"(?<!(\2)(b))c|c", "d", "bbc"),
    ] {
        let source = JsString::from(source);
        let flags = JsString::from(flags);
        let input = JsString::from(text);
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
fn previous_gap_programs_snapshot() {
    let mut rows = String::new();
    for source in [
        r"/(?<=(\1))a/.exec('a')",
        r"/(?<=a(?=(b)\1))b/.exec('abb')",
        r"/(?<=(a)\1)b/.exec('aab')",
        r"/(?<=(\1))a/.exec('a')",
        r"/(?<=a(?=(b)\1))b/.exec('abbbb')",
        r"/(?<=(\1))a/.exec('a')",
        r"/(?<=a(?=(b)\1))b/.exec('abb')",
        r"/(?<=(\1))a/.exec('a')",
    ] {
        let source = JsString::from(source);
        let program = format!("JSON.stringify({{result:eval({source:?})}})");
        let Value::String(result) = Realm::default().eval(&program).unwrap() else {
            panic!("expected JSON")
        };
        writeln!(rows, "{source:?} {result:?}").unwrap();
    }
    insta::assert_snapshot!(rows);
}
#[test]
fn local_open_future_named_ranges_imports_negative_commit_and_surrogates_are_exact() {
    check(
        r"let a=/(?<=a(?=(?<x>b)\k<x>))b/d.exec('abb');a.groups.x==='b'&&a.indices.groups.x===a.indices[1]&&a.indices[1][0]===1",
    );
    check(
        r"let a=/(?<=(?<x>\k<y>)(?<y>b))c/d.exec('bbc');a.groups.x==='b'&&a.groups.y==='b'&&a.indices.groups.x[0]===0&&a.indices.groups.y[0]===1",
    );
    check(
        r"let a=/(?<=(?<x>\k<x>))a/d.exec('a');a.groups.x===''&&a.indices.groups.x[0]===0&&a.indices.groups.x[1]===0",
    );
    check(
        r"let a=/(?<=a(?!(b)\1))b|(?<=a)b/d.exec('abb');a[1]===undefined&&a.indices[1]===undefined",
    );
    check(
        r"let a=/(?<!(\2)(b))c|c/d.exec('bbc');a[1]===undefined&&a[2]===undefined&&a.indices[1]===undefined",
    );
    check(
        r"let a=/(ab)(?<=b(?=(\1)\2))a/d.exec('ababab');a[1]==='ab'&&a[2]==='ab'&&a.indices[1][0]===0&&a.indices[2][0]===2",
    );
    check(
        r"let a=/(?<=a(?=([\uD800])\1))[\uD800]/d.exec('a\uD800\uD800\uDC00');a.index===1&&a.indices[1][0]===1",
    );
}
#[test]
fn consumers_global_sticky_empty_advance_and_callbacks_preserve_reference_positions() {
    check(
        r"let r=/(?<=a(?=(b)\1))b/dy;r.lastIndex=1;let a=r.exec('abb');a.index===1&&r.lastIndex===2&&r.exec('abb')===null&&r.lastIndex===0",
    );
    check(
        r"let r=/(?<=a(?=(b)\1))b/dg,a=[...'abb abb'.matchAll(r)];a.length===2&&a[1].index===5&&a[1].indices[1][0]===5&&r.lastIndex===0",
    );
    check(
        r"let a=[...'abb'.matchAll(/(?<=a(?=(b)\1))/dg)];a.length===1&&a[0].index===1&&a[0][0]===''&&a[0].indices[1][0]===1",
    );
    check(
        r"let seen=[];let s='abb abb'.replace(/(?<=a(?=(b)\1))b/g,(m,x,i)=>{seen.push(x,i);return '_'});s==='a_b a_b'&&seen.join('|')==='b|1|b|5'",
    );
    check(
        r"'abb'.search(/(?<=a(?=(b)\1))b/)===1&&'abb'.split(/(?<=a(?=(b)\1))b/).join('|')==='a|b|b'",
    );
}

#[test]
fn explicit_work_abort_keeps_last_index_and_bypasses_language_handlers() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval(r"let marker=0,r=/(?<=a(?=(b)\1))b/g;r.lastIndex=1;let text='b'.repeat(5000)+'abb'")
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
fn unproved_counts_choices_owner_reads_and_unicode_remain_pending() {
    for source in [
        r"/(?<=a(?=(b{1,2})\1))b/.exec('abbbb')",
        r"/(?<=(\1{1,2}a{1,2}))a/.exec('a')",
        r"/(?<=(a{1,2})\1)b/.exec('aab')",
        r"/(?<=a(?=(b|aa)\1))b/.exec('abb')",
        r"/(a)(?<=((b)(?=\2)))c/.exec('abc')",
        r"/(?<=a(?=(b)\1))b/u.exec('abb')",
        r"/(?<=a(?=(b)\1))b/v.exec('abb')",
    ] {
        assert!(
            crate::common::regexp_oracle::check(&mut Realm::default(), source),
            "{source}"
        );
    }
}
