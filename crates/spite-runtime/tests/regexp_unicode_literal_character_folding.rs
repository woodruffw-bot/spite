//! Single Unicode literal atoms and simple/common folding (22.2.2.7.3).
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
fn unicode_literal_character_folding_results_snapshot() {
    let mut rows = String::new();
    let inputs = [
        JsString::from("😀ſKaA$^.\n"),
        JsString::from("\u{10400}\u{10428}ßẞ"),
        JsString::from_code_units(vec![0xd800, 0xa, 0xdc00]),
        JsString::from("İıiI"),
    ];
    for body in [
        "a",
        "s",
        "k",
        "ſ",
        "K",
        r"\$",
        r"\^",
        r"\.",
        r"\u{1f600}",
        r"\uD800",
        r"\u{10400}",
        "ß",
    ] {
        for (left, right) in [("", ""), ("((", "))"), ("(?<x>^", "$)")] {
            let pattern = JsString::from(format!("{left}{body}{right}").as_str());
            for flags in ["diug", "diumy", "divg", "divmy"] {
                for input in &inputs {
                    for start in [0, 1, 2, input.len() + 1] {
                        let script = format!(
                            "let r=new RegExp({pattern:?},'{flags}');r.lastIndex={start};let m=r.exec({input:?});JSON.stringify(m===null?{{match:null,lastIndex:r.lastIndex}}:{{matches:[...m],index:m.index,input:m.input,groups:m.groups,indices:m.indices,indexGroups:m.indices.groups,lastIndex:r.lastIndex,source:r.source}})"
                        );
                        let Value::String(result) = Realm::default().eval(&script).unwrap() else {
                            panic!("{script}")
                        };
                        writeln!(rows,"{pattern:?} flags={flags:?} input={input:?} lastIndex={start} {result:?}").unwrap();
                    }
                }
            }
        }
    }
    insta::assert_snapshot!(rows);
}

#[test]
fn unicode_literal_character_folding_unicode_18_widths_snapshot() {
    let mut rows = String::new();
    for body in ["ß", r"\u{1df95}"] {
        for mode in ["u", "v"] {
            let pattern = JsString::from(body);
            for input in [JsString::from("ß"), JsString::from("\u{1df95}")] {
                for start in [0, 1] {
                    let script = format!(
                        "let r=new RegExp({pattern:?},'di{mode}g');r.lastIndex={start};let m=r.exec({input:?});JSON.stringify(m===null?{{match:null,lastIndex:r.lastIndex}}:{{text:m[0],index:m.index,indices:m.indices[0],lastIndex:r.lastIndex}})"
                    );
                    let Value::String(result) = Realm::default().eval(&script).unwrap() else {
                        panic!("{script}")
                    };
                    writeln!(
                        rows,
                        "{pattern:?} mode={mode:?} input={input:?} start={start} {result:?}"
                    )
                    .unwrap();
                }
            }
        }
    }
    insta::assert_snapshot!(rows);
}

#[test]
fn unicode_literal_character_folding_escaped_anchors_and_fold_distinctions() {
    check(r"/a/iu.test('A')&&/s/iv.test('ſ')&&/k/iu.test('K')&&/ſ/iv.test('S')");
    check(r"/ß/iu.test('ẞ')&&!/ß/iv.test('ss')&&!/İ/iu.test('i')&&!/ı/iv.test('I')");
    check(r"/\u1fd3/iu.test('\u0390')&&/\ufb05/iv.test('\ufb06')");
    check(r"/^a$/iu.test('A')&&!/^a$/u.test('A')&&/^a$/ium.test('\rA\n')&&!/^a$/iu.test('A\n')");
    check(
        r"/\$/iu.exec('a$')[0]==='$'&&/^\$$/iv.test('$')&&/\^/iu.test('^')&&/\./iv.exec('😀.')[0]==='.'",
    );
    check(r"/\\$/iu.test('\\')&&/\u0024/iv.test('$')");
    check(r"/\uD800/iu.exec('😀')===null&&/\uD800/iv.test('\uD800')&&!/\uDE00/iu.test('😀')");
    check(r"/^\u{1f600}$/u.test('😀')&&/^a$/vm.test('\na\n')");
}

#[test]
fn unicode_literal_character_folding_captures_consumers_and_original_text() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let r=/(?<x>\u{{10400}})/di{mode}y;r.lastIndex=1;let m=r.exec('\u{{10428}}');m[0]==='\u{{10428}}'&&m[1]==='\u{{10428}}'&&m.groups.x===m[1]&&m.index===0&&m.indices[1][1]===2&&m.indices.groups.x===m.indices[1]&&r.lastIndex===2"
        ));
        check(&format!(
            r"let a=[...'😀ſS'.matchAll(/(s)/di{mode}g)];a.length===2&&a[0][1]==='ſ'&&a[0].indices[1][0]===2&&a[1][1]==='S'"
        ));
        check(&format!(
            r"'ſS'.replace(/(?<x>s)/i{mode}g,'<$1:$<x>>')==='<ſ:ſ><S:S>'"
        ));
        check(&format!(
            r"'😀ſS'.split(/(s)/i{mode}).join('|')==='😀|ſ||S|'"
        ));
        check(&format!(
            r"let r=/s/i{mode};r.lastIndex=8;'😀ſ'.search(r)===2&&r.lastIndex===8"
        ));
    }
}

#[test]
fn unicode_literal_character_folding_metadata_coercions_and_strict_writes() {
    check(
        r"let r=/^a$/dimv,c=new RegExp(r,'diuy');c.source==='^a$'&&c.ignoreCase&&c.unicode&&!c.multiline&&c.exec('A')[0]==='A'&&c.lastIndex===1",
    );
    check(
        r"let t='',r=/s/iuy;r.lastIndex={valueOf(){t+='i';return 0}};let m=r.exec({toString(){t+='s';return 'ſ'}});t==='si'&&m[0]==='ſ'&&r.lastIndex===1",
    );
    check(
        r"let r=/s/iuy;Object.defineProperty(r,'lastIndex',{writable:false});let caught=false;try{r.exec('ſ')}catch(e){caught=e instanceof TypeError}caught&&r.lastIndex===0",
    );
    check(
        r"let r=/s/ivy;r.lastIndex=1;Object.defineProperty(r,'lastIndex',{writable:false});let caught=false;try{r.exec('😀')}catch(e){caught=e instanceof TypeError}caught&&r.lastIndex===1",
    );
}

#[test]
fn unicode_literal_character_folding_large_escapes_collection_and_defaults() {
    assert_eq!(Limits::default().max_steps, None);
    assert_eq!(Limits::default().max_heap_entries, None);
    let mut realm = Realm::default();
    realm.eval(r"let p='\\u{'+'0'.repeat(100000)+'61}',r=new RegExp(p,'ivg'),c=new RegExp(r),text='😀'.repeat(100000)+'A'").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(
        realm.eval("let m=c.exec(text);m[0]==='A'&&m.index===200000&&c.lastIndex===200001"),
        Ok(Value::Boolean(true))
    );
    realm.eval("p=null;r=null;c=null;text=null;m=null").unwrap();
    realm.collect(usize::MAX).unwrap();
}

#[test]
fn unicode_literal_character_folding_opted_work_and_remaining_gaps() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval("let r=/s/iug,marker=0,text='😀'.repeat(10000);r.lastIndex=1")
        .unwrap();
    assert!(matches!(
        realm.eval("try{r.exec(text)}catch{marker=1}finally{marker=2}"),
        Err(Error::Limit { .. })
    ));
    assert_eq!(
        realm.eval("marker===0&&r.lastIndex===1"),
        Ok(Value::Boolean(true))
    );
    for source in [
        r"/(?:ab)+/iu.exec('AB')",
        r"/a+/iv.exec('A')",
        r"/(?:(a)b)+/iu.exec('AB')",
        r"/a|b/iv.exec('A')",
        r"/^(a)+$/iu.exec('A')",
        r"/(?=a)/iv.exec('A')",
        r"/\u{D800}\u{DC00}/iu.exec('😀')",
    ] {
        assert!(
            matches!(
                Realm::default().eval(source),
                Err(Error::Unsupported { .. })
            ),
            "{source}"
        );
    }
}

#[test]
fn original_unicode_literal_character_folding_complete_programs() {
    let mut rows = String::new();
    for source in [
        r"/^a$/u.test('a')",
        r"/^a$/v.test('a')",
        r"/(^a)/u.test('a')",
        r"/a/iu.exec('a')",
        r"/^a/u.exec('a')",
        r"/\u{61}/iu.exec('a')",
        r"/^a$/u.exec('a')",
        r"/\u{1f600}/iu.exec('😀')",
        r"/a/iv.exec('a')",
        r"/^a/v.exec('a')",
        r"/\uD800/iu.exec('\uD800')",
    ] {
        let program = format!("JSON.stringify(eval({:?}))", JsString::from(source));
        let Value::String(result) = Realm::default().eval(&program).unwrap() else {
            panic!("{program}")
        };
        writeln!(rows, "{:?} {result:?}", JsString::from(source)).unwrap();
    }
    insta::assert_snapshot!(rows);
}
