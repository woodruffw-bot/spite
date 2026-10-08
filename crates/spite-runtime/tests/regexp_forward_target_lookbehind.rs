//! Fixed right-hand same-unit capture references in ordinary lookbehind.
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
fn forward_target_lookbehind_result_snapshot() {
    let mut rows = String::new();
    for (source, flags, text) in [
        (r"(?<=(\2(a)){2})b", "d", "aaaab"),
        (r"(?<=(\2(a)){2}?)b", "d", "aaaab"),
        (r"(?<=(\2([ab])){2})c", "d", "aabbc"),
        (r"(?<=(\2(ab)){2})c", "d", "ababababc"),
        (r"(?<=(\2(\3(a))){2})b", "d", "aaaaaaaab"),
        (r"(?<=(\2(a)\2){2})b", "d", "aaaab"),
        (r"(?<=(\2a()){2})b", "d", "aab"),
        (r"(?<=(\2(a)){0})b", "d", "b"),
        (r"(?<!(\2(a)){2}q)b", "d", "aaaaxb"),
        (r"(?<=((a)\1){2}(?=(\4(b)){2}))b", "d", "aabbb"),
        (r"(?=(\2(a)){2})a", "d", "aa"),
        (r"(b)(?<=(\3(a)){2}\1)c", "d", "aaaabc"),
        (r"(?<=(\2(a)){2})b", "di", "aAaAb"),
        (r"(?<=(?<y>\k<x>(?<x>a)){2})b", "d", "aaaab"),
        (r"(?<=(?<z>\k<y>(?<y>\k<x>(?<x>a))){2})b", "d", "aaaaaaaab"),
        (r"(?<=(\2([\uD800])){2})[\uDC00]", "d", "surrogates"),
        (r"(?<=(\2(a)){2})b", "d", "aab"),
    ] {
        let source = JsString::from(source);
        let flags = JsString::from(flags);
        let input = if text == "surrogates" {
            JsString::from_code_units(vec![0xd800, 0xd800, 0xd800, 0xd800, 0xdc00])
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
fn nested_future_names_empty_slots_outside_imports_and_matching_directions_keep_exact_ranges() {
    check(r"/(?<=((\3a)(b)){2})c/.exec('ababc')===null");
    check(
        r"let a=/(?<=(?<y>\k<x>(?<x>a)){2})b/d.exec('aaaab');a.index===4&&a.groups.y==='aa'&&a.groups.x==='a'&&a.indices.groups.x===a.indices[2]&&a.indices.groups.x[0]===1&&a.indices.groups.y[0]===0",
    );
    check(
        r"let a=/(?<=(?<z>\k<y>(?<y>\k<x>(?<x>a))){2})b/d.exec('aaaaaaaab');a.groups.z==='aaaa'&&a.groups.y==='aa'&&a.groups.x==='a'&&a.indices.groups.y[0]===2&&a.indices.groups.x[0]===3",
    );
    check(
        r"let a=/(?<=(\2a()){2})b/d.exec('aab');a[1]==='a'&&a[2]===''&&a.indices[2][0]===1&&a.indices[2][1]===1",
    );
    check(
        r"let a=/(?<!(?<y>\k<x>(?<x>a)){2}q)b/d.exec('aaaaxb');a.index===5&&a.groups.x===undefined&&a.groups.y===undefined&&a.indices.groups.x===undefined&&Object.hasOwn(a.groups,'y')",
    );
    check(
        r"let a=/(?<=((a)\1){2}(?=(\4(b)){2}))b/d.exec('aabbb'),b=/(?=(\2(a)){2})a/d.exec('aa');a.indices[3][0]===3&&a.indices[4][0]===3&&b.indices[1][0]===1&&b.indices[2][0]===1",
    );
    check(
        r"let a=/(b)(?<=(\3(a)){2}\1)c/d.exec('aaaabc');a.index===4&&a[1]==='b'&&a[2]==='aa'&&a.indices[3][0]===1",
    );
}
#[test]
fn consumers_sticky_global_zero_counts_and_callbacks_preserve_right_capture_positions() {
    check(
        r"let r=/(?<=(\2(a)){2})b/dy;r.lastIndex=4;let a=r.exec('aaaab');a.index===4&&r.lastIndex===5&&r.exec('aaaab')===null&&r.lastIndex===0",
    );
    check(
        r"let r=/(?<=(\2(a)){2})b/dg,a=[...'aaaab aaaab'.matchAll(r)];a.length===2&&a[1].index===10&&a[1].indices[2][0]===7&&r.lastIndex===0",
    );
    check(
        r"let r=/(?<=(\2(a)){0})/dg,a=[...'ab'.matchAll(r)];a.length===3&&a[2].index===2&&a[2][1]===undefined&&a[2].indices[2]===undefined",
    );
    check(
        r"let seen=[];let s='aaaab aaaab'.replace(/(?<=(\2(a)){2})b/g,(m,x,y,i)=>{seen.push(x,y,i);return '_'});s==='aaaa_ aaaa_'&&seen.join('|')==='aa|a|4|aa|a|10'",
    );
    check(
        r"'aaaab'.replace(/(?<=(?<y>\k<x>(?<x>a)){2})b/,'<$<y>>')==='aaaa<aa>'&&'aaaab'.search(/(?<=(\2(a)){2})b/)===4&&'aaaab'.split(/(?<=(\2(a)){2})b/).join('|')==='aaaa|aa|a|'",
    );
}
#[test]
fn deep_scopes_clones_collection_completed_negative_restore_and_large_counts_are_unlimited() {
    let mut realm = Realm::default();
    realm.eval(r"let n=100000,r=new RegExp('(?<=(\\2'+'('.repeat(n)+'a'+')'.repeat(n)+'){2})b','d'),copy=new RegExp(r)").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("let a=copy.exec('aaaab');a.index===4&&a.length===n+2&&a[1]==='aa'&&a[n+1]==='a'&&a.indices[n+1][0]===1&&copy.source===r.source"),Ok(Value::Boolean(true)));
    assert_eq!(realm.eval(r"let neg=new RegExp('(?<!(\\2'+'('.repeat(n)+'a'+')'.repeat(n)+'){2}q)b','d'),restored=neg.exec('aaaaxb');restored.index===5&&restored[1]===undefined&&restored[n+1]===undefined"),Ok(Value::Boolean(true)));
    assert_eq!(realm.eval(r"let long=/(?<=(\2(a)){50000})b/dy;long.lastIndex=100000;let found=long.exec('a'.repeat(100000)+'b');found.index===100000&&found.indices[1][0]===0&&found.indices[2][0]===1"),Ok(Value::Boolean(true)));
    realm.collect(usize::MAX).unwrap();
}
#[test]
fn explicit_work_aborts_preserve_last_index_and_bypass_handlers() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval(r"let marker=0,r=/(?<=(\2(a)){2})b/g;r.lastIndex=1;let text='a'.repeat(5000)+'b'")
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
fn variable_counts_mixed_outside_units_choices_dependent_children_and_unicode_remain_pending() {
    for source in [
        r"/(?<=(\2(a)){1,2})b/.exec('aaaab')",
        r"/(a)(?<=(\1\3(b)){2})c/.exec('ababc')",
        r"/(?<=(\2(a)){2}|a)b/.exec('aaaab')",
        r"/(?<=(\2(a)){2}(?<=\2))b/.exec('aaaab')",
        r"/(?<=(\2(a)){2})b/u.exec('aaaab')",
        r"/(?<=(\2(a)){2})b/v.exec('aaaab')",
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
