//! Fixed lookahead captures retain their forward iteration ranges.
use crate::common::assert_true;
use spite_core::JsString;
use spite_runtime::{Error, Limits, Realm, Value};
use std::fmt::Write;

#[test]
fn fixed_lookahead_capture_result_snapshot() {
    let mut rows = String::new();
    for (source, flags, text) in [
        (r"(?<=a(?=(b)))b", "d", "ab"),
        (r"(?<=a(?=(b){2}))b", "d", "abb"),
        (r"(?<=a(?=((b)){2}))b", "d", "abb"),
        (r"(?<=a(?=(b){0}))b", "d", "ab"),
        (r"(?<=a(?=((b){0}){2}))b", "d", "ab"),
        (r"(?<=a(?=((b){0})*))b", "d", "ab"),
        (r"(?<=a(?=(b|c)))b", "d", "ab"),
        (r"(?<=a(?=(b)|(c)))b", "d", "ab"),
        (r"(?<=a(?=(b.)|(bc)))b", "d", "abc"),
        (r"(?<=a(?!((b){2})))b", "d", "abb"),
        (r"(?<!(?=(b))q)c", "d", "bc"),
        (r"(?<=(?=(a))(a))b", "d", "ab"),
        (r"(?<=a(?=(?=(b))b))b", "d", "ab"),
        (r"(?<=a(?=(?!(c))b))b", "d", "ab"),
        (r"(?<=a(?=(?<=(a))b))b", "d", "ab"),
        (r"(?<=a(?=(?<!((c)))b))b", "d", "ab"),
        (r"(?<=a(?=(b{2})))b", "d", "abb"),
        (r"(?<=a(?=((b)b){2}))b", "d", "abbbb"),
        (r"(?<=a(?=(b()){2}))b", "d", "abb"),
        (r"(?<=a(?=(()b){2}))b", "d", "abb"),
        (r"(?<=a(?=(\b){2}b))b", "d", "ab"),
        (r"(?<=a(?=((?=b)){2}b))b", "d", "ab"),
        (r"(?<=((?=(b))){2})b", "d", "b"),
        (r"(?<=((?=(b)))*)b", "d", "b"),
        (r"(?<=a(?=(b)))b\1", "d", "abb"),
        (r"(?<=a(?=(b){2}))b\1", "d", "abb"),
        (r"(?<=a(?=(b))|a(?=(c)))b", "d", "ab"),
        (r"(?<!a(?=(b)))b", "d", "ab"),
        (r"(?<=µ(?=(Μ){2}))Μ", "di", "ΜΜΜ"),
        (r"(?<=[\uD800](?=([\uDC00]){2}))[\uDC00]", "d", "surrogates"),
        (r"(?:(?<=a(?=(b)))b|a)+c", "d", "ababc"),
        (r"(?=(?<=a(?=(b)))b)b", "d", "qab"),
        (r"(?<=a(?=(?<x>b){2}))b", "d", "qabb"),
        (r"(?<=a(?=(?:(?<x>b)|(?<x>c))))b", "d", "qab"),
        (r"(?<=a(?=(?<=(?<x>a){2})b))b", "d", "qaab"),
        (r"(?<!(?=(?<x>b))q)c", "d", "qbc"),
        (r"(?<=a(?=(b){2}))b", "dg", "qabb"),
        (r"(?<=a(?=(b.)))b", "ds", "ab\n"),
        (r"(?<=a(?=(b$)))b", "dm", "ab\nq"),
    ] {
        let source = JsString::from(source);
        let flags = JsString::from(flags);
        let input = if text == "surrogates" {
            JsString::from_code_units(vec![0xd800, 0xdc00, 0xdc00])
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
    insta::assert_snapshot!(rows, @r#"
        "(?<=a(?=(b)))b" flags="d" input="ab" "{\u0022matches\u0022:[\u0022b\u0022,\u0022b\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022ab\u0022,\u0022indices\u0022:[[1,2],[1,2]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=a(?=(b)))b\u0022}"
        "(?<=a(?=(b){2}))b" flags="d" input="abb" "{\u0022matches\u0022:[\u0022b\u0022,\u0022b\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022abb\u0022,\u0022indices\u0022:[[1,2],[2,3]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=a(?=(b){2}))b\u0022}"
        "(?<=a(?=((b)){2}))b" flags="d" input="abb" "{\u0022matches\u0022:[\u0022b\u0022,\u0022b\u0022,\u0022b\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022abb\u0022,\u0022indices\u0022:[[1,2],[2,3],[2,3]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=a(?=((b)){2}))b\u0022}"
        "(?<=a(?=(b){0}))b" flags="d" input="ab" "{\u0022matches\u0022:[\u0022b\u0022,null],\u0022index\u0022:1,\u0022input\u0022:\u0022ab\u0022,\u0022indices\u0022:[[1,2],null],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=a(?=(b){0}))b\u0022}"
        "(?<=a(?=((b){0}){2}))b" flags="d" input="ab" "{\u0022matches\u0022:[\u0022b\u0022,\u0022\u0022,null],\u0022index\u0022:1,\u0022input\u0022:\u0022ab\u0022,\u0022indices\u0022:[[1,2],[1,1],null],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=a(?=((b){0}){2}))b\u0022}"
        "(?<=a(?=((b){0})*))b" flags="d" input="ab" "{\u0022matches\u0022:[\u0022b\u0022,null,null],\u0022index\u0022:1,\u0022input\u0022:\u0022ab\u0022,\u0022indices\u0022:[[1,2],null,null],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=a(?=((b){0})*))b\u0022}"
        "(?<=a(?=(b|c)))b" flags="d" input="ab" "{\u0022matches\u0022:[\u0022b\u0022,\u0022b\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022ab\u0022,\u0022indices\u0022:[[1,2],[1,2]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=a(?=(b|c)))b\u0022}"
        "(?<=a(?=(b)|(c)))b" flags="d" input="ab" "{\u0022matches\u0022:[\u0022b\u0022,\u0022b\u0022,null],\u0022index\u0022:1,\u0022input\u0022:\u0022ab\u0022,\u0022indices\u0022:[[1,2],[1,2],null],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=a(?=(b)|(c)))b\u0022}"
        "(?<=a(?=(b.)|(bc)))b" flags="d" input="abc" "{\u0022matches\u0022:[\u0022b\u0022,\u0022bc\u0022,null],\u0022index\u0022:1,\u0022input\u0022:\u0022abc\u0022,\u0022indices\u0022:[[1,2],[1,3],null],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=a(?=(b.)|(bc)))b\u0022}"
        "(?<=a(?!((b){2})))b" flags="d" input="abb" "{\u0022match\u0022:null,\u0022lastIndex\u0022:0}"
        "(?<!(?=(b))q)c" flags="d" input="bc" "{\u0022matches\u0022:[\u0022c\u0022,null],\u0022index\u0022:1,\u0022input\u0022:\u0022bc\u0022,\u0022indices\u0022:[[1,2],null],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<!(?=(b))q)c\u0022}"
        "(?<=(?=(a))(a))b" flags="d" input="ab" "{\u0022matches\u0022:[\u0022b\u0022,\u0022a\u0022,\u0022a\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022ab\u0022,\u0022indices\u0022:[[1,2],[0,1],[0,1]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=(?=(a))(a))b\u0022}"
        "(?<=a(?=(?=(b))b))b" flags="d" input="ab" "{\u0022matches\u0022:[\u0022b\u0022,\u0022b\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022ab\u0022,\u0022indices\u0022:[[1,2],[1,2]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=a(?=(?=(b))b))b\u0022}"
        "(?<=a(?=(?!(c))b))b" flags="d" input="ab" "{\u0022matches\u0022:[\u0022b\u0022,null],\u0022index\u0022:1,\u0022input\u0022:\u0022ab\u0022,\u0022indices\u0022:[[1,2],null],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=a(?=(?!(c))b))b\u0022}"
        "(?<=a(?=(?<=(a))b))b" flags="d" input="ab" "{\u0022matches\u0022:[\u0022b\u0022,\u0022a\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022ab\u0022,\u0022indices\u0022:[[1,2],[0,1]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=a(?=(?<=(a))b))b\u0022}"
        "(?<=a(?=(?<!((c)))b))b" flags="d" input="ab" "{\u0022matches\u0022:[\u0022b\u0022,null,null],\u0022index\u0022:1,\u0022input\u0022:\u0022ab\u0022,\u0022indices\u0022:[[1,2],null,null],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=a(?=(?<!((c)))b))b\u0022}"
        "(?<=a(?=(b{2})))b" flags="d" input="abb" "{\u0022matches\u0022:[\u0022b\u0022,\u0022bb\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022abb\u0022,\u0022indices\u0022:[[1,2],[1,3]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=a(?=(b{2})))b\u0022}"
        "(?<=a(?=((b)b){2}))b" flags="d" input="abbbb" "{\u0022matches\u0022:[\u0022b\u0022,\u0022bb\u0022,\u0022b\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022abbbb\u0022,\u0022indices\u0022:[[1,2],[3,5],[3,4]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=a(?=((b)b){2}))b\u0022}"
        "(?<=a(?=(b()){2}))b" flags="d" input="abb" "{\u0022matches\u0022:[\u0022b\u0022,\u0022b\u0022,\u0022\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022abb\u0022,\u0022indices\u0022:[[1,2],[2,3],[3,3]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=a(?=(b()){2}))b\u0022}"
        "(?<=a(?=(()b){2}))b" flags="d" input="abb" "{\u0022matches\u0022:[\u0022b\u0022,\u0022b\u0022,\u0022\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022abb\u0022,\u0022indices\u0022:[[1,2],[2,3],[2,2]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=a(?=(()b){2}))b\u0022}"
        "(?<=a(?=(\u005cb){2}b))b" flags="d" input="ab" "{\u0022match\u0022:null,\u0022lastIndex\u0022:0}"
        "(?<=a(?=((?=b)){2}b))b" flags="d" input="ab" "{\u0022matches\u0022:[\u0022b\u0022,\u0022\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022ab\u0022,\u0022indices\u0022:[[1,2],[1,1]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=a(?=((?=b)){2}b))b\u0022}"
        "(?<=((?=(b))){2})b" flags="d" input="b" "{\u0022matches\u0022:[\u0022b\u0022,\u0022\u0022,\u0022b\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022b\u0022,\u0022indices\u0022:[[0,1],[0,0],[0,1]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=((?=(b))){2})b\u0022}"
        "(?<=((?=(b)))*)b" flags="d" input="b" "{\u0022matches\u0022:[\u0022b\u0022,null,null],\u0022index\u0022:0,\u0022input\u0022:\u0022b\u0022,\u0022indices\u0022:[[0,1],null,null],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=((?=(b)))*)b\u0022}"
        "(?<=a(?=(b)))b\u005c1" flags="d" input="abb" "{\u0022matches\u0022:[\u0022bb\u0022,\u0022b\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022abb\u0022,\u0022indices\u0022:[[1,3],[1,2]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=a(?=(b)))b\u005c\u005c1\u0022}"
        "(?<=a(?=(b){2}))b\u005c1" flags="d" input="abb" "{\u0022matches\u0022:[\u0022bb\u0022,\u0022b\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022abb\u0022,\u0022indices\u0022:[[1,3],[2,3]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=a(?=(b){2}))b\u005c\u005c1\u0022}"
        "(?<=a(?=(b))|a(?=(c)))b" flags="d" input="ab" "{\u0022matches\u0022:[\u0022b\u0022,\u0022b\u0022,null],\u0022index\u0022:1,\u0022input\u0022:\u0022ab\u0022,\u0022indices\u0022:[[1,2],[1,2],null],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=a(?=(b))|a(?=(c)))b\u0022}"
        "(?<!a(?=(b)))b" flags="d" input="ab" "{\u0022match\u0022:null,\u0022lastIndex\u0022:0}"
        "(?<=\u00b5(?=(\u039c){2}))\u039c" flags="di" input="\u039c\u039c\u039c" "{\u0022matches\u0022:[\u0022\u039c\u0022,\u0022\u039c\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022\u039c\u039c\u039c\u0022,\u0022indices\u0022:[[1,2],[2,3]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=\u00b5(?=(\u039c){2}))\u039c\u0022}"
        "(?<=[\u005cuD800](?=([\u005cuDC00]){2}))[\u005cuDC00]" flags="d" input="\ud800\udc00\udc00" "{\u0022matches\u0022:[\u0022\u005cudc00\u0022,\u0022\u005cudc00\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022\ud800\udc00\u005cudc00\u0022,\u0022indices\u0022:[[1,2],[2,3]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=[\u005c\u005cuD800](?=([\u005c\u005cuDC00]){2}))[\u005c\u005cuDC00]\u0022}"
        "(?:(?<=a(?=(b)))b|a)+c" flags="d" input="ababc" "{\u0022matches\u0022:[\u0022ababc\u0022,\u0022b\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022ababc\u0022,\u0022indices\u0022:[[0,5],[3,4]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?:(?<=a(?=(b)))b|a)+c\u0022}"
        "(?=(?<=a(?=(b)))b)b" flags="d" input="qab" "{\u0022matches\u0022:[\u0022b\u0022,\u0022b\u0022],\u0022index\u0022:2,\u0022input\u0022:\u0022qab\u0022,\u0022indices\u0022:[[2,3],[2,3]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?=(?<=a(?=(b)))b)b\u0022}"
        "(?<=a(?=(?<x>b){2}))b" flags="d" input="qabb" "{\u0022matches\u0022:[\u0022b\u0022,\u0022b\u0022],\u0022index\u0022:2,\u0022input\u0022:\u0022qabb\u0022,\u0022groups\u0022:{\u0022x\u0022:\u0022b\u0022},\u0022indices\u0022:[[2,3],[3,4]],\u0022indicesGroups\u0022:{\u0022x\u0022:[3,4]},\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=a(?=(?<x>b){2}))b\u0022}"
        "(?<=a(?=(?:(?<x>b)|(?<x>c))))b" flags="d" input="qab" "{\u0022matches\u0022:[\u0022b\u0022,\u0022b\u0022,null],\u0022index\u0022:2,\u0022input\u0022:\u0022qab\u0022,\u0022groups\u0022:{\u0022x\u0022:\u0022b\u0022},\u0022indices\u0022:[[2,3],[2,3],null],\u0022indicesGroups\u0022:{\u0022x\u0022:[2,3]},\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=a(?=(?:(?<x>b)|(?<x>c))))b\u0022}"
        "(?<=a(?=(?<=(?<x>a){2})b))b" flags="d" input="qaab" "{\u0022matches\u0022:[\u0022b\u0022,\u0022a\u0022],\u0022index\u0022:3,\u0022input\u0022:\u0022qaab\u0022,\u0022groups\u0022:{\u0022x\u0022:\u0022a\u0022},\u0022indices\u0022:[[3,4],[1,2]],\u0022indicesGroups\u0022:{\u0022x\u0022:[1,2]},\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=a(?=(?<=(?<x>a){2})b))b\u0022}"
        "(?<!(?=(?<x>b))q)c" flags="d" input="qbc" "{\u0022matches\u0022:[\u0022c\u0022,null],\u0022index\u0022:2,\u0022input\u0022:\u0022qbc\u0022,\u0022groups\u0022:{},\u0022indices\u0022:[[2,3],null],\u0022indicesGroups\u0022:{},\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<!(?=(?<x>b))q)c\u0022}"
        "(?<=a(?=(b){2}))b" flags="dg" input="qabb" "{\u0022matches\u0022:[\u0022b\u0022,\u0022b\u0022],\u0022index\u0022:2,\u0022input\u0022:\u0022qabb\u0022,\u0022indices\u0022:[[2,3],[3,4]],\u0022lastIndex\u0022:3,\u0022source\u0022:\u0022(?<=a(?=(b){2}))b\u0022}"
        "(?<=a(?=(b.)))b" flags="ds" input="ab\u000a" "{\u0022matches\u0022:[\u0022b\u0022,\u0022b\u005cn\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022ab\u005cn\u0022,\u0022indices\u0022:[[1,2],[1,3]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=a(?=(b.)))b\u0022}"
        "(?<=a(?=(b$)))b" flags="dm" input="ab\u000aq" "{\u0022matches\u0022:[\u0022b\u0022,\u0022b\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022ab\u005cnq\u0022,\u0022indices\u0022:[[1,2],[1,2]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=a(?=(b$)))b\u0022}"
        "#);
}

#[test]
fn named_forward_ranges_after_the_match_backward_children_and_negative_restoration() {
    assert_true(
        r#"
            let a = /(?<=a(?=(?<x>b){2}))b/d.exec('qabb');
            a.index === 2 && a.groups.x === 'b' && a.indices.groups.x === a.indices[1] && a.indices[1][
                0] === 3 && a.indices[1][1] === 4
        "#,
    );
    assert_true(
        r#"
            let a = /(?<=a(?=(?:(?<x>b)|(?<x>c))))b/d.exec('qab');
            a.groups.x === 'b' && a[1] === 'b' && a[2] === undefined && a.indices.groups.x === a.indices[
                1] && a.indices[1][0] === 2
        "#,
    );
    assert_true(
        r#"
            let a = /(?<=a(?=(?<=(?<x>a){2})b))b/d.exec('qaab');
            a.groups.x === 'a' && a.index === 3 && a.indices.groups.x[0] === 1 && a.indices.groups.x[1] ===
                2
        "#,
    );
    assert_true(
        r#"
            let a = /(?<!(?=(?<x>b))q)c/d.exec('qbc');
            a.index === 2 && a.groups.x === undefined && Object.hasOwn(a.groups, 'x') && a.indices.groups
                .x === undefined && Object.hasOwn(a.indices.groups, 'x')
        "#,
    );
    assert_true(
        r#"
            let a = /(?<=((?=(b))){2})b/d.exec('qb'),
                b = /(?<=((?=(b)))*)b/d.exec('qb');
            a[1] === '' && a[2] === 'b' && a.indices[1][0] === 1 && a.indices[2][1] === 2 && b[1] ===
                undefined && b[2] === undefined
        "#,
    );
}

#[test]
fn consumers_callbacks_imported_ranges_sticky_and_empty_advancement() {
    assert_true(
        r#"
            let r = /(?<=a(?=(b){2}))b/dg,
                a = [...'qabb abb'.matchAll(r)];
            a.length === 2 && a[1].index === 6 && a[1].indices[1][0] === 7 && r.lastIndex === 0
        "#,
    );
    assert_true(
        r#"
            let r = /(?<=a(?=(b){2}))b/dy;
            r.lastIndex = 2;
            let a = r.exec('qabb');
            a.index === 2 && a.indices[1][0] === 3 && r.lastIndex === 3 && r.exec('qabb') === null && r
                .lastIndex === 0
        "#,
    );
    assert_true(
        r#"
            let r = /(?<=((?=(b))){2})/dg,
                a = [...'bb'.matchAll(r)];
            a.length === 2 && a[1].index === 1 && a[1][1] === '' && a[1][2] === 'b' && a[1].indices[2][
                0] === 1 && r.lastIndex === 0
        "#,
    );
    assert_true(
        r#"
            let seen = [];
            let s = 'qabb abb'.replace(/(?<=a(?=(b){2}))b/g, (m, c, i) => {
                seen.push(c, i);
                return '_'
            });
            s === 'qa_b a_b' && seen.join('|') === 'b|2|b|6'
        "#,
    );
    assert_true(
        r"'qabb'.replace(/(?<=a(?=(?<x>b){2}))b/,'<$<x>>')==='qa<b>b'&&'qabb'.search(/(?<=a(?=(b){2}))b/)===2&&'abb'.split(/(?<=a(?=(b){2}))b/).join('|')==='a|b|b'",
    );
}

#[test]
fn explicit_work_aborts_keep_last_index_and_bypass_handlers() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval("let marker=0,r=/(?<=a(?=(b){2}))b/g;r.lastIndex=1;let text='a'.repeat(5000)")
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
fn composition_results_match_reference() {
    for source in [
        r"/(?<=a(?=(b+)))b/.exec('abb')",
        r"/(?<=a(?=(b{1,2})\1))b/.exec('abb')",
        r"/(?<=a(?=(b|cc)))b/.exec('abb')",
        r"/(?<=a(?=(b){1,2}))b/.exec('abb')",
        r"/(?<=a(?=(b)))b/u.exec('ab')",
        r"/(?<=a(?=(b)))b/v.exec('ab')",
    ] {
        crate::common::regexp_oracle::assert_matches_reference(&mut Realm::default(), source);
    }
}
