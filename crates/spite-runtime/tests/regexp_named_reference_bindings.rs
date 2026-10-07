//! Mutually exclusive named targets select original input captures directly.

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
    insta::assert_snapshot!(rows);
}

#[test]
fn duplicate_names_select_active_slots_and_preserve_indices_and_forward_state() {
    check(
        r"let a=/(?<x>a)\k<x>c|(?<x>b)\k<x>/d.exec('aabb');a.index===2&&a[0]==='bb'&&a[1]===undefined&&a[2]==='b'&&a.groups.x==='b'&&a.indices.groups.x===a.indices[2]&&a.indices[2][0]===2&&a.indices[2][1]===3",
    );
    check(
        r"let a=/(?<x>a)(?<y>b)\k<x>\k<y>|(?<x>b)(?<y>a)\k<x>\k<y>/d.exec('baba');a[1]===undefined&&a[2]===undefined&&a.groups.x==='b'&&a.groups.y==='a'&&a.indices.groups.x===a.indices[3]&&a.indices.groups.y===a.indices[4]&&Object.getOwnPropertyNames(a.groups).join(',')==='x,y'",
    );
    check(
        r"/(?<x>a)|(?<x>\k<x>b)\k<x>/.exec('bb').groups.x==='b'&&/\k<x>(?<x>a)|(?<x>b)\k<x>/.exec('aa')[0]==='a'&&/(?<x>a)\k<x>0|(?<x>b)\k<x>0/.exec('bb0')[0]==='bb0'",
    );
    check(
        r"let a=/(?<__proto__>a)\k<__proto__>|(?<__proto__>b)\k<__proto__>/d.exec('bb');Object.getPrototypeOf(a.groups)===null&&a.groups.__proto__==='b'&&a.indices.groups.__proto__===a.indices[2]",
    );
    check(
        r"let a=/(?<x>a)\k<x>|\B(?<x>.)\k<x>\B/d.exec('\uD800\uD800');a.groups.x.charCodeAt(0)===0xD800&&a.indices.groups.x===a.indices[2]&&a.indices[2][1]===1",
    );
}

#[test]
fn consumers_and_sticky_copies_keep_original_sources_and_named_results() {
    check(
        r"let r=/(?<x>a)\k<x>|(?<x>b)\k<x>/dgi,a=[...'aa bB aa'.matchAll(r)];a.length===3&&a[1].groups.x==='b'&&a[1].indices.groups.x===a[1].indices[2]&&r.lastIndex===0",
    );
    check(
        r"'bb'.replace(/(?<x>a)\k<x>|(?<x>b)\k<x>/,'<$<x>>')==='<b>'&&'qbb'.search(/(?<x>a)\k<x>|(?<x>b)\k<x>/)===1&&'qbbz'.split(/(?<x>a)\k<x>|(?<x>b)\k<x>/).join(',')==='q,,b,z'",
    );
    check(
        r"let r=/(?<x>a)\k<x>|(?<x>b)\k<x>/dy;r.lastIndex=1;let copy=new RegExp(r);copy.source===r.source&&copy.lastIndex===0&&r.exec('qbb').groups.x==='b'&&r.lastIndex===3&&r.exec('qbb')===null&&r.lastIndex===0",
    );
    check(
        r"let a=[...'qa'.matchAll(/(?<x>)\k<x>|(?<x>a)\k<x>/dg)];a.length===3&&a[2].index===2&&a[0].groups.x===''&&a[0].indices.groups.x===a[0].indices[1]",
    );
}

#[test]
fn wide_shared_name_inventories_and_references_survive_copy_and_collection() {
    let mut realm = Realm::default();
    realm.eval("let r=new RegExp('(?<x>b)|'.repeat(10000)+'(?<x>a)'+'\\\\k<x>'.repeat(10000),'d'),copy=new RegExp(r)").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("let a=copy.exec('a'.repeat(10001));a[0].length===10001&&a.length===10002&&a[1]===undefined&&a[10000]===undefined&&a.groups.x==='a'&&a.indices.groups.x===a.indices[10001]&&copy.source===r.source"),Ok(Value::Boolean(true)));
    realm.collect(usize::MAX).unwrap();
    assert_eq!(
        realm.eval("a.groups.x==='a'&&a.indices.groups.x===a.indices[10001]"),
        Ok(Value::Boolean(true))
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
fn wider_named_binding_compositions_keep_unsupported_outcomes() {
    for source in [
        r"/(?:(?<x>a)|(?<x>b))(?:\k<x>)+/.test('bb')",
        r"/(?<x>a)\k<x>|(?<x>b)(?:\k<x>)+/.test('bb')",
        r"/(?<x>a)\k<x>|(?<x>b)\k<x>/u.test('bb')",
        r"/(?=(?<x>a))\k<x>/.test('a')",
    ] {
        assert!(
            matches!(
                Realm::default().eval(source),
                Err(Error::Unsupported { .. })
            ),
            "{source}"
        );
    }
    check(r"/(?<x>a)\k<x>|(?<x>b)\k<x>/.exec('ab')===null");
}
