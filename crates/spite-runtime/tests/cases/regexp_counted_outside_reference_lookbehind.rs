//! Outside capture references in linear ordinary lookbehind.
use crate::common::assert_true;
use spite_core::JsString;
use spite_runtime::{Error, Limits, Realm, Value};
use std::fmt::Write;
#[test]
fn counted_outside_reference_lookbehind_result_snapshot() {
    let mut rows = String::new();
    for (source, flags, text) in [
        (r"(a)(?<=\1{2})b", "d", "aaab"),
        (r"(a)(?<=\1{2}?)b", "d", "aaab"),
        (r"(a)(?<=(\1){2})b", "d", "aaab"),
        (r"(a)(?<=(\1\1){2})b", "d", "aaaaab"),
        (r"(..)(?<=\1{3})", "d", "ababab"),
        (r"(a)(?<=a\1{2})b", "d", "aaaab"),
        (r"(a)(?<=\1{2}a)b", "d", "aaaab"),
        (r"(a)(?<!\1{2})b", "d", "aaab"),
        (r"(a)(?<!(\1){2}q)b", "d", "aaab"),
        (r"(?<=\1{2})(a)", "d", "qa"),
        (r"(?<=(\2){2})a(a)", "d", "qa"),
        (r"(a(?<=\1{2}))b", "d", "qab"),
        (r"()(?<=(\1){2})b", "d", "qb"),
        (r"()(?<=\1{30})b", "d", "qb"),
        (r"(a)(?<=(\1){0}\1)b", "d", "qab"),
        (r"(a)(?<=(\1){1}\1)b", "d", "aaab"),
        (r"(a)(?<=(\1\1){2}?)b", "d", "aaaaab"),
        (r"(a)(?<=\1{2}\b)b", "d", "aaab"),
        (r"(a)(?<=\1{2}\B)b", "d", "aaab"),
        (r"(a)(?<=^\1{2})b", "d", "aab"),
        (r"(a)(?<=\1{2}(?=b))b", "d", "aaab"),
        (r"(a)(?<=\1{2}(?=(b)))b", "d", "aaab"),
        (r"(a)(?<=(?<=a)\1{2})b", "d", "aaab"),
        (r"(a)(?<=\1{2}(?<=a))b", "d", "aaab"),
        (r"(a)(?<=(a|b){0}\1{2})b", "d", "aaab"),
        (r"(?:(a|b)(?<=\1{2}))+c", "d", "aababc"),
        (r"(µ)(?<=\1{2})Μ", "di", "µµΜ"),
        (r"([\uD800])(?<=\1{2})[\uDC00]", "d", "surrogates"),
        (r"(a)(?<=(\1\1){1})b", "d", "aaab"),
        (r"(?=(a))(?<=\1{2})a", "d", "aa"),
        (r"(a)(?<=\1{30})b", "d", "ab"),
        (r"(a)(?<!\1{30})b", "d", "ab"),
        (r"(?<x>a)(?<=(?<y>\k<x>){2})b", "d", "aaab"),
        (r"(?:(?<x>a)|(?<x>b))(?<=\k<x>{1})c", "d", "bc"),
        (r"(?<x>a)(?<!(?<y>\k<x>){2}q)b", "d", "aaab"),
        (r"(?<x>a)(?<=\k<x>{1}(?=(?<y>b)))b", "d", "qab"),
        (r"(?<=\k<x>{2})(?<x>a)", "d", "qa"),
        (r"(.)(?<=(\1){2})", "di", "abB"),
        (r"(a)(?<=^\1)b", "dm", "q\nab"),
    ] {
        let source = JsString::from(source);
        let flags = JsString::from(flags);
        let input = if text == "surrogates" {
            JsString::from_code_units(vec![0xd800, 0xdc00])
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
        "(a)(?<=\u005c1{2})b" flags="d" input="aaab" "{\u0022matches\u0022:[\u0022ab\u0022,\u0022a\u0022],\u0022index\u0022:2,\u0022input\u0022:\u0022aaab\u0022,\u0022indices\u0022:[[2,4],[2,3]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a)(?<=\u005c\u005c1{2})b\u0022}"
        "(a)(?<=\u005c1{2}?)b" flags="d" input="aaab" "{\u0022matches\u0022:[\u0022ab\u0022,\u0022a\u0022],\u0022index\u0022:2,\u0022input\u0022:\u0022aaab\u0022,\u0022indices\u0022:[[2,4],[2,3]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a)(?<=\u005c\u005c1{2}?)b\u0022}"
        "(a)(?<=(\u005c1){2})b" flags="d" input="aaab" "{\u0022matches\u0022:[\u0022ab\u0022,\u0022a\u0022,\u0022a\u0022],\u0022index\u0022:2,\u0022input\u0022:\u0022aaab\u0022,\u0022indices\u0022:[[2,4],[2,3],[1,2]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a)(?<=(\u005c\u005c1){2})b\u0022}"
        "(a)(?<=(\u005c1\u005c1){2})b" flags="d" input="aaaaab" "{\u0022matches\u0022:[\u0022ab\u0022,\u0022a\u0022,\u0022aa\u0022],\u0022index\u0022:4,\u0022input\u0022:\u0022aaaaab\u0022,\u0022indices\u0022:[[4,6],[4,5],[1,3]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a)(?<=(\u005c\u005c1\u005c\u005c1){2})b\u0022}"
        "(..)(?<=\u005c1{3})" flags="d" input="ababab" "{\u0022matches\u0022:[\u0022ab\u0022,\u0022ab\u0022],\u0022index\u0022:4,\u0022input\u0022:\u0022ababab\u0022,\u0022indices\u0022:[[4,6],[4,6]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(..)(?<=\u005c\u005c1{3})\u0022}"
        "(a)(?<=a\u005c1{2})b" flags="d" input="aaaab" "{\u0022matches\u0022:[\u0022ab\u0022,\u0022a\u0022],\u0022index\u0022:3,\u0022input\u0022:\u0022aaaab\u0022,\u0022indices\u0022:[[3,5],[3,4]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a)(?<=a\u005c\u005c1{2})b\u0022}"
        "(a)(?<=\u005c1{2}a)b" flags="d" input="aaaab" "{\u0022matches\u0022:[\u0022ab\u0022,\u0022a\u0022],\u0022index\u0022:3,\u0022input\u0022:\u0022aaaab\u0022,\u0022indices\u0022:[[3,5],[3,4]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a)(?<=\u005c\u005c1{2}a)b\u0022}"
        "(a)(?<!\u005c1{2})b" flags="d" input="aaab" "{\u0022match\u0022:null,\u0022lastIndex\u0022:0}"
        "(a)(?<!(\u005c1){2}q)b" flags="d" input="aaab" "{\u0022matches\u0022:[\u0022ab\u0022,\u0022a\u0022,null],\u0022index\u0022:2,\u0022input\u0022:\u0022aaab\u0022,\u0022indices\u0022:[[2,4],[2,3],null],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a)(?<!(\u005c\u005c1){2}q)b\u0022}"
        "(?<=\u005c1{2})(a)" flags="d" input="qa" "{\u0022matches\u0022:[\u0022a\u0022,\u0022a\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022qa\u0022,\u0022indices\u0022:[[1,2],[1,2]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=\u005c\u005c1{2})(a)\u0022}"
        "(?<=(\u005c2){2})a(a)" flags="d" input="qa" "{\u0022match\u0022:null,\u0022lastIndex\u0022:0}"
        "(a(?<=\u005c1{2}))b" flags="d" input="qab" "{\u0022matches\u0022:[\u0022ab\u0022,\u0022a\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022qab\u0022,\u0022indices\u0022:[[1,3],[1,2]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a(?<=\u005c\u005c1{2}))b\u0022}"
        "()(?<=(\u005c1){2})b" flags="d" input="qb" "{\u0022matches\u0022:[\u0022b\u0022,\u0022\u0022,\u0022\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022qb\u0022,\u0022indices\u0022:[[1,2],[1,1],[1,1]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022()(?<=(\u005c\u005c1){2})b\u0022}"
        "()(?<=\u005c1{30})b" flags="d" input="qb" "{\u0022matches\u0022:[\u0022b\u0022,\u0022\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022qb\u0022,\u0022indices\u0022:[[1,2],[1,1]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022()(?<=\u005c\u005c1{30})b\u0022}"
        "(a)(?<=(\u005c1){0}\u005c1)b" flags="d" input="qab" "{\u0022matches\u0022:[\u0022ab\u0022,\u0022a\u0022,null],\u0022index\u0022:1,\u0022input\u0022:\u0022qab\u0022,\u0022indices\u0022:[[1,3],[1,2],null],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a)(?<=(\u005c\u005c1){0}\u005c\u005c1)b\u0022}"
        "(a)(?<=(\u005c1){1}\u005c1)b" flags="d" input="aaab" "{\u0022matches\u0022:[\u0022ab\u0022,\u0022a\u0022,\u0022a\u0022],\u0022index\u0022:2,\u0022input\u0022:\u0022aaab\u0022,\u0022indices\u0022:[[2,4],[2,3],[1,2]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a)(?<=(\u005c\u005c1){1}\u005c\u005c1)b\u0022}"
        "(a)(?<=(\u005c1\u005c1){2}?)b" flags="d" input="aaaaab" "{\u0022matches\u0022:[\u0022ab\u0022,\u0022a\u0022,\u0022aa\u0022],\u0022index\u0022:4,\u0022input\u0022:\u0022aaaaab\u0022,\u0022indices\u0022:[[4,6],[4,5],[1,3]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a)(?<=(\u005c\u005c1\u005c\u005c1){2}?)b\u0022}"
        "(a)(?<=\u005c1{2}\u005cb)b" flags="d" input="aaab" "{\u0022match\u0022:null,\u0022lastIndex\u0022:0}"
        "(a)(?<=\u005c1{2}\u005cB)b" flags="d" input="aaab" "{\u0022matches\u0022:[\u0022ab\u0022,\u0022a\u0022],\u0022index\u0022:2,\u0022input\u0022:\u0022aaab\u0022,\u0022indices\u0022:[[2,4],[2,3]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a)(?<=\u005c\u005c1{2}\u005c\u005cB)b\u0022}"
        "(a)(?<=^\u005c1{2})b" flags="d" input="aab" "{\u0022matches\u0022:[\u0022ab\u0022,\u0022a\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022aab\u0022,\u0022indices\u0022:[[1,3],[1,2]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a)(?<=^\u005c\u005c1{2})b\u0022}"
        "(a)(?<=\u005c1{2}(?=b))b" flags="d" input="aaab" "{\u0022matches\u0022:[\u0022ab\u0022,\u0022a\u0022],\u0022index\u0022:2,\u0022input\u0022:\u0022aaab\u0022,\u0022indices\u0022:[[2,4],[2,3]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a)(?<=\u005c\u005c1{2}(?=b))b\u0022}"
        "(a)(?<=\u005c1{2}(?=(b)))b" flags="d" input="aaab" "{\u0022matches\u0022:[\u0022ab\u0022,\u0022a\u0022,\u0022b\u0022],\u0022index\u0022:2,\u0022input\u0022:\u0022aaab\u0022,\u0022indices\u0022:[[2,4],[2,3],[3,4]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a)(?<=\u005c\u005c1{2}(?=(b)))b\u0022}"
        "(a)(?<=(?<=a)\u005c1{2})b" flags="d" input="aaab" "{\u0022matches\u0022:[\u0022ab\u0022,\u0022a\u0022],\u0022index\u0022:2,\u0022input\u0022:\u0022aaab\u0022,\u0022indices\u0022:[[2,4],[2,3]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a)(?<=(?<=a)\u005c\u005c1{2})b\u0022}"
        "(a)(?<=\u005c1{2}(?<=a))b" flags="d" input="aaab" "{\u0022matches\u0022:[\u0022ab\u0022,\u0022a\u0022],\u0022index\u0022:2,\u0022input\u0022:\u0022aaab\u0022,\u0022indices\u0022:[[2,4],[2,3]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a)(?<=\u005c\u005c1{2}(?<=a))b\u0022}"
        "(a)(?<=(a|b){0}\u005c1{2})b" flags="d" input="aaab" "{\u0022matches\u0022:[\u0022ab\u0022,\u0022a\u0022,null],\u0022index\u0022:2,\u0022input\u0022:\u0022aaab\u0022,\u0022indices\u0022:[[2,4],[2,3],null],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a)(?<=(a|b){0}\u005c\u005c1{2})b\u0022}"
        "(?:(a|b)(?<=\u005c1{2}))+c" flags="d" input="aababc" "{\u0022match\u0022:null,\u0022lastIndex\u0022:0}"
        "(\u00b5)(?<=\u005c1{2})\u039c" flags="di" input="\u00b5\u00b5\u039c" "{\u0022matches\u0022:[\u0022\u00b5\u039c\u0022,\u0022\u00b5\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022\u00b5\u00b5\u039c\u0022,\u0022indices\u0022:[[1,3],[1,2]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(\u00b5)(?<=\u005c\u005c1{2})\u039c\u0022}"
        "([\u005cuD800])(?<=\u005c1{2})[\u005cuDC00]" flags="d" input="\ud800\udc00" "{\u0022match\u0022:null,\u0022lastIndex\u0022:0}"
        "(a)(?<=(\u005c1\u005c1){1})b" flags="d" input="aaab" "{\u0022matches\u0022:[\u0022ab\u0022,\u0022a\u0022,\u0022aa\u0022],\u0022index\u0022:2,\u0022input\u0022:\u0022aaab\u0022,\u0022indices\u0022:[[2,4],[2,3],[1,3]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a)(?<=(\u005c\u005c1\u005c\u005c1){1})b\u0022}"
        "(?=(a))(?<=\u005c1{2})a" flags="d" input="aa" "{\u0022match\u0022:null,\u0022lastIndex\u0022:0}"
        "(a)(?<=\u005c1{30})b" flags="d" input="ab" "{\u0022match\u0022:null,\u0022lastIndex\u0022:0}"
        "(a)(?<!\u005c1{30})b" flags="d" input="ab" "{\u0022matches\u0022:[\u0022ab\u0022,\u0022a\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022ab\u0022,\u0022indices\u0022:[[0,2],[0,1]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a)(?<!\u005c\u005c1{30})b\u0022}"
        "(?<x>a)(?<=(?<y>\u005ck<x>){2})b" flags="d" input="aaab" "{\u0022matches\u0022:[\u0022ab\u0022,\u0022a\u0022,\u0022a\u0022],\u0022index\u0022:2,\u0022input\u0022:\u0022aaab\u0022,\u0022groups\u0022:{\u0022x\u0022:\u0022a\u0022,\u0022y\u0022:\u0022a\u0022},\u0022indices\u0022:[[2,4],[2,3],[1,2]],\u0022indicesGroups\u0022:{\u0022x\u0022:[2,3],\u0022y\u0022:[1,2]},\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<x>a)(?<=(?<y>\u005c\u005ck<x>){2})b\u0022}"
        "(?:(?<x>a)|(?<x>b))(?<=\u005ck<x>{1})c" flags="d" input="bc" "{\u0022matches\u0022:[\u0022bc\u0022,null,\u0022b\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022bc\u0022,\u0022groups\u0022:{\u0022x\u0022:\u0022b\u0022},\u0022indices\u0022:[[0,2],null,[0,1]],\u0022indicesGroups\u0022:{\u0022x\u0022:[0,1]},\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?:(?<x>a)|(?<x>b))(?<=\u005c\u005ck<x>{1})c\u0022}"
        "(?<x>a)(?<!(?<y>\u005ck<x>){2}q)b" flags="d" input="aaab" "{\u0022matches\u0022:[\u0022ab\u0022,\u0022a\u0022,null],\u0022index\u0022:2,\u0022input\u0022:\u0022aaab\u0022,\u0022groups\u0022:{\u0022x\u0022:\u0022a\u0022},\u0022indices\u0022:[[2,4],[2,3],null],\u0022indicesGroups\u0022:{\u0022x\u0022:[2,3]},\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<x>a)(?<!(?<y>\u005c\u005ck<x>){2}q)b\u0022}"
        "(?<x>a)(?<=\u005ck<x>{1}(?=(?<y>b)))b" flags="d" input="qab" "{\u0022matches\u0022:[\u0022ab\u0022,\u0022a\u0022,\u0022b\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022qab\u0022,\u0022groups\u0022:{\u0022x\u0022:\u0022a\u0022,\u0022y\u0022:\u0022b\u0022},\u0022indices\u0022:[[1,3],[1,2],[2,3]],\u0022indicesGroups\u0022:{\u0022x\u0022:[1,2],\u0022y\u0022:[2,3]},\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<x>a)(?<=\u005c\u005ck<x>{1}(?=(?<y>b)))b\u0022}"
        "(?<=\u005ck<x>{2})(?<x>a)" flags="d" input="qa" "{\u0022matches\u0022:[\u0022a\u0022,\u0022a\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022qa\u0022,\u0022groups\u0022:{\u0022x\u0022:\u0022a\u0022},\u0022indices\u0022:[[1,2],[1,2]],\u0022indicesGroups\u0022:{\u0022x\u0022:[1,2]},\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=\u005c\u005ck<x>{2})(?<x>a)\u0022}"
        "(.)(?<=(\u005c1){2})" flags="di" input="abB" "{\u0022matches\u0022:[\u0022B\u0022,\u0022B\u0022,\u0022b\u0022],\u0022index\u0022:2,\u0022input\u0022:\u0022abB\u0022,\u0022indices\u0022:[[2,3],[2,3],[1,2]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(.)(?<=(\u005c\u005c1){2})\u0022}"
        "(a)(?<=^\u005c1)b" flags="dm" input="q\u000aab" "{\u0022matches\u0022:[\u0022ab\u0022,\u0022a\u0022],\u0022index\u0022:2,\u0022input\u0022:\u0022q\u005cnab\u0022,\u0022indices\u0022:[[2,4],[2,3]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a)(?<=^\u005c\u005c1)b\u0022}"
        "#);
}
#[test]
fn names_aliases_before_after_match_ranges_and_negative_slots() {
    assert_true(
        r"let a=/(?<=(\2){2})a()/d.exec('qa');a.index===1&&a[1]===''&&a[2]===''&&a.indices[1][0]===1&&a.indices[2][0]===2",
    );
    assert_true(
        r"let a=/(?<x>a)(?<=(?<y>\k<x>){2})b/d.exec('aaab');a.index===2&&a.groups.x==='a'&&a.groups.y==='a'&&a.indices.groups.x===a.indices[1]&&a.indices.groups.y===a.indices[2]&&a.indices[2][0]===1&&a.indices[2][1]===2",
    );
    assert_true(
        r"let a=/(?:(?<x>a)|(?<x>b))(?<=\k<x>{1})c/d.exec('bc');a.groups.x==='b'&&a[1]===undefined&&a[2]==='b'&&a.indices.groups.x===a.indices[2]",
    );
    assert_true(
        r"let a=/(?<x>a)(?<!(?<y>\k<x>){2}q)b/d.exec('aaab');a.groups.x==='a'&&a.groups.y===undefined&&a.indices.groups.y===undefined&&Object.hasOwn(a.groups,'y')",
    );
    assert_true(
        r"let a=/(?<x>a)(?<=\k<x>{1}(?=(?<y>b)))b/d.exec('qab');a.index===1&&a.indices.groups.x[0]===1&&a.indices.groups.y[0]===2&&a.groups.y==='b'",
    );
    assert_true(
        r"let a=/(?<=\k<x>{2})(?<x>a)/d.exec('qa');a.index===1&&a.groups.x==='a'&&a.indices.groups.x[0]===1",
    );
}
#[test]
fn consumers_callbacks_sticky_global_and_empty_advancement() {
    assert_true(
        r"let r=/(a)(?<=(\1){2})b/dg,a=[...'aaab aaab'.matchAll(r)];a.length===2&&a[1].index===7&&a[1][2]==='a'&&a[1].indices[2][0]===6&&r.lastIndex===0",
    );
    assert_true(
        r"let r=/(a)(?<=\1{1})b/dy;r.lastIndex=1;let a=r.exec('qab');a.index===1&&r.lastIndex===3&&r.exec('qab')===null&&r.lastIndex===0",
    );
    assert_true(
        r"let r=/()(?<=\1{30})/dg,a=[...'ab'.matchAll(r)];a.length===3&&a[2].index===2&&a[2][1]===''&&a[2].indices[1][0]===2&&r.lastIndex===0",
    );
    assert_true(
        r"let seen=[];let s='aaab aaab'.replace(/(a)(?<=(\1){2})b/g,(m,c,d,i)=>{seen.push(c,d,i);return '_'});s==='aa_ aa_'&&seen.join('|')==='a|a|2|a|a|7'",
    );
    assert_true(
        r"'aaab'.replace(/(?<x>a)(?<=(?<y>\k<x>){2})b/,'<$<y>>')==='aa<a>'&&'qab'.search(/(a)(?<=\1)b/)===1&&'qab'.split(/(a)(?<=\1)b/).join('|')==='q|a|'",
    );
}

#[test]
fn explicit_work_aborts_keep_last_index_and_bypass_handlers() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval(r"let marker=0,r=/(a)(?<=\1{2})b/g;r.lastIndex=1;let text='a'.repeat(5000)")
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
fn internal_reads_dependent_children_variable_counts_choices_and_unicode_match_reference_results() {
    for source in [
        r"/(?<=(\1{1,2}a{1,2}))a/.exec('a')",
        r"/(a)(?<=(?<=\1{1,2}))b/.exec('ab')",
        r"/(a)(?<=(?=\1{1,2})\1)b/.exec('ab')",
        r"/(a)(?<=\1|a)b/.exec('ab')",
        r"/(a)(?<=\1{1,2})b/.exec('aab')",
        r"/(a)(?<=\1+)b/.exec('ab')",
        r"/(a)(?<=\1)b/u.exec('ab')",
        r"/(a)(?<=\1)b/v.exec('ab')",
    ] {
        crate::common::regexp_oracle::assert_matches_reference(&mut Realm::default(), source);
    }
}
