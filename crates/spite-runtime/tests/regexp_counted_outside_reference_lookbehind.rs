//! Outside capture references in linear ordinary lookbehind.
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
fn counted_outside_reference_lookbehind_result_snapshot() {
    let mut rows = String::new();
    for (source, flags, text) in [
        (r"(a)(?<=\1{2})b", "d", "aaab"),
        (r"(a)(?<=\1{2}?)b", "d", "aaab"),
        (r"(a)(?<=(\1){2})b", "d", "aaab"),
        (r"(a)(?<=(\1\1){2})b", "d", "aaaaab"),
        (r"(..)(?<=\1{3})", "d", "ababab"),
        (r"(a)(?<=a\1{2})b", "d", "aaaab"),
        (r"(a)(?<=\1{2}a)b", "d", "aaaab"),
        (r"(a)(?<!\1{2})b", "d", "aaab"),
        (r"(a)(?<!(\1){2}q)b", "d", "aaab"),
        (r"(?<=\1{2})(a)", "d", "qa"),
        (r"(?<=(\2){2})a(a)", "d", "qa"),
        (r"(a(?<=\1{2}))b", "d", "qab"),
        (r"()(?<=(\1){2})b", "d", "qb"),
        (r"()(?<=\1{30})b", "d", "qb"),
        (r"(a)(?<=(\1){0}\1)b", "d", "qab"),
        (r"(a)(?<=(\1){1}\1)b", "d", "aaab"),
        (r"(a)(?<=(\1\1){2}?)b", "d", "aaaaab"),
        (r"(a)(?<=\1{2}\b)b", "d", "aaab"),
        (r"(a)(?<=\1{2}\B)b", "d", "aaab"),
        (r"(a)(?<=^\1{2})b", "d", "aab"),
        (r"(a)(?<=\1{2}(?=b))b", "d", "aaab"),
        (r"(a)(?<=\1{2}(?=(b)))b", "d", "aaab"),
        (r"(a)(?<=(?<=a)\1{2})b", "d", "aaab"),
        (r"(a)(?<=\1{2}(?<=a))b", "d", "aaab"),
        (r"(a)(?<=(a|b){0}\1{2})b", "d", "aaab"),
        (r"(?:(a|b)(?<=\1{2}))+c", "d", "aababc"),
        (r"(µ)(?<=\1{2})Μ", "di", "µµΜ"),
        (r"([\uD800])(?<=\1{2})[\uDC00]", "d", "surrogates"),
        (r"(a)(?<=(\1\1){1})b", "d", "aaab"),
        (r"(?=(a))(?<=\1{2})a", "d", "aa"),
        (r"(a)(?<=\1{30})b", "d", "ab"),
        (r"(a)(?<!\1{30})b", "d", "ab"),
        (r"(?<x>a)(?<=(?<y>\k<x>){2})b", "d", "aaab"),
        (r"(?:(?<x>a)|(?<x>b))(?<=\k<x>{1})c", "d", "bc"),
        (r"(?<x>a)(?<!(?<y>\k<x>){2}q)b", "d", "aaab"),
        (r"(?<x>a)(?<=\k<x>{1}(?=(?<y>b)))b", "d", "qab"),
        (r"(?<=\k<x>{2})(?<x>a)", "d", "qa"),
        (r"(.)(?<=(\1){2})", "di", "abB"),
        (r"(a)(?<=^\1)b", "dm", "q\nab"),
    ] {
        let source = JsString::from(source);
        let flags = JsString::from(flags);
        let input = if text == "surrogates" {
            JsString::from_code_units(vec![0xd800, 0xdc00])
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
fn names_aliases_before_after_match_ranges_and_negative_slots() {
    check(
        r"let a=/(?<=(\2){2})a()/d.exec('qa');a.index===1&&a[1]===''&&a[2]===''&&a.indices[1][0]===1&&a.indices[2][0]===2",
    );
    check(
        r"let a=/(?<x>a)(?<=(?<y>\k<x>){2})b/d.exec('aaab');a.index===2&&a.groups.x==='a'&&a.groups.y==='a'&&a.indices.groups.x===a.indices[1]&&a.indices.groups.y===a.indices[2]&&a.indices[2][0]===1&&a.indices[2][1]===2",
    );
    check(
        r"let a=/(?:(?<x>a)|(?<x>b))(?<=\k<x>{1})c/d.exec('bc');a.groups.x==='b'&&a[1]===undefined&&a[2]==='b'&&a.indices.groups.x===a.indices[2]",
    );
    check(
        r"let a=/(?<x>a)(?<!(?<y>\k<x>){2}q)b/d.exec('aaab');a.groups.x==='a'&&a.groups.y===undefined&&a.indices.groups.y===undefined&&Object.hasOwn(a.groups,'y')",
    );
    check(
        r"let a=/(?<x>a)(?<=\k<x>{1}(?=(?<y>b)))b/d.exec('qab');a.index===1&&a.indices.groups.x[0]===1&&a.indices.groups.y[0]===2&&a.groups.y==='b'",
    );
    check(
        r"let a=/(?<=\k<x>{2})(?<x>a)/d.exec('qa');a.index===1&&a.groups.x==='a'&&a.indices.groups.x[0]===1",
    );
}
#[test]
fn consumers_callbacks_sticky_global_and_empty_advancement() {
    check(
        r"let r=/(a)(?<=(\1){2})b/dg,a=[...'aaab aaab'.matchAll(r)];a.length===2&&a[1].index===7&&a[1][2]==='a'&&a[1].indices[2][0]===6&&r.lastIndex===0",
    );
    check(
        r"let r=/(a)(?<=\1{1})b/dy;r.lastIndex=1;let a=r.exec('qab');a.index===1&&r.lastIndex===3&&r.exec('qab')===null&&r.lastIndex===0",
    );
    check(
        r"let r=/()(?<=\1{30})/dg,a=[...'ab'.matchAll(r)];a.length===3&&a[2].index===2&&a[2][1]===''&&a[2].indices[1][0]===2&&r.lastIndex===0",
    );
    check(
        r"let seen=[];let s='aaab aaab'.replace(/(a)(?<=(\1){2})b/g,(m,c,d,i)=>{seen.push(c,d,i);return '_'});s==='aa_ aa_'&&seen.join('|')==='a|a|2|a|a|7'",
    );
    check(
        r"'aaab'.replace(/(?<x>a)(?<=(?<y>\k<x>){2})b/,'<$<y>>')==='aa<a>'&&'qab'.search(/(a)(?<=\1)b/)===1&&'qab'.split(/(a)(?<=\1)b/).join('|')==='q|a|'",
    );
}
#[test]
fn deep_scopes_clones_collection_negative_restore_and_long_imported_ranges_have_no_default_limits()
{
    assert_eq!(Realm::default().eval(r"let a=/()(?<=(\1){999999999})b/d.exec('qb');a.index===1&&a[2]===''&&a.indices[2][0]===1&&a.indices[2][1]===1"),Ok(Value::Boolean(true)));
    let mut realm = Realm::default();
    realm.eval(r"let n=100000,r=new RegExp('(a)(?<='+'('.repeat(n)+'\\1\\1'+')'.repeat(n)+'{2})b','d'),copy=new RegExp(r)").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("let a=copy.exec('aaaaab');a.index===4&&a.length===n+2&&a[1]==='a'&&a[n+1]==='aa'&&a.indices[n+1][0]===1&&a.indices[n+1][1]===3&&copy.source===r.source"),Ok(Value::Boolean(true)));
    assert_eq!(realm.eval(r"let negative=new RegExp('(a)(?<!'+'('.repeat(n)+'\\1\\1'+')'.repeat(n)+'{2}q)b','d'),restored=negative.exec('aaaaab');restored.index===4&&restored[1]==='a'&&restored[2]===undefined&&restored[n+1]===undefined"),Ok(Value::Boolean(true)));
    assert_eq!(realm.eval(r"let long=/(a+)(?<=\1{1})b/.exec('a'.repeat(50000)+'b');long[0].length===50001&&long[1].length===50000"),Ok(Value::Boolean(true)));
    realm.collect(usize::MAX).unwrap();
}
#[test]
fn explicit_work_aborts_keep_last_index_and_bypass_handlers() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval(r"let marker=0,r=/(a)(?<=\1{2})b/g;r.lastIndex=1;let text='a'.repeat(5000)")
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
fn internal_reads_dependent_children_variable_counts_choices_and_unicode_remain_unsupported() {
    for source in [
        r"/(?<=(\1))a/.exec('a')",
        r"/(a)(?<=(?<=\1))b/.exec('ab')",
        r"/(a)(?<=(?=\1{1,2})\1)b/.exec('ab')",
        r"/(a)(?<=\1|a)b/.exec('ab')",
        r"/(a)(?<=\1{1,2})b/.exec('aab')",
        r"/(a)(?<=\1+)b/.exec('ab')",
        r"/(a)(?<=\1)b/u.exec('ab')",
        r"/(a)(?<=\1)b/v.exec('ab')",
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
