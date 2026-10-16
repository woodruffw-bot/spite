//! Exact Cyrillic Script aliases, complements, consumers, and unchanged pinned membership.
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
fn unicode_cyrillic_script_aliases_complements_and_fold_modes() {
    check(
        r#"let patterns=["^\\p{Script=Cyrl}$", "^\\P{Script=Cyrl}$", "^[\\p{Script=Cyrl}]$", "^[\\P{Script=Cyrl}]$", "^[^\\p{Script=Cyrl}]$", "^[^\\P{Script=Cyrl}]$"],points=[0, 10, 32, 65, 97, 181, 837, 888, 931, 1024, 1025, 1040, 1072, 1105, 1327, 1328, 7296, 7305, 7306, 7467, 8192, 55296, 56320, 57344, 64976, 122928, 122989, 128512, 1114111],flags="u";patterns.map(pattern=>{let r=new RegExp(pattern,flags);return points.map(cp=>r.test(String.fromCodePoint(cp))?'1':'0').join('')}).join('|')==="00000000011111101111000001100|11111111100000010000111110011|00000000011111101111000001100|11111111100000010000111110011|11111111100000010000111110011|00000000011111101111000001100""#,
    );
    check(
        r#"let patterns=["^\\p{Script=Cyrl}$", "^\\P{Script=Cyrl}$", "^[\\p{Script=Cyrl}]$", "^[\\P{Script=Cyrl}]$", "^[^\\p{Script=Cyrl}]$", "^[^\\P{Script=Cyrl}]$"],points=[0, 10, 32, 65, 97, 181, 837, 888, 931, 1024, 1025, 1040, 1072, 1105, 1327, 1328, 7296, 7305, 7306, 7467, 8192, 55296, 56320, 57344, 64976, 122928, 122989, 128512, 1114111],flags="ui";patterns.map(pattern=>{let r=new RegExp(pattern,flags);return points.map(cp=>r.test(String.fromCodePoint(cp))?'1':'0').join('')}).join('|')==="00000000011111101111000001100|11111111100000010000111110011|00000000011111101111000001100|11111111100000010000111110011|11111111100000010000111110011|00000000011111101111000001100""#,
    );
    check(
        r#"let patterns=["^\\p{Script=Cyrl}$", "^\\P{Script=Cyrl}$", "^[\\p{Script=Cyrl}]$", "^[\\P{Script=Cyrl}]$", "^[^\\p{Script=Cyrl}]$", "^[^\\P{Script=Cyrl}]$"],points=[0, 10, 32, 65, 97, 181, 837, 888, 931, 1024, 1025, 1040, 1072, 1105, 1327, 1328, 7296, 7305, 7306, 7467, 8192, 55296, 56320, 57344, 64976, 122928, 122989, 128512, 1114111],flags="v";patterns.map(pattern=>{let r=new RegExp(pattern,flags);return points.map(cp=>r.test(String.fromCodePoint(cp))?'1':'0').join('')}).join('|')==="00000000011111101111000001100|11111111100000010000111110011|00000000011111101111000001100|11111111100000010000111110011|11111111100000010000111110011|00000000011111101111000001100""#,
    );
    check(
        r#"let patterns=["^\\p{Script=Cyrl}$", "^\\P{Script=Cyrl}$", "^[\\p{Script=Cyrl}]$", "^[\\P{Script=Cyrl}]$", "^[^\\p{Script=Cyrl}]$", "^[^\\P{Script=Cyrl}]$"],points=[0, 10, 32, 65, 97, 181, 837, 888, 931, 1024, 1025, 1040, 1072, 1105, 1327, 1328, 7296, 7305, 7306, 7467, 8192, 55296, 56320, 57344, 64976, 122928, 122989, 128512, 1114111],flags="vi";patterns.map(pattern=>{let r=new RegExp(pattern,flags);return points.map(cp=>r.test(String.fromCodePoint(cp))?'1':'0').join('')}).join('|')==="00000000011111101111000001100|11111111100000010000111110011|00000000011111101111000001100|11111111100000010000111110011|11111111100000010000111110011|00000000011111101111000001100""#,
    );
    check(
        r#"let patterns=["^\\p{Script=Cyrillic}$", "^\\P{Script=Cyrillic}$", "^[\\p{Script=Cyrillic}]$", "^[\\P{Script=Cyrillic}]$", "^[^\\p{Script=Cyrillic}]$", "^[^\\P{Script=Cyrillic}]$"],points=[0, 10, 32, 65, 97, 181, 837, 888, 931, 1024, 1025, 1040, 1072, 1105, 1327, 1328, 7296, 7305, 7306, 7467, 8192, 55296, 56320, 57344, 64976, 122928, 122989, 128512, 1114111],flags="u";patterns.map(pattern=>{let r=new RegExp(pattern,flags);return points.map(cp=>r.test(String.fromCodePoint(cp))?'1':'0').join('')}).join('|')==="00000000011111101111000001100|11111111100000010000111110011|00000000011111101111000001100|11111111100000010000111110011|11111111100000010000111110011|00000000011111101111000001100""#,
    );
    check(
        r#"let patterns=["^\\p{Script=Cyrillic}$", "^\\P{Script=Cyrillic}$", "^[\\p{Script=Cyrillic}]$", "^[\\P{Script=Cyrillic}]$", "^[^\\p{Script=Cyrillic}]$", "^[^\\P{Script=Cyrillic}]$"],points=[0, 10, 32, 65, 97, 181, 837, 888, 931, 1024, 1025, 1040, 1072, 1105, 1327, 1328, 7296, 7305, 7306, 7467, 8192, 55296, 56320, 57344, 64976, 122928, 122989, 128512, 1114111],flags="ui";patterns.map(pattern=>{let r=new RegExp(pattern,flags);return points.map(cp=>r.test(String.fromCodePoint(cp))?'1':'0').join('')}).join('|')==="00000000011111101111000001100|11111111100000010000111110011|00000000011111101111000001100|11111111100000010000111110011|11111111100000010000111110011|00000000011111101111000001100""#,
    );
    check(
        r#"let patterns=["^\\p{Script=Cyrillic}$", "^\\P{Script=Cyrillic}$", "^[\\p{Script=Cyrillic}]$", "^[\\P{Script=Cyrillic}]$", "^[^\\p{Script=Cyrillic}]$", "^[^\\P{Script=Cyrillic}]$"],points=[0, 10, 32, 65, 97, 181, 837, 888, 931, 1024, 1025, 1040, 1072, 1105, 1327, 1328, 7296, 7305, 7306, 7467, 8192, 55296, 56320, 57344, 64976, 122928, 122989, 128512, 1114111],flags="v";patterns.map(pattern=>{let r=new RegExp(pattern,flags);return points.map(cp=>r.test(String.fromCodePoint(cp))?'1':'0').join('')}).join('|')==="00000000011111101111000001100|11111111100000010000111110011|00000000011111101111000001100|11111111100000010000111110011|11111111100000010000111110011|00000000011111101111000001100""#,
    );
    check(
        r#"let patterns=["^\\p{Script=Cyrillic}$", "^\\P{Script=Cyrillic}$", "^[\\p{Script=Cyrillic}]$", "^[\\P{Script=Cyrillic}]$", "^[^\\p{Script=Cyrillic}]$", "^[^\\P{Script=Cyrillic}]$"],points=[0, 10, 32, 65, 97, 181, 837, 888, 931, 1024, 1025, 1040, 1072, 1105, 1327, 1328, 7296, 7305, 7306, 7467, 8192, 55296, 56320, 57344, 64976, 122928, 122989, 128512, 1114111],flags="vi";patterns.map(pattern=>{let r=new RegExp(pattern,flags);return points.map(cp=>r.test(String.fromCodePoint(cp))?'1':'0').join('')}).join('|')==="00000000011111101111000001100|11111111100000010000111110011|00000000011111101111000001100|11111111100000010000111110011|11111111100000010000111110011|00000000011111101111000001100""#,
    );
    check(
        r#"let patterns=["^\\p{sc=Cyrl}$", "^\\P{sc=Cyrl}$", "^[\\p{sc=Cyrl}]$", "^[\\P{sc=Cyrl}]$", "^[^\\p{sc=Cyrl}]$", "^[^\\P{sc=Cyrl}]$"],points=[0, 10, 32, 65, 97, 181, 837, 888, 931, 1024, 1025, 1040, 1072, 1105, 1327, 1328, 7296, 7305, 7306, 7467, 8192, 55296, 56320, 57344, 64976, 122928, 122989, 128512, 1114111],flags="u";patterns.map(pattern=>{let r=new RegExp(pattern,flags);return points.map(cp=>r.test(String.fromCodePoint(cp))?'1':'0').join('')}).join('|')==="00000000011111101111000001100|11111111100000010000111110011|00000000011111101111000001100|11111111100000010000111110011|11111111100000010000111110011|00000000011111101111000001100""#,
    );
    check(
        r#"let patterns=["^\\p{sc=Cyrl}$", "^\\P{sc=Cyrl}$", "^[\\p{sc=Cyrl}]$", "^[\\P{sc=Cyrl}]$", "^[^\\p{sc=Cyrl}]$", "^[^\\P{sc=Cyrl}]$"],points=[0, 10, 32, 65, 97, 181, 837, 888, 931, 1024, 1025, 1040, 1072, 1105, 1327, 1328, 7296, 7305, 7306, 7467, 8192, 55296, 56320, 57344, 64976, 122928, 122989, 128512, 1114111],flags="ui";patterns.map(pattern=>{let r=new RegExp(pattern,flags);return points.map(cp=>r.test(String.fromCodePoint(cp))?'1':'0').join('')}).join('|')==="00000000011111101111000001100|11111111100000010000111110011|00000000011111101111000001100|11111111100000010000111110011|11111111100000010000111110011|00000000011111101111000001100""#,
    );
    check(
        r#"let patterns=["^\\p{sc=Cyrl}$", "^\\P{sc=Cyrl}$", "^[\\p{sc=Cyrl}]$", "^[\\P{sc=Cyrl}]$", "^[^\\p{sc=Cyrl}]$", "^[^\\P{sc=Cyrl}]$"],points=[0, 10, 32, 65, 97, 181, 837, 888, 931, 1024, 1025, 1040, 1072, 1105, 1327, 1328, 7296, 7305, 7306, 7467, 8192, 55296, 56320, 57344, 64976, 122928, 122989, 128512, 1114111],flags="v";patterns.map(pattern=>{let r=new RegExp(pattern,flags);return points.map(cp=>r.test(String.fromCodePoint(cp))?'1':'0').join('')}).join('|')==="00000000011111101111000001100|11111111100000010000111110011|00000000011111101111000001100|11111111100000010000111110011|11111111100000010000111110011|00000000011111101111000001100""#,
    );
    check(
        r#"let patterns=["^\\p{sc=Cyrl}$", "^\\P{sc=Cyrl}$", "^[\\p{sc=Cyrl}]$", "^[\\P{sc=Cyrl}]$", "^[^\\p{sc=Cyrl}]$", "^[^\\P{sc=Cyrl}]$"],points=[0, 10, 32, 65, 97, 181, 837, 888, 931, 1024, 1025, 1040, 1072, 1105, 1327, 1328, 7296, 7305, 7306, 7467, 8192, 55296, 56320, 57344, 64976, 122928, 122989, 128512, 1114111],flags="vi";patterns.map(pattern=>{let r=new RegExp(pattern,flags);return points.map(cp=>r.test(String.fromCodePoint(cp))?'1':'0').join('')}).join('|')==="00000000011111101111000001100|11111111100000010000111110011|00000000011111101111000001100|11111111100000010000111110011|11111111100000010000111110011|00000000011111101111000001100""#,
    );
    check(
        r#"let patterns=["^\\p{sc=Cyrillic}$", "^\\P{sc=Cyrillic}$", "^[\\p{sc=Cyrillic}]$", "^[\\P{sc=Cyrillic}]$", "^[^\\p{sc=Cyrillic}]$", "^[^\\P{sc=Cyrillic}]$"],points=[0, 10, 32, 65, 97, 181, 837, 888, 931, 1024, 1025, 1040, 1072, 1105, 1327, 1328, 7296, 7305, 7306, 7467, 8192, 55296, 56320, 57344, 64976, 122928, 122989, 128512, 1114111],flags="u";patterns.map(pattern=>{let r=new RegExp(pattern,flags);return points.map(cp=>r.test(String.fromCodePoint(cp))?'1':'0').join('')}).join('|')==="00000000011111101111000001100|11111111100000010000111110011|00000000011111101111000001100|11111111100000010000111110011|11111111100000010000111110011|00000000011111101111000001100""#,
    );
    check(
        r#"let patterns=["^\\p{sc=Cyrillic}$", "^\\P{sc=Cyrillic}$", "^[\\p{sc=Cyrillic}]$", "^[\\P{sc=Cyrillic}]$", "^[^\\p{sc=Cyrillic}]$", "^[^\\P{sc=Cyrillic}]$"],points=[0, 10, 32, 65, 97, 181, 837, 888, 931, 1024, 1025, 1040, 1072, 1105, 1327, 1328, 7296, 7305, 7306, 7467, 8192, 55296, 56320, 57344, 64976, 122928, 122989, 128512, 1114111],flags="ui";patterns.map(pattern=>{let r=new RegExp(pattern,flags);return points.map(cp=>r.test(String.fromCodePoint(cp))?'1':'0').join('')}).join('|')==="00000000011111101111000001100|11111111100000010000111110011|00000000011111101111000001100|11111111100000010000111110011|11111111100000010000111110011|00000000011111101111000001100""#,
    );
    check(
        r#"let patterns=["^\\p{sc=Cyrillic}$", "^\\P{sc=Cyrillic}$", "^[\\p{sc=Cyrillic}]$", "^[\\P{sc=Cyrillic}]$", "^[^\\p{sc=Cyrillic}]$", "^[^\\P{sc=Cyrillic}]$"],points=[0, 10, 32, 65, 97, 181, 837, 888, 931, 1024, 1025, 1040, 1072, 1105, 1327, 1328, 7296, 7305, 7306, 7467, 8192, 55296, 56320, 57344, 64976, 122928, 122989, 128512, 1114111],flags="v";patterns.map(pattern=>{let r=new RegExp(pattern,flags);return points.map(cp=>r.test(String.fromCodePoint(cp))?'1':'0').join('')}).join('|')==="00000000011111101111000001100|11111111100000010000111110011|00000000011111101111000001100|11111111100000010000111110011|11111111100000010000111110011|00000000011111101111000001100""#,
    );
    check(
        r#"let patterns=["^\\p{sc=Cyrillic}$", "^\\P{sc=Cyrillic}$", "^[\\p{sc=Cyrillic}]$", "^[\\P{sc=Cyrillic}]$", "^[^\\p{sc=Cyrillic}]$", "^[^\\P{sc=Cyrillic}]$"],points=[0, 10, 32, 65, 97, 181, 837, 888, 931, 1024, 1025, 1040, 1072, 1105, 1327, 1328, 7296, 7305, 7306, 7467, 8192, 55296, 56320, 57344, 64976, 122928, 122989, 128512, 1114111],flags="vi";patterns.map(pattern=>{let r=new RegExp(pattern,flags);return points.map(cp=>r.test(String.fromCodePoint(cp))?'1':'0').join('')}).join('|')==="00000000011111101111000001100|11111111100000010000111110011|00000000011111101111000001100|11111111100000010000111110011|11111111100000010000111110011|00000000011111101111000001100""#,
    );
}

#[test]
fn unicode_cyrillic_script_results_snapshot() {
    let mut rows = String::new();
    for name in ["Script=Cyrillic", "Script=Cyrl", "sc=Cyrillic", "sc=Cyrl"] {
        for body in [
            format!(r"\p{{{name}}}"),
            format!(r"\P{{{name}}}"),
            format!(r"(?<x>[\p{{{name}}}K])"),
        ] {
            let pattern = JsString::from(body.as_str());
            for flags in ["dug", "diug", "divg", "divy"] {
                for input in ["ЁёᲀвΣ😀\u{378}\n", "X\r\nЁ\u{2028}𞀰", ""] {
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
fn unicode_cyrillic_script_captures_context_and_pair_starts() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let r=/^(?<x>([\p{{sc=Cyrl}}K]))$/dim{mode}g,m=r.exec('A\r\nЁ\u2028');m.index===3&&m[1]==='Ё'&&m[2]==='Ё'&&m.indices.groups.x===m.indices[1]&&m.indices[2][1]===4&&r.lastIndex===4"
        ));
        check(&format!(
            r"let r=/(?<__proto__>\p{{Script=Cyrillic}})/d{mode}y;r.lastIndex=1;let m=r.exec('𞀰A');m.index===0&&m[1]==='𞀰'&&m.indices[1][1]===2&&r.lastIndex===2&&Object.getPrototypeOf(m.groups)===null&&m.indices.groups.__proto__===m.indices[1]"
        ));
        check(&format!(
            r"let r=/\P{{sc=Cyrillic}}/d{mode}g;r.lastIndex=1;let m=r.exec('𞀰\uD800');m.index===2&&m[0]==='\uD800'&&r.lastIndex===3"
        ));
        check(&format!(
            r"let r=/\P{{Script=Cyrl}}/{mode}y;r.lastIndex=1;r.exec('𞀰\uD800')===null&&r.lastIndex===0"
        ));
    }
}

#[test]
fn unicode_cyrillic_script_consumers_preserve_original_text() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let a=[...'А😀AБ'.matchAll(/(?<x>\P{{sc=Cyrillic}})/d{mode}g)];a.length===2&&a[0][1]==='😀'&&a[0].index===1&&a[0].indices[1][1]===3&&a[1][1]==='A'&&a[1].index===3"
        ));
        check(&format!(
            r"'АБA'.replace(/(?<x>\p{{sc=Cyrl}})/{mode}g,'<$1:$<x>>')==='<А:А><Б:Б>A'"
        ));
        check(&format!(
            r"'AАБA'.split(/(\p{{Script=Cyrillic}})/{mode}).join('|')==='A|А||Б|A'&&'A😀А'.search(/\p{{sc=Cyrl}}/{mode})===3&&'АБA'.match(/\p{{Script=Cyrl}}/{mode}g).join('')==='АБ'"
        ));
        check(&format!(
            r"let t='',x='АБAА'.replace(/(\p{{sc=Cyrillic}})/{mode}g,(m,c,p)=>{{t+=p;return '_'}});x==='__A_'&&t==='013'"
        ));
    }
}

#[test]
fn unicode_cyrillic_script_clones_coercions_and_strict_writes() {
    check(
        r"let r=/^(?<x>\p{sc=Cyrl})$/dmu,c=new RegExp(r,'divy');c.source==='^(?<x>\\p{sc=Cyrl})$'&&c.unicodeSets&&!c.unicode&&!c.multiline&&c.exec('а').groups.x==='а'&&c.lastIndex===1",
    );
    check(
        r"let t='',r=/\p{Script=Cyrillic}/uy;r.lastIndex={valueOf(){t+='i';return 1}};let m=r.exec({toString(){t+='s';return '𞀰'}});t==='si'&&m[0]==='𞀰'&&m.index===0&&r.lastIndex===2",
    );
    check(
        r"let r=/\p{sc=Cyrillic}/vy;Object.defineProperty(r,'lastIndex',{writable:false});let caught=false;try{r.exec('А')}catch(e){caught=e instanceof TypeError}caught&&r.lastIndex===0",
    );
    check(
        r"let r=/\P{Script=Cyrl}/vy;r.lastIndex=1;Object.defineProperty(r,'lastIndex',{writable:false});let caught=false;try{r.exec('𞀰\uD800')}catch(e){caught=e instanceof TypeError}caught&&r.lastIndex===1",
    );
}

#[test]
fn unicode_cyrillic_script_repeated_sources_captures_and_collection() {
    assert_eq!(Limits::default().max_steps, None);
    assert_eq!(Limits::default().max_heap_entries, None);
    let mut realm = Realm::default();
    realm.eval("let p='^'+'('.repeat(3000)+'['+'\\\\p{sc=Cyrl}\\\\p{Cs}\\\\p{ASCII}'.repeat(10000)+']'+')'.repeat(3000)+'$',r=new RegExp(p,'dmvg'),c=new RegExp(r),text='A'.repeat(100000)+'\\nА'").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("let m=c.exec(text);m.length===3001&&m[1]==='А'&&m[3000]==='А'&&m.indices[3000][0]===100001&&m.indices[3000][1]===100002&&c.lastIndex===100002"), Ok(Value::Boolean(true)));
    realm.eval("p=null;r=null;c=null;text=null;m=null").unwrap();
    realm.collect(usize::MAX).unwrap();
}

#[test]
fn unicode_cyrillic_script_opted_work_and_remaining_sets() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval("let r=/\\p{Script=Cyrillic}/ug,marker=0,text='A'.repeat(10000);r.lastIndex=1")
        .unwrap();
    assert!(matches!(
        realm.eval("try{r.exec(text)}catch{marker=1}finally{marker=2}"),
        Err(Error::Limit { .. })
    ));
    assert_eq!(
        realm.eval("marker===0&&r.lastIndex===1"),
        Ok(Value::Boolean(true))
    );
    check(r"/\p{Script_Extensions=Cyrillic}/u.exec('A')===null");
    for source in [
        r"/[\p{sc=Cyrl}&&A]/v.exec('A')",
        r"/[\p{sc=Cyrl}--A]/v.exec('A')",
        r"/\p{sc=Cyrl}+b/u.exec('AA')",
        r"/\p{sc=Cyrl}A/v.exec('AA')",
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
fn unicode_cyrillic_script_repetitions_retain_last_capture_and_outside_words() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let r=/\B(?<x>\p{{sc=Cyrl}})+\B/d{mode}g,m=r.exec('µ𞀰𞀰𞀰µ');m.index===1&&m[0]==='𞀰𞀰𞀰'&&m.groups.x==='𞀰'&&m.indices.groups.x[0]===5&&m.indices.groups.x[1]===7&&r.lastIndex===7"
        ));
    }
}

#[test]
fn unicode_cyrillic_script_former_direct_eval_context_executes_normally() {
    check(
        r#"let marker=0,result;try{result=eval("marker=1; /\\p{Script=Cyrillic}/u.test('а');")}catch{marker=2}finally{marker=3}marker===3&&result===true"#,
    );
}
