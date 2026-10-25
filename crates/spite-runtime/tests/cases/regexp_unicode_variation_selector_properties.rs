//! Variation_Selector and its exact VS alias over complete Unicode characters.
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
fn unicode_variation_selector_property_results_snapshot() {
    let mut rows = String::new();
    for body in [
        r"\p{Variation_Selector}",
        r"\P{VS}",
        r"[^\p{VS}]",
        r"[\p{VS}K]",
        r"[\P{Variation_Selector}K]",
        r"^(?<x>[\p{VS}K])$",
        r"(?<x>\P{Variation_Selector})",
    ] {
        let pattern = JsString::from(body);
        for flags in ["dug", "diug", "divg", "divmy"] {
            for input in [
                "😀\u{180b}\u{180c}\u{180d}\u{180f}\u{fe00}\u{fe01}\u{fe02}\u{fe03}\u{fe04}\u{fe05}\u{fe06}\u{fe07}\u{fe08}\u{fe09}\u{fe0a}\u{fe0b}\u{fe0c}\u{fe0d}\u{fe0e}\u{fe0f}\u{e0100}\u{e0101}\u{e0102}\u{e0103}\u{e0104}\u{e0105}\u{e0106}\u{e0107}\u{e0108}\u{e0109}\u{e010a}\u{e010b}\u{e010c}\u{e010d}\u{e010e}\u{e010f}\u{e0110}\u{e0111}\u{e0112}\u{e0113}\u{e0114}\u{e0115}\u{e0116}\u{e0117}\u{e0118}\u{e0119}\u{e011a}\u{e011b}\u{e011c}\u{e011d}\u{e011e}\u{e011f}\u{e0120}\u{e0121}\u{e0122}\u{e0123}\u{e0124}\u{e0125}\u{e0126}\u{e0127}\u{e0128}\u{e0129}\u{e012a}\u{e012b}\u{e012c}\u{e012d}\u{e012e}\u{e012f}\u{e0130}\u{e0131}\u{e0132}\u{e0133}\u{e0134}\u{e0135}\u{e0136}\u{e0137}\u{e0138}\u{e0139}\u{e013a}\u{e013b}\u{e013c}\u{e013d}\u{e013e}\u{e013f}\u{e0140}\u{e0141}\u{e0142}\u{e0143}\u{e0144}\u{e0145}\u{e0146}\u{e0147}\u{e0148}\u{e0149}\u{e014a}\u{e014b}\u{e014c}\u{e014d}\u{e014e}\u{e014f}\u{e0150}\u{e0151}\u{e0152}\u{e0153}\u{e0154}\u{e0155}\u{e0156}\u{e0157}\u{e0158}\u{e0159}\u{e015a}\u{e015b}\u{e015c}\u{e015d}\u{e015e}\u{e015f}\u{e0160}\u{e0161}\u{e0162}\u{e0163}\u{e0164}\u{e0165}\u{e0166}\u{e0167}\u{e0168}\u{e0169}\u{e016a}\u{e016b}\u{e016c}\u{e016d}\u{e016e}\u{e016f}\u{e0170}\u{e0171}\u{e0172}\u{e0173}\u{e0174}\u{e0175}\u{e0176}\u{e0177}\u{e0178}\u{e0179}\u{e017a}\u{e017b}\u{e017c}\u{e017d}\u{e017e}\u{e017f}\u{e0180}\u{e0181}\u{e0182}\u{e0183}\u{e0184}\u{e0185}\u{e0186}\u{e0187}\u{e0188}\u{e0189}\u{e018a}\u{e018b}\u{e018c}\u{e018d}\u{e018e}\u{e018f}\u{e0190}\u{e0191}\u{e0192}\u{e0193}\u{e0194}\u{e0195}\u{e0196}\u{e0197}\u{e0198}\u{e0199}\u{e019a}\u{e019b}\u{e019c}\u{e019d}\u{e019e}\u{e019f}\u{e01a0}\u{e01a1}\u{e01a2}\u{e01a3}\u{e01a4}\u{e01a5}\u{e01a6}\u{e01a7}\u{e01a8}\u{e01a9}\u{e01aa}\u{e01ab}\u{e01ac}\u{e01ad}\u{e01ae}\u{e01af}\u{e01b0}\u{e01b1}\u{e01b2}\u{e01b3}\u{e01b4}\u{e01b5}\u{e01b6}\u{e01b7}\u{e01b8}\u{e01b9}\u{e01ba}\u{e01bb}\u{e01bc}\u{e01bd}\u{e01be}\u{e01bf}\u{e01c0}\u{e01c1}\u{e01c2}\u{e01c3}\u{e01c4}\u{e01c5}\u{e01c6}\u{e01c7}\u{e01c8}\u{e01c9}\u{e01ca}\u{e01cb}\u{e01cc}\u{e01cd}\u{e01ce}\u{e01cf}\u{e01d0}\u{e01d1}\u{e01d2}\u{e01d3}\u{e01d4}\u{e01d5}\u{e01d6}\u{e01d7}\u{e01d8}\u{e01d9}\u{e01da}\u{e01db}\u{e01dc}\u{e01dd}\u{e01de}\u{e01df}\u{e01e0}\u{e01e1}\u{e01e2}\u{e01e3}\u{e01e4}\u{e01e5}\u{e01e6}\u{e01e7}\u{e01e8}\u{e01e9}\u{e01ea}\u{e01eb}\u{e01ec}\u{e01ed}\u{e01ee}\u{e01ef}\u{180a}\u{180e}\u{1810}\u{fdff}\u{fe10}\u{e00ff}\u{e01f0}\u{feff}ſK\n",
                "X\r\n\u{fe00}\u{2028}\u{fe0f}\u{2029}",
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
fn unicode_variation_selector_properties_aliases_complements_and_unions() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let a=/\p{{Variation_Selector}}/{mode},b=/\p{{VS}}/i{mode},n=/\P{{VS}}/i{mode};a.test('\uFE00')&&a.test('\uFE01')&&b.test('\uFE00')&&!a.test('\u200D')&&!b.test('\u2010')&&!b.test('ſ')&&!b.test('K')&&!n.test('\uFE00')&&n.test('😀')&&n.test('\uD800')&&n.test('\uFEFF')"
        ));
        check(&format!(
            r"let p=/\p{{Variation_Selector}}/{mode},s=/\s/{mode};p.test('\uFE00')&&p.test('\uFE01')&&!s.test('\uFE00')&&!s.test('\uFE01')&&!p.test('\uFEFF')&&s.test('\uFEFF')"
        ));
        check(&format!(
            r"let a=/[\p{{VS}}K]/i{mode},b=/[^\P{{Variation_Selector}}K]/i{mode},c=/[\p{{VS}}\P{{Variation_Selector}}]/{mode};a.test('K')&&a.test('k')&&a.test('\uFE00')&&!a.test('ſ')&&b.test('\uFE01')&&!b.test('K')&&c.test('😀')&&c.test('A')&&c.test('\uD800')"
        ));
        check(&format!(
            r"let a=/^\p{{VS}}$/{mode},b=/^\P{{VS}}$/{mode};a.test('\uFE00')&&!a.test('\uFE01\n')&&!a.test('')&&!a.test('\n')&&b.test('😀')&&!b.test('AA')"
        ));
    }
}

#[test]
fn unicode_variation_selector_properties_captures_context_and_pair_starts() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let r=/(?<x>\p{{VS}})/d{mode}y;r.lastIndex=1;let m=r.exec('\uDB40\uDD00');m.index===0&&m[0]==='\uDB40\uDD00'&&m.groups.x===m[0]&&m.indices[1][0]===0&&m.indices[1][1]===2&&r.lastIndex===2"
        ));
    }
    for mode in ["u", "v"] {
        check(&format!(
            r"let r=/^(?<x>([\p{{VS}}K]))$/dim{mode}g,m=r.exec('x\r\nK\u2028');m.index===3&&m[1]==='K'&&m[2]==='K'&&m.indices.groups.x===m.indices[1]&&m.indices[2][1]===4&&r.lastIndex===4"
        ));
        check(&format!(
            r"let r=/(?<__proto__>\P{{VS}})/d{mode}y;r.lastIndex=1;let m=r.exec('😀\ufe00');m.index===0&&m[1]==='😀'&&m.indices[1][1]===2&&r.lastIndex===2&&Object.getPrototypeOf(m.groups)===null&&m.indices.groups.__proto__===m.indices[1]"
        ));
        check(&format!(
            r"let r=/\p{{VS}}/d{mode}g;r.lastIndex=1;let m=r.exec('😀\ufe00');m.index===2&&m[0]==='\ufe00'&&r.lastIndex===3"
        ));
        check(&format!(
            r"let r=/\p{{VS}}/{mode}y;r.lastIndex=1;r.exec('😀\ufe00')===null&&r.lastIndex===0"
        ));
    }
}

#[test]
fn unicode_variation_selector_properties_consumers_original_text() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let a=[...'\ufe01😀\ufe00G'.matchAll(/(?<x>\P{{VS}})/d{mode}g)];a.length===2&&a[0][1]==='😀'&&a[0].index===1&&a[0].indices[1][1]===3&&a[1][1]==='G'"
        ));
        check(&format!(
            r"'\ufe01\ufe00😀X'.replace(/(?<x>\p{{Variation_Selector}})/{mode}g,'<$1:$<x>>')==='<\ufe01:\ufe01><\ufe00:\ufe00>😀X'"
        ));
        check(&format!(
            r"'😀\ufe01\ufe00G'.split(/(\p{{VS}})/{mode}).join('|')==='😀|\ufe01||\ufe00|G'&&'😀\ufe00'.search(/\p{{VS}}/{mode})===2&&'\ufe01\ufe00😀'.match(/\p{{VS}}/{mode}g).join('')==='\ufe01\ufe00'"
        ));
        check(&format!(
            r"let t='';let x='\ufe00\ufe01😀\ufe00'.replace(/(\p{{VS}})/{mode}g,(m,c,p)=>{{t+=p;return '_'}});x==='__😀_'&&t==='014'"
        ));
    }
}

#[test]
fn unicode_variation_selector_properties_clones_coercions_and_strict_writes() {
    check(
        r"let r=/^(?<x>\p{Variation_Selector})$/dmu,c=new RegExp(r,'divy');c.source==='^(?<x>\\p{Variation_Selector})$'&&c.unicodeSets&&!c.unicode&&!c.multiline&&c.exec('\ufe00').groups.x==='\ufe00'&&c.lastIndex===1",
    );
    check(
        r"let t='',r=/\P{VS}/uy;r.lastIndex={valueOf(){t+='i';return 1}};let m=r.exec({toString(){t+='s';return '😀'}});t==='si'&&m[0]==='😀'&&m.index===0&&r.lastIndex===2",
    );
    check(
        r"let r=/\p{VS}/vy;Object.defineProperty(r,'lastIndex',{writable:false});let caught=false;try{r.exec('\ufe00')}catch(e){caught=e instanceof TypeError}caught&&r.lastIndex===0",
    );
    check(
        r"let r=/\p{VS}/vy;r.lastIndex=1;Object.defineProperty(r,'lastIndex',{writable:false});let caught=false;try{r.exec('😀\ufe00')}catch(e){caught=e instanceof TypeError}caught&&r.lastIndex===1",
    );
}

#[test]
fn unicode_variation_selector_properties_opted_work_and_remaining_sets() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval("let r=/\\p{VS}/ug,marker=0,text='😀'.repeat(10000);r.lastIndex=1")
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
    check(r"/[\p{VS}&&A]/v.exec('A')===null");
    check(r"/[\p{VS}--A]/v.exec('A')===null");
    check(
        r"let s=String.fromCodePoint(0xfe00),m=/[\p{VS}--A]/v.exec(s);m!==null&&m.length===1&&m[0]===s&&m.index===0&&m.input===s",
    );
    for source in [r"/\p{VS}+b/u.exec('AA')", r"/\p{VS}A/v.exec('AA')"] {
        assert!(
            crate::common::regexp_oracle::check(&mut Realm::default(), source),
            "{source}"
        );
    }
}

#[test]
fn unicode_variation_selector_repetitions_retain_last_capture_and_outside_words() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let r=/\B(?<x>\p{{VS}})+\B/d{mode}g,m=r.exec('😀\ufe00\ufe01\ufe0f!');m.index===2&&m[0]==='\ufe00\ufe01\ufe0f'&&m.groups.x==='\ufe0f'&&m.indices.groups.x[0]===4&&m.indices.groups.x[1]===5&&r.lastIndex===5"
        ));
    }
}
