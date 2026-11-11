//! Native named captures retain slots, null prototypes, and indices aliases.

use spite_core::JsString;
use spite_runtime::{Error, Limits, Realm, Value};
use std::fmt::Write;

use crate::common::assert_true;

#[test]
fn named_capture_result_snapshot() {
    let mut rows = String::new();
    for (source, flags, text) in [
        ("(?<x>a)", "d", "qa"),
        ("(?<x>a)", "", "a"),
        ("(?<x>a)(b)(?<y>c)", "d", "qabc"),
        ("(?<outer>(?<x>a))", "d", "a"),
        ("(?<x>a)|(?<y>b)", "d", "b"),
        ("(?<x>a)|(?<x>b)", "d", "b"),
        ("(?<outer>(?<x>a)|(?<x>b))", "d", "b"),
        ("(?<x>a)|(?<x>b)", "d", "a"),
        ("(?<x>a)|(?<x>b)", "d", "c"),
        ("(?<x>)(?<y>a)", "d", "a"),
        ("(?<x>a)*", "d", ""),
        ("(?<x>a)+", "d", "aaa"),
        ("(?<x>a|b)+", "d", "abab"),
        ("(?<x>a|b)*", "d", ""),
        ("(?<outer>x(?<x>a|b)+)y", "d", "qxabby"),
        ("(?<outer>x(?<x>[^z]|b))+y", "d", "xaxby"),
        (
            "(?<prefix>x)(?<outer>(?<x>a|b)c)+(?<suffix>y)",
            "d",
            "xacbcy",
        ),
        (r"(?<\u0078>a)", "d", "a"),
        (r"(?<\u{10400}>a)", "d", "a"),
        ("(?<𐐀>a)(?<µ>b)", "d", "ab"),
        ("(?<a\u{200c}>a)", "d", "a"),
        ("(?<__proto__>a)(?<constructor>b)(?<toString>c)", "d", "abc"),
        ("(?<x>µ|Μ)(?<y>ſ|S)", "di", "ΜS"),
        (r"(?<x>\uD800)(?<y>\uDC00)", "d", "𐀀"),
        ("(?<x>.)", "ds", "\n"),
        ("^(?<x>a|b)$", "dm", "q\nb\n"),
        (r"\b(?<x>a|b)\b", "d", " b "),
        ("(?<x>a|b)", "dg", "qab"),
        ("(?<x>a|b)", "dy", "ba"),
        ("(?<x>a|b)", "dy", "qa"),
        ("(a)", "d", "a"),
    ] {
        let source = JsString::from(source);
        let flags = JsString::from(flags);
        let text = JsString::from(text);
        let program = format!(
            "let r=new RegExp({source:?},{flags:?}),a=r.exec({text:?});JSON.stringify(a===null?null:{{matches:[...a],index:a.index,input:a.input,groups:a.groups,indices:a.indices,indicesGroups:a.indices===undefined?undefined:a.indices.groups,keys:a.groups===undefined?[]:Object.keys(a.groups),nullPrototype:a.groups===undefined?true:Object.getPrototypeOf(a.groups)===null,lastIndex:r.lastIndex}})"
        );
        let value = Realm::default().eval(&program).unwrap();
        let Value::String(value) = value else {
            panic!("expected JSON: {value:?}");
        };
        writeln!(rows, "{source:?} flags={flags:?} input={text:?} {value:?}").unwrap();
    }
    insta::assert_snapshot!(rows, @r#"
        "(?<x>a)" flags="d" input="qa" "{\u0022matches\u0022:[\u0022a\u0022,\u0022a\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022qa\u0022,\u0022groups\u0022:{\u0022x\u0022:\u0022a\u0022},\u0022indices\u0022:[[1,2],[1,2]],\u0022indicesGroups\u0022:{\u0022x\u0022:[1,2]},\u0022keys\u0022:[\u0022x\u0022],\u0022nullPrototype\u0022:true,\u0022lastIndex\u0022:0}"
        "(?<x>a)" flags="" input="a" "{\u0022matches\u0022:[\u0022a\u0022,\u0022a\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022a\u0022,\u0022groups\u0022:{\u0022x\u0022:\u0022a\u0022},\u0022keys\u0022:[\u0022x\u0022],\u0022nullPrototype\u0022:true,\u0022lastIndex\u0022:0}"
        "(?<x>a)(b)(?<y>c)" flags="d" input="qabc" "{\u0022matches\u0022:[\u0022abc\u0022,\u0022a\u0022,\u0022b\u0022,\u0022c\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022qabc\u0022,\u0022groups\u0022:{\u0022x\u0022:\u0022a\u0022,\u0022y\u0022:\u0022c\u0022},\u0022indices\u0022:[[1,4],[1,2],[2,3],[3,4]],\u0022indicesGroups\u0022:{\u0022x\u0022:[1,2],\u0022y\u0022:[3,4]},\u0022keys\u0022:[\u0022x\u0022,\u0022y\u0022],\u0022nullPrototype\u0022:true,\u0022lastIndex\u0022:0}"
        "(?<outer>(?<x>a))" flags="d" input="a" "{\u0022matches\u0022:[\u0022a\u0022,\u0022a\u0022,\u0022a\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022a\u0022,\u0022groups\u0022:{\u0022outer\u0022:\u0022a\u0022,\u0022x\u0022:\u0022a\u0022},\u0022indices\u0022:[[0,1],[0,1],[0,1]],\u0022indicesGroups\u0022:{\u0022outer\u0022:[0,1],\u0022x\u0022:[0,1]},\u0022keys\u0022:[\u0022outer\u0022,\u0022x\u0022],\u0022nullPrototype\u0022:true,\u0022lastIndex\u0022:0}"
        "(?<x>a)|(?<y>b)" flags="d" input="b" "{\u0022matches\u0022:[\u0022b\u0022,null,\u0022b\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022b\u0022,\u0022groups\u0022:{\u0022y\u0022:\u0022b\u0022},\u0022indices\u0022:[[0,1],null,[0,1]],\u0022indicesGroups\u0022:{\u0022y\u0022:[0,1]},\u0022keys\u0022:[\u0022x\u0022,\u0022y\u0022],\u0022nullPrototype\u0022:true,\u0022lastIndex\u0022:0}"
        "(?<x>a)|(?<x>b)" flags="d" input="b" "{\u0022matches\u0022:[\u0022b\u0022,null,\u0022b\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022b\u0022,\u0022groups\u0022:{\u0022x\u0022:\u0022b\u0022},\u0022indices\u0022:[[0,1],null,[0,1]],\u0022indicesGroups\u0022:{\u0022x\u0022:[0,1]},\u0022keys\u0022:[\u0022x\u0022],\u0022nullPrototype\u0022:true,\u0022lastIndex\u0022:0}"
        "(?<outer>(?<x>a)|(?<x>b))" flags="d" input="b" "{\u0022matches\u0022:[\u0022b\u0022,\u0022b\u0022,null,\u0022b\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022b\u0022,\u0022groups\u0022:{\u0022outer\u0022:\u0022b\u0022,\u0022x\u0022:\u0022b\u0022},\u0022indices\u0022:[[0,1],[0,1],null,[0,1]],\u0022indicesGroups\u0022:{\u0022outer\u0022:[0,1],\u0022x\u0022:[0,1]},\u0022keys\u0022:[\u0022outer\u0022,\u0022x\u0022],\u0022nullPrototype\u0022:true,\u0022lastIndex\u0022:0}"
        "(?<x>a)|(?<x>b)" flags="d" input="a" "{\u0022matches\u0022:[\u0022a\u0022,\u0022a\u0022,null],\u0022index\u0022:0,\u0022input\u0022:\u0022a\u0022,\u0022groups\u0022:{\u0022x\u0022:\u0022a\u0022},\u0022indices\u0022:[[0,1],[0,1],null],\u0022indicesGroups\u0022:{\u0022x\u0022:[0,1]},\u0022keys\u0022:[\u0022x\u0022],\u0022nullPrototype\u0022:true,\u0022lastIndex\u0022:0}"
        "(?<x>a)|(?<x>b)" flags="d" input="c" "null"
        "(?<x>)(?<y>a)" flags="d" input="a" "{\u0022matches\u0022:[\u0022a\u0022,\u0022\u0022,\u0022a\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022a\u0022,\u0022groups\u0022:{\u0022x\u0022:\u0022\u0022,\u0022y\u0022:\u0022a\u0022},\u0022indices\u0022:[[0,1],[0,0],[0,1]],\u0022indicesGroups\u0022:{\u0022x\u0022:[0,0],\u0022y\u0022:[0,1]},\u0022keys\u0022:[\u0022x\u0022,\u0022y\u0022],\u0022nullPrototype\u0022:true,\u0022lastIndex\u0022:0}"
        "(?<x>a)*" flags="d" input="" "{\u0022matches\u0022:[\u0022\u0022,null],\u0022index\u0022:0,\u0022input\u0022:\u0022\u0022,\u0022groups\u0022:{},\u0022indices\u0022:[[0,0],null],\u0022indicesGroups\u0022:{},\u0022keys\u0022:[\u0022x\u0022],\u0022nullPrototype\u0022:true,\u0022lastIndex\u0022:0}"
        "(?<x>a)+" flags="d" input="aaa" "{\u0022matches\u0022:[\u0022aaa\u0022,\u0022a\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022aaa\u0022,\u0022groups\u0022:{\u0022x\u0022:\u0022a\u0022},\u0022indices\u0022:[[0,3],[2,3]],\u0022indicesGroups\u0022:{\u0022x\u0022:[2,3]},\u0022keys\u0022:[\u0022x\u0022],\u0022nullPrototype\u0022:true,\u0022lastIndex\u0022:0}"
        "(?<x>a|b)+" flags="d" input="abab" "{\u0022matches\u0022:[\u0022abab\u0022,\u0022b\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022abab\u0022,\u0022groups\u0022:{\u0022x\u0022:\u0022b\u0022},\u0022indices\u0022:[[0,4],[3,4]],\u0022indicesGroups\u0022:{\u0022x\u0022:[3,4]},\u0022keys\u0022:[\u0022x\u0022],\u0022nullPrototype\u0022:true,\u0022lastIndex\u0022:0}"
        "(?<x>a|b)*" flags="d" input="" "{\u0022matches\u0022:[\u0022\u0022,null],\u0022index\u0022:0,\u0022input\u0022:\u0022\u0022,\u0022groups\u0022:{},\u0022indices\u0022:[[0,0],null],\u0022indicesGroups\u0022:{},\u0022keys\u0022:[\u0022x\u0022],\u0022nullPrototype\u0022:true,\u0022lastIndex\u0022:0}"
        "(?<outer>x(?<x>a|b)+)y" flags="d" input="qxabby" "{\u0022matches\u0022:[\u0022xabby\u0022,\u0022xabb\u0022,\u0022b\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022qxabby\u0022,\u0022groups\u0022:{\u0022outer\u0022:\u0022xabb\u0022,\u0022x\u0022:\u0022b\u0022},\u0022indices\u0022:[[1,6],[1,5],[4,5]],\u0022indicesGroups\u0022:{\u0022outer\u0022:[1,5],\u0022x\u0022:[4,5]},\u0022keys\u0022:[\u0022outer\u0022,\u0022x\u0022],\u0022nullPrototype\u0022:true,\u0022lastIndex\u0022:0}"
        "(?<outer>x(?<x>[^z]|b))+y" flags="d" input="xaxby" "{\u0022matches\u0022:[\u0022xaxby\u0022,\u0022xb\u0022,\u0022b\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022xaxby\u0022,\u0022groups\u0022:{\u0022outer\u0022:\u0022xb\u0022,\u0022x\u0022:\u0022b\u0022},\u0022indices\u0022:[[0,5],[2,4],[3,4]],\u0022indicesGroups\u0022:{\u0022outer\u0022:[2,4],\u0022x\u0022:[3,4]},\u0022keys\u0022:[\u0022outer\u0022,\u0022x\u0022],\u0022nullPrototype\u0022:true,\u0022lastIndex\u0022:0}"
        "(?<prefix>x)(?<outer>(?<x>a|b)c)+(?<suffix>y)" flags="d" input="xacbcy" "{\u0022matches\u0022:[\u0022xacbcy\u0022,\u0022x\u0022,\u0022bc\u0022,\u0022b\u0022,\u0022y\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022xacbcy\u0022,\u0022groups\u0022:{\u0022prefix\u0022:\u0022x\u0022,\u0022outer\u0022:\u0022bc\u0022,\u0022x\u0022:\u0022b\u0022,\u0022suffix\u0022:\u0022y\u0022},\u0022indices\u0022:[[0,6],[0,1],[3,5],[3,4],[5,6]],\u0022indicesGroups\u0022:{\u0022prefix\u0022:[0,1],\u0022outer\u0022:[3,5],\u0022x\u0022:[3,4],\u0022suffix\u0022:[5,6]},\u0022keys\u0022:[\u0022prefix\u0022,\u0022outer\u0022,\u0022x\u0022,\u0022suffix\u0022],\u0022nullPrototype\u0022:true,\u0022lastIndex\u0022:0}"
        "(?<\u005cu0078>a)" flags="d" input="a" "{\u0022matches\u0022:[\u0022a\u0022,\u0022a\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022a\u0022,\u0022groups\u0022:{\u0022x\u0022:\u0022a\u0022},\u0022indices\u0022:[[0,1],[0,1]],\u0022indicesGroups\u0022:{\u0022x\u0022:[0,1]},\u0022keys\u0022:[\u0022x\u0022],\u0022nullPrototype\u0022:true,\u0022lastIndex\u0022:0}"
        "(?<\u005cu{10400}>a)" flags="d" input="a" "{\u0022matches\u0022:[\u0022a\u0022,\u0022a\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022a\u0022,\u0022groups\u0022:{\u0022\ud801\udc00\u0022:\u0022a\u0022},\u0022indices\u0022:[[0,1],[0,1]],\u0022indicesGroups\u0022:{\u0022\ud801\udc00\u0022:[0,1]},\u0022keys\u0022:[\u0022\ud801\udc00\u0022],\u0022nullPrototype\u0022:true,\u0022lastIndex\u0022:0}"
        "(?<\ud801\udc00>a)(?<\u00b5>b)" flags="d" input="ab" "{\u0022matches\u0022:[\u0022ab\u0022,\u0022a\u0022,\u0022b\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022ab\u0022,\u0022groups\u0022:{\u0022\ud801\udc00\u0022:\u0022a\u0022,\u0022\u00b5\u0022:\u0022b\u0022},\u0022indices\u0022:[[0,2],[0,1],[1,2]],\u0022indicesGroups\u0022:{\u0022\ud801\udc00\u0022:[0,1],\u0022\u00b5\u0022:[1,2]},\u0022keys\u0022:[\u0022\ud801\udc00\u0022,\u0022\u00b5\u0022],\u0022nullPrototype\u0022:true,\u0022lastIndex\u0022:0}"
        "(?<a\u200c>a)" flags="d" input="a" "{\u0022matches\u0022:[\u0022a\u0022,\u0022a\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022a\u0022,\u0022groups\u0022:{\u0022a\u200c\u0022:\u0022a\u0022},\u0022indices\u0022:[[0,1],[0,1]],\u0022indicesGroups\u0022:{\u0022a\u200c\u0022:[0,1]},\u0022keys\u0022:[\u0022a\u200c\u0022],\u0022nullPrototype\u0022:true,\u0022lastIndex\u0022:0}"
        "(?<__proto__>a)(?<constructor>b)(?<toString>c)" flags="d" input="abc" "{\u0022matches\u0022:[\u0022abc\u0022,\u0022a\u0022,\u0022b\u0022,\u0022c\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022abc\u0022,\u0022groups\u0022:{\u0022__proto__\u0022:\u0022a\u0022,\u0022constructor\u0022:\u0022b\u0022,\u0022toString\u0022:\u0022c\u0022},\u0022indices\u0022:[[0,3],[0,1],[1,2],[2,3]],\u0022indicesGroups\u0022:{\u0022__proto__\u0022:[0,1],\u0022constructor\u0022:[1,2],\u0022toString\u0022:[2,3]},\u0022keys\u0022:[\u0022__proto__\u0022,\u0022constructor\u0022,\u0022toString\u0022],\u0022nullPrototype\u0022:true,\u0022lastIndex\u0022:0}"
        "(?<x>\u00b5|\u039c)(?<y>\u017f|S)" flags="di" input="\u039cS" "{\u0022matches\u0022:[\u0022\u039cS\u0022,\u0022\u039c\u0022,\u0022S\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022\u039cS\u0022,\u0022groups\u0022:{\u0022x\u0022:\u0022\u039c\u0022,\u0022y\u0022:\u0022S\u0022},\u0022indices\u0022:[[0,2],[0,1],[1,2]],\u0022indicesGroups\u0022:{\u0022x\u0022:[0,1],\u0022y\u0022:[1,2]},\u0022keys\u0022:[\u0022x\u0022,\u0022y\u0022],\u0022nullPrototype\u0022:true,\u0022lastIndex\u0022:0}"
        "(?<x>\u005cuD800)(?<y>\u005cuDC00)" flags="d" input="\ud800\udc00" "{\u0022matches\u0022:[\u0022\ud800\udc00\u0022,\u0022\u005cud800\u0022,\u0022\u005cudc00\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022\ud800\udc00\u0022,\u0022groups\u0022:{\u0022x\u0022:\u0022\u005cud800\u0022,\u0022y\u0022:\u0022\u005cudc00\u0022},\u0022indices\u0022:[[0,2],[0,1],[1,2]],\u0022indicesGroups\u0022:{\u0022x\u0022:[0,1],\u0022y\u0022:[1,2]},\u0022keys\u0022:[\u0022x\u0022,\u0022y\u0022],\u0022nullPrototype\u0022:true,\u0022lastIndex\u0022:0}"
        "(?<x>.)" flags="ds" input="\u000a" "{\u0022matches\u0022:[\u0022\u005cn\u0022,\u0022\u005cn\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022\u005cn\u0022,\u0022groups\u0022:{\u0022x\u0022:\u0022\u005cn\u0022},\u0022indices\u0022:[[0,1],[0,1]],\u0022indicesGroups\u0022:{\u0022x\u0022:[0,1]},\u0022keys\u0022:[\u0022x\u0022],\u0022nullPrototype\u0022:true,\u0022lastIndex\u0022:0}"
        "^(?<x>a|b)$" flags="dm" input="q\u000ab\u000a" "{\u0022matches\u0022:[\u0022b\u0022,\u0022b\u0022],\u0022index\u0022:2,\u0022input\u0022:\u0022q\u005cnb\u005cn\u0022,\u0022groups\u0022:{\u0022x\u0022:\u0022b\u0022},\u0022indices\u0022:[[2,3],[2,3]],\u0022indicesGroups\u0022:{\u0022x\u0022:[2,3]},\u0022keys\u0022:[\u0022x\u0022],\u0022nullPrototype\u0022:true,\u0022lastIndex\u0022:0}"
        "\u005cb(?<x>a|b)\u005cb" flags="d" input=" b " "{\u0022matches\u0022:[\u0022b\u0022,\u0022b\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022 b \u0022,\u0022groups\u0022:{\u0022x\u0022:\u0022b\u0022},\u0022indices\u0022:[[1,2],[1,2]],\u0022indicesGroups\u0022:{\u0022x\u0022:[1,2]},\u0022keys\u0022:[\u0022x\u0022],\u0022nullPrototype\u0022:true,\u0022lastIndex\u0022:0}"
        "(?<x>a|b)" flags="dg" input="qab" "{\u0022matches\u0022:[\u0022a\u0022,\u0022a\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022qab\u0022,\u0022groups\u0022:{\u0022x\u0022:\u0022a\u0022},\u0022indices\u0022:[[1,2],[1,2]],\u0022indicesGroups\u0022:{\u0022x\u0022:[1,2]},\u0022keys\u0022:[\u0022x\u0022],\u0022nullPrototype\u0022:true,\u0022lastIndex\u0022:2}"
        "(?<x>a|b)" flags="dy" input="ba" "{\u0022matches\u0022:[\u0022b\u0022,\u0022b\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022ba\u0022,\u0022groups\u0022:{\u0022x\u0022:\u0022b\u0022},\u0022indices\u0022:[[0,1],[0,1]],\u0022indicesGroups\u0022:{\u0022x\u0022:[0,1]},\u0022keys\u0022:[\u0022x\u0022],\u0022nullPrototype\u0022:true,\u0022lastIndex\u0022:1}"
        "(?<x>a|b)" flags="dy" input="qa" "null"
        "(a)" flags="d" input="a" "{\u0022matches\u0022:[\u0022a\u0022,\u0022a\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022a\u0022,\u0022indices\u0022:[[0,1],[0,1]],\u0022keys\u0022:[],\u0022nullPrototype\u0022:true,\u0022lastIndex\u0022:0}"
        "#);
}

#[test]
fn slots_aliases_key_order_and_descriptors_follow_original_names() {
    assert_true(
        r"let a=/(?<x>a)/d.exec('a'),p=a.indices[1];a.groups.x='z';a.indices.groups.x[0]=7;a.indices[1]=[8,9];a[1]==='a'&&p[0]===7&&a.indices.groups.x===p&&a.indices[1]!==p&&delete a.groups.x&&Object.hasOwn(a,'1')",
    );
    assert_true(
        r"let a=/(?<x>a)(b)(?<y>c)/d.exec('qabc'),p=Object.getOwnPropertyDescriptor(a.groups,'x');Object.getPrototypeOf(a.groups)===null&&Object.getPrototypeOf(a.indices.groups)===null&&a.groups.x==='a'&&a.groups.y==='c'&&a.indices.groups.x===a.indices[1]&&a.indices.groups.y===a.indices[3]&&Object.keys(a.groups).join(',')==='x,y'&&p.writable&&p.enumerable&&p.configurable&&p.value==='a'",
    );
    assert_true(
        r"let a=/(?<x>a)|(?<x>b)/d.exec('b');a[1]===undefined&&a[2]==='b'&&a.groups.x==='b'&&a.indices.groups.x===a.indices[2]&&Object.keys(a.groups).join(',')==='x'",
    );
    assert_true(
        r"let a=/(?<x>a)|(?<y>b)/d.exec('b');a.groups.x===undefined&&Object.hasOwn(a.groups,'x')&&a.indices.groups.x===undefined&&Object.hasOwn(a.indices.groups,'x')&&a.indices.groups.y===a.indices[2]",
    );
    assert_true(
        r"let a=/(?<x>)(?<y>a)/d.exec('a'),b=/(?<x>a)*/d.exec('');a.groups.x===''&&a.indices.groups.x===a.indices[1]&&a.indices.groups.x[0]===0&&a.indices.groups.x[1]===0&&b.groups.x===undefined&&Object.hasOwn(b.groups,'x')&&b.indices.groups.x===undefined",
    );
    assert_true(
        r"let a=/(?<__proto__>a)(?<constructor>b)(?<toString>c)/d.exec('abc');a.groups.__proto__==='a'&&a.groups.constructor==='b'&&a.groups.toString==='c'&&Object.getPrototypeOf(a.groups)===null&&Object.keys(a.groups).join(',')==='__proto__,constructor,toString'&&a.indices.groups.__proto__===a.indices[1]",
    );
}

#[test]
fn consumers_replacements_and_copies_observe_live_named_values() {
    assert_true(r"'qabc'.replace(/(?<x>a)(b)(?<y>c)/,'<$<y>,$<x>>')==='q<c,a>'");
    assert_true(
        r"let seen;let s='qabc'.replace(/(?<x>a)(b)(?<y>c)/,(whole,x,b,y,index,input,groups)=>{seen=[whole,x,b,y,index,input,groups.x,groups.y,Object.getPrototypeOf(groups)===null];return 'z'});s==='qz'&&seen.join(',')==='abc,a,b,c,1,qabc,a,c,true'",
    );
    assert_true(
        r"let r=/(?<x>a|b)/dg,a=[...'qab'.matchAll(r)];a.length===2&&a[0].groups.x==='a'&&a[1].groups.x==='b'&&a[1].indices.groups.x===a[1].indices[1]&&r.lastIndex===0",
    );
    assert_true(
        r"let r=/(?<x>a|b)/dg;Object.defineProperty(r,'source',{get(){throw 7}});let copy=new RegExp(r);copy.source==='(?<x>a|b)'&&copy.exec('b').groups.x==='b'&&copy.lastIndex===1&&r.lastIndex===0",
    );
    assert_true(
        r"'qabc'.split(/(?<x>a)(b)(?<y>c)/).join(',')==='q,a,b,c,'&&'qabc'.search(/(?<x>a)(b)(?<y>c)/)===1",
    );
}

#[test]
fn intrinsic_result_creation_does_not_call_prototype_setters() {
    assert_true(
        r"let calls=0;Object.defineProperty(Object.prototype,'x',{set(){calls++},configurable:true});Object.defineProperty(Array.prototype,'1',{set(){calls++},configurable:true});let a=/(?<x>a)/d.exec('a');calls===0&&a.groups.x==='a'&&a.indices.groups.x===a.indices[1]&&Object.getPrototypeOf(a.groups)===null",
    );
}

#[test]
fn unsupported_backreferences_modes_and_conditionally_captured_local_choices_remain_distinct() {
    for source in [
        r"/(?:(?:(?:(?<x>a)(?:\k<x>)+){2})|){2,3}/.test('aa')",
        r"/(?:(?:(?:(?<x>a)(?:\1)+){2})|){2,3}/.test('aa')",
        r"/(?:(?:(?:(?:x(?:(?<n>a)|(?<n>b))y){2}){2})|){2,3}/.test('xay')",
        "/(?<x>\\u{D800}\\u{DC00})/u.test('a')",
    ] {
        crate::common::regexp_oracle::assert_matches_reference(&mut Realm::default(), source);
    }
    let mut realm = Realm::new(Limits {
        max_properties: Some(200),
        ..Limits::default()
    });
    realm
        .eval("let r=new RegExp('()'.repeat(1000)+'(?<x>a)','g'),flag=0")
        .unwrap();
    assert!(matches!(
        realm.eval("try{r.exec('a')}catch{flag=1}finally{flag=2}"),
        Err(Error::Limit { .. })
    ));
    assert_eq!(
        realm.eval("flag===0&&r.lastIndex===1"),
        Ok(Value::Boolean(true))
    );
}
