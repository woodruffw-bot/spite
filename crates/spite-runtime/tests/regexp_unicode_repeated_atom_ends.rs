//! Nonmultiline trailing $ on one quantified complete Unicode atom.
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
fn unicode_repeated_atom_end_results_snapshot() {
    let mut rows = String::new();
    for atom in [
        r"a+",
        r"a*?",
        r"a??",
        r"a{2,3}",
        r"a{002,03}?",
        r".+",
        r"\D{1,2}",
        r"\W+",
        r"\p{Join_C}+",
        r"\p{NChar}{1,2}",
        r"\u{61}?",
        r"\??",
        r"a{0,0}",
        r"[aſK]+",
    ] {
        for begin in [false, true] {
            let pattern =
                JsString::from(format!("{}(?<x>{atom}$)", if begin { "^" } else { "" }).as_str());
            for flags in ["dug", "diug", "divg", "divy", "dsug", "disvy"] {
                for input in [
                    JsString::from("Aaaa"),
                    JsString::from("A\r\n\u{200c}\u{200d}\u{ffff}\u{10ffff}"),
                    JsString::from("😀??aaaſK"),
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
                        writeln!(rows,"{pattern:?} flags={flags:?} input={input:?} lastIndex={start} {result:?}").unwrap();
                    }
                }
            }
        }
    }
    insta::assert_snapshot!(rows);
}

#[test]
fn unicode_repeated_atom_ends_greedy_lazy_bounds_and_actual_endpoint() {
    for mode in ["u", "v"] {
        check(&format!(
            r"/a+$/{mode}.exec('Baaaa')[0]==='aaaa'&&/a+?$/{mode}.exec('Baaaa')[0]==='aaaa'&&/a*?$/{mode}.exec('Baaaa')[0]==='aaaa'&&/a??$/{mode}.exec('Baaaa')[0]==='a'"
        ));
        check(&format!(
            r"let a=/a{{2,3}}$/{mode}.exec('aaaa'),b=/a{{2,3}}?$/{mode}.exec('aaaa');a.index===1&&a[0]==='aaa'&&b.index===1&&b[0]==='aaa'&&/^a{{2,3}}$/{mode}.exec('aaaa')===null"
        ));
        check(&format!(
            r"['\n','\r','\u2028','\u2029'].every(t=>/a+$/{mode}.exec('aaa'+t)===null)&&/^a+$/{mode}.exec('aaa')[0]==='aaa'"
        ));
        check(&format!(
            r"/^\w+$/i{mode}.exec('ſK')[0]==='ſK'&&/^\w+$/{mode}.exec('ſK')===null&&/^\W+$/{mode}.exec('ſK')[0]==='ſK'"
        ));
        check(&format!(
            r"/^\p{{Join_C}}+$/{mode}.exec('\u200c\u200d')[0]==='\u200c\u200d'&&/\p{{NChar}}{{2}}$/{mode}.exec('A\uFFFF\u{{10FFFF}}')[0]==='\uFFFF\u{{10FFFF}}'"
        ));
        check(&format!(
            r"/.+$/{mode}.exec('😀\nA')[0]==='A'&&/.+$/s{mode}.exec('😀\nA')[0]==='😀\nA'&&/^.+?$/s{mode}.exec('😀\nA')[0]==='😀\nA'"
        ));
    }
}

#[test]
fn unicode_repeated_atom_ends_complete_captures_sticky_and_original_pairs() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let r=/^(?<__proto__>(😀+))$/d{mode}y;r.lastIndex=1;let m=r.exec('😀😀');m.index===0&&m[0]==='😀😀'&&m[2]==='😀😀'&&m.indices[2][1]===4&&m.indices.groups.__proto__===m.indices[1]&&r.lastIndex===4"
        ));
        check(&format!(
            r"let r=/(?<x>a{{0}}$)/d{mode}g;r.lastIndex=1;let m=r.exec('😀');m.index===2&&m.groups.x===''&&m.indices.groups.x[0]===2&&r.lastIndex===2"
        ));
        check(&format!(
            r"let r=/a{{2,3}}$/{mode}y;r.exec('aaaa')===null&&r.lastIndex===0&&(r.lastIndex=1,r.exec('aaaa')[0]==='aaa')&&r.lastIndex===4"
        ));
        check(&format!(
            r"let r=/\uD800+$/{mode}g,m=r.exec('B\uD800\uD800');m.index===1&&m[0]==='\uD800\uD800'&&r.lastIndex===3&&!/\uD800+$/{mode}.test('😀')"
        ));
        check(&format!(
            r"let r=/a{{0}}$/{mode}y;r.lastIndex=1;r.exec('😀')===null&&r.lastIndex===0&&/^a{{0}}$/{mode}.exec('')[0]===''&&/^a{{0}}$/{mode}.exec('a')===null"
        ));
    }
}

#[test]
fn unicode_repeated_atom_end_consumers_and_empty_advance() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let a=[...'A😀B'.matchAll(/(?<x>a{{0}}$)/d{mode}g)];a.length===1&&a[0].index===4&&a[0].groups.x===''&&a[0].indices.groups.x===a[0].indices[1]"
        ));
        check(&format!(
            r"'Baa'.replace(/(?<x>a+$)/{mode}g,'<$1:$<x>>')==='B<aa:aa>'"
        ));
        check(&format!(
            r"'Baaa'.split(/(a+$)/{mode}).join('|')==='B|aaa|'&&'Baaa'.search(/a+$/{mode})===1&&'aa aaaa'.match(/a+$/g{mode}).join(',')==='aaaa'"
        ));
        check(&format!(
            r"let t='',a='B😀😀'.replace(/(?<x>😀+?$)/{mode}g,(m,c,p,text,g)=>{{t+=p;return '['+g.x+']'}});a==='B[😀😀]'&&t==='1'"
        ));
    }
}

#[test]
fn unicode_repeated_atom_end_clones_coercions_and_strict_writes() {
    check(
        r"let r=/^(?<x>a+)$/diug,c=new RegExp(r,'dvy'),b=new RegExp(c,'diuy');c.unicodeSets&&!c.ignoreCase&&c.exec('AA')===null&&b.exec('AA').groups.x==='AA'&&b.lastIndex===2",
    );
    check(
        r"let t='',r=/^😀+$/uy;r.lastIndex={valueOf(){t+='i';return 1}};let m=r.exec({toString(){t+='s';return '😀😀'}});t==='si'&&m.index===0&&m[0]==='😀😀'&&r.lastIndex===4",
    );
    check(
        r"let r=/^a*$/vy;Object.defineProperty(r,'lastIndex',{writable:false});let caught=false;try{r.exec('')}catch(e){caught=e instanceof TypeError}caught&&r.lastIndex===0",
    );
    check(
        r"let r=/a+$/vy;r.lastIndex=1;Object.defineProperty(r,'lastIndex',{writable:false});let caught=false;try{r.exec('BBB')}catch(e){caught=e instanceof TypeError}caught&&r.lastIndex===1",
    );
}

#[test]
fn unicode_repeated_atom_ends_captures_collection_and_inputs_are_unlimited() {
    assert_eq!(Limits::default().max_steps, None);
    assert_eq!(Limits::default().max_heap_entries, None);
    let mut realm = Realm::default();
    realm.eval(r"let p='(' .repeat(3000)+'😀+'+')'.repeat(3000)+'$',r=new RegExp(p,'dvg'),c=new RegExp(r),text='B'+'😀'.repeat(100000)").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("let m=c.exec(text);m.index===1&&m.length===3001&&m[1]===text.slice(1)&&m[3000]===text.slice(1)&&m.indices[3000][0]===1&&m.indices[3000][1]===200001&&c.lastIndex===200001"),Ok(Value::Boolean(true)));
    realm.eval("p=null;r=null;c=null;text=null;m=null").unwrap();
    realm.collect(usize::MAX).unwrap();
}

#[test]
fn unicode_repeated_atom_ends_opted_work_and_remaining_compositions() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval(r"let r=/\P{NChar}+$/ug,marker=0,text='😀'.repeat(10000);r.lastIndex=1")
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
        r"/a+b$/um.exec('a')",
        r"/^a+b$/vm.exec('a')",
        r"/a+$$/u.exec('a')",
        r"/(a)+b$/u.exec('a')",
        r"/a+b$/v.exec('ab')",
        r"/\ba+$/u.exec('a')",
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
fn unicode_repeated_atom_end_former_whole_programs_snapshot() {
    let mut rows = String::new();
    for source in [
        r"/^a+$/u.test('a')",
        r"/^(.+)$/v.exec('a')",
        r"/^\D+$/v.exec('ab')",
        r"/^[a]+$/u.exec('a')",
        r"/^a+$/u.exec('a')",
        r"/a+$/u.exec('a')",
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
