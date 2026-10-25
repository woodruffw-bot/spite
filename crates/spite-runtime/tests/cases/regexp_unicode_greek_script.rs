//! Exact Greek Script aliases, complements, consumers, and Unicode 18 changes.
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
fn unicode_greek_script_aliases_complements_and_fold_modes() {
    check(r"/\p{Script=Greek}/v.exec('𠮷')===null");
    check(r"/[\p{Script=Greek}]/v.exec('𠮷')===null");
    check(
        r#"let patterns=["^\\p{Script=Grek}$", "^\\P{Script=Grek}$", "^[\\p{Script=Grek}]$", "^[\\P{Script=Grek}]$", "^[^\\p{Script=Grek}]$", "^[^\\P{Script=Grek}]$"],points=[0, 10, 32, 65, 97, 181, 837, 880, 881, 888, 953, 956, 931, 962, 963, 8064, 8072, 8126, 8192, 8486, 8490, 55296, 56320, 57344, 64976, 65856, 128512, 1114111],flags="u";patterns.map(pattern=>{let r=new RegExp(pattern,flags);return points.map(cp=>r.test(String.fromCodePoint(cp))?'1':'0').join('')}).join('|')==="0000000110111111110100000100|1111111001000000001011111011|0000000110111111110100000100|1111111001000000001011111011|1111111001000000001011111011|0000000110111111110100000100""#,
    );
    check(
        r#"let patterns=["^\\p{Script=Grek}$", "^\\P{Script=Grek}$", "^[\\p{Script=Grek}]$", "^[\\P{Script=Grek}]$", "^[^\\p{Script=Grek}]$", "^[^\\P{Script=Grek}]$"],points=[0, 10, 32, 65, 97, 181, 837, 880, 881, 888, 953, 956, 931, 962, 963, 8064, 8072, 8126, 8192, 8486, 8490, 55296, 56320, 57344, 64976, 65856, 128512, 1114111],flags="ui";patterns.map(pattern=>{let r=new RegExp(pattern,flags);return points.map(cp=>r.test(String.fromCodePoint(cp))?'1':'0').join('')}).join('|')==="0000011110111111110100000100|1111111001110000011011111011|0000011110111111110100000100|1111111001110000011011111011|1111100001000000001011111011|0000000110001111100100000100""#,
    );
    check(
        r#"let patterns=["^\\p{Script=Grek}$", "^\\P{Script=Grek}$", "^[\\p{Script=Grek}]$", "^[\\P{Script=Grek}]$", "^[^\\p{Script=Grek}]$", "^[^\\P{Script=Grek}]$"],points=[0, 10, 32, 65, 97, 181, 837, 880, 881, 888, 953, 956, 931, 962, 963, 8064, 8072, 8126, 8192, 8486, 8490, 55296, 56320, 57344, 64976, 65856, 128512, 1114111],flags="v";patterns.map(pattern=>{let r=new RegExp(pattern,flags);return points.map(cp=>r.test(String.fromCodePoint(cp))?'1':'0').join('')}).join('|')==="0000000110111111110100000100|1111111001000000001011111011|0000000110111111110100000100|1111111001000000001011111011|1111111001000000001011111011|0000000110111111110100000100""#,
    );
    check(
        r#"let patterns=["^\\p{Script=Grek}$", "^\\P{Script=Grek}$", "^[\\p{Script=Grek}]$", "^[\\P{Script=Grek}]$", "^[^\\p{Script=Grek}]$", "^[^\\P{Script=Grek}]$"],points=[0, 10, 32, 65, 97, 181, 837, 880, 881, 888, 953, 956, 931, 962, 963, 8064, 8072, 8126, 8192, 8486, 8490, 55296, 56320, 57344, 64976, 65856, 128512, 1114111],flags="vi";patterns.map(pattern=>{let r=new RegExp(pattern,flags);return points.map(cp=>r.test(String.fromCodePoint(cp))?'1':'0').join('')}).join('|')==="0000011110111111110100000100|1111100001000000001011111011|0000011110111111110100000100|1111100001000000001011111011|1111100001000000001011111011|0000011110111111110100000100""#,
    );
    check(
        r#"let patterns=["^\\p{Script=Greek}$", "^\\P{Script=Greek}$", "^[\\p{Script=Greek}]$", "^[\\P{Script=Greek}]$", "^[^\\p{Script=Greek}]$", "^[^\\P{Script=Greek}]$"],points=[0, 10, 32, 65, 97, 181, 837, 880, 881, 888, 953, 956, 931, 962, 963, 8064, 8072, 8126, 8192, 8486, 8490, 55296, 56320, 57344, 64976, 65856, 128512, 1114111],flags="u";patterns.map(pattern=>{let r=new RegExp(pattern,flags);return points.map(cp=>r.test(String.fromCodePoint(cp))?'1':'0').join('')}).join('|')==="0000000110111111110100000100|1111111001000000001011111011|0000000110111111110100000100|1111111001000000001011111011|1111111001000000001011111011|0000000110111111110100000100""#,
    );
    check(
        r#"let patterns=["^\\p{Script=Greek}$", "^\\P{Script=Greek}$", "^[\\p{Script=Greek}]$", "^[\\P{Script=Greek}]$", "^[^\\p{Script=Greek}]$", "^[^\\P{Script=Greek}]$"],points=[0, 10, 32, 65, 97, 181, 837, 880, 881, 888, 953, 956, 931, 962, 963, 8064, 8072, 8126, 8192, 8486, 8490, 55296, 56320, 57344, 64976, 65856, 128512, 1114111],flags="ui";patterns.map(pattern=>{let r=new RegExp(pattern,flags);return points.map(cp=>r.test(String.fromCodePoint(cp))?'1':'0').join('')}).join('|')==="0000011110111111110100000100|1111111001110000011011111011|0000011110111111110100000100|1111111001110000011011111011|1111100001000000001011111011|0000000110001111100100000100""#,
    );
    check(
        r#"let patterns=["^\\p{Script=Greek}$", "^\\P{Script=Greek}$", "^[\\p{Script=Greek}]$", "^[\\P{Script=Greek}]$", "^[^\\p{Script=Greek}]$", "^[^\\P{Script=Greek}]$"],points=[0, 10, 32, 65, 97, 181, 837, 880, 881, 888, 953, 956, 931, 962, 963, 8064, 8072, 8126, 8192, 8486, 8490, 55296, 56320, 57344, 64976, 65856, 128512, 1114111],flags="v";patterns.map(pattern=>{let r=new RegExp(pattern,flags);return points.map(cp=>r.test(String.fromCodePoint(cp))?'1':'0').join('')}).join('|')==="0000000110111111110100000100|1111111001000000001011111011|0000000110111111110100000100|1111111001000000001011111011|1111111001000000001011111011|0000000110111111110100000100""#,
    );
    check(
        r#"let patterns=["^\\p{Script=Greek}$", "^\\P{Script=Greek}$", "^[\\p{Script=Greek}]$", "^[\\P{Script=Greek}]$", "^[^\\p{Script=Greek}]$", "^[^\\P{Script=Greek}]$"],points=[0, 10, 32, 65, 97, 181, 837, 880, 881, 888, 953, 956, 931, 962, 963, 8064, 8072, 8126, 8192, 8486, 8490, 55296, 56320, 57344, 64976, 65856, 128512, 1114111],flags="vi";patterns.map(pattern=>{let r=new RegExp(pattern,flags);return points.map(cp=>r.test(String.fromCodePoint(cp))?'1':'0').join('')}).join('|')==="0000011110111111110100000100|1111100001000000001011111011|0000011110111111110100000100|1111100001000000001011111011|1111100001000000001011111011|0000011110111111110100000100""#,
    );
    check(
        r#"let patterns=["^\\p{sc=Grek}$", "^\\P{sc=Grek}$", "^[\\p{sc=Grek}]$", "^[\\P{sc=Grek}]$", "^[^\\p{sc=Grek}]$", "^[^\\P{sc=Grek}]$"],points=[0, 10, 32, 65, 97, 181, 837, 880, 881, 888, 953, 956, 931, 962, 963, 8064, 8072, 8126, 8192, 8486, 8490, 55296, 56320, 57344, 64976, 65856, 128512, 1114111],flags="u";patterns.map(pattern=>{let r=new RegExp(pattern,flags);return points.map(cp=>r.test(String.fromCodePoint(cp))?'1':'0').join('')}).join('|')==="0000000110111111110100000100|1111111001000000001011111011|0000000110111111110100000100|1111111001000000001011111011|1111111001000000001011111011|0000000110111111110100000100""#,
    );
    check(
        r#"let patterns=["^\\p{sc=Grek}$", "^\\P{sc=Grek}$", "^[\\p{sc=Grek}]$", "^[\\P{sc=Grek}]$", "^[^\\p{sc=Grek}]$", "^[^\\P{sc=Grek}]$"],points=[0, 10, 32, 65, 97, 181, 837, 880, 881, 888, 953, 956, 931, 962, 963, 8064, 8072, 8126, 8192, 8486, 8490, 55296, 56320, 57344, 64976, 65856, 128512, 1114111],flags="ui";patterns.map(pattern=>{let r=new RegExp(pattern,flags);return points.map(cp=>r.test(String.fromCodePoint(cp))?'1':'0').join('')}).join('|')==="0000011110111111110100000100|1111111001110000011011111011|0000011110111111110100000100|1111111001110000011011111011|1111100001000000001011111011|0000000110001111100100000100""#,
    );
    check(
        r#"let patterns=["^\\p{sc=Grek}$", "^\\P{sc=Grek}$", "^[\\p{sc=Grek}]$", "^[\\P{sc=Grek}]$", "^[^\\p{sc=Grek}]$", "^[^\\P{sc=Grek}]$"],points=[0, 10, 32, 65, 97, 181, 837, 880, 881, 888, 953, 956, 931, 962, 963, 8064, 8072, 8126, 8192, 8486, 8490, 55296, 56320, 57344, 64976, 65856, 128512, 1114111],flags="v";patterns.map(pattern=>{let r=new RegExp(pattern,flags);return points.map(cp=>r.test(String.fromCodePoint(cp))?'1':'0').join('')}).join('|')==="0000000110111111110100000100|1111111001000000001011111011|0000000110111111110100000100|1111111001000000001011111011|1111111001000000001011111011|0000000110111111110100000100""#,
    );
    check(
        r#"let patterns=["^\\p{sc=Grek}$", "^\\P{sc=Grek}$", "^[\\p{sc=Grek}]$", "^[\\P{sc=Grek}]$", "^[^\\p{sc=Grek}]$", "^[^\\P{sc=Grek}]$"],points=[0, 10, 32, 65, 97, 181, 837, 880, 881, 888, 953, 956, 931, 962, 963, 8064, 8072, 8126, 8192, 8486, 8490, 55296, 56320, 57344, 64976, 65856, 128512, 1114111],flags="vi";patterns.map(pattern=>{let r=new RegExp(pattern,flags);return points.map(cp=>r.test(String.fromCodePoint(cp))?'1':'0').join('')}).join('|')==="0000011110111111110100000100|1111100001000000001011111011|0000011110111111110100000100|1111100001000000001011111011|1111100001000000001011111011|0000011110111111110100000100""#,
    );
    check(
        r#"let patterns=["^\\p{sc=Greek}$", "^\\P{sc=Greek}$", "^[\\p{sc=Greek}]$", "^[\\P{sc=Greek}]$", "^[^\\p{sc=Greek}]$", "^[^\\P{sc=Greek}]$"],points=[0, 10, 32, 65, 97, 181, 837, 880, 881, 888, 953, 956, 931, 962, 963, 8064, 8072, 8126, 8192, 8486, 8490, 55296, 56320, 57344, 64976, 65856, 128512, 1114111],flags="u";patterns.map(pattern=>{let r=new RegExp(pattern,flags);return points.map(cp=>r.test(String.fromCodePoint(cp))?'1':'0').join('')}).join('|')==="0000000110111111110100000100|1111111001000000001011111011|0000000110111111110100000100|1111111001000000001011111011|1111111001000000001011111011|0000000110111111110100000100""#,
    );
    check(
        r#"let patterns=["^\\p{sc=Greek}$", "^\\P{sc=Greek}$", "^[\\p{sc=Greek}]$", "^[\\P{sc=Greek}]$", "^[^\\p{sc=Greek}]$", "^[^\\P{sc=Greek}]$"],points=[0, 10, 32, 65, 97, 181, 837, 880, 881, 888, 953, 956, 931, 962, 963, 8064, 8072, 8126, 8192, 8486, 8490, 55296, 56320, 57344, 64976, 65856, 128512, 1114111],flags="ui";patterns.map(pattern=>{let r=new RegExp(pattern,flags);return points.map(cp=>r.test(String.fromCodePoint(cp))?'1':'0').join('')}).join('|')==="0000011110111111110100000100|1111111001110000011011111011|0000011110111111110100000100|1111111001110000011011111011|1111100001000000001011111011|0000000110001111100100000100""#,
    );
    check(
        r#"let patterns=["^\\p{sc=Greek}$", "^\\P{sc=Greek}$", "^[\\p{sc=Greek}]$", "^[\\P{sc=Greek}]$", "^[^\\p{sc=Greek}]$", "^[^\\P{sc=Greek}]$"],points=[0, 10, 32, 65, 97, 181, 837, 880, 881, 888, 953, 956, 931, 962, 963, 8064, 8072, 8126, 8192, 8486, 8490, 55296, 56320, 57344, 64976, 65856, 128512, 1114111],flags="v";patterns.map(pattern=>{let r=new RegExp(pattern,flags);return points.map(cp=>r.test(String.fromCodePoint(cp))?'1':'0').join('')}).join('|')==="0000000110111111110100000100|1111111001000000001011111011|0000000110111111110100000100|1111111001000000001011111011|1111111001000000001011111011|0000000110111111110100000100""#,
    );
    check(
        r#"let patterns=["^\\p{sc=Greek}$", "^\\P{sc=Greek}$", "^[\\p{sc=Greek}]$", "^[\\P{sc=Greek}]$", "^[^\\p{sc=Greek}]$", "^[^\\P{sc=Greek}]$"],points=[0, 10, 32, 65, 97, 181, 837, 880, 881, 888, 953, 956, 931, 962, 963, 8064, 8072, 8126, 8192, 8486, 8490, 55296, 56320, 57344, 64976, 65856, 128512, 1114111],flags="vi";patterns.map(pattern=>{let r=new RegExp(pattern,flags);return points.map(cp=>r.test(String.fromCodePoint(cp))?'1':'0').join('')}).join('|')==="0000011110111111110100000100|1111100001000000001011111011|0000011110111111110100000100|1111100001000000001011111011|1111100001000000001011111011|0000011110111111110100000100""#,
    );
}

#[test]
fn unicode_greek_script_unicode18_additions() {
    check(
        r#"let patterns=["^\\p{sc=Grek}$", "^\\P{sc=Grek}$", "^[\\p{sc=Grek}]$", "^[\\P{sc=Grek}]$", "^[^\\p{sc=Grek}]$", "^[^\\P{sc=Grek}]$"],points=[122867, 122868],flags="u";patterns.map(pattern=>{let r=new RegExp(pattern,flags);return points.map(cp=>r.test(String.fromCodePoint(cp))?'1':'0').join('')}).join('|')==="11|00|11|00|00|11""#,
    );
    check(
        r#"let patterns=["^\\p{sc=Grek}$", "^\\P{sc=Grek}$", "^[\\p{sc=Grek}]$", "^[\\P{sc=Grek}]$", "^[^\\p{sc=Grek}]$", "^[^\\P{sc=Grek}]$"],points=[122867, 122868],flags="ui";patterns.map(pattern=>{let r=new RegExp(pattern,flags);return points.map(cp=>r.test(String.fromCodePoint(cp))?'1':'0').join('')}).join('|')==="11|00|11|00|00|11""#,
    );
    check(
        r#"let patterns=["^\\p{sc=Grek}$", "^\\P{sc=Grek}$", "^[\\p{sc=Grek}]$", "^[\\P{sc=Grek}]$", "^[^\\p{sc=Grek}]$", "^[^\\P{sc=Grek}]$"],points=[122867, 122868],flags="v";patterns.map(pattern=>{let r=new RegExp(pattern,flags);return points.map(cp=>r.test(String.fromCodePoint(cp))?'1':'0').join('')}).join('|')==="11|00|11|00|00|11""#,
    );
    check(
        r#"let patterns=["^\\p{sc=Grek}$", "^\\P{sc=Grek}$", "^[\\p{sc=Grek}]$", "^[\\P{sc=Grek}]$", "^[^\\p{sc=Grek}]$", "^[^\\P{sc=Grek}]$"],points=[122867, 122868],flags="vi";patterns.map(pattern=>{let r=new RegExp(pattern,flags);return points.map(cp=>r.test(String.fromCodePoint(cp))?'1':'0').join('')}).join('|')==="11|00|11|00|00|11""#,
    );
}

#[test]
fn unicode_greek_script_results_snapshot() {
    let mut rows = String::new();
    for name in ["Script=Greek", "Script=Grek", "sc=Greek", "sc=Grek"] {
        for body in [
            format!(r"\p{{{name}}}"),
            format!(r"\P{{{name}}}"),
            format!(r"(?<x>[\p{{{name}}}K])"),
        ] {
            let pattern = JsString::from(body.as_str());
            for flags in ["dug", "diug", "divg", "divy"] {
                for input in ["µͅιμΣς😀\u{378}\n", "X\r\nA\u{2028}𐅀", ""] {
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
fn unicode_greek_script_captures_context_and_pair_starts() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let r=/^(?<x>([\p{{sc=Grek}}K]))$/dim{mode}g,m=r.exec('A\r\nΣ\u2028');m.index===3&&m[1]==='Σ'&&m[2]==='Σ'&&m.indices.groups.x===m.indices[1]&&m.indices[2][1]===4&&r.lastIndex===4"
        ));
        check(&format!(
            r"let r=/(?<__proto__>\p{{Script=Greek}})/d{mode}y;r.lastIndex=1;let m=r.exec('𐅀A');m.index===0&&m[1]==='𐅀'&&m.indices[1][1]===2&&r.lastIndex===2&&Object.getPrototypeOf(m.groups)===null&&m.indices.groups.__proto__===m.indices[1]"
        ));
        check(&format!(
            r"let r=/\P{{sc=Greek}}/d{mode}g;r.lastIndex=1;let m=r.exec('𐅀\uD800');m.index===2&&m[0]==='\uD800'&&r.lastIndex===3"
        ));
        check(&format!(
            r"let r=/\P{{Script=Grek}}/{mode}y;r.lastIndex=1;r.exec('𐅀\uD800')===null&&r.lastIndex===0"
        ));
    }
}

#[test]
fn unicode_greek_script_consumers_preserve_original_text() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let a=[...'Α😀AΒ'.matchAll(/(?<x>\P{{sc=Greek}})/d{mode}g)];a.length===2&&a[0][1]==='😀'&&a[0].index===1&&a[0].indices[1][1]===3&&a[1][1]==='A'&&a[1].index===3"
        ));
        check(&format!(
            r"'ΑΒA'.replace(/(?<x>\p{{sc=Grek}})/{mode}g,'<$1:$<x>>')==='<Α:Α><Β:Β>A'"
        ));
        check(&format!(
            r"'AΑΒA'.split(/(\p{{Script=Greek}})/{mode}).join('|')==='A|Α||Β|A'&&'A😀Α'.search(/\p{{sc=Grek}}/{mode})===3&&'ΑΒA'.match(/\p{{Script=Grek}}/{mode}g).join('')==='ΑΒ'"
        ));
        check(&format!(
            r"let t='',x='ΑΒAΑ'.replace(/(\p{{sc=Greek}})/{mode}g,(m,c,p)=>{{t+=p;return '_'}});x==='__A_'&&t==='013'"
        ));
    }
}

#[test]
fn unicode_greek_script_clones_coercions_and_strict_writes() {
    check(
        r"let r=/^(?<x>\p{sc=Grek})$/dmu,c=new RegExp(r,'divy');c.source==='^(?<x>\\p{sc=Grek})$'&&c.unicodeSets&&!c.unicode&&!c.multiline&&c.exec('α').groups.x==='α'&&c.lastIndex===1",
    );
    check(
        r"let t='',r=/\p{Script=Greek}/uy;r.lastIndex={valueOf(){t+='i';return 1}};let m=r.exec({toString(){t+='s';return '𐅀'}});t==='si'&&m[0]==='𐅀'&&m.index===0&&r.lastIndex===2",
    );
    check(
        r"let r=/\p{sc=Greek}/vy;Object.defineProperty(r,'lastIndex',{writable:false});let caught=false;try{r.exec('Α')}catch(e){caught=e instanceof TypeError}caught&&r.lastIndex===0",
    );
    check(
        r"let r=/\P{Script=Grek}/vy;r.lastIndex=1;Object.defineProperty(r,'lastIndex',{writable:false});let caught=false;try{r.exec('𐅀\uD800')}catch(e){caught=e instanceof TypeError}caught&&r.lastIndex===1",
    );
}

#[test]
fn unicode_greek_script_opted_work_and_remaining_sets() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval("let r=/\\p{Script=Greek}/ug,marker=0,text='A'.repeat(10000);r.lastIndex=1")
        .unwrap();
    assert!(matches!(
        realm.eval("try{r.exec(text)}catch{marker=1}finally{marker=2}"),
        Err(Error::Limit { .. })
    ));
    assert_eq!(
        realm.eval("marker===0&&r.lastIndex===1"),
        Ok(Value::Boolean(true))
    );
    check(r"/\p{Script_Extensions=Greek}/u.exec('A')===null");
    check(r"/[\p{sc=Grek}&&A]/v.exec('A')===null");
    check(r"/[\p{sc=Grek}--A]/v.exec('A')===null");
    check(
        r"let s=String.fromCodePoint(0x370),m=/[\p{sc=Grek}--A]/v.exec(s);m!==null&&m.length===1&&m[0]===s&&m.index===0&&m.input===s",
    );
    for source in [
        r"/\p{sc=Grek}+b/u.exec('AA')",
        r"/\p{sc=Grek}A/v.exec('AA')",
    ] {
        assert!(
            crate::common::regexp_oracle::check(&mut Realm::default(), source),
            "{source}"
        );
    }
}

#[test]
fn unicode_greek_script_repetitions_retain_last_capture_and_outside_words() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let r=/\B(?<x>\p{{sc=Grek}})+\B/d{mode}g,m=r.exec('µ𐅀𐅀𐅀µ');m.index===1&&m[0]==='𐅀𐅀𐅀'&&m.groups.x==='𐅀'&&m.indices.groups.x[0]===5&&m.indices.groups.x[1]===7&&r.lastIndex===7"
        ));
    }
}

#[test]
fn unicode_greek_script_former_direct_eval_context_executes_normally() {
    check(
        r#"let marker=0,result;try{result=eval("marker=1; /\\p{Script=Greek}/u.test('α');")}catch{marker=2}finally{marker=3}marker===3&&result===true"#,
    );
}
