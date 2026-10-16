//! Exact Hebrew Script aliases, complements, consumers, and Unicode 18 changes.
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
fn unicode_hebrew_script_aliases_complements_and_fold_modes() {
    check(
        r#"let patterns=["^\\p{Script=Hebr}$", "^\\P{Script=Hebr}$", "^[\\p{Script=Hebr}]$", "^[\\P{Script=Hebr}]$", "^[^\\p{Script=Hebr}]$", "^[^\\P{Script=Hebr}]$"],points=[0, 10, 32, 65, 97, 181, 837, 888, 931, 1025, 1425, 1469, 1470, 1472, 1479, 1488, 1489, 1514, 1520, 1524, 64285, 64286, 64335, 55296, 56320, 126464, 128512, 1114111],flags="u";patterns.map(pattern=>{let r=new RegExp(pattern,flags);return points.map(cp=>r.test(String.fromCodePoint(cp))?'1':'0').join('')}).join('|')==="0000000000111111111111100000|1111111111000000000000011111|0000000000111111111111100000|1111111111000000000000011111|1111111111000000000000011111|0000000000111111111111100000""#,
    );
    check(
        r#"let patterns=["^\\p{Script=Hebr}$", "^\\P{Script=Hebr}$", "^[\\p{Script=Hebr}]$", "^[\\P{Script=Hebr}]$", "^[^\\p{Script=Hebr}]$", "^[^\\P{Script=Hebr}]$"],points=[0, 10, 32, 65, 97, 181, 837, 888, 931, 1025, 1425, 1469, 1470, 1472, 1479, 1488, 1489, 1514, 1520, 1524, 64285, 64286, 64335, 55296, 56320, 126464, 128512, 1114111],flags="ui";patterns.map(pattern=>{let r=new RegExp(pattern,flags);return points.map(cp=>r.test(String.fromCodePoint(cp))?'1':'0').join('')}).join('|')==="0000000000111111111111100000|1111111111000000000000011111|0000000000111111111111100000|1111111111000000000000011111|1111111111000000000000011111|0000000000111111111111100000""#,
    );
    check(
        r#"let patterns=["^\\p{Script=Hebr}$", "^\\P{Script=Hebr}$", "^[\\p{Script=Hebr}]$", "^[\\P{Script=Hebr}]$", "^[^\\p{Script=Hebr}]$", "^[^\\P{Script=Hebr}]$"],points=[0, 10, 32, 65, 97, 181, 837, 888, 931, 1025, 1425, 1469, 1470, 1472, 1479, 1488, 1489, 1514, 1520, 1524, 64285, 64286, 64335, 55296, 56320, 126464, 128512, 1114111],flags="v";patterns.map(pattern=>{let r=new RegExp(pattern,flags);return points.map(cp=>r.test(String.fromCodePoint(cp))?'1':'0').join('')}).join('|')==="0000000000111111111111100000|1111111111000000000000011111|0000000000111111111111100000|1111111111000000000000011111|1111111111000000000000011111|0000000000111111111111100000""#,
    );
    check(
        r#"let patterns=["^\\p{Script=Hebr}$", "^\\P{Script=Hebr}$", "^[\\p{Script=Hebr}]$", "^[\\P{Script=Hebr}]$", "^[^\\p{Script=Hebr}]$", "^[^\\P{Script=Hebr}]$"],points=[0, 10, 32, 65, 97, 181, 837, 888, 931, 1025, 1425, 1469, 1470, 1472, 1479, 1488, 1489, 1514, 1520, 1524, 64285, 64286, 64335, 55296, 56320, 126464, 128512, 1114111],flags="vi";patterns.map(pattern=>{let r=new RegExp(pattern,flags);return points.map(cp=>r.test(String.fromCodePoint(cp))?'1':'0').join('')}).join('|')==="0000000000111111111111100000|1111111111000000000000011111|0000000000111111111111100000|1111111111000000000000011111|1111111111000000000000011111|0000000000111111111111100000""#,
    );
    check(
        r#"let patterns=["^\\p{Script=Hebrew}$", "^\\P{Script=Hebrew}$", "^[\\p{Script=Hebrew}]$", "^[\\P{Script=Hebrew}]$", "^[^\\p{Script=Hebrew}]$", "^[^\\P{Script=Hebrew}]$"],points=[0, 10, 32, 65, 97, 181, 837, 888, 931, 1025, 1425, 1469, 1470, 1472, 1479, 1488, 1489, 1514, 1520, 1524, 64285, 64286, 64335, 55296, 56320, 126464, 128512, 1114111],flags="u";patterns.map(pattern=>{let r=new RegExp(pattern,flags);return points.map(cp=>r.test(String.fromCodePoint(cp))?'1':'0').join('')}).join('|')==="0000000000111111111111100000|1111111111000000000000011111|0000000000111111111111100000|1111111111000000000000011111|1111111111000000000000011111|0000000000111111111111100000""#,
    );
    check(
        r#"let patterns=["^\\p{Script=Hebrew}$", "^\\P{Script=Hebrew}$", "^[\\p{Script=Hebrew}]$", "^[\\P{Script=Hebrew}]$", "^[^\\p{Script=Hebrew}]$", "^[^\\P{Script=Hebrew}]$"],points=[0, 10, 32, 65, 97, 181, 837, 888, 931, 1025, 1425, 1469, 1470, 1472, 1479, 1488, 1489, 1514, 1520, 1524, 64285, 64286, 64335, 55296, 56320, 126464, 128512, 1114111],flags="ui";patterns.map(pattern=>{let r=new RegExp(pattern,flags);return points.map(cp=>r.test(String.fromCodePoint(cp))?'1':'0').join('')}).join('|')==="0000000000111111111111100000|1111111111000000000000011111|0000000000111111111111100000|1111111111000000000000011111|1111111111000000000000011111|0000000000111111111111100000""#,
    );
    check(
        r#"let patterns=["^\\p{Script=Hebrew}$", "^\\P{Script=Hebrew}$", "^[\\p{Script=Hebrew}]$", "^[\\P{Script=Hebrew}]$", "^[^\\p{Script=Hebrew}]$", "^[^\\P{Script=Hebrew}]$"],points=[0, 10, 32, 65, 97, 181, 837, 888, 931, 1025, 1425, 1469, 1470, 1472, 1479, 1488, 1489, 1514, 1520, 1524, 64285, 64286, 64335, 55296, 56320, 126464, 128512, 1114111],flags="v";patterns.map(pattern=>{let r=new RegExp(pattern,flags);return points.map(cp=>r.test(String.fromCodePoint(cp))?'1':'0').join('')}).join('|')==="0000000000111111111111100000|1111111111000000000000011111|0000000000111111111111100000|1111111111000000000000011111|1111111111000000000000011111|0000000000111111111111100000""#,
    );
    check(
        r#"let patterns=["^\\p{Script=Hebrew}$", "^\\P{Script=Hebrew}$", "^[\\p{Script=Hebrew}]$", "^[\\P{Script=Hebrew}]$", "^[^\\p{Script=Hebrew}]$", "^[^\\P{Script=Hebrew}]$"],points=[0, 10, 32, 65, 97, 181, 837, 888, 931, 1025, 1425, 1469, 1470, 1472, 1479, 1488, 1489, 1514, 1520, 1524, 64285, 64286, 64335, 55296, 56320, 126464, 128512, 1114111],flags="vi";patterns.map(pattern=>{let r=new RegExp(pattern,flags);return points.map(cp=>r.test(String.fromCodePoint(cp))?'1':'0').join('')}).join('|')==="0000000000111111111111100000|1111111111000000000000011111|0000000000111111111111100000|1111111111000000000000011111|1111111111000000000000011111|0000000000111111111111100000""#,
    );
    check(
        r#"let patterns=["^\\p{sc=Hebr}$", "^\\P{sc=Hebr}$", "^[\\p{sc=Hebr}]$", "^[\\P{sc=Hebr}]$", "^[^\\p{sc=Hebr}]$", "^[^\\P{sc=Hebr}]$"],points=[0, 10, 32, 65, 97, 181, 837, 888, 931, 1025, 1425, 1469, 1470, 1472, 1479, 1488, 1489, 1514, 1520, 1524, 64285, 64286, 64335, 55296, 56320, 126464, 128512, 1114111],flags="u";patterns.map(pattern=>{let r=new RegExp(pattern,flags);return points.map(cp=>r.test(String.fromCodePoint(cp))?'1':'0').join('')}).join('|')==="0000000000111111111111100000|1111111111000000000000011111|0000000000111111111111100000|1111111111000000000000011111|1111111111000000000000011111|0000000000111111111111100000""#,
    );
    check(
        r#"let patterns=["^\\p{sc=Hebr}$", "^\\P{sc=Hebr}$", "^[\\p{sc=Hebr}]$", "^[\\P{sc=Hebr}]$", "^[^\\p{sc=Hebr}]$", "^[^\\P{sc=Hebr}]$"],points=[0, 10, 32, 65, 97, 181, 837, 888, 931, 1025, 1425, 1469, 1470, 1472, 1479, 1488, 1489, 1514, 1520, 1524, 64285, 64286, 64335, 55296, 56320, 126464, 128512, 1114111],flags="ui";patterns.map(pattern=>{let r=new RegExp(pattern,flags);return points.map(cp=>r.test(String.fromCodePoint(cp))?'1':'0').join('')}).join('|')==="0000000000111111111111100000|1111111111000000000000011111|0000000000111111111111100000|1111111111000000000000011111|1111111111000000000000011111|0000000000111111111111100000""#,
    );
    check(
        r#"let patterns=["^\\p{sc=Hebr}$", "^\\P{sc=Hebr}$", "^[\\p{sc=Hebr}]$", "^[\\P{sc=Hebr}]$", "^[^\\p{sc=Hebr}]$", "^[^\\P{sc=Hebr}]$"],points=[0, 10, 32, 65, 97, 181, 837, 888, 931, 1025, 1425, 1469, 1470, 1472, 1479, 1488, 1489, 1514, 1520, 1524, 64285, 64286, 64335, 55296, 56320, 126464, 128512, 1114111],flags="v";patterns.map(pattern=>{let r=new RegExp(pattern,flags);return points.map(cp=>r.test(String.fromCodePoint(cp))?'1':'0').join('')}).join('|')==="0000000000111111111111100000|1111111111000000000000011111|0000000000111111111111100000|1111111111000000000000011111|1111111111000000000000011111|0000000000111111111111100000""#,
    );
    check(
        r#"let patterns=["^\\p{sc=Hebr}$", "^\\P{sc=Hebr}$", "^[\\p{sc=Hebr}]$", "^[\\P{sc=Hebr}]$", "^[^\\p{sc=Hebr}]$", "^[^\\P{sc=Hebr}]$"],points=[0, 10, 32, 65, 97, 181, 837, 888, 931, 1025, 1425, 1469, 1470, 1472, 1479, 1488, 1489, 1514, 1520, 1524, 64285, 64286, 64335, 55296, 56320, 126464, 128512, 1114111],flags="vi";patterns.map(pattern=>{let r=new RegExp(pattern,flags);return points.map(cp=>r.test(String.fromCodePoint(cp))?'1':'0').join('')}).join('|')==="0000000000111111111111100000|1111111111000000000000011111|0000000000111111111111100000|1111111111000000000000011111|1111111111000000000000011111|0000000000111111111111100000""#,
    );
    check(
        r#"let patterns=["^\\p{sc=Hebrew}$", "^\\P{sc=Hebrew}$", "^[\\p{sc=Hebrew}]$", "^[\\P{sc=Hebrew}]$", "^[^\\p{sc=Hebrew}]$", "^[^\\P{sc=Hebrew}]$"],points=[0, 10, 32, 65, 97, 181, 837, 888, 931, 1025, 1425, 1469, 1470, 1472, 1479, 1488, 1489, 1514, 1520, 1524, 64285, 64286, 64335, 55296, 56320, 126464, 128512, 1114111],flags="u";patterns.map(pattern=>{let r=new RegExp(pattern,flags);return points.map(cp=>r.test(String.fromCodePoint(cp))?'1':'0').join('')}).join('|')==="0000000000111111111111100000|1111111111000000000000011111|0000000000111111111111100000|1111111111000000000000011111|1111111111000000000000011111|0000000000111111111111100000""#,
    );
    check(
        r#"let patterns=["^\\p{sc=Hebrew}$", "^\\P{sc=Hebrew}$", "^[\\p{sc=Hebrew}]$", "^[\\P{sc=Hebrew}]$", "^[^\\p{sc=Hebrew}]$", "^[^\\P{sc=Hebrew}]$"],points=[0, 10, 32, 65, 97, 181, 837, 888, 931, 1025, 1425, 1469, 1470, 1472, 1479, 1488, 1489, 1514, 1520, 1524, 64285, 64286, 64335, 55296, 56320, 126464, 128512, 1114111],flags="ui";patterns.map(pattern=>{let r=new RegExp(pattern,flags);return points.map(cp=>r.test(String.fromCodePoint(cp))?'1':'0').join('')}).join('|')==="0000000000111111111111100000|1111111111000000000000011111|0000000000111111111111100000|1111111111000000000000011111|1111111111000000000000011111|0000000000111111111111100000""#,
    );
    check(
        r#"let patterns=["^\\p{sc=Hebrew}$", "^\\P{sc=Hebrew}$", "^[\\p{sc=Hebrew}]$", "^[\\P{sc=Hebrew}]$", "^[^\\p{sc=Hebrew}]$", "^[^\\P{sc=Hebrew}]$"],points=[0, 10, 32, 65, 97, 181, 837, 888, 931, 1025, 1425, 1469, 1470, 1472, 1479, 1488, 1489, 1514, 1520, 1524, 64285, 64286, 64335, 55296, 56320, 126464, 128512, 1114111],flags="v";patterns.map(pattern=>{let r=new RegExp(pattern,flags);return points.map(cp=>r.test(String.fromCodePoint(cp))?'1':'0').join('')}).join('|')==="0000000000111111111111100000|1111111111000000000000011111|0000000000111111111111100000|1111111111000000000000011111|1111111111000000000000011111|0000000000111111111111100000""#,
    );
    check(
        r#"let patterns=["^\\p{sc=Hebrew}$", "^\\P{sc=Hebrew}$", "^[\\p{sc=Hebrew}]$", "^[\\P{sc=Hebrew}]$", "^[^\\p{sc=Hebrew}]$", "^[^\\P{sc=Hebrew}]$"],points=[0, 10, 32, 65, 97, 181, 837, 888, 931, 1025, 1425, 1469, 1470, 1472, 1479, 1488, 1489, 1514, 1520, 1524, 64285, 64286, 64335, 55296, 56320, 126464, 128512, 1114111],flags="vi";patterns.map(pattern=>{let r=new RegExp(pattern,flags);return points.map(cp=>r.test(String.fromCodePoint(cp))?'1':'0').join('')}).join('|')==="0000000000111111111111100000|1111111111000000000000011111|0000000000111111111111100000|1111111111000000000000011111|1111111111000000000000011111|0000000000111111111111100000""#,
    );
}

#[test]
fn unicode_hebrew_script_unicode18_additions() {
    check(
        r#"let patterns=["^\\p{sc=Hebr}$", "^\\P{sc=Hebr}$", "^[\\p{sc=Hebr}]$", "^[\\P{sc=Hebr}]$", "^[^\\p{sc=Hebr}]$", "^[^\\P{sc=Hebr}]$"],points=[1480, 1481],flags="u";patterns.map(pattern=>{let r=new RegExp(pattern,flags);return points.map(cp=>r.test(String.fromCodePoint(cp))?'1':'0').join('')}).join('|')==="11|00|11|00|00|11""#,
    );
    check(
        r#"let patterns=["^\\p{sc=Hebr}$", "^\\P{sc=Hebr}$", "^[\\p{sc=Hebr}]$", "^[\\P{sc=Hebr}]$", "^[^\\p{sc=Hebr}]$", "^[^\\P{sc=Hebr}]$"],points=[1480, 1481],flags="ui";patterns.map(pattern=>{let r=new RegExp(pattern,flags);return points.map(cp=>r.test(String.fromCodePoint(cp))?'1':'0').join('')}).join('|')==="11|00|11|00|00|11""#,
    );
    check(
        r#"let patterns=["^\\p{sc=Hebr}$", "^\\P{sc=Hebr}$", "^[\\p{sc=Hebr}]$", "^[\\P{sc=Hebr}]$", "^[^\\p{sc=Hebr}]$", "^[^\\P{sc=Hebr}]$"],points=[1480, 1481],flags="v";patterns.map(pattern=>{let r=new RegExp(pattern,flags);return points.map(cp=>r.test(String.fromCodePoint(cp))?'1':'0').join('')}).join('|')==="11|00|11|00|00|11""#,
    );
    check(
        r#"let patterns=["^\\p{sc=Hebr}$", "^\\P{sc=Hebr}$", "^[\\p{sc=Hebr}]$", "^[\\P{sc=Hebr}]$", "^[^\\p{sc=Hebr}]$", "^[^\\P{sc=Hebr}]$"],points=[1480, 1481],flags="vi";patterns.map(pattern=>{let r=new RegExp(pattern,flags);return points.map(cp=>r.test(String.fromCodePoint(cp))?'1':'0').join('')}).join('|')==="11|00|11|00|00|11""#,
    );
}

#[test]
fn unicode_hebrew_script_results_snapshot() {
    let mut rows = String::new();
    for name in ["Script=Hebrew", "Script=Hebr", "sc=Hebrew", "sc=Hebr"] {
        for body in [
            format!(r"\p{{{name}}}"),
            format!(r"\P{{{name}}}"),
            format!(r"(?<x>[\p{{{name}}}K])"),
        ] {
            let pattern = JsString::from(body.as_str());
            for flags in ["dug", "diug", "divg", "divy"] {
                for input in ["אָיִµΣ😀\u{378}\n", "X\r\nא\u{2028}א", ""] {
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
fn unicode_hebrew_script_captures_context_and_pair_starts() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let r=/^(?<x>([\p{{sc=Hebr}}K]))$/dim{mode}g,m=r.exec('A\r\nא\u2028');m.index===3&&m[1]==='א'&&m[2]==='א'&&m.indices.groups.x===m.indices[1]&&m.indices[2][1]===4&&r.lastIndex===4"
        ));
        check(&format!(
            r"let r=/(?<__proto__>\P{{Script=Hebrew}})/d{mode}y;r.lastIndex=1;let m=r.exec('😀A');m.index===0&&m[1]==='😀'&&m.indices[1][1]===2&&r.lastIndex===2&&Object.getPrototypeOf(m.groups)===null&&m.indices.groups.__proto__===m.indices[1]"
        ));
        check(&format!(
            r"let r=/\p{{sc=Hebrew}}/d{mode}g;r.lastIndex=1;let m=r.exec('😀\uD800א');m.index===3&&m[0]==='א'&&r.lastIndex===4"
        ));
        check(&format!(
            r"let r=/\p{{Script=Hebr}}/{mode}y;r.lastIndex=1;r.exec('😀א')===null&&r.lastIndex===0"
        ));
    }
}

#[test]
fn unicode_hebrew_script_consumers_preserve_original_text() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let a=[...'א😀Aב'.matchAll(/(?<x>\P{{sc=Hebrew}})/d{mode}g)];a.length===2&&a[0][1]==='😀'&&a[0].index===1&&a[0].indices[1][1]===3&&a[1][1]==='A'&&a[1].index===3"
        ));
        check(&format!(
            r"'אבA'.replace(/(?<x>\p{{sc=Hebr}})/{mode}g,'<$1:$<x>>')==='<א:א><ב:ב>A'"
        ));
        check(&format!(
            r"'AאבA'.split(/(\p{{Script=Hebrew}})/{mode}).join('|')==='A|א||ב|A'&&'A😀א'.search(/\p{{sc=Hebr}}/{mode})===3&&'אבA'.match(/\p{{Script=Hebr}}/{mode}g).join('')==='אב'"
        ));
        check(&format!(
            r"let t='',x='אבAא'.replace(/(\p{{sc=Hebrew}})/{mode}g,(m,c,p)=>{{t+=p;return '_'}});x==='__A_'&&t==='013'"
        ));
    }
}

#[test]
fn unicode_hebrew_script_clones_coercions_and_strict_writes() {
    check(
        r"let r=/^(?<x>\p{sc=Hebr})$/dmu,c=new RegExp(r,'divy');c.source==='^(?<x>\\p{sc=Hebr})$'&&c.unicodeSets&&!c.unicode&&!c.multiline&&c.exec('א').groups.x==='א'&&c.lastIndex===1",
    );
    check(
        r"let t='',r=/\P{Script=Hebrew}/uy;r.lastIndex={valueOf(){t+='i';return 1}};let m=r.exec({toString(){t+='s';return '😀'}});t==='si'&&m[0]==='😀'&&m.index===0&&r.lastIndex===2",
    );
    check(
        r"let r=/\p{sc=Hebrew}/vy;Object.defineProperty(r,'lastIndex',{writable:false});let caught=false;try{r.exec('א')}catch(e){caught=e instanceof TypeError}caught&&r.lastIndex===0",
    );
    check(
        r"let r=/\P{Script=Hebr}/vy;r.lastIndex=1;Object.defineProperty(r,'lastIndex',{writable:false});let caught=false;try{r.exec('😀\uD800')}catch(e){caught=e instanceof TypeError}caught&&r.lastIndex===1",
    );
}

#[test]
fn unicode_hebrew_script_repeated_sources_captures_and_collection() {
    assert_eq!(Limits::default().max_steps, None);
    assert_eq!(Limits::default().max_heap_entries, None);
    let mut realm = Realm::default();
    realm.eval("let p='^'+'('.repeat(3000)+'['+'\\\\p{sc=Hebr}\\\\p{Cs}\\\\p{ASCII}'.repeat(10000)+']'+')'.repeat(3000)+'$',r=new RegExp(p,'dmvg'),c=new RegExp(r),text='A'.repeat(100000)+'\\nא'").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("let m=c.exec(text);m.length===3001&&m[1]==='א'&&m[3000]==='א'&&m.indices[3000][0]===100001&&m.indices[3000][1]===100002&&c.lastIndex===100002"), Ok(Value::Boolean(true)));
    realm.eval("p=null;r=null;c=null;text=null;m=null").unwrap();
    realm.collect(usize::MAX).unwrap();
}

#[test]
fn unicode_hebrew_script_opted_work_and_remaining_sets() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval("let r=/\\p{Script=Hebrew}/ug,marker=0,text='A'.repeat(10000);r.lastIndex=1")
        .unwrap();
    assert!(matches!(
        realm.eval("try{r.exec(text)}catch{marker=1}finally{marker=2}"),
        Err(Error::Limit { .. })
    ));
    assert_eq!(
        realm.eval("marker===0&&r.lastIndex===1"),
        Ok(Value::Boolean(true))
    );
    check(r"/\p{Script_Extensions=Hebrew}/u.exec('A')===null");
    check(r"/[\p{sc=Hebr}&&A]/v.exec('A')===null");
    check(r"/[\p{sc=Hebr}--A]/v.exec('A')===null");
    check(
        r"let s=String.fromCodePoint(0x591),m=/[\p{sc=Hebr}--A]/v.exec(s);m!==null&&m.length===1&&m[0]===s&&m.index===0&&m.input===s",
    );
    for source in [
        r"/\p{sc=Hebr}+b/u.exec('AA')",
        r"/\p{sc=Hebr}A/v.exec('AA')",
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
fn unicode_hebrew_script_repetitions_retain_last_capture_and_outside_words() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let r=/\B(?<x>\p{{sc=Hebr}})+\B/d{mode}g,m=r.exec('µאאאµ');m.index===1&&m[0]==='אאא'&&m.groups.x==='א'&&m.indices.groups.x[0]===3&&m.indices.groups.x[1]===4&&r.lastIndex===4"
        ));
    }
}

#[test]
fn unicode_hebrew_script_former_direct_eval_context_executes_normally() {
    check(
        r#"let marker=0,result;try{result=eval("marker=1; /\\p{Script=Hebrew}/u.test('א');")}catch{marker=2}finally{marker=3}marker===3&&result===true"#,
    );
}
