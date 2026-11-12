//! Mutually exclusive named targets select original input captures directly.

use spite_core::JsString;
use spite_runtime::{Error, Limits, Realm, Value};
use std::fmt::Write;

use crate::common::assert_true;

#[test]
fn named_binding_result_snapshot() {
    let mut rows = String::new();
    for (source, flags, text) in [
        (r"(?<x>a)\k<x>|(?<x>b)\k<x>", "d", "bb"),
        (r"(?<x>a)|(?<x>\k<x>b)\k<x>", "d", "bb"),
        (r"(?<x>\k<x>a)\k<x>|(?<x>b)\k<x>", "d", "aa"),
        (r"\k<x>(?<x>a)|(?<x>b)\k<x>", "d", "aa"),
        (r"(?<x>a)\k<x>0|(?<x>b)\k<x>0", "d", "bb0"),
        (r"((?<x>a)\k<x>)|(?<x>b)\k<x>", "d", "qbb"),
        (r"(?<x>[µ])\k<x>|(?<x>\w)\k<x>", "di", "µΜ"),
        (r"(?<x>a)\k<x>c|(?<x>b)\k<x>", "d", "aabb"),
        (r"^(?<x>)\k<x>$|(?<x>\b)\k<x>\w", "d", " a"),
        (
            r"(?<x>a)(?<y>b)\k<x>\k<y>|(?<x>b)(?<y>a)\k<x>\k<y>",
            "d",
            "baba",
        ),
        (r"^(?<x>[\s\S])\k<x>$|^(?<x>[ab])\k<x>$", "d", "\n\n"),
        (r"(?<x>a)\k<x>|", "dg", "qaa"),
        (r"^(?<x>\w)\k<x>$|^(?<x>.)\k<x>$", "dims", "\n\n"),
        (r"(?<x>a)\k<x>|(?<x>b)\k<x>", "dgi", "qbB"),
        (r"(?<x>a)\k<x>|(?<x>b)\k<x>", "dy", "qbb"),
        (r"(?<x>)\k<x>|(?<x>a)\k<x>", "dg", "aa"),
        (
            r"(?<__proto__>a)\k<__proto__>|(?<__proto__>b)\k<__proto__>",
            "d",
            "bb",
        ),
        (r"(?<\u0078>a)\k<\u{78}>|(?<x>b)\k<x>", "d", "bb"),
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
        "(?<x>a)\u005ck<x>|(?<x>b)\u005ck<x>" flags="d" input="bb" "{\u0022matches\u0022:[\u0022bb\u0022,null,\u0022b\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022bb\u0022,\u0022groups\u0022:{\u0022x\u0022:\u0022b\u0022},\u0022indices\u0022:[[0,2],null,[0,1]],\u0022indicesGroups\u0022:{\u0022x\u0022:[0,1]},\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<x>a)\u005c\u005ck<x>|(?<x>b)\u005c\u005ck<x>\u0022}"
        "(?<x>a)|(?<x>\u005ck<x>b)\u005ck<x>" flags="d" input="bb" "{\u0022matches\u0022:[\u0022bb\u0022,null,\u0022b\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022bb\u0022,\u0022groups\u0022:{\u0022x\u0022:\u0022b\u0022},\u0022indices\u0022:[[0,2],null,[0,1]],\u0022indicesGroups\u0022:{\u0022x\u0022:[0,1]},\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<x>a)|(?<x>\u005c\u005ck<x>b)\u005c\u005ck<x>\u0022}"
        "(?<x>\u005ck<x>a)\u005ck<x>|(?<x>b)\u005ck<x>" flags="d" input="aa" "{\u0022matches\u0022:[\u0022aa\u0022,\u0022a\u0022,null],\u0022index\u0022:0,\u0022input\u0022:\u0022aa\u0022,\u0022groups\u0022:{\u0022x\u0022:\u0022a\u0022},\u0022indices\u0022:[[0,2],[0,1],null],\u0022indicesGroups\u0022:{\u0022x\u0022:[0,1]},\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<x>\u005c\u005ck<x>a)\u005c\u005ck<x>|(?<x>b)\u005c\u005ck<x>\u0022}"
        "\u005ck<x>(?<x>a)|(?<x>b)\u005ck<x>" flags="d" input="aa" "{\u0022matches\u0022:[\u0022a\u0022,\u0022a\u0022,null],\u0022index\u0022:0,\u0022input\u0022:\u0022aa\u0022,\u0022groups\u0022:{\u0022x\u0022:\u0022a\u0022},\u0022indices\u0022:[[0,1],[0,1],null],\u0022indicesGroups\u0022:{\u0022x\u0022:[0,1]},\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022\u005c\u005ck<x>(?<x>a)|(?<x>b)\u005c\u005ck<x>\u0022}"
        "(?<x>a)\u005ck<x>0|(?<x>b)\u005ck<x>0" flags="d" input="bb0" "{\u0022matches\u0022:[\u0022bb0\u0022,null,\u0022b\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022bb0\u0022,\u0022groups\u0022:{\u0022x\u0022:\u0022b\u0022},\u0022indices\u0022:[[0,3],null,[0,1]],\u0022indicesGroups\u0022:{\u0022x\u0022:[0,1]},\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<x>a)\u005c\u005ck<x>0|(?<x>b)\u005c\u005ck<x>0\u0022}"
        "((?<x>a)\u005ck<x>)|(?<x>b)\u005ck<x>" flags="d" input="qbb" "{\u0022matches\u0022:[\u0022bb\u0022,null,null,\u0022b\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022qbb\u0022,\u0022groups\u0022:{\u0022x\u0022:\u0022b\u0022},\u0022indices\u0022:[[1,3],null,null,[1,2]],\u0022indicesGroups\u0022:{\u0022x\u0022:[1,2]},\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022((?<x>a)\u005c\u005ck<x>)|(?<x>b)\u005c\u005ck<x>\u0022}"
        "(?<x>[\u00b5])\u005ck<x>|(?<x>\u005cw)\u005ck<x>" flags="di" input="\u00b5\u039c" "{\u0022matches\u0022:[\u0022\u00b5\u039c\u0022,\u0022\u00b5\u0022,null],\u0022index\u0022:0,\u0022input\u0022:\u0022\u00b5\u039c\u0022,\u0022groups\u0022:{\u0022x\u0022:\u0022\u00b5\u0022},\u0022indices\u0022:[[0,2],[0,1],null],\u0022indicesGroups\u0022:{\u0022x\u0022:[0,1]},\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<x>[\u00b5])\u005c\u005ck<x>|(?<x>\u005c\u005cw)\u005c\u005ck<x>\u0022}"
        "(?<x>a)\u005ck<x>c|(?<x>b)\u005ck<x>" flags="d" input="aabb" "{\u0022matches\u0022:[\u0022bb\u0022,null,\u0022b\u0022],\u0022index\u0022:2,\u0022input\u0022:\u0022aabb\u0022,\u0022groups\u0022:{\u0022x\u0022:\u0022b\u0022},\u0022indices\u0022:[[2,4],null,[2,3]],\u0022indicesGroups\u0022:{\u0022x\u0022:[2,3]},\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<x>a)\u005c\u005ck<x>c|(?<x>b)\u005c\u005ck<x>\u0022}"
        "^(?<x>)\u005ck<x>$|(?<x>\u005cb)\u005ck<x>\u005cw" flags="d" input=" a" "{\u0022matches\u0022:[\u0022a\u0022,null,\u0022\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022 a\u0022,\u0022groups\u0022:{\u0022x\u0022:\u0022\u0022},\u0022indices\u0022:[[1,2],null,[1,1]],\u0022indicesGroups\u0022:{\u0022x\u0022:[1,1]},\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022^(?<x>)\u005c\u005ck<x>$|(?<x>\u005c\u005cb)\u005c\u005ck<x>\u005c\u005cw\u0022}"
        "(?<x>a)(?<y>b)\u005ck<x>\u005ck<y>|(?<x>b)(?<y>a)\u005ck<x>\u005ck<y>" flags="d" input="baba" "{\u0022matches\u0022:[\u0022baba\u0022,null,null,\u0022b\u0022,\u0022a\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022baba\u0022,\u0022groups\u0022:{\u0022x\u0022:\u0022b\u0022,\u0022y\u0022:\u0022a\u0022},\u0022indices\u0022:[[0,4],null,null,[0,1],[1,2]],\u0022indicesGroups\u0022:{\u0022x\u0022:[0,1],\u0022y\u0022:[1,2]},\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<x>a)(?<y>b)\u005c\u005ck<x>\u005c\u005ck<y>|(?<x>b)(?<y>a)\u005c\u005ck<x>\u005c\u005ck<y>\u0022}"
        "^(?<x>[\u005cs\u005cS])\u005ck<x>$|^(?<x>[ab])\u005ck<x>$" flags="d" input="\u000a\u000a" "{\u0022matches\u0022:[\u0022\u005cn\u005cn\u0022,\u0022\u005cn\u0022,null],\u0022index\u0022:0,\u0022input\u0022:\u0022\u005cn\u005cn\u0022,\u0022groups\u0022:{\u0022x\u0022:\u0022\u005cn\u0022},\u0022indices\u0022:[[0,2],[0,1],null],\u0022indicesGroups\u0022:{\u0022x\u0022:[0,1]},\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022^(?<x>[\u005c\u005cs\u005c\u005cS])\u005c\u005ck<x>$|^(?<x>[ab])\u005c\u005ck<x>$\u0022}"
        "(?<x>a)\u005ck<x>|" flags="dg" input="qaa" "{\u0022matches\u0022:[\u0022\u0022,null],\u0022index\u0022:0,\u0022input\u0022:\u0022qaa\u0022,\u0022groups\u0022:{},\u0022indices\u0022:[[0,0],null],\u0022indicesGroups\u0022:{},\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<x>a)\u005c\u005ck<x>|\u0022}"
        "^(?<x>\u005cw)\u005ck<x>$|^(?<x>.)\u005ck<x>$" flags="dims" input="\u000a\u000a" "{\u0022matches\u0022:[\u0022\u005cn\u005cn\u0022,null,\u0022\u005cn\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022\u005cn\u005cn\u0022,\u0022groups\u0022:{\u0022x\u0022:\u0022\u005cn\u0022},\u0022indices\u0022:[[0,2],null,[0,1]],\u0022indicesGroups\u0022:{\u0022x\u0022:[0,1]},\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022^(?<x>\u005c\u005cw)\u005c\u005ck<x>$|^(?<x>.)\u005c\u005ck<x>$\u0022}"
        "(?<x>a)\u005ck<x>|(?<x>b)\u005ck<x>" flags="dgi" input="qbB" "{\u0022matches\u0022:[\u0022bB\u0022,null,\u0022b\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022qbB\u0022,\u0022groups\u0022:{\u0022x\u0022:\u0022b\u0022},\u0022indices\u0022:[[1,3],null,[1,2]],\u0022indicesGroups\u0022:{\u0022x\u0022:[1,2]},\u0022lastIndex\u0022:3,\u0022source\u0022:\u0022(?<x>a)\u005c\u005ck<x>|(?<x>b)\u005c\u005ck<x>\u0022}"
        "(?<x>a)\u005ck<x>|(?<x>b)\u005ck<x>" flags="dy" input="qbb" "{\u0022match\u0022:null,\u0022lastIndex\u0022:0}"
        "(?<x>)\u005ck<x>|(?<x>a)\u005ck<x>" flags="dg" input="aa" "{\u0022matches\u0022:[\u0022\u0022,\u0022\u0022,null],\u0022index\u0022:0,\u0022input\u0022:\u0022aa\u0022,\u0022groups\u0022:{\u0022x\u0022:\u0022\u0022},\u0022indices\u0022:[[0,0],[0,0],null],\u0022indicesGroups\u0022:{\u0022x\u0022:[0,0]},\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<x>)\u005c\u005ck<x>|(?<x>a)\u005c\u005ck<x>\u0022}"
        "(?<__proto__>a)\u005ck<__proto__>|(?<__proto__>b)\u005ck<__proto__>" flags="d" input="bb" "{\u0022matches\u0022:[\u0022bb\u0022,null,\u0022b\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022bb\u0022,\u0022groups\u0022:{\u0022__proto__\u0022:\u0022b\u0022},\u0022indices\u0022:[[0,2],null,[0,1]],\u0022indicesGroups\u0022:{\u0022__proto__\u0022:[0,1]},\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<__proto__>a)\u005c\u005ck<__proto__>|(?<__proto__>b)\u005c\u005ck<__proto__>\u0022}"
        "(?<\u005cu0078>a)\u005ck<\u005cu{78}>|(?<x>b)\u005ck<x>" flags="d" input="bb" "{\u0022matches\u0022:[\u0022bb\u0022,null,\u0022b\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022bb\u0022,\u0022groups\u0022:{\u0022x\u0022:\u0022b\u0022},\u0022indices\u0022:[[0,2],null,[0,1]],\u0022indicesGroups\u0022:{\u0022x\u0022:[0,1]},\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<\u005c\u005cu0078>a)\u005c\u005ck<\u005c\u005cu{78}>|(?<x>b)\u005c\u005ck<x>\u0022}"
        "#);
}

#[test]
fn duplicate_names_select_active_slots_and_preserve_indices_and_forward_state() {
    assert_true(
        r#"
            let a = /(?<x>a)\k<x>c|(?<x>b)\k<x>/d.exec('aabb');
            a.index === 2 && a[0] === 'bb' && a[1] === undefined && a[2] === 'b' && a.groups.x === 'b' && a
                .indices.groups.x === a.indices[2] && a.indices[2][0] === 2 && a.indices[2][1] === 3
        "#,
    );
    assert_true(
        r#"
            let a = /(?<x>a)(?<y>b)\k<x>\k<y>|(?<x>b)(?<y>a)\k<x>\k<y>/d.exec('baba');
            a[1] === undefined && a[2] === undefined && a.groups.x === 'b' && a.groups.y === 'a' && a
                .indices.groups.x === a.indices[3] && a.indices.groups.y === a.indices[4] && Object
                .getOwnPropertyNames(a.groups).join(',') === 'x,y'
        "#,
    );
    assert_true(
        r"/(?<x>a)|(?<x>\k<x>b)\k<x>/.exec('bb').groups.x==='b'&&/\k<x>(?<x>a)|(?<x>b)\k<x>/.exec('aa')[0]==='a'&&/(?<x>a)\k<x>0|(?<x>b)\k<x>0/.exec('bb0')[0]==='bb0'",
    );
    assert_true(
        r#"
            let a = /(?<__proto__>a)\k<__proto__>|(?<__proto__>b)\k<__proto__>/d.exec('bb');
            Object.getPrototypeOf(a.groups) === null && a.groups.__proto__ === 'b' && a.indices.groups
                .__proto__ === a.indices[2]
        "#,
    );
    assert_true(
        r#"
            let a = /(?<x>a)\k<x>|\B(?<x>.)\k<x>\B/d.exec('\uD800\uD800');
            a.groups.x.charCodeAt(0) === 0xD800 && a.indices.groups.x === a.indices[2] && a.indices[2][
                1
            ] === 1
        "#,
    );
}

#[test]
fn consumers_and_sticky_copies_keep_original_sources_and_named_results() {
    assert_true(
        r#"
            let r = /(?<x>a)\k<x>|(?<x>b)\k<x>/dgi,
                a = [...'aa bB aa'.matchAll(r)];
            a.length === 3 && a[1].groups.x === 'b' && a[1].indices.groups.x === a[1].indices[2] && r
                .lastIndex === 0
        "#,
    );
    assert_true(
        r"'bb'.replace(/(?<x>a)\k<x>|(?<x>b)\k<x>/,'<$<x>>')==='<b>'&&'qbb'.search(/(?<x>a)\k<x>|(?<x>b)\k<x>/)===1&&'qbbz'.split(/(?<x>a)\k<x>|(?<x>b)\k<x>/).join(',')==='q,,b,z'",
    );
    assert_true(
        r#"
            let r = /(?<x>a)\k<x>|(?<x>b)\k<x>/dy;
            r.lastIndex = 1;
            let copy = new RegExp(r);
            copy.source === r.source && copy.lastIndex === 0 && r.exec('qbb').groups.x === 'b' && r
                .lastIndex === 3 && r.exec('qbb') === null && r.lastIndex === 0
        "#,
    );
    assert_true(
        r#"
            let a = [...'qa'.matchAll(/(?<x>)\k<x>|(?<x>a)\k<x>/dg)];
            a.length === 3 && a[2].index === 2 && a[0].groups.x === '' && a[0].indices.groups.x === a[0]
                .indices[1]
        "#,
    );
}

#[test]
fn explicit_duplicate_name_search_work_aborts_before_last_index_and_handlers() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval(r"let r=/(?<x>a)\k<x>c|(?<x>b)\k<x>c/g,flag=0,text='ab'.repeat(5000);r.lastIndex=1")
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
fn pattern_variants_match_reference_results() {
    for source in [
        r"/(?:(?:(?:(?:(?<x>a)|(?<x>b))(?:\k<x>)+){2})|){2,3}/.test('bb')",
        r"/(?:(?:(?:(?<x>a)\k<x>|(?<x>b)(?:\k<x>)+){2})|){2,3}/.test('bb')",
        r"/(?<x>a)\k<x>|(?<x>b)\k<x>/u.test('bb')",
        r"/(?<=(?<x>a+))\k<x>/.test('a')",
    ] {
        crate::common::regexp_oracle::assert_matches_reference(&mut Realm::default(), source);
    }
    assert_true(r"/(?<x>a)\k<x>|(?<x>b)\k<x>/.exec('ab')===null");
}
