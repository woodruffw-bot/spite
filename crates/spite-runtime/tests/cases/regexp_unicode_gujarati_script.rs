//! Exact Gujarati Script aliases, complements, consumers, and Unicode 18 membership.
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
fn unicode_gujarati_script_aliases_complements_and_fold_modes() {
    for name in ["Script=Gujarati", "Script=Gujr", "sc=Gujarati", "sc=Gujr"] {
        for flags in ["u", "ui", "v", "vi"] {
            check(&format!(
                r#"let patterns=["^\\p{{{name}}}$","^\\P{{{name}}}$","^[\\p{{{name}}}]$","^[\\P{{{name}}}]$","^[^\\p{{{name}}}]$","^[^\\P{{{name}}}]$"],points=[0,10,32,65,97,181,768,2709,2710,2689,2768,2404,2800,2688,2816,1488,8205,55296,56320,57344,64976,72448,72457,128512,1114111],flags={flags:?};patterns.map(pattern=>{{let r=new RegExp(pattern,flags);return points.map(cp=>r.test(String.fromCodePoint(cp))?'1':'0').join('')}}).join('|')==="0000000111101000000000000|1111111000010111111111111|0000000111101000000000000|1111111000010111111111111|1111111000010111111111111|0000000111101000000000000""#,
            ));
        }
    }
}

#[test]
fn unicode_gujarati_script_results_snapshot() {
    let mut rows = String::new();
    for name in ["Script=Gujarati", "Script=Gujr", "sc=Gujarati", "sc=Gujr"] {
        for body in [
            format!(r"\p{{{name}}}"),
            format!(r"\P{{{name}}}"),
            format!(r"(?<x>[\p{{{name}}}K])"),
        ] {
            let pattern = JsString::from(body.as_str());
            for flags in ["dug", "diug", "divg", "divy"] {
                for input in ["કખΣ😀𑬀\n", "X\r\nક\u{2028}א", ""] {
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
fn unicode_gujarati_script_captures_context_and_pair_starts() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let r=/^(?<x>([\p{{sc=Gujr}}K]))$/dim{mode}g,m=r.exec('A\r\nક\u2028');m.index===3&&m[1]==='ક'&&m[2]==='ક'&&m.indices.groups.x===m.indices[1]&&m.indices[2][1]===4&&r.lastIndex===4"
        ));
        check(&format!(
            r"let r=/(?<__proto__>\P{{Script=Gujarati}})/d{mode}y;r.lastIndex=1;let m=r.exec('😀ક');m.index===0&&m[1]==='😀'&&m.indices[1][1]===2&&r.lastIndex===2&&Object.getPrototypeOf(m.groups)===null&&m.indices.groups.__proto__===m.indices[1]"
        ));
        check(&format!(
            r"let r=/\p{{sc=Gujarati}}/d{mode}g;r.lastIndex=1;let m=r.exec('😀ક');m.index===2&&m[0]==='ક'&&r.lastIndex===3"
        ));
        check(&format!(
            r"let r=/\p{{Script=Gujr}}/{mode}y;r.lastIndex=1;r.exec('😀ક')===null&&r.lastIndex===0"
        ));
    }
}

#[test]
fn unicode_gujarati_script_consumers_preserve_original_text() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let a=[...'ક😀Aખ'.matchAll(/(?<x>\P{{sc=Gujarati}})/d{mode}g)];a.length===2&&a[0][1]==='😀'&&a[0].index===1&&a[0].indices[1][1]===3&&a[1][1]==='A'&&a[1].index===3"
        ));
        check(&format!(
            r"'કખA'.replace(/(?<x>\p{{sc=Gujr}})/{mode}g,'<$1:$<x>>')==='<ક:ક><ખ:ખ>A'"
        ));
        check(&format!(
            r"'AકખA'.split(/(\p{{Script=Gujarati}})/{mode}).join('|')==='A|ક||ખ|A'&&'A😀ક'.search(/\p{{sc=Gujr}}/{mode})===3&&'કખA'.match(/\p{{Script=Gujr}}/{mode}g).join('')==='કખ'"
        ));
        check(&format!(
            r"let t='',x='કખAક'.replace(/(\p{{sc=Gujarati}})/{mode}g,(m,c,p)=>{{t+=p;return '_'}});x==='__A_'&&t==='013'"
        ));
    }
}

#[test]
fn unicode_gujarati_script_clones_coercions_and_strict_writes() {
    check(
        r"let r=/^(?<x>\p{sc=Gujr})$/dmu,c=new RegExp(r,'divy');c.source==='^(?<x>\\p{sc=Gujr})$'&&c.unicodeSets&&!c.unicode&&!c.multiline&&c.exec('ક').groups.x==='ક'&&c.lastIndex===1",
    );
    check(
        r"let t='',r=/\P{Script=Gujarati}/uy;r.lastIndex={valueOf(){t+='i';return 1}};let m=r.exec({toString(){t+='s';return '😀'}});t==='si'&&m[0]==='😀'&&m.index===0&&r.lastIndex===2",
    );
    check(
        r"let r=/\p{sc=Gujarati}/vy;Object.defineProperty(r,'lastIndex',{writable:false});let caught=false;try{r.exec('ક')}catch(e){caught=e instanceof TypeError}caught&&r.lastIndex===0",
    );
    check(
        r"let r=/\P{Script=Gujr}/vy;r.lastIndex=1;Object.defineProperty(r,'lastIndex',{writable:false});let caught=false;try{r.exec('😀\uD800')}catch(e){caught=e instanceof TypeError}caught&&r.lastIndex===1",
    );
}

#[test]
fn unicode_gujarati_script_repeated_sources_captures_and_collection() {
    assert_eq!(Limits::default().max_steps, None);
    assert_eq!(Limits::default().max_heap_entries, None);
    let mut realm = Realm::default();
    realm.eval("let p='^'+'('.repeat(3000)+'['+'\\\\p{sc=Gujr}\\\\p{Cs}\\\\p{ASCII}'.repeat(10000)+']'+')'.repeat(3000)+'$',r=new RegExp(p,'dmvg'),c=new RegExp(r),text='A'.repeat(100000)+'\\nક'").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("let m=c.exec(text);m.length===3001&&m[1]==='ક'&&m[3000]==='ક'&&m.indices[3000][0]===100001&&m.indices[3000][1]===100002&&c.lastIndex===100002"), Ok(Value::Boolean(true)));
    realm.eval("p=null;r=null;c=null;text=null;m=null").unwrap();
    realm.collect(usize::MAX).unwrap();
}

#[test]
fn unicode_gujarati_script_opted_work_and_remaining_sets() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval("let r=/\\p{Script=Gujarati}/ug,marker=0,text='A'.repeat(10000);r.lastIndex=1")
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
        r"/\p{Script_Extensions=Gujarati}/u.exec('A')",
        r"/[\p{sc=Gujr}&&A]/v.exec('A')",
        r"/[\p{sc=Gujr}--A]/v.exec('A')",
        r"/\p{sc=Gujr}+b/u.exec('AA')",
        r"/\p{sc=Gujr}A/v.exec('AA')",
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
fn unicode_gujarati_script_repetitions_retain_last_capture_and_outside_words() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let r=/\B(?<x>\p{{sc=Gujr}})+\B/d{mode}g,m=r.exec('µકકકµ');m.index===1&&m[0]==='કકક'&&m.groups.x==='ક'&&m.indices.groups.x[0]===3&&m.indices.groups.x[1]===4&&r.lastIndex===4"
        ));
    }
}

#[test]
fn unicode_gujarati_script_former_direct_eval_context_executes_normally() {
    check(
        r#"let marker=0,result;try{result=eval("marker=1; /\\p{Script=Gujarati}/u.test('ક');")}catch{marker=2}finally{marker=3}marker===3&&result===true"#,
    );
}
