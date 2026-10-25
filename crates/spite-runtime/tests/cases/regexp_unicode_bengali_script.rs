//! Exact Bengali Script aliases, complements, consumers, and Unicode 18 changes.
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
fn unicode_bengali_script_aliases_complements_and_fold_modes() {
    for name in ["Script=Bengali", "Script=Beng", "sc=Bengali", "sc=Beng"] {
        for flags in ["u", "ui", "v", "vi"] {
            check(&format!(
                r#"let patterns=["^\\p{{{name}}}$","^\\P{{{name}}}$","^[\\p{{{name}}}]$","^[\\P{{{name}}}]$","^[^\\p{{{name}}}]$","^[^\\P{{{name}}}]$"],points=[0,10,32,65,97,181,768,2453,2454,2432,2512,2404,2544,2431,2560,1488,8205,55296,56320,57344,64976,72448,72457,128512,1114111],flags={flags:?};patterns.map(pattern=>{{let r=new RegExp(pattern,flags);return points.map(cp=>r.test(String.fromCodePoint(cp))?'1':'0').join('')}}).join('|')==="0000000111001000000000000|1111111000110111111111111|0000000111001000000000000|1111111000110111111111111|1111111000110111111111111|0000000111001000000000000""#,
            ));
        }
    }
}

#[test]
fn unicode_bengali_script_unicode18_additions() {
    for flags in ["u", "ui", "v", "vi"] {
        check(&format!(
            r#"let patterns=["^\\p{{sc=Beng}}$","^\\P{{sc=Beng}}$","^[\\p{{sc=Beng}}]$","^[\\P{{sc=Beng}}]$","^[^\\p{{sc=Beng}}]$","^[^\\P{{sc=Beng}}]$"],points=[73200,73201],flags={flags:?};patterns.map(pattern=>{{let r=new RegExp(pattern,flags);return points.map(cp=>r.test(String.fromCodePoint(cp))?'1':'0').join('')}}).join('|')==="11|00|11|00|00|11""#,
        ));
    }
}

#[test]
fn unicode_bengali_script_results_snapshot() {
    let mut rows = String::new();
    for name in ["Script=Bengali", "Script=Beng", "sc=Bengali", "sc=Beng"] {
        for body in [
            format!(r"\p{{{name}}}"),
            format!(r"\P{{{name}}}"),
            format!(r"(?<x>[\p{{{name}}}K])"),
        ] {
            let pattern = JsString::from(body.as_str());
            for flags in ["dug", "diug", "divg", "divy"] {
                for input in ["কখΣ😀𑬀\n", "X\r\nক\u{2028}א", ""] {
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
fn unicode_bengali_script_captures_context_and_pair_starts() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let r=/^(?<x>([\p{{sc=Beng}}K]))$/dim{mode}g,m=r.exec('A\r\nক\u2028');m.index===3&&m[1]==='ক'&&m[2]==='ক'&&m.indices.groups.x===m.indices[1]&&m.indices[2][1]===4&&r.lastIndex===4"
        ));
        check(&format!(
            r"let r=/(?<__proto__>\p{{Script=Bengali}})/d{mode}y;r.lastIndex=1;let m=r.exec('𑷰A');m.index===0&&m[1]==='𑷰'&&m.indices[1][1]===2&&r.lastIndex===2&&Object.getPrototypeOf(m.groups)===null&&m.indices.groups.__proto__===m.indices[1]"
        ));
        check(&format!(
            r"let r=/\P{{sc=Bengali}}/d{mode}g;r.lastIndex=1;let m=r.exec('𑷰\uD800');m.index===2&&m[0]==='\uD800'&&r.lastIndex===3"
        ));
        check(&format!(
            r"let r=/\P{{Script=Beng}}/{mode}y;r.lastIndex=1;r.exec('𑷰\uD800')===null&&r.lastIndex===0"
        ));
    }
}

#[test]
fn unicode_bengali_script_consumers_preserve_original_text() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let a=[...'ক😀Aখ'.matchAll(/(?<x>\P{{sc=Bengali}})/d{mode}g)];a.length===2&&a[0][1]==='😀'&&a[0].index===1&&a[0].indices[1][1]===3&&a[1][1]==='A'&&a[1].index===3"
        ));
        check(&format!(
            r"'কখA'.replace(/(?<x>\p{{sc=Beng}})/{mode}g,'<$1:$<x>>')==='<ক:ক><খ:খ>A'"
        ));
        check(&format!(
            r"'AকখA'.split(/(\p{{Script=Bengali}})/{mode}).join('|')==='A|ক||খ|A'&&'A😀ক'.search(/\p{{sc=Beng}}/{mode})===3&&'কখA'.match(/\p{{Script=Beng}}/{mode}g).join('')==='কখ'"
        ));
        check(&format!(
            r"let t='',x='কখAক'.replace(/(\p{{sc=Bengali}})/{mode}g,(m,c,p)=>{{t+=p;return '_'}});x==='__A_'&&t==='013'"
        ));
    }
}

#[test]
fn unicode_bengali_script_clones_coercions_and_strict_writes() {
    check(
        r"let r=/^(?<x>\p{sc=Beng})$/dmu,c=new RegExp(r,'divy');c.source==='^(?<x>\\p{sc=Beng})$'&&c.unicodeSets&&!c.unicode&&!c.multiline&&c.exec('ক').groups.x==='ক'&&c.lastIndex===1",
    );
    check(
        r"let t='',r=/\p{Script=Bengali}/uy;r.lastIndex={valueOf(){t+='i';return 1}};let m=r.exec({toString(){t+='s';return '𑷰'}});t==='si'&&m[0]==='𑷰'&&m.index===0&&r.lastIndex===2",
    );
    check(
        r"let r=/\p{sc=Bengali}/vy;Object.defineProperty(r,'lastIndex',{writable:false});let caught=false;try{r.exec('ক')}catch(e){caught=e instanceof TypeError}caught&&r.lastIndex===0",
    );
    check(
        r"let r=/\P{Script=Beng}/vy;r.lastIndex=1;Object.defineProperty(r,'lastIndex',{writable:false});let caught=false;try{r.exec('𑷰\uD800')}catch(e){caught=e instanceof TypeError}caught&&r.lastIndex===1",
    );
}

#[test]
fn unicode_bengali_script_opted_work_and_remaining_sets() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval("let r=/\\p{Script=Bengali}/ug,marker=0,text='A'.repeat(10000);r.lastIndex=1")
        .unwrap();
    assert!(matches!(
        realm.eval("try{r.exec(text)}catch{marker=1}finally{marker=2}"),
        Err(Error::Limit { .. })
    ));
    assert_eq!(
        realm.eval("marker===0&&r.lastIndex===1"),
        Ok(Value::Boolean(true))
    );
    check(r"/\p{Script_Extensions=Bengali}/u.exec('A')===null");
    check(r"/[\p{sc=Beng}&&A]/v.exec('A')===null");
    check(r"/[\p{sc=Beng}--A]/v.exec('A')===null");
    check(
        r"let s=String.fromCodePoint(0x995),m=/[\p{sc=Beng}--A]/v.exec(s);m!==null&&m.length===1&&m[0]===s&&m.index===0&&m.input===s",
    );
    for source in [
        r"/\p{sc=Beng}+b/u.exec('AA')",
        r"/\p{sc=Beng}A/v.exec('AA')",
    ] {
        assert!(
            crate::common::regexp_oracle::check(&mut Realm::default(), source),
            "{source}"
        );
    }
}

#[test]
fn unicode_bengali_script_repetitions_retain_last_capture_and_outside_words() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let r=/\B(?<x>\p{{sc=Beng}})+\B/d{mode}g,m=r.exec('µ𑷰𑷰𑷰µ');m.index===1&&m[0]==='𑷰𑷰𑷰'&&m.groups.x==='𑷰'&&m.indices.groups.x[0]===5&&m.indices.groups.x[1]===7&&r.lastIndex===7"
        ));
    }
}

#[test]
fn unicode_bengali_script_former_direct_eval_context_executes_normally() {
    check(
        r#"let marker=0,result;try{result=eval("marker=1; /\\p{Script=Bengali}/u.test('ক');")}catch{marker=2}finally{marker=3}marker===3&&result===true"#,
    );
}
