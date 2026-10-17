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

#[test]
fn checked_v_set_class_pair_state_context_and_consumers() {
    check(
        r"/[a--b][[c]]/v.test('ac')&&!/[a--b][[c]]/v.test('ab')&&/[[a]][[b]]/v.test('ab')&&!/[a--a][[b]]/v.test('ab')&&!/[[a]][b--b]/v.test('ab')",
    );
    check(
        r"let g=/[a--b][[c]]/vg,y=/[a--b][[c]]/vy,a=g.exec('aac');a.index===1&&a[0]==='ac'&&g.lastIndex===3&&g.exec('aac')===null&&g.lastIndex===0&&y.exec('aac')===null&&y.lastIndex===0&&(y.lastIndex=1,y.exec('aac')[0]==='ac')&&y.lastIndex===3",
    );
    check(
        r"/[[K]][[s]]/iv.test('Kſ')&&!/[[K]][[s]]/v.test('Kſ')&&/[[^K]][[s]]/iv.test('as')&&!/[[^K]][[s]]/iv.test('Ks')&&/[\P{Ll}&&A][[s]]/v.test('As')&&!/[\P{Ll}&&A][[s]]/iv.test('As')",
    );
    check(
        r"let a=/[[\p{Any}]][[a]]/dvy,b=/[[\u{1f600}]][[\u{1f601}]]/dvy;a.lastIndex=1;b.lastIndex=1;let x=a.exec('😀a!'),y=b.exec('😀😁');x.index===0&&x[0]==='😀a'&&x.indices[0][1]===3&&a.lastIndex===3&&y.index===0&&y[0]==='😀😁'&&y.indices[0][1]===4&&b.lastIndex===4",
    );
    check(
        r"let r=/[[\uD800]][[\uD800]]/dv,m=r.exec('A\uD800\uD800!');m.index===1&&m[0]==='\uD800\uD800'&&m.indices[0][1]===3&&!/[[\uD800]][[\uDC00]]/v.test('\uD800\uDC00')&&/[[\]]][[\[]]/v.test('][')",
    );
    check(
        r"let r=/^[[a]][[b]]$/dvmg,m=r.exec('x\r\nab\u2028cd');m.index===3&&m[0]==='ab'&&m.indices[0][1]===5&&r.lastIndex===5&&r.exec('x\r\nab\u2028cd')===null&&r.lastIndex===0&&!/^[[a]][[b]]$/v.test('ab\n')",
    );
    check(
        r"let r=/(?<whole>([a--b][[c]]))/dvg,a=[...'aac aac'.matchAll(r)];a.length===2&&a[0][0]==='ac'&&a[0][1]==='ac'&&a[0][2]==='ac'&&a[0].index===1&&a[0].groups.whole==='ac'&&a[0].indices.groups.whole===a[0].indices[1]&&a[0].indices[2][1]===3&&a[1].index===5&&a[1].indices[2][1]===7&&r.lastIndex===0",
    );
    check(
        r"'aac aac'.replace(/(?<whole>([a--b][[c]]))/vg,'<$1:$<whole>>')==='a<ac:ac> a<ac:ac>'&&'aac'.split(/([a--b][[c]])/v).join('|')==='a|ac|'&&'aac'.match(/[a--b][[c]]/v)[0]==='ac'",
    );
    check(
        r"let seen='',out='aac aac'.replace(/(?<whole>([a--b][[c]]))/vg,(m,c1,c2,p,text,g)=>{seen+=p+':'+g.whole+';';return 'X'});out==='aX aX'&&seen==='1:ac;5:ac;'",
    );
    check(r"let r=/[a--b][[c]]/vg;r.lastIndex=4;'aac'.search(r)===1&&r.lastIndex===4");
    check(
        r"let trace='',r=/[[\p{Any}]][[a]]/vy;r.lastIndex={valueOf(){trace+='i';return 1}};let m=r.exec({toString(){trace+='s';return '😀a!'}});trace==='si'&&m.index===0&&m[0]==='😀a'&&r.lastIndex===3",
    );
    check(
        r"let seen=0,r=/[a--b][[c]]/v;r.lastIndex={valueOf(){seen++;return 1}};let m=r.exec('aac');m.index===1&&seen===1&&typeof r.lastIndex==='object'",
    );
    check(
        r"let r=/[[K]][[s]]/iv,c=new RegExp(r,'dvy');c.source===r.source&&r.source==='[[K]][[s]]'&&c.unicodeSets&&!c.ignoreCase&&c.exec('Kſ')===null&&c.lastIndex===0&&r.exec('Kſ')[0]==='Kſ'",
    );
    check(
        r"let a=/[a--b][[c]]/vy,b=/[a--b][[c]]/vy;a.lastIndex=1;Object.defineProperty(a,'lastIndex',{writable:false});Object.defineProperty(b,'lastIndex',{writable:false});let x=false,y=false;try{a.exec('aac')}catch(e){x=e instanceof TypeError}try{b.exec('aac')}catch(e){y=e instanceof TypeError}x&&y&&a.lastIndex===1&&b.lastIndex===0",
    );
    check(r"/[[a]][[b]][[c]]/v.test('abc')");
    check(
        r"let r=/[[a]][[b]][[c]]/v,m=r.exec('abc');m.length===1&&m[0]==='abc'&&m.index===0&&m.input==='abc'&&m.groups===undefined&&m.indices===undefined&&r.source==='[[a]][[b]][[c]]'&&r.lastIndex===0&&!r.test('xbc')&&!r.test('axc')&&!r.test('abx')&&!r.test('ab')",
    );
    for source in [
        r"/c[a--b][c]/v.test('cac')",
        r"/[[a]]\b[[b]]/v.test('ab')",
        r"/[[a]]^[[b]]/v.test('ab')",
        r"/([[a]])[[b]]/v.test('ab')",
        r"/[[a]]([[b]])/v.test('ab')",
        r"/[[a]]+[[b]]/v.test('ab')",
        r"/[[a]][[b]]?/v.test('ab')",
        r"/[a--a][[\q{b}]]/v.test('ab')",
        r"/[a--a][[\p{RGI_Emoji}]]/v.test('ab')",
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
    realm
        .eval(r"var r=/[a--b][[c]]/vg,sticky=/[[a]][[a]]/vy,marker=0")
        .unwrap();
    let setup = format!("var text='{}';r.lastIndex=1", "a".repeat(20_000));
    realm.eval(&setup).unwrap();
    assert_eq!(
        realm.eval("sticky.test(text)&&sticky.lastIndex===2"),
        Ok(Value::Boolean(true))
    );
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
fn checked_v_set_class_pair_results() {
    let mut rows = String::new();
    let mut row_count = 0;
    // Complete objects and UTF-16 spans are hand-specified before execution.
    for (pattern, input, flags, start, expected) in [
        (
            r"(?<whole>([a--b][[c]]))",
            JsString::from("aac"),
            "dvg",
            0,
            r#"{"matches":["ac","ac","ac"],"index":1,"input":"aac","groups":{"whole":"ac"},"indices":[[1,3],[1,3],[1,3]],"indexGroups":{"whole":[1,3]},"lastIndex":3,"source":"(?<whole>([a--b][[c]]))"}"#,
        ),
        (
            r"(?<whole>([a--b][[c]]))",
            JsString::from("aac"),
            "dvy",
            0,
            r#"{"match":null,"lastIndex":0}"#,
        ),
        (
            r"(?<whole>([a--b][[c]]))",
            JsString::from("aac"),
            "dvy",
            1,
            r#"{"matches":["ac","ac","ac"],"index":1,"input":"aac","groups":{"whole":"ac"},"indices":[[1,3],[1,3],[1,3]],"indexGroups":{"whole":[1,3]},"lastIndex":3,"source":"(?<whole>([a--b][[c]]))"}"#,
        ),
        (
            r"(?<whole>([[K]][[s]]))",
            JsString::from("Kſ!"),
            "divg",
            0,
            r#"{"matches":["Kſ","Kſ","Kſ"],"index":0,"input":"Kſ!","groups":{"whole":"Kſ"},"indices":[[0,2],[0,2],[0,2]],"indexGroups":{"whole":[0,2]},"lastIndex":2,"source":"(?<whole>([[K]][[s]]))"}"#,
        ),
        (
            r"(?<whole>([\P{Ll}&&A][[s]]))",
            JsString::from("As"),
            "divg",
            0,
            r#"{"match":null,"lastIndex":0}"#,
        ),
        (
            r"(?<whole>([[\p{Any}]][[a]]))",
            JsString::from("😀a!"),
            "dvy",
            1,
            r#"{"matches":["😀a","😀a","😀a"],"index":0,"input":"😀a!","groups":{"whole":"😀a"},"indices":[[0,3],[0,3],[0,3]],"indexGroups":{"whole":[0,3]},"lastIndex":3,"source":"(?<whole>([[\\p{Any}]][[a]]))"}"#,
        ),
        (
            r"(?<whole>([[\u{1f600}]][[\u{1f601}]]))",
            JsString::from("x😀😁!"),
            "dvg",
            0,
            r#"{"matches":["😀😁","😀😁","😀😁"],"index":1,"input":"x😀😁!","groups":{"whole":"😀😁"},"indices":[[1,5],[1,5],[1,5]],"indexGroups":{"whole":[1,5]},"lastIndex":5,"source":"(?<whole>([[\\u{1f600}]][[\\u{1f601}]]))"}"#,
        ),
        (
            r"(?<whole>([[\uD800]][[\uD800]]))",
            JsString::from_code_units(vec![0x41, 0xd800, 0xd800, 0x21]),
            "dvg",
            0,
            r#"{"matches":["\ud800\ud800","\ud800\ud800","\ud800\ud800"],"index":1,"input":"A\ud800\ud800!","groups":{"whole":"\ud800\ud800"},"indices":[[1,3],[1,3],[1,3]],"indexGroups":{"whole":[1,3]},"lastIndex":3,"source":"(?<whole>([[\\uD800]][[\\uD800]]))"}"#,
        ),
        (
            r"(?<whole>([[\]]][[\[]]))",
            JsString::from("x][!"),
            "dvg",
            0,
            r#"{"matches":["][","][","]["],"index":1,"input":"x][!","groups":{"whole":"]["},"indices":[[1,3],[1,3],[1,3]],"indexGroups":{"whole":[1,3]},"lastIndex":3,"source":"(?<whole>([[\\]]][[\\[]]))"}"#,
        ),
        (
            r"^(?<whole>([[a]][[b]]))$",
            JsString::from("x\r\nab\u{2028}cd"),
            "dvmg",
            0,
            r#"{"matches":["ab","ab","ab"],"index":3,"input":"x\r\nab cd","groups":{"whole":"ab"},"indices":[[3,5],[3,5],[3,5]],"indexGroups":{"whole":[3,5]},"lastIndex":5,"source":"^(?<whole>([[a]][[b]]))$"}"#,
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
            input.code_units()
        )
        .unwrap();
        row_count += 1;
    }
    assert_eq!(row_count, 10);
    insta::assert_snapshot!(rows);
}

#[test]
fn checked_v_set_class_pair_word_state_context_and_consumers() {
    check(
        r"/\b[[a]][[b]]/v.test('ab')&&!/\b[[a]][[b]]/v.test('xab')&&/[[a]][[b]]\b/v.test('ab')&&!/[[a]][[b]]\b/v.test('abx')",
    );
    check(
        r"let g=/\B[[a]][[a]]\b/vg,y=/\B[[a]][[a]]\b/vy,a=g.exec('aaa!');a.index===1&&a[0]==='aa'&&g.lastIndex===3&&g.exec('aaa!')===null&&g.lastIndex===0&&y.exec('aaa!')===null&&y.lastIndex===0&&(y.lastIndex=1,y.exec('aaa!')[0]==='aa')&&y.lastIndex===3",
    );
    check(
        r"/\B[[\p{Any}]][[a]]\b/v.test('Ka!')&&!/\B[[\p{Any}]][[a]]\b/iv.test('Ka!')&&!/\b[[\p{Any}]][[a]]\b/v.test('Ka!')&&/\b[[\p{Any}]][[a]]\b/iv.test('Ka!')&&/\b[[K]][[s]]\b/iv.test('Kſ!')&&!/\b[[K]][[s]]\b/v.test('Kſ!')",
    );
    check(
        r"!/\b[[a]][[\p{Any}]]\b/v.test('aK!')&&/\b[[a]][[\p{Any}]]\b/iv.test('aK!')&&!/\b[[a]][[\p{Any}]]\b/v.test('aſ!')&&/\b[[a]][[\p{Any}]]\b/iv.test('aſ!')&&/\B[[\p{Any}]][[a]]\b/iv.test('αa!')&&/\B[[!]][[\?]]\B/v.test('!?')",
    );
    check(
        r"!/\b[a--a][[b]]/v.test('ab')&&!/\b[[a]][b--b]/v.test('ab')&&/\b[\P{Ll}&&A][[s]]\b/v.test('As!')&&!/\b[\P{Ll}&&A][[s]]\b/iv.test('As!')",
    );
    check(
        r"let r=/\B[[\p{Any}]][[a]]\b/dvy;r.lastIndex=1;let m=r.exec('😀a!');m.index===0&&m[0]==='😀a'&&m.indices[0][1]===3&&r.lastIndex===3&&/\B[[\u{1f600}]][[\u{1f601}]]\B/v.exec('!😀😁!').index===1",
    );
    check(
        r"let r=/\b[[\uD800]][[\uD800]]\B/dv,m=r.exec('A\uD800\uD800!');m.index===1&&m[0]==='\uD800\uD800'&&m.indices[0][1]===3&&!/\B[[\uD800]][[\uDC00]]\B/v.test('\uD800\uDC00')",
    );
    check(
        r"let r=/^\b[[a]][[b]]\b$/dvmg,m=r.exec('x\r\nab\u2028cd');m.index===3&&m[0]==='ab'&&m.indices[0][1]===5&&r.lastIndex===5&&r.exec('x\r\nab\u2028cd')===null&&r.lastIndex===0&&!/^\b[[a]][[b]]\b$/v.test('ab\n')",
    );
    check(
        r"let r=/(?<whole>(\B[[a]][[a]]\b))/dvg,a=[...'aaa! aaa!'.matchAll(r)];a.length===2&&a[0][0]==='aa'&&a[0][1]==='aa'&&a[0][2]==='aa'&&a[0].index===1&&a[0].groups.whole==='aa'&&a[0].indices.groups.whole===a[0].indices[1]&&a[0].indices[2][1]===3&&a[1].index===6&&a[1].indices[2][1]===8&&r.lastIndex===0",
    );
    check(
        r"'aaa! aaa!'.replace(/(?<whole>(\B[[a]][[a]]\b))/vg,'<$1:$<whole>>')==='a<aa:aa>! a<aa:aa>!'&&'aaa!'.split(/(\B[[a]][[a]]\b)/v).join('|')==='a|aa|!'&&'aaa!'.match(/\B[[a]][[a]]\b/v)[0]==='aa'",
    );
    check(
        r"let seen='',out='aaa! aaa!'.replace(/(?<whole>(\B[[a]][[a]]\b))/vg,(m,c1,c2,p,text,g)=>{seen+=p+':'+g.whole+';';return 'X'});out==='aX! aX!'&&seen==='1:aa;6:aa;'",
    );
    check(r"let r=/\B[[a]][[a]]\b/vg;r.lastIndex=4;'aaa!'.search(r)===1&&r.lastIndex===4");
    check(
        r"let trace='',r=/\B[[\p{Any}]][[a]]\b/vy;r.lastIndex={valueOf(){trace+='i';return 1}};let m=r.exec({toString(){trace+='s';return '😀a!'}});trace==='si'&&m.index===0&&m[0]==='😀a'&&r.lastIndex===3",
    );
    check(
        r"let seen=0,r=/\B[[a]][[a]]\b/v;r.lastIndex={valueOf(){seen++;return 1}};let m=r.exec('aaa!');m.index===1&&seen===1&&typeof r.lastIndex==='object'",
    );
    check(
        r"let r=/\b[[K]][[s]]\b/iv,c=new RegExp(r,'dvy');c.source===r.source&&r.source==='\\b[[K]][[s]]\\b'&&c.unicodeSets&&!c.ignoreCase&&c.exec('Kſ!')===null&&c.lastIndex===0&&r.exec('Kſ!')[0]==='Kſ'",
    );
    check(
        r"let a=/\B[[a]][[a]]\b/vy,b=/\B[[a]][[a]]\b/vy;a.lastIndex=1;Object.defineProperty(a,'lastIndex',{writable:false});Object.defineProperty(b,'lastIndex',{writable:false});let x=false,y=false;try{a.exec('aaa!')}catch(e){x=e instanceof TypeError}try{b.exec('aaa!')}catch(e){y=e instanceof TypeError}x&&y&&a.lastIndex===1&&b.lastIndex===0",
    );
    check(r"/\b[a][b--c]/v.test('ab')&&!/\b[a][b--c]/v.test('ac')&&!/\b[a][b--c]/v.test('xab')");
    for source in [
        r"/\b[[a]][[b]][[c]]/v.test('abc')",
        r"/[[a]]\b[[b]]/v.test('ab')",
        r"/\b^[[a]][[b]]/v.test('ab')",
        r"/[[a]][[b]]$\b/v.test('ab')",
        r"/\b\b[[a]][[b]]/v.test('ab')",
        r"/(\b[[a]])[[b]]/v.test('ab')",
        r"/\b[[a]]([[b]])/v.test('ab')",
        r"/\b[[a]]+[[b]]/v.test('ab')",
        r"/\b[[a]][[b]]?/v.test('ab')",
        r"/\b[a--a][[\q{b}]]/v.test('ab')",
        r"/\b[a--a][[\p{RGI_Emoji}]]/v.test('ab')",
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
    realm
        .eval(r"var r=/\B[[a]][[a]]\b/vg,sticky=/\b[[a]][[a]]/vy,marker=0")
        .unwrap();
    let setup = format!("var text='{}';r.lastIndex=1", "a".repeat(20_000));
    realm.eval(&setup).unwrap();
    assert_eq!(
        realm.eval("sticky.test(text)&&sticky.lastIndex===2"),
        Ok(Value::Boolean(true))
    );
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
fn checked_v_set_class_pair_word_results() {
    let mut rows = String::new();
    let mut row_count = 0;
    // Full field objects and original UTF-16 spans are hand-specified.
    for (pattern, input, flags, start, expected) in [
        (
            r"(?<whole>(\B[[a]][[a]]\b))",
            JsString::from("aaa!"),
            "dvg",
            0,
            r#"{"matches":["aa","aa","aa"],"index":1,"input":"aaa!","groups":{"whole":"aa"},"indices":[[1,3],[1,3],[1,3]],"indexGroups":{"whole":[1,3]},"lastIndex":3,"source":"(?<whole>(\\B[[a]][[a]]\\b))"}"#,
        ),
        (
            r"(?<whole>(\B[[a]][[a]]\b))",
            JsString::from("aaa!"),
            "dvy",
            0,
            r#"{"match":null,"lastIndex":0}"#,
        ),
        (
            r"(?<whole>(\b[[K]][[s]]\b))",
            JsString::from("Kſ!"),
            "divg",
            0,
            r#"{"matches":["Kſ","Kſ","Kſ"],"index":0,"input":"Kſ!","groups":{"whole":"Kſ"},"indices":[[0,2],[0,2],[0,2]],"indexGroups":{"whole":[0,2]},"lastIndex":2,"source":"(?<whole>(\\b[[K]][[s]]\\b))"}"#,
        ),
        (
            r"(?<whole>(\B[[\p{Any}]][[a]]\b))",
            JsString::from("Ka!"),
            "dvg",
            0,
            r#"{"matches":["Ka","Ka","Ka"],"index":0,"input":"Ka!","groups":{"whole":"Ka"},"indices":[[0,2],[0,2],[0,2]],"indexGroups":{"whole":[0,2]},"lastIndex":2,"source":"(?<whole>(\\B[[\\p{Any}]][[a]]\\b))"}"#,
        ),
        (
            r"(?<whole>(\B[[\p{Any}]][[a]]\b))",
            JsString::from("Ka!"),
            "divg",
            0,
            r#"{"match":null,"lastIndex":0}"#,
        ),
        (
            r"(?<whole>(\B[[\p{Any}]][[a]]\b))",
            JsString::from("😀a!"),
            "dvy",
            1,
            r#"{"matches":["😀a","😀a","😀a"],"index":0,"input":"😀a!","groups":{"whole":"😀a"},"indices":[[0,3],[0,3],[0,3]],"indexGroups":{"whole":[0,3]},"lastIndex":3,"source":"(?<whole>(\\B[[\\p{Any}]][[a]]\\b))"}"#,
        ),
        (
            r"(?<whole>(\b[[\uD800]][[\uD800]]\B))",
            JsString::from_code_units(vec![0x41, 0xd800, 0xd800, 0x21]),
            "dvg",
            0,
            r#"{"matches":["\ud800\ud800","\ud800\ud800","\ud800\ud800"],"index":1,"input":"A\ud800\ud800!","groups":{"whole":"\ud800\ud800"},"indices":[[1,3],[1,3],[1,3]],"indexGroups":{"whole":[1,3]},"lastIndex":3,"source":"(?<whole>(\\b[[\\uD800]][[\\uD800]]\\B))"}"#,
        ),
        (
            r"(?<whole>(\B[[!]][[\?]]\B))",
            JsString::from("!?"),
            "dvg",
            0,
            r#"{"matches":["!?","!?","!?"],"index":0,"input":"!?","groups":{"whole":"!?"},"indices":[[0,2],[0,2],[0,2]],"indexGroups":{"whole":[0,2]},"lastIndex":2,"source":"(?<whole>(\\B[[!]][[\\?]]\\B))"}"#,
        ),
        (
            r"(?<whole>(\B[[\u{1f600}]][[\u{1f601}]]\B))",
            JsString::from("!😀😁!"),
            "dvg",
            0,
            r#"{"matches":["😀😁","😀😁","😀😁"],"index":1,"input":"!😀😁!","groups":{"whole":"😀😁"},"indices":[[1,5],[1,5],[1,5]],"indexGroups":{"whole":[1,5]},"lastIndex":5,"source":"(?<whole>(\\B[[\\u{1f600}]][[\\u{1f601}]]\\B))"}"#,
        ),
        (
            r"^(?<whole>(\b[[a]][[b]]\b))$",
            JsString::from("x\r\nab\u{2028}cd"),
            "dvmg",
            0,
            r#"{"matches":["ab","ab","ab"],"index":3,"input":"x\r\nab cd","groups":{"whole":"ab"},"indices":[[3,5],[3,5],[3,5]],"indexGroups":{"whole":[3,5]},"lastIndex":5,"source":"^(?<whole>(\\b[[a]][[b]]\\b))$"}"#,
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
            input.code_units()
        )
        .unwrap();
        row_count += 1;
    }
    assert_eq!(row_count, 10);
    insta::assert_snapshot!(rows);
}

#[test]
fn checked_then_flat_v_set_pair_state_context_and_consumers() {
    check(
        r"/[a--b][c]/v.test('ac')&&!/[a--b][c]/v.test('ab')&&/[[a]][bcd]/v.test('ad')&&/[[a]][b-d]/iv.test('aD')&&!/[[a]][b-d]/v.test('aD')",
    );
    check(r#"let marker=0;eval("marker=1; /[a--b][c]/v.test('ac')")&&marker===1"#);
    check(
        r"let g=/[a--b][c]/vg,y=/[a--b][c]/vy,a=g.exec('aac');a.index===1&&a[0]==='ac'&&g.lastIndex===3&&g.exec('aac')===null&&g.lastIndex===0&&y.exec('aac')===null&&y.lastIndex===0&&(y.lastIndex=1,y.exec('aac')[0]==='ac')&&y.lastIndex===3",
    );
    check(
        r"/[[K]][s]/iv.test('Kſ')&&!/[[K]][s]/v.test('Kſ')&&!/[[a]][^K]/iv.test('aK')&&/[[a]][^K]/iv.exec('aKaſ')[0]==='aſ'&&/[[a]][\p{ASCII}]/v.test('aA')&&!/[[a]][\p{ASCII}]/v.test('aK')",
    );
    check(
        r"/[\P{Ll}&&A][s]/v.test('As')&&!/[\P{Ll}&&A][s]/iv.test('As')&&!/[a--a][c]/v.test('ac')&&!/[[a]][]/v.test('a')&&/[[a]][^]/v.test('a😀')&&/[[\]]][\[]/v.test('][')",
    );
    check(
        r"let r=/[[\p{Any}]][a]/dvy;r.lastIndex=1;let m=r.exec('😀a!');m.index===0&&m[0]==='😀a'&&m.indices[0][1]===3&&r.lastIndex===3&&/[[\u{1f600}]][\u{1f601}]/v.exec('x😀😁').index===1",
    );
    check(
        r"let r=/[[\uD800]][\uD800]/dv,m=r.exec('A\uD800\uD800!');m.index===1&&m[0]==='\uD800\uD800'&&m.indices[0][1]===3&&!/[[\uD800]][\uDC00]/v.test('\uD800\uDC00')",
    );
    check(
        r"let r=/^[[a]][b]$/dvmg,m=r.exec('x\r\nab\u2028cd');m.index===3&&m[0]==='ab'&&m.indices[0][1]===5&&r.lastIndex===5&&r.exec('x\r\nab\u2028cd')===null&&r.lastIndex===0&&!/^[[a]][b]$/v.test('ab\n')",
    );
    check(r"/^[[a]][\$]$/v.test('a$')&&!/^[[a]][\$]$/v.test('a$\n')");
    check(
        r"let r=/(?<whole>([a--b][c]))/dvg,a=[...'aac aac'.matchAll(r)];a.length===2&&a[0][0]==='ac'&&a[0][1]==='ac'&&a[0][2]==='ac'&&a[0].index===1&&a[0].groups.whole==='ac'&&a[0].indices.groups.whole===a[0].indices[1]&&a[0].indices[2][1]===3&&a[1].index===5&&a[1].indices[2][1]===7&&r.lastIndex===0",
    );
    check(
        r"'aac aac'.replace(/(?<whole>([a--b][c]))/vg,'<$1:$<whole>>')==='a<ac:ac> a<ac:ac>'&&'aac'.split(/([a--b][c])/v).join('|')==='a|ac|'&&'aac'.match(/[a--b][c]/v)[0]==='ac'",
    );
    check(
        r"let seen='',out='aac aac'.replace(/(?<whole>([a--b][c]))/vg,(m,c1,c2,p,text,g)=>{seen+=p+':'+g.whole+';';return 'X'});out==='aX aX'&&seen==='1:ac;5:ac;'",
    );
    check(r"let r=/[a--b][c]/vg;r.lastIndex=4;'aac'.search(r)===1&&r.lastIndex===4");
    check(
        r"let trace='',r=/[[\p{Any}]][a]/vy;r.lastIndex={valueOf(){trace+='i';return 1}};let m=r.exec({toString(){trace+='s';return '😀a!'}});trace==='si'&&m.index===0&&r.lastIndex===3",
    );
    check(
        r"let seen=0,r=/[a--b][c]/v;r.lastIndex={valueOf(){seen++;return 1}};let m=r.exec('aac');m.index===1&&seen===1&&typeof r.lastIndex==='object'",
    );
    check(
        r"let r=/[[K]][s]/iv,c=new RegExp(r,'dvy');c.source===r.source&&r.source==='[[K]][s]'&&c.unicodeSets&&!c.ignoreCase&&c.exec('Kſ')===null&&c.lastIndex===0&&r.exec('Kſ')[0]==='Kſ'",
    );
    check(
        r"let a=/[a--b][c]/vy,b=/[a--b][c]/vy;a.lastIndex=1;Object.defineProperty(a,'lastIndex',{writable:false});Object.defineProperty(b,'lastIndex',{writable:false});let x=false,y=false;try{a.exec('aac')}catch(e){x=e instanceof TypeError}try{b.exec('aac')}catch(e){y=e instanceof TypeError}x&&y&&a.lastIndex===1&&b.lastIndex===0",
    );
    check(r"/[a][b]/v.test('ab')&&!/[a][b]/v.test('ac')");
    for source in [
        r"/c[a--b][c]/v.test('cac')",
        r"/[a--b][c][d]/v.test('acd')",
        r"/[a--b]^[c]/v.test('ac')",
        r"/([a--b])[c]/v.test('ac')",
        r"/[a--b]([c])/v.test('ac')",
        r"/[a--b]+[c]/v.test('ac')",
        r"/[a--b][c]?/v.test('ac')",
        r"/[a--a][\q{c}]/v.test('ac')",
        r"/[a--a][\p{RGI_Emoji}]/v.test('ac')",
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
    realm
        .eval(r"var r=/[a--b][c]/vg,sticky=/[[a]][a]/vy,marker=0")
        .unwrap();
    let setup = format!("var text='{}';r.lastIndex=1", "a".repeat(20_000));
    realm.eval(&setup).unwrap();
    assert_eq!(
        realm.eval("sticky.test(text)&&sticky.lastIndex===2"),
        Ok(Value::Boolean(true))
    );
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
fn checked_then_flat_v_set_pair_results() {
    let mut rows = String::new();
    let mut row_count = 0;
    // These full field objects and original spans are manual expectations.
    for (pattern, input, flags, start, expected) in [
        (
            r"(?<whole>([a--b][c]))",
            JsString::from("aac"),
            "dvg",
            0,
            r#"{"matches":["ac","ac","ac"],"index":1,"input":"aac","groups":{"whole":"ac"},"indices":[[1,3],[1,3],[1,3]],"indexGroups":{"whole":[1,3]},"lastIndex":3,"source":"(?<whole>([a--b][c]))"}"#,
        ),
        (
            r"(?<whole>([a--b][c]))",
            JsString::from("aac"),
            "dvy",
            0,
            r#"{"match":null,"lastIndex":0}"#,
        ),
        (
            r"(?<whole>([a--b][c]))",
            JsString::from("aac"),
            "dvy",
            1,
            r#"{"matches":["ac","ac","ac"],"index":1,"input":"aac","groups":{"whole":"ac"},"indices":[[1,3],[1,3],[1,3]],"indexGroups":{"whole":[1,3]},"lastIndex":3,"source":"(?<whole>([a--b][c]))"}"#,
        ),
        (
            r"(?<whole>([[K]][s]))",
            JsString::from("Kſ!"),
            "divg",
            0,
            r#"{"matches":["Kſ","Kſ","Kſ"],"index":0,"input":"Kſ!","groups":{"whole":"Kſ"},"indices":[[0,2],[0,2],[0,2]],"indexGroups":{"whole":[0,2]},"lastIndex":2,"source":"(?<whole>([[K]][s]))"}"#,
        ),
        (
            r"(?<whole>([\P{Ll}&&A][s]))",
            JsString::from("As"),
            "divg",
            0,
            r#"{"match":null,"lastIndex":0}"#,
        ),
        (
            r"(?<whole>([[\p{Any}]][a]))",
            JsString::from("😀a!"),
            "dvy",
            1,
            r#"{"matches":["😀a","😀a","😀a"],"index":0,"input":"😀a!","groups":{"whole":"😀a"},"indices":[[0,3],[0,3],[0,3]],"indexGroups":{"whole":[0,3]},"lastIndex":3,"source":"(?<whole>([[\\p{Any}]][a]))"}"#,
        ),
        (
            r"(?<whole>([[\u{1f600}]][\u{1f601}]))",
            JsString::from("x😀😁!"),
            "dvg",
            0,
            r#"{"matches":["😀😁","😀😁","😀😁"],"index":1,"input":"x😀😁!","groups":{"whole":"😀😁"},"indices":[[1,5],[1,5],[1,5]],"indexGroups":{"whole":[1,5]},"lastIndex":5,"source":"(?<whole>([[\\u{1f600}]][\\u{1f601}]))"}"#,
        ),
        (
            r"(?<whole>([[\uD800]][\uD800]))",
            JsString::from_code_units(vec![0x41, 0xd800, 0xd800, 0x21]),
            "dvg",
            0,
            r#"{"matches":["\ud800\ud800","\ud800\ud800","\ud800\ud800"],"index":1,"input":"A\ud800\ud800!","groups":{"whole":"\ud800\ud800"},"indices":[[1,3],[1,3],[1,3]],"indexGroups":{"whole":[1,3]},"lastIndex":3,"source":"(?<whole>([[\\uD800]][\\uD800]))"}"#,
        ),
        (
            r"(?<whole>([[a]][^K]))",
            JsString::from("aKaſ!"),
            "divg",
            0,
            r#"{"matches":["aſ","aſ","aſ"],"index":2,"input":"aKaſ!","groups":{"whole":"aſ"},"indices":[[2,4],[2,4],[2,4]],"indexGroups":{"whole":[2,4]},"lastIndex":4,"source":"(?<whole>([[a]][^K]))"}"#,
        ),
        (
            r"^(?<whole>([[a]][b]))$",
            JsString::from("x\r\nab\u{2028}cd"),
            "dvmg",
            0,
            r#"{"matches":["ab","ab","ab"],"index":3,"input":"x\r\nab cd","groups":{"whole":"ab"},"indices":[[3,5],[3,5],[3,5]],"indexGroups":{"whole":[3,5]},"lastIndex":5,"source":"^(?<whole>([[a]][b]))$"}"#,
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
            input.code_units()
        )
        .unwrap();
        row_count += 1;
    }
    assert_eq!(row_count, 10);
    insta::assert_snapshot!(rows);
}

#[test]
fn flat_then_checked_v_set_pair_state_context_and_consumers() {
    check(
        r"/[a][b--c]/v.test('ab')&&!/[a][b--c]/v.test('ac')&&/[a][c--b]/v.test('ac')&&!/[a][c--b]/v.test('ab')&&/[abc][[d]]/v.test('ad')&&/[a-c][[d]]/iv.test('aD')&&!/[a-c][[d]]/v.test('aD')",
    );
    check(r#"let marker=0;eval("marker=1; /[a][b--c]/v.test('ab')")&&marker===1"#);
    check(
        r"let g=/[a][c--b]/vg,y=/[a][c--b]/vy,a=g.exec('aac');a.index===1&&a[0]==='ac'&&g.lastIndex===3&&g.exec('aac')===null&&g.lastIndex===0&&y.exec('aac')===null&&y.lastIndex===0&&(y.lastIndex=1,y.exec('aac')[0]==='ac')&&y.lastIndex===3",
    );
    check(
        r"/[K][[s]]/iv.test('Kſ')&&!/[K][[s]]/v.test('Kſ')&&!/[a][[^K]]/iv.test('aK')&&/[a][[^K]]/iv.exec('aKaſ')[0]==='aſ'&&/[\p{ASCII}][[a]]/v.test('Aa')&&!/[\p{ASCII}][[a]]/v.test('Ka')",
    );
    check(
        r"/[A][\P{Ll}&&A]/v.test('AA')&&!/[A][\P{Ll}&&A]/iv.test('AA')&&!/[][c--b]/v.test('ac')&&!/[a][b--b]/v.test('a')&&/[a][[^]]/v.test('a😀')&&/[\]][[\[]]/v.test('][')",
    );
    check(
        r"let r=/[\p{Any}][[a]]/dvy;r.lastIndex=1;let m=r.exec('😀a!');m.index===0&&m[0]==='😀a'&&m.indices[0][1]===3&&r.lastIndex===3&&/[\u{1f600}][[\u{1f601}]]/v.exec('x😀😁').index===1",
    );
    check(
        r"let r=/[\uD800][[\uD800]]/dv,m=r.exec('A\uD800\uD800!');m.index===1&&m[0]==='\uD800\uD800'&&m.indices[0][1]===3&&!/[\uD800][[\uDC00]]/v.test('\uD800\uDC00')",
    );
    check(
        r"let r=/^[a][[b]]$/dvmg,m=r.exec('x\r\nab\u2028cd');m.index===3&&m[0]==='ab'&&m.indices[0][1]===5&&r.lastIndex===5&&r.exec('x\r\nab\u2028cd')===null&&r.lastIndex===0&&!/^[a][[b]]$/v.test('ab\n')",
    );
    check(r"/^[a][[\$]]$/v.test('a$')&&!/^[a][[\$]]$/v.test('a$\n')");
    check(
        r"let r=/(?<whole>([a][c--b]))/dvg,a=[...'aac aac'.matchAll(r)];a.length===2&&a[0][0]==='ac'&&a[0][1]==='ac'&&a[0][2]==='ac'&&a[0].index===1&&a[0].groups.whole==='ac'&&a[0].indices.groups.whole===a[0].indices[1]&&a[0].indices[2][1]===3&&a[1].index===5&&a[1].indices[2][1]===7&&r.lastIndex===0",
    );
    check(
        r"'aac aac'.replace(/(?<whole>([a][c--b]))/vg,'<$1:$<whole>>')==='a<ac:ac> a<ac:ac>'&&'aac'.split(/([a][c--b])/v).join('|')==='a|ac|'&&'aac'.match(/[a][c--b]/v)[0]==='ac'",
    );
    check(
        r"let seen='',out='aac aac'.replace(/(?<whole>([a][c--b]))/vg,(m,c1,c2,p,text,g)=>{seen+=p+':'+g.whole+';';return 'X'});out==='aX aX'&&seen==='1:ac;5:ac;'",
    );
    check(r"let r=/[a][c--b]/vg;r.lastIndex=4;'aac'.search(r)===1&&r.lastIndex===4");
    check(
        r"let trace='',r=/[\p{Any}][[a]]/vy;r.lastIndex={valueOf(){trace+='i';return 1}};let m=r.exec({toString(){trace+='s';return '😀a!'}});trace==='si'&&m.index===0&&r.lastIndex===3",
    );
    check(
        r"let seen=0,r=/[a][c--b]/v;r.lastIndex={valueOf(){seen++;return 1}};let m=r.exec('aac');m.index===1&&seen===1&&typeof r.lastIndex==='object'",
    );
    check(
        r"let r=/[K][[s]]/iv,c=new RegExp(r,'dvy');c.source===r.source&&r.source==='[K][[s]]'&&c.unicodeSets&&!c.ignoreCase&&c.exec('Kſ')===null&&c.lastIndex===0&&r.exec('Kſ')[0]==='Kſ'",
    );
    check(
        r"let a=/[a][c--b]/vy,b=/[a][c--b]/vy;a.lastIndex=1;Object.defineProperty(a,'lastIndex',{writable:false});Object.defineProperty(b,'lastIndex',{writable:false});let x=false,y=false;try{a.exec('aac')}catch(e){x=e instanceof TypeError}try{b.exec('aac')}catch(e){y=e instanceof TypeError}x&&y&&a.lastIndex===1&&b.lastIndex===0",
    );
    check(r"/\b[a][c--b]/v.test('ac')&&!/\b[a][c--b]/v.test('ab')&&!/\b[a][c--b]/v.test('xac')");
    check(r"/[a][c--b]\b/v.test('ac')&&!/[a][c--b]\b/v.test('ab')&&!/[a][c--b]\b/v.test('acx')");
    check(r"/[a][b]/v.test('ab')&&!/[a][b]/v.test('ac')");
    for source in [
        r"/c[a][c--b]/v.test('cac')",
        r"/[a][c--b][d]/v.test('acd')",
        r"/[a]^[b--c]/v.test('ac')",
        r"/([a])[b--c]/v.test('ac')",
        r"/[a]([b--c])/v.test('ac')",
        r"/[a]+[b--c]/v.test('ac')",
        r"/[a][c--b]?/v.test('ac')",
        r"/[][[\q{c}]]/v.test('ac')",
        r"/[][[\p{RGI_Emoji}]]/v.test('ac')",
        r"/[\q{a}][[b]]/v.test('ab')",
        r"/[\p{RGI_Emoji}][[b]]/v.test('ab')",
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
    realm
        .eval(r"var r=/[a][c--b]/vg,sticky=/[a][[a]]/vy,marker=0")
        .unwrap();
    let setup = format!("var text='{}';r.lastIndex=1", "a".repeat(20_000));
    realm.eval(&setup).unwrap();
    assert_eq!(
        realm.eval("sticky.test(text)&&sticky.lastIndex===2"),
        Ok(Value::Boolean(true))
    );
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
fn flat_then_checked_v_set_pair_results() {
    let mut rows = String::new();
    let mut row_count = 0;
    // Full field objects and original spans below are independent manual expectations.
    for (pattern, input, flags, start, expected) in [
        (
            r"(?<whole>([a][c--b]))",
            JsString::from("aac"),
            "dvg",
            0,
            r#"{"matches":["ac","ac","ac"],"index":1,"input":"aac","groups":{"whole":"ac"},"indices":[[1,3],[1,3],[1,3]],"indexGroups":{"whole":[1,3]},"lastIndex":3,"source":"(?<whole>([a][c--b]))"}"#,
        ),
        (
            r"(?<whole>([a][c--b]))",
            JsString::from("aac"),
            "dvy",
            0,
            r#"{"match":null,"lastIndex":0}"#,
        ),
        (
            r"(?<whole>([a][c--b]))",
            JsString::from("aac"),
            "dvy",
            1,
            r#"{"matches":["ac","ac","ac"],"index":1,"input":"aac","groups":{"whole":"ac"},"indices":[[1,3],[1,3],[1,3]],"indexGroups":{"whole":[1,3]},"lastIndex":3,"source":"(?<whole>([a][c--b]))"}"#,
        ),
        (
            r"(?<whole>([K][[s]]))",
            JsString::from("Kſ!"),
            "divg",
            0,
            r#"{"matches":["Kſ","Kſ","Kſ"],"index":0,"input":"Kſ!","groups":{"whole":"Kſ"},"indices":[[0,2],[0,2],[0,2]],"indexGroups":{"whole":[0,2]},"lastIndex":2,"source":"(?<whole>([K][[s]]))"}"#,
        ),
        (
            r"(?<whole>([A][\P{Ll}&&A]))",
            JsString::from("AA"),
            "divg",
            0,
            r#"{"match":null,"lastIndex":0}"#,
        ),
        (
            r"(?<whole>([\p{Any}][[a]]))",
            JsString::from("😀a!"),
            "dvy",
            1,
            r#"{"matches":["😀a","😀a","😀a"],"index":0,"input":"😀a!","groups":{"whole":"😀a"},"indices":[[0,3],[0,3],[0,3]],"indexGroups":{"whole":[0,3]},"lastIndex":3,"source":"(?<whole>([\\p{Any}][[a]]))"}"#,
        ),
        (
            r"(?<whole>([\u{1f600}][[\u{1f601}]]))",
            JsString::from("x😀😁!"),
            "dvg",
            0,
            r#"{"matches":["😀😁","😀😁","😀😁"],"index":1,"input":"x😀😁!","groups":{"whole":"😀😁"},"indices":[[1,5],[1,5],[1,5]],"indexGroups":{"whole":[1,5]},"lastIndex":5,"source":"(?<whole>([\\u{1f600}][[\\u{1f601}]]))"}"#,
        ),
        (
            r"(?<whole>([\uD800][[\uD800]]))",
            JsString::from_code_units(vec![0x41, 0xd800, 0xd800, 0x21]),
            "dvg",
            0,
            r#"{"matches":["\ud800\ud800","\ud800\ud800","\ud800\ud800"],"index":1,"input":"A\ud800\ud800!","groups":{"whole":"\ud800\ud800"},"indices":[[1,3],[1,3],[1,3]],"indexGroups":{"whole":[1,3]},"lastIndex":3,"source":"(?<whole>([\\uD800][[\\uD800]]))"}"#,
        ),
        (
            r"(?<whole>([a][[^K]]))",
            JsString::from("aKaſ!"),
            "divg",
            0,
            r#"{"matches":["aſ","aſ","aſ"],"index":2,"input":"aKaſ!","groups":{"whole":"aſ"},"indices":[[2,4],[2,4],[2,4]],"indexGroups":{"whole":[2,4]},"lastIndex":4,"source":"(?<whole>([a][[^K]]))"}"#,
        ),
        (
            r"^(?<whole>([a][[b]]))$",
            JsString::from("x\r\nab\u{2028}cd"),
            "dvmg",
            0,
            r#"{"matches":["ab","ab","ab"],"index":3,"input":"x\r\nab cd","groups":{"whole":"ab"},"indices":[[3,5],[3,5],[3,5]],"indexGroups":{"whole":[3,5]},"lastIndex":5,"source":"^(?<whole>([a][[b]]))$"}"#,
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
            input.code_units()
        )
        .unwrap();
        row_count += 1;
    }
    assert_eq!(row_count, 10);
    insta::assert_snapshot!(rows);
}

#[test]
fn checked_then_flat_v_set_pair_word_state_context_and_consumers() {
    check(
        r"/\b[a--b][c]/v.test('ac')&&!/\b[a--b][c]/v.test('xac')&&/[a--b][c]\b/v.test('ac')&&!/[a--b][c]\b/v.test('acx')",
    );
    check(r#"let marker=0;eval("marker=1; /\\b[a--b][c]/v.test('ac')")&&marker===1"#);
    check(
        r"/\b[[a]][b]/v.test('ab')&&!/\b[[a]][b]/v.test('xab')&&/[[a]][b]\b/v.test('ab')&&!/[[a]][b]\b/v.test('abx')",
    );
    check(
        r"let g=/\B[[a]][a]\b/vg,y=/\B[[a]][a]\b/vy,a=g.exec('aaa!');a.index===1&&a[0]==='aa'&&g.lastIndex===3&&g.exec('aaa!')===null&&g.lastIndex===0&&y.exec('aaa!')===null&&y.lastIndex===0&&(y.lastIndex=1,y.exec('aaa!')[0]==='aa')&&y.lastIndex===3",
    );
    check(
        r"/\B[[\p{Any}]][a]\b/v.test('Ka!')&&!/\B[[\p{Any}]][a]\b/iv.test('Ka!')&&!/\b[[\p{Any}]][a]\b/v.test('Ka!')&&/\b[[\p{Any}]][a]\b/iv.test('Ka!')&&/\b[[K]][s]\b/iv.test('Kſ!')&&!/\b[[K]][s]\b/v.test('Kſ!')",
    );
    check(
        r"!/\b[[a]][\p{Any}]\b/v.test('aK!')&&/\b[[a]][\p{Any}]\b/iv.test('aK!')&&!/\b[[a]][\p{Any}]\b/v.test('aſ!')&&/\b[[a]][\p{Any}]\b/iv.test('aſ!')&&/\B[[\p{Any}]][a]\b/iv.test('αa!')&&/\B[[!]][\?]\B/v.test('!?')",
    );
    check(
        r"!/\b[a--a][b]/v.test('ab')&&!/\b[[a]][]/v.test('ab')&&/\b[\P{Ll}&&A][s]\b/v.test('As!')&&!/\b[\P{Ll}&&A][s]\b/iv.test('As!')",
    );
    check(
        r"let r=/\B[[\p{Any}]][a]\b/dvy;r.lastIndex=1;let m=r.exec('😀a!');m.index===0&&m[0]==='😀a'&&m.indices[0][1]===3&&r.lastIndex===3&&/\B[[\u{1f600}]][\u{1f601}]\B/v.exec('!😀😁!').index===1",
    );
    check(
        r"let r=/\b[[\uD800]][\uD800]\B/dv,m=r.exec('A\uD800\uD800!');m.index===1&&m[0]==='\uD800\uD800'&&m.indices[0][1]===3&&!/\B[[\uD800]][\uDC00]\B/v.test('\uD800\uDC00')",
    );
    check(
        r"let r=/^\b[[a]][b]\b$/dvmg,m=r.exec('x\r\nab\u2028cd');m.index===3&&m[0]==='ab'&&m.indices[0][1]===5&&r.lastIndex===5&&r.exec('x\r\nab\u2028cd')===null&&r.lastIndex===0&&!/^\b[[a]][b]\b$/v.test('ab\n')",
    );
    check(
        r"let r=/(?<whole>(\B[[a]][a]\b))/dvg,a=[...'aaa! aaa!'.matchAll(r)];a.length===2&&a[0][0]==='aa'&&a[0][1]==='aa'&&a[0][2]==='aa'&&a[0].index===1&&a[0].groups.whole==='aa'&&a[0].indices.groups.whole===a[0].indices[1]&&a[0].indices[2][1]===3&&a[1].index===6&&a[1].indices[2][1]===8&&r.lastIndex===0",
    );
    check(
        r"'aaa! aaa!'.replace(/(?<whole>(\B[[a]][a]\b))/vg,'<$1:$<whole>>')==='a<aa:aa>! a<aa:aa>!'&&'aaa!'.split(/(\B[[a]][a]\b)/v).join('|')==='a|aa|!'&&'aaa!'.match(/\B[[a]][a]\b/v)[0]==='aa'",
    );
    check(
        r"let seen='',out='aaa! aaa!'.replace(/(?<whole>(\B[[a]][a]\b))/vg,(m,c1,c2,p,text,g)=>{seen+=p+':'+g.whole+';';return 'X'});out==='aX! aX!'&&seen==='1:aa;6:aa;'",
    );
    check(r"let r=/\B[[a]][a]\b/vg;r.lastIndex=4;'aaa!'.search(r)===1&&r.lastIndex===4");
    check(
        r"let trace='',r=/\B[[\p{Any}]][a]\b/vy;r.lastIndex={valueOf(){trace+='i';return 1}};let m=r.exec({toString(){trace+='s';return '😀a!'}});trace==='si'&&m.index===0&&m[0]==='😀a'&&r.lastIndex===3",
    );
    check(
        r"let seen=0,r=/\B[[a]][a]\b/v;r.lastIndex={valueOf(){seen++;return 1}};let m=r.exec('aaa!');m.index===1&&seen===1&&typeof r.lastIndex==='object'",
    );
    check(
        r"let r=/\b[[K]][s]\b/iv,c=new RegExp(r,'dvy');c.source===r.source&&r.source==='\\b[[K]][s]\\b'&&c.unicodeSets&&!c.ignoreCase&&c.exec('Kſ!')===null&&c.lastIndex===0&&r.exec('Kſ!')[0]==='Kſ'",
    );
    check(
        r"let a=/\B[[a]][a]\b/vy,b=/\B[[a]][a]\b/vy;a.lastIndex=1;Object.defineProperty(a,'lastIndex',{writable:false});Object.defineProperty(b,'lastIndex',{writable:false});let x=false,y=false;try{a.exec('aaa!')}catch(e){x=e instanceof TypeError}try{b.exec('aaa!')}catch(e){y=e instanceof TypeError}x&&y&&a.lastIndex===1&&b.lastIndex===0",
    );
    check(r"/\b[a][b--c]/v.test('ab')&&!/\b[a][b--c]/v.test('ac')&&!/\b[a][b--c]/v.test('xab')");
    check(r"/\b[a][b]/v.test('ab')&&!/\b[a][b]/v.test('xab')&&!/\b[a][b]/v.test('ac')");
    for source in [
        r"/\b[[a]][b][[c]]/v.test('abc')",
        r"/[[a]]\b[b]/v.test('ab')",
        r"/\b^[[a]][b]/v.test('ab')",
        r"/[[a]][b]$\b/v.test('ab')",
        r"/\b\b[[a]][b]/v.test('ab')",
        r"/(\b[[a]])[b]/v.test('ab')",
        r"/\b[[a]]([b])/v.test('ab')",
        r"/\b[[a]]+[b]/v.test('ab')",
        r"/\b[[a]][b]?/v.test('ab')",
        r"/\b[a--a][\q{b}]/v.test('ab')",
        r"/\b[a--a][\p{RGI_Emoji}]/v.test('ab')",
        r"/\b[[\q{a}]][]/v.test('ab')",
        r"/\b[[\p{RGI_Emoji}]][]/v.test('ab')",
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
    realm
        .eval(r"var r=/\B[[a]][a]\b/vg,sticky=/\b[[a]][a]/vy,marker=0")
        .unwrap();
    let setup = format!("var text='{}';r.lastIndex=1", "a".repeat(20_000));
    realm.eval(&setup).unwrap();
    assert_eq!(
        realm.eval("sticky.test(text)&&sticky.lastIndex===2"),
        Ok(Value::Boolean(true))
    );
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
fn checked_then_flat_v_set_pair_word_results() {
    let mut rows = String::new();
    let mut row_count = 0;
    // Full field objects and original spans below are independent manual expectations.
    for (pattern, input, flags, start, expected) in [
        (
            r"(?<whole>(\B[[a]][a]\b))",
            JsString::from("aaa!"),
            "dvg",
            0,
            r#"{"matches":["aa","aa","aa"],"index":1,"input":"aaa!","groups":{"whole":"aa"},"indices":[[1,3],[1,3],[1,3]],"indexGroups":{"whole":[1,3]},"lastIndex":3,"source":"(?<whole>(\\B[[a]][a]\\b))"}"#,
        ),
        (
            r"(?<whole>(\B[[a]][a]\b))",
            JsString::from("aaa!"),
            "dvy",
            0,
            r#"{"match":null,"lastIndex":0}"#,
        ),
        (
            r"(?<whole>(\B[[a]][a]\b))",
            JsString::from("aaa!"),
            "dvy",
            1,
            r#"{"matches":["aa","aa","aa"],"index":1,"input":"aaa!","groups":{"whole":"aa"},"indices":[[1,3],[1,3],[1,3]],"indexGroups":{"whole":[1,3]},"lastIndex":3,"source":"(?<whole>(\\B[[a]][a]\\b))"}"#,
        ),
        (
            r"(?<whole>(\b[[K]][s]\b))",
            JsString::from("Kſ!"),
            "divg",
            0,
            r#"{"matches":["Kſ","Kſ","Kſ"],"index":0,"input":"Kſ!","groups":{"whole":"Kſ"},"indices":[[0,2],[0,2],[0,2]],"indexGroups":{"whole":[0,2]},"lastIndex":2,"source":"(?<whole>(\\b[[K]][s]\\b))"}"#,
        ),
        (
            r"(?<whole>(\B[[\p{Any}]][a]\b))",
            JsString::from("Ka!"),
            "divg",
            0,
            r#"{"match":null,"lastIndex":0}"#,
        ),
        (
            r"(?<whole>(\B[[\p{Any}]][a]\b))",
            JsString::from("😀a!"),
            "dvy",
            1,
            r#"{"matches":["😀a","😀a","😀a"],"index":0,"input":"😀a!","groups":{"whole":"😀a"},"indices":[[0,3],[0,3],[0,3]],"indexGroups":{"whole":[0,3]},"lastIndex":3,"source":"(?<whole>(\\B[[\\p{Any}]][a]\\b))"}"#,
        ),
        (
            r"(?<whole>(\B[[\u{1f600}]][\u{1f601}]\B))",
            JsString::from("!😀😁!"),
            "dvg",
            0,
            r#"{"matches":["😀😁","😀😁","😀😁"],"index":1,"input":"!😀😁!","groups":{"whole":"😀😁"},"indices":[[1,5],[1,5],[1,5]],"indexGroups":{"whole":[1,5]},"lastIndex":5,"source":"(?<whole>(\\B[[\\u{1f600}]][\\u{1f601}]\\B))"}"#,
        ),
        (
            r"(?<whole>(\b[[\uD800]][\uD800]\B))",
            JsString::from_code_units(vec![0x41, 0xd800, 0xd800, 0x21]),
            "dvg",
            0,
            r#"{"matches":["\ud800\ud800","\ud800\ud800","\ud800\ud800"],"index":1,"input":"A\ud800\ud800!","groups":{"whole":"\ud800\ud800"},"indices":[[1,3],[1,3],[1,3]],"indexGroups":{"whole":[1,3]},"lastIndex":3,"source":"(?<whole>(\\b[[\\uD800]][\\uD800]\\B))"}"#,
        ),
        (
            r"(?<whole>(\b[[a]][\p{Any}]\b))",
            JsString::from("aK!"),
            "divg",
            0,
            r#"{"matches":["aK","aK","aK"],"index":0,"input":"aK!","groups":{"whole":"aK"},"indices":[[0,2],[0,2],[0,2]],"indexGroups":{"whole":[0,2]},"lastIndex":2,"source":"(?<whole>(\\b[[a]][\\p{Any}]\\b))"}"#,
        ),
        (
            r"^(?<whole>(\b[[a]][b]\b))$",
            JsString::from("x\r\nab\u{2028}cd"),
            "dvmg",
            0,
            r#"{"matches":["ab","ab","ab"],"index":3,"input":"x\r\nab cd","groups":{"whole":"ab"},"indices":[[3,5],[3,5],[3,5]],"indexGroups":{"whole":[3,5]},"lastIndex":5,"source":"^(?<whole>(\\b[[a]][b]\\b))$"}"#,
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
            input.code_units()
        )
        .unwrap();
        row_count += 1;
    }
    assert_eq!(row_count, 10);
    insta::assert_snapshot!(rows);
}

#[test]
fn flat_then_checked_v_set_pair_word_state_context_and_consumers() {
    check(
        r"/\b[a][b--c]/v.test('ab')&&!/\b[a][b--c]/v.test('ac')&&!/\b[a][b--c]/v.test('xab')&&/[a][c--b]\b/v.test('ac')&&!/[a][c--b]\b/v.test('acx')",
    );
    check(r#"let marker=0;eval("marker=1; /\\b[a][b--c]/v.test('ab')")&&marker===1"#);
    check(
        r"/\b[a][[b]]/v.test('ab')&&!/\b[a][[b]]/v.test('xab')&&/[a][[b]]\b/v.test('ab')&&!/[a][[b]]\b/v.test('abx')",
    );
    check(
        r"let g=/\B[a][[a]]\b/vg,y=/\B[a][[a]]\b/vy,a=g.exec('aaa!');a.index===1&&a[0]==='aa'&&g.lastIndex===3&&g.exec('aaa!')===null&&g.lastIndex===0&&y.exec('aaa!')===null&&y.lastIndex===0&&(y.lastIndex=1,y.exec('aaa!')[0]==='aa')&&y.lastIndex===3",
    );
    check(
        r"/\B[\p{Any}][[a]]\b/v.test('Ka!')&&!/\B[\p{Any}][[a]]\b/iv.test('Ka!')&&!/\b[\p{Any}][[a]]\b/v.test('Ka!')&&/\b[\p{Any}][[a]]\b/iv.test('Ka!')&&/\b[K][[s]]\b/iv.test('Kſ!')&&!/\b[K][[s]]\b/v.test('Kſ!')",
    );
    check(
        r"!/\b[a][[\p{Any}]]\b/v.test('aK!')&&/\b[a][[\p{Any}]]\b/iv.test('aK!')&&!/\b[a][[\p{Any}]]\b/v.test('aſ!')&&/\b[a][[\p{Any}]]\b/iv.test('aſ!')&&/\B[\p{Any}][[a]]\b/iv.test('αa!')&&/\B[!][[\?]]\B/v.test('!?')",
    );
    check(
        r"!/\b[][a--b]/v.test('ab')&&!/\b[a][[a--a]]/v.test('aa')&&/\b[A][\P{Ll}&&A]\b/v.test('AA!')&&!/\b[A][\P{Ll}&&A]\b/iv.test('AA!')",
    );
    check(
        r"let r=/\B[\p{Any}][[a]]\b/dvy;r.lastIndex=1;let m=r.exec('😀a!');m.index===0&&m[0]==='😀a'&&m.indices[0][1]===3&&r.lastIndex===3&&/\B[\u{1f600}][[\u{1f601}]]\B/v.exec('!😀😁!').index===1",
    );
    check(
        r"let r=/\b[\uD800][[\uD800]]\B/dv,m=r.exec('A\uD800\uD800!');m.index===1&&m[0]==='\uD800\uD800'&&m.indices[0][1]===3&&!/\B[\uD800][[\uDC00]]\B/v.test('\uD800\uDC00')",
    );
    check(
        r"let r=/^\b[a][[b]]\b$/dvmg,m=r.exec('x\r\nab\u2028cd');m.index===3&&m[0]==='ab'&&m.indices[0][1]===5&&r.lastIndex===5&&r.exec('x\r\nab\u2028cd')===null&&r.lastIndex===0&&!/^\b[a][[b]]\b$/v.test('ab\n')",
    );
    check(
        r"let r=/(?<whole>(\B[a][[a]]\b))/dvg,a=[...'aaa! aaa!'.matchAll(r)];a.length===2&&a[0][0]==='aa'&&a[0][1]==='aa'&&a[0][2]==='aa'&&a[0].index===1&&a[0].groups.whole==='aa'&&a[0].indices.groups.whole===a[0].indices[1]&&a[0].indices[2][1]===3&&a[1].index===6&&a[1].indices[2][1]===8&&r.lastIndex===0",
    );
    check(
        r"'aaa! aaa!'.replace(/(?<whole>(\B[a][[a]]\b))/vg,'<$1:$<whole>>')==='a<aa:aa>! a<aa:aa>!'&&'aaa!'.split(/(\B[a][[a]]\b)/v).join('|')==='a|aa|!'&&'aaa!'.match(/\B[a][[a]]\b/v)[0]==='aa'",
    );
    check(
        r"let seen='',out='aaa! aaa!'.replace(/(?<whole>(\B[a][[a]]\b))/vg,(m,c1,c2,p,text,g)=>{seen+=p+':'+g.whole+';';return 'X'});out==='aX! aX!'&&seen==='1:aa;6:aa;'",
    );
    check(r"let r=/\B[a][[a]]\b/vg;r.lastIndex=4;'aaa!'.search(r)===1&&r.lastIndex===4");
    check(
        r"let trace='',r=/\B[\p{Any}][[a]]\b/vy;r.lastIndex={valueOf(){trace+='i';return 1}};let m=r.exec({toString(){trace+='s';return '😀a!'}});trace==='si'&&m.index===0&&m[0]==='😀a'&&r.lastIndex===3",
    );
    check(
        r"let seen=0,r=/\B[a][[a]]\b/v;r.lastIndex={valueOf(){seen++;return 1}};let m=r.exec('aaa!');m.index===1&&seen===1&&typeof r.lastIndex==='object'",
    );
    check(
        r"let r=/\b[K][[s]]\b/iv,c=new RegExp(r,'dvy');c.source===r.source&&r.source==='\\b[K][[s]]\\b'&&c.unicodeSets&&!c.ignoreCase&&c.exec('Kſ!')===null&&c.lastIndex===0&&r.exec('Kſ!')[0]==='Kſ'",
    );
    check(
        r"let a=/\B[a][[a]]\b/vy,b=/\B[a][[a]]\b/vy;a.lastIndex=1;Object.defineProperty(a,'lastIndex',{writable:false});Object.defineProperty(b,'lastIndex',{writable:false});let x=false,y=false;try{a.exec('aaa!')}catch(e){x=e instanceof TypeError}try{b.exec('aaa!')}catch(e){y=e instanceof TypeError}x&&y&&a.lastIndex===1&&b.lastIndex===0",
    );
    check(
        r"let marker=0,caught=false;try{eval('marker=1; /\\b[a][b&&]/v;')}catch(e){caught=e instanceof SyntaxError}caught&&marker===0",
    );
    check(r"/\b[a][b]/v.test('ab')&&!/\b[a][b]/v.test('xab')&&!/\b[a][b]/v.test('ac')");
    for source in [
        r"/\b[a][[b]][c]/v.test('abc')",
        r"/[a]\b[[b]]/v.test('ab')",
        r"/\b^[a][[b]]/v.test('ab')",
        r"/[a][[b]]$\b/v.test('ab')",
        r"/\b\b[a][[b]]/v.test('ab')",
        r"/(\b[a])[[b]]/v.test('ab')",
        r"/\b[a]([[b]])/v.test('ab')",
        r"/\b[a]+[[b]]/v.test('ab')",
        r"/\b[a][[b]]?/v.test('ab')",
        r"/\b[][[\q{b}]]/v.test('ab')",
        r"/\b[][[\p{RGI_Emoji}]]/v.test('ab')",
        r"/\b[\q{a}][[a--a]]/v.test('ab')",
        r"/\b[\p{RGI_Emoji}][[a--a]]/v.test('ab')",
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
    realm
        .eval(r"var r=/\B[a][[a]]\b/vg,sticky=/\b[a][[a]]/vy,marker=0")
        .unwrap();
    let setup = format!("var text='{}';r.lastIndex=1", "a".repeat(20_000));
    realm.eval(&setup).unwrap();
    assert_eq!(
        realm.eval("sticky.test(text)&&sticky.lastIndex===2"),
        Ok(Value::Boolean(true))
    );
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
fn flat_then_checked_v_set_pair_word_results() {
    let mut rows = String::new();
    let mut row_count = 0;
    // Full field objects and original spans below are independent manual expectations.
    for (pattern, input, flags, start, expected) in [
        (
            r"(?<whole>(\B[a][[a]]\b))",
            JsString::from("aaa!"),
            "dvg",
            0,
            r#"{"matches":["aa","aa","aa"],"index":1,"input":"aaa!","groups":{"whole":"aa"},"indices":[[1,3],[1,3],[1,3]],"indexGroups":{"whole":[1,3]},"lastIndex":3,"source":"(?<whole>(\\B[a][[a]]\\b))"}"#,
        ),
        (
            r"(?<whole>(\B[a][[a]]\b))",
            JsString::from("aaa!"),
            "dvy",
            0,
            r#"{"match":null,"lastIndex":0}"#,
        ),
        (
            r"(?<whole>(\B[a][[a]]\b))",
            JsString::from("aaa!"),
            "dvy",
            1,
            r#"{"matches":["aa","aa","aa"],"index":1,"input":"aaa!","groups":{"whole":"aa"},"indices":[[1,3],[1,3],[1,3]],"indexGroups":{"whole":[1,3]},"lastIndex":3,"source":"(?<whole>(\\B[a][[a]]\\b))"}"#,
        ),
        (
            r"(?<whole>(\b[K][[s]]\b))",
            JsString::from("Kſ!"),
            "divg",
            0,
            r#"{"matches":["Kſ","Kſ","Kſ"],"index":0,"input":"Kſ!","groups":{"whole":"Kſ"},"indices":[[0,2],[0,2],[0,2]],"indexGroups":{"whole":[0,2]},"lastIndex":2,"source":"(?<whole>(\\b[K][[s]]\\b))"}"#,
        ),
        (
            r"(?<whole>(\B[\p{Any}][[a]]\b))",
            JsString::from("Ka!"),
            "divg",
            0,
            r#"{"match":null,"lastIndex":0}"#,
        ),
        (
            r"(?<whole>(\B[\p{Any}][[a]]\b))",
            JsString::from("😀a!"),
            "dvy",
            1,
            r#"{"matches":["😀a","😀a","😀a"],"index":0,"input":"😀a!","groups":{"whole":"😀a"},"indices":[[0,3],[0,3],[0,3]],"indexGroups":{"whole":[0,3]},"lastIndex":3,"source":"(?<whole>(\\B[\\p{Any}][[a]]\\b))"}"#,
        ),
        (
            r"(?<whole>(\B[\u{1f600}][[\u{1f601}]]\B))",
            JsString::from("!😀😁!"),
            "dvg",
            0,
            r#"{"matches":["😀😁","😀😁","😀😁"],"index":1,"input":"!😀😁!","groups":{"whole":"😀😁"},"indices":[[1,5],[1,5],[1,5]],"indexGroups":{"whole":[1,5]},"lastIndex":5,"source":"(?<whole>(\\B[\\u{1f600}][[\\u{1f601}]]\\B))"}"#,
        ),
        (
            r"(?<whole>(\b[\uD800][[\uD800]]\B))",
            JsString::from_code_units(vec![0x41, 0xd800, 0xd800, 0x21]),
            "dvg",
            0,
            r#"{"matches":["\ud800\ud800","\ud800\ud800","\ud800\ud800"],"index":1,"input":"A\ud800\ud800!","groups":{"whole":"\ud800\ud800"},"indices":[[1,3],[1,3],[1,3]],"indexGroups":{"whole":[1,3]},"lastIndex":3,"source":"(?<whole>(\\b[\\uD800][[\\uD800]]\\B))"}"#,
        ),
        (
            r"(?<whole>(\b[a][[\p{Any}]]\b))",
            JsString::from("aK!"),
            "divg",
            0,
            r#"{"matches":["aK","aK","aK"],"index":0,"input":"aK!","groups":{"whole":"aK"},"indices":[[0,2],[0,2],[0,2]],"indexGroups":{"whole":[0,2]},"lastIndex":2,"source":"(?<whole>(\\b[a][[\\p{Any}]]\\b))"}"#,
        ),
        (
            r"^(?<whole>(\b[a][[b]]\b))$",
            JsString::from("x\r\nab\u{2028}cd"),
            "dvmg",
            0,
            r#"{"matches":["ab","ab","ab"],"index":3,"input":"x\r\nab cd","groups":{"whole":"ab"},"indices":[[3,5],[3,5],[3,5]],"indexGroups":{"whole":[3,5]},"lastIndex":5,"source":"^(?<whole>(\\b[a][[b]]\\b))$"}"#,
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
            input.code_units()
        )
        .unwrap();
        row_count += 1;
    }
    assert_eq!(row_count, 10);
    insta::assert_snapshot!(rows);
}

#[test]
fn flat_v_set_pair_state_context_and_consumers() {
    check(
        r"/[a][c]/v.test('ac')&&!/[a][c]/v.test('ab')&&/[a][bcd]/v.test('ad')&&/[a][b-d]/iv.test('aD')&&!/[a][b-d]/v.test('aD')",
    );
    check(r#"let marker=0;eval("marker=1; /[a][c]/v.test('ac')")&&marker===1"#);
    check(
        r"let g=/[a][c]/vg,y=/[a][c]/vy,a=g.exec('aac');a.index===1&&a[0]==='ac'&&g.lastIndex===3&&g.exec('aac')===null&&g.lastIndex===0&&y.exec('aac')===null&&y.lastIndex===0&&(y.lastIndex=1,y.exec('aac')[0]==='ac')&&y.lastIndex===3",
    );
    check(
        r"/[K][s]/iv.test('Kſ')&&!/[K][s]/v.test('Kſ')&&!/[a][^K]/iv.test('aK')&&/[a][^K]/iv.exec('aKaſ')[0]==='aſ'&&/[a][\p{ASCII}]/v.test('aA')&&!/[a][\p{ASCII}]/v.test('aK')",
    );
    check(
        r"/[\P{Ll}][s]/v.test('As')&&!/[\P{Ll}][s]/iv.test('As')&&!/[][c]/v.test('ac')&&!/[a][]/v.test('a')&&/[a][^]/v.test('a😀')&&/[\]][\[]/v.test('][')",
    );
    check(
        r"let r=/[\p{Any}][a]/dvy;r.lastIndex=1;let m=r.exec('😀a!');m.index===0&&m[0]==='😀a'&&m.indices[0][1]===3&&r.lastIndex===3&&/[\u{1f600}][\u{1f601}]/v.exec('x😀😁').index===1",
    );
    check(
        r"let r=/[\uD800][\uD800]/dv,m=r.exec('A\uD800\uD800!');m.index===1&&m[0]==='\uD800\uD800'&&m.indices[0][1]===3&&!/[\uD800][\uDC00]/v.test('\uD800\uDC00')",
    );
    check(
        r"let r=/^[a][b]$/dvmg,m=r.exec('x\r\nab\u2028cd');m.index===3&&m[0]==='ab'&&m.indices[0][1]===5&&r.lastIndex===5&&r.exec('x\r\nab\u2028cd')===null&&r.lastIndex===0&&!/^[a][b]$/v.test('ab\n')",
    );
    check(r"/^[a][\$]$/v.test('a$')&&!/^[a][\$]$/v.test('a$\n')");
    check(
        r"let r=/(?<whole>([a][c]))/dvg,a=[...'aac aac'.matchAll(r)];a.length===2&&a[0][0]==='ac'&&a[0][1]==='ac'&&a[0][2]==='ac'&&a[0].index===1&&a[0].groups.whole==='ac'&&a[0].indices.groups.whole===a[0].indices[1]&&a[0].indices[2][1]===3&&a[1].index===5&&a[1].indices[2][1]===7&&r.lastIndex===0",
    );
    check(
        r"'aac aac'.replace(/(?<whole>([a][c]))/vg,'<$1:$<whole>>')==='a<ac:ac> a<ac:ac>'&&'aac'.split(/([a][c])/v).join('|')==='a|ac|'&&'aac'.match(/[a][c]/v)[0]==='ac'",
    );
    check(
        r"let seen='',out='aac aac'.replace(/(?<whole>([a][c]))/vg,(m,c1,c2,p,text,g)=>{seen+=p+':'+g.whole+';';return 'X'});out==='aX aX'&&seen==='1:ac;5:ac;'",
    );
    check(r"let r=/[a][c]/vg;r.lastIndex=4;'aac'.search(r)===1&&r.lastIndex===4");
    check(
        r"let trace='',r=/[\p{Any}][a]/vy;r.lastIndex={valueOf(){trace+='i';return 1}};let m=r.exec({toString(){trace+='s';return '😀a!'}});trace==='si'&&m.index===0&&r.lastIndex===3",
    );
    check(
        r"let seen=0,r=/[a][c]/v;r.lastIndex={valueOf(){seen++;return 1}};let m=r.exec('aac');m.index===1&&seen===1&&typeof r.lastIndex==='object'",
    );
    check(
        r"let r=/[K][s]/iv,c=new RegExp(r,'dvy');c.source===r.source&&r.source==='[K][s]'&&c.unicodeSets&&!c.ignoreCase&&c.exec('Kſ')===null&&c.lastIndex===0&&r.exec('Kſ')[0]==='Kſ'",
    );
    check(
        r"let a=/[a][c]/vy,b=/[a][c]/vy;a.lastIndex=1;Object.defineProperty(a,'lastIndex',{writable:false});Object.defineProperty(b,'lastIndex',{writable:false});let x=false,y=false;try{a.exec('aac')}catch(e){x=e instanceof TypeError}try{b.exec('aac')}catch(e){y=e instanceof TypeError}x&&y&&a.lastIndex===1&&b.lastIndex===0",
    );
    check(
        r"let marker=0,caught=false;try{eval('marker=1; /[][b&&]/v;')}catch(e){caught=e instanceof SyntaxError}caught&&marker===0",
    );
    check(r"/\b[a][b]/v.test('ab')&&!/\b[a][b]/v.test('xab')&&!/\b[a][b]/v.test('ac')");
    for source in [
        r"/c[a][c]/v.test('cac')",
        r"/[a][c][d]/v.test('acd')",
        r"/[a]^[c]/v.test('ac')",
        r"/([a])[c]/v.test('ac')",
        r"/[a]([c])/v.test('ac')",
        r"/[a]+[c]/v.test('ac')",
        r"/[a][c]?/v.test('ac')",
        r"/[][\q{c}]/v.test('ac')",
        r"/[][\p{RGI_Emoji}]/v.test('ac')",
        r"/[\q{a}][]/v.test('ab')",
        r"/[\p{RGI_Emoji}][]/v.test('ab')",
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
    realm
        .eval(r"var r=/[a][c]/vg,sticky=/[a][a]/vy,marker=0")
        .unwrap();
    let setup = format!("var text='{}';r.lastIndex=1", "a".repeat(20_000));
    realm.eval(&setup).unwrap();
    assert_eq!(
        realm.eval("sticky.test(text)&&sticky.lastIndex===2"),
        Ok(Value::Boolean(true))
    );
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
fn flat_v_set_pair_results() {
    let mut rows = String::new();
    let mut row_count = 0;
    // Full objects and original spans below are independent manual expectations.
    for (pattern, input, flags, start, expected) in [
        (
            r"(?<whole>([a][c]))",
            JsString::from("aac"),
            "dvg",
            0,
            r#"{"matches":["ac","ac","ac"],"index":1,"input":"aac","groups":{"whole":"ac"},"indices":[[1,3],[1,3],[1,3]],"indexGroups":{"whole":[1,3]},"lastIndex":3,"source":"(?<whole>([a][c]))"}"#,
        ),
        (
            r"(?<whole>([a][c]))",
            JsString::from("aac"),
            "dvy",
            0,
            r#"{"match":null,"lastIndex":0}"#,
        ),
        (
            r"(?<whole>([a][c]))",
            JsString::from("aac"),
            "dvy",
            1,
            r#"{"matches":["ac","ac","ac"],"index":1,"input":"aac","groups":{"whole":"ac"},"indices":[[1,3],[1,3],[1,3]],"indexGroups":{"whole":[1,3]},"lastIndex":3,"source":"(?<whole>([a][c]))"}"#,
        ),
        (
            r"(?<whole>([K][s]))",
            JsString::from("Kſ!"),
            "divg",
            0,
            r#"{"matches":["Kſ","Kſ","Kſ"],"index":0,"input":"Kſ!","groups":{"whole":"Kſ"},"indices":[[0,2],[0,2],[0,2]],"indexGroups":{"whole":[0,2]},"lastIndex":2,"source":"(?<whole>([K][s]))"}"#,
        ),
        (
            r"(?<whole>([\P{Ll}][s]))",
            JsString::from("As"),
            "divg",
            0,
            r#"{"match":null,"lastIndex":0}"#,
        ),
        (
            r"(?<whole>([\p{Any}][a]))",
            JsString::from("😀a!"),
            "dvy",
            1,
            r#"{"matches":["😀a","😀a","😀a"],"index":0,"input":"😀a!","groups":{"whole":"😀a"},"indices":[[0,3],[0,3],[0,3]],"indexGroups":{"whole":[0,3]},"lastIndex":3,"source":"(?<whole>([\\p{Any}][a]))"}"#,
        ),
        (
            r"(?<whole>([\u{1f600}][\u{1f601}]))",
            JsString::from("x😀😁!"),
            "dvg",
            0,
            r#"{"matches":["😀😁","😀😁","😀😁"],"index":1,"input":"x😀😁!","groups":{"whole":"😀😁"},"indices":[[1,5],[1,5],[1,5]],"indexGroups":{"whole":[1,5]},"lastIndex":5,"source":"(?<whole>([\\u{1f600}][\\u{1f601}]))"}"#,
        ),
        (
            r"(?<whole>([\uD800][\uD800]))",
            JsString::from_code_units(vec![0x41, 0xd800, 0xd800, 0x21]),
            "dvg",
            0,
            r#"{"matches":["\ud800\ud800","\ud800\ud800","\ud800\ud800"],"index":1,"input":"A\ud800\ud800!","groups":{"whole":"\ud800\ud800"},"indices":[[1,3],[1,3],[1,3]],"indexGroups":{"whole":[1,3]},"lastIndex":3,"source":"(?<whole>([\\uD800][\\uD800]))"}"#,
        ),
        (
            r"(?<whole>([a][^K]))",
            JsString::from("aKaſ!"),
            "divg",
            0,
            r#"{"matches":["aſ","aſ","aſ"],"index":2,"input":"aKaſ!","groups":{"whole":"aſ"},"indices":[[2,4],[2,4],[2,4]],"indexGroups":{"whole":[2,4]},"lastIndex":4,"source":"(?<whole>([a][^K]))"}"#,
        ),
        (
            r"^(?<whole>([a][b]))$",
            JsString::from("x\r\nab\u{2028}cd"),
            "dvmg",
            0,
            r#"{"matches":["ab","ab","ab"],"index":3,"input":"x\r\nab cd","groups":{"whole":"ab"},"indices":[[3,5],[3,5],[3,5]],"indexGroups":{"whole":[3,5]},"lastIndex":5,"source":"^(?<whole>([a][b]))$"}"#,
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
            input.code_units()
        )
        .unwrap();
        row_count += 1;
    }
    assert_eq!(row_count, 10);
    insta::assert_snapshot!(rows);
}

#[test]
fn flat_v_set_pair_word_state_context_and_consumers() {
    check(
        r"/\b[a][b]/v.test('ab')&&!/\b[a][b]/v.test('ac')&&!/\b[a][b]/v.test('xab')&&/[a][c]\b/v.test('ac')&&!/[a][c]\b/v.test('acx')",
    );
    check(r#"let marker=0;eval("marker=1; /\\b[a][b]/v.test('ab')")&&marker===1"#);
    check(
        r"/\b[a][b]/v.test('ab')&&!/\b[a][b]/v.test('xab')&&/[a][b]\b/v.test('ab')&&!/[a][b]\b/v.test('abx')",
    );
    check(
        r"let g=/\B[a][a]\b/vg,y=/\B[a][a]\b/vy,a=g.exec('aaa!');a.index===1&&a[0]==='aa'&&g.lastIndex===3&&g.exec('aaa!')===null&&g.lastIndex===0&&y.exec('aaa!')===null&&y.lastIndex===0&&(y.lastIndex=1,y.exec('aaa!')[0]==='aa')&&y.lastIndex===3",
    );
    check(
        r"/\B[\p{Any}][a]\b/v.test('Ka!')&&!/\B[\p{Any}][a]\b/iv.test('Ka!')&&!/\b[\p{Any}][a]\b/v.test('Ka!')&&/\b[\p{Any}][a]\b/iv.test('Ka!')&&/\b[K][s]\b/iv.test('Kſ!')&&!/\b[K][s]\b/v.test('Kſ!')",
    );
    check(
        r"!/\b[a][\p{Any}]\b/v.test('aK!')&&/\b[a][\p{Any}]\b/iv.test('aK!')&&!/\b[a][\p{Any}]\b/v.test('aſ!')&&/\b[a][\p{Any}]\b/iv.test('aſ!')&&/\B[\p{Any}][a]\b/iv.test('αa!')&&/\B[!][\?]\B/v.test('!?')",
    );
    check(
        r"!/\b[][a]/v.test('ab')&&!/\b[a][]/v.test('aa')&&/\b[A][\P{Ll}]\b/v.test('AA!')&&!/\b[A][\P{Ll}]\b/iv.test('AA!')",
    );
    check(
        r"let r=/\B[\p{Any}][a]\b/dvy;r.lastIndex=1;let m=r.exec('😀a!');m.index===0&&m[0]==='😀a'&&m.indices[0][1]===3&&r.lastIndex===3&&/\B[\u{1f600}][\u{1f601}]\B/v.exec('!😀😁!').index===1",
    );
    check(
        r"let r=/\b[\uD800][\uD800]\B/dv,m=r.exec('A\uD800\uD800!');m.index===1&&m[0]==='\uD800\uD800'&&m.indices[0][1]===3&&!/\B[\uD800][\uDC00]\B/v.test('\uD800\uDC00')",
    );
    check(
        r"let r=/^\b[a][b]\b$/dvmg,m=r.exec('x\r\nab\u2028cd');m.index===3&&m[0]==='ab'&&m.indices[0][1]===5&&r.lastIndex===5&&r.exec('x\r\nab\u2028cd')===null&&r.lastIndex===0&&!/^\b[a][b]\b$/v.test('ab\n')",
    );
    check(
        r"let r=/(?<whole>(\B[a][a]\b))/dvg,a=[...'aaa! aaa!'.matchAll(r)];a.length===2&&a[0][0]==='aa'&&a[0][1]==='aa'&&a[0][2]==='aa'&&a[0].index===1&&a[0].groups.whole==='aa'&&a[0].indices.groups.whole===a[0].indices[1]&&a[0].indices[2][1]===3&&a[1].index===6&&a[1].indices[2][1]===8&&r.lastIndex===0",
    );
    check(
        r"'aaa! aaa!'.replace(/(?<whole>(\B[a][a]\b))/vg,'<$1:$<whole>>')==='a<aa:aa>! a<aa:aa>!'&&'aaa!'.split(/(\B[a][a]\b)/v).join('|')==='a|aa|!'&&'aaa!'.match(/\B[a][a]\b/v)[0]==='aa'",
    );
    check(
        r"let seen='',out='aaa! aaa!'.replace(/(?<whole>(\B[a][a]\b))/vg,(m,c1,c2,p,text,g)=>{seen+=p+':'+g.whole+';';return 'X'});out==='aX! aX!'&&seen==='1:aa;6:aa;'",
    );
    check(r"let r=/\B[a][a]\b/vg;r.lastIndex=4;'aaa!'.search(r)===1&&r.lastIndex===4");
    check(
        r"let trace='',r=/\B[\p{Any}][a]\b/vy;r.lastIndex={valueOf(){trace+='i';return 1}};let m=r.exec({toString(){trace+='s';return '😀a!'}});trace==='si'&&m.index===0&&m[0]==='😀a'&&r.lastIndex===3",
    );
    check(
        r"let seen=0,r=/\B[a][a]\b/v;r.lastIndex={valueOf(){seen++;return 1}};let m=r.exec('aaa!');m.index===1&&seen===1&&typeof r.lastIndex==='object'",
    );
    check(
        r"let r=/\b[K][s]\b/iv,c=new RegExp(r,'dvy');c.source===r.source&&r.source==='\\b[K][s]\\b'&&c.unicodeSets&&!c.ignoreCase&&c.exec('Kſ!')===null&&c.lastIndex===0&&r.exec('Kſ!')[0]==='Kſ'",
    );
    check(
        r"let a=/\B[a][a]\b/vy,b=/\B[a][a]\b/vy;a.lastIndex=1;Object.defineProperty(a,'lastIndex',{writable:false});Object.defineProperty(b,'lastIndex',{writable:false});let x=false,y=false;try{a.exec('aaa!')}catch(e){x=e instanceof TypeError}try{b.exec('aaa!')}catch(e){y=e instanceof TypeError}x&&y&&a.lastIndex===1&&b.lastIndex===0",
    );
    check(
        r"let marker=0,caught=false;try{eval('marker=1; /\\b[][b&&]/v;')}catch(e){caught=e instanceof SyntaxError}caught&&marker===0",
    );
    for source in [
        r"/\b[a][b][c]/v.test('abc')",
        r"/[a]\b[b]/v.test('ab')",
        r"/\b^[a][b]/v.test('ab')",
        r"/[a][b]$\b/v.test('ab')",
        r"/\b\b[a][b]/v.test('ab')",
        r"/(\b[a])[b]/v.test('ab')",
        r"/\b[a]([b])/v.test('ab')",
        r"/\b[a]+[b]/v.test('ab')",
        r"/\b[a][b]?/v.test('ab')",
        r"/\b[][\q{b}]/v.test('ab')",
        r"/\b[][\p{RGI_Emoji}]/v.test('ab')",
        r"/\b[\q{a}][]/v.test('ab')",
        r"/\b[\p{RGI_Emoji}][]/v.test('ab')",
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
    realm
        .eval(r"var r=/\B[a][a]\b/vg,sticky=/\b[a][a]/vy,marker=0")
        .unwrap();
    let setup = format!("var text='{}';r.lastIndex=1", "a".repeat(20_000));
    realm.eval(&setup).unwrap();
    assert_eq!(
        realm.eval("sticky.test(text)&&sticky.lastIndex===2"),
        Ok(Value::Boolean(true))
    );
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
fn flat_v_set_pair_word_results() {
    let mut rows = String::new();
    let mut row_count = 0;
    // Full objects and original spans below are independent manual expectations.
    for (pattern, input, flags, start, expected) in [
        (
            r"(?<whole>(\B[a][a]\b))",
            JsString::from("aaa!"),
            "dvg",
            0,
            r#"{"matches":["aa","aa","aa"],"index":1,"input":"aaa!","groups":{"whole":"aa"},"indices":[[1,3],[1,3],[1,3]],"indexGroups":{"whole":[1,3]},"lastIndex":3,"source":"(?<whole>(\\B[a][a]\\b))"}"#,
        ),
        (
            r"(?<whole>(\B[a][a]\b))",
            JsString::from("aaa!"),
            "dvy",
            0,
            r#"{"match":null,"lastIndex":0}"#,
        ),
        (
            r"(?<whole>(\B[a][a]\b))",
            JsString::from("aaa!"),
            "dvy",
            1,
            r#"{"matches":["aa","aa","aa"],"index":1,"input":"aaa!","groups":{"whole":"aa"},"indices":[[1,3],[1,3],[1,3]],"indexGroups":{"whole":[1,3]},"lastIndex":3,"source":"(?<whole>(\\B[a][a]\\b))"}"#,
        ),
        (
            r"(?<whole>(\b[K][s]\b))",
            JsString::from("Kſ!"),
            "divg",
            0,
            r#"{"matches":["Kſ","Kſ","Kſ"],"index":0,"input":"Kſ!","groups":{"whole":"Kſ"},"indices":[[0,2],[0,2],[0,2]],"indexGroups":{"whole":[0,2]},"lastIndex":2,"source":"(?<whole>(\\b[K][s]\\b))"}"#,
        ),
        (
            r"(?<whole>(\B[\p{Any}][a]\b))",
            JsString::from("Ka!"),
            "divg",
            0,
            r#"{"match":null,"lastIndex":0}"#,
        ),
        (
            r"(?<whole>(\B[\p{Any}][a]\b))",
            JsString::from("😀a!"),
            "dvy",
            1,
            r#"{"matches":["😀a","😀a","😀a"],"index":0,"input":"😀a!","groups":{"whole":"😀a"},"indices":[[0,3],[0,3],[0,3]],"indexGroups":{"whole":[0,3]},"lastIndex":3,"source":"(?<whole>(\\B[\\p{Any}][a]\\b))"}"#,
        ),
        (
            r"(?<whole>(\B[\u{1f600}][\u{1f601}]\B))",
            JsString::from("!😀😁!"),
            "dvg",
            0,
            r#"{"matches":["😀😁","😀😁","😀😁"],"index":1,"input":"!😀😁!","groups":{"whole":"😀😁"},"indices":[[1,5],[1,5],[1,5]],"indexGroups":{"whole":[1,5]},"lastIndex":5,"source":"(?<whole>(\\B[\\u{1f600}][\\u{1f601}]\\B))"}"#,
        ),
        (
            r"(?<whole>(\b[\uD800][\uD800]\B))",
            JsString::from_code_units(vec![0x41, 0xd800, 0xd800, 0x21]),
            "dvg",
            0,
            r#"{"matches":["\ud800\ud800","\ud800\ud800","\ud800\ud800"],"index":1,"input":"A\ud800\ud800!","groups":{"whole":"\ud800\ud800"},"indices":[[1,3],[1,3],[1,3]],"indexGroups":{"whole":[1,3]},"lastIndex":3,"source":"(?<whole>(\\b[\\uD800][\\uD800]\\B))"}"#,
        ),
        (
            r"(?<whole>(\b[a][\p{Any}]\b))",
            JsString::from("aK!"),
            "divg",
            0,
            r#"{"matches":["aK","aK","aK"],"index":0,"input":"aK!","groups":{"whole":"aK"},"indices":[[0,2],[0,2],[0,2]],"indexGroups":{"whole":[0,2]},"lastIndex":2,"source":"(?<whole>(\\b[a][\\p{Any}]\\b))"}"#,
        ),
        (
            r"^(?<whole>(\b[a][b]\b))$",
            JsString::from("x\r\nab\u{2028}cd"),
            "dvmg",
            0,
            r#"{"matches":["ab","ab","ab"],"index":3,"input":"x\r\nab cd","groups":{"whole":"ab"},"indices":[[3,5],[3,5],[3,5]],"indexGroups":{"whole":[3,5]},"lastIndex":5,"source":"^(?<whole>(\\b[a][b]\\b))$"}"#,
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
            input.code_units()
        )
        .unwrap();
        row_count += 1;
    }
    assert_eq!(row_count, 10);
    insta::assert_snapshot!(rows);
}

#[test]
fn literal_checked_v_set_pair_state_context_and_consumers() {
    check(
        r"/c[a--b][[c]]/v.test('cac')&&!/c[a--b][[c]]/v.test('cbc')&&!/c[a--b][[c]]/v.test('ca')&&/a[[a]][[c]]/v.exec('aaac').index===1&&!/c[a--a][[b]]/v.test('cab')&&!/c[[a]][b--b]/v.test('cab')",
    );
    check(r#"let marker=0;eval("marker=1; /c[a--b][[c]]/v.test('cac')")&&marker===1"#);
    check(
        r"let g=/a[[a]][[c]]/vg,y=/a[[a]][[c]]/vy,a=g.exec('aaac');a.index===1&&a[0]==='aac'&&g.lastIndex===4&&g.exec('aaac')===null&&g.lastIndex===0&&y.exec('aaac')===null&&y.lastIndex===0&&(y.lastIndex=1,y.exec('aaac')[0]==='aac')&&y.lastIndex===4",
    );
    check(
        r"/k[[s]][[A]]/iv.test('Kſa')&&!/k[[s]][[A]]/v.test('Kſa')&&!/k[[s]][[A]]/v.test('KSA')&&/k[[s]][[A]]/v.test('ksA')&&/a[[a]][[^K]]/iv.exec('aaKaaſ')[0]==='aaſ'&&!/a[[a]][[^K]]/iv.test('aaK')",
    );
    check(
        r"/x[\P{Ll}&&A][[s]]/v.test('xAs')&&!/x[\P{Ll}&&A][[s]]/iv.test('xAs')&&/x[[\p{ASCII}]][[a]]/v.test('xAa')&&!/x[[\p{ASCII}]][[a]]/v.test('xKa')",
    );
    check(
        r"let r=/\u{1f600}[[\p{Any}]][[a]]/dvy;r.lastIndex=1;let m=r.exec('😀😁a!');m.index===0&&m[0]==='😀😁a'&&m.indices[0][1]===5&&r.lastIndex===5&&/😀[[😁]][[😂]]/v.test('😀😁😂')&&/\uD83D\uDE00[[\u{1f601}]][[\u{1f602}]]/v.test('😀😁😂')",
    );
    check(
        r"let r=/\u{1f600}[[\u{1f601}]][[\u{1f602}]]/dvy;r.lastIndex=1;let m=r.exec('😀😁😂!');m.index===0&&m[0]==='😀😁😂'&&m.indices[0][1]===6&&r.lastIndex===6",
    );
    check(
        r"let r=/\uD800[[\uD800]][[\uD800]]/dv,m=r.exec('A\uD800\uD800\uD800!');m.index===1&&m[0]==='\uD800\uD800\uD800'&&m.indices[0][1]===4&&!/\uD83D[[\uDE00]][[a]]/v.test('😀a')",
    );
    check(
        r"/\[[[a]][[b]]/v.test('[ab')&&/\/[[a]][[b]]/v.test('/ab')&&/\/[[a]][[b]]/v.source==='\\/[[a]][[b]]'&&/^\$[[a]][[b]]$/v.test('$ab')&&!/^\$[[a]][[b]]$/v.test('$ab\n')",
    );
    check(
        r"let r=/^c[[a]][[b]]$/dvmg,m=r.exec('x\r\ncab\u2028cd');m.index===3&&m[0]==='cab'&&m.indices[0][1]===6&&r.lastIndex===6&&r.exec('x\r\ncab\u2028cd')===null&&r.lastIndex===0&&!/^c[[a]][[b]]$/v.test('cab\n')",
    );
    check(
        r"let r=/(?<whole>(a[[a]][[c]]))/dvg,a=[...'aaac aaac'.matchAll(r)];a.length===2&&a[0][0]==='aac'&&a[0][1]==='aac'&&a[0][2]==='aac'&&a[0].index===1&&a[0].groups.whole==='aac'&&a[0].indices.groups.whole===a[0].indices[1]&&a[0].indices[2][1]===4&&a[1].index===6&&a[1].indices[2][1]===9&&r.lastIndex===0",
    );
    check(
        r"'aaac aaac'.replace(/(?<whole>(a[[a]][[c]]))/vg,'<$1:$<whole>>')==='a<aac:aac> a<aac:aac>'&&'aaac'.split(/(a[[a]][[c]])/v).join('|')==='a|aac|'&&'aaac'.match(/a[[a]][[c]]/v)[0]==='aac'",
    );
    check(
        r"let seen='',out='aaac aaac'.replace(/(?<whole>(a[[a]][[c]]))/vg,(m,c1,c2,p,text,g)=>{seen+=p+':'+g.whole+';';return 'X'});out==='aX aX'&&seen==='1:aac;6:aac;'",
    );
    check(r"let r=/a[[a]][[c]]/vg;r.lastIndex=4;'aaac'.search(r)===1&&r.lastIndex===4");
    check(
        r"let trace='',r=/\u{1f600}[[\p{Any}]][[a]]/vy;r.lastIndex={valueOf(){trace+='i';return 1}};let m=r.exec({toString(){trace+='s';return '😀😁a!'}});trace==='si'&&m.index===0&&m[0]==='😀😁a'&&r.lastIndex===5",
    );
    check(
        r"let seen=0,r=/a[[a]][[c]]/v;r.lastIndex={valueOf(){seen++;return 1}};let m=r.exec('aaac');m.index===1&&seen===1&&typeof r.lastIndex==='object'",
    );
    check(
        r"let r=/k[[s]][[A]]/iv,c=new RegExp(r,'dvy');c.source===r.source&&r.source==='k[[s]][[A]]'&&c.unicodeSets&&!c.ignoreCase&&c.exec('Kſa')===null&&c.lastIndex===0&&r.exec('Kſa')[0]==='Kſa'",
    );
    check(
        r"let a=/a[[a]][[c]]/vy,b=/a[[a]][[c]]/vy;a.lastIndex=1;Object.defineProperty(a,'lastIndex',{writable:false});Object.defineProperty(b,'lastIndex',{writable:false});let x=false,y=false;try{a.exec('aaac')}catch(e){x=e instanceof TypeError}try{b.exec('aaac')}catch(e){y=e instanceof TypeError}x&&y&&a.lastIndex===1&&b.lastIndex===0",
    );
    check(
        r"let marker=0,caught=false;try{eval('marker=1; /c[a--a][b&&]/v;')}catch(e){caught=e instanceof SyntaxError}caught&&marker===0",
    );
    check(r"/[[a]][[b]][[c]]/v.test('abc')");
    check(
        r"let r=/[[a]][[b]][[c]]/v,m=r.exec('abc');m.length===1&&m[0]==='abc'&&m.index===0&&m.input==='abc'&&m.groups===undefined&&m.indices===undefined&&r.source==='[[a]][[b]][[c]]'&&r.lastIndex===0&&!r.test('xbc')&&!r.test('axc')&&!r.test('abx')&&!r.test('ab')",
    );
    for source in [
        r"/ca[[a]][[b]]/v.test('caab')",
        r"/c[a][[b]]/v.test('cab')",
        r"/c[[a]][b]/v.test('cab')",
        r"/c[a][b]/v.test('cab')",
        r"/c[[a]][[b]][[c]]/v.test('cabc')",
        r"/c[[a]]x[[b]]/v.test('caxb')",
        r"/c[[a]]\b[[b]]/v.test('cab')",
        r"/\bc[[a]][[b]]/v.test('cab')",
        r"/c[[a]][[b]]\b/v.test('cab')",
        r"/(c)[[a]][[b]]/v.test('cab')",
        r"/c([[a]])[[b]]/v.test('cab')",
        r"/c[[a]]([[b]])/v.test('cab')",
        r"/c[[a]]+[[b]]/v.test('cab')",
        r"/c[[a]][[b]]?/v.test('cab')",
        r"/(?:c[[a]][[b]])+/v.test('cab')",
        r"/c[a--a][[\q{b}]]/v.test('cab')",
        r"/c[[\q{a}]][b--b]/v.test('cab')",
        r"/c[a--a][[\p{RGI_Emoji}]]/v.test('cab')",
        r"/c[[\p{RGI_Emoji}]][b--b]/v.test('cab')",
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
    realm
        .eval(r"var r=/a[[a]][[c]]/vg,sticky=/a[[a]][[a]]/vy,marker=0")
        .unwrap();
    // Every literal and first class succeeds while the second class fails.
    let setup = format!("var text='{}';r.lastIndex=1", "a".repeat(20_000));
    realm.eval(&setup).unwrap();
    assert_eq!(
        realm.eval("sticky.test(text)&&sticky.lastIndex===3"),
        Ok(Value::Boolean(true))
    );
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
fn literal_checked_v_set_pair_results() {
    let mut rows = String::new();
    let mut row_count = 0;
    // Full objects and original spans below are independent manual expectations.
    for (pattern, input, flags, start, expected) in [
        (
            r"(?<whole>(a[[a]][[c]]))",
            JsString::from("aaac"),
            "dvg",
            0,
            r#"{"matches":["aac","aac","aac"],"index":1,"input":"aaac","groups":{"whole":"aac"},"indices":[[1,4],[1,4],[1,4]],"indexGroups":{"whole":[1,4]},"lastIndex":4,"source":"(?<whole>(a[[a]][[c]]))"}"#,
        ),
        (
            r"(?<whole>(a[[a]][[c]]))",
            JsString::from("aaac"),
            "dvy",
            0,
            r#"{"match":null,"lastIndex":0}"#,
        ),
        (
            r"(?<whole>(a[[a]][[c]]))",
            JsString::from("aaac"),
            "dvy",
            1,
            r#"{"matches":["aac","aac","aac"],"index":1,"input":"aaac","groups":{"whole":"aac"},"indices":[[1,4],[1,4],[1,4]],"indexGroups":{"whole":[1,4]},"lastIndex":4,"source":"(?<whole>(a[[a]][[c]]))"}"#,
        ),
        (
            r"(?<whole>(k[[s]][[A]]))",
            JsString::from("Kſa!"),
            "divg",
            0,
            r#"{"matches":["Kſa","Kſa","Kſa"],"index":0,"input":"Kſa!","groups":{"whole":"Kſa"},"indices":[[0,3],[0,3],[0,3]],"indexGroups":{"whole":[0,3]},"lastIndex":3,"source":"(?<whole>(k[[s]][[A]]))"}"#,
        ),
        (
            r"(?<whole>(x[\P{Ll}&&A][[s]]))",
            JsString::from("xAs"),
            "divg",
            0,
            r#"{"match":null,"lastIndex":0}"#,
        ),
        (
            r"(?<whole>(\u{1f600}[[\p{Any}]][[a]]))",
            JsString::from("😀😁a!"),
            "dvy",
            1,
            r#"{"matches":["😀😁a","😀😁a","😀😁a"],"index":0,"input":"😀😁a!","groups":{"whole":"😀😁a"},"indices":[[0,5],[0,5],[0,5]],"indexGroups":{"whole":[0,5]},"lastIndex":5,"source":"(?<whole>(\\u{1f600}[[\\p{Any}]][[a]]))"}"#,
        ),
        (
            r"(?<whole>(\u{1f600}[[\u{1f601}]][[\u{1f602}]]))",
            JsString::from("x😀😁😂!"),
            "dvg",
            0,
            r#"{"matches":["😀😁😂","😀😁😂","😀😁😂"],"index":1,"input":"x😀😁😂!","groups":{"whole":"😀😁😂"},"indices":[[1,7],[1,7],[1,7]],"indexGroups":{"whole":[1,7]},"lastIndex":7,"source":"(?<whole>(\\u{1f600}[[\\u{1f601}]][[\\u{1f602}]]))"}"#,
        ),
        (
            r"(?<whole>(\uD800[[\uD800]][[\uD800]]))",
            JsString::from_code_units(vec![0x41, 0xd800, 0xd800, 0xd800, 0x21]),
            "dvg",
            0,
            r#"{"matches":["\ud800\ud800\ud800","\ud800\ud800\ud800","\ud800\ud800\ud800"],"index":1,"input":"A\ud800\ud800\ud800!","groups":{"whole":"\ud800\ud800\ud800"},"indices":[[1,4],[1,4],[1,4]],"indexGroups":{"whole":[1,4]},"lastIndex":4,"source":"(?<whole>(\\uD800[[\\uD800]][[\\uD800]]))"}"#,
        ),
        (
            r"(?<whole>(a[[a]][[^K]]))",
            JsString::from("aaKaaſ!"),
            "divg",
            0,
            r#"{"matches":["aaſ","aaſ","aaſ"],"index":3,"input":"aaKaaſ!","groups":{"whole":"aaſ"},"indices":[[3,6],[3,6],[3,6]],"indexGroups":{"whole":[3,6]},"lastIndex":6,"source":"(?<whole>(a[[a]][[^K]]))"}"#,
        ),
        (
            r"^(?<whole>(c[[a]][[b]]))$",
            JsString::from("x\r\ncab\u{2028}cd"),
            "dvmg",
            0,
            r#"{"matches":["cab","cab","cab"],"index":3,"input":"x\r\ncab cd","groups":{"whole":"cab"},"indices":[[3,6],[3,6],[3,6]],"indexGroups":{"whole":[3,6]},"lastIndex":6,"source":"^(?<whole>(c[[a]][[b]]))$"}"#,
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
            input.code_units()
        )
        .unwrap();
        row_count += 1;
    }
    assert_eq!(row_count, 10);
    insta::assert_snapshot!(rows);
}

#[test]
fn checked_v_set_pair_literal_state_context_and_consumers() {
    check(
        r"/[a--b][[c]]d/v.test('acd')&&!/[a--b][[c]]d/v.test('bcd')&&!/[a--b][[c]]d/v.test('ac')&&/[[a]][[a]]c/v.exec('aaac').index===1&&!/[a--a][[b]]c/v.test('abc')&&!/[[a]][b--b]c/v.test('abc')",
    );
    check(r#"let marker=0;eval("marker=1; /[a--b][[c]]d/v.test('acd')")&&marker===1"#);
    check(
        r"let g=/[[a]][[a]]c/vg,y=/[[a]][[a]]c/vy,a=g.exec('aaac');a.index===1&&a[0]==='aac'&&g.lastIndex===4&&g.exec('aaac')===null&&g.lastIndex===0&&y.exec('aaac')===null&&y.lastIndex===0&&(y.lastIndex=1,y.exec('aaac')[0]==='aac')&&y.lastIndex===4",
    );
    check(
        r"/[[K]][[s]]a/iv.test('KſA')&&!/[[K]][[s]]a/v.test('KſA')&&!/[[K]][[s]]a/v.test('KsA')&&/[[K]][[s]]a/v.test('Ksa')&&/[[a]][[^K]]s/iv.exec('aKsaſſ')[0]==='aſſ'&&!/[[a]][[^K]]s/iv.test('aKs')",
    );
    check(
        r"/[\P{Ll}&&A][[s]]x/v.test('Asx')&&!/[\P{Ll}&&A][[s]]x/iv.test('Asx')&&/[[\p{ASCII}]][[a]]x/v.test('Aax')&&!/[[\p{ASCII}]][[a]]x/v.test('Kax')",
    );
    check(
        r"let r=/[[\p{Any}]][[\p{Any}]]a/dvy;r.lastIndex=1;let m=r.exec('😀😁a!');m.index===0&&m[0]==='😀😁a'&&m.indices[0][1]===5&&r.lastIndex===5&&/[[😀]][[😁]]😂/v.test('😀😁😂')&&/[[\u{1f600}]][[\u{1f601}]]\uD83D\uDE02/v.test('😀😁😂')",
    );
    check(
        r"let r=/[[\u{1f600}]][[\u{1f601}]]\u{1f602}/dvy;r.lastIndex=1;let m=r.exec('😀😁😂!');m.index===0&&m[0]==='😀😁😂'&&m.indices[0][1]===6&&r.lastIndex===6",
    );
    check(
        r"let r=/[[\uD800]][[\uD800]]\uD800/dv,m=r.exec('A\uD800\uD800\uD800!');m.index===1&&m[0]==='\uD800\uD800\uD800'&&m.indices[0][1]===4&&!/[[a]][[\uD83D]]\uDE00/v.test('a😀')",
    );
    check(
        r"/[[a]][[b]]\[/v.test('ab[')&&/[[a]][[b]]\//v.test('ab/')&&/[[a]][[b]]\//v.source==='[[a]][[b]]\\/'&&/^[[a]][[b]]\$$/v.test('ab$')&&!/^[[a]][[b]]\$$/v.test('ab$\n')",
    );
    check(
        r"let r=/^[[a]][[b]]c$/dvmg,m=r.exec('x\r\nabc\u2028de');m.index===3&&m[0]==='abc'&&m.indices[0][1]===6&&r.lastIndex===6&&r.exec('x\r\nabc\u2028de')===null&&r.lastIndex===0&&!/^[[a]][[b]]c$/v.test('abc\n')",
    );
    check(
        r"let r=/(?<whole>([[a]][[a]]c))/dvg,a=[...'aaac aaac'.matchAll(r)];a.length===2&&a[0][0]==='aac'&&a[0][1]==='aac'&&a[0][2]==='aac'&&a[0].index===1&&a[0].groups.whole==='aac'&&a[0].indices.groups.whole===a[0].indices[1]&&a[0].indices[2][1]===4&&a[1].index===6&&a[1].indices[2][1]===9&&r.lastIndex===0",
    );
    check(
        r"'aaac aaac'.replace(/(?<whole>([[a]][[a]]c))/vg,'<$1:$<whole>>')==='a<aac:aac> a<aac:aac>'&&'aaac'.split(/([[a]][[a]]c)/v).join('|')==='a|aac|'&&'aaac'.match(/[[a]][[a]]c/v)[0]==='aac'",
    );
    check(
        r"let seen='',out='aaac aaac'.replace(/(?<whole>([[a]][[a]]c))/vg,(m,c1,c2,p,text,g)=>{seen+=p+':'+g.whole+';';return 'X'});out==='aX aX'&&seen==='1:aac;6:aac;'",
    );
    check(r"let r=/[[a]][[a]]c/vg;r.lastIndex=4;'aaac'.search(r)===1&&r.lastIndex===4");
    check(
        r"let trace='',r=/[[\p{Any}]][[\p{Any}]]a/vy;r.lastIndex={valueOf(){trace+='i';return 1}};let m=r.exec({toString(){trace+='s';return '😀😁a!'}});trace==='si'&&m.index===0&&m[0]==='😀😁a'&&r.lastIndex===5",
    );
    check(
        r"let seen=0,r=/[[a]][[a]]c/v;r.lastIndex={valueOf(){seen++;return 1}};let m=r.exec('aaac');m.index===1&&seen===1&&typeof r.lastIndex==='object'",
    );
    check(
        r"let r=/[[K]][[s]]a/iv,c=new RegExp(r,'dvy');c.source===r.source&&r.source==='[[K]][[s]]a'&&c.unicodeSets&&!c.ignoreCase&&c.exec('KſA')===null&&c.lastIndex===0&&r.exec('KſA')[0]==='KſA'",
    );
    check(
        r"let a=/[[a]][[a]]c/vy,b=/[[a]][[a]]c/vy;a.lastIndex=1;Object.defineProperty(a,'lastIndex',{writable:false});Object.defineProperty(b,'lastIndex',{writable:false});let x=false,y=false;try{a.exec('aaac')}catch(e){x=e instanceof TypeError}try{b.exec('aaac')}catch(e){y=e instanceof TypeError}x&&y&&a.lastIndex===1&&b.lastIndex===0",
    );
    check(
        r"let marker=0,caught=false;try{eval('marker=1; /[a--a][b&&]c/v;')}catch(e){caught=e instanceof SyntaxError}caught&&marker===0",
    );
    check(r"/[[a]][[b]][[c]]/v.test('abc')");
    check(
        r"let r=/[[a]][[b]][[c]]/v,m=r.exec('abc');m.length===1&&m[0]==='abc'&&m.index===0&&m.input==='abc'&&m.groups===undefined&&m.indices===undefined&&r.source==='[[a]][[b]][[c]]'&&r.lastIndex===0&&!r.test('xbc')&&!r.test('axc')&&!r.test('abx')&&!r.test('ab')",
    );
    for source in [
        r"/[[a]][[b]]cd/v.test('abcd')",
        r"/[a][[b]]c/v.test('abc')",
        r"/[[a]][b]c/v.test('abc')",
        r"/[[a]][[b]]^c/v.test('abc')",
        r"/[[a]]x[[b]]c/v.test('axbc')",
        r"/[[a]][[b]]\bc/v.test('abc')",
        r"/\b[[a]][[b]]c/v.test('abc')",
        r"/[[a]][[b]]c\b/v.test('abc')",
        r"/([[a]])[[b]]c/v.test('abc')",
        r"/[[a]]([[b]])c/v.test('abc')",
        r"/[[a]][[b]](c)/v.test('abc')",
        r"/[[a]]+[[b]]c/v.test('abc')",
        r"/[[a]][[b]]?c/v.test('abc')",
        r"/[[a]][[b]]c+/v.test('abc')",
        r"/(?:[[a]][[b]]c)+/v.test('abc')",
        r"/[a--a][[\q{b}]]c/v.test('abc')",
        r"/[[\q{a}]][b--b]c/v.test('abc')",
        r"/[a--a][[\p{RGI_Emoji}]]c/v.test('abc')",
        r"/[[\p{RGI_Emoji}]][b--b]c/v.test('abc')",
        r"/c[[a]][[b]]d/v.test('cabd')",
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
    realm
        .eval(r"var r=/[[a]][[a]]c/vg,sticky=/[[a]][[a]]a/vy,marker=0")
        .unwrap();
    // Both checked classes succeed while the final literal fails.
    let setup = format!("var text='{}';r.lastIndex=1", "a".repeat(20_000));
    realm.eval(&setup).unwrap();
    assert_eq!(
        realm.eval("sticky.test(text)&&sticky.lastIndex===3"),
        Ok(Value::Boolean(true))
    );
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
fn checked_v_set_pair_literal_results() {
    let mut rows = String::new();
    let mut row_count = 0;
    // Full objects and original spans below are independent manual expectations.
    for (pattern, input, flags, start, expected) in [
        (
            r"(?<whole>([[a]][[a]]c))",
            JsString::from("aaac"),
            "dvg",
            0,
            r#"{"matches":["aac","aac","aac"],"index":1,"input":"aaac","groups":{"whole":"aac"},"indices":[[1,4],[1,4],[1,4]],"indexGroups":{"whole":[1,4]},"lastIndex":4,"source":"(?<whole>([[a]][[a]]c))"}"#,
        ),
        (
            r"(?<whole>([[a]][[a]]c))",
            JsString::from("aaac"),
            "dvy",
            0,
            r#"{"match":null,"lastIndex":0}"#,
        ),
        (
            r"(?<whole>([[a]][[a]]c))",
            JsString::from("aaac"),
            "dvy",
            1,
            r#"{"matches":["aac","aac","aac"],"index":1,"input":"aaac","groups":{"whole":"aac"},"indices":[[1,4],[1,4],[1,4]],"indexGroups":{"whole":[1,4]},"lastIndex":4,"source":"(?<whole>([[a]][[a]]c))"}"#,
        ),
        (
            r"(?<whole>([[K]][[s]]a))",
            JsString::from("KſA!"),
            "divg",
            0,
            r#"{"matches":["KſA","KſA","KſA"],"index":0,"input":"KſA!","groups":{"whole":"KſA"},"indices":[[0,3],[0,3],[0,3]],"indexGroups":{"whole":[0,3]},"lastIndex":3,"source":"(?<whole>([[K]][[s]]a))"}"#,
        ),
        (
            r"(?<whole>([\P{Ll}&&A][[s]]x))",
            JsString::from("Asx"),
            "divg",
            0,
            r#"{"match":null,"lastIndex":0}"#,
        ),
        (
            r"(?<whole>([[\p{Any}]][[\p{Any}]]a))",
            JsString::from("😀😁a!"),
            "dvy",
            1,
            r#"{"matches":["😀😁a","😀😁a","😀😁a"],"index":0,"input":"😀😁a!","groups":{"whole":"😀😁a"},"indices":[[0,5],[0,5],[0,5]],"indexGroups":{"whole":[0,5]},"lastIndex":5,"source":"(?<whole>([[\\p{Any}]][[\\p{Any}]]a))"}"#,
        ),
        (
            r"(?<whole>([[\u{1f600}]][[\u{1f601}]]\u{1f602}))",
            JsString::from("x😀😁😂!"),
            "dvg",
            0,
            r#"{"matches":["😀😁😂","😀😁😂","😀😁😂"],"index":1,"input":"x😀😁😂!","groups":{"whole":"😀😁😂"},"indices":[[1,7],[1,7],[1,7]],"indexGroups":{"whole":[1,7]},"lastIndex":7,"source":"(?<whole>([[\\u{1f600}]][[\\u{1f601}]]\\u{1f602}))"}"#,
        ),
        (
            r"(?<whole>([[\uD800]][[\uD800]]\uD800))",
            JsString::from_code_units(vec![0x41, 0xd800, 0xd800, 0xd800, 0x21]),
            "dvg",
            0,
            r#"{"matches":["\ud800\ud800\ud800","\ud800\ud800\ud800","\ud800\ud800\ud800"],"index":1,"input":"A\ud800\ud800\ud800!","groups":{"whole":"\ud800\ud800\ud800"},"indices":[[1,4],[1,4],[1,4]],"indexGroups":{"whole":[1,4]},"lastIndex":4,"source":"(?<whole>([[\\uD800]][[\\uD800]]\\uD800))"}"#,
        ),
        (
            r"(?<whole>([[a]][[^K]]s))",
            JsString::from("aKsaſſ!"),
            "divg",
            0,
            r#"{"matches":["aſſ","aſſ","aſſ"],"index":3,"input":"aKsaſſ!","groups":{"whole":"aſſ"},"indices":[[3,6],[3,6],[3,6]],"indexGroups":{"whole":[3,6]},"lastIndex":6,"source":"(?<whole>([[a]][[^K]]s))"}"#,
        ),
        (
            r"^(?<whole>([[a]][[b]]c))$",
            JsString::from("x\r\nabc\u{2028}de"),
            "dvmg",
            0,
            r#"{"matches":["abc","abc","abc"],"index":3,"input":"x\r\nabc de","groups":{"whole":"abc"},"indices":[[3,6],[3,6],[3,6]],"indexGroups":{"whole":[3,6]},"lastIndex":6,"source":"^(?<whole>([[a]][[b]]c))$"}"#,
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
            input.code_units()
        )
        .unwrap();
        row_count += 1;
    }
    assert_eq!(row_count, 10);
    insta::assert_snapshot!(rows);
}

#[test]
fn checked_literal_checked_v_set_state_context_and_consumers() {
    check(
        r"/[a--b]c[[d]]/v.test('acd')&&!/[a--b]c[[d]]/v.test('bcd')&&!/[a--b]c[[d]]/v.test('ac')&&/[[a]]a[[c]]/v.exec('aaac').index===1&&!/[a--a]c[[b]]/v.test('acb')&&!/[[a]]c[b--b]/v.test('acb')",
    );
    check(r#"let marker=0;eval("marker=1; /[a--b]c[[d]]/v.test('acd')")&&marker===1"#);
    check(
        r"let g=/[[a]]a[[c]]/vg,y=/[[a]]a[[c]]/vy,a=g.exec('aaac');a.index===1&&a[0]==='aac'&&g.lastIndex===4&&g.exec('aaac')===null&&g.lastIndex===0&&y.exec('aaac')===null&&y.lastIndex===0&&(y.lastIndex=1,y.exec('aaac')[0]==='aac')&&y.lastIndex===4",
    );
    check(
        r"/[[K]]s[[A]]/iv.test('Kſa')&&!/[[K]]s[[A]]/v.test('Kſa')&&!/[[K]]s[[A]]/v.test('Ksa')&&/[[K]]s[[A]]/v.test('KsA')&&/[[a]]a[[^K]]/iv.exec('aaKaaſ')[0]==='aaſ'&&!/[[a]]a[[^K]]/iv.test('aaK')",
    );
    check(
        r"/[\P{Ll}&&A]x[[s]]/v.test('Axs')&&!/[\P{Ll}&&A]x[[s]]/iv.test('Axs')&&/[[\p{ASCII}]]x[[a]]/v.test('Axa')&&!/[[\p{ASCII}]]x[[a]]/v.test('Kxa')",
    );
    check(
        r"let r=/[[\p{Any}]]\u{1f601}[[a]]/dvy;r.lastIndex=1;let m=r.exec('😀😁a!');m.index===0&&m[0]==='😀😁a'&&m.indices[0][1]===5&&r.lastIndex===5&&/[[😀]]😁[[😂]]/v.test('😀😁😂')&&/[[\u{1f600}]]\uD83D\uDE01[[\u{1f602}]]/v.test('😀😁😂')",
    );
    check(
        r"let r=/[[\u{1f600}]]\u{1f601}[[\u{1f602}]]/dvy;r.lastIndex=1;let m=r.exec('😀😁😂!');m.index===0&&m[0]==='😀😁😂'&&m.indices[0][1]===6&&r.lastIndex===6",
    );
    check(
        r"let r=/[[\uD800]]\uD800[[\uD800]]/dv,m=r.exec('A\uD800\uD800\uD800!');m.index===1&&m[0]==='\uD800\uD800\uD800'&&m.indices[0][1]===4&&!/[[\uD83D]]\uDE00[[a]]/v.test('😀a')&&!/[[a]]\uD83D[[\uDE00]]/v.test('a😀')",
    );
    check(
        r"/[[a]]\[[[b]]/v.test('a[b')&&/[[a]]\/[[b]]/v.test('a/b')&&/[[a]]\/[[b]]/v.source==='[[a]]\\/[[b]]'&&/^[[a]]\$[[b]]$/v.test('a$b')&&!/^[[a]]\$[[b]]$/v.test('a$b\n')",
    );
    check(
        r"let r=/^[[a]]b[[c]]$/dvmg,m=r.exec('x\r\nabc\u2028de');m.index===3&&m[0]==='abc'&&m.indices[0][1]===6&&r.lastIndex===6&&r.exec('x\r\nabc\u2028de')===null&&r.lastIndex===0&&!/^[[a]]b[[c]]$/v.test('abc\n')",
    );
    check(
        r"let r=/(?<whole>([[a]]a[[c]]))/dvg,a=[...'aaac aaac'.matchAll(r)];a.length===2&&a[0][0]==='aac'&&a[0][1]==='aac'&&a[0][2]==='aac'&&a[0].index===1&&a[0].groups.whole==='aac'&&a[0].indices.groups.whole===a[0].indices[1]&&a[0].indices[2][1]===4&&a[1].index===6&&a[1].indices[2][1]===9&&r.lastIndex===0",
    );
    check(
        r"'aaac aaac'.replace(/(?<whole>([[a]]a[[c]]))/vg,'<$1:$<whole>>')==='a<aac:aac> a<aac:aac>'&&'aaac'.split(/([[a]]a[[c]])/v).join('|')==='a|aac|'&&'aaac'.match(/[[a]]a[[c]]/v)[0]==='aac'",
    );
    check(
        r"let seen='',out='aaac aaac'.replace(/(?<whole>([[a]]a[[c]]))/vg,(m,c1,c2,p,text,g)=>{seen+=p+':'+g.whole+';';return 'X'});out==='aX aX'&&seen==='1:aac;6:aac;'",
    );
    check(r"let r=/[[a]]a[[c]]/vg;r.lastIndex=4;'aaac'.search(r)===1&&r.lastIndex===4");
    check(
        r"let trace='',r=/[[\p{Any}]]\u{1f601}[[a]]/vy;r.lastIndex={valueOf(){trace+='i';return 1}};let m=r.exec({toString(){trace+='s';return '😀😁a!'}});trace==='si'&&m.index===0&&m[0]==='😀😁a'&&r.lastIndex===5",
    );
    check(
        r"let seen=0,r=/[[a]]a[[c]]/v;r.lastIndex={valueOf(){seen++;return 1}};let m=r.exec('aaac');m.index===1&&seen===1&&typeof r.lastIndex==='object'",
    );
    check(
        r"let r=/[[K]]s[[A]]/iv,c=new RegExp(r,'dvy');c.source===r.source&&r.source==='[[K]]s[[A]]'&&c.unicodeSets&&!c.ignoreCase&&c.exec('Kſa')===null&&c.lastIndex===0&&r.exec('Kſa')[0]==='Kſa'",
    );
    check(
        r"let a=/[[a]]a[[c]]/vy,b=/[[a]]a[[c]]/vy;a.lastIndex=1;Object.defineProperty(a,'lastIndex',{writable:false});Object.defineProperty(b,'lastIndex',{writable:false});let x=false,y=false;try{a.exec('aaac')}catch(e){x=e instanceof TypeError}try{b.exec('aaac')}catch(e){y=e instanceof TypeError}x&&y&&a.lastIndex===1&&b.lastIndex===0",
    );
    check(
        r"let marker=0,caught=false;try{eval('marker=1; /[a--a]c[b&&]/v;')}catch(e){caught=e instanceof SyntaxError}caught&&marker===0",
    );
    check(r"/[[a]][[b]][[c]]/v.test('abc')");
    check(
        r"let r=/[[a]][[b]][[c]]/v,m=r.exec('abc');m.length===1&&m[0]==='abc'&&m.index===0&&m.input==='abc'&&m.groups===undefined&&m.indices===undefined&&r.source==='[[a]][[b]][[c]]'&&r.lastIndex===0&&!r.test('xbc')&&!r.test('axc')&&!r.test('abx')&&!r.test('ab')",
    );
    for source in [
        r"/[a]b[[c]]/v.test('abc')",
        r"/[[a]]b[c]/v.test('abc')",
        r"/[[a]][b][[c]]/v.test('abc')",
        r"/a[[a]]b[[c]]/v.test('aabc')",
        r"/[[a]]b[[c]]d/v.test('abcd')",
        r"/[[a]]bc[[d]]/v.test('abcd')",
        r"/[[a]]^b[[c]]/v.test('abc')",
        r"/[[a]]\bb[[c]]/v.test('abc')",
        r"/\b[[a]]b[[c]]/v.test('abc')",
        r"/[[a]]b[[c]]\b/v.test('abc')",
        r"/([[a]])b[[c]]/v.test('abc')",
        r"/[[a]](b)[[c]]/v.test('abc')",
        r"/[[a]]b([[c]])/v.test('abc')",
        r"/[[a]]+b[[c]]/v.test('abc')",
        r"/[[a]]b+[[c]]/v.test('abc')",
        r"/[[a]]b[[c]]?/v.test('abc')",
        r"/(?:[[a]]b[[c]])+/v.test('abc')",
        r"/[a--a]b[[\q{c}]]/v.test('abc')",
        r"/[[\q{a}]]b[c--c]/v.test('abc')",
        r"/[a--a]b[[\p{RGI_Emoji}]]/v.test('abc')",
        r"/[[\p{RGI_Emoji}]]b[c--c]/v.test('abc')",
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
    realm
        .eval(r"var r=/[[a]]a[[c]]/vg,sticky=/[[a]]a[[a]]/vy,marker=0")
        .unwrap();
    // The first checked class and middle literal succeed; the final class fails.
    let setup = format!("var text='{}';r.lastIndex=1", "a".repeat(20_000));
    realm.eval(&setup).unwrap();
    assert_eq!(
        realm.eval("sticky.test(text)&&sticky.lastIndex===3"),
        Ok(Value::Boolean(true))
    );
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
fn checked_literal_checked_v_set_results() {
    let mut rows = String::new();
    let mut row_count = 0;
    // Full objects and original spans below are independent manual expectations.
    for (pattern, input, flags, start, expected) in [
        (
            r"(?<whole>([[a]]a[[c]]))",
            JsString::from("aaac"),
            "dvg",
            0,
            r#"{"matches":["aac","aac","aac"],"index":1,"input":"aaac","groups":{"whole":"aac"},"indices":[[1,4],[1,4],[1,4]],"indexGroups":{"whole":[1,4]},"lastIndex":4,"source":"(?<whole>([[a]]a[[c]]))"}"#,
        ),
        (
            r"(?<whole>([[a]]a[[c]]))",
            JsString::from("aaac"),
            "dvy",
            0,
            r#"{"match":null,"lastIndex":0}"#,
        ),
        (
            r"(?<whole>([[a]]a[[c]]))",
            JsString::from("aaac"),
            "dvy",
            1,
            r#"{"matches":["aac","aac","aac"],"index":1,"input":"aaac","groups":{"whole":"aac"},"indices":[[1,4],[1,4],[1,4]],"indexGroups":{"whole":[1,4]},"lastIndex":4,"source":"(?<whole>([[a]]a[[c]]))"}"#,
        ),
        (
            r"(?<whole>([[K]]s[[A]]))",
            JsString::from("Kſa!"),
            "divg",
            0,
            r#"{"matches":["Kſa","Kſa","Kſa"],"index":0,"input":"Kſa!","groups":{"whole":"Kſa"},"indices":[[0,3],[0,3],[0,3]],"indexGroups":{"whole":[0,3]},"lastIndex":3,"source":"(?<whole>([[K]]s[[A]]))"}"#,
        ),
        (
            r"(?<whole>([\P{Ll}&&A]x[[s]]))",
            JsString::from("Axs"),
            "divg",
            0,
            r#"{"match":null,"lastIndex":0}"#,
        ),
        (
            r"(?<whole>([[\p{Any}]]\u{1f601}[[a]]))",
            JsString::from("😀😁a!"),
            "dvy",
            1,
            r#"{"matches":["😀😁a","😀😁a","😀😁a"],"index":0,"input":"😀😁a!","groups":{"whole":"😀😁a"},"indices":[[0,5],[0,5],[0,5]],"indexGroups":{"whole":[0,5]},"lastIndex":5,"source":"(?<whole>([[\\p{Any}]]\\u{1f601}[[a]]))"}"#,
        ),
        (
            r"(?<whole>([[\u{1f600}]]\u{1f601}[[\u{1f602}]]))",
            JsString::from("x😀😁😂!"),
            "dvg",
            0,
            r#"{"matches":["😀😁😂","😀😁😂","😀😁😂"],"index":1,"input":"x😀😁😂!","groups":{"whole":"😀😁😂"},"indices":[[1,7],[1,7],[1,7]],"indexGroups":{"whole":[1,7]},"lastIndex":7,"source":"(?<whole>([[\\u{1f600}]]\\u{1f601}[[\\u{1f602}]]))"}"#,
        ),
        (
            r"(?<whole>([[\uD800]]\uD800[[\uD800]]))",
            JsString::from_code_units(vec![0x41, 0xd800, 0xd800, 0xd800, 0x21]),
            "dvg",
            0,
            r#"{"matches":["\ud800\ud800\ud800","\ud800\ud800\ud800","\ud800\ud800\ud800"],"index":1,"input":"A\ud800\ud800\ud800!","groups":{"whole":"\ud800\ud800\ud800"},"indices":[[1,4],[1,4],[1,4]],"indexGroups":{"whole":[1,4]},"lastIndex":4,"source":"(?<whole>([[\\uD800]]\\uD800[[\\uD800]]))"}"#,
        ),
        (
            r"(?<whole>([[a]]a[[^K]]))",
            JsString::from("aaKaaſ!"),
            "divg",
            0,
            r#"{"matches":["aaſ","aaſ","aaſ"],"index":3,"input":"aaKaaſ!","groups":{"whole":"aaſ"},"indices":[[3,6],[3,6],[3,6]],"indexGroups":{"whole":[3,6]},"lastIndex":6,"source":"(?<whole>([[a]]a[[^K]]))"}"#,
        ),
        (
            r"^(?<whole>([[a]]b[[c]]))$",
            JsString::from("x\r\nabc\u{2028}de"),
            "dvmg",
            0,
            r#"{"matches":["abc","abc","abc"],"index":3,"input":"x\r\nabc de","groups":{"whole":"abc"},"indices":[[3,6],[3,6],[3,6]],"indexGroups":{"whole":[3,6]},"lastIndex":6,"source":"^(?<whole>([[a]]b[[c]]))$"}"#,
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
            input.code_units()
        )
        .unwrap();
        row_count += 1;
    }
    assert_eq!(row_count, 10);
    insta::assert_snapshot!(rows);
}

#[test]
fn checked_v_set_class_triple_state_context_and_consumers() {
    check(
        r"/[a--b][[c]][[d]]/v.test('acd')&&!/[a--b][[c]][[d]]/v.test('bcd')&&!/[a--b][[c]][[d]]/v.test('ac')&&/[[a]][[a]][[c]]/v.exec('aaac').index===1&&!/[a--a][[c]][[b]]/v.test('acb')&&!/[[a]][[c]][b--b]/v.test('acb')",
    );
    check(
        r"let r=/[[a]][[b]][[c]]/dvg,m=r.exec('aabc');m.index===1&&m[0]==='abc'&&m.indices[0][0]===1&&m.indices[0][1]===4&&r.lastIndex===4&&!/[[a]][c--c][[b]]/v.test('acb')",
    );
    check(r#"let marker=0;eval("marker=1; /[a--b][[c]][[d]]/v.test('acd')")&&marker===1"#);
    check(
        r"let g=/[[a]][[a]][[c]]/vg,y=/[[a]][[a]][[c]]/vy,a=g.exec('aaac');a.index===1&&a[0]==='aac'&&g.lastIndex===4&&g.exec('aaac')===null&&g.lastIndex===0&&y.exec('aaac')===null&&y.lastIndex===0&&(y.lastIndex=1,y.exec('aaac')[0]==='aac')&&y.lastIndex===4",
    );
    check(
        r"/[[K]][[s]][[A]]/iv.test('Kſa')&&!/[[K]][[s]][[A]]/v.test('Kſa')&&!/[[K]][[s]][[A]]/v.test('Ksa')&&/[[K]][[s]][[A]]/v.test('KsA')&&/[[a]][[a]][[^K]]/iv.exec('aaKaaſ')[0]==='aaſ'&&!/[[a]][[a]][[^K]]/iv.test('aaK')",
    );
    check(
        r"/[\P{Ll}&&A][[x]][[s]]/v.test('Axs')&&!/[\P{Ll}&&A][[x]][[s]]/iv.test('Axs')&&/[[\p{ASCII}]][[x]][[a]]/v.test('Axa')&&!/[[\p{ASCII}]][[x]][[a]]/v.test('Kxa')",
    );
    check(
        r"let r=/[[\p{Any}]][[\u{1f601}]][[a]]/dvy;r.lastIndex=1;let m=r.exec('😀😁a!');m.index===0&&m[0]==='😀😁a'&&m.indices[0][1]===5&&r.lastIndex===5&&/[[😀]][[😁]][[😂]]/v.test('😀😁😂')&&/[[\u{1f600}]][[\u{1f601}]][[\u{1f602}]]/v.test('😀😁😂')",
    );
    check(
        r"let r=/[[\u{1f600}]][[\u{1f601}]][[\u{1f602}]]/dvy;r.lastIndex=1;let m=r.exec('😀😁😂!');m.index===0&&m[0]==='😀😁😂'&&m.indices[0][1]===6&&r.lastIndex===6",
    );
    check(
        r"let r=/[[\uD800]][[\uD800]][[\uD800]]/dv,m=r.exec('A\uD800\uD800\uD800!');m.index===1&&m[0]==='\uD800\uD800\uD800'&&m.indices[0][1]===4&&!/[[\uD83D]][[\uDE00]][[a]]/v.test('😀a')&&!/[[a]][[\uD83D]][[\uDE00]]/v.test('a😀')",
    );
    check(
        r"/[[a]][[\[]][[b]]/v.test('a[b')&&/[[a]][[\/]][[b]]/v.test('a/b')&&/[[a]][[\/]][[b]]/v.source==='[[a]][[\\/]][[b]]'&&/^[[a]][[$]][[b]]$/v.test('a$b')&&!/^[[a]][[$]][[b]]$/v.test('a$b\n')",
    );
    check(
        r"let r=/^[[a]][[b]][[c]]$/dvmg,m=r.exec('x\r\nabc\u2028de');m.index===3&&m[0]==='abc'&&m.indices[0][1]===6&&r.lastIndex===6&&r.exec('x\r\nabc\u2028de')===null&&r.lastIndex===0&&!/^[[a]][[b]][[c]]$/v.test('abc\n')",
    );
    check(
        r"let r=/(?<whole>([[a]][[a]][[c]]))/dvg,a=[...'aaac aaac'.matchAll(r)];a.length===2&&a[0][0]==='aac'&&a[0][1]==='aac'&&a[0][2]==='aac'&&a[0].index===1&&a[0].groups.whole==='aac'&&a[0].indices.groups.whole===a[0].indices[1]&&a[0].indices[2][1]===4&&a[1].index===6&&a[1].indices[2][1]===9&&r.lastIndex===0",
    );
    check(
        r"'aaac aaac'.replace(/(?<whole>([[a]][[a]][[c]]))/vg,'<$1:$<whole>>')==='a<aac:aac> a<aac:aac>'&&'aaac'.split(/([[a]][[a]][[c]])/v).join('|')==='a|aac|'&&'aaac'.match(/[[a]][[a]][[c]]/v)[0]==='aac'",
    );
    check(
        r"let seen='',out='aaac aaac'.replace(/(?<whole>([[a]][[a]][[c]]))/vg,(m,c1,c2,p,text,g)=>{seen+=p+':'+g.whole+';';return 'X'});out==='aX aX'&&seen==='1:aac;6:aac;'",
    );
    check(r"let r=/[[a]][[a]][[c]]/vg;r.lastIndex=4;'aaac'.search(r)===1&&r.lastIndex===4");
    check(
        r"let trace='',r=/[[\p{Any}]][[\u{1f601}]][[a]]/vy;r.lastIndex={valueOf(){trace+='i';return 1}};let m=r.exec({toString(){trace+='s';return '😀😁a!'}});trace==='si'&&m.index===0&&m[0]==='😀😁a'&&r.lastIndex===5",
    );
    check(
        r"let seen=0,r=/[[a]][[a]][[c]]/v;r.lastIndex={valueOf(){seen++;return 1}};let m=r.exec('aaac');m.index===1&&seen===1&&typeof r.lastIndex==='object'",
    );
    check(
        r"let r=/[[K]][[s]][[A]]/iv,c=new RegExp(r,'dvy');c.source===r.source&&r.source==='[[K]][[s]][[A]]'&&c.unicodeSets&&!c.ignoreCase&&c.exec('Kſa')===null&&c.lastIndex===0&&r.exec('Kſa')[0]==='Kſa'",
    );
    check(
        r"let a=/[[a]][[a]][[c]]/vy,b=/[[a]][[a]][[c]]/vy;a.lastIndex=1;Object.defineProperty(a,'lastIndex',{writable:false});Object.defineProperty(b,'lastIndex',{writable:false});let x=false,y=false;try{a.exec('aaac')}catch(e){x=e instanceof TypeError}try{b.exec('aaac')}catch(e){y=e instanceof TypeError}x&&y&&a.lastIndex===1&&b.lastIndex===0",
    );
    check(
        r"let marker=0,caught=false;try{eval('marker=1; /[a--a][[b]][c&&]/v;')}catch(e){caught=e instanceof SyntaxError}caught&&marker===0",
    );
    for source in [
        r"/[a][[b]][[c]]/v.test('abc')",
        r"/[[a]][b][[c]]/v.test('abc')",
        r"/[[a]][[b]][c]/v.test('abc')",
        r"/x[[a]][[b]][[c]]/v.test('xabc')",
        r"/[[a]][[b]][[c]]x/v.test('abcx')",
        r"/[[a]][[b]][[c]][[d]]/v.test('abcd')",
        r"/\b[[a]][[b]][[c]]/v.test('abc')",
        r"/[[a]][[b]][[c]]\b/v.test('abc')",
        r"/[[a]]\b[[b]][[c]]/v.test('abc')",
        r"/[[a]][[b]]\B[[c]]/v.test('abc')",
        r"/[[a]]^[[b]][[c]]/v.test('abc')",
        r"/[[a]][[b]]$[[c]]/v.test('abc')",
        r"/([[a]])[[b]][[c]]/v.test('abc')",
        r"/[[a]]([[b]])[[c]]/v.test('abc')",
        r"/[[a]][[b]]([[c]])/v.test('abc')",
        r"/[[a]]+[[b]][[c]]/v.test('abc')",
        r"/[[a]][[b]]?[[c]]/v.test('abc')",
        r"/[[a]][[b]][[c]]?/v.test('abc')",
        r"/(?:[[a]][[b]][[c]])+/v.test('abc')",
        r"/[a--a][[\q{b}]][[c]]/v.test('abc')",
        r"/[a--a][[b]][[\q{c}]]/v.test('abc')",
        r"/[[a]][b--b][[\q{c}]]/v.test('abc')",
        r"/[[\q{a}]][[b]][c--c]/v.test('abc')",
        r"/[a--a][[b]][[\p{RGI_Emoji}]]/v.test('abc')",
        r"/[[a]][b--b][[\p{RGI_Emoji}]]/v.test('abc')",
        r"/[[\p{RGI_Emoji}]][[b]][c--c]/v.test('abc')",
    ] {
        assert!(
            matches!(
                Realm::default().eval(source),
                Err(Error::Unsupported { .. })
            ),
            "{source}"
        );
    }
    let mut realm = Realm::default();
    realm
        .eval(r"var kept=/[[a]][[a]][[c]]/dvg;kept.lastIndex=1")
        .unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(
        realm.eval(r"let m=kept.exec('aaac');m.index===1&&m[0]==='aac'&&m.indices[0][1]===4&&kept.lastIndex===4"),
        Ok(Value::Boolean(true))
    );
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval(r"var r=/[[a]][[a]][[c]]/vg,sticky=/[[a]][[a]][[a]]/vy,marker=0")
        .unwrap();
    // The first two checked classes succeed; the third checked class fails.
    let setup = format!("var text='{}';r.lastIndex=1", "a".repeat(20_000));
    realm.eval(&setup).unwrap();
    assert_eq!(
        realm.eval("sticky.test(text)&&sticky.lastIndex===3"),
        Ok(Value::Boolean(true))
    );
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
fn checked_v_set_class_triple_results() {
    let mut rows = String::new();
    let mut row_count = 0;
    // Full objects and original spans below are independent manual expectations.
    for (pattern, input, flags, start, expected) in [
        (
            r"(?<whole>([[a]][[a]][[c]]))",
            JsString::from("aaac"),
            "dvg",
            0,
            r#"{"matches":["aac","aac","aac"],"index":1,"input":"aaac","groups":{"whole":"aac"},"indices":[[1,4],[1,4],[1,4]],"indexGroups":{"whole":[1,4]},"lastIndex":4,"source":"(?<whole>([[a]][[a]][[c]]))"}"#,
        ),
        (
            r"(?<whole>([[a]][[a]][[c]]))",
            JsString::from("aaac"),
            "dvy",
            0,
            r#"{"match":null,"lastIndex":0}"#,
        ),
        (
            r"(?<whole>([[a]][[a]][[c]]))",
            JsString::from("aaac"),
            "dvy",
            1,
            r#"{"matches":["aac","aac","aac"],"index":1,"input":"aaac","groups":{"whole":"aac"},"indices":[[1,4],[1,4],[1,4]],"indexGroups":{"whole":[1,4]},"lastIndex":4,"source":"(?<whole>([[a]][[a]][[c]]))"}"#,
        ),
        (
            r"(?<whole>([[K]][[s]][[A]]))",
            JsString::from("Kſa!"),
            "divg",
            0,
            r#"{"matches":["Kſa","Kſa","Kſa"],"index":0,"input":"Kſa!","groups":{"whole":"Kſa"},"indices":[[0,3],[0,3],[0,3]],"indexGroups":{"whole":[0,3]},"lastIndex":3,"source":"(?<whole>([[K]][[s]][[A]]))"}"#,
        ),
        (
            r"(?<whole>([\P{Ll}&&A][[x]][[s]]))",
            JsString::from("Axs"),
            "divg",
            0,
            r#"{"match":null,"lastIndex":0}"#,
        ),
        (
            r"(?<whole>([[\p{Any}]][[\u{1f601}]][[a]]))",
            JsString::from("😀😁a!"),
            "dvy",
            1,
            r#"{"matches":["😀😁a","😀😁a","😀😁a"],"index":0,"input":"😀😁a!","groups":{"whole":"😀😁a"},"indices":[[0,5],[0,5],[0,5]],"indexGroups":{"whole":[0,5]},"lastIndex":5,"source":"(?<whole>([[\\p{Any}]][[\\u{1f601}]][[a]]))"}"#,
        ),
        (
            r"(?<whole>([[\u{1f600}]][[\u{1f601}]][[\u{1f602}]]))",
            JsString::from("x😀😁😂!"),
            "dvg",
            0,
            r#"{"matches":["😀😁😂","😀😁😂","😀😁😂"],"index":1,"input":"x😀😁😂!","groups":{"whole":"😀😁😂"},"indices":[[1,7],[1,7],[1,7]],"indexGroups":{"whole":[1,7]},"lastIndex":7,"source":"(?<whole>([[\\u{1f600}]][[\\u{1f601}]][[\\u{1f602}]]))"}"#,
        ),
        (
            r"(?<whole>([[\uD800]][[\uD800]][[\uD800]]))",
            JsString::from_code_units(vec![0x41, 0xd800, 0xd800, 0xd800, 0x21]),
            "dvg",
            0,
            r#"{"matches":["\ud800\ud800\ud800","\ud800\ud800\ud800","\ud800\ud800\ud800"],"index":1,"input":"A\ud800\ud800\ud800!","groups":{"whole":"\ud800\ud800\ud800"},"indices":[[1,4],[1,4],[1,4]],"indexGroups":{"whole":[1,4]},"lastIndex":4,"source":"(?<whole>([[\\uD800]][[\\uD800]][[\\uD800]]))"}"#,
        ),
        (
            r"(?<whole>([[a]][[a]][[^K]]))",
            JsString::from("aaKaaſ!"),
            "divg",
            0,
            r#"{"matches":["aaſ","aaſ","aaſ"],"index":3,"input":"aaKaaſ!","groups":{"whole":"aaſ"},"indices":[[3,6],[3,6],[3,6]],"indexGroups":{"whole":[3,6]},"lastIndex":6,"source":"(?<whole>([[a]][[a]][[^K]]))"}"#,
        ),
        (
            r"^(?<whole>([[a]][[b]][[c]]))$",
            JsString::from("x\r\nabc\u{2028}de"),
            "dvmg",
            0,
            r#"{"matches":["abc","abc","abc"],"index":3,"input":"x\r\nabc de","groups":{"whole":"abc"},"indices":[[3,6],[3,6],[3,6]],"indexGroups":{"whole":[3,6]},"lastIndex":6,"source":"^(?<whole>([[a]][[b]][[c]]))$"}"#,
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
            input.code_units()
        )
        .unwrap();
        row_count += 1;
    }
    assert_eq!(row_count, 10);
    insta::assert_snapshot!(rows);
}
