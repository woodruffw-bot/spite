//! Logical_Order_Exception and its exact LOE alias over complete Unicode characters.
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
fn unicode_logical_order_exception_property_results_snapshot() {
    let mut rows = String::new();
    for body in [
        r"\p{Logical_Order_Exception}",
        r"\P{LOE}",
        r"[^\p{LOE}]",
        r"[\p{LOE}K]",
        r"[\P{Logical_Order_Exception}K]",
        r"^(?<x>[\p{LOE}K])$",
        r"(?<x>\P{Logical_Order_Exception})",
    ] {
        let pattern = JsString::from(body);
        for flags in ["dug", "diug", "divg", "divmy"] {
            for input in [
                "😀\u{e40}\u{e41}\u{e42}\u{e43}\u{e44}\u{ec0}\u{ec1}\u{ec2}\u{ec3}\u{ec4}\u{19b5}\u{19b6}\u{19b7}\u{19ba}\u{aab5}\u{aab6}\u{aab9}\u{aabb}\u{aabc}\u{e3f}\u{e45}\u{ebf}\u{ec5}\u{19b4}\u{19b8}\u{19b9}\u{19bb}\u{aab4}\u{aab7}\u{aab8}\u{aaba}\u{aabd}\u{feff}ſK\n",
                "X\r\n\u{e40}\u{2028}\u{ec0}\u{2029}",
                "\u{ff10}\u{ff26}",
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
fn unicode_logical_order_exception_properties_aliases_complements_and_unions() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let a=/\p{{Logical_Order_Exception}}/{mode},b=/\p{{LOE}}/i{mode},n=/\P{{LOE}}/i{mode};a.test('\u0E40')&&a.test('\u0E41')&&b.test('\u0E40')&&!a.test('\u200D')&&!b.test('\u2010')&&!b.test('ſ')&&!b.test('K')&&!n.test('\u0E40')&&n.test('😀')&&n.test('\uD800')&&n.test('\uFEFF')"
        ));
        check(&format!(
            r"let p=/\p{{Logical_Order_Exception}}/{mode},s=/\s/{mode};p.test('\u0E40')&&p.test('\u0E41')&&!s.test('\u0E40')&&!s.test('\u0E41')&&!p.test('\uFEFF')&&s.test('\uFEFF')"
        ));
        check(&format!(
            r"let a=/[\p{{LOE}}K]/i{mode},b=/[^\P{{Logical_Order_Exception}}K]/i{mode},c=/[\p{{LOE}}\P{{Logical_Order_Exception}}]/{mode};a.test('K')&&a.test('k')&&a.test('\u0E40')&&!a.test('ſ')&&b.test('\u0E41')&&!b.test('K')&&c.test('😀')&&c.test('A')&&c.test('\uD800')"
        ));
        check(&format!(
            r"let a=/^\p{{LOE}}$/{mode},b=/^\P{{LOE}}$/{mode};a.test('\u0E40')&&!a.test('\u0E41\n')&&!a.test('')&&!a.test('\n')&&b.test('😀')&&!b.test('AA')"
        ));
    }
}

#[test]
fn unicode_logical_order_exception_properties_captures_context_and_pair_starts() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let r=/^(?<x>([\p{{LOE}}K]))$/dim{mode}g,m=r.exec('x\r\nK\u2028');m.index===3&&m[1]==='K'&&m[2]==='K'&&m.indices.groups.x===m.indices[1]&&m.indices[2][1]===4&&r.lastIndex===4"
        ));
        check(&format!(
            r"let r=/(?<__proto__>\P{{LOE}})/d{mode}y;r.lastIndex=1;let m=r.exec('😀\u0e40');m.index===0&&m[1]==='😀'&&m.indices[1][1]===2&&r.lastIndex===2&&Object.getPrototypeOf(m.groups)===null&&m.indices.groups.__proto__===m.indices[1]"
        ));
        check(&format!(
            r"let r=/\p{{LOE}}/d{mode}g;r.lastIndex=1;let m=r.exec('😀\u0e40');m.index===2&&m[0]==='\u0e40'&&r.lastIndex===3"
        ));
        check(&format!(
            r"let r=/\p{{LOE}}/{mode}y;r.lastIndex=1;r.exec('😀\u0e40')===null&&r.lastIndex===0"
        ));
    }
}

#[test]
fn unicode_logical_order_exception_properties_consumers_original_text() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let a=[...'\u0e41😀\u0e40G'.matchAll(/(?<x>\P{{LOE}})/d{mode}g)];a.length===2&&a[0][1]==='😀'&&a[0].index===1&&a[0].indices[1][1]===3&&a[1][1]==='G'"
        ));
        check(&format!(
            r"'\u0e41\u0e40😀X'.replace(/(?<x>\p{{Logical_Order_Exception}})/{mode}g,'<$1:$<x>>')==='<\u0e41:\u0e41><\u0e40:\u0e40>😀X'"
        ));
        check(&format!(
            r"'😀\u0e41\u0e40G'.split(/(\p{{LOE}})/{mode}).join('|')==='😀|\u0e41||\u0e40|G'&&'😀\u0e40'.search(/\p{{LOE}}/{mode})===2&&'\u0e41\u0e40😀'.match(/\p{{LOE}}/{mode}g).join('')==='\u0e41\u0e40'"
        ));
        check(&format!(
            r"let t='';let x='\u0e40\u0e41😀\u0e40'.replace(/(\p{{LOE}})/{mode}g,(m,c,p)=>{{t+=p;return '_'}});x==='__😀_'&&t==='014'"
        ));
    }
}

#[test]
fn unicode_logical_order_exception_properties_clones_coercions_and_strict_writes() {
    check(
        r"let r=/^(?<x>\p{Logical_Order_Exception})$/dmu,c=new RegExp(r,'divy');c.source==='^(?<x>\\p{Logical_Order_Exception})$'&&c.unicodeSets&&!c.unicode&&!c.multiline&&c.exec('\u0e40').groups.x==='\u0e40'&&c.lastIndex===1",
    );
    check(
        r"let t='',r=/\P{LOE}/uy;r.lastIndex={valueOf(){t+='i';return 1}};let m=r.exec({toString(){t+='s';return '😀'}});t==='si'&&m[0]==='😀'&&m.index===0&&r.lastIndex===2",
    );
    check(
        r"let r=/\p{LOE}/vy;Object.defineProperty(r,'lastIndex',{writable:false});let caught=false;try{r.exec('\u0e40')}catch(e){caught=e instanceof TypeError}caught&&r.lastIndex===0",
    );
    check(
        r"let r=/\p{LOE}/vy;r.lastIndex=1;Object.defineProperty(r,'lastIndex',{writable:false});let caught=false;try{r.exec('😀\u0e40')}catch(e){caught=e instanceof TypeError}caught&&r.lastIndex===1",
    );
}

#[test]
fn unicode_logical_order_exception_properties_repeated_sources_captures_and_collection() {
    assert_eq!(Limits::default().max_steps, None);
    assert_eq!(Limits::default().max_heap_entries, None);
    let mut realm = Realm::default();
    realm.eval("let p='^'+'('.repeat(3000)+'['+'\\\\p{LOE}'.repeat(10000)+']'+')'.repeat(3000)+'$',r=new RegExp(p,'dmvg'),c=new RegExp(r),text='😀'.repeat(100000)+'\\n\\u0e40'").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("let m=c.exec(text);m.length===3001&&m[1]==='\\u0e40'&&m[3000]==='\\u0e40'&&m.indices[3000][0]===200001&&m.indices[3000][1]===200002&&c.lastIndex===200002"),Ok(Value::Boolean(true)));
    realm.eval("p=null;r=null;c=null;text=null;m=null").unwrap();
    realm.collect(usize::MAX).unwrap();
}

#[test]
fn unicode_logical_order_exception_properties_opted_work_and_remaining_sets() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval("let r=/\\p{LOE}/ug,marker=0,text='😀'.repeat(10000);r.lastIndex=1")
        .unwrap();
    assert!(matches!(
        realm.eval("try{r.exec(text)}catch{marker=1}finally{marker=2}"),
        Err(Error::Limit { .. })
    ));
    assert_eq!(
        realm.eval("marker===0&&r.lastIndex===1"),
        Ok(Value::Boolean(true))
    );
    check(
        r#"let m=/\p{Script_Extensions=Latin}/u.exec('A');JSON.stringify(m)==='["A"]'&&m.index===0&&m.input==='A'"#,
    );
    for source in [
        r"/[\p{LOE}&&A]/v.exec('A')",
        r"/[\p{LOE}--A]/v.exec('A')",
        r"/\p{LOE}+b/u.exec('AA')",
        r"/\p{LOE}A/v.exec('AA')",
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
fn unicode_logical_order_exception_repetitions_retain_last_capture_and_outside_words() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let r=/\B(?<x>\p{{LOE}})+\B/d{mode}g,m=r.exec('😀\u0e40\u0e41\u0ec0!');m.index===2&&m[0]==='\u0e40\u0e41\u0ec0'&&m.groups.x==='\u0ec0'&&m.indices.groups.x[0]===4&&m.indices.groups.x[1]===5&&r.lastIndex===5"
        ));
    }
}
