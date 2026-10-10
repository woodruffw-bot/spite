//! Boundary assertions outside complete single Unicode atom captures.
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
fn unicode_atom_outer_assertion_results_snapshot() {
    let mut rows = String::new();
    for body in [
        r"^(?<x>.)$",
        r"^((?:\w))$",
        r"^([😀])$",
        r"^(?<x>\p{Any})$",
        r"^([\P{ASCII}K])$",
        r"((.))$",
        r"^(?<x>[\p{ASCII}])",
        r"^((\u{10400}))$",
        r"^(\$)$",
    ] {
        let pattern = JsString::from(body);
        for flags in ["dug", "diumg", "dvy", "divmy"] {
            for input in ["A\nſ\nK\n😀\n$", "😀", "\u{10428}", ""] {
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
fn unicode_atom_outer_assertion_original_named_captures_context_and_widths() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let r=/^(?<x>(.))$/dm{mode}g,m=r.exec('xx\r\n😀\u2028A\u2029');m.index===4&&m[1]==='😀'&&m[2]==='😀'&&m.groups.x==='😀'&&m.indices.groups.x===m.indices[1]&&m.indices[2][1]===6&&r.lastIndex===6"
        ));
        check(&format!(
            r"let r=/^(?<__proto__>(\p{{Any}}))$/d{mode}y;r.lastIndex=1;let m=r.exec('😀');m.index===0&&m[1]==='😀'&&m[2]==='😀'&&m.indices[2][1]===2&&Object.getPrototypeOf(m.groups)===null&&m.indices.groups.__proto__===m.indices[1]&&r.lastIndex===2"
        ));
        check(&format!(
            r"let r=/^((\w))$/dim{mode}g,m=r.exec('xx\nſ\n');m.index===3&&m[1]==='ſ'&&m[2]==='ſ'&&m.indices[1][1]===4&&!/^((\w))$/{mode}.test('ſ')"
        ));
        check(&format!(
            r"let r=/^(\uD800)$/d{mode},m=r.exec('\uD800');m[1]==='\uD800'&&m.indices[1][1]===1&&!r.test('\uD800\uDC00')"
        ));
    }
}

#[test]
fn unicode_atom_outer_assertion_dollar_escape_parity_and_dot_all() {
    for mode in ["u", "v"] {
        check(&format!(
            r"/^(\$)$/{mode}.exec('$')[1]==='$'&&!/^(\$)$/{mode}.test('$x')&&/^(\\)$/{mode}.exec('\\')[1]==='\\'"
        ));
        check(&format!(
            r"/^(.)$/s{mode}.exec('\n')[1]==='\n'&&!/^(.)$/{mode}.test('\n')&&!/^(.)$/{mode}.test('A\n')"
        ));
        check(&format!(
            r"/(.)$/{mode}.exec('xA')[1]==='A'&&/^([A])/{mode}.exec('Ax')[1]==='A'"
        ));
        check(&format!(
            r"let r=/^([\P{{ASCII}}K])$/im{mode};r.exec('x\nK')[1]==='K'"
        ));
    }
}

#[test]
fn unicode_atom_outer_assertion_consumers_original_substrings_and_indices() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let a=[...'A\n😀\n\uD800'.matchAll(/^(?<x>.)$/dm{mode}g)];a.length===3&&a[0][1]==='A'&&a[1][1]==='😀'&&a[1].index===2&&a[1].indices[1][1]===4&&a[2][1]==='\uD800'"
        ));
        check(&format!(
            r"'A\n😀'.replace(/^(?<x>.)$/m{mode}g,'<$1:$<x>>')==='<A:A>\n<😀:😀>'"
        ));
        check(&format!(
            r"'A\n😀'.split(/^(.)$/m{mode}).join('|')==='|A|\n|😀|'"
        ));
        check(&format!(
            r"let r=/^(.)$/m{mode};r.lastIndex=99;'xx\n😀'.search(r)===3&&r.lastIndex===99"
        ));
    }
}

#[test]
fn unicode_atom_outer_assertion_metadata_mode_clones_coercions_and_writes() {
    check(
        r"let r=/^(?<x>\p{Any})$/dmu,c=new RegExp(r,'dvy');c.source==='^(?<x>\\p{Any})$'&&!c.multiline&&c.unicodeSets&&!c.unicode&&c.exec('😀').groups.x==='😀'&&c.lastIndex===2",
    );
    check(
        r"let t='',r=/^(.)$/uy;r.lastIndex={valueOf(){t+='i';return 0}};let m=r.exec({toString(){t+='s';return '😀'}});t==='si'&&m[1]==='😀'&&r.lastIndex===2",
    );
    check(
        r"let r=/^(.)$/uy;Object.defineProperty(r,'lastIndex',{writable:false});let caught=false;try{r.exec('😀')}catch(e){caught=e instanceof TypeError}caught&&r.lastIndex===0",
    );
    check(
        r"let r=/^([A])$/vy;r.lastIndex=1;Object.defineProperty(r,'lastIndex',{writable:false});let caught=false;try{r.exec('xA')}catch(e){caught=e instanceof TypeError}caught&&r.lastIndex===1",
    );
}

#[test]
fn unicode_atom_outer_assertion_many_wrappers_collection_and_defaults() {
    assert_eq!(Limits::default().max_steps, None);
    assert_eq!(Limits::default().max_heap_entries, None);
    let mut realm = Realm::default();
    realm.eval("let p='^'+'('.repeat(3000)+'\\\\p{Any}'+')'.repeat(3000)+'$',r=new RegExp(p,'dmvg'),c=new RegExp(r),text='😀'.repeat(100000)+'\\n😀'").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("let m=c.exec(text);m.length===3001&&m[1]==='😀'&&m[3000]==='😀'&&m.indices[3000][0]===200001&&m.indices[3000][1]===200003&&c.lastIndex===200003"),Ok(Value::Boolean(true)));
    realm.eval("p=null;r=null;c=null;text=null;m=null").unwrap();
    realm.collect(usize::MAX).unwrap();
}

#[test]
fn unicode_atom_outer_assertion_opted_work_and_remaining_bodies() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval("let r=/^([A])$/mug,marker=0,text='😀'.repeat(10000);r.lastIndex=1")
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
        r"/^((.)+)$/u.exec('A')",
        r"/^(.)(.)$/v.exec('AA')",
        r"/^(\p{Assigned})$/u.exec('A')",
        r"/^(a|b)$/v.exec('a')",
        r"/^(^.$)$/u.exec('A')",
        r"/^([\q{a\)b}])$/v.exec('a)b')",
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
fn original_unicode_atom_outer_assertion_complete_programs() {
    let mut rows = String::new();
    for source in [
        r"/^([a])$/u.test('a')",
        r"/^([a])$/v.test('a')",
        r"/^([a])$/iu.exec('a')",
        r"/^(\p{ASCII})$/u.exec('A')",
        r"/^(.)$/v.exec('a')",
        r"/^([a])$/v.exec('a')",
        r"/^([a])$/u.exec('a')",
    ] {
        let program = format!("JSON.stringify(eval({:?}))", JsString::from(source));
        let Value::String(result) = Realm::default().eval(&program).unwrap() else {
            panic!("{program}")
        };
        writeln!(rows, "{:?} {result:?}", JsString::from(source)).unwrap();
    }
    insta::assert_snapshot!(rows);
}
