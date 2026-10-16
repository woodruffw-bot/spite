//! Deprecated and its exact Dep alias over complete Unicode characters.
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
fn unicode_deprecated_property_results_snapshot() {
    let mut rows = String::new();
    for body in [
        r"\p{Deprecated}",
        r"\P{Dep}",
        r"[^\p{Dep}]",
        r"[\p{Dep}K]",
        r"[\P{Deprecated}K]",
        r"^(?<x>[\p{Dep}K])$",
        r"(?<x>\P{Deprecated})",
    ] {
        let pattern = JsString::from(body);
        for flags in ["dug", "diug", "divg", "divmy"] {
            for input in [
                "😀\u{148}\u{149}\u{14a}\u{672}\u{673}\u{674}\u{f76}\u{f77}\u{f78}\u{f79}\u{f7a}\u{17a2}\u{17a3}\u{17a4}\u{17a5}\u{17a3}\u{206a}\u{206f}\u{2070}\u{2328}\u{2329}\u{232a}\u{232b}\u{e0000}\u{e0001}\u{e0002}\u{feff}ſK\n",
                "X\r\n\u{673}\u{2028}\u{17a3}\u{2029}",
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
fn unicode_deprecated_properties_aliases_complements_and_unions() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let a=/\p{{Deprecated}}/{mode},b=/\p{{Dep}}/i{mode},n=/\P{{Dep}}/i{mode};a.test('\u0673')&&a.test('\u17A3')&&b.test('\u0673')&&!a.test('\u200D')&&!b.test('\u005f')&&!b.test('ſ')&&!b.test('K')&&!n.test('\u0673')&&n.test('😀')&&n.test('\uD800')&&n.test('\uFEFF')"
        ));
        check(&format!(
            r"let p=/\p{{Deprecated}}/{mode},s=/\s/{mode};p.test('\u0673')&&p.test('\u17A3')&&!s.test('\u0673')&&!s.test('\u17A3')&&!p.test('\uFEFF')&&s.test('\uFEFF')"
        ));
        check(&format!(
            r"let a=/[\p{{Dep}}K]/i{mode},b=/[^\P{{Deprecated}}K]/i{mode},c=/[\p{{Dep}}\P{{Deprecated}}]/{mode};a.test('K')&&a.test('k')&&a.test('\u0673')&&!a.test('ſ')&&b.test('\u17A3')&&!b.test('K')&&c.test('😀')&&c.test('A')&&c.test('\uD800')"
        ));
        check(&format!(
            r"let a=/^\p{{Dep}}$/{mode},b=/^\P{{Dep}}$/{mode};a.test('\u0673')&&!a.test('\u17A3\n')&&!a.test('')&&!a.test('\n')&&b.test('😀')&&!b.test('AA')"
        ));
    }
}

#[test]
fn unicode_deprecated_properties_captures_context_and_pair_starts() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let r=/(?<x>\p{{Dep}})/d{mode}y;r.lastIndex=1;let m=r.exec('\uDB40\uDC01');m.index===0&&m[0]==='\uDB40\uDC01'&&m.groups.x===m[0]&&m.indices[1][0]===0&&m.indices[1][1]===2&&r.lastIndex===2"
        ));
    }

    for mode in ["u", "v"] {
        check(&format!(
            r"let r=/^(?<x>([\p{{Dep}}K]))$/dim{mode}g,m=r.exec('x\r\nK\u2028');m.index===3&&m[1]==='K'&&m[2]==='K'&&m.indices.groups.x===m.indices[1]&&m.indices[2][1]===4&&r.lastIndex===4"
        ));
        check(&format!(
            r"let r=/(?<__proto__>\P{{Dep}})/d{mode}y;r.lastIndex=1;let m=r.exec('😀\u0673');m.index===0&&m[1]==='😀'&&m.indices[1][1]===2&&r.lastIndex===2&&Object.getPrototypeOf(m.groups)===null&&m.indices.groups.__proto__===m.indices[1]"
        ));
        check(&format!(
            r"let r=/\p{{Dep}}/d{mode}g;r.lastIndex=1;let m=r.exec('😀\u0673');m.index===2&&m[0]==='\u0673'&&r.lastIndex===3"
        ));
        check(&format!(
            r"let r=/\p{{Dep}}/{mode}y;r.lastIndex=1;r.exec('😀\u0673')===null&&r.lastIndex===0"
        ));
    }
}

#[test]
fn unicode_deprecated_properties_consumers_original_text() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let a=[...'\u17A3😀\u0673G'.matchAll(/(?<x>\P{{Dep}})/d{mode}g)];a.length===2&&a[0][1]==='😀'&&a[0].index===1&&a[0].indices[1][1]===3&&a[1][1]==='G'"
        ));
        check(&format!(
            r"'\u17A3\u0673😀X'.replace(/(?<x>\p{{Deprecated}})/{mode}g,'<$1:$<x>>')==='<\u17A3:\u17A3><\u0673:\u0673>😀X'"
        ));
        check(&format!(
            r"'😀\u17A3\u0673G'.split(/(\p{{Dep}})/{mode}).join('|')==='😀|\u17A3||\u0673|G'&&'😀\u0673'.search(/\p{{Dep}}/{mode})===2&&'\u17A3\u0673😀'.match(/\p{{Dep}}/{mode}g).join('')==='\u17A3\u0673'"
        ));
        check(&format!(
            r"let t='';let x='\u0673\u17A3😀\u0673'.replace(/(\p{{Dep}})/{mode}g,(m,c,p)=>{{t+=p;return '_'}});x==='__😀_'&&t==='014'"
        ));
    }
}

#[test]
fn unicode_deprecated_properties_clones_coercions_and_strict_writes() {
    check(
        r"let r=/^(?<x>\p{Deprecated})$/dmu,c=new RegExp(r,'divy');c.source==='^(?<x>\\p{Deprecated})$'&&c.unicodeSets&&!c.unicode&&!c.multiline&&c.exec('\u0673').groups.x==='\u0673'&&c.lastIndex===1",
    );
    check(
        r"let t='',r=/\P{Dep}/uy;r.lastIndex={valueOf(){t+='i';return 1}};let m=r.exec({toString(){t+='s';return '😀'}});t==='si'&&m[0]==='😀'&&m.index===0&&r.lastIndex===2",
    );
    check(
        r"let r=/\p{Dep}/vy;Object.defineProperty(r,'lastIndex',{writable:false});let caught=false;try{r.exec('\u0673')}catch(e){caught=e instanceof TypeError}caught&&r.lastIndex===0",
    );
    check(
        r"let r=/\p{Dep}/vy;r.lastIndex=1;Object.defineProperty(r,'lastIndex',{writable:false});let caught=false;try{r.exec('😀\u0673')}catch(e){caught=e instanceof TypeError}caught&&r.lastIndex===1",
    );
}

#[test]
fn unicode_deprecated_properties_repeated_sources_captures_and_collection() {
    assert_eq!(Limits::default().max_steps, None);
    assert_eq!(Limits::default().max_heap_entries, None);
    let mut realm = Realm::default();
    realm.eval("let p='^'+'('.repeat(3000)+'['+'\\\\p{Dep}'.repeat(10000)+']'+')'.repeat(3000)+'$',r=new RegExp(p,'dmvg'),c=new RegExp(r),text='😀'.repeat(100000)+'\\n\\u0673'").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("let m=c.exec(text);m.length===3001&&m[1]==='\\u0673'&&m[3000]==='\\u0673'&&m.indices[3000][0]===200001&&m.indices[3000][1]===200002&&c.lastIndex===200002"),Ok(Value::Boolean(true)));
    realm.eval("p=null;r=null;c=null;text=null;m=null").unwrap();
    realm.collect(usize::MAX).unwrap();
}

#[test]
fn unicode_deprecated_properties_opted_work_and_remaining_sets() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval("let r=/\\p{Dep}/ug,marker=0,text='😀'.repeat(10000);r.lastIndex=1")
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
        r"/\p{Script_Extensions=Latin}/u.exec('A')",
        r"/[\p{Dep}&&A]/v.exec('A')",
        r"/[\p{Dep}--A]/v.exec('A')",
        r"/\p{Dep}+b/u.exec('AA')",
        r"/\p{Dep}A/v.exec('AA')",
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
fn unicode_deprecated_repetitions_retain_last_capture_and_outside_words() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let r=/\B(?<x>\p{{Dep}})+\B/d{mode}g,m=r.exec('😀\u0673\u17A3\u17A3\uFEFF');m.index===2&&m[0]==='\u0673\u17A3\u17A3'&&m.groups.x==='\u17A3'&&m.indices.groups.x[0]===4&&m.indices.groups.x[1]===5&&r.lastIndex===5"
        ));
    }
}
