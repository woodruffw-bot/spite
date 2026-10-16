//! Exact backward units with outside captures and dependent right-hand ranges.
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
fn dependent_outside_lookbehind_result_snapshot() {
    let mut rows = String::new();
    for (source, flags, text) in [
        (r"(a)(?<=(\3(b)\1){2})c", "d", "bbabbac"),
        (r"(a)(?<=(\3(b)\1){2}?)c", "d", "bbabbac"),
        (r"(ab)(?<=(\3(\1)){2})c", "d", "ababababc"),
        (r"(a+)(?<=(\3(\1)){2})b", "d", "aaaaaaaab"),
        (r"(a)(?<=(\3(\4(b))\1){2})c", "d", "bbbbabbbbac"),
        (r"(a)(?<=(\3(b)\1\3){2})c", "d", "bbabbac"),
        (r"(a)(?<=(\1\3(b)){2})c", "d", "ababc"),
        (r"(a)(?<!(\3(b)\1){2}q)c", "d", "bbabbaac"),
        (r"(a)(?<=(\3(\1)){0})b", "d", "ab"),
        (r"()(?<=(\3(\1)){2})b", "d", "b"),
        (r"(?<=(\2(\3)\3){2})(a)", "d", "a"),
        (r"(a(?<=(\3(\1)){2}))b", "d", "ab"),
        (r"(a)(?<=(\3(b)\1){2})c", "di", "bBAbbAc"),
        (r"(?<x>a)(?<=(?<y>\k<z>(?<z>b)\k<x>){2})c", "d", "bbabbac"),
        (r"(?<x>ab)(?<=(?<y>\k<z>(?<z>\k<x>)){2})c", "d", "ababababc"),
        (r"(?:(?<x>a)|(?<x>b))(?<=(\4(\k<x>)){2})c", "d", "bbbbc"),
        (r"(a+)(?<=(\3(\1)){2})b", "d", "aaaab"),
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
fn nested_dependencies_import_retries_named_aliases_and_negative_effects_keep_ranges() {
    check(
        r"let a=/(?<x>a)(?<=(?<y>\k<z>(?<z>b)\k<x>){2})c/d.exec('bbabbac');a.index===5&&a.groups.x==='a'&&a.groups.y==='bba'&&a.groups.z==='b'&&a.indices.groups.z===a.indices[3]&&a.indices[3][0]===1&&a.indices[1][0]===5",
    );
    check(
        r"let a=/(?<x>ab)(?<=(?<y>\k<z>(?<z>\k<x>)){2})c/d.exec('ababababc');a.groups.y==='abab'&&a.groups.z==='ab'&&a.indices[1][0]===6&&a.indices[2][0]===0&&a.indices[3][0]===2",
    );
    check(
        r"let a=/(a+)(?<=(\3(\1)){2})b/d.exec('aaaaaaaab');a.index===6&&a[1]==='aa'&&a[2]==='aaaa'&&a[3]==='aa'&&a.indices[3][0]===2",
    );
    check(
        r"let a=/(?:(?<x>a)|(?<x>b))(?<=(\4(\k<x>)){2})c/d.exec('bbbbc');a.index===3&&a[1]===undefined&&a[2]==='b'&&a[3]==='bb'&&a[4]==='b'&&a.groups.x==='b'&&a.indices.groups.x===a.indices[2]",
    );
    check(
        r"let a=/(a)(?<!(\3(b)\1){2}q)c/d.exec('bbabbaac');a.index===6&&a[1]==='a'&&a[2]===undefined&&a[3]===undefined&&a.indices[1][0]===6",
    );
    check(r"/(a)(?<=(\1\3(b)){2})c/.exec('ababc')===null");
    check(
        r"let a=/([\uD800])(?<=(\3([\uDC00])\1){2})c/d.exec('\uDC00\uDC00\uD800\uDC00\uDC00\uD800c');a.index===5&&a.indices[2][0]===0&&a.indices[3][0]===1",
    );
}

#[test]
fn consumers_global_sticky_empty_skip_and_callbacks_use_resolved_unit_ranges() {
    check(
        r"let r=/(a)(?<=(\3(b)\1){2})c/dy;r.lastIndex=5;let a=r.exec('bbabbac');a.index===5&&r.lastIndex===7&&r.exec('bbabbac')===null&&r.lastIndex===0",
    );
    check(
        r"let r=/(a)(?<=(\3(b)\1){2})c/dg,a=[...'bbabbac bbabbac'.matchAll(r)];a.length===2&&a[1].index===13&&a[1].indices[3][0]===9&&r.lastIndex===0",
    );
    check(
        r"let a=/(a)(?<=(\3(\1)){0})b/d.exec('ab');a[1]==='a'&&a[2]===undefined&&a.indices[3]===undefined",
    );
    check(
        r"let a=[...'ab'.matchAll(/()(?<=(\3(\1)){2})/dg)];a.length===3&&a[2].index===2&&a[2].indices[3][0]===2",
    );
    check(
        r"let seen=[];let s='bbabbac bbabbac'.replace(/(a)(?<=(\3(b)\1){2})c/g,(m,x,y,z,i)=>{seen.push(x,y,z,i);return '_'});s==='bbabb_ bbabb_'&&seen.join('|')==='a|bba|b|5|a|bba|b|13'",
    );
    check(
        r"'bbabbac'.search(/(a)(?<=(\3(b)\1){2})c/)===5&&'bbabbac'.split(/(a)(?<=(\3(b)\1){2})c/).join('|')==='bbabb|a|bba|b|'",
    );
}

#[test]
fn deep_captures_clones_collection_empty_dependency_chains_overflow_and_counts_stay_unlimited() {
    let mut realm = Realm::default();
    let count = usize::MAX / 2 + 1;
    assert_eq!(realm.eval(&format!("new RegExp('(?<=(?:ab){{{count}}})c').exec('abc')===null&&new RegExp('(?<!(?:ab){{{count}}})c').exec('abc').index===2")),Ok(Value::Boolean(true)));
    realm.eval(r"let n=100000,r=new RegExp('(a)(?<=(\\3'+'('.repeat(n)+'b'+')'.repeat(n)+'\\1){2})c','d'),copy=new RegExp(r)").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("let a=copy.exec('bbabbac');a.length===n+3&&a.index===5&&a[2]==='bba'&&a[n+2]==='b'&&a.indices[n+2][0]===1&&copy.source===r.source"),Ok(Value::Boolean(true)));
    assert_eq!(realm.eval(r"let neg=new RegExp('(a)(?<!(\\3'+'('.repeat(n)+'b'+')'.repeat(n)+'\\1){2}q)c','d'),b=neg.exec('bbabbaac');b[1]==='a'&&b[2]===undefined&&b[n+2]===undefined"),Ok(Value::Boolean(true)));
    let chain = (0..10000)
        .map(|i| format!("\\{}(", i + 3))
        .collect::<String>();
    let body = "(?<=(".to_owned() + &chain + r"\1" + &")".repeat(10000) + "){1})b";
    let empty = JsString::from(("()".to_owned() + &body).as_str());
    let nonempty = JsString::from(("(a)".to_owned() + &body).as_str());
    assert_eq!(realm.eval(&format!("let chain=new RegExp({empty:?},'d'),e=chain.exec('b');e.length===10003&&e[10002]===''&&e.indices[10002][0]===0")),Ok(Value::Boolean(true)));
    assert_eq!(
        realm.eval(&format!("new RegExp({nonempty:?}).exec('ab')===null")),
        Ok(Value::Boolean(true))
    );
    assert_eq!(realm.eval(r"let huge=new RegExp('()(?<=(\\3(\\1)){'+'9'.repeat(100)+'})b','d'),h=huge.exec('b');h[3]===''&&h.indices[3][0]===0"),Ok(Value::Boolean(true)));
    assert_eq!(realm.eval(r"let long=/(a)(?<=(\3(b)\1){50000})c/dy;long.lastIndex=149999;let found=long.exec('bba'.repeat(50000)+'c');found.index===149999&&found.indices[2][0]===0&&found.indices[3][0]===1"),Ok(Value::Boolean(true)));
    realm.collect(usize::MAX).unwrap();
}

#[test]
fn explicit_work_aborts_preserve_last_index_and_bypass_handlers() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm.eval(r"let marker=0,r=/(a)(?<=(\3(b)\1){2})c/g;r.lastIndex=1;let text='bba'.repeat(5000)+'c'").unwrap();
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
fn variable_counts_choices_dependent_children_and_unicode_remain_pending() {
    for source in [
        r"/(a)(?<=(\3(b)\1){1,2})c/.exec('bbabbac')",
        r"/(a)(?<=(\3(b)\1){2}|b)c/.exec('bbabbac')",
        r"/(a)(?<=(\3(b)\1){2}(?<=\3))c/.exec('bbabbac')",
        r"/(a)(?<=(\3(b)\1){2})c/u.exec('bbabbac')",
        r"/(a)(?<=(\3(b)\1){2})c/v.exec('bbabbac')",
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
