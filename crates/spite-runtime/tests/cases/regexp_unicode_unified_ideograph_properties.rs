//! Unified_Ideograph and its exact UIdeo alias over complete Unicode characters.
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
fn unicode_unified_ideograph_property_results_snapshot() {
    let mut rows = String::new();
    for body in [
        r"\p{Unified_Ideograph}",
        r"\P{UIdeo}",
        r"[^\p{UIdeo}]",
        r"[\p{UIdeo}K]",
        r"[\P{Unified_Ideograph}K]",
        r"^(?<x>[\p{UIdeo}K])$",
        r"(?<x>\P{Unified_Ideograph})",
    ] {
        let pattern = JsString::from(body);
        for flags in ["dug", "diug", "divg", "divmy"] {
            for input in [
                "😀\u{33ff}\u{3400}\u{4dbf}\u{4dc0}\u{4dff}\u{4e00}\u{9fff}\u{a000}\u{fa0d}\u{fa0e}\u{fa0f}\u{fa10}\u{fa11}\u{fa12}\u{fa13}\u{fa14}\u{fa15}\u{fa1e}\u{fa1f}\u{fa20}\u{fa21}\u{fa22}\u{fa23}\u{fa24}\u{fa25}\u{fa26}\u{fa27}\u{fa29}\u{fa2a}\u{1ffff}\u{20000}\u{2a6df}\u{2a6e0}\u{2a6ff}\u{2a700}\u{2b81f}\u{2b820}\u{2cead}\u{2ceae}\u{2ceaf}\u{2ceb0}\u{2ebe0}\u{2ebe1}\u{2ebef}\u{2ebf0}\u{2ee5d}\u{2ee5e}\u{2ffff}\u{30000}\u{3134a}\u{3134b}\u{3134f}\u{31350}\u{33479}\u{3347a}\u{feff}ſK\n",
                "X\r\n\u{4e00}\u{2028}\u{4e09}\u{2029}",
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
fn unicode_unified_ideograph_properties_aliases_complements_and_unions() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let a=/\p{{Unified_Ideograph}}/{mode},b=/\p{{UIdeo}}/i{mode},n=/\P{{UIdeo}}/i{mode};a.test('\u4E00')&&a.test('\u4E8C')&&b.test('\u4E00')&&!a.test('\u200D')&&!b.test('\u005f')&&!b.test('ſ')&&!b.test('K')&&!n.test('\u4E00')&&n.test('😀')&&n.test('\uD800')&&n.test('\uFEFF')"
        ));
        check(&format!(
            r"let p=/\p{{Unified_Ideograph}}/{mode},s=/\s/{mode};p.test('\u4E00')&&p.test('\u4E8C')&&!s.test('\u4E00')&&!s.test('\u4E8C')&&!p.test('\uFEFF')&&s.test('\uFEFF')"
        ));
        check(&format!(
            r"let a=/[\p{{UIdeo}}K]/i{mode},b=/[^\P{{Unified_Ideograph}}K]/i{mode},c=/[\p{{UIdeo}}\P{{Unified_Ideograph}}]/{mode};a.test('K')&&a.test('k')&&a.test('\u4E00')&&!a.test('ſ')&&b.test('\u4E8C')&&!b.test('K')&&c.test('😀')&&c.test('A')&&c.test('\uD800')"
        ));
        check(&format!(
            r"let a=/^\p{{UIdeo}}$/{mode},b=/^\P{{UIdeo}}$/{mode};a.test('\u4E00')&&!a.test('\u4E8C\n')&&!a.test('')&&!a.test('\n')&&b.test('😀')&&!b.test('AA')"
        ));
    }
}

#[test]
fn unicode_unified_ideograph_properties_captures_context_and_pair_starts() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let r=/^(?<x>([\p{{UIdeo}}K]))$/dim{mode}g,m=r.exec('x\r\nK\u2028');m.index===3&&m[1]==='K'&&m[2]==='K'&&m.indices.groups.x===m.indices[1]&&m.indices[2][1]===4&&r.lastIndex===4"
        ));
        check(&format!(
            r"let r=/(?<__proto__>\P{{UIdeo}})/d{mode}y;r.lastIndex=1;let m=r.exec('😀\u4E00');m.index===0&&m[1]==='😀'&&m.indices[1][1]===2&&r.lastIndex===2&&Object.getPrototypeOf(m.groups)===null&&m.indices.groups.__proto__===m.indices[1]"
        ));
        check(&format!(
            r"let r=/\p{{UIdeo}}/d{mode}g;r.lastIndex=1;let m=r.exec('😀\u4E00');m.index===2&&m[0]==='\u4E00'&&r.lastIndex===3"
        ));
        check(&format!(
            r"let r=/\p{{UIdeo}}/{mode}y;r.lastIndex=1;r.exec('😀\u4E00')===null&&r.lastIndex===0"
        ));
    }
}

#[test]
fn unicode_unified_ideograph_properties_consumers_original_text() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let a=[...'\u4E8C😀\u4E00G'.matchAll(/(?<x>\P{{UIdeo}})/d{mode}g)];a.length===2&&a[0][1]==='😀'&&a[0].index===1&&a[0].indices[1][1]===3&&a[1][1]==='G'"
        ));
        check(&format!(
            r"'\u4E8C\u4E00😀X'.replace(/(?<x>\p{{Unified_Ideograph}})/{mode}g,'<$1:$<x>>')==='<\u4E8C:\u4E8C><\u4E00:\u4E00>😀X'"
        ));
        check(&format!(
            r"'😀\u4E8C\u4E00G'.split(/(\p{{UIdeo}})/{mode}).join('|')==='😀|\u4E8C||\u4E00|G'&&'😀\u4E00'.search(/\p{{UIdeo}}/{mode})===2&&'\u4E8C\u4E00😀'.match(/\p{{UIdeo}}/{mode}g).join('')==='\u4E8C\u4E00'"
        ));
        check(&format!(
            r"let t='';let x='\u4E00\u4E8C😀\u4E00'.replace(/(\p{{UIdeo}})/{mode}g,(m,c,p)=>{{t+=p;return '_'}});x==='__😀_'&&t==='014'"
        ));
    }
}

#[test]
fn unicode_unified_ideograph_properties_clones_coercions_and_strict_writes() {
    check(
        r"let r=/^(?<x>\p{Unified_Ideograph})$/dmu,c=new RegExp(r,'divy');c.source==='^(?<x>\\p{Unified_Ideograph})$'&&c.unicodeSets&&!c.unicode&&!c.multiline&&c.exec('\u4E00').groups.x==='\u4E00'&&c.lastIndex===1",
    );
    check(
        r"let t='',r=/\P{UIdeo}/uy;r.lastIndex={valueOf(){t+='i';return 1}};let m=r.exec({toString(){t+='s';return '😀'}});t==='si'&&m[0]==='😀'&&m.index===0&&r.lastIndex===2",
    );
    check(
        r"let r=/\p{UIdeo}/vy;Object.defineProperty(r,'lastIndex',{writable:false});let caught=false;try{r.exec('\u4E00')}catch(e){caught=e instanceof TypeError}caught&&r.lastIndex===0",
    );
    check(
        r"let r=/\p{UIdeo}/vy;r.lastIndex=1;Object.defineProperty(r,'lastIndex',{writable:false});let caught=false;try{r.exec('😀\u4E00')}catch(e){caught=e instanceof TypeError}caught&&r.lastIndex===1",
    );
}

#[test]
fn unicode_unified_ideograph_properties_repeated_sources_captures_and_collection() {
    assert_eq!(Limits::default().max_steps, None);
    assert_eq!(Limits::default().max_heap_entries, None);
    let mut realm = Realm::default();
    realm.eval("let p='^'+'('.repeat(3000)+'['+'\\\\p{UIdeo}'.repeat(10000)+']'+')'.repeat(3000)+'$',r=new RegExp(p,'dmvg'),c=new RegExp(r),text='😀'.repeat(100000)+'\\n\\u4E00'").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("let m=c.exec(text);m.length===3001&&m[1]==='\\u4E00'&&m[3000]==='\\u4E00'&&m.indices[3000][0]===200001&&m.indices[3000][1]===200002&&c.lastIndex===200002"),Ok(Value::Boolean(true)));
    realm.eval("p=null;r=null;c=null;text=null;m=null").unwrap();
    realm.collect(usize::MAX).unwrap();
}

#[test]
fn unicode_unified_ideograph_properties_opted_work_and_remaining_sets() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval("let r=/\\p{UIdeo}/ug,marker=0,text='😀'.repeat(10000);r.lastIndex=1")
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
    check(r"/[\p{UIdeo}&&A]/v.exec('A')===null");
    check(r"/[\p{UIdeo}--A]/v.exec('A')===null");
    check(
        r"let s=String.fromCodePoint(0x4e00),m=/[\p{UIdeo}--A]/v.exec(s);m!==null&&m.length===1&&m[0]===s&&m.index===0&&m.input===s",
    );
    for source in [r"/\p{UIdeo}+b/u.exec('AA')", r"/\p{UIdeo}A/v.exec('AA')"] {
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
fn unicode_unified_ideograph_repetitions_retain_last_capture_and_outside_words() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let r=/\B(?<x>\p{{UIdeo}})+\B/d{mode}g,m=r.exec('😀\u4E00\u4E8C\u4E09\uFEFF');m.index===2&&m[0]==='\u4E00\u4E8C\u4E09'&&m.groups.x==='\u4E09'&&m.indices.groups.x[0]===4&&m.indices.groups.x[1]===5&&r.lastIndex===5"
        ));
    }
}

#[test]
fn unicode_unified_ideograph_property_unicode18_addition() {
    let mut comparisons = 0;
    for point in [0x2b81eu32] {
        let text = JsString::from(char::from_u32(point).unwrap().to_string().as_str());
        for name in ["Unified_Ideograph", "UIdeo"] {
            for flags in ["u", "ui", "v", "vi"] {
                check(&format!(
                    r"let p=new RegExp('^\\p{{{name}}}$','{flags}'),n=new RegExp('^\\P{{{name}}}$','{flags}');p.test({text:?})&&!n.test({text:?})"
                ));
                comparisons += 1;
            }
        }
    }
    assert_eq!(comparisons, 8);
}
