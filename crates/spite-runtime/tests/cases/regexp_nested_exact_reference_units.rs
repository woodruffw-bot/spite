//! Exact scalar reference terms composed in ordinary repeated units.
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
fn nested_exact_reference_units_result_snapshot() {
    let mut rows = String::new();
    for (source, flags, text) in [
        (r"((b)\2{2})+", "d", "bbbbbbb"),
        (r"((b)\2{2})+?", "d", "bbbbbbb"),
        (r"((b)(\2{2})\3{2}){2}", "d", "bbbbbbbbbbbbbb"),
        (r"(\1{2}b)+", "d", "bbb"),
        (r"(\1*b)+", "d", "bbb"),
        (r"(\2{2}(b))+", "d", "bbb"),
        (r"(a)(\1{2}b)+", "d", "aaabaab"),
        (r"((b)\2{2}){2}", "d", "bbbbbb"),
        (r"((b)\2{2}){1,3}", "d", "bbbbbbbbbb"),
        (r"((b)\2{2}){1,3}?", "d", "bbbbbbbbbb"),
        (r"((b)\2{2})*", "d", "bbbbbb"),
        (r"((b)\2{2})*?", "d", "bbbbbb"),
        (r"((b)(?:\2\2){2})+", "d", "bbbbbbbbbb"),
        (r"(?<=((\3{2})(b)){2})c", "d", "bbbbbbc"),
        (r"(?<=((b)\2{2}){2})c", "d", "bbc"),
        (r"(?<=((\1*b)){2})c", "d", "bbc"),
        (r"(?<=a(?=((b)\2{2}){2}))b", "d", "abbbbbb"),
        (r"(?:()(?:\1)+){2}", "d", ""),
        (r"(?<x>(?<y>b)\k<y>{2})+", "d", "bbbbbbb"),
        (r"(?<x>\k<x>*b)+", "d", "bbb"),
        (r"(?<=(?<x>(\k<y>{2})(?<y>b)){2})c", "d", "bbbbbbc"),
        (r"((b)\2{2})+", "di", "bBbBBb"),
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
fn named_ranges_retry_required_empty_original_gap_and_both_directions_are_exact() {
    check(
        r"let a=/(?<x>(?<y>b)\k<y>{2})+bb/d.exec('bbbbbbb');a.index===0&&a[0]==='bbbbb'&&a.groups.x==='bbb'&&a.groups.y==='b'&&a.indices.groups.x===a.indices[1]&&a.indices[1][0]===0&&a.indices[2][0]===0",
    );
    check(
        r"let a=/(?<=(?<x>(\k<y>{2})(?<y>b)){2})c/d.exec('bbbbbbc');a.groups.x==='bbb'&&a[2]==='bb'&&a.groups.y==='b'&&a.indices.groups.x[0]===0&&a.indices.groups.y[0]===2",
    );
    check(
        r"let a=/(?<=a(?=((b)\2{2}){2}))b/d.exec('abbbbbb');a.index===1&&a.indices[1][0]===4&&a.indices[1][1]===7&&a.indices[2][0]===4",
    );
    check(r"/(?:()(?:\1)+){2}/.test('')");
    check(
        r"let a=/(?:()(?:\1)+){2}/d.exec('');a[1]===''&&a.indices[1][0]===0&&a.indices[1][1]===0",
    );
    check(r"let a=/(\1*b)+/d.exec('bbb');a[1]==='b'&&a.indices[1][0]===2");
    check(
        r"let a=/(?<x>(?<y>b)\k<y>{2})+/di.exec('bBbBBb');a.groups.x==='BBb'&&a.groups.y==='B'&&a.indices[1][0]===3",
    );
    check(
        r"let a=/(([\uD800])\2{2})+/d.exec('\uD800\uD800\uD800\uD800\uD800\uD800\uDC00');a[0].length===6&&a.indices[1][0]===3&&a.indices[2][0]===3",
    );
}
#[test]
fn consumers_keep_sticky_global_lazy_zero_advancement_and_callback_slots() {
    check(
        r"let r=/((b)\2{2})+/dy;r.lastIndex=1;let a=r.exec('cbbbbbb');a.index===1&&a[1]==='bbb'&&r.lastIndex===7&&r.exec('cbbbbbb')===null&&r.lastIndex===0",
    );
    check(
        r"let r=/((b)\2{2})+/dg,a=[...'bbbbbb bbb'.matchAll(r)];a.length===2&&a[0].indices[1][0]===3&&a[1].index===7&&a[1][2]==='b'&&r.lastIndex===0",
    );
    check(
        r"let a=[...'bbb'.matchAll(/((b)\2{2})*?/dg)];a.length===4&&a[3].index===3&&a[0][1]===undefined&&a[3].indices[2]===undefined",
    );
    check(
        r"let seen=[];let s='bbbbbb bbb'.replace(/((b)\2{2})+/g,(m,x,y,i)=>{seen.push(x,y,i);return '_'});s==='_ _'&&seen.join('|')==='bbb|b|0|bbb|b|7'",
    );
    check(
        r"'cbbbbbb'.search(/((b)\2{2})+/)===1&&'cbbbbbb'.split(/((b)\2{2})+/).join('|')==='c|bbb|b|'",
    );
}
#[test]
fn deep_scopes_long_repetitions_copies_collection_and_huge_empty_counts_stay_unlimited() {
    let mut realm = Realm::default();
    realm.eval(r"let n=100000,r=new RegExp('('+'('.repeat(n)+'b'+')'.repeat(n)+'\\2{2})+','d'),copy=new RegExp(r)").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("let a=copy.exec('bbbbbbb');a.length===n+2&&a[0].length===6&&a[1]==='bbb'&&a[n+1]==='b'&&a.indices[n+1][0]===3&&copy.source===r.source"),Ok(Value::Boolean(true)));
    assert_eq!(realm.eval(r"let long=/((b)\2{2})+/d.exec('b'.repeat(30000));long[0].length===30000&&long.indices[1][0]===29997&&long.indices[2][0]===29997"),Ok(Value::Boolean(true)));
    assert_eq!(realm.eval(r"let count='9'.repeat(100),empty=new RegExp('(?:()(?:\\1)+){'+count+'}','d').exec('');empty[1]===''&&empty.indices[1][0]===0"),Ok(Value::Boolean(true)));
    realm.collect(usize::MAX).unwrap();
}
#[test]
fn opted_in_work_abort_preserves_last_index_and_bypasses_handlers() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval(r"let marker=0,r=/((b)\2{2})+/g;r.lastIndex=1;let text='c'.repeat(10000)+'bbb'")
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
fn unproved_variable_assertion_widths_nullable_required_choices_and_unicode_stay_pending() {
    for source in [
        r"/(?<=a(?=((b)\2{2,3}){2}))b/.exec('abbbbbb')",
        r"/(?<=((\3{2,3})(b)){2})c/.exec('bbbbbbc')",
        r"/(?:(a|)(?:\1)+){2}/.test('')",
        r"/((b)\2{2})+/u.exec('bbb')",
        r"/((b)\2{2})+/v.exec('bbb')",
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
