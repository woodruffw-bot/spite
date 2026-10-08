//! Required and optional counts of empty and boundary bodies in ordinary lookbehind.

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
fn zero_width_lookbehind_count_result_snapshot() {
    let mut rows = String::new();
    for (source, flags, text) in [
        (r"(?<=()?)a", "d", "qa"),
        (r"(?<=()*?)a", "d", "qa"),
        (r"(?<=()+)a", "d", "qa"),
        (r"(?<=()+?)a", "d", "qa"),
        (r"(?<=(){0,2})a", "d", "qa"),
        (r"(?<=(){1,2}?)a", "d", "qa"),
        (r"(?<=(){2,})a", "d", "qa"),
        (r"(?<=(?:)*)a", "d", "qa"),
        (r"(?<=(()){1,3})a", "d", "qa"),
        (r"(?<=(()()){1,})a", "d", "qa"),
        (r"(?<=(^)?)a", "d", "qa"),
        (r"(?<=(^)+)a", "d", "qa"),
        (r"(?<=(^)+)a", "dm", "q\na"),
        (r"(?<=(\b){0,3})a", "d", "qa"),
        (r"(?<=(\b){1,3}?)a", "d", " a"),
        (r"(?<=(\B)+)a", "d", "qa"),
        (r"(?<=(^\b)+)a", "d", "a"),
        (r"(?<=(^$){0,2})a", "d", "qa"),
        (r"(?<!()+)a", "d", "a"),
        (r"(?<!(^)+)a", "d", "qa"),
        (r"(?<!()+q)b", "d", "ab"),
        (r"(?<=()+a)b", "d", "ab"),
        (r"(?<=a()*)b", "d", "ab"),
        (r"(?<=()+|())a", "d", "a"),
        (r"(?<=()*|())a", "d", "a"),
        (r"(?<=()+)a\1", "d", "a"),
        (r"(?<=(?:()+)(?<=a))b", "d", "ab"),
        (r"((?<=()+)){2}a\1\2", "d", "a"),
        (r"((?<=()+))*a\1\2", "d", "a"),
        (r"(?:(?<=()+)a|b)+c", "d", "aabbc"),
        (r"(?<=((?:)*µ))Μ", "di", "ΜΜ"),
        (r"(?<=(?<x>)+)a", "d", "qa"),
        (r"(?<=(?<x>)*)a", "d", "qa"),
        (r"(?<=()+)", "d", "a"),
        (r"(?<=()+[\uD800])b", "d", "lone surrogates"),
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
fn zero_width_counts_restore_names_and_distinguish_required_and_optional_captures() {
    check(
        r"let a=/(?<=(?<x>)+a)b/d.exec('qab'),b=/(?<=a(?<x>)*)b/d.exec('qab');a.groups.x===''&&a.indices.groups.x===a.indices[1]&&a.indices[1][0]===1&&b.groups.x===undefined&&b.indices.groups.x===undefined",
    );
    check(
        r"let a=/(?<=(^)?)a/d.exec('qa'),b=/(?<=(^)+)a/dm.exec('q\na');a[1]===undefined&&a.indices[1]===undefined&&b.index===2&&b[1]===''&&b.indices[1][0]===2",
    );
    check(
        r"let a=/(?<=(?<x>)+|(?<x>))a\k<x>/d.exec('a'),b=/(?<=(?<x>)*|(?<x>))a\k<x>/d.exec('a');a[1]===''&&a[2]===undefined&&a.groups.x===''&&a.indices.groups.x===a.indices[1]&&b[1]===undefined&&b[2]===undefined&&b.groups.x===undefined",
    );
    check(
        r"let a=/(?<!((?<x>))+q)b/d.exec('ab');a[1]===undefined&&a[2]===undefined&&a.groups.x===undefined&&a.indices.groups.x===undefined",
    );
    check(
        r"let a=/((?<=()+)){2}a\1\2/d.exec('a'),b=/((?<=()+))*a\1\2/d.exec('a');a[1]===''&&a[2]===''&&b[1]===undefined&&b[2]===undefined",
    );
    check(r"let a=/(?<=(?:()+)(?<=a))b/d.exec('ab');a.index===1&&a[1]===''&&a.indices[1][0]===1");
    check(
        r"let a=/(?<=(^\b)+)a/d.exec('a'),b=/(?<=(\b){1,3}?)a/d.exec(' a');a[1]===''&&b.index===1&&b.indices[1][0]===1&&/(?<=(^$){0,2})a/.test('qa')&&/(?<!(^)+)a/.test('qa')",
    );
}

#[test]
fn zero_width_count_consumers_keep_last_index_empty_advancement_and_callback_slots() {
    check(
        r"let r=/(?<=(?<x>)+)a/dg,a=[...'qa a'.matchAll(r)];a.length===2&&a[1].index===3&&a[1].groups.x===''&&a[1].indices.groups.x[0]===3&&r.lastIndex===0",
    );
    check(
        r"let r=/(?<=()+)a/dy;r.lastIndex=1;let a=r.exec('qa');a[1]===''&&a.index===1&&r.lastIndex===2&&r.exec('qa')===null&&r.lastIndex===0",
    );
    check(
        r"let r=/(?<=()*)/dg,a=[...'ab'.matchAll(r)];a.length===3&&a[2].index===2&&a[2][1]===undefined&&a[2].indices[1]===undefined&&r.lastIndex===0",
    );
    check(
        r"let seen=[];let s='qa a'.replace(/(?<=(?<x>)+)a/g,(m,c,i,s,g)=>{seen.push(m,c,i,s,g.x);return '_'});s==='q_ _'&&seen.length===10&&seen[1]===''&&seen[2]===1&&seen[4]===''&&seen[7]===3",
    );
    check(
        r"'qa a'.replace(/(?<=(?<x>)+)a/g,'<$<x>>')==='q<> <>'&&'qa'.search(/(?<=()+)a/)===1&&'qa a'.split(/(?<=()+)a/).join('|')==='q|| ||'",
    );
}

#[test]
fn zero_width_count_deep_slots_unrepresentable_counts_copies_and_gc_have_no_default_quotas() {
    let mut realm = Realm::default();
    realm.eval("let n=100000,huge='184467440737095516160000000000000000000',r=new RegExp('(?<='+'('.repeat(n)+')'.repeat(n)+'{'+huge+',})a','d'),copy=new RegExp(r)").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("let a=copy.exec('qa');a.index===1&&a.length===n+1&&a[1]===''&&a[n]===''&&a.indices[1][0]===1&&a.indices[n][1]===1&&a.indices[1]!==a.indices[n]&&copy.source===r.source"),Ok(Value::Boolean(true)));
    assert_eq!(realm.eval("let negative=new RegExp('(?<!'+'('.repeat(n)+')'.repeat(n)+'+q)b','d'),b=negative.exec('ab');b.length===n+1&&b[1]===undefined&&b[n]===undefined&&b.indices[n]===undefined"),Ok(Value::Boolean(true)));
    assert_eq!(realm.eval("let c=new RegExp('(?<=(^){'+huge+',})a','d'),d=new RegExp('(?<=(^){0,'+huge+'})a','d');c.exec('a')[1]===''&&c.exec('qa')===null&&d.exec('qa')[1]===undefined"),Ok(Value::Boolean(true)));
    assert_eq!(realm.eval("let many=/(?:(?<=()+)a|b)+c/.exec('a'+'b'.repeat(10000)+'c');many.index===0&&many[0].length===10002&&many[1]===undefined"),Ok(Value::Boolean(true)));
    realm.collect(usize::MAX).unwrap();
}

#[test]
fn explicit_zero_width_count_work_aborts_before_last_index_and_handlers() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval("let marker=0,r=/(?<=()+)a/g;r.lastIndex=1;let text='b'.repeat(5000)")
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
fn consuming_variable_counts_repeated_choices_references_nested_counts_and_lookaround_are_pending()
{
    for source in [
        r"/(?<=a{1,2})b/.exec('ab')",
        r"/(?<=(a?){1,2})b/.exec('ab')",
        r"/(?<=((?=a)){1,2})a/.exec('a')",
        r"/(?<=(|a)+)a/.exec('a')",
        r"/(?<=(\2)+)a()/.exec('a')",
        r"/(?<=((a*)+))a/.exec('a')",
        r"/(?<=()+)a/u.exec('a')",
        r"/(?<=()+)a/v.exec('a')",
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
