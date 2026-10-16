//! Exact Gurmukhi Script aliases, complements, consumers, and Unicode 18 membership.
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
fn unicode_gurmukhi_script_aliases_complements_and_fold_modes() {
    for name in ["Script=Gurmukhi", "Script=Guru", "sc=Gurmukhi", "sc=Guru"] {
        for flags in ["u", "ui", "v", "vi"] {
            check(&format!(
                r#"let patterns=["^\\p{{{name}}}$","^\\P{{{name}}}$","^[\\p{{{name}}}]$","^[\\P{{{name}}}]$","^[^\\p{{{name}}}]$","^[^\\P{{{name}}}]$"],points=[0,10,32,65,97,181,768,2581,2582,2561,2641,2404,2672,2560,2688,1488,8205,55296,56320,57344,64976,72448,72457,128512,1114111],flags={flags:?};patterns.map(pattern=>{{let r=new RegExp(pattern,flags);return points.map(cp=>r.test(String.fromCodePoint(cp))?'1':'0').join('')}}).join('|')==="0000000111101000000000000|1111111000010111111111111|0000000111101000000000000|1111111000010111111111111|1111111000010111111111111|0000000111101000000000000""#,
            ));
        }
    }
}

#[test]
fn unicode_gurmukhi_script_results_snapshot() {
    let mut rows = String::new();
    for name in ["Script=Gurmukhi", "Script=Guru", "sc=Gurmukhi", "sc=Guru"] {
        for body in [
            format!(r"\p{{{name}}}"),
            format!(r"\P{{{name}}}"),
            format!(r"(?<x>[\p{{{name}}}K])"),
        ] {
            let pattern = JsString::from(body.as_str());
            for flags in ["dug", "diug", "divg", "divy"] {
                for input in ["ਕਖΣ😀𑬀\n", "X\r\nਕ\u{2028}א", ""] {
                    let input = JsString::from(input);
                    for start in [0, 1, 3] {
                        let script = format!(
                            "let r=new RegExp({pattern:?},{flags:?});r.lastIndex={start};let m=r.exec({input:?});JSON.stringify({{m,index:m?.index,groups:m?.groups,indices:m?.indices,indexGroups:m?.indices?.groups,lastIndex:r.lastIndex,source:r.source}})"
                        );
                        let Value::String(result) = Realm::default().eval(&script).unwrap() else {
                            panic!("{script}")
                        };
                        writeln!(
                            rows,
                            "{body:?} flags={flags:?} input={input:?} lastIndex={start} {result:?}"
                        )
                        .unwrap();
                    }
                }
            }
        }
    }
    insta::assert_snapshot!(rows);
}
#[test]
fn unicode_gurmukhi_script_captures_context_and_pair_starts() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let r=/^(?<x>([\p{{sc=Guru}}K]))$/dim{mode}g,m=r.exec('A\r\nਕ\u2028');m.index===3&&m[1]==='ਕ'&&m[2]==='ਕ'&&m.indices.groups.x===m.indices[1]&&m.indices[2][1]===4&&r.lastIndex===4"
        ));
        check(&format!(
            r"let r=/(?<__proto__>\P{{Script=Gurmukhi}})/d{mode}y;r.lastIndex=1;let m=r.exec('😀ਕ');m.index===0&&m[1]==='😀'&&m.indices[1][1]===2&&r.lastIndex===2&&Object.getPrototypeOf(m.groups)===null&&m.indices.groups.__proto__===m.indices[1]"
        ));
        check(&format!(
            r"let r=/\p{{sc=Gurmukhi}}/d{mode}g;r.lastIndex=1;let m=r.exec('😀ਕ');m.index===2&&m[0]==='ਕ'&&r.lastIndex===3"
        ));
        check(&format!(
            r"let r=/\p{{Script=Guru}}/{mode}y;r.lastIndex=1;r.exec('😀ਕ')===null&&r.lastIndex===0"
        ));
    }
}

#[test]
fn unicode_gurmukhi_script_consumers_preserve_original_text() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let a=[...'ਕ😀Aਖ'.matchAll(/(?<x>\P{{sc=Gurmukhi}})/d{mode}g)];a.length===2&&a[0][1]==='😀'&&a[0].index===1&&a[0].indices[1][1]===3&&a[1][1]==='A'&&a[1].index===3"
        ));
        check(&format!(
            r"'ਕਖA'.replace(/(?<x>\p{{sc=Guru}})/{mode}g,'<$1:$<x>>')==='<ਕ:ਕ><ਖ:ਖ>A'"
        ));
        check(&format!(
            r"'AਕਖA'.split(/(\p{{Script=Gurmukhi}})/{mode}).join('|')==='A|ਕ||ਖ|A'&&'A😀ਕ'.search(/\p{{sc=Guru}}/{mode})===3&&'ਕਖA'.match(/\p{{Script=Guru}}/{mode}g).join('')==='ਕਖ'"
        ));
        check(&format!(
            r"let t='',x='ਕਖAਕ'.replace(/(\p{{sc=Gurmukhi}})/{mode}g,(m,c,p)=>{{t+=p;return '_'}});x==='__A_'&&t==='013'"
        ));
    }
}

#[test]
fn unicode_gurmukhi_script_clones_coercions_and_strict_writes() {
    check(
        r"let r=/^(?<x>\p{sc=Guru})$/dmu,c=new RegExp(r,'divy');c.source==='^(?<x>\\p{sc=Guru})$'&&c.unicodeSets&&!c.unicode&&!c.multiline&&c.exec('ਕ').groups.x==='ਕ'&&c.lastIndex===1",
    );
    check(
        r"let t='',r=/\P{Script=Gurmukhi}/uy;r.lastIndex={valueOf(){t+='i';return 1}};let m=r.exec({toString(){t+='s';return '😀'}});t==='si'&&m[0]==='😀'&&m.index===0&&r.lastIndex===2",
    );
    check(
        r"let r=/\p{sc=Gurmukhi}/vy;Object.defineProperty(r,'lastIndex',{writable:false});let caught=false;try{r.exec('ਕ')}catch(e){caught=e instanceof TypeError}caught&&r.lastIndex===0",
    );
    check(
        r"let r=/\P{Script=Guru}/vy;r.lastIndex=1;Object.defineProperty(r,'lastIndex',{writable:false});let caught=false;try{r.exec('😀\uD800')}catch(e){caught=e instanceof TypeError}caught&&r.lastIndex===1",
    );
}

#[test]
fn unicode_gurmukhi_script_repeated_sources_captures_and_collection() {
    assert_eq!(Limits::default().max_steps, None);
    assert_eq!(Limits::default().max_heap_entries, None);
    let mut realm = Realm::default();
    realm.eval("let p='^'+'('.repeat(3000)+'['+'\\\\p{sc=Guru}\\\\p{Cs}\\\\p{ASCII}'.repeat(10000)+']'+')'.repeat(3000)+'$',r=new RegExp(p,'dmvg'),c=new RegExp(r),text='A'.repeat(100000)+'\\nਕ'").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("let m=c.exec(text);m.length===3001&&m[1]==='ਕ'&&m[3000]==='ਕ'&&m.indices[3000][0]===100001&&m.indices[3000][1]===100002&&c.lastIndex===100002"), Ok(Value::Boolean(true)));
    realm.eval("p=null;r=null;c=null;text=null;m=null").unwrap();
    realm.collect(usize::MAX).unwrap();
}

#[test]
fn unicode_gurmukhi_script_opted_work_and_remaining_sets() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval("let r=/\\p{Script=Gurmukhi}/ug,marker=0,text='A'.repeat(10000);r.lastIndex=1")
        .unwrap();
    assert!(matches!(
        realm.eval("try{r.exec(text)}catch{marker=1}finally{marker=2}"),
        Err(Error::Limit { .. })
    ));
    assert_eq!(
        realm.eval("marker===0&&r.lastIndex===1"),
        Ok(Value::Boolean(true))
    );
    check(r"/\p{Script_Extensions=Gurmukhi}/u.exec('A')===null");
    for source in [
        r"/[\p{sc=Guru}&&A]/v.exec('A')",
        r"/[\p{sc=Guru}--A]/v.exec('A')",
        r"/\p{sc=Guru}+b/u.exec('AA')",
        r"/\p{sc=Guru}A/v.exec('AA')",
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
fn unicode_gurmukhi_script_repetitions_retain_last_capture_and_outside_words() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let r=/\B(?<x>\p{{sc=Guru}})+\B/d{mode}g,m=r.exec('µਕਕਕµ');m.index===1&&m[0]==='ਕਕਕ'&&m.groups.x==='ਕ'&&m.indices.groups.x[0]===3&&m.indices.groups.x[1]===4&&r.lastIndex===4"
        ));
    }
}

#[test]
fn unicode_gurmukhi_script_former_direct_eval_context_executes_normally() {
    check(
        r#"let marker=0,result;try{result=eval("marker=1; /\\p{Script=Gurmukhi}/u.test('ਕ');")}catch{marker=2}finally{marker=3}marker===3&&result===true"#,
    );
}
