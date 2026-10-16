//! Character-only v-set algebra through checked Runtime dispatch and consumers.
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
fn checked_v_set_algebra_and_folding() {
    check(
        r"!/[A&&a]/v.test('A')&&!/[A&&a]/v.test('a')&&/[A&&a]/iv.test('A')&&/[A&&a]/iv.test('a')",
    );
    check(
        r"/[A--a]/v.test('A')&&!/[A--a]/v.test('a')&&!/[A--a]/iv.test('A')&&!/[A--a]/iv.test('a')",
    );
    check(r"let r=/[K&&k]/iv;r.test('K')&&r.test('k')&&r.test('\u212A')&&!r.test('s')");
    check(r"!/[s--\u017F]/iv.test('s')&&!/[s--\u017F]/iv.test('\u017F')");
    check(
        r"let r=/[[Z-a]--[a]]/iv;r.test('[')&&r.test('_')&&r.test('`')&&r.test('Z')&&r.test('z')&&!r.test('A')&&!r.test('b')",
    );
    check(r"/[a&&a&&a]/v.test('a')&&!/[a--b--a]/v.test('a')");
    check(r"let r=/[[a-z]--[aeiou]--[b-d]]/v;r.test('f')&&r.test('z')&&!r.test('b')&&!r.test('e')");
    check(r"/[[^A]&&[a]]/v.test('a')&&!/[[^A]&&[a]]/iv.test('a')");
    check(r"!/[[A]--[^a]]/v.test('A')&&/[[A]--[^a]]/iv.test('A')&&/[[A]--[^a]]/iv.test('a')");
    check(r"!/[[]&&a]/v.test('a')&&/[a--[]]/v.test('a')&&!/[a--[]]/v.test('b')");
    check(r"let r=/[^[a-z]--[aeiou]]/v;r.test('a')&&r.test('😀')&&!r.test('b')");
    check(r"/[[^]--a]/v.test('A')&&!/[[^]--a]/iv.test('A')&&/[[^]--a]/iv.test('😀')");
}

#[test]
fn checked_v_set_property_complements_and_scripts() {
    // U18 explicitly extends these Inherited points to Greek. U+0345 folds to
    // Greek U+03B9, so folding before subtraction changes just that witness.
    check(
        r"let r=/[\p{scx=Greek}--\p{sc=Greek}]/v;r.test('\u0300')&&r.test('\u0345')&&!r.test('\u03B1')&&!r.test('A')",
    );
    check(
        r"let r=/[\p{scx=Greek}--\p{sc=Greek}]/iv;r.test('\u0300')&&!r.test('\u0345')&&!r.test('\u03B9')",
    );
    check(
        r"let a=/[\p{sc=Greek}&&\p{scx=Greek}]/v,b=/[\p{sc=Greek}&&\p{scx=Greek}]/iv;!a.test('\u0345')&&a.test('\u03B1')&&b.test('\u0345')&&b.test('\u03B9')",
    );
    check(r"/[\P{Ll}&&A]/v.test('A')&&!/[\P{Ll}&&A]/iv.test('A')&&!/[\P{Ll}&&A]/iv.test('a')");
    check(r"/[\P{sc=Greek}&&\u0345]/v.test('\u0345')&&!/[\P{sc=Greek}&&\u0345]/iv.test('\u0345')");
    check(
        r"!/[\P{scx=Greek}&&\u0345]/v.test('\u0345')&&!/[\P{scx=Greek}&&\u0345]/iv.test('\u0345')",
    );
    check(r"/[\P{scx=Greek}&&\u0302]/v.test('\u0302')&&/[\P{scx=Greek}&&\u0302]/iv.test('\u0302')");
    check(r"!/[\p{sc=Hrkt}&&a]/v.test('a')&&/[\P{sc=Hrkt}&&a]/v.test('a')");
    check(
        r"let r=/[\p{sc=Unknown}--[\uDC00-\uDFFF]]/v;r.test('\uD800')&&r.test('\uE000')&&!r.test('\uDC00')&&!r.test('😀')",
    );
}

#[test]
fn checked_v_set_results() {
    let mut rows = String::new();
    let mut row_count = 0;
    for (pattern, input) in [
        (r"(?<whole>([A&&a]))", JsString::from("😀aA")),
        (r"(?<whole>([[A-Z]--[K]]))", JsString::from("😀KkA")),
        (
            r"(?<whole>([[\u{1f600}-\u{1f601}]--[\u{1f601}]]))",
            JsString::from_code_units(vec![0xd800, 0xd83d, 0xde00, 0xdc00]),
        ),
        (
            r"(?<whole>([[\uD800-\uDFFF]--[\uDC00-\uDFFF]]))",
            JsString::from_code_units(vec![0xd83d, 0xde00, 0xd800, 0x61, 0xdc00]),
        ),
        (
            r"^(?<whole>([[a-z]--[aeiou]]+))$",
            JsString::from("😀\nbcd\nz"),
        ),
        (r"(?<whole>([A--a]*?))", JsString::from("A😀a")),
    ] {
        let pattern = JsString::from(pattern);
        for flags in ["dvmg", "divmg"] {
            for start in [0, 1] {
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
                row_count += 1;
            }
        }
    }
    assert_eq!(row_count, 24);
    insta::assert_snapshot!(rows);
}

#[test]
fn checked_v_set_global_sticky_and_utf16_starts() {
    // The existing approved Node-compatible rule moves an inside-pair Unicode
    // start to that pair's leading boundary, including this checked class plan.
    check(
        r"let r=/[[\u{1f600}-\u{1f601}]--[\u{1f601}]]/dvy;r.lastIndex=1;let m=r.exec('😀😁');m.index===0&&m[0]==='😀'&&m.indices[0][0]===0&&m.indices[0][1]===2&&r.lastIndex===2&&r.exec('😀😁')===null&&r.lastIndex===0",
    );
    check(
        r"let r=/[A&&a]/divy;r.lastIndex=1;let a=r.exec('😀aA');a===null&&r.lastIndex===0&&(r.lastIndex=2,r.exec('😀aA')[0]==='a')&&r.lastIndex===3&&r.exec('😀aA')[0]==='A'&&r.lastIndex===4&&r.exec('😀aA')===null&&r.lastIndex===0",
    );
    check(
        r"let r=/[[a-z]--[aeiou]]/dvg;r.lastIndex=1;let a=r.exec('😀b😀c'),b=r.exec('😀b😀c');a.index===2&&a.indices[0][1]===3&&b.index===5&&b.indices[0][1]===6&&r.lastIndex===6&&r.exec('😀b😀c')===null&&r.lastIndex===0",
    );
    check(
        r"let seen=0,r=/[A&&a]/iv;r.lastIndex={valueOf(){seen++;return 1}};let m=r.exec('😀a');m.index===2&&m[0]==='a'&&seen===1&&typeof r.lastIndex==='object'",
    );
    check(
        r"let r=/[A&&a]/divg,c=new RegExp(r,'dvy');c.source==='[A&&a]'&&c.unicodeSets&&!c.ignoreCase&&c.exec('a')===null&&c.lastIndex===0&&r.exec('a')[0]==='a'",
    );
    check(
        r"let r=/[a--b]/vy;Object.defineProperty(r,'lastIndex',{writable:false});let caught=false;try{r.exec('a')}catch(e){caught=e instanceof TypeError}caught&&r.lastIndex===0",
    );
}

#[test]
fn checked_v_set_consumers() {
    check(r"'😀bcd aef'.match(/[[a-z]--[aeiou]]+/vg).join('|')==='bcd|f'");
    check(
        r"let r=/(?<x>([[a-z]--[aeiou]]+))/dvg,a=[...'😀bcd😀f'.matchAll(r)];a.length===2&&a[0][0]==='bcd'&&a[0][1]==='bcd'&&a[0][2]==='bcd'&&a[0].index===2&&a[0].indices.groups.x===a[0].indices[1]&&a[0].indices[2][1]===5&&a[1].groups.x==='f'&&a[1].index===7&&a[1].indices[0][1]===8&&r.lastIndex===0",
    );
    check(r"let r=/[a--b]/vg;r.lastIndex=1;'😀ca'.search(r)===3&&r.lastIndex===1");
    check(r"'😀bc a f'.replace(/(?<x>[[a-z]--[aeiou]]+)/vg,'<$1:$<x>>')==='😀<bc:bc> a <f:f>'");
    check(
        r"let t='',s='😀b😀c'.replace(/(?<x>[[a-z]--[aeiou]])/vg,(m,c,p,text,g)=>{t+=p+':'+c+';';return '<'+g.x+'>'});s==='😀<b>😀<c>'&&t==='2:b;5:c;'",
    );
    check(r"'😀bc a f'.split(/([[a-z]--[aeiou]]+)/v).join('|')==='😀|bc| a |f|'");
    check(r"[...'😀a'.matchAll(/[a--a]*/dvg)].map(m=>m.index).join(',')==='0,2,3'");
}

#[test]
fn checked_v_set_anchored_repetition() {
    check(
        r"/^[[a-z]--[aeiou]]+$/v.test('bcdf')&&!/^[[a-z]--[aeiou]]+$/v.test('bca')&&!/^[[a-z]--[aeiou]]+$/v.test('')",
    );
    check(r"/[a--b]+/v.exec('aaa')[0]==='aaa'&&/[a--b]+?/v.exec('aaa')[0]==='a'");
    check(r"/[a--b]{2,3}/v.exec('aaaa')[0]==='aaa'&&/[a--b]{2,3}?/v.exec('aaaa')[0]==='aa'");
    check(
        r"/^[a--b]{2,3}?$/v.exec('aaa')[0]==='aaa'&&!/^[a--b]{2,3}$/v.test('a')&&!/^[a--b]{2,3}$/v.test('aaaa')",
    );
    check(
        r"/[a--b]?/v.exec('a')[0]==='a'&&/[a--b]??/v.exec('a')[0]===''&&/[a--b]{0}/v.exec('a')[0]===''",
    );
    check(
        r"/[a--a]+/v.exec('')===null&&/[a--a]+/v.exec('a')===null&&/^[a--a]*$/v.test('')&&!/^[a--a]*$/v.test('a')&&/[a--a]{2}/v.exec('')===null",
    );
    check(r"/^[^a--a]{2}$/v.test('😀a')&&!/^[^a--a]{3}$/v.test('😀a')");
    check(
        r"let r=/^[[a-z]--[aeiou]]+$/dmvg,a=r.exec('😀\r\nbcd\u2028f'),b=r.exec('😀\r\nbcd\u2028f');a[0]==='bcd'&&a.index===4&&a.indices[0][1]===7&&b[0]==='f'&&b.index===8&&r.lastIndex===9",
    );
    check(
        r"/[[a--a]]+/v.exec('')===null&&/[[a--a]]+/v.exec('a')===null&&/^[[a--a]]*$/v.test('')&&!/^[[a--a]]*$/v.test('a')",
    );
    check(r"/^[^[[a--a]]]{2}$/v.test('😀a')&&!/^[^[[a--a]]]{3}$/v.test('😀a')");
}

#[test]
fn checked_v_set_error_categories_and_effects() {
    for pattern in [
        "[a&&]",
        "[a&&b--c]",
        "[z-a]",
        r"[^\q{}]",
        "[[a&&b--c]]",
        "[[a]-z]",
        "[a-[z]]",
    ] {
        let code = JsString::from(format!("marker=1; /{pattern}/v;").as_str());
        check(&format!(
            "let marker=0,caught=false;try{{eval({code:?})}}catch(e){{caught=e instanceof SyntaxError}}caught&&marker===0"
        ));
    }
    for (pattern, expected) in [
        ("[a--b]", true),
        ("[a&&b]", false),
        ("[[a]]", true),
        ("[[[a]]&&a]", true),
        ("[[a&&a]&&a]", true),
        (r"\b[a--b]\b", true),
        ("[a--b]c", false),
    ] {
        let code = JsString::from(format!("marker=1; /{pattern}/v.test('a');").as_str());
        check(&format!(
            "let marker=0,result,caught=false;try{{result=eval({code:?})}}catch{{caught=true}}finally{{marker=3}}!caught&&marker===3&&result==={expected}"
        ));
    }
    check(r"!/\b[a--b]\b/v.test('b')");
    for pattern in [
        r"[a&&b&&\q{ab}]",
        r"[a--a--\q{ab}]",
        r"[\q{}&&a]",
        r"[\p{RGI_Emoji}&&a]",
        r"[[a--a]&&[\q{ab}]]",
        r"[[][^][\p{RGI_Emoji}]]",
        "([a--b])+",
    ] {
        let code = JsString::from(format!("marker=1; /{pattern}/v.test('a');").as_str());
        let mut realm = Realm::default();
        realm.eval("var marker=0").unwrap();
        assert!(
            matches!(
                realm.eval(&format!(
                    "try{{eval({code:?})}}catch{{marker=2}}finally{{marker=3}}"
                )),
                Err(Error::Unsupported { .. })
            ),
            "{pattern}"
        );
        assert_eq!(realm.eval("marker"), Ok(Value::Number(1.0)), "{pattern}");
    }
}

#[test]
fn checked_v_set_opted_work_preserves_state() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm.eval(r"var r=/[a--b]/vg,marker=0").unwrap();
    // Prepare the input outside JavaScript's opted-in string-construction work.
    // This quota test must reach the search before exhausting the allowance.
    let setup = format!("var text='{}';r.lastIndex=1", "b".repeat(20_000));
    realm.eval(&setup).unwrap();
    assert!(matches!(
        realm.eval("try{r.exec(text)}catch{marker=1}finally{marker=2}"),
        Err(Error::Limit { .. })
    ));
    assert_eq!(
        realm.eval("marker===0&&r.lastIndex===1"),
        Ok(Value::Boolean(true))
    );
}

#[test]
fn checked_v_set_nested_algebra_and_state() {
    check(r"let r=/[[a[b]]c]/v;r.test('a')&&r.test('b')&&r.test('c')&&!r.test('A')&&!r.test('d')");
    check(r"/[a--[b--a]]/v.test('a')&&!/[a--b--a]/v.test('a')");
    check(r"/[A--[a--A]]/iv.test('A')&&/[A--[a--A]]/iv.test('a')&&!/[A--a--A]/iv.test('A')");
    check(r"/[[^[A&&a]]&&[a]]/v.test('a')&&!/[[^[A&&a]]&&[a]]/iv.test('a')");
    check(r"!/[[a--a]]/v.test('a')&&/[^[[a--a]]]/v.test('a')&&/[^[[a--a]]]/v.test('😀')");
    check(
        r"let r=/[[[\u{1f600}]]--[[\uD800-\uDFFF]]]/dvy;r.lastIndex=1;let m=r.exec('😀a');m[0]==='😀'&&m.index===0&&m.indices[0][0]===0&&m.indices[0][1]===2&&r.lastIndex===2&&r.exec('😀a')===null&&r.lastIndex===0",
    );
    check(
        r"let r=/^(?<whole>([[\(\)]]))$/dv,a=r.exec('('),b=r.exec(')');a.length===3&&a[0]==='('&&a[1]==='('&&a[2]==='('&&a.groups.whole==='('&&a.indices.groups.whole===a.indices[1]&&a.indices[2][1]===1&&b[0]===')'&&b[2]===')'&&b.index===0&&b.input===')'",
    );
}

#[test]
fn checked_v_set_nested_results() {
    let mut rows = String::new();
    let mut row_count = 0;
    for (pattern, input) in [
        (r"(?<whole>([[a]b]))", JsString::from("😀A")),
        (r"(?<whole>([[A--a]&&[A]]))", JsString::from("😀Aa")),
        (r"^(?<whole>([[a-z]--[[aeiou]b]]+))$", JsString::from("cdf")),
        (
            r"(?<whole>([[[\u{1f600}]]--[[\uD800-\uDFFF]]]))",
            JsString::from_code_units(vec![0xd800, 0xd83d, 0xde00, 0xdc00]),
        ),
    ] {
        let pattern = JsString::from(pattern);
        let start = 0;
        for flags in ["dvg", "divg"] {
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
            row_count += 1;
        }
    }
    assert_eq!(row_count, 8);
    insta::assert_snapshot!(rows);
}

#[test]
fn checked_v_set_word_state_context_and_consumers() {
    check(
        r"let w=/\b[[Kſα]]\b/iv,n=/\B[[Kſα]]\B/iv;w.test('K')&&w.test('ſ')&&!w.test('α')&&n.test('α')&&!n.test('K')&&!n.test('ſ')&&/\B[[Kſα]]\B/v.test('ſ')",
    );
    check(
        r"/\b[a--b]\B/v.exec('aa').index===0&&/\B[a--b]\b/v.exec('aa').index===1&&/\b[a--b]/v.test('aa')&&/[a--b]\b/v.exec('aa').index===1&&!/\b[[a--a]]\b/v.test('a')&&!/\B[[a--a]]\B/iv.test('!')",
    );
    check(
        r"let a=/\b[[\p{Any}]]\b/vy,b=/\B[[\p{Any}]]\b/vy;a.lastIndex=1;b.lastIndex=1;let m=b.exec('😀B');a.exec('😀B')===null&&a.lastIndex===0&&m.index===0&&m[0]==='😀'&&b.lastIndex===2&&b.exec('😀B')===null&&b.lastIndex===0",
    );
    check(
        r"let r=/\b[[\P{space}]]\b/dvy;r.lastIndex=1;let m=r.exec('A\uD800B');m.index===1&&m[0]==='\uD800'&&m.indices[0][0]===1&&m.indices[0][1]===2&&r.lastIndex===2",
    );
    check(
        r"/^\b[[K]]\b$/iv.test('K')&&!/^\b[[K]]\b$/iv.test('K\n')&&/^\b[[K]]\b$/imv.exec('x\r\nK\u2028').index===3",
    );
    check(
        r"let r=/\b[[Kſ]]\b/ivg;r.lastIndex=1;'A K ſ K_'.search(r)===2&&r.lastIndex===1&&'K ſ K_'.replace(/(?<whole>\b[[Kſ]]\b)/ivg,'<$<whole>>')==='<K> <ſ> K_'",
    );
    check(
        r"let r=/(?<whole>(\b[[Kſ]]\b))/divg,a=[...'K ſ K_'.matchAll(r)];a.length===2&&a[0][1]==='K'&&a[0][2]==='K'&&a[0].index===0&&a[0].indices.groups.whole===a[0].indices[1]&&a[1].groups.whole==='ſ'&&a[1].index===2&&a[1].indices[2][1]===3&&r.lastIndex===0",
    );
    check(
        r"let t='',r=/\b[[\p{Any}]]\b/vy;r.lastIndex={valueOf(){t+='i';return 2}};let m=r.exec({toString(){t+='s';return 'A😀B'}});t==='si'&&m.index===1&&m[0]==='😀'&&r.lastIndex===3",
    );
    check(
        r"let r=/\b[a--b]\b/vy;r.lastIndex=1;Object.defineProperty(r,'lastIndex',{writable:false});let caught=false;try{r.exec('aa')}catch(e){caught=e instanceof TypeError}caught&&r.lastIndex===1",
    );
    check(
        r"let marker=0,result,caught=false;try{marker=1;result=/\b[a--b]+\b/v.test('a')}catch{caught=true}finally{marker=3}!caught&&marker===3&&result&&!/\b[a--b]+\b/v.test('b')&&/\b[a--b]+\b/v.exec('aaa!')[0]==='aaa'",
    );
    for source in [r"/\b^[[K]]/v.test('K')", r"/[[K]]$\b/v.test('K')"] {
        assert!(
            matches!(
                Realm::default().eval(source),
                Err(Error::Unsupported { .. })
            ),
            "{source}"
        );
    }
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm.eval(r"var r=/\b[[a--b]]\b/vg,marker=0").unwrap();
    // Construct this nonmember input outside the opted-in JavaScript work.
    let setup = format!("var text='{}';r.lastIndex=1", "b".repeat(20_000));
    realm.eval(&setup).unwrap();
    assert!(matches!(
        realm.eval("try{r.exec(text)}catch{marker=1}finally{marker=2}"),
        Err(Error::Limit { .. })
    ));
    assert_eq!(
        realm.eval("marker===0&&r.lastIndex===1"),
        Ok(Value::Boolean(true))
    );
}

#[test]
fn checked_v_set_word_results() {
    let mut rows = String::new();
    let mut row_count = 0;
    for (pattern, input, start, flag_modes) in [
        (
            r"(?<whole>(\b[[Kſα]]\b))",
            JsString::from("K ſ α"),
            0,
            ["dvg", "divg"],
        ),
        (
            r"\B(?<whole>([[Kſα]]))\B",
            JsString::from("K ſ α"),
            0,
            ["dvg", "divg"],
        ),
        (
            r"(?<whole>(\b[[\p{Any}]]\b))",
            JsString::from("A😀B"),
            2,
            ["dvy", "divy"],
        ),
        (
            r"^(?<whole>(\b[[K]]\b))$",
            JsString::from("x\r\nK\u{2028}K\u{2029}"),
            0,
            ["dvmg", "divmg"],
        ),
        (
            r"(?<whole>(\b[[\b]]\B))",
            JsString::from("A\u{8}!"),
            0,
            ["dvg", "divg"],
        ),
    ] {
        let pattern = JsString::from(pattern);
        for flags in flag_modes {
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
            row_count += 1;
        }
    }
    assert_eq!(row_count, 10);
    insta::assert_snapshot!(rows);
}

#[test]
fn checked_v_set_repeated_word_state_context_and_consumers() {
    check(
        r"/\b[[a]]{0002,0003}\b/v.exec('aaa!')[0]==='aaa'&&/\b[[a]]{2,}\b/v.exec('aaa!')[0]==='aaa'&&/\b[[a]]??\b/v.exec('aaa!')[0]===''&&!/\b[[a]]{999999999999999999999999}\b/v.test('a')&&/\b[[a]]{0,999999999999999999999999}\b/v.exec('aaa!')[0]==='aaa'",
    );
    check(
        r"let r=/\b[a--a]*\b/dvg,m=r.exec('a');m[0]===''&&m.index===0&&m.indices[0][1]===0&&r.lastIndex===0&&r.exec('a').index===0&&r.lastIndex===0&&!/\b[a--a]+\b/v.test('a')&&/\B[a--a]{0}\B/v.exec('')[0]===''&&/\b[a--a]*\B/v.exec('a')===null",
    );
    check(
        r"let r=/\b[a--a]*\b/dvy;r.lastIndex=2;let m=r.exec('A😀B');m[0]===''&&m.index===1&&m.indices[0][0]===1&&m.indices[0][1]===1&&r.lastIndex===1&&(r.lastIndex=1,r.exec('😀!')===null)&&r.lastIndex===0",
    );
    check(
        r"let r=/\b[a--a]*\b/dvg,a=[...'A😀B'.matchAll(r)];a.map(m=>m.index).join(',')==='0,1,3,4'&&r.lastIndex===0&&'A😀B'.replace(/\b[a--a]*\b/vg,'|')==='|A|😀|B|'",
    );
    check(
        r"'aa_!'.replace(/(?<whole>(\b[[a_]]+\B))/v,'<$1:$<whole>>')==='<aa:aa>_!'&&'aaa!'.match(/\b[[a]]+?\b/v)[0]==='aaa'",
    );
    check(r"let r=/\b[[a]]+?\b/vg;r.lastIndex=1;'x aaa!'.search(r)===2&&r.lastIndex===1");
    check(
        r"let r=/\b[[\p{Any}]]{2}\b/dvy,m=r.exec('A\uD800B');m.index===0&&m[0]==='A\uD800'&&m.indices[0][1]===2&&r.lastIndex===2&&r.exec('A\uD800B')===null&&r.lastIndex===0",
    );
    check(
        r"let marker=0,caught=false;try{eval('marker=1; /\\b[[a]]{3,2}\\b/v;')}catch(e){caught=e instanceof SyntaxError}caught&&marker===0",
    );
    for source in [
        r"/\b^[[a]]+/v.test('a')",
        r"/\b[[a]]+$\b/v.test('a')",
        r"/(\b[[a]]\b)+/v.test('a')",
        r"/\b[[\q{a}]]{0}\b/v.test('a')",
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
fn checked_v_set_repeated_word_results() {
    let mut rows = String::new();
    let mut row_count = 0;
    for (pattern, input, flags, start) in [
        (r"(?<whole>(\b[[a_]]+\B))", JsString::from("aa_!"), "dvg", 0),
        (
            r"(?<whole>(\b[[a_]]+?\B))",
            JsString::from("aa_!"),
            "dvg",
            0,
        ),
        (r"(?<whole>(\b[[a]]+?\b))", JsString::from("aaa!"), "dvg", 0),
        (r"(?<whole>(\b[[K]]+\b))", JsString::from("KK!"), "dvg", 0),
        (r"(?<whole>(\b[[K]]+\b))", JsString::from("KK!"), "divg", 0),
        (r"(?<whole>(\B[a--a]*\B))", JsString::from("😀!"), "dvy", 1),
        (r"(?<whole>(\b[a--a]*\b))", JsString::from("A😀B"), "dvy", 2),
        (
            r"^(?<whole>(\b[[a]]{2,3}?\b))$",
            JsString::from("x\r\naaa\u{2028}"),
            "dvmg",
            0,
        ),
        (
            r"(?<whole>(\b[[\p{Any}]]{2}\b))",
            JsString::from("A😀B"),
            "dvg",
            0,
        ),
        (
            r"(?<whole>(\b[[a]]{0,1}?\b))",
            JsString::from("a!"),
            "dvg",
            0,
        ),
    ] {
        let pattern = JsString::from(pattern);
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
        row_count += 1;
    }
    assert_eq!(row_count, 10);
    insta::assert_snapshot!(rows);
}

#[test]
fn checked_v_set_literal_state_context_and_consumers() {
    check(
        r"/[a--b]c/v.test('ac')&&!/[a--b]c/v.test('bc')&&!/[a--b]c/v.test('a')&&!/[a--b]c/v.test('abc')&&/[a--b]c/v.exec('aac').index===1",
    );
    check(
        r"let r=/[a--b]c/dvy;r.exec('aac')===null&&r.lastIndex===0&&(r.lastIndex=1,r.exec('aac')[0]==='ac')&&r.lastIndex===3&&r.exec('aac')===null&&r.lastIndex===0",
    );
    check(
        r"let r=/(?<whole>([[K]]s))/div,m=r.exec('Kſ!');m.length===3&&m[0]==='Kſ'&&m[1]==='Kſ'&&m[2]==='Kſ'&&m.groups.whole==='Kſ'&&m.indices.groups.whole===m.indices[1]&&m.indices[2][1]===2&&/[[K]]s/v.test('Ks')&&!/[[K]]s/v.test('Ks')&&!/[[K]]s/v.test('Kſ')",
    );
    check(
        r"let r=/[[\p{Any}]]\u{1f601}/dvy;r.lastIndex=1;let a=r.exec('😀😁😀😁'),b=r.exec('😀😁😀😁');a.index===0&&a[0]==='😀😁'&&a.indices[0][1]===4&&b.index===4&&r.lastIndex===8&&r.exec('😀😁😀😁')===null&&r.lastIndex===0",
    );
    check(
        r"let r=/[[\uD800]]a/dv,m=r.exec('\uD800a!');m[0]==='\uD800a'&&m.index===0&&m.indices[0][1]===2&&/[[\uD83D]]\uDE00/v.exec('😀')===null",
    );
    check(
        r"/[[\]]]\$/v.exec('x]$!')[0]===']$'&&/^[[a]]\$$/v.test('a$')&&!/^[[a]]\$$/v.test('a$\n')&&/[[a]]\]/v.test('a]')",
    );
    check(
        r"let r=/^(?<whole>([[a]]c))$/dvmg,m=r.exec('x\r\nac\u2028bc');m[0]==='ac'&&m[1]==='ac'&&m[2]==='ac'&&m.index===3&&m.indices[2][0]===3&&m.indices[2][1]===5&&r.lastIndex===5&&r.exec('x\r\nac\u2028bc')===null&&r.lastIndex===0",
    );
    check(
        r"let r=/(?<whole>([[a]]c))/dvg,a=[...'😀ac aac'.matchAll(r)];a.length===2&&a[0][0]==='ac'&&a[0][1]==='ac'&&a[0][2]==='ac'&&a[0].index===2&&a[0].indices.groups.whole===a[0].indices[1]&&a[1].groups.whole==='ac'&&a[1].index===6&&a[1].indices[2][1]===8&&r.lastIndex===0",
    );
    check(
        r"'😀ac aac'.replace(/(?<whole>([[a]]c))/vg,'<$1:$<whole>>')==='😀<ac:ac> a<ac:ac>'&&'😀acx'.split(/([[a]]c)/v).join('|')==='😀|ac|x'&&'aac'.match(/[a--b]c/v)[0]==='ac'",
    );
    check(
        r"let seen='',out='😀ac aac'.replace(/(?<whole>([[a]]c))/vg,(m,c1,c2,p,text,g)=>{seen+=p+':'+g.whole+';';return '!'});out==='😀! a!'&&seen==='2:ac;6:ac;'",
    );
    check(r"let r=/[[a]]c/vg;r.lastIndex=1;'😀aac'.search(r)===3&&r.lastIndex===1");
    check(
        r"let trace='',r=/[[\p{Any}]]c/vy;r.lastIndex={valueOf(){trace+='i';return 1}};let m=r.exec({toString(){trace+='s';return '😀c'}});trace==='si'&&m.index===0&&m[0]==='😀c'&&r.lastIndex===3",
    );
    check(
        r"let seen=0,r=/[[a]]c/v;r.lastIndex={valueOf(){seen++;return 1}};let m=r.exec('aac');m.index===1&&m[0]==='ac'&&seen===1&&typeof r.lastIndex==='object'",
    );
    check(
        r"let r=/[[K]]s/iv,c=new RegExp(r,'dvy');c.source==='[[K]]s'&&c.unicodeSets&&!c.ignoreCase&&c.exec('Kſ')===null&&c.lastIndex===0&&r.exec('Kſ')[0]==='Kſ'",
    );
    check(
        r"let r=/[[a]]c/vy;r.lastIndex=1;Object.defineProperty(r,'lastIndex',{writable:false});let caught=false;try{r.exec('aac')}catch(e){caught=e instanceof TypeError}caught&&r.lastIndex===1",
    );
    check(
        r"let marker=0,caught=false;try{eval('marker=1; /[a&&]c/v;')}catch(e){caught=e instanceof SyntaxError}caught&&marker===0",
    );
    for source in [
        r"/[a--b]+c/v.test('ac')",
        r"/[a--b]cd/v.test('acd')",
        r"/[a--b][c]/v.test('ac')",
        r"/([a--b])c/v.test('ac')",
        r"/[a--b](c)/v.test('ac')",
        r"/([a--b]c)+/v.test('ac')",
        r"/[a--b]^c/v.test('ac')",
        r"/[a--b]$c/v.test('ac')",
        r"/[a--a--\q{a}]c/v.test('ac')",
        r"/[\p{RGI_Emoji}&&a]c/v.test('ac')",
    ] {
        assert!(
            matches!(
                Realm::default().eval(source),
                Err(Error::Unsupported { .. })
            ),
            "{source}"
        );
    }
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm.eval(r"var r=/[a--b]c/vg,marker=0").unwrap();
    // Every prefix succeeds while its literal continuation fails. Build this
    // input outside JavaScript's opted-in string-construction work.
    let setup = format!("var text='{}';r.lastIndex=1", "a".repeat(20_000));
    realm.eval(&setup).unwrap();
    assert!(matches!(
        realm.eval("try{r.exec(text)}catch{marker=1}finally{marker=2}"),
        Err(Error::Limit { .. })
    ));
    assert_eq!(
        realm.eval("marker===0&&r.lastIndex===1"),
        Ok(Value::Boolean(true))
    );
}

#[test]
fn checked_v_set_literal_results() {
    let mut rows = String::new();
    let mut row_count = 0;
    for (pattern, input, flags, start) in [
        (r"(?<whole>([a--b]c))", JsString::from("aac"), "dvg", 0),
        (r"(?<whole>([a--b]c))", JsString::from("aac"), "dvy", 0),
        (r"(?<whole>([[K]]s))", JsString::from("Kſ!"), "divg", 0),
        (r"(?<whole>([[K]]s))", JsString::from("Kſ!"), "dvg", 0),
        (
            r"(?<whole>([[\p{Any}]]\u{1f601}))",
            JsString::from("😀😁!"),
            "dvy",
            1,
        ),
        (
            r"(?<whole>([[\uD800]]a))",
            JsString::from_code_units(vec![0xd800, 0x61, 0x21]),
            "dvg",
            0,
        ),
        (
            r"(?<whole>([[\uD83D]]\uDE00))",
            JsString::from("😀"),
            "dvg",
            0,
        ),
        (r"(?<whole>([[\]]]\$))", JsString::from("x]$!"), "dvg", 0),
        (
            r"^(?<whole>([[a]]c))$",
            JsString::from("x\r\nac\u{2028}"),
            "dvmg",
            0,
        ),
        (r"^(?<whole>([[a]]\$))$", JsString::from("a$"), "dvg", 0),
    ] {
        let pattern = JsString::from(pattern);
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
        row_count += 1;
    }
    assert_eq!(row_count, 10);
    insta::assert_snapshot!(rows);
}

#[test]
fn checked_literal_v_set_state_context_and_consumers() {
    check(
        r#"let marker=0,result,caught=false;try{result=eval("marker=1; /c[a--b]/v.test('ca')")}catch{caught=true}finally{marker=3}!caught&&marker===3&&result&&!/c[a--b]/v.test('cb')&&!/c[a--b]/v.test('c')&&!/c[a--b]/v.test('cba')&&/c[a--b]/v.exec('ccca').index===2"#,
    );
    check(
        r"let r=/c[a--b]/dvy;r.exec('ccca')===null&&r.lastIndex===0&&(r.lastIndex=2,r.exec('ccca')[0]==='ca')&&r.lastIndex===4&&r.exec('ccca')===null&&r.lastIndex===0",
    );
    check(
        r"let r=/(?<whole>(k[[s]]))/div,m=r.exec('Kſ!');m.length===3&&m[0]==='Kſ'&&m[1]==='Kſ'&&m[2]==='Kſ'&&m.groups.whole==='Kſ'&&m.indices.groups.whole===m.indices[1]&&m.indices[2][1]===2&&/k[[s]]/v.test('ks')&&!/k[[s]]/v.test('Ks')&&!/k[[s]]/v.test('kſ')",
    );
    check(
        r"let r=/\u{1f600}[[\p{Any}]]/dvy;r.lastIndex=1;let a=r.exec('😀😁😀😁'),b=r.exec('😀😁😀😁');a.index===0&&a[0]==='😀😁'&&a.indices[0][1]===4&&b.index===4&&r.lastIndex===8&&r.exec('😀😁😀😁')===null&&r.lastIndex===0",
    );
    check(
        r"let r=/\uD800[[a]]/dv,m=r.exec('\uD800a!');m[0]==='\uD800a'&&m.index===0&&m.indices[0][1]===2&&/\uD83D[[\uDE00]]/v.exec('😀')===null",
    );
    check(
        r"/\[[[a]]/v.exec('x[a!')[0]==='[a'&&/\/[[a]]/v.test('/a')&&/\/[[a]]/v.source==='\\/[[a]]'&&/^\$[[a]]$/v.test('$a')&&!/^\$[[a]]$/v.test('$a\n')",
    );
    check(
        r"let r=/^(?<whole>(c[[a]]))$/dvmg,m=r.exec('x\r\nca\u2028cb');m[0]==='ca'&&m[1]==='ca'&&m[2]==='ca'&&m.index===3&&m.indices[2][0]===3&&m.indices[2][1]===5&&r.lastIndex===5&&r.exec('x\r\nca\u2028cb')===null&&r.lastIndex===0",
    );
    check(
        r"!/c[a--a]/v.test('ca')&&!/c[a--a]/v.test('')&&/c[^[[a--a]]]/v.test('ca')&&/c[^[[a--a]]]/v.test('c😀')&&!/c[^[[a--a]]]/v.test('c')",
    );
    check(
        r"let r=/(?<whole>(c[[a]]))/dvg,a=[...'😀ca cca'.matchAll(r)];a.length===2&&a[0][0]==='ca'&&a[0][1]==='ca'&&a[0][2]==='ca'&&a[0].index===2&&a[0].indices.groups.whole===a[0].indices[1]&&a[1].groups.whole==='ca'&&a[1].index===6&&a[1].indices[2][1]===8&&r.lastIndex===0",
    );
    check(
        r"'😀ca cca'.replace(/(?<whole>(c[[a]]))/vg,'<$1:$<whole>>')==='😀<ca:ca> c<ca:ca>'&&'😀cax'.split(/(c[[a]])/v).join('|')==='😀|ca|x'&&'ccca'.match(/c[a--b]/v)[0]==='ca'",
    );
    check(
        r"let seen='',out='😀ca cca'.replace(/(?<whole>(c[[a]]))/vg,(m,c1,c2,p,text,g)=>{seen+=p+':'+g.whole+';';return '!'});out==='😀! c!'&&seen==='2:ca;6:ca;'",
    );
    check(r"let r=/c[[a]]/vg;r.lastIndex=1;'😀ccca'.search(r)===4&&r.lastIndex===1");
    check(
        r"let trace='',r=/\u{1f600}[[a]]/vy;r.lastIndex={valueOf(){trace+='i';return 1}};let m=r.exec({toString(){trace+='s';return '😀a'}});trace==='si'&&m.index===0&&m[0]==='😀a'&&r.lastIndex===3",
    );
    check(
        r"let seen=0,r=/c[[a]]/v;r.lastIndex={valueOf(){seen++;return 1}};let m=r.exec('ccca');m.index===2&&m[0]==='ca'&&seen===1&&typeof r.lastIndex==='object'",
    );
    check(
        r"let r=/k[[s]]/iv,c=new RegExp(r,'dvy');c.source==='k[[s]]'&&c.unicodeSets&&!c.ignoreCase&&c.exec('Kſ')===null&&c.lastIndex===0&&r.exec('Kſ')[0]==='Kſ'",
    );
    check(
        r"let r=/c[[a]]/vy;r.lastIndex=2;Object.defineProperty(r,'lastIndex',{writable:false});let caught=false;try{r.exec('ccca')}catch(e){caught=e instanceof TypeError}caught&&r.lastIndex===2",
    );
    check(
        r"let marker=0,caught=false;try{eval('marker=1; /c[a&&]/v;')}catch(e){caught=e instanceof SyntaxError}caught&&marker===0",
    );
    for source in [
        r"/cc[a--b]/v.test('cca')",
        r"/c[a--b]+/v.test('ca')",
        r"/c[a--b][c]/v.test('cac')",
        r"/(c)[a--b]/v.test('ca')",
        r"/c([a--b])/v.test('ca')",
        r"/(c[a--b])+/v.test('ca')",
        r"/c^[a--b]/v.test('ca')",
        r"/c$[a--b]/v.test('ca')",
        r"/c[a--a--\q{a}]/v.test('ca')",
        r"/c[\p{RGI_Emoji}&&a]/v.test('ca')",
    ] {
        assert!(
            matches!(
                Realm::default().eval(source),
                Err(Error::Unsupported { .. })
            ),
            "{source}"
        );
    }
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm.eval(r"var r=/c[a--b]/vg,marker=0").unwrap();
    // Every literal prefix succeeds while its checked class fails. Build the
    // input outside JavaScript's opted-in string-construction work.
    let setup = format!("var text='{}';r.lastIndex=1", "c".repeat(20_000));
    realm.eval(&setup).unwrap();
    assert!(matches!(
        realm.eval("try{r.exec(text)}catch{marker=1}finally{marker=2}"),
        Err(Error::Limit { .. })
    ));
    assert_eq!(
        realm.eval("marker===0&&r.lastIndex===1"),
        Ok(Value::Boolean(true))
    );
}

#[test]
fn checked_literal_v_set_results() {
    let mut rows = String::new();
    let mut row_count = 0;
    for (pattern, input, flags, start) in [
        (r"(?<whole>(c[a--b]))", JsString::from("ccca"), "dvg", 0),
        (r"(?<whole>(c[a--b]))", JsString::from("ccca"), "dvy", 0),
        (r"(?<whole>(k[[s]]))", JsString::from("Kſ!"), "divg", 0),
        (r"(?<whole>(k[[s]]))", JsString::from("kſ!"), "dvg", 0),
        (
            r"(?<whole>(\u{1f600}[[\p{Any}]]))",
            JsString::from("😀😁!"),
            "dvy",
            1,
        ),
        (
            r"(?<whole>(\uD800[[a]]))",
            JsString::from_code_units(vec![0xd800, 0x61, 0x21]),
            "dvg",
            0,
        ),
        (
            r"(?<whole>(\uD83D[[\uDE00]]))",
            JsString::from("😀"),
            "dvg",
            0,
        ),
        (r"(?<whole>(\[[[a]]))", JsString::from("x[a!"), "dvg", 0),
        (
            r"^(?<whole>(c[[a]]))$",
            JsString::from("x\r\nca\u{2028}"),
            "dvmg",
            0,
        ),
        (r"^(?<whole>(\$[[a]]))$", JsString::from("$a"), "dvg", 0),
    ] {
        let pattern = JsString::from(pattern);
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
        row_count += 1;
    }
    assert_eq!(row_count, 10);
    insta::assert_snapshot!(rows);
}

#[test]
fn checked_v_set_pair_word_state_context_and_consumers() {
    // ECMA-262 22.2.2.4 tests these assertions at the complete pair's edges.
    // A failed assertion must retry at the first character's end: the valid
    // overlapping pair begins at 1, although the first pair ends at 2.
    check(
        r"let a=/\B[[a]]a\b/dvg,b=/\Ba[[a]]\b/dvg,x=a.exec('aaa!'),y=b.exec('aaa!');x[0]==='aa'&&x.index===1&&x.indices[0][1]===3&&a.lastIndex===3&&y[0]==='aa'&&y.index===1&&y.indices[0][1]===3&&b.lastIndex===3&&a.exec('aaa!')===null&&a.lastIndex===0&&b.exec('aaa!')===null&&b.lastIndex===0",
    );
    check(
        r"let a=/\B[[a]]a\b/vy,b=/\Ba[[a]]\b/vy;a.exec('aaa!')===null&&a.lastIndex===0&&b.exec('aaa!')===null&&b.lastIndex===0&&(a.lastIndex=1,b.lastIndex=1,a.exec('aaa!')[0]==='aa')&&a.lastIndex===3&&b.exec('aaa!')[0]==='aa'&&b.lastIndex===3",
    );
    for (source, input, nonmember) in [(r"\b[a--b]c", "ac", "bc"), (r"\bc[a--b]", "ca", "cb")] {
        let code = JsString::from(format!("marker=1; /{source}/v.test('{input}');").as_str());
        check(&format!(
            "let marker=0,result,caught=false;try{{result=eval({code:?})}}catch{{caught=true}}finally{{marker=3}}!caught&&marker===3&&result&&!/{source}/v.test('{nonmember}')"
        ));
    }
    // Any is fold-invariant, but WordCharacters still depends on actual i.
    check(
        r"/\B[[\p{Any}]]a\b/v.test('Ka!')&&!/\B[[\p{Any}]]a\b/iv.test('Ka!')&&!/\b[[\p{Any}]]a\b/v.test('Ka!')&&/\b[[\p{Any}]]a\b/iv.test('Ka!')&&/\b[[\p{Any}]]a\b/iv.test('ſa!')&&!/\b[[\p{Any}]]a\b/v.test('ſa!')",
    );
    check(
        r"!/\ba[[\p{Any}]]\b/v.test('aK!')&&/\ba[[\p{Any}]]\b/iv.test('aK!')&&!/\ba[[\p{Any}]]\b/v.test('aſ!')&&/\ba[[\p{Any}]]\b/iv.test('aſ!')&&/\b[[K]]s\b/iv.test('Kſ!')&&/\bk[[s]]\b/iv.test('Kſ!')&&!/\b[[K]]s\b/v.test('Kſ!')&&!/\bk[[s]]\b/v.test('Ks!')",
    );
    check(
        r"/[[a]]c\b/v.exec('xac!').index===1&&/a[[a]]\b/v.exec('aaa!').index===1&&/\B[[!]]\?\B/v.test('!?')&&/\B\?[[!]]\B/v.test('?!')&&!/\b[a--a]c/v.test('ac')&&!/\bc[a--a]/v.test('ca')",
    );
    check(
        r"let a=/\B[[\p{Any}]]a\b/dvy,b=/\B\u{1f600}[[a]]\b/dvy;a.lastIndex=1;b.lastIndex=1;let x=a.exec('😀a!'),y=b.exec('😀a!');x.index===0&&x[0]==='😀a'&&x.indices[0][0]===0&&x.indices[0][1]===3&&a.lastIndex===3&&y.index===0&&y[0]==='😀a'&&y.indices[0][1]===3&&b.lastIndex===3",
    );
    check(
        r"let a=/\b[[\uD800]]a\b/dv,b=/\b\uD800[[a]]\b/dv,x=a.exec('A\uD800a!'),y=b.exec('A\uD800a!');x.index===1&&x[0]==='\uD800a'&&x.indices[0][1]===3&&y.index===1&&y[0]==='\uD800a'&&y.indices[0][1]===3&&/\B[[\uD83D]]\uDE00\B/v.exec('😀')===null&&/\B\uD83D[[\uDE00]]\B/v.exec('😀')===null",
    );
    check(
        r"/^\b[[a]]\$\B$/v.test('a$')&&!/^\b[[a]]\$\B$/v.test('a$\n')&&/^\B\$[[a]]\b$/v.test('$a')&&!/^\B\$[[a]]\b$/v.test('$a\n')",
    );
    check(
        r"let a=/^\b[[a]]c\b$/dvmg,b=/^\bc[[a]]\b$/dvmg,x=a.exec('x\r\nac\u2028bc'),y=b.exec('x\r\nca\u2028cb');x.index===3&&x[0]==='ac'&&x.indices[0][1]===5&&a.lastIndex===5&&y.index===3&&y[0]==='ca'&&y.indices[0][1]===5&&b.lastIndex===5&&a.exec('x\r\nac\u2028bc')===null&&a.lastIndex===0&&b.exec('x\r\nca\u2028cb')===null&&b.lastIndex===0",
    );
    check(
        r"let r=/(?<whole>(\B[[a]]a\b))/dvg,a=[...'aaa! aaa!'.matchAll(r)];a.length===2&&a[0][0]==='aa'&&a[0][1]==='aa'&&a[0][2]==='aa'&&a[0].index===1&&a[0].indices.groups.whole===a[0].indices[1]&&a[0].indices[2][1]===3&&a[1].groups.whole==='aa'&&a[1].index===6&&a[1].indices[2][1]===8&&r.lastIndex===0",
    );
    check(
        r"'aaa! aaa!'.replace(/(?<whole>(\B[[a]]a\b))/vg,'<$1:$<whole>>')==='a<aa:aa>! a<aa:aa>!'&&'aaa!'.split(/(\Ba[[a]]\b)/v).join('|')==='a|aa|!'&&'aaa!'.match(/\Ba[[a]]\b/v)[0]==='aa'",
    );
    check(
        r"let seen='',out='aaa! aaa!'.replace(/(?<whole>(\Ba[[a]]\b))/vg,(m,c1,c2,p,text,g)=>{seen+=p+':'+g.whole+';';return 'X'});out==='aX! aX!'&&seen==='1:aa;6:aa;'",
    );
    check(r"let r=/\B[[a]]a\b/vg;r.lastIndex=4;'aaa!'.search(r)===1&&r.lastIndex===4");
    check(
        r"let trace='',r=/\B\u{1f600}[[a]]\b/vy;r.lastIndex={valueOf(){trace+='i';return 1}};let m=r.exec({toString(){trace+='s';return '😀a!'}});trace==='si'&&m.index===0&&m[0]==='😀a'&&r.lastIndex===3",
    );
    check(
        r"let seen=0,r=/\B[[a]]a\b/v;r.lastIndex={valueOf(){seen++;return 1}};let m=r.exec('aaa!');m.index===1&&m[0]==='aa'&&seen===1&&typeof r.lastIndex==='object'",
    );
    check(
        r"let r=/\b[[K]]s\b/iv,c=new RegExp(r,'dvy');c.source===r.source&&r.source==='\\b[[K]]s\\b'&&c.unicodeSets&&!c.ignoreCase&&c.exec('Kſ')===null&&c.lastIndex===0&&r.exec('Kſ')[0]==='Kſ'",
    );
    check(
        r"let a=/\B[[a]]a\b/vy,b=/\Ba[[a]]\b/vy;a.lastIndex=1;Object.defineProperty(a,'lastIndex',{writable:false});Object.defineProperty(b,'lastIndex',{writable:false});let x=false,y=false;try{a.exec('aaa!')}catch(e){x=e instanceof TypeError}try{b.exec('aaa!')}catch(e){y=e instanceof TypeError}x&&y&&a.lastIndex===1&&b.lastIndex===0",
    );
    for source in [
        r"/\b^[[a]]c/v.test('ac')",
        r"/[[a]]c$\b/v.test('ac')",
        r"/\b\b[[a]]c/v.test('ac')",
        r"/\b[[a]]\Bc/v.test('ac')",
        r"/\ba[[a]]+/v.test('aa')",
        r"/(\b[[a]])c/v.test('ac')",
        r"/\ba([[a]])/v.test('aa')",
        r"/(\b[[a]]c)+/v.test('ac')",
        r"/\b[[a]]cd/v.test('acd')",
        r"/\ba[[a]][c]/v.test('aac')",
        r"/\b[[a--a]&&[\q{a}]]c/v.test('ac')",
        r"/\ba[[a--a]&&[\p{RGI_Emoji}]]/v.test('aa')",
    ] {
        assert!(
            matches!(
                Realm::default().eval(source),
                Err(Error::Unsupported { .. })
            ),
            "{source}"
        );
    }
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm.eval(r"var r=/\B[[a]]a\b/vg,marker=0").unwrap();
    // Every pair before the final one fails an outside assertion. Build this
    // input outside JavaScript's opted-in string-construction work.
    let setup = format!("var text='{}';r.lastIndex=1", "a".repeat(20_000));
    realm.eval(&setup).unwrap();
    assert!(matches!(
        realm.eval("try{r.exec(text)}catch{marker=1}finally{marker=2}"),
        Err(Error::Limit { .. })
    ));
    assert_eq!(
        realm.eval("marker===0&&r.lastIndex===1"),
        Ok(Value::Boolean(true))
    );
}

#[test]
fn checked_v_set_pair_word_results() {
    let mut rows = String::new();
    let mut row_count = 0;
    // These full expected objects are specified independently of the matcher.
    // Explicit UTF-16 input units keep lone surrogates and start offsets visible.
    for (pattern, input, flags, start, expected) in [
        (
            r"(?<whole>(\B[[a]]a\b))",
            JsString::from("aaa!"),
            "dvg",
            0,
            r#"{"matches":["aa","aa","aa"],"index":1,"input":"aaa!","groups":{"whole":"aa"},"indices":[[1,3],[1,3],[1,3]],"indexGroups":{"whole":[1,3]},"lastIndex":3,"source":"(?<whole>(\\B[[a]]a\\b))"}"#,
        ),
        (
            r"(?<whole>(\Ba[[a]]\b))",
            JsString::from("aaa!"),
            "dvg",
            0,
            r#"{"matches":["aa","aa","aa"],"index":1,"input":"aaa!","groups":{"whole":"aa"},"indices":[[1,3],[1,3],[1,3]],"indexGroups":{"whole":[1,3]},"lastIndex":3,"source":"(?<whole>(\\Ba[[a]]\\b))"}"#,
        ),
        (
            r"(?<whole>(\B[[a]]a\b))",
            JsString::from("aaa!"),
            "dvy",
            0,
            r#"{"match":null,"lastIndex":0}"#,
        ),
        (
            r"(?<whole>(\B[[\p{Any}]]a\b))",
            JsString::from("Ka!"),
            "dvg",
            0,
            r#"{"matches":["Ka","Ka","Ka"],"index":0,"input":"Ka!","groups":{"whole":"Ka"},"indices":[[0,2],[0,2],[0,2]],"indexGroups":{"whole":[0,2]},"lastIndex":2,"source":"(?<whole>(\\B[[\\p{Any}]]a\\b))"}"#,
        ),
        (
            r"(?<whole>(\B[[\p{Any}]]a\b))",
            JsString::from("Ka!"),
            "divg",
            0,
            r#"{"match":null,"lastIndex":0}"#,
        ),
        (
            r"(?<whole>(\B\u{1f600}[[a]]\b))",
            JsString::from("😀a!"),
            "dvy",
            1,
            r#"{"matches":["😀a","😀a","😀a"],"index":0,"input":"😀a!","groups":{"whole":"😀a"},"indices":[[0,3],[0,3],[0,3]],"indexGroups":{"whole":[0,3]},"lastIndex":3,"source":"(?<whole>(\\B\\u{1f600}[[a]]\\b))"}"#,
        ),
        (
            r"(?<whole>(\b[[\uD800]]a\b))",
            JsString::from_code_units(vec![0x41, 0xd800, 0x61, 0x21]),
            "dvg",
            0,
            r#"{"matches":["\ud800a","\ud800a","\ud800a"],"index":1,"input":"A\ud800a!","groups":{"whole":"\ud800a"},"indices":[[1,3],[1,3],[1,3]],"indexGroups":{"whole":[1,3]},"lastIndex":3,"source":"(?<whole>(\\b[[\\uD800]]a\\b))"}"#,
        ),
        (
            r"(?<whole>(\b[[a]]\$\B))",
            JsString::from("a$!"),
            "dvg",
            0,
            r#"{"matches":["a$","a$","a$"],"index":0,"input":"a$!","groups":{"whole":"a$"},"indices":[[0,2],[0,2],[0,2]],"indexGroups":{"whole":[0,2]},"lastIndex":2,"source":"(?<whole>(\\b[[a]]\\$\\B))"}"#,
        ),
        (
            r"(?<whole>(\B\$[[a]]\b))",
            JsString::from("$a!"),
            "dvg",
            0,
            r#"{"matches":["$a","$a","$a"],"index":0,"input":"$a!","groups":{"whole":"$a"},"indices":[[0,2],[0,2],[0,2]],"indexGroups":{"whole":[0,2]},"lastIndex":2,"source":"(?<whole>(\\B\\$[[a]]\\b))"}"#,
        ),
        (
            r"^(?<whole>(\b[[a]]c\b))$",
            JsString::from("x\r\nac\u{2028}bc"),
            "dvmg",
            0,
            r#"{"matches":["ac","ac","ac"],"index":3,"input":"x\r\nac bc","groups":{"whole":"ac"},"indices":[[3,5],[3,5],[3,5]],"indexGroups":{"whole":[3,5]},"lastIndex":5,"source":"^(?<whole>(\\b[[a]]c\\b))$"}"#,
        ),
    ] {
        let pattern = JsString::from(pattern);
        let script = format!(
            "let r=new RegExp({pattern:?},'{flags}');r.lastIndex={start};let m=r.exec({input:?});JSON.stringify(m===null?{{match:null,lastIndex:r.lastIndex}}:{{matches:[...m],index:m.index,input:m.input,groups:m.groups,indices:m.indices,indexGroups:m.indices.groups,lastIndex:r.lastIndex,source:r.source}})"
        );
        let Value::String(result) = Realm::default().eval(&script).unwrap() else {
            panic!("{script}")
        };
        assert_eq!(result, JsString::from(expected), "{script}");
        writeln!(
            rows,
            "{pattern:?} flags={flags:?} input_units={:?} lastIndex={start} {result:?}",
            input.code_units(),
        )
        .unwrap();
        row_count += 1;
    }
    assert_eq!(row_count, 10);
    insta::assert_snapshot!(rows);
}
