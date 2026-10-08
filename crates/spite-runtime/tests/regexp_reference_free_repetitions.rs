//! Repeated groups feed final captures to references outside their bodies.

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
fn reference_free_repetition_result_snapshot() {
    let mut rows = String::new();
    for (source, flags, text) in [
        (r"(a)+\1", "d", "aaaaa"),
        (r"(a)+?\1", "d", "aaaaa"),
        (r"([ab])*\1c", "d", "aacc"),
        (r"([ab])*?\1c", "d", "aacc"),
        (r"(a){1,3}\1", "d", "aaaaa"),
        (r"([ab]){1,3}?\1", "d", "aabb"),
        (r"(?:(a)(b)){1,3}\1\2", "d", "ababab"),
        (r"(?:(a)(b)){1,3}?\1\2", "d", "ababab"),
        (r"()(a)*\1\2", "d", "aaa"),
        (r"()(a)*?\1\2", "d", "aaa"),
        (r"\1(a)+", "d", "aaa"),
        (r"\1(a)+?", "d", "aaa"),
        (r"((a)(b)+)\1", "d", "abbabb"),
        (r"((a)(b)+?)\1", "d", "abbabb"),
        (r"(a)(b)*\1\2", "d", "abbbab"),
        (r"(a)(b)*?\1\2", "d", "abbbab"),
        (r"()?\1", "d", "q"),
        (r"()+\1", "d", "q"),
        (r"(){2,3}\1", "d", "q"),
        (r"([ab]){999999999999999999999999999999}\1", "d", "aaa"),
        (r"(?:(?<x>a)+|\k<x>(?<x>b)+)\k<x>", "d", "abbb"),
        (r"(?<all>(?<x>a)(?<last>b)+)\k<all>", "dgi", "abbabb"),
        (r"(?<x>)+\k<x>", "dgy", "q"),
        (r"(?<x>^)+\k<x>", "dm", "\nq"),
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
fn reference_free_retries_keep_final_named_captures_and_outside_aliases() {
    check(
        r"let a=/(?<x>a)+\k<x>/d.exec('aaaaa'),b=/(?<x>a)+?\k<x>/d.exec('aaaaa');a[0]==='aaaaa'&&a.groups.x==='a'&&a.indices.groups.x===a.indices[1]&&a.indices[1][0]===3&&a.indices[1][1]===4&&b[0]==='aa'&&b.indices.groups.x[0]===0",
    );
    check(
        r"let a=/(?<x>[ab])+\k<x>/d.exec('abba');a[0]==='abb'&&a.groups.x==='b'&&a.indices.groups.x===a.indices[1]&&a.indices[1][0]===1&&a.indices[1][1]===2",
    );
    check(
        r"let a=/(?:(?<x>a)(?<y>b)){1,3}\k<x>\k<y>/d.exec('ababab');a[0]==='ababab'&&a.groups.x==='a'&&a.groups.y==='b'&&a.indices.groups.x===a.indices[1]&&a.indices[1][0]===2&&a.indices.groups.y[0]===3",
    );
    check(
        r"let a=/(?<all>(?<x>a)(?<last>b)+)\k<all>/d.exec('abbabb');a[0]==='abbabb'&&a.groups.all==='abb'&&a.groups.last==='b'&&a.indices.groups.all===a.indices[1]&&a.indices[1][1]===3&&a.indices.groups.last===a.indices[3]&&a.indices[3][0]===2",
    );
    check(
        r"let a=/(?:(?<x>a)+|\k<x>(?<x>b)+)\k<x>/d.exec('abbb');a.index===1&&a[0]==='bbb'&&a[1]===undefined&&a.groups.x==='b'&&a.indices.groups.x===a.indices[2]&&a.indices[2][0]===2",
    );
    check(
        r"let a=/(?<x>.)+\k<x>/d.exec('\uD800\uD800');a[0].length===2&&a.groups.x.charCodeAt(0)===0xD800&&a.indices.groups.x===a.indices[1]&&a.indices[1][0]===0&&a.indices[1][1]===1",
    );
}

#[test]
fn reference_free_consumers_empty_groups_and_full_input_assertions_keep_semantics() {
    check(
        r"let r=/(?<x>[ab])+\k<x>/dg,a=[...'aabb abba'.matchAll(r)];a.length===2&&a[0][0]==='aabb'&&a[0].groups.x==='b'&&a[0].indices.groups.x[0]===2&&a[1][0]==='abb'&&a[1].index===5&&a[1].indices.groups.x[0]===6&&r.lastIndex===0",
    );
    check(
        r"'qabbaZ'.replace(/(?<x>[ab])+\k<x>/,'<$<x>>')==='q<b>aZ'&&'qabbaZ'.search(/([ab])+\1/)===1&&'qabbaZ'.split(/([ab])+\1/).join(',')==='q,b,aZ'",
    );
    check(
        r"let r=/(?<x>a)+\k<x>/dy;r.lastIndex=1;let copy=new RegExp(r),a=r.exec('qaaaaa');a[0]==='aaaaa'&&a.indices.groups.x[0]===4&&r.lastIndex===6&&r.exec('qaaaaa')===null&&r.lastIndex===0&&copy.source===r.source&&copy.lastIndex===0",
    );
    check(
        r"let a=[...'q'.matchAll(/(?<x>)?\k<x>/dg)],b=[...'q'.matchAll(/(?<x>)+\k<x>/dg)];a.length===2&&a[0].groups.x===undefined&&a[1].indices.groups.x===undefined&&b.length===2&&b[0].groups.x===''&&b[1].indices.groups.x[0]===1",
    );
    check(
        r"let r=/(?<x>^)+\k<x>/dmy;r.lastIndex=1;let a=r.exec('\nq');a.index===1&&a.groups.x===''&&a.indices.groups.x[0]===1&&r.lastIndex===1&&/(?<x>\b)+\k<x>/.exec(' ')===null&&/(?<x>\B)+\k<x>/.exec(' ').groups.x===''",
    );
}

#[test]
fn deep_reference_free_groups_and_huge_empty_counts_copy_and_collect_unlimited() {
    let mut realm = Realm::default();
    assert_eq!(realm.eval("let slash=String.fromCharCode(92),huge=new RegExp('(?<x>){'+'9'.repeat(10000)+'}'+slash+'k<x>','d'),empty=huge.exec('q');empty[0]===''&&empty.groups.x===''&&empty.indices.groups.x===empty.indices[1]&&empty.indices[1][0]===0"),Ok(Value::Boolean(true)));
    realm.eval("let r=new RegExp('(?<x>'+'('.repeat(99999)+'a'+')'.repeat(100000)+'+'+slash+'k<x>','d'),copy=new RegExp(r)").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("let a=copy.exec('aaa');a[0]==='aaa'&&a.length===100001&&a.groups.x==='a'&&a[100000]==='a'&&a.indices[100000][0]===1&&a.indices[100000][1]===2&&a.indices.groups.x===a.indices[1]&&copy.source===r.source"),Ok(Value::Boolean(true)));
    realm.collect(usize::MAX).unwrap();
}

#[test]
fn explicit_reference_free_work_aborts_before_last_index_and_handlers() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval(r"let marker=0,r=/(?<x>[ab])+\k<x>c/g;r.lastIndex=1;let text='a'.repeat(5000)")
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
fn repeated_choices_nested_quantifiers_atoms_and_lookaround_remain_unsupported() {
    for source in [
        r"/(a|b)+\1/.test('aabb')",
        r"/((a)+)+\1/.test('aaaa')",
        r"/(a+)\1/.test('aaaa')",
        r"/((?=a)a)+\1/.test('aaaa')",
        r"/((?i:a))+\1/.test('aaaa')",
        r"/(a)+\1/u.test('aaaa')",
    ] {
        assert!(
            matches!(
                Realm::default().eval(source),
                Err(Error::Unsupported { .. })
            ),
            "{source}"
        );
    }
    check(r"/(a)+\1/.exec('ab')===null");
}
