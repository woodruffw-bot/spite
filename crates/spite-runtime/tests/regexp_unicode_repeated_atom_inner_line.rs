//! Inner line assertions keep their per-iteration scopes.
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
fn unicode_repeated_atom_inner_line_results_snapshot() {
    let mut rows = String::new();
    for atom in [
        r"(^a)+",
        r"((a$)){0,3}?",
        r"(?:(^a$)){2,4}",
        r"(?:^a)+",
        r"((?:a$)){0}",
        r"(^[aſK])+",
        r"((.$)){1,3}$",
        r"^((^\D$)){0,2}?$",
        r"^(^\p{Join_C})+$",
        r"(\p{NChar}$){1,2}$",
        r"(^😀)+",
        r"((\uD800$)){0,1}",
        r"(^\?)??",
        r"(?:(\W$))+",
        r"(^\u{61})?",
        r"((^a)){1,3}$",
    ] {
        let pattern = JsString::from(format!("(?<outer>{atom})").as_str());
        for flags in ["dug", "diumg", "divmg", "divy", "dsumg", "disvmy"] {
            for input in [
                JsString::from("Aaaa\nAA"),
                JsString::from("A\r\n\u{200c}\u{200d}\u{ffff}\u{10ffff}"),
                JsString::from("😀??aaſK"),
                JsString::from_code_units(vec![0x61, 0xd800, 0x61, 0xdc00]),
                JsString::from(""),
            ] {
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
fn unicode_repeated_atom_inner_line_counts_and_scopes() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let a=/(^a)+/{mode}.exec('aaa');a[0]==='a'&&a[1]==='a'&&a.index===0&&/^a+/{mode}.exec('aaa')[0]==='aaa'"
        ));
        check(&format!(
            r"let a=/(a$)+/{mode}.exec('aaa');a[0]==='a'&&a[1]==='a'&&a.index===2&&/a+$/{mode}.exec('aaa')[0]==='aaa'"
        ));
        check(&format!(r"/(?:(^a$)){{2}}/{mode}m.exec('a\na')===null"));
        check(&format!(
            r"let a=/((a$)){{0,3}}?/{mode}.exec('aaa');a.index===0&&a[0]===''&&a[1]===undefined&&a[2]===undefined&&Object.hasOwn(a,'1')&&Object.hasOwn(a,'2')"
        ));
        check(&format!(
            r"let r=/((?:a$)){{0}}/d{mode}y;r.lastIndex=2;let m=r.exec('aaa');m.index===2&&m[0]===''&&m[1]===undefined&&m.indices[0][0]===2&&r.lastIndex===2"
        ));
        check(&format!(
            r"let a=/(^.$){{2,3}}/ds{mode}m.exec('\r\n\nA');a[0]==='\r\n'&&a[1]==='\n'&&a.indices[1][0]===1&&a.indices[1][1]===2"
        ));
        check(&format!(
            r"let a=/^((^.$))+?$/ds{mode}m.exec('\r\n\n'),b=/^((^.$))+$/ds{mode}m.exec('\r\n\n');a[0]==='\r'&&a[1]==='\r'&&b[0]==='\r\n\n'&&b[1]==='\n'&&b.indices[2][0]===2"
        ));
    }
}
#[test]
fn unicode_repeated_atom_inner_line_names_pairs_and_folding() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let r=/(?<whole>(?<__proto__>^😀)+)/d{mode}y;r.lastIndex=1;let m=r.exec('😀😀');m.index===0&&m.groups.whole==='😀'&&m.groups.__proto__==='😀'&&m.indices.groups.__proto__[0]===0&&m.indices.groups.__proto__[1]===2&&m.indices.groups.__proto__===m.indices[2]&&r.lastIndex===2"
        ));
        check(&format!(
            r"let r=/((a$))+/di{mode}m,a=r.exec('A\nA'),b=new RegExp(r,'di{mode}').exec('A\nA');a.index===0&&a[0]==='A'&&a[1]==='A'&&b.index===2&&b[2]==='A'"
        ));
        check(&format!(
            r"let a=/(\uD800$)+/{mode}m.exec('B\uD800\n\uD800'),b=/(^\uD800)+/{mode}m.exec('\n\uD800\uD800');a.index===1&&a[0]==='\uD800'&&b.index===1&&b[0]==='\uD800'&&!/(\uD800$)+/{mode}.test('😀')"
        ));
    }
}
#[test]
fn unicode_repeated_atom_inner_line_consumers_and_empty_advance() {
    for mode in ["u", "v"] {
        check(&format!(
            r"'aaa'.replace(/(?<whole>(?<last>^a)+)/{mode},'<$1:$2:$<last>>')==='<a:a:a>aa'"
        ));
        check(&format!(
            r"'aaa'.split(/((a$))+/{mode}).join('|')==='aa|a|a|'&&'baa'.search(/(a$)+/{mode})===2"
        ));
        check(&format!(
            r"let a=[...'a\na'.matchAll(/(?<x>^a$)+/d{mode}mg)];a.length===2&&a[0].index===0&&a[0].groups.x==='a'&&a[1].index===2&&a[1].indices.groups.x[0]===2"
        ));
        check(&format!(
            r"let a=[...'😀'.matchAll(/(?<x>^a){{0}}/d{mode}g)];a.length===2&&a[0].index===0&&a[1].index===2&&a.every(m=>m.groups.x===undefined&&Object.hasOwn(m.groups,'x')&&m.indices[1]===undefined)"
        ));
    }
}
#[test]
fn unicode_repeated_atom_inner_line_clones_coercion_and_strict_writes() {
    check(
        r"let r=/(?<x>^.$)+/dsmug,c=new RegExp(r,'dvmy'),b=new RegExp(c,'dsumy');c.exec('\r\n')===null&&b.exec('\r\n').groups.x==='\n'&&b.lastIndex===2",
    );
    check(
        r"let t='',r=/(^😀)+/uy;r.lastIndex={valueOf(){t+='i';return 1}};let m=r.exec({toString(){t+='s';return '😀😀'}});t==='si'&&m.index===0&&m[1]==='😀'&&r.lastIndex===2",
    );
    check(
        r"let r=/(^a)*/vmy;Object.defineProperty(r,'lastIndex',{writable:false});let caught=false;try{r.exec('')}catch(e){caught=e instanceof TypeError}caught&&r.lastIndex===0",
    );
    check(
        r"let r=/(a$)+/vmy;r.lastIndex=1;Object.defineProperty(r,'lastIndex',{writable:false});let caught=false;try{r.exec('BBB')}catch(e){caught=e instanceof TypeError}caught&&r.lastIndex===1",
    );
}
#[test]
fn unicode_repeated_atom_inner_line_large_groups_and_collection() {
    assert_eq!(Limits::default().max_steps, None);
    assert_eq!(Limits::default().max_heap_entries, None);
    let mut realm = Realm::default();
    realm.eval(r"let inner='('.repeat(3000)+'^.$'+')'.repeat(3000),p='('.repeat(3000)+inner+'+'+')'.repeat(3000)+'$',r=new RegExp(p,'dsmvg'),c=new RegExp(r),text='\n'.repeat(200001)").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval(r"let m=c.exec(text);m.index===0&&m.length===6001&&m[1]===text&&m[3000]===text&&m[3001]==='\n'&&m[6000]==='\n'&&m.indices[6000][0]===200000&&m.indices[6000][1]===200001&&c.lastIndex===200001"),Ok(Value::Boolean(true)));
    realm
        .eval("inner=null;p=null;r=null;c=null;text=null;m=null")
        .unwrap();
    realm.collect(usize::MAX).unwrap();
}
#[test]
fn unicode_repeated_atom_inner_line_opted_work_and_unproved_units() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval(r"let r=/(\P{NChar}$){2}/umg,marker=0,text='😀'.repeat(10000);r.lastIndex=1")
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
        r"/(^$)+/u.exec('')",
        r"/(\b.)+b/u.exec('a')",
        r"/(a?)+/u.exec('a')",
        r"/(a|b)+b/v.exec('a')",
        r"/()+/u.exec('a')",
        r"/(ab)+/v.exec('ab')",
        r"/(^a)+b/u.exec('ab')",
        r"/\b(^a)+\bb/um.exec('a')",
        r"/(a^)+/u.exec('a')",
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
fn unicode_repeated_atom_inner_line_former_whole_programs_snapshot() {
    let mut rows = String::new();
    for source in [
        r"/(?:^a$)+/u.test('a')",
        r"/(?:^a$)+/v.test('a')",
        r"/(a$)+/u.test('a')",
        r"/(?:(^a))+/u.test('a')",
        r"/(?:^a)+/u.exec('a')",
        r"/(^a)+/um.exec('aa')",
        r"/(a$)+/um.exec('aa')",
        r"/(^.$)+/iu.exec('a')",
        r"/(?:^a$)+/u.exec('a')",
        r"/(^[a]$)+/iu.exec('a')",
        r"/(?:^a)+/v.exec('a')",
    ] {
        let Value::String(result) = Realm::default()
            .eval(&format!("JSON.stringify({source})"))
            .unwrap()
        else {
            panic!("{source}")
        };
        writeln!(rows, "{source} {result:?}").unwrap();
    }
    insta::assert_snapshot!(rows);
}
