//! Single Unicode atom quantifiers and complete enclosing original captures.
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
fn unicode_repeated_atom_results_snapshot() {
    let mut rows = String::new();
    for body in [
        r"(?<x>a+)",
        r"(?<x>a*?)",
        r"(?<x>a??)",
        r"(?<x>a{2,3})",
        r"(?<x>a{002,03}?)",
        r"(?<x>.+)",
        r"(?<x>\D{1,2})",
        r"(?<x>\W+)",
        r"(?<x>\p{Join_C}+)",
        r"(?<x>\p{NChar}{1,2})",
        r"(?<x>\u{61}?)",
        r"(?<x>\??)",
        r"(?<x>a{0,0})",
        r"(?<x>[aſK]+)",
    ] {
        let pattern = JsString::from(body);
        for flags in ["dug", "diug", "divg", "divy", "dsug", "disvy"] {
            for input in [
                JsString::from("aaa AA1 ſK\n"),
                JsString::from("A\r\n\u{200c}\u{200d}\u{ffff}\u{10ffff}"),
                JsString::from("😀??a"),
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
fn unicode_repeated_atoms_greedy_lazy_bounds_escapes_and_properties() {
    for mode in ["u", "v"] {
        check(&format!(
            r"/a+/{mode}.exec('aaaa')[0]==='aaaa'&&/a+?/{mode}.exec('aaaa')[0]==='a'&&/a*?/{mode}.exec('aaaa')[0]===''&&/a??/{mode}.exec('aaaa')[0]===''"
        ));
        check(&format!(
            r"/a{{2,3}}/{mode}.exec('aaaa')[0]==='aaa'&&/a{{002,03}}?/{mode}.exec('aaaa')[0]==='aa'&&/a{{0,0}}/{mode}.exec('aaaa')[0]===''&&/a{{2,}}/{mode}.exec('a')===null"
        ));
        check(&format!(
            r"/\u{{61}}?/{mode}.exec('aa')[0]==='a'&&/\u{{61}}??/{mode}.exec('aa')[0]===''&&/\??/{mode}.exec('??')[0]==='?'&&/\???/{mode}.exec('??')[0]===''&&/\$+/{mode}.exec('$$')[0]==='$$'"
        ));
        check(&format!(
            r"/\w+/i{mode}.exec('ſK')[0]==='ſK'&&/\w+/{mode}.exec('ſK')===null&&/\W+/{mode}.exec('ſK')[0]==='ſK'&&!/\W+/i{mode}.test('ſK')"
        ));
        check(&format!(
            r"/\p{{Join_C}}+/{mode}.exec('\u200c\u200d')[0]==='\u200c\u200d'&&/\p{{NChar}}{{2}}/{mode}.exec('\uFFFF\u{{10FFFF}}')[0]==='\uFFFF\u{{10FFFF}}'"
        ));
        check(&format!(
            r"/.+/{mode}.exec('😀\nA')[0]==='😀'&&/.+/s{mode}.exec('😀\nA')[0]==='😀\nA'&&/.+?/s{mode}.exec('😀\nA')[0]==='😀'"
        ));
    }
}

#[test]
fn unicode_repeated_atoms_original_complete_captures_and_pair_starts() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let r=/(?<__proto__>(😀+))/d{mode}y;r.lastIndex=1;let m=r.exec('😀😀');m.index===0&&m[0]==='😀😀'&&m[1]==='😀😀'&&m[2]==='😀😀'&&m.indices[2][1]===4&&m.indices.groups.__proto__===m.indices[1]&&r.lastIndex===4"
        ));
        check(&format!(
            r"let r=/(?<x>a*?)/d{mode}y;r.lastIndex=1;let m=r.exec('😀');m.index===0&&m.groups.x===''&&m.indices.groups.x[0]===0&&m.indices.groups.x[1]===0&&r.lastIndex===0"
        ));
        check(&format!(
            r"let r=/\uD800+/{mode}g,m=r.exec('\uD800\uD800');m[0]==='\uD800\uD800'&&r.lastIndex===2&&!/\uD800+/{mode}.test('😀')"
        ));
        check(&format!(
            r"let r=/a{{3}}/{mode}g,m=r.exec('aaBaaa');m.index===3&&m[0]==='aaa'&&r.lastIndex===6"
        ));
        check(&format!(
            r"let r=/a{{3}}/{mode}y;r.exec('aaBaaa')===null&&r.lastIndex===0&&/a{{0}}/{mode}.exec('')[0]===''"
        ));
    }
}

#[test]
fn unicode_repeated_atom_consumers_advance_empty_and_complete_matches() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let a=[...'A😀B'.matchAll(/(?<x>a*?)/d{mode}g)];a.map(m=>m.index).join(',')==='0,1,3,4'&&a.every(m=>m[0]===''&&m.groups.x===''&&m.indices.groups.x===m.indices[1])"
        ));
        check(&format!(
            r"'AA aa'.replace(/(?<x>a+)/i{mode}g,'<$1:$<x>>')==='<AA:AA> <aa:aa>'"
        ));
        check(&format!(
            r"'x aa y'.split(/(a+)/{mode}).join('|')==='x |aa| y'&&'x aa y'.search(/a+/{mode})===2&&'aa aaaa'.match(/a+/g{mode}).join(',')==='aa,aaaa'"
        ));
        check(&format!(
            r"let t='',a='😀 a'.replace(/(?<x>.+?)/s{mode}g,(m,c,p,text,g)=>{{t+=p;return '['+g.x+']'}});a==='[😀][ ][a]'&&t==='023'"
        ));
    }
}

#[test]
fn unicode_repeated_atom_clones_coercions_and_strict_writes() {
    check(
        r"let r=/(?<x>a+)/diug,c=new RegExp(r,'dvy'),b=new RegExp(c,'diuy');c.unicodeSets&&!c.ignoreCase&&c.exec('AA')===null&&b.exec('AA').groups.x==='AA'&&b.lastIndex===2",
    );
    check(
        r"let t='',r=/😀+/uy;r.lastIndex={valueOf(){t+='i';return 1}};let m=r.exec({toString(){t+='s';return '😀😀'}});t==='si'&&m.index===0&&m[0]==='😀😀'&&r.lastIndex===4",
    );
    check(
        r"let r=/a*/vy;Object.defineProperty(r,'lastIndex',{writable:false});let caught=false;try{r.exec('')}catch(e){caught=e instanceof TypeError}caught&&r.lastIndex===0",
    );
    check(
        r"let r=/a+/vy;r.lastIndex=1;Object.defineProperty(r,'lastIndex',{writable:false});let caught=false;try{r.exec('BBB')}catch(e){caught=e instanceof TypeError}caught&&r.lastIndex===1",
    );
}

#[test]
fn unicode_repeated_atoms_huge_counts_captures_collection_and_inputs_are_unlimited() {
    assert_eq!(Limits::default().max_steps, None);
    assert_eq!(Limits::default().max_heap_entries, None);
    let mut realm = Realm::default();
    assert_eq!(realm.eval(r"let huge='9'.repeat(200),never=new RegExp('a{'+huge+'}','u'),wide=new RegExp('a{0,'+huge+'}','v'),zeros=new RegExp('a{'+'0'.repeat(100000)+'2}','v');never.exec('aaa')===null&&wide.exec('aaa')[0]==='aaa'&&zeros.exec('aaa')[0]==='aa'"), Ok(Value::Boolean(true)));
    realm.eval(r"let p='('.repeat(3000)+'😀+'+')'.repeat(3000),r=new RegExp(p,'dvg'),c=new RegExp(r),text='😀'.repeat(100000)").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("let m=c.exec(text);m.length===3001&&m[1]===text&&m[3000]===text&&m.indices[3000][0]===0&&m.indices[3000][1]===200000&&c.lastIndex===200000"),Ok(Value::Boolean(true)));
    realm
        .eval("p=null;r=null;c=null;text=null;m=null;never=null;wide=null;zeros=null")
        .unwrap();
    realm.collect(usize::MAX).unwrap();
}

#[test]
fn unicode_repeated_atoms_opted_work_and_remaining_compositions() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval(r"let r=/\P{NChar}+/ug,marker=0,text='😀'.repeat(10000);r.lastIndex=1")
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
        r"/(a)+b/u.exec('a')",
        r"/^^a+/u.exec('a')",
        r"/a+b$/u.exec('a')",
        r"/a+b/v.exec('ab')",
        r"/\ba+/u.exec('a')",
        r"/[a--b]+/v.exec('a')",
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
fn unicode_repeated_atom_former_whole_programs_snapshot() {
    let mut rows = String::new();
    for source in [
        r"/a+/u.test('a')",
        r"/[a]+/v.test('a')",
        r"/\p{AHex}+/u.exec('AA')",
        r"/\p{ASCII}+/v.exec('A')",
        r"/[a]*/u.exec('a')",
        r"/(?:a?)/u.exec('a')",
        r"/[\u{61}]*/u.exec('a')",
        r"/\u{61}+/v.exec('aa')",
        r"/\w+/iv.exec('a')",
        r"/[a]+/iv.exec('a')",
        r"/[\p{ASCII}]+/v.exec('A')",
        r"/\D+/u.exec('a')",
        r"/\p{Hex}+/u.exec('AA')",
        r"/\p{Join_C}+/u.exec('AA')",
        r"/a+/iv.exec('A')",
        r"/\p{NChar}+/u.exec('AA')",
        r"/[a]+/u.exec('a')",
        r"/😀+/u.exec('😀')",
        r"/[a]*/v.exec('a')",
        r"/(?:a?)/v.exec('a')",
        r"/\uDE00+/u.exec('\uDE00')",
        r"/\p{space}+/u.exec('AA')",
    ] {
        let program = format!("JSON.stringify({source})");
        let Value::String(result) = Realm::default().eval(&program).unwrap() else {
            panic!("{source}")
        };
        writeln!(rows, "{source} {result:?}").unwrap();
    }
    insta::assert_snapshot!(rows);
}
