//! Repeated complete atom groups capture their final selected iteration.
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
fn unicode_captured_atom_results_snapshot() {
    let mut rows = String::new();
    for atom in [
        r"(a)+",
        r"((a)){0,3}?",
        r"(?:(a)){2,4}",
        r"(?:a)+",
        r"((?:a)){0}",
        r"([aſK])+",
        r"((.)){1,3}$",
        r"^((\D)){0,2}?$",
        r"^(\p{Join_C})+$",
        r"(\p{NChar}){1,2}$",
        r"(😀)+",
        r"((\uD800)){0,1}",
        r"(\?)??",
        r"(?:(\W))+",
        r"(\u{61})?",
        r"((a)){1,3}$",
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
fn unicode_captured_atoms_last_iteration_zero_and_bounds() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let a=/((a))+/{mode}.exec('aaa');a[0]==='aaa'&&a[1]==='a'&&a[2]==='a'&&a.length===3"
        ));
        check(&format!(
            r"let a=/((a)){{0,3}}?/{mode}.exec('aaa');a[0]===''&&a[1]===undefined&&a[2]===undefined&&Object.hasOwn(a,'1')&&Object.hasOwn(a,'2')"
        ));
        check(&format!(
            r"let a=/((?:a)){{0}}/{mode}.exec('aaa'),b=/(?:(a)){{2,3}}?/{mode}.exec('aaaa');a[0]===''&&a[1]===undefined&&b[0]==='aa'&&b[1]==='a'&&b.length===2"
        ));
        check(&format!(
            r"let a=/^(?<x>(\D)){{1,4}}$/{mode}m.exec('ab\ncdef');a[0]==='ab'&&a.groups.x==='b'&&a[2]==='b'&&a.index===0"
        ));
        check(&format!(
            r"let a=/([aſK])+/i{mode}.exec('AſK');a[0]==='AſK'&&a[1]==='K'&&/(?:a)+/{mode}.exec('aaa').length===1"
        ));
        check(&format!(
            r"let a=/^((.))+$/{mode}sm.exec('a\nb'),b=/((\D))+?$/{mode}m.exec('a\nb');a[1]==='b'&&a[2]==='b'&&b[0]==='a'&&b[1]==='a'&&b[2]==='a'"
        ));
    }
}

#[test]
fn unicode_captured_atoms_names_outer_scopes_and_original_pairs() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let r=/^(?<whole>(?<__proto__>😀)+)$/d{mode}my;r.lastIndex=1;let m=r.exec('😀😀\n');m.index===0&&m[0]==='😀😀'&&m.groups.whole==='😀😀'&&m.groups.__proto__==='😀'&&m.indices[1][0]===0&&m.indices[1][1]===4&&m.indices[2][0]===2&&m.indices[2][1]===4&&m.indices.groups.__proto__===m.indices[2]&&r.lastIndex===4"
        ));
        check(&format!(
            r"let a=/(?<whole>(?<inner>a){{0}})/d{mode}.exec('A');a.groups.whole===''&&a.groups.inner===undefined&&Object.hasOwn(a.groups,'inner')&&a.indices.groups.whole===a.indices[1]&&a.indices[2]===undefined&&Object.hasOwn(a.indices.groups,'inner')"
        ));
        check(&format!(
            r"let a=/((\uD800))+/{mode}.exec('B\uD800\uD800');a.index===1&&a[0]==='\uD800\uD800'&&a[1]==='\uD800'&&a[2]==='\uD800'&&!/(\uD800)+/{mode}.test('😀')"
        ));
    }
}

#[test]
fn unicode_captured_atoms_consumers_and_empty_advance() {
    for mode in ["u", "v"] {
        check(&format!(
            r"'aaaa'.replace(/(?<whole>(?<last>a)+)/{mode},'<$1:$2:$<last>>')==='<aaaa:a:a>'"
        ));
        check(&format!(
            r"let a='baaac'.split(/((a))+/{mode});a.join('|')==='b|a|a|c'&&'baaac'.search(/(a)+/{mode})===1"
        ));
        check(&format!(
            r"let a=[...'aaa\naa'.matchAll(/(?<x>a)+$/d{mode}mg)];a.length===2&&a[0][0]==='aaa'&&a[0].groups.x==='a'&&a[0].indices.groups.x[0]===2&&a[1].index===4&&a[1].indices.groups.x[0]===5"
        ));
        check(&format!(
            r"let a=[...'😀'.matchAll(/(?<x>a){{0}}/d{mode}g)];a.length===2&&a[0].index===0&&a[1].index===2&&a.every(m=>m.groups.x===undefined&&Object.hasOwn(m.groups,'x')&&m.indices[1]===undefined)"
        ));
        check(&format!(
            r"let t='',s='B😀😀'.replace(/(?<x>😀)+/{mode}g,(m,c,p,text,g)=>{{t+=p;return '['+g.x+']'}});s==='B[😀]'&&t==='1'"
        ));
    }
}

#[test]
fn unicode_captured_atoms_clones_coercions_and_strict_writes() {
    check(
        r"let r=/^(?<x>a)+$/diumg,c=new RegExp(r,'dvmy'),b=new RegExp(c,'diumy');c.exec('AA\n')===null&&b.exec('AA\n').groups.x==='A'&&b.lastIndex===2",
    );
    check(
        r"let t='',r=/^(😀)+$/umy;r.lastIndex={valueOf(){t+='i';return 1}};let m=r.exec({toString(){t+='s';return '😀😀\n'}});t==='si'&&m.index===0&&m[1]==='😀'&&r.lastIndex===4",
    );
    check(
        r"let r=/(a)*/vmy;Object.defineProperty(r,'lastIndex',{writable:false});let caught=false;try{r.exec('')}catch(e){caught=e instanceof TypeError}caught&&r.lastIndex===0",
    );
    check(
        r"let r=/(a)+/vmy;r.lastIndex=1;Object.defineProperty(r,'lastIndex',{writable:false});let caught=false;try{r.exec('BBB')}catch(e){caught=e instanceof TypeError}caught&&r.lastIndex===1",
    );
}

#[test]
fn unicode_captured_atoms_large_inner_outer_groups_and_collection() {
    assert_eq!(Limits::default().max_steps, None);
    assert_eq!(Limits::default().max_heap_entries, None);
    let mut realm = Realm::default();
    realm.eval(r"let inner='('.repeat(3000)+'😀'+')'.repeat(3000),p='('.repeat(3000)+inner+'+'+')'.repeat(3000)+'$',r=new RegExp(p,'dvg'),c=new RegExp(r),text='B'+'😀'.repeat(100000)").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("let m=c.exec(text);m.index===1&&m.length===6001&&m[1]===text.slice(1)&&m[3000]===text.slice(1)&&m[3001]==='😀'&&m[6000]==='😀'&&m.indices[3000][0]===1&&m.indices[3000][1]===200001&&m.indices[6000][0]===199999&&m.indices[6000][1]===200001&&c.lastIndex===200001"),Ok(Value::Boolean(true)));
    realm
        .eval("inner=null;p=null;r=null;c=null;text=null;m=null")
        .unwrap();
    realm.collect(usize::MAX).unwrap();
}

#[test]
fn unicode_captured_atoms_opted_work_and_remaining_compositions() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval(r"let r=/(\P{NChar})+$/umg,marker=0,text='😀'.repeat(10000);r.lastIndex=1")
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
        r"/(^a)+/um.exec('aa')",
        r"/(a$)+/um.exec('aa')",
        r"/(a?)+/u.exec('aa')",
        r"/(a|b)+/v.exec('aa')",
        r"/()+/u.exec('a')",
        r"/(ab)+/v.exec('ab')",
        r"/(a)+b/u.exec('ab')",
        r"/(\b)+/u.exec('a')",
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
fn unicode_captured_atom_former_whole_programs_snapshot() {
    let mut rows = String::new();
    for source in [
        r"/([\u{1f600}])+/u.test('a')",
        r"/(.)+/v.test('a')",
        r"/(a)+/u.test('a')",
        r"/(?:a)+/u.test('a')",
        r"/^([a])+$/u.test('a')",
        r"/^([a])+$/v.test('a')",
        r"/([a])+/iu.exec('a')",
        r"/^([a])+$/iu.exec('a')",
        r"/^((.)+)$/u.exec('A')",
        r"/([\u{1f600}])+/v.exec('a')",
        r"/([^a])+/u.exec('b')",
        r"/(\D)+/u.exec('a')",
        r"/([\uD800])+/u.exec('\uD800')",
        r"/([\uD7FF-\uE000])+/u.exec('a')",
        r"/([\u{1f600}])+/u.exec('a')",
        r"/([a])+/u.exec('a')",
        r"/(.)+/u.exec('a')",
        r"/(?:a)+/iu.exec('a')",
        r"/([\u{10000}])+/u.exec('a')",
        r"/([\u{D800}])+/v.exec('\uD800')",
        r"/([\u{D7FF}-\u{E000}])+/u.exec('a')",
        r"/([\u{61}])+/iv.exec('a')",
        r"/([^\u{61}])+/u.exec('b')",
        r"/([\u{61}])+/u.exec('a')",
        r"/(?:\u{61})+/iu.exec('a')",
        r"/^([a])+$/v.exec('a')",
        r"/([^a])+/iv.exec('b')",
        r"/(\w)+/iu.exec('a')",
        r"/^([a])+$/u.exec('a')",
        r"/(.)+/iu.exec('a')",
        r"/(\W)+/iv.exec('a')",
        r"/./v.exec('a')+/(.)+/v.exec('a')",
        r"/([\D])+/u.exec('a')",
        r"/([^a])+/v.exec('b')",
        r"/^(a)+$/iu.exec('A')",
        r"/(a)+$/u.exec('a')",
        r"/(a)+$/um.exec('a')",
        r"/^(a)+/u.exec('a')",
        r"/(a)+/u.exec('a')",
        r"/([\W])+/iv.exec('a')",
        r"/(?:\u{1f600})+/iu.exec('😀')",
        r"/([\u{1f600}])+/v.exec('😀')",
        r"/([a])+/iv.exec('a')",
        r"/(\D)+/v.exec('a')",
        r"/([\uD800])+/v.exec('\uD800')",
        r"/([\uD7FF-\uE000])+/v.exec('a')",
        r"/([a])+/v.exec('a')",
        r"/(.)+/v.exec('a')",
        r"/(?:a)+/iv.exec('a')",
        r"/(?:\uD800)+/iu.exec('\uD800')",
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

#[test]
fn unicode_captured_atom_former_direct_eval_runs_finally() {
    let mut realm = Realm::default();
    realm.eval("var marker=0;").unwrap();
    let code = "marker=1; /([\\u{1f600}])+/v.test('a');";
    let operation = format!("try{{eval({code:?});}}catch{{marker=2;}}finally{{marker=3;}}");
    assert_eq!(realm.eval(&operation), Ok(Value::Boolean(false)));
    assert_eq!(realm.eval("marker"), Ok(Value::Number(3.0)));
}
