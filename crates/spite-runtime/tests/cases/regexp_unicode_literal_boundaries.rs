//! Exact complete Unicode literal matching with outer boundary assertions.
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
fn unicode_literal_boundary_results_snapshot() {
    let mut rows = String::new();
    for body in [
        "^(A)(b)$",
        "((^A(b)$))",
        "(Ab$)",
        "^A(b)",
        r"^(\u{10400})(\u{10428})$",
        r"^(\$)(A)\$$",
        "^(())A(b)$",
        "^(?<x>A)(?<y>b)$",
    ] {
        let pattern = JsString::from(body);
        for flags in ["dug", "dumg", "dvy", "dvmy"] {
            for input in [
                "Ab\naB\r\nAb\u{2028}AB\u{2029}",
                "\u{10400}\u{10428}\n\u{10428}\u{10400}",
                "$A$\n$a$",
                "Ab\n",
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
fn unicode_literal_boundary_exact_context_and_capture_aliases() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let r=/^((A)b)$/dm{mode}g,m=r.exec('ab\r\nAb\u2028AB\u2029');m.index===4&&m[1]==='Ab'&&m[2]==='A'&&m.indices[1][0]===4&&m.indices[2][1]===5&&r.lastIndex===6"
        ));
        check(&format!(
            r"let r=/A(A)$/d{mode}g,m=r.exec('AAAA');m.index===2&&m[1]==='A'&&m.indices[1][0]===3&&r.lastIndex===4"
        ));
        check(&format!(
            r"/^A(b)$/m{mode}.test('xAb\nAb')&&!/^A(b)$/{mode}.test('Ab\n')&&!/^A(b)$/m{mode}.test('aB')"
        ));
        check(&format!(
            r"let r=/^(?<x>\u{{10400}})(?<y>\u{{10428}})$/d{mode}y;r.lastIndex=1;let m=r.exec('\u{{10400}}\u{{10428}}');m.index===0&&m[1]==='\u{{10400}}'&&m[2]==='\u{{10428}}'&&m.indices.groups.x===m.indices[1]&&m.indices[1][1]===2&&r.lastIndex===4"
        ));
        check(&format!(
            r"let r=/^(?<__proto__>A)(())b$/dm{mode}g,m=r.exec('x\nAb');m.groups.__proto__==='A'&&Object.getPrototypeOf(m.groups)===null&&m[2]===''&&m[3]===''&&m.indices[2][0]===3&&m.indices.groups.__proto__===m.indices[1]"
        ));
        check(&format!(
            r"let r=/^(\uD800)(A)$/d{mode},m=r.exec('\uD800A');m[1]==='\uD800'&&m.indices[1][1]===1&&!r.test('\uD800\uDC00A')"
        ));
    }
}

#[test]
fn unicode_literal_boundary_escaped_anchors_and_consumers() {
    for mode in ["u", "v"] {
        check(&format!(
            r"/^(\$)(A)\$$/{mode}.exec('$A$')[2]==='A'&&!/^(\$)(A)\$$/{mode}.test('$a$')&&/^(A)\\$/{mode}.exec('A\\')[1]==='A'"
        ));
        check(&format!(
            r"let a=[...'Ab\r\nAb\u2028Ab\u2029'.matchAll(/^(A)(b)$/dm{mode}g)];a.length===3&&a[0].index===0&&a[1].index===4&&a[2].index===7&&a[2].indices[2][1]===9"
        ));
        check(&format!(
            r"'Ab\naB\nAb'.replace(/^(?<x>A)(b)$/m{mode}g,'<$1:$2:$<x>>')==='<A:b:A>\naB\n<A:b:A>'"
        ));
        check(&format!(
            r"let r=/^A(b)$/m{mode};r.lastIndex=99;'x\nAb'.search(r)===2&&r.lastIndex===99"
        ));
        check(&format!(
            r"'Ab\nAb'.split(/^(A)(b)$/m{mode}).join('|')==='|A|b|\n|A|b|'"
        ));
    }
}

#[test]
fn unicode_literal_boundary_metadata_flag_clones_coercions_and_writes() {
    check(
        r"let r=/^(?<x>A)(b)$/dmv,c=new RegExp(r,'diuy');c.source==='^(?<x>A)(b)$'&&c.ignoreCase&&c.unicode&&!c.multiline&&c.exec('aB').groups.x==='a'&&c.lastIndex===2",
    );
    check(
        r"let r=/^(?<x>A)(b)$/dimv,c=new RegExp(r,'duy');c.source==='^(?<x>A)(b)$'&&!c.ignoreCase&&c.exec('aB')===null&&c.exec('Ab').groups.x==='A'&&c.lastIndex===2",
    );
    check(
        r"let t='',r=/^A(b)$/uy;r.lastIndex={valueOf(){t+='i';return 0}};let m=r.exec({toString(){t+='s';return 'Ab'}});t==='si'&&m[1]==='b'&&r.lastIndex===2",
    );
    check(
        r"let r=/^A(b)$/uy;Object.defineProperty(r,'lastIndex',{writable:false});let caught=false;try{r.exec('Ab')}catch(e){caught=e instanceof TypeError}caught&&r.lastIndex===0",
    );
    check(
        r"let r=/^A(b)$/vy;r.lastIndex=1;Object.defineProperty(r,'lastIndex',{writable:false});let caught=false;try{r.exec('xAb')}catch(e){caught=e instanceof TypeError}caught&&r.lastIndex===1",
    );
}

#[test]
fn unicode_literal_boundary_exact_unicode_and_ignore_case_are_distinct() {
    for mode in ["u", "v"] {
        check(&format!(
            r"/^(ſ)(K)$/{mode}.test('ſK')&&!/^(ſ)(K)$/{mode}.test('SK')&&/^(ſ)(K)$/i{mode}.test('SK')"
        ));
        check(&format!(
            r"/^(ß)(S)$/{mode}.test('ßS')&&!/^(ß)(S)$/{mode}.test('\u{{1df95}}S')&&!/^(ß)(S)$/{mode}.test('ßs')"
        ));
        check(&format!(
            r"let r=/^(\u{{10400}})(\u{{10428}})$/{mode};r.test('\u{{10400}}\u{{10428}}')&&!r.test('\u{{10428}}\u{{10400}}')"
        ));
    }
}

#[test]
fn unicode_literal_boundary_collection_linear_rejections_and_defaults() {
    assert_eq!(Limits::default().max_steps, None);
    assert_eq!(Limits::default().max_heap_entries, None);
    let mut realm = Realm::default();
    realm.eval("let p='('.repeat(3000)+'^Ab$'+')'.repeat(3000),r=new RegExp(p,'dmvg'),c=new RegExp(r),text='😀'.repeat(100000)+'\\nAb'").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("let m=c.exec(text);m.length===3001&&m[1]==='Ab'&&m[3000]==='Ab'&&m.indices[3000][0]===200001&&m.indices[3000][1]===200003&&c.lastIndex===200003"),Ok(Value::Boolean(true)));
    realm.eval("p=null;r=null;c=null;text=null;m=null").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("let n=5000,longPattern='^'+'A'.repeat(n)+'$',longRegex=new RegExp(longPattern,'mu'),longText='A'.repeat(100000)+'b';longRegex.exec(longText)===null"),Ok(Value::Boolean(true)));
}

#[test]
fn unicode_literal_boundary_opted_work_and_remaining_gaps() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval("let r=/^A(b)$/mug,marker=0,text='😀'.repeat(10000);r.lastIndex=1")
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
        r"/^Ab+$/u.exec('Ab')",
        r"/^(Ab)+$/v.exec('Ab')",
        r"/^A|b$/u.exec('A')",
        r"/^(A$)b/v.exec('Ab')",
        r"/^[A]b$/u.exec('Ab')",
        r"/^(A)\1$/v.exec('AA')",
        r"/^(?=Ab)/u.exec('Ab')",
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
