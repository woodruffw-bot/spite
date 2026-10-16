//! Actual matching direction for effect-free undefined reads in repeated units.
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
fn unit_matching_direction_result_snapshot() {
    let mut rows = String::new();
    for (source, flags, text) in [
        (r"(\2+(b))+", "d", "bbb"),
        (r"(\2{2,3}(b)){2}", "d", "bbb"),
        (r"(\2{2,3}(b)){2}?", "d", "bbb"),
        (r"(?<=((b)\2+){2})c", "d", "bbc"),
        (r"(?<=((b)\2{2,3}){2})c", "d", "bbc"),
        (r"(?<=a(?=(\2+(b)){2}))b", "d", "abb"),
        (r"(?=(?<=((b)\2+){2})c)c", "d", "bbc"),
        (r"(?!(\2+(b)){2})c", "d", "c"),
        (r"(?<!((b)\2+){2})c", "d", "c"),
        (r"(?<x>\k<y>+(?<y>b))+", "d", "bbb"),
        (r"(?<=(?<x>(?<y>b)\k<y>+){2})c", "d", "bbc"),
        (r"(?<=a(?=(?<x>\k<y>+(?<y>b)){2}))b", "d", "abb"),
        (r"(?:()){2}", "d", ""),
        (r"(?:a){2}", "d", "aaa"),
        (r"(\1+b)+", "d", "bbb"),
        (r"((b)\2{2})+", "d", "bbbbbbb"),
        (r"(a)(\1{2}b)+", "d", "aaabaab"),
        (r"(\2+(b))+", "di", "bBb"),
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
fn actual_directions_names_forward_resets_and_negative_rollback_keep_original_ranges() {
    check(
        r"let a=/(?<x>\k<y>+(?<y>b))+/d.exec('bbb');a[0]==='bbb'&&a.groups.x==='b'&&a.groups.y==='b'&&a.indices.groups.x===a.indices[1]&&a.indices[1][0]===2&&a.indices.groups.y[0]===2",
    );
    check(
        r"let a=/(?<=(?<x>(?<y>b)\k<y>+){2})c/d.exec('bbc');a.index===2&&a.indices.groups.x[0]===0&&a.indices.groups.y[0]===0",
    );
    check(
        r"let a=/(?<=a(?=(?<x>\k<y>+(?<y>b)){2}))b/d.exec('abb');a.index===1&&a.indices.groups.x[0]===2&&a.indices.groups.y[0]===2",
    );
    check(
        r"let a=/(?=(?<=((b)\2+){2})c)c/d.exec('bbc');a.index===2&&a.indices[1][0]===0&&a.indices[2][0]===0",
    );
    check(
        r"let a=/(?!(\2+(b)){2})c/d.exec('c');a[1]===undefined&&a[2]===undefined&&a.indices[1]===undefined",
    );
    check(
        r"let a=/(?<!((b)\2+){2})c/d.exec('c');a[1]===undefined&&a[2]===undefined&&a.indices[2]===undefined",
    );
    check(
        r"let a=/(\2+([\uD800]))+/d.exec('\uD800\uD800\uDC00');a[0].length===2&&a.indices[2][0]===1",
    );
}
#[test]
fn consumers_global_sticky_empty_advancement_and_callbacks_keep_slots() {
    check(
        r"let r=/(\2+(b))+/dy;r.lastIndex=1;let a=r.exec('cbbb');a.index===1&&a.indices[2][0]===3&&r.lastIndex===4&&r.exec('cbbb')===null&&r.lastIndex===0",
    );
    check(
        r"let r=/(\2+(b))+/dg,a=[...'bb b'.matchAll(r)];a.length===2&&a[0].indices[2][0]===1&&a[1].index===3&&r.lastIndex===0",
    );
    check(
        r"let a=[...'bb'.matchAll(/(\2+(b))*?/dg)];a.length===3&&a[2].index===2&&a[0][1]===undefined&&a[2].indices[2]===undefined",
    );
    check(
        r"let seen=[];let s='bb b'.replace(/(\2+(b))+/g,(m,x,y,i)=>{seen.push(x,y,i);return '_'});s==='_ _'&&seen.join('|')==='b|b|0|b|b|3'",
    );
    check(r"'cbbb'.search(/(\2+(b))+/)===1&&'cbbb'.split(/(\2+(b))+/).join('|')==='c|b|b|'");
}
#[test]
fn deep_direction_scopes_clones_collection_and_huge_undefined_counts_stay_unlimited() {
    let mut realm = Realm::default();
    realm.eval(r"let n=100000,r=new RegExp('(\\2+'+'('.repeat(n)+'b'+')'.repeat(n)+')+','d'),copy=new RegExp(r)").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("let a=copy.exec('bbb');a.length===n+2&&a[0]==='bbb'&&a.indices[n+1][0]===2&&copy.source===r.source"),Ok(Value::Boolean(true)));
    assert_eq!(realm.eval(r"let long=/(\2+(b))+/d.exec('b'.repeat(30000));long[0].length===30000&&long.indices[2][0]===29999"),Ok(Value::Boolean(true)));
    assert_eq!(realm.eval(r"let count='9'.repeat(100),huge=new RegExp('(\\2{'+count+'}(b)){2}','d').exec('bb');huge[0]==='bb'&&huge.indices[2][0]===1"),Ok(Value::Boolean(true)));
    assert_eq!(realm.eval(r"let behind=new RegExp('(?<=((b)\\2{'+count+'}){2})c','d').exec('bbc');behind.index===2&&behind.indices[2][0]===0"),Ok(Value::Boolean(true)));
    realm.collect(usize::MAX).unwrap();
}
#[test]
fn opted_in_work_abort_preserves_last_index_and_bypasses_handlers() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval(r"let marker=0,r=/(\2+(b))+/g;r.lastIndex=1;let text='c'.repeat(10000)+'b'")
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
fn forward_consuming_variable_widths_nullable_choices_and_unicode_stay_pending() {
    for source in [
        r"/(?<=a(?=((b)\2+){2}))b/.exec('abbbbbb')",
        r"/(?<=a(?=((b)\2{2,3}){2}))b/.exec('abbbbbb')",
        r"/(?<=((\3{2,3})(b)){2})c/.exec('bbbbbbc')",
        r"/(?:(a|)(?:\1)+){2}/.test('')",
        r"/(\2+(b))+/u.exec('bbb')",
        r"/(\2+(b))+/v.exec('bbb')",
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
