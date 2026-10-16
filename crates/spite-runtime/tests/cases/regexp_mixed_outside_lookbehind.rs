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
fn mixed_outside_lookbehind_result_snapshot() {
    let mut rows = String::new();
    for (source, flags, text) in [
        (r"(a)(?<=(a\1){2})b", "d", "aaaaab"),
        (r"(a)(?<=(\1a){2})b", "d", "aaaaab"),
        (r"(a)(?<=(\1a){2}?)b", "d", "aaaaab"),
        (r"(a)(?<=(a\1){1})b", "d", "aaab"),
        (r"(a)(b)(?<=(\1\2){2})c", "d", "abababc"),
        (r"(a)(bb)(?<=(\1\2){2})c", "d", "abbabbabbc"),
        (r"(a)(b)(?<=(\2\1){2})c", "d", "abababc"),
        (r"(a)(b)(?<=(\1\2a){2})c", "d", "abababc"),
        (r"(?=(a))(?<=(\1b){2})a", "d", "ababa"),
        (r"(?=(ab))(?<=(\1b){2})a", "d", "abbabbab"),
        (r"(a)(?<=(\1\B){2})b", "d", "aaab"),
        (r"()(?<=(\1\b){2})b", "d", "b"),
        (r"()()(?<=(\1\2){2})b", "d", "b"),
        (r"()(?<=(\1\B){2})b", "d", "b"),
        (r"(a)(?<=(\1[ab]){2})b", "d", "aaaaab"),
        (r"(a)(?<=(.\1){2})b", "d", "aaaaab"),
        (r"(a)(?<=(\1.){2})b", "d", "aaaaab"),
        (r"(a)(?<=(^\1){2})b", "d", "aaab"),
        (r"(a)(?<=(\1$){2})b", "d", "aaab"),
        (r"(a)(?<!(\1a){2}q)b", "d", "aaaaab"),
        (r"(b)(?<!(\1a){2}q)c", "d", "bababc"),
        (r"(a)(?<!(a\1){2})b", "d", "aaaaab"),
        (r"(?<=(\2a){2})(a)", "d", "aaa"),
        (r"(a(?<=(\1a){2}))b", "d", "aaab"),
        (r"(?=(a))(?<=(\1[ab]\1){2})a", "d", "aaaaaaa"),
        (r"(a)(?<=(a\1){2}(?=b))b", "d", "aaaaab"),
        (r"(a)(?<=(?<=a)(a\1){2})b", "d", "aaaaab"),
        (r"(a)(?<=(a\1){2}(?=(b)))b", "d", "aaaaab"),
        (r"(?:(a|b)(?<=(.\1){2}))+c", "d", "aaaabbc"),
        (r"(µ)(?<=(.\1){2})Μ", "di", "µµµµµΜ"),
        (r"([\uD800])(?<=([\uD800]\1){2})[\uDC00]", "d", "surrogates"),
        (r"()()(?<=(\1\2\b){30})b", "d", "b"),
        (r"(?<x>a)(?<=(?<y>a\k<x>){2})b", "d", "aaaaab"),
        (
            r"(?<x>a)(?<z>bb)(?<=(?<y>\k<x>\k<z>){2})c",
            "d",
            "abbabbabbc",
        ),
        (r"(?<x>b)(?<!(?<y>\k<x>a){2}q)c", "d", "bababc"),
        (r"(?=(?<x>a))(?<=(?<y>\k<x>b){2})a", "d", "ababa"),
        (r"(?<=(?<y>\k<x>a){2})(?<x>a)", "d", "aaa"),
        (r"(.)(?<=(a\1){2})", "di", "aaaaA"),
        (r"(a)(?<=(\1[ab]){2})b", "dm", "q\naaaab"),
    ] {
        let source = JsString::from(source);
        let flags = JsString::from(flags);
        let input = if text == "surrogates" {
            JsString::from_code_units(vec![0xd800, 0xd800, 0xd800, 0xd800, 0xd800, 0xdc00])
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
fn names_aliases_different_widths_future_imports_and_scoped_negative_slots() {
    check(
        r"let a=/(?=(a))(?<=(?:\1b){2})a/d.exec('ababa');a.index===4&&a.length===2&&a.indices[1][0]===4",
    );
    check(
        r"let a=/(?<x>a)(?<=(?<y>a\k<x>){2})b/d.exec('aaaaab');a.index===4&&a.groups.y==='aa'&&a.indices.groups.y===a.indices[2]&&a.indices[2][0]===1&&a.indices[2][1]===3",
    );
    check(
        r"let a=/(?<x>a)(?<z>bb)(?<=(?<y>\k<x>\k<z>){2})c/d.exec('abbabbabbc');a.index===6&&a.groups.y==='abb'&&a.indices.groups.y[0]===3&&a.indices.groups.y[1]===6",
    );
    check(
        r"let a=/(?<x>b)(?<!(?<y>\k<x>a){2}q)c/d.exec('bababc');a.groups.x==='b'&&a.groups.y===undefined&&a.indices.groups.y===undefined&&Object.hasOwn(a.groups,'y')",
    );
    check(
        r"let a=/(?=(?<x>a))(?<=(?<y>\k<x>b){2})a/d.exec('ababa');a.index===4&&a.indices.groups.x[0]===4&&a.indices.groups.y[0]===0&&a.groups.y==='ab'",
    );
    check(
        r"let a=/(?:(?<x>a)|(?<x>b))(?<=(a\k<x>){2})c/d.exec('abababc');a.index===5&&a.groups.x==='b'&&a[1]===undefined&&a[2]==='b'&&a.indices.groups.x===a.indices[2]",
    );
    check(
        r"let a=/()()(?<=(\1\2\b){30})b/d.exec('b');a[3]===''&&a.indices[3][0]===0&&a.indices[3][1]===0",
    );
}
#[test]
fn consumers_callbacks_sticky_global_and_empty_advancement() {
    check(
        r"let r=/(a)(?<=(a\1){2})b/dg,a=[...'aaaaab aaaaab'.matchAll(r)];a.length===2&&a[1].index===11&&a[1][2]==='aa'&&a[1].indices[2][0]===8&&r.lastIndex===0",
    );
    check(
        r"let r=/(a)(?<=(a\1){2})b/dy;r.lastIndex=4;let a=r.exec('aaaaab');a.index===4&&r.lastIndex===6&&r.exec('aaaaab')===null&&r.lastIndex===0",
    );
    check(
        r"let r=/()()(?<=(\1\2){2})/dg,a=[...'ab'.matchAll(r)];a.length===3&&a[2].index===2&&a[2][3]===''&&a[2].indices[3][0]===2&&r.lastIndex===0",
    );
    check(
        r"let seen=[];let s='aaaaab aaaaab'.replace(/(a)(?<=(a\1){2})b/g,(m,c,d,i)=>{seen.push(c,d,i);return '_'});s==='aaaa_ aaaa_'&&seen.join('|')==='a|aa|4|a|aa|11'",
    );
    check(
        r"'aaaaab'.replace(/(?<x>a)(?<=(?<y>a\k<x>){2})b/,'<$<y>>')==='aaaa<aa>'&&'aaaaab'.search(/(a)(?<=(a\1){2})b/)===4&&'aaaaab'.split(/(a)(?<=(a\1){2})b/).join('|')==='aaaa|a|aa|'",
    );
}
#[test]
fn deep_scopes_clones_collection_negative_restore_large_counts_and_ranges_have_no_default_limits() {
    let mut realm = Realm::default();
    realm.eval(r"let n=100000,r=new RegExp('(b)(?<='+'('.repeat(n)+'a\\1'+')'.repeat(n)+'{2})c','d'),copy=new RegExp(r)").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("let a=copy.exec('qababc');a.index===4&&a.length===n+2&&a[1]==='b'&&a[n+1]==='ab'&&a.indices[n+1][0]===1&&a.indices[n+1][1]===3&&copy.source===r.source"),Ok(Value::Boolean(true)));
    assert_eq!(realm.eval(r"let negative=new RegExp('(b)(?<!'+'('.repeat(n)+'\\1a'+')'.repeat(n)+'{2}q)c','d'),restored=negative.exec('bababc');restored.index===4&&restored[1]==='b'&&restored[2]===undefined&&restored[n+1]===undefined"),Ok(Value::Boolean(true)));
    assert_eq!(realm.eval(r"let text='a'.repeat(50000),long=/(?=(a+))(?<=(\1b){2})a/dy;long.lastIndex=100002;let found=long.exec(text+'b'+text+'b'+text);found.index===100002&&found[1].length===50000&&found[2].length===50001&&found.indices[2][0]===0&&found.indices[1][0]===100002"),Ok(Value::Boolean(true)));
    assert_eq!(realm.eval(r"let huge=/()()(?<=(\1\2\b){999999999})b/d.exec('b');huge[3]===''&&huge.indices[3][0]===0"),Ok(Value::Boolean(true)));
    realm.collect(usize::MAX).unwrap();
}
#[test]
fn explicit_work_aborts_keep_last_index_and_bypass_handlers() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval(r"let marker=0,r=/(a)(?<=(a\1){2})b/g;r.lastIndex=1;let text='a'.repeat(5000)")
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
        r"/(?<=(\1{1,2}a{1,2}))a/.exec('a')",
        r"/(a)(?<=(?<=\1{1,2}))b/.exec('ab')",
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
