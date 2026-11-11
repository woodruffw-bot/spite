//! Exact scalar reference terms composed in ordinary repeated units.
use crate::common::assert_true;
use spite_core::JsString;
use spite_runtime::{Error, Limits, Realm, Value};
use std::fmt::Write;
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
    insta::assert_snapshot!(rows, @r#"
        "((b)\u005c2{2})+" flags="d" input="bbbbbbb" "{\u0022matches\u0022:[\u0022bbbbbb\u0022,\u0022bbb\u0022,\u0022b\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022bbbbbbb\u0022,\u0022indices\u0022:[[0,6],[3,6],[3,4]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022((b)\u005c\u005c2{2})+\u0022}"
        "((b)\u005c2{2})+?" flags="d" input="bbbbbbb" "{\u0022matches\u0022:[\u0022bbb\u0022,\u0022bbb\u0022,\u0022b\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022bbbbbbb\u0022,\u0022indices\u0022:[[0,3],[0,3],[0,1]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022((b)\u005c\u005c2{2})+?\u0022}"
        "((b)(\u005c2{2})\u005c3{2}){2}" flags="d" input="bbbbbbbbbbbbbb" "{\u0022matches\u0022:[\u0022bbbbbbbbbbbbbb\u0022,\u0022bbbbbbb\u0022,\u0022b\u0022,\u0022bb\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022bbbbbbbbbbbbbb\u0022,\u0022indices\u0022:[[0,14],[7,14],[7,8],[8,10]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022((b)(\u005c\u005c2{2})\u005c\u005c3{2}){2}\u0022}"
        "(\u005c1{2}b)+" flags="d" input="bbb" "{\u0022matches\u0022:[\u0022bbb\u0022,\u0022b\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022bbb\u0022,\u0022indices\u0022:[[0,3],[2,3]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(\u005c\u005c1{2}b)+\u0022}"
        "(\u005c1*b)+" flags="d" input="bbb" "{\u0022matches\u0022:[\u0022bbb\u0022,\u0022b\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022bbb\u0022,\u0022indices\u0022:[[0,3],[2,3]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(\u005c\u005c1*b)+\u0022}"
        "(\u005c2{2}(b))+" flags="d" input="bbb" "{\u0022matches\u0022:[\u0022bbb\u0022,\u0022b\u0022,\u0022b\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022bbb\u0022,\u0022indices\u0022:[[0,3],[2,3],[2,3]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(\u005c\u005c2{2}(b))+\u0022}"
        "(a)(\u005c1{2}b)+" flags="d" input="aaabaab" "{\u0022matches\u0022:[\u0022aaabaab\u0022,\u0022a\u0022,\u0022aab\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022aaabaab\u0022,\u0022indices\u0022:[[0,7],[0,1],[4,7]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a)(\u005c\u005c1{2}b)+\u0022}"
        "((b)\u005c2{2}){2}" flags="d" input="bbbbbb" "{\u0022matches\u0022:[\u0022bbbbbb\u0022,\u0022bbb\u0022,\u0022b\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022bbbbbb\u0022,\u0022indices\u0022:[[0,6],[3,6],[3,4]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022((b)\u005c\u005c2{2}){2}\u0022}"
        "((b)\u005c2{2}){1,3}" flags="d" input="bbbbbbbbbb" "{\u0022matches\u0022:[\u0022bbbbbbbbb\u0022,\u0022bbb\u0022,\u0022b\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022bbbbbbbbbb\u0022,\u0022indices\u0022:[[0,9],[6,9],[6,7]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022((b)\u005c\u005c2{2}){1,3}\u0022}"
        "((b)\u005c2{2}){1,3}?" flags="d" input="bbbbbbbbbb" "{\u0022matches\u0022:[\u0022bbb\u0022,\u0022bbb\u0022,\u0022b\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022bbbbbbbbbb\u0022,\u0022indices\u0022:[[0,3],[0,3],[0,1]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022((b)\u005c\u005c2{2}){1,3}?\u0022}"
        "((b)\u005c2{2})*" flags="d" input="bbbbbb" "{\u0022matches\u0022:[\u0022bbbbbb\u0022,\u0022bbb\u0022,\u0022b\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022bbbbbb\u0022,\u0022indices\u0022:[[0,6],[3,6],[3,4]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022((b)\u005c\u005c2{2})*\u0022}"
        "((b)\u005c2{2})*?" flags="d" input="bbbbbb" "{\u0022matches\u0022:[\u0022\u0022,null,null],\u0022index\u0022:0,\u0022input\u0022:\u0022bbbbbb\u0022,\u0022indices\u0022:[[0,0],null,null],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022((b)\u005c\u005c2{2})*?\u0022}"
        "((b)(?:\u005c2\u005c2){2})+" flags="d" input="bbbbbbbbbb" "{\u0022matches\u0022:[\u0022bbbbbbbbbb\u0022,\u0022bbbbb\u0022,\u0022b\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022bbbbbbbbbb\u0022,\u0022indices\u0022:[[0,10],[5,10],[5,6]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022((b)(?:\u005c\u005c2\u005c\u005c2){2})+\u0022}"
        "(?<=((\u005c3{2})(b)){2})c" flags="d" input="bbbbbbc" "{\u0022matches\u0022:[\u0022c\u0022,\u0022bbb\u0022,\u0022bb\u0022,\u0022b\u0022],\u0022index\u0022:6,\u0022input\u0022:\u0022bbbbbbc\u0022,\u0022indices\u0022:[[6,7],[0,3],[0,2],[2,3]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=((\u005c\u005c3{2})(b)){2})c\u0022}"
        "(?<=((b)\u005c2{2}){2})c" flags="d" input="bbc" "{\u0022matches\u0022:[\u0022c\u0022,\u0022b\u0022,\u0022b\u0022],\u0022index\u0022:2,\u0022input\u0022:\u0022bbc\u0022,\u0022indices\u0022:[[2,3],[0,1],[0,1]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=((b)\u005c\u005c2{2}){2})c\u0022}"
        "(?<=((\u005c1*b)){2})c" flags="d" input="bbc" "{\u0022matches\u0022:[\u0022c\u0022,\u0022b\u0022,\u0022b\u0022],\u0022index\u0022:2,\u0022input\u0022:\u0022bbc\u0022,\u0022indices\u0022:[[2,3],[0,1],[0,1]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=((\u005c\u005c1*b)){2})c\u0022}"
        "(?<=a(?=((b)\u005c2{2}){2}))b" flags="d" input="abbbbbb" "{\u0022matches\u0022:[\u0022b\u0022,\u0022bbb\u0022,\u0022b\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022abbbbbb\u0022,\u0022indices\u0022:[[1,2],[4,7],[4,5]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=a(?=((b)\u005c\u005c2{2}){2}))b\u0022}"
        "(?:()(?:\u005c1)+){2}" flags="d" input="" "{\u0022matches\u0022:[\u0022\u0022,\u0022\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022\u0022,\u0022indices\u0022:[[0,0],[0,0]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?:()(?:\u005c\u005c1)+){2}\u0022}"
        "(?<x>(?<y>b)\u005ck<y>{2})+" flags="d" input="bbbbbbb" "{\u0022matches\u0022:[\u0022bbbbbb\u0022,\u0022bbb\u0022,\u0022b\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022bbbbbbb\u0022,\u0022groups\u0022:{\u0022x\u0022:\u0022bbb\u0022,\u0022y\u0022:\u0022b\u0022},\u0022indices\u0022:[[0,6],[3,6],[3,4]],\u0022indicesGroups\u0022:{\u0022x\u0022:[3,6],\u0022y\u0022:[3,4]},\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<x>(?<y>b)\u005c\u005ck<y>{2})+\u0022}"
        "(?<x>\u005ck<x>*b)+" flags="d" input="bbb" "{\u0022matches\u0022:[\u0022bbb\u0022,\u0022b\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022bbb\u0022,\u0022groups\u0022:{\u0022x\u0022:\u0022b\u0022},\u0022indices\u0022:[[0,3],[2,3]],\u0022indicesGroups\u0022:{\u0022x\u0022:[2,3]},\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<x>\u005c\u005ck<x>*b)+\u0022}"
        "(?<=(?<x>(\u005ck<y>{2})(?<y>b)){2})c" flags="d" input="bbbbbbc" "{\u0022matches\u0022:[\u0022c\u0022,\u0022bbb\u0022,\u0022bb\u0022,\u0022b\u0022],\u0022index\u0022:6,\u0022input\u0022:\u0022bbbbbbc\u0022,\u0022groups\u0022:{\u0022x\u0022:\u0022bbb\u0022,\u0022y\u0022:\u0022b\u0022},\u0022indices\u0022:[[6,7],[0,3],[0,2],[2,3]],\u0022indicesGroups\u0022:{\u0022x\u0022:[0,3],\u0022y\u0022:[2,3]},\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=(?<x>(\u005c\u005ck<y>{2})(?<y>b)){2})c\u0022}"
        "((b)\u005c2{2})+" flags="di" input="bBbBBb" "{\u0022matches\u0022:[\u0022bBbBBb\u0022,\u0022BBb\u0022,\u0022B\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022bBbBBb\u0022,\u0022indices\u0022:[[0,6],[3,6],[3,4]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022((b)\u005c\u005c2{2})+\u0022}"
        "#);
}
#[test]
fn named_ranges_retry_required_empty_original_gap_and_both_directions_are_exact() {
    assert_true(
        r#"
            let a = /(?<x>(?<y>b)\k<y>{2})+bb/d.exec('bbbbbbb');
            a.index === 0 && a[0] === 'bbbbb' && a.groups.x === 'bbb' && a.groups.y === 'b' && a.indices
                .groups.x === a.indices[1] && a.indices[1][0] === 0 && a.indices[2][0] === 0
        "#,
    );
    assert_true(
        r#"
            let a = /(?<=(?<x>(\k<y>{2})(?<y>b)){2})c/d.exec('bbbbbbc');
            a.groups.x === 'bbb' && a[2] === 'bb' && a.groups.y === 'b' && a.indices.groups.x[0] === 0 && a
                .indices.groups.y[0] === 2
        "#,
    );
    assert_true(
        r#"
            let a = /(?<=a(?=((b)\2{2}){2}))b/d.exec('abbbbbb');
            a.index === 1 && a.indices[1][0] === 4 && a.indices[1][1] === 7 && a.indices[2][0] === 4
        "#,
    );
    assert_true(r"/(?:()(?:\1)+){2}/.test('')");
    assert_true(
        r"let a=/(?:()(?:\1)+){2}/d.exec('');a[1]===''&&a.indices[1][0]===0&&a.indices[1][1]===0",
    );
    assert_true(r"let a=/(\1*b)+/d.exec('bbb');a[1]==='b'&&a.indices[1][0]===2");
    assert_true(
        r"let a=/(?<x>(?<y>b)\k<y>{2})+/di.exec('bBbBBb');a.groups.x==='BBb'&&a.groups.y==='B'&&a.indices[1][0]===3",
    );
    assert_true(
        r#"
            let a = /(([\uD800])\2{2})+/d.exec('\uD800\uD800\uD800\uD800\uD800\uD800\uDC00');
            a[0].length === 6 && a.indices[1][0] === 3 && a.indices[2][0] === 3
        "#,
    );
}
#[test]
fn consumers_keep_sticky_global_lazy_zero_advancement_and_callback_slots() {
    assert_true(
        r#"
            let r = /((b)\2{2})+/dy;
            r.lastIndex = 1;
            let a = r.exec('cbbbbbb');
            a.index === 1 && a[1] === 'bbb' && r.lastIndex === 7 && r.exec('cbbbbbb') === null && r
                .lastIndex === 0
        "#,
    );
    assert_true(
        r#"
            let r = /((b)\2{2})+/dg,
                a = [...'bbbbbb bbb'.matchAll(r)];
            a.length === 2 && a[0].indices[1][0] === 3 && a[1].index === 7 && a[1][2] === 'b' && r
                .lastIndex === 0
        "#,
    );
    assert_true(
        r#"
            let a = [...'bbb'.matchAll(/((b)\2{2})*?/dg)];
            a.length === 4 && a[3].index === 3 && a[0][1] === undefined && a[3].indices[2] === undefined
        "#,
    );
    assert_true(
        r#"
            let seen = [];
            let s = 'bbbbbb bbb'.replace(/((b)\2{2})+/g, (m, x, y, i) => {
                seen.push(x, y, i);
                return '_'
            });
            s === '_ _' && seen.join('|') === 'bbb|b|0|bbb|b|7'
        "#,
    );
    assert_true(
        r"'cbbbbbb'.search(/((b)\2{2})+/)===1&&'cbbbbbb'.split(/((b)\2{2})+/).join('|')==='c|bbb|b|'",
    );
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
        crate::common::regexp_oracle::assert_matches_reference(&mut Realm::default(), source);
    }
}
