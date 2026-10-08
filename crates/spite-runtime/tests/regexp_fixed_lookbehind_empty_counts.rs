//! Exact-count empty bodies and local capture boundaries in lookbehind.
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
fn fixed_lookbehind_empty_count_result_snapshot() {
    let mut rows = String::new();
    for (source, flags, text) in [
        (r"(?<=(){2})a", "d", "qa"),
        (r"(?<!(){2})a", "d", "a"),
        (r"(?<=(){0})a", "d", "qa"),
        (r"(?<!(){0})a", "d", "a"),
        (r"(?<=(()){2})a", "d", "a"),
        (r"(?<=(()()){2})a", "d", "a"),
        (r"(?<=((?:){2}))a", "d", "a"),
        (r"(?<=(?:){2})a", "d", "a"),
        (r"(?<=(?:){0})a", "d", "a"),
        (r"(?<=(){2}a)b", "d", "ab"),
        (r"(?<=a(){2})b", "d", "ab"),
        (r"(?<=(((){2})a))b", "d", "ab"),
        (r"(?<=(((){0})a))b", "d", "ab"),
        (r"(?<!(){2}q)b", "d", "ab"),
        (r"(?<=(a)((){2}))b", "d", "ab"),
        (r"(?<=(){2}|())a", "d", "a"),
        (r"(?<=(){0}|())a", "d", "a"),
        (r"(?<=(?:(){2})(?<=a))b", "d", "ab"),
        (r"(?<=(?:(){2})(?<!b))a", "d", "qa"),
        (r"(?<=(){2})a\1", "d", "a"),
        (r"(?:(?<=(){2})a|b)+c", "d", "aabbc"),
        (r"((?<=(){2})){2}a\1\2", "d", "a"),
        (r"((?<=(){2}))*a\1\2", "d", "a"),
        (r"(?<=((?:){2}µ))Μ", "di", "ΜΜ"),
        (r"(?<=(?<x>){2})a", "d", "qa"),
        (r"(?<=(?<x>){0})a", "d", "qa"),
        (r"(?<=(){2})", "d", "a"),
        (r"(?<=(){2}[\uD800])b", "d", "lone surrogates"),
    ] {
        let source = JsString::from(source);
        let flags = JsString::from(flags);
        let input = if text == "lone surrogates" {
            JsString::from_code_units(vec![0xd800, 0x62])
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
fn empty_count_lookbehind_local_boundaries_named_aliases_zero_slots_and_rollback_are_exact() {
    check(
        r"let a=/(?<=(?<x>){2}a)b/d.exec('qab');a.index===2&&a.groups.x===''&&a.indices.groups.x===a.indices[1]&&a.indices[1][0]===1&&a.indices[1][1]===1",
    );
    check(
        r"let a=/(?<=a(?<x>){2})b/d.exec('qab');a.index===2&&a.groups.x===''&&a.indices.groups.x[0]===2&&a.indices.groups.x[1]===2",
    );
    check(
        r"let a=/(?<=(((){0})a))b/d.exec('ab');a[1]==='a'&&a[2]===''&&a[3]===undefined&&a.indices[2][0]===0&&a.indices[2][1]===0&&a.indices[3]===undefined",
    );
    check(
        r"let a=/(?<=(?<x>){0})a/d.exec('qa');a.index===1&&a.groups.x===undefined&&a[1]===undefined&&a.indices.groups.x===undefined",
    );
    check(
        r"let a=/(?<!((?<x>)){2}q)b/d.exec('ab');a.index===1&&a[1]===undefined&&a[2]===undefined&&a.groups.x===undefined&&a.indices[1]===undefined&&a.indices.groups.x===undefined",
    );
    check(
        r"let a=/(?<=(?<x>){2}|(?<x>))a\k<x>/d.exec('a'),b=/(?<=(?<x>){0}|(?<x>))a\k<x>/d.exec('a');a[1]===''&&a[2]===undefined&&a.groups.x===''&&a.indices.groups.x===a.indices[1]&&b[1]===undefined&&b[2]===undefined&&b.groups.x===undefined",
    );
    check(
        r"let a=/((?<=(){2})){2}a\1\2/d.exec('a'),b=/((?<=(){2}))*a\1\2/d.exec('a');a[1]===''&&a[2]===''&&a.indices[1][0]===0&&a.indices[2][0]===0&&b[1]===undefined&&b[2]===undefined",
    );
    check(
        r"let a=/(?<=(?:(){2})(?<=a))b/d.exec('ab');a.index===1&&a[1]===''&&a.indices[1][0]===1&&a.indices[1][1]===1",
    );
}

#[test]
fn empty_count_lookbehind_consumers_global_sticky_empty_and_callbacks_preserve_boundaries() {
    check(
        r"let r=/(?<=(?<x>){2})a/dg,a=[...'qa a'.matchAll(r)];a.length===2&&a[0].index===1&&a[1].index===3&&a[1].groups.x===''&&a[1].indices.groups.x[0]===3&&r.lastIndex===0",
    );
    check(
        r"let r=/(?<=(){2})a/dy;r.lastIndex=1;let a=r.exec('qa');a[1]===''&&a.index===1&&r.lastIndex===2&&r.exec('qa')===null&&r.lastIndex===0",
    );
    check(
        r"let r=/(?<=(){2})/dg,a=[...'ab'.matchAll(r)];a.length===3&&a[0].index===0&&a[1].index===1&&a[2].index===2&&a[2][1]===''&&a[2].indices[1][0]===2&&r.lastIndex===0",
    );
    check(
        r"let seen=[];let s='qa a'.replace(/(?<=(?<x>){2})a/g,(m,c,i,s,g)=>{seen.push(m,c,i,s,g.x);return '_'});s==='q_ _'&&seen.length===10&&seen[0]==='a'&&seen[1]===''&&seen[2]===1&&seen[4]===''&&seen[7]===3&&seen[9]===''",
    );
    check(
        r"'qa a'.replace(/(?<=(?<x>){2})a/g,'<$<x>>')==='q<> <>'&&'qa'.search(/(?<=(){2})a/)===1&&'qa a'.split(/(?<=(){2})a/).join('|')==='q|| ||'",
    );
}

#[test]
fn empty_count_lookbehind_deep_slots_huge_counts_copies_gc_and_parent_loops_are_unlimited() {
    let mut realm = Realm::default();
    let maximum = usize::MAX;
    realm.eval(&format!("let n=100000,r=new RegExp('(?<='+'('.repeat(n)+')'.repeat(n)+'{{{maximum}}})a','d'),copy=new RegExp(r)")).unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("let a=copy.exec('qa');a.index===1&&a.length===n+1&&a[1]===''&&a[n]===''&&a.indices[1][0]===1&&a.indices[n][1]===1&&a.indices[1]!==a.indices[n]&&copy.source===r.source"),Ok(Value::Boolean(true)));
    assert_eq!(realm.eval("let negative=new RegExp('(?<!'+'('.repeat(n)+')'.repeat(n)+'{2}q)b','d'),last=negative.exec('ab');last.length===n+1&&last[1]===undefined&&last[n]===undefined&&last.indices[1]===undefined&&last.indices[n]===undefined"),Ok(Value::Boolean(true)));
    assert_eq!(realm.eval(&format!("new RegExp('(?<=(?:){{{maximum}}})a').exec('qa').index===1&&new RegExp('(?<!(){{{maximum}}})a').exec('a')===null")),Ok(Value::Boolean(true)));
    assert_eq!(realm.eval("let many=/(?:(?<=(){2})a|b)+c/.exec('a'+'b'.repeat(10000)+'c');many.index===0&&many[0].length===10002&&many[1]===undefined"),Ok(Value::Boolean(true)));
    realm.collect(usize::MAX).unwrap();
}

#[test]
fn explicit_empty_count_lookbehind_work_aborts_before_last_index_and_handlers() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval("let marker=0,r=/(?<=(){2})a/g;r.lastIndex=1;let text='b'.repeat(5000)")
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
fn variable_counts_nested_lookaround_repeated_choices_references_and_counts_remain_unsupported() {
    for source in [
        r"/(?<=((?=a+)){1,2})a/.exec('a')",
        r"/(?<=((?=a+)){2})a/.exec('a')",
        r"/(?<=(|a){2})a/.exec('a')",
        r"/(?<=(\2){1,2})a()/.exec('a')",
        r"/(?<=((a{0,2}){2}){2})a/.exec('a')",
        r"/(?<=(){2})a/u.exec('a')",
        r"/(?<=(){2})a/v.exec('a')",
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
