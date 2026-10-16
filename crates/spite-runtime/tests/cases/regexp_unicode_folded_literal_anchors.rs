//! Boundary assertions around Unicode folded literal concatenations.
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
fn unicode_folded_literal_boundary_results_snapshot() {
    let mut rows = String::new();
    for body in [
        "^(a)A(b)$",
        "((^a(a)$))",
        "(aA$)",
        "^a(a)",
        r"^(\u{10400})(\u{10428})$",
        r"^(\$)(a)\$$",
        "^(())a(a)$",
        "^(?<x>a)(?<y>A)$",
        "^a(a)$",
        "a(a)$",
    ] {
        let pattern = JsString::from(body);
        for flags in ["diug", "diumg", "divy", "divmy"] {
            for input in [
                "aaab\nAAB\r\nAA\u{2028}aa\u{2029}",
                "\u{10428}\u{10400}\n\u{10400}\u{10428}",
                "$A$\n$A$",
                "AA\n",
                "",
            ] {
                let input = JsString::from(input);
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
fn unicode_folded_literal_boundary_context_overlap_and_original_captures() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let r=/^((a)A)$/dim{mode}g,m=r.exec('aaa\r\nAA\u2028aa\u2029');m.index===5&&m[1]==='AA'&&m[2]==='A'&&m.indices[1][0]===5&&m.indices[2][1]===6&&r.lastIndex===7"
        ));
        check(&format!(
            r"let r=/a(a)$/di{mode}g,m=r.exec('AAAA');m.index===2&&m[1]==='A'&&m.indices[1][0]===3&&r.lastIndex===4"
        ));
        check(&format!(
            r"/^a(a)$/im{mode}.test('xAA\nAA')&&!/^a(a)$/i{mode}.test('AA\n')"
        ));
        check(&format!(
            r"let r=/^(?<x>\u{{10400}})(?<y>\u{{10428}})$/di{mode}y;r.lastIndex=1;let m=r.exec('\u{{10428}}\u{{10400}}');m.index===0&&m[1]==='\u{{10428}}'&&m[2]==='\u{{10400}}'&&m.indices.groups.x===m.indices[1]&&m.indices[1][1]===2&&r.lastIndex===4"
        ));
        check(&format!(
            r"let r=/^(?<__proto__>a)(())a$/dim{mode}g,m=r.exec('x\nAA');m.groups.__proto__==='A'&&Object.getPrototypeOf(m.groups)===null&&m[2]===''&&m[3]===''&&m.indices[2][0]===3&&m.indices.groups.__proto__===m.indices[1]"
        ));
        check(&format!(
            r"let r=/^(\uD800)(a)$/di{mode};let m=r.exec('\uD800A');m[1]==='\uD800'&&m.indices[1][1]===1&&!r.test('\uD800\uDC00A')"
        ));
    }
}

#[test]
fn unicode_folded_literal_boundary_escaped_anchors_and_consumers() {
    for mode in ["u", "v"] {
        check(&format!(
            r"/^(\$)(a)\$$/i{mode}.exec('$A$')[2]==='A'&&/^(a)\\$/i{mode}.exec('A\\')[1]==='A'"
        ));
        check(&format!(
            r"let a=[...'AA\r\nAa\u2028aA\u2029'.matchAll(/^(a)(a)$/dim{mode}g)];a.length===3&&a[0].index===0&&a[1].index===4&&a[2].index===7&&a[2].indices[2][1]===9"
        ));
        check(&format!(
            r"'AA\nAa'.replace(/^(?<x>a)(a)$/im{mode}g,'<$1:$2:$<x>>')==='<A:A:A>\n<A:a:A>'"
        ));
        check(&format!(
            r"let r=/^a(a)$/im{mode};r.lastIndex=99;'x\nAA'.search(r)===2&&r.lastIndex===99"
        ));
        check(&format!(
            r"'AA\nAa'.split(/^(a)(a)$/im{mode}).join('|')==='|A|A|\n|A|a|'"
        ));
    }
}

#[test]
fn unicode_folded_literal_boundary_metadata_clones_coercions_and_writes() {
    check(
        r"let r=/^(?<x>a)(a)$/dimv,c=new RegExp(r,'diuy');c.source==='^(?<x>a)(a)$'&&c.ignoreCase&&c.unicode&&!c.multiline&&c.exec('AA').groups.x==='A'&&c.lastIndex===2",
    );
    check(
        r"let t='',r=/^a(a)$/iuy;r.lastIndex={valueOf(){t+='i';return 0}};let m=r.exec({toString(){t+='s';return 'AA'}});t==='si'&&m[1]==='A'&&r.lastIndex===2",
    );
    check(
        r"let r=/^a(a)$/iuy;Object.defineProperty(r,'lastIndex',{writable:false});let caught=false;try{r.exec('AA')}catch(e){caught=e instanceof TypeError}caught&&r.lastIndex===0",
    );
    check(
        r"let r=/^a(a)$/ivy;r.lastIndex=1;Object.defineProperty(r,'lastIndex',{writable:false});let caught=false;try{r.exec('xAA')}catch(e){caught=e instanceof TypeError}caught&&r.lastIndex===1",
    );
}

#[test]
fn unicode_folded_literal_boundary_collection_and_unlimited_defaults() {
    assert_eq!(Limits::default().max_steps, None);
    assert_eq!(Limits::default().max_heap_entries, None);
    let mut realm = Realm::default();
    realm.eval("let p='('.repeat(3000)+'^aA$'+')'.repeat(3000),r=new RegExp(p,'dimvg'),c=new RegExp(r),text='😀'.repeat(100000)+'\\nAa'").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("let m=c.exec(text);m.length===3001&&m[1]==='Aa'&&m[3000]==='Aa'&&m.indices[3000][0]===200001&&m.indices[3000][1]===200003&&c.lastIndex===200003"),Ok(Value::Boolean(true)));
    realm.eval("p=null;r=null;c=null;text=null;m=null").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("let n=5000,longPattern='^'+'a'.repeat(n)+'$',longRegex=new RegExp(longPattern,'imu'),longText='A'.repeat(100000)+'b';longRegex.exec(longText)===null"),Ok(Value::Boolean(true)));
}

#[test]
fn unicode_folded_literal_boundary_opted_work_and_remaining_gaps() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval("let r=/^a(a)$/imug,marker=0,text='😀'.repeat(10000);r.lastIndex=1")
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
        r"/^ab+$/iu.exec('AB')",
        r"/^(ab)+$/iv.exec('AB')",
        r"/^a|b$/iu.exec('A')",
        r"/^(a$)b/iv.exec('AB')",
        r"/^[a]b$/iu.exec('AB')",
        r"/^(a)\1$/iv.exec('AA')",
        r"/^(?=ab)/iu.exec('AB')",
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
fn original_unicode_folded_literal_boundary_complete_programs() {
    let mut rows = String::new();
    for source in [r"/^ab$/iv.exec('AB')", r"/^(a)$/iu.exec('A')"] {
        let program = format!("JSON.stringify(eval({:?}))", JsString::from(source));
        let Value::String(result) = Realm::default().eval(&program).unwrap() else {
            panic!("{program}")
        };
        writeln!(rows, "{:?} {result:?}", JsString::from(source)).unwrap();
    }
    insta::assert_snapshot!(rows);
}

#[test]
fn unicode_folded_literal_boundary_unicode_18_capture_widths_snapshot() {
    let mut rows = String::new();
    for body in ["^(?<x>ß)(s)()$", r"^(?<x>\u{1df95})(s)()$", "((^(ß)()s$))"] {
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
