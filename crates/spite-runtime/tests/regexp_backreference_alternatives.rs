//! Source-order top-level branches retain independent original capture slots.

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
    insta::assert_snapshot!(rows);
}

#[test]
fn branch_order_and_failed_capture_resets_preserve_undefined_slots_and_indices_aliases() {
    check(
        r"let a=/(?<x>a)\k<x>c|(?<y>\k<x>b)\k<y>/d.exec('aabb');a.index===2&&a[0]==='bb'&&a[1]===undefined&&a.groups.x===undefined&&a.groups.y==='b'&&a.indices[1]===undefined&&a.indices.groups.x===undefined&&a.indices.groups.y===a.indices[2]&&a.indices[2][0]===2&&a.indices[2][1]===3",
    );
    check(
        r"let a=/(a)\1|b/d.exec('baa');a.index===0&&a[0]==='b'&&a[1]===undefined&&a.indices[1]===undefined&&/a|(ab)\1/.exec('abab')[0]==='a'&&/(ab)\1|a/.exec('abab')[0]==='abab'",
    );
    check(
        r"let a=/\2(a)\1|(b)\2/d.exec('bb');a[1]===undefined&&a[2]==='b'&&a.indices[2][0]===0&&/(a)\1|(\2b)\2/.exec('bb')[2]==='b'",
    );
    check(
        r"let a=/(a)\1|\B(.)\2\B/d.exec('\uD800\uD800');a[1]===undefined&&a[2].charCodeAt(0)===0xD800&&a.indices[2][0]===0&&a.indices[2][1]===1",
    );
    check(
        r"let a=/^(?<x>\w)\k<x>$|^(?<y>.)\k<y>$/dims.exec('\n\n');a.groups.x===undefined&&a.groups.y==='\n'&&a.indices.groups.y===a.indices[2]",
    );
}

#[test]
fn consumers_and_stateful_exec_keep_source_order_and_empty_branch_advancement() {
    check(
        r"let r=/(?<x>a)\k<x>|(?<y>b)\k<y>/dgi,a=[...'aa bB aa'.matchAll(r)];a.length===3&&a[1].index===3&&a[1].groups.x===undefined&&a[1].groups.y==='b'&&a[1].indices.groups.y===a[1].indices[2]&&r.lastIndex===0",
    );
    check(
        r"'bb'.replace(/(?<x>a)\k<x>|(?<y>b)\k<y>/,'<$<x>,$<y>>')==='<,b>'&&'qbb'.search(/(a)\1|(b)\2/)===1&&'qbbz'.split(/(a)\1|(b)\2/).join(',')==='q,,b,z'",
    );
    check(
        r"let r=/(?<x>a)\k<x>|(?<y>b)\k<y>/dy;r.lastIndex=1;let a=r.exec('qbb');a.index===1&&a.groups.y==='b'&&r.lastIndex===3&&r.exec('qbb')===null&&r.lastIndex===0",
    );
    check(
        r"let r=/(?<x>a)\k<x>|/dg,a=[...'qa'.matchAll(r)];a.length===3&&a[0].index===0&&a[2].index===2&&a[0].groups.x===undefined&&r.lastIndex===0",
    );
}

#[test]
fn wide_branch_capture_layouts_survive_copy_and_collection_without_default_quotas() {
    let mut realm = Realm::default();
    realm
        .eval("let r=new RegExp('(b)|'.repeat(10000)+'(?<x>a)\\\\k<x>','d'),copy=new RegExp(r)")
        .unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("let a=copy.exec('aa');a[0]==='aa'&&a.length===10002&&a[1]===undefined&&a[10000]===undefined&&a.groups.x==='a'&&a.indices.groups.x===a.indices[10001]&&copy.source===r.source"),Ok(Value::Boolean(true)));
    realm.collect(usize::MAX).unwrap();
    assert_eq!(
        realm.eval("a.groups.x==='a'&&a.indices[10001][1]===1"),
        Ok(Value::Boolean(true))
    );
    assert_eq!(realm.eval("new RegExp('('.repeat(100000)+'b'+')'.repeat(100000)+'|(?<x>a)\\\\k<x>','d').exec('aa').groups.x==='a'"),Ok(Value::Boolean(true)));
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
fn nested_repeated_duplicate_name_and_unicode_reference_choices_remain_unsupported() {
    for source in [
        r"/(?:(?:(?:(a|b)(?:\1)+){2})|)*/.test('aa')",
        r"/(?:(?:(?:(a)(?:\1)+|(b)\2){2})|)*/.test('aa')",
        r"/(?:(?:(?:(?<x>a)\k<x>|(?<x>b)(?:\k<x>)+){2})|)*/.test('bb')",
        r"/(?<x>a)\k<x>|(?<y>b)\k<y>/u.test('bb')",
    ] {
        assert!(
            matches!(
                Realm::default().eval(source),
                Err(Error::Unsupported { .. })
            ),
            "{source}"
        );
    }
    check(r"/(a)\1|(b)\2/.exec('ab')===null");
}
