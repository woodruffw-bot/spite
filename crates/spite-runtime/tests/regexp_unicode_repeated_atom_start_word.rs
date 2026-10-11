//! Leading word assertions retain complete repetition and capture scopes.
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
fn unicode_repeated_atom_start_word_results_snapshot() {
    let mut rows = String::new();
    for atom in [
        r"\b(a)+",
        r"\B((a)){0,3}?",
        r"\b(?:(a)){2,4}",
        r"\B(?:a)+",
        r"\B((?:a)){0}",
        r"\b([aſK])+",
        r"\B((.)){1,3}$",
        r"^\b((\D)){0,2}?$",
        r"^\b(\p{Join_C})+$",
        r"\B(\p{NChar}){1,2}$",
        r"\b(😀)+",
        r"\B((\uD800)){0,1}",
        r"\b(\?)??",
        r"\B(?:(\W))+",
        r"\b(\u{61})?",
        r"\B((a)){1,3}$",
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
fn unicode_repeated_atom_start_word_bounds_and_final_captures() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let a=/\B((a))+/{mode}.exec('aaaa');a.index===1&&a[0]==='aaa'&&a[1]==='a'&&a[2]==='a'"
        ));
        check(&format!(
            r"let a=/\b(a)+/{mode}.exec('baaa aaa');a.index===5&&a[0]==='aaa'&&a[1]==='a'"
        ));
        check(&format!(
            r"let a=/\B(a){{0,2}}?/{mode}.exec('aa');a.index===1&&a[0]===''&&a[1]===undefined&&Object.hasOwn(a,'1')"
        ));
        check(&format!(
            r"let a=/\B(a){{0}}$/{mode}.exec('a ');a.index===2&&a[0]===''&&a[1]===undefined"
        ));
        check(&format!(
            r"let a=/\B(a){{2,3}}$/{mode}.exec('aaaa');a.index===1&&a[0]==='aaa'&&a[1]==='a'"
        ));
        check(&format!(
            r"let a=/^\b((\D))+?$/{mode}m.exec('ab\ncdef');a.index===0&&a[0]==='ab'&&a[1]==='b'&&a[2]==='b'"
        ));
        check(&format!(
            r"let a=/^\B((.))+$/s{mode}m.exec('\nabc');a.index===0&&a[0]==='\nabc'&&a[1]==='c'"
        ));
    }
}

#[test]
fn unicode_repeated_atom_start_word_names_pairs_and_folding() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let r=/(?<whole>\b(?<__proto__>😀)+)/d{mode}y;r.lastIndex=2;let m=r.exec('A😀😀');m.index===1&&m.groups.whole==='😀😀'&&m.groups.__proto__==='😀'&&m.indices.groups.whole[0]===1&&m.indices.groups.__proto__[0]===3&&m.indices.groups.__proto__[1]===5&&m.indices.groups.__proto__===m.indices[2]&&r.lastIndex===5"
        ));
        check(&format!(
            r"let a=/\b([sK])+$/i{mode}.exec('ſK');a.index===0&&a[0]==='ſK'&&a[1]==='K'&&/\B([sK])+$/i{mode}.exec('ſK').index===1"
        ));
        check(&format!(
            r"let a=/\b(\uD800)+/{mode}.exec('A\uD800\uD800');a.index===1&&a[0]==='\uD800\uD800'&&a[1]==='\uD800'&&!/\b(\uD800)+/{mode}.test('A😀')"
        ));
    }
}

#[test]
fn unicode_repeated_atom_start_word_consumers_and_empty_advance() {
    for mode in ["u", "v"] {
        check(&format!(
            r"'aaaa aaa'.replace(/(?<whole>\B(?<last>a)+)/{mode}g,'<$1:$2:$<last>>')==='a<aaa:a:a> a<aa:a:a>'"
        ));
        check(&format!(
            r"'aaaa'.split(/\B((a))+/{mode}).join('|')==='a|a|a|'&&'baaa aaa'.search(/\b(a)+/{mode})===5"
        ));
        check(&format!(
            r"let a=[...'aaaa\naa'.matchAll(/\B(?<x>a)+$/d{mode}mg)];a.length===2&&a[0].index===1&&a[0].indices.groups.x[0]===3&&a[1].index===6&&a[1].groups.x==='a'"
        ));
        check(&format!(
            r"let a=[...'😀'.matchAll(/\B(?<x>a){{0}}/d{mode}g)];a.length===2&&a[0].index===0&&a[1].index===2&&a.every(m=>m.groups.x===undefined&&m.indices[1]===undefined)"
        ));
    }
}

#[test]
fn unicode_repeated_atom_start_word_clone_coercion_and_strict_writes() {
    check(
        r"let r=/\b(?<x>a)+$/diumg,c=new RegExp(r,'dvmy'),b=new RegExp(c,'diumy');c.exec('AA\n')===null&&b.exec('AA\n').groups.x==='A'&&b.lastIndex===2",
    );
    check(
        r"let t='',r=/\b(😀)+/uy;r.lastIndex={valueOf(){t+='i';return 2}};let m=r.exec({toString(){t+='s';return 'A😀😀'}});t==='si'&&m.index===1&&m[1]==='😀'&&r.lastIndex===5",
    );
    check(
        r"let r=/\B(a)*/vmy;Object.defineProperty(r,'lastIndex',{writable:false});let caught=false;try{r.exec('')}catch(e){caught=e instanceof TypeError}caught&&r.lastIndex===0",
    );
    check(
        r"let r=/\b(a)+/vmy;r.lastIndex=1;Object.defineProperty(r,'lastIndex',{writable:false});let caught=false;try{r.exec('BBB')}catch(e){caught=e instanceof TypeError}caught&&r.lastIndex===1",
    );
}

#[test]
fn unicode_repeated_atom_start_word_large_captures_and_collection() {
    assert_eq!(Limits::default().max_steps, None);
    assert_eq!(Limits::default().max_heap_entries, None);
    let mut realm = Realm::default();
    realm.eval(r"let inner='('.repeat(3000)+'😀'+')'.repeat(3000),p='('.repeat(3000)+'\\b'+inner+'+'+')'.repeat(3000)+'$',r=new RegExp(p,'dvg'),c=new RegExp(r),text='A'+'😀'.repeat(100000)").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("let m=c.exec(text);m.index===1&&m.length===6001&&m[1]===text.slice(1)&&m[3000]===text.slice(1)&&m[3001]==='😀'&&m[6000]==='😀'&&m.indices[6000][0]===199999&&m.indices[6000][1]===200001&&c.lastIndex===200001"),Ok(Value::Boolean(true)));
    realm
        .eval("inner=null;p=null;r=null;c=null;text=null;m=null")
        .unwrap();
    realm.collect(usize::MAX).unwrap();
}

#[test]
fn unicode_repeated_atom_start_word_opted_work_and_unproved_scopes() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval(r"let r=/\B(\P{NChar})+$/umg,marker=0,text='😀'.repeat(10000);r.lastIndex=1")
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
        r"/\b(a)+\bb/u.exec('a')",
        r"/\b(^a)+b/um.exec('a')",
        r"/\b(a$)+b/um.exec('a')",
        r"/\b(a?)+/u.exec('a')",
        r"/\b(a|b)+/v.exec('a')",
        r"/\b()+/u.exec('a')",
        r"/\b(ab)+/v.exec('ab')",
        r"/\b(a)+b/u.exec('ab')",
        r"/\b(\b.)+b/v.exec('a')",
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
fn unicode_repeated_atom_start_word_former_whole_programs_snapshot() {
    let mut rows = String::new();
    for source in [
        r"/\ba+$/u.exec('a')",
        r"/\ba+$/um.exec('a')",
        r"/^\ba+/u.exec('a')",
        r"/\ba+/u.exec('a')",
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
