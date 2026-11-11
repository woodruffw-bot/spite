//! Outside capture references in linear ordinary lookbehind.
use crate::common::assert_true;
use spite_core::JsString;
use spite_runtime::{Error, Limits, Realm, Value};
use std::fmt::Write;
#[test]
fn mixed_outside_lookbehind_result_snapshot() {
    let mut rows = String::new();
    for (source, flags, text) in [
        (r"(a)(?<=(a\1){2})b", "d", "aaaaab"),
        (r"(a)(?<=(\1a){2})b", "d", "aaaaab"),
        (r"(a)(?<=(\1a){2}?)b", "d", "aaaaab"),
        (r"(a)(?<=(a\1){1})b", "d", "aaab"),
        (r"(a)(b)(?<=(\1\2){2})c", "d", "abababc"),
        (r"(a)(bb)(?<=(\1\2){2})c", "d", "abbabbabbc"),
        (r"(a)(b)(?<=(\2\1){2})c", "d", "abababc"),
        (r"(a)(b)(?<=(\1\2a){2})c", "d", "abababc"),
        (r"(?=(a))(?<=(\1b){2})a", "d", "ababa"),
        (r"(?=(ab))(?<=(\1b){2})a", "d", "abbabbab"),
        (r"(a)(?<=(\1\B){2})b", "d", "aaab"),
        (r"()(?<=(\1\b){2})b", "d", "b"),
        (r"()()(?<=(\1\2){2})b", "d", "b"),
        (r"()(?<=(\1\B){2})b", "d", "b"),
        (r"(a)(?<=(\1[ab]){2})b", "d", "aaaaab"),
        (r"(a)(?<=(.\1){2})b", "d", "aaaaab"),
        (r"(a)(?<=(\1.){2})b", "d", "aaaaab"),
        (r"(a)(?<=(^\1){2})b", "d", "aaab"),
        (r"(a)(?<=(\1$){2})b", "d", "aaab"),
        (r"(a)(?<!(\1a){2}q)b", "d", "aaaaab"),
        (r"(b)(?<!(\1a){2}q)c", "d", "bababc"),
        (r"(a)(?<!(a\1){2})b", "d", "aaaaab"),
        (r"(?<=(\2a){2})(a)", "d", "aaa"),
        (r"(a(?<=(\1a){2}))b", "d", "aaab"),
        (r"(?=(a))(?<=(\1[ab]\1){2})a", "d", "aaaaaaa"),
        (r"(a)(?<=(a\1){2}(?=b))b", "d", "aaaaab"),
        (r"(a)(?<=(?<=a)(a\1){2})b", "d", "aaaaab"),
        (r"(a)(?<=(a\1){2}(?=(b)))b", "d", "aaaaab"),
        (r"(?:(a|b)(?<=(.\1){2}))+c", "d", "aaaabbc"),
        (r"(µ)(?<=(.\1){2})Μ", "di", "µµµµµΜ"),
        (r"([\uD800])(?<=([\uD800]\1){2})[\uDC00]", "d", "surrogates"),
        (r"()()(?<=(\1\2\b){30})b", "d", "b"),
        (r"(?<x>a)(?<=(?<y>a\k<x>){2})b", "d", "aaaaab"),
        (
            r"(?<x>a)(?<z>bb)(?<=(?<y>\k<x>\k<z>){2})c",
            "d",
            "abbabbabbc",
        ),
        (r"(?<x>b)(?<!(?<y>\k<x>a){2}q)c", "d", "bababc"),
        (r"(?=(?<x>a))(?<=(?<y>\k<x>b){2})a", "d", "ababa"),
        (r"(?<=(?<y>\k<x>a){2})(?<x>a)", "d", "aaa"),
        (r"(.)(?<=(a\1){2})", "di", "aaaaA"),
        (r"(a)(?<=(\1[ab]){2})b", "dm", "q\naaaab"),
    ] {
        let source = JsString::from(source);
        let flags = JsString::from(flags);
        let input = if text == "surrogates" {
            JsString::from_code_units(vec![0xd800, 0xd800, 0xd800, 0xd800, 0xd800, 0xdc00])
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
        "(a)(?<=(a\u005c1){2})b" flags="d" input="aaaaab" "{\u0022matches\u0022:[\u0022ab\u0022,\u0022a\u0022,\u0022aa\u0022],\u0022index\u0022:4,\u0022input\u0022:\u0022aaaaab\u0022,\u0022indices\u0022:[[4,6],[4,5],[1,3]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a)(?<=(a\u005c\u005c1){2})b\u0022}"
        "(a)(?<=(\u005c1a){2})b" flags="d" input="aaaaab" "{\u0022matches\u0022:[\u0022ab\u0022,\u0022a\u0022,\u0022aa\u0022],\u0022index\u0022:4,\u0022input\u0022:\u0022aaaaab\u0022,\u0022indices\u0022:[[4,6],[4,5],[1,3]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a)(?<=(\u005c\u005c1a){2})b\u0022}"
        "(a)(?<=(\u005c1a){2}?)b" flags="d" input="aaaaab" "{\u0022matches\u0022:[\u0022ab\u0022,\u0022a\u0022,\u0022aa\u0022],\u0022index\u0022:4,\u0022input\u0022:\u0022aaaaab\u0022,\u0022indices\u0022:[[4,6],[4,5],[1,3]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a)(?<=(\u005c\u005c1a){2}?)b\u0022}"
        "(a)(?<=(a\u005c1){1})b" flags="d" input="aaab" "{\u0022matches\u0022:[\u0022ab\u0022,\u0022a\u0022,\u0022aa\u0022],\u0022index\u0022:2,\u0022input\u0022:\u0022aaab\u0022,\u0022indices\u0022:[[2,4],[2,3],[1,3]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a)(?<=(a\u005c\u005c1){1})b\u0022}"
        "(a)(b)(?<=(\u005c1\u005c2){2})c" flags="d" input="abababc" "{\u0022matches\u0022:[\u0022abc\u0022,\u0022a\u0022,\u0022b\u0022,\u0022ab\u0022],\u0022index\u0022:4,\u0022input\u0022:\u0022abababc\u0022,\u0022indices\u0022:[[4,7],[4,5],[5,6],[2,4]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a)(b)(?<=(\u005c\u005c1\u005c\u005c2){2})c\u0022}"
        "(a)(bb)(?<=(\u005c1\u005c2){2})c" flags="d" input="abbabbabbc" "{\u0022matches\u0022:[\u0022abbc\u0022,\u0022a\u0022,\u0022bb\u0022,\u0022abb\u0022],\u0022index\u0022:6,\u0022input\u0022:\u0022abbabbabbc\u0022,\u0022indices\u0022:[[6,10],[6,7],[7,9],[3,6]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a)(bb)(?<=(\u005c\u005c1\u005c\u005c2){2})c\u0022}"
        "(a)(b)(?<=(\u005c2\u005c1){2})c" flags="d" input="abababc" "{\u0022match\u0022:null,\u0022lastIndex\u0022:0}"
        "(a)(b)(?<=(\u005c1\u005c2a){2})c" flags="d" input="abababc" "{\u0022match\u0022:null,\u0022lastIndex\u0022:0}"
        "(?=(a))(?<=(\u005c1b){2})a" flags="d" input="ababa" "{\u0022matches\u0022:[\u0022a\u0022,\u0022a\u0022,\u0022ab\u0022],\u0022index\u0022:4,\u0022input\u0022:\u0022ababa\u0022,\u0022indices\u0022:[[4,5],[4,5],[0,2]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?=(a))(?<=(\u005c\u005c1b){2})a\u0022}"
        "(?=(ab))(?<=(\u005c1b){2})a" flags="d" input="abbabbab" "{\u0022matches\u0022:[\u0022a\u0022,\u0022ab\u0022,\u0022abb\u0022],\u0022index\u0022:6,\u0022input\u0022:\u0022abbabbab\u0022,\u0022indices\u0022:[[6,7],[6,8],[0,3]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?=(ab))(?<=(\u005c\u005c1b){2})a\u0022}"
        "(a)(?<=(\u005c1\u005cB){2})b" flags="d" input="aaab" "{\u0022matches\u0022:[\u0022ab\u0022,\u0022a\u0022,\u0022a\u0022],\u0022index\u0022:2,\u0022input\u0022:\u0022aaab\u0022,\u0022indices\u0022:[[2,4],[2,3],[1,2]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a)(?<=(\u005c\u005c1\u005c\u005cB){2})b\u0022}"
        "()(?<=(\u005c1\u005cb){2})b" flags="d" input="b" "{\u0022matches\u0022:[\u0022b\u0022,\u0022\u0022,\u0022\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022b\u0022,\u0022indices\u0022:[[0,1],[0,0],[0,0]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022()(?<=(\u005c\u005c1\u005c\u005cb){2})b\u0022}"
        "()()(?<=(\u005c1\u005c2){2})b" flags="d" input="b" "{\u0022matches\u0022:[\u0022b\u0022,\u0022\u0022,\u0022\u0022,\u0022\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022b\u0022,\u0022indices\u0022:[[0,1],[0,0],[0,0],[0,0]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022()()(?<=(\u005c\u005c1\u005c\u005c2){2})b\u0022}"
        "()(?<=(\u005c1\u005cB){2})b" flags="d" input="b" "{\u0022match\u0022:null,\u0022lastIndex\u0022:0}"
        "(a)(?<=(\u005c1[ab]){2})b" flags="d" input="aaaaab" "{\u0022matches\u0022:[\u0022ab\u0022,\u0022a\u0022,\u0022aa\u0022],\u0022index\u0022:4,\u0022input\u0022:\u0022aaaaab\u0022,\u0022indices\u0022:[[4,6],[4,5],[1,3]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a)(?<=(\u005c\u005c1[ab]){2})b\u0022}"
        "(a)(?<=(.\u005c1){2})b" flags="d" input="aaaaab" "{\u0022matches\u0022:[\u0022ab\u0022,\u0022a\u0022,\u0022aa\u0022],\u0022index\u0022:4,\u0022input\u0022:\u0022aaaaab\u0022,\u0022indices\u0022:[[4,6],[4,5],[1,3]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a)(?<=(.\u005c\u005c1){2})b\u0022}"
        "(a)(?<=(\u005c1.){2})b" flags="d" input="aaaaab" "{\u0022matches\u0022:[\u0022ab\u0022,\u0022a\u0022,\u0022aa\u0022],\u0022index\u0022:4,\u0022input\u0022:\u0022aaaaab\u0022,\u0022indices\u0022:[[4,6],[4,5],[1,3]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a)(?<=(\u005c\u005c1.){2})b\u0022}"
        "(a)(?<=(^\u005c1){2})b" flags="d" input="aaab" "{\u0022match\u0022:null,\u0022lastIndex\u0022:0}"
        "(a)(?<=(\u005c1$){2})b" flags="d" input="aaab" "{\u0022match\u0022:null,\u0022lastIndex\u0022:0}"
        "(a)(?<!(\u005c1a){2}q)b" flags="d" input="aaaaab" "{\u0022matches\u0022:[\u0022ab\u0022,\u0022a\u0022,null],\u0022index\u0022:4,\u0022input\u0022:\u0022aaaaab\u0022,\u0022indices\u0022:[[4,6],[4,5],null],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a)(?<!(\u005c\u005c1a){2}q)b\u0022}"
        "(b)(?<!(\u005c1a){2}q)c" flags="d" input="bababc" "{\u0022matches\u0022:[\u0022bc\u0022,\u0022b\u0022,null],\u0022index\u0022:4,\u0022input\u0022:\u0022bababc\u0022,\u0022indices\u0022:[[4,6],[4,5],null],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(b)(?<!(\u005c\u005c1a){2}q)c\u0022}"
        "(a)(?<!(a\u005c1){2})b" flags="d" input="aaaaab" "{\u0022match\u0022:null,\u0022lastIndex\u0022:0}"
        "(?<=(\u005c2a){2})(a)" flags="d" input="aaa" "{\u0022matches\u0022:[\u0022a\u0022,\u0022a\u0022,\u0022a\u0022],\u0022index\u0022:2,\u0022input\u0022:\u0022aaa\u0022,\u0022indices\u0022:[[2,3],[0,1],[2,3]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=(\u005c\u005c2a){2})(a)\u0022}"
        "(a(?<=(\u005c1a){2}))b" flags="d" input="aaab" "{\u0022matches\u0022:[\u0022ab\u0022,\u0022a\u0022,\u0022a\u0022],\u0022index\u0022:2,\u0022input\u0022:\u0022aaab\u0022,\u0022indices\u0022:[[2,4],[2,3],[1,2]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a(?<=(\u005c\u005c1a){2}))b\u0022}"
        "(?=(a))(?<=(\u005c1[ab]\u005c1){2})a" flags="d" input="aaaaaaa" "{\u0022matches\u0022:[\u0022a\u0022,\u0022a\u0022,\u0022aaa\u0022],\u0022index\u0022:6,\u0022input\u0022:\u0022aaaaaaa\u0022,\u0022indices\u0022:[[6,7],[6,7],[0,3]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?=(a))(?<=(\u005c\u005c1[ab]\u005c\u005c1){2})a\u0022}"
        "(a)(?<=(a\u005c1){2}(?=b))b" flags="d" input="aaaaab" "{\u0022matches\u0022:[\u0022ab\u0022,\u0022a\u0022,\u0022aa\u0022],\u0022index\u0022:4,\u0022input\u0022:\u0022aaaaab\u0022,\u0022indices\u0022:[[4,6],[4,5],[1,3]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a)(?<=(a\u005c\u005c1){2}(?=b))b\u0022}"
        "(a)(?<=(?<=a)(a\u005c1){2})b" flags="d" input="aaaaab" "{\u0022matches\u0022:[\u0022ab\u0022,\u0022a\u0022,\u0022aa\u0022],\u0022index\u0022:4,\u0022input\u0022:\u0022aaaaab\u0022,\u0022indices\u0022:[[4,6],[4,5],[1,3]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a)(?<=(?<=a)(a\u005c\u005c1){2})b\u0022}"
        "(a)(?<=(a\u005c1){2}(?=(b)))b" flags="d" input="aaaaab" "{\u0022matches\u0022:[\u0022ab\u0022,\u0022a\u0022,\u0022aa\u0022,\u0022b\u0022],\u0022index\u0022:4,\u0022input\u0022:\u0022aaaaab\u0022,\u0022indices\u0022:[[4,6],[4,5],[1,3],[5,6]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a)(?<=(a\u005c\u005c1){2}(?=(b)))b\u0022}"
        "(?:(a|b)(?<=(.\u005c1){2}))+c" flags="d" input="aaaabbc" "{\u0022match\u0022:null,\u0022lastIndex\u0022:0}"
        "(\u00b5)(?<=(.\u005c1){2})\u039c" flags="di" input="\u00b5\u00b5\u00b5\u00b5\u00b5\u039c" "{\u0022matches\u0022:[\u0022\u00b5\u00b5\u0022,\u0022\u00b5\u0022,\u0022\u00b5\u00b5\u0022],\u0022index\u0022:3,\u0022input\u0022:\u0022\u00b5\u00b5\u00b5\u00b5\u00b5\u039c\u0022,\u0022indices\u0022:[[3,5],[3,4],[0,2]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(\u00b5)(?<=(.\u005c\u005c1){2})\u039c\u0022}"
        "([\u005cuD800])(?<=([\u005cuD800]\u005c1){2})[\u005cuDC00]" flags="d" input="\ud800\ud800\ud800\ud800\ud800\udc00" "{\u0022matches\u0022:[\u0022\ud800\udc00\u0022,\u0022\u005cud800\u0022,\u0022\u005cud800\u005cud800\u0022],\u0022index\u0022:4,\u0022input\u0022:\u0022\u005cud800\u005cud800\u005cud800\u005cud800\ud800\udc00\u0022,\u0022indices\u0022:[[4,6],[4,5],[1,3]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022([\u005c\u005cuD800])(?<=([\u005c\u005cuD800]\u005c\u005c1){2})[\u005c\u005cuDC00]\u0022}"
        "()()(?<=(\u005c1\u005c2\u005cb){30})b" flags="d" input="b" "{\u0022matches\u0022:[\u0022b\u0022,\u0022\u0022,\u0022\u0022,\u0022\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022b\u0022,\u0022indices\u0022:[[0,1],[0,0],[0,0],[0,0]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022()()(?<=(\u005c\u005c1\u005c\u005c2\u005c\u005cb){30})b\u0022}"
        "(?<x>a)(?<=(?<y>a\u005ck<x>){2})b" flags="d" input="aaaaab" "{\u0022matches\u0022:[\u0022ab\u0022,\u0022a\u0022,\u0022aa\u0022],\u0022index\u0022:4,\u0022input\u0022:\u0022aaaaab\u0022,\u0022groups\u0022:{\u0022x\u0022:\u0022a\u0022,\u0022y\u0022:\u0022aa\u0022},\u0022indices\u0022:[[4,6],[4,5],[1,3]],\u0022indicesGroups\u0022:{\u0022x\u0022:[4,5],\u0022y\u0022:[1,3]},\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<x>a)(?<=(?<y>a\u005c\u005ck<x>){2})b\u0022}"
        "(?<x>a)(?<z>bb)(?<=(?<y>\u005ck<x>\u005ck<z>){2})c" flags="d" input="abbabbabbc" "{\u0022matches\u0022:[\u0022abbc\u0022,\u0022a\u0022,\u0022bb\u0022,\u0022abb\u0022],\u0022index\u0022:6,\u0022input\u0022:\u0022abbabbabbc\u0022,\u0022groups\u0022:{\u0022x\u0022:\u0022a\u0022,\u0022z\u0022:\u0022bb\u0022,\u0022y\u0022:\u0022abb\u0022},\u0022indices\u0022:[[6,10],[6,7],[7,9],[3,6]],\u0022indicesGroups\u0022:{\u0022x\u0022:[6,7],\u0022z\u0022:[7,9],\u0022y\u0022:[3,6]},\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<x>a)(?<z>bb)(?<=(?<y>\u005c\u005ck<x>\u005c\u005ck<z>){2})c\u0022}"
        "(?<x>b)(?<!(?<y>\u005ck<x>a){2}q)c" flags="d" input="bababc" "{\u0022matches\u0022:[\u0022bc\u0022,\u0022b\u0022,null],\u0022index\u0022:4,\u0022input\u0022:\u0022bababc\u0022,\u0022groups\u0022:{\u0022x\u0022:\u0022b\u0022},\u0022indices\u0022:[[4,6],[4,5],null],\u0022indicesGroups\u0022:{\u0022x\u0022:[4,5]},\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<x>b)(?<!(?<y>\u005c\u005ck<x>a){2}q)c\u0022}"
        "(?=(?<x>a))(?<=(?<y>\u005ck<x>b){2})a" flags="d" input="ababa" "{\u0022matches\u0022:[\u0022a\u0022,\u0022a\u0022,\u0022ab\u0022],\u0022index\u0022:4,\u0022input\u0022:\u0022ababa\u0022,\u0022groups\u0022:{\u0022x\u0022:\u0022a\u0022,\u0022y\u0022:\u0022ab\u0022},\u0022indices\u0022:[[4,5],[4,5],[0,2]],\u0022indicesGroups\u0022:{\u0022x\u0022:[4,5],\u0022y\u0022:[0,2]},\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?=(?<x>a))(?<=(?<y>\u005c\u005ck<x>b){2})a\u0022}"
        "(?<=(?<y>\u005ck<x>a){2})(?<x>a)" flags="d" input="aaa" "{\u0022matches\u0022:[\u0022a\u0022,\u0022a\u0022,\u0022a\u0022],\u0022index\u0022:2,\u0022input\u0022:\u0022aaa\u0022,\u0022groups\u0022:{\u0022y\u0022:\u0022a\u0022,\u0022x\u0022:\u0022a\u0022},\u0022indices\u0022:[[2,3],[0,1],[2,3]],\u0022indicesGroups\u0022:{\u0022y\u0022:[0,1],\u0022x\u0022:[2,3]},\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=(?<y>\u005c\u005ck<x>a){2})(?<x>a)\u0022}"
        "(.)(?<=(a\u005c1){2})" flags="di" input="aaaaA" "{\u0022matches\u0022:[\u0022a\u0022,\u0022a\u0022,\u0022aa\u0022],\u0022index\u0022:3,\u0022input\u0022:\u0022aaaaA\u0022,\u0022indices\u0022:[[3,4],[3,4],[0,2]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(.)(?<=(a\u005c\u005c1){2})\u0022}"
        "(a)(?<=(\u005c1[ab]){2})b" flags="dm" input="q\u000aaaaab" "{\u0022matches\u0022:[\u0022ab\u0022,\u0022a\u0022,\u0022aa\u0022],\u0022index\u0022:5,\u0022input\u0022:\u0022q\u005cnaaaab\u0022,\u0022indices\u0022:[[5,7],[5,6],[2,4]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a)(?<=(\u005c\u005c1[ab]){2})b\u0022}"
        "#);
}
#[test]
fn names_aliases_different_widths_future_imports_and_scoped_negative_slots() {
    assert_true(
        r"let a=/(?=(a))(?<=(?:\1b){2})a/d.exec('ababa');a.index===4&&a.length===2&&a.indices[1][0]===4",
    );
    assert_true(
        r#"
            let a = /(?<x>a)(?<=(?<y>a\k<x>){2})b/d.exec('aaaaab');
            a.index === 4 && a.groups.y === 'aa' && a.indices.groups.y === a.indices[2] && a.indices[2][
                0] === 1 && a.indices[2][1] === 3
        "#,
    );
    assert_true(
        r#"
            let a = /(?<x>a)(?<z>bb)(?<=(?<y>\k<x>\k<z>){2})c/d.exec('abbabbabbc');
            a.index === 6 && a.groups.y === 'abb' && a.indices.groups.y[0] === 3 && a.indices.groups.y[
                1] === 6
        "#,
    );
    assert_true(
        r#"
            let a = /(?<x>b)(?<!(?<y>\k<x>a){2}q)c/d.exec('bababc');
            a.groups.x === 'b' && a.groups.y === undefined && a.indices.groups.y === undefined && Object
                .hasOwn(a.groups, 'y')
        "#,
    );
    assert_true(
        r#"
            let a = /(?=(?<x>a))(?<=(?<y>\k<x>b){2})a/d.exec('ababa');
            a.index === 4 && a.indices.groups.x[0] === 4 && a.indices.groups.y[0] === 0 && a.groups.y ===
                'ab'
        "#,
    );
    assert_true(
        r#"
            let a = /(?:(?<x>a)|(?<x>b))(?<=(a\k<x>){2})c/d.exec('abababc');
            a.index === 5 && a.groups.x === 'b' && a[1] === undefined && a[2] === 'b' && a.indices.groups
                .x === a.indices[2]
        "#,
    );
    assert_true(
        r"let a=/()()(?<=(\1\2\b){30})b/d.exec('b');a[3]===''&&a.indices[3][0]===0&&a.indices[3][1]===0",
    );
}
#[test]
fn consumers_callbacks_sticky_global_and_empty_advancement() {
    assert_true(
        r#"
            let r = /(a)(?<=(a\1){2})b/dg,
                a = [...'aaaaab aaaaab'.matchAll(r)];
            a.length === 2 && a[1].index === 11 && a[1][2] === 'aa' && a[1].indices[2][0] === 8 && r
                .lastIndex === 0
        "#,
    );
    assert_true(
        r#"
            let r = /(a)(?<=(a\1){2})b/dy;
            r.lastIndex = 4;
            let a = r.exec('aaaaab');
            a.index === 4 && r.lastIndex === 6 && r.exec('aaaaab') === null && r.lastIndex === 0
        "#,
    );
    assert_true(
        r#"
            let r = /()()(?<=(\1\2){2})/dg,
                a = [...'ab'.matchAll(r)];
            a.length === 3 && a[2].index === 2 && a[2][3] === '' && a[2].indices[3][0] === 2 && r
                .lastIndex === 0
        "#,
    );
    assert_true(
        r#"
            let seen = [];
            let s = 'aaaaab aaaaab'.replace(/(a)(?<=(a\1){2})b/g, (m, c, d, i) => {
                seen.push(c, d, i);
                return '_'
            });
            s === 'aaaa_ aaaa_' && seen.join('|') === 'a|aa|4|a|aa|11'
        "#,
    );
    assert_true(
        r"'aaaaab'.replace(/(?<x>a)(?<=(?<y>a\k<x>){2})b/,'<$<y>>')==='aaaa<aa>'&&'aaaaab'.search(/(a)(?<=(a\1){2})b/)===4&&'aaaaab'.split(/(a)(?<=(a\1){2})b/).join('|')==='aaaa|a|aa|'",
    );
}

#[test]
fn explicit_work_aborts_keep_last_index_and_bypass_handlers() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval(r"let marker=0,r=/(a)(?<=(a\1){2})b/g;r.lastIndex=1;let text='a'.repeat(5000)")
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
