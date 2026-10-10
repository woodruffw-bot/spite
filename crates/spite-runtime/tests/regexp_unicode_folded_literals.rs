//! Unicode folded literal concatenations with original capture bounds.
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
fn unicode_folded_literal_results_snapshot() {
    let mut rows = String::new();
    let inputs = [
        JsString::from("aaabſK😀ßS"),
        JsString::from("\u{10428}\u{10400}"),
        JsString::from_code_units(vec![0xd800, 0x41, 0xdc00, 0x61]),
        JsString::from(""),
    ];
    for body in [
        "(a)A(b)",
        "(())a(a)",
        "(ß)(s)()",
        r"(\u{10400})(\u{10428})",
        r"(\uD800)(a)",
        "((ſ)(K))",
        "(a)(A)",
        "(a)a(a)",
        "(a)()",
        "ab",
        "(?<x>a)(?<y>A)(?<z>b)",
    ] {
        let pattern = JsString::from(body);
        for flags in ["diug", "diumy", "divg", "divmy"] {
            for input in &inputs {
                for start in [0, 1, 2, input.len(), input.len() + 1] {
                    let script = format!(
                        "let r=new RegExp({pattern:?},'{flags}');r.lastIndex={start};let m=r.exec({input:?});JSON.stringify(m===null?{{match:null,lastIndex:r.lastIndex}}:{{matches:[...m],index:m.index,input:m.input,groups:m.groups,indices:m.indices,indexGroups:m.indices.groups,lastIndex:r.lastIndex,source:r.source}})"
                    );
                    let Value::String(result) = Realm::default().eval(&script).unwrap() else {
                        panic!("{script}")
                    };
                    writeln!(
                        rows,
                        "{pattern:?} flags={flags:?} input={input:?} lastIndex={start} {result:?}"
                    )
                    .unwrap();
                }
            }
        }
    }
    insta::assert_snapshot!(rows);
}

#[test]
fn unicode_folded_literal_unicode_18_capture_widths_snapshot() {
    let mut rows = String::new();
    for body in ["(?<x>ß)(s)()", r"(?<x>\u{1df95})(s)()", "((ß)())s"] {
        for mode in ["u", "v"] {
            let pattern = JsString::from(body);
            for input in [JsString::from("ßS"), JsString::from("\u{1df95}ſ")] {
                for start in [0, 1] {
                    let script = format!(
                        "let r=new RegExp({pattern:?},'di{mode}g');r.lastIndex={start};let m=r.exec({input:?});JSON.stringify(m===null?{{match:null,lastIndex:r.lastIndex}}:{{matches:[...m],index:m.index,groups:m.groups,indices:m.indices,indexGroups:m.indices.groups,lastIndex:r.lastIndex}})"
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
fn unicode_folded_literal_prefix_retries_empty_captures_and_fold_distinctions() {
    check(
        r"let m=/(a)A(b)/diu.exec('aaab');m[0]==='aab'&&m.index===1&&m[1]==='a'&&m[2]==='b'&&m.indices[1][0]===1&&m.indices[2][0]===3",
    );
    check(
        r"let m=/(a())(A)/div.exec('aA');m.length===4&&m[1]==='a'&&m[2]===''&&m[3]==='A'&&m.indices[2][0]===1&&m.indices[2][1]===1",
    );
    check(
        r"let m=/(())a(A)/diu.exec('aA');m[1]===''&&m[2]===''&&m[3]==='A'&&m.indices[1][0]===0&&m.indices[2][1]===0",
    );
    check(r"let r=/(a)A(b)/iuy;r.lastIndex=0;r.exec('aaab')===null&&r.lastIndex===0");
    check(
        r"let m=/(?<x>ſ)(?<y>K)/div.exec('SK');m.groups.x==='S'&&m.groups.y==='K'&&m.indices.groups.x===m.indices[1]&&m.indices.groups.y===m.indices[2]",
    );
    check(
        r"/(ß)s/iu.test('ẞS')&&!/(ß)s/iv.test('sss')&&!/(İ)i/iu.test('ii')&&!/(ı)i/iv.test('Ii')",
    );
    check(r"/(\u1fd3)(\ufb05)/iu.test('\u0390\ufb06')");
    check(
        r"let m=/(\uD800)(a)/div.exec('\uD800A');m[1]==='\uD800'&&m[2]==='A'&&m.indices[2][0]===1&&!/(\uD800)(a)/iu.test('\uD800\uDC00a')",
    );
}

#[test]
fn unicode_folded_literal_consumers_named_groups_and_original_text() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let r=/(?<x>\u{{10400}})(?<y>s)()/di{mode}y;r.lastIndex=1;let m=r.exec('\u{{10428}}ſ');m[1]==='\u{{10428}}'&&m[2]==='ſ'&&m[3]===''&&m.index===0&&m.indices[1][1]===2&&m.indices[2][0]===2&&m.indices[3][0]===3&&m.indices.groups.x===m.indices[1]&&r.lastIndex===3"
        ));
        check(&format!(
            r"let a=[...'ſKSK'.matchAll(/(?<x>s)(k)/di{mode}g)];a.length===2&&a[0][1]==='ſ'&&a[0][2]==='K'&&a[1][1]==='S'&&a[1].indices[2][1]===4"
        ));
        check(&format!(
            r"'ſKSK'.replace(/(?<x>s)(k)/i{mode}g,'<$1:$2:$<x>>')==='<ſ:K:ſ><S:K:S>'"
        ));
        check(&format!(
            r"'ſKSK'.split(/(s)(k)/i{mode}).join('|')==='|ſ|K||S|K|'"
        ));
        check(&format!(
            r"let r=/(s)(k)/i{mode};r.lastIndex=8;'😀ſK'.search(r)===2&&r.lastIndex===8"
        ));
    }
}

#[test]
fn unicode_folded_literal_metadata_coercions_and_strict_writes() {
    check(
        r"let r=/(?<x>s)(k)/dimv,c=new RegExp(r,'diuy');c.source==='(?<x>s)(k)'&&c.ignoreCase&&c.unicode&&!c.multiline&&c.exec('ſK').groups.x==='ſ'&&c.lastIndex===2",
    );
    check(
        r"let t='',r=/(s)(k)/iuy;r.lastIndex={valueOf(){t+='i';return 0}};let m=r.exec({toString(){t+='s';return 'ſK'}});t==='si'&&m[1]==='ſ'&&m[2]==='K'&&r.lastIndex===2",
    );
    check(
        r"let r=/(s)(k)/iuy;Object.defineProperty(r,'lastIndex',{writable:false});let caught=false;try{r.exec('ſK')}catch(e){caught=e instanceof TypeError}caught&&r.lastIndex===0",
    );
    check(
        r"let r=/(s)(k)/ivy;r.lastIndex=1;Object.defineProperty(r,'lastIndex',{writable:false});let caught=false;try{r.exec('😀')}catch(e){caught=e instanceof TypeError}caught&&r.lastIndex===1",
    );
}

#[test]
fn unicode_folded_literal_many_captures_long_prefixes_collection_and_defaults() {
    assert_eq!(Limits::default().max_steps, None);
    assert_eq!(Limits::default().max_heap_entries, None);
    let mut realm = Realm::default();
    realm.eval("let p='('.repeat(3000)+'aA'+')'.repeat(3000),r=new RegExp(p,'divg'),c=new RegExp(r),text='😀'.repeat(100000)+'Aa'").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("let m=c.exec(text);m.length===3001&&m[1]==='Aa'&&m[3000]==='Aa'&&m.indices[3000][0]===200000&&m.indices[3000][1]===200002&&c.lastIndex===200002"),Ok(Value::Boolean(true)));
    realm.eval("p=null;r=null;c=null;text=null;m=null").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("let n=5000,longPattern='a'.repeat(n)+'B',longRegex=new RegExp(longPattern,'iu'),longText='A'.repeat(n+5000)+'b';longRegex.exec(longText).index===5000"),Ok(Value::Boolean(true)));
}

#[test]
fn unicode_folded_literal_opted_work_and_remaining_gaps() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval("let r=/(s)(k)/iug,marker=0,text='😀'.repeat(10000);r.lastIndex=1")
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
        r"/ab+/iu.exec('AB')",
        r"/(ab)+/iv.exec('AB')",
        r"/a|b/iu.exec('A')",
        r"/^(ab)+$/iv.exec('AB')",
        r"/[a]b/iu.exec('AB')",
        r"/(a)\1/iv.exec('AA')",
        r"/(?=ab)/iu.exec('AB')",
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
fn original_unicode_folded_literal_complete_programs() {
    let mut rows = String::new();
    for source in [r"/ab/iu.exec('AB')", r"/(a)b/iu.exec('AB')"] {
        let program = format!("JSON.stringify(eval({:?}))", JsString::from(source));
        let Value::String(result) = Realm::default().eval(&program).unwrap() else {
            panic!("{program}")
        };
        writeln!(rows, "{:?} {result:?}", JsString::from(source)).unwrap();
    }
    insta::assert_snapshot!(rows);
}
