//! Proven empty local/open/forward references in repeated ordinary lookbehind.

use spite_core::JsString;
use spite_runtime::{Error, Limits, Realm, Value};
use std::fmt::Write;

use crate::common::assert_true;

#[test]
fn zero_reference_lookbehind_count_result_snapshot() {
    let mut rows = String::new();
    for (source, flags, text) in [
        (r"(?<=(\1)+)a()", "d", "qa"),
        (r"(?<=(\1){2})a()", "d", "qa"),
        (r"(?<=(()\2){2})a", "d", "qa"),
        (r"(?<=(()\2){0,2})a", "d", "qa"),
        (r"(?<=((\2)){2})a", "d", "qa"),
        (r"(?<=((\3)()){2})a", "d", "qa"),
        (r"(?<=(()(\2)\3){2})a", "d", "qa"),
        (r"(?<=(()(\2)(\3)\4){2})a", "d", "qa"),
        (r"(?<=((()\3){2}){2})a", "d", "qa"),
        (r"(?<=((()\3){0,2}){2})a", "d", "qa"),
        (r"(?<=((()\3){2})*)a", "d", "qa"),
        (r"(?<=((\2){2}){2})a", "d", "qa"),
        (r"(?<=((\3()){2}){2})a", "d", "qa"),
        (r"(?<=((()\3\3){2}){2})a", "d", "qa"),
        (r"(?<=((()(\3)){2}){2})a", "d", "qa"),
        (r"(?<=((\2){0,2}){2})a", "d", "qa"),
        (r"(?<=((()\3)+)+)a", "d", "qa"),
        (r"(?<=((()\3)*)+)a", "d", "qa"),
        (r"(?<=((()\3)+)*)a", "d", "qa"),
        (r"(?<=((()\3){1,3}?){2})a", "d", "qa"),
        (r"(?<=((^()\3){2}){2})a", "d", "qa"),
        (r"(?<=((\b()\3){2}){2})a", "d", "qa"),
        (r"(?<=((\B()\3){2}){2})a", "d", "qa"),
        (r"(?<=((()^\3){0,2}){2})a", "d", "qa"),
        (r"(?<=(?:(?:(()\2){2}){2}|()){2})a", "d", "qa"),
        (r"(?<=(?:(?:(()\2){0,2}){2}|()){2})a", "d", "qa"),
        (r"(?<!(?:(()\2){2}){2}q)a", "d", "xa"),
        (r"(?<=((()\3){2}){2}a)b", "d", "ab"),
        (r"(?<=a((()\3){2}){2})b", "d", "ab"),
        (r"(?:(?<=((()\3){2}){2})a|b)+c", "d", "qa"),
        (r"(?<=((()\3){2}){2})a\1\2\3", "d", "qa"),
        (r"(?<=((()\3){0,2}){2})a\1\2\3", "d", "qa"),
        (r"(?<=((()\3){2}){2}µ)Μ", "di", "ΜΜ"),
        (r"(?<=((()\3){2}){2}[\uD800])b", "d", "lone surrogates"),
        (r"(?<=(?:((?<x>)\k<x>){2}){2})a\k<x>", "d", "qa"),
        (r"(?<=(?:((?<x>)\k<x>){0,2}){2})a\k<x>", "d", "qa"),
        (r"(?<=(?:(\k<x>(?<x>)){2}){2})a", "d", "qa"),
        (r"(?<=(?:((?<x>\k<x>)){2}){2})a", "d", "qa"),
        (r"(?<=(?:(?:(?<x>)\k<x>){2}|(?<x>)){2})a\k<x>", "d", "qa"),
        (
            r"(?<=(?:((?<x>)(?<y>\k<x>)\k<y>){2}){2})a\k<x>\k<y>",
            "d",
            "qa",
        ),
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
    insta::assert_snapshot!(rows, @r#"
        "(?<=(\u005c1)+)a()" flags="d" input="qa" "{\u0022matches\u0022:[\u0022a\u0022,\u0022\u0022,\u0022\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022qa\u0022,\u0022indices\u0022:[[1,2],[1,1],[2,2]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=(\u005c\u005c1)+)a()\u0022}"
        "(?<=(\u005c1){2})a()" flags="d" input="qa" "{\u0022matches\u0022:[\u0022a\u0022,\u0022\u0022,\u0022\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022qa\u0022,\u0022indices\u0022:[[1,2],[1,1],[2,2]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=(\u005c\u005c1){2})a()\u0022}"
        "(?<=(()\u005c2){2})a" flags="d" input="qa" "{\u0022matches\u0022:[\u0022a\u0022,\u0022\u0022,\u0022\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022qa\u0022,\u0022indices\u0022:[[1,2],[1,1],[1,1]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=(()\u005c\u005c2){2})a\u0022}"
        "(?<=(()\u005c2){0,2})a" flags="d" input="qa" "{\u0022matches\u0022:[\u0022a\u0022,null,null],\u0022index\u0022:1,\u0022input\u0022:\u0022qa\u0022,\u0022indices\u0022:[[1,2],null,null],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=(()\u005c\u005c2){0,2})a\u0022}"
        "(?<=((\u005c2)){2})a" flags="d" input="qa" "{\u0022matches\u0022:[\u0022a\u0022,\u0022\u0022,\u0022\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022qa\u0022,\u0022indices\u0022:[[1,2],[1,1],[1,1]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=((\u005c\u005c2)){2})a\u0022}"
        "(?<=((\u005c3)()){2})a" flags="d" input="qa" "{\u0022matches\u0022:[\u0022a\u0022,\u0022\u0022,\u0022\u0022,\u0022\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022qa\u0022,\u0022indices\u0022:[[1,2],[1,1],[1,1],[1,1]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=((\u005c\u005c3)()){2})a\u0022}"
        "(?<=(()(\u005c2)\u005c3){2})a" flags="d" input="qa" "{\u0022matches\u0022:[\u0022a\u0022,\u0022\u0022,\u0022\u0022,\u0022\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022qa\u0022,\u0022indices\u0022:[[1,2],[1,1],[1,1],[1,1]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=(()(\u005c\u005c2)\u005c\u005c3){2})a\u0022}"
        "(?<=(()(\u005c2)(\u005c3)\u005c4){2})a" flags="d" input="qa" "{\u0022matches\u0022:[\u0022a\u0022,\u0022\u0022,\u0022\u0022,\u0022\u0022,\u0022\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022qa\u0022,\u0022indices\u0022:[[1,2],[1,1],[1,1],[1,1],[1,1]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=(()(\u005c\u005c2)(\u005c\u005c3)\u005c\u005c4){2})a\u0022}"
        "(?<=((()\u005c3){2}){2})a" flags="d" input="qa" "{\u0022matches\u0022:[\u0022a\u0022,\u0022\u0022,\u0022\u0022,\u0022\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022qa\u0022,\u0022indices\u0022:[[1,2],[1,1],[1,1],[1,1]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=((()\u005c\u005c3){2}){2})a\u0022}"
        "(?<=((()\u005c3){0,2}){2})a" flags="d" input="qa" "{\u0022matches\u0022:[\u0022a\u0022,\u0022\u0022,null,null],\u0022index\u0022:1,\u0022input\u0022:\u0022qa\u0022,\u0022indices\u0022:[[1,2],[1,1],null,null],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=((()\u005c\u005c3){0,2}){2})a\u0022}"
        "(?<=((()\u005c3){2})*)a" flags="d" input="qa" "{\u0022matches\u0022:[\u0022a\u0022,null,null,null],\u0022index\u0022:1,\u0022input\u0022:\u0022qa\u0022,\u0022indices\u0022:[[1,2],null,null,null],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=((()\u005c\u005c3){2})*)a\u0022}"
        "(?<=((\u005c2){2}){2})a" flags="d" input="qa" "{\u0022matches\u0022:[\u0022a\u0022,\u0022\u0022,\u0022\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022qa\u0022,\u0022indices\u0022:[[1,2],[1,1],[1,1]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=((\u005c\u005c2){2}){2})a\u0022}"
        "(?<=((\u005c3()){2}){2})a" flags="d" input="qa" "{\u0022matches\u0022:[\u0022a\u0022,\u0022\u0022,\u0022\u0022,\u0022\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022qa\u0022,\u0022indices\u0022:[[1,2],[1,1],[1,1],[1,1]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=((\u005c\u005c3()){2}){2})a\u0022}"
        "(?<=((()\u005c3\u005c3){2}){2})a" flags="d" input="qa" "{\u0022matches\u0022:[\u0022a\u0022,\u0022\u0022,\u0022\u0022,\u0022\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022qa\u0022,\u0022indices\u0022:[[1,2],[1,1],[1,1],[1,1]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=((()\u005c\u005c3\u005c\u005c3){2}){2})a\u0022}"
        "(?<=((()(\u005c3)){2}){2})a" flags="d" input="qa" "{\u0022matches\u0022:[\u0022a\u0022,\u0022\u0022,\u0022\u0022,\u0022\u0022,\u0022\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022qa\u0022,\u0022indices\u0022:[[1,2],[1,1],[1,1],[1,1],[1,1]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=((()(\u005c\u005c3)){2}){2})a\u0022}"
        "(?<=((\u005c2){0,2}){2})a" flags="d" input="qa" "{\u0022matches\u0022:[\u0022a\u0022,\u0022\u0022,null],\u0022index\u0022:1,\u0022input\u0022:\u0022qa\u0022,\u0022indices\u0022:[[1,2],[1,1],null],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=((\u005c\u005c2){0,2}){2})a\u0022}"
        "(?<=((()\u005c3)+)+)a" flags="d" input="qa" "{\u0022matches\u0022:[\u0022a\u0022,\u0022\u0022,\u0022\u0022,\u0022\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022qa\u0022,\u0022indices\u0022:[[1,2],[1,1],[1,1],[1,1]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=((()\u005c\u005c3)+)+)a\u0022}"
        "(?<=((()\u005c3)*)+)a" flags="d" input="qa" "{\u0022matches\u0022:[\u0022a\u0022,\u0022\u0022,null,null],\u0022index\u0022:1,\u0022input\u0022:\u0022qa\u0022,\u0022indices\u0022:[[1,2],[1,1],null,null],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=((()\u005c\u005c3)*)+)a\u0022}"
        "(?<=((()\u005c3)+)*)a" flags="d" input="qa" "{\u0022matches\u0022:[\u0022a\u0022,null,null,null],\u0022index\u0022:1,\u0022input\u0022:\u0022qa\u0022,\u0022indices\u0022:[[1,2],null,null,null],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=((()\u005c\u005c3)+)*)a\u0022}"
        "(?<=((()\u005c3){1,3}?){2})a" flags="d" input="qa" "{\u0022matches\u0022:[\u0022a\u0022,\u0022\u0022,\u0022\u0022,\u0022\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022qa\u0022,\u0022indices\u0022:[[1,2],[1,1],[1,1],[1,1]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=((()\u005c\u005c3){1,3}?){2})a\u0022}"
        "(?<=((^()\u005c3){2}){2})a" flags="d" input="qa" "{\u0022match\u0022:null,\u0022lastIndex\u0022:0}"
        "(?<=((\u005cb()\u005c3){2}){2})a" flags="d" input="qa" "{\u0022match\u0022:null,\u0022lastIndex\u0022:0}"
        "(?<=((\u005cB()\u005c3){2}){2})a" flags="d" input="qa" "{\u0022matches\u0022:[\u0022a\u0022,\u0022\u0022,\u0022\u0022,\u0022\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022qa\u0022,\u0022indices\u0022:[[1,2],[1,1],[1,1],[1,1]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=((\u005c\u005cB()\u005c\u005c3){2}){2})a\u0022}"
        "(?<=((()^\u005c3){0,2}){2})a" flags="d" input="qa" "{\u0022matches\u0022:[\u0022a\u0022,\u0022\u0022,null,null],\u0022index\u0022:1,\u0022input\u0022:\u0022qa\u0022,\u0022indices\u0022:[[1,2],[1,1],null,null],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=((()^\u005c\u005c3){0,2}){2})a\u0022}"
        "(?<=(?:(?:(()\u005c2){2}){2}|()){2})a" flags="d" input="qa" "{\u0022matches\u0022:[\u0022a\u0022,\u0022\u0022,\u0022\u0022,null],\u0022index\u0022:1,\u0022input\u0022:\u0022qa\u0022,\u0022indices\u0022:[[1,2],[1,1],[1,1],null],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=(?:(?:(()\u005c\u005c2){2}){2}|()){2})a\u0022}"
        "(?<=(?:(?:(()\u005c2){0,2}){2}|()){2})a" flags="d" input="qa" "{\u0022matches\u0022:[\u0022a\u0022,null,null,null],\u0022index\u0022:1,\u0022input\u0022:\u0022qa\u0022,\u0022indices\u0022:[[1,2],null,null,null],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=(?:(?:(()\u005c\u005c2){0,2}){2}|()){2})a\u0022}"
        "(?<!(?:(()\u005c2){2}){2}q)a" flags="d" input="xa" "{\u0022matches\u0022:[\u0022a\u0022,null,null],\u0022index\u0022:1,\u0022input\u0022:\u0022xa\u0022,\u0022indices\u0022:[[1,2],null,null],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<!(?:(()\u005c\u005c2){2}){2}q)a\u0022}"
        "(?<=((()\u005c3){2}){2}a)b" flags="d" input="ab" "{\u0022matches\u0022:[\u0022b\u0022,\u0022\u0022,\u0022\u0022,\u0022\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022ab\u0022,\u0022indices\u0022:[[1,2],[0,0],[0,0],[0,0]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=((()\u005c\u005c3){2}){2}a)b\u0022}"
        "(?<=a((()\u005c3){2}){2})b" flags="d" input="ab" "{\u0022matches\u0022:[\u0022b\u0022,\u0022\u0022,\u0022\u0022,\u0022\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022ab\u0022,\u0022indices\u0022:[[1,2],[1,1],[1,1],[1,1]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=a((()\u005c\u005c3){2}){2})b\u0022}"
        "(?:(?<=((()\u005c3){2}){2})a|b)+c" flags="d" input="qa" "{\u0022match\u0022:null,\u0022lastIndex\u0022:0}"
        "(?<=((()\u005c3){2}){2})a\u005c1\u005c2\u005c3" flags="d" input="qa" "{\u0022matches\u0022:[\u0022a\u0022,\u0022\u0022,\u0022\u0022,\u0022\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022qa\u0022,\u0022indices\u0022:[[1,2],[1,1],[1,1],[1,1]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=((()\u005c\u005c3){2}){2})a\u005c\u005c1\u005c\u005c2\u005c\u005c3\u0022}"
        "(?<=((()\u005c3){0,2}){2})a\u005c1\u005c2\u005c3" flags="d" input="qa" "{\u0022matches\u0022:[\u0022a\u0022,\u0022\u0022,null,null],\u0022index\u0022:1,\u0022input\u0022:\u0022qa\u0022,\u0022indices\u0022:[[1,2],[1,1],null,null],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=((()\u005c\u005c3){0,2}){2})a\u005c\u005c1\u005c\u005c2\u005c\u005c3\u0022}"
        "(?<=((()\u005c3){2}){2}\u00b5)\u039c" flags="di" input="\u039c\u039c" "{\u0022matches\u0022:[\u0022\u039c\u0022,\u0022\u0022,\u0022\u0022,\u0022\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022\u039c\u039c\u0022,\u0022indices\u0022:[[1,2],[0,0],[0,0],[0,0]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=((()\u005c\u005c3){2}){2}\u00b5)\u039c\u0022}"
        "(?<=((()\u005c3){2}){2}[\u005cuD800])b" flags="d" input="\ud800b" "{\u0022matches\u0022:[\u0022b\u0022,\u0022\u0022,\u0022\u0022,\u0022\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022\u005cud800b\u0022,\u0022indices\u0022:[[1,2],[0,0],[0,0],[0,0]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=((()\u005c\u005c3){2}){2}[\u005c\u005cuD800])b\u0022}"
        "(?<=(?:((?<x>)\u005ck<x>){2}){2})a\u005ck<x>" flags="d" input="qa" "{\u0022matches\u0022:[\u0022a\u0022,\u0022\u0022,\u0022\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022qa\u0022,\u0022groups\u0022:{\u0022x\u0022:\u0022\u0022},\u0022indices\u0022:[[1,2],[1,1],[1,1]],\u0022indicesGroups\u0022:{\u0022x\u0022:[1,1]},\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=(?:((?<x>)\u005c\u005ck<x>){2}){2})a\u005c\u005ck<x>\u0022}"
        "(?<=(?:((?<x>)\u005ck<x>){0,2}){2})a\u005ck<x>" flags="d" input="qa" "{\u0022matches\u0022:[\u0022a\u0022,null,null],\u0022index\u0022:1,\u0022input\u0022:\u0022qa\u0022,\u0022groups\u0022:{},\u0022indices\u0022:[[1,2],null,null],\u0022indicesGroups\u0022:{},\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=(?:((?<x>)\u005c\u005ck<x>){0,2}){2})a\u005c\u005ck<x>\u0022}"
        "(?<=(?:(\u005ck<x>(?<x>)){2}){2})a" flags="d" input="qa" "{\u0022matches\u0022:[\u0022a\u0022,\u0022\u0022,\u0022\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022qa\u0022,\u0022groups\u0022:{\u0022x\u0022:\u0022\u0022},\u0022indices\u0022:[[1,2],[1,1],[1,1]],\u0022indicesGroups\u0022:{\u0022x\u0022:[1,1]},\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=(?:(\u005c\u005ck<x>(?<x>)){2}){2})a\u0022}"
        "(?<=(?:((?<x>\u005ck<x>)){2}){2})a" flags="d" input="qa" "{\u0022matches\u0022:[\u0022a\u0022,\u0022\u0022,\u0022\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022qa\u0022,\u0022groups\u0022:{\u0022x\u0022:\u0022\u0022},\u0022indices\u0022:[[1,2],[1,1],[1,1]],\u0022indicesGroups\u0022:{\u0022x\u0022:[1,1]},\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=(?:((?<x>\u005c\u005ck<x>)){2}){2})a\u0022}"
        "(?<=(?:(?:(?<x>)\u005ck<x>){2}|(?<x>)){2})a\u005ck<x>" flags="d" input="qa" "{\u0022matches\u0022:[\u0022a\u0022,\u0022\u0022,null],\u0022index\u0022:1,\u0022input\u0022:\u0022qa\u0022,\u0022groups\u0022:{\u0022x\u0022:\u0022\u0022},\u0022indices\u0022:[[1,2],[1,1],null],\u0022indicesGroups\u0022:{\u0022x\u0022:[1,1]},\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=(?:(?:(?<x>)\u005c\u005ck<x>){2}|(?<x>)){2})a\u005c\u005ck<x>\u0022}"
        "(?<=(?:((?<x>)(?<y>\u005ck<x>)\u005ck<y>){2}){2})a\u005ck<x>\u005ck<y>" flags="d" input="qa" "{\u0022matches\u0022:[\u0022a\u0022,\u0022\u0022,\u0022\u0022,\u0022\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022qa\u0022,\u0022groups\u0022:{\u0022x\u0022:\u0022\u0022,\u0022y\u0022:\u0022\u0022},\u0022indices\u0022:[[1,2],[1,1],[1,1],[1,1]],\u0022indicesGroups\u0022:{\u0022x\u0022:[1,1],\u0022y\u0022:[1,1]},\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=(?:((?<x>)(?<y>\u005c\u005ck<x>)\u005c\u005ck<y>){2}){2})a\u005c\u005ck<x>\u005c\u005ck<y>\u0022}"
        "#);
}

#[test]
fn empty_lookbehind_references_keep_named_open_forward_chained_and_optional_slots() {
    assert_true(
        r"let a=/(?<=(?:((?<x>)\k<x>){2}){2})a\k<x>/d.exec('qa'),b=/(?<=(?:((?<x>)\k<x>){0,2}){2})a\k<x>/d.exec('qa');a[1]===''&&a.groups.x===''&&a.indices.groups.x===a.indices[2]&&a.indices[2][0]===1&&b[1]===undefined&&b.groups.x===undefined&&b.indices[2]===undefined",
    );
    assert_true(
        r"let a=/(?<=(?:(\k<x>(?<x>)){2}){2})a/d.exec('qa'),b=/(?<=(?:((?<x>\k<x>)){2}){2})a/d.exec('qa');a[1]===''&&a.groups.x===''&&a.indices.groups.x===a.indices[2]&&b[1]===''&&b.groups.x===''&&b.indices[2][0]===1",
    );
    assert_true(
        r"let a=/(?<=(?:((?<x>)(?<y>\k<x>)\k<y>){2}){2})a\k<x>\k<y>/d.exec('qa');a[1]===''&&a.groups.x===''&&a.groups.y===''&&a.indices.groups.x===a.indices[2]&&a.indices.groups.y===a.indices[3]&&a.indices[2]!==a.indices[3]",
    );
    assert_true(
        r"let a=/(?<=(?:(?:(?<x>)\k<x>){2}|(?<x>)){2})a\k<x>/d.exec('qa');a[1]===''&&a[2]===undefined&&a.groups.x===''&&a.indices.groups.x===a.indices[1]",
    );
    assert_true(
        r"let a=/(?<=((()\3){0,2}){2})a/d.exec('qa');a[1]===''&&a[2]===undefined&&a[3]===undefined&&a.indices[1][0]===1&&a.indices[2]===undefined",
    );
    assert_true(
        r"let a=/(?<!(?:(()\2){2}){2}q)a/d.exec('xa');a.index===1&&a[1]===undefined&&a[2]===undefined&&a.indices[2]===undefined",
    );
    assert_true(
        r"let a=/(?<=((^()\3){2}){2}a)b/d.exec('ab'),b=/(?<=a((()\3){2}){2})b/d.exec('ab');a.index===1&&a.indices[3][0]===0&&b.index===1&&b.indices[3][0]===1",
    );
}

#[test]
fn empty_lookbehind_reference_consumers_keep_last_index_empty_advancement_and_callback_slots() {
    assert_true(
        r"let r=/(?<=(?:((?<x>)\k<x>){2}){2})a/dg,a=[...'qa a'.matchAll(r)];a.length===2&&a[1].index===3&&a[1].groups.x===''&&a[1].indices.groups.x[0]===3&&r.lastIndex===0",
    );
    assert_true(
        r"let r=/(?<=((()\3){2}){2})a/dy;r.lastIndex=1;let a=r.exec('qa');a[3]===''&&a.index===1&&r.lastIndex===2&&r.exec('qa')===null&&r.lastIndex===0",
    );
    assert_true(
        r"let r=/(?<=((()\3){0,2}){2})/dg,a=[...'ab'.matchAll(r)];a.length===3&&a[2].index===2&&a[2][1]===''&&a[2][3]===undefined&&a[2].indices[1][0]===2&&r.lastIndex===0",
    );
    assert_true(
        r"let seen=[];let s='qa a'.replace(/(?<=(?:((?<x>)\k<x>){2}){2})a/g,(m,c,d,i,s,g)=>{seen.push(c,d,i,g.x);return '_'});s==='q_ _'&&seen.length===8&&seen[0]===''&&seen[1]===''&&seen[2]===1&&seen[3]===''&&seen[6]===3",
    );
    assert_true(
        r"'qa a'.replace(/(?<=(?:((?<x>)\k<x>){2}){2})a/g,'<$<x>>')==='q<> <>'&&'qa'.search(/(?<=((()\3){2}){2})a/)===1&&'qa a'.split(/(?<=((()\3){2}){2})a/).join('|')==='q|||| ||||'",
    );
}

#[test]
fn explicit_empty_lookbehind_reference_work_aborts_before_last_index_and_handlers() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval(r"let marker=0,r=/(?<=((()\3){2}){2})a/g;r.lastIndex=1;let text='b'.repeat(5000)")
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
fn consuming_reference_units_outside_reads_lookahead_and_unicode_are_pending() {
    for source in [
        r"/(?<=((a)\2){1,2})b/.exec('aab')",
        r"/(?<=((\2)a){1,2})b/.exec('aab')",
        r"/(?<=(((a)\3){0,2}){2})a/.exec('a')",
        r"/(?<=(\2)+)a()/.exec('a')",
        r"/(?<=((?=a)\1){2})a/.exec('a')",
        r"/(?<=(()\2){2})a/u.exec('a')",
        r"/(?<=(()\2){2})a/v.exec('a')",
    ] {
        crate::common::regexp_oracle::assert_matches_reference(&mut Realm::default(), source);
    }
}
