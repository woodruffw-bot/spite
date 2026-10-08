//! Exact-count ordinary character sequences in fixed lookbehind (22.2.2.8).
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
fn fixed_lookbehind_sequence_result_snapshot() {
    let mut rows = String::new();
    for (source, flags, text) in [
        (r"(?<=(?:ab){2})c", "d", "ababc"),
        (r"(?<!(?:ab){2})c", "d", "ababc"),
        (r"(?<!(?:ab){2})c", "d", "abc"),
        (r"(?<=(?:ab){0})c", "d", "c"),
        (r"(?<!(?:ab){0})c", "d", "c"),
        (r"(?<=(?:ab){2,2})c", "d", "ababc"),
        (r"(?<=(?:ab){2}?)c", "d", "ababc"),
        (r"(?<=((?:ab){2}))c", "d", "ababc"),
        (r"(?<=((?:ab){2}))\1", "d", "abababab"),
        (r"(?<!((?:ab){2})q)c", "d", "ababrc"),
        (r"(?<=((?:[a-c]\d){2}))c", "d", "a1a2c"),
        (r"(?<=(?:a.){2})b", "ds", "a\na\nb"),
        (r"(?<=(?:a.){2})b", "d", "a\na\nb"),
        (r"(?<=(?:µΜ){2})a", "di", "ΜµµΜa"),
        (r"(?<=^(?:ab){2})c", "dm", "q\nababc"),
        (r"(?<=(?:ab){2}\b)c", "d", "ababc"),
        (r"(?<=(?:ab){2}\B)c", "d", "ababc"),
        (r"(?<=a(?:ba){2})b", "d", "ababab"),
        (r"(?<=(?:ab){2}(?<=ab))c", "d", "ababc"),
        (r"(?<=(?:ab){2}(?<!ba))c", "d", "ababc"),
        (r"(?<=((?:ab){2})(?<!(ba)))c", "d", "ababc"),
        (r"(?:(?<=(?:ab){2})c|d)+e", "d", "ababcdde"),
        (r"((?<=(?:ab){2})){2}c\1", "d", "ababc"),
        (r"((?<=(?:ab){2}))*c\1", "d", "ababc"),
        (r"(?=(a(?:ba){2}(?<=((?:ba){2}))))\1\2", "d", "ababababa"),
        (r"(?<=(?:a(?:b)){2})c", "d", "ababc"),
        (r"(?<=((?:ab){0}))c", "d", "c"),
        (r"(?<=(?:[\uD800]b){2})c", "d", "lone surrogates"),
    ] {
        let source = JsString::from(source);
        let flags = JsString::from(flags);
        let input = if text == "lone surrogates" {
            JsString::from_code_units(vec![0xd800, 0x62, 0xd800, 0x62, 0x63])
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
    insta::assert_snapshot!(rows);
}

#[test]
fn sequence_lookbehind_captures_negative_restore_and_outer_retry_preserve_ranges() {
    check(
        r"let a=/(?<=(?<x>(?:ab){2}))\k<x>/d.exec('abababab');a[0]==='abab'&&a.index===4&&a.groups.x==='abab'&&a.indices.groups.x===a.indices[1]&&a.indices[1][0]===0&&a.indices[1][1]===4",
    );
    check(
        r"let a=/(?<!((?:ab){2})q)c/d.exec('ababrc');a.index===5&&a[1]===undefined&&a.indices[1]===undefined",
    );
    check(
        r"let a=/(?<=((?:ab){2})(?<!(?<y>ba)))c/d.exec('ababc');a[1]==='abab'&&a.groups.y===undefined&&a.indices[1][0]===0&&a.indices[1][1]===4&&a.indices.groups.y===undefined",
    );
    check(
        r"let a=/(?:(?<=(?<x>(?:ab){2}))q|(?<=(?<x>(?:ab){2}))c)\k<x>/d.exec('ababcabab');a[0]==='cabab'&&a.index===4&&a[1]===undefined&&a[2]==='abab'&&a.indices[1]===undefined&&a.indices.groups.x===a.indices[2]",
    );
    check(
        r"let a=/((?<=(?:ab){2})){2}c\1/d.exec('ababc'),b=/((?<=(?:ab){2}))*c\1/d.exec('ababc');a[1]===''&&a.indices[1][0]===4&&a.indices[1][1]===4&&b[1]===undefined&&b.indices[1]===undefined",
    );
    check(
        r"let a=/(?<=((?:ab){0}))c/d.exec('c');a.index===0&&a[1]===''&&a.indices[1][0]===0&&a.indices[1][1]===0",
    );
}

#[test]
fn sequence_lookbehind_consumers_global_sticky_and_callbacks_keep_input_positions() {
    check(
        r"let r=/(?<=(?<x>(?:ab){2}))c/dg,a=[...'ababcqababc'.matchAll(r)];a.length===2&&a[0].index===4&&a[1].index===10&&a[1].groups.x==='abab'&&a[1].indices.groups.x[0]===6&&r.lastIndex===0",
    );
    check(
        r"let r=/(?<=(?:ab){2})c/dy;r.lastIndex=4;let a=r.exec('ababc');a.index===4&&r.lastIndex===5&&r.exec('ababc')===null&&r.lastIndex===0",
    );
    check(
        r"let r=/(?<=(?:ab){2})/dg,a=[...'ababab'.matchAll(r)];a.length===2&&a[0].index===4&&a[1].index===6&&r.lastIndex===0",
    );
    check(
        r"let seen=[];let s='ababcqababc'.replace(/(?<=(?<x>(?:ab){2}))c/g,(m,c,i,s,g)=>{seen.push(m,c,i,s,g.x);return '_'});s==='abab_qabab_'&&seen.length===10&&seen[0]==='c'&&seen[1]==='abab'&&seen[2]===4&&seen[4]==='abab'&&seen[7]===10",
    );
    check(
        r"'ababcqababc'.replace(/(?<=(?<x>(?:ab){2}))c/g,'<$<x>>')==='abab<abab>qabab<abab>'&&'ababcqababc'.search(/(?<=(?:ab){2})c/)===4&&'ababcqababc'.split(/(?<=(?:ab){2})c/).join('|')==='abab|qabab|'",
    );
}

#[test]
fn sequence_lookbehind_deep_wrappers_large_counts_copies_and_gc_are_unlimited() {
    let mut realm = Realm::default();
    realm.eval("let r=new RegExp('(?<='+'(?:'.repeat(100000)+'(?:ab){2}'+')'.repeat(100000)+')(?<x>c)','d'),copy=new RegExp(r)").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("let a=copy.exec('ababc');a.index===4&&a.groups.x==='c'&&a.indices.groups.x===a.indices[1]&&copy.source===r.source"),Ok(Value::Boolean(true)));
    assert_eq!(realm.eval("let long=/(?<=((?:ab){10000}))c/dy;long.lastIndex=20000;let last=long.exec('ab'.repeat(10000)+'c');last.index===20000&&last[1].length===20000&&last.indices[1][0]===0&&last.indices[1][1]===20000&&long.lastIndex===20001"),Ok(Value::Boolean(true)));
    assert_eq!(realm.eval("let many=/(?:(?<=(?:ab){2})c|d)+e/.exec('ababc'+'d'.repeat(10000)+'e');many.index===4&&many[0].length===10002"),Ok(Value::Boolean(true)));
    let maximum = usize::MAX / 2;
    assert_eq!(realm.eval(&format!("new RegExp('(?<=(?:ab){{{maximum}}})c').exec('abc')===null&&new RegExp('(?<!(?:ab){{{maximum}}})c').exec('abc').index===2")),Ok(Value::Boolean(true)));
    realm.collect(usize::MAX).unwrap();
}

#[test]
fn explicit_sequence_lookbehind_work_aborts_before_last_index_and_handlers() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval("let marker=0,r=/(?<=(?:ab){8})c/g;r.lastIndex=1;let text='ab'.repeat(5000)")
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
fn variable_repeated_capture_assertion_reference_and_nested_counts_remain_unsupported() {
    for source in [
        r"/(?<=(?:ab){1,2})c/.exec('ababc')",
        r"/(?<=(ab){1,2})c/.exec('ababc')",
        r"/(?<=(?:a(?=a)){2})c/.exec('ababc')",
        r"/(?<=(?:a(?<=a)){2})c/.exec('ababc')",
        r"/(a)(?<=(?:\1b){1,2})c/.exec('ababc')",
        r"/(?<=(?:(?:ab){2}){2})c/.exec('ababababc')",
        r"/(?<=(?:ab){2})c/u.exec('ababc')",
        r"/(?<=(?:ab){2})c/v.exec('ababc')",
    ] {
        assert!(
            matches!(
                Realm::default().eval(source),
                Err(Error::Unsupported { .. })
            ),
            "{source}"
        );
    }
    let source = format!(
        "new RegExp('(?<=(?:ab){{{}}})c').exec('ababc')",
        usize::MAX / 2 + 1
    );
    assert!(matches!(
        Realm::default().eval(&source),
        Err(Error::Unsupported { .. })
    ));
}
