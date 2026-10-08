//! Quantified characters feed complete capture prefixes to later references.

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
fn quantified_reference_character_result_snapshot() {
    let mut rows = String::new();
    for (source, flags, text) in [
        (r"(a+)\1", "d", "aaaaa"),
        (r"(a+?)\1", "d", "aaaaa"),
        (r"([ab]*)\1c", "d", "aacc"),
        (r"([ab]*?)\1c", "d", "aacc"),
        (r"(a{1,3})\1", "d", "aaaaa"),
        (r"([ab]{1,3}?)\1", "d", "aabb"),
        (r"(?:a+)(b+)\1", "d", "aaabbbb"),
        (r"(?:a+?)(b+?)\1", "d", "aaabbbb"),
        (r"()(a*)\1\2", "d", "aaa"),
        (r"()(a*?)\1\2", "d", "aaa"),
        (r"\1(a+)", "d", "aaa"),
        (r"\1(a+?)", "d", "aaa"),
        (r"((a+)(b+))\1", "d", "aabbaabb"),
        (r"((a+?)(b+?))\1", "d", "aabbaabb"),
        (r"(a)(b*)\1\2", "d", "abbbabbb"),
        (r"(a)(b*?)\1\2", "d", "abbbabbb"),
        (r"(a?)\1", "d", "q"),
        (r"([ab]?)\1", "d", "q"),
        (r"(?:(a*)b(c*))\1\2", "d", "aabccaacc"),
        (r"([ab]{999999999999999999999999999999})\1", "d", "aaa"),
        (r"(?:(?<x>a+)|\k<x>(?<x>b+))\k<x>", "d", "abbbb"),
        (r"(?<all>(?<x>a+)(?<y>b+))\k<all>", "dgi", "aabbaabb"),
        (r"(?<x>[ab]*)\k<x>", "dgy", "q"),
        (r"(?<x>.+)\k<x>", "ds", "\n\n"),
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
fn quantified_units_restore_whole_prefixes_named_aliases_and_parent_closes() {
    check(
        r"let a=/(?<x>a+)\k<x>/d.exec('aaaaa'),b=/(?<x>a+?)\k<x>/d.exec('aaaaa');a[0]==='aaaa'&&a.groups.x==='aa'&&a.indices.groups.x===a.indices[1]&&a.indices[1][0]===0&&a.indices[1][1]===2&&b[0]==='aa'&&b.groups.x==='a'&&b.indices.groups.x[1]===1",
    );
    check(
        r"let a=/(?<x>[ab]*)\k<x>c/d.exec('aacc'),b=/(?<x>[ab]*)\k<x>c/d.exec('c');a[0]==='aac'&&a.groups.x==='a'&&a.indices.groups.x[1]===1&&b[0]==='c'&&b.groups.x===''&&b.indices.groups.x===b.indices[1]&&b.indices.groups.x[0]===0&&b.indices.groups.x[1]===0",
    );
    check(
        r"let a=/(?<all>(?<x>a+)(?<y>b+))\k<all>/d.exec('aabbaabb');a[0]==='aabbaabb'&&a.groups.all==='aabb'&&a.groups.x==='aa'&&a.groups.y==='bb'&&a.indices.groups.all===a.indices[1]&&a.indices.groups.all[1]===4&&a.indices.groups.x===a.indices[2]&&a.indices.groups.y===a.indices[3]&&a.indices.groups.y[0]===2",
    );
    check(
        r"let a=/(?:(?<x>a+)|\k<x>(?<x>b+))\k<x>/d.exec('abbbb');a.index===1&&a[0]==='bbbb'&&a[1]===undefined&&a.groups.x==='bb'&&a.indices.groups.x===a.indices[2]&&a.indices.groups.x[0]===1&&a.indices.groups.x[1]===3",
    );
    check(
        r"let a=/(?<x>.+)\k<x>/d.exec('\uD800\uD800\uD800\uD800');a[0].length===4&&a.groups.x.length===2&&a.groups.x.charCodeAt(0)===0xD800&&a.indices.groups.x===a.indices[1]&&a.indices.groups.x[1]===2",
    );
    check(
        r"let a=/(?<x>.+)\k<x>/ds.exec('\n\n'),b=/\1(a+)/d.exec('aaa');a[0]==='\n\n'&&a.groups.x==='\n'&&a.indices.groups.x[1]===1&&b[0]==='aaa'&&b[1]==='aaa'&&b.indices[1][1]===3",
    );
}

#[test]
fn quantified_prefix_consumers_empty_captures_and_sticky_retries_keep_semantics() {
    check(
        r"let r=/(?<x>a+)\k<x>/dg,a=[...'aaaaa aa'.matchAll(r)];a.length===2&&a[0][0]==='aaaa'&&a[0].groups.x==='aa'&&a[0].indices.groups.x[1]===2&&a[1][0]==='aa'&&a[1].index===6&&a[1].indices.groups.x[0]===6&&r.lastIndex===0",
    );
    check(
        r"'qaaaaaZ'.replace(/(?<x>a+)\k<x>/,'<$<x>>')==='q<aa>aZ'&&'qaaaaaZ'.search(/(a+)\1/)===1&&'qaaaaaZ'.split(/(a+)\1/).join(',')==='q,aa,aZ'",
    );
    check(
        r"let r=/(?<x>a+)\k<x>/dy;r.lastIndex=1;let copy=new RegExp(r),a=r.exec('qaaaaa');a[0]==='aaaa'&&a.groups.x==='aa'&&a.indices.groups.x[0]===1&&a.indices.groups.x[1]===3&&r.lastIndex===5&&r.exec('qaaaaa')===null&&r.lastIndex===0&&copy.source===r.source&&copy.lastIndex===0",
    );
    check(
        r"let a=[...'q'.matchAll(/(?<x>a?)\k<x>/dg)];a.length===2&&a[0].groups.x===''&&a[1].groups.x===''&&a[1].indices.groups.x[0]===1&&a[1].indices.groups.x[1]===1",
    );
    check(
        r"let a=/(?:a+)(b+)\1/d.exec('aaabbbb'),b=/(?:a+?)(b+?)\1/d.exec('aaabbbb');a[0]==='aaabbbb'&&a[1]==='bb'&&a.indices[1][0]===3&&a.indices[1][1]===5&&b[0]==='aaabb'&&b[1]==='b'&&b.indices[1][1]===4",
    );
}

#[test]
fn deep_quantified_prefixes_and_huge_bounds_copy_and_collect_unlimited() {
    let mut realm = Realm::default();
    assert_eq!(realm.eval("let slash=String.fromCharCode(92),n='9'.repeat(10000),huge=new RegExp('(?<x>a{'+n+'})'+slash+'k<x>','d'),zero=new RegExp('(?<x>a{0,'+n+'})'+slash+'k<x>','d'),empty=zero.exec('q');huge.exec('aaa')===null&&empty[0]===''&&empty.groups.x===''&&empty.indices.groups.x===empty.indices[1]&&empty.indices.groups.x[0]===0&&empty.indices.groups.x[1]===0"),Ok(Value::Boolean(true)));
    realm.eval("let r=new RegExp('(?<x>'+'('.repeat(99999)+'a+'+')'.repeat(100000)+slash+'k<x>','d'),copy=new RegExp(r)").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("let a=copy.exec('aaaaa');a[0]==='aaaa'&&a.length===100001&&a.groups.x==='aa'&&a[100000]==='aa'&&a.indices[100000][0]===0&&a.indices[100000][1]===2&&a.indices.groups.x===a.indices[1]&&copy.source===r.source"),Ok(Value::Boolean(true)));
    realm.collect(usize::MAX).unwrap();
}

#[test]
fn explicit_quantified_prefix_work_aborts_before_last_index_and_handlers() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval(r"let marker=0,r=/(?<x>a+)\k<x>c/g;r.lastIndex=1;let text='a'.repeat(5000)")
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
fn nested_quantifiers_repeated_choices_and_lookaround_remain_unsupported() {
    for source in [
        r"/(?:(?:(a|b)+\1)|)*/.test('aabb')",
        r"/(?:(?:((a)+)+\1)|)*/.test('aaaa')",
        r"/(?:(?:(?:(a+)\1){2})|)*/.test('aaaaaaaa')",
        r"/((?<=a)a)+\1/.test('aaaa')",
        r"/((?i:a))+\1/.test('aaaa')",
        r"/(a+)\1/u.test('aaaa')",
    ] {
        assert!(
            matches!(
                Realm::default().eval(source),
                Err(Error::Unsupported { .. })
            ),
            "{source}"
        );
    }
    check(r"/(a+)\1/.exec('ab')===null");
}
