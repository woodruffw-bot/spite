//! Exact Han script properties over complete Unicode characters (22.2.2.9).
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
fn unicode_han_script_former_whole_programs_snapshot() {
    let mut rows = String::new();
    for source in [
        r"/\p{Script=Han}/v.exec('𠮷')",
        r"/[\p{Script=Han}]/v.exec('𠮷')",
    ] {
        assert!(
            matches!(Realm::default().eval(source), Ok(Value::Object(_))),
            "{source}"
        );
        let Value::String(result) = Realm::default()
            .eval(&format!("JSON.stringify({source})"))
            .unwrap()
        else {
            panic!("{source}")
        };
        assert_eq!(result, JsString::from("[\"𠮷\"]"));
        writeln!(rows, "{source:?} {result:?}").unwrap();
    }
    insta::assert_snapshot!(rows);
}

#[test]
fn unicode_han_script_property_results_snapshot() {
    let mut rows = String::new();
    for body in [
        r"\p{Script=Han}",
        r"\P{sc=Hani}",
        r"[^\p{sc=Han}]",
        r"[\p{Script=Hani}K]",
        r"[\P{sc=Han}K]",
        r"^(?<x>[\p{sc=Hani}K])$",
        r"(?<x>\P{Script=Han})",
    ] {
        let pattern = JsString::from(body);
        for flags in ["dug", "diug", "divg", "divmy"] {
            for input in [
                "𠮷a一々〆〇⺀⻳⿕😀ſK\n",
                "X\r\n𠮷\u{2028}一\u{2029}",
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
fn unicode_han_script_aliases_complements_and_unions() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let a=/\p{{Script=Han}}/{mode},b=/\p{{Script=Hani}}/i{mode},c=/\p{{sc=Han}}/{mode},d=/\p{{sc=Hani}}/i{mode},n=/\P{{sc=Han}}/i{mode};a.test('𠮷')&&b.test('一')&&c.test('々')&&d.test('〇')&&!a.test('〆')&&!b.test('ſ')&&!c.test('K')&&!n.test('𠮷')&&n.test('😀')&&n.test('\uD800')&&n.test('A')"
        ));
        check(&format!(
            r"let a=/[\p{{sc=Han}}K]/i{mode},b=/[^\P{{Script=Hani}}K]/i{mode},c=/[\p{{Script=Han}}\P{{sc=Hani}}]/{mode};a.test('K')&&a.test('k')&&a.test('𠮷')&&!a.test('ſ')&&b.test('一')&&!b.test('K')&&c.test('😀')&&c.test('A')&&c.test('\uD800')"
        ));
        check(&format!(
            r"let a=/^\p{{sc=Han}}$/{mode},b=/^\P{{sc=Hani}}$/{mode};a.test('𠮷')&&!a.test('一\n')&&!a.test('')&&!a.test('\n')&&b.test('😀')&&!b.test('AA')"
        ));
    }
}

#[test]
fn unicode_han_script_captures_context_and_pair_starts() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let r=/^(?<x>([\p{{sc=Han}}K]))$/dim{mode}g,m=r.exec('x\r\n𠮷\u2028');m.index===3&&m[1]==='𠮷'&&m[2]==='𠮷'&&m.indices.groups.x===m.indices[1]&&m.indices[2][1]===5&&r.lastIndex===5"
        ));
        check(&format!(
            r"let r=/(?<__proto__>\p{{Script=Han}})/d{mode}y;r.lastIndex=1;let m=r.exec('𠮷a');m.index===0&&m[1]==='𠮷'&&m.indices[1][1]===2&&r.lastIndex===2&&Object.getPrototypeOf(m.groups)===null&&m.indices.groups.__proto__===m.indices[1]"
        ));
        check(&format!(
            r"let r=/\p{{sc=Hani}}/d{mode}g;r.lastIndex=1;let m=r.exec('😀𠮷');m.index===2&&m[0]==='𠮷'&&r.lastIndex===4"
        ));
        check(&format!(
            r"let r=/\p{{sc=Han}}/{mode}y;r.lastIndex=1;r.exec('😀𠮷')===null&&r.lastIndex===0"
        ));
    }
}

#[test]
fn unicode_han_script_consumers_original_text() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let a=[...'一😀𠮷G'.matchAll(/(?<x>\P{{sc=Han}})/d{mode}g)];a.length===2&&a[0][1]==='😀'&&a[0].index===1&&a[0].indices[1][1]===3&&a[1][1]==='G'"
        ));
        check(&format!(
            r"'一𠮷😀X'.replace(/(?<x>\p{{Script=Han}})/{mode}g,'<$1:$<x>>')==='<一:一><𠮷:𠮷>😀X'"
        ));
        check(&format!(
            r"'😀一𠮷G'.split(/(\p{{sc=Han}})/{mode}).join('|')==='😀|一||𠮷|G'&&'😀𠮷'.search(/\p{{Script=Hani}}/{mode})===2&&'一𠮷😀'.match(/\p{{sc=Han}}/{mode}g).join('')==='一𠮷'"
        ));
        check(&format!(
            r"let t='',x='一𠮷😀一'.replace(/(\p{{sc=Han}})/{mode}g,(m,c,p)=>{{t+=p;return '_'}});x==='__😀_'&&t==='015'"
        ));
    }
}

#[test]
fn unicode_han_script_clones_coercions_and_strict_writes() {
    check(
        r"let r=/^(?<x>\p{Script=Han})$/dmu,c=new RegExp(r,'divy');c.source==='^(?<x>\\p{Script=Han})$'&&c.unicodeSets&&!c.unicode&&!c.multiline&&c.exec('𠮷').groups.x==='𠮷'&&c.lastIndex===2",
    );
    check(
        r"let t='',r=/\p{sc=Han}/uy;r.lastIndex={valueOf(){t+='i';return 1}};let m=r.exec({toString(){t+='s';return '𠮷'}});t==='si'&&m[0]==='𠮷'&&m.index===0&&r.lastIndex===2",
    );
    check(
        r"let r=/\p{sc=Hani}/vy;Object.defineProperty(r,'lastIndex',{writable:false});let caught=false;try{r.exec('𠮷')}catch(e){caught=e instanceof TypeError}caught&&r.lastIndex===0",
    );
    check(
        r"let r=/\p{Script=Hani}/vy;r.lastIndex=1;Object.defineProperty(r,'lastIndex',{writable:false});let caught=false;try{r.exec('😀𠮷')}catch(e){caught=e instanceof TypeError}caught&&r.lastIndex===1",
    );
}

#[test]
fn unicode_han_script_repeated_sources_captures_and_collection() {
    assert_eq!(Limits::default().max_steps, None);
    assert_eq!(Limits::default().max_heap_entries, None);
    let mut realm = Realm::default();
    realm.eval("let p='^'+'('.repeat(3000)+'['+'\\\\p{sc=Han}'.repeat(10000)+']'+')'.repeat(3000)+'$',r=new RegExp(p,'dmvg'),c=new RegExp(r),text='😀'.repeat(100000)+'\\n𠮷'").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("let m=c.exec(text);m.length===3001&&m[1]==='𠮷'&&m[3000]==='𠮷'&&m.indices[3000][0]===200001&&m.indices[3000][1]===200003&&c.lastIndex===200003"),Ok(Value::Boolean(true)));
    realm.eval("p=null;r=null;c=null;text=null;m=null").unwrap();
    realm.collect(usize::MAX).unwrap();
}

#[test]
fn unicode_han_script_opted_work_and_remaining_sets() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval("let r=/\\p{sc=Han}/ug,marker=0,text='😀'.repeat(10000);r.lastIndex=1")
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
    check(
        r#"let m=/\p{scx=Han}/u.exec('一');JSON.stringify(m)==='["一"]'&&m.index===0&&m.input==='一'"#,
    );
    check(r"/[\p{sc=Han}&&A]/v.exec('A')===null");
    check(r"/[\p{sc=Han}--A]/v.exec('A')===null");
    check(
        r"let s=String.fromCodePoint(0x20bb7),m=/[\p{sc=Han}--A]/v.exec(s);m!==null&&m.length===1&&m[0]===s&&m.index===0&&m.input===s",
    );
    for source in [
        r"/\p{sc=Han}+b/u.exec('一一')",
        r"/\p{sc=Han}A/v.exec('一A')",
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
fn unicode_han_script_repetition_captures_and_unicode18_member() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let r=/\B(?<x>\p{{sc=Han}})+\B/d{mode}g,m=r.exec('😀一𠮷〇!');m.index===2&&m[0]==='一𠮷〇'&&m.groups.x==='〇'&&m.indices.groups.x[0]===5&&m.indices.groups.x[1]===6&&r.lastIndex===6"
        ));
        // Unicode 18 extends Han by U+2B81E; older Node data excludes it.
        check(&format!(
            r"/\p{{Script=Han}}/i{mode}.test('\u{{2B81E}}')&&!/\P{{sc=Hani}}/i{mode}.test('\u{{2B81E}}')"
        ));
    }
}
