//! Complete groups around single Unicode atoms (22.2.2.7, 22.2.7.2).
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
fn unicode_atom_capture_results_snapshot() {
    let mut rows = String::new();
    let inputs = [
        JsString::from("😀ſK9\n"),
        JsString::from_code_units(vec![0xd800, 0xa, 0xdc00]),
        JsString::from("\nſ\n"),
    ];
    for body in [
        ".",
        r"\w",
        r"\W",
        "[A-Z]",
        "[^A-Z]",
        r"[\uD800-\uDFFF]",
        r"[\u{1f600}]",
        "[^]",
    ] {
        for (left, right) in [
            ("(", ")"),
            ("(?:(", "))"),
            ("((?:", "))"),
            ("((", "))"),
            ("(?<x>", ")"),
            ("(^", "$)"),
        ] {
            let pattern = JsString::from(format!("{left}{body}{right}").as_str());
            for flags in ["diug", "diumsy", "divg", "divmsy"] {
                for input in &inputs {
                    for start in [0, 1, 2, input.len() + 1] {
                        let program = format!(
                            "let r=new RegExp({pattern:?},'{flags}');r.lastIndex={start};let m=r.exec({input:?});JSON.stringify(m===null?{{match:null,lastIndex:r.lastIndex}}:{{matches:[...m],index:m.index,input:m.input,groups:m.groups,indices:m.indices,indexGroups:m.indices.groups,lastIndex:r.lastIndex,source:r.source}})"
                        );
                        let Value::String(result) = Realm::default().eval(&program).unwrap() else {
                            panic!("{program}")
                        };
                        writeln!(rows, "{pattern:?} flags={flags:?} input={input:?} lastIndex={start} {result:?}").unwrap();
                    }
                }
            }
        }
    }
    insta::assert_snapshot!(rows);
}

#[test]
fn unicode_atom_captures_named_groups_and_original_boundaries() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let r=/(?<a>(?:(?<b>.)))/di{mode}y;r.lastIndex=1;let m=r.exec('😀');m.length===3&&m[0]==='😀'&&m[1]==='😀'&&m[2]==='😀'&&m.index===0&&m.indices[0][0]===0&&m.indices[1][0]===0&&m.indices[2][1]===2&&m.groups.a==='😀'&&m.groups.b==='😀'&&m.indices.groups.a===m.indices[1]&&m.indices.groups.b===m.indices[2]&&r.lastIndex===2"
        ));
        check(&format!(
            r"let m=/(?<__proto__>[A-Z])/di{mode}.exec('ſ');Object.getPrototypeOf(m.groups)===null&&m.groups.__proto__==='ſ'&&Object.getPrototypeOf(m.indices.groups)===null&&m.indices.groups.__proto__===m.indices[1]"
        ));
        check(&format!(
            r"let m=/((?:[\uD800-\uDFFF]))/d{mode}.exec('😀\uD800');m[1]==='\uD800'&&m.index===2&&m.indices[1][0]===2&&m.indices[1][1]===3"
        ));
        check(&format!(
            r"let m=/(?:(^[A-Z]$))/dim{mode}.exec('\rſ\n');m[1]==='ſ'&&m.index===1&&m.indices[1][0]===1&&m.indices[1][1]===2"
        ));
    }
}

#[test]
fn unicode_atom_captures_string_consumers() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let a=[...'😀ſ'.matchAll(/((?:.))/d{mode}g)];a.length===2&&a[0][1]==='😀'&&a[0].indices[1][1]===2&&a[1][1]==='ſ'&&a[1].index===2"
        ));
        check(&format!(
            r"'😀ſ'.replace(/(?<x>.)/{mode}g,'<$1:$<x>>')==='<😀:😀><ſ:ſ>'"
        ));
        check(&format!(
            r"let seen=[];let s='😀ſ'.replace(/((.))/{mode}g,(m,a,b,i)=>{{seen.push(i);return a+b}});s==='😀😀ſſ'&&seen.join(',')==='0,2'"
        ));
        check(&format!(
            r"'😀ſ'.split(/((?:.))/{mode}).join('|')==='|😀||ſ|'"
        ));
        check(&format!(
            r"let r=/([A-Z])/i{mode};r.lastIndex=7;'😀ſ'.search(r)===2&&r.lastIndex===7"
        ));
    }
}

#[test]
fn unicode_atom_captures_clones_coercions_and_strict_writes() {
    check(
        r"let r=/(?<x>[A-Z])/dimv,c=new RegExp(r,'diuy');c.source==='(?<x>[A-Z])'&&c.ignoreCase&&c.unicode&&!c.unicodeSets&&!c.multiline&&c.exec('K').groups.x==='K'&&c.lastIndex===1",
    );
    check(
        r"let t='',r=/(.)/duy;r.lastIndex={valueOf(){t+='i';return 1}};let m=r.exec({toString(){t+='s';return '😀'}});t==='si'&&m[1]==='😀'&&m.index===0&&r.lastIndex===2",
    );
    check(
        r"let r=/(.)/uy;Object.defineProperty(r,'lastIndex',{writable:false});let caught=false;try{r.exec('😀')}catch(e){caught=e instanceof TypeError}caught&&r.lastIndex===0",
    );
    check(
        r"let r=/([A-Z])/ivy;r.lastIndex=1;Object.defineProperty(r,'lastIndex',{writable:false});let caught=false;try{r.exec('😀')}catch(e){caught=e instanceof TypeError}caught&&r.lastIndex===1",
    );
}

#[test]
fn unicode_atom_captures_many_wrappers_collection_and_defaults() {
    assert_eq!(Limits::default().max_steps, None);
    assert_eq!(Limits::default().max_heap_entries, None);
    let mut realm = Realm::default();
    realm.eval("let p='('.repeat(2000)+'[A-Z]'+')'.repeat(2000),r=new RegExp(p,'divg'),c=new RegExp(r),text='😀'.repeat(100000)+'K'").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("let m=c.exec(text);m.length===2001&&m[1]==='K'&&m[2000]==='K'&&m.indices[2000][0]===200000&&m.indices[2000][1]===200001&&c.lastIndex===200001"), Ok(Value::Boolean(true)));
    realm.eval("p=null;r=null;c=null;text=null;m=null").unwrap();
    realm.collect(usize::MAX).unwrap();
}

#[test]
fn unicode_atom_captures_opted_work_and_unproved_syntax() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval("let r=/([A-Z])/iug,marker=0,text='😀'.repeat(10000);r.lastIndex=1")
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
        r"/([a])+/iu.exec('a')",
        r"/([a])b/iv.exec('ab')",
        r"/([a]|b)/iu.exec('a')",
        r"/^([a])$/iu.exec('a')",
        r"/(?=([a]))/iv.exec('a')",
        r"/(?:[[a]])/iv.exec('a')",
        r"/(?:[\q{a\)b}])/iv.exec('a)b')",
        r"/([\p{ASCII}])/iu.exec('a')",
        r"/(.)(.)/u.exec('ab')",
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
fn original_unicode_atom_capture_complete_programs() {
    let mut rows = String::new();
    for source in [
        r"/([\u{1f600}])/u.test('a')",
        r"/(.)/v.test('a')",
        r"/([\u{1f600}])/v.exec('a')",
        r"/([a])/iu.exec('a')",
        r"/([^a])/u.exec('b')",
        r"/(\D)/u.exec('a')",
        r"/([\uD800])/u.exec('\uD800')",
        r"/([\uD7FF-\uE000])/u.exec('a')",
        r"/([\u{1f600}])/u.exec('a')",
        r"/([a])/u.exec('a')",
        r"/(.)/u.exec('a')",
        r"/([\u{10000}])/u.exec('a')",
        r"/([\u{D800}])/v.exec('\uD800')",
        r"/([\u{D7FF}-\u{E000}])/u.exec('a')",
        r"/([\u{61}])/iv.exec('a')",
        r"/([^\u{61}])/u.exec('b')",
        r"/([\u{61}])/u.exec('a')",
        r"/(^.$)/iu.exec('a')",
        r"/([^a])/iv.exec('b')",
        r"/(\w)/iu.exec('a')",
        r"/(^[a]$)/iu.exec('a')",
        r"/(.)/iu.exec('a')",
        r"/(\W)/iv.exec('a')",
        r"/./v.exec('a')+/(.)/v.exec('a')",
        r"/([\D])/u.exec('a')",
        r"/([^a])/v.exec('b')",
        r"/([\W])/iv.exec('a')",
        r"/([\u{1f600}])/v.exec('😀')",
        r"/([a])/iv.exec('a')",
        r"/(\D)/v.exec('a')",
        r"/([\uD800])/v.exec('\uD800')",
        r"/([\uD7FF-\uE000])/v.exec('a')",
        r"/([a])/v.exec('a')",
        r"/(.)/v.exec('a')",
    ] {
        let program = format!("JSON.stringify(eval({:?}))", JsString::from(source));
        let Value::String(result) = Realm::default().eval(&program).unwrap() else {
            panic!("{program}")
        };
        writeln!(rows, "{:?} {result:?}", JsString::from(source)).unwrap();
    }
    insta::assert_snapshot!(rows);
}
