//! Multiline trailing assertions preserve start and greedy/lazy priorities.
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
fn unicode_repeated_atom_multiline_end_results_snapshot() {
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
        r".+?",
        r"\D+?",
        r"\D{1,3}?",
        r"a{1,3}?",
    ] {
        for begin in [false, true] {
            let pattern =
                JsString::from(format!("{}(?<x>{atom}$)", if begin { "^" } else { "" }).as_str());
            for flags in ["dumg", "diumg", "divmg", "divmy", "dsumg", "disvmy"] {
                for input in [
                    JsString::from("Aaaa\nAA"),
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
fn unicode_repeated_atom_multiline_ends_order_bounds_and_terminators() {
    for mode in ["u", "v"] {
        check(&format!(
            r"/\D+$/{mode}m.exec('ab\ncd')[0]==='ab\ncd'&&/\D+?$/{mode}m.exec('ab\ncd')[0]==='ab'&&/.+$/{mode}m.exec('ab\ncd')[0]==='ab'&&/.+?$/{mode}sm.exec('ab\ncd')[0]==='ab'&&/.+$/{mode}sm.exec('ab\ncd')[0]==='ab\ncd'"
        ));
        check(&format!(
            r"let a=/a{{2,3}}$/{mode}m.exec('aaaa\nbb'),b=/a{{2,3}}?$/{mode}m.exec('aaaa\nbb');a.index===1&&a[0]==='aaa'&&b.index===1&&b[0]==='aaa'&&/^a{{2,3}}$/{mode}m.exec('aaaa\nbb')===null"
        ));
        check(&format!(
            r"let a=/.{{2,4}}$/{mode}sm.exec('ab\ncdef');a.index===0&&a[0]==='ab'&&/a*?$/{mode}m.exec('aaa\n')[0]==='aaa'&&/a??$/{mode}m.exec('aaa\n').index===2"
        ));
        check(&format!(
            r"['\n','\r','\u2028','\u2029'].every(t=>/a+$/{mode}m.exec('aaa'+t)[0]==='aaa')&&/^a{{2}}$/{mode}m.exec('aaa\naa\nA').index===4"
        ));
        check(&format!(
            r"let a=[...'\r\n'.matchAll(/^a{{0}}$/{mode}mg)];a.length===3&&a[0].index===0&&a[1].index===1&&a[2].index===2"
        ));
        check(&format!(
            r"/^\w+$/i{mode}m.exec('ſK\n')[0]==='ſK'&&/^\w+$/{mode}m.exec('ſK\n')===null&&/^\W+$/{mode}m.exec('ſK\n')[0]==='ſK\n'"
        ));
    }
}

#[test]
fn unicode_repeated_atom_multiline_ends_captures_sticky_and_original_pairs() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let r=/^(?<__proto__>(😀+))$/d{mode}my;r.lastIndex=1;let m=r.exec('😀😀\n');m.index===0&&m[0]==='😀😀'&&m[2]==='😀😀'&&m.indices[2][1]===4&&m.indices.groups.__proto__===m.indices[1]&&r.lastIndex===4"
        ));
        check(&format!(
            r"let r=/a{{2,3}}$/{mode}my;r.exec('aaaa\n')===null&&r.lastIndex===0&&(r.lastIndex=1,r.exec('aaaa\n')[0]==='aaa')&&r.lastIndex===4"
        ));
        check(&format!(
            r"let r=/a{{0}}$/{mode}my;r.lastIndex=1;r.exec('😀\n')===null&&r.lastIndex===0&&(r.lastIndex=2,r.exec('😀\n').index===2)&&r.lastIndex===2"
        ));
        check(&format!(
            r"let r=/\uD800+$/{mode}mg,m=r.exec('B\uD800\uD800\n');m.index===1&&m[0]==='\uD800\uD800'&&r.lastIndex===3&&!/\uD800+$/{mode}m.test('😀\n')"
        ));
        check(&format!(
            r"let a=/^\p{{Join_C}}+$/{mode}m.exec('A\n\u200c\u200d\n');a.index===2&&a[0]==='\u200c\u200d'&&/\p{{NChar}}{{2}}$/{mode}m.exec('A\uFFFF\u{{10FFFF}}\n')[0]==='\uFFFF\u{{10FFFF}}'"
        ));
    }
}

#[test]
fn unicode_repeated_atom_multiline_end_consumers_and_empty_advance() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let a=[...'A\r\nB'.matchAll(/(?<x>a{{0}}$)/d{mode}mg)];a.length===3&&a[0].index===1&&a[1].index===2&&a[2].index===4&&a.every(m=>m.groups.x===''&&m.indices.groups.x===m.indices[1])"
        ));
        check(&format!(
            r"'aa\nbb'.replace(/(?<x>.+?$)/{mode}mg,'<$1:$<x>>')==='<aa:aa>\n<bb:bb>'"
        ));
        check(&format!(
            r"'Baaa\nC'.split(/(a+$)/{mode}m).join('|')==='B|aaa|\nC'&&'Baaa\nC'.search(/a+$/{mode}m)===1&&'aa\naaaa'.match(/a+$/g{mode}m).join(',')==='aa,aaaa'"
        ));
        check(&format!(
            r"let t='',a='B😀😀\n'.replace(/(?<x>😀+?$)/{mode}mg,(m,c,p,text,g)=>{{t+=p;return '['+g.x+']'}});a==='B[😀😀]\n'&&t==='1'"
        ));
    }
}

#[test]
fn unicode_repeated_atom_multiline_end_clones_coercions_and_strict_writes() {
    check(
        r"let r=/^(?<x>a+)$/diumg,c=new RegExp(r,'dvmy'),b=new RegExp(c,'divy');c.multiline&&!c.ignoreCase&&c.exec('AA\n')===null&&b.exec('AA\n')===null&&(c.lastIndex=0,c.exec('aa\n').groups.x==='aa')&&c.lastIndex===2",
    );
    check(
        r"let t='',r=/^😀+$/umy;r.lastIndex={valueOf(){t+='i';return 1}};let m=r.exec({toString(){t+='s';return '😀😀\n'}});t==='si'&&m.index===0&&m[0]==='😀😀'&&r.lastIndex===4",
    );
    check(
        r"let r=/a*$/vmy;Object.defineProperty(r,'lastIndex',{writable:false});let caught=false;try{r.exec('')}catch(e){caught=e instanceof TypeError}caught&&r.lastIndex===0",
    );
    check(
        r"let r=/a+$/vmy;r.lastIndex=1;Object.defineProperty(r,'lastIndex',{writable:false});let caught=false;try{r.exec('BBB\n')}catch(e){caught=e instanceof TypeError}caught&&r.lastIndex===1",
    );
}

#[test]
fn unicode_repeated_atom_multiline_ends_collection_and_inputs_are_unlimited() {
    assert_eq!(Limits::default().max_steps, None);
    assert_eq!(Limits::default().max_heap_entries, None);
    let mut realm = Realm::default();
    realm.eval(r"let p='('.repeat(3000)+'😀+'+')'.repeat(3000)+'$',r=new RegExp(p,'dvmg'),c=new RegExp(r),text='B'+'😀'.repeat(100000)+'\nZ'").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("let m=c.exec(text);m.index===1&&m.length===3001&&m[1]===text.slice(1,200001)&&m[3000]===text.slice(1,200001)&&m.indices[3000][0]===1&&m.indices[3000][1]===200001&&c.lastIndex===200001"),Ok(Value::Boolean(true)));
    realm.eval("p=null;r=null;c=null;text=null;m=null").unwrap();
    realm.collect(usize::MAX).unwrap();
}

#[test]
fn unicode_repeated_atom_multiline_ends_opted_work_and_remaining_compositions() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval(r"let r=/\P{NChar}+$/umg,marker=0,text='😀'.repeat(10000);r.lastIndex=1")
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
        r"/a+b$/um.exec('ab')",
        r"/^a+b$/vm.exec('ab')",
        r"/(a)+b$/um.exec('a')",
        r"/a+$$/um.exec('a')",
        r"/\ba+b$/um.exec('a')",
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
fn unicode_repeated_atom_multiline_end_former_whole_programs_snapshot() {
    let mut rows = String::new();
    for source in [r"/a+$/um.exec('a')", r"/^a+$/vm.exec('a')"] {
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
