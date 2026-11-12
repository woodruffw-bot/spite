//! Source-order top-level branches retain independent original capture slots.

use spite_core::JsString;
use spite_runtime::{Error, Limits, Realm, Value};
use std::fmt::Write;

use crate::common::assert_true;

#[test]
fn alternative_reference_result_snapshot() {
    let mut rows = String::new();
    for (source, flags, text) in [
        (r"(a)\1|(b)\2", "d", "qbb"),
        (r"(a)\1|b", "d", "baa"),
        (r"a|(ab)\1", "d", "abab"),
        (r"(ab)\1|a", "d", "abab"),
        (r"(a)\1|(a)\2", "d", "aa"),
        (r"(a)\1|", "d", "qaa"),
        (r"|(a)\1", "d", "aa"),
        (r"(a)\1|\1(b)", "d", "b"),
        (r"\2(a)\1|(b)\2", "d", "bb"),
        (r"(a)\1|(\2b)\2", "d", "bb"),
        (r"(\1a)\1|b", "d", "aa"),
        (r"^()\1$|(\b)\2\w", "d", " a"),
        (r"([µ])\1|(\w)\2", "di", "µΜ"),
        (r"([])\1|([^])\2", "d", "\n\n"),
        (r"(a)\1|\B(.)\2\B", "d", "\u{d7ff}\u{d7ff}"),
        (r"^^(a)\1$$|bb", "dm", "\nbb\n"),
        (r"(a)\1c|\1b", "d", "aab"),
        (r"(a)\1c|(\1b)\2", "d", "aabb"),
        (r"^(?<x>\w)\k<x>$|^(?<y>.)\k<y>$", "dims", "\n\n"),
        (r"(?<x>a)\k<x>|(?<y>b)\k<y>", "dgi", "qbB"),
        (r"(?<x>a)\k<x>|(?<y>b)\k<y>", "dy", "qbb"),
        (r"a|(?<x>ab)\k<x>", "d", "abab"),
    ] {
        let source = JsString::from(source);
        let flags = JsString::from(flags);
        let text = JsString::from(text);
        let program = format!(
            "let r=new RegExp({source:?},{flags:?}),a=r.exec({text:?});JSON.stringify(a===null?{{match:null,lastIndex:r.lastIndex}}:{{matches:[...a],index:a.index,input:a.input,groups:a.groups,indices:a.indices,indicesGroups:a.indices.groups,lastIndex:r.lastIndex,source:r.source}})"
        );
        let Value::String(value) = Realm::default().eval(&program).unwrap() else {
            panic!("expected JSON")
        };
        writeln!(rows, "{source:?} flags={flags:?} input={text:?} {value:?}").unwrap();
    }
    insta::assert_snapshot!(rows, @r#"
        "(a)\u005c1|(b)\u005c2" flags="d" input="qbb" "{\u0022matches\u0022:[\u0022bb\u0022,null,\u0022b\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022qbb\u0022,\u0022indices\u0022:[[1,3],null,[1,2]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a)\u005c\u005c1|(b)\u005c\u005c2\u0022}"
        "(a)\u005c1|b" flags="d" input="baa" "{\u0022matches\u0022:[\u0022b\u0022,null],\u0022index\u0022:0,\u0022input\u0022:\u0022baa\u0022,\u0022indices\u0022:[[0,1],null],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a)\u005c\u005c1|b\u0022}"
        "a|(ab)\u005c1" flags="d" input="abab" "{\u0022matches\u0022:[\u0022a\u0022,null],\u0022index\u0022:0,\u0022input\u0022:\u0022abab\u0022,\u0022indices\u0022:[[0,1],null],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022a|(ab)\u005c\u005c1\u0022}"
        "(ab)\u005c1|a" flags="d" input="abab" "{\u0022matches\u0022:[\u0022abab\u0022,\u0022ab\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022abab\u0022,\u0022indices\u0022:[[0,4],[0,2]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(ab)\u005c\u005c1|a\u0022}"
        "(a)\u005c1|(a)\u005c2" flags="d" input="aa" "{\u0022matches\u0022:[\u0022aa\u0022,\u0022a\u0022,null],\u0022index\u0022:0,\u0022input\u0022:\u0022aa\u0022,\u0022indices\u0022:[[0,2],[0,1],null],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a)\u005c\u005c1|(a)\u005c\u005c2\u0022}"
        "(a)\u005c1|" flags="d" input="qaa" "{\u0022matches\u0022:[\u0022\u0022,null],\u0022index\u0022:0,\u0022input\u0022:\u0022qaa\u0022,\u0022indices\u0022:[[0,0],null],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a)\u005c\u005c1|\u0022}"
        "|(a)\u005c1" flags="d" input="aa" "{\u0022matches\u0022:[\u0022\u0022,null],\u0022index\u0022:0,\u0022input\u0022:\u0022aa\u0022,\u0022indices\u0022:[[0,0],null],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022|(a)\u005c\u005c1\u0022}"
        "(a)\u005c1|\u005c1(b)" flags="d" input="b" "{\u0022matches\u0022:[\u0022b\u0022,null,\u0022b\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022b\u0022,\u0022indices\u0022:[[0,1],null,[0,1]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a)\u005c\u005c1|\u005c\u005c1(b)\u0022}"
        "\u005c2(a)\u005c1|(b)\u005c2" flags="d" input="bb" "{\u0022matches\u0022:[\u0022bb\u0022,null,\u0022b\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022bb\u0022,\u0022indices\u0022:[[0,2],null,[0,1]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022\u005c\u005c2(a)\u005c\u005c1|(b)\u005c\u005c2\u0022}"
        "(a)\u005c1|(\u005c2b)\u005c2" flags="d" input="bb" "{\u0022matches\u0022:[\u0022bb\u0022,null,\u0022b\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022bb\u0022,\u0022indices\u0022:[[0,2],null,[0,1]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a)\u005c\u005c1|(\u005c\u005c2b)\u005c\u005c2\u0022}"
        "(\u005c1a)\u005c1|b" flags="d" input="aa" "{\u0022matches\u0022:[\u0022aa\u0022,\u0022a\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022aa\u0022,\u0022indices\u0022:[[0,2],[0,1]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(\u005c\u005c1a)\u005c\u005c1|b\u0022}"
        "^()\u005c1$|(\u005cb)\u005c2\u005cw" flags="d" input=" a" "{\u0022matches\u0022:[\u0022a\u0022,null,\u0022\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022 a\u0022,\u0022indices\u0022:[[1,2],null,[1,1]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022^()\u005c\u005c1$|(\u005c\u005cb)\u005c\u005c2\u005c\u005cw\u0022}"
        "([\u00b5])\u005c1|(\u005cw)\u005c2" flags="di" input="\u00b5\u039c" "{\u0022matches\u0022:[\u0022\u00b5\u039c\u0022,\u0022\u00b5\u0022,null],\u0022index\u0022:0,\u0022input\u0022:\u0022\u00b5\u039c\u0022,\u0022indices\u0022:[[0,2],[0,1],null],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022([\u00b5])\u005c\u005c1|(\u005c\u005cw)\u005c\u005c2\u0022}"
        "([])\u005c1|([^])\u005c2" flags="d" input="\u000a\u000a" "{\u0022matches\u0022:[\u0022\u005cn\u005cn\u0022,null,\u0022\u005cn\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022\u005cn\u005cn\u0022,\u0022indices\u0022:[[0,2],null,[0,1]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022([])\u005c\u005c1|([^])\u005c\u005c2\u0022}"
        "(a)\u005c1|\u005cB(.)\u005c2\u005cB" flags="d" input="\ud7ff\ud7ff" "{\u0022matches\u0022:[\u0022\ud7ff\ud7ff\u0022,null,\u0022\ud7ff\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022\ud7ff\ud7ff\u0022,\u0022indices\u0022:[[0,2],null,[0,1]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a)\u005c\u005c1|\u005c\u005cB(.)\u005c\u005c2\u005c\u005cB\u0022}"
        "^^(a)\u005c1$$|bb" flags="dm" input="\u000abb\u000a" "{\u0022matches\u0022:[\u0022bb\u0022,null],\u0022index\u0022:1,\u0022input\u0022:\u0022\u005cnbb\u005cn\u0022,\u0022indices\u0022:[[1,3],null],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022^^(a)\u005c\u005c1$$|bb\u0022}"
        "(a)\u005c1c|\u005c1b" flags="d" input="aab" "{\u0022matches\u0022:[\u0022b\u0022,null],\u0022index\u0022:2,\u0022input\u0022:\u0022aab\u0022,\u0022indices\u0022:[[2,3],null],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a)\u005c\u005c1c|\u005c\u005c1b\u0022}"
        "(a)\u005c1c|(\u005c1b)\u005c2" flags="d" input="aabb" "{\u0022matches\u0022:[\u0022bb\u0022,null,\u0022b\u0022],\u0022index\u0022:2,\u0022input\u0022:\u0022aabb\u0022,\u0022indices\u0022:[[2,4],null,[2,3]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a)\u005c\u005c1c|(\u005c\u005c1b)\u005c\u005c2\u0022}"
        "^(?<x>\u005cw)\u005ck<x>$|^(?<y>.)\u005ck<y>$" flags="dims" input="\u000a\u000a" "{\u0022matches\u0022:[\u0022\u005cn\u005cn\u0022,null,\u0022\u005cn\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022\u005cn\u005cn\u0022,\u0022groups\u0022:{\u0022y\u0022:\u0022\u005cn\u0022},\u0022indices\u0022:[[0,2],null,[0,1]],\u0022indicesGroups\u0022:{\u0022y\u0022:[0,1]},\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022^(?<x>\u005c\u005cw)\u005c\u005ck<x>$|^(?<y>.)\u005c\u005ck<y>$\u0022}"
        "(?<x>a)\u005ck<x>|(?<y>b)\u005ck<y>" flags="dgi" input="qbB" "{\u0022matches\u0022:[\u0022bB\u0022,null,\u0022b\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022qbB\u0022,\u0022groups\u0022:{\u0022y\u0022:\u0022b\u0022},\u0022indices\u0022:[[1,3],null,[1,2]],\u0022indicesGroups\u0022:{\u0022y\u0022:[1,2]},\u0022lastIndex\u0022:3,\u0022source\u0022:\u0022(?<x>a)\u005c\u005ck<x>|(?<y>b)\u005c\u005ck<y>\u0022}"
        "(?<x>a)\u005ck<x>|(?<y>b)\u005ck<y>" flags="dy" input="qbb" "{\u0022match\u0022:null,\u0022lastIndex\u0022:0}"
        "a|(?<x>ab)\u005ck<x>" flags="d" input="abab" "{\u0022matches\u0022:[\u0022a\u0022,null],\u0022index\u0022:0,\u0022input\u0022:\u0022abab\u0022,\u0022groups\u0022:{},\u0022indices\u0022:[[0,1],null],\u0022indicesGroups\u0022:{},\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022a|(?<x>ab)\u005c\u005ck<x>\u0022}"
        "#);
}

#[test]
fn branch_order_and_failed_capture_resets_preserve_undefined_slots_and_indices_aliases() {
    assert_true(
        r#"
            let a = /(?<x>a)\k<x>c|(?<y>\k<x>b)\k<y>/d.exec('aabb');
            a.index === 2 && a[0] === 'bb' && a[1] === undefined && a.groups.x === undefined && a.groups
                .y === 'b' && a.indices[1] === undefined && a.indices.groups.x === undefined && a.indices
                .groups.y === a.indices[2] && a.indices[2][0] === 2 && a.indices[2][1] === 3
        "#,
    );
    assert_true(
        r#"
            let a = /(a)\1|b/d.exec('baa');
            a.index === 0 && a[0] === 'b' && a[1] === undefined && a.indices[1] === undefined && /a|(ab)\1/
                .exec('abab')[0] === 'a' && /(ab)\1|a/.exec('abab')[0] === 'abab'
        "#,
    );
    assert_true(
        r#"
            let a = /\2(a)\1|(b)\2/d.exec('bb');
            a[1] === undefined && a[2] === 'b' && a.indices[2][0] === 0 && /(a)\1|(\2b)\2/.exec('bb')[
                2] === 'b'
        "#,
    );
    assert_true(
        r#"
            let a = /(a)\1|\B(.)\2\B/d.exec('\uD800\uD800');
            a[1] === undefined && a[2].charCodeAt(0) === 0xD800 && a.indices[2][0] === 0 && a.indices[2][
                1
            ] === 1
        "#,
    );
    assert_true(
        r#"
            let a = /^(?<x>\w)\k<x>$|^(?<y>.)\k<y>$/dims.exec('\n\n');
            a.groups.x === undefined && a.groups.y === '\n' && a.indices.groups.y === a.indices[2]
        "#,
    );
}

#[test]
fn consumers_and_stateful_exec_keep_source_order_and_empty_branch_advancement() {
    assert_true(
        r"let r=/(?<x>a)\k<x>|(?<y>b)\k<y>/dgi,a=[...'aa bB aa'.matchAll(r)];a.length===3&&a[1].index===3&&a[1].groups.x===undefined&&a[1].groups.y==='b'&&a[1].indices.groups.y===a[1].indices[2]&&r.lastIndex===0",
    );
    assert_true(
        r"'bb'.replace(/(?<x>a)\k<x>|(?<y>b)\k<y>/,'<$<x>,$<y>>')==='<,b>'&&'qbb'.search(/(a)\1|(b)\2/)===1&&'qbbz'.split(/(a)\1|(b)\2/).join(',')==='q,,b,z'",
    );
    assert_true(
        r"let r=/(?<x>a)\k<x>|(?<y>b)\k<y>/dy;r.lastIndex=1;let a=r.exec('qbb');a.index===1&&a.groups.y==='b'&&r.lastIndex===3&&r.exec('qbb')===null&&r.lastIndex===0",
    );
    assert_true(
        r"let r=/(?<x>a)\k<x>|/dg,a=[...'qa'.matchAll(r)];a.length===3&&a[0].index===0&&a[2].index===2&&a[0].groups.x===undefined&&r.lastIndex===0",
    );
}

#[test]
fn explicit_branch_search_work_aborts_before_last_index_and_language_handlers() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval(r"let r=/(?<x>a)\k<x>c|(?<y>b)\k<y>c/g,flag=0,text='ab'.repeat(5000);r.lastIndex=1")
        .unwrap();
    assert!(matches!(
        realm.eval("try{r.exec(text)}catch{flag=1}finally{flag=2}"),
        Err(Error::Limit { .. })
    ));
    assert_eq!(
        realm.eval("flag===0&&r.lastIndex===1"),
        Ok(Value::Boolean(true))
    );
}

#[test]
fn nested_repeated_duplicate_name_and_unicode_reference_choices_match_reference_results() {
    for source in [
        r"/(?:(?:(?:(a|b)(?:\1)+){2})|){2,3}/.test('aa')",
        r"/(?:(?:(?:(a)(?:\1)+|(b)\2){2})|){2,3}/.test('aa')",
        r"/(?:(?:(?:(?<x>a)\k<x>|(?<x>b)(?:\k<x>)+){2})|){2,3}/.test('bb')",
        r"/(?<x>a)\k<x>|(?<y>b)\k<y>/u.test('bb')",
    ] {
        crate::common::regexp_oracle::assert_matches_reference(&mut Realm::default(), source);
    }
    assert_true(r"/(a)\1|(b)\2/.exec('ab')===null");
}
