//! Leading line assertions on a quantified complete Unicode atom.
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
fn unicode_repeated_atom_start_former_whole_program_snapshot() {
    let source = r"/^a+/u.exec('a')";
    let Value::String(result) = Realm::default()
        .eval(&format!("JSON.stringify({source})"))
        .unwrap()
    else {
        panic!("{source}")
    };
    insta::assert_snapshot!(format!("{source} {result:?}"));
}

#[test]
fn unicode_repeated_atom_start_results_snapshot() {
    let mut rows = String::new();
    for body in [
        r"^(?<x>a+)",
        r"^(?<x>a*?)",
        r"^(?<x>a??)",
        r"^(?<x>a{2,3})",
        r"^(?<x>.+)",
        r"^(?<x>\D{1,2})",
        r"^(?<x>\W+)",
        r"^(?<x>\p{Join_C}+)",
        r"^(?<x>\p{NChar}{1,2})",
        r"^(?<x>[aſK]+)",
    ] {
        let pattern = JsString::from(body);
        for flags in ["dug", "dumg", "diumg", "dimvy", "dsumg", "dimsvy"] {
            for input in [
                JsString::from("B\raaa\nAAſK\u{2028}"),
                JsString::from("\r\n\u{200c}\u{200d}\u{ffff}\u{10ffff}"),
                JsString::from("😀\naaa"),
                JsString::from_code_units(vec![0x61, 0xd800, 0x0a, 0x61, 0xdc00]),
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
fn unicode_repeated_atom_starts_counts_folding_and_line_context() {
    for mode in ["u", "v"] {
        check(&format!(
            r"/^a+/{mode}.exec('Ba')===null&&/^a+/m{mode}.exec('B\naaaa')[0]==='aaaa'&&/^a+?/m{mode}.exec('B\naaaa')[0]==='a'"
        ));
        check(&format!(
            r"/^a{{2,3}}/m{mode}.exec('aaB\naaaa')[0]==='aa'&&/^a{{2,3}}?/m{mode}.exec('B\naaaa')[0]==='aa'&&/^a*?/m{mode}.exec('B')[0]===''"
        ));
        check(&format!(
            r"['\n','\r','\u2028','\u2029'].every(t=>{{let m=/^a+/m{mode}.exec('B'+t+'aaa');return m.index===2&&m[0]==='aaa'}})"
        ));
        check(&format!(
            r"let r=/^a*/m{mode}y;r.lastIndex=2;let m=r.exec('B\r\nA');m.index===2&&m[0]===''&&r.lastIndex===2"
        ));
        check(&format!(
            r"/^\w+/im{mode}.exec('B\nſK')[0]==='B'&&/^\w+/i{mode}.exec('ſK')[0]==='ſK'&&/^\W+/{mode}.exec('ſK')[0]==='ſK'"
        ));
        check(&format!(
            r"/^\p{{Join_C}}+/m{mode}.exec('A\n\u200c\u200d')[0]==='\u200c\u200d'&&/^\p{{NChar}}{{2}}/m{mode}.exec('A\n\uFFFF\u{{10FFFF}}')[0]==='\uFFFF\u{{10FFFF}}'"
        ));
        check(&format!(
            r"/^.+/m{mode}.exec('😀\nA')[0]==='😀'&&/^.+/ms{mode}.exec('😀\nA')[0]==='😀\nA'&&/^.+?/ms{mode}.exec('😀\nA')[0]==='😀'"
        ));
    }
}

#[test]
fn unicode_repeated_atom_starts_complete_captures_and_initial_pairs() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let r=/^(?<__proto__>(😀+))/dm{mode}y;r.lastIndex=3;let m=r.exec('A\n😀😀');m.index===2&&m[0]==='😀😀'&&m[2]==='😀😀'&&m.indices[2][0]===2&&m.indices[2][1]===6&&m.indices.groups.__proto__===m.indices[1]&&r.lastIndex===6"
        ));
        check(&format!(
            r"let r=/^(?<x>a*?)/d{mode}y;r.lastIndex=1;let m=r.exec('😀');m.index===0&&m.groups.x===''&&m.indices.groups.x[0]===0&&m.indices.groups.x[1]===0&&r.lastIndex===0"
        ));
        check(&format!(
            r"let r=/^a+/m{mode}y;r.lastIndex=3;r.exec('B\naa')===null&&r.lastIndex===0"
        ));
        check(&format!(
            r"let r=/^\uD800+/m{mode}g,m=r.exec('B\n\uD800\uD800');m.index===2&&m[0]==='\uD800\uD800'&&r.lastIndex===4&&!/^\uD800+/m{mode}.test('😀')"
        ));
        check(&format!(
            r"let r=/^a{{3}}/m{mode}g,m=r.exec('aaB\naaa');m.index===4&&m[0]==='aaa'&&r.lastIndex===7"
        ));
    }
}

#[test]
fn unicode_repeated_atom_start_consumers_and_empty_advance() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let a=[...'B\n😀\n'.matchAll(/^(?<x>a*?)/dm{mode}g)];a.map(m=>m.index).join(',')==='0,2,5'&&a.every(m=>m[0]===''&&m.groups.x===''&&m.indices.groups.x===m.indices[1])"
        ));
        check(&format!(
            r"'AA\naa'.replace(/^(?<x>a+)/im{mode}g,'<$1:$<x>>')==='<AA:AA>\n<aa:aa>'"
        ));
        check(&format!(
            r"'B\naaa\nBB\naa'.split(/^(a+)/m{mode}).join('|')==='B\n|aaa|\nBB\n|aa|'&&'B\naaa'.search(/^a+/m{mode})===2&&'aa\naaaa'.match(/^a+/gm{mode}).join(',')==='aa,aaaa'"
        ));
        check(&format!(
            r"let t='',a='A\n😀\nB'.replace(/^(?<x>.+?)/ms{mode}g,(m,c,p,text,g)=>{{t+=p;return '['+g.x+']'}});a==='[A]\n[😀]\n[B]'&&t==='025'"
        ));
    }
}

#[test]
fn unicode_repeated_atom_start_clones_coercions_and_strict_writes() {
    check(
        r"let r=/^(?<x>a+)/dimug,c=new RegExp(r,'dvy'),b=new RegExp(c,'dimuy');c.unicodeSets&&!c.ignoreCase&&c.exec('B\nAA')===null&&b.exec('AA').groups.x==='AA'&&b.lastIndex===2",
    );
    check(
        r"let t='',r=/^😀+/muy;r.lastIndex={valueOf(){t+='i';return 3}};let m=r.exec({toString(){t+='s';return 'A\n😀😀'}});t==='si'&&m.index===2&&m[0]==='😀😀'&&r.lastIndex===6",
    );
    check(
        r"let r=/^a*/mvy;Object.defineProperty(r,'lastIndex',{writable:false});let caught=false;try{r.exec('')}catch(e){caught=e instanceof TypeError}caught&&r.lastIndex===0",
    );
    check(
        r"let r=/^a+/mvy;r.lastIndex=1;Object.defineProperty(r,'lastIndex',{writable:false});let caught=false;try{r.exec('BBB')}catch(e){caught=e instanceof TypeError}caught&&r.lastIndex===1",
    );
}

#[test]
fn unicode_repeated_atom_starts_captures_collection_and_inputs_are_unlimited() {
    assert_eq!(Limits::default().max_steps, None);
    assert_eq!(Limits::default().max_heap_entries, None);
    let mut realm = Realm::default();
    realm.eval(r"let p='^'+'('.repeat(3000)+'😀+'+')'.repeat(3000),r=new RegExp(p,'dmvg'),c=new RegExp(r),text='B\n'+'😀'.repeat(100000)").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("let m=c.exec(text);m.index===2&&m.length===3001&&m[1]===text.slice(2)&&m[3000]===text.slice(2)&&m.indices[3000][0]===2&&m.indices[3000][1]===200002&&c.lastIndex===200002"),Ok(Value::Boolean(true)));
    realm.eval("p=null;r=null;c=null;text=null;m=null").unwrap();
    realm.collect(usize::MAX).unwrap();
}

#[test]
fn unicode_repeated_atom_starts_opted_work_and_remaining_compositions() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval(r"let r=/^\P{NChar}+/umg,marker=0,text='😀'.repeat(10000);r.lastIndex=1")
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
        r"/^^a+/u.exec('a')",
        r"/^a+b$/u.exec('a')",
        r"/^a+b/v.exec('ab')",
        r"/^\ba+b/u.exec('a')",
        r"/^(a)+b/u.exec('a')",
        r"/^[a--b]+/v.exec('a')",
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
