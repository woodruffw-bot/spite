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
fn fixed_units_outside_lookbehind_result_snapshot() {
    let mut rows = String::new();
    for (source, flags, text) in [
        (r"(a)(?<=a{2}\1)b", "d", "aaaab"),
        (r"(a)(?<=[a]{2}\1)b", "d", "aaaab"),
        (r"(b)(?<=(ab){2}\1)c", "d", "ababbc"),
        (r"(b)(?<=\1(ab){2})c", "d", "bababc"),
        (r"(a)(?<=a{2}(\1){2})b", "d", "aaaab"),
        (r"(a)(?<=(a){2}(\1){2})b", "d", "aaaab"),
        (r"(a)(?<=(a\B){2}\1)b", "d", "aaab"),
        (r"(a)(?<=(^){2}\1)b", "d", "ab"),
        (r"(a)(?<=\1(\B){2})b", "d", "ab"),
        (r"(a)(?<=(\b){2}\1)b", "d", "ab"),
        (r"(a)(?<=(()\3){2}\1)b", "d", "ab"),
        (r"(a)(?<=(\1b){0}\1)b", "d", "ab"),
        (r"(a)(?<!(a){2}\1q)b", "d", "aaab"),
        (r"(a)(?<!a{2}\1)b", "d", "aaab"),
        (r"(a)(?<!a{2}\1q)b", "d", "aaab"),
        (r"(?<=(a){2}\2)(a)", "d", "aaa"),
        (r"(a(?<=a{2}\1))b", "d", "aaab"),
        (r"()(?<=(a){2}\1)b", "d", "aab"),
        (r"()(?<=(\b){2}\1)b", "d", "b"),
        (r"(a)(?<=(a){0}\1)b", "d", "ab"),
        (r"(a)(?<=(ab){0}\1)b", "d", "ab"),
        (r"(a)(?<=(a){2}?\1)b", "d", "aaab"),
        (r"(a)(?<=.{2}\1)b", "d", "qqab"),
        (r"(a)(?<=[^b]{2}\1)b", "d", "aaab"),
        (r"(a)(?<=a{2}\1(?=b))b", "d", "aaab"),
        (r"(a)(?<=(?<=a)a{2}\1)b", "d", "aaab"),
        (r"(a)(?<=a{2}\1(?=(b)))b", "d", "aaab"),
        (r"(?:(a|b)(?<=[ab]{2}\1))+c", "d", "aabbabc"),
        (r"(µ)(?<=(µ){2}\1)Μ", "di", "µµµΜ"),
        (r"([\uD800])(?<=[\uD800]{2}\1)[\uDC00]", "d", "surrogates"),
        (r"(a)(?<=(a){30}\1)b", "d", "ab"),
        (r"(a)(?<!(a){30}\1)b", "d", "ab"),
        (r"(?<x>a)(?<=(?<y>a){2}\k<x>)b", "d", "aaab"),
        (r"(?<x>b)(?<=\k<x>(?<y>ab){2})c", "d", "bababc"),
        (r"(?<x>a)(?<!(?<y>a){2}\k<x>q)b", "d", "aaab"),
        (r"(?<x>a)(?<=a{2}\k<x>(?=(?<y>b)))b", "d", "aaab"),
        (r"(?<=(a){2}\k<x>)(?<x>a)", "d", "aaa"),
        (r"(.)(?<=(a){2}\1)", "di", "aaA"),
        (r"(a)(?<=(^){2}\1)b", "dm", "q\nab"),
    ] {
        let source = JsString::from(source);
        let flags = JsString::from(flags);
        let input = if text == "surrogates" {
            JsString::from_code_units(vec![0xd800, 0xd800, 0xd800, 0xdc00])
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
fn names_aliases_backward_offsets_and_boundary_captures() {
    check(
        r"let a=/(?<x>b)(?<=(?<y>ab){2}\k<x>)c/d.exec('ababbc');a.index===4&&a.groups.x==='b'&&a.groups.y==='ab'&&a.indices.groups.y===a.indices[2]&&a.indices[2][0]===0&&a.indices[2][1]===2",
    );
    check(
        r"let a=/(?<x>b)(?<=\k<x>(?<y>ab){2})c/d.exec('bababc');a.index===4&&a.groups.y==='ab'&&a.indices.groups.y[0]===1&&a.indices.groups.y[1]===3",
    );
    check(
        r"let a=/(?<x>a)(?<!(?<y>a){2}\k<x>q)b/d.exec('aaab');a.groups.x==='a'&&a.groups.y===undefined&&a.indices.groups.y===undefined&&Object.hasOwn(a.groups,'y')",
    );
    check(
        r"let a=/(?<x>a)(?<=a{2}\k<x>(?=(?<y>b)))b/d.exec('aaab');a.index===2&&a.indices.groups.y[0]===3&&a.groups.y==='b'",
    );
    check(
        r"let a=/(?<x>a)(?<=\k<x>(?<y>\B){2})b/d.exec('ab');a.groups.y===''&&a.indices.groups.y[0]===1&&a.indices.groups.y[1]===1",
    );
    check(
        r"let a=/(?:(?<x>a)|(?<x>b))(?<=[ab]{2}\k<x>)c/d.exec('abbc');a.index===2&&a.groups.x==='b'&&a[1]===undefined&&a[2]==='b'&&a.indices.groups.x===a.indices[2]",
    );
}
#[test]
fn consumers_callbacks_sticky_global_and_empty_advancement() {
    check(
        r"let r=/(a)(?<=(a){2}\1)b/dg,a=[...'aaab aaab'.matchAll(r)];a.length===2&&a[1].index===7&&a[1][2]==='a'&&a[1].indices[2][0]===5&&r.lastIndex===0",
    );
    check(
        r"let r=/(a)(?<=a{2}\1)b/dy;r.lastIndex=2;let a=r.exec('aaab');a.index===2&&r.lastIndex===4&&r.exec('aaab')===null&&r.lastIndex===0",
    );
    check(
        r"let r=/()(?<=(){2}\1)/dg,a=[...'ab'.matchAll(r)];a.length===3&&a[2].index===2&&a[2][2]===''&&a[2].indices[2][0]===2&&r.lastIndex===0",
    );
    check(
        r"let seen=[];let s='aaab aaab'.replace(/(a)(?<=(a){2}\1)b/g,(m,c,d,i)=>{seen.push(c,d,i);return '_'});s==='aa_ aa_'&&seen.join('|')==='a|a|2|a|a|7'",
    );
    check(
        r"'aaab'.replace(/(?<x>a)(?<=(?<y>a){2}\k<x>)b/,'<$<y>>')==='aa<a>'&&'aaab'.search(/(a)(?<=a{2}\1)b/)===2&&'aaab'.split(/(a)(?<=a{2}\1)b/).join('|')==='aa|a|'",
    );
}
#[test]
fn deep_scopes_clones_collection_negative_restore_and_long_ranges_have_no_default_limits() {
    let mut realm = Realm::default();
    realm.eval(r"let n=100000,r=new RegExp('(a)(?<='+'('.repeat(n)+'a'+')'.repeat(n)+'{2}\\1)b','d'),copy=new RegExp(r)").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("let a=copy.exec('aaaab');a.index===3&&a.length===n+2&&a[1]==='a'&&a[n+1]==='a'&&a.indices[n+1][0]===1&&a.indices[n+1][1]===2&&copy.source===r.source"),Ok(Value::Boolean(true)));
    assert_eq!(realm.eval(r"let negative=new RegExp('(a)(?<!'+'('.repeat(n)+'a'+')'.repeat(n)+'{2}\\1q)b','d'),restored=negative.exec('aaaab');restored.index===3&&restored[1]==='a'&&restored[2]===undefined&&restored[n+1]===undefined"),Ok(Value::Boolean(true)));
    assert_eq!(realm.eval(r"let long=/(a+)(?<=(b){0}\1)b/.exec('a'.repeat(50000)+'b');long[0].length===50001&&long[1].length===50000&&long[2]===undefined"),Ok(Value::Boolean(true)));
    realm.collect(usize::MAX).unwrap();
}
#[test]
fn explicit_work_aborts_keep_last_index_and_bypass_handlers() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval(r"let marker=0,r=/(a)(?<=a{2}\1)b/g;r.lastIndex=1;let text='a'.repeat(5000)")
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
        r"/(a)(?<=[a]{1,2}\1)b/.exec('aaab')",
        r"/(?<x>a)(?<=(a\k<x>){1,2})b/.exec('aaab')",
        r"/(a)(?<=a{2}(\1){1,2})b/.exec('aaab')",
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
