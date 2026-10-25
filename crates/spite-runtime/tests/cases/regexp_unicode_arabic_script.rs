//! Exact Arabic Script aliases, complements, consumers, and Unicode 18 changes.
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
fn unicode_arabic_script_aliases_complements_and_fold_modes() {
    check(
        r#"let patterns=["^\\p{Script=Arab}$", "^\\P{Script=Arab}$", "^[\\p{Script=Arab}]$", "^[\\P{Script=Arab}]$", "^[^\\p{Script=Arab}]$", "^[^\\P{Script=Arab}]$"],points=[0, 10, 32, 65, 97, 181, 837, 888, 931, 1025, 1536, 1569, 1575, 1610, 1616, 1791, 1872, 2303, 69314, 8192, 55296, 56320, 57344, 64976, 126464, 126719, 128512, 1114111],flags="u";patterns.map(pattern=>{let r=new RegExp(pattern,flags);return points.map(cp=>r.test(String.fromCodePoint(cp))?'1':'0').join('')}).join('|')==="0000000000111101111000001000|1111111111000010000111110111|0000000000111101111000001000|1111111111000010000111110111|1111111111000010000111110111|0000000000111101111000001000""#,
    );
    check(
        r#"let patterns=["^\\p{Script=Arab}$", "^\\P{Script=Arab}$", "^[\\p{Script=Arab}]$", "^[\\P{Script=Arab}]$", "^[^\\p{Script=Arab}]$", "^[^\\P{Script=Arab}]$"],points=[0, 10, 32, 65, 97, 181, 837, 888, 931, 1025, 1536, 1569, 1575, 1610, 1616, 1791, 1872, 2303, 69314, 8192, 55296, 56320, 57344, 64976, 126464, 126719, 128512, 1114111],flags="ui";patterns.map(pattern=>{let r=new RegExp(pattern,flags);return points.map(cp=>r.test(String.fromCodePoint(cp))?'1':'0').join('')}).join('|')==="0000000000111101111000001000|1111111111000010000111110111|0000000000111101111000001000|1111111111000010000111110111|1111111111000010000111110111|0000000000111101111000001000""#,
    );
    check(
        r#"let patterns=["^\\p{Script=Arab}$", "^\\P{Script=Arab}$", "^[\\p{Script=Arab}]$", "^[\\P{Script=Arab}]$", "^[^\\p{Script=Arab}]$", "^[^\\P{Script=Arab}]$"],points=[0, 10, 32, 65, 97, 181, 837, 888, 931, 1025, 1536, 1569, 1575, 1610, 1616, 1791, 1872, 2303, 69314, 8192, 55296, 56320, 57344, 64976, 126464, 126719, 128512, 1114111],flags="v";patterns.map(pattern=>{let r=new RegExp(pattern,flags);return points.map(cp=>r.test(String.fromCodePoint(cp))?'1':'0').join('')}).join('|')==="0000000000111101111000001000|1111111111000010000111110111|0000000000111101111000001000|1111111111000010000111110111|1111111111000010000111110111|0000000000111101111000001000""#,
    );
    check(
        r#"let patterns=["^\\p{Script=Arab}$", "^\\P{Script=Arab}$", "^[\\p{Script=Arab}]$", "^[\\P{Script=Arab}]$", "^[^\\p{Script=Arab}]$", "^[^\\P{Script=Arab}]$"],points=[0, 10, 32, 65, 97, 181, 837, 888, 931, 1025, 1536, 1569, 1575, 1610, 1616, 1791, 1872, 2303, 69314, 8192, 55296, 56320, 57344, 64976, 126464, 126719, 128512, 1114111],flags="vi";patterns.map(pattern=>{let r=new RegExp(pattern,flags);return points.map(cp=>r.test(String.fromCodePoint(cp))?'1':'0').join('')}).join('|')==="0000000000111101111000001000|1111111111000010000111110111|0000000000111101111000001000|1111111111000010000111110111|1111111111000010000111110111|0000000000111101111000001000""#,
    );
    check(
        r#"let patterns=["^\\p{Script=Arabic}$", "^\\P{Script=Arabic}$", "^[\\p{Script=Arabic}]$", "^[\\P{Script=Arabic}]$", "^[^\\p{Script=Arabic}]$", "^[^\\P{Script=Arabic}]$"],points=[0, 10, 32, 65, 97, 181, 837, 888, 931, 1025, 1536, 1569, 1575, 1610, 1616, 1791, 1872, 2303, 69314, 8192, 55296, 56320, 57344, 64976, 126464, 126719, 128512, 1114111],flags="u";patterns.map(pattern=>{let r=new RegExp(pattern,flags);return points.map(cp=>r.test(String.fromCodePoint(cp))?'1':'0').join('')}).join('|')==="0000000000111101111000001000|1111111111000010000111110111|0000000000111101111000001000|1111111111000010000111110111|1111111111000010000111110111|0000000000111101111000001000""#,
    );
    check(
        r#"let patterns=["^\\p{Script=Arabic}$", "^\\P{Script=Arabic}$", "^[\\p{Script=Arabic}]$", "^[\\P{Script=Arabic}]$", "^[^\\p{Script=Arabic}]$", "^[^\\P{Script=Arabic}]$"],points=[0, 10, 32, 65, 97, 181, 837, 888, 931, 1025, 1536, 1569, 1575, 1610, 1616, 1791, 1872, 2303, 69314, 8192, 55296, 56320, 57344, 64976, 126464, 126719, 128512, 1114111],flags="ui";patterns.map(pattern=>{let r=new RegExp(pattern,flags);return points.map(cp=>r.test(String.fromCodePoint(cp))?'1':'0').join('')}).join('|')==="0000000000111101111000001000|1111111111000010000111110111|0000000000111101111000001000|1111111111000010000111110111|1111111111000010000111110111|0000000000111101111000001000""#,
    );
    check(
        r#"let patterns=["^\\p{Script=Arabic}$", "^\\P{Script=Arabic}$", "^[\\p{Script=Arabic}]$", "^[\\P{Script=Arabic}]$", "^[^\\p{Script=Arabic}]$", "^[^\\P{Script=Arabic}]$"],points=[0, 10, 32, 65, 97, 181, 837, 888, 931, 1025, 1536, 1569, 1575, 1610, 1616, 1791, 1872, 2303, 69314, 8192, 55296, 56320, 57344, 64976, 126464, 126719, 128512, 1114111],flags="v";patterns.map(pattern=>{let r=new RegExp(pattern,flags);return points.map(cp=>r.test(String.fromCodePoint(cp))?'1':'0').join('')}).join('|')==="0000000000111101111000001000|1111111111000010000111110111|0000000000111101111000001000|1111111111000010000111110111|1111111111000010000111110111|0000000000111101111000001000""#,
    );
    check(
        r#"let patterns=["^\\p{Script=Arabic}$", "^\\P{Script=Arabic}$", "^[\\p{Script=Arabic}]$", "^[\\P{Script=Arabic}]$", "^[^\\p{Script=Arabic}]$", "^[^\\P{Script=Arabic}]$"],points=[0, 10, 32, 65, 97, 181, 837, 888, 931, 1025, 1536, 1569, 1575, 1610, 1616, 1791, 1872, 2303, 69314, 8192, 55296, 56320, 57344, 64976, 126464, 126719, 128512, 1114111],flags="vi";patterns.map(pattern=>{let r=new RegExp(pattern,flags);return points.map(cp=>r.test(String.fromCodePoint(cp))?'1':'0').join('')}).join('|')==="0000000000111101111000001000|1111111111000010000111110111|0000000000111101111000001000|1111111111000010000111110111|1111111111000010000111110111|0000000000111101111000001000""#,
    );
    check(
        r#"let patterns=["^\\p{sc=Arab}$", "^\\P{sc=Arab}$", "^[\\p{sc=Arab}]$", "^[\\P{sc=Arab}]$", "^[^\\p{sc=Arab}]$", "^[^\\P{sc=Arab}]$"],points=[0, 10, 32, 65, 97, 181, 837, 888, 931, 1025, 1536, 1569, 1575, 1610, 1616, 1791, 1872, 2303, 69314, 8192, 55296, 56320, 57344, 64976, 126464, 126719, 128512, 1114111],flags="u";patterns.map(pattern=>{let r=new RegExp(pattern,flags);return points.map(cp=>r.test(String.fromCodePoint(cp))?'1':'0').join('')}).join('|')==="0000000000111101111000001000|1111111111000010000111110111|0000000000111101111000001000|1111111111000010000111110111|1111111111000010000111110111|0000000000111101111000001000""#,
    );
    check(
        r#"let patterns=["^\\p{sc=Arab}$", "^\\P{sc=Arab}$", "^[\\p{sc=Arab}]$", "^[\\P{sc=Arab}]$", "^[^\\p{sc=Arab}]$", "^[^\\P{sc=Arab}]$"],points=[0, 10, 32, 65, 97, 181, 837, 888, 931, 1025, 1536, 1569, 1575, 1610, 1616, 1791, 1872, 2303, 69314, 8192, 55296, 56320, 57344, 64976, 126464, 126719, 128512, 1114111],flags="ui";patterns.map(pattern=>{let r=new RegExp(pattern,flags);return points.map(cp=>r.test(String.fromCodePoint(cp))?'1':'0').join('')}).join('|')==="0000000000111101111000001000|1111111111000010000111110111|0000000000111101111000001000|1111111111000010000111110111|1111111111000010000111110111|0000000000111101111000001000""#,
    );
    check(
        r#"let patterns=["^\\p{sc=Arab}$", "^\\P{sc=Arab}$", "^[\\p{sc=Arab}]$", "^[\\P{sc=Arab}]$", "^[^\\p{sc=Arab}]$", "^[^\\P{sc=Arab}]$"],points=[0, 10, 32, 65, 97, 181, 837, 888, 931, 1025, 1536, 1569, 1575, 1610, 1616, 1791, 1872, 2303, 69314, 8192, 55296, 56320, 57344, 64976, 126464, 126719, 128512, 1114111],flags="v";patterns.map(pattern=>{let r=new RegExp(pattern,flags);return points.map(cp=>r.test(String.fromCodePoint(cp))?'1':'0').join('')}).join('|')==="0000000000111101111000001000|1111111111000010000111110111|0000000000111101111000001000|1111111111000010000111110111|1111111111000010000111110111|0000000000111101111000001000""#,
    );
    check(
        r#"let patterns=["^\\p{sc=Arab}$", "^\\P{sc=Arab}$", "^[\\p{sc=Arab}]$", "^[\\P{sc=Arab}]$", "^[^\\p{sc=Arab}]$", "^[^\\P{sc=Arab}]$"],points=[0, 10, 32, 65, 97, 181, 837, 888, 931, 1025, 1536, 1569, 1575, 1610, 1616, 1791, 1872, 2303, 69314, 8192, 55296, 56320, 57344, 64976, 126464, 126719, 128512, 1114111],flags="vi";patterns.map(pattern=>{let r=new RegExp(pattern,flags);return points.map(cp=>r.test(String.fromCodePoint(cp))?'1':'0').join('')}).join('|')==="0000000000111101111000001000|1111111111000010000111110111|0000000000111101111000001000|1111111111000010000111110111|1111111111000010000111110111|0000000000111101111000001000""#,
    );
    check(
        r#"let patterns=["^\\p{sc=Arabic}$", "^\\P{sc=Arabic}$", "^[\\p{sc=Arabic}]$", "^[\\P{sc=Arabic}]$", "^[^\\p{sc=Arabic}]$", "^[^\\P{sc=Arabic}]$"],points=[0, 10, 32, 65, 97, 181, 837, 888, 931, 1025, 1536, 1569, 1575, 1610, 1616, 1791, 1872, 2303, 69314, 8192, 55296, 56320, 57344, 64976, 126464, 126719, 128512, 1114111],flags="u";patterns.map(pattern=>{let r=new RegExp(pattern,flags);return points.map(cp=>r.test(String.fromCodePoint(cp))?'1':'0').join('')}).join('|')==="0000000000111101111000001000|1111111111000010000111110111|0000000000111101111000001000|1111111111000010000111110111|1111111111000010000111110111|0000000000111101111000001000""#,
    );
    check(
        r#"let patterns=["^\\p{sc=Arabic}$", "^\\P{sc=Arabic}$", "^[\\p{sc=Arabic}]$", "^[\\P{sc=Arabic}]$", "^[^\\p{sc=Arabic}]$", "^[^\\P{sc=Arabic}]$"],points=[0, 10, 32, 65, 97, 181, 837, 888, 931, 1025, 1536, 1569, 1575, 1610, 1616, 1791, 1872, 2303, 69314, 8192, 55296, 56320, 57344, 64976, 126464, 126719, 128512, 1114111],flags="ui";patterns.map(pattern=>{let r=new RegExp(pattern,flags);return points.map(cp=>r.test(String.fromCodePoint(cp))?'1':'0').join('')}).join('|')==="0000000000111101111000001000|1111111111000010000111110111|0000000000111101111000001000|1111111111000010000111110111|1111111111000010000111110111|0000000000111101111000001000""#,
    );
    check(
        r#"let patterns=["^\\p{sc=Arabic}$", "^\\P{sc=Arabic}$", "^[\\p{sc=Arabic}]$", "^[\\P{sc=Arabic}]$", "^[^\\p{sc=Arabic}]$", "^[^\\P{sc=Arabic}]$"],points=[0, 10, 32, 65, 97, 181, 837, 888, 931, 1025, 1536, 1569, 1575, 1610, 1616, 1791, 1872, 2303, 69314, 8192, 55296, 56320, 57344, 64976, 126464, 126719, 128512, 1114111],flags="v";patterns.map(pattern=>{let r=new RegExp(pattern,flags);return points.map(cp=>r.test(String.fromCodePoint(cp))?'1':'0').join('')}).join('|')==="0000000000111101111000001000|1111111111000010000111110111|0000000000111101111000001000|1111111111000010000111110111|1111111111000010000111110111|0000000000111101111000001000""#,
    );
    check(
        r#"let patterns=["^\\p{sc=Arabic}$", "^\\P{sc=Arabic}$", "^[\\p{sc=Arabic}]$", "^[\\P{sc=Arabic}]$", "^[^\\p{sc=Arabic}]$", "^[^\\P{sc=Arabic}]$"],points=[0, 10, 32, 65, 97, 181, 837, 888, 931, 1025, 1536, 1569, 1575, 1610, 1616, 1791, 1872, 2303, 69314, 8192, 55296, 56320, 57344, 64976, 126464, 126719, 128512, 1114111],flags="vi";patterns.map(pattern=>{let r=new RegExp(pattern,flags);return points.map(cp=>r.test(String.fromCodePoint(cp))?'1':'0').join('')}).join('|')==="0000000000111101111000001000|1111111111000010000111110111|0000000000111101111000001000|1111111111000010000111110111|1111111111000010000111110111|0000000000111101111000001000""#,
    );
}

#[test]
fn unicode_arabic_script_unicode18_additions() {
    check(
        r#"let patterns=["^\\p{sc=Arab}$", "^\\P{sc=Arab}$", "^[\\p{sc=Arab}]$", "^[\\P{sc=Arab}]$", "^[^\\p{sc=Arab}]$", "^[^\\P{sc=Arab}]$"],points=[69321, 69322, 69323, 69324, 69325, 69326, 69327, 69337, 69338, 69339, 69340, 69341, 69342, 69343, 69344, 69345, 69346, 69347, 69348, 69349, 69350, 69351, 69352, 69353, 69354, 69355, 69356, 69357, 69358, 69360, 69361, 69362, 69363, 69364, 69365, 69366, 69367, 69368, 69369],flags="u";patterns.map(pattern=>{let r=new RegExp(pattern,flags);return points.map(cp=>r.test(String.fromCodePoint(cp))?'1':'0').join('')}).join('|')==="111111111111111111111111111111111111111|000000000000000000000000000000000000000|111111111111111111111111111111111111111|000000000000000000000000000000000000000|000000000000000000000000000000000000000|111111111111111111111111111111111111111""#,
    );
    check(
        r#"let patterns=["^\\p{sc=Arab}$", "^\\P{sc=Arab}$", "^[\\p{sc=Arab}]$", "^[\\P{sc=Arab}]$", "^[^\\p{sc=Arab}]$", "^[^\\P{sc=Arab}]$"],points=[69321, 69322, 69323, 69324, 69325, 69326, 69327, 69337, 69338, 69339, 69340, 69341, 69342, 69343, 69344, 69345, 69346, 69347, 69348, 69349, 69350, 69351, 69352, 69353, 69354, 69355, 69356, 69357, 69358, 69360, 69361, 69362, 69363, 69364, 69365, 69366, 69367, 69368, 69369],flags="ui";patterns.map(pattern=>{let r=new RegExp(pattern,flags);return points.map(cp=>r.test(String.fromCodePoint(cp))?'1':'0').join('')}).join('|')==="111111111111111111111111111111111111111|000000000000000000000000000000000000000|111111111111111111111111111111111111111|000000000000000000000000000000000000000|000000000000000000000000000000000000000|111111111111111111111111111111111111111""#,
    );
    check(
        r#"let patterns=["^\\p{sc=Arab}$", "^\\P{sc=Arab}$", "^[\\p{sc=Arab}]$", "^[\\P{sc=Arab}]$", "^[^\\p{sc=Arab}]$", "^[^\\P{sc=Arab}]$"],points=[69321, 69322, 69323, 69324, 69325, 69326, 69327, 69337, 69338, 69339, 69340, 69341, 69342, 69343, 69344, 69345, 69346, 69347, 69348, 69349, 69350, 69351, 69352, 69353, 69354, 69355, 69356, 69357, 69358, 69360, 69361, 69362, 69363, 69364, 69365, 69366, 69367, 69368, 69369],flags="v";patterns.map(pattern=>{let r=new RegExp(pattern,flags);return points.map(cp=>r.test(String.fromCodePoint(cp))?'1':'0').join('')}).join('|')==="111111111111111111111111111111111111111|000000000000000000000000000000000000000|111111111111111111111111111111111111111|000000000000000000000000000000000000000|000000000000000000000000000000000000000|111111111111111111111111111111111111111""#,
    );
    check(
        r#"let patterns=["^\\p{sc=Arab}$", "^\\P{sc=Arab}$", "^[\\p{sc=Arab}]$", "^[\\P{sc=Arab}]$", "^[^\\p{sc=Arab}]$", "^[^\\P{sc=Arab}]$"],points=[69321, 69322, 69323, 69324, 69325, 69326, 69327, 69337, 69338, 69339, 69340, 69341, 69342, 69343, 69344, 69345, 69346, 69347, 69348, 69349, 69350, 69351, 69352, 69353, 69354, 69355, 69356, 69357, 69358, 69360, 69361, 69362, 69363, 69364, 69365, 69366, 69367, 69368, 69369],flags="vi";patterns.map(pattern=>{let r=new RegExp(pattern,flags);return points.map(cp=>r.test(String.fromCodePoint(cp))?'1':'0').join('')}).join('|')==="111111111111111111111111111111111111111|000000000000000000000000000000000000000|111111111111111111111111111111111111111|000000000000000000000000000000000000000|000000000000000000000000000000000000000|111111111111111111111111111111111111111""#,
    );
}

#[test]
fn unicode_arabic_script_results_snapshot() {
    let mut rows = String::new();
    for name in ["Script=Arabic", "Script=Arab", "sc=Arabic", "sc=Arab"] {
        for body in [
            format!(r"\p{{{name}}}"),
            format!(r"\P{{{name}}}"),
            format!(r"(?<x>[\p{{{name}}}K])"),
        ] {
            let pattern = JsString::from(body.as_str());
            for flags in ["dug", "diug", "divg", "divy"] {
                for input in ["ءايµΣ😀\u{378}\n", "X\r\nء\u{2028}𞸀", ""] {
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
fn unicode_arabic_script_captures_context_and_pair_starts() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let r=/^(?<x>([\p{{sc=Arab}}K]))$/dim{mode}g,m=r.exec('A\r\nء\u2028');m.index===3&&m[1]==='ء'&&m[2]==='ء'&&m.indices.groups.x===m.indices[1]&&m.indices[2][1]===4&&r.lastIndex===4"
        ));
        check(&format!(
            r"let r=/(?<__proto__>\p{{Script=Arabic}})/d{mode}y;r.lastIndex=1;let m=r.exec('𞸀A');m.index===0&&m[1]==='𞸀'&&m.indices[1][1]===2&&r.lastIndex===2&&Object.getPrototypeOf(m.groups)===null&&m.indices.groups.__proto__===m.indices[1]"
        ));
        check(&format!(
            r"let r=/\P{{sc=Arabic}}/d{mode}g;r.lastIndex=1;let m=r.exec('𞸀\uD800');m.index===2&&m[0]==='\uD800'&&r.lastIndex===3"
        ));
        check(&format!(
            r"let r=/\P{{Script=Arab}}/{mode}y;r.lastIndex=1;r.exec('𞸀\uD800')===null&&r.lastIndex===0"
        ));
    }
}

#[test]
fn unicode_arabic_script_consumers_preserve_original_text() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let a=[...'ا😀Aب'.matchAll(/(?<x>\P{{sc=Arabic}})/d{mode}g)];a.length===2&&a[0][1]==='😀'&&a[0].index===1&&a[0].indices[1][1]===3&&a[1][1]==='A'&&a[1].index===3"
        ));
        check(&format!(
            r"'ابA'.replace(/(?<x>\p{{sc=Arab}})/{mode}g,'<$1:$<x>>')==='<ا:ا><ب:ب>A'"
        ));
        check(&format!(
            r"'AابA'.split(/(\p{{Script=Arabic}})/{mode}).join('|')==='A|ا||ب|A'&&'A😀ا'.search(/\p{{sc=Arab}}/{mode})===3&&'ابA'.match(/\p{{Script=Arab}}/{mode}g).join('')==='اب'"
        ));
        check(&format!(
            r"let t='',x='ابAا'.replace(/(\p{{sc=Arabic}})/{mode}g,(m,c,p)=>{{t+=p;return '_'}});x==='__A_'&&t==='013'"
        ));
    }
}

#[test]
fn unicode_arabic_script_clones_coercions_and_strict_writes() {
    check(
        r"let r=/^(?<x>\p{sc=Arab})$/dmu,c=new RegExp(r,'divy');c.source==='^(?<x>\\p{sc=Arab})$'&&c.unicodeSets&&!c.unicode&&!c.multiline&&c.exec('ا').groups.x==='ا'&&c.lastIndex===1",
    );
    check(
        r"let t='',r=/\p{Script=Arabic}/uy;r.lastIndex={valueOf(){t+='i';return 1}};let m=r.exec({toString(){t+='s';return '𞸀'}});t==='si'&&m[0]==='𞸀'&&m.index===0&&r.lastIndex===2",
    );
    check(
        r"let r=/\p{sc=Arabic}/vy;Object.defineProperty(r,'lastIndex',{writable:false});let caught=false;try{r.exec('ا')}catch(e){caught=e instanceof TypeError}caught&&r.lastIndex===0",
    );
    check(
        r"let r=/\P{Script=Arab}/vy;r.lastIndex=1;Object.defineProperty(r,'lastIndex',{writable:false});let caught=false;try{r.exec('𞸀\uD800')}catch(e){caught=e instanceof TypeError}caught&&r.lastIndex===1",
    );
}

#[test]
fn unicode_arabic_script_opted_work_and_remaining_sets() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval("let r=/\\p{Script=Arabic}/ug,marker=0,text='A'.repeat(10000);r.lastIndex=1")
        .unwrap();
    assert!(matches!(
        realm.eval("try{r.exec(text)}catch{marker=1}finally{marker=2}"),
        Err(Error::Limit { .. })
    ));
    assert_eq!(
        realm.eval("marker===0&&r.lastIndex===1"),
        Ok(Value::Boolean(true))
    );
    check(r"/\p{Script_Extensions=Arabic}/u.exec('A')===null");
    check(r"/[\p{sc=Arab}&&A]/v.exec('A')===null");
    check(r"/[\p{sc=Arab}--A]/v.exec('A')===null");
    check(
        r"let s=String.fromCodePoint(0x600),m=/[\p{sc=Arab}--A]/v.exec(s);m!==null&&m.length===1&&m[0]===s&&m.index===0&&m.input===s",
    );
    for source in [
        r"/\p{sc=Arab}+b/u.exec('AA')",
        r"/\p{sc=Arab}A/v.exec('AA')",
    ] {
        assert!(
            crate::common::regexp_oracle::check(&mut Realm::default(), source),
            "{source}"
        );
    }
}

#[test]
fn unicode_arabic_script_repetitions_retain_last_capture_and_outside_words() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let r=/\B(?<x>\p{{sc=Arab}})+\B/d{mode}g,m=r.exec('µ𞸀𞸀𞸀µ');m.index===1&&m[0]==='𞸀𞸀𞸀'&&m.groups.x==='𞸀'&&m.indices.groups.x[0]===5&&m.indices.groups.x[1]===7&&r.lastIndex===7"
        ));
    }
}

#[test]
fn unicode_arabic_script_former_direct_eval_context_executes_normally() {
    check(
        r#"let marker=0,result;try{result=eval("marker=1; /\\p{Script=Arabic}/u.test('ا');")}catch{marker=2}finally{marker=3}marker===3&&result===true"#,
    );
}
